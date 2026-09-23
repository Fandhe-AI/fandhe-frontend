//! 列数固定グリッド配置部品（`Grid`、イシュー #2611、Phase 1「レイアウト骨格」）。
//!
//! 画面設計図で「N 列のグリッドにこれだけの要素が並ぶ」という配置イメージを
//! 伝えるための、非インタラクティブなローファイ・プレースホルダー。blocks.pm
//! に対応する部品は無く、wireframe-ui 独自追加部品である
//! （`site/wireframes.md` Phase 1 一覧・`docs/design/wireframe-ui-architecture.md`
//! §8 の kebab 集合参照）。
//!
//! # 呼び出し文脈
//!
//! `fandhe-frontend-docs-site` の `wireframes::grid` showcase
//! （`/wireframes/grid/`）から呼ばれる。`children` は既に構築済みの
//! [`fandhe_frontend_core::Node`] であり、テキストの直接受け取りは行わない
//! （既定エスケープ・REQ-1 は `children` を構築した呼び出し元・その先の
//! `fandhe_frontend_core::text` が既に保証している）。
//!
//! # API 設計の由来
//!
//! イシュー本文の想定引数（子ノード列・列数・間隔）をそのまま
//! `children: Vec<Node>` / `columns: u32` / `gap: Size` として受け取る。
//! `&[Node]` ではなく `Vec<Node>` を選んだのは、深いノード木の呼び出し元に
//! 不要な `clone()` を強制しないため（`.claude/rules/coding-rust.md`
//! 「不要な `clone()` を避け」）。詳細は `site/wireframes/grid.md` の
//! 「原案差分メモ」節を参照。

use fandhe_frontend_core::{el_owned, Node};

use crate::class::class_list;
use crate::size::Size;

/// `columns` の上限。これを超える値は本値へ、`0` は `1` へ丸める
/// （[`grid`] 参照）。
pub const MAX_COLUMNS: u32 = 12;

/// パート class（部品ルートなしで単独使用しない、[`grid`] 専用）。
const ITEM_CLASS: &str = "fw-wire-grid-item";

/// 列数 1〜[`MAX_COLUMNS`] それぞれに対応する root 修飾 class
/// （`fw-wire-grid-cols-<n>`）。`format!` で動的に組み立てず `&'static str`
/// リテラルの固定集合から返す（`class_list` の型制約に揃える、A03）。
const fn columns_class(columns: u32) -> &'static str {
    match columns {
        1 => "fw-wire-grid-cols-1",
        2 => "fw-wire-grid-cols-2",
        3 => "fw-wire-grid-cols-3",
        4 => "fw-wire-grid-cols-4",
        5 => "fw-wire-grid-cols-5",
        6 => "fw-wire-grid-cols-6",
        7 => "fw-wire-grid-cols-7",
        8 => "fw-wire-grid-cols-8",
        9 => "fw-wire-grid-cols-9",
        10 => "fw-wire-grid-cols-10",
        11 => "fw-wire-grid-cols-11",
        _ => "fw-wire-grid-cols-12",
    }
}

/// グリッド CSS（19 セレクタ）。[`crate::css::PARTS`] へ登録される。
///
/// `gap` の値は [`Size`] 段階ごとに本 CSS が直接宣言する（共有ファイル
/// `crate::size` の `SCALE` は変更しない。`.fw-wire-grid` 固有の値のため、
/// 他部品と共有する `--fw-wire-font-size`/`--fw-wire-control-size` とは
/// 別に宣言する設計判断、`site/wireframes/grid.md` 参照）。
pub const GRID_CSS: &str = "\
.fw-wire-grid {
  display: grid;
  box-sizing: border-box;
  max-width: 100%;
  color: var(--fw-wire-ink);
  font-family: var(--fw-wire-font-family);
  font-size: var(--fw-wire-font-size, 1rem);
}
.fw-wire-grid-cols-1 { grid-template-columns: repeat(1, minmax(0, 1fr)); }
.fw-wire-grid-cols-2 { grid-template-columns: repeat(2, minmax(0, 1fr)); }
.fw-wire-grid-cols-3 { grid-template-columns: repeat(3, minmax(0, 1fr)); }
.fw-wire-grid-cols-4 { grid-template-columns: repeat(4, minmax(0, 1fr)); }
.fw-wire-grid-cols-5 { grid-template-columns: repeat(5, minmax(0, 1fr)); }
.fw-wire-grid-cols-6 { grid-template-columns: repeat(6, minmax(0, 1fr)); }
.fw-wire-grid-cols-7 { grid-template-columns: repeat(7, minmax(0, 1fr)); }
.fw-wire-grid-cols-8 { grid-template-columns: repeat(8, minmax(0, 1fr)); }
.fw-wire-grid-cols-9 { grid-template-columns: repeat(9, minmax(0, 1fr)); }
.fw-wire-grid-cols-10 { grid-template-columns: repeat(10, minmax(0, 1fr)); }
.fw-wire-grid-cols-11 { grid-template-columns: repeat(11, minmax(0, 1fr)); }
.fw-wire-grid-cols-12 { grid-template-columns: repeat(12, minmax(0, 1fr)); }
.fw-wire-grid.fw-wire-size-xs { gap: 0.25rem; }
.fw-wire-grid.fw-wire-size-sm { gap: 0.5rem; }
.fw-wire-grid.fw-wire-size-md { gap: 1rem; }
.fw-wire-grid.fw-wire-size-lg { gap: 1.5rem; }
.fw-wire-grid.fw-wire-size-xl { gap: 2rem; }
.fw-wire-grid-item {
  min-width: 0;
  box-sizing: border-box;
  padding: 0.5em;
  border: var(--fw-wire-line-width) dashed var(--fw-wire-line-subtle);
  border-radius: var(--fw-wire-radius);
  background: var(--fw-wire-paper);
}
";

