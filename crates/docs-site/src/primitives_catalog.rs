//! Primitives（`fandhe-frontend-headless-ui`）部品ページ台帳（イシュー
//! #1020、設計 `docs/design/docs-site-primitives-themes-split.md` §6/§7）。
//!
//! # 役割・呼び出し文脈
//!
//! `/primitives/<kebab>/` 63 ページの「どの部品が・どの URL に・どの表示名
//! で・どのカテゴリに属するか」の唯一の正。#1021（nav 登録・ページ生成）と
//! #1024〜#1029（原稿充填）が本モジュールを一次情報として参照する。
//!
//! # 本モジュールが `std::fs` を持たない理由
//!
//! 台帳とコードの突合（`crates/headless-ui/src/` のソース走査）は
//! `tests/primitives_catalog.rs` にのみ置く。docs-site は headless-ui へ
//! 直接依存しない方針（イシュー #693）であり、出荷される lib にリポジトリの
//! ディレクトリレイアウトへの実行時依存を持ち込まないため（イシュー #1020
//! 本文「テスト専用の走査であればソース走査でよい」）。本モジュールは
//! 純データ + 純関数に限定する。
//!
//! # 判別規約（設計 §6 の要旨）
//!
//! `crates/headless-ui/src/*.rs` のうち本文に `anatomy(` を含むもの
//! （`anatomy.rs` 自身を除く）が部品 63 件、基盤モジュール（[`FOUNDATION_MODULES`]）
//! が 9 件、`lib.rs` を加えて 73 件が `crates/headless-ui/src/*.rs` の総数
//! （実測: `ls crates/headless-ui/src/*.rs | wc -l` => 73、
//! `grep -l 'anatomy(' crates/headless-ui/src/*.rs | grep -v '/anatomy.rs' | wc -l`
//! => 63）。この判別規約とコードの突合は `tests/primitives_catalog.rs` の
//! 責務。

use std::collections::BTreeSet;

/// 台帳 1 件。
///
/// フィールドはすべて `&'static str` 定数のみで、HTML 組み立ては行わない
/// （REQ-1: これらの値を将来 #1021 が描画する際は `fandhe_frontend_core` の
/// ノード木 API 経由で既定エスケープを通す前提）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PrimitiveEntry {
    /// headless-ui の mod 名（snake_case）。`crates/headless-ui/src/<module>.rs`
    /// に対応する。
    pub module: &'static str,
    /// サイト上のページ path（`"/primitives/<kebab>/"`）。kebab は `module`
    /// の `_` を `-` に置き換えたもの（[`kebab_of`] 参照）。
    pub path: &'static str,
    /// サイト上の表示名（`site/nav.toml` の title に使う想定）。
    pub title: &'static str,
    /// 所属カテゴリ（設計 §7 の 6 グループ）。
    pub category: PrimitiveCategory,
}

/// 設計 §7 の 6 グループ。並び順は §7 の表順（サイドバーの並びでもある）。
///
/// `Ord` の派生順（列挙順）が §7 の表順と一致することを
/// `category_counts_and_order_follow_the_design_spec`
/// （`src` 内ユニットテスト・`tests/primitives_catalog.rs` の双方）が固定する。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PrimitiveCategory {
    /// Forms A（11 件、原稿は #1024）。
    FormsA,
    /// Forms B（11 件、原稿は #1025）。
    FormsB,
    /// Forms C・日付・状態表示（10 件、原稿は #1026）。
    FormsCDateStatus,
    /// Overlay / Disclosure（10 件、原稿は #1027）。
    OverlayDisclosure,
    /// Navigation（11 件、原稿は #1028）。
    Navigation,
    /// Data Display / Utilities（10 件、原稿は #1029）。
    DataDisplayUtilities,
}

