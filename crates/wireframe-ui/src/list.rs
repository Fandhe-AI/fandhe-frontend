//! 箇条書きリスト部品（`List`、イシュー #2657、Phase 7「Data display」）。
//!
//! 項目を縦に積んだ箇条書き（先頭マーカー + テキストの繰り返し）の配置
//! イメージだけを示す、非インタラクティブなローファイ・プレースホルダー。
//! blocks.pm に対応部品はなく、wireframe-ui 独自追加 14 部品の 1 つ
//! （`docs/design/wireframe-ui-architecture.md` §8）。
//!
//! # 呼び出し文脈
//!
//! `fandhe-frontend-docs-site` の `wireframes::list` showcase
//! （`/wireframes/list/`）から呼ばれる。項目は構築済み `Node` として受け
//! 取るため、テキストの既定エスケープ（REQ-1）は項目ノードを組み立てる
//! 呼び出し元（`fandhe_frontend_core::text` 等）の契約に委譲される。
//!
//! # API 設計の由来（原案からの差分）
//!
//! - **`items: Vec<Node>`（`&[Node]` ではない）**: `crate::grid`/
//!   `crate::frame`/`crate::stack` と同じく、子ツリー全体の不要な
//!   `clone()` を避けるため所有権を受け取る（`coding-rust.md`「不要な
//!   clone を避ける」）。この差分は `site/wireframes/list.md` の
//!   「原案差分メモ」節にも記録する。
//! - **`Size` 引数を持たない**: [`crate::stack`] の「API 設計の由来」と
//!   同じ理由で、任意 `Node` を子として受け取る容器のルートに
//!   `fw-wire-size-*` を付けると `--fw-wire-font-size`/
//!   `--fw-wire-control-size` が子孫へ意図せず継承され、呼び出し側が
//!   `Vec<Node>` に何を渡すか関知しない Stack/List の責務を越えて子部品の
//!   寸法まで暗黙に変える。マーカー寸法は継承フォントに合わせて `em`
//!   基準にする。
//! - **`ordered: bool` は部品固有の修飾 class**: [`ORDERED_CLASS`]
//!   （`fw-wire-list-ordered`）として表現し、`crate::props` へ共通型を
//!   追加しない（[`crate::avatar`] の `circle: bool` と同じ判断）。
//! - **`<ul>`/`<ol>`/`<li>` は出力しない**: `docs/design/wireframe-ui-architecture.md`
//!   §7（非対話制約）と [`crate::stepper`]/[`crate::breadcrumbs`] の先例
//!   に従い、`div`/`span` だけでレイアウトを表す。
//! - **マーカーは CSS 擬似要素のみで描く**: DOM へマーカーノードを出力
//!   しない。箇条書きは `::before` の小円、番号付きは CSS カウンタ
//!   （`counter-reset`/`counter-increment`/`content: counter(...)`）で
//!   描く（[`crate::breadcrumbs`] の区切り記号・[`crate::stepper`] の
//!   連結線と同じ先例）。
//! - **入れ子リスト**: 項目セレクタを子結合子 `>` で書き、カウンタは
//!   ルート単位でリセットする（[`LIST_CSS`] 参照）。入れ子にしたとき、
//!   内側のリストへ外側の番号・マーカーが漏れない。`list()` は
//!   `items` の各要素をそのまま 1 項目としてラップするため、ある項目に
//!   「本文 + 入れ子 `list()`」を持たせたい場合は、呼び出し側が
//!   `fandhe_frontend_core::div(vec![], vec![text("本文"), list(...)])`
//!   のように 1 つの `Node` へ合成してから渡す（入れ子 `list()` を
//!   `items` の**別要素**として並べると、ネストではなく単なる隣接項目
//!   になり `ordered` のカウンタも余分に 1 つ進む。イシュー #2657
//!   Review 指摘、当初の `crates/docs-site/src/wireframes/list.rs`
//!   Demo「入れ子」節がこの誤用パターンだったため、番号がずれたうえ
//!   字下げもされていなかった）。[`LIST_CSS`] は項目内のどの深さに
//!   現れた入れ子 `.fw-wire-list` も子孫セレクタで字下げするため、上記の
//!   合成方法であれば呼び出し側が追加の CSS を書く必要はない。
//! - **行頭アイコンはスロット化しない**: 行頭にアイコンを置きたい場合は
//!   呼び出し側が項目 `Node` 自体（例: [`crate::rich_text`] や
//!   `icon::*` + `text` の組み合わせ）で表現する。マーカー差し替え用の
//!   `Option<Node>` スロットは持たない（利用実績が出た時点で別イシュー
//!   として検討する、out-of-scope）。
//! - **表示状態を持たない**: `data-*` は出力しない（[`crate::stack`]/
//!   [`crate::annotation`] と同じ）。
//! - **件数の上限を設けない**: [`crate::breadcrumbs`] の先例と同じく、
//!   件数は呼び出し側が決める。空の `Vec` でも panic せず項目 0 件の
//!   ルートだけを出す。

