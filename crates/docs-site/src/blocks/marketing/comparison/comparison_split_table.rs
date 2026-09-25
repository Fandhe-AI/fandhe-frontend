//! `comparison-split-table` block（イシュー #2824。親トラッキング #2730
//! 「Blocks 目的別パーツ拡充ツリー」配下、対応表 ID R0437（1 件）を構造の
//! 参照元とする合成例。見出しブロックの下に自社/他社を比べる表を続ける
//! 構成ではなく、**左に見出しブロック（タグライン・見出し・説明・
//! アクション）・右に比較表**を並べる「見出し左 + 比較表右」レイアウト）。
//! 取得手段・ファイル名・内部コンポーネント識別子は記載しない
//! （`docs/design/motion-reference-adoption-policy.md` §9 と同じライセンス
//! 上の転記制限）。
//!
//! **Marketing / Comparison カテゴリで 3 番目の block**（`super`
//! （`comparison/mod.rs`）参照）。
//!
//! # 使用部品
//!
//! `heading` / `text` / `badge` / `button` / `table` / `icon` の 6 部品を
//! 合成する（[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//!
//! # レイアウト構造（`## Demo` 見出しに続く 1 インスタンス）
//!
//! 既定（狭い幅）は縦積みで、上から「見出しブロック（タグライン `badge` +
//! セクション見出し `heading H3` + リード文 `text` + アクション `button` ×
//! 2）」→「比較表（`table::scroll_area` で包んだ `table::root`）」の順に
//! 並べる。`>= 64rem`（lg）で 2 カラム grid（左: 見出しブロック / 右: 比較
//! 表）へ切り替える（下記「レスポンシブ」節参照）。
//!
//! 比較表は列見出しを「機能」「自社」「他社」の 3 列とし、本文行は
//! [`ROWS`] の件数だけ並べる。各セルは [`CellValue::Yes`]/[`CellValue::No`]
//! （自作の幾何アイコン + アクセシブルネーム）と [`CellValue::Text`]
//! （テキスト値）が混在する。機能名セルには機能名の下に補足説明を常時
//! 表示する（下記「補足説明の表示方法」節参照）。
//!
//! # レスポンシブ（`LAYOUT_CSS` 参照）
//!
//! 既定は 1 カラム grid（縦積み）。`>= 64rem`（lg）で 2 カラム grid
//! （左: 見出しブロック `minmax(0, 1fr)` / 右: 比較表 `minmax(0, 1.5fr)`）
//! へ切り替える。ブレークポイントのリテラル rem 値は
//! [`fandhe_frontend_pre_styled_ui`] の `Breakpoint`（lg=1024px）に合わせて
//! いる（`@media` 内ではトークン変数を使えないため）。狭い幅では比較表を
//! [`fandhe_frontend_pre_styled_ui::table::scroll_area`] で包み、`min-width`
//! を明示した `table::root` を横スクロールさせる（`.blocks-demo` 自身の
//! `overflow-x` には頼らない）。
//!
//! # 補足説明の表示方法（参照元との意図的な差分）
//!
//! 参照元は機能名の隣に情報アイコン + ツールチップで補足説明を出すが、
//! docs サイトは無 JS のためホバー起動のツールチップを実装できない
//! （`crate::blocks` モジュール doc「`<form>` を使わない」節と同じ無 JS
//! 制約）。本 block では補足説明を機能名セル内の常時表示テキスト
//! （[`styled_text::text`]、[`TextVariant::Muted`] + [`TextSize::Sm`]）へ
//! 置き換える。
//!
//! # アイコンは独自の抽象図形のみ・アクセシブルネームを持つ
//!
//! 参照元の SVG path 文字列は一切コピーしない。[`icon_check`]/
//! [`icon_cross`] は `comparison_feature_rows::glyph_circle` 等と同型の
//! 判断で、単純なチェックマーク・バツ印を自前の `d`（`fill="none"` +
//! `stroke="currentColor"`）で描く。色だけで対応/非対応を区別しない
//! （WCAG 1.4.1）ため、形も変えたうえで [`IconProps::label`] へ
//! `Some("対応")`/`Some("非対応")` を渡し `role="img"` + `aria-label` を
//! 付与する（`icon::icon` の契約、[`crate::blocks::icon`] ではなく
//! [`fandhe_frontend_pre_styled_ui::icon`] を直接使う）。強調色は
//! [`props::Primary`] のような部品固有の新型を新設せず、CSS フック
//! （[`YES_ATTR`]/[`NO_ATTR`]）で `--fandhe-color-*` トークンを塗り分ける。
//! `icon::icon` は呼び出し側 `attrs` の `class` を `drop_class_attr` で
//! 黙って除去する契約（下記「CSS フックの選び方」節参照）のため、フックは
//! `class` ではなく `data-*` 属性で渡す（`class` で渡すとスタイルが一切
//! 適用されない）。
//!
//! 塗り分けの色トークンは `--fandhe-color-accent-fg`（アクセント色の
//! 背景に載せるコントラスト色）ではなく `--fandhe-color-accent`
//! （`crate::theme` の `LARGE_TEXT_UI_PAIRS` で `bg`/`bg-muted` 背景上の
//! 大字・UI 要素として WCAG 3:1 を満たすと検証済みのトークン）を使う。
//! 通常のセル背景（`bg`/`bg-muted`）の上に直接アイコンを描くため、
//! `accent-fg` を使うと両テーマでチェックマークが背景と同化して見えなく
//! なる（`docs/design/color-token-system.md` 参照）。
//!
//! # 自社・他社ラベルは `badge` で表す
//!
//! 列見出し（自社/他社）は [`fandhe_frontend_pre_styled_ui::badge::badge`]
//! で表現し、`props.rs` へ新型を追加しない。自社側は
//! [`BadgeVariant::Solid`] + [`ColorPalette::Accent`]（強調）、他社側は
//! [`BadgeVariant::Outline`] + [`ColorPalette::Neutral`]（中立）で視覚的に
//! 区別する（`comparison_feature_rows` と同じ判断）。実在の企業名・製品名・
//! 競合名は使わず、中立的な「自社」「他社」ラベルに固定する。
//!
//! # `table` の合成方法
//!
//! [`fandhe_frontend_pre_styled_ui::table`] はコンビニ関数を提供せず各
//! パーツを個別に呼び出す契約（`crate::table` モジュール doc「コンビニ
//! 関数」節参照）のため、`root`/`header`/`body`/`row`/`column_header`/
//! `cell`/`scroll_area` を個別に組み立てる。列見出し・セル整列には
//! `data-align`（`table` モジュール doc「`data-align` セル整列」節の共有
//! 語彙）を使い、自社/他社列を `center` 揃えにする。
//!
//! 行見出し（`th scope="row"`）に対応するパーツは `table` に存在しないが、
//! `crate::table` モジュール doc の CSS 出力契約（`[data-scope="table"]
//! [data-part="<slot>"]` はタグ名に依存しない属性セレクタ、同モジュール
//! doc「セキュリティ不変条件」節近傍の recipe 実装参照）により、`cell`
//! スロットのスタイルは要素のタグ名を問わず適用される。この性質を使い、
//! 機能名セルは `table::cell`（`<td>`）を呼ぶ代わりに
//! [`fandhe_frontend_core::el`] で直接 `<th scope="row"
//! data-scope="table" data-part="cell">` を組み立て（[`feature_cell`]
//! 参照）、`cell` と同一の見た目を保ったままスクリーンリーダーが値セルを
//! 対応する機能名へ関連付けられるようにする（`pre-styled-ui` 側へ新規
//! パーツを追加せずに済む、本 block 側の安全なノード API 構築による対応）。
//!
//! # `id` 属性を出力しない理由
//!
//! `id`/`aria-labelledby` を持つ部品を使わないため、宙に浮いた ARIA 参照や
//! id 重複を防ぐための出力自体を行わない（`comparison_feature_rows` と
//! 同じ判断）。横スクロール枠のキーボード到達可能性は
//! `role="region"` + `aria-label` + `tabindex="0"`（[`scroll_area`] の
//! `attrs` 経由）で確保し、id 参照を経由しない。
//!
//! # `text` の名前衝突
//!
//! `fandhe_frontend_pre_styled_ui::text::text`（`<p>` を組み立てる styled
//! パート関数）と `fandhe_frontend_core::text`（テキストノード生成関数）が
//! 同名のため、styled 側を `styled_text` として取り込む
//! （`comparison_feature_rows` と同じ回避方法）。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `heading::heading` / `text::text` / `badge::badge` / `button::button` /
//! `table::root` / `icon::icon` はいずれも `drop_class_attr` により
//! 呼び出し側 `attrs` の `class` を黙って除去する契約を持つため、Demo
//! 固有のスタイルフックは `data-blocks-comparison-split-table-*` 属性で
//! 渡す。素の `div` には `.blocks-comparison-split-table-*` クラス
//! セレクタを使う。
//!
//! # `<form>` を持たない・送信先を持たないボタン
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節に従い、本 Demo は
//! フォーム・状態機械を持たない静的な合成例である。2 個の `button` は
//! いずれも既定の `type="button"` のまま用い、送信先・クリック配線を
//! 持たない。文言はすべて架空のもの（実在の製品・企業名・PII を含まない）
//! である。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, el, span, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps, BadgeVariant};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::table::{self, TableProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::ColorPalette;

