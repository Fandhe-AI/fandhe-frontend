# pre-styled-ui スケールトークン設計（radius / shadow / spacing / z-index、イシュー #1423）

## 1. 位置づけ

親イシュー #1421（Phase 0: スタイル調整の共通基盤）の一環として、
`crates/pre-styled-ui/src/theme.rs` の角丸（radii）・影（shadows）・
余白（spaces）の既定スケールを拡充し、新規に重なり順（z-index）トークン
グループを追加する。Phase 1 以降で 107 部品を chakra-ui / Radix Themes
基準へ個別調整する前提として、部品側が載せられる「段階」を先に固定する
ことが目的であり、**部品ソース側のリテラル値をトークン参照へ置換する
一括作業は本イシューのスコープ外**（§5 の棚卸しが後続 Phase の各部品
issue の消し込み対象）。

兄弟イシュー #1422（色トークン）・#1424（focus ring / size）も同じ
`theme.rs` を変更対象とするため、本イシューは `DEFAULT_COLORS` と
color 系 API（`push_color`/`upsert_color`/`color_var`）には一切触れず、
radii/shadows/spaces の定数ブロックと新設 z-index グループのみを変更して
コンフリクト面を最小化する。

## 2. 参照元と一次記録

Radix Themes 側のトークン体系の一次記録は `docs/design/radix-themes-survey.md`
§4.2〜§4.4（radius/spacing/shadow）にあり、本書では書き写さず参照する。
chakra-ui 側は `.agents/skills/chakra-ui/references/theming/design-tokens/`
配下（本リポジトリのローカル skill 参照ファイル）を一次根拠とする。
両者との対比表は `docs/design/radix-themes-survey.md` §5 に追記済み
（本イシューでの拡充分を含む）。

## 3. スケール定義と設計判断

### 3.1 radius（5 段 → 8 段）

既存 5 段（`sm`/`md`/`lg`/`xl`/`full`）は名前・値とも不変。純追加した
3 段:

| name | value | 根拠 |
|------|-------|------|
| `none` | `0` | 意図的な角無し（mark / textarea attached / avatar square）用 |
| `xs` | `0.125rem` | chakra-ui `xs` 相当。密なインライン部品向け |
| `2xl` | `1rem` | chakra-ui `2xl` 相当。大きめの面パネル向け |

chakra-ui の `2xs`/`3xl`/`4xl`、Radix の `thumb`/`factor` は fandhe 部品に
用途がないため見送った（`docs/design/radix-themes-survey.md` §5 参照）。

Radix `--radius-1..6` との対応: `1`≈`xs`、`2`≈`sm`、`3`≈`md`、`4`≈`lg`、
`5`≈`xl`、`6`≈`2xl`。

**部品カテゴリごとの既定段割り当て方針**（後続 Phase の各部品 issue が
適用する。本イシューでは部品ソースを変更しない）:

| カテゴリ | 段 | 対象例 |
|---------|-----|--------|
| 操作部品 | `md` | button / input / select trigger / textarea / checkbox |
| 密なインライン部品 | `sm` | badge / tag / kbd / code / menu item / tooltip / tree item |
| 面パネル | `lg` | card / dialog / drawer / popover / menu content / hover-card / floating-panel |
| pill・円形 | `full` | avatar / switch / radio / steps indicator / color-swatch circle |
| 意図的な角無し | `none` | mark / textarea attached / avatar square |

`50%`（真円）は radio 系のような真円意図が明示的な箇所のみ許容し、
`full`（`9999px`）への統一は行わない。

### 3.2 shadow（4 段 → 6 段）

既存 4 段（`xs`/`sm`/`md`/`lg`）は不変。純追加:

| name | light | dark |
|------|-------|------|
| `xl` | `0 20px 25px rgba(0, 0, 0, 0.2)` | `0 20px 25px rgba(0, 0, 0, 0.5)` |
| `2xl` | `0 25px 50px rgba(0, 0, 0, 0.25)` | `0 25px 50px rgba(0, 0, 0, 0.55)` |

dark 値は既存規則（light 比で不透明度を上げる）を踏襲した。chakra-ui の
`inner`/Radix の inset 相当は見送った（fandhe の inset 用途は現状すべて
ドット・リング・マスク描画であり「影」ではないため）。

**overlay 系の割り当て方針**（後続 Phase 適用）: dropdown 型（menu /
menubar / select / combobox / popover / hover-card / navigation-menu /
floating-panel / date-picker）= `md`（現行の生値
`0 4px 6px rgba(0, 0, 0, 0.15)` は `md` light 値
`0 4px 6px rgba(0, 0, 0, 0.1)` へ寄せる）、dialog / drawer content = `lg`、
toast / action-bar = `lg`、tooltip = `sm`。

