//! Primitives（`fandhe-frontend-headless-ui`）部品ページの Demo 供給機構
//! （イシュー #1022、親トラッキング #1035 Phase 4、設計は本イシュー本文の
//! 実装計画を正とする）。
//!
//! # 役割・呼び出し文脈
//!
//! `crate::component_page::generated_content` は層（[`crate::component_page::Layer`]）
//! ごとに Demo の供給元レジストリを切り替える（イシュー #1021 の層分岐を
//! 踏襲）。[`crate::showcase`] が `/themes/`（`fandhe-frontend-pre-styled-ui`）
//! を担うのに対し、本モジュールは `/primitives/`（`fandhe-frontend-headless-ui`）
//! を担う。優先順位は「本モジュール → 見つからなければ
//! `ComponentPageSpec::demo`（原稿供給、Phase 5〔#1024〜#1029〕）」であり、
//! `component_page::generated_content` がこの順で照会する。
//!
//! # なぜ `crate::showcase` の私有関数を再利用しないか
//!
//! `showcase.rs` は既に 6,598 行（イシュー #941 時点）であり、63 部品
//! （headless-ui の anatomy パート総数 427）を単一ファイルへ追加すると
//! 保守不能になる。分割単位は [`crate::primitives_catalog::PrimitiveCategory`]
//! の 6 グループに一致させ、Phase 5（#1024〜#1029）のカテゴリ分担と 1:1
//! 対応させる（`forms_a` / `forms_b` / `forms_c_date_status` /
//! `overlay_disclosure` / `navigation` / `data_display_utilities`）。
//!
//! # スタイル分離が必須である理由（Themes との衝突回避）
//!
//! `fandhe-frontend-pre-styled-ui` の recipe CSS は `[data-scope="…"]`
//! 属性セレクタで書かれており、headless-ui が出力する同名の
//! `data-scope`/`data-part` へそのまま当たる。`crate::build::build_site`
//! が Primitives ページへ `crate::showcase::STYLESHEET_REL_PATH`
//! （`assets/pre-styled-ui.css`）を配線してしまうと、「スタイルを持たない
//! 層」という Primitives セクションの存在意義が崩れる。このため
//! 本モジュールは専用 CSS（[`STYLESHEET_REL_PATH`]）を新設し、
//! `build.rs` は層で `<link>` を切り替える（Primitives ページに
//! `pre-styled-ui.css` を配線しない）。
//!
//! [`stylesheet`] が返す CSS は **`[data-scope=` / `[data-part=` を
//! 1 個も含まない**（`tests/site_css_contract.rs` が機械検査で固定する）。
//! デモ枠（[`demo_page`] が付与する `.primitives-demo-frame`）の枠線・
//! 余白のみを中和し、headless-ui のマークアップ自体へは一切スタイルを
//! 到達させない。各デモ節の冒頭には「この枠線・余白は docs サイト側の
//! デモ枠であり、fandhe-frontend-headless-ui 自体はスタイルを持たない」
//! 旨の注記段落（[`DEMO_NOTE`]）を出し、pre-styled-ui との混同を防ぐ。
//!
//! # セキュリティ不変条件（REQ-1）
//!
//! マークアップはすべて [`fandhe_frontend_core`] のノード木 API と
//! headless-ui のパート関数（`fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui`
//! 経由の再エクスポート、イシュー #693 方針により直接依存は追加しない）
//! のみで組み立てる。`raw_html()` および HTML 文字列の直接組み立て
//! （`format!("<div>{}</div>", …)`）は使わない。ダミー文字列は無害な
//! ものに限り、実在ドメイン・実クレデンシャル・PII を含めない。
//!
//! # デモ執筆規約（カテゴリ submodule 共通）
//!
//! 1. 対象部品の `data-scope` を最外殻にする（他 scope を内包しない）。
//!    `component_page::resolve_anatomy_scope` は第一候補（path 末尾）が
//!    当たらないと最も外側の `data-scope` へフォールバックするため、
//!    違反は「別部品の Anatomy/`data-*` 表が生成されるがテストは通る」
//!    というサイレント破壊になる（`tests/primitive_showcase.rs` の
//!    scope 一致テストが fail-closed に検知する）。
//! 2. anatomy パートを可能な限り全網羅する。網羅できないものは
//!    `tests/primitive_showcase.rs::KNOWN_UNCOVERED` に理由付きで登録する
//!    （headless-ui の改変・直接依存の追加では解決しない）。
//! 3. 見出しタグ（`h2`/`h3`）を部品側の出力として使わない
//!    （右カラム目次 `nav.docs-toc` の汚染を避ける。過去事故 #980 参照）。
//! 4. class 属性を足す場合は `primitives-demo-*` プレフィックスのみを
//!    使う（`showcase-*` は `pre-styled-ui.css` 専用のため使わない）。
//! 5. 静的な初期状態のみを描画する（`data-state` を 2 値で描き分けて
//!    `Observed Values` を充実させるのは可）。
//! 6. ダミー文字列は無害なものに限る（`example.com` 等の予約ドメイン、
//!    架空の名前）。`password_input` 等の値フィールドへ `value` を
//!    設定しない。

