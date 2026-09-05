//! Primitives（`fandhe-frontend-headless-ui`）Overlay / Disclosure 系
//! 10 部品ページの原稿データ（イシュー #1027、親トラッキング #1035
//! Phase 5）。
//!
//! # 役割・呼び出し文脈
//!
//! [`crate::primitive_specs::SPEC_TABLES`] から参照される
//! [`crate::component_page::ComponentPageSpec`] 定数群を保持する専用
//! モジュール。本モジュール自体は生成物へ直接寄与しない
//! （`component_page::render_component_page` が `spec_for` 経由で読み取り、
//! Demo〔[`crate::primitive_showcase::overlay_disclosure`]〕・Anatomy・
//! `data-*` 属性表（いずれも機械導出、Primitives 層は CSS 変数表を持たない）
//! と合成して 6 節ページを組み立てる）。
//!
//! 対象は accordion・collapsible・dialog・drawer・floating-panel・
//! hover-card・popover・toast・toggle-tip・tooltip の 10 部品（トリガー
//! 起点のオーバーレイ、または項目開閉のディスクロージャ系）。
//!
//! # Arguments 抽出規約（`/themes/` 側 #946 規約からの層固有の変更点）
//!
//! `/themes/` 側（[`crate::component_specs_overlay`]）は
//! `fandhe-frontend-pre-styled-ui` の `pub fn root` 等のシグネチャを抽出元と
//! するが、Primitives 層は `fandhe-frontend-headless-ui` の**当該モジュール
//! の全 public パーツ関数の型付き引数**を抽出元とする（`attrs: Vec<(&str,
//! &str)>`/`children: Vec<Node>` は全部品共通の定型引数のため除外する）。
//! 理由: `accordion::root` は `(attrs, children)` のみであり、`/themes/`
//! 側と同じ「root だけ」規約を持ち込むと accordion の `arguments` が
//! 空になってしまう（root ではなく item/item_trigger 等が状態を持つ設計の
//! ため）。状態機械 struct（`Accordion`/`Dialog`/`Toaster` 等）のメソッドは
//! 同名パーツ関数への薄い委譲のため重複計上しない。
//!
//! [`ArgRow`] には part 列が無い（`component_page.rs` の Name/Type/
//! Default/Description 固定）ため、`name` 列へ `<パーツ関数名>: <引数名>`
//! 形式で埋め込む（10 部品で表記を統一する）。`default` 列はソースで
//! `#[derive(Default)]`/`#[default]`/手書き `impl Default` を確認できた
//! 場合のみ記入し、それ以外（位置引数は必須のため既定値が存在しない）は
//! 空文字列のままにする（推測で埋めない）。
//!
//! # `keyboard` を空にする既定と、行を追加してよい基準
//!
//! 本モジュール対象の 10 モジュール自体（`fandhe-frontend-headless-ui`
//! 側）はキーイベントを解釈しない（`decode_action` はいずれも文字列
//! アクション名（`"open"`/`"close"`/`"toggle"` 等）のデコードであり、キー
//! 割り当てではない）ため、`keyboard: &[]` が既定である。
//!
//! `KeyRow` を追加してよいのは、その挙動が `fandhe-frontend-wasm-full`
//! 側で実 DOM 配線として実装されており、行の説明が配線の所在（モジュール・
//! 関数名）を名指しできる場合に限る（`data_display_utilities.rs` の
//! scroll_area/splitter と同じ基準）。実装のどこにも無い挙動を書くと
//! 利用者へ誤った安心を与えるため、この基準を満たさない限り
//! `keyboard: &[]` のまま `aria` 表のみで節を成立させる
//! （`component_page.rs` の Accessibility 節省略規則参照）。dialog は
//! イシュー #1638 で `fandhe-frontend-wasm-full`（`overlay`/`focus_trap`/
//! `headless`）の配線を確認できたため本モジュールで最初に `KeyRow` を
//! 持つ。他 9 部品が空のままなのは各兄弟イシューでの確認待ちであり、
//! 配線が無いと確定した結果ではない。
//!
//! **`collapsible` は #1637 で例外を追加した**: `trigger` はネイティブ
//! `<button type="button">` であり、Space/Enter → click 発火はブラウザ標準
//! 操作として成立し、click から開閉 dispatch（`"toggle"`）への配線は
//! `fandhe-frontend-wasm-full` の `MAPPING_TABLE` が担う。ark-ui/Radix 双方の
//! Keyboard Support 表が Space/Enter を掲げていることを踏まえ、
//! `COLLAPSIBLE.keyboard` のみ 2 行を持つ（他 9 部品の `keyboard: &[]` は
//! 不変）。
//!
//! **`hover_card` は #1641 で 2 例目の例外を追加した**: `trigger`
//! （`crates/headless-ui/src/hover_card.rs:150-162`）はネイティブ `<a>` 要素
//! であり、`href` が `Some` のときブラウザ標準で Tab フォーカス到達・
//! Enter によるリンク遷移が成立する（`fandhe-frontend-wasm-full` 側の
//! hover/focus タイマー配線は未実装だが、この 2 行はネイティブ `<a>` の
//! 挙動そのものであり配線の有無に依存しない）。Radix の「Tab で hover
//! card を開閉」に相当する focus/blur 駆動の開閉配線は未配線のため書か
//! ない（`HOVER_CARD.keyboard` は 2 行、`COLLAPSIBLE.keyboard` と合わせて
//! 他 8 部品の `keyboard: &[]` は不変）。
//!
//! # `hover_card` の Accessibility 節が空にならない理由
//!
//! `hover_card` は `aria-expanded`/`aria-controls`/`aria-haspopup` を
//! **意図的に付与しない**（`crates/headless-ui/src/hover_card.rs` の
//! モジュール doc §「WAI-ARIA と `aria-expanded`/`aria-controls`/
//! `aria-haspopup` を付与しない理由」、`trigger`/`content` の実装）。`aria`
//! を空のままにすると Accessibility 節ごと省略されてしまうため、非付与の
//! 事実そのものと、代わりに `arrow`/`arrow_tip` へ固定付与される
//! `aria-hidden="true"` を行として明示する（`component_specs_nav_data.rs`
//! の `AVATAR` と同型の先例）。
//!
//! # `collapsible` に Themes 版へのリンクを追加しない理由
//!
//! `crates/pre-styled-ui/src/` に `collapsible.rs` は存在せず、
//! `site/themes/collapsible.md` も無い（`/themes/` 側に対応部品が無い）。
//! `site/primitives/collapsible.md` の導入文でも Themes へのリンクを
//! 追加しない（存在しないページへのリンクは `linkcheck` を fail-closed に
//! 壊すため）。
//!
//! # セキュリティ不変条件（REQ-1）
//!
//! 本モジュールはリテラル `&'static str` のみで [`ArgRow`]/[`AriaRow`] を
//! 構築し、`raw_html()` や HTML 文字列の直接組み立て
//! （`format!("<td>{}</td>", …)`）を一切行わない。実際のエスケープは
//! `component_page.rs` 側の `table`/`td`/`text` ノード木経由で `render()`
//! が行う。`examples` のレンダラは `fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui`
//! （イシュー #693 方針、`hui` エイリアス）経由の headless-ui パート関数
//! のみで組み立て、`fandhe_frontend_pre_styled_ui::` の部品関数（styled
//! 層）は一切呼ばない（受け入れ条件 3）。ダミー文字列は無害なもの
//! （`example.com` 等の予約ドメイン、架空の名前）に限る。

use fandhe_frontend_core::{code, div, p, pre, text, Node};
use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui as hui;
use hui::accordion;
use hui::collapsible;
use hui::dialog::{self, ContentIds, DialogRole};
use hui::drawer::{self, DrawerPlacement};
use hui::floating_panel::{self, Stage};
use hui::hover_card::{self, HoverCardDelays};
use hui::popover;
use hui::toast;
use hui::toggle_tip;
use hui::tooltip;
use hui::OpenState;

use crate::component_page::{ArgRow, AriaRow, ComponentPageSpec, ExampleEntry, KeyRow};

/// Examples 用の枠組み（`forms_a.rs::wrap_example`/`forms_c_date_status.rs::wrap_example`
/// と同型。[`crate::primitive_showcase`] のデモ本体と同じ
/// `primitives-demo-frame`/`primitives-demo-note` class のみを使い、
/// `h2`/`h3` は出さない）。イシュー #1641 で「自前 CSS を当てる最小例」を
/// 追加するために本モジュールへ初めて導入した。
fn wrap_example(note: &'static str, body: Vec<Node>) -> Node {
    div(
        vec![],
        vec![
            p(vec![("class", "primitives-demo-note")], vec![text(note)]),
            div(vec![("class", "primitives-demo-frame")], body),
        ],
    )
}

// ---------------------------------------------------------------------
// Accordion（/primitives/accordion/）
// ---------------------------------------------------------------------

