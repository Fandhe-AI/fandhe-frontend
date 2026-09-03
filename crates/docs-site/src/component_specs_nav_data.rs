//! イシュー #947（部品ページ充填 — Navigation / Data Display 系）の原稿データ。
//!
//! # 役割・呼び出し文脈
//!
//! [`crate::component_page::COMPONENT_SPECS`] から `path -> ComponentPageSpec`
//! の 1 タプルとして参照される定数群。[`crate::component_page`] 側の合成規則
//! （Demo → Features → Anatomy → API Reference → Examples → Accessibility の
//! 6 節、Anatomy・`data-*` 属性表・CSS 変数表は機械導出）に対し、本モジュールは
//! 原稿供給が必要な 5 フィールド（`features`/`arguments`/`examples`/
//! `keyboard`/`aria`）のみを埋める。
//!
//! # 一次情報
//!
//! `features`/`arguments`/`keyboard`/`aria` の各行は `crates/pre-styled-ui/src/`
//! （必要に応じて `crates/headless-ui/src/`）の実ソースを一次情報とし、各定数の
//! doc コメントに `file:line` 形式で根拠を付す。根拠を示せない行は掲載しない
//! （固有の ARIA/キーボード操作を持たない部品は、その旨を明示する 1 行のみを
//! `aria` に置く。`docs/design/docs-site-component-pages.md` §10・
//! `.claude/rules/out-of-scope-tracking.md` の「推測で補完しない」方針）。
//!
//! `examples` のレンダラは `fandhe_frontend_pre_styled_ui`（および、
//! `crates/docs-site` が直接依存しない headless-ui 型は
//! `fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui` 経由、
//! `docs/design/docs-site-component-pages.md` イシュー #693 方針）の公開 API
//! のみを呼び出して組み立てる。`showcase.rs`（イシュー #941 のレジストリ）の
//! 私有関数は再利用しない（#947 実装計画 §4「`showcase.rs` の私有関数は
//! 再利用不可」）。
//!
//! # セキュリティ不変条件（REQ-1）
//!
//! すべてのデータは `&'static str` リテラルであり、[`crate::component_page`]
//! 側で `fandhe_frontend_core::text()` 経由（既定エスケープ）にのみ出力される。
//! 本モジュールは `raw_html()` および HTML 文字列の直接組み立て
//! （`format!("<td>{}</td>", …)`）を一切使わない。Examples レンダラも
//! ノード木 API のみで組み立て、`docs-` 接頭辞の class を持ち込まない
//! （`tests/site_css_contract.rs::component_page_render_introduces_no_class_outside_the_contract`
//! が層 1 (c) 方向で回帰として固定する）。

use fandhe_frontend_core::{el, text, Node};
use fandhe_frontend_pre_styled_ui::{
    alert, avatar, badge, breadcrumb, callout, card, carousel, color_swatch, data_list,
    empty_state, icon, image, json_tree_view, marquee, pagination, progress, scroll_area,
    separator, skeleton, spinner, splitter, stat, status, steps, tab_nav, table, tag, timeline,
    tree_view, AlertProps, ColorPalette, Orientation, Size,
};

use crate::component_page::{ArgRow, AriaRow, ComponentPageSpec, ExampleEntry, KeyRow};

// ---------------------------------------------------------------------
// Data Display（本文明示 15 件）
// ---------------------------------------------------------------------

/// `crates/pre-styled-ui/src/alert.rs`（`root` が `role="alert"` を固定
/// 付与、`AlertStatus`/`AlertVariant`/`Size` の 3 軸、イシュー #1553）。
fn ex_alert() -> Node {
    let props = AlertProps {
        status: alert::AlertStatus::Warning,
        ..AlertProps::default()
    };
    alert::root(
        &props,
        vec![],
        vec![
            alert::indicator(vec![], vec![]),
            alert::content(
                vec![],
                vec![
                    alert::title(vec![], vec![text("Heads up")]),
                    alert::description(vec![], vec![text("Something needs attention")]),
                ],
            ),
        ],
    )
}

pub(crate) const ALERT: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "AlertStatus（Info/Success/Warning/Error/Neutral、crates/pre-styled-ui/src/alert.rs）で 5 種の状態色を切り替える（イシュー #1553）",
        "AlertVariant（Subtle/Surface/Solid/Outline、既定 Subtle）で見た目のトーンを切り替える（イシュー #1553）",
        "size（Xs〜Xl、既定 Md）でパディング・フォントサイズ・indicator サイズを切り替える（イシュー #1553）",
        "root に WAI-ARIA live region の role=\"alert\" を状態に関わらず固定付与する（alert.rs）",
        "indicator/content/title/description の 4 パーツで見出し・本文を構造化できる（alert.rs）",
    ],
    arguments: &[
        ArgRow {
            name: "status",
            kind: "AlertStatus",
            default: "Info",
            description: "見た目の状態色（alert.rs、#[default] は Info）。",
        },
        ArgRow {
            name: "variant",
            kind: "AlertVariant",
            default: "Subtle",
            description: "見た目のトーン（Subtle/Surface/Solid/Outline、イシュー #1553）。",
        },
        ArgRow {
            name: "size",
            kind: "Size",
            default: "Md",
            description: "サイズ（Xs〜Xl、イシュー #1553）。",
        },
    ],
    examples: &[ExampleEntry {
        title: "Warning",
        description: "Warning 状態の Alert を indicator + title/description で組み立てた例です。",
        render: ex_alert,
    }],
    keyboard: &[],
    aria: &[AriaRow {
        attribute: "role=\"alert\"",
        description: "WAI-ARIA live region。status の値に関わらず固定で付与される（alert.rs）。",
    }],
    demo: None,
};

fn ex_avatar() -> Node {
    avatar::root(
        &avatar::AvatarProps::default(),
        vec![],
        vec![avatar::fallback(
            avatar::ImageStatus::Error,
            vec![],
            vec![text("FT")],
        )],
    )
}

pub(crate) const AVATAR: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "AvatarShape（Circle/Rounded/Square、crates/pre-styled-ui/src/avatar.rs）で外形を切り替える",
        "AvatarVariant（Subtle/Solid/Outline、イシュー #1554 で追加）で見た目バリアントを切り替える",
        "ColorPalette（6 値、既定 Neutral、イシュー #1554 で追加）で colorPalette 軸を切り替える",
        "ImageStatus に連動して image/fallback パーツの表示・非表示を CSS の [data-state=\"hidden\"] で切り替える",
        "image パーツは alt テキストを必須引数として要求する（avatar.rs 内 image 再エクスポート）",
    ],
    arguments: &[
        ArgRow {
            name: "size",
            kind: "Size",
            default: "Md",
            description: "サイズ（avatar.rs の AvatarProps、#[default] は Md。イシュー #1554 で 24/32/40/48/56px へ是正）。",
        },
        ArgRow {
            name: "shape",
            kind: "AvatarShape",
            default: "Circle",
            description: "外形（avatar.rs の AvatarShape、#[default] は Circle）。",
        },
        ArgRow {
            name: "variant",
            kind: "AvatarVariant",
            default: "Subtle",
            description: "見た目バリアント（avatar.rs の AvatarVariant、#[default] は Subtle。イシュー #1554 で追加）。",
        },
        ArgRow {
            name: "palette",
            kind: "ColorPalette",
            default: "Neutral",
            description: "colorPalette 軸（avatar.rs の AvatarProps、#[default] は Neutral。イシュー #1554 で追加）。",
        },
    ],
    examples: &[ExampleEntry {
        title: "Fallback",
        description: "画像読み込み失敗（ImageStatus::Error）時のイニシャル表示例です。",
        render: ex_avatar,
    }],
    keyboard: &[],
    aria: &[AriaRow {
        attribute: "(該当なし)",
        description: "root/image/fallback は固有の role/aria-* を出力しない。image パーツの alt テキストのみが代替情報を提供する（avatar.rs 全文で role/aria-* を grep しても 0 件）。",
    }],
    demo: None,
};

fn ex_badge() -> Node {
    badge::badge(
        &badge::BadgeProps {
            variant: badge::BadgeVariant::Solid,
            ..badge::BadgeProps::default()
        },
        vec![],
        vec![text("New")],
    )
}

