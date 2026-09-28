//! `profile-detail-datalist` block（イシュー #2937。Application / Profile
//! カテゴリ、最初の block）。上段のアバター・氏名・役職・操作ボタン群と、
//! 下段の連絡先・経歴の定義リストを合成する。主参照 R0219（代表構成）を
//! 軸に、R0221（操作 4 個 + 横並び定義リスト）を集約する。
//! `_/blocks-intake/` の対応ファイルは本イシュー着手時点で本 worktree に
//! 存在しないため、原稿・本コメントには対応表 ID のみを記す
//! （`list-title-meta`〔イシュー #2925〕と同じ扱い）。
//!
//! # 使用部品
//!
//! `avatar` / `heading` / `text` / `button` / `data-list` / `badge` /
//! `icon` の 7 部品を合成する（[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が
//! 検証する）。新しい UI 部品は追加しない。
//!
//! # 2 版と集約元の対応（原稿「原案差分メモ」節と対になる索引）
//!
//! - **A（代表構成）**: R0219。操作ボタン 2 個（通話・メッセージ）+
//!   縦積み（[`fandhe_frontend_pre_styled_ui::data_list::DataListOrientation::Vertical`]）
//!   の定義リスト 2 セクション（連絡先・経歴）
//! - **B（操作 4 個 + 横並び定義リスト）**: R0221。操作ボタン 4 個
//!   （通話・メッセージ・予定を追加・共有）+ 横並び
//!   （[`fandhe_frontend_pre_styled_ui::data_list::DataListOrientation::Horizontal`]）
//!   の定義リスト 1 本に統合
//!
//! # 区切り線・狭幅の縦積みは block 側 CSS が描く理由
//!
//! [`fandhe_frontend_pre_styled_ui::data_list`] は `divideY`
//! （区切り線ユーティリティ）を意図的に非採用としている（同モジュール doc
//! 「意図的に追随しない点」節）。本 block は版 B の横並び定義リストへ行間
//! 罫線を求めるため、[`LAYOUT_CSS`] 側で `data-blocks-profile-detail-
//! datalist-list` 配下の `[data-part="item"]` へ罫線を宣言する
//! （`description_list_horizontal` と同型の判断）。
//!
//! # `class` と `data-*` の使い分け
//!
//! [`fandhe_frontend_pre_styled_ui::avatar::root`]・
//! [`fandhe_frontend_pre_styled_ui::button::button`]・
//! [`fandhe_frontend_pre_styled_ui::data_list::root`] はいずれも
//! `drop_class_attr` で呼び出し側 `class` を除去してから内部 variant
//! クラスと合成するため、これらへの CSS フックは `data-*` 属性で渡す
//! （`data-blocks-profile-detail-datalist-*`）。レイアウト用ラッパー
//! （見出し行・操作群・セクション）は素の `<div>` のため
//! `class="blocks-profile-detail-datalist-*"` を使う
//! （`description_list_horizontal` と同型の判断）。
//!
//! # 狭い幅では操作ボタン群を折り返し・横並びリストを縦積みにする（`@container`）
//!
//! Demo 枠の幅はビューポート幅と一致しないため、`@container`
//! （コンテナクエリ）で判定する（`description_list_horizontal` と同型の
//! パターン）。[`LAYOUT_CSS`] のラッパー
//! `.blocks-profile-detail-datalist-stack` へ `container-type:
//! inline-size` を宣言し、コンテナ幅が `36rem` 未満のとき操作群の
//! `margin-inline-start: auto` を解除して下段へ折り返し、版 B の横並び
//! リストの `item`/`item-label` を詳細度 0,3,0 の上書きで縦積みへ切り替える。
//!
//! # `<form>` を使わない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない静的表示のみで、送信処理・送信先は一切持たない。操作ボタンは
//! `button::button` の既定 `type="button"` のまま用いる。
//!
//! # アイコンは自作の単純図形
//!
//! `icon::icon` + `el("path", ...)` による線画（`stroke="currentColor"`,
//! `fill="none"`）のみで構成する（`contact_image_info` と同型の判断）。
//! 各操作ボタンはアイコンとテキストラベルを併記するため、アイコン自体は
//! 装飾用途（`IconProps::default()` の `label: None` → `aria-hidden`）とし、
//! アクセシブルネームはボタンの可視テキストが担う。
//!
//! # アバターの `alt` が氏名を担う
//!
//! [`fandhe_frontend_headless_ui::avatar::image`] は `<img>` として
//! `alt` 属性を出力するため、`role="img"`/`aria-label` の追加供給は不要
//! （`ImageStatus::Loaded` のとき画像が可視、`fallback` は不可視になる）。
//!
//! # ダミー素材について
//!
//! 氏名・役職・社名は `crate::blocks::dummy_assets`（架空セット）を使う。
//! メールアドレスは `example.com` ドメイン、電話番号は架空パターンとし、
//! 実在の人物・企業・PII は含まない。アバター画像はビルド時生成の同梱
//! SVG（[`dummy_assets::AVATAR_SRC`]）を使う（外部 URL・`data:` URI は
//! 使わない）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::badge::{badge, BadgeProps, BadgeVariant};
use fandhe_frontend_pre_styled_ui::button::{button, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::data_list::{self, DataListOrientation, DataListProps};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::recipe::Size;
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};

