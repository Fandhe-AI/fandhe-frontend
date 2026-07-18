//! シナリオ 3「機能追加」の回帰テスト（TASK-13.4d、#147。親 TASK-13.4 #143）。
//!
//! `docs/scenario-regression-design.md`（TASK-13.4a・#144）§4.1 行 3・§4.2・§4.3・
//! §4.4 を単一の情報源とする。PoC-7 土台
//! （`docs/spec/03-poc/ai-self-maintenance/scenarios/feature-search/`）の
//! 実測値（`impact.json`: `affected_crates: ["rws-server"]` / `breaking_risk:
//! medium` / ルート 3 件 / `requires_human_approval: true`、`gate.json`: 全
//! チェック PASS（新規テスト込み））を、製品 CLI（`fw`）に対する統合テストとして
//! 再現する。
//!
//! # フィクスチャ構成
//!
//! `common::write_scenario_workspace` で 2 クレート構成
//! （`app` = `scenario-fixture-app` lib クレート、`server` = `rws-server` bin
//! クレート、`server` は `app` へ path 依存）を生成する。「機能追加」の
//! before/after は `app::search_page`（タイトル部分一致検索）を app 側に、
//! `search_handler` + `.route("/search", ...)` を server 側にそれぞれ追加する
//! 変更として表現する（`common::replace_unique` による一意な部分文字列置換）。
//!
//! `search_page` の定義は `app/src/lib.rs`（定義ファイル）に閉じ、`server` 側
//! からのみ参照させる。`impact::analyze` の `scan_usages` は定義元ファイル
//! 自身を使用箇所から除外するため、これにより `affected_crates` が
//! `["rws-server"]` の 1 件に収まる（設計文書 §4.1 行 3 の前提）。
//!
//! # 期待値の導出根拠（`cli/src/impact.rs`）
//!
//! - `affected_files` = `server/src/main.rs` のみ → seeds `{rws-server}` →
//!   `rws-server` に依存する他クレートがないため逆依存閉包も `{rws-server}`
//!   のまま → `affected_crates: ["rws-server"]`
//! - `judge_breaking_risk`: 影響クレート 1 件・クライアント境界クレート
//!   （`rws-wasm-client`/`rws-wasm-full`/`rws-wasm-thin`）を含まない →
//!   `medium`
//! - `requires_human_approval(medium, routes 非空, ambiguous=false)` → `true`
//! - `affected_route_paths`: 影響ファイル `server/src/main.rs` 内の全ルートを
//!   BTreeSet 昇順で列挙 → `["/", "/items/:id", "/search"]`
//!   （ファイル単位の過検知は仕様。既存 2 ルートも含まれる）

use crate::common::{
    cargo_deny_available, check_passed, json_array_contains_str, json_bool_field,
    json_string_field, replace_unique, run_fw, write_scenario_workspace, MemberSpec,
};

/// `app`（`scenario-fixture-app`）クレートのベースラインソース。PoC-7
/// `target-project` の `Item`/`find_item` 相当に `list_page` を加えた最小構成。
/// `search_page`（機能追加対象）はまだ存在しない
/// （`search_page_is_absent_from_baseline` の対照群を成立させる前提）。
fn app_baseline_src() -> String {
    r#"//! シナリオ 3（機能追加、TASK-13.4d・#147）フィクスチャ用の最小 app crate。
//! `find_item`/`list_page` はベースライン（改修前）から存在し、
//! `search_page`（タイトル部分一致検索）は変更適用後にのみ追加される
//! （`scenario3_feature.rs` が before/after を切り替える）。

pub struct Item {
    pub id: String,
    pub title: String,
}

pub fn find_item<'a>(items: &'a [Item], target_id: &str) -> Option<&'a Item> {
    items.iter().find(|it| it.id == target_id)
}

pub fn list_page(items: &[Item]) -> String {
    items
        .iter()
        .map(|it| it.title.clone())
        .collect::<Vec<_>>()
        .join(", ")
}

