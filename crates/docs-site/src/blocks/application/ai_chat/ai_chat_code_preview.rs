//! `ai-chat-code-preview` block（イシュー #2958。親 #2957「Application /
//! AI Chat」区分の 1 件目、Application / AI Chat カテゴリの卒業を伴う）。
//! 「上部にナビ、左列にチャット、右列にプレビュー/コード切替タブ」を持つ
//! コード生成チャット画面の合成例。
//!
//! # 使用部品
//!
//! `message` / `message-scroller` / `textarea` / `tabs` / `button` /
//! `menu` / `code` の 7 部品のみを合成する（[`BLOCK`] の `parts` に一致
//! させる契約）。新しい UI 部品は追加しない。
//!
//! # 状態は 1 インスタンスに固定する（状態違いの併記は #2959）
//!
//! 無 JS のため実際の送信・生成・タブ切替は起きない。右列の tabs は
//! `preview` 選択で固定し、`code` パネルは headless tabs の `hidden` 属性で
//! 非表示になる（初期状態固定の方針どおり）。code 側を可視で示す「code
//! 選択の状態違い併記」は後半 #2959 のスコープ。
//!
//! # 狭幅では「見出し表示 + 縦積み」で表現し、要素を隠さない
//!
//! `@container` で [`LAYOUT_CSS`] が body を 1 列へ縮退させるだけで、両
//! パネル（chat/preview）はどちらも常に到達可能なまま DOM に残る。狭幅
//! 専用の [`switch_group`] は表示の目安を示す `role="group"` の
//! `aria-pressed` トグル 2 個であり、実際のパネル切替（`display: none`）
//! は行わない（`page_heading_avatar.rs` の codex レビュー指摘「狭幅で
//! 操作列を非表示にすると無 JS では到達不能になる」と同型の判断。`tabs`
//! 部品を切替 UI に使わない理由: tabs は content パネルを伴うため、広幅で
//! 2 列に並ぶ実パネルを tabpanel 内に置けず空 tabpanel を出すことになる）。
//! 広幅では [`LAYOUT_CSS`] が `-switch` を `display: none` で隠す（隠すのは
//! 切替 UI 自体のみで、内容パネルではない）。
//!
//! # `drop_class_attr` 対策
//!
//! `message::*` / `message_scroller::*` / `menu::root` / `button::button` /
//! `textarea::textarea` / `code::code` は呼び出し側 `class` を除去するため、
//! CSS フックは `data-blocks-ai-chat-code-preview-*` 属性で渡す
//! （`page_heading_avatar.rs`「メタ行を素の `<p>` で組み立てる理由」節と
//! 同型の制約）。素の `div`/`section`/`pre` には `class` を使う。
//!
//! # id の一意性
//!
//! menu（trigger/content）・tabs（`{id}-trigger-*`/`{id}-content-*`）・
//! textarea（[`FieldProps::id`]）を block 固有プレフィックスで付与する。
//! `blocks_contract.rs::demo_output_has_no_dangling_aria_references_or_duplicate_ids`
//! が全 block を対象に検証する。
//!
//! # ダミー素材
//!
//! ロゴは `dummy_assets::LOGO_SRC`、人名は `dummy_assets::PERSON_NAMES`
//! を使う。プロジェクト名・会話文・コード片は独自の架空の文言であり、
//! 実企業名・実クレデンシャル・PII を含まない。
//!
//! # 参照素材について
//!
//! 対応表の主参照 R0001（集約元 1 件）を軸にするが、`_/blocks-intake/`
//! の対応ファイルは本イシュー着手時点で本 worktree に存在しないため、
//! 対応表 ID のみを記す（`page_heading_avatar.rs` モジュール doc と同じ
//! 扱い）。状態違いの併記・原稿の差分メモの仕上げは #2959。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, span, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::code::{self, CodeProps};
use fandhe_frontend_pre_styled_ui::menu::{self, OpenState};
use fandhe_frontend_pre_styled_ui::message::{self, MessageAlign, MessageRole, MessageRootProps};
use fandhe_frontend_pre_styled_ui::message_scroller::{
    self, MessageScrollerRootProps, MessageScrollerStuck,
};
use fandhe_frontend_pre_styled_ui::tabs::{
    self, ActivationMode, Orientation, TabItem, TabsProps, TabsVariant,
};
use fandhe_frontend_pre_styled_ui::textarea::{self, FieldIds, FieldProps, TextareaProps};
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// 架空のプロジェクト名（実企業名・実サービス名とは無関係）。
const PROJECT_NAME: &str = "在庫 API ジェネレーター";

