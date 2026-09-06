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
use fandhe_frontend_pre_styled_ui::badge::{badge, BadgeProps};
use fandhe_frontend_pre_styled_ui::blockquote::{self, BlockquoteVariant};
use fandhe_frontend_pre_styled_ui::button::{button, close_button, icon_button, ButtonProps};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::charts::data::{ChartData, Series};
use fandhe_frontend_pre_styled_ui::charts::radar_chart::{self, RadarChartProps};
use fandhe_frontend_pre_styled_ui::charts::scatter_chart::{
    self, ScatterChartProps, ScatterData, ScatterSeries,
};
use fandhe_frontend_pre_styled_ui::checkbox::{self, CheckboxProps};
use fandhe_frontend_pre_styled_ui::checkbox_card;
use fandhe_frontend_pre_styled_ui::clipboard;
use fandhe_frontend_pre_styled_ui::collapsible;
use fandhe_frontend_pre_styled_ui::date_input::{self, DateInputProps, DateSegment};
use fandhe_frontend_pre_styled_ui::donut_chart::{donut_chart, DonutChartProps};
use fandhe_frontend_pre_styled_ui::download_trigger::{self, DownloadTriggerProps};
use fandhe_frontend_pre_styled_ui::drawer::{self, DrawerPlacement};
use fandhe_frontend_pre_styled_ui::editable::{
    self, EditMode, EditableInputFlags, EditableInputProps,
};
use fandhe_frontend_pre_styled_ui::em::em;
use fandhe_frontend_pre_styled_ui::empty_state::{self, EmptyStateProps};
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
use fandhe_frontend_pre_styled_ui::list::{self, ListType, ListVariant};
use fandhe_frontend_pre_styled_ui::listbox;
use fandhe_frontend_pre_styled_ui::mark::{mark, MarkProps};
use fandhe_frontend_pre_styled_ui::marquee::{self, MarqueeProps};
use fandhe_frontend_pre_styled_ui::native_select::{self, NativeSelectProps};
use fandhe_frontend_pre_styled_ui::number_input::{self, NumberInputFlags};
use fandhe_frontend_pre_styled_ui::pagination::{self, ItemMode};
use fandhe_frontend_pre_styled_ui::password_input::{
    self, PasswordAutocomplete, PasswordInputProps,
};
use fandhe_frontend_pre_styled_ui::pie_chart::{pie_chart, PieChartProps};
use fandhe_frontend_pre_styled_ui::pin_input::{self, PinInputKind, PinInputProps};
use fandhe_frontend_pre_styled_ui::qr_code;
use fandhe_frontend_pre_styled_ui::quote::quote;
use fandhe_frontend_pre_styled_ui::radio_card;
use fandhe_frontend_pre_styled_ui::rating_group::{self, RatingItemFlags};
use fandhe_frontend_pre_styled_ui::scroll_area;
use fandhe_frontend_pre_styled_ui::separator::{separator, SeparatorProps};
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

        let html = render(&card::body(vec![], vec![text(payload)]));
        assert_payload_is_escaped(payload, &html, "card::body children コンテキスト");

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

        let html = render(&alert::root(
            &AlertProps::default(),
            vec![("data-testid", payload)],
            vec![],
        ));
        assert_payload_is_escaped(payload, &html, "alert::root 呼び出し側 attrs コンテキスト");

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

/// (9) pin_input 経路（イシュー #739）: styled `root` の呼び出し側 `attrs`・
/// `class`、および headless-ui から選択的再エクスポートした `label` の
/// children・`input` の `value`・`hidden_input` の `name`/`value` の 5 箇所
/// すべてで既定エスケープ（REQ-1）が貫通することを固定する
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
        let html = render(&carousel::prev_trigger(false, payload, vec![], vec![]));
        assert_payload_is_escaped(
            payload,
            &html,
            "carousel::prev_trigger aria-label コンテキスト",
        );

        // 選択的再エクスポートした item の children 経路。
        let html = render(&carousel::item(0, 1, false, vec![], vec![text(payload)]));
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
    }
}

/// (11) progress 経路（circle 対応、イシュー #763）: styled `root` の
/// `aria_valuetext` 引数・呼び出し側 `attrs`・`class`、および headless
/// `Progress` の inherent メソッド（`circle`/`circle_track`/`circle_range`。
/// styled 層の独自ラッパーを持たず headless をそのまま呼ぶ契約、
/// `crates/pre-styled-ui/src/progress.rs` rustdoc 参照）の呼び出し側
/// `attrs` すべてで既定エスケープ（REQ-1）が貫通することを固定する。
#[test]
fn progress_styled_root_and_headless_circle_parts_are_escaped_for_all_payloads() {
    use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::progress::Progress;
    use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::Orientation;
    use fandhe_frontend_pre_styled_ui::progress::{self, ProgressProps};

    let p = Progress::new(0.0, 100.0, Some(40.0), Orientation::Horizontal);

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
    use fandhe_frontend_pre_styled_ui::kbd::{kbd, KbdProps};
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

/// styled ScrollArea（イシュー #825）の headless 再エクスポート経路（attrs
/// breakout・children `<script>` ペイロード）がエスケープされることを固定する。
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
