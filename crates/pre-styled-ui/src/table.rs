//! styled Table（イシュー #767）: slot recipe 静的部品。root/header/body/
//! footer/row/column-header/cell/caption/scroll-area の 9 パーツで
//! `table`/`thead`/`tbody`/`tfoot`/`tr`/`th`/`td`/`caption`/`div`
//! （`scroll-area`、下記「`scroll-area` パーツ」節参照）の HTML 意味論を
//! そのまま尊重する（chakra-ui `data-display/table` 相当）。
//!
//! [`crate::card`]・[`crate::alert`] と同型の「状態機械を持たない静的
//! styled 部品」であり、`fandhe-frontend-headless-ui` 側に対応する anatomy は
//! 存在しない（[`crate::checkbox_card`]/[`crate::radio_card`] と同じく、
//! 本クレートで新規 anatomy `data-scope="table"` を定義する）。コンビニ関数
//! （全部入り `table(...)`）は提供せず、各パーツを個別に呼び出して組み立てる
//! 契約とする（[`crate::card`] と同じ判断、呼び出し例は各関数の rustdoc
//! `# Examples` を参照）。
//!
//! # variant（`variant`/`size`/`striped`/`sticky_header`/`interactive`）について
//!
//! [`crate::card`] と異なり 5 軸の variant を持つ（chakra-ui Table の
//! `variant`/`size`/`interactive`/`stickyHeader` のうち `showColumnBorder`
//! はスコープ外、下記参照。`stickyHeader` はイシュー #1571、`interactive`
//! はイシュー #2052 で shadcn/ui との突合を経て実装した）:
//!
//! - [`TableVariant`]: `Line`（既定、行ごとの下線区切り）/ `Outline`
//!   （外枠 + 角丸）。参照サイトとの対応は chakra-ui `line`≈`Line` /
//!   `outline`≈`Outline`、Radix Themes Table `ghost`≈`Line` /
//!   `surface`≈`Outline`（イシュー #1572 実測。両参照サイトとも 2 語彙のみで
//!   本クレートの 2 variant に過不足がないため、新規 variant は追加しない）。
//! - `size`（[`crate::recipe::Size`]）: セルの padding・font-size を切り替える
//!   （padding は `--fandhe-space-*` トークン、イシュー #1572 で
//!   リテラル値からトークン化した。font-size 写像は #1681 のまま）。
//! - `striped`（`bool`）: 縞模様表示。有効時は本文行の背景色を交互に変える。
//! - `sticky_header`（`bool`）: 有効時、`column-header`（`th`）を
//!   `position: sticky; top: 0` にする（下記「sticky ヘッダーの実装」節参照）。
//! - `interactive`（`bool`、イシュー #2052）: 有効時、`row`（`thead`/`tbody`/
//!   `tfoot` いずれの行も含む）に `bg-muted` の hover 背景を付ける（下記
//!   「`interactive` 軸（行 hover）」節参照）。
//!
//! クラスは `root` パーツのみへ付与する（複合部品の variant 統一方針、
//! `crates/pre-styled-ui/src/lib.rs` §「複合部品の variant 統一方針」参照）。
//! `row`/`cell`/`column-header` への伝搬は `root` の variant 宣言が登録する
//! root スコープの CSS custom property（`--fandhe-table-cell-padding` 等）の
//! 通常の CSS 継承で行い、[`crate::recipe::SlotRecipe`] へ子孫セレクタ機構は
//! 追加しない（[`crate::switch`]/[`crate::breadcrumb`] と同型のパターン）。
//!
//! # striped の実装（イシュー #767・[`crate::recipe::StateCondition::NthChildEven`]）
//!
//! `striped` は常に `false`/`true` いずれかのクラスを `root` へ付与する
//! （決定性維持、[`crate::breadcrumb::BreadcrumbVariant`] と同じ「既定値も
//! 明示的に登録する」判断）。`true` 側は root スコープへ
//! `--fandhe-table-stripe-bg: var(--fandhe-color-bg-subtle)` を設定し、`false`
//! 側は `--fandhe-table-stripe-bg: transparent` を明示設定する。`row` slot の
//! [`crate::recipe::StateCondition::NthChildEven`] 規則が
//! `background: var(--fandhe-table-stripe-bg, transparent)` を消費する。
//!
//! `:nth-child(even)` は親要素（`thead`/`tbody`/`tfoot` それぞれ）内の兄弟を
//! 基準に数えるため、通常構成（1 行の `thead` + 複数行の `tbody`）では
//! `tbody` 内の行のみが交互に縞模様になる。`thead` が複数行の場合は 2 行目
//! 以降も対象になりうるが、`column-header`（`th`）は base 規則で背景色を
//! 明示するため視覚的な影響は小さい。
//!
//! # sticky ヘッダーの実装（イシュー #1571）
//!
//! `sticky_header` は `striped` と同型に常に `false`/`true` いずれかの
//! クラスを `root` へ付与し（決定性維持）、root スコープへ
//! `--fandhe-table-header-position`（`static`/`sticky`）・
//! `--fandhe-table-sticky-offset`（常に `0`）の 2 custom property を設定する。
//! `column-header`（`th`）base 規則がこれを
//! `position: var(--fandhe-table-header-position, static); top:
//! var(--fandhe-table-sticky-offset, 0)` として消費する。
//!
//! `thead`/`tr`（`header`/`row` slot）ではなく `column-header`（`th`）へ
//! `position: sticky` を置く理由: 表セルへの sticky 指定はブラウザ横断で
//! 最も安定した適用対象である。chakra-ui は `tr` を対象にするが、
//! [`crate::recipe::SlotRecipe`] は子孫セレクタ機構を持たず（本モジュール doc
//! 「variant について」節参照）、`row` slot へ base 規則を追加すること自体が
//! PR #811 の不変条件（`separate` border モデル下で `tr` への border が
//! 無視される、上記「striped の実装」節と同じ制約源）と衝突しないよう
//! `column-header`/`cell` に閉じる既存方針を踏襲する。
//!
//! sticky 時も `column-header` の `background: var(--fandhe-color-bg)`
//! （既存の base 規則）は維持する。これが無いと sticky 中の見出しの背後に
//! スクロールしてきた本文行が透けて見えてしまう。
//!
//! `z-index` は `var(--fandhe-z-index-docked, 10)` を使う（[`crate::theme`]
//! の z-index スケール、イシュー #1423）。同スケールの `sticky`（1100）は
//! dropdown/popover 帯を越える値であり、単なる「スクロール内で自身の位置に
//! 留まる」sticky ヘッダーには強すぎるため採用しない。
//!
//! [`TableVariant::Outline`] の角丸クリップ（上記「Outline」variant 参照）は
//! `overflow: hidden` ではなく `clip-path: inset(0 round
//! var(--fandhe-radius-lg))` で行う（codex-review P1 是正、下記
//! 「`Outline` の角丸クリップに `overflow` を使わない理由」節参照）ため、
//! `root` 自身をスクロールコンテナ化せず `sticky_header` は `Outline`
//! でもページスクロールへ追従する。ただし `root` 自身がスクロール可能な
//! コンテナ（`overflow-y: auto` 等のスクロール枠）に包まれていない限り、
//! `sticky_header` はページ全体のスクロールでのみ効果を持つ（これは
//! `position: sticky` 自体の一般的な性質であり `Outline`/`Line` を問わない）。
//! スクロール枠との連携（chakra `ScrollArea` 相当）は下記「`scroll-area`
//! パーツ」節（イシュー #1572）を参照。
//!
//! # `interactive` 軸（行 hover、イシュー #2052）
//!
//! shadcn/ui Table の `tr` は無条件 hover（`hover:bg-muted/50`）を持つが、
//! 本クレートは chakra-ui 由来の `interactive` variant 名・opt-in 契約を
//! 採る（下記「参照サイトとの競合判定」節参照）。有効時は `root` スコープへ
//! `--fandhe-table-row-hover-bg: var(--fandhe-color-bg-muted)`
//! （[`crate::recipe::hover_bg_muted`] と同じトークン）を設定し、既定
//! （`Off`）は `transparent` を明示する（`striped`/`sticky_header` と同じ
//! 「既定値も明示的に登録する」決定性維持の判断）。
//!
//! `row` slot の [`crate::recipe::StateCondition::HoverExceptAttr`]
//! （`"data-selected"`）規則が
//! `background-image: linear-gradient(var(--fandhe-table-row-hover-bg),
//! var(--fandhe-table-row-hover-bg))` を消費する。
//!
//! **`background` shorthand ではなく `background-image` longhand を使う
//! 理由**: [`crate::recipe::SlotRecipe`] は variant × state の複合条件を
//! 持たず、`Hover` 系規則は [`crate::recipe::SlotRecipe::css`] 末尾の
//! `@media (hover: hover)` へ specificity `(0,4,0)` で集約出力される。
//! `background` shorthand で `transparent` を当てると、`striped: true` かつ
//! `interactive: false` の偶数行（`:nth-child(even)`、`(0,3,0)`、`background`
//! shorthand で縞を描画）が hover 時に縞を失ってしまう（既存出力への視覚的
//! 回帰であり、`docs/design/shadcn-reference-adoption-policy.md` §8 の
//! golden 純追加原則に反する）。`background-image` longhand は
//! `background-color`（縞）の上に重ねて描画されるため、`Off`（`transparent`
//! のグラデーション）では見た目が一切変わらず、`On` では縞・非縞を問わず
//! `bg-muted` で塗り潰される。
//!
//! `HoverExceptAttr("data-selected")` により、選択行（下記「`data-selected`
//! 行状態」節）は hover で洗い流されない（[`crate::tree_view`] と同型）。
//!
//! `cursor: pointer` やトランジションは付けない（chakra `interactive` /
//! shadcn/ui のいずれも持たない。クリック配線は呼び出し側の責務、
//! `docs/policy/intentional-non-adoption.md` §3.25 の責務境界）。
//!
//! **意図的な差分**: `row` slot は `thead`/`tbody`/`tfoot` いずれの行にも
//! 使われる（[`crate::recipe::SlotRecipe`] は子孫セレクタを持たないため
//! slot 単位でしか条件を切れない）。`column-header`（見出し行の `th`）は
//! 不透明背景（`--fandhe-table-header-bg`）を持つため見出し行では hover が
//! 視覚的に見えないが、`tfoot` の行では見える。chakra-ui `interactive` は
//! body 行のみを対象にするため、この点は参照サイトからの意図的な差分である
//! （子孫セレクタ機構を新設するコストに見合わないと判断した）。
//!
//! # `data-selected` 行状態（イシュー #2052）
//!
//! `row` slot の [`crate::recipe::StateCondition::Attr`]（`"data-selected"`）
//! 規則が `background: var(--fandhe-color-accent-subtle); color:
//! var(--fandhe-color-accent-fg-subtle)` を消費する（[`crate::theme`] の
//! `CONTRAST_PAIRS` 登録済みペア、[`crate::tree_view`] の選択状態と同型）。
//! `row` slot 自身は `data-*` の生産者を持たない（[`Anatomy::part`] が
//! `data-scope`/`data-part` 以外を自己出力しない静的部品、下記「セキュリティ
//! 不変条件」節参照）。`data-selected` は**呼び出し側が付与する**共有語彙
//! （既存の `pagination`/`calendar`/`tree_view` と同じ意味論・値なしの存在
//! 属性、`docs/design/pre-styled-ui-data-attr-vocabulary.md` §2.2 参照）で
//! あり、`table.rs` 自身は選択の生産・保持・送信を一切行わない
//! （`docs/policy/intentional-non-adoption.md` §3.25 の責務境界）。
//!
//! この state 規則は `row` の [`crate::recipe::StateCondition::NthChildEven`]
//! （striped）規則の**後**に登録する（両者とも specificity `(0,3,0)` のため
//! ソース順の後勝ちで選択が縞に勝つ、[`SlotRecipe::state`] rustdoc の
//! 「登録順」契約参照）。
//!
//! 行の tint は補強表示であり、選択状態そのものは行内の `checkbox` 等が
//! 持つ `aria-checked`/`aria-selected` が担う（`docs/design/
//! pre-styled-ui-interaction-visual-language.md` §9「pressed/selected の
//! 非テキストコントラスト」の装飾扱いの例外）。`tr` には `box-shadow` が
//! 描画されないため、境界リングでの選択表示はできない。
//!
//! # `data-align` セル整列（イシュー #2052）
//!
//! `cell`/`column-header` の両 slot に
//! [`crate::recipe::StateCondition::AttrEq`]（`"data-align"`,
//! `"start"`|`"center"`|`"end"`）規則を追加し、`text-align:
//! start`/`center`/`end` を消費する（属性セレクタ `(0,3,0)` が
//! `column-header` base の `text-align: inherit`（`(0,2,0)`）より勝つ）。
//!
//! `data-align`（値域 `start`/`center`/`end`）は `fandhe-frontend-headless-ui`
//! の `positioning.rs`（`data-side`/`data-align`）で既に定義済みの語彙を
//! 「軸方向の整列」という同一意味論・同一値域で再利用したもので、新規語彙は
//! 増やさない（`docs/design/pre-styled-ui-data-attr-vocabulary.md` §3.2
//! 規約 B-2）。`fandhe-frontend-wasm-full` の `position.rs` は `positioner`
//! パート（`data-scope="popover"` 等）からのみこの属性を読み戻すため、
//! `table` の `td`/`th` に付いた `data-align` とは干渉しない。
//!
//! # 参照サイトとの競合判定（イシュー #2052、
//! `docs/design/shadcn-reference-adoption-policy.md` §8）
//!
//! - **行 hover**: chakra-ui の値（`interactive` 軸名・opt-in）を採り、
//!   shadcn/ui の値（無条件 hover）は採らない。理由: 無条件 hover は既存
//!   出力の視覚的変更であり golden 純追加原則に反する。
//! - **選択行の配色**: shadcn/ui の値（neutral `muted`）ではなく本リポジトリ
//!   既存の selected 規約（[`crate::tree_view`] と同型の `accent-subtle`/
//!   `accent-fg-subtle`）を採る。理由: hover 規約が `bg-muted` を占有し、
//!   shadcn の `muted`/`muted/50` の 2 段を neutral 2 段で再現すると
//!   striped の `bg-subtle` とも衝突する。
//! - **セル整列の指定方法**: shadcn/ui（`text-right` ユーティリティクラス）・
//!   chakra-ui（`textAlign` prop）・Radix Themes（`justify` prop）のいずれの
//!   名称も持ち込まず、本リポジトリ既存語彙 `data-align`（start/center/end）
//!   で表現する。理由: `docs/design/shadcn-reference-adoption-policy.md` §8
//!   第 4 項（shadcn 固有語彙の禁止）・`data-*` 語彙規約 B-2。
//!
//! # caption（イシュー #1572）
//!
//! chakra-ui Table の `Table.Caption`（`font-weight: medium`・`textStyle:
//! xs`・既定 `captionSide: bottom`）を基準に是正した（Radix Themes
//! Table は caption 相当を持たない）。`caption-side: bottom` は既存どおり
//! 維持する（chakra 既定と一致し、`top` は参照側でも明示指定例のみ）。
//! `text-align: inherit` を追加し、`<caption>` の UA 既定である中央揃えを
//! 打ち消して `root` の `text-align: left`（`column-header` と同じ
//! `inherit` パターン）に揃える。`caption-side`（top/bottom）の切り替え
//! API は本イシューでは追加しない（下記「スコープ外」節参照）。
//!
//! # `Outline` variant の行罫線・ヘッダー背景の是正（イシュー #1572）
//!
//! イシュー #1571 実装時点の `Outline` は `--fandhe-table-row-border: none`
//! （行罫線なし）・ヘッダー背景を `Line` と共通の `bg` としていたが、
//! chakra-ui `outline`（行罫線維持・`bg.subtle` 相当のヘッダー背景）・
//! Radix Themes `surface`（行罫線維持・panel 系のヘッダー背景）のいずれとも
//! 一致しなかった。本イシューで `Outline` の行罫線を `Line` と同値
//! （`1px solid var(--fandhe-color-border-muted)`）へ戻し、ヘッダー背景を
//! `column-header` base が消費する `--fandhe-table-header-bg` custom
//! property（`Line` は `var(--fandhe-color-bg)`、`Outline` は
//! `var(--fandhe-color-bg-subtle)`）で variant ごとに切り替える。
//!
//! 行罫線を維持すると、`Outline` の外枠下端と body 最終行の `cell`
//! `border-bottom` が二重線になる。これを避けるため `--fandhe-table-
//! last-row-border` custom property（`Line` はリテラル `1px solid
//! var(--fandhe-color-border-muted)`〔`--fandhe-table-row-border` と同値だが
//! `var()` を介した循環参照を避けるためリテラルで独立させる〕、`Outline` は
//! `none`）を新設し、`row` slot の [`crate::recipe::StateCondition::LastChild`]
//! 規則が `--fandhe-table-row-border` をこの値で上書きする（`cell` の
//! `border-bottom` は継承で消費、`SlotRecipe` が子孫セレクタを持たない
//! 制約下での既存パターン、上記「variant について」節参照）。
//!
//! **トレードオフ**: `:last-child` は `thead`/`tbody`/`tfoot` 各行グループ
//! 内で個別に数えるため、`Outline` + `footer`（`tfoot`）併用時は body 最終行
//! （`tbody` の最終 `tr`）の罫線も消える。`Line` では `last-row-border` が
//! 通常の `row-border` と同値のため影響がない。chakra-ui `outline` も
//! 同じ挙動（最終行の border を 0 にする）であり、参照サイトへ揃える判断
//! として記録する。
//!
//! # `scroll-area` パーツ（chakra `Table.ScrollArea` 相当、イシュー #1572）
//!
//! [`scroll_area`] は headless-ui に対応物を持たない pre-styled-ui 専用
//! パーツで、[`crate::steps::body`]（PR #1814）と同型の判断（状態機械を
//! 伴わないグルーピング div は headless 側に anatomy を新設せず
//! pre-styled-ui 側で完結させる）を踏襲する。`crate::scroll_area`
//! （headless ScrollArea ラッパー、`data-state` 等を持つ状態機械付き
//! パーツ）は使わない。chakra `Table.ScrollArea` 自体が状態を持たない素の
//! `overflow: auto` ボックスであり、本パーツも状態を持たないため
//! （利用側が別途 `crate::scroll_area` で包む構成も引き続き可能）。
//!
//! `overflow: auto` によりスクロールコンテナ化する（`root` の `Outline`
//! クリップが `overflow` ではなく `clip-path` を使う理由〔上記
//! 「`Outline` の角丸クリップに `overflow` を使わない理由」節〕とは独立:
//! `scroll-area` は `root` を包む外側の要素であり、`root` 自身の
//! スクロール祖先化は起こさない）。高さは既定 `none`（無制限）で、利用側が
//! `--fandhe-table-scroll-max-height` custom property を上書きして
//! 与える（`Declaration::value` が `&'static str` のみのため props で数値を
//! 直接受けられない、[`css::decl`](crate::css::decl) の制約）。
//! `sticky_header` との関係: `scroll-area` が `root` にとって最も近い
//! スクロール祖先となり、`column-header` の `position: sticky` がこの
//! スクロールポート内で上端固定される。`border`/`border-radius` は
//! `scroll-area` に付与しない（外枠は `Outline` variant の責務、chakra も
//! 利用側 props に委ねる）。
//!
//! フォーカスリングは [`crate::recipe::FocusRingOffset::Inset`]
//! （splitter/listbox/scroll-area 系の inset 規約、
//! `docs/design/pre-styled-ui-focus-ring-and-size-conventions.md` §3）を
//! `:focus-visible` へ適用する。`tabindex` は固定付与しない（スクロール枠を
//! キーボード到達可能にするかは利用側の判断に委ねる。`tabindex="0"` +
//! `aria-label` を渡した場合や、キーボードでスクロール可能な要素として
//! ブラウザが自動的に focusable にする場合〔Chromium 等〕にリングが出る）。
//!
//! # `Outline` の角丸クリップに `overflow` を使わない理由（イシュー #1571
//! codex-review P1 是正）
//!
//! 当初 `Outline` variant の `root` は角丸クリップに `overflow: hidden` を
//! 使っていた。しかし CSS 仕様上、`overflow` を `visible` 以外の値にする
//! 要素は自動的に「スクロールコンテナ」となり、`position: sticky` の
//! 子孫にとって最も近い祖先スクロールコンテナとして扱われる。`root`
//! （`<table>`）自身はコンテンツに合わせて伸びるだけで実際にはスクロール
//! しないため、`column-header` の sticky 位置決めがこの祖先の
//! スクロールポート基準に固定されてしまい、ページをスクロールしても
//! `column-header` が追従しない（`root` 自身がスクロール可能なコンテナに
//! 包まれる構成でしか効かない）という契約違反を起こしていた。
//!
//! `clip-path` は `overflow` プロパティを変更せずに描画結果だけを
//! クリップするため、上記のスクロールコンテナ化を引き起こさない。
//! [`crate::rating_group`]/[`crate::stat`] が採用済みの「外部リソースを
//! 参照しないインライン `clip-path`」パターンをここでも踏襲し、
//! 半径は `border-radius` 宣言と同じ `--fandhe-radius-lg` custom property
//! を参照することで両宣言の齟齬を防ぐ。
//!
//! # 意図的に参照サイトへ合わせなかった点（イシュー #1571・#1572）
//!
//! - **ヘッダー文字の太さ**: chakra-ui は `medium`、Radix Themes Table は
//!   `bold` を使う。本クレートは Table を chakra-ui 由来の部品として
//!   `docs/design/component-coverage-map.md` に位置づけているため chakra 基準
//!   （`medium`）を採用し、Radix の `bold` は採らない。
//! - **striped の偶奇**: chakra-ui は奇数行（`odd`）に縞模様を付けるが、本
//!   クレートは #767 導入時からの偶数行（`even`、
//!   [`crate::recipe::StateCondition::NthChildEven`]）を維持する。視覚上の
//!   優劣が無い選択であり、`odd` 化には新しい `StateCondition` バリアントの
//!   追加が必要になる（既存 `even` を消費している呼び出し側との互換性を
//!   崩さない判断）。
//! - **行・セルの hover/transition**: イシュー #2052 で `interactive`
//!   variant（opt-in）を追加した（上記「`interactive` 軸（行 hover）」節
//!   参照）。無条件 hover（shadcn/ui）は既存出力の視覚変更になるため
//!   採らない。`cursor: pointer`・トランジションは付けない（chakra
//!   `interactive`/shadcn/ui のいずれも持たない）。行は `button`/`a` の
//!   ような操作可能ロールでもないため、`docs/design/
//!   pre-styled-ui-interaction-visual-language.md`（インタラクション視覚
//!   言語）が定義する「インタラクティブ slot」のフォーカスリング契約は
//!   適用しない（セルはフォーカス対象にならない）。
//! - **`data-selected` 状態属性**: イシュー #2052 で `row` slot の消費側
//!   規則を追加した（上記「`data-selected` 行状態」節参照）。値は呼び出し側
//!   が付与する共有語彙であり `table.rs` 自身は生産しない。
//! - **フッターの区切り線**: chakra-ui は `tfoot` に `border-top` を持つが、
//!   `root` の `border-collapse: separate` モデル下では `tfoot`（`footer`
//!   slot）への border 指定はブラウザに無視される（上記「cell」base の
//!   PR #811 不変条件と同型）。body/footer の視覚的な区切りは body 最終行の
//!   `cell` が持つ `border-bottom` に委ねる。
//! - **size の段階数**: chakra-ui は 3 段（`sm`/`md`/`lg`）、Radix Themes は
//!   3 段（`1`/`2`/`3`）だが、本クレートは他の pre-styled-ui 部品と共通の
//!   `size`（[`crate::recipe::Size`]、Xs〜Xl の 5 段）を使う（`docs/design/
//!   pre-styled-ui-size-and-color-palette-axes.md` §3.1 の共通 enum 規約。
//!   イシュー #1681/#1714 で追加済み、本イシューでは padding のトークン化
//!   のみを行った）。
//! - **`Outline` の背景・影**: Radix Themes `surface` は panel 系の背景色・
//!   微弱な inset shadow を持つが、本クレートの `Outline` は既存どおり
//!   `border` + `border-radius`（+ 本イシューで追加した `--fandhe-table-
//!   header-bg`）のみで、root 全体への背景・box-shadow は追加しない
//!   （chakra `outline` が背景・影を持たないことを優先した判断）。
//! - **shadcn/ui の `footer` 背景（`bg-muted/50`）・`border-top`**（イシュー
//!   #2052）: `root` の `border-collapse: separate` モデル下では `tfoot`
//!   （`footer` slot）への border 指定はブラウザに無視される（上記
//!   「フッターの区切り線」項と同じ制約）。背景色の追加は既存 `footer`
//!   base 規則（`font-weight` のみ）の変更になり golden 純追加原則に反する
//!   ため見送る。
//! - **shadcn/ui の caption サイズ（`text-sm`）・`mt-4`**（イシュー #2052）:
//!   本クレートの caption は chakra-ui 基準（`xs`/`medium`・
//!   `padding: var(--fandhe-space-3) 0`、上記「caption」節）を #1572 で
//!   確定済みであり、既存 `caption` base 規則の変更になるため合わせない。
//! - **shadcn/ui の `whitespace-nowrap`**（イシュー #2052）: `cell`/
//!   `column-header` へ追加すると既存の折り返し表示が変わる（既存 base
//!   規則の変更）ため採らない。chakra-ui / Radix Themes も nowrap を既定に
//!   持たない。
//! - **shadcn/ui の select 列余白調整（`[&:has([role=checkbox])]:pr-0`
//!   相当）**（イシュー #2052）: `:has()` は
//!   [`crate::recipe::StateCondition`] のいずれの variant にも対応する
//!   ものがない（複合セレクタ機構は未実装）。専用 variant を追加する動機に
//!   乏しく、Demo（下記 showcase 参照）では標準の `padding` のまま構成する。
//!
//! # セキュリティ不変条件
//!
//! - セル値・列見出し・caption はすべて呼び出し側が渡す `children`
//!   （`fandhe_frontend_core::text()` 等）としてノード木経由で受け取り、HTML
//!   文字列の直接組み立ては行わない。出力は `render()` の既定エスケープを
//!   必ず経由する（`raw_html()` は使用しない）。
//! - variant クラス名は [`crate::recipe::SlotRecipe::variant_classes`] が
//!   `&'static str` enum 値から決定的に生成し、動的文字列合成を行わない。
//! - 呼び出し側 `attrs` に含まれる `class` は
//!   [`crate::class_attr::drop_class_attr`] で除去してから recipe 生成
//!   クラスと合成するため、`class` 属性は常に単一（呼び出し側からのクラス
//!   偽装・重複混入を防ぐ）。
//! - [`column_header`] の `scope="col"` は関数側で固定するため、呼び出し側
//!   `attrs` に `scope`（大文字小文字無視）が含まれていても除去する
//!   （[`checkbox_card`](crate::checkbox_card) の `drop_reserved` と同型の
//!   fail-closed 判断。重複 `scope` 属性による無効な HTML・意味論の後勝ち
//!   混乱を防ぐ）。
//!
//! # スコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - chakra-ui の `showColumnBorder`・`ColumnGroup`（`colgroup`/`col`）は
//!   スコープ外（PR 本文に記録）。`stickyHeader` はイシュー #1571・
//!   `ScrollArea` 連携（[`scroll_area`]）はイシュー #1572・`interactive`/
//!   `data-selected`/`data-align` はイシュー #2052 で実装済み（上記各節
//!   参照）。
//! - `caption-side`（top/bottom）の切り替え API・[`scroll_area`] への
//!   `border`/`border-radius` 付与オプションは、必要になった時点で純追加で
//!   対応する（上記「caption」節・「`scroll-area` パーツ」節参照）。
//! - `aria-sort`/`data-sort`・全選択（select-all）・行選択のキーボード操作の
//!   生産・保持・送信は `fandhe-frontend-headless-ui` 側の data-table
//!   （イシュー #2124）の責務であり、本クレートは `column_header` の
//!   `aria-sort` 等の呼び出し側属性をそのまま通過させる（生産しない、
//!   `docs/policy/intentional-non-adoption.md` §3.25）。
//! - `examples/headless-pre-styled-ui` の追随・crates.io への公開は公開
//!   イシュー側のスコープ。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{
    focus_ring_declarations, FocusRingColor, FocusRingOffset, Size, SlotRecipe, StateCondition,
    VariantValue,
};
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
use fandhe_frontend_headless_ui::{anatomy, Anatomy};

