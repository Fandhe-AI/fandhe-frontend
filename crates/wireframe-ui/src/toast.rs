//! 通知トースト部品（`Toast`、イシュー #2647、Phase 6「Overlay・Feedback」）。
//!
//! 任意のアイコン + 短い本文 + 見た目だけの閉じる「×」で「一時的な通知
//! カード」の配置イメージだけを示す、非インタラクティブなローファイ・
//! プレースホルダー。
//!
//! # 呼び出し文脈
//!
//! `fandhe-frontend-docs-site` の `wireframes::toast` showcase
//! （`/wireframes/toast/`）から呼ばれる。`fandhe_frontend_core::text`
//! のみでテキストを流し込むため、既定エスケープ（REQ-1）は本モジュールが
//! 独自に保証する必要はなく core 側の契約に委譲される。
//!
//! # API 設計の由来
//!
//! `toast` は blocks.pm に対応する部品を持たない、wireframe-ui 独自追加の
//! 14 部品のひとつである（`docs/design/wireframe-ui-architecture.md` §8）。
//! イシュー本文が想定する引数名は `text` だが、`fandhe_frontend_core::text`
//! と同じ値名前空間で衝突するため `message` に改める（意味は同じ）。
//!
//! # アイコンは `Option<Node>` スロット
//!
//! イシュー本文が想定する見た目（アイコン + 短文 + close）に対し、
//! [`crate::icon`] に info/success 専用のグリフはない。専用グリフを新設
//! すると `tests/icon.rs`・CLAUDE.md の件数表記など影響範囲が広がり本
//! イシューのスコープを超えるため、`link`（イシュー #2618）・`file_drop`
//! （イシュー #2633）と同じ `Option<Node>` アイコンスロット規約
//! （`docs/design/wireframe-ui-architecture.md` §11.4）を採用する。呼び
//! 出し側は [`crate::icon::bell`] や [`crate::icon::check`] 等、任意の
//! 既存アイコンを渡せる。`None` のときはアイコンのパート要素自体を出力
//! しない。
//!
//! # 閉じるグリフは `icon::x` 固定（instance swap にしない）
//!
//! 閉じる「×」は `select` の末尾指示子・`ratings` の `icon::star` と同じ
//! く固定パートとして扱い、`dismissible: bool` の 1 引数だけで有無を
//! 切り替える（`tag` の `remove: Option<Node>` のような差し替え可能な
//! スロットにはしない）。閉じるグリフは常に [`crate::icon::x`] で、
//! `dismissible` が `false` のときはパート要素自体を出力しない。
//!
//! # 表示状態軸を持たない
//!
//! 重要度バリアント（info/success/error 等）や `Bold`/`Primary`/`Active`/
//! `Disabled` はいずれも受け取らない。最小構成にとどめ、重要度の表現が
//! 必要な利用者には Themes（`/themes/toast/`）/ Primitives
//! （`/primitives/toast/`）を案内する（`site/wireframes/toast.md` 参照）。
//!
//! # `<button>`・`role`・`aria-live` 等は出力しない（最重要）
//!
//! `docs/design/wireframe-ui-architecture.md` §7（非対話制約）に従い、
//! 閉じるパートは `<button>` にしない。`role="status"`・`aria-live`・
//! `<output>`・`tabindex`・`style`・`on*` は一切出力しない。実際に操作
//! 可能な通知が必要な利用者には Themes（`/themes/toast/`）/ Primitives
//! （`/primitives/toast/`）を案内する。
//!
//! # 固定配置は付けない
//!
//! `position: fixed`/`absolute` による画面隅への固定配置は利用者の
//! レイアウトの責務とし、docs のデモ枠の中に収まる in-flow のカードと
//! して描く。「浮いている感じ」はトークン参照のハードシャドウで表現する。

use fandhe_frontend_core::{el_owned, span, text, Node};

use crate::class::class_list;
use crate::icon::x as x_icon;
use crate::size::Size;

/// パート class（部品ルートなしで単独使用しない、[`toast`] 専用）。
const ICON_CLASS: &str = "fw-wire-toast-icon";
const MESSAGE_CLASS: &str = "fw-wire-toast-message";
const DISMISS_CLASS: &str = "fw-wire-toast-dismiss";

