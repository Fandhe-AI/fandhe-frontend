//! `card-heading-basic` block（イシュー #3229。Application / Card Heading
//! カテゴリ 2 件目の block、1 件目は `card_heading_toolbar.rs`〔イシュー
//! #2902〕）。「カード上部の最小構成区画見出し」の 6 形を 1 block・
//! 6 インスタンス縦積みで併記する合成例。主参照は対応表 ID R0808（基準形）、
//! 集約元は R0809〜R0813（出典の固有名・ファイル名は記載しない）。
//!
//! # 使用部品
//!
//! `card` / `heading` / `text` / `avatar` / `button` / `menu` / `link` の
//! 7 部品のみを合成する（[`BLOCK`] の `parts` に一致させる契約）。新しい
//! UI 部品は追加しない。
//!
//! # 6 形の対応（`data-blocks-card-heading-basic-variant`）
//!
//! - **title**（R0808・主参照）: 題名のみ。
//! - **title-actions**: 題名 + 操作ボタン 1 個。
//! - **avatar-outline**: アバター + 見出し群、右に輪郭（outline）ボタン
//!   2 個。
//! - **title-description-actions**: 題名 + 説明文 + 操作ボタン + テキスト
//!   リンク。
//! - **title-description**: 題名 + 説明文のみ（操作なし）。
//! - **avatar-meta-menu**: アバター + メタ情報（投稿者名・日時） + 三点
//!   メニュー。
//!
//! # 下罫線の有無は形により異なる
//!
//! `data-blocks-card-heading-basic-divider` 属性の有無で表現する。
//! title・title-actions・title-description-actions の 3 形に付与し、
//! avatar-outline・title-description・avatar-meta-menu の 3 形は付与しない
//! （Issue 要件「下罫線の有無は形により異なる」の最小表現）。
//!
//! # 三点メニューは無 JS のため閉じた状態で固定する
//!
//! 開閉状態機械・実際のポップアップ表示はクライアント配線層（wasm-full）の
//! 責務であり、無 JS の docs サイトでは `OpenState::Closed` の静的表示のみを
//! 描画する（`card_heading_toolbar.rs::overflow_menu` と同型）。本 block は
//! メニューを 1 個しか持たないため id 重複の懸念はない。
//!
//! # リンクは `href="#"` を使わない
//!
//! `page_heading_avatar.rs` と同型の判断で、テキストリンクの遷移先は実在の
//! 公開リポジトリ URL（`REPO`）を使い、可視テキストを遷移先がわかる
//! 「GitHub で見る」にする（表示文言と遷移先の意味不一致を避ける）。
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
//! 持たない（`docs/policy/intentional-non-adoption.md` §3.25）。文言はすべて
//! 独自の架空の日本語ダミー（`dummy_assets::PERSON_NAMES` 等。実企業名・
//! 実クレデンシャル・PII を含まない）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::menu::{self, OpenState};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};
use fandhe_frontend_pre_styled_ui::Size;

/// テキストリンクの遷移先（外部の実在 URL、`href="#"` は使わない。
/// モジュール doc「リンクは `href="#"` を使わない」節参照）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// 見出し（H3）。docs ページ自体が H1 を持つため block 内は H2 以下とする
/// （`card_heading_toolbar.rs` と同型の判断）。
fn title(label: &str) -> Node {
    heading::heading(
        HeadingLevel::H3,
        &HeadingProps {
            size: HeadingSize::Md,
            ..HeadingProps::default()
        },
        vec![],
        vec![text(label)],
    )
}

