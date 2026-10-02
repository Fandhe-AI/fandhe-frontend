//! `docs-layout-sidebar-api` block（イシュー #3108。親トラッキング #3099
//! 「Phase 6」配下）。
//!
//! API ドキュメント用サイドバーの合成例。上端の検索欄・中央のスクロール
//! 領域（カテゴリ見出し + HTTP メソッドバッジ付きエンドポイントリンク）・
//! 下端の固定フッター（外部リンク 3 件）の 3 段構成を 2 variant で並記する。
//!
//! # 使用部品
//!
//! `input-group` / `input` / `scroll-area` / `nav-list` / `badge` /
//! `collapsible` / `link` / `button` / `icon` の 9 部品を合成する（[`BLOCK`]
//! の `parts` に一致させる契約、`crates/docs-site/tests/
//! blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//!
//! # 2 variant の構成（対応表 ID R0089/R0090 の差分、`site/blocks/
//! docs-layout-sidebar-api.md` の「集約元との差分メモ」節参照）
//!
//! | variant | 上端 | 中央 | 対応 ID |
//! |---|---|---|---|
//! | `search-input` | `input_group` 内に検索アイコン + `input`（`type="search"`） | カテゴリごとに `h3` 見出し（[`category_heading`]） + `nav_list::list` を常時展開で並べる | R0089（代表） |
//! | `search-ai` | 検索トリガー `button` + AI 質問 `button::icon_button`（いずれも `disabled: true`） | カテゴリごとに `collapsible` で包み、trigger を `h3` 見出しで包む | R0090 |
//!
//! # 見出しレベルの統一（Bugbot 指摘、PR #3548）
//!
//! 両 variant とも同一ページ上に並記されるため、カテゴリ見出しの階層
//! （heading outline）は variant 間で揃える必要がある。`nav_list::heading`
//! は `h2` 固定（headless 層の anatomy、レベル可変化は非対応）のため、
//! `search-ai` の `collapsible` trigger 見出し（素の `h3`）と混在させると
//! ページ内で h2 → h3 → h2 → h3 の非単調な見出し階層になる。このため
//! `search-input` 側も [`nav_list::heading`] は使わず
//! [`category_heading`]（素の `h3`）に統一する。
//!
//! # 静的表示・全 disabled 固定（無 JS）
//!
//! docs サイトは JS ハイドレーションを行わないため、`search-ai` の 2
//! ボタン（検索トリガー・AI 質問）は押しても何も起きないことを明示する
//! `disabled: true` で固定し、[`LAYOUT_CSS`] の `[data-disabled]` 複合
//! セレクタで `opacity: 1; cursor: default;` に中和する（`navbar_with_search`
//! と同じ判断）。`search-ai` の開閉式グループはすべて [`OpenState::Open`]
//! 固定で静的に展開した状態を示す（無 JS では開閉自体が機能しないため、
//! `api_reference_param_accordion` と同じく閉じた内容へ到達不能になるのを
//! 避ける）。`trigger` 自体も `disabled: true` にし、クリック・Enter/Space
//! が no-op であることを示す。
//!
//! # `search-input` の検索欄も disabled 固定にする（Codex 指摘、PR #3548）
//!
//! 当初案は `search-input` variant の `input::input` のみ `disabled` を
//! 付けていなかった。しかし docs サイトは JS ハイドレーションを行わないため
//! 検索は実際には機能せず、`disabled: false` の表示は「操作可能だが実は
//! 何も起きない」状態を利用者に誤って伝える（`search-ai` の 2 ボタンに
//! 対する「静的表示・全 disabled 固定」節と同じ判断軸）。このため
//! `search-input` の `input`/`input_group` も `disabled: true` で固定し、
//! [`LAYOUT_CSS`] の `[data-disabled]` 複合セレクタで中和する（`search-ai`
//! の 2 ボタンと同じ処理）。
//!
//! # 固定フッター（`position: fixed` は使わない）
//!
//! 外枠（[`LAYOUT_CSS`] の `[data-blocks-docs-layout-sidebar-api-shell]`）を
//! 固定高の flex column にし、上端・フッターを `flex: none`、中央の
//! `scroll_area::root` を `flex: 1; min-block-size: 0;` にすることで、中央
//! だけがスクロールしフッターは外枠の下端に常に見える（`position: fixed`
//! はビューポート基準の配置であり本 Demo の意図〔枠内下端への固定〕とは
//! 異なるため使わない）。
//!
//! # `search-ai` の collapsible 既定スタイルの上書き（Bugbot 指摘、PR #3548）
//!
//! `search-ai` の全グループは [`OpenState::Open`] + `disabled: true` 固定
//! （「静的表示・全 disabled 固定」節参照）のため、`collapsible` レシピの
//! base 規則（`[data-scope="collapsible"][data-part="…"]`、詳細度 2）と
//! open/disabled の state 規則（同 + `[data-state="open"]`/`[data-disabled]`、
//! 詳細度 3）が常時適用される。block 側のフック属性だけを書いた
//! `[data-blocks-docs-layout-sidebar-api-group-trigger]`（詳細度 1）は
//! これらすべてに負けるため、trigger の `color`/`display`/`padding` も
//! content の枠線・パディング・角丸・`[data-disabled]` 時のミュート文字色
//! も上書きできず、カテゴリ見出しだけ accent 色になり、開いたグループが
//! 積み重なった disclosure カードに見えていた。
//!
//! このため [`LAYOUT_CSS`] の collapsible 各パーツ（trigger / indicator /
//! content）への上書きは、**同一要素上で** レシピ属性と block フック属性を
//! 連結する形に統一する（`filter_expandable_panel`・
//! `docs_layout_toc_collapsible`・`notification_tray` と同じ流儀）:
//!
//! - base 上書き: `[data-scope="collapsible"][data-part="<part>"][data-blocks-docs-layout-sidebar-api-group-<part>]`（詳細度 3 > レシピ base の 2）
//! - state 上書き: 上記 + `[data-state="open"]` / `[data-disabled]`（詳細度 4 > レシピ state の 3）
//!
//! フック属性のみの単独セレクタで collapsible パーツを狙う規則は置かない
//! （置くとレシピに負けて no-op になる）。この不変条件は
//! `collapsible_overrides_chain_recipe_attributes_on_the_same_element`
//! テストが [`LAYOUT_CSS`] のセレクタ全件に対して固定する。
//!
//! # 検索欄のアクセシブルネーム
//!
//! `input::input` の `extra_attrs` へ直接 `aria-label` を渡す（headless
//! `field::input`/`control_attrs` はこのキーを予約・除去しないため、
//! `navbar_with_search` のような `Field` + `visually_hidden` + `label` の
//! 3 層構成を経由せずに済む。[`BLOCK`] の `parts` に `Field` を含めない
//! 理由）。
//!
//! # id と ARIA の一意性
//!
//! collapsible content の id は `blocks-docs-layout-sidebar-api-group-
//! {key}-{variant}`、scroll viewport の `aria-label`（「API エンドポイント
//! 一覧（{variant}）」）は variant を含める形にし、2 variant 間で重複しない
//! ようにする（`demo_output_has_no_dangling_aria_references_or_duplicate_ids`
//! 契約）。
//!
//! # カテゴリ一覧を `nav` ランドマークで包まない（Codex 指摘、PR #3548）
//!
//! 当初案はカテゴリ一覧全体を `nav_list::root`（`nav` + `aria-label`）で
//! 包んでいたが、[`endpoint_item`] は前節のとおり `href` を持たない静的な
//! 表示要素であり、`nav` ランドマークは「リンクを辿って移動できる領域」を
//! 期待する支援技術の利用者の期待と食い違う（`nav_list` モジュール doc
//! 「文書ナビ向け Link リスト」の用途外使用）。そのため [`sidebar`] は
//! カテゴリ一覧を素の `div`（ランドマークなし）で包み、ランドマークは
//! 外側の `scroll_area::viewport` が既に持つ `role="region"` + `aria-label`
//! （「API エンドポイント一覧（{variant}）」）のみに一本化する。カテゴリ
//! ごとの `nav_list::list`/`nav_list::item`（`ul`/`li`、ランドマークを
//! 持たない中立な構造ロール）は引き続き使う。
//!
//! # エンドポイント行はリンクにしない（非ブロック指摘対応、PR #3548）
//!
//! エンドポイント名は架空の API（実在サービス名を含まない）であり、
//! 1 件 1 ページの実在する個別ドキュメントページを持たない。当初案は
//! 各行を自リポジトリ `docs/api` ディレクトリ URL への `#<slug>` フラグ
//! メント付きリンクにしていたが、ディレクトリ表示側にそのアンカーへ
//! 対応する要素がなく、リンク先が実際には存在しない見出しへ遷移できない
//! 状態だった（codex-review P1 指摘）。そのため本 Demo のエンドポイント行は
//! `href` を持たない静的な表示要素（[`endpoint_item`]）とする。
//! フッターリンクのみ実在する自リポジトリ・仕様リポジトリの URL
//! （[`REPO`]/[`SPEC_REPO`]）を指すリンクを持つ（`#` だけの値・`data:` を
//! 使わない契約は `block_pages_never_contain_a_form_element_or_data_uri`
//! が検証する）。
//!
//! # `<form>` を使わない・データの取得/送信を行わない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従う静的な合成例。検索欄は飾りの入力であり、入力値は
//! どこにも送信しない（原稿側にも明記する）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
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

