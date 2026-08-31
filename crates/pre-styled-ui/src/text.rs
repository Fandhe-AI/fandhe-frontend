//! Text（イシュー #771）: 単一 recipe styled 部品。段落テキスト（`<p>`）を
//! `size`/`weight` variant 付きで組み立てる。[`crate::heading`] と対になる
//! 本文向け静的部品（headless 状態機械を要しない、badge/skeleton と同型）。
//!
//! # 呼び出し側の名前衝突に関する注意
//!
//! この関数名 [`text`] は [`fandhe_frontend_core::text`]（テキストノード
//! 生成関数）と同名である。両方を同一スコープへ `use` する場合は呼び出し側
//! でモジュールパスを使う（`text::text(...)`）か `use ... as` でどちらかを
//! 別名にする必要がある。本クレートの他モジュール（例:
//! `fandhe_frontend_pre_styled_ui::input`）も `fandhe_frontend_core::text`
//! （子ノードのテキスト）と styled 部品を同時に使うため、この衝突は本部品
//! 固有の問題ではない。
//!
//! # colorPalette 軸を持たない理由
//!
//! [`crate::heading`] と同じ（中立部品、前景色トークンを継承する）。
//!
//! # prose（記事全体カスケード）とこの部品群の役割分担（イシュー #771）
//!
//! chakra-ui の `Prose`（記事全体へ一括カスケード適用するコンポーネント）
//! に相当する機構は、本クレート（`fandhe-frontend-pre-styled-ui`）へは
//! 導入しない。[`mod@heading`]/[`mod@text`]/[`mod@em`]/[`mod@mark`]/
//! [`mod@blockquote`]/[`mod@list`] はいずれも「要素単位のオプトイン適用」
//! （呼び出し側が明示的に呼んだ要素にのみ既定スタイルが付く）であり、
//! Markdown 由来の記事本文へ無選別にカスケード適用する仕組みは持たない。
//!
//! 記事全体へのカスケードスタイルは `fandhe-frontend-docs-site` の
//! `site/assets/site.css`（`.docs-content` 配下の `h1`-`h3`/`p`/`ul`/`ol`/
//! `blockquote` 規則）が既に担っており、本イシューはこの既存機構を置き換え
//! ない（`docs/design/component-coverage-map.md` prose.md 行の「対象外」
//! 区分の根拠）。両者の役割は次のように分かれる:
//!
//! - `site.css` の `.docs-content` 規則: Markdown から生成した記事本文全体へ
//!   無条件にカスケード適用する、docs サイト固有の骨格スタイル
//! - 本モジュール群: 呼び出し側が Rust コードで明示的に組み立てる UI 部品
//!   として、要素単位で個別に呼び出して使う styled 部品
//!
//! 将来 `fandhe-frontend-pre-styled-ui` 側にも prose 相当の一括適用機構を
//! 追加する場合は、`site.css` との重複適用・詳細度の衝突を新たな設計課題
//! として扱う必要があり、本イシューのスコープには含めない。
//!
//! ## 参考サイト基準との 7 軸比較（イシュー #1442）
//!
//! chakra-ui / Radix Themes の Text 相当部品とサイズ・バリアント・色・
//! 状態・ダーク・フォーカス・余白 / hover / disabled / transition の
//! 7 軸で比較した結果（スクショは
//! `docs/design/reference-screenshots/{chakra,radixt}-text-*.png`）:
//!
//! - **サイズ軸**: chakra の Text サイズ一覧スクショ（`chakra-text-2.png`）は
//!   `xs` 相当の極小から大型見出し級までの広い段階を持つ。旧実装は
//!   `xs`〜`xl` の 5 段階までしかなく、[`crate::heading`] が既に持つ
//!   `xl2`/`xl3`/`xl4`（[`crate::theme`] のテーマトークン `font-size-2xl`〜
//!   `font-size-4xl` に対応）に相当する段が Text 側には欠落していた。本
//!   イシューで [`TextSize::Xl2`]/[`TextSize::Xl3`]/[`TextSize::Xl4`] を
//!   追加し、[`crate::heading`] と同じくテーマトークン全 8 段
//!   （`xs`〜`4xl`）を網羅する形にした（上端の非採用範囲・再評価トリガーは
//!   [`crate::heading`] のモジュール rustdoc「サイズトークンの縮約」節と
//!   同じ）。
//! - **weight 軸**: chakra の Text はフォントウェイトのバリエーション
//!   （`chakra-text-3.png` で通常〜太字までの複数段を確認）を持ち、
//!   Radix Themes の Text も `weight` prop（light/regular/medium/bold）を
//!   持つ。旧実装は weight 軸を持たず、常に本文の font-weight を継承する
//!   のみだった。[`crate::theme`] は `font-weight-normal`/`-medium`/
//!   `-semibold`/`-bold` の 4 トークンを既に持つため、トークン新設なしで
//!   [`TextWeight`] 軸（`Normal`/`Medium`/`Semibold`/`Bold`。値名は
//!   参照サイトの `light`/`regular` ではなく本リポジトリ既存のトークン
//!   語彙に合わせる）を追加した。既定は `Normal`（両サイトの既定と一致）。
//! - **バリアント軸**: 両サイト共に Text へ `variant`（solid/subtle 等）
//!   prop を持たない。当部品も軸を追加しない。
//! - **色**: 両サイト共に前景色を継承する中立部品として実装されており、
//!   `colorPalette` 相当の軸を持たない。当部品も一致（前節「colorPalette
//!   軸を持たない理由」参照）。
//! - **状態（data-*）**: text は headless 状態機械を持たない静的部品であり、
//!   両サイトの Text も操作状態（`data-state` 等）を持たない。一致。
//! - **ダーク**: 色宣言を持たず本文色に自動追従するため、両サイトと同様
//!   ライト / ダーク双方で自動的に一致する。
//! - **フォーカス / hover / disabled / transition**: text は非インタラ
//!   クティブな表示専用 slot であり、両サイトの Text もこれらの状態を
//!   持たない。本フレームワークでも hover / disabled / transition は
//!   インタラクティブ slot のみに適用する方針
//!   （`docs/design/pre-styled-ui-interaction-visual-language.md` §3）
//!   であり、フォーカスリングもフォーカス対象部品限定
//!   （`docs/design/pre-styled-ui-focus-ring-and-size-conventions.md` §3）
//!   のため付与しない（一致）。
//! - **余白**: `margin: 0`（余白は利用者責務）は chakra の marginless 方針と
//!   一致。現状維持。
//! - **align / trim / truncate / wrap / `as` prop 相当（Radix 固有のレイ
//!   アウトユーティリティ prop）**: `radixt-text-3.png` が示す `as` prop
//!   （`p`/`label`/`div`/`span` の切り替え）を含め非採用とする。本部品は
//!   要素単位の styled 部品という設計であり、[`crate::heading`] の
//!   `HeadingLevel` のように意味論選択が構造上必然な軸ではないため、
//!   タグは `<p>` 固定のまま変更しない。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{SlotRecipe, VariantValue};
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
use fandhe_frontend_headless_ui::{anatomy, Anatomy};

