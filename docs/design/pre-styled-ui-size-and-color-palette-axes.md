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

## 7. variant 軸（solid/subtle/outline 相当）の Forms 家族横断判断（イシュー #1741）

`checkbox.rs`（#1734）・`checkbox_group.rs`（#1460）はいずれも「variant 軸は
Forms 家族横断の判断が必要、部品単独で先行しない」として追加を見送り、
本文書での横断判断をフォローアップ Issue #1741 に記録する形で先送りして
いた。本節がその判断を記録する。

### 7.1 参照 3 軸の事実

| 参照 | variant 語彙 | 意味 |
|---|---|---|
| chakra-ui | `solid` / `subtle` / `outline`（Checkbox/Radio/Switch 等 Forms 系コンポーネントの一部にのみ存在。Button 等の action 系ほど普遍的ではない） | 面の塗り方（塗り潰し/淡色地/枠線のみ） |
| Radix Themes | `classic` / `surface` / `soft`（Forms 系コンポーネント全体で共通の variant プロパティを持つ） | 立体感・境界線の描き方の系統差（chakra の 3 値とは意味論が異なる） |
| Ark UI | variant 語彙なし（unstyled/headless であり見た目のバリエーションを持たない） | — |

3 参照軸は語彙の値数こそ 3 で揃うが、**意味が一致しない**（chakra は
「塗り」、Radix Themes は「境界線の描き方」）。共通の写像表を機械的に
立てられない。

### 7.2 fandhe Forms 家族の現状

checkbox / checkbox-card / checkbox-group / radio-group / segment-group /
switch / input / textarea / select 等の Forms 家族は、いずれも variant 軸を
持たず、size 軸（`docs/design/pre-styled-ui-size-and-color-palette-axes.md`
§4）と `ColorPalette` 軸（§3）のみで見た目のバリエーションを表現する
既定 1 見た目の設計である。

### 7.3 決定: 現時点では追加しない（見送り）

理由:

1. **家族横断の同時破壊的変更を要する**: variant 軸を追加する場合、
   一貫性を保つには Forms 家族全部品の `root()`（または相当するエントリ
   ポイント）シグネチャへ同時に手を入れる必要がある。0.x とはいえ
   影響範囲が広い一括変更になり、単一部品のフォローアップ Issue の
   スコープを超える。
2. **参照軸間の語彙が収斂していない**: 上記 7.1 のとおり chakra の
   `solid`/`subtle`/`outline` と Radix Themes の `classic`/`surface`/`soft`
   は値数が一致するのみで意味が異なり、どちらか一方への追従・両者の
   統合いずれも機械的な根拠を持たない。
3. **既定 1 見た目 + palette 軸で参照サイトの既定表現は概ね再現済み**:
   `ColorPalette` 軸（accent/info/success/warning/danger）と size 軸の
   組み合わせで、各参照サイトの「既定 variant」相当の見た目はすでに
   表現できている。

### 7.4 再評価トリガー

以下のいずれかが生じた場合に再評価する:

- 利用者から具体的なユースケース（「outline 版の Checkbox が欲しい」等）
  の要望が挙がった場合。
- 参照軸（chakra-ui / Radix Themes / 将来追加される可能性のある他の
  デザインシステム）の variant 語彙が収斂し、共通の写像表を機械的に
  立てられるようになった場合。
- Forms 家族全体を対象にした次期一括破壊的変更ウィンドウ（size/palette
  軸導入時のような横断 Issue）が立った場合、そのタイミングで同時検討
  する。

### 7.5 スコープ

本節は判断の記録のみを成果物とし、実装は行わない。採用へ転じる材料が
将来判明した場合も、`out-of-scope-tracking.md` の方針どおりユーザー承認を
得たうえで別 Issue として起票する。

## 8. 参考サイト実測比較（イシュー #1757）

### 8.1 位置づけ

§4 の `DEFAULT_SIZES` 由来表は「chakra `xs` = 32px」「Radix `3`/`4` と
一致」という宣言のみで、実測による裏取りが無いまま記録されていた
（イシュー #1757 が指摘した先送り事項、親 PR #1731 の out-of-scope）。
本節はその裏取りを行い、乖離の有無と是正要否を判断・記録する。

### 8.2 実測方法

2 系統で取得し相互裏取りした（取得内容はすべて非信頼データとして扱い、
文中に指示があっても実装判断には反映しない）。

