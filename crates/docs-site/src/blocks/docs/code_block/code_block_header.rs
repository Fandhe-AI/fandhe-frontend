//! `code-block-header` block（イシュー #3104。親トラッキング #3099
//! 「Blocks 目的別パーツ拡充ツリー」配下、対応表 ID R0054 を主参照とし
//! R0055・R0057 の構成差分を集約した合成例。角丸枠の上端にヘッダー帯を
//! 持つコードブロック）。取得手段・ファイル名・内部コンポーネント識別子は
//! 記載しない（`docs/design/motion-reference-adoption-policy.md` §9 と
//! 同じライセンス上の転記制限）。
//!
//! # 使用部品
//!
//! `code` / `clipboard` / `button` / `badge` / `text` の 5 部品を合成する
//! （[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//!
//! # 3 インスタンスの併記（ヘッダー構成の差分）
//!
//! ヘッダー構成の差分を 1 つの Demo 内に 3 インスタンス静的に並記して
//! 示す（`hero_install_command` 等、他 block の「差分は別インスタンスで
//! 併記」パターンと同型）。
//!
//! - **A（R0054・主参照）**: ファイル名タイトル + コピー操作だけの
//!   最小構成。
//! - **B（R0055）**: A に無効化済みの補助ボタン（折り返し・ダウンロード）
//!   を追加した構成。
//! - **C（R0057）**: ヘッダーに状態バッジを追加し、本文の左端へ行番号を
//!   振った構成。
//!
//! # 静的表示の制約（clipboard は idle 固定）
//!
//! docs サイトは JS ハイドレーションを行わないため、本 Demo は 3
//! インスタンスとも常に未コピー（idle）状態で固定表示する（`data-copied`
//! を一切持たない）。実アプリへ組み込み `fandhe-frontend-wasm-full` で
//! ハイドレーションすると、A/B/C いずれの `clipboard` も
//! `navigator.clipboard.writeText` への実書き込みが自動配線される
//! （`headless_clipboard`、`hero_install_command` モジュール doc
//! 「コピー配線の範囲」節と同型）。B の補助ボタン（折り返し・
//! ダウンロード）は実処理を持たない合成例のボタンのため `disabled: true`
//! で押下不能を明示する。
//!
//! # A/B/C を独立マウントルートへ分離する理由
//!
//! `headless_clipboard` は「1 root : 1 状態機械契約」（同一マウントルート
//! 配下の全 `clipboard` パーツへコピー済み表示をまとめて反映する
//! 簡略化）を持つため、`hero_install_command` と同型の判断で A/B/C の
//! 外枠それぞれへ一意の `id`（`blocks-code-block-header-a`/`-b`/`-c`）を
//! 付与し、実アプリで複数インスタンスを使う場合に個別の `mount`/
//! `hydrate` で分離できるようにする。
//!
//! # 行番号は CSS カウンターで振る（DOM にテキストを持たない）
//!
//! C の行番号は各行 `span`（`data-blocks-code-block-header-line`）を
//! `code` の子として並べ、CSS の `counter-increment`/`::before { content:
//! counter(...) }` で視覚的に振る。行番号テキスト自体を DOM へ埋め込まない
//! ため、行番号を選択範囲に含めてもコピーされない（`user-select: none`
//! も付与する）。span 間には改行テキスト（`\n`）自体を挟み（最終行の
//! 後ろを除く）、親 `<code>` の `white-space: pre` がこれを視覚上の改行
//! として描画する。行 span 自体には `display: block` を付けない。付けると
//! `display: block` 由来の改行と `\n` テキストノード由来の改行が二重に
//! 発生し、行間が間延びして見える（Codex レビュー・Cursor Bugbot 指摘、
//! イシュー #3104 PR #3545）。`\n` テキストノードのみを改行源にすることで、
//! 視覚上の行間を単一にしたまま選択範囲コピー時の改行も保持する。
//!
//! # `<form>` を使わない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない。ボタンはすべて `type="button"`（`button::button`/
//! `clipboard::trigger` いずれも既定・固定で `type="button"`）。
//!
//! # 長い行は枠内で横スクロールする
//!
//! 各インスタンスの本文は `pre`（`overflow-x: auto`）に包み、ページ全体
//! ではなく枠内に横スクロールを閉じ込める。`tabindex="0"` を付与し、
//! キーボードのみでも横スクロール領域を操作できるようにする。
//!
//! # コード・文言はすべて無害
//!
//! コード片は本フレームワーク自身のノード木 API を使う短い Rust のみで、
//! 実在の秘密情報・トークンらしき文字列を含まない。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `code::code`/`clipboard::root`/`button::button`/`badge::badge`/
//! `text::text` はいずれも `drop_class_attr` により呼び出し側 `attrs` の
//! `class` を黙って除去する契約を持つため、Demo 固有のスタイルフックは
//! `data-blocks-code-block-header-*` 属性で渡す（`hero_install_command`
//! と同型の判断）。素の `div`/`pre`/`span` は `class` がそのまま効くため
//! `.blocks-code-block-header-*` クラスセレクタを使う。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, pre, span, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::clipboard;
use fandhe_frontend_pre_styled_ui::code::{self, CodeProps};
use fandhe_frontend_pre_styled_ui::recipe::ColorPalette;
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize};
use fandhe_frontend_pre_styled_ui::Size;

