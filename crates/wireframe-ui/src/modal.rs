//! モーダルダイアログ部品（`Modal`、イシュー #2645、Phase 6「Overlay・
//! Feedback」の 2 番目の部品。最初の部品は `tooltip`）。
//!
//! 「中央に置かれたダイアログ枠（タイトル・本文・アクション行）」の
//! 配置イメージだけを示す、非インタラクティブなローファイ・
//! プレースホルダー。開閉状態・フォーカストラップ・Esc 操作などの対話は
//! 一切持たない（`docs/design/wireframe-ui-architecture.md` §1/§5/§7）。
//!
//! # 呼び出し文脈
//!
//! `fandhe-frontend-docs-site` の `wireframes::modal` showcase
//! （`/wireframes/modal/`）から呼ばれる。タイトルは
//! [`fandhe_frontend_core::text`] のみで流し込むため既定エスケープ
//! （REQ-1）は core 側の契約に委譲され、本文・アクションは構築済み
//! `Node` スロットとしてそのまま 1 子・複数子として包むだけで検査・
//! 再加工を行わない。
//!
//! # API 設計の由来
//!
//! blocks.pm には対応部品がなく `wireframe-ui` 独自追加の 1 つ
//! （`docs/design/wireframe-ui-architecture.md` §8）。実際に操作可能な
//! ダイアログが必要な利用者には Themes（`/themes/dialog/`）/
//! Primitives（`/primitives/dialog/`）を案内する
//! （`site/wireframes/modal.md` 参照）。
//!
//! # 中央配置を `position: fixed` にしない理由（最重要）
//!
//! docs ページ全体を覆う `position: fixed`/`absolute`/`z-index` は他の
//! デモ枠のレイアウトを破壊するため使わない。ルート `.fw-wire-modal` は
//! in-flow のブロック（`display: grid; place-items: center;`）とし、
//! その内側の `.fw-wire-modal-panel` を中央寄せする。
//!
//! # `<dialog>`・`role`・`aria-*` 等は出力しない
//!
//! `docs/design/wireframe-ui-architecture.md` §7（非対話制約）に従い、
//! `<dialog>`/`role`（`dialog` 含む）/`aria-*`（`aria-modal` 含む）/
//! `tabindex`/`style`/`on*`/`<button>`/`<form>`/`<input>`/`<a href>` は
//! 一切出力しない。タイトルも見出し要素（`h1`〜`h6`）ではなく `div` で
//! 出力する（docs ページの目次・検索インデックスへ見出しとして混入
//! させないため。見た目は太字で表現する）。ルート自体は表示状態軸を
//! 持たないため `data-*` も付与しない（`body`/`actions` スロット内部の
//! `data-*` はそのまま透過する）。

use fandhe_frontend_core::{div, el_owned, text, Node};

use crate::class::class_list;
use crate::size::Size;

/// パート class（部品ルートなしで単独使用しない、[`modal`] 専用）。
const PANEL_CLASS: &str = "fw-wire-modal-panel";
const TITLE_CLASS: &str = "fw-wire-modal-title";
const BODY_CLASS: &str = "fw-wire-modal-body";
const ACTIONS_CLASS: &str = "fw-wire-modal-actions";

/// モーダル CSS（ルート・パネル・タイトル・本文・アクション行の 5 セレクタ +
/// `Size` 5 段のパネル最大幅ルール）。[`crate::css::PARTS`] へ登録される。
///
/// パネル最大幅は `size::SCALE` に存在しない部品固有値のため、
/// `crate::css` の非 `PARTS` 経路例外（`size::SCALE` 由来の動的生成関数）
/// には該当しない静的な 5 ルールとして本定数へ直書きする
/// （`docs/design/wireframe-ui-architecture.md` §10.4）。
///
/// 最大幅ルールは共有 `fw-wire-size-*`（`--fw-wire-font-size`/
/// `--fw-wire-control-size` を同時定義し子孫へ継承される）ではなく
/// Modal 専用の修飾 class `fw-wire-modal-max-width-<段階>`
/// （[`max_width_class`]）で宣言する。共有 class をルートへ付けると、
/// `body`/`actions` スロットへ渡される任意の `Node`（呼び出し側が
/// 何を渡すかは Modal の関知しない契約）や `panel`/`title` のタイポ
/// グラフィまで暗黙に変更してしまう（コードレビュー指摘。
/// `crate::frame` が `fw-wire-frame-padding-*` で・`crate::stack` が
/// `fw-wire-stack-gap-*` で同種の問題を先に回避した前例と同じ設計、
/// イシュー #2609）。このため `.fw-wire-modal-panel` は `font-size` を
/// 設定しない（レイアウト・サイズ責務をパネル最大幅に限定する）。
pub const MODAL_CSS: &str = "\
.fw-wire-modal {
  display: grid;
  place-items: center;
  min-height: 12rem;
  padding: 1.5rem;
  box-sizing: border-box;
  background: var(--fw-wire-fill-subtle);
}
.fw-wire-modal-panel {
  display: flex;
  flex-direction: column;
  gap: 0.75em;
  width: 100%;
  box-sizing: border-box;
  padding: 1.25em;
  background: var(--fw-wire-paper);
  border: var(--fw-wire-line-width) solid var(--fw-wire-line);
  border-radius: var(--fw-wire-radius);
  color: var(--fw-wire-ink);
  font-family: var(--fw-wire-font-family);
  line-height: 1.4;
}
.fw-wire-modal-title {
  font-weight: 600;
}
.fw-wire-modal-body {
  color: var(--fw-wire-ink-muted);
}
.fw-wire-modal-actions {
  display: flex;
  justify-content: flex-end;
  align-items: center;
  gap: 0.5em;
  flex-wrap: wrap;
}
.fw-wire-modal.fw-wire-modal-max-width-xs .fw-wire-modal-panel {
  max-width: 20rem;
}
.fw-wire-modal.fw-wire-modal-max-width-sm .fw-wire-modal-panel {
  max-width: 26rem;
}
.fw-wire-modal.fw-wire-modal-max-width-md .fw-wire-modal-panel {
  max-width: 32rem;
}
.fw-wire-modal.fw-wire-modal-max-width-lg .fw-wire-modal-panel {
  max-width: 40rem;
}
.fw-wire-modal.fw-wire-modal-max-width-xl .fw-wire-modal-panel {
  max-width: 50rem;
}
";

