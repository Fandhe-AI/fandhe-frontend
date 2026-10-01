//! `page-heading-actions` block（イシュー #2930。親トラッキング #2730
//! 「Blocks 目的別パーツ拡充」配下、phase:3）。「左に見出し・右に操作
//! ボタン」を持つページ見出しの合成例。
//!
//! # 使用部品
//!
//! `heading` / `text` / `badge` / `button` / `button_group` / `menu` /
//! `breadcrumb` / `link` / `input_group` / `input` / `icon` / `avatar` の
//! 12 部品のみを合成する（[`BLOCK`] の `parts` に一致させる契約）。新しい
//! UI 部品は追加しない。
//!
//! # 5 インスタンスで派生形を表現する（無 JS のため静的併記）
//!
//! 1 block・5 インスタンス縦積みの構成で、集約元の差分を次のとおり表す。
//!
//! - **A（基本形）**: 見出し・バッジ・説明 + 副操作ボタン・主操作ボタン・
//!   三点メニュー。
//! - **B（区画見出し）**: 下罫線付き。装飾アイコンのみのボタン列。
//! - **C（パンくず + ボタン群）**: 上段にパンくず、右に `button_group`（
//!   アイコンのみの設定ボタン + テキストボタン 2 個）+ 三点メニュー。
//! - **D（検索入力グループ）**: 右の操作列を「検索アイコン付き入力 +
//!   並べ替えボタン」の `input_group` へ差し替え。
//! - **E（戻るリンク + アカウントメニュー）**: 上段に戻るリンク、右に
//!   アバターを内包するアカウントメニュー。
//!
//! # 狭幅ではボタン列を縦積みにし、一部操作をメニューへ集約する
//!
//! `header` は既定で縦積み（`flex-direction: column`）、`40rem` 以上で
//! 左右配置（`row` + `space-between`）へ切り替える（他の block と同じ
//! リテラル値。テーマの breakpoint トークンは `@media` 条件式の中では
//! 解決できない）。A・C の副操作ボタンには
//! `data-blocks-page-heading-actions-collapsible` を付与し、`40rem` 未満
//! では非表示にする（無 JS のため静的な集約）。同じ操作は各インスタンスの
//! 三点メニューにも常に項目として含めており、狭幅では三点メニュー経由で
//! 到達できる体裁を保つ。D の検索 `input_group` は狭幅で全幅、`40rem`
//! 以上で `18rem` に収める。
//!
//! # メニューは無 JS のため閉じた状態で固定する
//!
//! 開閉状態機械・実際のポップアップ表示はクライアント配線層（wasm-full）の
//! 責務であり、無 JS の docs サイトでは `OpenState::Closed` の静的表示のみを
//! 描画する（`card_heading_toolbar.rs::overflow_menu`・
//! `dashboard_01.rs::user_menu` と同型の構成: `root` は `trigger` と
//! `positioner`（`content` を内包）を子に持つ）。三点メニュー 4 件の
//! `content` id はインスタンスごとに一意にする（id 重複・宙ぶらりん
//! `aria-controls` 参照の回避、`blocks_contract.rs` の汎用チェック参照）。
//!
//! # アイコンは自作の単純幾何図形
//!
//! `sidebar_03.rs::geo_icon` と同型で、`icon::icon` へ独自の `<path d="...">`
//! を渡すのみ（実在ブランドのアイコンセットは使わない）。アイコンのみの
//! ボタンは可視ラベルを持たないため `aria-label` でアクセシブル名を確保
//! する（`field`/`visually_hidden` を使用部品に含めないため。
//! `dashboard_01.rs::user_menu` と同じ回避方法）。
//!
//! # `text` の名前衝突
//!
//! `fandhe_frontend_pre_styled_ui::text::text` と
//! `fandhe_frontend_core::text` が同名のため、styled 側を `styled_text`
//! として取り込む（`crate::blocks` 内の他 block と同じ回避方法）。
//!
//! # `<form>` を持たない・データ取得/送信を行わない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である。ボタンは `button::button` の既定 `type="button"` のまま送信先を
//! 持たない（`docs/policy/intentional-non-adoption.md` §3.25：バリデーション・
//! 送信処理は UI コンポーネント層の責務外）。リンクは `href="#"` を使わず、
//! `/blocks/` 索引・パンくずの上位ページなど実在の相対パスのみを指す。
//! 文言はすべて独自の架空の日本語ダミー（実企業名・実クレデンシャル・PII を
//! 含まない）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::breadcrumb::{self, BreadcrumbVariant};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::button_group;
use fandhe_frontend_pre_styled_ui::field::{FieldIds, FieldProps};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::input::{self, InputProps};
use fandhe_frontend_pre_styled_ui::input_group::{self, InputGroupAlign, InputGroupProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::menu::{self, OpenState};
use fandhe_frontend_pre_styled_ui::recipe::ColorPalette;
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};
use fandhe_frontend_pre_styled_ui::{Orientation, Size};

