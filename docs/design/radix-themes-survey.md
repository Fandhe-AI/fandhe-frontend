# Radix Themes 一次調査記録（コンポーネント・トークン体系）

## 1. 本文書のステータスと位置づけ

- 本書はイシュー #936（親 #925、トラッキング #924）の成果物です。
- 取得日: 2026-07-25。取得方法: WebFetch（要約抽出）に加え、構造化された
  一覧が必要な箇所（サイドバーのカテゴリ分類・CSS カスタムプロパティの
  スケール段数）は `curl` で取得した生 HTML を機械的に解析しました
  （§2 参照）。
- 取得元ドメインは `https://www.radix-ui.com`（Radix Themes 公式ドキュメント）
  のみです。
- 消費先はイシュー #937 です。#937 は `docs/design/component-coverage-map.md`
  へ Radix 列を追加する際、本書の §3・§4 を一次情報として参照します。
- **本書は Radix 側の事実の記録に留まり、実装対象（採用・保留・非採用）の
  判定は行いません。** `component-coverage-map.md` §2 の 5 区分への
  割り当ては #937 の責務です。

## 2. 取得方法と根拠

本書のコンポーネント一覧・トークン数値は、モデルの記憶による列挙では
なく、以下の一次情報を機械的に解析して作成しました。

| 取得対象 | URL | 取得方法 |
|---|---|---|
| コンポーネント一覧（サイドバー） | `https://www.radix-ui.com/themes/docs/components` | `curl` で生 HTML 取得後、`<h4 class="rt-Heading...">` 見出しと直後の `<a class="docs-nav-module...DocsNavItem" href="...">` リンクを Python の正規表現で対応付け、見出し単位にグルーピング |
| 各コンポーネントの概要文 | `https://www.radix-ui.com/themes/docs/components/<slug>`（56 ページ全件） | `curl` で生 HTML 取得後、`<meta name="description" content="...">` を抽出 |
| Color トークン | `https://www.radix-ui.com/themes/docs/theme/color` | `curl` 取得後、`--accent-*` / `--gray-*` CSS カスタムプロパティ名を正規表現で列挙 |
| Radius トークン | `https://www.radix-ui.com/themes/docs/theme/radius` | 同上、`--radius-*` を列挙 |
| Spacing トークン | `https://www.radix-ui.com/themes/docs/theme/spacing` | 同上、`--space-*` を列挙 |
| Shadow トークン | `https://www.radix-ui.com/themes/docs/theme/shadows` | 同上、`--shadow-*` を列挙 |
| Typography トークン | `https://www.radix-ui.com/themes/docs/theme/typography` | 同上、`--font-size-*` / `--font-weight-*` 等を列挙 |
| Breakpoints トークン | `https://www.radix-ui.com/themes/docs/theme/breakpoints` | 同上、既知の値（`520px`/`768px`/`1024px`/`1280px`/`1640px`）の出現を確認 |
| Cursors トークン | `https://www.radix-ui.com/themes/docs/theme/cursors` | 同上、`--cursor-*` を列挙 |
| Theme prop（overview） | `https://www.radix-ui.com/themes/docs/theme/overview` | `accentColor` / `grayColor` / `panelBackground` / `radius` / `scaling` 文字列の出現確認 |
| Theme prop（appearance / scaling 値） | `https://www.radix-ui.com/themes/docs/components/theme` | `appearance` / `light` / `dark` / `90%`〜`110%` 文字列の出現確認 |

**完全性の判定条件（V2 相当）**: 本書 §3 に列挙した `/themes/docs/components/*`
の slug 集合と、`https://www.radix-ui.com/themes/docs/components` から
`grep -oE '/themes/docs/components/[a-z-]+'` で再抽出した集合との差分が
空であることをもって、コンポーネント一覧の完全性を確認できます。

**取得内容の限界**: SSR された HTML には一部のトークン（line-height の
CSS カスタムプロパティ名、コンポーネント別セマンティックロール説明文
など）が含まれず、クライアント側 JS 実行後にのみ描画される場合があり
ました。該当箇所は §4 に「SSR では未確認」と明記します。