/// 一次情報: `crates/headless-ui/src/accordion.rs:1-24`（モジュール doc、
/// single/multiple 2 状態機械・dispatch 契約）、`:45-58`（out-of-scope、
/// orientation/キーボードナビゲーションは未提供）、`:78-160`（`root`/
/// `item`/`item_trigger`/`item_indicator`/`item_content` シグネチャ）、
/// `:481-579`（`aria-expanded`/`aria-controls`/`role="region"` の実出力
/// テスト）。
fn ex_accordion_disabled_item() -> Node {
    let closed = OpenState::Closed;
    accordion::root(
        vec![],
        vec![accordion::item(
            closed,
            true,
            vec![],
            vec![
                accordion::item_trigger(
                    closed,
                    true,
                    "unavailable",
                    Some("acc-ex-trigger"),
                    Some("acc-ex-content"),
                    vec![],
                    vec![
                        text("Unavailable section"),
                        accordion::item_indicator(closed, vec![], vec![text("▾")]),
                    ],
                ),
                accordion::item_content(
                    closed,
                    Some("acc-ex-content"),
                    Some("acc-ex-trigger"),
                    vec![],
                    vec![text("This section is temporarily disabled.")],
                ),
            ],
        )],
    )
}

pub const ACCORDION: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "高々 1 項目が開く single モード（Accordion）と、複数項目が同時に開く multiple モード（MultiAccordion）の 2 状態機械を提供する（dispatch: select/deselect/toggle）。",
        "Root / Item / ItemTrigger / ItemIndicator / ItemContent の 5 anatomy パーツで構成される。",
        "item_trigger は type=\"button\" を固定付与し、フォーム内配置時の意図しない submit を防ぐ。",
        "item_content は labelled_by が Some のときのみ role=\"region\" + aria-labelledby をセットで付与し、名前なし region を作らない。",
        "orientation・キーボードナビゲーションは SSR 静的マークアップに寄与しない CSR 挙動層の責務としてスコープ外（`data-orientation` は attrs 経由で呼び出し側が付与可能）。",
    ],
    arguments: &[
        ArgRow {
            name: "item: state",
            kind: "OpenState",
            default: "Closed",
            description: "項目の開閉状態（data-state へ反映）。",
        },
        ArgRow {
            name: "item: disabled",
            kind: "bool",
            default: "",
            description: "項目の disabled 状態（data-disabled へ反映）。",
        },
        ArgRow {
            name: "item_trigger: state",
            kind: "OpenState",
            default: "Closed",
            description: "開閉状態（aria-expanded/data-state へ反映）。",
        },
        ArgRow {
            name: "item_trigger: disabled",
            kind: "bool",
            default: "",
            description: "disabled 状態（ネイティブ disabled 属性 + data-disabled の両方へ反映）。",
        },
        ArgRow {
            name: "item_trigger: value",
            kind: "&str",
            default: "",
            description: "項目値。data-value へ出力し、wasm-full MAPPING_TABLE の \"toggle\" payload 契約として使う（イシュー #1127）。",
        },
        ArgRow {
            name: "item_trigger: id",
            kind: "Option<&str>",
            default: "",
            description: "trigger 自身の id。",
        },
        ArgRow {
            name: "item_trigger: controls",
            kind: "Option<&str>",
            default: "",
            description: "Some のとき aria-controls で item_content と関連付ける。",
        },
        ArgRow {
            name: "item_indicator: state",
            kind: "OpenState",
            default: "Closed",
            description: "開閉状態のみを data-state へ反映する最小主義な装飾用パーツ。",
        },
        ArgRow {
            name: "item_content: state",
            kind: "OpenState",
            default: "Closed",
            description: "開閉状態（closed のとき hidden 存在属性を付与）。",
        },
        ArgRow {
            name: "item_content: id",
            kind: "Option<&str>",
            default: "",
            description: "item_trigger の controls と対で aria-controls 関連付けを成立させる。",
        },
        ArgRow {
            name: "item_content: labelled_by",
            kind: "Option<&str>",
            default: "",
            description: "Some のときのみ role=\"region\" + aria-labelledby をセットで付与する（名前なし region を作らないため）。",
        },
    ],
    examples: &[ExampleEntry {
        title: "Disabled item",
        description: "disabled な項目は item_trigger にネイティブ disabled 属性と data-disabled が付与され、フォーカス・展開ができなくなります。",
        render: ex_accordion_disabled_item,
    }],
    keyboard: &[],
    aria: &[
        AriaRow {
            attribute: "aria-expanded",
            description: "item_trigger に付与。項目の開閉状態（open で true）を表す。",
        },
        AriaRow {
            attribute: "aria-controls",
            description: "item_trigger に付与。controls が Some のとき対応する item_content の id を指す。",
        },
        AriaRow {
            attribute: "role=\"region\"",
            description: "item_content に付与。labelled_by が Some のときのみ aria-labelledby とセットで付与される。",
        },
    ],
    demo: None,
};

// ---------------------------------------------------------------------
// Collapsible（/primitives/collapsible/）
// ---------------------------------------------------------------------

/// 一次情報: `crates/headless-ui/src/collapsible.rs:1-29`（モジュール doc、
/// Disclosure を埋め込んだ開閉状態機械）、`:47-123`（`root`/`trigger`/
/// `indicator`/`content` シグネチャ）、`:276-355`（`aria-expanded`/
/// `aria-controls` の実出力テスト）。
fn ex_collapsible_disabled() -> Node {
    let state = OpenState::Closed;
    collapsible::root(
        state,
        true,
        vec![],
        vec![
            collapsible::trigger(
                state,
                true,
                Some("collapsible-ex-content"),
                vec![],
                vec![
                    text("Locked section"),
                    collapsible::indicator(state, true, vec![], vec![text("▾")]),
                ],
            ),
            collapsible::content(
                state,
                true,
                Some("collapsible-ex-content"),
                vec![],
                vec![text("Unlock to view this content.")],
            ),
        ],
    )
}

pub const COLLAPSIBLE: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "Disclosure を埋め込んだ単純な開閉状態機械 Collapsible を提供する（dispatch: open/close/toggle）。",
        "Root / Trigger / Indicator / Content の 4 anatomy パーツで構成される。",
        "trigger は type=\"button\" を固定付与し、フォーム内配置時の意図しない submit を防ぐ。",
        "content は closed のとき hidden 存在属性を付与し、JS なしの SSR でも閉状態を表現する。",
        "content / indicator は disabled 状態を data-disabled へ反映する（#1637。ネイティブ disabled 存在属性は div/span に無効なため付与しない）。",
        "Space/Enter はネイティブ button 経由（wasm-full の MAPPING_TABLE がクリックを toggle dispatch へ配線する、#1637）。",
    ],
    arguments: &[
        ArgRow {
            name: "root: state",
            kind: "OpenState",
            default: "Closed",
            description: "パネル全体の開閉状態（data-state へ反映）。",
        },
        ArgRow {
            name: "root: disabled",
            kind: "bool",
            default: "",
            description: "disabled 状態（data-disabled へ反映）。",
        },
        ArgRow {
            name: "trigger: state",
            kind: "OpenState",
            default: "Closed",
            description: "開閉状態（aria-expanded/data-state へ反映）。",
        },
        ArgRow {
            name: "trigger: disabled",
            kind: "bool",
            default: "",
            description: "disabled 状態（ネイティブ disabled 属性 + data-disabled の両方へ反映）。",
        },
        ArgRow {
            name: "trigger: controls",
            kind: "Option<&str>",
            default: "",
            description: "Some のとき aria-controls で content と関連付ける。",
        },
        ArgRow {
            name: "indicator: state",
            kind: "OpenState",
            default: "Closed",
            description: "開閉状態を data-state へ反映する装飾用パーツ。",
        },
        ArgRow {
            name: "indicator: disabled",
            kind: "bool",
            default: "",
            description: "disabled 状態（data-disabled へ反映。#1637、ark-ui Indicator 準拠）。",
        },
        ArgRow {
            name: "content: state",
            kind: "OpenState",
            default: "Closed",
            description: "開閉状態（closed のとき hidden 存在属性を付与）。",
        },
        ArgRow {
            name: "content: disabled",
            kind: "bool",
            default: "",
            description: "disabled 状態（data-disabled へ反映。#1637、ark-ui/Radix Content 準拠）。",
        },
        ArgRow {
            name: "content: id",
            kind: "Option<&str>",
            default: "",
            description: "trigger の controls と対で aria-controls 関連付けを成立させる。",
        },
    ],
    examples: &[ExampleEntry {
        title: "Disabled trigger",
        description: "disabled な trigger はネイティブ disabled 属性と data-disabled が付与され、フォーカス・展開ができなくなります。",
        render: ex_collapsible_disabled,
    }],
    keyboard: &[
        KeyRow {
            key: "Space / Enter",
            description: "trigger はネイティブ <button type=\"button\"> のため、フォーカス時の Space/Enter によるクリック相当の発火はブラウザ標準操作として成立する。クリックから開閉切替への dispatch 配線（\"toggle\"）は fandhe-frontend-wasm-full の MAPPING_TABLE の責務（#1637）。",
        },
        KeyRow {
            key: "Tab",
            description: "trigger のみがタブ順に含まれる。closed の content は hidden 存在属性によりタブ順・支援技術双方から除外される。",
        },
    ],
    aria: &[
        AriaRow {
            attribute: "aria-expanded",
            description: "trigger に付与。開閉状態（open で true）を表す。",
        },
        AriaRow {
            attribute: "aria-controls",
            description: "trigger に付与。controls が Some のとき対応する content の id を指す。",
        },
        AriaRow {
            attribute: "hidden",
            description: "content に付与（closed のとき）。支援技術・タブ順の双方から除外する。",
        },
    ],
    demo: None,
};

