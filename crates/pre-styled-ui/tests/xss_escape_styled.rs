//! `fandhe-frontend-pre-styled-ui` の styled 部品公開 API 経由の XSS 回帰
//! テスト（イシュー #607、REQ-1）。
//!
//! # 本ファイルのスコープ
//!
//! `crates/pre-styled-ui/tests/xss_escape.rs`（イシュー #553）は本クレートが
//! 公開 API を持たなかった時点で headless-ui 経由のフォールバックとして
//! 3 経路（テキスト・属性値・URL 属性）を固定した。その後 #550（button /
//! badge / spinner / alert / card）・#551（dialog / tabs / accordion / menu /
//! select の styled ラッパー）で本クレートの公開 API が揃ったため、本ファイル
//! では「styled 部品の公開 API を実際に呼び出す」形へ回帰テストを拡充する。
//!
//! 対象の入力面（テスト対象になり得る攻撃面）:
//! 1. テキスト経路: `button`/`badge`/`alert::title`/`card::body` の
//!    `children: Vec<Node>`（`text()` 経由）。
//! 2. 属性値経路 a: `spinner::SpinnerProps::label` のように部品内部で
//!    `aria-*` へ透過するプロパティ文字列。
//! 3. 属性値経路 b: 呼び出し側 `attrs: Vec<(&str, &str)>`（`data-testid` 等）。
//! 4. 属性値経路 c: 呼び出し側 `attrs` の `class`。`class_attr::drop_class_attr`
//!    契約により生ペイロードが動的クラス名合成へ混入しないことを固定する
//!    （`crates/pre-styled-ui/src/button.rs` 等の rustdoc 参照）。
//! 5. URL 属性経路: 呼び出し側 `attrs` の `href`/`src` に対する
//!    `fandhe_frontend_core::render` 側の許可リスト検証（deny by default、
//!    `crates/core/src/lib.rs::render_into` の `is_url_attr` 分岐）が styled
//!    部品の attrs 透過経路を通しても貫通することを固定する。
//!
//! 既存の `tests/xss_escape.rs`（headless-ui 経由フォールバック）は削除・
//! 変更しない（本ファイルはそれを補完する独立ファイル）。
//!
//! # 削除・弱体化の禁止
//!
//! `.claude/rules/coding-rust.md` の規約により、本ファイルの XSS 回帰
//! テストは以後の削除・弱体化・`#[ignore]` 化を禁止する。

use fandhe_frontend_core::{el, escape_html, render, text};
use fandhe_frontend_pre_styled_ui::alert::{self, AlertProps};
use fandhe_frontend_pre_styled_ui::area_chart::{area_chart, AreaChartProps, AreaFill, AreaStack};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarBadgeProps};
use fandhe_frontend_pre_styled_ui::badge::{badge, BadgeProps};
use fandhe_frontend_pre_styled_ui::blockquote::{self, BlockquoteVariant};
use fandhe_frontend_pre_styled_ui::button::{button, close_button, icon_button, ButtonProps};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::charts::data::{ChartData, Series};
use fandhe_frontend_pre_styled_ui::charts::legend::{category_legend, LegendProps};
use fandhe_frontend_pre_styled_ui::charts::radar_chart::{self, RadarChartProps};
use fandhe_frontend_pre_styled_ui::charts::scatter_chart::{
    self, ScatterChartProps, ScatterData, ScatterSeries,
};
use fandhe_frontend_pre_styled_ui::checkbox::{self, CheckboxProps};
use fandhe_frontend_pre_styled_ui::checkbox_card;
use fandhe_frontend_pre_styled_ui::clipboard;
use fandhe_frontend_pre_styled_ui::collapsible;
use fandhe_frontend_pre_styled_ui::date_input::{self, DateInputProps, DateSegment};
use fandhe_frontend_pre_styled_ui::donut_chart::{donut_chart, DonutChartProps, PieCenterText};
use fandhe_frontend_pre_styled_ui::download_trigger::{self, DownloadTriggerProps};
use fandhe_frontend_pre_styled_ui::drawer::{self, DrawerPlacement};
use fandhe_frontend_pre_styled_ui::editable::{
    self, EditMode, EditableInputFlags, EditableInputProps,
};
use fandhe_frontend_pre_styled_ui::em::em;
use fandhe_frontend_pre_styled_ui::empty_state::{
    self, EmptyStateIndicatorVariant, EmptyStateProps, EmptyStateVariant,
};
use fandhe_frontend_pre_styled_ui::field::{self, FieldRootProps};
use fandhe_frontend_pre_styled_ui::fieldset::{self, FieldsetRootProps};
use fandhe_frontend_pre_styled_ui::file_upload;
use fandhe_frontend_pre_styled_ui::floating_panel::{self, Stage};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps};
use fandhe_frontend_pre_styled_ui::highlight::{highlight, HighlightProps};
use fandhe_frontend_pre_styled_ui::hover_card::{self, HoverCardDelays};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::image::{image, ImageProps};
use fandhe_frontend_pre_styled_ui::image_cropper;
use fandhe_frontend_pre_styled_ui::input::{self, FieldIds, FieldProps, InputProps};
use fandhe_frontend_pre_styled_ui::line_chart::{line_chart, LineChartProps, LineDots, LineLabel};
use fandhe_frontend_pre_styled_ui::list::{self, ListType, ListVariant};
use fandhe_frontend_pre_styled_ui::listbox;
use fandhe_frontend_pre_styled_ui::mark::{mark, MarkProps};
use fandhe_frontend_pre_styled_ui::marquee::{self, MarqueeProps};
use fandhe_frontend_pre_styled_ui::native_select::{self, NativeSelectProps};
use fandhe_frontend_pre_styled_ui::navigation_menu::{self, NavigationMenuProps};
use fandhe_frontend_pre_styled_ui::number_input::{self, NumberInputFlags};
use fandhe_frontend_pre_styled_ui::pagination::{self, ItemMode};
use fandhe_frontend_pre_styled_ui::password_input::{
    self, PasswordAutocomplete, PasswordInputProps,
};
use fandhe_frontend_pre_styled_ui::pie_chart::{pie_chart, PieChartProps, PieLabelPosition};
use fandhe_frontend_pre_styled_ui::pin_input::{self, PinInputKind, PinInputProps};
use fandhe_frontend_pre_styled_ui::qr_code;
use fandhe_frontend_pre_styled_ui::quote::quote;
use fandhe_frontend_pre_styled_ui::radial_chart::{
    radial_chart, RadialCenterText, RadialChartProps,
};
use fandhe_frontend_pre_styled_ui::radio_card;
use fandhe_frontend_pre_styled_ui::rating_group::{self, RatingItemFlags};
use fandhe_frontend_pre_styled_ui::scroll_area;
use fandhe_frontend_pre_styled_ui::separator::{
    group as separator_group, label as separator_label, separator, SeparatorProps,
};
use fandhe_frontend_pre_styled_ui::signature_pad;
use fandhe_frontend_pre_styled_ui::skeleton::{skeleton, SkeletonProps};
use fandhe_frontend_pre_styled_ui::slider;
use fandhe_frontend_pre_styled_ui::spinner::{spinner, SpinnerProps};
use fandhe_frontend_pre_styled_ui::splitter;
use fandhe_frontend_pre_styled_ui::stat;
use fandhe_frontend_pre_styled_ui::status::{self, StatusProps};
use fandhe_frontend_pre_styled_ui::steps;
use fandhe_frontend_pre_styled_ui::strong::strong;
use fandhe_frontend_pre_styled_ui::tags_input;
use fandhe_frontend_pre_styled_ui::text::{text as styled_text, TextProps};
use fandhe_frontend_pre_styled_ui::textarea::{self, TextareaProps};
use fandhe_frontend_pre_styled_ui::timeline::{self, TimelineVariant};
use fandhe_frontend_pre_styled_ui::timer::{self, TimerControl, TimerPhase, TimerUnit};
use fandhe_frontend_pre_styled_ui::toast::{self, ToastPlacement, ToastStatus};
use fandhe_frontend_pre_styled_ui::tour::{self, ContentIds as TourContentIds, TourStep};
use fandhe_frontend_pre_styled_ui::{accordion, dialog, menu, select};
use fandhe_frontend_pre_styled_ui::{ColorPalette, OpenState, Size};

/// OWASP XSS Prevention Cheat Sheet Rule #1 系の共有ペイロード集合。
///
/// `crates/pre-styled-ui/tests/xss_escape.rs::payloads` と観点を揃えるが、
/// クレート境界・ファイル境界をまたいで共有しない既存方針に従い本ファイル
/// 内で独立に定義する。
mod payloads {
    /// タグ注入。
    pub const SCRIPT_TAG: &str = "<script>alert('xss')</script>";
    /// 二重引用符属性値からの breakout。
    pub const DOUBLE_QUOTE_BREAKOUT: &str = "\"><script>alert(1)</script>";
    /// 単一引用符属性値からの breakout（イベントハンドラ注入込み）。
    pub const SINGLE_QUOTE_BREAKOUT: &str = "' onmouseover='alert(1)";
    /// 非 ASCII 混在文字列（マルチバイト透過の確認）。
    pub const NON_ASCII_MIXED: &str = "こんにちは<script>alert(1)</script>世界";

    /// 全ペイロードをまとめて返す（網羅的にループ検証する用途）。
    pub fn all() -> Vec<&'static str> {
        vec![
            SCRIPT_TAG,
            DOUBLE_QUOTE_BREAKOUT,
            SINGLE_QUOTE_BREAKOUT,
            NON_ASCII_MIXED,
        ]
    }
}

/// テキスト・属性値経路の共通アサーション
/// （`crates/pre-styled-ui/tests/xss_escape.rs::assert_payload_is_escaped` と
/// 同型）。
fn assert_payload_is_escaped(payload: &str, html: &str, context_label: &str) {
    let expected_escaped = escape_html(payload);
    assert!(
        html.contains(&expected_escaped),
        "{context_label}で期待されるエスケープ済み表現が出力に見当たらない: \
         payload={payload:?}, expected_escaped={expected_escaped:?}, html={html}"
    );
    assert!(
        !html.contains(payload),
        "{context_label}で生ペイロードが出力にそのまま残っている: payload={payload:?}, html={html}"
    );
    assert!(
        !html.contains("<script>"),
        "{context_label}で実タグとしての <script> が出力に出現している: html={html}"
    );
}

/// (1) テキスト経路: `button`/`badge`/`alert::title`/`card::body` の
/// children へ全ペイロードを注入し、既定エスケープが styled 部品公開 API
/// 経由で貫通することを固定する。`dialog::title`（headless-ui からの
/// `pub use` 再エクスポート、`crates/pre-styled-ui/src/dialog.rs` 参照）も
/// 1 系統含め、再エクスポートが新たな迂回経路を持たないことを確認する。
#[test]
fn styled_text_children_are_escaped_for_all_payloads() {
    for payload in payloads::all() {
        let html = render(&button(
            &ButtonProps::default(),
            vec![],
            vec![text(payload)],
        ));
        assert_payload_is_escaped(payload, &html, "button children コンテキスト");

        let html = render(&badge(&BadgeProps::default(), vec![], vec![text(payload)]));
        assert_payload_is_escaped(payload, &html, "badge children コンテキスト");

        let html = render(&alert::title(vec![], vec![text(payload)]));
        assert_payload_is_escaped(payload, &html, "alert::title children コンテキスト");

        let html = render(&alert::action(vec![], vec![text(payload)]));
        assert_payload_is_escaped(payload, &html, "alert::action children コンテキスト");

        let html = render(&card::body(vec![], vec![text(payload)]));
        assert_payload_is_escaped(payload, &html, "card::body children コンテキスト");

        let html = render(&card::action(vec![], vec![text(payload)]));
        assert_payload_is_escaped(payload, &html, "card::action children コンテキスト");

        let html = render(&card::cover(vec![], vec![text(payload)]));
        assert_payload_is_escaped(payload, &html, "card::cover children コンテキスト");

        let html = render(&dialog::title(None, vec![], vec![text(payload)]));
        assert_payload_is_escaped(
            payload,
            &html,
            "pre-styled-ui 再エクスポート dialog::title children コンテキスト",
        );

        let html = render(&drawer::title(None, vec![], vec![text(payload)]));
        assert_payload_is_escaped(
            payload,
            &html,
            "pre-styled-ui 再エクスポート drawer::title children コンテキスト",
        );

        let html = render(&stat::label(vec![], vec![text(payload)]));
        assert_payload_is_escaped(payload, &html, "stat::label children コンテキスト");

        let html = render(&stat::value_text(vec![], vec![text(payload)]));
        assert_payload_is_escaped(payload, &html, "stat::value_text children コンテキスト");

        let html = render(&timeline::title(vec![], vec![text(payload)]));
        assert_payload_is_escaped(payload, &html, "timeline::title children コンテキスト");

        let html = render(&timeline::description(vec![], vec![text(payload)]));
        assert_payload_is_escaped(
            payload,
            &html,
            "timeline::description children コンテキスト",
        );

        let html = render(&marquee::marquee(
            &MarqueeProps::default(),
            vec![],
            vec![text(payload)],
        ));
        assert_payload_is_escaped(payload, &html, "marquee::marquee children コンテキスト");

        let html = render(&marquee::item(vec![], vec![text(payload)]));
        assert_payload_is_escaped(payload, &html, "marquee::item children コンテキスト");
    }
}

/// (2) 属性値経路 a: `spinner::SpinnerProps::label` は部品内部で
/// `aria-label` へ透過する（`crates/pre-styled-ui/src/spinner.rs` rustdoc の
/// 契約「`\"` や `<` を含む値を渡しても構造は壊れない」）。styled 部品公開
/// API 経由でこの契約が貫通することを固定する。
#[test]
fn spinner_label_attribute_is_escaped_for_all_payloads() {
    for payload in payloads::all() {
        let node = spinner(&SpinnerProps {
            size: fandhe_frontend_pre_styled_ui::Size::Md,
            palette: ColorPalette::Accent,
            label: payload,
        });
        let html = render(&node);
        assert_payload_is_escaped(payload, &html, "spinner label 属性値コンテキスト");
    }
}

/// (2b) 属性値経路: `icon_button`/`close_button`（イシュー #830）の必須
/// `label` 引数が `aria-label` へ透過する経路で、既定エスケープが貫通する
/// ことを固定する（REQ-1 回帰、button.rs 冒頭 rustdoc の「aria-label 経由も
/// `render` の既定エスケープを通る」契約の裏付け）。
#[test]
fn icon_button_and_close_button_label_attribute_is_escaped_for_all_payloads() {
    for payload in payloads::all() {
        let html = render(&icon_button(
            &ButtonProps::default(),
            payload,
            vec![],
            vec![],
        ));
        assert_payload_is_escaped(payload, &html, "icon_button label 属性値コンテキスト");

        let html = render(&close_button(&ButtonProps::default(), payload, vec![]));
        assert_payload_is_escaped(payload, &html, "close_button label 属性値コンテキスト");
    }
}

/// (2) 属性値経路 a（続き、イシュー #831）: `marquee::MarqueeProps::label` は
/// `decorative: false`（既定）時に `aria-label` へ透過する。
#[test]
fn marquee_label_attribute_is_escaped_for_all_payloads() {
    for payload in payloads::all() {
        let props = MarqueeProps {
            label: Some(payload),
            ..MarqueeProps::default()
        };
        let html = render(&marquee::marquee(&props, vec![], vec![]));
        assert_payload_is_escaped(payload, &html, "marquee label 属性値コンテキスト");
    }
}

/// (3) 属性値経路 b: 呼び出し側 `attrs`（`data-testid`/`aria-label`）へ
/// 全ペイロードを注入し、styled 部品が attrs をそのまま `render` へ渡す
/// 経路でもエスケープが貫通することを固定する。
#[test]
fn caller_attrs_are_escaped_for_all_payloads() {
    for payload in payloads::all() {
        let html = render(&button(
            &ButtonProps::default(),
            vec![("data-testid", payload)],
            vec![],
        ));
        assert_payload_is_escaped(payload, &html, "button 呼び出し側 attrs コンテキスト");

        let html = render(&card::root(
            CardProps::default(),
            vec![("aria-label", payload)],
            vec![],
        ));
        assert_payload_is_escaped(payload, &html, "card::root 呼び出し側 attrs コンテキスト");

        let html = render(&card::action(vec![("data-testid", payload)], vec![]));
        assert_payload_is_escaped(payload, &html, "card::action 呼び出し側 attrs コンテキスト");

        let html = render(&card::cover(vec![("data-testid", payload)], vec![]));
        assert_payload_is_escaped(payload, &html, "card::cover 呼び出し側 attrs コンテキスト");

        let html = render(&alert::root(
            &AlertProps::default(),
            vec![("data-testid", payload)],
            vec![],
        ));
        assert_payload_is_escaped(payload, &html, "alert::root 呼び出し側 attrs コンテキスト");

        let html = render(&alert::action(vec![("data-testid", payload)], vec![]));
        assert_payload_is_escaped(
            payload,
            &html,
            "alert::action 呼び出し側 attrs コンテキスト",
        );

        let html = render(&stat::root(
            Size::Md,
            vec![("data-testid", payload)],
            vec![],
        ));
        assert_payload_is_escaped(payload, &html, "stat::root 呼び出し側 attrs コンテキスト");

        let html = render(&timeline::root(
            TimelineVariant::default(),
            Size::Md,
            ColorPalette::default(),
            vec![("data-testid", payload)],
            vec![],
        ));
        assert_payload_is_escaped(
            payload,
            &html,
            "timeline::root 呼び出し側 attrs コンテキスト",
        );

        let html = render(&marquee::marquee(
            &MarqueeProps::default(),
            vec![("data-testid", payload)],
            vec![],
        ));
        assert_payload_is_escaped(
            payload,
            &html,
            "marquee::marquee 呼び出し側 attrs コンテキスト",
        );
    }
}

/// (4) 属性値経路 c: 呼び出し側 `attrs` に `class` を渡した場合、
/// `class_attr::drop_class_attr` 契約（`crates/pre-styled-ui/src/class_attr.rs`）
/// により呼び出し側の値は完全に破棄され、recipe が生成する単一クラスに
/// 置き換わることを固定する（動的クラス名合成による注入がないこと）。
#[test]
fn caller_class_attr_is_dropped_not_merged_raw_for_all_payloads() {
    for payload in payloads::all() {
        let html = render(&button(
            &ButtonProps::default(),
            vec![("class", payload)],
            vec![],
        ));
        assert!(
            !html.contains(payload),
            "class 属性に渡した生ペイロードが出力に残っている: payload={payload:?}, html={html}"
        );
        assert_eq!(
            html.matches("class=\"").count(),
            1,
            "class 属性が複数出現している（合成ではなく単一置換であるべき）: html={html}"
        );
        assert!(
            html.contains("fd-button--"),
            "recipe 生成クラスが失われている: html={html}"
        );

        let html = render(&stat::root(Size::Md, vec![("class", payload)], vec![]));
        assert!(
            !html.contains(payload),
            "stat::root の class 属性に渡した生ペイロードが出力に残っている: payload={payload:?}, html={html}"
        );
        assert_eq!(
            html.matches("class=\"").count(),
            1,
            "stat::root の class 属性が複数出現している: html={html}"
        );
        assert!(
            html.contains("fd-stat--"),
            "stat::root の recipe 生成クラスが失われている: html={html}"
        );

        let html = render(&timeline::root(
            TimelineVariant::default(),
            Size::Md,
            ColorPalette::default(),
            vec![("class", payload)],
            vec![],
        ));
        assert!(
            !html.contains(payload),
            "timeline::root の class 属性に渡した生ペイロードが出力に残っている: payload={payload:?}, html={html}"
        );
        assert_eq!(
            html.matches("class=\"").count(),
            1,
            "timeline::root の class 属性が複数出現している: html={html}"
        );
        assert!(
            html.contains("fd-timeline--"),
            "timeline::root の recipe 生成クラスが失われている: html={html}"
        );

        let html = render(&marquee::marquee(
            &MarqueeProps::default(),
            vec![("class", payload)],
            vec![],
        ));
        assert!(
            !html.contains(payload),
            "marquee::marquee の class 属性に渡した生ペイロードが出力に残っている: payload={payload:?}, html={html}"
        );
        assert_eq!(
            html.matches("class=\"").count(),
            1,
            "marquee::marquee の class 属性が複数出現している: html={html}"
        );
        assert!(
            html.contains("fd-marquee--"),
            "marquee::marquee の recipe 生成クラスが失われている: html={html}"
        );
    }
}

/// (5) URL 属性経路（拒否）: 呼び出し側 `attrs` の `href`/`src` に危険な
/// URL スキームを渡し、core の許可リスト方式（deny by default）が styled
/// 部品の attrs 透過経路を通しても貫通することを固定する（拒否時に属性
/// ごとスキップされる契約は `crates/core/src/lib.rs::render_into` 参照）。
#[test]
fn dangerous_url_schemes_in_caller_attrs_are_rejected() {
    let dangerous_urls = [
        "javascript:alert(1)",
        "JaVaScRiPt:alert(1)",
        "data:text/html;base64,PHNjcmlwdD4=",
        "vbscript:msgbox(1)",
    ];

    for url in dangerous_urls {
        let html = render(&card::root(
            CardProps::default(),
            vec![("href", url), ("data-testid", "safe-sibling")],
            vec![],
        ));
        assert!(
            !html.contains("href="),
            "危険な URL スキームなのに href 属性が出力されている: url={url:?}, html={html}"
        );
        assert!(
            html.contains(r#"data-testid="safe-sibling""#),
            "href 属性の拒否によって兄弟属性まで欠落している: html={html}"
        );
        assert!(
            html.contains("fd-card--"),
            "href 属性の拒否によって recipe 生成クラスまで欠落している: html={html}"
        );

        let html = render(&badge(&BadgeProps::default(), vec![("src", url)], vec![]));
        assert!(
            !html.contains("src="),
            "危険な URL スキームなのに src 属性が出力されている: url={url:?}, html={html}"
        );
        assert!(
            html.contains("fd-badge--"),
            "src 属性の拒否によって recipe 生成クラスまで欠落している: html={html}"
        );
    }
}

/// (5) URL 属性経路（透過）: 安全な URL は `href="..."` として透過することを
/// 固定する（陽性・陰性の両建て、vacuous pass 防止）。
#[test]
fn safe_urls_in_caller_attrs_pass_through() {
    for url in ["/items/1", "https://example.com/a"] {
        let html = render(&card::root(
            CardProps::default(),
            vec![("href", url)],
            vec![],
        ));
        let expected = format!(r#"href="{}""#, escape_html(url));
        assert!(
            html.contains(&expected),
            "安全な URL が href 属性として透過していない: url={url:?}, html={html}"
        );
    }
}

/// (6) 属性値経路 d（イシュー #729）: `accordion`/`dialog`/`menu`/`select` の
/// 新設 styled `root`（`size` variant クラス付与）でも `class_attr::drop_class_attr`
/// 契約により呼び出し側 `class` の生ペイロードが動的クラス名合成へ混入しない
/// ことを固定する（`switch`/`avatar` 等の既存 styled root と同型の回帰、
/// #708/#719 の一般化）。
#[test]
fn size_variant_root_caller_class_attr_is_dropped_not_merged_raw_for_all_payloads() {
    for payload in payloads::all() {
        let html = render(&accordion::root(
            Size::Md,
            &accordion::AccordionProps::default(),
            vec![("class", payload)],
            vec![],
        ));
        assert!(
            !html.contains(payload),
            "accordion::root の class 属性に渡した生ペイロードが出力に残っている: payload={payload:?}, html={html}"
        );
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(html.contains("fd-accordion--"));

        let html = render(&dialog::root(
            Size::Md,
            OpenState::Closed,
            vec![("class", payload)],
            vec![],
        ));
        assert!(
            !html.contains(payload),
            "dialog::root の class 属性に渡した生ペイロードが出力に残っている: payload={payload:?}, html={html}"
        );
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(html.contains("fd-dialog--"));

        let html = render(&menu::root(
            Size::Md,
            OpenState::Closed,
            vec![("class", payload)],
            vec![],
        ));
        assert!(
            !html.contains(payload),
            "menu::root の class 属性に渡した生ペイロードが出力に残っている: payload={payload:?}, html={html}"
        );
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(html.contains("fd-menu--"));

        let html = render(&select::root(
            Size::Md,
            OpenState::Closed,
            &select::SelectProps::default(),
            vec![("class", payload)],
            vec![],
        ));
        assert!(
            !html.contains(payload),
            "select::root の class 属性に渡した生ペイロードが出力に残っている: payload={payload:?}, html={html}"
        );
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(html.contains("fd-select--"));

        let html = render(&drawer::root(
            Size::Md,
            OpenState::Closed,
            DrawerPlacement::End,
            vec![("class", payload)],
            vec![],
        ));
        assert!(
            !html.contains(payload),
            "drawer::root の class 属性に渡した生ペイロードが出力に残っている: payload={payload:?}, html={html}"
        );
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(html.contains("fd-drawer--"));
    }
}

/// (7) 属性値経路 e（イシュー #729）: 呼び出し側 `attrs` の `data-testid` 等が
/// `size` variant root 経由でも既定エスケープを経由することを固定する。
#[test]
fn size_variant_root_caller_attrs_are_escaped_for_all_payloads() {
    for payload in payloads::all() {
        let html = render(&accordion::root(
            Size::Md,
            &accordion::AccordionProps::default(),
            vec![("data-testid", payload)],
            vec![],
        ));
        assert_payload_is_escaped(
            payload,
            &html,
            "accordion::root 呼び出し側 attrs コンテキスト",
        );

        let html = render(&dialog::root(
            Size::Md,
            OpenState::Closed,
            vec![("data-testid", payload)],
            vec![],
        ));
        assert_payload_is_escaped(payload, &html, "dialog::root 呼び出し側 attrs コンテキスト");

        let html = render(&menu::root(
            Size::Md,
            OpenState::Closed,
            vec![("data-testid", payload)],
            vec![],
        ));
        assert_payload_is_escaped(payload, &html, "menu::root 呼び出し側 attrs コンテキスト");

        let html = render(&select::root(
            Size::Md,
            OpenState::Closed,
            &select::SelectProps::default(),
            vec![("data-testid", payload)],
            vec![],
        ));
        assert_payload_is_escaped(payload, &html, "select::root 呼び出し側 attrs コンテキスト");

        // イシュー #2186: 新設 3 パーツ（separator/scroll-up-button/
        // scroll-down-button）の attrs/children 経路。
        let html = render(&select::separator(
            vec![("data-testid", payload)],
            vec![text(payload)],
        ));
        assert_payload_is_escaped(
            payload,
            &html,
            "select::separator attrs/children コンテキスト",
        );

        let html = render(&select::scroll_up_button(
            vec![("data-testid", payload)],
            vec![text(payload)],
        ));
        assert_payload_is_escaped(
            payload,
            &html,
            "select::scroll_up_button attrs/children コンテキスト",
        );

        let html = render(&select::scroll_down_button(
            vec![("data-testid", payload)],
            vec![text(payload)],
        ));
        assert_payload_is_escaped(
            payload,
            &html,
            "select::scroll_down_button attrs/children コンテキスト",
        );

        let html = render(&drawer::root(
            Size::Md,
            OpenState::Closed,
            DrawerPlacement::End,
            vec![("data-testid", payload)],
            vec![],
        ));
        assert_payload_is_escaped(payload, &html, "drawer::root 呼び出し側 attrs コンテキスト");
    }
}

/// (8) checkbox 経路（イシュー #730）: styled `root` の呼び出し側 `attrs`・
/// `class`、および headless-ui から選択的再エクスポートした `label` の
/// children・`hidden_input` の `name`/`value` の 4 箇所すべてで既定エスケープ
/// （REQ-1）が貫通することを固定する。`root`/`label`/`hidden_input` は
/// `crates/pre-styled-ui/src/checkbox.rs` の同型 inline テストの単一ペイロード
/// 版を、本ファイルの共有ペイロード集合（`payloads::all()`）へ拡張する。
#[test]
fn checkbox_styled_root_and_reexported_parts_are_escaped_for_all_payloads() {
    for payload in payloads::all() {
        // styled root の呼び出し側 attrs 経路。
        let html = render(&checkbox::root(
            Size::Md,
            ColorPalette::Accent,
            &CheckboxProps::default(),
            vec![("data-testid", payload)],
            vec![],
        ));
        assert_payload_is_escaped(
            payload,
            &html,
            "checkbox::root 呼び出し側 attrs コンテキスト",
        );

        // styled root の class 属性経路（drop_class_attr により生ペイロードは
        // 出力されず、recipe 生成クラスへ完全に置き換わる）。
        let html = render(&checkbox::root(
            Size::Md,
            ColorPalette::Accent,
            &CheckboxProps::default(),
            vec![("class", payload)],
            vec![],
        ));
        assert!(
            !html.contains(payload),
            "checkbox::root の class 属性に渡した生ペイロードが出力に残っている: \
             payload={payload:?}, html={html}"
        );
        assert_eq!(
            html.matches("class=\"").count(),
            1,
            "checkbox::root の class 属性が複数出現している: html={html}"
        );
        assert!(
            html.contains("fd-checkbox--"),
            "checkbox::root で recipe 生成クラスが失われている: html={html}"
        );

        // 選択的再エクスポートした label の children 経路。
        let html = render(&checkbox::label(
            &CheckboxProps::default(),
            vec![],
            vec![text(payload)],
        ));
        assert_payload_is_escaped(payload, &html, "checkbox::label children コンテキスト");

        // 選択的再エクスポートした hidden_input の name/value 経路。
        let html = render(&checkbox::hidden_input(
            &CheckboxProps::default(),
            payload,
            payload,
            vec![],
        ));
        assert_payload_is_escaped(
            payload,
            &html,
            "checkbox::hidden_input name/value コンテキスト",
        );
    }
}

/// (7) input/textarea/native_select 経路（イシュー #737）: 状態機械を持たない
/// 静的フォーム部品 3 種の `extra_attrs`（`value`/`placeholder` 等）・
/// `children`（textarea のテキスト・native_select の option）・呼び出し側
/// `class` の 3 経路で既定エスケープ（REQ-1）が貫通することを固定する。
/// アクセシビリティ配線は headless `field::*` へ委譲するのみだが（本ファイル
/// 冒頭の対象範囲外）、styled 公開 API 経由での既定エスケープはここで固定する。
#[test]
fn form_controls_extra_attrs_and_children_are_escaped_for_all_payloads() {
    fn field(id: &str) -> FieldProps<'_> {
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

    for payload in payloads::all() {
        // input の extra_attrs（value）経路。
        let f = field("f");
        let html = render(&input::input(
            &InputProps::default(),
            &f,
            vec![("value", payload)],
        ));
        assert_payload_is_escaped(payload, &html, "input extra_attrs value コンテキスト");

        // textarea の children（テキスト）経路。
        let f = field("f");
        let html = render(&textarea::textarea(
            &TextareaProps::default(),
            &f,
            false,
            vec![],
            vec![text(payload)],
        ));
        assert_payload_is_escaped(payload, &html, "textarea children コンテキスト");

        // native_select の option children（テキスト）経路。
        let f = field("f");
        let option = el("option", vec![("value", "x")], vec![text(payload)]);
        let html = render(&native_select::native_select(
            &NativeSelectProps::default(),
            &f,
            vec![],
            vec![option],
        ));
        assert_payload_is_escaped(payload, &html, "native_select option children コンテキスト");

        // 3 部品共通の class 属性経路（drop_class_attr により生ペイロードは
        // 出力されず、recipe 生成クラスへ完全に置き換わる）。
        let f = field("f");
        let html = render(&input::input(
            &InputProps::default(),
            &f,
            vec![("class", payload)],
        ));
        assert!(
            !html.contains(payload),
            "input の class 属性に渡した生ペイロードが出力に残っている: \
             payload={payload:?}, html={html}"
        );
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(html.contains("fd-field--"));
    }
}

/// (7b) styled Field `root` 経路（イシュー #1684）: 呼び出し側 `attrs`・
/// `class`、選択的再エクスポートした `label`/`helper_text`/`error_text`/
/// `required_indicator` の children、`FieldProps::id` から派生する
/// `id`/`for`/`aria-describedby` 属性値のいずれの経路でも既定エスケープ
/// （REQ-1）が貫通することを固定する（(7) の `input`/`textarea`/
/// `native_select` と同粒度）。
#[test]
fn field_root_and_reexported_parts_are_escaped_for_all_payloads() {
    fn field(id: &str) -> fandhe_frontend_pre_styled_ui::field::FieldProps<'_> {
        fandhe_frontend_pre_styled_ui::field::FieldProps {
            id,
            ids: fandhe_frontend_pre_styled_ui::field::FieldIds::default(),
            disabled: false,
            invalid: false,
            required: false,
            readonly: false,
            has_helper_text: false,
        }
    }

    for payload in payloads::all() {
        // styled root の呼び出し側 attrs 経路。
        let f = field("f");
        let html = render(&field::root(
            &FieldRootProps::default(),
            &f,
            vec![("data-testid", payload)],
            vec![],
        ));
        assert_payload_is_escaped(payload, &html, "field::root attrs コンテキスト");

        // styled root の呼び出し側 class 属性経路（drop_class_attr により
        // 生ペイロードは出力されず、recipe 生成クラスへ完全に置き換わる）。
        let f = field("f");
        let html = render(&field::root(
            &FieldRootProps::default(),
            &f,
            vec![("class", payload)],
            vec![],
        ));
        assert!(
            !html.contains(payload),
            "field::root の class 属性に渡した生ペイロードが出力に残っている: \
             payload={payload:?}, html={html}"
        );
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(html.contains("fd-field--orientation-"));

        // 選択的再エクスポート（label/helper_text/error_text/
        // required_indicator）の children 経路。
        let f = field("f");
        let html = render(&field::label(&f, vec![], vec![text(payload)]));
        assert_payload_is_escaped(payload, &html, "field::label children コンテキスト");

        let f = field("f");
        let html = render(&field::helper_text(&f, vec![], vec![text(payload)]));
        assert_payload_is_escaped(payload, &html, "field::helper_text children コンテキスト");

        let mut f = field("f");
        f.invalid = true;
        let html = render(&field::error_text(&f, vec![], vec![text(payload)]));
        assert_payload_is_escaped(payload, &html, "field::error_text children コンテキスト");

        let mut f = field("f");
        f.required = true;
        let html = render(&field::required_indicator(&f, vec![], vec![text(payload)]));
        assert_payload_is_escaped(
            payload,
            &html,
            "field::required_indicator children コンテキスト",
        );

        // `FieldProps::id` から派生する `id`/`for`/`aria-describedby`
        // 属性値経路（`label` の `for`/`id`、`helper_text` を併用した
        // `error_text`/コントロール側の `aria-describedby`）。
        let mut f = field(payload);
        f.has_helper_text = true;
        f.invalid = true;
        let html = render(&field::label(&f, vec![], vec![]));
        assert_payload_is_escaped(payload, &html, "field id 由来 label for/id コンテキスト");
        let html = render(&field::helper_text(&f, vec![], vec![]));
        assert_payload_is_escaped(payload, &html, "field id 由来 helper_text id コンテキスト");
        let html = render(&field::error_text(&f, vec![], vec![]));
        assert_payload_is_escaped(payload, &html, "field id 由来 error_text id コンテキスト");
    }
}

/// (7b-2) styled Field 拡張パーツ（`group`/`content`/`title`/`separator`、
/// イシュー #2185）経路: 呼び出し側 `attrs`・children、`title` の `id`
/// attrs 経路のいずれでも既定エスケープ（REQ-1）が貫通することを固定する
/// （(7b) と同粒度）。
#[test]
fn field_extended_2185_parts_are_escaped_for_all_payloads() {
    fn field(id: &str) -> fandhe_frontend_pre_styled_ui::field::FieldProps<'_> {
        fandhe_frontend_pre_styled_ui::field::FieldProps {
            id,
            ids: fandhe_frontend_pre_styled_ui::field::FieldIds::default(),
            disabled: false,
            invalid: false,
            required: false,
            readonly: false,
            has_helper_text: false,
        }
    }

    for payload in payloads::all() {
        // group: attrs コンテキスト・children コンテキスト。
        let html = render(&field::group(
            vec![("data-testid", payload)],
            vec![text(payload)],
        ));
        assert_payload_is_escaped(payload, &html, "field::group attrs/children コンテキスト");

        // content/title: attrs コンテキスト・children コンテキスト。
        let f = field("f");
        let html = render(&field::content(
            &f,
            vec![("data-testid", payload)],
            vec![text(payload)],
        ));
        assert_payload_is_escaped(payload, &html, "field::content attrs/children コンテキスト");

        // title は id 自動導出を持たないため、呼び出し側が渡す id attrs
        // 経路（aria-labelledby 運用の前提、field.rs モジュール doc 参照）
        // が既定エスケープを通ることを固定する。
        let html = render(&field::title(
            &f,
            vec![("id", payload)],
            vec![text(payload)],
        ));
        assert_payload_is_escaped(
            payload,
            &html,
            "field::title id attrs/children コンテキスト",
        );

        // separator: ラッパー attrs コンテキスト・テキスト children
        // コンテキスト（separator-content 経由）。
        let html = render(&field::separator(
            vec![("data-testid", payload)],
            vec![text(payload)],
        ));
        assert_payload_is_escaped(
            payload,
            &html,
            "field::separator attrs/content コンテキスト",
        );
    }
}

