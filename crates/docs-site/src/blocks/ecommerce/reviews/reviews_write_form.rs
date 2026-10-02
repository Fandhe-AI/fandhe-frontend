//! `reviews-write-form` block（イシュー #3091。親トラッキング #2730 系
//! 「Blocks 目的別パーツ拡充」の一件）。対応表 ID R0213（主参照・集約元
//! ともに R0213 の 1 件のみ）を構造の参照元とする合成例。取得手段・
//! ファイル名・内部コンポーネント識別子は記載しない
//! （`docs/design/motion-reference-adoption-policy.md` §9 と同じライセンス
//! 上の転記制限）。
//!
//! # 使用部品
//!
//! `heading` / `field` / `input` / `textarea` / `rating-group` / `button`
//! の 6 部品を合成する（[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//!
//! # レイアウト（見出し → 名前 → メール → 評価 → タイトル → 本文 → 投稿）
//!
//! 見出しの下に、名前・メールアドレス・評価・タイトル・本文の入力欄を
//! 縦 1 列に並べ、最後に投稿ボタンを置く。コンテナクエリ・メディアクエリ
//! は持たず、常に 1 列（狭幅でも列replace が起きない）。
//!
//! # `<form>` を持たない・送信処理を持たない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo はフォーム・
//! 状態機械を持たない静的な合成例である。ボタンは `button::button`
//! （既定 `type="button"`）のまま送信先・バリデーションを持たず、実際の
//! 送信処理は利用者自身の Rust/JS コードで実装する
//! （`docs/policy/intentional-non-adoption.md` §3.25）。
//!
//! # 評価は `rating-group`（編集可能、既定値 4 の静的表示）で表現する
//!
//! `reviews_card_grid.rs::rating_row` が readonly の表示専用評価なのに
//! 対し、本 block は「投稿フォーム」という性質上、編集可能な
//! （`readonly: false`）`rating-group` を使う。ただし無 JS の静的合成例
//! という block 全体の設計方針により、実際のクリック・キーボード操作に
//! よる値変更は配線しない（`item` は `tabindex` を出力しない仕様、
//! `crates/headless-ui/src/rating_group.rs` 参照）。既定値は
//! `RatingGroup::new(5, Some(4), false)` で固定し、`required: true` を
//! 付与して投稿フォームとしての意味（評価必須）を保つ。`disabled` は
//! 付与しない（`consent_checkbox` 系 block のように誤操作でネイティブ
//! 状態が変わる要素を持たないため。`item` の `span` は labelable でも
//! `tabindex` 保持でもなく、クリックしても状態は変化しない）。
//!
//! # 評価に `field::root` を使わない理由
//!
//! `field::label` は `<label for>` を出すが、rating の control は
//! labelable 要素ではないため dangling 参照になる。評価は
//! `rating_group::label` で名前を付け、`div` で包んで他フィールドと
//! 縦並びの見た目を揃える（`contact_centered_form.rs` の
//! `field::root`/`label` とは別経路で統一する）。
//!
//! # hidden input の送信値
//!
//! `rating_group::hidden_input` は `name="rating"` で現在値（`"4"`）を
//! 保持する。`<form>` を持たないため実際の送信経路はないが、利用者が
//! 本 block を実フォームへ組み込む際にそのまま送信値として使える構造を
//! 示す。
//!
//! # `text` の名前衝突
//!
//! `fandhe_frontend_core::text`（テキストノード生成関数）のみを使い、
//! styled `text::text` は使わない（本 block は見出し・ラベル・ボタン文言
//! のみで `<p>` を必要としないため）。
//!
//! # 見出しレベル（`H3`）
//!
//! ページ側が `## Demo` として `h2` を出すため、セクション見出しは
//! [`HeadingLevel::H3`] にする（`contact_centered_form.rs` 等と同型の
//! 判断）。
//!
//! # ダミー素材について
//!
//! 名前・メールアドレス・タイトル・本文の placeholder はすべて独自に
//! 書いた架空の内容であり、実在の企業名・人物・PII・有料アセット名を
//! 含まない。メールの placeholder は `example.com` 系を使う。
//!
//! # 原案差分メモ・スコープ外
//!
//! R0213（名前・メール・評価・題名・本文 + 投稿）の構成をそのまま採用
//! した。見出しを `h3` にし、`<form>`/送信処理を持たず、評価を既存の
//! rating-group（clip-path 星）へ置き換えた。文言・配色・装飾は参照元
//! から持ち込まず独自に書いた（詳細は `site/blocks/reviews-write-form.md`
//! の「原案差分メモ」節参照）。`_/blocks-intake/` の参照ファイルは本
//! イシュー着手時点で worktree に存在しないため、対応表 ID のみを記して
//! 実装した。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::field::{self, FieldOrientation, FieldRootProps};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps};
use fandhe_frontend_pre_styled_ui::input::{self, FieldIds, FieldProps, InputProps};
use fandhe_frontend_pre_styled_ui::rating_group::{
    self, RatingGroup, RatingGroupProps, RatingItemFlags,
};
use fandhe_frontend_pre_styled_ui::textarea::{self, TextareaProps};
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// 一意な id を組み立てる（`blocks-reviews-write-form-` 接頭辞を共通化し、
/// フィールド追加時の綴り間違いを防ぐ）。
fn field_id(suffix: &str) -> String {
    format!("blocks-reviews-write-form-{suffix}")
}

