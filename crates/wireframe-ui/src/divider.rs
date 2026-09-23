//! 区切り線部品（`Divider`、イシュー #2612、Phase 1「レイアウト骨格」）。
//!
//! 水平/垂直の区切り線に、省略可能なラベルを中央へ置くための
//! 非インタラクティブなローファイ・プレースホルダー。[`crate::props::Orientation`]
//! の最初の実消費者でもある。
//!
//! # 呼び出し文脈
//!
//! `fandhe-frontend-docs-site` の `wireframes::divider` showcase
//! （`/wireframes/divider/`）から呼ばれる。`fandhe_frontend_core::text`
//! のみでラベルを流し込むため、既定エスケープ（REQ-1）は本モジュールが
//! 独自に保証する必要はなく core 側の契約に委譲される。
//!
//! # API 設計の由来
//!
//! blocks.pm の Divider 部品の Figma プロパティ構成（`Size`/`Label`/
//! `Vertical` 相当のトグル）をそのまま転写したものではなく、
//! `docs/design/wireframe-ui-architecture.md` §6 の汎用変換規約（`Size` 軸 +
//! 方向 + 省略可能テキスト）から独立設計した（イシュー #2612 実装計画 §2、
//! `site/wireframes/divider.md` の「原案差分メモ」節も参照）。
//!
//! `hr` 要素は使わない（垂直・ラベル付きを表現できず、タグが variant で
//! 変わると非対話制約の契約テストが複雑化するため）。`role="separator"` も
//! 付与しない（本クレートは `role`/`aria-*` を一切出力しない方針、設計文書
//! §5/§7）。実際に操作可能・アクセシブルな区切り線が必要な場合は
//! Themes の Separator（`fandhe-frontend-pre-styled-ui::separator`）を使う。

use fandhe_frontend_core::{el_owned, span, text, Node};

use crate::class::class_list;
use crate::props::Orientation;
use crate::size::Size;

/// ラベルパート class（部品ルートなしで単独使用しない、[`divider`] 専用）。
const LABEL_CLASS: &str = "fw-wire-divider-label";

/// 区切り線 CSS。[`crate::css::PARTS`] へ登録される。
///
/// 線はルートの `::before`/`::after` 擬似要素で描く（`flex: 1`）。
/// ラベルなしのときは 2 つの擬似要素が隣接して 1 本の連続線になるため、
/// 「ラベル有無」を区別する追加 class は不要である。垂直方向はデモ環境
/// （docs サイトの `wireframes-demo` 枠内、`style=` を付与できない非
/// インタラクティブ部品のため）で 0 高さに潰れないよう `min-height` を
/// `--fw-wire-control-size`（[`crate::size::css`] が `Size` 段階ごとに
/// 定義）で自立させる。ただしこのコンテナ `min-height` はラベルが無い
/// ときの下限に過ぎず、ラベル文字列の実高さ（フォントサイズ + 縦
/// `padding`）が `--fw-wire-control-size` を超える場合はコンテナが
/// 内容に合わせて伸びるだけで `::before`/`::after`（`flex-basis: 0`）
/// に配分される余白が生まれず線が消える。このため `::before`/`::after`
/// 自体にも `min-height`（固定 `0.75em`、`Size` に連動しない下限）を
/// 与え、ラベル有無・サイズ段階によらず線分が可視のまま残ることを保証
/// する（イシュー #2612 PR レビュー指摘の是正）。
pub const DIVIDER_CSS: &str = "\
.fw-wire-divider {
  display: flex;
  align-items: center;
  color: var(--fw-wire-ink-muted);
  font-family: var(--fw-wire-font-family);
  font-size: var(--fw-wire-font-size, 1rem);
}
.fw-wire-divider::before,
.fw-wire-divider::after {
  content: \"\";
  flex: 1;
}
.fw-wire-divider.fw-wire-horizontal {
  width: 100%;
  margin: 0.75em 0;
}
.fw-wire-divider.fw-wire-horizontal::before,
.fw-wire-divider.fw-wire-horizontal::after {
  border-top: var(--fw-wire-line-width) solid var(--fw-wire-line);
}
.fw-wire-divider.fw-wire-vertical {
  display: inline-flex;
  flex-direction: column;
  min-height: var(--fw-wire-control-size, 2rem);
  align-self: stretch;
  margin: 0 0.75em;
}
.fw-wire-divider.fw-wire-vertical::before,
.fw-wire-divider.fw-wire-vertical::after {
  border-left: var(--fw-wire-line-width) solid var(--fw-wire-line);
  min-height: 0.75em;
}
.fw-wire-divider-label {
  padding: 0 0.75em;
  white-space: nowrap;
}
.fw-wire-divider.fw-wire-vertical .fw-wire-divider-label {
  padding: 0.5em 0;
}
";

/// 区切り線を組み立てる。
///
/// - `label`: 省略可能な、線の中央に置くテキスト。`None` のときはラベル
///   パート要素自体を出力しない（空要素を残さない）。
/// - `size`: [`Size`] 5 段。ラベルのフォントサイズと垂直方向の最小長さ
///   （`--fw-wire-control-size`）へ反映する。線の太さは
///   `--fw-wire-line-width` 固定で `size` に連動しない。
/// - `orientation`: [`Orientation`] 水平/垂直。ルート class
///   `fw-wire-horizontal`/`fw-wire-vertical` として付与する。
///
/// テキストは [`fandhe_frontend_core::text`] のみで流し込み（REQ-1 既定
/// エスケープ）、`role`/`aria-*`/`tabindex`/`style`/`data-*` は一切付与
/// しない（`docs/design/wireframe-ui-architecture.md` §5/§7、Divider は
/// 表示状態軸を持たない部品のため `data-*` も不要）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_wireframe_ui::{divider, Orientation, Size};
///
/// let node = divider(None, Size::Md, Orientation::Horizontal);
/// let html = render(&node);
/// assert!(html.contains(r#"class="fw-wire-divider fw-wire-size-md fw-wire-horizontal""#));
/// assert!(!html.contains("fw-wire-divider-label"));
///
/// let with_label = divider(Some("または"), Size::Md, Orientation::Vertical);
/// let html = render(&with_label);
/// assert!(html.contains("fw-wire-vertical"));
/// assert!(html.contains(r#"class="fw-wire-divider-label""#));
/// assert!(html.contains("または"));
///
/// // XSS 回帰: ラベルは既定エスケープを経由する。
/// let escaped = divider(Some("<script>alert(1)</script>"), Size::Md, Orientation::Horizontal);
/// let escaped_html = render(&escaped);
/// assert!(!escaped_html.contains("<script>alert(1)</script>"));
/// assert!(escaped_html.contains("&lt;script&gt;"));
/// ```
#[must_use]
pub fn divider(label: Option<&str>, size: Size, orientation: Orientation) -> Node {
    let class = class_list(
        "fw-wire-divider",
        &[Some(size.class()), Some(orientation.class())],
    );

    let mut children: Vec<Node> = Vec::new();
    if let Some(label) = label {
        children.push(span(vec![("class", LABEL_CLASS)], vec![text(label)]));
    }

    el_owned("div", vec![("class".to_string(), class)], children)
}
