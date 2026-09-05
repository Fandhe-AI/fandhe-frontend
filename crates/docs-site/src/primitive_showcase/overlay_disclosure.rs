//! Primitives Demo — Overlay / Disclosure（10 件、原稿は #1027）。
//! 執筆規約は `crate::primitive_showcase` モジュール doc 参照。

use fandhe_frontend_core::{text, Node};
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

use super::demo_page;

pub(super) fn accordion_section() -> Node {
    // イシュー #1636（accordion の参照突合）: 3 件目に disabled 項目を加え、
    // `item-indicator`/`item-content` の `data-disabled`（本イシューで
    // 新規追加）を Demo から機械導出される data-* 属性表へ反映させる。
    let open = hui::state::OpenState::Open;
    let closed = hui::state::OpenState::Closed;
    let props = accordion::AccordionProps::default();
    let body = vec![accordion::root(
        &props,
        vec![],
        vec![
            accordion::item(
                open,
                false,
                &props,
                vec![],
                vec![
                    accordion::item_trigger(
                        open,
                        false,
                        &props,
                        "faq-0",
                        Some("acc-trigger-0"),
                        Some("acc-content-0"),
                        vec![],
                        vec![
                            text("What is fandhe-frontend?"),
                            accordion::item_indicator(open, false, &props, vec![], vec![text("▾")]),
                        ],
                    ),
                    accordion::item_content(
                        open,
                        false,
                        &props,
                        Some("acc-content-0"),
                        Some("acc-trigger-0"),
                        vec![],
                        vec![text("A Rust frontend framework.")],
                    ),
                ],
            ),
            accordion::item(
                closed,
                false,
                &props,
                vec![],
                vec![
                    accordion::item_trigger(
                        closed,
                        false,
                        &props,
                        "faq-1",
                        Some("acc-trigger-1"),
                        Some("acc-content-1"),
                        vec![],
                        vec![
                            text("Is it production ready?"),
                            accordion::item_indicator(
                                closed,
                                false,
                                &props,
                                vec![],
                                vec![text("▾")],
                            ),
                        ],
                    ),
                    accordion::item_content(
                        closed,
                        false,
                        &props,
                        Some("acc-content-1"),
                        Some("acc-trigger-1"),
                        vec![],
                        vec![text("It is under active development.")],
                    ),
                ],
            ),
            accordion::item(
                closed,
                true,
                &props,
                vec![],
                vec![
                    accordion::item_trigger(
                        closed,
                        true,
                        &props,
                        "faq-2",
                        Some("acc-trigger-2"),
                        Some("acc-content-2"),
                        vec![],
                        vec![
                            text("Is enterprise support available?"),
                            accordion::item_indicator(
                                closed,
                                true,
                                &props,
                                vec![],
                                vec![text("▾")],
                            ),
                        ],
                    ),
                    accordion::item_content(
                        closed,
                        true,
                        &props,
                        Some("acc-content-2"),
                        Some("acc-trigger-2"),
                        vec![],
                        vec![text("Coming soon.")],
                    ),
                ],
            ),
        ],
    )];
    demo_page("Accordion", body)
}

pub(super) fn collapsible_section() -> Node {
    // 2 インスタンス構成（open + closed/disabled）にすることで、機械導出表
    // （Anatomy・data-* 属性表）へ `data-state: closed, open` と 4 パート
    // 全ての `data-disabled` を反映させる（イシュー #1637）。
    let open_state = OpenState::Open;
    let body_open = vec![collapsible::root(
        open_state,
        false,
        vec![],
        vec![
            collapsible::trigger(
                open_state,
                false,
                Some("collapsible-content"),
                vec![],
                vec![
                    text("Show details"),
                    collapsible::indicator(open_state, false, vec![], vec![text("▾")]),
                ],
            ),
            collapsible::content(
                open_state,
                false,
                Some("collapsible-content"),
                vec![],
                vec![text("Hidden details revealed here.")],
            ),
        ],
    )];

    let closed_state = OpenState::Closed;
    let body_disabled = vec![collapsible::root(
        closed_state,
        true,
        vec![],
        vec![
            collapsible::trigger(
                closed_state,
                true,
                Some("collapsible-content-disabled"),
                vec![],
                vec![
                    text("Show details (disabled)"),
                    collapsible::indicator(closed_state, true, vec![], vec![text("▾")]),
                ],
            ),
            collapsible::content(
                closed_state,
                true,
                Some("collapsible-content-disabled"),
                vec![],
                vec![text("Hidden details revealed here.")],
            ),
        ],
    )];

    let mut body = body_open;
    body.extend(body_disabled);
    demo_page("Collapsible", body)
}