/// 縦積み（label 上・control 下）の共通 orientation。
fn orientation() -> FieldRootProps {
    FieldRootProps {
        orientation: FieldOrientation::Vertical,
    }
}

/// 通常フィールド（`text`/`email` 等）を組み立てる。全項目を必須にする
/// （R0213 の「名前・メール・評価・題名・本文 + 投稿」が全項目必須の構成
/// であることに対応）。
fn text_field(
    id: String,
    label_text: &'static str,
    input_type: &'static str,
    autocomplete: &'static str,
    placeholder: &'static str,
) -> Node {
    let props = FieldProps {
        id: id.as_str(),
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: true,
        readonly: false,
        has_helper_text: false,
    };
    field::root(
        &orientation(),
        &props,
        vec![("data-blocks-reviews-write-form-field", "")],
        vec![
            field::label(&props, vec![], vec![text(label_text)]),
            input::input(
                &InputProps::default(),
                &props,
                vec![
                    ("type", input_type),
                    ("autocomplete", autocomplete),
                    ("placeholder", placeholder),
                ],
            ),
        ],
    )
}

/// 評価フィールド（編集可能 `rating-group`、既定値 4 固定の静的表示。
/// モジュール doc「評価は `rating-group`」節参照）。`field::root` ではなく
/// 専用の `div` ラッパーで他フィールドと縦並びの見た目を揃える
/// （モジュール doc「評価に `field::root` を使わない理由」節参照）。
fn rating_field() -> Node {
    let label_id = field_id("rating-label");
    let props = RatingGroupProps {
        disabled: false,
        readonly: false,
        required: true,
    };
    let state = RatingGroup::new(5, Some(4), false);
    let label = rating_group::label(&props, Some(label_id.as_str()), vec![], vec![text("評価")]);
    let items: Vec<Node> = (1..=state.count())
        .map(|i| {
            rating_group::item(
                i,
                RatingItemFlags {
                    checked: state.is_checked(i),
                    highlighted: state.is_highlighted(i),
                    disabled: false,
                    readonly: false,
                },
                &format!("{i} star{}", if i == 1 { "" } else { "s" }),
                vec![],
                vec![],
            )
        })
        .collect();
    let control = rating_group::control(&props, Some(label_id.as_str()), vec![], items);
    let hidden = rating_group::hidden_input(&props, Some("rating"), &state.value_text(), vec![]);
    div(
        vec![("data-blocks-reviews-write-form-field", "")],
        vec![rating_group::root(
            Size::Md,
            ColorPalette::Accent,
            &props,
            vec![],
            vec![label, control, hidden],
        )],
    )
}

/// お題名（全幅・必須の `input`）。
fn title_field() -> Node {
    text_field(
        field_id("title"),
        "タイトル",
        "text",
        "off",
        "ひとことで言うと",
    )
}

/// 本文（必須の `textarea`）。
fn body_field() -> Node {
    let id = field_id("body");
    let props = FieldProps {
        id: id.as_str(),
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: true,
        readonly: false,
        has_helper_text: false,
    };
    field::root(
        &orientation(),
        &props,
        vec![("data-blocks-reviews-write-form-field", "")],
        vec![
            field::label(&props, vec![], vec![text("本文")]),
            textarea::textarea(
                &TextareaProps::default(),
                &props,
                false,
                vec![
                    ("placeholder", "実際に使ってみた感想をご記入ください。"),
                    ("data-blocks-reviews-write-form-body", ""),
                ],
                vec![],
            ),
        ],
    )
}

