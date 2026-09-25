//! `hero-email-signup` block（イシュー #2783。親トラッキング #2730
//! 「Blocks 目的別パーツ拡充ツリー」配下、Phase 1（#2738、マーケティング A）
//! に属する。対応表 ID R0132（基準形）/ R0549（一体化した入力グループ）/
//! R0550（動画プレースホルダ）/ R0449（旧 cta-email-split-image 統合）を
//! 集約した 3 形として合成する。取得手段・ファイル名・内部コンポーネント
//! 識別子は記載しない（`super::super::cta::cta_split_image` と同じ
//! ライセンス上の転記制限、対応表 ID のみを記す）。
//!
//! # 使用部品
//!
//! `badge` / `heading` / `text` / `field` / `input_group` / `input` /
//! `button` / `image` / `visually_hidden` の 9 部品を合成する（[`BLOCK`]
//! の `parts` に一致させる契約）。新規 UI 部品は作らない。
//!
//! # 3 形を 1 つの Demo に並記する
//!
//! [`super::super::cta::cta_split_image`] と同型に、3 形を
//! [`variant_label`] で見出しを付けながら [`demo`] 1 つの中へ縦に並べる。
//!
//! | 形 | 対応 ID | 内容 |
//! |----|---------|------|
//! | A 基準形 | R0132/R0549/R0449 | 左: badge/見出し/本文/メール入力グループ、右: 正方形画像 |
//! | B 動画プレースホルダ | R0550 | 右列を画像の代わりに再生アイコン付きの静的プレースホルダにする |
//! | C セクション末尾 CTA | R0132 派生 | A と同じ骨格。見出しは参照元の `h2` ではなく `h3` のまま [`HeadingSize`] を一段小さくして表現する |
//!
//! # `<form>` を使わない・送信処理を持たない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない。登録ボタンは `type="button"`（[`button::button`] の既定
//! 契約）のまま送信先を持たず、実際のバリデーション・送信処理は利用者
//! 自身の Rust/JS コードで実装する（`docs/policy/intentional-non-adoption.md`
//! §3.25）。
//!
//! # 可視ラベルの代わりに `visually_hidden` + `<label for>`
//!
//! [`super::super::banner::banner_email_signup`] と同じ判断軸で、可視
//! ラベルを出さず [`visually_hidden::root`] で包んだ [`field::label`] に
//! より入力欄のアクセシブル名を `<label for>` の関連付けで確保する。
//!
//! # 入力グループのボタンは Themes `button` を addon 内に置く
//!
//! 仕様の使用部品に `button` が明記され、視覚的にも主 CTA の重みが要る
//! ため、[`input_group::addon`] の子には `input_group::button`
//! ではなく [`button::button`] を置く（`showcase::input_group_section`
//! と同じ合成契約: `field::root` > [`field::label`, `input_group::root` >
//! [`input::input`, `input_group::addon` > ボタン]]）。
//!
//! # `id` は 3 インスタンス分すべて別値にする
//!
//! A/B/C の 3 インスタンスは同一 Demo 内へ並記するため、`FieldProps::id`
//! （ラベル `for`/input `id` の関連付け元）を suffix で分ける
//! （`demo_output_has_no_dangling_aria_references_or_duplicate_ids` 契約、
//! `crate::blocks` モジュール doc 参照）。
//!
//! # B: 動画プレースホルダは実動画・再生操作を持たない
//!
//! 再生アイコンは自作の幾何アイコン（[`super::super::cta::cta_split_image::geo_icon`]
//! と同型の単純な三角形の折れ線）を装飾（`aria-hidden="true"`）として
//! 重ねるのみで、実際の動画再生・ボタン化は行わない静的な合成例である。
//!
//! # 見出しレベル
//!
//! ページ側が `## Demo` として `h2` を出すため、全ての見出しは
//! `HeadingLevel::H3` を使う。C（セクション末尾 CTA）の参照元は `h2` を
//! 使うが、ページ内 TOC を汚さないため `h3` のまま [`HeadingSize`] を
//! 一段小さくして視覚的な階層差を表現する。
//!
//! # ブレークポイントをリテラルで直書きする理由
//!
//! テーマの breakpoint トークンは `@media` 条件式の中では解決できないため、
//! [`fandhe_frontend_pre_styled_ui::recipe::Breakpoint::Lg`]（1024px =
//! 64rem）と一致するリテラル値を [`LAYOUT_CSS`] へ直書きする（既存 block と
//! 同じ判断）。
//!
//! # `drop_class_attr` と CSS フックの選び方
//!
//! `badge::badge`/`heading::heading`/`text::text`/`field::root`/
//! `input::input`/`input_group::root`/`input_group::addon`/
//! `button::button`/`image::image`/`visually_hidden::root` はいずれも
//! `drop_class_attr` により呼び出し側 `attrs` の `class` を黙って除去する
//! 契約を持つため、Demo 固有のスタイルフックは
//! `data-blocks-hero-email-signup-*` 属性で渡し、[`LAYOUT_CSS`] 側も同じ
//! 属性セレクタで対応する。素の `div` には `class` がそのまま効くため、
//! レイアウト用のグリッド配置は従来どおり `.blocks-hero-email-signup-*`
//! クラスセレクタを使う。レイアウト root の class
//! （`blocks-hero-email-signup-layout`）は [`Block::demo_class`]
//! （`blocks-hero-email-signup`）と意図的に別名にする（既存 block と同じ
//! Bugbot 教訓の回避）。
//!
//! # `crate::blocks::dummy_assets` を直接参照しない理由（コードフェンス自己完結）
//!
//! `crate::blocks::dummy_assets` は `pub(crate)` のため、Markdown 原稿の
//! Rust コードフェンス（`// blocks-code:begin`/`:end` マーカー内）から
//! `use crate::blocks::dummy_assets;` は単体ではコンパイルできない懸念が
//! あるが、他の block（`cta_split_image` 等）も同じ経路を持ち込んでおり、
//! 原稿は「実装からの引用」であって単体コンパイル対象ではない契約
//! （`blocks_code_drift.rs` はバイト一致のみを検証する）のため、本 block
//! も同じ経路を採用する。
//!
//! # `<form>` を持たない・データ取得/送信を行わない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である。文言はすべて架空のもの（実企業名・実クレデンシャル・PII を
//! 含まない）。画像は [`crate::blocks::dummy_assets`] のビルド時生成
//! プレースホルダー SVG のみを使い、装飾扱いの `alt=""` で出力する
//! （`data:` URI は使わない）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::field::{
    self, FieldIds, FieldOrientation, FieldProps, FieldRootProps,
};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::image::{self, AspectRatio, ImageFit, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::input::{self, InputProps};
use fandhe_frontend_pre_styled_ui::input_group::{self, InputGroupAlign, InputGroupProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::visually_hidden;

/// 各形の直前に置く短い形ラベル（`cta_split_image::variant_label` と同型）。
fn variant_label(label: &'static str) -> Node {
    styled_text::text(
        &TextProps {
            size: TextSize::Sm,
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![],
        vec![text(label)],
    )
}

/// メールアドレス入力 + 送信ボタンを一体化した入力グループ
/// （モジュール doc「入力グループのボタンは Themes `button` を addon 内に
/// 置く」節）。`instance` はインスタンスごとに異なる `id` の suffix
/// （モジュール doc「`id` は 3 インスタンス分すべて別値にする」節）。
fn signup_group(instance: &'static str) -> Node {
    let field_id = format!("blocks-hero-email-signup-email-{instance}");
    let email_field = FieldProps {
        id: &field_id,
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: true,
        readonly: false,
        has_helper_text: false,
    };
    let group_props = InputGroupProps {
        disabled: false,
        invalid: false,
    };

    field::root(
        &FieldRootProps {
            orientation: FieldOrientation::Vertical,
        },
        &email_field,
        vec![("data-blocks-hero-email-signup-field", "")],
        vec![
            visually_hidden::root(
                vec![],
                vec![field::label(
                    &email_field,
                    vec![],
                    vec![text("メールアドレス")],
                )],
            ),
            input_group::root(
                &group_props,
                vec![("data-blocks-hero-email-signup-group", "")],
                vec![
                    input::input(
                        &InputProps::default(),
                        &email_field,
                        vec![
                            ("type", "email"),
                            ("autocomplete", "email"),
                            ("placeholder", "you@example.com"),
                        ],
                    ),
                    input_group::addon(
                        InputGroupAlign::InlineEnd,
                        &group_props,
                        vec![("data-blocks-hero-email-signup-addon", "")],
                        vec![button::button(
                            &ButtonProps::default(),
                            vec![("data-blocks-hero-email-signup-submit", "")],
                            vec![text("登録する")],
                        )],
                    ),
                ],
            ),
        ],
    )
}

/// コピー列（eyebrow badge + 見出し + リード文 + 入力グループ）。
fn copy_column(
    instance: &'static str,
    heading_size: HeadingSize,
    eyebrow: &'static str,
    title: &'static str,
    body: &'static str,
) -> Node {
    div(
        vec![("class", "blocks-hero-email-signup-copy")],
        vec![
            badge::badge(&BadgeProps::default(), vec![], vec![text(eyebrow)]),
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: heading_size,
                    ..HeadingProps::default()
                },
                vec![],
                vec![text(title)],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(body)],
            ),
            signup_group(instance),
        ],
    )
}

