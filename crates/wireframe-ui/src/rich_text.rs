//! リッチテキスト行部品（`Rich text`、イシュー #2616、Phase 2「テキスト・注釈」）。
//!
//! アイコン + ラベル + 末尾アイコンを 1 行（または縦積み）で並べる
//! 非インタラクティブなローファイ・プレースホルダー。「アイコン付き行」の
//! 配置イメージ提示に特化し、実際のリンク・ボタンとしての振る舞いは
//! 一切持たない。
//!
//! # 呼び出し文脈
//!
//! `fandhe-frontend-docs-site` の `wireframes::rich_text` showcase
//! （`/wireframes/rich-text/`）から呼ばれる。`fandhe_frontend_core::text`
//! のみでラベルを流し込むため、既定エスケープ（REQ-1）は本モジュールが
//! 独自に保証する必要はなく core 側の契約に委譲される。先頭・末尾の
//! スロットは [`crate::icon`] が返す構築済み `Node`（`icon::plus(size)` 等）
//! をそのまま受け取る（`docs/design/wireframe-ui-architecture.md` §11.4
//! の `Node` スロット規約）。
//!
//! # API 設計の由来
//!
//! blocks.pm の Rich text 部品の Figma プロパティ構成をそのまま転写した
//! ものではない（`docs/design/wireframe-ui-architecture.md` §2、Figma
//! プラグイン・スクリーンショットを開いて外観・プロパティ構成を書き写さ
//! ない全部品共通前提。同文書 §7）。API は設計文書 §6「Text/Paragraph 系は
//! `Size` + `Bold` + テキスト」の 3 点セットと §11.4 の `leading`/`trailing`
//! `Option<Node>` スロットを組み合わせて独立設計した（イシュー #2616
//! 実装計画 §2.1、`site/wireframes/rich-text.md` の「原案差分メモ」節も
//! 参照）。`Orientation`（[`crate::props::Orientation`]）は stack/divider/
//! tabs と共用する既存の共通型であり、blocks.pm 固有の命名ではない。

use fandhe_frontend_core::{el_owned, span, text, Node};

use crate::class::class_list;
use crate::props::{Bold, Orientation};
use crate::size::Size;

/// パート class（部品ルートなしで単独使用しない、[`rich_text`] 専用）。
const LABEL_CLASS: &str = "fw-wire-rich-text-label";

/// リッチテキスト行 CSS（4 セレクタ）。[`crate::css::PARTS`] へ登録される。
///
/// `fw-wire-bold`/`fw-wire-vertical` は [`crate::props::Bold`]/
/// [`crate::props::Orientation`] が class 名のみを定義し、横断 CSS
/// （`crate::tokens::css`/`crate::size::css`/[`crate::css::PARTS`]）には
/// 対応する宣言が存在しなかった（Annotation は `Bold` を意図的に不採用の
/// ため未消費だった）。本部品が両 class の最初の消費者になるため、部品
/// スコープで宣言する（`docs/design/wireframe-ui-architecture.md` §10
/// 「部品 CSS は自分のルート配下のみ」に従い、横断ルールとしては新設
/// しない）。フォントサイズは `size::SCALE` の値を書き写さず
/// `var(--fw-wire-font-size, 1rem)` を参照する（同文書 §10「値を書き
/// 写さない」）。アイコングリフ自体の寸法は [`crate::icon::ICON_GLYPH_CSS`]
/// が `flex-shrink: 0`/`1em` を持つため本 CSS では追加宣言しない。
pub const RICH_TEXT_CSS: &str = "\
.fw-wire-rich-text {
  display: inline-flex;
  align-items: center;
  gap: 0.5em;
  max-width: 100%;
  color: var(--fw-wire-ink);
  font-family: var(--fw-wire-font-family);
  font-size: var(--fw-wire-font-size, 1rem);
  line-height: 1.4;
}
.fw-wire-rich-text-label {
  min-width: 0;
}
.fw-wire-rich-text.fw-wire-bold {
  font-weight: 600;
}
.fw-wire-rich-text.fw-wire-vertical {
  flex-direction: column;
  align-items: flex-start;
  gap: 0.25em;
}
";

