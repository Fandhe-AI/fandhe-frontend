//! SVG ノード木生成ヘルパー（イシュー #846）。
//!
//! 後続チャート部品（#847〜#851）は本モジュールのヘルパーのみを経由して
//! SVG マークアップを組み立て、`format!("<svg>...")` のような HTML/SVG
//! 文字列の直接組み立てを行わない（`.claude/rules/coding-rust.md`
//! 「HTML 文字列の直接組み立て禁止」の SVG への適用、`crates/headless-ui/src/qr_code.rs`
//! の `frame` が先例）。マークアップはすべて
//! [`fandhe_frontend_headless_ui::fandhe_frontend_core::el`] を経由するため、
//! `fandhe_frontend_core::render` の既定エスケープ（REQ-1）を必ず通る。
//!
//! # 数値の決定的文字列化（[`fmt_coord`]）
//!
//! 座標・寸法・tick 値の文字列化は本モジュールの [`fmt_coord`] にのみ実装を
//! 一元化する。規則:
//!
//! 1. `format!("{:.2}", v)`（Rust 標準の小数第 2 位への丸め）
//! 2. 小数点を含む場合、末尾の連続する `0` を除去し、続けて末尾の `.` も
//!    除去する
//! 3. 結果が `"-0"` の場合は `"0"` に正規化する（丸めにより符号なし 0 が
//!    負符号付きで出力される退化を防ぐ）
//!
//! 出力文字列の文字集合は `[0-9.-]` に閉じる（`v` が有限である限り、これ
//! 以外の文字が混入する経路はない）。呼び出し元は `v` が有限であることを
//! 事前に保証する契約とする（[`super::data::ChartData::new`]/
//! [`super::scale::LinearScale::new`] の検証を経由した値のみを渡す）。

use fandhe_frontend_headless_ui::fandhe_frontend_core::{el, Node};

/// 座標・寸法の決定的文字列化（モジュール doc の丸め規則を参照）。
///
/// `v` が非有限（`NaN`/`±inf`）の場合の出力は未規定とする（`debug_assert`
/// で開発時に検出する。呼び出し元は本関数へ到達する前に
/// [`super::data::ChartData::new`]/[`super::scale::LinearScale::new`] の
/// 検証を経由した有限値のみを渡す契約、モジュール doc 参照）。
#[must_use]
pub fn fmt_coord(v: f64) -> String {
    debug_assert!(v.is_finite(), "fmt_coord は有限値のみを契約入力とする");
    let mut s = format!("{v:.2}");
    if s.contains('.') {
        while s.ends_with('0') {
            s.pop();
        }
        if s.ends_with('.') {
            s.pop();
        }
    }
    if s == "-0" {
        s = "0".to_string();
    }
    s
}

/// SVG の `viewBox` 寸法（原点 + 幅 + 高さ）。
///
/// [`ViewBox::new`] を経由した構築のみを公開し、4 要素すべてが有限、かつ
/// `width`/`height` が正であることを保証する。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ViewBox {
    min_x: f64,
    min_y: f64,
    width: f64,
    height: f64,
}

/// [`ViewBox::new`] の検証失敗。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewBoxError {
    /// `min_x`/`min_y`/`width`/`height` のいずれかが `NaN`/`±inf`。
    NonFinite,
    /// `width`/`height` が 0 以下（描画領域として無意味な寸法）。
    NonPositiveSize,
}

impl std::fmt::Display for ViewBoxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            ViewBoxError::NonFinite => "viewBox components must be finite",
            ViewBoxError::NonPositiveSize => "viewBox width/height must be positive",
        };
        write!(f, "{message}")
    }
}

impl std::error::Error for ViewBoxError {}

