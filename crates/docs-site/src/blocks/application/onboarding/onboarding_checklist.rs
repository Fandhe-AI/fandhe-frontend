//! `onboarding-checklist` block（イシュー #2979。Application / Onboarding
//! カテゴリ、最初の block）。見出し + 達成率バーの下に開始タスクの
//! チェックリストを合成する。主参照 R0173（代表構成のみ、集約差分なし）。
//! `_/blocks-intake/` の対応ファイルは本イシュー着手時点で本 worktree に
//! 存在しないため、原稿・本コメントには対応表 ID のみを記す
//! （`profile_detail_datalist.rs`〔イシュー #2937〕と同じ扱い）。
//!
//! # 使用部品
//!
//! `progress` / `steps` / `checkbox` / `button` / `heading` / `text` /
//! `visually-hidden` の 7 部品を合成する（[`BLOCK`] の `parts` に一致
//! させる契約、`crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs`
//! が検証する）。新しい UI 部品は追加しない。
//!
//! # 構成
//!
//! 見出し + 達成率テキスト + `progress::root`（横棒）の下に、
//! `steps::root(Orientation::Vertical)` で 5 タスクの縦積みチェックリストを
//! 組む。完了タスクはチェック済み checkbox + 取り消し線ラベルのみを持ち、
//! 未完了タスクはさらに説明文 + 実行ボタンを持つ。**先頭の未完了タスクだけ**
//! `steps::content` が `open` 状態で表示され（他は `hidden`）、「次にやる
//! こと」だけが展開された静的表示になる。
//!
//! # `steps::trigger`/`indicator`/`separator` を置かない理由
//!
//! - **`trigger`**: 実 `<button>` であり、無 JS の docs サイトでは押しても
//!   何も起きない dead control になる（`empty_state_setup_steps.rs` と
//!   同型の判断）。タスクの完了印は checkbox（チェック状態）が担うため
//!   trigger は不要。
//! - **`indicator`**: 円形の順序マーカーだが、本 Demo の「印」は checkbox
//!   が担うため二重表現になる。
//! - **`separator`**: 各 item は「checkbox + 本文」の縦積みブロックであり、
//!   item 間の接続線は意味を持たない。
//!
//! # checkbox をネイティブ `disabled` にする理由
//!
//! `checkbox::hidden_input` は有効なネイティブ `<input>` であり、`disabled`
//! を渡さない構成では docs サイトが JS ハイドレーションを行わなくても
//! ラベルクリックでブラウザが `checked` をネイティブに切り替えてしまう。
//! 一方 `control`/`indicator` の見た目（`data-state`）は SSR 時の `checked`
//! 引数から固定生成されるため追従せず、静的な初期状態のみという block
//! 全体の設計方針に反する（`form_layout_stacked.rs` と同型の判断）。
//! 全 checkbox へ `disabled: true` を共有し、ネイティブ `disabled` 属性で
//! フォーカス・操作を不能にして状態が二度と変化しないことを構造的に保証
//! する。`disabled_declarations()`（既定 `opacity: 0.5` + `cursor:
//! not-allowed`）は [`LAYOUT_CSS`] で中和し、通常の checkbox と同じ見た目
//! に保つ。
//!
//! # 「先頭の未完了タスクだけ展開」を部品の意味論で表現
//!
//! `Steps::new(TASKS.len(), DONE, Orientation::Vertical)` の `step` を
//! 最初の未完了タスクの index（[`DONE`]）に固定する。`steps::content` は
//! current な index（`== step`）のときのみ `data-state="open"` で表示し、
//! それ以外は `data-state="closed"` + `hidden` 属性で隠す（`steps.rs` の
//! 既定契約）。完了タスク（`index < DONE`）には `content` 自体を持たせ
//! ない（説明・実行ボタンは未完了タスクのみが持つため）。
//!
//! # `class` と `data-*` の使い分け
//!
//! `progress::root`/`steps::root`/`checkbox::root`/`button::button` は
//! いずれも `drop_class_attr` で呼び出し側 `class` を除去してから内部
//! variant クラスと合成するため、これらへの CSS フックは `data-*` 属性で
//! 渡す（`data-blocks-onboarding-checklist-*`）。レイアウト用ラッパー
//! （見出し行・タスク本文行）は素の `<div>` のため
//! `class="blocks-onboarding-checklist-*"` を使う
//! （`profile_detail_datalist` と同型の判断）。
//!
//! # 狭い幅では実行ボタンを説明の下へ回す（`@container`）
//!
//! Demo 枠の幅はビューポート幅と一致しないため、`@container`
//! （コンテナクエリ）で判定する（`profile_detail_datalist`/
//! `empty_state_setup_steps` と同型のパターン）。[`LAYOUT_CSS`] のラッパー
//! `.blocks-onboarding-checklist-stack` へ `container-type: inline-size`
//! を宣言し、コンテナ幅が `32rem` 未満のとき
//! `.blocks-onboarding-checklist-task-body` を列方向へ切り替える。
//!
//! # 実行ボタンのアクセシブルネームを区別する（`visually-hidden`）
//!
//! 各実行ボタンの可視テキストはすべて「開始する」で同一のため、ボタン内へ
//! `visually_hidden::root` でタスク名を不可視テキストとして補い、支援
//! 技術がボタンを個別に区別できるようにする。
//!
//! # `<form>` を使わない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない静的表示のみで、送信処理・送信先は一切持たない。実行ボタンは
//! `button::button` の既定 `type="button"` のまま用いる。
//!
//! # ダミー素材について
//!
//! タスク名・説明はすべて架空の文言であり、実在の人物・企業・PII は含まない。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
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
                    // 静的デモ（無 JS）では押しても何も起きないため、checkbox
                    // と同様にネイティブ `disabled` で操作不能であることを
                    // 構造的に表現する（Codex #2979 指摘の是正。`steps::trigger`
                    // を避けた理由と同型）。checkbox と異なり見た目の減衰
                    // （既定 `disabled_declarations()`）はあえて中和しない。
                    disabled: true,
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
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/onboarding-checklist/",
    title: "onboarding-checklist",
    category: BlockCategory::Onboarding,
    rust_source: "crates/docs-site/src/blocks/application/onboarding/onboarding_checklist.rs",
    demo_class: "blocks-onboarding-checklist",
    parts: &[
        Part {
            label: "Progress",
            path: "/themes/progress/",
        },
        Part {
            label: "Steps",
            path: "/themes/steps/",
        },
        Part {
            label: "Checkbox",
            path: "/themes/checkbox/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
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
            label: "Visually Hidden",
            path: "/themes/visually-hidden/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `onboarding_checklist` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節と同型）。
///
/// - vertical な `steps::root` の既定（list 左・content 右の横並び、
///   `steps.rs` モジュール doc「`body`」節参照）を、詳細度 0,4,0 の
///   追加属性セレクタで通常の縦積みへ打ち消す（モジュール冒頭
///   「先頭の未完了タスクだけ展開」節参照）。
/// - 完了タスクの取り消し線・checkbox の disabled 中和は
///   モジュール冒頭「checkbox をネイティブ `disabled` にする理由」節参照。
///   取り消し線セレクタは本 block の `[data-blocks-onboarding-checklist-steps]`
///   祖先スコープ必須（`assets/blocks.css` が全 block ページで読み込まれる
///   ため、祖先スコープを欠くと他ページの完了 `steps` アイテムへ漏れて
///   適用される。Cursor Bugbot #2979 指摘の是正）。
/// - `.blocks-onboarding-checklist-task-body` の `justify-content:
///   space-between` は `align-self: stretch` を伴わせる（親 `steps::item` が
///   `align-items: flex-start`〔column 軸〕のため、指定しないとコンテンツ
///   ボックスが shrink-wrap して余白が生まれず space-between が効かない。
///   Cursor Bugbot #2979 指摘の是正）。
const LAYOUT_CSS: &str = "\
.blocks-onboarding-checklist-stack {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n  container-type: inline-size;\n  container-name: blocks-onboarding-checklist;\n}\n\
.blocks-onboarding-checklist-header {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n}\n\
[data-scope=\"steps\"][data-part=\"root\"][data-orientation=\"vertical\"][data-blocks-onboarding-checklist-steps] {\n  flex-direction: column;\n}\n\
[data-blocks-onboarding-checklist-steps] [data-scope=\"steps\"][data-part=\"item\"] {\n  border-top: 1px solid var(--fandhe-color-border);\n  padding-block: var(--fandhe-space-3);\n}\n\
[data-blocks-onboarding-checklist-steps] [data-scope=\"steps\"][data-part=\"item\"][data-complete] [data-scope=\"checkbox\"][data-part=\"label\"] {\n  text-decoration: line-through;\n  color: var(--fandhe-color-fg-muted);\n}\n\
[data-scope=\"checkbox\"][data-part=\"root\"][data-blocks-onboarding-checklist-task][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
.blocks-onboarding-checklist-task-body {\n  display: flex;\n  align-items: flex-start;\n  justify-content: space-between;\n  align-self: stretch;\n  gap: var(--fandhe-space-4);\n  padding-inline-start: calc(var(--fandhe-steps-indicator-size, 2rem) + var(--fandhe-space-2));\n}\n\
@container blocks-onboarding-checklist (max-width: 32rem) {\n  \
.blocks-onboarding-checklist-task-body {\n    flex-direction: column;\n  }\n\
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
            "data-scope=\"progress\"",
            "data-scope=\"steps\"",
            "data-scope=\"checkbox\"",
            "data-scope=\"button\"",
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
        assert_eq!(html.matches("<button").count(), 3);
        assert_eq!(
            html.matches("data-scope=\"checkbox\" data-part=\"control\" data-state=\"checked\"")
                .count(),
            2
        );
        assert_eq!(
            html.matches("data-scope=\"steps\" data-part=\"content\" data-state=\"open\"")
                .count(),
            1
        );
        assert_eq!(
            html.matches("data-scope=\"steps\" data-part=\"content\" data-state=\"closed\"")
                .count(),
            2
        );
        assert_eq!(
            html.matches("data-scope=\"steps\" data-part=\"item\"")
                .count(),
            5
        );
        assert_eq!(html.matches("data-complete").count(), 2);
    }

    #[test]
    fn no_form_submit_or_dead_links() {
        let html = demo_html();
        assert!(!html.contains("<form"));
        assert!(!html.contains("type=\"submit\""));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("<script"));
        assert!(!html.contains("src=\"data:"));
    }

    /// 静的デモの実行ボタンは無 JS では押しても何も起きないため、
    /// ネイティブ `disabled` で操作不能であることを構造的に表現する
    /// （Codex #2979 指摘の是正）。全 3 個の実行ボタンが対象。
    #[test]
    fn start_buttons_are_disabled() {
        let html = demo_html();
        let button_tags: Vec<&str> = html
            .split("<button data-scope=\"button\"")
            .skip(1)
            .collect();
        assert_eq!(button_tags.len(), 3, "expected 3 start buttons");
        for tag in button_tags {
            let end = tag.find('>').expect("button tag must close");
            let open_tag = &tag[..end];
            assert!(
                open_tag.contains(r#"disabled="""#) && open_tag.contains(r#"aria-disabled="true""#),
                "start button must be natively disabled: {open_tag}"
            );
        }
    }

    #[test]
    fn layout_css_is_safe_and_stacks_on_narrow_container() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("container-type: inline-size;"));
        assert!(LAYOUT_CSS.contains("@container blocks-onboarding-checklist (max-width: 32rem)"));
    }

    #[test]
    fn progress_has_accessible_name_and_value_text() {
        let html = demo_html();
        assert!(html.contains("role=\"progressbar\""));
        assert!(html.contains("aria-label=\"設定の達成率\""));
        assert!(html.contains("aria-valuenow=\"2\""));
        assert!(html.contains("aria-valuemax=\"5\""));
        assert!(html.contains("aria-valuetext="));
    }

    #[test]
    fn start_buttons_are_distinguishable() {
        let html = demo_html();
        assert_eq!(html.matches("data-scope=\"visually-hidden\"").count(), 3);
        for title in [
            "最初のプロジェクトを作成する",
            "通知先を接続する",
            "モバイルアプリをインストールする",
        ] {
            assert!(
                html.contains(title),
                "missing visually-hidden task name: {title}"
            );
        }
    }
}
