//! カード型データ表示のプレースホルダー部品（`Card basic`、イシュー
//! #2658、Phase 7「Data display」の部品）。
//!
//! 先頭の視覚要素（アバター等）スロット・主テキストと補足テキストの
//! 2 段・末尾の補助アイコンスロットを 1 枚の枠線カードにまとめる、
//! 非インタラクティブなローファイ・プレースホルダー。
//!
//! # 呼び出し文脈
//!
//! `fandhe-frontend-docs-site` の `wireframes::card_basic` showcase
//! （`/wireframes/card-basic/`）から呼ばれる。`primary`/`secondary` は
//! `fandhe_frontend_core::text` のみで流し込むため、既定エスケープ
//! （REQ-1）は本モジュールが独自に保証する必要はなく core の `render()`
//! 契約に委譲される。`leading`/`trailing` は [`crate::icon`] や
//! [`crate::avatar`] が返す構築済み `Node` をそのまま受け取る
//! （`docs/design/wireframe-ui-architecture.md` §11.4 の `Node` スロット
//! 規約）。
//!
//! # API 設計の由来（原案からの差分）
//!
//! blocks.pm の Figma プロパティ構成（右アイコンの bool・asset の
//! swap・子要素のネスト props）をそのまま Rust 引数へ変換したものでは
//! ない。`docs/design/wireframe-ui-architecture.md` §2・§7（blocks.pm の
//! 外観・anatomy・プロパティ構成を閲覧・転記して構造的に一致させない、
//! PR #2670 の codex P1 指摘を受けた追記）に従い、API は §6・§11.4 の
//! 汎用規約から独自に設計した。先頭・末尾のスロットは §11.4 の
//! `Option<Node>` アイコンスロット規約へ統一し、`avatar` はこの部品の
//! 中から呼ばない（部品同士の合成は呼び出し側の選択とする、`toast`/
//! `alert` の `Option<Node>` スロットと同じ判断）。この差分は
//! `site/wireframes/card-basic.md` の「原案差分メモ」節にも記録する。
//!
//! `secondary` は [`crate::nav_item`] の `counter` と同じく
//! `Option<&str>` とし、`None` のときは要素自体を出力しない。表示状態
//! （`Active`/`Disabled`）は持たない（`progress`/`spinner` と同じく表示
//! 専用の data display 部品であるため）。

use fandhe_frontend_core::{el_owned, span, text, Node};

use crate::class::class_list;
use crate::size::Size;

/// 本文パート（primary/secondary をまとめる container）の class
/// （部品ルートなしで単独使用しない、[`card_basic`] 専用）。
const BODY_CLASS: &str = "fw-wire-card-basic-body";

/// 主テキストのパート class（[`card_basic`] 専用）。
const PRIMARY_CLASS: &str = "fw-wire-card-basic-primary";

/// 補足テキストのパート class（[`card_basic`] 専用）。
const SECONDARY_CLASS: &str = "fw-wire-card-basic-secondary";