impl ViewBox {
    /// `viewBox` を構築する。
    ///
    /// # Errors
    ///
    /// - 4 要素のいずれかが非有限の場合 [`ViewBoxError::NonFinite`]
    /// - `width`/`height` が 0 以下の場合 [`ViewBoxError::NonPositiveSize`]
    pub fn new(min_x: f64, min_y: f64, width: f64, height: f64) -> Result<Self, ViewBoxError> {
        if !min_x.is_finite() || !min_y.is_finite() || !width.is_finite() || !height.is_finite() {
            return Err(ViewBoxError::NonFinite);
        }
        if width <= 0.0 || height <= 0.0 {
            return Err(ViewBoxError::NonPositiveSize);
        }
        Ok(ViewBox {
            min_x,
            min_y,
            width,
            height,
        })
    }

    /// `viewBox` 属性値（`"min_x min_y width height"`）を組み立てる。
    #[must_use]
    pub fn attr_value(&self) -> String {
        format!(
            "{} {} {} {}",
            fmt_coord(self.min_x),
            fmt_coord(self.min_y),
            fmt_coord(self.width),
            fmt_coord(self.height)
        )
    }
}

/// `svg` ルート要素を組み立てる（`viewBox`・`role="img"` を既定付与する、
/// `crates/headless-ui/src/qr_code.rs` の `frame` と同型）。
///
/// `attrs` に `viewBox`/`role`（大文字小文字を無視）が含まれていても
/// 黙って除去してから既定値を付与する。[`fandhe_frontend_core::render`]
/// は同名属性の重複除去を行わない契約（`crates/core/src/lib.rs`
/// `find_attr_values` doc 参照）であるため、除去せずに連結すると
/// `viewBox`/`role` が 2 回出力される無効な HTML を生みかねない
/// （[`crate::class_attr::drop_class_attr`] と同型の判断）。
#[must_use]
pub fn svg_root(view_box: &ViewBox, attrs: Vec<(&str, &str)>, children: Vec<Node>) -> Node {
    let view_box_value = view_box.attr_value();
    let filtered = attrs
        .into_iter()
        .filter(|(k, _)| !k.eq_ignore_ascii_case("viewBox") && !k.eq_ignore_ascii_case("role"));
    let mut merged: Vec<(&str, &str)> = vec![("viewBox", view_box_value.as_str()), ("role", "img")];
    merged.extend(filtered);
    el("svg", merged, children)
}

/// `g`（グループ）要素を組み立てる。
#[must_use]
pub fn group<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    el("g", attrs, children)
}

/// `line` 要素を組み立てる（軸・グリッド線、#847 が主に使用する想定）。
#[must_use]
pub fn line<'a>(x1: f64, y1: f64, x2: f64, y2: f64, attrs: Vec<(&'a str, &'a str)>) -> Node {
    let (x1, y1, x2, y2) = (fmt_coord(x1), fmt_coord(y1), fmt_coord(x2), fmt_coord(y2));
    let mut merged: Vec<(&str, &str)> = vec![
        ("x1", x1.as_str()),
        ("y1", y1.as_str()),
        ("x2", x2.as_str()),
        ("y2", y2.as_str()),
    ];
    merged.extend(attrs);
    el("line", merged, vec![])
}

/// `rect` 要素を組み立てる（棒グラフ等、#848 以降が主に使用する想定）。
#[must_use]
pub fn rect<'a>(x: f64, y: f64, width: f64, height: f64, attrs: Vec<(&'a str, &'a str)>) -> Node {
    let (x, y, width, height) = (
        fmt_coord(x),
        fmt_coord(y),
        fmt_coord(width),
        fmt_coord(height),
    );
    let mut merged: Vec<(&str, &str)> = vec![
        ("x", x.as_str()),
        ("y", y.as_str()),
        ("width", width.as_str()),
        ("height", height.as_str()),
    ];
    merged.extend(attrs);
    el("rect", merged, vec![])
}

/// `circle` 要素を組み立てる（点マーカー等、#849 以降が主に使用する想定）。
#[must_use]
pub fn circle<'a>(cx: f64, cy: f64, r: f64, attrs: Vec<(&'a str, &'a str)>) -> Node {
    let (cx, cy, r) = (fmt_coord(cx), fmt_coord(cy), fmt_coord(r));
    let mut merged: Vec<(&str, &str)> =
        vec![("cx", cx.as_str()), ("cy", cy.as_str()), ("r", r.as_str())];
    merged.extend(attrs);
    el("circle", merged, vec![])
}