mod data_display_utilities;
mod forms_a;
mod forms_b;
mod forms_c_date_status;
mod navigation;
mod overlay_disclosure;

use fandhe_frontend_core::{div, h2, p, section, text, Node};
use fandhe_frontend_pre_styled_ui::theme::Theme;
use fandhe_frontend_pre_styled_ui::{StyleSheet, StylesheetError};

/// Primitives 専用 CSS の出力先（`out_dir` 起点の相対パス）。
/// `crate::build::build_site` が [`stylesheet`] の内容をこのパスへ書き出し、
/// `/primitives/<kebab>/` ページのみが `<link>` で参照する
/// （`/themes/` ページは `crate::showcase::STYLESHEET_REL_PATH` を参照し、
/// 双方が同一ページへ同時配線されることはない）。
pub const STYLESHEET_REL_PATH: &str = "assets/primitives-showcase.css";

/// 各デモ節冒頭の注記（モジュール doc 参照）。
const DEMO_NOTE: &str =
    "枠線・余白は docs サイト側のデモ枠であり、fandhe-frontend-headless-ui 自体はスタイルを持ちません。";

/// デモ枠・注記段落の中和 CSS。`[data-scope=`/`[data-part=` を含めない
/// （headless-ui のマークアップへスタイルを到達させないことを構造で
/// 保証する、モジュール doc 参照）。
const LAYOUT_CSS: &str = "\
.primitives-showcase {\n  display: block;\n}\n\
.primitives-demo-note {\n  font-size: 0.875rem;\n  color: var(--fandhe-color-fg-muted);\n  margin: 0 0 0.75rem;\n}\n\
.primitives-demo-frame {\n  border: 1px dashed var(--fandhe-color-border);\n  border-radius: 0.5rem;\n  padding: 1rem;\n  margin: 0 0 1.5rem;\n  background: var(--fandhe-color-bg-subtle);\n  overflow-x: auto;\n}\n";

/// Demo 1 件分の共通ラッパ。`title` は部品名（`h2`）、`body` はデモ本体
/// （headless-ui のパート関数呼び出しのみで組み立てた静的マークアップ）。
///
/// 戻り値の形は `component_page::strip_demo_heading` が前提とする
/// `div > section > [h2, …]`（先頭 `section` の先頭 `h2` のみを 1 個
/// 剥がす契約）に一致させる。
fn demo_page(title: &str, body: Vec<Node>) -> Node {
    let mut children = vec![
        h2(vec![], vec![text(title)]),
        p(
            vec![("class", "primitives-demo-note")],
            vec![text(DEMO_NOTE)],
        ),
    ];
    children.push(div(vec![("class", "primitives-demo-frame")], body));
    div(vec![], vec![section(vec![], children)])
}

/// レジストリ 1 件。`path` は [`crate::primitives_catalog::PRIMITIVES`] の
/// `path` と一致させる（`tests/primitive_showcase.rs` が集合として完全
/// 一致することを固定する）。
struct PrimitivePage {
    path: &'static str,
    render: fn() -> Node,
}

