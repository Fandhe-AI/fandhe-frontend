//! `ai-chat-prompt-start` block（イシュー #2961。親トラッキング #2951
//! 「Blocks アプリケーション B」配下、phase:4）。AI チャットの開始画面。
//! 対応表の主参照
//! R0002（代表構成）を軸に、集約元 R0003（挨拶見出し + 控えめな候補
//! ボタン意匠）・R0004（見出しと入力欄を画面中央へ寄せる構成）を
//! `default`/`centered` の 2 インスタンスとして併記する。`_/blocks-intake/`
//! の対応ファイルは本イシュー着手時点で本 worktree に存在しないため、
//! 対応表 ID のみを記す（`page_heading_avatar.rs` と同じ扱い）。
//!
//! # 使用部品
//!
//! `textarea` / `button` / `empty-state` / `menu` / `icon` / `heading` の
//! 6 部品のみを合成する（[`BLOCK`] の `parts` に一致させる契約）。新しい
//! UI 部品は追加しない。
//!
//! # 2 インスタンスで派生形を表現する（無 JS のため静的併記）
//!
//! - **default**（R0002 主参照 + R0003 の候補ボタン意匠）: [`empty_state`]
//!   の挨拶見出し（アイコン + 見出し + 説明文） → 候補ボタン 2×2
//!   （`ButtonVariant::Outline`） → composer（下記）の順に縦積みする。
//!   composer はパネル下端へ寄せる（`margin-top: auto`、`default` 変種限定。
//!   `centered` は `justify-content: center` でパネル自体を中央寄せするため
//!   composer への下端固定は適用しない）。
//! - **centered**（R0004）: パネルを縦方向中央へ寄せ、見出し → composer →
//!   控えめな候補ボタン（`ButtonVariant::Ghost`、R0003 の「控えめ」）の順に
//!   置く。
//!
//! # composer に `field`/`input-group` を使わない理由
//!
//! `hero_prompt_input.rs` は `field::root` + `input_group::root` で
//! textarea を包むが、本 block の使用部品一覧（Issue 本文由来）に
//! `field`/`input-group` は含まれない。`textarea::textarea` は
//! `fandhe_frontend_headless_ui::field::textarea` を直接呼び出すため、
//! `field::root`/`input_group::root` を経由せず `div` で直接包んでも
//! `data-scope="field"` の `<textarea>` 自体は出力される（headless
//! `field` パーツの契約は `field::root` を要求しない）。
//!
//! # 下端固定を `margin-top: auto` で表現する理由
//!
//! `.blocks-demo` 枠は `overflow-x: auto`（横スクロールコンテナ）のため
//! `position: sticky` は Demo 内で機能保証がなく（`content_article_toc.rs`
//! の判断と同型）、`position: fixed` も使わない（`content_article.rs` に
//! 前例なし）。本 block では composer をパネル内の flex 末尾（`margin-top:
//! auto`）へ寄せることで「下端に固定された入力欄」を表現する。この下端
//! 寄せは `default` 変種限定（`[data-blocks-ai-chat-prompt-start-variant=
//! "default"] .blocks-ai-chat-prompt-start-composer` セレクタで限定）で
//! ある。`centered` 変種はパネル自体を `justify-content: center` で中央へ
//! 寄せる構成のため、composer にも `margin-top: auto` を適用すると flex
//! auto-margin が中央寄せより優先され、見出しと入力欄が下端へ押し下げら
//! れてしまう（R0004 の表示契約に反する）。実アプリでは `position: sticky;
//! bottom: 0` を使える旨を `site/blocks/ai-chat-prompt-start.md` の差分
//! メモに明記する。
//!
//! # 三点メニューは無 JS のため閉じた状態で固定する
//!
//! ツール選択メニューは開閉状態機械・実際のポップアップ表示を持たず、
//! `OpenState::Closed` の静的表示のみを描画する
//! （`page_heading_avatar.rs::overflow_menu` と同型）。2 インスタンスの
//! `content`/`trigger` id はインスタンスごとに一意にする（id 重複・宙ぶらり
//! `aria-controls`/`aria-labelledby` 参照の回避、`blocks_contract.rs`
//! 参照）。
//!
//! # `drop_class_attr` のための CSS フック方針
//!
//! `textarea::textarea` / `button::*` / `icon::icon` / `empty_state::*` /
//! `heading::heading` / `menu::*` はいずれも `drop_class_attr` により
//! 呼び出し側 `attrs` の `class` を除去する契約を持つため、Demo 固有の
//! スタイルフックは `data-blocks-ai-chat-prompt-start-*` 属性で渡す。素の
//! `div` にのみ `class` を使う（`hero_prompt_input.rs` と同型の判断）。
//!
//! # `<form>` を使わない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない。ボタンは既定 `type="button"` のまま用い、送信・添付・
//! ツール選択の処理は一切実装しない静的表示のみである。
//!
//! アイコンはすべて抽象的な線画（実在ブランドのロゴ・商標を模さない）。
//! 文言はすべて架空のもの（実在の人物・企業・PII を含まない）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
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
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/ai-chat-prompt-start/",
    title: "ai-chat-prompt-start",
    category: BlockCategory::AiChat,
    rust_source: "crates/docs-site/src/blocks/application/ai_chat/ai_chat_prompt_start.rs",
    demo_class: "blocks-ai-chat-prompt-start",
    parts: &[
        Part {
            label: "Textarea",
            path: "/themes/textarea/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Empty State",
            path: "/themes/empty-state/",
        },
        Part {
            label: "Menu",
            path: "/themes/menu/",
        },
        Part {
            label: "Icon",
            path: "/themes/icon/",
        },
        Part {
            label: "Heading",
            path: "/themes/heading/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `ai_chat_prompt_start` 固有のレイアウト規則（`crate::blocks`
/// モジュール doc「CSS の置き場」節と同型）。候補ボタンの列数はデモ枠
/// （`.blocks-demo` 本文カラム内）の実測幅に追随させる必要があるため
/// ビューポート幅ベースの `@media` ではなくパネル自身を名前付き
/// コンテナ（`container-type: inline-size`）とした `@container` クエリで
/// 判定し、狭い場合は 1 列に積む（`list_title_meta.rs` と同型のパターン）。
/// composer をパネル下端へ寄せる（モジュール doc「下端固定を
/// `margin-top: auto` で表現する理由」節参照）のは `default` 変種限定。
const LAYOUT_CSS: &str = "\
.blocks-ai-chat-prompt-start-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n}\n\
.blocks-ai-chat-prompt-start-panel {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n  min-height: 28rem;\n  max-width: 48rem;\n  margin-inline: auto;\n  width: 100%;\n  container-type: inline-size;\n  container-name: blocks-ai-chat-prompt-start;\n}\n\
[data-blocks-ai-chat-prompt-start-variant=\"centered\"] {\n  justify-content: center;\n}\n\
.blocks-ai-chat-prompt-start-suggestions {\n  display: grid;\n  grid-template-columns: 1fr;\n  gap: var(--fandhe-space-3);\n}\n\
[data-blocks-ai-chat-prompt-start-variant=\"default\"] .blocks-ai-chat-prompt-start-composer {\n  margin-top: auto;\n}\n\
.blocks-ai-chat-prompt-start-composer {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n  border: 1px solid var(--fandhe-color-border);\n  border-radius: var(--fandhe-radius-lg);\n  padding: var(--fandhe-space-3);\n}\n\
[data-blocks-ai-chat-prompt-start-variant] [data-scope=\"field\"][data-part=\"textarea\"] {\n  border: 0;\n}\n\
.blocks-ai-chat-prompt-start-composer-actions {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-ai-chat-prompt-start-composer-actions [data-scope=\"button\"]:last-child {\n  margin-inline-start: auto;\n}\n\
@container blocks-ai-chat-prompt-start (min-width: 28rem) {\n  .blocks-ai-chat-prompt-start-suggestions {\n    grid-template-columns: repeat(2, minmax(0, 1fr));\n  }\n}\n";

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
            "data-scope=\"button\"",
            "data-scope=\"empty-state\"",
            "data-scope=\"menu\"",
            "data-scope=\"icon\"",
            "data-scope=\"heading\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
        assert!(html.contains(r#"data-part="textarea""#));
    }

    #[test]
    fn demo_wires_both_variant_hooks() {
        let html = demo_html();
        for variant in ["default", "centered"] {
            assert!(html.contains(&format!(
                "data-blocks-ai-chat-prompt-start-variant=\"{variant}\""
            )));
        }
    }

    #[test]
    fn no_form_semantics_or_dead_links() {
        let html = demo_html();
        assert!(!html.contains("<form"));
        assert!(!html.contains("type=\"submit\""));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("src=\"data:"));
    }

    #[test]
    fn all_buttons_are_type_button() {
        let html = demo_html();
        let count_open = html.matches("<button").count();
        let count_typed = html.matches("type=\"button\"").count();
        assert!(count_open > 0);
        assert!(count_typed >= count_open, "html={html}");
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
    fn suggestions_are_four_per_instance() {
        let html = demo_html();
        assert_eq!(
            html.matches("data-blocks-ai-chat-prompt-start-suggestion=\"\"")
                .count(),
            8
        );
    }

    #[test]
    fn layout_css_is_safe_and_stacks_suggestions_on_narrow() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("container-type: inline-size;"));
        assert!(LAYOUT_CSS.contains("@container blocks-ai-chat-prompt-start (min-width: 28rem)"));
        assert!(LAYOUT_CSS.contains("grid-template-columns: 1fr;"));
        assert!(!LAYOUT_CSS.contains("position: fixed"));
        assert!(!LAYOUT_CSS.contains("display: none"));
    }

    #[test]
    fn composer_bottom_margin_is_scoped_to_default_variant() {
        assert!(LAYOUT_CSS.contains(
            "[data-blocks-ai-chat-prompt-start-variant=\"default\"] .blocks-ai-chat-prompt-start-composer {\n  margin-top: auto;\n}"
        ));
    }

    #[test]
    fn textarea_border_reset_uses_ancestor_selector() {
        assert!(LAYOUT_CSS.contains(
            "[data-blocks-ai-chat-prompt-start-variant] [data-scope=\"field\"][data-part=\"textarea\"] {\n  border: 0;\n}"
        ));
    }

    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = demo_html();
        assert!(html.contains("class=\"blocks-ai-chat-prompt-start-layout\""));
        assert_ne!(
            super::BLOCK.demo_class,
            "blocks-ai-chat-prompt-start-layout"
        );
    }

    #[test]
    fn demo_is_deterministic() {
        assert_eq!(demo_html(), demo_html());
    }
}
