//! パンくずリスト部品（`Breadcrumbs`、イシュー #2639、Phase 5「Navigation」）。
//!
//! 上位階層 → 現在ページへ至る経路の配置イメージだけを示す、
//! 非インタラクティブなローファイ・プレースホルダー。リンク遷移・
//! `nav`/`ol`/`li`/`a[href]`/`aria-current` のいずれも実装しない
//! 表示専用部品である点に注意する。
//!
//! # 呼び出し文脈
//!
//! `fandhe-frontend-docs-site` の `wireframes::breadcrumbs` showcase
//! （`/wireframes/breadcrumbs/`）から呼ばれる。`fandhe_frontend_core::text`
//! のみでラベルを流し込むため、既定エスケープ（REQ-1）は本モジュールが
//! 独自に保証する必要はなく core 側の契約に委譲される。
//!
//! # API 設計の由来
//!
//! `docs/design/wireframe-ui-architecture.md` §6 の汎用変換規約と、
//! Phase 5 の先行部品（[`crate::tabs`]/[`crate::nav_item`]）の先例から
//! 独立設計した（`site/wireframes/breadcrumbs.md` の「原案差分メモ」節も
//! 参照）。blocks.pm の Breadcrumbs 部品が持つ Figma プロパティ構成
//! （`Size`/`Levels`/`Level text` ×N の固定スロット）は書き写さない
//! （同文書 §2、イシュー #2602）。固定スロットの代わりに階層ラベルは
//! `&[&str]` スライスで受け、項目数の上限を設けない。
//!
//! **現在階層（選択状態）は新しい専用型を導入せず、既存の共通型
//! [`crate::props::Active`] を再利用する**（[`crate::radio`]/[`crate::tabs`]
//! の先例と同じ判断）。ただし `tabs` の `active: Option<usize>` とは異なり、
//! 本部品は引数を一切持たない: パンくずは定義上「現在ページで終わる経路」
//! であり、現在項目は常に最後の要素である。この不変条件を利用者に選ばせず
//! 構造で保証するため、`items` が空でない限り最後の項目へ常に
//! `data-active=""` を付与する（不正な組み合わせ状態が型レベルで
//! そもそも発生しない、`tabs` の `Option<usize>` より不正状態が少ない
//! 設計）。
//!
//! 区切り記号は **CSS 擬似要素のみ**で描く（[`crate::stepper`] の連結線
//! `::before` と同じ先例）。DOM へ区切りノード・テキストを出力しない。
//! [`crate::icon::caret_right`] 等の SVG アイコンによる区切りは
//! `aria-hidden` 付き SVG を出力し非対話テストを複雑にするため不採用
//! （`stepper` と同じ理由）。
//!
//! [`crate::props::Orientation`]（パンくずは水平専用）、`Disabled`/`Bold`/
//! `Primary`、先頭のホームアイコン `Node` スロット、中間階層の省略
//! （「…」折りたたみ）は意図的に持たない（`site/wireframes/breadcrumbs.md`
//! の原案差分メモ参照）。
//!
//! # 対話セマンティクスは出力しない（最重要）
//!
//! `docs/design/wireframe-ui-architecture.md` §7（非対話制約）に従い、
//! ルートは `div` とし `role`/`aria-*`/`tabindex`/`on*`/`style` は一切
//! 出力しない。`<nav>`/`<ol>`/`<li>`/`<a>`/`<button>`/`<input>`/`<select>`
//! も出力しない。実際に操作可能なパンくずが必要な利用者には
//! Primitives/Themes の Breadcrumb を案内する（`site/wireframes/breadcrumbs.md`
//! 参照）。

use fandhe_frontend_core::{el_owned, text, Node};

use crate::class::class_list;
use crate::props::Active;
use crate::size::Size;

/// パンくず項目 1 件のパート class（部品ルートなしで単独使用しない、
/// [`breadcrumbs`] 専用）。
const ITEM_CLASS: &str = "fw-wire-breadcrumbs-item";

