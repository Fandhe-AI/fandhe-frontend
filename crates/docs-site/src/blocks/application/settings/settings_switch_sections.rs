//! `settings-switch-sections` block（イシュー #3015。Application / Settings
//! カテゴリ）。見出し付きセクション内にラベル・説明・右端スイッチの行を
//! 区切り線で並べ、末尾に保存ボタンを置く設定画面の定番レイアウト。主参照
//! R0260（2 セクション構成は R0261、ラジオ群を差し込む版は R0259、
//! ショートカット案内は R0231、カード内に収める版は R0026 の差分を集約）。
//! `_/blocks-intake/` の対応ファイルは本イシュー着手時点で本 worktree に
//! 存在しないため、原稿・本コメントには対応表 ID のみを記す
//! （`settings_notification_matrix`〔イシュー #2998〕と同じ扱い）。
//!
//! # 使用部品
//!
//! `switch` / `field` / `fieldset` / `radio_group` / `card` / `button` /
//! `separator` / `heading` / `kbd` の 9 部品を合成する（[`BLOCK`] の
//! `parts` に一致させる契約、`crates/docs-site/tests/blocks_nav.rs`/
//! `blocks_contract.rs` が検証する）。新しい UI 部品は追加しない。
//!
//! # 3 variant で集約元の差分を表す
//!
//! - `basic`（R0260 + R0261）: セクション 2 つ（通知・セキュリティ）。
//! - `detailed`（R0259 + R0231）: セクション 1 つに、ラジオ群
//!   （通知する条件）とキーボードショートカット案内（`kbd`）を追加で
//!   差し込む。
//! - `card`（R0026）: 同じ行構成を `card` の中に収める。
//!
//! # switch / radio をネイティブ disabled にする理由
//!
//! docs サイトは JS ハイドレーションを行わない静的表示のみのため、
//! `switch::hidden_input`/`radio_group::item_hidden_input` を有効なまま
//! 描画するとラベルクリックでブラウザがネイティブに `checked` を切り替えて
//! しまい、見た目（`data-state`）が SSR 時点の固定値から乖離する
//! （`settings_notification_matrix`/`form_layout_stacked` と同型の判断）。
//! 全 switch・radio item へ `disabled: true` を共有し、既定の
//! `opacity: 0.5`/`cursor: not-allowed` は [`LAYOUT_CSS`] で中和して通常時と
//! 同じ見た目に保つ。
//!
//! # `class` と `data-*` の使い分け
//!
//! `switch::root`/`field::root`/`fieldset::root`/`radio_group::root`/
//! `card::root`/`button::button`/`heading::heading`/`kbd::kbd` はいずれも
//! `drop_class_attr` により呼び出し側 `class` を除去してから内部 variant
//! クラスと合成するため、これらへの CSS フックは `data-*` 属性で渡す
//! （`data-blocks-settings-switch-sections-*`）。`separator::separator` も
//! 同様に `class` を除去するため `data-*` を使う。素の `div`（ラッパー・
//! セクション・行内テキスト）・`card::header`/`body`/`footer`
//! （`class`/`data-*` いずれも透過する）には
//! `class="blocks-settings-switch-sections-*"` を使う。`card::body` も
//! `attrs` を透過するため、`card` variant では行間隔を揃える目的で
//! `class="blocks-settings-switch-sections-section"` を共用する
//! （専用クラスを新設しない）。
//!
//! # 行の横並びは `FieldOrientation::Horizontal` に委譲する
//!
//! `field::root` の `FieldRootProps { orientation: FieldOrientation::
//! Horizontal }` は `display: flex; justify-content: space-between;` を
//! 既に持つ（`crates/pre-styled-ui/src/field.rs` の
//! `horizontal_root_declarations`）。本 block は行レイアウト用の追加 CSS を
//! 持たない（既存トークンで十分なため新規宣言を増やさない）。
//!
//! # id の一意化
//!
//! `field::label`/`field::helper_text` の `for`/`aria-describedby` 連携は
//! `FieldProps::id` からの決定的派生（`"{id}-control"`/`"{id}-helper-text"`）
//! に従う。各行の `id` は `instance`（variant）と行キーで一意化する
//! （`crates/docs-site/tests/blocks_contract.rs` の id 重複禁止検査対策）。
//!
//! # `<form>` を使わない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない静的表示のみで、送信処理・送信先は一切持たない。保存ボタンは
//! `button::button` の既定 `type="button"` のまま用いる。
//!
//! # ダミー素材について
//!
//! セクション名・設定項目名・説明文はすべて架空のもの（実在の人物・企業・
//! PII を含まない）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::field::{self, FieldIds, FieldProps};
use fandhe_frontend_pre_styled_ui::fieldset::{self, FieldsetProps};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::kbd::{self, KbdProps};
use fandhe_frontend_pre_styled_ui::radio_group::{self, RadioGroupProps};
use fandhe_frontend_pre_styled_ui::separator::{self, SeparatorProps};
use fandhe_frontend_pre_styled_ui::switch::{self, SwitchProps};
use fandhe_frontend_pre_styled_ui::{
    ColorPalette, FieldOrientation, FieldRootProps, FieldsetRootProps, Size,
};

