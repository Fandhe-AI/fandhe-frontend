//! `description-list-horizontal` block（イシュー #2906。Application/
//! Description List カテゴリ、最初の block）。主参照 R0888（左にラベル・
//! 右に値が並ぶ横並び説明リスト。見出し行の操作ボタンは R0453 相当）を
//! 中心に、R0454（各値に編集ボタン）・R0889（カード枠）・R0890（縞模様）・
//! R0891（値に添付一覧+操作リンク）の 6 件を集約する。
//!
//! # 使用部品
//!
//! `data-list` / `heading` / `text` / `button` / `card` / `attachment` /
//! `link` の 7 部品を合成する（[`BLOCK`] の `parts` に一致させる契約）。
//! 新しい UI 部品は追加しない。
//!
//! # 4 版と集約元の対応（原稿「原案差分メモ」節と対になる索引）
//!
//! - **A（代表構成）**: R0888（見出し行の一括編集ボタンは R0453 相当）+
//!   値の 1 項目として添付ファイル一覧（R0891 相当）を 1 セクションへ合成
//! - **B（値ごとの編集ボタン）**: R0454。各行の `item_value` 内へ値と
//!   `button::button`（Ghost/Sm）を並べる
//! - **C（カード枠）**: R0889。`card::root`（Outline）で全体を囲む
//! - **D（縞模様）**: R0890。`data_list::root` へ
//!   `data-blocks-description-list-horizontal-striped` を付与し、偶数行に
//!   背景色を敷く（区切り線は付けない）
//!
//! # 区切り線・縞模様は block 側 CSS が描く理由
//!
//! [`fandhe_frontend_pre_styled_ui::data_list`] は chakra-ui の `divideY`
//! （区切り線ユーティリティ）を意図的に非採用としている（同モジュール doc
//! 「意図的に追随しない点」節）。本 block は横並び説明リストとしての
//! 見た目（行間の罫線・縞模様）を求めるため、[`LAYOUT_CSS`] 側で
//! `[data-part="item"]` へ罫線・背景色を宣言する（部品自体の契約は変更
//! しない）。
//!
//! # `class` と `data-*` の使い分け
//!
//! [`fandhe_frontend_pre_styled_ui::data_list::root`]・
//! [`fandhe_frontend_pre_styled_ui::button::button`]・
//! [`fandhe_frontend_pre_styled_ui::card::root`]・
//! [`fandhe_frontend_pre_styled_ui::attachment`] 各パーツ・
//! [`fandhe_frontend_pre_styled_ui::link::root`] はいずれも
//! `drop_class_attr` で呼び出し側 `class` を除去してから内部 variant
//! クラスと合成するため、これらへの CSS フックは `data-*` 属性で渡す
//! （`data-blocks-description-list-horizontal-*`）。一方
//! `data_list::item`/`item_value`・`card::header` は `class` をそのまま
//! 通す素の `<div>` パーツのため、レイアウト用ラッパには
//! `class="blocks-description-list-horizontal-*"` を使う（`login-01`
//! codex-review 是正〔イシュー #2088〕と同型の判断）。
//!
//! # 狭い幅ではラベルを値の上に積む（`@container`）
//!
//! Demo 枠の幅はビューポート幅と一致しないため、`@container`
//! （コンテナクエリ）で判定する（`header_flyout_menu`/`app_shell_stacked`
//! と同型のパターン）。[`LAYOUT_CSS`] のラッパー
//! `.blocks-description-list-horizontal-stack` へ `container-type:
//! inline-size` を宣言し、コンテナ幅が `36rem` 未満のとき `data_list` の
//! `item`/`item-label` へ詳細度 0,3,0 の上書きを適用して縦積みへ切り替える
//! （`item` 自身の base 宣言は 0,2,0 のため、この上書きが確実に勝つ）。
//!
//! # `<form>` を使わない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない静的表示のみで、送信処理・送信先は一切持たない。
//!
//! # 添付ファイルリンクの向き先・文言・アイコンを使わない理由
//!
//! 本 Demo は実ファイルを配布しないため、添付ファイルのリンクは死リンク
//! （`href="#"`）を避けるため本リポジトリ自身（[`REPO`]）を指す（他 block
//! の慣例と同型）。リンク文言は「ダウンロード」のような実ファイル取得を
//! 期待させる語にせず、実際の遷移先どおり「リポジトリで確認」とする
//! （codex レビュー是正、イシュー #2906）。種別表示（PDF/ZIP）はモノトーン
//! の文字ラベルのみとし、参照元由来のアイコンや絵文字は持ち込まない。
//!
//! # 参照について
//!
//! 主参照は対応表 ID R0888、他 5 件は R0453/R0454/R0889/R0890/R0891。
//! 文言・配色・アイコンは独自に書く（他 block と同じライセンス上の転記
//! 制限）。デモデータ（担当者名・部署・契約情報等）は架空のものであり、
//! 実在の企業名・PII は含まない。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::attachment::{
    self, AttachmentRootProps, AttachmentState, AttachmentVariant,
};
use fandhe_frontend_pre_styled_ui::button::{button, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps, CardVariant};
use fandhe_frontend_pre_styled_ui::data_list::{self, DataListOrientation, DataListProps};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps};
use fandhe_frontend_pre_styled_ui::link;
use fandhe_frontend_pre_styled_ui::recipe::Size;
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};