/// Card basic CSS（ルート・body・primary・secondary の 4 セレクタ）。
/// [`crate::css::PARTS`] へ登録される。
///
/// 黒塗り二値ではなくグレースケールのトークン（`--fw-wire-ink-muted`）で
/// secondary を塗る（`docs/design/wireframe-ui-architecture.md`
/// §3「読みやすさ優先のグレースケール」）。寸法・文字サイズは
/// [`crate::size::css`] が定義するカスタムプロパティを `var()` で参照
/// するのみで、`size::SCALE` の値そのものはここへ書き写さない（同文書
/// §10.4）。
pub const CARD_BASIC_CSS: &str = "\
.fw-wire-card-basic {
  display: flex;
  align-items: center;
  gap: 0.75em;
  box-sizing: border-box;
  padding: 0.75em 1em;
  border: var(--fw-wire-line-width) solid var(--fw-wire-line);
  border-radius: var(--fw-wire-radius);
  background: var(--fw-wire-paper);
  color: var(--fw-wire-ink);
  font-family: var(--fw-wire-font-family);
  font-size: var(--fw-wire-font-size, 1rem);
  line-height: 1.4;
}
.fw-wire-card-basic-body {
  display: flex;
  flex: 1 1 auto;
  min-width: 0;
  flex-direction: column;
}
.fw-wire-card-basic-primary {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.fw-wire-card-basic-secondary {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: var(--fw-wire-ink-muted);
  font-size: 0.875em;
}
";

/// カード（Card basic）を組み立てる。
///
/// - `primary`: 必須の主テキスト。[`fandhe_frontend_core::text`] のみで
///   流し込む（REQ-1）。
/// - `secondary`: 省略可能な補足テキスト。`None` のときは要素自体を
///   出力しない（[`crate::nav_item`] の `counter` と同じ扱い）。
/// - `leading`: 省略可能な先頭スロット（`docs/design/wireframe-ui-architecture.md`
///   §11.4）。例: `Some(avatar(None, size, true))`。`None` のときは
///   スロット要素自体を出力しない。
/// - `trailing`: 省略可能な末尾スロット。`leading` と同じ規約。例:
///   `Some(icon::ellipsis(size))`。
/// - `size`: [`Size`] 5 段。ルート class `fw-wire-size-<段階>` として
///   付与する。
///
/// 出力構造はルート `div` の子として `[leading]` → `div.fw-wire-card-basic-body`
/// （`span.fw-wire-card-basic-primary` + `[span.fw-wire-card-basic-secondary]`）
/// → `[trailing]` の順。`role`/`aria-*`/`tabindex`/`style`/`href`/`src`/
/// `on*`/表示状態の `data-*` は一切出力せず、`<a>`/`<button>`/`<img>` も
/// 出力しない（`docs/design/wireframe-ui-architecture.md` §5/§7）。
/// スロットは構築済みの `Node` をそのまま子として差し込むのみで、
/// `format!` によるマークアップ組み立て・`raw_html` は使わない。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_wireframe_ui::{card_basic, icon, Size};
///
/// let node = card_basic(
///     "山田太郎",
///     Some("エンジニア"),
///     Some(icon::user(Size::Md)),
///     Some(icon::ellipsis(Size::Md)),
///     Size::Md,
/// );
/// let html = render(&node);
/// assert!(html.contains(r#"class="fw-wire-card-basic fw-wire-size-md""#));
/// assert!(html.contains(r#"class="fw-wire-card-basic-primary""#));
/// assert!(html.contains(r#"class="fw-wire-card-basic-secondary""#));
/// assert!(html.contains("山田太郎"));
/// assert!(html.contains("エンジニア"));
///
/// // secondary を省略すると要素自体を出力しない。
/// let no_secondary = card_basic("見出しのみ", None, None, None, Size::Md);
/// let no_secondary_html = render(&no_secondary);
/// assert!(!no_secondary_html.contains("fw-wire-card-basic-secondary"));
/// assert!(!no_secondary_html.contains("<svg"));
///
/// // XSS 回帰: primary/secondary は既定エスケープを経由する。
/// let escaped = card_basic(
///     "<script>alert(1)</script>",
///     Some("<script>alert(2)</script>"),
///     None,
///     None,
///     Size::Md,
/// );
/// let escaped_html = render(&escaped);
/// assert!(!escaped_html.contains("<script>"));
/// assert!(escaped_html.contains("&lt;script&gt;"));
/// ```
#[must_use]
pub fn card_basic(
    primary: &str,
    secondary: Option<&str>,
    leading: Option<Node>,
    trailing: Option<Node>,
    size: Size,
) -> Node {
    let class = class_list("fw-wire-card-basic", &[Some(size.class())]);

    let mut body_children: Vec<Node> =
        vec![span(vec![("class", PRIMARY_CLASS)], vec![text(primary)])];
    if let Some(secondary) = secondary {
        body_children.push(span(
            vec![("class", SECONDARY_CLASS)],
            vec![text(secondary)],
        ));
    }

    let mut children: Vec<Node> = Vec::new();
    if let Some(leading) = leading {
        children.push(leading);
    }
    children.push(el_owned(
        "div",
        vec![("class".to_string(), BODY_CLASS.to_string())],
        body_children,
    ));
    if let Some(trailing) = trailing {
        children.push(trailing);
    }

    el_owned("div", vec![("class".to_string(), class)], children)
}
