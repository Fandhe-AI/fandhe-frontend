//! `fandhe-frontend-pre-styled-ui`: pre-styled UI コンポーネント層（外部依存は
//! `fandhe-frontend-headless-ui` のみ）。
//!
//! chakra-ui 相当の pre-styled（既定スタイル付き）UI コンポーネント層を提供する。
//! `fandhe-frontend-headless-ui`（anatomy・`data-*`・WAI-ARIA、イシュー #522）の上に
//! テーマトークン・variant API・静的 CSS 生成を重ね、styled 部品を実装する 2 層構造の
//! 上層を担う（親トラッキング #520、Phase 3 親 #545）。
//!
//! # 本クレートの不変条件（REQ-1・REQ-2・REQ-5、`.claude/rules/coding-rust.md`）
//!
//! 1. コンポーネントは [`fandhe_frontend_headless_ui`] 経由で
//!    `fandhe_frontend_core::Node` を返す通常の Rust 関数として実装する
//!    （REQ-5、マクロ DSL は採用しない）。
//! 2. 出力は `fandhe_frontend_core::render` の既定エスケープを必ず経由する。
//!    **`raw_html()` の使用は [`stylesheet::StyleSheet::style_element`] 内の
//!    レビュー済み 1 箇所（`#[expect(clippy::disallowed_methods, reason =
//!    "ESCAPE-REVIEWED: ...")]` 付き）に限定する**（イシュー #605）。
//!    [`StyleSheet`] は検証済み CSS のみを保持する型であり、任意文字列からの
//!    直接構築経路を公開しないため、新たなエスケープ迂回経路を作らない
//!    （詳細は [`stylesheet`] モジュール doc 参照）。
//! 3. **`unsafe` コード禁止**: `#![forbid(unsafe_code)]` によりクレート全体で
//!    機械的に禁止する（`crates/core/tests/unsafe_boundary.rs` が workspace
//!    member を自動発見して強制する）。
//! 4. **外部依存は `fandhe-frontend-headless-ui`（path）のみ**:
//!    `pre-styled-ui/Cargo.toml` の `[dependencies]` にサードパーティクレートを
//!    追加しない。`fandhe-frontend-core` への直接依存は headless-ui 経由で
//!    間接的に得る（dev-dependency としてのみ利用、後述）。styled 部品の
//!    `Node` 型参照は `fandhe_frontend_headless_ui::fandhe_frontend_core::Node`
//!    （headless-ui が再エクスポートする core、イシュー #550）経由で得る。
//!
//! # 実装済み API（イシュー #546/#547/#548/#550/#551/#606）
//!
//! - [`theme`]（#547/#606）: テーマトークン・ダークモード基盤。#606 で
//!   角丸（`radii`）・影（`shadows`）トークングループを追加した。
//! - [`css`]（#548）: CSS 宣言の低レベル表現・検証・シリアライズ。
//! - [`recipe`]（#548/#606/#604）: slot recipe 本体・[`recipe::SlotRecipe`]・
//!   [`recipe::VariantValue`]。#606 で標準 `colorPalette` 軸
//!   （[`recipe::ColorPalette`]）を追加した。compoundVariants 相当（複数
//!   variant 軸の組み合わせ条件スタイル）は
//!   [`recipe::SlotRecipe::compound_variant`]・[`recipe::VariantCondition`]・
//!   [`recipe::when`]（イシュー #604）。
//! - 状態機械を要しない単純 styled 部品 5 種（#550、#606 で colorPalette 軸・
//!   radii/shadow トークン参照へ配線）:
//!   - [`mod@button`]: [`button::button`]（単一 recipe、`<button type="button">`。
//!     `loading` 時は [`mod@spinner`] を子ノード先頭へ埋め込む。`palette`
//!     variant で色を切り替える）。
//!   - [`mod@badge`]: [`badge::badge`]（単一 recipe、`<span>`。`palette` variant
//!     を持つ）。
//!   - [`mod@spinner`]: [`spinner::spinner`]（単一 recipe、
//!     `<span role="status">`。`palette` variant を持つ）。
//!   - [`mod@alert`]: [`alert::root`] ほかパーツ関数群（slot recipe、
//!     root/indicator/content/title/description の 5 パーツ、`role="alert"`。
//!     公開 API は [`alert::AlertStatus`] のまま、内部で `status` を
//!     `--fandhe-palette-*` へ束ねる）。
//!   - [`mod@card`]: [`card::root`] ほかパーツ関数群（slot recipe、
//!     root/header/body/footer/title/description の 6 パーツ、装飾的コンテナ、
//!     role 付与なし。中立コンテナのため colorPalette 軸は付与しない）。
//!   - [`mod@skeleton`]（#764）: [`skeleton::skeleton`]（単一 recipe、
//!     `<div>`。ローディングプレースホルダー。`text`/`circle`/`rect` の
//!     `variant`、常時 `aria-hidden="true"`、`prefers-reduced-motion: reduce`
//!     でのアニメーション停止 CSS を持つ。装飾的占位要素のため card と同じ
//!     判断で colorPalette 軸を付与しない）。
//!   - [`mod@separator`]（#772）: [`separator::separator`]（単一 recipe、
//!     `<hr>`。区切り線であり中立的な罫線のため colorPalette 軸を付与しない。
//!     `orientation`（horizontal/vertical）・`variant`（solid/dashed）の
//!     2 軸を持ち、`role="separator"`・`aria-orientation`・
//!     `data-orientation` を常時出力する）。
//!
//!   いずれも variant/size/status は Rust enum（[`recipe::VariantValue`] 実装）
//!   として型安全に表現し、クラス名文字列を動的合成しない
//!   （[`recipe::SlotRecipe::variant_classes`] が決定的に生成する）。
//!   呼び出し側 `attrs` に含まれる `class` は `class_attr`（内部専用モジュール）
//!   が除去してから recipe 生成クラスと合成し、`class` 属性が常に単一になる
//!   ことを保証する。
//!
//! - headless 状態機械を持つ複合部品 5 種の styled ラッパー第 1 弾（#551）:
//!   [`mod@dialog`] / [`mod@tabs`] / [`mod@accordion`] / [`mod@menu`] /
//!   [`mod@select`]。examples・利用ガイド（#552）は別イシューのスコープ。
//! - headless 状態機械を持つ複合部品 2 種の styled ラッパー第 2 弾（#664）:
//!   [`mod@popover`] / [`mod@tooltip`]。設計方針・スコープ外は第 1 弾と同じ。
//! - headless 状態機械を持つ複合部品 1 種の styled ラッパー第 3 弾（#682）:
//!   [`mod@switch`]。`data-state` 語彙が `"checked"/"unchecked"`
//!   （open/closed ではない）である点、`hidden-input` の視覚的非表示化に
//!   [`crate::select`] の `hidden-select` と同じ visually-hidden パターンを
//!   再利用する点は [`mod@switch`] rustdoc 参照。
//! - headless 状態機械を持つ複合部品の styled ラッパー第 4 弾（#683）:
//!   [`mod@radio_group`]。`item-hidden-input` の visually-hidden 化は
//!   [`mod@select`] の `hidden-select` と同じ責務分担、フォーカスリングは
//!   新設の [`recipe::StateCondition::FocusWithin`] を使う（モジュール
//!   rustdoc 参照）。
//! - headless ラッパー（#684）: [`mod@avatar`]（Avatar、`size`/`shape` の
//!   2 軸 variant を持つ最初のラッパー）。
//! - [`mod@switch`]/[`mod@radio_group`] への `size`/`color-palette` variant
//!   拡張（#708）: 下記「複合部品の variant 統一方針」節を参照。
//! - [`mod@tabs`]/[`mod@accordion`]/[`mod@dialog`]/[`mod@menu`]/
//!   [`mod@select`] への `size` variant 拡張（tabs のみ `color-palette` も、
//!   イシュー #729）: 下記「複合部品の variant 統一方針」節を参照。
//! - headless ラッパー（#738）: [`mod@number_input`]（NumberInput、`size`
//!   variant のみを持つ。フォーム入力部品のため `color-palette` 軸は提供
//!   しない。詳細は [`mod@number_input`] rustdoc 参照）。
//! - headless ラッパー（#739）: [`mod@pin_input`]（PinInput、`size` variant
//!   のみを持つ。[`mod@number_input`] と同型の判断で `color-palette` 軸は
//!   提供しない。詳細は [`mod@pin_input`] rustdoc 参照）。
//! - headless ラッパー（#743）: [`mod@segment_group`]（SegmentGroup、
//!   segmented control。`size` variant のみを持ち `color-palette` 軸は
//!   提供しない。状態機械は [`mod@radio_group`] へ全委譲。indicator の
//!   位置表現・visually-hidden 化・フォーカスリングの設計は
//!   [`mod@segment_group`] rustdoc 参照）。
//! - headless ラッパー（#744）: [`mod@tags_input`]（TagsInput、`size`
//!   variant のみを持つ。[`mod@pin_input`]/[`mod@number_input`] と同型の
//!   判断で `color-palette` 軸は提供しない。詳細は [`mod@tags_input`] rustdoc
//!   参照）。
//! - headless 状態機械を持つ複合部品の styled ラッパー第 5 弾（#730）:
//!   [`mod@checkbox`]。`size`/`color-palette` variant・`data-focus-visible`
//!   フォーカスリングは [`mod@switch`] と同型で最初から実装する。`indicator`
//!   の `hidden` 属性意味論を CSS が壊さない設計（`display` 宣言を置かない）
//!   は [`mod@checkbox`] rustdoc 参照。
//! - 状態機械を持たない静的フォーム部品 3 種（#737）:
//!   [`mod@input`] / [`mod@textarea`] / [`mod@native_select`]。ブラウザ
//!   ネイティブ挙動をそのまま尊重し、アクセシビリティ配線（`id`・ネイティブ
//!   `disabled`/`required`/`readonly`・`aria-invalid`・`aria-describedby`・
//!   `data-*`）は `fandhe_frontend_headless_ui::field`（#538/#602）へ全面委譲
//!   する。`variant`（`Outline`/`Subtle`/`Flushed`、NativeSelect のみ
//!   `Flushed` の代わりに `Plain`）と `size` の 2 軸を持つが、`color-palette`
//!   軸は提供しない（「複合部品の variant 統一方針」§3 参照。フォーム入力は
//!   選択・チェック状態を示す部品ではないため）。recipe scope は独自の scope
//!   を新設せず `"field"` を共有する設計判断は [`mod@input`] rustdoc 参照。
//! - headless 状態機械を持つ複合部品の styled ラッパー第 6 弾（#742）:
//!   [`mod@rating_group`]。星形 indicator は SVG/icon font/画像 URL を一切
//!   参照しない `clip-path` によるインライン表現（外部リソース非参照）。
//!   `size`（Sm/Md/Lg、星の寸法）/`color-palette`（点灯時の塗り色）の 2 軸
//!   variant を最初から持つ。詳細は [`mod@rating_group`] rustdoc 参照。
//! - headless 状態機械を持つ複合部品 2 種の styled ラッパー（イシュー #746）:
//!   [`mod@toggle`] / [`mod@toggle_group`]。実フォーカスをネイティブ
//!   `<button>` 自身が受けるため（Switch/RadioGroup の hidden-input パターン
//!   非該当）、フォーカスリングは `data-focus-visible` 配線ではなく
//!   [`recipe::StateCondition::FocusVisible`] で足りる。`size`/
//!   `color-palette` variant を最初から持つ（`toggle_group` は root のみへ
//!   クラスを付与する複合部品の統一方針に従う）。詳細は [`mod@toggle`]
//!   rustdoc 参照。
//! - headless ラッパー（#753）: [`mod@tree_view`]（TreeView、階層構造の展開・
//!   折りたたみ・選択）。ナビゲーション/コレクション表示部品であり
//!   [`mod@popover`]/[`mod@tooltip`] と同じ判断で `size`/`color-palette` の
//!   いずれの variant も提供しない（[`mod@tree_view`] rustdoc 参照）。branch
//!   のインデントは CSS custom property（`--fandhe-tree-view-indent`）で
//!   表現し、DOM ネストにより深さ分が自然に累積する。
//! - headless ラッパー（イシュー #755）: [`mod@breadcrumb`]（Breadcrumb、
//!   `docs/api/headless-ui-api.md` §4b の追加候補消化。状態機械を持たない
//!   静的意味論ナビ）。`size`/[`breadcrumb::BreadcrumbVariant`]（`link` の
//!   下線表示切り替え）の 2 軸 variant を root のみへ付与し、`link` への
//!   伝搬は root スコープ CSS custom property の継承で行う（[`mod@switch`]
//!   と同型のパターン、[`mod@breadcrumb`] rustdoc 参照）。
//! - headless ラッパー（イシュー #759）: [`mod@hover_card`]（HoverCard、
//!   リンク先プレビュー等 hover/focus で開閉するオーバーレイ）。構造上
//!   最も近い先行例は [`mod@tooltip`] であり、`content` の開閉連動・
//!   `--fandhe-reference-width` 非消費・focus-visible リングの各方針を
//!   継承する（[`mod@hover_card`] rustdoc 参照）。
//! - カード型選択 UI 2 種（#747）: [`mod@checkbox_card`]/[`mod@radio_card`]。
//!   chakra-ui の checkbox-card/radio-card 相当（ark-ui には対応する
//!   headless anatomy が存在しないため、headless-ui は変更せず pre-styled
//!   層で新規 anatomy `data-scope="checkbox-card"`/`"radio-card"` を定義する
//!   [`crate::card`] 型の構成）。状態機械は headless の
//!   [`fandhe_frontend_headless_ui::checkbox::Checkbox`]/
//!   [`fandhe_frontend_headless_ui::radio_group::RadioGroup`] をそのまま
//!   再利用し、新規状態機械は作らない。詳細は各モジュール rustdoc 参照。
//! - headless 状態機械を持つ複合部品の styled ラッパー（イシュー #754）:
//!   [`mod@carousel`]。`size` variant のみを持ち（`item-group` の縦横
//!   transform 切替は `data-orientation` 属性条件、[`mod@segment_group`] と
//!   同型）、`color-palette` 軸は提供しない（選択・チェック状態を示す部品
//!   ではないため）。`--fandhe-carousel-index` CSS カスタムプロパティによる
//!   決定的なスライド位置表現・autoplay スコープ外は [`mod@carousel`]
//!   rustdoc 参照。
//! - headless ラッパー（イシュー #758）: [`mod@drawer`]（Drawer、dialog の
//!   変種。WAI-ARIA 上は同じ Dialog パターンのため、開閉状態機械は
//!   [`mod@dialog`] を再利用する headless 層の設計をそのまま引き継ぎ、本
//!   モジュールも新規状態機械を持たない）。`size`（drawer の占有幅/高さ）
//!   variant のみを持ち `color-palette` 軸は提供しない（[`mod@number_input`]
//!   等と同型の判断）。placement（`start`/`end`/`top`/`bottom`）は variant
//!   ではなく headless 層が出力する `data-placement` に連動する CSS で表現
//!   する。詳細は [`mod@drawer`] rustdoc 参照。
//! - headless ラッパー（Progress circular 対応、イシュー #763）:
//!   [`mod@progress`]。headless の値状態機械
//!   [`fandhe_frontend_headless_ui::progress::Progress`] が既に持つ Circle/
//!   CircleTrack/CircleRange（SVG）の inherent メソッドへ CSS のみを追加提供
//!   する薄い委譲層で、新規状態機械は持たない。`size` variant のみを持ち
//!   `color-palette` 軸は提供しない（`Progress` 型はあえて再エクスポートせず、
//!   `size` variant クラス付与のため styled root のみを新設する。
//!   [`mod@dialog`]/[`mod@switch`] と同型の判断）。linear（Track/Range）用の
//!   styled ラッパーは対応表（`docs/design/component-coverage-map.md`）が
//!   本イシューと切り分けたスコープ外。詳細は [`mod@progress`] rustdoc 参照。
//! - headless ラッパー 3 種（イシュー #756、#716 追加候補・最優先候補の消化）:
//!   [`mod@link`]（Link、`variant` の下線表示切り替え + `aria-current="page"`
//!   状態装飾）、[`mod@link_overlay`]（LinkOverlay、`::before` 疑似要素の
//!   代わりに `overlay` 自身を `position: absolute; inset: 0;` で展開する
//!   カード全面クリック化。詳細は headless 層 rustdoc 参照）、
//!   [`mod@nav_list`]（NavList、`docs/design/docs-site-styled-ui-adoption.md`
//!   §3.1 が指摘した `menu` ロール誤転用を解消する文書ナビ専用部品。`role`
//!   を一切付与しない）。`fandhe-frontend-docs-site` は本クレートの styled
//!   `root`/`stylesheet` ではなく headless 再エクスポート
//!   （[`nav_list::heading`]/[`nav_list::list`]/[`nav_list::item`]/
//!   [`nav_list::link`]）のみを使い、`site/assets/site.css` の自己完結
//!   不変条件（§3.4）を維持したまま §3.1/§3.2 の意味論不整合を解消する
//!   （[`mod@nav_list`] rustdoc 参照）。
//! - 状態機械を要しない静的部品 2 種（イシュー #765）:
//!   [`mod@status`]（Status、root/indicator の 2 パーツ、`size`/
//!   `color-palette` の 2 軸 variant を持つ。ラベルテキスト自体が状態を
//!   伝えるため `role`/live region は付与しない）・[`mod@empty_state`]
//!   （EmptyState、root/content/indicator/title/description/actions の
//!   6 パーツ、[`crate::card`] と同型の中立コンテナで `color-palette` 軸は
//!   提供しない）。
//! - タイポグラフィ静的部品 6 種（イシュー #771）: [`mod@heading`]
//!   （[`heading::heading`]、`h1`〜`h6` のタグ選択 + `size` variant）・
//!   [`mod@text`]（[`text::text`]、`<p>`、`size` variant）・[`mod@em`]
//!   （[`em::em`]、`<em>`、variant なし）・[`mod@mark`]（[`mark::mark`]、
//!   `<mark>`、`variant`/`color-palette` の 2 軸）・[`mod@blockquote`]
//!   （[`blockquote::root`] ほかパーツ関数群、root/content/caption の 3
//!   パーツ、`variant`/`color-palette`）・[`mod@list`]（[`list::root`] ほか
//!   パーツ関数群、root/item/indicator の 3 パーツ、`ListType` によるタグ
//!   選択 + `variant`）。いずれも headless 状態機械を要しない静的部品
//!   （badge/skeleton と同型）。記事全体へのカスケードスタイル（chakra-ui の
//!   `Prose` 相当）は本クレートへ導入せず、`fandhe-frontend-docs-site` の
//!   `site/assets/site.css`（`.docs-content` 規則）が引き続き担う（役割分担
//!   の詳細は [`mod@text`] rustdoc 参照）。
//!
//! # headless ラッパーの設計（#551/#664/#682/#683/#729）
//!
//! [`mod@dialog`]・[`mod@accordion`]・[`mod@menu`]・[`mod@select`]・
//! [`mod@tabs`]・[`mod@popover`]・[`mod@tooltip`] はいずれも
//! `fandhe_frontend_headless_ui` の対応モジュールが出力する
//! `data-scope`/`data-part` 属性セレクタへ [`recipe::SlotRecipe`] で静的 CSS
//! を対応付ける薄い委譲層である。各モジュールの `stylesheet()` が生成する
//! CSS は静的 `.css` ファイルとして配信する、または
//! [`stylesheet::StyleSheet`]（#605）へ取り込んで `<style>` タグへインライン
//! 埋め込む、両方の利用形態を前提とする（不変条件 2 を参照）。新たな出力
//! 経路・エスケープ迂回は一切持たない。
//!
//! [`mod@popover`]・[`mod@tooltip`] はパーツ関数・状態機械を
//! headless 層からそのまま再エクスポートし（`pub use ...::*`）、variant
//! （size 等）ごとのクラス切り替えはスコープ外のままとする（提供しない方針、
//! 下記「複合部品の variant 統一方針」節 3 参照）。[`mod@switch`]・
//! [`mod@radio_group`]（#708）・[`mod@tabs`]/[`mod@accordion`]/
//! [`mod@dialog`]/[`mod@menu`]/[`mod@select`]（#729）は `size`（tabs のみ
//! `color-palette` も）variant を追加したため、[`crate::avatar`]・
//! [`crate::card`] と同型の選択的 re-export（薄い委譲層である点は変わらない）
//! へ移行済み（各モジュール rustdoc 参照）。
//!
//! # 複合部品の variant 統一方針（イシュー #708）
//!
//! 単純部品（button/badge/spinner）・avatar に続き、headless 状態機械を持つ
//! 複合部品ラッパーへ `size`/`color-palette` variant を拡張する際の統一方針:
//!
//! 1. **クラスは root slot のみに付与する**（[`crate::card::root`]・
//!    [`crate::avatar::root`] と同型）。子孫 slot（control/thumb/
//!    item-control 等）への寸法・色の伝搬は、root の variant 宣言が登録する
//!    **root スコープの CSS custom property**（`--fandhe-<scope>-*`
//!    名前空間）の通常の CSS 継承と、palette は既存の
//!    [`recipe::palette_declarations`]（`--fandhe-palette-*`、#606）で行う。
//!    [`recipe::SlotRecipe`] へ子孫セレクタ機構は追加しない（recipe 無改変で
//!    決定的生成を維持する）。
//! 2. **base 規則の `var()` には Md/Accent 相当のフォールバック値を書く**
//!    （例: `width: var(--fandhe-switch-track-width, 2.5rem)`）。styled root
//!    を経由しない headless 直接利用マークアップでも現行外観を維持する
//!    （fail-safe）。
//! 3. **軸の提供基準**: `size` は寸法スケールに意味がある部品（フォーム操作
//!    部品・トリガー系）へ、`color-palette` はアクセント色で選択・チェック
//!    状態を示す部品へ提供する。オーバーレイの配置・寸法がコンテンツ/
//!    positioning 起因の popover/tooltip には提供しない（方針として確定）。
//! 4. **styled root の API 形**: [`crate::avatar`] 前例に従い、variant 引数
//!    （`size`, `palette`）を先頭に置いた styled `root` 関数を各モジュールで
//!    再定義し、glob 再エクスポートから選択的再エクスポートへ切り替える。
//!    inherent `root()` を持つ状態機械型（[`fandhe_frontend_headless_ui::switch::Switch`]）は
//!    [`crate::avatar::AvatarShape`] 前例と同じ理由（未スタイル root の静かな
//!    適用漏れを防ぐ fail-closed）で再エクスポートしない。必要な呼び出し側
//!    は [`fandhe_frontend_headless_ui`]（クレートルート再エクスポート、
//!    #685 のエスケープハッチ）経由で到達できる。
//! 5. **実装範囲**: [`mod@switch`]・[`mod@radio_group`]（#708）に続き、
//!    [`mod@tabs`]・[`mod@accordion`]・[`mod@dialog`]・[`mod@menu`]・
//!    [`mod@select`] の 5 部品へ `size`（sm/md/lg）を展開した（イシュー
//!    #729、tabs のみ `color-palette`（5 値）も追加）。tabs は他 4 部品と
//!    異なり headless 側に root への attrs 注入点自体が存在しなかったため、
//!    追加的（非破壊）な
//!    [`fandhe_frontend_headless_ui::tabs::tabs_with_root_attrs`] を新設した
//!    （`crates/headless-ui/src/tabs.rs` rustdoc 参照）。popover/tooltip へは
//!    引き続き提供しない（方針 3 参照）。accordion/dialog/menu/select は
//!    `color-palette` 軸を持たない（variant 表の方針、`docs/api/pre-styled-ui-api.md`
//!    参照）。
//!
//! [`theme`] が生成する CSS・styled 部品各モジュールの `css()`/`stylesheet()` は
//! いずれも静的 `.css` ファイルとして配信する利用形態、または
//! [`stylesheet::StyleSheet`]（#605）へ取り込んでの `<style>` 要素埋め込みの
//! 両方の利用形態を前提とする。
//!
//! # CSS の書き出し・埋め込みヘルパ（#605）
//!
//! [`stylesheet::StyleSheet`] は [`recipe::SlotRecipe::css`]・[`theme::Theme::to_css`]
//! ・各 styled 部品の `css()`/`stylesheet()` が返す決定的 CSS 文字列を集約し、
//! (a) [`stylesheet::StyleSheet::write_css_file`] による静的 `.css` ファイル
//! 書き出し（SSG・ビルドスクリプト向け）と、(b)
//! [`stylesheet::StyleSheet::style_element`] による SSR 用 `<style>` 要素
//! 埋め込みの 2 経路を提供する。検証済み CSS のみを保持する型で `raw_html()`
//! を内部に閉じ込め、呼び出し側へエスケープ迂回経路を公開しない（不変条件 2 の
//! 唯一の例外、詳細は [`stylesheet`] モジュール doc 参照）。
//!
//! # headless 型の再エクスポート契約（イシュー #685）
//!
//! [`mod@dialog`]・[`mod@accordion`]・[`mod@menu`]・[`mod@select`]・
//! [`mod@tabs`]・[`mod@popover`]・[`mod@tooltip`] の各 `pub fn` シグネチャ・
//! `impl Component` の `Action` には、各モジュールの `pub use
//! fandhe_frontend_headless_ui::<mod>::*;` では到達しない
//! `fandhe_frontend_headless_ui::state`（[`OpenState`] 等の状態値・
//! `DisclosureAction`/`SingleSelectAction`/`MultiSelectAction`/
//! `CheckableAction` 等の dispatch action）・`data_attrs`（`tabs` の
//! [`Orientation`]）由来の型が露出する。これらは呼び出し側が
//! `fandhe-frontend-pre-styled-ui` のみに依存してラッパーを呼び出せることを
//! 保証するため、各モジュール内で明示 `pub use` により再エクスポートする
//! （棚卸し表は `docs/api/pre-styled-ui-api.md` 参照）。
//!
//! 加えて、`Node` を組み立てる `fandhe_frontend_core`（[`fandhe_frontend_core`]）
//! と headless 層自体（[`fandhe_frontend_headless_ui`]）をクレートルートから
//! 再エクスポートし、`fandhe_frontend_pre_styled_ui::fandhe_frontend_core::{el,
//! text, render, Node}` のような単独依存パスを完結させる（headless-ui が
//! core に対して行う #550 と同型のエスケープハッチ）。`raw_html()` への到達
//! パスがこの再エクスポートにより増えるが、`raw_html()` 自体は既存の明示的
//! オプトイン API であり新たな迂回経路ではない（REQ-1、`.claude/rules/security.md`
//! A03 参照）。頻用の状態値 [`OpenState`]・[`Orientation`] はルートからも
//! 再エクスポートし、docs-site 等の実利用パスと同型の import を可能にする。
//!
//! # interactive 層の再エクスポート契約と判断根拠（イシュー #712）
//!
//! 上記の headless-ui/core クレート再エクスポートに加え、
//! `fandhe_frontend_headless_ui::fandhe_frontend_interactive`（推移的に
//! [`fandhe_frontend_core`] と同格のクレートそのものの再エクスポート）を
//! ルートへ追加する。hydration/dispatch まで書く場合に必要な `Component`/
//! `Hydrate`/`dispatch`/`HydrateError`/`render_for_hydration`/
//! `HYDRATE_ATTR_PREFIX`/`codec` モジュール/`DirtyTracked` は、この
//! 再エクスポートにより `fandhe_frontend_pre_styled_ui::fandhe_frontend_interactive::{...}`
//! パスで全て到達可能になる（SSR に限らず hydration まで単独依存で完結する）。
//! ルート直下への個別型再エクスポート（`Component`/`dispatch` 等を
//! `fandhe_frontend_pre_styled_ui::` 直下へ置く案）は、`dispatch` のような
//! 汎用名が名前衝突・責務混濁を招くため見送った。`OpenState`/`Orientation`
//! （#685）はルートへ置く実利用パス（docs-site）が既にあったため例外的に
//! 採用したが、interactive 系項目には現時点で in-repo の実利用者がおらず、
//! 必要になれば非破壊的に追加できる。詳細な判断根拠・棄却案は
//! `docs/api/pre-styled-ui-api.md` §3b を参照。

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod accordion;
pub mod alert;
pub mod avatar;
pub mod badge;
pub mod blockquote;
pub mod breadcrumb;
pub mod button;
pub mod card;
pub mod carousel;
pub mod checkbox;
pub mod checkbox_card;
mod class_attr;
pub mod combobox;
pub mod css;
pub mod dialog;
pub mod drawer;
pub mod em;
pub mod empty_state;
pub mod heading;
pub mod hover_card;
pub mod input;
pub mod link;
pub mod link_overlay;
pub mod list;
pub mod mark;
pub mod menu;
pub mod native_select;
pub mod nav_list;
pub mod number_input;
pub mod pagination;
pub mod pin_input;
pub mod popover;
pub mod progress;
pub mod radio_card;
pub mod radio_group;
pub mod rating_group;
pub mod recipe;
pub mod segment_group;
pub mod select;
pub mod separator;
pub mod skeleton;
pub mod slider;
pub mod spinner;
pub mod status;
pub mod stylesheet;
pub mod switch;
pub mod tabs;
pub mod tags_input;
pub mod text;
pub mod textarea;
pub mod theme;
pub mod toggle;
pub mod toggle_group;
pub mod toggle_tip;
pub mod tooltip;
pub mod tree_view;

