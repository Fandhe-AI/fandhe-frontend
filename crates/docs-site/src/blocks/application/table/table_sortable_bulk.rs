//! `table-sortable-bulk` block（イシュー #2945。親トラッキング #2892
//! 「Blocks アプリケーション A」配下、Application / Table カテゴリ）。
//! 列見出しに並び替え印を付け、先頭列に行選択
//! チェックボックスを置くデータテーブル。1 行以上選択された状態では
//! 見出し行の上に一括操作ツールバー（件数 + 操作ボタン）を重ねて表示する。
//! 対応表 ID R1335（主参照・代表構成）を軸に、R1331（ソート可能な列見出しの
//! みの構成）を「未選択」インスタンスへ集約する（`_/blocks-intake/` の
//! 対応ファイルは本イシュー着手時点で本 worktree に存在しないため、原稿・
//! 本コメントには対応表 ID のみを記す。`list_title_meta`/`list_people` と
//! 同じ扱い）。
//!
//! # 使用部品
//!
//! `data-table` / `table` / `checkbox` / `button` / `icon` の 5 部品を
//! 合成する（[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//! 新しい UI 部品は追加しない。
//!
//! # 「未選択」「選択中」を静的に併記する理由
//!
//! docs サイトは JS ハイドレーションを行わない（`crate::blocks` モジュール
//! doc 参照）。実運用では行チェックボックスの操作に応じて一括操作
//! ツールバーの表示/非表示が切り替わるが、本 Demo は無 JS の静的表示
//! しかできないため、`data-blocks-table-sortable-bulk-variant`（`"none"`/
//! `"selected"`）で 2 パネルを縦に並べて両状態を同時に示す
//! （`list_people`/`cta_signup_celebrate` 等と同型の判断）。
//!
//! # Themes 推奨の組み立て（`table::column_header` へ委譲する理由）
//!
//! `fandhe_frontend_pre_styled_ui::data_table` モジュール doc「Themes 推奨
//! の組み立て」節が定める契約どおり、表本体（`<table>`/`<thead>`/
//! `<tbody>`/`<tr>`）は `data_table` 側の `column_header`/`sort_trigger`
//! （node を生成するパーツ、Primitives 経路向け）を使わず、
//! `data_table::column_header_attrs`/`row_attrs`/`column_attrs`（node を
//! 作らない属性ヘルパ）を [`crate::table`] の `column_header`/`row`/`cell`
//! の `attrs` へ渡す形で合成する。`data_table::sort_trigger` はそのまま
//! 使う（`th`/`td` の中身であり `table` 側に対応するパーツがないため）。
//!
//! # 一括操作ツールバーを見出し行へ重ねる実装（`@container` 幅切替）
//!
//! `data_table::toolbar` を表本体のラッパー
//! `.blocks-table-sortable-bulk-table-wrap`（`position: relative;
//! container-type: inline-size;`）の子として配置し、
//! `position: absolute; top: 0; inset-inline-start/-end` で見出し行に
//! 重ねて表示する。`inset-inline-start` は選択列幅の既定値
//! `var(--fandhe-data-table-select-width, 2.5rem)` に固定するため、選択列
//! （`data_table::select_all`/`select_row` の共有 base）が持つ左右
//! padding（`--fandhe-space-4`）ぶん実際の列幅が `2.5rem` を超えると、
//! 通常幅でもツールバーが全行選択チェックボックスへ重なってしまう
//! （box-sizing 既定の `content-box` では `width` に padding が加算される
//! ため。Codex 指摘、イシュー #2945 PR #3393）。`table::column_header`/
//! `table::cell`（`crate::table`）は呼び出し側の `data-scope`/`data-part`
//! を強制的に上書きし常に `data-scope="table"` の `column-header`/`cell`
//! パーツへ固定するため（`data_table::select_all`/`select_row` が本来
//! 持つ `data-scope="data-table"` の `select-all`/`select-row` パーツは
//! 実際の DOM には現れない。同一指摘の再発、イシュー #2945 PR #3393
//! 追加レビュー）、選択列だけを識別する block 固有属性
//! `data-blocks-table-sortable-bulk-select-cell` を選択列の見出しセル・
//! 各行のセルへ付与し、本 block 局所の CSS オーバーライド（同属性へ
//! `box-sizing: border-box; padding-inline: 0;
//! width: var(--fandhe-data-table-select-width, 2.5rem);
//! text-align: center;` を適用）で解消する。`width` を明示しないと
//! `box-sizing`/`padding-inline: 0` だけでは列幅が定まらず、
//! `select_all`/`select_row` 共有 base の他の暗黙幅（フォントサイズ等）
//! 次第でツールバーの `inset-inline-start` 前提から実幅がずれ得るため、
//! `inset-inline-start` と同じ `var(--fandhe-data-table-select-width,
//! 2.5rem)` を選択列の実幅として明示し厳密に一致させる（Codex 指摘、
//! イシュー #2945 PR #3393 追加レビュー）。`text-align: center` も
//! block 固有属性側で明示する: `select_all`/`select_row` 共有 base が
//! 本来持つ `text-align: center` は `data-scope="data-table"` を要求する
//! セレクタのため、`table::column_header`/`table::cell` が強制する
//! `data-scope="table"` の実際の DOM には適用されず、`padding-inline: 0`
//! のままではチェックボックス（`inline-flex`）が列内で左寄せに描画される
//! （Cursor Bugbot 指摘、イシュー #2945 PR #3393 レビュー）。見出し行の
//! `th`（`.blocks-table-sortable-bulk-table-wrap thead th`）にも
//! `box-sizing: border-box` を明示する: 既定の `content-box` のままでは
//! `height: 3rem` が `column-header` base の `padding`/`border-bottom` を
//! 含まない content 領域のみの高さとなり、実際の `th` が同じ
//! `height: 3rem` のツールバーより高くなって、ツールバー下端からソート
//! ラベルがはみ出して透けて見える（Cursor Bugbot 指摘、イシュー #2945
//! PR #3393 レビュー）。`.blocks-table-sortable-bulk-table-wrap` に
//! `overflow-x: auto` を、内側の `table` に `min-width: 32rem` を付与する:
//! 残る氏名・ステータス・担当の 3 列（`select_row` の固定列幅と合わせて
//! 実測で 32rem 程度）には最小限の可読幅があり、`min-width` なしでは
//! `40rem` 以下でセルが可読限界を超えて圧縮され表がラップ幅を超えて
//! はみ出していた（絶対配置のツールバーはラップ幅に収まるため見出し行を
//! 覆う契約が崩れる。Codex 指摘、イシュー #2945 PR #3393）。`overflow-x:
//! auto` で表本体を横スクロール可能にすると、ツールバーはラップの
//! containing block 上に絶対配置されたまま横スクロールに追従するため
//! （absolute 配置の子要素はスクロールコンテナの overflow 領域に含まれる）、
//! 表とツールバーの幅・スクロール位置が常に一致し続ける。狭幅
//! （`@container ... (max-width: 40rem)`）では
//! 副次列（役割・最終更新）を隠し、ツールバーは全幅帯
//! （`inset-inline-start: 0`）として残す。この幅切替でもツールバーが
//! 全行選択チェックボックスを覆うため、同じ `@container` 規則内で
//! `[data-blocks-table-sortable-bulk-select-header]` に `visibility:
//! hidden` を適用し、覆われている間はチェックボックスをフォーカス対象
//! からも外す（`visibility: hidden` は Tab 移動・アクセシビリティツリー
//! の双方から除外するため、可視状態とフォーカス可否が常に一致する）。
//! 「未選択」パネルはツールバー自体を出力しない（1 行も選択されていない
//! 状態を JS 無しで正しく表す）。
//!
//! 並び替え可能な列見出しの `sort-trigger`（名前・ステータス列）は、
//! ツールバーが選択中パネルでは幅を問わず常に見出し行へ重なるため、
//! `tabindex="-1"` に加え `data-blocks-table-sortable-bulk-covered`
//! フックを付与し、CSS で `visibility: hidden` を適用する。当初
//! `tabindex="-1"` のみで対処していたが、Tab 移動からは外れても
//! アクセシビリティツリーからは除外されずスクリーンリーダーからは発見・
//! 操作可能なまま残っていた（github-actions 自動レビュー指摘、イシュー
//! #2945 PR #3393 追加レビュー）。`select-header` と異なり狭幅
//! `@container` 規則の内側に限定せず常時適用する: 通常幅でもツールバーの
//! `inset-inline-start`（選択列幅）〜`inset-inline-end: 0` は選択列を
//! 除く全列見出しを覆うため。
//!
//! # `menu`/ボタンを disabled にしない理由
//!
//! 一括操作ボタン（アーカイブ・削除）は `type="button"` で送信先を
//! 持たず、押しても何も起きない（`<form>` を出力しないため暗黙 submit も
//! 起きない）。`list_title_meta` と異なり、これらのボタンは「実運用で
//! 有効化される操作」を示す静的な実例であり、無効化して操作不能に見せる
//! 必要はないと判断した（並び替え可能な列見出しの `sort-trigger` も同様
//! に `disabled` を持たない。押下しても no-op であることは `<form>` 不在・
//! JS 非配線から自明であり、`list_title_meta` の「押しても何も起きない
//! 要素を操作可能に見せない」注意は行選択・一括削除のような破壊的操作を
//! 無効化して隠す趣旨ではなく menu/button の disabled 軸を持つ部品のみに
//! 適用した判断である）。
//!
//! # `<form>` を使わない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない。並び替え・行選択・一括操作の実処理（送信・永続化）は
//! アプリケーション責務であり本 block は静的表示のみを担う
//! （`docs/policy/intentional-non-adoption.md` §3.25）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::checkbox::{self, CheckboxProps};
use fandhe_frontend_pre_styled_ui::data_table::{
    self, ColumnHeaderProps, ColumnProps, DataTable, DataTableProps, SortDirection,
};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::table::{self, TableProps};
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// 行選択チェックボックス（`crate::showcase::data_table_section` の
/// `row_select_checkbox` と同型）。`name` は panel・行ごとに一意にし
/// （選択中/未選択パネル双方を同じページへ静的併記するため）、
/// `demo_output_has_no_dangling_aria_references_or_duplicate_ids`
/// （`crates/docs-site/tests/blocks_contract.rs`）の id 重複検知に抵触
/// しないよう `id` 属性自体を持たない（`aria-label` のみで名前付け）。
///
/// 常にフォーカス可能（`tabindex` を持たない）。全行選択チェックボックスが
/// 選択中パネルの狭幅表示で一時的に覆い隠される問題は、[`panel`] 内の
/// CSS `visibility: hidden`（`select-header` 属性）で解決している
/// （可視状態とフォーカス可否を一致させる方針、イシュー #2945 PR #3393）。
fn row_select_checkbox(name: &str, checked: checkbox::CheckedState, label: &str) -> Node {
    let props = CheckboxProps {
        checked,
        ..CheckboxProps::default()
    };
    let hidden_input_attrs = vec![("aria-label", label)];
    checkbox::root(
        Size::Sm,
        ColorPalette::Accent,
        &props,
        vec![],
        vec![
            checkbox::hidden_input(&props, name, "on", hidden_input_attrs),
            checkbox::control(
                &props,
                vec![],
                vec![checkbox::indicator(&props, vec![], vec![])],
            ),
        ],
    )
}