// SCENARIO3_SEARCH_INSERTION_POINT

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_items() -> Vec<Item> {
        vec![
            Item {
                id: "1".to_string(),
                title: "widget".to_string(),
            },
            Item {
                id: "2".to_string(),
                title: "gadget".to_string(),
            },
        ]
    }

    #[test]
    fn find_item_locates_existing_id() {
        let items = sample_items();
        assert_eq!(
            find_item(&items, "1").map(|it| it.title.as_str()),
            Some("widget")
        );
    }

    // SCENARIO3_TEST_INSERTION_POINT
}
"#
    .to_string()
}

/// `app_baseline_src` に「機能追加」（`search_page`）を適用した後のソース。
/// PoC-7 gate.json の「新規テスト込みで通過」を再現するため、公開 API に加えて
/// `#[cfg(test)]` 側にも `search_items` の新規ユニットテストを追加する。
fn app_after_src() -> String {
    let with_search_fn = replace_unique(
        &app_baseline_src(),
        "// SCENARIO3_SEARCH_INSERTION_POINT",
        r#"/// タイトルの部分一致検索（大文字小文字を区別しない）。
/// `server` crate の `search_handler`（`server/src/main.rs`）から呼ばれる、
/// シナリオ 3「機能追加」の対象 API。
pub fn search_items<'a>(items: &'a [Item], query: &str) -> Vec<&'a Item> {
    let needle = query.to_lowercase();
    items
        .iter()
        .filter(|it| it.title.to_lowercase().contains(&needle))
        .collect()
}

/// `search_items` の結果をカンマ区切りのページ本文へ整形する。
pub fn search_page(items: &[Item], query: &str) -> String {
    search_items(items, query)
        .into_iter()
        .map(|it| it.title.clone())
        .collect::<Vec<_>>()
        .join(", ")
}"#,
    );

    replace_unique(
        &with_search_fn,
        "    // SCENARIO3_TEST_INSERTION_POINT",
        r#"    #[test]
    fn search_items_filters_by_title_substring() {
        let items = sample_items();
        let found = search_items(&items, "WID");
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].id, "1");
    }"#,
    )
}

/// `server`（`rws-server`）クレートのベースラインソース。依存ゼロのスタブ
/// `Router`（`.route("<path>", handler)` の実体を持たせるだけの最小実装）に
/// 既存 2 ルート（`/`, `/items/:id`）を登録する。`search_page` はまだ
/// 参照しない。
fn server_baseline_src() -> String {
    r#"//! シナリオ 3（機能追加、TASK-13.4d・#147）フィクスチャ用の最小 server crate
//! （`rws-server` 役）。実依存を持たないスタブ `Router` に `app` crate
//! （`scenario-fixture-app`）のハンドラを登録する。`fw impact` のルート抽出器
//! （`cli/src/routes.rs`）が対象とする `.route("<path>", handler)` 構文を実体
//! として持つ。

struct Router;

impl Router {
    fn route(self, _path: &str, _handler: fn() -> String) -> Self {
        self
    }
}

fn list_handler() -> String {
    let items = vec![
        scenario_fixture_app::Item {
            id: "1".to_string(),
            title: "widget".to_string(),
        },
        scenario_fixture_app::Item {
            id: "2".to_string(),
            title: "gadget".to_string(),
        },
    ];
    scenario_fixture_app::list_page(&items)
}

fn detail_handler() -> String {
    let items = vec![scenario_fixture_app::Item {
        id: "1".to_string(),
        title: "widget".to_string(),
    }];
    scenario_fixture_app::find_item(&items, "1")
        .map(|it| it.title.clone())
        .unwrap_or_default()
}

// SCENARIO3_ROUTE_INSERTION_POINT

fn main() {
    let _router = Router
        .route("/", list_handler)
        .route("/items/:id", detail_handler);
}
"#
    .to_string()
}

