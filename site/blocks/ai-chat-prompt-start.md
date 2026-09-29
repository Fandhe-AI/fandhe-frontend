# ai-chat-prompt-start

`fandhe-frontend-pre-styled-ui` の `textarea` / `button` / `empty-state` /
`menu` / `icon` / `heading` 部品を合成した、AI チャットの開始画面の実例
です。Blocks セクションは新規部品を追加するものではなく、既存の
Themes/Primitives 部品を組み合わせた実例集であることに注意してください
（主参照は対応表 ID R0002、集約元は R0003・R0004。出典の固有名・ファイル名は
記載しません）。

上部にアイコン付きの挨拶見出し、その下に候補ボタン 2×2、下部に添付・
ツール選択・送信ボタン付きの複数行入力欄（composer）を配置した基本形
（default）に加え、見出しと composer を画面中央へ寄せ、候補ボタンを控えめな
意匠で composer の下に配置した形（centered）の 2 通りを並べています。狭い幅
では候補ボタンが 1 列に積み重なります。

本 Demo は静的な表示例であり、`<form>` 要素は一切持たず、送信・添付・ツール
選択の処理を行いません。ボタンは `type="button"` のまま送信先を持たず、
ツール選択メニューは閉じた状態の固定表示です（開閉には
`fandhe-frontend-wasm-full` の JS 配線が必要で、docs サイトは JS
ハイドレーションを行いません）。文言はすべて独自に書いた架空のものであり、
実企業名・実クレデンシャル・PII を含みません。

## Rust コード

```rust
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::empty_state::{self, EmptyStateProps};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::icon::{self, IconProps};
use fandhe_frontend_pre_styled_ui::menu::{self, OpenState};
use fandhe_frontend_pre_styled_ui::textarea::{self, FieldIds, FieldProps, TextareaProps};
use fandhe_frontend_pre_styled_ui::Size;

/// 装飾用アイコン（吹き出し状の抽象図形、`aria-hidden` 固定・実在ブランド
/// のロゴを模さない）。
fn sparkle_icon() -> Node {
    icon::icon(
        &IconProps::default(),
        vec![],
        vec![el(
            "path",
            vec![(
                "d",
                "M12 3l1.8 5.2L19 10l-5.2 1.8L12 17l-1.8-5.2L5 10l5.2-1.8z",
            )],
            vec![],
        )],
    )
}

/// 添付アイコン（クリップ状の抽象図形）。
fn attach_icon() -> Node {
    icon::icon(
        &IconProps {
            size: Size::Sm,
            ..IconProps::default()
        },
        vec![],
        vec![el(
            "path",
            vec![(
                "d",
                "M8 12V6a3 3 0 0 1 6 0v8a5 5 0 0 1-10 0V7h2v7a3 3 0 0 0 6 0V6a1 1 0 0 0-2 0v6H8z",
            )],
            vec![],
        )],
    )
}

/// 送信アイコン（上矢印の抽象図形）。
fn send_icon() -> Node {
    icon::icon(
        &IconProps {
            size: Size::Sm,
            ..IconProps::default()
        },
        vec![],
        vec![el("path", vec![("d", "M12 4l6 7h-4v9h-4v-9H6z")], vec![])],
    )
}

/// 候補ボタン 1 個（`data-blocks-ai-chat-prompt-start-suggestion` で
/// `suggestions_are_four_per_instance` テストが数える）。
fn suggestion_button(variant: ButtonVariant, label: &'static str) -> Node {
    button::button(
        &ButtonProps {
            variant,
            size: Size::Md,
            ..ButtonProps::default()
        },
        vec![("data-blocks-ai-chat-prompt-start-suggestion", "")],
        vec![text(label)],
    )
}

/// 候補ボタン群（2×2、[`LAYOUT_CSS`] の grid で狭幅では 1 列に積む）。
fn suggestions(variant: ButtonVariant, labels: [&'static str; 4]) -> Node {
    div(
        vec![("class", "blocks-ai-chat-prompt-start-suggestions")],
        labels
            .into_iter()
            .map(|label| suggestion_button(variant, label))
            .collect(),
    )
}

/// 挨拶見出し（[`empty_state`] + [`heading`] の合成、無 JS の装飾アイコン
/// 付き）。
fn greeting() -> Node {
    empty_state::root(
        &EmptyStateProps::default(),
        vec![],
        vec![
            empty_state::indicator(vec![], vec![sparkle_icon()]),
            empty_state::content(
                vec![],
                vec![
                    empty_state::title(
                        vec![],
                        vec![heading::heading(
                            HeadingLevel::H2,
                            &HeadingProps {
                                size: HeadingSize::Lg,
                                ..HeadingProps::default()
                            },
                            vec![],
                            vec![text("今日は何から始めますか")],
                        )],
                    ),
                    empty_state::description(
                        vec![],
                        vec![text(
                            "コードの相談、文章の下書き、調べものまで、なんでも聞いてください。",
                        )],
                    ),
                ],
            ),
        ],
    )
}

/// ツール選択メニュー（[`menu`] 部品、無 JS のため `OpenState::Closed`
/// 固定。モジュール doc「三点メニューは無 JS のため閉じた状態で固定する」
/// 節参照）。`trigger_id`/`content_id` はインスタンスごとに一意な id を
/// 呼び出し側（[`composer`]）が渡す。
fn tools_menu(trigger_id: &str, content_id: &str) -> Node {
    let trigger = menu::trigger(
        OpenState::Closed,
        false,
        Some(content_id),
        vec![("id", trigger_id)],
        vec![text("ツール")],
    );
    let content = menu::content(
        OpenState::Closed,
        Some(content_id),
        Some(trigger_id),
        vec![],
        vec![
            menu::item("web-search", false, false, vec![], vec![text("Web 検索")]),
            menu::item(
                "code-interpreter",
                false,
                false,
                vec![],
                vec![text("コード実行")],
            ),
            menu::item(
                "image-generation",
                false,
                false,
                vec![],
                vec![text("画像生成")],
            ),
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

/// composer（複数行入力欄 + 添付・ツール選択・送信ボタン）。`suffix` で
/// インスタンスごとに id を一意化する（`page_heading_avatar.rs` と同型に
/// `format!` で組み立てた `String` をローカル変数として保持し、その借用を
/// 各部品関数へ渡す）。
fn composer(suffix: &str) -> Node {
    let field_id = format!("blocks-ai-chat-prompt-start-prompt-{suffix}");
    let trigger_id = format!("blocks-ai-chat-prompt-start-tools-trigger-{suffix}");
    let content_id = format!("blocks-ai-chat-prompt-start-tools-content-{suffix}");
    let field_props = FieldProps {
        id: &field_id,
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: false,
    };

    let prompt = textarea::textarea(
        &TextareaProps::default(),
        &field_props,
        false,
        vec![
            ("placeholder", "メッセージを入力…"),
            ("aria-label", "メッセージを入力"),
            ("rows", "3"),
        ],
        vec![],
    );

    let attach = button::icon_button(
        &ButtonProps {
            variant: ButtonVariant::Outline,
            size: Size::Sm,
            ..ButtonProps::default()
        },
        "ファイルを添付",
        vec![],
        vec![attach_icon()],
    );
    let tools = tools_menu(&trigger_id, &content_id);
    let send = button::icon_button(
        &ButtonProps {
            variant: ButtonVariant::Solid,
            size: Size::Sm,
            ..ButtonProps::default()
        },
        "送信",
        vec![],
        vec![send_icon()],
    );

    div(
        vec![("class", "blocks-ai-chat-prompt-start-composer")],
        vec![
            prompt,
            div(
                vec![("class", "blocks-ai-chat-prompt-start-composer-actions")],
                vec![attach, tools, send],
            ),
        ],
    )
}

/// **default**（R0002 主参照 + R0003 の候補ボタン意匠）インスタンス。
fn default_instance() -> Node {
    div(
        vec![
            ("class", "blocks-ai-chat-prompt-start-panel"),
            ("data-blocks-ai-chat-prompt-start-variant", "default"),
        ],
        vec![
            greeting(),
            suggestions(
                ButtonVariant::Outline,
                [
                    "要点を 3 行にまとめる",
                    "テストケースを提案する",
                    "文章を校正する",
                    "アイデアを出し合う",
                ],
            ),
            composer("default"),
        ],
    )
}

/// **centered**（R0004、見出しと入力欄を中央へ寄せる）インスタンス。
fn centered_instance() -> Node {
    div(
        vec![
            ("class", "blocks-ai-chat-prompt-start-panel"),
            ("data-blocks-ai-chat-prompt-start-variant", "centered"),
        ],
        vec![
            greeting(),
            composer("centered"),
            suggestions(
                ButtonVariant::Ghost,
                [
                    "旅行の計画を立てる",
                    "レシピを提案する",
                    "学習プランを作る",
                    "メールの下書きを書く",
                ],
            ),
        ],
    )
}

/// `ai-chat-prompt-start` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数。主参照（R0002・default）を先頭に、2 インスタンスを縦に並べる。
#[must_use]
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-ai-chat-prompt-start-layout")],
        vec![default_instance(), centered_instance()],
    )
}
```

