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

/// フレーム CSS（2 セレクタ）。[`crate::css::PARTS`] へ登録される。
///
/// padding は `Size` 段階の `--fw-wire-control-size` の 1/2（xs 0.75rem〜
/// xl 1.5rem）を `calc()` で参照する（`size::css` が定義済みのスコープ付き
/// カスタムプロパティを再利用し、値を書き写さない）。非 bordered でも
/// `transparent` の境界線幅を確保し、`bordered` の切り替えで子の配置が
/// ずれない（決定的なレイアウト）。`font-size` は設定しない
/// （コンテナの padding 段階が子のフォントサイズへ波及しないようにする。
/// 子部品は自分の `fw-wire-size-*` class を持つため custom property の
/// 継承は自己上書きされる）。`background` も設定しない（入れ子時に紙面が
/// 透けるようにする）。
pub const FRAME_CSS: &str = "\
.fw-wire-frame {
  display: block;
  box-sizing: border-box;
  min-width: 0;
  padding: calc(var(--fw-wire-control-size, 2rem) / 2);
  border: var(--fw-wire-line-width) solid transparent;
  border-radius: var(--fw-wire-radius);
}
.fw-wire-frame.fw-wire-frame-bordered {
  border-color: var(--fw-wire-line);
}
";

/// 配置コンテナを組み立てる。
///
/// - `children`: 子ノード群。空の場合でも空要素（`<div class="...">
///   </div>`）を出力する（配置枠として空の領域を示す用途があるため）。
/// - `padding`: [`Size`] 5 段。ルート class `fw-wire-size-<段階>` として
///   付与し、padding は `--fw-wire-control-size` を参照する。
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
/// assert!(html.contains(r#"class="fw-wire-frame fw-wire-size-md fw-wire-frame-bordered""#));
/// assert!(html.contains("子要素"));
///
/// // bordered=false では修飾 class が付かない。
/// let without_border = frame(vec![], Size::Md, false);
/// assert!(!render(&without_border).contains("fw-wire-frame-bordered"));
///
/// // 子が空でも空要素を出力する。
/// assert!(render(&without_border).contains(r#"class="fw-wire-frame fw-wire-size-md"></div>"#));
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
        &[Some(padding.class()), bordered.then_some(BORDERED_CLASS)],
    );

    el_owned("div", vec![("class".to_string(), class)], children)
}
