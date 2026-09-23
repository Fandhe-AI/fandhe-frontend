//! スイッチ部品（`Switch`、イシュー #2627、Phase 3「Forms A」）。
//!
//! 楕円トラック + つまみ + 任意のラベルを持つ、非インタラクティブな
//! ローファイ・プレースホルダー。「Switch」という名前だが、
//! `<input type="checkbox">`・`role="switch"`・`aria-checked`・クリック
//! 操作のいずれも実装しない表示専用部品である点に注意する。
//!
//! # 呼び出し文脈
//!
//! `fandhe-frontend-docs-site` の `wireframes::switch` showcase
//! （`/wireframes/switch/`）から呼ばれる。`fandhe_frontend_core::text`
//! のみでラベルを流し込むため、既定エスケープ（REQ-1）は本モジュールが
//! 独自に保証する必要はなく core 側の契約に委譲される。
//!
//! # API 設計の由来
//!
//! `docs/design/wireframe-ui-architecture.md` §6 の汎用変換規約から独立
//! 設計した（`site/wireframes/switch.md` の「原案差分メモ」節も参照）。
//! blocks.pm の外観・anatomy・プロパティ構成の実装への転用は書面許諾が
//! 得られるまで保留されている（同文書 §2、イシュー #2602）ため、本部品の
//! 引数構成は blocks.pm の Figma プロパティを参照・書き写さず、
//! [`crate::annotation`] と同型の「bool + テキストを 1 スロットへ畳み込む」
//! パターンから独立に起こした。「ラベルの有無」は専用の bool 引数ではなく
//! `label: Option<&str>` へ畳み込む。
//!
//! `active`（[`crate::props::Active`]）は本部品では ON 状態そのものを表す
//! （`select`/`button` 系のフォーカス風強調とは意味が異なる。部品ごとの
//! 意味の違いは `site/wireframes/switch.md` の原案差分メモにも明記する）。
//! `disabled`（[`crate::props::Disabled`]）は Figma 原案には無い自己定義
//! プロパティだが、Phase 3「Forms A」の他部品（`input`/`select` 等）と
//! 状態軸を揃えるために併用する。`Bold`/`Primary` は使わない（トグルに
//! 太字・強調軸を持たせる根拠がないため）。
//!
//! # 対話的な要素・属性は出力しない（最重要）
//!
//! `docs/design/wireframe-ui-architecture.md` §7（非対話制約）に従い、
//! ルートは `div` とし `role`/`aria-*`/`tabindex`/`style`/`on*` は一切
//! 出力しない。`<input type="checkbox">` も出力しない。実際に操作可能な
//! スイッチが必要な利用者には Primitives/Themes の Switch を案内する
//! （`site/wireframes/switch.md` 参照）。

use fandhe_frontend_core::{el_owned, span, text, Node};

use crate::class::class_list;
use crate::props::{Active, Disabled};
use crate::size::Size;

/// トラックのパート class（部品ルートなしで単独使用しない、[`switch`] 専用）。
const TRACK_CLASS: &str = "fw-wire-switch-track";

/// つまみのパート class（部品ルートなしで単独使用しない、[`switch`] 専用）。
const THUMB_CLASS: &str = "fw-wire-switch-thumb";

/// ラベルのパート class（部品ルートなしで単独使用しない、[`switch`] 専用）。
const LABEL_CLASS: &str = "fw-wire-switch-label";

/// スイッチ CSS（8 セレクタ）。[`crate::css::PARTS`] へ登録される。
///
/// トラック寸法・フォントサイズは値を書き写さず [`crate::size::css`] が
/// 定義する `--fw-wire-control-size`/`--fw-wire-font-size` を `var()` で
/// 参照する。`[data-active]`/`[data-disabled]` 単独セレクタは
/// `crates/wireframe-ui/tests/common_api.rs` の「`.` で始まる行はすべて
/// `.fw-wire-` プレフィックス」走査に引っかからないよう
/// `.fw-wire-switch[data-active]`/`.fw-wire-switch[data-disabled]` の形で
/// 書く（`docs/design/wireframe-ui-architecture.md` §10.4）。ON 状態は
/// 黒塗り二値ではなく `ink` トークン 1 段のグレースケール反転で表す
/// （設計文書 §3）。
pub const SWITCH_CSS: &str = "\
.fw-wire-switch {
  display: inline-flex;
  align-items: center;
  gap: 0.5em;
  min-height: var(--fw-wire-control-size, 2rem);
  color: var(--fw-wire-ink);
  font-family: var(--fw-wire-font-family);
  font-size: var(--fw-wire-font-size, 1rem);
  user-select: none;
}
.fw-wire-switch-track {
  position: relative;
  box-sizing: border-box;
  flex: 0 0 auto;
  width: var(--fw-wire-control-size, 2rem);
  height: calc(var(--fw-wire-control-size, 2rem) * 0.55);
  border: var(--fw-wire-line-width) solid var(--fw-wire-line);
  border-radius: 999px;
  background: var(--fw-wire-fill-subtle);
}
.fw-wire-switch-thumb {
  position: absolute;
  top: 50%;
  left: 0.125em;
  transform: translateY(-50%);
  height: calc(100% - 0.25em);
  aspect-ratio: 1 / 1;
  box-sizing: border-box;
  border: var(--fw-wire-line-width) solid var(--fw-wire-line);
  border-radius: 50%;
  background: var(--fw-wire-paper);
}
.fw-wire-switch-label {
  white-space: nowrap;
}
.fw-wire-switch[data-active] .fw-wire-switch-track {
  background: var(--fw-wire-ink);
  border-color: var(--fw-wire-ink);
}
.fw-wire-switch[data-active] .fw-wire-switch-thumb {
  left: auto;
  right: 0.125em;
  border-color: var(--fw-wire-paper);
}
.fw-wire-switch[data-disabled] {
  opacity: 0.5;
}
.fw-wire-switch[data-disabled] .fw-wire-switch-track {
  border-style: dashed;
}
";

