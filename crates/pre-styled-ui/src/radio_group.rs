//! styled RadioGroup（headless ラッパー、イシュー #683、`size`/`palette`
//! variant 拡張はイシュー #708、親 #520/#545）。
//!
//! `fandhe_frontend_headless_ui::radio_group`（イシュー #558/#536）の
//! Label / Item / ItemControl / ItemText / ItemHiddenInput 5 anatomy パーツと
//! [`fandhe_frontend_headless_ui::radio_group::RadioGroup`] 状態機械を
//! そのまま再エクスポートし、[`stylesheet`] で既定 CSS を追加提供する。薄い
//! 委譲の根拠・スコープ外事項は [`crate::select`] の rustdoc と同じ方針に
//! 従う。
//!
//! # 選択的 re-export（`pub use ...::*` を使わない理由、イシュー #708）
//!
//! 本モジュールは `size`/`palette` variant クラス付与のため styled `root`
//! （[`crate::avatar::root`]・[`crate::card::root`] と同型）を本モジュールで
//! 再定義する。headless 自由関数 `root` と名前衝突するため、
//! `pub use ...::*` ではなく必要な識別子（[`label`]/[`item`]/
//! [`item_control`]/[`item_text`]/[`item_hidden_input`]/[`RadioGroup`]）のみ
//! を選択的に再エクスポートする。
//!
//! [`RadioGroup`] 状態機械は inherent `root()` を持たない（item 系メソッド
//! のみ、`crates/headless-ui/src/radio_group.rs` 参照）ため、[`crate::avatar`]
//! の `Avatar` 非再エクスポートと異なり、そのまま再エクスポートしても未
//! スタイル root の静かな適用漏れは発生しない（型を経由して `size`/
//! `palette` 抜きの `root` を誤って呼んでしまう経路がない）。
//!
//! # item-hidden-input の視覚的非表示化（[`crate::select`] の hidden-select
//! と同じ責務分担）
//!
//! headless 層（`crates/headless-ui/src/radio_group.rs`）はネイティブ
//! `<input type="radio">` に `type`/`value`/`name`/`checked`/`disabled`/
//! `data-state` のみを設定し、視覚的な非表示化は行わない契約になっている。
//! styled 層である本モジュールが visually-hidden パターン（`position:
//! absolute` + 1px クリップ、[`crate::select`] の `hidden-select` 規則と
//! 同一の 9 宣言）で覆い隠し、`item-control` をカスタムラジオ円として描画
//! する。フォーム送信・キーボード操作・グループ内排他選択はネイティブ
//! semantics のまま維持される（headless 側モジュール doc 参照）。
//!
//! # data-state とスタイルの連動
//!
//! `item`/`item-control`（選択状態、`data-state="checked"`/`"unchecked"`）の
//! 見た目の切り替えを [`recipe`] へ登録する（[`crate::recipe::SlotRecipe::state`]）。
//! `root` の `data-orientation="horizontal"` でも配置切り替えを行う。
//!
//! # `:focus-within` によるフォーカスリング（イシュー #683）
//!
//! `item-hidden-input` を視覚的に隠すと、ネイティブのフォーカスリングも
//! 見えなくなる。実フォーカスは隠された `<input>` にあり、`item`
//! （`<label>`、input の祖先）へ `:focus-within` を当てるのが CSS 的に成立
//! する唯一の経路（[`crate::recipe::StateCondition`] は `Attr`/`AttrEq`/
//! `FocusVisible` のみで兄弟・子孫セレクタを持たなかったため、本イシューで
//! [`crate::recipe::StateCondition::FocusWithin`] を追加した）。
//!
//! # `data-focus-visible` によるキーボード専用フォーカスリング（イシュー #709）
//!
//! 上記 `:focus-within` は「input にフォーカスがある」ことのみを条件とし、
//! マウスクリックによるフォーカスでも発火する（chakra-ui/ark-ui が区別する
//! キーボード操作専用の `:focus-visible` 意味論とは異なる、包括的な
//! フォールバック）。これを補完するため、headless 層
//! （`fandhe_frontend_headless_ui::data_attrs::data_focus_visible`、
//! `crates/headless-ui/src/radio_group.rs` のフォーカスリング契約 doc
//! 参照）が出力し `fandhe-frontend-wasm-full` の focus 配線が `item`/
//! `item-control` へ付け外しする `data-focus-visible` を `item-control`
//! slot の状態規則として追加する。役割分担: `:focus-within`（`item`） =
//! wasm なしでも成立する no-JS フォールバック / `data-focus-visible`
//! （`item-control`） = wasm 配線時のみ有効なキーボード専用リング。両者は
//! 独立した条件として共存し、どちらか一方が成立すればリングが表示される。
//! イシュー #1494 で `item-control` のリング色を `FocusRingColor::Token`
//! から `FocusRingColor::Palette` へ変更し、`item` の `:focus-within`
//! リング（既に `Palette`）と統一した（`docs/design/
//! pre-styled-ui-focus-ring-and-size-conventions.md` §6 手順 2、
//! `crate::checkbox` の `control` は #1454 で同型の変更を先行適用済み）。
//!
//! # スタイル調整（イシュー #1494、root/item/item-control パートのみ）
//!
//! 親 #1493（chakra-ui / Radix Themes / Radix Primitives / ark-ui 基準への
//! 調整）のうち **root/item/item-control の状態表現とフォーカスリング**を
//! 担当する分割 1/2。分割 2/2（size/orientation バリアント・ラベル/説明の
//! 型階層、イシュー #1495）とはファイルを共有するため、以下は本イシューが
//! 確定した意図的差分である（`crate::checkbox` #1454/#1455 分割と同型の
//! 判断）:
//!
//! - **hover は `--fandhe-hover-bg` custom property 経由の間接参照で表現
//!   する**（`crate::recipe` の disabled/hover/transition 共通ビジュアル
//!   言語、イシュー #1425）。`item-control` base が [`hover_bg_muted`] で
//!   unchecked 時の面色を定義し、checked の `state` 規則が同名プロパティを
//!   [`hover_bg_solid_with_fallback`] で上書きする。hover の実適用は
//!   `item-control` へ 1 本（`StateCondition::Hover`）のみ登録する
//!   （`crate::checkbox` の `control` と同型）。
//! - **`data-readonly` は視覚化しない**: 参照 4 サイトのいずれも readonly
//!   状態に radio 固有の視覚差を付けないため、`data-invalid`（下記）とは
//!   異なり CSS 規則を追加しない。
//! - **`data-invalid` は headless `radio_group` が `RadioGroupProps.invalid`
//!   から出力する**（イシュー #1616 の ark-ui / Radix Primitives 参照突合で
//!   `item`/`item_control`/`item_text`/`root`/`label` へ反映するよう是正
//!   済み。styled 本モジュールの CSS 規則は headless の出力属性へそのまま
//!   反応する。Field（#538）との `aria-describedby` 連携は引き続き別途の
//!   追跡対象、`crate::checkbox`/`crate::radio_card` と同型の判断）。
//! - **`hover_bg_solid_with_fallback` は `crate::recipe` の既存共通ヘルパを
//!   そのまま再利用する**（イシュー #1741 で checkbox 系から共通化済みの
//!   ものを流用するのみで、本モジュール専用のローカル複製は作らない）。
//! - **`box-sizing: border-box`** を `item-control` base へ追加し、同じ
//!   size トークン系の `crate::checkbox` `control` と寸法解釈（border 込み
//!   か否か）を統一する。
//!
//! `root`/`item` の `gap`・`item` の `cursor`・size 用 custom property 群・
//! orientation 切り替えは 2/2（#1495）で実装済み（次節）。
//!
//! # スタイル調整（イシュー #1495、size/orientation バリアントとラベル・
//! 説明の型階層）
//!
//! 分割 2/2。1/2（イシュー #1494）が残した「root/item の gap・item の
//! cursor・size 用 custom property 群・orientation 切り替え」を担当する
//! （`crate::checkbox_group` #1460/#1461 分割・`crate::checkbox` #1735 と
//! 同型の判断を踏襲）:
//!
//! - **size バリアントの一括登録**: 5 段の `.variant(Size::*, "root", ...)`
//!   を [`crate::recipe::SlotRecipe::size_variants`]（#1424 の共通生成手段）
//!   へ置換し、既定 `md` の設定漏れを構造的に防ぐ。control 寸法は xs/sm の
//!   み 4px 格子（0.7rem/0.85rem → 0.75rem/0.875rem）へ是正し、md/lg/xl は
//!   既存外観を維持する（`crate::checkbox` #1735 と同一の段階・根拠）。
//!   `--fandhe-radio-group-dot-inset`（選択ドットの光学的比率値）・
//!   `--fandhe-radio-group-font-size` の段階は現状維持する（意図的な例外、
//!   `checkbox` の check-width/height と同じ扱い）。
//! - **gap の size 連動**: `root` の項目間余白を
//!   `--fandhe-radio-group-gap`、`item` 内の control ↔ text 余白を
//!   `--fandhe-radio-group-item-gap` として custom property 化する
//!   （`crate::checkbox_group` #1461 と同型の 2 種類の gap 分離）。md は
//!   フォールバック値と同値にし既定外観を変えない。
//! - **orientation（horizontal）の是正**: `flex-direction: row` のみだった
//!   規則へ `flex-wrap: wrap`（折り返しなしで溢れる現状の是正）・
//!   `column-gap: var(--fandhe-space-4)`（chakra `HStack gap=4` 相当、
//!   `row-gap` は `root` の `gap` を継承させ個別宣言しない）・
//!   `--fandhe-radio-group-label-basis: 100%` を追加する
//!   （`crate::checkbox_group` #1460 と同型）。`label` base へ
//!   `flex-basis: var(--fandhe-radio-group-label-basis, auto)` と
//!   `flex-shrink: 0` を追加し、横並び時のみ `label` が独立行として
//!   フルライン幅を占有するようにする（縦積み時は `auto` フォールバックで
//!   無影響）。
//! - **`label`（グループ見出し）/`item-text`（項目テキスト）の型階層**:
//!   `label` の `font-size` を固定 `var(--fandhe-font-font-size-sm)` から
//!   size 連動 `var(--fandhe-radio-group-font-size, ...)` へ変更し、
//!   `font-weight: medium`・`line-height: normal` を追加する。`item-text`
//!   は `line-height: normal` と `user-select: none`（`<label>` のクリックで
//!   選択がトグルするため誤選択防止、chakra label と同じ）を追加し、
//!   `font-weight` は宣言せず通常ウェイトを継承させることで `label`
//!   （medium）との 2 段階の型階層を作る（`crate::checkbox_group` #1461 と
//!   同型）。
//! - **item のクリック領域**: 縦積み時に `<label>` が行幅いっぱいへ伸びて
//!   クリック領域が余白まで広がる問題を是正するため、`item` base へ
//!   `width: fit-content` を追加する（Radix Themes 準拠、
//!   `crate::checkbox_group` と同型）。
//!
//! **本イシューで意図的に合わせなかった点**（`.claude/rules/code-comment-style.md`）:
//!
//! - **`description` パートは追加しない**: headless anatomy
//!   （`crates/headless-ui/src/radio_group.rs`）に存在せず、anatomy 構造は
//!   headless 層の責務。参照元 chakra-ui も専用パートを持たず利用者合成に
//!   委ねている。pre-styled-ui 単独での `data-part` 新設は anatomy
//!   ドリフト検知（`wrap_state.rs` 等）と公開 API 追加を伴うため見送る
//!   （`crate::checkbox` #1735 / `crate::checkbox_group` #1461 と同一判断）。
//!   説明文が必要な呼び出し側は `fg-muted` + 1 段小さいサイズで自前合成する。
//! - **`--fandhe-radio-group-dot-inset` は size 連動の比率値のまま維持**:
//!   選択ドットの太さは control 寸法に対する光学的比率であり、4px 格子の
//!   是正対象ではない（`checkbox` の check-width/height と同じ意図的例外）。
//! - **variant 軸（solid/subtle 等の面表現）は追加しない**: radio に面
//!   variant を持つ参照サイトはなく、`root()` シグネチャ変更は破壊的である。
//!
//! # `size`/`palette` variant（イシュー #708）
//!
//! `size`（[`Size`]）は `root` へのみクラスを付与し、[`recipe`] が登録する
//! `--fandhe-radio-group-control-size`/`-dot-inset`/`-font-size` の root
//! スコープ custom property（CSS の通常のプロパティ継承により `item`/
//! `item-control`/`item-text` へ伝わる。`root` はこれらのパーツを内包する
//! 祖先要素であるため、[`crate::recipe::SlotRecipe`] へ子孫セレクタ機構を
//! 追加せずに実現できる）経由で `item-control` の寸法・選択ドットの見た目を
//! 切り替える。`palette`（[`ColorPalette`]）は既存の
//! [`crate::recipe::palette_scale_declarations`]（chakra-ui virtual token 方式、
//! #606）を `root` へ登録し、checked 時の `item-control` の枠色・背景・
//! `:focus-within` のアウトライン色を `var(--fandhe-palette, ...)` 経由で
//! 切り替える。`base`/`state` 規則の `var()` にはいずれも Md サイズ・
//! Accent パレット相当のフォールバック値を書き、styled `root` を経由しない
//! headless 直接利用マークアップでも現行外観を維持する（fail-safe、
//! `crate::lib` rustdoc「複合部品の variant 統一方針」節参照）。
//!
//! # セキュリティ不変条件
//!
//! 本モジュールは headless 層の再エクスポートと静的 CSS 生成のみで構成され、
//! `raw_html()` を使用しない。CSS 宣言値はすべてコンパイル時静的リテラルで
//! あり、動的値（`value`/`name`/属性/children）へ CSS 値として流し込む経路
//! を持たない（動的値は headless 層経由で `fandhe_frontend_core::render` の
//! 既定エスケープを必ず通る、REQ-1）。styled `root` は [`drop_class_attr`]
//! により呼び出し側の `class` を除去してから合成するため、`class` 属性は
//! 常に単一（[`crate::avatar::root`] と同型）。
//!
//! # 本イシューのスコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - tabs/accordion/dialog/menu/select への size（および tabs への
//!   palette）展開は本イシューの方針を第 2 弾として別途適用する
//!   （`docs/api/pre-styled-ui-api.md` の variant 表参照）。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{
    disabled_declarations, focus_ring_declarations, hover_bg_muted, hover_bg_solid_with_fallback,
    hover_surface_declarations, palette_scale_declarations, transition_declarations, ColorPalette,
    FocusRingColor, FocusRingOffset, MotionDuration, Size, SlotRecipe, StateCondition,
    VariantValue,
};