/// `data-scope="table"` を固定した本コンポーネントの anatomy。
const ANATOMY: Anatomy = anatomy("table");

/// [`SlotRecipe::new`] に渡す slot 一覧（recipe とレンダリング関数の両方が
/// この配列を共有し、slot 名の乖離を防ぐ）。
const SLOTS: &[&str] = &[
    "root",
    "header",
    "body",
    "footer",
    "row",
    "column-header",
    "cell",
    "caption",
    // イシュー #1572: chakra `Table.ScrollArea` 相当のスクロール枠。
    // 状態を持たない静的パーツで headless-ui に対応物はない
    // （上記モジュール doc「`scroll-area` パーツ」節参照）。
    "scroll-area",
];

/// Table の見た目 variant（chakra-ui Table の `variant` を最小構成へ縮約）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TableVariant {
    /// 行ごとの下線区切り（既定）。
    #[default]
    Line,
    /// 外枠 + 角丸。
    Outline,
}

impl VariantValue for TableVariant {
    fn axis(self) -> &'static str {
        "variant"
    }

    fn value(self) -> &'static str {
        match self {
            Self::Line => "line",
            Self::Outline => "outline",
        }
    }
}

/// striped variant 値（内部専用、公開 API は `bool` のまま。
/// [`crate::table` モジュール doc](self)「striped の実装」節参照）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StripedVariant {
    /// 縞模様なし（既定）。
    Off,
    /// 縞模様あり。
    On,
}

