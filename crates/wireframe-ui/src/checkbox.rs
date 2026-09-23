//! チェックボックス部品（`Checkbox`、イシュー #2625、Phase 3「Forms A」）。
//!
//! 正方形のボックス + 任意のラベルを持つ、非インタラクティブなローファイ・
//! プレースホルダー。`<input type="checkbox">`・`role="checkbox"`・
//! `aria-checked`・`tabindex` のいずれも実装しない表示専用部品である点に
//! 注意する。
//!
//! # 呼び出し文脈
//!
//! `fandhe-frontend-docs-site` の `wireframes::checkbox` showcase
//! （`/wireframes/checkbox/`）から呼ばれる。`fandhe_frontend_core::text`
//! のみでラベルを流し込むため、既定エスケープ（REQ-1）は本モジュールが
//! 独自に保証する必要はなく core 側の契約に委譲される。
//!
//! # API 設計の由来
//!
//! `docs/design/wireframe-ui-architecture.md` §6 の汎用変換規約から独立
//! 設計した（`site/wireframes/checkbox.md` の「原案差分メモ」節も参照）。
//! blocks.pm の外観・anatomy・プロパティ構成の実装への転用は書面許諾が
//! 得られるまで保留されている（同文書 §2、イシュー #2602）ため、本部品の
//! 引数構成は blocks.pm の Figma プロパティを参照・書き写さず独立に設計
//! した。「ラベル有無 + ラベル文言」は [`crate::annotation`] の
//! `description: Option<&str>` と同型の 1 引数へ畳み込む。
//!
//! チェック済み状態は、専用の `Checked`/`data-checked` 型を新設せず
//! 既存共通型 [`crate::props::Active`]（`data-active`）を再利用する
//! （`docs/design/wireframe-ui-architecture.md` §10.1 が現時点の表示状態
//! 属性として `data-active`/`data-disabled` を規定しているため。詳細は
//! `site/wireframes/checkbox.md` の「原案差分メモ」節）。`Disabled` は
//! Forms 部品向けに用意済みの共通型 [`crate::props::Disabled`] をそのまま
//! 使う。`Primary`/`Bold`/`Orientation` は付与しない（Checkbox に強調・
//! 太字・方向を持たせる根拠がないため）。
//!
//! # `<input>` 要素は出力しない（最重要）
//!
//! `docs/design/wireframe-ui-architecture.md` §7（非対話制約）に従い、
//! ルートは `div` とし `role`/`aria-*`（グリフの `aria-hidden="true"` を
//! 除く）/`tabindex`/`on*` は一切出力しない。実際に操作可能なチェック
//! ボックスが必要な利用者には Primitives/Themes の Checkbox を案内する
//! （`site/wireframes/checkbox.md` 参照）。

use fandhe_frontend_core::{el_owned, span, text, Node};

use crate::class::class_list;
use crate::icon;
use crate::props::{Active, Disabled};
use crate::size::Size;

/// ボックスのパート class（部品ルートなしで単独使用しない、[`checkbox`] 専用）。
const BOX_CLASS: &str = "fw-wire-checkbox-box";
/// ラベルのパート class（部品ルートなしで単独使用しない、[`checkbox`] 専用）。
const LABEL_CLASS: &str = "fw-wire-checkbox-label";

/// チェックボックス CSS（5 セレクタ）。[`crate::css::PARTS`] へ登録される。
///
/// 寸法は値を書き写さず [`crate::size::css`] が定義する
/// `--fw-wire-font-size`/`--fw-wire-control-size` を `var()` で参照する。
/// `[data-active]`/`[data-disabled]` は必ず `.fw-wire-checkbox` に連結して
/// 書く（`crates/wireframe-ui/tests/common_api.rs` の「`.` で始まる行は
/// すべて `.fw-wire-` プレフィックス」走査に引っかからないよう、
/// [`crate::button::BUTTON_CSS`] と同じ形にする）。チェック済みは
/// 「ボックスを `ink` で反転塗り + `paper` 色のチェック線」で表す（黒塗り
/// 二値を機械的に再現せず読みやすさ優先、`docs/design/
/// wireframe-ui-architecture.md` §3）。
pub const CHECKBOX_CSS: &str = "\
.fw-wire-checkbox {
  display: inline-flex;
  align-items: center;
  gap: 0.5em;
  font-family: var(--fw-wire-font-family);
  font-size: var(--fw-wire-font-size, 1rem);
  color: var(--fw-wire-ink);
  user-select: none;
}
.fw-wire-checkbox-box {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  box-sizing: border-box;
  width: 1.25em;
  height: 1.25em;
  border: var(--fw-wire-line-width) solid var(--fw-wire-line);
  border-radius: var(--fw-wire-radius);
  background: var(--fw-wire-paper);
  flex-shrink: 0;
}
.fw-wire-checkbox-label {
  white-space: nowrap;
}
.fw-wire-checkbox[data-active] .fw-wire-checkbox-box {
  background: var(--fw-wire-ink);
  border-color: var(--fw-wire-ink);
  color: var(--fw-wire-paper);
}
.fw-wire-checkbox[data-disabled] {
  opacity: 0.5;
}
";