/// 自作の単純な幾何アイコン（`cta_split_actions.rs::geo_icon` と同型。
/// モジュール doc「アイコンは自作の単純幾何図形」参照）。
///
/// 本ファイルの `path_d` はいずれも開いた線分（moveto/lineto のみで
/// `z` 閉曲線を持たない）ため、`icon::icon` 既定の `fill="currentColor"`
/// のままでは面積ゼロで何も描画されない（`sidebar_03.rs::geo_icon` が
/// 使う閉じた矩形パスとの差異）。`path` 要素へ `fill="none"` +
/// `stroke="currentColor"` を上書きしてアウトライン描画にする。
fn geo_icon(path_d: &'static str) -> Node {
    icon(
        &IconProps::default(),
        vec![],
        vec![el(
            "path",
            vec![
                ("d", path_d),
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

/// 三点メニュー（無 JS のため閉じた状態で固定する。モジュール doc
/// 「メニューは無 JS のため閉じた状態で固定する」節参照）。
///
/// `collapsed_items`（`(id, ラベル)`）は `40rem` 未満で非表示になる
/// `data-blocks-page-heading-actions-collapsible` 付きボタンと同じ操作を
/// 表す項目で、共通の書き出す/複製する/削除するより前に挿入する。狭幅では
/// 三点メニュー経由でしか到達できないため、`export`/`duplicate`/`delete`
/// のみを常時列挙する構成では狭幅操作が失われる（codex/bugbot 指摘）。
fn overflow_menu(
    content_id: &'static str,
    collapsed_items: &[(&'static str, &'static str)],
) -> Node {
    let trigger = menu::trigger(
        OpenState::Closed,
        false,
        Some(content_id),
        vec![("aria-label", "その他の操作")],
        vec![text("\u{2026}")],
    );
    let mut items: Vec<Node> = collapsed_items
        .iter()
        .map(|(id, label)| menu::item(id, false, false, vec![], vec![text(*label)]))
        .collect();
    if !collapsed_items.is_empty() {
        items.push(menu::separator(vec![], vec![]));
    }
    items.extend([
        menu::item("export", false, false, vec![], vec![text("書き出す")]),
        menu::item("duplicate", false, false, vec![], vec![text("複製する")]),
        menu::separator(vec![], vec![]),
        menu::item("delete", false, false, vec![], vec![text("削除する")]),
    ]);
    let content = menu::content(OpenState::Closed, Some(content_id), None, vec![], items);
    let positioner = menu::positioner(OpenState::Closed, vec![], vec![content]);
    menu::root(
        Size::Sm,
        OpenState::Closed,
        vec![],
        vec![trigger, positioner],
    )
}

/// アカウントメニュー（trigger の子にアバターを持つ。モジュール doc
/// 「E（戻るリンク + アカウントメニュー）」節参照）。
fn account_menu(content_id: &'static str) -> Node {
    let avatar_node = avatar::root(
        &AvatarProps::default(),
        vec![],
        vec![avatar::fallback(
            ImageStatus::Error,
            vec![],
            vec![text("HF")],
        )],
    );
    let trigger = menu::trigger(
        OpenState::Closed,
        false,
        Some(content_id),
        vec![("aria-label", "アカウントメニューを開く（Haruto Fujimaki）")],
        vec![avatar_node],
    );
    let content = menu::content(
        OpenState::Closed,
        Some(content_id),
        None,
        vec![],
        vec![
            menu::item("profile", false, false, vec![], vec![text("プロフィール")]),
            menu::item("settings", false, false, vec![], vec![text("設定")]),
            menu::separator(vec![], vec![]),
            menu::item("logout", false, false, vec![], vec![text("ログアウト")]),
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

/// 見出し群（左側）。`badge`/`description` は任意（インスタンスごとに
/// 差し替える）。
fn heading_group(
    level: HeadingLevel,
    title: &'static str,
    badge: Option<Node>,
    description: Option<&'static str>,
) -> Node {
    let mut title_row_children = vec![heading(
        level,
        &HeadingProps {
            size: HeadingSize::Xl,
            ..HeadingProps::default()
        },
        vec![],
        vec![text(title)],
    )];
    if let Some(b) = badge {
        title_row_children.push(b);
    }
    let mut children = vec![div(
        vec![("data-blocks-page-heading-actions-title-row", "")],
        title_row_children,
    )];
    if let Some(d) = description {
        children.push(styled_text::text(
            &TextProps {
                variant: TextVariant::Muted,
                ..TextProps::default()
            },
            vec![],
            vec![text(d)],
        ));
    }
    div(
        vec![("data-blocks-page-heading-actions-heading-group", "")],
        children,
    )
}

/// A: 基本形（ページ見出し）。
fn instance_a() -> Node {
    let heading_group_node = heading_group(
        HeadingLevel::H1,
        "プロジェクト設定",
        Some(badge::badge(
            &BadgeProps {
                palette: ColorPalette::Success,
                ..BadgeProps::default()
            },
            vec![],
            vec![text("公開中")],
        )),
        Some("チームの権限とインテグレーションをここで管理します。"),
    );
    let actions = div(
        vec![("data-blocks-page-heading-actions-actions", "")],
        vec![
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Outline,
                    ..ButtonProps::default()
                },
                vec![("data-blocks-page-heading-actions-collapsible", "")],
                vec![text("下書きを保存")],
            ),
            button::button(&ButtonProps::default(), vec![], vec![text("変更を公開")]),
            overflow_menu(
                "blocks-page-heading-actions-menu-a",
                &[("draft-save", "下書きを保存")],
            ),
        ],
    );
    div(
        vec![
            ("data-blocks-page-heading-actions-instance", ""),
            ("data-blocks-page-heading-actions-variant", "a"),
        ],
        vec![div(
            vec![("data-blocks-page-heading-actions-header", "")],
            vec![heading_group_node, actions],
        )],
    )
}

/// B: 区画見出し（下罫線）。
fn instance_b() -> Node {
    let heading_group_node = heading_group(
        HeadingLevel::H2,
        "メンバー",
        None,
        Some("このワークスペースに参加しているメンバーの一覧です。"),
    );
    let actions = div(
        vec![("data-blocks-page-heading-actions-actions", "")],
        vec![
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Ghost,
                    ..ButtonProps::default()
                },
                vec![("aria-label", "並べ替え")],
                vec![geo_icon("M4 6h16M4 12h10M4 18h6")],
            ),
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Ghost,
                    ..ButtonProps::default()
                },
                vec![("aria-label", "表示設定")],
                vec![geo_icon("M12 4v16M4 12h16")],
            ),
        ],
    );
    div(
        vec![
            ("data-blocks-page-heading-actions-instance", ""),
            ("data-blocks-page-heading-actions-variant", "b"),
        ],
        vec![div(
            vec![
                ("data-blocks-page-heading-actions-header", ""),
                ("data-blocks-page-heading-actions-section-heading", ""),
            ],
            vec![heading_group_node, actions],
        )],
    )
}

/// C: パンくず上段 + ボタン群。
fn instance_c() -> Node {
    let breadcrumb_row = breadcrumb::root(
        Size::Md,
        BreadcrumbVariant::default(),
        Some("パンくずリスト"),
        vec![],
        vec![breadcrumb::list(
            vec![],
            vec![
                breadcrumb::item(
                    vec![],
                    vec![breadcrumb::link("../", vec![], vec![text("Blocks")])],
                ),
                breadcrumb::separator(vec![], vec![text("/")]),
                breadcrumb::item(
                    vec![],
                    vec![breadcrumb::current_link(vec![], vec![text("請求設定")])],
                ),
            ],
        )],
    );
    let heading_group_node = heading_group(HeadingLevel::H1, "請求設定", None, None);
    let button_group_node = button_group::root(
        Orientation::Horizontal,
        "請求の操作",
        vec![],
        vec![
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Ghost,
                    ..ButtonProps::default()
                },
                vec![
                    ("aria-label", "詳細設定"),
                    ("data-blocks-page-heading-actions-collapsible", ""),
                ],
                vec![geo_icon(
                    "M12 15a3 3 0 100-6 3 3 0 000 6zM4 12h2m12 0h2M12 4v2m0 12v2",
                )],
            ),
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Outline,
                    ..ButtonProps::default()
                },
                vec![("data-blocks-page-heading-actions-collapsible", "")],
                vec![text("請求書をダウンロード")],
            ),
            button::button(
                &ButtonProps::default(),
                vec![("data-blocks-page-heading-actions-primary-action", "")],
                vec![text("プランを変更")],
            ),
        ],
    );
    let actions = div(
        vec![("data-blocks-page-heading-actions-actions", "")],
        vec![
            button_group_node,
            overflow_menu(
                "blocks-page-heading-actions-menu-c",
                &[
                    ("detail-settings", "詳細設定"),
                    ("invoice-download", "請求書をダウンロード"),
                ],
            ),
        ],
    );
    div(
        vec![
            ("data-blocks-page-heading-actions-instance", ""),
            ("data-blocks-page-heading-actions-variant", "c"),
        ],
        vec![
            div(
                vec![("data-blocks-page-heading-actions-top-row", "")],
                vec![breadcrumb_row],
            ),
            div(
                vec![("data-blocks-page-heading-actions-header", "")],
                vec![heading_group_node, actions],
            ),
        ],
    )
}

