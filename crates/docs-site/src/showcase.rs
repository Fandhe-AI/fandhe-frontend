//! pre-styled-ui コンポーネントショーケースページの Rust 生成コンテンツ。
//!
//! # 役割・呼び出し文脈
//!
//! docs サイトのビルドパイプライン（`crate::build::build_site`）は基本的に
//! 「Markdown → Node 木」の変換のみを行うが、UI コンポーネントの実レンダリング
//! 結果を掲載するショーケースページは Markdown では表現できない。本モジュールは
//! そのための「Rust 生成コンテンツページ」の最小機構であり、
//!
//! 1. [`generated_content`]: `site/nav.toml` の `page.path` をキーに、Markdown
//!    本文の**後ろへ追記する** `Node` 木を返す（該当しないページは `None`。
//!    Markdown ページ処理・linkcheck の既存パイプラインへは一切干渉しない）
//! 2. [`stylesheet`]: ショーケースが参照する CSS（テーマトークン + 使用
//!    recipe の全量 + ショーケース専用の配置スタイル）を
//!    [`StyleSheet`] として組み立てる。`build_site` がビルド成果物
//!    [`STYLESHEET_REL_PATH`] へ書き出し、ページ側は
//!    `crate::layout::docs_page_with_assets` の追加 `<link>` で参照する
//!
//! の 2 点だけを `build.rs` へ公開する。サイト骨格スタイル
//! （[`crate::site_theme`] によるビルド時生成、出力先 `assets/site.css`）
//! とは分離ファイルに保ち、既存ページのカスケードへ
//! 影響させない（イシュー #520 系のショーケース統合方針）。
//!
//! # セキュリティ不変条件（REQ-1）
//!
//! - マークアップはすべて `fandhe_frontend_core` / `fandhe_frontend_pre_styled_ui`
//!   のノード木 API で組み立てる。`raw_html()`・HTML 文字列の直接組み立ては
//!   使わない。headless 層の状態値（[`OpenState`] / [`Orientation`]）は
//!   pre-styled-ui のルート再エクスポート（イシュー #685）経由で使用し、
//!   headless-ui への直接依存は持たない（イシュー #693）
//! - CSS は [`StyleSheet`]（検証済み CSS のみを保持し `<` を拒否する型、
//!   `crates/pre-styled-ui/src/stylesheet.rs`）経由でのみ書き出す
//!
//! # インタラクティブ部品の扱い
//!
//! Tabs / Accordion / Dialog / Menu / Select / Popover / Tooltip 等の状態
//! 機械を持つ部品は、SSR 静的マークアップ（選択中・開いた状態を
//! `data-state="open"`/`"active"` 等で固定した掲示）のみを載せる。実際の
//! クリック挙動（dispatch 状態遷移）は wasm 層の責務であり docs サイトの
//! スコープ外（`examples/headless-pre-styled-ui` と同じ方針）。
//!
//! Dialog/Menu/Select/Popover/Tooltip/Tour は開いた（Active）状態を固定して掲示するため、
//! recipe CSS のオーバーレイ配置（`position: fixed`/`absolute` + `z-index`）
//! をそのまま反映するとページ全体を覆う・後続セクションに重なってしまう。
//! [`SHOWCASE_LAYOUT_CSS`] がショーケース内に限定してこれを中和する
//! （recipe CSS・サイト骨格 CSS（[`crate::site_theme`] によるビルド時
//! 生成、出力先 `assets/site.css`）はいずれも変更しない）。
//!
//! # ページ単位分解（イシュー #941・#943）
//!
//! [`generated_content`] は「1 ページ = pre-styled-ui の公開部品 1 件」
//! （`docs/design/docs-site-component-pages.md` §3）を実現するため、
//! `path -> 部品セクション` のレジストリ（`ComponentPage` / `COMPONENT_PAGES`）
//! を持つ。`/components/pre-styled-ui/`（[`PAGE_PATH`]）は #943 で索引
//! （凡例 + カテゴリ別リンク集）へ改組済みであり、Rust 生成コンテンツを
//! 持たない（URL 自体は既存被リンク〔`docs/api/pre-styled-ui-api.md` 等〕
//! 維持のため変更していない）。集約レンダリング（旧 `showcase_body`）は
//! 全部品を横断する回帰テスト専用のテストヘルパーとしてのみ
//! `#[cfg(test)]` 配下に残す。`site/nav.toml` への部品ページ登録・原稿
//! スタブ作成は #943、Demo/Features/Anatomy/API Reference の雛形合成は
//! #942 の責務であり、本モジュールは器（レジストリと照会 API）のみを
//! 提供する。

use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::action_bar;
use fandhe_frontend_pre_styled_ui::area_chart::{self, AreaChartProps};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarShape, ImageStatus};
use fandhe_frontend_pre_styled_ui::blockquote::{self, BlockquoteVariant};
use fandhe_frontend_pre_styled_ui::breadcrumb::{self, BreadcrumbItem, BreadcrumbVariant};
use fandhe_frontend_pre_styled_ui::button::{
    button, close_button, icon_button, ButtonProps, ButtonVariant,
};
use fandhe_frontend_pre_styled_ui::calendar::{self, PlainDate};
use fandhe_frontend_pre_styled_ui::carousel;
use fandhe_frontend_pre_styled_ui::charts::axis::{self, AxisProps};
use fandhe_frontend_pre_styled_ui::charts::bar_chart::{
    self, BarChartProps, Orientation as BarChartOrientation,
};
use fandhe_frontend_pre_styled_ui::charts::bar_list;
use fandhe_frontend_pre_styled_ui::charts::bar_segment;
use fandhe_frontend_pre_styled_ui::charts::data::{ChartData, Series};
use fandhe_frontend_pre_styled_ui::charts::grid::{self, GridProps};
use fandhe_frontend_pre_styled_ui::charts::legend::{self, LegendProps};
use fandhe_frontend_pre_styled_ui::charts::radar_chart::{self, RadarChartProps};
use fandhe_frontend_pre_styled_ui::charts::scale::LinearScale;
use fandhe_frontend_pre_styled_ui::charts::scatter_chart::{
    self, ScatterChartProps, ScatterData, ScatterSeries,
};
use fandhe_frontend_pre_styled_ui::charts::svg::{svg_root, ViewBox};
use fandhe_frontend_pre_styled_ui::charts::tooltip as chart_tooltip;
use fandhe_frontend_pre_styled_ui::checkbox::{self, CheckboxProps, CheckedState};
use fandhe_frontend_pre_styled_ui::checkbox_card;
use fandhe_frontend_pre_styled_ui::code::code;
use fandhe_frontend_pre_styled_ui::color_picker;
use fandhe_frontend_pre_styled_ui::color_swatch::{
    self, Color, ColorSwatchProps, Rgb, SwatchShape,
};
use fandhe_frontend_pre_styled_ui::data_list::{self, DataListOrientation, DataListProps};
use fandhe_frontend_pre_styled_ui::date_input::{self, DateSegment};
use fandhe_frontend_pre_styled_ui::date_picker;
use fandhe_frontend_pre_styled_ui::dialog::{self, ContentIds, DialogRole};
use fandhe_frontend_pre_styled_ui::donut_chart::{donut_chart, DonutChartProps};
use fandhe_frontend_pre_styled_ui::download_trigger::{self, DownloadTriggerProps};
use fandhe_frontend_pre_styled_ui::drawer::{self, DrawerPlacement};
use fandhe_frontend_pre_styled_ui::editable::{
    self, EditMode, EditableInputFlags, EditableInputProps,
};
use fandhe_frontend_pre_styled_ui::em::em;
use fandhe_frontend_pre_styled_ui::empty_state::{self, EmptyStateProps};
use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::carousel::Carousel;
use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::color_picker::ColorPicker;
use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::positioning::{
    Align, Placement, Side,
};
use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::slider::Slider;
use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::splitter::{PanelSpec, Splitter};
use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::steps::Steps;
use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::tour::{Tour, TourStep};
use fandhe_frontend_pre_styled_ui::file_upload;
use fandhe_frontend_pre_styled_ui::floating_panel::{self, Stage};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::highlight::{highlight, HighlightProps};
use fandhe_frontend_pre_styled_ui::hover_card::{self, HoverCardDelays};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::image::{image, AspectRatio, ImageFit, ImageProps};
use fandhe_frontend_pre_styled_ui::input::{self, FieldIds, FieldProps, InputProps};
use fandhe_frontend_pre_styled_ui::json_tree_view::{self, JsonValue};
use fandhe_frontend_pre_styled_ui::kbd::kbd;
use fandhe_frontend_pre_styled_ui::line_chart::{self, LineChartProps};
use fandhe_frontend_pre_styled_ui::list::{self, ListType, ListVariant};
use fandhe_frontend_pre_styled_ui::listbox;
use fandhe_frontend_pre_styled_ui::mark::{mark, MarkProps, MarkVariant};
use fandhe_frontend_pre_styled_ui::marquee::{self, MarqueeDirection, MarqueeProps};
use fandhe_frontend_pre_styled_ui::menubar::{self, Menubar};
use fandhe_frontend_pre_styled_ui::native_select::{self, NativeSelectProps};
use fandhe_frontend_pre_styled_ui::number_input::{self, NumberInputFlags};
use fandhe_frontend_pre_styled_ui::pagination::{self, ItemMode, Pagination};
use fandhe_frontend_pre_styled_ui::password_input::{
    self, PasswordAutocomplete, PasswordInputProps,
};
use fandhe_frontend_pre_styled_ui::pie_chart::{pie_chart, PieChartProps};
use fandhe_frontend_pre_styled_ui::qr_code;
use fandhe_frontend_pre_styled_ui::radio_card;
use fandhe_frontend_pre_styled_ui::rating_group::{self, RatingGroup, RatingItemFlags};
use fandhe_frontend_pre_styled_ui::scroll_area;
use fandhe_frontend_pre_styled_ui::segment_group;
use fandhe_frontend_pre_styled_ui::separator::{separator, SeparatorProps, SeparatorVariant};
use fandhe_frontend_pre_styled_ui::skeleton::{skeleton, SkeletonProps, SkeletonVariant};
use fandhe_frontend_pre_styled_ui::slider;
use fandhe_frontend_pre_styled_ui::sparkline::{self, SparklineProps};
use fandhe_frontend_pre_styled_ui::spinner::{spinner, SpinnerProps};
use fandhe_frontend_pre_styled_ui::splitter;
use fandhe_frontend_pre_styled_ui::stat;
use fandhe_frontend_pre_styled_ui::status::{self, StatusProps};
use fandhe_frontend_pre_styled_ui::steps;
use fandhe_frontend_pre_styled_ui::table::{self, TableProps, TableVariant};
use fandhe_frontend_pre_styled_ui::tabs::{tabs, ActivationMode, TabItem, TabsProps};
use fandhe_frontend_pre_styled_ui::tag::{self, TagProps, TagVariant};
use fandhe_frontend_pre_styled_ui::tags_input;
use fandhe_frontend_pre_styled_ui::text::{text as styled_text, TextProps, TextSize};
use fandhe_frontend_pre_styled_ui::textarea::{self, TextareaProps};
use fandhe_frontend_pre_styled_ui::theme::Theme;
use fandhe_frontend_pre_styled_ui::timeline::{self, TimelineVariant};
use fandhe_frontend_pre_styled_ui::timer::{self, Timer, TimerControl, TimerUnit};
use fandhe_frontend_pre_styled_ui::toast::{self, ToastPlacement, ToastStatus};
use fandhe_frontend_pre_styled_ui::toggle_group;
use fandhe_frontend_pre_styled_ui::toolbar::{self, Toolbar};
use fandhe_frontend_pre_styled_ui::tour::{self, ContentIds as TourContentIds};
use fandhe_frontend_pre_styled_ui::tree_view::{self, TreeNode, TreeView};
use fandhe_frontend_pre_styled_ui::visually_hidden;
use fandhe_frontend_pre_styled_ui::{
    accordion, alert, badge, card, combobox, menu, popover, radio_group, select, switch, toggle,
    toggle_tip, tooltip, AlertStatus, BadgeProps, BadgeVariant, CardVariant, ColorPalette,
    OpenState, Orientation, Size, StyleSheet, StylesheetError,
};

/// 索引ページ（凡例 + カテゴリ別リンク集）の `page.path`。`site/nav.toml`
/// の `[[section.page]]` 宣言と一致させる契約（乖離を防ぐ用途で
/// `tests/site_nav.rs` / `tests/site_showcase.rs` が参照する）。イシュー
/// #943 で索引ページへ改組済みのため、本モジュールはこのページ向けに
/// Rust 生成コンテンツを持たない（[`generated_content`] は常に `None` を
/// 返す。索引の本文はすべて `site/components-pre-styled-ui.md` 側で持つ）。
pub const PAGE_PATH: &str = "/components/pre-styled-ui/";

/// ショーケース専用 CSS の出力先（`out_dir` 起点の相対パス）。
/// `crate::build::build_site` が [`stylesheet`] の内容をこのパスへ書き出し、
/// ページ `<head>` の追加 `<link>`（`docs_page_with_assets`）が参照する。
pub const STYLESHEET_REL_PATH: &str = "assets/pre-styled-ui.css";

/// ショーケース内の配置（グリッド・縦積み）専用スタイル。コンポーネント
/// 自体の見た目は pre-styled-ui の recipe が担い、ここではデモの並びのみを
/// 整える（サイト骨格 CSS（[`crate::site_theme`] によるビルド時生成、
/// 出力先 `assets/site.css`）のクラスとは名前空間を分ける）。
///
/// 末尾の見出しリセットは、showcase ページが `.docs-content` 内へ埋め込まれる
/// ことによる `site.css` 見出しルール（`.docs-content h3` の margin・
/// フォント指定）の Accordion anatomy `h3`（item trigger のラッパ）への漏れを
/// 遮断する（Bugbot 指摘）。`site.css` 側は変更せず（`site_css_contract` を
/// 壊さない）、`data-scope` 属性ベースの決定的セレクタで showcase 領域内に
/// 限定して上書きする（`.pre-styled-showcase` + 属性 + 型 = (0,2,1) が
/// `.docs-content h3` = (0,1,1) より優先される）。
///
/// Dialog/Menu/Select/Popover/Tooltip の掲示（イシュー #691）に伴い、以下の
/// オーバーレイ配置中和ルールを追加している（いずれも recipe CSS（
/// `crates/pre-styled-ui/src/{dialog,menu,select,popover,tooltip}.rs`）・
/// `site.css` は変更せず、showcase 領域内に限定した上書きのみで完結させる）:
///
/// - `[data-scope="dialog"][data-part="backdrop"]`/`[data-scope="drawer"][data-part="backdrop"]`
///   の非表示化: dialog/drawer の backdrop は `position: fixed; inset: 0` の
///   ビューポート全体暗幕であり、開いた状態を固定掲示するとページ全体を
///   覆ってしまうため掲示用にのみ隠す（実際の modal 表示では backdrop は
///   必須であり、ここでの非表示化はショーケースの掲示都合に限定する）。
/// - dialog/drawer/menu/select/combobox/popover/tooltip/hover-card/toggle-tip/
///   action-bar の `[data-part="positioner"]` を `position: static` へ中和:
///   recipe CSS は dialog/drawer を `position: fixed; inset: 0`、menu/select/
///   combobox/popover/hover-card を `position: absolute; top: 100%`、
///   tooltip/toggle-tip を `position: absolute; bottom: 100%`、action-bar を
///   `position: fixed; bottom: ...; left: 50%; transform: translateX(-50%)`
///   としており、いずれも開いた content をページ内の別位置・別セクションに
///   重ねてしまう。static 化してフロー内へインライン表示させることで、後続
///   セクションと重ならずに掲示できる（dialog はさらに `padding`/
///   `justify-content` も中和し、中央寄せのための余白・配置指定を解除する。
///   drawer は recipe CSS が `padding`/`justify-content` を宣言しないため
///   `position` のみで足りる。action-bar はさらに `transform` も中和し、
///   水平方向のずらしを解除する）。
/// - dialog/drawer/popover の `title`（`h2`）見出しリセット: Accordion の `h3` と
///   同じ理由（`site.css` の `.docs-content h2` が漏れる）で、showcase 領域
///   内に限定して `border-top`/`padding-top`/`letter-spacing` を打ち消す
///   （margin/font-size/font-weight は recipe が宣言済みで自然に勝つため
///   宣言しない。recipe との二重管理を避ける最小リセット）。
/// - Toast（イシュー #760）の `[data-part="group"]` を `position: static` へ
///   中和: recipe CSS は `position: fixed`（ビューポート角への固定配置）を
///   宣言しており、dialog/drawer の positioner と同じ理由でページ全体を覆う
///   固定表示になってしまうため同型の中和を適用する（backdrop のような
///   非表示化ではなく static 化のみで足りる。通知 1 件ずつを表す `root`
///   slot は掲示位置に影響しないため対象外）。
/// - `[data-scope="blockquote"][data-part="content"]`（素の `<blockquote>`
///   要素）のリセット（イシュー #771 タイポグラフィ節掲示、Bugbot 指摘）:
///   `site.css` の `.docs-content blockquote` が `padding`/`border-left`/
///   `color`（muted）を素の `blockquote` 要素へ直接宣言しており、Blockquote
///   recipe（`crates/pre-styled-ui/src/blockquote.rs`）の `content` slot は
///   この要素そのものである。recipe 側は `content` slot へ `margin: 0` しか
///   宣言せず `padding`/`border-left`/`color` を宣言しないため、`.docs-content
///   blockquote`（詳細度 (0,1,1)）がそのまま適用され、`root`（`<figure>`）
///   自身の padding・左ボーダーと二重になり、かつ引用文字色が意図せず
///   muted 化する。Accordion `h3`/Dialog `h2` と同じ理由（`site.css` 側は
///   変更せず、showcase 領域内に限定した `data-scope`/`data-part` 属性
///   セレクタで打ち消す）で、`.pre-styled-showcase` + 属性 2 個 = (0,3,0) が
///   `.docs-content blockquote` = (0,1,1) より優先されるようにリセットする。
const SHOWCASE_LAYOUT_CSS: &str = "\
.pre-styled-showcase {\n  display: flex;\n  flex-direction: column;\n  gap: 1.5rem;\n}\n\
.showcase-row {\n  display: flex;\n  flex-wrap: wrap;\n  gap: 0.75rem;\n  align-items: center;\n  margin: 1rem 0;\n}\n\
.showcase-stack {\n  display: flex;\n  flex-direction: column;\n  gap: 0.75rem;\n  margin: 1rem 0;\n  max-width: 36rem;\n}\n\
.showcase-form-field-group {\n  display: flex;\n  flex-direction: column;\n  gap: 0.25rem;\n  width: 100%;\n}\n\
.pre-styled-showcase [data-scope=\"accordion\"] h3 {\n  margin: 0;\n  font-size: 1rem;\n  font-weight: 400;\n  line-height: 1.5;\n  letter-spacing: normal;\n}\n\
.pre-styled-showcase [data-scope=\"dialog\"][data-part=\"backdrop\"],\n.pre-styled-showcase [data-scope=\"drawer\"][data-part=\"backdrop\"] {\n  display: none;\n}\n\
.pre-styled-showcase [data-scope=\"dialog\"][data-part=\"positioner\"] {\n  position: static;\n  padding: 0;\n  justify-content: flex-start;\n}\n\
.pre-styled-showcase [data-scope=\"drawer\"][data-part=\"positioner\"] {\n  position: static;\n}\n\
.pre-styled-showcase [data-scope=\"menu\"][data-part=\"positioner\"],\n.pre-styled-showcase [data-scope=\"select\"][data-part=\"positioner\"],\n.pre-styled-showcase [data-scope=\"combobox\"][data-part=\"positioner\"],\n.pre-styled-showcase [data-scope=\"popover\"][data-part=\"positioner\"],\n.pre-styled-showcase [data-scope=\"tooltip\"][data-part=\"positioner\"],\n.pre-styled-showcase [data-scope=\"hover-card\"][data-part=\"positioner\"],\n.pre-styled-showcase [data-scope=\"toggle-tip\"][data-part=\"positioner\"] {\n  position: static;\n}\n\
.pre-styled-showcase [data-scope=\"action-bar\"][data-part=\"positioner\"] {\n  position: static;\n  transform: none;\n}\n\
.pre-styled-showcase [data-scope=\"floating-panel\"][data-part=\"positioner\"] {\n  position: static;\n  transform: none;\n  z-index: auto;\n}\n\
.pre-styled-showcase [data-scope=\"dialog\"] h2,\n.pre-styled-showcase [data-scope=\"drawer\"] h2,\n.pre-styled-showcase [data-scope=\"popover\"] h2,\n.pre-styled-showcase [data-scope=\"floating-panel\"] h2 {\n  border-top: none;\n  padding-top: 0;\n  letter-spacing: normal;\n}\n\
.pre-styled-showcase [data-scope=\"toast\"][data-part=\"group\"] {\n  position: static;\n}\n\
.pre-styled-showcase [data-scope=\"blockquote\"][data-part=\"content\"] {\n  padding: 0;\n  border-left: none;\n  color: inherit;\n}\n\
.pre-styled-showcase [data-scope=\"tour\"][data-part=\"backdrop\"],\n.pre-styled-showcase [data-scope=\"tour\"][data-part=\"spotlight\"] {\n  display: none;\n}\n\
.pre-styled-showcase [data-scope=\"tour\"][data-part=\"positioner\"] {\n  position: static;\n  transform: none;\n  z-index: auto;\n}\n";

/// 部品ページ 1 件分のレジストリエントリ（イシュー #941）。
///
/// [`COMPONENT_PAGES`] の各要素が「pre-styled-ui の公開部品 1 件 = ページ
/// 1 件」（`docs/design/docs-site-component-pages.md` §3）を表す。
/// [`generated_content`] は `path` を鍵に `render` を呼び出して当該部品の
/// デモ節のみを返す。テスト専用の集約ヘルパー（`tests` モジュール内
/// `showcase_body`）も本テーブルを走査して全節を連結するため、両者が
/// 同一テーブルから導出され「集約側にだけ節がある／レジストリにだけ
/// 登録がある」というドリフトが構造的に起こらない。
struct ComponentPage {
    /// `/components/<kebab-name>/`。`site/nav.toml` の `page.path`（#943 で
    /// 登録）と一致させる契約で、`nav::validate_page_path` のセグメント
    /// allowlist（英数・`-`・`_`、`/` 始まり `/` 終わり）を満たす。
    path: &'static str,
    /// 当該部品のデモ節（[`section`] が返す `<section>` 1 件）を生成する。
    render: fn() -> Node,
}

/// 部品ページのレジストリ本体（89 件、旧集約ページの並び順を保つ。
/// イシュー #991 で Toolbar を追加）。
/// 原則として `docs/design/docs-site-component-pages.md` の台帳に掲載済み
/// の部品のみを登録し、掲載順はテスト専用集約ヘルパーの表示順にのみ効く
/// （#943 の nav 上の並びはカテゴリ別で、本テーブルの順序に依存しない）。
const COMPONENT_PAGES: &[ComponentPage] = &[
    ComponentPage {
        path: "/components/button/",
        render: button_section,
    },
    ComponentPage {
        path: "/components/download-trigger/",
        render: download_trigger_section,
    },
    ComponentPage {
        path: "/components/badge/",
        render: badge_section,
    },
    ComponentPage {
        path: "/components/spinner/",
        render: spinner_section,
    },
    ComponentPage {
        path: "/components/skeleton/",
        render: skeleton_section,
    },
    ComponentPage {
        path: "/components/heading/",
        render: heading_section,
    },
    ComponentPage {
        path: "/components/text/",
        render: text_section,
    },
    ComponentPage {
        path: "/components/em/",
        render: em_section,
    },
    ComponentPage {
        path: "/components/mark/",
        render: mark_section,
    },
    ComponentPage {
        path: "/components/blockquote/",
        render: blockquote_section,
    },
    ComponentPage {
        path: "/components/list/",
        render: list_section,
    },
    ComponentPage {
        path: "/components/separator/",
        render: separator_section,
    },
    ComponentPage {
        path: "/components/highlight/",
        render: highlight_section,
    },
    ComponentPage {
        path: "/components/alert/",
        render: alert_section,
    },
    ComponentPage {
        path: "/components/card/",
        render: card_section,
    },
    ComponentPage {
        path: "/components/tabs/",
        render: tabs_section,
    },
    ComponentPage {
        path: "/components/accordion/",
        render: accordion_section,
    },
    ComponentPage {
        path: "/components/dialog/",
        render: dialog_section,
    },
    ComponentPage {
        path: "/components/drawer/",
        render: drawer_section,
    },
    ComponentPage {
        path: "/components/menu/",
        render: menu_section,
    },
    ComponentPage {
        path: "/components/select/",
        render: select_section,
    },
    ComponentPage {
        path: "/components/listbox/",
        render: listbox_section,
    },
    ComponentPage {
        path: "/components/combobox/",
        render: combobox_section,
    },
    ComponentPage {
        path: "/components/popover/",
        render: popover_section,
    },
    ComponentPage {
        path: "/components/floating-panel/",
        render: floating_panel_section,
    },
    ComponentPage {
        path: "/components/tooltip/",
        render: tooltip_section,
    },
    ComponentPage {
        path: "/components/hover-card/",
        render: hover_card_section,
    },
    ComponentPage {
        path: "/components/toggle-tip/",
        render: toggle_tip_section,
    },
    ComponentPage {
        path: "/components/switch/",
        render: switch_section,
    },
    ComponentPage {
        path: "/components/radio-group/",
        render: radio_group_section,
    },
    ComponentPage {
        path: "/components/avatar/",
        render: avatar_section,
    },
    ComponentPage {
        path: "/components/checkbox/",
        render: checkbox_section,
    },
    ComponentPage {
        path: "/components/input/",
        render: input_section,
    },
    ComponentPage {
        path: "/components/textarea/",
        render: textarea_section,
    },
    ComponentPage {
        path: "/components/native-select/",
        render: native_select_section,
    },
    ComponentPage {
        path: "/components/number-input/",
        render: number_input_section,
    },
    ComponentPage {
        path: "/components/password-input/",
        render: password_input_section,
    },
    ComponentPage {
        path: "/components/tags-input/",
        render: tags_input_section,
    },
    ComponentPage {
        path: "/components/file-upload/",
        render: file_upload_section,
    },
    ComponentPage {
        path: "/components/rating-group/",
        render: rating_group_section,
    },
    ComponentPage {
        path: "/components/slider/",
        render: slider_section,
    },
    ComponentPage {
        path: "/components/editable/",
        render: editable_section,
    },
    ComponentPage {
        path: "/components/segment-group/",
        render: segment_group_section,
    },
    ComponentPage {
        path: "/components/toggle/",
        render: toggle_section,
    },
    ComponentPage {
        path: "/components/toggle-group/",
        render: toggle_group_section,
    },
    ComponentPage {
        path: "/components/carousel/",
        render: carousel_section,
    },
    ComponentPage {
        path: "/components/tree-view/",
        render: tree_view_section,
    },
    ComponentPage {
        path: "/components/json-tree-view/",
        render: json_tree_view_section,
    },
    ComponentPage {
        path: "/components/pagination/",
        render: pagination_section,
    },
    ComponentPage {
        path: "/components/steps/",
        render: steps_section,
    },
    ComponentPage {
        path: "/components/tour/",
        render: tour_section,
    },
    ComponentPage {
        path: "/components/splitter/",
        render: splitter_section,
    },
    ComponentPage {
        path: "/components/checkbox-card/",
        render: checkbox_card_section,
    },
    ComponentPage {
        path: "/components/radio-card/",
        render: radio_card_section,
    },
    ComponentPage {
        path: "/components/breadcrumb/",
        render: breadcrumb_section,
    },
    ComponentPage {
        path: "/components/action-bar/",
        render: action_bar_section,
    },
    ComponentPage {
        path: "/components/toast/",
        render: toast_section,
    },
    ComponentPage {
        path: "/components/progress/",
        render: progress_section,
    },
    ComponentPage {
        path: "/components/image/",
        render: image_section,
    },
    ComponentPage {
        path: "/components/icon/",
        render: icon_section,
    },
    ComponentPage {
        path: "/components/tag/",
        render: tag_section,
    },
    ComponentPage {
        path: "/components/kbd/",
        render: kbd_section,
    },
    ComponentPage {
        path: "/components/code/",
        render: code_section,
    },
    ComponentPage {
        path: "/components/color-swatch/",
        render: color_swatch_section,
    },
    ComponentPage {
        path: "/components/color-picker/",
        render: color_picker_section,
    },
    ComponentPage {
        path: "/components/status/",
        render: status_section,
    },
    ComponentPage {
        path: "/components/empty-state/",
        render: empty_state_section,
    },
    ComponentPage {
        path: "/components/visually-hidden/",
        render: visually_hidden_section,
    },
    ComponentPage {
        path: "/components/qr-code/",
        render: qr_code_section,
    },
    ComponentPage {
        path: "/components/table/",
        render: table_section,
    },
    ComponentPage {
        path: "/components/data-list/",
        render: data_list_section,
    },
    ComponentPage {
        path: "/components/stat/",
        render: stat_section,
    },
    ComponentPage {
        path: "/components/timeline/",
        render: timeline_section,
    },
    ComponentPage {
        path: "/components/marquee/",
        render: marquee_section,
    },
    ComponentPage {
        path: "/components/scroll-area/",
        render: scroll_area_section,
    },
    ComponentPage {
        path: "/components/calendar/",
        render: calendar_section,
    },
    ComponentPage {
        path: "/components/date-picker/",
        render: date_picker_section,
    },
    ComponentPage {
        path: "/components/date-input/",
        render: date_input_section,
    },
    ComponentPage {
        path: "/components/timer/",
        render: timer_section,
    },
    ComponentPage {
        path: "/components/charts/",
        render: charts_section,
    },
    ComponentPage {
        path: "/components/bar-chart/",
        render: bar_chart_section,
    },
    ComponentPage {
        path: "/components/bar-list/",
        render: bar_list_section,
    },
    ComponentPage {
        path: "/components/bar-segment/",
        render: bar_segment_section,
    },
    ComponentPage {
        path: "/components/line-chart/",
        render: line_chart_section,
    },
    ComponentPage {
        path: "/components/area-chart/",
        render: area_chart_section,
    },
    ComponentPage {
        path: "/components/sparkline/",
        render: sparkline_section,
    },
    ComponentPage {
        path: "/components/pie-chart/",
        render: pie_chart_section,
    },
    ComponentPage {
        path: "/components/donut-chart/",
        render: donut_chart_section,
    },
    ComponentPage {
        path: "/components/scatter-chart/",
        render: scatter_chart_section,
    },
    ComponentPage {
        path: "/components/radar-chart/",
        render: radar_chart_section,
    },
    ComponentPage {
        path: "/components/toolbar/",
        render: toolbar_section,
    },
    ComponentPage {
        path: "/components/menubar/",
        render: menubar_section,
    },
];