## 3. コンポーネント一覧（カテゴリ別・出典 URL 付き）

Radix Themes のドキュメントサイドバーは Overview / Theme / Layout /
Typography / Components / Utilities の 6 見出しで構成されており、
`/themes/docs/components/*` 配下のページ（実測 56 件）は Layout /
Typography / Components / Utilities の 4 見出しに分散しています
（Overview・Theme 見出しはコンポーネントページではなく概念解説・
トークンページです）。

### 3.1 Layout（参照対象外、§6 参照）

| 名前 | slug | 出典 URL | 概要 |
|---|---|---|---|
| Box | `box` | `/themes/docs/components/box` | Fundamental layout building block. |
| Flex | `flex` | `/themes/docs/components/flex` | Component for creating flex layouts. |
| Grid | `grid` | `/themes/docs/components/grid` | Component for creating grid layouts. |
| Container | `container` | `/themes/docs/components/container` | Constrains the maximum width of page content. |
| Section | `section` | `/themes/docs/components/section` | Denotes a section of page content. |

**注記**: イシュー #936 本文の仮説では Layout カテゴリに Inset を含めて
いましたが、実際のサイドバー構造では Inset は Layout ではなく
Components カテゴリに分類されていました（§3.3）。本書は取得結果
（サイドバー構造）を正としてこの差異を記録します。Inset は §6 の
参照対象外リストには含めません（区分判定自体は #937 の責務のため、
本書は Radix 側の分類のみを記録します）。

### 3.2 Typography（参照対象）

| 名前 | slug | 出典 URL | 概要 |
|---|---|---|---|
| Text | `text` | `/themes/docs/components/text` | Foundational text primitive. |
| Heading | `heading` | `/themes/docs/components/heading` | Semantic heading element. |
| Blockquote | `blockquote` | `/themes/docs/components/blockquote` | Block-level quotation from another source. |
| Code | `code` | `/themes/docs/components/code` | Marks text to signify a short fragment of computer code. |
| Em | `em` | `/themes/docs/components/em` | Marks text to stress emphasis. |
| Kbd | `kbd` | `/themes/docs/components/kbd` | Represents keyboard input or a hotkey. |
| Link | `link` | `/themes/docs/components/link` | Semantic element for navigation between pages. |
| Quote | `quote` | `/themes/docs/components/quote` | Short inline quotation. |
| Strong | `strong` | `/themes/docs/components/strong` | Marks text to signify strong importance. |

### 3.3 Components（参照対象）