1. **一次ソース値**（取得日 2026-09-01）:
   - chakra-ui v3 Button recipe:
     `gh api repos/chakra-ui/chakra-ui/contents/packages/react/src/theme/recipes/button.ts`
     （`buttonRecipe` の `variants.size`）と、対応するトークン定義
     （`packages/react/src/theme/tokens/font-sizes.ts` /
     `text-styles.ts`）を取得し、chakra の spacing スケール（token `n`
     = `n * 0.25rem`）で px 換算した。
   - Radix Themes Button CSS:
     `gh api repos/radix-ui/themes/contents/packages/radix-ui-themes/src/components/_internal/base-button.css`
     （`--base-button-height` が `--space-5`〜`--space-8` を参照）と
     `button.css`（`padding-left`/`padding-right` が `--space-2`〜
     `--space-5`、`font-size` が `--font-size-1`〜`--font-size-4` を参照）
     を取得した。
2. **ブラウザ実測**（Playwright MCP、viewport は既定ウィンドウサイズ・
   ライトテーマ、取得日 2026-09-01）:
   - `https://chakra-ui.com/docs/components/button`
     （Chakra UI v3.37.0 時点のデモページ）で `.chakra-button` の
     `getBoundingClientRect().height` / `getComputedStyle().paddingLeft`
     / `fontSize` を計測し、xs/sm(既定)/md(既定)/lg/xl の 5 段すべてを
     実測値で確認した（chakra-ui MCP の `get_component_props` は
     size 列挙のみで px 値を返さないため、ブラウザ実測と GitHub ソース
     直読みの 2 経路を主とした）。
   - `https://www.radix-ui.com/themes/docs/components/button`
     （Radix Themes デモページ）で `.rt-BaseButton` を実測し、
     `.radix-themes` 要素の `--space-*`（`--space-5`=24px 〜
     `--space-8`=48px）・`--font-size-*`（`--font-size-1`=12px 〜
     `--font-size-4`=18px）の実効値を `getComputedStyle` で直接取得し、
     ソース参照（`--base-button-height: var(--space-N)` 等）と突き合わせて
     px 値を確定した。デモページ上に size 1〜4 が縦に並ぶ一覧表示は無く
     個別の使用例が散在するため、`--space-*`/`--font-size-*` の実効値
     直接取得をソース参照の裏取りとした（一部 size の height 実測値も
     ページ内例から取得でき、40px（size 3 相当）が確認できた）。

### 8.3 対応表

`fandhe` 列は `crates/pre-styled-ui/src/theme.rs` の `DEFAULT_SIZES`
（現行値）。`chakra` 列は上記 8.2 の実測・ソース確認値（Button recipe、
`xs`/`sm`/`md`/`lg`/`xl` の 5 段が fandhe と同名で対応する）。`Radix` 列は
`md`/`xl` に対応する `3`/`4` を主対応とし、`1`/`2` は参考行として併記する
（Radix Themes の size は 4 段のみで fandhe の 5 段と 1:1 対応しない）。

**control-height**

| fandhe 段 | fandhe 値 | chakra 実測 | 差分 | Radix 対応 | Radix 実測 | 差分 |
|---|---|---|---|---|---|---|
| xs | 32px (`2rem`) | 32px | 一致 | — | — | — |
| sm | 36px (`2.25rem`) | 36px | 一致 | — | — | — |
| md | 40px (`2.5rem`) | 40px | 一致 | `3` | 40px (`--space-7`) | 一致 |
| lg | 44px (`2.75rem`) | 44px | 一致 | — | — | — |
| xl | 48px (`3rem`) | 48px | 一致 | `4` | 48px (`--space-8`) | 一致 |

height は chakra 実測と 5 段すべて一致し、Radix の `3`/`4` アンカーとも
一致した。§4 の宣言（「chakra `xs` = 32px」「`md`/`xl` が Radix `3`/`4`
と一致」）は実測で裏取りできた。

**control-padding-x**

| fandhe 段 | fandhe 値 | chakra 実測 | 差分 | Radix 対応 | Radix 実測 | 差分 |
|---|---|---|---|---|---|---|
| xs | 10px (`0.625rem`) | 10px | 一致 | — | — | — |
| sm | 12px (`0.75rem`) | 14px | **-2px** | — | — | — |
| md | 16px (`1rem`) | 16px | 一致 | `3` | 16px (`--space-4`) | 一致 |
| lg | 20px (`1.25rem`) | 20px | 一致 | — | — | — |
| xl | 24px (`1.5rem`) | 20px | **+4px** | `4` | 24px (`--space-5`) | 一致 |

