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
//! を持つ。`/themes/`（[`PAGE_PATH`]）は #943 で索引
//! （凡例 + カテゴリ別リンク集）へ改組済みであり、Rust 生成コンテンツを
//! 持たない（URL 自体は既存被リンク〔`docs/api/pre-styled-ui-api.md` 等〕
//! 維持のため変更していない）。集約レンダリング（旧 `showcase_body`）は
//! 全部品を横断する回帰テスト専用のテストヘルパーとしてのみ
//! `#[cfg(test)]` 配下に残す。`site/nav.toml` への部品ページ登録・原稿
//! スタブ作成は #943、Demo/Features/Anatomy/API Reference の雛形合成は
//! #942 の責務であり、本モジュールは器（レジストリと照会 API）のみを
//! 提供する。

use fandhe_frontend_core::{div, el, render, text, Node};
use fandhe_frontend_pre_styled_ui::action_bar;
use fandhe_frontend_pre_styled_ui::area_chart::{self, AreaChartProps};
use fandhe_frontend_pre_styled_ui::avatar::{
    self, AvatarProps, AvatarShape, AvatarVariant, ImageStatus,
};
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
use fandhe_frontend_pre_styled_ui::checkbox_group;
use fandhe_frontend_pre_styled_ui::code::{code, CodeProps, CodeVariant};
use fandhe_frontend_pre_styled_ui::color_picker;
use fandhe_frontend_pre_styled_ui::color_swatch::{
    self, Color, ColorSwatchProps, Rgb, SwatchShape,
};
use fandhe_frontend_pre_styled_ui::data_list::{
    self, DataListOrientation, DataListProps, DataListVariant,
};
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
use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::data_attrs::data_state;
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
use fandhe_frontend_pre_styled_ui::highlight::{highlight, HighlightProps, HighlightVariant};
use fandhe_frontend_pre_styled_ui::hover_card::{self, HoverCardDelays};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::image::{image, AspectRatio, ImageFit, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::input::{self, FieldIds, FieldProps, InputProps};
use fandhe_frontend_pre_styled_ui::json_tree_view::{self, JsonValue};
use fandhe_frontend_pre_styled_ui::kbd::{kbd, KbdProps, KbdVariant};
use fandhe_frontend_pre_styled_ui::line_chart::{self, LineChartProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps, LinkVariant};
use fandhe_frontend_pre_styled_ui::link_overlay;
use fandhe_frontend_pre_styled_ui::list::{self, ListType, ListVariant};
use fandhe_frontend_pre_styled_ui::listbox;
use fandhe_frontend_pre_styled_ui::mark::{mark, MarkProps, MarkVariant};
use fandhe_frontend_pre_styled_ui::marquee::{self, MarqueeDirection, MarqueeProps};
use fandhe_frontend_pre_styled_ui::menubar::{self, Menubar};
use fandhe_frontend_pre_styled_ui::native_select::{self, NativeSelectProps};
use fandhe_frontend_pre_styled_ui::nav_list;
use fandhe_frontend_pre_styled_ui::navigation_menu;
use fandhe_frontend_pre_styled_ui::number_input::{self, NumberInputFlags};
use fandhe_frontend_pre_styled_ui::pagination::{self, ItemMode, Pagination};
use fandhe_frontend_pre_styled_ui::password_input::{
    self, PasswordAutocomplete, PasswordInputProps,
};
use fandhe_frontend_pre_styled_ui::pie_chart::{pie_chart, PieChartProps};
use fandhe_frontend_pre_styled_ui::qr_code;
use fandhe_frontend_pre_styled_ui::quote::quote;
use fandhe_frontend_pre_styled_ui::radio_card;
use fandhe_frontend_pre_styled_ui::rating_group::{self, RatingGroup, RatingItemFlags};
use fandhe_frontend_pre_styled_ui::scroll_area;
use fandhe_frontend_pre_styled_ui::segment_group;
use fandhe_frontend_pre_styled_ui::separator::{separator, SeparatorProps, SeparatorVariant};
use fandhe_frontend_pre_styled_ui::skeleton::{
    skeleton, SkeletonAnimation, SkeletonProps, SkeletonVariant,
};
use fandhe_frontend_pre_styled_ui::slider;
use fandhe_frontend_pre_styled_ui::sparkline::{self, SparklineProps};
use fandhe_frontend_pre_styled_ui::spinner::{spinner, SpinnerProps};
use fandhe_frontend_pre_styled_ui::splitter;
use fandhe_frontend_pre_styled_ui::stat;
use fandhe_frontend_pre_styled_ui::status::{self, StatusProps};
use fandhe_frontend_pre_styled_ui::steps;
use fandhe_frontend_pre_styled_ui::strong::strong;
use fandhe_frontend_pre_styled_ui::tab_nav;
use fandhe_frontend_pre_styled_ui::table::{self, TableProps, TableVariant};
use fandhe_frontend_pre_styled_ui::tabs::{tabs, ActivationMode, TabItem, TabsProps};
use fandhe_frontend_pre_styled_ui::tag::{self, TagProps, TagVariant};
use fandhe_frontend_pre_styled_ui::tags_input;
use fandhe_frontend_pre_styled_ui::text::{text as styled_text, TextProps, TextSize, TextWeight};
use fandhe_frontend_pre_styled_ui::textarea::{self, TextareaProps};
use fandhe_frontend_pre_styled_ui::theme::Theme;
use fandhe_frontend_pre_styled_ui::timeline::{self, TimelineVariant};
use fandhe_frontend_pre_styled_ui::timer::{self, Timer, TimerControl, TimerPhase, TimerUnit};
use fandhe_frontend_pre_styled_ui::toast::{self, ToastPlacement, ToastStatus};
use fandhe_frontend_pre_styled_ui::toggle_group;
use fandhe_frontend_pre_styled_ui::toolbar::{self, Toolbar};
use fandhe_frontend_pre_styled_ui::tour::{self, ContentIds as TourContentIds};
use fandhe_frontend_pre_styled_ui::tree_view::{self, TreeNode, TreeView};
use fandhe_frontend_pre_styled_ui::visually_hidden;
use fandhe_frontend_pre_styled_ui::{
    accordion, alert, badge, callout, card, combobox, menu, popover, radio_group, select, switch,
    toggle, toggle_tip, tooltip, AlertProps, AlertStatus, AlertVariant, BadgeProps, BadgeVariant,
    CalloutProps, CalloutVariant, CardProps, CardVariant, ColorPalette, OpenState, Orientation,
    Size, StyleSheet, StylesheetError, VariantValue,
};

/// 索引ページ（凡例 + カテゴリ別リンク集）の `page.path`。`site/nav.toml`
/// の `[[section.page]]` 宣言と一致させる契約（乖離を防ぐ用途で
/// `tests/site_nav.rs` / `tests/site_showcase.rs` が参照する）。イシュー
/// #943 で索引ページへ改組済みのため、本モジュールはこのページ向けに
/// Rust 生成コンテンツを持たない（[`generated_content`] は常に `None` を
/// 返す。索引の本文はすべて `site/themes.md` 側で持つ。URL 自体はイシュー
/// #1018 で `/components/pre-styled-ui/` から `/themes/` へ移設した）。
pub const PAGE_PATH: &str = "/themes/";

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
/// - dialog/drawer/menu/menubar/select/combobox/popover/tooltip/hover-card/
///   toggle-tip/action-bar の `[data-part="positioner"]` を `position: static`
///   へ中和: recipe CSS は dialog/drawer を `position: fixed; inset: 0`、
///   menu/menubar/select/combobox/popover/hover-card を `position: absolute;
///   top: 100%`、tooltip/toggle-tip を `position: absolute; bottom: 100%`、
///   action-bar を `position: fixed; bottom: ...; left: 50%; transform:
///   translateX(-50%)` としており、いずれも開いた content をページ内の別
///   位置・別セクションに重ねてしまう。static 化してフロー内へインライン
///   表示させることで、後続セクションと重ならずに掲示できる（dialog は
///   さらに `padding`/`justify-content` も中和し、中央寄せのための余白・
///   配置指定を解除する。drawer は recipe CSS が `padding`/`justify-content`
///   を宣言しないため `position` のみで足りる。action-bar はさらに
///   `transform` も中和し、水平方向のずらしを解除する。menubar は Menu の
///   `open=Some(0)` 掲示（イシュー #992）で File Menu の `content` を開いた
///   状態にレンダリングするため、他のオーバーレイ `positioner` と同様の
///   中和が必要になる）。
/// - `[data-scope="menubar"][data-part="root"]` の `align-items: flex-start`
///   への上書き（イシュー #992、PR #1000 Bugbot 指摘 1 対応）: 上記の
///   `positioner` 中和により、開いた File Menu の `content` は
///   per-menu ラッパー（`menu` パーツ、`root` の flex item）の中で
///   `trigger` の下へ通常フローで積み上がる。recipe CSS の `root` は
///   `align-items: center`（トリガーのみの Menu を想定した既定値）を
///   宣言しており、この既定のままだと「トリガー + 開いた content」で
///   縦に長くなった File の flex item が高さの中央で揃えられてしまい、
///   `content` を持たない Edit の flex item だけが上へ押し上げられて
///   トリガー行から外れる（Edit が File パネルの横へずれ、水平な
///   menubar に見えなくなる回帰）。`align-items: flex-start` へ限定
///   上書きし、各 `menu` flex item の上端（= 各 `trigger` の位置）を
///   揃えることでトリガー行を保つ（`content` の高さ差は下方向にのみ
///   影響し、トリガー行のレイアウトには影響しない）。recipe CSS
///   （`crates/pre-styled-ui/src/menubar.rs`）自体は変更しない
///   （showcase 領域内に限定した上書きのみで完結させる、本節冒頭の方針
///   と同型）。
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
/// - `[data-scope="navigation-menu"][data-part="content"]` を
///   `position: static` へ中和（イシュー #993、PR #1000 Menubar showcase
///   の重なり回帰と同型の予防）: recipe CSS
///   （`crates/pre-styled-ui/src/navigation_menu.rs`）は `content` を
///   `position: absolute; top: 100%` としており、Products トリガーを開いた
///   状態で固定掲示すると後続の About 項目・後続セクションへ重なって
///   しまう。static 化してフロー内へインライン表示させることで、他の
///   オーバーレイ系パーツと同様に後続セクションと重ならずに掲示できる。
///   recipe CSS 側の `list` の `align-items: flex-start`（`center` ではなく）
///   は、この中和後にも Products 項目だけが Content の高さぶん縦に伸びて
///   About 項目が縦ずれする回帰を recipe 側の既定値そのもので防ぐ設計判断
///   であり、本 showcase 側の中和ルールは追加しない
///   （`crates/pre-styled-ui/src/navigation_menu.rs` rustdoc「レイアウト」
///   節参照。menubar の `align-items: flex-start` 上書きが showcase 側の
///   中和として必要だったのとは異なり、navigation-menu は recipe 自体が
///   `flex-start` を既定にしているため showcase 側の追加中和は不要）。
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
/// - Link Overlay デモの素の `h3`（イシュー #1154、Bugbot 指摘 1 回目 + 2 回目）:
///   `link_overlay::root` は `overlay` 以外の子ノードを anatomy 化せず
///   デモ側が素の `<h3>` を直接渡す構成であり、data-scope/data-part を
///   持たないため Accordion `h3` と同じ経路で `.docs-content h3`
///   （詳細度 (0,1,1)、`margin: 1.6rem 0 0.5rem` を含む）が漏れ込む。
///   Accordion の `h3` はスタイル済みトリガーのラッパに過ぎず見出しとして
///   視認されないため font-size/font-weight ごとフルリセットしてよいが、
///   Link Overlay の `h3` はカードの可視見出しそのものであり、同じフル
///   リセットを適用すると隣接する `p`（説明文）と区別が付かない見た目に
///   なってしまう（2 回目の Bugbot 指摘）。したがってカード上端の余分な
///   `margin-top`（`root` はカード状の位置決めコンテキストであり見出し前に
///   1.6rem の空白は不要）のみを打ち消す（ショートハンド `margin: 0` は
///   `.docs-content h3` の `margin-bottom: 0.5rem` まで潰してしまい、
///   直後の `p`〔説明文〕と見出しが密着する新たな欠陥を生むため longhand
///   の `margin-top` のみを宣言する。dialog/popover の `h2` リセットが
///   `border-top`/`padding-top` のみを longhand で打ち消すのと同型）。
///   見出しらしさを与える font-size/font-weight/line-height/letter-spacing
///   は `.docs-content h3` のまま活かす最小リセットを
///   `.pre-styled-showcase [data-scope="link-overlay"][data-part="root"] h3`
///   （詳細度 (0,3,1)）で適用する。
/// - Link Overlay の `border-radius: inherit` / `:focus-visible` リング
///   （イシュー #1580）: `link_overlay::stylesheet()` を無条件出荷しない
///   （下記コメント参照）ため、`crates/pre-styled-ui/src/link_overlay.rs`
///   の `recipe()` へ新規追加した `overlay` base の `border-radius: inherit`
///   と `StateCondition::FocusVisible` 状態規則
///   （`focus_ring_declarations`）も、既存の位置決め規則と同じ理由で
///   `.pre-styled-showcase` スコープ付きの等価ルールとして個別に複製する。
///   **追補（Bugbot 指摘、PR #1853）**: `root` base にも追加した
///   `border-radius: inherit`（`overlay` へ角丸を連鎖させるための宣言）も
///   同じ理由で対で複製する。
///   既存の無条件出荷禁止ガード
///   （`stylesheet_never_takes_up_link_overlay_stylesheet` テスト）は
///   スコープなしの overlay 位置決め規則本文（`position: absolute;`
///   から始まる行）の不在のみを検査するため、新規宣言を同じ本文へ混ぜず
///   別ブロックとして追加すれば当該ガードは影響を受けない。
/// - Nav List の `heading`（`h2`）見出しリセット（イシュー #1154、Bugbot
///   指摘）: `heading` パーツ自体が `data-scope="nav-list"
///   data-part="heading"` を持つため、dialog/drawer/popover の `h2` と同じ
///   理由・同じ最小リセット（`border-top`/`padding-top`/`letter-spacing`
///   のみ。margin/font-size/font-weight は recipe（`nav_list::recipe`）の
///   `heading` base 宣言が既に持ち自然に勝つため宣言しない）を、子孫
///   セレクタではなく要素自身への属性セレクタ
///   `.pre-styled-showcase [data-scope="nav-list"][data-part="heading"]`
///   （詳細度 (0,3,0)）で適用する。
/// - Link / Nav List デモの hover 下線漏れ（イシュー #1154、Bugbot 指摘）:
///   `site.css` の `.docs-content a:hover`（詳細度 (0,2,1)、`text-decoration:
///   underline` を宣言）が、Link recipe の `root`（`crates/pre-styled-ui/src/link.rs`
///   の `text-decoration: var(--fandhe-link-text-decoration, none)`、詳細度
///   (0,2,0)）・Nav List recipe の `link`（`crates/pre-styled-ui/src/nav_list.rs`
///   の `text-decoration: none`、詳細度 (0,2,0)）のいずれよりも詳細度が
///   高く hover 時に勝ってしまい、Plain Link（下線なし）・Nav List の
///   リンクがホバー時に一律下線付きになる（Underline Link との視覚的な
///   区別が失われる）。recipe CSS 自体は変更せず、showcase 領域内に限定
///   した `data-scope`/`data-part` 属性セレクタ + `:hover` で明示的に
///   recipe が意図する下線状態へ引き戻す（`.pre-styled-showcase` + 属性
///   2 個 + `:hover` = (0,4,0) が `.docs-content a:hover` = (0,2,1) より
///   優先される）。Link 側は variant による切り替え（Underline は
///   ホバー時も下線のまま）を保つため `var(--fandhe-link-text-decoration,
///   none)` をそのまま再適用し、Nav List 側は recipe と同じ固定値
///   `none` を再適用する。
const SHOWCASE_LAYOUT_CSS: &str = "\
.pre-styled-showcase {\n  display: flex;\n  flex-direction: column;\n  gap: 1.5rem;\n}\n\
.showcase-row {\n  display: flex;\n  flex-wrap: wrap;\n  gap: 0.75rem;\n  align-items: center;\n  margin: 1rem 0;\n}\n\
.showcase-stack {\n  display: flex;\n  flex-direction: column;\n  gap: 0.75rem;\n  margin: 1rem 0;\n  max-width: 36rem;\n}\n\
.showcase-form-field-group {\n  display: flex;\n  flex-direction: column;\n  gap: 0.25rem;\n  width: 100%;\n}\n\
.pre-styled-showcase [data-scope=\"accordion\"] h3 {\n  margin: 0;\n  font-size: 1rem;\n  font-weight: 400;\n  line-height: 1.5;\n  letter-spacing: normal;\n}\n\
.pre-styled-showcase [data-scope=\"dialog\"][data-part=\"backdrop\"],\n.pre-styled-showcase [data-scope=\"drawer\"][data-part=\"backdrop\"] {\n  display: none;\n}\n\
.pre-styled-showcase [data-scope=\"dialog\"][data-part=\"positioner\"] {\n  position: static;\n  padding: 0;\n  justify-content: flex-start;\n}\n\
.pre-styled-showcase [data-scope=\"drawer\"][data-part=\"positioner\"] {\n  position: static;\n}\n\
.pre-styled-showcase [data-scope=\"menu\"][data-part=\"positioner\"],\n.pre-styled-showcase [data-scope=\"menubar\"][data-part=\"positioner\"],\n.pre-styled-showcase [data-scope=\"select\"][data-part=\"positioner\"],\n.pre-styled-showcase [data-scope=\"combobox\"][data-part=\"positioner\"],\n.pre-styled-showcase [data-scope=\"popover\"][data-part=\"positioner\"],\n.pre-styled-showcase [data-scope=\"tooltip\"][data-part=\"positioner\"],\n.pre-styled-showcase [data-scope=\"hover-card\"][data-part=\"positioner\"],\n.pre-styled-showcase [data-scope=\"toggle-tip\"][data-part=\"positioner\"] {\n  position: static;\n}\n\
.pre-styled-showcase [data-scope=\"menubar\"][data-part=\"root\"] {\n  align-items: flex-start;\n}\n\
.pre-styled-showcase [data-scope=\"action-bar\"][data-part=\"positioner\"] {\n  position: static;\n  transform: none;\n}\n\
.pre-styled-showcase [data-scope=\"floating-panel\"][data-part=\"positioner\"] {\n  position: static;\n  transform: none;\n  z-index: auto;\n}\n\
.pre-styled-showcase [data-scope=\"dialog\"] h2,\n.pre-styled-showcase [data-scope=\"drawer\"] h2,\n.pre-styled-showcase [data-scope=\"popover\"] h2,\n.pre-styled-showcase [data-scope=\"floating-panel\"] h2,\n.pre-styled-showcase [data-scope=\"tour\"] h2 {\n  border-top: none;\n  padding-top: 0;\n  letter-spacing: normal;\n}\n\
.pre-styled-showcase [data-scope=\"toast\"][data-part=\"group\"] {\n  position: static;\n}\n\
.pre-styled-showcase [data-scope=\"blockquote\"][data-part=\"content\"] {\n  padding: 0;\n  border-left: none;\n  color: inherit;\n}\n\
.pre-styled-showcase [data-scope=\"navigation-menu\"][data-part=\"content\"] {\n  position: static;\n}\n\
.pre-styled-showcase [data-scope=\"tour\"][data-part=\"backdrop\"],\n.pre-styled-showcase [data-scope=\"tour\"][data-part=\"spotlight\"] {\n  display: none;\n}\n\
.pre-styled-showcase [data-scope=\"tour\"][data-part=\"positioner\"] {\n  position: static;\n  transform: none;\n  z-index: auto;\n}\n\
.pre-styled-showcase [data-scope=\"link-overlay\"][data-part=\"root\"] {\n  position: relative;\n}\n\
.pre-styled-showcase [data-scope=\"link-overlay\"][data-part=\"overlay\"] {\n  position: absolute;\n  inset: 0;\n  z-index: 0;\n  border-radius: inherit;\n  cursor: pointer;\n}\n\
.pre-styled-showcase [data-scope=\"link-overlay\"][data-part=\"root\"] {\n  border-radius: inherit;\n}\n\
.pre-styled-showcase [data-scope=\"link-overlay\"][data-part=\"overlay\"]:focus-visible {\n  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));\n  outline-offset: var(--fandhe-focus-ring-offset, 2px);\n}\n\
.pre-styled-showcase [data-scope=\"link-overlay\"][data-part=\"root\"] h3 {\n  margin-top: 0;\n}\n\
.pre-styled-showcase [data-scope=\"nav-list\"][data-part=\"heading\"] {\n  border-top: none;\n  padding-top: 0;\n  letter-spacing: normal;\n}\n\
.pre-styled-showcase [data-scope=\"link\"][data-part=\"root\"]:hover {\n  text-decoration: var(--fandhe-link-text-decoration, none);\n}\n\
.pre-styled-showcase [data-scope=\"nav-list\"][data-part=\"link\"]:hover {\n  text-decoration: none;\n}\n";

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
    /// `/themes/<kebab-name>/`。`site/nav.toml` の `page.path`（#943 で
    /// 登録）と一致させる契約で、`nav::validate_page_path` のセグメント
    /// allowlist（英数・`-`・`_`、`/` 始まり `/` 終わり）を満たす。
    path: &'static str,
    /// 当該部品のデモ節（[`section`] が返す `<section>` 1 件）を生成する。
    render: fn() -> Node,
}