/// チェックボックスを組み立てる。
///
/// - `label`: 省略可能なラベル文言。`None` のときはラベルのパート要素
///   自体を出力しない（[`crate::annotation`] の `description` と同型）。
/// - `size`: [`Size`] 5 段。ルート class `fw-wire-size-<段階>` として付与し、
///   寸法は [`crate::size::css`] が定義する CSS カスタムプロパティを
///   `var()` で参照する（本モジュールは値を書き写さない）。
/// - `active`: チェック済み状態。`true` のとき `data-active=""` を付与し、
///   ボックス内へ [`crate::icon::check`] を描画する（`false` のときは
///   グリフ自体を出力しない。CSS で非表示にする方式は採らない）。
/// - `disabled`: `true` のとき `data-disabled=""` を付与する（見た目のみ。
///   操作不能を実装するものではない）。
///
/// ラベルは [`fandhe_frontend_core::text`] のみで流し込み（REQ-1 既定
/// エスケープ）、`<input>`・`role`/`aria-checked`/`tabindex`/`style`/`on*`
/// は一切出力しない（`docs/design/wireframe-ui-architecture.md` §7）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_wireframe_ui::{checkbox, Active, Disabled, Size};
///
/// let node = checkbox(Some("利用規約に同意する"), Size::Md, Active(false), Disabled(false));
/// let html = render(&node);
/// assert!(html.contains(r#"class="fw-wire-checkbox fw-wire-size-md""#));
/// assert!(html.contains(r#"class="fw-wire-checkbox-box""#));
/// assert!(html.contains(r#"class="fw-wire-checkbox-label""#));
/// assert!(html.contains("利用規約に同意する"));
/// assert!(!html.contains("<svg"));
/// assert!(!html.contains("data-active"));
/// assert!(!html.contains("data-disabled"));
///
/// // チェック済み + ラベルなし + Disabled。
/// let full = checkbox(None, Size::Md, Active(true), Disabled(true));
/// let full_html = render(&full);
/// assert!(full_html.contains("<svg"));
/// assert!(full_html.contains("data-icon=\"check\""));
/// assert!(full_html.contains(r#"data-active="""#));
/// assert!(full_html.contains(r#"data-disabled="""#));
/// assert!(!full_html.contains("fw-wire-checkbox-label"));
///
/// // `<input>`/`role`/`aria-checked` は一切出力しない
/// // （`docs/design/wireframe-ui-architecture.md` §7）。
/// assert!(!full_html.contains("<input"));
/// assert!(!full_html.contains(" role=\""));
/// assert!(!full_html.contains("aria-checked"));
///
/// // XSS 回帰: ラベルは既定エスケープを経由する。
/// let escaped = checkbox(Some("<script>alert(1)</script>"), Size::Md, Active(false), Disabled(false));
/// let escaped_html = render(&escaped);
/// assert!(!escaped_html.contains("<script>alert(1)</script>"));
/// assert!(escaped_html.contains("&lt;script&gt;"));
/// ```
#[must_use]
pub fn checkbox(label: Option<&str>, size: Size, active: Active, disabled: Disabled) -> Node {
    let class = class_list("fw-wire-checkbox", &[Some(size.class())]);

    let mut attrs: Vec<(String, String)> = vec![("class".to_string(), class)];
    if let Some(attr) = active.attr() {
        attrs.push(attr);
    }
    if let Some(attr) = disabled.attr() {
        attrs.push(attr);
    }

    let mut box_children: Vec<Node> = Vec::new();
    if active.0 {
        box_children.push(icon::check(size));
    }

    let mut children: Vec<Node> = vec![span(vec![("class", BOX_CLASS)], box_children)];
    if let Some(label) = label {
        children.push(span(vec![("class", LABEL_CLASS)], vec![text(label)]));
    }

    el_owned("div", attrs, children)
}