/// 自作の幾何アイコン（線画。モジュール doc「アイコンは自作の単純図形」
/// 節参照）。`path` へ `fill="none"` + `stroke="currentColor"` を明示し、
/// `icon` の `<svg>` 側が固定で持つ `fill="currentColor"`（塗り面）を
/// 上書きして線画（ストローク）として描画する。
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

/// 電話アイコン（受話器: 角丸の折れ線）。
fn phone_icon() -> Node {
    geo_icon("M6 4c-1 0-2 1-2 2 0 8 6 14 14 14 1 0 2-1 2-2v-3l-4-1-2 2c-2-1-4-3-5-5l2-2-1-4z")
}

/// メールアイコン（封筒 + V 字の折り返し線）。
fn mail_icon() -> Node {
    geo_icon("M3 6h18v12H3z M3 7l9 6 9-6")
}

/// カレンダーアイコン（角丸長方形 + 上端の 2 本のタブ）。
fn calendar_icon() -> Node {
    geo_icon("M4 5h16v16H4z M4 9h16 M8 3v4 M16 3v4")
}

/// 共有アイコン（3 節点 + 接続線）。
fn share_icon() -> Node {
    geo_icon(
        "M6 12a2 2 0 1 0 0-4 2 2 0 0 0 0 4z M18 6a2 2 0 1 0 0-4 2 2 0 0 0 0 4z \
         M18 22a2 2 0 1 0 0-4 2 2 0 0 0 0 4z M8 11l8-5 M8 13l8 5",
    )
}

/// アイコン + テキストラベルの操作ボタン（アイコンは装飾用途、
/// アクセシブルネームは可視テキストが担う）。
fn action_button(variant: ButtonVariant, icon_node: Node, label: &'static str) -> Node {
    button(
        &ButtonProps {
            variant,
            size: Size::Sm,
            ..ButtonProps::default()
        },
        vec![],
        vec![icon_node, text(label)],
    )
}

/// アバター・氏名・役職・ステータスバッジ・操作ボタン群を束ねる見出し行。
/// `actions` は呼び出し側で組んだボタン列（版 A/B で個数が異なる）。
fn profile_header(
    name: &'static str,
    title: &'static str,
    company: &'static str,
    actions: Vec<Node>,
) -> Node {
    div(
        vec![("class", "blocks-profile-detail-datalist-header")],
        vec![
            avatar::root(
                &AvatarProps {
                    size: Size::Xl,
                    ..AvatarProps::default()
                },
                vec![],
                vec![
                    avatar::image(ImageStatus::Loaded, dummy_assets::AVATAR_SRC, name, vec![]),
                    avatar::fallback(
                        ImageStatus::Loaded,
                        vec![],
                        vec![text(name.chars().take(1).collect::<String>())],
                    ),
                ],
            ),
            div(
                vec![("class", "blocks-profile-detail-datalist-identity")],
                vec![
                    heading(
                        HeadingLevel::H3,
                        &HeadingProps::default(),
                        vec![],
                        vec![text(name)],
                    ),
                    div(
                        vec![("class", "blocks-profile-detail-datalist-identity-meta")],
                        vec![
                            styled_text::text(
                                &TextProps {
                                    variant: TextVariant::Muted,
                                    size: TextSize::Sm,
                                    ..TextProps::default()
                                },
                                vec![],
                                vec![text(format!("{title} · {company}"))],
                            ),
                            badge(
                                &BadgeProps {
                                    variant: BadgeVariant::Subtle,
                                    ..BadgeProps::default()
                                },
                                vec![],
                                vec![text("在籍中")],
                            ),
                        ],
                    ),
                ],
            ),
            div(
                vec![("class", "blocks-profile-detail-datalist-actions")],
                actions,
            ),
        ],
    )
}