/// 列数固定のグリッド配置プレースホルダーを組み立てる。
///
/// - `children`: グリッドへ配置する子ノード列。各要素は
///   `div.fw-wire-grid-item`（破線境界のセルプレースホルダー）で 1 個ずつ
///   包まれる。空の場合は item を 1 つも出力しない（プレースホルダーの
///   自動生成はしない純関数）。
/// - `columns`: 列数。`1..=`[`MAX_COLUMNS`] の範囲へ丸める
///   （`0` は `1` へ、[`MAX_COLUMNS`] 超過は [`MAX_COLUMNS`] へ）。
/// - `gap`: [`Size`] 5 段。root へ `gap.class()`（`fw-wire-size-<段階>`）を
///   付与し、セル間隔は [`GRID_CSS`] の `.fw-wire-grid.fw-wire-size-<段階>`
///   が定義する。副作用として `--fw-wire-font-size`/`--fw-wire-control-size`
///   も root に設定され子孫へ継承されるが、各 wireframe 部品は自分の
///   size class で上書きするため実害はない（`site/wireframes/grid.md`
///   「原案差分メモ」節参照）。
///
/// root class の順序は `fw-wire-grid fw-wire-size-<gap> fw-wire-grid-cols-<n>`
/// （共通修飾 → 部品固有修飾、[`crate::annotation::annotation`] と同じ並び）。
/// `role`/`aria-*`/`tabindex`/`style`/`data-*` は一切付与しない
/// （`docs/design/wireframe-ui-architecture.md` §5/§7）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::{render, text};
/// use fandhe_frontend_wireframe_ui::{grid, Size};
///
/// let node = grid(vec![text("A"), text("B"), text("C")], 3, Size::Md);
/// let html = render(&node);
/// assert!(html.contains(r#"class="fw-wire-grid fw-wire-size-md fw-wire-grid-cols-3""#));
/// assert_eq!(html.matches(r#"class="fw-wire-grid-item""#).count(), 3);
///
/// // columns は 1..=12 へ丸める。
/// let clamped_low = grid(vec![], 0, Size::Md);
/// assert!(render(&clamped_low).contains("fw-wire-grid-cols-1"));
/// let clamped_high = grid(vec![], 999, Size::Md);
/// assert!(render(&clamped_high).contains("fw-wire-grid-cols-12"));
///
/// // XSS 回帰: 子ノードのテキストは既定エスケープを経由する。
/// let escaped = grid(vec![text("<script>alert(1)</script>")], 2, Size::Md);
/// let escaped_html = render(&escaped);
/// assert!(!escaped_html.contains("<script>alert(1)</script>"));
/// assert!(escaped_html.contains("&lt;script&gt;"));
/// ```
#[must_use]
pub fn grid(children: Vec<Node>, columns: u32, gap: Size) -> Node {
    let columns = columns.clamp(1, MAX_COLUMNS);
    let class = class_list(
        "fw-wire-grid",
        &[Some(gap.class()), Some(columns_class(columns))],
    );

    let items: Vec<Node> = children
        .into_iter()
        .map(|child| {
            el_owned(
                "div",
                vec![("class".to_string(), ITEM_CLASS.to_string())],
                vec![child],
            )
        })
        .collect();

    el_owned("div", vec![("class".to_string(), class)], items)
}
