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
//! [`Size`] 5 段の名称を再利用するが、値は [`crate::size::css`] の
//! スコープ付き変数（`--fw-wire-font-size`/`--fw-wire-control-size`）を
//! 経由しない。`fw-wire-size-*`（`crate::size::css` が生成する共有
//! class）をルートへ付けると、この 2 変数が意図せず子孫へ継承され、
//! 独自に size class を再宣言しない任意の子部品（呼び出し側が
//! `Vec<Node>` に何を渡すかは Stack の関知しない契約）の文字・
//! コントロールサイズまで暗黙に変更してしまう（コードレビュー指摘、
//! イシュー #2610）。レイアウトコンテナの責務を gap に限定するため、
//! Stack は共有 `fw-wire-size-*` を使わず、専用の
//! `fw-wire-stack-gap-<段階>` class（[`gap_class`]）を [`STACK_CSS`] 内に
//! 5 段明示する（イシュー #2610 実装計画 §2「gap の表現」の改訂）。

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
.fw-wire-stack.fw-wire-stack-gap-xs {
  gap: 0.25rem;
}
.fw-wire-stack.fw-wire-stack-gap-sm {
  gap: 0.5rem;
}
.fw-wire-stack.fw-wire-stack-gap-md {
  gap: 1rem;
}
.fw-wire-stack.fw-wire-stack-gap-lg {
  gap: 1.5rem;
}
.fw-wire-stack.fw-wire-stack-gap-xl {
  gap: 2rem;
}
";

/// `gap`（[`Size`]）を Stack 専用の gap class 名（`fw-wire-stack-gap-<段階>`）
/// へ変換する。
///
/// [`crate::size::css`] が生成する共有 `fw-wire-size-*` class は
/// `--fw-wire-font-size`/`--fw-wire-control-size` を同時に定義するため
/// 使わない（モジュール doc「API 設計の由来」参照）。Stack はレイアウト
/// （gap）にのみ責務を持つため、独自 class で子孫への副作用を遮断する。
///
/// [`crate::class::class_list`] の modifiers は `&'static str` に限定
/// （利用者入力が class へ流れ込む経路を型で塞ぐ、イシュー #2605 実装
/// 計画 §3.3）されているため、`Size::class` と同じくリテラルを返す
/// `match` で実装する（`format!` の動的 `String` は使わない）。
const fn gap_class(gap: Size) -> &'static str {
    match gap {
        Size::Xs => "fw-wire-stack-gap-xs",
        Size::Sm => "fw-wire-stack-gap-sm",
        Size::Md => "fw-wire-stack-gap-md",
        Size::Lg => "fw-wire-stack-gap-lg",
        Size::Xl => "fw-wire-stack-gap-xl",
    }
}

/// スタックを組み立てる。
///
/// - `children`: 子ノード列。渡した順序どおりに描画する。空でもルート
///   `div` 自体は出力する。
/// - `orientation`: [`Orientation`]。`Horizontal`（既定）は行方向、
///   `Vertical` は列方向に並べる。
/// - `gap`: [`Size`] 5 段。子要素間の間隔を [`STACK_CSS`] の対応セレクタ
///   （`.fw-wire-stack.fw-wire-stack-gap-<段階>`）で決める。
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
/// assert!(html.contains(r#"class="fw-wire-stack fw-wire-horizontal fw-wire-stack-gap-md""#));
/// assert!(html.starts_with("<div"));
/// assert!(html.trim_end().ends_with("</div>"));
///
/// // Vertical 方向。
/// let vertical = stack(vec![text("A")], Orientation::Vertical, Size::Sm);
/// assert!(render(&vertical).contains(r#"class="fw-wire-stack fw-wire-vertical fw-wire-stack-gap-sm""#));
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
        &[Some(orientation.class()), Some(gap_class(gap))],
    );

    el_owned("div", vec![("class".to_string(), class)], children)
}