/// ラベル・値の 1 行（`data_list::item` + `item-label` + `item-value`）を
/// 組み立てる。
fn row(label: &'static str, value: &'static str) -> Node {
    data_list::item(
        vec![],
        vec![
            data_list::item_label(vec![], vec![text(label)]),
            data_list::item_value(vec![], vec![text(value)]),
        ],
    )
}

/// 指定 orientation の `data_list::root`（CSS フック
/// `data-blocks-profile-detail-datalist-list` 付き）を組む。
fn list(orientation: DataListOrientation, rows: Vec<Node>) -> Node {
    data_list::root(
        DataListProps {
            orientation,
            ..DataListProps::default()
        },
        vec![("data-blocks-profile-detail-datalist-list", "")],
        rows,
    )
}

/// 見出し + 定義リストの 1 セクション。
fn section(title: &'static str, list_node: Node) -> Node {
    div(
        vec![("class", "blocks-profile-detail-datalist-section")],
        vec![
            heading(
                HeadingLevel::H4,
                &HeadingProps::default(),
                vec![],
                vec![text(title)],
            ),
            list_node,
        ],
    )
}

/// A: 代表構成（R0219）。操作ボタン 2 個 + 縦積みの定義リスト 2 セクション。
fn version_representative() -> Node {
    let name = dummy_assets::PERSON_NAMES[0];
    let header = profile_header(
        name,
        dummy_assets::JOB_TITLES[0],
        dummy_assets::COMPANY_NAMES[0],
        vec![
            action_button(ButtonVariant::Outline, phone_icon(), "通話"),
            action_button(ButtonVariant::Outline, mail_icon(), "メッセージ"),
        ],
    );
    let contact = section(
        "連絡先",
        list(
            DataListOrientation::Vertical,
            vec![
                row("メール", "haruto.fujimaki@example.com"),
                row("電話", "090-1234-5678"),
                row("所在地", "東京都渋谷区"),
                row("所属", "プロダクト開発部"),
            ],
        ),
    );
    let history = section(
        "経歴",
        list(
            DataListOrientation::Vertical,
            vec![
                row("入社", "2021-04-01"),
                row("前職", "Verdant Foundry"),
                row("資格", "情報処理安全確保支援士"),
            ],
        ),
    );
    div(
        vec![("class", "blocks-profile-detail-datalist-section")],
        vec![header, contact, history],
    )
}

/// B: 操作 4 個 + 横並び定義リスト（R0221）。連絡先・経歴を 1 本の
/// 横並びリストへ統合する。
fn version_four_actions_horizontal() -> Node {
    let name = dummy_assets::PERSON_NAMES[1];
    let header = profile_header(
        name,
        dummy_assets::JOB_TITLES[1],
        dummy_assets::COMPANY_NAMES[1],
        vec![
            action_button(ButtonVariant::Outline, phone_icon(), "通話"),
            action_button(ButtonVariant::Outline, mail_icon(), "メッセージ"),
            action_button(ButtonVariant::Ghost, calendar_icon(), "予定を追加"),
            action_button(ButtonVariant::Ghost, share_icon(), "共有"),
        ],
    );
    let rows = list(
        DataListOrientation::Horizontal,
        vec![
            row("メール", "elena.vasquez@example.com"),
            row("電話", "080-2468-1357"),
            row("所在地", "大阪府大阪市"),
            row("所属", "エンジニアリング部"),
            row("入社", "2019-07-01"),
            row("資格", "PMP"),
        ],
    );
    div(
        vec![("class", "blocks-profile-detail-datalist-section")],
        vec![header, rows],
    )
}