impl PrimitiveCategory {
    /// `site/nav.toml` の `[[section.group]]` title に使う表示名。
    pub const fn title(self) -> &'static str {
        match self {
            PrimitiveCategory::FormsA => "Forms A",
            PrimitiveCategory::FormsB => "Forms B",
            PrimitiveCategory::FormsCDateStatus => "Forms C・日付・状態表示",
            PrimitiveCategory::OverlayDisclosure => "Overlay / Disclosure",
            PrimitiveCategory::Navigation => "Navigation",
            PrimitiveCategory::DataDisplayUtilities => "Data Display / Utilities",
        }
    }

    /// 設計 §7 の表順で全カテゴリを返す（本 enum の宣言順と一致する）。
    pub const fn all() -> &'static [PrimitiveCategory] {
        &[
            PrimitiveCategory::FormsA,
            PrimitiveCategory::FormsB,
            PrimitiveCategory::FormsCDateStatus,
            PrimitiveCategory::OverlayDisclosure,
            PrimitiveCategory::Navigation,
            PrimitiveCategory::DataDisplayUtilities,
        ]
    }

    /// 原稿充填（Phase 5）の対応 issue 番号（設計 §7 の表から転記）。
    pub const fn spec_issue(self) -> u32 {
        match self {
            PrimitiveCategory::FormsA => 1024,
            PrimitiveCategory::FormsB => 1025,
            PrimitiveCategory::FormsCDateStatus => 1026,
            PrimitiveCategory::OverlayDisclosure => 1027,
            PrimitiveCategory::Navigation => 1028,
            PrimitiveCategory::DataDisplayUtilities => 1029,
        }
    }
}

