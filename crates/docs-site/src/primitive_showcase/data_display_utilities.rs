//! Primitives Demo — Data Display / Utilities（16 件、原稿は #1029。
//! イシュー #2125 で `data_table` 追加、旧 15。
//! イシュー #2121 で `message_scroller` 追加、旧 14。
//! イシュー #2114 で `marker` 追加、旧 14。イシュー #2111 で `attachment`
//! 追加、旧 13。イシュー #2108 で `bubble`
//! 追加、旧 12。イシュー #2105 で `message`
//! 追加、旧 11。イシュー #2065 で `item`
//! 追加、旧 10）。
//! 執筆規約は `crate::primitive_showcase` モジュール doc 参照。

use fandhe_frontend_core::{button, li, table, tbody, td, text, thead, tr, ul, Node};
use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui as hui;
use hui::attachment::{self, AttachmentRootProps, AttachmentState, AttachmentVariant};
use hui::avatar::{self, ImageStatus};
use hui::bubble::{self, BubbleGroupPosition, BubbleRootProps, BubbleVariant};
use hui::checkbox::{self, CheckboxProps, CheckedState};
use hui::data_attrs::Orientation;
use hui::data_table::{self, ColumnProps, DataTable, DataTableProps};
use hui::fandhe_frontend_interactive::Component;
use hui::field::{self, FieldIds, FieldProps};
use hui::item::{self, ItemMediaVariant, ItemRootProps, ItemVariant};
use hui::json_tree_view::{self, JsonValue};
use hui::marker::{self, MarkerRootProps, MarkerTone, MarkerVariant};
use hui::menu;
use hui::message::{self, MessageAlign, MessageRole, MessageRootProps};
use hui::message_scroller::{self, MessageScrollerRootProps, MessageScrollerStuck};
use hui::pagination::{ItemMode, Pagination};
use hui::positioning::{Align, Placement, Side};
use hui::progress::Progress;
use hui::scroll_area;
use hui::skip_nav;
use hui::splitter;
use hui::steps::Steps;
use hui::tour::{Tour, TourAction, TourStep, TourTriggerKind};
use hui::tree_view;
use hui::visually_hidden;
use hui::OpenState;

use super::demo_page;

/// variant 2 値（file/image）× state 3 値（idle/uploading/error）×
/// disabled あり/なしを複数の `root` で網羅し、`media`/`content`
/// （`name`/`meta`）/`progress`（`Progress` の root/track/range を入れ子）/
/// `actions` > `action`（有効/disabled）も描画する（デモ執筆規約 2「全
/// anatomy パートを可能な限り全網羅する」・Anatomy/`data-*` 表の機械導出元、
/// `crates/headless-ui/src/attachment.rs` モジュール doc参照）。
pub(super) fn attachment_section() -> Node {
    let file_idle = attachment::root(
        AttachmentRootProps {
            variant: AttachmentVariant::File,
            state: AttachmentState::Idle,
            disabled: false,
        },
        vec![],
        vec![
            attachment::media(vec![], vec![text("📄")]),
            attachment::content(
                vec![],
                vec![
                    attachment::name(vec![], vec![text("report.pdf")]),
                    attachment::meta(vec![], vec![text("PDF · 128 KB")]),
                ],
            ),
            attachment::actions(
                vec![],
                vec![attachment::action(
                    "Delete report.pdf",
                    false,
                    vec![],
                    vec![text("✕")],
                )],
            ),
        ],
    );

    let image_uploading = {
        let progress = Progress::new(0.0, 100.0, Some(64.0), Orientation::Horizontal);
        attachment::root(
            AttachmentRootProps {
                variant: AttachmentVariant::Image,
                state: AttachmentState::Uploading,
                disabled: false,
            },
            vec![],
            vec![
                attachment::media(
                    vec![],
                    vec![fandhe_frontend_core::img(
                        vec![
                            ("src", crate::showcase::IMAGE_DEMO_SRC),
                            ("alt", "photo.png のプレビュー"),
                        ],
                        vec![],
                    )],
                ),
                attachment::content(
                    vec![],
                    vec![
                        attachment::name(vec![], vec![text("photo.png")]),
                        attachment::meta(vec![], vec![text("PNG · 2.4 MB · 64%")]),
                    ],
                ),
                attachment::progress(
                    vec![],
                    vec![progress.root(
                        Some("64%"),
                        vec![],
                        vec![progress.track(vec![], vec![progress.range(vec![], vec![])])],
                    )],
                ),
            ],
        )
    };

    let error_disabled = attachment::root(
        AttachmentRootProps {
            variant: AttachmentVariant::File,
            state: AttachmentState::Error,
            disabled: true,
        },
        vec![],
        vec![
            attachment::media(vec![], vec![text("📄")]),
            attachment::content(
                vec![],
                vec![
                    attachment::name(vec![], vec![text("archive.zip")]),
                    attachment::meta(vec![], vec![text("Upload failed")]),
                ],
            ),
            attachment::actions(
                vec![],
                vec![attachment::action(
                    "Retry archive.zip",
                    true,
                    vec![],
                    vec![text("↻")],
                )],
            ),
        ],
    );

    let body = vec![file_idle, image_uploading, error_disabled];
    demo_page("Attachment", body)
}

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

