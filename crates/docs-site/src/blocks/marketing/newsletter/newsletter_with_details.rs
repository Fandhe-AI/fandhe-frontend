//! `newsletter-with-details` block（イシュー #2797。親トラッキング #2738
//! 「Blocks マーケティング A」配下、対応表 ID R1096（主参照 1 件のみ、
//! 集約元の差分なし）の合成例。補足項目付きの newsletter 登録フォーム。
//! 取得手段・ファイル名・内部コンポーネント識別子は記載しない
//! （`contact_info_columns` と同じライセンス上の転記制限）。
//!
//! # 使用部品
//!
//! `heading` / `text` / `field` / `input` / `button` / `icon` /
//! `visually_hidden` の 7 部品のみを合成する（[`BLOCK`] の `parts` に
//! 一致させる契約、`crates/docs-site/tests/blocks_nav.rs`/
//! `blocks_contract.rs` が検証する）。新しい UI 部品は追加しない。
//! `link` は使わない（要件に外部リンクがない。プライバシー文へのリンク等、
//! 他 block が持つ `link` 由来の要素はいずれも本 block には存在しない）。
//!
//! # レイアウト（`lg` で 2 列、`lg` 未満は補足項目を 2 列にしてフォームの
//! 下へ回す）
//!
//! 狭幅は「見出し + 説明 + フォーム」の下に補足項目 2 件が並ぶ 1 カラム
//! 縦積み。`40rem` 以上で補足項目を 2 列（`repeat(2, minmax(0,1fr))`）に
//! し、あわせて入力欄 + 送信ボタンも横並びへ切り替える
//! （`contact_info_columns`/`contact_split_info` と同じ `40rem` 判断）。
//! `64rem`（[`Breakpoint::Lg`](fandhe_frontend_pre_styled_ui::recipe::Breakpoint::Lg)）
//! 以上で左（見出し・フォーム）/ 右（補足項目 2 列のまま）の 2 カラムへ
//! 切り替える（`contact_info_columns` と同じ「テーマの breakpoint
//! トークンは `@media` 条件式の中では解決できないためリテラル値を直書き
//! する」判断）。
//!
//! # 可視ラベルの代わりに `visually_hidden` + `<label for>`
//!
//! `banner_email_signup::signup` と同じく可視ラベルを出さず、
//! [`visually_hidden::root`] で包んだ [`field::label`] が `<label for>`
//! の関連付けでアクセシブル名を確保する。
//!
//! # `text` の名前衝突
//!
//! `fandhe_frontend_pre_styled_ui::text::text` と
//! `fandhe_frontend_core::text` が同名のため、styled 側を `styled_text`
//! として取り込む（`crate::blocks` 内の他 block と同じ回避方法）。
//!
//! # 自作の幾何アイコン
//!
//! lucide 等の既存アイコンセットの path を複製しないため、
//! `contact_info_columns::geo_icon` と同型の自作ヘルパ [`geo_icon`] で
//! 描く単純図形 2 種（カレンダー風の矩形＋横線・盾風の抽象図形）を補足
//! 項目のアイコンとして使う。隣に見出しテキストがあるため装飾扱い
//! （`IconProps::label` は `None` のまま、`aria-hidden="true"`）とする。
//!
//! # `<form>` を持たない・データ取得/送信を行わない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である。送信ボタンは `button::button` の既定 `type="button"` のまま
//! 送信先を持たない。文言はすべて独自の架空の日本語ダミー（実企業名・
//! 実クレデンシャル・PII を含まない）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::field::{self, FieldOrientation, FieldRootProps};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::input::{self, FieldIds, FieldProps, InputProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};
use fandhe_frontend_pre_styled_ui::visually_hidden;
use fandhe_frontend_pre_styled_ui::Size;

/// 装飾用の自作幾何アイコン（lucide 等の既存アイコンセットの path を
/// 複製しないための単純図形、`contact_info_columns::geo_icon` と同型）。
fn geo_icon(path_d: &'static str) -> Node {
    icon(
        &IconProps {
            size: Size::Sm,
            ..IconProps::default()
        },
        vec![],
        vec![el(
            "path",
            vec![
                ("d", path_d),
                ("fill", "none"),
                ("stroke", "currentColor"),
                ("stroke-width", "2"),
                ("stroke-linecap", "round"),
                ("stroke-linejoin", "round"),
            ],
            vec![],
        )],
    )
}

/// カレンダー風の幾何アイコン（配信頻度の補足項目）。
fn calendar_icon() -> Node {
    geo_icon("M4 5h16v15H4V5z M4 9h16 M8 3v4 M16 3v4")
}

