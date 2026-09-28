//! `page-heading-avatar` block（イシュー #2931。親トラッキング #2892
//! 「Blocks アプリケーション A」配下、phase:3）。「左に円形アバター
//! （または企業ロゴ）、右に名前と補足行、右端に操作ボタン列」を持つ
//! ページ見出しの合成例。Application / Page Heading カテゴリ 2 件目の
//! block（1 件目は `page_heading_actions.rs`、イシュー #2930）。
//!
//! # 使用部品
//!
//! `avatar` / `image` / `heading` / `text` / `link` / `button` / `menu` の
//! 7 部品のみを合成する（[`BLOCK`] の `parts` に一致させる契約）。新しい
//! UI 部品は追加しない。
//!
//! # 3 インスタンスで派生形を表現する（無 JS のため静的併記）
//!
//! 対応表の主参照 R1130（代表構成）を軸に、集約元 R0594（アバター +
//! メール + 操作列）・R1134（企業ロゴ + 請求書番号）を次の 3 インスタンス
//! として併記する。`_/blocks-intake/` の対応ファイルは本イシュー着手時点で
//! 本 worktree に存在しないため、対応表 ID のみを記す
//! （`list-title-meta`〔イシュー #2925〕・`card-form-footer`〔イシュー
//! #2899〕と同じ扱い）。
//!
//! - **profile**（R1130・主参照）: 円形アバター + 氏名 + 役職・所属チーム
//!   （中黒区切り） + 「メッセージ」「プロフィールを編集」ボタン + 三点
//!   メニュー。
//! - **invite**（R0594）: 同じアバター骨格 + メールアドレス（プレーン
//!   テキスト） + 応募内容へのリンク + 応募日 + 「却下」「面接に進める」
//!   ボタン + 三点メニュー。
//! - **invoice**（R1134）: media を企業ロゴ画像へ差し替え + 社名 + 請求書
//!   番号 + 発行日 + 「PDF をダウンロード」「支払いを記録」ボタン + 三点
//!   メニュー。
//!
//! # 狭幅ではボタン列を三点メニューへ集約する
//!
//! `panel` は名前付き `@container`（`blocks-page-heading-avatar`）を宣言し、
//! `data-blocks-page-heading-avatar-action` を持つボタンは `40rem` 未満で
//! 非表示にする（`list_title_meta.rs` の `@container` 方式と同型）。三点
//! メニューの `content` には常に同じ操作項目を含めるため、狭幅でも到達
//! 手段は失われない。
//!
//! # メールをリンク化しない理由
//!
//! `contact_info_columns.rs` は `mailto:` リンクの前例を持つが、本 block は
//! 補足行をプレーンテキストのまま扱う（イシュー本文「補足行のリンク・
//! 日時は本文テキストの一部として扱う」要件を、他 block に前例のない
//! `mailto:` リンク化ではなく最小の解釈で満たす）。
//!
//! # 三点メニューは無 JS のため閉じた状態で固定する
//!
//! 開閉状態機械・実際のポップアップ表示はクライアント配線層（wasm-full）の
//! 責務であり、無 JS の docs サイトでは `OpenState::Closed` の静的表示のみを
//! 描画する（`card_heading_toolbar.rs::overflow_menu` と同型）。3 インス
//! タンスの `content` id はインスタンスごとに一意にする（id 重複・宙ぶらり
//! `aria-controls`/`aria-labelledby` 参照の回避、`blocks_contract.rs` 参照）。
//! ボタン・メニュートリガーはいずれも enabled のまま用いる（`disabled` に
//! 固定しない）。同じ heading 家族の `card_heading_toolbar.rs` と同型の
//! 判断であり、無 JS のため実際には開閉・遷移は起きない旨を
//! `site/blocks/page-heading-avatar.md` に明記する。
//!
//! # `<form>` を使わない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない。ボタンは `button::button` の既定 `type="button"` のまま
//! 用い、送信処理・送信先は一切持たない。
//!
//! # ダミー素材について
//!
//! アバター画像・ロゴ画像は `crate::blocks::dummy_assets::AVATAR_SRC`/
//! `LOGO_SRC`（モノトーン抽象図形の SVG、`build.rs` がビルド時に書き出す）
//! を使う。氏名・役職は `dummy_assets::PERSON_NAMES`/`JOB_TITLES`、社名は
//! `dummy_assets::COMPANY_NAMES` の架空セットを使う。メールアドレス・
//! 請求書番号・日付はすべて独自に書いた架空の文言であり、実在の人物・
//! 企業・クレデンシャルとは無関係。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, span, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::image::{self, ImageFit, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::menu::{self, OpenState};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};
use fandhe_frontend_pre_styled_ui::Size;

