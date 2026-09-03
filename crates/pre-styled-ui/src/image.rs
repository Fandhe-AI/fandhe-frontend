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
//!
//! # イシュー #1562 の参照サイト比較
//!
//! chakra-ui `Image`（Radix Themes / Radix Primitives / ark-ui には Image
//! 部品が存在しないため参照軸は chakra-ui のみ）と 7 軸で突合し、以下を
//! 是正した:
//!
//! - **基本レイアウト**: chakra preflight の `img { display:block;
//!   max-width:100%; height:auto }` に合わせ、base へ `height: auto` を
//!   追加した（`max-width:100%` による横方向の縮小に追従して縦横比を保ち、
//!   `aspect-ratio` variant の実効に必要な前提でもある）。
//! - **バリアント**: 公式デモが角丸矩形・真円の 2 通りの角丸表現を持つ
//!   ため、[`ImageShape`] 軸（`Square`/`Rounded`/`Circle`）を新設した。
//!   [`crate::avatar::AvatarShape`]・[`crate::color_swatch::SwatchShape`] と
//!   同じ語彙（chakra 側に対応 prop は無く、これらの既存部品との統一を
//!   優先した独自命名）。角丸段はイシュー #1423 §3.1
//!   （「操作部品 md / pill full / 角無し none」）に合わせ
//!   `--fandhe-radius-none`/`-md`/`-full` トークンを使う。
//! - **aspect-ratio**: 公式デモに `aspectRatio={4/3}` の例があり、
//!   chakra テーマの aspect-ratio トークンは square/landscape(4:3)/
//!   portrait(3:4)/wide(16:9) の 4 段であるため、[`AspectRatio`] に
//!   `Landscape`(4:3)/`Portrait`(3:4) を追加した（`Video` は既存の
//!   `wide` 相当として名称を維持）。
//!
//! 以下は参照サイトの構成を踏まえたうえで、意図的に合わせなかった:
//!
//! - **サイズ**: chakra `Image` は `size` prop を持たず、寸法は
//!   `boxSize`/`w`/`h` という任意値の style prop（トークン化された段では
//!   ない）で呼び出し側が決める。本部品も寸法軸を持たず、コンテンツの
//!   実寸または呼び出し側の `attrs`（`style`/`width`/`height` 属性）に
//!   委ねる。
//! - **`align`（`object-position`）**: chakra の `align` prop 相当。
//!   呼び出し側 `attrs` の `style` で表現可能なため軸を増設しない。
//! - **色 / 状態 `data-*` / ダーク / フォーカス / hover・disabled・
//!   transition**: `<img>` は色宣言・状態変化・フォーカスを持たない
//!   静的コンテンツ表示のため、参照サイト側にも対応する挙動がない
//!   （[`crate::card`]・[`crate::skeleton`] と同型の判断）。

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

/// `aspect-ratio` variant（chakra-ui `Image` の `aspectRatio` prop 相当。
/// chakra テーマの aspect-ratio トークン square/landscape/portrait/wide の
/// 4 段に対応する）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AspectRatio {
    /// 画像本来の比率をそのまま使う（既定）。
    #[default]
    Auto,
    /// 1:1（正方形）。
    Square,
    /// 4:3（横長。chakra `landscape` 相当）。
    Landscape,
    /// 3:4（縦長。chakra `portrait` 相当）。
    Portrait,
    /// 16:9（動画サムネイル等。chakra `wide` 相当）。
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
            Self::Landscape => "landscape",
            Self::Portrait => "portrait",
            Self::Video => "video",
        }
    }
}

/// 角丸 variant（chakra-ui 公式デモの角丸矩形・真円の 2 表現を吸収する）。
/// [`crate::avatar::AvatarShape`]・[`crate::color_swatch::SwatchShape`] と
/// 同じ語彙（chakra 側に対応する prop はなく、既存部品との統一を優先した
/// 独自命名）。[`Self::Circle`] は正方形の画像（[`AspectRatio::Square`]）と
/// 組み合わせて初めて真円になり、非正方形では pill 状になる（両軸は
/// 直交する）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ImageShape {
    /// 角丸なし（既定）。
    #[default]
    Square,
    /// 中程度の角丸。
    Rounded,
    /// 完全な丸角（正方形画像と組み合わせると真円）。
    Circle,
}