| 名前 | slug | 出典 URL | 概要 |
|---|---|---|---|
| Alert Dialog | `alert-dialog` | `/themes/docs/components/alert-dialog` | Modal confirmation dialog that interrupts the user and expects a response. |
| Aspect Ratio | `aspect-ratio` | `/themes/docs/components/aspect-ratio` | Displays content within a desired ratio. |
| Avatar | `avatar` | `/themes/docs/components/avatar` | Profile picture, user initials or fallback icon. |
| Badge | `badge` | `/themes/docs/components/badge` | Stylized badge element. |
| Button | `button` | `/themes/docs/components/button` | Trigger an action or event, such as submitting a form or displaying a dialog. |
| Callout | `callout` | `/themes/docs/components/callout` | Short message to attract user's attention. |
| Card | `card` | `/themes/docs/components/card` | Container that groups related content and actions. |
| Checkbox | `checkbox` | `/themes/docs/components/checkbox` | Base input element to toggle an option on and off. |
| Checkbox Group | `checkbox-group` | `/themes/docs/components/checkbox-group` | Set of interactive buttons where multiple options can be selected at a time. |
| Checkbox Cards | `checkbox-cards` | `/themes/docs/components/checkbox-cards` | Set of interactive cards where multiple options can be selected at a time. |
| Context Menu | `context-menu` | `/themes/docs/components/context-menu` | Menu representing a set of actions, displayed at the point of right click or long press. |
| Data List | `data-list` | `/themes/docs/components/data-list` | Displays metadata as a list of key-value pairs. |
| Dialog | `dialog` | `/themes/docs/components/dialog` | Modal dialog window displayed above the page. |
| Dropdown Menu | `dropdown-menu` | `/themes/docs/components/dropdown-menu` | Menu representing a set of actions, triggered by a button. |
| Hover Card | `hover-card` | `/themes/docs/components/hover-card` | For sighted users to preview content available behind a link. |
| Icon Button | `icon-button` | `/themes/docs/components/icon-button` | Button designed specifically for usage with a single icon. |
| Inset | `inset` | `/themes/docs/components/inset` | Applies a negative margin to allow content to bleed into the surrounding container. |
| Popover | `popover` | `/themes/docs/components/popover` | Floating element for displaying rich content, triggered by a button. |
| Progress | `progress` | `/themes/docs/components/progress` | Displays a progress bar related to a task. |
| Radio | `radio` | `/themes/docs/components/radio` | Standalone radio button that can be used in any layout. |
| Radio Group | `radio-group` | `/themes/docs/components/radio-group` | Set of interactive radio buttons where only one can be selected at a time. |
| Radio Cards | `radio-cards` | `/themes/docs/components/radio-cards` | Set of interactive cards where only one can be selected at a time. |
| Scroll Area | `scroll-area` | `/themes/docs/components/scroll-area` | Custom-styled scrollable area using native functionality. |
| Segmented Control | `segmented-control` | `/themes/docs/components/segmented-control` | Toggle buttons for switching between different values or views. |
| Select | `select` | `/themes/docs/components/select` | Displays a list of options for the user to pick from—triggered by a button. |
| Separator | `separator` | `/themes/docs/components/separator` | Visually or semantically separates content. |
| Skeleton | `skeleton` | `/themes/docs/components/skeleton` | Replaces content with same shape placeholder that indicates a loading state. |
| Slider | `slider` | `/themes/docs/components/slider` | Provides user selection from a range of values. |
| Spinner | `spinner` | `/themes/docs/components/spinner` | Displays an animated loading indicator. |
| Switch | `switch` | `/themes/docs/components/switch` | Toggle switch alternative to the checkbox. |
| Table | `table` | `/themes/docs/components/table` | Semantic table element for presenting data. |
| Tabs | `tabs` | `/themes/docs/components/tabs` | Set of content sections to be displayed one at a time. |
| Tab Nav | `tab-nav` | `/themes/docs/components/tab-nav` | Navigation menu with links styled as tabs. |
| Text Area | `text-area` | `/themes/docs/components/text-area` | Captures multi-line user input. |
| Text Field | `text-field` | `/themes/docs/components/text-field` | Captures user input with an optional slot for buttons and icons. |
| Tooltip | `tooltip` | `/themes/docs/components/tooltip` | Floating element that provides a control with contextual information via pointer or focus. |

### 3.4 Utilities（Theme のみ参照対象外、§6 参照）

| 名前 | slug | 出典 URL | 概要 | 区分メモ |
|---|---|---|---|---|
| Accessible Icon | `accessible-icon` | `/themes/docs/components/accessible-icon` | Makes icons accessible by adding a label. | 参照対象 |
| Portal | `portal` | `/themes/docs/components/portal` | Renders a React subtree in a different part of the DOM. | 参照対象（React 固有の DOM ポータル API。`intentional-non-adoption.md` §3.23 は headless-ui 側の JS ランタイム固有 utilities の非採用を扱うが、本書は Radix 側事実の記録に留め判定は行わない） |
| Reset | `reset` | `/themes/docs/components/reset` | Removes default browser styles from any component. | 参照対象 |
| Slot | `slot` | `/themes/docs/components/slot` | Merges its props onto its immediate child. | 参照対象 |
| Theme | `theme` | `/themes/docs/components/theme` | Wraps all or part of a React tree to provide theme configuration. | **参照対象外**（§3.5・§6） |
| Visually Hidden | `visually-hidden` | `/themes/docs/components/visually-hidden` | Hides content from the screen in an accessible way. | 参照対象 |

