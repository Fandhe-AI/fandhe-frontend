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
//! Forms A（#1024）が [`forms_a::SPECS`] を、Forms C・日付・状態表示（#1026）が
//! [`forms_c_date_status::SPECS`] を、Overlay / Disclosure（#1027）が
//! [`overlay_disclosure::SPECS`] を充填済みである。Forms A 対象 11 部品
//! （angle-slider / checkbox / checkbox-group / color-picker / combobox /
//! editable / field / fieldset / file-upload / image-cropper / listbox）と
//! Forms C・日付・状態表示対象 10 部品、Overlay / Disclosure 対象 10 部品の
//! `/primitives/<kebab>/` は Features / API Reference 引数表 / Examples /
//! Accessibility の 4 節を持つ。残り 3 カテゴリは以下の issue が引き続き
//! 担当する（`crate::primitives_catalog::PrimitiveCategory::spec_issue` と
//! 同一の対応。未充填カテゴリの `/primitives/<kebab>/` は
//! Markdown 原稿のみで生成される、[`crate::component_page::generated_content`]
//! が `ComponentPageSpec::EMPTY` を返すため）。
//!
//! | カテゴリ | 対応 issue | 状態 |
//! |---|---|---|
//! | Forms A（11 件） | #1024 | 充填済み |
//! | Forms B（11 件） | #1025 | 未充填 |
//! | Forms C・日付・状態表示（10 件） | #1026 | 充填済み |
//! | Overlay / Disclosure（10 件） | #1027 | 充填済み |
//! | Navigation（11 件） | #1028 | 未充填 |
//! | Data Display / Utilities（10 件） | #1029 | 未充填 |
//!
//! 追加時は本モジュール配下にカテゴリ別サブモジュールを新設し、
//! `(path, ComponentPageSpec)` のテーブルを 1 モジュール 1 定数で持たせたうえで
//! 本ファイルの [`SPEC_TABLES`] へ 1 行追記する（`/themes/` 側
//! `component_page::SPEC_TABLES` と同じ集約方式。テーブル**間**の path 重複は
//! `component_page.rs` 内の `spec_tables_have_no_cross_table_duplicate_paths`
//! が担う検査対象へ本レジストリも含める想定）。

use crate::component_page::ComponentPageSpec;

pub mod forms_a;
pub mod forms_c_date_status;
mod overlay_disclosure;

/// `path -> ComponentPageSpec` レジストリを供給するカテゴリ別テーブルの集約。
/// Phase 5 の各 issue はカテゴリ 1 個につき 1 テーブルを追加し、本配列へ
/// 1 行追記する（`crate::component_page::SPEC_TABLES` と同型の集約方式）。
pub const SPEC_TABLES: &[&[(&str, ComponentPageSpec)]] = &[
    forms_a::SPECS,
    forms_c_date_status::SPECS,
    overlay_disclosure::SPECS,
];
