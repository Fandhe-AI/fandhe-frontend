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
//! # 責務境界（2 層構成、`docs/policy/intentional-non-adoption.md` §3.25）
//!
//! 本クレートと `fandhe-frontend-pre-styled-ui` の 2 層構成における責務
//! 境界は `docs/policy/intentional-non-adoption.md` §3.25 が正であり、
//! 以下は要約のみ（規則本文はそちらを参照）。
//!
//! - **規則 1（非採用）**: 本クレートが担うのは anatomy（構造）・
//!   アクセシビリティ（WAI-ARIA・キーボード操作）・表示状態（`data-*`）
//!   までとする。バリデーション・送信処理・データ整形・永続化といった
//!   アプリケーションロジックを内包する部品は、参照軸（ark-ui /
//!   chakra-ui / Radix）に存在しても実装しない（Radix `Form` が確定
//!   対象。構造部分は [`mod@field`]/[`mod@fieldset`] が担う）。
//! - **規則 2（層の割り当て）**: 装飾・アニメーション・レイアウト計測の
//!   関心（Radix の `data-motion`、viewport 測定等）は本クレートへ
//!   持ち込まず、上層の `fandhe-frontend-pre-styled-ui`（実 DOM 計測は
//!   `fandhe-frontend-wasm-full`）の責務とする。適用例は
//!   [`mod@navigation_menu`]。
//! - **境界事例**: [`mod@positioning`] は viewport 寸法を引数で受け取るが
//!   計測主体（実 DOM 接触・再計算トリガーの所有）ではないため規則 2 の
//!   対象外である。判別根拠は同モジュールのモジュールレベル rustdoc を
//!   参照。
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
//! - [`mod@download_trigger`]: `root`（`a[download]`）1 anatomy パーツ
//!   （イシュー #828）。ark-ui/chakra-ui の DownloadTrigger（JS の `Blob`
//!   生成前提）を `a[download]` 属性による宣言的トリガーとして静的部品化
//!   したもので、[`mod@link`]/[`mod@breadcrumb`] と同型の状態機械なし純粋
//!   関数のみで構成する。
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
//!   DecrementTrigger / ValueText の 7 anatomy パーツ（#738、親 #736。
//!   ValueText はイシュー #1613 の参考サイト突合で追加）と、数値 `value`
//!   （`min`..=`max`、または未入力を表す `None`）を持つ
//!   [`number_input::NumberInput`] 値状態機械。[`mod@progress`] と同じく
//!   `data-state` を持たず、[`fandhe_frontend_interactive::Component`]/
//!   [`fandhe_frontend_interactive::Hydrate`] を直接実装する。ark-ui の
//!   Scrubber パーツ・キーボード操作の DOM 配線は引き続きスコープ外
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
//!   [`pin_input::PinInputProps`]（disabled/readonly/invalid/required）・
//!   `data-index`/`data-filled`・dispatch `"delete"`/`"prev"`/`"next"`の
//!   追加、`"backspace"` 挙動の是正は ark-ui/Radix 参照突合（イシュー
//!   #1615）による。
//! - [`mod@editable`]: Root / Label / Area / Input / Preview / Control /
//!   EditTrigger / SubmitTrigger / CancelTrigger の 9 anatomy パーツと、
//!   `preview`/`edit` の 2 モードを持つ [`editable::Editable`] 状態機械
//!   （#745、親 #736）。[`mod@switch`]/[`mod@progress`]/[`mod@pin_input`]
//!   と同じく [`state`] の既存語彙に収まらないため、
//!   [`fandhe_frontend_interactive::Component`]/
//!   [`fandhe_frontend_interactive::Hydrate`] を直接実装する。activationMode/
//!   submitMode の実挙動・autoResize は本イシューのスコープ外
//!   （[`editable`] モジュール doc 参照）。イシュー #1606 で ark-ui との
//!   参照突合を行い、`label`/`area`/`preview`/`input` へ
//!   [`editable::EditableInputFlags::invalid`]/`required` 由来の
//!   `data-invalid`/`data-required` を、`preview` へ `tabindex`/`aria-*` を
//!   追加した（詳細は [`editable`] モジュール doc「参照突合」節参照）。
//! - [`mod@combobox`]: Root / Label / Control / Input / Trigger /
//!   ClearTrigger / Positioner / Content / ItemGroup / ItemGroupLabel /
//!   Item / ItemText / ItemIndicator / LiveRegion の 14 anatomy パーツと、
//!   [`state::Disclosure`]（listbox 開閉）と [`state::SingleSelect`]（選択値）
//!   と [`state::TextInput`]（入力値、本イシューで新設）を合成した
//!   [`combobox::Combobox`] 状態機械（#749、親トラッキング #520）。候補列は
//!   [`mod@select`] と同じ `(value, label)` タプル列で表現し、
//!   [`combobox::filter_options`] が大文字小文字非区別の部分一致フィルタを
//!   提供する。ARIA 1.2 combobox パターンに準拠し `aria-activedescendant`
//!   は `content`（[`mod@select`]）ではなく `input` 側に配線する
//!   （[`combobox`] モジュール doc 参照）。フィルタの実 DOM 配線・
//!   キーボードナビゲーションは wasm 層の後続イシューのスコープ。
//!   `live_region` は候補件数の変化を通知する live region（`role="status"`
//!   + `aria-live="polite"` + `aria-atomic="true"` 固定、イシュー #1069）。
//! - [`mod@tags_input`]: Root / Label / Control / Input / Item / ItemPreview /
//!   ItemText / ItemInput / ItemDeleteTrigger / ClearTrigger / HiddenInput /
//!   LiveRegion の 12 anatomy パーツと、可変長タグ文字列リスト + 編集中
//!   インデックスを持つ [`tags_input::TagsInput`] 状態機械（#744、親
//!   #736/#726）。[`mod@pin_input`]/[`mod@number_input`] と同じく [`state`]
//!   の既存語彙に収まらないため、[`fandhe_frontend_interactive::Component`]/
//!   [`fandhe_frontend_interactive::Hydrate`] を直接実装する。`control` は
//!   `role="listbox"`、`item_preview` は `role="option"`（イシュー本文が
//!   指定する listbox 相当の ARIA）。`live_region` はタグ数の変化を通知する
//!   live region（[`mod@combobox`] の `live_region` と同型、イシュー #1069）。
//! - [`mod@file_upload`]: Root / Label / Dropzone / Trigger / ItemGroup /
//!   Item / ItemName / ItemSizeText / ItemDeleteTrigger / ClearTrigger /
//!   HiddenInput の 11 anatomy パーツと、ファイルメタデータ（[`file_upload::FileUploadItem`]:
//!   name / size_bytes / mime_type、`File` オブジェクト自体は非保持）の
//!   受理済み一覧 + 直近拒否履歴を持つ [`file_upload::FileUpload`] 状態機械
//!   （#840、`docs/policy/intentional-non-adoption.md` §7 の保留解除）。
//!   [`mod@tags_input`] と同じく [`state`] の既存語彙に収まらないため、
//!   [`fandhe_frontend_interactive::Component`]/
//!   [`fandhe_frontend_interactive::Hydrate`] を直接実装する。実 `File` API
//!   接触は `fandhe-frontend-wasm-full` 側に隔離する（[`mod@file_upload`]
//!   モジュール doc 参照）。disabled/readonly/invalid/required の状態束は
//!   [`file_upload::FileUploadProps`] が担い、item 系パーツの受理/拒否種別は
//!   [`file_upload::ItemType`] 固定語彙が `data-type` として表す
//!   （参照突合、イシュー #1609）。
//! - [`mod@steps`]: Root / List / Item / Trigger / Indicator / Separator /
//!   Content / CompletedContent / PrevTrigger / NextTrigger の 10 anatomy
//!   パーツと、`count`（全 step 数）+ `step`（現在位置、`0..=count`）を持つ
//!   [`steps::Steps`] 状態機械（#752、`docs/api/headless-ui-api.md` §4b.3
//!   の保留解除）。[`mod@progress`]/[`mod@pin_input`] と同じく [`state`] の
//!   既存語彙に収まらないため、[`fandhe_frontend_interactive::Component`]/
//!   [`fandhe_frontend_interactive::Hydrate`] を直接実装する。item は
//!   complete/current/incomplete の 3 状態を持ち、current な item の
//!   trigger のみ `aria-current="step"` を付与する。`linear`（順序強制）・
//!   `isStepValid`/`isStepSkippable`・キーボード操作/roving focus は本
//!   イシューのスコープ外（[`steps`] モジュール doc §out-of-scope 参照）。
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
//! - [`mod@action_bar`]: Root / Positioner / Content / SelectionTrigger /
//!   Separator / CloseTrigger の 6 anatomy パーツと [`state::Disclosure`] を
//!   埋め込んだ [`action_bar::ActionBar`] 状態機械（複数選択時に画面下部へ
//!   表示される操作バー、#762、親トラッキング #520）。構造上最も近い先行例は
//!   [`dialog::Dialog`]（`Disclosure` 埋め込み + positioner/close-trigger
//!   構成）であり、本モジュールはそのパターンに完全準拠する。`content` は
//!   `role="toolbar"` + `aria-label`、`separator` は `role="separator"` +
//!   `aria-orientation="vertical"` を出力する。選択件数から `open` を導出する
//!   糖衣 API は持たず、開閉は呼び出し側が dispatch（`"open"`/`"close"`/
//!   `"toggle"`）で制御する（[`action_bar`] モジュール doc §選択件数から
//!   open を導出する糖衣 API は持たない 参照）。
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
//! - [`mod@toast`]: group（live region）/ root / title / description /
//!   action-trigger / close-trigger の 6 anatomy パーツと、有界なキュー
//!   （`max` 超過時に最古を押し出す）を管理する [`toast::Toaster`] 状態機械
//!   （#760、親トラッキング #520）。[`mod@avatar`]/[`mod@progress`] と同じく
//!   [`state`] の既存語彙に収まらないため
//!   [`fandhe_frontend_interactive::Component`]/
//!   [`fandhe_frontend_interactive::Hydrate`] を直接実装する。`aria-live` は
//!   [`toast::ToastStatus`] から決定的に導出し（`Error` のみ `"assertive"`）、
//!   タイマーによる自動 dismiss の実配線・`"push"` の文字列 dispatch は
//!   `fandhe-frontend-wasm-full` の後続イシューのスコープ。
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
//! - [`mod@splitter`]: Root / Panel / ResizeTrigger / ResizeTriggerIndicator の
//!   4 anatomy パーツと、パネルサイズ状態機械 [`splitter::Splitter`]
//!   （#826、`docs/policy/intentional-non-adoption.md` §7・
//!   `docs/design/component-coverage-map.md` の「保留」を解除）。
//!   [`mod@slider`] と同じく [`state`] の既存語彙に収まらないため、
//!   [`fandhe_frontend_interactive::Component`]/
//!   [`fandhe_frontend_interactive::Hydrate`] を直接実装する。
//!   `resize-trigger` は `role="separator"` + `aria-valuemin`/`aria-valuemax`/
//!   `aria-valuenow`（先行パネルのサイズ%）+ `aria-orientation`（セパレータ
//!   自体の向き、パネルレイアウトの向きとは逆。[`splitter`] モジュール doc
//!   参照）+ `aria-controls`（先行パネル id）を出力する WAI-ARIA Window
//!   Splitter パターン準拠。パネル構成の正規化は fail-closed
//!   （[`splitter::Splitter::new`] 参照）。pointer ドラッグ・キーボード操作の
//!   DOM 配線・collapse/expand・`onResize`/`onCollapse` コールバックは本
//!   イシューのスコープ外（[`splitter`] モジュール doc §スコープ外参照）。
//! - [`mod@floating_panel`]: Root / Trigger / Positioner / Content / Header /
//!   Title / Control / StageTrigger / CloseTrigger / Body の 10 anatomy
//!   パーツと、[`state::Disclosure`]（開閉）+ 独自 [`floating_panel::Stage`]
//!   （default/minimized/maximized）+ 座標（x, y）を持つ
//!   [`floating_panel::FloatingPanel`] 状態機械（#827、`docs/policy/intentional-non-adoption.md`
//!   §7 の保留解除）。[`mod@popover`] と同じく [`state::Disclosure`] を
//!   埋め込みつつ、`stage` は既存語彙に収まらないため独自 enum とする
//!   （[`mod@steps`] と同型の判断）。座標出力（[`floating_panel::FloatingPanel::position_style`]）は
//!   [`mod@positioning`] の `--fandhe-x`/`--fandhe-y` 変数名を再利用するが、
//!   anchor 相対の placement 計算自体は行わない（ドラッグ操作によるビュー
//!   ポート絶対座標のため）。ドラッグ移動・リサイズの実 DOM 配線は
//!   `fandhe-frontend-wasm-full` の将来イシューのスコープ（詳細は
//!   [`floating_panel`] モジュール doc §スコープ外参照）。
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
//! - [`mod@checkbox_group`]: Root / Label / Item / ItemControl /
//!   ItemIndicator / ItemText の 6 anatomy パーツと [`state::MultiSelect`]
//!   を埋め込んだ [`checkbox_group::CheckboxGroup`]（複数選択グループ、
//!   #997、親 #534）。[`mod@radio_group`] と対称の構造だが、ネイティブ
//!   `<input type="checkbox">` は自前パーツを持たず [`checkbox::hidden_input`]
//!   の再利用で賄う。クライアント由来の文字列 dispatch は `"select"`/
//!   `"deselect"`/`"toggle"` の 3 語彙を受理する（WAI-ARIA checkbox パターン
//!   には選択解除ジェスチャが実在するため、[`radio_group::RadioGroup`] が
//!   `"select"` のみに絞るのとは意図的に異なる）。root/item 系パーツ共通の
//!   disabled/readonly/invalid 状態束は [`checkbox_group::CheckboxGroupProps`]
//!   が担う（イシュー #1603、参照突合で ark-ui `Checkbox.Group` props 相当を
//!   追加）。
//! - [`mod@positioning`]: anchor positioning の位置計算純粋関数モジュール
//!   （[`positioning::compute_position`]、イシュー #590、親 #588、正の規範
//!   文書は `docs/design/anchor-positioning-design.md`）。12 placement 語彙
//!   （[`positioning::Placement`]）・flip/shift/sameWidth・CSS 変数出力
//!   （[`positioning::css_vars_style`]）を提供し、Popover/Tooltip/Menu/Select
//!   の `positioner`/`arrow`/`arrow_tip` が「CSS フックのみ」だったスコープ
//!   外事項を解消する。実 DOM 計測は `fandhe-frontend-wasm-full`（`position`
//!   モジュール）の責務であり、本クレートは `web-sys` 非依存のまま維持する。
//! - [`mod@password_input`]: Root / Label / Control / Input /
//!   VisibilityTrigger / Indicator の 6 anatomy パーツと、表示切替
//!   （`"visible"`/`"hidden"`）の [`password_input::PasswordInput`] 状態機械
//!   （#740、親 #736）。[`mod@switch`]/[`mod@avatar`] と同じく、既存の
//!   [`state::Checkable`]/[`state::Disclosure`] のいずれとも値語彙が一致
//!   しないため [`state`] を埋め込まず個別実装する。**パスワード値そのもの
//!   は一切扱わない**（`value` を出力する API を持たない。セキュリティ
//!   不変条件はモジュール doc 参照）。
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
//! - [`mod@json_tree_view`]: JSON 風データ構造 [`json_tree_view::JsonValue`]
//!   （外部依存ゼロの自前 enum）をツリー表示する（イシュー #829、[`mod@tree_view`]
//!   （#753）の派生）。構造部は [`mod@tree_view`] の既存パーツ関数・
//!   [`tree_view::TreeView`] 状態機械をそのまま再利用し、JSON 固有の `key`/
//!   `value`（`data-scope="json-tree-view"`）の 2 パーツのみを追加する。
//!   ノード識別子は RFC 6901 JSON Pointer で決定的に導出する
//!   （[`json_tree_view::render_json`] モジュール doc 参照）。
//! - [`mod@scroll_area`]: Root / Viewport / Content / Scrollbar / Thumb /
//!   Corner の 6 anatomy パーツ（イシュー #825、`docs/design/component-coverage-map.md`
//!   保留解除）。[`mod@breadcrumb`]/[`mod@nav_list`] と同じく状態機械を持たない
//!   自由関数のみで構成する。`viewport` は WAI 慣行に従い `tabindex="0"` を
//!   固定付与し、`scrollbar`/`corner` はネイティブスクロールバーと意味が
//!   重複する装飾要素のため `aria-hidden="true"` を固定付与する。JS による
//!   スクロール位置追従・thumb drag は本イシューのスコープ外（モジュール doc
//!   参照）。
//! - [`mod@color`]: RGB / HSL / HSV / HEX の相互変換を提供する外部依存ゼロ・
//!   整数演算のみの純粋関数モジュール（イシュー #838、`docs/design/
//!   component-coverage-map.md` の ColorSwatch 保留解除）。[`qr_encode`]
//!   （#774）と同型の「標準ライブラリのみで完結する決定的アルゴリズム
//!   モジュール」であり、ブラウザ API 依存がなく wasm 境界隔離の対象外
//!   （純粋計算のみ）。`fandhe-frontend-pre-styled-ui::color_swatch`
//!   （ColorSwatch、#838）と後続の ColorPicker（#837 配下）が本モジュールの
//!   型・変換関数を土台にする。anatomy を持たない（headless 列は
//!   coverage-map 上も「—」）。
//! - [`mod@date`]: 決定的な暦計算コア（proleptic Gregorian・date-only、
//!   イシュー #833、親トラッキング #832）。[`date::PlainDate`]（年月日）・
//!   [`date::Weekday`]・[`date::month_grid`] を提供し、現在時刻を一切取得
//!   しない（「今日」は常に呼び出し側が明示的に渡す）。date-time 系 4 部品
//!   （Calendar / DatePicker / DateInput / Timer、#834 以降）が描画前の
//!   暦計算に共通で使う先行前提であり、本モジュール自体は非描画の純計算
//!   モジュールで anatomy・状態機械を持たない。
//! - [`mod@color_picker`]: HSV + アルファ + [`state::Disclosure`] を埋め込んだ
//!   ColorPicker（イシュー #839、親 #837、`docs/design/component-coverage-map.md`
//!   保留解除）。Root / Label / Control / Trigger / Positioner / Content /
//!   Area / AreaBackground / AreaThumb / ChannelSlider(+Track/+Thumb) /
//!   ChannelInput / ValueText / HiddenInput の各 anatomy パーツを提供する。
//!   [`mod@color`] の型・変換関数（外部依存ゼロ・整数演算のみ）のみを土台に
//!   し、色領域・色相/アルファスライダーの見た目は CSS グラデーション +
//!   thumb 位置（本モジュールの導出 getter が算出する割合）で表現する
//!   canvas 非依存の設計（`docs/policy/intentional-non-adoption.md` §7
//!   再評価トリガー充足）。
//! - [`mod@date_input`]: Root / Label / Control / SegmentGroup / Segment /
//!   HiddenInput の 6 anatomy パーツと、年/月/日セグメント + フォーカス位置を
//!   持つ [`date_input::DateInput`] 状態機械（イシュー #834、
//!   `docs/policy/intentional-non-adoption.md` §7・
//!   `docs/design/component-coverage-map.md` の「保留」を DateInput 分のみ
//!   解除）。[`mod@date`] の [`date::PlainDate::new`]/[`date::PlainDate::parse_iso`]/
//!   [`date::days_in_month`] を利用し、3 セグメント充足時のみ実在日付として
//!   検証する fail-closed 契約（[`date_input::DateInput::value`]）を持つ。
//!   `date_input::segment_group` は [`mod@segment_group`]（segmented control）
//!   とは無関係の別 anatomy スコープ（[`mod@date_input`] モジュール doc
//!   参照）。granularity（時分秒）・range 選択・locale 依存整形・キーボード
//!   操作の DOM 配線は本イシューのスコープ外（[`date_input`] モジュール doc
//!   §スコープ外参照）。
//! - [`mod@timer`]: Root / Area / Item / ItemValue / ItemLabel / Separator /
//!   Control / ActionTrigger の 8 anatomy パーツと、idle/running/paused/
//!   completed の 4 値状態機械 [`timer::Timer`]（イシュー #836、
//!   `docs/design/component-coverage-map.md` 保留解除）。tick（経過ミリ秒）を
//!   外部から明示的に注入する決定的状態機械であり、`std::time`/`Instant`
//!   等の時計 API に一切依存しない（[`mod@timer`] モジュール doc 参照）。
//!   実 tick 駆動（`setInterval`）は `fandhe-frontend-wasm-full` の
//!   `headless_timer` モジュールの責務。
//! - [`mod@format`]: byte / number / time / relative-time の Format 系
//!   ユーティリティ（イシュー #853、親 Phase 5 #852）。ark-ui `format-byte`/
//!   `format-number`/`format-time`/`format-relative-time` 相当を、JS の
//!   `Intl` API に依存せず外部依存ゼロの決定的純関数として実装する
//!   （`docs/policy/intentional-non-adoption.md` §3.23 の非採用判断を
//!   「headless-ui 内モジュール化」で解消）。ノードを返さない `String` 純
//!   関数であり anatomy を持たない（coverage-map 上も「—」）。現在時刻 API
//!   を一切呼ばず、[`format::format_relative_time`] の基準時刻は呼び出し側
//!   が明示的に注入する（[`mod@timer`]/[`mod@date`] と同型の「時刻を渡さ
//!   れる」設計）。ロケールは [`format::Locale`]（en/ja、イシュー #854）を
//!   各 `Format*Options::locale` フィールド経由で呼び出し側が明示的に渡す
//!   値型として実装し、`LocaleProvider` の Context/Provider 機構・グローバ
//!   ル既定ロケールは持たない。
//! - [`mod@toolbar`]: Root / Button / Link / Separator / ToggleGroup /
//!   ToggleItem の 6 anatomy パーツと、roving tabindex（`focused`/
//!   `item_count`/`loop_focus`/`orientation`）の状態機械 [`toolbar::Toolbar`]
//!   （イシュー #991、`docs/design/component-coverage-map.md` 保留解除、
//!   Radix Primitives Toolbar 相当）。押下状態の管理は独自実装せず
//!   [`toggle_group::ToggleGroup`]/[`toggle_group::MultiToggleGroup`] を
//!   [`mod@toolbar`] から再エクスポートして再利用する（[`mod@toolbar`]
//!   モジュール doc 参照）。矢印キーの実 DOM 配線は
//!   `fandhe-frontend-wasm-full` の後続責務。
//! - [`mod@menubar`]: Root / Menu / Trigger / Positioner / Content / Item /
//!   ItemGroup / ItemGroupLabel / Separator / SubTrigger / SubContent の 11
//!   anatomy パーツと、roving tabindex + 単一開閉（`focused`/
//!   `trigger_count`/`open`/`loop_focus`/`orientation`）の状態機械
//!   [`menubar::Menubar`]（イシュー #992、`docs/design/component-coverage-map.md`
//!   保留解除、Radix Primitives Menubar 相当）。複数 [`mod@menu`] を水平
//!   （または垂直）に並べ、開いている Menu を跨いだ左右移動を提供する
//!   （[`mod@menubar`] モジュール doc「開いている Menu を跨いだ左右移動」
//!   参照）。[`mod@menu`] の anatomy はそのまま再利用せず、状態機械・値
//!   語彙のみを再利用する（[`mod@menubar`] モジュール doc「`menu` mod
//!   再利用の内訳」参照）。矢印キーの実 DOM 配線は
//!   `fandhe-frontend-wasm-full` の後続責務。
//! - [`mod@navigation_menu`]: Root / List / Item / Trigger / Content / Link
//!   の 6 anatomy パーツと、[`crate::state::SingleSelect`] を埋め込んだ
//!   「高々 1 個の Trigger だけが開く」状態機械
//!   [`navigation_menu::NavigationMenu`]（イシュー #993、
//!   `docs/design/component-coverage-map.md` 実装対象、Radix Primitives
//!   Navigation Menu 相当）。Radix が primitives 層に持ち込んでいる
//!   viewport 寸法測定・`data-motion`（アニメーション方向の露出）は
//!   `docs/policy/intentional-non-adoption.md` §3.25 規則 2（層の割り当て）
//!   により本クレートへは持ち込まない（[`mod@navigation_menu`] モジュール
//!   doc参照）。[`mod@nav_list`]（状態機械を持たない静的リンク集）とは
//!   ディスクロージャの有無で使い分ける（[`mod@navigation_menu`] モジュール
//!   doc「`nav_list` との使い分け」参照）。

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod accordion;
pub mod action_bar;
pub mod anatomy;
pub mod angle_slider;
pub mod aria;
pub mod avatar;
pub mod breadcrumb;
pub mod calendar;
pub mod carousel;
pub mod checkbox;
pub mod checkbox_group;
pub mod clipboard;
pub mod collapsible;
pub mod color;
pub mod color_picker;
pub mod combobox;
pub mod data_attrs;
pub mod date;
pub mod date_input;
pub mod date_picker;
pub mod dialog;
pub mod download_trigger;
pub mod drawer;
pub mod editable;
pub mod field;
pub mod fieldset;
pub mod file_upload;
pub mod floating_panel;
pub mod format;
pub mod hover_card;
// イシュー #1610: 参照実装（ark-ui/zag.js）との突合で `ImageCropperProps`
// （`data-disabled`/`data-dragging`）・キーボード操作の受け口が selection
// （`role="slider"` の 2D slider 意味論）へ移った変更・`action_for_key`
// （キー → アクションの純粋関数）を追加した。詳細は `image_cropper`
// モジュール doc「参照突合」節参照。
pub mod image_cropper;
pub mod json_tree_view;
pub mod link;
pub mod link_overlay;
pub mod listbox;
pub mod menu;
pub mod menubar;
pub mod nav_list;
pub mod navigation_menu;
pub mod number_input;
pub mod pagination;
pub mod password_input;
pub mod pin_input;
pub mod popover;
pub mod positioning;
pub mod progress;
pub mod qr_code;
mod qr_encode;
pub mod radio_group;
pub mod rating_group;
pub mod scroll_area;
pub mod segment_group;
pub mod select;
pub mod signature_pad;
pub mod skip_nav;
pub mod slider;
pub mod splitter;
pub mod state;
pub mod steps;
pub mod switch;
pub mod tabs;
pub mod tags_input;
pub mod timer;
pub mod toast;
pub mod toggle;
pub mod toggle_group;
pub mod toggle_tip;
pub mod toolbar;
pub mod tooltip;
pub mod tour;
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