// ---------------------------------------------------------------------
// Dialog（/primitives/dialog/）
// ---------------------------------------------------------------------

/// 一次情報: `crates/headless-ui/src/dialog.rs:1-63`（モジュール doc、
/// Disclosure を埋め込んだ開閉状態機械）、`:18-63`（スコープ外・参考サイトとの
/// 意図的な差分。Escape・外側クリック閉鎖・フォーカストラップは
/// `fandhe-frontend-wasm-full` が担う、イシュー #1638）、`:118-284`
/// （`root`/`trigger`/`backdrop`/`positioner`/`content`/`ContentIds`/
/// `title`/`description`/`close_trigger` シグネチャ、`content` の
/// `tabindex="-1"` 固定付与）、`role`/`aria-modal`/`aria-haspopup`/
/// `aria-hidden`/`tabindex` の実出力テスト。
fn ex_dialog_alert_non_modal() -> Node {
    let state = OpenState::Open;
    div(
        vec![],
        vec![
            dialog::trigger(
                state,
                Some("dialog-ex-content"),
                vec![],
                vec![text("Delete item")],
            ),
            dialog::root(
                state,
                vec![],
                vec![
                    dialog::backdrop(state, vec![], vec![]),
                    dialog::positioner(
                        state,
                        vec![],
                        vec![dialog::content(
                            state,
                            DialogRole::Alertdialog,
                            false,
                            ContentIds {
                                id: Some("dialog-ex-content"),
                                labelledby: Some("dialog-ex-title"),
                                describedby: None,
                            },
                            vec![],
                            vec![
                                dialog::title(
                                    Some("dialog-ex-title"),
                                    vec![],
                                    vec![text("Delete this item?")],
                                ),
                                // codex-review 指摘（PR #1795）: dialog の
                                // close-trigger はアイコン専用契約（0.59.0〜、
                                // `crates/pre-styled-ui/src/dialog.rs` rustdoc
                                // 参照）。支援技術向けラベルは aria-label で維持する。
                                dialog::close_trigger(
                                    vec![("aria-label", "Cancel")],
                                    vec![text("×")],
                                ),
                            ],
                        )],
                    ),
                ],
            ),
        ],
    )
}

pub const DIALOG: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "Root / Trigger / Backdrop / Positioner / Content / Title / Description / CloseTrigger の 8 anatomy パーツと、Disclosure を埋め込んだ開閉状態機械 Dialog を提供する（dispatch: open/close/toggle）。",
        "trigger は type=\"button\" を固定付与し、aria-haspopup=\"dialog\" を常に付与する。",
        "content は DialogRole（Dialog/Alertdialog）で role を切り替えられ、modal 引数で aria-modal の値を制御できる。content は tabindex=\"-1\" を固定付与する（zag dialog.connect.ts と同じく、プログラム的フォーカスのみを許可する前提。イシュー #1638）。",
        "backdrop/positioner は closed のとき hidden 存在属性を付与し、JS なしの SSR でも閉状態を表現する。",
        "Escape キー閉鎖・外側クリック閉鎖・フォーカストラップ・閉鎖時の trigger へのフォーカス復帰・click → dispatch 配線は本モジュールが属性を出力するのみで、実 DOM 配線は fandhe-frontend-wasm-full（overlay/focus_trap/headless の part → action 対応表）が担う。content の attrs 経由で data-close-on-escape=\"false\" / data-close-on-interact-outside=\"false\"（\"false\" リテラルのときのみ無効化）を渡せる。初期フォーカス先を指定する data-autofocus は content ではなく、content 配下の tabbable な対象の子要素へ付与する（fandhe-frontend-wasm-full の focus_trap::collect_tabbable が content の子孫のみを候補として収集するため）。",
    ],
    arguments: &[
        ArgRow {
            name: "root: state",
            kind: "OpenState",
            default: "Closed",
            description: "ダイアログ全体の開閉状態（data-state へ反映）。",
        },
        ArgRow {
            name: "trigger: state",
            kind: "OpenState",
            default: "Closed",
            description: "aria-expanded/data-state へ反映される開閉状態。",
        },
        ArgRow {
            name: "trigger: controls",
            kind: "Option<&str>",
            default: "",
            description: "Some のとき aria-controls で content と関連付ける。",
        },
        ArgRow {
            name: "backdrop: state",
            kind: "OpenState",
            default: "Closed",
            description: "closed のとき hidden 存在属性を付与する背面レイヤーの開閉状態。",
        },
        ArgRow {
            name: "positioner: state",
            kind: "OpenState",
            default: "Closed",
            description: "closed のとき hidden 存在属性を付与する配置ラッパーの開閉状態。",
        },
        ArgRow {
            name: "content: state",
            kind: "OpenState",
            default: "Closed",
            description: "開閉状態（closed のとき hidden 存在属性を付与）。",
        },
        ArgRow {
            name: "content: role_kind",
            kind: "DialogRole",
            default: "",
            description: "role 属性の値（Dialog=\"dialog\" / Alertdialog=\"alertdialog\"）。",
        },
        ArgRow {
            name: "content: modal",
            kind: "bool",
            default: "",
            description: "aria-modal へ反映する値。",
        },
        ArgRow {
            name: "content: ids",
            kind: "ContentIds",
            default: "ContentIds::default()",
            description: "content 自身の id・aria-labelledby・aria-describedby をまとめた構造体（各フィールドが Some のときのみ対応する属性を出力）。",
        },
        ArgRow {
            name: "title: id",
            kind: "Option<&str>",
            default: "",
            description: "content の labelledby と対で aria-labelledby 関連付けを成立させる。",
        },
        ArgRow {
            name: "description: id",
            kind: "Option<&str>",
            default: "",
            description: "content の describedby と対で aria-describedby 関連付けを成立させる。",
        },
    ],
    examples: &[ExampleEntry {
        title: "Alert dialog (non-modal)",
        description: "DialogRole::Alertdialog + modal=false の組み合わせで、role=\"alertdialog\" かつ aria-modal=\"false\" を出力する例です。",
        render: ex_dialog_alert_non_modal,
    }],
    keyboard: &[
        KeyRow {
            key: "Enter / Space (trigger)",
            description: "ネイティブ button 要素の click 相当が成立し、fandhe-frontend-wasm-full の headless part → action 対応表が \"toggle\" を dispatch する。",
        },
        KeyRow {
            key: "Enter / Space (close-trigger)",
            description: "同対応表が \"close\" を dispatch する。",
        },
        KeyRow {
            key: "Escape",
            description: "overlay::close_on_escape_for が \"close\" を通知する（role=\"alertdialog\" でも Escape は閉じる。外側クリックのみ alertdialog は既定で無効）。data-close-on-escape=\"false\" で無効化できる。",
        },
        KeyRow {
            key: "Tab / Shift+Tab",
            description: "aria-modal=\"true\" のとき focus_trap::should_trap が content 内でフォーカスを循環させる。data-autofocus で初期フォーカス先を指定でき、tabbable な子が無い場合は content 自身（tabindex=\"-1\"）へフォーカスする。",
        },
        KeyRow {
            key: "(閉鎖時)",
            description: "focus_trap::push_trap が push 時点でフォーカスされていた要素（取得不能なら trigger 引数）をスナップショットしており、pop_trap がその要素へフォーカスを復帰する。",
        },
    ],
    aria: &[
        AriaRow {
            attribute: "aria-haspopup=\"dialog\"",
            description: "trigger に固定付与。",
        },
        AriaRow {
            attribute: "aria-expanded",
            description: "trigger に付与。開閉状態を反映する。",
        },
        AriaRow {
            attribute: "aria-controls",
            description: "trigger に付与。controls が Some のとき content と関連付ける。",
        },
        AriaRow {
            attribute: "aria-hidden=\"true\"",
            description: "backdrop に固定付与（装飾層のため読み上げ対象外）。",
        },
        AriaRow {
            attribute: "role=\"dialog\" / role=\"alertdialog\"",
            description: "content に付与。role_kind の値を反映する。",
        },
        AriaRow {
            attribute: "aria-modal",
            description: "content に付与。modal 引数の値を反映する。",
        },
        AriaRow {
            attribute: "aria-labelledby / aria-describedby",
            description: "content に付与。ids.labelledby/ids.describedby が Some のときのみ出力される。",
        },
        AriaRow {
            attribute: "tabindex=\"-1\"",
            description: "content に固定付与（zag dialog.connect.ts の getContentProps と同じく、プログラム的フォーカスのみを許可する）。",
        },
    ],
    demo: None,
};

// ---------------------------------------------------------------------
// Drawer（/primitives/drawer/）
// ---------------------------------------------------------------------

