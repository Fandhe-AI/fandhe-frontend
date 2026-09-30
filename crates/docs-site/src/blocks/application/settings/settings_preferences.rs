//! `settings-preferences` block（イシュー #3009、親 #2951。Application /
//! Settings カテゴリ）。表示テーマ・文字サイズ・配置をラジオカード群で、
//! 言語・タイムゾーン・日付形式・通貨をネイティブセレクトで選ばせる環境設定
//! 画面の 2 構成（版 A・版 B）を縦に並べる。主参照 R0235（代表構成）と
//! R0257（選択欄 5 つの言語・地域設定）を集約する。`_/blocks-intake/` の
//! 対応ファイルは本イシュー着手時点で本 worktree に存在しないため、原稿・
//! 本コメントには対応表 ID のみを記す（`profile-detail-datalist`〔イシュー
//! #2937〕・`settings-billing-overview`〔イシュー #2984〕と同じ扱い）。
//!
//! # 使用部品
//!
//! `radio-card` / `native-select` / `switch` / `field` / `button` /
//! `heading` の 6 部品を合成する（[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//! 新しい UI 部品は追加しない。
//!
//! # 版 A（代表構成、R0235）と版 B（言語・地域設定、R0257）
//!
//! 他の block（`profile-detail-datalist`・`settings-billing-overview` 等）と
//! 同様、複数構成を 1 つの Demo へ縦に並記する。版 A はテーマ・文字サイズ・
//! 配置のラジオカード群 3 つ + 言語・タイムゾーンのセレクト 2 つ +
//! アニメーション低減のスイッチ 1 つ、版 B は言語・地域・タイムゾーン・
//! 日付形式・通貨のセレクト 5 つのみで構成する。いずれも末尾に保存ボタンを
//! 置く。
//!
//! # `<form>` を使わない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない静的表示のみで、送信処理・送信先は一切持たない。保存ボタンは
//! `button::button` の既定 `type="button"` のまま用いる（送信先を持たない）。
//!
//! # 初期状態は固定（無 JS）・全操作をネイティブ disabled にする
//!
//! docs サイトは無 JS 制約のため、ラジオカード・セレクト・スイッチの選択
//! 状態はすべて本ファイル内の固定値で決定的に描画する（クリック等の動的な
//! 状態遷移は持たない）。クリックでネイティブ state のみ変わり render 時
//! 固定の `data-state` と乖離することを防ぐため、全ラジオカード・セレクト・
//! スイッチをネイティブ `disabled` にする（`settings_notification_matrix`
//! の checkbox と同じ判断、codex/cursor レビュー指摘 #3452）。disabled 化で
//! 生じる減光（`crate::recipe::disabled_declarations`）は [`LAYOUT_CSS`] で
//! `opacity: 1` へ中和し、静的デモの見た目自体は変えない。
//!
//! # 狭幅ではラジオカードを 1 列に積む（`@container`）
//!
//! Demo 枠の幅はビューポート幅と一致しないため、`@container`（コンテナ
//! クエリ）で判定する（`profile_detail_datalist`・
//! `settings_billing_overview` 等と同型のパターン）。
//! `.blocks-settings-preferences-stack` へ `container-type: inline-size` を
//! 宣言したうえで、コンテナ幅が `36rem` 未満のときラジオカード群を 1 列へ、
//! スイッチ行を縦積みへ切り替える。
//!
//! # `class` と `data-*` の使い分け
//!
//! [`fandhe_frontend_pre_styled_ui::radio_card::root`] は `drop_class_attr`
//! で呼び出し側 `class` を除去してから内部 variant クラスと合成するため、
//! CSS フック（3 列グリッド化）は `data-*` 属性（`data-blocks-settings-
//! preferences-cards`）で渡す。レイアウト用ラッパー（stack/section/group/
//! switch-row/actions）・説明文は素の `class="blocks-settings-preferences-*"`
//! を使う。
//!
//! # ダミー素材について
//!
//! 言語・地域・タイムゾーン・日付形式・通貨の選択肢はすべて一般的な架空の
//! 語彙であり、実在の企業・人物・メールアドレス・クレデンシャル・PII は
//! 含まない。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::button::{button, ButtonProps};
use fandhe_frontend_pre_styled_ui::field::{self, FieldIds, FieldProps, FieldRootProps};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps};
use fandhe_frontend_pre_styled_ui::native_select::{native_select, NativeSelectProps};
use fandhe_frontend_pre_styled_ui::radio_card::{self, Orientation};
use fandhe_frontend_pre_styled_ui::switch::{self, SwitchProps};
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// `field_id`/ラジオカードグループの一意な `id` を組み立てる（版 A/B で
/// 衝突させない。`blocks_contract::demo_output_has_no_dangling_aria_references_or_duplicate_ids`
/// が同一 Demo 内の id 重複を検査する）。
fn field_id(version: &str, name: &str) -> String {
    format!("blocks-settings-preferences-{version}-{name}")
}

