//! Primitives Demo — Data Display / Utilities（10 件、原稿は #1029）。
//! 執筆規約は `crate::primitive_showcase` モジュール doc 参照。

use fandhe_frontend_core::{text, Node};
use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui as hui;
use hui::avatar::{self, ImageStatus};
use hui::data_attrs::Orientation;
use hui::fandhe_frontend_interactive::Component;
use hui::json_tree_view::{self, JsonValue};
use hui::positioning::{Align, Placement, Side};
use hui::scroll_area;
use hui::skip_nav;
use hui::splitter;
use hui::steps::Steps;
use hui::tour::{Tour, TourAction, TourStep};
use hui::tree_view;
use hui::visually_hidden;
use hui::OpenState;

use super::demo_page;

pub(super) fn avatar_section() -> Node {
    // Loaded インスタンス: 実在アセット（`crate::showcase::IMAGE_DEMO_SRC`）を
    // 使い、壊れた画像アイコンを表示しない（イシュー #1659 で是正、
    // 旧実装は解決できない `https://example.com/avatar.png` を描画していた）。
    let loaded = ImageStatus::Loaded;
    let loaded_avatar = avatar::root(
        vec![],
        vec![
            avatar::image(
                loaded,
                crate::showcase::IMAGE_DEMO_SRC,
                "Ada Lovelace",
                vec![],
            ),
            avatar::fallback(loaded, vec![], vec![text("AL")]),
        ],
    );
    // Error インスタンス: 参照サイト（Radix Primitives 等）と同様、画像が
    // 読み込めない場合のフォールバック表示（イニシャル）を Demo 上で示す。
    let error = ImageStatus::Error;
    let error_avatar = avatar::root(
        vec![],
        vec![
            avatar::image(
                error,
                "https://example.com/missing-avatar.png",
                "Priya Das",
                vec![],
            ),
            avatar::fallback(error, vec![], vec![text("PD")]),
        ],
    );
    let body = vec![loaded_avatar, error_avatar];
    demo_page("Avatar", body)
}

pub(super) fn carousel_section() -> Node {
    // 状態機械 `Carousel` の利便メソッド経由で組み立てる（イシュー #1660）。
    // 従来は自由関数を直接呼び `prev_trigger(false, ...)` を固定していたが、
    // index=0・非 loop の構成では実際には `prev_disabled() == true` であり
    // Demo の `data-*` 表に `data-disabled` が現れない不整合があった。
    // 利便メソッドは `Carousel::prev_disabled()`/`next_disabled()` を自動で
    // 注入するため、この不整合が構造的に起きない。
    let carousel = hui::carousel::Carousel::new(0, 3, false, Orientation::Horizontal);
    let body = vec![carousel.root(
        "Featured photos",
        vec![],
        vec![
            carousel.item_group(
                vec![],
                vec![
                    carousel.item(0, vec![], vec![text("Slide 1")]),
                    carousel.item(1, vec![], vec![text("Slide 2")]),
                    carousel.item(2, vec![], vec![text("Slide 3")]),
                ],
            ),
            carousel.control(
                vec![],
                vec![
                    carousel.prev_trigger("Previous slide", vec![], vec![text("‹")]),
                    carousel.indicator_group(
                        vec![],
                        vec![
                            carousel.indicator(0, vec![]),
                            carousel.indicator(1, vec![]),
                            carousel.indicator(2, vec![]),
                        ],
                    ),
                    carousel.next_trigger("Next slide", vec![], vec![text("›")]),
                ],
            ),
        ],
    )];
    demo_page("Carousel", body)
}

pub(super) fn json_tree_view_section() -> Node {
    let value = JsonValue::Object(vec![
        (
            "name".to_string(),
            JsonValue::String("fandhe-frontend".to_string()),
        ),
        (
            "tags".to_string(),
            JsonValue::Array(vec![
                JsonValue::String("rust".to_string()),
                JsonValue::String("ssg".to_string()),
            ]),
        ),
        ("stable".to_string(), JsonValue::Bool(false)),
    ]);
    let tree = json_tree_view::expanded_to_depth(&value, 2);
    let body = vec![json_tree_view::render_json(&tree, &value)];
    demo_page("JSON Tree View", body)
}

pub(super) fn scroll_area_section() -> Node {
    let body = vec![scroll_area::root(
        vec![],
        vec![
            scroll_area::viewport(
                vec![],
                vec![scroll_area::content(
                    vec![],
                    vec![text("Long scrollable content…")],
                )],
            ),
            scroll_area::scrollbar(
                Orientation::Vertical,
                vec![],
                vec![scroll_area::thumb(Orientation::Vertical, vec![], vec![])],
            ),
            scroll_area::corner(vec![], vec![]),
        ],
    )];
    demo_page("Scroll Area", body)
}

pub(super) fn skip_nav_section() -> Node {
    let body = vec![
        skip_nav::link("main-content", vec![], vec![text("Skip to content")]),
        skip_nav::content(
            "main-content",
            vec![],
            vec![text("Main content starts here.")],
        ),
    ];
    demo_page("Skip Nav", body)
}