/// `server_baseline_src` に「機能追加」（`/search` ルート）を適用した後のソース。
fn server_after_src() -> String {
    let with_handler = replace_unique(
        &server_baseline_src(),
        "// SCENARIO3_ROUTE_INSERTION_POINT",
        r#"fn search_handler() -> String {
    let items = vec![
        scenario_fixture_app::Item {
            id: "1".to_string(),
            title: "widget".to_string(),
        },
        scenario_fixture_app::Item {
            id: "2".to_string(),
            title: "gadget".to_string(),
        },
    ];
    scenario_fixture_app::search_page(&items, "wid")
}"#,
    );

    replace_unique(
        &with_handler,
        "        .route(\"/items/:id\", detail_handler);",
        "        .route(\"/items/:id\", detail_handler)\n        .route(\"/search\", search_handler);",
    )
}

/// フィクスチャ共通の member 構成（`source` のみ before/after で差し替える）。
fn members_with(app_src: String, server_src: String) -> Vec<MemberSpec> {
    vec![
        MemberSpec {
            dir: "app",
            package_name: "scenario-fixture-app",
            role: "component",
            is_bin: false,
            path_deps: &[],
            source: app_src,
        },
        MemberSpec {
            dir: "server",
            package_name: "rws-server",
            role: "server-entrypoint",
            is_bin: true,
            path_deps: &["app"],
            source: server_src,
        },
    ]
}

/// テスト 1（新規性の対照群）: ベースライン（改修前、`search_page` 不在）に
/// 対して `fw impact search_page` が `SymbolNotFound`（終了コード 1、
/// `cli/src/main.rs::run_impact` の終了コード契約）で拒否されることを確認する。
/// 「シナリオ 3 が真に加法的な機能追加である」ことをフィクスチャ前提として
/// 固定する。
#[test]
fn search_page_is_absent_from_baseline() {
    let members = members_with(app_baseline_src(), server_baseline_src());
    let project = write_scenario_workspace(
        "feature-baseline-impact",
        &members,
        Some(("server", "rws-router-v1")),
    );
    let (code, stdout, stderr) = run_fw("impact", &["search_page"], &project);

    assert_eq!(
        code, 1,
        "ベースラインには search_page が存在しないため SymbolNotFound（終了コード 1）\
         のはず: stdout={stdout} stderr={stderr}"
    );
}