/// パンくずリスト CSS（グレースケール、`ColorPalette` 非依存）。
/// [`crate::css::PARTS`] へ登録される。
///
/// `[data-active]` 単独セレクタは `crates/wireframe-ui/tests/common_api.rs`
/// の「`.` で始まる行はすべて `.fw-wire-` プレフィックス」走査に引っかから
/// ないよう `.fw-wire-breadcrumbs-item[data-active]` の形で書く
/// （`docs/design/wireframe-ui-architecture.md` §10.4）。区切り記号は
/// 隣接項目の `::before` 擬似要素にのみ存在し、DOM のテキストノードとは
/// 無関係（利用者入力と混ざらない）。
pub const BREADCRUMBS_CSS: &str = "\
.fw-wire-breadcrumbs {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  box-sizing: border-box;
  font-family: var(--fw-wire-font-family);
  font-size: var(--fw-wire-font-size, 1rem);
  color: var(--fw-wire-ink-muted);
}
.fw-wire-breadcrumbs-item {
  box-sizing: border-box;
  white-space: nowrap;
  color: var(--fw-wire-ink-muted);
}
.fw-wire-breadcrumbs-item + .fw-wire-breadcrumbs-item {
  margin-left: 0.4em;
}
.fw-wire-breadcrumbs-item + .fw-wire-breadcrumbs-item::before {
  content: \"/\";
  display: inline-block;
  margin-right: 0.4em;
  color: var(--fw-wire-line);
}
.fw-wire-breadcrumbs-item[data-active] {
  color: var(--fw-wire-ink);
  font-weight: 600;
}
";

/// パンくずリストを組み立てる。
///
/// - `items`: 階層ラベル列。上位階層から現在ページの順で渡す。固定
///   スロットの bool 列ではなくスライスで受け、要素数の上限は設けない。
///   空スライスのときは項目を持たないルート要素だけを出力する
///   （panic しない）。
/// - `size`: [`Size`] 5 段。ルート class `fw-wire-size-<段階>` として
///   付与し、フォントサイズは [`crate::size::css`] が定義する
///   `--fw-wire-font-size` を `em` 基準で参照する（本モジュールは値を
///   書き写さない）。
///
/// 現在階層は選択専用の引数を持たず、`items` が空でない限り常に最後の
/// 項目へ [`Active`]（`data-active=""`）を付与する（モジュール doc
/// 参照）。区切り記号は CSS の `::before` 擬似要素のみで描き、DOM へ
/// 区切りノードを出力しない。テキストは [`fandhe_frontend_core::text`]
/// のみで流し込み（REQ-1 既定エスケープ）、`role`/`aria-*`/`tabindex`/
/// `style`/`on*`/`href` は一切出力しない。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_wireframe_ui::{breadcrumbs, Size};
///
/// let node = breadcrumbs(&["ホーム", "商品", "詳細"], Size::Md);
/// let html = render(&node);
/// assert!(html.contains(r#"class="fw-wire-breadcrumbs fw-wire-size-md""#));
/// assert_eq!(html.matches(r#"class="fw-wire-breadcrumbs-item""#).count(), 3);
/// assert_eq!(html.matches(r#"data-active="""#).count(), 1);
/// assert!(html.contains("詳細"));
///
/// // 現在階層（data-active）は常に最後の項目。
/// let idx_active = html.find(r#"data-active="""#).unwrap();
/// let idx_last_label = html.rfind("詳細").unwrap();
/// assert!(idx_active < idx_last_label);
///
/// // 区切り文字はテキストノードとして出力しない。
/// assert!(!html.contains(">/<"));
///
/// // 1 項目のみでもその項目が現在階層になる。
/// let single = breadcrumbs(&["ホーム"], Size::Md);
/// assert_eq!(render(&single).matches(r#"data-active="""#).count(), 1);
///
/// // 空スライスは panic せず項目 0 件になる。
/// let empty = breadcrumbs(&[], Size::Md);
/// assert!(!render(&empty).contains("fw-wire-breadcrumbs-item"));
///
/// // XSS 回帰: ラベルは既定エスケープを経由する。
/// let escaped = breadcrumbs(&["<script>alert(1)</script>"], Size::Md);
/// let escaped_html = render(&escaped);
/// assert!(!escaped_html.contains("<script>alert(1)</script>"));
/// assert!(escaped_html.contains("&lt;script&gt;"));
///
/// // 対話セマンティクスは一切出力しない。
/// assert!(!html.contains(" role=\""));
/// assert!(!html.contains(" aria-"));
/// assert!(!html.contains("<a "));
/// assert!(!html.contains("href="));
/// ```
#[must_use]
pub fn breadcrumbs(items: &[&str], size: Size) -> Node {
    let class = class_list("fw-wire-breadcrumbs", &[Some(size.class())]);

    let children: Vec<Node> = items
        .iter()
        .enumerate()
        .map(|(index, label)| {
            let is_current = index + 1 == items.len();
            let mut attrs: Vec<(String, String)> =
                vec![("class".to_string(), ITEM_CLASS.to_string())];
            if let Some(attr) = Active(is_current).attr() {
                attrs.push(attr);
            }
            el_owned("span", attrs, vec![text(*label)])
        })
        .collect();

    el_owned("div", vec![("class".to_string(), class)], children)
}