/// 説明文・メタ情報用の淡色テキスト。
fn muted(label: &str) -> Node {
    styled_text::text(
        &TextProps {
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![],
        vec![text(label)],
    )
}

/// アバター（円形）。氏名をアクセシブルネームとして `root` へ直接付与し、
/// フォールバックは氏名の先頭 1 文字を表示する（`page_heading_avatar.rs::
/// profile_avatar` と同型）。
fn person_avatar(name: &str) -> Node {
    let initial: String = name.chars().take(1).collect();
    avatar::root(
        &AvatarProps {
            size: Size::Md,
            ..AvatarProps::default()
        },
        vec![("role", "img"), ("aria-label", name)],
        vec![
            avatar::image(ImageStatus::Loaded, dummy_assets::AVATAR_SRC, "", vec![]),
            avatar::fallback(ImageStatus::Loaded, vec![], vec![text(initial)]),
        ],
    )
}

/// 三点メニュー（無 JS のため閉じた状態で固定する。モジュール doc
/// 「三点メニューは無 JS のため閉じた状態で固定する」節参照）。
fn overflow_menu() -> Node {
    const CONTENT_ID: &str = "blocks-card-heading-basic-menu";
    let trigger = menu::trigger(
        OpenState::Closed,
        false,
        Some(CONTENT_ID),
        vec![("aria-label", "その他の操作")],
        vec![text("\u{2026}")],
    );
    let content = menu::content(
        OpenState::Closed,
        Some(CONTENT_ID),
        None,
        vec![],
        vec![
            menu::item("edit", false, false, vec![], vec![text("編集する")]),
            menu::item(
                "archive",
                false,
                false,
                vec![],
                vec![text("アーカイブする")],
            ),
            menu::separator(vec![], vec![]),
            menu::item("delete", false, false, vec![], vec![text("削除する")]),
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

/// カード本文（点線枠の短いプレースホルダ。見出しが「カード上部」で
/// あることを示すためのダミー本文）。
fn placeholder_body() -> Node {
    card::body(
        vec![],
        vec![div(
            vec![("data-blocks-card-heading-basic-placeholder", "")],
            vec![muted("ここにカードの本文が入ります。")],
        )],
    )
}

/// カード 1 枚（見出し行 + プレースホルダ本文）を組み立てる。`divider` は
/// 下罫線の有無（モジュール doc「下罫線の有無は形により異なる」節参照）。
fn panel(variant: &'static str, divider: bool, header_content: Node) -> Node {
    let mut header_attrs: Vec<(&str, &str)> = vec![("data-blocks-card-heading-basic-header", "")];
    if divider {
        header_attrs.push(("data-blocks-card-heading-basic-divider", ""));
    }
    card::root(
        CardProps::default(),
        vec![("data-blocks-card-heading-basic-variant", variant)],
        vec![
            card::header(header_attrs, vec![header_content]),
            placeholder_body(),
        ],
    )
}

/// 1. 題名のみ（R0808・主参照）。
fn variant_title() -> Node {
    panel("title", true, title("プロジェクト概要"))
}

/// 2. 題名 + 操作ボタン。
fn variant_title_actions() -> Node {
    let header = div(
        vec![("data-blocks-card-heading-basic-header-row", "")],
        vec![
            title("チームメンバー"),
            button::button(&ButtonProps::default(), vec![], vec![text("招待する")]),
        ],
    );
    panel("title-actions", true, header)
}

/// 3. アバター + 見出し群、右に輪郭ボタン 2 個。
fn variant_avatar_outline() -> Node {
    let name = dummy_assets::PERSON_NAMES[0];
    let group = div(
        vec![("data-blocks-card-heading-basic-group", "")],
        vec![
            person_avatar(name),
            div(
                vec![("data-blocks-card-heading-basic-stack", "")],
                vec![title(name), muted("プラットフォームチーム")],
            ),
        ],
    );
    let actions = div(
        vec![("data-blocks-card-heading-basic-actions", "")],
        vec![
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Outline,
                    ..ButtonProps::default()
                },
                vec![],
                vec![text("メッセージ")],
            ),
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Outline,
                    ..ButtonProps::default()
                },
                vec![],
                vec![text("プロフィール")],
            ),
        ],
    );
    let header = div(
        vec![("data-blocks-card-heading-basic-header-row", "")],
        vec![group, actions],
    );
    panel("avatar-outline", false, header)
}