/// `text` 要素を組み立てる（軸ラベル・凡例等、#847 が主に使用する想定）。
///
/// SVG の `<text>` タグと [`fandhe_frontend_core::text`](fandhe_frontend_headless_ui::fandhe_frontend_core::text)
/// （テキストノード）の名前衝突を避けるため `svg_text` と命名する
/// （`crates/pre-styled-ui/src/qr_code.rs` モジュール doc の命名衝突回避方針
/// と同型）。`children` に渡すテキストノードは
/// `fandhe_frontend_core::render` の既定エスケープを必ず通る（REQ-1）。
#[must_use]
pub fn svg_text<'a>(x: f64, y: f64, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let (x, y) = (fmt_coord(x), fmt_coord(y));
    let mut merged: Vec<(&str, &str)> = vec![("x", x.as_str()), ("y", y.as_str())];
    merged.extend(attrs);
    el("text", merged, children)
}

/// `path` の `d` 属性値を組み立てるビルダー（折れ線・領域グラフ、
/// #849〜#851 が主に使用する想定）。
///
/// 出力文字列の文字集合は `M`（moveto）・`L`（lineto）・`Z`（closepath）・
/// 半角スペース・半角数字・`.`・`-`・座標区切りの `,` に閉じる（座標は
/// [`fmt_coord`] のみを経由するため、任意文字列の混入経路を持たない）。
#[derive(Debug, Clone, Default)]
pub struct PathBuilder {
    segments: Vec<String>,
}

impl PathBuilder {
    /// 空のパスビルダーを作る。
    #[must_use]
    pub fn new() -> Self {
        PathBuilder::default()
    }

    /// `M x,y`（moveto）セグメントを追加する。
    #[must_use]
    pub fn move_to(mut self, x: f64, y: f64) -> Self {
        self.segments
            .push(format!("M{},{}", fmt_coord(x), fmt_coord(y)));
        self
    }

    /// `L x,y`（lineto）セグメントを追加する。
    #[must_use]
    pub fn line_to(mut self, x: f64, y: f64) -> Self {
        self.segments
            .push(format!("L{},{}", fmt_coord(x), fmt_coord(y)));
        self
    }

    /// `Z`（closepath）セグメントを追加する。
    #[must_use]
    pub fn close(mut self) -> Self {
        self.segments.push("Z".to_string());
        self
    }

    /// 積み上げたセグメントを 1 個のスペース区切り `d` 属性値として返す。
    #[must_use]
    pub fn build(self) -> String {
        self.segments.join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};

    fn is_closed_charset(s: &str) -> bool {
        s.chars().all(|c| {
            c.is_ascii_digit()
                || c == '.'
                || c == '-'
                || c == ' '
                || c == ','
                || matches!(c, 'M' | 'L' | 'Z')
        })
    }

    #[test]
    fn fmt_coord_strips_trailing_zeros_and_dot() {
        assert_eq!(fmt_coord(0.0), "0");
        assert_eq!(fmt_coord(1.0), "1");
        assert_eq!(fmt_coord(1.5), "1.5");
        assert_eq!(fmt_coord(1.25), "1.25");
        assert_eq!(fmt_coord(100.0), "100");
        assert_eq!(fmt_coord(0.1), "0.1");
    }

    #[test]
    fn fmt_coord_normalizes_negative_zero() {
        assert_eq!(fmt_coord(-0.0), "0");
        // 丸め後に -0.00 へ潰れる微小負値も正規化する。
        assert_eq!(fmt_coord(-0.001), "0");
    }

    #[test]
    fn fmt_coord_handles_negative_values() {
        assert_eq!(fmt_coord(-1.5), "-1.5");
        assert_eq!(fmt_coord(-100.0), "-100");
    }

