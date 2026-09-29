# ai-chat-code-preview

`fandhe-frontend-pre-styled-ui` の `message` / `message-scroller` /
`textarea` / `button` / `menu` / `code` 部品を合成した、コード生成チャット
画面の骨格の実例です。Blocks セクションは新規部品を追加するものではなく、
既存の Themes/Primitives 部品を組み合わせた実例集であることに注意して
ください（主参照は対応表 ID R0001。出典の固有名・ファイル名は記載しません）。

上部にロゴ・プロジェクト名・操作ボタン（共有/公開）・三点メニューを持つ
ナビ、左列にメッセージ履歴と入力欄、右列に「プレビュー / コード」の見出し
タブ列 + 両パネルの常時併記を配置しています。狭い幅では左右 2 列が縦積みに
切り替わり、どちらの領域も常に到達できます（`display: none` で隠すのは
狭幅専用の表示切替 UI のみで、内容パネル自体は隠しません）。

本 Demo は静的な表示例であり、`<form>` 要素は一切持たず、送信・生成処理・
永続化を行いません。右列は実物の `tabs::tabs` を使わず、タブ列の見た目
だけを非対話表示で再現し、プレビュー・コードの両パネルを `hidden` なしで
常に可視のまま縦積みで併記します（無 JS のためタブ切替自体が実際には起き
ず、`hidden` パネルにすると主要コンテンツであるコード例が恒久的に到達
不能になるため）。狭幅専用の表示切替も同じ理由で `aria-pressed` を持つ
`button` ではなく非対話 `span` にしています。三点メニューは閉じた状態の
固定表示です（開閉には `fandhe-frontend-wasm-full` の JS 配線が必要で、
docs サイトは JS ハイドレーションを行いません）。文言はすべて独自に書いた
架空のものであり、実企業名・実クレデンシャル・PII を含みません。

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
```

## 原案差分メモ

- 主参照は R0001 のみ（集約元 1 件）です。`_/blocks-intake/` の対応
  ファイルは本イシュー着手時点で本 worktree に存在しないため、対応表 ID
  のみを記録しています（`page-heading-avatar.md` と同じ扱い）。
- PR #3411 レビュー（Codex P1/P2、Bugbot Medium）の是正として、右列は
  実物の `tabs::tabs` の使用をやめ、静的タブ列 + 両パネル常時併記へ変更
  しました。狭幅専用の表示切替も `aria-pressed` を外し非対話 `span` へ
  変更しています。左列のメッセージスクローラーの高さ指定は CSS カスタム
  プロパティ経由に変更し、cascade specificity の不具合を解消しました。
  状態違いの併記・本メモの差分反映の仕上げは後続イシュー #2959 で行い
  ます。
- 実データ取得・生成処理・メニュー開閉・ボタン押下は行わず、静的な初期
  状態のみを示します。プロジェクト名・会話文・コード片はすべて独自の
  架空データです。
- ブラウザでの実機確認（`48rem` 前後のコンテナ幅切替・ライト/ダーク両
  テーマ）はサンドボックス制約により未実施です。cargo test による出力
  検証のみで代替しました。