/// D: 操作列差し替え（検索入力グループ）。
fn instance_d() -> Node {
    let heading_group_node = heading_group(
        HeadingLevel::H1,
        "問い合わせ一覧",
        Some(badge::badge(
            &BadgeProps {
                palette: ColorPalette::Neutral,
                ..BadgeProps::default()
            },
            vec![],
            vec![text("128 件")],
        )),
        None,
    );
    let search_field = FieldProps {
        id: "blocks-page-heading-actions-search",
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
    let search_group = input_group::root(
        &group_props,
        vec![("data-blocks-page-heading-actions-search-group", "")],
        vec![
            input_group::addon(
                InputGroupAlign::InlineStart,
                &group_props,
                vec![],
                vec![geo_icon(
                    "M11 4a7 7 0 1 0 0 14 7 7 0 0 0 0-14zm9 17-5.2-5.2",
                )],
            ),
            input::input(
                &InputProps::default(),
                &search_field,
                vec![
                    ("type", "search"),
                    ("placeholder", "問い合わせを検索"),
                    ("aria-label", "問い合わせを検索"),
                ],
            ),
            input_group::addon(
                InputGroupAlign::InlineEnd,
                &group_props,
                vec![],
                vec![input_group::button(
                    &group_props,
                    vec![],
                    vec![text("並べ替え")],
                )],
            ),
        ],
    );
    let actions = div(
        vec![("data-blocks-page-heading-actions-actions", "")],
        vec![
            search_group,
            overflow_menu("blocks-page-heading-actions-menu-d", &[]),
        ],
    );
    div(
        vec![
            ("data-blocks-page-heading-actions-instance", ""),
            ("data-blocks-page-heading-actions-variant", "d"),
        ],
        vec![div(
            vec![("data-blocks-page-heading-actions-header", "")],
            vec![heading_group_node, actions],
        )],
    )
}

/// E: 戻るリンク上段 + アカウントメニュー。
fn instance_e() -> Node {
    let back_link = link::root(
        "../",
        &LinkProps::default(),
        vec![],
        vec![geo_icon("M15 18l-6-6 6-6"), text("一覧へ戻る")],
    );
    let heading_group_node = heading_group(HeadingLevel::H1, "APIキー", None, None);
    let actions = div(
        vec![("data-blocks-page-heading-actions-actions", "")],
        vec![account_menu("blocks-page-heading-actions-menu-e")],
    );
    div(
        vec![
            ("data-blocks-page-heading-actions-instance", ""),
            ("data-blocks-page-heading-actions-variant", "e"),
        ],
        vec![
            div(
                vec![("data-blocks-page-heading-actions-top-row", "")],
                vec![back_link],
            ),
            div(
                vec![("data-blocks-page-heading-actions-header", "")],
                vec![heading_group_node, actions],
            ),
        ],
    )
}

/// `page-heading-actions` の Demo 本体（5 インスタンスを縦積みで並記する。
/// 呼び出しごとに同一の `Node` を返す純関数）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-page-heading-actions-layout")],
        vec![
            instance_a(),
            instance_b(),
            instance_c(),
            instance_d(),
            instance_e(),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/page-heading-actions/",
    title: "page-heading-actions",
    category: BlockCategory::PageHeading,
    rust_source: "crates/docs-site/src/blocks/application/page_heading/page_heading_actions.rs",
    demo_class: "blocks-page-heading-actions",
    parts: &[
        Part {
            label: "Heading",
            path: "/themes/heading/",
        },
        Part {
            label: "Text",
            path: "/themes/text/",
        },
        Part {
            label: "Badge",
            path: "/themes/badge/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Button Group",
            path: "/themes/button-group/",
        },
        Part {
            label: "Menu",
            path: "/themes/menu/",
        },
        Part {
            label: "Breadcrumb",
            path: "/themes/breadcrumb/",
        },
        Part {
            label: "Link",
            path: "/themes/link/",
        },
        Part {
            label: "Input Group",
            path: "/themes/input-group/",
        },
        Part {
            label: "Input",
            path: "/themes/input/",
        },
        Part {
            label: "Icon",
            path: "/themes/icon/",
        },
        Part {
            label: "Avatar",
            path: "/themes/avatar/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `page_heading_actions` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節と同型で、本ファイル内 private 定数として
/// `super::stylesheet` 経由の `push_css` で連結される）。
///
/// セレクタは `.blocks-page-heading-actions-*` と
/// `[data-blocks-page-heading-actions-*]` のみを用いる。
///
/// # ルート class を `demo_class` と別名にする理由
///
/// [`Block::demo_class`] は `blocks-page-heading-actions` だが、`demo()`
/// が返すルート `div` の class は `blocks-page-heading-actions-layout`
/// という別名にする（`card_heading_toolbar` 等と同じ Bugbot 教訓の回避）。
///
/// # `data-blocks-page-heading-actions-primary-action` に `!important` を使う理由
///
/// C の `button_group` は `40rem` 未満で先頭 2 個の子（アイコンのみの
/// 「詳細設定」・「請求書をダウンロード」、いずれも
/// `data-blocks-page-heading-actions-collapsible`）を `display: none` で
/// 隠すが、`button_group`（`crate::button_group` の角丸連結規則）は
/// `:not(:first-child)`/`:not(:last-child)` という DOM 上の位置で判定する
/// 構造的擬似クラスを使う。`display: none` は要素を DOM から取り除かない
/// ため、最後に残る「プランを変更」は `display: none` の兄弟が存在する
/// 限り常に `:not(:first-child)` に一致し続け、単独表示なのに左端の角丸
/// だけが 0 のまま（接続端スタイル）になる（bugbot 指摘）。無 JS のため
/// この不一致を検知する手段は CSS のみで、`button_group` 側の規則は
/// 複数の属性セレクタ + 擬似クラスで特異度が高く（`:has()` で緩衝しても
/// 上回れない）、`!important` なしでは上書きできない。影響範囲を
/// `data-blocks-page-heading-actions-primary-action` を持つ要素 1 個の
/// 角丸・境界幅 3 プロパティのみに限定して局所化する。
const LAYOUT_CSS: &str = "\
.blocks-page-heading-actions-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n}\n\
[data-blocks-page-heading-actions-instance] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n}\n\
[data-blocks-page-heading-actions-top-row] {\n  display: flex;\n  align-items: center;\n}\n\
[data-blocks-page-heading-actions-header] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n  padding-bottom: var(--fandhe-space-4);\n}\n\
[data-blocks-page-heading-actions-section-heading] {\n  border-bottom: 1px solid var(--fandhe-color-border);\n}\n\
[data-blocks-page-heading-actions-heading-group] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n  min-width: 0;\n}\n\
[data-blocks-page-heading-actions-title-row] {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-3);\n}\n\
[data-blocks-page-heading-actions-actions] {\n  display: flex;\n  flex-wrap: wrap;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n}\n\
[data-blocks-page-heading-actions-search-group] {\n  flex: 1 1 100%;\n  min-width: 0;\n}\n\
[data-blocks-page-heading-actions-collapsible] {\n  display: none;\n}\n\
[data-blocks-page-heading-actions-primary-action] {\n  border-start-start-radius: var(--fandhe-radius-md) !important;\n  border-end-start-radius: var(--fandhe-radius-md) !important;\n  border-inline-start-width: 1px !important;\n}\n\
@media (min-width: 40rem) {\n  [data-blocks-page-heading-actions-header] {\n    flex-direction: row;\n    justify-content: space-between;\n    align-items: flex-start;\n  }\n  [data-blocks-page-heading-actions-collapsible] {\n    display: inline-flex;\n  }\n  [data-blocks-page-heading-actions-search-group] {\n    flex: 0 1 18rem;\n  }\n  [data-blocks-page-heading-actions-primary-action] {\n    border-start-start-radius: 0 !important;\n    border-end-start-radius: 0 !important;\n    border-inline-start-width: 0 !important;\n  }\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// Demo が使用部品（heading/text/badge/button/button-group/menu/
    /// breadcrumb/link/input-group/input/icon/avatar）の anatomy をすべて
    /// 実際に出力していること。
    #[test]
    fn demo_composes_expected_parts() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"badge\"",
            "data-scope=\"button\"",
            "data-scope=\"button-group\"",
            "data-scope=\"menu\"",
            "data-scope=\"breadcrumb\"",
            "data-scope=\"link\"",
            "data-scope=\"input-group\"",
            "data-scope=\"field\" data-part=\"input\"",
            "data-scope=\"icon\"",
            "data-scope=\"avatar\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
    }

    /// インスタンス数はちょうど 5 件（A〜E）。
    #[test]
    fn demo_instance_count() {
        let html = render(&demo());
        assert_eq!(
            html.matches("data-blocks-page-heading-actions-instance")
                .count(),
            5
        );
    }

    /// メニューはちょうど 4 件、いずれも閉じた状態（`aria-expanded="false"`）
    /// で固定し、trigger の `aria-controls` が content の id と一致する。
    #[test]
    fn menus_are_static_closed() {
        let html = render(&demo());
        assert_eq!(html.matches(r#"aria-expanded="false""#).count(), 4);
        for suffix in ["a", "c", "d", "e"] {
            let id = format!("blocks-page-heading-actions-menu-{suffix}");
            assert!(html.contains(&format!(r#"aria-controls="{id}""#)));
            assert!(html.contains(&format!(r#"id="{id}""#)));
        }
    }

    /// `<form>`・`href="#"`・`data:` URI・`type="submit"` を出力しない
    /// （`crate::blocks` モジュール doc）。
    #[test]
    fn demo_has_no_form_or_unsafe_output() {
        let html = render(&demo());
        for absent in ["<form", "type=\"submit\"", "href=\"#\"", "src=\"data:"] {
            assert!(!html.contains(absent), "demo should never contain {absent}");
        }
    }

    /// アイコンのみのボタン・トリガーは `aria-label` を持つ。
    #[test]
    fn icon_only_controls_have_aria_label() {
        let html = render(&demo());
        for label in [
            "並べ替え",
            "表示設定",
            "詳細設定",
            "アカウントメニューを開く（Haruto Fujimaki）",
        ] {
            assert!(
                html.contains(&format!(r#"aria-label="{label}""#)),
                "demo should contain aria-label=\"{label}\""
            );
        }
    }

    /// [`LAYOUT_CSS`] が想定するブレークポイントを持ち、`<` を含まない
    /// （REQ-1: `</style>` によるスタイル脱出を防ぐ）。
    #[test]
    fn layout_css_declares_breakpoint_and_no_angle_bracket() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("@media (min-width: 40rem)"));
    }

    /// ルート class（`demo_class` とは別名）が `demo()` の出力へ実際に
    /// 現れること。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-page-heading-actions-layout\""));
        assert_ne!(
            super::BLOCK.demo_class,
            "blocks-page-heading-actions-layout"
        );
    }
}
