//! 配置コンテナ部品（`Frame`、イシュー #2609、Phase 1「レイアウト骨格」）。
//!
//! padding と境界線だけを持つ矩形のコンテナ。画面設計図上で領域をまとめる
//! 用途に使う、非インタラクティブなローファイ・プレースホルダー。
//!
//! # 呼び出し文脈
//!
//! `fandhe-frontend-docs-site` の `wireframes::frame` showcase
//! （`/wireframes/frame/`）から呼ばれる。テキスト引数を持たず、子は
//! 構築済み `Node`（既定エスケープ済み）で受けるため、REQ-1 の既定
//! エスケープは core の `render()` 契約に委譲される。
//!
//! # API 設計の由来（原案からの差分）
//!
//! blocks.pm に同名部品はなく `wireframe-ui` 独自追加の 1 つ
//! （`docs/design/wireframe-ui-architecture.md` §8）。イシュー本文の想定
//! 引数は「子ノード群・padding の `Size` 段階・境界線の有無 bool」の 3 点
//! だったが、公開 API は子を `&[Node]`（借用）ではなく `Vec<Node>`
//! （所有渡し）で受ける。core のノード木 API（`div` 等）・pre-styled-ui
//! 全部品・wireframe-ui `icon::glyph` がいずれも `Vec<Node>` 所有渡しで
//! あり、`&[Node]` を選ぶと子ツリー全体の `clone()` が毎回発生し
//! `.claude/rules/coding-rust.md`「不要な clone() を避け、借用を優先」に
//! 反するため（差分の記録は `site/wireframes/frame.md` の「原案差分メモ」
//! 節も参照）。

use fandhe_frontend_core::{el_owned, Node};

use crate::class::class_list;
use crate::size::Size;

/// 部品固有の修飾 class（`bordered=true` のときのみ付与）。
const BORDERED_CLASS: &str = "fw-wire-frame-bordered";

/// フレーム CSS（7 セレクタ: ルート・bordered 修飾・padding 5 段）。
/// [`crate::css::PARTS`] へ登録される。
///
/// padding は `crate::size::css` が生成する共有 `fw-wire-size-*`
/// （`--fw-wire-font-size`/`--fw-wire-control-size` を同時定義し子孫へ
/// 継承される）を経由せず、Frame 専用の `fw-wire-frame-padding-<段階>`
/// class（[`padding_class`]）で padding 値を直接宣言する。共有 class を
/// ルートへ付けると、独自に size class を再宣言しない任意の子部品
/// （呼び出し側が `Vec<Node>` に何を渡すかは Frame の関知しない契約。
/// 例: 自身の `fw-wire-size-*` を持たない子）の文字・コントロールサイズ
/// まで暗黙に変更してしまう（コードレビュー指摘、イシュー #2609。
/// `crate::stack` が `fw-wire-stack-gap-*` で同種の問題を先に回避した
/// 前例と同じ設計）。値は `size::SCALE` の `control_size` の 1/2
/// （xs 0.75rem〜xl 1.5rem）。非 bordered でも `transparent` の境界線幅を
/// 確保し、`bordered` の切り替えで子の配置がずれない（決定的な
/// レイアウト）。`font-size` は設定しない（レイアウトコンテナの責務は
/// padding に限定する）。`background` も設定しない（入れ子時に紙面が
/// 透けるようにする）。
pub const FRAME_CSS: &str = "\
.fw-wire-frame {
  display: block;
  box-sizing: border-box;
  min-width: 0;
  border: var(--fw-wire-line-width) solid transparent;
  border-radius: var(--fw-wire-radius);
}
.fw-wire-frame.fw-wire-frame-bordered {
  border-color: var(--fw-wire-line);
}
.fw-wire-frame.fw-wire-frame-padding-xs {
  padding: 0.75rem;
}
.fw-wire-frame.fw-wire-frame-padding-sm {
  padding: 0.875rem;
}
.fw-wire-frame.fw-wire-frame-padding-md {
  padding: 1rem;
}
.fw-wire-frame.fw-wire-frame-padding-lg {
  padding: 1.25rem;
}
.fw-wire-frame.fw-wire-frame-padding-xl {
  padding: 1.5rem;
}
";

/// `padding`（[`Size`]）を Frame 専用の padding class 名
/// （`fw-wire-frame-padding-<段階>`）へ変換する。
///
/// [`crate::size::css`] が生成する共有 `fw-wire-size-*` class は
/// `--fw-wire-font-size`/`--fw-wire-control-size` を同時に定義するため
/// 使わない（モジュール doc「フレーム CSS」節参照）。
const fn padding_class(padding: Size) -> &'static str {
    match padding {
        Size::Xs => "fw-wire-frame-padding-xs",
        Size::Sm => "fw-wire-frame-padding-sm",
        Size::Md => "fw-wire-frame-padding-md",
        Size::Lg => "fw-wire-frame-padding-lg",
        Size::Xl => "fw-wire-frame-padding-xl",
    }
}

/// 配置コンテナを組み立てる。
///
/// - `children`: 子ノード群。空の場合でも空要素（`<div class="...">
///   </div>`）を出力する（配置枠として空の領域を示す用途があるため）。
/// - `padding`: [`Size`] 5 段。ルート class `fw-wire-frame-padding-<段階>`
///   （[`padding_class`]）として付与し、padding 値を直接決める。
/// - `bordered`: `true` のとき部品固有の修飾 class
///   [`BORDERED_CLASS`]（`fw-wire-frame-bordered`）を付与し境界線を
///   表示する。
///
/// ルート要素は `div`。`role`/`aria-*`/`tabindex`/`style`/`data-*`/対話要素
/// は一切出力しない（`docs/design/wireframe-ui-architecture.md` §5/§7）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::{render, text};
/// use fandhe_frontend_wireframe_ui::{frame, Size};
///
/// let node = frame(vec![text("子要素")], Size::Md, true);
/// let html = render(&node);
/// assert!(html.contains(r#"class="fw-wire-frame fw-wire-frame-padding-md fw-wire-frame-bordered""#));
/// assert!(html.contains("子要素"));
///
/// // bordered=false では修飾 class が付かない。
/// let without_border = frame(vec![], Size::Md, false);
/// assert!(!render(&without_border).contains("fw-wire-frame-bordered"));
///
/// // 子が空でも空要素を出力する。
/// assert!(render(&without_border).contains(r#"class="fw-wire-frame fw-wire-frame-padding-md"></div>"#));
///
/// // XSS 回帰: 子テキストは core の既定エスケープを経由する（テキスト
/// // 引数を持たない部品のため子テキスト経由で固定する）。
/// let escaped = frame(vec![text("<script>alert(1)</script>")], Size::Md, false);
/// let escaped_html = render(&escaped);
/// assert!(!escaped_html.contains("<script>alert(1)</script>"));
/// assert!(escaped_html.contains("&lt;script&gt;"));
/// ```
#[must_use]
pub fn frame(children: Vec<Node>, padding: Size, bordered: bool) -> Node {
    let class = class_list(
        "fw-wire-frame",
        &[
            Some(padding_class(padding)),
            bordered.then_some(BORDERED_CLASS),
        ],
    );

    el_owned("div", vec![("class".to_string(), class)], children)
}
