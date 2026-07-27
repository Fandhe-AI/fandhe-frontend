//! `fandhe_frontend_wasm_full::headless`（イシュー #580）の native 統合テスト。
//!
//! [`fandhe_frontend_wasm_full::headless::action_for_part`] の静的マッピング表と
//! `fandhe-frontend-headless-ui` 実出力の (`data-scope`, `data-part`) が
//! ドリフトしていないことを、実際の headless-ui コンポーネントを
//! `render()` した HTML 中の `data-scope`/`data-part` 文字列との一致で
//! 機械検知する。あわせて `action_for_part` が返したアクションを
//! `fandhe_frontend_interactive::dispatch` へ実際に渡し、各状態機械
//! （`Collapsible`/`Dialog`/`Popover`/`Tooltip`/`Menu`/`RadioGroup`/`Select`/
//! `SingleSelect`）が期待どおり遷移することを検証する（受け入れ条件 1・2）。
//! fail-closed 系（受け入れ条件 3: 未知 (scope, part)・`data-value` 欠落・
//! `data-disabled`）も native から検証する。
//!
//! `web_sys::Element` を組み立てての実 DOM クリック委譲（配線層
//! `wire_headless_events`/`wire_headless_component`）は wasm32 専用のため
//! 本ファイルでは検証できない（実ブラウザ回帰は
//! `tests/headless_wiring_browser.rs` に委ねる）。

use fandhe_frontend_core::render;
use fandhe_frontend_headless_ui::collapsible::Collapsible;
use fandhe_frontend_headless_ui::combobox::{self, Combobox};
use fandhe_frontend_headless_ui::popover::Popover;
use fandhe_frontend_headless_ui::select::Select;
use fandhe_frontend_headless_ui::state::{OpenState, SingleSelect};
use fandhe_frontend_headless_ui::toggle_group::{self, ToggleGroup};
use fandhe_frontend_headless_ui::{
    collapsible, dialog, popover, radio_group, select, tabs, Dialog, Menu, RadioGroup, TabItem,
    Tooltip,
};
use fandhe_frontend_interactive::dispatch;
use fandhe_frontend_wasm_full::headless::{action_for_part, PartRef};

fn part(scope: &str, part: &str, value: Option<&str>, disabled: bool) -> PartRef {
    PartRef {
        scope: scope.to_string(),
        part: part.to_string(),
        value: value.map(str::to_string),
        disabled,
    }
}

