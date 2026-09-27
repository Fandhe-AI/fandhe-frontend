# grid-list-compact-tiles

`fandhe-frontend-pre-styled-ui` の `list` / `avatar` / `heading` / `link` /
`link-overlay` / `button` / `menu` / `item` 部品を合成した、コンパクトな
横長タイルをグリッドで並べる目的別パーツです。Blocks セクションは新規部品を
追加するものではなく、既存の Themes/Primitives 部品を組み合わせた実例集で
あることに注意してください（主参照は対応表 ID R0975。集約元は R0976。出典の
固有名・ファイル名は記載しません）。

Demo は配置の異なる 2 variant を並記します。1 つ目「固定したプロジェクト」
（R0975 相当）は、色付きのイニシャル枠 + タイル全面のクリック領域 + 三点
メニューを持つプロジェクトタイルです。2 つ目「チームメンバー」（R0976 相当）
は、アバター + タイル自体がリンクになった人物タイルで、三点メニューは持ちま
せん。狭い幅ではどちらの variant も 1 列、40rem 以上で 2 列、64rem 以上で
4 列のグリッドになります。

本 Demo は静的な表示例であり、docs サイトは JS ハイドレーションを行わない
ため、`<form>` 要素は一切持たず、データの取得・送信・状態管理を行いません。
三点メニューの各項目（開く/固定を解除/削除）は表示するだけで処理を持たず、
閉じた状態のまま固定表示します。プロジェクト名・人名・役職はすべて独自に
書いた架空のものであり、実在の企業・プロダクト・人物・PII を含みません。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, AvatarShape, ImageStatus};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::item::{
    self, ItemMediaVariant, ItemRootProps, ItemSize, ItemVariant,
};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::link_overlay::overlay;
use fandhe_frontend_pre_styled_ui::list::{self, ListType, ListVariant};
use fandhe_frontend_pre_styled_ui::menu::{self, OpenState};
use fandhe_frontend_pre_styled_ui::Size;

/// リンク先の固定外部 URL（モジュール doc「リンク先の方針」節参照）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// プロジェクトタイル 1 件分のダミーデータ（架空）。
struct Project {
    /// 色トークンの軸（`--fandhe-color-{tone}-subtle`/`-fg` の組と対応）。
    tone: &'static str,
    /// イニシャル枠に表示する 2 文字。
    initials: &'static str,
    name: &'static str,
    member_label: &'static str,
}

/// プロジェクトタイル 4 件（架空、実在の企業・プロダクトとは無関係）。
const PROJECTS: [Project; 4] = [
    Project {
        tone: "accent",
        initials: "FE",
        name: "Frontend Refresh",
        member_label: "メンバー 12 人",
    },
    Project {
        tone: "info",
        initials: "DS",
        name: "Docs Site v2",
        member_label: "メンバー 5 人",
    },
    Project {
        tone: "success",
        initials: "SR",
        name: "Server Runtime",
        member_label: "メンバー 8 人",
    },
    Project {
        tone: "warning",
        initials: "QA",
        name: "Release QA",
        member_label: "メンバー 4 人",
    },
];

/// プロジェクトタイルの三点メニュー（開く/固定を解除/削除。処理は持たない
/// 表示専用、モジュール doc「`<form>` を持たない」節参照）。
fn project_menu(index: usize, project_name: &str) -> Node {
    let content_id = format!("blocks-grid-list-compact-tiles-menu-{index}");
    let trigger_label = format!("{project_name} の操作を開く");
    let trigger = menu::trigger(
        OpenState::Closed,
        false,
        Some(content_id.as_str()),
        vec![
            ("aria-label", trigger_label.as_str()),
            ("data-blocks-grid-list-compact-tiles-menu-trigger", ""),
        ],
        vec![text("\u{2026}")],
    );
    let content = menu::content(
        OpenState::Closed,
        Some(content_id.as_str()),
        None,
        vec![],
        vec![
            menu::item("open", false, false, vec![], vec![text("開く")]),
            menu::item("unpin", false, false, vec![], vec![text("固定を解除")]),
            menu::separator(vec![], vec![]),
            menu::item("delete", false, false, vec![], vec![text("削除")]),
        ],
    );
    let positioner = menu::positioner(OpenState::Closed, vec![], vec![content]);
    menu::root(
        Size::Sm,
        OpenState::Closed,
        vec![],
        vec![trigger, positioner],
    )
}

/// プロジェクトタイル 1 件（イニシャル枠 + 全面クリック用 `overlay` +
/// 三点メニュー、モジュール doc「`item::root` を `link_overlay::root` で
/// 包まない理由」節参照）。
fn project_tile(index: usize, project: &Project) -> Node {
    item::root(
        ItemRootProps {
            variant: ItemVariant::Outline,
            size: ItemSize::Sm,
            ..ItemRootProps::default()
        },
        vec![("data-blocks-grid-list-compact-tiles-tile", "")],
        vec![
            item::media(
                ItemMediaVariant::Default,
                vec![
                    ("data-blocks-grid-list-compact-tiles-icon", ""),
                    ("data-tone", project.tone),
                    ("aria-hidden", "true"),
                ],
                vec![text(project.initials)],
            ),
            item::content(
                vec![],
                vec![
                    item::title(vec![], vec![text(project.name)]),
                    item::description(vec![], vec![text(project.member_label)]),
                ],
            ),
            overlay(REPO, vec![("aria-label", project.name)], vec![]),
            item::actions(vec![], vec![project_menu(index, project.name)]),
        ],
    )
}