pub(super) fn dialog_section() -> Node {
    let state = OpenState::Open;
    let body = vec![
        dialog::trigger(
            state,
            Some("dialog-content"),
            vec![],
            vec![text("Open dialog")],
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
                        DialogRole::Dialog,
                        true,
                        ContentIds {
                            id: Some("dialog-content"),
                            labelledby: Some("dialog-title"),
                            describedby: Some("dialog-desc"),
                        },
                        vec![],
                        vec![
                            dialog::title(
                                Some("dialog-title"),
                                vec![],
                                vec![text("Confirm action")],
                            ),
                            dialog::description(
                                Some("dialog-desc"),
                                vec![],
                                vec![text("This cannot be undone.")],
                            ),
                            // codex-review 指摘（PR #1795）: dialog の
                            // close-trigger はアイコン専用契約（0.59.0〜、
                            // `crates/pre-styled-ui/src/dialog.rs` rustdoc
                            // 参照）。支援技術向けラベルは aria-label で維持する。
                            dialog::close_trigger(vec![("aria-label", "Close")], vec![text("×")]),
                        ],
                    )],
                ),
            ],
        ),
    ];
    demo_page("Dialog", body)
}

pub(super) fn drawer_section() -> Node {
    let state = OpenState::Open;
    let placement = DrawerPlacement::End;
    let body = vec![
        drawer::trigger(
            state,
            Some("drawer-content"),
            vec![],
            vec![text("Open drawer")],
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
                        hui::dialog::ContentIds {
                            id: Some("drawer-content"),
                            labelledby: Some("drawer-title"),
                            describedby: None,
                        },
                        vec![],
                        vec![
                            drawer::title(Some("drawer-title"), vec![], vec![text("Filters")]),
                            drawer::description(None, vec![], vec![text("Refine your search.")]),
                            // codex-review 指摘（PR #1795）と同型（イシュー
                            // #1695）: drawer の close-trigger はアイコン
                            // 専用契約（0.6x〜、
                            // `crates/pre-styled-ui/src/drawer.rs` rustdoc
                            // 参照）。支援技術向けラベルは aria-label で維持する。
                            drawer::close_trigger(vec![("aria-label", "Close")], vec![text("×")]),
                        ],
                    )],
                ),
            ],
        ),
    ];
    demo_page("Drawer", body)
}

pub(super) fn floating_panel_section() -> Node {
    let state = OpenState::Open;
    let stage = Stage::Default;
    let body = vec![
        floating_panel::trigger(
            state,
            false,
            Some("fp-content"),
            vec![],
            vec![text("Open panel")],
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
                    Some("fp-content"),
                    Some("fp-title"),
                    vec![],
                    vec![
                        floating_panel::header(
                            stage,
                            vec![],
                            vec![
                                floating_panel::title(
                                    Some("fp-title"),
                                    vec![],
                                    vec![text("Notes")],
                                ),
                                floating_panel::control(
                                    stage,
                                    vec![],
                                    vec![
                                        floating_panel::stage_trigger(
                                            Stage::Minimized,
                                            vec![("aria-label", "Minimize")],
                                            vec![text("_")],
                                        ),
                                        floating_panel::close_trigger(
                                            vec![("aria-label", "Close Window")],
                                            vec![text("×")],
                                        ),
                                    ],
                                ),
                            ],
                        ),
                        floating_panel::body(stage, vec![], vec![text("Panel body content.")]),
                    ],
                )],
            )],
        ),
    ];
    demo_page("Floating Panel", body)
}

