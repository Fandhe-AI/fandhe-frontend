//! ナビゲーション項目 1 行分のプレースホルダー部品（`Nav item`、
//! イシュー #2636、Phase 5「Navigation」の 2 番目の部品。main 取り込み時点
//! で Tabs（イシュー #2638）が先に Phase 5 最初の部品として merge 済み）。
//!
//! 先頭アイコン + ラベル + 件数表示（カウンター）+ 任意の末尾アイコンを
//! 幅いっぱいの 1 行（または縦積み）で並べる非インタラクティブな
//! ローファイ・プレースホルダー。アクティブ（選択中）状態を表現できるが、
//! 実際に遷移するリンク・ボタンとしての振る舞いは一切持たない。
//!
//! # 呼び出し文脈
//!
//! `fandhe-frontend-docs-site` の `wireframes::nav_item` showcase
//! （`/wireframes/nav-item/`）から呼ばれる。`fandhe_frontend_core::text`
//! のみでラベル・カウンターを流し込むため、既定エスケープ（REQ-1）は
//! 本モジュールが独自に保証する必要はなく core 側の契約に委譲される。
//! 先頭・末尾のスロットは [`crate::icon`] が返す構築済み `Node`
//! （`icon::house(size)` 等）をそのまま受け取る（`docs/design/wireframe-ui-architecture.md`
//! §11.4 の `Node` スロット規約。同文書は nav-item をこの規約の標準
//! 適用先として名指ししている）。
//!
//! # API 設計の由来
//!
//! blocks.pm の Nav item 部品の Figma プロパティ構成をそのまま転写した
//! ものではない（`docs/design/wireframe-ui-architecture.md` §2、Figma
//! プラグイン・スクリーンショットを開いて外観・プロパティ構成を書き写さ
//! ない全部品共通前提。同文書 §7）。API は §6/§11.4 と既存部品
//! （[`crate::rich_text`] のスロット + ラベル + `Orientation`、
//! [`crate::input`] の `Active`/`Disabled` `.attr()` 併用形）から独立
//! 設計した（`site/wireframes/nav-item.md` の「原案差分メモ」節も参照）。
//!
//! `counter` は `Node` スロットではなく `Option<&str>` の内部パートと
//! した。Phase 7 に単独部品 `counter`（#2655）が予定されているが、
//! 本イシュー時点で未実装のため依存できず、また nav 行内の件数ピルは
//! 行レイアウト（アクティブ時の反転配色を含む）と一体であるため。
//! `Disabled` は持たない（イシューの要求範囲外であり、`too_many_arguments`
//! の上限（8 個以上で発火）にちょうど収まる 7 引数を超えないため）。

use fandhe_frontend_core::{el_owned, span, text, Node};

use crate::class::class_list;
use crate::props::{Active, Orientation};
use crate::size::Size;

/// ラベルのパート class（部品ルートなしで単独使用しない、[`nav_item`] 専用）。
const LABEL_CLASS: &str = "fw-wire-nav-item-label";

/// カウンターのパート class（部品ルートなしで単独使用しない、[`nav_item`] 専用）。
const COUNTER_CLASS: &str = "fw-wire-nav-item-counter";

