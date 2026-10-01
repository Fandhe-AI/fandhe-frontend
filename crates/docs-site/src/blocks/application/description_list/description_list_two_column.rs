//! `description-list-two-column` block（イシュー #2908。親トラッキング
//! #2734/#2733）。
//!
//! # 使用部品
//!
//! `data-list` / `heading` / `text` / `attachment` / `button` の 5 部品を
//! 合成する（[`BLOCK`] の `parts` に一致させる契約）。新しい UI 部品は
//! 追加しない。
//!
//! # レイアウト仕様
//!
//! 各項目はラベルを値の上に積む（[`data_list::DataListOrientation::Vertical`]）。
//! 項目は 2 列グリッドに並べ、長文の項目（概要）と添付ファイル一覧は
//! `data-blocks-description-list-two-column-span="full"` を付けて 2 列を
//! またぐ。狭い幅（40rem 未満）では 1 列に戻す（[`LAYOUT_CSS`] 参照）。
//!
//! # 2 インスタンス併記（集約元との対応）
//!
//! docs サイトは JS ハイドレーションを行わない設計（CLAUDE.md）のため、
//! 集約元 2 件の違いを 1 ページに縦へ併記する。インスタンス A（主参照
//! R0893）は見出し + リード文のみ、インスタンス B（集約元 R0456）は見出し
//! 行の右側に操作ボタンを持つ。共通部分は `instance(with_action: bool)`
//! に 1 本化し、新しい抽象は増やさない。
//!
//! # CSS フックに data 属性を使う理由
//!
//! `data_list::root` / `heading::heading` / `text::text` /
//! `attachment::root` / `button::button` はいずれも `drop_class_attr`
//! （または headless 側の同型処理）により呼び出し側 `attrs` の `class` を
//! 黙って除去するため、Demo 固有のスタイルフックは
//! `data-blocks-description-list-two-column-*` 属性で渡す（`team-bio-rows`
//! と同じ判断）。素の `div`/`ul`/`li`/`p` には `class` をそのまま使う。
//!
//! # アイコン・`id`・`href` を持たない理由
//!
//! アイコン部品は使用部品を Issue 指定の 5 件に揃えるため使わない。`id`
//! 属性は付けず ARIA 参照の宙づりを避ける（`demo_output_has_no_dangling_
//! aria_references_or_duplicate_ids` 対応）。`href` を持つ部品（リンク）は
//! 使わないため `linkcheck` の死リンク対象も生じない。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, li, p, text, ul, Node};
use fandhe_frontend_pre_styled_ui::attachment::{self, AttachmentRootProps};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::data_list::{self, DataListOrientation, DataListProps};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::Size;

/// 概要欄の架空の紹介文（検索インデックス容量対策で 2〜3 文に抑える）。
const SUMMARIES: &[&str] = &[
    "在宅勤務を中心とした働き方への切り替えを希望しています。\
     チーム内の定例は既存のオンライン会議のまま継続する想定です。",
    "隣接チームとの兼務を解消し、現チームの業務に専念したいという申請です。\
     引き継ぎ期間として 2 週間を見込んでいます。",
];

/// 添付ファイル 1 件（`media` に拡張子表記、`content` に名前・サイズ、
/// `actions` にダウンロードボタンを置く）。
fn attachment_item(extension: &str, name: &str, size: &str) -> Node {
    li(
        vec![],
        vec![attachment::root(
            AttachmentRootProps::default(),
            vec![],
            vec![
                attachment::media(
                    vec![],
                    vec![styled_text::text(
                        &TextProps {
                            size: TextSize::Xs,
                            variant: TextVariant::Muted,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text(extension)],
                    )],
                ),
                attachment::content(
                    vec![],
                    vec![
                        attachment::name(vec![], vec![text(name)]),
                        attachment::meta(vec![], vec![text(size)]),
                    ],
                ),
                attachment::actions(
                    vec![],
                    vec![button::button(
                        &ButtonProps {
                            variant: ButtonVariant::Ghost,
                            size: Size::Sm,
                            ..ButtonProps::default()
                        },
                        vec![],
                        vec![text("ダウンロード")],
                    )],
                ),
            ],
        )],
    )
}

/// 半幅の項目（ラベル + 値）1 件。
fn field(label: &str, value: &str) -> Node {
    data_list::item(
        vec![],
        vec![
            data_list::item_label(vec![], vec![text(label)]),
            data_list::item_value(vec![], vec![text(value)]),
        ],
    )
}

