//! `tests/golden_css.rs` が参照する golden 期待値リテラルのサブ
//! モジュール群。各ファイルは `pub const EXPECTED_CSS: &str` のみを
//! 持つ（実装への参照は一切含まない）。ディレクトリ名 `golden/` は
//! `tests/` 直下に置かないことで cargo の統合テスト自動検出の対象
//! から外れる（`tests/support/` と同型の規約）。
//!
//! 更新手順は `docs/internal/wireframe-ui-golden-test-update-guide.md`
//! を参照。

// 基盤（`PARTS` を経由しない実行時生成 CSS 3 件 + `PARTS` 先頭に
// 登録されるグリフ基底 class 1 件）。
pub mod frame_padding;
pub mod icon_glyph;
pub mod size;
pub mod tokens;

// 部品（`docs/design/wireframe-ui-architecture.md` §8 の 49 部品、
// `PARTS` 登録順ではなくアルファベット順）。
pub mod accordion;
pub mod alert;
pub mod annotation;
pub mod avatar;
pub mod brand;
pub mod breadcrumbs;
pub mod button;
pub mod calendar;
pub mod card_basic;
pub mod chart;
pub mod checkbox;
pub mod counter;
pub mod cursor;
pub mod divider;
pub mod emoji;
pub mod file_drop;
pub mod frame;
pub mod grid;
pub mod icon;
pub mod image;
pub mod input;
pub mod link;
pub mod list;
pub mod map;
pub mod media;
pub mod menu;
pub mod modal;
pub mod nav_item;
pub mod pagination;
pub mod paragraph;
pub mod progress;
pub mod question;
pub mod radio;
pub mod ratings;
pub mod rich_text;
pub mod select;
pub mod slider;
pub mod spinner;
pub mod stack;
pub mod stat;
pub mod stepper;
pub mod switch;
pub mod table;
pub mod tabs;
pub mod tag;
pub mod text;
pub mod textarea;
pub mod toast;
pub mod tooltip;