/// ナビゲーション項目 CSS（6 セレクタ）。[`crate::css::PARTS`] へ登録される。
///
/// `[data-active]` は必ず `.fw-wire-nav-item` に連結した形
/// （`.fw-wire-nav-item[data-active]`）で書く（素の `[data-active]`
/// セレクタは書かない、`docs/design/wireframe-ui-architecture.md` §10
/// の追記契約）。アクティブ時はグレースケール反転（背景を `--fw-wire-ink`、
/// 文字を `--fw-wire-paper`）で選択中を表現する。アイコンは `currentColor`
/// を使う設計（[`crate::icon::ICON_GLYPH_CSS`]）のため自動で追従する。
/// フォントサイズ等の値は `size::SCALE` から書き写さず `var()` を参照する
/// （同文書 §10「値を書き写さない」）。
pub const NAV_ITEM_CSS: &str = "\
.fw-wire-nav-item {
  display: flex;
  align-items: center;
  gap: 0.5em;
  width: 100%;
  box-sizing: border-box;
  padding: 0.5em 0.75em;
  border-radius: var(--fw-wire-radius);
  color: var(--fw-wire-ink);
  font-family: var(--fw-wire-font-family);
  font-size: var(--fw-wire-font-size, 1rem);
  line-height: 1.4;
}
.fw-wire-nav-item-label {
  flex: 1 1 auto;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.fw-wire-nav-item-counter {
  flex: 0 0 auto;
  padding: 0.125em 0.5em;
  border-radius: 999px;
  background: var(--fw-wire-fill-subtle);
  color: var(--fw-wire-ink-muted);
  font-size: 0.875em;
  line-height: 1.4;
}
.fw-wire-nav-item[data-active] {
  background: var(--fw-wire-ink);
  color: var(--fw-wire-paper);
}
.fw-wire-nav-item[data-active] .fw-wire-nav-item-counter {
  background: var(--fw-wire-paper);
  color: var(--fw-wire-ink);
}
.fw-wire-nav-item.fw-wire-vertical {
  flex-direction: column;
  align-items: center;
  gap: 0.25em;
}
";

/// ナビゲーション項目を組み立てる。
///
/// - `label`: 必須。表示するラベル文言。
/// - `leading`: 省略可能な先頭アイコンスロット（`docs/design/wireframe-ui-architecture.md`
///   §11.4）。例: `Some(icon::house(size))`。`None` のときはスロット要素
///   自体を出力しない。
/// - `trailing`: 省略可能な末尾アイコンスロット。`leading` と同じ規約
///   （例: `icon::caret_right(size)`）。
/// - `counter`: 省略可能な件数表示。`Some("12")` のときのみ
///   `fw-wire-nav-item-counter` パート要素を出力する。`None` のときは
///   要素自体を出力しない。
/// - `size`: [`Size`] 5 段。ルート class `fw-wire-size-<段階>` として付与し、
///   フォントサイズは [`crate::size::css`] が定義する `--fw-wire-font-size`
///   を `var()` で参照する（本モジュールは値を書き写さない）。
/// - `active`: [`Active`]。`true` のとき `data-active=""` を付与し、
///   グレースケール反転配色で選択中を表現する。
/// - `orientation`: [`Orientation`]。`Horizontal`（既定）は横一行、
///   `Vertical` はアイコンの下にラベルを置く縦積み（タブバー風）になる。
///
/// ラベル・カウンターは [`fandhe_frontend_core::text`] のみで流し込み
/// （REQ-1 既定エスケープ）、`role`/`aria-*`/`tabindex`/`style`/`href`/
/// `on*` は一切付与しない。`<a>`/`<button>` も出力しない
/// （`docs/design/wireframe-ui-architecture.md` §7、ナビ項目であっても
/// 実際に遷移するリンクにはしない）。スロットは構築済みの `Node` を
/// そのまま子として差し込むのみで、`format!` によるマークアップ組み立て・
/// `raw_html` は使わない。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_wireframe_ui::{icon, nav_item, Active, Orientation, Size};
///
/// let node = nav_item(
///     "ホーム",
///     Some(icon::house(Size::Md)),
///     Some(icon::caret_right(Size::Md)),
///     Some("12"),
///     Size::Md,
///     Active(true),
///     Orientation::Horizontal,
/// );
/// let html = render(&node);
/// assert!(html.contains(r#"class="fw-wire-nav-item fw-wire-size-md fw-wire-horizontal""#));
/// assert!(html.contains(r#"class="fw-wire-nav-item-label""#));
/// assert!(html.contains(r#"class="fw-wire-nav-item-counter""#));
/// assert!(html.contains(r#"data-active="""#));
/// assert!(html.contains("ホーム"));
/// assert!(html.contains("12"));
///
/// // スロット・カウンターをいずれも省略すると要素自体を出力しない。
/// let minimal = nav_item(
///     "設定",
///     None,
///     None,
///     None,
///     Size::Md,
///     Active(false),
///     Orientation::Horizontal,
/// );
/// let minimal_html = render(&minimal);
/// assert!(!minimal_html.contains("<svg"));
/// assert!(!minimal_html.contains("fw-wire-nav-item-counter"));
/// assert!(!minimal_html.contains("data-active"));
///
/// // XSS 回帰: ラベル・カウンターは既定エスケープを経由する。
/// let escaped = nav_item(
///     "<script>alert(1)</script>",
///     None,
///     None,
///     Some("<script>alert(2)</script>"),
///     Size::Md,
///     Active(false),
///     Orientation::Horizontal,
/// );
/// let escaped_html = render(&escaped);
/// assert!(!escaped_html.contains("<script>"));
/// assert!(escaped_html.contains("&lt;script&gt;"));
/// ```
#[must_use]
pub fn nav_item(
    label: &str,
    leading: Option<Node>,
    trailing: Option<Node>,
    counter: Option<&str>,
    size: Size,
    active: Active,
    orientation: Orientation,
) -> Node {
    let class = class_list(
        "fw-wire-nav-item",
        &[Some(size.class()), Some(orientation.class())],
    );

    let mut children: Vec<Node> = Vec::new();
    if let Some(leading) = leading {
        children.push(leading);
    }
    children.push(span(vec![("class", LABEL_CLASS)], vec![text(label)]));
    if let Some(counter) = counter {
        children.push(span(vec![("class", COUNTER_CLASS)], vec![text(counter)]));
    }
    if let Some(trailing) = trailing {
        children.push(trailing);
    }

    let mut attrs: Vec<(String, String)> = vec![("class".to_string(), class)];
    attrs.extend(active.attr());

    el_owned("div", attrs, children)
}
