//! テキストリンク部品（`Link`、イシュー #2618、Phase 2「テキスト・注釈」）。
//!
//! 下線付きテキスト + 任意の末尾アイコン（外部リンク等）で「リンクらしさ」
//! だけを表現する非インタラクティブなローファイ・プレースホルダー。
//!
//! # 呼び出し文脈
//!
//! `fandhe-frontend-docs-site` の `wireframes::link` showcase
//! （`/wireframes/link/`）から呼ばれる。`fandhe_frontend_core::text` のみで
//! テキストを流し込むため、既定エスケープ（REQ-1）は本モジュールが独自に
//! 保証する必要はなく core 側の契約に委譲される。末尾アイコンスロットは
//! [`crate::icon`] の関数の戻り値（`Node`）をそのまま受け取る。
//!
//! # API 設計の由来
//!
//! blocks.pm の Link 部品の Figma プロパティ構成（`External`/`Icon` 相当の
//! bool トグル群）をそのまま転写したものではなく、
//! `docs/design/wireframe-ui-architecture.md` §6 の汎用変換規約と §11.4 の
//! `Option<Node>` アイコンスロット規約から独立設計した（イシュー #2618
//! 実装計画 §2、`site/wireframes/link.md` の「原案差分メモ」節も参照）。
//! 外部リンク表示は呼び出し側が `Some(icon::external(size))` を渡すことで
//! 表現し、`External` 型・第 2 の bool 引数・`external_link` のような
//! 便宜ラッパは追加しない（`Some(icon::external(size))` で十分表現できる
//! ため、公開面を増やさない判断）。
//!
//! # `a[href]` は出力しない（最重要）
//!
//! `docs/design/wireframe-ui-architecture.md` §7（非対話制約）に従い、
//! ルートは `span` とし `href`/`rel`/`target` は一切出力しない。実際に
//! 遷移可能なリンクが必要な利用者には Themes の Link（`/themes/link/`）を
//! 案内する（`site/wireframes/link.md` 参照）。

use fandhe_frontend_core::{el_owned, span, text, Node};

use crate::class::class_list;
use crate::props::Bold;
use crate::size::Size;

/// ラベルのパート class（部品ルートなしで単独使用しない、[`link`] 専用）。
const LABEL_CLASS: &str = "fw-wire-link-label";

/// リンク CSS（3 セレクタ）。[`crate::css::PARTS`] へ登録される。
///
/// 下線はルート（`inline-flex`）ではなくラベルパート側に置く。flex 子への
/// `text-decoration` 伝播のブラウザ差を避け、末尾アイコン（SVG）に下線が
/// 乗らないようにするため。
pub const LINK_CSS: &str = "\
.fw-wire-link {
  display: inline-flex;
  align-items: center;
  gap: 0.25em;
  color: var(--fw-wire-ink);
  font-family: var(--fw-wire-font-family);
  font-size: var(--fw-wire-font-size, 1rem);
  line-height: 1.4;
}
.fw-wire-link-label {
  text-decoration: underline;
  text-decoration-color: var(--fw-wire-line);
  text-underline-offset: 0.15em;
}
.fw-wire-link.fw-wire-bold {
  font-weight: 600;
}
";

/// テキストリンクを組み立てる。
///
/// - `label`: 必須。下線付きで表示するリンク文言。
/// - `trailing`: 省略可能な末尾アイコンスロット（`docs/design/wireframe-ui-architecture.md`
///   §11.4）。外部リンク表示は `Some(icon::external(size))` を渡す。`None`
///   のときはアイコン要素自体を出力しない。
/// - `size`: [`Size`] 5 段。ルート class `fw-wire-size-<段階>` として付与し、
///   フォントサイズは [`crate::size::css`] が定義する `--fw-wire-font-size`
///   を `var()` で参照する（本モジュールは値を書き写さない）。
/// - `bold`: `true` のとき `fw-wire-bold` を付与する。`Primary`/`Active`/
///   `Disabled` は使わない（Link に強調反転・表示状態軸を持たせる根拠が
///   ないため）。
///
/// テキストは [`fandhe_frontend_core::text`] のみで流し込み（REQ-1 既定
/// エスケープ）、`role`/`aria-*`/`tabindex`/`style`/`data-*`（部品側）・
/// `href`/`rel`/`target` は一切付与しない。`trailing` に渡した `Node`
/// （例: [`crate::icon::external`]）が持つ `data-icon` 属性はアイコン基盤
/// 側の識別子であり、部品側の出力ではない。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_wireframe_ui::{icon, link, Bold, Size};
///
/// let node = link("詳細を見る", None, Size::Md, Bold(false));
/// let html = render(&node);
/// assert!(html.contains(r#"class="fw-wire-link fw-wire-size-md""#));
/// assert!(html.contains(r#"class="fw-wire-link-label""#));
/// assert!(html.contains("詳細を見る"));
/// assert!(!html.contains("<svg"));
///
/// // 外部リンクアイコン付き。
/// let external = link("外部サイト", Some(icon::external(Size::Md)), Size::Md, Bold(false));
/// let external_html = render(&external);
/// assert!(external_html.contains("<svg"));
///
/// // `href`/`a` は一切出力しない（`docs/design/wireframe-ui-architecture.md` §7）。
/// assert!(!external_html.contains("<a "));
/// assert!(!external_html.contains("href="));
///
/// // XSS 回帰: ラベルは既定エスケープを経由する。
/// let escaped = link("<script>alert(1)</script>", None, Size::Md, Bold(false));
/// let escaped_html = render(&escaped);
/// assert!(!escaped_html.contains("<script>alert(1)</script>"));
/// assert!(escaped_html.contains("&lt;script&gt;"));
/// ```
#[must_use]
pub fn link(label: &str, trailing: Option<Node>, size: Size, bold: Bold) -> Node {
    let class = class_list("fw-wire-link", &[Some(size.class()), bold.class()]);

    let mut children: Vec<Node> = vec![span(vec![("class", LABEL_CLASS)], vec![text(label)])];
    if let Some(trailing) = trailing {
        children.push(trailing);
    }

    el_owned("span", vec![("class".to_string(), class)], children)
}
