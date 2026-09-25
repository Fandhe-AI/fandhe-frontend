//! `contact-dialog-form` block（イシュー #2827。親トラッキング
//! #2730/#2731「Blocks 目的別パーツ拡充ツリー」Phase 2 配下）。ダイアログ
//! 内に問い合わせフォーム（氏名・メール・本文）を配置した合成例。
//!
//! # 使用部品
//!
//! `dialog`（構造）/ `field`（`group`/`root`/`label`）/ `input`（氏名・
//! メール）/ `textarea`（本文）/ `button`（キャンセル・送信）の 5 部品を
//! 合成する（`crate::blocks::Block::parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//! 新しい UI 部品は追加しない。
//!
//! # 静的表示・`trigger`/`close_trigger` を置かない理由
//!
//! docs サイトは JS ハイドレーションを行わない設計（CLAUDE.md）のため、
//! 本 Demo はダイアログが**既に開いた静的な初期状態**のみを描く
//! （`game_ui_modal`/`testimonials_stack` と同じ設計判断）。
//! [`fandhe_frontend_pre_styled_ui::dialog::trigger`] は無 JS 下では
//! 開閉を切り替えられず表示上の意味を持たないため、意図的に置かない。
//! [`fandhe_frontend_pre_styled_ui::dialog::close_trigger`] も同じ理由で
//! 置かない: 押しても何も起きない `button` をキーボード・支援技術利用者に
//! 実行可能な操作として提示すると表示契約（「閉じる機構を持たない」）と
//! アクセシビリティが不一致になるため（`game_ui_modal` に close trigger が
//! ないのと同じ判断、PR #3195 の codex レビュー P1 指摘を踏襲）。
//! 開閉・フォーカストラップ・Escape 等の挙動は一切扱わない。
//!
//! # `aria-modal` を false にする理由
//!
//! 静的なデモは閉じる機構を持たず、ダイアログの外側に説明・コード・
//! ナビゲーションがある。支援技術が外側を無視しないよう、表示の実態と
//! 一致させて `aria-modal` は false にする（`game_ui_modal` と同じ判断、
//! イシュー #2552 レビュー指摘の踏襲）。
//!
//! # `<form>` を使わない・送信処理を持たない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力せず、ボタンはすべて `button::button` の既定 `type="button"` の
//! まま用いる。送信先・検証・状態機械は持たない静的な合成例である
//! （`docs/policy/intentional-non-adoption.md` §3.25、UI コンポーネント層は
//! アプリケーションロジックを内包しない）。
//!
//! # 参照について
//!
//! 主参照・集約元はいずれも私有カタログ上の 1 件のみであり、取得手段・
//! ファイル名・内部コンポーネント識別子は記載しない（購入者限定素材の
//! ライセンス上の転記制限、`docs/design/motion-reference-adoption-policy.md`
//! §9 と同じ方針）。参照から取り込むのは構造（領域の配置と部品構成）のみで、
//! 文言・配色・装飾・アイコンは独自に書く。
//!
//! # `body` の `max-height` 上書き
//!
//! [`fandhe_frontend_pre_styled_ui::dialog::body`] の既定 CSS は
//! `max-height: 50vh; overflow-y: auto` だが、本 Demo は入力欄 3 件のみで
//! 縦スクロールが必要な高さにならない。既定のままだとデモ枠内で不要な
//! スクロールバーが生じ、入力欄のフォーカスリングが枠に切られるおそれが
//! あるため、[`LAYOUT_CSS`] で `max-height: none; overflow: visible;` へ
//! 上書きする。
use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::dialog::{self, ContentIds, DialogRole, OpenState};
use fandhe_frontend_pre_styled_ui::field::{self, FieldOrientation, FieldRootProps};
use fandhe_frontend_pre_styled_ui::input::{self, FieldIds, FieldProps, InputProps};
use fandhe_frontend_pre_styled_ui::textarea::{self, TextareaProps};
use fandhe_frontend_pre_styled_ui::Size;