use fandhe_frontend_core::{el_owned, Node};

use crate::class::class_list;

/// リスト項目 1 件のラッパー class（部品ルートなしで単独使用しない、
/// [`list`] 専用）。
const ITEM_CLASS: &str = "fw-wire-list-item";

/// `ordered=true` のときのみ付与する部品固有の修飾 class。
const ORDERED_CLASS: &str = "fw-wire-list-ordered";

/// リスト CSS（ルート・項目・箇条書きマーカー・番号付き修飾の 4 セレクタ
/// 系統）。[`crate::css::PARTS`] へ登録される。
///
/// 寸法・間隔はすべて `em` 基準（継承フォントに合わせる）で、`Size` の
/// スコープ付きカスタムプロパティ（`--fw-wire-control-size` 等）は
/// 参照しない（モジュール doc「API 設計の由来」参照）。項目セレクタは
/// 子結合子 `>` で書き、入れ子リストへ外側のカウンタ・マーカーが漏れない
/// ようにする。項目内に現れた入れ子 `.fw-wire-list`（直接の子・孫の
/// いずれも）へは子孫セレクタで左マージンの字下げ規則を適用する
/// （モジュール doc「入れ子リスト」参照）。
pub const LIST_CSS: &str = "\
.fw-wire-list {
  display: flex;
  flex-direction: column;
  gap: 0.5em;
  box-sizing: border-box;
  font-family: var(--fw-wire-font-family);
  font-size: var(--fw-wire-font-size, 1rem);
  color: var(--fw-wire-ink);
}
.fw-wire-list > .fw-wire-list-item {
  display: flex;
  align-items: baseline;
  gap: 0.5em;
}
.fw-wire-list > .fw-wire-list-item::before {
  content: \"\";
  flex-shrink: 0;
  align-self: center;
  width: 0.35em;
  height: 0.35em;
  border-radius: 50%;
  background: var(--fw-wire-ink-muted);
}
.fw-wire-list > .fw-wire-list-item .fw-wire-list {
  margin-left: 1.5em;
  margin-top: 0.25em;
}
.fw-wire-list.fw-wire-list-ordered {
  counter-reset: fw-wire-list;
}
.fw-wire-list.fw-wire-list-ordered > .fw-wire-list-item {
  counter-increment: fw-wire-list;
}
.fw-wire-list.fw-wire-list-ordered > .fw-wire-list-item::before {
  content: counter(fw-wire-list) \".\";
  align-self: baseline;
  width: auto;
  height: auto;
  border-radius: 0;
  background: none;
  color: var(--fw-wire-ink-muted);
  font-variant-numeric: tabular-nums;
}
";

