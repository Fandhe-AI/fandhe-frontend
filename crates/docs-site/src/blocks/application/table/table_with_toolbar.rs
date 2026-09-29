//! `table-with-toolbar` block（親 #2948「Blocks に Application / Table
//! カテゴリ block `table-with-toolbar` を追加する」）。前半 #2949
//! （PR #3401、`43661548`）で骨格 + 版 A（代表構成: 見出し帯・検索・
//! 絞り込み/新規作成・横スクロール表・件数表示/ページ送り）を実装済み。
//! 後半 #2950 で残り 3 版（期間選択ボタン版 R0716・常時縦積み +
//! エクスポート版 R0717・タブ型絞り込み版 R0718）を並記し、既存部品
//! （`heading` / `text` / `input_group` / `input` / `button` / `icon` /
//! `table` / `badge` / `pagination` / `scroll_area`）に `tabs` を加えた
//! 計 11 部品の合成で示す。対応表 ID は主参照 R0645（代表構成）・集約
//! R0715/R0716/R0717/R0718。`_/blocks-intake/` の対応ファイルは本イシュー
//! 着手時点で本 worktree に存在しないため、原稿・本コメントには対応表 ID
//! のみを記す（`page_heading_meta.rs`〔イシュー #2933〕・
//! `table_with_heading.rs`〔イシュー #2947〕と同じ扱い）。
//!
//! # 4 版の並記（イシュー #2950）
//!
//! `demo()` は版 A〜D を [`caption`]（`h2`）の見出しで区切って縦に並べる
//! （`marketing/footer/footer_inline_nav.rs` と同じ並記パターン）。各版の
//! `title_group()` は `h3`「Invoices」で 4 版共通のため、`caption` の `h2`
//! を版名の一意な見出しとして先に置くことで見出し一覧上でも版を区別できる
//! ようにする（Codex レビュー指摘 #3404 是正: 版名が `p` の平文だった旧実装
//! は見出し一覧から区別できなかった）。表・フッターのデータは 4 版で共有し
//! （[`ROWS`]・
//! [`footer`]）、差分は見出し帯（`header`）と絞り込み表現のみに絞る。
//!
//! - **A（R0645/R0715、代表構成）**: 検索 + 絞り込み/新規作成ボタン。
//!   `@container (min-width: 40rem)` で見出し帯・ツールバー・フッターが
//!   縦積み ↔ 横並びを切り替える
//! - **B（R0716、期間選択ボタン）**: ツールバーに「7 日間/30 日間/90 日間」
//!   の期間選択ボタン列（[`period_buttons`]）を追加する。各ボタンは
//!   `feature_accordion_image::category_button` と同型の静的固定
//!   （先頭のみ `ButtonVariant::Solid` + `aria-pressed="true"`、他は
//!   `Outline` + `aria-pressed="false"`、全て `disabled: true`）で、押せる
//!   が何も起きない dead control にしない（`aria-pressed` で状態を明示）。
//!   参照元の期間区切り定義は持ち込まず独自の文言を使う。ツールバーの
//!   横並び時（`@container (min-width: 40rem)`）は検索欄・期間ボタン 3
//!   個・操作ボタン 2 個が 1 行に収まらずコンテナ幅を超えるため
//!   `[data-blocks-table-with-toolbar-toolbar]` に `flex-wrap: wrap` を
//!   与えて折り返し可能にする（Codex レビュー指摘 #3404 是正、`LAYOUT_CSS`
//!   参照）。検索欄（[`input_group::root`]）は base CSS で `width: 100%`
//!   を持つため `flex-wrap: wrap` だけでは自身の flex-basis が行全体を
//!   専有し期間ボタン・操作ボタンが常に折り返される（Cursor Bugbot
//!   レビュー指摘 #3404 是正）。`[data-blocks-table-with-toolbar-search]`
//!   に `flex: 1 1 12rem` を与えて非 `auto` の flex-basis で `width` を
//!   上書きし、`card_heading_toolbar` と同型の縮小可能な検索欄にする
//!   （版 A/D は 1 行、版 B はコンテナ幅超過時のみ折り返す）。操作ボタン列
//!   （[`actions`]）も `flex-wrap: wrap` を持ち、Demo 枠が版 B のボタン
//!   合計幅（絞り込み + 期間ボタン 3 個 + 新規作成）より狭い場合に折り返す
//!   （Codex レビュー指摘 #3404 是正: `display: flex` のみで折り返し不可
//!   だったため、狭い Demo 枠でボタン列がコンテナ幅を超えてはみ出していた）
//! - **C（R0717、常時縦積み + エクスポート）**: `@container` の横並び
//!   切り替えを持たず、幅に関係なく見出し帯・ツールバーが常時縦積み
//!   （`[data-blocks-table-with-toolbar-variant="stacked"]` 配下の属性
//!   セレクタ 2 段（詳細度 `(0,2,0)`）で `@container` 内の `(0,1,0)` を
//!   上書きする、`LAYOUT_CSS` 参照）。操作列にエクスポートボタン
//!   （[`export_icon`] の自作線画）を追加する。検索欄
//!   （[`toolbar`] 内の `[data-blocks-table-with-toolbar-search]`）は
//!   `@container` 内の `flex: 1 1 12rem` が常時縦積みでも詳細度 `(0,1,0)`
//!   のまま残るため、`column` 方向の flex コンテナ下では `flex-basis` が
//!   高さへ転用され検索欄が 12rem 丈に伸びていた（Cursor Bugbot レビュー
//!   指摘 #3404 是正）。`[data-blocks-table-with-toolbar-variant="stacked"]
//!   [data-blocks-table-with-toolbar-search]`（詳細度 `(0,2,0)`）で
//!   `flex: 1 1 auto` に戻し、高さを内容依存（`auto`）へ復元する
//! - **D（R0718、タブ型の絞り込み）**: 実物 `tabs::tabs`
//!   （[`status_tabs`]）で「すべて/支払済み/未払い/期限超過」の 4 タブを
//!   静的表示する。**選択中「すべて」のパネルにのみ表を置き、非選択の 3
//!   パネルは空のままとする**（絞り込み条件の静的表示であり、実際の
//!   絞り込み処理は UI コンポーネント層の責務外
//!   `docs/policy/intentional-non-adoption.md` §3.25）。表を tabs の外へ
//!   常時可視で置いていた旧実装は、選択中 trigger の `aria-controls` が
//!   指すパネルに実体がなく表とタブの選択操作が構造的に不整合だった
//!   （Codex レビュー指摘 #3404 是正。詳細は [`status_tabs`] rustdoc
//!   参照）。`tabs` は attrs を受け取らない
//!   （`crate::tabs` モジュール doc「選択的 re-export」節参照）ため
//!   `div[data-blocks-table-with-toolbar-tabs]` で包む。選択中「すべて」
//!   以外の 3 タブは `disabled: true` の静的固定（[`status_tabs`] rustdoc
//!   参照。Codex レビュー指摘 #3404 是正: 押せるが表が変わらない
//!   dead control にしない）
//!
//! # `scroll_area` を選ぶ理由（`table::scroll_area` ではなく独立部品）
//!
//! `table` 部品内蔵の [`fandhe_frontend_pre_styled_ui::table::scroll_area`]
//! ではなく [`fandhe_frontend_pre_styled_ui::scroll_area`] を使う。親仕様
//! （対応表 R0645/R0715）の使用部品一覧に「Scroll Area」が明記されており、
//! [`crate::blocks::Part::path`] で `/themes/scroll-area/` へ相互リンクする
//! ため（`list_sticky_groups.rs` と同じ判断）。
//!
//! # `@container` で狭幅レイアウトを切り替える
//!
//! ルート（`.blocks-table-with-toolbar-layout`）に `container-type:
//! inline-size` を与え、Demo 枠自体の幅で見出し帯・ツールバー・フッターの
//! 縦積み/横並びを切り替える（`@media` のビューポート幅ではなく block の
//! 実際の描画幅で判定するため。`docs/design/docs-site-blocks-section.md`
//! の既存 `@media (min-width: 40rem)` 系 block とは異なる判断だが、狭い
//! Demo 枠に埋め込まれる Blocks ページの実際のレイアウト崩れを防ぐには
//! コンテナクエリの方が実態に合う）。
//!
//! # `class` と `data-*` の使い分け
//!
//! `input_group::root`・`button::button`・`table::root`・
//! `pagination::root`・`scroll_area::root` は `drop_class_attr` を経由して
//! 呼び出し側の `class` を除去するため、これらへは
//! `data-blocks-table-with-toolbar-*` 属性でレイアウトフックする。素の
//! `div` へは `.blocks-table-with-toolbar-*` クラスを使う。
//!
//! # アイコンは自作の単純図形（線画）
//!
//! 検索・絞り込み・新規作成の各アイコンは [`profile_detail_datalist`]
//! （`crate::blocks::application::profile::profile_detail_datalist`）の
//! `geo_icon` と同型の自作単純図形（`fill="none"` + `stroke="currentColor"`）
//! で、いずれも装飾用途（`IconProps::label: None` の既定で `aria-hidden`）。
//! 参照元（対応表 R0645/R0715）のアイコンセットは持ち込まない。
//!
//! # 状態は静的固定（無 JS）
//!
//! 検索欄は空、1 ページ目を選択し前ページ送りボタンは無効の固定表示。
//! 開閉・遷移・データ取得は行わない（`crate::blocks` モジュール doc
//! 「`<form>` を使わない」節・「セキュリティ不変条件」節に従う）。
//! ボタンは `button::button` の既定 `type="button"` のまま送信先を持たず
//! （`docs/policy/intentional-non-adoption.md` §3.25：バリデーション・
//! 送信処理は UI コンポーネント層の責務外）、`pagination::item`/
//! `prev_trigger`/`next_trigger` は `ItemMode::Button` を使い `href` を
//! 持たせない（静的表示のため遷移先を持たせない）。架空の請求書番号・
//! 顧客名・金額・期日はすべてダミー（実企業名・PII・クレデンシャルを
//! 含まない。顧客名は [`crate::blocks::dummy_assets::COMPANY_NAMES`] を
//! 再利用する）。版 B の期間選択ボタン（`disabled: true` +
//! `aria-pressed` 固定）・版 D の `tabs`（`selected: "all"` 固定・パネル
//! 空）も同じ静的固定方針に従う。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, section, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps, BadgeVariant};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::field::{FieldIds, FieldProps};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::input::{self, InputProps};
use fandhe_frontend_pre_styled_ui::input_group::{self, InputGroupAlign, InputGroupProps};
use fandhe_frontend_pre_styled_ui::pagination::{self, ItemMode};
use fandhe_frontend_pre_styled_ui::recipe::ColorPalette;
use fandhe_frontend_pre_styled_ui::scroll_area;
use fandhe_frontend_pre_styled_ui::table::{self, TableProps, TableVariant};
use fandhe_frontend_pre_styled_ui::tabs::{
    self, ActivationMode, Orientation, TabItem, TabsProps, TabsVariant,
};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};
use fandhe_frontend_pre_styled_ui::Size;