/// [`COMPONENT_PAGES`] に登録済みの部品ページパスを登録順に返す。
///
/// #943（nav.toml への一括登録）・#944（CI 契約テストでの充足率計測）が
/// 「レジストリの path がすべて nav へ登録されているか」を機械検証する
/// ための最小公開 API。これ以上の内部構造（`ComponentPage` 自体・
/// `render` 関数ポインタ）は公開しない。
pub fn component_page_paths() -> impl Iterator<Item = &'static str> {
    COMPONENT_PAGES.iter().map(|entry| entry.path)
}

/// 部品 1 件分のデモ節をショーケーススコープ（`.pre-styled-showcase`）で
/// 包む。[`SHOWCASE_LAYOUT_CSS`] のオーバーレイ中和・見出しリセット等は
/// すべて `.pre-styled-showcase` 起点のセレクタであり、このラッパを
/// 外すと Dialog/Menu/Toast/Tour 等の掲示がページ全体を覆う回帰になる
/// （テスト専用集約ヘルパー `showcase_body` も同じラッパを共有する）。
fn showcase_wrapper(sections: Vec<Node>) -> Node {
    div(vec![("class", "pre-styled-showcase")], sections)
}

/// `page_path` が Rust 生成コンテンツを持つページなら、Markdown 本文の後ろへ
/// 追記する `Node` 木を返す。
///
/// [`COMPONENT_PAGES`] レジストリ（部品単位のページ、`/components/<kebab>/`）
/// のみを照会する。[`PAGE_PATH`]（索引ページ）はレジストリに含まれないため
/// 常に `None` を返す（イシュー #943 で索引ページへ改組済み。索引の本文は
/// `site/components-pre-styled-ui.md` 側の Markdown のみで完結する）。
/// `crate::component_page::generated_content`（#942）が本関数の戻り値を
/// Demo 節として利用する。
#[must_use]
pub fn generated_content(page_path: &str) -> Option<Node> {
    COMPONENT_PAGES
        .iter()
        .find(|entry| entry.path == page_path)
        .map(|entry| showcase_wrapper(vec![(entry.render)()]))
}

/// ショーケースが参照する CSS 全量を組み立てる。
///
/// 内訳: テーマトークン（`Theme::default`、ライト/ダーク両対応）→ 掲載
/// コンポーネントの recipe CSS（button/download_trigger/badge/spinner/alert/
/// card/tabs/accordion/dialog/drawer/menu/select/combobox/popover/tooltip/
/// hover_card/toggle_tip/switch/radio_group/avatar/checkbox/checkbox_card/
/// radio_card/input/textarea/native_select/number_input/tags_input/
/// rating_group/slider/segment_group/toggle/toggle_group/pagination/
/// breadcrumb/carousel/action_bar/toast/progress/tag/kbd/code/image/icon/
/// status/empty_state/visually_hidden/qr_code/heading/text/em/mark/
/// blockquote/list/table/data_list/stat/timeline/scroll_area/splitter/tour）
/// → ショーケース配置スタイル、の順で決定的に連結する。
///
/// # 部品ごとの CSS 分離を行わない理由（イシュー #941）
///
/// ページ単位分解（[`COMPONENT_PAGES`]）後も本関数は単一の CSS 束を返し、
/// 部品ごとのファイル分割は行わない。理由:
///
/// 1. [`SHOWCASE_LAYOUT_CSS`] の中和ルールはすべて `.pre-styled-showcase`
///    スコープで閉じており、ページ数が増えても他ページのカスケードへ
///    漏れない
/// 2. `build::build_site` は生成コンテンツを持つページへ一律に
///    [`STYLESHEET_REL_PATH`] の `<link>` を配線する。1 ファイルであれば
///    ブラウザキャッシュが部品ページ間で再利用され、ページ遷移ごとの
///    再取得が起きない
/// 3. 部品別に分割するとテーマトークン（`Theme::to_css`）が各ファイルへ
///    重複出力され、総バイト数と生成ロジックの複雑度がともに増える
///
/// 分割は将来の最適化余地として残すが、現時点では計測上の必要がないため
/// 実施しない。
///
/// # Errors
///
/// いずれかの CSS 断片が [`StyleSheet::push_css`] の検証（`<`・制御文字の
/// 拒否）に落ちた場合 [`StylesheetError`] を返す。pre-styled-ui 側の生成 CSS
/// は構造上 `<` を含み得ないため通常は到達しないが、黙って欠けた CSS を
/// 公開しない fail-closed 方針で伝播させる。
pub fn stylesheet() -> Result<StyleSheet, StylesheetError> {
    let mut sheet = StyleSheet::new();
    sheet.push_theme(&Theme::default());
    sheet.push_css(&fandhe_frontend_pre_styled_ui::button::css())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::download_trigger::css())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::badge::css())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::spinner::css())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::alert::css())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::card::css())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::tabs::stylesheet())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::accordion::stylesheet())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::dialog::stylesheet())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::drawer::stylesheet())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::menu::stylesheet())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::select::stylesheet())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::listbox::stylesheet())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::skeleton::css())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::separator::css())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::highlight::css())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::combobox::stylesheet())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::popover::stylesheet())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::floating_panel::stylesheet())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::tooltip::stylesheet())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::hover_card::stylesheet())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::toggle_tip::stylesheet())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::switch::stylesheet())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::radio_group::stylesheet())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::avatar::stylesheet())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::checkbox::stylesheet())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::checkbox_card::stylesheet())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::radio_card::stylesheet())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::input::css())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::textarea::css())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::native_select::css())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::number_input::stylesheet())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::password_input::stylesheet())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::tags_input::stylesheet())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::file_upload::stylesheet())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::rating_group::stylesheet())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::slider::stylesheet())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::editable::stylesheet())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::segment_group::stylesheet())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::tree_view::stylesheet())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::json_tree_view::stylesheet())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::pagination::stylesheet())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::steps::stylesheet())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::breadcrumb::stylesheet())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::carousel::stylesheet())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::action_bar::stylesheet())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::toast::stylesheet())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::progress::stylesheet())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::tag::css())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::kbd::css())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::code::css())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::color_swatch::css())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::color_picker::css())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::image::css())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::icon::css())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::status::css())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::empty_state::css())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::visually_hidden::css())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::qr_code::stylesheet())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::heading::css())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::text::css())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::em::css())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::mark::css())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::blockquote::css())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::list::css())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::table::css())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::data_list::css())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::stat::css())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::timeline::css())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::marquee::css())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::scroll_area::stylesheet())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::splitter::stylesheet())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::calendar::stylesheet())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::date_picker::stylesheet())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::date_input::stylesheet())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::timer::stylesheet())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::tour::stylesheet())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::charts::bar_chart::css())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::charts::bar_list::css())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::charts::bar_segment::css())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::line_chart::stylesheet())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::area_chart::stylesheet())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::sparkline::stylesheet())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::charts::scatter_chart::css())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::charts::radar_chart::css())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::charts::axis::css())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::charts::grid::css())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::charts::legend::css())?;
    sheet.push_css(&chart_tooltip::css())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::pie_chart::css())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::donut_chart::css())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::angle_slider::stylesheet())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::image_cropper::stylesheet())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::pin_input::stylesheet())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::signature_pad::stylesheet())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::toggle::stylesheet())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::toggle_group::stylesheet())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::toolbar::stylesheet())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::menubar::stylesheet())?;
    sheet.push_css(SHOWCASE_LAYOUT_CSS)?;
    Ok(sheet)
}

/// 見出し（`h2`）+ 説明文 + デモ本体、のショーケース 1 節を組み立てる小
/// ヘルパ。見出しは `crate::layout::with_heading_anchors` が id を注入して
/// ページ内 TOC（on this page）へ自動掲載する。
fn section(heading: &str, description: &str, demos: Vec<Node>) -> Node {
    let mut children = vec![
        el("h2", vec![], vec![text(heading)]),
        el("p", vec![], vec![text(description)]),
    ];
    children.extend(demos);
    el("section", vec![], children)
}

/// 横並びのデモ行。
fn row(children: Vec<Node>) -> Node {
    div(vec![("class", "showcase-row")], children)
}

/// 縦積みのデモ列（Alert / Card 等の幅を取る部品向け）。
fn stack(children: Vec<Node>) -> Node {
    div(vec![("class", "showcase-stack")], children)
}

/// Button 節: variant / size / palette / 状態（disabled・loading）の各軸。
fn button_section() -> Node {
    let variants = [
        (ButtonVariant::Solid, "Solid"),
        (ButtonVariant::Outline, "Outline"),
        (ButtonVariant::Ghost, "Ghost"),
        (ButtonVariant::Subtle, "Subtle"),
    ];
    let variant_row = row(variants
        .iter()
        .map(|(variant, label)| {
            button(
                &ButtonProps {
                    variant: *variant,
                    ..ButtonProps::default()
                },
                vec![],
                vec![text(*label)],
            )
        })
        .collect());

    let sizes = [
        (Size::Sm, "Small"),
        (Size::Md, "Medium"),
        (Size::Lg, "Large"),
    ];
    let size_row = row(sizes
        .iter()
        .map(|(size, label)| {
            button(
                &ButtonProps {
                    size: *size,
                    ..ButtonProps::default()
                },
                vec![],
                vec![text(*label)],
            )
        })
        .collect());

    let palette_row = row(palettes()
        .iter()
        .map(|(palette, label)| {
            button(
                &ButtonProps {
                    palette: *palette,
                    ..ButtonProps::default()
                },
                vec![],
                vec![text(*label)],
            )
        })
        .collect());

    let state_row = row(vec![
        button(
            &ButtonProps {
                disabled: true,
                ..ButtonProps::default()
            },
            vec![],
            vec![text("Disabled")],
        ),
        button(
            &ButtonProps {
                loading: true,
                ..ButtonProps::default()
            },
            vec![],
            vec![text("Loading")],
        ),
    ]);

    // IconButton / CloseButton（イシュー #830）: 独立部品ではなく本 recipe の
    // icon-only 修飾 variant として実装した（`crates/pre-styled-ui/src/button.rs`
    // モジュール doc 参照）。IconButton は `aria-label` を必須引数として明示し、
    // CloseButton は既定 `aria-label="Close"` + 内蔵の × アイコンを持つ。
    let icon_close_row = row(vec![
        icon_button(
            &ButtonProps::default(),
            "Search",
            vec![],
            vec![icon(
                &IconProps {
                    label: None,
                    ..IconProps::default()
                },
                vec![],
                vec![el(
                    "path",
                    vec![(
                        "d",
                        "M10 2a8 8 0 105.29 14.29l4.7 4.7 1.42-1.42-4.7-4.7A8 8 0 0010 2zm0 2a6 6 0 110 12 6 6 0 010-12z",
                    )],
                    vec![],
                )],
            )],
        ),
        close_button(
            &ButtonProps {
                variant: ButtonVariant::Ghost,
                ..ButtonProps::default()
            },
            "",
            vec![],
        ),
    ]);

    section(
        "Button",
        "variant（solid / outline / ghost / subtle）・size・colorPalette・状態（disabled / loading）の各軸を型安全な props で切り替えます。IconButton / CloseButton（イシュー #830）は独立部品ではなく本 recipe の icon-only 修飾 variant です。",
        vec![
            variant_row,
            size_row,
            palette_row,
            state_row,
            icon_close_row,
        ],
    )
}

/// DownloadTrigger 節（イシュー #828）: variant / size / palette の各軸。
/// Button recipe の流用（`crates/pre-styled-ui/src/download_trigger.rs`
/// rustdoc 参照）であるため、デモの構成は [`button_section`] と対称に
/// 揃える。`href` は空文字列を使う（`breadcrumb_section` と同じ理由:
/// `crate::linkcheck::check_links` は空 href を無条件でスキップする契約で
/// あり、生成コンテンツを linkcheck の突合対象へ含めない本モジュールの
/// 既存設計（`showcase_markup_has_no_href_attributes_for_linkcheck_neutrality`
/// 参照）を壊さずに `a[download]` 要素を掲示するための選択。実配信への
/// 導線は呼び出し側アプリケーションの責務であり、本ショーケースは recipe
/// CSS の見た目確認が目的）。
fn download_trigger_section() -> Node {
    let variants = [
        (ButtonVariant::Solid, "Solid"),
        (ButtonVariant::Outline, "Outline"),
        (ButtonVariant::Ghost, "Ghost"),
        (ButtonVariant::Subtle, "Subtle"),
    ];
    let variant_row = row(variants
        .iter()
        .map(|(variant, label)| {
            download_trigger::root(
                &DownloadTriggerProps {
                    variant: *variant,
                    ..DownloadTriggerProps::default()
                },
                "",
                Some("sample-report.pdf"),
                vec![],
                vec![text(*label)],
            )
        })
        .collect());

    let sizes = [
        (Size::Sm, "Small"),
        (Size::Md, "Medium"),
        (Size::Lg, "Large"),
    ];
    let size_row = row(sizes
        .iter()
        .map(|(size, label)| {
            download_trigger::root(
                &DownloadTriggerProps {
                    size: *size,
                    ..DownloadTriggerProps::default()
                },
                "",
                Some("sample-report.pdf"),
                vec![],
                vec![text(*label)],
            )
        })
        .collect());

    let palette_row = row(palettes()
        .iter()
        .map(|(palette, label)| {
            download_trigger::root(
                &DownloadTriggerProps {
                    palette: *palette,
                    ..DownloadTriggerProps::default()
                },
                "",
                Some("sample-report.pdf"),
                vec![],
                vec![text(*label)],
            )
        })
        .collect());

    section(
        "DownloadTrigger",
        "a[download] 属性による宣言的ダウンロードトリガー（JS 不要の静的部品）。variant・size・colorPalette は Button recipe を流用します。",
        vec![variant_row, size_row, palette_row],
    )
}

/// Badge 節: variant × palette。
fn badge_section() -> Node {
    let variants = [
        (BadgeVariant::Solid, "Solid"),
        (BadgeVariant::Subtle, "Subtle"),
        (BadgeVariant::Outline, "Outline"),
    ];
    let variant_row = row(variants
        .iter()
        .map(|(variant, label)| {
            badge::badge(
                &BadgeProps {
                    variant: *variant,
                    ..BadgeProps::default()
                },
                vec![],
                vec![text(*label)],
            )
        })
        .collect());
    let palette_row = row(palettes()
        .iter()
        .map(|(palette, label)| {
            badge::badge(
                &BadgeProps {
                    palette: *palette,
                    ..BadgeProps::default()
                },
                vec![],
                vec![text(*label)],
            )
        })
        .collect());
    section(
        "Badge",
        "ステータス表示向けの小型ラベル。variant と colorPalette を組み合わせます。",
        vec![variant_row, palette_row],
    )
}

/// Spinner 節: size バリエーション。
fn spinner_section() -> Node {
    let sizes = [
        (Size::Sm, "Loading (small)"),
        (Size::Md, "Loading (medium)"),
        (Size::Lg, "Loading (large)"),
    ];
    let size_row = row(sizes
        .iter()
        .map(|(size, label)| {
            spinner(&SpinnerProps {
                size: *size,
                label,
                ..SpinnerProps::default()
            })
        })
        .collect());
    section(
        "Spinner",
        "読み込み中表示。role=\"status\" と aria-label でスクリーンリーダーへ状態を伝えます。",
        vec![size_row],
    )
}

/// Skeleton 節（イシュー #764）: variant（text/circle/rect）バリエーション。
fn skeleton_section() -> Node {
    let variants = [
        (SkeletonVariant::Text, "width: 12rem;"),
        (SkeletonVariant::Circle, ""),
        (SkeletonVariant::Rect, "width: 12rem;"),
    ];
    let variant_row = row(variants
        .iter()
        .map(|(variant, style)| {
            skeleton(
                &SkeletonProps { variant: *variant },
                if style.is_empty() {
                    vec![]
                } else {
                    vec![("style", *style)]
                },
            )
        })
        .collect());
    section(
        "Skeleton",
        "データ読み込み中のコンテンツ形状を模した占位要素。常に aria-hidden=\"true\" を持ち、読み込み中であることをスクリーンリーダーへ伝える責務はコンテナ側（aria-busy）にあります。prefers-reduced-motion: reduce ではパルスアニメーションを停止します。",
        vec![variant_row],
    )
}

// タイポグラフィ節群（イシュー #771 で導入、#941 で複合節 typography_section
// から Heading/Text/Em/Mark/Blockquote/List の 6 部品ページ相当の関数へ分解）。
// 素の HTML 意味論（h1〜h6・p・em・mark・blockquote・ul/ol/li）をそのまま
// styled 化する方針は変わらないが、記事全体へのカスケード適用（chakra-ui の
// Prose 相当）は本クレートへ導入せず、docs サイト骨格スタイル
// （`.docs-content`、`crate::site_theme` によるビルド時生成）が引き続き担う
// （`docs/design/docs-site-component-pages.md` §3・§4 の「1 ページ = 部品
// 1 件」方針）。Heading は `h4`〜`h6`（`.docs-content` 見出し規則が対象と
// する `h1`〜`h3` の範囲外）のみを掲示し、サイト骨格の見出しスタイルとの
// 衝突を避ける（本節自体の `h2` はショーケース節見出し〔[`section`] ヘルパ〕
// であり対象外）。各関数はこの前提のもとで 1〜2 文の部品固有説明のみを持つ。

/// Heading 節: `h4`（size=lg）/ `h5`（size=md）/ `h6`（size=sm）の 3 段。
fn heading_section() -> Node {
    let heading_row = row(vec![
        heading(
            HeadingLevel::H4,
            &HeadingProps {
                size: HeadingSize::Lg,
            },
            vec![],
            vec![text("見出し (h4, size=lg)")],
        ),
        heading(
            HeadingLevel::H5,
            &HeadingProps {
                size: HeadingSize::Md,
            },
            vec![],
            vec![text("見出し (h5, size=md)")],
        ),
        heading(
            HeadingLevel::H6,
            &HeadingProps {
                size: HeadingSize::Sm,
            },
            vec![],
            vec![text("見出し (h6, size=sm)")],
        ),
    ]);

    section(
        "Heading",
        "素の h1〜h6 意味論を size（sm/md/lg）でスタイル化した見出し部品。",
        vec![heading_row],
    )
}

/// Text 節: size（sm/md/lg）3 段の本文テキスト。
fn text_section() -> Node {
    let text_stack = stack(
        [TextSize::Sm, TextSize::Md, TextSize::Lg]
            .iter()
            .map(|size| {
                styled_text(
                    &TextProps { size: *size },
                    vec![],
                    vec![text(format!("本文テキスト（size={size:?}）"))],
                )
            })
            .collect(),
    );

    section(
        "Text",
        "素の p 要素を size（sm/md/lg）でスタイル化した本文テキスト部品。",
        vec![text_stack],
    )
}

/// Em 節: 素の `<em>` の強調表現。
fn em_section() -> Node {
    let em_row = row(vec![el(
        "p",
        vec![],
        vec![
            text("この文の"),
            em(vec![], vec![text("強調部分")]),
            text("は重要です。"),
        ],
    )]);

    section(
        "Em",
        "素の em 要素をそのまま styled 化した強調テキスト部品。",
        vec![em_row],
    )
}

/// Mark 節: variant（subtle/solid/text/plain）4 種のハイライト表現。
fn mark_section() -> Node {
    let mark_row = row(vec![
        mark(&MarkProps::default(), vec![], vec![text("subtle")]),
        mark(
            &MarkProps {
                variant: MarkVariant::Solid,
                ..MarkProps::default()
            },
            vec![],
            vec![text("solid")],
        ),
        mark(
            &MarkProps {
                variant: MarkVariant::Text,
                ..MarkProps::default()
            },
            vec![],
            vec![text("text")],
        ),
        mark(
            &MarkProps {
                variant: MarkVariant::Plain,
                ..MarkProps::default()
            },
            vec![],
            vec![text("plain")],
        ),
    ]);

    section(
        "Mark",
        "テキストの一部を強調する Mark 部品。variant（subtle/solid/text/plain）4 種。",
        vec![mark_row],
    )
}

/// Blockquote 節: `subtle` variant の引用ブロック（content + caption）。
fn blockquote_section() -> Node {
    let blockquote_demo = blockquote::root(
        BlockquoteVariant::Subtle,
        ColorPalette::Accent,
        vec![],
        vec![
            blockquote::content(
                vec![],
                vec![text("プレーンな HTML / JavaScript / CSS を尊重する。")],
            ),
            blockquote::caption(vec![], vec![text("— fandhe-frontend CLAUDE.md")]),
        ],
    );

    section(
        "Blockquote",
        "素の blockquote 要素を content/caption の 2 パーツで styled 化した引用部品。",
        vec![blockquote_demo],
    )
}

/// List 節: 順序なし（marker variant）・順序ありの 2 種。
fn list_section() -> Node {
    let marker_list = list::root(
        ListType::Unordered,
        ListVariant::Marker,
        vec![],
        vec![
            list::item(vec![], vec![text("SSR")]),
            list::item(vec![], vec![text("SPA")]),
            list::item(vec![], vec![text("SSG")]),
        ],
    );
    let ordered_list = list::root(
        ListType::Ordered,
        ListVariant::Marker,
        vec![],
        vec![
            list::item(vec![], vec![text("計画")]),
            list::item(vec![], vec![text("実装")]),
            list::item(vec![], vec![text("検証")]),
        ],
    );

    section(
        "List",
        "素の ul/ol/li 意味論をそのまま styled 化したリスト部品。順序なし（marker variant）・順序ありの 2 種。",
        vec![stack(vec![marker_list, ordered_list])],
    )
}

/// Separator 節（イシュー #772）: `orientation`（horizontal/vertical）・
/// `variant`（solid/dashed）の 2 軸。vertical は自身では高さを決定できない
/// （`--fandhe-separator-height` フォールバック）ため、`style` で高さを
/// 明示して並べる（`crates/pre-styled-ui/src/separator.rs` rustdoc 参照）。
fn separator_section() -> Node {
    let horizontal_row = row(vec![
        separator(
            &SeparatorProps {
                orientation: Orientation::Horizontal,
                variant: SeparatorVariant::Solid,
            },
            vec![("style", "width: 12rem;")],
        ),
        separator(
            &SeparatorProps {
                orientation: Orientation::Horizontal,
                variant: SeparatorVariant::Dashed,
            },
            vec![("style", "width: 12rem;")],
        ),
    ]);
    let vertical_row = row(vec![separator(
        &SeparatorProps {
            orientation: Orientation::Vertical,
            variant: SeparatorVariant::Solid,
        },
        vec![("style", "height: 3rem;")],
    )]);
    section(
        "Separator",
        "区切り線。role=\"separator\" と aria-orientation/data-orientation を常時出力します。orientation（horizontal/vertical）と variant（solid/dashed）の 2 軸を持ちます。",
        vec![horizontal_row, vertical_row],
    )
}

/// Highlight 節（イシュー #775）: 単一一致・複数一致（`match_all`）・
/// `ignore_case` の実演。一致判定は正規表現を使わない決定的な部分文字列
/// 検索（`crates/pre-styled-ui/src/highlight.rs` rustdoc 参照）。
fn highlight_section() -> Node {
    let single_match_row = row(vec![highlight(
        &HighlightProps {
            query: &["brown fox"],
            ..HighlightProps::default()
        },
        vec![],
        "The quick brown fox jumps over the lazy dog",
    )]);
    let match_all_row = row(vec![highlight(
        &HighlightProps {
            query: &["o"],
            match_all: true,
            ..HighlightProps::default()
        },
        vec![],
        "The quick brown fox jumps over the lazy dog",
    )]);
    let ignore_case_row = row(vec![highlight(
        &HighlightProps {
            query: &["LAZY"],
            ignore_case: true,
            ..HighlightProps::default()
        },
        vec![],
        "The quick brown fox jumps over the lazy dog",
    )]);
    section(
        "Highlight",
        "テキスト中の一致語句を <mark> で強調します。正規表現ではなく決定的な部分文字列検索のみで一致判定します。query（複数可）・match_all（全一致 or 最初の 1 件）・ignore_case（ASCII 限定）の 3 プロパティを持ちます。",
        vec![single_match_row, match_all_row, ignore_case_row],
    )
}