/// `data-scope="text"` を固定した本コンポーネントの anatomy。
const ANATOMY: Anatomy = anatomy("text");

/// Text の視覚サイズ variant（`font-size`/`line-height`。[`crate::heading`]
/// と同じくテーマトークンの範囲に合わせ `xs`〜`4xl` の 8 段階を持つ）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextSize {
    /// 極小サイズ。
    Xs,
    /// 小サイズ。
    Sm,
    /// 中サイズ（既定）。
    #[default]
    Md,
    /// 大サイズ。
    Lg,
    /// 特大サイズ。
    Xl,
    /// 2 段階特大（イシュー #1442 で追加。モジュール rustdoc「参考サイト
    /// 基準との 7 軸比較」節参照）。
    Xl2,
    /// 3 段階特大（イシュー #1442 で追加）。
    Xl3,
    /// 4 段階特大（テーマトークンが持つ最大サイズ。イシュー #1442 で追加）。
    Xl4,
}

impl VariantValue for TextSize {
    fn axis(self) -> &'static str {
        "size"
    }

    fn value(self) -> &'static str {
        match self {
            Self::Xs => "xs",
            Self::Sm => "sm",
            Self::Md => "md",
            Self::Lg => "lg",
            Self::Xl => "xl",
            // `is_valid_identifier`（`crates/pre-styled-ui/src/css.rs`）は先頭文字に
            // ASCII 小文字のみを許容し数字始まりを拒否するため、[`crate::heading`]
            // の `HeadingSize` と同じく `xl2`/`xl3`/`xl4`（enum variant 名と一致する
            // 表記）を使う。
            Self::Xl2 => "xl2",
            Self::Xl3 => "xl3",
            Self::Xl4 => "xl4",
        }
    }
}

