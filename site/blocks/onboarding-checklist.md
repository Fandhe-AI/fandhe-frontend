# onboarding-checklist

見出し + 達成率バーの下に、開始タスクのチェックリストを組み合わせた
スタートガイドブロックです。`progress` / `steps` / `checkbox` / `button` /
`heading` / `text` / `visually-hidden` の 7 部品を合成します。Blocks は
既存部品の合成例であり、新しい UI 部品は追加しません。

対応表 ID R0173（代表構成、集約差分なし）を参照しています。完了タスクは
チェック済み checkbox + 取り消し線ラベルのみを持ち、未完了タスクは説明文と
実行ボタンを持ちます。先頭の未完了タスクだけが展開表示され、他の未完了
タスクは折りたたまれます。タスク名・説明はすべて架空の文言です。

本 Demo は無 JS の静的表示のみであり、`<form>` を含みません。

## Rust コード

```rust
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::checkbox::{self, CheckboxProps, CheckedState};
use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::progress::Progress;
use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::steps::Steps;
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps};
use fandhe_frontend_pre_styled_ui::progress::{self, ProgressProps};
use fandhe_frontend_pre_styled_ui::steps;
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::visually_hidden;
use fandhe_frontend_pre_styled_ui::{ColorPalette, Orientation, Size};

/// 開始タスク一覧（タイトル・説明）。すべて架空の文言。
const TASKS: [(&str, &str); 5] = [
    (
        "プロフィールを設定する",
        "顔写真と自己紹介を登録すると、チームに自分を知ってもらえます。",
    ),
    (
        "チームメンバーを招待する",
        "メールアドレスを入力して、一緒に作業するメンバーを招待します。",
    ),
    (
        "最初のプロジェクトを作成する",
        "名前と説明を入力して、最初のプロジェクトを作ります。",
    ),
    (
        "通知先を接続する",
        "チャットツールと連携して、更新情報を受け取れるようにします。",
    ),
    (
        "モバイルアプリをインストールする",
        "外出先でも進捗を確認できるよう、モバイルアプリを設定します。",
    ),
];

/// 完了済みタスク件数（先頭 `DONE` 件が完了・`DONE` 件目が「次にやること」
/// として展開表示される）。
const DONE: usize = 2;

/// 見出し + 達成率テキスト + 達成率バーの見出し行。
fn header(progress: &Progress) -> Node {
    div(
        vec![("class", "blocks-onboarding-checklist-header")],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps::default(),
                vec![],
                vec![text("はじめの設定")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    size: TextSize::Sm,
                    ..TextProps::default()
                },
                vec![],
                vec![text(format!(
                    "{} 件中 {} 件が完了しました",
                    TASKS.len(),
                    DONE
                ))],
            ),
            progress::root(
                progress,
                &ProgressProps::default(),
                Some(&format!("{} 件中 {} 件完了", TASKS.len(), DONE)),
                // `aria-labelledby` の自動配線は headless/styled いずれの層
                // の責務でもない（`fandhe_frontend_pre_styled_ui::progress`
                // rustdoc「イシュー #2049」節参照）ため呼び出し側で明示する。
                vec![("aria-label", "設定の達成率")],
                vec![progress.track(vec![], vec![progress::range(progress, vec![])])],
            ),
        ],
    )
}

/// タスクの完了印（checkbox）。ネイティブ `disabled` で状態を固定する
/// 理由はモジュール冒頭「checkbox をネイティブ `disabled` にする理由」節
/// 参照。
fn task_mark(index: usize, title: &'static str, complete: bool) -> Node {
    let props = CheckboxProps {
        checked: if complete {
            CheckedState::Checked
        } else {
            CheckedState::Unchecked
        },
        disabled: true,
        ..CheckboxProps::default()
    };
    let name = format!("task-{}", index + 1);
    checkbox::root(
        Size::Md,
        ColorPalette::Accent,
        &props,
        vec![("data-blocks-onboarding-checklist-task", "")],
        vec![
            checkbox::hidden_input(&props, &name, "on", vec![]),
            checkbox::control(
                &props,
                vec![],
                vec![checkbox::indicator(&props, vec![], vec![])],
            ),
            checkbox::label(&props, vec![], vec![text(title)]),
        ],
    )
}

/// 未完了タスクの説明 + 実行ボタン（`steps::content`。current
/// のみ `open`、他は `hidden`。モジュール冒頭「先頭の未完了タスクだけ
/// 展開」節参照）。実行ボタンの可視テキストは全タスク共通のため、
/// `visually_hidden::root` でタスク名を不可視テキストとして補う。
fn task_body(steps: &Steps, index: usize, title: &'static str, description: &'static str) -> Node {
    steps::content(
        steps,
        index,
        vec![("class", "blocks-onboarding-checklist-task-body")],
        vec![
            styled_text::text(
                &TextProps {
                    size: TextSize::Sm,
                    ..TextProps::default()
                },
                vec![],
                vec![text(description)],
            ),
            button::button(
                &ButtonProps {
                    size: Size::Sm,
                    ..ButtonProps::default()
                },
                vec![],
                vec![
                    text("開始する"),
                    visually_hidden::root(vec![], vec![text(format!("（{title}）"))]),
                ],
            ),
        ],
    )
}

/// タスク 1 件分（checkbox + 未完了時のみ本文）。
fn task_item(steps: &Steps, index: usize, title: &'static str, description: &'static str) -> Node {
    let complete = index < DONE;
    let mut children = vec![task_mark(index, title, complete)];
    if !complete {
        children.push(task_body(steps, index, title, description));
    }
    steps::item(steps, index, vec![], children)
}

/// `onboarding-checklist` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。
pub fn demo() -> Node {
    let steps_state = Steps::new(TASKS.len(), DONE, Orientation::Vertical);
    let progress_state = Progress::new(
        0.0,
        TASKS.len() as f64,
        Some(DONE as f64),
        Orientation::Horizontal,
    );

    let items: Vec<Node> = TASKS
        .iter()
        .enumerate()
        .map(|(index, (title, description))| task_item(&steps_state, index, title, description))
        .collect();

    let steps_root = steps::root(
        Size::Md,
        ColorPalette::Accent,
        &steps_state,
        vec![("data-blocks-onboarding-checklist-steps", "")],
        vec![steps::list(&steps_state, vec![], items)],
    );

    div(
        vec![("class", "blocks-onboarding-checklist-stack")],
        vec![header(&progress_state), steps_root],
    )
}
```

