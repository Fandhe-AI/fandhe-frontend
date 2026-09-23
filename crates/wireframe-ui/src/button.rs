//! ボタン部品（`Button`、イシュー #2621、Phase 3「Forms A」）。
//!
//! テキストラベル + 任意の先頭アイコン + サイズ/強調/無効状態を持つ、
//! 非インタラクティブなローファイ・プレースホルダー。「Button」という
//! 名前だが、`<button>` 要素・クリック操作・フォーム送信のいずれも実装
//! しない表示専用部品である点に注意する。
//!
//! # 呼び出し文脈
//!
//! `fandhe-frontend-docs-site` の `wireframes::button` showcase
//! （`/wireframes/button/`）から呼ばれる。`fandhe_frontend_core::text`
//! のみでラベルを流し込むため、既定エスケープ（REQ-1）は本モジュールが
//! 独自に保証する必要はなく core 側の契約に委譲される。
//!
//! # API 設計の由来
//!
//! blocks.pm の Button 部品が持つ Figma プロパティ（Size / Primary /
//! Type〔アイコン+テキスト・テキストのみ〕/ Icon〔instance swap〕/ Text）を、
//! `docs/design/wireframe-ui-architecture.md` §6 の汎用変換規約と §11.4 の
//! `Option<Node>` アイコンスロット規約でそのまま Rust 引数へ変換した
//! （`site/wireframes/button.md` の「原案差分メモ」節も参照）。Type
//! （アイコン+テキスト / テキストのみ）は独立した列挙型を持たず `icon`
//! 引数の `Some`/`None` へ畳み込む。アイコンのみ（テキストなし）の
//! variant は本イシューでは実装しない（意図的な絞り込み、原案差分メモ
//! 参照）。
//!
//! `Disabled`（`docs/design/wireframe-ui-architecture.md` §5 の表示状態
//! 軸）は blocks.pm の列挙外だが、Forms 部品向けに用意済みの共通型
//! [`crate::props::Disabled`] をそのまま使う。`Bold`/`Active` は付与しない
//! （Button に太字強調・アクティブ状態を持たせる根拠がないため）。
//!
//! # `<button>` 要素は出力しない（最重要）
//!
//! `docs/design/wireframe-ui-architecture.md` §7（非対話制約）に従い、
//! ルートは `div` とし `role`/`aria-*`/`tabindex`/`on*` は一切出力しない。
//! 実際に操作可能なボタンが必要な利用者には Primitives/Themes の Button
//! を案内する（`site/wireframes/button.md` 参照）。

use fandhe_frontend_core::{el_owned, span, text, Node};

use crate::class::class_list;
use crate::props::{Disabled, Primary};
use crate::size::Size;

/// ラベルのパート class（部品ルートなしで単独使用しない、[`button`] 専用）。
const LABEL_CLASS: &str = "fw-wire-button-label";

/// ボタン CSS（4 セレクタ）。[`crate::css::PARTS`] へ登録される。
///
/// 高さ・フォントサイズは値を書き写さず [`crate::size::css`] が定義する
/// `--fw-wire-control-size`/`--fw-wire-font-size` を `var()` で参照する。
/// `[data-disabled]` 単独セレクタは `crates/wireframe-ui/tests/common_api.rs`
/// の「`.` で始まる行はすべて `.fw-wire-` プレフィックス」走査に
/// 引っかからないよう `.fw-wire-button[data-disabled]` の形で書く
/// （`docs/design/wireframe-ui-architecture.md` §10.4）。
pub const BUTTON_CSS: &str = "\
.fw-wire-button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 0.5em;
  box-sizing: border-box;
  min-height: var(--fw-wire-control-size, 2rem);
  padding: 0 0.75em;
  border: var(--fw-wire-line-width) solid var(--fw-wire-line);
  border-radius: var(--fw-wire-radius);
  background: var(--fw-wire-paper);
  color: var(--fw-wire-ink);
  font-family: var(--fw-wire-font-family);
  font-size: var(--fw-wire-font-size, 1rem);
  user-select: none;
}
.fw-wire-button-label {
  white-space: nowrap;
  font-weight: 600;
}
.fw-wire-button.fw-wire-primary {
  background: var(--fw-wire-ink);
  border-color: var(--fw-wire-ink);
  color: var(--fw-wire-paper);
}
.fw-wire-button[data-disabled] {
  opacity: 0.5;
  border-style: dashed;
}
";