/// `reviews-write-form` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数（`crate::blocks` モジュール doc「静的表示」節）。
#[must_use]
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-reviews-write-form-layout")],
        vec![
            heading::heading(
                HeadingLevel::H3,
                &HeadingProps::default(),
                vec![],
                vec![text("レビューを書く")],
            ),
            text_field(field_id("name"), "名前", "text", "name", "山田 太郎"),
            text_field(
                field_id("email"),
                "メールアドレス",
                "email",
                "email",
                "you@example.com",
            ),
            rating_field(),
            title_field(),
            body_field(),
            button::button(
                &ButtonProps::default(),
                vec![("data-blocks-reviews-write-form-submit", "")],
                vec![text("レビューを投稿する")],
            ),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/reviews-write-form/",
    title: "reviews-write-form",
    category: BlockCategory::Reviews,
    rust_source: "crates/docs-site/src/blocks/ecommerce/reviews/reviews_write_form.rs",
    demo_class: "blocks-reviews-write-form",
    parts: &[
        Part {
            label: "Heading",
            path: "/themes/heading/",
        },
        Part {
            label: "Field",
            path: "/themes/field/",
        },
        Part {
            label: "Input",
            path: "/themes/input/",
        },
        Part {
            label: "Textarea",
            path: "/themes/textarea/",
        },
        Part {
            label: "Rating Group",
            path: "/themes/rating-group/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `reviews_write_form` 固有のレイアウト規則（`crate::blocks` モジュール
/// doc「block 固有 CSS の置き場」節と同型）。狭幅でも 1 列のまま
/// （メディアクエリ・コンテナクエリを持たない）。評価ラベルの見た目を
/// `field::label` と揃えるため、`rating-group` の label へ `--fandhe-*`
/// トークンで同じ書体規則を適用する（base 規則〔詳細度 0,2,0〕に勝つよう
/// 詳細度を上げる）。
const LAYOUT_CSS: &str = "\
.blocks-reviews-write-form-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n  max-width: 36rem;\n  margin-inline: auto;\n}\n\
[data-scope=\"rating-group\"][data-part=\"label\"][data-blocks-reviews-write-form-field] {\n  font-size: var(--fandhe-font-font-size-sm);\n  font-weight: var(--fandhe-font-font-weight-medium);\n  color: var(--fandhe-color-fg);\n  margin-bottom: var(--fandhe-space-2);\n}\n\
[data-scope=\"button\"][data-part=\"root\"][data-blocks-reviews-write-form-submit] {\n  width: 100%;\n}\n\
";

#[cfg(test)]
mod tests {
    use super::{demo, BLOCK, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    fn demo_html() -> String {
        render(&demo())
    }

    #[test]
    fn demo_composes_expected_parts() {
        let html = demo_html();
        for scope in [
            "data-scope=\"heading\"",
            "data-scope=\"field\"",
            "data-scope=\"rating-group\"",
            "data-scope=\"button\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
        assert!(html.contains(r#"data-part="input""#));
        assert!(html.contains(r#"data-part="textarea""#));
    }

    #[test]
    fn demo_has_no_form_or_unsafe_output() {
        let html = demo_html();
        assert!(!html.contains("<form"));
        assert!(!html.contains("action="));
        assert!(!html.contains("href="));
        assert!(!html.contains("src=\"data:"));
    }

    #[test]
    fn exactly_one_type_button_and_no_type_submit() {
        let html = demo_html();
        let count_open = html.matches("<button").count();
        let count_typed = html.matches("type=\"button\"").count();
        assert!(count_open > 0);
        assert_eq!(count_open, 1);
        assert_eq!(count_typed, count_open);
        assert!(!html.contains(r#"type="submit""#));
    }

    #[test]
    fn labels_point_at_their_controls() {
        let html = demo_html();
        for suffix in ["name", "email", "title", "body"] {
            let control_id = format!("blocks-reviews-write-form-{suffix}-control");
            assert!(
                html.contains(&format!(r#"for="{control_id}""#)),
                "expected a label pointing at {control_id} in {html}"
            );
            assert!(
                html.contains(&format!(r#"id="{control_id}""#)),
                "expected control id {control_id} in {html}"
            );
        }
    }

    #[test]
    fn rating_defaults_to_four_and_is_editable() {
        let html = demo_html();
        assert_eq!(html.matches(r#"aria-checked="true""#).count(), 1);
        assert!(html.contains(r#"data-value="4" role="radio" aria-checked="true""#));
        assert_eq!(html.matches("data-highlighted").count(), 4);
        assert!(html.contains(r#"aria-required="true""#));
        assert!(html.contains(r#"value="4""#));
        assert!(html.contains(r#"name="rating""#));
        // 編集可能（readonly ではない）ことを data-readonly 不在で固定する。
        assert!(!html.contains("data-readonly"));
    }

    #[test]
    fn rating_control_labelledby_target_exists() {
        let html = demo_html();
        let label_id = "blocks-reviews-write-form-rating-label";
        assert!(html.contains(&format!(r#"aria-labelledby="{label_id}""#)));
        assert!(html.contains(&format!(r#"id="{label_id}""#)));
    }

    #[test]
    fn demo_is_deterministic() {
        assert_eq!(demo_html(), demo_html());
    }

    #[test]
    fn ids_are_unique() {
        let html = demo_html();
        let ids: Vec<&str> = html
            .split("id=\"")
            .skip(1)
            .filter_map(|rest| rest.split('"').next())
            .collect();
        let unique: std::collections::BTreeSet<&str> = ids.iter().copied().collect();
        assert_eq!(ids.len(), unique.len(), "ids should all be unique: {ids:?}");
    }

    #[test]
    fn layout_css_is_safe_single_column_and_has_no_breakpoints() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("flex-direction: column"));
        assert!(!LAYOUT_CSS.contains("@media"));
        assert!(!LAYOUT_CSS.contains("@container"));
        assert!(!LAYOUT_CSS.contains("grid-template-columns: repeat"));
    }

    #[test]
    fn demo_class_differs_from_layout_root_class() {
        assert_ne!(BLOCK.demo_class, "blocks-reviews-write-form-layout");
    }
}