/// 1 件のネイティブセレクト欄（ラベル + セレクト + 補足の `field` 合成）を
/// 組み立てる。
fn select_field(
    id: &str,
    label_text: &str,
    helper: &str,
    options: &[(&str, &str)],
    selected: &str,
) -> Node {
    let props = FieldProps {
        id,
        ids: FieldIds::default(),
        // 無 JS 静的デモの契約（モジュール doc「初期状態は固定（無 JS）」節）:
        // クリックでネイティブ state のみ変わり render 時固定の data-state と
        // 乖離することを防ぐため、ネイティブ disabled で操作自体を封じる
        // （`settings_notification_matrix` の checkbox と同じ判断）。
        disabled: true,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: true,
    };
    let opts: Vec<Node> = options
        .iter()
        .map(|(value, label)| {
            let mut attrs = vec![("value", *value)];
            if *value == selected {
                attrs.push(("selected", ""));
            }
            el("option", attrs, vec![text(*label)])
        })
        .collect();
    field::root(
        &FieldRootProps::default(),
        &props,
        vec![],
        vec![
            field::label(&props, vec![], vec![text(label_text)]),
            native_select(&NativeSelectProps::default(), &props, vec![], opts),
            field::helper_text(&props, vec![], vec![text(helper)]),
        ],
    )
}

/// 1 群のラジオカード（グループ見出し + 選択肢カード列）を組み立てる。
/// `items` は `(value, label, description, checked)` の並び。
fn radio_card_group(
    group_id: &str,
    name: &str,
    title: &str,
    items: &[(&str, &str, &str, bool)],
) -> Node {
    let mut children = vec![radio_card::label(Some(group_id), vec![], vec![text(title)])];
    // 無 JS 静的デモの契約（モジュール doc「初期状態は固定（無 JS）」節）:
    // クリックでネイティブ state のみ変わり render 時固定の data-state と
    // 乖離することを防ぐため、各カードをネイティブ disabled にする
    // （`settings_notification_matrix` の checkbox と同じ判断）。
    children.extend(items.iter().map(|(value, label, description, checked)| {
        radio_card::item(
            *checked,
            true,
            value,
            vec![],
            vec![
                radio_card::item_hidden_input(*checked, true, Some(name), value, vec![]),
                radio_card::item_control(
                    *checked,
                    true,
                    vec![],
                    vec![
                        radio_card::item_content(
                            vec![],
                            vec![
                                radio_card::item_text(vec![], vec![text(*label)]),
                                radio_card::item_description(vec![], vec![text(*description)]),
                            ],
                        ),
                        radio_card::item_indicator(*checked, true, false, vec![]),
                    ],
                ),
            ],
        )
    }));
    radio_card::root(
        Size::Md,
        ColorPalette::Accent,
        false,
        Some(Orientation::Horizontal),
        Some(group_id),
        vec![("data-blocks-settings-preferences-cards", "")],
        children,
    )
}