/// コードパネルに表示する短い架空の Rust コード片。
const CODE_SNIPPET: &str =
    "async fn inventory() -> Json<Vec<Item>> {\n    Json(fetch_items().await)\n}";

/// 上部ナビ（ロゴ + プロジェクト名 + 操作ボタン + 三点メニュー）を組み立てる。
fn top_nav() -> Node {
    let brand = div(
        vec![("class", "blocks-ai-chat-code-preview-brand")],
        vec![
            el(
                "img",
                vec![("src", dummy_assets::LOGO_SRC), ("alt", "")],
                vec![],
            ),
            span(vec![], vec![text(PROJECT_NAME)]),
        ],
    );

    let share = button::button(
        &ButtonProps {
            variant: ButtonVariant::Outline,
            size: Size::Sm,
            ..ButtonProps::default()
        },
        vec![],
        vec![text("共有")],
    );
    let publish = button::button(
        &ButtonProps {
            variant: ButtonVariant::Solid,
            size: Size::Sm,
            ..ButtonProps::default()
        },
        vec![],
        vec![text("公開")],
    );
    let overflow = overflow_menu();

    let actions = div(
        vec![("class", "blocks-ai-chat-code-preview-nav-actions")],
        vec![share, publish, overflow],
    );

    div(
        vec![("class", "blocks-ai-chat-code-preview-nav")],
        vec![brand, actions],
    )
}

/// 三点メニュー（無 JS のため [`OpenState::Closed`] 固定、
/// `page_heading_avatar.rs::overflow_menu` と同型）。
fn overflow_menu() -> Node {
    const TRIGGER_ID: &str = "blocks-ai-chat-code-preview-menu-trigger";
    const CONTENT_ID: &str = "blocks-ai-chat-code-preview-menu-content";

    let items = [
        ("rename", "プロジェクト名を変更"),
        ("duplicate", "複製"),
        ("archive", "アーカイブ"),
    ];
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
        Some(CONTENT_ID),
        vec![
            ("id", TRIGGER_ID),
            ("aria-label", &format!("その他の操作、{PROJECT_NAME}")),
        ],
        vec![text("\u{2026}")],
    );
    let content = menu::content(
        OpenState::Closed,
        Some(CONTENT_ID),
        Some(TRIGGER_ID),
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

/// 狭幅専用の表示切替（`role="group"`、`aria-pressed` トグル 2 個）。
/// 実際のパネル切替は起きない静的表示（モジュール doc「狭幅では
/// 『見出し表示 + 縦積み』で表現し、要素を隠さない」節参照）。
fn switch_group() -> Node {
    let chat = button::button(
        &ButtonProps {
            variant: ButtonVariant::Solid,
            size: Size::Sm,
            ..ButtonProps::default()
        },
        vec![("aria-pressed", "true")],
        vec![text("チャット")],
    );
    let preview = button::button(
        &ButtonProps {
            variant: ButtonVariant::Outline,
            size: Size::Sm,
            ..ButtonProps::default()
        },
        vec![("aria-pressed", "false")],
        vec![text("プレビュー")],
    );
    div(
        vec![
            ("class", "blocks-ai-chat-code-preview-switch"),
            ("role", "group"),
            ("aria-label", "表示切替"),
        ],
        vec![chat, preview],
    )
}

/// 会話 1 発言を組み立てる。`code_snippet` が `Some` のとき `content` 内へ
/// [`code::code`] のインライン片を差し込む。
fn message_bubble(
    role: MessageRole,
    align: MessageAlign,
    body: &str,
    code_snippet: Option<&str>,
) -> Node {
    let mut content_children = vec![text(body)];
    if let Some(snippet) = code_snippet {
        content_children.push(text(" "));
        content_children.push(code::code(
            &CodeProps::default(),
            vec![],
            vec![text(snippet)],
        ));
    }
    message::root(
        MessageRootProps {
            role,
            align,
            loading: false,
            error: false,
        },
        vec![],
        vec![message::content(vec![], content_children)],
    )
}

/// 左列（チャット履歴 + 入力欄）を組み立てる。
fn chat_pane() -> Node {
    let history = vec![
        message_bubble(
            MessageRole::User,
            MessageAlign::End,
            "在庫一覧を取得する API エンドポイントを Rust で書いて",
            None,
        ),
        message_bubble(
            MessageRole::Assistant,
            MessageAlign::Start,
            "在庫一覧を返すハンドラを用意しました:",
            Some("GET /api/inventory"),
        ),
        message_bubble(
            MessageRole::User,
            MessageAlign::End,
            "レスポンスを JSON にして",
            None,
        ),
    ];

    let scroller = message_scroller::root(
        MessageScrollerRootProps {
            stuck: MessageScrollerStuck::Bottom,
            has_new: false,
        },
        vec![("data-blocks-ai-chat-code-preview-scroller", "")],
        vec![
            message_scroller::viewport(
                "会話履歴",
                vec![],
                vec![message_scroller::content(
                    vec![],
                    vec![message::group("会話", vec![], history)],
                )],
            ),
            message_scroller::anchor(vec![]),
        ],
    );

    let composer = div(
        vec![("class", "blocks-ai-chat-code-preview-composer")],
        vec![
            textarea::textarea(
                &TextareaProps::default(),
                &FieldProps {
                    id: "blocks-ai-chat-code-preview-prompt",
                    ids: FieldIds::default(),
                    disabled: false,
                    invalid: false,
                    required: false,
                    readonly: false,
                    has_helper_text: false,
                },
                false,
                vec![
                    ("placeholder", "メッセージを入力"),
                    ("aria-label", "メッセージを入力"),
                    ("rows", "3"),
                ],
                vec![],
            ),
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Solid,
                    size: Size::Sm,
                    ..ButtonProps::default()
                },
                vec![],
                vec![text("送信")],
            ),
        ],
    );

    el(
        "section",
        vec![("data-blocks-ai-chat-code-preview-pane", "chat")],
        vec![scroller, composer],
    )
}