impl VariantValue for ImageShape {
    fn axis(self) -> &'static str {
        "shape"
    }

    fn value(self) -> &'static str {
        match self {
            Self::Square => "square",
            Self::Rounded => "rounded",
            Self::Circle => "circle",
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
            vec![
                decl("display", "block"),
                decl("max-width", "100%"),
                decl("height", "auto"),
            ],
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
            AspectRatio::Landscape,
            "root",
            vec![decl("aspect-ratio", "4 / 3")],
        )
        .variant(
            AspectRatio::Portrait,
            "root",
            vec![decl("aspect-ratio", "3 / 4")],
        )
        .variant(
            AspectRatio::Video,
            "root",
            vec![decl("aspect-ratio", "16 / 9")],
        )
        .variant(
            ImageShape::Square,
            "root",
            vec![decl("border-radius", "var(--fandhe-radius-none)")],
        )
        .variant(
            ImageShape::Rounded,
            "root",
            vec![decl("border-radius", "var(--fandhe-radius-md)")],
        )
        .variant(
            ImageShape::Circle,
            "root",
            vec![decl("border-radius", "var(--fandhe-radius-full)")],
        )
        .default_variant(ImageFit::Cover)
        .default_variant(AspectRatio::Auto)
        .default_variant(ImageShape::Square)
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
    /// 角丸 variant（既定 `Square`）。
    pub shape: ImageShape,
}

impl<'a> ImageProps<'a> {
    /// `src`/`alt` を指定し、variant は既定値（`Cover`/`Auto`/`Square`）の
    /// まま組み立てる。
    #[must_use]
    pub fn new(src: &'a str, alt: &'a str) -> Self {
        ImageProps {
            src,
            alt,
            fit: ImageFit::default(),
            aspect_ratio: AspectRatio::default(),
            shape: ImageShape::default(),
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
        ("shape", props.shape.value()),
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
            r#"<img data-scope="image" data-part="root" class="fd-image--fit-cover fd-image--aspect-ratio-auto fd-image--shape-square" src="/a.png" alt="alt text">"#
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
                html.contains(&format!(
                    "class=\"{class} fd-image--aspect-ratio-auto fd-image--shape-square\""
                )),
                "fit={fit:?} -> {html}"
            );
        }
    }

    #[test]
    fn aspect_ratio_variants_map_to_expected_classes() {
        for (ratio, class) in [
            (AspectRatio::Auto, "fd-image--aspect-ratio-auto"),
            (AspectRatio::Square, "fd-image--aspect-ratio-square"),
            (AspectRatio::Landscape, "fd-image--aspect-ratio-landscape"),
            (AspectRatio::Portrait, "fd-image--aspect-ratio-portrait"),
            (AspectRatio::Video, "fd-image--aspect-ratio-video"),
        ] {
            let props = ImageProps {
                aspect_ratio: ratio,
                ..ImageProps::new("/a.png", "alt")
            };
            let html = render(&image(&props, vec![]));
            assert!(
                html.contains(&format!(
                    "class=\"fd-image--fit-cover {class} fd-image--shape-square\""
                )),
                "aspect_ratio={ratio:?} -> {html}"
            );
        }
    }

    #[test]
    fn shape_variants_map_to_expected_classes() {
        for (shape, class) in [
            (ImageShape::Square, "fd-image--shape-square"),
            (ImageShape::Rounded, "fd-image--shape-rounded"),
            (ImageShape::Circle, "fd-image--shape-circle"),
        ] {
            let props = ImageProps {
                shape,
                ..ImageProps::new("/a.png", "alt")
            };
            let html = render(&image(&props, vec![]));
            assert!(
                html.contains(&format!(
                    "class=\"fd-image--fit-cover fd-image--aspect-ratio-auto {class}\""
                )),
                "shape={shape:?} -> {html}"
            );
        }
    }

    #[test]
    fn css_output_declares_object_fit_and_aspect_ratio() {
        let out = css();
        assert!(out.contains("object-fit: cover;"));
        assert!(out.contains("object-fit: none;"));
        assert!(out.contains("aspect-ratio: 1 / 1;"));
        assert!(out.contains("aspect-ratio: 4 / 3;"));
        assert!(out.contains("aspect-ratio: 3 / 4;"));
        assert!(out.contains("aspect-ratio: 16 / 9;"));
        assert!(out.contains("height: auto;"));
    }

    #[test]
    fn css_output_declares_shape_radius_tokens() {
        let out = css();
        assert!(out.contains("border-radius: var(--fandhe-radius-none);"));
        assert!(out.contains("border-radius: var(--fandhe-radius-md);"));
        assert!(out.contains("border-radius: var(--fandhe-radius-full);"));
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