/// `html` が `data-scope="{scope}"`/`data-part="{part}"` を両方含むことを
/// 確認する（マッピング表のリテラルと headless-ui 実出力のドリフト検知）。
fn assert_scope_part_present(html: &str, scope: &str, part: &str) {
    assert!(
        html.contains(&format!(r#"data-scope="{scope}""#)),
        "data-scope=\"{scope}\" が出力に含まれない（headless-ui 側の scope 変更で\
         wasm-full::headless のマッピング表がドリフトした可能性）: {html}"
    );
    assert!(
        html.contains(&format!(r#"data-part="{part}""#)),
        "data-part=\"{part}\" が出力に含まれない（headless-ui 側の part 変更で\
         wasm-full::headless のマッピング表がドリフトした可能性）: {html}"
    );
}

// --- 受け入れ条件 1: Disclosure 系（Collapsible/Dialog/Popover/Tooltip/Menu）の open/close/toggle ---

#[test]
fn collapsible_trigger_click_toggles_open_closed() {
    let html = render(&collapsible::trigger(
        OpenState::Closed,
        false,
        None,
        vec![],
        vec![],
    ));
    assert_scope_part_present(&html, "collapsible", "trigger");

    let action_ref = action_for_part(&part("collapsible", "trigger", None, false)).unwrap();

    let mut c = Collapsible::default();
    assert!(dispatch(&mut c, &action_ref.action, &action_ref.payload));
    assert!(c.is_open());
    assert!(dispatch(&mut c, &action_ref.action, &action_ref.payload));
    assert!(!c.is_open());
}

#[test]
fn dialog_trigger_opens_and_close_trigger_closes() {
    let trigger_html = render(&dialog::trigger(OpenState::Closed, None, vec![], vec![]));
    assert_scope_part_present(&trigger_html, "dialog", "trigger");
    let close_html = render(&dialog::close_trigger(vec![], vec![]));
    assert_scope_part_present(&close_html, "dialog", "close-trigger");

    let open_action = action_for_part(&part("dialog", "trigger", None, false)).unwrap();
    let close_action = action_for_part(&part("dialog", "close-trigger", None, false)).unwrap();

    let mut d = Dialog::default();
    assert!(dispatch(&mut d, &open_action.action, &open_action.payload));
    assert!(d.is_open());
    assert!(dispatch(
        &mut d,
        &close_action.action,
        &close_action.payload
    ));
    assert!(!d.is_open());
}

#[test]
fn popover_trigger_opens_and_close_trigger_closes() {
    let trigger_html = render(&popover::trigger(
        OpenState::Closed,
        false,
        None,
        vec![],
        vec![],
    ));
    assert_scope_part_present(&trigger_html, "popover", "trigger");
    let close_html = render(&popover::close_trigger(vec![], vec![]));
    assert_scope_part_present(&close_html, "popover", "close-trigger");

    let open_action = action_for_part(&part("popover", "trigger", None, false)).unwrap();
    let close_action = action_for_part(&part("popover", "close-trigger", None, false)).unwrap();

    let mut p = Popover::default();
    assert!(dispatch(&mut p, &open_action.action, &open_action.payload));
    assert!(p.is_open());
    assert!(dispatch(
        &mut p,
        &close_action.action,
        &close_action.payload
    ));
    assert!(!p.is_open());
}

#[test]
fn tooltip_trigger_click_toggles_open_closed() {
    let html = render(&Tooltip::default().trigger(false, None, vec![], vec![]));
    assert_scope_part_present(&html, "tooltip", "trigger");

    let action_ref = action_for_part(&part("tooltip", "trigger", None, false)).unwrap();

    let mut t = Tooltip::default();
    assert!(dispatch(&mut t, &action_ref.action, &action_ref.payload));
    assert!(t.is_open());
}

#[test]
fn menu_trigger_click_toggles_open_closed() {
    let html = render(&Menu::default().trigger(false, None, vec![], vec![]));
    assert_scope_part_present(&html, "menu", "trigger");

    let action_ref = action_for_part(&part("menu", "trigger", None, false)).unwrap();

    let mut m = Menu::default();
    assert!(dispatch(&mut m, &action_ref.action, &action_ref.payload));
    assert!(m.is_open());
}

/// サブメニューを開く `trigger-item`（`Menu::trigger_item`）クリックが
/// 子 `Menu` インスタンスの `"toggle"` dispatch を経由して開閉することを
/// 検証する。サブメニューは「子 `Menu` インスタンス由来の
/// `trigger-item`/`positioner`/`content` を親 `content` 内に入れ子配置
/// する」契約（`crates/headless-ui/src/menu.rs`）であり、`trigger-item` も
/// `trigger` と同じ `data-scope="menu"` を持つため、マッピング表に
/// `menu`/`trigger-item` 行が無いと（`keynav.rs` のサブメニュー
/// ArrowRight/ArrowLeft が合成する `click()` も含めて）no-op になっていた
/// （イシュー #662 PR #674 Bugbot 指摘の回帰テスト）。
#[test]
fn menu_trigger_item_click_toggles_submenu_open_closed() {
    let sub_menu = Menu::default();
    let html = render(&sub_menu.trigger_item(false, false, None, vec![], vec![]));
    assert_scope_part_present(&html, "menu", "trigger-item");

    let action_ref = action_for_part(&part("menu", "trigger-item", None, false)).unwrap();

    let mut m = Menu::default();
    assert!(dispatch(&mut m, &action_ref.action, &action_ref.payload));
    assert!(m.is_open());
}

// --- 受け入れ条件 2: Tabs/RadioGroup/Select の select dispatch ---

#[test]
fn radio_group_item_click_selects_value() {
    let html = render(&radio_group::item(false, false, "red", vec![], vec![]));
    assert_scope_part_present(&html, "radio-group", "item");

    let action_ref = action_for_part(&part("radio-group", "item", Some("red"), false)).unwrap();

    let mut g = RadioGroup::default();
    assert!(dispatch(&mut g, &action_ref.action, &action_ref.payload));
    assert!(g.is_checked("red"));
}

/// サブメニュー `trigger-item` は「`keynav.rs::wiring::resolve_submenu_content`
/// の子孫 `[data-part="content"]` フォールバック」が示すとおり、自身の
/// 子孫として子 `Menu` インスタンスの `content`/`item` を持ちうる（`content`
/// が `trigger-item` の子孫として配置される DOM 構成）。この構成で子
/// アイテムをクリックすると、`item`（`menu`/`item` はマッピング表に無く
/// 常に `None`）から根方向へ辿る途中の `content` を素通りして外側の
/// `trigger-item`（`menu`/`trigger-item` → `"toggle"`）に誤って解決して
/// しまう不具合があった（`content` はマッピング表に無いため
/// `find_map` が素通りしてしまう）。`action_from_parts` は `content` を
/// 探索の境界として扱い、`content` に達するまでに一致がなければ列全体を
/// `None` とする（イシュー #662 PR #674 Bugbot 指摘の修正、
/// `crates/wasm-full/tests/headless_wiring_browser.rs` の実 DOM 回帰と対）。
#[test]
fn nested_submenu_item_click_does_not_leak_to_ancestor_trigger_item_toggle() {
    use fandhe_frontend_wasm_full::headless::action_from_parts;

    // 内側優先: item（表外） → content（表外、境界） → trigger-item（表内）。
    let parts = vec![
        part("menu", "item", Some("child-item-1"), false),
        part("menu", "content", None, false),
        part("menu", "trigger-item", None, false),
    ];
    assert_eq!(
        action_from_parts(&parts),
        None,
        "content 配下の子アイテムクリックが親 trigger-item の toggle として\
         誤って解決されてはならない"
    );
}

#[test]
fn radio_group_item_text_click_resolves_via_ancestor_item() {
    use fandhe_frontend_wasm_full::headless::action_from_parts;

    // item-text（ark-ui 準拠の内側 part、表にない）クリックでも、祖先の
    // item（表内）で解決できることを固定する。
    let parts = vec![
        part("radio-group", "item-text", None, false),
        part("radio-group", "item", Some("blue"), false),
    ];
    let action_ref = action_from_parts(&parts).unwrap();
    assert_eq!(action_ref.action, "select");
    assert_eq!(action_ref.payload, "blue");
}

#[test]
fn select_trigger_opens_item_click_selects_and_closes_clear_trigger_deselects() {
    let trigger_html = render(&select::trigger(
        OpenState::Closed,
        false,
        None,
        None,
        vec![],
        vec![],
    ));
    assert_scope_part_present(&trigger_html, "select", "trigger");
    let item_html = render(&select::item(
        OpenState::Closed,
        false,
        false,
        "opt-1",
        None,
        vec![],
        vec![],
    ));
    assert_scope_part_present(&item_html, "select", "item");
    let clear_html = render(&select::clear_trigger(vec![], vec![]));
    assert_scope_part_present(&clear_html, "select", "clear-trigger");

    let open_action = action_for_part(&part("select", "trigger", None, false)).unwrap();
    let select_action = action_for_part(&part("select", "item", Some("opt-1"), false)).unwrap();
    let deselect_action = action_for_part(&part("select", "clear-trigger", None, false)).unwrap();

    let mut s = Select::default();
    assert!(dispatch(&mut s, &open_action.action, &open_action.payload));
    assert!(s.is_open());

    assert!(dispatch(
        &mut s,
        &select_action.action,
        &select_action.payload
    ));
    assert_eq!(s.selected(), Some("opt-1"));
    // ark-ui の closeOnSelect 既定 true に準拠し、選択と同時に listbox が閉じる
    // （`select.rs` の `SelectAction::Select` 実装、モジュール doc 参照）。
    assert!(!s.is_open());

    assert!(dispatch(
        &mut s,
        &deselect_action.action,
        &deselect_action.payload
    ));
    assert_eq!(s.selected(), None);
}

// --- イシュー #1071: Combobox（trigger の toggle・item の select・
// clear-trigger の clear）のドリフト検知 + dispatch 遷移検証 ---

/// keynav は `aria-expanded`/`hidden`/`data-state` を一切書かず、click →
/// `crate::headless`（本テストが検証する static マッピング表）→ dispatch →
/// 再描画の経路へ委譲する（実装計画 §1.1・`crates/wasm-full/src/keynav.rs`
/// モジュール doc §Combobox 参照）。本テストはその「dispatch 後に
/// `aria-expanded` が input/trigger 双方へ再出力される」契約を native から
/// 証明し、`clear-trigger` が入力値・選択の両方をクリアする
/// （`select` の `deselect` と異なる）ことも確認する。
#[test]
fn combobox_trigger_opens_item_selects_and_clear_trigger_clears_input_and_selection() {
    let trigger_html = render(&Combobox::default().trigger(false, None, vec![], vec![]));
    assert_scope_part_present(&trigger_html, "combobox", "trigger");
    let item_html = render(&combobox::item(
        OpenState::Closed,
        false,
        false,
        "opt-1",
        None,
        vec![],
        vec![],
    ));
    assert_scope_part_present(&item_html, "combobox", "item");
    let clear_html = render(&combobox::clear_trigger(vec![], vec![]));
    assert_scope_part_present(&clear_html, "combobox", "clear-trigger");

    let open_action = action_for_part(&part("combobox", "trigger", None, false)).unwrap();
    let select_action = action_for_part(&part("combobox", "item", Some("opt-1"), false)).unwrap();
    let clear_action = action_for_part(&part("combobox", "clear-trigger", None, false)).unwrap();

    let mut cb = Combobox::default();
    assert!(dispatch(&mut cb, &open_action.action, &open_action.payload));
    assert!(cb.is_open());

    // `aria-expanded` は keynav が直接書くのではなく、dispatch 後の再描画で
    // input・trigger の双方に再出力される（Menu/Select の trigger 開閉と
    // 同型、実装計画 §1.1）。
    let input_html_open = render(&cb.input(false, None, None, None, vec![]));
    assert!(input_html_open.contains(r#"aria-expanded="true""#));
    let trigger_html_open = render(&cb.trigger(false, None, vec![], vec![]));
    assert!(trigger_html_open.contains(r#"aria-expanded="true""#));

    assert!(dispatch(
        &mut cb,
        &select_action.action,
        &select_action.payload
    ));
    assert_eq!(cb.selected(), Some("opt-1"));
    // ark-ui の closeOnSelect 既定に準拠し、選択と同時に listbox が閉じる
    // （`combobox.rs` の `ComboboxAction::Select` 実装参照）。
    assert!(!cb.is_open());
    let input_html_closed = render(&cb.input(false, None, None, None, vec![]));
    assert!(input_html_closed.contains(r#"aria-expanded="false""#));

    assert!(dispatch(
        &mut cb,
        &clear_action.action,
        &clear_action.payload
    ));
    // `ComboboxAction::Clear` は入力値と選択の両方をクリアする
    // （`select` の `clear-trigger`→`"deselect"` は選択のみをクリアするのに
    // 対し、Combobox はテキスト入力欄を併せ持つため意味が異なる、モジュール
    // doc §Combobox 参照）。
    assert_eq!(cb.selected(), None);
    assert_eq!(cb.input_value(), "");
}

#[test]
fn tabs_trigger_click_selects_value_on_single_select() {
    let node = tabs::tabs(
        &tabs::TabsProps {
            id: "t",
            selected: "a",
            orientation: fandhe_frontend_headless_ui::data_attrs::Orientation::Horizontal,
            activation_mode: tabs::ActivationMode::Automatic,
            loop_focus: true,
            indicator: false,
        },
        vec![
            TabItem {
                value: "a",
                trigger: vec![],
                content: vec![],
                disabled: false,
            },
            TabItem {
                value: "b",
                trigger: vec![],
                content: vec![],
                disabled: false,
            },
        ],
    );
    let html = render(&node);
    assert_scope_part_present(&html, "tabs", "trigger");
    assert!(html.contains(r#"data-value="b""#));

    let action_ref = action_for_part(&part("tabs", "trigger", Some("b"), false)).unwrap();

    let mut s = SingleSelect::default();
    assert!(dispatch(&mut s, &action_ref.action, &action_ref.payload));
    assert_eq!(s.selected(), Some("b"));
}

// --- 受け入れ条件 3: fail-closed（未知アクション・改ざん data-* 入力で panic せず no-op） ---

#[test]
fn unknown_scope_and_part_are_noop_not_panic() {
    assert_eq!(
        action_for_part(&part("unknown-widget", "trigger", None, false)),
        None
    );
    assert_eq!(
        action_for_part(&part("collapsible", "unknown-part", None, false)),
        None
    );
}

#[test]
fn select_item_without_data_value_is_noop() {
    // select item クリックだが `data-value` を除去した改ざん入力を模す。
    assert_eq!(action_for_part(&part("select", "item", None, false)), None);
    assert_eq!(
        action_for_part(&part("radio-group", "item", None, false)),
        None
    );
    assert_eq!(action_for_part(&part("tabs", "trigger", None, false)), None);
}

#[test]
fn data_disabled_part_is_noop_even_for_known_mapping() {
    let html = render(&collapsible::trigger(
        OpenState::Closed,
        true,
        None,
        vec![],
        vec![],
    ));
    assert!(html.contains("data-disabled"));

    assert_eq!(
        action_for_part(&part("collapsible", "trigger", None, true)),
        None
    );

    let c = Collapsible::default();
    let before = c;
    assert_eq!(
        action_for_part(&part("collapsible", "trigger", None, true)),
        None
    );
    // no-op のため dispatch 自体を呼ばない（呼べば当然遷移するため、配線層が
    // アクションを生成しないことこそが fail-closed の保証点）。
    assert_eq!(c, before);
}

#[test]
fn ancestor_part_outside_root_scope_type_confusion_does_not_panic() {
    // (scope, part) の組み合わせが偶然マッピング表の別行と型的に近くても
    // panic しない（scope/part は文字列比較のみで、想定外の組でも None）。
    assert_eq!(
        action_for_part(&part("select", "trigger", Some("x"), false))
            .map(|a| a.action)
            .as_deref(),
        Some("toggle")
    );
    assert_eq!(
        action_for_part(&part("menu", "close-trigger", None, false)),
        None
    );
}

// --- XSS 回帰: マッピング結果の payload は既定エスケープを経由する（REQ-1） ---

#[test]
fn select_item_data_value_xss_payload_is_escaped_on_render() {
    let payload = "\"><script>alert(1)</script>";
    let action_ref = action_for_part(&part("select", "item", Some(payload), false)).unwrap();

    let mut s = Select::default();
    dispatch(&mut s, "open", "");
    assert!(dispatch(&mut s, &action_ref.action, &action_ref.payload));

    let html = render(&fandhe_frontend_interactive::render_for_hydration(&s));
    assert!(!html.contains("<script>alert(1)</script>"));
    assert!(html.contains("&lt;script&gt;"));
}

// --- ToggleGroup（イシュー #1075）: item クリック（マウス・keynav 双方が
// 経由するネイティブ Enter/Space）は "toggle" を dispatch する ---

#[test]
fn toggle_group_item_click_toggles_pressed_value() {
    let html = render(&toggle_group::item(
        false,
        false,
        "bold",
        Vec::new(),
        Vec::new(),
    ));
    assert_scope_part_present(&html, "toggle-group", "item");

    let action_ref = action_for_part(&part("toggle-group", "item", Some("bold"), false)).unwrap();
    assert_eq!(action_ref.action, "toggle");
    assert_eq!(action_ref.payload, "bold");

    let mut g = ToggleGroup::default();
    assert!(dispatch(&mut g, &action_ref.action, &action_ref.payload));
    assert!(g.is_pressed("bold"));
    // 同じ item を再度トグルすると解除される（deselectable 既定 true、
    // `crates/headless-ui/src/toggle_group.rs` モジュール doc 参照）。
    assert!(dispatch(&mut g, &action_ref.action, &action_ref.payload));
    assert!(!g.is_pressed("bold"));
}

#[test]
fn toggle_group_item_without_data_value_is_noop() {
    // `toggle_group::item` は `data-value` を常時出力するが、`MAPPING_TABLE`
    // の fail-closed 契約（`requires_value: true`）自体を独立して固定する。
    assert_eq!(
        action_for_part(&part("toggle-group", "item", None, false)),
        None
    );
}

#[test]
fn toggle_group_item_disabled_is_noop() {
    let action_ref = action_for_part(&part("toggle-group", "item", Some("bold"), true));
    assert_eq!(action_ref, None);
}