/// 添付ファイルリンクの死リンク回避先（`href="#"` を使わない、他 block の
/// 慣例と同型）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// 見出し・説明文・（任意の）右端操作を横並びに束ねる。`action` を渡さない
/// 版（B）は見出し行にボタンを出さない。
fn section_header(title: &str, description: &str, action: Option<Node>) -> Node {
    let mut children = vec![div(
        vec![("class", "blocks-description-list-horizontal-header-text")],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps::default(),
                vec![],
                vec![text(title)],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    size: TextSize::Sm,
                    ..TextProps::default()
                },
                vec![],
                vec![text(description)],
            ),
        ],
    )];
    if let Some(action) = action {
        children.push(action);
    }
    div(
        vec![("class", "blocks-description-list-horizontal-header")],
        children,
    )
}

/// ラベル・値の 1 行（`data_list::item` + `item-label` + `item-value`）を
/// 組み立てる。
fn row(label: &str, value_children: Vec<Node>) -> Node {
    data_list::item(
        vec![],
        vec![
            data_list::item_label(vec![], vec![text(label)]),
            data_list::item_value(vec![], value_children),
        ],
    )
}

/// Horizontal 配置の `data_list::root` を組む。`extra_attrs` は
/// `data-blocks-description-list-horizontal-list`/`-striped` 等の CSS
/// フック用（`root` は `drop_class_attr` で `class` を除去するため）。
fn list<'a>(extra_attrs: Vec<(&'a str, &'a str)>, rows: Vec<Node>) -> Node {
    data_list::root(
        DataListProps {
            orientation: DataListOrientation::Horizontal,
            ..DataListProps::default()
        },
        extra_attrs,
        rows,
    )
}

/// 添付ファイル 1 件（種別ラベル・ファイル名・サイズ・ダウンロードリンク）
/// を組み立てる。
fn attachment_row(kind: &str, file_name: &str, size: &str) -> Node {
    attachment::root(
        AttachmentRootProps {
            variant: AttachmentVariant::File,
            state: AttachmentState::Idle,
            disabled: false,
        },
        vec![],
        vec![
            attachment::media(vec![], vec![text(kind)]),
            attachment::content(
                vec![],
                vec![
                    attachment::name(vec![], vec![text(file_name)]),
                    attachment::meta(vec![], vec![text(size)]),
                ],
            ),
            attachment::actions(
                vec![],
                vec![link::root(
                    REPO,
                    &link::LinkProps {
                        external: true,
                        ..link::LinkProps::default()
                    },
                    vec![("data-blocks-description-list-horizontal-download", "")],
                    vec![text("リポジトリで確認")],
                )],
            ),
        ],
    )
}

/// A: 代表構成（見出し行の一括編集ボタンは R0453 相当 + 添付ファイル一覧
/// は R0891 相当、主参照 R0888）。
fn version_representative() -> Node {
    let edit_button = button(
        &ButtonProps {
            variant: ButtonVariant::Outline,
            size: Size::Sm,
            ..ButtonProps::default()
        },
        vec![("data-blocks-description-list-horizontal-edit", "")],
        vec![text("編集")],
    );
    let header = section_header(
        "プロジェクト概要",
        "基本情報と進捗をまとめた説明です。",
        Some(edit_button),
    );
    let rows = vec![
        row("担当者", vec![text("佐藤 円香")]),
        row("役割", vec![text("プロジェクトリード")]),
        row("連絡先", vec![text("project-a@example.test")]),
        row("予算", vec![text("¥3,200,000")]),
        row(
            "概要",
            vec![text(
                "社内向けドキュメント基盤の刷新プロジェクト。既存資料の移行と検索性向上を目的とする。",
            )],
        ),
        row(
            "添付ファイル",
            vec![div(
                vec![(
                    "class",
                    "blocks-description-list-horizontal-attachments",
                )],
                vec![
                    attachment_row("PDF", "要件定義書.pdf", "820 KB"),
                    attachment_row("ZIP", "デザイン素材.zip", "4.1 MB"),
                ],
            )],
        ),
    ];
    div(
        vec![("class", "blocks-description-list-horizontal-section")],
        vec![
            header,
            list(
                vec![("data-blocks-description-list-horizontal-list", "")],
                rows,
            ),
        ],
    )
}

