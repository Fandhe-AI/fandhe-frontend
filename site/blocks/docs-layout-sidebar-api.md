# docs-layout-sidebar-api

`fandhe-frontend-pre-styled-ui` の `input-group` / `input` / `scroll-area` /
`nav-list` / `badge` / `collapsible` / `link` / `button` / `icon` 部品を
合成した、API ドキュメント用サイドバーです。Blocks セクションは新規部品を
追加するものではなく、既存の Themes/Primitives 部品を組み合わせた実例集で
あることに注意してください（主参照は対応表 ID R0089。出典の固有名・
ファイル名は記載しません）。

上端に検索欄、中央のスクロール領域にカテゴリ見出しと HTTP メソッド
バッジ付きのエンドポイントリンク、下端に外部リンク 3 件の固定フッターを
置く 3 段構成です。docs サイトは JS を使わない静的なサイトであるため、
開閉式のカテゴリは常に開いた状態で表示します。

本 Demo は静的な表示例であり、検索欄・検索トリガー・AI 質問ボタンは
いずれも送信・取得を一切行わない飾りの要素です（`<form>` 要素は持たず、
検索トリガー・AI 質問ボタンは `disabled` で操作自体を無効化しています）。
エンドポイントの名前・メソッドはすべて独自に書いた架空のものであり、
実在する API・実企業名・実クレデンシャル・PII を含みません。

## Rust コード

