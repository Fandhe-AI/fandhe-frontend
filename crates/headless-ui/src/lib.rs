//! `fandhe-frontend-headless-ui`: headless UI コンポーネント層（外部依存は
//! `fandhe-frontend-core` / `fandhe-frontend-interactive`（いずれも path）のみ）。
//!
//! ark-ui 相当の headless（unstyled）UI コンポーネント層を提供する。
//! anatomy（部品構成）・`data-*` 属性・WAI-ARIA 属性付与のための共通 API を
//! 本クレートで整備し、その上に styled 層（`fandhe-frontend-pre-styled-ui`、
//! イシュー #546）が重なる 2 層構造の下層を担う（親トラッキング #520、
//! Phase 1 親 #521）。
//!
//! # 本クレートの不変条件（REQ-1・REQ-2・REQ-5、`.claude/rules/coding-rust.md`）
//!
//! 1. コンポーネントは [`fandhe_frontend_core::Node`] を返す通常の Rust 関数として
//!    実装する（REQ-5、マクロ DSL は採用しない）。
//! 2. 出力は [`fandhe_frontend_core::render`] の既定エスケープを必ず経由する。
//!    **本クレート内では `raw_html()` を使用しない**（新たなエスケープ迂回経路を
//!    作らない）。
//! 3. **`unsafe` コード禁止**: `#![forbid(unsafe_code)]` によりクレート全体で
//!    機械的に禁止する（`crates/core/tests/unsafe_boundary.rs` が workspace
//!    member を自動発見して強制する）。
//! 4. **外部依存は `fandhe-frontend-core` / `fandhe-frontend-interactive`
//!    （いずれも path）のみ**: `headless-ui/Cargo.toml` の `[dependencies]` に
//!    サードパーティクレートを追加しない。
//!
//! # 再エクスポート契約（イシュー #550・#712）
//!
//! [`fandhe_frontend_core`]・[`fandhe_frontend_interactive`] はクレートその
//! ものを再エクスポートする（型/値単位の個別再エクスポートではない）。これに
//! より本クレートのみに依存する利用者も `fandhe_frontend_headless_ui::fandhe_frontend_core`
//! / `fandhe_frontend_headless_ui::fandhe_frontend_interactive` 経由で
//! `Component`/`Hydrate`/`dispatch`/`HydrateError`/`render_for_hydration` を
//! 含む hydration API まで単独依存で到達できる（`docs/api/headless-ui-api.md`
//! 参照）。ルート直下への個別型再エクスポート（例: `pub use
//! fandhe_frontend_interactive::Component;`）は、実利用パスが確認できるまで
//! 見送る判断とした（`dispatch` 等の汎用名を UI クレートのルートへ置くと
//! 名前衝突・責務の混濁を招くため）。
//!
//! # 実装済み API（イシュー #523/#524）
//!
//! - [`mod@anatomy`]: `data-scope` / `data-part` を付与してパーツノードを組み立てる
//!   [`anatomy::Anatomy`]（全コンポーネント共通の anatomy 基盤）。
//! - [`data_attrs`]: `data-state` / `data-disabled` 等の状態属性ヘルパ（#523）。
//! - [`aria`]: `role` / `aria-*` の WAI-ARIA 属性ヘルパ（#523）。
//! - [`state`]: `fandhe-frontend-interactive` の
//!   [`fandhe_frontend_interactive::Component`]/[`fandhe_frontend_interactive::Hydrate`]
//!   抽象へ乗る状態機械（[`state::Disclosure`]/[`state::SingleSelect`]/
//!   [`state::MultiSelect`]/[`state::Checkable`]、#524・#594・#595）。
//!   Dialog / Accordion / Tabs / Collapsible / Popover / Tooltip（Phase 2 の
//!   #526〜#533）が共通で使う「open/closed・selected」の dispatch 契約・
//!   `data-state` 整合・SSR/hydration 契約、および Switch / Checkbox /
//!   RadioGroup が共通で使う「checked/unchecked」の同契約をここに一度だけ
//!   実装し、各コンポーネントはフィールドとして埋め込んで再利用する。
//!   [`state::MultiSelect`]（#594）は 0 個以上の同時選択
//!   （[`accordion::MultiAccordion`] の multiple モード）向けに
//!   [`state::SingleSelect`]（高々 1 個選択）を補完する。[`state::Checkable`]
//!   はイシュー #595 で [`mod@switch`] から共通化昇格した。[`state::TextInput`]
//!   （#749）は自由入力文字列 1 個を持つ状態機械であり、[`mod@combobox`] が
//!   埋め込む。
//! - [`mod@tabs`]: WAI-ARIA APG の Tabs パターンに準拠したマークアップを組み立てる
//!   [`tabs::tabs`]（#528）。SSR 時点の静的な選択状態のみを扱い、クリック操作・
//!   状態機械連携は後続イシューのスコープ。[`tabs::TabsProps`] の
//!   `activation_mode`/`loop_focus`（イシュー #582）は `list` パーツへ
//!   `data-activation-mode`/`data-loop-focus` として出力され、
//!   `fandhe-frontend-wasm-full` の `keynav` モジュールがキーボード操作時の
//!   活性化タイミング・フォーカス循環を分岐するために読む契約。
//!   [`tabs::TabsProps::indicator`]（イシュー #601、既定 `false` の opt-in）
//!   は選択タブの位置を示す `indicator` パーツを追加し、SSR では
//!   `data-*` フックと CSS 変数（`--left`/`--top`/`--width`/`--height`）の
//!   初期値のみを出力する（動的計測は wasm/CSR 層の後続責務）。
//! - [`mod@collapsible`]: Root/Trigger/Indicator/Content の anatomy パーツ関数群と、
//!   [`state::Disclosure`] を埋め込んだ [`collapsible::Collapsible`] 状態機械
//!   （#529、親 #526）。Phase 2 で [`state`] を具象コンポーネントへ適用する最初の例。
//! - [`mod@accordion`]: Root / Item / ItemTrigger / ItemIndicator / ItemContent の
//!   5 anatomy パーツと [`state::SingleSelect`] を埋め込んだ single モード
//!   Accordion（[`accordion::Accordion`]、#527）、および
//!   [`state::MultiSelect`] を埋め込んだ multiple モード Accordion
//!   （[`accordion::MultiAccordion`]、#594）。
//! - [`mod@tooltip`]: Root/Trigger/Positioner/Content/Arrow/ArrowTip の anatomy
//!   パーツ関数群と、[`state::Disclosure`] を埋め込んだ [`tooltip::Tooltip`]
//!   状態機械（#533、親 #530）。WAI-ARIA tooltip パターンに従い `aria-describedby`
//!   を使う点が [`mod@collapsible`] との違い。
//! - [`mod@dialog`]: [`dialog::Dialog`] — Root / Trigger / Backdrop /
//!   Positioner / Content / Title / Description / CloseTrigger の 8 anatomy
//!   パーツと [`state::Disclosure`] を埋め込んだモーダルダイアログ（#531）。
//! - [`mod@drawer`]: Dialog パターンの変種（画面端からスライドインするパネル）
//!   である [`drawer::Drawer`]。dialog と同じ 8 anatomy パーツ（`data-scope="drawer"`）
//!   を持つが、開閉状態機械は新設せず [`dialog::Dialog`] へ全委譲する
//!   （[`segment_group::SegmentGroup`] が [`radio_group::RadioGroup`] へ
//!   全委譲するのと同型のパターン）。固有に持つのは画面端の方向を表す
//!   [`drawer::DrawerPlacement`]（`data-placement`）のみ（#758）。
//! - [`mod@radio_group`]: Root / Label / Item / ItemControl / ItemText /
//!   ItemHiddenInput の 6 anatomy パーツと [`state::SingleSelect`] を埋め込んだ
//!   [`radio_group::RadioGroup`]（#536、親 #534）。クライアント由来の文字列
//!   dispatch は `"select"` のみを受理する（WAI-ARIA radio パターンに選択解除
//!   ジェスチャは存在しないため、型付き API の `Deselect` のみプログラム的な
//!   選択解除を許す）。
//! - [`mod@segment_group`]: Root / Indicator / Item / ItemText / ItemControl /
//!   ItemHiddenInput の 6 anatomy パーツと、状態機械・dispatch・hydration の
//!   すべてを [`radio_group::RadioGroup`] へ全委譲する
//!   [`segment_group::SegmentGroup`]（#743、親トラッキング #520）。segmented
//!   control は WAI-ARIA radio パターンそのものであるため独自の状態機械を
//!   新設しない。固有に持つのは segment 用 anatomy と、選択項目の
//!   `(index, count)` から CSS カスタムプロパティ 2 種を導出する
//!   [`segment_group::indicator`] の SSR 決定的な位置表現のみ（詳細は
//!   [`mod@segment_group`] module doc 参照）。
//! - [`popover`]: Root / Trigger / Anchor / Positioner / Arrow / ArrowTip /
//!   Content / Title / Description / CloseTrigger / Indicator の 11 anatomy
//!   パーツと [`state::Disclosure`] を埋め込んだ [`popover::Popover`] を提供する
//!   headless Popover コンポーネント（#532）。
//! - [`mod@field`]: Root / Label / Input / Textarea / Select / HelperText /
//!   ErrorText / RequiredIndicator の 8 anatomy パーツ関数群
//!   （[`field::FieldProps`] から決定的に描画する純粋関数、#538）。
//!   `invalid`/`disabled`/`required`/`readonly` は SSR 静的な props であり、
//!   開閉のような時間変化する内部状態を持たないため [`mod@state`] の状態機械を
//!   適用しない（[`mod@tabs`] と同型の判断）。[`field::FieldProps::ids`]
//!   （[`field::FieldIds`]）による派生 id の個別上書き、[`field::textarea`]
//!   の `autoresize` 引数（`data-autoresize` フックのみ）、[`field::select`]
//!   の readonly 解消（`<select readonly>` は無効な HTML のためネイティブ
//!   属性を出力しない）はイシュー #602 で追加。
//! - [`mod@fieldset`]: Root / Legend / HelperText / ErrorText の 4 anatomy
//!   パーツ関数群（[`fieldset::FieldsetProps`] から決定的に描画する純粋関数、
//!   #602、親 #578）。[`fieldset::FieldsetProps::merge_field_props`] で
//!   `disabled` を内包する [`field::FieldProps`] へ OR 伝播する（`invalid` は
//!   伝播しない）。[`mod@field`] と同じく状態機械を適用しない。
//! - [`mod@listbox`]: Root / Label / Content / ItemGroup / ItemGroupLabel /
//!   Item / ItemText / ItemIndicator / ValueText の 9 anatomy パーツと、
//!   single モード [`state::SingleSelect`] を埋め込んだ
//!   [`listbox::Listbox`]、multiple モード [`state::MultiSelect`] を
//!   埋め込んだ [`listbox::MultiListbox`]（#750、親 #748）。
//!   [`mod@select`]（ポップアップ型、`Disclosure` + trigger/positioner/
//!   hidden-select を持つ）とは異なり、Listbox は常時展開で開閉状態を
//!   持たない（責務境界の詳細は [`mod@listbox`] module doc 参照）。
//! - [`mod@menu`]: Root / Trigger / Indicator / Positioner / Content / Arrow /
//!   ArrowTip / Item / ItemGroup / ItemGroupLabel / Separator / TriggerItem /
//!   ContextTrigger / CheckboxItem / RadioItemGroup / RadioItem の 16 anatomy
//!   パーツと [`state::Disclosure`] を埋め込んだ [`menu::Menu`]
//!   （headless Menu コンポーネント、#540/#598）。構造上最も近い先行例は
//!   [`popover::Popover`]（trigger 起点のオーバーレイ + `Disclosure` 埋め込み）
//!   であり、本モジュールはそのパターンに完全準拠する。CheckboxItem/
//!   RadioItemGroup/RadioItem（#597）は checked 状態を [`state::Checkable`]/
//!   [`state::SingleSelect`] を埋め込んだ [`menu::MenuCheckboxItem`]/
//!   [`menu::MenuRadioItemGroup`] で表現し、`Menu` の開閉状態とは独立させる。
//! - [`mod@select`]: Root / Label / Control / Trigger / ValueText /
//!   ClearTrigger / Indicator / Positioner / Content / ItemGroup /
//!   ItemGroupLabel / Item / ItemText / ItemIndicator / HiddenSelect の 15
//!   anatomy パーツと、[`state::Disclosure`]（listbox 開閉）+
//!   [`state::SingleSelect`]（選択値）を合成した [`select::Select`] 状態機械
//!   （#541、親 #539）。Disclosure と SingleSelect を 1 コンポーネントに
//!   合成する初の例。[`select::item`] の `highlighted`/`id` 引数と
//!   [`select::content`] の `activedescendant` 引数が `data-highlighted`/
//!   `aria-activedescendant` の SSR 静的表現を提供する（イシュー #599）。
//! - [`mod@switch`]: Root / Control / Thumb / Label / HiddenInput の 5 anatomy
//!   パーツと、[`state::Checkable`] を埋め込んだ [`switch::Switch`] 状態機械
//!   （#537、親 #534）。ark-ui 準拠の `"checked"`/`"unchecked"` 値語彙が
//!   [`state::Disclosure`] の `"open"`/`"closed"` と異なるため
//!   [`state::Checkable`]（[`state::Disclosure`] とは別の共通機械）を
//!   埋め込む点が [`mod@collapsible`] との違い（#595 で共通化昇格するまでは
//!   本モジュール内に個別実装していた）。
//! - [`mod@avatar`]: Root / Image / Fallback の 3 anatomy パーツと、画像読み込み
//!   ステータス（`"loading"`/`"loaded"`/`"error"`）の [`avatar::Avatar`] 状態
//!   機械（#543、親 #542）。[`mod@switch`] と同様、[`state`] を埋め込まず
//!   [`fandhe_frontend_interactive::Component`]/
//!   [`fandhe_frontend_interactive::Hydrate`] を直接実装する（3 値ステータス
//!   が [`state::Disclosure`]/[`state::SingleSelect`] のいずれにも写像
//!   できないため）。`data-state`（`"visible"`/`"hidden"`）は Image/Fallback
//!   のみに付与し、ark-ui 準拠で Root には付与しない。
//! - [`mod@progress`]: Root / Label / ValueText / Track / Range の 5 anatomy
//!   パーツと、数値 `value`（`min`..=`max`、または indeterminate を表す
//!   `None`）を持つ [`progress::Progress`] 状態機械（#544、親 #542）。
//!   [`mod@switch`] と同じく `data-state` 値語彙（`"indeterminate"`/
//!   `"loading"`/`"complete"`）が [`state::Disclosure`] と異なるため、
//!   [`state`] を埋め込まず [`fandhe_frontend_interactive::Component`]/
//!   [`fandhe_frontend_interactive::Hydrate`] を直接実装する。加えて
//!   Circular（SVG）用の Circle/CircleTrack/CircleRange の 3 パーツ
//!   （#600、親 #542）を持つ。CSS 変数（`--size`/`--thickness`）参照の
//!   固定 `style` で描画し、状態機械・hydration フォーマットへの追加は
//!   ない（詳細は [`progress`] モジュール doc の circular 節を参照）。
//! - [`mod@number_input`]: Root / Label / Control / Input / IncrementTrigger /
//!   DecrementTrigger の 6 anatomy パーツと、数値 `value`（`min`..=`max`、
//!   または未入力を表す `None`）を持つ [`number_input::NumberInput`] 値状態
//!   機械（#738、親 #736）。[`mod@progress`] と同じく `data-state` を持たず、
//!   [`fandhe_frontend_interactive::Component`]/
//!   [`fandhe_frontend_interactive::Hydrate`] を直接実装する。ark-ui の
//!   Scrubber パーツ・キーボード操作の DOM 配線は本イシューのスコープ外
//!   （[`number_input`] モジュール doc 参照）。
//! - [`mod@rating_group`]: Root / Label / Control / Item / HiddenInput の 5
//!   anatomy パーツと、`1..=count` の数値評価値（未評価は `None`）+ hover
//!   プレビューを持つ [`rating_group::RatingGroup`] 状態機械（#742、親
//!   #736）。`hover` は SSR 非活性・hydration 非直列化（[`rating_group`]
//!   モジュール doc 参照）。`allow_half`（0.5 刻み）・hover/クリック/
//!   キーボードナビの DOM 配線は本イシューのスコープ外。
//! - [`mod@pin_input`]: Root / Label / Control / Input / HiddenInput の 5
//!   anatomy パーツと、固定桁数の文字配列 + フォーカス位置を持つ
//!   [`pin_input::PinInput`] 状態機械（#739、親 #736/#726）。[`mod@switch`]/
//!   [`mod@progress`] と同じく [`state`] の既存語彙に収まらないため、
//!   [`fandhe_frontend_interactive::Component`]/
//!   [`fandhe_frontend_interactive::Hydrate`] を直接実装する。
//! - [`mod@combobox`]: Root / Label / Control / Input / Trigger /
//!   ClearTrigger / Positioner / Content / ItemGroup / ItemGroupLabel /
//!   Item / ItemText / ItemIndicator の 13 anatomy パーツと、
//!   [`state::Disclosure`]（listbox 開閉）と [`state::SingleSelect`]（選択値）
//!   と [`state::TextInput`]（入力値、本イシューで新設）を合成した
//!   [`combobox::Combobox`] 状態機械（#749、親トラッキング #520）。候補列は
//!   [`mod@select`] と同じ `(value, label)` タプル列で表現し、
//!   [`combobox::filter_options`] が大文字小文字非区別の部分一致フィルタを
//!   提供する。ARIA 1.2 combobox パターンに準拠し `aria-activedescendant`
//!   は `content`（[`mod@select`]）ではなく `input` 側に配線する
//!   （[`combobox`] モジュール doc 参照）。フィルタの実 DOM 配線・
//!   キーボードナビゲーションは wasm 層の後続イシューのスコープ。
//! - [`mod@tags_input`]: Root / Label / Control / Input / Item / ItemPreview /
//!   ItemText / ItemInput / ItemDeleteTrigger / ClearTrigger / HiddenInput の
//!   11 anatomy パーツと、可変長タグ文字列リスト + 編集中インデックスを持つ
//!   [`tags_input::TagsInput`] 状態機械（#744、親 #736/#726）。[`mod@pin_input`]/
//!   [`mod@number_input`] と同じく [`state`] の既存語彙に収まらないため、
//!   [`fandhe_frontend_interactive::Component`]/
//!   [`fandhe_frontend_interactive::Hydrate`] を直接実装する。`control` は
//!   `role="listbox"`、`item_preview` は `role="option"`（イシュー本文が
//!   指定する listbox 相当の ARIA）。
//! - [`mod@carousel`]: Root / Control / PrevTrigger / NextTrigger /
//!   ItemGroup / Item / IndicatorGroup / Indicator の 8 anatomy パーツと、
//!   `0..slide_count` を循環し得る index 値を持つ [`carousel::Carousel`]
//!   状態機械（#754、親 #748/#520）。[`mod@slider`]/[`mod@number_input`] と
//!   同じく [`state`] の既存語彙に収まらないため、
//!   [`fandhe_frontend_interactive::Component`]/
//!   [`fandhe_frontend_interactive::Hydrate`] を直接実装する。`item` は
//!   `role="group"` + `aria-roledescription="slide"` + 位置ラベル
//!   （`"{n} of {m}"`）、`indicator` は `aria-current`（現在位置のみ）を
//!   出力する ARIA carousel パターン準拠。autoplay（play/pause/`aria-live`
//!   切替）・pointer ドラッグ/キーボード操作の DOM 配線は本イシューの
//!   スコープ外（[`carousel`] モジュール doc 参照）。
//! - [`mod@pagination`]: Root / Item / Ellipsis / PrevTrigger / NextTrigger の
//!   5 anatomy パーツと、[`pagination::page_range`]（総件数・ページサイズ・
//!   現在ページ・sibling/boundary 件数から省略記号を含むページ列を導出する
//!   決定的な純粋関数）、および [`fandhe_frontend_interactive::Component`]/
//!   [`fandhe_frontend_interactive::Hydrate`] を直接実装する
//!   [`pagination::Pagination`] 値状態機械を提供する（#751、
//!   `docs/api/headless-ui-api.md` §4b.3 の保留を解除、先行判断は #716）。
//!   [`mod@number_input`]/[`mod@progress`] と同じく `data-state` を持たず、
//!   現在ページは `aria-current="page"`/`data-selected` で、端到達は
//!   `disabled`/`data-disabled` で表現する。ページ列生成は
//!   `O(boundary_count + sibling_count)` で `total_pages` を全列挙しない
//!   （巨大 `count` でも有界、モジュール doc 参照）。wasm 層のクリック配線・
//!   キーボードナビゲーションは本イシューのスコープ外。
//! - [`mod@hover_card`]: Root/Trigger/Positioner/Content/Arrow/ArrowTip の
//!   6 anatomy パーツ関数群と、[`state::Disclosure`] を埋め込んだ
//!   [`hover_card::HoverCard`] 状態機械（#759、親トラッキング #726）。
//!   [`mod@tooltip`] に最も近い構造だが、trigger がリンク先プレビュー用途の
//!   `a` 要素である点が異なる。`openDelay`/`closeDelay`
//!   （[`hover_card::HoverCardDelays`]、ark-ui 既定 600ms/300ms）は
//!   決定的な SSR 設定値として `root` の `data-open-delay`/
//!   `data-close-delay` へ出力するのみで、実タイマー駆動・DOM 読み取り
//!   配線は `fandhe-frontend-wasm-full` の後続イシューのスコープ
//!   （[`hover_card`] モジュール doc §スコープ外参照）。
//! - [`mod@toggle_tip`]: Root / Trigger / Positioner / Content / Arrow /
//!   ArrowTip の 6 anatomy パーツと、[`state::Disclosure`] を埋め込んだ
//!   [`toggle_tip::ToggleTip`] 状態機械（#761、親トラッキング #520）。
//!   chakra-ui の ToggleTip（「見た目は Tooltip・挙動は Popover」の変種）に
//!   倣い、[`toggle_tip::trigger`] は `aria-expanded`/`aria-controls` を持つが
//!   `aria-haspopup` は付与せず、[`toggle_tip::content`] は `role="tooltip"`
//!   を持たない（[`mod@tooltip`]・[`mod@popover`] との 3 者境界は
//!   [`mod@toggle_tip`] モジュール doc §3 者境界参照）。click-outside
//!   dismiss・Escape 閉鎖の DOM 配線は本イシューのスコープ外。
//! - [`mod@visually_hidden`]: `root`（`span`）1 anatomy パーツ（イシュー #776、
//!   親 #766）。視覚的には隠すが支援技術には読ませ続けるテキストコンテナで、
//!   [`mod@field`]/[`mod@link`] と同型の状態機械なし純粋関数。`aria-hidden` を
//!   一切出力しない不変条件がある（[`visually_hidden`] モジュール doc §`aria-hidden`
//!   を付けない不変条件 参照）。
//! - [`mod@skip_nav`]: `link`（`a`）/ `content`（`div`）の 2 anatomy パーツ
//!   （イシュー #776、親 #766）。WCAG 2.1 SC 2.4.1 Bypass Blocks 対応の
//!   「本文へスキップ」リンク。[`skip_nav::link`] は呼び出し側から任意の
//!   URL を受け取らず常に `#<id>` のみを組み立てるため、スキーム注入経路を
//!   構造的に持たない（[`mod@skip_nav`] モジュール doc §href の構成 参照）。
//! - [`mod@clipboard`]: Root / Label / Control / Input / Trigger / Indicator /
//!   ValueText の 7 anatomy パーツと、コピー済みかどうかの 2 値状態機械
//!   [`clipboard::Clipboard`]（#773、親トラッキング #520）。[`mod@avatar`]/
//!   [`mod@switch`] と同じく [`state`] の既存語彙に収まらないため、
//!   [`fandhe_frontend_interactive::Component`]/
//!   [`fandhe_frontend_interactive::Hydrate`] を直接実装する。コピー済み
//!   表示は `data-state` 値語彙ではなく `data-copied`
//!   （[`data_attrs::data_copied`]）存在属性で表現する ark-ui/chakra-ui の
//!   慣習に従う。コピー対象値（`value`）は状態機械に持たせず、
//!   [`clipboard::root`]の `data-value` 属性としてのみ出力する（[`clipboard`] モジュール doc
//!   「`value` は状態機械に持たせない」節参照）。`navigator.clipboard`
//!   実配線・タイムアウトによる自動リセットは
//!   `fandhe-frontend-wasm-full`（#773 後続）のスコープ。
//!
//! # `fandhe-frontend-core` の再エクスポート（イシュー #550）
//!
//! `fandhe-frontend-pre-styled-ui` は方針上 `fandhe-frontend-core` を直接の
//! ランタイム依存に持たず（`crates/pre-styled-ui/Cargo.toml` 参照。`core` は
//! dev-dependency のみ）、styled 部品が組み立てる [`fandhe_frontend_core::Node`]
//! への型参照は本クレート経由の間接依存で得る契約になっている。そのため
//! [`fandhe_frontend_core`] クレート自体を本クレートのルートから再エクスポート
//! する（`pre_styled_ui` 側は `fandhe_frontend_headless_ui::fandhe_frontend_core::Node`
//! のようにアクセスする）。新規の外部依存追加ではなく、既存 path 依存の可視性を
//! 広げるだけであり `structure.toml` の depends_on 検証には影響しない。
//!
//! いずれも [`fandhe_frontend_core::el`] への薄い委譲・属性タプルの組み立てに
//! 留め、独自のエスケープ経路や HTML 文字列組み立てを持たない
//! （`docs/api/component-api.md` 不変条件準拠）。`data-state` 属性名自体は
//! [`data_attrs::data_state`] が一元管理し、[`state`] モジュールはそれを
//! 呼び出して値（`"open"`/`"closed"`）を決める側に徹する（属性名の重複定義を
//! 避ける）。各コンポーネントの anatomy 定義（Accordion / Dialog 等の parts
//! 一覧）は Phase 2（#526〜#544）のスコープ。
//!
//! - [`mod@checkbox`]: ark-ui Checkbox 相当の anatomy（イシュー #535）と、
//!   [`state::Checkable`] を埋め込んだ [`checkbox::Checkbox`] 状態機械
//!   （dispatch 統合、#595）。indeterminate（3 値目）は
//!   [`state::Checkable`] のスコープ外のため SSR 静的 props
//!   （[`checkbox::CheckedState`]）としてのみ表現する。
//! - [`mod@positioning`]: anchor positioning の位置計算純粋関数モジュール
//!   （[`positioning::compute_position`]、イシュー #590、親 #588、正の規範
//!   文書は `docs/design/anchor-positioning-design.md`）。12 placement 語彙
//!   （[`positioning::Placement`]）・flip/shift/sameWidth・CSS 変数出力
//!   （[`positioning::css_vars_style`]）を提供し、Popover/Tooltip/Menu/Select
//!   の `positioner`/`arrow`/`arrow_tip` が「CSS フックのみ」だったスコープ
//!   外事項を解消する。実 DOM 計測は `fandhe-frontend-wasm-full`（`position`
//!   モジュール）の責務であり、本クレートは `web-sys` 非依存のまま維持する。
//! - [`mod@toggle`]: ark-ui Toggle 相当の Root/Indicator anatomy と、
//!   [`state::Checkable`] を埋め込んだ [`toggle::Toggle`] 状態機械
//!   （イシュー #746）。Switch と同じ [`state::Checkable`] を再利用しつつ
//!   `data-state` 語彙は `"on"`/`"off"`（[`state::pressed_data_state`]）で
//!   分離する（意味論差はモジュール doc 参照）。
//! - [`mod@toggle_group`]: Root/Item anatomy と、[`state::SingleSelect`] を
//!   埋め込んだ single モード [`toggle_group::ToggleGroup`]、
//!   [`state::MultiSelect`] を埋め込んだ multiple モード
//!   [`toggle_group::MultiToggleGroup`]（イシュー #746）。roving focus は
//!   wasm keynav 層のスコープとして未提供（モジュール doc §out-of-scope 参照）。
//! - [`mod@tree_view`]: Root / Label / Tree / Branch / BranchControl /
//!   BranchIndicator / BranchText / BranchContent / BranchIndentGuide / Item /
//!   ItemText / ItemIndicator の 12 anatomy パーツと、[`state::MultiSelect`]
//!   （展開中のブランチ値の集合）+ [`state::SingleSelect`]（選択中のノード値）
//!   を合成した [`tree_view::TreeView`] 状態機械（#753、親トラッキング
//!   #748/#520）。ツリーデータは [`tree_view::TreeNode`]（決定的な静的
//!   コレクション）で表現し、[`tree_view::TreeView::render_nodes`] が深さ・
//!   `aria-posinset`/`aria-setsize` を再帰的に計算しながら描画する。両埋め込み
//!   状態機械がともに `"selected"` フィールド名を使うため、hydration 属性名の
//!   衝突回避（展開集合側のみ `"expanded"` へ書き換え）を行う点が
//!   [`mod@combobox`] 以前の合成例と異なる（[`tree_view`] モジュール doc
//!   §hydration フィールド名 参照）。キーボードナビゲーション・checkbox
//!   モード・複数選択・lazy loading は本イシューのスコープ外。
//! - [`mod@breadcrumb`]: `root`（`nav`）/ `list`（`ol`）/ `item`（`li`）/
//!   `link`（`a`）/ `current-link`（`span`）/ `separator`（`li`）/
//!   `ellipsis`（`li`）の 7 anatomy パーツと利便ビルダー
//!   [`breadcrumb::breadcrumb`]（イシュー #755、`docs/api/headless-ui-api.md`
//!   §4b の追加候補消化）。[`mod@field`]/[`mod@tabs`] と同じく SSR 静的な
//!   意味論ナビであり状態機械を持たない。現在位置は `aria-current="page"`
//!   （[`aria::AriaCurrent`]）+ `data-current`（[`data_attrs::data_current`]）
//!   の併用で表現する。
//! - [`mod@link`]: `root`（`a`）1 anatomy パーツ（イシュー #756、
//!   `docs/api/headless-ui-api.md` §4b の追加候補消化）。`external`
//!   オプトインは `target="_blank"` + `rel="noopener noreferrer"` を不可分に
//!   付与する（reverse tabnabbing 対策）。`current` は [`mod@breadcrumb`] と
//!   同じ `aria-current`/`data-current` 語彙を共有する。
//! - [`mod@link_overlay`]: `root`（`div`）/ `overlay`（`a`）の 2 anatomy
//!   パーツ（イシュー #756）。chakra-ui の LinkBox/LinkOverlay パターンに
//!   倣い、カード全面クリック化を提供する。`::before` 疑似要素の代わりに
//!   `overlay` 自身を `position: absolute; inset: 0;` で展開する方式を採る
//!   （styled 層の CSS 責務、モジュール doc「全面拡張の実装方針」参照）。
//! - [`mod@nav_list`]: `root`（`nav`）/ `heading`（`h2`）/ `list`（`ul`）/
//!   `item`（`li`）/ `link`（`a`）の 5 anatomy パーツ（イシュー #756、#716
//!   最優先候補）。`docs/design/docs-site-styled-ui-adoption.md` §3.1 が
//!   指摘した「`menu` ロールの文書ナビへの誤転用」を解消するため、**`role`
//!   を一切付与しない**（モジュール doc 参照）。

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod accordion;
pub mod anatomy;
pub mod aria;
pub mod avatar;
pub mod breadcrumb;
pub mod carousel;
pub mod checkbox;
pub mod clipboard;
pub mod collapsible;
pub mod combobox;
pub mod data_attrs;
pub mod dialog;
pub mod drawer;
pub mod field;
pub mod fieldset;
pub mod hover_card;
pub mod link;
pub mod link_overlay;
pub mod listbox;
pub mod menu;
pub mod nav_list;
pub mod number_input;
pub mod pagination;
pub mod pin_input;
pub mod popover;
pub mod positioning;
pub mod progress;
pub mod qr_code;
mod qr_encode;
pub mod radio_group;
pub mod rating_group;
pub mod segment_group;
pub mod select;
pub mod skip_nav;
pub mod slider;
pub mod state;
pub mod switch;
pub mod tabs;
pub mod tags_input;
pub mod toggle;
pub mod toggle_group;
pub mod toggle_tip;
pub mod tooltip;
pub mod tree_view;
pub mod visually_hidden;

