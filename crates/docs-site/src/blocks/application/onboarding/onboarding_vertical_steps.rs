//! `onboarding-vertical-steps` block（イシュー #2981。親トラッキング #2731
//! 「Blocks 目的別パーツ拡充」配下、Application / Onboarding カテゴリの
//! 最初の block）。左に縦向きのステップ一覧、右に現在ステップの内容
//! （動画枠 + 見出し + 説明文 + 前へ/次へ操作）を並べる 2 カラム構成を
//! 合成する。`_/blocks-intake/` の対応ファイルは本イシュー着手時点で本
//! worktree に存在しないため、原稿・本コメントには対応表 ID（R0178）の
//! みを記す（`help_center_article_list.rs`/`profile_detail_datalist.rs` と
//! 同じ扱い）。
//!
//! # 使用部品
//!
//! `steps` / `button` / `image` / `heading` / `text` の 5 部品のみを
//! 合成する（[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//! 新しい UI 部品は追加しない。
//!
//! # 前へ/次へを右カラム（`body` 側）に置く理由
//!
//! [`fandhe_frontend_pre_styled_ui::steps::root`] は縦向き
//! （`Orientation::Vertical`）で `root` 自体が `flex-direction: row` に
//! 切り替わり、`list`（左）と `body`（右）の 2 要素のみを直下に置く契約を
//! 持つ（`root`/`body` の rustdoc、PR #1814 codex-review 対応）。`root` は
//! 自身の属性しか条件化できず祖先・子孫関係を見られないため、`list` 以外を
//! 直接 `root` の子として複数並べると全て横並びに崩れる（過去の同型回帰は
//! PR #1814 で是正済み）。参照構成（R0178）は前へ/次へをステップ一覧側
//! （左）に描くが、本 block は既存部品の契約を優先し、前へ/次へ
//! （[`fandhe_frontend_pre_styled_ui::steps::prev_trigger`]/
//! [`next_trigger`]）を [`fandhe_frontend_pre_styled_ui::steps::body`]
//! （右カラム、内容の下）へ配置する。この配置差分は原稿「原案差分メモ」
//! 節にも明記する。
//!
//! # 状態は固定（無 JS docs サイトの静的表示）
//!
//! `Steps::new(4, 2, Orientation::Vertical)` で固定する: step 0・1 が
//! 完了（`data-state="complete"`）、step 2 が現在（`aria-current="step"`）、
//! step 3 が未着手。開いている step 2 の [`fandhe_frontend_pre_styled_ui::steps::content`]
//! のみを描画し（閉じた content は `hidden` が付くだけの無駄な出力になる
//! ため描画しない、`showcase::steps_demo` と同型の判断）、状態遷移・
//! クリック配線は行わない。
//!
//! # 動画は 16:9 の静止画枠 + 再生ボタンで代替する
//!
//! 動画ソース資産を持たず、無 JS の docs サイトへ `<video>` の再生 UI を
//! 持ち込まない設計方針（`crate::blocks` モジュール doc の不変条件）に
//! 従い、[`fandhe_frontend_pre_styled_ui::image::image`]（`AspectRatio::Video`
//! の 16:9 枠）を「手順動画のプレビュー（静止画）」として提示し、直下に
//! 装飾的な「動画を再生」ボタン（`button::button`、`ButtonVariant::Outline`、
//! `type="button"` 既定）を置く。実際の再生処理・送信処理は持たない。
//!
//! # `class` と `data-*` の使い分け
//!
//! [`fandhe_frontend_pre_styled_ui::steps::root`]・
//! [`fandhe_frontend_pre_styled_ui::button::button`]・
//! [`fandhe_frontend_pre_styled_ui::image::image`]・
//! [`fandhe_frontend_pre_styled_ui::heading::heading`]・
//! [`fandhe_frontend_pre_styled_ui::text::text`] はいずれも
//! `drop_class_attr` で呼び出し側 `class` を除去してから内部 variant
//! クラスと合成するため、これらへの CSS フックは `data-*` 属性で渡す
//! （`data-blocks-onboarding-vertical-steps-*`）。素の `<div>` ラッパーは
//! `class="blocks-onboarding-vertical-steps-*"` を使う
//! （`help_center_article_list` と同型の判断）。
//!
//! # 狭幅では一覧を内容の上に横並びで置く（`@container`）
//!
//! Demo 枠の幅はビューポート幅と一致しないため `@container`（コンテナ
//! クエリ）で判定する（`profile_detail_datalist`/`help_center_article_list`
//! と同型のパターン）。最外ラッパー
//! `.blocks-onboarding-vertical-steps-stack` へ `container-type:
//! inline-size` を宣言し、コンテナ幅が `40rem` 未満のとき
//! `[data-scope="steps"][data-part="root"][data-orientation="vertical"]` を
//! `flex-direction: column` に、`list` を `flex-direction: row; flex-wrap:
//! wrap` に、`separator` を `display: none` に切り替える。
//!
//! # ダミー素材について
//!
//! ステップの題名・見出し・説明文はすべて独自の架空ダミーであり、実企業名・
//! 実クレデンシャル・PII は含まない。画像は
//! [`crate::blocks::dummy_assets::SCREENSHOT_SRC`]（ビルド時生成の同梱 SVG。
//! 外部 URL・`data:` URI は使わない）を使う。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, text as core_text, Node};
use fandhe_frontend_pre_styled_ui::button::{button, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::steps::Steps;
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps};
use fandhe_frontend_pre_styled_ui::image::{image, AspectRatio, ImageFit, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::recipe::{ColorPalette, Size};
use fandhe_frontend_pre_styled_ui::steps;
use fandhe_frontend_pre_styled_ui::text::{text as styled_text, TextProps};
use fandhe_frontend_pre_styled_ui::Orientation;

/// ステップ 4 件分の題名（[`step_list`] の trigger ラベルに使う架空
/// ダミー）。
const STEP_TITLES: [&str; 4] = [
    "ワークスペースを作成",
    "メンバーを招待",
    "最初のプロジェクトを設定",
    "通知を確認",
];

/// 現在ステップ（step 2）の見出し。[`STEP_TITLES`][2] と対応させる。
const CURRENT_STEP_HEADING: &str = "最初のプロジェクトを設定";

/// 現在ステップ（step 2）の説明文。
const CURRENT_STEP_DESCRIPTION: &str =
    "テンプレートを選び、チームで使う最初のプロジェクトを数分で立ち上げます。\
     動画の手順に沿って進めると、設定は自動的に保存されます。";

/// 縦向きステップ一覧（左カラム）。`showcase::steps_demo` と同型に
/// `item` → `trigger`（`indicator` に番号 + 題名 `text`）+ 末尾以外に
/// `separator` を並べる。
fn step_list(s: &Steps) -> Node {
    let mut items = Vec::new();
    for (index, title) in STEP_TITLES.iter().enumerate() {
        let mut item_children = vec![steps::trigger(
            s,
            index,
            vec![],
            vec![
                steps::indicator(s, index, vec![], vec![core_text((index + 1).to_string())]),
                core_text(*title),
            ],
        )];
        if index + 1 < STEP_TITLES.len() {
            item_children.push(steps::separator(s, index, vec![], vec![]));
        }
        items.push(steps::item(s, index, vec![], item_children));
    }
    steps::list(s, vec![], items)
}

/// 動画枠（16:9 の静止画プレビュー + 装飾的な再生ボタン、モジュール doc
/// 「動画は 16:9 の静止画枠 + 再生ボタンで代替する」節参照）。
fn media_frame() -> Node {
    div(
        vec![("class", "blocks-onboarding-vertical-steps-media")],
        vec![
            image(
                &ImageProps {
                    src: dummy_assets::SCREENSHOT_SRC,
                    alt: "手順動画のプレビュー（静止画）",
                    fit: ImageFit::Cover,
                    aspect_ratio: AspectRatio::Video,
                    shape: ImageShape::Rounded,
                },
                vec![("data-blocks-onboarding-vertical-steps-video", "")],
            ),
            button(
                &ButtonProps {
                    variant: ButtonVariant::Outline,
                    ..ButtonProps::default()
                },
                vec![("data-blocks-onboarding-vertical-steps-play", "")],
                vec![core_text("動画を再生")],
            ),
        ],
    )
}

/// 現在ステップ（step 2）の内容（動画枠 + 見出し + 説明文）。
fn current_content(s: &Steps) -> Node {
    steps::content(
        s,
        2,
        vec![],
        vec![
            media_frame(),
            heading(
                HeadingLevel::H3,
                &HeadingProps::default(),
                vec![],
                vec![core_text(CURRENT_STEP_HEADING)],
            ),
            styled_text(
                &TextProps::default(),
                vec![],
                vec![core_text(CURRENT_STEP_DESCRIPTION)],
            ),
        ],
    )
}

/// 前へ・次へ（右カラム下部、モジュール doc「前へ/次へを右カラムに置く
/// 理由」節参照）。
fn nav(s: &Steps) -> Node {
    div(
        vec![("class", "blocks-onboarding-vertical-steps-nav")],
        vec![
            steps::prev_trigger(s, vec![], vec![core_text("前へ")]),
            steps::next_trigger(s, vec![], vec![core_text("次へ")]),
        ],
    )
}

/// `onboarding-vertical-steps` の Demo 本体（呼び出しごとに同一の `Node`
/// を返す純関数。状態は `Steps::new(4, 2, Orientation::Vertical)` 固定、
/// モジュール doc「状態は固定」節参照）。
pub fn demo() -> Node {
    let s = Steps::new(4, 2, Orientation::Vertical);
    let body = steps::body(vec![], vec![current_content(&s), nav(&s)]);
    let root = steps::root(
        Size::Md,
        ColorPalette::Accent,
        &s,
        vec![("data-blocks-onboarding-vertical-steps-root", "")],
        vec![step_list(&s), body],
    );
    div(
        vec![("class", "blocks-onboarding-vertical-steps-stack")],
        vec![root],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/onboarding-vertical-steps/",
    title: "onboarding-vertical-steps",
    category: BlockCategory::Onboarding,
    rust_source: "crates/docs-site/src/blocks/application/onboarding/onboarding_vertical_steps.rs",
    demo_class: "blocks-onboarding-vertical-steps",
    parts: &[
        Part {
            label: "Steps",
            path: "/themes/steps/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
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
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `onboarding_vertical_steps` 固有のレイアウト規則（`crate::blocks::
/// LAYOUT_CSS` doc「block 固有 CSS の置き場」節と同型）。
const LAYOUT_CSS: &str = "\
.blocks-onboarding-vertical-steps-stack {\n  container-type: inline-size;\n  container-name: blocks-onboarding-vertical-steps;\n}\n\
[data-blocks-onboarding-vertical-steps-root] {\n  gap: var(--fandhe-space-8);\n  align-items: flex-start;\n}\n\
[data-blocks-onboarding-vertical-steps-root] > [data-scope=\"steps\"][data-part=\"list\"] {\n  flex: 0 0 16rem;\n}\n\
[data-blocks-onboarding-vertical-steps-root] > [data-scope=\"steps\"][data-part=\"body\"] {\n  flex: 1 1 0;\n  min-width: 0;\n}\n\
.blocks-onboarding-vertical-steps-media {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-onboarding-vertical-steps-nav {\n  display: flex;\n  gap: var(--fandhe-space-2);\n  justify-content: flex-end;\n}\n\
@container blocks-onboarding-vertical-steps (max-width: 40rem) {\n  \
[data-scope=\"steps\"][data-part=\"root\"][data-orientation=\"vertical\"] {\n    flex-direction: column;\n  }\n  \
[data-blocks-onboarding-vertical-steps-root] > [data-scope=\"steps\"][data-part=\"list\"] {\n    flex-basis: auto;\n    flex-direction: row;\n    flex-wrap: wrap;\n  }\n  \
[data-scope=\"steps\"][data-part=\"separator\"] {\n    display: none;\n  }\n\
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
            "data-scope=\"steps\"",
            "data-scope=\"image\"",
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"button\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
    }

    #[test]
    fn root_is_vertical_and_split_into_list_and_body() {
        let html = demo_html();
        assert!(html.contains(r#"data-orientation="vertical""#));
        assert!(html.contains(r#"data-scope="steps" data-part="body""#));
    }

    #[test]
    fn exactly_one_step_is_current_and_two_are_complete() {
        let html = demo_html();
        assert_eq!(html.matches(r#"aria-current="step""#).count(), 1);
        // 完了ステップ（step 0・1）は item/trigger/indicator/separator の
        // 4 パーツそれぞれが `data-state="complete"` を持つため、
        // 2 ステップ分で計 8 件（`crates/headless-ui/src/steps.rs` の
        // `item`/`trigger`/`indicator`/`separator` 各実装が共通で付与）。
        assert_eq!(html.matches(r#"data-state="complete""#).count(), 8);
    }

    #[test]
    fn only_the_current_step_content_is_rendered() {
        let html = demo_html();
        // step 2（現在）の content のみを描画し、`hidden` 付きの閉じた
        // content を出力しない（モジュール doc「状態は固定」節参照）。
        assert_eq!(
            html.matches(r#"data-scope="steps" data-part="content""#)
                .count(),
            1
        );
        // 閉じた content が付与するブール属性 `hidden`（`aria-hidden` の
        // ような他属性の部分文字列一致を避けるため空白区切りで判定する）。
        assert!(!html.contains(" hidden ") && !html.contains(" hidden>"));
    }

    #[test]
    fn no_form_submit_video_tag_or_data_uri() {
        let html = demo_html();
        assert!(!html.contains("<form"));
        assert!(!html.contains("<video"));
        assert!(!html.contains("src=\"data:"));
        assert!(!html.contains("<script"));
        assert!(!html.contains("type=\"submit\""));
    }

    #[test]
    fn all_buttons_are_type_button() {
        let html = demo_html();
        for occurrence in html.match_indices("<button") {
            let tail = &html[occurrence.0..];
            let end = tail.find('>').unwrap_or(tail.len());
            assert!(
                tail[..end].contains(r#"type="button""#),
                "button element must be type=\"button\": {}",
                &tail[..end]
            );
        }
    }

    #[test]
    fn layout_css_is_safe_and_stacks_on_narrow_container() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("container-type: inline-size;"));
        assert!(
            LAYOUT_CSS.contains("@container blocks-onboarding-vertical-steps (max-width: 40rem)")
        );
        assert!(LAYOUT_CSS.contains("flex-direction: column;"));
    }
}
