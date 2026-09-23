//! タブ列部品（`Tabs`、イシュー #2638、Phase 5「Navigation」）。
//!
//! タブ項目の並び + 選択中インジケータの配置イメージだけを示す、
//! 非インタラクティブなローファイ・プレースホルダー。「Tabs」という名前
//! だが、`role="tablist"`/`role="tab"`・`aria-selected`・実際のパネル
//! 切り替えのいずれも実装しない表示専用部品である点に注意する。
//!
//! # 呼び出し文脈
//!
//! `fandhe-frontend-docs-site` の `wireframes::tabs` showcase
//! （`/wireframes/tabs/`）から呼ばれる。`fandhe_frontend_core::text`
//! のみでラベルを流し込むため、既定エスケープ（REQ-1）は本モジュールが
//! 独自に保証する必要はなく core 側の契約に委譲される。
//!
//! # API 設計の由来
//!
//! `docs/design/wireframe-ui-architecture.md` §6 の汎用変換規約と、Phase 3
//! 既存部品（[`crate::radio`]/[`crate::select`]）の先例から独立設計した
//! （`site/wireframes/tabs.md` の「原案差分メモ」節も参照）。blocks.pm の
//! Tabs 部品が持つ Figma プロパティ構成（`Size`/`Active tab`/`Tab1`〜`Tab5`
//! の個別 bool+text スロット）は書き写さない（同文書 §2、イシュー #2602）。
//! 固定スロット bool 列の代わりにタブラベルは `&[&str]` スライスで受け、
//! 項目数の上限を設けない（要素ごとにテキスト 1 つだけで出力が線形に
//! 増えるだけであり、固定スロットへ畳み込む理由がない）。
//!
//! **選択状態は新しい専用型を導入せず、既存の共通型 [`crate::props::Active`]
//! を再利用する**（[`crate::radio`] の先例と同じ判断）。個々の項目に
//! `Active` を持たせず、`active: Option<usize>` 1 引数で「どの添字が
//! 選択中か」を表す（`Active` を項目数分のスライスとして受け取ると呼び
//! 出し側が「選択中は高々 1 件」という不変条件を自分で維持する必要が
//! 生じるため、型で保証する設計にした）。範囲外（`items.len()` 以上）の
//! 値や `None` は「どの項目にも選択インジケータを付けない」へ倒し、
//! `unwrap`/`expect`/`panic` は一切使わない。
//!
//! [`crate::props::Orientation`] を必須引数に取る（`props.rs` の doc が
//! 消費者として「stack / divider / tabs / slider」を明記しているため）。
//! `Disabled`/`Bold`/`Primary`、アイコンスロット、パネル領域は意図的に
//! 持たない。パネルは [`crate::frame`] 等との合成で表現する対象であり、
//! タブ列部品自体の責務ではない。
//!
//! # 対話セマンティクスは出力しない（最重要）
//!
//! `docs/design/wireframe-ui-architecture.md` §7（非対話制約）に従い、
//! ルートは `div` とし `role`/`aria-*`/`tabindex`/`on*`/`style` は一切
//! 出力しない。`<button>`/`<a>`/`<input>`/`<select>` も出力しない。実際に
//! 操作可能なタブが必要な利用者には Primitives/Themes の Tabs を案内する
//! （`site/wireframes/tabs.md` 参照）。

use fandhe_frontend_core::{el_owned, text, Node};

use crate::class::class_list;
use crate::props::{Active, Orientation};
use crate::size::Size;

/// タブ項目 1 件のパート class（部品ルートなしで単独使用しない、[`tabs`] 専用）。
const ITEM_CLASS: &str = "fw-wire-tabs-item";

/// タブ列 CSS（グレースケール、`ColorPalette` 非依存）。[`crate::css::PARTS`]
/// へ登録される。
///
/// `[data-active]` 単独セレクタは `crates/wireframe-ui/tests/common_api.rs`
/// の「`.` で始まる行はすべて `.fw-wire-` プレフィックス」走査に引っかから
/// ないよう `.fw-wire-tabs-item[data-active]` の形で書く
/// （`docs/design/wireframe-ui-architecture.md` §10.4）。黒塗り二値ではなく
/// `--fw-wire-ink`/`--fw-wire-ink-muted`/`--fw-wire-fill-subtle` のグレー
/// スケールで選択中項目を強調し、読みやすさを優先する（§3 の配色方針）。
pub const TABS_CSS: &str = "\
.fw-wire-tabs {
  display: flex;
  box-sizing: border-box;
  font-family: var(--fw-wire-font-family);
  font-size: var(--fw-wire-font-size, 1rem);
  color: var(--fw-wire-ink);
}
.fw-wire-tabs.fw-wire-horizontal {
  flex-direction: row;
  border-bottom: var(--fw-wire-line-width) solid var(--fw-wire-line);
}
.fw-wire-tabs.fw-wire-vertical {
  display: inline-flex;
  flex-direction: column;
  border-right: var(--fw-wire-line-width) solid var(--fw-wire-line);
}
.fw-wire-tabs-item {
  box-sizing: border-box;
  padding: 0.5em 1em;
  white-space: nowrap;
  color: var(--fw-wire-ink-muted);
  border-bottom: calc(var(--fw-wire-line-width) * 2) solid transparent;
}
.fw-wire-tabs.fw-wire-vertical .fw-wire-tabs-item {
  border-bottom: none;
  border-right: calc(var(--fw-wire-line-width) * 2) solid transparent;
}
.fw-wire-tabs-item[data-active] {
  color: var(--fw-wire-ink);
  background: var(--fw-wire-fill-subtle);
  border-bottom-color: var(--fw-wire-ink);
}
.fw-wire-tabs.fw-wire-vertical .fw-wire-tabs-item[data-active] {
  border-bottom-color: transparent;
  border-right-color: var(--fw-wire-ink);
}
";