**ダーク時の方針の再評価トリガー**: Radix は「ダークでは影を弱め border
で境界を出す」方式だが、fandhe の overlay 系部品は既に全部品が
`border: 1px solid var(--fandhe-color-border)` を持ち境界は border が
担保済みのため、dark 値は既存規則（不透明度を上げる）を維持する。この
判断は色トークン（#1422）確定後に再評価する。

### 3.3 spacing（10 段 → 15 段）

既存 10 段（`1,2,3,4,5,6,8,10,12,16`）は不変。純追加:

| name | value | chakra 相当 |
|------|-------|-------------|
| `0-5` | `0.125rem`（2px） | `0.5` |
| `1-5` | `0.375rem`（6px） | `1.5` |
| `2-5` | `0.625rem`（10px） | `2.5` |
| `20` | `5rem` | `20` |
| `24` | `6rem` | `24` |

[`TokenName`] は `.` を許可しない（CSS custom property 名の一部として
安全な文字集合に制限するため）ため、chakra の `0.5`/`1.5`/`2.5` 相当は
`-` 区切り（`0-5`/`1-5`/`2-5`）で表記する。

4px 格子に載らない既存生値（`0.0625rem`/`0.1rem`/`0.15rem`）は §5 の
棚卸しで「後続 Phase で `0-5` または `1` へ丸める」対象として記載する。
visually-hidden 系の `-1px` はレイアウト外へ意図的に配置するための負値
であり、トークン化対象外とする。

### 3.4 z-index（新規グループ、12 段）

`Theme` にモード非依存の新規グループ `z_indices: Vec<ScaleToken>` を
追加した。出力プレフィックスは既に `toast.rs` が使っていた
`--fandhe-z-index-<name>` を踏襲する
（`crates/docs-site/tests/css_var_scope_prefix.rs` の `collect_fandhe_var_names`
は `--fandhe-` 前方一致で収集するため名前衝突なし）。

既定値（chakra-ui の `hide`〜`max` を参考にした 100 刻み。fandhe の現行
生値からの移行が単調になる順序で選定）:

| name | value | 割り当て予定（後続 Phase） | 現行生値 |
|------|-------|---------------------------|----------|
| `hide` | `-1` | — | — |
| `base` | `0` | link-overlay | `0` |
| `docked` | `10` | 非オーバーレイの浮き要素（sticky header 等） | — |
| `dropdown` | `1000` | menu / menubar / select / combobox / navigation-menu / date-picker positioner | `10` |
| `sticky` | `1100` | action-bar / floating-panel | `900` |
| `popover` | `1200` | popover / hover-card / toggle-tip | `10` |
| `overlay` | `1300` | dialog / drawer backdrop、tour backdrop | `1000`/`1100` |
| `modal` | `1400` | dialog / drawer positioner、tour positioner | `1001`/`1101` |
| `skip-nav` | `1500` | skip-nav | `1200` |
| `toast` | `1600` | toast group | `var(..., 9999)` |
| `tooltip` | `1700` | tooltip / tour spotlight | `1100`/`1101`/`1102` |
| `max` | `2147483647` | 緊急用 | — |

順序は「dropdown < sticky < popover < overlay < modal < skip-nav < toast <
tooltip」を満たす。dialog と drawer は同段（`overlay`/`modal`）とし、
同時表示時の前後関係は DOM 順に委ねる（chakra も同段）。chakra の
`banner`（1200）は fandhe に対応部品がないため見送った。`skip-nav` は
[`TokenName`] 規則（`[a-z0-9][a-z0-9-]*`）に沿いケバブ表記とした。

Radix Themes 自体には `--z-index-*` 相当の公開トークン変数は確認できず
（`docs/design/radix-themes-survey.md` の取得範囲では未確認）、本グループ
は chakra-ui のみを参照元とする。

API は既存 `push_scale`/`upsert_scale`（内部共通ヘルパ）を再利用した
`push_z_index`/`upsert_z_index`、および `var(--fandhe-z-index-<name>)`
参照ヘルパ `z_index_var` を追加した（新たな検証迂回路を作らない）。
`Theme::to_css()` は radii の後・shadows の前ではなく **shadows の後
（`:root` ブロック末尾）** に z-indices を出力する。既存出力のバイト
同一性（z-indices を push しないテーマの出力は #1423 前と同一）を
`crates/pre-styled-ui/tests/theme_css.rs` の回帰テストで固定している。