pub(crate) const BADGE: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "BadgeVariant（Solid/Subtle/Outline/Surface、crates/pre-styled-ui/src/badge.rs。イシュー #1555 で Surface を追加）で塗り方を切り替える",
        "colorPalette 軸（badge.rs の BadgeProps）でセマンティック色を選択する",
        "Subtle/Outline/Surface は 6 役割 palette の淡色トークンを消費する（badge.rs、イシュー #1555）",
        "role/aria-* は付与しない最小サブセット（badge.rs モジュール冒頭）",
    ],
    arguments: &[
        ArgRow {
            name: "variant",
            kind: "BadgeVariant",
            default: "Subtle",
            description: "塗り方（badge.rs の BadgeVariant、#[default] は Subtle。イシュー #1555 で Surface を追加）。",
        },
        ArgRow {
            name: "size",
            kind: "Size",
            default: "Md",
            description: "サイズ（badge.rs の BadgeProps）。",
        },
        ArgRow {
            name: "palette",
            kind: "ColorPalette",
            default: "Accent",
            description: "colorPalette 軸（badge.rs の BadgeProps）。",
        },
    ],
    examples: &[ExampleEntry {
        title: "Solid",
        description: "Solid variant の Badge です。",
        render: ex_badge,
    }],
    keyboard: &[],
    aria: &[AriaRow {
        attribute: "(該当なし)",
        description: "chakra-ui v3 準拠の最小サブセットとして role/aria-* を付与しない（badge.rs モジュール冒頭）。",
    }],
    demo: None,
};

/// `crates/pre-styled-ui/src/callout.rs:68`（`CalloutVariant` 3 バリアント）・
/// 同 `:300`（`root` が `role`/`aria-*` を一切付与しない）。
fn ex_callout() -> Node {
    let props = callout::CalloutProps {
        variant: callout::CalloutVariant::Surface,
        ..callout::CalloutProps::default()
    };
    callout::root(
        &props,
        vec![],
        vec![
            callout::icon(vec![], vec![]),
            callout::text(vec![], vec![text("Heads up: this is supplementary info")]),
        ],
    )
}

pub(crate) const CALLOUT: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "CalloutVariant（Soft/Surface/Outline、crates/pre-styled-ui/src/callout.rs:68-76）で見た目を切り替える",
        "colorPalette 軸（callout.rs:94-101）でセマンティック色を選択する",
        "size（xs〜xl、callout.rs:94-101）で padding / gap / 角丸 / 文字サイズが root の `--fandhe-callout-*` custom property を通じて連動する（イシュー #1556）",
        "root/icon/text の 3 パーツで補足情報を構造化できる（callout.rs 全文参照）",
        "alert と異なり role を一切付与しない静的な補足表示部品（callout.rs:1-18 module doc）",
    ],
    arguments: &[
        ArgRow {
            name: "variant",
            kind: "CalloutVariant",
            default: "Soft",
            description: "見た目（callout.rs:68-76、#[default] は Soft）。",
        },
        ArgRow {
            name: "size",
            kind: "Size",
            default: "Md",
            description: "サイズ（callout.rs:94-101）。padding / gap / 角丸 / 文字サイズが連動する（root の `--fandhe-callout-*`、イシュー #1556）。",
        },
        ArgRow {
            name: "palette",
            kind: "ColorPalette",
            default: "Accent",
            description: "colorPalette 軸（callout.rs:94-101）。",
        },
    ],
    examples: &[ExampleEntry {
        title: "Surface",
        description: "Surface variant の Callout を icon + text で組み立てた例です。",
        render: ex_callout,
    }],
    keyboard: &[],
    aria: &[AriaRow {
        attribute: "(該当なし)",
        description: "固有の role/aria-* を出力しない。alert と異なり live region ではないため支援技術へ割り込み通知をしない（callout.rs module doc 参照）。",
    }],
    demo: None,
};

fn ex_card() -> Node {
    let props = card::CardProps {
        variant: card::CardVariant::Elevated,
        ..card::CardProps::default()
    };
    card::root(
        props,
        vec![],
        vec![
            card::header(vec![], vec![card::title(vec![], vec![text("Title")])]),
            card::body(vec![], vec![text("Body")]),
            card::footer(vec![], vec![text("Footer")]),
        ],
    )
}

pub(crate) const CARD: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "CardVariant（Elevated/Outline/Subtle、crates/pre-styled-ui/src/card.rs:80-106）で見た目を切り替える",
        "size（xs〜xl、card.rs:142 以降）で padding / 角丸 / title の文字サイズが root の `--fandhe-card-*` custom property を通じて連動する（イシュー #1557）",
        "header/body/footer/title/description の 6 パーツでレイアウトを構造化する（card.rs 全文参照）",
        "純粋なレイアウトコンテナのため role/aria-* は付与しない（card.rs:4）",
    ],
    arguments: &[
        ArgRow {
            name: "variant",
            kind: "CardVariant",
            default: "Outline",
            description: "見た目（card.rs:80-106、#[default] は Outline）。",
        },
        ArgRow {
            name: "size",
            kind: "Size",
            default: "Md",
            description: "サイズ（card.rs:142 以降）。padding / 角丸 / title の文字サイズが連動する（root の `--fandhe-card-*`、イシュー #1557）。",
        },
    ],
    examples: &[ExampleEntry {
        title: "Elevated card",
        description: "header/body/footer を組み合わせた Elevated variant の例です。",
        render: ex_card,
    }],
    keyboard: &[],
    aria: &[AriaRow {
        attribute: "(該当なし)",
        description: "純粋なレイアウトコンテナであり role/aria-* を付与しない（card.rs:4）。",
    }],
    demo: None,
};

fn ex_data_list() -> Node {
    data_list::root(
        data_list::DataListProps {
            orientation: data_list::DataListOrientation::Horizontal,
            ..data_list::DataListProps::default()
        },
        vec![],
        vec![data_list::item(
            vec![],
            vec![
                data_list::item_label(vec![], vec![text("Name")]),
                data_list::item_value(vec![], vec![text("Alice")]),
            ],
        )],
    )
}

fn ex_data_list_bold() -> Node {
    data_list::root(
        data_list::DataListProps {
            variant: data_list::DataListVariant::Bold,
            ..data_list::DataListProps::default()
        },
        vec![],
        vec![data_list::item(
            vec![],
            vec![
                data_list::item_label(vec![], vec![text("Name")]),
                data_list::item_value(vec![], vec![text("Alice")]),
            ],
        )],
    )
}

pub(crate) const DATA_LIST: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "DataListOrientation（Vertical/Horizontal、crates/pre-styled-ui/src/data_list.rs:106-113）でラベル・値の並びを切り替える",
        "DataListVariant（Subtle/Bold、data_list.rs:129-136、イシュー #1559）でラベル・値の強調配色を切り替える",
        "size（Size::Xs〜Xl、既定 Md、data_list.rs recipe() の size_variants）で gap・font-size を段階的に切り替える",
        "item/item-label/item-value の 3 パーツで dl/dt/dd 構造を組み立てる",
        "orientation/variant/size の伝搬は root の CSS custom property 経由（通常の CSS 継承、data_list.rs モジュール doc）",
    ],
    arguments: &[
        ArgRow {
            name: "orientation",
            kind: "DataListOrientation",
            default: "Vertical",
            description: "並び方向（#[default] は Vertical）。",
        },
        ArgRow {
            name: "variant",
            kind: "DataListVariant",
            default: "Subtle",
            description: "見た目 variant（イシュー #1559。#[default] は Subtle。ラベル muted・値 fg / Bold はラベル fg+medium 太字・値 muted）。",
        },
        ArgRow {
            name: "size",
            kind: "Size",
            default: "Md",
            description: "サイズ variant（イシュー #1559。Xs〜Xl の 5 段、既定 Md）。",
        },
    ],
    examples: &[
        ExampleEntry {
            title: "Horizontal",
            description: "ラベル・値を横並び表示する Horizontal variant の例です。",
            render: ex_data_list,
        },
        ExampleEntry {
            title: "Bold",
            description: "ラベルを強調表示する Bold variant の例です。",
            render: ex_data_list_bold,
        },
    ],
    keyboard: &[],
    aria: &[AriaRow {
        attribute: "(該当なし)",
        description: "dl/dt/dd のネイティブ意味論のみで固有の role/aria-* は出力しない（data_list.rs 全文で role/aria-* を grep しても 0 件）。",
    }],
    demo: None,
};