impl VariantValue for StripedVariant {
    fn axis(self) -> &'static str {
        "striped"
    }

    fn value(self) -> &'static str {
        match self {
            Self::Off => "false",
            Self::On => "true",
        }
    }
}

impl From<bool> for StripedVariant {
    fn from(b: bool) -> Self {
        if b {
            Self::On
        } else {
            Self::Off
        }
    }
}

/// sticky ヘッダー variant 値（内部専用、公開 API は `bool` のまま。
/// [`crate::table` モジュール doc](self)「sticky ヘッダーの実装」節参照）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StickyHeaderVariant {
    /// 通常表示（既定）。
    Off,
    /// `column-header` を `position: sticky` にする。
    On,
}

impl VariantValue for StickyHeaderVariant {
    fn axis(self) -> &'static str {
        "sticky-header"
    }

    fn value(self) -> &'static str {
        match self {
            Self::Off => "false",
            Self::On => "true",
        }
    }
}

impl From<bool> for StickyHeaderVariant {
    fn from(b: bool) -> Self {
        if b {
            Self::On
        } else {
            Self::Off
        }
    }
}

/// interactive variant 値（内部専用、公開 API は `bool` のまま。
/// [`crate::table` モジュール doc](self)「`interactive` 軸（行 hover）」節
/// 参照）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InteractiveVariant {
    /// 行 hover なし（既定）。
    Off,
    /// 行 hover あり（`bg-muted`）。
    On,
}