/// タブ列を組み立てる。
///
/// - `items`: タブのラベル列。固定スロットの bool 列ではなくスライスで
///   受け、要素数の上限は設けない。空スライスのときは項目を持たない
///   ルート要素だけを出力する（panic しない）。
/// - `active`: 選択中タブの添字。`None` または `items.len()` 以上の範囲外
///   値のときは、どの項目にも選択インジケータ（`data-active`）を付けない
///   （決定的・fail-closed）。
/// - `orientation`: [`Orientation`] 水平/垂直。ルート class
///   `fw-wire-horizontal`/`fw-wire-vertical` として付与する。
/// - `size`: [`Size`] 5 段。ルート class `fw-wire-size-<段階>` として付与し、
///   フォントサイズは [`crate::size::css`] が定義する `--fw-wire-font-size`
///   を `em` 基準で参照する（本モジュールは値を書き写さない）。
///
/// 選択状態は新しい専用型を導入せず既存の [`crate::props::Active`] を
/// 項目ごとに `Some(active) == Some(index)` の比較結果として適用する
/// （モジュール doc参照）。テキストは [`fandhe_frontend_core::text`] の
/// みで流し込み（REQ-1 既定エスケープ）、`role`/`aria-*`/`tabindex`/
/// `style`/`on*` は一切出力しない。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_wireframe_ui::{tabs, Orientation, Size};
///
/// let node = tabs(&["概要", "詳細", "設定"], Some(0), Orientation::Horizontal, Size::Md);
/// let html = render(&node);
/// assert!(html.contains(r#"class="fw-wire-tabs fw-wire-size-md fw-wire-horizontal""#));
/// assert_eq!(html.matches(r#"class="fw-wire-tabs-item""#).count(), 3);
/// assert_eq!(html.matches(r#"data-active="""#).count(), 1);
/// assert!(html.contains("概要"));
///
/// // 範囲外・None は選択インジケータなし。
/// let none_active = tabs(&["概要", "詳細"], None, Orientation::Horizontal, Size::Md);
/// assert!(!render(&none_active).contains("data-active"));
/// let out_of_range = tabs(&["概要", "詳細"], Some(2), Orientation::Horizontal, Size::Md);
/// assert!(!render(&out_of_range).contains("data-active"));
///
/// // 空スライスは panic せず項目 0 件になる。
/// let empty = tabs(&[], None, Orientation::Horizontal, Size::Md);
/// assert!(!render(&empty).contains("fw-wire-tabs-item"));
///
/// // XSS 回帰: ラベルは既定エスケープを経由する。
/// let escaped = tabs(&["<script>alert(1)</script>"], None, Orientation::Horizontal, Size::Md);
/// let escaped_html = render(&escaped);
/// assert!(!escaped_html.contains("<script>alert(1)</script>"));
/// assert!(escaped_html.contains("&lt;script&gt;"));
///
/// // 対話セマンティクスは一切出力しない。
/// assert!(!html.contains(" role=\""));
/// assert!(!html.contains(" aria-"));
/// assert!(!html.contains(" tabindex=\""));
/// assert!(!html.contains("<button"));
/// ```
#[must_use]
pub fn tabs(items: &[&str], active: Option<usize>, orientation: Orientation, size: Size) -> Node {
    let class = class_list(
        "fw-wire-tabs",
        &[Some(size.class()), Some(orientation.class())],
    );

    // 項目ごとに `data-active` の有無（可変属性）が変わるため、固定
    // シグネチャの `fandhe_frontend_core::span` ヘルパ（`&str` 属性値限定）
    // ではなく `el_owned`（`String` 属性値）で直接組み立てる。
    let children: Vec<Node> = items
        .iter()
        .enumerate()
        .map(|(index, label)| {
            let is_active = active == Some(index);
            let mut attrs: Vec<(String, String)> =
                vec![("class".to_string(), ITEM_CLASS.to_string())];
            if let Some(attr) = Active(is_active).attr() {
                attrs.push(attr);
            }
            el_owned("span", attrs, vec![text(*label)])
        })
        .collect();

    el_owned("div", vec![("class".to_string(), class)], children)
}