fn ex_empty_state() -> Node {
    empty_state::root(
        &empty_state::EmptyStateProps::default(),
        vec![],
        vec![empty_state::content(
            vec![],
            vec![
                empty_state::title(vec![], vec![text("No results")]),
                empty_state::description(vec![], vec![text("Try a different search.")]),
            ],
        )],
    )
}

pub(crate) const EMPTY_STATE: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "content/indicator/title/description/actions の 5 パーツで空状態の掲示を構造化する（crates/pre-styled-ui/src/empty_state.rs の recipe 関数）",
        "size variant（既定 Md）が root の `--fandhe-empty-state-*` custom property 経由で padding・gap・indicator/title/description の文字サイズを連動させる（empty_state.rs の recipe 関数、イシュー #1560）",
        "aria-* は付与しない（empty_state.rs 冒頭 doc コメント）",
    ],
    arguments: &[ArgRow {
        name: "size",
        kind: "Size",
        default: "Md",
        description: "root の custom property（`--fandhe-empty-state-*`）経由で padding・gap・indicator/title/description の文字サイズを連動させるサイズ（empty_state.rs の recipe 関数、イシュー #1560）。",
    }],
    examples: &[ExampleEntry {
        title: "Basic",
        description: "title + description のみで組み立てた最小構成の例です。",
        render: ex_empty_state,
    }],
    keyboard: &[],
    aria: &[AriaRow {
        attribute: "(該当なし)",
        description: "aria-* は付与しない（empty_state.rs:5、root_has_no_role_attribute 回帰テストあり）。",
    }],
    demo: None,
};

/// JsonTreeView の Demo と同じ構成方針（`crates/docs-site/src/showcase.rs`
/// の `json_tree_view_section` を参考に、より小さいデータで再構築）。
/// `tree_view::root`/`label`/`tree` は json-tree-view/tree-view の両方が
/// 共有する headless anatomy であり、両モジュールとも `showcase.rs` と同じ
/// 呼び出し経路（`json_tree_view::expanded_to_depth`/`render_json` →
/// `tree_view::root`）を使う（json_tree_view.rs:42-49 の re-export 宣言）。
fn ex_json_tree_view() -> Node {
    let data = json_tree_view::JsonValue::Object(vec![
        (
            "name".to_string(),
            json_tree_view::JsonValue::String("fandhe-frontend".to_string()),
        ),
        ("stable".to_string(), json_tree_view::JsonValue::Bool(true)),
    ]);
    let tree = json_tree_view::expanded_to_depth(&data, 1);
    tree_view::root(
        Size::Md,
        vec![],
        vec![
            tree_view::label(vec![], vec![text("Package metadata")]),
            tree_view::tree(
                Some("Package metadata"),
                None,
                vec![],
                vec![json_tree_view::render_json(&tree, &data)],
            ),
        ],
    )
}

pub(crate) const JSON_TREE_VIEW: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "key/value の 2 anatomy パーツを headless-ui からそのまま再エクスポートする（crates/pre-styled-ui/src/json_tree_view.rs:7）",
        "expanded_to_depth で ark-ui の defaultExpandedDepth 相当の初期展開状態を決定的に作る（json_tree_view.rs 冒頭 doc、showcase.rs:2057-2058 の利用例）",
        "値の型（string/number/bool/null/array/object）ごとに [data-scope=\"json-tree-view\"][data-part=\"value\"][data-kind=\"...\"] で配色を切り替える（json_tree_view.rs:111-113）",
    ],
    arguments: &[],
    examples: &[ExampleEntry {
        title: "Package metadata",
        description: "name/stable の 2 フィールドを持つ JSON を深さ 1 まで展開した例です。",
        render: ex_json_tree_view,
    }],
    keyboard: &[],
    aria: &[AriaRow {
        attribute: "role=\"tree\" / role=\"treeitem\"",
        description: "headless-ui の TreeView anatomy を再利用するため role=\"tree\"/role=\"treeitem\" を持つ（crates/pre-styled-ui/src/tree_view.rs:45 の rustdoc 記述、json_tree_view.rs:7-8 が同 anatomy を再エクスポート）。",
    }],
    demo: None,
};

fn ex_progress() -> Node {
    use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::progress::Progress;
    use fandhe_frontend_pre_styled_ui::progress::ProgressProps;
    let p = Progress::new(0.0, 100.0, Some(65.0), Orientation::Horizontal);
    progress::root(
        &p,
        &ProgressProps::default(),
        Some("65%"),
        vec![],
        vec![
            p.label(vec![], vec![fandhe_frontend_core::text("Upload")]),
            p.value_text(vec![], vec![fandhe_frontend_core::text("65%")]),
            p.track(vec![], vec![progress::range(&p, vec![])]),
        ],
    )
}

fn ex_progress_circle() -> Node {
    use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::progress::Progress;
    use fandhe_frontend_pre_styled_ui::progress::ProgressProps;
    let p = Progress::new(0.0, 100.0, Some(65.0), Orientation::Horizontal);
    progress::root(
        &p,
        &ProgressProps::default(),
        Some("65%"),
        vec![],
        vec![p.circle(
            vec![],
            vec![
                p.circle_track(vec![], vec![]),
                p.circle_range(vec![], vec![]),
            ],
        )],
    )
}

pub(crate) const PROGRESS: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "track/range（linear）と circle/circle-track/circle-range（circular）はいずれも headless の inherent メソッドをそのまま呼ばせる契約（crates/pre-styled-ui/src/progress.rs テスト caller_headless_track_and_circle_parts_render_without_wrapper）",
        "value が None（indeterminate）のとき [data-state=\"indeterminate\"] でアニメーション（linear は横スライド・circular は回転）を付与し、prefers-reduced-motion: reduce で停止する",
        "ProgressProps（size/variant/color-palette の 3 軸）を root へ付与する。styled range() が --fandhe-progress-percent を determinate 時のみ付与する",
    ],
    arguments: &[
        ArgRow {
            name: "props",
            kind: "&ProgressProps",
            default: "ProgressProps::default()",
            description: "size（既定 Md）/variant（既定 Outline）/palette（既定 Accent）の 3 軸をまとめた設定（progress.rs）。",
        },
        ArgRow {
            name: "aria_valuetext",
            kind: "Option<&str>",
            default: "None",
            description: "aria-valuetext へ渡す表示用テキスト（progress.rs）。",
        },
    ],
    examples: &[
        ExampleEntry {
            title: "Determinate (linear)",
            description: "value=65 の determinate linear progress（Track/Range）の例です。",
            render: ex_progress,
        },
        ExampleEntry {
            title: "Determinate (circular)",
            description: "value=65 の determinate circular progress（SVG）の例です。",
            render: ex_progress_circle,
        },
    ],
    keyboard: &[],
    aria: &[AriaRow {
        attribute: "aria-valuetext",
        description: "呼び出し側が渡した aria_valuetext を root（headless progress.root への委譲）へ出力する（progress.rs）。",
    }],
    demo: None,
};

fn ex_skeleton() -> Node {
    skeleton::skeleton(
        &skeleton::SkeletonProps {
            variant: skeleton::SkeletonVariant::Circle,
            ..Default::default()
        },
        vec![],
    )
}

pub(crate) const SKELETON: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "SkeletonVariant（Text/Circle/Rect、crates/pre-styled-ui/src/skeleton.rs:138-149）で占位形状を切り替える",
        "SkeletonAnimation（Pulse/Shine/None、skeleton.rs:169-181、イシュー #1566）で第 2 軸のアニメーション種別を切り替える",
        "常に aria-hidden=\"true\" を固定付与する（skeleton.rs:364-369）",
        "呼び出し側が偽装した aria-hidden（大文字小文字問わず）も除去し常時 true へ一本化する（skeleton.rs:364-369、回帰テストは skeleton.rs:429-436）",
    ],
    arguments: &[
        ArgRow {
            name: "variant",
            kind: "SkeletonVariant",
            default: "Text",
            description: "占位形状（skeleton.rs:138-149、#[default] は Text）。",
        },
        ArgRow {
            name: "animation",
            kind: "SkeletonAnimation",
            default: "Pulse",
            description: "アニメーション種別（skeleton.rs:169-181、#[default] は Pulse、イシュー #1566）。",
        },
    ],
    examples: &[ExampleEntry {
        title: "Circle",
        description: "アバター等の占位に使う Circle variant の例です。",
        render: ex_skeleton,
    }],
    keyboard: &[],
    aria: &[AriaRow {
        attribute: "aria-hidden=\"true\"",
        description: "常に固定付与される（呼び出し側の偽装値は除去、skeleton.rs:364-369、回帰テストは skeleton.rs:429-436）。",
    }],
    demo: None,
};