/// (7c) styled Fieldset `root` 経路（イシュー #1686）: 呼び出し側 `attrs`・
/// `class`、選択的再エクスポートした `legend`/`helper_text`/`error_text` の
/// children、`FieldsetProps::id` から派生する `id`/`aria-describedby`
/// 属性値のいずれの経路でも既定エスケープ（REQ-1）が貫通することを固定
/// する（(7b) `field` と同粒度）。
#[test]
fn fieldset_root_and_reexported_parts_are_escaped_for_all_payloads() {
    fn fieldset_props(id: &str) -> fandhe_frontend_pre_styled_ui::fieldset::FieldsetProps<'_> {
        fandhe_frontend_pre_styled_ui::fieldset::FieldsetProps {
            id,
            disabled: false,
            invalid: false,
            has_helper_text: false,
        }
    }

    for payload in payloads::all() {
        // styled root の呼び出し側 attrs 経路。
        let f = fieldset_props("f");
        let html = render(&fieldset::root(
            &FieldsetRootProps::default(),
            &f,
            vec![("data-testid", payload)],
            vec![],
        ));
        assert_payload_is_escaped(payload, &html, "fieldset::root attrs コンテキスト");

        // styled root の呼び出し側 class 属性経路（drop_class_attr により
        // 生ペイロードは出力されず、recipe 生成クラスへ完全に置き換わる）。
        let f = fieldset_props("f");
        let html = render(&fieldset::root(
            &FieldsetRootProps::default(),
            &f,
            vec![("class", payload)],
            vec![],
        ));
        assert!(
            !html.contains(payload),
            "fieldset::root の class 属性に渡した生ペイロードが出力に残っている: \
             payload={payload:?}, html={html}"
        );
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(html.contains("fd-fieldset--size-"));

        // 選択的再エクスポート（legend/helper_text/error_text）の children
        // 経路。
        let f = fieldset_props("f");
        let html = render(&fieldset::legend(&f, vec![], vec![text(payload)]));
        assert_payload_is_escaped(payload, &html, "fieldset::legend children コンテキスト");

        let f = fieldset_props("f");
        let html = render(&fieldset::helper_text(&f, vec![], vec![text(payload)]));
        assert_payload_is_escaped(
            payload,
            &html,
            "fieldset::helper_text children コンテキスト",
        );

        let mut f = fieldset_props("f");
        f.invalid = true;
        let html = render(&fieldset::error_text(&f, vec![], vec![text(payload)]));
        assert_payload_is_escaped(payload, &html, "fieldset::error_text children コンテキスト");

        // `FieldsetProps::id` から派生する `id`/`aria-describedby` 属性値
        // 経路（`legend`/`helper_text`/`error_text` の id、invalid +
        // has_helper_text の aria-describedby 合成）。
        let mut f = fieldset_props(payload);
        f.has_helper_text = true;
        f.invalid = true;
        let html = render(&fieldset::legend(&f, vec![], vec![]));
        assert_payload_is_escaped(payload, &html, "fieldset id 由来 legend id コンテキスト");
        let html = render(&fieldset::helper_text(&f, vec![], vec![]));
        assert_payload_is_escaped(
            payload,
            &html,
            "fieldset id 由来 helper_text id コンテキスト",
        );
        let html = render(&fieldset::error_text(&f, vec![], vec![]));
        assert_payload_is_escaped(
            payload,
            &html,
            "fieldset id 由来 error_text id コンテキスト",
        );
        let html = render(&fieldset::root(
            &FieldsetRootProps::default(),
            &f,
            vec![],
            vec![],
        ));
        assert_payload_is_escaped(
            payload,
            &html,
            "fieldset id 由来 root aria-describedby コンテキスト",
        );
    }
}

/// (7d) styled Input Group `root`/`addon`/`text`/`button` 経路（イシュー
/// #2063）: 4 パーツいずれも見た目クラスを付与しない（`src/input_group.rs`
/// モジュール doc「variant 軸: 持たない」節参照）ため、呼び出し側 `attrs`・
/// `class`（[`drop_class_attr`] により除去、recipe クラスを持たないため
/// `class` 属性自体が出力から消える）、children の各経路で既定エスケープ
/// （REQ-1）が貫通することを固定する。
#[test]
fn input_group_parts_are_escaped_for_all_payloads() {
    use fandhe_frontend_pre_styled_ui::input_group::{self, InputGroupAlign, InputGroupProps};

    fn enabled_props() -> InputGroupProps {
        InputGroupProps {
            disabled: false,
            invalid: false,
        }
    }

    for payload in payloads::all() {
        let props = enabled_props();

        // styled root の呼び出し側 attrs 経路。
        let html = render(&input_group::root(
            &props,
            vec![("data-testid", payload)],
            vec![],
        ));
        assert_payload_is_escaped(payload, &html, "input_group::root attrs コンテキスト");

        // styled root の呼び出し側 class 属性経路（見た目クラスを持たない
        // ため drop_class_attr により class 属性自体が出力から消える）。
        let html = render(&input_group::root(&props, vec![("class", payload)], vec![]));
        assert!(
            !html.contains(payload),
            "input_group::root の class 属性に渡した生ペイロードが出力に残っている: \
             payload={payload:?}, html={html}"
        );
        assert_eq!(html.matches("class=\"").count(), 0);

        // styled addon の呼び出し側 attrs 経路。
        let html = render(&input_group::addon(
            InputGroupAlign::InlineStart,
            &props,
            vec![("data-testid", payload)],
            vec![],
        ));
        assert_payload_is_escaped(payload, &html, "input_group::addon attrs コンテキスト");

        // styled text の children 経路。
        let html = render(&input_group::text(vec![], vec![text(payload)]));
        assert_payload_is_escaped(payload, &html, "input_group::text children コンテキスト");

        // styled button の呼び出し側 attrs・children 経路。
        let html = render(&input_group::button(
            &props,
            vec![("data-testid", payload)],
            vec![text(payload)],
        ));
        assert_payload_is_escaped(payload, &html, "input_group::button attrs コンテキスト");
        assert_payload_is_escaped(payload, &html, "input_group::button children コンテキスト");
    }
}