/// アーカイブボタンの自作アイコン（トレイ + 下矢印の抽象図形。参照元の
/// アイコンは持ち込まず単純な幾何図形とする、`stats_row`/
/// `error_page_popular_links` と同型の判断）。
fn archive_icon() -> Node {
    icon(
        &IconProps {
            size: Size::Sm,
            ..IconProps::default()
        },
        vec![],
        vec![
            el(
                "path",
                vec![
                    ("d", "M3 5h18v4H3z"),
                    ("fill", "none"),
                    ("stroke", "currentColor"),
                    ("stroke-width", "2"),
                    ("stroke-linejoin", "round"),
                ],
                vec![],
            ),
            el(
                "path",
                vec![
                    ("d", "M5 9v9a1 1 0 0 0 1 1h12a1 1 0 0 0 1-1V9"),
                    ("fill", "none"),
                    ("stroke", "currentColor"),
                    ("stroke-width", "2"),
                    ("stroke-linecap", "round"),
                    ("stroke-linejoin", "round"),
                ],
                vec![],
            ),
            el(
                "path",
                vec![
                    ("d", "M10 13h4"),
                    ("stroke", "currentColor"),
                    ("stroke-width", "2"),
                    ("stroke-linecap", "round"),
                ],
                vec![],
            ),
        ],
    )
}

