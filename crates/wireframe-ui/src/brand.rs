//! ブランドロゴプレースホルダー部品（`Brand`、イシュー #2653、Phase 7
//! 「Data display」の 8 番目の部品）。
//!
//! 正方形の枠の中に汎用ブランドマークの線画（既定）または差し替え可能な
//! スロットを置く、非インタラクティブなローファイ・プレースホルダー。
//!
//! # 呼び出し文脈
//!
//! `fandhe-frontend-docs-site` の `wireframes::brand` showcase
//! （`/wireframes/brand/`）から呼ばれる。子は構築済み `Node`（既定
//! エスケープ済み）で受けるため、REQ-1 の既定エスケープは core の
//! `render()` 契約に委譲される。
//!
//! # API 設計の由来（原案からの差分）
//!
//! blocks.pm 原案の Figma プロパティ構成は `Size` と `Brand(swap)`
//! （instance swap）の 2 軸のみである。`docs/design/wireframe-ui-architecture.md`
//! §6・§11.4 の変換規約に従い、`Size` は本クレート共通の [`Size`] 5 段へ
//! 畳み込み、`Brand(swap)` は `Option<Node>` のスロット引数として受ける
//! （[`crate::avatar`] と同型の判断）。実在ブランドのロゴ・商標を模した
//! SVG はこのクレートへ一切持ち込まない（`docs/design/wireframe-ui-architecture.md`
//! §2 のライセンス方針、Issue 本文の明示要求）。この差分は
//! `site/wireframes/brand.md` の「原案差分メモ」節にも記録する。
//!
//! # `None` のときの既定コンテンツフォールバック（§11.4 からの意図的な逸脱）
//!
//! `crate::icon`（§11.4）の `Option<Node>` アイコンスロット規約は
//! 「`None` ならスロット要素を出力しない」を原則とするが、本部品は
//! [`crate::avatar`] と同じ理由（中身のない枠はワイヤーフレームとして
//! 配置意図が伝わらない）で、`content` が `None` のとき既定の汎用抽象
//! ブランドマーク [`crate::icon::brand`]（実在の商標を模さない、六角形 +
//! 中心円のバッジ状の線画）を差し込む。`Some(node)` を渡した場合はその
//! ノードをそのまま子要素にする（例: 別ブランド枠を表すための
//! `text("A")` のようなイニシャル表示）。この差分は
//! `site/wireframes/brand.md` の「原案差分メモ」節にも記録する。

use fandhe_frontend_core::{el_owned, Node};

use crate::class::class_list;
use crate::icon;
use crate::size::Size;

/// ブランド CSS（ルート・内部グリフサイズ調整の 2 セレクタ）。
/// [`crate::css::PARTS`] へ登録される。
///
/// 黒塗り二値ではなくグレースケールのトークン（`--fw-wire-fill-subtle`/
/// `--fw-wire-ink-muted`）で塗る（`docs/design/wireframe-ui-architecture.md`
/// §3「読みやすさ優先のグレースケール」、[`crate::avatar::AVATAR_CSS`] と
/// 同じ配色軸）。寸法は [`crate::size::css`] が定義する
/// `--fw-wire-control-size` を `var()` で参照するのみで、`size::SCALE` の
/// 値そのものはここへ書き写さない（同文書 §10.4）。
pub const BRAND_CSS: &str = "\
.fw-wire-brand {
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
.fw-wire-brand .fw-wire-icon-glyph {
  font-size: calc(var(--fw-wire-control-size, 2rem) * 0.6);
}
";

/// ブランドロゴプレースホルダーを組み立てる。
///
/// - `content`: 省略可能なコンテンツスロット（Figma の `Brand(swap)` に
///   相当）。`None` のときは既定の汎用抽象ブランドマーク
///   [`crate::icon::brand`] を差し込む（§11.4 からの意図的な逸脱、
///   モジュール doc参照）。`Some(node)` を渡した場合はそのノードを
///   そのまま子要素にする。
/// - `size`: [`Size`] 5 段。ルート class `fw-wire-size-<段階>` として
///   付与し、寸法は [`crate::size::css`] が定義する
///   `--fw-wire-control-size` を `var()` で参照する。
///
/// ルート要素は `div`。`role`/`aria-*`/`tabindex`/`style`/`href`/`src`/
/// `on*`/`<img>`/`<a>`/`<button>`/表示状態の `data-*` は一切出力しない
/// （`docs/design/wireframe-ui-architecture.md` §5/§7）。`content` に
/// [`crate::icon`] の関数を渡した場合、その戻り値が持つ `data-icon`
/// 属性（例: `data-icon="brand"`）はアイコン基盤側の識別子であり、部品側
/// の出力ではない。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::{render, text};
/// use fandhe_frontend_wireframe_ui::{brand, icon, Size};
///
/// // 既定（content: None）は汎用抽象ブランドマークにフォールバックする。
/// let node = brand(None, Size::Md);
/// let html = render(&node);
/// assert!(html.contains(r#"class="fw-wire-brand fw-wire-size-md""#));
/// assert!(html.contains(r#"data-icon="brand""#));
///
/// // スロット差し替え（イニシャルテキスト）。
/// let with_initial = brand(Some(text("A")), Size::Md);
/// let with_initial_html = render(&with_initial);
/// assert!(!with_initial_html.contains("<svg"));
/// assert!(with_initial_html.contains(">A<"));
///
/// // スロット差し替え（別アイコン）。
/// let with_icon = brand(Some(icon::star(Size::Md)), Size::Md);
/// let with_icon_html = render(&with_icon);
/// assert!(with_icon_html.contains(r#"data-icon="star""#));
/// assert!(!with_icon_html.contains(r#"data-icon="brand""#));
///
/// // XSS 回帰: スロットのテキストは既定エスケープを経由する。
/// let escaped = brand(Some(text("<script>alert(1)</script>")), Size::Md);
/// let escaped_html = render(&escaped);
/// assert!(!escaped_html.contains("<script>alert(1)</script>"));
/// assert!(escaped_html.contains("&lt;script&gt;"));
/// ```
#[must_use]
pub fn brand(content: Option<Node>, size: Size) -> Node {
    let class = class_list("fw-wire-brand", &[Some(size.class())]);

    let child = content.unwrap_or_else(|| icon::brand(size));

    el_owned("div", vec![("class".to_string(), class)], vec![child])
}