/// variant 3 値 × align 2 値 × group-position 4 値を複数の `root` で網羅し、
/// `reactions`/`reaction`（selected あり/なし）・`collapse-trigger`/
/// `collapse-content`（open/closed の両方）も描画する（デモ執筆規約 2「全
/// anatomy パートを可能な限り全網羅する」・Anatomy/`data-*` 表の機械導出元、
/// `crates/headless-ui/src/bubble.rs` モジュール doc参照）。
pub(super) fn bubble_section() -> Node {
    let solid_start_single = bubble::root(
        BubbleRootProps {
            variant: BubbleVariant::Solid,
            align: MessageAlign::Start,
            group_position: BubbleGroupPosition::Single,
        },
        vec![],
        vec![bubble::content(
            vec![],
            vec![text("How do I center a div?")],
        )],
    );
    let outline_end_first = bubble::root(
        BubbleRootProps {
            variant: BubbleVariant::Outline,
            align: MessageAlign::End,
            group_position: BubbleGroupPosition::First,
        },
        vec![],
        vec![bubble::content(vec![], vec![text("Use flexbox:")])],
    );
    let plain_end_middle = bubble::root(
        BubbleRootProps {
            variant: BubbleVariant::Plain,
            align: MessageAlign::End,
            group_position: BubbleGroupPosition::Middle,
        },
        vec![],
        vec![bubble::content(
            vec![],
            vec![text(
                "display: flex; align-items: center; justify-content: center;",
            )],
        )],
    );
    let outline_end_last_with_reactions_and_collapse = bubble::root(
        BubbleRootProps {
            variant: BubbleVariant::Outline,
            align: MessageAlign::End,
            group_position: BubbleGroupPosition::Last,
        },
        vec![],
        vec![
            bubble::content(vec![], vec![text("That should do it.")]),
            bubble::reactions(
                "2 reactions",
                vec![],
                vec![
                    bubble::reaction(true, vec![], vec![text("👍")]),
                    bubble::reaction(false, vec![], vec![text("❤")]),
                ],
            ),
            bubble::collapse_trigger(
                OpenState::Open,
                Some("bubble-demo-detail"),
                vec![],
                vec![text("Hide details")],
            ),
            bubble::collapse_content(
                OpenState::Open,
                Some("bubble-demo-detail"),
                vec![],
                vec![text("Sent 09:41 · Edited")],
            ),
        ],
    );
    let plain_start_single_closed_collapse = bubble::root(
        BubbleRootProps {
            variant: BubbleVariant::Plain,
            align: MessageAlign::Start,
            group_position: BubbleGroupPosition::Single,
        },
        vec![],
        vec![
            bubble::content(vec![], vec![text("Thanks!")]),
            bubble::collapse_trigger(OpenState::Closed, None, vec![], vec![text("Show details")]),
            bubble::collapse_content(OpenState::Closed, None, vec![], vec![text("Sent 09:42")]),
        ],
    );
    let body = vec![
        solid_start_single,
        outline_end_first,
        plain_end_middle,
        outline_end_last_with_reactions_and_collapse,
        plain_start_single_closed_collapse,
    ];
    demo_page("Bubble", body)
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

/// DataTable（イシュー #2125）の Demo。8 パーツ（root/toolbar/
/// column-header/sort-trigger/select-all/select-row/footer/
/// selection-count）を、フィルタ入力（`field::input`）・列表示切替
/// （`menu` の checkbox item）・行選択（`checkbox`）・ページング
/// （`pagination`）を入れ子にして 1 行分の構成として示す。headless-ui は
/// `<table>` を生成しないため（モジュール doc「イシュータイトルとの差分」
/// 参照）、Demo でも表組み要素は使わず素の `div` で構造を示す。
pub(super) fn data_table_section() -> Node {
    let t = DataTable::new(
        Some((
            "name".to_string(),
            hui::data_table::SortDirection::Ascending,
        )),
        vec!["email".to_string()],
    );

    let filter_field = FieldProps {
        id: "data-table-filter",
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: false,
    };

    let email_column = ColumnProps {
        id: "email",
        hidden: true,
    };

    // codex-review P1 指摘（PR #2303）の是正: `menu::root` 直下に
    // `role="menuitemcheckbox"` の項目だけを置くと `role="menu"` を持つ
    // `menu::content` が存在せず、支援技術から見て孤立した項目になる
    // （`crates/headless-ui/src/menu.rs` の `content` doc 参照）。
    // `trigger`/`positioner`/`content` を正規の anatomy で組み、Demo として
    // 常時視認できるよう [`OpenState::Open`] で開いた状態を示す。
    let column_toggle_state = OpenState::Open;
    let toolbar = data_table::toolbar(
        vec![],
        vec![
            field::input(&filter_field, vec![("placeholder", "Filter names...")]),
            menu::root(
                column_toggle_state,
                vec![],
                vec![
                    menu::trigger(
                        column_toggle_state,
                        false,
                        Some("data-table-column-toggle-menu"),
                        vec![("id", "data-table-column-toggle-trigger")],
                        vec![text("Columns")],
                    ),
                    menu::positioner(
                        column_toggle_state,
                        vec![],
                        vec![menu::content(
                            column_toggle_state,
                            Some("data-table-column-toggle-menu"),
                            Some("data-table-column-toggle-trigger"),
                            vec![],
                            vec![
                                data_table::column_toggle_item(
                                    &ColumnProps {
                                        id: "name",
                                        hidden: false,
                                    },
                                    false,
                                    false,
                                    vec![],
                                    vec![text("Name")],
                                ),
                                data_table::column_toggle_item(
                                    &email_column,
                                    false,
                                    false,
                                    vec![],
                                    vec![text("Email")],
                                ),
                            ],
                        )],
                    ),
                ],
            ),
        ],
    );

    let name_header = t.column_header(
        "name",
        true,
        vec![],
        vec![t.sort_trigger("name", vec![], vec![text("Name ▲")])],
    );
    let email_header = t.column_header("email", false, vec![], vec![text("Email")]);
    // codex-review P1 指摘（PR #2303）の是正: `checkbox::control` は
    // `aria-hidden="true"` の視覚専用パーツであり、アクセシビリティの実体は
    // `checkbox::hidden_input` が担う契約（`crates/headless-ui/src/
    // checkbox.rs` の `control`/`hidden_input` doc 参照）。`forms_a.rs` の
    // `checkbox_instance` と同じ構成（control > indicator / label /
    // hidden_input）へ揃え、可視ラベルはデモの文脈上冗長なため
    // `visually_hidden::root` でアクセシブルな名前のみ与える。
    let select_all_props = CheckboxProps {
        checked: CheckedState::Indeterminate,
        ..Default::default()
    };
    let select_all_header = DataTable::select_all(
        CheckedState::Indeterminate,
        vec![],
        vec![checkbox::root(
            &select_all_props,
            vec![],
            vec![
                checkbox::control(
                    &select_all_props,
                    vec![],
                    vec![checkbox::indicator(
                        &select_all_props,
                        vec![],
                        vec![text("◐")],
                    )],
                ),
                checkbox::label(
                    &select_all_props,
                    vec![],
                    vec![visually_hidden::root(vec![], vec![text("Select all rows")])],
                ),
                checkbox::hidden_input(&select_all_props, "data-table-select-all", "on", vec![]),
            ],
        )],
    );

    let select_row_props = CheckboxProps {
        checked: CheckedState::Checked,
        ..Default::default()
    };
    let select_row_cell = DataTable::select_row(
        CheckedState::Checked,
        vec![],
        vec![checkbox::root(
            &select_row_props,
            vec![],
            vec![
                checkbox::control(
                    &select_row_props,
                    vec![],
                    vec![checkbox::indicator(
                        &select_row_props,
                        vec![],
                        vec![text("✓")],
                    )],
                ),
                checkbox::label(
                    &select_row_props,
                    vec![],
                    vec![visually_hidden::root(
                        vec![],
                        vec![text("Select row: Ada Lovelace")],
                    )],
                ),
                checkbox::hidden_input(&select_row_props, "data-table-select-row-1", "on", vec![]),
            ],
        )],
    );

    let pager = Pagination::new(30, 10, 1, 1, 1);
    let footer = data_table::footer(
        vec![],
        vec![
            data_table::selection_count(vec![], vec![text("1 of 3 row(s) selected")]),
            pager.root(
                "Table pagination",
                vec![],
                vec![
                    pager.prev_trigger(ItemMode::Button, vec![], vec![text("‹")]),
                    pager.item(ItemMode::Button, 1, false, vec![], vec![text("1")]),
                    pager.next_trigger(ItemMode::Button, vec![], vec![text("›")]),
                ],
            ),
        ],
    );

    // codex-review P1 指摘（PR #2303）の是正: `column_header`/`select_all`
    // は `th`、`select_row` は `td` を生成するため、`table`/`thead`/`tbody`/
    // `tr`（core のノード API）で正規の table 構造を組んでからその中へ
    // 配置する。`div` 直下に置くとブラウザの HTML 解析で `th`/`td` が
    // table 構造外として無視され、`aria-sort`/`hidden`/`data-part` 等の
    // 表示状態が失われるため（呼び出し側の責務。モジュール doc「責務
    // 境界」参照。headless-ui 自体は新規に表組みを生成しない）。
    let header_row = tr(vec![], vec![select_all_header, name_header, email_header]);
    let body_row = tr(
        vec![],
        vec![
            select_row_cell,
            td(vec![], vec![text("Ada Lovelace")]),
            td(
                data_table::column_attrs(&email_column),
                vec![text("ada@example.com")],
            ),
        ],
    );
    // primitives-showcase.css は headless-ui のマークアップへスタイルを
    // 到達させない不変条件を持つため（モジュール doc「スタイル分離が
    // 必須である理由」参照）、`table`/`th`/`td` を装飾する class は
    // 付与しない（codex-review P1 指摘、PR #2303 是正）。
    let data_table = table(
        vec![],
        vec![
            thead(vec![], vec![header_row]),
            tbody(vec![], vec![body_row]),
        ],
    );

    let root = DataTable::root(
        DataTableProps {
            loading: false,
            empty: false,
        },
        vec![],
        vec![toolbar, data_table, footer],
    );
    demo_page("Data Table", vec![root])
}

/// Item（イシュー #2065）の Demo。10 パーツ（root/media/content/title/
/// description/actions/header/footer/group/separator）すべてを描画する
/// （`anatomy_coverage_matches_known_uncovered_exactly` が `.part("…")`
/// 集合と Demo の完全一致を要求するため）。`group` の中に `root`（div）+
/// `separator` + `root`（a、`example.com` への外部リンク。サイト内部
/// パスへの直リンクは linkcheck の base_path 解決対象になるため、他の
/// Demo と同じく RFC 2606 予約ドメインのダミー URL を使う）を並べ、
/// `group` が
/// `role="group"` を持つこと（`role="list"` ではない、`item` モジュール
/// doc の意図的差分）を Anatomy 表から確認できるようにする。
pub(super) fn item_section() -> Node {
    let div_item = item::root(
        ItemRootProps {
            variant: ItemVariant::Outline,
            ..Default::default()
        },
        vec![],
        vec![
            item::header(vec![], vec![text("New")]),
            item::media(ItemMediaVariant::Icon, vec![], vec![text("🔔")]),
            item::content(
                vec![],
                vec![
                    item::title(vec![], vec![text("First item")]),
                    item::description(vec![], vec![text("A short description.")]),
                ],
            ),
            item::actions(
                vec![],
                vec![button(vec![("type", "button")], vec![text("Open")])],
            ),
            item::footer(vec![], vec![text("Updated just now")]),
        ],
    );
    let separator = item::separator(vec![], vec![]);
    let link_item = item::root(
        ItemRootProps {
            href: Some("https://example.com/items/42"),
            ..Default::default()
        },
        vec![],
        vec![
            item::media(ItemMediaVariant::Image, vec![], vec![]),
            item::content(vec![], vec![item::title(vec![], vec![text("Second item")])]),
        ],
    );
    let group = item::group("Recent items", vec![], vec![div_item, separator, link_item]);
    demo_page("Item", vec![group])
}

pub(super) fn json_tree_view_section() -> Node {
    // イシュー #1661: `data-kind` 6 語彙（null/boolean/number/string/array/
    // object）すべてが Demo 経由で data-* 表へ機械導出されるよう、当初
    // 欠けていた number（"version"）・null（"license"）を追加した。
    let value = JsonValue::Object(vec![
        (
            "name".to_string(),
            JsonValue::String("fandhe-frontend".to_string()),
        ),
        ("version".to_string(), JsonValue::Number(1.0)),
        (
            "tags".to_string(),
            JsonValue::Array(vec![
                JsonValue::String("rust".to_string()),
                JsonValue::String("ssg".to_string()),
            ]),
        ),
        ("stable".to_string(), JsonValue::Bool(false)),
        ("license".to_string(), JsonValue::Null),
    ]);
    let tree = json_tree_view::expanded_to_depth(&value, 2);
    let body = vec![json_tree_view::render_json(&tree, &value)];
    demo_page("JSON Tree View", body)
}

/// イシュー #2114: 3 anatomy パーツ全て（root/icon/content）と
/// `data-variant` 3 値（note/divider/label）× `data-tone` 4 値
/// （neutral/info/warning/danger）を複数の `root` で全網羅する（デモ執筆
/// 規約 2「全 anatomy パートを可能な限り全網羅する」・Anatomy/`data-*` 表の
/// 機械導出元、`crates/headless-ui/src/marker.rs` モジュール doc参照）。
/// 区切り線は headless 層が出力しないため（同モジュール doc「区切り線は
/// headless で描かない」参照）、Demo でも線要素は描画しない。
pub(super) fn marker_section() -> Node {
    let note_neutral = marker::root(
        MarkerRootProps {
            variant: MarkerVariant::Note,
            tone: MarkerTone::Neutral,
        },
        vec![],
        vec![
            marker::icon(vec![], vec![text("i")]),
            marker::content(vec![], vec![text("Explored 4 files")]),
        ],
    );
    let divider_info = marker::root(
        MarkerRootProps {
            variant: MarkerVariant::Divider,
            tone: MarkerTone::Info,
        },
        vec![],
        vec![marker::content(
            vec![],
            vec![text("Conversation compacted")],
        )],
    );
    let label_warning = marker::root(
        MarkerRootProps {
            variant: MarkerVariant::Label,
            tone: MarkerTone::Warning,
        },
        vec![],
        vec![marker::content(vec![], vec![text("Today")])],
    );
    let note_danger = marker::root(
        MarkerRootProps {
            variant: MarkerVariant::Note,
            tone: MarkerTone::Danger,
        },
        vec![],
        vec![marker::content(vec![], vec![text("Connection lost")])],
    );
    demo_page(
        "Marker",
        vec![note_neutral, divider_info, label_warning, note_danger],
    )
}

/// イシュー #2105: 6 anatomy パーツ全て（root/avatar/header/content/footer/
/// group）と `data-role` 3 値 × `data-align` 2 値 + `data-loading`/
/// `data-error` を必ず描画する（`anatomy_coverage_matches_known_uncovered_exactly`
/// が `.part("…")` 集合の完全一致を、`data-*` 表が観測値のみを機械導出する
/// ため）。`avatar` スロットへ既存 `avatar` モジュールを入れ子にする
/// （scope フィルタにより message 側の表を汚さない、モジュール doc
/// 「`avatar` はスロット」参照）。
pub(super) fn message_section() -> Node {
    let user_message = message::root(
        MessageRootProps {
            role: MessageRole::User,
            align: MessageAlign::End,
            ..Default::default()
        },
        vec![],
        vec![
            message::avatar(
                vec![],
                vec![avatar::image(
                    ImageStatus::Loaded,
                    crate::showcase::IMAGE_DEMO_SRC,
                    "You",
                    vec![],
                )],
            ),
            message::header(vec![], vec![text("You")]),
            message::content(vec![], vec![text("How do I center a div?")]),
            message::footer(vec![], vec![text("09:41")]),
        ],
    );
    let assistant_loading = message::root(
        MessageRootProps {
            role: MessageRole::Assistant,
            align: MessageAlign::Start,
            loading: true,
            ..Default::default()
        },
        vec![],
        vec![
            message::avatar(vec![], vec![text("AI")]),
            message::content(vec![], vec![text("Thinking...")]),
        ],
    );
    let system_error = message::root(
        MessageRootProps {
            role: MessageRole::System,
            align: MessageAlign::Start,
            error: true,
            ..Default::default()
        },
        vec![],
        vec![message::content(
            vec![],
            vec![text("Failed to send message.")],
        )],
    );
    let group = message::group(
        "Conversation",
        vec![],
        vec![user_message, assistant_loading, system_error],
    );
    demo_page("Message", vec![group])
}

/// イシュー #2121: 6 anatomy パーツ全て（root/viewport/content/anchor/
/// jump-to-latest/load-more）と `data-stuck` 2 値（bottom/free）を必ず
/// 描画する（`anatomy_coverage_matches_known_uncovered_exactly` が
/// `.part("…")` 集合の完全一致を、`data-*` 表が観測値のみを機械導出する
/// ため）。`content` には [`mod@hui::message`] の `group`/`root` を入れ子に
/// する（scope フィルタにより message-scroller 側の表を汚さない、
/// `crates/headless-ui/src/attachment.rs` が Progress を入れ子にした先例と
/// 同型）。1 件目は既定状態（`data-stuck="bottom"`、jump-to-latest 非表示、
/// load-more 通常）、2 件目は離脱状態（`data-stuck="free"`・`data-has-new`
/// あり・jump-to-latest 可視・load-more `loading`/`disabled`）の
/// 2 インスタンスを静的に併記する（`sidebar-07` block が
/// expanded/collapsed を並記した先例と同型の Demo 規約）。
pub(super) fn message_scroller_section() -> Node {
    let conversation = message::group(
        "Conversation",
        vec![],
        vec![
            message::root(
                MessageRootProps {
                    role: MessageRole::User,
                    ..Default::default()
                },
                vec![],
                vec![message::content(
                    vec![],
                    vec![text("How do I center a div?")],
                )],
            ),
            message::root(
                MessageRootProps {
                    role: MessageRole::Assistant,
                    ..Default::default()
                },
                vec![],
                vec![message::content(
                    vec![],
                    vec![text("Use display: flex and align-items: center.")],
                )],
            ),
        ],
    );
    let at_bottom = message_scroller::root(
        MessageScrollerRootProps {
            stuck: MessageScrollerStuck::Bottom,
            has_new: false,
        },
        vec![],
        vec![
            message_scroller::viewport(
                "Conversation history",
                vec![],
                vec![
                    message_scroller::content(vec![], vec![conversation]),
                    message_scroller::anchor(vec![]),
                ],
            ),
            message_scroller::jump_to_latest("Jump to latest", false, vec![], vec![text("↓")]),
            message_scroller::load_more(false, false, vec![], vec![text("Load older messages")]),
        ],
    );

    let scrolled_up_thread = message::group(
        "Conversation",
        vec![],
        vec![message::root(
            MessageRootProps {
                role: MessageRole::Assistant,
                ..Default::default()
            },
            vec![],
            vec![message::content(
                vec![],
                vec![text("New reply while you were scrolled up.")],
            )],
        )],
    );
    let scrolled_up = message_scroller::root(
        MessageScrollerRootProps {
            stuck: MessageScrollerStuck::Free,
            has_new: true,
        },
        vec![],
        vec![
            message_scroller::viewport(
                "Conversation history",
                vec![],
                vec![
                    message_scroller::content(vec![], vec![scrolled_up_thread]),
                    message_scroller::anchor(vec![]),
                ],
            ),
            message_scroller::jump_to_latest("Jump to latest", true, vec![], vec![text("↓")]),
            message_scroller::load_more(
                true,
                true,
                vec![],
                vec![text("Loading older messages...")],
            ),
        ],
    );

    demo_page("Message Scroller", vec![at_bottom, scrolled_up])
}

// イシュー #1662（参考サイトとの突合）: Radix Primitives の Anatomy は
// 水平・垂直 2 本の scrollbar + corner を組み合わせて示す（corner は両軸が
// 揃って初めて意味を持つ交差部分のため）。旧 Demo は垂直 1 本のみで corner
// の役割が伝わらなかったため、複数項目のリスト状コンテンツ（Radix デモの
// 「Tags」相当）とあわせて両軸構成へ更新した。
pub(super) fn scroll_area_section() -> Node {
    let items: Vec<Node> = (1..=12)
        .map(|n| li(vec![], vec![text(format!("Tag {n}"))]))
        .collect();
    let body = vec![scroll_area::root(
        vec![],
        vec![
            scroll_area::viewport(
                vec![],
                vec![scroll_area::content(vec![], vec![ul(vec![], items)])],
            ),
            scroll_area::scrollbar(
                Orientation::Vertical,
                vec![],
                vec![scroll_area::thumb(Orientation::Vertical, vec![], vec![])],
            ),
            scroll_area::scrollbar(
                Orientation::Horizontal,
                vec![],
                vec![scroll_area::thumb(Orientation::Horizontal, vec![], vec![])],
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
                0,
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
                "splitter-panel-b",
                false,
                vec![],
                vec![splitter::resize_trigger_indicator(vec![], vec![])],
            ),
            splitter::panel(
                "splitter-panel-b",
                1,
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
            steps.progress(vec![("aria-label", "Steps progress")], vec![]),
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
                            tour.control(
                                vec![],
                                vec![
                                    tour.action_trigger(
                                        TourTriggerKind::Prev,
                                        vec![],
                                        vec![text("Prev")],
                                    ),
                                    tour.action_trigger(
                                        TourTriggerKind::Next,
                                        vec![],
                                        vec![text("Next")],
                                    ),
                                ],
                            ),
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
    // イシュー #1667 の参照突合で追加した data-* 属性（`branch` の
    // `data-branch`、`branch-control`/`branch-content` の `data-value`/
    // `data-depth`、テキスト・インジケータ系の `data-selected`/
    // `data-disabled`/`data-state`、item-indicator の `hidden`/
    // `aria-hidden`）が docs-site の data-* 属性表へ機械導出されるよう、
    // open ブランチ（選択済み葉を含む）・closed ブランチ（`hidden` な
    // branch-content）・disabled 葉・非選択葉をすべて含める。
    let open = OpenState::Open;
    let closed = OpenState::Closed;
    let src_props = tree_view::TreeItemProps {
        value: "src",
        selected: false,
        disabled: false,
        level: "1",
        posinset: "1",
        setsize: "3",
        depth: "0",
    };
    let lib_rs_props = tree_view::TreeItemProps {
        value: "lib.rs",
        selected: true,
        disabled: false,
        level: "2",
        posinset: "1",
        setsize: "1",
        depth: "1",
    };
    let docs_props = tree_view::TreeItemProps {
        value: "docs",
        selected: false,
        disabled: false,
        level: "1",
        posinset: "2",
        setsize: "3",
        depth: "0",
    };
    let guide_md_props = tree_view::TreeItemProps {
        value: "guide.md",
        selected: false,
        disabled: true,
        level: "2",
        posinset: "1",
        setsize: "1",
        depth: "1",
    };
    let readme_props = tree_view::TreeItemProps {
        value: "README.md",
        selected: false,
        disabled: false,
        level: "1",
        posinset: "3",
        setsize: "3",
        depth: "0",
    };
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
                        src_props,
                        vec![],
                        vec![
                            tree_view::branch_control(
                                open,
                                src_props,
                                vec![],
                                vec![
                                    tree_view::branch_indicator(
                                        open,
                                        src_props,
                                        vec![],
                                        vec![text("▾")],
                                    ),
                                    tree_view::branch_text(
                                        open,
                                        src_props,
                                        vec![],
                                        vec![text("src")],
                                    ),
                                ],
                            ),
                            tree_view::branch_content(
                                open,
                                src_props,
                                vec![],
                                vec![
                                    tree_view::branch_indent_guide(src_props, vec![], vec![]),
                                    tree_view::item(
                                        lib_rs_props,
                                        vec![],
                                        vec![
                                            tree_view::item_indicator(
                                                lib_rs_props,
                                                vec![],
                                                vec![text("✓")],
                                            ),
                                            tree_view::item_text(
                                                lib_rs_props,
                                                vec![],
                                                vec![text("lib.rs")],
                                            ),
                                        ],
                                    ),
                                ],
                            ),
                        ],
                    ),
                    tree_view::branch(
                        closed,
                        docs_props,
                        vec![],
                        vec![
                            tree_view::branch_control(
                                closed,
                                docs_props,
                                vec![],
                                vec![
                                    tree_view::branch_indicator(
                                        closed,
                                        docs_props,
                                        vec![],
                                        vec![text("▸")],
                                    ),
                                    tree_view::branch_text(
                                        closed,
                                        docs_props,
                                        vec![],
                                        vec![text("docs")],
                                    ),
                                ],
                            ),
                            tree_view::branch_content(
                                closed,
                                docs_props,
                                vec![],
                                vec![
                                    tree_view::branch_indent_guide(docs_props, vec![], vec![]),
                                    tree_view::item(
                                        guide_md_props,
                                        vec![],
                                        vec![
                                            tree_view::item_indicator(
                                                guide_md_props,
                                                vec![],
                                                vec![],
                                            ),
                                            tree_view::item_text(
                                                guide_md_props,
                                                vec![],
                                                vec![text("guide.md")],
                                            ),
                                        ],
                                    ),
                                ],
                            ),
                        ],
                    ),
                    tree_view::item(
                        readme_props,
                        vec![],
                        vec![
                            tree_view::item_indicator(readme_props, vec![], vec![]),
                            tree_view::item_text(readme_props, vec![], vec![text("README.md")]),
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