/// 全幅の項目（`data-blocks-description-list-two-column-span="full"` 付き）。
fn field_full(label: &str, value_children: Vec<Node>) -> Node {
    data_list::item(
        vec![("data-blocks-description-list-two-column-span", "full")],
        vec![
            data_list::item_label(vec![], vec![text(label)]),
            data_list::item_value(vec![], value_children),
        ],
    )
}

/// [`dummy_assets::PERSON_NAMES`] の氏名から架空メールアドレスを導出する
/// （空白をピリオドへ・小文字化。実在するメールとの一致を避けるための
/// 機械的な変換であり、氏名との対応が取れた実例にする）。
fn applicant_email(name: &str) -> String {
    format!("{}@example.com", name.to_lowercase().replace(' ', "."))
}

/// インスタンス 1 件分（`index` で人物・文言をずらす。`with_action` で
/// 見出し行の右側に操作ボタンを持つか切り替える、モジュール doc「2
/// インスタンス併記」参照）。
fn instance(index: usize, with_action: bool) -> Node {
    let applicant = dummy_assets::PERSON_NAMES[index];
    let department = dummy_assets::JOB_TITLES[index % dummy_assets::JOB_TITLES.len()];
    let email = applicant_email(applicant);
    let start_date = "2026-11-01";
    let summary = SUMMARIES[index % SUMMARIES.len()];

    let mut header_children: Vec<Node> = vec![div(
        vec![("data-blocks-description-list-two-column-header-text", "")],
        vec![
            heading::heading(
                HeadingLevel::H3,
                &HeadingProps::default(),
                vec![],
                vec![text("申請内容")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text("勤務条件の変更申請です。")],
            ),
        ],
    )];
    // with_action が false のときは第 2 子（ボタン）を挿入しない。空の
    // div を置くと header の `gap` により見出し下へ余分な隙間が生じるため
    // （Cursor Bugbot 指摘、PR #3350）。
    if with_action {
        header_children.push(button::button(
            &ButtonProps {
                variant: ButtonVariant::Outline,
                size: Size::Sm,
                ..ButtonProps::default()
            },
            vec![],
            vec![text("編集")],
        ));
    }

    div(
        vec![("data-blocks-description-list-two-column-layout", "")],
        vec![
            div(
                vec![("data-blocks-description-list-two-column-header", "")],
                header_children,
            ),
            data_list::root(
                DataListProps {
                    orientation: DataListOrientation::Vertical,
                    ..DataListProps::default()
                },
                vec![("data-blocks-description-list-two-column-list", "")],
                vec![
                    field("申請者", applicant),
                    field("所属", department),
                    field("連絡先", &email),
                    field("希望開始日", start_date),
                    field_full("概要", vec![text(summary)]),
                    field_full(
                        "添付ファイル",
                        vec![ul(
                            vec![("data-blocks-description-list-two-column-attachments", "")],
                            vec![
                                attachment_item("PDF", "勤務条件変更申請書.pdf", "1.2 MB"),
                                attachment_item("PNG", "現行シフト表.png", "480 KB"),
                            ],
                        )],
                    ),
                ],
            ),
        ],
    )
}

/// caption（並記された各インスタンスの見出し）。
fn caption(label: &str) -> Node {
    p(
        vec![("data-blocks-description-list-two-column-caption", "")],
        vec![text(label)],
    )
}

