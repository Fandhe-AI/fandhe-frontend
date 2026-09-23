//! タグ部品（`Tag`、イシュー #2619、Phase 2「テキスト・注釈」）。
//!
//! 分類・キーワードラベルを示す、ピル形状の非インタラクティブな
//! ローファイ・プレースホルダー。削除操作の実演は行わない（見た目だけの
//! 「×」アイコンパートを任意で持つ）。
//!
//! # 呼び出し文脈
//!
//! `fandhe-frontend-docs-site` の `wireframes::tag` showcase
//! （`/wireframes/tag/`）から呼ばれる。`fandhe_frontend_core::text` の
//! みでラベル文言を流し込むため、既定エスケープ（REQ-1）は本モジュールが
//! 独自に保証する必要はなく core 側の契約に委譲される。削除アイコンは
//! 呼び出し側が [`crate::icon::x`] の戻り値をそのまま渡す（本モジュールが
//! 新たな SVG 定義を持つ必要がない）。
//!
//! # API 設計の由来
//!
//! blocks.pm の Tag 部品の Figma プロパティ構成をそのまま転写したもの
//! ではなく、`docs/design/wireframe-ui-architecture.md` §6 の汎用変換
//! 規約（`Size` 軸 + 強調 bool + テキスト + 表示状態 bool）から独立設計
//! した（イシュー #2619 実装計画 §3、`site/wireframes/tag.md` の
//! 「原案差分メモ」節も参照）。
//!
//! `remove`（削除アイコンパート）は同文書 §11.4 の `Node` スロット規約
//! （[`crate::link`] の `trailing` と同型）に従い `Option<Node>` で受ける。
//! 呼び出し側が `Some(icon::x(size))` を渡したときのみ削除パートを出力
//! し、`None` のときはパート要素自体を出力しない。§11.4 は `tag` をこの
//! 規約の標準適用先として明示的に列挙しており、本モジュールは他の部品
//! （[`crate::link`] の `trailing` 等）と同じ形に統一する。

use fandhe_frontend_core::{el_owned, span, text, Node};

use crate::class::class_list;
use crate::props::Primary;
use crate::size::Size;

/// パート class（部品ルートなしで単独使用しない、[`tag`] 専用）。
const LABEL_CLASS: &str = "fw-wire-tag-label";
const REMOVE_CLASS: &str = "fw-wire-tag-remove";

/// タグ CSS（5 セレクタ）。[`crate::css::PARTS`] へ登録される。
///
/// ピル形状は `border-radius: 999px`（[`crate::tokens`] の
/// `--fw-wire-radius` は角丸カード等向けの小さい値のため使わない）。
/// `Primary(true)`（強調）時は背景・文字色を反転する。反転時の削除
/// アイコンは `--fw-wire-ink-muted` ではなく `--fw-wire-fill`（明るい
/// トークン）を使う。暗地上で `ink-muted` を使うとコントラスト不足に
/// なるため（[`crate::annotation::ANNOTATION_CSS`] と同じ判断軸、設計
/// 文書 §3「黒塗り二値を強制せず読みやすさ優先」）。
pub const TAG_CSS: &str = "\
.fw-wire-tag {
  display: inline-flex;
  align-items: center;
  gap: 0.25em;
  max-width: 100%;
  box-sizing: border-box;
  padding: 0.125em 0.625em;
  border: var(--fw-wire-line-width) solid var(--fw-wire-line);
  border-radius: 999px;
  background: var(--fw-wire-fill-subtle);
  color: var(--fw-wire-ink);
  font-family: var(--fw-wire-font-family);
  font-size: var(--fw-wire-font-size, 1rem);
  line-height: 1.4;
}
.fw-wire-tag-label {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.fw-wire-tag-remove {
  display: inline-flex;
  align-items: center;
  color: var(--fw-wire-ink-muted);
}
.fw-wire-tag.fw-wire-primary {
  background: var(--fw-wire-ink);
  border-color: var(--fw-wire-ink);
  color: var(--fw-wire-paper);
}
.fw-wire-tag.fw-wire-primary .fw-wire-tag-remove {
  color: var(--fw-wire-fill);
}
";

/// タグを組み立てる。
///
/// - `label`: 必須のタグ文言。
/// - `size`: [`Size`] 5 段。ルート class `fw-wire-size-<段階>` として付与し、
///   フォントサイズは [`crate::size::css`] が定義する `--fw-wire-font-size`
///   を `var()` で参照する（本モジュールは値を書き写さない）。
/// - `primary`: `true` のとき強調（反転色）バリアントにする。
/// - `remove`: 省略可能な削除「×」アイコンスロット（`docs/design/wireframe-ui-architecture.md`
///   §11.4）。削除パートを表示したい場合は `Some(icon::x(size))` を渡す。
///   `None` のときはパート要素自体を出力しない（空要素を残さない）。
///
/// 「×」は `<button>`/`<a>` にしない（見た目だけの表示専用パートであり、
/// 実際の削除操作が必要な場合は Themes/Primitives の Tags Input を
/// 案内する、`docs/design/wireframe-ui-architecture.md` §7「対話的な
/// WAI-ARIA セマンティクス・対話要素を出力しない」）。テキストは
/// [`fandhe_frontend_core::text`] のみで流し込み（REQ-1 既定エスケープ）、
/// `role`/`aria-*`/`tabindex`/`style`/`data-*` は本モジュールが一切
/// 付与しない（`remove` に渡した `Node`（例: [`crate::icon::x`]）が持つ
/// `data-icon="x"` はアイコン基盤側の識別子であり、本部品が付与する表示
/// 状態 `data-*` ではない）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_wireframe_ui::{icon, tag, Primary, Size};
///
/// let node = tag("draft", Size::Md, Primary(false), None);
/// let html = render(&node);
/// assert!(html.contains(r#"class="fw-wire-tag fw-wire-size-md""#));
/// assert!(html.contains(r#"class="fw-wire-tag-label""#));
/// assert!(html.contains("draft"));
/// assert!(!html.contains("fw-wire-tag-remove"));
///
/// // Some を渡したときだけ削除アイコンパートが出力される。
/// let removable = tag("removable", Size::Md, Primary(false), Some(icon::x(Size::Md)));
/// let removable_html = render(&removable);
/// assert!(removable_html.contains("fw-wire-tag-remove"));
/// assert!(removable_html.contains(r#"data-icon="x""#));
///
/// // XSS 回帰: label は既定エスケープを経由する。
/// let escaped = tag("<script>alert(1)</script>", Size::Md, Primary(false), None);
/// let escaped_html = render(&escaped);
/// assert!(!escaped_html.contains("<script>alert(1)</script>"));
/// assert!(escaped_html.contains("&lt;script&gt;"));
/// ```
#[must_use]
pub fn tag(label: &str, size: Size, primary: Primary, remove: Option<Node>) -> Node {
    let class = class_list("fw-wire-tag", &[Some(size.class()), primary.class()]);

    let mut children: Vec<Node> = vec![span(vec![("class", LABEL_CLASS)], vec![text(label)])];
    if let Some(remove) = remove {
        children.push(span(vec![("class", REMOVE_CLASS)], vec![remove]));
    }

    el_owned("span", vec![("class".to_string(), class)], children)
}
