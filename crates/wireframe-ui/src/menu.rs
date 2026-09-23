//! メニュー部品（`Menu`、イシュー #2637、Phase 5「Navigation」の 5 番目の
//! 部品）。
//!
//! 検索欄（任意）と、有効・無効が混在する項目リストからなる
//! ドロップダウン風パネルの配置イメージだけを示す、非インタラクティブな
//! ローファイ・プレースホルダー。「Menu」という名前だが、`role="menu"`/
//! `menuitem`・`aria-expanded`/`aria-haspopup`・実際の開閉・キーボード
//! ナビゲーションのいずれも実装しない表示専用部品である点に注意する。
//!
//! # 呼び出し文脈
//!
//! `fandhe-frontend-docs-site` の `wireframes::menu` showcase
//! （`/wireframes/menu/`）から呼ばれる。`fandhe_frontend_core::text`
//! のみでラベル・検索プレースホルダー文言を流し込むため、既定エスケープ
//! （REQ-1）は本モジュールが独自に保証する必要はなく core 側の契約に
//! 委譲される。
//!
//! # API 設計の由来
//!
//! `docs/design/wireframe-ui-architecture.md` §6 の汎用変換規約から独立
//! 設計した（`site/wireframes/menu.md` の「原案差分メモ」節も参照）。
//! blocks.pm の Menu 部品が持つ Figma プロパティ構成（`Option1`〜
//! `Option5` の個別 bool+text スロット等）は書き写さない（同文書 §2、
//! イシュー #2602）。
//!
//! - **項目はスライスへ畳む**: 固定 5 スロットの bool+text は持たず、
//!   [`MenuItem`] のスライス `items: &[MenuItem]` で受ける（[`crate::tabs`]
//!   の `&[&str]` と同じ判断だが、項目ごとに無効状態が要るため
//!   `Copy` 構造体を公開する）。項目数に上限は設けない（[`crate::tabs`]/
//!   [`crate::stepper`] と同じく、1 項目 = 出力が線形に増えるだけ）。
//!   空スライスでも panic しない。
//! - **強調（選択中）状態は `active: Option<usize>`**: [`crate::tabs`] と
//!   同じく「強調中は高々 1 件」を型で保証する。`None` または範囲外の
//!   添字は `data-active` を付与しない。**無効な項目を指す添字も
//!   `data-active` を付与しない**（無効状態を優先する。決定的で見た目の
//!   矛盾を避ける）。`unwrap`/`expect`/`panic` は使わない。
//! - **検索欄は `search: Option<&str>`**: bool ではなく検索行の
//!   プレースホルダー文言を持つ内部パートとした（[`crate::nav_item`] の
//!   `counter: Option<&str>` と同型）。`None` のときは検索行を出力しない。
//!   `Some("")` は空文言の検索行になる。検索行の先頭は固定パートの
//!   [`crate::icon::search`] とし、`Option<Node>` スロットにはしない
//!   （[`crate::select`] のドロップダウン指示子 `caret_down` と同じ判断:
//!   利用者が省略・差し替えできない部品の同一性を担う要素のため）。
//!   `<input>` は出力しない。
//! - **アイコンスロット（§11.4）は持たない**: 項目のアイコン差し替えは
//!   対応範囲外とする（§11.4 は Menu をこの規約の適用先の例に挙げるが、
//!   本イシューでは非採用とする判断。詳細は原案差分メモ参照）。
//! - [`crate::props::Orientation`]/[`crate::props::Bold`]/
//!   [`crate::props::Primary`]、メニュー全体の
//!   [`crate::props::Disabled`] は持たない。
//!
//! # 対話セマンティクスは出力しない（最重要）
//!
//! `docs/design/wireframe-ui-architecture.md` §7（非対話制約）に従い、
//! ルートは `div` とし `role`/`aria-*`（アイコン基盤が付与する装飾用の
//! `aria-hidden="true"` を除く）/`tabindex`/`on*`/`style` は一切出力
//! しない。`<input>`/`<button>`/`<select>`/`<a href>` も出力しない。実際に
//! 操作可能なメニューが必要な利用者には Primitives/Themes の Menu を案内
//! する（`site/wireframes/menu.md` 参照）。

use fandhe_frontend_core::{el_owned, span, text, Node};

use crate::class::class_list;
use crate::icon;
use crate::props::{Active, Disabled};
use crate::size::Size;