    #[test]
    fn fmt_coord_output_charset_is_closed() {
        for v in [0.0, -0.0, 1.0, -1.5, 123.456, -123.456, 0.001, 99999.99] {
            let s = fmt_coord(v);
            assert!(
                s.chars()
                    .all(|c| c.is_ascii_digit() || c == '.' || c == '-'),
                "unexpected char in {s:?}"
            );
        }
    }

    #[test]
    fn fmt_coord_is_deterministic() {
        for v in [3.14259, -2.71928, 0.0, 42.0] {
            assert_eq!(fmt_coord(v), fmt_coord(v));
        }
    }

    #[test]
    fn view_box_rejects_non_finite_or_non_positive_size() {
        assert_eq!(
            ViewBox::new(f64::NAN, 0.0, 10.0, 10.0).unwrap_err(),
            ViewBoxError::NonFinite
        );
        assert_eq!(
            ViewBox::new(0.0, 0.0, 0.0, 10.0).unwrap_err(),
            ViewBoxError::NonPositiveSize
        );
        assert_eq!(
            ViewBox::new(0.0, 0.0, -1.0, 10.0).unwrap_err(),
            ViewBoxError::NonPositiveSize
        );
    }

    #[test]
    fn view_box_attr_value_matches_expected_format() {
        let vb = ViewBox::new(0.0, 0.0, 100.0, 50.0).unwrap();
        assert_eq!(vb.attr_value(), "0 0 100 50");
    }

    #[test]
    fn svg_root_sets_view_box_and_role_and_drops_caller_duplicates() {
        let vb = ViewBox::new(0.0, 0.0, 100.0, 100.0).unwrap();
        let node = svg_root(&vb, vec![], vec![]);
        let html = render(&node);
        assert!(html.contains(r#"viewBox="0 0 100 100""#));
        assert!(html.contains(r#"role="img""#));

        // 呼び出し側が `role`/`viewBox` を偽装しても既定値のみが単一出現する
        // （重複属性による無効な HTML を防ぐ）。
        let spoofed = svg_root(
            &vb,
            vec![("role", "presentation"), ("viewBox", "0 0 1 1")],
            vec![],
        );
        let html = render(&spoofed);
        assert_eq!(html.matches("role=").count(), 1);
        assert_eq!(html.matches("viewBox=").count(), 1);
        assert!(html.contains(r#"role="img""#));
        assert!(html.contains(r#"viewBox="0 0 100 100""#));
    }

    #[test]
    fn line_rect_circle_svg_text_render_expected_attrs() {
        let line_html = render(&line(0.0, 1.5, 10.0, -1.5, vec![]));
        assert!(line_html.contains(r#"x1="0""#));
        assert!(line_html.contains(r#"y1="1.5""#));
        assert!(line_html.contains(r#"x2="10""#));
        assert!(line_html.contains(r#"y2="-1.5""#));

        let rect_html = render(&rect(1.0, 2.0, 3.0, 4.0, vec![]));
        assert!(rect_html.contains(r#"x="1""#));
        assert!(rect_html.contains(r#"width="3""#));

        let circle_html = render(&circle(5.0, 6.0, 7.0, vec![]));
        assert!(circle_html.contains(r#"cx="5""#));
        assert!(circle_html.contains(r#"r="7""#));

        let text_html = render(&svg_text(1.0, 2.0, vec![], vec![text("<script>")]));
        assert!(text_html.contains(r#"x="1""#));
        assert!(text_html.contains("&lt;script&gt;"));
        assert!(!text_html.contains("<script>"));
    }

    #[test]
    fn path_builder_produces_closed_charset_d_attribute() {
        let d = PathBuilder::new()
            .move_to(0.0, 0.0)
            .line_to(10.5, -5.25)
            .line_to(20.0, 0.0)
            .close()
            .build();
        assert_eq!(d, "M0,0 L10.5,-5.25 L20,0 Z");
        assert!(is_closed_charset(&d));
    }

    #[test]
    fn path_builder_empty_produces_empty_string() {
        assert_eq!(PathBuilder::new().build(), "");
    }
}
