//! レイアウトコンテナ部品（`Stack`、イシュー #2610、Phase 1「レイアウト骨格」）。
//!
//! 子要素を縦または横に等間隔で並べる純粋なレイアウトコンテナ。
//! blocks.pm に対応部品はなく、wireframe-ui 独自追加 14 部品の 1 つ
//! （`docs/design/wireframe-ui-architecture.md` §8）。
//!
//! # 呼び出し文脈
//!
//! `fandhe-frontend-docs-site` の `wireframes::stack` showcase
//! （`/wireframes/stack/`）から呼ばれる。子ノードはそのまま子として
//! 描画するのみで、テキストの既定エスケープ（REQ-1）は子ノードを構築する
//! 呼び出し元（`fandhe_frontend_core::text` 等）の契約に委譲される。
//!
//! # API 設計の由来
//!
//! 方向は新型を導入せず [`crate::props::Orientation`] を再利用する
//! （同型の rustdoc が stack を消費者として明記済み、
//! `docs/design/wireframe-ui-architecture.md` §10.1 が
//! `fw-wire-horizontal`/`fw-wire-vertical` を割り当て済み）。間隔は
//! [`Size`] 5 段を再利用するが、[`crate::size::css`] のスコープ付き変数
//! （`--fw-wire-font-size`/`--fw-wire-control-size`）には gap 用の値が
//! 無いため、[`STACK_CSS`] 内に `gap` を 5 段明示する（イシュー #2610
//! 実装計画 §2「gap の表現」）。副作用として、ルートへ `fw-wire-size-*`
//! を付けると `--fw-wire-font-size` も配下へ継承されるが、既存部品
//! （annotation・icon glyph）はいずれも自身のルートで size class を
//! 再宣言するため実害はない。

use fandhe_frontend_core::{el_owned, Node};

use crate::class::class_list;
use crate::props::Orientation;
use crate::size::Size;

/// スタック CSS（8 セレクタ: ルート・水平・垂直・`Size` 5 段の gap）。
/// [`crate::css::PARTS`] へ登録される。
pub const STACK_CSS: &str = "\
.fw-wire-stack {
  display: flex;
  box-sizing: border-box;
  max-width: 100%;
  gap: 1rem;
}
.fw-wire-stack.fw-wire-horizontal {
  flex-direction: row;
  flex-wrap: wrap;
  align-items: center;
}
.fw-wire-stack.fw-wire-vertical {
  flex-direction: column;
  align-items: stretch;
}
.fw-wire-stack.fw-wire-size-xs {
  gap: 0.25rem;
}
.fw-wire-stack.fw-wire-size-sm {
  gap: 0.5rem;
}
.fw-wire-stack.fw-wire-size-md {
  gap: 1rem;
}
.fw-wire-stack.fw-wire-size-lg {
  gap: 1.5rem;
}
.fw-wire-stack.fw-wire-size-xl {
  gap: 2rem;
}
";

/// スタックを組み立てる。
///
/// - `children`: 子ノード列。渡した順序どおりに描画する。空でもルート
///   `div` 自体は出力する。
/// - `orientation`: [`Orientation`]。`Horizontal`（既定）は行方向、
///   `Vertical` は列方向に並べる。
/// - `gap`: [`Size`] 5 段。子要素間の間隔を [`STACK_CSS`] の対応セレクタ
///   （`.fw-wire-stack.fw-wire-size-<段階>`）で決める。
///
/// ルートは `div.fw-wire-stack` のみで、子要素はラップせずそのまま
/// flex アイテムとして描画する。`role`/`aria-*`/`tabindex`/`style`/`data-*`
/// は一切付与しない（表示状態軸を持たない部品、annotation と同じ）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::{render, text};
/// use fandhe_frontend_wireframe_ui::{stack, Orientation, Size};
///
/// let node = stack(vec![text("A"), text("B")], Orientation::Horizontal, Size::Md);
/// let html = render(&node);
/// assert!(html.contains(r#"class="fw-wire-stack fw-wire-horizontal fw-wire-size-md""#));
/// assert!(html.starts_with("<div"));
/// assert!(html.trim_end().ends_with("</div>"));
///
/// // Vertical 方向。
/// let vertical = stack(vec![text("A")], Orientation::Vertical, Size::Sm);
/// assert!(render(&vertical).contains(r#"class="fw-wire-stack fw-wire-vertical fw-wire-size-sm""#));
///
/// // 空の children でもルート div は出力される。
/// let empty = stack(vec![], Orientation::Horizontal, Size::Md);
/// assert!(render(&empty).contains("<div"));
///
/// // XSS 回帰: 子として渡した text はいずれも既定エスケープを経由する。
/// let escaped = stack(
///     vec![text("<script>alert(1)</script>")],
///     Orientation::Horizontal,
///     Size::Md,
/// );
/// let escaped_html = render(&escaped);
/// assert!(!escaped_html.contains("<script>alert(1)</script>"));
/// assert!(escaped_html.contains("&lt;script&gt;"));
/// ```
#[must_use]
pub fn stack(children: Vec<Node>, orientation: Orientation, gap: Size) -> Node {
    let class = class_list(
        "fw-wire-stack",
        &[Some(orientation.class()), Some(gap.class())],
    );

    el_owned("div", vec![("class".to_string(), class)], children)
}