/// 部品ページのレジストリ本体（旧集約ページの並び順を保つ。
/// イシュー #991 で Toolbar・#992 で Menubar・#993 で Navigation Menu を
/// 追加。件数は `tests::component_page_paths_are_unique_and_well_formed`
/// 参照）。
/// 原則として `docs/design/docs-site-component-pages.md` の台帳に掲載済み
/// の部品のみを登録し、掲載順はテスト専用集約ヘルパーの表示順にのみ効く
/// （#943 の nav 上の並びはカテゴリ別で、本テーブルの順序に依存しない）。
const COMPONENT_PAGES: &[ComponentPage] = &[
    ComponentPage {
        path: "/themes/button/",
        render: button_section,
    },
    ComponentPage {
        path: "/themes/download-trigger/",
        render: download_trigger_section,
    },
    ComponentPage {
        path: "/themes/badge/",
        render: badge_section,
    },
    ComponentPage {
        path: "/themes/spinner/",
        render: spinner_section,
    },
    ComponentPage {
        path: "/themes/skeleton/",
        render: skeleton_section,
    },
    ComponentPage {
        path: "/themes/heading/",
        render: heading_section,
    },
    ComponentPage {
        path: "/themes/text/",
        render: text_section,
    },
    ComponentPage {
        path: "/themes/em/",
        render: em_section,
    },
    ComponentPage {
        path: "/themes/mark/",
        render: mark_section,
    },
    ComponentPage {
        path: "/themes/quote/",
        render: quote_section,
    },
    ComponentPage {
        path: "/themes/strong/",
        render: strong_section,
    },
    ComponentPage {
        path: "/themes/blockquote/",
        render: blockquote_section,
    },
    ComponentPage {
        path: "/themes/list/",
        render: list_section,
    },
    ComponentPage {
        path: "/themes/separator/",
        render: separator_section,
    },
    ComponentPage {
        path: "/themes/highlight/",
        render: highlight_section,
    },
    ComponentPage {
        path: "/themes/alert/",
        render: alert_section,
    },
    ComponentPage {
        path: "/themes/callout/",
        render: callout_section,
    },
    ComponentPage {
        path: "/themes/card/",
        render: card_section,
    },
    ComponentPage {
        path: "/themes/tabs/",
        render: tabs_section,
    },
    ComponentPage {
        path: "/themes/accordion/",
        render: accordion_section,
    },
    ComponentPage {
        path: "/themes/dialog/",
        render: dialog_section,
    },
    ComponentPage {
        path: "/themes/drawer/",
        render: drawer_section,
    },
    ComponentPage {
        path: "/themes/menu/",
        render: menu_section,
    },
    ComponentPage {
        path: "/themes/select/",
        render: select_section,
    },
    ComponentPage {
        path: "/themes/listbox/",
        render: listbox_section,
    },
    ComponentPage {
        path: "/themes/combobox/",
        render: combobox_section,
    },
    ComponentPage {
        path: "/themes/popover/",
        render: popover_section,
    },
    ComponentPage {
        path: "/themes/floating-panel/",
        render: floating_panel_section,
    },
    ComponentPage {
        path: "/themes/tooltip/",
        render: tooltip_section,
    },
    ComponentPage {
        path: "/themes/hover-card/",
        render: hover_card_section,
    },
    ComponentPage {
        path: "/themes/toggle-tip/",
        render: toggle_tip_section,
    },
    ComponentPage {
        path: "/themes/switch/",
        render: switch_section,
    },
    ComponentPage {
        path: "/themes/radio-group/",
        render: radio_group_section,
    },
    ComponentPage {
        path: "/themes/avatar/",
        render: avatar_section,
    },
    ComponentPage {
        path: "/themes/checkbox/",
        render: checkbox_section,
    },
    ComponentPage {
        path: "/themes/input/",
        render: input_section,
    },
    ComponentPage {
        path: "/themes/textarea/",
        render: textarea_section,
    },
    ComponentPage {
        path: "/themes/native-select/",
        render: native_select_section,
    },
    ComponentPage {
        path: "/themes/number-input/",
        render: number_input_section,
    },
    ComponentPage {
        path: "/themes/password-input/",
        render: password_input_section,
    },
    ComponentPage {
        path: "/themes/tags-input/",
        render: tags_input_section,
    },
    ComponentPage {
        path: "/themes/file-upload/",
        render: file_upload_section,
    },
    ComponentPage {
        path: "/themes/rating-group/",
        render: rating_group_section,
    },
    ComponentPage {
        path: "/themes/slider/",
        render: slider_section,
    },
    ComponentPage {
        path: "/themes/editable/",
        render: editable_section,
    },
    ComponentPage {
        path: "/themes/segment-group/",
        render: segment_group_section,
    },
    ComponentPage {
        path: "/themes/toggle/",
        render: toggle_section,
    },
    ComponentPage {
        path: "/themes/toggle-group/",
        render: toggle_group_section,
    },
    ComponentPage {
        path: "/themes/carousel/",
        render: carousel_section,
    },
    ComponentPage {
        path: "/themes/tree-view/",
        render: tree_view_section,
    },
    ComponentPage {
        path: "/themes/json-tree-view/",
        render: json_tree_view_section,
    },
    ComponentPage {
        path: "/themes/pagination/",
        render: pagination_section,
    },
    ComponentPage {
        path: "/themes/steps/",
        render: steps_section,
    },
    ComponentPage {
        path: "/themes/tour/",
        render: tour_section,
    },
    ComponentPage {
        path: "/themes/splitter/",
        render: splitter_section,
    },
    ComponentPage {
        path: "/themes/checkbox-card/",
        render: checkbox_card_section,
    },
    ComponentPage {
        path: "/themes/checkbox-group/",
        render: checkbox_group_section,
    },
    ComponentPage {
        path: "/themes/radio-card/",
        render: radio_card_section,
    },
    ComponentPage {
        path: "/themes/breadcrumb/",
        render: breadcrumb_section,
    },
    ComponentPage {
        path: "/themes/action-bar/",
        render: action_bar_section,
    },
    ComponentPage {
        path: "/themes/toast/",
        render: toast_section,
    },
    ComponentPage {
        path: "/themes/progress/",
        render: progress_section,
    },
    ComponentPage {
        path: "/themes/image/",
        render: image_section,
    },
    ComponentPage {
        path: "/themes/icon/",
        render: icon_section,
    },
    ComponentPage {
        path: "/themes/tag/",
        render: tag_section,
    },
    ComponentPage {
        path: "/themes/kbd/",
        render: kbd_section,
    },
    ComponentPage {
        path: "/themes/code/",
        render: code_section,
    },
    ComponentPage {
        path: "/themes/color-swatch/",
        render: color_swatch_section,
    },
    ComponentPage {
        path: "/themes/color-picker/",
        render: color_picker_section,
    },
    ComponentPage {
        path: "/themes/status/",
        render: status_section,
    },
    ComponentPage {
        path: "/themes/empty-state/",
        render: empty_state_section,
    },
    ComponentPage {
        path: "/themes/visually-hidden/",
        render: visually_hidden_section,
    },
    ComponentPage {
        path: "/themes/qr-code/",
        render: qr_code_section,
    },
    ComponentPage {
        path: "/themes/table/",
        render: table_section,
    },
    ComponentPage {
        path: "/themes/data-list/",
        render: data_list_section,
    },
    ComponentPage {
        path: "/themes/stat/",
        render: stat_section,
    },
    ComponentPage {
        path: "/themes/timeline/",
        render: timeline_section,
    },
    ComponentPage {
        path: "/themes/marquee/",
        render: marquee_section,
    },
    ComponentPage {
        path: "/themes/scroll-area/",
        render: scroll_area_section,
    },
    ComponentPage {
        path: "/themes/calendar/",
        render: calendar_section,
    },
    ComponentPage {
        path: "/themes/date-picker/",
        render: date_picker_section,
    },
    ComponentPage {
        path: "/themes/date-input/",
        render: date_input_section,
    },
    ComponentPage {
        path: "/themes/timer/",
        render: timer_section,
    },
    ComponentPage {
        path: "/themes/charts/",
        render: charts_section,
    },
    ComponentPage {
        path: "/themes/bar-chart/",
        render: bar_chart_section,
    },
    ComponentPage {
        path: "/themes/bar-list/",
        render: bar_list_section,
    },
    ComponentPage {
        path: "/themes/bar-segment/",
        render: bar_segment_section,
    },
    ComponentPage {
        path: "/themes/line-chart/",
        render: line_chart_section,
    },
    ComponentPage {
        path: "/themes/area-chart/",
        render: area_chart_section,
    },
    ComponentPage {
        path: "/themes/sparkline/",
        render: sparkline_section,
    },
    ComponentPage {
        path: "/themes/pie-chart/",
        render: pie_chart_section,
    },
    ComponentPage {
        path: "/themes/donut-chart/",
        render: donut_chart_section,
    },
    ComponentPage {
        path: "/themes/scatter-chart/",
        render: scatter_chart_section,
    },
    ComponentPage {
        path: "/themes/radar-chart/",
        render: radar_chart_section,
    },
    ComponentPage {
        path: "/themes/toolbar/",
        render: toolbar_section,
    },
    ComponentPage {
        path: "/themes/menubar/",
        render: menubar_section,
    },
    ComponentPage {
        path: "/themes/navigation-menu/",
        render: navigation_menu_section,
    },
    ComponentPage {
        path: "/themes/tab-nav/",
        render: tab_nav_section,
    },
    ComponentPage {
        path: "/themes/link/",
        render: link_section,
    },
    ComponentPage {
        path: "/themes/link-overlay/",
        render: link_overlay_section,
    },
    ComponentPage {
        path: "/themes/nav-list/",
        render: nav_list_section,
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
/// `site/themes.md` 側の Markdown のみで完結する）。
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
/// callout/card/tabs/accordion/dialog/drawer/menu/select/combobox/popover/tooltip/
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
    sheet.push_css(&fandhe_frontend_pre_styled_ui::callout::css())?;
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
    sheet.push_css(&fandhe_frontend_pre_styled_ui::checkbox_group::stylesheet())?;
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
    sheet.push_css(&fandhe_frontend_pre_styled_ui::quote::css())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::strong::css())?;
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
    sheet.push_css(&fandhe_frontend_pre_styled_ui::navigation_menu::stylesheet())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::tab_nav::stylesheet())?;
    sheet.push_css(&fandhe_frontend_pre_styled_ui::link::stylesheet())?;
    // Link Overlay（イシュー #1154）: `link_overlay::stylesheet()` を
    // ここで無条件出荷しない（Bugbot 指摘、PR #1165 review comment
    // 3697116537）。当該スタイルシートは
    // `[data-scope="link-overlay"][data-part="overlay"]` に
    // `position: absolute; inset: 0;` を無条件（`.pre-styled-showcase`
    // スコープなし）に適用する。`crate::nav::prev_next_nav`（サイト共通
    // クロームの前後ページナビゲーション）は同じ headless マーカー
    // （overlay を唯一の子要素として使用するカード）を再利用しており、
    // `crate::site_theme` はまさにこの理由で当該スタイルシートを意図的に
    // 除外している（`site_theme` 冒頭コメント・
    // `stylesheet_never_takes_up_link_overlay_stylesheet` 参照）。Themes
    // 部品ページ共有 CSS（`assets/pre-styled-ui.css`）は `nav.prev-next`
    // を含む全ページへ配信されるため、無条件出荷すると同じ回帰
    // （prev/next ナビが高さ 0 に潰れる）が起きる。デモの見た目に必要な
    // 等価ルールは [`SHOWCASE_LAYOUT_CSS`] 側で `.pre-styled-showcase`
    // スコープ付きとして個別に持つ（下記）。イシュー #1580 で `recipe()`
    // へ追加した `border-radius: inherit`（root/overlay 双方）・
    // `cursor: pointer`・`:focus-visible` リングも同じ理由で scoped
    // 複製する（並行 PR #1852 とのマージで統合した内容を反映）。
    sheet.push_css(&fandhe_frontend_pre_styled_ui::nav_list::stylesheet())?;
    // Clipboard（イシュー #1155）: showcase.rs の COMPONENT_PAGES には未登録
    // だが、`crate::component_specs::interactive_utilities::demo_clipboard`
    // が Demo フォールバック（`ComponentPageSpec::demo`、#979）経由で
    // `data-scope="clipboard"` マークアップを `/themes/clipboard/` へ出荷
    // するため、対応 CSS もここへ出荷する（#979 Bugbot 指摘の再発防止:
    // Demo マークアップと出荷 CSS を必ず対で追加する）。SkipNav は
    // `crate::skip_nav::stylesheet()` が `assets/skip-nav.css` として全ページ
    // 無条件で既に出荷しているため、ここへは追加しない（二重出荷回避）。
    sheet.push_css(&fandhe_frontend_pre_styled_ui::clipboard::stylesheet())?;
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
        (ButtonVariant::Surface, "Surface"),
        (ButtonVariant::Plain, "Plain"),
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

    // イシュー #1449: size variant を 5 段（Xs〜Xl）へ拡張したことに伴い、
    // Demo でも Xs/Xl を含む全段を掲示する（従来は Sm/Md/Lg の 3 段のみ）。
    let sizes = [
        (Size::Xs, "Extra Small"),
        (Size::Sm, "Small"),
        (Size::Md, "Medium"),
        (Size::Lg, "Large"),
        (Size::Xl, "Extra Large"),
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
        (ButtonVariant::Surface, "Surface"),
        (ButtonVariant::Plain, "Plain"),
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

    // イシュー #1474: Button 側（#1449）の size variant 5 段化に対称して
    // Xs/Xl を含む全段を掲示する（従来は Sm/Md/Lg の 3 段のみで非対称
    // だった。本モジュール冒頭 rustdoc「デモの構成は button_section と
    // 対称に揃える」契約の是正）。
    let sizes = [
        (Size::Xs, "Extra Small"),
        (Size::Sm, "Small"),
        (Size::Md, "Medium"),
        (Size::Lg, "Large"),
        (Size::Xl, "Extra Large"),
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

/// Badge 節: variant × palette × size。
fn badge_section() -> Node {
    let variants = [
        (BadgeVariant::Solid, "Solid"),
        (BadgeVariant::Subtle, "Subtle"),
        (BadgeVariant::Outline, "Outline"),
        (BadgeVariant::Surface, "Surface"),
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
    // イシュー #1681: 共有 `palettes()`（5 値）はまだ Neutral を含めない
    // （#1680 適用完了まで宣言なしデモの公開を避ける）。この節限定で
    // Neutral エントリを末尾へ連結する。
    let palette_row = row(palettes()
        .iter()
        .copied()
        .chain([(ColorPalette::Neutral, "Neutral")])
        .map(|(palette, label)| {
            badge::badge(
                &BadgeProps {
                    palette,
                    ..BadgeProps::default()
                },
                vec![],
                vec![text(label)],
            )
        })
        .collect());
    // イシュー #1555: size 5 段を目視確認できるようにする（spinner_section
    // と同型）。
    let sizes = [
        (Size::Xs, "Xs"),
        (Size::Sm, "Sm"),
        (Size::Md, "Md"),
        (Size::Lg, "Lg"),
        (Size::Xl, "Xl"),
    ];
    let size_row = row(sizes
        .iter()
        .map(|(size, label)| {
            badge::badge(
                &BadgeProps {
                    size: *size,
                    ..BadgeProps::default()
                },
                vec![],
                vec![text(*label)],
            )
        })
        .collect());
    section(
        "Badge",
        "ステータス表示向けの小型ラベル。variant と colorPalette、size を組み合わせます。",
        vec![variant_row, palette_row, size_row],
    )
}

/// Spinner 節（イシュー #1567 で palette 行を追加）: size・colorPalette
/// バリエーション。
fn spinner_section() -> Node {
    let sizes = [
        (Size::Xs, "Loading (xs)"),
        (Size::Sm, "Loading (small)"),
        (Size::Md, "Loading (medium)"),
        (Size::Lg, "Loading (large)"),
        (Size::Xl, "Loading (xl)"),
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
    // イシュー #1567: chakra-ui docs の色グリッド相当。トラック透明既定・
    // 上右 2 辺の弧が palette ごとに区別できることを目視確認できるように
    // する（badge_section の palette_row と同型に Neutral を末尾連結）。
    let palette_row = row(palettes()
        .iter()
        .copied()
        .chain([(ColorPalette::Neutral, "Neutral")])
        .map(|(palette, label)| {
            spinner(&SpinnerProps {
                palette,
                label,
                ..SpinnerProps::default()
            })
        })
        .collect());
    section(
        "Spinner",
        "読み込み中表示。role=\"status\" と aria-label でスクリーンリーダーへ状態を伝えます。トラックは既定で透明（上右 2 辺のみ弧を描画）で、OS の prefers-reduced-motion 設定時は回転を停止します。",
        vec![size_row, palette_row],
    )
}

/// Skeleton 節（イシュー #764、イシュー #1566 で `animation` 軸の行と
/// 複合デモ行を追加）: variant（text/circle/rect）× animation
/// （pulse/shine/none）バリエーション。
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
                &SkeletonProps {
                    variant: *variant,
                    ..Default::default()
                },
                if style.is_empty() {
                    vec![]
                } else {
                    vec![("style", *style)]
                },
            )
        })
        .collect());
    // イシュー #1566: 第 2 軸 `animation`（pulse/shine/none）を text
    // variant 上で並べ、参照サイト（chakra-ui）の 3 種アニメーションの
    // 見た目差を確認できるようにする。
    let animations = [
        SkeletonAnimation::Pulse,
        SkeletonAnimation::Shine,
        SkeletonAnimation::None,
    ];
    let animation_row = row(animations
        .iter()
        .map(|animation| {
            skeleton(
                &SkeletonProps {
                    animation: *animation,
                    ..Default::default()
                },
                vec![("style", "width: 12rem;")],
            )
        })
        .collect());
    // イシュー #1566: 参照スクショ（chakra-ui Skeleton pulse スクショ 1）の
    // ように circle + text 2 本を横並びで組み合わせる複合デモ。
    let composite_row = row(vec![div(
        vec![("style", "display: flex; gap: 1rem; align-items: center;")],
        vec![
            skeleton(
                &SkeletonProps {
                    variant: SkeletonVariant::Circle,
                    ..Default::default()
                },
                vec![("style", "--fandhe-skeleton-size: 3rem;")],
            ),
            div(
                vec![(
                    "style",
                    "display: flex; flex-direction: column; gap: 0.5rem;",
                )],
                vec![
                    skeleton(&SkeletonProps::default(), vec![("style", "width: 12rem;")]),
                    skeleton(&SkeletonProps::default(), vec![("style", "width: 8rem;")]),
                ],
            ),
        ],
    )]);
    section(
        "Skeleton",
        "データ読み込み中のコンテンツ形状を模した占位要素。常に aria-hidden=\"true\" を持ち、読み込み中であることをスクリーンリーダーへ伝える責務はコンテナ側（aria-busy）にあります。animation 軸（pulse/shine/none）でアニメーション種別を切り替えられ、prefers-reduced-motion: reduce ではいずれも停止します。",
        vec![variant_row, animation_row, composite_row],
    )
}