pub use alert::AlertStatus;
pub use badge::{badge, BadgeProps, BadgeVariant};
pub use blockquote::BlockquoteVariant;
pub use button::{button, ButtonProps, ButtonVariant};
pub use card::CardVariant;
pub use css::{decl, Declaration};
pub use em::em;
pub use empty_state::EmptyStateProps;
pub use heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
pub use input::{input, InputProps, InputVariant};
pub use list::{ListType, ListVariant};
pub use mark::{mark, MarkProps, MarkVariant};
pub use native_select::{native_select, NativeSelectProps, NativeSelectVariant};
pub use recipe::{when, ColorPalette, Size, SlotRecipe, VariantCondition, VariantValue};
pub use separator::{separator, SeparatorProps, SeparatorVariant};
pub use skeleton::{skeleton, SkeletonProps, SkeletonVariant};
pub use spinner::{spinner, SpinnerProps};
pub use status::StatusProps;
pub use stylesheet::{StyleSheet, StylesheetError};
pub use text::{text, TextProps, TextSize};
pub use textarea::{textarea, TextareaProps, TextareaVariant};

// `fandhe_frontend_headless_ui` クレートそのものの再エクスポート（イシュー #685）。
// headless-ui が core に対して行う #550 と同型のエスケープハッチであり、
// 各ラッパーモジュールの glob 再エクスポートでは到達しない headless API 全域
// （`positioning`/`aria` 等）への逃げ道を pre-styled-ui 経由でも確保する。
pub use fandhe_frontend_headless_ui;
// `Node` を組み立てる core API（`el`/`text`/`render` 等）への推移的再エクスポート。
// headless-ui 経由で得ることで pre-styled-ui の `Cargo.toml` に
// `fandhe-frontend-core` への直接依存を追加せずに単独依存パスを完結させる
// （不変条件 4 参照）。
pub use fandhe_frontend_headless_ui::fandhe_frontend_core;
// `fandhe_frontend_interactive` クレートそのものへの推移的再エクスポート
// （イシュー #712）。hydration/dispatch まで書く場合に必要な
// `Component`/`Hydrate`/`dispatch`/`HydrateError`/`render_for_hydration`/
// `HYDRATE_ATTR_PREFIX`/`hydration` モジュール相当（`codec` モジュール）/
// `DirtyTracked` は、これまで pre-styled-ui 経由で到達できず
// `fandhe-frontend-interactive` への直接依存を利用者に強いていた
// （`crates/pre-styled-ui/tests/headless_reexports.rs` の dev-dependency
// import が実例）。core 再エクスポート（#550）・headless-ui 経由の core
// 再エクスポート（本ファイル前段）と同型のクレート再エクスポートにすることで、
// 利用者側の `fandhe-frontend-interactive` バージョン指定が headless-ui/
// pre-styled-ui 内部の依存とズレて「別バージョンの `Component` を実装している」
// というトレイト不一致エラーを踏む余地を無くす。ルート直下への個別型
// 再エクスポート（`Component`/`dispatch` 等をルートへ置く）は、`dispatch`
// のような汎用名の名前衝突・責務混濁を避けるため見送り、実利用パスが
// 生まれた時点で非破壊的に追加する判断とした（イシュー #712、
// `docs/api/pre-styled-ui-api.md` §3b 参照）。
pub use fandhe_frontend_headless_ui::fandhe_frontend_interactive;
// ラッパー呼び出しに頻出する状態値をルートからも再エクスポートする
// （`docs-site` の実利用パス `fandhe_frontend_headless_ui::{OpenState,
// Orientation}` と同型の import を pre-styled-ui 単独依存で可能にする）。
pub use fandhe_frontend_headless_ui::{OpenState, Orientation};
