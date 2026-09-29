//! `table-with-heading` block（イシュー #2947。親トラッキング #2892
//! 「Blocks 目的別パーツ拡充」配下）。Application / Table カテゴリ最初の
//! block（本カテゴリの雛形卒業は本ファイルと同一 PR、`super`（`table/mod.rs`）
//! 参照）。上部に表題・説明・右端の「追加」ボタン、その下に一覧テーブルを
//! 置く基本形を、既存部品（`heading` / `text` / `button` / `table` /
//! `card` / `badge` / `link` の 7 部品）のみの合成で示す。
//! `_/blocks-intake/` の対応ファイルは本イシュー着手時点で本 worktree に
//! 存在しないため、原稿・本コメントには対応表 ID のみを記す
//! （`page_heading_meta.rs`〔イシュー #2933〕と同じ扱い）。
//!
//! # 対応表 ID の畳み込み方（10 インスタンスで見せ方の違いを併記する）
//!
//! 主参照 R1319（代表構成）に、集約元 10 件（R1320〜R1330・R1334）を
//! 同一骨格（[`instance`] ヘルパ 1 本、J のみ [`plans_instance`]）の
//! 静的併記で表現する。狭幅では役職・メール列（副次列）を非表示にする
//! （R1326、全インスタンス共通、下記「狭幅では副次列を隠す」節参照）。
//!
//! - **A（R1319・代表構成）**: 既定 variant・既定 size。
//! - **B（R1320・カード枠）**: [`fandhe_frontend_pre_styled_ui::card`] で
//!   表題行 + 表を包む。
//! - **C（R1321・全幅）**: フレームの左右 padding を 0 にする。
//! - **D（R1322・全幅 + 内容最大幅）**: C に加え表題行だけ最大幅で中央寄せ
//!   する。
//! - **E（R1323・縞模様）**: `TableProps::striped`。
//! - **F（R1324・小型大文字見出し）**: 列見出しへ独自フックで
//!   `text-transform: uppercase` を適用する。
//! - **G（R1328・固定ヘッダー）**: `TableProps::sticky_header` +
//!   `scroll_area` の最大高でスクロールさせる（行数を増やして確実に
//!   はみ出させる）。
//! - **H（R1329・縦罫線）**: 列間に `border-inline-start` を独自フックで
//!   足す（`table` 部品自体は列区切り線を持たないため、モジュール doc
//!   「縦罫線は block 固有 CSS で表現する」節参照）。
//! - **I（R1330・詰めた行間）**: `TableProps::size` に `Size::Sm`。
//! - **J（R1334・外枠 + 現在プラン + 選択ボタン）**: [`plans_instance`]
//!   （下記参照）。
//!
//! # J: 外枠 + 現在プラン + 選択ボタン（R1334）
//!
//! `TableVariant::Outline`。列はプラン/月額/上限/状態/操作。現在プラン行
//! のみ状態列に `badge`（`Solid`）を置き、操作列のボタンは
//! `disabled: true` にする（他行は `Outline` variant の通常ボタン）。
//! `data-selected` は使わない（静的表示のため、行状態の永続化は行わない）。
//! 月額・上限列は「狭幅では副次列を隠す」節の対象外（下記参照）。
//! 表題行の右端ボタンは持たない（プラン選択は行ごとの「選択」ボタンが
//! 担うため、A〜I の「追加」ボタンに相当する操作がない）。
//!
//! # 狭幅では副次列を隠す（R1326、A〜I 共通。J は対象外）
//!
//! 役職・メール列（`column_header`/`cell`）へ
//! `data-blocks-table-with-heading-secondary` を付与し、[`LAYOUT_CSS`] が
//! `40rem` 未満で `display: none` にする（既存 block と同じ `40rem`
//! リテラル。テーマの breakpoint トークンは `@media` 条件式の中では解決
//! できない）。J（[`plans_instance`]）の月額・上限列は料金・利用上限と
//! いう主要情報であり隠すと「選択」ボタンだけが残ってしまうため、この
//! 副次列扱いにしない（狭幅では折り返し表示のまま維持する）。
//!
//! # 縦罫線は block 固有 CSS で表現する
//!
//! [`fandhe_frontend_pre_styled_ui::table`] は列間の縦罫線（chakra-ui
//! `showColumnBorder` 相当）を実装していない（`table.rs` モジュール doc
//! 「variant について」節、意図的にスコープ外）。本 block はこの見せ方
//! （H）を `column_header`/`cell` への独自フック
//! （`data-blocks-table-with-heading-rule`）+ `border-inline-start` で
//! 表現し、`table` 部品自体への機能追加は行わない
//! （`.claude/rules/out-of-scope-tracking.md` 対応、モジュール doc
//! 「スコープ外」節参照）。列間の区切りを表すため先頭列（名前）には
//! 付けず、2 列目以降にのみ付与する（先頭列に付けると存在しない左端の
//! 区切り線が描画されてしまう）。
//!
//! # `<form>` を持たない・データ取得/送信を行わない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である。「追加」「選択」ボタンは `button::button` の既定 `type="button"`
//! のまま送信先を持たない（`docs/policy/intentional-non-adoption.md`
//! §3.25：バリデーション・送信処理は UI コンポーネント層の責務外）。
//! 行末の操作リンクは既存 block と同じリポジトリ URL 定数へ張る
//! （`href="#"` は使わない）。氏名・メールは架空のダミー（`example.com`、
//! 実企業名・実クレデンシャル・PII を含まない）。
//!
//! # スコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! `table` 部品への縦罫線（`showColumnBorder` 相当）機能追加は本イシューの
//! スコープ外（上記「縦罫線は block 固有 CSS で表現する」節参照）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, span, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps, BadgeVariant};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card;
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::recipe::ColorPalette;
use fandhe_frontend_pre_styled_ui::table::{self, TableProps, TableVariant};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextVariant};
use fandhe_frontend_pre_styled_ui::Size;