// `pub use fandhe_frontend_core;` はクレートそのものの再エクスポート（型/値の
// 再エクスポートではない）。`missing_docs` は extern crate 再エクスポートには
// 適用されないため doc コメントは不要（rustc の既定挙動）。
pub use fandhe_frontend_core;

// `pub use fandhe_frontend_interactive;` も同型のクレート再エクスポート
// （イシュー #712）。[`state`] モジュールが `fandhe_frontend_interactive::Component`/
// `Hydrate` を internal に利用しているが、hydration/dispatch まで書く利用者が
// `Component`/`Hydrate`/`dispatch`/`HydrateError`/`render_for_hydration` へ
// 到達するには、これまで本クレートを経由する手段がなく
// `fandhe-frontend-interactive` への直接依存を強いていた（半端な状態、
// PR #699/#695 の out-of-scope 節で検出）。クレート再エクスポートにすることで、
// 依存元の `fandhe-frontend-interactive` バージョン指定が本クレート内部の
// 依存（`Cargo.toml` 参照）とズレて「別バージョンの `Component` を実装している」
// というトレイト不一致エラーを踏む余地を無くす（core 再エクスポートと同じ動機、
// #550 に倣う）。`fandhe_frontend_interactive` は `raw_html()` を公開せず、
// `Component::view`/`render_for_hydration` の戻り値は `Node` のみで既定
// エスケープを必ず経由するため、この再エクスポートは新たなエスケープ迂回
// 経路を作らない（REQ-1 を弱めない）。
pub use fandhe_frontend_interactive;