/// Primitives Demo レジストリ。`primitives_catalog::PRIMITIVES` の並び順
/// （設計 §7 のカテゴリ表順）に揃える。
const PRIMITIVE_PAGES: &[PrimitivePage] = &[
    // --- Forms A（11、#1024） ---
    PrimitivePage {
        path: "/primitives/angle-slider/",
        render: forms_a::angle_slider_section,
    },
    PrimitivePage {
        path: "/primitives/checkbox/",
        render: forms_a::checkbox_section,
    },
    PrimitivePage {
        path: "/primitives/checkbox-group/",
        render: forms_a::checkbox_group_section,
    },
    PrimitivePage {
        path: "/primitives/color-picker/",
        render: forms_a::color_picker_section,
    },
    PrimitivePage {
        path: "/primitives/combobox/",
        render: forms_a::combobox_section,
    },
    PrimitivePage {
        path: "/primitives/editable/",
        render: forms_a::editable_section,
    },
    PrimitivePage {
        path: "/primitives/field/",
        render: forms_a::field_section,
    },
    PrimitivePage {
        path: "/primitives/fieldset/",
        render: forms_a::fieldset_section,
    },
    PrimitivePage {
        path: "/primitives/file-upload/",
        render: forms_a::file_upload_section,
    },
    PrimitivePage {
        path: "/primitives/image-cropper/",
        render: forms_a::image_cropper_section,
    },
    PrimitivePage {
        path: "/primitives/listbox/",
        render: forms_a::listbox_section,
    },
    // --- Forms B（11、#1025） ---
    PrimitivePage {
        path: "/primitives/number-input/",
        render: forms_b::number_input_section,
    },
    PrimitivePage {
        path: "/primitives/password-input/",
        render: forms_b::password_input_section,
    },
    PrimitivePage {
        path: "/primitives/pin-input/",
        render: forms_b::pin_input_section,
    },
    PrimitivePage {
        path: "/primitives/radio-group/",
        render: forms_b::radio_group_section,
    },
    PrimitivePage {
        path: "/primitives/rating-group/",
        render: forms_b::rating_group_section,
    },
    PrimitivePage {
        path: "/primitives/segment-group/",
        render: forms_b::segment_group_section,
    },
    PrimitivePage {
        path: "/primitives/select/",
        render: forms_b::select_section,
    },
    PrimitivePage {
        path: "/primitives/signature-pad/",
        render: forms_b::signature_pad_section,
    },
    PrimitivePage {
        path: "/primitives/slider/",
        render: forms_b::slider_section,
    },
    PrimitivePage {
        path: "/primitives/switch/",
        render: forms_b::switch_section,
    },
    PrimitivePage {
        path: "/primitives/tags-input/",
        render: forms_b::tags_input_section,
    },
    // --- Forms C・日付・状態表示（10、#1026） ---
    PrimitivePage {
        path: "/primitives/calendar/",
        render: forms_c_date_status::calendar_section,
    },
    PrimitivePage {
        path: "/primitives/date-input/",
        render: forms_c_date_status::date_input_section,
    },
    PrimitivePage {
        path: "/primitives/date-picker/",
        render: forms_c_date_status::date_picker_section,
    },
    PrimitivePage {
        path: "/primitives/download-trigger/",
        render: forms_c_date_status::download_trigger_section,
    },
    PrimitivePage {
        path: "/primitives/toggle/",
        render: forms_c_date_status::toggle_section,
    },
    PrimitivePage {
        path: "/primitives/toggle-group/",
        render: forms_c_date_status::toggle_group_section,
    },
    PrimitivePage {
        path: "/primitives/clipboard/",
        render: forms_c_date_status::clipboard_section,
    },
    PrimitivePage {
        path: "/primitives/timer/",
        render: forms_c_date_status::timer_section,
    },
    PrimitivePage {
        path: "/primitives/progress/",
        render: forms_c_date_status::progress_section,
    },
    PrimitivePage {
        path: "/primitives/qr-code/",
        render: forms_c_date_status::qr_code_section,
    },
    // --- Overlay / Disclosure（10、#1027） ---
    PrimitivePage {
        path: "/primitives/accordion/",
        render: overlay_disclosure::accordion_section,
    },
    PrimitivePage {
        path: "/primitives/collapsible/",
        render: overlay_disclosure::collapsible_section,
    },
    PrimitivePage {
        path: "/primitives/dialog/",
        render: overlay_disclosure::dialog_section,
    },
    PrimitivePage {
        path: "/primitives/drawer/",
        render: overlay_disclosure::drawer_section,
    },
    PrimitivePage {
        path: "/primitives/floating-panel/",
        render: overlay_disclosure::floating_panel_section,
    },
    PrimitivePage {
        path: "/primitives/hover-card/",
        render: overlay_disclosure::hover_card_section,
    },
    PrimitivePage {
        path: "/primitives/popover/",
        render: overlay_disclosure::popover_section,
    },
    PrimitivePage {
        path: "/primitives/toast/",
        render: overlay_disclosure::toast_section,
    },
    PrimitivePage {
        path: "/primitives/toggle-tip/",
        render: overlay_disclosure::toggle_tip_section,
    },
    PrimitivePage {
        path: "/primitives/tooltip/",
        render: overlay_disclosure::tooltip_section,
    },
    // --- Navigation（11、#1028） ---
    PrimitivePage {
        path: "/primitives/action-bar/",
        render: navigation::action_bar_section,
    },
    PrimitivePage {
        path: "/primitives/breadcrumb/",
        render: navigation::breadcrumb_section,
    },
    PrimitivePage {
        path: "/primitives/link/",
        render: navigation::link_section,
    },
    PrimitivePage {
        path: "/primitives/link-overlay/",
        render: navigation::link_overlay_section,
    },
    PrimitivePage {
        path: "/primitives/menu/",
        render: navigation::menu_section,
    },
    PrimitivePage {
        path: "/primitives/menubar/",
        render: navigation::menubar_section,
    },
    PrimitivePage {
        path: "/primitives/nav-list/",
        render: navigation::nav_list_section,
    },
    PrimitivePage {
        path: "/primitives/navigation-menu/",
        render: navigation::navigation_menu_section,
    },
    PrimitivePage {
        path: "/primitives/pagination/",
        render: navigation::pagination_section,
    },
    PrimitivePage {
        path: "/primitives/tabs/",
        render: navigation::tabs_section,
    },
    PrimitivePage {
        path: "/primitives/toolbar/",
        render: navigation::toolbar_section,
    },
    // --- Data Display / Utilities（10、#1029） ---
    PrimitivePage {
        path: "/primitives/avatar/",
        render: data_display_utilities::avatar_section,
    },
    PrimitivePage {
        path: "/primitives/carousel/",
        render: data_display_utilities::carousel_section,
    },
    PrimitivePage {
        path: "/primitives/json-tree-view/",
        render: data_display_utilities::json_tree_view_section,
    },
    PrimitivePage {
        path: "/primitives/scroll-area/",
        render: data_display_utilities::scroll_area_section,
    },
    PrimitivePage {
        path: "/primitives/skip-nav/",
        render: data_display_utilities::skip_nav_section,
    },
    PrimitivePage {
        path: "/primitives/splitter/",
        render: data_display_utilities::splitter_section,
    },
    PrimitivePage {
        path: "/primitives/steps/",
        render: data_display_utilities::steps_section,
    },
    PrimitivePage {
        path: "/primitives/tour/",
        render: data_display_utilities::tour_section,
    },
    PrimitivePage {
        path: "/primitives/tree-view/",
        render: data_display_utilities::tree_view_section,
    },
    PrimitivePage {
        path: "/primitives/visually-hidden/",
        render: data_display_utilities::visually_hidden_section,
    },
];

