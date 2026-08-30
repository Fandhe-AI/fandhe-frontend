# pre-styled-ui size / ColorPalette 軸の段階数・名称設計（イシュー #1678）

## 1. 位置づけ

親イシュー #1426（Phase 0: size / colorPalette 軸の判断）の実装分。
`crates/pre-styled-ui/src/recipe.rs` の共通 variant 軸 `Size`（3 段:
`Sm`/`Md`/`Lg`）・`ColorPalette`（5 値: `Accent`/`Info`/`Success`/
`Warning`/`Danger`）の段階数・名称を確定し、対応するテーマトークン
（`crates/pre-styled-ui/src/theme.rs`）を新設する。Phase 1 以降の
107 部品 issue（#1679〜#1681 等）が個別部品へ適用する前段として、
**列挙型・トークンの定義まで**を担う。既存部品の recipe・golden CSS の
更新は本イシューのスコープ外であり、実際に一切変更していない
（§6 の検証で機械確認する）。

兄弟イシュー #1424（focus ring / size 規約、PR #1707）は本イシューと同じ
`recipe.rs`/`theme.rs` を変更対象とする。実装時点で #1707 は未マージの
ため、本実装は `sizes` トークングループを `z_indices` の直後（`focus_ring`
グループの前）に置く。#1707 が先にマージされた場合は `focus_ring` の後ろ
へ並べ替える（コンフリクト解消時に対応する）。

## 2. 参照元の事実

- **chakra-ui v3**: `size` は部品ごとに異なる語彙を持つが、共通の
  t-shirt サイズ（`xs`/`sm`/`md`/`lg`/`xl`、一部 `2xl`）を基準とする。
  Button の既定サイズは `md`。`colorPalette` は任意のパレット名文字列を
  受け付ける仕組みで、既定パレットに `gray`（中立色）を含む。
- **Radix Themes**: `size` は部品ごとに `1`〜`4`（一部 `1`〜`3`）の数値
  連番。`accent`/`gray` の 2 スケールを基本とし、`accentScale`/`grayScale`
  として個別に切り替え可能。
- 3 参照軸の対応表・詳細な出典は `docs/design/radix-themes-survey.md`・
  `.agents/skills/chakra-ui/` ローカル参照ファイルとし、本書では書き写さ
  ない（`pre-styled-ui-scale-tokens.md` と同じ方針）。

## 3. 決定事項

### 3.1 size 軸: 5 段 `Xs` / `Sm` / `Md` / `Lg` / `Xl`

t-shirt 語彙（chakra 系）を採用し、Radix の数値連番（`1`〜`4`）は不採用と
した。根拠:

1. 既存 `Sm`/`Md`/`Lg`・既存 radius/shadow/font-size トークン
   （`xs`〜`2xl`）と語彙が一貫する。
2. PR #1707（focus ring / size 規約）の設計文書 §4 が同じ語彙を予約済み。
3. `docs/design/color-token-system.md` §6 で数値トークンを不採用とした
   判断（semantic 名の明示性・機械検証との整合）と同じ判断軸。

`2xl` は追加しない。共通 enum に載せると全部品が空の段を抱えるため、
必要な部品（Avatar 等）は専用の `VariantValue` 実装で扱う。

**再評価トリガー**: Phase 1 で 3 部品以上が `2xl` 相当のサイズを要求した
時点。

既定値（`#[derive(Default)]`）は実装しない。呼び出し元が明示的に選択する
既存の契約を変えない安全側判断。`#[non_exhaustive]` も付けない（利用者側
の網羅 `match` を将来にわたって禁止するデメリットが、追加の破壊度低減
効果を上回るため）。

enum は値をすべて公開するが、各 styled 部品が実際に登録する段は
レシピごとに異なる（`SlotRecipe::variant` は登録した段のみ CSS を出す。
未登録の段を `size` に指定しても class は付くが宣言は出ない、既存の
挙動のまま）。Phase 1 の保有判定基準は PR #1707 設計文書 §4 に従う。

### 3.2 ColorPalette 軸: 6 値（`Neutral` を追加）

`Neutral`（`value() == "neutral"`）を末尾に追加した。イシュー #1422 で
追加済みの `neutral*` トークン（`neutral`/`neutral-emphasized`/
`neutral-fg`/`neutral-subtle`/`neutral-muted`/`neutral-fg-subtle`）と
1:1 対応する。`Gray` という別名は設けない（テーマ側のトークン名が
`neutral` のため、命名を統一する）。

任意色（利用者定義パレット）を受け付ける軸は作らない。`VariantValue::
value()` は `&'static str` を返す固定語彙の設計であり、動的文字列を
受け付ける入口を追加すると、CSS クラス名・custom property 名の
allowlist 検証（`CssValue`/`TokenName`）とは別系統の検証入口が増える
（REQ-1 相当の迂回経路の増加）。利用者はテーマ側で
`upsert_color("accent", ...)` 等により 6 系統の色値自体を差し替える
ことで独自ブランド色を実現できる（既存経路のまま）。