**toast の正式トークン化**: `toast.rs` は
`decl("z-index", "var(--fandhe-z-index-toast, 9999)")`（未宣言変数への
fallback 付き参照）だったが、`Theme::default()` が必ず宣言するように
なったため `decl("z-index", "var(--fandhe-z-index-toast)")` へ fallback を
除去した。これに伴い `crates/docs-site/tests/css_var_scope_prefix.rs` の
`SHARED_VARS`（免除表）から `--fandhe-z-index-toast` を削除した
（`theme_token_names()` が `Theme::default().to_css()` から自動収集する
ため、免除を残すと `shared_vars_table_has_no_stale_entries` が stale
エントリとして FAIL する）。

## 4. 対象ファイル

| パス | 変更内容 |
|------|----------|
| `crates/pre-styled-ui/src/theme.rs` | `DEFAULT_RADII`/`DEFAULT_SHADOWS`/`DEFAULT_SPACES` への純追加、`DEFAULT_Z_INDICES` 新設、`Theme` へ `z_indices` フィールド、`push_z_index`/`upsert_z_index`/`z_index_var`、`to_css()` 末尾出力、ユニットテスト |
| `crates/pre-styled-ui/src/toast.rs` | `z-index` の fallback 除去 |
| `crates/pre-styled-ui/tests/toast_css.rs` | golden CSS の z-index 行を追随 |
| `crates/pre-styled-ui/tests/theme_css.rs` | z-index の出力構造 golden・var helper 一致・dark ブロック不在の確認 |
| `crates/pre-styled-ui/tests/theme_injection.rs` | `push_z_index`/`upsert_z_index` のインジェクション payload・重複拒否テスト |
| `crates/pre-styled-ui/Cargo.toml` | `0.40.6` → `0.41.0`（公開 API 追加） |
| `crates/docs-site/tests/css_var_scope_prefix.rs` | `SHARED_VARS` から `--fandhe-z-index-toast` を削除 |
| `docs/api/pre-styled-ui-api.md` §4l | 新 API のシグネチャ追記 |
| `docs/design/radix-themes-survey.md` §5 | fandhe 側の段数更新・z-index 行追加 |

## 5. 部品ソースの「トークン外の生の値」棚卸し（後続 Phase の消し込み対象）

以下は本イシュー実装時点（2026-08-31、`crates/pre-styled-ui/src/` 配下）の
実測。Phase 1 以降の各部品 issue がこの一覧を元に生値をトークン参照へ
置換する（本イシューでは部品ソースを変更しない）。再取得コマンドは
各節に記載した `grep` をそのまま使う。

### 5.1 z-index（`decl("z-index", "<生値>")`、`var(--fandhe-z-index-*)` を除く）

再取得: `grep -rn '"z-index"' crates/pre-styled-ui/src/*.rs | grep -v z-index-toast`

25 箇所（`toast.rs` は正式トークン参照へ移行済みのため対象外）:

| 現行生値 | 出現箇所（抜粋、file:line） |
|----------|------------------------------|
| `0` | `link_overlay.rs:51`、`segment_group.rs:139` |
| `1` | `color_picker.rs:203`、`segment_group.rs:184` |
| `10` | `combobox.rs:180`、`date_picker.rs:100`、`hover_card.rs:105`、`menu.rs:156`、`menubar.rs:145,207`、`navigation_menu.rs:112`、`popover.rs:115`、`select.rs:153` |
| `900` | `action_bar.rs:91`、`floating_panel.rs:106` |
| `1000`/`1001` | `dialog.rs:142,151`、`drawer.rs:103,112` |
| `1100` | `tooltip.rs:98`、`toggle_tip.rs:102`、`tour.rs:130` |
| `1101`/`1102` | `tour.rs:143,162` |
| `1200` | `skip_nav.rs:86` |

移行予定は §3.4 の表（`dropdown`/`sticky`/`popover`/`overlay`/`modal`/
`skip-nav`/`tooltip`）を参照。

### 5.2 box-shadow（`decl("box-shadow", "<生値>")`、`var(--fandhe-shadow-*)` を除く）

再取得: `grep -rn '"box-shadow"' crates/pre-styled-ui/src/*.rs`

トークン参照は `shadow-sm`（`card.rs`/`segment_group.rs` の fallback 付き
参照を含む）・`shadow-md`（`toast.rs`）の計 3 箇所のみ、残り約 25 箇所が
生値。うち overlay 系 9 部品（`menu.rs`/`menubar.rs`×2/`select.rs`/
`combobox.rs`/`popover.rs`/`hover_card.rs`/`navigation_menu.rs`/
`floating_panel.rs`/`date_picker.rs`）が同一値
`0 4px 6px rgba(0, 0, 0, 0.15)` を持ち、§3.2 の割り当て方針で `shadow-md`
（`0 4px 6px rgba(0, 0, 0, 0.1)`）へ寄せる予定。`action_bar.rs:106`
（`0 0.25rem 1rem rgba(0, 0, 0, 0.15)`）は `shadow-lg` へ寄せる予定。
残りは影ではなくドット・リング・マスク描画用途（
`checkbox_card.rs`/`radio_card.rs`/`radio_group.rs` のリング、
`color_picker.rs` の `0 0 0 1px`、`tour.rs`/`image_cropper.rs` のマスク
`0 0 0 max(100vw, 100vh)`/`0 0 0 9999px`、`splitter.rs`、`skip_nav.rs`、
`date_input.rs`）であり、トークン化対象外として棚卸しに残す。