/// `profile-detail-datalist` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-profile-detail-datalist-stack")],
        vec![version_representative(), version_four_actions_horizontal()],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/profile-detail-datalist/",
    title: "profile-detail-datalist",
    category: BlockCategory::Profile,
    rust_source: "crates/docs-site/src/blocks/application/profile/profile_detail_datalist.rs",
    demo_class: "blocks-profile-detail-datalist",
    parts: &[
        Part {
            label: "Avatar",
            path: "/themes/avatar/",
        },
        Part {
            label: "Heading",
            path: "/themes/heading/",
        },
        Part {
            label: "Text",
            path: "/themes/text/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Data List",
            path: "/themes/data-list/",
        },
        Part {
            label: "Badge",
            path: "/themes/badge/",
        },
        Part {
            label: "Icon",
            path: "/themes/icon/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `profile_detail_datalist` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節と同型）。
///
/// `--fandhe-data-list-gap: 0` の上書きセレクタは `data-scope`/`data-part`
/// を併記する（`description_list_horizontal` と同型の判断。size
/// バリアントの詳細度 0,3,0 に勝つため）。
const LAYOUT_CSS: &str = "\
.blocks-profile-detail-datalist-stack {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-10);\n  container-type: inline-size;\n  container-name: blocks-profile-detail-datalist;\n}\n\
.blocks-profile-detail-datalist-header {\n  display: flex;\n  flex-wrap: wrap;\n  align-items: center;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-profile-detail-datalist-identity {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1);\n  min-width: 0;\n}\n\
.blocks-profile-detail-datalist-identity-meta {\n  display: flex;\n  flex-wrap: wrap;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-profile-detail-datalist-actions {\n  display: flex;\n  flex-wrap: wrap;\n  gap: var(--fandhe-space-2);\n  margin-inline-start: auto;\n}\n\
.blocks-profile-detail-datalist-section {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n}\n\
[data-scope=\"data-list\"][data-part=\"root\"][data-blocks-profile-detail-datalist-list] {\n  --fandhe-data-list-gap: 0;\n}\n\
[data-blocks-profile-detail-datalist-list] > [data-scope=\"data-list\"][data-part=\"item\"] {\n  border-top: 1px solid var(--fandhe-color-border);\n  padding-block: var(--fandhe-space-3);\n}\n\
@container blocks-profile-detail-datalist (max-width: 36rem) {\n  \
.blocks-profile-detail-datalist-actions {\n    margin-inline-start: 0;\n    width: 100%;\n  }\n  \
.blocks-profile-detail-datalist-stack [data-scope=\"data-list\"][data-part=\"item\"] {\n    flex-direction: column;\n    gap: var(--fandhe-space-1);\n  }\n  \
.blocks-profile-detail-datalist-stack [data-scope=\"data-list\"][data-part=\"item-label\"] {\n    min-width: auto;\n  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    fn demo_html() -> String {
        render(&demo())
    }

    #[test]
    fn demo_composes_expected_parts() {
        let html = demo_html();
        for scope in [
            "data-scope=\"avatar\"",
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"button\"",
            "data-scope=\"data-list\"",
            "data-scope=\"badge\"",
            "data-scope=\"icon\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
        assert_eq!(html.matches("<h3").count(), 2);
        assert_eq!(
            html.matches("fd-data-list--orientation-vertical").count(),
            2
        );
        assert_eq!(
            html.matches("fd-data-list--orientation-horizontal").count(),
            1
        );
        assert_eq!(html.matches("<button").count(), 6);
        assert_eq!(
            html.matches("data-blocks-profile-detail-datalist-list=\"\"")
                .count(),
            3
        );
    }

    #[test]
    fn no_form_submit_or_dead_links() {
        let html = demo_html();
        assert!(!html.contains("<form"));
        assert!(!html.contains("src=\"data:"));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("<script"));
        assert!(!html.contains("type=\"submit\""));
        assert!(html.contains("../../assets/blocks-demo-avatar.svg"));
    }

    #[test]
    fn layout_css_is_safe_and_stacks_on_narrow_container() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("container-type: inline-size;"));
        assert!(LAYOUT_CSS.contains("@container blocks-profile-detail-datalist (max-width: 36rem)"));
    }

    #[test]
    fn avatar_image_alt_carries_full_name() {
        let html = demo_html();
        assert!(html.contains(r#"alt="Haruto Fujimaki""#));
        assert!(html.contains(r#"alt="Elena Vasquez""#));
    }

    #[test]
    fn action_buttons_have_visible_text_labels() {
        let html = demo_html();
        for expected in ["通話", "メッセージ", "予定を追加", "共有"] {
            assert!(html.contains(expected), "missing visible label: {expected}");
        }
    }
}