/// 既存 block と同じ架空リンク先（`href="#"` を使わないための定数）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// [`dummy_assets::PERSON_NAMES`] と対にした架空メールアドレス（`example.com`。
/// 実ドメイン・実クレデンシャルを含まない）。
const MEMBER_EMAILS: &[&str] = &[
    "haruto.f@example.com",
    "elena.v@example.com",
    "kwame.b@example.com",
    "mei.l@example.com",
    "noor.a@example.com",
    "ola.b@example.com",
    "priya.c@example.com",
    "theo.m@example.com",
];

/// 状態列に使う 3 状態（ラベル + palette）。行番号で周回させる。
const STATUSES: &[(&str, ColorPalette)] = &[
    ("稼働中", ColorPalette::Success),
    ("招待中", ColorPalette::Warning),
    ("停止", ColorPalette::Danger),
];

/// このインスタンスの見せ方を宣言するデータ（[`VARIANTS`] から駆動する）。
struct Variant {
    label: &'static str,
    card: bool,
    full_bleed: bool,
    content_max: bool,
    striped: bool,
    caps_header: bool,
    sticky: bool,
    rules: bool,
    size: Size,
    /// 行数（既定 5。sticky ヘッダーの実演〔G〕のみスクロールを起こすため
    /// 8 に増やす）。
    rows: usize,
}

const DEFAULT_ROWS: usize = 5;

