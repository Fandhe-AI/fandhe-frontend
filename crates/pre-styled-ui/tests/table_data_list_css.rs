//! styled Table / DataList（イシュー #767）の決定的 CSS 出力の固定テスト。
//!
//! `crates/pre-styled-ui/tests/form_controls_css.rs`（Input/Textarea/
//! NativeSelect、#737）は `css()` の全文をバイト単位で固定する golden
//! fixture 形式だが、本ファイルは breadcrumb（`src/breadcrumb.rs` の
//! `stylesheet_is_deterministic_and_contains_variant_selectors_and_tokens`）
//! と同型の「決定性 + 重要規則の存在確認」形式を採る。Table の recipe は
//! `variant`/`size`/`striped` の 3 軸が絡み合い、全文バイト一致固定は軸を
//! 1 つ調整するだけで無関係な差分が大量発生し、レビュー・保守コストに見合わ
//! ないと判断したため（決定性・fail-closed の不変条件はどちらの形式でも
//! 等しく検証できる）。
//!
//! 固定する観点:
//! 1. `css()` の呼び出しは決定的（複数回呼んでもバイト一致）。
//! 2. striped の `:nth-child(even)` 状態規則・`--fandhe-table-stripe-bg`
//!    custom property が出力に含まれる。
//! 3. size 3 値（sm/md/lg）の custom property 切り替えクラスが出力に含まれる。
//! 4. `variant`（line/outline）のクラスセレクタが出力に含まれる。
//! 5. `<` を一切含まない（`<style>` RAWTEXT 脱出防止、`StyleSheet::push_css`
//!    が要求する不変条件と同型）。
//! 6. recipe が生成するセレクタが、実際に本クレートがレンダリングする
//!    `data-scope`/`data-part` 属性と一致する（`recipe_css.rs` の
//!    `base_selectors_match_actual_headless_markup` と同型の接続照合）。
//! 7. `sticky_header`（イシュー #1571）の `false`/`true` クラスセレクタ・
//!    custom property が出力に含まれる。

use fandhe_frontend_core::render;
use fandhe_frontend_pre_styled_ui::data_list::{self, DataListOrientation, DataListProps};
use fandhe_frontend_pre_styled_ui::table::{self, TableProps, TableVariant};
use fandhe_frontend_pre_styled_ui::Size;

#[test]
fn table_css_is_deterministic() {
    assert_eq!(table::css(), table::css());
}

