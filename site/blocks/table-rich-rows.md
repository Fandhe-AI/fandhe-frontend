# table-rich-rows

`table` / `avatar` / `badge` / `status` / `link` の 5 部品を合成した、先頭列にアバター + 氏名/メールの 2 段テキスト、次列に役割/部署の 2 段テキスト、状態列にバッジを置くリッチテーブルの合成例です。

2 インスタンスを静的に併記します。

- **members**（主参照）: メンバー一覧。状態列は `badge`（有効 = Success / 招待中 = Warning / 停止 = Danger）、操作列は実在 URL への「GitHub で見る」リンクです。
- **deploys**（集約元・全幅版）: 同じ骨格をデプロイ履歴（作業ログ）へ流用したものです。結果列は `status`（成功 = Success / 実行中 = Info / 失敗 = Danger）、開始時刻列は `<time datetime>` で表します（実行中の行にも意味が通る全行共通の列見出しです）。全幅表示にするため `table` の `root` へ `data-blocks-table-rich-rows-full` を付与しています。

先頭列（アバター + 2 段テキスト）と状態列は幅に関わらず常に表示します。副次列（役割/変更内容の列）のみ、`@container` で狭幅時に非表示にします。各テーブルは `table::scroll_area`（`role="region"` + `aria-label` + `tabindex="0"`）で包み、狭幅で列がはみ出す場合に横スクロールできるようにします。

本 Demo は無 JS の静的表示です。`<form>` は出力せず、送信・取得等の対話処理は一切行いません。氏名・役職・メールアドレス・部署名・ブランチ名・コミット要約・ハッシュ・日付はすべて架空の文言で、実在の人物・企業・クレデンシャルとは無関係です。

## Rust コード

```rust
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

/// 先頭列（アバター + 2 段テキスト）の `<td>`。
///
/// `blocks-table-rich-rows-identity`（`display: flex`）は `<td>` 自体では
/// なく内側の `div` に適用する。`<td>` に `display: flex` を当てると
/// `table-cell` 表示から外れ、テーブルレイアウトに参加しなくなり見出しと
/// 本文の列配置がずれるため（レビュー指摘対応）。
fn identity_cell(name: &str, sub: &str) -> Node {
    table::cell(
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
```

## 差分メモ

- members（代表構成）→ deploys（全幅版）で、列の意味を「メンバー / 役割 / 状態 / 操作」から「実行者 / 変更内容 / 結果 / 開始時刻」へ読み替えています。
- 状態表現は `badge`（members、離散的な区分値）と `status`（deploys、進行中を含む実行結果）を使い分けています。
- deploys は `table` の `root` に `data-blocks-table-rich-rows-full` を付与し、全幅表示にしています。