// headless 自由関数 `root` はあえて再エクスポートしない（本モジュール冒頭
// の rustdoc「選択的 re-export」節参照、`root` は本モジュールで styled 版
// として再定義する）。
use fandhe_frontend_headless_ui::data_attrs::Orientation;
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
pub use fandhe_frontend_headless_ui::radio_group::{
    item, item_control, item_hidden_input, item_text, label, RadioGroup, RadioGroupProps,
    DATA_STATE_CHECKED, DATA_STATE_UNCHECKED,
};

/// headless `radio_group` anatomy の `data-part` 一覧（`crates/headless-ui/src/radio_group.rs`
/// の `ANATOMY.part(...)` 呼び出しと同期させる契約）。
const SLOTS: &[&str] = &[
    "root",
    "label",
    "item",
    "item-control",
    "item-text",
    "item-hidden-input",
];

/// この styled RadioGroup の既定 CSS を組み立てる（内部ヘルパ、[`stylesheet`]
/// のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    let mut recipe = SlotRecipe::new("radio-group", SLOTS)
        .base(
            "root",
            vec![
                decl("display", "flex"),
                decl("flex-direction", "column"),
                // 項目間余白の size 連動受け口（イシュー #1495）。
                // フォールバックは是正前の固定値（md 相当）を保ち、styled
                // `root` を経由しない headless 直接利用でも現行外観を
                // 維持する（fail-safe）。
                decl(
                    "gap",
                    "var(--fandhe-radio-group-gap, var(--fandhe-space-1))",
                ),
            ],
        )
        .base(
            "label",
            vec![
                decl("display", "block"),
                decl("color", "var(--fandhe-color-fg)"),
                // イシュー #1495: 固定 `font-size-sm` から size 連動へ変更し、
                // `font-weight`/`line-height` を追加して `item-text`
                // （通常ウェイト）との型階層を作る（`crate::checkbox_group`
                // #1461 と同型）。
                decl(
                    "font-size",
                    "var(--fandhe-radio-group-font-size, var(--fandhe-font-font-size-sm))",
                ),
                decl("font-weight", "var(--fandhe-font-font-weight-medium)"),
                decl("line-height", "var(--fandhe-font-line-height-normal)"),
                decl("margin-bottom", "var(--fandhe-space-1)"),
                // horizontal orientation 時のみ `label` を独立行にする
                // 間接参照（`root` の `data-orientation="horizontal"` 規則が
                // `--fandhe-radio-group-label-basis: 100%` を定義する。
                // 縦積み時は `auto` フォールバックで無影響。
                // `SlotRecipe` は子孫セレクタを持たないため custom property
                // 経由の伝播が唯一の手段、`crate::checkbox_group` と同型）。
                decl("flex-basis", "var(--fandhe-radio-group-label-basis, auto)"),
                decl("flex-shrink", "0"),
            ],
        )
        .base(
            "item",
            vec![
                decl("display", "flex"),
                decl("align-items", "center"),
                // control ↔ text 余白の size 連動受け口（イシュー #1495）。
                decl(
                    "gap",
                    "var(--fandhe-radio-group-item-gap, var(--fandhe-space-2))",
                ),
                decl("cursor", "pointer"),
                // イシュー #1495: 縦積み時に `<label>` が行幅いっぱいへ
                // 伸びてクリック領域が余白まで広がる問題の是正（Radix
                // Themes 準拠、`crate::checkbox_group` と同型）。
                decl("width", "fit-content"),
            ],
        )
        .base(
            "item-control",
            vec![
                decl("display", "inline-flex"),
                // checkbox `control`（イシュー #1454）と寸法解釈を統一する
                // ため `border-box` を明示する。追加前は content-box 解釈
                // となり、border 1px × 2 分だけ描画寸法が同じ size トークン
                // 値でも checkbox より大きく見えていた（意匠上の純是正、
                // イシュー #1494）。
                decl("box-sizing", "border-box"),
                decl("width", "var(--fandhe-radio-group-control-size, 1rem)"),
                decl("height", "var(--fandhe-radio-group-control-size, 1rem)"),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "50%"),
                decl("background", "var(--fandhe-color-bg)"),
                decl("flex-shrink", "0"),
                // unchecked 時の hover 面（`crate::checkbox` の `control` と
                // 同型の間接参照設計、`crate::recipe` 冒頭 doc「disabled /
                // hover / transition の共通ビジュアル言語」節参照）。checked
                // state 規則が同名カスタムプロパティを上書きし、hover
                // セレクタ側は下記の `hover_surface_declarations()` 1 本の
                // みで両方の面色に追従する。
                hover_bg_muted(),
            ],
        )
        // `base` は同一 slot への複数回登録が許され出力順で連結される
        // （`checkbox.rs` の transition 追加と同型のパターン）。
        .base(
            "item-control",
            transition_declarations("background, border-color", MotionDuration::Fast),
        )
        .base(
            "item-text",
            vec![
                decl("color", "var(--fandhe-color-fg)"),
                decl(
                    "font-size",
                    "var(--fandhe-radio-group-font-size, var(--fandhe-font-font-size-sm))",
                ),
                // イシュー #1495: `label`（medium）との 2 段階の型階層を
                // 作るため `font-weight` は宣言せず通常ウェイトを継承する。
                decl("line-height", "var(--fandhe-font-line-height-normal)"),
                // `item`（`<label>`）のクリックで選択がトグルするための
                // 誤選択防止（chakra label と同じ、`crate::checkbox_group`
                // と同型）。
                decl("user-select", "none"),
            ],
        )
        .base(
            "item-hidden-input",
            vec![
                decl("position", "absolute"),
                decl("width", "1px"),
                decl("height", "1px"),
                decl("padding", "0"),
                decl("margin", "-1px"),
                decl("overflow", "hidden"),
                decl("clip", "rect(0, 0, 0, 0)"),
                decl("white-space", "nowrap"),
                decl("border", "0"),
            ],
        )
        // `root` の `data-orientation="horizontal"`（headless 層が
        // `data_orientation` 経由で出力、`crates/headless-ui/src/radio_group.rs`
        // 参照）では縦積みではなく横並びへ切り替える。イシュー #1495 で
        // `flex-wrap: wrap`（折り返しなしで溢れる現状の是正）・
        // `column-gap: var(--fandhe-space-4)`（chakra `HStack gap=4` 相当。
        // `row-gap` は `root` の `gap` を継承させ個別宣言しない）・
        // `--fandhe-radio-group-label-basis: 100%`（`label` base の
        // `flex-basis` 間接参照先、`crate::checkbox_group` #1460 と同型）を
        // 追加した。
        .state(
            "root",
            StateCondition::AttrEq("data-orientation", "horizontal"),
            vec![
                decl("flex-direction", "row"),
                decl("flex-wrap", "wrap"),
                decl("column-gap", "var(--fandhe-space-4)"),
                decl("--fandhe-radio-group-label-basis", "100%"),
            ],
        )
        // 選択済み項目のカスタムラジオ円の見た目（アクセントカラーの外枠 +
        // 内側ドット。`box-shadow` の inset で描く。ドットの太さは
        // `--fandhe-radio-group-dot-inset` で size ごとに切り替える）。
        .state(
            "item-control",
            StateCondition::AttrEq("data-state", "checked"),
            vec![
                decl(
                    "border-color",
                    "var(--fandhe-palette, var(--fandhe-color-accent))",
                ),
                decl(
                    "background",
                    "var(--fandhe-palette, var(--fandhe-color-accent))",
                ),
                decl(
                    "box-shadow",
                    "inset 0 0 0 var(--fandhe-radio-group-dot-inset, 3px) var(--fandhe-color-bg)",
                ),
                // checked 面の hover は palette の emphasized 段へ
                // （`checkbox` の checked/indeterminate 規則と同型。hover
                // セレクタは `:hover:not([data-disabled])` で詳細度が本規則
                // より高いため、直値ではなく間接参照でなければ checked 面が
                // 中立色（unchecked base の `hover_bg_muted()`）へ落ちて
                // しまう。`hover_bg_solid_with_fallback` は styled root
                // （`palette_scale_declarations`）非経由の headless 直接
                // 利用時も `--fandhe-color-accent-emphasized` へフォール
                // バックする共通ヘルパ〔イシュー #1741 で checkbox 系から
                // `crate::recipe` へ共通化済み〕を再利用する。新規ローカル
                // 複製は作らない）。
                hover_bg_solid_with_fallback(),
            ],
        )
        // `data-invalid`（`crate::input`/`checkbox` と同型の視覚言語）を
        // `item-control` slot へ反映する。headless `radio_group`
        // （`crates/headless-ui/src/radio_group.rs`）はイシュー #1616 で
        // `RadioGroupProps.invalid` から `data-invalid` を出力するよう
        // 是正済みであり、本規則はその出力へそのまま反応する（Field #538
        // との `aria-describedby` 連携は別途の追跡対象、`checkbox`/
        // `radio-card` と同型の判断）。
        .state(
            "item-control",
            StateCondition::Attr("data-invalid"),
            vec![decl("border-color", "var(--fandhe-color-danger)")],
        )
        // hover の実適用は 1 本のみ（`--fandhe-hover-bg` の間接参照経由で
        // unchecked/checked いずれの面色にも追従する。`crate::checkbox` の
        // `control` と同型）。`item-control` は headless 層が
        // `data-disabled` を出力するため `hover_surface_declarations` が
        // 直列化する `:hover:not([data-disabled])` セレクタで disabled 時の
        // hover を自然に除外できる。
        .state(
            "item-control",
            StateCondition::Hover,
            hover_surface_declarations(),
        )
        // `data-disabled`（headless 層が `data_disabled` 経由で `item`/
        // `item-control`/`item-text`/`item-hidden-input` へ出力）時の
        // 操作不能な見た目。`crate::recipe::disabled_declarations`
        // （共通ビジュアル言語、宣言順は opacity → cursor）へ canonical 化
        // する（`checkbox` と同型。宣言内容は既存の ad-hoc 実装と同値）。
        .state(
            "item",
            StateCondition::Attr("data-disabled"),
            disabled_declarations(),
        )
        // グループ全体の一括 disabled（headless `root` は `disabled=true`
        // で `data-disabled` を出力済み、`root()` 関数 doc 参照）時の
        // `root` 側 CSS 規則は意図的に**登録しない**（PR #1769 codex-review
        // P1 再指摘への最終対応）。経緯: 当初 `cursor: not-allowed` のみの
        // `root[data-disabled]` 規則を追加したが、`item` base が独立に
        // `cursor: pointer` を宣言するため item 要素上では pointer が
        // 表示され続け、root 自身の隙間でしか disabled 表現にならず
        // 「各 item にも disabled=true を渡す契約下では冗長、渡さない
        // 誤用下では item 上は pointer のまま」という指摘を受けた
        // （pointer-events: none の不採用は既に撤回済み、直上の履歴参照）。
        // 対応案として custom property 経由で `item` の cursor へ root
        // disabled を CSS カスケードで伝播する案も検討したが、
        // `crate::checkbox_group` が同型の 3 種 custom property
        // （`-item-opacity`/`-item-cursor`/`-item-pointer-events`）伝播を
        // 一度実装した後に撤回した判断（同モジュール rustdoc「スタイル
        // 調整」節参照）と同じ理由で不採用とした: 呼び出し側が契約
        // （各 `item`/`item-control`/`item-text`/`item-hidden-input` へも
        // `disabled=true` を渡す）を満たさない構成では、ネイティブ
        // `<input type="radio">` は実際には操作可能（Tab+Space で値変更
        // 可能）なまま `cursor: not-allowed` だけが表示され、「CSS だけで
        // disabled の実効性を偽装する」ことになる（`checkbox_group` と
        // 同一原因）。したがって `root[data-disabled]` はどの CSS 宣言も
        // 出力しない。誤操作防止は CSS ではなく、各パートへ個別に
        // `disabled=true` を渡す運用契約（`root()`/`item()` の headless
        // doc 参照）が担い、`item` 側の `[data-disabled]` 規則（直上）が
        // 実際の opacity/cursor を担う。恒久対応（`checkbox_group` が
        // イシュー #1741 で実装した、headless 層が root/item の disabled
        // を OR 計算してネイティブ属性まで一貫伝播する型）は本モジュール
        // （headless `radio_group::RadioGroup` は現状この種の便利メソッドを
        // 持たない）には未実装であり、別途の追跡対象とする。
        // イシュー #683: visually-hidden 化した `item-hidden-input` へ実
        // フォーカスがあるときのフォーカスリングを、祖先 `item`
        // （モジュール rustdoc 参照）へ `:focus-within` で反映する。
        // イシュー #1424: フォーカスリング規約に従い canonical ヘルパへ
        // 移行（`docs/design/pre-styled-ui-focus-ring-and-size-conventions.md`
        // §3）。`palette` 軸を公開する部品のため
        // `FocusRingColor::Palette`（`var(--fandhe-palette, var(--fandhe-color-focus-ring))`）
        // を使う。
        .state(
            "item",
            StateCondition::FocusWithin,
            focus_ring_declarations(FocusRingColor::Palette, FocusRingOffset::Outside),
        )
        // イシュー #709: wasm 層が付け外しする `data-focus-visible` による
        // キーボード操作専用のフォーカスリング（`:focus-within` の no-JS
        // フォールバックとは独立に共存する。モジュール rustdoc 参照）。
        // イシュー #1424 では暫定的に `palette` 非連動の `Token` を使って
        // いたが、イシュー #1494 で `docs/design/
        // pre-styled-ui-focus-ring-and-size-conventions.md` §6 の規約
        // （`palette` 軸を公開する部品は `Palette` を使う）へ合わせ、
        // `item` の `:focus-within` リング（上記、既に `Palette`）と統一
        // した（`checkbox` の `control` は #1454 で同じ変更を先行適用済み）。
        .state(
            "item-control",
            StateCondition::Attr("data-focus-visible"),
            focus_ring_declarations(FocusRingColor::Palette, FocusRingOffset::Outside),
        )
        // `data-readonly` は視覚化しない: 参照 4 サイト（chakra-ui / Radix
        // Themes / Radix Primitives / ark-ui）のいずれも readonly 状態に
        // radio 固有の視覚差を付けないため、`data-invalid`（上記）とは
        // 異なり CSS 規則を追加しない（`checkbox` #1454 と同じ判断）。
        // イシュー #1495: 5 段の `.variant(Size::*, "root", ...)` を個別に
        // 手書きする代わりに `size_variants`（イシュー #1424 の共通生成
        // 手段、`checkbox.rs`/`checkbox_group.rs` と同型）を使い、既定 `md`
        // の設定漏れを構造的に防ぐ（規約は `docs/design/
        // pre-styled-ui-focus-ring-and-size-conventions.md` §4）。control
        // 寸法は xs/sm のみ 4px 格子（0.75rem/0.875rem）へ是正し、md/lg/xl
        // は既存外観を維持する（`checkbox.rs` #1735 と同値）。dot-inset・
        // font-size の段階は現状維持（意図的な例外、モジュール rustdoc
        // 参照）。`--fandhe-radio-group-gap`（root の項目間余白）・
        // `--fandhe-radio-group-item-gap`（item 内の control ↔ text 余白）は
        // 本イシューで新設した size 連動 custom property（値は
        // `checkbox_group.rs` の同名段階と揃える）。
        .size_variants(
            "root",
            &[
                (
                    Size::Xs,
                    vec![
                        decl("--fandhe-radio-group-control-size", "0.75rem"),
                        decl("--fandhe-radio-group-dot-inset", "1px"),
                        decl(
                            "--fandhe-radio-group-font-size",
                            "var(--fandhe-font-font-size-xs)",
                        ),
                        decl("--fandhe-radio-group-item-gap", "var(--fandhe-space-1)"),
                        decl("--fandhe-radio-group-gap", "var(--fandhe-space-0-5)"),
                    ],
                ),
                (
                    Size::Sm,
                    vec![
                        decl("--fandhe-radio-group-control-size", "0.875rem"),
                        decl("--fandhe-radio-group-dot-inset", "2px"),
                        decl(
                            "--fandhe-radio-group-font-size",
                            "var(--fandhe-font-font-size-sm)",
                        ),
                        decl("--fandhe-radio-group-item-gap", "var(--fandhe-space-1-5)"),
                        decl("--fandhe-radio-group-gap", "var(--fandhe-space-1)"),
                    ],
                ),
                (
                    Size::Md,
                    vec![
                        decl("--fandhe-radio-group-control-size", "1rem"),
                        decl("--fandhe-radio-group-dot-inset", "3px"),
                        decl(
                            "--fandhe-radio-group-font-size",
                            "var(--fandhe-font-font-size-sm)",
                        ),
                        decl("--fandhe-radio-group-item-gap", "var(--fandhe-space-2)"),
                        decl("--fandhe-radio-group-gap", "var(--fandhe-space-1)"),
                    ],
                ),
                (
                    Size::Lg,
                    vec![
                        decl("--fandhe-radio-group-control-size", "1.25rem"),
                        decl("--fandhe-radio-group-dot-inset", "4px"),
                        decl(
                            "--fandhe-radio-group-font-size",
                            "var(--fandhe-font-font-size-md)",
                        ),
                        decl("--fandhe-radio-group-item-gap", "var(--fandhe-space-2-5)"),
                        decl("--fandhe-radio-group-gap", "var(--fandhe-space-1-5)"),
                    ],
                ),
                (
                    Size::Xl,
                    vec![
                        decl("--fandhe-radio-group-control-size", "1.5rem"),
                        decl("--fandhe-radio-group-dot-inset", "5px"),
                        decl(
                            "--fandhe-radio-group-font-size",
                            "var(--fandhe-font-font-size-lg)",
                        ),
                        decl("--fandhe-radio-group-item-gap", "var(--fandhe-space-3)"),
                        decl("--fandhe-radio-group-gap", "var(--fandhe-space-2)"),
                    ],
                ),
            ],
        )
        .default_variant(ColorPalette::Accent);

    for palette in [
        ColorPalette::Accent,
        ColorPalette::Info,
        ColorPalette::Success,
        ColorPalette::Warning,
        ColorPalette::Danger,
        ColorPalette::Neutral,
    ] {
        recipe = recipe.variant(palette, "root", palette_scale_declarations(palette));
    }
    recipe
}

