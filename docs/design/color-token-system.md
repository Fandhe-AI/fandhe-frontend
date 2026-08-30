# 色トークン体系（chakra-ui / Radix Themes 基準への整理）

イシュー #1422（親 #1421: Phase 0 スタイル調整の共通基盤）の実装記録。
`crates/pre-styled-ui/src/theme.rs` の `DEFAULT_COLORS` を chakra-ui v3 の
semantic token・Radix Themes の色スケールを参照軸として整理し、未定義参照の
救済とコントラスト回帰検証を行った。

## 1. 目的・スコープ

- 部品側が色リテラル（hex / `rgba(...)`）を書かずに済む semantic 名の段階を
  `Theme::default()` に用意する。
- 未定義のまま部品が参照していた 2 トークン（`--fandhe-color-accent-subtle` /
  `--fandhe-color-focus-ring`）を正式定義し、透明描画の潜在バグを閉じる。
- WCAG コントラスト比を段階ペア単位で機械検証する回帰テストを追加する。
- 「どの部品がどの semantic 名を使うべきか」の対応方針を本文書に残す。

**スコープ外**（申し送り、§6 参照）:

- `crates/pre-styled-ui/src/recipe.rs` の `ColorPalette` 列挙型・
  `palette_declarations` の変更（隣接イシュー #1426/#1678 のスコープ）。
- dialog/drawer の `rgba(0, 0, 0, 0.4)` オーバーレイリテラルの
  `var(--fandhe-color-bg-overlay)` への置換（Phase 1 部品 issue へ申し送り）。
- `box-shadow` の rgba リテラルのトークン化（#1423 shadow トークンのスコープ）。
- Radix Themes 12 段数値トークンの公開トークン化（§6 で非採用と結論）。

## 2. 参照軸の事実記録

### 2.1 chakra-ui v3 semantic token

chakra-ui v3 は `bg.*` / `fg.*` / `border.*` の階層と、colorPalette 単位の
`<palette>.solid` / `.contrast` / `.fg` / `.muted` / `.subtle` /
`.emphasized` / `.focusRing` という役割名を持つ（`chakra-ui` skill 参照）。
本イシューではこの役割名を fandhe の命名規則（`<palette>-subtle` のような
ハイフン区切り）へ写像した。

### 2.2 Radix Themes 12 段 + alpha

Radix Themes は各カラーにつき 1〜12 の数値段階（+ alpha 版）を持ち、
`step-9` を solid、`step-3` を subtle 背景、`step-11` を高コントラスト文字色
として使う設計（詳細は `docs/design/radix-themes-survey.md` §4.1）。

### 2.3 比較観点チェックリストと結論

| 観点 | 現状（#1422 実装前） | 結論（#1422） |
|---|---|---|
| 段数 | 直接命名 29 件、数値スケールなし | 55 件へ拡張。数値スケールは非採用のまま維持（§6） |
| semantic 名 | `bg`/`fg`/`border` の 3 系統 + ステータス色 | `-emphasized`/`-subtle`/`-overlay` 等の役割語を各系統へ拡張 |
| 5 系統（info/success/warning/error/neutral） | neutral（gray）系統が無い | `neutral`/`neutral-emphasized`/`neutral-fg`/`neutral-subtle`/`neutral-muted`/`neutral-fg-subtle` を追加 |
| colorPalette 軸 | `palette_declarations` は base/emphasized/fg の 3 役割のみ | 本イシューでは変更せず、#1678 へ `-subtle`/`-muted`/`-fg-subtle`/`Neutral` 追加を申し送り |
| ダーク定義方式 | `Theme::to_css` のトークン再定義のみ（部品側に `prefers-color-scheme`/`data-theme` の出現なし） | 変更なし。新規 26 件も同方式に従う |
| コントラスト | 機械検証なし | `crates/pre-styled-ui/src/theme.rs` に WCAG 2.x 回帰テストを追加（§7） |

## 3. トークン一覧

`bg` グループ:

| 名前 | light | dark | chakra 対応 | Radix 段 |
|---|---|---|---|---|
| `bg` | `#ffffff` | `#111111` | `bg` | gray 1 |
| `bg-subtle` | `#f7f7f7` | `#1a1a1a` | `bg.subtle` | gray 2 |
| `bg-muted` | `#eeeeee` | `#242424` | `bg.muted` | gray 3 |
| `bg-emphasized`（新規） | `#e2e2e2` | `#2e2e2e` | `bg.emphasized` | gray 4-5 |
| `bg-overlay`（新規） | `rgba(0, 0, 0, 0.4)` | `rgba(0, 0, 0, 0.6)` | 対応なし（Radix `blackA`/`whiteA` 相当） | — |

`fg` グループ（変更なし）: `fg` / `fg-muted` / `fg-subtle`。

`border` グループ:

| 名前 | light | dark | chakra 対応 | Radix 段 |
|---|---|---|---|---|
| `border` | `#d9d9d9` | `#3a3a3a` | `border` | gray 7 |
| `border-muted` | `#e6e6e6` | `#2a2a2a` | `border.muted` | gray 6 |
| `border-subtle`（新規） | `#f0f0f0` | `#202020` | `border.subtle` | gray 6 |
| `border-emphasized`（新規） | `#b3b3b3` | `#525252` | `border.emphasized` | gray 8 |