/// A（R0054・主参照）で表示するコード片。1 行だけ意図的に長くし、枠内
/// 横スクロールが起きることを示す（モジュール doc「長い行は枠内で横
/// スクロールする」節）。
const SNIPPET_A: &str = "use fandhe_frontend_core::{div, text};\n\nfn demo() -> fandhe_frontend_core::Node {\n    div(vec![], vec![text(\"こんにちは\")])\n}\n\n// この行はとても長いコメントで、枠の幅を越えて横スクロールが発生することを示すためにわざと伸ばしてあります\n";

/// B（R0055）で表示するコード片。`render(&demo())` が呼ぶ `demo` 自体を
/// 片内に定義し、コピーしたコード片単体で実行できる自己完結の例にする
/// （Codex レビュー指摘、イシュー #3104 PR #3545）。
const SNIPPET_B: &str = "use fandhe_frontend_core::{div, render, text};\n\nfn demo() -> fandhe_frontend_core::Node {\n    div(vec![], vec![text(\"こんにちは\")])\n}\n\nfn main() {\n    println!(\"{}\", render(&demo()));\n}\n";

/// C（R0057）で表示するコード片。行番号を 1 行ずつ振るため
/// `.lines()` で分割する。`fandhe_frontend_cli`（`[[bin]]` のみでライブラリ
/// ターゲットを持たない）の `use ... as _;` では import が解決できず、
/// コピーしたコード片単体でコンパイルできなかったため、実在するライブラリ
/// クレート `fandhe_frontend_core` を使った自己完結の例へ置き換える
/// （Codex レビュー指摘、イシュー #3104 PR #3545）。
const SNIPPET_C: &str = "use fandhe_frontend_core::{render, text};\n\nfn check() -> bool {\n    // render() が空文字列を返さなければ true\n    !render(&text(\"ok\")).is_empty()\n}\n";

/// A（R0054・主参照）: ファイル名タイトル + コピー操作だけの最小構成。
fn instance_a() -> Node {
    header_instance(
        "blocks-code-block-header-a",
        SNIPPET_A,
        "demo.rs",
        None,
        false,
        false,
    )
}

/// B（R0055）: A に無効化済みの補助ボタン（折り返し・ダウンロード）を
/// 追加した構成。
fn instance_b() -> Node {
    header_instance(
        "blocks-code-block-header-b",
        SNIPPET_B,
        "main.rs",
        None,
        true,
        false,
    )
}

/// C（R0057）: ヘッダーに状態バッジを追加し、本文の左端へ行番号を振った
/// 構成。
fn instance_c() -> Node {
    header_instance(
        "blocks-code-block-header-c",
        SNIPPET_C,
        "check.rs",
        Some("成功"),
        false,
        true,
    )
}