/// 盾風の幾何アイコン（配信停止のしやすさの補足項目）。
fn shield_icon() -> Node {
    geo_icon("M12 3l7 3v6c0 4.5-3 7.5-7 9-4-1.5-7-4.5-7-9V6l7-3z")
}

/// メールアドレス入力 + 送信ボタン（`signup` 領域）。可視ラベルは出さず
/// `visually_hidden::root` で包んだ `field::label` が `<label for>` の
/// 関連付けを担う（モジュール doc「可視ラベルの代わりに」節参照）。
fn signup() -> Node {
    let email_field = FieldProps {
        id: "blocks-newsletter-with-details-email",
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: true,
        readonly: false,
        has_helper_text: false,
    };
    let orientation = FieldRootProps {
        orientation: FieldOrientation::Vertical,
    };

    div(
        vec![("class", "blocks-newsletter-with-details-controls")],
        vec![
            field::root(
                &orientation,
                &email_field,
                vec![],
                vec![
                    visually_hidden::root(
                        vec![],
                        vec![field::label(
                            &email_field,
                            vec![],
                            vec![text("メールアドレス")],
                        )],
                    ),
                    input::input(
                        &InputProps::default(),
                        &email_field,
                        vec![
                            ("type", "email"),
                            ("autocomplete", "email"),
                            ("placeholder", "you@example.com"),
                        ],
                    ),
                ],
            ),
            button::button(
                &ButtonProps::default(),
                vec![("data-blocks-newsletter-with-details-submit", "")],
                vec![text("登録する")],
            ),
        ],
    )
}

/// 見出し + 説明 + 登録フォーム（`main` 領域）。
fn copy_and_signup() -> Node {
    div(
        vec![("class", "blocks-newsletter-with-details-main")],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    ..HeadingProps::default()
                },
                vec![],
                vec![text("お知らせをメールでお届けします")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(
                    "新しい部品・block の追加やイベント情報を、月 1 回程度のペースでまとめてお届けします。",
                )],
            ),
            signup(),
        ],
    )
}

/// 補足項目 1 件（アイコン箱 + 見出し + 説明）。
fn detail(icon_fn: fn() -> Node, title: &'static str, description: &'static str) -> Node {
    div(
        vec![("data-blocks-newsletter-with-details-item", "")],
        vec![
            div(
                vec![("data-blocks-newsletter-with-details-icon-box", "")],
                vec![icon_fn()],
            ),
            heading(
                HeadingLevel::H4,
                &HeadingProps {
                    size: HeadingSize::Md,
                    weight: HeadingWeight::Semibold,
                },
                vec![],
                vec![text(title)],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(description)],
            ),
        ],
    )
}

