//! `fandhe-frontend-headless-ui` の XSS 回帰テスト（イシュー #553、REQ-1）。
//!
//! 本クレートは `data-*`/ARIA ヘルパ単体の回帰（`tests/helpers_escape.rs`）を
//! 既に持つが、実際にコンポーネントを組み立てたときの (1) テキスト経路、
//! (2) 属性値経路、(3) URL 属性経路の 3 つを横断してエスケープ保証が
//! 貫通することを固定するテストが存在しなかった。本ファイルはその欠落を
//! 埋める（`docs/spec/06-roadmap.md` MS-1〜MS-5・REQ-1 の受け入れ基準の
//! headless-ui 層への拡張）。
//!
//! ペイロード集合は `crates/core/tests/xss_escape.rs` の `mod payloads`・
//! `crates/interactive/tests/xss_escape.rs` の先例に倣い、OWASP XSS
//! Prevention Cheat Sheet Rule #1 系の脅威パターンを核として本ファイル内に
//! 再定義する（クレート境界をまたいでテストコードを共有しない既存方針、
//! `crates/core/tests/xss_escape.rs` 冒頭コメント参照）。
//!
//! 本クレートは [`fandhe_frontend_core::render`] への薄い委譲層であり、
//! 独自のエスケープ処理を持たない（`crates/headless-ui/src/lib.rs` 冒頭の
//! 不変条件 2 参照）。したがって本ファイルの各テストは「委譲先である
//! `render()` の既定エスケープが、コンポーネント呼び出しを経由しても
//! 最後まで貫通する」ことを固定する契約検証であり、headless-ui 自体が
//! 新たなエスケープロジックを実装しているわけではない。
//!
//! # 削除・弱体化の禁止
//!
//! `.claude/rules/coding-rust.md` の規約により、本ファイルの XSS 回帰
//! テストは以後の削除・弱体化・`#[ignore]` 化を禁止する。

use fandhe_frontend_core::{escape_html, render, text};
use fandhe_frontend_headless_ui::calendar;
use fandhe_frontend_headless_ui::date::{PlainDate, Weekday};
use fandhe_frontend_headless_ui::date_picker;
use fandhe_frontend_headless_ui::file_upload;
use fandhe_frontend_headless_ui::positioning::{Align, Placement, Side};
use fandhe_frontend_headless_ui::qr_code;
use fandhe_frontend_headless_ui::scroll_area;
use fandhe_frontend_headless_ui::tour::{self, TourStep};
use fandhe_frontend_headless_ui::{
    action_bar, aria_controls, aria_label, avatar, carousel, clipboard, color_picker, data_state,
    date_input, dialog, download_trigger, editable, floating_panel, hover_card, image_cropper,
    listbox, number_input, password_input, pin_input, popover, rating_group, segment_group,
    signature_pad, slider, splitter, tags_input, timer, toast, tree_view, Calendar, DatePicker,
    DateSegmentFlags, ImageStatus, OpenState, Orientation, PasswordAutocomplete,
    PasswordInputProps, Steps, ToastStatus, Tour,
};

/// OWASP XSS Prevention Cheat Sheet Rule #1 系の共有ペイロード集合。
///
/// `crates/core/tests/xss_escape.rs::payloads` と観点を揃えるが、クレート
/// 境界をまたいで共有しない既存方針（同ファイル冒頭コメント）に従い、
/// 本ファイル内で独立に定義する。
mod payloads {
    /// タグ注入。
    pub const SCRIPT_TAG: &str = "<script>alert('xss')</script>";
    /// イベントハンドラ属性つきタグ注入。
    pub const IMG_ONERROR: &str = "<img src=x onerror=alert(1)>";
    /// 二重引用符属性値からの breakout。
    pub const DOUBLE_QUOTE_BREAKOUT: &str = "\"><script>alert(1)</script>";
    /// 単一引用符属性値からの breakout（イベントハンドラ注入込み）。
    pub const SINGLE_QUOTE_BREAKOUT: &str = "' onmouseover='alert(1)";
    /// コンテキスト脱出系（閉じタグによる親コンテキスト離脱）。
    pub const CONTEXT_BREAKOUT: &str = "</title><script>alert(1)</script>";
    /// 非 ASCII 混在文字列（マルチバイト透過の確認）。
    pub const NON_ASCII_MIXED: &str = "こんにちは<script>alert(1)</script>世界";

    /// 全ペイロードをまとめて返す（網羅的にループ検証する用途）。
    pub fn all() -> Vec<&'static str> {
        vec![
            SCRIPT_TAG,
            IMG_ONERROR,
            DOUBLE_QUOTE_BREAKOUT,
            SINGLE_QUOTE_BREAKOUT,
            CONTEXT_BREAKOUT,
            NON_ASCII_MIXED,
        ]
    }
}

/// (1) テキスト経路 + (2) 属性値経路の共通アサーション。
///
/// (a) [`escape_html`] が返す正解のエスケープ済み表現が出力に実在すること
///     （render() が内容ごと出力しなくなる偽陰性リグレッションの検知）、
/// (b) 生ペイロードが部分文字列として出力に現れないこと、
/// (c) `<script>` / `<img` の実タグ開始が出力に現れないこと、の 3 点を
/// 見る（`crates/core/tests/xss_escape.rs::csr::assert_fragment_is_safe` と同型）。
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
        !html.contains("<script>") && !html.contains("<img"),
        "{context_label}で実タグとしての <script>/<img> が出力に出現している: html={html}"
    );
}

/// (1) テキスト経路: [`dialog::title`] の children テキストへ全ペイロードを
/// 注入し、エスケープが貫通することを固定する。
#[test]
fn dialog_title_children_text_is_escaped_for_all_payloads() {
    for payload in payloads::all() {
        let node = dialog::title(None, vec![], vec![text(payload)]);
        let html = render(&node);
        assert_payload_is_escaped(payload, &html, "dialog::title のテキストコンテキスト");
    }
}

/// (2) 属性値経路: `aria_label`/`aria_controls`（WAI-ARIA ヘルパ）・
/// `data_state`（データ属性ヘルパ）・呼び出し側 `attrs` の 4 系統へ属性
/// breakout 系ペイロードを注入し、エスケープが貫通することを固定する。
#[test]
fn attribute_value_paths_are_escaped_for_all_payloads() {
    for payload in payloads::all() {
        let aria_label_node = dialog::title(None, vec![aria_label(payload)], vec![]);
        let html = render(&aria_label_node);
        assert_payload_is_escaped(payload, &html, "aria_label 属性値コンテキスト");

        let aria_controls_node = dialog::title(None, vec![aria_controls(payload)], vec![]);
        let html = render(&aria_controls_node);
        assert_payload_is_escaped(payload, &html, "aria_controls 属性値コンテキスト");

        let data_state_node = dialog::title(None, vec![data_state(payload)], vec![]);
        let html = render(&data_state_node);
        assert_payload_is_escaped(payload, &html, "data_state 属性値コンテキスト");

        // 呼び出し側 attrs（例: data-testid）経由の注入。
        let caller_attrs_node = dialog::title(None, vec![("data-testid", payload)], vec![]);
        let html = render(&caller_attrs_node);
        assert_payload_is_escaped(payload, &html, "呼び出し側 attrs 属性値コンテキスト");
    }
}