**再評価トリガー**: 同一画面で 7 系統以上の並列色が必要な部品要求が出た
時点。

`palette_declarations`（3 役割: base/emphasized/fg）は**出力を変えない**
（Button/Badge/Alert/Toast 等の golden を守るため）。代わりに 6 役割版
`palette_scale_declarations(p)` を新設し、`--fandhe-palette`/
`-emphasized`/`-fg`/`-subtle`/`-muted`/`-fg-subtle` を返す。先頭 3 件は
`palette_declarations` と同一順・同一値であることをテストで固定した
（`recipe_css.rs::palette_scale_declarations_prefix_equals_palette_declarations`）。
Phase 1 部品は golden 更新時にこちらへ移行する。

`emphasized` 1 段しかない濃淡差の問題は #1704（イシュー #1422）で
`-subtle`/`-muted`/`-fg-subtle` が追加された時点で解消済みとみなす。
hover/active の使い分け規約自体は #1425 のスコープ。

## 4. size トークン（`theme.rs`）: 新グループ `size`

`DEFAULT_Z_INDICES`/`focus_ring`（#1707）と同型の新スケールグループ
`sizes: Vec<ScaleToken>` を追加し、`push_size`/`upsert_size`/`size_var`
を公開する。既定トークン（`DEFAULT_SIZES`）:

| name | 値 | 備考 |
|------|-----|------|
| `control-height-xs` | `2rem` | chakra `xs` = 32px |
| `control-height-sm` | `2.25rem` | 36px |
| `control-height-md` | `2.5rem` | 40px（Radix `3` と一致） |
| `control-height-lg` | `2.75rem` | 44px |
| `control-height-xl` | `3rem` | 48px（Radix `4` と一致） |
| `control-padding-x-xs` | `0.625rem` | chakra `2.5` |
| `control-padding-x-sm` | `0.75rem` | |
| `control-padding-x-md` | `1rem` | |
| `control-padding-x-lg` | `1.25rem` | |
| `control-padding-x-xl` | `1.5rem` | |
| `control-font-size-xs` | `var(--fandhe-font-font-size-xs)` | 既存タイポグラフィトークン参照 |
| `control-font-size-sm` | `var(--fandhe-font-font-size-sm)` | |
| `control-font-size-md` | `var(--fandhe-font-font-size-md)` | |
| `control-font-size-lg` | `var(--fandhe-font-font-size-lg)` | |
| `control-font-size-xl` | `var(--fandhe-font-font-size-xl)` | |

`control-font-size-*` は独立した px/rem 値を持たず、既存タイポグラフィ
スケール（`DEFAULT_TYPOGRAPHY`）を `var()` 参照で束ねる。フォントサイズ
変更時に size 軸側の追随漏れが起きない構成であり、`var(...)` の値は
`CssValue` の allowlist（英数字・空白・`#` `%` `.` `,` `(` `)` `-` `_`
`+`）を満たす（`theme.rs::size_token_values_stay_within_css_value_allowlist`
で固定）。

既存部品の `padding`/`font-size` リテラル（button md の
`0.5rem 1rem` 等）は変更しない。既存値と size トークンとの差分吸収は
各部品 issue が golden 更新時に行う。

`to_css` の出力順は「colors → spaces → typography → radii → shadows →
z-indices → sizes → motions（#1707 でマージされた focus_ring 由来の
motion トークン）」で**末尾に純追加**する。`Theme::empty()` ベースで
`push_size` を呼ばないテーマの出力はバイト不変
（`theme_css.rs::custom_theme_output_matches_full_snapshot`
がそのまま保証、`empty_theme_without_sizes_omits_size_vars` も参照）。

prefix `--fandhe-size-` は既存の部品 scope 名と衝突しない（`size` という
data-scope は存在しない）。docs-site の `css_var_scope_prefix.rs` は
`Theme::default().to_css()` からテーマトークン名を自動収集するため
登録作業は不要。`SHARED_VARS` には追加しない（参照する部品が本 PR に
無いため。Phase 1 で参照部品が現れた PR が追加する）。

## 5. スコープ外として記録

- size 軸を持たない部品（card/alert/separator/tooltip 等）への size 追加:
  Phase 1 各部品 issue（PR #1707 設計文書 §4(b) の判定基準に従う）。
- `palette_declarations` 自体の 6 役割化と既存部品の移行: #1679〜#1681。
- バンプ運用の最終決定: #1429（本 PR は既定規約〔0.x マイナーバンプ〕に
  従った）。

## 6. 検証

```
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test -p fandhe-frontend-pre-styled-ui
cargo test -p fandhe-frontend-docs-site
cargo run -p xtask -- check-dep-versions
git diff --stat origin/main -- crates/pre-styled-ui/tests \
  | grep -v -E 'recipe_css|recipe_determinism|theme_css'
```

最後の `git diff` は空行のみが出力されることを確認し、既存 golden
テストファイル（button_css.rs 等）を一切変更していないことを機械的に
固定する。