/// この styled RadioGroup が生成する静的 CSS 全量を返す（決定的。
/// [`crate::select::stylesheet`] と同じ契約）。
#[must_use]
pub fn stylesheet() -> String {
    recipe().css()
}

/// styled root パーツを組み立てる。`size`/`palette` に応じたクラスを付与
/// する唯一のパーツ（[`drop_class_attr`] により呼び出し側の `class` は除去
/// してから合成する）。実体は
/// [`fandhe_frontend_headless_ui::radio_group::root`] へ委譲する。
///
/// 公開シグネチャは互換性のため `disabled: bool` 単体のまま維持し、内部で
/// 他フラグ（readonly/invalid/required）を既定値（false）とした
/// [`RadioGroupProps`] を組み立てて [`root_with_props`] へ委譲する。
/// readonly/invalid/required も反映したい場合は [`root_with_props`] を使う
/// こと（イシュー #1616 codex-review 指摘: 本関数は disabled 以外を常に
/// 既定値へ戻してしまい、子パーツ側は `RadioGroupProps` 対応済みなのに
/// root の `aria-readonly`/`aria-required`/`data-invalid`/`data-required`
/// だけが出力できずグループ全体でアクセシビリティ契約が不整合になる問題の
/// 是正）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::radio_group;
/// use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};
///
/// let node = radio_group::root(
///     Size::Md,
///     ColorPalette::Accent,
///     false,
///     None,
///     None,
///     vec![],
///     vec![],
/// );
/// assert!(render(&node).contains(r#"data-scope="radio-group" data-part="root""#));
/// ```
#[must_use]
pub fn root<'a>(
    size: Size,
    palette: ColorPalette,
    disabled: bool,
    orientation: Option<Orientation>,
    labelled_by: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let props = RadioGroupProps {
        disabled,
        ..RadioGroupProps::default()
    };
    root_with_props(
        size,
        palette,
        &props,
        orientation,
        labelled_by,
        attrs,
        children,
    )
}