// タイポグラフィ節群（イシュー #771 で導入、#941 で複合節 typography_section
// から Heading/Text/Em/Mark/Blockquote/List の 6 部品ページ相当の関数へ分解。
// #995 で Quote/Strong の 2 部品ページ相当の関数を追加）。
// 素の HTML 意味論（h1〜h6・p・em・mark・blockquote・ul/ol/li）をそのまま
// styled 化する方針は変わらないが、記事全体へのカスケード適用（chakra-ui の
// Prose 相当）は本クレートへ導入せず、docs サイト骨格スタイル
// （`.docs-content`、`crate::site_theme` によるビルド時生成）が引き続き担う
// （`docs/design/docs-site-component-pages.md` §3・§4 の「1 ページ = 部品
// 1 件」方針）。Heading は `h4`〜`h6`（`.docs-content` 見出し規則が対象と
// する `h1`〜`h3` の範囲外）のみを掲示し、サイト骨格の見出しスタイルとの
// 衝突を避ける（本節自体の `h2` はショーケース節見出し〔[`section`] ヘルパ〕
// であり対象外）。各関数はこの前提のもとで 1〜2 文の部品固有説明のみを持つ。

/// Heading 節: `size` スケール全 8 段（`xs`〜`xl4`）を縦積み掲示する
/// （イシュー #1434。chakra-ui のサイズデモ（`docs/design/
/// reference-screenshots/chakra-heading-2.png`、sm〜6xl を縦積み掲示）と
/// 視覚比較できる状態にするため、意味論タグ（h1〜h6）とは独立に単一タグ
/// で size 軸のみを掲示する。タグと視覚サイズの独立性自体は
/// [`fandhe_frontend_pre_styled_ui::heading`] のモジュール rustdoc
/// 「意味論レベルと視覚サイズの独立」節で説明済み。タグは `h2` ではなく
/// `h4` を使う（イシュー #1434 の codex-review/Bugbot 指摘）: 本ファイル
/// 冒頭の節コメントが明記するとおり `.docs-content` 見出し規則は
/// `h1`〜`h3` を対象とし、`border-top`/`padding-top` 等のサイト骨格装飾を
/// 付与する。デモを `h2` にすると全 8 段へこの装飾が混入しサイズ比較の
/// 見た目を汚すため、装飾対象外の `h4` で統一する）。
fn heading_section() -> Node {
    let heading_stack = stack(
        [
            HeadingSize::Xs,
            HeadingSize::Sm,
            HeadingSize::Md,
            HeadingSize::Lg,
            HeadingSize::Xl,
            HeadingSize::Xl2,
            HeadingSize::Xl3,
            HeadingSize::Xl4,
        ]
        .iter()
        .map(|size| {
            heading(
                HeadingLevel::H4,
                &HeadingProps { size: *size },
                vec![],
                vec![text(format!("見出し（size={size:?}）"))],
            )
        })
        .collect(),
    );

    section(
        "Heading",
        "素の h1〜h6 意味論を size（xs〜xl4 の 8 段階）でスタイル化した見出し部品。",
        vec![heading_stack],
    )
}