/// 一次情報: `crates/headless-ui/src/drawer.rs:1-97`（モジュール doc、
/// dialog パターンの変種として `crate::dialog` の状態機械をそのまま再利用。
/// イシュー #1639 で `fandhe-frontend-wasm-full` は drawer scope を
/// `MAPPING_TABLE`/`OverlayKind::from_scope`/`focus_trap::should_trap` の
/// いずれにも配線しておらず、ハイドレーション後の Drawer が inert である
/// ことを確認・記述した）、`:127-146`（`DrawerPlacement` 4 値・既定
/// `End`）、`:181-327`（`root`/`trigger`/`backdrop`/`positioner`/
/// `content`/`title`/`description`/`close_trigger` シグネチャ。`content`
/// は #1639 で `tabindex="-1"` を固定付与）。
fn ex_drawer_start_placement() -> Node {
    let state = OpenState::Open;
    let placement = DrawerPlacement::Start;
    div(
        vec![],
        vec![
            drawer::trigger(
                state,
                Some("drawer-ex-content"),
                vec![],
                vec![text("Open navigation")],
            ),
            drawer::root(
                state,
                placement,
                vec![],
                vec![
                    drawer::backdrop(state, vec![], vec![]),
                    drawer::positioner(
                        state,
                        placement,
                        vec![],
                        vec![drawer::content(
                            state,
                            placement,
                            true,
                            ContentIds {
                                id: Some("drawer-ex-content"),
                                labelledby: Some("drawer-ex-title"),
                                describedby: None,
                            },
                            vec![],
                            vec![
                                drawer::title(
                                    Some("drawer-ex-title"),
                                    vec![],
                                    vec![text("Navigation")],
                                ),
                                // codex-review 指摘（PR #1795）と同型（イシュー
                                // #1695）: drawer の close-trigger はアイコン
                                // 専用契約（0.6x〜、
                                // `crates/pre-styled-ui/src/drawer.rs` rustdoc
                                // 参照）。支援技術向けラベルは aria-label で
                                // 維持する。
                                drawer::close_trigger(
                                    vec![("aria-label", "Close")],
                                    vec![text("×")],
                                ),
                            ],
                        )],
                    ),
                ],
            ),
        ],
    )
}

pub const DRAWER: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "dialog パターンの変種として crate::dialog の Disclosure 状態機械をそのまま再利用する（新規状態機械を作らない）。",
        "DrawerPlacement（Start/End/Top/Bottom、既定 End）で画面のどの端から出現するかを切り替え、root/positioner/content の data-placement へ反映する。",
        "content は role=\"dialog\" を固定付与する（Alertdialog 相当の variant は提供しない。常設ナビ・フィルタ等の補助パネル用途に一致させるため）。",
        "content は tabindex=\"-1\" を固定付与する（zag drawer.connect.ts/WAI-ARIA dialog パターンの前提。イシュー #1639）。",
        "fandhe-frontend-wasm-full は drawer scope を一切配線していない（MAPPING_TABLE に scope=\"drawer\" の行がなく trigger/close-trigger の click が dispatch されない、OverlayKind::from_scope は \"drawer\" を拒否、focus_trap::should_trap も data-scope=\"dialog\" のみ対象）。ハイドレーション後の Drawer は現状 inert であり、Escape/外側クリック閉鎖・フォーカストラップのいずれも未対応（別イシューで追跡、fail-closed のため未対応でも安全側。イシュー #1639 で判明・記述是正）。",
        "grabber/swipe-area 等のドラッグ操作パーツ・data-swipe-direction 等のスワイプ状態語彙は意図的に非採用（ドラッグ操作という JS ランタイムの実行時計測関心のため。イシュー #1639）。",
    ],
    arguments: &[
        ArgRow {
            name: "root: state",
            kind: "OpenState",
            default: "Closed",
            description: "ドロワー全体の開閉状態（data-state へ反映）。",
        },
        ArgRow {
            name: "root: placement",
            kind: "DrawerPlacement",
            default: "End",
            description: "画面のどの端から出現するか（root/positioner/content の data-placement へ反映）。",
        },
        ArgRow {
            name: "trigger: state",
            kind: "OpenState",
            default: "Closed",
            description: "aria-expanded/data-state へ反映される開閉状態。",
        },
        ArgRow {
            name: "trigger: controls",
            kind: "Option<&str>",
            default: "",
            description: "Some のとき aria-controls で content と関連付ける。",
        },
        ArgRow {
            name: "backdrop: state",
            kind: "OpenState",
            default: "Closed",
            description: "closed のとき hidden 存在属性を付与する背面レイヤーの開閉状態。",
        },
        ArgRow {
            name: "positioner: state",
            kind: "OpenState",
            default: "Closed",
            description: "closed のとき hidden 存在属性を付与する配置ラッパーの開閉状態。",
        },
        ArgRow {
            name: "positioner: placement",
            kind: "DrawerPlacement",
            default: "End",
            description: "data-placement へ反映される配置（styled 層が方向別レイアウトを切り替える起点）。",
        },
        ArgRow {
            name: "content: state",
            kind: "OpenState",
            default: "Closed",
            description: "開閉状態（closed のとき hidden 存在属性を付与）。",
        },
        ArgRow {
            name: "content: placement",
            kind: "DrawerPlacement",
            default: "End",
            description: "data-placement へ反映される配置。",
        },
        ArgRow {
            name: "content: modal",
            kind: "bool",
            default: "",
            description: "aria-modal へ反映する値。",
        },
        ArgRow {
            name: "content: ids",
            kind: "ContentIds",
            default: "ContentIds::default()",
            description: "content 自身の id・aria-labelledby・aria-describedby をまとめた構造体（crate::dialog の型を再利用）。",
        },
        ArgRow {
            name: "title: id",
            kind: "Option<&str>",
            default: "",
            description: "content の labelledby と対で aria-labelledby 関連付けを成立させる。",
        },
        ArgRow {
            name: "description: id",
            kind: "Option<&str>",
            default: "",
            description: "content の describedby と対で aria-describedby 関連付けを成立させる。",
        },
    ],
    examples: &[ExampleEntry {
        title: "Start placement",
        description: "DrawerPlacement::Start（既定 End とは逆側）から出現するナビゲーション用ドロワーの例です。",
        render: ex_drawer_start_placement,
    }],
    keyboard: &[],
    aria: &[
        AriaRow {
            attribute: "aria-haspopup=\"dialog\"",
            description: "trigger に固定付与。",
        },
        AriaRow {
            attribute: "aria-expanded",
            description: "trigger に付与。開閉状態を反映する。",
        },
        AriaRow {
            attribute: "aria-controls",
            description: "trigger に付与。controls が Some のとき content と関連付ける。",
        },
        AriaRow {
            attribute: "aria-hidden=\"true\"",
            description: "backdrop に固定付与（装飾層のため読み上げ対象外）。",
        },
        AriaRow {
            attribute: "role=\"dialog\"",
            description: "content に固定付与（Alertdialog 相当の variant は提供しない）。",
        },
        AriaRow {
            attribute: "aria-modal",
            description: "content に付与。modal 引数の値を反映する。",
        },
        AriaRow {
            attribute: "aria-labelledby / aria-describedby",
            description: "content に付与。ids.labelledby/ids.describedby が Some のときのみ出力される。",
        },
        AriaRow {
            attribute: "tabindex=\"-1\"",
            description: "content に固定付与。プログラム的フォーカスのみを許可する WAI-ARIA dialog パターンの前提（イシュー #1639）。",
        },
    ],
    demo: None,
};

// ---------------------------------------------------------------------
// Floating Panel（/primitives/floating-panel/）
// ---------------------------------------------------------------------

/// 一次情報: `crates/headless-ui/src/floating_panel.rs:1-119`（モジュール
/// doc、`Stage` を独自 enum とする理由・content は role="dialog" 固定だが
/// aria-modal を出力しない非モーダル overlay という不変条件・イシュー
/// #1640 の参考サイト突合結果）、`:143-151`（`Stage` 3 値・既定 Default）、
/// `:193-390`（`root`/`trigger`/`positioner`/`content`/`header`/`title`/
/// `control`/`stage_trigger`/`close_trigger`/`body` シグネチャ。`header`/
/// `control` は #1640 で `data-stage` を追加、`body` は同じく #1640 で
/// `Stage::Minimized` 時の `hidden` を追加）。
fn ex_floating_panel_maximized() -> Node {
    let state = OpenState::Open;
    let stage = Stage::Maximized;
    div(
        vec![],
        vec![
            floating_panel::trigger(
                state,
                false,
                Some("fp-ex-content"),
                vec![],
                vec![text("Open (maximized)")],
            ),
            floating_panel::root(
                state,
                stage,
                vec![],
                vec![floating_panel::positioner(
                    state,
                    stage,
                    vec![],
                    vec![floating_panel::content(
                        state,
                        stage,
                        Some("fp-ex-content"),
                        Some("fp-ex-title"),
                        vec![],
                        vec![
                            floating_panel::header(
                                stage,
                                vec![],
                                vec![
                                    floating_panel::title(
                                        Some("fp-ex-title"),
                                        vec![],
                                        vec![text("Editor")],
                                    ),
                                    floating_panel::control(
                                        stage,
                                        vec![],
                                        vec![
                                            floating_panel::stage_trigger(
                                                Stage::Default,
                                                vec![("aria-label", "Restore")],
                                                vec![text("Restore")],
                                            ),
                                            floating_panel::close_trigger(
                                                vec![("aria-label", "Close Window")],
                                                vec![text("×")],
                                            ),
                                        ],
                                    ),
                                ],
                            ),
                            floating_panel::body(
                                stage,
                                vec![],
                                vec![text("Maximized panel body.")],
                            ),
                        ],
                    )],
                )],
            ),
        ],
    )
}