## 原案差分メモ

- 主参照は R0002（挨拶見出し + 候補ボタン + composer の代表構成）です。
  R0003（挨拶見出し + 控えめな候補ボタン意匠）を `centered` インスタンスの
  候補ボタン variant（`Ghost`）として、R0004（見出しと入力欄を画面中央へ
  寄せる構成）を `centered` インスタンス全体のレイアウトとして反映しました。
- 使用部品一覧（`textarea` / `button` / `empty-state` / `menu` / `icon` /
  `heading`）に `field`/`input-group` が含まれないため、`hero_prompt_input`
  のように `field::root`/`input_group::root` で複数行入力欄を包まず、
  `textarea::textarea` を素の `div` で直接包んでいます。
- 「入力欄を画面下端に固定する」という原案の要件は、Demo 表示枠
  （`.blocks-demo`）が横スクロールコンテナのため `position: sticky` が
  機能保証されない制約から、パネル内の flex 末尾へ寄せる
  `margin-top: auto` で表現しました。実アプリでは
  `position: sticky; bottom: 0` を使えます。
- アイコンはすべて抽象的な線画（吹き出し・クリップ・矢印）の独自 SVG
  path であり、実在ブランドのロゴ・商標は模していません。
- 添付・送信・ツール選択の実処理は行わず、静的な初期状態のみを示します。
  文言はすべて独自の架空データです。
- ブラウザでの実機確認（`48rem` 前後のコンテナ幅切替・ライト/ダーク両
  テーマ）はサンドボックス制約により未実施です。cargo test による出力
  検証のみで代替しました。