/// ボタンを組み立てる。
///
/// - `label`: 必須。ボタン内に表示する文言。
/// - `icon`: 省略可能な先頭アイコンスロット（`docs/design/wireframe-ui-architecture.md`
///   §11.4）。`Some(icon::plus(size))` のように渡す。`None` のときは
///   アイコン要素自体を出力しない。
/// - `size`: [`Size`] 5 段。ルート class `fw-wire-size-<段階>` として付与し、
///   コントロール高さ・フォントサイズは [`crate::size::css`] が定義する
///   `--fw-wire-control-size`/`--fw-wire-font-size` を `var()` で参照する
///   （本モジュールは値を書き写さない）。
/// - `primary`: `true` のとき強調（反転色）バリアントにする。
/// - `disabled`: `true` のとき `data-disabled=""` を付与する（見た目のみ。
///   クリック不能を実装するものではない）。
///
/// テキストは [`fandhe_frontend_core::text`] のみで流し込み（REQ-1 既定
/// エスケープ）、`<button>`・`role`/`aria-*`/`tabindex`/`style`/`on*` は
/// 一切出力しない。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_wireframe_ui::{button, icon, Disabled, Primary, Size};
///
/// let node = button("送信", None, Size::Md, Primary(false), Disabled(false));
/// let html = render(&node);
/// assert!(html.contains(r#"class="fw-wire-button fw-wire-size-md""#));
/// assert!(html.contains(r#"class="fw-wire-button-label""#));
/// assert!(html.contains("送信"));
/// assert!(!html.contains("<svg"));
/// assert!(!html.contains("data-disabled"));
///
/// // アイコン付き + Primary + Disabled。
/// let full = button(
///     "追加",
///     Some(icon::plus(Size::Md)),
///     Size::Md,
///     Primary(true),
///     Disabled(true),
/// );
/// let full_html = render(&full);
/// assert!(full_html.contains("<svg"));
/// assert!(full_html.contains("fw-wire-primary"));
/// assert!(full_html.contains(r#"data-disabled="""#));
///
/// // `<button>`/`role`/`aria-*` は一切出力しない
/// // （`docs/design/wireframe-ui-architecture.md` §7）。
/// assert!(!full_html.contains("<button"));
/// assert!(!full_html.contains(" role=\""));
///
/// // XSS 回帰: ラベルは既定エスケープを経由する。
/// let escaped = button("<script>alert(1)</script>", None, Size::Md, Primary(false), Disabled(false));
/// let escaped_html = render(&escaped);
/// assert!(!escaped_html.contains("<script>alert(1)</script>"));
/// assert!(escaped_html.contains("&lt;script&gt;"));
/// ```
#[must_use]
pub fn button(
    label: &str,
    icon: Option<Node>,
    size: Size,
    primary: Primary,
    disabled: Disabled,
) -> Node {
    let class = class_list("fw-wire-button", &[Some(size.class()), primary.class()]);

    let mut attrs: Vec<(String, String)> = vec![("class".to_string(), class)];
    if let Some(attr) = disabled.attr() {
        attrs.push(attr);
    }

    let mut children: Vec<Node> = Vec::new();
    if let Some(icon) = icon {
        children.push(icon);
    }
    children.push(span(vec![("class", LABEL_CLASS)], vec![text(label)]));

    el_owned("div", attrs, children)
}