/// Alert 節: status（info / success / warning / error）ごとの表示。
fn alert_section() -> Node {
    let statuses = [
        (
            AlertStatus::Info,
            "Info",
            "新しいバージョンが利用可能です。",
        ),
        (AlertStatus::Success, "Success", "ビルドが完了しました。"),
        (
            AlertStatus::Warning,
            "Warning",
            "依存クレート数が上限に近づいています。",
        ),
        (
            AlertStatus::Error,
            "Error",
            "リンク切れを検出したため書き出しを中止しました。",
        ),
    ];
    let demos = stack(
        statuses
            .iter()
            .map(|(status, title, description)| {
                alert::root(
                    *status,
                    vec![],
                    vec![
                        alert::indicator(vec![], vec![text("!")]),
                        alert::content(
                            vec![],
                            vec![
                                alert::title(vec![], vec![text(*title)]),
                                alert::description(vec![], vec![text(*description)]),
                            ],
                        ),
                    ],
                )
            })
            .collect(),
    );
    section(
        "Alert",
        "status（info / success / warning / error）で色が切り替わる通知領域。root / indicator / content / title / description の slot 構成です。",
        vec![demos],
    )
}

/// Card 節: variant（elevated / outline / subtle）ごとの表示。
fn card_section() -> Node {
    let variants = [
        (CardVariant::Elevated, "Elevated"),
        (CardVariant::Outline, "Outline"),
        (CardVariant::Subtle, "Subtle"),
    ];
    let demos = stack(
        variants
            .iter()
            .map(|(variant, label)| {
                card::root(
                    *variant,
                    vec![],
                    vec![
                        card::header(
                            vec![],
                            vec![
                                card::title(vec![], vec![text(*label)]),
                                card::description(
                                    vec![],
                                    vec![text("card variant のデモです。")],
                                ),
                            ],
                        ),
                        card::body(
                            vec![],
                            vec![el(
                                "p",
                                vec![],
                                vec![text(
                                    "header / body / footer / title / description の slot 構成を持つ汎用コンテナです。",
                                )],
                            )],
                        ),
                        card::footer(
                            vec![],
                            vec![button(
                                &ButtonProps {
                                    variant: ButtonVariant::Outline,
                                    size: Size::Sm,
                                    ..ButtonProps::default()
                                },
                                vec![],
                                vec![text("Action")],
                            )],
                        ),
                    ],
                )
            })
            .collect(),
    );
    section(
        "Card",
        "variant（elevated / outline / subtle）を持つ装飾的コンテナ。",
        vec![demos],
    )
}

/// Tabs 節: 1 番目のタブが選択された静的マークアップ。
fn tabs_section() -> Node {
    let node = tabs(
        Size::Md,
        ColorPalette::Accent,
        &TabsProps {
            id: "showcase-tabs",
            selected: "overview",
            orientation: Orientation::Horizontal,
            activation_mode: ActivationMode::Automatic,
            loop_focus: true,
            indicator: false,
        },
        vec![
            TabItem {
                value: "overview",
                trigger: vec![text("Overview")],
                content: vec![el(
                    "p",
                    vec![],
                    vec![text(
                        "選択中のタブは data-state=\"active\" で強調されます。",
                    )],
                )],
                disabled: false,
            },
            TabItem {
                value: "usage",
                trigger: vec![text("Usage")],
                content: vec![el(
                    "p",
                    vec![],
                    vec![text("非選択タブの content は CSS で非表示になります。")],
                )],
                disabled: false,
            },
        ],
    );
    section(
        "Tabs",
        "headless-ui の Tabs（WAI-ARIA Tabs パターン）に pre-styled-ui の data-scope / data-part セレクタ CSS を適用した静的掲示です。",
        vec![node],
    )
}

/// Accordion 節: 1 項目目が開いた静的マークアップ（single モード想定）。
fn accordion_section() -> Node {
    let items: [(&str, &str, &str, OpenState); 2] = [
        (
            "showcase-acc-1",
            "pre-styled-ui とは何ですか？",
            "headless-ui の anatomy（data-scope / data-part）へテーマトークンと recipe CSS を重ねる styled 層です。",
            OpenState::Open,
        ),
        (
            "showcase-acc-2",
            "クリックで開閉できますか？",
            "この掲示は SSR 静的マークアップです。状態遷移（dispatch）は wasm 層の責務のため、docs サイトでは開いた状態を固定表示しています。",
            OpenState::Closed,
        ),
    ];
    let mut children = Vec::new();
    for (value, question, answer, state) in items {
        let trigger_id = format!("{value}-trigger");
        let content_id = format!("{value}-content");
        children.push(accordion::item(
            state,
            false,
            vec![],
            vec![
                el(
                    "h3",
                    vec![],
                    vec![accordion::item_trigger(
                        state,
                        false,
                        Some(trigger_id.as_str()),
                        Some(content_id.as_str()),
                        vec![],
                        vec![text(question)],
                    )],
                ),
                accordion::item_content(
                    state,
                    Some(content_id.as_str()),
                    Some(trigger_id.as_str()),
                    vec![],
                    vec![el("p", vec![], vec![text(answer)])],
                ),
            ],
        ));
    }
    section(
        "Accordion",
        "開閉状態（data-state=\"open\" / \"closed\"）に応じてスタイルが切り替わる開閉パネルの静的掲示です。",
        vec![accordion::root(Size::Md, vec![], children)],
    )
}

/// Dialog 節: 開いた状態の静的マークアップ（イシュー #691）。
///
/// backdrop は掲示用に非表示化し（[`SHOWCASE_LAYOUT_CSS`]）、positioner は
/// フロー内配置へ中和している。実際の modal オーバーレイ配置は recipe CSS
/// （`crates/pre-styled-ui/src/dialog.rs`）がそのまま担う。
fn dialog_section() -> Node {
    let node = div(
        vec![],
        vec![
            dialog::trigger(
                OpenState::Open,
                Some("showcase-dialog-content"),
                vec![],
                vec![text("Open dialog")],
            ),
            dialog::root(
                Size::Md,
                OpenState::Open,
                vec![],
                vec![
                    dialog::backdrop(OpenState::Open, vec![], vec![]),
                    dialog::positioner(
                        OpenState::Open,
                        vec![],
                        vec![dialog::content(
                            OpenState::Open,
                            DialogRole::Dialog,
                            true,
                            ContentIds {
                                id: Some("showcase-dialog-content"),
                                labelledby: Some("showcase-dialog-title"),
                                describedby: Some("showcase-dialog-desc"),
                            },
                            vec![],
                            vec![
                                dialog::title(
                                    Some("showcase-dialog-title"),
                                    vec![],
                                    vec![text("Confirm action")],
                                ),
                                dialog::description(
                                    Some("showcase-dialog-desc"),
                                    vec![],
                                    vec![text("この操作は取り消せません。")],
                                ),
                                dialog::close_trigger(vec![], vec![text("Close")]),
                            ],
                        )],
                    ),
                ],
            ),
        ],
    );
    section(
        "Dialog",
        "headless-ui の Dialog（WAI-ARIA dialog パターン）に pre-styled-ui の data-scope / data-part セレクタ CSS を適用した静的掲示です。backdrop は掲示用に非表示化し、positioner はフロー内配置へ中和しています（実際の overlay 配置は recipe CSS が担います）。",
        vec![node],
    )
}

/// Drawer 節: 開いた状態（`placement="end"`）の静的マークアップ（イシュー #758）。
///
/// Drawer は WAI-ARIA 上 Dialog パターンの変種であり、開閉状態機械は
/// headless 層の [`dialog::Dialog`] をそのまま再利用する（`crates/headless-ui/src/drawer.rs`
/// rustdoc 参照）。backdrop は [`dialog_section`] と同じく掲示用に非表示化し、
/// positioner はフロー内配置へ中和している（[`SHOWCASE_LAYOUT_CSS`]）。
/// 実際の画面端固定パネル配置・placement 別レイアウトは recipe CSS
/// （`crates/pre-styled-ui/src/drawer.rs`）がそのまま担う。
fn drawer_section() -> Node {
    let node = div(
        vec![],
        vec![
            drawer::trigger(
                OpenState::Open,
                Some("showcase-drawer-content"),
                vec![],
                vec![text("Open drawer")],
            ),
            drawer::root(
                Size::Md,
                OpenState::Open,
                DrawerPlacement::End,
                vec![],
                vec![
                    drawer::backdrop(OpenState::Open, vec![], vec![]),
                    drawer::positioner(
                        OpenState::Open,
                        DrawerPlacement::End,
                        vec![],
                        vec![drawer::content(
                            OpenState::Open,
                            DrawerPlacement::End,
                            true,
                            ContentIds {
                                id: Some("showcase-drawer-content"),
                                labelledby: Some("showcase-drawer-title"),
                                describedby: Some("showcase-drawer-desc"),
                            },
                            vec![],
                            vec![
                                drawer::title(
                                    Some("showcase-drawer-title"),
                                    vec![],
                                    vec![text("Navigation")],
                                ),
                                drawer::description(
                                    Some("showcase-drawer-desc"),
                                    vec![],
                                    vec![text("画面端からスライドインする補助パネルです。")],
                                ),
                                drawer::close_trigger(vec![], vec![text("Close")]),
                            ],
                        )],
                    ),
                ],
            ),
        ],
    );
    section(
        "Drawer",
        "headless-ui の Drawer（WAI-ARIA dialog パターンの変種、dialog の状態機械を再利用）に pre-styled-ui の data-scope / data-part セレクタ CSS を適用した静的掲示です。placement=\"end\" を掲示しています。backdrop は掲示用に非表示化し、positioner はフロー内配置へ中和しています。",
        vec![node],
    )
}

/// Menu 節: highlighted / 通常 / separator / disabled の各状態を持つ項目リスト
/// が開いた静的マークアップ（イシュー #691）。
fn menu_section() -> Node {
    let node = menu::root(
        Size::Md,
        OpenState::Open,
        vec![],
        vec![
            menu::trigger(
                OpenState::Open,
                false,
                Some("showcase-menu-content"),
                vec![],
                vec![text("Actions")],
            ),
            menu::positioner(
                OpenState::Open,
                vec![],
                vec![menu::content(
                    OpenState::Open,
                    Some("showcase-menu-content"),
                    None,
                    vec![],
                    vec![
                        menu::item("edit", false, true, vec![], vec![text("Edit")]),
                        menu::item("duplicate", false, false, vec![], vec![text("Duplicate")]),
                        menu::separator(vec![], vec![]),
                        menu::item("delete", true, false, vec![], vec![text("Delete")]),
                    ],
                )],
            ),
        ],
    );
    section(
        "Menu",
        "headless-ui の Menu（role=\"menu\"）に pre-styled-ui の recipe CSS を適用した静的掲示です。highlighted（キーボードフォーカス位置）・separator・disabled の各状態を含みます。positioner はフロー内配置へ中和しています。",
        vec![node],
    )
}

/// Select 節: 1 項目が選択済みの listbox が開いた静的マークアップ
/// （イシュー #691）。
fn select_section() -> Node {
    let node = select::root(
        Size::Md,
        OpenState::Open,
        vec![],
        vec![
            select::label(
                Some("showcase-select-label"),
                vec![],
                vec![text("Framework")],
            ),
            select::control(
                OpenState::Open,
                vec![],
                vec![select::trigger(
                    OpenState::Open,
                    false,
                    Some("showcase-select-content"),
                    Some("showcase-select-label"),
                    vec![],
                    vec![
                        select::value_text(false, vec![], vec![text("fandhe-frontend")]),
                        select::indicator(OpenState::Open, vec![], vec![text("▾")]),
                    ],
                )],
            ),
            select::positioner(
                OpenState::Open,
                vec![],
                vec![select::content(
                    OpenState::Open,
                    Some("showcase-select-content"),
                    Some("showcase-select-label"),
                    None,
                    vec![],
                    vec![
                        select::item(
                            OpenState::Open,
                            false,
                            false,
                            "fandhe-frontend",
                            Some("showcase-select-item-fandhe"),
                            vec![],
                            vec![
                                select::item_text(None, vec![], vec![text("fandhe-frontend")]),
                                select::item_indicator(OpenState::Open, vec![], vec![text("✓")]),
                            ],
                        ),
                        select::item(
                            OpenState::Closed,
                            false,
                            false,
                            "other",
                            None,
                            vec![],
                            vec![
                                select::item_text(None, vec![], vec![text("Other framework")]),
                                select::item_indicator(OpenState::Closed, vec![], vec![text("✓")]),
                            ],
                        ),
                    ],
                )],
            ),
        ],
    );
    section(
        "Select",
        "headless-ui の Select（role=\"listbox\"）に pre-styled-ui の recipe CSS を適用した静的掲示です。1 項目が選択済み（data-state=\"open\"）の listbox が開いた状態を固定表示しています。positioner はフロー内配置へ中和しています。",
        vec![node],
    )
}

/// Listbox 節: 常時展開のリスト選択（single/multiple 両モード）の静的
/// マークアップ（イシュー #750）。[`select_section`] とは異なり trigger/
/// positioner を持たず、`content` が常に表示される（責務境界の詳細は
/// `fandhe_frontend_headless_ui::listbox` module doc 参照）。
fn listbox_section() -> Node {
    let single = listbox::root(
        Size::Md,
        OpenState::Open,
        false,
        vec![],
        vec![
            listbox::label(
                Some("showcase-listbox-single-label"),
                vec![],
                vec![text("Fruit")],
            ),
            listbox::content(
                false,
                Some("showcase-listbox-single-content"),
                Some("showcase-listbox-single-label"),
                None,
                vec![],
                vec![
                    listbox::item(
                        OpenState::Open,
                        false,
                        false,
                        "apple",
                        None,
                        vec![],
                        vec![
                            listbox::item_text(None, vec![], vec![text("Apple")]),
                            listbox::item_indicator(OpenState::Open, vec![], vec![text("✓")]),
                        ],
                    ),
                    listbox::item(
                        OpenState::Closed,
                        true,
                        false,
                        "banana",
                        None,
                        vec![],
                        vec![
                            listbox::item_text(None, vec![], vec![text("Banana (disabled)")]),
                            listbox::item_indicator(OpenState::Closed, vec![], vec![text("✓")]),
                        ],
                    ),
                ],
            ),
        ],
    );

    let multiple = listbox::root(
        Size::Md,
        OpenState::Open,
        false,
        vec![],
        vec![
            listbox::label(
                Some("showcase-listbox-multi-label"),
                vec![],
                vec![text("Toppings")],
            ),
            listbox::content(
                true,
                Some("showcase-listbox-multi-content"),
                Some("showcase-listbox-multi-label"),
                None,
                vec![],
                vec![listbox::item_group(
                    Some("showcase-listbox-multi-group-label"),
                    vec![],
                    vec![
                        listbox::item_group_label(
                            Some("showcase-listbox-multi-group-label"),
                            vec![],
                            vec![text("Cheese")],
                        ),
                        listbox::item(
                            OpenState::Open,
                            false,
                            false,
                            "cheddar",
                            None,
                            vec![],
                            vec![
                                listbox::item_text(None, vec![], vec![text("Cheddar")]),
                                listbox::item_indicator(OpenState::Open, vec![], vec![text("✓")]),
                            ],
                        ),
                        listbox::item(
                            OpenState::Open,
                            false,
                            false,
                            "mozzarella",
                            None,
                            vec![],
                            vec![
                                listbox::item_text(None, vec![], vec![text("Mozzarella")]),
                                listbox::item_indicator(OpenState::Open, vec![], vec![text("✓")]),
                            ],
                        ),
                    ],
                )],
            ),
        ],
    );

    section(
        "Listbox",
        "headless-ui の Listbox（role=\"listbox\"）に pre-styled-ui の recipe CSS を適用した静的掲示です。Select（ポップアップ型）と異なり trigger/positioner を持たず、常時展開のリストとして表示されます。左は single モード（1 項目選択済み、1 項目 disabled）、右は multiple モード（aria-multiselectable、複数項目選択済み・item-group 付き）です。",
        vec![row(vec![single, multiple])],
    )
}

/// Combobox 節: 入力によるフィルタリング後の listbox が開いた静的マークアップ
/// （イシュー #749）。[`combobox::filter_options`] を実演し、入力値
/// `"re"` に対するフィルタ結果（`"React"` のみ）をそのまま候補として掲示する。
fn combobox_section() -> Node {
    let options = [("vue", "Vue"), ("react", "React"), ("svelte", "Svelte")];
    let query = "re";
    let filtered = combobox::filter_options(&options, query);

    let items = filtered
        .into_iter()
        .map(|(value, label)| {
            combobox::item(
                OpenState::Closed,
                false,
                false,
                value,
                None,
                vec![],
                vec![combobox::item_text(None, vec![], vec![text(label)])],
            )
        })
        .collect();

    let node = combobox::root(
        Size::Md,
        OpenState::Open,
        vec![],
        vec![
            combobox::label(
                Some("showcase-combobox-label"),
                Some("showcase-combobox-input"),
                vec![],
                vec![text("Framework")],
            ),
            combobox::control(
                OpenState::Open,
                vec![],
                vec![
                    combobox::input(
                        OpenState::Open,
                        query,
                        false,
                        Some("showcase-combobox-content"),
                        None,
                        None,
                        vec![("id", "showcase-combobox-input")],
                    ),
                    combobox::trigger(
                        OpenState::Open,
                        false,
                        Some("showcase-combobox-content"),
                        vec![],
                        vec![text("▾")],
                    ),
                ],
            ),
            combobox::positioner(
                OpenState::Open,
                vec![],
                vec![combobox::content(
                    OpenState::Open,
                    Some("showcase-combobox-content"),
                    Some("showcase-combobox-label"),
                    vec![],
                    items,
                )],
            ),
        ],
    );
    section(
        "Combobox",
        &format!(
            "headless-ui の Combobox（role=\"combobox\"）に pre-styled-ui の recipe CSS を適用した静的掲示です。入力値 \"{query}\" による filter_options の絞り込み結果を候補として表示しています。positioner はフロー内配置へ中和しています。"
        ),
        vec![node],
    )
}

/// TreeView 節: 2〜3 階層の静的コレクション（イシュー #753）。
///
/// "src" ブランチのみ展開済み（`data-state="open"`）、"src/lib.rs" を選択中
/// （`data-selected`）で固定掲示する。positioner を持たないため
/// [`SHOWCASE_LAYOUT_CSS`] の中和ルール追加は不要（[`mod@tree_view`]
/// module doc「`size`/`color-palette` variant を提供しない」節参照）。
fn tree_view_section() -> Node {
    // SSR は本来 dispatch 履歴なしの初期状態から始まるが、ショーケースは
    // 「展開・選択済みの見た目」を固定掲示する目的のため、他セクション
    // （Accordion/Combobox 等）と同じく意図的に dispatch で非初期状態を作る。
    use fandhe_frontend_pre_styled_ui::fandhe_frontend_interactive::dispatch;
    let mut tree = TreeView::default();
    dispatch(&mut tree, "expand", "src");
    dispatch(&mut tree, "select", "src/lib.rs");

    let nodes = vec![
        TreeNode::new("src", "src").with_children(vec![
            TreeNode::new("src/lib.rs", "lib.rs"),
            TreeNode::new("src/nested", "nested")
                .with_children(vec![TreeNode::new("src/nested/util.rs", "util.rs")]),
        ]),
        TreeNode::new("Cargo.toml", "Cargo.toml"),
        TreeNode::new("README.md", "README.md").disabled(true),
    ];

    let root_children = tree.render_nodes(&nodes);
    let node = tree_view::root(
        vec![],
        vec![
            tree_view::label(vec![], vec![text("Project files")]),
            tree_view::tree(Some("Project files"), None, vec![], root_children),
        ],
    );

    section(
        "TreeView",
        "headless-ui の TreeView（role=\"tree\"/role=\"treeitem\"）に pre-styled-ui の recipe CSS を適用した静的掲示です。\"src\" ブランチを展開済み、\"src/lib.rs\" を選択中、\"README.md\" を disabled として固定表示しています。インデントは CSS custom property（--fandhe-tree-view-indent）で表現しています。",
        vec![node],
    )
}

/// JsonTreeView 節: JSON 風データ構造のツリー表示（イシュー #829、
/// [`tree_view_section`]（#753）の派生）。
///
/// [`json_tree_view::expanded_to_depth`] で ark-ui の `defaultExpandedDepth`
/// 相当の初期展開状態を決定的に作り、[`json_tree_view::render_json`] で
/// 型別配色付きのマークアップを組み立てる。name/version/tags/address の
/// ネストしたサンプルデータは ark-ui の json-tree-view 例に準拠する。
fn json_tree_view_section() -> Node {
    let data = JsonValue::Object(vec![
        (
            "name".to_string(),
            JsonValue::String("fandhe-frontend".to_string()),
        ),
        (
            "version".to_string(),
            JsonValue::String("0.16.0".to_string()),
        ),
        ("stable".to_string(), JsonValue::Bool(true)),
        ("deprecated".to_string(), JsonValue::Null),
        (
            "tags".to_string(),
            JsonValue::Array(vec![
                JsonValue::String("headless".to_string()),
                JsonValue::String("ui".to_string()),
            ]),
        ),
        (
            "address".to_string(),
            JsonValue::Object(vec![
                ("city".to_string(), JsonValue::String("Tokyo".to_string())),
                ("zip".to_string(), JsonValue::Number(100.0)),
            ]),
        ),
    ]);
    // defaultExpandedDepth 相当（深さ 2 まで展開）で「開いた見た目」を固定掲示する。
    let tree = json_tree_view::expanded_to_depth(&data, 2);
    let node = tree_view::root(
        vec![],
        vec![
            tree_view::label(vec![], vec![text("Package metadata")]),
            tree_view::tree(
                Some("Package metadata"),
                None,
                vec![],
                vec![json_tree_view::render_json(&tree, &data)],
            ),
        ],
    );

    section(
        "JsonTreeView",
        "headless-ui の JsonTreeView（tree_view #753 の派生、role=\"tree\"/role=\"treeitem\"）に pre-styled-ui の recipe CSS を適用した静的掲示です。expanded_to_depth(2) でルートと直下ブランチを展開済みとして固定表示し、値の型（string/number/bool/null/array/object）ごとに配色を切り替えています。",
        vec![node],
    )
}

/// Popover 節: 開いた状態の静的マークアップ（イシュー #691）。
///
/// [`dialog_section`] と同じく、実際の overlay 配置は recipe CSS
/// （`crates/pre-styled-ui/src/popover.rs`）が担い、掲示用にのみフロー内配置へ
/// 中和している。
fn popover_section() -> Node {
    let node = popover::root(
        OpenState::Open,
        vec![],
        vec![
            popover::trigger(
                OpenState::Open,
                false,
                Some("showcase-popover-content"),
                vec![],
                vec![text("More info")],
            ),
            popover::positioner(
                OpenState::Open,
                vec![],
                vec![popover::content(
                    OpenState::Open,
                    Some("showcase-popover-content"),
                    Some("showcase-popover-title"),
                    Some("showcase-popover-desc"),
                    vec![],
                    vec![
                        popover::title(
                            Some("showcase-popover-title"),
                            vec![],
                            vec![text("About this feature")],
                        ),
                        popover::description(
                            Some("showcase-popover-desc"),
                            vec![],
                            vec![text("必要なときだけ表示される補足情報です。")],
                        ),
                        popover::close_trigger(vec![], vec![text("Close")]),
                    ],
                )],
            ),
        ],
    );
    section(
        "Popover",
        "headless-ui の Popover（role=\"dialog\"、非モーダル）に pre-styled-ui の recipe CSS を適用した静的掲示です。positioner はフロー内配置へ中和しています（実際の overlay 配置は recipe CSS が担います）。",
        vec![node],
    )
}

/// FloatingPanel 節: 開いた状態（stage=default）の静的マークアップ
/// （イシュー #827）。positioner はドラッグ移動によるビューポート絶対座標
/// （`--fandhe-x`/`--fandhe-y`、[`floating_panel::FloatingPanel::position_style`]）
/// を持つが、ショーケース内ではフロー内配置へ中和している
/// （[`SHOWCASE_LAYOUT_CSS`]。実際の overlay 配置・ドラッグ配線は
/// recipe CSS/wasm 層が担う）。
fn floating_panel_section() -> Node {
    let panel = fandhe_frontend_pre_styled_ui::floating_panel::FloatingPanel::new(
        OpenState::Open,
        Stage::Default,
        24.0,
        24.0,
    );
    let style = panel.position_style();
    let node = panel.root(
        vec![],
        vec![
            panel.trigger(
                false,
                Some("showcase-floating-panel-content"),
                vec![],
                vec![text("Open panel")],
            ),
            panel.positioner(
                vec![("style", style.as_str())],
                vec![panel.content(
                    Some("showcase-floating-panel-content"),
                    Some("showcase-floating-panel-title"),
                    vec![],
                    vec![
                        floating_panel::header(
                            vec![],
                            vec![
                                floating_panel::title(
                                    Some("showcase-floating-panel-title"),
                                    vec![],
                                    vec![text("Panel title")],
                                ),
                                floating_panel::control(
                                    vec![],
                                    vec![
                                        panel.stage_trigger(
                                            Stage::Minimized,
                                            vec![],
                                            vec![text("_")],
                                        ),
                                        panel.stage_trigger(
                                            Stage::Maximized,
                                            vec![],
                                            vec![text("[]")],
                                        ),
                                        floating_panel::close_trigger(vec![], vec![text("x")]),
                                    ],
                                ),
                            ],
                        ),
                        panel.body(vec![], vec![text("ドラッグで移動できるパネルの本文です。")]),
                    ],
                )],
            ),
        ],
    );
    section(
        "FloatingPanel",
        "headless-ui の FloatingPanel（role=\"dialog\"、非モーダル）に pre-styled-ui の recipe CSS を適用した静的掲示です。stage=\"default\" の状態を固定表示しています。positioner はフロー内配置へ中和しています（実際のドラッグ移動・overlay 配置は recipe CSS/wasm 層が担います）。",
        vec![node],
    )
}

/// Tooltip 節: 開いた状態の静的マークアップ（イシュー #691）。
fn tooltip_section() -> Node {
    let node = tooltip::root(
        OpenState::Open,
        vec![],
        vec![
            tooltip::trigger(
                OpenState::Open,
                false,
                Some("showcase-tooltip-content"),
                vec![],
                vec![text("Hover target")],
            ),
            tooltip::positioner(
                OpenState::Open,
                vec![],
                vec![tooltip::content(
                    OpenState::Open,
                    Some("showcase-tooltip-content"),
                    vec![],
                    vec![text("補足のヒントテキストです。")],
                )],
            ),
        ],
    );
    section(
        "Tooltip",
        "headless-ui の Tooltip（role=\"tooltip\"、WAI-ARIA tooltip パターン）に pre-styled-ui の recipe CSS を適用した静的掲示です。positioner はフロー内配置へ中和しています。",
        vec![node],
    )
}