### 3.5 Theme（トークン設定ページ、参照対象外は Theme *コンポーネント* のみ）

「Theme」という見出しは 2 つの異なる対象を指すため区別します。

- サイドバー見出し「Theme」配下の 9 ページ（`/themes/docs/theme/*`）は
  トークン設定の**解説ページ**であり、コンポーネントではありません
  （§4 で内容を記録）。
- サイドバー見出し「Utilities」配下の `Theme` コンポーネント
  （`/themes/docs/components/theme`）は React ツリーへテーマ設定を注入する
  **provider コンポーネント**です。これが §6 で参照対象外とする対象です。

| ページ | 出典 URL |
|---|---|
| Overview | `/themes/docs/theme/overview` |
| Color | `/themes/docs/theme/color` |
| Dark mode | `/themes/docs/theme/dark-mode` |
| Typography | `/themes/docs/theme/typography` |
| Spacing | `/themes/docs/theme/spacing` |
| Breakpoints | `/themes/docs/theme/breakpoints` |
| Radius | `/themes/docs/theme/radius` |
| Shadows | `/themes/docs/theme/shadows` |
| Cursors | `/themes/docs/theme/cursors` |

## 4. トークン体系（Radix 側の事実記録）

### 4.1 Color

- CSS カスタムプロパティのスケールは `--accent-1`〜`--accent-12`
  （12 段）と `--gray-1`〜`--gray-12`（12 段）の 2 系統が存在します
  （出典: `/themes/docs/theme/color`）。
- それぞれにアルファ版のバリアント `--accent-a1`〜`--accent-a12` /
  `--gray-a1`〜`--gray-a12`（各 12 段）が併存し、accent/gray とも
  実質 24 変数（不透明 12 + アルファ 12）です。
- accent（アクセントカラー）と gray（グレースケール）はいずれも
  Theme コンポーネントの `accentColor` / `grayColor` prop で切り替え
  可能な semantic なロール名です（出典: `/themes/docs/theme/overview`）。
- セマンティックな役割説明文（例: 「Step 1-2 は背景用」等の用途別
  ガイドテキスト）は SSR された HTML には含まれておらず、本書では
  未確認としています（§2「取得内容の限界」参照）。

### 4.2 Radius

CSS 変数スケールと Theme prop の選択肢は基数が異なるため分けて記録
します（出典: `/themes/docs/theme/radius`）。

- **CSS 変数スケール**: `--radius-1`〜`--radius-6`（6 段）に加え、
  `--radius-full`（完全な丸み）・`--radius-thumb`（スライダー等の
  つまみ専用）・`--radius-factor`（スケール全体の倍率調整用）が存在
  します。
- **Theme の `radius` prop の選択肢**: `none` / `small` / `medium` /
  `large` / `full` の文字列がページ内に出現しました（5 択、CSS 変数
  スケールとは別の語彙）。

### 4.3 Spacing

- `--space-1`〜`--space-9`（9 段）の CSS カスタムプロパティが存在
  します（出典: `/themes/docs/theme/spacing`）。

### 4.4 Shadow

- `--shadow-1`〜`--shadow-6`（6 段）の CSS カスタムプロパティが存在
  します（出典: `/themes/docs/theme/shadows`）。ライト/ダークモードは
  変数値自体が `Theme` の `appearance` 状態に応じて切り替わる方式で
  あり、Radix 側は shadow 用に light/dark 専用の別名変数を持たず、
  同一の `--shadow-N` 変数名で値だけが再定義される構成です（`Theme`
  コンポーネントのラップにより値が切り替わる。§4.6 参照）。

### 4.5 Typography / Breakpoints / Cursors

