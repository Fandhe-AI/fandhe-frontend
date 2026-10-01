//! Application / Settings カテゴリの block 登録点。カテゴリ別のモジュール構成と
//! 「カテゴリの卒業」手順は `docs/design/docs-site-blocks-section.md` §18
//! 参照（イシュー #2734）。
//!
//! 本カテゴリ配下の block 実装モジュールを宣言し、[`blocks`] で集約する。
//! 新規 block を追加する際は本ファイルへ `mod` 宣言と `blocks()` への追記を
//! 行うだけでよく、`super`（`application`）側・トップレベル `crate::blocks` 側の
//! 変更は不要（並列 PR 間の衝突をカテゴリ内へ閉じ込めるための構造）。
//! block ごとの追加経緯は git 履歴と PR を正とし、本コメントには書かない
//! （並列 PR が同じ行を書き換えて競合するため、§18 参照）。

mod settings_api_key_created;
mod settings_api_keys_table;
mod settings_billing_overview;
mod settings_billing_usage;
mod settings_event_accordion;
mod settings_export_data;
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
mod settings_page_tabs;
mod settings_preferences;
mod settings_profile_form;
mod settings_share_link;
mod settings_share_members;
mod settings_switch_sections;
mod settings_team_invite;
mod settings_team_table;
mod settings_webhook_detail;
mod settings_webhook_form;
mod settings_webhook_stats;
mod settings_webhook_wizard;

use crate::blocks::Block;

pub(super) fn blocks() -> Vec<Block> {
    vec![
        settings_api_key_created::BLOCK,
        settings_api_keys_table::BLOCK,
        settings_billing_overview::BLOCK,
        settings_billing_usage::BLOCK,
        settings_event_accordion::BLOCK,
        settings_export_data::BLOCK,
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
        settings_page_tabs::BLOCK,
        settings_preferences::BLOCK,
        settings_profile_form::BLOCK,
        settings_share_link::BLOCK,
        settings_share_members::BLOCK,
        settings_switch_sections::BLOCK,
        settings_team_invite::BLOCK,
        settings_team_table::BLOCK,
        settings_webhook_detail::BLOCK,
        settings_webhook_form::BLOCK,
        settings_webhook_stats::BLOCK,
        settings_webhook_wizard::BLOCK,
    ]
}
