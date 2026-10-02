//! `table-rich-rows` block（イシュー #2944。親トラッキング #2892「Blocks
//! アプリケーション A」配下、phase:3）。先頭列にアバター + 氏名/メールの
//! 2 段テキスト、次列に役職/部署の 2 段テキスト、状態列にバッジを置く
//! リッチテーブルの合成例。
//!
//! # 使用部品
//!
//! `table` / `avatar` / `badge` / `status` / `link` の 5 部品のみを合成する
//! （[`BLOCK`] の `parts` に一致させる契約）。新しい UI 部品は追加しない。
//!
//! # 2 インスタンスで派生形を表現する（無 JS のため静的併記）
//!
//! 主参照（メンバー一覧）を軸に、集約元（全幅のデプロイ履歴/作業ログ）を
//! 同じ骨格の 2 件目として併記する。`_/blocks-intake/` の対応ファイルは
//! 本イシュー着手時点で本 worktree に存在しないため、他 block（
//! `page_heading_avatar.rs` 等）と同じ扱いで対応表 ID の言及は省く。
//!
//! - **members**: メンバー（アバター + 氏名/メール 2 段）| 役割（役職/部署
//!   2 段）| 状態（`badge`: 有効 = Success / 招待中 = Warning / 停止 =
//!   Danger）| 操作（`link::root` 「GitHub で見る」）。
//! - **deploys**: 同じ骨格をデプロイ履歴（作業ログ）へ流用。実行者
//!   （アバター + 氏名/ブランチ名 2 段）| 変更内容（コミット要約/短縮
//!   ハッシュ 2 段）| 結果（`status::root`: 成功 = Success / 実行中 = Info /
//!   失敗 = Danger）| 開始時刻（`<time datetime>`。実行中の行にも意味が
//!   通る全行共通の列見出しとする。`完了時刻` は未完了の実行中行にまで
//!   完了済みの含意を持たせてしまうため codex-review 指摘で改称した）。
//!   `table::root` の `data-blocks-table-rich-rows-full` を付与し全幅表示
//!   にする。
//!
//! # 狭幅では副次列のみ隠す（`@container`）
//!
//! Demo 枠の幅はビューポート幅と一致しないため、`@media` ではなく
//! `profile_detail_datalist.rs` と同型の `@container` を使う。副次列
//! （役割/変更内容の列）にのみ `data-blocks-table-rich-rows-secondary` を
//! 付与し、狭幅ではこの列だけを非表示にする。先頭列の 2 段テキスト・
//! 状態列は幅に関わらず残す（主要情報を隠さない）。
//!
//! # リンクの遷移先について
//!
//! `href="#"` は使わず、実在の URL（`REPO`）へ遷移する「GitHub で見る」
//! ラベルを使う（`page_heading_avatar.rs` と同型の判断: 可視ラベルが
//! 遷移先の意味を過大に主張しない）。
//!
//! # `<form>` を使わない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない。操作列はリンクのみで、ボタン・送信処理を持たない。
//!
//! # ダミー素材について
//!
//! 氏名・役職は `crate::blocks::dummy_assets::PERSON_NAMES`/`JOB_TITLES`、
//! アバター画像は `dummy_assets::AVATAR_SRC` の架空セットを使う。メール
//! アドレス・部署名・ブランチ名・コミット要約・ハッシュ・日付はすべて
//! 独自に書いた架空の文言であり、実在の人物・企業・クレデンシャルとは
//! 無関係。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps, BadgeVariant};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::status::{self, StatusProps};
use fandhe_frontend_pre_styled_ui::table::{self, TableProps};
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// 外部の実在 URL（`href="#"` は使わない、他 block と同型の判断）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// 氏名からイニシャル（先頭 1 文字）を組み立てる（`list_people.rs` と同型）。
fn initials(name: &str) -> String {
    name.chars().take(1).collect()
}

/// アバター（円形）を組み立てる（`list_people.rs::person_avatar` と同型）。
fn person_avatar(name: &str) -> Node {
    avatar::root(
        &AvatarProps {
            size: Size::Sm,
            ..AvatarProps::default()
        },
        vec![],
        vec![
            avatar::image(ImageStatus::Loaded, dummy_assets::AVATAR_SRC, "", vec![]),
            avatar::fallback(ImageStatus::Loaded, vec![], vec![text(initials(name))]),
        ],
    )
}

/// 2 段テキスト（主要行 + 補足行）。`fandhe_frontend_pre_styled_ui::text::text`
/// は呼び出し側の `class` を `drop_class_attr` で除去するため、
/// `page_heading_avatar.rs`/`list_people.rs` と同様に素の `div` で組み立てる。
fn two_line(primary: Node, secondary: &str) -> Node {
    div(
        vec![("class", "blocks-table-rich-rows-lines")],
        vec![
            div(
                vec![("class", "blocks-table-rich-rows-primary")],
                vec![primary],
            ),
            div(
                vec![("class", "blocks-table-rich-rows-secondary-text")],
                vec![text(secondary)],
            ),
        ],
    )
}

