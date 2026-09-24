//! Application 区分の block 登録点（イシュー #2733 の 4 区分の一つ、
//! カテゴリ別モジュール分割はイシュー #2734）。
//!
//! 本区分に属する全カテゴリの `mod` 宣言と [`blocks`]（各カテゴリの
//! `blocks()` を連結するだけの集約関数）を持つ。**新規カテゴリの追加や
//! 既存カテゴリへの block 追加はここを経由しない**（カテゴリ側
//! `<category>.rs`／`<category>/mod.rs` が完結して担う）。本ファイルを
//! 変更するのはカテゴリの「卒業」（空雛形 → ディレクトリ化、
//! `docs/design/docs-site-blocks-section.md` §18 参照）のときのみ。

mod action_panel;
mod ai_chat;
mod app_shell;
mod auth;
mod calendar;
mod card;
mod card_heading;
mod chart;
mod command_palette;
mod dashboard;
mod description_list;
mod dialog;
mod drawer;
mod empty_state;
mod feed;
mod form_layout;
mod grid_list;
mod help_center;
mod list;
mod navbar;
mod notification;
mod onboarding;
mod page_heading;
mod profile;
mod settings;
mod sidebar;
mod table;

use crate::blocks::Block;

pub(super) fn blocks() -> Vec<Block> {
    let mut items = Vec::new();
    items.extend(app_shell::blocks());
    items.extend(sidebar::blocks());
    items.extend(navbar::blocks());
    items.extend(page_heading::blocks());
    items.extend(card_heading::blocks());
    items.extend(list::blocks());
    items.extend(table::blocks());
    items.extend(grid_list::blocks());
    items.extend(description_list::blocks());
    items.extend(calendar::blocks());
    items.extend(feed::blocks());
    items.extend(form_layout::blocks());
    items.extend(auth::blocks());
    items.extend(settings::blocks());
    items.extend(action_panel::blocks());
    items.extend(empty_state::blocks());
    items.extend(notification::blocks());
    items.extend(dialog::blocks());
    items.extend(drawer::blocks());
    items.extend(command_palette::blocks());
    items.extend(onboarding::blocks());
    items.extend(card::blocks());
    items.extend(profile::blocks());
    items.extend(chart::blocks());
    items.extend(dashboard::blocks());
    items.extend(ai_chat::blocks());
    items.extend(help_center::blocks());
    items
}