/// メニュー項目 1 件（ラベル + 無効状態）。
///
/// 項目ごとに無効状態を持てるようにするための小さな公開 `Copy` 構造体
/// （[`crate::calendar::Week`] のような公開補助型の先例に倣う）。フィールド
/// を直接組み立てず [`MenuItem::new`]/[`MenuItem::disabled`] で構築する
/// ことを推奨するが、フィールドは公開されているため直接構築もできる。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MenuItem<'a> {
    /// 表示するラベル文言。
    pub label: &'a str,
    /// 無効状態。`true` のとき `data-disabled=""` を付与する。
    pub disabled: Disabled,
}

impl<'a> MenuItem<'a> {
    /// 有効な項目を作る。
    #[must_use]
    pub const fn new(label: &'a str) -> Self {
        MenuItem {
            label,
            disabled: Disabled(false),
        }
    }

    /// 無効な項目を作る。
    #[must_use]
    pub const fn disabled(label: &'a str) -> Self {
        MenuItem {
            label,
            disabled: Disabled(true),
        }
    }
}

/// 検索行のパート class（部品ルートなしで単独使用しない、[`menu`] 専用）。
const SEARCH_CLASS: &str = "fw-wire-menu-search";

/// 検索行のプレースホルダー文言のパート class（部品ルートなしで単独使用
/// しない、[`menu`] 専用）。
const SEARCH_TEXT_CLASS: &str = "fw-wire-menu-search-text";

/// メニュー項目 1 件のパート class（部品ルートなしで単独使用しない、
/// [`menu`] 専用）。
const ITEM_CLASS: &str = "fw-wire-menu-item";

