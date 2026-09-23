//! ファイルドロップ領域部品（`File drop`、イシュー #2633、Phase 4「Forms B」）。
//!
//! 点線枠 + 任意のアイコン + 説明文 + 任意のヒントで「ファイルをドラッグ
//! ＆ドロップする領域」の配置イメージだけを示す、非インタラクティブな
//! ローファイ・プレースホルダー。
//!
//! # 呼び出し文脈
//!
//! `fandhe-frontend-docs-site` の `wireframes::file_drop` showcase
//! （`/wireframes/file-drop/`）から呼ばれる。`fandhe_frontend_core::text`
//! のみでテキストを流し込むため、既定エスケープ（REQ-1）は本モジュールが
//! 独自に保証する必要はなく core 側の契約に委譲される。
//!
//! # API 設計の由来
//!
//! `file-drop` は blocks.pm に対応する部品を持たない、wireframe-ui
//! 独自追加の 14 部品のひとつである（`docs/design/wireframe-ui-architecture.md`
//! §8）。このため blocks.pm の Figma プロパティを転写する対象がなく、
//! 引数構成は同文書 §6 の汎用変換規約から独立設計した
//! （`site/wireframes/file-drop.md` の「原案差分メモ」節も参照）。
//!
//! # アイコンは `Option<Node>` スロット
//!
//! [`crate::icon`]（イシュー #2633 時点で 21 種）にはアップロード専用の
//! アイコンがない。新しいアイコンを追加すると、アイコン基盤・
//! `tests/icon.rs`・CLAUDE.md の件数表記など影響範囲が広がり本イシューの
//! スコープを超えるため、
//! `link`（イシュー #2618）と同じ `Option<Node>` アイコンスロット規約
//! （`docs/design/wireframe-ui-architecture.md` §11.4）を採用する。呼び
//! 出し側は [`crate::icon::image`] や [`crate::icon::plus`] 等、任意の
//! 既存アイコンを渡せる。`None` のときはアイコンのパート要素自体を
//! 出力しない。
//!
//! # 表示状態軸を持たない
//!
//! 「ドラッグ中」はインタラクションの状態であり、非対話層（同文書 §7）
//! の責務外である。このため本部品は `Bold`/`Primary`/`Active`/`Disabled`
//! のいずれも受け取らず、ルート・子要素のいずれにも `data-*` を付与
//! しない（アイコンスロットに渡した `Node` が持つ `data-icon` はアイコン
//! 基盤側の識別子であり、そのまま透過する）。
//!
//! # `<input type="file">`・`<label>` 等は出力しない（最重要）
//!
//! `docs/design/wireframe-ui-architecture.md` §7（非対話制約）に従い、
//! `<input>`/`<label>`/`<button>`/`<form>`/`<a href>`・`draggable`/
//! `ondrop`/`ondragover` 等のドラッグイベント・`accept`/`multiple` 等の
//! ファイル入力属性・`role`/`aria-*`/`tabindex`/`style` は一切出力しない。
//! 実際に操作可能なファイルアップロードが必要な利用者には Themes
//! （`/themes/file-upload/`）/ Primitives（`/primitives/file-upload/`）を
//! 案内する（`site/wireframes/file-drop.md` 参照）。

use fandhe_frontend_core::{div, el_owned, text, Node};

use crate::class::class_list;
use crate::size::Size;

/// パート class（部品ルートなしで単独使用しない、[`file_drop`] 専用）。
const ICON_CLASS: &str = "fw-wire-file-drop-icon";
const LABEL_CLASS: &str = "fw-wire-file-drop-label";
const HINT_CLASS: &str = "fw-wire-file-drop-hint";

/// ファイルドロップ領域 CSS（5 セレクタ）。[`crate::css::PARTS`] へ登録
/// される。
///
/// 値はトークン（`--fw-wire-*`）と [`crate::size::css`] が定義するカスタム
/// プロパティを `var()` で参照するのみで書き写さない。枠線は点線
/// （`dashed`）で「ドロップ可能領域」を示し、`grid`/`frame` 等の実線枠
/// とは意図的に区別する。
pub const FILE_DROP_CSS: &str = "\
.fw-wire-file-drop {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 0.5em;
  text-align: center;
  box-sizing: border-box;
  max-width: 100%;
  padding: calc(var(--fw-wire-control-size, 2rem) * 0.75) 1.5em;
  min-height: calc(var(--fw-wire-control-size, 2rem) * 4);
  border: var(--fw-wire-line-width) dashed var(--fw-wire-line);
  border-radius: var(--fw-wire-radius);
  background: var(--fw-wire-fill-subtle);
  color: var(--fw-wire-ink);
  font-family: var(--fw-wire-font-family);
  font-size: var(--fw-wire-font-size, 1rem);
  line-height: 1.4;
  user-select: none;
}
.fw-wire-file-drop-icon {
  display: flex;
  color: var(--fw-wire-ink-muted);
}
.fw-wire-file-drop-icon .fw-wire-icon-glyph {
  width: 2em;
  height: 2em;
}
.fw-wire-file-drop-label {
  font-weight: 600;
}
.fw-wire-file-drop-hint {
  font-size: 0.875em;
  color: var(--fw-wire-ink-muted);
}
";