/// `newsletter-with-details` の Demo 本体（左: 見出し・説明・フォーム、
/// 右: 補足項目 2 件。呼び出しごとに同一の `Node` を返す純関数）。
pub fn demo() -> Node {
    let details = div(
        vec![("data-blocks-newsletter-with-details-details", "")],
        vec![
            detail(
                calendar_icon,
                "配信頻度",
                "月 1 回程度の頻度でお届けします。不要な通知はいたしません。",
            ),
            detail(
                shield_icon,
                "配信停止も簡単",
                "メール本文のリンクからいつでも配信停止の手続きができます。",
            ),
        ],
    );

    div(
        vec![("class", "blocks-newsletter-with-details-layout")],
        vec![copy_and_signup(), details],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/newsletter-with-details/",
    title: "newsletter-with-details",
    category: BlockCategory::Newsletter,
    rust_source: "crates/docs-site/src/blocks/marketing/newsletter/newsletter_with_details.rs",
    demo_class: "blocks-newsletter-with-details",
    parts: &[
        Part {
            label: "Heading",
            path: "/themes/heading/",
        },
        Part {
            label: "Text",
            path: "/themes/text/",
        },
        Part {
            label: "Field",
            path: "/themes/field/",
        },
        Part {
            label: "Input",
            path: "/themes/input/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Icon",
            path: "/themes/icon/",
        },
        Part {
            label: "Visually Hidden",
            path: "/themes/visually-hidden/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `newsletter_with_details` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節と同型で、本ファイル内 private 定数として
/// `super::stylesheet` 経由の `push_css` で連結される）。
///
/// セレクタは `.blocks-newsletter-with-details-*` と
/// `[data-blocks-newsletter-with-details-*]` のみを用いる
/// （`contact_info_columns`/`careers_card_grid` と同じ名前空間分離）。
///
/// # ルート class を `demo_class` と別名にする理由
///
/// [`Block::demo_class`] は `blocks-newsletter-with-details` だが、
/// `demo()` が返すルート `div` の class は
/// `blocks-newsletter-with-details-layout` という別名にする
/// （既存 block と同じ Bugbot 教訓の回避）。
const LAYOUT_CSS: &str = "\
.blocks-newsletter-with-details-layout {\n  display: grid;\n  grid-template-columns: 1fr;\n  gap: var(--fandhe-space-10);\n}\n\
.blocks-newsletter-with-details-main {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-newsletter-with-details-controls {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n}\n\
[data-blocks-newsletter-with-details-details] {\n  display: grid;\n  grid-template-columns: 1fr;\n  gap: var(--fandhe-space-8);\n}\n\
[data-blocks-newsletter-with-details-item] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n}\n\
[data-blocks-newsletter-with-details-icon-box] {\n  display: inline-flex;\n  align-items: center;\n  justify-content: center;\n  width: var(--fandhe-space-10);\n  height: var(--fandhe-space-10);\n  border-radius: var(--fandhe-radius-lg);\n  background: var(--fandhe-color-bg-subtle);\n  color: var(--fandhe-color-accent);\n}\n\
@media (min-width: 40rem) {\n  .blocks-newsletter-with-details-controls {\n    flex-direction: row;\n  }\n  .blocks-newsletter-with-details-controls [data-scope=\"field\"][data-part=\"root\"] {\n    flex: 1 1 0;\n    min-width: 0;\n  }\n  [data-blocks-newsletter-with-details-details] {\n    grid-template-columns: repeat(2, minmax(0, 1fr));\n  }\n}\n\
@media (min-width: 64rem) {\n  .blocks-newsletter-with-details-layout {\n    grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);\n    align-items: start;\n  }\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// Demo が使用部品（heading/text/field/input/button/icon/
    /// visually-hidden）の anatomy をすべて実際に出力していることと、
    /// 補足項目・アイコン箱の件数を固定する。
    #[test]
    fn demo_composes_expected_parts() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"field\" data-part=\"root\"",
            "data-scope=\"field\" data-part=\"input\"",
            "data-scope=\"button\"",
            "data-scope=\"icon\"",
            "data-scope=\"visually-hidden\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert_eq!(
            html.matches("data-blocks-newsletter-with-details-item")
                .count(),
            2,
            "demo should render exactly 2 detail items"
        );
        assert_eq!(
            html.matches("data-blocks-newsletter-with-details-icon-box")
                .count(),
            2,
            "demo should render exactly 2 icon boxes"
        );
    }

    /// `<form>` を出力しない・XSS 回帰の不変条件を固定する
    /// （`crate::blocks` モジュール doc）。
    #[test]
    fn demo_has_no_form_or_unsafe_output() {
        let html = render(&demo());
        for absent in [
            "<form",
            "type=\"submit\"",
            "href=\"#\"",
            "src=\"data:",
            "mailto:",
            "<script",
        ] {
            assert!(!html.contains(absent), "demo should never contain {absent}");
        }
    }

    /// 送信ボタンはちょうど 1 個、`type="button"`。
    #[test]
    fn demo_has_exactly_one_type_button_button() {
        let html = render(&demo());
        assert_eq!(html.matches(r#"type="button""#).count(), 1);
    }

    /// visually-hidden な `<label for>` と同じ `id` を持つ input が
    /// 存在する（アクセシブル名の関連付けを固定）。
    #[test]
    fn demo_label_for_matches_input_id() {
        let html = render(&demo());
        let for_attr = r#"for="blocks-newsletter-with-details-email-control""#;
        let id_attr = r#"id="blocks-newsletter-with-details-email-control""#;
        assert!(html.contains(for_attr), "{for_attr} should be present");
        assert!(html.contains(id_attr), "{id_attr} should be present");
    }

    /// アイコン `svg` が `aria-hidden="true"` を持つ（装飾扱い）。
    #[test]
    fn demo_icons_are_decorative() {
        let html = render(&demo());
        assert!(html.contains("aria-hidden=\"true\""));
    }

    /// [`LAYOUT_CSS`] が想定するブレークポイントを持ち、`<` を含まない
    /// こと（REQ-1: `</style>` によるスタイル脱出を防ぐ）。
    #[test]
    fn layout_css_declares_breakpoints_and_no_angle_bracket() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("@media (min-width: 40rem)"));
        assert!(LAYOUT_CSS.contains("@media (min-width: 64rem)"));
    }

    /// ルート class（`demo_class` とは別名）が `demo()` の出力へ実際に
    /// 現れること（モジュール doc「ルート class を `demo_class` と別名に
    /// する理由」節の固定）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-newsletter-with-details-layout\""));
        assert_ne!(
            super::BLOCK.demo_class,
            "blocks-newsletter-with-details-layout"
        );
    }
}