/// 削除ボタンの自作アイコン（ゴミ箱の抽象図形）。
fn delete_icon() -> Node {
    icon(
        &IconProps {
            size: Size::Sm,
            ..IconProps::default()
        },
        vec![],
        vec![
            el(
                "path",
                vec![
                    ("d", "M4 7h16"),
                    ("stroke", "currentColor"),
                    ("stroke-width", "2"),
                    ("stroke-linecap", "round"),
                ],
                vec![],
            ),
            el(
                "path",
                vec![
                    ("d", "M9 7V4h6v3"),
                    ("fill", "none"),
                    ("stroke", "currentColor"),
                    ("stroke-width", "2"),
                    ("stroke-linejoin", "round"),
                ],
                vec![],
            ),
            el(
                "path",
                vec![
                    ("d", "M6 7l1 13h10l1-13"),
                    ("fill", "none"),
                    ("stroke", "currentColor"),
                    ("stroke-width", "2"),
                    ("stroke-linejoin", "round"),
                ],
                vec![],
            ),
        ],
    )
}

/// 行 1 件分のダミーデータ（架空・`dummy_assets` 由来）。
struct RowData {
    name: &'static str,
    status: &'static str,
    role: &'static str,
    updated: (&'static str, &'static str),
    assignee: &'static str,
}

/// 4 行分の共通ダミーデータ（「未選択」「選択中」両パネルで共有する）。
/// 名前列を `aria-sort="ascending"` 表示するため、行の並びも名前の
/// 昇順（Elena, Haruto, Kwame, Mei）にする（表示順とソート表示の矛盾を
/// 防ぐ。Codex レビュー指摘、イシュー #2945 PR #3393）。
fn rows_data() -> [RowData; 4] {
    [
        RowData {
            name: dummy_assets::PERSON_NAMES[1],
            status: "Pending",
            role: dummy_assets::JOB_TITLES[1],
            updated: ("2026-09-18", "Sep 18, 2026"),
            assignee: dummy_assets::PERSON_NAMES[5],
        },
        RowData {
            name: dummy_assets::PERSON_NAMES[0],
            status: "Active",
            role: dummy_assets::JOB_TITLES[0],
            updated: ("2026-09-20", "Sep 20, 2026"),
            assignee: dummy_assets::PERSON_NAMES[4],
        },
        RowData {
            name: dummy_assets::PERSON_NAMES[2],
            status: "Active",
            role: dummy_assets::JOB_TITLES[2],
            updated: ("2026-09-15", "Sep 15, 2026"),
            assignee: dummy_assets::PERSON_NAMES[6],
        },
        RowData {
            name: dummy_assets::PERSON_NAMES[3],
            status: "Archived",
            role: dummy_assets::JOB_TITLES[3],
            updated: ("2026-09-02", "Sep 2, 2026"),
            assignee: dummy_assets::PERSON_NAMES[7],
        },
    ]
}