/// 「固定したプロジェクト」セクション（Variant A、主参照相当の代表構成）。
fn instance_projects() -> Node {
    let tiles: Vec<Node> = PROJECTS
        .iter()
        .enumerate()
        .map(|(i, project)| list::item(vec![], vec![project_tile(i, project)]))
        .collect();
    div(
        vec![("class", "blocks-grid-list-compact-tiles-section")],
        vec![
            div(
                vec![("class", "blocks-grid-list-compact-tiles-header")],
                vec![
                    heading::heading(
                        HeadingLevel::H3,
                        &HeadingProps {
                            size: HeadingSize::Xl2,
                            ..HeadingProps::default()
                        },
                        vec![],
                        vec![text("固定したプロジェクト")],
                    ),
                    button::button(
                        &ButtonProps {
                            variant: ButtonVariant::Outline,
                            size: Size::Sm,
                            ..ButtonProps::default()
                        },
                        vec![],
                        vec![text("新規プロジェクト")],
                    ),
                ],
            ),
            list::root(
                ListType::Unordered,
                ListVariant::Plain,
                vec![("data-blocks-grid-list-compact-tiles-list", "")],
                tiles,
            ),
        ],
    )
}

/// 人物タイル 1 件（アバター + タイル自体がリンク、メニューなし。
/// モジュール doc「2 variant 併記」節参照）。
fn member_tile(index: usize) -> Node {
    let name = dummy_assets::PERSON_NAMES[index];
    let role = dummy_assets::JOB_TITLES[index % dummy_assets::JOB_TITLES.len()];
    let initials: String = name
        .split_whitespace()
        .filter_map(|part| part.chars().next())
        .collect();
    item::root(
        ItemRootProps {
            href: Some(REPO),
            variant: ItemVariant::Outline,
            size: ItemSize::Sm,
            ..ItemRootProps::default()
        },
        vec![],
        vec![
            item::media(
                ItemMediaVariant::Default,
                vec![],
                vec![avatar::root(
                    &AvatarProps {
                        shape: AvatarShape::Circle,
                        size: Size::Sm,
                        ..AvatarProps::default()
                    },
                    vec![],
                    vec![
                        avatar::image(ImageStatus::Loaded, dummy_assets::AVATAR_SRC, "", vec![]),
                        avatar::fallback(ImageStatus::Loaded, vec![], vec![text(initials)]),
                    ],
                )],
            ),
            item::content(
                vec![],
                vec![
                    item::title(vec![], vec![text(name)]),
                    item::description(vec![], vec![text(role)]),
                ],
            ),
        ],
    )
}

/// 「チームメンバー」セクション（Variant B、集約元相当の全面リンクカード
/// 版）。
fn instance_members() -> Node {
    let tiles: Vec<Node> = (0..4)
        .map(|i| list::item(vec![], vec![member_tile(i)]))
        .collect();
    div(
        vec![("class", "blocks-grid-list-compact-tiles-section")],
        vec![
            div(
                vec![("class", "blocks-grid-list-compact-tiles-header")],
                vec![
                    heading::heading(
                        HeadingLevel::H3,
                        &HeadingProps {
                            size: HeadingSize::Xl2,
                            ..HeadingProps::default()
                        },
                        vec![],
                        vec![text("チームメンバー")],
                    ),
                    link::root(
                        REPO,
                        &LinkProps {
                            external: true,
                            ..LinkProps::default()
                        },
                        vec![],
                        vec![text("すべて表示")],
                    ),
                ],
            ),
            list::root(
                ListType::Unordered,
                ListVariant::Plain,
                vec![("data-blocks-grid-list-compact-tiles-list", "")],
                tiles,
            ),
        ],
    )
}

/// `grid-list-compact-tiles` の Demo 本体（2 variant 併記）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-grid-list-compact-tiles-stack")],
        vec![instance_projects(), instance_members()],
    )
}
```

## 集約元との差分メモ

- R0975（主参照）: 色付きのイニシャル枠 + タイル全面リンク + 三点メニュー
  を持つプロジェクトタイル構成です。
- R0976（集約元）: アバター + タイル自体がリンクになった人物タイル構成で、
  三点メニューは持ちません（`<a>` の内側に対話要素を入れ子にしないため）。
- 両 variant とも同じグリッド・タイルの外形（`item` の `Outline`/`Sm`）を
  共有し、内側の構成（イニシャル枠 vs アバター、メニューの有無）だけが
  異なります。