/// Text のフォントウェイト variant（イシュー #1442 で追加。モジュール
/// rustdoc「参考サイト基準との 7 軸比較」節参照）。既定 `Normal` は本文の
/// 継承ウェイトと視覚的に一致する明示宣言であり、CSS 上は他軸と同じく
/// 常にクラスが付与される（[`crate::recipe::SlotRecipe::default_variant`]
/// の規約どおり）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextWeight {
    /// 通常ウェイト（既定）。
    #[default]
    Normal,
    /// 中間ウェイト。
    Medium,
    /// やや太いウェイト。
    Semibold,
    /// 太字ウェイト。
    Bold,
}

impl VariantValue for TextWeight {
    fn axis(self) -> &'static str {
        "weight"
    }

    fn value(self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Medium => "medium",
            Self::Semibold => "semibold",
            Self::Bold => "bold",
        }
    }
}

/// [`text`] の設定。
#[derive(Debug, Clone, Copy, Default)]
pub struct TextProps {
    /// 視覚サイズ variant（既定 `Md`）。
    pub size: TextSize,
    /// フォントウェイト variant（既定 `Normal`。イシュー #1442 で追加）。
    pub weight: TextWeight,
}

/// Text の recipe（scope `"text"`、slot `"root"` のみ、`<p>` 固定）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("text", &["root"])
        .base("root", vec![decl("margin", "0")])
        .variant(
            TextSize::Xs,
            "root",
            vec![
                decl("font-size", "var(--fandhe-font-font-size-xs)"),
                decl("line-height", "1.4"),
            ],
        )
        .variant(
            TextSize::Sm,
            "root",
            vec![
                decl("font-size", "var(--fandhe-font-font-size-sm)"),
                decl("line-height", "1.45"),
            ],
        )
        .variant(
            TextSize::Md,
            "root",
            vec![
                decl("font-size", "var(--fandhe-font-font-size-md)"),
                decl("line-height", "1.5"),
            ],
        )
        .variant(
            TextSize::Lg,
            "root",
            vec![
                decl("font-size", "var(--fandhe-font-font-size-lg)"),
                decl("line-height", "1.5"),
            ],
        )
        .variant(
            TextSize::Xl,
            "root",
            vec![
                decl("font-size", "var(--fandhe-font-font-size-xl)"),
                decl("line-height", "1.55"),
            ],
        )
        .variant(
            TextSize::Xl2,
            "root",
            vec![
                decl("font-size", "var(--fandhe-font-font-size-2xl)"),
                decl("line-height", "1.5"),
            ],
        )
        .variant(
            TextSize::Xl3,
            "root",
            vec![
                decl("font-size", "var(--fandhe-font-font-size-3xl)"),
                decl("line-height", "1.45"),
            ],
        )
        .variant(
            TextSize::Xl4,
            "root",
            vec![
                decl("font-size", "var(--fandhe-font-font-size-4xl)"),
                decl("line-height", "1.4"),
            ],
        )
        .default_variant(TextSize::Md)
        .variant(
            TextWeight::Normal,
            "root",
            vec![decl("font-weight", "var(--fandhe-font-font-weight-normal)")],
        )
        .variant(
            TextWeight::Medium,
            "root",
            vec![decl("font-weight", "var(--fandhe-font-font-weight-medium)")],
        )
        .variant(
            TextWeight::Semibold,
            "root",
            vec![decl(
                "font-weight",
                "var(--fandhe-font-font-weight-semibold)",
            )],
        )
        .variant(
            TextWeight::Bold,
            "root",
            vec![decl("font-weight", "var(--fandhe-font-font-weight-bold)")],
        )
        .default_variant(TextWeight::Normal)
}