/// Text 節: size（xs〜xl4 の 8 段階）・weight（normal/medium/semibold/bold
/// の 4 段階）でスタイル化した本文テキスト（イシュー #1442 で拡充）。
fn text_section() -> Node {
    let size_stack = stack(
        [
            TextSize::Xs,
            TextSize::Sm,
            TextSize::Md,
            TextSize::Lg,
            TextSize::Xl,
            TextSize::Xl2,
            TextSize::Xl3,
            TextSize::Xl4,
        ]
        .iter()
        .map(|size| {
            styled_text(
                &TextProps {
                    size: *size,
                    ..TextProps::default()
                },
                vec![],
                vec![text(format!("本文テキスト（size={size:?}）"))],
            )
        })
        .collect(),
    );

    let weight_stack = stack(
        [
            TextWeight::Normal,
            TextWeight::Medium,
            TextWeight::Semibold,
            TextWeight::Bold,
        ]
        .iter()
        .map(|weight| {
            styled_text(
                &TextProps {
                    weight: *weight,
                    ..TextProps::default()
                },
                vec![],
                vec![text(format!("本文テキスト（weight={weight:?}）"))],
            )
        })
        .collect(),
    );

    section(
        "Text",
        "素の p 要素を size（xs〜xl4 の 8 段階）・weight（normal/medium/semibold/bold の 4 段階）でスタイル化した本文テキスト部品。",
        vec![size_stack, weight_stack],
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

/// Quote 節: 素の `<q>` による短いインライン引用（イシュー #995）。
/// ブロックレベルの引用は [`blockquote_section`] が担う。
fn quote_section() -> Node {
    let quote_row = row(vec![el(
        "p",
        vec![],
        vec![
            text("彼はこう言った、"),
            quote(vec![], vec![text("為せば成る")]),
            text("と。"),
        ],
    )]);

    section(
        "Quote",
        "素の q 要素をそのまま styled 化した短いインライン引用部品。",
        vec![quote_row],
    )
}

/// Strong 節: 素の `<strong>` による重要性の強調（イシュー #995）。
/// 文法的な強勢の強調は [`em_section`] が担う。
fn strong_section() -> Node {
    let strong_row = row(vec![el(
        "p",
        vec![],
        vec![
            text("この操作は"),
            strong(vec![], vec![text("元に戻せません")]),
            text("。"),
        ],
    )]);

    section(
        "Strong",
        "素の strong 要素をそのまま styled 化した重要性の強調テキスト部品。",
        vec![strong_row],
    )
}

/// Blockquote 節: 3 variant（Subtle/Solid/Plain）の引用ブロック
/// （content + caption）を縦積みで並置する（イシュー #1431 の視覚比較
/// 対象を Demo 上でも確認できるようにする）。
fn blockquote_section() -> Node {
    let make = |variant: BlockquoteVariant, label: &'static str| {
        blockquote::root(
            variant,
            ColorPalette::Accent,
            vec![],
            vec![
                blockquote::content(
                    vec![],
                    vec![text("プレーンな HTML / JavaScript / CSS を尊重する。")],
                ),
                blockquote::caption(vec![], vec![text(label)]),
            ],
        )
    };

    let blockquote_stack = stack(vec![
        make(BlockquoteVariant::Subtle, "— subtle（既定）"),
        make(BlockquoteVariant::Solid, "— solid"),
        make(BlockquoteVariant::Plain, "— plain"),
    ]);

    section(
        "Blockquote",
        "素の blockquote 要素を content/caption の 2 パーツで styled 化した引用部品。variant（subtle/solid/plain）3 種。",
        vec![blockquote_stack],
    )
}

/// List 節: 順序なし（marker variant）・順序あり・plain + indicator の 3 種。
///
/// イシュー #1438: 3 つ目（`ListVariant::Plain` + [`list::indicator`]）は
/// 参照サイト是正（indicator の間隔・整列・`Plain` variant の item
/// 整列）を Demo 上で視覚確認できるようにするための追加。
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
    let plain_list_with_indicator = list::root(
        ListType::Unordered,
        ListVariant::Plain,
        vec![],
        vec![
            list::item(
                vec![],
                vec![
                    list::indicator(vec![], vec![text("✓")]),
                    text("既定エスケープ"),
                ],
            ),
            list::item(
                vec![],
                vec![
                    list::indicator(vec![], vec![text("✓")]),
                    text("forbid(unsafe_code)"),
                ],
            ),
        ],
    );

    section(
        "List",
        "素の ul/ol/li 意味論をそのまま styled 化したリスト部品。順序なし（marker variant）・順序あり・plain + indicator（カスタムマーカー）の 3 種。",
        vec![stack(vec![
            marker_list,
            ordered_list,
            plain_list_with_indicator,
        ])],
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

/// Highlight 節（イシュー #775、イシュー #1435 で variant/palette 軸を追加）:
/// 単一一致・複数一致（`match_all`）・`ignore_case`・variant・colorPalette の
/// 実演。一致判定は正規表現を使わない決定的な部分文字列検索
/// （`crates/pre-styled-ui/src/highlight.rs` rustdoc 参照）。
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
    let variants = [
        (HighlightVariant::Subtle, "subtle"),
        (HighlightVariant::Solid, "solid"),
        (HighlightVariant::Text, "text"),
        (HighlightVariant::Plain, "plain"),
    ];
    let variant_row = row(variants
        .iter()
        .map(|(variant, label)| {
            highlight(
                &HighlightProps {
                    query: &[*label],
                    variant: *variant,
                    ..HighlightProps::default()
                },
                vec![],
                label,
            )
        })
        .collect());
    // 共有 `palettes()`（5 値、Neutral なし）に本部品の既定 palette
    // （Accent、mark::section と同様）以外を末尾連結する（badge/code と同型）。
    let palette_row = row(palettes()
        .iter()
        .copied()
        .chain([(ColorPalette::Neutral, "Neutral")])
        .map(|(palette, label)| {
            highlight(
                &HighlightProps {
                    query: &[label],
                    palette,
                    ..HighlightProps::default()
                },
                vec![],
                label,
            )
        })
        .collect());
    section(
        "Highlight",
        "テキスト中の一致語句を <mark> で強調します。正規表現ではなく決定的な部分文字列検索のみで一致判定します。query（複数可）・match_all（全一致 or 最初の 1 件）・ignore_case（ASCII 限定）・variant（subtle/solid/text/plain）・colorPalette の各プロパティを持ちます。",
        vec![
            single_match_row,
            match_all_row,
            ignore_case_row,
            variant_row,
            palette_row,
        ],
    )
}

/// Alert 節: status（info / success / warning / error / neutral）・variant
/// （subtle / surface / solid / outline）・size（xs〜xl）ごとの表示
/// （イシュー #1553 で variant/size 軸を追加）。
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
        (AlertStatus::Neutral, "Neutral", "中立な状態の通知です。"),
    ];
    let status_row = stack(
        statuses
            .iter()
            .map(|(status, title, description)| {
                let props = AlertProps {
                    status: *status,
                    ..AlertProps::default()
                };
                alert::root(
                    &props,
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
    let variants = [
        (AlertVariant::Subtle, "Subtle"),
        (AlertVariant::Surface, "Surface"),
        (AlertVariant::Solid, "Solid"),
        (AlertVariant::Outline, "Outline"),
    ];
    let variant_row = stack(
        variants
            .iter()
            .map(|(variant, label)| {
                let props = AlertProps {
                    variant: *variant,
                    ..AlertProps::default()
                };
                alert::root(
                    &props,
                    vec![],
                    vec![
                        alert::indicator(vec![], vec![text("!")]),
                        alert::content(vec![], vec![alert::title(vec![], vec![text(*label)])]),
                    ],
                )
            })
            .collect(),
    );
    let sizes = [
        (Size::Xs, "Xs"),
        (Size::Sm, "Sm"),
        (Size::Md, "Md"),
        (Size::Lg, "Lg"),
        (Size::Xl, "Xl"),
    ];
    let size_row = stack(
        sizes
            .iter()
            .map(|(size, label)| {
                let props = AlertProps {
                    size: *size,
                    ..AlertProps::default()
                };
                alert::root(
                    &props,
                    vec![],
                    vec![
                        alert::indicator(vec![], vec![text("!")]),
                        alert::content(vec![], vec![alert::title(vec![], vec![text(*label)])]),
                    ],
                )
            })
            .collect(),
    );
    section(
        "Alert",
        "status（info / success / warning / error / neutral）・variant（subtle / surface / solid / outline）・size（xs〜xl）で見た目が切り替わる通知領域。root / indicator / content / title / description の slot 構成です。",
        vec![status_row, variant_row, size_row],
    )
}

/// Callout 節: variant（soft / surface / outline）と colorPalette の
/// デモ。`alert` と異なり `role="alert"` を付与しない（module doc 参照）。
fn callout_section() -> Node {
    let variants = [
        (CalloutVariant::Soft, "Soft"),
        (CalloutVariant::Surface, "Surface"),
        (CalloutVariant::Outline, "Outline"),
    ];
    let variant_row = stack(
        variants
            .iter()
            .map(|(variant, label)| {
                let props = CalloutProps {
                    variant: *variant,
                    ..CalloutProps::default()
                };
                callout::root(
                    &props,
                    vec![],
                    vec![
                        callout::icon(vec![], vec![text("i")]),
                        callout::text(
                            vec![],
                            vec![text(format!(
                                "{label} variant の補足情報です（本文中の強調表示）。"
                            ))],
                        ),
                    ],
                )
            })
            .collect(),
    );
    // イシュー #1681: 共有 `palettes()`（5 値）はまだ Neutral を含めない
    // （#1680 適用完了まで宣言なしデモの公開を避ける）。この節限定で
    // Neutral エントリを末尾へ連結する。
    let palette_row = stack(
        palettes()
            .iter()
            .copied()
            .chain([(ColorPalette::Neutral, "Neutral")])
            .map(|(palette, label)| {
                let props = CalloutProps {
                    palette,
                    ..CalloutProps::default()
                };
                callout::root(
                    &props,
                    vec![],
                    vec![
                        callout::icon(vec![], vec![text("i")]),
                        callout::text(vec![], vec![text(label)]),
                    ],
                )
            })
            .collect(),
    );
    let sizes = [
        (Size::Xs, "Xs"),
        (Size::Sm, "Sm"),
        (Size::Md, "Md"),
        (Size::Lg, "Lg"),
        (Size::Xl, "Xl"),
    ];
    let size_row = stack(
        sizes
            .iter()
            .map(|(size, label)| {
                let props = CalloutProps {
                    size: *size,
                    ..CalloutProps::default()
                };
                callout::root(
                    &props,
                    vec![],
                    vec![
                        callout::icon(vec![], vec![text("i")]),
                        callout::text(vec![], vec![text(*label)]),
                    ],
                )
            })
            .collect(),
    );
    section(
        "Callout",
        "本文フロー中に置く補足情報。alert と異なり live region ではなく role を付与しません。variant（soft / surface / outline）・colorPalette・size（xs〜xl、padding / gap / 角丸 / 文字サイズが連動）を組み合わせます。",
        vec![variant_row, palette_row, size_row],
    )
}

/// Card 節: variant（elevated / outline / subtle）ごとの表示。
fn card_section() -> Node {
    let variants = [
        (CardVariant::Elevated, "Elevated"),
        (CardVariant::Outline, "Outline"),
        (CardVariant::Subtle, "Subtle"),
    ];
    let variant_demo = |variant: CardVariant, label: &str| {
        let props = CardProps {
            variant,
            ..CardProps::default()
        };
        card::root(
            props,
            vec![],
            vec![
                card::header(
                    vec![],
                    vec![
                        card::title(vec![], vec![text(label)]),
                        card::description(vec![], vec![text("card variant のデモです。")]),
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
    };
    let demos = stack(
        variants
            .iter()
            .map(|(variant, label)| variant_demo(*variant, label))
            .collect(),
    );
    let sizes = [
        (Size::Xs, "Xs"),
        (Size::Sm, "Sm"),
        (Size::Md, "Md"),
        (Size::Lg, "Lg"),
        (Size::Xl, "Xl"),
    ];
    let size_row = stack(
        sizes
            .iter()
            .map(|(size, label)| {
                let props = CardProps {
                    size: *size,
                    ..CardProps::default()
                };
                card::root(
                    props,
                    vec![],
                    vec![
                        card::header(vec![], vec![card::title(vec![], vec![text(*label)])]),
                        card::body(vec![], vec![text("size デモです。")]),
                    ],
                )
            })
            .collect(),
    );
    section(
        "Card",
        "variant（elevated / outline / subtle）・size（xs〜xl、padding / 角丸 / title の文字サイズが連動）を持つ装飾的コンテナ。",
        vec![demos, size_row],
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
            // イシュー #1542: 参考サイト基準の是正で追加した `[data-disabled]`
            // スタイル（`opacity: 0.5`/`cursor: not-allowed`）を docs サイトの
            // Demo で目視確認できるようにする（hover/focus-visible/vertical
            // 対応も同 recipe 変更の対象だが、本 Demo は静的 SSR 掲示のため
            // 追加パーツを持たない disabled のみを追加した）。
            TabItem {
                value: "archived",
                trigger: vec![text("Archived")],
                content: vec![el(
                    "p",
                    vec![],
                    vec![text(
                        "無効化されたタブ（data-disabled）は半透明表示になります。",
                    )],
                )],
                disabled: true,
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
                        value,
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
                                // イシュー #1693: footer 相当のアクション配置例
                                // （headless anatomy に専用 footer パートが
                                // 存在しないため、description 直後に通常の
                                // 行として掲示する。`.showcase-row` は掲示用
                                // レイアウトのみを担い、製品 CSS には footer
                                // 規則を持ち込まない）。
                                div(
                                    vec![("class", "showcase-row")],
                                    vec![
                                        button(
                                            &ButtonProps {
                                                variant: ButtonVariant::Outline,
                                                ..ButtonProps::default()
                                            },
                                            vec![],
                                            vec![text("Cancel")],
                                        ),
                                        button(&ButtonProps::default(), vec![], vec![text("Save")]),
                                    ],
                                ),
                                // イシュー #1693: content 右上のゴーストボタン
                                // 化に伴い視覚はアイコンボタン化（`×`）。
                                // 支援技術向けラベルは `aria-label` で維持する。
                                dialog::close_trigger(
                                    vec![("aria-label", "Close")],
                                    vec![text("×")],
                                ),
                            ],
                        )],
                    ),
                ],
            ),
        ],
    );
    section(
        "Dialog",
        "headless-ui の Dialog（WAI-ARIA dialog パターン）に pre-styled-ui の data-scope / data-part セレクタ CSS を適用した静的掲示です。backdrop は掲示用に非表示化し、positioner はフロー内配置へ中和しています（実際の overlay 配置は recipe CSS が担います）。close-trigger は content 右上のゴーストボタン（× アイコン + aria-label）として掲示し、description の下にアクション行（footer 相当、掲示用レイアウトのみ）を配置しています。",
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
                                // イシュー #1695: footer 相当のアクション配置例
                                // （dialog #1693 と同型。headless anatomy に
                                // 専用 footer パートが存在しないため、
                                // description 直後に通常の行として掲示する。
                                // `.showcase-row` は掲示用レイアウトのみを
                                // 担い、製品 CSS には footer 規則を持ち込ま
                                // ない）。
                                div(
                                    vec![("class", "showcase-row")],
                                    vec![
                                        button(
                                            &ButtonProps {
                                                variant: ButtonVariant::Outline,
                                                ..ButtonProps::default()
                                            },
                                            vec![],
                                            vec![text("Cancel")],
                                        ),
                                        button(&ButtonProps::default(), vec![], vec![text("Save")]),
                                    ],
                                ),
                                // イシュー #1695: content 右上のゴーストボタン
                                // 化に伴い視覚はアイコンボタン化（`×`）。
                                // 支援技術向けラベルは `aria-label` で維持する
                                // （dialog #1693 と同型）。
                                drawer::close_trigger(
                                    vec![("aria-label", "Close")],
                                    vec![text("×")],
                                ),
                            ],
                        )],
                    ),
                ],
            ),
        ],
    );
    section(
        "Drawer",
        "headless-ui の Drawer（WAI-ARIA dialog パターンの変種、dialog の状態機械を再利用）に pre-styled-ui の data-scope / data-part セレクタ CSS を適用した静的掲示です。placement=\"end\" を掲示しています。backdrop は掲示用に非表示化し、positioner はフロー内配置へ中和しています。close-trigger は content 右上のゴーストボタン（× アイコン + aria-label）として掲示し、description の下にアクション行（footer 相当、掲示用レイアウトのみ）を配置しています。",
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
                        // "Other framework" を highlight 中の項目として固定
                        // する（イシュー #1502、item/item-indicator パートの
                        // 状態表現デモ。combobox 2/2 #1468 の先例に倣う）。
                        select::item(
                            OpenState::Closed,
                            false,
                            true,
                            "other",
                            None,
                            vec![],
                            vec![
                                select::item_text(None, vec![], vec![text("Other framework")]),
                                select::item_indicator(OpenState::Closed, vec![], vec![text("✓")]),
                            ],
                        ),
                        // disabled 項目の視覚状態デモ（イシュー #1502）。
                        select::item(
                            OpenState::Closed,
                            true,
                            false,
                            "legacy",
                            None,
                            vec![],
                            vec![
                                select::item_text(None, vec![], vec![text("Legacy framework")]),
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
        "headless-ui の Select（role=\"listbox\"）に pre-styled-ui の recipe CSS を適用した静的掲示です。1 項目が選択済み（data-state=\"open\"）・1 項目が highlight 中（data-highlighted）・1 項目が disabled（data-disabled）の listbox が開いた状態を固定表示しています。positioner はフロー内配置へ中和しています。",
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
/// （イシュー #749）。[`combobox::filter_options`] を実演し、入力値 `"e"`
/// に対するフィルタ結果（3 件全件、いずれも `"e"` を含む）をそのまま候補
/// として掲示する。イシュー #1468 で item の選択済み（`item_indicator` 付き
/// チェックマーク表示）・highlight（`data-highlighted`）・disabled の 3 状態
/// を 1 件ずつ割り当て、リスト側パーツの視覚表現を確認できるようにした
/// （デモ専用の恒久デモデータであり、選択・キーボード操作等の動作は伴わない
/// 静的掲示）。
fn combobox_section() -> Node {
    let options = [("vue", "Vue"), ("react", "React"), ("svelte", "Svelte")];
    let query = "e";
    let filtered = combobox::filter_options(&options, query);

    let items = filtered
        .into_iter()
        .map(|(value, label)| {
            // "react" を選択済み（item-indicator 表示）、"svelte" を
            // highlight 中、"vue" を disabled として固定する
            // （イシュー #1468、item/item-indicator パートの状態表現デモ）。
            let selected = value == "react";
            let highlighted = value == "svelte";
            let disabled = value == "vue";
            let selected_state = if selected {
                OpenState::Open
            } else {
                OpenState::Closed
            };
            // R3（`crates/docs-site/tests/combobox_aria_association.rs`）:
            // highlight 中の item は `aria-activedescendant` から参照可能な
            // `id` を持つ必要があるため、highlight する項目にのみ id を付与
            // する（下の `input` 呼び出しの `activedescendant` 引数と対）。
            let id = highlighted.then_some("showcase-combobox-item-svelte");
            combobox::item(
                selected_state,
                disabled,
                highlighted,
                value,
                id,
                vec![],
                vec![
                    combobox::item_text(None, vec![], vec![text(label)]),
                    combobox::item_indicator(selected_state, vec![], vec![text("✓")]),
                ],
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
                        // "svelte" item が highlight 中のため、その id を
                        // `aria-activedescendant` として参照する（R3 契約、
                        // 上の item 生成コメント参照）。
                        Some("showcase-combobox-item-svelte"),
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
            "headless-ui の Combobox（role=\"combobox\"）に pre-styled-ui の recipe CSS を適用した静的掲示です。入力値 \"{query}\" による filter_options の絞り込み結果を候補として表示しています。\"React\" は選択済み（チェックマーク表示）、\"Svelte\" は highlight 中、\"Vue\" は disabled として固定しています。positioner はフロー内配置へ中和しています。"
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
        Size::Md,
        vec![],
        vec![
            tree_view::label(vec![], vec![text("Project files")]),
            tree_view::tree(Some("Project files"), None, vec![], root_children),
        ],
    );

    section(
        "TreeView",
        "headless-ui の TreeView（role=\"tree\"/role=\"treeitem\"）に pre-styled-ui の recipe CSS を適用した静的掲示です。\"src\" ブランチを展開済み、\"src/lib.rs\" を選択中、\"README.md\" を disabled として固定表示しています。インデントは CSS custom property（--fandhe-tree-view-indent）で表現しています。size（既定 md）は行密度・文字サイズを切り替えます。",
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
        Size::Md,
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
    // イシュー #1509: size 5 段（xs〜xl）で track/thumb 寸法・root 余白
    // （gap）・label font-size が単調に連動することを視覚確認できる行
    // （`crate::checkbox` #1455 の size_row と同型）。
    let size_row = row([Size::Xs, Size::Sm, Size::Md, Size::Lg, Size::Xl]
        .iter()
        .map(|size| {
            let name = format!("showcase-switch-size-{}", size.value());
            switch::root(
                *size,
                ColorPalette::Accent,
                true,
                false,
                vec![],
                vec![
                    switch::hidden_input(&name, "on", true, false, false, vec![]),
                    switch::control(
                        true,
                        false,
                        vec![],
                        vec![switch::thumb(true, vec![], vec![])],
                    ),
                    switch::label(true, vec![], vec![text(size.value())]),
                ],
            )
        })
        .collect());
    section(
        "Switch",
        "data-state=\"checked\"/\"unchecked\" で見た目が切り替わるオン/オフ スイッチ。visually-hidden な input[type=\"checkbox\"][role=\"switch\"] がフォーム送信・キーボード操作の意味論を担います。",
        vec![demo_row, size_row],
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

    // イシュー #1495: `data-orientation="horizontal"` の折り返し横並び
    // レイアウト（`flex-wrap: wrap` + `column-gap: var(--fandhe-space-4)`）と
    // `label` の独立行占有を可視化する 2 件目のデモ（`checkbox_group`
    // #1460 の `horizontal_demo` と同型）。項目・状態は縦積みデモと同一。
    let horizontal_label_id = "showcase-radio-horizontal-label";
    let mut horizontal_children = vec![radio_group::label(
        Some(horizontal_label_id),
        vec![],
        vec![text("Plan (horizontal)")],
    )];
    horizontal_children.extend(items.iter().map(|(value, label, checked, disabled)| {
        radio_group::item(
            *checked,
            *disabled,
            value,
            vec![],
            vec![
                radio_group::item_hidden_input(
                    *checked,
                    *disabled,
                    Some("showcase-radio-horizontal"),
                    value,
                    vec![],
                ),
                radio_group::item_control(*checked, *disabled, vec![]),
                radio_group::item_text(*checked, *disabled, vec![], vec![text(*label)]),
            ],
        )
    }));
    let horizontal_demo = radio_group::root(
        Size::Md,
        ColorPalette::Accent,
        false,
        Some(Orientation::Horizontal),
        Some(horizontal_label_id),
        vec![],
        horizontal_children,
    );

    // イシュー #1495: size 5 段（xs〜xl）で control 寸法・root/item 余白・
    // font-size が単調に連動し、label（見出し）が item-text（項目）より
    // 太いことを視覚確認できる行（`checkbox_group` #1461 の `size_row` と
    // 同型）。
    let size_row = row([Size::Xs, Size::Sm, Size::Md, Size::Lg, Size::Xl]
        .iter()
        .map(|size| {
            let size_label_id = format!("showcase-radio-size-{}-label", size.value());
            let mut size_children = vec![radio_group::label(
                Some(&size_label_id),
                vec![],
                vec![text(size.value())],
            )];
            size_children.extend(items.iter().map(|(value, label, checked, disabled)| {
                let name = format!("showcase-radio-size-{}", size.value());
                radio_group::item(
                    *checked,
                    *disabled,
                    value,
                    vec![],
                    vec![
                        radio_group::item_hidden_input(
                            *checked,
                            *disabled,
                            Some(&name),
                            value,
                            vec![],
                        ),
                        radio_group::item_control(*checked, *disabled, vec![]),
                        radio_group::item_text(*checked, *disabled, vec![], vec![text(*label)]),
                    ],
                )
            }));
            radio_group::root(
                *size,
                ColorPalette::Accent,
                false,
                Some(Orientation::Vertical),
                Some(&size_label_id),
                vec![],
                size_children,
            )
        })
        .collect());

    section(
        "RadioGroup",
        "単一選択の選択肢グループ。ネイティブ input[type=\"radio\"] による排他選択・キーボード操作を data-scope=\"radio-group\" の anatomy へ重ねます。",
        vec![demo, horizontal_demo, size_row],
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

/// Avatar 節: size（Xs〜Xl）・shape（Circle/Rounded/Square）・variant
/// （Subtle/Solid/Outline）・colorPalette（6 値）の 4 軸（イシュー #1554 で
/// variant/palette 軸を追加）。
///
/// `image` パーツの `src` は外部フェッチ・404 を発生させないダミー値
/// （[`AVATAR_EMPTY_IMAGE_SRC`]/[`AVATAR_INLINE_SVG_SRC`]）を使う
/// （`examples/headless-pre-styled-ui` の avatar 節と同じく実画像を同梱
/// しない方針）。`image` パーツ自体は `ImageStatus` に応じて headless 層が
/// `hidden` 存在属性を出力するため、Error 状態でも anatomy には含まれる。
fn avatar_section() -> Node {
    let size_row = row(vec![
        (Size::Xs, "FT"),
        (Size::Sm, "FT"),
        (Size::Md, "FT"),
        (Size::Lg, "FT"),
        (Size::Xl, "FT"),
    ]
    .into_iter()
    .map(|(size, initials)| {
        let props = AvatarProps {
            size,
            ..AvatarProps::default()
        };
        avatar::root(
            &props,
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
        let props = AvatarProps {
            shape,
            ..AvatarProps::default()
        };
        avatar::root(
            &props,
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

    let variant_row = row(vec![
        AvatarVariant::Subtle,
        AvatarVariant::Solid,
        AvatarVariant::Outline,
    ]
    .into_iter()
    .map(|variant| {
        let props = AvatarProps {
            variant,
            ..AvatarProps::default()
        };
        avatar::root(
            &props,
            vec![],
            vec![
                avatar::image(
                    ImageStatus::Error,
                    AVATAR_EMPTY_IMAGE_SRC,
                    "Fandhe Team",
                    vec![],
                ),
                avatar::fallback(ImageStatus::Error, vec![], vec![text("FT")]),
            ],
        )
    })
    .collect());

    let palette_row = row(vec![
        ColorPalette::Accent,
        ColorPalette::Info,
        ColorPalette::Success,
        ColorPalette::Warning,
        ColorPalette::Danger,
        ColorPalette::Neutral,
    ]
    .into_iter()
    .map(|palette| {
        let props = AvatarProps {
            variant: AvatarVariant::Solid,
            palette,
            ..AvatarProps::default()
        };
        avatar::root(
            &props,
            vec![],
            vec![
                avatar::image(
                    ImageStatus::Error,
                    AVATAR_EMPTY_IMAGE_SRC,
                    "Fandhe Team",
                    vec![],
                ),
                avatar::fallback(ImageStatus::Error, vec![], vec![text("FT")]),
            ],
        )
    })
    .collect());

    section(
        "Avatar",
        "size（Xs〜Xl）・shape（Circle/Rounded/Square）・variant（Subtle/Solid/Outline）・colorPalette（6 値）の 4 軸を持つユーザー画像表示。画像読み込み状態（ImageStatus）を固定し、Error 時はイニシャルのフォールバック表示、Loaded 時は画像表示を掲示します。",
        vec![size_row, shape_row, variant_row, palette_row],
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
    // イシュー #1455: size 5 段（xs〜xl）で control 寸法・root 余白（gap）・
    // label font-size が単調に連動することを視覚確認できる行。
    let size_row = row([Size::Xs, Size::Sm, Size::Md, Size::Lg, Size::Xl]
        .iter()
        .map(|size| {
            let props = CheckboxProps {
                checked: CheckedState::Checked,
                ..CheckboxProps::default()
            };
            let name = format!("showcase-checkbox-size-{}", size.value());
            checkbox::root(
                *size,
                ColorPalette::Accent,
                &props,
                vec![],
                vec![
                    checkbox::hidden_input(&props, &name, "on", vec![]),
                    checkbox::control(
                        &props,
                        vec![],
                        vec![checkbox::indicator(&props, vec![], vec![])],
                    ),
                    checkbox::label(&props, vec![], vec![text(size.value())]),
                ],
            )
        })
        .collect());
    section(
        "Checkbox",
        "data-state=\"checked\"/\"unchecked\"/\"indeterminate\" の 3 態を持つチェックボックス。visually-hidden な input[type=\"checkbox\"] がフォーム送信・キーボード操作の意味論を担い、チェックマークは CSS の border 合成で描画します（画像アセット不使用）。",
        vec![demo_row, size_row],
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
    // readonly（イシュー #1485: readonly 状態を Demo で確認できるよう
    // 追加。`input` パートはネイティブ `<input type="text">` のため
    // `data-readonly` へ視覚宣言〔`cursor: default` 等〕を追加しない
    // （`crate::input` の readonly 意図的非採用と同型、Cursor Bugbot
    // 指摘 PR #1764 の是正）。date-input #1469 の readonly 行追加とは
    // 見た目の扱いが異なる点に注意）。
    let readonly = number_input::root(
        Size::Md,
        false,
        false,
        vec![],
        vec![
            number_input::label(
                false,
                false,
                Some("showcase-number-input-readonly"),
                vec![],
                vec![text("Readonly")],
            ),
            number_input::control(
                false,
                false,
                vec![],
                vec![
                    number_input::input(
                        "quantity-readonly",
                        Some("showcase-number-input-readonly"),
                        Some("7"),
                        "0",
                        "10",
                        NumberInputFlags {
                            readonly: true,
                            ..NumberInputFlags::default()
                        },
                        vec![],
                    ),
                    number_input::increment_trigger(
                        Some("showcase-number-input-readonly"),
                        true,
                        vec![],
                        vec![text("+")],
                    ),
                    number_input::decrement_trigger(
                        Some("showcase-number-input-readonly"),
                        true,
                        vec![],
                        vec![text("-")],
                    ),
                ],
            ),
        ],
    );
    let demo_row = row(vec![mid, at_min, disabled, readonly]);
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

/// TagsInput 節: 通常タグ数件（highlighted 1 件込み）・編集中タグ・max
/// 到達（`data-invalid`/`aria-invalid`）・disabled の 4 態。
///
/// `control` は `role="listbox"`、各タグの `item-preview` は `role="option"`
/// （headless 層の listbox 相当 ARIA、`fandhe_frontend_pre_styled_ui::tags_input`
/// のモジュール doc 参照）。編集モード（`item-input`/`data-editing`）は
/// SSR 静的実演として `editing` 列に 1 件掲示する（イシュー #1699。
/// wasm 層の実対話〔キー入力に応じた編集開始/終了の状態遷移〕自体は
/// 引き続き wasm-full の配線層が担う。ここでは `item-input` slot の見た目
/// のみを静的マークアップで示す）。
fn tags_input_section() -> Node {
    fn tag_item(tag: &str, disabled: bool, highlighted: bool) -> Node {
        tags_input::item(
            disabled,
            false,
            vec![],
            vec![tags_input::item_preview(
                highlighted,
                vec![],
                vec![
                    tags_input::item_text(vec![], vec![text(tag)]),
                    tags_input::item_delete_trigger(tag, disabled, vec![], vec![text("\u{00d7}")]),
                ],
            )],
        )
    }

    /// 編集中のタグ 1 件（`data-editing` + `item-input`）を組み立てる
    /// （イシュー #1699 静的実演。`item_preview`/`item_text` は編集中は
    /// 描画せず `item_input` のみを子に持つ、headless
    /// `crates/headless-ui/src/tags_input.rs::item` rustdoc の想定
    /// マークアップに従う）。
    fn editing_tag_item(value: &str) -> Node {
        tags_input::item(
            false,
            true,
            vec![],
            vec![tags_input::item_input(value, vec![])],
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
                    tag_item("rust", false, false),
                    // highlighted: キーボード操作でフォーカス移動中の 1 件
                    // （`data-highlighted`、モジュール rustdoc「内部パートの
                    // スタイル調整」節が追加した transition の実演を兼ねる）。
                    tag_item("wasm", false, true),
                    tags_input::input("", false, false, vec![]),
                ],
            ),
            tags_input::hidden_input("skills", "rust,wasm", false, vec![]),
        ],
    );

    let editing = tags_input::root(
        Size::Md,
        false,
        vec![],
        vec![
            tags_input::label(vec![], vec![text("Editing")]),
            tags_input::control(
                false,
                false,
                "Editing",
                vec![],
                vec![
                    tag_item("go", false, false),
                    editing_tag_item("wa"),
                    tags_input::input("", false, false, vec![]),
                ],
            ),
            tags_input::hidden_input("editing-tags", "go", false, vec![]),
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
                    tag_item("a", false, false),
                    tag_item("b", false, false),
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
                    tag_item("readonly", true, false),
                    tags_input::input("", true, false, vec![]),
                ],
            ),
            tags_input::hidden_input("disabled-tags", "readonly", true, vec![]),
        ],
    );

    let demo_row = row(vec![normal, editing, at_max, disabled]);
    section(
        "TagsInput",
        "自由入力によるタグ配列。control は role=\"listbox\"、各タグは role=\"option\" を持ち、max 到達時は input が data-invalid/aria-invalid を伴います。編集中のタグは item-input（ネイティブ input）で表示されます。",
        vec![demo_row],
    )
}

/// FileUpload 節（イシュー #840）: 通常（受理済み 1 件）・disabled の 2 態。
/// `File` オブジェクトは headless 層で一切保持せず、ここでは静的な
/// `FileUploadItem` メタデータのみを直接組み立てて表示する（実 `File` API
/// 接触は `fandhe-frontend-wasm-full::headless_file_upload` の配線層のみが
/// 担う、`file_upload` モジュール rustdoc「保留解除」節参照）。
fn file_upload_section() -> Node {
    // `invalid` は headless `item` が出力しない属性のため（イシュー #1697
    // モジュール rustdoc「内部パートのスタイル調整」節参照）、呼び出し側が
    // `attrs` へ `("data-invalid", "")` を直接渡すことで CSS の
    // `[data-invalid]` 規則（border-color danger 化）を有効化できることを
    // ここで実演する。
    fn file_item(name: &str, size_bytes: u64, disabled: bool, invalid: bool) -> Node {
        let size_text = file_upload::item_size_text(size_bytes);
        let attrs = if invalid {
            vec![("data-invalid", "")]
        } else {
            vec![]
        };
        file_upload::item(
            disabled,
            attrs,
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
                vec![("aria-label", "Upload files")],
                vec![
                    file_upload::trigger(false, vec![], vec![text("Browse files")]),
                    file_upload::hidden_input("image/*,.pdf", true, false, vec![]),
                ],
            ),
            file_upload::item_group(
                vec![],
                vec![
                    file_item("report.pdf", 204_800, false, false),
                    // `data-invalid` の視覚差（border-color danger 化）を
                    // 通常態の一覧内で確認できるようにする実例。
                    file_item("oversized.zip", 52_428_800, false, true),
                ],
            ),
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
                vec![("aria-label", "Upload files")],
                vec![
                    file_upload::trigger(true, vec![], vec![text("Browse files")]),
                    file_upload::hidden_input("image/*,.pdf", true, true, vec![]),
                ],
            ),
            file_upload::item_group(vec![], vec![file_item("locked.txt", 1024, true, false)]),
            file_upload::clear_trigger(true, vec![], vec![text("Clear all")]),
        ],
    );

    // イシュー #1696: `data-dragging` state（ドラッグ中の dropzone 強調表示）
    // を確認できる態が Demo に無かったため追加する。
    let dragging = file_upload::root(
        Size::Md,
        false,
        vec![],
        vec![
            file_upload::label(vec![], vec![text("Dragging")]),
            file_upload::dropzone(
                false,
                true,
                vec![("aria-label", "Upload files")],
                vec![
                    file_upload::trigger(false, vec![], vec![text("Browse files")]),
                    file_upload::hidden_input("image/*,.pdf", true, false, vec![]),
                ],
            ),
        ],
    );

    let demo_row = row(vec![normal, disabled, dragging]);
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
    let build = |id_prefix: &'static str,
                 size: Size,
                 value: Option<u32>,
                 disabled: bool,
                 readonly: bool| {
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
            size,
            ColorPalette::Accent,
            disabled,
            readonly,
            vec![],
            children,
        )
    };

    let selected = build("showcase-rating-selected", Size::Md, Some(3), false, false);
    let readonly = build("showcase-rating-readonly", Size::Md, Some(4), false, true);
    let disabled = build("showcase-rating-disabled", Size::Md, Some(2), true, false);

    // イシュー #1496: size variant 5 段（Xs〜Xl）を並べ、`label`
    // font-size の size 連動を含めて視覚確認できる行を追加する
    // （`download_trigger_section` #1750 の size_row と同型）。
    let sizes = [
        (Size::Xs, "showcase-rating-size-xs"),
        (Size::Sm, "showcase-rating-size-sm"),
        (Size::Md, "showcase-rating-size-md"),
        (Size::Lg, "showcase-rating-size-lg"),
        (Size::Xl, "showcase-rating-size-xl"),
    ];
    let size_row = row(sizes
        .iter()
        .map(|(size, id_prefix)| build(id_prefix, *size, Some(3), false, false))
        .collect());

    section(
        "RatingGroup",
        "1..=count の星評価。data-highlighted が塗り表示（hover プレビュー優先）、data-checked が確定選択を表します。星形は SVG/画像 URL を使わない clip-path によるインライン表現です。",
        vec![row(vec![selected, readonly, disabled]), size_row],
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

/// SegmentGroup 節（イシュー #743）: 既定（選択済み）・disabled・Size 5 種・
/// vertical orientation の静的掲示。状態機械
/// （[`fandhe_frontend_pre_styled_ui::segment_group::SegmentGroup`]、
/// `radio_group::RadioGroup` への全委譲）は使わず、他の docs-site 節と同じく
/// SSR 静的マークアップのみを組み立てる（本モジュール冒頭「インタラクティブ
/// 部品の扱い」節参照）。indicator の位置は選択項目の `(index, count)` から
/// 手計算で `segment_group::indicator` へ渡す（headless 層の SSR 決定的な
/// 位置表現契約、`crates/headless-ui/src/segment_group.rs` module doc 参照）。
///
/// `orientation` はイシュー #1499 で追加した引数（既存呼び出しは `None` を
/// 渡し現行の horizontal 出力を維持する）。`indicator`/`root` 双方へ同じ
/// 値を渡す必要がある（`root` の `data-orientation` と `indicator` の CSS
/// 変数幾何〔translateY 対称形〕が対で成立する契約、`segment_group.rs`
/// 冒頭 rustdoc「Indicator の位置表現とスタイル連動」節参照）。
fn segment_group_demo(
    id_prefix: &str,
    size: Size,
    disabled: bool,
    selected_index: usize,
    orientation: Option<Orientation>,
) -> Node {
    let items = ["List", "Grid", "Table"];
    let mut children = vec![segment_group::indicator(
        Some((selected_index, items.len())),
        orientation,
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
    segment_group::root(size, disabled, orientation, None, vec![], children)
}

fn segment_group_section() -> Node {
    // イシュー #1499: size 5 段（xs〜xl）で padding・font-size が単調に
    // 連動することを視覚確認できる行（`radio_group` #1495 の `size_row` と
    // 同型）。
    let size_row = row([Size::Xs, Size::Sm, Size::Md, Size::Lg, Size::Xl]
        .iter()
        .enumerate()
        .map(|(index, size)| {
            segment_group_demo(
                &format!("showcase-segment-size-{}", size.value()),
                *size,
                false,
                index % 3,
                None,
            )
        })
        .collect());
    let disabled_demo = segment_group_demo("showcase-segment-disabled", Size::Md, true, 0, None);
    // イシュー #1499: vertical orientation（indicator の translateY 幾何と
    // column レイアウト）を可視化するデモ。
    let vertical_demo = segment_group_demo(
        "showcase-segment-vertical",
        Size::Md,
        false,
        1,
        Some(Orientation::Vertical),
    );
    section(
        "SegmentGroup",
        "単一選択のセグメント UI（segmented control）。ネイティブ input[type=\"radio\"] による排他選択を data-scope=\"segment-group\" の anatomy へ重ね、選択中の項目を indicator の CSS 変数（--fandhe-segment-group-index/-count）で示します。状態機械は RadioGroup（SingleSelect）への全委譲です。",
        vec![size_row, disabled_demo, vertical_demo],
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
                // イシュー #1545: 1 件目（Info）のみ action-trigger を含め、
                // outline 小ボタンの見た目（hover/focus/disabled/transition）
                // が Demo で確認できるようにする（Anatomy 表・`data-*` 属性表
                // の機械導出元）。
                let mut children = vec![
                    toast::title(vec![], vec![text(*title)]),
                    toast::description(vec![], vec![text(*description)]),
                ];
                if *status == ToastStatus::Info {
                    children.push(toast::action_trigger(vec![], vec![text("Update")]));
                }
                children.push(toast::close_trigger(
                    vec![("aria-label", "Dismiss")],
                    vec![text("×")],
                ));
                toast::root(*status, vec![], children)
            })
            .collect(),
    );
    section(
        "Toast",
        "headless-ui の Toast（`role=\"status\"` + `aria-live`（`error` のみ `assertive`）+ `aria-atomic=\"true\"`）に pre-styled-ui の placement（`group` slot）/status（`root` slot）/action-trigger・close-trigger（hover/focus/disabled）variant CSS を適用した静的掲示です。複数通知の有界キュー管理・自動 dismiss のタイマー配線は wasm 層の後続イシューのスコープ外です。",
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
/// `Steps` の Demo 1 件を組み立てる（`orientation` により horizontal/
/// vertical を切り替える内部ヘルパ、[`steps_section`] のみが呼ぶ）。
///
/// イシュー #1540: 従来は horizontal 固定の単一 Demo だったが、`root` の
/// 縦向きレイアウト是正（`data-orientation="vertical"` で
/// `flex-direction: row` へ切り替える recipe 追加）を部品ページで視覚確認
/// できるよう、vertical Demo も並べて表示する。
fn steps_demo(orientation: Orientation) -> Node {
    let s = Steps::new(3, 1, orientation);
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
    // PR #1814 codex-review 対応（P1）: `list` 以外（content/nav）を
    // `steps::body` でまとめる（[`steps::root`] rustdoc 「縦向きでの
    // children 構成契約」節参照）。縦向きで root が `flex-direction: row`
    // へ切り替わる際、root 直下を `list`/`body` の 2 要素に保つことで
    // list を左・content+nav を右カラムに縦積みして表示する。
    let body = steps::body(vec![], vec![content, nav]);

    steps::root(Size::Md, ColorPalette::Accent, &s, vec![], vec![list, body])
}

fn steps_section() -> Node {
    let horizontal = steps_demo(Orientation::Horizontal);
    let vertical = steps_demo(Orientation::Vertical);
    section(
        "Steps",
        "count（全 step 数）+ step（現在位置）を持つ headless Steps の静的掲示。item は complete/current/incomplete の 3 状態を持ち、current な item の trigger のみ aria-current=\"step\" を持ちます（クリック挙動は wasm 層のスコープ外）。orientation（横向き/縦向き）は root の `data-orientation` 属性により list/content の並び方向が切り替わります。",
        vec![row(vec![horizontal, vertical])],
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
                            // イシュー #1551: action-trigger の並置間隔
                            // （`margin-inline-end`）を実際に確認できる
                            // よう、2 個並べる（Prev/Next）。
                            tour::action_trigger(&t, vec![], vec![text("Prev")]),
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
        // イシュー #1537: `root` は既定で高さを強制しない（利用側の責務、
        // `splitter` モジュール rustdoc「意図的に採らなかった変更」節
        // 参照）ため、Demo 用途としてここで明示の高さを与える
        // （垂直デモの `height: 16rem;` と同じ位置づけ。参照サイト
        // 〔chakra-ui〕が docs のデモで `minH` props を与えるのと同型）。
        vec![("style", "min-height: 12rem;")],
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

    let build = |id_prefix: &str, state: &DateInput, size: Size, disabled: bool, readonly: bool| {
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
                                state.segment(DateSegment::Year, disabled, readonly, vec![]),
                                state.segment(DateSegment::Month, disabled, readonly, vec![]),
                                state.segment(DateSegment::Day, disabled, readonly, vec![]),
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
    let filled = build(
        "showcase-date-input-filled",
        &filled_state,
        Size::Md,
        false,
        false,
    );

    // 未入力（3 セグメントとも placeholder 表示）。
    let empty_state_value = DateInput::default();
    let empty = build(
        "showcase-date-input-empty",
        &empty_state_value,
        Size::Md,
        false,
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
        false,
    );

    // disabled。
    let disabled_state = DateInput::new(Some(2026), Some(1), Some(1), None, None);
    let disabled = build(
        "showcase-date-input-disabled",
        &disabled_state,
        Size::Md,
        true,
        false,
    );

    // readonly（イシュー #1469: `segment` の `data-readonly` 視覚
    // 〔`cursor: default`〕を Demo で確認できるよう追加）。
    let readonly_state = DateInput::new(Some(2026), Some(7), Some(22), None, None);
    let readonly = build(
        "showcase-date-input-readonly",
        &readonly_state,
        Size::Md,
        false,
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
            false,
        ));
    }

    section(
        "DateInput",
        "年/月/日セグメント入力 DateInput の静的掲示（入力済み・未入力・invalid・disabled・readonly・size 各種）。各セグメントは role=\"spinbutton\" + aria-valuemin/max/now（未入力時は valuenow 省略）を持ちます（キーボード操作は wasm 層のスコープ外）。",
        vec![
            row(vec![filled]),
            row(vec![empty]),
            row(vec![invalid]),
            row(vec![readonly]),
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
            false,
            "showcase-checkbox-card-unchecked",
            "Starter",
            "個人利用向けの基本プラン。",
        ),
        (
            CheckedState::Checked,
            false,
            false,
            "showcase-checkbox-card-checked",
            "Pro",
            "チームでの共同作業に対応。",
        ),
        (
            CheckedState::Checked,
            true,
            false,
            "showcase-checkbox-card-disabled",
            "Enterprise",
            "現在準備中のプランです。",
        ),
        // イシュー #1457: root の `data-invalid` 状態（枠線を
        // `--fandhe-color-danger` へ切り替える）を Demo で視覚確認できるよう
        // 追加する（root スタイル調整の担当範囲）。
        (
            CheckedState::Unchecked,
            false,
            true,
            "showcase-checkbox-card-invalid",
            "Team",
            "選択が必須の項目です。",
        ),
    ];
    let demo_row = row(states
        .iter()
        .map(|(checked, disabled, invalid, name, label, description)| {
            let props = CheckboxProps {
                checked: *checked,
                disabled: *disabled,
                invalid: *invalid,
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
                            // イシュー #1458: chakra-ui/Radix Themes は indicator を
                            // 右端に配置する（左右位置の意図的差分、
                            // `crates/pre-styled-ui/src/checkbox_card.rs` rustdoc
                            // 参照）。CSS `order` は使わず DOM 順（content →
                            // indicator）で決めるため、Demo でも同じ子順にして
                            // 視覚確認できるようにする。
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
                            checkbox_card::indicator(
                                &props,
                                vec![],
                                vec![checkbox_card::indicator_check(&props, vec![], vec![])],
                            ),
                        ],
                    ),
                ],
            )
        })
        .collect());
    // イシュー #1458: size 5 段（xs〜xl）で padding・control 寸法・
    // description font-size・root 余白（gap）が単調に連動することを
    // 視覚確認できる行（`crate::checkbox` #1455 の size_row と同型）。
    let size_row = row([Size::Xs, Size::Sm, Size::Md, Size::Lg, Size::Xl]
        .iter()
        .map(|size| {
            let props = CheckboxProps {
                checked: CheckedState::Checked,
                ..CheckboxProps::default()
            };
            let name = format!("showcase-checkbox-card-size-{}", size.value());
            checkbox_card::root(
                *size,
                ColorPalette::Accent,
                &props,
                vec![],
                vec![
                    checkbox_card::hidden_input(&props, &name, "on", vec![]),
                    checkbox_card::control(
                        &props,
                        vec![],
                        vec![
                            checkbox_card::content(
                                &props,
                                vec![],
                                vec![
                                    checkbox_card::label(&props, vec![], vec![text(size.value())]),
                                    checkbox_card::description(
                                        &props,
                                        vec![],
                                        vec![text("size demo")],
                                    ),
                                ],
                            ),
                            checkbox_card::indicator(
                                &props,
                                vec![],
                                vec![checkbox_card::indicator_check(&props, vec![], vec![])],
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
        vec![demo_row, size_row],
    )
}

/// CheckboxGroup 節: 複数選択の選択肢グループ（イシュー #997）。
///
/// Radix Themes Checkbox Group 相当。状態機械は
/// [`fandhe_frontend_pre_styled_ui::radio_group`] 節と対称の
/// [`fandhe_frontend_headless_ui::state::MultiSelect`] を埋め込んだ
/// `CheckboxGroup`（複数同時選択）。ネイティブ `<input type="checkbox">` は
/// 自前パーツを持たず [`fandhe_frontend_pre_styled_ui::checkbox::hidden_input`]
/// を [`checkbox_group::item`] 配下へ入れ子で再利用する（`crates/pre-styled-ui/src/checkbox_group.rs`
/// rustdoc「`item-hidden-input` を本モジュールが持たない理由」節参照。
/// [`stylesheet`] が `checkbox::stylesheet()` と `checkbox_group::stylesheet()`
/// の両方を push する契約もこの入れ子構成に対応する）。
fn checkbox_group_section() -> Node {
    let label_id = "showcase-checkbox-group-label";
    let items = [
        ("red", "Red", true, false),
        ("green", "Green", false, false),
        ("blue", "Blue", false, true),
    ];
    let mut children = vec![checkbox_group::label(
        Some(label_id),
        vec![],
        vec![text("Colors")],
    )];
    children.extend(items.iter().map(|(value, label, checked, disabled)| {
        let props = CheckboxProps {
            checked: if *checked {
                CheckedState::Checked
            } else {
                CheckedState::Unchecked
            },
            disabled: *disabled,
            ..CheckboxProps::default()
        };
        checkbox_group::item(
            *checked,
            *disabled,
            value,
            vec![],
            vec![
                checkbox::hidden_input(&props, "showcase-checkbox-group", value, vec![]),
                checkbox_group::item_control(
                    *checked,
                    *disabled,
                    vec![],
                    vec![checkbox_group::item_indicator(
                        *checked,
                        *disabled,
                        vec![],
                        vec![],
                    )],
                ),
                checkbox_group::item_text(*checked, *disabled, vec![], vec![text(*label)]),
            ],
        )
    }));
    let demo = checkbox_group::root(
        Size::Md,
        ColorPalette::Accent,
        false,
        Some(Orientation::Vertical),
        Some(label_id),
        vec![],
        children,
    );

    // イシュー #1460: `data-orientation="horizontal"` の折り返し横並び
    // レイアウト（`flex-wrap: wrap` + `column-gap: var(--fandhe-space-4)`）
    // を可視化する 2 件目のデモ。項目・状態は縦積みデモと同一。
    let horizontal_label_id = "showcase-checkbox-group-horizontal-label";
    let mut horizontal_children = vec![checkbox_group::label(
        Some(horizontal_label_id),
        vec![],
        vec![text("Colors (horizontal)")],
    )];
    horizontal_children.extend(items.iter().map(|(value, label, checked, disabled)| {
        let props = CheckboxProps {
            checked: if *checked {
                CheckedState::Checked
            } else {
                CheckedState::Unchecked
            },
            disabled: *disabled,
            ..CheckboxProps::default()
        };
        checkbox_group::item(
            *checked,
            *disabled,
            value,
            vec![],
            vec![
                checkbox::hidden_input(&props, "showcase-checkbox-group-horizontal", value, vec![]),
                checkbox_group::item_control(
                    *checked,
                    *disabled,
                    vec![],
                    vec![checkbox_group::item_indicator(
                        *checked,
                        *disabled,
                        vec![],
                        vec![],
                    )],
                ),
                checkbox_group::item_text(*checked, *disabled, vec![], vec![text(*label)]),
            ],
        )
    }));
    let horizontal_demo = checkbox_group::root(
        Size::Md,
        ColorPalette::Accent,
        false,
        Some(Orientation::Horizontal),
        Some(horizontal_label_id),
        vec![],
        horizontal_children,
    );

    // イシュー #1460: `data-invalid` は headless 層が出力しないため（`root`
    // の `attrs` へ利用者が直接付与する経路のみ）、その付与例をデモとして
    // 示す。CSS 側は `root[data-invalid]` から `item-control` の
    // border-color へ custom property 経由で伝播するのみで、headless 層に
    // `invalid` フラグを追加するものではない（#1603 の射程、`checkbox_group.rs`
    // rustdoc「本イシューのスコープ外」節参照）。
    let invalid_label_id = "showcase-checkbox-group-invalid-label";
    let mut invalid_children = vec![checkbox_group::label(
        Some(invalid_label_id),
        vec![],
        vec![text("Colors (invalid)")],
    )];
    invalid_children.extend(items.iter().map(|(value, label, checked, disabled)| {
        let props = CheckboxProps {
            checked: if *checked {
                CheckedState::Checked
            } else {
                CheckedState::Unchecked
            },
            disabled: *disabled,
            ..CheckboxProps::default()
        };
        checkbox_group::item(
            *checked,
            *disabled,
            value,
            vec![],
            vec![
                checkbox::hidden_input(&props, "showcase-checkbox-group-invalid", value, vec![]),
                checkbox_group::item_control(
                    *checked,
                    *disabled,
                    vec![],
                    vec![checkbox_group::item_indicator(
                        *checked,
                        *disabled,
                        vec![],
                        vec![],
                    )],
                ),
                checkbox_group::item_text(*checked, *disabled, vec![], vec![text(*label)]),
            ],
        )
    }));
    let invalid_demo = checkbox_group::root(
        Size::Md,
        ColorPalette::Accent,
        false,
        Some(Orientation::Vertical),
        Some(invalid_label_id),
        vec![("data-invalid", "")],
        invalid_children,
    );

    // イシュー #1461: size 5 段（xs〜xl）で control 寸法・root/item 余白・
    // font-size が単調に連動し、label（見出し）が item-text（項目）より
    // 太いことを視覚確認できる行（`checkbox_section`/`checkbox_card_section`
    // の `size_row` と同型）。
    let size_row = row([Size::Xs, Size::Sm, Size::Md, Size::Lg, Size::Xl]
        .iter()
        .map(|size| {
            let size_label_id = format!("showcase-checkbox-group-size-{}-label", size.value());
            let mut size_children = vec![checkbox_group::label(
                Some(&size_label_id),
                vec![],
                vec![text(size.value())],
            )];
            size_children.extend(items.iter().map(|(value, label, checked, disabled)| {
                let props = CheckboxProps {
                    checked: if *checked {
                        CheckedState::Checked
                    } else {
                        CheckedState::Unchecked
                    },
                    disabled: *disabled,
                    ..CheckboxProps::default()
                };
                let name = format!("showcase-checkbox-group-size-{}", size.value());
                checkbox_group::item(
                    *checked,
                    *disabled,
                    value,
                    vec![],
                    vec![
                        checkbox::hidden_input(&props, &name, value, vec![]),
                        checkbox_group::item_control(
                            *checked,
                            *disabled,
                            vec![],
                            vec![checkbox_group::item_indicator(
                                *checked,
                                *disabled,
                                vec![],
                                vec![],
                            )],
                        ),
                        checkbox_group::item_text(*checked, *disabled, vec![], vec![text(*label)]),
                    ],
                )
            }));
            checkbox_group::root(
                *size,
                ColorPalette::Accent,
                false,
                Some(Orientation::Vertical),
                Some(&size_label_id),
                vec![],
                size_children,
            )
        })
        .collect());
    section(
        "CheckboxGroup",
        "複数選択の選択肢グループ。ネイティブ input[type=\"checkbox\"]（fandhe_frontend_pre_styled_ui::checkbox::hidden_input の再利用）による同時選択・キーボード操作を data-scope=\"checkbox-group\" の anatomy へ重ねます。",
        vec![demo, horizontal_demo, invalid_demo, size_row],
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
    // イシュー #1491: 4 枚目に invalid 状態のカードを追加し、`item` の
    // `data-invalid` 状態表現（境界線の danger 色化）を Demo 上で確認できる
    // ようにする。headless `radio_group` は `data-invalid` を出力しないため
    // （モジュール rustdoc「スタイル調整」節参照）、呼び出し側 `attrs`
    // パススルーで付与する（`ITEM_RESERVED` は非予約）。
    let items = [
        (
            "plan-free-card",
            "Free",
            "基本機能のみ利用可能。",
            true,
            false,
            false,
        ),
        (
            "plan-pro-card",
            "Pro",
            "チーム機能・優先サポート付き。",
            false,
            false,
            false,
        ),
        (
            "plan-enterprise-card",
            "Enterprise",
            "SSO・監査ログに対応。",
            false,
            true,
            false,
        ),
        (
            "plan-invalid-card",
            "Invalid",
            "選択に問題があるカードの例。",
            false,
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
            .map(|(value, label, description, checked, disabled, invalid)| {
                let attrs = if *invalid {
                    vec![("data-invalid", "")]
                } else {
                    vec![]
                };
                // イシュー #1491 codex-review P1 是正: `data-invalid`
                // による枠線の視覚表現だけでは支援技術に invalid 状態が
                // 伝わらないため、フォーカスを受ける実体である hidden
                // input へも `aria-invalid="true"` を付与する
                // （`item` 側の `data-invalid` は装飾用の CSS フックに
                // とどめ、状態通知は WAI-ARIA 属性側の責務とする）。
                let hidden_input_attrs = if *invalid {
                    vec![("aria-invalid", "true")]
                } else {
                    vec![]
                };
                radio_card::item(
                    *checked,
                    *disabled,
                    value,
                    attrs,
                    vec![
                        radio_card::item_hidden_input(
                            *checked,
                            *disabled,
                            Some("showcase-radio-card"),
                            value,
                            hidden_input_attrs,
                        ),
                        // イシュー #1492: chakra-ui/Radix Themes は indicator を
                        // 右端に配置する（左右位置の意図的差分、
                        // `crates/pre-styled-ui/src/radio_card.rs` rustdoc
                        // 参照）。CSS `order` は使わず DOM 順（content →
                        // indicator）で決めるため、Demo でも同じ子順にして
                        // 視覚確認できるようにする（checkbox-card #1458 の
                        // Demo と同型）。
                        radio_card::item_control(
                            *checked,
                            *disabled,
                            vec![],
                            vec![
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
                                radio_card::item_indicator(*checked, *disabled, *invalid, vec![]),
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
    // イシュー #1492: size 5 段（xs〜xl）で padding・control 寸法・
    // description font-size・item-control 余白（gap）が単調に連動することを
    // 視覚確認できる行（checkbox-card #1458 の `size_row` と同型）。
    let size_row = row([Size::Xs, Size::Sm, Size::Md, Size::Lg, Size::Xl]
        .iter()
        .map(|size| {
            let value = format!("showcase-radio-card-size-{}", size.value());
            let name = format!("showcase-radio-card-size-group-{}", size.value());
            // `--fandhe-radio-card-padding` 等のサイズ依存 CSS 変数は
            // `radio_card::root` にのみ定義されるため（`checkbox_card`
            // #1458 の size_row と同型）、各カードを個別の root で包んで
            // Size を反映させる（root なしの item 単体では既定値へ
            // フォールバックし、xs〜xl のスケールデモが機能しない）。
            radio_card::root(
                *size,
                ColorPalette::Accent,
                false,
                None,
                None,
                vec![],
                vec![radio_card::item(
                    true,
                    false,
                    &value,
                    vec![],
                    vec![
                        radio_card::item_hidden_input(true, false, Some(&name), &value, vec![]),
                        radio_card::item_control(
                            true,
                            false,
                            vec![],
                            vec![
                                radio_card::item_content(
                                    vec![],
                                    vec![
                                        radio_card::item_text(vec![], vec![text(size.value())]),
                                        radio_card::item_description(
                                            vec![],
                                            vec![text("size demo")],
                                        ),
                                    ],
                                ),
                                radio_card::item_indicator(true, false, false, vec![]),
                            ],
                        ),
                    ],
                )],
            )
        })
        .collect());
    section(
        "RadioCard",
        "chakra-ui radio-card 相当のカード型選択 UI。状態機械は RadioGroup（headless）をそのまま再利用し、data-scope=\"radio-card\" の新規 anatomy でカード外観を重ねます。",
        vec![demo, size_row],
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
                                        // イシュー #1703: `disabled_declarations()`
                                        // 経由の視覚（opacity/cursor）を掲示する。
                                        menubar::item(
                                            "print",
                                            true,
                                            false,
                                            vec![],
                                            vec![text("Print…")],
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
                                    // イシュー #1703: anatomy に `indicator`
                                    // パートが無いため、`justify-content:
                                    // space-between`（既存）が確保する右端の
                                    // 余白へマークアップ側の示唆グリフを
                                    // 置く読み替えで表現する（モジュール doc
                                    // 「イシュー #1703」節「意図的に合わせ
                                    // なかった点」参照）。隣接する 2 つの
                                    // text() ノードのままだと 1 つの連続
                                    // テキストランとして単一の匿名 flex
                                    // item になり `justify-content:
                                    // space-between` が効かないため
                                    // （PR #1804 Bugbot 指摘）、ラベルと
                                    // グリフをそれぞれ `span` で包んで
                                    // 独立した flex item にする。グリフ側の
                                    // `span` には `aria-hidden="true"` を
                                    // 付け、装飾用の示唆グリフが "Export"
                                    // と一緒にアクセシブル名として読み上げ
                                    // られないようにする（PR #1804
                                    // codex-review 指摘、AGENTS.md「UI 部品
                                    // の責務境界（アクセシビリティ）」）。
                                    vec![
                                        el("span", vec![], vec![text("Export")]),
                                        el("span", vec![("aria-hidden", "true")], vec![text("▸")]),
                                    ],
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
                vec![bar.trigger(1, false, true, None, vec![], vec![text("Edit")])],
            ),
        ],
    );
    section(
        "Menubar",
        "headless-ui の Menubar（role=\"menubar\"）に pre-styled-ui の recipe CSS を適用した静的掲示です。File / Edit の 2 Menu を水平配置し、File Menu を開いた状態（open=Some(0)）で表示しています。Item Group（Recent）・Separator・SubTrigger/SubContent（Export → PDF）の入れ子構造も含みます。roving tabindex（focused=0）により先頭の File トリガーのみ tabindex=\"0\" です。Edit トリガーは highlighted=true とし、trigger の data-highlighted 配色（イシュー #1702）も掲示します。File Menu の Print… item は disabled=true とし、内部パート是正（イシュー #1703）の disabled_declarations 配色・item/sub-trigger の hover・トランジション・トークン整合（radius/shadow/border-muted）を掲示します。Export sub-trigger は右端に示唆グリフ（▸）のテキストノードを添え、anatomy に indicator パートが無い制約下でのサブメニュー示唆を表現します。",
        vec![node],
    )
}

/// Tab Nav 節（イシュー #996）: root/link の 2 anatomy パーツを 1 デモに
/// 全網羅する（Anatomy 節はデモ HTML から機械導出されるため、`tab_nav.rs`
/// が持つ全パーツを描画する）。3 リンクのうち 1 件目を現在ページ
/// （`aria-current="page"`）として掲示する。`href` は
/// `showcase_markup_has_no_href_attributes_for_linkcheck_neutrality` の
/// linkcheck 中立性契約に従い空文字列固定とする（`navigation_menu_section`
/// と同じ制約）。`role="tablist"`/`role="tab"` を一切出力しないことが
/// 本部品の受け入れ条件であり、[`crate::component_page`] の Anatomy 節に
/// `role` が現れないことでも間接的に確認できる。
fn tab_nav_section() -> Node {
    let node = tab_nav::root(
        Size::Md,
        "Section navigation",
        vec![],
        vec![
            tab_nav::link("", true, vec![], vec![text("Overview")]),
            tab_nav::link("", false, vec![], vec![text("Guides")]),
            tab_nav::link("", false, vec![], vec![text("API")]),
        ],
    );
    section(
        "Tab Nav",
        "pre-styled-ui 単独定義の anatomy（data-scope=\"tab-nav\"）による静的掲示です。role=\"tablist\"/role=\"tab\" を出力せず、素の nav/a の暗黙 ARIA ロールのみを使います。Overview を現在ページ（aria-current=\"page\"）として掲示しています。size 軸（既定 md、イシュー #1541）・hover・フォーカスリングも備えます。",
        vec![node],
    )
}

/// Link 節（イシュー #1154）: 唯一の anatomy パーツ `root` を 1 デモに
/// 全網羅する（Anatomy 節はデモ HTML から機械導出されるため、`link.rs` が
/// 持つ全パーツを描画する）。Plain/Underline の 2 variant・現在ページ
/// （`aria-current="page"`）・外部リンク（`target="_blank"` +
/// `rel="noopener noreferrer"`）の 4 例を並べる。`href` は 4 例すべて
/// 空文字列固定とする（`showcase_markup_has_no_href_attributes_for_linkcheck_neutrality`
/// が `showcase_body()` 全体を横断走査して非空 `href` の不在を検証するため、
/// 個別ページの linkcheck 対象外である外部 URL であっても本テストの対象
/// からは逃れられない。`external=true` の効果（`target`/`rel` の付与）は
/// `href` の値に依存しないため、空文字列のままでも掲示として成立する）。
fn link_section() -> Node {
    let node = row(vec![
        link::root("", &LinkProps::default(), vec![], vec![text("Plain link")]),
        link::root(
            "",
            &LinkProps {
                variant: LinkVariant::Underline,
                ..LinkProps::default()
            },
            vec![],
            vec![text("Underline link")],
        ),
        link::root(
            "",
            &LinkProps {
                current: true,
                ..LinkProps::default()
            },
            vec![],
            vec![text("Current page")],
        ),
        link::root(
            "",
            &LinkProps {
                external: true,
                ..LinkProps::default()
            },
            vec![],
            vec![text("External link")],
        ),
        // イシュー #1437: ColorPalette 軸新設のデモ（Accent 以外 1 例、
        // 視覚確認可能にする）。
        link::root(
            "",
            &LinkProps {
                palette: ColorPalette::Danger,
                ..LinkProps::default()
            },
            vec![],
            vec![text("Danger palette link")],
        ),
    ]);
    section(
        "Link",
        "pre-styled-ui 単独定義の anatomy（data-scope=\"link\"）による静的掲示です。Plain（既定・下線なし）/Underline（常時下線）の 2 variant、Current page（aria-current=\"page\"）、External link（target=\"_blank\" + rel=\"noopener noreferrer\" を不可分に付与）、ColorPalette（既定 Accent、Danger 例を併記）を並べています。",
        vec![node],
    )
}

/// Link Overlay 節（イシュー #1154）: root/overlay の 2 anatomy パーツを
/// 1 デモに全網羅する（Anatomy 節はデモ HTML から機械導出されるため、
/// `link_overlay.rs` が持つ全パーツを描画する）。`root` はカード状の見出し
/// テキスト・説明文（`overlay` 以外の子ノード）で高さを確立し、`overlay`
/// がカード全面へ展開されるリンクとして重なる構成です。`href` は
/// linkcheck 中立性契約に従い空文字列固定とする。
fn link_overlay_section() -> Node {
    // `link_overlay::root` は呼び出し側 `class` を `drop_class_attr` で除去
    // する（pre-styled-ui の class 制御方針、recipe CSS の外部上書き防止）。
    // そのため stack レイアウト（max-width・gap・margin）は `class` を
    // `root` へ直接渡すのではなく `stack()` ヘルパで別要素として外側から
    // ラップして与える（Bugbot 指摘の再発防止）。
    let node = link_overlay::root(
        vec![],
        vec![
            el("h3", vec![], vec![text("Getting started")]),
            el(
                "p",
                vec![],
                vec![text(
                    "プロジェクトの作成から最初のページ公開までの手順です。",
                )],
            ),
            link_overlay::overlay("", vec![("aria-label", "Getting started を開く")], vec![]),
        ],
    );
    section(
        "Link Overlay",
        "pre-styled-ui 単独定義の anatomy（data-scope=\"link-overlay\"）による静的掲示です。root（位置決めコンテキスト）配下の見出し・説明文が高さを確立し、overlay（position: absolute; inset: 0 でカード全面へ展開されるリンク）が重なります。overlay には aria-label でアクセシブルネームを与えています。",
        vec![stack(vec![node])],
    )
}

/// Nav List 節（イシュー #1154）: root/heading/list/item/link の
/// 5 anatomy パーツを 1 デモに全網羅する（Anatomy 節はデモ HTML から
/// 機械導出されるため、1 パーツでも欠けると節が不完全になる）。1 件目
/// （Overview）を現在ページ（`aria-current="page"`）として掲示する。`href`
/// は linkcheck 中立性契約に従い空文字列固定とする。
fn nav_list_section() -> Node {
    let node = nav_list::root(
        "Documentation",
        vec![],
        vec![
            nav_list::heading(vec![], vec![text("Guides")]),
            nav_list::list(
                vec![],
                vec![
                    nav_list::item(
                        vec![],
                        vec![nav_list::link("", true, vec![], vec![text("Overview")])],
                    ),
                    nav_list::item(
                        vec![],
                        vec![nav_list::link(
                            "",
                            false,
                            vec![],
                            vec![text("Installation")],
                        )],
                    ),
                ],
            ),
        ],
    );
    section(
        "Nav List",
        "pre-styled-ui 単独定義の anatomy（data-scope=\"nav-list\"）による静的掲示です。role を一切出力せず、素の nav/h2/ul/li/a の暗黙 ARIA ロールのみを使います。Overview を現在ページ（aria-current=\"page\"）として掲示しています。",
        vec![node],
    )
}

/// Navigation Menu 節（イシュー #993）: root/list/item/trigger/content/link
/// の 6 anatomy パーツを 1 デモに全網羅する（Anatomy 節はデモ HTML から
/// 機械導出されるため、1 パーツでも欠けると節が不完全になる、
/// `crates/docs-site/src/component_specs_overlay.rs` 参照）。
///
/// 1 項目目（Products）は Trigger を開いた状態（`data-state="open"`）で
/// 掲示し Content 内の Link を掲示する。2 項目目（About）はディスクロージャ
/// を持たない単独リンクとし `current: true`（`aria-current="page"`）で
/// アクティブリンク表現を掲示する。`href` は
/// `showcase_markup_has_no_href_attributes_for_linkcheck_neutrality` の
/// linkcheck 中立性契約に従い空文字列固定とする。
fn navigation_menu_section() -> Node {
    // 静的掲示のため状態機械（`NavigationMenu`）は経由せず、headless 層の
    // 自由関数へ `OpenState` を直接渡して組み立てる（`state` 引数を明示
    // 引数で受け取る設計であり、single/multiple いずれの状態機械経由でも
    // 状態機械を経由しない構成でも共用できる、`navigation_menu` モジュール
    // doc 参照）。
    let node = navigation_menu::root(
        "Main",
        vec![],
        vec![navigation_menu::list(
            vec![],
            vec![
                navigation_menu::item(
                    OpenState::Open,
                    false,
                    vec![],
                    vec![
                        navigation_menu::trigger(
                            OpenState::Open,
                            false,
                            "products",
                            Some("nav-menu-products-trigger"),
                            Some("nav-menu-products-content"),
                            vec![],
                            vec![text("Products")],
                        ),
                        navigation_menu::content(
                            OpenState::Open,
                            Some("nav-menu-products-content"),
                            Some("nav-menu-products-trigger"),
                            vec![],
                            vec![
                                navigation_menu::link("", false, vec![], vec![text("Analytics")]),
                                navigation_menu::link("", false, vec![], vec![text("Automation")]),
                            ],
                        ),
                    ],
                ),
                navigation_menu::item(
                    OpenState::Closed,
                    false,
                    vec![],
                    vec![navigation_menu::link("", true, vec![], vec![text("About")])],
                ),
            ],
        )],
    );
    section(
        "Navigation Menu",
        "headless-ui の Navigation Menu（役割は素の nav/ul/li/button/div/a の暗黙 ARIA role に依拠し、role は一切付与しません）に pre-styled-ui の recipe CSS を適用した静的掲示です。Products トリガーを開いた状態（data-state=\"open\"）で Content 内の 2 リンクを掲示し、About は Trigger/Content を持たない単独リンクとして aria-current=\"page\" によるアクティブリンク表現を掲示します。viewport 測定・data-motion は headless 層に存在しないため掲示していません（詳細は headless-ui の navigation_menu モジュール doc を参照）。",
        vec![node],
    )
}

/// Status 節（イシュー #765）: colorPalette 軸ごとのドット + ラベル表示。
fn status_section() -> Node {
    // イシュー #1569: size 行を追加し、ドット径と文字サイズが Xs〜Xl で
    // 連動することを示す（既存の Size Demo パターン、button 節の
    // `(Size::Xs, "Extra Small")` に倣う）。palette は既定 Accent のまま。
    let sizes = [
        (Size::Xs, "Extra Small"),
        (Size::Sm, "Small"),
        (Size::Md, "Medium"),
        (Size::Lg, "Large"),
        (Size::Xl, "Extra Large"),
    ];
    let size_row = row(sizes
        .iter()
        .map(|(size, label)| {
            status::root(
                &StatusProps {
                    size: *size,
                    ..StatusProps::default()
                },
                vec![],
                vec![status::indicator(vec![]), text(*label)],
            )
        })
        .collect());

    // イシュー #1681: 共有 `palettes()`（5 値）はまだ Neutral を含めない
    // （#1680 適用完了まで宣言なしデモの公開を避ける）。この節限定で
    // Neutral エントリを末尾へ連結する。
    let palette_row = row(palettes()
        .iter()
        .copied()
        .chain([(ColorPalette::Neutral, "Neutral")])
        .map(|(palette, label)| {
            status::root(
                &StatusProps {
                    palette,
                    ..StatusProps::default()
                },
                vec![],
                vec![status::indicator(vec![]), text(label)],
            )
        })
        .collect());
    section(
        "Status",
        "ドット（indicator）+ ラベルで状態を示す静的表示。size でドット径と文字サイズが連動し、colorPalette で色を切り替えます。",
        vec![size_row, palette_row],
    )
}

/// EmptyState 節（イシュー #765）: indicator/title/description/actions の
/// 構成例。`actions` 内は `button` を使い `href` を持たせない
/// （`showcase_markup_has_no_href_attributes_for_linkcheck_neutrality` の
/// linkcheck 中立性を維持する）。size 行（イシュー #1560）は Xs〜Xl の
/// 各段で padding・gap・indicator/title/description の文字サイズが連動
/// することを示すため、actions を持たない indicator/title/description の
/// 3 段構成で並べる（Md の既定デモのみ actions を持たせる）。
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
    let sizes = [
        (Size::Xs, "Xs"),
        (Size::Sm, "Sm"),
        (Size::Md, "Md"),
        (Size::Lg, "Lg"),
        (Size::Xl, "Xl"),
    ];
    let size_row = stack(
        sizes
            .iter()
            .map(|(size, label)| {
                let props = EmptyStateProps { size: *size };
                empty_state::root(
                    &props,
                    vec![],
                    vec![empty_state::content(
                        vec![],
                        vec![
                            empty_state::indicator(vec![], vec![text("∅")]),
                            empty_state::title(vec![], vec![text(format!("{label} size"))]),
                            empty_state::description(vec![], vec![text("No results found.")]),
                        ],
                    )],
                )
            })
            .collect(),
    );
    section(
        "EmptyState",
        "indicator / title / description / actions で構成する空状態レイアウト。colorPalette 軸は持たない中立コンテナです。size（xs〜xl）は root の padding・gap・indicator/title/description の文字サイズが連動します。",
        vec![node, size_row],
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
///
/// 読み替え（イシュー #1155）: 上記は「本ファイル（`showcase.rs`）の節と
/// してはデモを設けない」の意味であり、`/themes/skip-nav/` 部品ページ自体
/// が Demo 節を持たないわけではない。同ページは
/// `crate::component_specs::interactive_utilities::demo_skip_nav`（Demo
/// フォールバック、#979）がカスタム id で別途 Demo を供給する（本関数が
/// 追加する `id="fandhe-skip-nav"` の重複は起こさない）。
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
    use fandhe_frontend_pre_styled_ui::progress::{self, ProgressProps, ProgressVariant};

    fn linear_demo(
        p: &Progress,
        props: &ProgressProps,
        label_text: &str,
        value_text: &str,
    ) -> Node {
        progress::root(
            p,
            props,
            Some(value_text),
            vec![("style", "max-width: 20rem;")],
            vec![
                p.label(vec![], vec![text(label_text)]),
                p.value_text(vec![], vec![text(value_text)]),
                p.track(vec![], vec![progress::range(p, vec![])]),
            ],
        )
    }

    fn circle_demo(p: &Progress, props: &ProgressProps, aria_valuetext: Option<&str>) -> Node {
        progress::root(
            p,
            props,
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
    let basic_row = row(vec![linear_demo(
        &determinate,
        &ProgressProps::default(),
        "Upload",
        "40%",
    )]);

    let size_row = stack(
        vec![Size::Xs, Size::Sm, Size::Md, Size::Lg, Size::Xl]
            .into_iter()
            .map(|size| {
                let props = ProgressProps {
                    size,
                    ..ProgressProps::default()
                };
                linear_demo(&determinate, &props, size.value(), "40%")
            })
            .collect(),
    );

    let variant_row = row(vec![
        linear_demo(
            &determinate,
            &ProgressProps {
                variant: ProgressVariant::Outline,
                ..ProgressProps::default()
            },
            "Outline",
            "40%",
        ),
        linear_demo(
            &determinate,
            &ProgressProps {
                variant: ProgressVariant::Subtle,
                ..ProgressProps::default()
            },
            "Subtle",
            "40%",
        ),
    ]);

    let palette_row = stack(
        [
            ("Accent", ColorPalette::Accent),
            ("Info", ColorPalette::Info),
            ("Success", ColorPalette::Success),
            ("Warning", ColorPalette::Warning),
            ("Danger", ColorPalette::Danger),
            ("Neutral", ColorPalette::Neutral),
        ]
        .into_iter()
        .map(|(label_text, palette)| {
            let props = ProgressProps {
                palette,
                ..ProgressProps::default()
            };
            linear_demo(&determinate, &props, label_text, "40%")
        })
        .collect(),
    );

    let indeterminate = Progress::new(0.0, 100.0, None, Orientation::Horizontal);
    let indeterminate_row = row(vec![linear_demo(
        &indeterminate,
        &ProgressProps::default(),
        "Loading",
        "",
    )]);

    let complete = Progress::new(0.0, 100.0, Some(100.0), Orientation::Horizontal);
    let complete_row = row(vec![linear_demo(
        &complete,
        &ProgressProps::default(),
        "Complete",
        "100%",
    )]);

    let vertical = Progress::new(0.0, 100.0, Some(65.0), Orientation::Vertical);
    let vertical_row = row(vec![progress::root(
        &vertical,
        &ProgressProps::default(),
        Some("65%"),
        vec![("style", "height: 12rem;")],
        vec![vertical.track(vec![], vec![progress::range(&vertical, vec![])])],
    )]);

    let circle_row = row(vec![
        circle_demo(&determinate, &ProgressProps::default(), Some("40%")),
        circle_demo(
            &complete,
            &ProgressProps {
                size: Size::Sm,
                ..ProgressProps::default()
            },
            Some("100%"),
        ),
        circle_demo(&indeterminate, &ProgressProps::default(), None),
    ]);

    section(
        "Progress",
        "Linear（Track/Range）と Circular（SVG）両対応の進捗インジケータ。size（xs〜xl）で --fandhe-progress-track-height/--fandhe-progress-size/--fandhe-progress-thickness を、variant（outline/subtle）で track の見た目を、color-palette（accent/info/success/warning/danger/neutral）で range の塗り色を切り替えます。indeterminate（不定進捗）は data-state=\"indeterminate\" に連動したアニメーション（linear は横スライド、circular は回転）で表示し、prefers-reduced-motion: reduce では停止します。",
        vec![
            basic_row,
            size_row,
            variant_row,
            palette_row,
            indeterminate_row,
            complete_row,
            vertical_row,
            circle_row,
        ],
    )
}

/// QrCode（イシュー #774。overlay 中央固定・size 参照整列はイシュー
/// #1565）節: size（xs/sm/md/lg/xl）5 態・overlay（ロゴ想定の中央
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

    // overlay 付きデモ専用の matrix（イシュー #1565、codex P1 是正）。
    // overlay はロゴ等で QR モジュールの一部を隠す想定であり、誤り訂正
    // レベル M（訂正能力 15%）のまま流用すると overlay 領域の欠損で
    // 読み取り不能になり得る。Q（訂正能力 25%）以上を使う headless 側の
    // 指針（`crates/headless-ui/src/qr_code.rs` doc 参照）に合わせ、
    // overlay 用は別途 Q でエンコードした専用 matrix を使う。
    let overlay_matrix = qr_code::encode(
        "https://fandhe-frontend.example/",
        qr_code::ErrorCorrectionLevel::Q,
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

    let size_row = row(vec![
        demo(Size::Xs),
        demo(Size::Sm),
        demo(Size::Md),
        demo(Size::Lg),
        demo(Size::Xl),
    ]);

    let with_overlay = qr_code::root(
        Size::Lg,
        vec![],
        vec![
            qr_code::frame(
                &overlay_matrix,
                qr_code::DEFAULT_QUIET_ZONE,
                Some("QR code linking to https://fandhe-frontend.example/"),
                vec![],
                vec![qr_code::pattern(
                    &overlay_matrix,
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
        "外部依存ゼロの QR Model 2（ISO/IEC 18004）byte モードエンコーダによる QR コード表示。size（xs/sm/md/lg/xl）で --fandhe-qr-code-size を切り替えます。Overlay パーツは frame 中央に固定サイズ（--fandhe-qr-code-size の 1/3）で重なり、背景・角丸付きでロゴ等の呼び出し側コンテンツの可読性を確保します。",
        vec![size_row, overlay_row],
    )
}

/// Image 節（イシュー #770）の demo アセット（イシュー #1562）の出力先
/// （`out_dir` 起点の相対パス）。`crate::build::build_site` が
/// [`image_demo_svg`] の内容をこのパスへ書き出す。
///
/// `data:` URI は `fandhe_frontend_core::url::is_safe_url`（許可スキームは
/// http/https/mailto/tel と相対 URL のみ、REQ-1）が拒否し `src` 属性
/// ごと出力から落ちる（core の URL 検証は一元化されており本クレート側で
/// 迂回・複製しない）。旧実装が `data:image/svg+xml,...` を使っていたため
/// Demo が「壊れた画像アイコン」表示になっていた不具合（#1562 で発覚）を、
/// ビルド時生成の相対パスアセットへ切り替えることで是正する。
pub(crate) const IMAGE_DEMO_ASSET_REL_PATH: &str = "assets/image-demo.svg";

/// [`image_section`] から `/themes/image/` ページへ渡す `src`（ページ深さ
/// 2 階層上、`/themes/<kebab>/` → `assets/` の相対パス）。ページの実際の
/// 出力先が変わる場合はこの相対パス段数も追従が必要（現状は全部品ページが
/// 同じ深さのため固定値で足りる）。
pub(crate) const IMAGE_DEMO_SRC: &str = "../../assets/image-demo.svg";

/// Image 節の demo 用プレースホルダー SVG（イシュー #1562）。ユーザー入力を
/// 一切含まない固定図形のみを [`fandhe_frontend_core`] のノード木 API
/// （`el`/`render`）で組み立てる（`format!` による SVG 文字列直組みは
/// REQ-1 が禁じるパターンのため使わない）。cover/contain の差・
/// landscape/portrait の縦横比の違いが視認できるよう、空・丘・太陽の単純な
/// 図形を 16:10 の横長キャンバスへ描く。
#[must_use]
pub(crate) fn image_demo_svg() -> String {
    let svg = el(
        "svg",
        vec![
            ("xmlns", "http://www.w3.org/2000/svg"),
            ("viewBox", "0 0 160 100"),
            ("role", "img"),
            ("aria-label", "Placeholder landscape illustration"),
        ],
        vec![
            // 空。
            el(
                "rect",
                vec![("width", "160"), ("height", "100"), ("fill", "#bfe3f7")],
                vec![],
            ),
            // 太陽。
            el(
                "circle",
                vec![
                    ("cx", "128"),
                    ("cy", "24"),
                    ("r", "14"),
                    ("fill", "#fbd35a"),
                ],
                vec![],
            ),
            // 丘。
            el(
                "path",
                vec![
                    ("d", "M0 68 Q40 40 80 68 T160 68 V100 H0 Z"),
                    ("fill", "#4a9d5c"),
                ],
                vec![],
            ),
            // 手前の丘（奥行きを出す 2 層目）。
            el(
                "path",
                vec![
                    ("d", "M0 82 Q50 58 100 82 T160 78 V100 H0 Z"),
                    ("fill", "#2f7d43"),
                ],
                vec![],
            ),
        ],
    );
    render(&svg)
}

/// Image 節: `fit`（object-fit）× `aspect_ratio` × `shape`（角丸）の 3 軸
/// （イシュー #1562 で `shape` 追加・chakra-ui 公式デモ構成へ寄せた
/// 4 行構成に再編。基本デモ・shape 3 種・fit 5 種・aspect-ratio 5 種）。
fn image_section() -> Node {
    let basic_row = row(vec![image(
        &ImageProps {
            shape: ImageShape::Rounded,
            ..ImageProps::new(IMAGE_DEMO_SRC, "Basic rounded image")
        },
        vec![("style", "width: 12rem;")],
    )]);

    let shapes = [
        (ImageShape::Square, "Square"),
        (ImageShape::Rounded, "Rounded"),
        (ImageShape::Circle, "Circle"),
    ];
    let shape_row = row(shapes
        .iter()
        .map(|(shape, label)| {
            image(
                &ImageProps {
                    shape: *shape,
                    aspect_ratio: AspectRatio::Square,
                    ..ImageProps::new(IMAGE_DEMO_SRC, label)
                },
                vec![("style", "width: 6rem;")],
            )
        })
        .collect());

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
        (AspectRatio::Landscape, "Landscape"),
        (AspectRatio::Portrait, "Portrait"),
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
        "写真等の静的コンテンツを表示する img の styled ラッパー。fit（object-fit）・aspect-ratio・shape（角丸）を型安全な props で切り替えます。状態機械は持たず、avatar の ImageStatus とは独立です。",
        vec![basic_row, shape_row, fit_row, ratio_row],
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

    let size_row = row(vec![Size::Xs, Size::Sm, Size::Md, Size::Lg, Size::Xl]
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

    // イシュー #1561: color: currentColor 継承の視認確認用デモ。style
    // 値は theme.rs 実在のトークン参照（var(...)）のみで、生の色リテラル・
    // ユーザー入力は混ぜない（既存 showcase の style リテラル慣行と同型）。
    let current_color_row = row(vec![
        ("accent", "--fandhe-color-accent"),
        ("danger", "--fandhe-color-danger"),
        ("warning", "--fandhe-color-warning"),
    ]
    .into_iter()
    .map(|(label, token)| {
        el(
            "span",
            vec![("style", &format!("color: var({token});"))],
            vec![icon(
                &IconProps {
                    label: Some(label),
                    ..IconProps::default()
                },
                vec![],
                vec![star_path()],
            )],
        )
    })
    .collect());

    section(
        "Icon",
        "インライン SVG の寸法（size は Xs〜Xl の 5 段、既定 Md、chakra-ui 同名段の実寸に整合。イシュー #1561）・配色（color: currentColor 継承）を統一する svg ラッパー。SVG 本体（path 等）は呼び出し側がノード木 API で構築します。",
        vec![size_row, current_color_row],
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
    // イシュー #1572: size の全 5 段（Xs〜Xl）を実演する（padding が
    // `--fandhe-space-*` トークン化されたことを Demo でも確認できるように
    // する、`table.rs` モジュール doc「variant について」節参照）。
    let size_demo = stack(vec![
        sample_table(TableProps {
            size: Size::Xs,
            ..TableProps::default()
        }),
        sample_table(TableProps {
            size: Size::Sm,
            ..TableProps::default()
        }),
        sample_table(TableProps {
            size: Size::Md,
            ..TableProps::default()
        }),
        sample_table(TableProps {
            size: Size::Lg,
            ..TableProps::default()
        }),
        sample_table(TableProps {
            size: Size::Xl,
            ..TableProps::default()
        }),
    ]);
    let striped_demo = stack(vec![sample_table(TableProps {
        striped: true,
        ..TableProps::default()
    })]);
    // イシュー #1572: `scroll_area` + `sticky_header: true` を組み合わせた
    // Demo。行数を増やしてスクロール枠内で見出し行が上端固定されることを
    // 視覚的に確認できるようにする（`table.rs` モジュール doc「sticky
    // ヘッダーの実装」節・「`scroll-area` パーツ」節参照）。Anatomy 表・
    // `data-*` 属性表はこの Demo から機械導出されるため、`scroll-area` を
    // 必ず含める。
    let scroll_area_demo = stack(vec![table::scroll_area(
        vec![("style", "--fandhe-table-scroll-max-height: 12rem")],
        vec![table::root(
            TableProps {
                sticky_header: true,
                ..TableProps::default()
            },
            vec![],
            vec![
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
                    (1..=10)
                        .map(|n| {
                            table::row(
                                vec![],
                                vec![
                                    table::cell(vec![], vec![text(format!("User {n}"))]),
                                    table::cell(vec![], vec![text(format!("user{n}@example.com"))]),
                                    table::cell(vec![], vec![text("Member")]),
                                ],
                            )
                        })
                        .collect(),
                ),
            ],
        )],
    )]);

    section(
        "Table",
        "table/thead/tbody/tfoot/tr/th/td/caption の HTML 意味論を尊重した表組み。variant（line / outline）・size（xs 〜 xl）・striped・sticky_header の 4 軸 variant と、scroll_area（chakra Table.ScrollArea 相当のスクロール枠）を持ちます。",
        vec![variant_demo, size_demo, striped_demo, scroll_area_demo],
    )
}

/// DataList 節: orientation（vertical/horizontal）・variant（subtle/bold）・
/// size（xs〜xl）の 3 軸（イシュー #1559 で variant/size を追加）。
fn data_list_section() -> Node {
    fn sample_data_list(props: DataListProps) -> Node {
        data_list::root(
            props,
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

    let orientation_demos = stack(vec![
        sample_data_list(DataListProps {
            orientation: DataListOrientation::Vertical,
            ..DataListProps::default()
        }),
        sample_data_list(DataListProps {
            orientation: DataListOrientation::Horizontal,
            ..DataListProps::default()
        }),
    ]);

    let variants = [
        (DataListVariant::Subtle, "Subtle"),
        (DataListVariant::Bold, "Bold"),
    ];
    let variant_row = stack(
        variants
            .iter()
            .map(|(variant, _label)| {
                sample_data_list(DataListProps {
                    variant: *variant,
                    ..DataListProps::default()
                })
            })
            .collect(),
    );

    let sizes = [
        (Size::Xs, "Xs"),
        (Size::Sm, "Sm"),
        (Size::Md, "Md"),
        (Size::Lg, "Lg"),
        (Size::Xl, "Xl"),
    ];
    let size_row = stack(
        sizes
            .iter()
            .map(|(size, _label)| {
                sample_data_list(DataListProps {
                    size: *size,
                    ..DataListProps::default()
                })
            })
            .collect(),
    );

    section(
        "DataList",
        "dl/dt/dd の定義リスト意味論を尊重したラベル・値の一覧表示。orientation（vertical / horizontal）・variant（subtle / bold）・size（xs〜xl）の 3 軸 variant を持ちます。",
        vec![orientation_demos, variant_row, size_row],
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
    // size の段差を視覚確認できるよう Sm/Md/Lg を並べる（イシュー #1568）。
    let size_demo = row(vec![
        stat::root(
            Size::Sm,
            vec![],
            vec![
                stat::label(vec![], vec![text("Sm")]),
                stat::value_text(vec![], vec![text("1,234")]),
            ],
        ),
        stat::root(
            Size::Md,
            vec![],
            vec![
                stat::label(vec![], vec![text("Md")]),
                stat::value_text(vec![], vec![text("1,234")]),
            ],
        ),
        stat::root(
            Size::Lg,
            vec![],
            vec![
                stat::label(vec![], vec![text("Lg")]),
                stat::value_text(vec![], vec![text("1,234")]),
            ],
        ),
    ]);
    section(
        "Stat",
        "数値指標 1 件をラベル・値・補助テキスト・増減方向インジケーターの組で表示する静的部品です。size（xs〜xl、既定 md。chakra-ui の sm/md/lg は本実装の Sm/Md/Lg に対応）で value-text のフォントサイズを切り替えます。",
        vec![demo, size_demo],
    )
}

/// Timeline 節: 状態機械不要の静的部品（イシュー #769）。`variant`/`size`/
/// `color-palette` の 3 軸。
fn timeline_section() -> Node {
    // イシュー #1575: `data-state="complete"`/`"current"` は recipe 側では
    // なく呼び出し側が付与する契約（`crate::timeline` rustdoc「`data-state`
    // 契約」節参照）。3 item 構成（complete → current → 未着手）にして
    // `indicator`/`separator` の状態別スタイルが Demo・`data-*` 属性表の
    // 双方へ可視化されるようにする。
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
                            timeline::indicator(vec![data_state("complete")], vec![]),
                            timeline::separator(vec![data_state("complete")], vec![]),
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
                    timeline::connector(
                        vec![],
                        vec![
                            timeline::indicator(vec![data_state("current")], vec![]),
                            timeline::separator(vec![], vec![]),
                        ],
                    ),
                    timeline::content(
                        vec![],
                        vec![
                            timeline::title(vec![], vec![text("開発中")]),
                            timeline::description(vec![], vec![text("2026-03-01")]),
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
        "時系列に並ぶ出来事の一覧を connector（縦線）+ indicator（点）+ content で表示する静的部品です。variant（solid/subtle/outline/plain）で indicator の塗り方を切り替えます。呼び出し側が indicator/separator へ data-state=\"complete\"/\"current\" を付与すると、完了区間・現在位置のスタイルが適用されます。",
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
    // イシュー #1582: 両端フェード（`mask-image`）の opt-in デモ。
    // 参照サイト（ark-ui/chakra-ui）に `Edge` 幅の既定値記載がないため、
    // テキストティッカーで数文字分のフェードが視認できる `--fandhe-space-12`
    // を採用した。`--fandhe-marquee-gap` の上書きも併記し、両 custom
    // property が独立して調整可能なことを示す。
    let fade_demo = marquee::marquee(
        &MarqueeProps::default(),
        vec![(
            "style",
            "--fandhe-marquee-fade: var(--fandhe-space-12); --fandhe-marquee-gap: var(--fandhe-space-8);",
        )],
        vec![marquee::item(
            vec![],
            vec![text("両端フェード（--fandhe-marquee-fade）の例です。")],
        )],
    );
    section(
        "Marquee",
        "CSS のみ（JS ゼロ）の自動流動テキストです。direction（既定/end）でスクロール方向を切り替え、hover/focus-within で常時一時停止、prefers-reduced-motion: reduce 環境では停止します。decorative: true で装飾用途（aria-hidden）に、--fandhe-marquee-duration の上書きで速度を、--fandhe-marquee-gap の上書きで間隔を調整できます。--fandhe-marquee-fade（既定 0px）を上書きすると両端がフェードします。",
        vec![default_demo, end_demo, decorative_demo, fade_demo],
    )
}

/// ScrollArea 節（イシュー #825）: `overflow: auto` によるネイティブスクロール
/// とカスタムスクロールバー表現（`scrollbar-width`/`::-webkit-scrollbar`）。
/// JS によるスクロール位置追従は本イシューのスコープ外（`crate::scroll_area`
/// rustdoc 参照）のため、固定高の viewport と長文 content のみを掲示する。
fn scroll_area_section() -> Node {
    let items = || -> Vec<Node> {
        (1..=20)
            .map(|i| el("p", vec![], vec![text(format!("スクロール可能な行 {i}"))]))
            .collect()
    };
    let demo = scroll_area::root(
        vec![(
            "style",
            "height: 8rem; width: 16rem; border: 1px solid var(--fandhe-color-border);",
        )],
        vec![scroll_area::viewport(
            vec![],
            vec![scroll_area::content(vec![], items())],
        )],
    );
    // 2 例目: root へ --fandhe-scroll-area-thumb-bg: transparent を指定し、
    // chakra-ui の variant="hover"（既定は常時非表示、hover 時のみ出現）
    // 相当の見た目を custom property 上書きだけで再現するデモ（イシュー
    // #1584。variant 軸自体は新設しない判断の根拠を実演する）。
    let hover_reveal_demo = scroll_area::root(
        vec![(
            "style",
            "height: 8rem; width: 16rem; border: 1px solid var(--fandhe-color-border); --fandhe-scroll-area-thumb-bg: transparent;",
        )],
        vec![scroll_area::viewport(
            vec![],
            vec![scroll_area::content(vec![], items())],
        )],
    );
    section(
        "ScrollArea",
        "CSS overflow を主体としたスクロール領域です。カスタムスクロールバーの見た目は scrollbar-width/scrollbar-color と ::-webkit-scrollbar 系規則で表現し、thumb 色は custom property --fandhe-scroll-area-thumb-bg で一元化しています（hover 時は --fandhe-scroll-area-thumb-hover-bg へ強調。JS によるスクロール位置追従は対象外）。",
        vec![
            demo,
            el(
                "p",
                vec![],
                vec![text(
                    "--fandhe-scroll-area-thumb-bg: transparent を指定すると、chakra-ui の variant=\"hover\" 相当（既定は非表示、hover 時のみ出現）を再現できます。",
                )],
            ),
            hover_reveal_demo,
        ],
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

    // 90 秒カウントダウン、1 秒 tick。35 秒経過（残り 55 秒）まで進めた
    // running 状態を固定掲示する。
    let mut running = Timer::countdown(90_000, 1_000);
    dispatch(&mut running, "timer:start", "");
    dispatch(&mut running, "timer:tick", "35000");
    let running_node = timer_display_node(&running, "Min", "Sec");

    // 5 秒カウントダウンを 5000ms tick で completed まで進めた状態を並べ、
    // イシュー #1577 で是正した root[data-state="completed"] →
    // --fandhe-timer-value-color の値色切り替え（accent）が視覚確認できる
    // ようにする。
    let mut completed = Timer::countdown(5_000, 1_000);
    dispatch(&mut completed, "timer:start", "");
    dispatch(&mut completed, "timer:tick", "5000");
    debug_assert_eq!(completed.phase(), TimerPhase::Completed);
    let completed_node = timer_display_node(&completed, "Min", "Sec");

    section(
        "Timer",
        "headless-ui の Timer（tick 注入型・idle/running/paused/completed の決定的状態機械）に pre-styled-ui のセグメント表示（分:秒）CSS を適用した静的掲示です。左は 90 秒のカウントダウンを開始して 35 秒経過した running 状態、右は 5 秒のカウントダウンが completed に達した状態（root の data-state に応じて item-value の色が accent へ切り替わる）を固定表示しています。実 tick 駆動（setInterval）は fandhe-frontend-wasm-full::headless_timer のスコープです。",
        vec![row(vec![running_node, completed_node])],
    )
}

/// [`timer_section`] が running / completed の 2 状態で共有する表示ツリー
/// 組み立てヘルパ（分:秒セグメント + control）。
fn timer_display_node(t: &Timer, minutes_label: &str, seconds_label: &str) -> Node {
    let (_, _, minutes, seconds) = t.display_segments();
    t.root(
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
                            timer::item_label(
                                TimerUnit::Minutes,
                                vec![],
                                vec![text(minutes_label)],
                            ),
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
                            timer::item_label(
                                TimerUnit::Seconds,
                                vec![],
                                vec![text(seconds_label)],
                            ),
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

    let size_row = row([Size::Xs, Size::Sm, Size::Md, Size::Lg, Size::Xl]
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

    let size_row = row([Size::Xs, Size::Sm, Size::Md, Size::Lg, Size::Xl]
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
        (TagVariant::Surface, "Surface"),
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
    let size_row = row([Size::Xs, Size::Sm, Size::Md, Size::Lg, Size::Xl]
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
    // イシュー #1681: 共有 `palettes()`（5 値）はまだ Neutral を含めない
    // （Forms 側〔#1680〕の適用完了まで宣言なしデモが公開されるのを避ける
    // ため）。この節限定で Neutral エントリを末尾へ連結する。
    let palette_row = row(palettes()
        .iter()
        .copied()
        .chain([(ColorPalette::Neutral, "Neutral")])
        .map(|(palette, label)| {
            tag::root(
                &TagProps {
                    palette,
                    ..TagProps::default()
                },
                vec![],
                vec![text(label)],
            )
        })
        .collect());
    // イシュー #1573: close-trigger の hover/フォーカスリングを variant
    // 4 種で並べて目視確認できるようにする（キーボードフォーカスは
    // Tab キーで close-trigger まで移動して確認する）。
    let closable_row = row(variants
        .iter()
        .map(|(variant, label)| {
            tag::root(
                &TagProps {
                    variant: *variant,
                    ..TagProps::default()
                },
                vec![],
                vec![
                    tag::label(vec![], vec![text(*label)]),
                    tag::close_trigger(
                        Some("remove_tag"),
                        vec![("aria-label", "Remove")],
                        vec![text("×")],
                    ),
                ],
            )
        })
        .collect());
    section(
        "Tag",
        "ラベル・分類・除去可能なチップ表示。variant / size / colorPalette を組み合わせます。close-trigger は data-action 属性の出力のみを担い、実際のクリック処理は wasm 層のスコープです。",
        vec![variant_row, size_row, palette_row, closable_row],
    )
}

/// Kbd 節（イシュー #768、#1436 で variant/size/colorPalette 軸を追加）:
/// キーボード入力・ショートカット表示のための単一 slot 静的部品。
fn kbd_section() -> Node {
    let shortcut_row = row(vec![
        kbd(&KbdProps::default(), vec![], vec![text("Ctrl")]),
        text(" + "),
        kbd(&KbdProps::default(), vec![], vec![text("K")]),
    ]);
    let variants = [
        (KbdVariant::Raised, "Raised"),
        (KbdVariant::Subtle, "Subtle"),
        (KbdVariant::Outline, "Outline"),
    ];
    let variant_row = row(variants
        .iter()
        .map(|(variant, label)| {
            kbd(
                &KbdProps {
                    variant: *variant,
                    ..KbdProps::default()
                },
                vec![],
                vec![text(*label)],
            )
        })
        .collect());
    let size_row = row([Size::Xs, Size::Sm, Size::Md, Size::Lg, Size::Xl]
        .iter()
        .map(|size| {
            kbd(
                &KbdProps {
                    size: *size,
                    ..KbdProps::default()
                },
                vec![],
                vec![text("Esc")],
            )
        })
        .collect());
    // 共有 `palettes()`（5 値、Neutral なし）に本部品の既定 palette
    // （Neutral）を末尾連結する（code::section と同様の理由）。
    let palette_row = row(palettes()
        .iter()
        .copied()
        .chain([(ColorPalette::Neutral, "Neutral")])
        .map(|(palette, label)| {
            kbd(
                &KbdProps {
                    palette,
                    ..KbdProps::default()
                },
                vec![],
                vec![text(label)],
            )
        })
        .collect());
    section(
        "Kbd",
        "キーボード入力・ショートカット表示。variant / size / colorPalette を組み合わせます。",
        vec![shortcut_row, variant_row, size_row, palette_row],
    )
}

/// Code 節（イシュー #768、#1432 で variant/size/colorPalette 軸を追加）:
/// インライン `<code>` のみを扱う（CodeBlock は対象外確定済み）。
fn code_section() -> Node {
    let variants = [
        (CodeVariant::Solid, "Solid"),
        (CodeVariant::Subtle, "Subtle"),
        (CodeVariant::Outline, "Outline"),
    ];
    let variant_row = row(variants
        .iter()
        .map(|(variant, label)| {
            code(
                &CodeProps {
                    variant: *variant,
                    ..CodeProps::default()
                },
                vec![],
                vec![text(*label)],
            )
        })
        .collect());
    let size_row = row([Size::Xs, Size::Sm, Size::Md, Size::Lg, Size::Xl]
        .iter()
        .map(|size| {
            code(
                &CodeProps {
                    size: *size,
                    ..CodeProps::default()
                },
                vec![],
                vec![text("code")],
            )
        })
        .collect());
    // 共有 `palettes()`（5 値、Neutral なし）に本部品の既定 palette
    // （Neutral）を末尾連結する（tag::section と同様の理由）。
    let palette_row = row(palettes()
        .iter()
        .copied()
        .chain([(ColorPalette::Neutral, "Neutral")])
        .map(|(palette, label)| {
            code(
                &CodeProps {
                    palette,
                    ..CodeProps::default()
                },
                vec![],
                vec![text(label)],
            )
        })
        .collect());
    section(
        "Code",
        "インラインコード片の表示。variant / size / colorPalette を組み合わせます。chakra-ui の CodeBlock 相当は対象外です。",
        vec![variant_row, size_row, palette_row],
    )
}

/// ColorSwatch 節（イシュー #838）: size / shape の掲示と、透過色の
/// チェッカーボード表示確認。
fn color_swatch_section() -> Node {
    let blue = Color::from_rgb(Rgb::new(0x3b, 0x82, 0xf6));
    let size_row = row([Size::Xs, Size::Sm, Size::Md, Size::Lg, Size::Xl]
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
    // イシュー #1558: chakra-ui 参照スクショ 3（半透明 4 色）相当に拡充し、
    // 淡色・低アルファ色でも #1558 で追加した inset リングの輪郭が
    // 判別できることを確認できるようにする。
    let transparent_row = row(vec![
        Color::from_rgba(Rgb::new(0xff, 0x00, 0x00), 0x80),
        Color::from_rgba(Rgb::new(0x00, 0x00, 0xff), 0xb3),
        Color::from_rgba(Rgb::new(0x00, 0x80, 0x00), 0x66),
        Color::from_rgba(Rgb::new(0xff, 0xc0, 0xcb), 0x99),
    ]
    .into_iter()
    .map(|value| {
        color_swatch::color_swatch(
            &ColorSwatchProps {
                value,
                ..ColorSwatchProps::default()
            },
            vec![],
            vec![],
        )
    })
    .collect());
    section(
        "ColorSwatch",
        "色見本の静的表示です。size / shape を組み合わせられます。半透明色は下地のチェッカーボード模様で透過が、内側 1px の輪郭リングで淡色でも外形が視認できます。",
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
    // イシュー #1463: 閉状態の trigger（通常 / disabled）を Demo へ追加する。
    // 開状態の `content`（既存の `demo` 行）は positioner が `hidden` を
    // 出さない疑似的な「常に開いている」表示のため、trigger 自体は元々
    // 一度も描画されていなかった（親イシュー #1462 の指摘事項）。
    let closed = row(vec![color_picker::root(
        &state,
        vec![],
        vec![
            color_picker::label(vec![], vec![text("Color")]),
            color_picker::control(
                vec![],
                vec![
                    color_picker::channel_input(state.hex().as_str(), false, vec![]),
                    color_picker::trigger(&state, false, None, vec![], vec![]),
                ],
            ),
        ],
    )]);
    let closed_disabled = row(vec![color_picker::root(
        &state,
        vec![],
        vec![
            color_picker::label(vec![], vec![text("Color (disabled)")]),
            color_picker::control(
                vec![],
                vec![
                    color_picker::channel_input(state.hex().as_str(), true, vec![]),
                    color_picker::trigger(&state, true, None, vec![], vec![]),
                ],
            ),
        ],
    )]);
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
        "HSV 色相環 + アルファ選択の静的表示です（canvas 非依存、CSS グラデーション + 検証済み割合のみで構成）。ポインタ操作の実配線は wasm 層の後続対応です。閉状態の trigger（通常 / disabled）と、開状態の content を並べています。",
        vec![closed, closed_disabled, demo],
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
        // 持たない（本文はすべて site/themes.md 側）。
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
        assert!(generated_content("/themes/nonexistent/").is_none());
    }

    #[test]
    fn component_page_paths_are_unique_and_well_formed() {
        let paths: Vec<&str> = component_page_paths().collect();

        // 機械的な分解作業中の取りこぼし・重複追加を fail-closed で検知する
        // 件数センチネル。台帳（`docs/design/docs-site-component-pages.md`）
        // 99 件との突合は #944 の責務。
        // イシュー #993 で Navigation Menu を追加し 92 → 93 件になった。
        // イシュー #994 で Callout を追加し 93 → 94 件になった。
        // イシュー #995 で Quote / Strong を追加し 94 → 96 件になった。
        // イシュー #996 で Tab Nav を追加し 96 → 97 件になった。
        // イシュー #997 で Checkbox Group を追加し 97 → 98 件になった。
        // イシュー #1154 で Link / Link Overlay / Nav List を追加し
        // 98 → 101 件になった。
        assert_eq!(paths.len(), 101, "COMPONENT_PAGES should have 101 entries");

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
                path.starts_with("/themes/") && path.ends_with('/'),
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
            "callout",
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
        // Menubar（イシュー #992、PR #1000 Bugbot 指摘 1 対応）: File Menu を
        // `open=Some(0)` で固定掲示するため、他のオーバーレイ positioner と
        // 同様にフロー内配置へ中和されていることを固定する（回帰防止）。
        assert!(
            css.contains(r#".pre-styled-showcase [data-scope="menubar"][data-part="positioner"]"#)
        );
        // PR #1000 Bugbot 指摘（HEAD ef93488 に対する新規指摘、review
        // comment id 3650231029）の回帰防止: 上記 positioner 中和だけでは
        // 開いた File Menu の `content` が per-menu ラッパー（`root` の flex
        // item）の高さを押し上げ、`align-items: center`（recipe CSS 既定）
        // のままだと Edit トリガーが File トリガーの行から外れて縦にずれる
        // （「水平な menubar に見えない」回帰）。`root` の `align-items` を
        // `flex-start` へ上書きし、各 `menu` flex item の上端（= 各
        // `trigger` の位置）を揃えてトリガー行を保つルールが出力されて
        // いることを固定する。
        assert!(css.contains(
            r#".pre-styled-showcase [data-scope="menubar"][data-part="root"] {
  align-items: flex-start;
}"#
        ));
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
        // イシュー #1551: title（`h2`）に docs サイトの既定 `h2` スタイル
        // （罫線・letter-spacing）が漏れないよう中和されていることを固定
        // する（`docs/design/reference-screenshots/themes-tour.png` で
        // 「設定」の上に罫線が出ていた回帰の是正）。
        assert!(css.contains(r#".pre-styled-showcase [data-scope="tour"] h2"#));
        // Link Overlay（イシュー #1154、PR #1165 Bugbot 指摘「Link overlay
        // CSS collapses prev-next」の回帰防止）: `link_overlay::stylesheet()`
        // の無条件（`.pre-styled-showcase` スコープなし）出荷は
        // `crate::nav::prev_next_nav` が再利用する同一 headless マーカーの
        // 高さを 0 へ潰す。等価ルールは `.pre-styled-showcase` スコープ付き
        // でのみ出荷されていることを固定する。
        assert!(!css.contains(
            "\n[data-scope=\"link-overlay\"][data-part=\"overlay\"] {\n  position: absolute;"
        ));
        assert!(css.contains(
            r#".pre-styled-showcase [data-scope="link-overlay"][data-part="root"] {
  position: relative;
}"#
        ));
        assert!(css.contains(
            r#".pre-styled-showcase [data-scope="link-overlay"][data-part="overlay"] {
  position: absolute;
  inset: 0;
  z-index: 0;
  border-radius: inherit;
  cursor: pointer;
}"#
        ));
        // Link Overlay の `border-radius: inherit`（root 側、Bugbot 指摘の
        // 是正、PR #1853）/ `:focus-visible` リング（イシュー #1580）:
        // `link_overlay::recipe()` へ追加した新規宣言の scoped 複製が
        // 出荷されていることを固定する。`link_overlay::stylesheet()` 本体
        // と同じ理由（上記コメント）で無条件出荷せず
        // `.pre-styled-showcase` スコープ付きミラーとして個別に持つ契約
        // を固定する。
        assert!(
            !css.contains("\n[data-scope=\"link-overlay\"][data-part=\"overlay\"]:focus-visible {")
        );
        assert!(css.contains(
            r#".pre-styled-showcase [data-scope="link-overlay"][data-part="root"] {
  border-radius: inherit;
}"#
        ));
        assert!(css.contains(
            r#".pre-styled-showcase [data-scope="link-overlay"][data-part="overlay"]:focus-visible {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}"#
        ));
        // Link Overlay の素の h3（イシュー #1154、PR #1165 Bugbot 指摘 2 回目
        // 「Link Overlay heading over-reset」の回帰防止）: カード見出し h3 が
        // Accordion トリガー用のフルリセット（font-size/font-weight/
        // line-height/letter-spacing 込み）を誤って継承し、隣接する p（説明
        // 文）と見た目が同化してしまう欠陥の再発を防ぐ。margin-top のみを
        // longhand で打ち消し、見出しらしさ（font-size/font-weight）は
        // `.docs-content h3` のまま活かす契約を固定する。
        assert!(css.contains(
            r#".pre-styled-showcase [data-scope="link-overlay"][data-part="root"] h3 {
  margin-top: 0;
}"#
        ));
        let link_overlay_h3_rule_start = css
            .find(r#".pre-styled-showcase [data-scope="link-overlay"][data-part="root"] h3 {"#)
            .expect("link-overlay root h3 rule must exist");
        let link_overlay_h3_rule_end = css[link_overlay_h3_rule_start..]
            .find('}')
            .map(|offset| link_overlay_h3_rule_start + offset)
            .expect("link-overlay root h3 rule must be closed");
        let link_overlay_h3_rule = &css[link_overlay_h3_rule_start..=link_overlay_h3_rule_end];
        assert!(!link_overlay_h3_rule.contains("font-size"));
        assert!(!link_overlay_h3_rule.contains("font-weight"));
        assert!(!link_overlay_h3_rule.contains("line-height"));
        assert!(!link_overlay_h3_rule.contains("letter-spacing"));
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
