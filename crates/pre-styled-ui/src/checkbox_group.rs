//! styled CheckboxGroup（headless ラッパー、イシュー #997、親 #520/#545）。
//!
//! `fandhe_frontend_headless_ui::checkbox_group`（イシュー #997）の
//! Label / Item / ItemControl / ItemIndicator / ItemText 5 anatomy パーツと
//! [`fandhe_frontend_headless_ui::checkbox_group::CheckboxGroup`] 状態機械を
//! そのまま再エクスポートし、[`stylesheet`] で既定 CSS を追加提供する。薄い
//! 委譲の根拠・スコープ外事項は [`crate::radio_group`] の rustdoc と同じ
//! 方針に従う。
//!
//! # 選択的 re-export（`pub use ...::*` を使わない理由）
//!
//! 本モジュールは `size`/`palette` variant クラス付与のため styled `root`
//! （[`crate::radio_group::root`] と同型）を本モジュールで再定義する。
//! headless 自由関数 `root` と名前衝突するため、`pub use ...::*` ではなく
//! 必要な識別子（[`label`]/[`item`]/[`item_control`]/[`item_indicator`]/
//! [`item_text`]/[`CheckboxGroup`]/[`DATA_STATE_CHECKED`]/
//! [`DATA_STATE_UNCHECKED`]）のみを選択的に再エクスポートする。
//!
//! [`CheckboxGroup`] 状態機械は inherent `root()` を持たない（item 系
//! メソッドのみ、`crates/headless-ui/src/checkbox_group.rs` 参照）ため、
//! そのまま再エクスポートしても未スタイル `root` の静かな適用漏れは
//! 発生しない（[`crate::radio_group`] の `RadioGroup` 非対称処理と同じ判断）。
//!
//! # `item-hidden-input` を本モジュールが持たない理由（`checkbox::stylesheet()` 併用が必須）
//!
//! headless 層（`crates/headless-ui/src/checkbox_group.rs`）は
//! `item-hidden-input` パーツを新設せず、ネイティブ `<input type="checkbox">`
//! を [`fandhe_frontend_headless_ui::checkbox::hidden_input`] の入れ子
//! 再利用で賄う（headless 側モジュール doc「anatomy」節参照）。この設計を
//! 継承し、**本モジュールの [`recipe`] は `hidden-input` slot の
//! visually-hidden 規則を一切再宣言しない**
//! （`[data-scope="checkbox"][data-part="hidden-input"]` として
//! `crate::checkbox` の recipe に既存であり、本モジュールで重複実装すると
//! `checkbox` recipe とのドリフト・二重管理を招く）。styled CheckboxGroup を
//! 使う呼び出し側は、本モジュールの [`stylesheet`] に加えて
//! `fandhe_frontend_pre_styled_ui::checkbox::stylesheet()`
//! も併せて読み込む必要がある（`crates/docs-site/src/showcase.rs` が
//! 両方を `push_css` する実例を参照）。
//!
//! # data-state とスタイルの連動
//!
//! `item`/`item-control`/`item-indicator`（選択状態、
//! `data-state="checked"`/`"unchecked"`）の見た目の切り替えを [`recipe`] へ
//! 登録する（[`crate::recipe::SlotRecipe::state`]）。`root` の
//! `data-orientation="horizontal"` でも配置切り替えを行う（[`crate::radio_group`]
//! と同型）。
//!
//! # `:focus-within` によるフォーカスリング
//!
//! 実フォーカスは（呼び出し側が入れ子にする）
//! `fandhe_frontend_pre_styled_ui::checkbox::hidden_input` が受ける。
//! [`crate::radio_group`] と同じ理由（`item`〔`<label>`〕が hidden input の
//! 祖先であること）により、`item` へ `:focus-within` のフォーカスリングを
//! 登録する。
//!
//! # `size`/`palette` variant
//!
//! [`crate::radio_group`] rustdoc「`size`/`palette` variant」節と同じ設計
//! （`root` スコープの custom property 経由で `item-control`/`item-text` の
//! 寸法・見た目を切り替える）に従う。`size` は
//! `--fandhe-checkbox-group-control-size`/`-font-size` を、`palette` は
//! [`crate::recipe::palette_scale_declarations`] を登録する。`var()` にはいずれも
//! Md サイズ・Accent パレット相当のフォールバック値を書き、styled `root` を
//! 経由しない headless 直接利用マークアップでも現行外観を維持する
//! （fail-safe）。
//!
//! # セキュリティ不変条件
//!
//! 本モジュールは headless 層の再エクスポートと静的 CSS 生成のみで構成され、
//! `raw_html()` を使用しない。CSS 宣言値はすべてコンパイル時静的リテラルで
//! あり、動的値（`value`/属性/children）へ CSS 値として流し込む経路を持たない
//! （動的値は headless 層経由で `fandhe_frontend_core::render` の既定
//! エスケープを必ず通る、REQ-1）。styled `root` は [`drop_class_attr`] に
//! より呼び出し側の `class` を除去してから合成するため、`class` 属性は常に
//! 単一（[`crate::radio_group::root`] と同型）。
//!
//! # スタイル調整（イシュー #1460、グループレイアウト・invalid 伝播）
//!
//! 親 #1459「checkbox-group のスタイルを参考サイト基準へ調整」の 1/2。
//! 担当範囲は **グループレイアウト（`orientation`/`gap`）と invalid 伝播**
//! に限る（size バリアント値・label/item-text の型階層は 2/2 #1461 の担当）。
//! 参照サイトとの差分・意図的に合わせなかった判断は以下のとおり:
//!
//! - **横並び（`data-orientation="horizontal"`）は折り返し + `space-4` の
//!   列間隔を追加**: 是正前は `flex-direction: row` のみで、縦積み用の
//!   `gap: space-1`（4px）のまま横並びになり折り返しもなかった。
//!   Radix Themes に `orientation` 相当はないが chakra の `HStack gap=4`
//!   相当に寄せ、`flex-wrap: wrap` + `column-gap: var(--fandhe-space-4)` を
//!   追加する（`row-gap` は縦積みと同じ `gap`〔custom property 化、後述〕を
//!   継承させ、`row-gap` の個別宣言は追加しない）。
//! - **`gap` を `--fandhe-checkbox-group-gap` custom property 経由へ**:
//!   2/2（#1461）が size variant ごとに同名 custom property を定義する受け口
//!   として用意する（`checkbox::root` の `--fandhe-checkbox-gap` と同型）。
//!   未定義時は現行の `--fandhe-space-1` へフォールバックし、styled `root`
//!   を経由しない headless 直接利用マークアップでも外観を維持する。
//! - **item のクリック領域を `width: fit-content` に**: 是正前は縦積みの
//!   `flex`（`align-items: stretch` 既定）で `<label>` が行幅いっぱいに
//!   伸び、クリック領域が余白まで広がっていた。Radix Themes item の
//!   `width: fit-content` に合わせる。
//! - **`root` の `data-invalid`（headless 未出力、`attrs` 経由でのみ付与
//!   可能）を `item-control` の border-color へ伝播**: `crate::checkbox`
//!   （#1734）・`crate::checkbox_card`（#1736）と同じ Forms 家族の視覚言語
//!   （border-color のみ danger 色化）に統一する。chakra の `_invalid` が
//!   `colorPalette` ごと red へ反転する挙動は Forms 家族横断で判断すべき
//!   （`checkbox` 1/2 の variant 軸非採用判断と同じ理由）ため踏襲しない。
//!   headless 層（`crates/headless-ui/src/checkbox_group.rs`）は `root`/
//!   `item`/`item-control` のいずれにも `data-invalid` を出力しない
//!   （Field 連携は #1603 の射程、本モジュール doc「本イシューのスコープ外」
//!   節参照）ため、本 CSS は**参照のみ**を追加する。利用者は `root` の
//!   `attrs` へ `(\"data-invalid\", \"\")` を直接渡すことで有効化できる。
//! - **`root` の `data-disabled` を custom property 経由で `item` へ伝播**:
//!   `root`/`item` の双方が disabled のとき `disabled_declarations()` を
//!   両方へ直接置くと `opacity` が 0.5 × 0.5 に多重適用されるため、`root`
//!   側は `--fandhe-checkbox-group-item-opacity`/`-item-cursor` custom
//!   property のみを定義し、`item` がそれを既定値（`1`/`pointer`）付きで
//!   参照する形にする。ビジュアル言語自体（`opacity: 0.5` +
//!   `cursor: not-allowed`）は変えない。
//! - **`item:focus-within`/`item-control` のフォーカスリング・hover・
//!   transition を canonical ヘルパへ移行**: `recipe::focus_ring_declarations`
//!   （`FocusRingColor::Palette`、`FocusRingOffset::Outside`）・
//!   `recipe::hover_bg_muted`/`hover_surface_declarations`・
//!   `recipe::transition_declarations`（`MotionDuration::Fast`）へ置き換え、
//!   手書きのリテラル（`2px solid ...`・`0.15s` 等）を排除する。`checkbox.rs`
//!   と同型のパターン（`hover_bg_solid_with_fallback` はモジュール内へ複製、
//!   共通化は out-of-scope として記録）。
//! - **`item` 行全体の hover は追加しない**: 参照 3 サイト（chakra-ui /
//!   Ark UI / Radix Themes・Primitives）のいずれも checkbox-group の
//!   `item`（list-item 型のクリック領域）行全体を hover 対象にしない。
//!   `checkbox.rs` と同じく `item-control` のみを hover 対象とする。
//! - **variant 軸（chakra solid/subtle/outline 等）は追加しない**:
//!   `checkbox.rs`（#1734）の判断（Forms 家族横断の判断が必要、部品単独で
//!   先行しない）を踏襲する。
//! - **`item-control[data-focus-visible]` は追加しない**: wasm-full の
//!   `focus_visible.rs` に checkbox-group の写像がなく（out-of-scope として
//!   記録）、配線しても効かない CSS を残さないため。
//!
//! # 本イシューのスコープ外
//!
//! headless 層モジュール doc「out-of-scope」節（キーボードナビゲーション・
//! 実 DOM 配線・全選択/一部選択集約・Field 連携・`checkbox_card` を item
//! として使う構成）をそのまま継承する。加えて本イシュー（#1460）で新たに
//! 判明したスコープ外事項:
//!
//! - `hover_bg_solid_with_fallback()` の `crate::recipe` への共通化
//!   （`checkbox.rs`/`checkbox_group.rs` で重複）
//! - wasm-full `focus_visible.rs` への
//!   `(\"checkbox-group\", \"hidden-input\") -> \"item-control\"` 写像追加
//! - headless 側での `data-invalid` 出力（`root`/`item`/`item-control` への
//!   `invalid` フラグ追加）は #1603 の射程
//! - variant 軸（chakra solid/subtle/outline 相当）の Forms 家族横断判断
//! - size variant 値・label/item-text 型階層は 2/2（#1461）が担当