/// セクション見出し（H3、`Lg`/`Semibold` で本文より一段目立たせる）。
fn section_heading(title: &'static str) -> Node {
    heading(
        HeadingLevel::H3,
        &HeadingProps {
            size: HeadingSize::Lg,
            weight: HeadingWeight::Semibold,
        },
        vec![],
        vec![text(title)],
    )
}

/// 行間の区切り線。`separator::separator` は `class` を除去するため
/// `data-*` でフックする（モジュール doc「`class` と `data-*` の使い分け」
/// 節参照）。
fn row_separator() -> Node {
    separator::separator(
        &SeparatorProps::default(),
        vec![("data-blocks-settings-switch-sections-separator", "")],
    )
}

/// 「ラベル + 説明 + 右端スイッチ」の 1 行を組み立てる。`instance`
/// （variant 名）と `key`（行を一意にする短い識別子）から id を導出する
/// （モジュール doc「id の一意化」節参照）。
fn switch_row(
    instance: &str,
    key: &str,
    title: &'static str,
    description: &'static str,
    checked: bool,
) -> Node {
    let id = format!("blocks-settings-switch-sections-{instance}-{key}");
    // `field::label`/`field::helper_text` は `FieldProps::id` から
    // `"{id}-control"`/`"{id}-helper-text"` を決定的に導出する
    // （headless `crates/headless-ui/src/field.rs` の既定規則）。switch 側の
    // `hidden_input` id・`aria-describedby` はここで同じ値を組み立てて渡す。
    let control_id = format!("{id}-control");
    let helper_id = format!("{id}-helper-text");
    let field_props = FieldProps {
        id: id.as_str(),
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: true,
    };
    let switch_props = SwitchProps {
        disabled: true,
        ..SwitchProps::default()
    };
    field::root(
        &FieldRootProps {
            orientation: FieldOrientation::Horizontal,
        },
        &field_props,
        vec![("data-blocks-settings-switch-sections-row", "")],
        vec![
            div(
                vec![("class", "blocks-settings-switch-sections-row-text")],
                vec![
                    field::label(&field_props, vec![], vec![text(title)]),
                    field::helper_text(&field_props, vec![], vec![text(description)]),
                ],
            ),
            switch::root(
                Size::Md,
                ColorPalette::Accent,
                checked,
                &switch_props,
                vec![("data-blocks-settings-switch-sections-switch", "")],
                vec![
                    switch::hidden_input(
                        &id,
                        "on",
                        checked,
                        &switch_props,
                        vec![
                            ("id", control_id.as_str()),
                            ("aria-describedby", helper_id.as_str()),
                        ],
                    ),
                    switch::control(
                        checked,
                        &switch_props,
                        vec![],
                        vec![switch::thumb(checked, &switch_props, vec![], vec![])],
                    ),
                ],
            ),
        ],
    )
}

/// セクション（見出し + 行群、行の間に区切り線を挟む。末尾には置かない）。
fn section(title: &'static str, rows: Vec<Node>) -> Node {
    let last_idx = rows.len().saturating_sub(1);
    let mut children = vec![section_heading(title)];
    for (idx, row) in rows.into_iter().enumerate() {
        children.push(row);
        if idx != last_idx {
            children.push(row_separator());
        }
    }
    div(
        vec![("class", "blocks-settings-switch-sections-section")],
        children,
    )
}