/// `size`（[`Size`]）を Modal 専用の最大幅修飾 class 名
/// （`fw-wire-modal-max-width-<段階>`）へ変換する。
///
/// [`crate::size::css`] が生成する共有 `fw-wire-size-*` class は
/// `--fw-wire-font-size`/`--fw-wire-control-size` を同時に定義するため
/// 使わない（[`MODAL_CSS`] doc 参照）。
const fn max_width_class(size: Size) -> &'static str {
    match size {
        Size::Xs => "fw-wire-modal-max-width-xs",
        Size::Sm => "fw-wire-modal-max-width-sm",
        Size::Md => "fw-wire-modal-max-width-md",
        Size::Lg => "fw-wire-modal-max-width-lg",
        Size::Xl => "fw-wire-modal-max-width-xl",
    }
}

/// モーダルダイアログの配置イメージを組み立てる。
///
/// - `title`: 必須。ダイアログのタイトル（常に太字で表示、見出し要素は
///   使わない）。
/// - `body`: 本文のスロット（[`crate::paragraph`] 等の戻り値をそのまま
///   渡す。instance swap 相当）。本モジュールは内容を検査・再加工せず
///   1 子としてそのまま包む。
/// - `actions`: アクション行のスロット群（[`crate::button`] 等の戻り値を
///   そのまま渡す）。空のときはアクション行のパート要素自体を出力しない
///   （空要素を残さない）。
/// - `size`: [`Size`] 5 段。ルート class `fw-wire-modal-max-width-<段階>`
///   （[`max_width_class`]）として付与し、パネルの最大幅にのみ効く
///   （共有 `fw-wire-size-*` は使わないため、`body`/`actions` スロットや
///   `title`/`panel` のフォントサイズには影響しない。[`MODAL_CSS`] doc
///   参照）。
///
/// タイトルは [`fandhe_frontend_core::text`] のみで流し込み（REQ-1 既定
/// エスケープ）、`<dialog>`/`role`/`aria-*`/`tabindex`/`style`/`data-*` は
/// 一切付与しない（`docs/design/wireframe-ui-architecture.md` §5/§7、
/// Modal はルート自体の表示状態軸を持たない部品のため `data-*` も不要。
/// `body`/`actions` スロット内部の `data-*`（スロット自身の状態表現）は
/// そのまま透過する）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::{div, render, text};
/// use fandhe_frontend_wireframe_ui::{modal, Size};
///
/// let body = div(vec![("class", "probe-body")], vec![text("本当に削除しますか？")]);
/// let ok = div(vec![("class", "probe-ok")], vec![text("OK")]);
/// let cancel = div(vec![("class", "probe-cancel")], vec![text("キャンセル")]);
/// let node = modal("確認", body, vec![cancel, ok], Size::Md);
/// let html = render(&node);
/// assert!(html.contains(r#"class="fw-wire-modal fw-wire-modal-max-width-md""#));
/// assert!(html.contains(r#"class="fw-wire-modal-panel""#));
/// assert!(html.contains(r#"class="fw-wire-modal-title""#));
/// assert!(html.contains("確認"));
/// assert!(html.contains(r#"class="fw-wire-modal-body""#));
/// assert!(html.contains("probe-body"));
/// assert!(html.contains(r#"class="fw-wire-modal-actions""#));
/// assert!(html.contains("probe-cancel"));
/// assert!(html.contains("probe-ok"));
///
/// // actions を空にするとアクション行のパート要素自体が出力されない。
/// let minimal_body = div(vec![], vec![text("本文")]);
/// let minimal = modal("タイトル", minimal_body, vec![], Size::Md);
/// let minimal_html = render(&minimal);
/// assert!(!minimal_html.contains("fw-wire-modal-actions"));
///
/// // XSS 回帰: title はいずれも既定エスケープを経由する。
/// let payload_body = div(vec![], vec![text("本文")]);
/// let escaped = modal(
///     "<script>alert(1)</script>",
///     payload_body,
///     vec![],
///     Size::Md,
/// );
/// let escaped_html = render(&escaped);
/// assert!(!escaped_html.contains("<script>alert(1)</script>"));
/// assert!(escaped_html.contains("&lt;script&gt;"));
/// ```
#[must_use]
pub fn modal(title: &str, body: Node, actions: Vec<Node>, size: Size) -> Node {
    let class = class_list("fw-wire-modal", &[Some(max_width_class(size))]);

    let mut panel_children: Vec<Node> = vec![
        div(vec![("class", TITLE_CLASS)], vec![text(title)]),
        div(vec![("class", BODY_CLASS)], vec![body]),
    ];
    if !actions.is_empty() {
        panel_children.push(el_owned(
            "div",
            vec![("class".to_string(), ACTIONS_CLASS.to_string())],
            actions,
        ));
    }

    let panel = div(vec![("class", PANEL_CLASS)], panel_children);

    el_owned("div", vec![("class".to_string(), class)], vec![panel])
}