/// Primitives 台帳（63 件）。
///
/// 並びは設計 §7 のグループ順・グループ内記載順を**逐語で保存する**
/// （#1021 が「1020 の台帳順」で nav へ登録するため、アルファベット順へ
/// 正規化してはならない）。
pub const PRIMITIVES: &[PrimitiveEntry] = &[
    // --- Forms A（11、#1024） ---
    PrimitiveEntry {
        module: "angle_slider",
        path: "/primitives/angle-slider/",
        title: "Angle Slider",
        category: PrimitiveCategory::FormsA,
    },
    PrimitiveEntry {
        module: "checkbox",
        path: "/primitives/checkbox/",
        title: "Checkbox",
        category: PrimitiveCategory::FormsA,
    },
    PrimitiveEntry {
        module: "checkbox_group",
        path: "/primitives/checkbox-group/",
        title: "Checkbox Group",
        category: PrimitiveCategory::FormsA,
    },
    PrimitiveEntry {
        module: "color_picker",
        path: "/primitives/color-picker/",
        title: "Color Picker",
        category: PrimitiveCategory::FormsA,
    },
    PrimitiveEntry {
        module: "combobox",
        path: "/primitives/combobox/",
        title: "Combobox",
        category: PrimitiveCategory::FormsA,
    },
    PrimitiveEntry {
        module: "editable",
        path: "/primitives/editable/",
        title: "Editable",
        category: PrimitiveCategory::FormsA,
    },
    PrimitiveEntry {
        // Themes ページと title 一致（#1685、`site/themes/field.md` /
        // `site/nav.toml` を参照。
        // `primitives_titles_match_themes_page_titles_where_both_exist`
        // が突合する）。表示名は機械転記元がないため "Field" を直接定める。
        module: "field",
        path: "/primitives/field/",
        title: "Field",
        category: PrimitiveCategory::FormsA,
    },
    PrimitiveEntry {
        // Themes ページと title 一致（#1687、`site/themes/fieldset.md` /
        // `site/nav.toml` を参照。
        // `primitives_titles_match_themes_page_titles_where_both_exist`
        // が突合する）。表示名は機械転記元がないため "Fieldset" を直接定める。
        module: "fieldset",
        path: "/primitives/fieldset/",
        title: "Fieldset",
        category: PrimitiveCategory::FormsA,
    },
    PrimitiveEntry {
        module: "file_upload",
        path: "/primitives/file-upload/",
        title: "File Upload",
        category: PrimitiveCategory::FormsA,
    },
    PrimitiveEntry {
        module: "image_cropper",
        path: "/primitives/image-cropper/",
        title: "Image Cropper",
        category: PrimitiveCategory::FormsA,
    },
    PrimitiveEntry {
        module: "listbox",
        path: "/primitives/listbox/",
        title: "Listbox",
        category: PrimitiveCategory::FormsA,
    },
    // --- Forms B（11、#1025） ---
    PrimitiveEntry {
        module: "number_input",
        path: "/primitives/number-input/",
        title: "Number Input",
        category: PrimitiveCategory::FormsB,
    },
    PrimitiveEntry {
        module: "password_input",
        path: "/primitives/password-input/",
        title: "Password Input",
        category: PrimitiveCategory::FormsB,
    },
    PrimitiveEntry {
        module: "pin_input",
        path: "/primitives/pin-input/",
        title: "Pin Input",
        category: PrimitiveCategory::FormsB,
    },
    PrimitiveEntry {
        module: "radio_group",
        path: "/primitives/radio-group/",
        title: "Radio Group",
        category: PrimitiveCategory::FormsB,
    },
    PrimitiveEntry {
        module: "rating_group",
        path: "/primitives/rating-group/",
        title: "Rating Group",
        category: PrimitiveCategory::FormsB,
    },
    PrimitiveEntry {
        module: "segment_group",
        path: "/primitives/segment-group/",
        title: "Segment Group",
        category: PrimitiveCategory::FormsB,
    },
    PrimitiveEntry {
        module: "select",
        path: "/primitives/select/",
        title: "Select",
        category: PrimitiveCategory::FormsB,
    },
    PrimitiveEntry {
        module: "signature_pad",
        path: "/primitives/signature-pad/",
        title: "Signature Pad",
        category: PrimitiveCategory::FormsB,
    },
    PrimitiveEntry {
        module: "slider",
        path: "/primitives/slider/",
        title: "Slider",
        category: PrimitiveCategory::FormsB,
    },
    PrimitiveEntry {
        module: "switch",
        path: "/primitives/switch/",
        title: "Switch",
        category: PrimitiveCategory::FormsB,
    },
    PrimitiveEntry {
        module: "tags_input",
        path: "/primitives/tags-input/",
        title: "Tags Input",
        category: PrimitiveCategory::FormsB,
    },
    // --- Forms C・日付・状態表示（10、#1026） ---
    PrimitiveEntry {
        module: "calendar",
        path: "/primitives/calendar/",
        title: "Calendar",
        category: PrimitiveCategory::FormsCDateStatus,
    },
    PrimitiveEntry {
        module: "date_input",
        path: "/primitives/date-input/",
        title: "Date Input",
        category: PrimitiveCategory::FormsCDateStatus,
    },
    PrimitiveEntry {
        module: "date_picker",
        path: "/primitives/date-picker/",
        title: "Date Picker",
        category: PrimitiveCategory::FormsCDateStatus,
    },
    PrimitiveEntry {
        module: "download_trigger",
        path: "/primitives/download-trigger/",
        title: "Download Trigger",
        category: PrimitiveCategory::FormsCDateStatus,
    },
    PrimitiveEntry {
        module: "toggle",
        path: "/primitives/toggle/",
        title: "Toggle",
        category: PrimitiveCategory::FormsCDateStatus,
    },
    PrimitiveEntry {
        module: "toggle_group",
        path: "/primitives/toggle-group/",
        title: "Toggle Group",
        category: PrimitiveCategory::FormsCDateStatus,
    },
    PrimitiveEntry {
        module: "clipboard",
        path: "/primitives/clipboard/",
        title: "Clipboard",
        category: PrimitiveCategory::FormsCDateStatus,
    },
    PrimitiveEntry {
        module: "timer",
        path: "/primitives/timer/",
        title: "Timer",
        category: PrimitiveCategory::FormsCDateStatus,
    },
    PrimitiveEntry {
        module: "progress",
        path: "/primitives/progress/",
        title: "Progress",
        category: PrimitiveCategory::FormsCDateStatus,
    },
    PrimitiveEntry {
        module: "qr_code",
        path: "/primitives/qr-code/",
        title: "QR Code",
        category: PrimitiveCategory::FormsCDateStatus,
    },
    // --- Overlay / Disclosure（10、#1027） ---
    PrimitiveEntry {
        module: "accordion",
        path: "/primitives/accordion/",
        title: "Accordion",
        category: PrimitiveCategory::OverlayDisclosure,
    },
    PrimitiveEntry {
        // Themes ページと title 一致（#1683、primitives_titles_match_themes_page_titles_where_both_exist
        // が突合）。表示名を直接定める。
        module: "collapsible",
        path: "/primitives/collapsible/",
        title: "Collapsible",
        category: PrimitiveCategory::OverlayDisclosure,
    },
    PrimitiveEntry {
        module: "dialog",
        path: "/primitives/dialog/",
        title: "Dialog",
        category: PrimitiveCategory::OverlayDisclosure,
    },
    PrimitiveEntry {
        module: "drawer",
        path: "/primitives/drawer/",
        title: "Drawer",
        category: PrimitiveCategory::OverlayDisclosure,
    },
    PrimitiveEntry {
        module: "floating_panel",
        path: "/primitives/floating-panel/",
        title: "Floating Panel",
        category: PrimitiveCategory::OverlayDisclosure,
    },
    PrimitiveEntry {
        module: "hover_card",
        path: "/primitives/hover-card/",
        title: "Hover Card",
        category: PrimitiveCategory::OverlayDisclosure,
    },
    PrimitiveEntry {
        module: "popover",
        path: "/primitives/popover/",
        title: "Popover",
        category: PrimitiveCategory::OverlayDisclosure,
    },
    PrimitiveEntry {
        module: "toast",
        path: "/primitives/toast/",
        title: "Toast",
        category: PrimitiveCategory::OverlayDisclosure,
    },
    PrimitiveEntry {
        module: "toggle_tip",
        path: "/primitives/toggle-tip/",
        title: "Toggle Tip",
        category: PrimitiveCategory::OverlayDisclosure,
    },
    PrimitiveEntry {
        module: "tooltip",
        path: "/primitives/tooltip/",
        title: "Tooltip",
        category: PrimitiveCategory::OverlayDisclosure,
    },
    // --- Navigation（11、#1028） ---
    PrimitiveEntry {
        module: "action_bar",
        path: "/primitives/action-bar/",
        title: "Action Bar",
        category: PrimitiveCategory::Navigation,
    },
    PrimitiveEntry {
        module: "breadcrumb",
        path: "/primitives/breadcrumb/",
        title: "Breadcrumb",
        category: PrimitiveCategory::Navigation,
    },
    PrimitiveEntry {
        module: "link",
        path: "/primitives/link/",
        title: "Link",
        category: PrimitiveCategory::Navigation,
    },
    PrimitiveEntry {
        module: "link_overlay",
        path: "/primitives/link-overlay/",
        title: "Link Overlay",
        category: PrimitiveCategory::Navigation,
    },
    PrimitiveEntry {
        module: "menu",
        path: "/primitives/menu/",
        title: "Menu",
        category: PrimitiveCategory::Navigation,
    },
    PrimitiveEntry {
        module: "menubar",
        path: "/primitives/menubar/",
        title: "Menubar",
        category: PrimitiveCategory::Navigation,
    },
    PrimitiveEntry {
        module: "nav_list",
        path: "/primitives/nav-list/",
        title: "Nav List",
        category: PrimitiveCategory::Navigation,
    },
    PrimitiveEntry {
        module: "navigation_menu",
        path: "/primitives/navigation-menu/",
        title: "Navigation Menu",
        category: PrimitiveCategory::Navigation,
    },
    PrimitiveEntry {
        module: "pagination",
        path: "/primitives/pagination/",
        title: "Pagination",
        category: PrimitiveCategory::Navigation,
    },
    PrimitiveEntry {
        module: "tabs",
        path: "/primitives/tabs/",
        title: "Tabs",
        category: PrimitiveCategory::Navigation,
    },
    PrimitiveEntry {
        module: "toolbar",
        path: "/primitives/toolbar/",
        title: "Toolbar",
        category: PrimitiveCategory::Navigation,
    },
    // --- Data Display / Utilities（10、#1029） ---
    PrimitiveEntry {
        module: "avatar",
        path: "/primitives/avatar/",
        title: "Avatar",
        category: PrimitiveCategory::DataDisplayUtilities,
    },
    PrimitiveEntry {
        module: "carousel",
        path: "/primitives/carousel/",
        title: "Carousel",
        category: PrimitiveCategory::DataDisplayUtilities,
    },
    PrimitiveEntry {
        module: "json_tree_view",
        path: "/primitives/json-tree-view/",
        title: "JSON Tree View",
        category: PrimitiveCategory::DataDisplayUtilities,
    },
    PrimitiveEntry {
        module: "scroll_area",
        path: "/primitives/scroll-area/",
        title: "Scroll Area",
        category: PrimitiveCategory::DataDisplayUtilities,
    },
    PrimitiveEntry {
        module: "skip_nav",
        path: "/primitives/skip-nav/",
        title: "Skip Nav",
        category: PrimitiveCategory::DataDisplayUtilities,
    },
    PrimitiveEntry {
        module: "splitter",
        path: "/primitives/splitter/",
        title: "Splitter",
        category: PrimitiveCategory::DataDisplayUtilities,
    },
    PrimitiveEntry {
        module: "steps",
        path: "/primitives/steps/",
        title: "Steps",
        category: PrimitiveCategory::DataDisplayUtilities,
    },
    PrimitiveEntry {
        module: "tour",
        path: "/primitives/tour/",
        title: "Tour",
        category: PrimitiveCategory::DataDisplayUtilities,
    },
    PrimitiveEntry {
        module: "tree_view",
        path: "/primitives/tree-view/",
        title: "Tree View",
        category: PrimitiveCategory::DataDisplayUtilities,
    },
    PrimitiveEntry {
        module: "visually_hidden",
        path: "/primitives/visually-hidden/",
        title: "Visually Hidden",
        category: PrimitiveCategory::DataDisplayUtilities,
    },
];