/// 先頭列（アバター + 2 段テキスト）の `<th scope="row">`。
///
/// 行の識別子（人物名/ブランチ名）を表すため `table::cell`（`<td>`）ではなく
/// `table::row_header`（`<th scope="row">`）で出力し、行見出しとしての意味論
/// を持たせる（イシュー #2825 の `row_header` を使用、codex-review 指摘対応）。
/// `blocks-table-rich-rows-identity`（`display: flex`）は `<th>` 自体では
/// なく内側の `div` に適用する。`<th>` に `display: flex` を当てると
/// `table-cell` 表示から外れ、テーブルレイアウトに参加しなくなり見出しと
/// 本文の列配置がずれるため（レビュー指摘対応）。
fn identity_cell(name: &str, sub: &str) -> Node {
    table::row_header(
        vec![],
        vec![div(
            vec![("class", "blocks-table-rich-rows-identity")],
            vec![person_avatar(name), two_line(text(name), sub)],
        )],
    )
}

/// 副次列（役割/変更内容）の `<td>`。狭幅で非表示にする対象
/// （モジュール doc「狭幅では副次列のみ隠す」節参照）。
fn secondary_cell(primary: &str, sub: &str) -> Node {
    table::cell(
        vec![("data-blocks-table-rich-rows-secondary", "")],
        vec![two_line(text(primary), sub)],
    )
}

/// 副次列の `<th scope="col">`。
fn secondary_header(label: &str) -> Node {
    table::column_header(
        vec![("data-blocks-table-rich-rows-secondary", "")],
        vec![text(label)],
    )
}

/// パネル外枠。`container-type: inline-size` を宣言し `@container` の対象に
/// する（`profile_detail_datalist.rs` と同型）。狭幅で副次列を隠しても
/// members/deploys の残り列（アバター + 2 段テキスト・状態/バッジ・操作）が
/// なお枠内に収まらない場合に備え、`table::scroll_area` で横スクロール手段を
/// 確保する（`table_with_heading.rs`/`comparison_split_table.rs` と同型の
/// `role="region"` + `aria-label` + `tabindex="0"` 構成、codex-review 指摘
/// 是正）。
fn panel(variant: &'static str, aria_label: &str, table_node: Node) -> Node {
    div(
        vec![
            ("class", "blocks-table-rich-rows-panel"),
            ("data-blocks-table-rich-rows-variant", variant),
        ],
        vec![table::scroll_area(
            vec![
                ("role", "region"),
                ("aria-label", aria_label),
                ("tabindex", "0"),
            ],
            vec![table_node],
        )],
    )
}

/// members インスタンス。列: メンバー | 役割 | 状態 | 操作。
fn members_instance() -> Node {
    struct Member {
        name_index: usize,
        email: &'static str,
        role_index: usize,
        team: &'static str,
        state: &'static str,
        palette: ColorPalette,
    }
    const MEMBERS: [Member; 4] = [
        Member {
            name_index: 0,
            email: "haruto.fujimaki@example.com",
            role_index: 0,
            team: "プラットフォームチーム",
            state: "有効",
            palette: ColorPalette::Success,
        },
        Member {
            name_index: 1,
            email: "elena.vasquez@example.com",
            role_index: 1,
            team: "インフラチーム",
            state: "招待中",
            palette: ColorPalette::Warning,
        },
        Member {
            name_index: 2,
            email: "kwame.boateng@example.com",
            role_index: 2,
            team: "サポートチーム",
            state: "有効",
            palette: ColorPalette::Success,
        },
        Member {
            name_index: 3,
            email: "mei.lindqvist@example.com",
            role_index: 3,
            team: "データチーム",
            state: "停止",
            palette: ColorPalette::Danger,
        },
    ];

    let rows: Vec<Node> = MEMBERS
        .iter()
        .map(|member| {
            let name = dummy_assets::PERSON_NAMES[member.name_index];
            let role = dummy_assets::JOB_TITLES[member.role_index];
            table::row(
                vec![],
                vec![
                    identity_cell(name, member.email),
                    secondary_cell(role, member.team),
                    table::cell(
                        vec![],
                        vec![badge::badge(
                            &BadgeProps {
                                variant: BadgeVariant::Subtle,
                                size: Size::Sm,
                                palette: member.palette,
                                shape: None,
                            },
                            vec![],
                            vec![text(member.state)],
                        )],
                    ),
                    table::cell(
                        vec![],
                        vec![link::root(
                            REPO,
                            &LinkProps {
                                external: true,
                                ..LinkProps::default()
                            },
                            vec![],
                            vec![text("GitHub で見る")],
                        )],
                    ),
                ],
            )
        })
        .collect();

    panel(
        "members",
        "メンバー一覧",
        table::root(
            TableProps {
                interactive: true,
                ..TableProps::default()
            },
            vec![],
            vec![
                table::caption(vec![], vec![text("メンバー一覧")]),
                table::header(
                    vec![],
                    vec![table::row(
                        vec![],
                        vec![
                            table::column_header(vec![], vec![text("メンバー")]),
                            secondary_header("役割"),
                            table::column_header(vec![], vec![text("状態")]),
                            table::column_header(vec![], vec![text("操作")]),
                        ],
                    )],
                ),
                table::body(vec![], rows),
            ],
        ),
    )
}

