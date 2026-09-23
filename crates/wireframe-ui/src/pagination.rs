//! ページネーション部品（`Pagination`、イシュー #2640、Phase 5
//! 「Navigation」）。
//!
//! 「前へ/次へ・ページ番号の並び・省略記号（…）・現在ページの強調」という
//! 配置イメージだけを示す、非インタラクティブなローファイ・プレースホル
//! ダー。実際にページ送りできる部品ではなく、`<nav>`/`<a>`/`href`/
//! `<button>` のいずれも出力しない表示専用部品である点に注意する。
//!
//! # 呼び出し文脈
//!
//! `fandhe-frontend-docs-site` の `wireframes::pagination` showcase
//! （`/wireframes/pagination/`）から呼ばれる。ページラベルは
//! [`fandhe_frontend_core::text`] のみで流し込むため、既定エスケープ
//! （REQ-1）は本モジュールが独自に保証する必要はなく core 側の契約に
//! 委譲される。
//!
//! # API 設計の由来
//!
//! `docs/design/wireframe-ui-architecture.md` §6 の汎用変換規約と、
//! Phase 4「Forms B」既存部品（[`crate::calendar`]）・Phase 5 既存部品
//! （[`crate::tabs`]）の先例から独立設計した（`site/wireframes/pagination.md`
//! の「原案差分メモ」節も参照）。blocks.pm の Pagination 部品が持つ Figma
//! プロパティ構成（現在ページ・先頭/前/次/末尾の個別 bool・ページラベル
//! ごとの固定スロット）は書き写さない（同文書 §2、イシュー #2602）。
//!
//! 設計判断:
//!
//! 1. **ページ項目は `&[Option<&str>]`**: `Some(label)` がページ番号
//!    セル、`None` が省略記号（…）のギャップセルを表す
//!    ([`crate::calendar`] の `Option<u32>`（`None` = 空きマス）と同じ
//!    表現）。利用者に `"…"` という文字列そのものを渡させると通常の
//!    ページセル（枠線付き）として描画されてしまうため、ギャップは構造
//!    として区別する。ページ番号の妥当性検証・現在ページ前後の自動省略
//!    計算はアプリケーションロジックとして責務外とする
//!    （`docs/policy/intentional-non-adoption.md` §3.25 と同じ判断軸）。
//! 2. **選択状態は `active: Option<usize>`**（[`crate::tabs`] と同型）:
//!    `pages` への添字。`None`・範囲外（`pages.len()` 以上）・ギャップ
//!    （`None` 要素）を指す場合は、どのセルにも `data-active` を付けない
//!    （決定的・fail-closed、`unwrap`/`expect`/`panic` は使わない）。
//!    新しい専用型は導入せず、既存の [`crate::props::Active`] を再利用
//!    する（[`crate::radio`]/[`crate::tabs`] の先例と同じ判断）。
//! 3. **先頭/前/次/末尾コントロールは 2 つの bool に畳む**: Figma の 4 つ
//!    の独立 bool は実用上「前後」と「先頭末尾」の対で使われるため、
//!    `prev_next: bool`（前/次）・`first_last: bool`（先頭/末尾）の 2 引数
//!    へ畳む（§6 の boolean 爆発の畳み込み。生 bool 引数は
//!    [`crate::frame::frame`] の `bordered: bool` に先例がある）。新しい
//!    props 型・構造体は導入しない。片側だけの表示が必要なケースは本
//!    部品の対象外とする。
//! 4. **[`crate::props::Orientation`] は持たない**: ページ送りは水平のみ
//!    （`props.rs` の doc が消費者として挙げていない）。`Disabled`/
//!    `Bold`/`Primary`、アイコン差し替えスロット、件数表示・ページ
//!    サイズ選択も持たない。
//! 5. **資源有界化は不要**: 出力ノード数は `pages.len()` に線形であり、
//!    1 引数から増幅する経路がないため `MAX_*` 定数は設けない
//!    （[`crate::stepper`]/[`crate::tabs`] と同じ判断）。空スライスでも
//!    panic しない。
//!
//! # 対話セマンティクスは出力しない（最重要）
//!
//! `docs/design/wireframe-ui-architecture.md` §7（非対話制約）に従い、
//! ルートは `div` とし `role`/`aria-*`（アイコン基盤の装飾用
//! `aria-hidden` を除く）/`tabindex`/`on*`/`style` は一切出力しない。
//! `<nav>`/`<a>`/`href`/`<button>`/`<input>`/`<select>` も出力しない。
//! 実際に操作可能なページネーションが必要な利用者には Primitives/Themes
//! の Pagination を案内する（`site/wireframes/pagination.md` 参照）。