/// 基盤モジュール（部品ではない）9 件。設計 §6 の判別規約により
/// `anatomy(` を呼ばない（`anatomy` 自身のみ定義元として例外）
/// `crates/headless-ui/src/*.rs` として定まる。
///
/// `collapsible` / `field` / `fieldset` は `anatomy()` を持つ実部品であり
/// ここには含めない（CLAUDE.md の記述から基盤と誤読しないこと。設計 §6）。
pub const FOUNDATION_MODULES: &[&str] = &[
    // 部品が呼ぶ anatomy() の定義元。
    "anatomy",
    // ARIA 属性ヘルパ。
    "aria",
    // 色空間変換。
    "color",
    // data-* 属性出力ヘルパ。
    "data_attrs",
    // 日付演算。
    "date",
    // 数値・日時整形。
    "format",
    // 配置計算（floating 系の位置決め）。
    "positioning",
    "qr_encode",
    // 開閉等の状態型。
    "state",
];

/// クレート入口ファイル名。部品でも基盤でもないため、headless-ui ソース
/// との突合時は明示的に除外する。
pub const CRATE_ROOT_MODULE: &str = "lib";

/// Themes 側（`site/themes/<kebab>.md`）に対応ページを持たない
/// Primitives。現在 0 件（`collapsible` はイシュー #1683、`field` は
/// イシュー #1685、`fieldset` はイシュー #1687 でそれぞれ Themes ページ
/// 登録済みのため除外済み）。将来 Themes ページを持たない Primitives が
/// 増えた場合のみ使う。
/// `primitives_titles_match_themes_page_titles_where_both_exist` 相当の
/// 突合ロジックが例外として除外する用途に限定する（partition 検証からは
/// 除外しない。設計 §9 A05「特定モジュールを検査から外す汎用の除外リストを
/// 作らない」の限定用途の 1 つ）。
pub const PRIMITIVES_WITHOUT_THEMES_PAGE: &[&str] = &[];