/// 1 行のスイッチ（見出し・説明文とスイッチ本体を横並びにする）。
fn switch_row(name: &str, label_text: &str, description: &str, checked: bool) -> Node {
    let props = SwitchProps {
        // 無 JS 静的デモの契約（モジュール doc「初期状態は固定（無 JS）」節）:
        // クリックでネイティブ state のみ変わり render 時固定の data-state と
        // 乖離することを防ぐため、ネイティブ disabled で操作自体を封じる
        // （`settings_notification_matrix` の checkbox と同じ判断）。
        disabled: true,
        ..SwitchProps::default()
    };
    div(
        vec![("class", "blocks-settings-preferences-switch-row")],
        vec![
            div(
                vec![("class", "blocks-settings-preferences-group")],
                vec![
                    div(vec![], vec![text(label_text)]),
                    div(
                        vec![("class", "blocks-settings-preferences-description")],
                        vec![text(description)],
                    ),
                ],
            ),
            switch::root(
                Size::Md,
                ColorPalette::Accent,
                checked,
                &props,
                vec![],
                vec![
                    // `switch::root` の兄弟要素にラベルを置いても `<label>`
                    // 内の入力名として認識されないため（codex-review 指摘）、
                    // `hidden_input` へ直接 `aria-label` を付与しアクセシブル
                    // ネームを確定させる（見出しテキストは switch-row の
                    // 兄弟 div に残し、見た目の配置は変えない）。
                    switch::hidden_input(
                        name,
                        "on",
                        checked,
                        &props,
                        vec![("aria-label", label_text)],
                    ),
                    switch::control(
                        checked,
                        &props,
                        vec![],
                        vec![switch::thumb(checked, &props, vec![], vec![])],
                    ),
                ],
            ),
        ],
    )
}

/// 版 A（代表構成、R0235）: テーマ・文字サイズ・配置のラジオカード群 3 つ、
/// 言語・タイムゾーンのセレクト 2 つ、アニメーション低減のスイッチ 1 つ。
fn version_representative() -> Node {
    let theme_group = radio_card_group(
        &field_id("a", "theme-group"),
        "settings-preferences-a-theme",
        "テーマ",
        &[
            ("light", "ライト", "明るい配色で表示します。", false),
            ("dark", "ダーク", "暗い配色で表示します。", false),
            (
                "system",
                "システムに従う",
                "OS の設定に合わせて自動で切り替えます。",
                true,
            ),
        ],
    );
    let font_size_group = radio_card_group(
        &field_id("a", "font-size-group"),
        "settings-preferences-a-font-size",
        "文字サイズ",
        &[
            ("small", "小", "本文の文字を小さく表示します。", false),
            ("standard", "標準", "既定の文字サイズです。", true),
            ("large", "大", "本文の文字を大きく表示します。", false),
        ],
    );
    let density_group = radio_card_group(
        &field_id("a", "density-group"),
        "settings-preferences-a-density",
        "配置",
        &[
            (
                "compact",
                "コンパクト",
                "余白を減らして密に表示します。",
                false,
            ),
            ("standard", "標準", "既定の余白で表示します。", true),
            ("comfortable", "ゆったり", "余白を広めに表示します。", false),
        ],
    );

    let language = select_field(
        &field_id("a", "language"),
        "言語",
        "画面表示に使う言語です。",
        &[("ja", "日本語"), ("en", "English"), ("de", "Deutsch")],
        "ja",
    );
    let timezone = select_field(
        &field_id("a", "timezone"),
        "タイムゾーン",
        "日時の表示に使うタイムゾーンです。",
        &[
            ("asia-tokyo", "Asia/Tokyo"),
            ("utc", "UTC"),
            ("america-los-angeles", "America/Los_Angeles"),
        ],
        "asia-tokyo",
    );

    let reduce_motion = switch_row(
        "settings-preferences-a-reduce-motion",
        "アニメーションを減らす",
        "画面遷移・演出のアニメーションを最小限にします。",
        false,
    );

    div(
        vec![("class", "blocks-settings-preferences-section")],
        vec![
            heading(
                HeadingLevel::H2,
                &HeadingProps::default(),
                vec![],
                vec![text("表示と言語")],
            ),
            div(
                vec![("class", "blocks-settings-preferences-group")],
                vec![theme_group, font_size_group, density_group],
            ),
            div(
                vec![("class", "blocks-settings-preferences-group")],
                vec![language, timezone, reduce_motion],
            ),
            div(
                vec![("class", "blocks-settings-preferences-actions")],
                vec![button(
                    &ButtonProps::default(),
                    vec![],
                    vec![text("設定を保存")],
                )],
            ),
        ],
    )
}