/// リッチテキスト行を組み立てる。
///
/// - `label`: 必須。表示するラベル文言。
/// - `leading`: 省略可能な先頭スロット。`icon::plus(size)` のように
///   [`crate::icon`] の戻り値をそのまま `Some` で渡す（`Node` スロット
///   規約、`docs/design/wireframe-ui-architecture.md` §11.4）。`None` の
///   ときはスロット要素自体を出力しない。
/// - `trailing`: 省略可能な末尾スロット。`leading` と同じ規約（例:
///   `icon::caret_right(size)`）。
/// - `size`: [`Size`] 5 段。ルート class `fw-wire-size-<段階>` として付与し、
///   フォントサイズは [`crate::size::css`] が定義する `--fw-wire-font-size`
///   を `var()` で参照する（本モジュールは値を書き写さない）。
/// - `bold`: `true` のとき class `fw-wire-bold` を付与しラベルを太字にする。
/// - `orientation`: [`Orientation`]。`Horizontal`（既定）はスロット・
///   ラベルを横並びに、`Vertical` は縦積みにする。
///
/// ラベルは [`fandhe_frontend_core::text`] のみで流し込み（REQ-1 既定
/// エスケープ）、ルート・ラベルとも `role`/`aria-*`/`tabindex`/`style`/
/// `data-*` は一切付与しない（`docs/design/wireframe-ui-architecture.md`
/// §5/§7、本部品は表示状態軸を持たないため `data-*` も不要）。スロットは
/// 構築済みの `Node` をそのまま子として差し込むのみで、`format!` に
/// よるマークアップ組み立て・`raw_html` は使わない。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_wireframe_ui::{icon, rich_text, Bold, Orientation, Size};
///
/// let node = rich_text(
///     "設定",
///     Some(icon::cog(Size::Md)),
///     Some(icon::caret_right(Size::Md)),
///     Size::Md,
///     Bold(false),
///     Orientation::Horizontal,
/// );
/// let html = render(&node);
/// assert!(html.contains(r#"class="fw-wire-rich-text fw-wire-size-md fw-wire-horizontal""#));
/// assert!(html.contains(r#"class="fw-wire-rich-text-label""#));
/// assert!(html.contains("設定"));
/// assert!(html.contains("<svg"));
///
/// // スロットをいずれも省略すると `<svg` は出力されない。
/// let without_slots = rich_text("ラベルのみ", None, None, Size::Md, Bold(false), Orientation::Horizontal);
/// assert!(!render(&without_slots).contains("<svg"));
///
/// // XSS 回帰: ラベルは既定エスケープを経由する。
/// let escaped = rich_text(
///     "<script>alert(1)</script>",
///     None,
///     None,
///     Size::Md,
///     Bold(false),
///     Orientation::Horizontal,
/// );
/// let escaped_html = render(&escaped);
/// assert!(!escaped_html.contains("<script>alert(1)</script>"));
/// assert!(escaped_html.contains("&lt;script&gt;"));
/// ```
#[must_use]
pub fn rich_text(
    label: &str,
    leading: Option<Node>,
    trailing: Option<Node>,
    size: Size,
    bold: Bold,
    orientation: Orientation,
) -> Node {
    let class = class_list(
        "fw-wire-rich-text",
        &[Some(size.class()), bold.class(), Some(orientation.class())],
    );

    let mut children: Vec<Node> = Vec::new();
    if let Some(leading) = leading {
        children.push(leading);
    }
    children.push(span(vec![("class", LABEL_CLASS)], vec![text(label)]));
    if let Some(trailing) = trailing {
        children.push(trailing);
    }

    el_owned("div", vec![("class".to_string(), class)], children)
}