/// 右列（プレビュー/コード切替タブ）を組み立てる。`preview` 選択で固定
/// （モジュール doc「状態は 1 インスタンスに固定する」節参照）。
fn preview_pane() -> Node {
    let frame = div(
        vec![("class", "blocks-ai-chat-code-preview-frame")],
        vec![span(vec![], vec![text("プレビュー")])],
    );
    let code_block = el(
        "pre",
        vec![],
        vec![code::code(
            &CodeProps::default(),
            vec![],
            vec![text(CODE_SNIPPET)],
        )],
    );

    let tabs_node = tabs::tabs(
        TabsVariant::Line,
        Size::Sm,
        ColorPalette::Accent,
        &TabsProps {
            id: "blocks-ai-chat-code-preview-tabs",
            selected: "preview",
            orientation: Orientation::Horizontal,
            activation_mode: ActivationMode::Automatic,
            loop_focus: true,
            indicator: false,
        },
        vec![
            TabItem {
                value: "preview",
                trigger: vec![text("プレビュー")],
                content: vec![frame],
                disabled: false,
            },
            TabItem {
                value: "code",
                trigger: vec![text("コード")],
                content: vec![code_block],
                disabled: false,
            },
        ],
    );

    el(
        "section",
        vec![("data-blocks-ai-chat-code-preview-pane", "preview")],
        vec![tabs_node],
    )
}