/// 一次情報: `crates/headless-ui/src/floating_panel.rs`（`body` 関数、
/// [`Stage::Minimized`] のとき `hidden` 存在属性を付与する不変条件、
/// イシュー #1640 で zag `getBodyProps` の `hidden: isMinimized` との
/// 突合是正）。
fn ex_floating_panel_minimized() -> Node {
    let state = OpenState::Open;
    let stage = Stage::Minimized;
    div(
        vec![],
        vec![
            floating_panel::trigger(
                state,
                false,
                Some("fp-min-content"),
                vec![],
                vec![text("Open (minimized)")],
            ),
            floating_panel::root(
                state,
                stage,
                vec![],
                vec![floating_panel::positioner(
                    state,
                    stage,
                    vec![],
                    vec![floating_panel::content(
                        state,
                        stage,
                        Some("fp-min-content"),
                        Some("fp-min-title"),
                        vec![],
                        vec![
                            floating_panel::header(
                                stage,
                                vec![],
                                vec![
                                    floating_panel::title(
                                        Some("fp-min-title"),
                                        vec![],
                                        vec![text("Notes")],
                                    ),
                                    floating_panel::control(
                                        stage,
                                        vec![],
                                        vec![
                                            floating_panel::stage_trigger(
                                                Stage::Default,
                                                vec![("aria-label", "Restore")],
                                                vec![text("Restore")],
                                            ),
                                            floating_panel::close_trigger(
                                                vec![("aria-label", "Close Window")],
                                                vec![text("×")],
                                            ),
                                        ],
                                    ),
                                ],
                            ),
                            // Stage::Minimized のため body へ hidden 存在属性が
                            // 付与される（headless 層の不変条件。イシュー
                            // #1640 参照）。
                            floating_panel::body(
                                stage,
                                vec![],
                                vec![text("This body is hidden while minimized.")],
                            ),
                        ],
                    )],
                )],
            ),
        ],
    )
}

pub const FLOATING_PANEL: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "Root / Trigger / Positioner / Content / Header / Title / Control / StageTrigger / CloseTrigger / Body の 10 anatomy パーツと、開閉・stage（Default/Minimized/Maximized）・座標を持つ状態機械 FloatingPanel を提供する。",
        "stage（Default/Minimized/Maximized）は crate::state::Disclosure/SingleSelect に写像できないため、Steps/Progress と同じ判断で独自 enum として実装する。",
        "content は role=\"dialog\" を固定付与するが、非モーダル overlay のため aria-modal は出力しない（ユーザーは他の要素を操作し続けられる）。",
        "座標（--fandhe-x/--fandhe-y）は crate::positioning の CSS 変数語彙を再利用するが、実際のドラッグ・リサイズ操作・trigger/close-trigger/stage-trigger の click・Escape キー閉鎖・矢印キー移動はいずれも fandhe-frontend-wasm-full に未配線（イシュー #1640 時点の事実）。headless 層は型付きアクション（open/close/toggle/minimize/maximize/restore/set_position）のみを提供する。",
    ],
    arguments: &[
        ArgRow {
            name: "root: state",
            kind: "OpenState",
            default: "Closed",
            description: "パネル全体の開閉状態（data-state へ反映）。",
        },
        ArgRow {
            name: "root: stage",
            kind: "Stage",
            default: "Default",
            description: "パネルの表示段階（Default/Minimized/Maximized、data-stage へ反映）。",
        },
        ArgRow {
            name: "trigger: state",
            kind: "OpenState",
            default: "Closed",
            description: "aria-expanded/data-state へ反映される開閉状態。",
        },
        ArgRow {
            name: "trigger: disabled",
            kind: "bool",
            default: "",
            description: "ネイティブ disabled 存在属性のみで表現する disabled 状態。",
        },
        ArgRow {
            name: "trigger: controls",
            kind: "Option<&str>",
            default: "",
            description: "Some のとき aria-controls で content と関連付ける。",
        },
        ArgRow {
            name: "positioner: state",
            kind: "OpenState",
            default: "Closed",
            description: "closed のとき hidden 存在属性を付与する配置ラッパーの開閉状態。",
        },
        ArgRow {
            name: "positioner: stage",
            kind: "Stage",
            default: "Default",
            description: "data-stage へ反映される表示段階。",
        },
        ArgRow {
            name: "content: state",
            kind: "OpenState",
            default: "Closed",
            description: "開閉状態（closed のとき hidden 存在属性を付与）。",
        },
        ArgRow {
            name: "content: stage",
            kind: "Stage",
            default: "Default",
            description: "data-stage へ反映される表示段階。",
        },
        ArgRow {
            name: "content: id",
            kind: "Option<&str>",
            default: "",
            description: "trigger の controls と対で aria-controls 関連付けを成立させる。",
        },
        ArgRow {
            name: "content: labelledby",
            kind: "Option<&str>",
            default: "",
            description: "Some のとき title の id と対で aria-labelledby 関連付けを成立させる。",
        },
        ArgRow {
            name: "title: id",
            kind: "Option<&str>",
            default: "",
            description: "content の labelledby と対にする。",
        },
        ArgRow {
            name: "header: stage",
            kind: "Stage",
            default: "Default",
            description: "data-stage へ反映される表示段階（イシュー #1640。zag getHeaderProps との突合是正）。",
        },
        ArgRow {
            name: "control: stage",
            kind: "Stage",
            default: "Default",
            description: "data-stage へ反映される表示段階（イシュー #1640。zag getControlProps との突合是正。styled 層は data-stage 経由で stage-trigger の表示切替を実装できる）。",
        },
        ArgRow {
            name: "stage_trigger: target",
            kind: "Stage",
            default: "",
            description: "遷移先の表示段階（data-stage へ反映。実際の dispatch 配線は呼び出し側/wasm 層の責務）。",
        },
        ArgRow {
            name: "body: stage",
            kind: "Stage",
            default: "Default",
            description: "data-stage へ反映される表示段階。Stage::Minimized のとき hidden 存在属性も付与する（イシュー #1640。zag getBodyProps の hidden: isMinimized との突合是正）。styled 層の data-stage=\"minimized\" 折り畳みは headless 層の hidden と二重化するが無害（多層防御）。",
        },
    ],
    examples: &[
        ExampleEntry {
            title: "Maximized stage",
            description: "Stage::Maximized（既定 Default とは異なる表示段階）で開いた FloatingPanel の例です。",
            render: ex_floating_panel_maximized,
        },
        ExampleEntry {
            title: "Minimized stage",
            description: "Stage::Minimized で開いた FloatingPanel の例です。body へ hidden 存在属性が付与され本文が隠れます（header・control は表示されたままです）。",
            render: ex_floating_panel_minimized,
        },
    ],
    keyboard: &[],
    aria: &[
        AriaRow {
            attribute: "aria-haspopup=\"dialog\"",
            description: "trigger に固定付与。",
        },
        AriaRow {
            attribute: "aria-expanded",
            description: "trigger に付与。開閉状態を反映する。",
        },
        AriaRow {
            attribute: "aria-controls",
            description: "trigger に付与。controls が Some のとき content と関連付ける。",
        },
        AriaRow {
            attribute: "role=\"dialog\"",
            description: "content に固定付与。",
        },
        AriaRow {
            attribute: "aria-labelledby",
            description: "content に付与。labelledby が Some のときのみ出力される。",
        },
        AriaRow {
            attribute: "aria-modal（非付与）",
            description: "content は role=\"dialog\" を固定付与するが aria-modal は出力しない（非モーダル overlay。ユーザーは他の要素を操作し続けられ、支援技術へ誤ったモーダル通知を送らない）。",
        },
        AriaRow {
            attribute: "hidden",
            description: "positioner/content は closed のとき、body は Stage::Minimized のとき hidden 存在属性を付与する（イシュー #1640。body は zag getBodyProps との突合是正）。",
        },
    ],
    demo: None,
};

// ---------------------------------------------------------------------
// Hover Card（/primitives/hover-card/）
// ---------------------------------------------------------------------

/// 一次情報: `crates/headless-ui/src/hover_card.rs:1-40`（モジュール doc、
/// `aria-expanded`/`aria-controls`/`aria-haspopup` を付与しない理由、
/// `HoverCardDelays` の遅延設定値）、`:102-114`（`HoverCardDelays` フィール
/// ド・既定 `open_ms: 600`/`close_ms: 300`）、`:123-236`（`root`/
/// `trigger`/`positioner`/`content`/`arrow`/`arrow_tip` シグネチャ、
/// `aria-hidden="true"` の固定付与）。
fn ex_hover_card_custom_delays() -> Node {
    let state = OpenState::Open;
    let delays = HoverCardDelays {
        open_ms: 100,
        close_ms: 800,
    };
    div(
        vec![],
        vec![hover_card::root(
            state,
            delays,
            vec![],
            vec![
                hover_card::trigger(
                    state,
                    Some("https://example.com/team"),
                    vec![],
                    vec![text("@team")],
                ),
                hover_card::positioner(
                    state,
                    vec![],
                    vec![hover_card::content(
                        state,
                        Some("hc-ex-content"),
                        vec![],
                        vec![
                            hover_card::arrow(vec![], vec![hover_card::arrow_tip(vec![], vec![])]),
                            text("Opens fast, lingers longer before closing."),
                        ],
                    )],
                ),
            ],
        )],
    )
}