/// 対応セルの CSS フック（色だけに頼らずチェックマークの形でも区別する。
/// 上記モジュール doc「アイコンは独自の抽象図形のみ・アクセシブルネームを
/// 持つ」節参照）。`icon::icon` は呼び出し側 `attrs` の `class` を
/// `drop_class_attr` で除去する契約のため、`class` ではなく値なしの
/// `data-*` 属性で渡す（`class` で渡すと出力から消え CSS が一致しない）。
const YES_ATTR: &str = "data-blocks-comparison-split-table-yes";
/// 非対応セルの CSS フック（上記と対になる、バツ印用）。
const NO_ATTR: &str = "data-blocks-comparison-split-table-no";

/// チェックマークのみの自作アイコン（`comparison_feature_rows::glyph_circle`
/// と同型の対処。参照元の SVG path はコピーしない）。
fn icon_check() -> Node {
    icon(
        &IconProps {
            label: Some("対応"),
            ..IconProps::default()
        },
        vec![(YES_ATTR, "")],
        vec![el(
            "path",
            vec![
                ("d", "M20 6L9 17l-5-5"),
                ("fill", "none"),
                ("stroke", "currentColor"),
                ("stroke-width", "2"),
                ("stroke-linecap", "round"),
                ("stroke-linejoin", "round"),
            ],
            vec![],
        )],
    )
}

