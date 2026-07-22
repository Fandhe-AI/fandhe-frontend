//! Alert（イシュー #550）: slot recipe styled 部品。root/indicator/content/
//! title/description の 5 パーツで構成する通知バナー。
//!
//! `root` に `role="alert"`（WAI-ARIA live region、ステータスに関わらず固定）
//! を付与する。chakra-ui v3 準拠でステータスごとに `role` を切り替える設計も
//! あり得るが、本イシューでは「注意を要する通知」という `alert` ロールの
//! 意味を全ステータス共通で固定する（`status`（緊急度の低い更新通知）との
//! 使い分けは呼び出し側が [`AlertStatus`] を見て判断する設計としない）。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{SlotRecipe, VariantValue};
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
use fandhe_frontend_headless_ui::{anatomy, role, Anatomy};

/// `data-scope="alert"` を固定した本コンポーネントの anatomy。
const ANATOMY: Anatomy = anatomy("alert");

/// [`SlotRecipe::new`] に渡す slot 一覧。
const SLOTS: &[&str] = &["root", "indicator", "content", "title", "description"];

/// Alert のステータス（既定 `Info`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AlertStatus {
    /// 情報提供（既定）。
    #[default]
    Info,
    /// 成功。
    Success,
    /// 警告。
    Warning,
    /// エラー。
    Error,
}

impl VariantValue for AlertStatus {
    fn axis(self) -> &'static str {
        "status"
    }

    fn value(self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Success => "success",
            Self::Warning => "warning",
            Self::Error => "error",
        }
    }
}

/// Alert の recipe（scope `"alert"`、[`SLOTS`] の 5 パーツ）。
///
/// 公開 API は [`AlertStatus`] のまま変えず（イシュー #606 のスコープ境界:
/// #572 が示した「colorPalette は通常の variant 軸として表現可能」という
/// 設計を Alert では表出させない）、各 status の宣言内で
/// [`crate::recipe::palette_declarations`] が使う `--fandhe-palette-*` 系
/// custom property を対応するセマンティック色へ束ねたうえで
/// `color: var(--fandhe-palette)` を参照する（chakra-ui の status→colorPalette
/// 内部マッピングと同型）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("alert", SLOTS)
        .base(
            "root",
            vec![
                decl("display", "flex"),
                decl("gap", "0.75rem"),
                decl("padding", "1rem"),
                decl("border-radius", "var(--fandhe-radius-md)"),
            ],
        )
        .base("indicator", vec![decl("flex-shrink", "0")])
        .base(
            "content",
            vec![
                decl("display", "flex"),
                decl("flex-direction", "column"),
                decl("gap", "0.25rem"),
            ],
        )
        .base(
            "title",
            vec![decl(
                "font-weight",
                "var(--fandhe-font-font-weight-semibold)",
            )],
        )
        .base(
            "description",
            vec![decl("font-size", "var(--fandhe-font-font-size-sm)")],
        )
        .variant(AlertStatus::Info, "root", status_declarations("info"))
        .variant(AlertStatus::Success, "root", status_declarations("success"))
        .variant(AlertStatus::Warning, "root", status_declarations("warning"))
        .variant(AlertStatus::Error, "root", status_declarations("danger"))
        .default_variant(AlertStatus::Info)
}

/// `status` に対応するセマンティック色トークン名（`theme_name`、例:
/// `AlertStatus::Error` → `"danger"`）から、root slot への宣言列を組み立てる。
///
/// `--fandhe-palette`/`--fandhe-palette-emphasized`/`--fandhe-palette-fg` は
/// [`crate::recipe::palette_declarations`] と同一の名前空間を使うが、Alert は
/// `ColorPalette` variant を公開しないため直接 `palette_declarations` は使わず、
/// `status` → セマンティック色の対応をここで固定する。
fn status_declarations(theme_name: &'static str) -> Vec<crate::css::Declaration> {
    match theme_name {
        "info" => vec![
            decl("--fandhe-palette", "var(--fandhe-color-info)"),
            decl("background", "var(--fandhe-color-bg-subtle)"),
            decl("color", "var(--fandhe-palette)"),
        ],
        "success" => vec![
            decl("--fandhe-palette", "var(--fandhe-color-success)"),
            decl("background", "var(--fandhe-color-bg-subtle)"),
            decl("color", "var(--fandhe-palette)"),
        ],
        "warning" => vec![
            decl("--fandhe-palette", "var(--fandhe-color-warning)"),
            decl("background", "var(--fandhe-color-bg-subtle)"),
            decl("color", "var(--fandhe-palette)"),
        ],
        // "danger" および将来呼び出し漏れに対する fail-closed な既定値。
        _ => vec![
            decl("--fandhe-palette", "var(--fandhe-color-danger)"),
            decl("background", "var(--fandhe-color-bg-subtle)"),
            decl("color", "var(--fandhe-palette)"),
        ],
    }
}

