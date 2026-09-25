//! `contact-form-testimonial` block（イシュー #2828。親トラッキング #2730
//! 「Blocks 目的別パーツ拡充ツリー」配下、区分は marketing、カテゴリは
//! Contact の最初の block）。問い合わせフォームの列と推薦文の列を横に
//! 並べる 2 列コンテンツの合成例（対応表 ID R0858 の 1 件のみを参照元と
//! する。取得手段・ファイル名・内部コンポーネント識別子は記載しない、
//! `docs/design/motion-reference-adoption-policy.md` §9 と同じライセンス
//! 上の転記制限）。
//!
//! # 使用部品
//!
//! `heading` / `text` / `field` / `input` / `textarea` / `button` /
//! `blockquote` / `image` / `icon` の 9 部品を合成する（[`BLOCK`] の
//! `parts` に一致させる契約、`crates/docs-site/tests/blocks_nav.rs`/
//! `blocks_contract.rs` が検証する）。
//!
//! # レイアウトとブレークポイント
//!
//! `>= 64rem` でフォーム列と推薦文列の 2 列、`< 64rem` でフォーム →
//! 推薦文の順の 1 列にする。DOM 順は両形とも「フォーム → 推薦文」であり、
//! 1 列表示時の読み順は DOM 順とそのまま一致する（`order` プロパティは
//! 使わない）。テーマの breakpoint トークンは `@media` 条件式の中では
//! 解決できないため（CSS custom property は宣言側でのみ有効）、
//! [`fandhe_frontend_pre_styled_ui::recipe::Breakpoint::Lg`]（1024px =
//! 64rem）と一致するリテラル値を [`LAYOUT_CSS`] へ直書きする
//! （`content_with_testimonial` と同じ判断）。ルート grid の class
//! （`blocks-contact-form-testimonial-layout`）は [`Block::demo_class`]
//! （`blocks-contact-form-testimonial`）とは意図的に別名にする
//! （`content_columns_screenshot`/`content_with_testimonial` と同じ
//! Bugbot 教訓の回避）。
//!
//! 入力欄グリッド（`.fields`）は `>= 48rem`（
//! [`fandhe_frontend_pre_styled_ui::recipe::Breakpoint::Md`]）で 2 列に
//! なる。名・姓の 2 欄は横に並び、それ以外（予算・Web サイト）も同じ
//! 2 列グリッドへ自然に収まる。本文欄（`textarea`）と送信ボタンのみ
//! `data-blocks-contact-form-testimonial-field-wide` を付与して 2 列へ
//! またがらせる（全幅）。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `heading::heading` / `text::text` / `field::root` / `input::input` /
//! `textarea::textarea` / `button::button` / `blockquote::root` /
//! `image::image` / `icon` はいずれも `drop_class_attr` により呼び出し側
//! `attrs` の `class` を黙って除去する契約を持つため、Demo 固有のスタイル
//! フックは `data-blocks-contact-form-testimonial-*` 属性で渡し、
//! [`LAYOUT_CSS`] 側も同じ属性セレクタで対応する。素の `div`・
//! `blockquote::content`/`caption` には `class` がそのまま効くため、それら
//! は `.blocks-contact-form-testimonial-*` クラスセレクタを使う。
//!
//! # 詳細度の罠（recipe への勝ち方）
//!
//! 部品 recipe（詳細度 (0,2,0)）に確実に勝つため、上書きは
//! `[data-scope="…"][data-part="…"][data-blocks-contact-form-testimonial-*]`
//! の 3 セレクタ構成（詳細度 (0,3,0)）で行う
//! （`content_with_testimonial` と同型の判断）。
//!
//! # 見出しレベルに `H3` を使う理由
//!
//! ページ側が `## Demo` として `h2` を出すため、見出しは
//! `HeadingLevel::H3` にする（他 block と同じ判断）。
//!
//! # `text` の名前衝突
//!
//! `fandhe_frontend_pre_styled_ui::text::text` と
//! `fandhe_frontend_core::text` が同名のため、styled 側を `styled_text`
//! として取り込む（`crate::blocks` 内の他 block と同じ回避方法）。
//!
//! # `<form>` を持たない・データ取得/送信を行わない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である。文言はすべて架空のもの（実企業名・実クレデンシャル・PII を
//! 含まない）。送信ボタンは `button::button` の既定 `type="button"` の
//! まま使い、暗黙の submit は起きない。同意文はプレーンテキストのみで
//! リンクにしない（`href="#"` は `linkcheck::check_links` が broken として
//! 検知するため使わない。`link` 部品も使用部品一覧に含まれない）。
//!
//! # ロゴ相当のマーク
//!
//! 実在ブランドのロゴ・商標を模した SVG は持ち込まず、抽象的な六角形の
//! 幾何図形を [`icon`] で描く（`login_04::geo_icon` と同型の判断）。
//!
//! # 写真
//!
//! [`crate::blocks::dummy_assets::AVATAR_SRC`]（人物アバター、ビルド時
//! 生成のモノトーン SVG）を使い、`alt=""`（装飾扱い）で出力する。隣接する
//! 氏名テキストが同じ情報を伝えるため、代替テキストの重複を避ける。
//!
//! # 原案との差分（対応表 ID R0858 からの差分、ライセンス上の理由に加え
//! 独自の判断も含む）
//!
//! - 背景装飾（グリッド模様等の装飾レイヤ）は持ち込まない。
//! - 同意文はリンクにしない（プレーンテキストのみ）。
//! - ロゴは抽象的な `icon` で置き換える。
//! - 写真は共通のダミー素材（[`crate::blocks::dummy_assets::AVATAR_SRC`]）
//!   を使う。
//! - 配色・文言は既存のトーンに揃えた。
//! - `<form>` を出力しない。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::blockquote::{self, BlockquoteVariant};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::field::{self, FieldRootProps};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::image::{self, ImageFit, ImageProps};
use fandhe_frontend_pre_styled_ui::input::{self, FieldIds, FieldProps, InputProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::textarea::{self, TextareaProps};
use fandhe_frontend_pre_styled_ui::ColorPalette;

/// フォーム欄 1 個ぶんの `FieldProps` を組み立てる（`id` ごとに一意にし、
/// `field::root`/`field::label`/`input::input`/`textarea::textarea` へ
/// 使い回す。表示状態軸は持たない静的な合成例のためすべて既定値）。
fn contact_field(id: &'static str) -> FieldProps<'static> {
    FieldProps {
        id,
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: false,
    }
}

/// フォーム欄 1 個を `field::root` へ組み立てる（ラベル + コントロール、
/// `wide` が `true` のとき `data-blocks-contact-form-testimonial-field-wide`
/// を付与してグリッドの 2 列へまたがらせる）。
fn field_wrapper(
    field: &FieldProps<'_>,
    label_text: &'static str,
    control: Node,
    wide: bool,
) -> Node {
    let mut attrs = vec![("data-blocks-contact-form-testimonial-field", "")];
    if wide {
        attrs.push(("data-blocks-contact-form-testimonial-field-wide", ""));
    }
    field::root(
        &FieldRootProps::default(),
        field,
        attrs,
        vec![field::label(field, vec![], vec![text(label_text)]), control],
    )
}

/// 抽象的な六角形のロゴ相当マーク（実在ブランドのロゴ・商標を模さない、
/// モジュール doc「ロゴ相当のマーク」節参照）。
fn mark_icon() -> Node {
    icon(
        &IconProps::default(),
        vec![("data-blocks-contact-form-testimonial-mark", "")],
        vec![el(
            "path",
            vec![("d", "M12 2l8.66 5v10L12 22l-8.66-5V7z")],
            vec![],
        )],
    )
}

/// `contact-form-testimonial` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。
pub fn demo() -> Node {
    let first_name_field = contact_field("blocks-contact-form-testimonial-first-name");
    let last_name_field = contact_field("blocks-contact-form-testimonial-last-name");
    let budget_field = contact_field("blocks-contact-form-testimonial-budget");
    let website_field = contact_field("blocks-contact-form-testimonial-website");
    let message_field = contact_field("blocks-contact-form-testimonial-message");

    let fields = div(
        vec![("class", "blocks-contact-form-testimonial-fields")],
        vec![
            field_wrapper(
                &first_name_field,
                "名",
                input::input(
                    &InputProps::default(),
                    &first_name_field,
                    vec![("type", "text")],
                ),
                false,
            ),
            field_wrapper(
                &last_name_field,
                "姓",
                input::input(
                    &InputProps::default(),
                    &last_name_field,
                    vec![("type", "text")],
                ),
                false,
            ),
            field_wrapper(
                &budget_field,
                "ご予算",
                input::input(
                    &InputProps::default(),
                    &budget_field,
                    vec![("type", "text")],
                ),
                false,
            ),
            field_wrapper(
                &website_field,
                "Web サイト",
                input::input(
                    &InputProps::default(),
                    &website_field,
                    vec![("type", "url"), ("placeholder", "https://example.com")],
                ),
                false,
            ),
            field_wrapper(
                &message_field,
                "ご相談内容",
                textarea::textarea(
                    &TextareaProps::default(),
                    &message_field,
                    false,
                    vec![("rows", "4")],
                    vec![],
                ),
                true,
            ),
        ],
    );

    let form_col = div(
        vec![("class", "blocks-contact-form-testimonial-form")],
        vec![
            fields,
            button::button(
                &ButtonProps::default(),
                vec![("data-blocks-contact-form-testimonial-submit", "")],
                vec![text("送信する")],
            ),
            styled_text::text(
                &TextProps {
                    size: TextSize::Sm,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(
                    "送信すると、プライバシーポリシーに同意したものとみなされます。",
                )],
            ),
        ],
    );

    let aside = div(
        vec![("class", "blocks-contact-form-testimonial-aside")],
        vec![
            mark_icon(),
            blockquote::root(
                BlockquoteVariant::default(),
                ColorPalette::default(),
                vec![("data-blocks-contact-form-testimonial-quote", "")],
                vec![
                    blockquote::content(vec![], vec![text(dummy_assets::TESTIMONIAL_QUOTES[2])]),
                    blockquote::caption(
                        vec![("class", "blocks-contact-form-testimonial-meta")],
                        vec![
                            image::image(
                                &ImageProps {
                                    fit: ImageFit::Cover,
                                    ..ImageProps::new(dummy_assets::AVATAR_SRC, "")
                                },
                                vec![("data-blocks-contact-form-testimonial-photo", "")],
                            ),
                            div(
                                vec![("class", "blocks-contact-form-testimonial-byline")],
                                vec![
                                    div(vec![], vec![text(dummy_assets::PERSON_NAMES[3])]),
                                    div(vec![], vec![text(dummy_assets::JOB_TITLES[3])]),
                                ],
                            ),
                        ],
                    ),
                ],
            ),
        ],
    );

    let header = div(
        vec![("class", "blocks-contact-form-testimonial-header")],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl3,
                    weight: HeadingWeight::Bold,
                },
                vec![],
                vec![text("プロジェクトのご相談")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(
                    "まずはお気軽にお問い合わせください。担当より折り返しご連絡いたします。",
                )],
            ),
        ],
    );

    div(
        vec![("class", "blocks-contact-form-testimonial-layout")],
        vec![
            header,
            div(
                vec![("class", "blocks-contact-form-testimonial-grid")],
                vec![form_col, aside],
            ),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/contact-form-testimonial/",
    title: "contact-form-testimonial",
    category: BlockCategory::Contact,
    rust_source: "crates/docs-site/src/blocks/marketing/contact/contact_form_testimonial.rs",
    demo_class: "blocks-contact-form-testimonial",
    parts: &[
        Part {
            label: "Heading",
            path: "/themes/heading/",
        },
        Part {
            label: "Text",
            path: "/themes/text/",
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
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Blockquote",
            path: "/themes/blockquote/",
        },
        Part {
            label: "Image",
            path: "/themes/image/",
        },
        Part {
            label: "Icon",
            path: "/themes/icon/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `contact_form_testimonial` 固有のレイアウト規則（`crate::blocks::
/// LAYOUT_CSS` doc「block 固有 CSS の置き場」節。他 block と同型で
/// `pub(super)` ではなく本ファイル内 private 定数として `super::stylesheet`
/// 経由の `push_css` で連結される）。
///
/// セレクタは `.blocks-contact-form-testimonial-*` と
/// `[data-blocks-contact-form-testimonial-*]` のみを用い、他 block や
/// 部品の素のセレクタへ影響させない（`content_with_testimonial` と同じ
/// 名前空間分離）。
const LAYOUT_CSS: &str = "\
.blocks-contact-form-testimonial-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-10);\n}\n\
.blocks-contact-form-testimonial-header {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n  max-width: 40rem;\n}\n\
.blocks-contact-form-testimonial-grid {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-10);\n}\n\
.blocks-contact-form-testimonial-form {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n  min-width: 0;\n}\n\
.blocks-contact-form-testimonial-fields {\n  display: grid;\n  grid-template-columns: minmax(0, 1fr);\n  gap: var(--fandhe-space-4);\n}\n\
[data-scope=\"button\"][data-part=\"root\"][data-blocks-contact-form-testimonial-submit] {\n  width: 100%;\n}\n\
.blocks-contact-form-testimonial-aside {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n  min-width: 0;\n}\n\
[data-scope=\"icon\"][data-part=\"root\"][data-blocks-contact-form-testimonial-mark] {\n  width: 2rem;\n  height: 2rem;\n  color: var(--fandhe-color-fg-muted);\n}\n\
[data-scope=\"blockquote\"][data-part=\"caption\"].blocks-contact-form-testimonial-meta {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-3);\n}\n\
[data-scope=\"image\"][data-part=\"root\"][data-blocks-contact-form-testimonial-photo] {\n  width: 2.5rem;\n  height: 2.5rem;\n  border-radius: var(--fandhe-radius-full);\n  object-fit: cover;\n  flex-shrink: 0;\n}\n\
.blocks-contact-form-testimonial-byline {\n  display: flex;\n  flex-direction: column;\n  font-size: var(--fandhe-font-font-size-sm);\n}\n\
@media (min-width: 48rem) {\n  \
.blocks-contact-form-testimonial-fields {\n    grid-template-columns: repeat(2, minmax(0, 1fr));\n  }\n\
[data-scope=\"field\"][data-part=\"root\"][data-blocks-contact-form-testimonial-field-wide] {\n    grid-column: 1 / -1;\n  }\n\
}\n\
@media (min-width: 64rem) {\n  \
.blocks-contact-form-testimonial-grid {\n    display: grid;\n    grid-template-columns: minmax(0, 3fr) minmax(0, 2fr);\n    gap: var(--fandhe-space-12);\n    align-items: start;\n  }\n\
.blocks-contact-form-testimonial-aside {\n    border-inline-start: 1px solid var(--fandhe-color-border);\n    padding-inline-start: var(--fandhe-space-8);\n  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::{demo, dummy_assets, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// Demo が期待する 9 部品を出力すること、非対話制約（`<form>` 不在・
    /// `data:` URI 不在・`href="#"` 不在）を満たすことの単体回帰
    /// （`crates/docs-site/tests/blocks_contract.rs` の横断検査と重複し
    /// 過ぎない範囲での個別固定）。
    #[test]
    fn demo_composes_expected_parts_and_avoids_forms() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"field\"",
            "data-scope=\"blockquote\"",
            "data-scope=\"image\"",
            "data-scope=\"button\"",
            "data-scope=\"icon\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert!(html.contains("data-part=\"input\""));
        assert!(html.contains("data-part=\"textarea\""));
        assert_eq!(html.matches("<form").count(), 0);
        assert_eq!(html.matches("type=\"button\"").count(), 1);
        assert_eq!(html.matches("<img").count(), 1);
        assert_eq!(html.matches("<textarea").count(), 1);
        assert!(!html.contains("src=\"data:"));
        assert!(!html.contains("href=\"#\""));
        assert!(html.contains(dummy_assets::AVATAR_SRC));
    }

    /// 5 欄それぞれのラベルが一意なコントロール id を指し、id が重複しない
    /// こと（`field::label`/`field::input`/`field::textarea` の自動導出、
    /// `demo_output_has_no_dangling_aria_references_or_duplicate_ids` の
    /// 個別固定）。
    #[test]
    fn field_labels_point_at_unique_control_ids() {
        let html = render(&demo());
        let ids = [
            "blocks-contact-form-testimonial-first-name",
            "blocks-contact-form-testimonial-last-name",
            "blocks-contact-form-testimonial-budget",
            "blocks-contact-form-testimonial-website",
            "blocks-contact-form-testimonial-message",
        ];
        for id in ids {
            let control_id = format!("{id}-control");
            assert!(
                html.contains(&format!("for=\"{control_id}\"")),
                "missing label for {control_id}"
            );
            assert_eq!(
                html.matches(&format!("id=\"{control_id}\"")).count(),
                1,
                "control id {control_id} should be unique"
            );
        }
    }

    /// [`LAYOUT_CSS`] が想定するブレークポイント条件・全幅欄の
    /// `grid-column`・送信ボタンの全幅を持つこと。
    #[test]
    fn layout_css_declares_breakpoints_and_full_width_submit() {
        assert!(LAYOUT_CSS.contains("@media (min-width: 64rem)"));
        assert!(LAYOUT_CSS.contains("@media (min-width: 48rem)"));
        assert!(LAYOUT_CSS.contains("grid-column: 1 / -1"));
        assert!(LAYOUT_CSS.contains(
            "[data-scope=\"button\"][data-part=\"root\"][data-blocks-contact-form-testimonial-submit] {\n  width: 100%;\n}"
        ));
    }

    /// ルート grid class（`demo_class` とは別名）が `demo()` の出力へ
    /// 実際に現れること（モジュール doc「レイアウトとブレークポイント」
    /// 節の Bugbot 教訓の固定）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-contact-form-testimonial-layout\""));
        assert_ne!(
            super::BLOCK.demo_class,
            "blocks-contact-form-testimonial-layout"
        );
    }
}