impl VariantValue for InteractiveVariant {
    fn axis(self) -> &'static str {
        "interactive"
    }

    fn value(self) -> &'static str {
        match self {
            Self::Off => "false",
            Self::On => "true",
        }
    }
}

impl From<bool> for InteractiveVariant {
    fn from(b: bool) -> Self {
        if b {
            Self::On
        } else {
            Self::Off
        }
    }
}

/// Table の呼び出し側公開 props（`root` の引数）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TableProps {
    /// 見た目 variant（既定 [`TableVariant::Line`]）。
    pub variant: TableVariant,
    /// サイズ（既定 [`Size::Md`]）。
    pub size: Size,
    /// 縞模様表示の有無（既定 `false`）。
    pub striped: bool,
    /// sticky ヘッダーの有無（既定 `false`）。有効時は `column-header`
    /// （`th`）が `position: sticky; top: 0` になる（[`crate::table`
    /// モジュール doc](self)「sticky ヘッダーの実装」節参照）。
    pub sticky_header: bool,
    /// 行 hover の有無（既定 `false`、イシュー #2052）。有効時は `row`
    /// （`data-selected` を除く）が hover で `bg-muted` になる（[`crate::table`
    /// モジュール doc](self)「`interactive` 軸（行 hover）」節参照）。
    pub interactive: bool,
}

impl Default for TableProps {
    fn default() -> Self {
        Self {
            variant: TableVariant::default(),
            size: Size::Md,
            striped: false,
            sticky_header: false,
            interactive: false,
        }
    }
}

/// [`column_header`] が固定する属性名（呼び出し側 `attrs` からの偽装を
/// fail-closed で除去する対象）。
const COLUMN_HEADER_RESERVED: &[&str] = &["scope"];

/// 呼び出し側 `attrs` からフレームワーク固定キー（ASCII 大文字小文字無視）を
/// 除外する（`crates/pre-styled-ui/src/checkbox_card.rs` の `drop_reserved`
/// と同型）。
fn drop_reserved<'a>(
    attrs: Vec<(&'a str, &'a str)>,
    reserved: &'static [&'static str],
) -> Vec<(&'a str, &'a str)> {
    attrs
        .into_iter()
        .filter(|(k, _)| !reserved.iter().any(|r| k.eq_ignore_ascii_case(r)))
        .collect()
}

