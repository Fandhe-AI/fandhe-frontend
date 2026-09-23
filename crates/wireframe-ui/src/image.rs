//! 画像プレースホルダー部品（`Image`、イシュー #2660、Phase 8「Media・
//! Data」の最初の部品）。
//!
//! 対角のバツ印が入った正方形（または円形）の枠で、画像が入る場所を
//! ワイヤーフレーム上に示す非インタラクティブなローファイ・
//! プレースホルダー。
//!
//! # 呼び出し文脈
//!
//! `fandhe-frontend-docs-site` の `wireframes::image` showcase
//! （`/wireframes/image/`）から呼ばれる。子は構築済み `Node`（既定
//! エスケープ済み）で受けるため、REQ-1 の既定エスケープは core の
//! `render()` 契約に委譲される。
//!
//! # API 設計の由来（原案からの差分）
//!
//! blocks.pm の Image 部品が持つ Figma プロパティ（円形・強調・
//! プレースホルダー有無）はライセンス保留（`docs/design/wireframe-ui-architecture.md`
//! §2）のためそのまま転写せず、同文書 §4（`Size` 5 段）・§6（boolean の
//! 畳み込み、instance swap を `Node` スロットで受ける）の汎用変換規約から
//! 独立設計した（`site/wireframes/image.md` の「原案差分メモ」節も参照）。
//! サイズは他部品と同じく本クレート共通の [`Size`] 5 段へ畳み込み、
//! 円形フラグは `crate::avatar`/`crate::frame` の `circle`/`bordered` と
//! 同型の「部品固有の修飾 class」（§10.1）として表現し、`crate::props` に
//! 新しい型は追加しない。強調（黒塗り）は既存の共通型 [`crate::props::Primary`]
//! を再利用する（`crate::counter` と同じ判断、`props.rs` へ新型を追加しない）。
//!
//! # `content` スロット（§11.4 に準拠、`avatar` のような逸脱はない）
//!
//! `content` が `None` のときは既定のバツ印プレースホルダーを描き、子
//! 要素は一切出力しない（`docs/design/wireframe-ui-architecture.md`
//! §11.4 の「`None` ならスロット要素を出力しない」原則にそのまま合致する。
//! `crate::avatar` が `icon::user` へフォールバックする逸脱とは異なる）。
//! `Some(node)` のときはそのノードを子要素にし、バツ印の背景は出さない
//! （例: [`crate::icon::image`] のグリフ差し替えやキャプション用の
//! `text("...")`）。
//!
//! # バツ印の描画方式
//!
//! 疑似要素を `rotate()` で回す方式ではなく、`background-image` の
//! `linear-gradient` 2 本（角から角へ）を重ねて描く。理由は角へ正確に
//! 届く・アスペクト比が変わっても崩れない・追加マークアップが要らない、
//! の 3 点。線色は CSS カスタムプロパティ `--fw-wire-image-x-color`
//! （既定は [`crate::tokens`] の `--fw-wire-line`）を介して参照し、
//! `Primary` 修飾時のみ `--fw-wire-paper` へ上書きする（色の重複記述を
//! 避けるための設計）。
//!
//! # 出力制約
//!
//! ルート要素は `div`。`<img>`・`src`・`href`・`style`・`role`・
//! `aria-*`（[`crate::icon`] 自身が持つ装飾用の `aria-hidden="true"` は
//! アイコン基盤側の出力のため除く）・`tabindex`・`on*`・`<a>`・`<button>`
//! は一切出力しない。画像 URL を受け取る API も設けない（外部リソース
//! 読み込みの経路を作らないため）。

use fandhe_frontend_core::{el_owned, Node};

use crate::class::class_list;
use crate::props::Primary;
use crate::size::Size;

/// 部品固有の修飾 class（`content: None` のときのみ付与、バツ印を描く）。
const PLACEHOLDER_CLASS: &str = "fw-wire-image-placeholder";

/// 部品固有の修飾 class（`circle=true` のときのみ付与）。
const CIRCLE_CLASS: &str = "fw-wire-image-circle";

