# ai-chat-code-preview

`fandhe-frontend-pre-styled-ui` の `message` / `message-scroller` /
`textarea` / `tabs` / `button` / `menu` / `code` 部品を合成した、コード
生成チャット画面の骨格の実例です。Blocks セクションは新規部品を追加する
ものではなく、既存の Themes/Primitives 部品を組み合わせた実例集である
ことに注意してください（主参照は対応表 ID R0001。出典の固有名・ファイル
名は記載しません）。

上部にロゴ・プロジェクト名・操作ボタン（共有/公開）・三点メニューを持つ
ナビ、左列にメッセージ履歴と入力欄、右列に「プレビュー / コード」を
切り替えるタブ付き表示領域（プレビューは枠のみの静的表示、コードは短い
Rust コード例）を配置しています。狭い幅では左右 2 列が縦積みに切り替わり、
どちらの領域も常に到達できます（`display: none` で隠すのは狭幅専用の
表示切替 UI のみで、内容パネル自体は隠しません）。

本 Demo は静的な表示例であり、`<form>` 要素は一切持たず、送信・生成処理・
永続化を行いません。ボタンは `type="button"` のまま送信先を持たず、
右列の tabs は「プレビュー」選択で固定した初期状態のみを描画します
（`code` パネルは無 JS のため `hidden` 属性で非表示になります。コード
パネルを可視の状態違いとして併記する対応は後続イシュー #2959 で行い
ます）。三点メニューは閉じた状態の固定表示です（開閉には
`fandhe-frontend-wasm-full` の JS 配線が必要で、docs サイトは JS
ハイドレーションを行いません）。文言はすべて独自に書いた架空のものであり、
実企業名・実クレデンシャル・PII を含みません。

## Rust コード

```rust
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
```

## 原案差分メモ

- 主参照は R0001 のみ（集約元 1 件）です。`_/blocks-intake/` の対応
  ファイルは本イシュー着手時点で本 worktree に存在しないため、対応表 ID
  のみを記録しています（`page-heading-avatar.md` と同じ扱い）。
- 右列の tabs は「プレビュー」選択の 1 インスタンスのみを描画します。
  `code` タブ選択時の状態違い（コード例を可視で示す第 2 インスタンス）の
  併記、および本メモの差分反映の仕上げは後続イシュー #2959 で行います。
- 実データ取得・生成処理・メニュー開閉・ボタン押下は行わず、静的な初期
  状態のみを示します。プロジェクト名・会話文・コード片はすべて独自の
  架空データです。
- ブラウザでの実機確認（`48rem` 前後のコンテナ幅切替・ライト/ダーク両
  テーマ）はサンドボックス制約により未実施です。cargo test による出力
  検証のみで代替しました。