/// トースト CSS（4 セレクタ）。[`crate::css::PARTS`] へ登録される。
///
/// 値はトークン（`--fw-wire-*`）と [`crate::size::css`] が定義するカスタム
/// プロパティを `var()` で参照するのみで書き写さない。`box-shadow` は
/// リテラルの色値を使わずトークン参照のハードシャドウとし、`position` は
/// 一切指定しない（固定配置は利用者のレイアウトの責務、モジュール doc
/// 参照）。
pub const TOAST_CSS: &str = "\
.fw-wire-toast {
  display: flex;
  align-items: center;
  gap: 0.5em;
  box-sizing: border-box;
  max-width: 100%;
  padding: calc(var(--fw-wire-control-size, 2rem) * 0.4) 1em;
  border: var(--fw-wire-line-width) solid var(--fw-wire-line);
  border-radius: var(--fw-wire-radius);
  background: var(--fw-wire-paper);
  color: var(--fw-wire-ink);
  font-family: var(--fw-wire-font-family);
  font-size: var(--fw-wire-font-size, 1rem);
  line-height: 1.4;
  box-shadow: 0 0.25em 0 var(--fw-wire-line-subtle);
  user-select: none;
}
.fw-wire-toast-icon {
  display: flex;
  flex-shrink: 0;
  color: var(--fw-wire-ink-muted);
}
.fw-wire-toast-icon .fw-wire-icon-glyph {
  width: 1.25em;
  height: 1.25em;
}
.fw-wire-toast-message {
  flex: 1;
  min-width: 0;
}
.fw-wire-toast-dismiss {
  display: inline-flex;
  flex-shrink: 0;
  margin-inline-start: auto;
  color: var(--fw-wire-ink-muted);
}
";

/// トーストを組み立てる。
///
/// - `message`: 必須。常に出力する短い本文。
/// - `icon`: 省略可能なアイコンスロット（[`crate::icon::bell`]・
///   [`crate::icon::check`] 等の戻り値をそのまま渡す。
///   `docs/design/wireframe-ui-architecture.md` §11.4）。`None` のときは
///   アイコンのパート要素自体を出力しない。
/// - `dismissible`: `true` のときだけ末尾に閉じるパートを出力し、中に
///   [`crate::icon::x`] を `size` で合成する。`false` のときはパート
///   要素自体を出力しない。
/// - `size`: [`Size`] 5 段。ルート class `fw-wire-size-<段階>` として付与
///   し、閉じるグリフのサイズにも使う。
///
/// テキストは [`fandhe_frontend_core::text`] のみで流し込み（REQ-1 既定
/// エスケープ）、`<button>`/`role`/`aria-live`/`<output>`/`tabindex`/
/// `style`/`on*`/`data-*`（部品側）は一切付与しない。`icon` に渡した
/// `Node` が持つ `data-icon` 属性はアイコン基盤側の識別子であり、
/// そのまま透過する。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_wireframe_ui::{icon, toast, Size};
///
/// let node = toast(
///     "保存しました",
///     Some(icon::check(Size::Md)),
///     true,
///     Size::Md,
/// );
/// let html = render(&node);
/// assert!(html.contains(r#"class="fw-wire-toast fw-wire-size-md""#));
/// assert!(html.contains(r#"class="fw-wire-toast-icon""#));
/// assert!(html.contains(r#"class="fw-wire-toast-message""#));
/// assert!(html.contains("保存しました"));
/// assert!(html.contains(r#"class="fw-wire-toast-dismiss""#));
/// assert!(html.contains(r#"data-icon="x""#));
///
/// // icon・dismissible を省略するとパート要素自体が出力されない。
/// let minimal = toast("本文のみ", None, false, Size::Md);
/// let minimal_html = render(&minimal);
/// assert!(!minimal_html.contains("fw-wire-toast-icon"));
/// assert!(!minimal_html.contains("fw-wire-toast-dismiss"));
///
/// // XSS 回帰: message は既定エスケープを経由する。
/// let escaped = toast(
///     "<script>alert(1)</script>",
///     None,
///     false,
///     Size::Md,
/// );
/// let escaped_html = render(&escaped);
/// assert!(!escaped_html.contains("<script>alert(1)</script>"));
/// assert!(escaped_html.contains("&lt;script&gt;"));
/// ```
#[must_use]
pub fn toast(message: &str, icon: Option<Node>, dismissible: bool, size: Size) -> Node {
    let class = class_list("fw-wire-toast", &[Some(size.class())]);

    let mut children: Vec<Node> = Vec::new();
    if let Some(icon) = icon {
        children.push(span(vec![("class", ICON_CLASS)], vec![icon]));
    }
    children.push(span(vec![("class", MESSAGE_CLASS)], vec![text(message)]));
    if dismissible {
        children.push(span(vec![("class", DISMISS_CLASS)], vec![x_icon(size)]));
    }

    el_owned("div", vec![("class".to_string(), class)], children)
}