/// バツ印のみの自作アイコン（[`icon_check`] と対になる、非対応セル用）。
fn icon_cross() -> Node {
    icon(
        &IconProps {
            label: Some("非対応"),
            ..IconProps::default()
        },
        vec![(NO_ATTR, "")],
        vec![el(
            "path",
            vec![
                ("d", "M6 6l12 12M6 18L18 6"),
                ("fill", "none"),
                ("stroke", "currentColor"),
                ("stroke-width", "2"),
                ("stroke-linecap", "round"),
                ("stroke-linejoin", "round"),
            ],
            vec![],
        )],
    )
}

/// セル値（対応/非対応/テキスト値の混在、上記モジュール doc「レイアウト
/// 構造」節参照）。
enum CellValue {
    /// チェックマーク（[`icon_check`]、アクセシブルネーム「対応」）。
    Yes,
    /// バツ印（[`icon_cross`]、アクセシブルネーム「非対応」）。
    No,
    /// テキスト値（既定エスケープ経由で出力）。
    Text(&'static str),
}

/// 比較行 1 件分のダミーデータ（架空、実在の製品・企業とは無関係）。
struct Row {
    /// 機能名。
    name: &'static str,
    /// 補足説明（機能名の下へ常時表示、上記モジュール doc「補足説明の
    /// 表示方法」節参照）。
    note: &'static str,
    /// 自社列の値。
    ours: CellValue,
    /// 他社列の値。
    theirs: CellValue,
}

/// 比較行一覧（架空、5 行固定。検索インデックスの肥大を抑えるため件数を
/// 増やさない。Yes/No が混在する並びにする）。
const ROWS: [Row; 5] = [
    Row {
        name: "同時編集の人数",
        note: "1 ワークスペースで同時に編集できる人数の上限です。",
        ours: CellValue::Text("無制限"),
        theirs: CellValue::Text("10 人まで"),
    },
    Row {
        name: "権限管理",
        note: "メンバーごとに閲覧・編集・管理者の権限を分けられます。",
        ours: CellValue::Yes,
        theirs: CellValue::No,
    },
    Row {
        name: "監査ログ",
        note: "誰がいつ何を変更したかを記録し、いつでも確認できます。",
        ours: CellValue::Yes,
        theirs: CellValue::No,
    },
    Row {
        name: "共同編集",
        note: "複数人が同じドキュメントをリアルタイムに編集できます。",
        ours: CellValue::Yes,
        theirs: CellValue::Yes,
    },
    Row {
        name: "データの書き出し",
        note: "保存済みデータをまとめて書き出せます。",
        ours: CellValue::Text("全形式に対応"),
        theirs: CellValue::Text("CSV のみ"),
    },
];

/// 見出しブロック（タグライン `badge` + セクション見出し + リード文 +
/// アクション `button` × 2、上記モジュール doc「レイアウト構造」節参照）。
fn intro() -> Node {
    div(
        vec![("class", "blocks-comparison-split-table-intro")],
        vec![
            badge::badge(&BadgeProps::default(), vec![], vec![text("比較")]),
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    weight: HeadingWeight::Bold,
                },
                vec![],
                vec![text("選ばれる理由を表で確認")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(
                    "主要な機能を自社・他社で並べて比較できます。まずは無料でお試しください。",
                )],
            ),
            div(
                vec![("class", "blocks-comparison-split-table-actions")],
                vec![
                    button::button(&ButtonProps::default(), vec![], vec![text("無料で試す")]),
                    button::button(
                        &ButtonProps {
                            variant: ButtonVariant::Outline,
                            ..ButtonProps::default()
                        },
                        vec![],
                        vec![text("プランを見る")],
                    ),
                ],
            ),
        ],
    )
}