/// 保存ボタン。`<form>` を持たないため `button::button` の既定
/// `type="button"` のまま用いる（送信処理は持たない）。
fn save_actions() -> Node {
    div(
        vec![("class", "blocks-settings-switch-sections-actions")],
        vec![button::button(
            &ButtonProps::default(),
            vec![],
            vec![text("保存")],
        )],
    )
}

/// `basic` variant（R0260 + R0261）: 「通知」「セキュリティ」の 2 セクション。
fn basic_instance() -> Node {
    div(
        vec![("data-blocks-settings-switch-sections-variant", "basic")],
        vec![
            section(
                "通知",
                vec![
                    switch_row(
                        "basic",
                        "email",
                        "メール通知",
                        "重要な更新をメールでお知らせします",
                        true,
                    ),
                    switch_row(
                        "basic",
                        "push",
                        "プッシュ通知",
                        "デバイスへプッシュ通知を送信します",
                        false,
                    ),
                    switch_row(
                        "basic",
                        "digest",
                        "週次ダイジェスト",
                        "1 週間の活動をまとめて通知します",
                        false,
                    ),
                ],
            ),
            section(
                "セキュリティ",
                vec![
                    switch_row(
                        "basic",
                        "two-factor",
                        "2 段階認証",
                        "ログイン時に確認コードを要求します",
                        true,
                    ),
                    switch_row(
                        "basic",
                        "new-device",
                        "新しい端末からのサインイン通知",
                        "未知の端末からのログインを通知します",
                        true,
                    ),
                ],
            ),
            save_actions(),
        ],
    )
}

/// メンション通知条件のラジオ 1 件。
fn mention_condition_item(
    checked: bool,
    props: &RadioGroupProps,
    value: &'static str,
    label: &'static str,
) -> Node {
    radio_group::item(
        checked,
        props,
        value,
        vec![("data-blocks-settings-switch-sections-radio-item", "")],
        vec![
            radio_group::item_hidden_input(
                checked,
                props,
                Some("blocks-settings-switch-sections-mention-condition"),
                value,
                vec![],
            ),
            radio_group::item_control(checked, props, vec![]),
            radio_group::item_text(checked, props, vec![], vec![text(label)]),
        ],
    )
}

/// メンション通知の条件を選ぶラジオ群（`fieldset` + `radio_group`）。
fn mention_conditions_fieldset() -> Node {
    let fieldset_id = "blocks-settings-switch-sections-detailed-conditions";
    let legend_id = format!("{fieldset_id}-legend");
    let fieldset_props = FieldsetProps {
        id: fieldset_id,
        disabled: false,
        invalid: false,
        has_helper_text: false,
    };
    let radio_props = RadioGroupProps {
        disabled: true,
        ..RadioGroupProps::default()
    };
    fieldset::root(
        &FieldsetRootProps::default(),
        &fieldset_props,
        vec![("data-blocks-settings-switch-sections-fieldset", "")],
        vec![
            fieldset::legend(&fieldset_props, vec![], vec![text("通知する条件")]),
            radio_group::root(
                Size::Md,
                ColorPalette::Accent,
                true,
                None,
                Some(legend_id.as_str()),
                vec![("data-blocks-settings-switch-sections-radio-group", "")],
                vec![
                    mention_condition_item(true, &radio_props, "all", "すべて"),
                    mention_condition_item(false, &radio_props, "mentions", "自分宛のみ"),
                    mention_condition_item(false, &radio_props, "none", "なし"),
                ],
            ),
        ],
    )
}

/// キーボードショートカットの案内行（`kbd`）。
fn shortcut_hint_row() -> Node {
    div(
        vec![("class", "blocks-settings-switch-sections-row-text")],
        vec![
            div(
                vec![("class", "blocks-settings-switch-sections-shortcut-label")],
                vec![text("キーボードショートカット")],
            ),
            div(
                vec![("class", "blocks-settings-switch-sections-description")],
                vec![
                    text("切り替えのショートカット例（この静的レイアウト例では操作できません）: "),
                    kbd::group(
                        vec![],
                        vec![
                            kbd::kbd(&KbdProps::default(), vec![], vec![text("⌘")]),
                            text("+"),
                            kbd::kbd(&KbdProps::default(), vec![], vec![text("K")]),
                        ],
                    ),
                ],
            ),
        ],
    )
}