/// Alert の静的 CSS 全文。
#[must_use]
pub fn css() -> String {
    recipe().css()
}

/// root パーツを組み立てる。`role="alert"` + `status` に応じたクラスを
/// 付与する唯一のパーツ（`class_attr::drop_class_attr` により呼び出し側の
/// `class` は除去してから合成する）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::alert::{self, AlertStatus};
///
/// let node = alert::root(AlertStatus::Error, vec![], vec![]);
/// let html = render(&node);
/// assert!(html.contains(r#"role="alert""#));
/// assert!(html.contains("fd-alert--status-error"));
/// ```
#[must_use]
pub fn root<'a>(status: AlertStatus, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let recipe = recipe();
    let class = recipe.variant_classes(&[("status", status.value())]);
    let mut merged: Vec<(&str, &str)> = vec![role("alert"), ("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    ANATOMY.part("root", "div", merged, children)
}

/// indicator パーツ（`<span>`。アイコン等の装飾要素）を組み立てる。
#[must_use]
pub fn indicator<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("indicator", "span", attrs, children)
}

/// content パーツ（`<div>`。title/description をまとめる）を組み立てる。
#[must_use]
pub fn content<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("content", "div", attrs, children)
}

/// title パーツ（`<div>`）を組み立てる。
#[must_use]
pub fn title<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("title", "div", attrs, children)
}

/// description パーツ（`<div>`）を組み立てる。
#[must_use]
pub fn description<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("description", "div", attrs, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};

    #[test]
    fn default_status_is_info_with_role_alert() {
        let html = render(&root(AlertStatus::default(), vec![], vec![]));
        assert!(html.contains(r#"role="alert""#));
        assert!(html.contains("fd-alert--status-info"));
    }

    #[test]
    fn status_enumeration_maps_to_expected_classes() {
        for (status, class) in [
            (AlertStatus::Info, "fd-alert--status-info"),
            (AlertStatus::Success, "fd-alert--status-success"),
            (AlertStatus::Warning, "fd-alert--status-warning"),
            (AlertStatus::Error, "fd-alert--status-error"),
        ] {
            let html = render(&root(status, vec![], vec![]));
            assert!(
                html.contains(&format!("class=\"{class}\"")),
                "status={status:?} -> {html}"
            );
        }
    }

    #[test]
    fn parts_use_expected_tags_and_data_part() {
        assert!(render(&indicator(vec![], vec![]))
            .starts_with(r#"<span data-scope="alert" data-part="indicator""#));
        assert!(render(&content(vec![], vec![]))
            .starts_with(r#"<div data-scope="alert" data-part="content""#));
        assert!(render(&title(vec![], vec![]))
            .starts_with(r#"<div data-scope="alert" data-part="title""#));
        assert!(render(&description(vec![], vec![]))
            .starts_with(r#"<div data-scope="alert" data-part="description""#));
    }

    #[test]
    fn composed_alert_snapshot() {
        let node = root(
            AlertStatus::Warning,
            vec![],
            vec![content(
                vec![],
                vec![
                    title(vec![], vec![text("Heads up")]),
                    description(vec![], vec![text("Something needs attention")]),
                ],
            )],
        );
        let html = render(&node);
        assert_eq!(
            html,
            concat!(
                r#"<div data-scope="alert" data-part="root" role="alert" class="fd-alert--status-warning">"#,
                r#"<div data-scope="alert" data-part="content">"#,
                r#"<div data-scope="alert" data-part="title">Heads up</div>"#,
                r#"<div data-scope="alert" data-part="description">Something needs attention</div>"#,
                r#"</div>"#,
                r#"</div>"#,
            )
        );
    }

    #[test]
    fn caller_class_attr_on_root_is_dropped_not_duplicated() {
        let html = render(&root(
            AlertStatus::default(),
            vec![("class", "attacker-controlled")],
            vec![],
        ));
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(!html.contains("attacker-controlled"));
    }

    #[test]
    fn xss_payload_in_title_children_is_escaped() {
        let html = render(&title(vec![], vec![text("<script>alert(1)</script>")]));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
    }

    /// イシュー #606: 公開 API（[`AlertStatus`]）のクラス出力は不変のまま、
    /// 内部で status ごとに `--fandhe-palette` を対応するセマンティック色へ
    /// 束ね、radii トークンを参照することを固定する。
    #[test]
    fn css_output_declares_status_palette_mapping_and_radius_token() {
        let out = css();
        assert!(out.contains("border-radius: var(--fandhe-radius-md);"));
        assert!(out.contains("--fandhe-palette: var(--fandhe-color-info)"));
        assert!(out.contains("--fandhe-palette: var(--fandhe-color-success)"));
        assert!(out.contains("--fandhe-palette: var(--fandhe-color-warning)"));
        assert!(out.contains("--fandhe-palette: var(--fandhe-color-danger)"));
        assert!(out.contains("color: var(--fandhe-palette);"));
    }
}
