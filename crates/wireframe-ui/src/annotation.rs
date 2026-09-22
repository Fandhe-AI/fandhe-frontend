//! 注釈ボックス部品（`Annotation`、イシュー #2617、Phase 2「テキスト・注釈」）。
//!
//! 画面設計図の余白へ設計意図を書き込むための、太字タイトル + 任意の説明文
//! を持つ非インタラクティブなローファイ・プレースホルダー。
//!
//! # 呼び出し文脈
//!
//! `fandhe-frontend-docs-site` の `wireframes::annotation` showcase
//! （`/wireframes/annotation/`）から呼ばれる。`fandhe_frontend_core::text`
//! のみでテキストを流し込むため、既定エスケープ（REQ-1）は本モジュールが
//! 独自に保証する必要はなく core 側の契約に委譲される。
//!
//! # API 設計の由来
//!
//! blocks.pm の Annotation 部品の Figma プロパティ構成（`Type`/`Emphasis`
//! 相当の bool トグル等）をそのまま転写したものではなく、
//! `docs/design/wireframe-ui-architecture.md` §6 の汎用変換規約（`Size` 軸 +
//! 強調 bool + テキスト + 省略可能テキスト）から独立設計した（イシュー
//! #2617 実装計画 §2.1、`site/wireframes/annotation.md` の「原案差分メモ」
//! 節も参照）。

use fandhe_frontend_core::{div, el_owned, text, Node};

use crate::class::class_list;
use crate::props::Primary;
use crate::size::Size;

/// パート class（部品ルートなしで単独使用しない、[`annotation`] 専用）。
const TITLE_CLASS: &str = "fw-wire-annotation-title";
const DESCRIPTION_CLASS: &str = "fw-wire-annotation-description";

/// 注釈ボックス CSS（5 セレクタ）。[`crate::css::PARTS`] へ登録される。
///
/// `Primary(true)`（強調）時は背景・文字色を反転する。反転時の
/// description は `--fw-wire-ink-muted` ではなく `--fw-wire-fill`（明るい
/// トークン）を使う。暗地上で `ink-muted` を使うとコントラスト不足になる
/// ため（設計文書 §3「黒塗り二値を強制せず読みやすさ優先」の判断軸）。
pub const ANNOTATION_CSS: &str = "\
.fw-wire-annotation {
  display: inline-block;
  max-width: 100%;
  box-sizing: border-box;
  padding: 0.5em 0.75em;
  border: var(--fw-wire-line-width) solid var(--fw-wire-line);
  border-radius: var(--fw-wire-radius);
  background: var(--fw-wire-paper);
  color: var(--fw-wire-ink);
  font-family: var(--fw-wire-font-family);
  font-size: var(--fw-wire-font-size, 1rem);
  line-height: 1.4;
}
.fw-wire-annotation-title {
  font-weight: 600;
}
.fw-wire-annotation-description {
  margin-top: 0.25em;
  color: var(--fw-wire-ink-muted);
}
.fw-wire-annotation.fw-wire-primary {
  background: var(--fw-wire-ink);
  border-color: var(--fw-wire-ink);
  color: var(--fw-wire-paper);
}
.fw-wire-annotation.fw-wire-primary .fw-wire-annotation-description {
  color: var(--fw-wire-fill);
}
";

/// 注釈ボックスを組み立てる。
///
/// - `title`: 必須。太字で表示するタイトル文言。
/// - `description`: 省略可能な説明文。`None` のときはパート要素自体を
///   出力しない（空要素を残さない）。
/// - `size`: [`Size`] 5 段。ルート class `fw-wire-size-<段階>` として付与し、
///   フォントサイズは [`crate::size::css`] が定義する `--fw-wire-font-size`
///   を `var()` で参照する（本モジュールは値を書き写さない）。
/// - `primary`: `true` のとき強調（反転色）バリアントにする。`Bold` は
///   使わない（タイトルは常に太字であり、`Bold` 軸を追加すると
///   title/description のどちらに効くか曖昧になるため）。
///
/// テキストはいずれも [`fandhe_frontend_core::text`] のみで流し込み
/// （REQ-1 既定エスケープ）、`role`/`aria-*`/`tabindex`/`style`/`data-*` は
/// 一切付与しない（`docs/design/wireframe-ui-architecture.md` §5/§7、
/// Annotation は表示状態軸を持たない部品のため `data-*` も不要）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_wireframe_ui::{annotation, Primary, Size};
///
/// let node = annotation("設計メモ", Some("ここに補足を書く"), Size::Md, Primary(false));
/// let html = render(&node);
/// assert!(html.contains(r#"class="fw-wire-annotation fw-wire-size-md""#));
/// assert!(html.contains(r#"class="fw-wire-annotation-title""#));
/// assert!(html.contains("設計メモ"));
/// assert!(html.contains("ここに補足を書く"));
///
/// // description を省略するとパート要素自体が出力されない。
/// let without_description = annotation("タイトルのみ", None, Size::Md, Primary(false));
/// assert!(!render(&without_description).contains("fw-wire-annotation-description"));
///
/// // XSS 回帰: タイトル・説明文はいずれも既定エスケープを経由する。
/// let escaped = annotation("<script>alert(1)</script>", Some("\"><img src=x onerror=alert(1)>"), Size::Md, Primary(false));
/// let escaped_html = render(&escaped);
/// assert!(!escaped_html.contains("<script>alert(1)</script>"));
/// assert!(escaped_html.contains("&lt;script&gt;"));
/// ```
#[must_use]
pub fn annotation(title: &str, description: Option<&str>, size: Size, primary: Primary) -> Node {
    let class = class_list("fw-wire-annotation", &[Some(size.class()), primary.class()]);

    let mut children: Vec<Node> = vec![div(vec![("class", TITLE_CLASS)], vec![text(title)])];
    if let Some(description) = description {
        children.push(div(
            vec![("class", DESCRIPTION_CLASS)],
            vec![text(description)],
        ));
    }

    el_owned("div", vec![("class".to_string(), class)], children)
}