/// 外部の実在 URL（`href="#"` は使わない、他 block と同型の判断）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// アバター（円形）を組み立てる。氏名をアクセシブルネームとして `root` へ
/// 直接付与し、フォールバックは氏名の先頭 1 文字を表示する
/// （`list_title_meta.rs::initial_avatar` と同型。`role="img"` は role なし
/// `<div>` が `aria-label` を name computation の対象にしないため必須）。
fn profile_avatar(name: &str) -> Node {
    let initial: String = name.chars().take(1).collect();
    avatar::root(
        &AvatarProps {
            size: Size::Xl,
            ..AvatarProps::default()
        },
        vec![("role", "img"), ("aria-label", name)],
        vec![
            avatar::image(ImageStatus::Loaded, dummy_assets::AVATAR_SRC, "", vec![]),
            avatar::fallback(ImageStatus::Loaded, vec![], vec![text(initial)]),
        ],
    )
}

/// 企業ロゴ（invoice インスタンスの media）。`image` 部品（avatar とは別の
/// 単純画像部品）を使い、固定 `4rem` 角の枠を `data-blocks-page-heading-
/// avatar-logo` へ CSS で与える。
fn logo_image(company: &str) -> Node {
    image::image(
        &ImageProps {
            shape: ImageShape::Rounded,
            fit: ImageFit::Contain,
            ..ImageProps::new(dummy_assets::LOGO_SRC, &format!("{company} のロゴ"))
        },
        vec![("data-blocks-page-heading-avatar-logo", "")],
    )
}

/// 中黒区切り（装飾。支援技術には前後の語がそのまま連続して読み上げられる
/// ため、[`visually_hidden`] は使用部品に含めない、モジュール doc「使用
/// 部品」参照）。
fn dot_separator() -> Node {
    span(vec![], vec![text(" \u{b7} ")])
}

/// 見出し（H2、[`Heading`](heading) 部品）。docs ページ自体が H1 を持つため
/// block 内は H2 以下とする（`card_heading_toolbar.rs` と同型の判断）。
fn name_heading(name: &str) -> Node {
    heading::heading(
        HeadingLevel::H2,
        &HeadingProps {
            size: HeadingSize::Lg,
            ..HeadingProps::default()
        },
        vec![],
        vec![text(name)],
    )
}

