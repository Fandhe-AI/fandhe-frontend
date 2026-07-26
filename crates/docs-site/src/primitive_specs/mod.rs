//! Primitives（`fandhe-frontend-headless-ui`）63 部品ページの原稿レジストリ
//! （イシュー #1021、親トラッキング #1035 Phase 4）。
//!
//! # 役割・呼び出し文脈
//!
//! [`crate::component_page::spec_for`] が `Layer::Primitives` のときに探索する
//! テーブル集合。`/themes/` 側（[`crate::component_specs`] 系）と同じ
//! [`crate::component_page::ComponentPageSpec`] 型を再利用する（節構成は層で
//! 変わらない。差は「CSS 変数表を出すか」だけであり、原稿データ構造を分ける
//! 理由がない、設計 §5）。
//!
//! # Phase 5（#1024〜#1029）の追加規約
//!
//! カテゴリ別に以下の issue が [`SPEC_TABLES`] を段階的に充填する
//! （`crate::primitives_catalog::PrimitiveCategory::spec_issue` と同一の対応）。
//! 未充填のカテゴリに属するページは、そのカテゴリの issue が landed する
//! までの間 [`crate::component_page::ComponentPageSpec::EMPTY`] のまま
//! （Features/API 引数表/Examples/Accessibility の 4 節が省略された状態）
//! で構わない（fail-closed な既定動作、`component_page.rs` 参照）。
//!
//! | カテゴリ | 対応 issue |
//! |---|---|
//! | Forms A（11 件） | #1024 |
//! | Forms B（11 件） | #1025 |
//! | Forms C・日付・状態表示（10 件） | #1026 |
//! | Overlay / Disclosure（10 件） | #1027 |
//! | Navigation（11 件） | #1028 |
//! | Data Display / Utilities（10 件） | #1029 |
//!
//! 追加時は本モジュール配下にカテゴリ別サブモジュールを新設し、
//! `(path, ComponentPageSpec)` のテーブルを 1 モジュール 1 定数で持たせたうえで
//! 本ファイルの [`SPEC_TABLES`] へ 1 行追記する（`/themes/` 側
//! `component_page::SPEC_TABLES` と同じ集約方式。テーブル**間**の path 重複は
//! `component_page.rs` 内の `spec_tables_have_no_cross_table_duplicate_paths`
//! が担う検査対象へ本レジストリも含める想定）。並列実装される兄弟 issue の
//! PR 間で本ファイルへの追記行が衝突した場合は「両方の行を残す」ことで解決
//! する（1 モジュール 1 issue のため意味的な衝突は生じない）。

use crate::component_page::ComponentPageSpec;

mod data_display_utilities;
mod overlay_disclosure;

/// `path -> ComponentPageSpec` レジストリを供給するカテゴリ別テーブルの集約。
/// Phase 5 の各 issue はカテゴリ 1 個につき 1 テーブルを追加し、本配列へ
/// 1 行追記する（`crate::component_page::SPEC_TABLES` と同型の集約方式）。
pub const SPEC_TABLES: &[&[(&str, ComponentPageSpec)]] =
    &[overlay_disclosure::SPECS, data_display_utilities::SPECS];
