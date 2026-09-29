//! `ai-chat-code-preview` block（イシュー #2958。親 #2957「Application /
//! AI Chat」区分の 1 件目、Application / AI Chat カテゴリの卒業を伴う）。
//! 「上部にナビ、左列にチャット、右列にプレビュー/コード切替タブ」を持つ
//! コード生成チャット画面の合成例。
//!
//! # 使用部品
//!
//! `message` / `message-scroller` / `textarea` / `button` / `menu` /
//! `code` の 6 部品のみを合成する（[`BLOCK`] の `parts` に一致させる
//! 契約）。新しい UI 部品は追加しない。
//!
//! # 右列は実物の `tabs::tabs` を使わない（Codex P1 是正）
//!
//! 当初は `fandhe_frontend_pre_styled_ui::tabs::tabs` を `preview` 選択
//! 固定で使い、`code` パネルは headless tabs の `hidden` 属性で非表示に
//! していた。無 JS の docs サイトではタブ切替が実際には起きないため、
//! `code` パネル（コード例という主要コンテンツ）が恒久的に到達不能に
//! なる不具合だった（`feature_tabs_panel.rs`「無 JS での扱い」節と同型の
//! 判断軸）。是正として右列は実物の `tabs::tabs` を一切使わず、
//! [`static_tab_list`]（`role`/`tabindex`/`<button>` を持たない装飾のみの
//! 見た目、`data-scope="tabs"` とは意図的に不一致な独自 class）で
//! タブ列の見た目だけを再現し、プレビュー・コードの両パネルを常に
//! 可視のまま縦積みで併記する（`hidden` を一切使わない）。状態違いの
//! 原稿仕上げ・併記は #2959 のスコープ。
//!
//! # 狭幅の「見出し表示 + 縦積み」から独立した右列の常時併記
//!
//! 上記の右列縦積みは幅に関わらず常時であり、次節の左右 2 列 ⇔ 縦積みの
//! 切替（狭幅専用）とは別の層の設計判断である。
//!
//! # 狭幅では「見出し表示 + 縦積み」で表現し、要素を隠さない
//!
//! `@container` で [`LAYOUT_CSS`] が body を 1 列へ縮退させるだけで、両
//! パネル（chat/preview）はどちらも常に到達可能なまま DOM に残る。狭幅
//! 専用の [`switch_group`] は選択状態を示さない非対話の領域見出し `span`
//! 2 個であり、実際のパネル切替（`display: none`）は行わない
//! （Codex P2 是正: 当初は `aria-pressed` トグルだったが、押しても
//! 状態が変わらないため支援技術に誤った操作性を示唆していた。続けて
//! `aria-current="true"` へ差し替えたが、両領域が常に併記されるのに
//! チャット側だけを「現在表示中」と示すのは同じく誤りだったため、
//! 最終的に選択状態を持たない対等な見出しへ変更した。`page_heading_avatar.rs`
//! の codex レビュー指摘「狭幅で
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
use fandhe_frontend_pre_styled_ui::textarea::{self, FieldIds, FieldProps, TextareaProps};
use fandhe_frontend_pre_styled_ui::Size;

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