/// メタ行の外枠（`<p>` 相当、[`Text`](styled_text) 部品）。
fn meta_line(children: Vec<Node>) -> Node {
    styled_text::text(
        &TextProps {
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![("class", "blocks-page-heading-avatar-meta")],
        children,
    )
}

/// `<time datetime>` 要素。
fn time_el(iso: &'static str, label: &'static str) -> Node {
    el("time", vec![("datetime", iso)], vec![text(label)])
}

/// 三点メニュー（[`Menu`](menu) 部品）。`items` は `(value, label)` の組。
/// 無 JS のため `OpenState::Closed` 固定・disabled にはしない
/// （モジュール doc「三点メニューは無 JS のため閉じた状態で固定する」
/// 節参照）。
fn overflow_menu(
    heading_name: &str,
    content_id: &'static str,
    trigger_id: &'static str,
    items: &[(&'static str, &'static str)],
) -> Node {
    let mut menu_items: Vec<Node> = Vec::new();
    for (index, (value, label)) in items.iter().enumerate() {
        if index > 0 {
            menu_items.push(menu::separator(vec![], vec![]));
        }
        menu_items.push(menu::item(value, false, false, vec![], vec![text(*label)]));
    }
    let trigger = menu::trigger(
        OpenState::Closed,
        false,
        Some(content_id),
        vec![
            ("id", trigger_id),
            ("aria-label", &format!("その他の操作、{heading_name}")),
        ],
        vec![text("\u{2026}")],
    );
    let content = menu::content(
        OpenState::Closed,
        Some(content_id),
        Some(trigger_id),
        vec![],
        menu_items,
    );
    let positioner = menu::positioner(OpenState::Closed, vec![], vec![content]);
    menu::root(
        Size::Sm,
        OpenState::Closed,
        vec![],
        vec![trigger, positioner],
    )
}

/// 操作ボタン + 三点メニューをまとめた actions 列。`primary`/`secondary` の
/// 2 ボタンには `data-blocks-page-heading-avatar-action` を付与し、
/// `40rem` 未満では非表示にする（[`LAYOUT_CSS`] 参照）。
fn actions(secondary_label: &str, primary_label: &str, menu_node: Node) -> Node {
    let secondary = button::button(
        &ButtonProps {
            variant: ButtonVariant::Outline,
            size: Size::Sm,
            ..ButtonProps::default()
        },
        vec![("data-blocks-page-heading-avatar-action", "")],
        vec![text(secondary_label)],
    );
    let primary = button::button(
        &ButtonProps {
            variant: ButtonVariant::Solid,
            size: Size::Sm,
            ..ButtonProps::default()
        },
        vec![("data-blocks-page-heading-avatar-action", "")],
        vec![text(primary_label)],
    );
    div(
        vec![("class", "blocks-page-heading-avatar-actions")],
        vec![secondary, primary, menu_node],
    )
}

/// header（media | body | actions）の共通骨格。
fn header(media: Node, name_node: Node, meta: Node, actions_node: Node) -> Node {
    div(
        vec![("class", "blocks-page-heading-avatar-header")],
        vec![
            media,
            div(
                vec![("class", "blocks-page-heading-avatar-body")],
                vec![name_node, meta],
            ),
            actions_node,
        ],
    )
}

/// パネル外枠（`@container` の名前付きコンテナ）。
fn panel(variant: &'static str, content: Node) -> Node {
    div(
        vec![
            ("class", "blocks-page-heading-avatar-panel"),
            ("data-blocks-page-heading-avatar-variant", variant),
        ],
        vec![content],
    )
}

/// **profile**（R1130・主参照）インスタンス。
fn profile_instance() -> Node {
    let name = dummy_assets::PERSON_NAMES[0];
    let menu_node = overflow_menu(
        name,
        "blocks-page-heading-avatar-menu-profile",
        "blocks-page-heading-avatar-trigger-profile",
        &[
            ("edit-profile", "プロフィールを編集"),
            ("message", "メッセージ"),
            ("suspend", "アカウントを停止"),
        ],
    );
    panel(
        "profile",
        header(
            profile_avatar(name),
            name_heading(name),
            meta_line(vec![
                text(dummy_assets::JOB_TITLES[0]),
                dot_separator(),
                text("プラットフォームチーム"),
            ]),
            actions("メッセージ", "プロフィールを編集", menu_node),
        ),
    )
}

/// **invite**（R0594）インスタンス。
fn invite_instance() -> Node {
    let name = dummy_assets::PERSON_NAMES[1];
    let menu_node = overflow_menu(
        name,
        "blocks-page-heading-avatar-menu-invite",
        "blocks-page-heading-avatar-trigger-invite",
        &[
            ("advance", "面接に進める"),
            ("reject", "却下"),
            ("note", "メモを追加"),
        ],
    );
    panel(
        "invite",
        header(
            profile_avatar(name),
            name_heading(name),
            meta_line(vec![
                text("elena.vasquez@example.com"),
                dot_separator(),
                link::root(
                    REPO,
                    &LinkProps::default(),
                    vec![],
                    vec![text("応募内容を見る")],
                ),
                dot_separator(),
                time_el("2026-09-26", "9 月 26 日に応募"),
            ]),
            actions("却下", "面接に進める", menu_node),
        ),
    )
}

/// **invoice**（R1134）インスタンス。
fn invoice_instance() -> Node {
    let company = dummy_assets::COMPANY_NAMES[0];
    let menu_node = overflow_menu(
        company,
        "blocks-page-heading-avatar-menu-invoice",
        "blocks-page-heading-avatar-trigger-invoice",
        &[
            ("record-payment", "支払いを記録"),
            ("download", "PDF をダウンロード"),
            ("void", "請求書を無効化"),
        ],
    );
    panel(
        "invoice",
        header(
            logo_image(company),
            name_heading(company),
            meta_line(vec![
                text("請求書番号 INV-0000123"),
                dot_separator(),
                time_el("2026-09-30", "2026 年 9 月 30 日 発行"),
            ]),
            actions("PDF をダウンロード", "支払いを記録", menu_node),
        ),
    )
}

/// `page-heading-avatar` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数。主参照（R1130・profile）を先頭に、3 インスタンスを縦に並べる。
#[must_use]
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-page-heading-avatar-layout")],
        vec![profile_instance(), invite_instance(), invoice_instance()],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/page-heading-avatar/",
    title: "page-heading-avatar",
    category: BlockCategory::PageHeading,
    rust_source: "crates/docs-site/src/blocks/application/page_heading/page_heading_avatar.rs",
    demo_class: "blocks-page-heading-avatar",
    parts: &[
        Part {
            label: "Avatar",
            path: "/themes/avatar/",
        },
        Part {
            label: "Image",
            path: "/themes/image/",
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
            label: "Link",
            path: "/themes/link/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Menu",
            path: "/themes/menu/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `page_heading_avatar` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節と同型）。狭幅（`40rem` 未満）では
/// `data-blocks-page-heading-avatar-action` を持つボタンを隠し、三点
/// メニューへ操作を集約する（[`overflow_menu`] 参照）。
const LAYOUT_CSS: &str = "\
.blocks-page-heading-avatar-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n}\n\
.blocks-page-heading-avatar-panel {\n  container-type: inline-size;\n  container-name: blocks-page-heading-avatar;\n}\n\
.blocks-page-heading-avatar-header {\n  display: grid;\n  grid-template-columns: auto minmax(0, 1fr) auto;\n  align-items: center;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-page-heading-avatar-body {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1);\n  min-width: 0;\n}\n\
.blocks-page-heading-avatar-meta {\n  display: flex;\n  flex-wrap: wrap;\n  align-items: center;\n  gap: var(--fandhe-space-1);\n}\n\
.blocks-page-heading-avatar-actions {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n  flex-shrink: 0;\n}\n\
[data-blocks-page-heading-avatar-logo] {\n  width: 4rem;\n  height: 4rem;\n}\n\
.blocks-page-heading-avatar-panel [data-blocks-page-heading-avatar-action] {\n  display: none;\n}\n\
@container blocks-page-heading-avatar (min-width: 40rem) {\n  \
.blocks-page-heading-avatar-panel [data-blocks-page-heading-avatar-action] {\n    display: inline-flex;\n  }\n\
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
            "data-scope=\"image\"",
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"link\"",
            "data-scope=\"button\"",
            "data-scope=\"menu\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
    }

    #[test]
    fn demo_wires_all_three_variant_hooks() {
        let html = demo_html();
        for variant in ["profile", "invite", "invoice"] {
            assert!(html.contains(&format!(
                "data-blocks-page-heading-avatar-variant=\"{variant}\""
            )));
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
    fn menu_ids_have_no_duplicates() {
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
    fn all_buttons_are_type_button() {
        let html = demo_html();
        let count_open = html.matches("<button").count();
        let count_typed = html.matches("type=\"button\"").count();
        assert!(count_open > 0);
        assert!(count_typed >= count_open, "html={html}");
    }

    #[test]
    fn has_time_element_with_datetime() {
        let html = demo_html();
        assert!(html.contains("<time datetime="));
    }

    #[test]
    fn layout_css_is_safe_and_declares_container_query() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("container-type: inline-size;"));
        assert!(LAYOUT_CSS.contains("@container blocks-page-heading-avatar (min-width: 40rem)"));
    }

    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = demo_html();
        assert!(html.contains("class=\"blocks-page-heading-avatar-layout\""));
        assert_ne!(super::BLOCK.demo_class, "blocks-page-heading-avatar-layout");
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
