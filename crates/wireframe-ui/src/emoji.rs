//! 絵文字プレースホルダー部品（`Emoji`、イシュー #2654、Phase 7
//! 「Data display」の 2 番目の部品）。
//!
//! 画面設計図中に絵文字 1 個を置く、非インタラクティブなローファイ
//! プレースホルダー。[`crate::avatar`]（人物の線画または差し替え可能な
//! スロット）とは異なり、本部品は「どの文字を置くか」という 1 引数の
//! 選択だけを表す単純な部品である。
//!
//! # 呼び出し文脈
//!
//! `fandhe-frontend-docs-site` の `wireframes::emoji` showcase
//! （`/wireframes/emoji/`）から呼ばれる。`glyph` は
//! [`fandhe_frontend_core::text`] のみでテキストノード化するため、
//! 既定エスケープ（REQ-1）は core 側の契約に委譲される。
//!
//! # API 設計の由来（原案からの差分、§6 からの意図的な逸脱）
//!
//! 原案（blocks.pm）のプロパティ軸はサイズと絵文字の差し替えの 2 つ
//! だけである。絵文字は Unicode の 1 文字（または ZWJ シーケンス等の
//! 複数コードポイント列）であり、[`crate::icon`] のような SVG
//! インスタンスではないため、`Option<Node>` アイコンスロット規約
//! （`docs/design/wireframe-ui-architecture.md` §11.4）には従わず、
//! `glyph: &str` の 1 引数へ畳み込む（§6 の「boolean 爆発やスロットを
//! 最小引数に畳む」趣旨に沿う判断）。任意のノードを差し込みたい場合は
//! 既存の [`crate::icon`] や他部品を直接使うことを想定し、本部品では
//! スロットを持たない。この差分は `site/wireframes/emoji.md` の
//! 「原案差分メモ」節にも記録する。
//!
//! モノクロ化（`docs/design/wireframe-ui-architecture.md` §3）は CSS の
//! `filter: grayscale(1)` で行い、色値のリテラルは書かない。`glyph` が
//! 空文字列のときはテキストノードを出力せず、CSS の `:empty` 疑似クラス
//! で破線の円プレースホルダーとして表示する（「絵文字を置く場所」を
//! 示すため）。複数コードポイントの入力（ZWJ シーケンス・国旗等）は
//! 検証・切り詰めをせずそのまま 1 テキストノードとして出力する
//! （Unicode 依存を core へ持ち込まない責務境界、`docs/design/wireframe-ui-architecture.md`
//! §5）。

use fandhe_frontend_core::{el_owned, text as text_node, Node};

use crate::class::class_list;
use crate::size::Size;

/// 絵文字 CSS（ルート・空文字時プレースホルダーの 2 セレクタ）。
/// [`crate::css::PARTS`] へ登録される。
///
/// 寸法は [`crate::size::css`] が定義する `--fw-wire-font-size` を
/// `var()` で参照するのみで、`size::SCALE` の値は書き写さない
/// （`docs/design/wireframe-ui-architecture.md` §10）。カラー絵文字を
/// モノクロ方針（§3）へ合わせるため `filter: grayscale(1)` を宣言する。
pub const EMOJI_CSS: &str = "\
.fw-wire-emoji {
  display: inline-block;
  line-height: 1;
  white-space: nowrap;
  vertical-align: -0.125em;
  font-family: var(--fw-wire-font-family);
  font-size: var(--fw-wire-font-size, 1rem);
  color: var(--fw-wire-ink);
  filter: grayscale(1);
}
.fw-wire-emoji:empty {
  width: 1em;
  height: 1em;
  box-sizing: border-box;
  border: var(--fw-wire-line-width) dashed var(--fw-wire-line);
  border-radius: 50%;
}
";

/// 絵文字プレースホルダーを組み立てる。
///
/// - `glyph`: 表示する絵文字（複数コードポイントのシーケンスを含む）。
///   空文字列（`""`）のときはテキストノードを出力せず、CSS の `:empty`
///   規則が破線の円プレースホルダーを表示する。検証・切り詰めは行わず、
///   そのまま [`fandhe_frontend_core::text`] へ渡す（REQ-1 既定
///   エスケープ）。
/// - `size`: [`Size`] 5 段。ルート class `fw-wire-size-<段階>` として
///   付与し、フォントサイズは [`crate::size::css`] が定義する
///   `--fw-wire-font-size` を `var()` で参照する。
///
/// ルート要素は `<span>`（インラインの 1 グリフ要素、[`crate::text::text`]
/// と同型の判断）。`role`/`aria-*`/`tabindex`/`style`/`href`/`src`/`on*`/
/// `<img>`/`<a>`/`<button>`/表示状態の `data-*` は一切出力しない
/// （`docs/design/wireframe-ui-architecture.md` §5/§7、Emoji は表示状態軸
/// を持たない部品のため `data-*` も不要）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_wireframe_ui::{emoji, Size};
///
/// let node = emoji("🙂", Size::Md);
/// let html = render(&node);
/// assert!(html.contains(r#"class="fw-wire-emoji fw-wire-size-md""#));
/// assert!(html.starts_with("<span"));
/// assert!(html.contains("🙂"));
///
/// // 空文字列は破線の円プレースホルダー扱い（CSS 側で表現、マークアップは空）。
/// let empty = emoji("", Size::Md);
/// assert!(render(&empty).ends_with("</span>"));
///
/// // 複数コードポイントのシーケンス（ZWJ 等）もそのまま 1 テキストノードにする。
/// let sequence = emoji("👩\u{200d}💻", Size::Md);
/// assert!(render(&sequence).contains("👩\u{200d}💻"));
///
/// // XSS 回帰: glyph は既定エスケープを経由する。
/// let escaped = emoji("<script>alert(1)</script>", Size::Md);
/// let escaped_html = render(&escaped);
/// assert!(!escaped_html.contains("<script>alert(1)</script>"));
/// assert!(escaped_html.contains("&lt;script&gt;"));
/// ```
#[must_use]
pub fn emoji(glyph: &str, size: Size) -> Node {
    let class = class_list("fw-wire-emoji", &[Some(size.class())]);

    let children: Vec<Node> = if glyph.is_empty() {
        Vec::new()
    } else {
        vec![text_node(glyph)]
    };

    el_owned("span", vec![("class".to_string(), class)], children)
}
