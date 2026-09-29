//! `auth-oauth-consent` block（イシュー #2963。親トラッキング #2951
//! 「Blocks アプリケーション B」配下、phase:4）。OAuth 連携の同意画面
//! （連携元/連携先アプリの見出し・要求権限一覧・アカウント表示・許可/拒否
//! ボタン）を合成する。Application / Auth カテゴリ 5 件目の block（既存は
//! `login-01`/`login-04`/`signup-01`/`signup-05`）。
//!
//! # 使用部品
//!
//! `card` / `avatar` / `icon` / `list` / `separator` / `button` / `link` /
//! `heading` の 8 部品のみを合成する（[`BLOCK`] の `parts` に一致させる
//! 契約、`crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が
//! 検証する）。新しい UI 部品は追加しない。
//!
//! # 対応表 ID・参照ファイル不在について
//!
//! 主参照は対応表 ID R0250（集約元も R0250 の 1 件のみ、差分なし）。
//! `_/blocks-intake/` の対応ファイルは本イシュー着手時点で本 worktree に
//! 存在しないため、対応表 ID のみを記し、レイアウト仕様（イシュー本文の
//! 文言）のみから構成する（`page_heading_avatar.rs`〔#2931〕・
//! `list_title_meta.rs`〔#2925〕と同じ扱い）。参照元の文言・配色・装飾・
//! アイコンは持ち込まない。
//!
//! # `separator` を一覧の前後に実体出力する判断
//!
//! イシュー本文が使用部品として `separator` を明記しているため、装飾用の
//! 隣接セレクタ CSS だけで区切りを表現するのではなく、見出し↔一覧・一覧↔
//! アカウント表示の 2 箇所へ [`separator::separator`] を実体として置く。
//! 一覧項目どうしの区切り線（1 項目ごと）は `separator` 部品を `<li>` の
//! 中へ挟まずセマンティクスを壊さないため、[`LAYOUT_CSS`] の隣接兄弟
//! セレクタ（`[data-part="item"] + [data-part="item"]`）で描く
//! （`crate::list` recipe の `Plain` variant item 規則と同型の判断）。
//!
//! # `<form>` を使わない・認可処理を持たない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない。「許可」「拒否」ボタンは `button::button` の既定
//! `type="button"` のまま用い、実際の認可・トークン発行・リダイレクトは
//! 一切行わない静的な合成例である。`redirect_uri`/`client_id`/`state` の
//! ような実パラメータを模した属性・URL も出力しない（偽の認可エンドポイント
//! に見せないため）。
//!
//! # 狭幅ではカードが幅いっぱい・ボタンは縦積み
//!
//! [`LAYOUT_CSS`] はモバイルファーストで `card::footer` のボタン列を既定
//! 縦積み（`flex-direction: column`）にし、`@media (min-width: 40rem)` で
//! 横並びへ切り替える（`login_04.rs` の `@media` 前例と同型）。外枠
//! `.blocks-auth-oauth-consent-layout` は `max-width: 28rem` を持つが幅の
//! 下限は指定しないため、狭幅では親要素の幅いっぱいまで縮む。
//!
//! # 「別のアカウントを使う」等は遷移しない
//!
//! 無 JS の docs サイトのため、アカウント切り替えボタンは
//! `ButtonVariant::Link`（見た目だけリンク風の `<button type="button">`）
//! とし、実際のアカウント切り替え処理は持たない（`login_04.rs` の
//! 「パスワードを忘れた」links と同型の判断）。
//!
//! # ダミー素材について
//!
//! 連携元/連携先アプリのアイコン・アカウントのアバターはいずれも
//! `crate::blocks::dummy_assets::LOGO_SRC`/`AVATAR_SRC`（モノトーン抽象図形
//! の SVG、`build.rs` がビルド時に書き出す）を使う。実ブランドロゴは描かず、
//! 権限アイコンも自作の単純幾何パスのみで構成する（装飾専用、
//! `list::indicator` が常に `aria-hidden="true"` を固定するため意味を
//! 持たせない）。氏名は `dummy_assets::PERSON_NAMES`、メールアドレスは
//! 独自に書いた架空の文言であり、実在の人物・企業・クレデンシャルとは
//! 無関係。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::list::{self, ListType, ListVariant};
use fandhe_frontend_pre_styled_ui::separator::{self, SeparatorProps};
use fandhe_frontend_pre_styled_ui::Size;