/// メニュー CSS（グレースケール、`ColorPalette` 非依存）。
/// [`crate::css::PARTS`] へ登録される。
///
/// `[data-active]`/`[data-disabled]` 単独セレクタは
/// `crates/wireframe-ui/tests/common_api.rs` の「`.` で始まる行はすべて
/// `.fw-wire-` プレフィックス」走査に引っかからないよう
/// `.fw-wire-menu-item[data-active]`/`.fw-wire-menu-item[data-disabled]`
/// の形で書く（`docs/design/wireframe-ui-architecture.md` §10.4）。黒塗り
/// 二値ではなく `--fw-wire-fill-subtle`/`--fw-wire-ink-muted` のグレー
/// スケールで強調・無効を表現し、読みやすさを優先する（同文書 §3）。
pub const MENU_CSS: &str = "\
.fw-wire-menu {
  display: inline-flex;
  flex-direction: column;
  box-sizing: border-box;
  min-width: 12em;
  border: var(--fw-wire-line-width) solid var(--fw-wire-line);
  border-radius: var(--fw-wire-radius);
  background: var(--fw-wire-paper);
  color: var(--fw-wire-ink);
  font-family: var(--fw-wire-font-family);
  font-size: var(--fw-wire-font-size, 1rem);
}
.fw-wire-menu-search {
  display: flex;
  align-items: center;
  gap: 0.5em;
  box-sizing: border-box;
  padding: 0.5em 0.75em;
  border-bottom: var(--fw-wire-line-width) solid var(--fw-wire-line);
}
.fw-wire-menu-search-text {
  flex: 1 1 auto;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: var(--fw-wire-ink-muted);
}
.fw-wire-menu .fw-wire-icon-glyph {
  color: var(--fw-wire-ink-muted);
}
.fw-wire-menu-item {
  box-sizing: border-box;
  padding: 0.5em 0.75em;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.fw-wire-menu-item[data-active] {
  background: var(--fw-wire-fill-subtle);
  color: var(--fw-wire-ink);
}
.fw-wire-menu-item[data-disabled] {
  color: var(--fw-wire-ink-muted);
  opacity: 0.6;
}
";

/// メニューを組み立てる。
///
/// - `items`: メニュー項目のスライス（[`MenuItem`]）。固定スロットの
///   bool+text 列ではなくスライスで受け、要素数の上限は設けない。空
///   スライスのときは項目を持たないルート要素だけを出力する（panic
///   しない）。
/// - `active`: 強調（選択中）項目の添字。`None`、`items.len()` 以上の
///   範囲外値、または無効項目（`disabled: Disabled(true)`）を指す値の
///   ときは、どの項目にも `data-active` を付与しない（決定的・
///   fail-closed）。
/// - `search`: 省略可能な検索欄プレースホルダー文言。`Some(text)` のとき
///   のみ検索行（先頭固定の [`crate::icon::search`] + プレースホルダー
///   文言）を項目リストより前に出力する。`None` のときは検索行自体を
///   出力しない。`<input>` は出力しない。
/// - `size`: [`Size`] 5 段。ルート class `fw-wire-size-<段階>` として付与し、
///   フォントサイズは [`crate::size::css`] が定義する `--fw-wire-font-size`
///   を `var()` で参照する（本モジュールは値を書き写さない）。
///
/// ラベル・検索プレースホルダー文言は [`fandhe_frontend_core::text`] の
/// みで流し込み（REQ-1 既定エスケープ）、`role`/`aria-*`（アイコン基盤の
/// 装飾用 `aria-hidden` を除く）/`tabindex`/`style`/`on*` は一切出力
/// しない。`<input>`/`<button>`/`<select>`/`<a href>` も出力しない。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_wireframe_ui::menu::{menu, MenuItem};
/// use fandhe_frontend_wireframe_ui::Size;
///
/// let items = [
///     MenuItem::new("プロフィール"),
///     MenuItem::new("設定"),
///     MenuItem::disabled("請求情報"),
/// ];
/// let node = menu(&items, Some(0), Some("検索..."), Size::Md);
/// let html = render(&node);
/// assert!(html.contains(r#"class="fw-wire-menu fw-wire-size-md""#));
/// assert!(html.contains(r#"class="fw-wire-menu-search""#));
/// assert!(html.contains(r#"data-icon="search""#));
/// assert!(html.contains("検索..."));
/// assert_eq!(html.matches(r#"class="fw-wire-menu-item""#).count(), 3);
/// assert_eq!(html.matches(r#"data-active="""#).count(), 1);
/// assert_eq!(html.matches(r#"data-disabled="""#).count(), 1);
///
/// // search が None のときは検索行自体を出力しない。
/// let no_search = menu(&items, None, None, Size::Md);
/// let no_search_html = render(&no_search);
/// assert!(!no_search_html.contains("fw-wire-menu-search"));
/// assert!(!no_search_html.contains("<svg"));
///
/// // 無効な項目を指す active は data-active を付与しない。
/// let disabled_active = menu(&items, Some(2), None, Size::Md);
/// assert!(!render(&disabled_active).contains("data-active"));
///
/// // 範囲外・空スライスは panic せず安全側に倒す。
/// let out_of_range = menu(&items, Some(99), None, Size::Md);
/// assert!(!render(&out_of_range).contains("data-active"));
/// let empty = menu(&[], None, None, Size::Md);
/// assert!(!render(&empty).contains("fw-wire-menu-item"));
///
/// // XSS 回帰: ラベル・検索プレースホルダーは既定エスケープを経由する。
/// let escaped = menu(
///     &[MenuItem::new("<script>alert(1)</script>")],
///     None,
///     Some("<script>alert(2)</script>"),
///     Size::Md,
/// );
/// let escaped_html = render(&escaped);
/// assert!(!escaped_html.contains("<script>"));
/// assert!(escaped_html.contains("&lt;script&gt;"));
///
/// // 対話セマンティクス・ネイティブフォーム要素は一切出力しない。
/// assert!(!html.contains(" role=\""));
/// assert!(!html.contains("<input"));
/// assert!(!html.contains("<button"));
/// ```
#[must_use]
pub fn menu(
    items: &[MenuItem<'_>],
    active: Option<usize>,
    search: Option<&str>,
    size: Size,
) -> Node {
    let class = class_list("fw-wire-menu", &[Some(size.class())]);

    let mut children: Vec<Node> = Vec::new();

    if let Some(placeholder) = search {
        children.push(span(
            vec![("class", SEARCH_CLASS)],
            vec![
                icon::search(size),
                span(vec![("class", SEARCH_TEXT_CLASS)], vec![text(placeholder)]),
            ],
        ));
    }

    for (index, item) in items.iter().enumerate() {
        let is_active = !item.disabled.0 && active == Some(index);
        let mut attrs: Vec<(String, String)> = vec![("class".to_string(), ITEM_CLASS.to_string())];
        if let Some(attr) = Active(is_active).attr() {
            attrs.push(attr);
        }
        if let Some(attr) = item.disabled.attr() {
            attrs.push(attr);
        }
        children.push(el_owned("div", attrs, vec![text(item.label)]));
    }

    el_owned("div", vec![("class".to_string(), class)], children)
}