/// Table の recipe（scope `"table"`、[`SLOTS`] の 9 パーツ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("table", SLOTS)
        .base(
            "root",
            vec![
                decl("width", "100%"),
                // `border-collapse: collapse` だと `Outline` variant の
                // `border-radius` がブラウザにより無視される（角丸が効かない）ため、
                // `separate` + `border-spacing: 0` を既定にする（イシュー #767
                // PR #811 Bugbot 指摘）。`separate` でもセル間に境界線の重複は
                // 発生しない（`cell`/`column-header` は `border-bottom` のみを
                // 使い、隣接セル間の縦境界線を持たないため見た目に影響しない）。
                // 注意: `separate` モデルでは `row`（`tr`）への border 指定は
                // ブラウザに無視される（CSS 表モデル仕様上、border の対象は
                // table と cell のみ）。そのため Line variant の行区切り線は
                // `row` ではなく `cell`/`column-header` 側に持たせている
                // （下記 `cell` base 参照、PR #811 Bugbot 追加指摘で是正）。
                decl("border-collapse", "separate"),
                decl("border-spacing", "0"),
                decl("text-align", "left"),
            ],
        )
        .base(
            "caption",
            vec![
                decl("caption-side", "bottom"),
                // イシュー #1572: chakra-ui `Table.Caption`（`textStyle: xs`
                // ・`font-weight: medium`）基準へ是正（旧 `font-size-sm`・
                // font-weight 未指定から変更、モジュール doc「caption」節
                // 参照）。
                decl("padding", "var(--fandhe-space-3) 0"),
                decl("color", "var(--fandhe-color-fg-muted)"),
                decl("font-size", "var(--fandhe-font-font-size-xs)"),
                decl("font-weight", "var(--fandhe-font-font-weight-medium)"),
                // `<caption>` の UA 既定（中央揃え）を打ち消し、`root` の
                // `text-align: left` に揃える（`column-header` と同じ
                // `inherit` パターン、モジュール doc「caption」節参照）。
                decl("text-align", "inherit"),
            ],
        )
        .base(
            "column-header",
            vec![
                decl(
                    "padding",
                    "var(--fandhe-table-cell-padding, var(--fandhe-space-3) var(--fandhe-space-4))",
                ),
                decl(
                    "font-size",
                    "var(--fandhe-table-font-size, var(--fandhe-font-font-size-sm))",
                ),
                // イシュー #1571: chakra-ui Table のヘッダー太さ（medium）へ
                // 合わせる（semibold から変更。Radix Themes の bold は
                // 不採用、モジュール doc「意図的に参照サイトへ合わせなかった
                // 点」節参照）。
                decl("font-weight", "var(--fandhe-font-font-weight-medium)"),
                // イシュー #1571: 見出しテキストの色を明示する（chakra-ui /
                // Radix Themes とも既定の `fg` トークンで統一しており、
                // 従来は `color` を宣言せずブラウザ既定色に委ねていた）。
                decl("color", "var(--fandhe-color-fg)"),
                decl("text-align", "inherit"),
                // イシュー #1571: chakra-ui / Radix Themes とも 1px であり
                // `2px` は参照サイトより太い（縦罫線がある表と誤認しやすい）。
                // 行罫線に使う `border-muted` より一段強い `border` トークン
                // でヘッダーを区切る。
                decl("border-bottom", "1px solid var(--fandhe-color-border)"),
                // sticky 中に背後の本文行が透けないよう不透明背景を維持する
                // （sticky_header 有無に関わらず既存どおり必須）。イシュー
                // #1572: `--fandhe-table-header-bg`（variant スコープ）を
                // 消費するよう変更（`Outline` のみ `bg-subtle`、モジュール
                // doc「`Outline` variant の行罫線・ヘッダー背景の是正」節
                // 参照）。フォールバックは従来どおり `bg`（不透明維持）。
                decl(
                    "background",
                    "var(--fandhe-table-header-bg, var(--fandhe-color-bg))",
                ),
                // イシュー #1571: 数値列の桁揃え（chakra-ui root の
                // `tabular-nums` 相当。root ではなくリーフ側へ置く判断は
                // モジュール doc 参照）。
                decl("font-variant-numeric", "tabular-nums"),
                // イシュー #1571: sticky_header variant（root スコープ）が
                // 設定する custom property を消費する。既定（Off）は
                // `static`/`0` のため見た目に影響しない。
                decl("position", "var(--fandhe-table-header-position, static)"),
                decl("top", "var(--fandhe-table-sticky-offset, 0)"),
                // dropdown/popover 帯（1000/1200）より下、通常のドキュメント
                // フローより上の `docked` 段を使う（モジュール doc
                // 「sticky ヘッダーの実装」節参照）。
                decl("z-index", "var(--fandhe-z-index-docked, 10)"),
            ],
        )
        .base(
            "cell",
            vec![
                decl(
                    "padding",
                    "var(--fandhe-table-cell-padding, var(--fandhe-space-3) var(--fandhe-space-4))",
                ),
                decl(
                    "font-size",
                    "var(--fandhe-table-font-size, var(--fandhe-font-font-size-sm))",
                ),
                // `root` の `border-collapse: separate`（Outline variant の
                // `border-radius` 対応、直前コミットで導入）下では、CSS の
                // テーブルモデル仕様上 `tr`（`row` スロット）への border は
                // 描画されない（separate モデルでは table と cell のみが
                // border の対象）。そのため Line variant の行区切り線は
                // `row` ではなくここ（`td`/`th` にあたる `cell`/
                // `column-header` スロット）へ持たせる（イシュー #767
                // PR #811 Bugbot 指摘）。
                decl("border-bottom", "var(--fandhe-table-row-border, none)"),
                // イシュー #1571: column-header と同じ理由で数値列の桁揃え
                // を追加する。
                decl("font-variant-numeric", "tabular-nums"),
            ],
        )
        .base(
            "footer",
            vec![
                // イシュー #1571: chakra-ui `tfoot` の `font-weight: medium`
                // 相当。border は付けない（`root` の `border-collapse:
                // separate` モデル下では `tfoot`〔footer slot〕への border
                // 指定はブラウザに無視される、上記 `cell` base の PR #811
                // 不変条件と同型。body/footer の区切りは body 最終行の
                // `cell` が持つ `border-bottom` に委ねる）。
                decl("font-weight", "var(--fandhe-font-font-weight-medium)"),
            ],
        )
        // イシュー #1572: chakra `Table.ScrollArea` 相当のスクロール枠
        // （モジュール doc「`scroll-area` パーツ」節参照）。border/radius は
        // 付与しない（外枠は `Outline` variant の責務）。
        .base(
            "scroll-area",
            vec![
                decl("overflow", "auto"),
                decl("max-width", "100%"),
                decl("max-height", "var(--fandhe-table-scroll-max-height, none)"),
                decl("scrollbar-width", "thin"),
                decl("scrollbar-color", "var(--fandhe-color-border) transparent"),
            ],
        )
        .variant(
            TableVariant::Line,
            "root",
            vec![
                decl(
                    "--fandhe-table-row-border",
                    "1px solid var(--fandhe-color-border-muted)",
                ),
                // イシュー #1572: `Outline` と対称に明示登録する
                // （`column-header` background のフォールバック値と同値）。
                decl("--fandhe-table-header-bg", "var(--fandhe-color-bg)"),
                // `Line` は `row-border` と同値（`Outline` との非対称は
                // モジュール doc「`Outline` variant の行罫線・ヘッダー背景
                // の是正」節「トレードオフ」参照）。
                decl(
                    "--fandhe-table-last-row-border",
                    "1px solid var(--fandhe-color-border-muted)",
                ),
            ],
        )
        .variant(
            TableVariant::Outline,
            "root",
            vec![
                // イシュー #1572: chakra-ui `outline` / Radix Themes
                // `surface` とも行罫線を維持するため `Line` と同値へ是正
                // （旧 `none`、モジュール doc「`Outline` variant の行罫線・
                // ヘッダー背景の是正」節参照）。
                decl(
                    "--fandhe-table-row-border",
                    "1px solid var(--fandhe-color-border-muted)",
                ),
                // イシュー #1572: chakra-ui `outline` の `bg.subtle` 相当。
                decl("--fandhe-table-header-bg", "var(--fandhe-color-bg-subtle)"),
                // イシュー #1572: `Outline` の外枠下端と body 最終行の
                // `border-bottom` による二重線を避ける（同節参照）。
                decl("--fandhe-table-last-row-border", "none"),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "var(--fandhe-radius-lg)"),
                // `column-header` の不透明背景・striped 偶数行の背景は
                // `root` の `border-radius` に追従してクリップされない
                // （`border-collapse: separate` 下では子孫が親の角丸の
                // 外側にはみ出して矩形の角のまま描画される）。子孫の描画を
                // 角丸内へ収める必要がある（イシュー #767 PR #811 Bugbot
                // 指摘）。
                //
                // イシュー #1571 codex-review P1 是正: 当初 `overflow:
                // hidden` を使っていたが、`overflow` を `visible` 以外に
                // する宣言は CSS 仕様上その要素を `position: sticky` の
                // 「最も近いスクロール祖先」に仕立ててしまい、`root`
                // 自身はスクロールしない（コンテンツに追従して伸びるだけ）
                // ため `sticky_header` がページスクロールに追従しなくなる
                // （下記「sticky ヘッダーの実装」節参照）。`clip-path` は
                // `overflow` を変更せずに視覚的なクリップだけを行うため
                // スクロール祖先化を起こさず、`sticky_header` と共存できる
                // （[`crate::rating_group`]/[`crate::stat`] が採用済みの
                // 「外部リソースを参照しないインライン `clip-path`」
                // パターンをここでも踏襲する）。半径は `border-radius` と
                // 同じ custom property 値を参照し、両宣言が常に一致する
                // ようにする。
                decl("clip-path", "inset(0 round var(--fandhe-radius-lg))"),
            ],
        )
        .default_variant(TableVariant::Line)
        // イシュー #1572: padding をリテラル値から `--fandhe-space-*`
        // トークンへ置換（値は不変、モジュール doc「variant について」節
        // 参照）。`SlotRecipe::size_variants` を使うことで既定 `Md` の
        // 登録も一括で行う（`data_list.rs` と同型パターン）。
        .size_variants(
            "root",
            &[
                // イシュー #1681: Xs は cell-padding 0.25rem 刻みの等差
                // 進行を外挿した (space-1, space-2)。font-size はトークン
                // 下限 xs をそのまま使う（Sm と同一。より小さいトークンが
                // 存在しないため）。
                (
                    Size::Xs,
                    vec![
                        decl(
                            "--fandhe-table-cell-padding",
                            "var(--fandhe-space-1) var(--fandhe-space-2)",
                        ),
                        decl(
                            "--fandhe-table-font-size",
                            "var(--fandhe-font-font-size-xs)",
                        ),
                    ],
                ),
                (
                    Size::Sm,
                    vec![
                        decl(
                            "--fandhe-table-cell-padding",
                            "var(--fandhe-space-2) var(--fandhe-space-3)",
                        ),
                        decl(
                            "--fandhe-table-font-size",
                            "var(--fandhe-font-font-size-xs)",
                        ),
                    ],
                ),
                (
                    Size::Md,
                    vec![
                        decl(
                            "--fandhe-table-cell-padding",
                            "var(--fandhe-space-3) var(--fandhe-space-4)",
                        ),
                        decl(
                            "--fandhe-table-font-size",
                            "var(--fandhe-font-font-size-sm)",
                        ),
                    ],
                ),
                (
                    Size::Lg,
                    vec![
                        decl(
                            "--fandhe-table-cell-padding",
                            "var(--fandhe-space-4) var(--fandhe-space-5)",
                        ),
                        decl(
                            "--fandhe-table-font-size",
                            "var(--fandhe-font-font-size-md)",
                        ),
                    ],
                ),
                (
                    Size::Xl,
                    vec![
                        decl(
                            "--fandhe-table-cell-padding",
                            "var(--fandhe-space-5) var(--fandhe-space-6)",
                        ),
                        decl(
                            "--fandhe-table-font-size",
                            "var(--fandhe-font-font-size-lg)",
                        ),
                    ],
                ),
            ],
        )
        .variant(
            StripedVariant::Off,
            "root",
            vec![decl("--fandhe-table-stripe-bg", "transparent")],
        )
        .variant(
            StripedVariant::On,
            "root",
            vec![decl(
                "--fandhe-table-stripe-bg",
                "var(--fandhe-color-bg-subtle)",
            )],
        )
        .default_variant(StripedVariant::Off)
        // イシュー #1571: sticky ヘッダー variant。既定 Off も明示的に登録
        // する（striped と同じ決定性維持の判断、上記「sticky ヘッダーの
        // 実装」節参照）。
        .variant(
            StickyHeaderVariant::Off,
            "root",
            vec![
                decl("--fandhe-table-header-position", "static"),
                decl("--fandhe-table-sticky-offset", "0"),
            ],
        )
        .variant(
            StickyHeaderVariant::On,
            "root",
            vec![
                decl("--fandhe-table-header-position", "sticky"),
                decl("--fandhe-table-sticky-offset", "0"),
            ],
        )
        .default_variant(StickyHeaderVariant::Off)
        // イシュー #2052: `interactive` variant。既定 Off も明示的に登録する
        // （striped/sticky_header と同じ決定性維持の判断、上記モジュール doc
        // 「`interactive` 軸（行 hover）」節参照）。
        .variant(
            InteractiveVariant::Off,
            "root",
            vec![decl("--fandhe-table-row-hover-bg", "transparent")],
        )
        .variant(
            InteractiveVariant::On,
            "root",
            vec![decl(
                "--fandhe-table-row-hover-bg",
                "var(--fandhe-color-bg-muted)",
            )],
        )
        .default_variant(InteractiveVariant::Off)
        .state(
            "row",
            StateCondition::NthChildEven,
            vec![decl(
                "background",
                "var(--fandhe-table-stripe-bg, transparent)",
            )],
        )
        // イシュー #2052: 選択行（呼び出し側が付与する共有語彙
        // `data-selected`）。`NthChildEven`（縞）の後に登録することで、
        // どちらも specificity (0,3,0) のままソース順の後勝ちで選択が縞に
        // 勝つ（上記モジュール doc「`data-selected` 行状態」節参照）。
        .state(
            "row",
            StateCondition::Attr("data-selected"),
            vec![
                decl("background", "var(--fandhe-color-accent-subtle)"),
                decl("color", "var(--fandhe-color-accent-fg-subtle)"),
            ],
        )
        // イシュー #2052: `interactive: true` の行 hover。選択行
        // （`data-selected`）は hover で洗い流さない（上記モジュール doc
        // 「`interactive` 軸（行 hover）」節「`background` shorthand ではなく
        // `background-image` longhand を使う理由」参照）。
        .state(
            "row",
            StateCondition::HoverExceptAttr("data-selected"),
            vec![decl(
                "background-image",
                "linear-gradient(var(--fandhe-table-row-hover-bg), var(--fandhe-table-row-hover-bg))",
            )],
        )
        // イシュー #2052: セル整列（shadcn/ui の `text-right` 相当を本
        // リポジトリ既存語彙 `data-align` で表現、上記モジュール doc
        // 「`data-align` セル整列」節参照）。
        .state(
            "cell",
            StateCondition::AttrEq("data-align", "start"),
            vec![decl("text-align", "start")],
        )
        .state(
            "cell",
            StateCondition::AttrEq("data-align", "center"),
            vec![decl("text-align", "center")],
        )
        .state(
            "cell",
            StateCondition::AttrEq("data-align", "end"),
            vec![decl("text-align", "end")],
        )
        .state(
            "column-header",
            StateCondition::AttrEq("data-align", "start"),
            vec![decl("text-align", "start")],
        )
        .state(
            "column-header",
            StateCondition::AttrEq("data-align", "center"),
            vec![decl("text-align", "center")],
        )
        .state(
            "column-header",
            StateCondition::AttrEq("data-align", "end"),
            vec![decl("text-align", "end")],
        )
        // イシュー #1572: `Outline` + `footer` 併用時の二重線を避けるため、
        // 最終行の `--fandhe-table-row-border` を variant スコープの
        // `--fandhe-table-last-row-border` で上書きする（`cell` の
        // `border-bottom` は継承で消費、モジュール doc「`Outline` variant
        // の行罫線・ヘッダー背景の是正」節参照）。同一 slot の他の state
        // 規則より後段に置く（`StateCondition::LastChild` は最後に評価
        // される契約、`SlotRecipe::state` rustdoc 参照）。
        .state(
            "row",
            StateCondition::LastChild,
            vec![decl(
                "--fandhe-table-row-border",
                "var(--fandhe-table-last-row-border)",
            )],
        )
        // イシュー #1572: `scroll-area` の `:focus-visible` リング
        // （inset、splitter/listbox/scroll-area 系の規約。モジュール doc
        // 「`scroll-area` パーツ」節参照）。`tabindex` は固定付与しない
        // ため、利用側が focusable にした場合のみ現れる。
        .state(
            "scroll-area",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Inset),
        )
}

