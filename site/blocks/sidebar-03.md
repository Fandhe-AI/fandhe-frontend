# sidebar-03

`fandhe-frontend-pre-styled-ui` の `sidebar`（`size="lg"` のブランド header・
`menu-sub` を入れ子にしたナビゲーション・`data-active` の現在項目・rail）/
`breadcrumb`（inset ヘッダー）/ `separator` / `icon` を合成した、
shadcn/ui Blocks の `sidebar-03` に相当する submenu 付きサイドバーの合成例
です。Blocks セクションは新規部品を追加するものではなく、既存の
Themes/Primitives 部品を組み合わせた実例集であることに注意してください。

docs サイトは JS ハイドレーションを行わないため、開閉操作（トリガーボタン・
Cmd/Ctrl+B）は wasm-full 側の実行時責務であり本 SSR 合成例では動作しません。
`sidebar-03` は既定 `collapsible="offcanvas"`（icon 折りたたみを持たない）
のため、`sidebar-07` と異なり expanded インスタンス 1 つのみを掲示します。

本 Demo は静的な表示例であり、`<form>` 要素を持たず、値の送信・検証・認証
処理・データ取得を一切行いません。ナビゲーション項目はすべて `href: None`
（`<button type="button">`）として組み、死リンク（`href="#"`）は出しません。
パンくず・ナビゲーションラベルはすべて一般名詞または架空のものであり、
実企業名・実在人物・実クレデンシャル・PII を含みません（`docs/policy/
intentional-non-adoption.md` §3.25 の責務境界: UI コンポーネント層は
アプリケーションロジックを内包しません）。アイコンは lucide 等の著作物では
なく自作の単純な矩形図形です。

## Rust コード

```rust
use fandhe_frontend_core::{div, el, span, text, Node};
use fandhe_frontend_pre_styled_ui::breadcrumb::{self, BreadcrumbVariant};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::separator::{self, SeparatorProps, SeparatorVariant};
use fandhe_frontend_pre_styled_ui::sidebar;
use fandhe_frontend_pre_styled_ui::sidebar::{
    Sidebar, SidebarMenuButtonProps, SidebarMenuButtonSize, SidebarMenuSubButtonProps,
    SidebarProps, SidebarState,
};
use fandhe_frontend_pre_styled_ui::{Orientation, Size};

/// 自作の単純な矩形アイコン（`d` は呼び出し側が座標を選ぶ、モジュール doc
/// 「アイコンは自作の単純幾何図形」参照）。
fn geo_icon(path_d: &'static str) -> Node {
    icon(
        &IconProps::default(),
        vec![],
        vec![el("path", vec![("d", path_d)], vec![])],
    )
}

/// header のブランド行（`size="lg"` の `menu_button` 1 個。角丸の濃色
/// アイコン枠 + 「Documentation」/「v1.0.0」の 2 行ラベル。モジュール doc
/// 「Issue 見立てとの差異」参照）。
fn brand_header() -> Node {
    let icon_box = div(
        vec![("data-blocks-sidebar-03-brand-icon", "")],
        vec![geo_icon("M4 4h16v16H4z")],
    );
    let button = sidebar::menu_button(
        &SidebarMenuButtonProps {
            href: None,
            size: SidebarMenuButtonSize::Lg,
            ..Default::default()
        },
        Some(icon_box),
        vec![("data-blocks-sidebar-03-brand", "")],
        vec![
            span(vec![], vec![text("Documentation")]),
            span(vec![], vec![text("v1.0.0")]),
        ],
    );
    sidebar::header(
        vec![],
        vec![sidebar::menu(
            vec![],
            vec![sidebar::menu_item(vec![], vec![button])],
        )],
    )
}

/// nav の 1 グループ（親見出しの `menu_button` + `menu-sub` の子項目群）。
/// `active` に一致する子項目のみ `data-active` を付与する
/// （モジュール doc「`asChild` 相当が無い点と `href` の扱い」参照）。
fn nav_group(title: &'static str, active: Option<&'static str>, subs: &[&'static str]) -> Node {
    let parent = sidebar::menu_button(
        &SidebarMenuButtonProps {
            href: None,
            ..Default::default()
        },
        None,
        vec![("data-blocks-sidebar-03-parent", "")],
        vec![text(title)],
    );
    let sub_items: Vec<Node> = subs
        .iter()
        .map(|label| {
            sidebar::menu_sub_item(
                vec![],
                vec![sidebar::menu_sub_button(
                    &SidebarMenuSubButtonProps {
                        href: None,
                        active: active == Some(*label),
                        ..Default::default()
                    },
                    vec![],
                    vec![text(*label)],
                )],
            )
        })
        .collect();
    sidebar::menu_item(vec![], vec![parent, sidebar::menu_sub(vec![], sub_items)])
}

/// nav データ（参照実物の 5 グループ・26 項目。他社製品固有名は一般名へ
/// 置換、モジュール doc「ラベルは一般名詞へ置換」参照）。
fn nav_content() -> Node {
    let groups = vec![
        nav_group(
            "Getting Started",
            None,
            &["Installation", "Project Structure"],
        ),
        nav_group(
            "Build Your Application",
            Some("Data Fetching"),
            &[
                "Routing",
                "Data Fetching",
                "Rendering",
                "Caching",
                "Styling",
                "Optimizing",
                "Configuring",
                "Testing",
                "Authentication",
                "Deploying",
                "Upgrading",
                "Examples",
            ],
        ),
        nav_group(
            "API Reference",
            None,
            &[
                "Components",
                "File Conventions",
                "Functions",
                "Config Options",
                "CLI",
                "Edge Runtime",
            ],
        ),
        nav_group(
            "Architecture",
            None,
            &[
                "Accessibility",
                "Hot Reload",
                "Compiler",
                "Supported Browsers",
                "Bundler",
            ],
        ),
        nav_group("Community", None, &["Contribution Guide"]),
    ];
    sidebar::content(
        vec![],
        vec![sidebar::group(
            None,
            vec![],
            vec![sidebar::group_content(
                vec![],
                vec![sidebar::menu(vec![], groups)],
            )],
        )],
    )
}

/// 左サイドバーの `root`（`sidebar_07::app_sidebar` と同型、`provider` の
/// 直接の子として置く）。
fn app_sidebar(state: &Sidebar, props: &SidebarProps, root_id: &str) -> Node {
    sidebar::root(
        state,
        props,
        "Main navigation",
        Some(root_id),
        vec![],
        vec![
            brand_header(),
            nav_content(),
            sidebar::rail(state, "Toggle sidebar rail", vec![], vec![]),
        ],
    )
}

/// inset 側（トリガー + 縦 separator + breadcrumb のヘッダー、3 枚の
/// プレースホルダーカード、大きなプレースホルダー 1 枚。
/// `sidebar_07::inset_area` と同型）。
fn inset_area(state: &Sidebar, root_id: &str) -> Node {
    let header = div(
        vec![("data-blocks-sidebar-03-header", "")],
        vec![
            sidebar::trigger(state, "Toggle sidebar", Some(root_id), vec![], vec![]),
            separator::separator(
                &SeparatorProps {
                    orientation: Orientation::Vertical,
                    variant: SeparatorVariant::Solid,
                },
                vec![],
            ),
            breadcrumb::root(
                Size::Md,
                BreadcrumbVariant::default(),
                Some("Breadcrumb"),
                vec![],
                vec![breadcrumb::list(
                    vec![],
                    vec![
                        breadcrumb::item(
                            vec![],
                            vec![breadcrumb::link(
                                "../",
                                vec![],
                                vec![text("Build Your Application")],
                            )],
                        ),
                        breadcrumb::separator(vec![], vec![text("/")]),
                        breadcrumb::item(
                            vec![],
                            vec![breadcrumb::current_link(
                                vec![],
                                vec![text("Data Fetching")],
                            )],
                        ),
                    ],
                )],
            ),
        ],
    );

    let grid = div(
        vec![("data-blocks-sidebar-03-grid", "")],
        vec![
            div(vec![("data-blocks-sidebar-03-placeholder", "")], vec![]),
            div(vec![("data-blocks-sidebar-03-placeholder", "")], vec![]),
            div(vec![("data-blocks-sidebar-03-placeholder", "")], vec![]),
        ],
    );
    let large_placeholder = div(vec![("data-blocks-sidebar-03-placeholder-lg", "")], vec![]);

    sidebar::inset(vec![], vec![header, grid, large_placeholder])
}

/// `sidebar-03` の Demo 本体。呼び出しごとに同一の `Node` を返す純関数
/// （モジュール doc「無 JS のため静的 1 インスタンス」参照。既定
/// `collapsible: Offcanvas` のまま `Expanded` の単一インスタンスを描画）。
pub fn demo() -> Node {
    let state = Sidebar::new(SidebarState::Expanded);
    let props = SidebarProps::default();
    let root_id = "blocks-sidebar-03-root";
    sidebar::provider(
        &state,
        &props,
        vec![("data-blocks-sidebar-03-instance", "")],
        vec![
            app_sidebar(&state, &props, root_id),
            inset_area(&state, root_id),
        ],
    )
}
```