/// B: 値ごとの編集ボタン（R0454）。
fn version_per_row_action() -> Node {
    let header = section_header("連絡先情報", "各項目を個別に編集できます。", None);
    let field = |label: &str, value: &str| {
        let aria_label = format!("{label}を変更");
        row(
            label,
            vec![
                text(value),
                button(
                    &ButtonProps {
                        variant: ButtonVariant::Ghost,
                        size: Size::Sm,
                        ..ButtonProps::default()
                    },
                    vec![
                        ("data-blocks-description-list-horizontal-row-action", ""),
                        ("aria-label", aria_label.as_str()),
                    ],
                    vec![text("変更")],
                ),
            ],
        )
    };
    let rows = vec![
        field("氏名", "山田 太郎"),
        field("部署", "開発本部"),
        field("内線番号", "1234"),
    ];
    div(
        vec![("class", "blocks-description-list-horizontal-section")],
        vec![
            header,
            list(
                vec![("data-blocks-description-list-horizontal-list", "")],
                rows,
            ),
        ],
    )
}

/// C: カード枠（R0889）。
fn version_card() -> Node {
    let rows = vec![
        row("プラン", vec![text("Business")]),
        row("契約期間", vec![text("2026-04-01〜2027-03-31")]),
        row("次回更新", vec![text("2027-03-31")]),
    ];
    card::root(
        CardProps::from(CardVariant::Outline),
        vec![],
        vec![
            card::header(
                vec![("class", "blocks-description-list-horizontal-header")],
                vec![div(
                    vec![("class", "blocks-description-list-horizontal-header-text")],
                    vec![
                        heading(
                            HeadingLevel::H3,
                            &HeadingProps::default(),
                            vec![],
                            vec![text("契約情報")],
                        ),
                        styled_text::text(
                            &TextProps {
                                variant: TextVariant::Muted,
                                size: TextSize::Sm,
                                ..TextProps::default()
                            },
                            vec![],
                            vec![text("現在の契約状況です。")],
                        ),
                    ],
                )],
            ),
            card::body(
                vec![],
                vec![list(
                    vec![("data-blocks-description-list-horizontal-list", "")],
                    rows,
                )],
            ),
        ],
    )
}

/// D: 縞模様（R0890。区切り線は付けない）。
fn version_striped() -> Node {
    let header = section_header(
        "在庫サマリー",
        "偶数行に背景色を敷いて視認性を高めています。",
        None,
    );
    let rows = vec![
        row("商品コード", vec![text("SKU-2048")]),
        row("在庫数", vec![text("128 個")]),
        row("倉庫", vec![text("第 2 倉庫")]),
        row("最終棚卸日", vec![text("2026-09-01")]),
    ];
    div(
        vec![("class", "blocks-description-list-horizontal-section")],
        vec![
            header,
            list(
                vec![("data-blocks-description-list-horizontal-striped", "")],
                rows,
            ),
        ],
    )
}