use crate::class_attr::drop_class_attr;
use crate::css::{decl, Declaration};
use crate::recipe::{
    disabled_declarations, focus_ring_declarations, hover_bg_muted, hover_surface_declarations,
    palette_scale_declarations, transition_declarations, ColorPalette, FocusRingColor,
    FocusRingOffset, MotionDuration, Size, SlotRecipe, StateCondition, VariantValue,
};

// headless 自由関数 `root` はあえて再エクスポートしない（本モジュール冒頭
// の rustdoc「選択的 re-export」節参照、`root` は本モジュールで styled 版
// として再定義する）。
pub use fandhe_frontend_headless_ui::checkbox_group::{
    item, item_control, item_indicator, item_text, label, CheckboxGroup, DATA_STATE_CHECKED,
    DATA_STATE_UNCHECKED,
};
use fandhe_frontend_headless_ui::data_attrs::Orientation;
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;

/// headless `checkbox_group` anatomy の `data-part` 一覧（`crates/headless-ui/src/checkbox_group.rs`
/// の `ANATOMY.part(...)` 呼び出しと同期させる契約。`item-hidden-input` を
/// 含まない理由はモジュール doc「`item-hidden-input` を本モジュールが
/// 持たない理由」節参照）。
const SLOTS: &[&str] = &[
    "root",
    "label",
    "item",
    "item-control",
    "item-indicator",
    "item-text",
];