/// 実在の自リポジトリ URL（フッターリンクの方針、モジュール doc 参照）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";
/// 実在の仕様リポジトリ URL。
const SPEC_REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend-spec";

/// 架空の API エンドポイント 1 件（HTTP メソッド・表示名）。`href` は
/// 持たない（モジュール doc「エンドポイント行はリンクにしない」節）。
struct Endpoint {
    method: &'static str,
    name: &'static str,
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
            },
            Endpoint {
                method: "GET",
                name: "クイックスタート",
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
            },
            Endpoint {
                method: "DELETE",
                name: "トークン失効",
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
            },
            Endpoint {
                method: "POST",
                name: "プロジェクト作成",
            },
            Endpoint {
                method: "PATCH",
                name: "プロジェクト更新",
            },
            Endpoint {
                method: "DELETE",
                name: "プロジェクト削除",
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
            },
            Endpoint {
                method: "POST",
                name: "メンバー招待",
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

/// エンドポイント 1 件分の `nav_list::item`（表示名 + 右端メソッド
/// バッジ）。実在しないページへの `href` を持たない静的な表示要素
/// （モジュール doc「エンドポイント行はリンクにしない」節）。
fn endpoint_item(endpoint: &Endpoint) -> Node {
    nav_list::item(
        vec![],
        vec![span(
            vec![("data-blocks-docs-layout-sidebar-api-link", "")],
            vec![
                span(vec![], vec![text(endpoint.name)]),
                method_badge(endpoint.method),
            ],
        )],
    )
}

/// カテゴリ見出し（`h3`）。`search-input`/`search-ai` 両 variant で
/// 同一の見出しレベルに統一する（モジュール doc「見出しレベルの統一」節）。
fn category_heading(label: &'static str) -> Node {
    el(
        "h3",
        vec![("data-blocks-docs-layout-sidebar-api-category-heading", "")],
        vec![text(label)],
    )
}

/// `search-input` variant 用: カテゴリごとに常時展開の見出し + リストを
/// 並べる（[`category_heading`]/`nav_list::list`）。
fn static_category(category: &Category) -> Node {
    let items: Vec<Node> = category.endpoints.iter().map(endpoint_item).collect();
    div(
        vec![],
        vec![
            category_heading(category.label),
            nav_list::list(vec![], items),
        ],
    )
}

/// `search-ai` variant 用: カテゴリごとに開閉式グループで包む（全件
/// [`OpenState::Open`] + `disabled: true` 固定、モジュール doc「静的表示」
/// 節）。
fn collapsible_category(category: &Category) -> Node {
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
    let items: Vec<Node> = category.endpoints.iter().map(endpoint_item).collect();
    let content = collapsible::content(
        state,
        true,
        Some(content_id.as_str()),
        vec![("data-blocks-docs-layout-sidebar-api-group-content", "")],
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
/// 無 JS では機能しないため `disabled: true` で固定する（モジュール doc
/// 「`search-input` の検索欄も disabled 固定にする」節）。
fn search_input_top() -> Node {
    let field = FieldProps {
        id: "blocks-docs-layout-sidebar-api-query-search-input",
        ids: FieldIds::default(),
        disabled: true,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: false,
    };
    let group_props = InputGroupProps {
        disabled: true,
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
                    vec![("data-blocks-docs-layout-sidebar-api-search-addon", "")],
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
                        ("data-blocks-docs-layout-sidebar-api-search-input", ""),
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
    let viewport_label = format!("API エンドポイント一覧（{variant}）");

    let category_nodes: Vec<Node> = CATEGORIES
        .iter()
        .map(|category| {
            if use_collapsible {
                collapsible_category(category)
            } else {
                static_category(category)
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
                            vec![div(
                                vec![("data-blocks-docs-layout-sidebar-api-categories", "")],
                                category_nodes,
                            )],
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
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/docs-layout-sidebar-api/",
    title: "docs-layout-sidebar-api",
    category: BlockCategory::DocsLayout,
    rust_source: "crates/docs-site/src/blocks/docs/docs_layout/docs_layout_sidebar_api.rs",
    demo_class: "blocks-docs-layout-sidebar-api",
    parts: &[
        Part {
            label: "Input Group",
            path: "/themes/input-group/",
        },
        Part {
            label: "Input",
            path: "/themes/input/",
        },
        Part {
            label: "Scroll Area",
            path: "/themes/scroll-area/",
        },
        Part {
            label: "Nav List",
            path: "/themes/nav-list/",
        },
        Part {
            label: "Badge",
            path: "/themes/badge/",
        },
        Part {
            label: "Collapsible",
            path: "/themes/collapsible/",
        },
        Part {
            label: "Link",
            path: "/themes/link/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Icon",
            path: "/themes/icon/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `docs_layout_sidebar_api` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節）。`--fandhe-*` トークンのみを使い、
/// 未定義テーマに備えてフォールバック値を併記する。セレクタは
/// `.blocks-docs-layout-sidebar-api-*` / `[data-blocks-docs-layout-sidebar-api-*]`、
/// および styled 部品の `[data-scope][data-part]` セレクタとの複合セレクタ
/// のみを用いる（モジュール doc「固定フッター」節の 3 段構成・「静的表示」
/// 節の disabled 中和を実現する）。
const LAYOUT_CSS: &str = "\
.blocks-docs-layout-sidebar-api-stack {\n  display: flex;\n  flex-wrap: wrap;\n  gap: var(--fandhe-space-6);\n}\n\
.blocks-docs-layout-sidebar-api-caption {\n  flex-basis: 100%;\n  margin: 0;\n  font-size: var(--fandhe-font-font-size-sm, 0.875rem);\n  color: var(--fandhe-color-fg-muted);\n}\n\
[data-blocks-docs-layout-sidebar-api-shell] {\n  display: flex;\n  flex-direction: column;\n  inline-size: min(100%, 18rem);\n  block-size: 32rem;\n  border: 1px solid var(--fandhe-color-border);\n  border-radius: var(--fandhe-radius-lg);\n  overflow: hidden;\n  background: var(--fandhe-color-bg);\n}\n\
[data-blocks-docs-layout-sidebar-api-top] {\n  flex: none;\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n  padding: var(--fandhe-space-3) var(--fandhe-space-4);\n  border-block-end: 1px solid var(--fandhe-color-border);\n}\n\
[data-blocks-docs-layout-sidebar-api-search-group] {\n  flex: 1;\n}\n\
[data-blocks-docs-layout-sidebar-api-scroll] {\n  flex: 1;\n  min-block-size: 0;\n  padding: var(--fandhe-space-2) 0;\n}\n\
[data-blocks-docs-layout-sidebar-api-scroll] [data-scope=\"scroll-area\"][data-part=\"root\"] {\n  block-size: 100%;\n}\n\
[data-blocks-docs-layout-sidebar-api-scroll] [data-scope=\"scroll-area\"][data-part=\"content\"] {\n  padding: 0 var(--fandhe-space-4);\n}\n\
[data-blocks-docs-layout-sidebar-api-categories] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
[data-blocks-docs-layout-sidebar-api-link] {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n}\n\
[data-blocks-docs-layout-sidebar-api-method] {\n  margin-inline-start: auto;\n  font-family: var(--fandhe-font-font-family-mono, monospace);\n}\n\
[data-blocks-docs-layout-sidebar-api-group-heading] {\n  margin: 0;\n  font-size: inherit;\n  font-weight: inherit;\n}\n\
[data-blocks-docs-layout-sidebar-api-category-heading] {\n  margin: 0;\n  font-size: var(--fandhe-font-font-size-sm);\n  font-weight: var(--fandhe-font-font-weight-medium);\n  color: var(--fandhe-color-fg-muted);\n}\n\
[data-scope=\"collapsible\"][data-part=\"trigger\"][data-blocks-docs-layout-sidebar-api-group-trigger] {\n  display: flex;\n  align-items: center;\n  justify-content: flex-start;\n  gap: var(--fandhe-space-2);\n  inline-size: 100%;\n  padding: 0;\n  border-radius: 0;\n  text-align: start;\n  font-size: var(--fandhe-font-font-size-sm);\n  font-weight: var(--fandhe-font-font-weight-medium);\n  color: var(--fandhe-color-fg-muted);\n}\n\
[data-scope=\"collapsible\"][data-part=\"trigger\"][data-blocks-docs-layout-sidebar-api-group-trigger][data-state=\"open\"] {\n  color: var(--fandhe-color-fg-muted);\n}\n\
[data-scope=\"collapsible\"][data-part=\"trigger\"][data-blocks-docs-layout-sidebar-api-group-trigger][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
[data-scope=\"collapsible\"][data-part=\"indicator\"][data-blocks-docs-layout-sidebar-api-group-chevron] {\n  margin-inline-start: auto;\n}\n\
[data-scope=\"collapsible\"][data-part=\"content\"][data-blocks-docs-layout-sidebar-api-group-content] {\n  margin-top: 0;\n  padding: 0;\n  border: 0;\n  border-radius: 0;\n  color: inherit;\n}\n\
[data-scope=\"collapsible\"][data-part=\"content\"][data-blocks-docs-layout-sidebar-api-group-content][data-disabled] {\n  color: inherit;\n}\n\
[data-blocks-docs-layout-sidebar-api-footer] {\n  flex: none;\n  display: flex;\n  flex-wrap: wrap;\n  gap: var(--fandhe-space-3);\n  padding: var(--fandhe-space-3) var(--fandhe-space-4);\n  border-block-start: 1px solid var(--fandhe-color-border);\n  background: var(--fandhe-color-bg-subtle);\n  font-size: var(--fandhe-font-font-size-sm, 0.875rem);\n}\n\
[data-scope=\"button\"][data-part=\"root\"][data-blocks-docs-layout-sidebar-api-search-trigger][data-disabled],\n[data-scope=\"button\"][data-part=\"root\"][data-blocks-docs-layout-sidebar-api-ai-trigger][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
[data-scope=\"field\"][data-part=\"input\"][data-blocks-docs-layout-sidebar-api-search-input][data-disabled],\n[data-scope=\"input-group\"][data-part=\"addon\"][data-blocks-docs-layout-sidebar-api-search-addon][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, CATEGORIES, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// 9 部品の `data-scope` が揃い、`type="button"` があり、`<form>`・
    /// `href="#"`・`data:` src を持たないこと（`input` パーツは headless
    /// `field::input` へ委譲するため `data-scope="field"` として現れる）。
    #[test]
    fn demo_composes_expected_parts() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"input-group\"",
            "data-scope=\"field\"",
            "data-scope=\"scroll-area\"",
            "data-scope=\"nav-list\"",
            "data-scope=\"badge\"",
            "data-scope=\"collapsible\"",
            "data-scope=\"link\"",
            "data-scope=\"button\"",
            "data-scope=\"icon\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert!(html.contains(r#"type="button""#));
        assert!(!html.contains("<form"));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("src=\"data:"));
    }

    /// 2 variant がそれぞれ 1 回ずつ描画され、caption が 2 件あること。
    #[test]
    fn demo_renders_both_variants() {
        let html = render(&demo());
        for variant in ["search-input", "search-ai"] {
            assert!(
                html.contains(&format!(
                    "data-blocks-docs-layout-sidebar-api-variant=\"{variant}\""
                )),
                "variant {variant} should render"
            );
        }
        assert_eq!(
            html.matches("blocks-docs-layout-sidebar-api-caption")
                .count(),
            2
        );
    }

    /// フッターのリンクが各 3 件、合計 6 件あること。
    #[test]
    fn demo_renders_six_footer_links() {
        let html = render(&demo());
        assert_eq!(
            html.matches("data-blocks-docs-layout-sidebar-api-footer")
                .count(),
            2
        );
        assert_eq!(html.matches(r#">リポジトリ<"#).count(), 2);
        assert_eq!(html.matches(r#">仕様リポジトリ<"#).count(), 2);
        assert_eq!(html.matches(r#">Issue 一覧<"#).count(), 2);
    }

    /// `search-ai` variant の開閉式グループがすべて open（`hidden` を
    /// 持たない）で、trigger が `aria-expanded="true"` であること。
    #[test]
    fn search_ai_groups_are_all_open() {
        let html = render(&demo());
        let expected = CATEGORIES.len();
        assert_eq!(
            html.matches("data-blocks-docs-layout-sidebar-api-group-trigger")
                .count(),
            expected
        );
        assert_eq!(html.matches(r#"aria-expanded="true""#).count(), expected);
        assert_eq!(
            html.matches("data-blocks-docs-layout-sidebar-api-group-heading")
                .count(),
            expected
        );
    }

    /// バッジの件数がエンドポイント総数と一致すること。
    #[test]
    fn badge_count_matches_total_endpoints() {
        let html = render(&demo());
        let total_endpoints: usize = CATEGORIES.iter().map(|c| c.endpoints.len()).sum();
        // 2 variant 分で 2 倍になる。
        assert_eq!(
            html.matches("data-blocks-docs-layout-sidebar-api-method")
                .count(),
            total_endpoints * 2
        );
    }

    /// エンドポイント行は実在しないページへのリンクを持たない静的表示
    /// 要素であること（codex-review P1 指摘対応）。`aria-current="page"`
    /// も付与しない（codex-review P2 指摘対応、現在ページという概念を
    /// 静的 Demo に持ち込まない）。
    #[test]
    fn endpoint_items_are_static_without_current_marking() {
        let html = render(&demo());
        assert!(!html.contains("aria-current"));
        assert!(!html.contains("data-current"));
    }

    /// `search-input`/`search-ai` 両 variant のカテゴリ見出しが `h3` に
    /// 統一されていること（Bugbot 指摘対応、見出し階層の非単調な混在を
    /// 防ぐ）。
    #[test]
    fn category_headings_are_unified_at_h3() {
        let html = render(&demo());
        let expected = CATEGORIES.len() * 2; // 2 variant 分。
        assert_eq!(
            html.matches("data-blocks-docs-layout-sidebar-api-category-heading")
                .count()
                + html
                    .matches("data-blocks-docs-layout-sidebar-api-group-heading")
                    .count(),
            expected
        );
        assert!(!html.contains("<h2"));
    }

    /// 検索トリガー・AI 質問ボタンが disabled 固定であること。
    #[test]
    fn search_ai_top_controls_are_disabled() {
        let html = render(&demo());
        for hook in [
            "data-blocks-docs-layout-sidebar-api-search-trigger",
            "data-blocks-docs-layout-sidebar-api-ai-trigger",
        ] {
            assert!(html.contains(hook));
        }
        assert!(html.contains(r#"aria-label="AI に質問する""#));
    }

    /// `search-input` variant の検索欄が disabled 固定であること
    /// （Codex 指摘対応、PR #3548。無 JS では機能しない検索欄を操作可能に
    /// 見せない）。
    #[test]
    fn search_input_field_is_disabled() {
        let html = render(&demo());
        assert!(html.contains("data-blocks-docs-layout-sidebar-api-search-input"));
        assert!(html.contains("data-blocks-docs-layout-sidebar-api-search-addon"));
        // disabled 固定の `input`/`addon` が存在し、中和用セレクタが
        // 両方を一致させること（`data-disabled` 存在属性）。
        assert_eq!(
            html.matches("data-blocks-docs-layout-sidebar-api-search-input")
                .count(),
            1
        );
    }

    /// カテゴリ一覧がリンクを持たない静的表示であるため `nav` ランドマーク
    /// を使わないこと（Codex 指摘対応、PR #3548。`nav_list::root` の用途外
    /// 使用をやめ、ランドマークは `scroll_area::viewport` の
    /// `role="region"` のみに一本化する）。
    #[test]
    fn category_list_does_not_use_nav_landmark() {
        let html = render(&demo());
        assert!(!html.contains("<nav"));
        assert_eq!(
            html.matches("data-blocks-docs-layout-sidebar-api-categories")
                .count(),
            2
        );
    }

    /// [`LAYOUT_CSS`] が `<` を含まず、disabled 中和・固定フッターの
    /// flex 規則を持つこと。
    #[test]
    fn layout_css_has_expected_rules() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("block-size: 32rem;"));
        assert!(LAYOUT_CSS.contains("flex: 1;\n  min-block-size: 0;"));
        assert!(LAYOUT_CSS.contains("opacity: 1;"));
    }

    /// [`LAYOUT_CSS`] の各規則を `(セレクタ, 宣言ブロック)` へ分解する
    /// （テスト専用の簡易パーサ。ネストした `@` 規則は使っていない）。
    fn rules() -> Vec<(String, String)> {
        LAYOUT_CSS
            .split('}')
            .filter_map(|rule| {
                let (selector, body) = rule.split_once('{')?;
                Some((selector.trim().to_owned(), body.trim().to_owned()))
            })
            .collect()
    }

    /// collapsible パーツを狙う規則はすべて、同一要素上でレシピ属性
    /// `[data-scope="collapsible"][data-part="<part>"]` と block フック属性を
    /// 連結していること（Bugbot 指摘対応、PR #3548。フック属性だけの
    /// セレクタはレシピ base〔詳細度 2〕に負けて no-op になるため禁止）。
    #[test]
    fn collapsible_overrides_chain_recipe_attributes_on_the_same_element() {
        let parts = [
            (
                "trigger",
                "data-blocks-docs-layout-sidebar-api-group-trigger",
            ),
            (
                "indicator",
                "data-blocks-docs-layout-sidebar-api-group-chevron",
            ),
            (
                "content",
                "data-blocks-docs-layout-sidebar-api-group-content",
            ),
        ];
        let mut seen = 0;
        for (selector, _) in rules() {
            for (part, hook) in parts {
                if !selector.contains(hook) {
                    continue;
                }
                seen += 1;
                let prefix = format!("[data-scope=\"collapsible\"][data-part=\"{part}\"][{hook}]");
                assert!(
                    selector.starts_with(&prefix),
                    "selector `{selector}` must chain the recipe attributes before `{hook}`"
                );
                // 連結は同一要素上に限る（子孫結合子で祖先に退避しない）。
                assert!(
                    !selector.contains(' ') && !selector.contains('>'),
                    "selector `{selector}` must not use combinators"
                );
            }
        }
        assert!(
            seen >= parts.len(),
            "every collapsible part should be overridden"
        );
    }

    /// trigger の open 状態上書きがレシピの accent 色（詳細度 3）より高い
    /// 詳細度でミュート色を宣言し、`search-input` 側の
    /// [`category_heading`] と見出し色が揃うこと（Bugbot 指摘対応）。
    #[test]
    fn open_trigger_color_override_outranks_recipe_accent() {
        let selector = "[data-scope=\"collapsible\"][data-part=\"trigger\"]\
             [data-blocks-docs-layout-sidebar-api-group-trigger][data-state=\"open\"]";
        let (_, body) = rules()
            .into_iter()
            .find(|(s, _)| s == selector)
            .expect("open-state trigger override should exist");
        assert!(body.contains("color: var(--fandhe-color-fg-muted);"));
        // 見出し本体（`category_heading`）と同じミュート色・同じ書体規模。
        let (_, heading) = rules()
            .into_iter()
            .find(|(s, _)| s == "[data-blocks-docs-layout-sidebar-api-category-heading]")
            .expect("category heading rule should exist");
        for decl in [
            "color: var(--fandhe-color-fg-muted);",
            "font-size: var(--fandhe-font-font-size-sm);",
            "font-weight: var(--fandhe-font-font-weight-medium);",
        ] {
            assert!(heading.contains(decl));
            let (_, trigger) = rules()
                .into_iter()
                .find(|(s, _)| {
                    s == "[data-scope=\"collapsible\"][data-part=\"trigger\"]\
                          [data-blocks-docs-layout-sidebar-api-group-trigger]"
                })
                .expect("base trigger override should exist");
            assert!(
                trigger.contains(decl),
                "trigger override should declare `{decl}`"
            );
        }
    }

    /// content の disclosure カード風の既定（枠線・パディング・角丸・
    /// `[data-disabled]` 時のミュート文字色）がサイドバーへ漏れないこと
    /// （Bugbot 指摘対応）。content 要素にはフック属性が実際に付く。
    #[test]
    fn content_override_neutralizes_recipe_card_chrome() {
        let base = "[data-scope=\"collapsible\"][data-part=\"content\"]\
                    [data-blocks-docs-layout-sidebar-api-group-content]";
        let (_, body) = rules()
            .into_iter()
            .find(|(s, _)| s == base)
            .expect("content override should exist");
        for decl in [
            "margin-top: 0;",
            "padding: 0;",
            "border: 0;",
            "border-radius: 0;",
            "color: inherit;",
        ] {
            assert!(
                body.contains(decl),
                "content override should declare `{decl}`"
            );
        }
        let (_, disabled) = rules()
            .into_iter()
            .find(|(s, _)| *s == format!("{base}[data-disabled]"))
            .expect("disabled content override should exist");
        assert!(disabled.contains("color: inherit;"));

        let html = render(&demo());
        assert_eq!(
            html.matches("data-blocks-docs-layout-sidebar-api-group-content")
                .count(),
            CATEGORIES.len()
        );
    }

    /// ルート class（`demo_class` とは別名）が [`demo`] の出力へ実際に
    /// 現れること。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-docs-layout-sidebar-api-stack\""));
        assert_ne!(
            super::BLOCK.demo_class,
            "blocks-docs-layout-sidebar-api-stack"
        );
    }
}