pub(super) fn hover_card_section() -> Node {
    let state = OpenState::Open;
    let delays = HoverCardDelays::default();
    let body = vec![hover_card::root(
        state,
        delays,
        vec![],
        vec![
            hover_card::trigger(
                state,
                Some("https://example.com/user"),
                vec![],
                vec![text("@example")],
            ),
            hover_card::positioner(
                state,
                vec![],
                vec![hover_card::content(
                    state,
                    Some("hc-content"),
                    vec![],
                    vec![
                        hover_card::arrow(vec![], vec![hover_card::arrow_tip(vec![], vec![])]),
                        text("Example user profile preview."),
                    ],
                )],
            ),
        ],
    )];
    demo_page("Hover Card", body)
}

pub(super) fn popover_section() -> Node {
    // 2 インスタンス構成（open + closed/disabled）にすることで、機械導出表
    // （Anatomy・data-* 属性表）へ `data-state: closed, open` と
    // `data-disabled` を反映させる（イシュー #1642。collapsible #1637 と
    // 同じ理由）。2 インスタンス目は id を `pop-content-2`/`pop-title-2`/
    // `pop-desc-2` にして重複を避ける。
    let open_state = OpenState::Open;
    let body_open = vec![popover::root(
        open_state,
        vec![],
        vec![
            popover::trigger(
                open_state,
                false,
                Some("pop-content"),
                vec![],
                vec![text("Open popover")],
            ),
            popover::anchor(vec![], vec![]),
            popover::positioner(
                open_state,
                vec![],
                vec![popover::content(
                    open_state,
                    Some("pop-content"),
                    Some("pop-title"),
                    Some("pop-desc"),
                    vec![],
                    vec![
                        popover::arrow(vec![], vec![popover::arrow_tip(vec![], vec![])]),
                        popover::title(Some("pop-title"), vec![], vec![text("Details")]),
                        popover::description(
                            Some("pop-desc"),
                            vec![],
                            vec![text("More information here.")],
                        ),
                        popover::close_trigger(vec![], vec![text("Close")]),
                        popover::indicator(open_state, vec![], vec![text("▾")]),
                    ],
                )],
            ),
        ],
    )];

    let closed_state = OpenState::Closed;
    let body_disabled = vec![popover::root(
        closed_state,
        vec![],
        vec![
            popover::trigger(
                closed_state,
                true,
                Some("pop-content-2"),
                vec![],
                vec![text("Open popover (disabled)")],
            ),
            popover::anchor(vec![], vec![]),
            popover::positioner(
                closed_state,
                vec![],
                vec![popover::content(
                    closed_state,
                    Some("pop-content-2"),
                    Some("pop-title-2"),
                    Some("pop-desc-2"),
                    vec![],
                    vec![
                        popover::arrow(vec![], vec![popover::arrow_tip(vec![], vec![])]),
                        popover::title(Some("pop-title-2"), vec![], vec![text("Details")]),
                        popover::description(
                            Some("pop-desc-2"),
                            vec![],
                            vec![text("More information here.")],
                        ),
                        popover::close_trigger(vec![], vec![text("Close")]),
                        popover::indicator(closed_state, vec![], vec![text("▾")]),
                    ],
                )],
            ),
        ],
    )];

    let mut body = body_open;
    body.extend(body_disabled);
    demo_page("Popover", body)
}

pub(super) fn toast_section() -> Node {
    let status = toast::ToastStatus::Success;
    let placement = toast::ToastPlacement::BottomEnd;
    let body = vec![toast::group(
        placement,
        "Notifications",
        vec![],
        vec![toast::root(
            status,
            vec![],
            vec![
                toast::title(vec![], vec![text("Saved")]),
                toast::description(vec![], vec![text("Your changes have been saved.")]),
                toast::action_trigger(vec![], vec![text("Undo")]),
                toast::close_trigger(vec![], vec![text("×")]),
            ],
        )],
    )];
    demo_page("Toast", body)
}

