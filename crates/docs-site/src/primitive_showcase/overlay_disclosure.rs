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
    let open = hui::state::OpenState::Open;
    let closed = hui::state::OpenState::Closed;
    let body = vec![accordion::root(
        vec![],
        vec![
            accordion::item(
                open,
                false,
                vec![],
                vec![
                    accordion::item_trigger(
                        open,
                        false,
                        "faq-0",
                        Some("acc-trigger-0"),
                        Some("acc-content-0"),
                        vec![],
                        vec![
                            text("What is fandhe-frontend?"),
                            accordion::item_indicator(open, vec![], vec![text("▾")]),
                        ],
                    ),
                    accordion::item_content(
                        open,
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
                vec![],
                vec![
                    accordion::item_trigger(
                        closed,
                        false,
                        "faq-1",
                        Some("acc-trigger-1"),
                        Some("acc-content-1"),
                        vec![],
                        vec![
                            text("Is it production ready?"),
                            accordion::item_indicator(closed, vec![], vec![text("▾")]),
                        ],
                    ),
                    accordion::item_content(
                        closed,
                        Some("acc-content-1"),
                        Some("acc-trigger-1"),
                        vec![],
                        vec![text("It is under active development.")],
                    ),
                ],
            ),
        ],
    )];
    demo_page("Accordion", body)
}

pub(super) fn collapsible_section() -> Node {
    let state = OpenState::Open;
    let body = vec![collapsible::root(
        state,
        false,
        vec![],
        vec![
            collapsible::trigger(
                state,
                false,
                Some("collapsible-content"),
                vec![],
                vec![
                    text("Show details"),
                    collapsible::indicator(state, vec![], vec![text("▾")]),
                ],
            ),
            collapsible::content(
                state,
                Some("collapsible-content"),
                vec![],
                vec![text("Hidden details revealed here.")],
            ),
        ],
    )];
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
                            drawer::close_trigger(vec![], vec![text("Close")]),
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
                            vec![],
                            vec![
                                floating_panel::title(
                                    Some("fp-title"),
                                    vec![],
                                    vec![text("Notes")],
                                ),
                                floating_panel::control(
                                    vec![],
                                    vec![
                                        floating_panel::stage_trigger(
                                            Stage::Minimized,
                                            vec![],
                                            vec![text("_")],
                                        ),
                                        floating_panel::close_trigger(vec![], vec![text("×")]),
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
    let state = OpenState::Open;
    let body = vec![popover::root(
        state,
        vec![],
        vec![
            popover::trigger(
                state,
                false,
                Some("pop-content"),
                vec![],
                vec![text("Open popover")],
            ),
            popover::anchor(vec![], vec![]),
            popover::positioner(
                state,
                vec![],
                vec![popover::content(
                    state,
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
                        popover::indicator(state, vec![], vec![text("▾")]),
                    ],
                )],
            ),
        ],
    )];
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
    let state = OpenState::Open;
    let body = vec![toggle_tip::root(
        state,
        vec![],
        vec![
            toggle_tip::trigger(state, false, Some("tt-content"), vec![], vec![text("ⓘ")]),
            toggle_tip::positioner(
                state,
                vec![],
                vec![toggle_tip::content(
                    state,
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
    demo_page("Toggle Tip", body)
}

pub(super) fn tooltip_section() -> Node {
    let state = OpenState::Open;
    let body = vec![tooltip::root(
        state,
        vec![],
        vec![
            tooltip::trigger(
                state,
                false,
                Some("tip-content"),
                vec![],
                vec![text("Hover me")],
            ),
            tooltip::positioner(
                state,
                vec![],
                vec![tooltip::content(
                    state,
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
    demo_page("Tooltip", body)
}
