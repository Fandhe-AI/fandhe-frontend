//! セレクト部品（`Select`、イシュー #2624、Phase 3「Forms A」）。
//!
//! ドロップダウン選択欄の配置イメージだけを示す、非インタラクティブな
//! ローファイ・プレースホルダー。「Select」という名前だが、`<select>`
//! 要素・開閉・リストボックス・キーボード操作のいずれも実装しない表示
//! 専用部品である点に注意する。
//!
//! # 呼び出し文脈
//!
//! `fandhe-frontend-docs-site` の `wireframes::select` showcase
//! （`/wireframes/select/`）から呼ばれる。`fandhe_frontend_core::text`
//! のみで表示文言を流し込むため、既定エスケープ（REQ-1）は本モジュールが
//! 独自に保証する必要はなく core 側の契約に委譲される。
//!
//! # API 設計の由来
//!
//! `docs/design/wireframe-ui-architecture.md` §6 の汎用変換規約と §11.4 の
//! `Option<Node>` アイコンスロット規約から独立設計した（`site/wireframes/select.md`
//! の「原案差分メモ」節も参照）。blocks.pm の外観・anatomy・プロパティ構成
//! の実装への転用は書面許諾が得られるまで保留されている（同文書 §2、
//! イシュー #2602）ため、本部品の引数構成は blocks.pm の Figma プロパティを
//! 参照・書き写さず、[`crate::button`]/[`crate::link`] と同型の汎用パターン
//! から独立に起こした。「アイコンの有無」は専用の bool 引数ではなく
//! `leading: Option<Node>` スロットへ畳み込み、「フォーカス風の強調状態」
//!「無効状態」は共通型 [`crate::props::Active`]/[`crate::props::Disabled`]
//! の `data-*` へ畳み込む（専用の `SelectState` 列挙は追加しない）。開いた
//! 状態（リストボックス表示）の variant は非対話制約（下記）により実装
//! しない。
//!
//! ドロップダウン指示子（⌄）はスロットではなく部品固有の固定パートとし、
//! [`crate::icon::caret_down`] を常に末尾へ出力する（利用者が省略・差し替え
//! できない部品の同一性を担う要素のため）。
//!
//! # `<select>` 要素は出力しない（最重要）
//!
//! `docs/design/wireframe-ui-architecture.md` §7（非対話制約）に従い、
//! ルートは `div` とし `role`/`aria-*`（アイコン基盤が付与する装飾用の
//! `aria-hidden="true"` を除く）/`tabindex`/`style`/`on*` は一切出力しない。
//! 実際に操作可能な select が必要な利用者には Themes（`/themes/select/`）/
//! Primitives（`/primitives/select/`）を案内する（`site/wireframes/select.md`
//! 参照）。

use fandhe_frontend_core::{el_owned, span, text, Node};

use crate::class::class_list;
use crate::icon;
use crate::props::{Active, Disabled};
use crate::size::Size;

/// 表示文言のパート class（部品ルートなしで単独使用しない、[`select`] 専用）。
const TEXT_CLASS: &str = "fw-wire-select-text";

/// ドロップダウン指示子を包むラッパ class（先頭アイコンスロットと区別する
/// ための固定パート、部品ルートなしで単独使用しない、[`select`] 専用）。
const INDICATOR_CLASS: &str = "fw-wire-select-indicator";

