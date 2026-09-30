# auth-tabs-card

`fandhe-frontend-pre-styled-ui` の `card` / `tabs` / `field` / `input` /
`button` / `separator` / `dialog` の 7 部品を合成した、ログイン／新規登録
タブ付きカードの実例です。Blocks セクションは新規部品を追加するものでは
なく、既存の Themes/Primitives 部品を組み合わせた実例集であることに注意
してください（主参照は対応表 ID R0690・R0691・R0032。出典の固有名・
ファイル名は記載しません）。

docs サイトは JS ハイドレーションを行わないため、タブは切り替えられません。
このため選択状態が異なる 3 インスタンスを縦に並べ、代表構成（ログイン選択）
・カード直下にタブを密着させた構成（新規登録選択）・ダイアログ内 + ソーシャル
ログインボタン付きの構成（ログイン選択）を並記しています。

本 Demo は静的な表示例であり、`<form>` 要素は一切持たず、データの取得・
送信・認証処理を行いません。ボタンは `type="button"` のまま送信先を持たず、
パスワード欄は `type="password"` で値を持ちません。ソーシャルログイン
ボタンは実ブランドロゴを複製せず「provider A/B」という一般的な表記に
とどめています。ダイアログは既に開いた静的な状態のみを描き、外側に説明・
コードがあるため `aria-modal` を `false` にしています。

## Rust コード

```rust
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
```

## 原案差分メモ

- 主参照 R0690（カード + ヘッダー + タブの代表構成）を形 A とし、R0032
  （カード直下にタブを密着させる構成）を形 B、R0691（ダイアログ内 +
  ソーシャルログイン）を形 C として統合しています。
- Issue が使用部品候補に挙げた `link` 部品は使いません。遷移先を持たない
  「パスワードを忘れた」リンクは、既存 Auth block（`login-01`/`login-04`）
  と同じく見た目だけリンク風にする `ButtonVariant::Link` で表現しており、
  `href="#"` を禁止する既存契約（`blocks_contract.rs`）を優先しています。
- ソーシャルログインボタンはアイコン・実ブランドロゴを持たず、テキストの
  みで表現しています。
- ブラウザ実機確認は未実施、`cargo test` で代替しています。