/// Item 経路（イシュー #2066）: 10 パーツいずれも見た目クラスを付与しない
/// （`src/item.rs` モジュール doc「variant / size の表現」節参照）ため、
/// 呼び出し側 `attrs`・`class`（[`drop_class_attr`] により除去）、
/// `root` の `href`（危険スキームは headless 層の deny-by-default で属性
/// ごと欠落）、`group` の `label`（`aria-label` エスケープ）、children の
/// 各経路で既定エスケープ（REQ-1）が貫通することを固定する。
#[test]
fn item_parts_are_escaped_for_all_payloads() {
    use fandhe_frontend_pre_styled_ui::item::{self, ItemMediaVariant, ItemRootProps};

    for payload in payloads::all() {
        // styled root（div）の呼び出し側 attrs 経路。
        let html = render(&item::root(
            ItemRootProps::default(),
            vec![("data-testid", payload)],
            vec![],
        ));
        assert_payload_is_escaped(payload, &html, "item::root attrs コンテキスト");

        // styled root の呼び出し側 class 属性経路（見た目クラスを持たない
        // ため drop_class_attr により class 属性自体が出力から消える）。
        let html = render(&item::root(
            ItemRootProps::default(),
            vec![("class", payload)],
            vec![],
        ));
        assert!(
            !html.contains(payload),
            "item::root の class 属性に渡した生ペイロードが出力に残っている: \
             payload={payload:?}, html={html}"
        );
        assert_eq!(html.matches("class=\"").count(), 0);

        // styled root（a）の href 経路。危険スキームではない通常ペイロード
        // が値としてそのままエスケープ済みで出力されることを確認する
        // （危険スキームの拒否は headless 層 `crates/headless-ui/src/item.rs`
        // のテストで固定済み、本テストは責務の重複を避ける）。
        let href = format!("/docs/{payload}");
        let html = render(&item::root(
            ItemRootProps {
                href: Some(&href),
                external: true,
                ..Default::default()
            },
            vec![],
            vec![],
        ));
        assert_payload_is_escaped(payload, &html, "item::root href コンテキスト");
        // external: true は target/rel を不可分に付与する（headless 契約の
        // 透過確認）。
        assert!(html.contains(r#"target="_blank""#));
        assert!(html.contains(r#"rel="noopener noreferrer""#));

        // styled media の呼び出し側 attrs 経路。
        let html = render(&item::media(
            ItemMediaVariant::Icon,
            vec![("data-testid", payload)],
            vec![],
        ));
        assert_payload_is_escaped(payload, &html, "item::media attrs コンテキスト");

        // styled content/title/description/actions/header/footer の
        // 呼び出し側 attrs・children 経路。
        let html = render(&item::content(
            vec![("data-testid", payload)],
            vec![
                item::title(vec![], vec![text(payload)]),
                item::description(vec![], vec![text(payload)]),
            ],
        ));
        assert_payload_is_escaped(payload, &html, "item::content attrs コンテキスト");
        assert_payload_is_escaped(payload, &html, "item::title children コンテキスト");
        assert_payload_is_escaped(payload, &html, "item::description children コンテキスト");

        let html = render(&item::actions(
            vec![("data-testid", payload)],
            vec![text(payload)],
        ));
        assert_payload_is_escaped(payload, &html, "item::actions attrs コンテキスト");

        let html = render(&item::header(vec![("data-testid", payload)], vec![]));
        assert_payload_is_escaped(payload, &html, "item::header attrs コンテキスト");

        let html = render(&item::footer(vec![("data-testid", payload)], vec![]));
        assert_payload_is_escaped(payload, &html, "item::footer attrs コンテキスト");

        // styled group の label（aria-label）経路。
        let html = render(&item::group(payload, vec![], vec![]));
        assert_payload_is_escaped(payload, &html, "item::group label コンテキスト");

        // styled separator の呼び出し側 attrs 経路。
        let html = render(&item::separator(vec![("data-testid", payload)], vec![]));
        assert_payload_is_escaped(payload, &html, "item::separator attrs コンテキスト");
    }
}

/// Message 経路（イシュー #2106）: 6 パーツいずれも見た目クラスを付与しない
/// （`src/message.rs` モジュール doc「role / align / loading / error の
/// 表現」節参照）ため、呼び出し側 `attrs`・`class`（[`drop_class_attr`]
/// により除去）、`group` の `label`（`aria-label` エスケープ）、children
/// の各経路で既定エスケープ（REQ-1）が貫通することを固定する。
#[test]
fn message_parts_are_escaped_for_all_payloads() {
    use fandhe_frontend_pre_styled_ui::message::{self, MessageRootProps};

    for payload in payloads::all() {
        // styled root の呼び出し側 attrs 経路。
        let html = render(&message::root(
            MessageRootProps::default(),
            vec![("data-testid", payload)],
            vec![],
        ));
        assert_payload_is_escaped(payload, &html, "message::root attrs コンテキスト");

        // styled root の呼び出し側 class 属性経路（見た目クラスを持たない
        // ため drop_class_attr により class 属性自体が出力から消える）。
        let html = render(&message::root(
            MessageRootProps::default(),
            vec![("class", payload)],
            vec![],
        ));
        assert!(
            !html.contains(payload),
            "message::root の class 属性に渡した生ペイロードが出力に残って\
             いる: payload={payload:?}, html={html}"
        );
        assert_eq!(html.matches("class=\"").count(), 0);

        // styled avatar/header/content/footer の呼び出し側 attrs・children
        // 経路。
        let html = render(&message::avatar(
            vec![("data-testid", payload)],
            vec![text(payload)],
        ));
        assert_payload_is_escaped(payload, &html, "message::avatar attrs コンテキスト");
        assert_payload_is_escaped(payload, &html, "message::avatar children コンテキスト");

        let html = render(&message::header(
            vec![("data-testid", payload)],
            vec![text(payload)],
        ));
        assert_payload_is_escaped(payload, &html, "message::header attrs コンテキスト");
        assert_payload_is_escaped(payload, &html, "message::header children コンテキスト");

        let html = render(&message::content(
            vec![("data-testid", payload)],
            vec![text(payload)],
        ));
        assert_payload_is_escaped(payload, &html, "message::content attrs コンテキスト");
        assert_payload_is_escaped(payload, &html, "message::content children コンテキスト");

        let html = render(&message::footer(
            vec![("data-testid", payload)],
            vec![text(payload)],
        ));
        assert_payload_is_escaped(payload, &html, "message::footer attrs コンテキスト");
        assert_payload_is_escaped(payload, &html, "message::footer children コンテキスト");

        // styled group の label（aria-label）経路。
        let html = render(&message::group(payload, vec![], vec![]));
        assert_payload_is_escaped(payload, &html, "message::group label コンテキスト");
    }
}

/// Bubble 経路（イシュー #2109、headless 側 anatomy は #2108）: 6 パーツ
/// いずれも見た目クラスを付与しない（`src/bubble.rs` モジュール doc
/// 「headless の `data-*` を参照する」節参照）ため、呼び出し側
/// `attrs`・`class`（[`drop_class_attr`] により除去）、`reactions` の
/// `label`（`aria-label` エスケープ）、`collapse_trigger` の `controls`
/// （`aria-controls` エスケープ）、`collapse_content` の `id` エスケープ、
/// children の各経路で既定エスケープ（REQ-1）が貫通することを固定する
/// （`message_parts_are_escaped_for_all_payloads` と同型）。
#[test]
fn bubble_parts_are_escaped_for_all_payloads() {
    use fandhe_frontend_pre_styled_ui::bubble::{self, BubbleRootProps, OpenState};

    for payload in payloads::all() {
        // styled root の呼び出し側 attrs 経路。
        let html = render(&bubble::root(
            BubbleRootProps::default(),
            vec![("data-testid", payload)],
            vec![],
        ));
        assert_payload_is_escaped(payload, &html, "bubble::root attrs コンテキスト");

        // styled root の呼び出し側 class 属性経路（見た目クラスを持たない
        // ため drop_class_attr により class 属性自体が出力から消える）。
        let html = render(&bubble::root(
            BubbleRootProps::default(),
            vec![("class", payload)],
            vec![],
        ));
        assert!(
            !html.contains(payload),
            "bubble::root の class 属性に渡した生ペイロードが出力に残って\
             いる: payload={payload:?}, html={html}"
        );
        assert_eq!(html.matches("class=\"").count(), 0);

        // styled content の呼び出し側 attrs・children 経路。
        let html = render(&bubble::content(
            vec![("data-testid", payload)],
            vec![text(payload)],
        ));
        assert_payload_is_escaped(payload, &html, "bubble::content attrs コンテキスト");
        assert_payload_is_escaped(payload, &html, "bubble::content children コンテキスト");

        // styled reactions の label（aria-label）・attrs 経路。
        let html = render(&bubble::reactions(
            payload,
            vec![("data-testid", payload)],
            vec![],
        ));
        assert_payload_is_escaped(payload, &html, "bubble::reactions label コンテキスト");
        assert_payload_is_escaped(payload, &html, "bubble::reactions attrs コンテキスト");

        // styled reaction の呼び出し側 attrs・children 経路。
        let html = render(&bubble::reaction(
            false,
            vec![("data-testid", payload)],
            vec![text(payload)],
        ));
        assert_payload_is_escaped(payload, &html, "bubble::reaction attrs コンテキスト");
        assert_payload_is_escaped(payload, &html, "bubble::reaction children コンテキスト");

        // styled collapse_trigger の controls（aria-controls）・attrs・
        // children 経路。
        let html = render(&bubble::collapse_trigger(
            OpenState::Closed,
            Some(payload),
            vec![("data-testid", payload)],
            vec![text(payload)],
        ));
        assert_payload_is_escaped(
            payload,
            &html,
            "bubble::collapse_trigger controls コンテキスト",
        );
        assert_payload_is_escaped(
            payload,
            &html,
            "bubble::collapse_trigger attrs コンテキスト",
        );
        assert_payload_is_escaped(
            payload,
            &html,
            "bubble::collapse_trigger children コンテキスト",
        );

        // styled collapse_content の id・attrs・children 経路。
        let html = render(&bubble::collapse_content(
            OpenState::Open,
            Some(payload),
            vec![("data-testid", payload)],
            vec![text(payload)],
        ));
        assert_payload_is_escaped(payload, &html, "bubble::collapse_content id コンテキスト");
        assert_payload_is_escaped(
            payload,
            &html,
            "bubble::collapse_content attrs コンテキスト",
        );
        assert_payload_is_escaped(
            payload,
            &html,
            "bubble::collapse_content children コンテキスト",
        );
    }
}

/// (8) NumberInput 経路（イシュー #738）: styled `root` の呼び出し側
/// `attrs`・`class`、および headless-ui から選択的再エクスポートした
/// `label` の children・`input` の `name` の 4 箇所すべてで既定エスケープ
/// （REQ-1）が貫通することを固定する（checkbox 経路と同粒度）。
#[test]
fn number_input_styled_root_and_reexported_parts_are_escaped_for_all_payloads() {
    for payload in payloads::all() {
        // styled root の呼び出し側 attrs 経路。
        let html = render(&number_input::root(
            Size::Md,
            false,
            false,
            false,
            vec![("data-testid", payload)],
            vec![],
        ));
        assert_payload_is_escaped(
            payload,
            &html,
            "number_input::root 呼び出し側 attrs コンテキスト",
        );

        // styled root の class 属性経路（drop_class_attr により生ペイロードは
        // 出力されず、recipe 生成クラスへ完全に置き換わる）。
        let html = render(&number_input::root(
            Size::Md,
            false,
            false,
            false,
            vec![("class", payload)],
            vec![],
        ));
        assert!(
            !html.contains(payload),
            "number_input::root の class 属性に渡した生ペイロードが出力に残っている: \
             payload={payload:?}, html={html}"
        );
        assert_eq!(
            html.matches("class=\"").count(),
            1,
            "number_input::root の class 属性が複数出現している: html={html}"
        );
        assert!(
            html.contains("fd-number-input--"),
            "number_input::root で recipe 生成クラスが失われている: html={html}"
        );

        // 選択的再エクスポートした label の children 経路。
        let html = render(&number_input::label(
            NumberInputFlags::default(),
            None,
            vec![],
            vec![text(payload)],
        ));
        assert_payload_is_escaped(payload, &html, "number_input::label children コンテキスト");

        // 選択的再エクスポートした input の name 経路。
        let html = render(&number_input::input(
            payload,
            None,
            None,
            "0",
            "100",
            NumberInputFlags::default(),
            vec![],
        ));
        assert_payload_is_escaped(payload, &html, "number_input::input name コンテキスト");

        // 選択的再エクスポートした value_text の children 経路
        // （イシュー #1613 で headless 層に新設したパーツ）。
        let html = render(&number_input::value_text(
            NumberInputFlags::default(),
            vec![],
            vec![text(payload)],
        ));
        assert_payload_is_escaped(
            payload,
            &html,
            "number_input::value_text children コンテキスト",
        );
    }
}

#[test]
fn password_input_styled_root_and_reexported_parts_are_escaped_for_all_payloads() {
    for payload in payloads::all() {
        let field_props = PasswordInputProps {
            id: "pw",
            disabled: false,
            readonly: false,
            invalid: false,
            required: false,
            autocomplete: PasswordAutocomplete::CurrentPassword,
        };

        // styled root の呼び出し側 attrs 経路。
        let html = render(&password_input::root(
            Size::Md,
            ColorPalette::Accent,
            false,
            &field_props,
            vec![("data-testid", payload)],
            vec![],
        ));
        assert_payload_is_escaped(
            payload,
            &html,
            "password_input::root 呼び出し側 attrs コンテキスト",
        );

        // styled root の class 属性経路（drop_class_attr により生ペイロードは
        // 出力されず、recipe 生成クラスへ完全に置き換わる）。
        let html = render(&password_input::root(
            Size::Md,
            ColorPalette::Accent,
            false,
            &field_props,
            vec![("class", payload)],
            vec![],
        ));
        assert!(
            !html.contains(payload),
            "password_input::root の class 属性に渡した生ペイロードが出力に残っている: \
             payload={payload:?}, html={html}"
        );
        assert_eq!(
            html.matches("class=\"").count(),
            1,
            "password_input::root の class 属性が複数出現している: html={html}"
        );
        assert!(
            html.contains("fd-password-input--"),
            "password_input::root で recipe 生成クラスが失われている: html={html}"
        );

        // 選択的再エクスポートした label の children 経路。
        let html = render(&password_input::label(
            &field_props,
            vec![],
            vec![text(payload)],
        ));
        assert_payload_is_escaped(
            payload,
            &html,
            "password_input::label children コンテキスト",
        );

        // 選択的再エクスポートした id 由来の派生属性値経路（id そのものへの
        // ペイロード注入、`for`/`aria-controls` へ伝播する）。
        let id_props = PasswordInputProps {
            id: payload,
            disabled: false,
            readonly: false,
            invalid: false,
            required: false,
            autocomplete: PasswordAutocomplete::CurrentPassword,
        };
        let html = render(&password_input::visibility_trigger(
            false,
            &id_props,
            vec![],
            vec![],
        ));
        assert_payload_is_escaped(
            payload,
            &html,
            "password_input::visibility_trigger の aria-controls 属性値コンテキスト",
        );
        assert!(!html.contains("value="));
    }
}

/// (8) Slider 経路（イシュー #741）: styled `root` の呼び出し側 `attrs`・
/// `class`、および headless-ui から選択的再エクスポートした `label` の
/// children・`hidden_input` の `name` の 4 箇所すべてで既定エスケープ
/// （REQ-1）が貫通することを固定する（checkbox/number_input 経路と同粒度）。
/// styled `marker`/`marker_group`（イシュー #2020）の `attrs`・`style`
/// dedup は [`slider_marker_and_marker_group_are_escaped_for_all_payloads`]
/// で別途検証する。
#[test]
fn slider_styled_root_and_reexported_parts_are_escaped_for_all_payloads() {
    use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::slider::Slider;

    for payload in payloads::all() {
        let s = Slider::default();

        // styled root の呼び出し側 attrs 経路。
        let html = render(&slider::root(
            Size::Md,
            ColorPalette::Accent,
            &s,
            &slider::SliderProps::default(),
            vec![("data-testid", payload)],
            vec![],
        ));
        assert_payload_is_escaped(payload, &html, "slider::root 呼び出し側 attrs コンテキスト");

        // styled root の class 属性経路（drop_class_attr により生ペイロードは
        // 出力されず、recipe 生成クラスへ完全に置き換わる）。
        let html = render(&slider::root(
            Size::Md,
            ColorPalette::Accent,
            &s,
            &slider::SliderProps::default(),
            vec![("class", payload)],
            vec![],
        ));
        assert!(
            !html.contains(payload),
            "slider::root の class 属性に渡した生ペイロードが出力に残っている: \
             payload={payload:?}, html={html}"
        );
        assert_eq!(
            html.matches("class=\"").count(),
            1,
            "slider::root の class 属性が複数出現している: html={html}"
        );
        assert!(
            html.contains("fd-slider--"),
            "slider::root で recipe 生成クラスが失われている: html={html}"
        );

        // 選択的再エクスポートした label の children 経路。
        let html = render(&slider::label(
            &slider::SliderProps::default(),
            vec![],
            vec![text(payload)],
        ));
        assert_payload_is_escaped(payload, &html, "slider::label children コンテキスト");

        // 選択的再エクスポートした hidden_input の name 経路。
        let html = render(&slider::hidden_input(payload, "40", false, vec![]));
        assert_payload_is_escaped(payload, &html, "slider::hidden_input name コンテキスト");
    }
}

/// (8b) Slider marker/marker-group 経路（イシュー #2020）: styled `marker` の
/// 呼び出し側 `attrs`・`style` 上書き防止、`marker_group` の `attrs`・
/// children の各所で既定エスケープ（REQ-1）が貫通することを固定する
/// （`range`/`thumb_styled` と同粒度）。
#[test]
fn slider_marker_and_marker_group_are_escaped_for_all_payloads() {
    use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::slider::Slider;
    use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::Orientation;

    for payload in payloads::all() {
        let s = Slider::new(0.0, 100.0, 1.0, 40.0, Orientation::Horizontal);

        // styled marker の呼び出し側 attrs 経路。
        let html = render(&slider::marker(
            &s,
            20.0,
            false,
            vec![("data-testid", payload)],
            vec![],
        ));
        assert_payload_is_escaped(
            payload,
            &html,
            "slider::marker 呼び出し側 attrs コンテキスト",
        );

        // styled marker の style 上書き防止経路（フレームワーク生成の
        // `--fandhe-slider-marker-percent` を含む style のみが 1 つ出力される
        // ことを固定、`range`/`thumb_styled` と同型）。
        let html = render(&slider::marker(
            &s,
            20.0,
            false,
            vec![("style", payload)],
            vec![],
        ));
        assert!(
            !html.contains(payload),
            "slider::marker の style 属性に渡した生ペイロードが出力に残っている: \
             payload={payload:?}, html={html}"
        );
        assert_eq!(
            html.matches("style=\"").count(),
            1,
            "slider::marker の style 属性が複数出現している: html={html}"
        );
        assert!(
            html.contains("--fandhe-slider-marker-percent"),
            "slider::marker でフレームワーク生成の style が失われている: html={html}"
        );

        // styled marker_group の呼び出し側 attrs 経路。
        let html = render(&slider::marker_group(
            vec![("data-testid", payload)],
            vec![],
        ));
        assert_payload_is_escaped(
            payload,
            &html,
            "slider::marker_group 呼び出し側 attrs コンテキスト",
        );

        // styled marker_group の children 経路。
        let html = render(&slider::marker_group(vec![], vec![text(payload)]));
        assert_payload_is_escaped(payload, &html, "slider::marker_group children コンテキスト");
    }
}

/// (9) ImageCropper 経路（イシュー #844。シグネチャはイシュー #1610 で
/// `ImageCropperProps` 追加に追随）: styled `root` の呼び出し側
/// `attrs`・`class`、および headless-ui から選択的再エクスポートした
/// `image` の `src`/`alt`・`grid`（`attrs` 経路）の各所で既定エスケープ
/// （REQ-1）が貫通することを固定する（slider 経路と同粒度）。
#[test]
fn image_cropper_styled_root_and_reexported_parts_are_escaped_for_all_payloads() {
    use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::image_cropper::{
        ImageCropper, ImageCropperProps,
    };

    let props = ImageCropperProps::default();
    for payload in payloads::all() {
        let c = ImageCropper::default();

        // styled root の呼び出し側 attrs 経路。
        let html = render(&image_cropper::root(
            Size::Md,
            &c,
            &props,
            vec![("data-testid", payload)],
            vec![],
        ));
        assert_payload_is_escaped(
            payload,
            &html,
            "image_cropper::root 呼び出し側 attrs コンテキスト",
        );

        // styled root の class 属性経路（drop_class_attr により生ペイロードは
        // 出力されず、recipe 生成クラスへ完全に置き換わる）。
        let html = render(&image_cropper::root(
            Size::Md,
            &c,
            &props,
            vec![("class", payload)],
            vec![],
        ));
        assert!(
            !html.contains(payload),
            "image_cropper::root の class 属性に渡した生ペイロードが出力に残っている: \
             payload={payload:?}, html={html}"
        );
        assert_eq!(
            html.matches("class=\"").count(),
            1,
            "image_cropper::root の class 属性が複数出現している: html={html}"
        );
        assert!(
            html.contains("fd-image-cropper--"),
            "image_cropper::root で recipe 生成クラスが失われている: html={html}"
        );

        // 選択的再エクスポートした image の src/alt 経路（<img> タグ自体を
        // 出力するため実タグ有無チェックを含む assert_payload_is_escaped は
        // 使わず、エスケープ済み表現の実在・生ペイロードの不在のみを見る、
        // headless-ui 側テストと同じ整理）。
        let html = render(&image_cropper::image(payload, payload, vec![]));
        let expected_escaped = escape_html(payload);
        assert!(
            html.contains(&expected_escaped),
            "image_cropper::image の src/alt コンテキストで期待されるエスケープ済み表現が\
             出力に見当たらない: payload={payload:?}, html={html}"
        );
        assert!(
            !html.contains(payload),
            "image_cropper::image の src/alt コンテキストで生ペイロードが出力に\
             そのまま残っている: payload={payload:?}, html={html}"
        );

        // 選択的再エクスポートした grid の attrs 経路（イシュー #1610 で
        // `axis`/`props` 引数が増えた）。
        let html = render(&image_cropper::grid(
            None,
            &props,
            vec![("data-testid", payload)],
        ));
        assert_payload_is_escaped(payload, &html, "image_cropper::grid attrs コンテキスト");
    }
}

/// Splitter 経路（イシュー #826）: styled `root` の呼び出し側 `attrs`・
/// `class`、styled `panel` の `id`、および headless-ui から選択的
/// 再エクスポートした `resize_trigger_indicator` の children の 4 箇所すべて
/// で既定エスケープ（REQ-1）が貫通することを固定する（slider 経路と同粒度）。
#[test]
fn splitter_styled_root_panel_and_reexported_parts_are_escaped_for_all_payloads() {
    use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::splitter::{
        PanelSpec, Splitter,
    };
    use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::Orientation;

    for payload in payloads::all() {
        let s = Splitter::new(
            &[
                PanelSpec::new(50.0, 0.0, 100.0),
                PanelSpec::new(50.0, 0.0, 100.0),
            ],
            Orientation::Horizontal,
        );

        // styled root の呼び出し側 attrs 経路。
        let html = render(&splitter::root(
            Size::Md,
            ColorPalette::Accent,
            &s,
            false,
            vec![("data-testid", payload)],
            vec![],
        ));
        assert_payload_is_escaped(
            payload,
            &html,
            "splitter::root 呼び出し側 attrs コンテキスト",
        );

        // styled root の class 属性経路（drop_class_attr により生ペイロードは
        // 出力されず、recipe 生成クラスへ完全に置き換わる）。
        let html = render(&splitter::root(
            Size::Md,
            ColorPalette::Accent,
            &s,
            false,
            vec![("class", payload)],
            vec![],
        ));
        assert!(
            !html.contains(payload),
            "splitter::root の class 属性に渡した生ペイロードが出力に残っている: \
             payload={payload:?}, html={html}"
        );
        assert_eq!(
            html.matches("class=\"").count(),
            1,
            "splitter::root の class 属性が複数出現している: html={html}"
        );
        assert!(
            html.contains("fd-splitter--"),
            "splitter::root で recipe 生成クラスが失われている: html={html}"
        );

        // styled panel の id 属性経路。
        let html = render(&splitter::panel(&s, 0, payload, vec![], vec![]));
        assert_payload_is_escaped(payload, &html, "splitter::panel id コンテキスト");

        // 選択的再エクスポートした resize_trigger_indicator の children 経路。
        let html = render(&splitter::resize_trigger_indicator(
            vec![],
            vec![text(payload)],
        ));
        assert_payload_is_escaped(
            payload,
            &html,
            "splitter::resize_trigger_indicator children コンテキスト",
        );
    }
}

/// (9) pin_input 経路（イシュー #739、#2016 で separator を追加）: styled
/// `root` の呼び出し側 `attrs`・`class`、headless-ui から選択的再エクスポート
/// した `label` の children・`input` の `value`・`hidden_input` の
/// `name`/`value`、および pre-styled-only `separator` の `attrs`/`children`
/// の 6 箇所すべてで既定エスケープ（REQ-1）が貫通することを固定する
/// （`checkbox_styled_root_and_reexported_parts_are_escaped_for_all_payloads`
/// と同型）。
#[test]
fn pin_input_styled_root_and_reexported_parts_are_escaped_for_all_payloads() {
    for payload in payloads::all() {
        // styled root の呼び出し側 attrs 経路。
        let html = render(&pin_input::root(
            Size::Md,
            false,
            false,
            vec![("data-testid", payload)],
            vec![],
        ));
        assert_payload_is_escaped(
            payload,
            &html,
            "pin_input::root 呼び出し側 attrs コンテキスト",
        );

        // styled root の class 属性経路（drop_class_attr により生ペイロードは
        // 出力されず、recipe 生成クラスへ完全に置き換わる）。
        let html = render(&pin_input::root(
            Size::Md,
            false,
            false,
            vec![("class", payload)],
            vec![],
        ));
        assert!(
            !html.contains(payload),
            "pin_input::root の class 属性に渡した生ペイロードが出力に残っている: \
             payload={payload:?}, html={html}"
        );
        assert_eq!(
            html.matches("class=\"").count(),
            1,
            "pin_input::root の class 属性が複数出現している: html={html}"
        );
        assert!(
            html.contains("fd-pin-input--"),
            "pin_input::root で recipe 生成クラスが失われている: html={html}"
        );

        // 選択的再エクスポートした label の children 経路。
        let html = render(&pin_input::label(
            false,
            &PinInputProps::default(),
            vec![],
            vec![text(payload)],
        ));
        assert_payload_is_escaped(payload, &html, "pin_input::label children コンテキスト");

        // 選択的再エクスポートした input の value 経路。
        let html = render(&pin_input::input(
            0,
            1,
            payload,
            PinInputKind::Alphanumeric,
            false,
            false,
            &PinInputProps::default(),
            false,
            vec![],
        ));
        assert_payload_is_escaped(payload, &html, "pin_input::input value コンテキスト");

        // 選択的再エクスポートした hidden_input の name/value 経路。
        let html = render(&pin_input::hidden_input(payload, payload, false, vec![]));
        assert_payload_is_escaped(
            payload,
            &html,
            "pin_input::hidden_input name/value コンテキスト",
        );

        // pre-styled-only separator の attrs 経路（イシュー #2016）。
        let html = render(&pin_input::separator(vec![("data-x", payload)], vec![]));
        assert_payload_is_escaped(payload, &html, "pin_input::separator attrs コンテキスト");

        // pre-styled-only separator の children 経路（イシュー #2016）。
        let html = render(&pin_input::separator(vec![], vec![text(payload)]));
        assert_payload_is_escaped(payload, &html, "pin_input::separator children コンテキスト");
    }
}

/// (9) tags_input 経路（イシュー #744）: styled `root` の呼び出し側 `attrs`・
/// `class`、および headless-ui から選択的再エクスポートした `label` の
/// children・`item_text` の children（タグ文字列そのもの、REQ-1 の重点
/// 対象）・`item_input` の `value`・`item_delete_trigger` の `tag`
/// （`aria-label` に組み込まれる）・`hidden_input` の `name`/`value` の
/// 6 箇所すべてで既定エスケープ（REQ-1）が貫通することを固定する
/// （`pin_input_styled_root_and_reexported_parts_are_escaped_for_all_payloads`
/// と同型）。
#[test]
fn tags_input_styled_root_and_reexported_parts_are_escaped_for_all_payloads() {
    for payload in payloads::all() {
        // styled root の呼び出し側 attrs 経路。
        let html = render(&tags_input::root(
            Size::Md,
            false,
            vec![("data-testid", payload)],
            vec![],
        ));
        assert_payload_is_escaped(
            payload,
            &html,
            "tags_input::root 呼び出し側 attrs コンテキスト",
        );

        // styled root の class 属性経路（drop_class_attr により生ペイロードは
        // 出力されず、recipe 生成クラスへ完全に置き換わる）。
        let html = render(&tags_input::root(
            Size::Md,
            false,
            vec![("class", payload)],
            vec![],
        ));
        assert!(
            !html.contains(payload),
            "tags_input::root の class 属性に渡した生ペイロードが出力に残っている: \
             payload={payload:?}, html={html}"
        );
        assert_eq!(
            html.matches("class=\"").count(),
            1,
            "tags_input::root の class 属性が複数出現している: html={html}"
        );
        assert!(
            html.contains("fd-tags-input--"),
            "tags_input::root で recipe 生成クラスが失われている: html={html}"
        );

        // 選択的再エクスポートした label の children 経路。
        let html = render(&tags_input::label(
            &tags_input::TagsInputProps::default(),
            vec![],
            vec![text(payload)],
        ));
        assert_payload_is_escaped(payload, &html, "tags_input::label children コンテキスト");

        let item_state = tags_input::TagItem {
            value: payload,
            disabled: false,
            editing: false,
            highlighted: false,
        };

        // 選択的再エクスポートした item_text の children 経路（タグ文字列
        // そのもの、REQ-1 の重点対象）。
        let html = render(&tags_input::item_text(
            &item_state,
            vec![],
            vec![text(payload)],
        ));
        assert_payload_is_escaped(
            payload,
            &html,
            "tags_input::item_text children コンテキスト",
        );

        // 選択的再エクスポートした item_input の value 経路。
        let html = render(&tags_input::item_input(&item_state, payload, vec![]));
        assert_payload_is_escaped(payload, &html, "tags_input::item_input value コンテキスト");

        // 選択的再エクスポートした item_delete_trigger の aria-label コンテキスト。
        let html = render(&tags_input::item_delete_trigger(
            &item_state,
            vec![],
            vec![],
        ));
        assert_payload_is_escaped(
            payload,
            &html,
            "tags_input::item_delete_trigger aria-label コンテキスト",
        );

        // 選択的再エクスポートした hidden_input の name/value 経路。
        let html = render(&tags_input::hidden_input(
            &tags_input::TagsInputProps::default(),
            payload,
            payload,
            vec![],
        ));
        assert_payload_is_escaped(
            payload,
            &html,
            "tags_input::hidden_input name/value コンテキスト",
        );
    }
}

/// (10) file_upload 経路（イシュー #840）: styled `root` の呼び出し側
/// `attrs`・`class`、および headless-ui から選択的再エクスポートした
/// `label` の children・`item_name` の children（ファイル名そのもの、
/// REQ-1 の重点対象）・`item_delete_trigger` の `name`（`aria-label` に
/// 組み込まれる）・`hidden_input` の `accept` 属性の 5 箇所すべてで既定
/// エスケープ（REQ-1）が貫通することを固定する（`tags_input` 分と同型）。
#[test]
fn file_upload_styled_root_and_reexported_parts_are_escaped_for_all_payloads() {
    let props = file_upload::FileUploadProps::default();
    for payload in payloads::all() {
        // styled root の呼び出し側 attrs 経路。
        let html = render(&file_upload::root(
            Size::Md,
            &props,
            false,
            vec![("data-testid", payload)],
            vec![],
        ));
        assert_payload_is_escaped(
            payload,
            &html,
            "file_upload::root 呼び出し側 attrs コンテキスト",
        );

        // styled root の class 属性経路（drop_class_attr により生ペイロードは
        // 出力されず、recipe 生成クラスへ完全に置き換わる）。
        let html = render(&file_upload::root(
            Size::Md,
            &props,
            false,
            vec![("class", payload)],
            vec![],
        ));
        assert!(
            !html.contains(payload),
            "file_upload::root の class 属性に渡した生ペイロードが出力に残っている: \
             payload={payload:?}, html={html}"
        );
        assert_eq!(
            html.matches("class=\"").count(),
            1,
            "file_upload::root の class 属性が複数出現している: html={html}"
        );
        assert!(
            html.contains("fd-file-upload--"),
            "file_upload::root で recipe 生成クラスが失われている: html={html}"
        );

        // 選択的再エクスポートした label の children 経路。
        let html = render(&file_upload::label(&props, vec![], vec![text(payload)]));
        assert_payload_is_escaped(payload, &html, "file_upload::label children コンテキスト");

        // 選択的再エクスポートした item_name の children 経路（ファイル名
        // そのもの、REQ-1 の重点対象）。
        let html = render(&file_upload::item_name(
            file_upload::ItemType::Accepted,
            &props,
            vec![],
            vec![text(payload)],
        ));
        assert_payload_is_escaped(
            payload,
            &html,
            "file_upload::item_name children コンテキスト",
        );

        // 選択的再エクスポートした item_delete_trigger の aria-label コンテキスト。
        let html = render(&file_upload::item_delete_trigger(
            payload,
            file_upload::ItemType::Accepted,
            &props,
            vec![],
            vec![],
        ));
        assert_payload_is_escaped(
            payload,
            &html,
            "file_upload::item_delete_trigger aria-label コンテキスト",
        );

        // 選択的再エクスポートした hidden_input の accept 属性コンテキスト。
        let html = render(&file_upload::hidden_input(payload, false, &props, vec![]));
        assert_payload_is_escaped(
            payload,
            &html,
            "file_upload::hidden_input accept コンテキスト",
        );
    }
}

#[test]
fn listbox_styled_root_and_reexported_parts_are_escaped_for_all_payloads() {
    let props = listbox::ListboxProps::default();
    for payload in payloads::all() {
        // styled root の呼び出し側 attrs 経路。
        let html = render(&listbox::root(
            Size::Md,
            OpenState::Closed,
            &props,
            vec![("data-testid", payload)],
            vec![],
        ));
        assert_payload_is_escaped(
            payload,
            &html,
            "listbox::root 呼び出し側 attrs コンテキスト",
        );

        // styled root の class 属性経路（drop_class_attr により生ペイロードは
        // 出力されず、recipe 生成クラスへ完全に置き換わる）。
        let html = render(&listbox::root(
            Size::Md,
            OpenState::Closed,
            &props,
            vec![("class", payload)],
            vec![],
        ));
        assert!(
            !html.contains(payload),
            "listbox::root の class 属性に渡した生ペイロードが出力に残っている: \
             payload={payload:?}, html={html}"
        );
        assert_eq!(
            html.matches("class=\"").count(),
            1,
            "listbox::root の class 属性が複数出現している: html={html}"
        );
        assert!(
            html.contains("fd-listbox--"),
            "listbox::root で recipe 生成クラスが失われている: html={html}"
        );

        // 選択的再エクスポートした label の id/children 経路。
        let html = render(&listbox::label(
            &props,
            Some(payload),
            vec![],
            vec![text(payload)],
        ));
        assert_payload_is_escaped(payload, &html, "listbox::label id/children コンテキスト");

        // 選択的再エクスポートした content の id/labelledby/activedescendant 経路。
        let html = render(&listbox::content(
            false,
            &props,
            Some(payload),
            Some(payload),
            Some(payload),
            vec![],
            vec![],
        ));
        assert_payload_is_escaped(
            payload,
            &html,
            "listbox::content id/labelledby/activedescendant コンテキスト",
        );

        // 選択的再エクスポートした item の value/id 経路（タグ文字列そのもの、
        // REQ-1 の重点対象）。
        let html = render(&listbox::item(
            OpenState::Open,
            &props,
            false,
            false,
            payload,
            Some(payload),
            vec![],
            vec![],
        ));
        assert_payload_is_escaped(payload, &html, "listbox::item data-value/id コンテキスト");

        // 選択的再エクスポートした item_text の id/children 経路。
        let html = render(&listbox::item_text(
            OpenState::Open,
            &props,
            false,
            false,
            Some(payload),
            vec![],
            vec![text(payload)],
        ));
        assert_payload_is_escaped(
            payload,
            &html,
            "listbox::item_text id/children コンテキスト",
        );

        // 選択的再エクスポートした value_text の children 経路。
        let html = render(&listbox::value_text(
            false,
            &props,
            vec![],
            vec![text(payload)],
        ));
        assert_payload_is_escaped(payload, &html, "listbox::value_text children コンテキスト");
    }
}

#[test]
fn rating_group_styled_root_and_reexported_parts_are_escaped_for_all_payloads() {
    for payload in payloads::all() {
        let props = rating_group::RatingGroupProps::default();

        // styled root の呼び出し側 attrs 経路。
        let html = render(&rating_group::root(
            Size::Md,
            ColorPalette::Accent,
            &props,
            vec![("data-testid", payload)],
            vec![],
        ));
        assert_payload_is_escaped(
            payload,
            &html,
            "rating_group::root 呼び出し側 attrs コンテキスト",
        );

        // styled root の class 属性経路（drop_class_attr により生ペイロードは
        // 出力されず、recipe 生成クラスへ完全に置き換わる）。
        let html = render(&rating_group::root(
            Size::Md,
            ColorPalette::Accent,
            &props,
            vec![("class", payload)],
            vec![],
        ));
        assert!(
            !html.contains(payload),
            "rating_group::root の class 属性に渡した生ペイロードが出力に残っている: \
             payload={payload:?}, html={html}"
        );
        assert_eq!(
            html.matches("class=\"").count(),
            1,
            "rating_group::root の class 属性が複数出現している: html={html}"
        );
        assert!(
            html.contains("fd-rating-group--"),
            "rating_group::root で recipe 生成クラスが失われている: html={html}"
        );

        // 選択的再エクスポートした label の children 経路。
        let html = render(&rating_group::label(
            &props,
            None,
            vec![],
            vec![text(payload)],
        ));
        assert_payload_is_escaped(payload, &html, "rating_group::label children コンテキスト");

        // 選択的再エクスポートした item の aria_label 経路。
        let html = render(&rating_group::item(
            1,
            RatingItemFlags::default(),
            payload,
            vec![],
            vec![],
        ));
        assert_payload_is_escaped(payload, &html, "rating_group::item aria_label コンテキスト");

        // 選択的再エクスポートした hidden_input の name 経路。
        let html = render(&rating_group::hidden_input(
            &props,
            Some(payload),
            "3",
            vec![],
        ));
        assert_payload_is_escaped(
            payload,
            &html,
            "rating_group::hidden_input name コンテキスト",
        );
    }
}
/// (10) Editable 経路（イシュー #745）: styled `root` の呼び出し側 `attrs`・
/// `class`、および headless-ui から選択的再エクスポートした `label` の
/// children・`input` の `name`/`value`・`preview` の children の 5 箇所
/// すべてで既定エスケープ（REQ-1）が貫通することを固定する
/// （`number_input_styled_root_and_reexported_parts_are_escaped_for_all_payloads`
/// と同型）。
#[test]
fn editable_styled_root_and_reexported_parts_are_escaped_for_all_payloads() {
    for payload in payloads::all() {
        // styled root の呼び出し側 attrs 経路。
        let html = render(&editable::root(
            Size::Md,
            EditMode::Preview,
            EditableInputFlags::default(),
            Default::default(),
            Default::default(),
            vec![("data-testid", payload)],
            vec![],
        ));
        assert_payload_is_escaped(
            payload,
            &html,
            "editable::root 呼び出し側 attrs コンテキスト",
        );

        // styled root の class 属性経路（drop_class_attr により生ペイロードは
        // 出力されず、recipe 生成クラスへ完全に置き換わる）。
        let html = render(&editable::root(
            Size::Md,
            EditMode::Preview,
            EditableInputFlags::default(),
            Default::default(),
            Default::default(),
            vec![("class", payload)],
            vec![],
        ));
        assert!(
            !html.contains(payload),
            "editable::root の class 属性に渡した生ペイロードが出力に残っている: \
             payload={payload:?}, html={html}"
        );
        assert_eq!(
            html.matches("class=\"").count(),
            1,
            "editable::root の class 属性が複数出現している: html={html}"
        );
        assert!(
            html.contains("fd-editable--"),
            "editable::root で recipe 生成クラスが失われている: html={html}"
        );

        // 選択的再エクスポートした label の children 経路。
        let html = editable::label(
            EditMode::Preview,
            EditableInputFlags::default(),
            None,
            vec![],
            vec![text(payload)],
        );
        let html = render(&html);
        assert_payload_is_escaped(payload, &html, "editable::label children コンテキスト");

        // 選択的再エクスポートした input の name/value 経路。
        let html = render(&editable::input(
            EditMode::Edit,
            payload,
            payload,
            EditableInputProps::default(),
            EditableInputFlags::default(),
            vec![],
        ));
        assert_payload_is_escaped(payload, &html, "editable::input name/value コンテキスト");

        // 選択的再エクスポートした preview の children 経路。
        let html = render(&editable::preview(
            EditMode::Preview,
            EditableInputFlags::default(),
            false,
            vec![],
            vec![text(payload)],
        ));
        assert_payload_is_escaped(payload, &html, "editable::preview children コンテキスト");
    }
}

/// (10) steps 経路（イシュー #752）: styled `root` の呼び出し側 `attrs`・
/// `class`、および全パーツが `state: &Steps` を取る `item`/`trigger` の
/// children/attrs 経路すべてで既定エスケープ（REQ-1）が貫通することを固定
/// する（`slider_styled_root_and_reexported_parts_are_escaped_for_all_payloads`
/// と同型）。
#[test]
fn steps_styled_root_and_parts_are_escaped_for_all_payloads() {
    use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::steps::Steps;
    use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::Orientation;

    for payload in payloads::all() {
        let s = Steps::new(3, 1, Orientation::Horizontal);

        // styled root の呼び出し側 attrs 経路。
        let html = render(&steps::root(
            Size::Md,
            ColorPalette::Accent,
            &s,
            vec![("data-testid", payload)],
            vec![],
        ));
        assert_payload_is_escaped(payload, &html, "steps::root 呼び出し側 attrs コンテキスト");

        // styled root の class 属性経路（drop_class_attr により生ペイロードは
        // 出力されず、recipe 生成クラスへ完全に置き換わる）。
        let html = render(&steps::root(
            Size::Md,
            ColorPalette::Accent,
            &s,
            vec![("class", payload)],
            vec![],
        ));
        assert!(
            !html.contains(payload),
            "steps::root の class 属性に渡した生ペイロードが出力に残っている: \
             payload={payload:?}, html={html}"
        );
        assert_eq!(
            html.matches("class=\"").count(),
            1,
            "steps::root の class 属性が複数出現している: html={html}"
        );
        assert!(
            html.contains("fd-steps--"),
            "steps::root で recipe 生成クラスが失われている: html={html}"
        );

        // item の children 経路。
        let html = render(&steps::item(&s, 0, vec![], vec![text(payload)]));
        assert_payload_is_escaped(payload, &html, "steps::item children コンテキスト");

        // trigger の呼び出し側 attrs 経路。
        let html = render(&steps::trigger(
            &s,
            1,
            vec![("data-testid", payload)],
            vec![],
        ));
        assert_payload_is_escaped(
            payload,
            &html,
            "steps::trigger 呼び出し側 attrs コンテキスト",
        );
    }
}

/// QrCode（イシュー #774）: styled `root` の呼び出し側 `attrs`・`class`、
/// 選択的再エクスポートした `overlay` の children・`frame` の `aria_label`
/// の各所で既定エスケープ（REQ-1）が貫通することを固定する。`value`（符号化
/// 対象文字列）そのものは出力へ一切漏出しないこと（`pattern` の `d` 属性値が
/// 固定文字集合に閉じること）も headless 層と同型に確認する
/// （`crates/headless-ui/tests/xss_escape.rs::qr_code_value_never_leaks_into_output_for_all_payloads`
/// と対になる styled 層側の固定）。
#[test]
fn qr_code_styled_root_and_reexported_parts_are_escaped_for_all_payloads() {
    for payload in payloads::all() {
        // styled root の呼び出し側 attrs 経路。
        let html = render(&qr_code::root(
            Size::Md,
            vec![("data-testid", payload)],
            vec![],
        ));
        assert_payload_is_escaped(
            payload,
            &html,
            "qr_code::root 呼び出し側 attrs コンテキスト",
        );

        // styled root の class 属性経路（drop_class_attr により生ペイロードは
        // 出力されず、recipe 生成クラスへ完全に置き換わる）。
        let html = render(&qr_code::root(Size::Md, vec![("class", payload)], vec![]));
        assert!(
            !html.contains(payload),
            "qr_code::root の class 属性に渡した生ペイロードが出力に残っている: \
             payload={payload:?}, html={html}"
        );
        assert_eq!(
            html.matches("class=\"").count(),
            1,
            "qr_code::root の class 属性が複数出現している: html={html}"
        );
        assert!(
            html.contains("fd-qr-code--"),
            "qr_code::root で recipe 生成クラスが失われている: html={html}"
        );

        // 選択的再エクスポートした overlay の children 経路。
        let html = render(&qr_code::overlay(vec![], vec![text(payload)]));
        assert_payload_is_escaped(payload, &html, "qr_code::overlay children コンテキスト");

        // value 自体は出力へ一切漏出しない（headless 層と同型の不変条件）。
        let matrix = qr_code::encode(payload, qr_code::ErrorCorrectionLevel::L)
            .expect("payload はいずれもバージョン 40 容量内に収まる");
        let frame_html = render(&qr_code::frame(
            &matrix,
            qr_code::DEFAULT_QUIET_ZONE,
            Some(payload),
            vec![],
            vec![],
        ));
        assert_payload_is_escaped(
            payload,
            &frame_html,
            "qr_code::frame aria_label コンテキスト",
        );

        let pattern_html = render(&qr_code::pattern(
            &matrix,
            qr_code::DEFAULT_QUIET_ZONE,
            vec![],
        ));
        assert!(
            !pattern_html.contains(payload),
            "qr_code::pattern の d 属性値へ value が漏出している: payload={payload:?}, html={pattern_html}"
        );
    }
}

/// (11) pagination 経路（イシュー #751）: styled `root` の呼び出し側
/// `attrs`・`class`・`aria_label`、および headless-ui から選択的
/// 再エクスポートした `item` の `href`（Link モード）・children、
/// `ellipsis`/`prev_trigger`/`next_trigger` の呼び出し側 `attrs` の各所で
/// 既定エスケープ（REQ-1）が貫通することを固定する
/// （`tags_input_styled_root_and_reexported_parts_are_escaped_for_all_payloads`
/// と同型）。
#[test]
fn pagination_styled_root_and_reexported_parts_are_escaped_for_all_payloads() {
    for payload in payloads::all() {
        // styled root の aria_label 経路。
        let html = render(&pagination::root(
            Size::Md,
            ColorPalette::Accent,
            payload,
            vec![],
            vec![],
        ));
        assert_payload_is_escaped(payload, &html, "pagination::root aria_label コンテキスト");

        // styled root の呼び出し側 attrs 経路。
        let html = render(&pagination::root(
            Size::Md,
            ColorPalette::Accent,
            "pagination",
            vec![("data-testid", payload)],
            vec![],
        ));
        assert_payload_is_escaped(
            payload,
            &html,
            "pagination::root 呼び出し側 attrs コンテキスト",
        );

        // styled root の class 属性経路（drop_class_attr により生ペイロードは
        // 出力されず、recipe 生成クラスへ完全に置き換わる）。
        let html = render(&pagination::root(
            Size::Md,
            ColorPalette::Accent,
            "pagination",
            vec![("class", payload)],
            vec![],
        ));
        assert!(
            !html.contains(payload),
            "pagination::root の class 属性に渡した生ペイロードが出力に残っている: \
             payload={payload:?}, html={html}"
        );
        assert_eq!(
            html.matches("class=\"").count(),
            1,
            "pagination::root の class 属性が複数出現している: html={html}"
        );
        assert!(
            html.contains("fd-pagination--"),
            "pagination::root で recipe 生成クラスが失われている: html={html}"
        );

        // 選択的再エクスポートした item の Link モード href 経路。
        let html = render(&pagination::item(
            ItemMode::Link { href: payload },
            1,
            false,
            false,
            vec![],
            vec![],
        ));
        assert_payload_is_escaped(payload, &html, "pagination::item href コンテキスト");

        // 選択的再エクスポートした item の children 経路。
        let html = render(&pagination::item(
            ItemMode::Button,
            1,
            false,
            false,
            vec![],
            vec![text(payload)],
        ));
        assert_payload_is_escaped(payload, &html, "pagination::item children コンテキスト");

        // 選択的再エクスポートした ellipsis の呼び出し側 attrs 経路。
        let html = render(&pagination::ellipsis(
            vec![("data-testid", payload)],
            vec![],
        ));
        assert_payload_is_escaped(payload, &html, "pagination::ellipsis attrs コンテキスト");

        // 選択的再エクスポートした prev_trigger/next_trigger の呼び出し側
        // attrs 経路。
        let html = render(&pagination::prev_trigger(
            ItemMode::Button,
            false,
            vec![("data-testid", payload)],
            vec![],
        ));
        assert_payload_is_escaped(
            payload,
            &html,
            "pagination::prev_trigger attrs コンテキスト",
        );

        let html = render(&pagination::next_trigger(
            ItemMode::Button,
            false,
            vec![("data-testid", payload)],
            vec![],
        ));
        assert_payload_is_escaped(
            payload,
            &html,
            "pagination::next_trigger attrs コンテキスト",
        );
    }
}

/// styled CheckboxCard（イシュー #747）の XSS 回帰
/// （`checkbox_styled_root_and_reexported_parts_are_escaped_for_all_payloads`
/// と同型）。
#[test]
fn checkbox_card_styled_root_and_parts_are_escaped_for_all_payloads() {
    for payload in payloads::all() {
        // styled root の呼び出し側 attrs 経路。
        let html = render(&checkbox_card::root(
            Size::Md,
            ColorPalette::Accent,
            &CheckboxProps::default(),
            vec![("data-testid", payload)],
            vec![],
        ));
        assert_payload_is_escaped(
            payload,
            &html,
            "checkbox_card::root 呼び出し側 attrs コンテキスト",
        );

        // styled root の class 属性経路（drop_class_attr により生ペイロードは
        // 出力されず、recipe 生成クラスへ完全に置き換わる）。
        let html = render(&checkbox_card::root(
            Size::Md,
            ColorPalette::Accent,
            &CheckboxProps::default(),
            vec![("class", payload)],
            vec![],
        ));
        assert!(
            !html.contains(payload),
            "checkbox_card::root の class 属性に渡した生ペイロードが出力に残っている: \
             payload={payload:?}, html={html}"
        );
        assert_eq!(
            html.matches("class=\"").count(),
            1,
            "checkbox_card::root の class 属性が複数出現している: html={html}"
        );
        assert!(
            html.contains("fd-checkbox-card--"),
            "checkbox_card::root で recipe 生成クラスが失われている: html={html}"
        );

        // label の children 経路。
        let html = render(&checkbox_card::label(
            &CheckboxProps::default(),
            vec![],
            vec![text(payload)],
        ));
        assert_payload_is_escaped(payload, &html, "checkbox_card::label children コンテキスト");

        // hidden_input の name/value 経路。
        let html = render(&checkbox_card::hidden_input(
            &CheckboxProps::default(),
            payload,
            payload,
            vec![],
        ));
        assert_payload_is_escaped(
            payload,
            &html,
            "checkbox_card::hidden_input name/value コンテキスト",
        );
    }
}

/// styled RadioCard（イシュー #747）の XSS 回帰
/// （`checkbox_card_styled_root_and_parts_are_escaped_for_all_payloads` と同型）。
#[test]
fn radio_card_styled_root_and_parts_are_escaped_for_all_payloads() {
    for payload in payloads::all() {
        // styled root の呼び出し側 attrs 経路。
        let html = render(&radio_card::root(
            Size::Md,
            ColorPalette::Accent,
            false,
            None,
            None,
            vec![("data-testid", payload)],
            vec![],
        ));
        assert_payload_is_escaped(
            payload,
            &html,
            "radio_card::root 呼び出し側 attrs コンテキスト",
        );

        // styled root の class 属性経路（drop_class_attr により生ペイロードは
        // 出力されず、recipe 生成クラスへ完全に置き換わる）。
        let html = render(&radio_card::root(
            Size::Md,
            ColorPalette::Accent,
            false,
            None,
            None,
            vec![("class", payload)],
            vec![],
        ));
        assert!(
            !html.contains(payload),
            "radio_card::root の class 属性に渡した生ペイロードが出力に残っている: \
             payload={payload:?}, html={html}"
        );
        assert_eq!(
            html.matches("class=\"").count(),
            1,
            "radio_card::root の class 属性が複数出現している: html={html}"
        );
        assert!(
            html.contains("fd-radio-card--"),
            "radio_card::root で recipe 生成クラスが失われている: html={html}"
        );

        // item の data-value/children 経路。
        let html = render(&radio_card::item(
            false,
            false,
            payload,
            vec![],
            vec![text(payload)],
        ));
        assert_payload_is_escaped(
            payload,
            &html,
            "radio_card::item data-value/children コンテキスト",
        );

        // item_text の children 経路。
        let html = render(&radio_card::item_text(vec![], vec![text(payload)]));
        assert_payload_is_escaped(
            payload,
            &html,
            "radio_card::item_text children コンテキスト",
        );

        // item_hidden_input の name/value 経路。
        let html = render(&radio_card::item_hidden_input(
            false,
            false,
            Some(payload),
            payload,
            vec![],
        ));
        assert_payload_is_escaped(
            payload,
            &html,
            "radio_card::item_hidden_input name/value コンテキスト",
        );
    }
}

/// styled Toast（イシュー #760）の XSS 回帰: styled `group`/`root` の呼び出し側
/// attrs・class 属性経路、および再エクスポート済み `title`/`description` の
/// children 経路を固定する。
#[test]
fn toast_styled_group_root_and_reexported_parts_are_escaped_for_all_payloads() {
    for payload in payloads::all() {
        // styled group の呼び出し側 attrs 経路 + aria-label 経路。
        let html = render(&toast::group(
            ToastPlacement::Bottom,
            payload,
            vec![("data-testid", payload)],
            vec![],
        ));
        assert_payload_is_escaped(payload, &html, "toast::group aria-label/attrs コンテキスト");

        // styled group の class 属性経路（drop_class_attr により生ペイロードは
        // 出力されず、recipe 生成クラスへ完全に置き換わる）。
        let html = render(&toast::group(
            ToastPlacement::Bottom,
            "Notifications",
            vec![("class", payload)],
            vec![],
        ));
        assert!(
            !html.contains(payload),
            "toast::group の class 属性に渡した生ペイロードが出力に残っている: \
             payload={payload:?}, html={html}"
        );
        assert_eq!(
            html.matches("class=\"").count(),
            1,
            "toast::group の class 属性が複数出現している: html={html}"
        );
        assert!(
            html.contains("fd-toast--placement-"),
            "toast::group で recipe 生成クラスが失われている: html={html}"
        );

        // styled root の呼び出し側 attrs 経路 + title/description children 経路。
        let html = render(&toast::root(
            ToastStatus::Error,
            vec![("data-testid", payload)],
            vec![
                toast::title(vec![], vec![text(payload)]),
                toast::description(vec![], vec![text(payload)]),
            ],
        ));
        assert_payload_is_escaped(
            payload,
            &html,
            "toast::root attrs/title/description コンテキスト",
        );

        // styled root の class 属性経路。
        let html = render(&toast::root(
            ToastStatus::Error,
            vec![("class", payload)],
            vec![],
        ));
        assert!(
            !html.contains(payload),
            "toast::root の class 属性に渡した生ペイロードが出力に残っている: \
             payload={payload:?}, html={html}"
        );
        assert_eq!(
            html.matches("class=\"").count(),
            1,
            "toast::root の class 属性が複数出現している: html={html}"
        );
        assert!(
            html.contains("fd-toast--status-"),
            "toast::root で recipe 生成クラスが失われている: html={html}"
        );

        // action_trigger/close_trigger の children 経路（headless からの再エクスポート）。
        let html = render(&toast::action_trigger(vec![], vec![text(payload)]));
        assert_payload_is_escaped(
            payload,
            &html,
            "toast::action_trigger children コンテキスト",
        );

        let html = render(&toast::close_trigger(vec![], vec![text(payload)]));
        assert_payload_is_escaped(payload, &html, "toast::close_trigger children コンテキスト");
    }
}

/// styled HoverCard（イシュー #759）の XSS 回帰。[`hover_card`] は headless
/// 層をそのまま再エクスポートする薄い委譲層（`pub use ...::*`）であるため、
/// `crates/headless-ui/tests/xss_escape.rs::hover_card_href_and_content_id_are_escaped_for_all_payloads`
/// と同じ観点を `fandhe-frontend-pre-styled-ui` の公開 API 経由でも固定する
/// （styled 層のみに依存する利用者が同じ保証を得られることの確認）。
#[test]
fn hover_card_styled_trigger_href_and_content_id_are_escaped_for_all_payloads() {
    for payload in payloads::all() {
        // URL 属性経路: trigger の href。
        let html = render(&hover_card::trigger(
            OpenState::Closed,
            Some(payload),
            vec![],
            vec![],
        ));
        assert_payload_is_escaped(payload, &html, "hover_card::trigger href コンテキスト");

        // 属性値経路: content の id。
        let html = render(&hover_card::content(
            OpenState::Open,
            Some(payload),
            vec![],
            vec![],
        ));
        assert_payload_is_escaped(payload, &html, "hover_card::content id コンテキスト");

        // 属性値経路: root の呼び出し側 attrs（data-testid）。
        let html = render(&hover_card::root(
            OpenState::Closed,
            HoverCardDelays::default(),
            vec![("data-testid", payload)],
            vec![],
        ));
        assert_payload_is_escaped(
            payload,
            &html,
            "hover_card::root 呼び出し側 attrs コンテキスト",
        );

        // テキスト経路: content の children。
        let html = render(&hover_card::content(
            OpenState::Open,
            None,
            vec![],
            vec![text(payload)],
        ));
        assert_payload_is_escaped(payload, &html, "hover_card::content children コンテキスト");
    }

    // URL 属性経路: javascript: スキームは href 属性ごと出力から除去される
    // （`avatar_image_src_rejects_dangerous_url_schemes` と同型の許可リスト
    // 契約が styled 層の再エクスポート経由でも貫通することを固定する）。
    let html = render(&hover_card::trigger(
        OpenState::Closed,
        Some("javascript:alert(1)"),
        vec![],
        vec![],
    ));
    assert!(!html.contains("javascript:"));
    assert!(!html.contains("href="));
}

/// styled Collapsible（イシュー #1682）の XSS 回帰。[`collapsible`] は headless
/// 層をそのまま再エクスポートする薄い委譲層（`pub use ...::*`）であるため、
/// `crates/headless-ui/tests/collapsible.rs` の XSS 観点を
/// `fandhe-frontend-pre-styled-ui` の公開 API 経由でも固定する（styled 層の
/// みに依存する利用者が同じ保証を得られることの確認）。
#[test]
fn collapsible_styled_trigger_controls_and_content_id_are_escaped_for_all_payloads() {
    use collapsible::OpenState;

    for payload in payloads::all() {
        // 属性値経路: trigger の controls（aria-controls）。
        let html = render(&collapsible::trigger(
            OpenState::Closed,
            false,
            Some(payload),
            vec![],
            vec![],
        ));
        assert_payload_is_escaped(payload, &html, "collapsible::trigger controls コンテキスト");

        // 属性値経路: content の id。
        let html = render(&collapsible::content(
            OpenState::Open,
            false,
            Some(payload),
            vec![],
            vec![],
        ));
        assert_payload_is_escaped(payload, &html, "collapsible::content id コンテキスト");

        // 属性値経路: root の呼び出し側 attrs（data-testid）。
        let html = render(&collapsible::root(
            OpenState::Closed,
            false,
            vec![("data-testid", payload)],
            vec![],
        ));
        assert_payload_is_escaped(
            payload,
            &html,
            "collapsible::root 呼び出し側 attrs コンテキスト",
        );

        // テキスト経路: trigger/content の children。
        let html = render(&collapsible::trigger(
            OpenState::Closed,
            false,
            None,
            vec![],
            vec![text(payload)],
        ));
        assert_payload_is_escaped(payload, &html, "collapsible::trigger children コンテキスト");

        let html = render(&collapsible::content(
            OpenState::Open,
            false,
            None,
            vec![],
            vec![text(payload)],
        ));
        assert_payload_is_escaped(payload, &html, "collapsible::content children コンテキスト");
    }

    // 予約属性（data-state/data-disabled/hidden）の呼び出し側偽装が
    // styled 経路でも除去されることを固定する（headless 側テストの再固定）。
    let html = render(&collapsible::root(
        OpenState::Open,
        false,
        vec![("data-state", "closed"), ("data-disabled", "spoofed")],
        vec![],
    ));
    assert!(html.contains(r#"data-state="open""#));
    assert!(!html.contains("spoofed"));
    assert!(!html.contains("data-disabled"));
}

/// (10) carousel 経路（イシュー #754）: styled `root` の呼び出し側 `attrs`・
/// `class`（`aria-label` 引数含む）、および headless-ui から選択的
/// 再エクスポートした `prev_trigger`/`indicator` の `aria-label`・`item` の
/// children の各所すべてで既定エスケープ（REQ-1）が貫通することを固定する
/// （`slider_styled_root_and_reexported_parts_are_escaped_for_all_payloads`
/// と同型）。
#[test]
fn carousel_styled_root_and_reexported_parts_are_escaped_for_all_payloads() {
    use fandhe_frontend_pre_styled_ui::carousel;
    use fandhe_frontend_pre_styled_ui::carousel::Orientation;

    for payload in payloads::all() {
        // styled root の `aria-label` 引数経路。
        let html = render(&carousel::root(
            Size::Md,
            Orientation::Horizontal,
            payload,
            vec![],
            vec![],
        ));
        assert_payload_is_escaped(payload, &html, "carousel::root aria-label コンテキスト");

        // styled root の呼び出し側 attrs 経路。
        let html = render(&carousel::root(
            Size::Md,
            Orientation::Horizontal,
            "Products",
            vec![("data-testid", payload)],
            vec![],
        ));
        assert_payload_is_escaped(
            payload,
            &html,
            "carousel::root 呼び出し側 attrs コンテキスト",
        );

        // styled root の class 属性経路（drop_class_attr により生ペイロードは
        // 出力されず、recipe 生成クラスへ完全に置き換わる）。
        let html = render(&carousel::root(
            Size::Md,
            Orientation::Horizontal,
            "Products",
            vec![("class", payload)],
            vec![],
        ));
        assert!(
            !html.contains(payload),
            "carousel::root の class 属性に渡した生ペイロードが出力に残っている: \
             payload={payload:?}, html={html}"
        );
        assert_eq!(
            html.matches("class=\"").count(),
            1,
            "carousel::root の class 属性が複数出現している: html={html}"
        );
        assert!(
            html.contains("fd-carousel--"),
            "carousel::root で recipe 生成クラスが失われている: html={html}"
        );

        // 選択的再エクスポートした prev_trigger の aria-label 経路。
        let html = render(&carousel::prev_trigger(
            Orientation::Horizontal,
            false,
            payload,
            vec![],
            vec![],
        ));
        assert_payload_is_escaped(
            payload,
            &html,
            "carousel::prev_trigger aria-label コンテキスト",
        );

        // 選択的再エクスポートした item の children 経路。
        let html = render(&carousel::item(
            Orientation::Horizontal,
            0,
            1,
            false,
            vec![],
            vec![text(payload)],
        ));
        assert_payload_is_escaped(payload, &html, "carousel::item children コンテキスト");
    }
}

/// (11) action_bar 経路（イシュー #762）: 再エクスポートした `content` の
/// `aria-label`（属性値経路）・`selection_trigger`/`close_trigger` の
/// children（テキスト経路）で既定エスケープ（REQ-1）が貫通することを固定
/// する（`tooltip_styled_root_and_reexported_parts_are_escaped_for_all_payloads`
/// と同型）。
#[test]
fn action_bar_reexported_parts_are_escaped_for_all_payloads() {
    use fandhe_frontend_pre_styled_ui::action_bar;

    for payload in payloads::all() {
        let html = render(&action_bar::content(
            OpenState::Open,
            payload,
            vec![],
            vec![],
        ));
        assert_payload_is_escaped(
            payload,
            &html,
            "action_bar::content aria-label コンテキスト",
        );

        let html = render(&action_bar::selection_trigger(vec![], vec![text(payload)]));
        assert_payload_is_escaped(
            payload,
            &html,
            "action_bar::selection_trigger children コンテキスト",
        );

        let html = render(&action_bar::close_trigger(vec![], vec![text(payload)]));
        assert_payload_is_escaped(
            payload,
            &html,
            "action_bar::close_trigger children コンテキスト",
        );
    }
}

/// イシュー #764: `skeleton::skeleton` の呼び出し側 `attrs`（`data-testid` 等）
/// と `class` の 2 箇所で既定エスケープ（REQ-1）が貫通することを固定する
/// （badge/spinner と同型の単一 recipe 静的部品、children を持たないため
/// テキスト経路は対象外）。
#[test]
fn skeleton_attrs_and_class_are_escaped_for_all_payloads() {
    for payload in payloads::all() {
        // 属性値経路 b: 呼び出し側 attrs（data-testid 等）。
        let html = render(&skeleton(
            &SkeletonProps::default(),
            vec![("data-testid", payload)],
        ));
        assert_payload_is_escaped(payload, &html, "skeleton 呼び出し側 attrs コンテキスト");

        // 属性値経路 c: 呼び出し側 attrs の class（drop_class_attr により
        // 生ペイロードは出力されず、recipe 生成クラスへ完全に置き換わる）。
        let html = render(&skeleton(
            &SkeletonProps::default(),
            vec![("class", payload)],
        ));
        assert!(
            !html.contains(payload),
            "skeleton の class 属性に渡した生ペイロードが出力に残っている: \
             payload={payload:?}, html={html}"
        );
        assert_eq!(
            html.matches("class=\"").count(),
            1,
            "skeleton の class 属性が複数出現している: html={html}"
        );
        assert!(
            html.contains("fd-skeleton--"),
            "skeleton で recipe 生成クラスが失われている: html={html}"
        );
    }
}

/// イシュー #772: `separator::separator` の呼び出し側 `attrs`（`data-testid`
/// 等）と `class` の 2 箇所で既定エスケープ（REQ-1）が貫通することを固定する
/// （skeleton と同型の単一 recipe 静的部品、children を持たないためテキスト
/// 経路は対象外）。契約属性（`role`/`aria-orientation`/`data-orientation`）
/// の偽装除去そのものの回帰は `crates/pre-styled-ui/src/separator.rs` の
/// ユニットテストが担う（本ファイルは公開 API 経由の XSS 貫通のみを担当）。
///
/// イシュー #2053（shadcn/ui 突合）で追加した pre-styled-only
/// `group`/`label`（`kbd::group` #2230 と同型のパート）の `attrs`/
/// `children`・`data-scope`/`data-part` 偽装除去も同一関数で固定する。
#[test]
fn separator_attrs_and_class_are_escaped_for_all_payloads() {
    for payload in payloads::all() {
        // 属性値経路 b: 呼び出し側 attrs（data-testid 等）。
        let html = render(&separator(
            &SeparatorProps::default(),
            vec![("data-testid", payload)],
        ));
        assert_payload_is_escaped(payload, &html, "separator 呼び出し側 attrs コンテキスト");

        // 属性値経路 c: 呼び出し側 attrs の class（drop_class_attr により
        // 生ペイロードは出力されず、recipe 生成クラスへ完全に置き換わる）。
        let html = render(&separator(
            &SeparatorProps::default(),
            vec![("class", payload)],
        ));
        assert!(
            !html.contains(payload),
            "separator の class 属性に渡した生ペイロードが出力に残っている: \
             payload={payload:?}, html={html}"
        );
        assert_eq!(
            html.matches("class=\"").count(),
            1,
            "separator の class 属性が複数出現している: html={html}"
        );
        assert!(
            html.contains("fd-separator--"),
            "separator で recipe 生成クラスが失われている: html={html}"
        );

        // pre-styled-only group/label の attrs 経路（イシュー #2053）。
        let html = render(&separator_group(vec![("data-testid", payload)], vec![]));
        assert_payload_is_escaped(payload, &html, "separator::group attrs コンテキスト");
        let html = render(&separator_label(vec![("data-testid", payload)], vec![]));
        assert_payload_is_escaped(payload, &html, "separator::label attrs コンテキスト");

        // pre-styled-only group/label の children 経路（イシュー #2053）。
        let html = render(&separator_group(vec![], vec![text(payload)]));
        assert_payload_is_escaped(payload, &html, "separator::group children コンテキスト");
        let html = render(&separator_label(vec![], vec![text(payload)]));
        assert_payload_is_escaped(payload, &html, "separator::label children コンテキスト");

        // data-scope/data-part の偽装除去（headless Anatomy::part の
        // fail-closed 挙動、kbd::group と同型）。
        let html = render(&separator_group(
            vec![("data-scope", payload), ("data-part", payload)],
            vec![],
        ));
        assert!(
            !html.contains(payload),
            "separator::group の data-scope/data-part 偽装が出力に残っている: \
             payload={payload:?}, html={html}"
        );
    }
}

/// (11) progress 経路（circle 対応、イシュー #763/#1688）: styled `root` の
/// `aria_valuetext` 引数・呼び出し側 `attrs`・`class`、および headless
/// `Progress` の inherent メソッド（`circle`/`circle_track`/`circle_range`。
/// styled 層の独自ラッパーを持たず headless をそのまま呼ぶ契約、
/// `crates/pre-styled-ui/src/progress.rs` rustdoc 参照）の呼び出し側
/// `attrs` すべてで既定エスケープ（REQ-1）が貫通することを固定する。
/// イシュー #1688 で circle-range へ indeterminate 専用の固定弧 CSS
/// （`decl()` の固定リテラルのみで構成、外部入力は混入しない）を追加した
/// ことに伴い、indeterminate な `Progress`（`value = None`）でも同様に
/// `circle`/`circle_track`/`circle_range`/styled `range` の呼び出し側
/// `attrs` 経路が既定エスケープを貫通することを追加で固定する。
#[test]
fn progress_styled_root_and_headless_circle_parts_are_escaped_for_all_payloads() {
    use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::progress::Progress;
    use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::Orientation;
    use fandhe_frontend_pre_styled_ui::progress::{self, ProgressProps};

    let p = Progress::new(0.0, 100.0, Some(40.0), Orientation::Horizontal);
    let indeterminate_p = Progress::new(0.0, 100.0, None, Orientation::Horizontal);

    for payload in payloads::all() {
        // styled root の aria_valuetext 引数経路。
        let html = render(&progress::root(
            &p,
            &ProgressProps::default(),
            Some(payload),
            vec![],
            vec![],
        ));
        assert_payload_is_escaped(payload, &html, "progress::root aria_valuetext コンテキスト");

        // styled root の呼び出し側 attrs 経路。
        let html = render(&progress::root(
            &p,
            &ProgressProps::default(),
            None,
            vec![("data-testid", payload)],
            vec![],
        ));
        assert_payload_is_escaped(
            payload,
            &html,
            "progress::root 呼び出し側 attrs コンテキスト",
        );

        // styled root の class 属性経路（drop_class_attr により生ペイロードは
        // 出力されず、recipe 生成クラスへ完全に置き換わる）。
        let html = render(&progress::root(
            &p,
            &ProgressProps::default(),
            None,
            vec![("class", payload)],
            vec![],
        ));
        assert!(
            !html.contains(payload),
            "progress::root の class 属性に渡した生ペイロードが出力に残っている: \
             payload={payload:?}, html={html}"
        );
        assert_eq!(
            html.matches("class=\"").count(),
            1,
            "progress::root の class 属性が複数出現している: html={html}"
        );
        assert!(
            html.contains("fd-progress--"),
            "progress::root で recipe 生成クラスが失われている: html={html}"
        );

        // headless circle 系（styled 層の独自ラッパーなし）の呼び出し側 attrs 経路。
        let html = render(&p.circle(vec![("data-testid", payload)], vec![]));
        assert_payload_is_escaped(
            payload,
            &html,
            "Progress::circle 呼び出し側 attrs コンテキスト",
        );

        let html = render(&p.circle_track(vec![("data-testid", payload)], vec![]));
        assert_payload_is_escaped(
            payload,
            &html,
            "Progress::circle_track 呼び出し側 attrs コンテキスト",
        );

        let html = render(&p.circle_range(vec![("data-testid", payload)], vec![]));
        assert_payload_is_escaped(
            payload,
            &html,
            "Progress::circle_range 呼び出し側 attrs コンテキスト",
        );

        // styled range の呼び出し側 attrs 経路（percent style は headless
        // `Progress::percent` 由来の有限 f64 のみで、payload を含まない）。
        let html = render(&progress::range(&p, vec![("data-testid", payload)]));
        assert_payload_is_escaped(
            payload,
            &html,
            "progress::range 呼び出し側 attrs コンテキスト",
        );

        // styled range の呼び出し側 style 属性経路（drop_style_attr により
        // 生ペイロードは出力されず、--fandhe-progress-percent へ完全に
        // 置き換わる）。
        let html = render(&progress::range(&p, vec![("style", payload)]));
        assert!(
            !html.contains(payload),
            "progress::range の style 属性に渡した生ペイロードが出力に残っている: \
             payload={payload:?}, html={html}"
        );
        assert_eq!(
            html.matches("style=\"").count(),
            1,
            "progress::range の style 属性が複数出現している: html={html}"
        );
        assert!(
            html.contains("--fandhe-progress-percent: 40%"),
            "progress::range で percent style が失われている: html={html}"
        );

        // イシュー #1688: indeterminate（`value = None`）経路。circle 系
        // 3 parts の呼び出し側 attrs、および styled range の呼び出し側
        // attrs/style（indeterminate では style を一切出力しない headless
        // 契約、モジュール冒頭 rustdoc「indeterminate アニメーション」節
        // 参照）を通しても既定エスケープが貫通することを確認する。
        let html = render(&indeterminate_p.circle(vec![("data-testid", payload)], vec![]));
        assert_payload_is_escaped(
            payload,
            &html,
            "indeterminate Progress::circle 呼び出し側 attrs コンテキスト",
        );

        let html = render(&indeterminate_p.circle_track(vec![("data-testid", payload)], vec![]));
        assert_payload_is_escaped(
            payload,
            &html,
            "indeterminate Progress::circle_track 呼び出し側 attrs コンテキスト",
        );

        let html = render(&indeterminate_p.circle_range(vec![("data-testid", payload)], vec![]));
        assert_payload_is_escaped(
            payload,
            &html,
            "indeterminate Progress::circle_range 呼び出し側 attrs コンテキスト",
        );

        let html = render(&progress::range(
            &indeterminate_p,
            vec![("data-testid", payload)],
        ));
        assert_payload_is_escaped(
            payload,
            &html,
            "indeterminate progress::range 呼び出し側 attrs コンテキスト",
        );

        // indeterminate progress::range は drop_style_attr で呼び出し側
        // style を除去した上で、headless 契約どおり style 属性を一切
        // 出力しない（percent が存在しないため）。
        let html = render(&progress::range(&indeterminate_p, vec![("style", payload)]));
        assert!(
            !html.contains(payload),
            "indeterminate progress::range の style 属性に渡した生ペイロードが              出力に残っている: payload={payload:?}, html={html}"
        );
        assert!(
            !html.contains("style="),
            "indeterminate progress::range が style 属性を出力している: html={html}"
        );
    }
}

/// イシュー #770: Image/Icon の属性値経路（`src`/`alt`/`viewBox`/
/// `aria-label`/呼び出し側 `attrs`/`class`/SVG children 属性）が payload
/// 網羅で既定エスケープを経由することを固定する。
#[test]
fn image_and_icon_payload_paths_are_escaped_or_dropped() {
    for payload in payloads::all() {
        // Image: src/alt は属性値経路。
        let html = render(&image(&ImageProps::new(payload, payload), vec![]));
        assert_payload_is_escaped(payload, &html, "Image src/alt 属性値コンテキスト");

        // Image: 呼び出し側 attrs（data-testid）は素通りしつつエスケープされる。
        let html = render(&image(
            &ImageProps::new("/a.png", "alt"),
            vec![("data-testid", payload)],
        ));
        assert_payload_is_escaped(payload, &html, "Image 呼び出し側 attrs コンテキスト");

        // Image: 呼び出し側 class は drop_class_attr により出力に残らない。
        let html = render(&image(
            &ImageProps::new("/a.png", "alt"),
            vec![("class", payload)],
        ));
        assert!(
            !html.contains(payload),
            "image() の class 属性に渡した生ペイロードが出力に残っている: \
                 payload={payload:?}, html={html}"
        );
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(html.contains("fd-image--"));

        // Icon: viewBox/label は属性値経路。
        let props = IconProps {
            label: Some(payload),
            view_box: payload,
            ..IconProps::default()
        };
        let html = render(&icon(&props, vec![], vec![]));
        assert_payload_is_escaped(payload, &html, "Icon viewBox/aria-label 属性値コンテキスト");

        // Icon: 呼び出し側 attrs（data-testid）。
        let html = render(&icon(
            &IconProps::default(),
            vec![("data-testid", payload)],
            vec![],
        ));
        assert_payload_is_escaped(payload, &html, "Icon 呼び出し側 attrs コンテキスト");

        // Icon: 呼び出し側 class は drop_class_attr により出力に残らない。
        let html = render(&icon(
            &IconProps::default(),
            vec![("class", payload)],
            vec![],
        ));
        assert!(
            !html.contains(payload),
            "icon() の class 属性に渡した生ペイロードが出力に残っている: \
                 payload={payload:?}, html={html}"
        );
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(html.contains("fd-icon--"));

        // Icon: 呼び出し側が組み立てる SVG children（`path d` 属性）にも
        // 既定エスケープが適用される（本モジュールの外部リソース非参照
        // 契約の裏付け、`crate::icon` rustdoc 参照）。
        let node = icon(
            &IconProps::default(),
            vec![],
            vec![el("path", vec![("d", payload)], vec![])],
        );
        let html = render(&node);
        assert_payload_is_escaped(payload, &html, "Icon children path d 属性値コンテキスト");
    }
}

/// イシュー #770: Image の `src` に対する危険 URL スキームが属性ごと
/// 不出力になる（fail-closed）ことを固定する（`crates/core/src/url.rs`
/// の `is_safe_url`/`URL_ATTRS` 検証への依拠、本ファイル冒頭「URL 属性
/// 経路」と同型）。
#[test]
fn image_src_dangerous_url_schemes_are_rejected() {
    let dangerous_urls = [
        "javascript:alert(1)",
        "JaVaScRiPt:alert(1)",
        "data:text/html;base64,PHNjcmlwdD4=",
        "vbscript:msgbox(1)",
    ];

    for url in dangerous_urls {
        let html = render(&image(
            &ImageProps::new(url, "safe-alt"),
            vec![("data-testid", "sibling")],
        ));
        assert!(
            !html.contains("src="),
            "危険な URL スキームなのに src 属性が出力されている: url={url:?}, html={html}"
        );
        assert!(html.contains(r#"alt="safe-alt""#));
        assert!(html.contains(r#"data-testid="sibling""#));
        assert!(html.contains("fd-image--"));
    }
}

/// イシュー #770: Image の `src` に対する安全な URL は既定エスケープを
/// 経由してそのまま透過することを固定する。
#[test]
fn image_src_safe_urls_pass_through() {
    for url in ["/items/1.png", "https://example.com/a.png"] {
        let html = render(&image(&ImageProps::new(url, "alt"), vec![]));
        let expected = format!(r#"src="{}""#, escape_html(url));
        assert!(
            html.contains(&expected),
            "安全な URL が src 属性として透過していない: url={url:?}, html={html}"
        );
    }
}

/// イシュー #770: Icon 自身は外部リソース（`href`/`xlink:href`）を出力
/// しないが、children 経由で渡された危険スキームの `xlink:href` にも
/// core の `URL_ATTRS` 検証がそのまま適用される（属性ごと不出力）ことを
/// 固定する。
#[test]
fn icon_children_xlink_href_dangerous_scheme_is_rejected() {
    let node = icon(
        &IconProps::default(),
        vec![],
        vec![el(
            "use",
            vec![("xlink:href", "javascript:alert(1)")],
            vec![],
        )],
    );
    let html = render(&node);
    assert!(!html.contains("xlink:href"));
}

/// (12) status/empty_state 経路（イシュー #765）: 状態機械を要しない静的
/// styled 部品 2 種の全攻撃面（root children・呼び出し側 attrs・`class`
/// 属性・パーツ children）で既定エスケープ（REQ-1）が貫通することを固定
/// する（`card`/`checkbox_card` と同型）。
#[test]
fn status_empty_state_styled_parts_and_class_attr_are_escaped_for_all_payloads() {
    for payload in payloads::all() {
        // status::root の children（ラベルテキスト）経路。
        let html = render(&status::root(
            &StatusProps::default(),
            vec![],
            vec![text(payload)],
        ));
        assert_payload_is_escaped(payload, &html, "status::root children コンテキスト");

        // status::root の呼び出し側 attrs 経路。
        let html = render(&status::root(
            &StatusProps::default(),
            vec![("data-testid", payload)],
            vec![],
        ));
        assert_payload_is_escaped(payload, &html, "status::root 呼び出し側 attrs コンテキスト");

        // status::root の class 属性経路（drop_class_attr により生ペイロード
        // は出力されず、recipe 生成クラスへ完全に置き換わる）。
        let html = render(&status::root(
            &StatusProps::default(),
            vec![("class", payload)],
            vec![],
        ));
        assert!(
            !html.contains(payload),
            "status::root の class 属性に渡した生ペイロードが出力に残っている: \
             payload={payload:?}, html={html}"
        );
        assert_eq!(
            html.matches("class=\"").count(),
            1,
            "status::root の class 属性が複数出現している: html={html}"
        );
        assert!(
            html.contains("fd-status--"),
            "status::root で recipe 生成クラスが失われている: html={html}"
        );

        // status::indicator の呼び出し側 attrs 経路。
        let html = render(&status::indicator(vec![("data-testid", payload)]));
        assert_payload_is_escaped(
            payload,
            &html,
            "status::indicator 呼び出し側 attrs コンテキスト",
        );

        // empty_state::root の呼び出し側 attrs 経路。
        let html = render(&empty_state::root(
            &EmptyStateProps::default(),
            vec![("data-testid", payload)],
            vec![],
        ));
        assert_payload_is_escaped(
            payload,
            &html,
            "empty_state::root 呼び出し側 attrs コンテキスト",
        );

        // empty_state::root の class 属性経路。
        let html = render(&empty_state::root(
            &EmptyStateProps::default(),
            vec![("class", payload)],
            vec![],
        ));
        assert!(
            !html.contains(payload),
            "empty_state::root の class 属性に渡した生ペイロードが出力に残っている: \
             payload={payload:?}, html={html}"
        );
        assert_eq!(
            html.matches("class=\"").count(),
            1,
            "empty_state::root の class 属性が複数出現している: html={html}"
        );
        assert!(
            html.contains("fd-empty-state--"),
            "empty_state::root で recipe 生成クラスが失われている: html={html}"
        );

        // empty_state::title / description の children 経路。
        let html = render(&empty_state::title(vec![], vec![text(payload)]));
        assert_payload_is_escaped(payload, &html, "empty_state::title children コンテキスト");

        let html = render(&empty_state::description(vec![], vec![text(payload)]));
        assert_payload_is_escaped(
            payload,
            &html,
            "empty_state::description children コンテキスト",
        );

        // イシュー #2047: empty_state::root の variant（Outline）class 経路。
        let html = render(&empty_state::root(
            &EmptyStateProps {
                size: fandhe_frontend_pre_styled_ui::Size::Md,
                variant: EmptyStateVariant::Outline,
            },
            vec![("class", payload)],
            vec![],
        ));
        assert!(
            !html.contains(payload),
            "empty_state::root(Outline) の class 属性に渡した生ペイロードが出力に残っている: \
             payload={payload:?}, html={html}"
        );
        assert_eq!(
            html.matches("class=\"").count(),
            1,
            "empty_state::root(Outline) の class 属性が複数出現している: html={html}"
        );
        assert!(
            html.contains("fd-empty-state--variant-outline"),
            "empty_state::root(Outline) で variant class が失われている: html={html}"
        );

        // イシュー #2047: empty_state::indicator_with(Boxed) の attrs 経路。
        let html = render(&empty_state::indicator_with(
            EmptyStateIndicatorVariant::Boxed,
            vec![("data-testid", payload)],
            vec![],
        ));
        assert_payload_is_escaped(
            payload,
            &html,
            "empty_state::indicator_with(Boxed) 呼び出し側 attrs コンテキスト",
        );

        // イシュー #2047: empty_state::indicator_with(Boxed) の class 属性経路。
        let html = render(&empty_state::indicator_with(
            EmptyStateIndicatorVariant::Boxed,
            vec![("class", payload)],
            vec![],
        ));
        assert!(
            !html.contains(payload),
            "empty_state::indicator_with(Boxed) の class 属性に渡した生ペイロードが出力に残っている: \
             payload={payload:?}, html={html}"
        );
        assert_eq!(
            html.matches("class=\"").count(),
            1,
            "empty_state::indicator_with(Boxed) の class 属性が複数出現している: html={html}"
        );
        assert!(
            html.contains("fd-empty-state--indicator-boxed"),
            "empty_state::indicator_with(Boxed) で variant class が失われている: html={html}"
        );
    }
}

/// styled Clipboard（イシュー #773）の XSS 回帰。コピー対象値
/// （`data-value`/`input` の `value`）はパスワード等の機微情報を含みうる
/// ため、属性破りペイロードでも実タグ・属性破りが起きないことを固定する
/// （`crates/headless-ui/tests/xss_escape.rs::clipboard_root_data_value_and_input_value_are_escaped_for_all_payloads`
/// の styled 経由版）。
#[test]
fn clipboard_styled_root_data_value_and_value_text_are_escaped_for_all_payloads() {
    for payload in payloads::all() {
        let html = render(&clipboard::root(payload, false, vec![], vec![]));
        assert_payload_is_escaped(
            payload,
            &html,
            "clipboard::root の data-value 属性値コンテキスト",
        );

        let html = render(&clipboard::input(payload, false, vec![]));
        assert_payload_is_escaped(
            payload,
            &html,
            "clipboard::input の value 属性値コンテキスト",
        );

        let html = render(&clipboard::value_text(vec![], vec![text(payload)]));
        assert_payload_is_escaped(
            payload,
            &html,
            "clipboard::value_text children コンテキスト",
        );
    }
}

/// (13) highlight 経路（イシュー #775）: 本文（テキスト）・クエリ・呼び出し側
/// `attrs`・`class` の 4 箇所で既定エスケープ（REQ-1）が貫通することを固定
/// する。`query` はユーザー入力由来の一致キーワードであり、一致・不一致の
/// いずれの場合も `query` の生文字列がそのまま HTML へ出力される経路がない
/// ことが要点（`crates/pre-styled-ui/src/highlight.rs` モジュール冒頭
/// rustdoc「一致判定は決定的な文字列検索のみ」節参照）。
#[test]
fn highlight_text_query_and_attrs_are_escaped_for_all_payloads() {
    for payload in payloads::all() {
        // 本文経路: text にペイロードを渡す（query は本文に現れない語句のため
        // 不一致、mark 化されない状態でもエスケープが貫通することを固定）。
        let html = render(&highlight(
            &HighlightProps {
                query: &["nonexistent-query-term"],
                ..HighlightProps::default()
            },
            vec![],
            payload,
        ));
        assert_payload_is_escaped(payload, &html, "highlight 本文（text）コンテキスト");

        // クエリ経路: ペイロードを query に渡す。本文にペイロードと同じ
        // 文字列が含まれる場合は一致し mark 化されるが、その場合も mark 内
        // テキストは text() 経由でエスケープされる。
        let html = render(&highlight(
            &HighlightProps {
                query: &[payload],
                ..HighlightProps::default()
            },
            vec![],
            payload,
        ));
        assert_payload_is_escaped(payload, &html, "highlight クエリ（query）一致コンテキスト");

        // クエリ経路（不一致）: 本文に query が現れない場合、query の生文字列
        // がどの経路からも HTML へ出力されないことを固定する。
        let html = render(&highlight(
            &HighlightProps {
                query: &[payload],
                ..HighlightProps::default()
            },
            vec![],
            "The quick brown fox",
        ));
        assert!(
            !html.contains(payload),
            "highlight で不一致 query の生ペイロードが出力に残っている: \
             payload={payload:?}, html={html}"
        );

        // 属性値経路 b: 呼び出し側 attrs（data-testid 等）。
        let html = render(&highlight(
            &HighlightProps::default(),
            vec![("data-testid", payload)],
            "hello world",
        ));
        assert_payload_is_escaped(payload, &html, "highlight 呼び出し側 attrs コンテキスト");

        // 属性値経路 c: 呼び出し側 attrs の class（drop_class_attr により
        // 生ペイロードは出力されない。root には class を出力しないため、
        // 一致なし（"hello world" は query と不一致）の本ケースでは
        // `<mark>` も生成されず class 属性自体が出力されない
        // （イシュー #1435 で variant/palette 軸を持つ `<mark>` 生成 class は
        // 一致箇所にのみ付与されるようになった。root への漏出がないことは
        // `caller_attrs_class_and_data_scope_part_are_dropped`〔インライン
        // テスト〕が別途固定する）。
        let html = render(&highlight(
            &HighlightProps::default(),
            vec![("class", payload)],
            "hello world",
        ));
        assert!(
            !html.contains(payload),
            "highlight の class 属性に渡した生ペイロードが出力に残っている: \
             payload={payload:?}, html={html}"
        );
        assert!(
            !html.contains("class=\""),
            "一致なしの本ケースでは <mark> が生成されず class 属性が出力されないはずだが出力されている: \
             html={html}"
        );
    }
}

/// styled VisuallyHidden（イシュー #776）: children テキスト経路 + 呼び出し側
/// `attrs`（`class` を含む）経路を横断してエスケープ貫通を固定する。
#[test]
fn visually_hidden_children_and_attrs_are_escaped_for_all_payloads() {
    use fandhe_frontend_pre_styled_ui::visually_hidden;

    for payload in payloads::all() {
        let html = render(&visually_hidden::root(vec![], vec![text(payload)]));
        assert_payload_is_escaped(
            payload,
            &html,
            "visually_hidden::root children コンテキスト",
        );

        let html = render(&visually_hidden::root(
            vec![("data-testid", payload)],
            vec![],
        ));
        assert_payload_is_escaped(
            payload,
            &html,
            "visually_hidden::root 呼び出し側 attrs コンテキスト",
        );

        // 本部品は variant 軸を持たず class 属性自体を出力しないため
        // （`crates/pre-styled-ui/src/visually_hidden.rs` rustdoc 参照）、
        // 呼び出し側 `class` は出力から完全に消えることを確認する。
        let html = render(&visually_hidden::root(vec![("class", payload)], vec![]));
        assert!(
            !html.contains(payload),
            "visually_hidden::root の class 属性に渡した生ペイロードが出力に残っている: \
             payload={payload:?}, html={html}"
        );
        assert_eq!(
            html.matches("class=\"").count(),
            0,
            "visually_hidden::root は class 属性自体を出力しない契約: html={html}"
        );
    }
}

/// styled SkipNav（イシュー #776）: [`skip_nav::link`] の `id`（`href` 属性へ
/// 合成）・[`skip_nav::content`] の `id`（`id` 属性へ合成）・children・
/// 呼び出し側 `attrs`（`class` を含む）経路を横断してエスケープ貫通を固定
/// する。
#[test]
fn skip_nav_id_children_and_attrs_are_escaped_for_all_payloads() {
    use fandhe_frontend_pre_styled_ui::skip_nav;

    for payload in payloads::all() {
        let link_node = skip_nav::link(payload, vec![], vec![text(payload)]);
        let html = render(&link_node);
        assert_payload_is_escaped(
            payload,
            &html,
            "skip_nav::link の id(href 属性)/children コンテキスト",
        );

        let content_node = skip_nav::content(payload, vec![], vec![text(payload)]);
        let html = render(&content_node);
        assert_payload_is_escaped(
            payload,
            &html,
            "skip_nav::content の id(id 属性)/children コンテキスト",
        );

        let html = render(&skip_nav::link(
            skip_nav::DEFAULT_ID,
            vec![("class", payload)],
            vec![],
        ));
        assert!(
            !html.contains(payload),
            "skip_nav::link の class 属性に渡した生ペイロードが出力に残っている: \
             payload={payload:?}, html={html}"
        );
    }
}

/// (14) タイポグラフィ静的部品 8 種（イシュー #771: Heading / Text / Em /
/// Mark / Blockquote / List。イシュー #995 で Quote / Strong を追加）:
/// children テキスト経路・呼び出し側 attrs 経路・`class` 除去経路のすべてで
/// 既定エスケープ（REQ-1）が貫通することを固定する。
#[test]
fn typography_static_parts_are_escaped_for_all_payloads() {
    for payload in payloads::all() {
        // heading::heading: children テキスト経路 + 呼び出し側 attrs 経路 + class 除去経路。
        let html = render(&heading(
            HeadingLevel::default(),
            &HeadingProps::default(),
            vec![],
            vec![text(payload)],
        ));
        assert_payload_is_escaped(payload, &html, "heading::heading children コンテキスト");

        let html = render(&heading(
            HeadingLevel::default(),
            &HeadingProps::default(),
            vec![("data-testid", payload)],
            vec![],
        ));
        assert_payload_is_escaped(
            payload,
            &html,
            "heading::heading 呼び出し側 attrs コンテキスト",
        );

        let html = render(&heading(
            HeadingLevel::default(),
            &HeadingProps::default(),
            vec![("class", payload)],
            vec![],
        ));
        assert!(
            !html.contains(payload),
            "heading::heading の class 属性に渡した生ペイロードが出力に残っている: \
             payload={payload:?}, html={html}"
        );
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(html.contains("fd-heading--"));

        // text::text（core::text と同名だが別モジュールパス、モジュール rustdoc 参照）。
        let html = render(&styled_text(
            &TextProps::default(),
            vec![],
            vec![text(payload)],
        ));
        assert_payload_is_escaped(payload, &html, "text::text children コンテキスト");

        let html = render(&styled_text(
            &TextProps::default(),
            vec![("data-testid", payload)],
            vec![],
        ));
        assert_payload_is_escaped(payload, &html, "text::text 呼び出し側 attrs コンテキスト");

        // em::em: variant を持たないため class 出力なし。children・attrs 経路のみ。
        let html = render(&em(vec![], vec![text(payload)]));
        assert_payload_is_escaped(payload, &html, "em::em children コンテキスト");

        let html = render(&em(vec![("data-testid", payload)], vec![]));
        assert_payload_is_escaped(payload, &html, "em::em 呼び出し側 attrs コンテキスト");

        // mark::mark: children テキスト経路 + 呼び出し側 attrs 経路 + class 除去経路。
        let html = render(&mark(&MarkProps::default(), vec![], vec![text(payload)]));
        assert_payload_is_escaped(payload, &html, "mark::mark children コンテキスト");

        let html = render(&mark(
            &MarkProps::default(),
            vec![("data-testid", payload)],
            vec![],
        ));
        assert_payload_is_escaped(payload, &html, "mark::mark 呼び出し側 attrs コンテキスト");

        let html = render(&mark(
            &MarkProps::default(),
            vec![("class", payload)],
            vec![],
        ));
        assert!(!html.contains(payload));
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(html.contains("fd-mark--"));

        // blockquote: root/content/caption の 3 パーツ、cite 属性のような
        // 呼び出し側 attrs 経路を含む。
        let html = render(&blockquote::content(
            vec![("cite", payload)],
            vec![text(payload)],
        ));
        assert_payload_is_escaped(
            payload,
            &html,
            "blockquote::content children/cite コンテキスト",
        );

        let html = render(&blockquote::caption(vec![], vec![text(payload)]));
        assert_payload_is_escaped(payload, &html, "blockquote::caption children コンテキスト");

        let html = render(&blockquote::root(
            BlockquoteVariant::default(),
            fandhe_frontend_pre_styled_ui::ColorPalette::default(),
            vec![("class", payload)],
            vec![],
        ));
        assert!(!html.contains(payload));
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(html.contains("fd-blockquote--"));

        // list: root/item/indicator の 3 パーツ。indicator は常時
        // aria-hidden="true" のため呼び出し側偽装が無視されることも確認する。
        let html = render(&list::item(vec![], vec![text(payload)]));
        assert_payload_is_escaped(payload, &html, "list::item children コンテキスト");

        let html = render(&list::root(
            ListType::default(),
            ListVariant::default(),
            vec![("class", payload)],
            vec![],
        ));
        assert!(!html.contains(payload));
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(html.contains("fd-list--"));

        let html = render(&list::indicator(vec![("aria-hidden", payload)], vec![]));
        assert!(
            !html.contains(payload) || payload == "true",
            "list::indicator の aria-hidden 偽装が出力に残っている: payload={payload:?}, html={html}"
        );
        assert_eq!(html.matches("aria-hidden=").count(), 1);
        assert!(html.contains(r#"aria-hidden="true""#));

        // quote::quote: variant を持たないため class 出力なし。children・attrs 経路のみ。
        let html = render(&quote(vec![], vec![text(payload)]));
        assert_payload_is_escaped(payload, &html, "quote::quote children コンテキスト");

        let html = render(&quote(vec![("data-testid", payload)], vec![]));
        assert_payload_is_escaped(payload, &html, "quote::quote 呼び出し側 attrs コンテキスト");

        // strong::strong: variant を持たないため class 出力なし。children・attrs 経路のみ。
        let html = render(&strong(vec![], vec![text(payload)]));
        assert_payload_is_escaped(payload, &html, "strong::strong children コンテキスト");

        let html = render(&strong(vec![("data-testid", payload)], vec![]));
        assert_payload_is_escaped(
            payload,
            &html,
            "strong::strong 呼び出し側 attrs コンテキスト",
        );
    }
}

/// (15) Table 経路（イシュー #767）: styled `root` の呼び出し側 `attrs`・
/// `class`、および `cell`/`column_header`/`caption` のセル値・見出し
/// children（受け入れ条件「セル値・見出しにスクリプト断片」の対象）の各所
/// すべてで既定エスケープ（REQ-1）が貫通することを固定する。
#[test]
fn table_styled_root_and_parts_are_escaped_for_all_payloads() {
    use fandhe_frontend_pre_styled_ui::table::{self, TableProps};

    for payload in payloads::all() {
        // styled root の呼び出し側 attrs 経路。
        let html = render(&table::root(
            TableProps::default(),
            vec![("data-testid", payload)],
            vec![],
        ));
        assert_payload_is_escaped(payload, &html, "table::root 呼び出し側 attrs コンテキスト");

        // styled root の class 属性経路（drop_class_attr により生ペイロードは
        // 出力されず、recipe 生成クラスへ完全に置き換わる）。
        let html = render(&table::root(
            TableProps::default(),
            vec![("class", payload)],
            vec![],
        ));
        assert!(
            !html.contains(payload),
            "table::root の class 属性に渡した生ペイロードが出力に残っている: \
             payload={payload:?}, html={html}"
        );
        assert_eq!(
            html.matches("class=\"").count(),
            1,
            "table::root の class 属性が複数出現している: html={html}"
        );
        assert!(
            html.contains("fd-table--"),
            "table::root で recipe 生成クラスが失われている: html={html}"
        );

        // cell のセル値 children 経路。
        let html = render(&table::cell(vec![], vec![text(payload)]));
        assert_payload_is_escaped(payload, &html, "table::cell children コンテキスト");

        // column_header の見出し children 経路。
        let html = render(&table::column_header(vec![], vec![text(payload)]));
        assert_payload_is_escaped(payload, &html, "table::column_header children コンテキスト");

        // caption の children 経路。
        let html = render(&table::caption(vec![], vec![text(payload)]));
        assert_payload_is_escaped(payload, &html, "table::caption children コンテキスト");

        // column_header の scope 属性偽装経路（値そのものはエスケープ対象では
        // ないが、drop_reserved により呼び出し側の値が握りつぶされ固定値
        // `"col"` に置き換わることを確認する）。
        let html = render(&table::column_header(vec![("scope", payload)], vec![]));
        assert!(
            !html.contains(payload),
            "table::column_header の scope 属性に渡した生ペイロードが出力に残っている: \
             payload={payload:?}, html={html}"
        );
        assert!(html.contains(r#"scope="col""#));

        // イシュー #1572: scroll_area の呼び出し側 attrs 経路（header/body と
        // 同型で class を含む attrs をそのまま連結するため、既定エスケープが
        // 貫通することを固定する）。
        let html = render(&table::scroll_area(vec![("data-testid", payload)], vec![]));
        assert_payload_is_escaped(
            payload,
            &html,
            "table::scroll_area 呼び出し側 attrs コンテキスト",
        );

        // scroll_area の children 経路。
        let html = render(&table::scroll_area(vec![], vec![text(payload)]));
        assert_payload_is_escaped(payload, &html, "table::scroll_area children コンテキスト");

        // イシュー #2052: row の data-selected 属性値経路（呼び出し側が
        // 付与する共有語彙、table.rs モジュール doc「`data-selected` 行
        // 状態」節参照）。
        let html = render(&table::row(vec![("data-selected", payload)], vec![]));
        assert_payload_is_escaped(payload, &html, "table::row data-selected 属性コンテキスト");

        // イシュー #2052: cell の data-align 属性値経路（table.rs モジュール
        // doc「`data-align` セル整列」節参照）。
        let html = render(&table::cell(vec![("data-align", payload)], vec![]));
        assert_payload_is_escaped(payload, &html, "table::cell data-align 属性コンテキスト");

        // イシュー #2052: column_header の aria-sort 属性値経路（生産は
        // headless data-table〔#2124〕の責務、table.rs は通過させるのみ、
        // table.rs モジュール doc「スコープ外」節参照）。
        let html = render(&table::column_header(vec![("aria-sort", payload)], vec![]));
        assert_payload_is_escaped(
            payload,
            &html,
            "table::column_header aria-sort 属性コンテキスト",
        );
    }
}

/// (16) DataList 経路（イシュー #767）: styled `root` の呼び出し側 `attrs`・
/// `class`、および `item_label`/`item_value` のラベル・値 children の各所
/// すべてで既定エスケープ（REQ-1）が貫通することを固定する。
#[test]
fn data_list_styled_root_and_parts_are_escaped_for_all_payloads() {
    use fandhe_frontend_pre_styled_ui::data_list::{self, DataListProps};

    for payload in payloads::all() {
        // styled root の呼び出し側 attrs 経路。
        let html = render(&data_list::root(
            DataListProps::default(),
            vec![("data-testid", payload)],
            vec![],
        ));
        assert_payload_is_escaped(
            payload,
            &html,
            "data_list::root 呼び出し側 attrs コンテキスト",
        );

        // styled root の class 属性経路（drop_class_attr により生ペイロードは
        // 出力されず、recipe 生成クラスへ完全に置き換わる）。
        let html = render(&data_list::root(
            DataListProps::default(),
            vec![("class", payload)],
            vec![],
        ));
        assert!(
            !html.contains(payload),
            "data_list::root の class 属性に渡した生ペイロードが出力に残っている: \
             payload={payload:?}, html={html}"
        );
        assert_eq!(
            html.matches("class=\"").count(),
            1,
            "data_list::root の class 属性が複数出現している: html={html}"
        );
        assert!(
            html.contains("fd-data-list--"),
            "data_list::root で recipe 生成クラスが失われている: html={html}"
        );

        // item_label のラベル children 経路。
        let html = render(&data_list::item_label(vec![], vec![text(payload)]));
        assert_payload_is_escaped(
            payload,
            &html,
            "data_list::item_label children コンテキスト",
        );

        // item_value の値 children 経路。
        let html = render(&data_list::item_value(vec![], vec![text(payload)]));
        assert_payload_is_escaped(
            payload,
            &html,
            "data_list::item_value children コンテキスト",
        );
    }
}

/// イシュー #768: Tag / Kbd / Code の styled 公開 API 経由の XSS 回帰。
///
/// 対象の入力面:
/// 1. テキスト経路: `tag::root`/`tag::label`/`kbd::kbd`/`code::code` の
///    children。
/// 2. 属性値経路: `tag::close_trigger` の `action` 引数（`data-action`
///    属性値として出力される）・呼び出し側 `attrs`。
/// 3. class 破棄経路: `tag::root`/`kbd::kbd`/`code::code` へ `class` を渡し、
///    recipe 生成クラスへの完全置換を確認する（`code` はイシュー #1432、
///    `kbd` はイシュー #1436 でそれぞれ variant/size/colorPalette 軸を持つ
///    単一 recipe 部品へ変わったため、`tag::root` と同様に recipe 生成
///    クラスへの完全置換〔class 属性は 1 個のみ・payload 不残留〕を
///    確認する）。
#[test]
fn tag_kbd_code_styled_are_escaped_for_all_payloads() {
    use fandhe_frontend_pre_styled_ui::code::{code, CodeProps};
    use fandhe_frontend_pre_styled_ui::kbd::{group as kbd_group, kbd, KbdProps};
    use fandhe_frontend_pre_styled_ui::tag::{self, TagProps};

    for payload in payloads::all() {
        // (1) テキスト経路。
        let html = render(&tag::root(
            &TagProps::default(),
            vec![],
            vec![text(payload)],
        ));
        assert_payload_is_escaped(payload, &html, "tag::root children コンテキスト");

        let html = render(&tag::label(vec![], vec![text(payload)]));
        assert_payload_is_escaped(payload, &html, "tag::label children コンテキスト");

        let html = render(&kbd(&KbdProps::default(), vec![], vec![text(payload)]));
        assert_payload_is_escaped(payload, &html, "kbd children コンテキスト");

        // (1) テキスト経路: kbd::group（イシュー #2048、shadcn/ui
        // KbdGroup 相当の pre-styled-only パート）の children。
        let html = render(&kbd_group(vec![], vec![text(payload)]));
        assert_payload_is_escaped(payload, &html, "kbd::group children コンテキスト");

        let html = render(&code(&CodeProps::default(), vec![], vec![text(payload)]));
        assert_payload_is_escaped(payload, &html, "code children コンテキスト");

        // (2) 属性値経路: close_trigger の action（data-action 属性値）。
        let html = render(&tag::close_trigger(Some(payload), vec![], vec![]));
        assert_payload_is_escaped(
            payload,
            &html,
            "tag::close_trigger data-action 属性値コンテキスト",
        );

        // (2) 属性値経路: 呼び出し側 attrs。
        let html = render(&tag::root(
            &TagProps::default(),
            vec![("data-testid", payload)],
            vec![],
        ));
        assert_payload_is_escaped(payload, &html, "tag::root 呼び出し側 attrs コンテキスト");

        let html = render(&tag::close_trigger(
            None,
            vec![("aria-label", payload)],
            vec![],
        ));
        assert_payload_is_escaped(
            payload,
            &html,
            "tag::close_trigger 呼び出し側 attrs コンテキスト",
        );

        // (3) class 破棄経路: tag::root は recipe 生成クラスへ完全置換。
        let html = render(&tag::root(
            &TagProps::default(),
            vec![("class", payload)],
            vec![],
        ));
        assert!(
            !html.contains(payload),
            "tag::root の class 属性に渡した生ペイロードが出力に残っている: \
             payload={payload:?}, html={html}"
        );
        assert_eq!(
            html.matches("class=\"").count(),
            1,
            "tag::root の class 属性が複数出現している: html={html}"
        );
        assert!(
            html.contains("fd-tag--"),
            "tag::root で recipe 生成クラスが失われている: html={html}"
        );

        // (3) class 破棄経路: kbd はイシュー #1436 で variant/size/
        // colorPalette 軸を持つ単一 recipe 部品へ変わったため、tag::root/
        // code と同様に recipe 生成クラスへの完全置換（class 属性は 1 個
        // のみ・payload 不残留）を確認する。
        let html = render(&kbd(&KbdProps::default(), vec![("class", payload)], vec![]));
        assert!(
            !html.contains(payload),
            "kbd の class 属性に渡した生ペイロードが出力に残っている: \
             payload={payload:?}, html={html}"
        );
        assert_eq!(
            html.matches("class=\"").count(),
            1,
            "kbd の class 属性が複数出現している: html={html}"
        );
        assert!(
            html.contains("fd-kbd--"),
            "kbd で recipe 生成クラスが失われている: html={html}"
        );

        // (2)(3) 属性値経路 + data-scope/data-part 偽装除去: kbd::group
        // （イシュー #2048）は recipe 由来クラスを持たないため呼び出し側
        // `class`/`title` をそのまま通過させる契約（`crate::avatar::group`
        // と同型）。エスケープ済みで出力されることのみを確認する。
        let html = render(&kbd_group(vec![("class", payload)], vec![]));
        assert_payload_is_escaped(payload, &html, "kbd::group class 属性値コンテキスト");

        let html = render(&kbd_group(vec![("title", payload)], vec![]));
        assert_payload_is_escaped(payload, &html, "kbd::group title 属性値コンテキスト");

        let html = render(&kbd_group(
            vec![("data-scope", payload), ("data-part", payload)],
            vec![],
        ));
        assert!(
            !html.contains(payload),
            "kbd::group の data-scope/data-part 偽装ペイロードが出力に残っている: \
             payload={payload:?}, html={html}"
        );
        assert!(
            html.contains(r#"data-scope="kbd""#) && html.contains(r#"data-part="group""#),
            "kbd::group の anatomy 属性が偽装で上書きされている: html={html}"
        );

        // (3) class 破棄経路: code はイシュー #1432 で variant/size/
        // colorPalette 軸を持つ単一 recipe 部品へ変わったため、tag::root と
        // 同様に recipe 生成クラスへの完全置換（class 属性は 1 個のみ・
        // payload 不残留）を確認する。
        let html = render(&code(
            &CodeProps::default(),
            vec![("class", payload)],
            vec![],
        ));
        assert!(
            !html.contains(payload),
            "code の class 属性に渡した生ペイロードが出力に残っている: \
             payload={payload:?}, html={html}"
        );
        assert_eq!(
            html.matches("class=\"").count(),
            1,
            "code の class 属性が複数出現している: html={html}"
        );
        assert!(
            html.contains("fd-code--"),
            "code で recipe 生成クラスが失われている: html={html}"
        );
    }
}

/// JsonTreeView（イシュー #829）: `fandhe_frontend_pre_styled_ui::json_tree_view`
/// の再エクスポート経由（headless-ui を直接使わない）で `render_json` を呼び、
/// オブジェクトキー・文字列値の children テキスト経路へペイロードを注入しても
/// エスケープが貫通することを固定する。styled 層は薄い再エクスポートであり
/// 独自のエスケープ処理を持たないため、本テストは `crates/headless-ui/tests/xss_escape.rs`
/// の対応テストと同じ保証を styled 経路越しに固定する契約検証である。
#[test]
fn json_tree_view_styled_key_and_string_value_are_escaped_for_all_payloads() {
    use fandhe_frontend_pre_styled_ui::json_tree_view::{render_json, JsonValue, TreeView};

    for payload in payloads::all() {
        let by_key = JsonValue::Object(vec![(payload.to_string(), JsonValue::Null)]);
        let html = render(&render_json(&TreeView::default(), &by_key));
        assert_payload_is_escaped(
            payload,
            &html,
            "json_tree_view::render_json（styled 再エクスポート）のオブジェクトキー児テキストコンテキスト",
        );

        let by_string_value = JsonValue::Object(vec![(
            "k".to_string(),
            JsonValue::String(payload.to_string()),
        )]);
        let html = render(&render_json(&TreeView::default(), &by_string_value));
        assert_payload_is_escaped(
            payload,
            &html,
            "json_tree_view::render_json（styled 再エクスポート）の文字列値児テキストコンテキスト",
        );
    }
}

/// styled FloatingPanel（イシュー #827）の XSS 回帰。[`floating_panel`] は
/// headless 層をそのまま再エクスポートする薄い委譲層（`pub use ...::*`）で
/// あるため、
/// `crates/headless-ui/tests/xss_escape.rs::floating_panel_controls_id_labelledby_and_title_children_are_escaped_for_all_payloads`
/// と同じ観点を `fandhe-frontend-pre-styled-ui` の公開 API 経由でも固定する
/// （styled 層のみに依存する利用者が同じ保証を得られることの確認）。
#[test]
fn floating_panel_styled_controls_id_labelledby_and_title_children_are_escaped_for_all_payloads() {
    for payload in payloads::all() {
        // 属性値経路: trigger の controls。
        let html = render(&floating_panel::trigger(
            OpenState::Closed,
            false,
            Some(payload),
            vec![],
            vec![],
        ));
        assert_payload_is_escaped(
            payload,
            &html,
            "floating_panel::trigger controls コンテキスト",
        );

        // 属性値経路: content の id/labelledby。
        let html = render(&floating_panel::content(
            OpenState::Open,
            Stage::Default,
            Some(payload),
            Some(payload),
            vec![],
            vec![],
        ));
        assert_payload_is_escaped(
            payload,
            &html,
            "floating_panel::content id/labelledby コンテキスト",
        );

        // 属性値経路: root の呼び出し側 attrs（data-testid）。
        let html = render(&floating_panel::root(
            OpenState::Closed,
            Stage::Default,
            vec![("data-testid", payload)],
            vec![],
        ));
        assert_payload_is_escaped(
            payload,
            &html,
            "floating_panel::root 呼び出し側 attrs コンテキスト",
        );

        // テキスト経路: title の children。
        let html = render(&floating_panel::title(None, vec![], vec![text(payload)]));
        assert_payload_is_escaped(
            payload,
            &html,
            "floating_panel::title children コンテキスト",
        );

        // 属性値経路: positioner の style（position_style() 出力の透過経路）。
        let html = render(&floating_panel::positioner(
            OpenState::Open,
            Stage::Default,
            vec![("style", payload)],
            vec![],
        ));
        assert_payload_is_escaped(
            payload,
            &html,
            "floating_panel::positioner style コンテキスト",
        );
    }
}

/// styled DownloadTrigger（イシュー #828）の XSS 回帰: `download_trigger::root`
/// の `href`（URL 属性経路、危険スキームは fail-closed で href 自体が出力
/// されない）・`file_name`（`download` 属性値経路）・children テキスト・
/// 呼び出し側 `attrs` の `class`（`drop_class_attr` により生ペイロードが
/// 動的クラス名合成へ混入しない）の各経路でエスケープが貫通することを
/// 固定する。
#[test]
fn download_trigger_styled_root_href_file_name_children_and_class_are_escaped() {
    for payload in payloads::all() {
        let props = DownloadTriggerProps::default();

        let html = render(&download_trigger::root(
            &props,
            payload,
            None,
            vec![],
            vec![text(payload)],
        ));
        if html.contains("href=") {
            assert!(
                !html.contains(payload),
                "download_trigger::root の href コンテキストで生ペイロードが残っている: \
                 payload={payload:?}, html={html}"
            );
        }
        // children テキストは常時エスケープされる。
        assert!(
            !html.contains(&format!(">{payload}<")),
            "download_trigger::root の children コンテキストで生ペイロードが残っている: \
             payload={payload:?}, html={html}"
        );

        let html = render(&download_trigger::root(
            &props,
            "/assets/report.pdf",
            Some(payload),
            vec![],
            vec![],
        ));
        assert!(
            !html.contains(&format!("download=\"{payload}\"")),
            "download_trigger::root の download（file_name）コンテキストで \
             生ペイロードが残っている: payload={payload:?}, html={html}"
        );

        let html = render(&download_trigger::root(
            &props,
            "/assets/report.pdf",
            None,
            vec![("class", payload)],
            vec![],
        ));
        assert!(
            !html.contains(payload),
            "download_trigger::root の class 属性に渡した生ペイロードが出力に残っている: \
             payload={payload:?}, html={html}"
        );
        assert_eq!(
            html.matches("class=\"").count(),
            1,
            "download_trigger::root の class 属性が複数出現している: html={html}"
        );
        assert!(
            html.contains("fd-download-trigger--"),
            "download_trigger::root で recipe 生成クラスが失われている: html={html}"
        );
    }
}

/// styled ScrollArea（イシュー #825、#2054 で `root`/`content` の attrs
/// 経路を追加）の headless 再エクスポート経路（attrs breakout・children
/// `<script>` ペイロード）がエスケープされることを固定する。`root` の
/// attrs 経路は #2054 の RTL Examples（`(\"dir\", \"rtl\")` 透過）が使う
/// 経路と同一であり、`content` の attrs 経路は横スクロール demo が
/// `style` 属性を渡す経路と同一である（`crate::scroll_area` モジュール
/// doc「shadcn/ui 突合（イシュー #2054）」節参照）。
#[test]
fn scroll_area_attrs_and_children_payloads_are_escaped_for_all_payloads() {
    for payload in payloads::all() {
        let html = render(&scroll_area::viewport(
            vec![("data-testid", payload)],
            vec![],
        ));
        assert_payload_is_escaped(
            payload,
            &html,
            "scroll_area::viewport の attrs コンテキスト",
        );

        let html = render(&scroll_area::content(vec![], vec![text(payload)]));
        assert_payload_is_escaped(
            payload,
            &html,
            "scroll_area::content の children コンテキスト",
        );

        let html = render(&scroll_area::content(
            vec![("data-testid", payload)],
            vec![],
        ));
        assert_payload_is_escaped(payload, &html, "scroll_area::content の attrs コンテキスト");

        let html = render(&scroll_area::root(vec![("data-testid", payload)], vec![]));
        assert_payload_is_escaped(payload, &html, "scroll_area::root の attrs コンテキスト");
    }
}

/// styled DateInput（イシュー #834）: styled `root` の呼び出し側 `attrs`・
/// `class`、および headless-ui から選択的再エクスポートした `label` の
/// children・`hidden_input` の `name`・`segment` の `attrs` の 5 箇所すべてで
/// 既定エスケープ（REQ-1）が貫通することを固定する
/// （`number_input_styled_root_and_reexported_parts_are_escaped_for_all_payloads`
/// と同粒度）。
#[test]
fn date_input_styled_root_and_reexported_parts_are_escaped_for_all_payloads() {
    for payload in payloads::all() {
        // styled root の呼び出し側 attrs 経路。
        let html = render(&date_input::root(
            Size::Md,
            false,
            false,
            vec![("data-testid", payload)],
            vec![],
        ));
        assert_payload_is_escaped(
            payload,
            &html,
            "date_input::root 呼び出し側 attrs コンテキスト",
        );

        // styled root の class 属性経路（drop_class_attr により生ペイロードは
        // 出力されず、recipe 生成クラスへ完全に置き換わる）。
        let html = render(&date_input::root(
            Size::Md,
            false,
            false,
            vec![("class", payload)],
            vec![],
        ));
        assert!(
            !html.contains(payload),
            "date_input::root の class 属性に渡した生ペイロードが出力に残っている: \
             payload={payload:?}, html={html}"
        );
        assert_eq!(
            html.matches("class=\"").count(),
            1,
            "date_input::root の class 属性が複数出現している: html={html}"
        );
        assert!(
            html.contains("fd-date-input--"),
            "date_input::root で recipe 生成クラスが失われている: html={html}"
        );

        // 選択的再エクスポートした label の children 経路。
        let html = render(&date_input::label(
            DateInputProps::default(),
            None,
            vec![],
            vec![text(payload)],
        ));
        assert_payload_is_escaped(payload, &html, "date_input::label children コンテキスト");

        // 選択的再エクスポートした hidden_input の name 経路。
        let html = render(&date_input::hidden_input(
            payload,
            "2026-07-22",
            false,
            vec![],
        ));
        assert_payload_is_escaped(payload, &html, "date_input::hidden_input name コンテキスト");

        // 選択的再エクスポートした segment の attrs 経路。
        let html = render(&date_input::segment(
            DateSegment::Year,
            None,
            "0",
            "9999",
            DateInputProps::default(),
            vec![("data-testid", payload)],
        ));
        assert_payload_is_escaped(payload, &html, "date_input::segment attrs コンテキスト");
    }
}

/// Timer（イシュー #836）styled 公開 API 経由の children テキスト・呼び出し
/// 側 attrs のエスケープ貫通を固定する
/// （`crates/headless-ui/tests/xss_escape.rs::timer_children_and_attrs_are_escaped_for_all_payloads`
/// の styled 層版）。
#[test]
fn timer_styled_children_and_attrs_are_escaped_for_all_payloads() {
    for payload in payloads::all() {
        let html = render(&timer::item_value(
            TimerUnit::Seconds,
            vec![],
            vec![text(payload)],
        ));
        assert_payload_is_escaped(payload, &html, "timer::item_value children コンテキスト");

        let html = render(&timer::action_trigger(
            TimerControl::Start,
            TimerPhase::Idle,
            vec![],
            vec![text(payload)],
        ));
        assert_payload_is_escaped(
            payload,
            &html,
            "timer::action_trigger children コンテキスト",
        );

        let html = render(&timer::root(
            false,
            0,
            0,
            1000,
            0,
            TimerPhase::Idle,
            vec![("data-testid", payload)],
            vec![],
        ));
        assert_payload_is_escaped(payload, &html, "timer::root attrs コンテキスト");
    }
}

/// styled ColorSwatch（イシュー #838）の XSS 回帰。
///
/// 対象の入力面:
/// 1. class/style 破棄経路: 呼び出し側 `attrs` の `class`/`style` を渡しても
///    recipe 生成クラス・`--fd-swatch-color` style へ完全置換され、生
///    ペイロードが一切残らないことを確認する。
/// 2. 属性値経路: 呼び出し側 `attrs`（`class`/`style` 以外）。
/// 3. children テキスト経路。
/// 4. 色値経路: `ColorSwatchProps::value` は
///    [`fandhe_frontend_pre_styled_ui::color_swatch::Color`] 型のみを受け
///    取るため、攻撃者が制御しうる生文字列を `style` へ注入する経路が
///    構造的に存在しない（`Color::to_hex_string()` の出力が `#[0-9a-f]` に
///    閉じることは `crates/pre-styled-ui/src/color_swatch.rs` の単体テストが
///    別途固定する）。
#[test]
fn color_swatch_class_style_and_children_payloads_are_escaped_for_all_payloads() {
    use fandhe_frontend_pre_styled_ui::color_swatch::{self, Color, ColorSwatchProps, Rgb};

    for payload in payloads::all() {
        let props = ColorSwatchProps {
            value: Color::from_rgb(Rgb::new(0x3b, 0x82, 0xf6)),
            ..ColorSwatchProps::default()
        };

        // (1) class/style 破棄経路。
        let html = render(&color_swatch::color_swatch(
            &props,
            vec![("class", payload), ("style", payload)],
            vec![],
        ));
        assert!(
            !html.contains(payload),
            "color_swatch::color_swatch の class/style 属性に渡した生ペイロードが \
             出力に残っている: payload={payload:?}, html={html}"
        );
        assert_eq!(
            html.matches("class=\"").count(),
            1,
            "color_swatch::color_swatch の class 属性が複数出現している: html={html}"
        );
        assert_eq!(
            html.matches("style=\"").count(),
            1,
            "color_swatch::color_swatch の style 属性が複数出現している: html={html}"
        );
        assert!(
            html.contains("fd-color-swatch--"),
            "color_swatch::color_swatch で recipe 生成クラスが失われている: html={html}"
        );
        assert!(
            html.contains("--fd-swatch-color:"),
            "color_swatch::color_swatch でフレームワーク生成 style が失われている: html={html}"
        );

        // (2) 属性値経路: 呼び出し側 attrs（class/style 以外）。
        let html = render(&color_swatch::color_swatch(
            &props,
            vec![("data-testid", payload)],
            vec![],
        ));
        assert_payload_is_escaped(
            payload,
            &html,
            "color_swatch::color_swatch 呼び出し側 attrs コンテキスト",
        );

        // (3) children テキスト経路。
        let html = render(&color_swatch::color_swatch(
            &props,
            vec![],
            vec![text(payload)],
        ));
        assert_payload_is_escaped(
            payload,
            &html,
            "color_swatch::color_swatch children コンテキスト",
        );
    }
}

/// SignaturePad（イシュー #843）: styled root の呼び出し側 attrs・
/// styled segment の `aria_label_text`・選択的再エクスポートした label の
/// children・hidden_input の `name`/`value` の 4 経路すべてで既定エスケープ
/// （REQ-1）が貫通することを固定する
/// （`date_input_styled_root_and_reexported_parts_are_escaped_for_all_payloads`
/// と同粒度）。
#[test]
fn signature_pad_styled_root_and_reexported_parts_are_escaped_for_all_payloads() {
    for payload in payloads::all() {
        // styled root の呼び出し側 attrs 経路。
        let html = render(&signature_pad::root(
            false,
            true,
            vec![("data-testid", payload)],
            vec![],
        ));
        assert_payload_is_escaped(
            payload,
            &html,
            "signature_pad::root 呼び出し側 attrs コンテキスト",
        );

        // styled root の class 属性経路（signature_pad は variant を持たない
        // ため recipe 生成クラスへの置換ではなく、drop_class_attr により
        // 呼び出し側の `class` が完全に除去されることを確認する。
        // `root_drops_caller_class`（crates/pre-styled-ui/src/signature_pad.rs）
        // と同型）。
        let html = render(&signature_pad::root(
            false,
            true,
            vec![("class", payload)],
            vec![],
        ));
        assert!(
            !html.contains(payload),
            "signature_pad::root の class 属性に渡した生ペイロードが出力に残っている: \
             payload={payload:?}, html={html}"
        );
        assert!(
            !html.contains("class="),
            "signature_pad::root は variant を持たないため class 属性自体が \
             出力されないはずだが出力されている: html={html}"
        );

        // styled segment の aria_label_text 経路。
        let html = render(&signature_pad::segment(
            300,
            150,
            Some(payload),
            vec![],
            vec![],
        ));
        assert_payload_is_escaped(
            payload,
            &html,
            "signature_pad::segment aria_label_text コンテキスト",
        );

        // 選択的再エクスポートした label の children 経路。
        let html = render(&signature_pad::label(false, vec![], vec![text(payload)]));
        assert_payload_is_escaped(payload, &html, "signature_pad::label children コンテキスト");

        // 選択的再エクスポートした hidden_input の name/value 経路。
        let html = render(&signature_pad::hidden_input(
            payload,
            payload,
            false,
            vec![],
        ));
        assert_payload_is_escaped(
            payload,
            &html,
            "signature_pad::hidden_input name/value コンテキスト",
        );
    }
}

/// Tour 経路（イシュー #841）: styled `root` の呼び出し側 `attrs`・`class`、
/// および全パーツが `state: &Tour` を取る `title`/`description`/`spotlight`
/// の children/attrs/`data-target` 経路すべてで既定エスケープ（REQ-1）が
/// 貫通することを固定する（`steps_styled_root_and_parts_are_escaped_for_all_payloads`
/// と同型）。
#[test]
fn tour_styled_root_and_parts_are_escaped_for_all_payloads() {
    use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::positioning::{
        Align, Placement, Side,
    };
    use fandhe_frontend_pre_styled_ui::fandhe_frontend_interactive::dispatch;

    for payload in payloads::all() {
        let step_with_payload_target = TourStep {
            id: "s1".to_string(),
            target: Some(payload.to_string()),
            title: "t".to_string(),
            description: "d".to_string(),
            placement: Placement::new(Side::Bottom, Align::Center),
        };
        let mut with_target =
            fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::tour::Tour::new(vec![
                step_with_payload_target,
            ]);
        dispatch(&mut with_target, "start", "");

        // styled root の呼び出し側 attrs 経路。
        let html = render(&tour::root(
            ColorPalette::Accent,
            &with_target,
            vec![("data-testid", payload)],
            vec![],
        ));
        assert_payload_is_escaped(payload, &html, "tour::root 呼び出し側 attrs コンテキスト");

        // styled root の class 属性経路（drop_class_attr により生ペイロード
        // は出力されず、recipe 生成クラスへ完全に置き換わる）。
        let html = render(&tour::root(
            ColorPalette::Accent,
            &with_target,
            vec![("class", payload)],
            vec![],
        ));
        assert!(
            !html.contains(payload),
            "tour::root の class 属性に渡した生ペイロードが出力に残っている: \
             payload={payload:?}, html={html}"
        );
        assert_eq!(
            html.matches("class=\"").count(),
            1,
            "tour::root の class 属性が複数出現している: html={html}"
        );
        assert!(
            html.contains("fd-tour--"),
            "tour::root で recipe 生成クラスが失われている: html={html}"
        );

        // spotlight の data-target 属性経路（TourStep::target 由来）。
        let html = render(&tour::spotlight(&with_target, vec![], vec![]));
        assert_payload_is_escaped(payload, &html, "tour::spotlight data-target コンテキスト");

        // title/description の children 経路。
        let html = render(&tour::title(
            &with_target,
            None,
            vec![],
            vec![text(payload)],
        ));
        assert_payload_is_escaped(payload, &html, "tour::title children コンテキスト");

        let html = render(&tour::description(
            &with_target,
            None,
            vec![],
            vec![text(payload)],
        ));
        assert_payload_is_escaped(payload, &html, "tour::description children コンテキスト");

        // content の ContentIds 経路（id/labelledby/describedby はいずれも
        // 呼び出し側が渡す属性値であり、既定エスケープを経由する）。
        let html = render(&tour::content(
            &with_target,
            TourContentIds {
                id: Some(payload),
                labelledby: Some(payload),
                describedby: Some(payload),
            },
            vec![],
            vec![],
        ));
        assert_payload_is_escaped(payload, &html, "tour::content ContentIds コンテキスト");
    }
}

/// styled ColorPicker（イシュー #839）の XSS 回帰。
///
/// 対象の入力面:
/// 1. style 上書き経路: [`fandhe_frontend_pre_styled_ui::color_picker::trigger`]/
///    `area_background`/`area_thumb`/`channel_slider_track`（Alpha）/
///    `channel_slider_thumb` はフレームワーク生成の `style`（custom
///    property）を持つため、呼び出し側 `attrs` の `style` を渡しても
///    完全置換され、生ペイロードが一切残らないことを確認する。
/// 2. 属性値経路: styled `root` の呼び出し側 `attrs`（`style` 以外）。
/// 3. 選択的再エクスポートした `label`/`hidden_input`/`channel_input` の
///    children・属性値経路。
/// 4. 色値経路: 動的 style へ到達するのは
///    [`fandhe_frontend_headless_ui::color::Color::to_hex_string`] の出力
///    （常に `#[0-9a-f]` に閉じる）と検証済み整数のみのため、攻撃者が
///    制御しうる生文字列を `style` へ注入する経路が構造的に存在しない。
#[test]
fn color_picker_style_dedup_attrs_and_reexported_parts_are_escaped_for_all_payloads() {
    use fandhe_frontend_headless_ui::color::{Color, Rgb};
    use fandhe_frontend_headless_ui::color_picker::ColorPicker;
    use fandhe_frontend_pre_styled_ui::color_picker;

    for payload in payloads::all() {
        let state = ColorPicker::from_color(Color::from_rgb(Rgb::new(0x3b, 0x82, 0xf6)));

        // (1) style 上書き経路: 呼び出し側 `style` はフレームワーク生成
        // custom property へ完全置換されるため（[`assert_payload_is_escaped`]
        // が前提とする「エスケープ済みの形で出力に残る」経路ではない）、
        // ここでは `style="..."` が唯一であること・生ペイロードが一切
        // 出力に残らないことを固定する（`crates/pre-styled-ui/src/slider.rs`
        // の `range_caller_style_attr_is_dropped_not_duplicated` と同型）。
        let cp_props = color_picker::ColorPickerProps::default();

        let html = render(&color_picker::trigger(
            &state,
            &cp_props,
            None,
            vec![("style", payload)],
            vec![],
        ));
        assert_eq!(html.matches("style=\"").count(), 1);
        assert!(!html.contains(payload));
        assert!(!html.contains("<script>"));

        // (1) style 上書き経路: area_background。
        let html = render(&color_picker::area_background(
            &state,
            &cp_props,
            vec![("style", payload)],
            vec![],
        ));
        assert_eq!(html.matches("style=\"").count(), 1);
        assert!(!html.contains(payload));
        assert!(!html.contains("<script>"));

        // (1) style 上書き経路: area_thumb。
        let html = render(&color_picker::area_thumb(
            &state,
            &cp_props,
            vec![("style", payload)],
            vec![],
        ));
        assert_eq!(html.matches("style=\"").count(), 1);
        assert!(!html.contains(payload));
        assert!(!html.contains("<script>"));

        // (1) style 上書き経路: channel_slider_track（Alpha）。
        let html = render(&color_picker::channel_slider_track(
            color_picker::Channel::Alpha,
            &state,
            vec![("style", payload)],
            vec![],
        ));
        assert_eq!(html.matches("style=\"").count(), 1);
        assert!(!html.contains(payload));
        assert!(!html.contains("<script>"));

        // (1) style 上書き経路: channel_slider_thumb。
        let html = render(&color_picker::channel_slider_thumb(
            color_picker::Channel::Hue,
            &state,
            &cp_props,
            vec![("style", payload)],
            vec![],
        ));
        assert_eq!(html.matches("style=\"").count(), 1);
        assert!(!html.contains(payload));
        assert!(!html.contains("<script>"));

        // (2) 属性値経路: styled root。
        let html = render(&color_picker::root(
            &state,
            &cp_props,
            vec![("data-testid", payload)],
            vec![],
        ));
        assert_payload_is_escaped(
            payload,
            &html,
            "color_picker::root 呼び出し側 attrs コンテキスト",
        );

        // (3) 再エクスポート label の children テキスト経路。
        let html = render(&color_picker::label(&cp_props, vec![], vec![text(payload)]));
        assert_payload_is_escaped(payload, &html, "color_picker::label children コンテキスト");

        // (3) 再エクスポート hidden_input の name/value 属性値経路。
        let html = render(&color_picker::hidden_input(
            payload,
            "#ffffff",
            &cp_props,
            vec![],
        ));
        assert_payload_is_escaped(
            payload,
            &html,
            "color_picker::hidden_input の name 属性値コンテキスト",
        );

        // (3) 再エクスポート channel_input の value 属性値経路。
        let html = render(&color_picker::channel_input(payload, &cp_props, vec![]));
        assert_payload_is_escaped(
            payload,
            &html,
            "color_picker::channel_input の value 属性値コンテキスト",
        );
    }
}

/// styled PieChart / DonutChart（イシュー #850）の XSS 回帰。
///
/// 攻撃面: (1) カテゴリ名ラベル（`show_labels: true` の children テキスト
/// 経路、`crate::pie_chart`/`crate::donut_chart` モジュール doc「anatomy」
/// 節の `label` パーツ）。(2) `aria_label` プロパティ（`chart` の
/// `aria-label` 属性値経路）。(3) 呼び出し側 `attrs`（`root` への透過）。
/// (4) 呼び出し側 `attrs` の `class`（`drop_class_attr` による単一化）。
/// (5) `PieLabelPosition::Outside` 時のカテゴリ名ラベル（`outside-label`
/// children テキスト経路、イシュー #2084）。(6)
/// `PieChartProps::stacked` 時の `Series::name` → `data-series` 属性値
/// 経路（イシュー #2084）。(7) `DonutChartProps::center_text.value`/
/// `.label`（`center-value`/`center-label` children テキスト経路、
/// イシュー #2084）。(8) `charts::legend::category_legend` のカテゴリ名
/// （`label` children テキスト経路、イシュー #2084）。
///
/// `d`/`fill` 属性は [`crate::charts::pie`]/[`crate::charts::svg::fmt_coord`]
/// 経由の数値・固定リテラルのみで構成され任意文字列の混入経路を持たない
/// ため（`pie_chart.rs`/`donut_chart.rs` モジュール doc「セキュリティ不変
/// 条件」節）、本テストの対象外とする。
#[test]
fn pie_and_donut_chart_are_escaped_for_all_payloads() {
    for payload in payloads::all() {
        let data = ChartData::new(
            vec![payload.to_string(), "other".to_string()],
            vec![Series::new("total", vec![60.0, 40.0])],
        )
        .unwrap();

        // (1) カテゴリ名ラベル（children テキスト経路）。
        let pie_props = PieChartProps {
            show_labels: true,
            ..PieChartProps::default()
        };
        let html = render(&pie_chart(&pie_props, &data, vec![]).unwrap());
        assert_payload_is_escaped(payload, &html, "pie_chart label children コンテキスト");

        let donut_props = DonutChartProps {
            show_labels: true,
            ..DonutChartProps::default()
        };
        let html = render(&donut_chart(&donut_props, &data, vec![]).unwrap());
        assert_payload_is_escaped(payload, &html, "donut_chart label children コンテキスト");

        // (2) aria_label 属性値経路。
        let pie_props = PieChartProps {
            aria_label: Some(payload),
            ..PieChartProps::default()
        };
        let html = render(&pie_chart(&pie_props, &data, vec![]).unwrap());
        assert_payload_is_escaped(payload, &html, "pie_chart aria_label 属性値コンテキスト");

        let donut_props = DonutChartProps {
            aria_label: Some(payload),
            ..DonutChartProps::default()
        };
        let html = render(&donut_chart(&donut_props, &data, vec![]).unwrap());
        assert_payload_is_escaped(payload, &html, "donut_chart aria_label 属性値コンテキスト");

        // (3) 呼び出し側 attrs（root への透過）。
        let html = render(
            &pie_chart(
                &PieChartProps::default(),
                &data,
                vec![("data-testid", payload)],
            )
            .unwrap(),
        );
        assert_payload_is_escaped(payload, &html, "pie_chart 呼び出し側 attrs コンテキスト");

        let html = render(
            &donut_chart(
                &DonutChartProps::default(),
                &data,
                vec![("data-testid", payload)],
            )
            .unwrap(),
        );
        assert_payload_is_escaped(payload, &html, "donut_chart 呼び出し側 attrs コンテキスト");

        // (4) 呼び出し側 attrs の class（drop_class_attr による単一化）。
        let html =
            render(&pie_chart(&PieChartProps::default(), &data, vec![("class", payload)]).unwrap());
        assert!(
            !html.contains(payload),
            "pie_chart の class 属性に渡した生ペイロードが出力に残っている: payload={payload:?}, html={html}"
        );
        assert_eq!(
            html.matches("class=\"").count(),
            1,
            "pie_chart の class 属性が複数出現している: html={html}"
        );

        let html = render(
            &donut_chart(&DonutChartProps::default(), &data, vec![("class", payload)]).unwrap(),
        );
        assert!(
            !html.contains(payload),
            "donut_chart の class 属性に渡した生ペイロードが出力に残っている: payload={payload:?}, html={html}"
        );
        assert_eq!(
            html.matches("class=\"").count(),
            1,
            "donut_chart の class 属性が複数出現している: html={html}"
        );

        // (5) PieLabelPosition::Outside 時のカテゴリ名ラベル（children
        // テキスト経路、イシュー #2084）。
        let pie_outside_props = PieChartProps {
            show_labels: true,
            label_position: PieLabelPosition::Outside,
            ..PieChartProps::default()
        };
        let html = render(&pie_chart(&pie_outside_props, &data, vec![]).unwrap());
        assert_payload_is_escaped(
            payload,
            &html,
            "pie_chart outside-label children コンテキスト",
        );

        let donut_outside_props = DonutChartProps {
            show_labels: true,
            label_position: PieLabelPosition::Outside,
            ..DonutChartProps::default()
        };
        let html = render(&donut_chart(&donut_outside_props, &data, vec![]).unwrap());
        assert_payload_is_escaped(
            payload,
            &html,
            "donut_chart outside-label children コンテキスト",
        );

        // (6) PieChartProps::stacked 時の Series::name → data-series
        // 属性値経路（イシュー #2084）。
        let stacked_data = ChartData::new(
            vec!["A".to_string(), "B".to_string()],
            vec![Series::new(payload, vec![60.0, 40.0])],
        )
        .unwrap();
        let stacked_props = PieChartProps {
            stacked: true,
            ..PieChartProps::default()
        };
        let html = render(&pie_chart(&stacked_props, &stacked_data, vec![]).unwrap());
        assert_payload_is_escaped(payload, &html, "pie_chart data-series 属性値コンテキスト");

        // (7) DonutChartProps::center_text.value / .label（children テキスト
        // 経路、イシュー #2084）。
        let center_text_props = DonutChartProps {
            center_text: Some(PieCenterText {
                value: payload,
                label: Some(payload),
            }),
            ..DonutChartProps::default()
        };
        let html = render(&donut_chart(&center_text_props, &data, vec![]).unwrap());
        assert_payload_is_escaped(
            payload,
            &html,
            "donut_chart center_text children コンテキスト",
        );

        // (8) charts::legend::category_legend のカテゴリ名（children テキスト
        // 経路、イシュー #2084）。
        let html = render(&category_legend(&data, &LegendProps::default()));
        assert_payload_is_escaped(
            payload,
            &html,
            "category_legend label children コンテキスト",
        );
    }
}

/// charts の期間切替・凡例トグル SSR 構造（イシュー #2133）の XSS 回帰。
///
/// 攻撃面: (1) `LineChartProps::range`（root `data-range` 属性値経路。
/// `line_chart` を代表とし、他チャートも同一の `drop_range_attr` 経由の
/// 属性合成であることをソースレビューで確認済み）。(2)
/// `charts::legend::legend` の `trigger` の `data-series`（`Series::name`
/// が button の属性値として流れる経路。従来のテキストノード children
/// コンテキストとは別の属性値コンテキスト）。(3)
/// `LegendProps::controls`（`aria-controls` 属性値経路）。
#[test]
fn charts_range_and_legend_trigger_attrs_are_escaped_for_all_payloads() {
    use fandhe_frontend_pre_styled_ui::line_chart::{line_chart, LineChartProps};

    for payload in payloads::all() {
        let data = ChartData::new(
            vec!["a".to_string(), "b".to_string()],
            vec![Series::new("visits", vec![1.0, 2.0])],
        )
        .unwrap();

        // (1) range 属性値経路。
        let mut props = LineChartProps::new(&data, "label");
        props.range = Some(payload);
        let html = render(&line_chart(&props, vec![]).unwrap());
        assert_payload_is_escaped(payload, &html, "line_chart data-range 属性値コンテキスト");

        // (2) legend trigger の data-series 属性値経路（系列名）。
        let series_data =
            ChartData::new(vec!["a".to_string()], vec![Series::new(payload, vec![1.0])]).unwrap();
        let html = render(&fandhe_frontend_pre_styled_ui::charts::legend::legend(
            &series_data,
            &LegendProps::default(),
        ));
        assert_payload_is_escaped(
            payload,
            &html,
            "legend trigger data-series 属性値コンテキスト",
        );

        // (3) LegendProps::controls の aria-controls 属性値経路。
        let controls_props = LegendProps {
            controls: Some(payload.to_string()),
            ..LegendProps::default()
        };
        let html = render(&fandhe_frontend_pre_styled_ui::charts::legend::legend(
            &data,
            &controls_props,
        ));
        assert_payload_is_escaped(
            payload,
            &html,
            "legend trigger aria-controls 属性値コンテキスト",
        );
    }
}

/// styled RadialChart（イシュー #2079）の XSS 回帰。
///
/// 攻撃面: (1) カテゴリ名ラベル（`show_labels: true` の children テキスト
/// 経路、`crate::radial_chart` モジュール doc「anatomy」節の `label`
/// パーツ）。(2) `aria_label` プロパティ（`chart` の `aria-label` 属性値
/// 経路）。(3) 呼び出し側 `attrs`（`root` への透過）。(4) 呼び出し側
/// `attrs` の `class`（`drop_class_attr` による単一化）。(5)
/// `center_text.value`/`center_text.label`（`center-value`/`center-label`
/// children テキスト経路）。(6) `Series::name` → `data-series` 属性値
/// 経路。
///
/// `d`/`fill` 属性は [`crate::charts::pie`]/[`crate::charts::svg::fmt_coord`]
/// 経由の数値・固定リテラルのみで構成され任意文字列の混入経路を持たない
/// ため（`radial_chart.rs` モジュール doc「セキュリティ不変条件」節）、
/// 本テストの対象外とする。
#[test]
fn radial_chart_is_escaped_for_all_payloads() {
    for payload in payloads::all() {
        let data = ChartData::new(
            vec![payload.to_string(), "other".to_string()],
            vec![Series::new(payload.to_string(), vec![60.0, 40.0])],
        )
        .unwrap();

        // (1) カテゴリ名ラベル（children テキスト経路）。
        let props = RadialChartProps {
            show_labels: true,
            ..RadialChartProps::default()
        };
        let html = render(&radial_chart(&props, &data, vec![]).unwrap());
        assert_payload_is_escaped(payload, &html, "radial_chart label children コンテキスト");

        // (2) aria_label 属性値経路。
        let props = RadialChartProps {
            aria_label: Some(payload),
            ..RadialChartProps::default()
        };
        let html = render(&radial_chart(&props, &data, vec![]).unwrap());
        assert_payload_is_escaped(payload, &html, "radial_chart aria_label 属性値コンテキスト");

        // (3) 呼び出し側 attrs（root への透過）。
        let html = render(
            &radial_chart(
                &RadialChartProps::default(),
                &data,
                vec![("data-testid", payload)],
            )
            .unwrap(),
        );
        assert_payload_is_escaped(payload, &html, "radial_chart 呼び出し側 attrs コンテキスト");

        // (4) 呼び出し側 attrs の class（drop_class_attr による単一化）。
        let html = render(
            &radial_chart(
                &RadialChartProps::default(),
                &data,
                vec![("class", payload)],
            )
            .unwrap(),
        );
        assert!(
            !html.contains(payload),
            "radial_chart の class 属性に渡した生ペイロードが出力に残っている: payload={payload:?}, html={html}"
        );
        assert_eq!(
            html.matches("class=\"").count(),
            1,
            "radial_chart の class 属性が複数出現している: html={html}"
        );

        // (5) center_text.value / center_text.label（children テキスト経路）。
        let single_category = ChartData::new(
            vec!["visitors".to_string()],
            vec![Series::new("total", vec![100.0])],
        )
        .unwrap();
        let props = RadialChartProps {
            center_text: Some(RadialCenterText {
                value: payload,
                label: Some(payload),
            }),
            ..RadialChartProps::default()
        };
        let html = render(&radial_chart(&props, &single_category, vec![]).unwrap());
        assert_payload_is_escaped(
            payload,
            &html,
            "radial_chart center_text children コンテキスト",
        );

        // (6) Series::name → data-series 属性値経路。
        let html = render(&radial_chart(&RadialChartProps::default(), &data, vec![]).unwrap());
        assert_payload_is_escaped(
            payload,
            &html,
            "radial_chart data-series 属性値コンテキスト",
        );
    }
}

/// (25) charts ScatterChart/RadarChart 経路（イシュー #851）: `data-series`
/// 属性値（両部品共通）・カテゴリ名（`svg_text` children、RadarChart 軸
/// ラベル）・`aria_label`（両部品共通の `role="img"` 代替テキスト属性値）の
/// 3 入力面を `payloads::all()` で網羅する。
#[test]
fn charts_scatter_and_radar_are_escaped_for_all_payloads() {
    for payload in payloads::all() {
        // ScatterChart: data-series 属性値経路。
        let scatter_data =
            ScatterData::new(vec![ScatterSeries::new(payload, vec![(0.0, 0.0)])]).unwrap();
        let html = render(
            &scatter_chart::root(&scatter_data, ScatterChartProps::default(), "label").unwrap(),
        );
        assert_payload_is_escaped(
            payload,
            &html,
            "scatter_chart::root の data-series 属性値コンテキスト",
        );

        // ScatterChart: aria_label 属性値経路。
        let plain_scatter_data =
            ScatterData::new(vec![ScatterSeries::new("s1", vec![(0.0, 0.0)])]).unwrap();
        let html = render(
            &scatter_chart::root(&plain_scatter_data, ScatterChartProps::default(), payload)
                .unwrap(),
        );
        assert_payload_is_escaped(
            payload,
            &html,
            "scatter_chart::root の aria_label 属性値コンテキスト",
        );

        // RadarChart: カテゴリ名（軸ラベル、svg_text children）経路。
        let radar_data_category = ChartData::new(
            vec![payload.to_string(), "b".to_string(), "c".to_string()],
            vec![Series::new("s1", vec![1.0, 2.0, 3.0])],
        )
        .unwrap();
        let html = render(
            &radar_chart::root(&radar_data_category, RadarChartProps::default(), "label").unwrap(),
        );
        assert_payload_is_escaped(
            payload,
            &html,
            "radar_chart::root の軸ラベル children コンテキスト",
        );

        // RadarChart: data-series 属性値経路。
        let radar_data_series = ChartData::new(
            vec!["a".to_string(), "b".to_string(), "c".to_string()],
            vec![Series::new(payload, vec![1.0, 2.0, 3.0])],
        )
        .unwrap();
        let html = render(
            &radar_chart::root(&radar_data_series, RadarChartProps::default(), "label").unwrap(),
        );
        assert_payload_is_escaped(
            payload,
            &html,
            "radar_chart::root の data-series 属性値コンテキスト",
        );

        // RadarChart: aria_label 属性値経路。
        let plain_radar_data = ChartData::new(
            vec!["a".to_string(), "b".to_string(), "c".to_string()],
            vec![Series::new("s1", vec![1.0, 2.0, 3.0])],
        )
        .unwrap();
        let html = render(
            &radar_chart::root(&plain_radar_data, RadarChartProps::default(), payload).unwrap(),
        );
        assert_payload_is_escaped(
            payload,
            &html,
            "radar_chart::root の aria_label 属性値コンテキスト",
        );
    }
}

/// (25b) RadarChart 経路（イシュー #2085、shadcn/ui Charts（radar）突合）:
/// `axis_label: RadarAxisLabel::ValueAndCategory` のカテゴリ名（`tspan`
/// children 経路）と、全バリアント ON（`grid: Circle`・`grid_fill: Series`・
/// `dots: true`・`radius_axis: true`・`fill: None`・`spokes: false`）時の
/// `aria_label`・系列名（`data-series`）を `payloads::all()` で網羅する。
#[test]
fn radar_chart_shadcn_variants_are_escaped_for_all_payloads() {
    for payload in payloads::all() {
        // ValueAndCategory: カテゴリ名（tspan children）経路。
        let category_data = ChartData::new(
            vec![payload.to_string(), "b".to_string(), "c".to_string()],
            vec![Series::new("s1", vec![1.0, 2.0, 3.0])],
        )
        .unwrap();
        let props = RadarChartProps {
            axis_label: radar_chart::RadarAxisLabel::ValueAndCategory,
            ..RadarChartProps::default()
        };
        let html = render(&radar_chart::root(&category_data, props, "label").unwrap());
        assert_payload_is_escaped(
            payload,
            &html,
            "radar_chart::root の ValueAndCategory カテゴリ名 tspan children コンテキスト",
        );

        // 全バリアント ON: data-series 属性値・aria_label 属性値経路。
        let all_on_props = RadarChartProps {
            grid: radar_chart::RadarGrid::Circle,
            grid_fill: radar_chart::RadarGridFill::Series,
            dots: true,
            radius_axis: true,
            fill: radar_chart::RadarFill::None,
            spokes: false,
            ..RadarChartProps::default()
        };
        let series_data = ChartData::new(
            vec!["a".to_string(), "b".to_string(), "c".to_string()],
            vec![Series::new(payload, vec![1.0, 2.0, 3.0])],
        )
        .unwrap();
        let html = render(&radar_chart::root(&series_data, all_on_props.clone(), "label").unwrap());
        assert_payload_is_escaped(
            payload,
            &html,
            "radar_chart::root（全バリアント ON）の data-series 属性値コンテキスト",
        );

        let plain_data = ChartData::new(
            vec!["a".to_string(), "b".to_string(), "c".to_string()],
            vec![Series::new("s1", vec![1.0, 2.0, 3.0])],
        )
        .unwrap();
        let html = render(&radar_chart::root(&plain_data, all_on_props, payload).unwrap());
        assert_payload_is_escaped(
            payload,
            &html,
            "radar_chart::root（全バリアント ON）の aria_label 属性値コンテキスト",
        );
    }
}

/// (25a) AreaChart 経路（イシュー #2081、shadcn/ui Charts（area）突合）:
/// `aria_label`・呼び出し側 `attrs`・軸ラベル経路（カテゴリ名、
/// `show_x_axis` 有効時）の全ペイロードで既定エスケープが貫通することを
/// 固定する。`stack`/`fill: AreaFill::Gradient` を有効にした構成でも
/// 崩れないことをあわせて確認する（gradient の `<linearGradient>` は
/// `SeriesColor` 固定形の `stop-color` のみを埋め込む契約、
/// `crate::area_chart` モジュール doc「gradient の不変条件」参照）。
#[test]
fn area_chart_is_escaped_for_all_payloads() {
    let data = ChartData::new(
        vec!["Jan".to_string(), "Feb".to_string()],
        vec![Series::new("visits", vec![1.0, 2.0])],
    )
    .unwrap();

    for payload in payloads::all() {
        let html = render(&area_chart(&AreaChartProps::new(&data, payload), vec![]).unwrap());
        assert_payload_is_escaped(payload, &html, "area_chart aria_label 属性値コンテキスト");

        let html = render(
            &area_chart(
                &AreaChartProps::new(&data, "attrs"),
                vec![("data-testid", payload)],
            )
            .unwrap(),
        );
        assert_payload_is_escaped(payload, &html, "area_chart 呼び出し側 attrs コンテキスト");

        // カテゴリ名（軸ラベル経路、show_x_axis 有効時のみ描画される）。
        let payload_data = ChartData::new(
            vec![payload.to_string(), "b".to_string()],
            vec![Series::new("s", vec![1.0, 2.0])],
        )
        .unwrap();
        let mut axis_props = AreaChartProps::new(&payload_data, "axis-label");
        axis_props.show_x_axis = true;
        let html = render(&area_chart(&axis_props, vec![]).unwrap());
        assert_payload_is_escaped(payload, &html, "area_chart X 軸カテゴリラベルコンテキスト");
    }

    // stack + gradient を有効にした構成でも既定エスケープが崩れないことを
    // 確認する（gradient defs の `<defs>`/`<linearGradient>`/`<stop>` は
    // すべて `el()` 経由でありユーザー入力を埋め込まない契約）。
    let payload = "\"><script>alert(1)</script>";
    let stacked_data = ChartData::new(
        vec![payload.to_string(), "b".to_string()],
        vec![
            Series::new("a", vec![1.0, 2.0]),
            Series::new("b", vec![1.0, 2.0]),
        ],
    )
    .unwrap();
    let mut stacked_props = AreaChartProps::new(&stacked_data, payload);
    stacked_props.stack = AreaStack::Normal;
    stacked_props.fill = AreaFill::Gradient;
    stacked_props.show_x_axis = true;
    let html = render(&area_chart(&stacked_props, vec![]).unwrap());
    assert_payload_is_escaped(payload, &html, "area_chart stack+gradient 合成コンテキスト");
}

/// (25b) LineChart 経路（イシュー #2083、shadcn/ui Charts（line）突合）:
/// `aria_label`・呼び出し側 `attrs`・カテゴリ名経路（`show_x_axis`/
/// `label: LineLabel::Category` の 2 経路）の全ペイロードで既定エスケープが
/// 貫通することを固定する。`curve: Curve::Natural`・`dots: LineDots::Hollow`・
/// `label: LineLabel::Category`・軸/グリッド全有効を組み合わせた合成構成
/// でも崩れないことをあわせて確認する（`(25a)` の area_chart
/// stack+gradient 合成確認と同型）。
#[test]
fn line_chart_is_escaped_for_all_payloads() {
    let data = ChartData::new(
        vec!["Jan".to_string(), "Feb".to_string()],
        vec![Series::new("visits", vec![1.0, 2.0])],
    )
    .unwrap();

    for payload in payloads::all() {
        let html = render(&line_chart(&LineChartProps::new(&data, payload), vec![]).unwrap());
        assert_payload_is_escaped(payload, &html, "line_chart aria_label 属性値コンテキスト");

        let html = render(
            &line_chart(
                &LineChartProps::new(&data, "attrs"),
                vec![("data-testid", payload)],
            )
            .unwrap(),
        );
        assert_payload_is_escaped(payload, &html, "line_chart 呼び出し側 attrs コンテキスト");

        // カテゴリ名（X 軸ラベル経路、show_x_axis 有効時のみ描画される）。
        let payload_data = ChartData::new(
            vec![payload.to_string(), "b".to_string()],
            vec![Series::new("s", vec![1.0, 2.0])],
        )
        .unwrap();
        let mut axis_props = LineChartProps::new(&payload_data, "axis-label");
        axis_props.show_x_axis = true;
        let html = render(&line_chart(&axis_props, vec![]).unwrap());
        assert_payload_is_escaped(payload, &html, "line_chart X 軸カテゴリラベルコンテキスト");

        // カテゴリ名（value-label 経路、label: LineLabel::Category 有効時
        // のみ描画される。`text()` ノード経由で軸ラベルとは別のコード
        // パスを通るため独立して確認する）。
        let mut label_props = LineChartProps::new(&payload_data, "label-category");
        label_props.label = LineLabel::Category;
        let html = render(&line_chart(&label_props, vec![]).unwrap());
        assert_payload_is_escaped(
            payload,
            &html,
            "line_chart value-label カテゴリコンテキスト",
        );
    }

    // curve: Natural + dots: Hollow + label: Category + 軸/グリッド全有効の
    // 合成構成でも既定エスケープが崩れないことを確認する。
    let payload = "\"><script>alert(1)</script>";
    let composite_data = ChartData::new(
        vec![payload.to_string(), "b".to_string(), "c".to_string()],
        vec![Series::new("s", vec![1.0, 2.0, 3.0])],
    )
    .unwrap();
    let mut composite_props = LineChartProps::new(&composite_data, payload);
    composite_props.curve = fandhe_frontend_pre_styled_ui::charts::Curve::Natural;
    composite_props.dots = LineDots::Hollow;
    composite_props.label = LineLabel::Category;
    composite_props.show_x_axis = true;
    composite_props.show_y_axis = true;
    composite_props.show_grid = true;
    let html = render(&line_chart(&composite_props, vec![]).unwrap());
    assert_payload_is_escaped(payload, &html, "line_chart 合成コンテキスト");
}

/// (26) charts BarChart/BarList/BarSegment 経路（イシュー #849、親 Phase #845）:
/// カテゴリ名・系列名・BarChart の `aria_label` の各所すべてで既定エスケープ
/// （REQ-1）が貫通することを固定する。SVG（BarChart）/HTML（BarList/
/// BarSegment）双方の出力経路を対象とする。
#[test]
fn bar_charts_category_series_and_aria_label_are_escaped_for_all_payloads() {
    use fandhe_frontend_pre_styled_ui::charts::bar_chart::{self, BarChartProps};
    use fandhe_frontend_pre_styled_ui::charts::bar_list;
    use fandhe_frontend_pre_styled_ui::charts::bar_segment;
    use fandhe_frontend_pre_styled_ui::charts::data::{ChartData, Series};

    for payload in payloads::all() {
        // BarChart: カテゴリ名（svg_text children）経路。
        let data = ChartData::new(
            vec![payload.to_string(), "b".to_string()],
            vec![Series::new("s", vec![1.0, 2.0])],
        )
        .unwrap();
        let html = render(&bar_chart::root(&data, BarChartProps::default(), "label").unwrap());
        assert_payload_is_escaped(
            payload,
            &html,
            "bar_chart::root カテゴリ名 children コンテキスト",
        );

        // BarChart: `aria_label` 属性値経路。
        let data = ChartData::new(
            vec!["a".to_string(), "b".to_string()],
            vec![Series::new("s", vec![1.0, 2.0])],
        )
        .unwrap();
        let html = render(&bar_chart::root(&data, BarChartProps::default(), payload).unwrap());
        assert_payload_is_escaped(
            payload,
            &html,
            "bar_chart::root の aria-label 属性値コンテキスト",
        );

        // BarChart（イシュー #2082）: `label: BarLabel::Inside` の
        // inside-label（カテゴリ名を text() で描く新経路）。
        let data = ChartData::new(
            vec![payload.to_string(), "b".to_string()],
            vec![Series::new("s", vec![1.0, 2.0])],
        )
        .unwrap();
        let inside_props = BarChartProps {
            label: bar_chart::BarLabel::Inside,
            ..BarChartProps::default()
        };
        let html = render(&bar_chart::root(&data, inside_props, "label").unwrap());
        assert_payload_is_escaped(
            payload,
            &html,
            "bar_chart::root の inside-label（BarLabel::Inside）children コンテキスト",
        );

        // BarChart（イシュー #2082）: 積み上げ・角丸・active_index・軸を
        // 合成した経路の `aria_label` 属性値コンテキスト。
        let data = ChartData::new(
            vec!["a".to_string(), "b".to_string()],
            vec![
                Series::new("s1", vec![1.0, 2.0]),
                Series::new("s2", vec![3.0, 4.0]),
            ],
        )
        .unwrap();
        let combined_props = BarChartProps {
            stack: bar_chart::BarStack::Normal,
            corner_radius: 4.0,
            active_index: Some(0),
            show_value_axis: true,
            show_grid: true,
            ..BarChartProps::default()
        };
        let html = render(&bar_chart::root(&data, combined_props, payload).unwrap());
        assert_payload_is_escaped(
            payload,
            &html,
            "bar_chart::root の積み上げ/角丸/active/軸合成時の aria-label 属性値コンテキスト",
        );

        // BarList: カテゴリ名（children）経路。
        let data = ChartData::new(
            vec![payload.to_string(), "b".to_string()],
            vec![Series::new("visits", vec![1.0, 2.0])],
        )
        .unwrap();
        let html = render(&bar_list::root(&data, "visits").unwrap());
        assert_payload_is_escaped(
            payload,
            &html,
            "bar_list::root カテゴリ名 children コンテキスト",
        );

        // BarSegment: カテゴリ名（legend の label children）経路。
        let data = ChartData::new(
            vec![payload.to_string(), "b".to_string()],
            vec![Series::new("visits", vec![1.0, 2.0])],
        )
        .unwrap();
        let html = render(&bar_segment::root(&data, "visits").unwrap());
        assert_payload_is_escaped(
            payload,
            &html,
            "bar_segment::root legend ラベル children コンテキスト",
        );
    }
}

/// (27) `dialog::footer` 経路（イシュー #1690、親 #1675）: pre-styled-only
/// `footer` パート（`Anatomy::part` 直接呼び出し、`crate::card::footer` と
/// 同型）の children・呼び出し側 `attrs` の両方で既定エスケープ（REQ-1）が
/// 貫通することを固定する。あわせて `data-scope`/`data-part` の偽装が
/// headless 層（`Anatomy::part`）により除去され、生値が出力に残らないこと
/// も固定する。
#[test]
fn dialog_footer_children_and_attrs_are_escaped_for_all_payloads() {
    for payload in payloads::all() {
        // children 経路。
        let html = render(&dialog::footer(vec![], vec![text(payload)]));
        assert_payload_is_escaped(payload, &html, "dialog::footer children コンテキスト");

        // 呼び出し側 attrs（data-testid）経路。
        let html = render(&dialog::footer(vec![("data-testid", payload)], vec![]));
        assert_payload_is_escaped(
            payload,
            &html,
            "dialog::footer 呼び出し側 attrs コンテキスト",
        );

        // data-scope/data-part 偽装は headless `Anatomy::part` が除去する。
        let html = render(&dialog::footer(
            vec![("data-scope", payload), ("data-part", payload)],
            vec![],
        ));
        assert!(
            !html.contains(payload),
            "dialog::footer の data-scope/data-part 偽装ペイロードが出力に残っている: \
             payload={payload:?}, html={html}"
        );
        assert!(html.contains(r#"data-scope="dialog""#));
        assert!(html.contains(r#"data-part="footer""#));
    }
}

/// (28) `dialog::body` 経路（イシュー #2030、親 #2025、shadcn/ui 突合）:
/// pre-styled-only `body` パート（`Anatomy::part` 直接呼び出し、
/// `dialog::footer` と同型）の children・呼び出し側 `attrs` の両方で
/// 既定エスケープ（REQ-1）が貫通することを固定する。あわせて
/// `data-scope`/`data-part` の偽装が headless 層（`Anatomy::part`）により
/// 除去され、生値が出力に残らないことも固定する。
#[test]
fn dialog_body_children_and_attrs_are_escaped_for_all_payloads() {
    for payload in payloads::all() {
        // children 経路。
        let html = render(&dialog::body(vec![], vec![text(payload)]));
        assert_payload_is_escaped(payload, &html, "dialog::body children コンテキスト");

        // 呼び出し側 attrs（data-testid）経路。
        let html = render(&dialog::body(vec![("data-testid", payload)], vec![]));
        assert_payload_is_escaped(payload, &html, "dialog::body 呼び出し側 attrs コンテキスト");

        // data-scope/data-part 偽装は headless `Anatomy::part` が除去する。
        let html = render(&dialog::body(
            vec![("data-scope", payload), ("data-part", payload)],
            vec![],
        ));
        assert!(
            !html.contains(payload),
            "dialog::body の data-scope/data-part 偽装ペイロードが出力に残っている: \
             payload={payload:?}, html={html}"
        );
        assert!(html.contains(r#"data-scope="dialog""#));
        assert!(html.contains(r#"data-part="body""#));
    }
}

/// (29) `avatar::group`/`avatar::badge` 経路（イシュー #2044、shadcn/ui
/// 突合）: pre-styled-only `group`/`badge` パート（`Anatomy::part` 直接
/// 呼び出し、`dialog::footer`/`dialog::body` と同型）の children・呼び出し側
/// `attrs` の両方で既定エスケープ（REQ-1）が貫通することを固定する。
/// あわせて `data-scope`/`data-part` の偽装が headless 層（`Anatomy::part`）
/// により除去され、生値が出力に残らないことも固定する。
#[test]
fn avatar_group_and_badge_children_and_attrs_are_escaped_for_all_payloads() {
    for payload in payloads::all() {
        // group: children 経路。
        let html = render(&avatar::group(vec![], vec![text(payload)]));
        assert_payload_is_escaped(payload, &html, "avatar::group children コンテキスト");

        // group: 呼び出し側 attrs（data-testid）経路。
        let html = render(&avatar::group(vec![("data-testid", payload)], vec![]));
        assert_payload_is_escaped(
            payload,
            &html,
            "avatar::group 呼び出し側 attrs コンテキスト",
        );

        // group: data-scope/data-part 偽装は headless `Anatomy::part` が除去する。
        let html = render(&avatar::group(
            vec![("data-scope", payload), ("data-part", payload)],
            vec![],
        ));
        assert!(
            !html.contains(payload),
            "avatar::group の data-scope/data-part 偽装ペイロードが出力に残っている: \
             payload={payload:?}, html={html}"
        );
        assert!(html.contains(r#"data-scope="avatar""#));
        assert!(html.contains(r#"data-part="group""#));

        // badge: children 経路。
        let html = render(&avatar::badge(
            &AvatarBadgeProps::default(),
            vec![],
            vec![text(payload)],
        ));
        assert_payload_is_escaped(payload, &html, "avatar::badge children コンテキスト");

        // badge: 呼び出し側 attrs（data-testid）経路。
        let html = render(&avatar::badge(
            &AvatarBadgeProps::default(),
            vec![("data-testid", payload)],
            vec![],
        ));
        assert_payload_is_escaped(
            payload,
            &html,
            "avatar::badge 呼び出し側 attrs コンテキスト",
        );

        // badge: data-scope/data-part 偽装は headless `Anatomy::part` が除去する。
        let html = render(&avatar::badge(
            &AvatarBadgeProps::default(),
            vec![("data-scope", payload), ("data-part", payload)],
            vec![],
        ));
        assert!(
            !html.contains(payload),
            "avatar::badge の data-scope/data-part 偽装ペイロードが出力に残っている: \
             payload={payload:?}, html={html}"
        );
        assert!(html.contains(r#"data-scope="avatar""#));
        assert!(html.contains(r#"data-part="badge""#));
    }
}

/// Command 経路（イシュー #2070）: 10 パーツいずれも見た目クラスを付与
/// しない（`src/command.rs` モジュール doc「軸を持たない理由」節参照）
/// ため、呼び出し側 `attrs`・`class`（`drop_class_attr` により除去）、
/// `input` の `value`/`list_id`/`activedescendant`、`list`/`dialog` の
/// `label`、`item` の `value`/`id`、`group`/`group_heading` の
/// `labelledby`/`id`、children の各経路で既定エスケープ（REQ-1）が
/// 貫通することを固定する。
#[test]
fn command_parts_are_escaped_for_all_payloads() {
    use fandhe_frontend_pre_styled_ui::command::{self, OpenState};

    for payload in payloads::all() {
        let html = render(&command::root(
            OpenState::Open,
            false,
            vec![("data-testid", payload)],
            vec![],
        ));
        assert_payload_is_escaped(payload, &html, "command::root attrs context");

        let html = render(&command::root(
            OpenState::Open,
            false,
            vec![("class", payload)],
            vec![],
        ));
        assert!(
            !html.contains(payload),
            "command::root class payload leaked: payload={payload:?}, html={html}"
        );
        assert_eq!(html.matches("class=\"").count(), 0);

        let html = render(&command::dialog(OpenState::Open, payload, vec![], vec![]));
        assert_payload_is_escaped(payload, &html, "command::dialog label context");

        let html = render(&command::input(
            OpenState::Open,
            payload,
            payload,
            Some(payload),
            vec![],
        ));
        assert_payload_is_escaped(payload, &html, "command::input value context");
        assert_payload_is_escaped(payload, &html, "command::input list_id context");
        assert_payload_is_escaped(payload, &html, "command::input activedescendant context");

        let html = render(&command::list(payload, payload, false, vec![], vec![]));
        assert_payload_is_escaped(payload, &html, "command::list id context");
        assert_payload_is_escaped(payload, &html, "command::list label context");

        let html = render(&command::empty(
            true,
            vec![("data-testid", payload)],
            vec![text(payload)],
        ));
        assert_payload_is_escaped(payload, &html, "command::empty attrs context");
        assert_payload_is_escaped(payload, &html, "command::empty children context");

        let html = render(&command::group(Some(payload), vec![], vec![]));
        assert_payload_is_escaped(payload, &html, "command::group labelledby context");

        let html = render(&command::group_heading(
            Some(payload),
            vec![],
            vec![text(payload)],
        ));
        assert_payload_is_escaped(payload, &html, "command::group_heading id context");
        assert_payload_is_escaped(payload, &html, "command::group_heading children context");

        let html = render(&command::item(
            false,
            false,
            payload,
            Some(payload),
            vec![("data-testid", payload)],
            vec![text(payload)],
        ));
        assert_payload_is_escaped(payload, &html, "command::item value context");
        assert_payload_is_escaped(payload, &html, "command::item id context");
        assert_payload_is_escaped(payload, &html, "command::item attrs context");
        assert_payload_is_escaped(payload, &html, "command::item children context");

        let html = render(&command::shortcut(
            vec![("data-testid", payload)],
            vec![text(payload)],
        ));
        assert_payload_is_escaped(payload, &html, "command::shortcut attrs context");
        assert_payload_is_escaped(payload, &html, "command::shortcut children context");

        let html = render(&command::separator(vec![("data-testid", payload)], vec![]));
        assert_payload_is_escaped(payload, &html, "command::separator attrs context");

        let html = render(&command::root(
            OpenState::Open,
            false,
            vec![("data-scope", payload), ("data-part", payload)],
            vec![],
        ));
        assert!(
            !html.contains(payload),
            "command::root data-scope/data-part spoof payload leaked: \
             payload={payload:?}, html={html}"
        );
        assert!(html.contains(r#"data-scope="command""#));
        assert!(html.contains(r#"data-part="root""#));
    }
}

/// styled Button Group `root`/`separator`/`text` 経路（イシュー #2060、親
/// #2058、headless anatomy は #2059）: 3 パーツいずれも見た目クラスを付与
/// しない（`src/button_group.rs` モジュール doc「variant 軸: 持たない」節
/// 参照）ため、呼び出し側 `attrs`・`class`（[`drop_class_attr`] により除去、
/// recipe クラスを持たないため `class` 属性自体が出力から消える）・`label`・
/// children の各経路で既定エスケープ（REQ-1）が貫通することを固定する
/// （`input_group_parts_are_escaped_for_all_payloads` と同型）。
#[test]
fn button_group_parts_are_escaped_for_all_payloads() {
    use fandhe_frontend_pre_styled_ui::button_group::{self, Orientation};

    for payload in payloads::all() {
        // styled root の label（aria-label へ出力される動的値）経路。
        let html = render(&button_group::root(
            Orientation::Horizontal,
            payload,
            vec![],
            vec![],
        ));
        assert_payload_is_escaped(payload, &html, "button_group::root label コンテキスト");

        // styled root の呼び出し側 attrs 経路。
        let html = render(&button_group::root(
            Orientation::Horizontal,
            "",
            vec![("data-testid", payload)],
            vec![],
        ));
        assert_payload_is_escaped(payload, &html, "button_group::root attrs コンテキスト");

        // styled root の呼び出し側 class 属性経路（見た目クラスを持たない
        // ため drop_class_attr により class 属性自体が出力から消える）。
        let html = render(&button_group::root(
            Orientation::Horizontal,
            "",
            vec![("class", payload)],
            vec![],
        ));
        assert!(
            !html.contains(payload),
            "button_group::root の class 属性に渡した生ペイロードが出力に残っている: \
             payload={payload:?}, html={html}"
        );
        assert_eq!(html.matches("class=\"").count(), 0);

        // styled separator の呼び出し側 attrs 経路。
        let html = render(&button_group::separator(
            Orientation::Horizontal,
            vec![("data-testid", payload)],
            vec![],
        ));
        assert_payload_is_escaped(payload, &html, "button_group::separator attrs コンテキスト");

        // styled text の呼び出し側 attrs・children 経路。
        let html = render(&button_group::text(
            vec![("data-testid", payload)],
            vec![text(payload)],
        ));
        assert_payload_is_escaped(payload, &html, "button_group::text attrs コンテキスト");
        assert_payload_is_escaped(payload, &html, "button_group::text children コンテキスト");
    }
}

/// styled Sidebar（イシュー #2073、親 #2071）: 22 パーツ + `menu_skeleton`
/// いずれも見た目クラスを付与しない（`src/sidebar.rs` モジュール doc
/// 「選択的 re-export」節参照）ため、呼び出し側 `attrs`・`class`
/// （`drop_class_attr` により除去）・`label`・`id`・`href`・`controls`・
/// `describedby`・`labelledby`・children の各経路で既定エスケープ
/// （REQ-1）が貫通することを固定する。`data-scope`/`data-part` 偽装が
/// headless `Anatomy::part` により除去されることもあわせて固定する
/// （`command_parts_are_escaped_for_all_payloads` と同型）。
#[test]
fn sidebar_parts_are_escaped_for_all_payloads() {
    use fandhe_frontend_pre_styled_ui::sidebar::{
        self, Sidebar, SidebarMenuButtonProps, SidebarMenuSubButtonProps, SidebarProps,
        SidebarState,
    };

    let state = Sidebar::new(SidebarState::Expanded);
    let props = SidebarProps::default();

    for payload in payloads::all() {
        let html = render(&sidebar::provider(
            &state,
            &props,
            vec![("data-testid", payload)],
            vec![],
        ));
        assert_payload_is_escaped(payload, &html, "sidebar::provider attrs context");

        let html = render(&sidebar::provider(
            &state,
            &props,
            vec![("class", payload)],
            vec![],
        ));
        assert!(
            !html.contains(payload),
            "sidebar::provider class payload leaked: payload={payload:?}, html={html}"
        );
        assert_eq!(html.matches("class=\"").count(), 0);

        let html = render(&sidebar::root(
            &state,
            &props,
            payload,
            Some(payload),
            vec![],
            vec![],
        ));
        assert_payload_is_escaped(payload, &html, "sidebar::root label context");
        assert_payload_is_escaped(payload, &html, "sidebar::root id context");

        let html = render(&sidebar::header(
            vec![("data-testid", payload)],
            vec![text(payload)],
        ));
        assert_payload_is_escaped(payload, &html, "sidebar::header attrs context");
        assert_payload_is_escaped(payload, &html, "sidebar::header children context");

        let html = render(&sidebar::content(vec![("data-testid", payload)], vec![]));
        assert_payload_is_escaped(payload, &html, "sidebar::content attrs context");

        let html = render(&sidebar::footer(vec![("data-testid", payload)], vec![]));
        assert_payload_is_escaped(payload, &html, "sidebar::footer attrs context");

        let html = render(&sidebar::separator(vec![("data-testid", payload)], vec![]));
        assert_payload_is_escaped(payload, &html, "sidebar::separator attrs context");

        let html = render(&sidebar::input(vec![("data-testid", payload)]));
        assert_payload_is_escaped(payload, &html, "sidebar::input attrs context");

        let html = render(&sidebar::group(Some(payload), vec![], vec![]));
        assert_payload_is_escaped(payload, &html, "sidebar::group labelledby context");

        let html = render(&sidebar::group_label(
            Some(payload),
            vec![],
            vec![text(payload)],
        ));
        assert_payload_is_escaped(payload, &html, "sidebar::group_label id context");
        assert_payload_is_escaped(payload, &html, "sidebar::group_label children context");

        let html = render(&sidebar::group_content(
            vec![("data-testid", payload)],
            vec![],
        ));
        assert_payload_is_escaped(payload, &html, "sidebar::group_content attrs context");

        let html = render(&sidebar::group_action(payload, vec![], vec![]));
        assert_payload_is_escaped(payload, &html, "sidebar::group_action label context");

        let html = render(&sidebar::menu(vec![("data-testid", payload)], vec![]));
        assert_payload_is_escaped(payload, &html, "sidebar::menu attrs context");

        let html = render(&sidebar::menu_item(vec![("data-testid", payload)], vec![]));
        assert_payload_is_escaped(payload, &html, "sidebar::menu_item attrs context");

        let html = render(&sidebar::menu_button(
            &SidebarMenuButtonProps {
                href: Some(payload),
                active: true,
                describedby: Some(payload),
                ..Default::default()
            },
            None,
            vec![("data-testid", payload)],
            vec![text(payload)],
        ));
        assert_payload_is_escaped(payload, &html, "sidebar::menu_button href context");
        assert_payload_is_escaped(payload, &html, "sidebar::menu_button describedby context");
        assert_payload_is_escaped(payload, &html, "sidebar::menu_button attrs context");
        assert_payload_is_escaped(payload, &html, "sidebar::menu_button children context");

        let html = render(&sidebar::menu_action(payload, vec![], vec![]));
        assert_payload_is_escaped(payload, &html, "sidebar::menu_action label context");

        let html = render(&sidebar::menu_badge(
            vec![("data-testid", payload)],
            vec![text(payload)],
        ));
        assert_payload_is_escaped(payload, &html, "sidebar::menu_badge attrs context");
        assert_payload_is_escaped(payload, &html, "sidebar::menu_badge children context");

        let html = render(&sidebar::menu_sub(vec![("data-testid", payload)], vec![]));
        assert_payload_is_escaped(payload, &html, "sidebar::menu_sub attrs context");

        let html = render(&sidebar::menu_sub_item(
            vec![("data-testid", payload)],
            vec![],
        ));
        assert_payload_is_escaped(payload, &html, "sidebar::menu_sub_item attrs context");

        let html = render(&sidebar::menu_sub_button(
            &SidebarMenuSubButtonProps {
                href: Some(payload),
                active: true,
                ..Default::default()
            },
            vec![("data-testid", payload)],
            vec![text(payload)],
        ));
        assert_payload_is_escaped(payload, &html, "sidebar::menu_sub_button href context");
        assert_payload_is_escaped(payload, &html, "sidebar::menu_sub_button attrs context");
        assert_payload_is_escaped(payload, &html, "sidebar::menu_sub_button children context");

        let html = render(&sidebar::rail(&state, payload, vec![], vec![]));
        assert_payload_is_escaped(payload, &html, "sidebar::rail label context");

        let html = render(&sidebar::trigger(
            &state,
            payload,
            Some(payload),
            vec![],
            vec![],
        ));
        assert_payload_is_escaped(payload, &html, "sidebar::trigger label context");
        assert_payload_is_escaped(payload, &html, "sidebar::trigger controls context");

        let html = render(&sidebar::inset(
            vec![("data-testid", payload)],
            vec![text(payload)],
        ));
        assert_payload_is_escaped(payload, &html, "sidebar::inset attrs context");
        assert_payload_is_escaped(payload, &html, "sidebar::inset children context");

        let html = render(&sidebar::menu_skeleton(
            true,
            vec![("data-testid", payload)],
        ));
        assert_payload_is_escaped(payload, &html, "sidebar::menu_skeleton attrs context");

        let html = render(&sidebar::provider(
            &state,
            &props,
            vec![("data-scope", payload), ("data-part", payload)],
            vec![],
        ));
        assert!(
            !html.contains(payload),
            "sidebar::provider data-scope/data-part spoof payload leaked: \
             payload={payload:?}, html={html}"
        );
        assert!(html.contains(r#"data-scope="sidebar""#));
        assert!(html.contains(r#"data-part="provider""#));
    }
}

/// navigation-menu の indicator（イシュー #2187 で新設したルートレベル
/// パーツ）を pre-styled 再エクスポート経由で描画し、`value`（`data-value`
/// へ透過）・呼び出し側 `attrs`・`children` の 3 経路が既定エスケープ
/// されることを固定する。`data_attr_vocabulary.rs` は headless anatomy
/// 由来の語彙（`.part("indicator"` 等）を対象としないため（pre-styled は
/// glob 再エクスポートのみで自身が anatomy を定義しない）、本ファイルが
/// XSS 回帰の唯一の固定先となる。
#[test]
fn navigation_menu_indicator_value_attrs_and_children_are_escaped_for_all_payloads() {
    let props = NavigationMenuProps::default();
    for payload in payloads::all() {
        let html = render(&navigation_menu::indicator(
            OpenState::Open,
            &props,
            Some(payload),
            vec![("data-testid", payload)],
            vec![text(payload)],
        ));
        assert_payload_is_escaped(
            payload,
            &html,
            "navigation_menu::indicator data-value context",
        );
        assert_payload_is_escaped(payload, &html, "navigation_menu::indicator attrs context");
        assert_payload_is_escaped(
            payload,
            &html,
            "navigation_menu::indicator children context",
        );
    }
}

/// Attachment 経路（イシュー #2112、headless 側 anatomy は #2111）: 8
/// パーツいずれも見た目クラスを付与しない（`src/attachment.rs` モジュール
/// doc「headless の `data-*` を参照する」節参照）ため、呼び出し側
/// `attrs`・`class`（[`drop_class_attr`] により除去）、`action` の
/// `label`（`aria-label` エスケープ）、children の各経路で既定エスケープ
/// （REQ-1）が貫通することを固定する（`bubble_parts_are_escaped_for_all_payloads`
/// と同型）。
#[test]
fn attachment_parts_are_escaped_for_all_payloads() {
    use fandhe_frontend_pre_styled_ui::attachment::{self, AttachmentRootProps};

    for payload in payloads::all() {
        // styled root の呼び出し側 attrs 経路。
        let html = render(&attachment::root(
            AttachmentRootProps::default(),
            vec![("data-testid", payload)],
            vec![],
        ));
        assert_payload_is_escaped(payload, &html, "attachment::root attrs コンテキスト");

        // styled root の呼び出し側 class 属性経路（見た目クラスを持たない
        // ため drop_class_attr により class 属性自体が出力から消える）。
        let html = render(&attachment::root(
            AttachmentRootProps::default(),
            vec![("class", payload)],
            vec![],
        ));
        assert!(
            !html.contains(payload),
            "attachment::root の class 属性に渡した生ペイロードが出力に\
             残っている: payload={payload:?}, html={html}"
        );
        assert_eq!(html.matches("class=\"").count(), 0);

        // styled media の呼び出し側 attrs・children 経路。
        let html = render(&attachment::media(
            vec![("data-testid", payload)],
            vec![text(payload)],
        ));
        assert_payload_is_escaped(payload, &html, "attachment::media attrs コンテキスト");
        assert_payload_is_escaped(payload, &html, "attachment::media children コンテキスト");

        // styled content の呼び出し側 attrs・children 経路。
        let html = render(&attachment::content(
            vec![("data-testid", payload)],
            vec![text(payload)],
        ));
        assert_payload_is_escaped(payload, &html, "attachment::content attrs コンテキスト");
        assert_payload_is_escaped(payload, &html, "attachment::content children コンテキスト");

        // styled name の呼び出し側 attrs・children 経路。
        let html = render(&attachment::name(
            vec![("data-testid", payload)],
            vec![text(payload)],
        ));
        assert_payload_is_escaped(payload, &html, "attachment::name attrs コンテキスト");
        assert_payload_is_escaped(payload, &html, "attachment::name children コンテキスト");

        // styled meta の呼び出し側 attrs・children 経路。
        let html = render(&attachment::meta(
            vec![("data-testid", payload)],
            vec![text(payload)],
        ));
        assert_payload_is_escaped(payload, &html, "attachment::meta attrs コンテキスト");
        assert_payload_is_escaped(payload, &html, "attachment::meta children コンテキスト");

        // styled progress の呼び出し側 attrs・children 経路。
        let html = render(&attachment::progress(
            vec![("data-testid", payload)],
            vec![text(payload)],
        ));
        assert_payload_is_escaped(payload, &html, "attachment::progress attrs コンテキスト");
        assert_payload_is_escaped(payload, &html, "attachment::progress children コンテキスト");

        // styled actions の呼び出し側 attrs・children 経路。
        let html = render(&attachment::actions(
            vec![("data-testid", payload)],
            vec![text(payload)],
        ));
        assert_payload_is_escaped(payload, &html, "attachment::actions attrs コンテキスト");
        assert_payload_is_escaped(payload, &html, "attachment::actions children コンテキスト");

        // styled action の label（aria-label）・attrs・children 経路。
        let html = render(&attachment::action(
            payload,
            false,
            vec![("data-testid", payload)],
            vec![text(payload)],
        ));
        assert_payload_is_escaped(payload, &html, "attachment::action label コンテキスト");
        assert_payload_is_escaped(payload, &html, "attachment::action attrs コンテキスト");
        assert_payload_is_escaped(payload, &html, "attachment::action children コンテキスト");
    }
}

/// Marker 経路（イシュー #2115、headless 側 anatomy は #2114）: 3 パーツ
/// いずれも見た目クラスを付与しない（`src/marker.rs` モジュール doc
/// 「headless の `data-*` を参照する」節参照）ため、呼び出し側 `attrs`・
/// `class`（[`drop_class_attr`] により除去）、children の各経路で既定
/// エスケープ（REQ-1）が貫通することを固定する
/// （`attachment_parts_are_escaped_for_all_payloads` と同型）。加えて
/// `Label` 形態で挟み込む [`crate::separator::separator`] を経由しても
/// attrs・children ペイロードがエスケープされることを 1 ケース固定する。
#[test]
fn marker_parts_are_escaped_for_all_payloads() {
    use fandhe_frontend_pre_styled_ui::marker::{self, MarkerRootProps, MarkerVariant};

    for payload in payloads::all() {
        // styled root（Note 形態、separator を挟まない）の呼び出し側
        // attrs 経路。
        let html = render(&marker::root(
            MarkerRootProps::default(),
            vec![("data-testid", payload)],
            vec![],
        ));
        assert_payload_is_escaped(payload, &html, "marker::root attrs コンテキスト");

        // styled root の呼び出し側 class 属性経路（見た目クラスを持たない
        // ため drop_class_attr により class 属性自体が出力から消える。
        // Note 形態は separator を挟まないため class 完全不在で判定する）。
        let html = render(&marker::root(
            MarkerRootProps::default(),
            vec![("class", payload)],
            vec![],
        ));
        assert!(
            !html.contains(payload),
            "marker::root の class 属性に渡した生ペイロードが出力に残って\
             いる: payload={payload:?}, html={html}"
        );
        assert_eq!(html.matches("class=\"").count(), 0);

        // styled icon の呼び出し側 attrs・children 経路。
        let html = render(&marker::icon(
            vec![("data-testid", payload)],
            vec![text(payload)],
        ));
        assert_payload_is_escaped(payload, &html, "marker::icon attrs コンテキスト");
        assert_payload_is_escaped(payload, &html, "marker::icon children コンテキスト");

        // styled content の呼び出し側 attrs・children 経路。
        let html = render(&marker::content(
            vec![("data-testid", payload)],
            vec![text(payload)],
        ));
        assert_payload_is_escaped(payload, &html, "marker::content attrs コンテキスト");
        assert_payload_is_escaped(payload, &html, "marker::content children コンテキスト");

        // Label 形態: separator 挟み込み後も root の attrs・children
        // ペイロードがエスケープされることを固定する（モジュール doc
        // 「区切り線の描画方式」節参照）。
        let label_html = render(&marker::root(
            MarkerRootProps {
                variant: MarkerVariant::Label,
                ..Default::default()
            },
            vec![("data-testid", payload)],
            vec![marker::content(vec![], vec![text(payload)])],
        ));
        assert_payload_is_escaped(
            payload,
            &label_html,
            "marker::root (Label) attrs コンテキスト",
        );
        assert_payload_is_escaped(
            payload,
            &label_html,
            "marker::root (Label) children コンテキスト",
        );
    }
}