fn ex_spinner() -> Node {
    spinner::spinner(&spinner::SpinnerProps {
        label: "Loading products",
        ..spinner::SpinnerProps::default()
    })
}

pub(crate) const SPINNER: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "role=\"status\" + aria-label（既定 \"Loading\"）でスクリーンリーダーへ読み込み中を伝える（crates/pre-styled-ui/src/spinner.rs）",
        "spinner_decorative（別関数）は role/aria-label を持たず aria-hidden=\"true\" のみを付与する（spinner.rs）",
        "size・colorPalette の 2 軸でサイズとセマンティック色を選択する（spinner.rs 冒頭）",
        "上・右 2 辺の弧で描画し、トラックは既定で透明（イシュー #1567、chakra-ui 基準）。--fandhe-spinner-track-color / --fandhe-spinner-thickness / --fandhe-spinner-duration の custom property で線色・線幅・回転速度を上書きできる",
        "prefers-reduced-motion: reduce 環境では回転アニメーションを停止する（イシュー #1567）",
    ],
    arguments: &[
        ArgRow {
            name: "size",
            kind: "Size",
            default: "Md",
            description: "サイズ variant（spinner.rs）。chakra-ui 5 段基準（xs=0.75rem/sm=1rem/md=1.25rem/lg=2rem/xl=2.5rem、イシュー #1567）。",
        },
        ArgRow {
            name: "palette",
            kind: "ColorPalette",
            default: "Accent",
            description: "colorPalette 軸（spinner.rs、イシュー #606）。",
        },
        ArgRow {
            name: "label",
            kind: "&str",
            default: "\"Loading\"",
            description: "aria-label に渡すラベル文字列（spinner.rs）。",
        },
    ],
    examples: &[ExampleEntry {
        title: "Custom label",
        description: "aria-label をカスタマイズした Spinner の例です。",
        render: ex_spinner,
    }],
    keyboard: &[],
    aria: &[AriaRow {
        attribute: "role=\"status\" + aria-label",
        description: "読み込み中であることをスクリーンリーダーへ伝える（spinner.rs）。",
    }],
    demo: None,
};

fn ex_stat() -> Node {
    stat::root(
        Size::Md,
        vec![],
        vec![
            stat::label(vec![], vec![text("Revenue")]),
            stat::value_text(
                vec![],
                vec![text("1,234"), stat::value_unit(vec![], vec![text("USD")])],
            ),
            stat::help_text(vec![], vec![stat::up_indicator(vec![]), text("12%")]),
        ],
    )
}

pub(crate) const STAT: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "label/value-text/value-unit/help-text/up-indicator/down-indicator の 6 パーツで指標表示を構造化する（crates/pre-styled-ui/src/stat.rs:314-362）",
        "up-indicator/down-indicator は装飾用途のため aria-hidden=\"true\" を固定付与する（stat.rs:17, 348-362）",
        "呼び出し側が aria-hidden を渡してもフレームワーク値の後に連結される（stat.rs:442-449）",
    ],
    arguments: &[ArgRow {
        name: "size",
        kind: "Size",
        default: "Md",
        description: "root（dl）のサイズ（xs〜xl、既定 md。chakra-ui の sm/md/lg は本実装の Sm/Md/Lg に対応、stat.rs:158-256）。",
    }],
    examples: &[ExampleEntry {
        title: "Revenue",
        description: "value-unit と up-indicator を組み合わせた指標表示の例です。",
        render: ex_stat,
    }],
    keyboard: &[],
    aria: &[AriaRow {
        attribute: "aria-hidden=\"true\"（up-indicator/down-indicator のみ）",
        description: "装飾用途の増減インジケータに固定付与される（stat.rs:17, 348-362）。root/label/value-text 自体は固有の ARIA を出力しない。",
    }],
    demo: None,
};

fn ex_status() -> Node {
    status::root(
        &status::StatusProps {
            palette: ColorPalette::Success,
            ..status::StatusProps::default()
        },
        vec![],
        vec![status::indicator(vec![]), text("Online")],
    )
}

pub(crate) const STATUS: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "colorPalette 軸（status.rs の `StatusProps::palette` フィールド）でセマンティック色を選択する",
        "size 軸（Xs〜Xl、status.rs の `StatusProps::size` フィールド）でドット径（--fandhe-status-dot-size）と文字サイズが連動する",
        "root/indicator の 2 パーツのみで構成する最小部品",
        "role=\"status\"（WAI-ARIA live region）は付与しない設計（status.rs のクレート先頭 doc コメント、非同期の状態遷移がある場合は呼び出し側が明示的に足す）",
    ],
    arguments: &[
        ArgRow {
            name: "size",
            kind: "Size",
            default: "Md",
            description: "サイズ軸（status.rs の `StatusProps::size` フィールド）。ドット径と文字サイズが連動する。",
        },
        ArgRow {
            name: "palette",
            kind: "ColorPalette",
            default: "Accent",
            description: "colorPalette 軸（status.rs の `StatusProps::palette` フィールド）。",
        },
    ],
    examples: &[ExampleEntry {
        title: "Success",
        description: "Success パレットで \"Online\" を表示する例です。",
        render: ex_status,
    }],
    keyboard: &[],
    aria: &[AriaRow {
        attribute: "(該当なし)",
        description: "role=\"status\" は付与しない設計。非同期の状態遷移を伴う場合は呼び出し側が明示的に role/aria-live を足す契約（status.rs のクレート先頭 doc コメント、テストで role= の非出現を固定）。",
    }],
    demo: None,
};

fn ex_table() -> Node {
    table::root(
        table::TableProps {
            striped: true,
            ..table::TableProps::default()
        },
        vec![],
        vec![
            table::header(
                vec![],
                vec![table::row(
                    vec![],
                    vec![table::column_header(vec![], vec![text("Name")])],
                )],
            ),
            table::body(
                vec![],
                vec![table::row(
                    vec![],
                    vec![table::cell(vec![], vec![text("Alice")])],
                )],
            ),
        ],
    )
}

fn ex_table_scroll_area() -> Node {
    table::scroll_area(
        vec![("style", "--fandhe-table-scroll-max-height: 8rem")],
        vec![table::root(
            table::TableProps {
                sticky_header: true,
                ..table::TableProps::default()
            },
            vec![],
            vec![
                table::header(
                    vec![],
                    vec![table::row(
                        vec![],
                        vec![table::column_header(vec![], vec![text("Name")])],
                    )],
                ),
                table::body(
                    vec![],
                    vec![
                        table::row(vec![], vec![table::cell(vec![], vec![text("Alice")])]),
                        table::row(vec![], vec![table::cell(vec![], vec![text("Bob")])]),
                        table::row(vec![], vec![table::cell(vec![], vec![text("Carol")])]),
                    ],
                ),
            ],
        )],
    )
}

