//! 単一行テキスト部品（`Text`、イシュー #2614、Phase 2「テキスト・注釈」）。
//!
//! 画面設計図中の見出し・ラベル・短い文言を表す、非インタラクティブな
//! ローファイ部品。複数行本文（[`crate::paragraph`]、イシュー #2615）とは
//! 異なり、1 行に固定表示し、幅を超える文言は末尾を省略記号で切る点が
//! 独自の判断である。
//!
//! # 呼び出し文脈
//!
//! `fandhe-frontend-docs-site` の `wireframes::text` showcase
//! （`/wireframes/text/`）から呼ばれる。`fandhe_frontend_core::text`
//! のみでテキストを流し込むため、既定エスケープ（REQ-1）は本モジュールが
//! 独自に保証する必要はなく core 側の契約に委譲される。
//!
//! # API 設計の由来
//!
//! blocks.pm の Text 部品の Figma プロパティ構成をそのまま転写したもの
//! ではなく、`docs/design/wireframe-ui-architecture.md` §6 の
//! Text/Paragraph 系共通規約（`Size` + `Bold`(bool) + `Text`(文字列) の
//! 3 点セット）から独立設計した（[`crate::paragraph::paragraph`] と同型）。
//! [`crate::annotation`] が `Bold` を使わず `Primary` で強調を表すのとは
//! 対になる判断で、Text は表示状態軸を持たず `Bold` のみで強調を表す。

use fandhe_frontend_core::{el_owned, text as text_node, Node};

use crate::class::class_list;
use crate::props::Bold;
use crate::size::Size;

/// 単一行テキスト CSS（2 セレクタ）。[`crate::css::PARTS`] へ登録される。
///
/// 1 行固定表示は `white-space: nowrap` + `overflow: hidden` +
/// `text-overflow: ellipsis` のみで行う（[`crate::paragraph::PARAGRAPH_CSS`]
/// の `white-space: pre-line`・複数行許容とは対になる判断）。
pub const TEXT_CSS: &str = "\
.fw-wire-text {
  display: inline-block;
  max-width: 100%;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  vertical-align: bottom;
  color: var(--fw-wire-ink);
  font-family: var(--fw-wire-font-family);
  font-size: var(--fw-wire-font-size, 1rem);
  line-height: 1.4;
}
.fw-wire-text.fw-wire-bold {
  font-weight: 600;
}
";

/// 単一行テキストを組み立てる。
///
/// - `content`: 必須。表示する文言。1 行固定表示のため、幅を超える場合は
///   CSS（`text-overflow: ellipsis`）が末尾を省略記号で切る（マークアップ上
///   は改行・切り詰めのない素のテキストノードのまま）。
/// - `size`: [`Size`] 5 段。ルート class `fw-wire-size-<段階>` として付与し、
///   フォントサイズは [`crate::size::css`] が定義する `--fw-wire-font-size`
///   を `var()` で参照する（本モジュールは値を書き写さない）。
/// - `bold`: `true` のとき class `fw-wire-bold` を付与し太字にする。
///   `Primary` は使わない（Text/Paragraph 系は `Bold` 軸のみで強調を表す）。
///
/// ルート要素は `<span>` にする（インラインの 1 行要素であることを示す
/// ため）。docs サイト骨格 CSS（`crates/docs-site/src/site_theme.rs`）には
/// `.docs-content span` 規則が存在しないため、[`crate::paragraph::paragraph`]
/// が `<p>` を避けた理由（`.docs-content p` の詳細度負け）は本部品には
/// 当てはまらない。
///
/// テキストは [`fandhe_frontend_core::text`] のみで流し込み（REQ-1 既定
/// エスケープ）、`role`/`aria-*`/`tabindex`/`style`/`data-*` は一切付与
/// しない（`docs/design/wireframe-ui-architecture.md` §5/§7、Text は
/// 表示状態軸を持たない部品のため `data-*` も不要）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_wireframe_ui::{text, Bold, Size};
///
/// let node = text("見出しのダミーテキスト", Size::Md, Bold(false));
/// let html = render(&node);
/// assert!(html.contains(r#"class="fw-wire-text fw-wire-size-md""#));
/// assert!(html.starts_with("<span"));
/// assert!(html.contains("見出しのダミーテキスト"));
///
/// // Bold(true) で強調 class が付く。
/// let bold = text("強調テキスト", Size::Md, Bold(true));
/// assert!(render(&bold).contains("fw-wire-bold"));
///
/// // XSS 回帰: content は既定エスケープを経由する。
/// let escaped = text("<script>alert(1)</script>", Size::Md, Bold(false));
/// let escaped_html = render(&escaped);
/// assert!(!escaped_html.contains("<script>alert(1)</script>"));
/// assert!(escaped_html.contains("&lt;script&gt;"));
/// ```
#[must_use]
pub fn text(content: &str, size: Size, bold: Bold) -> Node {
    let class = class_list("fw-wire-text", &[Some(size.class()), bold.class()]);

    el_owned(
        "span",
        vec![("class".to_string(), class)],
        vec![text_node(content)],
    )
}