/// `description-list-horizontal` の Demo 本体。呼び出しごとに同一の
/// `Node` を返す純関数。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-description-list-horizontal-stack")],
        vec![
            version_representative(),
            version_per_row_action(),
            version_card(),
            version_striped(),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/description-list-horizontal/",
    title: "description-list-horizontal",
    category: BlockCategory::DescriptionList,
    rust_source:
        "crates/docs-site/src/blocks/application/description_list/description_list_horizontal.rs",
    demo_class: "blocks-description-list-horizontal",
    parts: &[
        Part {
            label: "Data List",
            path: "/themes/data-list/",
        },
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
            label: "Card",
            path: "/themes/card/",
        },
        Part {
            label: "Attachment",
            path: "/themes/attachment/",
        },
        Part {
            label: "Link",
            path: "/themes/link/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `description_list_horizontal` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節と同型）。
const LAYOUT_CSS: &str = "\
.blocks-description-list-horizontal-stack {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-10);\n  container-type: inline-size;\n  container-name: blocks-description-list-horizontal;\n}\n\
.blocks-description-list-horizontal-header {\n  display: flex;\n  flex-wrap: wrap;\n  align-items: flex-start;\n  justify-content: space-between;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-description-list-horizontal-header-text {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1);\n}\n\
.blocks-description-list-horizontal-attachments {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n  flex: 1;\n  min-width: 0;\n}\n\
[data-blocks-description-list-horizontal-list] {\n  --fandhe-data-list-gap: 0;\n}\n\
[data-blocks-description-list-horizontal-list] > [data-scope=\"data-list\"][data-part=\"item\"] {\n  border-top: 1px solid var(--fandhe-color-border);\n  padding-block: var(--fandhe-space-4);\n}\n\
[data-blocks-description-list-horizontal-striped] {\n  --fandhe-data-list-gap: 0;\n}\n\
[data-blocks-description-list-horizontal-striped] > [data-scope=\"data-list\"][data-part=\"item\"] {\n  padding-block: var(--fandhe-space-3);\n  padding-inline: var(--fandhe-space-3);\n}\n\
[data-blocks-description-list-horizontal-striped] > [data-scope=\"data-list\"][data-part=\"item\"]:nth-child(even) {\n  background: var(--fandhe-color-bg-muted);\n}\n\
[data-blocks-description-list-horizontal-row-action] {\n  margin-inline-start: auto;\n}\n\
@container blocks-description-list-horizontal (max-width: 36rem) {\n  \
.blocks-description-list-horizontal-stack [data-scope=\"data-list\"][data-part=\"item\"] {\n    flex-direction: column;\n    gap: var(--fandhe-space-1);\n  }\n  \
.blocks-description-list-horizontal-stack [data-scope=\"data-list\"][data-part=\"item-label\"] {\n    min-width: auto;\n  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    fn demo_html() -> String {
        render(&demo())
    }

    #[test]
    fn demo_composes_expected_parts() {
        let html = demo_html();
        for scope in [
            "data-scope=\"data-list\"",
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"button\"",
            "data-scope=\"card\"",
            "data-scope=\"attachment\"",
            "data-scope=\"link\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
        assert_eq!(html.matches("<h3").count(), 4);
        assert_eq!(
            html.matches("fd-data-list--orientation-horizontal").count(),
            4
        );
        assert_eq!(
            html.matches("data-blocks-description-list-horizontal-striped=\"\"")
                .count(),
            1
        );
    }

    #[test]
    fn no_form_submit_or_dead_links() {
        let html = demo_html();
        assert!(!html.contains("<form"));
        assert!(!html.contains("src=\"data:"));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("<script"));
        assert!(!html.contains("type=\"submit\""));
        assert!(html.contains(r#"href="https://github.com/Fandhe-AI/fandhe-frontend""#));
    }

    #[test]
    fn layout_css_is_safe_and_stacks_on_narrow_container() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("container-type: inline-size;"));
        assert!(
            LAYOUT_CSS.contains("@container blocks-description-list-horizontal (max-width: 36rem)")
        );
    }

    /// codex-review 是正（イシュー #2906）: 版 B の各行「変更」ボタンが
    /// すべて同名の `aria-label` なしボタンとして出力され、支援技術の
    /// ボタン一覧で対象項目を区別できなかった不具合。`field` へラベルを
    /// 渡し `<label>を変更` の項目固有アクセシブル名を付与したことを固定する。
    #[test]
    fn per_row_action_buttons_have_item_specific_accessible_names() {
        let html = demo_html();
        for expected in ["氏名を変更", "部署を変更", "内線番号を変更"] {
            assert!(
                html.contains(&format!(r#"aria-label="{expected}""#)),
                "missing accessible name: {expected}"
            );
        }
    }

    /// codex-review 是正（見出しと補足説明の間に余白がなく密着していた不具合）。
    /// `heading`/`text` はいずれも `margin: 0` にリセットするため、ラッパー側
    /// （`.blocks-description-list-horizontal-header-text`）が `gap` を持つ
    /// ことを固定する。
    #[test]
    fn header_text_wrapper_has_spacing() {
        assert!(LAYOUT_CSS
            .contains(".blocks-description-list-horizontal-header-text {\n  display: flex;\n  flex-direction: column;\n  gap:"));
    }
}