- **Typography**（出典: `/themes/docs/theme/typography`）: `--font-size-1`
  〜`--font-size-9`（9 段）、`--font-weight-light` / `-regular` /
  `-medium` / `-bold`（4 種類の名前付きウェイト）を確認しました。
  `--line-height-*` / `--letter-spacing-*` という命名の CSS 変数は
  SSR された HTML からは検出できませんでした（`--heading-letter-spacing`
  という見出し専用変数のみ検出。§2「取得内容の限界」参照）。
- **Breakpoints**（出典: `/themes/docs/theme/breakpoints`）: ページ内に
  `520px` / `768px` / `1024px` / `1280px` / `1640px` の値が出現し、
  Radix の公称ブレークポイント（`initial` / `xs` / `sm` / `md` / `lg`
  / `xl` の 6 段階）と符合します。
- **Cursors**（出典: `/themes/docs/theme/cursors`）: `--cursor-button` /
  `-checkbox` / `-disabled` / `-link` / `-menu-item` / `-radio` /
  `-slider-thumb` / `-slider-thumb-active` / `-switch`（9 種類）の
  CSS カスタムプロパティを確認しました。

### 4.6 Theme prop（provider の設定値、トークンスケールではない）

`Theme` コンポーネント（§3.4・§3.5）の prop は CSS カスタムプロパティの
スケールではなく、React コンポーネントに渡す設定値です。トークン節
（§4.1〜§4.5）と混同しないため独立させて記録します。

- `appearance`: `light` / `dark` の文字列が `/themes/docs/components/theme`
  ページ内に出現しました。
- `accentColor` / `grayColor`: §4.1 のカラースケール名（accent/gray）を
  選択する prop として `/themes/docs/theme/overview` に出現しました。
- `radius`: §4.2 の 5 択（`none`/`small`/`medium`/`large`/`full`）を
  選択する prop です。
- `scaling`: `90%` / `95%` / `100%` / `105%` / `110%` の 5 値が
  `/themes/docs/components/theme` ページ内に出現しました。全トークンの
  スケール倍率を一括調整する prop です。
- `panelBackground`: `/themes/docs/theme/overview` に出現を確認しました
  （値の選択肢は本書の取得範囲では未確認）。

## 5. `crates/pre-styled-ui/src/theme.rs` との対比表

`theme.rs`（816 行、2026-07-25 時点）の実測値です。