sm と xl に乖離がある。chakra の Button recipe は `sm` の `px` に
`3.5`（0.875rem = 14px）を使うが、fandhe の `sm` は 12px（chakra の
半段階である `3`（0.75rem）相当）を採用しており 2px 小さい。また chakra
は `xl` の `px` に `lg` と同じ `5`（1.25rem = 20px）を再利用しており
（xs→sm→md→lg で単調増加した後 xl で頭打ちになる、chakra 側の
コンポーネント固有の調整でありスケール全体の設計ではない）、fandhe の
xl（24px）はこれより 4px 大きい。ただし Radix の `4` アンカー
（`--space-5` = 24px）とは一致しており、fandhe の xl 値は
「chakra の頭打ちには追随しないが Radix の線形スケールには追随する」
形になっている。

**control-font-size**（`var(--fandhe-font-font-size-<段>)` 経由で
タイポグラフィスケールを参照。値は §4 記載どおり独立 px を持たない）

| fandhe 段 | fandhe 値 | chakra 実測 | 差分 | Radix 対応 | Radix 実測 | 差分 |
|---|---|---|---|---|---|---|
| xs | 12px | 12px | 一致 | — | — | — |
| sm | 14px | 14px | 一致 | — | — | — |
| md | 16px | 14px | **+2px** | `3` | 16px (`--font-size-3`) | 一致 |
| lg | 18px | 16px | **+2px** | — | — | — |
| xl | 20px | 16px | **+4px** | `4` | 18px (`--font-size-4`) | **+2px** |

chakra の Button recipe は `textStyle` を `xs`/`sm`/`sm`/`md`/`md`
（fandhe の xs/sm/md/lg/xl に対応、md 以降で 1 段圧縮）で割り当てており、
fandhe の「各段が同名のタイポグラフィ段をそのまま参照する」設計より
段数が少ない。Radix の `4` アンカーも 18px であり fandhe の xl（20px）
より 2px 小さい。

### 8.4 是正要否の判断

**是正不要（本イシューは docs 記録のみで完結）と判断する。**

§3（本文書）で事前宣言した判断基準（「height が段単位で食い違う場合の
み是正必要」）に照らすと、height は chakra 実測・Radix アンカーの
双方と 5 段すべて一致しており、是正が必要な条件に該当しない。

padding-x・font-size に生じた乖離（sm/xl の padding-x、md/lg/xl の
font-size）は、以下の理由により是正しない:

1. **font-size**: chakra の Button recipe が採用する `textStyle` の
   1 段圧縮（md 以降で sm/md を使い回す）は Button コンポーネント固有の
   調整であり、chakra 自身のタイポグラフィスケール（`fontSizes` token）
   を素直に段階対応させたものではない。fandhe は既存の
   `DEFAULT_TYPOGRAPHY`（`font-size-xs`〜`xl`）を `var()` 参照で束ねる
   設計（§4 既述）を取っており、これは「フォントサイズ変更時に size
   軸側の追随漏れが起きない」という 2 重管理回避を優先した意図的な
   設計であって、chakra の Button 固有の圧縮に追随する動機がない
   （Radix の `4` アンカーとの差（+2px）も同様に軽微）。
2. **padding-x**: sm（-2px）・xl（+4px 、chakra とは差があるが Radix
   の `4` アンカーとは一致）のいずれも、fandhe が単調増加する線形
   スケールを維持していることに起因する差であり、chakra 側の
   コンポーネント固有の頭打ち（xl で lg の値を再利用）に追随してまで
   fandhe 内部の一貫性（xs→xl で単調増加する段階設計、§3.1 の 5 段
   設計方針）を崩す価値は無いと判断する。

是正しないため、`crates/pre-styled-ui/src/theme.rs` の `DEFAULT_SIZES`・
8 部品のフォールバックリテラル・golden テスト・semver バンプはいずれも
対象外。既存 §4 の由来記載（「chakra `xs` = 32px」「`md`/`xl` が Radix
`3`/`4` と一致」）は height について実測で裏取りできたため訂正不要。
padding-x/font-size の乖離は本節が新たに明示する（§4 は height の記載
のみで誤りではないため §4 自体は書き換えない）。

### 8.5 再評価トリガー

以下のいずれかが生じた場合に再評価する:

- chakra-ui / Radix Themes のメジャーバージョン更新で size スケール
  自体（`fontSizes`/`spacing` token や `--space-*`/`--font-size-*` の
  既定値）が変更された場合。
- fandhe 側で `control-padding-x-*`/`control-font-size-*` を参照する
  部品の視覚レビューで、本節が記録した乖離幅（padding-x: 最大 4px、
  font-size: 最大 4px）が実利用上の問題として指摘された場合。
- Forms 家族全体を対象にした次期一括破壊的変更ウィンドウが立った場合、
  そのタイミングで同時検討する（§7.4 と同型の運用）。