```rust
use fandhe_frontend_core::{div, el, span, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps, BadgeVariant};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::collapsible::{self, OpenState};
use fandhe_frontend_pre_styled_ui::field::{FieldIds, FieldProps};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::input::{self, InputProps};
use fandhe_frontend_pre_styled_ui::input_group::{self, InputGroupAlign, InputGroupProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::nav_list;
use fandhe_frontend_pre_styled_ui::scroll_area;
use fandhe_frontend_pre_styled_ui::ColorPalette;

/// 実在の自リポジトリ URL（`href` の方針、モジュール doc 参照）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";
/// 実在の仕様リポジトリ URL。
const SPEC_REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend-spec";
/// エンドポイントリンクが指す、自リポジトリの docs/api ツリー（架空の
/// API パスではなく実在 URL を使う、モジュール doc「`href` は自組織の
/// 実在 URL のみ」節）。
const DOCS_API_URL: &str = "https://github.com/Fandhe-AI/fandhe-frontend/tree/main/docs/api";

/// 架空の API エンドポイント 1 件（HTTP メソッド・表示名・href の fragment）。
struct Endpoint {
    method: &'static str,
    name: &'static str,
    slug: &'static str,
}

/// 架空のカテゴリ 1 件（見出し + エンドポイント一覧）。
struct Category {
    key: &'static str,
    label: &'static str,
    endpoints: &'static [Endpoint],
}

/// 架空の API リファレンス一覧（実在サービスの API 名は含まない）。
const CATEGORIES: &[Category] = &[
    Category {
        key: "intro",
        label: "はじめに",
        endpoints: &[
            Endpoint {
                method: "GET",
                name: "概要",
                slug: "overview",
            },
            Endpoint {
                method: "GET",
                name: "クイックスタート",
                slug: "quickstart",
            },
        ],
    },
    Category {
        key: "auth",
        label: "認証",
        endpoints: &[
            Endpoint {
                method: "POST",
                name: "トークン発行",
                slug: "auth-token-issue",
            },
            Endpoint {
                method: "DELETE",
                name: "トークン失効",
                slug: "auth-token-revoke",
            },
        ],
    },
    Category {
        key: "projects",
        label: "プロジェクト",
        endpoints: &[
            Endpoint {
                method: "GET",
                name: "プロジェクト一覧",
                slug: "projects-list",
            },
            Endpoint {
                method: "POST",
                name: "プロジェクト作成",
                slug: "projects-create",
            },
            Endpoint {
                method: "PATCH",
                name: "プロジェクト更新",
                slug: "projects-update",
            },
            Endpoint {
                method: "DELETE",
                name: "プロジェクト削除",
                slug: "projects-delete",
            },
        ],
    },
    Category {
        key: "members",
        label: "メンバー",
        endpoints: &[
            Endpoint {
                method: "GET",
                name: "メンバー一覧",
                slug: "members-list",
            },
            Endpoint {
                method: "POST",
                name: "メンバー招待",
                slug: "members-invite",
            },
        ],
    },
];

/// 装飾用の自作幾何アイコン（実在ブランドのロゴを模さない、`label: None`）。
fn geo_icon(d: &'static str) -> Node {
    icon(
        &IconProps {
            label: None,
            ..IconProps::default()
        },
        vec![],
        vec![el("path", vec![("d", d)], vec![])],
    )
}

/// 検索アイコン（`navbar_with_search::search_icon` と同型の自作幾何
/// アイコン。輪郭のみのため `fill="none"` + `stroke="currentColor"`）。
fn search_icon() -> Node {
    icon(
        &IconProps {
            label: None,
            ..IconProps::default()
        },
        vec![],
        vec![el(
            "path",
            vec![
                ("d", "M11 4a7 7 0 1 0 0 14 7 7 0 0 0 0-14zm9 17-5.2-5.2"),
                ("fill", "none"),
                ("stroke", "currentColor"),
                ("stroke-width", "2"),
                ("stroke-linecap", "round"),
                ("stroke-linejoin", "round"),
            ],
            vec![],
        )],
    )
}

/// AI 質問アイコン（4 方向の光条を模した単純な星形、塗りつぶし図形）。
fn ai_icon() -> Node {
    geo_icon("M12 2l1.8 6.2L20 10l-6.2 1.8L12 18l-1.8-6.2L4 10l6.2-1.8z")
}

/// HTTP メソッドに応じた palette（モジュール doc「使用部品」節の表）。
fn method_palette(method: &str) -> ColorPalette {
    match method {
        "GET" => ColorPalette::Success,
        "POST" => ColorPalette::Info,
        "PATCH" => ColorPalette::Warning,
        "DELETE" => ColorPalette::Danger,
        _ => ColorPalette::Neutral,
    }
}

/// メソッドバッジ（等幅表示・行の右端へ寄せる、[`LAYOUT_CSS`] 参照）。
fn method_badge(method: &'static str) -> Node {
    badge::badge(
        &BadgeProps {
            variant: BadgeVariant::Subtle,
            palette: method_palette(method),
            ..BadgeProps::default()
        },
        vec![("data-blocks-docs-layout-sidebar-api-method", "")],
        vec![text(method)],
    )
}

/// エンドポイント 1 件分の `nav_list::item`（リンク名 + 右端メソッド
/// バッジ）。`current` は一覧内で高々 1 件のみ `true` にする。
fn endpoint_item(endpoint: &Endpoint, current: bool) -> Node {
    let href = format!("{DOCS_API_URL}#{}", endpoint.slug);
    nav_list::item(
        vec![],
        vec![nav_list::link(
            href.as_str(),
            current,
            vec![("data-blocks-docs-layout-sidebar-api-link", "")],
            vec![
                span(vec![], vec![text(endpoint.name)]),
                method_badge(endpoint.method),
            ],
        )],
    )
}

/// `search-input` variant 用: カテゴリごとに常時展開の見出し + リストを
/// 並べる（`nav_list::heading`/`nav_list::list` をそのまま使う）。
fn static_category(category: &Category, current_key: &'static str) -> Node {
    let items: Vec<Node> = category
        .endpoints
        .iter()
        .enumerate()
        .map(|(index, endpoint)| endpoint_item(endpoint, category.key == current_key && index == 0))
        .collect();
    div(
        vec![],
        vec![
            nav_list::heading(vec![], vec![text(category.label)]),
            nav_list::list(vec![], items),
        ],
    )
}

/// `search-ai` variant 用: カテゴリごとに開閉式グループで包む（全件
/// [`OpenState::Open`] + `disabled: true` 固定、モジュール doc「静的表示」
/// 節）。
fn collapsible_category(category: &Category, current_key: &'static str) -> Node {
    let state = OpenState::Open;
    let content_id = format!(
        "blocks-docs-layout-sidebar-api-group-{}-search-ai",
        category.key
    );
    let trigger = collapsible::trigger(
        state,
        true,
        Some(content_id.as_str()),
        vec![("data-blocks-docs-layout-sidebar-api-group-trigger", "")],
        vec![
            span(vec![], vec![text(category.label)]),
            collapsible::indicator(
                state,
                true,
                vec![("data-blocks-docs-layout-sidebar-api-group-chevron", "")],
                vec![text("\u{25be}")],
            ),
        ],
    );
    let items: Vec<Node> = category
        .endpoints
        .iter()
        .enumerate()
        .map(|(index, endpoint)| endpoint_item(endpoint, category.key == current_key && index == 0))
        .collect();
    let content = collapsible::content(
        state,
        true,
        Some(content_id.as_str()),
        vec![],
        vec![nav_list::list(vec![], items)],
    );
    collapsible::root(
        state,
        true,
        vec![("data-blocks-docs-layout-sidebar-api-group", "")],
        vec![
            el(
                "h3",
                vec![("data-blocks-docs-layout-sidebar-api-group-heading", "")],
                vec![trigger],
            ),
            content,
        ],
    )
}

/// 上端: `search-input` variant の検索欄（`input_group` + `input`、
/// アクセシブルネームは `aria-label` を直接渡す、モジュール doc参照）。
fn search_input_top() -> Node {
    let field = FieldProps {
        id: "blocks-docs-layout-sidebar-api-query-search-input",
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: false,
    };
    let group_props = InputGroupProps {
        disabled: false,
        invalid: false,
    };
    div(
        vec![("data-blocks-docs-layout-sidebar-api-top", "")],
        vec![input_group::root(
            &group_props,
            vec![("data-blocks-docs-layout-sidebar-api-search-group", "")],
            vec![
                input_group::addon(
                    InputGroupAlign::InlineStart,
                    &group_props,
                    vec![],
                    vec![search_icon()],
                ),
                input::input(
                    &InputProps::default(),
                    &field,
                    vec![
                        ("type", "search"),
                        ("autocomplete", "off"),
                        ("placeholder", "検索"),
                        ("aria-label", "API リファレンスを検索"),
                    ],
                ),
            ],
        )],
    )
}

/// 上端: `search-ai` variant の検索トリガー + AI 質問ボタン（いずれも
/// `disabled: true` 固定、モジュール doc「静的表示」節）。
fn search_ai_top() -> Node {
    div(
        vec![("data-blocks-docs-layout-sidebar-api-top", "")],
        vec![
            button::button(
                &ButtonProps {
                    disabled: true,
                    ..ButtonProps::default()
                },
                vec![("data-blocks-docs-layout-sidebar-api-search-trigger", "")],
                vec![search_icon(), text("検索")],
            ),
            button::icon_button(
                &ButtonProps {
                    disabled: true,
                    ..ButtonProps::default()
                },
                "AI に質問する",
                vec![("data-blocks-docs-layout-sidebar-api-ai-trigger", "")],
                vec![ai_icon()],
            ),
        ],
    )
}

/// 下端: 外部リンク 3 件を固定フッターとして置く（variant ごとに別要素を
/// 生成するが内容は同一）。
fn footer_links() -> Node {
    div(
        vec![("data-blocks-docs-layout-sidebar-api-footer", "")],
        vec![
            link::root(
                REPO,
                &LinkProps {
                    external: true,
                    ..LinkProps::default()
                },
                vec![],
                vec![text("リポジトリ")],
            ),
            link::root(
                SPEC_REPO,
                &LinkProps {
                    external: true,
                    ..LinkProps::default()
                },
                vec![],
                vec![text("仕様リポジトリ")],
            ),
            link::root(
                &format!("{REPO}/issues"),
                &LinkProps {
                    external: true,
                    ..LinkProps::default()
                },
                vec![],
                vec![text("Issue 一覧")],
            ),
        ],
    )
}

/// 1 variant 分のサイドバー本体（上端・中央スクロール・下端固定の 3 段、
/// モジュール doc「固定フッター」節）。
fn sidebar(variant: &'static str, top: Node, use_collapsible: bool) -> Node {
    let label = format!("API リファレンス（{variant}）");
    let viewport_label = format!("API エンドポイント一覧（{variant}）");
    let current_key = "projects";

    let category_nodes: Vec<Node> = CATEGORIES
        .iter()
        .map(|category| {
            if use_collapsible {
                collapsible_category(category, current_key)
            } else {
                static_category(category, current_key)
            }
        })
        .collect();

    div(
        vec![
            ("data-blocks-docs-layout-sidebar-api-shell", ""),
            ("data-blocks-docs-layout-sidebar-api-variant", variant),
        ],
        vec![
            top,
            div(
                vec![("data-blocks-docs-layout-sidebar-api-scroll", "")],
                vec![scroll_area::root(
                    vec![],
                    vec![scroll_area::viewport(
                        vec![("role", "region"), ("aria-label", viewport_label.as_str())],
                        vec![scroll_area::content(
                            vec![],
                            vec![nav_list::root(label.as_str(), vec![], category_nodes)],
                        )],
                    )],
                )],
            ),
            footer_links(),
        ],
    )
}

/// caption（並記された各 variant の見出し）。
fn caption(label: &'static str) -> Node {
    el(
        "p",
        vec![("class", "blocks-docs-layout-sidebar-api-caption")],
        vec![text(label)],
    )
}

/// `docs-layout-sidebar-api` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数（モジュール doc「静的表示」節）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-docs-layout-sidebar-api-stack")],
        vec![
            caption("検索欄 + 常時展開のカテゴリ一覧（search-input）"),
            sidebar("search-input", search_input_top(), false),
            caption("検索トリガー + AI 質問ボタン + 開閉式カテゴリ（search-ai）"),
            sidebar("search-ai", search_ai_top(), true),
        ],
    )
}
```