pub(crate) const TABLE: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "TableVariant（Line/Outline、crates/pre-styled-ui/src/table.rs:172-186）で外枠・区切り線を切り替える（イシュー #1572 で Outline の行罫線・ヘッダー背景を chakra-ui/Radix Themes 基準へ是正）",
        "size（Size、Xs〜Xl の 5 段）でセルの padding/font-size を切り替える（padding は --fandhe-space-* トークン、イシュー #1572）",
        "striped（bool）で本文行の背景を交互に変える（table.rs「striped の実装」節）",
        "sticky_header（bool、イシュー #1571）で column-header（th）を position: sticky にする（table.rs「sticky ヘッダーの実装」節）",
        "scroll_area（イシュー #1572、chakra Table.ScrollArea 相当）で root を overflow: auto のスクロール枠に包み、sticky_header と組み合わせて見出し行を固定できる（table.rs「scroll-area パーツ」節）",
        "caption は font-weight: medium・font-size: xs・text-align: inherit（chakra-ui 基準、イシュー #1572）",
        "column_header は scope=\"col\" を関数側で固定し呼び出し側の偽装を除去する（table.rs セキュリティ不変条件節、COLUMN_HEADER_RESERVED）",
    ],
    arguments: &[
        ArgRow {
            name: "variant",
            kind: "TableVariant",
            default: "Line",
            description: "外枠・区切り線の見た目（table.rs「variant について」節）。",
        },
        ArgRow {
            name: "size",
            kind: "Size",
            default: "Md",
            description: "セルの padding/font-size（table.rs「variant について」節）。",
        },
        ArgRow {
            name: "striped",
            kind: "bool",
            default: "false",
            description: "縞模様表示の有無（table.rs「striped の実装」節）。",
        },
        ArgRow {
            name: "sticky_header",
            kind: "bool",
            default: "false",
            description: "column-header（th）を position: sticky にする（イシュー #1571、table.rs「sticky ヘッダーの実装」節）。",
        },
    ],
    examples: &[
        ExampleEntry {
            title: "Striped",
            description: "striped=true・header/body を組み合わせた例です。",
            render: ex_table,
        },
        ExampleEntry {
            title: "Scroll area",
            description: "scroll_area で包み、sticky_header=true と組み合わせた例です（イシュー #1572）。",
            render: ex_table_scroll_area,
        },
    ],
    keyboard: &[],
    aria: &[AriaRow {
        attribute: "scope=\"col\"（column-header）",
        description: "column_header が固定付与するテーブル見出しの意味論属性（呼び出し側の偽装は除去、table.rs セキュリティ不変条件節）。role/aria-* 自体はネイティブ table 要素の意味論に委ねており固有の出力はない。",
    }],
    demo: None,
};

fn ex_tag() -> Node {
    tag::root(
        &tag::TagProps {
            variant: tag::TagVariant::Outline,
            ..tag::TagProps::default()
        },
        vec![],
        vec![
            tag::label(vec![], vec![text("beta")]),
            tag::close_trigger(Some("Remove beta tag"), vec![], vec![]),
        ],
    )
}

pub(crate) const TAG: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "TagVariant（Solid/Subtle/Outline/Surface、crates/pre-styled-ui/src/tag.rs。イシュー #1573 で Surface を追加）で塗り方を切り替える",
        "colorPalette 軸（tag.rs の TagProps）でセマンティック色を選択する",
        "Subtle/Outline/Surface は 6 役割 palette の淡色トークンを消費する（tag.rs、イシュー #1573）",
        "close_trigger（<button type=\"button\">）で削除可能な Tag を構成できる（tag.rs）",
        "close_trigger は hover 面・キーボードフォーカスリングを持つ（イシュー #1573。Solid variant ではリング色を --fandhe-palette-fg へ切り替え、背景との同化を避ける）",
        "close_trigger 自体は children/aria-label を持たないため呼び出し側が視覚内容とアクセシブルネームを渡す責務を持つ（tag.rs）",
    ],
    arguments: &[
        ArgRow {
            name: "variant",
            kind: "TagVariant",
            default: "Subtle",
            description: "塗り方（tag.rs の TagVariant、#[default] は Subtle。イシュー #1573 で Surface を追加）。",
        },
        ArgRow {
            name: "size",
            kind: "Size",
            default: "Md",
            description: "サイズ（tag.rs の TagProps）。",
        },
        ArgRow {
            name: "palette",
            kind: "ColorPalette",
            default: "Accent",
            description: "colorPalette 軸（tag.rs の TagProps）。",
        },
    ],
    examples: &[ExampleEntry {
        title: "Removable",
        description: "close_trigger を付与した削除可能な Tag の例です。",
        render: ex_tag,
    }],
    keyboard: &[],
    aria: &[AriaRow {
        attribute: "(呼び出し側責務)",
        description: "close_trigger の aria-label・視覚内容（×等）は呼び出し側が渡す責務であり、本部品自体は固有の ARIA を固定出力しない（tag.rs）。",
    }],
    demo: None,
};

fn ex_tree_view() -> Node {
    use fandhe_frontend_pre_styled_ui::fandhe_frontend_interactive::dispatch;
    let mut view = tree_view::TreeView::default();
    dispatch(&mut view, "expand", "src");
    let nodes = vec![tree_view::TreeNode::new("src", "src")
        .with_children(vec![tree_view::TreeNode::new("src/lib.rs", "lib.rs")])];
    let children = view.render_nodes(&nodes);
    tree_view::root(
        Size::Md,
        vec![],
        vec![
            tree_view::label(vec![], vec![text("Project files")]),
            tree_view::tree(Some("Project files"), None, vec![], children),
        ],
    )
}

pub(crate) const TREE_VIEW: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "role=\"tree\"/role=\"treeitem\" の WAI-ARIA Tree パターンを headless-ui が提供する（crates/pre-styled-ui/src/tree_view.rs:45）",
        "branch-content の展開・折りたたみは [hidden] 属性と CSS の詳細度制御で表現する（tree_view.rs 参照）",
        "disabled/selected 状態を data-disabled/data-selected で表現しフォーカス可視化と連動する（tree_view.rs 参照）",
        "size（xs/sm/md/lg/xl、既定 md）が行密度・文字サイズを切り替える。root スコープの CSS custom property 経由で子孫パーツへ継承される（イシュー #1578）",
        "hover 面・キーボードフォーカスリングを持つ（イシュー #1578。選択行の背景は hover で洗い流されない）",
    ],
    arguments: &[ArgRow {
        name: "size",
        kind: "Size",
        default: "Md",
        description: "行密度・文字サイズを切り替える（tree_view.rs の root）。",
    }],
    examples: &[ExampleEntry {
        title: "Expanded branch",
        description: "\"src\" ブランチを展開した状態で固定表示する例です。",
        render: ex_tree_view,
    }],
    keyboard: &[],
    aria: &[AriaRow {
        attribute: "role=\"tree\" / role=\"treeitem\"",
        description: "WAI-ARIA Tree View パターンに従い headless-ui が付与する（tree_view.rs:45）。",
    }],
    demo: None,
};

// ---------------------------------------------------------------------
// Data Display（設計 §5 規則 4 による追加 4 件）
// ---------------------------------------------------------------------

fn ex_color_swatch() -> Node {
    color_swatch::color_swatch(
        &color_swatch::ColorSwatchProps {
            value: color_swatch::Color::from_rgb(color_swatch::Rgb::new(0x3b, 0x82, 0xf6)),
            shape: color_swatch::SwatchShape::Circle,
            ..color_swatch::ColorSwatchProps::default()
        },
        vec![],
        vec![],
    )
}

pub(crate) const COLOR_SWATCH: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "SwatchShape（Square/Circle/Rounded）で外形を切り替える（color_swatch.rs `recipe` 節）",
        "検証済み Color 型のみを受け取り、--fd-swatch-color custom property 経由で色を出力する（color_swatch.rs モジュール冒頭「色値は検証済み型経由のみ」節）",
        "呼び出し側の class/style/data-scope/data-part 偽装はすべて除去する（color_swatch.rs テスト caller_class_and_style_attrs_are_dropped_not_duplicated 等）",
        "内側 1px の輪郭リング（box-shadow: inset）で淡色・低アルファ色でも外形が判別できる（イシュー #1558、color_swatch.rs モジュール冒頭「参照サイト比較」節）",
    ],
    arguments: &[
        ArgRow {
            name: "value",
            kind: "Color",
            default: "opaque black",
            description: "表示する色（検証済み型のみ受け取る、color_swatch.rs モジュール冒頭「色値は検証済み型経由のみ」節）。",
        },
        ArgRow {
            name: "size",
            kind: "Size",
            default: "Md",
            description: "サイズ（Xs〜Xl の 5 段、chakra-ui 同名段の実寸に整合。イシュー #1558）。",
        },
        ArgRow {
            name: "shape",
            kind: "SwatchShape",
            default: "Rounded",
            description: "外形（color_swatch.rs `SwatchShape` 節）。",
        },
    ],
    examples: &[ExampleEntry {
        title: "Circle",
        description: "円形の ColorSwatch の例です。",
        render: ex_color_swatch,
    }],
    keyboard: &[],
    aria: &[AriaRow {
        attribute: "(該当なし)",
        description: "純粋な色見本表示であり固有の role/aria-* を出力しない（color_swatch.rs 全文で role/aria-* を grep しても 0 件）。",
    }],
    demo: None,
};

