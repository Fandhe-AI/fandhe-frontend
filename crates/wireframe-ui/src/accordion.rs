//! アコーディオン部品（`Accordion`、イシュー #2641、Phase 5「Navigation」）。
//!
//! 見出し + 開閉キャレット + 本文からなる項目の並びの配置イメージだけを
//! 示す、非インタラクティブなローファイ・プレースホルダー。「Accordion」
//! という名前だが、実際の開閉動作・`<details>`/`<summary>`・
//! `aria-expanded` のいずれも実装しない表示専用部品である点に注意する。
//!
//! # 呼び出し文脈
//!
//! `fandhe-frontend-docs-site` の `wireframes::accordion` showcase
//! （`/wireframes/accordion/`）から呼ばれる。`fandhe_frontend_core::text`
//! のみで見出しを流し込むため、既定エスケープ（REQ-1）は本モジュールが
//! 独自に保証する必要はなく core 側の契約に委譲される。本文はスロット
//! （[`Node`]）として渡された時点で既に描画チェーンのエスケープ・
//! 属性検証を通っている前提のため、本モジュールは検査・再加工せずそのまま
//! 1 子として包む。
//!
//! # blocks.pm 対応部品なし・独自追加部品
//!
//! accordion は blocks.pm カタログに対応部品を持たない
//! **wireframe-ui 独自追加部品**（`docs/design/wireframe-ui-architecture.md`
//! §7 の 14 部品のひとつ）である。したがって外観・プロパティ構成は同文書
//! §4/§6 の blocks.pm 変換規約から独立に設計した（`site/wireframes/accordion.md`
//! の「原案差分メモ」節も参照）。
//!
//! # API 設計の由来
//!
//! 項目は「見出しテキスト・本文 [`Node`] スロット・展開済みフラグ」の組の
//! 列で受ける（固定スロットの bool 列は使わない）。呼び出し側は `Vec` の
//! 所有で渡す（[`crate::stack`]/[`crate::grid`]/[`crate::frame`]/
//! [`crate::question`] が `Node` を子として所有で受ける先例に揃えた設計。
//! 借用スライスで受けると項目ごとに `Node::clone()` が必要になり
//! `coding-rust.md`「不要な `clone()` を避ける」に反するため）。項目数に
//! 上限は設けない（出力は項目数に線形なだけのため）。空 `Vec` のときは
//! 項目を持たないルート要素だけを出力し、panic しない。
//!
//! **展開状態は新しい専用型を導入せず、既存の [`crate::props::Active`] を
//! 再利用する**（switch/radio/tabs の先例と同じ判断）。項目ルートへ
//! `data-active=""` を付与するのみで、`aria-expanded` は出力しない
//! （§7 で明示的に禁止されている）。
//!
//! 折りたたみ（`expanded == false`）の項目は本文 [`Node`] を出力しない
//! （渡された値は捨てる）。静的 SSR プレースホルダーとして隠しコンテンツを
//! 持たず、出力を決定的にするための判断である。`hidden` 属性や CSS で
//! 隠す方式は使わない。
//!
//! `Bold`/`Primary`/`Disabled`/[`crate::props::Orientation`]・先頭アイコン
//! スロットは持たない（責務外）。
//!
//! # 対話セマンティクスは出力しない（最重要）
//!
//! `docs/design/wireframe-ui-architecture.md` §7（非対話制約）に従い、
//! ルート・項目・見出し行はすべて `div`/`span` で組む。**`<details>`/
//! `<summary>` は使わない**（ブラウザ標準で開閉できる対話要素のため）。
//! `<button>`/`<a>`/`<input>`/`role`/`aria-*`（アイコン基盤が付与する装飾用
//! の `aria-hidden="true"` を除く）/`tabindex`/`style`/`on*` も一切出力
//! しない。実際に開閉できるアコーディオンが必要な利用者には Themes
//! （`/themes/accordion/`）/Primitives（`/primitives/accordion/`）を案内
//! する（`site/wireframes/accordion.md` 参照）。