| 軸 | Radix Themes | fandhe pre-styled-ui `theme.rs` | 粒度差・欠落 |
|---|---|---|---|
| color スケール段数 | accent 12 段 + gray 12 段（各アルファ版 12 段を含めると実質 24 変数 ×2 系統、§4.1） | `DEFAULT_COLORS`（theme.rs:274）29 件（semantic 23 + chart 6、`(name, light, dark)` の 3 つ組） | Radix は数値スケール（1〜12）+ アルファ版という機械的な段階構造。fandhe は `bg`/`fg`/`accent`/`info`/`success`/`warning`/`danger` 等の semantic 名 + chart 系列 6 色という直接命名構造で、数値スケールを持たない |
| color の accent/gray 切り替え | Theme の `accentColor`/`grayColor` prop で動的に切り替え可能（§4.6） | 切り替え機構なし（`accent`/`accent-emphasized`/`accent-fg` は固定値） | fandhe は単一アクセント色の直接値のみで、Radix のような複数カラースキームからの選択機構を持たない |
| radius（CSS 変数スケール） | `--radius-1`〜`--radius-6`（6 段）+ `full`/`thumb`/`factor`（§4.2） | `DEFAULT_RADII`（theme.rs、イシュー #1423 で 5 段 → 8 段へ拡充）: `none`/`xs`/`sm`/`md`/`lg`/`xl`/`2xl`/`full` | Radix は数値スケール、fandhe は t-shirt サイズ命名。段数は fandhe が同等以上（Radix 6 vs fandhe 8。`none`/`2xl` は chakra-ui 側の命名を採用）だが `thumb`/`factor` 専用トークンは fandhe に対応がない（#1423 でも見送り、`pre-styled-ui-scale-tokens.md` 参照） |
| radius（Theme prop の選択肢） | `none`/`small`/`medium`/`large`/`full`（5 択、§4.2） | 対応なし（fandhe は provider コンポーネントを持たない） | fandhe には Theme 相当の実行時 prop 切り替え機構自体が存在しない（§6） |
| spacing 段数 | `--space-1`〜`--space-9`（9 段、§4.3） | `DEFAULT_SPACES`（theme.rs、イシュー #1423 で 10 段 → 15 段へ拡充）: `0-5,1,1-5,2,2-5,3,4,5,6,8,10,12,16,20,24`（`.` を使えない [`TokenName`] 制約から chakra の `0.5`/`1.5`/`2.5` 相当を `-` 区切りで表記） | fandhe は 8 を超えると `10,12,16,20,24` と間隔が広がる非等比スケール。Radix は 1〜9 の連番でスケール自体は等比的（実際の px 値は非線形）。段数は fandhe が上回る（9 vs 15） |
| shadow 段数・light/dark 表現 | `--shadow-1`〜`--shadow-6`（6 段）。light/dark は同一変数名で値が `Theme` の `appearance` に応じ再定義される方式（§4.4） | `DEFAULT_SHADOWS`（theme.rs、イシュー #1423 で 4 段 → 6 段へ拡充）: `xs`/`sm`/`md`/`lg`/`xl`/`2xl`。`DualModeToken` として light/dark 値をトークン定義時に組で保持し、`to_css` が `:root[data-theme="dark"]` 等のブロックへ出力 | 段数は同数（6 vs 6）。light/dark の表現方式は両者とも「同名変数の値差し替え」という点で類似するが、fandhe は Rust の型（`DualModeToken`）でモデル化し出力側で決定的にブロック分けする点が異なる。fandhe は #1423 でも dark 値を「light 比で不透明度を上げる」既存規則を踏襲しており、Radix の「ダークでは影を弱め border で境界を出す」方式への合流は色トークン確定後の再評価事項（`pre-styled-ui-scale-tokens.md` 参照） |
| z-index 段数 | Radix Themes 自体には `--z-index-*` 相当の公開トークン変数は確認できず（本書の取得範囲では未確認） | `DEFAULT_Z_INDICES`（theme.rs、イシュー #1423 で新設）12 件: `hide`/`base`/`docked`/`dropdown`/`sticky`/`popover`/`overlay`/`modal`/`skip-nav`/`toast`/`tooltip`/`max`（chakra-ui の `hide`〜`max` 相当の 100 刻みスケールを参考にした） | Radix 側に対応するトークングループが確認できないため直接比較は保留（欠落ではなく未確認として記録）。fandhe は chakra-ui を参照元として新設した |
| typography（font-size） | `--font-size-1`〜`--font-size-9`（9 段、§4.5） | `DEFAULT_TYPOGRAPHY`（theme.rs:364）内 `font-size-xs`〜`font-size-4xl` の 8 段 | 段数はほぼ同等（9 vs 8）。命名は Radix が数値連番、fandhe が t-shirt サイズ |
| typography（font-weight） | `light`/`regular`/`medium`/`bold`（4 種、§4.5） | `font-weight-normal`/`-medium`/`-semibold`/`-bold`（4 種） | 件数は同じ（4）。Radix は `regular`、fandhe は `normal` と命名が異なり、fandhe のみ `semibold` を持つ（Radix にセミボールドの単独ステップはない） |
| typography（line-height） | SSR からは変数名を検出できず（§4.5、取得の限界） | `line-height-tight`/`-normal`/`-relaxed`（3 種） | Radix 側の対応変数名を本書の取得範囲では確認できなかったため、直接比較は保留（欠落ではなく未確認として記録） |
| breakpoints | `initial`/`xs`/`sm`/`md`/`lg`/`xl`（6 段、§4.5） | `theme.rs` に対応するフィールド・定数なし | fandhe 側はブレークポイントをトークンとして保持していない（欠落として記録） |
| cursors | 9 種の用途別カーソル変数（§4.5） | `theme.rs` に対応するフィールド・定数なし | fandhe 側はカーソルをトークンとして保持していない（欠落として記録） |
| 変数プレフィックス | `--` 直下（`--accent-*`/`--space-*`/`--radius-*`/`--shadow-*`/`--font-*`/`--cursor-*`。名前空間プレフィックスなし） | `--fandhe`（`VAR_PREFIX`、theme.rs:212）。出力名は `--fandhe-color-*`/`-space-*`/`-font-*`/`-radius-*`/`-shadow-*` | fandhe は他クレートとの名前空間衝突回避のため固定プレフィックスを持つ点が異なる |
| ダークモードの配布経路 | `Theme` コンポーネントの `appearance` prop（React ツリーのラップ、§4.6） | `:root` の `data-theme` 属性 + `@media (prefers-color-scheme: dark)`（`Theme::to_css`、theme.rs:497。`:root` → `:root[data-theme="light"]` → `@media (prefers-color-scheme: dark) { :root:not([data-theme="light"]) }` → `:root[data-theme="dark"]` の 4 ブロックを決定順で出力） | Radix は React provider 経由、fandhe は属性セレクタ + メディアクエリのみで完結する CSS ベースの配布経路。fandhe は React ランタイムに依存しない点が異なる（事実の対比であり、優劣の記述は行わない） |
| scaling | Theme の `scaling` prop（`90%`〜`110%` の 5 値、§4.6） | 対応する概念なし | fandhe には全トークンを一括倍率調整する機構がない（Theme provider 自体を持たないため、§6） |