fn ex_icon() -> Node {
    icon::icon(
        &icon::IconProps {
            label: Some("Search"),
            ..icon::IconProps::default()
        },
        vec![],
        vec![el("path", vec![("d", "M12 2L2 22h20z")], vec![])],
    )
}

pub(crate) const ICON: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "label が Some のとき role=\"img\" + aria-label を付与し意味のあるアイコンとして扱う（icon.rs `icon` 節）",
        "label が None（既定）のとき装飾用途とみなし aria-hidden=\"true\" を付与する（icon.rs `icon` 節）",
        "fill=\"currentColor\" を固定付与し祖先の color プロパティで着色する（icon.rs `icon` 節）",
    ],
    arguments: &[
        ArgRow {
            name: "size",
            kind: "Size",
            default: "Md",
            description: "サイズ（Xs〜Xl の 5 段、chakra-ui 同名段の実寸に整合。イシュー #1561）。",
        },
        ArgRow {
            name: "label",
            kind: "Option<&str>",
            default: "None",
            description: "アクセシブルネーム（icon.rs `IconProps` 節）。",
        },
        ArgRow {
            name: "view_box",
            kind: "&str",
            default: "\"0 0 24 24\"",
            description: "viewBox 属性値（icon.rs `IconProps` 節）。",
        },
    ],
    examples: &[ExampleEntry {
        title: "Labelled",
        description: "label=\"Search\" を指定し role=\"img\" として扱う例です。",
        render: ex_icon,
    }],
    keyboard: &[],
    aria: &[AriaRow {
        attribute: "role=\"img\" + aria-label（label が Some の場合）/ aria-hidden=\"true\"（None の場合）",
        description: "label の有無で意味のあるアイコンか装飾用途かを切り替える（icon.rs `icon` 節）。",
    }],
    demo: None,
};

fn ex_image() -> Node {
    image::image(
        &image::ImageProps {
            fit: image::ImageFit::Contain,
            aspect_ratio: image::AspectRatio::Square,
            shape: image::ImageShape::Rounded,
            ..image::ImageProps::new(crate::showcase::IMAGE_DEMO_SRC, "製品写真")
        },
        vec![],
    )
}

pub(crate) const IMAGE: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "ImageFit（Cover/Contain/Fill/ScaleDown/NoFit、crates/pre-styled-ui/src/image.rs:76-107）で object-fit を切り替える",
        "AspectRatio（Auto/Square/Landscape/Portrait/Video、image.rs:113-141）で aspect-ratio を切り替える（イシュー #1562 で Landscape(4:3)/Portrait(3:4) を追加）",
        "ImageShape（Square/Rounded/Circle、image.rs:150-172）で角丸を切り替える（イシュー #1562 で新設。radius トークン `--fandhe-radius-none`/`-md`/`-full` 経由）",
        "base に height: auto を持つ（image.rs、イシュー #1562）。max-width: 100% による縮小時に縦横比を保つ",
        "src は既定エスケープ + is_safe_url 検証を経由し危険なスキーム（javascript: 等・data: 等）は出力自体を落とす（image.rs テスト dangerous_src_scheme_is_not_output_but_sibling_attrs_survive）",
    ],
    arguments: &[
        ArgRow {
            name: "src",
            kind: "&str",
            default: "(必須)",
            description: "画像 URL（image.rs:258、is_safe_url 検証を経由）。",
        },
        ArgRow {
            name: "alt",
            kind: "&str",
            default: "(必須)",
            description: "代替テキスト（image.rs:261、空文字列も明示的な選択として許容）。",
        },
        ArgRow {
            name: "fit",
            kind: "ImageFit",
            default: "Cover",
            description: "object-fit（image.rs:76-107, 263）。",
        },
        ArgRow {
            name: "aspect_ratio",
            kind: "AspectRatio",
            default: "Auto",
            description: "aspect-ratio（image.rs:113-141, 265）。",
        },
        ArgRow {
            name: "shape",
            kind: "ImageShape",
            default: "Square",
            description: "角丸（image.rs:150-172, 267。イシュー #1562 で新設）。",
        },
    ],
    examples: &[ExampleEntry {
        title: "Square contain, rounded",
        description: "Contain fit + Square aspect ratio + Rounded shape の例です。",
        render: ex_image,
    }],
    keyboard: &[],
    aria: &[AriaRow {
        attribute: "alt（必須引数）",
        description: "代替テキストを img 要素の alt として必ず出力する（image.rs:261）。role/aria-* 自体は固有の出力を持たない。",
    }],
    demo: None,
};

fn ex_timeline() -> Node {
    timeline::root(
        timeline::TimelineVariant::Outline,
        Size::Md,
        ColorPalette::Accent,
        vec![],
        vec![
            timeline::item(
                vec![],
                vec![
                    timeline::connector(
                        vec![],
                        vec![
                            timeline::indicator(vec![], vec![]),
                            timeline::separator(vec![], vec![]),
                        ],
                    ),
                    timeline::content(
                        vec![],
                        vec![
                            timeline::title(vec![], vec![text("Started")]),
                            timeline::description(vec![], vec![text("2026-01-01")]),
                        ],
                    ),
                ],
            ),
            timeline::item(
                vec![],
                vec![
                    timeline::connector(vec![], vec![timeline::indicator(vec![], vec![])]),
                    timeline::content(vec![], vec![timeline::title(vec![], vec![text("Now")])]),
                ],
            ),
        ],
    )
}

pub(crate) const TIMELINE: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "TimelineVariant（Solid/Subtle/Outline/Plain、crates/pre-styled-ui/src/timeline.rs:232-247）で indicator の塗り方を切り替える",
        "item/connector/separator/indicator/content/title/description の 7 パーツで年表を構造化する（timeline.rs:711-756）",
        "最終 item は separator を組み込まないことで非表示にする契約（showLastSeparator 相当は実装しない、timeline.rs:48-55）",
        "content は縦積み flex で title（sm / medium、size 連動の font-size）・description（xs / muted）の型階層を持つ（イシュー #1576）",
        "呼び出し側が indicator/separator へ data-state=\"complete\"/\"current\" を付与すると完了区間・現在位置のスタイルが適用される（recipe 側は子孫セレクタを持たないため、値は呼び出し側の構成責務。イシュー #1575、timeline.rs:107-118）",
    ],
    arguments: &[
        ArgRow {
            name: "variant",
            kind: "TimelineVariant",
            default: "Solid",
            description: "indicator の塗り方（timeline.rs:232-247）。",
        },
        ArgRow {
            name: "palette",
            kind: "ColorPalette",
            default: "Accent",
            description: "colorPalette 軸（timeline.rs:659）。",
        },
    ],
    examples: &[ExampleEntry {
        title: "Two events",
        description: "2 件のイベントを Outline variant で表示する例です（最終 item は separator を省略）。",
        render: ex_timeline,
    }],
    keyboard: &[],
    aria: &[AriaRow {
        attribute: "(該当なし)",
        description: "ol/li のネイティブ意味論のみで固有の role/aria-* は出力しない（timeline.rs 全文で role/aria-* を grep しても 0 件）。",
    }],
    demo: None,
};

// ---------------------------------------------------------------------
// Interactive（navigation 系、本文明示 5 件）
// ---------------------------------------------------------------------

fn ex_breadcrumb() -> Node {
    breadcrumb::root(
        Size::Md,
        breadcrumb::BreadcrumbVariant::Underline,
        None,
        vec![],
        vec![breadcrumb::list(
            vec![],
            vec![
                breadcrumb::item(
                    vec![],
                    vec![breadcrumb::link("../table/", vec![], vec![text("Docs")])],
                ),
                breadcrumb::separator(vec![], vec![text("/")]),
                breadcrumb::item(
                    vec![],
                    vec![breadcrumb::current_link(vec![], vec![text("Components")])],
                ),
            ],
        )],
    )
}