ステータス系統（`accent` / `info` / `success` / `warning` / `danger` の 5 系統、
各系統同型）。`accent` を代表例として掲載:

| 名前 | light | dark | chakra 対応 | Radix 段 |
|---|---|---|---|---|
| `accent` | `#3182ce` | `#4299e1` | `<p>.solid` | accent 9 |
| `accent-emphasized` | `#2b6cb0` | `#63b3ed` | `<p>.emphasized` | accent 10 |
| `accent-fg` | `#ffffff` | `#0b1720` | `<p>.contrast` | accent contrast |
| `accent-subtle`（新規） | `#ebf8ff` | `#1a2b3d` | `<p>.subtle` | accent 3 |
| `accent-muted`（新規） | `#bee3f8` | `#2c4a66` | `<p>.muted` | accent 5-6 |
| `accent-fg-subtle`（新規） | `#1a4971` | `#90cdf4` | `<p>.fg` | accent 11 |

`info` / `success` / `warning` / `danger` は同じ役割構成で、値は
`crates/pre-styled-ui/src/theme.rs` の `DEFAULT_COLORS` を正とする
（本文書は構造・役割の記録であり、値の唯一の正は theme.rs のソースコード）。

`neutral` グループ（新規、5 系統目）:

| 名前 | light | dark | chakra 対応 | Radix 段 |
|---|---|---|---|---|
| `neutral` | `#718096` | `#a0aec0` | `gray.solid` | gray 9 |
| `neutral-emphasized` | `#4a5568` | `#cbd5e0` | `gray.emphasized` | gray 10 |
| `neutral-fg` | `#ffffff` | `#0b1720` | `gray.contrast` | gray contrast |
| `neutral-subtle` | `#f7f7f7` | `#1a1a1a` | `gray.subtle` | gray 3 |
| `neutral-muted` | `#e2e8f0` | `#2d3748` | `gray.muted` | gray 5-6 |
| `neutral-fg-subtle` | `#333333` | `#d4d4d4` | `gray.fg` | gray 11-12 |

`focus-ring`（新規、単独グループ）: light `#3182ce` / dark `#4299e1`。
`accent` と同値。`date-input.rs` が既に
`var(--fandhe-color-focus-ring, var(--fandhe-color-accent))` の
フォールバック付きで参照していたトークンを正式化した。#1424
（フォーカスリング規約）が値を上書きできる単一の入口として機能する。

`chart-1`〜`chart-6` は変更なし。

## 4. 「どの部品がどの semantic 名を使うべきか」対応方針

- **背景**:
  - 通常のカード/パネル面 → `bg` / `bg-subtle`。
  - hover・選択中の面 → `bg-emphasized`（中立）または各ステータスの
    `<p>-subtle`（意味を持たせたい場合。例: Tree/Menu/Navigation の選択行は
    `accent-subtle`）。
  - dialog/drawer の backdrop → `bg-overlay`（本イシューではトークン定義の
    みで、部品側の置換は Phase 1 部品 issue へ申し送り）。
- **文字**:
  - 本文 → `fg`。補助文字 → `fg-muted`。無効・キャプション → `fg-subtle`。
  - 淡色背景（`<p>-subtle`）上の本文 → `<p>-fg-subtle`。
  - solid 背景（`<p>`/`<p>-emphasized`）上の文字 → `<p>-fg`。
- **枠線**:
  - 既定 → `border`。より目立たせない区切り → `border-muted`/`border-subtle`。
  - hover 時の強調枠線 → `border-emphasized`。
- **ステータス表示**（Alert/Badge/Tag 等）:
  - solid 表示（強い強調） → `<p>` 背景 + `<p>-fg` 文字。
  - subtle 表示（弱い強調、chakra の `variant="subtle"` 相当） → `<p>-subtle`
    背景 + `<p>-fg-subtle` 文字。
- **フォーカスリング**: `focus-ring`（#1424 でフォーカスリング規約が確定する
  までの暫定値は `accent` と同一）。
- **代表部品での適用例**:
  - Button（solid/subtle variant）: `accent`/`accent-fg` と
    `accent-subtle`/`accent-fg-subtle`。
  - Alert: 5 ステータス × subtle 表示（`<p>-subtle`/`<p>-fg-subtle`）。
  - Badge: 5 ステータス + `neutral` の solid/subtle 両対応。
  - Menu/Tree View/Navigation Menu/Toolbar: 選択・hover 行に `accent-subtle`
    （本イシュー #1422 で唯一の既存部品見た目変更、§8 参照）。
  - Dialog/Drawer: `bg-overlay`（backdrop、置換は Phase 1 部品 issue）。
  - Input（date-input 等）: `focus-ring`。

**規則**: 部品 CSS に hex/rgba を直接書かない。必要な段階が本表に無ければ、
まず本表と `DEFAULT_COLORS` を更新してからトークンを追加する
（`docs/policy/intentional-non-adoption.md` の明示性・機械検証可能性の評価軸
に整合）。