## 原案差分メモ

- `steps::trigger`/`indicator`/`separator` は使いません。タスクの完了印は
  checkbox が担うため trigger・indicator は二重表現になり、各 item は
  「checkbox + 本文」の縦積みブロックのため item 間の接続線（separator）は
  意味を持ちません。
- checkbox はすべてネイティブ `disabled` にしています。`disabled` を渡さない
  構成では、JS ハイドレーションを行わない docs サイトでもラベルクリックで
  ブラウザが `checked` をネイティブに切り替えてしまい、見た目（`control`/
  `indicator`）は SSR 時の状態から追従しないため崩れます。`disabled` の
  既定の見た目（半透明・カーソル）は block 側 CSS で中和しています。
- 「先頭の未完了タスクだけ展開」は `Steps` の `step`（current）を最初の
  未完了タスクの index に固定し、`steps::content` の `open`/`hidden` 切り替え
  という部品の意味論だけで表現しています。
- 狭幅（コンテナ幅 32rem 未満）では実行ボタンが説明文の下へ折り返ります
  （`@container` によるコンテナクエリ判定）。

関連情報: [Progress](../themes/progress.md) / [Steps](../themes/steps.md) /
[Checkbox](../themes/checkbox.md) / [Button](../themes/button.md) /
[Heading](../themes/heading.md) / [Text](../themes/text.md) /
[Visually Hidden](../themes/visually-hidden.md)