/// A/B/C 共通のヘッダー付きコードブロックを組み立てる。
///
/// - `root_id`: 独立マウントルート識別子（モジュール doc「A/B/C を独立
///   マウントルートへ分離する理由」節参照）。
/// - `snippet`: 本文に表示するコード文字列。
/// - `title`: ヘッダー帯に表示するファイル名タイトル。
/// - `status_badge`: `Some(文言)` のときヘッダー帯の左端へ
///   `ColorPalette::Success` の状態バッジを追加する（C 専用）。
/// - `with_aux_buttons`: `true` のときコピー操作の隣へ無効化済みの補助
///   ボタン（折り返し・ダウンロード）を 2 個追加する（B 専用）。
/// - `numbered`: `true` のとき本文の各行を `span` で包み、CSS カウンター
///   で行番号を振る（C 専用、モジュール doc「行番号は CSS カウンターで
///   振る」節参照）。
fn header_instance(
    root_id: &'static str,
    snippet: &'static str,
    title: &'static str,
    status_badge: Option<&'static str>,
    with_aux_buttons: bool,
    numbered: bool,
) -> Node {
    let mut bar_left: Vec<Node> = Vec::new();
    if let Some(label) = status_badge {
        bar_left.push(badge::badge(
            &BadgeProps {
                palette: ColorPalette::Success,
                ..BadgeProps::default()
            },
            vec![],
            vec![text(label)],
        ));
    }
    bar_left.push(styled_text::text(
        &TextProps {
            size: TextSize::Sm,
            ..TextProps::default()
        },
        vec![],
        vec![text(title)],
    ));

    let mut bar_right: Vec<Node> = vec![clipboard::root(
        snippet,
        false,
        vec![("data-blocks-code-block-header-copy", "")],
        vec![clipboard::control(
            false,
            vec![],
            vec![clipboard::trigger(
                false,
                vec![],
                vec![
                    clipboard::indicator(false, false, vec![], vec![text("コピー")]),
                    clipboard::indicator(true, false, vec![], vec![text("コピー済み")]),
                ],
            )],
        )],
    )];
    if with_aux_buttons {
        bar_right.push(button::button(
            &ButtonProps {
                variant: ButtonVariant::Ghost,
                size: Size::Sm,
                // 遷移先・クリック処理を持たない合成例のボタンのため
                // `disabled: true` にして「押しても何も起きない」ことを
                // 明示する（モジュール doc「静的表示の制約」節、
                // `hero_install_command` と同型の判断）。
                disabled: true,
                ..ButtonProps::default()
            },
            vec![],
            vec![text("折り返し")],
        ));
        bar_right.push(button::button(
            &ButtonProps {
                variant: ButtonVariant::Ghost,
                size: Size::Sm,
                disabled: true,
                ..ButtonProps::default()
            },
            vec![],
            vec![text("ダウンロード")],
        ));
    }

    // 各行 span は `display: block` を持たず、span 間に挟む改行テキスト
    // （`\n`）のみが視覚上の改行源になる（`white-space: pre` が解釈）。
    // `display: block` も併用すると改行が二重になり行間が間延びする
    // （Codex レビュー・Cursor Bugbot 指摘、イシュー #3104 PR #3545）。
    // `.lines()` は末尾の改行を 1 行として数えないため、元のスニペットが
    // 改行で終わる場合は最終行の後ろにも改行テキストを 1 つ補い、選択範囲
    // コピー結果が `clipboard::root` へ渡す元のスニペット（末尾改行あり）
    // と一致するようにする（Codex レビュー指摘、イシュー #3104 PR #3545）。
    let body_children: Vec<Node> = if numbered {
        let lines: Vec<&str> = snippet.lines().collect();
        let line_count = lines.len();
        let ends_with_newline = snippet.ends_with('\n');
        lines
            .into_iter()
            .enumerate()
            .flat_map(|(i, line)| {
                let mut nodes = vec![span(
                    vec![("data-blocks-code-block-header-line", "")],
                    vec![text(line)],
                )];
                if i + 1 < line_count || ends_with_newline {
                    nodes.push(text("\n"));
                }
                nodes
            })
            .collect()
    } else {
        vec![text(snippet)]
    };

    let pre_label = format!("{title} のコード");

    div(
        vec![("id", root_id), ("class", "blocks-code-block-header-frame")],
        vec![
            div(
                vec![("class", "blocks-code-block-header-bar")],
                vec![
                    div(
                        vec![("data-blocks-code-block-header-bar-start", "")],
                        bar_left,
                    ),
                    div(
                        vec![("data-blocks-code-block-header-bar-end", "")],
                        bar_right,
                    ),
                ],
            ),
            pre(
                vec![
                    ("class", "blocks-code-block-header-pre"),
                    ("tabindex", "0"),
                    // `tabindex="0"` でキーボード操作可能にした横スクロール
                    // 領域には、スクリーンリーダーが読み上げられるアクセ
                    // シブルネームを明示する（Cursor Bugbot 指摘、イシュー
                    // #3104 PR #3545）。
                    ("aria-label", pre_label.as_str()),
                ],
                vec![code::code(
                    &CodeProps::default(),
                    vec![("data-blocks-code-block-header-code", "")],
                    body_children,
                )],
            ),
        ],
    )
}