#[test]
fn table_css_contains_striped_state_rule_and_custom_property() {
    let css = table::css();
    assert!(css.contains(r#"[data-scope="table"][data-part="row"]:nth-child(even) {"#));
    assert!(css.contains("background: var(--fandhe-table-stripe-bg, transparent);"));
    assert!(css.contains("--fandhe-table-stripe-bg: var(--fandhe-color-bg-subtle);"));
    assert!(css.contains("--fandhe-table-stripe-bg: transparent;"));
}

#[test]
fn table_css_contains_size_custom_property_variants() {
    let css = table::css();
    // イシュー #1572: 生値リテラルから `--fandhe-space-*` トークン参照へ
    // 置換した（値そのものは #1571/#1681 から不変）。Xs/Xl を含む 5 段全体を
    // 固定する。
    for (class, padding) in [
        (
            "fd-table--size-xs",
            "var(--fandhe-space-1) var(--fandhe-space-2)",
        ),
        (
            "fd-table--size-sm",
            "var(--fandhe-space-2) var(--fandhe-space-3)",
        ),
        (
            "fd-table--size-md",
            "var(--fandhe-space-3) var(--fandhe-space-4)",
        ),
        (
            "fd-table--size-lg",
            "var(--fandhe-space-4) var(--fandhe-space-5)",
        ),
        (
            "fd-table--size-xl",
            "var(--fandhe-space-5) var(--fandhe-space-6)",
        ),
    ] {
        let selector = format!(r#"[data-scope="table"][data-part="root"].{class} {{"#);
        assert!(
            css.contains(&selector),
            "missing selector: {selector}\n{css}"
        );
        assert!(
            css.contains(&format!("--fandhe-table-cell-padding: {padding};")),
            "missing padding decl for {class}\n{css}"
        );
    }
}

#[test]
fn table_css_contains_variant_selectors() {
    let css = table::css();
    assert!(css.contains(r#"[data-scope="table"][data-part="root"].fd-table--variant-line {"#));
    assert!(css.contains(r#"[data-scope="table"][data-part="root"].fd-table--variant-outline {"#));
}

/// `root` は `border-collapse: separate`（イシュー #767 PR #811 の Outline
/// variant 角丸修正で導入）であり、CSS 表モデル仕様上 `row`（`tr`）への
/// border 指定はブラウザに無視される。Line variant の行区切り線
/// （`--fandhe-table-row-border`）は `row` ではなく `cell`（`td`）側の
/// `border-bottom` として出力されなければならない（PR #811 Bugbot 追加指摘:
/// "Line row borders ignored" の回帰防止）。
#[test]
fn table_css_puts_row_border_on_cell_not_row() {
    let css = table::css();
    assert!(
        css.contains(
            r#"[data-scope="table"][data-part="cell"] {
  padding: var(--fandhe-table-cell-padding, var(--fandhe-space-3) var(--fandhe-space-4));
  font-size: var(--fandhe-table-font-size, var(--fandhe-font-font-size-sm));
  border-bottom: var(--fandhe-table-row-border, none);
  border-top: var(--fandhe-table-cell-border-top, none);
  font-variant-numeric: tabular-nums;
}"#
        ),
        "cell base rule must set border-bottom via --fandhe-table-row-border\n{css}"
    );
    // `row`（`tr`）の base 規則自体が出力されないこと（`separate` モデルでは
    // 無効なため、ここに書いても意味がない不変条件を CSS 出力レベルで固定する）。
    // ただし `row` の `:last-child` 状態規則（イシュー #1572）はこの
    // アサーションの対象外（`[data-part="row"] {` に完全一致せず
    // `[data-part="row"]:last-child {` のため文字列一致しない）。
    assert!(!css.contains(r#"[data-scope="table"][data-part="row"] {"#));
}

/// Outline variant は `border-radius` に加え `clip-path` による角丸クリップを
/// `root` に持たなければならない。`border-collapse: separate` では
/// `column-header` の不透明背景・striped 偶数行の背景が `root` の角丸に
/// 追従してクリップされず、矩形の角のまま描画される（イシュー #767
/// PR #811 Bugbot 指摘: "Outline corners not clipped" の回帰防止）。
///
/// クリップ手段には `overflow: hidden` ではなく `clip-path` を使う
/// （イシュー #1571 codex-review P1 是正: `overflow` を `visible` 以外に
/// すると `root` が `position: sticky`（`sticky_header`）の最も近い
/// スクロール祖先になってしまい、`root` 自身はスクロールしないため
/// ページスクロールへ `sticky_header` が追従しなくなる契約違反を
/// 起こしていた。`clip-path` は `overflow` を変更しないため
/// スクロールコンテナ化を起こさず、`sticky_header` と共存できる）。
#[test]
fn table_css_outline_variant_clips_descendants_to_border_radius() {
    let css = table::css();
    let outline_rule_start = css
        .find(r#"[data-scope="table"][data-part="root"].fd-table--variant-outline {"#)
        .expect("Outline variant の root 規則が css() 出力に存在すること");
    let outline_rule_end = css[outline_rule_start..]
        .find('}')
        .map(|offset| outline_rule_start + offset)
        .expect("Outline variant の root 規則が `}` で閉じられていること");
    let outline_rule = &css[outline_rule_start..outline_rule_end];
    assert!(
        outline_rule.contains("border-radius:"),
        "Outline variant の root 規則に border-radius が存在すること\n{outline_rule}"
    );
    assert!(
        outline_rule.contains("clip-path: inset(0 round var(--fandhe-radius-lg));"),
        "Outline variant の root 規則に clip-path による角丸クリップがなく、\
         column-header/striped 偶数行の背景が角丸からはみ出す\n{outline_rule}"
    );
    assert!(
        !outline_rule.contains("overflow:"),
        "Outline variant の root 規則が overflow を宣言していないこと\
         （sticky_header のスクロール祖先化を防ぐ不変条件）\n{outline_rule}"
    );
}

/// イシュー #1571: `sticky_header` variant（`false`/`true` 両側）のクラス
/// セレクタ・root スコープ custom property・`column-header` 側の消費規則が
/// `css()` 出力に含まれることを固定する。
#[test]
fn table_css_contains_sticky_header_variants() {
    let css = table::css();
    assert!(
        css.contains(r#"[data-scope="table"][data-part="root"].fd-table--sticky-header-false {"#)
    );
    assert!(
        css.contains(r#"[data-scope="table"][data-part="root"].fd-table--sticky-header-true {"#)
    );
    assert!(css.contains("--fandhe-table-header-position: static;"));
    assert!(css.contains("--fandhe-table-header-position: sticky;"));
    assert!(css.contains("--fandhe-table-sticky-offset: 0;"));
    assert!(css.contains("position: var(--fandhe-table-header-position, static);"));
    assert!(css.contains("top: var(--fandhe-table-sticky-offset, 0);"));
}

/// イシュー #1571: `column-header`（`th`）base 規則が chakra-ui / Radix
/// Themes 基準の 1px 罫線・medium 太さになっていることを固定する（旧 2px
/// semibold からの是正）。
#[test]
fn table_css_column_header_uses_one_pixel_border_and_medium_weight() {
    let css = table::css();
    // イシュー #1572: リテラルから `--fandhe-table-header-border`/
    // `--fandhe-table-header-bg` custom property 経由（既定値は 1px 罫線・
    // 通常背景のまま）へ変更した（`Outline` の実装、上記モジュール doc
    // 「`Outline` の実装」節参照）。
    assert!(css.contains(
        "border-bottom: var(--fandhe-table-header-border, 1px solid var(--fandhe-color-border));"
    ));
    assert!(css.contains("background: var(--fandhe-table-header-bg, var(--fandhe-color-bg));"));
    assert!(!css.contains("2px solid var(--fandhe-color-border-muted)"));
    let column_header_rule_start = css
        .find(r#"[data-scope="table"][data-part="column-header"] {"#)
        .expect("column-header base 規則が css() 出力に存在すること");
    let column_header_rule_end = css[column_header_rule_start..]
        .find('}')
        .map(|offset| column_header_rule_start + offset)
        .expect("column-header base 規則が `}` で閉じられていること");
    let column_header_rule = &css[column_header_rule_start..column_header_rule_end];
    assert!(column_header_rule.contains("font-weight: var(--fandhe-font-font-weight-medium);"));
    assert!(column_header_rule.contains("color: var(--fandhe-color-fg);"));
    assert!(column_header_rule.contains("font-variant-numeric: tabular-nums;"));
}

/// イシュー #1572: `footer`（`tfoot`）base 規則が medium 太さを持ち、
/// 実際に描画される `border-*` プロパティ自体は持たないことを固定する
/// （`separate` border モデル下では `tfoot` への border 指定が無効な
/// ため、`cell` の PR #811 型不変条件と対をなす）。`--fandhe-table-cell-
/// border-top` の委譲は PR #1844 是正で `footer` base から `css()` 追記の
/// `:first-child` 限定規則へ移した（複数行 `tfoot` での区切り線重複
/// バグ是正、`table_css_footer_first_row_delegates_border_top` 参照）。
#[test]
fn table_css_footer_has_medium_weight_and_no_direct_border_declaration() {
    let css = table::css();
    let footer_rule_start = css
        .find(r#"[data-scope="table"][data-part="footer"] {"#)
        .expect("footer base 規則が css() 出力に存在すること");
    let footer_rule_end = css[footer_rule_start..]
        .find('}')
        .map(|offset| footer_rule_start + offset)
        .expect("footer base 規則が `}` で閉じられていること");
    let footer_rule = &css[footer_rule_start..footer_rule_end];
    assert!(footer_rule.contains("font-weight: var(--fandhe-font-font-weight-medium);"));
    assert!(!footer_rule.contains("\n  border-top:"));
    assert!(!footer_rule.contains("\n  border-bottom:"));
    assert!(!footer_rule.contains("\n  border:"));
}

/// イシュー #1572 PR #1844 Cursor Bugbot Low severity 是正: `footer` の
/// 最初の行だけが `--fandhe-table-cell-border-top` を設定することを
/// 固定する（複数行 `tfoot` で境界線が全行に重複しない不変条件）。
#[test]
fn table_css_footer_first_row_delegates_border_top() {
    let css = table::css();
    assert!(css.contains(
        r#"[data-scope="table"][data-part="footer"] [data-scope="table"][data-part="row"]:first-child {"#
    ));
    assert!(
        css.contains("--fandhe-table-cell-border-top: var(--fandhe-table-footer-border, none);")
    );
}

/// イシュー #1572: `Outline` variant の root スコープ custom property が
/// 内側行罫線・最終行罫線なし・muted ヘッダー背景・footer 上罫線を宣言する
/// ことを固定する（`Line` と対比した意匠是正、上記モジュール doc
/// 「`Outline` の実装」節参照）。
#[test]
fn table_css_outline_variant_declares_row_header_and_footer_custom_properties() {
    let css = table::css();
    let outline_rule_start = css
        .find(r#"[data-scope="table"][data-part="root"].fd-table--variant-outline {"#)
        .expect("Outline variant の root 規則が css() 出力に存在すること");
    let outline_rule_end = css[outline_rule_start..]
        .find('}')
        .map(|offset| outline_rule_start + offset)
        .expect("Outline variant の root 規則が `}` で閉じられていること");
    let outline_rule = &css[outline_rule_start..outline_rule_end];
    assert!(
        outline_rule
            .contains("--fandhe-table-row-border: 1px solid var(--fandhe-color-border-muted);"),
        "Outline は内側の行罫線を持つ（chakra outline / Radix surface 相当）\n{outline_rule}"
    );
    assert!(
        outline_rule.contains("--fandhe-table-last-row-border: none;"),
        "Outline は最終行の罫線を外枠と二重にしない\n{outline_rule}"
    );
    assert!(
        outline_rule.contains("--fandhe-table-header-bg: var(--fandhe-color-bg-muted);"),
        "Outline はヘッダー背景に muted トークンを使う（chakra bg.muted 相当）\n{outline_rule}"
    );
    assert!(
        outline_rule
            .contains("--fandhe-table-footer-border: 1px solid var(--fandhe-color-border-muted);"),
        "Outline は tfoot 上罫線を持つ（chakra outline の tfoot border-top 相当）\n{outline_rule}"
    );
}

/// イシュー #1572: Line variant は最終行にも罫線を維持し（chakra line）、
/// footer には区切り線を持たないことを固定する（`Outline` との対比）。
#[test]
fn table_css_line_variant_keeps_last_row_border_and_no_footer_border() {
    let css = table::css();
    let line_rule_start = css
        .find(r#"[data-scope="table"][data-part="root"].fd-table--variant-line {"#)
        .expect("Line variant の root 規則が css() 出力に存在すること");
    let line_rule_end = css[line_rule_start..]
        .find('}')
        .map(|offset| line_rule_start + offset)
        .expect("Line variant の root 規則が `}` で閉じられていること");
    let line_rule = &css[line_rule_start..line_rule_end];
    assert!(line_rule
        .contains("--fandhe-table-last-row-border: 1px solid var(--fandhe-color-border-muted);"));
    assert!(line_rule.contains("--fandhe-table-footer-border: none;"));
}

/// イシュー #1572 PR #1844 codex-review P1 是正: 表全体で本当に最後の行
/// （`tfoot` があればその最終行、無ければ最後の `tbody` の最終行）だけが
/// `--fandhe-table-row-border` を `--fandhe-table-last-row-border` で
/// 上書きすることを固定する。複数 `tbody`/`tfoot` の中間グループの
/// 最終行が誤って一致しないことを合わせて検証する。
#[test]
fn table_css_row_last_child_overrides_row_border() {
    let css = table::css();
    assert!(css.contains(
        r#"[data-scope="table"][data-part="footer"] [data-scope="table"][data-part="row"]:last-child {"#
    ));
    assert!(css.contains(
        r#"[data-scope="table"][data-part="body"]:last-of-type:not(:has(~ [data-scope="table"][data-part="footer"])) [data-scope="table"][data-part="row"]:last-child {"#
    ));
    assert!(css.contains("--fandhe-table-row-border: var(--fandhe-table-last-row-border, none);"));
}

/// イシュー #1572: `cell` base が `border-top` として
/// `--fandhe-table-cell-border-top` を消費することを固定する（`footer` →
/// `cell` の委譲経路）。
#[test]
fn table_css_cell_consumes_border_top_custom_property() {
    let css = table::css();
    assert!(css.contains("border-top: var(--fandhe-table-cell-border-top, none);"));
}

/// イシュー #1572: `scroll-area` slot（chakra `Table.ScrollArea` 相当）の
/// base 規則を固定する（上記モジュール doc「スクロール枠の実装」節参照）。
#[test]
fn table_css_contains_scroll_area_slot() {
    let css = table::css();
    assert!(css.contains(r#"[data-scope="table"][data-part="scroll-area"] {"#));
    assert!(css.contains("overflow: auto;"));
    assert!(css.contains("max-height: var(--fandhe-table-scroll-area-max-height, none);"));
    assert!(css.contains("scrollbar-width: thin;"));
}

/// イシュー #1572: `caption` base 規則が chakra-ui 基準（`xs` + `medium`）へ
/// 更新され、padding が `--fandhe-space-3` トークンを使うことを固定する。
#[test]
fn table_css_caption_uses_xs_medium_and_space_token() {
    let css = table::css();
    let caption_rule_start = css
        .find(r#"[data-scope="table"][data-part="caption"] {"#)
        .expect("caption base 規則が css() 出力に存在すること");
    let caption_rule_end = css[caption_rule_start..]
        .find('}')
        .map(|offset| caption_rule_start + offset)
        .expect("caption base 規則が `}` で閉じられていること");
    let caption_rule = &css[caption_rule_start..caption_rule_end];
    assert!(caption_rule.contains("padding: var(--fandhe-space-3) 0;"));
    assert!(caption_rule.contains("font-size: var(--fandhe-font-font-size-xs);"));
    assert!(caption_rule.contains("font-weight: var(--fandhe-font-font-weight-medium);"));
}

#[test]
fn table_css_never_contains_style_breakout_sequences() {
    let css = table::css();
    assert!(!css.contains("</style"));
    assert!(!css.contains('<'));
}

#[test]
fn table_recipe_selectors_match_actual_rendered_markup() {
    let html = render(&table::root(
        TableProps {
            variant: TableVariant::Outline,
            size: Size::Lg,
            striped: true,
            sticky_header: true,
        },
        vec![],
        vec![],
    ));
    assert!(html.contains(r#"data-scope="table" data-part="root""#));
    assert!(html.contains("fd-table--variant-outline"));
    assert!(html.contains("fd-table--size-lg"));
    assert!(html.contains("fd-table--striped-true"));
    assert!(html.contains("fd-table--sticky-header-true"));

    let cell_html = render(&table::cell(vec![], vec![]));
    assert!(cell_html.starts_with(r#"<td data-scope="table" data-part="cell""#));

    let row_html = render(&table::row(vec![], vec![]));
    assert!(row_html.starts_with(r#"<tr data-scope="table" data-part="row""#));

    // イシュー #1572: `scroll_area`（chakra `Table.ScrollArea` 相当）。
    let scroll_area_html = render(&table::scroll_area(vec![], vec![]));
    assert!(scroll_area_html.starts_with(r#"<div data-scope="table" data-part="scroll-area""#));
}

#[test]
fn data_list_css_is_deterministic() {
    assert_eq!(data_list::css(), data_list::css());
}

#[test]
fn data_list_css_contains_orientation_custom_properties() {
    let css = data_list::css();
    assert!(css.contains(
        r#"[data-scope="data-list"][data-part="root"].fd-data-list--orientation-vertical {"#
    ));
    assert!(css.contains(
        r#"[data-scope="data-list"][data-part="root"].fd-data-list--orientation-horizontal {"#
    ));
    assert!(css.contains("--fandhe-data-list-item-flex-direction: column;"));
    assert!(css.contains("--fandhe-data-list-item-flex-direction: row;"));
}

/// イシュー #1559: `variant`（subtle/bold）・`size`（xs〜xl）の 2 軸を
/// 新設した。両軸のクラスセレクタが `css()` 出力に存在することを固定する
/// （table の `variant`/`size` 網羅アサーションと同型）。
#[test]
fn data_list_css_contains_variant_and_size_selectors() {
    let css = data_list::css();
    for class in [
        "fd-data-list--variant-subtle",
        "fd-data-list--variant-bold",
        "fd-data-list--size-xs",
        "fd-data-list--size-sm",
        "fd-data-list--size-md",
        "fd-data-list--size-lg",
        "fd-data-list--size-xl",
    ] {
        let selector = format!(r#"[data-scope="data-list"][data-part="root"].{class} {{"#);
        assert!(
            css.contains(&selector),
            "missing selector: {selector}\n{css}"
        );
    }
}

/// Horizontal 時のラベル最小幅（chakra `minW="120px"` / Radix Themes
/// `Label` の `minWidth: 120px` に一致、モジュール doc「参考サイト基準へ
/// の調整」参照）・variant ごとのラベル太字・root/item-label/item-value が
/// 共有する font-size custom property が出力へ含まれることを固定する。
#[test]
fn data_list_css_contains_label_min_width_and_font_tokens() {
    let css = data_list::css();
    assert!(css.contains("--fandhe-data-list-label-min-width: 7.5rem;"));
    assert!(css
        .contains("--fandhe-data-list-label-font-weight: var(--fandhe-font-font-weight-medium);"));
    assert!(css.contains(
        "font-size: var(--fandhe-data-list-font-size, var(--fandhe-font-font-size-sm));"
    ));
}

#[test]
fn data_list_css_never_contains_style_breakout_sequences() {
    let css = data_list::css();
    assert!(!css.contains("</style"));
    assert!(!css.contains('<'));
}

#[test]
fn data_list_recipe_selectors_match_actual_rendered_markup() {
    let html = render(&data_list::root(
        DataListProps {
            orientation: DataListOrientation::Horizontal,
            ..DataListProps::default()
        },
        vec![],
        vec![],
    ));
    assert!(html.contains(r#"data-scope="data-list" data-part="root""#));
    assert!(html.contains("fd-data-list--orientation-horizontal"));

    let item_html = render(&data_list::item(vec![], vec![]));
    assert!(item_html.starts_with(r#"<div data-scope="data-list" data-part="item""#));
}