/// 自前 CSS の最小例（イシュー #1641、`CHECKBOX_CUSTOM_CSS_SNIPPET`/
/// `ex_checkbox_custom_css`〔forms_a.rs、#1602〕と同型のパターン）。CSS は
/// テキストノード（[`code`]/[`pre`]）として既定エスケープを経由し、
/// `crate::primitive_showcase` の専用スタイルシート（`[data-scope=`/
/// `[data-part=` を持たない契約、`tests/site_css_contract.rs`）へは
/// 追加しない。
const HOVER_CARD_CUSTOM_CSS_SNIPPET: &str = "\
[data-scope=\"hover-card\"][data-part=\"content\"] {\n  \
  border: 1px solid #888;\n  border-radius: 8px;\n  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);\n\
}\n\
[data-scope=\"hover-card\"][data-part=\"content\"][data-state=\"closed\"] {\n  \
  display: none;\n\
}\n\
[data-scope=\"hover-card\"][data-part=\"trigger\"]:focus-visible {\n  \
  outline: 2px solid #2563eb;\n  outline-offset: 2px;\n\
}\n\
[data-scope=\"hover-card\"][data-part=\"positioner\"][data-side=\"top\"] {\n  \
  margin-bottom: 8px;\n\
}\n";

/// 一次情報: `crates/headless-ui/src/hover_card.rs:1-40`（モジュール doc、
/// `aria-expanded`/`aria-controls`/`aria-haspopup` を付与しない理由、
/// `HoverCardDelays` の遅延設定値）、`:123-236`（`root`/`trigger`/
/// `positioner`/`content`/`arrow`/`arrow_tip` シグネチャ）。イシュー #1641
/// の参照突合結果を踏まえ、利用者が `data-scope`/`data-part`/`data-state`/
/// `data-side` セレクタで自前 CSS を当てる最小例を示す。
fn ex_hover_card_custom_css() -> Node {
    let state = OpenState::Open;
    let markup = hover_card::root(
        state,
        HoverCardDelays::default(),
        vec![],
        vec![
            hover_card::trigger(
                state,
                Some("https://example.com/profile"),
                vec![],
                vec![text("@example")],
            ),
            hover_card::positioner(
                state,
                vec![("data-side", "top")],
                vec![hover_card::content(
                    state,
                    None,
                    vec![],
                    vec![text("Custom-styled preview card.")],
                )],
            ),
        ],
    );
    wrap_example(
        "利用者が data-scope / data-part / data-state / data-side セレクタで自前 CSS を当てる最小例です。headless-ui 自体はスタイルを持ちません。",
        vec![
            markup,
            pre(
                vec![],
                vec![code(vec![], vec![text(HOVER_CARD_CUSTOM_CSS_SNIPPET)])],
            ),
        ],
    )
}

pub const HOVER_CARD: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "Root / Trigger / Positioner / Content / Arrow / ArrowTip の 6 anatomy パーツと、Disclosure を埋め込んだ開閉状態機械 HoverCard を提供する。trigger はリンク先プレビュー用途の a 要素であり、Tooltip とはこの点が異なる。",
        "HoverCardDelays（open_ms 既定 600 / close_ms 既定 300、ark-ui 既定値）を data-open-delay/data-close-delay（10 進の ms 値のみ）として決定的に出力する。実際の hover/focus タイマー駆動は wasm-full 側の責務でスコープ外。",
        "WAI-ARIA に hover card 専用パターンが存在しないため、aria-expanded/aria-controls/aria-haspopup 及び content への固定 role を一切付与しない（Tooltip/Popover との違い）。",
        "ark-ui の Root > Trigger > Positioner > (Arrow > ArrowTip) + Content の 6 パートと完全一致（イシュー #1641 で Zag.js `hover-card.connect.ts`・Radix Primitives と突合、是正なし）。data-side/data-align は positioner へ透過させる（tooltip/popover と同型の positioning 規約、#590）。",
    ],
    arguments: &[
        ArgRow {
            name: "root: state",
            kind: "OpenState",
            default: "Closed",
            description: "開閉状態（data-state へ反映）。",
        },
        ArgRow {
            name: "root: delays",
            kind: "HoverCardDelays",
            default: "HoverCardDelays::default()（open_ms: 600, close_ms: 300）",
            description: "data-open-delay/data-close-delay（10 進の ms 値のみ）へ反映される遅延設定。",
        },
        ArgRow {
            name: "trigger: state",
            kind: "OpenState",
            default: "Closed",
            description: "data-state へ反映される開閉状態。",
        },
        ArgRow {
            name: "trigger: href",
            kind: "Option<&str>",
            default: "",
            description: "Some のときのみ href 属性を出力する（リンク先プレビュー用途）。",
        },
        ArgRow {
            name: "positioner: state",
            kind: "OpenState",
            default: "Closed",
            description: "closed のとき hidden 存在属性を付与する配置ラッパーの開閉状態。",
        },
        ArgRow {
            name: "content: state",
            kind: "OpenState",
            default: "Closed",
            description: "開閉状態（closed のとき hidden 存在属性を付与）。",
        },
        ArgRow {
            name: "content: id",
            kind: "Option<&str>",
            default: "",
            description: "呼び出し側が任意に使える id（本モジュールは固定の aria-describedby を配線しない）。",
        },
    ],
    examples: &[
        ExampleEntry {
            title: "Custom delays",
            description: "open_ms/close_ms を既定値（600/300）から変更し、開くまでの遅延を短く・閉じるまでの遅延を長くした例です。",
            render: ex_hover_card_custom_delays,
        },
        ExampleEntry {
            title: "Custom CSS",
            description: "data-scope / data-part / data-state / data-side セレクタで自前 CSS を当てる最小例です。",
            render: ex_hover_card_custom_css,
        },
    ],
    keyboard: &[
        KeyRow {
            key: "Tab",
            description: "trigger はネイティブ a 要素（hover_card.rs:150-162）のため、href が Some のときブラウザ標準でフォーカス到達する。Radix の「Tab で hover card を開閉」に相当する focus/blur 配線は fandhe-frontend-wasm-full 側の責務で未配線。",
        },
        KeyRow {
            key: "Enter",
            description: "href が Some の trigger でリンク先へ遷移する（ブラウザ標準）。hover card 自体の開閉はこのキー操作では行わない。",
        },
    ],
    aria: &[
        AriaRow {
            attribute: "aria-expanded / aria-controls / aria-haspopup（非付与）",
            description: "WAI-ARIA に hover card 専用パターンが無いため、trigger には付与しない（Zag.js `hover-card.connect.ts`・Radix Primitives とも一致、イシュー #1641 突合）。",
        },
        AriaRow {
            attribute: "role（非付与）",
            description: "content にも固定 role を付与しない（Zag.js・Radix とも一致）。Radix の Accessibility 注記: hover card は視覚ユーザー向けであり、content はキーボードユーザーには（フォーカス配線が無い限り）到達不能。",
        },
        AriaRow {
            attribute: "aria-hidden=\"true\"",
            description: "arrow/arrow_tip に固定付与（装飾用のみ、スクリーンリーダーの読み上げ対象外）。",
        },
    ],
    demo: None,
};

// ---------------------------------------------------------------------
// Popover（/primitives/popover/）
// ---------------------------------------------------------------------

/// 一次情報: `crates/headless-ui/src/popover.rs:1-30`（モジュール doc、
/// Disclosure を埋め込んだ開閉状態機械）、`:62-243`（`root`/`trigger`/
/// `anchor`/`positioner`/`arrow`/`arrow_tip`/`content`/`title`/
/// `description`/`close_trigger`/`indicator` シグネチャ、
/// `aria-haspopup="dialog"`/`role="dialog"` の実出力テスト）。
fn ex_popover_minimal_content() -> Node {
    let state = OpenState::Open;
    div(
        vec![],
        vec![
            popover::trigger(
                state,
                false,
                Some("pop-ex-content"),
                vec![],
                vec![
                    text("Options"),
                    popover::indicator(state, vec![], vec![text("▾")]),
                ],
            ),
            popover::positioner(
                state,
                vec![],
                vec![popover::content(
                    state,
                    Some("pop-ex-content"),
                    None,
                    None,
                    vec![],
                    vec![
                        popover::close_trigger(vec![], vec![text("Close")]),
                        text("Minimal content without title or description."),
                    ],
                )],
            ),
        ],
    )
}