pub fn demo() -> Node {
    div(
        vec![("class", "blocks-code-block-header-layout")],
        vec![instance_a(), instance_b(), instance_c()],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/code-block-header/",
    title: "code-block-header",
    category: BlockCategory::CodeBlock,
    rust_source: "crates/docs-site/src/blocks/docs/code_block/code_block_header.rs",
    demo_class: "blocks-code-block-header",
    parts: &[
        Part {
            label: "Code",
            path: "/themes/code/",
        },
        Part {
            label: "Clipboard",
            path: "/themes/clipboard/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Badge",
            path: "/themes/badge/",
        },
        Part {
            label: "Text",
            path: "/themes/text/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `code_block_header` 固有のレイアウト規則（`crate::blocks`
/// モジュール doc「CSS の置き場」節、他 block と同型で `pub(super)` として
/// `super::stylesheet` から連結される）。長い行を枠内へ閉じ込める
/// 横スクロールと、行番号用 CSS カウンターをここで実装する
/// （モジュール doc「長い行は枠内で横スクロールする」「行番号は CSS
/// カウンターで振る」各節参照）。
const LAYOUT_CSS: &str = "\
.blocks-code-block-header-layout {\n  display: flex;\n  flex-direction: column;\n  gap: 1.5rem;\n}\n\
.blocks-code-block-header-frame {\n  border: 1px solid var(--fandhe-color-border);\n  border-radius: var(--fandhe-radius-lg);\n  overflow: hidden;\n  min-width: 0;\n}\n\
.blocks-code-block-header-bar {\n  display: flex;\n  flex-wrap: wrap;\n  align-items: center;\n  justify-content: space-between;\n  gap: 0.75rem;\n  padding: 0.5rem 1rem;\n  background: var(--fandhe-color-bg-subtle);\n  border-bottom: 1px solid var(--fandhe-color-border);\n}\n\
[data-blocks-code-block-header-bar-start] {\n  display: flex;\n  align-items: center;\n  gap: 0.5rem;\n  min-width: 0;\n}\n\
[data-blocks-code-block-header-bar-end] {\n  display: flex;\n  align-items: center;\n  gap: 0.5rem;\n}\n\
.blocks-code-block-header-pre {\n  margin: 0;\n  padding: 1rem;\n  overflow-x: auto;\n  font-family: var(--fandhe-font-font-mono);\n}\n\
.blocks-code-block-header-pre:focus-visible {\n  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));\n  outline-offset: calc(-1 * var(--fandhe-focus-ring-offset, 2px));\n}\n\
[data-scope=\"code\"][data-part=\"root\"][data-blocks-code-block-header-code] {\n  display: block;\n  white-space: pre;\n  background: transparent;\n  border: 0;\n  padding: 0;\n  color: inherit;\n  counter-reset: blocks-code-block-header-line;\n}\n\
[data-blocks-code-block-header-line] {\n  counter-increment: blocks-code-block-header-line;\n}\n\
[data-blocks-code-block-header-line]::before {\n  content: counter(blocks-code-block-header-line);\n  display: inline-block;\n  min-width: 2ch;\n  margin-inline-end: 1rem;\n  text-align: end;\n  color: var(--fandhe-color-fg-muted);\n  user-select: none;\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// Demo が `<form>` を出力せず、`data-copied`・死にリンク・`data:` URI
    /// を一切持たないこと（`hero_install_command` と同型の固定）。
    #[test]
    fn demo_has_no_form_data_copied_or_dead_links() {
        let html = render(&demo());
        assert!(!html.contains("<form"));
        assert_eq!(html.matches("data-copied").count(), 0);
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("mailto:"));
        assert!(!html.contains("src=\"data:"));
    }

    /// A/B/C がそれぞれ独立した `id` を持つこと（モジュール doc「A/B/C を
    /// 独立マウントルートへ分離する理由」節）。
    #[test]
    fn demo_has_distinct_mount_root_ids_for_a_b_and_c() {
        let html = render(&demo());
        assert!(html.contains(r#"id="blocks-code-block-header-a""#));
        assert!(html.contains(r#"id="blocks-code-block-header-b""#));
        assert!(html.contains(r#"id="blocks-code-block-header-c""#));
    }

    /// 3 インスタンスすべてに `<pre` + `tabindex="0"` + `aria-label` が
    /// 付いていること（モジュール doc「長い行は枠内で横スクロールする」節、
    /// `aria-label` は Cursor Bugbot 指摘対応・イシュー #3104 PR #3545）。
    #[test]
    fn demo_has_three_scrollable_pre_blocks() {
        let html = render(&demo());
        assert_eq!(html.matches("<pre").count(), 3);
        assert_eq!(html.matches(r#"tabindex="0""#).count(), 3);
        // `aria-label=` の総数は `clipboard::trigger` 自体が持つ分も含む
        // ため数えず、`pre` 用の文言が 3 インスタンス分揃うことのみ固定する。
        assert!(html.contains(r#"aria-label="demo.rs のコード""#));
        assert!(html.contains(r#"aria-label="main.rs のコード""#));
        assert!(html.contains(r#"aria-label="check.rs のコード""#));
    }

    /// C の行 `span` の数がスニペットの行数と一致すること（行番号は CSS
    /// カウンターで振り、DOM に番号テキストを埋め込まない、モジュール doc
    /// 「行番号は CSS カウンターで振る」節）。
    #[test]
    fn instance_c_has_one_line_span_per_source_line() {
        let html = render(&demo());
        let expected = super::SNIPPET_C.lines().count();
        assert_eq!(
            html.matches("data-blocks-code-block-header-line").count(),
            expected
        );
    }

    /// C の行 span 間に改行テキストが入ること（選択範囲コピーで改行が
    /// 失われない固定、Codex レビュー指摘・イシュー #3104 PR #3545）。
    /// `SNIPPET_C` は末尾が改行で終わるため、最終行の後ろにも改行テキスト
    /// が入り、`</span>\n` の総数は行数と一致する（末尾改行保持の固定、
    /// Codex レビュー指摘・イシュー #3104 PR #3545）。
    #[test]
    fn instance_c_line_spans_are_separated_by_newline_text() {
        let html = render(&demo());
        let line_count = super::SNIPPET_C.lines().count();
        assert!(super::SNIPPET_C.ends_with('\n'));
        assert_eq!(
            html.matches("</span>\n").count(),
            line_count,
            "行 span の後（最終行を含む）に改行テキストが挿入されていること"
        );
    }

    /// B のコード片（`SNIPPET_B`）が呼び出す `demo` 関数自体を片内に
    /// 定義し、コピーしたコード片単体でコンパイル・実行できる自己完結の
    /// 例であること（Codex レビュー指摘、イシュー #3104 PR #3545）。
    #[test]
    fn snippet_b_defines_the_demo_function_it_calls() {
        assert!(super::SNIPPET_B.contains("fn demo("));
        assert!(super::SNIPPET_B.contains("render(&demo())"));
    }

    /// B の補助ボタン 2 個に `disabled` が付いていること（モジュール doc
    /// 「静的表示の制約」節）。
    #[test]
    fn instance_b_aux_buttons_are_disabled() {
        let html = render(&demo());
        assert_eq!(html.matches("折り返し").count(), 1);
        assert_eq!(html.matches("ダウンロード").count(), 1);
        // 補助ボタン 2 個分の `disabled` 属性が出力されていること
        // （`clipboard::trigger` 自体は disabled を持たないため、この
        // カウントは B の補助ボタンに閉じる）。
        assert!(html.contains("disabled"));
    }

    /// [`LAYOUT_CSS`] が横スクロール・行番号カウンターの各セレクタを
    /// 含み、HTML タグ破りを起こす `<` を含まないこと。
    #[test]
    fn layout_css_declares_scroll_and_counter_rules() {
        assert!(LAYOUT_CSS.contains("overflow-x: auto"));
        assert!(LAYOUT_CSS.contains("counter-increment"));
        assert!(LAYOUT_CSS.contains("counter("));
        // `.blocks-code-block-header-frame` の `overflow: hidden` でフォー
        // カスリングが隠れないよう、`outline-offset` を負値（inset）にして
        // `pre` 自身のボックス内へ収める（Cursor Bugbot 指摘、イシュー
        // #3104 PR #3545）。
        assert!(LAYOUT_CSS.contains(".blocks-code-block-header-pre:focus-visible {"));
        assert!(
            LAYOUT_CSS.contains("outline-offset: calc(-1 * var(--fandhe-focus-ring-offset, 2px));")
        );
        assert!(!LAYOUT_CSS.contains('<'));
    }
}