## 集約元との差分メモ

- 主参照（対応表 ID R0089）の「実入力の検索欄 + メソッドバッジ付き
  エンドポイント一覧」をそのまま採用しました。検索欄は `input-group` +
  `input` の通常の入力欄で、アクセシブルネームは `aria-label` を直接渡す
  形にしています。
- 対応表 ID R0090（検索トリガー + AI 質問ボタン・開閉式 API グループ）は
  2 つ目の variant として並記しています。無 JS の docs サイトでは検索
  トリガー・AI 質問ボタンを押しても何も起きないため、いずれも `disabled`
  で操作自体を無効化しています。
- R0090 の開閉式グループは、無 JS では開閉が機能せず閉じた内容が到達不能
  になるため、全グループを開いた状態で固定表示しています（トリガー自体も
  `disabled` にし、クリック・Enter/Space が no-op であることを示します）。
- 下端の固定フッターは `position: fixed` ではなく、外枠を固定高の
  flex column にして中央のスクロール領域だけを伸縮させる方法で実現して
  います。
- HTTP メソッドのバッジ色は GET=Success・POST=Info・PATCH=Warning・
  DELETE=Danger の割り当てにしています（既存のテーマトークンの範囲内）。
- エンドポイントの名前・パス・パラメータはすべて独自に書いた架空の API
  です。出典の固有名・ファイル名は記載していません。

関連情報: [Input Group](../themes/input-group.md) /
[Input](../themes/input.md) / [Scroll Area](../themes/scroll-area.md) /
[Nav List](../themes/nav-list.md) / [Badge](../themes/badge.md) /
[Collapsible](../themes/collapsible.md) / [Link](../themes/link.md) /
[Button](../themes/button.md) / [Icon](../themes/icon.md)
