//! Application / Settings カテゴリの block 登録点（イシュー #2734 で雛形新設、
//! イシュー #2984 で最初の block（[`settings_billing_overview`]）を追加し
//! ディレクトリ化して卒業、イシュー #2985 で
//! [`settings_billing_usage`]、イシュー #2986 で
//! [`settings_event_accordion`]、イシュー #2982 で
//! [`settings_api_key_created`]、イシュー #2991 で
//! [`settings_integrations_grid`]、イシュー #2995 で
//! [`settings_integrations_search`]、イシュー #2996 で
//! [`settings_item_cards`]、イシュー #2997 で
//! [`settings_log_table`]、イシュー #2998 で
//! [`settings_notification_matrix`]、イシュー #2999 で
//! [`settings_org_switcher`]、イシュー #3009 で
//! [`settings_preferences`]、イシュー #3010 で
//! [`settings_profile_form`]、イシュー #3001 で
//! [`settings_page_aside_nav`]、イシュー #3004 で
//! [`settings_page_sidebar`] を追加した）。手順は
//! `docs/design/docs-site-blocks-section.md` §18 参照。
//!
//! 本カテゴリ配下の block 実装モジュールを宣言し、[`blocks`] で集約する。
//! 新規 block を追加する際は本ファイルへ `mod` 宣言と `blocks()` への追記を
//! 行うだけでよく、`super`（`application`）側・トップレベル
//! `crate::blocks` 側の変更は不要（並列 PR 間の衝突をカテゴリ内へ閉じ込める
//! ための構造、イシュー #2734）。

mod settings_api_key_created;
mod settings_api_keys_table;
mod settings_billing_overview;
mod settings_billing_usage;
mod settings_event_accordion;
mod settings_integration_detail;
mod settings_integrations_grid;
mod settings_integrations_list;
mod settings_integrations_search;
mod settings_item_cards;
mod settings_log_table;
mod settings_notification_matrix;
mod settings_org_switcher;
mod settings_page_aside_nav;
mod settings_page_sidebar;
mod settings_preferences;
mod settings_profile_form;

use crate::blocks::Block;

pub(super) fn blocks() -> Vec<Block> {
    vec![
        settings_api_key_created::BLOCK,
        settings_api_keys_table::BLOCK,
        settings_billing_overview::BLOCK,
        settings_billing_usage::BLOCK,
        settings_event_accordion::BLOCK,
        settings_integration_detail::BLOCK,
        settings_integrations_grid::BLOCK,
        settings_integrations_list::BLOCK,
        settings_integrations_search::BLOCK,
        settings_item_cards::BLOCK,
        settings_log_table::BLOCK,
        settings_notification_matrix::BLOCK,
        settings_org_switcher::BLOCK,
        settings_page_aside_nav::BLOCK,
        settings_page_sidebar::BLOCK,
        settings_preferences::BLOCK,
        settings_profile_form::BLOCK,
    ]
}
