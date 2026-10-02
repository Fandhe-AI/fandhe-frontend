//! `empty-state-card-header` block（イシュー #2969。親トラッキング
//! #2730/#2731「Blocks 目的別パーツ拡充ツリー」配下）。カードのヘッダーに
//! 題名 + 作成ボタン、本文にカード内空状態（アイコン・見出し・説明・
//! 作成ボタン）を中央寄せで配置した合成例。
//!
//! # 使用部品
//!
//! `card`（構造）/ `empty-state`（本文の空状態）/ `button`（作成・
//! キャンセル）/ `dialog`（作成ダイアログ）/ `field`（`group`/`root`/
//! `label`）/ `input`（プロジェクト名）/ `native-select`（公開範囲）の
//! 7 部品を合成する（`crate::blocks::Block::parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//! 新しい UI 部品は追加しない。
//!
//! # 2 インスタンスの静的併記（参照 ID の差分表現）
//!
//! 対応表 ID R0375（代表構成: カード単体）と R0234（集約元: 作成ダイアログ
//! 付き）の差分を、docs サイトが無 JS ハイドレーションを行わない設計の
//! もとで開閉トグルではなく **2 インスタンスの静的併記**で表現する
//! （`contact-dialog-form`/`game_ui_modal` と同じ「既に開いた静的な初期
//! 状態のみを描く」方針の応用）。1 つ目がカード単体（R0375）、2 つ目が
//! 作成ダイアログの開いた状態（R0234）である。
//!
//! # `trigger`/`close_trigger` を置かない理由
//!
//! [`fandhe_frontend_pre_styled_ui::dialog::trigger`]/
//! [`fandhe_frontend_pre_styled_ui::dialog::close_trigger`] は無 JS 下では
//! 開閉を切り替えられず表示上の意味を持たないため、
//! `contact-dialog-form`/`game_ui_modal` と同じ判断で意図的に置かない。
//! 開閉・フォーカストラップ・Escape 等の挙動は一切扱わない。カード側の
//! 「新規作成」ボタン・空状態内の「最初のプロジェクトを作成」ボタンも、
//! 押しても何も起きない静的表示であることを示すため `type="button"` の
//! まま用いる。
//!
//! # `aria-modal` を false にする理由
//!
//! 静的なデモは閉じる機構を持たず、ダイアログの外側（カードインスタンス・
//! 説明・コード）に意味のあるコンテンツがある。支援技術が外側を無視
//! しないよう、表示の実態と一致させて `aria-modal` は false にする
//! （`contact-dialog-form`/`game_ui_modal` と同じ判断）。
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
//! 主参照 R0375・集約元 R0234 はいずれも私有カタログ上の対応表 ID のみを
//! 記載し、取得手段・ファイル名・内部コンポーネント識別子は記載しない
//! （購入者限定素材のライセンス上の転記制限、
//! `docs/design/motion-reference-adoption-policy.md` §9 と同じ方針）。
//! `_/blocks-intake/` はメイン worktree にも存在しないため参照ファイルは
//! 読んでおらず、レイアウトは Issue 本文の仕様記述から構成した
//! （`page-heading-avatar` #2931 と同じ扱い）。参照から取り込むのは構造
//! （領域の配置と部品構成）のみで、文言・配色・装飾・アイコンは独自に
//! 書く。
//!
//! # `body` の `max-height` 上書き
//!
//! [`fandhe_frontend_pre_styled_ui::dialog::body`] の既定 CSS は
//! `max-height: 50vh; overflow-y: auto` だが、本 Demo は入力欄 2 件のみで
//! 縦スクロールが必要な高さにならない。既定のままだとデモ枠内で不要な
//! スクロールバーが生じるため、[`LAYOUT_CSS`] で `max-height: none;
//! overflow: visible;` へ上書きする（`contact-dialog-form` と同じ判断）。
use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{el, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::dialog::{self, ContentIds, DialogRole, OpenState};
use fandhe_frontend_pre_styled_ui::empty_state::{
    self, EmptyStateIndicatorVariant, EmptyStateProps,
};
use fandhe_frontend_pre_styled_ui::field::{self, FieldOrientation, FieldRootProps};
use fandhe_frontend_pre_styled_ui::input::{self, FieldIds, FieldProps, InputProps};
use fandhe_frontend_pre_styled_ui::native_select::{self, NativeSelectProps};
use fandhe_frontend_pre_styled_ui::Size;

/// 自作の幾何アイコン（フォルダに「+」を重ねた単純な折れ線・矩形。
/// `hero_email_signup::play_icon` と同型の装飾用 SVG、`aria-hidden="true"`）。
/// 実在ブランドのロゴ・商標は模さない。
fn folder_plus_icon() -> Node {
    el(
        "svg",
        vec![
            ("viewBox", "0 0 24 24"),
            ("width", "1em"),
            ("height", "1em"),
            ("aria-hidden", "true"),
        ],
        vec![
            el(
                "path",
                vec![
                    (
                        "d",
                        "M3 6a1 1 0 0 1 1-1h5l2 2h9a1 1 0 0 1 1 1v10a1 1 0 0 1-1 1H4a1 1 0 0 1-1-1V6z",
                    ),
                    ("fill", "none"),
                    ("stroke", "currentColor"),
                    ("stroke-width", "1.5"),
                ],
                vec![],
            ),
            el(
                "path",
                vec![
                    ("d", "M12 11v5M9.5 13.5h5"),
                    ("fill", "none"),
                    ("stroke", "currentColor"),
                    ("stroke-width", "1.5"),
                    ("stroke-linecap", "round"),
                ],
                vec![],
            ),
        ],
    )
}

/// カード単体インスタンス（R0375・主参照）。ヘッダーに題名 + 作成ボタン、
/// 本文に空状態（アイコン・見出し・説明・作成ボタン）を中央寄せで置く。
fn card_instance() -> Node {
    card::root(
        CardProps::default(),
        vec![("data-blocks-empty-state-card-header-card", "")],
        vec![
            card::header(
                vec![("data-has-action", "")],
                vec![
                    card::title(vec![], vec![text("プロジェクト")]),
                    card::action(
                        vec![],
                        vec![button::button(
                            &ButtonProps::default(),
                            vec![],
                            vec![text("新規作成")],
                        )],
                    ),
                ],
            ),
            card::body(
                vec![],
                vec![empty_state::root(
                    &EmptyStateProps::default(),
                    vec![("data-blocks-empty-state-card-header-empty", "")],
                    vec![empty_state::content(
                        vec![],
                        vec![
                            empty_state::indicator_with(
                                EmptyStateIndicatorVariant::Boxed,
                                vec![],
                                vec![folder_plus_icon()],
                            ),
                            empty_state::title(vec![], vec![text("プロジェクトがありません")]),
                            empty_state::description(
                                vec![],
                                vec![text(
                                    "最初のプロジェクトを作成すると、ここに一覧が表示されます。",
                                )],
                            ),
                            empty_state::actions(
                                vec![],
                                vec![button::button(
                                    &ButtonProps::default(),
                                    vec![],
                                    vec![text("最初のプロジェクトを作成")],
                                )],
                            ),
                        ],
                    )],
                )],
            ),
        ],
    )
}

/// 作成ダイアログインスタンス（R0234・集約元）。`contact-dialog-form` と
/// 同型の「既に開いた静的な初期状態のみを描く」構成。
fn dialog_instance() -> Node {
    let title_id = "blocks-empty-state-card-header-dialog-title";
    let description_id = "blocks-empty-state-card-header-dialog-description";

    let name_field = FieldProps {
        id: "blocks-empty-state-card-header-name",
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: true,
        readonly: false,
        has_helper_text: false,
    };
    let visibility_field = FieldProps {
        id: "blocks-empty-state-card-header-visibility",
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: false,
    };
    let orientation = FieldRootProps {
        orientation: FieldOrientation::Vertical,
    };

    dialog::root(
        Size::Md,
        OpenState::Open,
        vec![("data-blocks-empty-state-card-header-dialog-root", "")],
        vec![
            dialog::backdrop(OpenState::Open, vec![], vec![]),
            dialog::positioner(
                OpenState::Open,
                vec![],
                vec![dialog::content(
                    OpenState::Open,
                    DialogRole::Dialog,
                    // 静的デモは閉じる機構を持たず外側にカードインスタンス・
                    // 説明・コードがあるため、表示実態と一致させ aria-modal
                    // は false にする（`contact-dialog-form` と同じ判断）。
                    false,
                    ContentIds {
                        id: Some("blocks-empty-state-card-header-dialog-content"),
                        labelledby: Some(title_id),
                        describedby: Some(description_id),
                    },
                    vec![("data-blocks-empty-state-card-header-dialog-content", "")],
                    vec![
                        dialog::title(Some(title_id), vec![], vec![text("プロジェクトを作成")]),
                        dialog::description(
                            Some(description_id),
                            vec![],
                            vec![text("プロジェクト名と公開範囲を指定してください。")],
                        ),
                        dialog::body(
                            vec![("data-blocks-empty-state-card-header-dialog-body", "")],
                            vec![field::group(
                                vec![],
                                vec![
                                    field::root(
                                        &orientation,
                                        &name_field,
                                        vec![],
                                        vec![
                                            field::label(
                                                &name_field,
                                                vec![],
                                                vec![text("プロジェクト名")],
                                            ),
                                            input::input(
                                                &InputProps::default(),
                                                &name_field,
                                                vec![
                                                    ("type", "text"),
                                                    ("placeholder", "新しいプロジェクト"),
                                                ],
                                            ),
                                        ],
                                    ),
                                    field::root(
                                        &orientation,
                                        &visibility_field,
                                        vec![],
                                        vec![
                                            field::label(
                                                &visibility_field,
                                                vec![],
                                                vec![text("公開範囲")],
                                            ),
                                            native_select::native_select(
                                                &NativeSelectProps::default(),
                                                &visibility_field,
                                                vec![],
                                                vec![
                                                    el(
                                                        "option",
                                                        vec![
                                                            ("value", "private"),
                                                            ("selected", "selected"),
                                                        ],
                                                        vec![text("非公開")],
                                                    ),
                                                    el(
                                                        "option",
                                                        vec![("value", "team")],
                                                        vec![text("チームのみ")],
                                                    ),
                                                    el(
                                                        "option",
                                                        vec![("value", "public")],
                                                        vec![text("公開")],
                                                    ),
                                                ],
                                            ),
                                        ],
                                    ),
                                ],
                            )],
                        ),
                        dialog::footer(
                            vec![("data-blocks-empty-state-card-header-dialog-footer", "")],
                            vec![
                                button::button(
                                    &ButtonProps {
                                        variant: ButtonVariant::Outline,
                                        ..ButtonProps::default()
                                    },
                                    vec![],
                                    vec![text("キャンセル")],
                                ),
                                button::button(&ButtonProps::default(), vec![], vec![text("作成")]),
                            ],
                        ),
                    ],
                )],
            ),
        ],
    )
}

/// `empty-state-card-header` の Demo 本体。カード単体（R0375）と作成
/// ダイアログ（R0234）を縦積みで静的併記する（モジュール doc「2 インス
/// タンスの静的併記」節参照）。
pub fn demo() -> Node {
    el(
        "div",
        vec![("class", "blocks-empty-state-card-header-layout")],
        vec![card_instance(), dialog_instance()],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/empty-state-card-header/",
    title: "empty-state-card-header",
    category: BlockCategory::EmptyState,
    rust_source: "crates/docs-site/src/blocks/application/empty_state/empty_state_card_header.rs",
    demo_class: "blocks-empty-state-card-header",
    parts: &[
        Part {
            label: "Card",
            path: "/themes/card/",
        },
        Part {
            label: "Empty State",
            path: "/themes/empty-state/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
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
            label: "Native Select",
            path: "/themes/native-select/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `empty_state_card_header` 固有のレイアウト規則（`crate::blocks`
/// モジュール doc「CSS の置き場」節、`contact_dialog_form`/`login_01` と同型）。
///
/// # 2 インスタンスの縦積みとカードの全幅表示
///
/// `.blocks-empty-state-card-header-layout` は 2 インスタンス（カード・
/// ダイアログ）を縦積みにする。カードは「一覧が入る領域と同じ幅」という
/// 仕様（デモ枠全幅）を満たすため `width: 100%` を明示する。
///
/// # ダイアログのデモ枠内での掲示
///
/// `dialog::positioner`/`backdrop` は本来 `position: fixed; inset: 0` の
/// ビューポート全体オーバーレイだが、Blocks の掲示は `.blocks-demo` 枠内へ
/// 収める必要がある。本 block スコープ（`.blocks-empty-state-card-header`
/// 配下）に限定した属性セレクタで `position: relative`・`inset: auto` へ
/// 差し替える（`contact_dialog_form` と同型）。`dialog::title` の `h2` は
/// `.docs-content` のタイポグラフィ（`border-top`/`padding-top`/
/// `letter-spacing`）を継承してしまうため、同型のリセットを適用する。
///
/// # `body` の `max-height` 上書き（モジュール doc 参照）
///
/// [`fandhe_frontend_pre_styled_ui::dialog::body`] の既定 `max-height: 50vh;
/// overflow-y: auto` を打ち消し、入力欄 2 件のみの本 Demo でスクロール
/// バーが生じる事態を防ぐ。
///
/// # 狭幅対応
///
/// `@media (max-width: 47.99rem)`（リポジトリの慣例的なブレークポイント）
/// でダイアログ positioner の padding を縮め、footer のボタンを縦積み・
/// 全幅にする。
const LAYOUT_CSS: &str = "\
.blocks-empty-state-card-header-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n}\n\
[data-blocks-empty-state-card-header-card] {\n  width: 100%;\n}\n\
.blocks-empty-state-card-header.blocks-demo {\n  overflow: visible;\n}\n\
[data-blocks-empty-state-card-header-dialog-root] {\n  position: relative;\n}\n\
.blocks-empty-state-card-header [data-scope=\"dialog\"] h2 {\n  border-top: none;\n  padding-top: 0;\n  letter-spacing: normal;\n}\n\
.blocks-empty-state-card-header [data-scope=\"dialog\"][data-part=\"backdrop\"] {\n  position: absolute;\n  inset: 0;\n  z-index: auto;\n  border-radius: var(--fandhe-radius-lg);\n  background: var(--fandhe-color-bg-subtle);\n}\n\
.blocks-empty-state-card-header [data-scope=\"dialog\"][data-part=\"positioner\"] {\n  position: relative;\n  inset: auto;\n  z-index: auto;\n  width: 100%;\n  min-height: 20rem;\n  display: flex;\n  align-items: center;\n  justify-content: center;\n  padding: var(--fandhe-space-6);\n}\n\
.blocks-empty-state-card-header [data-scope=\"dialog\"][data-part=\"body\"] {\n  max-height: none;\n  overflow: visible;\n}\n\
[data-blocks-empty-state-card-header-dialog-footer] {\n  display: flex;\n  justify-content: flex-end;\n  gap: var(--fandhe-space-3);\n}\n\
@media (max-width: 47.99rem) {\n  .blocks-empty-state-card-header [data-scope=\"dialog\"][data-part=\"positioner\"] {\n    padding: var(--fandhe-space-3);\n  }\n  [data-blocks-empty-state-card-header-dialog-footer] {\n    flex-direction: column-reverse;\n  }\n  [data-blocks-empty-state-card-header-dialog-footer] [data-scope=\"button\"] {\n    width: 100%;\n  }\n}\n";

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;

    fn demo_html() -> String {
        render(&demo())
    }

    /// Demo が使用部品（card/empty-state/button/dialog/field/input/
    /// native-select）の anatomy をすべて実際に出力していることを固定する
    /// （`contact_dialog_form` 先例と同型）。
    #[test]
    fn demo_composes_expected_parts() {
        let html = demo_html();
        for scope in [
            "data-scope=\"card\" data-part=\"root\"",
            "data-scope=\"card\" data-part=\"header\"",
            "data-scope=\"card\" data-part=\"title\"",
            "data-scope=\"card\" data-part=\"action\"",
            "data-scope=\"card\" data-part=\"body\"",
            "data-scope=\"empty-state\" data-part=\"root\"",
            "data-scope=\"empty-state\" data-part=\"indicator\"",
            "data-scope=\"empty-state\" data-part=\"title\"",
            "data-scope=\"empty-state\" data-part=\"description\"",
            "data-scope=\"empty-state\" data-part=\"actions\"",
            "data-scope=\"dialog\" data-part=\"backdrop\"",
            "data-scope=\"dialog\" data-part=\"positioner\"",
            "data-scope=\"dialog\" data-part=\"content\"",
            "data-scope=\"dialog\" data-part=\"title\"",
            "data-scope=\"dialog\" data-part=\"description\"",
            "data-scope=\"dialog\" data-part=\"body\"",
            "data-scope=\"dialog\" data-part=\"footer\"",
            "data-scope=\"field\" data-part=\"input\"",
            "data-scope=\"field\" data-part=\"select\"",
            "data-scope=\"button\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
    }

    /// `<form>` を出力せず、ボタンがすべて `type="button"` であることを
    /// 固定する（`crate::blocks` モジュール doc の不変条件）。
    #[test]
    fn demo_has_no_form_and_only_button_type_buttons() {
        let html = demo_html();
        assert!(!html.contains("<form"), "demo should never contain <form");
        for absent in ["type=\"submit\"", "form=\"", "href=", "src=\"data:"] {
            assert!(!html.contains(absent), "demo should never contain {absent}");
        }
        // `data-has-action` は `card::header` の opt-in 属性（フォーム送信の
        // `action` 属性ではない）。単純な `contains("action=")` は
        // `data-has-action=""` を誤検知するため、フォームの `action=`
        // 属性を先頭が英数字境界（`<tag ... action="..."`）で厳密に検知する。
        assert!(
            !html.contains(" action=\""),
            "demo should never contain a form action= attribute"
        );
        assert_eq!(
            html.matches("type=\"button\"").count(),
            4,
            "demo should have exactly 4 type=\"button\" buttons \
             (card header create / empty-state create / dialog cancel / dialog create)"
        );
    }

    /// 静的な開状態・非モーダル（`aria-modal=false`）であることを固定する
    /// （`contact_dialog_form` と同じ設計判断、モジュール doc 参照）。
    #[test]
    fn dialog_is_open_static_and_non_modal() {
        let html = demo_html();
        assert!(html.contains("aria-modal=\"false\""));
        assert!(html.contains("data-state=\"open\""));
        // backdrop の `aria-hidden="true"`（装飾層のため常時付与）に
        // "hidden" 部分文字列がたまたま含まれるため、素朴な
        // `contains("hidden")` は誤検知する。closed 時のみ付与される
        // 存在属性 `hidden` を精確に検知する。
        assert!(!html.contains(" hidden>"));
        assert!(!html.contains(" hidden "));
    }

    /// name/visibility の `label[for]` と、各コントロールの `id` が
    /// 一致することを固定する（アクセシビリティ上の関連付け不変条件）。
    #[test]
    fn labels_point_at_their_controls() {
        let html = demo_html();
        for (label_for, control_id) in [
            (
                "for=\"blocks-empty-state-card-header-name-control\"",
                "id=\"blocks-empty-state-card-header-name-control\"",
            ),
            (
                "for=\"blocks-empty-state-card-header-visibility-control\"",
                "id=\"blocks-empty-state-card-header-visibility-control\"",
            ),
        ] {
            assert!(html.contains(label_for), "missing {label_for}");
            assert!(html.contains(control_id), "missing {control_id}");
        }
    }

    /// 各 field/button の id が block 内で一意であることを固定する
    /// （`demo_output_has_no_dangling_aria_references_or_duplicate_ids`
    /// 契約と同種の観点をローカルでも固定する）。
    #[test]
    fn field_ids_are_unique() {
        let html = demo_html();
        for id_attr in [
            "id=\"blocks-empty-state-card-header-name-control\"",
            "id=\"blocks-empty-state-card-header-visibility-control\"",
            "id=\"blocks-empty-state-card-header-dialog-content\"",
        ] {
            assert_eq!(
                html.matches(id_attr).count(),
                1,
                "{id_attr} should appear exactly once"
            );
        }
    }

    /// [`LAYOUT_CSS`] が固定オーバーレイの中和・狭幅ブレークポイント・
    /// body の max-height 上書き・カード全幅指定を含み、`<` を含まない
    /// ことを固定する（REQ-1: `</style>` によるスタイル脱出を防ぐ）。
    #[test]
    fn layout_css_neutralizes_fixed_overlay_and_declares_narrow_breakpoint() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("position: relative;\n  inset: auto;"));
        assert!(
            LAYOUT_CSS.contains("[data-blocks-empty-state-card-header-card] {\n  width: 100%;\n}")
        );
        assert!(LAYOUT_CSS.contains("max-height: none;"));
        assert!(LAYOUT_CSS.contains("@media (max-width: 47.99rem)"));
    }
}