use fandhe_frontend_core::{el_owned, span, text, Node};

use crate::class::class_list;
use crate::icon;
use crate::props::Active;
use crate::size::Size;

/// ページ番号セルのパート class（部品ルートなしで単独使用しない、
/// [`pagination`] 専用）。
const ITEM_CLASS: &str = "fw-wire-pagination-item";

/// 省略記号（ギャップ）セルのパート class。
const GAP_CLASS: &str = "fw-wire-pagination-gap";

/// 先頭ページへのコントロール（`fw-wire-pagination-control` + 専用修飾）。
/// [`class_list`] ではなく静的連結値とする（要素数 2 固定・利用者入力を
/// 含まないため）。
const CONTROL_FIRST_CLASS: &str = "fw-wire-pagination-control fw-wire-pagination-first";

/// 前ページへのコントロール。
const CONTROL_PREVIOUS_CLASS: &str = "fw-wire-pagination-control fw-wire-pagination-previous";

/// 次ページへのコントロール。
const CONTROL_NEXT_CLASS: &str = "fw-wire-pagination-control fw-wire-pagination-next";

/// 末尾ページへのコントロール。
const CONTROL_LAST_CLASS: &str = "fw-wire-pagination-control fw-wire-pagination-last";

/// ページネーション CSS（グレースケール、`ColorPalette` 非依存）。
/// [`crate::css::PARTS`] へ登録される。
///
/// `[data-active]` 単独セレクタは `crates/wireframe-ui/tests/common_api.rs`
/// の「`.` で始まる行はすべて `.fw-wire-` プレフィックス」走査に引っかから
/// ないよう `.fw-wire-pagination-item[data-active]` の形で書く
/// （`docs/design/wireframe-ui-architecture.md` §10.4）。黒塗り二値では
/// なく `--fw-wire-ink`/`--fw-wire-ink-muted`/`--fw-wire-fill-subtle` の
/// グレースケールで現在ページを強調し、読みやすさを優先する（§3 の
/// 配色方針）。
pub const PAGINATION_CSS: &str = "\
.fw-wire-pagination {
  display: inline-flex;
  align-items: center;
  gap: 0.25em;
  box-sizing: border-box;
  font-family: var(--fw-wire-font-family);
  font-size: var(--fw-wire-font-size, 1rem);
  color: var(--fw-wire-ink);
}
.fw-wire-pagination-item {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  box-sizing: border-box;
  min-width: 2em;
  height: 2em;
  padding: 0 0.5em;
  border: var(--fw-wire-line-width) solid var(--fw-wire-line-subtle);
  border-radius: var(--fw-wire-radius);
  color: var(--fw-wire-ink-muted);
  white-space: nowrap;
}
.fw-wire-pagination-item[data-active] {
  color: var(--fw-wire-ink);
  background: var(--fw-wire-fill-subtle);
  border-color: var(--fw-wire-ink);
  font-weight: 600;
}
.fw-wire-pagination-control {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 2em;
  height: 2em;
  color: var(--fw-wire-ink-muted);
}
.fw-wire-pagination-gap {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 2em;
  height: 2em;
  color: var(--fw-wire-ink-muted);
}
";

