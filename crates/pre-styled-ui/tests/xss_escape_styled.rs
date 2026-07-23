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
use fandhe_frontend_pre_styled_ui::alert::{self, AlertStatus};
use fandhe_frontend_pre_styled_ui::badge::{badge, BadgeProps};
use fandhe_frontend_pre_styled_ui::button::{button, ButtonProps};
use fandhe_frontend_pre_styled_ui::card::{self, CardVariant};
use fandhe_frontend_pre_styled_ui::checkbox::{self, CheckboxProps};
use fandhe_frontend_pre_styled_ui::input::{self, FieldIds, FieldProps, InputProps};
use fandhe_frontend_pre_styled_ui::native_select::{self, NativeSelectProps};
use fandhe_frontend_pre_styled_ui::spinner::{spinner, SpinnerProps};
use fandhe_frontend_pre_styled_ui::textarea::{self, TextareaProps};
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
            CardVariant::default(),
            vec![("aria-label", payload)],
            vec![],
        ));
        assert_payload_is_escaped(payload, &html, "card::root 呼び出し側 attrs コンテキスト");

        let html = render(&alert::root(
            AlertStatus::default(),
            vec![("data-testid", payload)],
            vec![],
        ));
        assert_payload_is_escaped(payload, &html, "alert::root 呼び出し側 attrs コンテキスト");
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
            CardVariant::default(),
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
            CardVariant::default(),
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
        let html = render(&accordion::root(Size::Md, vec![("class", payload)], vec![]));
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
            vec![("class", payload)],
            vec![],
        ));
        assert!(
            !html.contains(payload),
            "select::root の class 属性に渡した生ペイロードが出力に残っている: payload={payload:?}, html={html}"
        );
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(html.contains("fd-select--"));
    }
}

/// (7) 属性値経路 e（イシュー #729）: 呼び出し側 `attrs` の `data-testid` 等が
/// `size` variant root 経由でも既定エスケープを経由することを固定する。
#[test]
fn size_variant_root_caller_attrs_are_escaped_for_all_payloads() {
    for payload in payloads::all() {
        let html = render(&accordion::root(
            Size::Md,
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
            vec![("data-testid", payload)],
            vec![],
        ));
        assert_payload_is_escaped(payload, &html, "select::root 呼び出し側 attrs コンテキスト");
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