pub(super) fn splitter_section() -> Node {
    let orientation = Orientation::Horizontal;
    let body = vec![splitter::root(
        orientation,
        false,
        vec![],
        vec![
            splitter::panel(
                "splitter-panel-a",
                orientation,
                vec![],
                vec![text("Panel A")],
            ),
            splitter::resize_trigger(
                orientation,
                "0",
                "100",
                "50",
                "splitter-panel-a",
                false,
                vec![],
                vec![splitter::resize_trigger_indicator(vec![], vec![])],
            ),
            splitter::panel(
                "splitter-panel-b",
                orientation,
                vec![],
                vec![text("Panel B")],
            ),
        ],
    )];
    demo_page("Splitter", body)
}

pub(super) fn steps_section() -> Node {
    let steps = Steps::new(3, 1, Orientation::Horizontal);
    let body = vec![steps.root(
        vec![],
        vec![
            steps.list(
                vec![],
                vec![
                    steps.item(
                        0,
                        vec![],
                        vec![steps.trigger(
                            0,
                            vec![],
                            vec![steps.indicator(0, vec![], vec![text("1")])],
                        )],
                    ),
                    steps.separator(0, vec![], vec![]),
                    steps.item(
                        1,
                        vec![],
                        vec![steps.trigger(
                            1,
                            vec![],
                            vec![steps.indicator(1, vec![], vec![text("2")])],
                        )],
                    ),
                    steps.separator(1, vec![], vec![]),
                    steps.item(
                        2,
                        vec![],
                        vec![steps.trigger(
                            2,
                            vec![],
                            vec![steps.indicator(2, vec![], vec![text("3")])],
                        )],
                    ),
                ],
            ),
            steps.content(1, vec![], vec![text("Step 2 content")]),
            steps.completed_content(vec![], vec![text("All steps completed.")]),
            steps.prev_trigger(vec![], vec![text("Back")]),
            steps.next_trigger(vec![], vec![text("Next")]),
        ],
    )];
    demo_page("Steps", body)
}

pub(super) fn tour_section() -> Node {
    let mut tour = Tour::new(vec![TourStep {
        id: "step-1".to_string(),
        target: Some("#docs-toc-heading".to_string()),
        title: "Page navigation".to_string(),
        description: "Use this menu to jump between sections.".to_string(),
        placement: Placement::new(Side::Bottom, Align::Center),
    }]);
    tour.update(TourAction::Start);
    let body = vec![tour.root(
        vec![],
        vec![
            tour.backdrop(vec![], vec![]),
            tour.spotlight(vec![], vec![]),
            tour.positioner(
                vec![],
                vec![
                    tour.arrow(vec![], vec![tour.arrow_tip(vec![], vec![])]),
                    tour.content(
                        hui::tour::ContentIds {
                            id: Some("tour-content"),
                            labelledby: Some("tour-title"),
                            describedby: Some("tour-desc"),
                        },
                        vec![],
                        vec![
                            tour.title(Some("tour-title"), vec![], vec![text("Page navigation")]),
                            tour.description(
                                Some("tour-desc"),
                                vec![],
                                vec![text("Use this menu to jump between sections.")],
                            ),
                            tour.progress_text(vec![], vec![text("Step 1 of 1")]),
                            tour.action_trigger(vec![], vec![text("Next")]),
                            tour.close_trigger(vec![], vec![text("×")]),
                        ],
                    ),
                ],
            ),
        ],
    )];
    demo_page("Tour", body)
}

pub(super) fn tree_view_section() -> Node {
    let open = OpenState::Open;
    let body = vec![tree_view::root(
        vec![],
        vec![
            tree_view::label(vec![], vec![text("Project files")]),
            tree_view::tree(
                Some("Project files"),
                None,
                vec![],
                vec![
                    tree_view::branch(
                        open,
                        "src",
                        false,
                        false,
                        "1",
                        "1",
                        "2",
                        "0",
                        vec![],
                        vec![
                            tree_view::branch_control(
                                open,
                                false,
                                false,
                                vec![],
                                vec![
                                    tree_view::branch_indicator(open, vec![], vec![text("▾")]),
                                    tree_view::branch_text(vec![], vec![text("src")]),
                                ],
                            ),
                            tree_view::branch_content(
                                open,
                                vec![],
                                vec![
                                    tree_view::branch_indent_guide(vec![], vec![]),
                                    tree_view::item(
                                        "lib.rs",
                                        true,
                                        false,
                                        "2",
                                        "1",
                                        "1",
                                        "1",
                                        vec![],
                                        vec![
                                            tree_view::item_indicator(
                                                true,
                                                vec![],
                                                vec![text("✓")],
                                            ),
                                            tree_view::item_text(vec![], vec![text("lib.rs")]),
                                        ],
                                    ),
                                ],
                            ),
                        ],
                    ),
                    tree_view::item(
                        "README.md",
                        false,
                        false,
                        "1",
                        "2",
                        "2",
                        "0",
                        vec![],
                        vec![
                            tree_view::item_indicator(false, vec![], vec![]),
                            tree_view::item_text(vec![], vec![text("README.md")]),
                        ],
                    ),
                ],
            ),
        ],
    )];
    demo_page("Tree View", body)
}

pub(super) fn visually_hidden_section() -> Node {
    let body = vec![
        text("Total: "),
        visually_hidden::root(vec![], vec![text("(screen-reader only) ")]),
        text("42 items"),
    ];
    demo_page("Visually Hidden", body)
}