/// コピー列 + メディア列を横並びグリッドへ束ねる（DOM 順はコピー列 →
/// メディア。`lg` 未満の 1 列表示でコピー列が上に来るようにするため）。
fn split(copy: Node, media: Node) -> Node {
    div(
        vec![("class", "blocks-hero-email-signup-grid")],
        vec![copy, media],
    )
}

/// 形 A（R0132/R0549/R0449 基準形）: 右列に正方形画像。
fn variant_image() -> Node {
    let media = image::image(
        &ImageProps {
            fit: ImageFit::Cover,
            aspect_ratio: AspectRatio::Square,
            shape: ImageShape::Rounded,
            ..ImageProps::new(dummy_assets::PRODUCT_SRC, "")
        },
        vec![("data-blocks-hero-email-signup-image", "")],
    );
    split(
        copy_column(
            "image",
            HeadingSize::Xl2,
            "早期アクセス",
            "リリース情報を、最初に受け取る",
            "新しいコンポーネントとテンプレートの通知を、公開の前に届けます。",
        ),
        media,
    )
}

/// 再生アイコン（自作の幾何アイコン。`cta_split_image::geo_icon` と同型の
/// 単純な三角形の折れ線。装飾のため `aria-hidden="true"`）。
fn play_icon() -> Node {
    el(
        "svg",
        vec![
            ("viewBox", "0 0 24 24"),
            ("width", "24"),
            ("height", "24"),
            ("aria-hidden", "true"),
        ],
        vec![el(
            "path",
            vec![("d", "M8 5l11 7-11 7V5z"), ("fill", "currentColor")],
            vec![],
        )],
    )
}