/// セル値 1 件を [`Node`] へ変換する（[`CellValue::Text`] は既定エスケープ
/// を経由する [`fandhe_frontend_core::text`] で出力する）。
fn value_node(value: &CellValue) -> Node {
    match value {
        CellValue::Yes => icon_check(),
        CellValue::No => icon_cross(),
        CellValue::Text(value) => span(
            vec![("data-blocks-comparison-split-table-value", "")],
            vec![text(*value)],
        ),
    }
}

/// 機能名セル（行見出し）を `<th scope="row">` として組み立てる
/// （上記モジュール doc「`table` の合成方法」節参照）。`table::cell`
/// （`<td>`）は呼ばず、`fandhe_frontend_pre_styled_ui::table` の CSS が
/// `[data-scope="table"][data-part="cell"]`（タグ名非依存の属性セレクタ）で
/// 出力されることを利用して、見た目は既存の `cell` パーツと同一のまま
/// `scope="row"` を持つ `<th>` を直接構築する。これによりスクリーンリーダー
/// が対応/非対応セルへ移動した際、行見出し（機能名）が自動的に読み上げ
/// られる（`table::column_header` は `scope="col"` を固定して呼び出し側の
/// `scope` 指定を除去するため使えない）。
///
/// 機能名 + 補足説明の縦積みレイアウトは `<th>` 自体ではなく内側の
/// wrapper 要素（`data-blocks-comparison-split-table-feature-inner`）に
/// 持たせる（PR #3189 codex レビュー P1 指摘の是正）。`<th>` へ直接
/// `display: flex` を当てると `table-cell` としての振る舞いが失われ、
/// `fandhe_frontend_pre_styled_ui::table` の `cell` パーツが前提とする
/// 列幅・罫線・padding 等のセル契約（ブラウザの匿名テーブルボックス
/// 補正に依存しない構造）が崩れるため、`<th>` は `table-cell` のまま
/// 維持し、flex は wrapper `<div>` に閉じ込める。
fn feature_cell(row: &Row) -> Node {
    el(
        "th",
        vec![
            ("data-scope", "table"),
            ("data-part", "cell"),
            ("scope", "row"),
            ("data-blocks-comparison-split-table-feature", ""),
        ],
        vec![div(
            vec![("data-blocks-comparison-split-table-feature-inner", "")],
            vec![
                span(
                    vec![("data-blocks-comparison-split-table-feature-name", "")],
                    vec![text(row.name)],
                ),
                styled_text::text(
                    &TextProps {
                        size: TextSize::Sm,
                        variant: TextVariant::Muted,
                        ..TextProps::default()
                    },
                    vec![("data-blocks-comparison-split-table-feature-note", "")],
                    vec![text(row.note)],
                ),
            ],
        )],
    )
}