/// Text の静的 CSS 全文。
#[must_use]
pub fn css() -> String {
    recipe().css()
}

/// Text 1 個（`<p>`）を組み立てる。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::{render, text as core_text};
/// use fandhe_frontend_pre_styled_ui::text::{text, TextProps};
///
/// let node = text(&TextProps::default(), vec![], vec![core_text("Body copy")]);
/// let html = render(&node);
/// assert!(html.starts_with("<p"));
/// assert!(html.contains("Body copy"));
/// ```
#[must_use]
pub fn text<'a>(props: &TextProps, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let recipe = recipe();
    let class = recipe.variant_classes(&[
        ("size", props.size.value()),
        ("weight", props.weight.value()),
    ]);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    ANATOMY.part("root", "p", merged, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text as core_text};

    #[test]
    fn default_props_render_p_md_size() {
        let html = render(&text(
            &TextProps::default(),
            vec![],
            vec![core_text("Body")],
        ));
        assert_eq!(
            html,
            r#"<p data-scope="text" data-part="root" class="fd-text--size-md fd-text--weight-normal">Body</p>"#
        );
    }

    #[test]
    fn size_enumeration_maps_to_expected_classes() {
        for (size, class) in [
            (TextSize::Xs, "fd-text--size-xs"),
            (TextSize::Sm, "fd-text--size-sm"),
            (TextSize::Md, "fd-text--size-md"),
            (TextSize::Lg, "fd-text--size-lg"),
            (TextSize::Xl, "fd-text--size-xl"),
            (TextSize::Xl2, "fd-text--size-xl2"),
            (TextSize::Xl3, "fd-text--size-xl3"),
            (TextSize::Xl4, "fd-text--size-xl4"),
        ] {
            let props = TextProps {
                size,
                ..TextProps::default()
            };
            let html = render(&text(&props, vec![], vec![]));
            assert!(
                html.contains(&format!("class=\"{class} ")),
                "size={size:?} -> {html}"
            );
        }
    }

    #[test]
    fn weight_enumeration_maps_to_expected_classes() {
        for (weight, class) in [
            (TextWeight::Normal, "fd-text--weight-normal"),
            (TextWeight::Medium, "fd-text--weight-medium"),
            (TextWeight::Semibold, "fd-text--weight-semibold"),
            (TextWeight::Bold, "fd-text--weight-bold"),
        ] {
            let props = TextProps {
                weight,
                ..TextProps::default()
            };
            let html = render(&text(&props, vec![], vec![]));
            assert!(
                html.contains(&format!(" {class}\"")),
                "weight={weight:?} -> {html}"
            );
        }
    }

    #[test]
    fn caller_class_attr_is_dropped_not_duplicated() {
        let html = render(&text(
            &TextProps::default(),
            vec![("class", "attacker-controlled")],
            vec![],
        ));
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(!html.contains("attacker-controlled"));
    }

    #[test]
    fn xss_payload_in_children_is_escaped() {
        let html = render(&text(
            &TextProps::default(),
            vec![],
            vec![core_text("<script>alert(1)</script>")],
        ));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
    }

    #[test]
    fn css_output_declares_all_size_font_tokens() {
        let out = css();
        for token in [
            "font-size-xs",
            "font-size-sm",
            "font-size-md",
            "font-size-lg",
            "font-size-xl",
            "font-size-2xl",
            "font-size-3xl",
            "font-size-4xl",
        ] {
            assert!(
                out.contains(&format!("var(--fandhe-font-{token})")),
                "missing {token} in {out}"
            );
        }
    }

    #[test]
    fn css_output_declares_all_weight_font_tokens() {
        let out = css();
        for token in [
            "font-weight-normal",
            "font-weight-medium",
            "font-weight-semibold",
            "font-weight-bold",
        ] {
            assert!(
                out.contains(&format!("var(--fandhe-font-{token})")),
                "missing {token} in {out}"
            );
        }
    }
}