/// 形 B（R0550）: 画像の代わりに再生アイコン付きの動画プレースホルダ
/// （モジュール doc「B: 動画プレースホルダは実動画・再生操作を持たない」
/// 節）。
fn variant_video() -> Node {
    let media = div(
        vec![("class", "blocks-hero-email-signup-video")],
        vec![
            image::image(
                &ImageProps {
                    fit: ImageFit::Cover,
                    ..ImageProps::new(dummy_assets::BACKGROUND_SRC, "")
                },
                vec![("data-blocks-hero-email-signup-video-image", "")],
            ),
            div(
                vec![("class", "blocks-hero-email-signup-play")],
                vec![play_icon()],
            ),
        ],
    );
    split(
        copy_column(
            "video",
            HeadingSize::Xl2,
            "プロダクトツアー",
            "動く画面で、機能を先取りする",
            "3 分のプロダクトツアーと最新の更新情報を、メールでお届けします。",
        ),
        media,
    )
}

/// 形 C（R0132 派生、セクション末尾 CTA）: A と同じ骨格。見出しは `h3` の
/// まま [`HeadingSize`] を一段小さくして参照元の `h2` との階層差を表現する
/// （モジュール doc「見出しレベル」節）。
fn variant_cta() -> Node {
    let media = image::image(
        &ImageProps {
            fit: ImageFit::Cover,
            aspect_ratio: AspectRatio::Square,
            shape: ImageShape::Rounded,
            ..ImageProps::new(dummy_assets::PRODUCT_SRC, "")
        },
        vec![("data-blocks-hero-email-signup-cta-image", "")],
    );
    split(
        copy_column(
            "cta",
            HeadingSize::Xl,
            "もうすぐ公開",
            "続報を、見逃さないために",
            "公開日が決まり次第、登録済みのメールアドレスへ最初にお知らせします。",
        ),
        media,
    )
}