/// styled root パーツを、全 [`RadioGroupProps`]（disabled/readonly/invalid/
/// required）を反映して組み立てる（イシュー #1616 codex-review 指摘の是正）。
/// [`root`] と実体を共有するが、readonly/invalid/required を既定値へ落とさず
/// 呼び出し側の `props` をそのまま headless
/// [`fandhe_frontend_headless_ui::radio_group::root`] へ渡す。子パーツ
/// （[`item`]/[`item_control`]/[`item_text`]/[`item_hidden_input`]）へ渡す
/// `props` と同一の値をここへも渡すことで、group 全体（root/label/item 系）
/// の `data-readonly`/`data-invalid`/`data-required`/`aria-*` が一貫する。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::radio_group;
/// use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};
/// use fandhe_frontend_headless_ui::radio_group::RadioGroupProps;
///
/// let props = RadioGroupProps {
///     readonly: true,
///     ..RadioGroupProps::default()
/// };
/// let node = radio_group::root_with_props(
///     Size::Md,
///     ColorPalette::Accent,
///     &props,
///     None,
///     None,
///     vec![],
///     vec![],
/// );
/// assert!(render(&node).contains(r#"aria-readonly="true""#));
/// ```
#[must_use]
pub fn root_with_props<'a>(
    size: Size,
    palette: ColorPalette,
    props: &RadioGroupProps,
    orientation: Option<Orientation>,
    labelled_by: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let recipe = recipe();
    let class =
        recipe.variant_classes(&[("size", size.value()), ("color-palette", palette.value())]);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    fandhe_frontend_headless_ui::radio_group::root(
        props,
        orientation,
        labelled_by,
        merged,
        children,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};

    /// 生成済み CSS 文字列から、指定セレクタで始まる 1 ルールブロック
    /// （`{` 〜 対応する `}` まで）の宣言本文だけを取り出す（PR #1769
    /// レビュー指摘対応のテスト専用ヘルパ）。`css.contains("opacity: 0.5;")`
    /// のような全文検索では、同じ宣言を持つ別セレクタのルール（例:
    /// `item` の disabled 規則）が存在するだけで偽陽性になり、対象
    /// セレクタ自身が opacity を出していないことを検証できないため導入した。
    fn rule_body<'a>(css: &'a str, selector: &str) -> &'a str {
        let start = css
            .find(selector)
            .unwrap_or_else(|| panic!("selector `{selector}` not found in stylesheet"));
        let open = css[start..]
            .find('{')
            .map(|i| start + i + 1)
            .unwrap_or_else(|| panic!("no `{{` after selector `{selector}`"));
        let close = css[open..]
            .find('}')
            .map(|i| open + i)
            .unwrap_or_else(|| panic!("no `}}` closing selector `{selector}`"));
        &css[open..close]
    }

    #[test]
    fn stylesheet_is_deterministic_and_targets_data_scope_selectors() {
        let a = stylesheet();
        let b = stylesheet();
        assert_eq!(a, b);
        assert!(a.contains(r#"[data-scope="radio-group"][data-part="item-control"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    #[test]
    fn item_hidden_input_is_visually_hidden() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="radio-group"][data-part="item-hidden-input"]"#));
        assert!(css.contains("clip: rect(0, 0, 0, 0);"));
        assert!(css.contains("position: absolute;"));
    }

    #[test]
    fn stylesheet_links_data_state_checked_to_item_control_style() {
        let css = stylesheet();
        assert!(css.contains(
            r#"[data-scope="radio-group"][data-part="item-control"][data-state="checked"]"#
        ));
        assert!(css.contains("border-color: var(--fandhe-palette, var(--fandhe-color-accent));"));
    }

    #[test]
    fn root_switches_to_row_layout_on_horizontal_orientation() {
        let css = stylesheet();
        assert!(css.contains(
            r#"[data-scope="radio-group"][data-part="root"][data-orientation="horizontal"]"#
        ));
        assert!(css.contains("flex-direction: row;"));
    }

    #[test]
    fn disabled_item_gets_not_allowed_cursor() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="radio-group"][data-part="item"][data-disabled]"#));
        assert!(css.contains("cursor: not-allowed;"));
    }

    #[test]
    fn item_focus_within_gets_accent_outline_ring() {
        // イシュー #683 受け入れ条件: visually-hidden 化した `item-hidden-input`
        // への実フォーカスが、祖先 `item` の `:focus-within` として反映される。
        // イシュー #1424 でリング色がフォーカスリング専用トークン
        // （`--fandhe-color-focus-ring`）経由の canonical 形へ移行した。
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="radio-group"][data-part="item"]:focus-within {"#));
        assert!(css.contains(
            "outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-palette, var(--fandhe-color-focus-ring, var(--fandhe-color-accent)));"
        ));
    }

    // --- イシュー #1494（root/item/item-control のスタイル調整） ---

    #[test]
    fn item_control_focus_visible_ring_is_palette_connected() {
        // イシュー #1494: `item` の `:focus-within` と同じ `Palette` 参照形
        // へ統一する（`docs/design/pre-styled-ui-focus-ring-and-size-
        // conventions.md` §6 手順 2）。
        let css = stylesheet();
        assert!(css.contains(
            r#"[data-scope="radio-group"][data-part="item-control"][data-focus-visible] {"#
        ));
        assert!(css.contains(
            "outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-palette, var(--fandhe-color-focus-ring, var(--fandhe-color-accent)));"
        ));
    }

    #[test]
    fn item_control_has_unchecked_hover_and_transition() {
        let css = stylesheet();
        assert!(css.contains("--fandhe-hover-bg: var(--fandhe-color-bg-muted);"));
        assert!(css.contains("transition-property: background, border-color;"));
        assert!(css.contains("transition-duration: var(--fandhe-motion-duration-fast);"));
        assert!(css.contains(
            r#"[data-scope="radio-group"][data-part="item-control"]:hover:not([data-disabled]) {"#
        ));
        assert!(css.contains("background: var(--fandhe-hover-bg);"));
    }

    #[test]
    fn item_control_checked_state_overrides_hover_bg_with_fallback() {
        let css = stylesheet();
        assert!(css.contains(
            "--fandhe-hover-bg: var(--fandhe-palette-emphasized, var(--fandhe-color-accent-emphasized));"
        ));
    }

    #[test]
    fn item_control_has_border_box_sizing() {
        let css = stylesheet();
        assert!(css.contains("box-sizing: border-box;"));
    }

    #[test]
    fn item_control_invalid_gets_danger_border() {
        let css = stylesheet();
        assert!(
            css.contains(r#"[data-scope="radio-group"][data-part="item-control"][data-invalid] {"#)
        );
        assert!(css.contains("border-color: var(--fandhe-color-danger);"));
    }

    #[test]
    fn root_disabled_state_block_is_not_emitted() {
        // PR #1769 codex-review P1 再指摘への最終対応（`root` disabled 規則
        // の doc コメント参照）: `root[data-disabled]` へ `.state` を登録
        // していないため、対応する CSS ブロック自体が出力されないことを
        // 固定する（再導入の検知。`crate::checkbox_group` の同名テストと
        // 同型）。
        let css = stylesheet();
        let scope = r#"[data-scope="radio-group"][data-part="root"][data-disabled]"#;
        assert!(
            !css.contains(scope),
            "root[data-disabled] block must not be emitted (see `root()` disabled              state comment in radio_group.rs for why the CSS-only cascade was              removed): {css:?}"
        );
    }

    #[test]
    fn item_disabled_still_gets_full_disabled_declarations() {
        // `root` disabled 規則を持たない現在の設計でも、`item` 個別 disabled
        // （グループ全体は有効なまま 1 件だけ無効化する既存ユースケース、
        // `showcase::radio_group_section` 参照）は従来どおり
        // `disabled_declarations()`（opacity + cursor）を保つ。
        let css = stylesheet();
        let item_disabled_selector =
            r#"[data-scope="radio-group"][data-part="item"][data-disabled]"#;
        let body = rule_body(&css, item_disabled_selector);
        assert!(body.contains("opacity: 0.5;"));
        assert!(body.contains("cursor: not-allowed;"));
    }

    // --- variant クラス（イシュー #708） ---

    #[test]
    fn root_outputs_scope_and_part() {
        let html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            false,
            None,
            None,
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="radio-group""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(html.contains(r#"role="radiogroup""#));
    }

    #[test]
    fn default_variant_is_md_and_accent() {
        let html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            false,
            None,
            None,
            vec![],
            vec![],
        ));
        assert!(html.contains("fd-radio-group--size-md"));
        assert!(html.contains("fd-radio-group--color-palette-accent"));
    }

    #[test]
    fn size_enumeration_maps_to_expected_classes() {
        for (size, class) in [
            (Size::Xs, "fd-radio-group--size-xs"),
            (Size::Sm, "fd-radio-group--size-sm"),
            (Size::Md, "fd-radio-group--size-md"),
            (Size::Lg, "fd-radio-group--size-lg"),
            (Size::Xl, "fd-radio-group--size-xl"),
        ] {
            let html = render(&root(
                size,
                ColorPalette::Accent,
                false,
                None,
                None,
                vec![],
                vec![],
            ));
            assert!(html.contains(class), "size={size:?} -> {html}");
        }
    }

    #[test]
    fn palette_enumeration_maps_to_expected_classes() {
        for (palette, class) in [
            (ColorPalette::Accent, "fd-radio-group--color-palette-accent"),
            (ColorPalette::Info, "fd-radio-group--color-palette-info"),
            (
                ColorPalette::Success,
                "fd-radio-group--color-palette-success",
            ),
            (
                ColorPalette::Warning,
                "fd-radio-group--color-palette-warning",
            ),
            (ColorPalette::Danger, "fd-radio-group--color-palette-danger"),
            (
                ColorPalette::Neutral,
                "fd-radio-group--color-palette-neutral",
            ),
        ] {
            let html = render(&root(Size::Md, palette, false, None, None, vec![], vec![]));
            assert!(html.contains(class), "palette={palette:?} -> {html}");
        }
    }

    #[test]
    fn reexported_root_with_horizontal_orientation_emits_data_orientation() {
        let html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            false,
            Some(Orientation::Horizontal),
            None,
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-orientation="horizontal""#));
    }

    #[test]
    fn class_attr_is_single_and_caller_class_is_dropped() {
        let html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            false,
            None,
            None,
            vec![("class", "attacker-controlled")],
            vec![],
        ));
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(!html.contains("attacker-controlled"));
    }

    #[test]
    fn stylesheet_contains_size_and_palette_variant_selectors() {
        let css = stylesheet();
        assert!(css.contains("--size-"));
        assert!(css.contains("--color-palette-"));
        assert!(css.contains("--fandhe-radio-group-control-size"));
    }

    // --- イシュー #1495（size/orientation バリアントとラベル・説明の型階層） ---

    #[test]
    fn root_orientation_horizontal_wraps_with_space_4_column_gap() {
        // モジュール doc「orientation（horizontal）の是正」節の回帰固定
        // （`crate::checkbox_group` #1460 と同型）。
        let css = stylesheet();
        let scope =
            r#"[data-scope="radio-group"][data-part="root"][data-orientation="horizontal"] {"#;
        let body = rule_body(&css, scope);
        assert!(body.contains("flex-direction: row;"));
        assert!(body.contains("flex-wrap: wrap;"));
        assert!(body.contains("column-gap: var(--fandhe-space-4);"));
        assert!(body.contains("--fandhe-radio-group-label-basis: 100%;"));
    }

    #[test]
    fn root_gap_uses_custom_property_with_space_1_fallback() {
        let css = stylesheet();
        let scope = r#"[data-scope="radio-group"][data-part="root"] {"#;
        let body = rule_body(&css, scope);
        assert!(body.contains("gap: var(--fandhe-radio-group-gap, var(--fandhe-space-1));"));
    }

    #[test]
    fn item_base_gap_uses_item_gap_custom_property_with_space_2_fallback() {
        let css = stylesheet();
        let scope = r#"[data-scope="radio-group"][data-part="item"] {"#;
        let body = rule_body(&css, scope);
        assert!(body.contains("gap: var(--fandhe-radio-group-item-gap, var(--fandhe-space-2));"));
        assert!(body.contains("width: fit-content;"));
    }

    #[test]
    fn label_has_medium_font_weight_and_item_text_does_not() {
        // イシュー #1495: `label`（グループ見出し、medium）と `item-text`
        // （項目テキスト、通常ウェイト）の 2 段階の型階層を固定する
        // （`crate::checkbox_group` #1461 と同型）。
        let css = stylesheet();
        let label_body = rule_body(&css, r#"[data-scope="radio-group"][data-part="label"] {"#);
        assert!(label_body.contains("font-weight: var(--fandhe-font-font-weight-medium);"));
        assert!(label_body.contains("line-height: var(--fandhe-font-line-height-normal);"));
        assert!(label_body.contains(
            "font-size: var(--fandhe-radio-group-font-size, var(--fandhe-font-font-size-sm));"
        ));

        let item_text_body = rule_body(
            &css,
            r#"[data-scope="radio-group"][data-part="item-text"] {"#,
        );
        assert!(item_text_body.contains("line-height: var(--fandhe-font-line-height-normal);"));
        assert!(item_text_body.contains("user-select: none;"));
        assert!(
            !item_text_body.contains("font-weight:"),
            "item-text block should not declare font-weight: {item_text_body}"
        );
    }

    #[test]
    fn size_variants_control_size_is_4px_grid_for_xs_and_sm() {
        // イシュー #1495: xs/sm を 4px 格子（12px/14px）へ是正（`checkbox.rs`
        // #1735 と同一段階）。md/lg/xl は既存外観を維持する。
        let css = stylesheet();
        let expected = [
            (Size::Xs, "0.75rem"),
            (Size::Sm, "0.875rem"),
            (Size::Md, "1rem"),
            (Size::Lg, "1.25rem"),
            (Size::Xl, "1.5rem"),
        ];
        for (size, control_size) in expected {
            let selector = format!(
                r#"[data-scope="radio-group"][data-part="root"].fd-radio-group--size-{}"#,
                size.value()
            );
            let body = rule_body(&css, &selector);
            let expected_decl = format!("--fandhe-radio-group-control-size: {control_size};");
            assert!(
                body.contains(&expected_decl),
                "size={size:?} variant block missing {expected_decl}: {body}"
            );
        }
    }

    #[test]
    fn size_variants_set_item_gap_custom_property_monotonically() {
        // イシュー #1495: `--fandhe-radio-group-item-gap` が xs〜xl で
        // spacing トークン経由の単調増加になることを固定する
        // （`checkbox_group.rs` の同名テストと同型）。
        let css = stylesheet();
        let expected = [
            (Size::Xs, "var(--fandhe-space-1)"),
            (Size::Sm, "var(--fandhe-space-1-5)"),
            (Size::Md, "var(--fandhe-space-2)"),
            (Size::Lg, "var(--fandhe-space-2-5)"),
            (Size::Xl, "var(--fandhe-space-3)"),
        ];
        for (size, gap) in expected {
            let selector = format!(
                r#"[data-scope="radio-group"][data-part="root"].fd-radio-group--size-{}"#,
                size.value()
            );
            let body = rule_body(&css, &selector);
            let expected_decl = format!("--fandhe-radio-group-item-gap: {gap};");
            assert!(
                body.contains(&expected_decl),
                "size={size:?} variant block missing {expected_decl}: {body}"
            );
        }
    }

    #[test]
    fn size_variants_set_root_gap_custom_property_non_decreasing() {
        // イシュー #1495: `--fandhe-radio-group-gap`（root の項目間余白）が
        // xs〜xl で非減少（sm と md が同値、md の現行外観維持のため厳密単調
        // にしない）になることを固定する。
        let css = stylesheet();
        let expected = [
            (Size::Xs, "var(--fandhe-space-0-5)"),
            (Size::Sm, "var(--fandhe-space-1)"),
            (Size::Md, "var(--fandhe-space-1)"),
            (Size::Lg, "var(--fandhe-space-1-5)"),
            (Size::Xl, "var(--fandhe-space-2)"),
        ];
        for (size, gap) in expected {
            let selector = format!(
                r#"[data-scope="radio-group"][data-part="root"].fd-radio-group--size-{}"#,
                size.value()
            );
            let body = rule_body(&css, &selector);
            let expected_decl = format!("--fandhe-radio-group-gap: {gap};");
            assert!(
                body.contains(&expected_decl),
                "size={size:?} variant block missing {expected_decl}: {body}"
            );
        }
    }

    #[test]
    fn caller_data_scope_and_part_spoofing_is_dropped() {
        let html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            false,
            None,
            None,
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="radio-group""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("attacker"));
    }

    // --- エスケープ回帰 ---

    #[test]
    fn xss_payload_in_item_value_is_escaped_by_render() {
        // REQ-1 回帰: `data-value`（動的値）へ与えた XSS ペイロードが
        // `render()` の既定エスケープを経由することを固定する。
        let payload = "\"><script>alert(1)</script>";
        let html = render(&item(
            false,
            &RadioGroupProps::default(),
            payload,
            vec![],
            vec![text(payload)],
        ));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn xss_payload_in_item_text_children_is_escaped_by_render() {
        let payload = "\"><img src=x onerror=alert(1)>";
        let html = render(&item_text(
            false,
            &RadioGroupProps::default(),
            vec![],
            vec![text(payload)],
        ));
        assert!(!html.contains("<img"));
        assert!(html.contains("&lt;img"));
    }

    #[test]
    fn ssr_and_hydration_round_trip_via_reexported_radio_group_state_machine() {
        // 再エクスポートされた `RadioGroup`（headless の Component/Hydrate
        // 実装をそのまま継承）経由で SSR/hydration 往復を固定する
        // （[`crate::select`] の同型テストに準拠）。
        use fandhe_frontend_interactive::{dispatch, render_for_hydration, Hydrate};

        let mut g = RadioGroup::default();
        assert_eq!(g.value(), None);

        assert!(dispatch(&mut g, "select", "red"));
        assert_eq!(g.value(), Some("red"));

        let ssr_html = render(&g.item_control("red", &RadioGroupProps::default(), vec![]));
        assert!(ssr_html.contains(r#"data-state="checked""#));

        let hydrate_html = render(&render_for_hydration(&g));
        assert!(hydrate_html.contains("data-hydrate-"));

        let restored = RadioGroup::from_hydration_attrs(&g.hydration_attrs()).unwrap();
        assert_eq!(restored.value(), Some("red"));
    }
}