pub(crate) const BREADCRUMB: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "BreadcrumbVariant（Plain/Underline、crates/pre-styled-ui/src/breadcrumb.rs:266-272）でリンクの下線表示を切り替える",
        "root は既定で aria-label=\"breadcrumb\" を付与する（breadcrumb.rs:152-154）",
        "current_link は aria-current=\"page\"、separator は role=\"presentation\" を固定付与する（breadcrumb.rs:184-188）",
    ],
    arguments: &[
        ArgRow {
            name: "variant",
            kind: "BreadcrumbVariant",
            default: "Plain",
            description: "リンクの下線表示（breadcrumb.rs:266-272）。",
        },
        ArgRow {
            name: "aria_label_value",
            kind: "Option<&str>",
            default: "None",
            description: "None の場合は既定値 \"breadcrumb\" を使う（breadcrumb.rs:122-134, 152-154）。",
        },
    ],
    examples: &[ExampleEntry {
        title: "Two levels",
        description: "Docs → Components の 2 階層パンくずの例です。",
        render: ex_breadcrumb,
    }],
    keyboard: &[],
    aria: &[
        AriaRow {
            attribute: "aria-label=\"breadcrumb\"（root、既定値）",
            description: "root へ既定で付与される（breadcrumb.rs:122-134, 152-154）。",
        },
        AriaRow {
            attribute: "aria-current=\"page\"（current_link） / role=\"presentation\"（separator）",
            description: "現在ページと区切り記号に付与される（breadcrumb.rs:184-188）。",
        },
    ],
    demo: None,
};

/// `crates/pre-styled-ui/src/tab_nav.rs:171`（`root` が `aria-label` を必須
/// 引数として要求）・`:198-209`（`link` が `current` に応じて
/// `aria-current="page"` + `data-current` を付与）。href は
/// `crate::linkcheck::check_links` の突合対象のため、実在ページへ解決
/// する相対パス（`/themes/` 配下の兄弟ページ。イシュー #1017 で
/// `/components/` から移行）を使う。
fn ex_tab_nav() -> Node {
    tab_nav::root(
        Size::Md,
        "Section navigation",
        vec![],
        vec![
            tab_nav::link("../tabs/", true, vec![], vec![text("Tabs")]),
            tab_nav::link("../nav-list/", false, vec![], vec![text("Nav List")]),
            tab_nav::link("../menubar/", false, vec![], vec![text("Menubar")]),
        ],
    )
}

pub(crate) const TAB_NAV: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "role=\"tablist\"/role=\"tab\" を出力しない。素の nav/a の暗黙 ARIA ロール（navigation/link）のみを使うナビゲーションリンク集合（crates/pre-styled-ui/src/tab_nav.rs:1-20）",
        "現在ページは aria-current=\"page\" + data-current で示す（tab_nav.rs:198-209）",
        "見た目は自前の宣言列から生成する（イシュー #1541 で crate::tabs との共有を解消）。size 軸（xs〜xl、既定 md）を持ち、color-palette 軸は非提供（tab_nav.rs 参照）",
    ],
    arguments: &[
        ArgRow {
            name: "size",
            kind: "Size",
            default: "Md",
            description: "サイズ（イシュー #1541 で追加、tab_nav.rs 参照）。",
        },
        ArgRow {
            name: "label",
            kind: "&str",
            default: "",
            description: "root に付与する aria-label（必須引数、tab_nav.rs:171）。",
        },
        ArgRow {
            name: "href",
            kind: "&str",
            default: "",
            description: "link の href（tab_nav.rs:198）。",
        },
        ArgRow {
            name: "current",
            kind: "bool",
            default: "false",
            description: "true のとき aria-current=\"page\" + data-current を付与する（tab_nav.rs:198-209）。",
        },
    ],
    examples: &[ExampleEntry {
        title: "Three links",
        description: "Tabs / Nav List / Menubar の 3 リンクのうち Tabs を現在ページとした例です。",
        render: ex_tab_nav,
    }],
    keyboard: &[KeyRow {
        key: "Tab / Shift+Tab",
        description: "通常のリンクとしてフォーカス移動する。矢印キーによる roving tabindex は持たない（crate::tabs との決定的な差）。",
    }],
    aria: &[
        AriaRow {
            attribute: "aria-label（root、必須引数）",
            description: "landmark のアクセシブルネームを常に持つ（tab_nav.rs:171）。",
        },
        AriaRow {
            attribute: "aria-current=\"page\"（link、current=true のとき）",
            description: "現在ページを示す（tab_nav.rs:198-209）。",
        },
        AriaRow {
            attribute: "role",
            description: "一切出力しない（tab_nav.rs:18）。",
        },
    ],
    demo: None,
};

fn ex_carousel() -> Node {
    carousel::root(
        Size::Md,
        Orientation::Horizontal,
        "Products",
        vec![],
        vec![],
    )
}

pub(crate) const CAROUSEL: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "orientation（Horizontal/Vertical）に応じて item-group の transform 軸を translateX/translateY に切り替える（crates/pre-styled-ui/src/carousel.rs:390-398）",
        "label 引数を root の aria-label へそのまま出力する（carousel.rs:277-318, 321-322）",
        "選択・チェック状態を示す部品ではないため colorPalette 軸を提供しない（carousel.rs テスト carousel_stylesheet_never_consumes_color_palette_axis）",
    ],
    arguments: &[
        ArgRow {
            name: "orientation",
            kind: "Orientation",
            default: "Horizontal",
            description: "スクロール方向（carousel.rs:277-318）。",
        },
        ArgRow {
            name: "label",
            kind: "&str",
            default: "(必須)",
            description: "root の aria-label に渡すラベル（carousel.rs:277-318, 321-322）。",
        },
    ],
    examples: &[ExampleEntry {
        title: "Root only",
        description: "aria-label=\"Products\" を持つ Carousel root の最小構成例です。",
        render: ex_carousel,
    }],
    keyboard: &[],
    aria: &[AriaRow {
        attribute: "aria-label（root、呼び出し側必須引数）",
        description: "root へそのまま出力される（carousel.rs:277-318, 321-322）。",
    }],
    demo: None,
};

fn ex_pagination() -> Node {
    pagination::root(
        Size::Md,
        ColorPalette::Accent,
        "pagination",
        vec![],
        vec![pagination::item(
            pagination::ItemMode::Button,
            true,
            false,
            vec![],
            vec![text("1")],
        )],
    )
}

pub(crate) const PAGINATION: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "item は data-selected マーカー + aria-current=\"page\" で現在ページを表す（crates/pre-styled-ui/src/pagination.rs:35-38）",
        "item 自体には class を付与しない（root のみへクラスが付く複合部品の variant 統一方針、pagination.rs テスト reexported_item_is_not_given_variant_classes）",
        "root は <nav> 要素として出力される（pagination.rs テスト root_outputs_scope_and_part）",
    ],
    arguments: &[
        ArgRow {
            name: "palette",
            kind: "ColorPalette",
            default: "Accent",
            description: "colorPalette 軸（pagination.rs:447-460）。",
        },
        ArgRow {
            name: "aria_label",
            kind: "&str",
            default: "(必須)",
            description: "root（nav）の aria-label（pagination.rs:447-460, 528-534）。",
        },
    ],
    examples: &[ExampleEntry {
        title: "Current page",
        description: "選択中ページ 1 件のみを表示する最小構成例です。",
        render: ex_pagination,
    }],
    keyboard: &[],
    aria: &[AriaRow {
        attribute: "aria-current=\"page\"（選択中 item） / data-selected",
        description: "現在ページを示すマーカー（pagination.rs:35-38）。root（nav）は呼び出し側指定の aria-label を持つ。",
    }],
    demo: None,
};

fn ex_splitter() -> Node {
    use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::splitter::{
        PanelSpec, Splitter,
    };
    let state = Splitter::new(
        &[
            PanelSpec::new(50.0, 0.0, 100.0),
            PanelSpec::new(50.0, 0.0, 100.0),
        ],
        Orientation::Horizontal,
    );
    splitter::root(
        Size::Md,
        ColorPalette::Accent,
        &state,
        false,
        vec![],
        vec![
            splitter::panel(&state, 0, "panel-a", vec![], vec![text("A")]),
            splitter::resize_trigger(&state, 0, "panel-a", false, vec![], vec![]),
            splitter::panel(&state, 1, "panel-b", vec![], vec![text("B")]),
        ],
    )
}