### 5.3 border-radius（`decl("border-radius", "<生値>")`、`var(--fandhe-radius-*)` を除く）

再取得: `grep -rn '"border-radius"' crates/pre-styled-ui/src/*.rs`

トークン参照済みが約 63 箇所、生値が残り約 69 箇所。代表的な生値と
対応予定トークン: `0.375rem`（`date_picker.rs`/`menubar.rs`/
`listbox.rs`/`combobox.rs`/`popover.rs`/`hover_card.rs`/`select.rs`/
`signature_pad.rs` 等）→ `radius-md`、`0.25rem`（`tooltip.rs`/
`tree_view.rs`/`menubar.rs`/`listbox.rs`/`combobox.rs`/
`signature_pad.rs`）→ `radius-sm`、`0.5rem`（`dialog.rs`）→ `radius-lg`、
`999px`/`9999px`（`switch.rs`/`angle_slider.rs`/`steps.rs`/
`color_swatch.rs`）→ `radius-full`、`50%`（`radio_group.rs`/
`radio_card.rs`、真円意図が明示的）→ 対象外のまま許容、`0`
（`avatar.rs`/`color_swatch.rs`/`mark.rs`/`textarea.rs`）→ `radius-none`
（本イシューで新設したトークンへの移行候補）。

### 5.4 padding / gap / margin（4px 格子に載る値・載らない値）

再取得: `grep -rn '"padding"\|"gap"\|"margin"' crates/pre-styled-ui/src/*.rs`

トークン参照済み（`var(--fandhe-space-*)`）が約 121 箇所、生値が残り
約 140 箇所。4px 格子上の値（`0.25rem`(1)/`0.5rem`(2)/`0.75rem`(3)/
`1rem`(4)/`2rem`(8)/`3rem`(12)/`4rem`(16)）は既存トークンへの単純寄せで
移行できる。格子外の値（`0.0625rem`(1px、`badge.rs`/`kbd.rs`/`code.rs`
の縦 padding)/`0.125rem`(2px)/`0.375rem`(6px)/`0.625rem`(10px)/`0.1rem`/
`0.15rem`）は本イシューで新設した `0-5`/`1-5`/`2-5` トークンへの移行
候補（`0.0625rem` のみ既存スケールに対応段がなく「後続 Phase で `0-5`
または `1` へ丸める」の判断が必要）。`-1px`（visually-hidden 系の意図的な
負値）はトークン化対象外。部品固有 fallback 付き参照（例:
`var(--fandhe-radio-card-padding, 0.75rem)`）は既存トークン
（`space-3`）と同値の fallback であり、後続 Phase での fallback 除去
検討対象として棚卸しに残す。

## 6. 見送った項目とその理由

| 項目 | 参照元 | 見送り理由 |
|------|--------|------------|
| radius `thumb`/`factor` | Radix | fandhe 部品に対応する用途がない（スライダーの thumb 専用 radius 制御・グローバル倍率は §7 の scaling 同様 Theme provider 前提の機構であり非採用方針〔`docs/policy/intentional-non-adoption.md` §3.24〕と整合） |
| radius `2xs`/`3xl`/`4xl` | chakra-ui | fandhe 部品に用途がない |
| shadow `inner`/inset | chakra-ui/Radix | fandhe の inset 用途はすべてドット・リング・マスク描画であり「影」ではない |
| z-index `banner` | chakra-ui | fandhe に対応する部品がない |
| ダーク時の影を弱め border へ寄せる方式 | Radix | overlay 系部品は既に border で境界を担保済み。色トークン（#1422）確定後に再評価 |

## 7. 再評価トリガー

- 色トークン（#1422）確定後、ダーク時の shadow を「不透明度を上げる」
  から「弱めて border 依存へ寄せる」方式へ切り替えるかどうかを再評価する
  （§3.2）。
- z-index の割り当て（§3.4 の「割り当て予定」列）は後続 Phase の各部品
  issue で実際に適用する際、想定外の重なり順衝突が見つかった場合は
  スケール自体（100 刻み）の見直しを検討する。