/// `detailed` variant（R0259 + R0231）: 「メンション」セクション 1 つに
/// ラジオ群とショートカット案内を差し込む。
fn detailed_instance() -> Node {
    div(
        vec![("data-blocks-settings-switch-sections-variant", "detailed")],
        vec![
            div(
                vec![("class", "blocks-settings-switch-sections-section")],
                vec![
                    section_heading("メンション"),
                    switch_row(
                        "detailed",
                        "mention",
                        "メンション通知",
                        "自分へのメンションを通知します",
                        true,
                    ),
                    row_separator(),
                    mention_conditions_fieldset(),
                    row_separator(),
                    shortcut_hint_row(),
                ],
            ),
            save_actions(),
        ],
    )
}

/// `card` variant（R0026）: 同じ行構成を `card` の中に収める。
fn card_instance() -> Node {
    div(
        vec![("data-blocks-settings-switch-sections-variant", "card")],
        vec![card::root(
            CardProps::default(),
            vec![("data-blocks-settings-switch-sections-card", "")],
            vec![
                card::header(
                    vec![],
                    vec![
                        section_heading("表示設定"),
                        div(
                            vec![("class", "blocks-settings-switch-sections-description")],
                            vec![text(
                                "このカードは静的レイアウト例です（スイッチは操作できません）。",
                            )],
                        ),
                    ],
                ),
                card::body(
                    vec![("class", "blocks-settings-switch-sections-section")],
                    vec![
                        switch_row(
                            "card",
                            "compact",
                            "コンパクト表示",
                            "余白を詰めて一覧性を高めます",
                            false,
                        ),
                        row_separator(),
                        switch_row(
                            "card",
                            "auto-save",
                            "自動保存",
                            "変更を自動的に保存します",
                            true,
                        ),
                    ],
                ),
                card::footer(
                    vec![("class", "blocks-settings-switch-sections-actions")],
                    vec![button::button(
                        &ButtonProps::default(),
                        vec![],
                        vec![text("保存")],
                    )],
                ),
            ],
        )],
    )
}