/// checked 状態の `item-control` hover 面を定義する `--fandhe-hover-bg`
/// 宣言（内部ヘルパ）。`crate::checkbox::hover_bg_solid_with_fallback` と
/// 同一の宣言を本モジュール内へ複製する（共通化は out-of-scope として記録、
/// モジュール doc「本イシューのスコープ外」節参照）。
fn hover_bg_solid_with_fallback() -> Declaration {
    decl(
        "--fandhe-hover-bg",
        "var(--fandhe-palette-emphasized, var(--fandhe-color-accent-emphasized))",
    )
}

/// この styled CheckboxGroup の既定 CSS を組み立てる（内部ヘルパ、
/// [`stylesheet`] のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    let mut recipe = SlotRecipe::new("checkbox-group", SLOTS)
        .base(
            "root",
            vec![
                decl("display", "flex"),
                decl("flex-direction", "column"),
                // gap は custom property 経由（2/2 が size variant で値を
                // 切り替える受け口。未定義時は現行の space-1、モジュール doc
                // 参照）。
                decl(
                    "gap",
                    "var(--fandhe-checkbox-group-gap, var(--fandhe-space-1))",
                ),
            ],
        )
        // `data-orientation="horizontal"`（headless 層が `data_orientation`
        // 経由で出力）では縦積みではなく折り返し付きの横並びへ切り替える
        // （モジュール doc 参照。`column-gap` のみ追加指定し、`row-gap` は
        // 上記 `gap` を継承させる）。
        .state(
            "root",
            StateCondition::AttrEq("data-orientation", "horizontal"),
            vec![
                decl("flex-direction", "row"),
                decl("flex-wrap", "wrap"),
                decl("column-gap", "var(--fandhe-space-4)"),
            ],
        )
        // `data-invalid`（headless 未出力、`attrs` 経由でのみ付与可能。
        // モジュール doc 参照）を item-control の border-color へ伝播する
        // custom property のみを定義する（子孫セレクタを持たない
        // `SlotRecipe` の制約下で `--fandhe-table-stripe-bg` と同型の間接
        // 参照パターン）。
        .state(
            "root",
            StateCondition::Attr("data-invalid"),
            vec![decl(
                "--fandhe-checkbox-group-control-border-color",
                "var(--fandhe-color-danger)",
            )],
        )
        // `data-disabled`（headless 層が `data_disabled` 経由で出力）を
        // `item` へ custom property 経由で伝播する（モジュール doc「`root`
        // の `data-disabled` を custom property 経由で `item` へ伝播」節
        // 参照。`disabled_declarations()` を `root` へ直接置かない理由も
        // 同節）。
        .state(
            "root",
            StateCondition::Attr("data-disabled"),
            vec![
                decl("--fandhe-checkbox-group-item-opacity", "0.5"),
                decl("--fandhe-checkbox-group-item-cursor", "not-allowed"),
            ],
        )
        .base(
            "label",
            vec![
                decl("display", "block"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("font-size", "var(--fandhe-font-font-size-sm)"),
                decl("margin-bottom", "var(--fandhe-space-1)"),
            ],
        )
        .base(
            "item",
            vec![
                decl("display", "flex"),
                decl("align-items", "center"),
                decl("gap", "var(--fandhe-space-2)"),
                // Radix Themes item に合わせ、縦積み時に `<label>` が行幅
                // いっぱいへ伸びクリック領域が余白まで広がるのを防ぐ
                // （モジュール doc 参照）。
                decl("width", "fit-content"),
                decl(
                    "opacity",
                    "var(--fandhe-checkbox-group-item-opacity, 1)",
                ),
                decl(
                    "cursor",
                    "var(--fandhe-checkbox-group-item-cursor, pointer)",
                ),
            ],
        )
        .base(
            "item-control",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("justify-content", "center"),
                decl("box-sizing", "border-box"),
                decl("width", "var(--fandhe-checkbox-group-control-size, 1rem)"),
                decl("height", "var(--fandhe-checkbox-group-control-size, 1rem)"),
                decl(
                    "border",
                    "1px solid var(--fandhe-checkbox-group-control-border-color, var(--fandhe-color-border))",
                ),
                decl("border-radius", "var(--fandhe-radius-sm)"),
                decl("background", "var(--fandhe-color-bg)"),
                decl("flex-shrink", "0"),
                // unchecked 時の hover 面（`checkbox.rs` と同型、モジュール
                // doc 参照）。checked 時は下記 state 規則が同名 custom
                // property を上書きする。
                hover_bg_muted(),
            ],
        )
        // `base` は同一 slot への複数回登録が許され出力順で連結されるため、
        // 上記 base ブロックを書き換えずに純追加する（`checkbox.rs` と同型）。
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
                    "var(--fandhe-checkbox-group-font-size, var(--fandhe-font-font-size-sm))",
                ),
            ],
        )
        // イシュー #997: indicator の base に `display` 宣言を置かない
        // （headless 層が `data-state="unchecked"` 時に `hidden` 存在属性を
        // 付与する規約と衝突させないため。`crates/pre-styled-ui/src/checkbox.rs`
        // 「`indicator` の `hidden` 属性意味論を CSS が壊さない設計」節と
        // 同型の判断）。
        .base(
            "item-indicator",
            vec![
                // イシュー #997 Bugbot 指摘（Medium）回帰固定: 固定寸法ではなく
                // `--fandhe-checkbox-group-check-width`/`-check-height`
                // custom property（`root` の size variant が切り替える）を
                // 参照する。`crates/pre-styled-ui/src/checkbox.rs` の
                // `indicator`（`--fandhe-checkbox-check-width`/`-check-height`）
                // と同型。
                decl("width", "var(--fandhe-checkbox-group-check-width, 0.25rem)"),
                decl(
                    "height",
                    "var(--fandhe-checkbox-group-check-height, 0.5rem)",
                ),
                decl(
                    "border-right",
                    "2px solid var(--fandhe-palette-fg, var(--fandhe-color-accent-fg))",
                ),
                decl(
                    "border-bottom",
                    "2px solid var(--fandhe-palette-fg, var(--fandhe-color-accent-fg))",
                ),
                decl("transform", "rotate(45deg)"),
                decl("margin-bottom", "0.1rem"),
            ],
        )
        // 選択済み item-control の見た目（角丸の四角、palette 色の塗り。
        // ラジオの円形〔`border-radius: 50%`〕ではない）。root からの
        // invalid 伝播（`--fandhe-checkbox-group-control-border-color`）が
        // checked 状態でも勝つよう、border-color はその custom property を
        // 経由する（同一詳細度では出力順が勝つため、後続の
        // `item-control[data-invalid]` 直接規則より本規則を先に登録する）。
        .state(
            "item-control",
            StateCondition::AttrEq("data-state", "checked"),
            vec![
                decl(
                    "border-color",
                    "var(--fandhe-checkbox-group-control-border-color, var(--fandhe-palette, var(--fandhe-color-accent)))",
                ),
                decl(
                    "background",
                    "var(--fandhe-palette, var(--fandhe-color-accent))",
                ),
                // checked 面の hover は palette の emphasized 段へ
                // （`checkbox.rs` と同型、モジュール doc 参照）。
                hover_bg_solid_with_fallback(),
            ],
        )
        // `data-invalid`（headless 未出力、`attrs` 経由でのみ付与可能。
        // モジュール doc 参照）の item-control 直接規則。checked 規則より
        // 後方に登録し、同一詳細度の出力順で invalid を優先させる
        // （`checkbox.rs` と同型）。
        .state(
            "item-control",
            StateCondition::Attr("data-invalid"),
            vec![decl("border-color", "var(--fandhe-color-danger)")],
        )
        // hover の実適用は 1 本のみ（`--fandhe-hover-bg` の間接参照経由で
        // unchecked/checked いずれの面色にも追従する、`checkbox.rs` と同型）。
        .state(
            "item-control",
            StateCondition::Hover,
            hover_surface_declarations(),
        )
        // `data-disabled`（headless 層が `data_disabled` 経由で `item`/
        // `item-control`/`item-indicator`/`item-text` へ出力）時の操作不能な
        // 見た目。canonical ヘルパへ移行（`checkbox.rs` と同型、値は現行と
        // 同一）。
        .state(
            "item",
            StateCondition::Attr("data-disabled"),
            disabled_declarations(),
        )
        // 実フォーカスは呼び出し側が入れ子にする checkbox::hidden_input が
        // 受ける（visually-hidden 化されている）ため、祖先 `item`
        // （`<label>`）へ `:focus-within` で反映する（[`crate::radio_group`]
        // と同型のフォールバック、モジュール doc 参照）。canonical ヘルパ
        // （`FocusRingColor::Palette`、`palette` 軸を持つ部品）へ移行。
        .state(
            "item",
            StateCondition::FocusWithin,
            focus_ring_declarations(FocusRingColor::Palette, FocusRingOffset::Outside),
        )
        .variant(
            Size::Xs,
            "root",
            vec![
                decl("--fandhe-checkbox-group-control-size", "0.7rem"),
                decl("--fandhe-checkbox-group-check-width", "0.15rem"),
                decl("--fandhe-checkbox-group-check-height", "0.3rem"),
                decl(
                    "--fandhe-checkbox-group-font-size",
                    "var(--fandhe-font-font-size-xs)",
                ),
            ],
        )
        .variant(
            Size::Sm,
            "root",
            vec![
                decl("--fandhe-checkbox-group-control-size", "0.85rem"),
                decl("--fandhe-checkbox-group-check-width", "0.2rem"),
                decl("--fandhe-checkbox-group-check-height", "0.4rem"),
                decl(
                    "--fandhe-checkbox-group-font-size",
                    "var(--fandhe-font-font-size-sm)",
                ),
            ],
        )
        .variant(
            Size::Md,
            "root",
            vec![
                decl("--fandhe-checkbox-group-control-size", "1rem"),
                decl("--fandhe-checkbox-group-check-width", "0.25rem"),
                decl("--fandhe-checkbox-group-check-height", "0.5rem"),
                decl(
                    "--fandhe-checkbox-group-font-size",
                    "var(--fandhe-font-font-size-sm)",
                ),
            ],
        )
        .variant(
            Size::Lg,
            "root",
            vec![
                decl("--fandhe-checkbox-group-control-size", "1.25rem"),
                decl("--fandhe-checkbox-group-check-width", "0.3rem"),
                decl("--fandhe-checkbox-group-check-height", "0.6rem"),
                decl(
                    "--fandhe-checkbox-group-font-size",
                    "var(--fandhe-font-font-size-md)",
                ),
            ],
        )
        .variant(
            Size::Xl,
            "root",
            vec![
                decl("--fandhe-checkbox-group-control-size", "1.5rem"),
                decl("--fandhe-checkbox-group-check-width", "0.35rem"),
                decl("--fandhe-checkbox-group-check-height", "0.7rem"),
                decl(
                    "--fandhe-checkbox-group-font-size",
                    "var(--fandhe-font-font-size-lg)",
                ),
            ],
        )
        .default_variant(Size::Md)
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