const VARIANTS: &[Variant] = &[
    Variant {
        label: "代表構成",
        card: false,
        full_bleed: false,
        content_max: false,
        striped: false,
        caps_header: false,
        sticky: false,
        rules: false,
        size: Size::Md,
        rows: DEFAULT_ROWS,
    },
    Variant {
        label: "カード枠",
        card: true,
        full_bleed: false,
        content_max: false,
        striped: false,
        caps_header: false,
        sticky: false,
        rules: false,
        size: Size::Md,
        rows: DEFAULT_ROWS,
    },
    Variant {
        label: "全幅",
        card: false,
        full_bleed: true,
        content_max: false,
        striped: false,
        caps_header: false,
        sticky: false,
        rules: false,
        size: Size::Md,
        rows: DEFAULT_ROWS,
    },
    Variant {
        label: "全幅＋内容最大幅",
        card: false,
        full_bleed: true,
        content_max: true,
        striped: false,
        caps_header: false,
        sticky: false,
        rules: false,
        size: Size::Md,
        rows: DEFAULT_ROWS,
    },
    Variant {
        label: "縞模様",
        card: false,
        full_bleed: false,
        content_max: false,
        striped: true,
        caps_header: false,
        sticky: false,
        rules: false,
        size: Size::Md,
        rows: DEFAULT_ROWS,
    },
    Variant {
        label: "小型大文字見出し",
        card: false,
        full_bleed: false,
        content_max: false,
        striped: false,
        caps_header: true,
        sticky: false,
        rules: false,
        size: Size::Md,
        rows: DEFAULT_ROWS,
    },
    Variant {
        label: "固定ヘッダー",
        card: false,
        full_bleed: false,
        content_max: false,
        striped: false,
        caps_header: false,
        sticky: true,
        rules: false,
        size: Size::Md,
        rows: 8,
    },
    Variant {
        label: "縦罫線",
        card: false,
        full_bleed: false,
        content_max: false,
        striped: false,
        caps_header: false,
        sticky: false,
        rules: true,
        size: Size::Md,
        rows: DEFAULT_ROWS,
    },
    Variant {
        label: "詰めた行間",
        card: false,
        full_bleed: false,
        content_max: false,
        striped: false,
        caps_header: false,
        sticky: false,
        rules: false,
        size: Size::Sm,
        rows: DEFAULT_ROWS,
    },
];

/// 列見出し（副次列は `data-blocks-table-with-heading-secondary` を持つ）。
/// 縦罫線（`rule`）は列間の区切りを表すため先頭列には付けない
/// （先頭列に付けると存在しない左端の区切り線が描画されてしまう）。
fn column_headers(caps: bool, rules: bool) -> Node {
    let attrs_for = |secondary: bool, rule: bool| -> Vec<(&'static str, &'static str)> {
        let mut a = Vec::new();
        if secondary {
            a.push(("data-blocks-table-with-heading-secondary", ""));
        }
        if caps {
            a.push(("data-blocks-table-with-heading-caps", ""));
        }
        if rule {
            a.push(("data-blocks-table-with-heading-rule", ""));
        }
        a
    };
    table::row(
        vec![],
        vec![
            table::column_header(attrs_for(false, false), vec![text("名前")]),
            table::column_header(attrs_for(true, rules), vec![text("役職")]),
            table::column_header(attrs_for(true, rules), vec![text("メール")]),
            table::column_header(attrs_for(false, rules), vec![text("状態")]),
            table::column_header(
                attrs_for(false, rules),
                vec![span(
                    vec![("data-blocks-table-with-heading-sr-only", "")],
                    vec![text("編集")],
                )],
            ),
        ],
    )
}

