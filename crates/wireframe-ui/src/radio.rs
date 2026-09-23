//! ラジオボタン部品（`Radio`、イシュー #2626、Phase 3「Forms A」）。
//!
//! 「ラジオボタン + ラベル」の配置イメージだけを示す、非インタラクティブな
//! ローファイ・プレースホルダー。「Radio」という名前だが、`<input
//! type="radio">`・`role="radio"`・選択状態の遷移のいずれも実装しない表示
//! 専用部品である点に注意する。
//!
//! # 呼び出し文脈
//!
//! `fandhe-frontend-docs-site` の `wireframes::radio` showcase
//! （`/wireframes/radio/`）から呼ばれる。`fandhe_frontend_core::text`
//! のみでラベルを流し込むため、既定エスケープ（REQ-1）は本モジュールが
//! 独自に保証する必要はなく core 側の契約に委譲される。
//!
//! # API 設計の由来
//!
//! `docs/design/wireframe-ui-architecture.md` §6 の汎用変換規約と、Phase 3
//! 既存部品（[`crate::button`]/[`crate::select`]）の先例から独立設計した
//! （`site/wireframes/radio.md` の「原案差分メモ」節も参照）。blocks.pm の
//! 外観・anatomy・プロパティ構成の実装への転用は書面許諾が得られるまで
//! 保留されている（同文書 §2、イシュー #2602）ため、本部品の引数構成は
//! blocks.pm の Figma プロパティを参照・書き写さず、一般的な UI キット設計
//! で広く使われる汎用パターンから独立に起こした。
//!
//! Figma プロパティの `Label`（bool）+ `Text` の 2 プロパティは、
//! `Option<&str>` 1 引数（`None` のときラベルパート要素自体を出力しない）
//! へ畳み込む（[`crate::annotation`] の `description` と同じ「空要素を
//! 残さない」方針）。
//!
//! **選択状態は新しい専用型を導入せず、既存の共通型 [`crate::props::Active`]
//! （`data-active`）を再利用する**。`props.rs` は checkbox（イシュー #2625）・
//! switch（イシュー #2627）も同じ共有ファイルとして触るため、`Selected`/
//! `Checked` 型の新設は本イシューでは意図的にスコープ外とした（衝突面を
//! 広げないため）。将来これらの部品間で選択状態を表す共通型を統一したく
//! なった場合は、`out-of-scope-tracking.md` に従い別イシューで提案する。
//!
//! `Disabled`（`docs/design/wireframe-ui-architecture.md` §5 の表示状態軸）
//! は、Forms 部品として整合させるため自己定義で追加する。`Bold`/`Primary`
//! は付与しない（ラジオに太字・強調軸を持たせる根拠がないため、`select`
//! と同じ判断）。
//!
//! 内側の黒丸（選択済み表現）は SVG アイコンではなく CSS `::after` で描く
//! （アイコンスロットを持たない部品のため、`icon::ALL` への追記契約に
//! 触れずに済む）。
//!
//! # `<input>` 要素は出力しない（最重要）
//!
//! `docs/design/wireframe-ui-architecture.md` §7（非対話制約）に従い、
//! ルートは `div` とし `role`/`aria-*`/`tabindex`/`on*` は一切出力しない。
//! 実際に操作可能なラジオが必要な利用者には Primitives/Themes の
//! Radio Group を案内する（`site/wireframes/radio.md` 参照）。

use fandhe_frontend_core::{el_owned, span, text, Node};

use crate::class::class_list;
use crate::props::{Active, Disabled};
use crate::size::Size;

/// 外円パートの class（部品ルートなしで単独使用しない、[`radio`] 専用）。
const CONTROL_CLASS: &str = "fw-wire-radio-control";

/// ラベルのパート class（部品ルートなしで単独使用しない、[`radio`] 専用）。
const LABEL_CLASS: &str = "fw-wire-radio-label";

