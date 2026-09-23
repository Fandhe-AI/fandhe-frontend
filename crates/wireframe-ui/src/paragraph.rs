//! 複数行本文部品（`Paragraph`、イシュー #2615、Phase 2「テキスト・注釈」）。
//!
//! 画面設計図中の本文プレースホルダーを表す、非インタラクティブなローファイ
//! 部品。単一行の見出し/ラベル相当（Phase 2 の `text`、イシュー #2614）とは
//! 異なり、複数行にわたるブロック本文を表現する。
//!
//! # 呼び出し文脈
//!
//! `fandhe-frontend-docs-site` の `wireframes::paragraph` showcase
//! （`/wireframes/paragraph/`）から呼ばれる。`fandhe_frontend_core::text`
//! のみでテキストを流し込むため、既定エスケープ（REQ-1）は本モジュールが
//! 独自に保証する必要はなく core 側の契約に委譲される。
//!
//! # API 設計の由来
//!
//! blocks.pm の Paragraph 部品の Figma プロパティ構成をそのまま転写した
//! ものではなく、`docs/design/wireframe-ui-architecture.md` §6 の
//! Text/Paragraph 系共通規約（`Size` + `Bold`(bool) + `Text`(文字列) の
//! 3 点セット）から独立設計した（イシュー #2615 実装計画 §2.1）。
//! [`crate::annotation`] が `Bold` を使わず `Primary` で強調を表すのとは
//! 対になる判断で、Paragraph は表示状態軸を持たず `Bold` のみで強調を
//! 表す（`site/wireframes/paragraph.md` の「原案差分メモ」節も参照）。

use fandhe_frontend_core::{el_owned, text, Node};

use crate::class::class_list;
use crate::props::Bold;
use crate::size::Size;

/// 複数行本文 CSS（2 セレクタ）。[`crate::css::PARTS`] へ登録される。
///
/// 複数行の表現は `white-space: pre-line` のみで行う（`content` に含まれる
/// `\n` は [`fandhe_frontend_core`] の既定エスケープを素通りしてそのまま
/// 出力され、本ルールが改行として描画する）。`<br>` への変換・`raw_html`
/// の使用は行わない（REQ-1 の議論を持ち込まないための設計判断）。
pub const PARAGRAPH_CSS: &str = "\
.fw-wire-paragraph {
  display: block;
  max-width: 100%;
  box-sizing: border-box;
  margin: 0;
  color: var(--fw-wire-ink);
  font-family: var(--fw-wire-font-family);
  font-size: var(--fw-wire-font-size, 1rem);
  line-height: 1.6;
  white-space: pre-line;
  overflow-wrap: anywhere;
}
.fw-wire-paragraph.fw-wire-bold {
  font-weight: 600;
}
";

/// 複数行本文を組み立てる。
///
/// - `content`: 必須。表示する本文文言。`\n` を含めると `white-space:
///   pre-line`（CSS）により改行として描画される（マークアップ上は改行を
///   含む素のテキストノードのまま）。
/// - `size`: [`Size`] 5 段。ルート class `fw-wire-size-<段階>` として付与し、
///   フォントサイズは [`crate::size::css`] が定義する `--fw-wire-font-size`
///   を `var()` で参照する（本モジュールは値を書き写さない）。
/// - `bold`: `true` のとき class `fw-wire-bold` を付与し太字にする。
///   `Primary` は使わない（Text/Paragraph 系は `Bold` 軸のみで強調を表す）。
///
/// ルート要素は `<p>` ではなく `<div>` にする。docs サイト骨格 CSS
/// （`crates/docs-site/src/site_theme.rs` の `.docs-content p`）が詳細度
/// `(0,1,1)` を持ち `.fw-wire-paragraph`（`(0,1,0)`）に勝ってしまうため、
/// `<p>` ルートでは `Size` 軸のフォントサイズ差が Demo 上で消える
/// （`crate::annotation::annotation` も同じ理由で `<div>` ルート）。
///
/// テキストは [`fandhe_frontend_core::text`] のみで流し込み（REQ-1 既定
/// エスケープ）、`role`/`aria-*`/`tabindex`/`style`/`data-*` は一切付与
/// しない（`docs/design/wireframe-ui-architecture.md` §5/§7、Paragraph は
/// 表示状態軸を持たない部品のため `data-*` も不要）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_wireframe_ui::{paragraph, Bold, Size};
///
/// let node = paragraph("本文のダミーテキストです。", Size::Md, Bold(false));
/// let html = render(&node);
/// assert!(html.contains(r#"class="fw-wire-paragraph fw-wire-size-md""#));
/// assert!(html.starts_with("<div"));
/// assert!(html.contains("本文のダミーテキストです。"));
///
/// // Bold(true) で強調 class が付く。
/// let bold = paragraph("強調本文", Size::Md, Bold(true));
/// assert!(render(&bold).contains("fw-wire-bold"));
///
/// // 複数行は改行をそのまま残す（<br> への変換は行わない）。
/// let multiline = paragraph("1 行目\n2 行目", Size::Md, Bold(false));
/// let multiline_html = render(&multiline);
/// assert!(multiline_html.contains("1 行目\n2 行目"));
/// assert!(!multiline_html.contains("<br"));
///
/// // XSS 回帰: content は既定エスケープを経由する。
/// let escaped = paragraph("<script>alert(1)</script>", Size::Md, Bold(false));
/// let escaped_html = render(&escaped);
/// assert!(!escaped_html.contains("<script>alert(1)</script>"));
/// assert!(escaped_html.contains("&lt;script&gt;"));
/// ```
#[must_use]
pub fn paragraph(content: &str, size: Size, bold: Bold) -> Node {
    let class = class_list("fw-wire-paragraph", &[Some(size.class()), bold.class()]);

    el_owned(
        "div",
        vec![("class".to_string(), class)],
        vec![text(content)],
    )
}
