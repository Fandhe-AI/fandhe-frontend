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
use fandhe_frontend_headless_ui::combobox::{self, Combobox, ComboboxProps};
use fandhe_frontend_headless_ui::popover::Popover;
use fandhe_frontend_headless_ui::select::Select;
use fandhe_frontend_headless_ui::state::{OpenState, SingleSelect};
use fandhe_frontend_headless_ui::toggle_group::{self, ToggleGroup};
use fandhe_frontend_headless_ui::tree_view::{self, TreeNode, TreeView};
use fandhe_frontend_headless_ui::{
    collapsible, dialog, popover, radio_group, select, tabs, Dialog, Menu, RadioGroup, TabItem,
    Tooltip,
};
use fandhe_frontend_interactive::dispatch;
use fandhe_frontend_wasm_full::headless::{action_for_part, action_from_parts, PartRef};

fn part(scope: &str, part: &str, value: Option<&str>, disabled: bool) -> PartRef {
    PartRef {
        scope: scope.to_string(),
        part: part.to_string(),
        value: value.map(str::to_string),
        disabled,
        readonly: false,
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

/// イシュー #2193: footer 内の text variant close-trigger（headless
/// `dialog::close_trigger_with_variant`）が既存の `(dialog, close-trigger)
/// -> "close"` 配線をそのまま共有することを固定する。`PartRef` は
/// `data-variant` を保持しないため [`action_for_part`] の判定自体は
/// variant に左右されないが、`action_from_parts` の内側優先探索が
/// footer/content/positioner/root を挟んでも close-trigger を正しく
/// 解決することを footer 越境の実配置に近い part 列で検証する。
#[test]
fn dialog_close_trigger_inside_footer_resolves_to_close() {
    use fandhe_frontend_wasm_full::headless::action_from_parts;

    // headless 側が実際に data-variant="text" を出力することを確認する
    // （SSR 突合、`fandhe_frontend_headless_ui::dialog::CloseTriggerVariant`）。
    let close_html = render(&dialog::close_trigger_with_variant(
        dialog::CloseTriggerVariant::Text,
        vec![],
        vec![],
    ));
    assert_scope_part_present(&close_html, "dialog", "close-trigger");
    assert!(close_html.contains(r#"data-variant="text""#));

    // footer は MAPPING_TABLE に登録がないが、click 委譲は event.target から
    // root 方向へ内側優先で辿るため、footer の内側にある close-trigger が
    // 最初に一致し "close" へ解決される（footer 自体は無視される）。
    let parts = vec![
        part("dialog", "close-trigger", None, false),
        part("dialog", "footer", None, false),
        part("dialog", "content", None, false),
        part("dialog", "positioner", None, false),
        part("dialog", "root", None, false),
    ];
    let action_ref =
        action_from_parts(&parts).expect("footer 内の close-trigger は close へ解決するはず");
    assert_eq!(action_ref.action, "close");

    let mut d = Dialog::new(OpenState::Open);
    assert!(dispatch(&mut d, &action_ref.action, &action_ref.payload));
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

// --- イシュー #2205: Menu checkbox-item/radio-item の highlight・click dispatch ---

/// checkbox-item クリックが `MenuCheckboxItem`（`Checkable` 埋め込み）の
/// `"toggle"` dispatch を経由してチェック状態をトグルすることを検証する
/// （`requires_value: false`。`Checkable::decode_action` は payload を
/// 無視する、`menu`/`trigger-item` 行と同型）。
#[test]
fn menu_checkbox_item_click_toggles_checked() {
    use fandhe_frontend_headless_ui::menu::MenuCheckboxItem;

    let html =
        render(&MenuCheckboxItem::default().checkbox_item("wrap", false, false, vec![], vec![]));
    assert_scope_part_present(&html, "menu", "checkbox-item");

    let action_ref = action_for_part(&part("menu", "checkbox-item", Some("wrap"), false)).unwrap();
    assert_eq!(action_ref.action, "toggle");
    assert_eq!(action_ref.payload, "");

    let mut item = MenuCheckboxItem::default();
    assert!(!item.is_checked());
    assert!(dispatch(&mut item, &action_ref.action, &action_ref.payload));
    assert!(item.is_checked());
    assert!(dispatch(&mut item, &action_ref.action, &action_ref.payload));
    assert!(!item.is_checked());
}

/// radio-item クリックが `MenuRadioItemGroup`（`SingleSelect` 埋め込み）の
/// `"select"` dispatch を経由してグループ内排他選択されることを検証する
/// （`requires_value: true`、`data-value` を項目値として使う）。
#[test]
fn menu_radio_item_click_selects_exclusively() {
    use fandhe_frontend_headless_ui::menu::MenuRadioItemGroup;

    let group = MenuRadioItemGroup::default();
    let html = render(&group.radio_item("grid", false, false, vec![], vec![]));
    assert_scope_part_present(&html, "menu", "radio-item");

    let action_ref = action_for_part(&part("menu", "radio-item", Some("grid"), false)).unwrap();
    assert_eq!(action_ref.action, "select");
    assert_eq!(action_ref.payload, "grid");

    let mut g = MenuRadioItemGroup::default();
    assert!(dispatch(&mut g, &action_ref.action, &action_ref.payload));
    assert!(g.is_checked("grid"));
    assert!(!g.is_checked("list"));

    let list_action = action_for_part(&part("menu", "radio-item", Some("list"), false)).unwrap();
    assert!(dispatch(&mut g, &list_action.action, &list_action.payload));
    assert!(g.is_checked("list"));
    assert!(!g.is_checked("grid"));
}

#[test]
fn menu_checkbox_item_without_data_value_still_resolves() {
    // checkbox-item は `requires_value: false` のため `data-value` 欠落でも
    // fail-closed にならない（`menu`/`trigger-item` 行と同型の判断）。
    let action_ref = action_for_part(&part("menu", "checkbox-item", None, false)).unwrap();
    assert_eq!(action_ref.action, "toggle");
    assert_eq!(action_ref.payload, "");
}

#[test]
fn menu_radio_item_without_data_value_is_noop() {
    // radio-item は `requires_value: true` のため `data-value` 欠落は
    // fail-closed で `None`（改ざん・欠損入力を dispatch へ流さない）。
    assert_eq!(
        action_for_part(&part("menu", "radio-item", None, false)),
        None
    );
}

#[test]
fn menu_checkbox_item_and_radio_item_disabled_are_noop() {
    assert_eq!(
        action_for_part(&part("menu", "checkbox-item", Some("wrap"), true)),
        None
    );
    assert_eq!(
        action_for_part(&part("menu", "radio-item", Some("grid"), true)),
        None
    );
}

/// [`action_for_part`] は (scope, part) の 1 段判定のみを行うため、
/// checkbox-item 単体で見れば `"toggle"`（`Disclosure::decode_action`
/// にも一致する語彙）を返す。**この結果を実際の Menu Disclosure へ手動で
/// dispatch すれば当然開閉は連動する**が、これは `action_for_part` 自体の
/// 契約確認であり、実 DOM クリック配線の挙動ではない（実クリックは常に
/// [`action_from_parts`]/`action_from_parts_scoped` 経由で解決され、以下の
/// [`menu_checkbox_item_click_bubbled_to_outer_menu_root_does_not_resolve`]
/// が示すとおり外側 Menu root への誤 dispatch は起きない、codex-review
/// PR #2321 P1 指摘の是正）。
#[test]
fn menu_checkbox_item_toggle_action_manually_dispatched_to_menu_disclosure_opens_it() {
    let action_ref = action_for_part(&part("menu", "checkbox-item", Some("wrap"), false)).unwrap();

    let mut m = Menu::default();
    assert!(dispatch(&mut m, &action_ref.action, &action_ref.payload));
    assert!(m.is_open());
}

/// 実クリック配線の回帰: `MenuCheckboxItem` の専用インスタンスへ
/// `wire_headless_component` せず、外側 Menu の root だけを配線した既存
/// アプリで checkbox-item をクリックした場合の再現（`collect_part_refs`
/// が構築する内側優先の part 列を模す。`parts` の末尾が wire された root
/// 自身、doc は [`fandhe_frontend_wasm_full::headless::action_from_parts`]
/// 参照）。[`action_for_part`] 単体は `"toggle"` を返すが、
/// [`action_from_parts`] は「解決に使われた part（checkbox-item）が wire
/// された root（menu/root）自身ではない」ため `None` を返し、外側 Menu へ
/// 誤って dispatch されない（codex-review PR #2321 P1 指摘の是正）。
#[test]
fn menu_checkbox_item_click_bubbled_to_outer_menu_root_does_not_resolve() {
    let parts = vec![
        part("menu", "checkbox-item", Some("wrap"), false),
        part("menu", "content", None, false),
        part("menu", "positioner", None, false),
        part("menu", "root", None, false),
    ];
    assert!(action_from_parts(&parts).is_none());
}

/// radio-item の `"select"` は `Menu`（Disclosure 埋め込み）の
/// `decode_action` では `None` になる（Disclosure は "select" を受理しない）
/// ため、`action_for_part` を手動 dispatch した場合でも外側 Menu へ越境
/// しない（`action_from_parts` 経由の実クリック配線では checkbox-item も
/// `resolved_part_targets_wired_root` により同様に越境しない、
/// `menu_checkbox_item_click_bubbled_to_outer_menu_root_does_not_resolve`
/// 参照）。
#[test]
fn menu_radio_item_select_action_is_noop_for_menu_disclosure() {
    let action_ref = action_for_part(&part("menu", "radio-item", Some("grid"), false)).unwrap();

    let mut m = Menu::default();
    assert!(!dispatch(&mut m, &action_ref.action, &action_ref.payload));
    assert!(!m.is_open());
}

#[test]
fn menu_checkbox_item_data_value_xss_payload_is_escaped_on_render() {
    use fandhe_frontend_headless_ui::menu::MenuCheckboxItem;

    let html = render(&MenuCheckboxItem::default().checkbox_item(
        "\"><script>alert(1)</script>",
        false,
        false,
        vec![],
        vec![],
    ));
    assert!(!html.contains("<script>"));
    assert!(html.contains("&lt;script&gt;"));
}

// --- イシュー #2205: Menubar checkbox-item/radio-item の click dispatch ---
//
// Menubar 自身は checked 状態機械を持たないため、`menu::MenuCheckboxItem`/
// `MenuRadioItemGroup` を流用する（`crates/headless-ui/src/menubar.rs`
// モジュール doc 参照）。

/// Menubar checkbox-item は `requires_value: false`（menu 側と異なり必須）:
/// `Menubar::decode_action("toggle", payload)` は
/// `payload.parse::<usize>()` を行うため、`data-value`（checkbox-item の
/// 値、Menu index ではない）をそのまま流すと無関係な index への誤 dispatch
/// を招くおそれがある。payload を常に空文字列にすることで
/// `Menubar::decode_action` 側は必ず `None` になる（下の
/// `menubar_checkbox_item_toggle_does_not_drive_menubar_open_state` で固定）。
#[test]
fn menubar_checkbox_item_click_toggles_checked() {
    use fandhe_frontend_headless_ui::menu::MenuCheckboxItem;
    use fandhe_frontend_headless_ui::menubar;

    let html = render(&menubar::checkbox_item(
        false,
        "wrap",
        false,
        false,
        vec![],
        vec![],
    ));
    assert_scope_part_present(&html, "menubar", "checkbox-item");

    let action_ref =
        action_for_part(&part("menubar", "checkbox-item", Some("wrap"), false)).unwrap();
    assert_eq!(action_ref.action, "toggle");
    assert_eq!(action_ref.payload, "");

    let mut item = MenuCheckboxItem::default();
    assert!(dispatch(&mut item, &action_ref.action, &action_ref.payload));
    assert!(item.is_checked());
}

#[test]
fn menubar_radio_item_click_selects_exclusively() {
    use fandhe_frontend_headless_ui::menu::MenuRadioItemGroup;
    use fandhe_frontend_headless_ui::menubar;

    let html = render(&menubar::radio_item(
        false,
        "grid",
        false,
        false,
        vec![],
        vec![],
    ));
    assert_scope_part_present(&html, "menubar", "radio-item");

    let action_ref = action_for_part(&part("menubar", "radio-item", Some("grid"), false)).unwrap();
    assert_eq!(action_ref.action, "select");
    assert_eq!(action_ref.payload, "grid");

    let mut g = MenuRadioItemGroup::default();
    assert!(dispatch(&mut g, &action_ref.action, &action_ref.payload));
    assert!(g.is_checked("grid"));
}

#[test]
fn menubar_checkbox_item_and_radio_item_disabled_are_noop() {
    assert_eq!(
        action_for_part(&part("menubar", "checkbox-item", Some("wrap"), true)),
        None
    );
    assert_eq!(
        action_for_part(&part("menubar", "radio-item", Some("grid"), true)),
        None
    );
}

#[test]
fn menubar_radio_item_without_data_value_is_noop() {
    assert_eq!(
        action_for_part(&part("menubar", "radio-item", None, false)),
        None
    );
}

/// menubar checkbox-item の `"toggle"` payload が常に空文字列であることの
/// 安全性を固定する: `Menubar::decode_action("toggle", "")` は
/// `"".parse::<usize>()` が `Err` になるため `None`（fail-closed）となり、
/// checkbox-item クリックが無関係な Menu の開閉を誤って引き起こさない
/// （`requires_value: true` にした場合の誤 dispatch リスクの回帰テスト）。
#[test]
fn menubar_checkbox_item_toggle_does_not_drive_menubar_open_state() {
    use fandhe_frontend_headless_ui::menubar::Menubar;

    let action_ref =
        action_for_part(&part("menubar", "checkbox-item", Some("wrap"), false)).unwrap();

    let mut mb = Menubar::new(
        0,
        2,
        None,
        false,
        fandhe_frontend_headless_ui::Orientation::Horizontal,
    );
    assert!(!dispatch(&mut mb, &action_ref.action, &action_ref.payload));
    assert_eq!(mb.open(), None);
}

/// menubar radio-item の `"select"` は `Menubar::decode_action` では
/// `None` になる（Menubar は "select" を受理しない）ため、Menu 側と同様に
/// 外側 Menubar へ越境 dispatch しない。
#[test]
fn menubar_radio_item_select_is_noop_for_menubar() {
    use fandhe_frontend_headless_ui::menubar::Menubar;

    let action_ref = action_for_part(&part("menubar", "radio-item", Some("grid"), false)).unwrap();

    let mut mb = Menubar::new(
        0,
        2,
        None,
        false,
        fandhe_frontend_headless_ui::Orientation::Horizontal,
    );
    assert!(!dispatch(&mut mb, &action_ref.action, &action_ref.payload));
    assert_eq!(mb.open(), None);
}

/// 実クリック配線の回帰（menu 側の
/// `menu_checkbox_item_click_bubbled_to_outer_menu_root_does_not_resolve`
/// と同型）: checkbox-item/radio-item の専用インスタンスへ
/// `wire_headless_component` せず、外側 Menubar の root だけを配線した
/// 既存アプリで checkbox-item/radio-item をクリックした場合の再現。
/// `Menubar::decode_action` 側は元々 fail-closed（前掲 2 テスト）だが、
/// ガードが無いと [`action_from_parts`] が `Some` を返し、配線層が
/// dispatch 成功の有無に関わらず `stop_propagation` を呼んでしまうため、
/// checkbox-item/radio-item を独自の click ハンドラで管理している既存
/// アプリのクリックが無言で握りつぶされる（codex-review PR #2321 P1
/// 指摘・同種再発の予防的是正）。
#[test]
fn menubar_checkbox_item_click_bubbled_to_outer_menubar_root_does_not_resolve() {
    let parts = vec![
        part("menubar", "checkbox-item", Some("wrap"), false),
        part("menubar", "content", None, false),
        part("menubar", "positioner", None, false),
        part("menubar", "root", None, false),
    ];
    assert!(action_from_parts(&parts).is_none());
}

#[test]
fn menubar_radio_item_click_bubbled_to_outer_menubar_root_does_not_resolve() {
    let parts = vec![
        part("menubar", "radio-item", Some("grid"), false),
        part("menubar", "radio-item-group", None, false),
        part("menubar", "content", None, false),
        part("menubar", "positioner", None, false),
        part("menubar", "root", None, false),
    ];
    assert!(action_from_parts(&parts).is_none());
}

#[test]
fn menubar_checkbox_item_data_value_xss_payload_is_escaped_on_render() {
    use fandhe_frontend_headless_ui::menubar;

    let html = render(&menubar::checkbox_item(
        false,
        "\"><script>alert(1)</script>",
        false,
        false,
        vec![],
        vec![],
    ));
    assert!(!html.contains("<script>"));
    assert!(html.contains("&lt;script&gt;"));
}

// --- 受け入れ条件 2: Tabs/RadioGroup/Select の select dispatch ---

#[test]
fn radio_group_item_click_selects_value() {
    let html = render(&radio_group::item(
        false,
        &radio_group::RadioGroupProps::default(),
        "red",
        vec![],
        vec![],
    ));
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
    let select_props = select::SelectProps::default();
    let trigger_html = render(&select::trigger(
        OpenState::Closed,
        &select_props,
        false,
        None,
        None,
        vec![],
        vec![],
    ));
    assert_scope_part_present(&trigger_html, "select", "trigger");
    let item_html = render(&select::item(
        OpenState::Closed,
        &select_props,
        false,
        false,
        "opt-1",
        None,
        vec![],
        vec![],
    ));
    assert_scope_part_present(&item_html, "select", "item");
    let clear_html = render(&select::clear_trigger(&select_props, vec![], vec![]));
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
    let props = ComboboxProps::default();
    let trigger_html = render(&Combobox::default().trigger(&props, None, vec![], vec![]));
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
    let clear_html = render(&combobox::clear_trigger(&props, vec![], vec![]));
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
    let input_html_open = render(&cb.input(&props, None, None, None, vec![]));
    assert!(input_html_open.contains(r#"aria-expanded="true""#));
    let trigger_html_open = render(&cb.trigger(&props, None, vec![], vec![]));
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
    let input_html_closed = render(&cb.input(&props, None, None, None, vec![]));
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
        &toggle_group::ToggleGroupProps::default(),
        false,
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

// --- イシュー #1072: TreeView（branch の toggle・item の select）のドリフト
// 検知 + dispatch 遷移検証 ---

/// `crate::keynav::wiring::synthesize_tree_click` はブランチの `click()` を
/// `branch-control`（自身に `data-value` を持たない）へ合成する。
/// `action_from_parts` の内側優先探索により祖先の `branch` 行
/// （`"toggle"`、`data-value` あり）で解決されることを固定する
/// （`crate::keynav` モジュール doc §TreeView §帰結参照）。
#[test]
fn tree_view_branch_control_click_resolves_to_ancestor_branch_toggle() {
    let props = tree_view::TreeItemProps {
        value: "src",
        selected: false,
        disabled: false,
        level: "1",
        posinset: "1",
        setsize: "1",
        depth: "0",
    };
    let branch_html = render(&tree_view::branch(OpenState::Closed, props, vec![], vec![]));
    assert_scope_part_present(&branch_html, "tree-view", "branch");
    let control_html = render(&tree_view::branch_control(
        OpenState::Closed,
        props,
        vec![],
        vec![],
    ));
    assert_scope_part_present(&control_html, "tree-view", "branch-control");

    // branch-control 自身はマッピング表に無い。イシュー #1667 で
    // `data-value` を実際に持つようになったが（`branch_control` の
    // `data-value`/`data-depth` 追加）、`action_for_part` は
    // `(scope, part)` の MAPPING_TABLE 完全一致でのみ解決するため、
    // 値の有無に関わらず None のままであることを固定する（実際の DOM 出力
    // に忠実な `Some("src")` を渡して検証する）。
    assert_eq!(
        action_for_part(&part("tree-view", "branch-control", Some("src"), false)),
        None
    );

    let toggle_action = action_for_part(&part("tree-view", "branch", Some("src"), false)).unwrap();
    assert_eq!(toggle_action.action, "toggle");
    assert_eq!(toggle_action.payload, "src");

    let mut t = TreeView::default();
    assert!(!t.is_expanded("src"));
    assert!(dispatch(
        &mut t,
        &toggle_action.action,
        &toggle_action.payload
    ));
    assert!(t.is_expanded("src"));
}

/// 葉ノード（`item`）は `branch-control` を持たず、
/// `synthesize_tree_click` は item 自身へ `click()` を合成する。
/// `tree-view`/`item` 行が `"select"`（`data-value` 必須）で解決されることを
/// 固定する。
#[test]
fn tree_view_item_click_resolves_to_select() {
    let item_html = render(&tree_view::item(
        tree_view::TreeItemProps {
            value: "a.rs",
            selected: false,
            disabled: false,
            level: "1",
            posinset: "1",
            setsize: "1",
            depth: "0",
        },
        vec![],
        vec![],
    ));
    assert_scope_part_present(&item_html, "tree-view", "item");

    let select_action = action_for_part(&part("tree-view", "item", Some("a.rs"), false)).unwrap();
    assert_eq!(select_action.action, "select");
    assert_eq!(select_action.payload, "a.rs");

    let mut t = TreeView::default();
    assert!(dispatch(
        &mut t,
        &select_action.action,
        &select_action.payload
    ));
    assert_eq!(t.selected(), Some("a.rs"));
}

/// fail-closed 系（受け入れ条件 3 と同型）: `data-value` 欠落・`data-disabled`
/// はいずれも `None`。
#[test]
fn tree_view_branch_and_item_fail_closed_on_missing_value_or_disabled() {
    assert_eq!(
        action_for_part(&part("tree-view", "branch", None, false)),
        None
    );
    assert_eq!(
        action_for_part(&part("tree-view", "item", None, false)),
        None
    );
    assert_eq!(
        action_for_part(&part("tree-view", "branch", Some("src"), true)),
        None
    );
    assert_eq!(
        action_for_part(&part("tree-view", "item", Some("a.rs"), true)),
        None
    );
}

/// `TreeView::render_nodes` の実出力（統合された `branch`/`item`）でも
/// マッピング表とのドリフトが無いことを確認する。
#[test]
fn tree_view_render_nodes_output_matches_mapping_table_scope_and_part() {
    let nodes = vec![
        TreeNode::new("src", "src").with_children(vec![TreeNode::new("a.rs", "a.rs")]),
        TreeNode::new("readme.md", "readme.md"),
    ];
    let t = TreeView::default();
    let rendered = t.render_nodes(&nodes);
    let html = render(&tree_view::root(vec![], rendered));
    assert_scope_part_present(&html, "tree-view", "branch");
    assert_scope_part_present(&html, "tree-view", "item");
    assert_scope_part_present(&html, "tree-view", "branch-control");
}
// --- Calendar（イシュー #1074）: PageUp/PageDown が合成する click が
// prev-month/next-month dispatch へ到達すること ---

#[test]
fn calendar_prev_and_next_trigger_map_to_month_navigation() {
    use fandhe_frontend_headless_ui::calendar::{self, Calendar};
    use fandhe_frontend_headless_ui::date::{PlainDate, Weekday};

    let today = PlainDate::new(2026, 7, 15).unwrap();
    let cal = Calendar::new(2026, 7, today, None, None, None, Weekday::Monday).unwrap();

    let prev_html = render(&cal.prev_trigger(vec![], vec![]));
    assert_scope_part_present(&prev_html, "calendar", "prev-trigger");
    let next_html = render(&cal.next_trigger(vec![], vec![]));
    assert_scope_part_present(&next_html, "calendar", "next-trigger");

    let prev_action = action_for_part(&part("calendar", "prev-trigger", None, false)).unwrap();
    assert_eq!(prev_action.action, "prev-month");
    let next_action = action_for_part(&part("calendar", "next-trigger", None, false)).unwrap();
    assert_eq!(next_action.action, "next-month");

    let mut c = cal;
    assert!(dispatch(&mut c, &next_action.action, &next_action.payload));
    assert_eq!(c.view_month(), 8);
    assert!(dispatch(&mut c, &prev_action.action, &prev_action.payload));
    assert_eq!(c.view_month(), 7);

    // 参考: table/table-body/table-row/day-trigger 実出力の scope/part も
    // ドリフト検知しておく（`crate::keynav::wiring` の CALENDAR_*_SELECTOR
    // 定数の前提）。
    let table_html = render(&calendar::table(None, vec![], vec![]));
    assert_scope_part_present(&table_html, "calendar", "table");
    let body_html = render(&c.table_body_from_grid(vec![]));
    assert_scope_part_present(&body_html, "calendar", "table-body");
    assert!(body_html.contains(r#"data-part="table-row""#));
    assert!(body_html.contains(r#"data-part="day-trigger""#));
}

// --- イシュー #1161: Calendar day-trigger（select クリック）のドリフト
// 検知 + dispatch 遷移検証 ---

#[test]
fn calendar_day_trigger_click_selects_date() {
    use fandhe_frontend_headless_ui::calendar::{self, Calendar};
    use fandhe_frontend_headless_ui::date::{PlainDate, Weekday};

    let today = PlainDate::new(2026, 7, 15).unwrap();
    let cal = Calendar::new(2026, 7, today, None, None, None, Weekday::Monday).unwrap();

    let day = PlainDate::new(2026, 7, 20).unwrap();
    let html = render(&calendar::day_trigger(
        day,
        false,
        false,
        false,
        false,
        None,
        Vec::new(),
        Vec::new(),
    ));
    assert_scope_part_present(&html, "calendar", "day-trigger");
    assert!(html.contains(r#"data-value="2026-07-20""#));

    let action_ref =
        action_for_part(&part("calendar", "day-trigger", Some("2026-07-20"), false)).unwrap();
    assert_eq!(action_ref.action, "select");
    assert_eq!(action_ref.payload, "2026-07-20");

    let mut c = cal;
    assert!(dispatch(&mut c, &action_ref.action, &action_ref.payload));
    assert_eq!(c.selected(), Some(day));
}

#[test]
fn calendar_day_trigger_without_data_value_is_noop() {
    // `calendar::day_trigger` は headless-ui 0.28.0（イシュー #1161）以降
    // `data-value`（ISO 日付）を常時出力するが、`MAPPING_TABLE` の
    // fail-closed 契約（`requires_value: true`）自体を独立して固定する。
    assert_eq!(
        action_for_part(&part("calendar", "day-trigger", None, false)),
        None
    );
}

#[test]
fn calendar_day_trigger_disabled_is_noop() {
    assert_eq!(
        action_for_part(&part("calendar", "day-trigger", Some("2026-07-20"), true)),
        None
    );
}

#[test]
fn calendar_day_trigger_non_iso_payload_fails_parse_and_is_noop() {
    // `Calendar::decode_action` は payload を `PlainDate` としてパースし、
    // パース不能は `None`（fail-closed）。改ざんされた `data-value` を
    // 想定した回帰。
    use fandhe_frontend_headless_ui::calendar::Calendar;
    use fandhe_frontend_headless_ui::date::{PlainDate, Weekday};

    let today = PlainDate::new(2026, 7, 15).unwrap();
    let mut c = Calendar::new(2026, 7, today, None, None, None, Weekday::Monday).unwrap();
    let action_ref =
        action_for_part(&part("calendar", "day-trigger", Some("not-a-date"), false)).unwrap();
    assert!(!dispatch(&mut c, &action_ref.action, &action_ref.payload));
    assert_eq!(c.selected(), None);
}

#[test]
fn calendar_day_trigger_data_value_xss_payload_is_escaped_on_render() {
    // `data-value` は `date.to_iso_string()` 由来（呼び出し側が直接文字列を
    // 注入できない）だが、`render()` 経由の既定エスケープを通ることを
    // `aria-label` と同型に固定する（accordion `data-value` の同名テストと
    // 対をなす回帰）。
    use fandhe_frontend_headless_ui::calendar;
    use fandhe_frontend_headless_ui::date::PlainDate;

    let day = PlainDate::new(2026, 7, 20).unwrap();
    let html = render(&calendar::day_trigger(
        day,
        false,
        false,
        false,
        false,
        None,
        Vec::new(),
        Vec::new(),
    ));
    assert!(html.contains(r#"data-value="2026-07-20""#));
    assert!(html.contains(r#"aria-label="2026-07-20""#));
}

// --- イシュー #1161: NavigationMenu（trigger クリック開閉）のドリフト
// 検知 + dispatch 遷移検証 ---

#[test]
fn navigation_menu_trigger_click_toggles_single_select() {
    use fandhe_frontend_headless_ui::navigation_menu::{self, NavigationMenu};

    let html = render(&navigation_menu::trigger(
        OpenState::Closed,
        false,
        "products",
        None,
        None,
        Vec::new(),
        Vec::new(),
    ));
    assert_scope_part_present(&html, "navigation-menu", "trigger");
    assert!(html.contains(r#"data-value="products""#));

    let action_ref =
        action_for_part(&part("navigation-menu", "trigger", Some("products"), false)).unwrap();
    assert_eq!(action_ref.action, "toggle");
    assert_eq!(action_ref.payload, "products");

    let mut m = NavigationMenu::default();
    assert!(dispatch(&mut m, &action_ref.action, &action_ref.payload));
    assert!(m.is_open("products"));

    // 高々 1 項目が開く（別項目の toggle で前項目が自動的に閉じる）。
    let other = action_for_part(&part(
        "navigation-menu",
        "trigger",
        Some("solutions"),
        false,
    ))
    .unwrap();
    assert!(dispatch(&mut m, &other.action, &other.payload));
    assert!(m.is_open("solutions"));
    assert!(!m.is_open("products"));

    // 再クリック（disclosure 挙動）で閉じる。
    assert!(dispatch(&mut m, &other.action, &other.payload));
    assert!(!m.is_open("solutions"));
}

#[test]
fn navigation_menu_trigger_without_data_value_is_noop() {
    // `navigation_menu::trigger` は headless-ui 0.28.0（イシュー #1161）
    // 以降 `data-value` を常時出力するが、`MAPPING_TABLE` の fail-closed
    // 契約（`requires_value: true`）自体を独立して固定する。
    assert_eq!(
        action_for_part(&part("navigation-menu", "trigger", None, false)),
        None
    );
}

#[test]
fn navigation_menu_trigger_disabled_is_noop() {
    assert_eq!(
        action_for_part(&part("navigation-menu", "trigger", Some("products"), true)),
        None
    );
}

#[test]
fn navigation_menu_trigger_data_value_xss_payload_is_escaped_on_render() {
    use fandhe_frontend_headless_ui::navigation_menu::{self, NavigationMenu};

    let payload = "\"><script>alert(1)</script>";
    let html = render(&navigation_menu::trigger(
        OpenState::Closed,
        false,
        payload,
        None,
        None,
        Vec::new(),
        Vec::new(),
    ));
    assert!(!html.contains("<script>alert(1)</script>"));
    assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));

    let action_ref =
        action_for_part(&part("navigation-menu", "trigger", Some(payload), false)).unwrap();
    let mut m = NavigationMenu::default();
    assert!(dispatch(&mut m, &action_ref.action, &action_ref.payload));

    let rendered_html = render(&fandhe_frontend_interactive::render_for_hydration(&m));
    assert!(!rendered_html.contains("<script>alert(1)</script>"));
    assert!(rendered_html.contains("&lt;script&gt;"));
}

// --- イシュー #1161: Menubar（trigger クリック開閉）のドリフト検知
// + dispatch 遷移検証 ---

#[test]
fn menubar_trigger_click_toggles_open_menu() {
    use fandhe_frontend_headless_ui::menubar::{self, Menubar};

    let html = render(&menubar::trigger(
        true,
        OpenState::Closed,
        false,
        false,
        1,
        None,
        Vec::new(),
        Vec::new(),
    ));
    assert_scope_part_present(&html, "menubar", "trigger");
    assert!(html.contains(r#"data-value="1""#));

    let action_ref = action_for_part(&part("menubar", "trigger", Some("1"), false)).unwrap();
    assert_eq!(action_ref.action, "toggle");
    assert_eq!(action_ref.payload, "1");

    let mut mb = Menubar::new(
        0,
        2,
        None,
        false,
        fandhe_frontend_headless_ui::Orientation::Horizontal,
    );
    assert!(dispatch(&mut mb, &action_ref.action, &action_ref.payload));
    assert_eq!(mb.open(), Some(1));

    // 開いている Menu を再クリックすると閉じる。
    assert!(dispatch(&mut mb, &action_ref.action, &action_ref.payload));
    assert_eq!(mb.open(), None);
}

#[test]
fn menubar_trigger_without_data_value_is_noop() {
    // `menubar::trigger` は headless-ui 0.28.0（イシュー #1161）以降
    // `data-value`（index）を常時出力するが、`MAPPING_TABLE` の
    // fail-closed 契約（`requires_value: true`）自体を独立して固定する。
    assert_eq!(
        action_for_part(&part("menubar", "trigger", None, false)),
        None
    );
}

#[test]
fn menubar_trigger_disabled_is_noop() {
    assert_eq!(
        action_for_part(&part("menubar", "trigger", Some("1"), true)),
        None
    );
}

#[test]
fn menubar_trigger_non_numeric_payload_fails_parse_and_is_noop() {
    // `Menubar::decode_action` は payload を `str::parse::<usize>()` で
    // パースし、パース不能は `None`（fail-closed）。改ざんされた
    // `data-value` を想定した回帰。
    use fandhe_frontend_headless_ui::menubar::Menubar;

    let mut mb = Menubar::new(
        0,
        2,
        None,
        false,
        fandhe_frontend_headless_ui::Orientation::Horizontal,
    );
    let action_ref =
        action_for_part(&part("menubar", "trigger", Some("not-a-number"), false)).unwrap();
    assert!(!dispatch(&mut mb, &action_ref.action, &action_ref.payload));
    assert_eq!(mb.open(), None);
}

#[test]
fn menubar_trigger_out_of_range_index_is_noop() {
    // 範囲外 index no-op は `Menubar::update` の既存契約
    // （`crates/headless-ui/src/menubar.rs` 参照）。
    use fandhe_frontend_headless_ui::menubar::Menubar;

    let mut mb = Menubar::new(
        0,
        2,
        None,
        false,
        fandhe_frontend_headless_ui::Orientation::Horizontal,
    );
    let action_ref = action_for_part(&part("menubar", "trigger", Some("99"), false)).unwrap();
    assert!(dispatch(&mut mb, &action_ref.action, &action_ref.payload));
    assert_eq!(mb.open(), None);
}

// --- イシュー #1127: Accordion（item-trigger クリック開閉）のドリフト
// 検知 + dispatch 遷移検証 ---

#[test]
fn accordion_item_trigger_click_toggles_single_select() {
    use fandhe_frontend_headless_ui::accordion::{self, Accordion, AccordionProps};

    let html = render(&accordion::item_trigger(
        OpenState::Closed,
        false,
        &AccordionProps::default(),
        "panel-1",
        None,
        None,
        Vec::new(),
        Vec::new(),
    ));
    assert_scope_part_present(&html, "accordion", "item-trigger");

    let action_ref =
        action_for_part(&part("accordion", "item-trigger", Some("panel-1"), false)).unwrap();
    assert_eq!(action_ref.action, "toggle");
    assert_eq!(action_ref.payload, "panel-1");

    let mut a = Accordion::default();
    assert!(dispatch(&mut a, &action_ref.action, &action_ref.payload));
    assert!(a.is_open("panel-1"));

    // single モード: 別項目のトグルで前項目が自動的に閉じる。
    let other =
        action_for_part(&part("accordion", "item-trigger", Some("panel-2"), false)).unwrap();
    assert!(dispatch(&mut a, &other.action, &other.payload));
    assert!(a.is_open("panel-2"));
    assert!(!a.is_open("panel-1"));

    // 再トグル（collapsible 挙動）で閉じる。
    assert!(dispatch(&mut a, &other.action, &other.payload));
    assert!(!a.is_open("panel-2"));
}

#[test]
fn accordion_item_trigger_click_toggles_multi_select_independently() {
    use fandhe_frontend_headless_ui::accordion::MultiAccordion;

    let panel_1 =
        action_for_part(&part("accordion", "item-trigger", Some("panel-1"), false)).unwrap();
    let panel_2 =
        action_for_part(&part("accordion", "item-trigger", Some("panel-2"), false)).unwrap();

    let mut a = MultiAccordion::default();
    assert!(dispatch(&mut a, &panel_1.action, &panel_1.payload));
    assert!(dispatch(&mut a, &panel_2.action, &panel_2.payload));
    // multiple モード: 複数項目が同時に開いたままになる。
    assert!(a.is_open("panel-1"));
    assert!(a.is_open("panel-2"));

    // 個別トグルで対象項目のみ閉じる。
    assert!(dispatch(&mut a, &panel_1.action, &panel_1.payload));
    assert!(!a.is_open("panel-1"));
    assert!(a.is_open("panel-2"));
}

#[test]
fn accordion_item_trigger_without_data_value_is_noop() {
    // `accordion::item_trigger` は headless-ui 0.27.0（イシュー #1127）以降
    // `data-value` を常時出力するが、`MAPPING_TABLE` の fail-closed 契約
    // （`requires_value: true`）自体を独立して固定する。
    assert_eq!(
        action_for_part(&part("accordion", "item-trigger", None, false)),
        None
    );
}

#[test]
fn accordion_item_trigger_disabled_is_noop() {
    assert_eq!(
        action_for_part(&part("accordion", "item-trigger", Some("panel-1"), true)),
        None
    );
}

#[test]
fn action_from_parts_is_none_when_ancestor_item_is_disabled_accordion() {
    // accordion の item（祖先）が disabled、内側の item-trigger 自体は
    // enabled。祖先列のいずれか 1 要素でも disabled なら全体を None とする
    // fail-closed 契約（イシュー #580 PR #611 と同型の回帰）。
    use fandhe_frontend_wasm_full::headless::action_from_parts;

    let parts = vec![
        part("accordion", "item-trigger", Some("panel-1"), false),
        part("accordion", "item", None, true),
    ];
    assert_eq!(action_from_parts(&parts), None);
}

#[test]
fn accordion_item_indicator_and_inner_text_click_resolve_via_ancestor_item_trigger() {
    // item-indicator・内側テキスト相当（表にない part）のクリックは
    // `action_from_parts` の内側優先探索により祖先の item-trigger（表内）へ
    // フォールスルーする。
    use fandhe_frontend_wasm_full::headless::action_from_parts;

    let parts = vec![
        part("accordion", "item-indicator", None, false),
        part("accordion", "item-trigger", Some("panel-1"), false),
    ];
    let action_ref = action_from_parts(&parts).unwrap();
    assert_eq!(action_ref.action, "toggle");
    assert_eq!(action_ref.payload, "panel-1");
}

#[test]
fn accordion_item_trigger_data_value_xss_payload_is_escaped_on_render() {
    use fandhe_frontend_headless_ui::accordion::{self, Accordion, AccordionProps};

    let payload = "\"><script>alert(1)</script>";
    let html = render(&accordion::item_trigger(
        OpenState::Closed,
        false,
        &AccordionProps::default(),
        payload,
        None,
        None,
        Vec::new(),
        Vec::new(),
    ));
    assert!(!html.contains("<script>alert(1)</script>"));
    assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));

    let action_ref =
        action_for_part(&part("accordion", "item-trigger", Some(payload), false)).unwrap();
    let mut a = Accordion::default();
    assert!(dispatch(&mut a, &action_ref.action, &action_ref.payload));

    let rendered_html = render(&fandhe_frontend_interactive::render_for_hydration(&a));
    assert!(!rendered_html.contains("<script>alert(1)</script>"));
    assert!(rendered_html.contains("&lt;script&gt;"));
}

// --- Sidebar（イシュー #2074）: trigger/rail → "toggle" ---

#[test]
fn sidebar_trigger_click_toggles_expanded_collapsed() {
    use fandhe_frontend_headless_ui::sidebar::{trigger, Sidebar, SidebarState};

    let sidebar = Sidebar::new(SidebarState::Expanded);
    let html = render(&trigger(
        &sidebar,
        "Toggle Sidebar",
        None,
        Vec::new(),
        Vec::new(),
    ));
    assert_scope_part_present(&html, "sidebar", "trigger");

    let action_ref = action_for_part(&part("sidebar", "trigger", None, false)).unwrap();
    let mut s = Sidebar::new(SidebarState::Expanded);
    assert!(dispatch(&mut s, &action_ref.action, &action_ref.payload));
    assert_eq!(s.state(), SidebarState::Collapsed);

    assert!(dispatch(&mut s, &action_ref.action, &action_ref.payload));
    assert_eq!(s.state(), SidebarState::Expanded);
}

#[test]
fn sidebar_rail_click_toggles_expanded_collapsed() {
    use fandhe_frontend_headless_ui::sidebar::{rail, Sidebar, SidebarState};

    let sidebar = Sidebar::new(SidebarState::Collapsed);
    let html = render(&rail(&sidebar, "Toggle Sidebar", Vec::new(), Vec::new()));
    assert_scope_part_present(&html, "sidebar", "rail");

    let action_ref = action_for_part(&part("sidebar", "rail", None, false)).unwrap();
    let mut s = Sidebar::new(SidebarState::Collapsed);
    assert!(dispatch(&mut s, &action_ref.action, &action_ref.payload));
    assert_eq!(s.state(), SidebarState::Expanded);
}

#[test]
fn sidebar_trigger_disabled_is_noop() {
    assert_eq!(
        action_for_part(&part("sidebar", "trigger", None, true)),
        None
    );
}

#[test]
fn sidebar_rail_disabled_is_noop() {
    assert_eq!(action_for_part(&part("sidebar", "rail", None, true)), None);
}