/// 狭幅専用の領域見出し（モジュール doc「狭幅では『見出し表示 + 縦積み』で
/// 表現し、要素を隠さない」節参照）。チャット・プレビューの両領域は狭幅でも
/// 常に併記表示され切替は起きないため、`button`/`aria-pressed`（Codex P2
/// 是正済み）に加えて `role="group"`/`aria-label="表示切替"`/
/// `aria-current="true"` も使わない（Codex P2 是正: 両領域が表示された
/// ままチャット側だけへ「切替グループ」「現在表示中」を示す ARIA を
/// 付けると、支援技術にはプレビューが表示されていないかのように伝わる）。
/// [`static_tab_list`] と同型の非対話 `span`（`role`/`tabindex`/`<button>`
/// なし）2 個を、選択状態の区別を持たない対等な見出しラベルとして並べる。
fn switch_group() -> Node {
    let chat = span(
        vec![("class", "blocks-ai-chat-code-preview-switch-item")],
        vec![text("チャット")],
    );
    let preview = span(
        vec![("class", "blocks-ai-chat-code-preview-switch-item")],
        vec![text("プレビュー")],
    );
    div(
        vec![("class", "blocks-ai-chat-code-preview-switch")],
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

    // 高さは `message_scroller::root` recipe が持つ CSS カスタムプロパティ
    // `--fandhe-message-scroller-height`（既定 24rem）を `style` で上書きして
    // 指定する（Bugbot 是正: block 固有属性セレクタ 1 個での `block-size`
    // 上書きは recipe 本体セレクタ `[data-scope="message-scroller"]
    // [data-part="root"]`〔属性 2 個〕より詳細度が低く、カスケードで負けて
    // 反映されなかった不具合。CSS カスタムプロパティは recipe が
    // `var(...)` で参照する側であり詳細度勝負にならないため確実に効く）。
    let scroller = message_scroller::root(
        MessageScrollerRootProps {
            stuck: MessageScrollerStuck::Bottom,
            has_new: false,
        },
        vec![
            ("data-blocks-ai-chat-code-preview-scroller", ""),
            ("style", "--fandhe-message-scroller-height: 18rem"),
        ],
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

/// タブ列の見た目だけを再現する非対話表示（モジュール doc「右列は実物の
/// `tabs::tabs` を使わない」節参照）。`role`/`tabindex`/`<button>` を
/// 一切持たず、`data-scope="tabs"` とも意図的に不一致な独自 class
/// （`.blocks-ai-chat-code-preview-tablist`/`-tab`）で `LAYOUT_CSS` 側の
/// 見た目を組む（`feature_tabs_panel.rs::static_tab_list` と同型の判断:
/// recipe とセレクタを共有すると `:hover` 規則が非対話タブ列にも当たり
/// 操作可能に見えてしまうため）。プレビュー・コードのどちらも常に可視で
/// 併記するため、どちらか一方だけを選択中として強調しない（`aria-hidden`
/// で装飾として支援技術のツリーから除外する。Codex P2 是正: 当初は
/// 「プレビュー」側にのみ `is-active` を付けていたが、直後の両パネルは
/// 常に併記されるため、コード側が未選択であるかのように示す表示状態の
/// 不一致だった。`switch_group` の Codex P2 是正〔対等な見出しへ変更〕と
/// 同型の判断で、選択状態を表す視覚的な強調を持たない対等な見出しへ
/// 変更した）。
fn static_tab_list() -> Node {
    div(
        vec![
            ("class", "blocks-ai-chat-code-preview-tablist"),
            ("aria-hidden", "true"),
        ],
        vec![
            span(
                vec![("class", "blocks-ai-chat-code-preview-tab")],
                vec![text("プレビュー")],
            ),
            span(
                vec![("class", "blocks-ai-chat-code-preview-tab")],
                vec![text("コード")],
            ),
        ],
    )
}

/// 右列（プレビュー + コード例）を組み立てる。タブ切替の見た目は
/// [`static_tab_list`] で示すのみで、プレビュー・コードの両パネルは
/// `hidden` を使わず常に可視のまま縦積みで併記する（モジュール doc
/// 「右列は実物の `tabs::tabs` を使わない」節参照、Codex P1 是正:
/// 無 JS では到達不能になる `hidden` パネルを持たない）。
fn preview_pane() -> Node {
    let frame = div(
        vec![("class", "blocks-ai-chat-code-preview-frame")],
        vec![span(vec![], vec![text("プレビュー")])],
    );
    let code_block = el(
        "pre",
        vec![("class", "blocks-ai-chat-code-preview-code")],
        vec![code::code(
            &CodeProps::default(),
            vec![],
            vec![text(CODE_SNIPPET)],
        )],
    );

    el(
        "section",
        vec![("data-blocks-ai-chat-code-preview-pane", "preview")],
        vec![static_tab_list(), frame, code_block],
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
.blocks-ai-chat-code-preview-switch-item {\n  padding: var(--fandhe-space-1) var(--fandhe-space-3);\n  border-radius: var(--fandhe-radius-md);\n  font-size: var(--fandhe-font-font-size-sm);\n  color: var(--fandhe-color-fg-muted);\n}\n\
.blocks-ai-chat-code-preview-body {\n  display: grid;\n  grid-template-columns: minmax(18rem, 2fr) minmax(0, 3fr);\n  min-block-size: 28rem;\n}\n\
[data-blocks-ai-chat-code-preview-pane] {\n  display: flex;\n  flex-direction: column;\n  min-width: 0;\n  padding: var(--fandhe-space-4);\n  gap: var(--fandhe-space-3);\n}\n\
[data-blocks-ai-chat-code-preview-pane=\"chat\"] {\n  border-inline-end: 1px solid var(--fandhe-color-border);\n}\n\
.blocks-ai-chat-code-preview-composer {\n  display: flex;\n  gap: var(--fandhe-space-2);\n  align-items: flex-end;\n}\n\
.blocks-ai-chat-code-preview-composer [data-scope=\"field\"][data-part=\"textarea\"] {\n  flex: 1;\n}\n\
.blocks-ai-chat-code-preview-tablist {\n  display: flex;\n  gap: var(--fandhe-space-2);\n  border-bottom: 1px solid var(--fandhe-color-border);\n  padding-bottom: var(--fandhe-space-2);\n}\n\
.blocks-ai-chat-code-preview-tab {\n  padding: var(--fandhe-space-1) var(--fandhe-space-3);\n  border-radius: var(--fandhe-radius-md) var(--fandhe-radius-md) 0 0;\n  font-size: var(--fandhe-font-font-size-sm);\n  color: var(--fandhe-color-fg-muted);\n}\n\
.blocks-ai-chat-code-preview-frame {\n  flex: 1;\n  min-block-size: 16rem;\n  border: 1px dashed var(--fandhe-color-border);\n  border-radius: var(--fandhe-radius-md);\n  display: grid;\n  place-items: center;\n  color: var(--fandhe-color-fg-muted);\n}\n\
.blocks-ai-chat-code-preview-code {\n  margin: 0;\n  padding: var(--fandhe-space-3);\n  border: 1px solid var(--fandhe-color-border);\n  border-radius: var(--fandhe-radius-md);\n  overflow-x: auto;\n}\n\
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
            "data-scope=\"button\"",
            "data-scope=\"menu\"",
            "data-scope=\"code\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
        assert!(html.contains("data-scope=\"field\" data-part=\"textarea\""));
    }

    /// 右列は `hidden` を一切使わず、プレビュー・コードの両パネルが常に
    /// 到達可能であることを固定する回帰ガード（Codex P1 是正、モジュール
    /// doc「右列は実物の `tabs::tabs` を使わない」節参照）。
    #[test]
    fn preview_and_code_panels_are_both_reachable_without_hidden() {
        let html = demo_html();
        // 右列（`data-blocks-ai-chat-code-preview-pane="preview"`）に限定して
        // `hidden` の不在を検証する（三点メニューの `OpenState::Closed` は
        // `overflow_menu` doc の既定判断どおり `hidden` を持つため、全文
        // 検証にすると無関係な既定挙動を誤検知する）。
        let pane_start = html
            .find("data-blocks-ai-chat-code-preview-pane=\"preview\"")
            .expect("preview pane should exist");
        let pane_html = &html[pane_start..];
        // `aria-hidden="true"`（[`static_tab_list`] の装飾用、支援技術の
        // ツリーからの除外であり視覚的な非表示ではない）は許容し、実際に
        // 内容を非表示にする `hidden` 真偽属性（` hidden` の形で出力される）
        // のみを検知する。
        assert!(
            !pane_html.contains(" hidden"),
            "preview pane should not hide any content: {pane_html}"
        );
        // `CODE_SNIPPET` は `<`/`>` を含み HTML エスケープ後は非一致になる
        // ため、エスケープの影響を受けない語で存在確認する。
        assert!(pane_html.contains("async fn inventory"));
        assert!(pane_html.contains("blocks-ai-chat-code-preview-tablist"));
    }

    /// タブ列の見た目は、直後に併記されるプレビュー・コードのどちらか
    /// 一方だけを選択中として強調しないことを固定する（Codex P2 是正、
    /// [`static_tab_list`] doc 参照）。`is-active` は両パネル併記という
    /// 実態と矛盾する表示状態を示すため使わない。
    #[test]
    fn static_tab_list_has_no_misleading_active_state() {
        let html = demo_html();
        assert!(!html.contains("is-active"));
    }

    /// 狭幅専用の表示切替は `aria-pressed`（実際に押せて状態が変わる
    /// トグルを示唆する ARIA）を持たないことを固定する（Codex P2 是正）。
    #[test]
    fn switch_group_has_no_misleading_aria_pressed() {
        let html = demo_html();
        assert!(!html.contains("aria-pressed"));
    }

    /// チャット・プレビューの両領域は狭幅でも常に併記表示されるため、
    /// 片方だけを「現在表示中」と示す `aria-current` や、実際には切り替わ
    /// らない「切替グループ」を示唆する `aria-label="表示切替"` を持たない
    /// ことを固定する（Codex P2 是正、[`switch_group`] doc 参照）。
    #[test]
    fn switch_group_has_no_misleading_current_state() {
        let html = demo_html();
        assert!(!html.contains("aria-current"));
        assert!(!html.contains("表示切替"));
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