/// 本文 1 行（`row_index` で氏名・メール・状態を周回させる）。
/// 縦罫線（`rule_attr`）は列間の区切りを表すため先頭列（名前）には付けない
/// （`column_headers` と同じ理由）。
fn body_row(row_index: usize, rules: bool) -> Node {
    let name_index = row_index % dummy_assets::PERSON_NAMES.len();
    let job_index = row_index % dummy_assets::JOB_TITLES.len();
    let (status_label, status_palette) = STATUSES[row_index % STATUSES.len()];
    let rule_attr: Vec<(&str, &str)> = if rules {
        vec![("data-blocks-table-with-heading-rule", "")]
    } else {
        vec![]
    };
    let mut role_attr = rule_attr.clone();
    role_attr.push(("data-blocks-table-with-heading-secondary", ""));
    let mut email_attr = rule_attr.clone();
    email_attr.push(("data-blocks-table-with-heading-secondary", ""));
    table::row(
        vec![],
        vec![
            table::cell(vec![], vec![text(dummy_assets::PERSON_NAMES[name_index])]),
            table::cell(role_attr, vec![text(dummy_assets::JOB_TITLES[job_index])]),
            table::cell(email_attr, vec![text(MEMBER_EMAILS[name_index])]),
            table::cell(
                rule_attr.clone(),
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
            table::cell(
                rule_attr,
                vec![link::root(
                    REPO,
                    &LinkProps::default(),
                    vec![],
                    vec![text("編集")],
                )],
            ),
        ],
    )
}

/// A〜I の 1 インスタンス（表題 + 「追加」ボタン + 表）。
fn instance(v: &Variant) -> Node {
    let heading_row = div(
        vec![("data-blocks-table-with-heading-header", "")],
        vec![
            div(
                vec![],
                vec![
                    heading(
                        HeadingLevel::H4,
                        &HeadingProps {
                            size: HeadingSize::Lg,
                            ..HeadingProps::default()
                        },
                        vec![],
                        vec![text("メンバー一覧")],
                    ),
                    styled_text::text(
                        &fandhe_frontend_pre_styled_ui::text::TextProps {
                            variant: TextVariant::Muted,
                            ..Default::default()
                        },
                        vec![],
                        vec![text("チームに参加しているメンバーの一覧です。")],
                    ),
                ],
            ),
            button::button(
                &ButtonProps::default(),
                vec![],
                vec![text("メンバーを追加")],
            ),
        ],
    );
    let heading_row = if v.content_max {
        div(
            vec![("data-blocks-table-with-heading-content-max", "")],
            vec![heading_row],
        )
    } else {
        heading_row
    };

    let rows: Vec<Node> = (0..v.rows).map(|i| body_row(i, v.rules)).collect();
    let table_node = table::root(
        TableProps {
            variant: TableVariant::Line,
            size: v.size,
            striped: v.striped,
            sticky_header: v.sticky,
            ..TableProps::default()
        },
        vec![],
        vec![
            table::header(vec![], vec![column_headers(v.caps_header, v.rules)]),
            table::body(vec![], rows),
        ],
    );
    let mut scroll_attrs: Vec<(&str, &str)> = vec![
        ("role", "region"),
        ("aria-label", "メンバー一覧テーブル"),
        ("tabindex", "0"),
    ];
    if v.sticky {
        scroll_attrs.push(("data-blocks-table-with-heading-scroll", "sticky"));
    }
    let scroll = table::scroll_area(scroll_attrs, vec![table_node]);

    let body = if v.card {
        card::root(
            card::CardProps::default(),
            vec![],
            vec![
                card::header(vec![], vec![heading_row]),
                card::body(vec![], vec![scroll]),
            ],
        )
    } else {
        div(
            vec![("data-blocks-table-with-heading-body", "")],
            vec![heading_row, scroll],
        )
    };

    let mut frame_attrs: Vec<(&str, &str)> = vec![("data-blocks-table-with-heading-frame", "")];
    if v.full_bleed {
        frame_attrs.push(("data-blocks-table-with-heading-full-bleed", ""));
    }

    div(
        vec![("data-blocks-table-with-heading-instance", "")],
        vec![
            el(
                "h3",
                vec![("data-blocks-table-with-heading-section-title", "")],
                vec![text(v.label)],
            ),
            div(frame_attrs, vec![body]),
        ],
    )
}

/// J: 外枠 + 現在プラン + 選択ボタン（R1334）。他インスタンスと骨格が
/// 異なるため専用に組み立てる（モジュール doc「J」節参照）。
fn plans_instance() -> Node {
    const PLANS: &[(&str, &str, &str, bool)] = &[
        ("Starter", "¥0 / 月", "3 プロジェクト", false),
        ("Pro", "¥2,980 / 月", "無制限", true),
        ("Business", "¥9,800 / 月", "無制限 + 監査ログ", false),
    ];
    let heading_row = div(
        vec![("data-blocks-table-with-heading-header", "")],
        vec![div(
            vec![],
            vec![
                heading(
                    HeadingLevel::H4,
                    &HeadingProps {
                        size: HeadingSize::Lg,
                        ..HeadingProps::default()
                    },
                    vec![],
                    vec![text("プラン一覧")],
                ),
                styled_text::text(
                    &fandhe_frontend_pre_styled_ui::text::TextProps {
                        variant: TextVariant::Muted,
                        ..Default::default()
                    },
                    vec![],
                    vec![text("チームの利用状況に合わせてプランを選べます。")],
                ),
            ],
        )],
    );
    let rows: Vec<Node> = PLANS
        .iter()
        .map(|(name, price, limit, current)| {
            let status_cell = if *current {
                table::cell(
                    vec![],
                    vec![badge::badge(
                        &BadgeProps {
                            variant: BadgeVariant::Solid,
                            palette: ColorPalette::Accent,
                            ..BadgeProps::default()
                        },
                        vec![],
                        vec![text("現在のプラン")],
                    )],
                )
            } else {
                table::cell(vec![], vec![])
            };
            table::row(
                vec![],
                vec![
                    table::cell(vec![], vec![text(*name)]),
                    table::cell(vec![], vec![text(*price)]),
                    table::cell(vec![], vec![text(*limit)]),
                    status_cell,
                    table::cell(
                        vec![],
                        vec![button::button(
                            &ButtonProps {
                                variant: ButtonVariant::Outline,
                                size: Size::Sm,
                                disabled: *current,
                                ..ButtonProps::default()
                            },
                            vec![],
                            vec![text("選択")],
                        )],
                    ),
                ],
            )
        })
        .collect();
    let table_node = table::root(
        TableProps {
            variant: TableVariant::Outline,
            ..TableProps::default()
        },
        vec![],
        vec![
            table::header(
                vec![],
                vec![table::row(
                    vec![],
                    vec![
                        table::column_header(vec![], vec![text("プラン")]),
                        table::column_header(vec![], vec![text("月額")]),
                        table::column_header(vec![], vec![text("上限")]),
                        table::column_header(vec![], vec![text("状態")]),
                        table::column_header(
                            vec![],
                            vec![span(
                                vec![("data-blocks-table-with-heading-sr-only", "")],
                                vec![text("操作")],
                            )],
                        ),
                    ],
                )],
            ),
            table::body(vec![], rows),
        ],
    );
    let scroll = table::scroll_area(
        vec![
            ("role", "region"),
            ("aria-label", "プラン一覧テーブル"),
            ("tabindex", "0"),
        ],
        vec![table_node],
    );
    div(
        vec![("data-blocks-table-with-heading-instance", "")],
        vec![
            el(
                "h3",
                vec![("data-blocks-table-with-heading-section-title", "")],
                vec![text("外枠＋現在プラン＋選択ボタン")],
            ),
            div(
                vec![("data-blocks-table-with-heading-frame", "")],
                vec![div(
                    vec![("data-blocks-table-with-heading-body", "")],
                    vec![heading_row, scroll],
                )],
            ),
        ],
    )
}

/// `table-with-heading` の Demo 本体（A〜I + J の 10 インスタンスを縦積みで
/// 並記する。呼び出しごとに同一の `Node` を返す純関数）。
pub fn demo() -> Node {
    let mut children: Vec<Node> = VARIANTS.iter().map(instance).collect();
    children.push(plans_instance());
    div(
        vec![("class", "blocks-table-with-heading-layout")],
        children,
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/table-with-heading/",
    title: "table-with-heading",
    category: BlockCategory::Table,
    rust_source: "crates/docs-site/src/blocks/application/table/table_with_heading.rs",
    demo_class: "blocks-table-with-heading",
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
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Table",
            path: "/themes/table/",
        },
        Part {
            label: "Card",
            path: "/themes/card/",
        },
        Part {
            label: "Badge",
            path: "/themes/badge/",
        },
        Part {
            label: "Link",
            path: "/themes/link/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `table_with_heading` 固有のレイアウト規則。セレクタは
/// `.blocks-table-with-heading-*` と `[data-blocks-table-with-heading-*]`
/// のみを用いる。ルート class を `demo_class` と別名にする（既存 block と
/// 同じ Bugbot 教訓の回避）。
const LAYOUT_CSS: &str = "\
.blocks-table-with-heading-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n}\n\
[data-blocks-table-with-heading-instance] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n}\n\
[data-blocks-table-with-heading-section-title] {\n  color: var(--fandhe-color-fg-muted);\n  font-size: var(--fandhe-font-font-size-sm);\n  font-weight: var(--fandhe-font-font-weight-medium);\n}\n\
[data-blocks-table-with-heading-frame] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n  padding-inline: var(--fandhe-space-4);\n}\n\
[data-blocks-table-with-heading-body] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
[data-blocks-table-with-heading-full-bleed] {\n  padding-inline: 0;\n}\n\
[data-blocks-table-with-heading-content-max] {\n  max-width: 40rem;\n  margin-inline: auto;\n}\n\
[data-blocks-table-with-heading-header] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n}\n\
[data-blocks-table-with-heading-sr-only] {\n  position: absolute;\n  width: 1px;\n  height: 1px;\n  overflow: hidden;\n  clip: rect(0, 0, 0, 0);\n  white-space: nowrap;\n}\n\
[data-blocks-table-with-heading-secondary] {\n  display: none;\n}\n\
[data-blocks-table-with-heading-caps] {\n  font-size: var(--fandhe-font-font-size-xs);\n  text-transform: uppercase;\n  letter-spacing: 0.05em;\n}\n\
[data-blocks-table-with-heading-rule] {\n  border-inline-start: 1px solid var(--fandhe-color-border);\n}\n\
[data-blocks-table-with-heading-scroll=\"sticky\"] {\n  max-height: 14rem;\n  overflow-y: auto;\n}\n\
@media (min-width: 40rem) {\n  [data-blocks-table-with-heading-header] {\n    flex-direction: row;\n    justify-content: space-between;\n    align-items: flex-end;\n  }\n  [data-blocks-table-with-heading-secondary] {\n    display: table-cell;\n  }\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// Demo が使用部品（heading/text/button/table/card/badge/link）の
    /// anatomy をすべて実際に出力していること。
    #[test]
    fn demo_composes_expected_parts() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"button\"",
            "data-scope=\"table\"",
            "data-scope=\"card\"",
            "data-scope=\"badge\"",
            "data-scope=\"link\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
    }

    /// インスタンス数はちょうど 10 件（A〜I + J）。
    #[test]
    fn demo_instance_count() {
        let html = render(&demo());
        assert_eq!(
            html.matches("data-blocks-table-with-heading-instance")
                .count(),
            10
        );
    }

    /// 副次列（役職/メール、J は月額/上限）が header・body 双方に現れる。
    #[test]
    fn secondary_columns_are_marked() {
        let html = render(&demo());
        assert!(
            html.matches("data-blocks-table-with-heading-secondary")
                .count()
                >= 4
        );
    }

    /// 詰めた行間（`Size::Sm`）・縞模様・固定ヘッダーの出力を含む。
    #[test]
    fn sticky_and_striped_and_sm_variants_present() {
        let html = render(&demo());
        assert!(html.contains("fd-table--size-sm"));
        assert!(html.contains("fd-table--striped-true"));
        assert!(html.contains("fd-table--sticky-header-true"));
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

    /// [`LAYOUT_CSS`] が想定するブレークポイントを持ち、`<` を含まない
    /// （REQ-1: `</style>` によるスタイル脱出を防ぐ）。
    #[test]
    fn layout_css_declares_breakpoint_and_no_angle_bracket() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("@media (min-width: 40rem)"));
    }

    /// ルート class（`demo_class` とは別名）が `demo()` の出力へ実際に
    /// 現れること。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-table-with-heading-layout\""));
        assert_ne!(super::BLOCK.demo_class, "blocks-table-with-heading-layout");
    }
}
