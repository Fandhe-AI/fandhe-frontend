//! アバタープレースホルダー部品（`Avatar`、イシュー #2651、Phase 7
//! 「Data display」の最初の部品）。
//!
//! 正方形または円形の枠の中に人物の線画（既定）または差し替え可能な
//! スロットを置く、非インタラクティブなローファイ・プレースホルダー。
//!
//! # 呼び出し文脈
//!
//! `fandhe-frontend-docs-site` の `wireframes::avatar` showcase
//! （`/wireframes/avatar/`）から呼ばれる。子は構築済み `Node`（既定
//! エスケープ済み）で受けるため、REQ-1 の既定エスケープは core の
//! `render()` 契約に委譲される。
//!
//! # API 設計の由来（原案からの差分）
//!
//! Figma 相当のプロパティ（サイズ・円形フラグ・人物の差し替え）は
//! `docs/design/wireframe-ui-architecture.md` §4・§6 の変換規約に従って
//! 変換した。サイズは原案が XXL まで想定していたが、本クレート共通の
//! [`Size`] 5 段（`Xs`〜`Xl`）へ畳み込む（他部品と同じ判断）。円形フラグは
//! `crate::frame` の `bordered: bool` と同型の「部品固有の修飾 class」
//! （§10.1）として表現し、`crate::props` に新しい型は追加しない。
//!
//! # `None` のときの既定コンテンツフォールバック（§11.4 からの意図的な逸脱）
//!
//! `crate::icon`（§11.4）の `Option<Node>` アイコンスロット規約は
//! 「`None` ならスロット要素を出力しない」を原則とするが、本部品は
//! `content` が `None` のとき既定の人物線画 [`crate::icon::user`] を
//! 差し込む。中身のないアバター枠はワイヤーフレームとして意味を持たない
//! ため（何もない矩形と区別が付かず、画面設計図上でアバターの配置意図が
//! 伝わらない）。`Some(node)` を渡した場合はそのノードをそのまま子要素に
//! する（例: [`crate::icon::image`] や `text("AB")` のイニシャル表示）。
//! この差分は `site/wireframes/avatar.md` の「原案差分メモ」節にも記録する。

use fandhe_frontend_core::{el_owned, Node};

use crate::class::class_list;
use crate::icon;
use crate::size::Size;

/// 部品固有の修飾 class（`circle=true` のときのみ付与）。
const CIRCLE_CLASS: &str = "fw-wire-avatar-circle";

/// アバター CSS（ルート・circle 修飾・内部グリフサイズ調整の 3 セレクタ）。
/// [`crate::css::PARTS`] へ登録される。
///
/// 黒塗り二値ではなくグレースケールのトークン（`--fw-wire-fill-subtle`/
/// `--fw-wire-ink-muted`）で塗る（`docs/design/wireframe-ui-architecture.md`
/// §3「読みやすさ優先のグレースケール」）。寸法は [`crate::size::css`] が
/// 定義する `--fw-wire-control-size` を `var()` で参照するのみで、
/// `size::SCALE` の値そのものはここへ書き写さない（同文書 §10）。
pub const AVATAR_CSS: &str = "\
.fw-wire-avatar {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  box-sizing: border-box;
  width: var(--fw-wire-control-size, 2rem);
  height: var(--fw-wire-control-size, 2rem);
  overflow: hidden;
  border: var(--fw-wire-line-width) solid var(--fw-wire-line);
  border-radius: var(--fw-wire-radius);
  background: var(--fw-wire-fill-subtle);
  color: var(--fw-wire-ink-muted);
  font-family: var(--fw-wire-font-family);
  font-size: var(--fw-wire-font-size, 1rem);
  line-height: 1;
}
.fw-wire-avatar.fw-wire-avatar-circle {
  border-radius: 50%;
}
.fw-wire-avatar .fw-wire-icon-glyph {
  font-size: calc(var(--fw-wire-control-size, 2rem) * 0.6);
}
";

/// アバターを組み立てる。
///
/// - `content`: 省略可能なコンテンツスロット。`None` のときは既定の人物
///   線画 [`crate::icon::user`] を差し込む（§11.4 からの意図的な逸脱、
///   モジュール doc参照）。`Some(node)` を渡した場合はそのノードを
///   そのまま子要素にする（例: [`crate::icon::image`]・イニシャル用の
///   `text("AB")`）。
/// - `size`: [`Size`] 5 段。ルート class `fw-wire-size-<段階>` として
///   付与し、寸法は [`crate::size::css`] が定義する
///   `--fw-wire-control-size` を `var()` で参照する。
/// - `circle`: `true` のとき部品固有の修飾 class
///   [`CIRCLE_CLASS`]（`fw-wire-avatar-circle`）を付与し円形にする。
///
/// ルート要素は `div`。`role`/`aria-*`/`tabindex`/`style`/`href`/`src`/
/// `on*`/`<img>`/`<a>`/`<button>`/表示状態の `data-*` は一切出力しない
/// （`docs/design/wireframe-ui-architecture.md` §5/§7）。`content` に
/// [`crate::icon`] の関数を渡した場合、その戻り値が持つ `data-icon`
/// 属性（例: `data-icon="user"`）はアイコン基盤側の識別子であり、部品側
/// の出力ではない。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::{render, text};
/// use fandhe_frontend_wireframe_ui::{avatar, icon, Size};
///
/// // 既定（content: None）は人物線画にフォールバックする。
/// let node = avatar(None, Size::Md, false);
/// let html = render(&node);
/// assert!(html.contains(r#"class="fw-wire-avatar fw-wire-size-md""#));
/// assert!(html.contains(r#"data-icon="user""#));
///
/// // circle=true で修飾 class が付く。
/// let circular = avatar(None, Size::Md, true);
/// assert!(render(&circular).contains("fw-wire-avatar-circle"));
///
/// // スロット差し替え（画像アイコン）。
/// let with_image = avatar(Some(icon::image(Size::Md)), Size::Md, false);
/// let with_image_html = render(&with_image);
/// assert!(with_image_html.contains(r#"data-icon="image""#));
/// assert!(!with_image_html.contains(r#"data-icon="user""#));
///
/// // スロット差し替え（イニシャルテキスト）。
/// let with_initials = avatar(Some(text("AB")), Size::Md, false);
/// let with_initials_html = render(&with_initials);
/// assert!(!with_initials_html.contains("<svg"));
/// assert!(with_initials_html.contains("AB"));
///
/// // XSS 回帰: スロットのテキストは既定エスケープを経由する。
/// let escaped = avatar(Some(text("<script>alert(1)</script>")), Size::Md, false);
/// let escaped_html = render(&escaped);
/// assert!(!escaped_html.contains("<script>alert(1)</script>"));
/// assert!(escaped_html.contains("&lt;script&gt;"));
/// ```
#[must_use]
pub fn avatar(content: Option<Node>, size: Size, circle: bool) -> Node {
    let class = class_list(
        "fw-wire-avatar",
        &[Some(size.class()), circle.then_some(CIRCLE_CLASS)],
    );

    let child = content.unwrap_or_else(|| icon::user(size));

    el_owned("div", vec![("class".to_string(), class)], vec![child])
}