/// `contact-dialog-form` の Demo 本体（既に開いた静的な初期状態のみ描く）。
pub fn demo() -> Node {
    let title_id = "blocks-contact-dialog-form-title";
    let description_id = "blocks-contact-dialog-form-description";

    let name_field = FieldProps {
        id: "blocks-contact-dialog-form-name",
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: true,
        readonly: false,
        has_helper_text: false,
    };
    let email_field = FieldProps {
        id: "blocks-contact-dialog-form-email",
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: true,
        readonly: false,
        has_helper_text: false,
    };
    let message_field = FieldProps {
        id: "blocks-contact-dialog-form-message",
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

    dialog::root(
        Size::Md,
        OpenState::Open,
        vec![("data-blocks-contact-dialog-form-root", "")],
        vec![
            dialog::backdrop(OpenState::Open, vec![], vec![]),
            dialog::positioner(
                OpenState::Open,
                vec![],
                vec![dialog::content(
                    OpenState::Open,
                    DialogRole::Dialog,
                    // 静的デモは閉じる機構を持たず外側に説明・コード・
                    // ナビゲーションがあるため、表示実態と一致させ
                    // aria-modal は false にする（`game_ui_modal` と同じ
                    // 判断、イシュー #2552 レビュー指摘の踏襲）。
                    false,
                    ContentIds {
                        id: Some("blocks-contact-dialog-form-content"),
                        labelledby: Some(title_id),
                        describedby: Some(description_id),
                    },
                    vec![("data-blocks-contact-dialog-form-content", "")],
                    vec![
                        dialog::title(Some(title_id), vec![], vec![text("お問い合わせ")]),
                        dialog::description(
                            Some(description_id),
                            vec![],
                            vec![text(
                                "ご質問・ご要望をお送りください。担当者が折り返しご連絡します。",
                            )],
                        ),
                        dialog::body(
                            vec![("data-blocks-contact-dialog-form-body", "")],
                            vec![field::group(
                                vec![],
                                vec![
                                    field::root(
                                        &orientation,
                                        &name_field,
                                        vec![("data-blocks-contact-dialog-form-field", "")],
                                        vec![
                                            field::label(&name_field, vec![], vec![text("氏名")]),
                                            input::input(
                                                &InputProps::default(),
                                                &name_field,
                                                vec![
                                                    ("type", "text"),
                                                    ("placeholder", "山田 太郎"),
                                                    ("autocomplete", "name"),
                                                ],
                                            ),
                                        ],
                                    ),
                                    field::root(
                                        &orientation,
                                        &email_field,
                                        vec![("data-blocks-contact-dialog-form-field", "")],
                                        vec![
                                            field::label(
                                                &email_field,
                                                vec![],
                                                vec![text("メールアドレス")],
                                            ),
                                            input::input(
                                                &InputProps::default(),
                                                &email_field,
                                                vec![
                                                    ("type", "email"),
                                                    ("placeholder", "you@example.com"),
                                                    ("autocomplete", "email"),
                                                ],
                                            ),
                                        ],
                                    ),
                                    field::root(
                                        &orientation,
                                        &message_field,
                                        vec![("data-blocks-contact-dialog-form-field", "")],
                                        vec![
                                            field::label(
                                                &message_field,
                                                vec![],
                                                vec![text("お問い合わせ内容")],
                                            ),
                                            textarea::textarea(
                                                &TextareaProps::default(),
                                                &message_field,
                                                false,
                                                vec![
                                                    ("rows", "4"),
                                                    (
                                                        "placeholder",
                                                        "ご質問・ご要望をご記入ください",
                                                    ),
                                                    ("data-blocks-contact-dialog-form-message", ""),
                                                ],
                                                vec![],
                                            ),
                                        ],
                                    ),
                                ],
                            )],
                        ),
                        dialog::footer(
                            vec![("data-blocks-contact-dialog-form-footer", "")],
                            vec![
                                button::button(
                                    &ButtonProps {
                                        variant: ButtonVariant::Outline,
                                        ..ButtonProps::default()
                                    },
                                    vec![],
                                    vec![text("キャンセル")],
                                ),
                                button::button(
                                    &ButtonProps::default(),
                                    vec![("data-blocks-contact-dialog-form-submit", "")],
                                    vec![text("送信")],
                                ),
                            ],
                        ),
                    ],
                )],
            ),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/contact-dialog-form/",
    title: "contact-dialog-form",
    category: BlockCategory::Contact,
    rust_source: "crates/docs-site/src/blocks/marketing/contact/contact_dialog_form.rs",
    demo_class: "blocks-contact-dialog-form",
    parts: &[
        Part {
            label: "Dialog",
            path: "/themes/dialog/",
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
            label: "Textarea",
            path: "/themes/textarea/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `contact_dialog_form` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節、`game_ui_modal`/`login_01` と同型）。
///
/// # デモ枠内での掲示
///
/// `dialog::positioner`/`backdrop` は本来 `position: fixed; inset: 0` の
/// ビューポート全体オーバーレイだが、Blocks の掲示は `.blocks-demo` 枠内へ
/// 収める必要がある。本 block スコープ（`.blocks-contact-dialog-form`
/// 配下）に限定した属性セレクタで `position: relative`・`inset: auto` へ
/// 差し替える。`dialog::title` の `h2` は `.docs-content` のタイポグラフィ
/// （`border-top`/`padding-top`/`letter-spacing`）を継承してしまうため、
/// `showcase.rs` の pre-styled-showcase と同型のリセットを併せて適用する。
///
/// # 高さは `positioner`（実コンテンツ側）が決め、`backdrop` はそれに追随する
///
/// `game_ui_modal` は `backdrop` に固定の `min-height` を持たせ、`inset: 0`
/// の `positioner` をその上へ絶対配置で重ねる構成を採る（`positioner` の
/// 高さは `backdrop` の高さに追随する）。本 block は `positioner`（3 件の
/// `field`・8rem の `textarea`・footer を含む実コンテンツ）の方が
/// `backdrop`（単なる背景色の装飾）より確実に高くなるため、両者の役割を
/// 反転させる: `positioner` を通常の flow（`position: relative`）へ戻して
/// **実コンテンツの高さで `[data-blocks-contact-dialog-form-root]`（両者の
/// 共通の位置指定祖先）の高さを決めさせ**、`backdrop` の方を `position:
/// absolute; inset: 0` にして positioner に追随させる（Bugbot 指摘: 固定
/// `min-height` の `backdrop` に `inset: 0` の `positioner` を重ねる向きだと
/// 実コンテンツが `backdrop` の高さを超えたときに `backdrop` の外へはみ
/// 出す。役割を反転すれば `root` の高さが常に実コンテンツに一致するため、
/// コンテンツ量に依存する固定値の当てずっぽうが不要になる）。`positioner`
/// の `min-height` は最低限の見栄え用の下限（floor）に過ぎず、実コンテンツ
/// がそれより高ければ自然に伸びる。
///
/// # `body` の `max-height` 上書き（モジュール doc参照）
///
/// [`fandhe_frontend_pre_styled_ui::dialog::body`] の既定 `max-height: 50vh;
/// overflow-y: auto` を打ち消し、入力欄 3 件のみの本 Demo でスクロール
/// バーが生じてフォーカスリングが枠に切られる事態を防ぐ。
///
/// # 狭幅対応
///
/// `@media (max-width: 47.99rem)`（リポジトリの慣例的なブレークポイント）
/// で positioner の padding を縮め、footer のボタンを縦積み・全幅にする。
/// `dialog::content` は recipe 既定で `width: 100%` のため、デモ枠の幅に
/// 合わせて自然に縮む。
const LAYOUT_CSS: &str = "\
.blocks-contact-dialog-form.blocks-demo {\n  overflow: visible;\n}\n\
[data-blocks-contact-dialog-form-root] {\n  position: relative;\n}\n\
.blocks-contact-dialog-form [data-scope=\"dialog\"] h2 {\n  border-top: none;\n  padding-top: 0;\n  letter-spacing: normal;\n}\n\
.blocks-contact-dialog-form [data-scope=\"dialog\"][data-part=\"backdrop\"] {\n  position: absolute;\n  inset: 0;\n  z-index: auto;\n  border-radius: var(--fandhe-radius-lg);\n  background: var(--fandhe-color-bg-subtle);\n}\n\
.blocks-contact-dialog-form [data-scope=\"dialog\"][data-part=\"positioner\"] {\n  position: relative;\n  inset: auto;\n  z-index: auto;\n  width: 100%;\n  min-height: 32rem;\n  display: flex;\n  align-items: center;\n  justify-content: center;\n  padding: var(--fandhe-space-6);\n}\n\
.blocks-contact-dialog-form [data-scope=\"dialog\"][data-part=\"body\"] {\n  max-height: none;\n  overflow: visible;\n}\n\
[data-blocks-contact-dialog-form-field] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n}\n\
[data-scope=\"field\"][data-part=\"textarea\"][data-blocks-contact-dialog-form-message] {\n  min-height: 8rem;\n}\n\
[data-blocks-contact-dialog-form-footer] {\n  display: flex;\n  justify-content: flex-end;\n  gap: var(--fandhe-space-3);\n}\n\
@media (max-width: 47.99rem) {\n  .blocks-contact-dialog-form [data-scope=\"dialog\"][data-part=\"positioner\"] {\n    padding: var(--fandhe-space-3);\n  }\n  [data-blocks-contact-dialog-form-footer] {\n    flex-direction: column-reverse;\n  }\n  [data-blocks-contact-dialog-form-footer] [data-scope=\"button\"] {\n    width: 100%;\n  }\n}\n";

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;

    fn demo_html() -> String {
        render(&demo())
    }

    /// Demo が使用部品（dialog/field/input/textarea/button）の anatomy を
    /// すべて実際に出力していることを固定する（`game_ui_modal` 先例と同型）。
    #[test]
    fn demo_composes_expected_parts() {
        let html = demo_html();
        for scope in [
            "data-scope=\"dialog\" data-part=\"backdrop\"",
            "data-scope=\"dialog\" data-part=\"positioner\"",
            "data-scope=\"dialog\" data-part=\"content\"",
            "data-scope=\"dialog\" data-part=\"title\"",
            "data-scope=\"dialog\" data-part=\"description\"",
            "data-scope=\"dialog\" data-part=\"body\"",
            "data-scope=\"dialog\" data-part=\"footer\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
        assert_eq!(
            html.matches("data-scope=\"field\" data-part=\"input\"")
                .count(),
            2,
            "demo should render exactly 2 input parts (name/email)"
        );
        assert!(
            html.contains("data-scope=\"field\" data-part=\"textarea\""),
            "demo should render a textarea part"
        );
        assert!(
            html.contains("data-scope=\"button\""),
            "demo should render button parts"
        );
    }

    /// `<form>` を出力せず、ボタンがすべて `type="button"` であることを
    /// 固定する（`crate::blocks` モジュール doc の不変条件）。
    #[test]
    fn demo_has_no_form_and_only_button_type_buttons() {
        let html = demo_html();
        assert!(!html.contains("<form"), "demo should never contain <form");
        for absent in [
            "type=\"submit\"",
            "form=\"",
            "action=",
            "href=",
            "src=\"data:",
        ] {
            assert!(!html.contains(absent), "demo should never contain {absent}");
        }
        assert_eq!(
            html.matches("type=\"button\"").count(),
            2,
            "demo should have exactly 2 type=\"button\" buttons (cancel/submit)"
        );
    }

    /// 静的な開状態・非モーダル（`aria-modal=false`）であることを固定する
    /// （`game_ui_modal` と同じ設計判断、モジュール doc参照）。
    #[test]
    fn dialog_is_open_static_and_non_modal() {
        let html = demo_html();
        assert!(html.contains("aria-modal=\"false\""));
        assert!(html.contains("data-state=\"open\""));
        // backdrop の `aria-hidden="true"`（装飾層のため常時付与、headless
        // 側の既定仕様）に "hidden" 部分文字列がたまたま含まれるため、
        // 素朴な `contains("hidden")` は誤検知する。closed 時のみ付与される
        // 存在属性 `hidden`（開いた状態では出力されないはずの単独属性）を
        // 精確に検知する。
        assert!(!html.contains(" hidden>"));
        assert!(!html.contains(" hidden "));
    }

    /// name / email / message の `label[for]` と、各コントロールの `id` が
    /// 一致することを固定する（アクセシビリティ上の関連付け不変条件）。
    #[test]
    fn labels_point_at_their_controls() {
        let html = demo_html();
        for (label_for, control_id) in [
            (
                "for=\"blocks-contact-dialog-form-name-control\"",
                "id=\"blocks-contact-dialog-form-name-control\"",
            ),
            (
                "for=\"blocks-contact-dialog-form-email-control\"",
                "id=\"blocks-contact-dialog-form-email-control\"",
            ),
            (
                "for=\"blocks-contact-dialog-form-message-control\"",
                "id=\"blocks-contact-dialog-form-message-control\"",
            ),
        ] {
            assert!(html.contains(label_for), "missing {label_for}");
            assert!(html.contains(control_id), "missing {control_id}");
        }
    }

    /// `message` フックが textarea パート要素そのものに付与されて
    /// いることを固定する（field root への誤付与を防ぐ、`#3193` レビュー
    /// 指摘と同型の検証観点）。
    #[test]
    fn message_hook_attribute_is_on_the_textarea_part_element() {
        let html = demo_html();
        assert_eq!(
            html.matches("data-blocks-contact-dialog-form-message")
                .count(),
            1,
            "the message hook attribute should appear exactly once"
        );
        let idx = html
            .find("data-blocks-contact-dialog-form-message")
            .expect("hook attribute should be present");
        let tag_start = html[..idx].rfind('<').expect("hook should be inside a tag");
        let tag = &html[tag_start..idx];
        assert!(
            tag.contains("data-part=\"textarea\""),
            "hook attribute should be on the textarea part element, tag was: {tag}"
        );
    }

    /// [`LAYOUT_CSS`] が固定オーバーレイの中和・狭幅ブレークポイント・
    /// body の max-height 上書きを含み、`<` を含まないことを固定する
    /// （REQ-1: `</style>` によるスタイル脱出を防ぐ）。
    #[test]
    fn layout_css_neutralizes_fixed_overlay_and_declares_narrow_breakpoint() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("position: relative;\n  inset: auto;"));
        assert!(LAYOUT_CSS
            .contains("[data-blocks-contact-dialog-form-root] {\n  position: relative;\n}"));
        assert!(LAYOUT_CSS.contains("max-height: none;"));
        assert!(LAYOUT_CSS.contains("@media (max-width: 47.99rem)"));
    }
}