/// Table の静的 CSS 全文。
#[must_use]
pub fn css() -> String {
    recipe().css()
}

/// root パーツ（`<table>`）を組み立てる。`variant`/`size`/`striped` に応じた
/// クラスを付与する唯一のパーツ（[`drop_class_attr`] により呼び出し側の
/// `class` は除去してから合成する）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::table::{self, TableProps};
///
/// let node = table::root(TableProps::default(), vec![], vec![]);
/// assert!(render(&node).contains(r#"data-scope="table" data-part="root""#));
/// ```
#[must_use]
pub fn root<'a>(props: TableProps, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let recipe = recipe();
    let striped: StripedVariant = props.striped.into();
    let sticky_header: StickyHeaderVariant = props.sticky_header.into();
    let interactive: InteractiveVariant = props.interactive.into();
    let class = recipe.variant_classes(&[
        ("variant", props.variant.value()),
        ("size", props.size.value()),
        ("striped", striped.value()),
        ("sticky-header", sticky_header.value()),
        ("interactive", interactive.value()),
    ]);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    ANATOMY.part("root", "table", merged, children)
}

/// header パーツ（`<thead>`）を組み立てる。variant を持たないため `class` は
/// 付与せず、呼び出し側 `attrs` をそのまま連結する。
#[must_use]
pub fn header<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("header", "thead", attrs, children)
}

/// body パーツ（`<tbody>`）を組み立てる。
#[must_use]
pub fn body<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("body", "tbody", attrs, children)
}

/// footer パーツ（`<tfoot>`）を組み立てる。
#[must_use]
pub fn footer<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("footer", "tfoot", attrs, children)
}

/// row パーツ（`<tr>`）を組み立てる。
#[must_use]
pub fn row<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("row", "tr", attrs, children)
}

/// column-header パーツ（`<th scope="col">`）を組み立てる。列見出しの
/// WAI-ARIA/HTML 意味論（`scope="col"`）を既定で担保する。呼び出し側 `attrs`
/// に `scope` を含めても [`drop_reserved`] により除去される（本モジュール
/// doc「セキュリティ不変条件」節参照）。
#[must_use]
pub fn column_header<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let mut merged: Vec<(&str, &str)> = vec![("scope", "col")];
    merged.extend(drop_reserved(attrs, COLUMN_HEADER_RESERVED));
    ANATOMY.part("column-header", "th", merged, children)
}

/// cell パーツ（`<td>`）を組み立てる。
#[must_use]
pub fn cell<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("cell", "td", attrs, children)
}

/// caption パーツ（`<caption>`）を組み立てる。呼び出し側は `<table>` の
/// 直接の子として `root` の `children` 先頭に置く必要がある（HTML 仕様上
/// `caption` は `table` の最初の子でなければならない。本関数自体は順序を
/// 強制しない）。
#[must_use]
pub fn caption<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("caption", "caption", attrs, children)
}