/// 1 パネル分（「未選択」または「選択中」）を組み立てる。
///
/// `variant` は `data-blocks-table-sortable-bulk-variant` の値
/// （`"none"`/`"selected"`）、`selected_count` は選択中行数（0 なら
/// ツールバーを出力しない）。
fn panel(variant: &'static str, selected_count: usize) -> Node {
    let rows = rows_data();
    let table_state = DataTable::new(Some(("name".to_string(), SortDirection::Ascending)), vec![]);

    let name_column = ColumnProps {
        id: "name",
        hidden: false,
    };
    let status_column = ColumnProps {
        id: "status",
        hidden: false,
    };
    let role_column = ColumnProps {
        id: "role",
        hidden: false,
    };
    let updated_column = ColumnProps {
        id: "updated",
        hidden: false,
    };
    let assignee_column = ColumnProps {
        id: "assignee",
        hidden: false,
    };

    let secondary_attr = ("data-blocks-table-sortable-bulk-secondary", "");

    // 選択中パネルでは一括操作ツールバーが見出し行に重なって覆い隠す
    // （モジュール doc「一括操作ツールバーを見出し行へ重ねる実装」節）。
    // 覆われている間は見出し行のソート操作要素（並び替えボタン）を
    // Tab 移動対象から外す（`tabindex="-1"`）。フォーカス先が視覚的に
    // 見えないまま操作可能になることを防ぐ（Codex レビュー指摘、イシュー
    // #2945 PR #3393）。「未選択」パネルはツールバー自体を出力しないため
    // 常にフォーカス可能のまま。
    //
    // 全行選択チェックボックスはこの一律判定に含めない: ツールバーの
    // `inset-inline-start` は通常幅では選択列幅ぶんオフセットされ
    // チェックボックスを覆わないが、狭幅（`@container` 切替）では
    // `0` になり覆う（CSS 定数 `[data-blocks-table-sortable-bulk-toolbar]`
    // 節参照）。無 JS の静的 SSR ではこの幅依存の可視状態を tabindex の
    // 静的付与で追従できないため、覆われている間だけ CSS
    // `visibility: hidden` で不可視化する（`select-header` 属性 + 狭幅
    // `@container` 規則）。`visibility: hidden` は要素をフォーカス対象・
    // アクセシビリティツリーからも除外するため、可視状態とフォーカス
    // 可否が常に一致する（見えているのに Tab で届かない／届くのに
    // 見えない状態を作らない。Codex/Cursor Bugbot 指摘、イシュー #2945
    // PR #3393）。
    let header_focusable = variant != "selected";
    // `tabindex="-1"` は Tab 移動からは外れるが、アクセシビリティツリーからは
    // 除外されないためスクリーンリーダーからは発見・操作可能なまま残る
    // （github-actions 自動レビュー指摘、イシュー #2945 PR #3393
    // 追加レビュー）。`select-header`（下記 `select_header_attrs`）と同じ
    // `data-blocks-table-sortable-bulk-covered` フックを付与し、CSS の
    // `visibility: hidden` で可視状態とフォーカス可否・アクセシビリティ
    // ツリーからの除外を一致させる。
    let header_tabindex_attr: Vec<(&str, &str)> = if header_focusable {
        vec![]
    } else {
        vec![
            ("tabindex", "-1"),
            ("data-blocks-table-sortable-bulk-covered", ""),
        ]
    };
    // `sort_trigger`（button）の子はソート可能な列名テキストのみであり、
    // これが `visibility: hidden`（上記 `[data-blocks-table-sortable-bulk-
    // covered]`）で不可視化されると `th` の中身がまるごと消え、列見出し名が
    // アクセシビリティツリーから失われる（Codex 指摘 P1、イシュー #2945
    // PR #3393）。覆われている間だけ、clip 手法（`visually_hidden` と同じ
    // 技法。`display: none`/`visibility: hidden` にしない — それらは支援
    // 技術からも要素を除外してしまう）で視覚的には隠しつつ DOM・
    // アクセシビリティツリーには残るフォールバックテキストを `th` へ
    // 追加する。「未覆時」は可視ボタンのテキストが唯一の情報源のため、
    // フォールバックは追加しない（二重読み上げの防止）。
    let sortable_header_children = |trigger: Node, label: &'static str| -> Vec<Node> {
        if header_focusable {
            vec![trigger]
        } else {
            vec![
                trigger,
                el(
                    "span",
                    vec![("data-blocks-table-sortable-bulk-header-fallback-label", "")],
                    vec![text(label)],
                ),
            ]
        }
    };
    let select_header_attrs: Vec<(&str, &str)> = if variant == "selected" {
        vec![
            ("scope", "col"),
            ("data-blocks-table-sortable-bulk-select-cell", ""),
            ("data-blocks-table-sortable-bulk-select-header", ""),
        ]
    } else {
        vec![
            ("scope", "col"),
            ("data-blocks-table-sortable-bulk-select-cell", ""),
        ]
    };

    let header_row = table::row(
        vec![],
        vec![
            table::column_header(
                select_header_attrs,
                vec![row_select_checkbox(
                    match variant {
                        "selected" => "select-selected-all",
                        _ => "select-none-all",
                    },
                    match selected_count {
                        0 => checkbox::CheckedState::Unchecked,
                        n if n >= rows.len() => checkbox::CheckedState::Checked,
                        _ => checkbox::CheckedState::Indeterminate,
                    },
                    "Select all rows",
                )],
            ),
            table::column_header(
                data_table::column_header_attrs(&ColumnHeaderProps {
                    column: name_column,
                    sort: Some(table_state.sort_direction_of("name").unwrap()),
                }),
                sortable_header_children(
                    data_table::sort_trigger(
                        &table_state,
                        "name",
                        header_tabindex_attr.clone(),
                        vec![text("名前")],
                    ),
                    "名前",
                ),
            ),
            table::column_header(
                data_table::column_header_attrs(&ColumnHeaderProps {
                    column: status_column,
                    sort: Some(
                        table_state
                            .sort_direction_of("status")
                            .unwrap_or(SortDirection::None),
                    ),
                }),
                sortable_header_children(
                    data_table::sort_trigger(
                        &table_state,
                        "status",
                        header_tabindex_attr.clone(),
                        vec![text("ステータス")],
                    ),
                    "ステータス",
                ),
            ),
            table::column_header(
                {
                    let mut attrs = data_table::column_header_attrs(&ColumnHeaderProps {
                        column: role_column,
                        sort: None,
                    });
                    attrs.push(secondary_attr);
                    attrs
                },
                vec![text("役割")],
            ),
            table::column_header(
                {
                    let mut attrs = data_table::column_header_attrs(&ColumnHeaderProps {
                        column: updated_column,
                        sort: None,
                    });
                    attrs.push(secondary_attr);
                    attrs
                },
                vec![text("最終更新")],
            ),
            table::column_header(
                data_table::column_header_attrs(&ColumnHeaderProps {
                    column: assignee_column,
                    sort: None,
                }),
                vec![text("担当")],
            ),
        ],
    );

    let mut body_rows = Vec::with_capacity(rows.len());
    for (index, row) in rows.iter().enumerate() {
        let is_selected = variant == "selected" && index < selected_count;
        let row_checkbox_name = format!("select-{variant}-row-{index}");
        body_rows.push(table::row(
            data_table::row_attrs(is_selected),
            vec![
                table::cell(
                    vec![("data-blocks-table-sortable-bulk-select-cell", "")],
                    vec![row_select_checkbox(
                        &row_checkbox_name,
                        if is_selected {
                            checkbox::CheckedState::Checked
                        } else {
                            checkbox::CheckedState::Unchecked
                        },
                        &format!("Select row: {}", row.name),
                    )],
                ),
                table::cell(data_table::column_attrs(&name_column), vec![text(row.name)]),
                table::cell(
                    data_table::column_attrs(&status_column),
                    vec![text(row.status)],
                ),
                table::cell(
                    {
                        let mut attrs = data_table::column_attrs(&role_column);
                        attrs.push(secondary_attr);
                        attrs
                    },
                    vec![text(row.role)],
                ),
                table::cell(
                    {
                        let mut attrs = data_table::column_attrs(&updated_column);
                        attrs.push(secondary_attr);
                        attrs
                    },
                    vec![el(
                        "time",
                        vec![("datetime", row.updated.0)],
                        vec![text(row.updated.1)],
                    )],
                ),
                table::cell(
                    data_table::column_attrs(&assignee_column),
                    vec![text(row.assignee)],
                ),
            ],
        ));
    }

    let mut wrap_children = Vec::new();
    if selected_count > 0 {
        wrap_children.push(data_table::toolbar(
            vec![("data-blocks-table-sortable-bulk-toolbar", "")],
            vec![
                data_table::selection_count(
                    vec![],
                    vec![text(format!("{selected_count} 件選択中"))],
                ),
                button::button(
                    &ButtonProps {
                        variant: ButtonVariant::Outline,
                        size: Size::Sm,
                        ..ButtonProps::default()
                    },
                    vec![],
                    vec![archive_icon(), text("アーカイブ")],
                ),
                button::button(
                    &ButtonProps {
                        variant: ButtonVariant::Outline,
                        size: Size::Sm,
                        ..ButtonProps::default()
                    },
                    vec![],
                    vec![delete_icon(), text("削除")],
                ),
            ],
        ));
    }
    wrap_children.push(table::root(
        TableProps {
            interactive: true,
            ..TableProps::default()
        },
        vec![],
        vec![
            table::header(vec![], vec![header_row]),
            table::body(vec![], body_rows),
        ],
    ));

    data_table::root(
        DataTableProps::default(),
        vec![("data-blocks-table-sortable-bulk-variant", variant)],
        vec![div(
            vec![("class", "blocks-table-sortable-bulk-table-wrap")],
            wrap_children,
        )],
    )
}