/// 4. 題名 + 説明文 + 操作ボタン + テキストリンク。
fn variant_title_description_actions() -> Node {
    let stack = div(
        vec![("data-blocks-card-heading-basic-stack", "")],
        vec![
            title("リリースノート"),
            muted("直近の更新内容と既知の不具合をまとめています。"),
        ],
    );
    let actions = div(
        vec![("data-blocks-card-heading-basic-actions", "")],
        vec![
            button::button(&ButtonProps::default(), vec![], vec![text("公開する")]),
            link::root(
                REPO,
                &LinkProps {
                    external: true,
                    ..LinkProps::default()
                },
                vec![],
                vec![text("GitHub で見る")],
            ),
        ],
    );
    let header = div(
        vec![("data-blocks-card-heading-basic-header-row", "")],
        vec![stack, actions],
    );
    panel("title-description-actions", true, header)
}

/// 5. 題名 + 説明文のみ（操作なし）。
fn variant_title_description() -> Node {
    let stack = div(
        vec![("data-blocks-card-heading-basic-stack", "")],
        vec![
            title("利用状況"),
            muted("今月のアクセス数と利用傾向の概要です。"),
        ],
    );
    panel("title-description", false, stack)
}

/// 6. アバター + メタ情報（投稿者名・日時） + 三点メニュー。
fn variant_avatar_meta_menu() -> Node {
    let name = dummy_assets::PERSON_NAMES[1];
    let group = div(
        vec![("data-blocks-card-heading-basic-group", "")],
        vec![
            person_avatar(name),
            div(
                vec![("data-blocks-card-heading-basic-stack", "")],
                vec![title(name), muted("9 月 26 日に投稿")],
            ),
        ],
    );
    let header = div(
        vec![("data-blocks-card-heading-basic-header-row", "")],
        vec![group, overflow_menu()],
    );
    panel("avatar-meta-menu", false, header)
}