/// 台帳の全件を宣言順に返す。
pub fn entries() -> impl Iterator<Item = &'static PrimitiveEntry> {
    PRIMITIVES.iter()
}

/// 指定カテゴリに属する台帳エントリを宣言順に返す。
pub fn entries_in(category: PrimitiveCategory) -> impl Iterator<Item = &'static PrimitiveEntry> {
    PRIMITIVES.iter().filter(move |e| e.category == category)
}

/// mod 名から台帳エントリを検索する。
pub fn find(module: &str) -> Option<&'static PrimitiveEntry> {
    PRIMITIVES.iter().find(|e| e.module == module)
}

/// 台帳全件のページ path を宣言順に返す。
pub fn page_paths() -> impl Iterator<Item = &'static str> {
    PRIMITIVES.iter().map(|e| e.path)
}

/// mod 名（snake_case）を kebab-case へ変換する。ページ path 生成規約の
/// 唯一の実装（`_` → `-` の単純置換。headless-ui の mod 名は ASCII
/// snake_case に閉じるため、この変換で url セグメントとして安全な文字列が
/// 得られる）。
pub fn kebab_of(module: &str) -> String {
    module.replace('_', "-")
}

/// 台帳・基盤リストと「実際に観測された mod 名の集合」の突合結果。
/// 空（[`CatalogAudit::is_clean`] == true）でなければ台帳がドリフトして
/// いる。
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct CatalogAudit {
    /// 観測されたが台帳にも基盤リストにも無い（新規モジュールの登録漏れ）。
    pub unregistered: Vec<String>,
    /// 台帳に載っているが観測されない（モジュールの消滅・改名）。
    pub missing_from_source: Vec<String>,
    /// 基盤リストに載っているが観測されない。
    pub missing_foundation: Vec<String>,
    /// 台帳と基盤リストの両方に載っている（「ちょうど一方」違反）。
    pub duplicated: Vec<String>,
}