## shadcn 側との差分メモ

- Issue 見立てでは header に検索 `input` を含む構成が想定されていましたが、
  参照実物（shadcn `sidebar-03` の `app-sidebar.tsx`/`page.tsx`）・
  スクリーンショットのいずれにも検索 `input` や version-switcher の
  ドロップダウンは存在せず、header は `size="lg"` の `menu_button` 1 個
  （アイコン + 「Documentation」/「v1.0.0」の 2 行ラベル）のみです。受け入れ
  条件がスクショ比較であるため、本実装は参照実物に忠実な構成（`input` 無し）
  を採りました。`sidebar::input` 自体は `/themes/sidebar/` ページで別途
  掲示済みのため、部品の不足を意味しません。
- ナビゲーション項目（親見出し・submenu 項目）はすべて `href: None`
  （`<button type="button">`）で組んでいます。`href="#"` の死リンクは
  `crates/docs-site/tests/blocks_contract.rs` が禁止しており、`<a>` に
  すると `aria-current="page"` が付与されますが `<button>` では付与
  されません。現在項目の視覚状態は `href` に依らない `data-active` で
  表現しています。
- 参照実物のナビゲーションラベルに含まれる他社製品固有の名称
  （「next.config.js Options」「Next.js Compiler」「Turbopack」
  「Fast Refresh」）は、他社製品名を持ち込まない方針に従いそれぞれ
  「Config Options」「Compiler」「Bundler」「Hot Reload」の一般名へ
  置換しています。
- `collapsible`/`menu`/`avatar` は使いません（`sidebar-07` との違い）。
  `sidebar-03` は team switcher・user footer のドロップダウンや icon
  折りたたみを持たない、より単純な構成です。
- アイコンは lucide ではなく自作の単純な矩形 SVG path です（著作物を
  複製しないため）。
- inset 側のプレースホルダー領域は素の `div` です（実データ取得は行い
  ません）。

関連情報: [Sidebar](../themes/sidebar.md) / [Breadcrumb](../themes/breadcrumb.md) /
[Separator](../themes/separator.md) / [Icon](../themes/icon.md)