/// ページネーションのプレースホルダーを組み立てる。
///
/// - `pages`: ページ項目列。`Some(label)` はページ番号セル、`None` は
///   省略記号（…）のギャップセルを表す。要素数の上限はなく、空スライス
///   でも panic しない（コントロールのみ、または何も持たないルートを
///   出力する）。
/// - `active`: 現在ページの添字（`pages` への添字）。`None`・範囲外
///   （`pages.len()` 以上）・ギャップ（`None` 要素）を指す添字のときは、
///   どのセルにも選択インジケータ（`data-active`）を付けない
///   （決定的・fail-closed）。
/// - `prev_next`: `true` のとき前/次への送りコントロール（キャレット
///   アイコン 1 個ずつ）を出力する。
/// - `first_last`: `true` のとき先頭/末尾への送りコントロール（キャレット
///   アイコン 2 個ずつ）を出力する。
/// - `size`: [`Size`] 5 段。ルート class `fw-wire-size-<段階>` として
///   付与し、フォントサイズ・アイコンサイズは [`crate::size::css`] が
///   定義する `--fw-wire-font-size` を `em` 基準で参照する（本モジュール
///   は値を書き写さない）。
///
/// 出力順は 先頭 → 前 → ページ項目列（順序どおり）→ 次 → 末尾。テキスト
/// は [`fandhe_frontend_core::text`] のみで流し込み（REQ-1 既定
/// エスケープ）、`role`/`aria-*`（アイコン基盤の装飾用 `aria-hidden` を
/// 除く）/`tabindex`/`style`/`on*`/`<nav>`/`<a>`/`<button>` は一切出力
/// しない。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_wireframe_ui::{pagination, Size};
///
/// let pages = [Some("1"), Some("2"), None, Some("16"), Some("17")];
/// let node = pagination(&pages, Some(1), true, true, Size::Md);
/// let html = render(&node);
/// assert!(html.contains(r#"class="fw-wire-pagination fw-wire-size-md""#));
/// assert_eq!(html.matches(r#"class="fw-wire-pagination-item""#).count(), 4);
/// assert_eq!(html.matches(r#"class="fw-wire-pagination-gap""#).count(), 1);
/// assert_eq!(html.matches("data-active=\"\"").count(), 1);
/// assert!(html.contains("fw-wire-pagination-first"));
/// assert!(html.contains("fw-wire-pagination-previous"));
/// assert!(html.contains("fw-wire-pagination-next"));
/// assert!(html.contains("fw-wire-pagination-last"));
///
/// // 範囲外・None・ギャップ指しは選択インジケータなし。
/// let none_active = pagination(&pages, None, false, false, Size::Md);
/// assert!(!render(&none_active).contains("data-active"));
/// let out_of_range = pagination(&pages, Some(99), false, false, Size::Md);
/// assert!(!render(&out_of_range).contains("data-active"));
/// let gap_active = pagination(&pages, Some(2), false, false, Size::Md);
/// assert!(!render(&gap_active).contains("data-active"));
///
/// // コントロールなしは 1 セルも増えない。
/// let no_controls = pagination(&pages, None, false, false, Size::Md);
/// let no_controls_html = render(&no_controls);
/// assert!(!no_controls_html.contains("fw-wire-pagination-control"));
///
/// // 空スライスは panic せず項目 0 件になる。
/// let empty = pagination(&[], None, true, true, Size::Md);
/// let empty_html = render(&empty);
/// assert!(!empty_html.contains("fw-wire-pagination-item"));
/// assert!(!empty_html.contains("fw-wire-pagination-gap"));
///
/// // XSS 回帰: ラベルは既定エスケープを経由する。
/// let escaped = pagination(&[Some("<script>alert(1)</script>")], None, false, false, Size::Md);
/// let escaped_html = render(&escaped);
/// assert!(!escaped_html.contains("<script>alert(1)</script>"));
/// assert!(escaped_html.contains("&lt;script&gt;"));
///
/// // 対話セマンティクスは一切出力しない。
/// assert!(!html.contains("<nav"));
/// assert!(!html.contains("<a "));
/// assert!(!html.contains("href="));
/// assert!(!html.contains("<button"));
/// assert!(!html.contains(" role=\""));
/// assert!(!html.contains(" tabindex=\""));
/// ```
#[must_use]
pub fn pagination(
    pages: &[Option<&str>],
    active: Option<usize>,
    prev_next: bool,
    first_last: bool,
    size: Size,
) -> Node {
    let class = class_list("fw-wire-pagination", &[Some(size.class())]);

    let mut children: Vec<Node> = Vec::new();

    if first_last {
        children.push(span(
            vec![("class", CONTROL_FIRST_CLASS)],
            vec![icon::caret_left(size), icon::caret_left(size)],
        ));
    }
    if prev_next {
        children.push(span(
            vec![("class", CONTROL_PREVIOUS_CLASS)],
            vec![icon::caret_left(size)],
        ));
    }

    for (index, page) in pages.iter().enumerate() {
        match page {
            Some(label) => {
                let is_active = active == Some(index);
                let mut attrs: Vec<(String, String)> =
                    vec![("class".to_string(), ITEM_CLASS.to_string())];
                if let Some(attr) = Active(is_active).attr() {
                    attrs.push(attr);
                }
                children.push(el_owned("span", attrs, vec![text(*label)]));
            }
            None => {
                children.push(span(vec![("class", GAP_CLASS)], vec![icon::ellipsis(size)]));
            }
        }
    }

    if prev_next {
        children.push(span(
            vec![("class", CONTROL_NEXT_CLASS)],
            vec![icon::caret_right(size)],
        ));
    }
    if first_last {
        children.push(span(
            vec![("class", CONTROL_LAST_CLASS)],
            vec![icon::caret_right(size), icon::caret_right(size)],
        ));
    }

    el_owned("div", vec![("class".to_string(), class)], children)
}