/// 画像プレースホルダー CSS（ルート・circle 修飾・placeholder バツ印・
/// `Primary` 反転の 4 セレクタ）。[`crate::css::PARTS`] へ登録される。
///
/// 寸法は [`crate::size::css`] が定義する `--fw-wire-control-size` を
/// `var()` で参照し、3 倍した正方形にする（`size::SCALE` の値そのものは
/// ここへ書き写さない、`docs/design/wireframe-ui-architecture.md` §10）。
/// 黒塗り二値ではなくグレースケールのトークンを既定にする判断は
/// [`crate::counter::COUNTER_CSS`] と同じ軸（同文書 §3）。
pub const IMAGE_CSS: &str = "\
.fw-wire-image {
  --fw-wire-image-x-color: var(--fw-wire-line);
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  box-sizing: border-box;
  width: calc(var(--fw-wire-control-size, 2rem) * 3);
  height: calc(var(--fw-wire-control-size, 2rem) * 3);
  overflow: hidden;
  border: var(--fw-wire-line-width) solid var(--fw-wire-line);
  border-radius: var(--fw-wire-radius);
  background: var(--fw-wire-fill-subtle);
}
.fw-wire-image.fw-wire-image-circle {
  border-radius: 50%;
}
.fw-wire-image.fw-wire-image-placeholder {
  background-image:
    linear-gradient(to top right, transparent calc(50% - var(--fw-wire-line-width) / 2), var(--fw-wire-image-x-color) calc(50% - var(--fw-wire-line-width) / 2), var(--fw-wire-image-x-color) calc(50% + var(--fw-wire-line-width) / 2), transparent calc(50% + var(--fw-wire-line-width) / 2)),
    linear-gradient(to bottom right, transparent calc(50% - var(--fw-wire-line-width) / 2), var(--fw-wire-image-x-color) calc(50% - var(--fw-wire-line-width) / 2), var(--fw-wire-image-x-color) calc(50% + var(--fw-wire-line-width) / 2), transparent calc(50% + var(--fw-wire-line-width) / 2));
}
.fw-wire-image.fw-wire-primary {
  --fw-wire-image-x-color: var(--fw-wire-paper);
  background: var(--fw-wire-ink);
  border-color: var(--fw-wire-ink);
}
";

/// 画像プレースホルダーを組み立てる。
///
/// - `content`: 省略可能なコンテンツスロット（`docs/design/wireframe-ui-architecture.md`
///   §11.4）。`None` のときは既定のバツ印プレースホルダー（[`PLACEHOLDER_CLASS`]）
///   になり子要素を出力しない。`Some(node)` を渡した場合はそのノードを
///   そのまま子要素にし、バツ印は描かない（例: [`crate::icon::image`]・
///   `text("...")`）。
/// - `size`: [`Size`] 5 段。ルート class `fw-wire-size-<段階>` として
///   付与し、寸法は [`crate::size::css`] が定義する
///   `--fw-wire-control-size` を `var()` で参照する。
/// - `circle`: `true` のとき部品固有の修飾 class
///   [`CIRCLE_CLASS`]（`fw-wire-image-circle`）を付与し円形にする。
/// - `primary`: `true` のとき共通型 [`crate::props::Primary`] による
///   `fw-wire-primary` class を付与し、反転配色（`--fw-wire-ink` 背景 +
///   バツ印を `--fw-wire-paper` 色）にする。
///
/// ルート要素は `div`。`role`/`aria-*`/`tabindex`/`style`/`href`/`src`/
/// `on*`/`<img>`/`<a>`/`<button>`/表示状態の `data-*` は一切出力しない
/// （`docs/design/wireframe-ui-architecture.md` §5/§7）。画像 URL を
/// 受け取る API も設けない。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::{render, text};
/// use fandhe_frontend_wireframe_ui::{icon, image, Primary, Size};
///
/// // 既定（content: None）はバツ印プレースホルダーで、子要素を持たない。
/// let node = image(None, Size::Md, false, Primary(false));
/// let html = render(&node);
/// assert!(html.contains(r#"class="fw-wire-image fw-wire-size-md fw-wire-image-placeholder""#));
/// assert!(html.contains("></div>") || html.ends_with("></div>"));
///
/// // circle=true で修飾 class が付く。
/// let circular = image(None, Size::Md, true, Primary(false));
/// assert!(render(&circular).contains("fw-wire-image-circle"));
///
/// // primary=true で反転配色の class が付く。
/// let primary = image(None, Size::Md, false, Primary(true));
/// assert!(render(&primary).contains("fw-wire-primary"));
///
/// // スロット差し替え（画像アイコン）。placeholder class は付かない。
/// let with_icon = image(Some(icon::image(Size::Md)), Size::Md, false, Primary(false));
/// let with_icon_html = render(&with_icon);
/// assert!(with_icon_html.contains(r#"data-icon="image""#));
/// assert!(!with_icon_html.contains("fw-wire-image-placeholder"));
///
/// // XSS 回帰: スロットのテキストは既定エスケープを経由する。
/// let escaped = image(Some(text("<script>alert(1)</script>")), Size::Md, false, Primary(false));
/// let escaped_html = render(&escaped);
/// assert!(!escaped_html.contains("<script>alert(1)</script>"));
/// assert!(escaped_html.contains("&lt;script&gt;"));
/// ```
#[must_use]
pub fn image(content: Option<Node>, size: Size, circle: bool, primary: Primary) -> Node {
    let class = class_list(
        "fw-wire-image",
        &[
            Some(size.class()),
            content.is_none().then_some(PLACEHOLDER_CLASS),
            circle.then_some(CIRCLE_CLASS),
            primary.class(),
        ],
    );

    let children = content.into_iter().collect::<Vec<Node>>();

    el_owned("div", vec![("class".to_string(), class)], children)
}