/// `description-list-two-column` の Demo 本体（2 インスタンス併記）。
pub fn demo() -> Node {
    div(
        vec![("data-blocks-description-list-two-column-stack", "")],
        vec![
            caption("2 列の説明リスト（主参照 R0893）"),
            instance(0, false),
            caption("見出し行に操作ボタン（集約元 R0456）"),
            instance(1, true),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/description-list-two-column/",
    title: "description-list-two-column",
    category: BlockCategory::DescriptionList,
    rust_source:
        "crates/docs-site/src/blocks/application/description_list/description_list_two_column.rs",
    demo_class: "blocks-description-list-two-column",
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
            label: "Attachment",
            path: "/themes/attachment/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `description_list_two_column` 固有のレイアウト規則（`--fandhe-*`
/// トークンのみ使用）。セレクタは `.blocks-description-list-two-column-*`
/// と `[data-blocks-description-list-two-column-*]` に限定する。既定
/// （狭幅）はラベル下値積み・1 列のリストにし、40rem 以上でヘッダー行を
/// 横並びへ、リストを 2 列グリッドへ切り替える。全幅項目
/// （`data-blocks-description-list-two-column-span="full"`）は 2 列を
/// またぐ。
const LAYOUT_CSS: &str = "\
[data-blocks-description-list-two-column-stack] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n}\n\
[data-blocks-description-list-two-column-caption] {\n  margin: 0;\n  font-size: var(--fandhe-font-font-size-sm, 0.875rem);\n  color: var(--fandhe-color-fg-muted);\n}\n\
[data-blocks-description-list-two-column-layout] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n}\n\
[data-blocks-description-list-two-column-header] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n}\n\
[data-blocks-description-list-two-column-header-text] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n}\n\
[data-scope=\"data-list\"][data-part=\"root\"][data-blocks-description-list-two-column-list] {\n  display: grid;\n  grid-template-columns: 1fr;\n  gap: var(--fandhe-space-4) var(--fandhe-space-6);\n}\n\
[data-blocks-description-list-two-column-attachments] {\n  list-style: none;\n  margin: 0;\n  padding: 0;\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n}\n\
@media (min-width: 40rem) {\n  \
[data-blocks-description-list-two-column-header] {\n    flex-direction: row;\n    justify-content: space-between;\n    align-items: flex-start;\n  }\n  \
[data-scope=\"data-list\"][data-part=\"root\"][data-blocks-description-list-two-column-list] {\n    grid-template-columns: repeat(2, minmax(0, 1fr));\n  }\n  \
[data-scope=\"data-list\"][data-part=\"item\"][data-blocks-description-list-two-column-span=\"full\"] {\n    grid-column: 1 / -1;\n  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// Demo が期待する 5 部品を持ち、非対話制約（`<form>` なし・
    /// `href="#"` なし・`data:` src なし）を満たすこと。
    #[test]
    fn demo_composes_expected_parts_and_has_no_form() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"data-list\"",
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"attachment\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        // Button は headless `button` scope を持たない静的部品のため、
        // `type="button"` の存在で使用を確認する（下の
        // `all_buttons_are_type_button` テストと合わせて二重に固定）。
        assert!(html.contains("type=\"button\""));
        assert!(!html.contains("<form"));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("src=\"data:"));
    }

    /// 2 インスタンス: caption が 2 件あり、「編集」ボタンは B（集約元
    /// R0456）だけに 1 回だけ出ること。
    #[test]
    fn demo_renders_both_instances() {
        let html = render(&demo());
        assert_eq!(
            html.matches("data-blocks-description-list-two-column-caption")
                .count(),
            2
        );
        assert_eq!(html.matches("編集").count(), 1);
    }

    /// 全幅項目（概要 + 添付ファイル）が 2 インスタンス × 2 件 = 4 件、
    /// 添付ファイル自体は 2 インスタンス × 2 件 = 4 件出ること。
    #[test]
    fn demo_renders_expected_full_span_and_attachment_counts() {
        let html = render(&demo());
        assert_eq!(
            html.matches(r#"data-blocks-description-list-two-column-span="full""#)
                .count(),
            4
        );
        assert_eq!(
            html.matches("data-scope=\"attachment\" data-part=\"root\"")
                .count(),
            4
        );
    }

    /// すべての `<button` に `type="button"` が付いていること（暗黙
    /// submit を起こさない）。
    #[test]
    fn all_buttons_are_type_button() {
        let html = render(&demo());
        let button_count = html.matches("<button").count();
        assert!(button_count > 0);
        assert_eq!(html.matches("type=\"button\"").count(), button_count);
    }

    /// [`LAYOUT_CSS`] が狭幅の 1 列グリッドと `@media (min-width: 40rem)`
    /// の 2 列化・全幅指定を持つこと。
    #[test]
    fn layout_css_has_responsive_grid_rules() {
        assert!(LAYOUT_CSS.contains("grid-template-columns: 1fr;"));
        assert!(LAYOUT_CSS.contains("@media (min-width: 40rem)"));
        assert!(LAYOUT_CSS.contains("grid-template-columns: repeat(2, minmax(0, 1fr));"));
        assert!(LAYOUT_CSS.contains("grid-column: 1 / -1;"));
    }

    /// ルート属性フック（`demo_class` とは別名）が [`demo`] の出力へ実際に
    /// 現れること。
    #[test]
    fn layout_root_hook_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("data-blocks-description-list-two-column-layout"));
        assert_ne!(
            super::BLOCK.demo_class,
            "data-blocks-description-list-two-column-layout"
        );
    }
}