pub const POPOVER: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "Root / Trigger / Anchor / Positioner / Arrow / ArrowTip / Content / Title / Description / CloseTrigger / Indicator の 11 anatomy パーツと、Disclosure を埋め込んだ開閉状態機械 Popover を提供する。",
        "trigger は type=\"button\" を固定付与し、aria-haspopup=\"dialog\"・aria-expanded・（controls が Some のとき）aria-controls を持つ。",
        "content は role=\"dialog\" を固定付与し、labelledby/describedby が Some のときのみ title/description と対で関連付ける。",
    ],
    arguments: &[
        ArgRow {
            name: "root: state",
            kind: "OpenState",
            default: "Closed",
            description: "開閉状態（data-state へ反映）。",
        },
        ArgRow {
            name: "trigger: state",
            kind: "OpenState",
            default: "Closed",
            description: "aria-expanded/data-state へ反映される開閉状態。",
        },
        ArgRow {
            name: "trigger: disabled",
            kind: "bool",
            default: "",
            description: "ネイティブ disabled 存在属性と data-disabled の両方へ反映する disabled 状態。",
        },
        ArgRow {
            name: "trigger: controls",
            kind: "Option<&str>",
            default: "",
            description: "Some のとき aria-controls で content と関連付ける。",
        },
        ArgRow {
            name: "positioner: state",
            kind: "OpenState",
            default: "Closed",
            description: "closed のとき hidden 存在属性を付与する配置ラッパーの開閉状態。",
        },
        ArgRow {
            name: "content: state",
            kind: "OpenState",
            default: "Closed",
            description: "開閉状態（closed のとき hidden 存在属性を付与）。",
        },
        ArgRow {
            name: "content: id",
            kind: "Option<&str>",
            default: "",
            description: "trigger の controls と対で aria-controls 関連付けを成立させる。",
        },
        ArgRow {
            name: "content: labelledby",
            kind: "Option<&str>",
            default: "",
            description: "Some のとき title の id と対で aria-labelledby 関連付けを成立させる。",
        },
        ArgRow {
            name: "content: describedby",
            kind: "Option<&str>",
            default: "",
            description: "Some のとき description の id と対で aria-describedby 関連付けを成立させる。",
        },
        ArgRow {
            name: "title: id",
            kind: "Option<&str>",
            default: "",
            description: "content の labelledby と対にする。",
        },
        ArgRow {
            name: "description: id",
            kind: "Option<&str>",
            default: "",
            description: "content の describedby と対にする。",
        },
        ArgRow {
            name: "indicator: state",
            kind: "OpenState",
            default: "Closed",
            description: "開閉状態のみを data-state へ反映する最小主義な装飾用パーツ。",
        },
    ],
    examples: &[ExampleEntry {
        title: "Minimal content",
        description: "title/description を持たず close_trigger のみを含む、最小構成の Popover content の例です。",
        render: ex_popover_minimal_content,
    }],
    keyboard: &[],
    aria: &[
        AriaRow {
            attribute: "aria-haspopup=\"dialog\"",
            description: "trigger に固定付与。",
        },
        AriaRow {
            attribute: "aria-expanded",
            description: "trigger に付与。開閉状態を反映する。",
        },
        AriaRow {
            attribute: "aria-controls",
            description: "trigger に付与。controls が Some のとき content と関連付ける。",
        },
        AriaRow {
            attribute: "role=\"dialog\"",
            description: "content に固定付与。",
        },
        AriaRow {
            attribute: "aria-labelledby / aria-describedby",
            description: "content に付与。labelledby/describedby が Some のときのみ出力される。",
        },
    ],
    demo: None,
};

// ---------------------------------------------------------------------
// Toast（/primitives/toast/）
// ---------------------------------------------------------------------

/// 一次情報: `crates/headless-ui/src/toast.rs:1-35`（モジュール doc、
/// 複数通知を有界キューとして管理する状態機械 Toaster、`aria-live` の
/// 決定的導出）、`:75-149`（`ToastStatus`/`ToastPlacement` 既定値）、
/// `:199-256`（`group`/`root`/`title`/`description`/`action_trigger`/
/// `close_trigger` シグネチャ、`role="region"`/`role="status"`/
/// `aria-atomic`/`aria-live` の実出力テスト）。
fn ex_toast_error() -> Node {
    let status = toast::ToastStatus::Error;
    let placement = toast::ToastPlacement::TopEnd;
    div(
        vec![],
        vec![toast::group(
            placement,
            "Alerts",
            vec![],
            vec![toast::root(
                status,
                vec![],
                vec![
                    toast::title(vec![], vec![text("Upload failed")]),
                    toast::description(vec![], vec![text("Network connection was lost.")]),
                    toast::action_trigger(vec![], vec![text("Retry")]),
                    toast::close_trigger(vec![], vec![text("×")]),
                ],
            )],
        )],
    )
}

pub const TOAST: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "group（live region）/ root（通知 1 件）/ title/description/action-trigger/close-trigger の 6 anatomy パーツと、複数通知を有界なキューとして管理する状態機械 Toaster を提供する（dispatch: dismiss/clear）。",
        "aria-live は ToastStatus から決定的に導出する（呼び出し側文字列を直接流し込まない）。Error のみ \"assertive\"、他は \"polite\"。aria-atomic=\"true\" を併用し、通知全体を単位として読み上げさせる。",
        "group は role=\"region\" + aria-label（label は必須引数）を固定付与する。",
        "タイマーによる自動 dismiss の実配線は fandhe-frontend-wasm-full 側の後続イシューのスコープ。",
    ],
    arguments: &[
        ArgRow {
            name: "group: placement",
            kind: "ToastPlacement",
            default: "BottomEnd",
            description: "data-placement へ反映するビューポート角配置（6 語彙）。",
        },
        ArgRow {
            name: "group: label",
            kind: "&str",
            default: "",
            description: "role=\"region\" とセットの aria-label（呼び出し側が必ず指定する）。",
        },
        ArgRow {
            name: "root: status",
            kind: "ToastStatus",
            default: "Info",
            description: "data-type と aria-live の緊急度（Error のみ assertive、他は polite）を導出する状態。",
        },
    ],
    examples: &[ExampleEntry {
        title: "Error status",
        description: "ToastStatus::Error の Toast は aria-live=\"assertive\" になり、他の状態より即座に割り込んで読み上げられます。",
        render: ex_toast_error,
    }],
    keyboard: &[],
    aria: &[
        AriaRow {
            attribute: "role=\"region\" / aria-label",
            description: "group に固定付与。",
        },
        AriaRow {
            attribute: "role=\"status\"",
            description: "root に固定付与。",
        },
        AriaRow {
            attribute: "aria-atomic=\"true\"",
            description: "root に固定付与。通知全体を単位として読み上げさせる。",
        },
        AriaRow {
            attribute: "aria-live",
            description: "root に付与。status から決定的に導出される（Error のみ assertive、他は polite）。",
        },
    ],
    demo: None,
};

// ---------------------------------------------------------------------
// Toggle Tip（/primitives/toggle-tip/）
// ---------------------------------------------------------------------

/// 一次情報: `crates/headless-ui/src/toggle_tip.rs:1-53`（モジュール doc、
/// Tooltip/Popover との 3 者境界: trigger は aria-expanded を持つが
/// aria-haspopup は付与しない、content は role="tooltip" を持たない）、
/// `:94-206`（`root`/`trigger`/`positioner`/`content`/`arrow`/
/// `arrow_tip` シグネチャ、`aria-hidden="true"` の固定付与）。
fn ex_toggle_tip_disabled() -> Node {
    let state = OpenState::Closed;
    div(
        vec![],
        vec![toggle_tip::root(
            state,
            vec![],
            vec![
                toggle_tip::trigger(state, true, Some("tt-ex-content"), vec![], vec![text("ⓘ")]),
                toggle_tip::positioner(
                    state,
                    vec![],
                    vec![toggle_tip::content(
                        state,
                        Some("tt-ex-content"),
                        vec![],
                        vec![
                            toggle_tip::arrow(vec![], vec![toggle_tip::arrow_tip(vec![], vec![])]),
                            text("Currently unavailable."),
                        ],
                    )],
                ),
            ],
        )],
    )
}

