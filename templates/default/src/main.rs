//! fandhe-frontend フレームワークの標準プロジェクトテンプレート（最小骨格）。
//!
//! # 役割・契約
//!
//! `templates/default/tests/negative_type_error.rs` と
//! `xtask/tests/template_negative_type_error.rs`（TASK-4.4 / REQ-4 受け入れ
//! 基準 3「型的に不正な AI 生成コードが `cargo check` の段階で機械的に
//! 弾かれること」）が、本ファイルを正例（ベースライン）として参照する。
//! 負例テストは本ファイルの内容を文字列置換で改変した一時プロジェクトに
//! 対して `cargo check` を実行し、型不正コードが検出されることを確認する。
//! したがって `find_item` の比較式（`it.id == target_id`）や `main` の
//! シグネチャを変更する場合は、上記 2 ファイルの前提（注入対象の文字列）が
//! 崩れないか確認すること。
//!
//! この最小構成は TASK-4.4 の成果物（負例検出テスト）を成立させるために
//! 必要な最小限のものであり、fandhe-frontend-core への依存・SSR/CSR の実体は持たない
//! （本格的なテンプレート骨格整備は別タスクのスコープ）。

#![forbid(unsafe_code)]

/// アイテム一覧から検索する対象の最小データ型。
///
/// `id` は文字列 ID として管理する。負例テストは、この `id: String` と
/// 整数リテラルを比較する型不正コード（PoC-7 の `negative-type-error`
/// ケース相当）を注入し、`cargo check` が `error[E0277]` で検出することを
/// 確認する。
struct Item {
    id: String,
    name: String,
}

/// 指定した id に一致する `Item` を線形探索で返す。
///
/// 呼び出し元（本ファイルの `main`）は返却された `Option<&Item>` を
/// そのまま表示に用いる。既定エスケープ済み文字列生成は fandhe-frontend-core 側の
/// 責務であり、本関数はプレーンなデータ探索のみを担う。
fn find_item<'a>(items: &'a [Item], target_id: &str) -> Option<&'a Item> {
    items.iter().find(|it| it.id == target_id)
}

fn main() {
    let items = vec![
        Item {
            id: "item-1".to_string(),
            name: "first".to_string(),
        },
        Item {
            id: "item-2".to_string(),
            name: "second".to_string(),
        },
    ];

    match find_item(&items, "item-2") {
        Some(item) => println!("found: {}", item.name),
        None => println!("not found"),
    }
}