/// 自作の幾何アイコン（線画）。`profile_detail_datalist::geo_icon` と同型
/// （モジュール doc「アイコンは自作の単純図形」節参照）。
fn geo_icon(path_d: &'static str) -> Node {
    icon(
        &IconProps::default(),
        vec![],
        vec![el(
            "path",
            vec![
                ("d", path_d),
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

/// 検索アイコン（虫眼鏡）。
fn search_icon() -> Node {
    geo_icon("M11 4a7 7 0 1 0 0 14 7 7 0 0 0 0-14z M16.5 16.5L21 21")
}

/// 絞り込みアイコン（漏斗）。
fn filter_icon() -> Node {
    geo_icon("M4 5h16 M7 12h10 M10 19h4")
}

/// 新規作成アイコン（プラス）。
fn plus_icon() -> Node {
    geo_icon("M12 5v14 M5 12h14")
}

/// エクスポートアイコン（下向き矢印 + トレイ、版 C 専用）。
fn export_icon() -> Node {
    geo_icon("M12 4v12 M7 11l5 5 5-5 M4 20h16")
}

/// 請求書 1 行分のダミーデータ。
struct InvoiceRow {
    /// 請求書番号（`row_header` に出す一意なキー）。
    number: &'static str,
    /// 顧客名（[`dummy_assets::COMPANY_NAMES`] を周回参照する）。
    customer_index: usize,
    /// 状態ラベル + バッジ配色。
    status: (&'static str, ColorPalette),
    /// 金額（整形処理を書かず固定文字列、`.claude/rules/coding-rust.md`
    /// §3.23 相当の判断: 数値・日時整形は UI コンポーネント層/block の
    /// 責務外）。
    amount: &'static str,
    /// 支払期日（固定文字列）。
    due: &'static str,
}

const ROWS: &[InvoiceRow] = &[
    InvoiceRow {
        number: "INV-2041",
        customer_index: 0,
        status: ("支払済み", ColorPalette::Success),
        amount: "¥128,000",
        due: "2026-08-15",
    },
    InvoiceRow {
        number: "INV-2042",
        customer_index: 1,
        status: ("未払い", ColorPalette::Warning),
        amount: "¥64,500",
        due: "2026-09-01",
    },
    InvoiceRow {
        number: "INV-2043",
        customer_index: 2,
        status: ("支払済み", ColorPalette::Success),
        amount: "¥212,300",
        due: "2026-09-10",
    },
    InvoiceRow {
        number: "INV-2044",
        customer_index: 3,
        status: ("期限超過", ColorPalette::Danger),
        amount: "¥38,900",
        due: "2026-08-28",
    },
    InvoiceRow {
        number: "INV-2045",
        customer_index: 4,
        status: ("支払済み", ColorPalette::Success),
        amount: "¥95,000",
        due: "2026-09-20",
    },
    InvoiceRow {
        number: "INV-2046",
        customer_index: 5,
        status: ("未払い", ColorPalette::Warning),
        amount: "¥171,250",
        due: "2026-09-25",
    },
];

/// 見出し帯左側（表題 + 説明）。
fn title_group() -> Node {
    div(
        vec![("data-blocks-table-with-toolbar-title", "")],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl,
                    ..HeadingProps::default()
                },
                vec![],
                vec![text("Invoices")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text("直近の請求書をまとめて確認できます。")],
            ),
        ],
    )
}

/// 期間選択ボタン 1 個（版 B 専用）。`feature_accordion_image::
/// category_button` と同型: 先頭のみ選択済み（`Solid` +
/// `aria-pressed="true"`）、他は非選択（`Outline` + `aria-pressed="false"`）。
/// 全ボタン `disabled: true` の静的固定（モジュール doc「B（期間選択
/// ボタン）」節参照）。
fn period_button(label: &'static str, selected: bool) -> Node {
    let (variant, pressed) = if selected {
        (ButtonVariant::Solid, "true")
    } else {
        (ButtonVariant::Outline, "false")
    };
    button::button(
        &ButtonProps {
            variant,
            size: Size::Sm,
            disabled: true,
            ..ButtonProps::default()
        },
        vec![("aria-pressed", pressed)],
        vec![text(label)],
    )
}

/// 期間選択ボタン列（版 B 専用。「7 日間」を初期選択として固定する）。
fn period_buttons() -> Node {
    div(
        vec![("data-blocks-table-with-toolbar-period", "")],
        vec![
            period_button("7 日間", true),
            period_button("30 日間", false),
            period_button("90 日間", false),
        ],
    )
}

/// 操作ボタン列（絞り込み → [エクスポート（版 C のみ）] → 新規作成）。
fn actions(export: bool) -> Node {
    let mut children = vec![button::button(
        &ButtonProps {
            variant: ButtonVariant::Outline,
            ..ButtonProps::default()
        },
        vec![],
        vec![filter_icon(), text("絞り込み")],
    )];
    if export {
        children.push(button::button(
            &ButtonProps {
                variant: ButtonVariant::Outline,
                ..ButtonProps::default()
            },
            vec![],
            vec![export_icon(), text("エクスポート")],
        ));
    }
    children.push(button::button(
        &ButtonProps::default(),
        vec![],
        vec![plus_icon(), text("新規作成")],
    ));
    div(
        vec![("data-blocks-table-with-toolbar-actions", "")],
        children,
    )
}

/// タブ型の絞り込み（版 D 専用）。実物 `tabs::tabs` を使い「すべて」を
/// 初期選択として固定する。選択中「すべて」のパネルにのみ `table`（引数
/// `table_node`）を置き、非選択の 3 パネルは空のままとする（モジュール doc
/// 「D（タブ型の絞り込み）」節参照）。`search_id` ごとに呼び出し側の `id` が
/// 変わるのと同様、本関数は版 D でのみ呼ばれるため固定 id を持たせてよい
/// （demo 内で 1 回しか呼ばれない契約）。
///
/// 表を「すべて」パネルの内側へ置くのは Codex レビュー指摘 #3404
/// （`role="tab"`/`aria-controls` が指す tabpanel に実体がなく、表が
/// `variant()` によりタブの外に置かれていたため選択操作と表示内容が
/// 構造的に不整合だった）の是正である。以前の実装は全パネルを空にし表を
/// tabs の外側へ常時可視で置いていたが、これは選択中パネルが空のまま
/// 可視化され（Cursor Bugbot レビュー指摘 #3404: 既定 padding で空白帯が
/// 表示され `tabindex="0"` の空パネルが読み上げ対象になる）、かつ
/// `aria-controls` の参照先に実体がない二重の不整合を生んでいた。表を
/// 「すべて」パネルへ移すことで、選択中 trigger の `aria-controls` は
/// 実際に表を含むパネルを指すようになり、非選択の 3 タブは
/// `disabled: true` の静的固定（絞り込み処理を実装しない以上、操作可能な
/// まま見せると「押せるが表の内容が変わらない」dead control になるため、
/// 版 B の期間選択ボタンと同じ判断）のため到達不能であり構造不整合が
/// 顕在化しない。tabs は選択中 trigger を `disabled` にすると
/// `aria-selected`/`data-state` が「未選択」扱いへ落ちる仕様
/// （`crates/headless-ui/src/tabs.rs` `selected_matching_disabled_item_is_treated_as_unselected`
/// 参照）のため、選択中の「すべて」のみ非 disabled のまま残す。
fn status_tabs(table_node: Node) -> Node {
    let items = vec![
        TabItem {
            value: "all",
            trigger: vec![text("すべて")],
            content: vec![table_node],
            disabled: false,
        },
        TabItem {
            value: "paid",
            trigger: vec![text("支払済み")],
            content: vec![],
            disabled: true,
        },
        TabItem {
            value: "unpaid",
            trigger: vec![text("未払い")],
            content: vec![],
            disabled: true,
        },
        TabItem {
            value: "overdue",
            trigger: vec![text("期限超過")],
            content: vec![],
            disabled: true,
        },
    ];
    div(
        vec![("data-blocks-table-with-toolbar-tabs", "")],
        vec![tabs::tabs(
            TabsVariant::Line,
            Size::Sm,
            ColorPalette::Accent,
            &TabsProps {
                id: "blocks-table-with-toolbar-tabs",
                selected: "all",
                orientation: Orientation::Horizontal,
                activation_mode: ActivationMode::Automatic,
                loop_focus: true,
                indicator: false,
            },
            items,
        )],
    )
}

/// 見出し帯右側（検索 + [期間選択（版 B）] + 操作ボタン列）。`search_id` は
/// 呼び出し側（[`variant`]）が版ごとに一意な値を渡し、複数版並記時の
/// `id` 重複（`tests/blocks_contract.rs`）を避ける。`label` は
/// [`table_section`]/[`footer`] と同じ理由で検索欄の `aria-label` を版ごとに
/// 一意化する（Cursor Bugbot レビュー指摘 #3404 是正: 4 版すべてが同一
/// `aria-label`「請求書を検索」を再利用し、支援技術上で区別できなかった）。
fn toolbar(search_id: &'static str, label: &str, period: bool, export: bool) -> Node {
    let search_aria_label = format!("請求書を検索（{label}）");
    let field = FieldProps {
        id: search_id,
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: false,
    };
    let group_props = InputGroupProps {
        disabled: false,
        invalid: false,
    };
    let mut children = vec![input_group::root(
        &group_props,
        vec![("data-blocks-table-with-toolbar-search", "")],
        vec![
            input_group::addon(
                InputGroupAlign::InlineStart,
                &group_props,
                vec![],
                vec![search_icon()],
            ),
            input::input(
                &InputProps::default(),
                &field,
                vec![
                    ("type", "search"),
                    ("placeholder", "請求書を検索"),
                    ("aria-label", &search_aria_label),
                ],
            ),
        ],
    )];
    if period {
        children.push(period_buttons());
    }
    children.push(actions(export));
    div(
        vec![("data-blocks-table-with-toolbar-toolbar", "")],
        children,
    )
}

/// 見出し帯全体（表題群 + ツールバー）。
fn header(search_id: &'static str, label: &str, period: bool, export: bool) -> Node {
    div(
        vec![("data-blocks-table-with-toolbar-header", "")],
        vec![title_group(), toolbar(search_id, label, period, export)],
    )
}

/// 列見出し行（Amount は右寄せの目印として `data-align="end"` を持つ。
/// `table` 部品自体は列寄せの variant を持たないため block 固有の CSS
/// フックとして扱う）。
fn column_headers() -> Node {
    table::row(
        vec![],
        vec![
            table::column_header(vec![], vec![text("請求書番号")]),
            table::column_header(vec![], vec![text("顧客")]),
            table::column_header(vec![], vec![text("状態")]),
            table::column_header(vec![("data-align", "end")], vec![text("金額")]),
            table::column_header(vec![], vec![text("期日")]),
        ],
    )
}

/// 本文 1 行。
fn body_row(row: &InvoiceRow) -> Node {
    let customer =
        dummy_assets::COMPANY_NAMES[row.customer_index % dummy_assets::COMPANY_NAMES.len()];
    let (status_label, status_palette) = row.status;
    table::row(
        vec![],
        vec![
            table::row_header(vec![], vec![text(row.number)]),
            table::cell(vec![], vec![text(customer)]),
            table::cell(
                vec![],
                vec![badge::badge(
                    &BadgeProps {
                        variant: BadgeVariant::Subtle,
                        palette: status_palette,
                        ..BadgeProps::default()
                    },
                    vec![],
                    vec![text(status_label)],
                )],
            ),
            table::cell(vec![("data-align", "end")], vec![text(row.amount)]),
            table::cell(vec![], vec![text(row.due)]),
        ],
    )
}

/// テーブル本体（横スクロール対応の `scroll_area` 包み）。`label` は
/// 呼び出し側（[`variant`]）が版ごとに一意な文言を渡し、複数版並記時の
/// ランドマーク名重複・テーブル無名化（Cursor Bugbot レビュー指摘 #3404
/// 是正: 4 版が同一 `aria-label`「請求書一覧」を再利用し、テーブル自体は
/// 無名のままだったため支援技術上区別できなかった）を避ける。`table::root`
/// へも `aria-label` を付け、テーブル自体に固有の名前を持たせる。
fn table_section(label: &str) -> Node {
    let table_aria_label = format!("請求書一覧表（{label}）");
    let scroll_aria_label = format!("請求書一覧（{label}）");
    let table_node = table::root(
        TableProps {
            variant: TableVariant::Line,
            size: Size::Md,
            ..TableProps::default()
        },
        vec![
            ("data-blocks-table-with-toolbar-table", ""),
            ("aria-label", &table_aria_label),
        ],
        vec![
            table::header(vec![], vec![column_headers()]),
            table::body(vec![], ROWS.iter().map(body_row).collect()),
        ],
    );
    scroll_area::root(
        vec![("data-blocks-table-with-toolbar-scroll", "")],
        vec![scroll_area::viewport(
            vec![("role", "region"), ("aria-label", &scroll_aria_label)],
            vec![scroll_area::content(vec![], vec![table_node])],
        )],
    )
}

/// フッター（件数表示 + ページ送り。1 ページ目選択・前ページ無効で固定）。
/// `label` は [`table_section`] と同じ理由でページ送り `nav` の
/// アクセシブルネーム（`aria-label`）を版ごとに一意化する。
fn footer(label: &str) -> Node {
    let nav_aria_label = format!("請求書ページ（{label}）");
    div(
        vec![("data-blocks-table-with-toolbar-footer", "")],
        vec![
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text("42 件中 1–6 件を表示")],
            ),
            pagination::root(
                Size::Sm,
                ColorPalette::Accent,
                &nav_aria_label,
                vec![("data-blocks-table-with-toolbar-pagination", "")],
                vec![
                    pagination::prev_trigger(ItemMode::Button, true, vec![], vec![text("前へ")]),
                    pagination::item(ItemMode::Button, 1, true, false, vec![], vec![text("1")]),
                    pagination::item(ItemMode::Button, 2, false, false, vec![], vec![text("2")]),
                    pagination::item(ItemMode::Button, 3, false, false, vec![], vec![text("3")]),
                    pagination::ellipsis(vec![], vec![text("…")]),
                    pagination::item(ItemMode::Button, 7, false, false, vec![], vec![text("7")]),
                    pagination::next_trigger(ItemMode::Button, false, vec![], vec![text("次へ")]),
                ],
            ),
        ],
    )
}

/// 版 1 つ分（見出し帯 + [タブ行] + テーブル + フッター）。`kind` は
/// `data-blocks-table-with-toolbar-variant` の値（`"standard"`/`"period"`/
/// `"stacked"`/`"tabs"`）。`search_id` は版ごとに一意な検索欄 `id`
/// （[`toolbar`] rustdoc「`id` 重複を避ける」節参照）。`label` は
/// [`table_section`]/[`footer`] のランドマーク名一意化に使う人間可読な
/// 版名（`demo()` のキャプション文言を再利用する）。
fn variant(
    kind: &'static str,
    search_id: &'static str,
    label: &'static str,
    period: bool,
    export: bool,
    show_tabs: bool,
) -> Node {
    let mut children = vec![header(search_id, label, period, export)];
    if show_tabs {
        // 表は「すべて」タブのパネル内へ置く（tabs の外へは置かない）。
        // [`status_tabs`] rustdoc「Codex レビュー指摘 #3404 是正」節参照。
        children.push(status_tabs(table_section(label)));
    } else {
        children.push(table_section(label));
    }
    children.push(footer(label));
    section(
        vec![("data-blocks-table-with-toolbar-variant", kind)],
        children,
    )
}

/// `table-with-toolbar` の Demo 本体（版 A〜D を並記。呼び出しごとに同一の
/// `Node` を返す純関数。モジュール doc「4 版の並記」節参照）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-table-with-toolbar-layout")],
        vec![
            caption("代表構成"),
            variant(
                "standard",
                "blocks-table-with-toolbar-search-a",
                "代表構成",
                false,
                false,
                false,
            ),
            caption("期間選択ボタン付き"),
            variant(
                "period",
                "blocks-table-with-toolbar-search-b",
                "期間選択ボタン付き",
                true,
                false,
                false,
            ),
            caption("常時縦積み + エクスポート"),
            variant(
                "stacked",
                "blocks-table-with-toolbar-search-c",
                "常時縦積み + エクスポート",
                false,
                true,
                false,
            ),
            caption("タブ型の絞り込み付き"),
            variant(
                "tabs",
                "blocks-table-with-toolbar-search-d",
                "タブ型の絞り込み付き",
                false,
                false,
                true,
            ),
        ],
    )
}