/// `hero-email-signup` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数（モジュール doc「3 形を 1 つの Demo に並記する」節）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-hero-email-signup-layout")],
        vec![
            variant_label("正方形画像（R0132/R0549/R0449 基準形）"),
            variant_image(),
            variant_label("動画プレースホルダ（R0550）"),
            variant_video(),
            variant_label("セクション末尾 CTA（R0132 派生）"),
            variant_cta(),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/hero-email-signup/",
    title: "hero-email-signup",
    category: BlockCategory::Hero,
    rust_source: "crates/docs-site/src/blocks/marketing/hero/hero_email_signup.rs",
    demo_class: "blocks-hero-email-signup",
    parts: &[
        Part {
            label: "Badge",
            path: "/themes/badge/",
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
            label: "Field",
            path: "/themes/field/",
        },
        Part {
            label: "Input Group",
            path: "/themes/input-group/",
        },
        Part {
            label: "Input",
            path: "/themes/input/",
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
            label: "Visually Hidden",
            path: "/themes/visually-hidden/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `hero_email_signup` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節。他 block と同型で `super::stylesheet`
/// から連結される）。既定（`lg` 未満）は 1 列で入力とボタンを全幅に畳み、
/// `>= 64rem` で 2 列グリッドへ切り替え、入力グループは横並びの通常表示へ
/// 戻す。
const LAYOUT_CSS: &str = "\
.blocks-hero-email-signup-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n}\n\
.blocks-hero-email-signup-grid {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n}\n\
.blocks-hero-email-signup-copy {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n  align-items: start;\n  min-width: 0;\n}\n\
[data-scope=\"field\"][data-part=\"root\"][data-blocks-hero-email-signup-field] {\n  width: 100%;\n  align-self: stretch;\n}\n\
[data-scope=\"input-group\"][data-part=\"root\"][data-blocks-hero-email-signup-group] {\n  flex-direction: column;\n  align-items: stretch;\n  border: 0;\n  background: transparent;\n  gap: var(--fandhe-space-3);\n}\n\
[data-scope=\"input-group\"][data-part=\"root\"][data-blocks-hero-email-signup-group] > [data-scope=\"field\"][data-part=\"input\"] {\n  border: 1px solid var(--fandhe-color-border);\n  border-radius: var(--fandhe-radius-md);\n}\n\
[data-scope=\"input-group\"][data-part=\"addon\"][data-blocks-hero-email-signup-addon] {\n  padding: 0;\n}\n\
[data-scope=\"button\"][data-part=\"root\"][data-blocks-hero-email-signup-submit] {\n  width: 100%;\n}\n\
[data-scope=\"image\"][data-part=\"root\"][data-blocks-hero-email-signup-image] {\n  display: block;\n  width: 100%;\n}\n\
[data-scope=\"image\"][data-part=\"root\"][data-blocks-hero-email-signup-cta-image] {\n  display: block;\n  width: 100%;\n}\n\
.blocks-hero-email-signup-video {\n  position: relative;\n  aspect-ratio: 1 / 1;\n  overflow: hidden;\n  border-radius: var(--fandhe-radius-lg);\n  background: var(--fandhe-color-bg-subtle);\n}\n\
[data-scope=\"image\"][data-part=\"root\"][data-blocks-hero-email-signup-video-image] {\n  display: block;\n  width: 100%;\n  height: 100%;\n}\n\
.blocks-hero-email-signup-play {\n  position: absolute;\n  inset: 0;\n  display: grid;\n  place-items: center;\n  color: var(--fandhe-color-fg);\n}\n\
.blocks-hero-email-signup-play svg {\n  width: 3rem;\n  height: 3rem;\n  padding: var(--fandhe-space-3);\n  border-radius: var(--fandhe-radius-full);\n  background: var(--fandhe-color-bg);\n  border: 1px solid var(--fandhe-color-border);\n}\n\
@media (min-width: 64rem) {\n  \
.blocks-hero-email-signup-grid {\n    display: grid;\n    grid-template-columns: repeat(2, minmax(0, 1fr));\n    gap: var(--fandhe-space-8);\n    align-items: center;\n  }\n  \
[data-scope=\"field\"][data-part=\"root\"][data-blocks-hero-email-signup-field] {\n    max-width: 28rem;\n  }\n  \
[data-scope=\"input-group\"][data-part=\"root\"][data-blocks-hero-email-signup-group] {\n    flex-direction: row;\n    align-items: center;\n    border: 1px solid var(--fandhe-color-border);\n    background: var(--fandhe-color-bg);\n    gap: 0;\n  }\n  \
[data-scope=\"input-group\"][data-part=\"root\"][data-blocks-hero-email-signup-group] > [data-scope=\"field\"][data-part=\"input\"] {\n    border: 0;\n  }\n  \
[data-scope=\"input-group\"][data-part=\"addon\"][data-blocks-hero-email-signup-addon] {\n    padding-block: var(--fandhe-space-1);\n    padding-inline: var(--fandhe-space-2);\n  }\n  \
[data-scope=\"button\"][data-part=\"root\"][data-blocks-hero-email-signup-submit] {\n    width: auto;\n  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::{demo, dummy_assets, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// Demo が期待する部品・構造・非対話制約を満たしていることの単体
    /// 回帰（`crates/docs-site/tests/blocks_contract.rs` の横断検査と重複
    /// し過ぎない範囲での個別固定）。
    #[test]
    fn demo_composes_expected_parts_and_avoids_forms() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"badge\"",
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"field\"",
            "data-scope=\"input-group\"",
            "data-scope=\"button\"",
            "data-scope=\"image\"",
            "data-scope=\"visually-hidden\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert_eq!(html.matches(r#"type="email""#).count(), 3);
        assert_eq!(html.matches(r#"type="button""#).count(), 3);
        assert_eq!(html.matches("<img").count(), 3);
        for absent in ["<form", "src=\"data:", "href=\"#\"", "<h2"] {
            assert!(
                !html.contains(absent),
                "demo output should never contain {absent}"
            );
        }
        assert!(html.contains(dummy_assets::PRODUCT_SRC));
        assert!(html.contains(dummy_assets::BACKGROUND_SRC));
    }

    /// visually-hidden な `<label for>` と同じ `id` を持つ input が 3
    /// インスタンス分存在する（アクセシブル名の関連付けを固定）。
    #[test]
    fn label_for_matches_input_id_for_each_instance() {
        let html = render(&demo());
        for instance in ["image", "video", "cta"] {
            let control_id = format!("blocks-hero-email-signup-email-{instance}-control");
            assert!(
                html.contains(&format!("for=\"{control_id}\"")),
                "demo output should contain for={control_id}"
            );
            assert!(
                html.contains(&format!("id=\"{control_id}\"")),
                "demo output should contain id={control_id}"
            );
        }
    }

    /// [`LAYOUT_CSS`] がレスポンシブ切替とレイアウトフックを含み、`<` を
    /// 含まない（`StyleSheet::push_css` の禁則文字チェック）。
    #[test]
    fn layout_css_declares_breakpoint_and_hooks() {
        assert!(LAYOUT_CSS.contains("@media (min-width: 64rem)"));
        for hook in [
            "data-blocks-hero-email-signup-field",
            "data-blocks-hero-email-signup-group",
            "data-blocks-hero-email-signup-addon",
            "data-blocks-hero-email-signup-submit",
        ] {
            assert!(LAYOUT_CSS.contains(hook), "LAYOUT_CSS に {hook} が無い");
        }
        assert!(!LAYOUT_CSS.contains('<'));
    }

    /// ルート grid class（`demo_class` とは別名）が `demo()` の出力へ実際に
    /// 現れること（既存 block と同じ Bugbot 教訓の固定）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-hero-email-signup-layout\""));
        assert_ne!(super::BLOCK.demo_class, "blocks-hero-email-signup-layout");
    }
}
