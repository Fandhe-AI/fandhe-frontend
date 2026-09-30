//! `onboarding-vertical-steps` block（イシュー #2981。親 #2951
//! 「Blocks アプリケーション B」配下、Application / Onboarding カテゴリの
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
//! # `content` 直下の縦積み間隔
//!
//! [`fandhe_frontend_pre_styled_ui::steps::content`] は既定で
//! `display`/`gap` を持たない素の `<div>` のため、直下に並べる
//! 動画枠・見出し・説明文が間隔なしで密着してしまう。[`LAYOUT_CSS`] で
//! `[data-scope="steps"][data-part="content"]` へ `display: flex;
//! flex-direction: column; gap: var(--fandhe-space-4)` を明示し、
//! `help_center_article_list` 等の他 block と同じ縦積み間隔を持たせる
//! （Bugbot #2981 指摘の是正）。
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
//! wrap` に、`separator` を `display: none` に切り替える。`root` 既定の
//! `align-items: flex-start`（cross-axis 縮小フィット）のままだと
//! `list`/`body` が内容幅にしか広がらず `flex-wrap` が機能しないため、
//! この幅でのみ `root` の `align-items` を `stretch` へ上書きし、
//! `list` がコンテナ幅いっぱいに広がって折り返せるようにする
//! （Bugbot #2981 指摘の是正）。この上書きセレクタは
//! `[data-blocks-onboarding-vertical-steps-root]` 単独（詳細度 (0,1,0)）
//! ではなく、`root` が同時に持つ `[data-scope="steps"][data-part="root"]
//! [data-orientation="vertical"]` を連結した詳細度 (0,4,0) で書く。
//! `crates/pre-styled-ui/src/steps.rs` の `root` 既定 CSS が縦向きで
//! `align-items: flex-start` を詳細度 (0,3,0) で宣言しており、
//! `@container` はカスケードの詳細度に影響しない（適用条件を絞るだけ）
//! ため、詳細度で下回る単独セレクタでは狭幅でも上書きが効かず
//! `stretch` に切り替わらなかった（Codex #2981 指摘の是正）。
//!
//! # 狭幅で `item` が折り返さず題名が潰れる不具合の是正
//!
//! `steps` の `item` 既定 CSS（`crates/pre-styled-ui/src/steps.rs`）は
//! `flex: 1`（= `flex-basis: 0%`）を持つ。狭幅で `list` を
//! `flex-wrap: wrap` に切り替えても、折り返し判定に使う各 `item` の
//! 仮定主寸法（flex-basis）が 0 のままだと、ブラウザは 4 件すべてが
//! 1 行に収まると判定してから `flex-grow` で残り幅を均等分配するため、
//! 実際には折り返さず題名だけが極端に細く潰れる（Codex #2981 指摘）。
//! `@container` 内で `item` に `flex: 1 1 10rem; min-width: 10rem;` を
//! 上書きし、仮定主寸法を実寸に近づけて折り返し判定を機能させる
//! （セレクタは `[data-blocks-onboarding-vertical-steps-root]
//! [data-scope="steps"][data-part="item"]` の詳細度 (0,3,0) で `item`
//! 既定 CSS の (0,2,0) を上回る）。
//!
//! # 狭幅の `root` `gap` 上書きが効かない不具合の是正
//!
//! [`LAYOUT_CSS`] 冒頭の `root` 用ルール（`gap: var(--fandhe-space-8)`）は
//! 元は `[data-blocks-onboarding-vertical-steps-root]` 単独（詳細度
//! (0,1,0)）で書かれており、`steps` `root` 既定 CSS の `gap:
//! var(--fandhe-space-4)`（`[data-scope="steps"][data-part="root"]`、
//! 詳細度 (0,2,0)）に負けて `list`/`body` の間隔が縮まったままになって
//! いた（Cursor Bugbot #2981 指摘）。上記「`align-items` 上書き」と同じ
//! 理由でカスケード順にも頼れないため、`data-scope`/`data-part` を連結
//! した詳細度 (0,3,0) のセレクタへ書き換えて確実に上回るようにする。
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
     右側には手順動画のプレビュー（静止画）を表示します。";