/// 補足リンクの遷移先（外部の実在 URL、`href="#"` は使わない、
/// `page_heading_avatar.rs::REPO` と同型の判断。アクセス許可の仕組み自体を
/// 説明する実在ページは持たないため、遷移先がわかる文言「GitHub で見る」を
/// 可視テキストにする）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// 連携元/連携先アプリのアイコン（円形アバター）。`role_hint` は
/// `"source"`/`"target"` を CSS フックとして与え、実ブランドロゴは使わず
/// `dummy_assets::LOGO_SRC` を共通で使い回す。
fn app_avatar(name: &str, role_hint: &'static str) -> Node {
    let initial: String = name.chars().take(1).collect();
    avatar::root(
        &AvatarProps {
            size: Size::Lg,
            ..AvatarProps::default()
        },
        vec![
            ("role", "img"),
            ("aria-label", name),
            ("data-blocks-auth-oauth-consent-app", role_hint),
        ],
        vec![
            avatar::image(ImageStatus::Loaded, dummy_assets::LOGO_SRC, "", vec![]),
            avatar::fallback(ImageStatus::Loaded, vec![], vec![text(initial)]),
        ],
    )
}

/// 権限アイコン（自作の単純幾何パス、装飾専用）。`login_04.rs::geo_icon` と
/// 同型で、実ブランドロゴ・既存アイコンセットを複製しない。
fn scope_icon(path_d: &'static str) -> Node {
    icon(
        &IconProps {
            size: Size::Sm,
            ..IconProps::default()
        },
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

/// 要求権限 1 件（アイコン + タイトル + 説明）。`list::indicator` は常に
/// `aria-hidden="true"` を固定するため、アイコンは装飾として扱う
/// （`pricing_upgrade_card.rs::check_icon` と同型）。
fn scope_item(path_d: &'static str, title: &'static str, desc: &'static str) -> Node {
    list::item(
        vec![],
        vec![
            list::indicator(vec![], vec![scope_icon(path_d)]),
            div(
                vec![("class", "blocks-auth-oauth-consent-scope-body")],
                vec![
                    el("span", vec![], vec![text(title)]),
                    el(
                        "span",
                        vec![("class", "blocks-auth-oauth-consent-scope-desc")],
                        vec![text(desc)],
                    ),
                ],
            ),
        ],
    )
}

/// アカウント表示欄のアバター（円形、小サイズ）。
fn account_avatar(name: &str) -> Node {
    let initial: String = name.chars().take(1).collect();
    avatar::root(
        &AvatarProps {
            size: Size::Sm,
            ..AvatarProps::default()
        },
        vec![("role", "img"), ("aria-label", name)],
        vec![
            avatar::image(ImageStatus::Loaded, dummy_assets::AVATAR_SRC, "", vec![]),
            avatar::fallback(ImageStatus::Loaded, vec![], vec![text(initial)]),
        ],
    )
}

/// `auth-oauth-consent` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数。
#[must_use]
pub fn demo() -> Node {
    let name = dummy_assets::PERSON_NAMES[0];

    let apps_row = div(
        vec![("class", "blocks-auth-oauth-consent-apps")],
        vec![
            app_avatar("Northwind Notes", "source"),
            scope_icon("M5 12h14M13 6l6 6-6 6"),
            app_avatar("Aurora Calendar", "target"),
        ],
    );

    let heading_node = heading::heading(
        HeadingLevel::H3,
        &HeadingProps {
            size: HeadingSize::Lg,
            ..HeadingProps::default()
        },
        vec![],
        vec![text(
            "Northwind Notes が Aurora Calendar へのアクセスを求めています",
        )],
    );

    let scopes = list::root(
        ListType::Unordered,
        ListVariant::Plain,
        vec![("data-blocks-auth-oauth-consent-scopes", "")],
        vec![
            scope_item(
                "M12 12a4 4 0 100-8 4 4 0 000 8zM4 20c0-3.3 3.6-6 8-6s8 2.7 8 6",
                "プロフィール情報",
                "氏名とアイコン画像を読み取ります",
            ),
            scope_item(
                "M4 5h16v14H4zM4 7l8 6 8-6",
                "カレンダーの予定",
                "予定の一覧を読み取り、新規作成します",
            ),
            scope_item(
                "M6 3v4M18 3v4M4 9h16M5 5h14a1 1 0 011 1v13a1 1 0 01-1 1H5a1 1 0 01-1-1V6a1 1 0 011-1z",
                "空き時間",
                "会議の空き状況を確認します",
            ),
        ],
    );

    let account_row = div(
        vec![("class", "blocks-auth-oauth-consent-account")],
        vec![
            account_avatar(name),
            div(
                vec![],
                vec![
                    el("span", vec![], vec![text(name)]),
                    el(
                        "span",
                        vec![("class", "blocks-auth-oauth-consent-email")],
                        vec![text("yamada@example.com")],
                    ),
                ],
            ),
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Link,
                    ..ButtonProps::default()
                },
                vec![],
                vec![text("別のアカウントを使う")],
            ),
        ],
    );

    let card_node = card::root(
        CardProps::default(),
        vec![("data-blocks-auth-oauth-consent-card", "")],
        vec![
            card::header(
                vec![],
                vec![
                    apps_row,
                    heading_node,
                    card::description(vec![], vec![text("続行すると、次の権限が付与されます")]),
                ],
            ),
            card::body(
                vec![],
                vec![
                    separator::separator(
                        &SeparatorProps::default(),
                        vec![("data-blocks-auth-oauth-consent-divider", "")],
                    ),
                    scopes,
                    separator::separator(
                        &SeparatorProps::default(),
                        vec![("data-blocks-auth-oauth-consent-divider", "")],
                    ),
                    account_row,
                ],
            ),
            card::footer(
                vec![("data-blocks-auth-oauth-consent-actions", "")],
                vec![
                    button::button(
                        &ButtonProps {
                            variant: ButtonVariant::Outline,
                            ..ButtonProps::default()
                        },
                        vec![],
                        vec![text("拒否")],
                    ),
                    button::button(&ButtonProps::default(), vec![], vec![text("許可")]),
                ],
            ),
        ],
    );

    let help_row = div(
        vec![("class", "blocks-auth-oauth-consent-help")],
        vec![
            text("この連携はいつでも設定画面から取り消せます。"),
            link::root(
                REPO,
                &LinkProps {
                    external: true,
                    ..LinkProps::default()
                },
                vec![],
                vec![text("アクセス許可の仕組みを GitHub で見る")],
            ),
        ],
    );

    div(
        vec![("class", "blocks-auth-oauth-consent-layout")],
        vec![card_node, help_row],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/auth-oauth-consent/",
    title: "auth-oauth-consent",
    category: BlockCategory::Auth,
    rust_source: "crates/docs-site/src/blocks/application/auth/auth_oauth_consent.rs",
    demo_class: "blocks-auth-oauth-consent",
    parts: &[
        Part {
            label: "Card",
            path: "/themes/card/",
        },
        Part {
            label: "Avatar",
            path: "/themes/avatar/",
        },
        Part {
            label: "Icon",
            path: "/themes/icon/",
        },
        Part {
            label: "List",
            path: "/themes/list/",
        },
        Part {
            label: "Separator",
            path: "/themes/separator/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Link",
            path: "/themes/link/",
        },
        Part {
            label: "Heading",
            path: "/themes/heading/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `auth_oauth_consent` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節と同型）。
///
/// # `[data-blocks-auth-oauth-consent-actions]` の詳細度を Card recipe
/// 以上にする
///
/// `card::footer` が出力する要素は `[data-scope="card"][data-part="footer"]`
/// （詳細度 (0,2,0)）で `display: flex`（既定 row）を持つため、block 側
/// セレクタを単独の `[data-blocks-auth-oauth-consent-actions]`（詳細度
/// (0,1,0)）のままにすると Card recipe の row 方向に負けて狭幅での縦積みが
/// 成立しない（`login_04.rs`「`[data-blocks-login-04-body]` の詳細度」節と
/// 同型の判断）。このためセレクタを
/// `[data-scope="card"][data-part="footer"][data-blocks-auth-oauth-consent-actions]`
/// （詳細度 (0,3,0)）へ結合し、Card recipe を確実に上書きする。
const LAYOUT_CSS: &str = "\
.blocks-auth-oauth-consent-layout {\n  display: flex;\n  flex-direction: column;\n  align-items: stretch;\n  gap: var(--fandhe-space-4);\n  max-width: 28rem;\n  margin-inline: auto;\n}\n\
.blocks-auth-oauth-consent-apps {\n  display: flex;\n  align-items: center;\n  justify-content: center;\n  gap: var(--fandhe-space-3);\n}\n\
[data-blocks-auth-oauth-consent-scopes] {\n  list-style: none;\n  margin: 0;\n  padding: 0;\n  display: flex;\n  flex-direction: column;\n}\n\
[data-blocks-auth-oauth-consent-scopes] [data-scope=\"list\"][data-part=\"item\"] {\n  display: flex;\n  gap: var(--fandhe-space-3);\n  padding-block: var(--fandhe-space-3);\n}\n\
[data-blocks-auth-oauth-consent-scopes] [data-scope=\"list\"][data-part=\"item\"] + [data-scope=\"list\"][data-part=\"item\"] {\n  border-top: 1px solid var(--fandhe-color-border);\n}\n\
.blocks-auth-oauth-consent-scope-body {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1);\n  min-width: 0;\n}\n\
.blocks-auth-oauth-consent-scope-desc {\n  color: var(--fandhe-color-fg-muted);\n}\n\
.blocks-auth-oauth-consent-account {\n  display: flex;\n  flex-wrap: wrap;\n  align-items: center;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-auth-oauth-consent-email {\n  display: block;\n  color: var(--fandhe-color-fg-muted);\n}\n\
[data-scope=\"card\"][data-part=\"footer\"][data-blocks-auth-oauth-consent-actions] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-auth-oauth-consent-help {\n  text-align: center;\n  color: var(--fandhe-color-fg-muted);\n}\n\
@media (min-width: 40rem) {\n  [data-scope=\"card\"][data-part=\"footer\"][data-blocks-auth-oauth-consent-actions] {\n    flex-direction: row;\n    justify-content: flex-end;\n  }\n}\n";

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
            "data-scope=\"card\"",
            "data-scope=\"avatar\"",
            "data-scope=\"icon\"",
            "data-scope=\"list\"",
            "data-scope=\"separator\"",
            "data-scope=\"button\"",
            "data-scope=\"link\"",
            "data-scope=\"heading\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
    }

    #[test]
    fn no_form_semantics_or_dead_links() {
        let html = demo_html();
        assert!(!html.contains("<form"));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("src=\"data:"));
        assert!(!html.contains("mailto:"));
    }

    #[test]
    fn buttons_are_type_button_and_enabled() {
        let html = demo_html();
        let count_open = html.matches("<button").count();
        let count_typed = html.matches("type=\"button\"").count();
        assert!(count_open > 0);
        assert!(count_typed >= count_open, "html={html}");
        assert!(!html.contains("disabled"));
    }

    #[test]
    fn layout_css_stacks_actions_on_narrow_width() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("@media (min-width: 40rem)"));
        assert!(LAYOUT_CSS.contains("flex-direction: column;"));
        assert!(LAYOUT_CSS.contains("flex-direction: row;"));
    }

    #[test]
    fn ids_have_no_duplicates() {
        let html = demo_html();
        let mut ids: Vec<&str> = Vec::new();
        for chunk in html.split("id=\"").skip(1) {
            if let Some(end) = chunk.find('"') {
                ids.push(&chunk[..end]);
            }
        }
        let mut sorted = ids.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(ids.len(), sorted.len(), "id が重複している: {ids:?}");
    }

    #[test]
    fn no_real_authorization_parameters() {
        let html = demo_html();
        assert!(!html.contains("redirect_uri"));
        assert!(!html.contains("client_id"));
        assert!(!html.contains("access_token"));
    }

    #[test]
    fn demo_is_deterministic() {
        assert_eq!(demo_html(), demo_html());
    }

    #[test]
    fn uses_shared_dummy_assets() {
        let html = demo_html();
        assert!(html.contains(super::dummy_assets::AVATAR_SRC));
        assert!(html.contains(super::dummy_assets::LOGO_SRC));
    }
}