/// テスト 2: 変更適用後（`search_page` 追加後）に `fw impact search_page` が
/// PoC-7 実測値・製品スキーマ差分表（設計文書 §4.1）どおりの JSON を返すことを
/// 検証する。
#[test]
fn feature_addition_impact_reports_medium_risk_and_new_route() {
    let members = members_with(app_after_src(), server_after_src());
    let project = write_scenario_workspace(
        "feature-after-impact",
        &members,
        Some(("server", "rws-router-v1")),
    );
    let (code, stdout, stderr) = run_fw("impact", &["search_page"], &project);

    assert_eq!(
        code, 0,
        "変更適用後は search_page が定義済みのため fw impact は成功するはず: \
         stdout={stdout} stderr={stderr}"
    );

    assert_eq!(
        json_string_field(&stdout, "breaking_risk"),
        Some("medium".to_string()),
        "影響クレートは rws-server の 1 件のみのため medium のはず: stdout={stdout}"
    );
    assert_eq!(
        json_bool_field(&stdout, "requires_human_approval"),
        Some(true),
        "medium リスク・影響ルート非空のため人間承認要のはず: stdout={stdout}"
    );
    assert_eq!(
        json_bool_field(&stdout, "ambiguous"),
        Some(false),
        "search_page の定義元は app crate の 1 箇所のみのはず: stdout={stdout}"
    );

    // affected_crates はシリアライズが決定的・昇順であるため厳密一致で検証する
    // （rws-server 1 件のみ = クライアント境界クレート非含有も同時に保証する）。
    assert!(
        stdout.contains(r#""affected_crates":["rws-server"]"#),
        "affected_crates は rws-server 1 件のみのはず: stdout={stdout}"
    );

    // affected_routes も厳密一致で検証する（既存 2 ルート + 新設 /search の
    // ファイル単位過検知込みで 3 件、BTreeSet 昇順）。
    assert!(
        stdout.contains(r#""affected_routes":["/","/items/:id","/search"]"#),
        "affected_routes は 3 ルート（既存 2 件 + 新設 /search）のはず: stdout={stdout}"
    );
    assert!(
        json_array_contains_str(&stdout, "affected_routes", "/search"),
        "新設ルート /search が affected_routes に含まれるはず: stdout={stdout}"
    );

    assert!(
        stdout.contains(
            r#""verdict":"requires human approval (impact spans multiple crates or public routes)""#
        ),
        "verdict は固定英語文字列（設計文書 §4.1 差分表 D1）のはず: stdout={stdout}"
    );
}

/// テスト 3: (a) ベースライン（対照群）・(b) 変更適用後のいずれも `fw gate` の
/// コア 4 チェック（`type_check`/`default_escape_check`/`lint`/`test`）を
/// 通過することを確認する。`policy` チェックのみ cargo-deny の導入有無で
/// 環境ごとに挙動が変わるため、`baseline_fixture_passes_gate_core_checks`
/// （`main.rs`）と同じ fail-closed 契約を検証する
/// （`#[ignore]`・スキップによる吸収は行わない、`coding-rust.md` 準拠）。
#[test]
fn feature_addition_passes_gate_core_checks() {
    // (a) 変更前（対照群）: 拡張フィクスチャ自体が健全であることを確認し、
    // 変更後 PASS が空虚な検証になっていないことを保証する。
    {
        let members = members_with(app_baseline_src(), server_baseline_src());
        let project = write_scenario_workspace(
            "feature-baseline-gate",
            &members,
            Some(("server", "rws-router-v1")),
        );
        let (_code, stdout, stderr) = run_fw("gate", &[], &project);
        for name in ["type_check", "default_escape_check", "lint", "test"] {
            assert_eq!(
                check_passed(&stdout, name),
                Some(true),
                "変更前フィクスチャで {name} が失敗した（対照群が壊れている）: \
                 stdout={stdout} stderr={stderr}"
            );
        }
    }

    // (b) 変更後: コア 4 チェックが新規テスト込みで通過する。
    let members = members_with(app_after_src(), server_after_src());
    let project = write_scenario_workspace(
        "feature-after-gate",
        &members,
        Some(("server", "rws-router-v1")),
    );
    let (code, stdout, stderr) = run_fw("gate", &[], &project);

    for name in ["type_check", "default_escape_check", "lint", "test"] {
        assert_eq!(
            check_passed(&stdout, name),
            Some(true),
            "変更後フィクスチャで {name} が失敗した: stdout={stdout} stderr={stderr}"
        );
    }

    // (c) policy チェックは環境差吸収（設計文書 §4.3 厳守）。
    if cargo_deny_available() {
        assert_eq!(
            code, 0,
            "cargo-deny 導入環境では変更後フィクスチャは PASS するはず: stdout={stdout}"
        );
        assert!(
            stdout.contains(r#""gate_result":"PASS""#),
            "stdout={stdout}"
        );
        assert_eq!(
            check_passed(&stdout, "policy"),
            Some(true),
            "stdout={stdout}"
        );
    } else {
        // cargo-deny 未導入環境（本リポジトリ CI 相当）では policy のみ
        // fail-closed で failed になり、他の 4 チェックは通過したまま全体として
        // BLOCKED になる、という fail-closed 契約を確認する。
        assert_eq!(
            code, 1,
            "cargo-deny 未導入環境では policy の fail-closed により BLOCKED \
             （終了コード 1）のはず: stdout={stdout}"
        );
        assert!(
            stdout.contains(r#""gate_result":"BLOCKED""#),
            "stdout={stdout}"
        );
        assert_eq!(
            check_passed(&stdout, "policy"),
            Some(false),
            "stdout={stdout}"
        );
    }
}