/// この styled CheckboxGroup が生成する静的 CSS 全量を返す（決定的。
/// [`crate::radio_group::stylesheet`] と同じ契約）。CheckboxGroup を実際に
/// 利用する際は `crate::checkbox::stylesheet()` も併せて読み込む必要がある
/// （モジュール doc「`item-hidden-input` を本モジュールが持たない理由」
/// 節参照）。
#[must_use]
pub fn stylesheet() -> String {
    recipe().css()
}

/// styled root パーツを組み立てる。`size`/`palette` に応じたクラスを付与
/// する唯一のパーツ（[`drop_class_attr`] により呼び出し側の `class` は除去
/// してから合成する）。実体は
/// [`fandhe_frontend_headless_ui::checkbox_group::root`] へ委譲する。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::checkbox_group;
/// use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};
///
/// let node = checkbox_group::root(
///     Size::Md,
///     ColorPalette::Accent,
///     false,
///     None,
///     None,
///     vec![],
///     vec![],
/// );
/// assert!(render(&node).contains(r#"data-scope="checkbox-group" data-part="root""#));
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
    let recipe = recipe();
    let class =
        recipe.variant_classes(&[("size", size.value()), ("color-palette", palette.value())]);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    fandhe_frontend_headless_ui::checkbox_group::root(
        disabled,
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

    #[test]
    fn stylesheet_is_deterministic_and_targets_data_scope_selectors() {
        let a = stylesheet();
        let b = stylesheet();
        assert_eq!(a, b);
        assert!(a.contains(r#"[data-scope="checkbox-group"][data-part="item-control"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    #[test]
    fn stylesheet_never_duplicates_checkbox_hidden_input_rules() {
        // §設計判断（モジュール doc 参照）: visually-hidden の 9 宣言は
        // `crate::checkbox` recipe の `hidden-input` slot にのみ存在し、
        // 本 stylesheet では再宣言しない（重複実装の回帰固定）。
        let css = stylesheet();
        assert!(!css.contains("hidden-input"));
        assert!(!css.contains("clip: rect(0, 0, 0, 0);"));
    }

    #[test]
    fn stylesheet_links_data_state_checked_to_item_control_style() {
        let css = stylesheet();
        assert!(css.contains(
            r#"[data-scope="checkbox-group"][data-part="item-control"][data-state="checked"]"#
        ));
        assert!(css.contains(
            "border-color: var(--fandhe-checkbox-group-control-border-color, var(--fandhe-palette, var(--fandhe-color-accent)));"
        ));
        // ラジオの円形ではなく角丸の四角（border-radius: 50% を含まない）。
        assert!(!css.contains("border-radius: 50%;"));
    }

    #[test]
    fn root_orientation_horizontal_wraps_with_space_4_column_gap() {
        // モジュール doc「横並び（`data-orientation="horizontal"`）は折り
        // 返し + `space-4` の列間隔を追加」節の回帰固定（イシュー #1460）。
        let css = stylesheet();
        let scope =
            r#"[data-scope="checkbox-group"][data-part="root"][data-orientation="horizontal"] {"#;
        let start = css
            .find(scope)
            .expect("orientation=horizontal block missing");
        let end = css[start..]
            .find('}')
            .map(|i| start + i)
            .expect("closing brace missing");
        let block = &css[start..end];
        assert!(block.contains("flex-direction: row;"));
        assert!(block.contains("flex-wrap: wrap;"));
        assert!(block.contains("column-gap: var(--fandhe-space-4);"));
    }

    #[test]
    fn root_gap_uses_custom_property_with_space_1_fallback() {
        let css = stylesheet();
        let scope = r#"[data-scope="checkbox-group"][data-part="root"] {"#;
        let start = css.find(scope).expect("root base block missing");
        let end = css[start..]
            .find('}')
            .map(|i| start + i)
            .expect("closing brace missing");
        let block = &css[start..end];
        assert!(block.contains("gap: var(--fandhe-checkbox-group-gap, var(--fandhe-space-1));"));
    }

    #[test]
    fn root_invalid_defines_control_border_color_custom_property() {
        // headless 層は `data-invalid` を出力しないため（モジュール doc
        // 参照）、`attrs` 経由で付与されたときのみ有効になる参照のみを
        // 固定する。
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="checkbox-group"][data-part="root"][data-invalid]"#));
        assert!(css
            .contains("--fandhe-checkbox-group-control-border-color: var(--fandhe-color-danger);"));
    }

    #[test]
    fn item_control_invalid_sets_danger_border_color_after_checked_rule() {
        let css = stylesheet();
        let checked_pos = css
            .find(
                r#"[data-scope="checkbox-group"][data-part="item-control"][data-state="checked"]"#,
            )
            .expect("checked rule missing");
        let invalid_pos = css
            .find(r#"[data-scope="checkbox-group"][data-part="item-control"][data-invalid]"#)
            .expect("invalid rule missing");
        assert!(
            invalid_pos > checked_pos,
            "data-invalid rule must be registered after the checked rule so it wins at equal specificity"
        );
        let end = css[invalid_pos..]
            .find('}')
            .map(|i| invalid_pos + i)
            .expect("closing brace missing");
        let block = &css[invalid_pos..end];
        assert!(block.contains("border-color: var(--fandhe-color-danger);"));
    }

    #[test]
    fn root_disabled_defines_item_custom_properties_without_direct_opacity() {
        // root と item の双方が disabled のとき opacity が多重適用されない
        // 回帰固定（モジュール doc「`root` の `data-disabled` を custom
        // property 経由で `item` へ伝播」節参照）。
        let css = stylesheet();
        let scope = r#"[data-scope="checkbox-group"][data-part="root"][data-disabled] {"#;
        let start = css.find(scope).expect("root[data-disabled] block missing");
        let end = css[start..]
            .find('}')
            .map(|i| start + i)
            .expect("closing brace missing");
        let block = &css[start..end];
        assert!(block.contains("--fandhe-checkbox-group-item-opacity: 0.5;"));
        assert!(block.contains("--fandhe-checkbox-group-item-cursor: not-allowed;"));
        // `--fandhe-checkbox-group-item-opacity: 0.5;` 自体が部分文字列として
        // `opacity: 0.5;` を含むため、単純な `!contains` では検知できない
        // （custom property の間接参照 1 行のみで、直接 `opacity` 宣言
        // （行頭が `opacity:` で始まる行）が無いことを回帰固定する）。
        assert!(!block
            .lines()
            .any(|line| line.trim_start().starts_with("opacity:")));
    }

    #[test]
    fn item_base_uses_fit_content_width_and_disabled_custom_properties() {
        let css = stylesheet();
        let scope = r#"[data-scope="checkbox-group"][data-part="item"] {"#;
        let start = css.find(scope).expect("item base block missing");
        let end = css[start..]
            .find('}')
            .map(|i| start + i)
            .expect("closing brace missing");
        let block = &css[start..end];
        assert!(block.contains("width: fit-content;"));
        assert!(block.contains("opacity: var(--fandhe-checkbox-group-item-opacity, 1);"));
        assert!(block.contains("cursor: var(--fandhe-checkbox-group-item-cursor, pointer);"));
    }

    #[test]
    fn item_control_base_has_transition_tokens_and_invalid_border_reference() {
        // `item-control` の base は `checkbox.rs` と同型に 2 回の `.base()`
        // 純追加で構成され（寸法・hover 面の 1 本目、transition の 2 本目）、
        // `SlotRecipe::css` は同一セレクタでも登録ごとに別ブロックとして
        // 出力する（`checkbox.rs` の `item_control_base_has_transition_tokens`
        // 系テストと同じ「ブロック分割は非スコープ判定」前提）ため、両ブロック
        // を横断する CSS 全体テキストへ対して直接アサーションする。
        let css = stylesheet();
        assert!(css.contains(
            "border: 1px solid var(--fandhe-checkbox-group-control-border-color, var(--fandhe-color-border));"
        ));
        assert!(css.contains("--fandhe-hover-bg: var(--fandhe-color-bg-muted);"));
        assert!(css.contains("transition-property: background, border-color;"));
        assert!(css.contains("transition-duration: var(--fandhe-motion-duration-fast);"));
        assert!(!css.contains("transition: background 0.15s"));
    }

    #[test]
    fn item_control_checked_defines_solid_hover_bg_with_fallback() {
        let css = stylesheet();
        let scope =
            r#"[data-scope="checkbox-group"][data-part="item-control"][data-state="checked"] {"#;
        let start = css.find(scope).expect("checked rule block missing");
        let end = css[start..]
            .find('}')
            .map(|i| start + i)
            .expect("closing brace missing");
        let block = &css[start..end];
        assert!(block.contains(
            "--fandhe-hover-bg: var(--fandhe-palette-emphasized, var(--fandhe-color-accent-emphasized));"
        ));
    }

    #[test]
    fn item_control_hover_uses_hover_bg_indirection_under_media_hover() {
        let css = stylesheet();
        let media_pos = css
            .find("@media (hover: hover) {")
            .expect("hover media query missing");
        let rule = r#"[data-scope="checkbox-group"][data-part="item-control"]:hover:not([data-disabled]) {"#;
        let rule_pos = css.find(rule).expect("item-control hover rule missing");
        assert!(
            rule_pos > media_pos,
            "item-control hover rule must be nested inside the hover media query"
        );
        let end = css[rule_pos..]
            .find('}')
            .map(|i| rule_pos + i)
            .expect("closing brace missing");
        let block = &css[rule_pos..end];
        assert!(block.contains("background: var(--fandhe-hover-bg);"));
    }

    #[test]
    fn indicator_base_has_no_display_declaration() {
        // headless 層の `hidden` 存在属性の意味論を壊さないため
        // （`crate::checkbox` の同名テストと同型の回帰固定）。
        let css = stylesheet();
        let scope = r#"[data-scope="checkbox-group"][data-part="item-indicator"] {"#;
        let start = css.find(scope).expect("item-indicator base block missing");
        let end = css[start..]
            .find('}')
            .map(|i| start + i)
            .expect("closing brace missing");
        let block = &css[start..end];
        assert!(!block.contains("display:"));
    }

    #[test]
    fn root_switches_to_row_layout_on_horizontal_orientation() {
        let css = stylesheet();
        assert!(css.contains(
            r#"[data-scope="checkbox-group"][data-part="root"][data-orientation="horizontal"]"#
        ));
        assert!(css.contains("flex-direction: row;"));
    }

    #[test]
    fn disabled_item_gets_not_allowed_cursor() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="checkbox-group"][data-part="item"][data-disabled]"#));
        assert!(css.contains("cursor: not-allowed;"));
    }

    #[test]
    fn item_focus_within_gets_accent_outline_ring() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="checkbox-group"][data-part="item"]:focus-within {"#));
        assert!(css.contains(
            "outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-palette, var(--fandhe-color-focus-ring, var(--fandhe-color-accent)));"
        ));
        assert!(css.contains("outline-offset: var(--fandhe-focus-ring-offset, 2px);"));
    }

    // --- variant クラス ---

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
        assert!(html.contains(r#"data-scope="checkbox-group""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(html.contains(r#"role="group""#));
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
        assert!(html.contains("fd-checkbox-group--size-md"));
        assert!(html.contains("fd-checkbox-group--color-palette-accent"));
    }

    #[test]
    fn size_enumeration_maps_to_expected_classes() {
        for (size, class) in [
            (Size::Xs, "fd-checkbox-group--size-xs"),
            (Size::Sm, "fd-checkbox-group--size-sm"),
            (Size::Md, "fd-checkbox-group--size-md"),
            (Size::Lg, "fd-checkbox-group--size-lg"),
            (Size::Xl, "fd-checkbox-group--size-xl"),
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
            (
                ColorPalette::Accent,
                "fd-checkbox-group--color-palette-accent",
            ),
            (ColorPalette::Info, "fd-checkbox-group--color-palette-info"),
            (
                ColorPalette::Success,
                "fd-checkbox-group--color-palette-success",
            ),
            (
                ColorPalette::Warning,
                "fd-checkbox-group--color-palette-warning",
            ),
            (
                ColorPalette::Danger,
                "fd-checkbox-group--color-palette-danger",
            ),
            (
                ColorPalette::Neutral,
                "fd-checkbox-group--color-palette-neutral",
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
        assert!(css.contains("--fandhe-checkbox-group-control-size"));
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
        assert!(html.contains(r#"data-scope="checkbox-group""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("attacker"));
    }

    // --- エスケープ回帰 ---

    #[test]
    fn xss_payload_in_item_value_is_escaped_by_render() {
        let payload = "\"><script>alert(1)</script>";
        let html = render(&item(false, false, payload, vec![], vec![text(payload)]));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn xss_payload_in_item_text_children_is_escaped_by_render() {
        let payload = "\"><img src=x onerror=alert(1)>";
        let html = render(&item_text(false, false, vec![], vec![text(payload)]));
        assert!(!html.contains("<img"));
        assert!(html.contains("&lt;img"));
    }

    #[test]
    fn ssr_and_hydration_round_trip_via_reexported_checkbox_group_state_machine() {
        use fandhe_frontend_interactive::{dispatch, render_for_hydration, Hydrate};

        let mut g = CheckboxGroup::default();
        assert_eq!(g.selected(), &[] as &[String]);

        assert!(dispatch(&mut g, "select", "red"));
        assert!(g.is_checked("red"));

        let ssr_html = render(&g.item_control("red", false, vec![], vec![]));
        assert!(ssr_html.contains(r#"data-state="checked""#));

        let hydrate_html = render(&render_for_hydration(&g));
        assert!(hydrate_html.contains("data-hydrate-"));

        let restored = CheckboxGroup::from_hydration_attrs(&g.hydration_attrs()).unwrap();
        assert!(restored.is_checked("red"));
    }

    #[test]
    fn item_indicator_hidden_state_semantics_preserved_through_reexport() {
        let unchecked = render(&item_indicator(false, false, vec![], vec![]));
        assert!(unchecked.contains(r#"hidden="""#));

        let checked = render(&item_indicator(true, false, vec![], vec![]));
        assert!(!checked.contains(r#"hidden="""#));
    }

    #[test]
    fn data_state_constants_reexported() {
        assert_eq!(DATA_STATE_CHECKED, "checked");
        assert_eq!(DATA_STATE_UNCHECKED, "unchecked");
    }
}