/// (3) URL 属性経路: [`avatar::image`] の唯一の URL 属性 `src` へ、
/// `fandhe_frontend_core::url` モジュールと同一系の危険 URL スキームを渡し、
/// `render()` の許可リスト方式（deny by default）が headless-ui コンポーネント
/// 経由でも貫通することを固定する。
///
/// core の `render_into`（`crates/core/src/lib.rs`）は `is_url_attr` に該当する
/// 属性値が `is_safe_url` 不合格のとき、属性そのものをスキップして出力する
/// （`src="..."` が丸ごと出力から消える）。headless-ui 層は URL スキーム検証を
/// 行わない（`crates/headless-ui/src/avatar.rs` 冒頭コメント参照）が、これは
/// core 層の `render()` がこの契約を保証する前提に基づく意図的な設計であり、
/// 本テストはその契約が avatar::image の呼び出し経路でも成立することを
/// 確認する。
#[test]
fn avatar_image_src_rejects_dangerous_url_schemes() {
    let dangerous_urls = [
        "javascript:alert(1)",
        "JaVaScRiPt:alert(1)",
        "java\tscript:alert(1)",
        "java\nscript:alert(1)",
        "\u{0}javascript:alert(1)",
        "data:text/html;base64,PHNjcmlwdD4=",
        "vbscript:msgbox(1)",
    ];

    for url in dangerous_urls {
        let node = avatar::image(ImageStatus::Loaded, url, "safe alt text", vec![]);
        let html = render(&node);
        assert!(
            !html.contains("src="),
            "危険な URL スキームなのに src 属性が出力されている: url={url:?}, html={html}"
        );
        // 兄弟属性（alt 等）は影響を受けず出力され続けることを確認する。
        assert!(
            html.contains(r#"alt="safe alt text""#),
            "src 属性の拒否によって兄弟属性 alt まで欠落している: html={html}"
        );
    }
}

/// (2) 属性値経路（style、イシュー #590 anchor positioning 追加分）:
/// [`popover::positioner`] の `attrs` 経由で渡す `style` 属性値（ADR §4.4 の
/// CSS 変数出力経路）へ属性境界脱出（`"` breakout）系ペイロードを注入し、
/// エスケープが貫通することを固定する。
///
/// `positioning::css_vars_style` 自体は内部生成の数値書式のみを組み立てる
/// ため実運用でこの経路にユーザー入力が流れることはないが、`attrs` 引数は
/// 型上 `&str` を受け取れる（既存の動的値契約、ADR §4.4）ため、`style`
/// 属性という経路自体が既定エスケープを迂回しないことを回帰として固定する。
#[test]
fn positioner_style_attr_payload_is_escaped_for_all_payloads() {
    for payload in payloads::all() {
        let node = popover::positioner(OpenState::Open, vec![("style", payload)], vec![]);
        let html = render(&node);
        assert_payload_is_escaped(payload, &html, "positioner の style 属性値コンテキスト");
    }
}

/// (1)/(2) NumberInput（イシュー #738）: `name`（属性値経路）と
/// `label` の children（テキスト経路）へ全ペイロードを注入し、エスケープが
/// 貫通することを固定する。
#[test]
fn number_input_name_and_label_children_are_escaped_for_all_payloads() {
    for payload in payloads::all() {
        let input_node = number_input::input(
            payload,
            None,
            None,
            "0",
            "100",
            number_input::NumberInputFlags::default(),
            vec![],
        );
        let html = render(&input_node);
        assert_payload_is_escaped(
            payload,
            &html,
            "number_input::input の name 属性値コンテキスト",
        );

        let label_node = number_input::label(
            number_input::NumberInputFlags::default(),
            None,
            vec![],
            vec![text(payload)],
        );
        let html = render(&label_node);
        assert_payload_is_escaped(payload, &html, "number_input::label のテキストコンテキスト");

        // ValueText パーツ（イシュー #1613）: children のテキスト経路。
        let value_text_node = number_input::value_text(
            number_input::NumberInputFlags::default(),
            vec![],
            vec![text(payload)],
        );
        let html = render(&value_text_node);
        assert_payload_is_escaped(
            payload,
            &html,
            "number_input::value_text のテキストコンテキスト",
        );
    }
}

/// (1)/(2) RatingGroup（イシュー #742）: `hidden_input` の `name`（属性値
/// 経路）と `label` の children（テキスト経路）へ全ペイロードを注入し、
/// エスケープが貫通することを固定する（`number_input` 分と同型）。
#[test]
fn rating_group_name_and_label_children_are_escaped_for_all_payloads() {
    for payload in payloads::all() {
        let props = rating_group::RatingGroupProps::default();
        let hidden_input_node = rating_group::hidden_input(&props, Some(payload), "3", vec![]);
        let html = render(&hidden_input_node);
        assert_payload_is_escaped(
            payload,
            &html,
            "rating_group::hidden_input の name 属性値コンテキスト",
        );

        let label_node = rating_group::label(&props, None, vec![], vec![text(payload)]);
        let html = render(&label_node);
        assert_payload_is_escaped(payload, &html, "rating_group::label のテキストコンテキスト");
    }
}

/// (1)/(2) SegmentGroup（イシュー #743）: `item_hidden_input` の `name`/
/// `value`（属性値経路）と `item_text` の children（テキスト経路）へ全
/// ペイロードを注入し、エスケープが貫通することを固定する
/// （`radio_group` の対応テストと同型、責務は委譲だが anatomy 出力経路は
/// 本モジュール固有のため個別に固定する）。
#[test]
fn segment_group_name_value_and_item_text_children_are_escaped_for_all_payloads() {
    for payload in payloads::all() {
        let input_node =
            segment_group::item_hidden_input(false, false, Some(payload), payload, vec![]);
        let html = render(&input_node);
        assert_payload_is_escaped(
            payload,
            &html,
            "segment_group::item_hidden_input の name/value 属性値コンテキスト",
        );

        let item_text_node = segment_group::item_text(false, false, vec![], vec![text(payload)]);
        let html = render(&item_text_node);
        assert_payload_is_escaped(
            payload,
            &html,
            "segment_group::item_text のテキストコンテキスト",
        );
    }
}

/// (1)/(2) Slider（イシュー #741）: `hidden_input` の `name`（属性値経路）・
/// `label` の children（テキスト経路）・`thumb` の `aria-valuetext`（属性値
/// 経路）へ全ペイロードを注入し、エスケープが貫通することを固定する。
#[test]
fn slider_name_label_and_valuetext_are_escaped_for_all_payloads() {
    for payload in payloads::all() {
        let hidden_input_node = slider::hidden_input(payload, "40", false, vec![]);
        let html = render(&hidden_input_node);
        assert_payload_is_escaped(
            payload,
            &html,
            "slider::hidden_input の name 属性値コンテキスト",
        );

        let label_node = slider::label(vec![], vec![text(payload)]);
        let html = render(&label_node);
        assert_payload_is_escaped(payload, &html, "slider::label のテキストコンテキスト");

        let thumb_node = slider::thumb(
            Orientation::Horizontal,
            "0",
            "100",
            "40",
            Some(payload),
            false,
            vec![],
            vec![],
        );
        let html = render(&thumb_node);
        assert_payload_is_escaped(
            payload,
            &html,
            "slider::thumb の aria-valuetext 属性値コンテキスト",
        );
    }
}

/// (1)/(2)/(3) ImageCropper（イシュー #844、シグネチャはイシュー #1610 で
/// `ImageCropperProps` 追加に追随）: `image` の `src`/`alt`（属性値
/// 経路）・`selection` の children（テキスト経路）・`root`/`handle` の
/// 呼び出し側 `attrs`（属性値経路）へ全ペイロードを注入し、エスケープが
/// 貫通することを固定する。
#[test]
fn image_cropper_src_alt_children_and_attrs_are_escaped_for_all_payloads() {
    let props = image_cropper::ImageCropperProps::default();
    let state = image_cropper::ImageCropper::default();
    for payload in payloads::all() {
        // `image_cropper::image` は `<img>` タグ自体を出力するため、
        // 実タグ出現の有無を見る `assert_payload_is_escaped` の共通チェック
        // は使えない（`avatar::image` の src 検証と同じ事情、本ファイル
        // `avatar_image_src_rejects_dangerous_url_schemes` 参照）。ここでは
        // エスケープ済み表現の実在・生ペイロードの不在のみを個別に確認する。
        let image_node = image_cropper::image(payload, payload, vec![]);
        let html = render(&image_node);
        let expected_escaped = escape_html(payload);
        assert!(
            html.contains(&expected_escaped),
            "image_cropper::image の src/alt 属性値コンテキストで期待される\
             エスケープ済み表現が出力に見当たらない: payload={payload:?}, html={html}"
        );
        assert!(
            !html.contains(payload),
            "image_cropper::image の src/alt 属性値コンテキストで生ペイロード\
             が出力にそのまま残っている: payload={payload:?}, html={html}"
        );

        let selection_node = image_cropper::selection(&state, &props, vec![], vec![text(payload)]);
        let html = render(&selection_node);
        assert_payload_is_escaped(
            payload,
            &html,
            "image_cropper::selection のテキストコンテキスト",
        );

        let root_node = image_cropper::root(&props, vec![("data-testid", payload)], vec![]);
        let html = render(&root_node);
        assert_payload_is_escaped(
            payload,
            &html,
            "image_cropper::root の attrs 属性値コンテキスト",
        );

        let handle_node = image_cropper::handle(
            image_cropper::HandlePosition::Se,
            &props,
            vec![("data-testid", payload)],
        );
        let html = render(&handle_node);
        assert_payload_is_escaped(
            payload,
            &html,
            "image_cropper::handle の attrs 属性値コンテキスト",
        );
    }
}

/// (1)/(2) Carousel（イシュー #754）: `root`/`prev_trigger` の `aria-label`
/// （属性値経路）・`item` の children（テキスト経路）へ全ペイロードを注入し、
/// エスケープが貫通することを固定する。
#[test]
fn carousel_aria_label_and_item_children_are_escaped_for_all_payloads() {
    for payload in payloads::all() {
        let root_node = carousel::root(Orientation::Horizontal, payload, vec![], vec![]);
        let html = render(&root_node);
        assert_payload_is_escaped(
            payload,
            &html,
            "carousel::root の aria-label 属性値コンテキスト",
        );

        let prev_trigger_node = carousel::prev_trigger(false, payload, vec![], vec![]);
        let html = render(&prev_trigger_node);
        assert_payload_is_escaped(
            payload,
            &html,
            "carousel::prev_trigger の aria-label 属性値コンテキスト",
        );

        let item_node = carousel::item(0, 1, false, vec![], vec![text(payload)]);
        let html = render(&item_node);
        assert_payload_is_escaped(payload, &html, "carousel::item のテキストコンテキスト");
    }
}

/// (3) URL 属性経路: 安全な URL（相対パス・https）は `src="..."` として
/// 透過することを固定する（deny by default が過剰側に倒れて安全な値まで
/// 落としていないことの確認。陽性・陰性の両建て、`assert_payload_is_escaped`
/// と同じ vacuous pass 防止の考え方）。
#[test]
fn avatar_image_src_passes_through_safe_urls() {
    let safe_urls = ["/avatars/1.png", "https://example.com/a.png", "./rel.png"];

    for url in safe_urls {
        let node = avatar::image(ImageStatus::Loaded, url, "avatar", vec![]);
        let html = render(&node);
        let expected = format!(r#"src="{}""#, escape_html(url));
        assert!(
            html.contains(&expected),
            "安全な URL が src 属性として透過していない: url={url:?}, html={html}"
        );
    }
}

/// (1)(2) テキスト・属性値経路（イシュー #740 追加分）: `password_input` の
/// `id`（派生属性値へ伝播）・`label` の children テキストへ全ペイロードを
/// 注入し、エスケープが貫通することを固定する。あわせて `value=` が
/// いかなる経路でも出力に現れないこと（本コンポーネントのセキュリティ
/// 不変条件、`crates/headless-ui/src/password_input.rs` モジュール doc 参照）
/// も同時に確認する。
#[test]
fn password_input_id_and_label_text_are_escaped_for_all_payloads() {
    for payload in payloads::all() {
        let field_props = PasswordInputProps {
            id: payload,
            disabled: false,
            readonly: false,
            invalid: false,
            required: false,
            autocomplete: PasswordAutocomplete::CurrentPassword,
        };
        let html = render(&password_input::label(&field_props, vec![], vec![]));
        assert_payload_is_escaped(
            payload,
            &html,
            "password_input::label の for 属性値コンテキスト",
        );
        assert!(!html.contains("value="));

        let default_props = PasswordInputProps {
            id: "pw",
            disabled: false,
            readonly: false,
            invalid: false,
            required: false,
            autocomplete: PasswordAutocomplete::CurrentPassword,
        };
        let label_node = password_input::label(&default_props, vec![], vec![text(payload)]);
        let html = render(&label_node);
        assert_payload_is_escaped(
            payload,
            &html,
            "password_input::label のテキストコンテキスト",
        );
        assert!(!html.contains("value="));
    }
}

/// (2) 属性値経路（イシュー #739 PinInput）: `pin_input::hidden_input` の
/// `name`/`value`・`pin_input::input` の `value`・呼び出し側 `attrs` へ全
/// ペイロードを注入し、エスケープが貫通することを固定する。`aria-label`
/// は `format!` で組み立てた動的文字列（`crates/headless-ui/src/pin_input.rs`
/// 参照）だが本テストの対象ではなく専用の inline テスト
/// （`pin_input::tests`）で固定済みのため、ここでは `name`/`value`/`attrs`
/// の 3 経路のみを扱う。
#[test]
fn pin_input_hidden_input_and_input_value_are_escaped_for_all_payloads() {
    use fandhe_frontend_headless_ui::pin_input::{PinInputKind, PinInputProps};

    let props = PinInputProps::default();
    for payload in payloads::all() {
        let hidden_node = pin_input::hidden_input(payload, payload, false, vec![]);
        let html = render(&hidden_node);
        assert_payload_is_escaped(
            payload,
            &html,
            "pin_input::hidden_input の name/value コンテキスト",
        );

        let input_node = pin_input::input(
            0,
            1,
            payload,
            PinInputKind::Alphanumeric,
            false,
            false,
            &props,
            false,
            vec![],
        );
        let html = render(&input_node);
        assert_payload_is_escaped(payload, &html, "pin_input::input の value コンテキスト");

        let attrs_node = pin_input::root(false, &props, vec![("data-testid", payload)], vec![]);
        let html = render(&attrs_node);
        assert_payload_is_escaped(
            payload,
            &html,
            "pin_input::root の呼び出し側 attrs コンテキスト",
        );
    }
}

/// (1)/(2) Editable（イシュー #745）: `input` の `name`/`value`（属性値
/// 経路）・`label`/`preview` の children（テキスト経路）・呼び出し側
/// `attrs` へ全ペイロードを注入し、エスケープが貫通することを固定する。
#[test]
fn editable_name_value_label_and_preview_are_escaped_for_all_payloads() {
    use fandhe_frontend_headless_ui::editable::{EditMode, EditableInputFlags, EditableInputProps};

    for payload in payloads::all() {
        let input_node = editable::input(
            EditMode::Edit,
            payload,
            payload,
            EditableInputProps::default(),
            EditableInputFlags::default(),
            vec![],
        );
        let html = render(&input_node);
        assert_payload_is_escaped(payload, &html, "editable::input の name/value コンテキスト");

        let label_node = editable::label(
            EditMode::Preview,
            EditableInputFlags::default(),
            None,
            vec![],
            vec![text(payload)],
        );
        let html = render(&label_node);
        assert_payload_is_escaped(payload, &html, "editable::label のテキストコンテキスト");

        let preview_node = editable::preview(
            EditMode::Preview,
            EditableInputFlags::default(),
            false,
            vec![],
            vec![text(payload)],
        );
        let html = render(&preview_node);
        assert_payload_is_escaped(payload, &html, "editable::preview のテキストコンテキスト");

        let attrs_node = editable::root(
            EditMode::Preview,
            EditableInputFlags::default(),
            Default::default(),
            Default::default(),
            vec![("data-testid", payload)],
            vec![],
        );
        let html = render(&attrs_node);
        assert_payload_is_escaped(
            payload,
            &html,
            "editable::root の呼び出し側 attrs コンテキスト",
        );
    }
}

/// (1) テキスト経路 + (2) 属性値経路（イシュー #744 TagsInput）:
/// タグ文字列そのものがユーザー入力である（REQ-1 の重点対象）ため、
/// `tags_input::item_text` の children テキスト・`tags_input::hidden_input`
/// の `name`/`value`・`tags_input::item_input` の `value`・
/// `tags_input::item_delete_trigger` の `tag`（`format!` で組み立てる
/// `aria-label` の一部）・呼び出し側 `attrs` へ全ペイロードを注入し、
/// エスケープが貫通することを固定する。
#[test]
fn tags_input_tag_text_and_attribute_paths_are_escaped_for_all_payloads() {
    for payload in payloads::all() {
        let item_text_node = tags_input::item_text(vec![], vec![text(payload)]);
        let html = render(&item_text_node);
        assert_payload_is_escaped(
            payload,
            &html,
            "tags_input::item_text の children コンテキスト",
        );

        let hidden_node = tags_input::hidden_input(payload, payload, false, vec![]);
        let html = render(&hidden_node);
        assert_payload_is_escaped(
            payload,
            &html,
            "tags_input::hidden_input の name/value コンテキスト",
        );

        let item_input_node = tags_input::item_input(payload, vec![]);
        let html = render(&item_input_node);
        assert_payload_is_escaped(
            payload,
            &html,
            "tags_input::item_input の value コンテキスト",
        );

        let delete_trigger_node = tags_input::item_delete_trigger(payload, false, vec![], vec![]);
        let html = render(&delete_trigger_node);
        assert_payload_is_escaped(
            payload,
            &html,
            "tags_input::item_delete_trigger の aria-label コンテキスト",
        );

        let attrs_node = tags_input::root(false, vec![("data-testid", payload)], vec![]);
        let html = render(&attrs_node);
        assert_payload_is_escaped(
            payload,
            &html,
            "tags_input::root の呼び出し側 attrs コンテキスト",
        );
    }
}

/// (1) テキスト経路 + (2) 属性値経路（イシュー #840 FileUpload）:
/// ファイル名・MIME 文字列は攻撃者が完全に制御可能な入力そのもの（REQ-1 の
/// 重点対象）であるため、`file_upload::item_name` の children テキスト・
/// `file_upload::item_delete_trigger` の `name`（`format!` で組み立てる
/// `aria-label` の一部）・`file_upload::hidden_input` の `accept` 属性・
/// 呼び出し側 `attrs` へ全ペイロードを注入し、エスケープが貫通することを
/// 固定する（`tags_input` 分と同型の網羅方針）。
#[test]
fn file_upload_item_name_and_attribute_paths_are_escaped_for_all_payloads() {
    let props = file_upload::FileUploadProps::default();
    for payload in payloads::all() {
        let item_name_node = file_upload::item_name(
            file_upload::ItemType::Accepted,
            &props,
            vec![],
            vec![text(payload)],
        );
        let html = render(&item_name_node);
        assert_payload_is_escaped(
            payload,
            &html,
            "file_upload::item_name の children コンテキスト",
        );

        let item_size_text_node = file_upload::item_size_text_node(
            file_upload::ItemType::Accepted,
            &props,
            vec![],
            vec![text(payload)],
        );
        let html = render(&item_size_text_node);
        assert_payload_is_escaped(
            payload,
            &html,
            "file_upload::item_size_text_node の children コンテキスト",
        );

        let delete_trigger_node = file_upload::item_delete_trigger(
            payload,
            file_upload::ItemType::Accepted,
            &props,
            vec![],
            vec![],
        );
        let html = render(&delete_trigger_node);
        assert_payload_is_escaped(
            payload,
            &html,
            "file_upload::item_delete_trigger の aria-label コンテキスト",
        );

        let hidden_input_node = file_upload::hidden_input(payload, false, &props, vec![]);
        let html = render(&hidden_input_node);
        assert_payload_is_escaped(
            payload,
            &html,
            "file_upload::hidden_input の accept 属性コンテキスト",
        );

        let attrs_node = file_upload::root(&props, false, vec![("data-testid", payload)], vec![]);
        let html = render(&attrs_node);
        assert_payload_is_escaped(
            payload,
            &html,
            "file_upload::root の呼び出し側 attrs コンテキスト",
        );
    }
}

/// (1)/(2) Steps（イシュー #752）: `item`/`content` の children（テキスト
/// 経路）と呼び出し側 `attrs`（属性値経路）へ全ペイロードを注入し、
/// エスケープが貫通することを固定する。
#[test]
fn steps_item_content_children_and_attrs_are_escaped_for_all_payloads() {
    let s = Steps::new(3, 1, Orientation::Horizontal);
    for payload in payloads::all() {
        let item_node = s.item(0, vec![], vec![text(payload)]);
        let html = render(&item_node);
        assert_payload_is_escaped(payload, &html, "steps::item のテキストコンテキスト");

        let content_node = s.content(1, vec![], vec![text(payload)]);
        let html = render(&content_node);
        assert_payload_is_escaped(payload, &html, "steps::content のテキストコンテキスト");

        let trigger_attrs_node = s.trigger(0, vec![("data-testid", payload)], vec![]);
        let html = render(&trigger_attrs_node);
        assert_payload_is_escaped(
            payload,
            &html,
            "steps::trigger の呼び出し側 attrs コンテキスト",
        );
    }
}

/// (1) テキスト経路 + (2) 属性値経路（イシュー #753 TreeView）:
/// ノードラベル（`branch_text`/`item_text` の children）・ノード値
/// （`branch`/`item` の `data-value`）・呼び出し側 `attrs` へ全ペイロードを
/// 注入し、エスケープが貫通することを固定する。TreeView は木構造全体を
/// [`fandhe_frontend_headless_ui::TreeView::render_nodes`] で組み立てる
/// ため、`TreeNode` のラベル・値へペイロードを埋め込んだ木を実際に描画して
/// 検証する（`tags_input` 分と同型の網羅方針）。
#[test]
fn tree_view_node_label_and_value_paths_are_escaped_for_all_payloads() {
    use fandhe_frontend_headless_ui::{TreeNode, TreeView};

    for payload in payloads::all() {
        // ラベル: branch_text/item_text の children テキスト経路。
        let branch_text_node = tree_view::branch_text(vec![], vec![text(payload)]);
        let html = render(&branch_text_node);
        assert_payload_is_escaped(
            payload,
            &html,
            "tree_view::branch_text の children コンテキスト",
        );

        let item_text_node = tree_view::item_text(vec![], vec![text(payload)]);
        let html = render(&item_text_node);
        assert_payload_is_escaped(
            payload,
            &html,
            "tree_view::item_text の children コンテキスト",
        );

        // ノード値: branch/item の data-value 属性経路。
        let branch_node = tree_view::branch(
            OpenState::Closed,
            payload,
            false,
            false,
            "1",
            "1",
            "1",
            "0",
            vec![],
            vec![],
        );
        let html = render(&branch_node);
        assert_payload_is_escaped(
            payload,
            &html,
            "tree_view::branch の data-value コンテキスト",
        );

        let item_node = tree_view::item(payload, false, false, "1", "1", "1", "0", vec![], vec![]);
        let html = render(&item_node);
        assert_payload_is_escaped(payload, &html, "tree_view::item の data-value コンテキスト");

        // 呼び出し側 attrs 経路。
        let attrs_node = tree_view::root(vec![("data-testid", payload)], vec![]);
        let html = render(&attrs_node);
        assert_payload_is_escaped(
            payload,
            &html,
            "tree_view::root の呼び出し側 attrs コンテキスト",
        );

        // TreeView::render_nodes 経由の全体組み立て（ラベル・値を両方汚染した
        // 木を実際に描画し、再帰ヘルパを経由してもエスケープが貫通することを
        // 固定する）。
        let nodes =
            vec![TreeNode::new(payload, payload)
                .with_children(vec![TreeNode::new(payload, payload)])];
        let rendered = TreeView::default().render_nodes(&nodes);
        let html = rendered.iter().map(render).collect::<Vec<_>>().join("");
        assert_payload_is_escaped(
            payload,
            &html,
            "TreeView::render_nodes の全体組み立てコンテキスト",
        );
    }
}

/// (4) dispatch payload → hydration 経路（イシュー #753 TreeView）:
/// クライアント由来の展開/選択 dispatch payload が改ざんされうる入力として
/// 扱われ、hydration 属性へ埋め込まれてもエスケープが貫通することを固定する
/// （`SingleSelect`/`MultiSelect` 単体の既存回帰を `TreeView` 合成経由でも
/// 固定する）。
#[test]
fn tree_view_dispatch_payload_is_escaped_in_hydration_output() {
    use fandhe_frontend_headless_ui::TreeView;
    use fandhe_frontend_interactive::{dispatch, render_for_hydration};

    let mut t = TreeView::default();
    let payload = "\"><script>alert(1)</script>";
    assert!(dispatch(&mut t, "expand", payload));
    assert!(dispatch(&mut t, "select", payload));

    let rendered = render(&render_for_hydration(&t));
    assert!(rendered.contains("data-hydrate-expanded="));
    assert!(rendered.contains("data-hydrate-selected="));
    assert!(rendered.contains("&lt;script&gt;"));
    assert!(!rendered.contains("<script>alert(1)</script>"));
    assert!(!rendered.contains(r#""><script"#));
}

/// (1)(2) テキスト経路 + 属性値経路（イシュー #829 JsonTreeView）:
/// オブジェクトキー・文字列値・JSON Pointer（`data-value`）の各コンテキストへ
/// 全ペイロードを注入し、[`json_tree_view::render_json`] 経由でもエスケープが
/// 貫通することを固定する（`tree_view` 分と同型の網羅方針）。
#[test]
fn json_tree_view_key_string_value_and_pointer_paths_are_escaped_for_all_payloads() {
    use fandhe_frontend_headless_ui::json_tree_view::{render_json, JsonValue, TreeView};

    for payload in payloads::all() {
        // キー: オブジェクトキーが key パーツの children テキスト経路へ乗る。
        let by_key = JsonValue::Object(vec![(payload.to_string(), JsonValue::Null)]);
        let html_by_key = render(&render_json(&TreeView::default(), &by_key));
        assert_payload_is_escaped(
            payload,
            &html_by_key,
            "json_tree_view::render_json のオブジェクトキー children コンテキスト",
        );

        // 文字列値: value パーツの children テキスト経路。
        let by_string_value = JsonValue::Object(vec![(
            "k".to_string(),
            JsonValue::String(payload.to_string()),
        )]);
        let html_by_string_value = render(&render_json(&TreeView::default(), &by_string_value));
        assert_payload_is_escaped(
            payload,
            &html_by_string_value,
            "json_tree_view::render_json の文字列値 children コンテキスト",
        );

        // JSON Pointer: オブジェクトキーが data-value（RFC 6901 ポインタ）の
        // セグメントとしても使われるため、by_key（payload をオブジェクトキーに
        // 使うケース）の html を再検証し、属性値コンテキストでのエスケープを
        // 直接固定する（by_string_value の html を誤って再利用しない）。
        assert_payload_is_escaped(
            payload,
            &html_by_key,
            "json_tree_view::render_json の data-value（JSON Pointer）コンテキスト",
        );
    }
}

/// (4) dispatch payload → hydration 経路（イシュー #829 JsonTreeView）:
/// [`json_tree_view::expanded_to_depth`]/`TreeView` 経由の展開・選択
/// dispatch payload（本モジュールでは RFC 6901 JSON Pointer 文字列）が
/// 改ざんされうる入力として扱われ、hydration 属性へ埋め込まれてもエスケープが
/// 貫通することを固定する（`tree_view` 分の回帰を JsonTreeView 経由でも固定）。
#[test]
fn json_tree_view_dispatch_payload_is_escaped_in_hydration_output() {
    use fandhe_frontend_headless_ui::json_tree_view::TreeView;
    use fandhe_frontend_interactive::{dispatch, render_for_hydration};

    let mut t = TreeView::default();
    let payload = "\"><script>alert(1)</script>";
    assert!(dispatch(&mut t, "expand", payload));
    assert!(dispatch(&mut t, "select", payload));

    let rendered = render(&render_for_hydration(&t));
    assert!(rendered.contains("data-hydrate-expanded="));
    assert!(rendered.contains("data-hydrate-selected="));
    assert!(rendered.contains("&lt;script&gt;"));
    assert!(!rendered.contains("<script>alert(1)</script>"));
    assert!(!rendered.contains(r#""><script"#));
}

/// Toast（イシュー #760）: group の `label`・root の title/description
/// children・`Toaster::push` した通知の title/description・呼び出し側 attrs
/// の各経路へペイロードを注入し、エスケープが貫通することを固定する。
#[test]
fn toast_label_title_description_and_view_are_escaped_for_all_payloads() {
    use fandhe_frontend_headless_ui::Toaster;
    use fandhe_frontend_interactive::Component;

    for payload in payloads::all() {
        let group_node = toast::group(
            fandhe_frontend_headless_ui::ToastPlacement::Bottom,
            payload,
            vec![],
            vec![],
        );
        let html = render(&group_node);
        assert_payload_is_escaped(payload, &html, "toast::group の aria-label コンテキスト");

        let root_node = toast::root(
            ToastStatus::Info,
            vec![("data-testid", payload)],
            vec![
                toast::title(vec![], vec![text(payload)]),
                toast::description(vec![], vec![text(payload)]),
            ],
        );
        let html = render(&root_node);
        assert_payload_is_escaped(payload, &html, "toast::root/title/description コンテキスト");

        let mut toaster = Toaster::new(5, fandhe_frontend_headless_ui::ToastPlacement::Bottom);
        toaster.push(fandhe_frontend_headless_ui::ToastEntry {
            id: "toast-1".to_string(),
            status: ToastStatus::Error,
            title: payload.to_string(),
            description: payload.to_string(),
        });
        let html = render(&toaster.view());
        assert_payload_is_escaped(payload, &html, "Toaster::view の全体組み立てコンテキスト");
    }
}

/// (1)/(2) ActionBar（イシュー #762）: `content` の `aria-label`（属性値経路）
/// ・`selection_trigger`/`close_trigger` の children（テキスト経路）へ全
/// ペイロードを注入し、エスケープが貫通することを固定する。
#[test]
fn action_bar_label_and_trigger_children_are_escaped_for_all_payloads() {
    for payload in payloads::all() {
        let content_node = action_bar::content(OpenState::Open, payload, vec![], vec![]);
        let html = render(&content_node);
        assert_payload_is_escaped(
            payload,
            &html,
            "action_bar::content の aria-label 属性値コンテキスト",
        );

        let selection_trigger_node = action_bar::selection_trigger(vec![], vec![text(payload)]);
        let html = render(&selection_trigger_node);
        assert_payload_is_escaped(
            payload,
            &html,
            "action_bar::selection_trigger のテキストコンテキスト",
        );

        let close_trigger_node = action_bar::close_trigger(vec![], vec![text(payload)]);
        let html = render(&close_trigger_node);
        assert_payload_is_escaped(
            payload,
            &html,
            "action_bar::close_trigger のテキストコンテキスト",
        );
    }
}

/// (2) 属性値経路 + (3) URL 属性経路（イシュー #759 HoverCard）:
/// [`hover_card::trigger`] の唯一の URL 属性 `href` へ全ペイロードを注入し
/// エスケープ貫通を、[`hover_card::content`] の `id` 属性へ全ペイロードを
/// 注入しエスケープ貫通を、それぞれ固定する。`href` は URL 属性のため
/// `render()` の許可リスト方式（`avatar_image_src_rejects_dangerous_url_schemes`
/// と同型の契約）も併せて確認する。
#[test]
fn hover_card_href_and_content_id_are_escaped_for_all_payloads() {
    for payload in payloads::all() {
        let trigger_node = hover_card::trigger(OpenState::Closed, Some(payload), vec![], vec![]);
        let html = render(&trigger_node);
        assert_payload_is_escaped(
            payload,
            &html,
            "hover_card::trigger の href 属性値コンテキスト",
        );

        let content_node = hover_card::content(OpenState::Open, Some(payload), vec![], vec![]);
        let html = render(&content_node);
        assert_payload_is_escaped(
            payload,
            &html,
            "hover_card::content の id 属性値コンテキスト",
        );
    }

    // URL 属性経路: javascript: スキームは href 属性ごと出力から除去される
    // （`crates/headless-ui/src/breadcrumb.rs` の同型契約を継承）。
    let dangerous = hover_card::trigger(
        OpenState::Closed,
        Some("javascript:alert(1)"),
        vec![],
        vec![],
    );
    let html = render(&dangerous);
    assert!(!html.contains("javascript:"));
    assert!(!html.contains("href="));
}

/// (1) テキスト経路（イシュー #776 VisuallyHidden）: [`visually_hidden::root`]
/// の子ノードへ全ペイロードを注入し、エスケープ貫通を固定する。
#[test]
fn visually_hidden_children_text_is_escaped_for_all_payloads() {
    use fandhe_frontend_headless_ui::fandhe_frontend_core::text;
    use fandhe_frontend_headless_ui::visually_hidden;

    for payload in payloads::all() {
        let node = visually_hidden::root(vec![], vec![text(payload)]);
        let html = render(&node);
        assert_payload_is_escaped(
            payload,
            &html,
            "visually_hidden::root の children テキストコンテキスト",
        );
    }
}

/// (2) 属性値経路（イシュー #773 Clipboard）: [`clipboard::root`] の
/// `data-value` 属性・[`clipboard::input`] の `value` 属性へ全ペイロードを
/// 注入し、エスケープ貫通を固定する。コピー対象値はパスワード等の機微情報を
/// 含みうるため、属性破りペイロードでも実タグ・属性破りが起きないことを
/// 特に固定する（`.claude/rules/security.md` A03 対応）。
#[test]
fn clipboard_root_data_value_and_input_value_are_escaped_for_all_payloads() {
    for payload in payloads::all() {
        let root_node = clipboard::root(payload, false, vec![], vec![]);
        let html = render(&root_node);
        assert_payload_is_escaped(
            payload,
            &html,
            "clipboard::root の data-value 属性値コンテキスト",
        );

        let input_node = clipboard::input(payload, false, vec![]);
        let html = render(&input_node);
        assert_payload_is_escaped(
            payload,
            &html,
            "clipboard::input の value 属性値コンテキスト",
        );
    }
}

/// (1) テキスト経路（イシュー #773 Clipboard）: [`clipboard::value_text`] の
/// children テキストへ全ペイロードを注入し、エスケープが貫通することを
/// 固定する。
#[test]
fn clipboard_value_text_children_are_escaped_for_all_payloads() {
    for payload in payloads::all() {
        let node = clipboard::value_text(vec![], vec![text(payload)]);
        let html = render(&node);
        assert_payload_is_escaped(
            payload,
            &html,
            "clipboard::value_text のテキストコンテキスト",
        );
    }
}

/// QrCode（イシュー #774）: `value`（符号化対象文字列）はマークアップへ
/// 一切出力されない契約（`crates/headless-ui/src/qr_code.rs`「セキュリティ
/// 不変条件」参照）を、敵対的ペイロード全量で固定する。[`qr_code::pattern`]
/// の `d` 属性値は暗モジュール座標から内部生成される固定文字集合
/// （`M`/`h`/`v`/`z`/`-`/半角数字/`,`）のみであることも合わせて確認する。
#[test]
fn qr_code_value_never_leaks_into_output_for_all_payloads() {
    for payload in payloads::all() {
        let matrix = qr_code::encode(payload, qr_code::ErrorCorrectionLevel::L)
            .expect("payload はいずれもバージョン 40 容量内に収まる");
        let frame_node = qr_code::frame(&matrix, qr_code::DEFAULT_QUIET_ZONE, None, vec![], vec![]);
        let pattern_node = qr_code::pattern(&matrix, qr_code::DEFAULT_QUIET_ZONE, vec![]);
        let html = format!("{}{}", render(&frame_node), render(&pattern_node));

        assert!(
            !html.contains(payload),
            "QrCode の value が出力へ漏出している: payload={payload:?}, html={html}"
        );
        assert!(
            !html.contains("<script>") && !html.contains("<img"),
            "QrCode 出力に実タグとしての <script>/<img> が出現している: html={html}"
        );

        let d_start = html.find(r#" d=""#).expect("d 属性が出力される") + 4;
        let d_end = html[d_start..].find('"').expect("d 属性値の終端");
        let d_value = &html[d_start..d_start + d_end];
        assert!(
            d_value
                .chars()
                .all(|c| matches!(c, 'M' | 'h' | 'v' | 'z' | '-' | ',' | '0'..='9')),
            "d 属性値に想定外の文字が含まれている: d_value={d_value:?}"
        );
    }
}

/// (1)/(2) テキスト経路 + 属性値経路（イシュー #776 SkipNav）:
/// [`skip_nav::link`]/[`skip_nav::content`] の `id`（href/id 属性へ合成される）
/// と children へ全ペイロードを注入し、エスケープ貫通を固定する。
#[test]
fn skip_nav_id_and_children_are_escaped_for_all_payloads() {
    use fandhe_frontend_headless_ui::fandhe_frontend_core::text;
    use fandhe_frontend_headless_ui::skip_nav;

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
    }
}

/// QrCode の [`qr_code::root`]/[`qr_code::frame`]/[`qr_code::overlay`] は
/// 他 anatomy パーツと同型に呼び出し側 `attrs`/`children` を
/// [`fandhe_frontend_core::render`] の既定エスケープ経由で出力する
/// （属性値経路・テキスト経路）。
#[test]
fn qr_code_root_attrs_and_overlay_children_are_escaped_for_all_payloads() {
    for payload in payloads::all() {
        let root_node = qr_code::root(vec![("aria-label", payload)], vec![]);
        let html = render(&root_node);
        assert_payload_is_escaped(
            payload,
            &html,
            "qr_code::root の aria-label 属性値コンテキスト",
        );

        let overlay_node = qr_code::overlay(vec![], vec![text(payload)]);
        let html = render(&overlay_node);
        assert_payload_is_escaped(
            payload,
            &html,
            "qr_code::overlay の children テキストコンテキスト",
        );
    }
}

/// Listbox（イシュー #750）の XSS 回帰: [`listbox::item`] の `value`（`data-value`
/// 属性）・children テキスト・`id`・[`listbox::content`] の `labelledby`/
/// `activedescendant`・[`listbox::value_text`] の children テキスト・
/// hydration dispatch payload（`data-hydrate-selected` へ全ペイロードを注入し、
/// エスケープが貫通することを固定する。
#[test]
fn listbox_item_text_value_id_and_hydration_paths_are_escaped_for_all_payloads() {
    use fandhe_frontend_headless_ui::listbox::ListboxProps;
    let props = ListboxProps::default();
    for payload in payloads::all() {
        let item_node = listbox::item(
            OpenState::Open,
            &props,
            false,
            false,
            payload,
            Some(payload),
            vec![],
            vec![],
        );
        let html = render(&item_node);
        assert_payload_is_escaped(
            payload,
            &html,
            "listbox::item の data-value/id コンテキスト",
        );

        let item_text_node = listbox::item_text(
            OpenState::Open,
            &listbox::ListboxProps::default(),
            false,
            false,
            Some(payload),
            vec![],
            vec![text(payload)],
        );
        let html = render(&item_text_node);
        assert_payload_is_escaped(
            payload,
            &html,
            "listbox::item_text の id/children コンテキスト",
        );

        let content_node = listbox::content(
            false,
            &props,
            Some(payload),
            Some(payload),
            Some(payload),
            vec![],
            vec![],
        );
        let html = render(&content_node);
        assert_payload_is_escaped(
            payload,
            &html,
            "listbox::content の id/labelledby/activedescendant コンテキスト",
        );

        let value_text_node = listbox::value_text(false, &props, vec![], vec![text(payload)]);
        let html = render(&value_text_node);
        assert_payload_is_escaped(
            payload,
            &html,
            "listbox::value_text の children コンテキスト",
        );
    }
}

/// Listbox の dispatch payload（クライアント由来の改ざんされうる選択値）が
/// hydration 属性へエンコードされたのち `render()` を経由してもエスケープが
/// 貫通することを固定する（`crates/interactive` の `HYDRATE_ATTR_PREFIX` +
/// `codec::encode_list` 経由で `data-hydrate-selected` へ乗る値）。
#[test]
fn listbox_dispatch_select_payload_is_escaped_on_hydration_render() {
    use fandhe_frontend_headless_ui::listbox::Listbox;
    use fandhe_frontend_interactive::{dispatch, render_for_hydration};

    for payload in payloads::all() {
        let mut l = Listbox::default();
        assert!(dispatch(&mut l, "select", payload));
        let html = render(&render_for_hydration(&l));
        assert_payload_is_escaped(
            payload,
            &html,
            "Listbox dispatch select payload の data-hydrate-selected コンテキスト",
        );
    }
}

/// (1)/(2) Splitter（イシュー #826）: `panel` の `id`（属性値経路）・
/// `resize_trigger` の `aria-controls`（属性値経路）・
/// `resize_trigger_indicator` の children（テキスト経路）へ全ペイロードを
/// 注入し、エスケープが貫通することを固定する。
#[test]
fn splitter_panel_id_and_resize_trigger_controls_are_escaped_for_all_payloads() {
    for payload in payloads::all() {
        let panel_node = splitter::panel(payload, Orientation::Horizontal, vec![], vec![]);
        let html = render(&panel_node);
        assert_payload_is_escaped(payload, &html, "splitter::panel の id 属性値コンテキスト");

        let resize_trigger_node = splitter::resize_trigger(
            Orientation::Horizontal,
            "0",
            "100",
            "50",
            payload,
            false,
            vec![],
            vec![],
        );
        let html = render(&resize_trigger_node);
        assert_payload_is_escaped(
            payload,
            &html,
            "splitter::resize_trigger の aria-controls 属性値コンテキスト",
        );

        let indicator_node = splitter::resize_trigger_indicator(vec![], vec![text(payload)]);
        let html = render(&indicator_node);
        assert_payload_is_escaped(
            payload,
            &html,
            "splitter::resize_trigger_indicator のテキストコンテキスト",
        );
    }
}

/// FloatingPanel（イシュー #827）: (2) 属性値経路（`trigger` の `controls`、
/// `content` の `id`/`labelledby`）と (1) テキスト経路（`title` children）へ
/// 全ペイロードを注入し、エスケープが貫通することを固定する。
#[test]
fn floating_panel_controls_id_labelledby_and_title_children_are_escaped_for_all_payloads() {
    for payload in payloads::all() {
        let trigger_html = render(&floating_panel::trigger(
            OpenState::Closed,
            false,
            Some(payload),
            vec![],
            vec![],
        ));
        assert_payload_is_escaped(
            payload,
            &trigger_html,
            "floating_panel::trigger の controls コンテキスト",
        );

        let content_html = render(&floating_panel::content(
            OpenState::Open,
            floating_panel::Stage::Default,
            Some(payload),
            Some(payload),
            vec![],
            vec![],
        ));
        assert_payload_is_escaped(
            payload,
            &content_html,
            "floating_panel::content の id/labelledby コンテキスト",
        );

        let title_html = render(&floating_panel::title(None, vec![], vec![text(payload)]));
        assert_payload_is_escaped(
            payload,
            &title_html,
            "floating_panel::title の children コンテキスト",
        );
    }
}

/// FloatingPanel の dispatch payload（クライアント由来の改ざんされうる
/// `set_position` の x,y 文字列）が hydration 属性へエンコードされたのち
/// `render()` を経由してもエスケープが貫通することを固定する
/// （`FloatingPanel::position_style` 自体は内部生成の数値書式のみを扱うが、
/// `positioner` の `attrs` 引数は型上 `&str` を受け取れるため、経路自体が
/// 既定エスケープを迂回しないことを回帰として固定する。`popover` の
/// `positioner_style_attr_payload_is_escaped_for_all_payloads` と同型）。
#[test]
fn floating_panel_positioner_style_attr_payload_is_escaped_for_all_payloads() {
    for payload in payloads::all() {
        let node = floating_panel::positioner(
            OpenState::Open,
            floating_panel::Stage::Default,
            vec![("style", payload)],
            vec![],
        );
        let html = render(&node);
        assert_payload_is_escaped(
            payload,
            &html,
            "floating_panel::positioner の style 属性値コンテキスト",
        );
    }
}

/// DownloadTrigger（イシュー #828）の XSS 回帰: [`download_trigger::root`] の
/// `href`（URL 属性経路。危険スキームは fail-closed で `href` 自体が出力
/// されない）・`file_name`（`download` 属性値経路）・children テキストの
/// 3 経路でエスケープが貫通することを固定する。
#[test]
fn download_trigger_root_href_file_name_and_children_are_escaped() {
    for payload in payloads::all() {
        let html = render(&download_trigger::root(
            payload,
            None,
            vec![],
            vec![text(payload)],
        ));
        if !html.contains("href=") {
            // 危険スキーム（javascript:/data: 等）は core の render() が
            // fail-closed で href 属性ごと出力しない。URL 属性経路として
            // 「エスケープされず貫通した」ことにはならないため
            // assert_payload_is_escaped の対象から除外する（`link.rs`
            // `dangerous_url_schemes_are_rejected` と同じ整理）。
        } else {
            assert_payload_is_escaped(
                payload,
                &html,
                "download_trigger::root の href コンテキスト",
            );
        }

        let html = render(&download_trigger::root(
            "/assets/report.pdf",
            Some(payload),
            vec![],
            vec![],
        ));
        assert_payload_is_escaped(
            payload,
            &html,
            "download_trigger::root の download（file_name）コンテキスト",
        );

        let html = render(&download_trigger::root(
            "/assets/report.pdf",
            None,
            vec![],
            vec![text(payload)],
        ));
        assert_payload_is_escaped(
            payload,
            &html,
            "download_trigger::root の children コンテキスト",
        );
    }
}

/// Splitter の dispatch `"set"` payload（クライアント由来の改ざんされうる
/// 入力）が hydration 属性へエンコードされたのち `render()` を経由しても
/// エスケープが貫通することを固定する（`data-hydrate-sizes` 等へは正規化
/// 済みの数値のみが乗るため直接ペイロードは現れないが、`orientation` は
/// dispatch を経由しない属性のため hydration ラウンドトリップ全体として
/// 不正値を混入させないことを併せて確認する）。
#[test]
fn splitter_dispatch_set_payload_is_rejected_or_escaped_on_hydration_render() {
    use fandhe_frontend_headless_ui::splitter::Splitter;
    use fandhe_frontend_interactive::{dispatch, render_for_hydration};

    for payload in payloads::all() {
        let mut s = Splitter::default();
        // 不正な payload は fail-closed に no-op となり、状態を汚染しない。
        let _ = dispatch(&mut s, "set", payload);
        let html = render(&render_for_hydration(&s));
        assert!(
            !html.contains("<script"),
            "payload={payload:?} が hydration 出力に script タグとして混入した: {html}"
        );
    }
}

/// ScrollArea（イシュー #825）の属性 breakout・children `<script>` ペイロード
/// がエスケープされることを固定する。
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

/// (1)/(2) ColorPicker（イシュー #839）: `channel_input`/`hidden_input` の
/// 属性値経路・`area_thumb` の `aria-valuetext` 属性値経路・`label` の
/// テキスト経路へ全ペイロードを注入し、エスケープが貫通することを固定する。
#[test]
fn color_picker_attrs_valuetext_and_children_are_escaped_for_all_payloads() {
    let props = color_picker::ColorPickerProps::default();
    for payload in payloads::all() {
        let html = render(&color_picker::channel_input(payload, &props, vec![]));
        assert_payload_is_escaped(
            payload,
            &html,
            "color_picker::channel_input の value 属性値コンテキスト",
        );

        let html = render(&color_picker::hidden_input(
            payload,
            "#ffffff",
            &props,
            vec![],
        ));
        assert_payload_is_escaped(
            payload,
            &html,
            "color_picker::hidden_input の name 属性値コンテキスト",
        );

        let html = render(&color_picker::area_thumb(payload, &props, vec![], vec![]));
        assert_payload_is_escaped(
            payload,
            &html,
            "color_picker::area_thumb の aria-valuetext 属性値コンテキスト",
        );

        let html = render(&color_picker::label(&props, vec![], vec![text(payload)]));
        assert_payload_is_escaped(payload, &html, "color_picker::label のテキストコンテキスト");
    }
}

/// (3) ColorPicker: hydration 復元経路（`data-hydrate-h`）へ XSS payload を
/// 渡しても復元に失敗し（`HydrateError`）、値がそのまま出力に混入しない
/// ことを固定する（`slider`/`json_tree_view` の hydration payload 回帰と
/// 同型）。
#[test]
fn color_picker_hydration_xss_payload_is_rejected_not_rendered() {
    use fandhe_frontend_interactive::Hydrate;
    for payload in payloads::all() {
        let attrs = vec![
            ("data-hydrate-h".to_string(), payload.to_string()),
            ("data-hydrate-s".to_string(), "0".to_string()),
            ("data-hydrate-v".to_string(), "0".to_string()),
            ("data-hydrate-a".to_string(), "255".to_string()),
            ("data-hydrate-state".to_string(), "closed".to_string()),
        ];
        assert!(color_picker::ColorPicker::from_hydration_attrs(&attrs).is_err());
    }
}

/// DateInput（イシュー #834）: (2) 属性値経路（`hidden_input` の
/// `name`/`value`・呼び出し側 `attrs`）が全ペイロードでエスケープされる
/// ことを固定する（`pin_input_hidden_input_and_input_value_are_escaped_for_all_payloads`
/// と同型）。
#[test]
fn date_input_hidden_input_name_value_and_attrs_are_escaped_for_all_payloads() {
    for payload in payloads::all() {
        let hidden_node = date_input::hidden_input(payload, payload, false, vec![]);
        let html = render(&hidden_node);
        assert_payload_is_escaped(
            payload,
            &html,
            "date_input::hidden_input の name/value コンテキスト",
        );

        let attrs_node = date_input::root(false, false, vec![("data-testid", payload)], vec![]);
        let html = render(&attrs_node);
        assert_payload_is_escaped(
            payload,
            &html,
            "date_input::root の呼び出し側 attrs コンテキスト",
        );
    }
}

/// DateInput: (1) テキスト経路（[`date_input::label`] の children）が
/// 全ペイロードでエスケープされることを固定する。
#[test]
fn date_input_label_children_text_is_escaped_for_all_payloads() {
    for payload in payloads::all() {
        let node = date_input::label(false, false, None, vec![], vec![text(payload)]);
        let html = render(&node);
        assert_payload_is_escaped(payload, &html, "date_input::label のテキストコンテキスト");
    }
}

/// DateInput: dispatch `"set-segment"`/`"set"` の payload はクライアント
/// 由来の信頼できない入力として fail-closed にパース・拒否される
/// （`splitter_dispatch_set_payload_is_rejected_or_escaped_on_hydration_render`
/// と同型）。不正 payload は no-op のため hydration 出力に script タグとして
/// 混入しないことを併せて確認する。
#[test]
fn date_input_dispatch_set_segment_and_set_payload_is_rejected_on_hydration_render() {
    use fandhe_frontend_headless_ui::date_input::DateInput;
    use fandhe_frontend_interactive::{dispatch, render_for_hydration};

    for payload in payloads::all() {
        let mut d = DateInput::default();
        let set_segment_payload = format!("year:{payload}");
        let _ = dispatch(&mut d, "set-segment", &set_segment_payload);
        let _ = dispatch(&mut d, "set", payload);
        let html = render(&render_for_hydration(&d));
        assert!(
            !html.contains("<script"),
            "payload={payload:?} が hydration 出力に script タグとして混入した: {html}"
        );
    }
}

/// DateInput: [`date_input::segment`] の `attrs` 経由の属性値注入が
/// エスケープされることを固定する（`DateSegmentFlags` 経由のパーツ関数も
/// 他コンポーネントと同じ既定エスケープ委譲を維持することの確認）。
#[test]
fn date_input_segment_attrs_payload_is_escaped_for_all_payloads() {
    for payload in payloads::all() {
        let node = date_input::segment(
            fandhe_frontend_headless_ui::date_input::DateSegment::Year,
            None,
            "0",
            "9999",
            DateSegmentFlags::default(),
            vec![("data-testid", payload)],
        );
        let html = render(&node);
        assert_payload_is_escaped(payload, &html, "date_input::segment の attrs コンテキスト");
    }
}

/// Calendar（イシュー #835）の `heading` children・`day_trigger` の `id`
/// 属性・呼び出し側 `attrs` の 3 経路がエスケープされることを固定する。
#[test]
fn calendar_heading_children_and_day_trigger_attrs_are_escaped_for_all_payloads() {
    let d = PlainDate::new(2026, 7, 22).unwrap();
    for payload in payloads::all() {
        let html = render(&calendar::heading(None, vec![], vec![text(payload)]));
        assert_payload_is_escaped(payload, &html, "calendar::heading の children コンテキスト");

        let html = render(&calendar::day_trigger(
            d,
            false,
            false,
            false,
            false,
            Some(payload),
            vec![],
            vec![],
        ));
        assert_payload_is_escaped(
            payload,
            &html,
            "calendar::day_trigger の id 属性コンテキスト",
        );

        let html = render(&calendar::root(vec![("data-testid", payload)], vec![]));
        assert_payload_is_escaped(payload, &html, "calendar::root の attrs コンテキスト");
    }
}

/// Calendar の dispatch `"select"` payload（クライアント由来の改ざんされうる
/// 入力）は `PlainDate::parse_iso` の fail-closed 検証を通らない限り状態を
/// 変更しないため、hydration 出力へ生ペイロードが混入しないことを固定する。
#[test]
fn calendar_dispatch_select_payload_is_rejected_or_escaped_on_hydration_render() {
    use fandhe_frontend_interactive::{dispatch, render_for_hydration};

    for payload in payloads::all() {
        let mut cal = Calendar::new(
            2026,
            7,
            PlainDate::new(2026, 7, 1).unwrap(),
            None,
            None,
            None,
            Weekday::Monday,
        )
        .unwrap();
        let _ = dispatch(&mut cal, "select", payload);
        let html = render(&render_for_hydration(&cal));
        assert!(
            !html.contains("<script"),
            "payload={payload:?} が hydration 出力に script タグとして混入した: {html}"
        );
    }
}

/// DatePicker（イシュー #835）の `trigger`/`input` の属性経路がエスケープ
/// されることを固定する。
#[test]
fn date_picker_trigger_and_input_attrs_are_escaped_for_all_payloads() {
    for payload in payloads::all() {
        let html = render(&date_picker::trigger(
            OpenState::Closed,
            false,
            Some(payload),
            vec![],
            vec![],
        ));
        assert_payload_is_escaped(
            payload,
            &html,
            "date_picker::trigger の controls 属性コンテキスト",
        );

        let html = render(&date_picker::input(Some(payload), false, None, vec![]));
        assert_payload_is_escaped(
            payload,
            &html,
            "date_picker::input の value 属性コンテキスト",
        );
    }
}

/// DatePicker の dispatch `"select"` payload が hydration 出力へ生のまま
/// 混入しないことを固定する（[`calendar_dispatch_select_payload_is_rejected_or_escaped_on_hydration_render`]
/// と同型。DatePicker は Calendar を埋め込むため同じ fail-closed 経路を
/// 継承する）。
#[test]
fn date_picker_dispatch_select_payload_is_rejected_or_escaped_on_hydration_render() {
    use fandhe_frontend_interactive::{dispatch, render_for_hydration};

    for payload in payloads::all() {
        let calendar = Calendar::new(
            2026,
            7,
            PlainDate::new(2026, 7, 1).unwrap(),
            None,
            None,
            None,
            Weekday::Monday,
        )
        .unwrap();
        let mut dp = DatePicker::new(calendar);
        let _ = dispatch(&mut dp, "select", payload);
        let html = render(&render_for_hydration(&dp));
        assert!(
            !html.contains("<script"),
            "payload={payload:?} が hydration 出力に script タグとして混入した: {html}"
        );
    }
}

/// Timer（イシュー #836）の children テキスト経路・呼び出し側 attrs 経路の
/// エスケープ貫通を固定する。Timer の `data-*` 属性はすべて `u64`/固定
/// 語彙（`TimerUnit`/`TimerControl`/`TimerPhase`）から整形されるため任意
/// 文字列の混入経路がなく、動的テキストが混入しうるのは children
/// （[`timer::item_value`]/[`timer::item_label`]/[`timer::action_trigger`]
/// の呼び出し側 children）と呼び出し側が明示的に渡す `attrs` のみである。
#[test]
fn timer_children_and_attrs_are_escaped_for_all_payloads() {
    for payload in payloads::all() {
        let html = render(&timer::item_value(
            timer::TimerUnit::Seconds,
            vec![],
            vec![text(payload)],
        ));
        assert_payload_is_escaped(payload, &html, "timer::item_value のテキストコンテキスト");

        let html = render(&timer::item_label(
            timer::TimerUnit::Hours,
            vec![],
            vec![text(payload)],
        ));
        assert_payload_is_escaped(payload, &html, "timer::item_label のテキストコンテキスト");

        let html = render(&timer::action_trigger(
            timer::TimerControl::Start,
            vec![],
            vec![text(payload)],
        ));
        assert_payload_is_escaped(
            payload,
            &html,
            "timer::action_trigger のテキストコンテキスト",
        );

        let html = render(&timer::root(
            false,
            0,
            0,
            1000,
            0,
            timer::TimerPhase::Idle,
            vec![("data-testid", payload)],
            vec![],
        ));
        assert_payload_is_escaped(payload, &html, "timer::root の attrs コンテキスト");
    }
}

/// SignaturePad（イシュー #843）: (1) テキスト経路（[`signature_pad::label`]
/// の children）が全ペイロードでエスケープされることを固定する
/// （`date_input_label_children_text_is_escaped_for_all_payloads` と同型）。
#[test]
fn signature_pad_label_children_text_is_escaped_for_all_payloads() {
    for payload in payloads::all() {
        let node = signature_pad::label(vec![], vec![text(payload)]);
        let html = render(&node);
        assert_payload_is_escaped(
            payload,
            &html,
            "signature_pad::label のテキストコンテキスト",
        );
    }
}

/// SignaturePad: (2) 属性値経路（[`signature_pad::segment`] の
/// `aria_label_text`・[`signature_pad::hidden_input`] の `name`/`value`・
/// 呼び出し側 `attrs`）が全ペイロードでエスケープされることを固定する
/// （`date_input_hidden_input_name_value_and_attrs_are_escaped_for_all_payloads`
/// と同型）。
#[test]
fn signature_pad_hidden_input_name_value_and_segment_aria_label_are_escaped_for_all_payloads() {
    for payload in payloads::all() {
        let hidden_node = signature_pad::hidden_input(payload, payload, false, vec![]);
        let html = render(&hidden_node);
        assert_payload_is_escaped(
            payload,
            &html,
            "signature_pad::hidden_input の name/value コンテキスト",
        );

        let segment_node = signature_pad::segment(300, 150, Some(payload), vec![], vec![]);
        let html = render(&segment_node);
        assert_payload_is_escaped(
            payload,
            &html,
            "signature_pad::segment の aria_label_text コンテキスト",
        );

        let attrs_node = signature_pad::root(false, true, vec![("data-testid", payload)], vec![]);
        let html = render(&attrs_node);
        assert_payload_is_escaped(
            payload,
            &html,
            "signature_pad::root の呼び出し側 attrs コンテキスト",
        );
    }
}

/// (1)/(2) Tour（イシュー #841）: `title`/`description` の children
/// （テキスト経路）・`target`（`spotlight` の `data-target` 属性値経路）・
/// 呼び出し側 `attrs`（属性値経路）へ全ペイロードを注入し、エスケープが
/// 貫通することを固定する。
#[test]
fn tour_title_description_target_and_attrs_are_escaped_for_all_payloads() {
    for payload in payloads::all() {
        let title_node = tour::Tour::default().title(None, vec![], vec![text(payload)]);
        let html = render(&title_node);
        assert_payload_is_escaped(payload, &html, "tour::Tour::title のテキストコンテキスト");

        let description_node = tour::Tour::default().description(None, vec![], vec![text(payload)]);
        let html = render(&description_node);
        assert_payload_is_escaped(
            payload,
            &html,
            "tour::Tour::description のテキストコンテキスト",
        );

        let step = TourStep {
            id: "s1".to_string(),
            target: Some(payload.to_string()),
            title: "t".to_string(),
            description: "d".to_string(),
            placement: Placement::new(Side::Bottom, Align::Center),
        };
        let mut t = Tour::new(vec![step]);
        fandhe_frontend_interactive::dispatch(&mut t, "start", "");
        let html = render(&t.spotlight(vec![], vec![]));
        assert_payload_is_escaped(
            payload,
            &html,
            "tour::Tour::spotlight の data-target コンテキスト",
        );

        let root_attrs_node = Tour::default().root(vec![("data-testid", payload)], vec![]);
        let html = render(&root_attrs_node);
        assert_payload_is_escaped(
            payload,
            &html,
            "tour::Tour::root の呼び出し側 attrs コンテキスト",
        );
    }
}