/// `settings-switch-sections` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。ルートの `class`（`-layout`）は [`BLOCK::demo_class`]
/// （`blocks-settings-switch-sections`）とは異なる名前にする（他 block と
/// 同じ規約、CSS セレクタの衝突防止）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-settings-switch-sections-layout")],
        vec![basic_instance(), detailed_instance(), card_instance()],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/settings-switch-sections/",
    title: "settings-switch-sections",
    category: BlockCategory::Settings,
    rust_source: "crates/docs-site/src/blocks/application/settings/settings_switch_sections.rs",
    demo_class: "blocks-settings-switch-sections",
    parts: &[
        Part {
            label: "Switch",
            path: "/themes/switch/",
        },
        Part {
            label: "Field",
            path: "/themes/field/",
        },
        Part {
            label: "Fieldset",
            path: "/themes/fieldset/",
        },
        Part {
            label: "Radio Group",
            path: "/themes/radio-group/",
        },
        Part {
            label: "Card",
            path: "/themes/card/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Separator",
            path: "/themes/separator/",
        },
        Part {
            label: "Heading",
            path: "/themes/heading/",
        },
        Part {
            label: "Kbd",
            path: "/themes/kbd/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `settings_switch_sections` 固有のレイアウト規則（`crate::blocks` モジュール
/// doc「CSS の置き場」節）。行の横並び自体は `FieldOrientation::Horizontal`
/// が既に担うため（モジュール冒頭 doc 参照）、ここでは縦積みの間隔・
/// disabled 中和・保存ボタンの右寄せのみを追加する。
const LAYOUT_CSS: &str = "\
.blocks-settings-switch-sections-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n}\n\
[data-blocks-settings-switch-sections-variant] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n}\n\
.blocks-settings-switch-sections-section {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-settings-switch-sections-row-text {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1);\n  min-width: 0;\n}\n\
.blocks-settings-switch-sections-description {\n  margin: 0;\n  color: var(--fandhe-color-fg-muted);\n}\n\
.blocks-settings-switch-sections-shortcut-label {\n  font-weight: var(--fandhe-font-font-weight-medium);\n  color: var(--fandhe-color-fg);\n}\n\
.blocks-settings-switch-sections-actions {\n  display: flex;\n  justify-content: flex-end;\n}\n\
[data-blocks-settings-switch-sections-separator] {\n  margin: 0;\n}\n\
[data-scope=\"switch\"][data-part=\"root\"][data-blocks-settings-switch-sections-switch][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
[data-scope=\"radio-group\"][data-part=\"item\"][data-blocks-settings-switch-sections-radio-item][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    fn demo_html() -> String {
        render(&demo())
    }

    #[test]
    fn demo_composes_expected_parts() {
        let html = demo_html();
        for scope in [
            "data-scope=\"switch\"",
            "data-scope=\"field\"",
            "data-scope=\"fieldset\"",
            "data-scope=\"radio-group\"",
            "data-scope=\"card\"",
            "data-scope=\"button\"",
            "data-scope=\"separator\"",
            "data-scope=\"heading\"",
            "data-scope=\"kbd\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
    }

    #[test]
    fn all_three_variants_are_present() {
        let html = demo_html();
        for variant in ["basic", "detailed", "card"] {
            assert!(
                html.contains(&format!(
                    r#"data-blocks-settings-switch-sections-variant="{variant}""#
                )),
                "missing variant hook for {variant}"
            );
        }
    }

    #[test]
    fn switches_and_radio_items_are_natively_disabled() {
        let html = demo_html();
        assert!(html.matches("role=\"switch\"").count() >= 7);
        // switch の hidden_input・radio item の hidden_input はいずれも
        // ネイティブ disabled（静的表示のため、モジュール doc「switch /
        // radio をネイティブ disabled にする理由」節参照）。
        assert!(html.matches(" disabled=\"\"").count() >= 10);
    }

    #[test]
    fn no_form_submit_or_dead_links() {
        let html = demo_html();
        assert!(!html.contains("<form"));
        assert!(!html.contains("src=\"data:"));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("<script"));
        assert!(!html.contains("type=\"submit\""));
    }

    #[test]
    fn all_buttons_are_type_button() {
        let html = demo_html();
        let button_count = html
            .matches("data-scope=\"button\" data-part=\"root\"")
            .count();
        assert!(button_count >= 2);
        assert_eq!(html.matches(r#"type="button""#).count(), button_count);
    }

    #[test]
    fn aria_describedby_targets_exist_for_every_switch_row() {
        let html = demo_html();
        for (instance, key) in [
            ("basic", "email"),
            ("basic", "push"),
            ("basic", "digest"),
            ("basic", "two-factor"),
            ("basic", "new-device"),
            ("detailed", "mention"),
            ("card", "compact"),
            ("card", "auto-save"),
        ] {
            let id = format!("blocks-settings-switch-sections-{instance}-{key}");
            let helper_id = format!("{id}-helper-text");
            assert!(
                html.contains(&format!(r#"aria-describedby="{helper_id}""#)),
                "missing aria-describedby for {helper_id}"
            );
            assert!(
                html.contains(&format!(r#"id="{helper_id}""#)),
                "missing helper text id {helper_id}"
            );
        }
    }

    #[test]
    fn ids_are_unique() {
        let html = demo_html();
        let mut ids: Vec<&str> = Vec::new();
        for chunk in html.split("id=\"").skip(1) {
            if let Some(end) = chunk.find('"') {
                ids.push(&chunk[..end]);
            }
        }
        let mut sorted = ids.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), ids.len(), "duplicate id found in demo output");
    }

    #[test]
    fn layout_css_is_safe() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(!LAYOUT_CSS.contains("</style"));
    }

    /// variant ラッパー（`[data-blocks-settings-switch-sections-variant]`）が
    /// 縦方向の間隔規則を持つことを固定する（セクション同士・保存ボタンが
    /// 密着する回帰の再発防止、Bugbot 指摘）。
    #[test]
    fn variant_wrapper_has_vertical_gap() {
        assert!(LAYOUT_CSS.contains("[data-blocks-settings-switch-sections-variant] {"));
    }

    #[test]
    fn root_class_differs_from_demo_class() {
        let html = demo_html();
        assert!(html.starts_with(r#"<div class="blocks-settings-switch-sections-layout""#));
        assert_ne!(
            super::BLOCK.demo_class,
            "blocks-settings-switch-sections-layout"
        );
    }

    #[test]
    fn demo_is_deterministic() {
        assert_eq!(demo_html(), demo_html());
    }
}
