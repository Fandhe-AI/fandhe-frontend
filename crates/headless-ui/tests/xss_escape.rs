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
use fandhe_frontend_headless_ui::{
    aria_controls, aria_label, avatar, data_state, dialog, editable, number_input, pin_input,
    popover, rating_group, segment_group, slider, tags_input, ImageStatus, OpenState, Orientation,
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

        let label_node = number_input::label(false, false, None, vec![], vec![text(payload)]);
        let html = render(&label_node);
        assert_payload_is_escaped(payload, &html, "number_input::label のテキストコンテキスト");
    }
}

/// (1)/(2) RatingGroup（イシュー #742）: `hidden_input` の `name`（属性値
/// 経路）と `label` の children（テキスト経路）へ全ペイロードを注入し、
/// エスケープが貫通することを固定する（`number_input` 分と同型）。
#[test]
fn rating_group_name_and_label_children_are_escaped_for_all_payloads() {
    for payload in payloads::all() {
        let hidden_input_node = rating_group::hidden_input(Some(payload), "3", false, vec![]);
        let html = render(&hidden_input_node);
        assert_payload_is_escaped(
            payload,
            &html,
            "rating_group::hidden_input の name 属性値コンテキスト",
        );

        let label_node = rating_group::label(None, vec![], vec![text(payload)]);
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

/// (2) 属性値経路（イシュー #739 PinInput）: `pin_input::hidden_input` の
/// `name`/`value`・`pin_input::input` の `value`・呼び出し側 `attrs` へ全
/// ペイロードを注入し、エスケープが貫通することを固定する。`aria-label`
/// は `format!` で組み立てた動的文字列（`crates/headless-ui/src/pin_input.rs`
/// 参照）だが本テストの対象ではなく専用の inline テスト
/// （`pin_input::tests`）で固定済みのため、ここでは `name`/`value`/`attrs`
/// の 3 経路のみを扱う。
#[test]
fn pin_input_hidden_input_and_input_value_are_escaped_for_all_payloads() {
    use fandhe_frontend_headless_ui::pin_input::PinInputKind;

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
            false,
            false,
            vec![],
        );
        let html = render(&input_node);
        assert_payload_is_escaped(payload, &html, "pin_input::input の value コンテキスト");

        let attrs_node = pin_input::root(false, false, vec![("data-testid", payload)], vec![]);
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

        let label_node =
            editable::label(EditMode::Preview, false, None, vec![], vec![text(payload)]);
        let html = render(&label_node);
        assert_payload_is_escaped(payload, &html, "editable::label のテキストコンテキスト");

        let preview_node = editable::preview(EditMode::Preview, false, vec![], vec![text(payload)]);
        let html = render(&preview_node);
        assert_payload_is_escaped(payload, &html, "editable::preview のテキストコンテキスト");

        let attrs_node = editable::root(
            EditMode::Preview,
            false,
            false,
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