/// `card-heading-basic` の Demo 本体（6 形を縦積みで併記する。呼び出し
/// ごとに同一の `Node` を返す純関数）。
#[must_use]
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-card-heading-basic-layout")],
        vec![
            variant_title(),
            variant_title_actions(),
            variant_avatar_outline(),
            variant_title_description_actions(),
            variant_title_description(),
            variant_avatar_meta_menu(),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/card-heading-basic/",
    title: "card-heading-basic",
    category: BlockCategory::CardHeading,
    rust_source: "crates/docs-site/src/blocks/application/card_heading/card_heading_basic.rs",
    demo_class: "blocks-card-heading-basic",
    parts: &[
        Part {
            label: "Card",
            path: "/themes/card/",
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
            label: "Avatar",
            path: "/themes/avatar/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Menu",
            path: "/themes/menu/",
        },
        Part {
            label: "Link",
            path: "/themes/link/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `card_heading_basic` 固有のレイアウト規則（`crate::blocks`
/// モジュール doc「CSS の置き場」節と同型）。
///
/// セレクタは `.blocks-card-heading-basic-*` と
/// `[data-blocks-card-heading-basic-*]` のみを用いる。
///
/// # ルート class を `demo_class` と別名にする理由
///
/// [`Block::demo_class`] は `blocks-card-heading-basic` だが、`demo()`
/// が返すルート `div` の class は `blocks-card-heading-basic-layout`
/// という別名にする（`card_heading_toolbar` 等と同じ Bugbot 教訓の回避）。
const LAYOUT_CSS: &str = "\
.blocks-card-heading-basic-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n}\n\
[data-blocks-card-heading-basic-header-row] {\n  display: flex;\n  flex-wrap: wrap;\n  align-items: center;\n  justify-content: space-between;\n  gap: var(--fandhe-space-3);\n}\n\
[data-blocks-card-heading-basic-group] {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-3);\n  min-width: 0;\n}\n\
[data-blocks-card-heading-basic-stack] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1);\n  min-width: 0;\n}\n\
[data-blocks-card-heading-basic-actions] {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n  flex-shrink: 0;\n}\n\
[data-blocks-card-heading-basic-divider] {\n  border-bottom: 1px solid var(--fandhe-color-border);\n}\n\
[data-blocks-card-heading-basic-placeholder] {\n  min-height: 4rem;\n  display: flex;\n  align-items: center;\n  border: 1px dashed var(--fandhe-color-border);\n  border-radius: var(--fandhe-radius-md);\n  padding: var(--fandhe-space-4);\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    fn demo_html() -> String {
        render(&demo())
    }

    /// Demo が使用部品（card/heading/text/avatar/button/menu/link）の
    /// anatomy をすべて実際に出力していること。
    #[test]
    fn demo_composes_expected_parts() {
        let html = demo_html();
        for scope in [
            "data-scope=\"card\"",
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"avatar\"",
            "data-scope=\"button\"",
            "data-scope=\"menu\"",
            "data-scope=\"link\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
    }

    /// カードはちょうど 6 枚（6 形）。
    #[test]
    fn demo_card_count() {
        let html = demo_html();
        assert_eq!(
            html.matches("data-scope=\"card\" data-part=\"root\"")
                .count(),
            6
        );
        for variant in [
            "title",
            "title-actions",
            "avatar-outline",
            "title-description-actions",
            "title-description",
            "avatar-meta-menu",
        ] {
            assert!(html.contains(&format!(
                "data-blocks-card-heading-basic-variant=\"{variant}\""
            )));
        }
    }

    /// 下罫線ありはちょうど 3 形（title/title-actions/
    /// title-description-actions）。
    #[test]
    fn divider_is_on_three_variants() {
        let html = demo_html();
        assert_eq!(
            html.matches("data-blocks-card-heading-basic-divider")
                .count(),
            3
        );
    }

    /// menu は閉じた状態（`aria-expanded="false"`）で固定し、trigger の
    /// `aria-controls` が content の id と一致する。
    #[test]
    fn menu_is_static_closed() {
        let html = demo_html();
        assert!(html.contains(r#"aria-expanded="false""#));
        assert!(html.contains(r#"aria-controls="blocks-card-heading-basic-menu""#));
        assert!(html.contains(r#"id="blocks-card-heading-basic-menu""#));
    }

    /// `<form>` を出力しない・`href="#"` を使わない・XSS 回帰の不変条件を
    /// 固定する（`crate::blocks` モジュール doc）。
    #[test]
    fn demo_has_no_form_or_unsafe_output() {
        let html = demo_html();
        for absent in ["<form", "type=\"submit\"", "href=\"#\"", "src=\"data:"] {
            assert!(!html.contains(absent), "demo should never contain {absent}");
        }
    }

    /// ボタンはすべて `type="button"`（暗黙 submit が起きないことの
    /// 回帰ガード）。
    #[test]
    fn all_buttons_are_type_button() {
        let html = demo_html();
        let count_open = html.matches("<button").count();
        let count_typed = html.matches("type=\"button\"").count();
        assert!(count_open > 0);
        assert!(count_typed >= count_open, "html={html}");
    }

    /// リンクの遷移先が実在の公開リポジトリ URL であり、可視テキストが
    /// 遷移先を過大に主張しないこと。
    #[test]
    fn link_destination_matches_label() {
        let html = demo_html();
        assert!(html.contains("href=\"https://github.com/Fandhe-AI/fandhe-frontend\""));
        assert!(html.contains("GitHub で見る"));
    }

    /// [`LAYOUT_CSS`] が `<` を含まない（REQ-1: `</style>` によるスタイル
    /// 脱出防止）。
    #[test]
    fn layout_css_has_no_angle_bracket() {
        assert!(!LAYOUT_CSS.contains('<'));
    }

    /// ルート class（`demo_class` とは別名）が `demo()` の出力へ実際に
    /// 現れること。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = demo_html();
        assert!(html.contains("class=\"blocks-card-heading-basic-layout\""));
        assert_ne!(super::BLOCK.demo_class, "blocks-card-heading-basic-layout");
    }

    /// 呼び出しごとに同一の `Node` を返す純関数であること。
    #[test]
    fn demo_is_deterministic() {
        assert_eq!(demo_html(), demo_html());
    }

    /// id はすべて一意（`aria-controls`/`aria-labelledby` の宙ぶらり参照
    /// 回避、`blocks_contract.rs` と同型のガード）。
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
}
