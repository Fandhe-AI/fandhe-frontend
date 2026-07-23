//! Image（イシュー #770）: 単一 recipe styled 部品。写真等の静的コンテンツを
//! 表示する `<img>` を組み立てる（chakra-ui `data-display/image` 相当）。
//!
//! [`fandhe_frontend_headless_ui::avatar`] の `image` パーツが持つ
//! `ImageStatus`（loading/loaded/error の状態機械、`src` 差し替え検知）とは
//! 独立した部品である。avatar の `image` はイニシャル/アイコンフォールバック
//! との切り替えという avatar 固有の関心事を担うのに対し、本モジュールは
//! 単体画像の `object-fit`/`aspect-ratio` の見た目 variant のみを提供する
//! 状態機械を持たない静的部品（`crate` 冒頭の「単純 styled 部品」群と同型）。
//!
//! # `src` の安全性（イシュー #373 検証への依拠）
//!
//! `src` は `fandhe_frontend_core::render` の既定エスケープに加え、
//! `fandhe_frontend_core::URL_ATTRS`（`src` を収載済み）経由の
//! `is_safe_url` 検証を通過しないと属性ごと不出力になる（fail-closed、
//! `docs/policy/attribute-output-policy.md` 参照）。本モジュールは検証
//! ロジックを複製せず、この render 経由の一元的な検証にのみ依拠する
//! （`.claude/rules/security.md` A05: 単一情報源の維持）。`javascript:` 等の
//! 不許可スキームを渡しても `<img>` 自体は出力されるが `src` 属性のみが
//! 欠落する（回帰テストで固定、[`tests`] モジュール参照）。
//!
//! 装飾用途ではない写真等のコンテンツを想定するため、[`ImageProps::alt`] を
//! 必須引数とする（[`fandhe_frontend_headless_ui::avatar::image`] と同じ
//! アクセシビリティ既定の判断）。中立的なコンテンツ表示部品のため
//! colorPalette 軸は付与しない（[`crate::card`]・[`crate::skeleton`] と
//! 同型の判断）。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{SlotRecipe, VariantValue};
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
use fandhe_frontend_headless_ui::{anatomy, Anatomy};

/// `data-scope="image"` を固定した本コンポーネントの anatomy。
const ANATOMY: Anatomy = anatomy("image");

/// `object-fit` variant（chakra-ui `Image` の `fit` prop 相当）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ImageFit {
    /// アスペクト比を保ちつつ枠を埋める（既定）。
    #[default]
    Cover,
    /// アスペクト比を保ちつつ枠内に収める。
    Contain,
    /// アスペクト比を無視して枠いっぱいに引き伸ばす。
    Fill,
    /// `contain` と `none` のうち小さく表示される方を採用する。
    ScaleDown,
    /// 原寸のまま配置する（CSS `object-fit: none`）。列挙子名を `None` に
    /// すると `Option::None` との視認上の混同を招くため避ける
    /// （`crates/core/src/tags.rs` の `select`/`option` 命名回避と同型の
    /// 判断）。
    NoFit,
}

impl VariantValue for ImageFit {
    fn axis(self) -> &'static str {
        "fit"
    }

    fn value(self) -> &'static str {
        match self {
            Self::Cover => "cover",
            Self::Contain => "contain",
            Self::Fill => "fill",
            Self::ScaleDown => "scale-down",
            Self::NoFit => "none",
        }
    }
}

/// `aspect-ratio` variant（chakra-ui `Image` の `aspectRatio` prop 相当）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AspectRatio {
    /// 画像本来の比率をそのまま使う（既定）。
    #[default]
    Auto,
    /// 1:1（正方形）。
    Square,
    /// 16:9（動画サムネイル等）。
    Video,
}

impl VariantValue for AspectRatio {
    fn axis(self) -> &'static str {
        "aspect-ratio"
    }

    fn value(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Square => "square",
            Self::Video => "video",
        }
    }
}

/// Image の recipe（scope `"image"`、slot `"root"` のみ）。
///
/// `decl()` の値検証（`crate::css::is_valid_value`）は `{`/`}`/`;`/`<`/制御
/// 文字のみを拒否するため、`/` を含む `aspect-ratio` の値（`1 / 1` 等）は
/// 宣言として表現できる。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("image", &["root"])
        .base(
            "root",
            vec![decl("display", "block"), decl("max-width", "100%")],
        )
        .variant(ImageFit::Cover, "root", vec![decl("object-fit", "cover")])
        .variant(
            ImageFit::Contain,
            "root",
            vec![decl("object-fit", "contain")],
        )
        .variant(ImageFit::Fill, "root", vec![decl("object-fit", "fill")])
        .variant(
            ImageFit::ScaleDown,
            "root",
            vec![decl("object-fit", "scale-down")],
        )
        .variant(ImageFit::NoFit, "root", vec![decl("object-fit", "none")])
        .variant(
            AspectRatio::Auto,
            "root",
            vec![decl("aspect-ratio", "auto")],
        )
        .variant(
            AspectRatio::Square,
            "root",
            vec![decl("aspect-ratio", "1 / 1")],
        )
        .variant(
            AspectRatio::Video,
            "root",
            vec![decl("aspect-ratio", "16 / 9")],
        )
        .default_variant(ImageFit::Cover)
        .default_variant(AspectRatio::Auto)
}

/// Image の静的 CSS 全文。
#[must_use]
pub fn css() -> String {
    recipe().css()
}

/// [`image`] の設定。
#[derive(Debug, Clone, Copy)]
pub struct ImageProps<'a> {
    /// 画像 URL。既定エスケープ + `is_safe_url` 検証（本モジュール doc
    /// 参照）を経由する。
    pub src: &'a str,
    /// 代替テキスト（アクセシビリティ上必須、空文字列も明示的な選択として
    /// 許容する）。
    pub alt: &'a str,
    /// `object-fit` variant（既定 `Cover`）。
    pub fit: ImageFit,
    /// `aspect-ratio` variant（既定 `Auto`）。
    pub aspect_ratio: AspectRatio,
}