impl CatalogAudit {
    /// 4 分類すべてが空であれば `true`（ドリフトなし）。
    pub fn is_clean(&self) -> bool {
        self.unregistered.is_empty()
            && self.missing_from_source.is_empty()
            && self.missing_foundation.is_empty()
            && self.duplicated.is_empty()
    }
}

/// `catalog` と `foundation` の両方に含まれる要素を宣言順（`catalog` 側の
/// 順）で返す純関数。[`audit`] が「台帳と基盤の重複」を検出する際の内部
/// 実装であり、実定数（[`PRIMITIVES`] / [`FOUNDATION_MODULES`]）に対して
/// 交差が常に空であることは統合テスト側で表明する。合成引数を渡す
/// ユニットテストでこの交差計算自体を検証できるよう `pub(crate)` で切り出す
/// （実台帳は定数のため、観測集合だけでは「重複」ケースを合成できない）。
pub(crate) fn duplicated_between(catalog: &[&str], foundation: &[&str]) -> Vec<String> {
    catalog
        .iter()
        .filter(|m| foundation.contains(m))
        .map(|m| m.to_string())
        .collect()
}

/// 観測 mod 名集合（`lib` を除く）を台帳・基盤リストと突合する純関数。
///
/// 呼び出し元は `tests/primitives_catalog.rs`（`crates/headless-ui/src/` の
/// 実ソース走査結果）と、本モジュールの `#[cfg(test)]` ユニットテスト
/// （合成入力）。
pub fn audit(observed: &BTreeSet<String>) -> CatalogAudit {
    let catalog_modules: Vec<&str> = PRIMITIVES.iter().map(|e| e.module).collect();

    let unregistered: Vec<String> = observed
        .iter()
        .filter(|m| {
            m.as_str() != CRATE_ROOT_MODULE
                && !catalog_modules.contains(&m.as_str())
                && !FOUNDATION_MODULES.contains(&m.as_str())
        })
        .cloned()
        .collect();

    let missing_from_source: Vec<String> = catalog_modules
        .iter()
        .filter(|m| !observed.contains(**m))
        .map(|m| m.to_string())
        .collect();

    let missing_foundation: Vec<String> = FOUNDATION_MODULES
        .iter()
        .filter(|m| !observed.contains(**m))
        .map(|m| m.to_string())
        .collect();

    let duplicated = duplicated_between(&catalog_modules, FOUNDATION_MODULES);

    CatalogAudit {
        unregistered,
        missing_from_source,
        missing_foundation,
        duplicated,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 実台帳・実基盤リストから観測集合を合成するヘルパ（negative test の
    /// 基点）。
    fn observed_from_catalog_and_foundation() -> BTreeSet<String> {
        let mut set: BTreeSet<String> = PRIMITIVES.iter().map(|e| e.module.to_string()).collect();
        set.extend(FOUNDATION_MODULES.iter().map(|m| m.to_string()));
        set
    }

    /// (a) 未登録の新規モジュール追加を検知できること。
    #[test]
    fn audit_reports_unregistered_module() {
        let mut observed = observed_from_catalog_and_foundation();
        observed.insert("__new_widget".to_string());

        let result = audit(&observed);

        assert_eq!(result.unregistered, vec!["__new_widget".to_string()]);
        assert!(result.missing_from_source.is_empty());
        assert!(result.missing_foundation.is_empty());
        assert!(result.duplicated.is_empty());
        assert!(!result.is_clean());
    }

    /// (b) 台帳掲載モジュールの消滅を検知できること。
    #[test]
    fn audit_reports_module_missing_from_source() {
        let mut observed = observed_from_catalog_and_foundation();
        observed.remove("accordion");

        let result = audit(&observed);

        assert_eq!(result.missing_from_source, vec!["accordion".to_string()]);
        assert!(result.unregistered.is_empty());
        assert!(result.missing_foundation.is_empty());
        assert!(result.duplicated.is_empty());
        assert!(!result.is_clean());
    }

    /// 基盤モジュールの消滅を検知できること（missing_foundation 分類）。
    #[test]
    fn audit_reports_foundation_module_missing_from_source() {
        let mut observed = observed_from_catalog_and_foundation();
        observed.remove("anatomy");

        let result = audit(&observed);

        assert_eq!(result.missing_foundation, vec!["anatomy".to_string()]);
        assert!(!result.is_clean());
    }

    /// (c) 台帳と基盤リストの重複は実定数からは合成できないため、内部の
    /// 交差計算 `duplicated_between` を合成引数で直接検証する。
    #[test]
    fn audit_reports_duplicate_between_catalog_and_foundation() {
        let catalog = ["accordion", "checkbox"];
        let foundation = ["anatomy", "accordion"];

        let result = duplicated_between(&catalog, &foundation);

        assert_eq!(result, vec!["accordion".to_string()]);
    }

    /// クリーンな観測集合（台帳 ∪ 基盤とちょうど一致、lib.rs も含む）は
    /// `is_clean() == true` を返すこと（正常系の対照）。
    #[test]
    fn audit_is_clean_for_matching_observation() {
        let mut observed = observed_from_catalog_and_foundation();
        observed.insert(CRATE_ROOT_MODULE.to_string());

        let result = audit(&observed);

        assert!(result.is_clean(), "{result:?}");
    }

    /// 台帳が 63 件・6 カテゴリで、件数配分（11/11/10/10/11/10）と
    /// カテゴリ出現順が設計 §7 の表順であること。
    #[test]
    fn catalog_has_63_entries_in_six_categories_in_spec_order() {
        assert_eq!(PRIMITIVES.len(), 63);

        let expected_order_and_counts: [(PrimitiveCategory, usize); 6] = [
            (PrimitiveCategory::FormsA, 11),
            (PrimitiveCategory::FormsB, 11),
            (PrimitiveCategory::FormsCDateStatus, 10),
            (PrimitiveCategory::OverlayDisclosure, 10),
            (PrimitiveCategory::Navigation, 11),
            (PrimitiveCategory::DataDisplayUtilities, 10),
        ];

        assert_eq!(
            expected_order_and_counts
                .iter()
                .map(|(c, _)| *c)
                .collect::<Vec<_>>(),
            PrimitiveCategory::all().to_vec()
        );

        for (category, expected_count) in expected_order_and_counts {
            let actual_count = entries_in(category).count();
            assert_eq!(
                actual_count, expected_count,
                "category {category:?} expected {expected_count} entries, got {actual_count}"
            );
        }

        // カテゴリ出現順（PRIMITIVES 内の並び）も §7 表順と一致すること。
        let mut seen_order: Vec<PrimitiveCategory> = Vec::new();
        for entry in PRIMITIVES {
            if seen_order.last() != Some(&entry.category) {
                seen_order.push(entry.category);
            }
        }
        assert_eq!(
            seen_order,
            expected_order_and_counts
                .iter()
                .map(|(c, _)| *c)
                .collect::<Vec<_>>()
        );
    }

    /// 全件の `path` が `kebab_of(module)` から機械導出される規約通り
    /// であること。
    #[test]
    fn page_paths_are_derived_from_module_names() {
        for entry in PRIMITIVES {
            let expected = format!("/primitives/{}/", kebab_of(entry.module));
            assert_eq!(
                entry.path, expected,
                "module `{}` has path `{}`, expected `{expected}`",
                entry.module, entry.path
            );
        }
    }

    /// `module` / `path` / `title` のいずれも重複がないこと。
    #[test]
    fn module_path_and_title_are_unique() {
        let modules: BTreeSet<&str> = PRIMITIVES.iter().map(|e| e.module).collect();
        let paths: BTreeSet<&str> = PRIMITIVES.iter().map(|e| e.path).collect();
        let titles: BTreeSet<&str> = PRIMITIVES.iter().map(|e| e.title).collect();

        assert_eq!(modules.len(), PRIMITIVES.len(), "module に重複がある");
        assert_eq!(paths.len(), PRIMITIVES.len(), "path に重複がある");
        assert_eq!(titles.len(), PRIMITIVES.len(), "title に重複がある");
    }

    /// 基盤リストが 9 件・重複なしであること。
    #[test]
    fn foundation_list_has_nine_unique_modules() {
        let set: BTreeSet<&str> = FOUNDATION_MODULES.iter().copied().collect();
        assert_eq!(FOUNDATION_MODULES.len(), 9);
        assert_eq!(set.len(), 9, "FOUNDATION_MODULES に重複がある");
    }

    /// `find` / `entries` / `page_paths` の基本契約。
    #[test]
    fn find_and_iterators_are_consistent() {
        assert!(find("accordion").is_some());
        assert!(find("__does_not_exist").is_none());
        assert_eq!(entries().count(), PRIMITIVES.len());
        assert_eq!(page_paths().count(), PRIMITIVES.len());
    }

    /// `kebab_of` は `_` を `-` に置換するのみであること。
    #[test]
    fn kebab_of_replaces_underscores_only() {
        assert_eq!(kebab_of("json_tree_view"), "json-tree-view");
        assert_eq!(kebab_of("accordion"), "accordion");
    }
}