/// ラジオボタン CSS（7 セレクタ）。[`crate::css::PARTS`] へ登録される。
///
/// 円の直径・フォントサイズは値を書き写さず [`crate::size::css`] が定義する
/// `--fw-wire-font-size` を `em` 基準で参照する。`[data-active]`/
/// `[data-disabled]` 単独セレクタは `crates/wireframe-ui/tests/common_api.rs`
/// の「`.` で始まる行はすべて `.fw-wire-` プレフィックス」走査に
/// 引っかからないよう `.fw-wire-radio[data-active] ...`/
/// `.fw-wire-radio[data-disabled]` の形で書く
/// （`docs/design/wireframe-ui-architecture.md` §10.4）。
pub const RADIO_CSS: &str = "\
.fw-wire-radio {
  display: inline-flex;
  align-items: center;
  gap: 0.5em;
  box-sizing: border-box;
  color: var(--fw-wire-ink);
  font-family: var(--fw-wire-font-family);
  font-size: var(--fw-wire-font-size, 1rem);
  user-select: none;
}
.fw-wire-radio-control {
  position: relative;
  box-sizing: border-box;
  width: 1.25em;
  height: 1.25em;
  flex-shrink: 0;
  border: var(--fw-wire-line-width) solid var(--fw-wire-line);
  border-radius: 50%;
  background: var(--fw-wire-paper);
}
.fw-wire-radio-label {
  white-space: nowrap;
}
.fw-wire-radio[data-active] .fw-wire-radio-control {
  border-color: var(--fw-wire-ink);
}
.fw-wire-radio[data-active] .fw-wire-radio-control::after {
  content: \"\";
  position: absolute;
  inset: 0.25em;
  border-radius: 50%;
  background: var(--fw-wire-ink);
}
.fw-wire-radio[data-disabled] {
  opacity: 0.5;
}
.fw-wire-radio[data-disabled] .fw-wire-radio-control {
  border-style: dashed;
  background: var(--fw-wire-fill-subtle);
}
";

/// ラジオボタンを組み立てる。
///
/// - `label`: 省略可能なラベル文言。`None` のときはラベルパート要素自体を
///   出力しない（空要素を残さない）。
/// - `size`: [`Size`] 5 段。ルート class `fw-wire-size-<段階>` として付与し、
///   円の直径・フォントサイズは [`crate::size::css`] が定義する
///   `--fw-wire-font-size` を `em` 基準で参照する（本モジュールは値を
///   書き写さない）。
/// - `active`: `true` のとき `data-active=""` を付与する。**選択済み
///   （内側の黒丸あり）を表す表示状態**（`Selected`/`Checked` 型は新設せず
///   既存の [`crate::props::Active`] を再利用する。モジュール doc参照）。
/// - `disabled`: `true` のとき `data-disabled=""` を付与する（見た目のみ。
///   操作不能を実装するものではない）。
///
/// テキストは [`fandhe_frontend_core::text`] のみで流し込み（REQ-1 既定
/// エスケープ）、`<input>`・`<label>`・`role`/`aria-*`/`tabindex`/`style`/
/// `on*` は一切出力しない。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_wireframe_ui::{radio, Active, Disabled, Size};
///
/// let node = radio(Some("選択肢 A"), Size::Md, Active(false), Disabled(false));
/// let html = render(&node);
/// assert!(html.contains(r#"class="fw-wire-radio fw-wire-size-md""#));
/// assert!(html.contains(r#"class="fw-wire-radio-control""#));
/// assert!(html.contains(r#"class="fw-wire-radio-label""#));
/// assert!(html.contains("選択肢 A"));
/// assert!(!html.contains("data-active"));
/// assert!(!html.contains("data-disabled"));
///
/// // ラベルを省略するとパート要素自体が出力されない。
/// let without_label = radio(None, Size::Md, Active(false), Disabled(false));
/// assert!(!render(&without_label).contains("fw-wire-radio-label"));
///
/// // 選択済み（Active）+ Disabled。
/// let full = radio(Some("選択肢 B"), Size::Md, Active(true), Disabled(true));
/// let full_html = render(&full);
/// assert!(full_html.contains(r#"data-active="""#));
/// assert!(full_html.contains(r#"data-disabled="""#));
///
/// // `<input>`/`<label>`/`role` は一切出力しない
/// // （`docs/design/wireframe-ui-architecture.md` §7）。
/// assert!(!full_html.contains("<input"));
/// assert!(!full_html.contains("<label"));
/// assert!(!full_html.contains(" role=\""));
///
/// // XSS 回帰: ラベルは既定エスケープを経由する。
/// let escaped = radio(Some("<script>alert(1)</script>"), Size::Md, Active(false), Disabled(false));
/// let escaped_html = render(&escaped);
/// assert!(!escaped_html.contains("<script>alert(1)</script>"));
/// assert!(escaped_html.contains("&lt;script&gt;"));
/// ```
#[must_use]
pub fn radio(label: Option<&str>, size: Size, active: Active, disabled: Disabled) -> Node {
    let class = class_list("fw-wire-radio", &[Some(size.class())]);

    let mut attrs: Vec<(String, String)> = vec![("class".to_string(), class)];
    if let Some(attr) = active.attr() {
        attrs.push(attr);
    }
    if let Some(attr) = disabled.attr() {
        attrs.push(attr);
    }

    let mut children: Vec<Node> = vec![span(vec![("class", CONTROL_CLASS)], vec![])];
    if let Some(label) = label {
        children.push(span(vec![("class", LABEL_CLASS)], vec![text(label)]));
    }

    el_owned("div", attrs, children)
}