impl<'a> ImageProps<'a> {
    /// `src`/`alt` を指定し、variant は既定値（`Cover`/`Auto`）のまま組み立てる。
    #[must_use]
    pub fn new(src: &'a str, alt: &'a str) -> Self {
        ImageProps {
            src,
            alt,
            fit: ImageFit::default(),
            aspect_ratio: AspectRatio::default(),
        }
    }
}

/// Image 1 個を組み立てる（`<img>`、空要素のため `children` 引数は持たない）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::image::{image, ImageProps};
///
/// let node = image(&ImageProps::new("/photo.png", "説明"), vec![]);
/// let html = render(&node);
/// assert!(html.contains(r#"src="/photo.png""#));
/// assert!(html.contains(r#"alt="説明""#));
/// ```
#[must_use]
pub fn image<'a>(props: &ImageProps<'a>, attrs: Vec<(&'a str, &'a str)>) -> Node {
    let recipe = recipe();
    let class = recipe.variant_classes(&[
        ("fit", props.fit.value()),
        ("aspect-ratio", props.aspect_ratio.value()),
    ]);
    // `merged` の型を `Vec<(&'a str, &'a str)>` と明示しない（推論に任せる）:
    // ローカル変数 `class` の借用寿命は `'a` より短いため、明示すると
    // 「`class` は `'a` まで生存しない」という借用エラーになる（badge/card
    // 等の既存 styled 部品と同じ回避策、`crate::badge::badge` 参照）。
    let mut merged: Vec<(&str, &str)> = vec![
        ("class", class.as_str()),
        ("src", props.src),
        ("alt", props.alt),
    ];
    merged.extend(drop_class_attr(attrs));
    ANATOMY.part("root", "img", merged, Vec::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;

    #[test]
    fn default_props_render_cover_auto() {
        let node = image(&ImageProps::new("/a.png", "alt text"), vec![]);
        let html = render(&node);
        assert_eq!(
            html,
            r#"<img data-scope="image" data-part="root" class="fd-image--fit-cover fd-image--aspect-ratio-auto" src="/a.png" alt="alt text"></img>"#
        );
    }

    #[test]
    fn fit_variants_map_to_expected_classes() {
        for (fit, class) in [
            (ImageFit::Cover, "fd-image--fit-cover"),
            (ImageFit::Contain, "fd-image--fit-contain"),
            (ImageFit::Fill, "fd-image--fit-fill"),
            (ImageFit::ScaleDown, "fd-image--fit-scale-down"),
            (ImageFit::NoFit, "fd-image--fit-none"),
        ] {
            let props = ImageProps {
                fit,
                ..ImageProps::new("/a.png", "alt")
            };
            let html = render(&image(&props, vec![]));
            assert!(
                html.contains(&format!("class=\"{class} fd-image--aspect-ratio-auto\"")),
                "fit={fit:?} -> {html}"
            );
        }
    }

    #[test]
    fn aspect_ratio_variants_map_to_expected_classes() {
        for (ratio, class) in [
            (AspectRatio::Auto, "fd-image--aspect-ratio-auto"),
            (AspectRatio::Square, "fd-image--aspect-ratio-square"),
            (AspectRatio::Video, "fd-image--aspect-ratio-video"),
        ] {
            let props = ImageProps {
                aspect_ratio: ratio,
                ..ImageProps::new("/a.png", "alt")
            };
            let html = render(&image(&props, vec![]));
            assert!(
                html.contains(&format!("class=\"fd-image--fit-cover {class}\"")),
                "aspect_ratio={ratio:?} -> {html}"
            );
        }
    }

    #[test]
    fn css_output_declares_object_fit_and_aspect_ratio() {
        let out = css();
        assert!(out.contains("object-fit: cover;"));
        assert!(out.contains("object-fit: none;"));
        assert!(out.contains("aspect-ratio: 1 / 1;"));
        assert!(out.contains("aspect-ratio: 16 / 9;"));
    }

    #[test]
    fn css_output_is_deterministic() {
        assert_eq!(css(), css());
    }

    #[test]
    fn caller_class_attr_is_dropped_not_duplicated() {
        let html = render(&image(
            &ImageProps::new("/a.png", "alt"),
            vec![("class", "attacker-controlled")],
        ));
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(!html.contains("attacker-controlled"));
    }

    #[test]
    fn alt_and_src_are_escaped() {
        let html = render(&image(
            &ImageProps::new(
                "/a.png?x=\"><script>alert(1)</script>",
                "\"><script>alert(1)</script>",
            ),
            vec![],
        ));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
    }

    #[test]
    fn dangerous_src_scheme_is_not_output_but_sibling_attrs_survive() {
        for url in [
            "javascript:alert(1)",
            "JaVaScRiPt:alert(1)",
            "data:text/html;base64,PHNjcmlwdD4=",
            "vbscript:msgbox(1)",
        ] {
            let html = render(&image(&ImageProps::new(url, "alt"), vec![]));
            assert!(
                !html.contains("src="),
                "危険な URL スキームなのに src 属性が出力されている: url={url:?}, html={html}"
            );
            assert!(html.contains(r#"alt="alt""#));
            assert!(html.contains("fd-image--fit-cover"));
        }
    }

    #[test]
    fn safe_src_passes_through() {
        for url in ["/items/1.png", "https://example.com/a.png"] {
            let html = render(&image(&ImageProps::new(url, "alt"), vec![]));
            assert!(html.contains(&format!(r#"src="{url}""#)));
        }
    }
}