/// スイッチを組み立てる。
///
/// - `label`: 省略可能なラベル文言。`None` のときはパート要素自体を
///   出力しない（空要素を残さない）。
/// - `size`: [`Size`] 5 段。ルート class `fw-wire-size-<段階>` として付与し、
///   トラック寸法・フォントサイズは [`crate::size::css`] が定義する
///   `--fw-wire-control-size`/`--fw-wire-font-size` を `var()` で参照する
///   （本モジュールは値を書き写さない）。
/// - `active`: `true` のとき `data-active=""` を付与する（本部品では
///   ON 状態そのものを表す）。
/// - `disabled`: `true` のとき `data-disabled=""` を付与する（見た目のみ。
///   操作不能を実装するものではない）。
///
/// テキストは [`fandhe_frontend_core::text`] のみで流し込み（REQ-1 既定
/// エスケープ）、`<input`・`role`/`aria-*`/`tabindex`/`style`/`on*` は
/// 一切出力しない。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_wireframe_ui::{switch, Active, Disabled, Size};
///
/// let node = switch(None, Size::Md, Active(false), Disabled(false));
/// let html = render(&node);
/// assert!(html.contains(r#"class="fw-wire-switch fw-wire-size-md""#));
/// assert!(html.contains(r#"class="fw-wire-switch-track""#));
/// assert!(html.contains(r#"class="fw-wire-switch-thumb""#));
/// assert!(!html.contains("fw-wire-switch-label"));
/// assert!(!html.contains("data-active"));
/// assert!(!html.contains("data-disabled"));
///
/// // ラベル + ON + Disabled。
/// let full = switch(Some("通知"), Size::Md, Active(true), Disabled(true));
/// let full_html = render(&full);
/// assert!(full_html.contains(r#"class="fw-wire-switch-label""#));
/// assert!(full_html.contains("通知"));
/// assert!(full_html.contains(r#"data-active="""#));
/// assert!(full_html.contains(r#"data-disabled="""#));
///
/// // `<input`/`role`/`aria-*`/`tabindex` は一切出力しない
/// // （`docs/design/wireframe-ui-architecture.md` §7）。
/// assert!(!full_html.contains("<input"));
/// assert!(!full_html.contains(" role=\""));
/// assert!(!full_html.contains("aria-"));
/// assert!(!full_html.contains("tabindex"));
///
/// // XSS 回帰: ラベルは既定エスケープを経由する。
/// let escaped = switch(Some("<script>alert(1)</script>"), Size::Md, Active(false), Disabled(false));
/// let escaped_html = render(&escaped);
/// assert!(!escaped_html.contains("<script>alert(1)</script>"));
/// assert!(escaped_html.contains("&lt;script&gt;"));
/// ```
#[must_use]
pub fn switch(label: Option<&str>, size: Size, active: Active, disabled: Disabled) -> Node {
    let class = class_list("fw-wire-switch", &[Some(size.class())]);

    let mut attrs: Vec<(String, String)> = vec![("class".to_string(), class)];
    if let Some(attr) = active.attr() {
        attrs.push(attr);
    }
    if let Some(attr) = disabled.attr() {
        attrs.push(attr);
    }

    let mut children: Vec<Node> = vec![span(
        vec![("class", TRACK_CLASS)],
        vec![span(vec![("class", THUMB_CLASS)], vec![])],
    )];
    if let Some(label) = label {
        children.push(span(vec![("class", LABEL_CLASS)], vec![text(label)]));
    }

    el_owned("div", attrs, children)
}