/// 縦向きステップ一覧（左カラム）。`showcase::steps_demo` と同型に
/// `item` → `trigger`（`indicator` に番号 + 題名 `text`）+ 末尾以外に
/// `separator` を並べる。
fn step_list(s: &Steps) -> Node {
    let mut items = Vec::new();
    for (index, title) in STEP_TITLES.iter().enumerate() {
        let mut item_children = vec![steps::trigger(
            s,
            index,
            // 無 JS の docs サイトでは押しても状態遷移しない dead control
            // になるため、ネイティブ `disabled` で操作不能を構造的に表現
            // する（Codex #2981 指摘の是正。prev/next と同型、モジュール
            // 冒頭 doc「状態は固定」節参照）。`data-disabled` も併記する:
            // `crates/pre-styled-ui/src/steps.rs` の `trigger` hover 規則
            // （`StateCondition::Hover` が自動生成する
            // `:hover:not([data-disabled])`）はネイティブ `disabled` 属性
            // を条件に含まないため、`disabled` のみではホバー表示・
            // ポインターカーソルが無効ステップに残っていた（Codex #2981
            // 指摘の是正）。
            vec![("disabled", ""), ("data-disabled", "")],
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
                    // 動画ソース・再生処理を持たない装飾的なボタンのため、
                    // ネイティブ `disabled` で操作不能であることを構造的に
                    // 保証する（`onboarding_checklist` の実行ボタンと同型の
                    // 判断。Codex #2981 指摘の是正）。
                    disabled: true,
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
            // `step` は 0（先頭）でも `count`（末尾）でもないため headless
            // 側の境界無効化（`step == 0`/`step == count`）は働かない。
            // 無 JS の静的 Demo では押しても状態遷移しないため、trigger と
            // 同様にネイティブ `disabled` で操作不能を明示する（Codex
            // #2981 指摘の是正）。
            steps::prev_trigger(s, vec![("disabled", "")], vec![core_text("前へ")]),
            steps::next_trigger(s, vec![("disabled", "")], vec![core_text("次へ")]),
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
[data-blocks-onboarding-vertical-steps-root][data-scope=\"steps\"][data-part=\"root\"] {\n  gap: var(--fandhe-space-8);\n  align-items: flex-start;\n}\n\
[data-blocks-onboarding-vertical-steps-root] > [data-scope=\"steps\"][data-part=\"list\"] {\n  flex: 0 0 16rem;\n}\n\
[data-blocks-onboarding-vertical-steps-root] > [data-scope=\"steps\"][data-part=\"body\"] {\n  flex: 1 1 0;\n  min-width: 0;\n}\n\
[data-blocks-onboarding-vertical-steps-root] [data-scope=\"steps\"][data-part=\"content\"] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-onboarding-vertical-steps-media {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-onboarding-vertical-steps-nav {\n  display: flex;\n  gap: var(--fandhe-space-2);\n  justify-content: flex-end;\n}\n\
@container blocks-onboarding-vertical-steps (max-width: 40rem) {\n  \
[data-blocks-onboarding-vertical-steps-root][data-scope=\"steps\"][data-part=\"root\"][data-orientation=\"vertical\"] {\n    align-items: stretch;\n  }\n  \
[data-scope=\"steps\"][data-part=\"root\"][data-orientation=\"vertical\"] {\n    flex-direction: column;\n  }\n  \
[data-blocks-onboarding-vertical-steps-root] > [data-scope=\"steps\"][data-part=\"list\"] {\n    flex-basis: auto;\n    flex-direction: row;\n    flex-wrap: wrap;\n  }\n  \
[data-blocks-onboarding-vertical-steps-root] [data-scope=\"steps\"][data-part=\"item\"] {\n    flex: 1 1 10rem;\n    min-width: 10rem;\n  }\n  \
[data-scope=\"steps\"][data-part=\"separator\"] {\n    display: none;\n  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS, STEP_TITLES};
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
        // 狭幅で list が折り返せるよう root の align-items を上書きする
        // （Bugbot #2981 指摘の是正）。
        assert!(LAYOUT_CSS.contains("align-items: stretch;"));
        // content 直下の縦積み間隔（Bugbot #2981 指摘の是正）。
        assert!(LAYOUT_CSS.contains(
            "[data-scope=\"steps\"][data-part=\"content\"] {\n  display: flex;\n  flex-direction: column;\n  gap:"
        ));
    }

    #[test]
    fn all_step_controls_are_natively_disabled() {
        // 無 JS の静的 Demo では押しても状態遷移しないため、trigger /
        // prev-trigger / next-trigger / 再生ボタンをすべてネイティブ
        // `disabled` で操作不能にする（dead control 回避、Codex #2981
        // 指摘の是正）。
        let html = demo_html();
        for part in ["trigger", "prev-trigger", "next-trigger"] {
            let needle = format!("data-part=\"{part}\"");
            let pos = html.find(&needle).unwrap_or_else(|| {
                panic!("expected {part} to be rendered");
            });
            let tail = &html[..pos];
            let start = tail.rfind("<button").unwrap();
            let end = html[start..].find('>').unwrap() + start;
            assert!(
                html[start..end].contains("disabled"),
                "{part} button should be disabled: {}",
                &html[start..end]
            );
        }
        // trigger x4 + prev-trigger + next-trigger + 再生ボタン = 7 件。
        assert_eq!(html.matches(" disabled").count(), 7, "{html}");
    }

    #[test]
    fn narrow_container_stretch_override_beats_root_default_specificity() {
        // 詳細度 (0,4,0) の上書きセレクタが root 既定（詳細度 (0,3,0)）を
        // 上回ることを固定する（Codex #2981 指摘の是正）。単独属性
        // セレクタ（詳細度 (0,1,0)）へ後退させない回帰防止。
        assert!(LAYOUT_CSS.contains(
            "[data-blocks-onboarding-vertical-steps-root][data-scope=\"steps\"][data-part=\"root\"][data-orientation=\"vertical\"] {\n    align-items: stretch;\n  }"
        ));
    }

    #[test]
    fn disabled_step_triggers_also_carry_data_disabled() {
        // trigger の hover 抑制（`crates/pre-styled-ui/src/steps.rs` の
        // `StateCondition::Hover` が自動生成する
        // `:hover:not([data-disabled])`）はネイティブ `disabled` 属性を
        // 条件に含まないため、`data-disabled` 併記がないとホバー表示が
        // 無効ステップに残る（Codex #2981 指摘の是正）。
        let html = demo_html();
        let needle = "data-part=\"trigger\"";
        let mut search_from = 0;
        let mut count = 0;
        while let Some(rel) = html[search_from..].find(needle) {
            let pos = search_from + rel;
            let start = html[..pos].rfind("<button").unwrap();
            let end = html[start..].find('>').unwrap() + start;
            assert!(
                html[start..end].contains("data-disabled"),
                "trigger button should carry data-disabled: {}",
                &html[start..end]
            );
            count += 1;
            search_from = end;
        }
        assert_eq!(count, STEP_TITLES.len());
    }

    #[test]
    fn narrow_container_item_gets_basis_so_it_can_wrap_per_item() {
        // `item` 既定は `flex: 1`（flex-basis 0%）のため、`list` を
        // `flex-wrap: wrap` にしただけでは折り返し判定が機能せず
        // 4 件が 1 行に押し込まれて題名が潰れる（PR #3433 Codex 指摘の
        // 是正）。狭幅では実寸に近い flex-basis/min-width を与え、
        // item 単位で折り返させる。詳細度 (0,3,0) が item 既定
        // （詳細度 (0,2,0)）を確実に上回ることも固定する。
        assert!(LAYOUT_CSS.contains(
            "[data-blocks-onboarding-vertical-steps-root] [data-scope=\"steps\"][data-part=\"item\"] {\n    flex: 1 1 10rem;\n    min-width: 10rem;\n  }"
        ));
    }

    #[test]
    fn root_gap_override_beats_steps_root_default_specificity() {
        // root の gap 上書き（`var(--fandhe-space-8)`）が単独属性セレクタ
        // （詳細度 (0,1,0)）のままだと steps root 既定の gap（詳細度
        // (0,2,0)）に負けて縮まったままになる（PR #3433 Cursor Bugbot
        // 指摘の是正）。`data-scope`/`data-part` を連結した詳細度
        // (0,3,0) へ後退させない回帰防止。
        assert!(LAYOUT_CSS.contains(
            "[data-blocks-onboarding-vertical-steps-root][data-scope=\"steps\"][data-part=\"root\"] {\n  gap: var(--fandhe-space-8);"
        ));
    }
}