/// 版 B（言語・地域設定、R0257）: 言語・地域・タイムゾーン・日付形式・通貨の
/// セレクト 5 つ。
fn version_locale_five_selects() -> Node {
    let language = select_field(
        &field_id("b", "language"),
        "言語",
        "画面表示に使う言語です。",
        &[("ja", "日本語"), ("en", "English"), ("de", "Deutsch")],
        "ja",
    );
    let region = select_field(
        &field_id("b", "region"),
        "地域",
        "通貨・単位系の既定値に使う地域です。",
        &[("jp", "日本"), ("us", "アメリカ合衆国"), ("de", "ドイツ")],
        "jp",
    );
    let timezone = select_field(
        &field_id("b", "timezone"),
        "タイムゾーン",
        "日時の表示に使うタイムゾーンです。",
        &[
            ("asia-tokyo", "Asia/Tokyo"),
            ("utc", "UTC"),
            ("america-los-angeles", "America/Los_Angeles"),
        ],
        "asia-tokyo",
    );
    let date_format = select_field(
        &field_id("b", "date-format"),
        "日付形式",
        "一覧・詳細画面で日付を表示する形式です。",
        &[
            ("iso", "2026-09-30"),
            ("slash", "2026/09/30"),
            ("dmy", "30 Sep 2026"),
        ],
        "iso",
    );
    let currency = select_field(
        &field_id("b", "currency"),
        "通貨",
        "金額の表示に使う通貨単位です。",
        &[("jpy", "JPY"), ("usd", "USD"), ("eur", "EUR")],
        "jpy",
    );

    div(
        vec![("class", "blocks-settings-preferences-section")],
        vec![
            heading(
                HeadingLevel::H2,
                &HeadingProps::default(),
                vec![],
                vec![text("言語と地域")],
            ),
            div(
                vec![("class", "blocks-settings-preferences-group")],
                vec![language, region, timezone, date_format, currency],
            ),
            div(
                vec![("class", "blocks-settings-preferences-actions")],
                vec![button(
                    &ButtonProps::default(),
                    vec![],
                    vec![text("設定を保存")],
                )],
            ),
        ],
    )
}