/// HoverCard 節: 開いた状態の静的マークアップ（イシュー #759）。
///
/// `trigger` はリンク先プレビュー用途の `a` 要素だが、掲示コンテンツは
/// 実ページへ解決されないため `href` は渡さない（`None`）。`build.rs` の
/// linkcheck が生成コンテンツ内の非空 `href` を持たない設計を前提とする
/// （`showcase_markup_has_no_href_attributes_for_linkcheck_neutrality`
/// 参照。[`breadcrumb_section`] が空文字列 `href=""` で同じ制約を満たす
/// のとは異なり、本節は `href` 属性自体を出力しない選択で満たす）。
fn hover_card_section() -> Node {
    let node = hover_card::root(
        OpenState::Open,
        HoverCardDelays::default(),
        vec![],
        vec![
            hover_card::trigger(
                OpenState::Open,
                None,
                vec![],
                vec![text("Hover to preview")],
            ),
            hover_card::positioner(
                OpenState::Open,
                vec![],
                vec![hover_card::content(
                    OpenState::Open,
                    None,
                    vec![],
                    vec![text("リンク先のプレビュー内容です。")],
                )],
            ),
        ],
    );
    section(
        "HoverCard",
        "headless-ui の HoverCard（リンク先プレビュー等 hover/focus で開閉するオーバーレイ）に pre-styled-ui の recipe CSS を適用した静的掲示です。positioner はフロー内配置へ中和しています。",
        vec![node],
    )
}

/// ToggleTip 節: 開いた状態の静的マークアップ（イシュー #761）。
///
/// [`tooltip_section`] と同じ視覚系だが、クリック開閉（`aria-expanded`/
/// `aria-controls`、`role="tooltip"` なし）である点が異なる（headless 層の
/// 3 者境界、`crates/headless-ui/src/toggle_tip.rs` モジュール doc 参照）。
fn toggle_tip_section() -> Node {
    let node = toggle_tip::root(
        OpenState::Open,
        vec![],
        vec![
            toggle_tip::trigger(
                OpenState::Open,
                false,
                Some("showcase-toggle-tip-content"),
                vec![],
                vec![text("More info")],
            ),
            toggle_tip::positioner(
                OpenState::Open,
                vec![],
                vec![toggle_tip::content(
                    OpenState::Open,
                    Some("showcase-toggle-tip-content"),
                    vec![],
                    vec![text("クリックで開閉する補足のヒントテキストです。")],
                )],
            ),
        ],
    );
    section(
        "ToggleTip",
        "headless-ui の ToggleTip（クリック開閉、role=\"tooltip\" なし）に pre-styled-ui の recipe CSS を適用した静的掲示です。positioner はフロー内配置へ中和しています。",
        vec![node],
    )
}

/// Switch 節: unchecked / checked / disabled の 3 態。
///
/// headless 層は `"checked"`/`"unchecked"` の `data-state` 語彙で状態を
/// 表現する（open/closed ではない、`fandhe_frontend_pre_styled_ui::switch`
/// のモジュール doc 参照）。フォーム意味論は visually-hidden な
/// `<input type="checkbox" role="switch">`（[`switch::hidden_input`]）が
/// 担い、見た目（トラック/つまみ）は `control`/`thumb` が装飾として担う。
fn switch_section() -> Node {
    let states = [
        (false, false, "showcase-switch-unchecked", "Unchecked"),
        (true, false, "showcase-switch-checked", "Checked"),
        (false, true, "showcase-switch-disabled", "Disabled"),
    ];
    let demo_row = row(states
        .iter()
        .map(|(checked, disabled, name, label)| {
            switch::root(
                Size::Md,
                ColorPalette::Accent,
                *checked,
                *disabled,
                vec![],
                vec![
                    switch::hidden_input(name, "on", *checked, *disabled, false, vec![]),
                    switch::control(
                        *checked,
                        *disabled,
                        vec![],
                        vec![switch::thumb(*checked, vec![], vec![])],
                    ),
                    switch::label(*checked, vec![], vec![text(*label)]),
                ],
            )
        })
        .collect());
    section(
        "Switch",
        "data-state=\"checked\"/\"unchecked\" で見た目が切り替わるオン/オフ スイッチ。visually-hidden な input[type=\"checkbox\"][role=\"switch\"] がフォーム送信・キーボード操作の意味論を担います。",
        vec![demo_row],
    )
}

/// RadioGroup 節: 3 択のうち 1 件が選択済み・1 件が disabled な静的掲示。
///
/// `label` パーツの `id` を `root` の `labelled_by` に渡し、グループ全体の
/// 見出しとの関連付け（`aria-labelledby`）を成立させる（headless
/// `radio_group` モジュールの契約）。
fn radio_group_section() -> Node {
    let label_id = "showcase-radio-label";
    let items = [
        ("plan-free", "Free", true, false),
        ("plan-pro", "Pro", false, false),
        ("plan-enterprise", "Enterprise", false, true),
    ];
    let mut children = vec![radio_group::label(
        Some(label_id),
        vec![],
        vec![text("Plan")],
    )];
    children.extend(items.iter().map(|(value, label, checked, disabled)| {
        radio_group::item(
            *checked,
            *disabled,
            value,
            vec![],
            vec![
                radio_group::item_hidden_input(
                    *checked,
                    *disabled,
                    Some("showcase-radio"),
                    value,
                    vec![],
                ),
                radio_group::item_control(*checked, *disabled, vec![]),
                radio_group::item_text(*checked, *disabled, vec![], vec![text(*label)]),
            ],
        )
    }));
    let demo = radio_group::root(
        Size::Md,
        ColorPalette::Accent,
        false,
        Some(Orientation::Vertical),
        Some(label_id),
        vec![],
        children,
    );
    section(
        "RadioGroup",
        "単一選択の選択肢グループ。ネイティブ input[type=\"radio\"] による排他選択・キーボード操作を data-scope=\"radio-group\" の anatomy へ重ねます。",
        vec![demo],
    )
}

/// 空 data URI（画像フェッチを一切発生させない `src`。イシュー #692 実装計画
/// 「外部フェッチ・404 を発生させない値」参照）。Error 状態デモの `image` src
/// として使う。
const AVATAR_EMPTY_IMAGE_SRC: &str = "data:,";

/// パーセントエンコード済みインライン SVG data URI（生の `<`・引用符を含まず、
/// GitHub Pages 上で外部リクエスト・404 を発生させない。Loaded 状態デモの
/// `image` src として使う）。単色円のプレースホルダーアイコン。
const AVATAR_INLINE_SVG_SRC: &str =
    "data:image/svg+xml,%3Csvg%20xmlns%3D%27http%3A%2F%2Fwww.w3.org%2F2000%2Fsvg%27%20viewBox%3D%270%200%2064%2064%27%3E%3Ccircle%20cx%3D%2732%27%20cy%3D%2732%27%20r%3D%2732%27%20fill%3D%27%234a90d9%27%2F%3E%3C%2Fsvg%3E";

/// Avatar 節: size（Sm/Md/Lg、いずれも `ImageStatus::Error` でフォールバック
/// 表示）と shape（Circle/Rounded/Square）の 2 軸。
///
/// `image` パーツの `src` は外部フェッチ・404 を発生させないダミー値
/// （[`AVATAR_EMPTY_IMAGE_SRC`]/[`AVATAR_INLINE_SVG_SRC`]）を使う
/// （`examples/headless-pre-styled-ui` の avatar 節と同じく実画像を同梱
/// しない方針）。`image` パーツ自体は `ImageStatus` に応じて headless 層が
/// `hidden` 存在属性を出力するため、Error 状態でも anatomy には含まれる。
fn avatar_section() -> Node {
    let size_row = row(vec![(Size::Sm, "FT"), (Size::Md, "FT"), (Size::Lg, "FT")]
        .into_iter()
        .map(|(size, initials)| {
            avatar::root(
                size,
                AvatarShape::default(),
                vec![],
                vec![
                    avatar::image(
                        ImageStatus::Error,
                        AVATAR_EMPTY_IMAGE_SRC,
                        "Fandhe Team",
                        vec![],
                    ),
                    avatar::fallback(ImageStatus::Error, vec![], vec![text(initials)]),
                ],
            )
        })
        .collect());

    let shape_row = row(vec![
        AvatarShape::Circle,
        AvatarShape::Rounded,
        AvatarShape::Square,
    ]
    .into_iter()
    .map(|shape| {
        avatar::root(
            Size::Md,
            shape,
            vec![],
            vec![
                avatar::image(
                    ImageStatus::Loaded,
                    AVATAR_INLINE_SVG_SRC,
                    "Fandhe Team",
                    vec![],
                ),
                avatar::fallback(ImageStatus::Loaded, vec![], vec![text("FT")]),
            ],
        )
    })
    .collect());

    section(
        "Avatar",
        "size（Sm/Md/Lg）・shape（Circle/Rounded/Square）の 2 軸を持つユーザー画像表示。画像読み込み状態（ImageStatus）を固定し、Error 時はイニシャルのフォールバック表示、Loaded 時は画像表示を掲示します。",
        vec![size_row, shape_row],
    )
}

/// Checkbox 節: unchecked / checked / indeterminate / disabled の 4 態
/// （イシュー #730）。
///
/// headless 層は checked/unchecked/indeterminate の 3 値 `data-state` 語彙を
/// 持つ（`fandhe_frontend_pre_styled_ui::checkbox` のモジュール doc
/// 参照）。フォーム意味論は visually-hidden な `<input type="checkbox">`
/// （[`checkbox::hidden_input`]）が担い、見た目（チェックマーク）は
/// `control`/`indicator` が装飾として担う。
fn checkbox_section() -> Node {
    let states = [
        (
            CheckedState::Unchecked,
            false,
            "showcase-checkbox-unchecked",
            "Unchecked",
        ),
        (
            CheckedState::Checked,
            false,
            "showcase-checkbox-checked",
            "Checked",
        ),
        (
            CheckedState::Indeterminate,
            false,
            "showcase-checkbox-indeterminate",
            "Indeterminate",
        ),
        (
            CheckedState::Checked,
            true,
            "showcase-checkbox-disabled",
            "Disabled",
        ),
    ];
    let demo_row = row(states
        .iter()
        .map(|(checked, disabled, name, label)| {
            let props = CheckboxProps {
                checked: *checked,
                disabled: *disabled,
                ..CheckboxProps::default()
            };
            checkbox::root(
                Size::Md,
                ColorPalette::Accent,
                &props,
                vec![],
                vec![
                    checkbox::hidden_input(&props, name, "on", vec![]),
                    checkbox::control(
                        &props,
                        vec![],
                        vec![checkbox::indicator(&props, vec![], vec![])],
                    ),
                    checkbox::label(&props, vec![], vec![text(*label)]),
                ],
            )
        })
        .collect());
    section(
        "Checkbox",
        "data-state=\"checked\"/\"unchecked\"/\"indeterminate\" の 3 態を持つチェックボックス。visually-hidden な input[type=\"checkbox\"] がフォーム送信・キーボード操作の意味論を担い、チェックマークは CSS の border 合成で描画します（画像アセット不使用）。",
        vec![demo_row],
    )
}

// Input / Textarea / NativeSelect 節群（イシュー #737 で導入、#941 で複合節
// form_controls_section から 3 部品ページ相当の関数へ分解）。いずれも
// 状態機械を持たない静的フォーム部品で、ブラウザネイティブ挙動をそのまま
// 尊重する。アクセシビリティ配線（`id`・ネイティブ `disabled`/`required`/
// `readonly`・`aria-invalid`・`aria-describedby`・`data-*`）は headless
// `field::*`（#538/#602）へ全面委譲するため、本節では invalid/disabled の
// 2 態と variant/size の切り替えのみを掲示する
// （`fandhe_frontend_pre_styled_ui::input` モジュール doc 参照）。
// `color-palette` 軸は提供しない設計のため掲示しない。3 関数が共有する
// `FieldProps` ビルダーをモジュールレベル関数として引き上げる。