pub use anatomy::{anatomy, Anatomy};
pub use aria::{
    aria_activedescendant, aria_autocomplete, aria_checked, aria_controls, aria_current,
    aria_describedby, aria_disabled, aria_expanded, aria_haspopup, aria_hidden, aria_invalid,
    aria_label, aria_labelledby, aria_modal, aria_orientation, aria_pressed, aria_roledescription,
    aria_selected, role, AriaAutocomplete, AriaChecked, AriaCurrent, AriaPopup,
};
pub use avatar::{Avatar, AvatarAction, ImageStatus};
pub use breadcrumb::{breadcrumb, BreadcrumbItem};
pub use carousel::{Carousel, CarouselAction};
pub use checkbox::{Checkbox, CheckboxFlags};
pub use clipboard::{Clipboard, ClipboardAction};
pub use combobox::{Combobox, ComboboxAction};
pub use data_attrs::{
    data_checked, data_copied, data_current, data_disabled, data_highlighted, data_invalid,
    data_orientation, data_pressed, data_readonly, data_required, data_state, Orientation,
};
pub use dialog::Dialog;
pub use drawer::{Drawer, DrawerPlacement};
pub use field::{FieldIds, FieldProps};
pub use fieldset::FieldsetProps;
pub use hover_card::{HoverCard, HoverCardDelays};
pub use menu::{Menu, MenuCheckboxItem, MenuRadioItemGroup};
pub use number_input::{NumberInput, NumberInputAction, NumberInputFlags};
pub use pagination::{ItemMode, PageEntry, Pagination, PaginationAction};
pub use pin_input::{PinInput, PinInputAction, PinInputKind};
pub use positioning::{
    compute_position, css_vars_style, data_align, data_side, placement_attrs, Align, ArrowPosition,
    Placement, PositioningConfig, Rect, ResolvedPosition, Side, Size,
};
pub use progress::{Progress, ProgressAction};
pub use qr_code::{ErrorCorrectionLevel, QrEncodeError, QrMatrix};
pub use radio_group::RadioGroup;
pub use rating_group::{RatingGroup, RatingGroupAction, RatingItemFlags};
pub use segment_group::SegmentGroup;
pub use slider::{Slider, SliderAction};
pub use state::{
    Checkable, CheckableAction, Disclosure, DisclosureAction, MultiSelect, MultiSelectAction,
    OpenState, SingleSelect, SingleSelectAction, TextInput, TextInputAction, DATA_STATE_CHECKED,
    DATA_STATE_CLOSED, DATA_STATE_OFF, DATA_STATE_ON, DATA_STATE_OPEN, DATA_STATE_UNCHECKED,
};
pub use switch::{Switch, SwitchAction};
pub use tabs::{tabs, ActivationMode, TabItem, TabsProps};
pub use tags_input::{TagsInput, TagsInputAction};
pub use toggle::{Toggle, ToggleAction};
pub use toggle_group::{MultiToggleGroup, ToggleGroup};
pub use toggle_tip::ToggleTip;
pub use tooltip::Tooltip;
pub use tree_view::{TreeNode, TreeView, TreeViewAction};