/// deploys インスタンス（全幅のデプロイ履歴。作業ログ）。
fn deploys_instance() -> Node {
    struct Deploy {
        name_index: usize,
        branch: &'static str,
        summary: &'static str,
        hash: &'static str,
        result: &'static str,
        palette: ColorPalette,
        iso: &'static str,
        label: &'static str,
    }
    const DEPLOYS: [Deploy; 3] = [
        Deploy {
            name_index: 4,
            branch: "feat/checkout-redesign",
            summary: "チェックアウト導線の見直し",
            hash: "a1c9f02",
            result: "成功",
            palette: ColorPalette::Success,
            iso: "2026-09-28T10:12:00+09:00",
            label: "9 月 28 日 10:12",
        },
        Deploy {
            name_index: 5,
            branch: "fix/table-overflow",
            summary: "table 部品の overflow 是正",
            hash: "7b40de1",
            result: "実行中",
            palette: ColorPalette::Info,
            iso: "2026-09-28T11:40:00+09:00",
            label: "9 月 28 日 11:40",
        },
        Deploy {
            name_index: 6,
            branch: "chore/deps-bump",
            summary: "依存クレートの更新",
            hash: "5e2af90",
            result: "失敗",
            palette: ColorPalette::Danger,
            iso: "2026-09-27T09:05:00+09:00",
            label: "9 月 27 日 9:05",
        },
    ];

    let rows: Vec<Node> = DEPLOYS
        .iter()
        .map(|deploy| {
            let name = dummy_assets::PERSON_NAMES[deploy.name_index];
            table::row(
                vec![],
                vec![
                    identity_cell(name, deploy.branch),
                    secondary_cell(deploy.summary, deploy.hash),
                    table::cell(
                        vec![],
                        vec![status::root(
                            &StatusProps {
                                size: Size::Sm,
                                palette: deploy.palette,
                            },
                            vec![],
                            vec![status::indicator(vec![]), text(deploy.result)],
                        )],
                    ),
                    table::cell(
                        vec![],
                        vec![el(
                            "time",
                            vec![("datetime", deploy.iso)],
                            vec![text(deploy.label)],
                        )],
                    ),
                ],
            )
        })
        .collect();

    panel(
        "deploys",
        "デプロイ履歴",
        table::root(
            TableProps {
                interactive: true,
                ..TableProps::default()
            },
            vec![("data-blocks-table-rich-rows-full", "")],
            vec![
                table::caption(vec![], vec![text("デプロイ履歴")]),
                table::header(
                    vec![],
                    vec![table::row(
                        vec![],
                        vec![
                            table::column_header(vec![], vec![text("実行者")]),
                            secondary_header("変更内容"),
                            table::column_header(vec![], vec![text("結果")]),
                            table::column_header(vec![], vec![text("開始時刻")]),
                        ],
                    )],
                ),
                table::body(vec![], rows),
            ],
        ),
    )
}