pub const TOGGLE_TIP: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "Tooltip（hover/focus 由来）・Popover（クリック起点のダイアログ相当）のいずれとも異なる立ち位置を持つ: trigger は aria-expanded と（controls が Some のとき）aria-controls を持つが aria-haspopup は付与せず、content は role=\"tooltip\" を持たない非対話テキストとして扱う（モジュール doc §3 者境界）。",
        "Root / Trigger / Positioner / Content / Arrow / ArrowTip の 6 anatomy パーツと、Disclosure を埋め込んだ開閉状態機械 ToggleTip を提供する。",
        "click-outside dismiss（トリガー外クリックでの自動閉鎖）・Escape 閉鎖はクライアントサイド実行時のイベント処理としてスコープ外。",
    ],
    arguments: &[
        ArgRow {
            name: "root: state",
            kind: "OpenState",
            default: "Closed",
            description: "開閉状態（data-state へ反映）。",
        },
        ArgRow {
            name: "trigger: state",
            kind: "OpenState",
            default: "Closed",
            description: "aria-expanded/data-state へ反映される開閉状態。",
        },
        ArgRow {
            name: "trigger: disabled",
            kind: "bool",
            default: "",
            description: "ネイティブ disabled 存在属性と data-disabled の両方へ反映する disabled 状態。",
        },
        ArgRow {
            name: "trigger: controls",
            kind: "Option<&str>",
            default: "",
            description: "Some のとき aria-controls で content と関連付ける。",
        },
        ArgRow {
            name: "positioner: state",
            kind: "OpenState",
            default: "Closed",
            description: "closed のとき hidden 存在属性を付与する配置ラッパーの開閉状態。",
        },
        ArgRow {
            name: "content: state",
            kind: "OpenState",
            default: "Closed",
            description: "開閉状態（closed のとき hidden 存在属性を付与）。",
        },
        ArgRow {
            name: "content: id",
            kind: "Option<&str>",
            default: "",
            description: "trigger の controls と対で aria-controls 関連付けを成立させる。",
        },
    ],
    examples: &[ExampleEntry {
        title: "Disabled trigger",
        description: "disabled な trigger はネイティブ disabled 属性と data-disabled が付与され、フォーカス・展開ができなくなります。",
        render: ex_toggle_tip_disabled,
    }],
    keyboard: &[],
    aria: &[
        AriaRow {
            attribute: "aria-expanded",
            description: "trigger に付与。開閉状態を反映する。",
        },
        AriaRow {
            attribute: "aria-controls",
            description: "trigger に付与。controls が Some のとき content と関連付ける。",
        },
        AriaRow {
            attribute: "aria-haspopup / role=\"tooltip\"（非付与）",
            description: "trigger には aria-haspopup を、content には role=\"tooltip\" を付与しない（Tooltip/Popover との違い、モジュール doc §3 者境界参照）。",
        },
        AriaRow {
            attribute: "aria-hidden=\"true\"",
            description: "arrow/arrow_tip に固定付与（装飾用のみ、スクリーンリーダーの読み上げ対象外）。",
        },
    ],
    demo: None,
};

// ---------------------------------------------------------------------
// Tooltip（/primitives/tooltip/）
// ---------------------------------------------------------------------

/// 一次情報: `crates/headless-ui/src/tooltip.rs:1-35`（モジュール doc、
/// WAI-ARIA tooltip パターン、openDelay/closeDelay/interactive/
/// closeOnEscape はスコープ外）、`:75-186`（`root`/`trigger`/
/// `positioner`/`content`/`arrow`/`arrow_tip` シグネチャ、
/// `aria-describedby`/`role="tooltip"` の実出力テスト）。
fn ex_tooltip_disabled() -> Node {
    let state = OpenState::Closed;
    div(
        vec![],
        vec![tooltip::root(
            state,
            vec![],
            vec![
                tooltip::trigger(
                    state,
                    true,
                    Some("tip-ex-content"),
                    vec![],
                    vec![text("Disabled action")],
                ),
                tooltip::positioner(
                    state,
                    vec![],
                    vec![tooltip::content(
                        state,
                        Some("tip-ex-content"),
                        vec![],
                        vec![
                            tooltip::arrow(vec![], vec![tooltip::arrow_tip(vec![], vec![])]),
                            text("This action is currently unavailable."),
                        ],
                    )],
                ),
            ],
        )],
    )
}

pub const TOOLTIP: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "WAI-ARIA tooltip パターンに従い、trigger は aria-describedby で content（role=\"tooltip\"）と関連付ける。aria-expanded/aria-controls は使用しない（trigger 自体が展開可能なウィジェットではないため）。",
        "Root / Trigger / Positioner / Content / Arrow / ArrowTip の 6 anatomy パーツと、Disclosure を埋め込んだ開閉状態機械 Tooltip を提供する。",
        "openDelay/closeDelay/interactive/closeOnEscape はタイマー・ポインタ座標などクライアントサイド実行時挙動としてスコープ外。",
    ],
    arguments: &[
        ArgRow {
            name: "root: state",
            kind: "OpenState",
            default: "Closed",
            description: "開閉状態（data-state へ反映）。",
        },
        ArgRow {
            name: "trigger: state",
            kind: "OpenState",
            default: "Closed",
            description: "data-state へ反映される開閉状態。",
        },
        ArgRow {
            name: "trigger: disabled",
            kind: "bool",
            default: "",
            description: "ネイティブ disabled 存在属性と data-disabled の両方へ反映する disabled 状態。",
        },
        ArgRow {
            name: "trigger: describedby",
            kind: "Option<&str>",
            default: "",
            description: "Some のとき aria-describedby で content と関連付ける（WAI-ARIA tooltip パターン）。",
        },
        ArgRow {
            name: "positioner: state",
            kind: "OpenState",
            default: "Closed",
            description: "closed のとき hidden 存在属性を付与する配置ラッパーの開閉状態。",
        },
        ArgRow {
            name: "content: state",
            kind: "OpenState",
            default: "Closed",
            description: "開閉状態（closed のとき hidden 存在属性を付与）。",
        },
        ArgRow {
            name: "content: id",
            kind: "Option<&str>",
            default: "",
            description: "trigger の describedby と対で aria-describedby 関連付けを成立させる。",
        },
    ],
    examples: &[ExampleEntry {
        title: "Disabled trigger",
        description: "disabled な trigger はネイティブ disabled 属性と data-disabled が付与され、フォーカス・展開ができなくなります。",
        render: ex_tooltip_disabled,
    }],
    keyboard: &[],
    aria: &[
        AriaRow {
            attribute: "aria-describedby",
            description: "trigger に付与。describedby が Some のとき content と関連付ける。",
        },
        AriaRow {
            attribute: "role=\"tooltip\"",
            description: "content に固定付与。",
        },
        AriaRow {
            attribute: "aria-expanded / aria-controls（非付与）",
            description: "trigger 自体が展開可能なウィジェットではないため付与しない（Collapsible との違い）。",
        },
        AriaRow {
            attribute: "aria-hidden=\"true\"",
            description: "arrow/arrow_tip に固定付与（装飾用のみ、スクリーンリーダーの読み上げ対象外）。",
        },
    ],
    demo: None,
};

/// 本モジュールの `path -> ComponentPageSpec` テーブル（10 部品）。
/// `crate::primitive_specs::SPEC_TABLES` から集約される。
pub(crate) const SPECS: &[(&str, ComponentPageSpec)] = &[
    ("/primitives/accordion/", ACCORDION),
    ("/primitives/collapsible/", COLLAPSIBLE),
    ("/primitives/dialog/", DIALOG),
    ("/primitives/drawer/", DRAWER),
    ("/primitives/floating-panel/", FLOATING_PANEL),
    ("/primitives/hover-card/", HOVER_CARD),
    ("/primitives/popover/", POPOVER),
    ("/primitives/toast/", TOAST),
    ("/primitives/toggle-tip/", TOGGLE_TIP),
    ("/primitives/tooltip/", TOOLTIP),
];

#[cfg(test)]
mod tests {
    use super::SPECS;
    use std::collections::BTreeSet;

    /// `SPECS` 内の path 重複が無いこと（レジストリ追記漏れ・二重登録の
    /// fail-closed 検知）。
    #[test]
    fn specs_have_no_duplicate_paths() {
        let mut seen = BTreeSet::new();
        for (path, _) in SPECS {
            assert!(seen.insert(*path), "duplicate path in SPECS: {path}");
        }
        assert_eq!(SPECS.len(), 10, "SPECS should register exactly 10 pages");
    }

    /// 10 件すべてで `features`/`arguments`/`examples`/`aria` が非空である
    /// こと（受け入れ条件 1 の自己検査。`demo` は Phase 4（#1022）の
    /// `primitive_showcase` が供給するため常に `None` のままでよい）。
    #[test]
    fn specs_have_non_empty_sections_except_demo() {
        for (path, spec) in SPECS {
            assert!(
                !spec.features.is_empty(),
                "{path}: features must not be empty"
            );
            assert!(
                !spec.arguments.is_empty(),
                "{path}: arguments must not be empty"
            );
            assert!(
                !spec.examples.is_empty(),
                "{path}: examples must not be empty"
            );
            assert!(!spec.aria.is_empty(), "{path}: aria must not be empty");
            // collapsible（イシュー #1637）・dialog（イシュー #1638）・
            // hover-card（イシュー #1641）のみ例外: collapsible は trigger が
            // ネイティブ <button> のため Space/Enter が標準操作として成立し、
            // dialog は fandhe-frontend-wasm-full 側（overlay/focus_trap/
            // headless）の実 DOM 配線を確認できたため、hover-card は trigger
            // がネイティブ <a> のため href が Some のときの Tab フォーカス
            // 到達・Enter 遷移がブラウザ標準として成立するため。他 7 部品は
            // キー割り当てを持つ実装が無いため keyboard: &[] を維持する
            // （モジュール doc「keyboard を 10 件すべて空にする理由」参照）。
            if *path == "/primitives/collapsible/"
                || *path == "/primitives/dialog/"
                || *path == "/primitives/hover-card/"
            {
                assert!(
                    !spec.keyboard.is_empty(),
                    "{path}: keyboard should hold the documented rows (#1637/#1638/#1641)"
                );
            } else {
                assert!(
                    spec.keyboard.is_empty(),
                    "{path}: keyboard should stay empty (no keyboard handling implemented)"
                );
            }
            assert!(
                spec.demo.is_none(),
                "{path}: demo should stay None (supplied by primitive_showcase)"
            );
        }
    }
}
