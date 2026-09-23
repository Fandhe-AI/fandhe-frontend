//! テキストフィールド風プレースホルダー部品（`Input`、イシュー #2622、
//! Phase 3「Forms A」）。
//!
//! 先頭アイコン + プレースホルダー風テキストで「テキストフィールドらしさ」
//! だけを表現する非インタラクティブなローファイ・プレースホルダー。
//!
//! # 呼び出し文脈
//!
//! `fandhe-frontend-docs-site` の `wireframes::input` showcase
//! （`/wireframes/input/`）から呼ばれる。`fandhe_frontend_core::text` のみで
//! テキストを流し込むため、既定エスケープ（REQ-1）は本モジュールが独自に
//! 保証する必要はなく core 側の契約に委譲される。先頭アイコンスロットは
//! [`crate::icon`] の関数の戻り値（`Node`）をそのまま受け取る。
//!
//! # API 設計の由来
//!
//! `docs/design/wireframe-ui-architecture.md` §6 の汎用変換規約と §11.4 の
//! `Option<Node>` アイコンスロット規約から独立設計した（`site/wireframes/input.md`
//! の「原案差分メモ」節も参照）。blocks.pm の Figma プロパティ構成（Icon
//! bool + swap・State 列挙等）をそのまま転写したものではなく、アイコンは
//! `leading: Option<Node>` スロットへ、状態は共通型 [`crate::props::Active`]/
//! [`crate::props::Disabled`]（`data-*`）へそれぞれ畳み込む。専用の
//! `Icon` bool 型・`InputState` 列挙は追加しない。
//!
//! `text` はプレースホルダー風（`--fw-wire-ink-muted`）で描画する 1 種類の
//! みとし、実際の入力値とプレースホルダーの区別はモデル化しない
//! （表示専用プレースホルダー部品としての責務範囲、`docs/design/wireframe-ui-architecture.md`
//! §1）。`Bold`/`Primary` は使わない（テキストフィールドに強調軸を持たせる
//! 根拠がないため）。
//!
//! # `<input>` は出力しない（最重要）
//!
//! `docs/design/wireframe-ui-architecture.md` §7（非対話制約）に従い、
//! ルートは `div` とし、`<input>`/`role`/`aria-*`/`tabindex`/`style`/
//! `placeholder`/`value`/`on*` は一切出力しない。実際に入力可能な
//! フィールドが必要な利用者には Themes の Input（`/themes/input/`）を
//! 案内する（`site/wireframes/input.md` 参照）。
//!
//! [`crate::props::Active`]/[`crate::props::Disabled`] の `.attr()` は
//! wireframe-ui 内で本モジュールが最初の実消費者である（イシュー #2605 で
//! 型自体は定義済みだったが、`data-active`/`data-disabled` を実際に出力する
//! 部品はこれまで存在しなかった）。

use fandhe_frontend_core::{el_owned, span, text, Node};

use crate::class::class_list;
use crate::props::{Active, Disabled};
use crate::size::Size;

/// テキストのパート class（部品ルートなしで単独使用しない、[`input`] 専用）。
const TEXT_CLASS: &str = "fw-wire-input-text";