/// `ai-chat-code-preview` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。
#[must_use]
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-ai-chat-code-preview-shell")],
        vec![
            top_nav(),
            switch_group(),
            div(
                vec![("class", "blocks-ai-chat-code-preview-body")],
                vec![chat_pane(), preview_pane()],
            ),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/ai-chat-code-preview/",
    title: "ai-chat-code-preview",
    category: BlockCategory::AiChat,
    rust_source: "crates/docs-site/src/blocks/application/ai_chat/ai_chat_code_preview.rs",
    demo_class: "blocks-ai-chat-code-preview",
    parts: &[
        Part {
            label: "Message",
            path: "/themes/message/",
        },
        Part {
            label: "Message Scroller",
            path: "/themes/message-scroller/",
        },
        Part {
            label: "Textarea",
            path: "/themes/textarea/",
        },
        Part {
            label: "Tabs",
            path: "/themes/tabs/",
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
            label: "Code",
            path: "/themes/code/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `ai_chat_code_preview` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節と同型）。`-switch` の `display: none`
/// のみが唯一の非表示規則であり、chat/preview パネル自体を隠す規則は
/// 持たない（モジュール doc「狭幅では『見出し表示 + 縦積み』で表現し、
/// 要素を隠さない」節参照）。
const LAYOUT_CSS: &str = "\
.blocks-demo.blocks-ai-chat-code-preview {\n  padding: 0;\n}\n\
.blocks-ai-chat-code-preview-shell {\n  container-type: inline-size;\n  container-name: blocks-ai-chat-code-preview;\n  border: 1px solid var(--fandhe-color-border);\n  border-radius: var(--fandhe-radius-lg);\n  overflow: hidden;\n}\n\
.blocks-ai-chat-code-preview-nav {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-3);\n  padding: var(--fandhe-space-3) var(--fandhe-space-4);\n  border-bottom: 1px solid var(--fandhe-color-border);\n}\n\
.blocks-ai-chat-code-preview-brand {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n  font-weight: 600;\n  min-width: 0;\n}\n\
.blocks-ai-chat-code-preview-brand img {\n  width: 1.5rem;\n  height: 1.5rem;\n  flex-shrink: 0;\n}\n\
.blocks-ai-chat-code-preview-nav-actions {\n  margin-inline-start: auto;\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-ai-chat-code-preview-switch {\n  display: none;\n  gap: var(--fandhe-space-2);\n  padding: var(--fandhe-space-2) var(--fandhe-space-4);\n  border-bottom: 1px solid var(--fandhe-color-border);\n}\n\
.blocks-ai-chat-code-preview-body {\n  display: grid;\n  grid-template-columns: minmax(18rem, 2fr) minmax(0, 3fr);\n  min-block-size: 28rem;\n}\n\
[data-blocks-ai-chat-code-preview-pane] {\n  display: flex;\n  flex-direction: column;\n  min-width: 0;\n  padding: var(--fandhe-space-4);\n  gap: var(--fandhe-space-3);\n}\n\
[data-blocks-ai-chat-code-preview-pane=\"chat\"] {\n  border-inline-end: 1px solid var(--fandhe-color-border);\n}\n\
[data-blocks-ai-chat-code-preview-scroller] {\n  block-size: 18rem;\n  overflow-y: auto;\n}\n\
.blocks-ai-chat-code-preview-composer {\n  display: flex;\n  gap: var(--fandhe-space-2);\n  align-items: flex-end;\n}\n\
.blocks-ai-chat-code-preview-composer [data-scope=\"field\"][data-part=\"textarea\"] {\n  flex: 1;\n}\n\
.blocks-ai-chat-code-preview-frame {\n  flex: 1;\n  min-block-size: 16rem;\n  border: 1px dashed var(--fandhe-color-border);\n  border-radius: var(--fandhe-radius-md);\n  display: grid;\n  place-items: center;\n  color: var(--fandhe-color-fg-muted);\n}\n\
@container blocks-ai-chat-code-preview (max-width: 47.99rem) {\n  .blocks-ai-chat-code-preview-body {\n    grid-template-columns: 1fr;\n  }\n  [data-blocks-ai-chat-code-preview-pane=\"chat\"] {\n    border-inline-end: 0;\n    border-block-end: 1px solid var(--fandhe-color-border);\n  }\n  .blocks-ai-chat-code-preview-switch {\n    display: flex;\n  }\n}\n";

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
            "data-scope=\"message\"",
            "data-scope=\"message-scroller\"",
            "data-scope=\"tabs\"",
            "data-scope=\"button\"",
            "data-scope=\"menu\"",
            "data-scope=\"code\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
        assert!(html.contains("data-scope=\"field\" data-part=\"textarea\""));
    }

    #[test]
    fn preview_tab_is_selected_and_code_tab_is_hidden() {
        let html = demo_html();
        assert!(html.contains(
            "id=\"blocks-ai-chat-code-preview-tabs-content-preview\" role=\"tabpanel\" aria-labelledby=\"blocks-ai-chat-code-preview-tabs-trigger-preview\" data-state=\"active\""
        ));
        let code_content_start = html
            .find("id=\"blocks-ai-chat-code-preview-tabs-content-code\"")
            .expect("code content should exist");
        let tail = &html[code_content_start..];
        let tag_end = tail.find('>').expect("content tag should close");
        assert!(tail[..tag_end].contains("hidden"));
    }

    #[test]
    fn no_form_and_all_buttons_are_type_button() {
        let html = demo_html();
        assert!(!html.contains("<form"));
        let count_open = html.matches("<button").count();
        let count_typed = html.matches("type=\"button\"").count();
        assert!(count_open > 0);
        assert!(count_typed >= count_open, "html={html}");
    }

    #[test]
    fn no_data_uri_or_dead_links() {
        let html = demo_html();
        assert!(!html.contains("src=\"data:"));
        assert!(!html.contains("href=\"#\""));
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
    fn layout_css_is_safe_and_never_hides_panes() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("container-type: inline-size;"));
        // `-switch` 規則以外に `display: none` が現れないことの回帰ガード
        // （モジュール doc「狭幅では『見出し表示 + 縦積み』で表現し、要素を
        // 隠さない」節参照）。
        let hide_count = LAYOUT_CSS.matches("display: none;").count();
        assert_eq!(hide_count, 1);
        assert!(LAYOUT_CSS.contains(".blocks-ai-chat-code-preview-switch {\n  display: none;"));
    }

    #[test]
    fn demo_is_deterministic() {
        assert_eq!(demo_html(), demo_html());
    }

    #[test]
    fn uses_shared_dummy_assets() {
        let html = demo_html();
        assert!(html.contains(super::dummy_assets::LOGO_SRC));
    }
}