/// 比較表本体（列見出し「機能」「自社」「他社」+ [`ROWS`] の件数分の本文
/// 行。上記モジュール doc「`table` の合成方法」節参照）。
fn comparison_table() -> Node {
    let header_row = table::row(
        vec![],
        vec![
            table::column_header(vec![], vec![text("機能")]),
            table::column_header(
                vec![("data-align", "center")],
                vec![badge::badge(
                    &BadgeProps {
                        variant: BadgeVariant::Solid,
                        palette: ColorPalette::Accent,
                        ..BadgeProps::default()
                    },
                    vec![],
                    vec![text("自社")],
                )],
            ),
            table::column_header(
                vec![("data-align", "center")],
                vec![badge::badge(
                    &BadgeProps {
                        variant: BadgeVariant::Outline,
                        palette: ColorPalette::Neutral,
                        ..BadgeProps::default()
                    },
                    vec![],
                    vec![text("他社")],
                )],
            ),
        ],
    );

    let body_rows: Vec<Node> = ROWS
        .iter()
        .map(|row| {
            table::row(
                vec![("data-blocks-comparison-split-table-row", "")],
                vec![
                    feature_cell(row),
                    table::cell(vec![("data-align", "center")], vec![value_node(&row.ours)]),
                    table::cell(
                        vec![("data-align", "center")],
                        vec![value_node(&row.theirs)],
                    ),
                ],
            )
        })
        .collect();

    table::scroll_area(
        vec![
            ("data-blocks-comparison-split-table-scroll", ""),
            ("tabindex", "0"),
            ("role", "region"),
            ("aria-label", "機能比較表"),
        ],
        vec![table::root(
            TableProps::default(),
            vec![("data-blocks-comparison-split-table-table", "")],
            vec![
                table::header(vec![], vec![header_row]),
                table::body(vec![], body_rows),
            ],
        )],
    )
}