## 5. 未定義参照の救済（イシュー #1422 の直接動機）

`--fandhe-color-accent-subtle` は `tree_view.rs` / `menubar.rs` /
`navigation_menu.rs` / `toolbar.rs` がフォールバック無しで `background` に
使用していたが、`Theme::default()` に未登録のため実際には透明で描画されて
いた（潜在バグ）。`--fandhe-color-focus-ring` は `date_input.rs` が
`var(--fandhe-color-accent)` フォールバック付きで参照していた。本イシューで
両方を正式なテーマトークンとして定義し、透明描画のバグを閉じた。

`crates/docs-site/tests/css_var_scope_prefix.rs` の `SHARED_VARS` は、この
2 件が「`Theme::default()` に未登録の共有トークン」として免除登録されて
いたが、正式なテーマトークンになったことで `analyze()` の
`theme_tokens.contains(&name)` 判定が `SHARED_VARS` 判定より先に成立し、
免除エントリとして残すと `shared_vars_table_has_no_stale_entries` が
stale 検知で FAIL する。よって本イシューでこの 2 件を `SHARED_VARS` から
削除した（削除後もこの 2 トークンを複数部品が横断参照する事実は変わらない
が、免除の実現方法が `theme_tokens` 経由に一本化される）。

## 6. 非採用判断

- **Radix Themes の 12 段数値トークンを公開トークンとして採用しない**:
  直接命名構造（semantic 名）を保つ方が AI 保守の明示性・機械検証（scope
  prefix 完全一致契約）と整合するため。**再評価トリガー**: 部品側から
  3 段以上の同一系統グラデーション（例: ホバー→アクティブ→選択の 3 段階
  変化を 1 系統で表現する必要）が要求された時点で再評価する。
- **`palette_declarations`（`recipe.rs`）への `-subtle`/`-muted`/
  `-fg-subtle` 追加、`ColorPalette::Neutral` の追加は本イシューでは行わない**:
  追加すると Button/Badge/Alert/Toast 等 colorPalette を持つ全部品の出力
  CSS が変わり golden テストが広範に壊れるため。#1678（ColorPalette 軸の
  段階数決定）へ提案として申し送る。**追記（イシュー #1678 実装済み）**:
  `ColorPalette::Neutral` を追加し、6 役割版 `palette_scale_declarations`
  を新設した。`palette_declarations`（3 役割）自体は本イシューの判断
  どおり不変（既存部品の golden CSS はバイト不変）。詳細は
  `docs/design/pre-styled-ui-size-and-color-palette-axes.md` 参照。
- **dialog/drawer の `rgba(0, 0, 0, 0.4)` → `bg-overlay` への置換は本
  イシューでは行わない**: 両部品の golden/`contains` テスト更新と見た目
  変更（dark 側の濃度変化）を伴うため。Phase 1 の部品 issue へ申し送る。

## 7. コントラスト閾値と検証手段

検証は `crates/pre-styled-ui/src/theme.rs` の `#[cfg(test)] mod tests` 内、
`relative_luminance` / `contrast_ratio`（std のみ、WCAG 2.x 相対輝度計算）を
用いた 2 テストが担う:

- `body_text_pairs_meet_wcag_4_5_to_1_in_light_and_dark`（本文 4.5:1）:
  `fg`/`bg`、`fg`/`bg-subtle`、`fg`/`bg-muted`、`fg-muted`/`bg`、
  5 ステータス + `neutral` の `<p>-fg-subtle`/`<p>-subtle`。
- `large_text_and_ui_pairs_meet_wcag_3_to_1_in_light_and_dark`（大字・UI
  部品 3:1）: 5 ステータス + `neutral` の `<p>-fg`/`<p>` と
  `<p>-fg`/`<p>-emphasized`、`fg-subtle`/`bg`、各ステータス色/`bg`、
  `focus-ring`/`bg`。

`border`/`bg` の 3:1 は対象外とする。chakra-ui / Radix Themes も gray 6-7
系の枠線色でこの閾値を満たさない設計であり、fandhe の既存 `border`
（`#d9d9d9`/`#3a3a3a`）も同様のため、閾値を無理に満たす値へ変更しない。

実測時点（#1422 実装時）でいずれのペアも閾値未達は無く、既存 29 件の値は
変更していない（§8）。

## 8. 既存部品への影響

- **見た目が変わる唯一の箇所**: `tree_view.rs` / `menubar.rs` /
  `navigation_menu.rs` / `toolbar.rs` の選択・hover 背景が、透明描画
  （未定義変数の実質無効化）から `accent-subtle`（light `#ebf8ff` /
  dark `#1a2b3d`）へ変わる。これは「意図した修正」であり、本イシューが
  閉じる潜在バグそのものである。
- それ以外の 29 件の既存トークンは名前・値ともに変更していないため、
  既存部品の見た目は変わらない。
- `docs-accent-bg`（docs サイト固有トークン）は `accent-subtle` と役割が
  近いが、値・トークン名を変更せず docs 固有のまま維持した（無用な見た目
  変更を避けるため、`crates/docs-site/src/site_theme.rs` のコメント参照）。