/// invalid/disabled/required いずれも立てない既定の [`FieldProps`]。
fn plain_field(id: &'static str) -> FieldProps<'static> {
    FieldProps {
        id,
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: false,
    }
}

/// `invalid: true` のみを立てた [`FieldProps`]（[`plain_field`] 派生）。
fn invalid_field(id: &'static str) -> FieldProps<'static> {
    FieldProps {
        invalid: true,
        ..plain_field(id)
    }
}

/// `disabled: true` のみを立てた [`FieldProps`]（[`plain_field`] 派生）。
fn disabled_field(id: &'static str) -> FieldProps<'static> {
    FieldProps {
        disabled: true,
        ..plain_field(id)
    }
}

/// Input 節: Outline（既定）/ Invalid / Disabled の 3 態。
fn input_section() -> Node {
    let input_row = row(vec![
        input::input(
            &InputProps::default(),
            &plain_field("showcase-input-default"),
            vec![("placeholder", "Outline (default)")],
        ),
        // invalid 時、headless `field::input` は `aria-describedby` に
        // `{id}-error-text` を出力する（`field.rs` の describedby 合成則）。
        // 参照先の id を持つ `field::error_text` を併設し、存在しない id への
        // 参照を残さない（Bugbot 指摘、PR #783）。
        //
        // ラッパー div には `.showcase-form-field-group`（`width: 100%`）を
        // 付与する。付与しないと `showcase-row` の直接 flex item がこの div
        // になり、兄弟 input が持つ `width: 100%`（`field` recipe base）による
        // flex-basis 解決を div 自身が持たず auto（contents 由来の縮小）に
        // なってしまい、Invalid デモだけ Default/Disabled より狭く描画される
        // （Bugbot 指摘、PR #783 review）。
        div(
            vec![("class", "showcase-form-field-group")],
            vec![
                input::input(
                    &InputProps::default(),
                    &invalid_field("showcase-input-invalid"),
                    vec![("placeholder", "Invalid")],
                ),
                input::error_text(
                    &invalid_field("showcase-input-invalid"),
                    vec![],
                    vec![text("This field is required.")],
                ),
            ],
        ),
        input::input(
            &InputProps::default(),
            &disabled_field("showcase-input-disabled"),
            vec![("placeholder", "Disabled")],
        ),
    ]);

    section(
        "Input",
        "ブラウザネイティブ挙動をそのまま尊重する静的テキスト入力部品。invalid/disabled 状態は headless field:: へ委譲した data-* 属性・aria-invalid で表現します。",
        vec![input_row],
    )
}

/// Textarea 節: Outline（既定）の複数行テキスト入力。
fn textarea_section() -> Node {
    let textarea_row = row(vec![textarea::textarea(
        &TextareaProps::default(),
        &plain_field("showcase-textarea-default"),
        false,
        vec![("placeholder", "Outline (default)")],
        vec![],
    )]);

    section(
        "Textarea",
        "ブラウザネイティブ挙動をそのまま尊重する静的複数行テキスト入力部品。",
        vec![textarea_row],
    )
}

/// Native Select 節: 素の `<select>`/`<option>` をスタイル化。
fn native_select_section() -> Node {
    let native_select_row = row(vec![native_select::native_select(
        &NativeSelectProps::default(),
        &plain_field("showcase-native-select-default"),
        vec![],
        vec![
            el("option", vec![("value", "jp")], vec![text("Japan")]),
            el("option", vec![("value", "us")], vec![text("United States")]),
        ],
    )]);

    section(
        "Native Select",
        "素の select/option 要素をそのまま styled 化した選択部品。",
        vec![native_select_row],
    )
}

/// NumberInput 節: 中間値・境界値（min 到達で decrement disabled）・
/// disabled の 3 態。
///
/// headless 層は連続量の値を扱うため `data-state` を持たず、境界到達は
/// increment/decrement トリガーの `data-disabled` 存在属性のみで表現する
/// （`fandhe_frontend_pre_styled_ui::number_input` のモジュール doc 参照）。
fn number_input_section() -> Node {
    let mid = number_input::root(
        Size::Md,
        false,
        false,
        vec![],
        vec![
            number_input::label(
                false,
                false,
                Some("showcase-number-input-mid"),
                vec![],
                vec![text("Quantity")],
            ),
            number_input::control(
                false,
                false,
                vec![],
                vec![
                    number_input::input(
                        "quantity",
                        Some("showcase-number-input-mid"),
                        Some("5"),
                        "0",
                        "10",
                        NumberInputFlags::default(),
                        vec![],
                    ),
                    number_input::increment_trigger(
                        Some("showcase-number-input-mid"),
                        false,
                        vec![],
                        vec![text("+")],
                    ),
                    number_input::decrement_trigger(
                        Some("showcase-number-input-mid"),
                        false,
                        vec![],
                        vec![text("-")],
                    ),
                ],
            ),
        ],
    );
    let at_min = number_input::root(
        Size::Md,
        false,
        false,
        vec![],
        vec![
            number_input::label(
                false,
                false,
                Some("showcase-number-input-min"),
                vec![],
                vec![text("At min")],
            ),
            number_input::control(
                false,
                false,
                vec![],
                vec![
                    number_input::input(
                        "quantity-min",
                        Some("showcase-number-input-min"),
                        Some("0"),
                        "0",
                        "10",
                        NumberInputFlags::default(),
                        vec![],
                    ),
                    number_input::increment_trigger(
                        Some("showcase-number-input-min"),
                        false,
                        vec![],
                        vec![text("+")],
                    ),
                    // 下限到達のため decrement トリガーを disabled にする
                    // （境界到達時の唯一の視覚的合図、モジュール doc 参照）。
                    number_input::decrement_trigger(
                        Some("showcase-number-input-min"),
                        true,
                        vec![],
                        vec![text("-")],
                    ),
                ],
            ),
        ],
    );
    let disabled = number_input::root(
        Size::Md,
        true,
        false,
        vec![],
        vec![
            number_input::label(
                true,
                false,
                Some("showcase-number-input-disabled"),
                vec![],
                vec![text("Disabled")],
            ),
            number_input::control(
                true,
                false,
                vec![],
                vec![
                    number_input::input(
                        "quantity-disabled",
                        Some("showcase-number-input-disabled"),
                        Some("3"),
                        "0",
                        "10",
                        NumberInputFlags {
                            disabled: true,
                            ..NumberInputFlags::default()
                        },
                        vec![],
                    ),
                    number_input::increment_trigger(
                        Some("showcase-number-input-disabled"),
                        true,
                        vec![],
                        vec![text("+")],
                    ),
                    number_input::decrement_trigger(
                        Some("showcase-number-input-disabled"),
                        true,
                        vec![],
                        vec![text("-")],
                    ),
                ],
            ),
        ],
    );
    let demo_row = row(vec![mid, at_min, disabled]);
    section(
        "NumberInput",
        "min/max/step でクランプされる数値入力。increment/decrement トリガーは境界到達時に data-disabled を伴い無効化されます。",
        vec![demo_row],
    )
}

/// PasswordInput 節: 表示切替トリガー付きパスワード入力の Hidden/Visible/
/// Invalid/Disabled 4 状態を静的掲示する（イシュー #740）。
///
/// `visibility_trigger` へ `aria-label` を呼び出し側 attrs として付与する
/// お手本を示す（headless 層は固定文言を持たない、
/// `crates/headless-ui/src/password_input.rs` rustdoc 参照）。
fn password_input_section() -> Node {
    let states = [
        (false, false, false, "showcase-password-hidden", "Hidden"),
        (true, false, false, "showcase-password-visible", "Visible"),
        (false, true, false, "showcase-password-invalid", "Invalid"),
        (false, false, true, "showcase-password-disabled", "Disabled"),
    ];
    let demo_row = row(states
        .iter()
        .map(|(visible, invalid, disabled, id, label)| {
            let props = PasswordInputProps {
                id,
                disabled: *disabled,
                invalid: *invalid,
                required: false,
                autocomplete: PasswordAutocomplete::CurrentPassword,
            };
            password_input::root(
                Size::Md,
                ColorPalette::Accent,
                *visible,
                &props,
                vec![],
                vec![
                    password_input::label(&props, vec![], vec![text(*label)]),
                    password_input::control(
                        *visible,
                        &props,
                        vec![],
                        vec![
                            password_input::input(*visible, &props, vec![]),
                            password_input::visibility_trigger(
                                *visible,
                                &props,
                                vec![("aria-label", "Toggle password visibility")],
                                vec![text(if *visible { "Hide" } else { "Show" })],
                            ),
                        ],
                    ),
                ],
            )
        })
        .collect());
    section(
        "PasswordInput",
        "data-state=\"visible\"/\"hidden\" で type=\"password\"/\"text\" が切り替わるパスワード入力。visibility-trigger は aria-pressed/aria-controls で意味論を担い、パスワード値そのものは一切保持しません。",
        vec![demo_row],
    )
}

/// TagsInput 節: 通常タグ数件・max 到達（`data-invalid`/`aria-invalid`）・
/// disabled の 3 態。
///
/// `control` は `role="listbox"`、各タグの `item-preview` は `role="option"`
/// （headless 層の listbox 相当 ARIA、`fandhe_frontend_pre_styled_ui::tags_input`
/// のモジュール doc 参照）。SSR 静的掲示のため編集モード
/// （`item-input`/`data-editing`）は掲載しない（wasm 層の対話が必要なため、
/// モジュール rustdoc「スコープ外」節参照）。
fn tags_input_section() -> Node {
    fn tag_item(tag: &str, disabled: bool) -> Node {
        tags_input::item(
            disabled,
            false,
            vec![],
            vec![tags_input::item_preview(
                false,
                vec![],
                vec![
                    tags_input::item_text(vec![], vec![text(tag)]),
                    tags_input::item_delete_trigger(tag, disabled, vec![], vec![text("\u{00d7}")]),
                ],
            )],
        )
    }

    let normal = tags_input::root(
        Size::Md,
        false,
        vec![],
        vec![
            tags_input::label(vec![], vec![text("Skills")]),
            tags_input::control(
                false,
                false,
                "Skills",
                vec![],
                vec![
                    tag_item("rust", false),
                    tag_item("wasm", false),
                    tags_input::input("", false, false, vec![]),
                ],
            ),
            tags_input::hidden_input("skills", "rust,wasm", false, vec![]),
        ],
    );

    let at_max = tags_input::root(
        Size::Md,
        false,
        vec![],
        vec![
            tags_input::label(vec![], vec![text("At max (2)")]),
            tags_input::control(
                false,
                // max 到達のため `control` へ data-invalid、`input` へ
                // data-invalid/aria-invalid を出力する（境界到達時の唯一の
                // 視覚的合図、モジュール rustdoc「セキュリティ不変条件」節参照）。
                true,
                "At max",
                vec![],
                vec![
                    tag_item("a", false),
                    tag_item("b", false),
                    tags_input::input("", false, true, vec![]),
                ],
            ),
            tags_input::hidden_input("at-max", "a,b", false, vec![]),
        ],
    );

    let disabled = tags_input::root(
        Size::Md,
        true,
        vec![],
        vec![
            tags_input::label(vec![], vec![text("Disabled")]),
            tags_input::control(
                true,
                false,
                "Disabled",
                vec![],
                vec![
                    tag_item("readonly", true),
                    tags_input::input("", true, false, vec![]),
                ],
            ),
            tags_input::hidden_input("disabled-tags", "readonly", true, vec![]),
        ],
    );

    let demo_row = row(vec![normal, at_max, disabled]);
    section(
        "TagsInput",
        "自由入力によるタグ配列。control は role=\"listbox\"、各タグは role=\"option\" を持ち、max 到達時は input が data-invalid/aria-invalid を伴います。",
        vec![demo_row],
    )
}

/// FileUpload 節（イシュー #840）: 通常（受理済み 1 件）・disabled の 2 態。
/// `File` オブジェクトは headless 層で一切保持せず、ここでは静的な
/// `FileUploadItem` メタデータのみを直接組み立てて表示する（実 `File` API
/// 接触は `fandhe-frontend-wasm-full::headless_file_upload` の配線層のみが
/// 担う、`file_upload` モジュール rustdoc「保留解除」節参照）。
fn file_upload_section() -> Node {
    fn file_item(name: &str, size_bytes: u64, disabled: bool) -> Node {
        let size_text = file_upload::item_size_text(size_bytes);
        file_upload::item(
            disabled,
            vec![],
            vec![
                file_upload::item_name(vec![], vec![text(name)]),
                file_upload::item_size_text_node(vec![], vec![text(&size_text)]),
                file_upload::item_delete_trigger(name, disabled, vec![], vec![text("\u{00d7}")]),
            ],
        )
    }

    let normal = file_upload::root(
        Size::Md,
        false,
        vec![],
        vec![
            file_upload::label(vec![], vec![text("Attachments")]),
            file_upload::dropzone(
                false,
                false,
                vec![],
                vec![
                    file_upload::trigger(false, vec![], vec![text("Browse files")]),
                    file_upload::hidden_input("image/*,.pdf", true, false, vec![]),
                ],
            ),
            file_upload::item_group(vec![], vec![file_item("report.pdf", 204_800, false)]),
            file_upload::clear_trigger(false, vec![], vec![text("Clear all")]),
        ],
    );

    let disabled = file_upload::root(
        Size::Md,
        true,
        vec![],
        vec![
            file_upload::label(vec![], vec![text("Disabled")]),
            file_upload::dropzone(
                true,
                false,
                vec![],
                vec![
                    file_upload::trigger(true, vec![], vec![text("Browse files")]),
                    file_upload::hidden_input("image/*,.pdf", true, true, vec![]),
                ],
            ),
            file_upload::item_group(vec![], vec![file_item("locked.txt", 1024, true)]),
            file_upload::clear_trigger(true, vec![], vec![text("Clear all")]),
        ],
    );

    let demo_row = row(vec![normal, disabled]);
    section(
        "FileUpload",
        "ファイルメタデータ（name/size/mime type）のみを扱い、File オブジェクト自体は headless 層で保持しません。実 File API 接触は wasm-full 側の配線層に隔離されています。",
        vec![demo_row],
    )
}

/// RatingGroup 節: 選択中（value=3）・readonly（他ユーザーの平均評価想定）・
/// disabled の 3 態。星形 indicator は外部リソース非参照の `clip-path`
/// インライン表現（`fandhe_frontend_pre_styled_ui::rating_group` のモジュール
/// doc「星形 indicator」節参照）。`hidden_input` はフォーム送信用の現在値を
/// 送るネイティブ input（視覚上非表示、`display: none` の既定 CSS）。
fn rating_group_section() -> Node {
    let build = |id_prefix: &'static str, value: Option<u32>, disabled: bool, readonly: bool| {
        let g = RatingGroup::new(5, value, readonly);
        let label_id = format!("{id_prefix}-label");
        let mut children = vec![rating_group::label(
            Some(label_id.as_str()),
            vec![],
            vec![text("Rate this product")],
        )];
        let items: Vec<Node> = (1..=g.count())
            .map(|i| {
                let checked = g.is_checked(i);
                let highlighted = g.is_highlighted(i);
                rating_group::item(
                    i,
                    RatingItemFlags {
                        checked,
                        highlighted,
                        disabled,
                        readonly,
                    },
                    &format!("{i} star{}", if i == 1 { "" } else { "s" }),
                    vec![],
                    vec![],
                )
            })
            .collect();
        children.push(rating_group::control(
            Some(label_id.as_str()),
            vec![],
            items,
        ));
        children.push(rating_group::hidden_input(
            Some("rating"),
            g.value_text().as_str(),
            disabled,
            vec![],
        ));
        rating_group::root(
            Size::Md,
            ColorPalette::Accent,
            disabled,
            readonly,
            vec![],
            children,
        )
    };

    let selected = build("showcase-rating-selected", Some(3), false, false);
    let readonly = build("showcase-rating-readonly", Some(4), false, true);
    let disabled = build("showcase-rating-disabled", Some(2), true, false);

    section(
        "RatingGroup",
        "1..=count の星評価。data-highlighted が塗り表示（hover プレビュー優先）、data-checked が確定選択を表します。星形は SVG/画像 URL を使わない clip-path によるインライン表現です。",
        vec![row(vec![selected, readonly, disabled])],
    )
}

/// Slider 節: 中間値・境界値（max 到達）・disabled の 3 態。
///
/// `range`/`thumb` の塗りつぶし・位置は headless 中立な
/// [`Slider::percent`] から導出する `--fandhe-slider-percent` CSS custom
/// property の 1 点のみで伝搬する
/// （`fandhe_frontend_pre_styled_ui::slider` のモジュール doc 参照）。
fn slider_section() -> Node {
    let mid_state = Slider::new(0.0, 100.0, 1.0, 40.0, Orientation::Horizontal);
    let mid = slider::root(
        Size::Md,
        ColorPalette::Accent,
        &mid_state,
        false,
        vec![],
        vec![
            slider::label(vec![], vec![text("Volume")]),
            slider::control(
                Orientation::Horizontal,
                false,
                vec![],
                vec![
                    slider::track(
                        Orientation::Horizontal,
                        false,
                        vec![],
                        vec![slider::range(&mid_state, false, vec![])],
                    ),
                    slider::thumb_styled(&mid_state, Some("40 percent"), false, vec![]),
                ],
            ),
            slider::hidden_input("volume", "40", false, vec![]),
        ],
    );

    let at_max_state = Slider::new(0.0, 100.0, 1.0, 100.0, Orientation::Horizontal);
    let at_max = slider::root(
        Size::Md,
        ColorPalette::Accent,
        &at_max_state,
        false,
        vec![],
        vec![
            slider::label(vec![], vec![text("At max")]),
            slider::control(
                Orientation::Horizontal,
                false,
                vec![],
                vec![
                    slider::track(
                        Orientation::Horizontal,
                        false,
                        vec![],
                        vec![slider::range(&at_max_state, false, vec![])],
                    ),
                    slider::thumb_styled(&at_max_state, Some("100 percent"), false, vec![]),
                ],
            ),
            slider::hidden_input("volume-max", "100", false, vec![]),
        ],
    );

    let disabled_state = Slider::new(0.0, 100.0, 1.0, 25.0, Orientation::Horizontal);
    let disabled = slider::root(
        Size::Md,
        ColorPalette::Accent,
        &disabled_state,
        true,
        vec![],
        vec![
            slider::label(vec![], vec![text("Disabled")]),
            slider::control(
                Orientation::Horizontal,
                true,
                vec![],
                vec![
                    slider::track(
                        Orientation::Horizontal,
                        true,
                        vec![],
                        vec![slider::range(&disabled_state, true, vec![])],
                    ),
                    slider::thumb_styled(&disabled_state, Some("25 percent"), true, vec![]),
                ],
            ),
            slider::hidden_input("volume-disabled", "25", true, vec![]),
        ],
    );

    let demo_row = row(vec![mid, at_max, disabled]);
    section(
        "Slider",
        "min/max/step でクランプされる連続値スライダー。塗りつぶし・つまみの位置は --fandhe-slider-percent の 1 点で伝搬します。",
        vec![demo_row],
    )
}

/// Editable 節: preview 表示・edit 中・disabled の 3 態。
///
/// preview 中は `input` が `hidden`・`preview` が可視、edit 中はその逆
/// （`fandhe_frontend_pre_styled_ui::editable` のモジュール doc「`input`/
/// `preview` の重ね合わせレイアウト」参照）。
fn editable_section() -> Node {
    let preview_mode = editable::root(
        Size::Md,
        EditMode::Preview,
        false,
        false,
        Default::default(),
        Default::default(),
        vec![],
        vec![
            editable::label(
                EditMode::Preview,
                false,
                Some("showcase-editable-preview"),
                vec![],
                vec![text("Name")],
            ),
            editable::area(
                EditMode::Preview,
                false,
                vec![],
                vec![
                    editable::input(
                        EditMode::Preview,
                        "name",
                        "Ada Lovelace",
                        EditableInputProps {
                            id: Some("showcase-editable-preview"),
                            ..EditableInputProps::default()
                        },
                        EditableInputFlags::default(),
                        vec![],
                    ),
                    editable::preview(EditMode::Preview, false, vec![], vec![text("Ada Lovelace")]),
                ],
            ),
            editable::control(
                EditMode::Preview,
                vec![],
                vec![editable::edit_trigger(
                    EditMode::Preview,
                    false,
                    vec![],
                    vec![text("Edit")],
                )],
            ),
        ],
    );

    let editing = editable::root(
        Size::Md,
        EditMode::Edit,
        false,
        false,
        Default::default(),
        Default::default(),
        vec![],
        vec![
            editable::label(
                EditMode::Edit,
                false,
                Some("showcase-editable-editing"),
                vec![],
                vec![text("Name")],
            ),
            editable::area(
                EditMode::Edit,
                false,
                vec![],
                vec![
                    editable::input(
                        EditMode::Edit,
                        "name-editing",
                        "Grace Hopper",
                        EditableInputProps {
                            id: Some("showcase-editable-editing"),
                            ..EditableInputProps::default()
                        },
                        EditableInputFlags::default(),
                        vec![],
                    ),
                    editable::preview(EditMode::Edit, false, vec![], vec![text("Grace Hopper")]),
                ],
            ),
            editable::control(
                EditMode::Edit,
                vec![],
                vec![
                    editable::submit_trigger(EditMode::Edit, false, vec![], vec![text("Save")]),
                    editable::cancel_trigger(EditMode::Edit, false, vec![], vec![text("Cancel")]),
                ],
            ),
        ],
    );

    let disabled = editable::root(
        Size::Md,
        EditMode::Preview,
        true,
        false,
        Default::default(),
        Default::default(),
        vec![],
        vec![
            editable::label(
                EditMode::Preview,
                true,
                Some("showcase-editable-disabled"),
                vec![],
                vec![text("Disabled")],
            ),
            editable::area(
                EditMode::Preview,
                false,
                vec![],
                vec![
                    editable::input(
                        EditMode::Preview,
                        "name-disabled",
                        "Locked value",
                        EditableInputProps {
                            id: Some("showcase-editable-disabled"),
                            ..EditableInputProps::default()
                        },
                        EditableInputFlags {
                            disabled: true,
                            ..EditableInputFlags::default()
                        },
                        vec![],
                    ),
                    editable::preview(EditMode::Preview, false, vec![], vec![text("Locked value")]),
                ],
            ),
            editable::control(
                EditMode::Preview,
                vec![],
                vec![editable::edit_trigger(
                    EditMode::Preview,
                    true,
                    vec![],
                    vec![text("Edit")],
                )],
            ),
        ],
    );

    let demo_row = row(vec![preview_mode, editing, disabled]);
    section(
        "Editable",
        "preview/edit の 2 モードを切り替えるインプレース編集。input/preview は data-* と hidden 属性で排他表示されます。",
        vec![demo_row],
    )
}

/// SegmentGroup 節（イシュー #743）: 既定（選択済み）・disabled・Size 3 種の
/// 静的掲示。状態機械（[`fandhe_frontend_pre_styled_ui::segment_group::SegmentGroup`]、
/// `radio_group::RadioGroup` への全委譲）は使わず、他の docs-site 節と同じく
/// SSR 静的マークアップのみを組み立てる（本モジュール冒頭「インタラクティブ
/// 部品の扱い」節参照）。indicator の位置は選択項目の `(index, count)` から
/// 手計算で `segment_group::indicator` へ渡す（headless 層の SSR 決定的な
/// 位置表現契約、`crates/headless-ui/src/segment_group.rs` module doc 参照）。
fn segment_group_demo(id_prefix: &str, size: Size, disabled: bool, selected_index: usize) -> Node {
    let items = ["List", "Grid", "Table"];
    let mut children = vec![segment_group::indicator(
        Some((selected_index, items.len())),
        None,
        vec![],
    )];
    children.extend(items.iter().enumerate().map(|(index, label)| {
        let checked = index == selected_index;
        let value = label.to_lowercase();
        segment_group::item(
            checked,
            disabled,
            &value,
            vec![],
            vec![
                segment_group::item_hidden_input(
                    checked,
                    disabled,
                    Some(id_prefix),
                    &value,
                    vec![],
                ),
                segment_group::item_control(checked, disabled, vec![]),
                segment_group::item_text(checked, disabled, vec![], vec![text(*label)]),
            ],
        )
    }));
    segment_group::root(size, disabled, None, None, vec![], children)
}

fn segment_group_section() -> Node {
    let size_row = row(vec![
        segment_group_demo("showcase-segment-sm", Size::Sm, false, 0),
        segment_group_demo("showcase-segment-md", Size::Md, false, 1),
        segment_group_demo("showcase-segment-lg", Size::Lg, false, 2),
    ]);
    let disabled_demo = segment_group_demo("showcase-segment-disabled", Size::Md, true, 0);
    section(
        "SegmentGroup",
        "単一選択のセグメント UI（segmented control）。ネイティブ input[type=\"radio\"] による排他選択を data-scope=\"segment-group\" の anatomy へ重ね、選択中の項目を indicator の CSS 変数（--fandhe-segment-group-index/-count）で示します。状態機械は RadioGroup（SingleSelect）への全委譲です。",
        vec![size_row, disabled_demo],
    )
}

/// Toggle 節（イシュー #980）: `pressed`/`disabled` 2 状態フラグを直接
/// 引数で受け取るネイティブ `<button>` ベースのトグル。`indicator` を
/// children に含めることで anatomy 導出（`component_page::collect_anatomy_parts`）
/// が `{root, indicator}` を得られるようにする（`component_specs::forms::TOGGLE`
/// の Demo フォールバックから本節へ移設。イシュー #979 で CSS 配線
/// （[`stylesheet`]）は完了済み、本節は Demo 節の正経路供給を担う）。
fn toggle_section() -> Node {
    let checkmark = || vec![text("✔")];
    let states = [
        (false, false, "Off"),
        (true, false, "On"),
        (true, true, "Disabled"),
    ];
    let state_row = row(states
        .iter()
        .map(|(pressed, disabled, label)| {
            toggle::root(
                Size::Md,
                ColorPalette::Accent,
                *pressed,
                *disabled,
                vec![],
                vec![
                    toggle::indicator(*pressed, vec![], checkmark()),
                    text(*label),
                ],
            )
        })
        .collect());

    let sizes = [
        (Size::Sm, "Small"),
        (Size::Md, "Medium"),
        (Size::Lg, "Large"),
    ];
    let size_row = row(sizes
        .iter()
        .map(|(size, label)| {
            toggle::root(
                *size,
                ColorPalette::Accent,
                true,
                false,
                vec![],
                vec![toggle::indicator(true, vec![], checkmark()), text(*label)],
            )
        })
        .collect());

    let palette_row = row(palettes()
        .iter()
        .map(|(palette, label)| {
            toggle::root(
                Size::Md,
                *palette,
                true,
                false,
                vec![],
                vec![toggle::indicator(true, vec![], checkmark()), text(*label)],
            )
        })
        .collect());

    section(
        "Toggle",
        "押下状態を持つ 2 状態ボタン。data-state 語彙は Switch の checked/unchecked ではなく on/off です（root 自身がネイティブ button であり、hidden input を持ちません）。",
        vec![state_row, size_row, palette_row],
    )
}

/// ToggleGroup 節（イシュー #980）: 高々 1 項目押下の single / 複数押下の
/// multiple の 2 状態機械を選べるボタングループ。`orientation` は
/// `data-orientation` のみで `aria-orientation` は付与しません
/// （`crates/headless-ui/src/toggle_group.rs` 参照）。anatomy 導出は
/// `{root, item}`（`component_specs::forms::TOGGLE_GROUP` の Demo
/// フォールバックから本節へ移設）。
fn toggle_group_section() -> Node {
    let horizontal = toggle_group::root(
        Size::Md,
        ColorPalette::Accent,
        false,
        None,
        None,
        vec![],
        vec![
            toggle_group::item(false, false, "left", vec![], vec![text("Left")]),
            toggle_group::item(true, false, "center", vec![], vec![text("Center")]),
            toggle_group::item(false, false, "right", vec![], vec![text("Right")]),
        ],
    );
    let vertical = toggle_group::root(
        Size::Md,
        ColorPalette::Accent,
        false,
        Some(Orientation::Vertical),
        None,
        vec![],
        vec![
            toggle_group::item(true, false, "top", vec![], vec![text("Top")]),
            toggle_group::item(false, false, "middle", vec![], vec![text("Middle")]),
            toggle_group::item(false, false, "bottom", vec![], vec![text("Bottom")]),
        ],
    );
    let disabled = toggle_group::root(
        Size::Md,
        ColorPalette::Accent,
        true,
        None,
        None,
        vec![],
        vec![
            toggle_group::item(false, true, "left", vec![], vec![text("Left")]),
            toggle_group::item(false, true, "right", vec![], vec![text("Right")]),
        ],
    );
    section(
        "Toggle Group",
        "複数の Toggle をまとめて排他/複数選択させるグループ部品。root にのみ role=\"group\" を固定付与します（RadioGroup の role=\"radiogroup\" とは異なります）。",
        vec![stack(vec![horizontal, vertical, disabled])],
    )
}

/// Carousel 節: 3 スライド中の 2 番目を現在位置として固定表示（イシュー #754）。
///
/// headless の [`Carousel`] 状態機械（`index=1, slide_count=3, loop=false`）を
/// 使って `item-group`/`item`/`indicator` へ現在位置を注入し、
/// pre-styled-ui の recipe CSS（`--fandhe-carousel-index` CSS カスタム
/// プロパティによる transform ベースのスライド位置表現）を適用した静的掲示
/// です。実際のクリック操作（dispatch 状態遷移）は wasm 層の責務であり
/// 本ショーケースのスコープ外（モジュール冒頭 rustdoc「インタラクティブ
/// 部品の扱い」節参照）。
fn carousel_section() -> Node {
    let c = Carousel::new(1, 3, false, Orientation::Horizontal);
    let slides = ["Slide A", "Slide B", "Slide C"];

    let node = carousel::root(
        Size::Md,
        Orientation::Horizontal,
        "Featured products",
        vec![],
        vec![
            c.control(
                vec![],
                vec![
                    c.prev_trigger("Previous slide", vec![], vec![]),
                    c.item_group(
                        vec![],
                        slides
                            .iter()
                            .enumerate()
                            .map(|(i, label)| c.item(i, vec![], vec![text(*label)]))
                            .collect(),
                    ),
                    c.next_trigger("Next slide", vec![], vec![]),
                ],
            ),
            c.indicator_group(
                vec![],
                (0..slides.len()).map(|i| c.indicator(i, vec![])).collect(),
            ),
        ],
    );
    section(
        "Carousel",
        "headless-ui の Carousel（role=\"region\" aria-roledescription=\"carousel\"）に pre-styled-ui の recipe CSS を適用した静的掲示です。3 スライド中 2 番目（index=1）を現在位置として固定表示しています。--fandhe-carousel-index CSS カスタムプロパティによる transform ベースのスライド位置表現で、JS 計測に依存しません。autoplay・ドラッグ操作は本イシューのスコープ外です。",
        vec![node],
    )
}

/// Toast 節（イシュー #760）: status（info/success/warning/error）4 態を
/// 1 つの group（`placement="bottom-end"` 既定）内に固定掲示する。
///
/// headless 層のキュー状態機械（[`fandhe_frontend_pre_styled_ui::toast::Toaster`]
/// 相当。ここでは非再エクスポートの `Toaster` は使わず、モジュール冒頭
/// 「インタラクティブ部品の扱い」節の方針どおり SSR 静的マークアップのみを
/// 組み立てる）は掲示しない。dismiss/push の実際の dispatch は wasm 層の
/// スコープ外（`crates/headless-ui/src/toast.rs` モジュール doc 参照）。
fn toast_section() -> Node {
    let entries = [
        (
            ToastStatus::Info,
            "Info",
            "新しいバージョンが利用可能です。",
        ),
        (ToastStatus::Success, "Success", "ビルドが完了しました。"),
        (
            ToastStatus::Warning,
            "Warning",
            "依存クレート数が上限に近づいています。",
        ),
        (
            ToastStatus::Error,
            "Error",
            "リンク切れを検出したため書き出しを中止しました。",
        ),
    ];
    let group = toast::group(
        ToastPlacement::BottomEnd,
        "Notifications",
        vec![],
        entries
            .iter()
            .map(|(status, title, description)| {
                toast::root(
                    *status,
                    vec![],
                    vec![
                        toast::title(vec![], vec![text(*title)]),
                        toast::description(vec![], vec![text(*description)]),
                        toast::close_trigger(vec![("aria-label", "Dismiss")], vec![text("×")]),
                    ],
                )
            })
            .collect(),
    );
    section(
        "Toast",
        "headless-ui の Toast（`role=\"status\"` + `aria-live`（`error` のみ `assertive`）+ `aria-atomic=\"true\"`）に pre-styled-ui の placement（`group` slot）/status（`root` slot）variant CSS を適用した静的掲示です。複数通知の有界キュー管理・自動 dismiss のタイマー配線は wasm 層の後続イシューのスコープ外です。",
        vec![group],
    )
}

/// Pagination 節（イシュー #751）: `page_entries()` から ellipsis を含む
/// ページ列を組み立てた静的掲示 + 現在ページ・prev/next の disabled 連動。
/// 状態機械は SSR 静的な現在ページの固定表示のみ（クリック挙動は wasm 層の
/// スコープ外、モジュール冒頭「インタラクティブ部品の扱い」節参照）。
fn pagination_section() -> Node {
    // 総ページ数 20（count=200, page_size=10）、page=10 で両側 ellipsis を
    // 固定掲示する（headless 層のテスト `both_ellipsis` と同じ入力）。
    let p = Pagination::new(200, 10, 1, 1, 10);
    let mut children = vec![p.prev_trigger(ItemMode::Button, vec![], vec![text("Prev")])];
    for entry in p.page_entries() {
        match entry {
            pagination::PageEntry::Page(n) => {
                children.push(p.item(
                    ItemMode::Button,
                    n,
                    false,
                    vec![],
                    vec![text(n.to_string())],
                ));
            }
            pagination::PageEntry::Ellipsis => {
                children.push(pagination::ellipsis(vec![], vec![text("…")]));
            }
        }
    }
    children.push(p.next_trigger(ItemMode::Button, vec![], vec![text("Next")]));

    let demo = pagination::root(
        Size::Md,
        ColorPalette::Accent,
        "pagination",
        vec![],
        children,
    );
    section(
        "Pagination",
        "総件数・ページサイズ・現在ページから省略記号（ellipsis）を含むページ列を決定的に導出する headless Pagination の静的掲示。現在ページは aria-current=\"page\"/data-selected で、端到達は prev/next の disabled で表現します（クリック挙動は wasm 層のスコープ外）。",
        vec![row(vec![demo])],
    )
}

/// Steps 節（イシュー #752）: 3 step 中 2 番目（index=1）を current として
/// 固定表示する静的掲示。indicator は complete/current/incomplete の 3
/// 状態で塗り色を切り替え、separator は `data-complete` の有無で完了色に
/// 変化する（[`crate::steps`] rustdoc §indicator/separator の状態連動色
/// 参照）。current な item の trigger のみ `aria-current="step"` を持つ
/// （クリック挙動は wasm 層のスコープ外、モジュール冒頭「インタラクティブ
/// 部品の扱い」節参照）。
fn steps_section() -> Node {
    let s = Steps::new(3, 1, Orientation::Horizontal);
    let labels = ["Account", "Shipping", "Confirm"];

    let mut items = Vec::new();
    for (index, label) in labels.iter().enumerate() {
        let trigger = steps::trigger(
            &s,
            index,
            vec![],
            vec![
                steps::indicator(&s, index, vec![], vec![text((index + 1).to_string())]),
                text(*label),
            ],
        );
        let mut item_children = vec![trigger];
        if index + 1 < labels.len() {
            item_children.push(steps::separator(&s, index, vec![], vec![]));
        }
        items.push(steps::item(&s, index, vec![], item_children));
    }

    let list = steps::list(&s, vec![], items);
    let content = steps::content(&s, 1, vec![], vec![text("配送先住所を入力してください。")]);
    let nav = div(
        vec![],
        vec![
            steps::prev_trigger(&s, vec![], vec![text("Prev")]),
            steps::next_trigger(&s, vec![], vec![text("Next")]),
        ],
    );

    let demo = steps::root(
        Size::Md,
        ColorPalette::Accent,
        &s,
        vec![],
        vec![list, content, nav],
    );
    section(
        "Steps",
        "count（全 step 数）+ step（現在位置）を持つ headless Steps の静的掲示。item は complete/current/incomplete の 3 状態を持ち、current な item の trigger のみ aria-current=\"step\" を持ちます（クリック挙動は wasm 層のスコープ外）。",
        vec![row(vec![demo])],
    )
}

/// Tour 節（イシュー #841、#735 保留の解除）: 3 step 中 2 番目（index=1、
/// Active { step: 1 }）を現在ステップとして固定表示する静的掲示。
/// spotlight は現在ステップの `target`（`data-target`）を、positioner は
/// `placement`（`data-side`/`data-align`）を反映します。対象要素の実座標
/// 追従・`target` セレクタの実解決は wasm 層の後続イシューのスコープ外
/// （[`crate::showcase`]（本モジュール）ではなく
/// `fandhe_frontend_headless_ui::tour` モジュール doc §スコープ参照）。
fn tour_section() -> Node {
    let steps = vec![
        TourStep {
            id: "welcome".to_string(),
            target: Some("#showcase-tour-target-1".to_string()),
            title: "ようこそ".to_string(),
            description: "このダッシュボードの概要を紹介します。".to_string(),
            placement: Placement::new(Side::Bottom, Align::Center),
        },
        TourStep {
            id: "settings".to_string(),
            target: Some("#showcase-tour-target-2".to_string()),
            title: "設定".to_string(),
            description: "アカウント設定はここから行えます。".to_string(),
            placement: Placement::new(Side::Left, Align::Start),
        },
        TourStep {
            id: "done".to_string(),
            target: None,
            title: "完了".to_string(),
            description: "ツアーはこれで終わりです。".to_string(),
            placement: Placement::new(Side::Top, Align::Center),
        },
    ];
    let t = {
        use fandhe_frontend_pre_styled_ui::fandhe_frontend_interactive::dispatch;
        let mut t = Tour::new(steps);
        dispatch(&mut t, "start", "");
        dispatch(&mut t, "next", "");
        t
    };

    let demo = tour::root(
        ColorPalette::Accent,
        &t,
        vec![],
        vec![
            tour::backdrop(&t, vec![], vec![]),
            tour::spotlight(&t, vec![], vec![]),
            tour::positioner(
                &t,
                vec![],
                vec![
                    tour::arrow(&t, vec![], vec![tour::arrow_tip(&t, vec![], vec![])]),
                    tour::content(
                        &t,
                        TourContentIds {
                            id: Some("showcase-tour-content"),
                            labelledby: Some("showcase-tour-title"),
                            describedby: Some("showcase-tour-desc"),
                        },
                        vec![],
                        vec![
                            tour::title(
                                &t,
                                Some("showcase-tour-title"),
                                vec![],
                                vec![text("設定")],
                            ),
                            tour::description(
                                &t,
                                Some("showcase-tour-desc"),
                                vec![],
                                vec![text("アカウント設定はここから行えます。")],
                            ),
                            tour::progress_text(&t, vec![], vec![text("Step 2 of 3")]),
                            tour::close_trigger(&t, vec![("aria-label", "Close")], vec![text("×")]),
                            tour::action_trigger(&t, vec![], vec![text("Next")]),
                        ],
                    ),
                ],
            ),
        ],
    );
    section(
        "Tour",
        "steps（全ステップ）+ status（idle/active/skipped/completed）を持つ headless Tour の静的掲示（現在 Active { step: 1 } を固定表示）。content は role=\"dialog\" + aria-labelledby/aria-describedby、progress-text は aria-live=\"polite\" を持ちます。対象要素の実座標追従・target セレクタの実解決・クリック/キーボードの実配線は wasm 層の後続イシューのスコープ外です。",
        vec![row(vec![demo])],
    )
}

/// Splitter 節（イシュー #826）: 水平 2 パネルと垂直 3 パネルの静的掲示。
///
/// `panel` の伸縮は headless 中立な
/// [`Splitter::size`](fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::splitter::Splitter::size)
/// から導出する `--fandhe-splitter-size` CSS custom property（flex-basis
/// 経由）の 1 点のみで伝搬する（`fandhe_frontend_pre_styled_ui::splitter`
/// のモジュール doc「動的な値は 1 点のみ」参照）。resize-trigger のクリック・
/// ドラッグ挙動は wasm 層のスコープ外（`crates/headless-ui/src/splitter.rs`
/// モジュール doc §スコープ外参照）。
fn splitter_section() -> Node {
    let horizontal_state = Splitter::new(
        &[
            PanelSpec::new(60.0, 20.0, 80.0),
            PanelSpec::new(40.0, 20.0, 80.0),
        ],
        Orientation::Horizontal,
    );
    let horizontal_demo = splitter::root(
        Size::Md,
        ColorPalette::Accent,
        &horizontal_state,
        false,
        vec![],
        vec![
            splitter::panel(
                &horizontal_state,
                0,
                "showcase-splitter-h-panel-a",
                vec![],
                vec![text("Panel A")],
            ),
            splitter::resize_trigger(
                &horizontal_state,
                0,
                "showcase-splitter-h-panel-a",
                false,
                vec![],
                vec![],
            ),
            splitter::panel(
                &horizontal_state,
                1,
                "showcase-splitter-h-panel-b",
                vec![],
                vec![text("Panel B")],
            ),
        ],
    );

    let vertical_state = Splitter::new(
        &[
            PanelSpec::new(33.0, 0.0, 100.0),
            PanelSpec::new(33.0, 0.0, 100.0),
            PanelSpec::new(34.0, 0.0, 100.0),
        ],
        Orientation::Vertical,
    );
    let mut vertical_children = Vec::new();
    for (index, label) in ["Top", "Middle", "Bottom"].iter().enumerate() {
        let id = format!("showcase-splitter-v-panel-{index}");
        vertical_children.push(splitter::panel(
            &vertical_state,
            index,
            &id,
            vec![],
            vec![text(*label)],
        ));
        if index + 1 < 3 {
            vertical_children.push(splitter::resize_trigger(
                &vertical_state,
                index,
                &id,
                false,
                vec![],
                vec![],
            ));
        }
    }
    let vertical_demo = splitter::root(
        Size::Md,
        ColorPalette::Accent,
        &vertical_state,
        false,
        // column flex の root では各 panel が `flex-basis` にパーセンテージを
        // 使うため、root 自身に解決済みの main size（高さ）がないとパーセン
        // テージが適用されない（Bugbot 指摘、PR #862）。明示的な高さを与えて
        // 33/33/34 分割が実際に反映されるようにする。
        vec![("style", "height: 16rem;")],
        vertical_children,
    );

    section(
        "Splitter",
        "パネルサイズ状態機械 Splitter の静的掲示（水平 2 パネル・垂直 3 パネル）。resize-trigger は role=\"separator\" + aria-valuemin/max/now（先行パネルのサイズ %）+ aria-controls を持ちます（ドラッグ・キーボード操作は wasm 層のスコープ外）。",
        vec![row(vec![horizontal_demo]), row(vec![vertical_demo])],
    )
}

/// DateInput 節: 入力済み / placeholder（未入力） / invalid（実在しない日付
/// 2/30 相当） / disabled / size 各種の静的掲示（イシュー #834）。
///
/// 状態機械 [`fandhe_frontend_pre_styled_ui::date_input`] は headless の
/// [`fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::date_input::DateInput`]
/// をそのまま利用し、SSR 静的マークアップのみを掲示する（ドラッグ・
/// キーボード操作は wasm 層のスコープ外、他コンポーネント節と同型）。
fn date_input_section() -> Node {
    use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::date_input::DateInput;

    let build = |id_prefix: &str, state: &DateInput, size: Size, disabled: bool| {
        date_input::root(
            size,
            disabled,
            state.is_invalid(),
            vec![],
            vec![
                date_input::label(
                    disabled,
                    state.is_invalid(),
                    Some(&format!("{id_prefix}-year")),
                    vec![],
                    vec![text("Date")],
                ),
                date_input::control(
                    disabled,
                    state.is_invalid(),
                    vec![],
                    vec![
                        date_input::segment_group(
                            disabled,
                            state.is_invalid(),
                            vec![],
                            vec![
                                state.segment(DateSegment::Year, disabled, false, vec![]),
                                state.segment(DateSegment::Month, disabled, false, vec![]),
                                state.segment(DateSegment::Day, disabled, false, vec![]),
                            ],
                        ),
                        state.hidden_input(&format!("{id_prefix}-value"), disabled, vec![]),
                    ],
                ),
            ],
        )
    };

    // 入力済み・妥当な日付。
    let filled_state = DateInput::new(Some(2026), Some(7), Some(22), None, None);
    let filled = build("showcase-date-input-filled", &filled_state, Size::Md, false);

    // 未入力（3 セグメントとも placeholder 表示）。
    let empty_state_value = DateInput::default();
    let empty = build(
        "showcase-date-input-empty",
        &empty_state_value,
        Size::Md,
        false,
    );

    // invalid: 2024-02-30 は実在しない日付（fail-closed 検証、モジュール
    // doc「fail-closed な日付検証」参照）。
    let invalid_state = DateInput::new(Some(2024), Some(2), Some(30), None, None);
    let invalid = build(
        "showcase-date-input-invalid",
        &invalid_state,
        Size::Md,
        false,
    );

    // disabled。
    let disabled_state = DateInput::new(Some(2026), Some(1), Some(1), None, None);
    let disabled = build(
        "showcase-date-input-disabled",
        &disabled_state,
        Size::Md,
        true,
    );

    // size 各種（Sm/Md/Lg）。
    let mut size_demos = Vec::new();
    for (size, suffix) in [(Size::Sm, "sm"), (Size::Md, "md"), (Size::Lg, "lg")] {
        let state = DateInput::new(Some(2026), Some(7), Some(22), None, None);
        size_demos.push(build(
            &format!("showcase-date-input-size-{suffix}"),
            &state,
            size,
            false,
        ));
    }

    section(
        "DateInput",
        "年/月/日セグメント入力 DateInput の静的掲示（入力済み・未入力・invalid・disabled・size 各種）。各セグメントは role=\"spinbutton\" + aria-valuemin/max/now（未入力時は valuenow 省略）を持ちます（キーボード操作は wasm 層のスコープ外）。",
        vec![
            row(vec![filled]),
            row(vec![empty]),
            row(vec![invalid]),
            row(vec![disabled]),
            row(size_demos),
        ],
    )
}

/// CheckboxCard 節: unchecked / checked / disabled の 3 態（イシュー #747）。
///
/// chakra-ui checkbox-card 相当のカード型選択 UI。状態機械は
/// [`fandhe_frontend_pre_styled_ui::checkbox`] 節と同じ headless `Checkbox`/
/// `CheckboxProps` を再利用し、`data-scope="checkbox-card"` の新規 anatomy
/// （`crates/pre-styled-ui/src/checkbox_card.rs` 参照）でカード外観を重ねる。
fn checkbox_card_section() -> Node {
    let states = [
        (
            CheckedState::Unchecked,
            false,
            "showcase-checkbox-card-unchecked",
            "Starter",
            "個人利用向けの基本プラン。",
        ),
        (
            CheckedState::Checked,
            false,
            "showcase-checkbox-card-checked",
            "Pro",
            "チームでの共同作業に対応。",
        ),
        (
            CheckedState::Checked,
            true,
            "showcase-checkbox-card-disabled",
            "Enterprise",
            "現在準備中のプランです。",
        ),
    ];
    let demo_row = row(states
        .iter()
        .map(|(checked, disabled, name, label, description)| {
            let props = CheckboxProps {
                checked: *checked,
                disabled: *disabled,
                ..CheckboxProps::default()
            };
            checkbox_card::root(
                Size::Md,
                ColorPalette::Accent,
                &props,
                vec![],
                vec![
                    checkbox_card::hidden_input(&props, name, "on", vec![]),
                    checkbox_card::control(
                        &props,
                        vec![],
                        vec![
                            checkbox_card::indicator(
                                &props,
                                vec![],
                                vec![checkbox_card::indicator_check(&props, vec![], vec![])],
                            ),
                            checkbox_card::content(
                                &props,
                                vec![],
                                vec![
                                    checkbox_card::label(&props, vec![], vec![text(*label)]),
                                    checkbox_card::description(
                                        &props,
                                        vec![],
                                        vec![text(*description)],
                                    ),
                                ],
                            ),
                        ],
                    ),
                ],
            )
        })
        .collect());
    section(
        "CheckboxCard",
        "chakra-ui checkbox-card 相当のカード型選択 UI。状態機械は Checkbox（headless）をそのまま再利用し、data-scope=\"checkbox-card\" の新規 anatomy でカード外観を重ねます。",
        vec![demo_row],
    )
}

/// RadioCard 節: 単一選択のカード型選択 UI（イシュー #747）。
///
/// 状態機械は [`fandhe_frontend_pre_styled_ui::radio_group`] 節と同じ headless
/// `RadioGroup`（`SingleSelect`）をそのまま再利用し、
/// `data-scope="radio-card"` の新規 anatomy（`crates/pre-styled-ui/src/radio_card.rs`
/// 参照）でカード外観を重ねる。
fn radio_card_section() -> Node {
    let label_id = "showcase-radio-card-label";
    let items = [
        (
            "plan-free-card",
            "Free",
            "基本機能のみ利用可能。",
            true,
            false,
        ),
        (
            "plan-pro-card",
            "Pro",
            "チーム機能・優先サポート付き。",
            false,
            false,
        ),
        (
            "plan-enterprise-card",
            "Enterprise",
            "SSO・監査ログに対応。",
            false,
            true,
        ),
    ];
    let mut children = vec![radio_card::label(
        Some(label_id),
        vec![],
        vec![text("Plan")],
    )];
    children.extend(
        items
            .iter()
            .map(|(value, label, description, checked, disabled)| {
                radio_card::item(
                    *checked,
                    *disabled,
                    value,
                    vec![],
                    vec![
                        radio_card::item_hidden_input(
                            *checked,
                            *disabled,
                            Some("showcase-radio-card"),
                            value,
                            vec![],
                        ),
                        radio_card::item_control(
                            *checked,
                            *disabled,
                            vec![],
                            vec![
                                radio_card::item_indicator(*checked, *disabled, vec![]),
                                radio_card::item_content(
                                    vec![],
                                    vec![
                                        radio_card::item_text(vec![], vec![text(*label)]),
                                        radio_card::item_description(
                                            vec![],
                                            vec![text(*description)],
                                        ),
                                    ],
                                ),
                            ],
                        ),
                    ],
                )
            }),
    );
    let demo = radio_card::root(
        Size::Md,
        ColorPalette::Accent,
        false,
        Some(Orientation::Vertical),
        Some(label_id),
        vec![],
        children,
    );
    section(
        "RadioCard",
        "chakra-ui radio-card 相当のカード型選択 UI。状態機械は RadioGroup（headless）をそのまま再利用し、data-scope=\"radio-card\" の新規 anatomy でカード外観を重ねます。",
        vec![demo],
    )
}

/// Breadcrumb 節: `size`/[`BreadcrumbVariant`] を既定値で掲示する（イシュー
/// #755）。状態機械を持たない静的意味論ナビのため、開閉等の状態掲示は不要
/// （3 階層のパンくずをそのまま組み立てる）。
fn breadcrumb_section() -> Node {
    // `href` は空文字列（`fandhe_frontend_core::render` の URL 検証上は
    // 相対 URL として許可されるが、linkcheck 対象からは除外される。
    // `crate::linkcheck::check_links` は空 href を無条件でスキップする
    // 契約であり、生成コンテンツを linkcheck の突合対象へ含めない本モジュール
    // の既存設計（`showcase_markup_has_no_href_attributes_for_linkcheck_neutrality`
    // 参照）を壊さずに `link` パーツ（実際に `href` 属性を持つ要素）を掲示する
    // ための選択。実サイトへの導線が必要な利用は呼び出し側アプリケーションの
    // 責務（本ショーケースは recipe CSS の見た目確認が目的）。
    let items = [
        BreadcrumbItem {
            label: "Docs",
            href: "",
        },
        BreadcrumbItem {
            label: "Components",
            href: "",
        },
        BreadcrumbItem {
            label: "Breadcrumb",
            href: "",
        },
    ];
    let node = breadcrumb::root(
        Size::Md,
        BreadcrumbVariant::Plain,
        None,
        vec![],
        vec![breadcrumb::list(
            vec![],
            items
                .iter()
                .enumerate()
                .flat_map(|(index, entry)| {
                    let inner = if index == items.len() - 1 {
                        breadcrumb::current_link(vec![], vec![text(entry.label)])
                    } else {
                        breadcrumb::link(entry.href, vec![], vec![text(entry.label)])
                    };
                    let mut parts = vec![breadcrumb::item(vec![], vec![inner])];
                    if index != items.len() - 1 {
                        parts.push(breadcrumb::separator(vec![], vec![text("/")]));
                    }
                    parts
                })
                .collect(),
        )],
    );
    section(
        "Breadcrumb",
        "headless-ui の Breadcrumb（nav[aria-label=\"breadcrumb\"] + ol/li）に pre-styled-ui の recipe CSS を適用した静的掲示です。末尾項目のみ aria-current=\"page\"/data-current を持つ非対話の現在位置表示（span）として描画します。",
        vec![node],
    )
}

/// ActionBar 節: 開いた状態の静的マークアップ（イシュー #762）。
///
/// 複数選択時に画面下部へ表示される操作バーの掲示。「2 selected」の選択件数
/// 表示 + 全解除ボタン + separator + close trigger を組み立てる。`positioner`
/// の画面下部固定配置は [`SHOWCASE_LAYOUT_CSS`] でフロー内配置へ中和する
/// （[`dialog_section`]/[`tooltip_section`] と同じ方針。実 overlay 配置は
/// recipe CSS に委ねる）。
fn action_bar_section() -> Node {
    let node = action_bar::root(
        OpenState::Open,
        vec![],
        vec![action_bar::positioner(
            OpenState::Open,
            vec![],
            vec![action_bar::content(
                OpenState::Open,
                "2 selected",
                vec![],
                vec![
                    action_bar::selection_trigger(vec![], vec![text("2 selected")]),
                    action_bar::separator(vec![], vec![]),
                    action_bar::selection_trigger(vec![], vec![text("Delete")]),
                    action_bar::close_trigger(vec![], vec![text("Close")]),
                ],
            )],
        )],
    );
    section(
        "ActionBar",
        "headless-ui の ActionBar（role=\"toolbar\"）に pre-styled-ui の recipe CSS を適用した静的掲示です。positioner はフロー内配置へ中和しています（実際の画面下部固定配置は recipe CSS が担います）。",
        vec![node],
    )
}

/// Toolbar 節（イシュー #991）: root/button/separator/toggle-group/
/// toggle-item/link の 6 anatomy パーツすべてを 1 つのノード木で描画する
/// （Anatomy 節はこのデモから機械導出されるため、6 パーツすべてを網羅する
/// 必要がある。`crates/headless-ui/src/toolbar.rs` モジュール doc 参照）。
/// 押下状態の管理は headless-ui `toggle_group` の状態機械（`ToggleGroup`）を
/// そのまま使い、独自の押下管理を持ち込まない（toolbar モジュール doc
/// 「ToggleGroup / ToggleItem を再エクスポートしない理由」参照）。
fn toolbar_section() -> Node {
    let bar = Toolbar::new(0, 4, false, Orientation::Horizontal);
    let group = toggle_group::ToggleGroup::default();

    let node = bar.root(
        "Text formatting",
        vec![],
        vec![
            bar.button(0, false, vec![], vec![text("Undo")]),
            bar.separator(vec![], vec![]),
            toolbar::toggle_group(
                vec![],
                vec![
                    bar.toggle_item(
                        1,
                        group.is_pressed("bold"),
                        false,
                        "bold",
                        vec![],
                        vec![text("B")],
                    ),
                    bar.toggle_item(
                        2,
                        group.is_pressed("italic"),
                        false,
                        "italic",
                        vec![],
                        vec![text("I")],
                    ),
                ],
            ),
            bar.separator(vec![], vec![]),
            // href は空文字列に固定する（`crate::linkcheck::check_links` が
            // 無条件スキップする値。showcase 掲示コンテンツは実ページへ
            // 解決される href を持たない設計、`breadcrumb_section` と同じ
            // 制約。`showcase_markup_has_no_href_attributes_for_linkcheck_neutrality`
            // 参照）。
            bar.link(3, "", true, vec![], vec![text("Docs")]),
        ],
    );
    section(
        "Toolbar",
        "headless-ui の Toolbar（role=\"toolbar\"）に pre-styled-ui の recipe CSS を適用した静的掲示です。Button / Separator / ToggleGroup（既存の ToggleGroup 状態機械を再利用）/ Link の 6 パーツすべてを 1 つの Toolbar 内に組み合わせています。roving tabindex（focused=0）により先頭の Undo ボタンのみ tabindex=\"0\" です。",
        vec![node],
    )
}

/// Menubar 節（イシュー #992）: root/menu/trigger/positioner/content/item/
/// item-group/item-group-label/separator/sub-trigger/sub-content の 11
/// anatomy パーツすべてを 1 つのノード木で描画する（Anatomy 節はこの
/// デモから機械導出されるため、11 パーツすべてを網羅する必要がある。
/// `crates/headless-ui/src/menubar.rs` モジュール doc 参照）。File Menu を
/// 開いた状態で表示し、その中に「開いている Menu を跨いだ左右移動」の
/// 対象であるサブメニュー（Export）を組み込む。サブメニューの開閉状態は
/// `Menubar` 自身ではなく [`OpenState`] を直接注入する（headless-ui への
/// 直接依存を持たない docs-site の制約上、[`fandhe_frontend_headless_ui::menu::Menu`]
/// は使わず、モジュール doc「`menu` mod 再利用の内訳」が示す「サブメニュー
/// 状態は呼び出し側が別インスタンスとして持つ」設計をここでは
/// `OpenState` 値で直接表現する）。
fn menubar_section() -> Node {
    let bar = Menubar::new(0, 2, Some(0), false, Orientation::Horizontal);
    let export_submenu_state = OpenState::Closed;

    let node = bar.root(
        "App menu",
        vec![],
        vec![
            bar.menu(
                0,
                vec![],
                vec![
                    bar.trigger(
                        0,
                        false,
                        false,
                        Some("menubar-file-content"),
                        vec![],
                        vec![text("File")],
                    ),
                    bar.positioner(
                        0,
                        vec![],
                        vec![bar.content(
                            0,
                            Some("menubar-file-content"),
                            None,
                            vec![],
                            vec![
                                menubar::item_group(
                                    Some("menubar-recent-label"),
                                    vec![],
                                    vec![
                                        menubar::item_group_label(
                                            Some("menubar-recent-label"),
                                            vec![],
                                            vec![text("Recent")],
                                        ),
                                        menubar::item(
                                            "report.md",
                                            false,
                                            true,
                                            vec![],
                                            vec![text("report.md")],
                                        ),
                                    ],
                                ),
                                menubar::separator(vec![], vec![]),
                                menubar::sub_trigger(
                                    export_submenu_state,
                                    false,
                                    false,
                                    Some("menubar-export-sub-content"),
                                    vec![],
                                    vec![text("Export")],
                                ),
                                menubar::sub_content(
                                    export_submenu_state,
                                    Some("menubar-export-sub-content"),
                                    None,
                                    vec![],
                                    vec![menubar::item(
                                        "pdf",
                                        false,
                                        false,
                                        vec![],
                                        vec![text("PDF")],
                                    )],
                                ),
                            ],
                        )],
                    ),
                ],
            ),
            bar.menu(
                1,
                vec![],
                vec![bar.trigger(1, false, false, None, vec![], vec![text("Edit")])],
            ),
        ],
    );
    section(
        "Menubar",
        "headless-ui の Menubar（role=\"menubar\"）に pre-styled-ui の recipe CSS を適用した静的掲示です。File / Edit の 2 Menu を水平配置し、File Menu を開いた状態（open=Some(0)）で表示しています。Item Group（Recent）・Separator・SubTrigger/SubContent（Export → PDF）の入れ子構造も含みます。roving tabindex（focused=0）により先頭の File トリガーのみ tabindex=\"0\" です。",
        vec![node],
    )
}

/// Status 節（イシュー #765）: colorPalette 軸ごとのドット + ラベル表示。
fn status_section() -> Node {
    let palette_row = row(palettes()
        .iter()
        .map(|(palette, label)| {
            status::root(
                &StatusProps {
                    palette: *palette,
                    ..StatusProps::default()
                },
                vec![],
                vec![status::indicator(vec![]), text(*label)],
            )
        })
        .collect());
    section(
        "Status",
        "ドット（indicator）+ ラベルで状態を示す静的表示。colorPalette で色を切り替えます。",
        vec![palette_row],
    )
}

/// EmptyState 節（イシュー #765）: indicator/title/description/actions の
/// 構成例。`actions` 内は `button` を使い `href` を持たせない
/// （`showcase_markup_has_no_href_attributes_for_linkcheck_neutrality` の
/// linkcheck 中立性を維持する）。
fn empty_state_section() -> Node {
    let node = empty_state::root(
        &EmptyStateProps::default(),
        vec![],
        vec![empty_state::content(
            vec![],
            vec![
                empty_state::indicator(vec![], vec![text("∅")]),
                empty_state::title(vec![], vec![text("No results found")]),
                empty_state::description(
                    vec![],
                    vec![text(
                        "Try adjusting your search or filter to find what you are looking for.",
                    )],
                ),
                empty_state::actions(
                    vec![],
                    vec![button(
                        &ButtonProps::default(),
                        vec![],
                        vec![text("Clear filters")],
                    )],
                ),
            ],
        )],
    );
    section(
        "EmptyState",
        "indicator / title / description / actions で構成する空状態レイアウト。colorPalette 軸は持たない中立コンテナです。",
        vec![node],
    )
}

/// VisuallyHidden 節（イシュー #776）: アイコンのみのボタンに、視覚的には
/// 隠すがスクリーンリーダーには読ませる補足テキストを添えるパターンを掲示
/// する（chakra-ui/ark-ui の典型的な用例と同じ構成）。
///
/// SkipNav（同イシュー）はページ骨格（`crate::layout::docs_page_with_assets`）
/// へ全ページ共通で 1 個だけ実適用する構成のため、既に本ページの `<body>`
/// 先頭にも SkipNav リンクが存在する。ショーケース節として別 id のデモを
/// 追加すると `id="fandhe-skip-nav"` の重複や紛らわしさを招くため、
/// SkipNav 自体のショーケースデモは設けない（実装計画 §3 が明示的に許容する
/// 判断: 「デモ省略しレイアウト実適用を正とする」）。
fn visually_hidden_section() -> Node {
    // 「★」自体は装飾（アイコン）であり、ボタンのアクセシブルネームは
    // 後続の `visually_hidden::root` テキストのみに担わせる（`aria-label` を
    // 併用すると accessible-name 計算で `aria-label` が勝ち、VisuallyHidden
    // テキストが読み上げられなくなってしまう。アイコンのみのボタンに
    // 補足テキストを添える本来の用途を壊さないための必須の組み合わせ方）。
    // イシュー #830 の `icon_button()`（`aria-label` を必須引数化）へは
    // 意図的に移行しない: このデモの本旨は「`aria-label` を使わずアクセシブル
    // ネームを子孫の VisuallyHidden テキストへ委ねる」パターンの掲示であり、
    // `icon_button()` へ切り替えると強制的に `aria-label` が付与されて
    // このデモが成立しなくなる（`button_section` 側の別デモで `icon_button`/
    // `close_button` を掲示済み）。
    let visually_hidden_icon_button = button(
        &ButtonProps::default(),
        vec![],
        vec![
            el("span", vec![("aria-hidden", "true")], vec![text("★")]),
            visually_hidden::root(vec![], vec![text("お気に入りに追加")]),
        ],
    );
    section(
        "VisuallyHidden",
        "視覚的には隠す（clip 手法）が支援技術には読ませ続けるテキストコンテナ。アイコンのみのボタンに補足テキストを添える用途などに使います。aria-hidden は一切出力しません。",
        vec![row(vec![visually_hidden_icon_button])],
    )
}

/// Progress（circle 対応、イシュー #763）節: determinate（40%）の size
/// バリエーション・complete・indeterminate の 3 状態を掲示する。
///
/// `Progress` は headless の値状態機械（`fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::progress::Progress`）
/// を直接 import して構築する（`progress::root` は `size` variant クラス
/// 付与のみを担う薄いラッパーであり、状態は呼び出し側が headless 型で持つ
/// 契約、`crates/pre-styled-ui/src/progress.rs` rustdoc 参照）。circle 系
/// パーツ（Circle/CircleTrack/CircleRange）は styled 層の独自ラッパーを持たず
/// headless の inherent メソッドをそのまま呼ぶ。
fn progress_section() -> Node {
    use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::progress::Progress;
    use fandhe_frontend_pre_styled_ui::progress;

    fn circle_demo(p: &Progress, size: Size, aria_valuetext: Option<&str>) -> Node {
        progress::root(
            p,
            size,
            aria_valuetext,
            vec![],
            vec![p.circle(
                vec![],
                vec![
                    p.circle_track(vec![], vec![]),
                    p.circle_range(vec![], vec![]),
                ],
            )],
        )
    }

    let determinate = Progress::new(0.0, 100.0, Some(40.0), Orientation::Horizontal);
    let size_row = row(vec![
        circle_demo(&determinate, Size::Sm, Some("40%")),
        circle_demo(&determinate, Size::Md, Some("40%")),
        circle_demo(&determinate, Size::Lg, Some("40%")),
    ]);

    let complete = Progress::new(0.0, 100.0, Some(100.0), Orientation::Horizontal);
    let complete_row = row(vec![circle_demo(&complete, Size::Md, Some("100%"))]);

    let indeterminate = Progress::new(0.0, 100.0, None, Orientation::Horizontal);
    let indeterminate_row = row(vec![circle_demo(&indeterminate, Size::Md, None)]);

    section(
        "Progress",
        "Circular（SVG）表示の進捗インジケータ。size（sm/md/lg）で --fandhe-progress-size/--fandhe-progress-thickness を切り替えます。indeterminate（不定進捗）は data-state=\"indeterminate\" に連動した回転アニメーションで表示します。",
        vec![size_row, complete_row, indeterminate_row],
    )
}

/// QrCode（イシュー #774）節: size（sm/md/lg）3 態・overlay（ロゴ想定の中央
/// コンテンツ）付きの 1 態を掲示する。エンコード対象は固定の URL 文字列
/// （`fandhe_frontend_pre_styled_ui::qr_code::encode` は外部依存ゼロの
/// QR Model 2 byte モードエンコーダ、`crates/headless-ui/src/qr_code.rs`
/// 参照）。
fn qr_code_section() -> Node {
    let matrix = qr_code::encode(
        "https://fandhe-frontend.example/",
        qr_code::ErrorCorrectionLevel::M,
    )
    .expect("ショーケース固定 URL はバージョン 40 容量内に収まる");

    let demo = |size: Size| {
        qr_code::root(
            size,
            vec![],
            vec![qr_code::frame(
                &matrix,
                qr_code::DEFAULT_QUIET_ZONE,
                Some("QR code linking to https://fandhe-frontend.example/"),
                vec![],
                vec![qr_code::pattern(
                    &matrix,
                    qr_code::DEFAULT_QUIET_ZONE,
                    vec![],
                )],
            )],
        )
    };

    let size_row = row(vec![demo(Size::Sm), demo(Size::Md), demo(Size::Lg)]);

    let with_overlay = qr_code::root(
        Size::Lg,
        vec![],
        vec![
            qr_code::frame(
                &matrix,
                qr_code::DEFAULT_QUIET_ZONE,
                Some("QR code linking to https://fandhe-frontend.example/"),
                vec![],
                vec![qr_code::pattern(
                    &matrix,
                    qr_code::DEFAULT_QUIET_ZONE,
                    vec![],
                )],
            ),
            qr_code::overlay(vec![], vec![text("FW")]),
        ],
    );
    let overlay_row = row(vec![with_overlay]);

    section(
        "QrCode",
        "外部依存ゼロの QR Model 2（ISO/IEC 18004）byte モードエンコーダによる QR コード表示。size（sm/md/lg）で --fandhe-qr-code-size を切り替えます。Overlay パーツはロゴ等の呼び出し側コンテンツを中央に重ねる用途です。",
        vec![size_row, overlay_row],
    )
}

/// Image 節（イシュー #770）の demo `src`。実画像を同梱せず、外部フェッチ・
/// 404 を発生させないインライン SVG data URI を使う（相対パスではなく
/// [`AVATAR_INLINE_SVG_SRC`] と同じくパーセントエンコード済み data URI と
/// することで、実在しないファイルパスによる 404 を防ぐ。矩形プレースホル
/// ダー柄のアイコン）。
const IMAGE_DEMO_SRC: &str =
    "data:image/svg+xml,%3Csvg%20xmlns%3D%27http%3A%2F%2Fwww.w3.org%2F2000%2Fsvg%27%20viewBox%3D%270%200%2064%2064%27%3E%3Crect%20width%3D%2764%27%20height%3D%2764%27%20fill%3D%27%234a90d9%27%2F%3E%3C%2Fsvg%3E";

/// Image 節: `fit`（object-fit）× `aspect_ratio` の 2 軸。
fn image_section() -> Node {
    let fits = [
        (ImageFit::Cover, "Cover"),
        (ImageFit::Contain, "Contain"),
        (ImageFit::Fill, "Fill"),
        (ImageFit::ScaleDown, "ScaleDown"),
        (ImageFit::NoFit, "NoFit (none)"),
    ];
    let fit_row = row(fits
        .iter()
        .map(|(fit, label)| {
            image(
                &ImageProps {
                    fit: *fit,
                    ..ImageProps::new(IMAGE_DEMO_SRC, label)
                },
                vec![(
                    "style",
                    "width: 6rem; height: 4rem; background: var(--fandhe-color-bg-subtle);",
                )],
            )
        })
        .collect());

    let ratios = [
        (AspectRatio::Auto, "Auto"),
        (AspectRatio::Square, "Square"),
        (AspectRatio::Video, "Video"),
    ];
    let ratio_row = row(ratios
        .iter()
        .map(|(ratio, label)| {
            image(
                &ImageProps {
                    aspect_ratio: *ratio,
                    ..ImageProps::new(IMAGE_DEMO_SRC, label)
                },
                vec![(
                    "style",
                    "width: 6rem; background: var(--fandhe-color-bg-subtle);",
                )],
            )
        })
        .collect());

    section(
        "Image",
        "写真等の静的コンテンツを表示する img の styled ラッパー。fit（object-fit）と aspect-ratio を型安全な props で切り替えます。状態機械は持たず、avatar の ImageStatus とは独立です。",
        vec![fit_row, ratio_row],
    )
}

/// Icon 節: `size` variant のみ。SVG 本体は呼び出し側がノード木 API
/// （`el(\"path\", ...)`）で構築する（本モジュールは外部リソースを参照しない）。
fn icon_section() -> Node {
    let star_path = || {
        el(
            "path",
            vec![(
                "d",
                "M12 2l2.9 6.9 7.1.6-5.4 4.6 1.6 7-6.2-3.9-6.2 3.9 1.6-7-5.4-4.6 7.1-.6z",
            )],
            vec![],
        )
    };

    let size_row = row(vec![Size::Sm, Size::Md, Size::Lg]
        .into_iter()
        .map(|size| {
            icon(
                &IconProps {
                    size,
                    label: Some("Star"),
                    ..IconProps::default()
                },
                vec![],
                vec![star_path()],
            )
        })
        .collect());

    section(
        "Icon",
        "インライン SVG の寸法（size）・配色（color: currentColor 継承）を統一する svg ラッパー。SVG 本体（path 等）は呼び出し側がノード木 API で構築します。",
        vec![size_row],
    )
}

/// Table 節: variant（line/outline）・size（sm/md/lg）・striped の各軸。
fn table_section() -> Node {
    fn sample_table(props: TableProps) -> Node {
        table::root(
            props,
            vec![],
            vec![
                table::caption(vec![], vec![text("登録済みユーザー一覧")]),
                table::header(
                    vec![],
                    vec![table::row(
                        vec![],
                        vec![
                            table::column_header(vec![], vec![text("Name")]),
                            table::column_header(vec![], vec![text("Email")]),
                            table::column_header(vec![], vec![text("Role")]),
                        ],
                    )],
                ),
                table::body(
                    vec![],
                    vec![
                        table::row(
                            vec![],
                            vec![
                                table::cell(vec![], vec![text("Alice")]),
                                table::cell(vec![], vec![text("alice@example.com")]),
                                table::cell(vec![], vec![text("Admin")]),
                            ],
                        ),
                        table::row(
                            vec![],
                            vec![
                                table::cell(vec![], vec![text("Bob")]),
                                table::cell(vec![], vec![text("bob@example.com")]),
                                table::cell(vec![], vec![text("Member")]),
                            ],
                        ),
                        table::row(
                            vec![],
                            vec![
                                table::cell(vec![], vec![text("Carol")]),
                                table::cell(vec![], vec![text("carol@example.com")]),
                                table::cell(vec![], vec![text("Member")]),
                            ],
                        ),
                    ],
                ),
            ],
        )
    }

    let variant_demo = stack(vec![
        sample_table(TableProps {
            variant: TableVariant::Line,
            ..TableProps::default()
        }),
        sample_table(TableProps {
            variant: TableVariant::Outline,
            ..TableProps::default()
        }),
    ]);
    let size_demo = stack(vec![
        sample_table(TableProps {
            size: Size::Sm,
            ..TableProps::default()
        }),
        sample_table(TableProps {
            size: Size::Lg,
            ..TableProps::default()
        }),
    ]);
    let striped_demo = stack(vec![sample_table(TableProps {
        striped: true,
        ..TableProps::default()
    })]);

    section(
        "Table",
        "table/thead/tbody/tfoot/tr/th/td/caption の HTML 意味論を尊重した表組み。variant（line / outline）・size（sm / md / lg）・striped の 3 軸 variant を持ちます。",
        vec![variant_demo, size_demo, striped_demo],
    )
}

/// DataList 節: orientation（vertical/horizontal）の軸。
fn data_list_section() -> Node {
    fn sample_data_list(orientation: DataListOrientation) -> Node {
        data_list::root(
            DataListProps { orientation },
            vec![],
            vec![
                data_list::item(
                    vec![],
                    vec![
                        data_list::item_label(vec![], vec![text("Name")]),
                        data_list::item_value(vec![], vec![text("Alice")]),
                    ],
                ),
                data_list::item(
                    vec![],
                    vec![
                        data_list::item_label(vec![], vec![text("Email")]),
                        data_list::item_value(vec![], vec![text("alice@example.com")]),
                    ],
                ),
            ],
        )
    }

    let demos = stack(vec![
        sample_data_list(DataListOrientation::Vertical),
        sample_data_list(DataListOrientation::Horizontal),
    ]);
    section(
        "DataList",
        "dl/dt/dd の定義リスト意味論を尊重したラベル・値の一覧表示。orientation（vertical / horizontal）の 1 軸 variant を持ちます。",
        vec![demos],
    )
}

/// Stat 節: 状態機械不要の静的部品（イシュー #769）。`size` variant のみ。
fn stat_section() -> Node {
    let demo = row(vec![
        stat::root(
            Size::Md,
            vec![],
            vec![
                stat::label(vec![], vec![text("Revenue")]),
                stat::value_text(
                    vec![],
                    vec![text("1,234"), stat::value_unit(vec![], vec![text("USD")])],
                ),
                stat::help_text(
                    vec![],
                    vec![stat::up_indicator(vec![]), text("12% vs 先月")],
                ),
            ],
        ),
        stat::root(
            Size::Md,
            vec![],
            vec![
                stat::label(vec![], vec![text("Churn")]),
                stat::value_text(vec![], vec![text("4.2%")]),
                stat::help_text(
                    vec![],
                    vec![stat::down_indicator(vec![]), text("0.8% vs 先月")],
                ),
            ],
        ),
    ]);
    section(
        "Stat",
        "数値指標 1 件をラベル・値・補助テキスト・増減方向インジケーターの組で表示する静的部品です。size（sm/md/lg）で value-text のフォントサイズを切り替えます。",
        vec![demo],
    )
}

/// Timeline 節: 状態機械不要の静的部品（イシュー #769）。`variant`/`size`/
/// `color-palette` の 3 軸。
fn timeline_section() -> Node {
    let demo = timeline::root(
        TimelineVariant::Solid,
        Size::Md,
        ColorPalette::Accent,
        vec![],
        vec![
            timeline::item(
                vec![],
                vec![
                    timeline::connector(
                        vec![],
                        vec![
                            timeline::indicator(vec![], vec![]),
                            timeline::separator(vec![], vec![]),
                        ],
                    ),
                    timeline::content(
                        vec![],
                        vec![
                            timeline::title(vec![], vec![text("プロジェクト開始")]),
                            timeline::description(vec![], vec![text("2026-01-01")]),
                        ],
                    ),
                ],
            ),
            timeline::item(
                vec![],
                vec![
                    // 最終 item は separator を組み込まないことで非表示にする
                    // 契約（`crate::timeline` rustdoc 参照）。
                    timeline::connector(vec![], vec![timeline::indicator(vec![], vec![])]),
                    timeline::content(
                        vec![],
                        vec![timeline::title(vec![], vec![text("リリース")])],
                    ),
                ],
            ),
        ],
    );
    section(
        "Timeline",
        "時系列に並ぶ出来事の一覧を connector（縦線）+ indicator（点）+ content で表示する静的部品です。variant（solid/subtle/outline/plain）で indicator の塗り方を切り替えます。",
        vec![demo],
    )
}

/// Marquee 節（イシュー #831、`docs/policy/intentional-non-adoption.md` §3.24
/// の再導入）: CSS のみ（JS ゼロ）の自動流動テキスト。`direction`（既定/`End`）
/// の切り替え・装飾用途（`decorative: true`）・`--fandhe-marquee-duration`
/// 上書きの掲示例を並べる。
fn marquee_section() -> Node {
    let default_demo = marquee::marquee(
        &MarqueeProps::default(),
        vec![],
        vec![marquee::item(
            vec![],
            vec![text(
                "Fandhe frontend — CSS のみで動く自動流動テキストです。",
            )],
        )],
    );
    let end_demo = marquee::marquee(
        &MarqueeProps {
            direction: MarqueeDirection::End,
            ..MarqueeProps::default()
        },
        vec![],
        vec![marquee::item(
            vec![],
            vec![text("逆方向スクロールの例です。")],
        )],
    );
    let decorative_demo = marquee::marquee(
        &MarqueeProps {
            decorative: true,
            ..MarqueeProps::default()
        },
        vec![("style", "--fandhe-marquee-duration: 8s;")],
        vec![marquee::item(
            vec![],
            vec![text("装飾用途（aria-hidden）+ 速度上書きの例です。")],
        )],
    );
    section(
        "Marquee",
        "CSS のみ（JS ゼロ）の自動流動テキストです。direction（既定/end）でスクロール方向を切り替え、hover/focus-within で常時一時停止、prefers-reduced-motion: reduce 環境では停止します。decorative: true で装飾用途（aria-hidden）に、--fandhe-marquee-duration の上書きで速度を調整できます。",
        vec![default_demo, end_demo, decorative_demo],
    )
}

/// ScrollArea 節（イシュー #825）: `overflow: auto` によるネイティブスクロール
/// とカスタムスクロールバー表現（`scrollbar-width`/`::-webkit-scrollbar`）。
/// JS によるスクロール位置追従は本イシューのスコープ外（`crate::scroll_area`
/// rustdoc 参照）のため、固定高の viewport と長文 content のみを掲示する。
fn scroll_area_section() -> Node {
    let items: Vec<Node> = (1..=20)
        .map(|i| el("p", vec![], vec![text(format!("スクロール可能な行 {i}"))]))
        .collect();
    let demo = scroll_area::root(
        vec![(
            "style",
            "height: 8rem; width: 16rem; border: 1px solid var(--fandhe-color-border);",
        )],
        vec![scroll_area::viewport(
            vec![],
            vec![scroll_area::content(vec![], items)],
        )],
    );
    section(
        "ScrollArea",
        "CSS overflow を主体としたスクロール領域です。カスタムスクロールバーの見た目は scrollbar-width/scrollbar-color と ::-webkit-scrollbar 系規則で表現します（JS によるスクロール位置追従は対象外）。",
        vec![demo],
    )
}

/// （モジュール冒頭の rustdoc 方針と同じ）。
fn calendar_section() -> Node {
    let today = PlainDate::new(2026, 7, 22).unwrap();
    let selected = PlainDate::new(2026, 7, 15).unwrap();
    let weekday_labels = ["Mo", "Tu", "We", "Th", "Fr", "Sa", "Su"];
    let header_row = calendar::table_row(
        vec![],
        weekday_labels
            .iter()
            .map(|label| calendar::table_head_cell(vec![], vec![text(*label)]))
            .collect(),
    );

    // 2026-07-01 は水曜日。月曜始まりの週配列を SSR 静的表示として手組みする
    // （`fandhe_frontend_headless_ui::date::month_grid` 相当の決定的レイアウト。
    // 本ショーケースは headless-ui へ直接依存しない方針のため、`Calendar`
    // 状態機械は使わず日付のみを列挙する）。
    let first_of_month = PlainDate::new(2026, 7, 1).unwrap();
    let grid_start = first_of_month.add_days(-2).unwrap(); // 2026-06-29 (Mon)
    let body_rows: Vec<Node> = (0..5)
        .map(|week| {
            let cells: Vec<Node> = (0..7)
                .map(|day| {
                    let date = grid_start.add_days(week * 7 + day).unwrap();
                    let is_selected = date == selected;
                    let is_today = date == today;
                    let is_outside = date.month() != 7 || date.year() != 2026;
                    calendar::table_cell(
                        is_selected,
                        vec![],
                        vec![calendar::day_trigger(
                            date,
                            is_selected,
                            is_today,
                            is_outside,
                            false,
                            None,
                            vec![],
                            vec![text(date.day().to_string())],
                        )],
                    )
                })
                .collect();
            calendar::table_row(vec![], cells)
        })
        .collect();

    let node = calendar::root(
        Size::Md,
        vec![],
        vec![
            calendar::heading(
                Some("showcase-calendar-heading"),
                vec![],
                vec![text("July 2026")],
            ),
            calendar::prev_trigger(false, vec![], vec![text("‹")]),
            calendar::next_trigger(false, vec![], vec![text("›")]),
            calendar::table(
                Some("showcase-calendar-heading"),
                vec![],
                vec![
                    calendar::table_header(vec![], vec![header_row]),
                    calendar::table_body(vec![], body_rows),
                ],
            ),
        ],
    );
    section(
        "Calendar",
        "headless-ui の Calendar（role=\"grid\"）に pre-styled-ui の recipe CSS を適用した静的掲示です。2026-07 を週開始 Monday で表示し、今日（07-22）・選択日（07-15）・表示月外セルの見た目を固定表示しています。キーボードナビゲーション・クリック挙動は wasm 層のスコープ外です。",
        vec![node],
    )
}

/// DatePicker 節（イシュー #835）: popover が開いた状態で [`calendar_section`]
/// と同じ月グリッドを内包した静的掲示。positioner はフロー内配置へ中和して
/// います。
fn date_picker_section() -> Node {
    let today = PlainDate::new(2026, 7, 22).unwrap();
    let selected = PlainDate::new(2026, 7, 15).unwrap();
    let weekday_labels = ["Mo", "Tu", "We", "Th", "Fr", "Sa", "Su"];
    let header_row = calendar::table_row(
        vec![],
        weekday_labels
            .iter()
            .map(|label| calendar::table_head_cell(vec![], vec![text(*label)]))
            .collect(),
    );
    let first_of_month = PlainDate::new(2026, 7, 1).unwrap();
    let grid_start = first_of_month.add_days(-2).unwrap();
    let body_rows: Vec<Node> = (0..5)
        .map(|week| {
            let cells: Vec<Node> = (0..7)
                .map(|day| {
                    let date = grid_start.add_days(week * 7 + day).unwrap();
                    let is_selected = date == selected;
                    let is_today = date == today;
                    let is_outside = date.month() != 7 || date.year() != 2026;
                    calendar::table_cell(
                        is_selected,
                        vec![],
                        vec![calendar::day_trigger(
                            date,
                            is_selected,
                            is_today,
                            is_outside,
                            false,
                            None,
                            vec![],
                            vec![text(date.day().to_string())],
                        )],
                    )
                })
                .collect();
            calendar::table_row(vec![], cells)
        })
        .collect();

    let node = date_picker::root(
        Size::Md,
        OpenState::Open,
        vec![],
        vec![
            date_picker::label(
                Some("showcase-date-picker-label"),
                vec![],
                vec![text("Delivery date")],
            ),
            date_picker::control(
                OpenState::Open,
                vec![],
                vec![
                    date_picker::input(Some("2026-07-15"), false, None, vec![]),
                    date_picker::trigger(
                        OpenState::Open,
                        false,
                        Some("showcase-date-picker-content"),
                        vec![],
                        vec![text("📅")],
                    ),
                ],
            ),
            date_picker::positioner(
                OpenState::Open,
                vec![],
                vec![date_picker::content(
                    OpenState::Open,
                    Some("showcase-date-picker-content"),
                    Some("showcase-date-picker-label"),
                    vec![],
                    vec![calendar::table(
                        None,
                        vec![],
                        vec![
                            calendar::table_header(vec![], vec![header_row]),
                            calendar::table_body(vec![], body_rows),
                        ],
                    )],
                )],
            ),
        ],
    );
    section(
        "DatePicker",
        "headless-ui の DatePicker（popover 基盤 + Calendar 合成）に pre-styled-ui の recipe CSS を適用した静的掲示です。popover が開いた状態を固定表示し、内部に Calendar の月グリッドを合成しています。positioner はフロー内配置へ中和しています。",
        vec![node],
    )
}

/// Timer 節（イシュー #836）: countdown 型 Timer の running 状態を固定表示する
/// SSR 静的掲示。
///
/// [`tree_view_section`]/JsonTreeView 節と同じ方針で、SSR 本来の初期状態
/// （Idle・経過ゼロ）ではなく意図的に dispatch で「開始してしばらく経過した
/// running 状態」を作って固定掲示する（見た目を実演する目的、実際のクリック
/// 挙動・tick 駆動は wasm 層のスコープ外、モジュール冒頭「インタラクティブ
/// 部品の扱い」節参照）。
fn timer_section() -> Node {
    use fandhe_frontend_pre_styled_ui::fandhe_frontend_interactive::dispatch;

    // 90 秒カウントダウン、1 秒 tick。35 秒経過（残り 55 秒）まで進めた状態を
    // 固定掲示する。
    let mut t = Timer::countdown(90_000, 1_000);
    dispatch(&mut t, "timer:start", "");
    dispatch(&mut t, "timer:tick", "35000");

    let (_, _, minutes, seconds) = t.display_segments();
    let node = t.root(
        vec![],
        vec![
            timer::area(
                vec![],
                vec![
                    timer::item(
                        TimerUnit::Minutes,
                        vec![],
                        vec![
                            timer::item_value(
                                TimerUnit::Minutes,
                                vec![],
                                vec![text(timer::format_segment(minutes))],
                            ),
                            timer::item_label(TimerUnit::Minutes, vec![], vec![text("Min")]),
                        ],
                    ),
                    timer::separator(vec![], vec![text(":")]),
                    timer::item(
                        TimerUnit::Seconds,
                        vec![],
                        vec![
                            timer::item_value(
                                TimerUnit::Seconds,
                                vec![],
                                vec![text(timer::format_segment(seconds))],
                            ),
                            timer::item_label(TimerUnit::Seconds, vec![], vec![text("Sec")]),
                        ],
                    ),
                ],
            ),
            timer::control(
                vec![],
                vec![
                    timer::action_trigger(TimerControl::Pause, vec![], vec![text("Pause")]),
                    timer::action_trigger(TimerControl::Reset, vec![], vec![text("Reset")]),
                ],
            ),
        ],
    );
    section(
        "Timer",
        "headless-ui の Timer（tick 注入型・idle/running/paused/completed の決定的状態機械）に pre-styled-ui のセグメント表示（分:秒）CSS を適用した静的掲示です。90 秒のカウントダウンを開始して 35 秒経過した running 状態を固定表示しています。実 tick 駆動（setInterval）は fandhe-frontend-wasm-full::headless_timer のスコープです。",
        vec![node],
    )
}

/// charts 用の固定サンプルデータ（BarChart/BarList/BarSegment 節が共有、
/// イシュー #849）。
fn bar_charts_sample_data() -> ChartData {
    ChartData::new(
        vec![
            "Jan".to_string(),
            "Feb".to_string(),
            "Mar".to_string(),
            "Apr".to_string(),
        ],
        vec![
            Series::new("visits", vec![120.0, 200.0, 150.0, 80.0]),
            Series::new("signups", vec![20.0, 35.0, 28.0, 12.0]),
        ],
    )
    .expect("ショーケース固定サンプルはカテゴリ数・系列長が一致する")
}

/// Charts 節（イシュー #847）: 軸（Y 軸 + X 軸カテゴリ）・CartesianGrid・
/// データ点（`charts::tooltip::datum`、hover でネイティブ `<title>` 表示 +
/// `:hover` 強調）・凡例を合成した最小の折れ線チャート様デモ。
///
/// インタラクティブなチャート部品（Area/Bar/Line/Pie）は #848〜#851 の
/// スコープであり、本節は軸・グリッド・凡例・ツールチップの単体掲示に
/// 留める（データ点は `<circle>` のみで、系列間を結ぶ `<path>` は
/// 描画しない）。
fn charts_section() -> Node {
    let data = ChartData::new(
        vec![
            "Jan".to_string(),
            "Feb".to_string(),
            "Mar".to_string(),
            "Apr".to_string(),
        ],
        vec![
            Series::new("Visits", vec![120.0, 180.0, 150.0, 220.0]),
            Series::new("Signups", vec![20.0, 35.0, 28.0, 40.0]),
        ],
    )
    .expect("showcase 固定データは不変条件を満たす");

    // プロット領域: 全体 320x220 のうち左 40px（Y 軸ラベル）・下 30px
    // （X 軸ラベル）・上下左右の余白 10px を除いた範囲。
    let (plot_left, plot_right) = (40.0, 310.0);
    let (plot_top, plot_bottom) = (10.0, 170.0);

    let (min, max) = data.domain();
    // SVG は y が下向き正のため range を反転し、データの大小を視覚的な上下へ対応させる。
    let y_scale = LinearScale::new((min, max), (plot_bottom, plot_top))
        .expect("domain() は非退化な値域を返す")
        .nice();
    let y_ticks = y_scale.ticks(4).expect("target=4 は許容範囲 1..=50 内");
    // cartesian_grid の y_positions はスケール済みピクセル座標を期待する
    // （y_axis が内部で y_scale を通すのと同じ変換）。y_ticks（ドメイン値）
    // をそのまま渡すとグリッド線が Y 軸目盛り・データ点とずれるため、
    // ここで y_scale を適用してから渡す。
    let y_tick_positions: Vec<f64> = y_ticks.iter().map(|&tick| y_scale.scale(tick)).collect();

    let category_count = data.categories().len() as f64;
    let band_width = (plot_right - plot_left) / category_count;

    let mut svg_children = vec![
        grid::cartesian_grid(
            (plot_left, plot_right),
            (plot_top, plot_bottom),
            &[],
            &y_tick_positions,
            &GridProps::default(),
        )
        .expect("有限な range/ticks のみを渡す"),
        axis::y_axis(&y_scale, &y_ticks, plot_left, &AxisProps::default())
            .expect("有限な ticks のみを渡す"),
        axis::x_axis_categories(
            (plot_left, plot_right),
            data.categories(),
            plot_bottom,
            &AxisProps::default(),
        )
        .expect("categories は非空・range は有限"),
    ];

    for (series_index, series) in data.series().iter().enumerate() {
        let color = fandhe_frontend_pre_styled_ui::charts::series_color_var(series_index);
        for (category_index, &value) in series.values.iter().enumerate() {
            let cx = plot_left + (category_index as f64 + 0.5) * band_width;
            let cy = y_scale.scale(value);
            let label =
                chart_tooltip::datum_label(&data.categories()[category_index], &series.name, value);
            svg_children.push(chart_tooltip::datum(
                cx,
                cy,
                4.0,
                &label,
                vec![("fill", &color)],
            ));
        }
    }

    let view_box = ViewBox::new(0.0, 0.0, 320.0, 220.0).expect("固定寸法は正の有限値");
    let chart = svg_root(
        &view_box,
        vec![("aria-label", "Visits and signups by month")],
        svg_children,
    );

    let legend_node = legend::legend(
        &data,
        &LegendProps {
            title: Some("Series".to_string()),
        },
    );

    section(
        "Charts",
        "軸（Axes）・CartesianGrid・凡例（Legend）・ツールチップ（Tooltip）を合成した最小デモです。データ点はホバーするとブラウザネイティブの `<title>` によるツールチップと `:hover` 強調が表示されます（JS 不要）。系列を結ぶ折れ線・棒等の描画部品は別イシュー（#848〜#851）のスコープです。",
        vec![stack(vec![chart, legend_node])],
    )
}

/// BarChart 節（イシュー #849、親 Phase #845）: 外部依存ゼロの SVG グループ棒
/// グラフ。縦（既定）/横 orientation を並べて掲示する。
fn bar_chart_section() -> Node {
    let data = bar_charts_sample_data();
    let vertical = bar_chart::root(
        &data,
        BarChartProps::default(),
        "monthly visits and signups",
    )
    .expect("ショーケース固定データは domain・viewBox とも常に有効");
    let horizontal = bar_chart::root(
        &data,
        BarChartProps {
            orientation: BarChartOrientation::Horizontal,
            ..BarChartProps::default()
        },
        "monthly visits and signups (horizontal)",
    )
    .expect("ショーケース固定データは domain・viewBox とも常に有効");

    section(
        "BarChart",
        "ChartData（複数系列）+ LinearScale + SVG ノード木生成ヘルパーのみで組み立てる、外部依存ゼロのグループ棒グラフです。orientation で縦/横を切り替えます。軸線・グリッド・凡例・ツールチップはイシュー #847 のスコープです。",
        vec![row(vec![vertical]), row(vec![horizontal])],
    )
}

/// BarList 節（イシュー #849）: ランキング型バーリスト。単一系列を対象に、
/// 系列内最大値に対する比率でバー幅を決める。
fn bar_list_section() -> Node {
    let data = bar_charts_sample_data();
    let node = bar_list::root(&data, "visits")
        .expect("ショーケース固定データの visits 系列は常に存在する");

    section(
        "BarList",
        "系列の最大値に対する比率でバー幅を決めるランキング型バーリストです。カテゴリ順（挿入順）にそのまま描画するため、降順表示にしたい場合は呼び出し側で ChartData::sort_by_series を事前に適用します。",
        vec![node],
    )
}

/// LineChart 節（イシュー #848、親 #845）: `charts` 基盤（#846）の消費者。
/// 3 カテゴリ 1 系列の折れ線を掲示する。
fn line_chart_section() -> Node {
    let data = ChartData::new(
        vec!["Jan".to_string(), "Feb".to_string(), "Mar".to_string()],
        vec![Series::new("visits", vec![10.0, 30.0, 20.0])],
    )
    .expect("showcase 固定データは常に有効");
    let node = line_chart::line_chart(&LineChartProps::new(&data, "monthly visits"), vec![])
        .expect("showcase 固定データは常に有効");
    section(
        "LineChart",
        "charts 基盤（座標スケーリング・SVG ノード木生成）を使った折れ線チャートです。軸・グリッド・凡例・ツールチップは別イシュー（#847）のスコープです。",
        vec![node],
    )
}

/// BarSegment 節（イシュー #849）: 構成比バー（100% 積み上げ）。単一系列の
/// 合計に対する各カテゴリの比率をセグメント幅として描画し、凡例を添える。
fn bar_segment_section() -> Node {
    let data = bar_charts_sample_data();
    let node = bar_segment::root(&data, "visits")
        .expect("ショーケース固定データの visits 系列合計は 0 ではない");

    section(
        "BarSegment",
        "系列合計に対する各カテゴリの構成比を 100% 積み上げバーとして表示します。合計が 0 の系列は構成比が定義できないため構築時に拒否されます（ChartError::ZeroTotal）。",
        vec![node],
    )
}

/// AreaChart 節（イシュー #848、親 #845）: 折れ線 + domain 下端へ閉じた
/// 塗りつぶし面を重ねて描く。
fn area_chart_section() -> Node {
    let data = ChartData::new(
        vec!["Jan".to_string(), "Feb".to_string(), "Mar".to_string()],
        vec![Series::new("visits", vec![10.0, 30.0, 20.0])],
    )
    .expect("showcase 固定データは常に有効");
    let node = area_chart::area_chart(&AreaChartProps::new(&data, "monthly visits"), vec![])
        .expect("showcase 固定データは常に有効");
    section(
        "AreaChart",
        "系列ごとに折れ線 + 塗りつぶし面を重ねて描く自己完結チャートです。積み上げ・曲線補間は別イシュー（#847 以降）のスコープです。",
        vec![node],
    )
}

/// Sparkline 節（イシュー #848、親 #845）: 軸・ラベルなしの縮小チャート。
/// 単一の `&[f64]` から直接描画する。
fn sparkline_section() -> Node {
    let values = [10.0, 30.0, 20.0, 40.0];
    let node = sparkline::sparkline(&SparklineProps::new(&values, "weekly trend"), vec![])
        .expect("showcase 固定データは常に有効");
    section(
        "Sparkline",
        "ラベル・軸なしの小さな面 + 線チャートです。単一系列専用（`&[f64]`）で、複数系列は LineChart/AreaChart を使います。",
        vec![node],
    )
}

/// PieChart 節（イシュー #850）: `size`（sm/md/lg）と `show_labels` の掲示。
fn pie_chart_section() -> Node {
    let data = ChartData::new(
        vec![
            "Q1".to_string(),
            "Q2".to_string(),
            "Q3".to_string(),
            "Q4".to_string(),
        ],
        vec![Series::new("revenue", vec![400.0, 300.0, 300.0, 200.0])],
    )
    .expect("ショーケース固定データは常に有効な ChartData を構築できる");

    let size_row = row([Size::Sm, Size::Md, Size::Lg]
        .into_iter()
        .map(|size| {
            pie_chart(
                &PieChartProps {
                    size,
                    ..PieChartProps::default()
                },
                &data,
                vec![],
            )
            .expect("ショーケース固定データは常に描画に成功する")
        })
        .collect());

    let with_labels = pie_chart(
        &PieChartProps {
            show_labels: true,
            ..PieChartProps::default()
        },
        &data,
        vec![],
    )
    .expect("ショーケース固定データは常に描画に成功する");
    let labels_row = row(vec![with_labels]);

    section(
        "PieChart",
        "外部依存ゼロの SVG ノード木生成による円グラフ（イシュー #850）。size（sm/md/lg）で --fandhe-pie-chart-size を切り替えます。show_labels を有効にするとカテゴリ名ラベルをセグメント上に描画します。",
        vec![size_row, labels_row],
    )
}

/// DonutChart 節（イシュー #850）: `size`・`inner_ratio`・`show_labels` の掲示。
fn donut_chart_section() -> Node {
    let data = ChartData::new(
        vec![
            "Q1".to_string(),
            "Q2".to_string(),
            "Q3".to_string(),
            "Q4".to_string(),
        ],
        vec![Series::new("revenue", vec![400.0, 300.0, 300.0, 200.0])],
    )
    .expect("ショーケース固定データは常に有効な ChartData を構築できる");

    let size_row = row([Size::Sm, Size::Md, Size::Lg]
        .into_iter()
        .map(|size| {
            donut_chart(
                &DonutChartProps {
                    size,
                    ..DonutChartProps::default()
                },
                &data,
                vec![],
            )
            .expect("ショーケース固定データは常に描画に成功する")
        })
        .collect());

    let thin_ring = donut_chart(
        &DonutChartProps {
            inner_ratio: 0.85,
            show_labels: true,
            ..DonutChartProps::default()
        },
        &data,
        vec![],
    )
    .expect("inner_ratio=0.85 は許容範囲内であり常に描画に成功する");
    let variant_row = row(vec![thin_ring]);

    section(
        "DonutChart",
        "外部依存ゼロの SVG ノード木生成によるドーナツグラフ（イシュー #850）。inner_ratio（既定 0.6）で内径を調整できます。show_labels を有効にするとカテゴリ名ラベルをセグメント上に描画します。",
        vec![size_row, variant_row],
    )
}

/// ScatterChart 節（イシュー #851、親 Phase #845）: 2 軸線形スケール +
/// 点マーカーのみで組み立てる、外部依存ゼロの SVG 散布図。
fn scatter_chart_section() -> Node {
    let data = ScatterData::new(vec![
        ScatterSeries::new(
            "cohort a",
            vec![(1.0, 2.0), (2.0, 4.5), (3.0, 3.0), (4.0, 6.0), (5.0, 5.5)],
        ),
        ScatterSeries::new(
            "cohort b",
            vec![(1.5, 1.0), (2.5, 2.5), (3.5, 4.0), (4.5, 3.5)],
        ),
    ])
    .expect("ショーケース固定データは常に有効");
    let node = scatter_chart::root(&data, ScatterChartProps::default(), "cohort comparison")
        .expect("ショーケース固定データは domain・viewBox とも常に有効");

    section(
        "ScatterChart",
        "散布図専用の ScatterData（系列ごとの (x, y) 座標列）+ LinearScale（x/y 双方）+ SVG ノード木生成ヘルパーのみで組み立てる、外部依存ゼロの散布図です。軸線・グリッド・凡例・ツールチップはイシュー #847 のスコープです。",
        vec![node],
    )
}

/// RadarChart 節（イシュー #851、親 Phase #845）: 正多角形グリッド + 系列
/// ポリゴンの SVG レーダーチャート。頂点角度は決定的な式で算出する。
fn radar_chart_section() -> Node {
    let data = ChartData::new(
        vec![
            "speed".to_string(),
            "power".to_string(),
            "range".to_string(),
            "control".to_string(),
            "armor".to_string(),
        ],
        vec![
            Series::new("mercury", vec![80.0, 60.0, 40.0, 90.0, 55.0]),
            Series::new("venus", vec![50.0, 85.0, 70.0, 45.0, 65.0]),
        ],
    )
    .expect("ショーケース固定データはカテゴリ数・系列長が一致する");
    let node = radar_chart::root(&data, RadarChartProps::default(), "stat comparison")
        .expect("ショーケース固定データは軸数 3 以上・非負値・viewBox とも常に有効");

    section(
        "RadarChart",
        "ChartData（カテゴリ = 軸、系列 = ポリゴン）+ LinearScale + SVG ノード木生成ヘルパーのみで組み立てる、外部依存ゼロのレーダーチャートです。頂点角度は θ_i = -π/2 + i・2π/n（12 時方向開始・時計回り）の決定的な式で算出します。凡例・ツールチップはイシュー #847 のスコープです。",
        vec![node],
    )
}

/// Tag 節（イシュー #768）: variant / size / colorPalette と、
/// close-trigger（`data-action` 配線のみ、クリック処理は wasm 層の
/// スコープ外）の掲示。
fn tag_section() -> Node {
    let variants = [
        (TagVariant::Solid, "Solid"),
        (TagVariant::Subtle, "Subtle"),
        (TagVariant::Outline, "Outline"),
    ];
    let variant_row = row(variants
        .iter()
        .map(|(variant, label)| {
            tag::root(
                &TagProps {
                    variant: *variant,
                    ..TagProps::default()
                },
                vec![],
                vec![text(*label)],
            )
        })
        .collect());
    let size_row = row([Size::Sm, Size::Md, Size::Lg]
        .iter()
        .map(|size| {
            tag::root(
                &TagProps {
                    size: *size,
                    ..TagProps::default()
                },
                vec![],
                vec![text("Tag")],
            )
        })
        .collect());
    let palette_row = row(palettes()
        .iter()
        .map(|(palette, label)| {
            tag::root(
                &TagProps {
                    palette: *palette,
                    ..TagProps::default()
                },
                vec![],
                vec![text(*label)],
            )
        })
        .collect());
    let closable = tag::root(
        &TagProps::default(),
        vec![],
        vec![
            tag::label(vec![], vec![text("Removable")]),
            tag::close_trigger(
                Some("remove_tag"),
                vec![("aria-label", "Remove")],
                vec![text("×")],
            ),
        ],
    );
    section(
        "Tag",
        "ラベル・分類・除去可能なチップ表示。variant / size / colorPalette を組み合わせます。close-trigger は data-action 属性の出力のみを担い、実際のクリック処理は wasm 層のスコープです。",
        vec![variant_row, size_row, palette_row, row(vec![closable])],
    )
}

/// Kbd 節（イシュー #768）: variant 軸を持たない単一 slot 部品。
fn kbd_section() -> Node {
    let row_node = row(vec![
        kbd(vec![], vec![text("Ctrl")]),
        text(" + "),
        kbd(vec![], vec![text("K")]),
    ]);
    section(
        "Kbd",
        "キーボード入力・ショートカット表示。variant 軸を持たない単一 slot の静的部品です。",
        vec![row_node],
    )
}

/// Code 節（イシュー #768）: インライン `<code>` のみを扱う（CodeBlock は
/// 対象外確定済み）。
fn code_section() -> Node {
    let row_node = row(vec![code(vec![], vec![text("cargo build")])]);
    section(
        "Code",
        "インラインコード片の表示。chakra-ui の CodeBlock 相当は対象外です。",
        vec![row_node],
    )
}

/// ColorSwatch 節（イシュー #838）: size / shape の掲示と、透過色の
/// チェッカーボード表示確認。
fn color_swatch_section() -> Node {
    let blue = Color::from_rgb(Rgb::new(0x3b, 0x82, 0xf6));
    let size_row = row([Size::Sm, Size::Md, Size::Lg]
        .iter()
        .map(|size| {
            color_swatch::color_swatch(
                &ColorSwatchProps {
                    value: blue,
                    size: *size,
                    ..ColorSwatchProps::default()
                },
                vec![],
                vec![],
            )
        })
        .collect());
    let shape_row = row([
        SwatchShape::Square,
        SwatchShape::Circle,
        SwatchShape::Rounded,
    ]
    .iter()
    .map(|shape| {
        color_swatch::color_swatch(
            &ColorSwatchProps {
                value: blue,
                shape: *shape,
                ..ColorSwatchProps::default()
            },
            vec![],
            vec![],
        )
    })
    .collect());
    let transparent_row = row(vec![color_swatch::color_swatch(
        &ColorSwatchProps {
            value: Color::from_rgba(Rgb::new(0x3b, 0x82, 0xf6), 0x80),
            ..ColorSwatchProps::default()
        },
        vec![],
        vec![],
    )]);
    section(
        "ColorSwatch",
        "色見本の静的表示です。size / shape を組み合わせられます。アルファ付き色は下地のチェッカーボード模様で透過が視認できます。",
        vec![size_row, shape_row, transparent_row],
    )
}

/// ColorPicker 節（イシュー #839）: 開いた状態を固定して掲示する（本モジュール
/// 冒頭「インタラクティブ部品の扱い」参照）。Area（彩度・明度の 2 次元
/// グラデーション）・色相/アルファスライダー・HEX 入力を静的 SSR マークアップ
/// として表示する。canvas は使わず、すべて CSS グラデーション + 検証済み
/// 整数割合（`state.area_x_percent()` 等）の custom property のみで組み立てる
/// （`fandhe_frontend_pre_styled_ui::color_picker` モジュール doc「canvas
/// 非依存」参照）。
fn color_picker_section() -> Node {
    use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::color_picker::Channel;

    let state = ColorPicker::from_color(Color::from_rgba(Rgb::new(0x3b, 0x82, 0xf6), 0xcc));
    let demo = row(vec![color_picker::content(
        state.state(),
        None,
        vec![],
        vec![
            color_picker::area(
                &state,
                vec![],
                vec![
                    color_picker::area_background(&state, vec![], vec![]),
                    color_picker::area_thumb(&state, false, vec![], vec![]),
                ],
            ),
            color_picker::channel_slider(
                Channel::Hue,
                &state,
                vec![],
                vec![
                    color_picker::channel_slider_track(Channel::Hue, &state, vec![], vec![]),
                    color_picker::channel_slider_thumb(Channel::Hue, &state, false, vec![], vec![]),
                ],
            ),
            color_picker::channel_slider(
                Channel::Alpha,
                &state,
                vec![],
                vec![
                    color_picker::channel_slider_track(Channel::Alpha, &state, vec![], vec![]),
                    color_picker::channel_slider_thumb(
                        Channel::Alpha,
                        &state,
                        false,
                        vec![],
                        vec![],
                    ),
                ],
            ),
            color_picker::control(
                vec![],
                vec![
                    color_picker::channel_input(state.hex().as_str(), false, vec![]),
                    color_picker::value_text(vec![], vec![text(state.hex())]),
                ],
            ),
        ],
    )]);
    section(
        "ColorPicker",
        "HSV 色相環 + アルファ選択の静的表示です（canvas 非依存、CSS グラデーション + 検証済み割合のみで構成）。ポインタ操作の実配線は wasm 層の後続対応です。",
        vec![demo],
    )
}

/// colorPalette 軸の全値（表示ラベル付き）。Button / Badge の palette 行で
/// 共有する。
fn palettes() -> [(ColorPalette, &'static str); 5] {
    [
        (ColorPalette::Accent, "Accent"),
        (ColorPalette::Info, "Info"),
        (ColorPalette::Success, "Success"),
        (ColorPalette::Warning, "Warning"),
        (ColorPalette::Danger, "Danger"),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;

    /// 全部品を横断する回帰テスト専用のテストヘルパー（旧・集約ページ
    /// `showcase_body`、イシュー #943 でページ生成用途からは撤去済み）。
    /// [`COMPONENT_PAGES`] レジストリを登録順に走査して全節を連結し、
    /// data-scope 網羅性・WAI-ARIA 属性・recipe CSS 網羅性の各回帰テストが
    /// 全部品を一括レンダリングして検査できるようにする。本番のページ
    /// 生成（[`generated_content`]）はこの関数を経由しない。
    fn showcase_body() -> Node {
        showcase_wrapper(
            COMPONENT_PAGES
                .iter()
                .map(|entry| (entry.render)())
                .collect(),
        )
    }

    #[test]
    fn generated_content_returns_none_for_index_page() {
        // イシュー #943: 索引ページ（PAGE_PATH）は Rust 生成コンテンツを
        // 持たない（本文はすべて site/components-pre-styled-ui.md 側）。
        assert!(generated_content(PAGE_PATH).is_none());
    }

    #[test]
    fn generated_content_returns_section_for_every_registered_component_page() {
        // COMPONENT_PAGES に登録済みの全パスが Some を返す
        // （レジストリと 2 段照会のドリフトを検知する）。
        for path in component_page_paths() {
            assert!(
                generated_content(path).is_some(),
                "generated_content({path}) should return Some"
            );
        }
    }

    #[test]
    fn generated_content_returns_none_for_unregistered_paths() {
        assert!(generated_content("/").is_none());
        assert!(generated_content("/guides/embedding-guide/").is_none());
        assert!(generated_content("/components/nonexistent/").is_none());
    }

    #[test]
    fn component_page_paths_are_unique_and_well_formed() {
        let paths: Vec<&str> = component_page_paths().collect();

        // 機械的な分解作業中の取りこぼし・重複追加を fail-closed で検知する
        // 件数センチネル。台帳（`docs/design/docs-site-component-pages.md`）
        // 99 件との突合は #944 の責務。
        assert_eq!(paths.len(), 92, "COMPONENT_PAGES should have 92 entries");

        let mut sorted = paths.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(
            sorted.len(),
            paths.len(),
            "component page paths must be unique"
        );

        for path in &paths {
            // `nav::validate_page_path`（`is_safe_path_segment`）の
            // allowlist（英数・`-`・`_`、`/` 始まり `/` 終わり）をミラーする。
            assert!(
                path.starts_with("/components/") && path.ends_with('/'),
                "unexpected path shape: {path}"
            );
            let inner = &path[1..path.len() - 1];
            assert!(
                inner.split('/').all(|seg| !seg.is_empty()
                    && seg
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')),
                "path segment fails nav::validate_page_path allowlist: {path}"
            );
        }
    }

    #[test]
    fn component_page_markup_has_no_non_empty_href() {
        // 集約ページ単位の href 中立性テスト
        // （showcase_markup_has_no_href_attributes_for_linkcheck_neutrality）
        // をエントリ単位へ強化する。build.rs の linkcheck は全 href を
        // 突合検証するため、生成コンテンツは実ページへ解決される href を
        // 持たない設計を各部品ページ単位でも維持する。
        for path in component_page_paths() {
            let node = generated_content(path).expect("registered path should resolve");
            let html = render(&node);
            let non_empty_hrefs: Vec<&str> = html
                .match_indices("href=\"")
                .filter(|(i, _)| !html[i + 6..].starts_with('"'))
                .map(|(i, _)| &html[i..i + 20.min(html.len() - i)])
                .collect();
            assert!(
                non_empty_hrefs.is_empty(),
                "non-empty href found on {path}: {non_empty_hrefs:?}"
            );
        }
    }

    #[test]
    fn component_page_content_is_wrapped_in_showcase_scope() {
        // 単体レンダリング結果が .pre-styled-showcase でラップされている
        // ことを確認する（オーバーレイ中和 CSS のスコープ前提、
        // showcase_wrapper 参照）。
        for path in component_page_paths() {
            let node = generated_content(path).expect("registered path should resolve");
            let html = render(&node);
            assert!(
                html.contains(r#"class="pre-styled-showcase""#),
                "missing pre-styled-showcase wrapper on {path}"
            );
        }
    }

    #[test]
    fn showcase_markup_contains_all_component_scopes() {
        let html = render(&showcase_body());
        for scope in [
            "button",
            "badge",
            "spinner",
            "alert",
            "card",
            "tabs",
            "accordion",
            "dialog",
            "drawer",
            "menu",
            "select",
            "listbox",
            "popover",
            "tooltip",
            "hover-card",
            "toggle-tip",
            "switch",
            "radio-group",
            "avatar",
            "checkbox",
            "field",
            "number-input",
            "password-input",
            "tags-input",
            "rating-group",
            "slider",
            "editable",
            "segment-group",
            "toggle",
            "toggle-group",
            "pagination",
            "steps",
            "splitter",
            "checkbox-card",
            "radio-card",
            "breadcrumb",
            "toast",
            "image",
            "icon",
            "tag",
            "kbd",
            "code",
            "color-swatch",
            "status",
            "empty-state",
            "visually-hidden",
            "table",
            "data-list",
            "date-input",
            "timer",
            "chart",
            "chart-legend",
            "bar-chart",
            "bar-list",
            "bar-segment",
            "line-chart",
            "area-chart",
            "sparkline",
            "scatter-chart",
            "radar-chart",
        ] {
            assert!(
                html.contains(&format!(r#"data-scope="{scope}""#)),
                "missing data-scope={scope}"
            );
        }
        // Input / Textarea / NativeSelect（イシュー #737）: field scope 内の
        // 3 パーツすべてが掲示されていることを固定する。
        for part in ["input", "textarea", "select"] {
            assert!(
                html.contains(&format!(r#"data-scope="field" data-part="{part}""#)),
                "missing data-scope=field data-part={part}"
            );
        }
        // 静的掲示の状態固定: 選択中タブ・開いた Accordion 項目・checked
        // Switch/RadioGroup item・indeterminate Checkbox。
        assert!(html.contains(r#"data-state="active""#));
        assert!(html.contains(r#"data-state="open""#));
        assert!(html.contains(r#"data-state="checked""#));
        assert!(html.contains(r#"data-state="indeterminate""#));
        // PasswordInput（イシュー #740）: 表示切替の Visible/Hidden 両状態と
        // aria-pressed によるトグルボタン意味論を固定する。
        assert!(html.contains(r#"data-state="visible""#));
        assert!(html.contains(r#"data-state="hidden""#));
        assert!(html.contains(r#"aria-pressed="true""#));
        assert!(html.contains(r#"aria-pressed="false""#));
        // visibility-trigger は可視のラベルテキストを持つ（Bugbot 指摘の
        // 回帰防止: 空 children では show/hide ボタンに可視コンテンツが
        // 一切なくなる、イシュー #740 PR #786 レビュー）。
        assert!(html.contains(r#">Show<"#));
        assert!(html.contains(r#">Hide<"#));
    }

    #[test]
    fn showcase_markup_fixes_overlay_components_open_with_wai_aria_roles() {
        // イシュー #691 受け入れ条件: Dialog/Menu/Select/Popover/Tooltip は
        // 開いた状態を固定し、対応する WAI-ARIA role/属性が出力されることを
        // 固定する（headless 層の既存保証をショーケース掲示側でも回帰させる）。
        let html = render(&showcase_body());
        assert!(html.contains(r#"aria-modal="true""#)); // dialog content
        assert!(html.contains(r#"role="menu""#));
        assert!(html.contains(r#"role="listbox""#));
        assert!(html.contains(r#"role="tooltip""#));
        assert!(html.contains(r#"aria-expanded="true""#)); // trigger 群（開状態）
        assert!(html.contains(r#"aria-haspopup="dialog""#)); // dialog/popover trigger
        assert!(html.contains(r#"aria-haspopup="menu""#));
        assert!(html.contains(r#"aria-haspopup="listbox""#));
        // Toast（イシュー #760）: root は role="status" + aria-atomic="true"。
        // aria-live は status 別に導出され、Error のみ assertive（本モジュール
        // 冒頭 rustdoc の `aria-live` 節・`toast_section` 参照）。
        assert!(html.contains(r#"role="status""#));
        assert!(html.contains(r#"aria-atomic="true""#));
        assert!(html.contains(r#"aria-live="assertive""#));
        assert!(html.contains(r#"aria-live="polite""#));
    }

    #[test]
    fn showcase_markup_shows_listbox_single_and_multiple_modes() {
        // イシュー #750 受け入れ条件: Listbox は常時展開（trigger/positioner
        // なし）で、single/multiple 双方の掲示が固定されていることを確認する。
        let html = render(&showcase_body());
        assert!(html.contains(r#"data-scope="listbox" data-part="content""#));
        assert!(html.contains(r#"aria-multiselectable="true""#));
        assert!(html.contains(r#"data-scope="listbox" data-part="item-group""#));
    }

    #[test]
    fn showcase_markup_has_no_href_attributes_for_linkcheck_neutrality() {
        // build.rs の linkcheck は全 href を突合検証する。生成コンテンツは
        // 実ページへ解決される href を持たない設計とし、リンク検証対象を
        // Markdown 側へ限定する。イシュー #755 で Breadcrumb（`link` パーツ、
        // 実際に `href` 属性を持つ anatomy）を掲示したため、本テストは
        // 「`href=""`（空文字列、`crate::linkcheck::check_links` が無条件
        // スキップする値）以外の href が存在しないこと」へ更新した
        // （`showcase::breadcrumb_section` rustdoc 参照。空 href 以外を
        // 足す場合はこのテストを更新して linkcheck との整合を明示的に
        // 設計し直すこと）。
        let html = render(&showcase_body());
        let non_empty_hrefs: Vec<&str> = html
            .match_indices("href=\"")
            .filter(|(i, _)| !html[i + 6..].starts_with('"'))
            .map(|(i, _)| &html[i..i + 20.min(html.len() - i)])
            .collect();
        assert!(
            non_empty_hrefs.is_empty(),
            "non-empty href found: {non_empty_hrefs:?}"
        );
    }

    #[test]
    fn stylesheet_covers_theme_component_and_layout_css() {
        let sheet = stylesheet().expect("showcase stylesheet should assemble");
        let css = sheet.as_css();
        // テーマトークン（ライト/ダーク基盤）。
        assert!(css.contains("--fandhe-color-"));
        // 各コンポーネントの recipe セレクタ。
        assert!(css.contains(".fd-button--variant-solid"));
        // DownloadTrigger（イシュー #828）の recipe CSS が stylesheet() に
        // 反映されていること（Bugbot 指摘: 追加当初漏れていた回帰防止）。
        assert!(css.contains(".fd-download-trigger--variant-solid"));
        assert!(css.contains(".fd-badge--variant-subtle"));
        assert!(css.contains(r#"[data-scope="tabs"][data-part="trigger"]"#));
        assert!(css.contains(r#"[data-scope="accordion"]"#));
        assert!(css.contains(r#"[data-scope="dialog"][data-part="content"]"#));
        assert!(css.contains(r#"[data-scope="drawer"][data-part="content"]"#));
        assert!(css.contains(r#"[data-scope="menu"][data-part="content"]"#));
        assert!(css.contains(r#"[data-scope="select"][data-part="content"]"#));
        assert!(css.contains(r#"[data-scope="popover"][data-part="content"]"#));
        assert!(css.contains(r#"[data-scope="tooltip"][data-part="content"]"#));
        assert!(css.contains(r#"[data-scope="hover-card"][data-part="content"]"#));
        assert!(css.contains(r#"[data-scope="switch"][data-part="control"]"#));
        assert!(css.contains(r#"[data-scope="radio-group"][data-part="item-control"]"#));
        assert!(css.contains(".fd-avatar--size-md"));
        assert!(css.contains(".fd-avatar--shape-circle"));
        assert!(css.contains(r#"[data-scope="checkbox"][data-part="control"]"#));
        assert!(css.contains(r#"[data-scope="checkbox-card"][data-part="indicator"]"#));
        assert!(css.contains(r#"[data-scope="radio-card"][data-part="item-indicator"]"#));
        assert!(css.contains(r#"[data-scope="field"][data-part="input"]"#));
        assert!(css.contains(r#"[data-scope="field"][data-part="textarea"]"#));
        assert!(css.contains(r#"[data-scope="field"][data-part="select"]"#));
        assert!(css.contains(r#"[data-scope="number-input"][data-part="control"]"#));
        assert!(css.contains(r#"[data-scope="password-input"][data-part="control"]"#));
        assert!(css.contains(r#"[data-scope="tags-input"][data-part="control"]"#));
        assert!(css.contains(".fd-tag--variant-subtle"));
        assert!(css.contains(r#"[data-scope="kbd"][data-part="root"]"#));
        assert!(css.contains(r#"[data-scope="code"][data-part="root"]"#));
        assert!(css.contains(r#"[data-scope="toast"][data-part="root"]"#));
        assert!(css.contains(r#"[data-scope="status"][data-part="indicator"]"#));
        assert!(css.contains(r#"[data-scope="empty-state"][data-part="content"]"#));
        assert!(css.contains(r#"[data-scope="table"][data-part="row"]:nth-child(even)"#));
        assert!(css.contains(r#"[data-scope="data-list"][data-part="root"]"#));
        // Charts（イシュー #847）: axis/grid/legend/tooltip の recipe CSS。
        assert!(css.contains(r#"[data-scope="chart"][data-part="axis-line"]"#));
        assert!(css.contains(r#"[data-scope="chart"][data-part="grid-line"]"#));
        assert!(css.contains(r#"[data-scope="chart-legend"][data-part="root"]"#));
        assert!(css.contains(r#"[data-scope="chart"][data-part="datum"]:hover"#));
        // ショーケース配置スタイル。
        assert!(css.contains(".showcase-row"));
        assert!(css.contains(".showcase-stack"));
        // Accordion anatomy の h3 への `.docs-content h3`（site.css）漏れを
        // 遮断する見出しリセット（Bugbot 指摘の回帰防止）。
        assert!(css.contains(r#".pre-styled-showcase [data-scope="accordion"] h3"#));
        // オーバーレイ配置中和ルール（イシュー #691）。
        assert!(css.contains(r#".pre-styled-showcase [data-scope="dialog"][data-part="backdrop"]"#));
        assert!(
            css.contains(r#".pre-styled-showcase [data-scope="dialog"][data-part="positioner"]"#)
        );
        assert!(css.contains(r#".pre-styled-showcase [data-scope="menu"][data-part="positioner"]"#));
        assert!(css.contains(r#".pre-styled-showcase [data-scope="dialog"] h2"#));
        assert!(css.contains(r#".pre-styled-showcase [data-scope="popover"] h2"#));
        assert!(css.contains(r#".pre-styled-showcase [data-scope="toast"][data-part="group"]"#));
        // Tour（イシュー #841、PR #870 Bugbot 指摘 High severity「Showcase
        // omits Tour CSS wiring」の回帰防止）: recipe CSS 本体が組み込まれ、
        // かつ Active 固定掲示のオーバーレイ（backdrop/spotlight/positioner）
        // がショーケース内でページ全体を覆わないよう中和されていること。
        assert!(css.contains(
            r#"[data-scope="tour"][data-part="positioner"][data-side="left"][data-align="start"]"#
        ));
        assert!(css.contains(r#".pre-styled-showcase [data-scope="tour"][data-part="backdrop"]"#));
        assert!(css.contains(r#".pre-styled-showcase [data-scope="tour"][data-part="spotlight"]"#));
        assert!(css.contains(r#".pre-styled-showcase [data-scope="tour"][data-part="positioner"]"#));
        // StyleSheet の不変条件（<style> 埋め込み・CSS ファイル双方で安全）。
        assert!(!css.contains('<'));
    }

    #[test]
    fn stylesheet_is_deterministic() {
        let a = stylesheet().unwrap().as_css().to_string();
        let b = stylesheet().unwrap().as_css().to_string();
        assert_eq!(a, b);
    }
}