pub(super) fn toggle_tip_section() -> Node {
    // 2 インスタンス構成（open + closed/disabled）にすることで、機械導出表
    // （Anatomy・data-* 属性表）へ `data-state: closed, open` と
    // `data-disabled` を反映させる（イシュー #1644。popover #1642・
    // collapsible #1637 と同じ理由）。2 インスタンス目は id を
    // `tt-content-2` にして重複を避ける。
    let open_state = OpenState::Open;
    let body_open = vec![toggle_tip::root(
        open_state,
        vec![],
        vec![
            toggle_tip::trigger(
                open_state,
                false,
                Some("tt-content"),
                vec![],
                vec![text("ⓘ")],
            ),
            toggle_tip::positioner(
                open_state,
                vec![],
                vec![toggle_tip::content(
                    open_state,
                    Some("tt-content"),
                    vec![],
                    vec![
                        toggle_tip::arrow(vec![], vec![toggle_tip::arrow_tip(vec![], vec![])]),
                        text("Click again to dismiss."),
                    ],
                )],
            ),
        ],
    )];

    let closed_state = OpenState::Closed;
    let body_disabled = vec![toggle_tip::root(
        closed_state,
        vec![],
        vec![
            toggle_tip::trigger(
                closed_state,
                true,
                Some("tt-content-2"),
                vec![],
                vec![text("ⓘ (disabled)")],
            ),
            toggle_tip::positioner(
                closed_state,
                vec![],
                vec![toggle_tip::content(
                    closed_state,
                    Some("tt-content-2"),
                    vec![],
                    vec![
                        toggle_tip::arrow(vec![], vec![toggle_tip::arrow_tip(vec![], vec![])]),
                        text("Click again to dismiss."),
                    ],
                )],
            ),
        ],
    )];

    let mut body = body_open;
    body.extend(body_disabled);
    demo_page("Toggle Tip", body)
}

pub(super) fn tooltip_section() -> Node {
    // 2 インスタンス構成（open + closed/disabled）にすることで、機械導出表
    // （Anatomy・data-* 属性表）へ `data-state: closed, open` と
    // `data-disabled` を反映させる（イシュー #1645。toggle-tip #1644・
    // popover #1642 と同じ理由）。2 インスタンス目は id を
    // `tip-content-2` にして重複を避ける。
    let open_state = OpenState::Open;
    let body_open = vec![tooltip::root(
        open_state,
        vec![],
        vec![
            tooltip::trigger(
                open_state,
                false,
                Some("tip-content"),
                vec![],
                vec![text("Hover me")],
            ),
            tooltip::positioner(
                open_state,
                vec![],
                vec![tooltip::content(
                    open_state,
                    Some("tip-content"),
                    vec![],
                    vec![
                        tooltip::arrow(vec![], vec![tooltip::arrow_tip(vec![], vec![])]),
                        text("Additional context."),
                    ],
                )],
            ),
        ],
    )];

    let closed_state = OpenState::Closed;
    let body_disabled = vec![tooltip::root(
        closed_state,
        vec![],
        vec![
            tooltip::trigger(
                closed_state,
                true,
                Some("tip-content-2"),
                vec![],
                vec![text("Hover me (disabled)")],
            ),
            tooltip::positioner(
                closed_state,
                vec![],
                vec![tooltip::content(
                    closed_state,
                    Some("tip-content-2"),
                    vec![],
                    vec![
                        tooltip::arrow(vec![], vec![tooltip::arrow_tip(vec![], vec![])]),
                        text("Additional context."),
                    ],
                )],
            ),
        ],
    )];

    let mut body = body_open;
    body.extend(body_disabled);
    demo_page("Tooltip", body)
}