/// 版のキャプション（見出し）。`h2`（[`title_group`] の `h3`「Invoices」の
/// 親階層）として構造化する。4 版すべてが同一の `h3`「Invoices」を持つため
/// `p` の平文キャプションのままでは見出し一覧上で版を区別できなかった
/// （Codex レビュー指摘 #3404 是正）。`heading` は `class` を
/// `drop_class_attr` 経由で除去するため、スタイルフックには
/// `data-blocks-table-with-toolbar-caption` を使う
/// （モジュール doc「`class` と `data-*` の使い分け」節参照）。
fn caption(label: &'static str) -> Node {
    heading(
        HeadingLevel::H2,
        &HeadingProps {
            size: HeadingSize::Sm,
            ..HeadingProps::default()
        },
        vec![("data-blocks-table-with-toolbar-caption", "")],
        vec![text(label)],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/table-with-toolbar/",
    title: "table-with-toolbar",
    category: BlockCategory::Table,
    rust_source: "crates/docs-site/src/blocks/application/table/table_with_toolbar.rs",
    demo_class: "blocks-table-with-toolbar",
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
            label: "Input Group",
            path: "/themes/input-group/",
        },
        Part {
            label: "Input",
            path: "/themes/input/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Icon",
            path: "/themes/icon/",
        },
        Part {
            label: "Table",
            path: "/themes/table/",
        },
        Part {
            label: "Badge",
            path: "/themes/badge/",
        },
        Part {
            label: "Pagination",
            path: "/themes/pagination/",
        },
        Part {
            label: "Scroll Area",
            path: "/themes/scroll-area/",
        },
        Part {
            label: "Tabs",
            path: "/themes/tabs/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `table_with_toolbar` 固有のレイアウト規則。セレクタは
/// `.blocks-table-with-toolbar-*` と `[data-blocks-table-with-toolbar-*]`
/// のみを用いる。ルート class を `demo_class` と別名にする（既存 block と
/// 同じ Bugbot 教訓の回避）。狭幅判定は `@media` ではなく `@container`
/// （モジュール doc「`@container` で狭幅レイアウトを切り替える」節参照）。
const LAYOUT_CSS: &str = "\
.blocks-table-with-toolbar-layout {\n  container-type: inline-size;\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n}\n\
[data-blocks-table-with-toolbar-caption] {\n  margin: 0;\n  color: var(--fandhe-color-fg-muted);\n}\n\
[data-blocks-table-with-toolbar-variant] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
[data-blocks-table-with-toolbar-header] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n  justify-content: space-between;\n}\n\
[data-blocks-table-with-toolbar-toolbar] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n}\n\
[data-blocks-table-with-toolbar-period] {\n  display: flex;\n  gap: var(--fandhe-space-1);\n}\n\
[data-blocks-table-with-toolbar-actions] {\n  display: flex;\n  flex-wrap: wrap;\n  gap: var(--fandhe-space-2);\n}\n\
[data-blocks-table-with-toolbar-tabs] {\n  overflow-x: auto;\n}\n\
[data-blocks-table-with-toolbar-scroll] {\n  max-width: 100%;\n}\n\
[data-blocks-table-with-toolbar-table] {\n  min-width: 42rem;\n}\n\
[data-blocks-table-with-toolbar-table] [data-align=\"end\"] {\n  text-align: end;\n}\n\
[data-blocks-table-with-toolbar-footer] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n  align-items: flex-start;\n  justify-content: space-between;\n  border-top: 1px solid var(--fandhe-color-border);\n  padding-top: var(--fandhe-space-4);\n}\n\
@container (min-width: 40rem) {\n  [data-blocks-table-with-toolbar-header] {\n    flex-direction: row;\n    align-items: flex-end;\n  }\n  [data-blocks-table-with-toolbar-toolbar] {\n    flex-direction: row;\n    flex-wrap: wrap;\n    align-items: center;\n  }\n  [data-blocks-table-with-toolbar-search] {\n    flex: 1 1 12rem;\n    min-width: 0;\n  }\n  [data-blocks-table-with-toolbar-footer] {\n    flex-direction: row;\n    align-items: center;\n  }\n}\n\
[data-blocks-table-with-toolbar-variant=\"stacked\"] [data-blocks-table-with-toolbar-header], [data-blocks-table-with-toolbar-variant=\"stacked\"] [data-blocks-table-with-toolbar-toolbar] {\n  flex-direction: column;\n  align-items: stretch;\n}\n\
[data-blocks-table-with-toolbar-variant=\"stacked\"] [data-blocks-table-with-toolbar-search] {\n  flex: 1 1 auto;\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// Demo が使用部品（heading/text/input-group/field(input)/button/icon/
    /// table/badge/pagination/scroll-area/tabs）の anatomy をすべて実際に
    /// 出力していること。
    #[test]
    fn demo_composes_expected_parts() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"input-group\"",
            "data-scope=\"field\" data-part=\"input\"",
            "data-scope=\"button\"",
            "data-scope=\"icon\"",
            "data-scope=\"table\"",
            "data-scope=\"badge\"",
            "data-scope=\"pagination\"",
            "data-scope=\"scroll-area\"",
            "data-scope=\"tabs\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
    }

    /// 主要領域の目印（見出し帯の検索・テーブル・ページ送り・スクロール
    /// 領域）が render 出力に現れる。
    #[test]
    fn demo_marks_primary_regions() {
        let html = render(&demo());
        for marker in [
            "data-blocks-table-with-toolbar-search",
            "data-blocks-table-with-toolbar-table",
            "data-blocks-table-with-toolbar-pagination",
            "data-blocks-table-with-toolbar-scroll",
        ] {
            assert!(html.contains(marker), "demo output should contain {marker}");
        }
    }

    /// 請求書行はちょうど 24 件（6 行 × 4 版、`table::row_header` の出力
    /// 件数で数える）。
    #[test]
    fn demo_row_count() {
        let html = render(&demo());
        assert_eq!(
            html.matches("data-scope=\"table\" data-part=\"row-header\"")
                .count(),
            24
        );
    }

    /// 4 版すべてが `data-blocks-table-with-toolbar-variant` で 1 回ずつ
    /// 現れる（版の並記漏れ・重複の検知）。
    #[test]
    fn demo_declares_four_variants() {
        let html = render(&demo());
        for kind in ["standard", "period", "stacked", "tabs"] {
            let needle = format!("data-blocks-table-with-toolbar-variant=\"{kind}\"");
            assert_eq!(
                html.matches(&needle).count(),
                1,
                "expected exactly one {needle} in demo output"
            );
        }
    }

    /// 版 B の期間選択ボタンは先頭のみ `aria-pressed="true"`、残り 2 件は
    /// `"false"` の静的固定（モジュール doc「B（期間選択ボタン）」節参照）。
    #[test]
    fn period_buttons_are_static_pressed_state() {
        let html = render(&demo());
        assert_eq!(html.matches(r#"aria-pressed="true""#).count(), 1);
        assert_eq!(html.matches(r#"aria-pressed="false""#).count(), 2);
    }

    /// 版 D の `tabs` は 4 trigger を持ち、選択中「すべて」パネルにのみ表を
    /// 持つ（非選択の 3 パネルは空のまま）。表を tabs の外へ常時可視で置く
    /// 旧実装は選択中 trigger の `aria-controls` が指すパネルに実体がなく
    /// 構造的に不整合だったため、[`status_tabs`] rustdoc「Codex レビュー
    /// 指摘 #3404 是正」節の判断で表を「すべて」パネルの内側へ移した
    /// （モジュール doc「D（タブ型の絞り込み）」節参照）。
    #[test]
    fn tabs_all_panel_contains_table_others_are_empty() {
        let html = render(&demo());
        assert_eq!(
            html.matches("data-scope=\"tabs\" data-part=\"trigger\"")
                .count(),
            4
        );
        assert_eq!(
            html.matches("data-scope=\"table\" data-part=\"root\"")
                .count(),
            4
        );
        // 「すべて」パネル（`id="…-content-all"`）の開始タグ直後に、表を
        // 包む `scroll_area` の root（`data-scope="scroll-area"`）が続く。
        let all_panel_needle = "id=\"blocks-table-with-toolbar-tabs-content-all\"";
        let all_panel_start = html
            .find(all_panel_needle)
            .expect("all panel should be present");
        let Some(tag_end_rel) = html[all_panel_start..].find('>') else {
            panic!("all panel opening tag should be closed");
        };
        let after_all_panel_tag = all_panel_start + tag_end_rel + 1;
        assert!(
            html[after_all_panel_tag..].starts_with("<div data-scope=\"scroll-area\""),
            "all panel should contain the table's scroll_area wrapper"
        );
        // 非選択の 3 パネル（paid/unpaid/overdue）は子ノードを持たない
        // （開始タグ直後に閉じタグ）。
        for value in ["paid", "unpaid", "overdue"] {
            let needle = format!("id=\"blocks-table-with-toolbar-tabs-content-{value}\"");
            let panel_start = html
                .find(&needle)
                .unwrap_or_else(|| panic!("{value} panel should be present"));
            let Some(tag_end_rel) = html[panel_start..].find('>') else {
                panic!("{value} panel opening tag should be closed");
            };
            let after_tag = panel_start + tag_end_rel + 1;
            assert!(
                html[after_tag..].starts_with("</div>"),
                "{value} panel should have no children"
            );
        }
    }

    /// 版 D の `tabs` は選択中「すべて」以外の 3 タブが `disabled` の静的
    /// 固定（Codex レビュー指摘 #3404 是正の回帰検知: 押せるが表の内容が
    /// 変わらない dead control にしないこと。[`status_tabs`] rustdoc
    /// 参照）。選択中「すべて」は `disabled` を持たない。
    #[test]
    fn tabs_non_selected_triggers_are_disabled() {
        let html = render(&demo());
        for value in ["paid", "unpaid", "overdue"] {
            let needle = format!(
                r#"data-value="{value}" disabled="" data-disabled="" aria-disabled="true""#
            );
            assert!(html.contains(&needle), "expected {value} trigger disabled");
        }
        assert!(
            !html.contains(r#"data-value="all" disabled="" data-disabled="" aria-disabled="true""#)
        );
        assert!(html.contains(
            r#"id="blocks-table-with-toolbar-tabs-trigger-all" role="tab" aria-selected="true""#
        ));
    }

    /// `toolbar`/`table_section`/`footer` のランドマーク名（`aria-label`）が
    /// 4 版で一意（Cursor Bugbot レビュー指摘 #3404 是正: 同一名の再利用で
    /// 支援技術上 4 つの区別不能なコピーとして列挙されないこと。検索欄は
    /// 当初 4 版とも `aria-label="請求書を検索"` を再利用していたため個別に
    /// 追加是正した）。テーブル自体にも `aria-label` が付き無名のままに
    /// ならない。
    #[test]
    fn landmark_names_are_unique_per_variant_and_table_is_named() {
        let html = render(&demo());
        for label in [
            "代表構成",
            "期間選択ボタン付き",
            "常時縦積み + エクスポート",
            "タブ型の絞り込み付き",
        ] {
            assert_eq!(
                html.matches(&format!("aria-label=\"請求書を検索（{label}）\""))
                    .count(),
                1,
                "search aria-label should be unique for {label}"
            );
            assert_eq!(
                html.matches(&format!("aria-label=\"請求書一覧表（{label}）\""))
                    .count(),
                1,
                "table aria-label should be unique for {label}"
            );
            assert_eq!(
                html.matches(&format!("aria-label=\"請求書一覧（{label}）\""))
                    .count(),
                1,
                "scroll region aria-label should be unique for {label}"
            );
            assert_eq!(
                html.matches(&format!("aria-label=\"請求書ページ（{label}）\""))
                    .count(),
                1,
                "pagination nav aria-label should be unique for {label}"
            );
        }
    }

    /// 版 B のツールバー（検索欄・期間ボタン 3 個・操作ボタン 2 個）が
    /// `@container` 横並び時に 1 行へ収まらずコンテナ幅を超えていた
    /// （Codex レビュー指摘 #3404）ため、`flex-wrap: wrap` で折り返し可能に
    /// したことの回帰検知。
    #[test]
    fn toolbar_wraps_when_row_exceeds_container_width() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains(
            "[data-blocks-table-with-toolbar-toolbar] {\n    flex-direction: row;\n    flex-wrap: wrap;"
        ));
    }

    /// 検索欄（`input_group::root`）の base CSS が `width: 100%` を持つ
    /// ため、`flex-wrap: wrap` だけでは検索欄の flex-basis が行全体を
    /// 専有し、版 A/D が常時 1 行のツールバーを失い、版 B がコンテナ幅
    /// 超過時のみ折り返すべき検索欄が常に折り返されていた（Cursor Bugbot
    /// レビュー指摘 #3404）。`[data-blocks-table-with-toolbar-search]` に
    /// 非 `auto` の `flex-basis`（`flex: 1 1 12rem`）を与え `width` の
    /// 専有を上書きしたことの回帰検知。
    #[test]
    fn search_field_has_shrinkable_flex_basis_in_row_layout() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains(
            "[data-blocks-table-with-toolbar-search] {\n    flex: 1 1 12rem;\n    min-width: 0;\n  }\n"
        ));
    }

    /// 版 C の常時縦積み CSS が `@container` 内の横並び規則を上書きする
    /// セレクタを持ち、`<` を含まない（REQ-1）。
    #[test]
    fn stacked_variant_css_overrides_container_query() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains(r#"[data-blocks-table-with-toolbar-variant="stacked"]"#));
    }

    /// 操作ボタン列（絞り込み/[エクスポート]/新規作成）が `flex-wrap: wrap`
    /// を持つ（Codex レビュー指摘 #3404 是正の回帰検知: 版 B の操作ボタン
    /// 列は `display: flex` のみでは折り返せず、Demo 枠がボタン合計幅より
    /// 狭いと常時縦積み版も含めてコンテナ幅を超えてはみ出していた。
    /// モジュール doc「B（期間選択ボタン）」節参照）。
    #[test]
    fn actions_wrap_when_row_exceeds_container_width() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains(
            "[data-blocks-table-with-toolbar-actions] {\n  display: flex;\n  flex-wrap: wrap;\n  gap: var(--fandhe-space-2);\n}\n"
        ));
    }

    /// 版 C（常時縦積み）の検索欄は `@container` 内の `flex: 1 1 12rem`
    /// （詳細度 `(0,1,0)`）が column 方向の flex コンテナ下で高さへ転用され
    /// 12rem 丈に伸びるのを、`[data-blocks-table-with-toolbar-variant=
    /// "stacked"] [data-blocks-table-with-toolbar-search]`（詳細度
    /// `(0,2,0)`）の `flex: 1 1 auto` で上書きする（Cursor Bugbot レビュー
    /// 指摘 #3404 是正の回帰検知。モジュール doc「C（常時縦積み +
    /// エクスポート）」節参照）。
    #[test]
    fn stacked_variant_search_field_flex_basis_is_reset() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains(
            "[data-blocks-table-with-toolbar-variant=\"stacked\"] [data-blocks-table-with-toolbar-search] {\n  flex: 1 1 auto;\n}\n"
        ));
    }

    /// ページ送りは 1 ページ目選択・前ページ無効の静的表示。
    #[test]
    fn pagination_is_static_first_page() {
        let html = render(&demo());
        assert!(html.contains(r#"aria-current="page""#));
        assert!(html.contains("disabled"));
    }

    /// `<form>`・`href="#"`・`data:` URI・`type="submit"` を出力しない
    /// （`crate::blocks` モジュール doc）。
    #[test]
    fn demo_has_no_form_or_unsafe_output() {
        let html = render(&demo());
        for absent in ["<form", "type=\"submit\"", "href=\"#\"", "src=\"data:"] {
            assert!(!html.contains(absent), "demo should never contain {absent}");
        }
    }

    /// [`LAYOUT_CSS`] が `demo_class`/主要 `data-*` セレクタと `@container`
    /// を含み、`<` を含まない（REQ-1: `</style>` によるスタイル脱出防止）。
    #[test]
    fn layout_css_declares_container_query_and_no_angle_bracket() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains(".blocks-table-with-toolbar-layout"));
        assert!(LAYOUT_CSS.contains("data-blocks-table-with-toolbar-scroll"));
        assert!(LAYOUT_CSS.contains("@container (min-width: 40rem)"));
    }

    /// ルート class（`demo_class` とは別名）が `demo()` の出力へ実際に
    /// 現れること。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-table-with-toolbar-layout\""));
        assert_ne!(super::BLOCK.demo_class, "blocks-table-with-toolbar-layout");
    }

    /// `parts` 11 件が全て render 出力の `data-scope` に対応する
    /// （未使用宣言の検知）。
    #[test]
    fn all_declared_parts_are_used() {
        let html = render(&demo());
        let expected_scopes = [
            "heading",
            "text",
            "input-group",
            "field",
            "button",
            "icon",
            "table",
            "badge",
            "pagination",
            "scroll-area",
            "tabs",
        ];
        assert_eq!(super::BLOCK.parts.len(), expected_scopes.len());
        for scope in expected_scopes {
            let needle = format!("data-scope=\"{scope}\"");
            assert!(html.contains(&needle), "missing {needle} in demo output");
        }
    }
}