pub(crate) const SPLITTER: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "panel は --fandhe-splitter-size custom property を通じてのみ動的な flex-basis を伝える唯一のパーツ（crates/pre-styled-ui/src/splitter.rs:688-697）",
        "resize_trigger は role=\"separator\" + aria-controls を固定付与する（splitter.rs:930-935）",
        "panel_index が範囲外の場合は style 属性自体を省略する fail-closed 動作（splitter.rs:694-697, 908-913）",
    ],
    arguments: &[ArgRow {
        name: "disabled",
        kind: "bool",
        default: "false",
        description: "root/resize_trigger の無効化状態（splitter.rs:672-686）。",
    }],
    examples: &[ExampleEntry {
        title: "Two panels",
        description: "50/50 の 2 パネルと resize_trigger 1 個の例です。",
        render: ex_splitter,
    }],
    keyboard: &[],
    aria: &[AriaRow {
        attribute: "role=\"separator\" + aria-controls（resize_trigger）",
        description: "リサイズハンドルが操作対象パネルを aria-controls で指し示す（splitter.rs:930-935）。",
    }],
    demo: None,
};

fn ex_steps() -> Node {
    use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::steps::Steps;
    let state = Steps::new(3, 1, Orientation::Horizontal);
    steps::root(
        Size::Md,
        ColorPalette::Accent,
        &state,
        vec![],
        vec![steps::list(
            &state,
            vec![],
            vec![
                steps::item(
                    &state,
                    0,
                    vec![],
                    vec![
                        steps::trigger(
                            &state,
                            0,
                            vec![],
                            vec![steps::indicator(&state, 0, vec![], vec![])],
                        ),
                        steps::separator(&state, 0, vec![], vec![]),
                    ],
                ),
                steps::item(
                    &state,
                    1,
                    vec![],
                    vec![steps::trigger(
                        &state,
                        1,
                        vec![],
                        vec![steps::indicator(&state, 1, vec![], vec![])],
                    )],
                ),
            ],
        )],
    )
}

pub(crate) const STEPS: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "trigger は現在ステップに aria-current=\"step\" を固定付与する（crates/pre-styled-ui/src/steps.rs テスト list_item_trigger_indicator_separator_delegate_to_headless、行 1256）",
        "separator は role=\"separator\" を持つ（steps.rs テスト同上、行 1258）",
        "indicator の data-state（current/complete）で見た目を切り替える（steps.rs:1134-1142）",
    ],
    arguments: &[],
    examples: &[ExampleEntry {
        title: "3 steps, step 2 current",
        description: "3 ステップ中 2 番目が current の状態を固定表示する例です。",
        render: ex_steps,
    }],
    keyboard: &[],
    aria: &[AriaRow {
        attribute: "aria-current=\"step\"（trigger） / role=\"separator\"（separator）",
        description: "現在ステップと区切りの意味論（steps.rs テスト list_item_trigger_indicator_separator_delegate_to_headless、行 1256, 1258）。",
    }],
    demo: None,
};

// ---------------------------------------------------------------------
// Utilities（本文明示 3 件）
// ---------------------------------------------------------------------

fn ex_marquee() -> Node {
    marquee::marquee(
        &marquee::MarqueeProps {
            label: Some("Breaking news"),
            ..marquee::MarqueeProps::default()
        },
        vec![],
        vec![marquee::item(vec![], vec![text("Breaking news ticker")])],
    )
}

pub(crate) const MARQUEE: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "content パーツを内部で 2 回複製しシームレスループを実現する（crates/pre-styled-ui/src/marquee.rs:472-480）",
        "root:hover/:focus-within で常時一時停止する CSS を持つ（marquee.rs テスト css_output_declares_hover_and_focus_within_pause）",
        "prefers-reduced-motion: reduce でアニメーションを停止する（marquee.rs テスト css_output_declares_reduced_motion_media_query）",
        "イシュー #1582: content の animation を longhand へ分解し、--fandhe-marquee-delay / -loop-count を追加公開する（marquee.rs テスト css_output_declares_scroll_animation_and_keyframes）",
        "イシュー #1582: edge: Fade で root に mask-image の両端フェードを適用する（marquee.rs テスト css_output_declares_edge_fade_mask_only_under_fade_variant）",
    ],
    arguments: &[
        ArgRow {
            name: "direction",
            kind: "MarqueeDirection",
            default: "Start",
            description: "スクロール方向。End で逆方向スクロールする（marquee.rs:216-237）。",
        },
        ArgRow {
            name: "edge",
            kind: "MarqueeEdge",
            default: "None",
            description: "Fade で root に両端フェード（mask-image、--fandhe-marquee-edge-size 既定 20% で調整可）を適用する（marquee.rs、イシュー #1582）。",
        },
        ArgRow {
            name: "decorative",
            kind: "bool",
            default: "false",
            description: "true なら root へ aria-hidden=\"true\" を付与する（marquee.rs:277, 442-449）。",
        },
        ArgRow {
            name: "label",
            kind: "Option<&str>",
            default: "None",
            description: "decorative が false のときのみ有効な aria-label（marquee.rs:280, 450-451）。",
        },
    ],
    examples: &[ExampleEntry {
        title: "Labelled",
        description: "aria-label 付きの非装飾 Marquee の例です。",
        render: ex_marquee,
    }],
    keyboard: &[],
    aria: &[AriaRow {
        attribute: "aria-hidden=\"true\"（decorative=true の root と複製 content） / aria-label（非 decorative時の root）",
        description: "decorative/label の組み合わせで root の意味論を切り替える（marquee.rs:442-451）。複製した 2 個目の content は常に aria-hidden=\"true\" + inert を持つ（marquee.rs:472-480）。",
    }],
    demo: None,
};

fn ex_scroll_area() -> Node {
    let items: Vec<Node> = (1..=5)
        .map(|i| el("p", vec![], vec![text(format!("Row {i}"))]))
        .collect();
    scroll_area::root(
        vec![(
            "style",
            "height: 6rem; width: 12rem; border: 1px solid var(--fandhe-color-border);",
        )],
        vec![scroll_area::viewport(
            vec![],
            vec![scroll_area::content(vec![], items)],
        )],
    )
}

pub(crate) const SCROLL_AREA: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "headless 層は anatomy（data-scope/data-part）と tabindex=\"0\" のみを提供し、CSS overflow が実際のスクロール可能性を担う（crates/pre-styled-ui/src/scroll_area.rs:11）",
        "::-webkit-scrollbar 系規則でカスタムスクロールバーの見た目を表現する（scroll_area.rs:124-134）",
        "JS によるスクロール位置追従は対象外（showcase.rs の scroll_area_section 記述と同方針）",
    ],
    arguments: &[],
    examples: &[ExampleEntry {
        title: "Scrollable list",
        description: "固定高さのビューポート内に 5 行を表示するスクロール領域の例です。",
        render: ex_scroll_area,
    }],
    keyboard: &[],
    aria: &[AriaRow {
        attribute: "tabindex=\"0\"（viewport）",
        description: "キーボードフォーカス可能にするのみで、固有の role/aria-* は出力しない（scroll_area.rs:11, 69）。",
    }],
    demo: None,
};

fn ex_separator() -> Node {
    separator::separator(
        &separator::SeparatorProps {
            orientation: Orientation::Vertical,
            variant: separator::SeparatorVariant::Dashed,
        },
        vec![],
    )
}

pub(crate) const SEPARATOR: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "orientation が role=\"separator\"（固定） + aria-orientation + data-orientation + variant クラスの 3 箇所へ連動する（crates/pre-styled-ui/src/separator.rs:10, 17）",
        "SeparatorVariant（Solid/Dashed、separator.rs:74-84）で罫線種別を切り替える",
        "role/aria-orientation/data-orientation は呼び出し側の偽装を除去し常にフレームワーク値へ一本化する（separator.rs:213-217、skeleton の aria-hidden 除去と同型）",
    ],
    arguments: &[
        ArgRow {
            name: "orientation",
            kind: "Orientation",
            default: "Horizontal",
            description: "向き（separator.rs:114-124）。",
        },
        ArgRow {
            name: "variant",
            kind: "SeparatorVariant",
            default: "Solid",
            description: "罫線種別（separator.rs:74-84, 122-124）。",
        },
    ],
    examples: &[ExampleEntry {
        title: "Vertical dashed",
        description: "縦向き・破線の Separator の例です。",
        render: ex_separator,
    }],
    keyboard: &[],
    aria: &[AriaRow {
        attribute: "role=\"separator\" + aria-orientation",
        description: "orientation と連動し常に固定出力される（separator.rs:10, 17, 58-59）。",
    }],
    demo: None,
};