/// 登録済み全ページの path を宣言順に返す（`tests/primitive_showcase.rs`
/// が `primitives_catalog::page_paths()` との集合完全一致を固定する）。
pub fn page_paths() -> impl Iterator<Item = &'static str> {
    PRIMITIVE_PAGES.iter().map(|p| p.path)
}

/// `page_path` に対応する Demo 木を返す（`component_page::generated_content`
/// が Themes 側〔`crate::showcase::generated_content`〕に次ぐ優先度で照会する、
/// モジュール doc §役割・呼び出し文脈参照）。未登録パスは `None`。
#[must_use]
pub fn generated_content(page_path: &str) -> Option<Node> {
    PRIMITIVE_PAGES
        .iter()
        .find(|entry| entry.path == page_path)
        .map(|entry| (entry.render)())
}

/// Primitives 専用 CSS を組み立てる（テーマトークン + [`LAYOUT_CSS`]）。
///
/// # Errors
///
/// [`LAYOUT_CSS`] が [`StyleSheet::push_css`] の検証（`<`・制御文字の拒否）
/// に落ちた場合 [`StylesheetError`] を返す。本モジュールの CSS は静的
/// リテラルであり通常は到達しないが、黙って欠けた CSS を公開しない
/// fail-closed 方針で伝播させる（`crate::showcase`/`crate::admonition`
/// と同じ扱い）。
pub fn stylesheet() -> Result<StyleSheet, StylesheetError> {
    let mut sheet = StyleSheet::new();
    sheet.push_theme(&Theme::default());
    sheet.push_css(LAYOUT_CSS)?;
    Ok(sheet)
}
