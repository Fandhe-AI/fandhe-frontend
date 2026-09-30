//! `auth-tabs-card` block（イシュー #2967。ログイン／新規登録をカード
//! 上部のタブで切り替える合成例。9 件目の Auth block）。
//!
//! # 使用部品
//!
//! `card`（構造）/ `tabs`（ログイン・新規登録の切り替え）/ `field`（`group`/
//! `root`/`label`/`helper_text`）+ `input`（メール・氏名・パスワード入力）/
//! `button`（Solid 送信・Outline ソーシャルログイン・リンク風アクション）/
//! `separator`（`group`/`label` によるソーシャルログイン区切り）/ `dialog`
//! （ダイアログ内バリエーション）の 7 部品を合成する（`Block::parts` に
//! 一致させる契約、`crates/docs-site/tests/blocks_nav.rs`/
//! `blocks_contract.rs` が検証する）。`link` 部品は使わない（後述）。
//!
//! # 3 インスタンス構成（集約元 3 件との対応）
//!
//! 無 JS の docs サイトではタブは切り替えられない（`headless-ui` の
//! `tabs` は非選択 `content` に `hidden` を付与するのみ）ため、選択状態が
//! 異なる 3 インスタンスを縦に並べて全パターンを静的に提示する
//! （`pricing_tiers_comparison::billing_band` と同型の設計判断）。
//!
//! - **形 A**（ログイン選択）: `card::header` + `card::body` 内にタブを
//!   置く代表構成
//! - **形 B**（新規登録選択）: `card::header` を持たず、タブをカード上端に
//!   密着させる構成
//! - **形 C**（ログイン選択）: `dialog` 内にタブを置き、各タブ content の
//!   先頭にソーシャルログインボタン + 「または」区切りを持つ構成
//!
//! # `<form>` を使わない・認証処理を持たない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力せず、送信ボタンは `button::button` の既定 `type="button"` のまま
//! 用いる。値は一切送信されず、認証処理も行わない静的な合成例である
//! （`docs/policy/intentional-non-adoption.md` §3.25）。
//!
//! # `link` 部品を使わない理由
//!
//! `crates/docs-site/tests/blocks_contract.rs` は `href="#"` を禁止する。
//! 「パスワードを忘れた」等の遷移先を持たないリンクは、既存 Auth block
//! （`login_01`/`login_04`）と同じく見た目だけリンク風にする
//! `button::ButtonVariant::Link`（`<button type="button">` のまま）で表現
//! する。Issue が挙げた使用部品候補からの逸脱だが、既存契約を優先する。
//!
//! # ダイアログは静的な開状態・非モーダル
//!
//! `contact_dialog_form`/`game_ui_modal` と同じ設計判断で、`trigger`/
//! `close_trigger` は置かず、`aria-modal` は `false` にする（外側に説明・
//! コードがあるため）。開閉・フォーカストラップは扱わない。
//!
//! # ソーシャルログインボタンは実ブランドロゴを複製しない
//!
//! `login_04` と同じ方針で、プロバイダ名は一般的な「provider A/B」表記に
//! とどめる。テキストのみで、アイコン・実ブランドロゴは持たない。
use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, p, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::dialog::{self, ContentIds, DialogRole, OpenState};
use fandhe_frontend_pre_styled_ui::field::{self, FieldOrientation, FieldRootProps};
use fandhe_frontend_pre_styled_ui::input::{self, FieldIds, FieldProps, InputProps};
use fandhe_frontend_pre_styled_ui::separator::{self, SeparatorProps};
use fandhe_frontend_pre_styled_ui::tabs::{
    self, ActivationMode, Orientation, TabItem, TabsProps, TabsVariant,
};
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// `id` サフィックス（`"a"`/`"b"`/`"c"`）ごとに一意な `FieldProps` を作る
/// （3 インスタンスで DOM id を衝突させないため）。
fn field_props(id: &'static str) -> FieldProps<'static> {
    FieldProps {
        id,
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: true,
        readonly: false,
        has_helper_text: false,
    }
}

/// ログインフォーム本体（メール・パスワード + 送信）。
fn login_form(suffix: &'static str) -> Vec<Node> {
    let orientation = FieldRootProps {
        orientation: FieldOrientation::Vertical,
    };
    let link_button = ButtonProps {
        variant: ButtonVariant::Link,
        ..ButtonProps::default()
    };
    let email_field = field_props(match suffix {
        "a" => "blocks-auth-tabs-card-a-email",
        "b" => "blocks-auth-tabs-card-b-email",
        _ => "blocks-auth-tabs-card-c-email",
    });
    let password_field = field_props(match suffix {
        "a" => "blocks-auth-tabs-card-a-password",
        "b" => "blocks-auth-tabs-card-b-password",
        _ => "blocks-auth-tabs-card-c-password",
    });

    vec![field::group(
        vec![],
        vec![
            field::root(
                &orientation,
                &email_field,
                vec![("data-blocks-auth-tabs-card-field", "")],
                vec![
                    field::label(&email_field, vec![], vec![text("Email")]),
                    input::input(
                        &InputProps::default(),
                        &email_field,
                        vec![("type", "email"), ("placeholder", "you@example.com")],
                    ),
                ],
            ),
            field::root(
                &orientation,
                &password_field,
                vec![("data-blocks-auth-tabs-card-field", "")],
                vec![
                    div(
                        vec![("class", "blocks-auth-tabs-card-password-row")],
                        vec![
                            field::label(&password_field, vec![], vec![text("Password")]),
                            button::button(&link_button, vec![], vec![text("Forgot password?")]),
                        ],
                    ),
                    input::input(
                        &InputProps::default(),
                        &password_field,
                        vec![("type", "password")],
                    ),
                ],
            ),
            button::button(
                &ButtonProps::default(),
                vec![("data-blocks-auth-tabs-card-submit", "")],
                vec![text("Sign in")],
            ),
        ],
    )]
}

/// 新規登録フォーム本体（氏名・メール・パスワード + 送信）。
fn signup_form(suffix: &'static str) -> Vec<Node> {
    let orientation = FieldRootProps {
        orientation: FieldOrientation::Vertical,
    };
    let name_field = field_props(match suffix {
        "a" => "blocks-auth-tabs-card-a-name",
        "b" => "blocks-auth-tabs-card-b-name",
        _ => "blocks-auth-tabs-card-c-name",
    });
    let email_field = field_props(match suffix {
        "a" => "blocks-auth-tabs-card-a-signup-email",
        "b" => "blocks-auth-tabs-card-b-signup-email",
        _ => "blocks-auth-tabs-card-c-signup-email",
    });
    let mut password_field = field_props(match suffix {
        "a" => "blocks-auth-tabs-card-a-signup-password",
        "b" => "blocks-auth-tabs-card-b-signup-password",
        _ => "blocks-auth-tabs-card-c-signup-password",
    });
    password_field.has_helper_text = true;

    vec![field::group(
        vec![],
        vec![
            field::root(
                &orientation,
                &name_field,
                vec![("data-blocks-auth-tabs-card-field", "")],
                vec![
                    field::label(&name_field, vec![], vec![text("Name")]),
                    input::input(
                        &InputProps::default(),
                        &name_field,
                        vec![("type", "text"), ("placeholder", "Jane Doe")],
                    ),
                ],
            ),
            field::root(
                &orientation,
                &email_field,
                vec![("data-blocks-auth-tabs-card-field", "")],
                vec![
                    field::label(&email_field, vec![], vec![text("Email")]),
                    input::input(
                        &InputProps::default(),
                        &email_field,
                        vec![("type", "email"), ("placeholder", "you@example.com")],
                    ),
                ],
            ),
            field::root(
                &orientation,
                &password_field,
                vec![("data-blocks-auth-tabs-card-field", "")],
                vec![
                    field::label(&password_field, vec![], vec![text("Password")]),
                    input::input(
                        &InputProps::default(),
                        &password_field,
                        vec![("type", "password")],
                    ),
                    field::helper_text(
                        &password_field,
                        vec![],
                        vec![text("Must be at least 8 characters long.")],
                    ),
                ],
            ),
            button::button(
                &ButtonProps::default(),
                vec![("data-blocks-auth-tabs-card-submit", "")],
                vec![text("Create account")],
            ),
        ],
    )]
}

/// ソーシャルログインボタン 2 個（実ブランドロゴは複製しない、モジュール
/// doc「ソーシャルログインボタンは実ブランドロゴを複製しない」節参照）。
fn social_row() -> Node {
    let outline_button = ButtonProps {
        variant: ButtonVariant::Outline,
        ..ButtonProps::default()
    };
    div(
        vec![("data-blocks-auth-tabs-card-social", "")],
        vec![
            button::button(
                &outline_button,
                vec![],
                vec![text("Continue with provider A")],
            ),
            button::button(
                &outline_button,
                vec![],
                vec![text("Continue with provider B")],
            ),
        ],
    )
}

/// 「または」区切り線（`separator::group`）。
fn or_separator() -> Node {
    separator::group(
        vec![],
        vec![
            separator::separator(&SeparatorProps::default(), vec![]),
            separator::label(vec![], vec![text("or")]),
            separator::separator(&SeparatorProps::default(), vec![]),
        ],
    )
}

/// ログインタブ（`social` が true ならソーシャルログイン + 区切りを先頭に
/// 追加する、形 C 用）。
fn login_tab(suffix: &'static str, social: bool) -> TabItem<'static> {
    let mut content = Vec::new();
    if social {
        content.push(social_row());
        content.push(or_separator());
    }
    content.extend(login_form(suffix));
    TabItem {
        value: "login",
        trigger: vec![text("Sign in")],
        content,
        disabled: false,
    }
}

/// 新規登録タブ（`social` が true ならソーシャルログイン + 区切りを先頭に
/// 追加する、形 C 用）。
fn signup_tab(suffix: &'static str, social: bool) -> TabItem<'static> {
    let mut content = Vec::new();
    if social {
        content.push(social_row());
        content.push(or_separator());
    }
    content.extend(signup_form(suffix));
    TabItem {
        value: "signup",
        trigger: vec![text("Sign up")],
        content,
        disabled: false,
    }
}

/// キャプション 1 行（muted な `p`、`pricing_tiers_comparison::caption` と
/// 同型）。
fn caption(label: &'static str) -> Node {
    p(
        vec![("class", "blocks-auth-tabs-card-caption")],
        vec![text(label)],
    )
}

/// 形 A: `card::header` + `card::body` 内にタブを置く代表構成（ログイン
/// 選択）。
fn form_a() -> Node {
    let props = TabsProps {
        id: "blocks-auth-tabs-card-a",
        selected: "login",
        orientation: Orientation::Horizontal,
        activation_mode: ActivationMode::Automatic,
        loop_focus: true,
        indicator: false,
    };
    card::root(
        CardProps::default(),
        vec![("data-blocks-auth-tabs-card-card", "")],
        vec![
            card::header(
                vec![],
                vec![
                    card::title(vec![], vec![text("Welcome")]),
                    card::description(
                        vec![],
                        vec![text("Sign in to your account or create a new one")],
                    ),
                ],
            ),
            card::body(
                vec![],
                vec![tabs::tabs(
                    TabsVariant::Enclosed,
                    Size::Md,
                    ColorPalette::Accent,
                    &props,
                    vec![login_tab("a", false), signup_tab("a", false)],
                )],
            ),
        ],
    )
}

/// 形 B: `card::header` を持たず、タブをカード上端に密着させる構成（新規
/// 登録選択）。
fn form_b() -> Node {
    let props = TabsProps {
        id: "blocks-auth-tabs-card-b",
        selected: "signup",
        orientation: Orientation::Horizontal,
        activation_mode: ActivationMode::Automatic,
        loop_focus: true,
        indicator: false,
    };
    card::root(
        CardProps::default(),
        vec![
            ("data-blocks-auth-tabs-card-card", ""),
            ("data-blocks-auth-tabs-card-flush", ""),
        ],
        vec![div(
            vec![("data-blocks-auth-tabs-card-panel", "")],
            vec![tabs::tabs(
                TabsVariant::Enclosed,
                Size::Md,
                ColorPalette::Accent,
                &props,
                vec![login_tab("b", false), signup_tab("b", false)],
            )],
        )],
    )
}

/// 形 C: `dialog` 内にタブを置き、各タブ content 先頭にソーシャルログイン
/// と区切りを持つ構成（ログイン選択。モジュール doc「ダイアログは静的な
/// 開状態・非モーダル」節参照）。
fn form_c() -> Node {
    let title_id = "blocks-auth-tabs-card-c-title";
    let description_id = "blocks-auth-tabs-card-c-description";
    let props = TabsProps {
        id: "blocks-auth-tabs-card-c",
        selected: "login",
        orientation: Orientation::Horizontal,
        activation_mode: ActivationMode::Automatic,
        loop_focus: true,
        indicator: false,
    };

    dialog::root(
        Size::Md,
        OpenState::Open,
        vec![("data-blocks-auth-tabs-card-dialog-root", "")],
        vec![
            dialog::backdrop(OpenState::Open, vec![], vec![]),
            dialog::positioner(
                OpenState::Open,
                vec![],
                vec![dialog::content(
                    OpenState::Open,
                    DialogRole::Dialog,
                    false,
                    ContentIds {
                        id: Some("blocks-auth-tabs-card-c-content"),
                        labelledby: Some(title_id),
                        describedby: Some(description_id),
                    },
                    vec![("data-blocks-auth-tabs-card-content", "")],
                    vec![
                        dialog::title(Some(title_id), vec![], vec![text("Welcome back")]),
                        dialog::description(
                            Some(description_id),
                            vec![],
                            vec![text("Sign in or create an account to continue")],
                        ),
                        dialog::body(
                            vec![],
                            vec![tabs::tabs(
                                TabsVariant::Enclosed,
                                Size::Md,
                                ColorPalette::Accent,
                                &props,
                                vec![login_tab("c", true), signup_tab("c", true)],
                            )],
                        ),
                    ],
                )],
            ),
        ],
    )
}

/// `auth-tabs-card` の Demo 本体。3 インスタンスを縦に並べる。
pub fn demo() -> Node {
    div(
        vec![("data-blocks-auth-tabs-card-stack", "")],
        vec![
            caption("代表構成（ログイン選択）"),
            form_a(),
            caption("カード直下にタブ（新規登録選択）"),
            form_b(),
            caption("ダイアログ内 + ソーシャルログイン（ログイン選択）"),
            form_c(),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/auth-tabs-card/",
    title: "auth-tabs-card",
    category: BlockCategory::Auth,
    rust_source: "crates/docs-site/src/blocks/application/auth/auth_tabs_card.rs",
    demo_class: "blocks-auth-tabs-card",
    parts: &[
        Part {
            label: "Card",
            path: "/themes/card/",
        },
        Part {
            label: "Tabs",
            path: "/themes/tabs/",
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
            label: "Separator",
            path: "/themes/separator/",
        },
        Part {
            label: "Dialog",
            path: "/themes/dialog/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `auth_tabs_card` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS` doc
/// 「block 固有 CSS の置き場」節）。
///
/// # 形 B: Card recipe の `padding`/`display: flex` を上書きする詳細度
///
/// `card::root` の既定 `body`（ここでは使わず `card::root` 直下へ `div` を
/// 置く）ではなく、`[data-scope="card"][data-part="root"]`
/// （詳細度 (0,2,0)）自身へ `padding: 0` を当てて Card recipe の既定余白を
/// 消す。`[data-blocks-auth-tabs-card-flush]` 単独（(0,1,0)）では recipe に
/// 負けるため、`login_04` と同型で `[data-scope="card"][data-part="root"]`
/// を結合したセレクタ（(0,3,0)）にする。
///
/// タブ一覧をカード上端に密着させるため、`[data-blocks-auth-tabs-card-panel]`
/// （タブ一覧 + フォームを包む div）の `padding-top` のみ 0 にし、左右・下は
/// `--fandhe-space-6` を維持する（`padding: 0 var(--fandhe-space-6)
/// var(--fandhe-space-6);`）。全方向 padding のままだとタブ一覧の上にも
/// 余白が残り、`card::root` の padding を 0 にした意味が失われる（PR #3426
/// レビュー指摘）。
///
/// # 形 C: `contact_dialog_form` と同じ固定オーバーレイの中和
///
/// `dialog::positioner`/`backdrop` は本来 `position: fixed; inset: 0` の
/// ビューポート全体オーバーレイだが、Blocks の掲示は `.blocks-demo` 枠内へ
/// 収める必要がある。`contact_dialog_form` の判断（実コンテンツ側
/// `positioner` が高さを決め、`backdrop` がそれに追随する）をそのまま踏襲
/// する。
const LAYOUT_CSS: &str = "\
.blocks-auth-tabs-card.blocks-demo {\n  overflow: visible;\n}\n\
[data-blocks-auth-tabs-card-stack] {\n  display: flex;\n  flex-direction: column;\n  align-items: center;\n  gap: var(--fandhe-space-6);\n}\n\
.blocks-auth-tabs-card-caption {\n  font-size: var(--fandhe-font-font-size-sm);\n  color: var(--fandhe-color-fg-muted);\n  margin: 0;\n}\n\
[data-blocks-auth-tabs-card-card] {\n  width: 100%;\n  max-width: 28rem;\n}\n\
.blocks-auth-tabs-card [data-scope=\"tabs\"][data-part=\"list\"] {\n  display: grid;\n  grid-template-columns: 1fr 1fr;\n}\n\
[data-scope=\"card\"][data-part=\"root\"][data-blocks-auth-tabs-card-flush] {\n  padding: 0;\n  overflow: hidden;\n}\n\
[data-blocks-auth-tabs-card-panel] {\n  padding: 0 var(--fandhe-space-6) var(--fandhe-space-6);\n}\n\
[data-blocks-auth-tabs-card-field] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-auth-tabs-card-password-row {\n  display: flex;\n  justify-content: space-between;\n  align-items: baseline;\n  gap: var(--fandhe-space-2);\n}\n\
[data-scope=\"button\"][data-part=\"root\"][data-blocks-auth-tabs-card-submit] {\n  width: 100%;\n}\n\
[data-blocks-auth-tabs-card-social] {\n  display: grid;\n  grid-template-columns: 1fr 1fr;\n  gap: var(--fandhe-space-3);\n}\n\
[data-blocks-auth-tabs-card-dialog-root] {\n  position: relative;\n}\n\
.blocks-auth-tabs-card [data-scope=\"dialog\"] h2 {\n  border-top: none;\n  padding-top: 0;\n  letter-spacing: normal;\n}\n\
.blocks-auth-tabs-card [data-scope=\"dialog\"][data-part=\"backdrop\"] {\n  position: absolute;\n  inset: 0;\n  z-index: auto;\n  border-radius: var(--fandhe-radius-lg);\n  background: var(--fandhe-color-bg-subtle);\n}\n\
.blocks-auth-tabs-card [data-scope=\"dialog\"][data-part=\"positioner\"] {\n  position: relative;\n  inset: auto;\n  z-index: auto;\n  width: 100%;\n  display: flex;\n  justify-content: center;\n  padding: var(--fandhe-space-6);\n}\n\
.blocks-auth-tabs-card [data-scope=\"dialog\"][data-part=\"body\"] {\n  max-height: none;\n  overflow: visible;\n}\n\
@media (max-width: 47.99rem) {\n  [data-blocks-auth-tabs-card-social] {\n    grid-template-columns: 1fr;\n  }\n  .blocks-auth-tabs-card [data-scope=\"dialog\"][data-part=\"positioner\"] {\n    padding: var(--fandhe-space-3);\n  }\n}\n";

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;

    fn demo_html() -> String {
        render(&demo())
    }

    /// Demo が使用部品（card/tabs/field/input/button/separator/dialog）の
    /// anatomy をすべて実際に出力していることを固定する。
    #[test]
    fn demo_composes_expected_parts() {
        let html = demo_html();
        for scope in [
            "data-scope=\"card\"",
            "data-scope=\"tabs\"",
            "data-scope=\"field\" data-part=\"root\"",
            "data-scope=\"field\" data-part=\"input\"",
            "data-scope=\"button\"",
            "data-scope=\"separator\"",
            "data-scope=\"dialog\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
    }

    /// 3 インスタンスがそれぞれ意図した選択状態（形 A: login / 形 B:
    /// signup / 形 C: login）を持ち、非選択 content には `hidden` が
    /// 付与されることを固定する。
    #[test]
    fn three_tab_instances_with_expected_selection() {
        let html = demo_html();
        assert!(html.contains(
            r#"id="blocks-auth-tabs-card-a-trigger-login" role="tab" aria-selected="true""#
        ));
        assert!(html.contains(
            r#"id="blocks-auth-tabs-card-b-trigger-signup" role="tab" aria-selected="true""#
        ));
        assert!(html.contains(
            r#"id="blocks-auth-tabs-card-c-trigger-login" role="tab" aria-selected="true""#
        ));
        assert_eq!(
            html.matches("hidden=\"\"").count(),
            3,
            "non-selected content (a-signup/b-login/c-signup) should each be hidden once"
        );
    }

    /// `<form>` を使わず、死リンク（`href="#"`）・`data:` URI を出さず、
    /// 全ボタンが `type="button"` であることを固定する
    /// （`crate::blocks` モジュール doc の不変条件）。
    #[test]
    fn no_form_semantics_or_dead_links() {
        let html = demo_html();
        assert!(!html.contains("<form"));
        for absent in ["href=\"#\"", "src=\"data:", "type=\"submit\"", "action="] {
            assert!(!html.contains(absent), "demo should never contain {absent}");
        }
        let button_open_tags = html.matches("<button").count();
        let type_button_tags = html.matches("type=\"button\"").count();
        assert_eq!(
            button_open_tags, type_button_tags,
            "every <button> should carry type=\"button\""
        );
    }

    /// ダイアログ（形 C）が静的な開状態・非モーダルであることを固定する
    /// （`contact_dialog_form` と同じ設計判断）。
    #[test]
    fn dialog_is_open_static_and_non_modal() {
        let html = demo_html();
        assert!(html.contains("aria-modal=\"false\""));
        assert!(html.contains(r#"id="blocks-auth-tabs-card-c-content""#));
    }

    /// 3 インスタンス分の `id` に重複がないことを固定する（`login`/
    /// `signup` の value 共有はインスタンス間で `id` が衝突しない設計に
    /// 依存するため）。
    #[test]
    fn ids_have_no_duplicates() {
        let html = demo_html();
        let mut ids = Vec::new();
        let mut rest = html.as_str();
        while let Some(idx) = rest.find("id=\"") {
            rest = &rest[idx + 4..];
            if let Some(end) = rest.find('"') {
                ids.push(&rest[..end]);
                rest = &rest[end + 1..];
            } else {
                break;
            }
        }
        let mut sorted = ids.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), ids.len(), "duplicate id found among: {ids:?}");
    }

    /// [`LAYOUT_CSS`] が `<`・NUL を含まず（REQ-1: `</style>` 脱出防止）、
    /// block スコープの主要セレクタを含むことを固定する。
    #[test]
    fn layout_css_is_safe() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(!LAYOUT_CSS.contains('\0'));
        assert!(LAYOUT_CSS.contains(".blocks-auth-tabs-card"));
    }

    /// Demo が呼び出しごとに同一の HTML を返す決定的な純関数であることを
    /// 固定する。
    #[test]
    fn demo_is_deterministic() {
        assert_eq!(demo_html(), demo_html());
    }
}