use fandhe_frontend_core::{div, el_owned, text, Node};

use crate::class::class_list;
use crate::icon;
use crate::props::Active;
use crate::size::Size;

/// 項目ルートのパート class（部品ルートなしで単独使用しない、[`accordion`] 専用）。
const ITEM_CLASS: &str = "fw-wire-accordion-item";
/// 見出し行のパート class。
const HEADER_CLASS: &str = "fw-wire-accordion-header";
/// 見出しテキストのパート class。
const TITLE_CLASS: &str = "fw-wire-accordion-title";
/// 開閉キャレットを包むラッパ class。
const INDICATOR_CLASS: &str = "fw-wire-accordion-indicator";
/// 本文のパート class（展開済み項目のみ出力する）。
const BODY_CLASS: &str = "fw-wire-accordion-body";

/// アコーディオン CSS（グレースケール、`ColorPalette` 非依存）。
/// [`crate::css::PARTS`] へ登録される。
///
/// 値は `--fw-wire-*` トークンと [`crate::size::css`] が定義するカスタム
/// プロパティを `var()` で参照するのみで書き写さない。`[data-active]`
/// 単独セレクタは `crates/wireframe-ui/tests/common_api.rs` の「`.` で
/// 始まる行はすべて `.fw-wire-` プレフィックス」走査に引っかからないよう
/// `.fw-wire-accordion-item[data-active]` の形で書く
/// （`docs/design/wireframe-ui-architecture.md` §10.4）。展開中の項目は
/// `--fw-wire-fill-subtle` 背景と `--fw-wire-ink` の強調で示し、黒塗りの
/// 二値表現は使わずグレースケールで読みやすさを優先する（§3 の配色方針）。
pub const ACCORDION_CSS: &str = "\
.fw-wire-accordion {
  display: flex;
  flex-direction: column;
  box-sizing: border-box;
  width: 100%;
  max-width: 32em;
  border: var(--fw-wire-line-width) solid var(--fw-wire-line);
  border-radius: var(--fw-wire-radius);
  font-family: var(--fw-wire-font-family);
  font-size: var(--fw-wire-font-size, 1rem);
  color: var(--fw-wire-ink);
}
.fw-wire-accordion-item {
  box-sizing: border-box;
  border-bottom: var(--fw-wire-line-width) solid var(--fw-wire-line);
}
.fw-wire-accordion-item:last-child {
  border-bottom: none;
}
.fw-wire-accordion-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.5em;
  box-sizing: border-box;
  min-height: var(--fw-wire-control-size, 2rem);
  padding: 0.5em 0.75em;
}
.fw-wire-accordion-title {
  flex: 1 1 auto;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.fw-wire-accordion-indicator {
  display: inline-flex;
  flex: 0 0 auto;
  color: var(--fw-wire-ink-muted);
}
.fw-wire-accordion-body {
  box-sizing: border-box;
  padding: 0 0.75em 0.75em;
  color: var(--fw-wire-ink-muted);
}
.fw-wire-accordion-item[data-active] {
  background: var(--fw-wire-fill-subtle);
}
.fw-wire-accordion-item[data-active] .fw-wire-accordion-title {
  color: var(--fw-wire-ink);
  font-weight: 600;
}
.fw-wire-accordion-item[data-active] .fw-wire-accordion-indicator {
  color: var(--fw-wire-ink);
}
";

/// アコーディオンを組み立てる。
///
/// - `items`: 「見出しテキスト・本文スロット・展開済みか」の組の列。
///   `Vec` の所有で受ける（モジュール doc の「API 設計の由来」参照）。
///   要素数の上限は設けず、空 `Vec` のときは項目を持たないルート要素
///   だけを出力する（panic しない）。
/// - `size`: [`Size`] 5 段。ルート class `fw-wire-size-<段階>` として付与
///   する。
///
/// 展開済み（`expanded == true`）の項目は `data-active=""` を付与し
/// （[`crate::props::Active`] 再利用、モジュール doc参照）、本文スロット
/// を [`BODY_CLASS`] で包んで出力する。折りたたみ（`expanded == false`）の
/// 項目は本文スロットを出力しない（渡された [`Node`] は捨てる）。見出しは
/// [`fandhe_frontend_core::text`] のみで流し込み（REQ-1 既定エスケープ）、
/// `<details>`/`<summary>`/`<button>`/`role`/`aria-*`（アイコン基盤の装飾用
/// `aria-hidden` を除く）/`tabindex`/`style`/`on*`/`aria-expanded` は一切
/// 出力しない。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::{div, render, text};
/// use fandhe_frontend_wireframe_ui::{accordion, Size};
///
/// let body_a = div(vec![("class", "probe-a")], vec![text("本文A")]);
/// let body_b = div(vec![("class", "probe-b")], vec![text("本文B")]);
/// let node = accordion(
///     vec![("項目1", body_a, true), ("項目2", body_b, false)],
///     Size::Md,
/// );
/// let html = render(&node);
/// assert!(html.contains(r#"class="fw-wire-accordion fw-wire-size-md""#));
/// assert_eq!(html.matches(r#"class="fw-wire-accordion-item""#).count(), 2);
/// assert_eq!(html.matches("data-active").count(), 1);
/// assert!(html.contains("項目1"));
/// assert!(html.contains("項目2"));
/// assert!(html.contains("probe-a"));
/// assert!(!html.contains("probe-b"));
/// assert!(html.contains(r#"data-icon="caret-up""#));
/// assert!(html.contains(r#"data-icon="caret-down""#));
///
/// // 空 Vec は panic せず項目 0 件になる。
/// let empty = accordion(vec![], Size::Md);
/// assert!(!render(&empty).contains("fw-wire-accordion-item"));
///
/// // XSS 回帰: 見出しは既定エスケープを経由する。
/// let payload_body = div(vec![], vec![text("c")]);
/// let escaped = accordion(
///     vec![("<script>alert(1)</script>", payload_body, false)],
///     Size::Md,
/// );
/// let escaped_html = render(&escaped);
/// assert!(!escaped_html.contains("<script>alert(1)</script>"));
/// assert!(escaped_html.contains("&lt;script&gt;"));
///
/// // 対話セマンティクスは一切出力しない。
/// assert!(!html.contains("<details"));
/// assert!(!html.contains("<summary"));
/// assert!(!html.contains("<button"));
/// assert!(!html.contains(" role=\""));
/// assert!(!html.contains(" tabindex=\""));
/// assert!(!html.contains("aria-expanded"));
/// ```
#[must_use]
pub fn accordion(items: Vec<(&str, Node, bool)>, size: Size) -> Node {
    let class = class_list("fw-wire-accordion", &[Some(size.class())]);

    let children: Vec<Node> = items
        .into_iter()
        .map(|(title, body, expanded)| {
            let mut item_attrs: Vec<(String, String)> =
                vec![("class".to_string(), ITEM_CLASS.to_string())];
            if let Some(attr) = Active(expanded).attr() {
                item_attrs.push(attr);
            }

            let indicator_icon = if expanded {
                icon::caret_up(size)
            } else {
                icon::caret_down(size)
            };

            let mut item_children: Vec<Node> = vec![div(
                vec![("class", HEADER_CLASS)],
                vec![
                    div(vec![("class", TITLE_CLASS)], vec![text(title)]),
                    div(vec![("class", INDICATOR_CLASS)], vec![indicator_icon]),
                ],
            )];
            if expanded {
                item_children.push(div(vec![("class", BODY_CLASS)], vec![body]));
            }

            el_owned("div", item_attrs, item_children)
        })
        .collect();

    el_owned("div", vec![("class".to_string(), class)], children)
}