/// `table-rich-rows` の Demo 本体。呼び出しごとに同一の `Node` を返す純
/// 関数。members（主参照）を先頭に、deploys（全幅版）を縦に並べる。
#[must_use]
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-table-rich-rows-layout")],
        vec![members_instance(), deploys_instance()],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/table-rich-rows/",
    title: "table-rich-rows",
    category: BlockCategory::Table,
    rust_source: "crates/docs-site/src/blocks/application/table/table_rich_rows.rs",
    demo_class: "blocks-table-rich-rows",
    parts: &[
        Part {
            label: "Table",
            path: "/themes/table/",
        },
        Part {
            label: "Avatar",
            path: "/themes/avatar/",
        },
        Part {
            label: "Badge",
            path: "/themes/badge/",
        },
        Part {
            label: "Status",
            path: "/themes/status/",
        },
        Part {
            label: "Link",
            path: "/themes/link/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `table_rich_rows` 固有のレイアウト規則（`crate::blocks`
/// モジュール doc「CSS の置き場」節と同型）。
const LAYOUT_CSS: &str = "\
.blocks-table-rich-rows-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n}\n\
.blocks-table-rich-rows-panel {\n  container-type: inline-size;\n  container-name: blocks-table-rich-rows;\n}\n\
.blocks-table-rich-rows-identity {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-table-rich-rows-lines {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1);\n  min-width: 0;\n}\n\
.blocks-table-rich-rows-secondary-text {\n  color: var(--fandhe-color-fg-muted);\n  font-size: var(--fandhe-font-font-size-sm);\n}\n\
table[data-scope=\"table\"][data-blocks-table-rich-rows-full] {\n  width: 100%;\n}\n\
@container blocks-table-rich-rows (max-width: 36rem) {\n  \
[data-blocks-table-rich-rows-variant] [data-blocks-table-rich-rows-secondary] {\n    display: none;\n  }\n\
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
            "data-scope=\"table\"",
            "data-scope=\"avatar\"",
            "data-scope=\"badge\"",
            "data-scope=\"status\"",
            "data-scope=\"link\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
    }

    #[test]
    fn demo_wires_both_variant_hooks() {
        let html = demo_html();
        for variant in ["members", "deploys"] {
            assert!(html.contains(&format!(
                "data-blocks-table-rich-rows-variant=\"{variant}\""
            )));
        }
    }

    #[test]
    fn no_form_semantics_or_dead_links() {
        let html = demo_html();
        assert!(!html.contains("<form"));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("src=\"data:"));
        assert!(!html.contains("mailto:"));
    }

    #[test]
    fn secondary_columns_are_marked_on_header_and_cells() {
        let html = demo_html();
        // 副次列は column-header（<th>）2 件（members/deploys 各 1）+ cell
        // （<td>）7 件（members 4 行 + deploys 3 行）の計 9 件に付与される。
        assert_eq!(
            html.matches("data-blocks-table-rich-rows-secondary")
                .count(),
            9
        );
        assert!(html.contains(
            "data-part=\"column-header\" scope=\"col\" data-blocks-table-rich-rows-secondary"
        ));
        assert!(html.contains("data-part=\"cell\" data-blocks-table-rich-rows-secondary"));
    }

    #[test]
    fn layout_css_hides_only_secondary_columns_in_narrow_container() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("@container blocks-table-rich-rows (max-width: 36rem)"));
        assert!(LAYOUT_CSS.contains("data-blocks-table-rich-rows-secondary] {\n    display: none;"));
        assert!(!LAYOUT_CSS.contains("-identity] {\n    display: none;"));
        assert!(!LAYOUT_CSS.contains("-lines] {\n    display: none;"));
    }

    #[test]
    fn full_width_selector_beats_table_recipe_base() {
        assert!(
            LAYOUT_CSS.contains("table[data-scope=\"table\"][data-blocks-table-rich-rows-full]")
        );
    }

    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = demo_html();
        assert!(html.contains("class=\"blocks-table-rich-rows-layout\""));
        assert_ne!(super::BLOCK.demo_class, "blocks-table-rich-rows-layout");
    }

    #[test]
    fn demo_is_deterministic() {
        assert_eq!(demo_html(), demo_html());
    }

    #[test]
    fn uses_shared_dummy_assets() {
        let html = demo_html();
        assert!(html.contains(super::dummy_assets::AVATAR_SRC));
    }

    #[test]
    fn has_time_element_with_datetime() {
        let html = demo_html();
        assert!(html.contains("<time datetime="));
    }

    #[test]
    fn link_label_does_not_overclaim_destination() {
        let html = demo_html();
        assert!(html.contains("href=\"https://github.com/Fandhe-AI/fandhe-frontend\""));
        assert!(html.contains("GitHub で見る"));
    }

    /// codex-review 指摘是正（狭幅時の横スクロール手段）: 各 `panel` が
    /// `table::scroll_area`（`role="region"` + `aria-label` + `tabindex="0"`）
    /// で `table::root` を包んでいることを固定する。
    #[test]
    fn tables_are_wrapped_in_scroll_area() {
        let html = demo_html();
        assert_eq!(html.matches("data-part=\"scroll-area\"").count(), 2);
        assert!(html.contains(r#"role="region" aria-label="メンバー一覧""#));
        assert!(html.contains(r#"role="region" aria-label="デプロイ履歴""#));
        assert_eq!(html.matches(r#"tabindex="0""#).count(), 2);
    }

    /// codex-review 指摘是正（実行中行の完了時刻誤読）: 列見出しは全行
    /// 共通の意味を持つ「開始時刻」を使い、完了済みを含意する
    /// 「完了時刻」は使わない。
    #[test]
    fn deploys_time_column_uses_start_time_label_not_completion() {
        let html = demo_html();
        assert!(html.contains("開始時刻"));
        assert!(!html.contains("完了時刻"));
    }
}