/// テキストフィールド CSS（5 セレクタ）。[`crate::css::PARTS`] へ登録される。
///
/// `[data-active]`/`[data-disabled]` は必ず `.fw-wire-input` に連結した形
/// （`.fw-wire-input[data-active]`）で書く（素の `[data-active]` セレクタは
/// 書かない、`docs/design/wireframe-ui-architecture.md` §10 の追記契約）。
pub const INPUT_CSS: &str = "\
.fw-wire-input {
  display: inline-flex;
  align-items: center;
  gap: 0.5em;
  box-sizing: border-box;
  width: 100%;
  max-width: 24em;
  min-height: var(--fw-wire-control-size, 2rem);
  padding: 0 0.75em;
  border: var(--fw-wire-line-width) solid var(--fw-wire-line);
  border-radius: var(--fw-wire-radius);
  background: var(--fw-wire-paper);
  color: var(--fw-wire-ink-muted);
  font-family: var(--fw-wire-font-family);
  font-size: var(--fw-wire-font-size, 1rem);
  user-select: none;
}
.fw-wire-input-text {
  flex: 1 1 auto;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.fw-wire-input .fw-wire-icon-glyph {
  color: var(--fw-wire-ink-muted);
}
.fw-wire-input[data-active] {
  border-color: var(--fw-wire-ink);
  box-shadow: 0 0 0 1px var(--fw-wire-ink);
  color: var(--fw-wire-ink);
}
.fw-wire-input[data-disabled] {
  opacity: 0.5;
  border-style: dashed;
  background: var(--fw-wire-fill-subtle);
}
";

/// テキストフィールド風プレースホルダーを組み立てる。
///
/// - `text`: 必須。表示文言（プレースホルダー風の 1 種類のみ。空文字でも
///   パート要素自体は常に出力する）。
/// - `leading`: 省略可能な先頭アイコンスロット（`docs/design/wireframe-ui-architecture.md`
///   §11.4）。例: `Some(icon::search(size))`。`None` のときはアイコン要素
///   自体を出力しない。
/// - `size`: [`Size`] 5 段。ルート class `fw-wire-size-<段階>` として付与し、
///   最小高さ・フォントサイズは [`crate::size::css`] が定義するカスタム
///   プロパティを `var()` で参照する（本モジュールは値を書き写さない）。
/// - `active`: `true` のとき `data-active=""` を付与し、フォーカス風の
///   強調枠（`--fw-wire-ink` の枠線・box-shadow）にする。
/// - `disabled`: `true` のとき `data-disabled=""` を付与し、破線・淡色に
///   する。
///
/// テキストは [`fandhe_frontend_core::text`] のみで流し込み（REQ-1 既定
/// エスケープ）、`role`/`aria-*`/`tabindex`/`style`/`placeholder`/`value`/
/// `on*` は一切付与せず、`<input>` 要素も出力しない。`leading` に渡した
/// `Node`（例: [`crate::icon::search`]）が持つ `data-icon` 属性はアイコン
/// 基盤側の識別子であり、部品側の出力ではない。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_wireframe_ui::{icon, input, Active, Disabled, Size};
///
/// let node = input("メールアドレス", None, Size::Md, Active(false), Disabled(false));
/// let html = render(&node);
/// assert!(html.contains(r#"class="fw-wire-input fw-wire-size-md""#));
/// assert!(html.contains(r#"class="fw-wire-input-text""#));
/// assert!(html.contains("メールアドレス"));
/// assert!(!html.contains("<svg"));
/// assert!(!html.contains("data-active"));
/// assert!(!html.contains("data-disabled"));
///
/// // 先頭アイコン + Active + Disabled。
/// let full = input(
///     "検索",
///     Some(icon::search(Size::Md)),
///     Size::Md,
///     Active(true),
///     Disabled(true),
/// );
/// let full_html = render(&full);
/// assert!(full_html.contains("<svg"));
/// assert!(full_html.contains(r#"data-active="""#));
/// assert!(full_html.contains(r#"data-disabled="""#));
///
/// // `<input>`/`role` は一切出力しない（`docs/design/wireframe-ui-architecture.md` §7）。
/// assert!(!full_html.contains("<input"));
/// assert!(!full_html.contains(" role=\""));
///
/// // XSS 回帰: テキストは既定エスケープを経由する。
/// let escaped = input("<script>alert(1)</script>", None, Size::Md, Active(false), Disabled(false));
/// let escaped_html = render(&escaped);
/// assert!(!escaped_html.contains("<script>alert(1)</script>"));
/// assert!(escaped_html.contains("&lt;script&gt;"));
/// ```
#[must_use]
pub fn input(
    text_content: &str,
    leading: Option<Node>,
    size: Size,
    active: Active,
    disabled: Disabled,
) -> Node {
    let class = class_list("fw-wire-input", &[Some(size.class())]);

    let mut children: Vec<Node> = Vec::new();
    if let Some(leading) = leading {
        children.push(leading);
    }
    children.push(span(vec![("class", TEXT_CLASS)], vec![text(text_content)]));

    let mut attrs: Vec<(String, String)> = vec![("class".to_string(), class)];
    attrs.extend(active.attr());
    attrs.extend(disabled.attr());

    el_owned("div", attrs, children)
}