/// ファイルドロップ領域を組み立てる。
///
/// - `label`: 必須。常に出力する説明文（例:「ここにファイルをドロップ」）。
/// - `hint`: 省略可能な補助文言（例:「PNG / JPG、最大 10MB」）。`None`
///   のときはパート要素自体を出力しない（空要素を残さない）。
/// - `icon`: 省略可能なアイコンスロット（[`crate::icon::image`]・
///   [`crate::icon::plus`] 等の戻り値をそのまま渡す。`docs/design/wireframe-ui-architecture.md`
///   §11.4）。`None` のときはアイコンのパート要素自体を出力しない。
/// - `size`: [`Size`] 5 段。ルート class `fw-wire-size-<段階>` として付与
///   する。
///
/// テキストは [`fandhe_frontend_core::text`] のみで流し込み（REQ-1 既定
/// エスケープ）、`<input>`/`<label>`/`<button>`/`<form>`/`<a href>`/
/// `role`/`aria-*`/`tabindex`/`style`/`draggable`/`on*`/`accept`/
/// `multiple`/`data-*`（部品側）は一切付与しない。`icon` に渡した
/// `Node` が持つ `data-icon` 属性はアイコン基盤側の識別子であり、
/// そのまま透過する。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_wireframe_ui::{icon, file_drop, Size};
///
/// let node = file_drop(
///     "ここにファイルをドロップ",
///     Some("PNG / JPG、最大 10MB"),
///     Some(icon::image(Size::Md)),
///     Size::Md,
/// );
/// let html = render(&node);
/// assert!(html.contains(r#"class="fw-wire-file-drop fw-wire-size-md""#));
/// assert!(html.contains(r#"class="fw-wire-file-drop-icon""#));
/// assert!(html.contains("<svg"));
/// assert!(html.contains(r#"class="fw-wire-file-drop-label""#));
/// assert!(html.contains("ここにファイルをドロップ"));
/// assert!(html.contains(r#"class="fw-wire-file-drop-hint""#));
/// assert!(html.contains("PNG / JPG、最大 10MB"));
///
/// // hint・icon を省略するとパート要素自体が出力されない。
/// let minimal = file_drop("ラベルのみ", None, None, Size::Md);
/// let minimal_html = render(&minimal);
/// assert!(!minimal_html.contains("fw-wire-file-drop-hint"));
/// assert!(!minimal_html.contains("fw-wire-file-drop-icon"));
/// assert!(!minimal_html.contains("<svg"));
///
/// // XSS 回帰: label/hint はいずれも既定エスケープを経由する。
/// let escaped = file_drop(
///     "<script>alert(1)</script>",
///     Some("\"><img src=x onerror=alert(1)>"),
///     None,
///     Size::Md,
/// );
/// let escaped_html = render(&escaped);
/// assert!(!escaped_html.contains("<script>alert(1)</script>"));
/// assert!(escaped_html.contains("&lt;script&gt;"));
/// assert!(escaped_html.contains("&quot;"));
/// ```
#[must_use]
pub fn file_drop(label: &str, hint: Option<&str>, icon: Option<Node>, size: Size) -> Node {
    let class = class_list("fw-wire-file-drop", &[Some(size.class())]);

    let mut children: Vec<Node> = Vec::new();
    if let Some(icon) = icon {
        children.push(div(vec![("class", ICON_CLASS)], vec![icon]));
    }
    children.push(div(vec![("class", LABEL_CLASS)], vec![text(label)]));
    if let Some(hint) = hint {
        children.push(div(vec![("class", HINT_CLASS)], vec![text(hint)]));
    }

    el_owned("div", vec![("class".to_string(), class)], children)
}