/// `settings-preferences` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-settings-preferences-stack")],
        vec![version_representative(), version_locale_five_selects()],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/settings-preferences/",
    title: "settings-preferences",
    category: BlockCategory::Settings,
    rust_source: "crates/docs-site/src/blocks/application/settings/settings_preferences.rs",
    demo_class: "blocks-settings-preferences",
    parts: &[
        Part {
            label: "Radio Card",
            path: "/themes/radio-card/",
        },
        Part {
            label: "Native Select",
            path: "/themes/native-select/",
        },
        Part {
            label: "Switch",
            path: "/themes/switch/",
        },
        Part {
            label: "Field",
            path: "/themes/field/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Heading",
            path: "/themes/heading/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `settings_preferences` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節と同型）。
const LAYOUT_CSS: &str = "\
.blocks-settings-preferences-stack {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-10);\n  container-type: inline-size;\n  container-name: blocks-settings-preferences;\n}\n\
.blocks-settings-preferences-section {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n}\n\
.blocks-settings-preferences-group {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-settings-preferences-description {\n  margin: 0;\n  color: var(--fandhe-color-fg-muted);\n  font-size: var(--fandhe-font-font-size-sm);\n}\n\
[data-scope=\"radio-card\"][data-part=\"root\"][data-blocks-settings-preferences-cards] {\n  display: grid;\n  grid-template-columns: repeat(3, minmax(0, 1fr));\n  gap: var(--fandhe-space-3);\n}\n\
[data-scope=\"radio-card\"][data-part=\"root\"][data-blocks-settings-preferences-cards] [data-scope=\"radio-card\"][data-part=\"label\"] {\n  grid-column: 1 / -1;\n}\n\
.blocks-settings-preferences-switch-row {\n  display: flex;\n  align-items: flex-start;\n  justify-content: space-between;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-settings-preferences-actions {\n  display: flex;\n  justify-content: flex-end;\n}\n\
.blocks-settings-preferences-stack [data-scope=\"radio-card\"][data-part=\"item\"][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
.blocks-settings-preferences-stack [data-scope=\"native-select\"][data-part=\"select\"][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
.blocks-settings-preferences-stack [data-scope=\"switch\"][data-part=\"root\"][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
.blocks-settings-preferences-stack [data-scope=\"field\"][data-part=\"label\"][data-disabled],\n\
.blocks-settings-preferences-stack [data-scope=\"field\"][data-part=\"helper-text\"][data-disabled] {\n  opacity: 1;\n}\n\
@container blocks-settings-preferences (max-width: 36rem) {\n  \
[data-scope=\"radio-card\"][data-part=\"root\"][data-blocks-settings-preferences-cards] {\n    grid-template-columns: 1fr;\n  }\n  \
.blocks-settings-preferences-switch-row {\n    flex-direction: column;\n  }\n\
}\n";

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
            "data-scope=\"radio-card\"",
            "data-scope=\"field\"",
            "data-scope=\"switch\"",
            "data-scope=\"button\"",
            "data-scope=\"heading\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
    }

    #[test]
    fn demo_has_expected_control_counts() {
        let html = demo_html();
        assert_eq!(
            html.matches("<select").count(),
            7,
            "expected 7 <select>: {html}"
        );
        assert_eq!(
            html.matches("type=\"radio\"").count(),
            9,
            "expected 9 radio inputs: {html}"
        );
        assert_eq!(
            html.matches("role=\"switch\"").count(),
            1,
            "expected 1 switch: {html}"
        );
        assert_eq!(html.matches("<h2").count(), 2, "expected 2 <h2>: {html}");
        assert_eq!(
            html.matches("<button").count(),
            2,
            "expected 2 <button>: {html}"
        );
        assert_eq!(
            html.matches("type=\"button\"").count(),
            2,
            "buttons must all be type=\"button\": {html}"
        );
    }

    #[test]
    fn no_form_submit_or_dead_links() {
        let html = demo_html();
        assert!(!html.contains("<form"));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("src=\"data:"));
        assert!(!html.contains("<script"));
        assert!(!html.contains("type=\"submit\""));
    }

    #[test]
    fn initial_state_is_fixed() {
        let html = demo_html();
        assert_eq!(
            html.matches("checked=\"\"").count(),
            3,
            "expected 3 checked radios in version A: {html}"
        );
        assert_eq!(
            html.matches("selected=\"\"").count(),
            7,
            "expected 7 selected options: {html}"
        );
    }

    #[test]
    fn all_interactive_controls_are_natively_disabled() {
        // 無 JS 静的デモの契約（モジュール doc「初期状態は固定（無 JS）・
        // 全操作をネイティブ disabled にする」節）: 9 ラジオ + 7 セレクト +
        // 1 スイッチ = 17 件全てが `disabled=""` を持つ（codex/cursor
        // レビュー指摘 #3452）。
        let html = demo_html();
        assert_eq!(
            html.matches(" disabled=\"\"").count(),
            17,
            "expected 17 disabled controls (9 radio + 7 select + 1 switch): {html}"
        );
    }

    #[test]
    fn switch_has_accessible_name() {
        // switch::root の兄弟要素にラベルがあるだけでは `<label>` 内の入力名
        // として認識されないため（codex-review 指摘）、hidden_input へ直接
        // aria-label を付与している。
        let html = demo_html();
        assert!(
            html.contains(r#"aria-label="アニメーションを減らす""#),
            "switch hidden_input should carry an accessible name: {html}"
        );
    }

    #[test]
    fn radio_groups_are_labelled() {
        let html = demo_html();
        for id in [
            "blocks-settings-preferences-a-theme-group",
            "blocks-settings-preferences-a-font-size-group",
            "blocks-settings-preferences-a-density-group",
        ] {
            assert!(
                html.contains(&format!("aria-labelledby=\"{id}\"")),
                "missing aria-labelledby for {id}: {html}"
            );
            assert!(
                html.contains(&format!("id=\"{id}\"")),
                "missing label id for {id}: {html}"
            );
        }
    }

    #[test]
    fn layout_css_is_safe_and_stacks_on_narrow_container() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("container-type: inline-size;"));
        assert!(LAYOUT_CSS.contains("@container blocks-settings-preferences (max-width: 36rem)"));
    }

    #[test]
    fn radio_card_group_heading_spans_full_grid_width() {
        // codex-review 指摘: 見出しが 3 列グリッドの 1 列目にしか配置されず
        // 選択肢カードと同段で崩れる問題の是正。
        assert!(LAYOUT_CSS.contains(
            "[data-scope=\"radio-card\"][data-part=\"root\"][data-blocks-settings-preferences-cards] [data-scope=\"radio-card\"][data-part=\"label\"] {\n  grid-column: 1 / -1;\n}"
        ));
    }
}