/// scroll-area パーツ（`<div>`、chakra `Table.ScrollArea` 相当）を組み立てる。
/// headless-ui に対応物を持たない pre-styled-ui 専用パーツで、状態を持たない
/// （モジュール doc「`scroll-area` パーツ」節参照）。呼び出し側は `root`（と
/// その `children`）を本関数の `children` として渡す契約とする。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::table::{self, TableProps};
///
/// let node = table::scroll_area(
///     vec![("style", "--fandhe-table-scroll-max-height: 12rem")],
///     vec![table::root(TableProps::default(), vec![], vec![])],
/// );
/// assert!(render(&node).contains(r#"data-scope="table" data-part="scroll-area""#));
/// ```
#[must_use]
pub fn scroll_area<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("scroll-area", "div", attrs, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};

    #[test]
    fn default_variant_is_line_md_not_striped() {
        let html = render(&root(TableProps::default(), vec![], vec![]));
        assert!(html.contains("fd-table--variant-line"));
        assert!(html.contains("fd-table--size-md"));
        assert!(html.contains("fd-table--striped-false"));
        assert!(html.contains("fd-table--sticky-header-false"));
        assert!(html.contains("fd-table--interactive-false"));
    }

    #[test]
    fn interactive_true_maps_to_expected_class() {
        let props = TableProps {
            interactive: true,
            ..TableProps::default()
        };
        let html = render(&root(props, vec![], vec![]));
        assert!(html.contains("fd-table--interactive-true"));
    }

    #[test]
    fn variant_enumeration_maps_to_expected_classes() {
        for (variant, class) in [
            (TableVariant::Line, "fd-table--variant-line"),
            (TableVariant::Outline, "fd-table--variant-outline"),
        ] {
            let props = TableProps {
                variant,
                ..TableProps::default()
            };
            let html = render(&root(props, vec![], vec![]));
            assert!(html.contains(class), "variant={variant:?} -> {html}");
        }
    }

    #[test]
    fn size_enumeration_maps_to_expected_classes() {
        for (size, class) in [
            (Size::Sm, "fd-table--size-sm"),
            (Size::Md, "fd-table--size-md"),
            (Size::Lg, "fd-table--size-lg"),
        ] {
            let props = TableProps {
                size,
                ..TableProps::default()
            };
            let html = render(&root(props, vec![], vec![]));
            assert!(html.contains(class), "size={size:?} -> {html}");
        }
    }

    #[test]
    fn striped_true_maps_to_expected_class() {
        let props = TableProps {
            striped: true,
            ..TableProps::default()
        };
        let html = render(&root(props, vec![], vec![]));
        assert!(html.contains("fd-table--striped-true"));
    }

    #[test]
    fn sticky_header_true_maps_to_expected_class() {
        let props = TableProps {
            sticky_header: true,
            ..TableProps::default()
        };
        let html = render(&root(props, vec![], vec![]));
        assert!(html.contains("fd-table--sticky-header-true"));
    }

    #[test]
    fn parts_use_expected_tags_and_data_part() {
        assert!(render(&header(vec![], vec![]))
            .starts_with(r#"<thead data-scope="table" data-part="header""#));
        assert!(render(&body(vec![], vec![]))
            .starts_with(r#"<tbody data-scope="table" data-part="body""#));
        assert!(render(&footer(vec![], vec![]))
            .starts_with(r#"<tfoot data-scope="table" data-part="footer""#));
        assert!(
            render(&row(vec![], vec![])).starts_with(r#"<tr data-scope="table" data-part="row""#)
        );
        assert!(
            render(&cell(vec![], vec![])).starts_with(r#"<td data-scope="table" data-part="cell""#)
        );
        assert!(render(&caption(vec![], vec![]))
            .starts_with(r#"<caption data-scope="table" data-part="caption""#));
        assert!(render(&scroll_area(vec![], vec![]))
            .starts_with(r#"<div data-scope="table" data-part="scroll-area""#));
    }

    #[test]
    fn column_header_fixes_scope_col_and_drops_caller_scope() {
        let html = render(&column_header(vec![("scope", "row")], vec![]));
        assert!(html.starts_with(r#"<th data-scope="table" data-part="column-header""#));
        assert!(html.contains(r#"scope="col""#));
        assert!(!html.contains(r#"scope="row""#));
        assert_eq!(html.matches(r#" scope=""#).count(), 1);
    }

    #[test]
    fn composed_table_snapshot() {
        let node = root(
            TableProps::default(),
            vec![],
            vec![
                caption(vec![], vec![text("Users")]),
                header(
                    vec![],
                    vec![row(vec![], vec![column_header(vec![], vec![text("Name")])])],
                ),
                body(
                    vec![],
                    vec![row(vec![], vec![cell(vec![], vec![text("Alice")])])],
                ),
            ],
        );
        let html = render(&node);
        assert_eq!(
            html,
            concat!(
                r#"<table data-scope="table" data-part="root" class="fd-table--variant-line fd-table--size-md fd-table--striped-false fd-table--sticky-header-false fd-table--interactive-false">"#,
                r#"<caption data-scope="table" data-part="caption">Users</caption>"#,
                r#"<thead data-scope="table" data-part="header">"#,
                r#"<tr data-scope="table" data-part="row">"#,
                r#"<th data-scope="table" data-part="column-header" scope="col">Name</th>"#,
                r#"</tr>"#,
                r#"</thead>"#,
                r#"<tbody data-scope="table" data-part="body">"#,
                r#"<tr data-scope="table" data-part="row">"#,
                r#"<td data-scope="table" data-part="cell">Alice</td>"#,
                r#"</tr>"#,
                r#"</tbody>"#,
                r#"</table>"#,
            )
        );
    }

    #[test]
    fn caller_class_attr_on_root_is_dropped_not_duplicated() {
        let html = render(&root(
            TableProps::default(),
            vec![("class", "attacker-controlled")],
            vec![],
        ));
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(!html.contains("attacker-controlled"));
    }

    #[test]
    fn xss_payload_in_cell_and_column_header_children_is_escaped() {
        let cell_html = render(&cell(vec![], vec![text("<script>alert(1)</script>")]));
        assert!(!cell_html.contains("<script>"));
        assert!(cell_html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));

        let header_html = render(&column_header(
            vec![],
            vec![text("<img src=x onerror=alert(1)>")],
        ));
        assert!(!header_html.contains("<img"));
        assert!(header_html.contains("&lt;img"));

        let caption_html = render(&caption(vec![], vec![text("<script>alert(1)</script>")]));
        assert!(!caption_html.contains("<script>"));

        // イシュー #1572: scroll-area の children・attrs も同じ既定エスケープ
        // 経路を通ることを固定する。
        let scroll_area_children_html = render(&scroll_area(
            vec![],
            vec![text("<script>alert(1)</script>")],
        ));
        assert!(!scroll_area_children_html.contains("<script>"));
        assert!(scroll_area_children_html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));

        let scroll_area_attr_html = render(&scroll_area(
            vec![("data-x", "\"><script>alert(1)</script>")],
            vec![],
        ));
        assert!(!scroll_area_attr_html.contains("<script>"));
    }

    #[test]
    fn css_output_declares_striped_and_size_tokens() {
        let out = css();
        assert!(out.contains(":nth-child(even)"));
        assert!(out.contains("--fandhe-table-stripe-bg"));
        assert!(out.contains("--fandhe-table-cell-padding"));
        assert!(out.contains("--fandhe-table-header-position"));
        assert!(out.contains("position: var(--fandhe-table-header-position, static);"));
        assert!(!out.contains('<'));
    }

    /// イシュー #1571: `column-header` base 規則が chakra-ui / Radix Themes
    /// 基準の 1px 罫線・medium 太さになっていることを固定する
    /// （旧 2px semibold からの是正、上記モジュール doc「意図的に参照
    /// サイトへ合わせなかった点」節参照）。
    #[test]
    fn column_header_uses_one_pixel_border_and_medium_weight() {
        let out = css();
        assert!(out.contains("border-bottom: 1px solid var(--fandhe-color-border);"));
        assert!(out.contains("font-weight: var(--fandhe-font-font-weight-medium);"));
        assert!(!out.contains("2px solid var(--fandhe-color-border-muted)"));
    }

    /// イシュー #1571: `footer`（`tfoot`）base 規則が medium 太さのみを持ち、
    /// border を持たないことを固定する（`separate` border モデル下では
    /// `tfoot` への border 指定が無効なため、上記モジュール doc「sticky
    /// ヘッダーの実装」節と対をなす PR #811 型の不変条件）。
    #[test]
    fn footer_has_medium_weight_and_no_border_rule() {
        let out = css();
        let footer_rule_start = out
            .find(r#"[data-scope="table"][data-part="footer"] {"#)
            .expect("footer base 規則が css() 出力に存在すること");
        let footer_rule_end = out[footer_rule_start..]
            .find('}')
            .map(|offset| footer_rule_start + offset)
            .expect("footer base 規則が `}` で閉じられていること");
        let footer_rule = &out[footer_rule_start..footer_rule_end];
        assert!(footer_rule.contains("font-weight: var(--fandhe-font-font-weight-medium);"));
        assert!(!footer_rule.contains("border"));
    }

    /// イシュー #1572: size 軸の cell padding が `--fandhe-space-*`
    /// トークンへ置換されていることを固定する（旧リテラル値からの是正、
    /// モジュール doc「variant について」節参照）。5 段すべてを確認する。
    #[test]
    fn table_css_contains_size_custom_property_variants() {
        let out = css();
        for padding in [
            "var(--fandhe-space-1) var(--fandhe-space-2)",
            "var(--fandhe-space-2) var(--fandhe-space-3)",
            "var(--fandhe-space-3) var(--fandhe-space-4)",
            "var(--fandhe-space-4) var(--fandhe-space-5)",
            "var(--fandhe-space-5) var(--fandhe-space-6)",
        ] {
            let decl = format!("--fandhe-table-cell-padding: {padding};");
            assert!(out.contains(&decl), "missing {decl} in {out}");
        }
    }

    /// イシュー #1572: caption base 規則が chakra-ui 基準（medium/xs/
    /// `text-align: inherit`）へトークン化されていることを固定する
    /// （モジュール doc「caption」節参照）。
    #[test]
    fn table_css_caption_uses_tokens_and_inherits_alignment() {
        let out = css();
        let caption_rule_start = out
            .find(r#"[data-scope="table"][data-part="caption"] {"#)
            .expect("caption base 規則が css() 出力に存在すること");
        let caption_rule_end = out[caption_rule_start..]
            .find('}')
            .map(|offset| caption_rule_start + offset)
            .expect("caption base 規則が `}` で閉じられていること");
        let caption_rule = &out[caption_rule_start..caption_rule_end];
        assert!(caption_rule.contains("font-weight: var(--fandhe-font-font-weight-medium);"));
        assert!(caption_rule.contains("font-size: var(--fandhe-font-font-size-xs);"));
        assert!(caption_rule.contains("padding: var(--fandhe-space-3) 0;"));
        assert!(caption_rule.contains("text-align: inherit;"));
        assert!(caption_rule.contains("caption-side: bottom;"));
        assert!(!caption_rule.contains("0.75rem 0;"));
    }

    /// イシュー #1572: `Outline` variant が行罫線を維持しヘッダー背景に
    /// `bg-subtle` を使うこと、`Line` はヘッダー背景に `bg` を使うことを
    /// 固定する（モジュール doc「`Outline` variant の行罫線・ヘッダー背景
    /// の是正」節参照）。
    #[test]
    fn table_css_outline_variant_keeps_row_border_and_subtle_header_bg() {
        let out = css();
        let outline_rule_start = out
            .find(r#"[data-scope="table"][data-part="root"].fd-table--variant-outline {"#)
            .expect("Outline root variant 規則が css() 出力に存在すること");
        let outline_rule_end = out[outline_rule_start..]
            .find('}')
            .map(|offset| outline_rule_start + offset)
            .expect("Outline root variant 規則が `}` で閉じられていること");
        let outline_rule = &out[outline_rule_start..outline_rule_end];
        assert!(outline_rule
            .contains("--fandhe-table-row-border: 1px solid var(--fandhe-color-border-muted);"));
        assert!(outline_rule.contains("--fandhe-table-header-bg: var(--fandhe-color-bg-subtle);"));
        assert!(outline_rule.contains("--fandhe-table-last-row-border: none;"));

        let line_rule_start = out
            .find(r#"[data-scope="table"][data-part="root"].fd-table--variant-line {"#)
            .expect("Line root variant 規則が css() 出力に存在すること");
        let line_rule_end = out[line_rule_start..]
            .find('}')
            .map(|offset| line_rule_start + offset)
            .expect("Line root variant 規則が `}` で閉じられていること");
        let line_rule = &out[line_rule_start..line_rule_end];
        assert!(line_rule.contains("--fandhe-table-header-bg: var(--fandhe-color-bg);"));
    }

    /// イシュー #1572: `row` の最終要素で `--fandhe-table-row-border` が
    /// `--fandhe-table-last-row-border` custom property の値で上書き
    /// されることを固定する（モジュール doc「`Outline` variant の行罫線・
    /// ヘッダー背景の是正」節参照）。`row` slot 自体に base 規則が無いこと
    /// （下記 `table_css_puts_row_border_on_cell_not_row` 相当の既存前提）
    /// とは両立する（`:last-child` 付きセレクタは base ではないため）。
    #[test]
    fn table_css_last_row_suppresses_border_via_custom_property() {
        let out = css();
        assert!(out.contains(r#"[data-scope="table"][data-part="row"]:last-child {"#));
        let rule_start = out
            .find(r#"[data-scope="table"][data-part="row"]:last-child {"#)
            .expect("row:last-child 規則が css() 出力に存在すること");
        let rule_end = out[rule_start..]
            .find('}')
            .map(|offset| rule_start + offset)
            .expect("row:last-child 規則が `}` で閉じられていること");
        let rule = &out[rule_start..rule_end];
        assert!(rule.contains("--fandhe-table-row-border: var(--fandhe-table-last-row-border);"));
    }

    /// イシュー #1572: `scroll-area` が `overflow: auto` のスクロール
    /// コンテナであり、`:focus-visible` に inset フォーカスリングを持つ
    /// ことを固定する（モジュール doc「`scroll-area` パーツ」節参照）。
    #[test]
    fn table_css_scroll_area_is_scroll_container_with_inset_focus_ring() {
        let out = css();
        let base_rule_start = out
            .find(r#"[data-scope="table"][data-part="scroll-area"] {"#)
            .expect("scroll-area base 規則が css() 出力に存在すること");
        let base_rule_end = out[base_rule_start..]
            .find('}')
            .map(|offset| base_rule_start + offset)
            .expect("scroll-area base 規則が `}` で閉じられていること");
        let base_rule = &out[base_rule_start..base_rule_end];
        assert!(base_rule.contains("overflow: auto;"));
        assert!(base_rule.contains("max-height: var(--fandhe-table-scroll-max-height, none);"));

        let focus_rule_start = out
            .find(r#"[data-scope="table"][data-part="scroll-area"]:focus-visible {"#)
            .expect("scroll-area :focus-visible 規則が css() 出力に存在すること");
        let focus_rule_end = out[focus_rule_start..]
            .find('}')
            .map(|offset| focus_rule_start + offset)
            .expect("scroll-area :focus-visible 規則が `}` で閉じられていること");
        let focus_rule = &out[focus_rule_start..focus_rule_end];
        assert!(
            focus_rule.contains("outline-offset: calc(-1 * var(--fandhe-focus-ring-offset, 2px));")
        );
    }

    /// イシュー #1572: `scroll_area` パーツが実際のレンダリング出力で
    /// recipe が想定するセレクタ（`[data-scope="table"][data-part=
    /// "scroll-area"]`）と一致することを固定する。
    #[test]
    fn table_recipe_selectors_match_actual_rendered_markup_scroll_area() {
        let html = render(&scroll_area(vec![], vec![]));
        assert!(html.starts_with(r#"<div data-scope="table" data-part="scroll-area""#));
    }

    /// イシュー #2052: `interactive` variant（false/true）のクラスセレクタ・
    /// root スコープ custom property が `css()` 出力に含まれることを固定
    /// する（`sticky_header` と同型）。
    #[test]
    fn table_css_contains_interactive_variants() {
        let out = css();
        assert!(
            out.contains(r#"[data-scope="table"][data-part="root"].fd-table--interactive-false {"#)
        );
        assert!(
            out.contains(r#"[data-scope="table"][data-part="root"].fd-table--interactive-true {"#)
        );
        assert!(out.contains("--fandhe-table-row-hover-bg: transparent;"));
        assert!(out.contains("--fandhe-table-row-hover-bg: var(--fandhe-color-bg-muted);"));
    }

    /// イシュー #2052: `Off`（interactive: false）は `--fandhe-table-
    /// row-hover-bg` を `transparent` にするため、hover 規則
    /// （`background-image` longhand）が striped 縞を消さないことを固定
    /// する（モジュール doc「`interactive` 軸（行 hover）」節「`background`
    /// shorthand ではなく `background-image` longhand を使う理由」参照）。
    #[test]
    fn table_css_row_hover_uses_background_image_longhand_not_shorthand() {
        let out = css();
        assert!(out.contains(
            "background-image: linear-gradient(var(--fandhe-table-row-hover-bg), \
             var(--fandhe-table-row-hover-bg));"
        ));
        // `@media (hover: hover)` 配下の HoverExceptAttr 規則セレクタが
        // data-selected を除外していることを確認する。
        assert!(out.contains(
            r#"[data-scope="table"][data-part="row"]:hover:not([data-disabled]):not([data-selected]) {"#
        ));
    }

    /// イシュー #2052: 選択行（`data-selected`）の背景・文字色が
    /// `crate::tree_view` と同型のペア（`accent-subtle`/`accent-fg-subtle`）
    /// であることと、`:nth-child(even)`（striped）規則より後に出力される
    /// ことを固定する（モジュール doc「`data-selected` 行状態」節参照。
    /// 両者とも specificity (0,3,0) のためソース順の後勝ちで選択が縞に
    /// 勝つ契約）。
    #[test]
    fn table_css_selected_row_declares_accent_pair_after_striped_rule() {
        let out = css();
        let selected_selector = r#"[data-scope="table"][data-part="row"][data-selected] {"#;
        assert!(out.contains(selected_selector));
        let selected_rule_start = out.find(selected_selector).unwrap();
        let selected_rule_end = out[selected_rule_start..]
            .find('}')
            .map(|offset| selected_rule_start + offset)
            .unwrap();
        let selected_rule = &out[selected_rule_start..selected_rule_end];
        assert!(selected_rule.contains("background: var(--fandhe-color-accent-subtle);"));
        assert!(selected_rule.contains("color: var(--fandhe-color-accent-fg-subtle);"));

        let striped_selector = r#"[data-scope="table"][data-part="row"]:nth-child(even) {"#;
        let striped_rule_start = out
            .find(striped_selector)
            .expect("striped 規則が css() 出力に存在すること");
        assert!(
            striped_rule_start < selected_rule_start,
            "data-selected 規則は :nth-child(even)（striped）規則より後に出力されなければならない"
        );
    }

    /// イシュー #2052: `cell`/`column-header` の `data-align`
    /// （start/center/end）状態規則が `css()` 出力に含まれることを固定する
    /// （モジュール doc「`data-align` セル整列」節参照）。
    #[test]
    fn table_css_contains_data_align_state_rules_for_cell_and_column_header() {
        let out = css();
        for part in ["cell", "column-header"] {
            for (value, text_align) in [("start", "start"), ("center", "center"), ("end", "end")] {
                let selector =
                    format!(r#"[data-scope="table"][data-part="{part}"][data-align="{value}"] {{"#);
                assert!(out.contains(&selector), "missing {selector}\n{out}");
                let decl = format!("text-align: {text_align};");
                let rule_start = out.find(&selector).unwrap();
                let rule_end = out[rule_start..]
                    .find('}')
                    .map(|offset| rule_start + offset)
                    .unwrap();
                assert!(
                    out[rule_start..rule_end].contains(&decl),
                    "missing {decl} in rule for {selector}"
                );
            }
        }
    }

    /// イシュー #2052: `column_header` は `aria-sort` 等の呼び出し側属性を
    /// そのまま通過させる（生産しない。`aria-sort` の生産は headless
    /// data-table〔#2124〕の責務、モジュール doc「スコープ外」節参照）。
    #[test]
    fn column_header_passes_through_caller_aria_sort_attribute() {
        let html = render(&column_header(
            vec![("aria-sort", "ascending")],
            vec![text("Name")],
        ));
        assert!(html.contains(r#"aria-sort="ascending""#));
    }

    /// イシュー #2052: 既定 `row`/`cell`/`column_header` は
    /// `data-scope`/`data-part` 以外の `data-*` を自己出力しない
    /// （`data_attr_vocabulary.rs` の caller-sourced 契約と対をなす最小
    /// 回帰、`crates/pre-styled-ui/tests/data_attr_vocabulary.rs` の
    /// `table_row_and_cell_data_attrs_are_caller_sourced_not_self_emitted`
    /// も参照）。
    #[test]
    fn row_and_cell_emit_no_self_produced_data_attrs_by_default() {
        let row_html = render(&row(vec![], vec![]));
        assert_eq!(row_html.matches("data-").count(), 2);

        let cell_html = render(&cell(vec![], vec![]));
        assert_eq!(cell_html.matches("data-").count(), 2);
    }
}