/// リストを組み立てる。
///
/// - `items`: 項目ノード列。渡した順序どおりに描画する。各要素は
///   [`ITEM_CLASS`]（`fw-wire-list-item`）でラップしてそのまま子にする。
///   空でもルート `div` 自体は出力する（panic しない）。
/// - `ordered`: `true` のとき部品固有の修飾 class [`ORDERED_CLASS`]
///   （`fw-wire-list-ordered`）を付与し、マーカーを箇条書きの小円から
///   CSS カウンタによる番号へ切り替える。
///
/// ルートは `div.fw-wire-list`（+ 修飾）のみで、`<ul>`/`<ol>`/`<li>`・
/// `role`/`aria-*`/`tabindex`/`style`/`data-*` は一切出力しない
/// （`docs/design/wireframe-ui-architecture.md` §5/§7）。マーカー・番号は
/// [`LIST_CSS`] の擬似要素のみで描き、DOM へマーカーノードを出力しない。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::{render, text};
/// use fandhe_frontend_wireframe_ui::list;
///
/// // 箇条書き（既定）。
/// let node = list(vec![text("A"), text("B"), text("C")], false);
/// let html = render(&node);
/// assert!(html.contains(r#"class="fw-wire-list""#));
/// assert!(!html.contains("fw-wire-list-ordered"));
/// assert_eq!(html.matches(r#"class="fw-wire-list-item""#).count(), 3);
///
/// // 番号付き。
/// let ordered = list(vec![text("A"), text("B")], true);
/// assert!(render(&ordered).contains(r#"class="fw-wire-list fw-wire-list-ordered""#));
///
/// // 空でも panic せず項目 0 件になる。
/// let empty = list(vec![], false);
/// assert!(!render(&empty).contains("fw-wire-list-item"));
///
/// // XSS 回帰: 項目テキストは既定エスケープを経由する。
/// let escaped = list(vec![text("<script>alert(1)</script>")], false);
/// let escaped_html = render(&escaped);
/// assert!(!escaped_html.contains("<script>alert(1)</script>"));
/// assert!(escaped_html.contains("&lt;script&gt;"));
///
/// // 対話セマンティクスは一切出力しない。
/// assert!(!html.contains("<ul"));
/// assert!(!html.contains("<ol"));
/// assert!(!html.contains("<li"));
/// assert!(!html.contains(" role=\""));
/// assert!(!html.contains(" aria-"));
///
/// // 入れ子: 「本文 + 入れ子 list()」を持たせたい項目は
/// // `fandhe_frontend_core::div` で 1 つの Node に合成してから渡す
/// // （入れ子 list() を items の別要素として並べない。モジュール doc
/// // 「入れ子リスト」参照）。
/// use fandhe_frontend_core::div;
/// let nested = list(vec![text("子項目 A-1"), text("子項目 A-2")], false);
/// let parent_item = div(vec![], vec![text("親項目 A"), nested]);
/// let outer = list(vec![parent_item, text("親項目 B")], true);
/// let outer_html = render(&outer);
/// // 項目ラッパーは外側 2 件 + 入れ子 2 件の計 4 件。外側の並びは
/// // 「親項目 A（本文 + 入れ子）」「親項目 B」の 2 件のまま。
/// assert_eq!(outer_html.matches(r#"class="fw-wire-list-item""#).count(), 4);
/// assert_eq!(
///     outer_html.matches(r#"class="fw-wire-list fw-wire-list-ordered""#).count(),
///     1
/// );
/// ```
#[must_use]
pub fn list(items: Vec<Node>, ordered: bool) -> Node {
    let class = class_list("fw-wire-list", &[ordered.then_some(ORDERED_CLASS)]);

    let children: Vec<Node> = items
        .into_iter()
        .map(|item| {
            el_owned(
                "div",
                vec![("class".to_string(), ITEM_CLASS.to_string())],
                vec![item],
            )
        })
        .collect();

    el_owned("div", vec![("class".to_string(), class)], children)
}