/// セレクト CSS（6 セレクタ）。[`crate::css::PARTS`] へ登録される。
///
/// 高さ・フォントサイズは値を書き写さず [`crate::size::css`] が定義する
/// `--fw-wire-control-size`/`--fw-wire-font-size` を `var()` で参照する。
/// `[data-active]`/`[data-disabled]` 単独セレクタは
/// `crates/wireframe-ui/tests/common_api.rs` の「`.` で始まる行はすべて
/// `.fw-wire-` プレフィックス」走査に引っかからないよう
/// `.fw-wire-select[data-active]`/`.fw-wire-select[data-disabled]` の形で
/// 書く（`docs/design/wireframe-ui-architecture.md` §10.4）。
pub const SELECT_CSS: &str = "\
.fw-wire-select {
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
  color: var(--fw-wire-ink);
  font-family: var(--fw-wire-font-family);
  font-size: var(--fw-wire-font-size, 1rem);
  user-select: none;
}
.fw-wire-select-text {
  flex: 1 1 auto;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.fw-wire-select-indicator {
  display: inline-flex;
  margin-left: auto;
}
.fw-wire-select .fw-wire-icon-glyph {
  color: var(--fw-wire-ink-muted);
}
.fw-wire-select[data-active] {
  border-color: var(--fw-wire-ink);
  box-shadow: 0 0 0 1px var(--fw-wire-ink);
}
.fw-wire-select[data-disabled] {
  opacity: 0.5;
  border-style: dashed;
  background: var(--fw-wire-fill-subtle);
}
";

/// セレクトを組み立てる。
///
/// - `text_content`: 表示文言（選択済み値・プレースホルダー風のいずれも
///   1 種類の文言として扱う。空文字も可）。
/// - `leading`: 省略可能な先頭アイコンスロット（`docs/design/wireframe-ui-architecture.md`
///   §11.4）。`Some(icon::user(size))` のように渡す。`None` のときは
///   アイコン要素自体を出力しない。ドロップダウン指示子とは別の要素であり、
///   指示子は常に末尾へ出力される（下記参照）。
/// - `size`: [`Size`] 5 段。ルート class `fw-wire-size-<段階>` として付与し、
///   コントロール高さ・フォントサイズは [`crate::size::css`] が定義する
///   `--fw-wire-control-size`/`--fw-wire-font-size` を `var()` で参照する
///   （本モジュールは値を書き写さない）。
/// - `active`: `true` のとき `data-active=""` を付与する（フォーカス風の
///   強調枠。実際のフォーカス管理を実装するものではない）。
/// - `disabled`: `true` のとき `data-disabled=""` を付与する（見た目のみ。
///   操作不能を実装するものではない）。
///
/// テキストは [`fandhe_frontend_core::text`] のみで流し込み（REQ-1 既定
/// エスケープ）、`<select>`・`role`/`aria-*`（アイコン基盤の装飾用
/// `aria-hidden` を除く）/`tabindex`/`style`/`on*` は一切出力しない。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_wireframe_ui::{icon, select, Active, Disabled, Size};
///
/// let node = select("未選択", None, Size::Md, Active(false), Disabled(false));
/// let html = render(&node);
/// assert!(html.contains(r#"class="fw-wire-select fw-wire-size-md""#));
/// assert!(html.contains(r#"class="fw-wire-select-text""#));
/// assert!(html.contains("未選択"));
/// assert!(html.contains(r#"class="fw-wire-select-indicator""#));
/// assert!(html.contains(r#"data-icon="caret-down""#));
/// assert!(!html.contains("data-active"));
/// assert!(!html.contains("data-disabled"));
///
/// // 先頭アイコン + Active + Disabled。
/// let full = select(
///     "山田太郎",
///     Some(icon::user(Size::Md)),
///     Size::Md,
///     Active(true),
///     Disabled(true),
/// );
/// let full_html = render(&full);
/// assert!(full_html.contains(r#"data-active="""#));
/// assert!(full_html.contains(r#"data-disabled="""#));
///
/// // `<select>`/`role`/`tabindex` は一切出力しない
/// // （`docs/design/wireframe-ui-architecture.md` §7）。
/// assert!(!full_html.contains("<select"));
/// assert!(!full_html.contains(" role=\""));
/// assert!(!full_html.contains("tabindex"));
///
/// // XSS 回帰: 表示文言は既定エスケープを経由する。
/// let escaped = select("<script>alert(1)</script>", None, Size::Md, Active(false), Disabled(false));
/// let escaped_html = render(&escaped);
/// assert!(!escaped_html.contains("<script>alert(1)</script>"));
/// assert!(escaped_html.contains("&lt;script&gt;"));
/// ```
#[must_use]
pub fn select(
    text_content: &str,
    leading: Option<Node>,
    size: Size,
    active: Active,
    disabled: Disabled,
) -> Node {
    let class = class_list("fw-wire-select", &[Some(size.class())]);

    let mut attrs: Vec<(String, String)> = vec![("class".to_string(), class)];
    if let Some(attr) = active.attr() {
        attrs.push(attr);
    }
    if let Some(attr) = disabled.attr() {
        attrs.push(attr);
    }

    let mut children: Vec<Node> = Vec::new();
    if let Some(leading) = leading {
        children.push(leading);
    }
    children.push(span(vec![("class", TEXT_CLASS)], vec![text(text_content)]));
    children.push(span(
        vec![("class", INDICATOR_CLASS)],
        vec![icon::caret_down(size)],
    ));

    el_owned("div", attrs, children)
}