## 6. 参照対象外の記録（Layout / Theme provider）

以下の 2 グループは、本リポジトリの意図的非採用方針に照らして
**参照対象外**として記録します（区分自体の最終確定は #937 の責務ですが、
再導入を示唆しないための事実確認として本書に残します）。

- **Layout カテゴリ**（Box / Flex / Grid / Container / Section、§3.1）:
  レイアウトプリミティブの提供は、本リポジトリでは `crates/core` の
  ノード木 API・`crates/app` の構築層が担う領域であり、chakra-ui /
  ark-ui を参照軸とした既存調査（`docs/design/component-coverage-map.md`）
  でも同種の layout primitive 群は非採用区分として扱われています。
  根拠となる Issue: **#716 / #724 / #735**。
- **Theme provider コンポーネント**（`/themes/docs/components/theme`、
  §3.4・§3.5）: `docs/policy/intentional-non-adoption.md` **§3.24**
  （「その他 UI 部品（marquee / chakra `Theme` コンポーネント）」）で、
  chakra-ui の `Theme` コンポーネント相当の provider パターンは既に
  非採用と確定しています。Radix Themes の `Theme` コンポーネントも
  同種の React provider パターンであり、同じ非採用方針の対象範囲内の
  事実として記録します。

上記はいずれも `.claude/rules/coding-rust.md`「意図的非採用機能の
再導入提案には評価軸の充足確認が必須」の対象です。本書はこれらの
再導入を提案するものではなく、Radix 側に存在する事実を記録するに
留めます。

## 7. 参照

- `docs/design/component-coverage-map.md`（chakra-ui / ark-ui を参照軸と
  した既存の全コンポーネント対応表、359 件基準、イシュー #734）
- `docs/policy/intentional-non-adoption.md` §3.22〜§3.24（イシュー #735、
  出典 #716 / #724 / #735）
- `crates/pre-styled-ui/src/theme.rs`（`Theme` 構造体・`DEFAULT_COLORS`
  等の既定トークン定義・`Theme::to_css` の出力順）
- `_/local-plans/radix-reference-and-docs-site-ia.md` §Phase 1-3（本 issue
  の親計画）
- イシュー #924（Radix UI を第 3 の参照軸として追加する方針決定）・
  #925（Phase 1 親）・#937（本書の消費先、`component-coverage-map.md`
  への Radix 列追加）