/// `comparison-split-table` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数（[`crate::blocks`] モジュール doc「静的表示」節）。
pub fn demo() -> Node {
    div(
        vec![("data-blocks-comparison-split-table-root", "")],
        vec![intro(), comparison_table()],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/comparison-split-table/",
    title: "comparison-split-table",
    category: BlockCategory::Comparison,
    rust_source: "crates/docs-site/src/blocks/marketing/comparison/comparison_split_table.rs",
    demo_class: "blocks-comparison-split-table",
    parts: &[
        Part {
            label: "Heading",
            path: "/themes/heading/",
        },
        Part {
            label: "Text",
            path: "/themes/text/",
        },
        Part {
            label: "Badge",
            path: "/themes/badge/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Table",
            path: "/themes/table/",
        },
        Part {
            label: "Icon",
            path: "/themes/icon/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `comparison_split_table` 固有のレイアウト規則（`crate::blocks::
/// LAYOUT_CSS` doc「block 固有 CSS の置き場」節）。モバイルファーストで
/// 既定（狭い幅）は縦積み（見出しブロック → 比較表）、`>= 64rem`（lg）で
/// 2 カラム grid（左: 見出しブロック / 右: 比較表）へ切り替える。狭い幅の
/// 比較表は `table::scroll_area` + `min-width` で横スクロールさせる
/// （上記モジュール doc「レスポンシブ」節参照）。ブレークポイントは
/// `fandhe_frontend_pre_styled_ui` の `Breakpoint`（lg=1024px）に合わせた
/// リテラル rem 値（`@media` 内ではトークン変数を使えないため）。
///
/// `[data-blocks-comparison-split-table-feature]`（`<th scope="row">` 自体）
/// は `display: flex` を持たない。`<th>` は `table-cell` のまま維持し、
/// 機能名 + 補足説明の縦積み flex レイアウトは内側 wrapper
/// `[data-blocks-comparison-split-table-feature-inner]` にのみ適用する
/// （`feature_cell` の rustdoc・PR #3189 codex レビュー P1 指摘の是正）。
const LAYOUT_CSS: &str = "\
[data-blocks-comparison-split-table-root] {\n  display: grid;\n  grid-template-columns: minmax(0, 1fr);\n  gap: var(--fandhe-space-8);\n  width: 100%;\n}\n\
.blocks-comparison-split-table-intro {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n  align-items: flex-start;\n  max-width: 40rem;\n}\n\
.blocks-comparison-split-table-actions {\n  display: flex;\n  flex-wrap: wrap;\n  gap: var(--fandhe-space-2);\n  margin-top: var(--fandhe-space-2);\n}\n\
[data-blocks-comparison-split-table-scroll] {\n  overflow-x: auto;\n  max-width: 100%;\n}\n\
[data-blocks-comparison-split-table-table] {\n  min-width: 32rem;\n}\n\
[data-blocks-comparison-split-table-feature] {\n  text-align: left;\n  font-weight: var(--fandhe-font-font-weight-normal, normal);\n}\n\
[data-blocks-comparison-split-table-feature-inner] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1);\n}\n\
[data-blocks-comparison-split-table-yes] {\n  color: var(--fandhe-color-accent);\n  vertical-align: middle;\n}\n\
[data-blocks-comparison-split-table-no] {\n  color: var(--fandhe-color-fg-muted);\n  vertical-align: middle;\n}\n\
@media (min-width: 64rem) {\n  [data-blocks-comparison-split-table-root] {\n    grid-template-columns: minmax(0, 1fr) minmax(0, 1.5fr);\n    align-items: start;\n  }\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, CellValue, LAYOUT_CSS, ROWS};
    use fandhe_frontend_core::render;

    /// Demo は呼び出しごとに同一の `Node` を返す純関数であること
    /// （`crate::blocks` モジュール doc「静的表示」節）。
    #[test]
    fn demo_is_deterministic() {
        assert_eq!(render(&demo()), render(&demo()));
    }

    /// Demo が期待する 6 種の部品を出力すること。
    #[test]
    fn demo_composes_expected_parts() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"badge\"",
            "data-scope=\"button\"",
            "data-scope=\"table\"",
            "data-scope=\"icon\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
    }

    /// 本文行フックが [`ROWS`] の件数だけ出力されること。
    #[test]
    fn demo_renders_one_body_row_per_entry() {
        let html = render(&demo());
        assert_eq!(
            html.matches("data-blocks-comparison-split-table-row=\"\"")
                .count(),
            ROWS.len(),
            "demo should render exactly {} body rows",
            ROWS.len()
        );
    }

    /// アイコンだけのセル（[`CellValue::Yes`]/[`CellValue::No`]）が
    /// アクセシブルネームを持つこと（上記モジュール doc「アイコンは独自の
    /// 抽象図形のみ・アクセシブルネームを持つ」節）。
    #[test]
    fn icon_cells_have_accessible_names() {
        let html = render(&demo());
        let expected_yes = ROWS
            .iter()
            .flat_map(|row| [&row.ours, &row.theirs])
            .filter(|value| matches!(value, CellValue::Yes))
            .count();
        let expected_no = ROWS
            .iter()
            .flat_map(|row| [&row.ours, &row.theirs])
            .filter(|value| matches!(value, CellValue::No))
            .count();
        assert_eq!(
            html.matches("aria-label=\"対応\"").count(),
            expected_yes,
            "demo should render exactly {expected_yes} accessible names for Yes cells"
        );
        assert_eq!(
            html.matches("aria-label=\"非対応\"").count(),
            expected_no,
            "demo should render exactly {expected_no} accessible names for No cells"
        );
    }

    /// 非対話・安全性の不変条件（`<form>`・`<a`・`href="#"`・`data:` URI・
    /// `id=` 属性・`type="submit"` を持たないこと。モジュール doc「id 属性を
    /// 出力しない理由」節「`<form>` を持たない・送信先を持たないボタン」
    /// 節参照。button 自体は使用するため `<button` は禁止対象に含めない）。
    #[test]
    fn demo_never_contains_forbidden_markup() {
        let html = render(&demo());
        for absent in [
            "<form",
            "<a ",
            "href=\"#\"",
            "src=\"data:",
            "id=\"",
            "type=\"submit\"",
        ] {
            assert!(!html.contains(absent), "demo should never contain {absent}");
        }
        let button_count = html.matches("<button").count();
        assert_eq!(button_count, 2, "demo should render exactly 2 buttons");
        assert_eq!(
            html.matches("type=\"button\"").count(),
            button_count,
            "every <button> should carry type=\"button\""
        );
    }

    /// [`LAYOUT_CSS`] が lg のブレークポイント切り替え・横スクロール枠を
    /// 持つこと。
    #[test]
    fn layout_css_switches_to_two_columns_at_lg() {
        assert!(LAYOUT_CSS.contains("@media (min-width: 64rem)"));
        assert!(LAYOUT_CSS.contains("[data-blocks-comparison-split-table-table]"));
        assert!(LAYOUT_CSS.contains("overflow-x: auto"));
    }

    /// 機能名セルが `<th scope="row">` として出力されること（上記モジュール
    /// doc「`table` の合成方法」節参照。スクリーンリーダーが対応/非対応
    /// セルへ移動した際に機能名が行見出しとして読み上げられるための
    /// アクセシビリティ不変条件）。件数は [`ROWS`] と一致する。
    #[test]
    fn feature_cells_are_row_headers() {
        let html = render(&demo());
        assert_eq!(
            html.matches("<th data-scope=\"table\" data-part=\"cell\" scope=\"row\"")
                .count(),
            ROWS.len(),
            "every feature cell should be a <th scope=\"row\"> row header"
        );
        // `column-header`（列見出し `scope=\"col\"`）と行見出し
        // （`scope=\"row\"`）が混在するため、行見出しの総数がテーブル内の
        // `scope=\"row\"` 出現数と一致することも確認する。
        assert_eq!(
            html.matches("scope=\"row\"").count(),
            ROWS.len(),
            "scope=\"row\" should appear exactly once per row"
        );
    }

    /// 対応/非対応アイコンの CSS フックが `class` ではなく `data-*` 属性で
    /// 出力されること（上記モジュール doc「アイコンは独自の抽象図形のみ・
    /// アクセシブルネームを持つ」節。`icon::icon` が呼び出し側 `class` を
    /// `drop_class_attr` で除去するため、`class` 経由では CSS が一切
    /// 適用されない不具合の回帰防止）。
    #[test]
    fn icon_color_hooks_survive_as_data_attributes() {
        let html = render(&demo());
        let expected_yes = ROWS
            .iter()
            .flat_map(|row| [&row.ours, &row.theirs])
            .filter(|value| matches!(value, CellValue::Yes))
            .count();
        let expected_no = ROWS
            .iter()
            .flat_map(|row| [&row.ours, &row.theirs])
            .filter(|value| matches!(value, CellValue::No))
            .count();
        assert_eq!(
            html.matches("data-blocks-comparison-split-table-yes=\"\"")
                .count(),
            expected_yes,
            "Yes icons should carry the yes CSS hook as a data attribute"
        );
        assert_eq!(
            html.matches("data-blocks-comparison-split-table-no=\"\"")
                .count(),
            expected_no,
            "No icons should carry the no CSS hook as a data attribute"
        );
        assert!(
            !html.contains("class=\"blocks-comparison-split-table-yes\"")
                && !html.contains("class=\"blocks-comparison-split-table-no\""),
            "icon color hooks must not rely on a class attribute dropped by icon::icon"
        );
        assert!(
            LAYOUT_CSS.contains("[data-blocks-comparison-split-table-yes]")
                && LAYOUT_CSS.contains("[data-blocks-comparison-split-table-no]"),
            "LAYOUT_CSS must target the icon color hooks as attribute selectors"
        );
    }

    /// Yes アイコンの配色が `--fandhe-color-accent-fg`（アクセント背景上の
    /// コントラスト色）ではなく `--fandhe-color-accent`（`bg`/`bg-muted`
    /// 背景上で WCAG 3:1 を満たす検証済みトークン）を使うこと（上記モジュール
    /// doc「アイコンは独自の抽象図形のみ・アクセシブルネームを持つ」節参照）。
    #[test]
    fn yes_icon_uses_accent_not_accent_fg() {
        assert!(LAYOUT_CSS.contains("var(--fandhe-color-accent)"));
        assert!(!LAYOUT_CSS.contains("var(--fandhe-color-accent-fg)"));
    }
}