/// `table-sortable-bulk` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数。「未選択」（R1331 相当・0 行選択）と「選択中」（R1335 相当・
/// 4 行中 3 行選択）の 2 パネルを縦に併記する。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-table-sortable-bulk-layout")],
        vec![
            div(
                vec![("class", "blocks-table-sortable-bulk-panel")],
                vec![panel("none", 0)],
            ),
            div(
                vec![("class", "blocks-table-sortable-bulk-panel")],
                vec![panel("selected", 3)],
            ),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/table-sortable-bulk/",
    title: "table-sortable-bulk",
    category: BlockCategory::Table,
    rust_source: "crates/docs-site/src/blocks/application/table/table_sortable_bulk.rs",
    demo_class: "blocks-table-sortable-bulk",
    parts: &[
        Part {
            label: "Data Table",
            path: "/themes/data-table/",
        },
        Part {
            label: "Table",
            path: "/themes/table/",
        },
        Part {
            label: "Checkbox",
            path: "/themes/checkbox/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Icon",
            path: "/themes/icon/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `table_sortable_bulk` 固有のレイアウト規則（`crate::blocks`
/// モジュール doc「CSS の置き場」節）。`--fandhe-*` トークンを参照する
/// 宣言に加え、選択列の実幅・中央寄せを一括操作ツールバーの
/// `inset-inline-start` 前提へ合わせる `[data-blocks-table-sortable-bulk-
/// select-cell]` セレクタと、見出し行の外形高さをツールバーへ一致させる
/// `thead th` セレクタを持つ（モジュール doc「一括操作
/// ツールバーを見出し行へ重ねる実装」節、イシュー #2945 PR #3393）。本
/// 定数は `crate::showcase::stylesheet()` には集約されない block 固有
/// LAYOUT_CSS（[`Block::layout_css`](crate::blocks::Block::layout_css)）
/// のため `css_var_scope_prefix.rs`（`showcase::stylesheet()` のみ走査）
/// の対象外であり、custom property を新設しないためカスタムプロパティの
/// プレフィックス規約とも衝突しない。
const LAYOUT_CSS: &str = "\
.blocks-table-sortable-bulk-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n}\n\
.blocks-table-sortable-bulk-panel {\n  display: flex;\n  flex-direction: column;\n}\n\
.blocks-table-sortable-bulk-table-wrap {\n  position: relative;\n  container-type: inline-size;\n  container-name: blocks-table-sortable-bulk;\n  overflow-x: auto;\n}\n\
.blocks-table-sortable-bulk-table-wrap table {\n  min-width: 32rem;\n}\n\
.blocks-table-sortable-bulk-table-wrap thead th {\n  box-sizing: border-box;\n  height: 3rem;\n}\n\
.blocks-table-sortable-bulk-table-wrap [data-blocks-table-sortable-bulk-select-cell] {\n  box-sizing: border-box;\n  padding-inline: 0;\n  width: var(--fandhe-data-table-select-width, 2.5rem);\n  text-align: center;\n}\n\
[data-blocks-table-sortable-bulk-toolbar] {\n  position: absolute;\n  top: 0;\n  inset-inline-start: var(--fandhe-data-table-select-width, 2.5rem);\n  inset-inline-end: 0;\n  height: 3rem;\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n  padding-inline: var(--fandhe-space-3);\n  background: var(--fandhe-color-bg);\n  z-index: 1;\n}\n\
[data-blocks-table-sortable-bulk-covered] {\n  visibility: hidden;\n}\n\
[data-blocks-table-sortable-bulk-header-fallback-label] {\n  position: absolute;\n  width: 1px;\n  height: 1px;\n  padding: 0;\n  margin: -1px;\n  overflow: hidden;\n  clip: rect(0, 0, 0, 0);\n  white-space: nowrap;\n  overflow-wrap: normal;\n  border-width: 0;\n}\n\
@container blocks-table-sortable-bulk (max-width: 40rem) {\n  [data-blocks-table-sortable-bulk-secondary] {\n    display: none;\n  }\n\n  [data-blocks-table-sortable-bulk-toolbar] {\n    inset-inline-start: 0;\n  }\n\n  [data-blocks-table-sortable-bulk-select-header] {\n    visibility: hidden;\n  }\n}\n";

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;

    fn html() -> String {
        render(&demo())
    }

    #[test]
    fn demo_uses_all_declared_parts_scopes() {
        let html = html();
        for scope in ["data-table", "table", "checkbox", "button", "icon"] {
            assert!(
                html.contains(&format!(r#"data-scope="{scope}""#)),
                "missing data-scope=\"{scope}\""
            );
        }
    }

    #[test]
    fn demo_renders_both_variants() {
        let html = html();
        assert!(html.contains(r#"data-blocks-table-sortable-bulk-variant="none""#));
        assert!(html.contains(r#"data-blocks-table-sortable-bulk-variant="selected""#));
    }

    #[test]
    fn demo_shows_sort_direction_marks() {
        let html = html();
        assert!(html.contains(r#"aria-sort="ascending""#));
        assert!(html.contains(r#"data-sort="none""#));
    }

    #[test]
    fn selected_panel_has_three_selected_rows_and_indeterminate_select_all() {
        let html = html();
        assert_eq!(html.matches("data-selected").count(), 3);
        assert!(html.contains(r#"data-state="indeterminate""#));
    }

    #[test]
    fn toolbar_hook_appears_once_for_selected_panel_only() {
        let html = html();
        assert_eq!(
            html.matches("data-blocks-table-sortable-bulk-toolbar")
                .count(),
            1
        );
    }

    #[test]
    fn buttons_are_type_button_and_no_form_or_data_uri() {
        let html = html();
        let button_count = html.matches("<button").count();
        let type_button_count = html.matches(r#"type="button""#).count();
        assert!(type_button_count >= button_count);
        assert!(!html.contains("<form"));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("src=\"data:"));
    }

    #[test]
    fn layout_css_has_no_style_breakout_and_declares_container_query() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(!LAYOUT_CSS.contains("</style"));
        assert!(LAYOUT_CSS.contains("@container blocks-table-sortable-bulk (max-width: 40rem)"));
        assert!(LAYOUT_CSS.contains("position: absolute"));
    }

    /// 選択列（見出しセル・各行セル）の実幅をツールバーの
    /// `inset-inline-start` 前提（選択列幅の既定値 `2.5rem`）へ一致させる
    /// `box-sizing: border-box; padding-inline: 0;
    /// width: var(--fandhe-data-table-select-width, 2.5rem);`
    /// オーバーライドが block 固有属性
    /// `data-blocks-table-sortable-bulk-select-cell` に適用されて
    /// いることを固定する（通常幅での重なり回帰防止、
    /// Codex/Cursor Bugbot 指摘・イシュー #2945 PR #3393。`width` 明示は
    /// 追加レビュー分: `box-sizing`/`padding-inline: 0` だけでは実幅が
    /// `inset-inline-start` の前提どおり `2.5rem` になる保証がない）。
    /// CSS 側のセレクタが実際に描画される DOM 属性
    /// （`table::column_header`/`table::cell` が強制する
    /// `data-scope="table"`）と一致しない旧セレクタ
    /// （`data-scope="data-table"`）への回帰を防ぐ。
    #[test]
    fn select_column_width_override_matches_toolbar_inset_assumption() {
        assert!(LAYOUT_CSS.contains(
            "[data-blocks-table-sortable-bulk-select-cell] {\n  box-sizing: border-box;\n  padding-inline: 0;\n  width: var(--fandhe-data-table-select-width, 2.5rem);\n  text-align: center;\n}"
        ));
        assert!(!LAYOUT_CSS.contains("data-scope=\"data-table\""));
    }

    /// 選択列セルの中央寄せを固定する。`table::column_header`/`table::cell`
    /// が強制する `data-scope="table"` の下では
    /// `data_table::select_all`/`select_row` 共有 base の `text-align:
    /// center` が実際の DOM に適用されない（セレクタが
    /// `data-scope="data-table"` を要求するため）。`padding-inline: 0` で
    /// 左右余白を消した状態のまま放置すると、チェックボックス
    /// （`inline-flex`）が `2.5rem` 幅の列内で左寄せのまま描画される
    /// ため、`select-cell` 局所 CSS で `text-align: center` を明示する
    /// （Cursor Bugbot 指摘、イシュー #2945 PR #3393 レビュー）。
    #[test]
    fn select_cell_override_centers_checkbox_horizontally() {
        assert!(LAYOUT_CSS.contains(
            "[data-blocks-table-sortable-bulk-select-cell] {\n  box-sizing: border-box;\n  padding-inline: 0;\n  width: var(--fandhe-data-table-select-width, 2.5rem);\n  text-align: center;\n}"
        ));
    }

    /// 見出し行の `th` を `box-sizing: border-box` にし、`height: 3rem` が
    /// `column-header` base の `padding`（`--fandhe-space-3`
    /// `--fandhe-space-4`）+ `border-bottom`（1px）込みの外形高さになる
    /// ことを固定する。`content-box`（既定）のままだと `height: 3rem` は
    /// content 領域のみの高さで、実際の `th` はそれより高くなり、同じ
    /// `height: 3rem` の一括操作ツールバーの下端からソートラベルが
    /// はみ出して透けて見える（Cursor Bugbot 指摘、イシュー #2945 PR
    /// #3393 レビュー、L606-609）。
    #[test]
    fn header_row_th_uses_border_box_to_match_toolbar_height() {
        assert!(LAYOUT_CSS.contains("thead th {\n  box-sizing: border-box;\n  height: 3rem;\n}"));
    }

    /// 選択列の見出しセル・各行セルの双方に
    /// `data-blocks-table-sortable-bulk-select-cell` が付与されていることを
    /// 固定する（行数 4 + 見出し 1 = 5 回、両パネル分で 10 回。
    /// イシュー #2945 PR #3393 追加レビュー）。
    #[test]
    fn select_cell_hook_appears_on_header_and_every_row_in_both_panels() {
        let html = html();
        let rows_per_panel = rows_data().len();
        let expected = (rows_per_panel + 1) * 2;
        assert_eq!(
            html.matches("data-blocks-table-sortable-bulk-select-cell")
                .count(),
            expected
        );
    }

    /// 全行選択チェックボックスは選択中パネルでも静的な `tabindex="-1"` を
    /// 持たない（通常幅ではツールバーに覆われず見えたまま操作可能である
    /// ため）。狭幅で覆われる間の不可視化は `visibility: hidden`（CSS）が
    /// 担い、Tab 移動からの除外も自動的に伴う（Codex/Cursor Bugbot 指摘、
    /// イシュー #2945 PR #3393）。
    /// 選択中パネルの並び替え可能な列見出し（名前・ステータス）は、
    /// `tabindex="-1"` だけでなく `data-blocks-table-sortable-bulk-covered`
    /// フックも持ち、CSS 側で `visibility: hidden` が対応付けられている
    /// ことを固定する。`tabindex="-1"` のみではアクセシビリティツリーから
    /// 除外されずスクリーンリーダーから発見・操作可能なまま残るための
    /// 回帰防止（github-actions 指摘、イシュー #2945 PR #3393 追加レビュー）。
    #[test]
    fn covered_sort_triggers_are_hidden_from_accessibility_tree() {
        let html = html();
        assert_eq!(
            html.matches("data-blocks-table-sortable-bulk-covered")
                .count(),
            2,
            "選択中パネルの名前・ステータス 2 列分のみ付与される"
        );
        assert!(LAYOUT_CSS
            .contains("[data-blocks-table-sortable-bulk-covered] {\n  visibility: hidden;\n}"));
    }

    /// `covered_sort_triggers_are_hidden_from_accessibility_tree` が固定する
    /// `visibility: hidden` は `sort_trigger`（button）の唯一の子である
    /// 列名テキストごと不可視化するため、`th` の中身がまるごと
    /// アクセシビリティツリーから消え、支援技術で表を読み上げた際に
    /// 名前・ステータス列の見出し名が失われていた（Codex 指摘 P1、イシュー
    /// #2945 PR #3393）。覆われている間だけ clip 手法（`display: none`/
    /// `visibility: hidden` を使わない）のフォールバックテキストを `th` へ
    /// 追加して埋めたことを固定する。「未選択」パネルは可視ボタンの
    /// テキストが唯一の情報源のため、このフォールバックを持たない
    /// （二重読み上げの防止）。
    #[test]
    fn covered_headers_keep_column_name_accessible_via_fallback_label() {
        let html = html();
        assert_eq!(
            html.matches("data-blocks-table-sortable-bulk-header-fallback-label")
                .count(),
            2,
            "選択中パネルの名前・ステータス 2 列分のみ付与される"
        );
        assert!(html.contains(
            r#"<span data-blocks-table-sortable-bulk-header-fallback-label="">名前</span>"#
        ));
        assert!(html.contains(
            r#"<span data-blocks-table-sortable-bulk-header-fallback-label="">ステータス</span>"#
        ));
        assert!(LAYOUT_CSS.contains(
            "[data-blocks-table-sortable-bulk-header-fallback-label] {\n  position: absolute;"
        ));
    }

    #[test]
    fn select_all_checkbox_has_no_static_tabindex() {
        let html = html();
        assert!(!html.contains(r#"aria-label="Select all rows" tabindex="-1""#));
        assert!(!html.contains(r#"tabindex="-1" aria-label="Select all rows""#));
    }

    /// 選択中パネルの見出しセル（全行選択チェックボックスの `th`）だけが
    /// `select-header` フックを持ち、CSS の狭幅 `@container` 規則内で
    /// `visibility: hidden` が対応付けられていることを固定する。
    #[test]
    fn select_header_hook_appears_once_and_is_hidden_only_in_narrow_container_query() {
        let html = html();
        assert_eq!(
            html.matches("data-blocks-table-sortable-bulk-select-header")
                .count(),
            1
        );
        assert!(LAYOUT_CSS.contains(
            "[data-blocks-table-sortable-bulk-select-header] {\n    visibility: hidden;\n  }"
        ));
    }
}