pub use action_bar::ActionBar;
pub use anatomy::{anatomy, Anatomy};
pub use angle_slider::{AngleSlider, AngleSliderAction, AngleSliderProps};
pub use aria::{
    aria_activedescendant, aria_atomic, aria_autocomplete, aria_checked, aria_controls,
    aria_current, aria_describedby, aria_disabled, aria_expanded, aria_haspopup, aria_hidden,
    aria_invalid, aria_label, aria_labelledby, aria_live, aria_modal, aria_orientation,
    aria_pressed, aria_roledescription, aria_selected, role, AriaAutocomplete, AriaChecked,
    AriaCurrent, AriaLive, AriaPopup,
};
pub use avatar::{Avatar, AvatarAction, ImageStatus};
pub use breadcrumb::{breadcrumb, BreadcrumbItem};
pub use calendar::{Calendar, CalendarAction};
pub use carousel::{Carousel, CarouselAction};
pub use checkbox::{Checkbox, CheckboxFlags};
pub use checkbox_group::{CheckboxGroup, CheckboxGroupProps};
pub use clipboard::{Clipboard, ClipboardAction};
pub use color::{Color, ColorError, Hsl, Hsv, Rgb};
pub use combobox::{Combobox, ComboboxAction};
pub use data_attrs::{
    data_checked, data_complete, data_copied, data_current, data_disabled, data_highlighted,
    data_incomplete, data_invalid, data_orientation, data_pressed, data_readonly, data_required,
    data_state, Orientation,
};
pub use date_input::{DateInput, DateInputAction, DateSegment, DateSegmentFlags};
pub use date_picker::{DatePicker, DatePickerAction};
pub use dialog::Dialog;
pub use drawer::{Drawer, DrawerPlacement};
pub use editable::{Editable, EditableAction};
pub use field::{FieldIds, FieldProps};
pub use fieldset::FieldsetProps;
pub use floating_panel::{FloatingPanel, FloatingPanelAction, Stage};
pub use format::{
    format_byte, format_number, format_relative_time, format_time, ByteUnit, FormatByteOptions,
    FormatNumberOptions, FormatRelativeTimeOptions, FormatTimeOptions, Locale, NumberStyle,
    SignDisplay, UnitDisplay, UnitSystem,
};
pub use hover_card::{HoverCard, HoverCardDelays};
pub use menu::{Menu, MenuCheckboxItem, MenuRadioItemGroup};
pub use menubar::{Menubar, MenubarAction};
pub use navigation_menu::NavigationMenu;
pub use number_input::{NumberInput, NumberInputAction, NumberInputFlags};
pub use pagination::{ItemMode, PageEntry, Pagination, PaginationAction};
pub use password_input::{
    PasswordAutocomplete, PasswordInput, PasswordInputAction, PasswordInputProps,
};
pub use pin_input::{PinInput, PinInputAction, PinInputKind, PinInputProps};
pub use positioning::{
    compute_position, css_vars_style, data_align, data_side, placement_attrs, Align, ArrowPosition,
    Placement, PositioningConfig, Rect, ResolvedPosition, Side, Size,
};
pub use progress::{Progress, ProgressAction};
pub use qr_code::{ErrorCorrectionLevel, QrEncodeError, QrMatrix};
pub use radio_group::RadioGroup;
pub use rating_group::{RatingGroup, RatingGroupAction, RatingGroupProps, RatingItemFlags};
pub use segment_group::SegmentGroup;
pub use signature_pad::{Point, SignaturePad, SignaturePadAction, Stroke, StrokeError};
pub use slider::{Slider, SliderAction};
pub use splitter::{PanelSpec, Splitter, SplitterAction};
pub use state::{
    Checkable, CheckableAction, Disclosure, DisclosureAction, MultiSelect, MultiSelectAction,
    OpenState, SingleSelect, SingleSelectAction, TextInput, TextInputAction, DATA_STATE_CHECKED,
    DATA_STATE_CLOSED, DATA_STATE_OFF, DATA_STATE_ON, DATA_STATE_OPEN, DATA_STATE_UNCHECKED,
};
pub use steps::{Steps, StepsAction};
pub use switch::{Switch, SwitchAction};
pub use tabs::{tabs, ActivationMode, TabItem, TabsProps};
pub use tags_input::{TagsInput, TagsInputAction};
pub use toast::{ToastAction, ToastEntry, ToastPlacement, ToastStatus, Toaster};
pub use toggle::{Toggle, ToggleAction};
pub use toggle_group::{MultiToggleGroup, ToggleGroup};
pub use toggle_tip::ToggleTip;
pub use toolbar::{Toolbar, ToolbarAction};
pub use tooltip::Tooltip;
pub use tour::{ContentIds as TourContentIds, Tour, TourAction, TourStatus, TourStep};
pub use tree_view::{TreeNode, TreeView, TreeViewAction};
