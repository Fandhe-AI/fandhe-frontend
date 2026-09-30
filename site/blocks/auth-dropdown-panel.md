# auth-dropdown-panel

`fandhe-frontend-pre-styled-ui` の `popover` / `menu` / `field` / `input` /
`button` / `link` を合成した、ナビバーのボタンを起点に開くドロップダウン型
サインインパネルです（主参照は対応表 ID R0685、集約元 1 件のみのため個別の
差分メモはありません）。

Demo は `wide`（既定幅、パネルをトリガー右揃えで絶対配置）と `narrow`
（Demo 枠を強制的に `22rem` 未満へ固定した狭幅、パネルをヘッダー幅いっぱいに
広げてトリガー直下へ表示）の 2 レイアウトを縦に並記します。いずれもサインイン
パネルを開いた状態のまま静的に固定し、ヘルプメニューは閉じた状態で併記して
`popover`/`menu` 2 種類のオーバーレイ trigger の見た目を区別できるように
しています。

本 Demo は静的表示例です。docs サイトは JS ハイドレーションを行わないため
サインインボタン・ヘルプメニューの trigger はいずれも無効化しています。
`<form>` は使わず、パスワード入力欄も値・送信先を持ちません（`type="button"`
のまま、実際の認証処理は利用者自身の Rust コードで実装してください）。
「パスワードをお忘れですか」は遷移先を持たないため見た目だけリンク風の
ボタン、「新規登録」は実在する [signup-01](./signup-01.md) block ページへの
リンクです。文言はすべて架空のものです。対応表 ID の実物（取得手段・
ファイル名）は本原稿・実装ソースに記載しません。

## Rust コード

```rust
use fandhe_frontend_core::{div, header, span, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::field::{self, FieldOrientation, FieldRootProps};
use fandhe_frontend_pre_styled_ui::input::{self, FieldIds, FieldProps, InputProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::menu;
use fandhe_frontend_pre_styled_ui::popover::{self, OpenState};
use fandhe_frontend_pre_styled_ui::Size;

/// ロゴ（ブランド名テキストのみ、リンクにしない）。
fn logo() -> Node {
    span(
        vec![("class", "blocks-auth-dropdown-panel-logo")],
        vec![text("Fandhe Console")],
    )
}

/// ヘルプメニュー（閉状態固定、`disabled: true`）。`variant` は id の
/// suffix（モジュール doc「id と ARIA の一意性」節）。
fn help_menu(variant: &str) -> Node {
    let trigger_id = format!("blocks-auth-dropdown-panel-menu-trigger-{variant}");
    let content_id = format!("blocks-auth-dropdown-panel-menu-{variant}");
    let trigger = menu::trigger(
        OpenState::Closed,
        true,
        Some(content_id.as_str()),
        vec![
            ("id", trigger_id.as_str()),
            ("data-blocks-auth-dropdown-panel-menu-trigger", ""),
        ],
        vec![text("ヘルプ")],
    );
    let content = menu::content(
        OpenState::Closed,
        Some(content_id.as_str()),
        Some(trigger_id.as_str()),
        vec![],
        vec![
            menu::item("docs", false, false, vec![], vec![text("ドキュメント")]),
            menu::item("contact", false, false, vec![], vec![text("お問い合わせ")]),
            menu::item("status", false, false, vec![], vec![text("稼働状況")]),
        ],
    );
    let positioner = menu::positioner(OpenState::Closed, vec![], vec![content]);
    menu::root(
        Size::Sm,
        OpenState::Closed,
        vec![],
        vec![trigger, positioner],
    )
}

/// メール・パスワード入力欄一式（[`FieldOrientation::Vertical`] で縦積み）。
/// `variant` は id の suffix。
fn credential_fields(variant: &str) -> Vec<Node> {
    let orientation = FieldRootProps {
        orientation: FieldOrientation::Vertical,
    };
    let email_id = format!("blocks-auth-dropdown-panel-email-{variant}");
    let email_field = FieldProps {
        id: &email_id,
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: true,
        readonly: false,
        has_helper_text: false,
    };
    let password_id = format!("blocks-auth-dropdown-panel-password-{variant}");
    let password_field = FieldProps {
        id: &password_id,
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: true,
        readonly: false,
        has_helper_text: false,
    };
    vec![
        field::root(
            &orientation,
            &email_field,
            vec![("data-blocks-auth-dropdown-panel-field", "")],
            vec![
                field::label(&email_field, vec![], vec![text("メールアドレス")]),
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
            vec![("data-blocks-auth-dropdown-panel-field", "")],
            vec![
                field::label(&password_field, vec![], vec![text("パスワード")]),
                input::input(
                    &InputProps::default(),
                    &password_field,
                    vec![("type", "password")],
                ),
            ],
        ),
    ]
}

/// 補助導線行（パスワード再設定・新規登録）。
fn auxiliary_links() -> Node {
    div(
        vec![("data-blocks-auth-dropdown-panel-links", "")],
        vec![
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Link,
                    ..ButtonProps::default()
                },
                vec![],
                vec![text("パスワードをお忘れですか")],
            ),
            span(
                vec![],
                vec![
                    text("アカウントをお持ちでない方は "),
                    link::root(
                        "../signup-01/",
                        &LinkProps::default(),
                        vec![],
                        vec![text("新規登録")],
                    ),
                ],
            ),
        ],
    )
}

/// サインインパネル（`popover`、開状態固定・`disabled: true`）。`variant`
/// は id の suffix。
fn sign_in_panel(variant: &str) -> Node {
    let trigger_id = format!("blocks-auth-dropdown-panel-trigger-{variant}");
    let panel_id = format!("blocks-auth-dropdown-panel-panel-{variant}");
    let title_id = format!("blocks-auth-dropdown-panel-title-{variant}");

    let trigger = popover::trigger(
        OpenState::Open,
        true,
        Some(panel_id.as_str()),
        vec![
            ("id", trigger_id.as_str()),
            ("data-blocks-auth-dropdown-panel-trigger", ""),
        ],
        vec![text("サインイン")],
    );

    let mut panel_children: Vec<Node> = vec![
        popover::title(Some(title_id.as_str()), vec![], vec![text("サインイン")]),
        popover::description(
            None,
            vec![],
            vec![text("登録済みのメールアドレスでサインインします")],
        ),
    ];
    panel_children.extend(credential_fields(variant));
    panel_children.push(button::button(
        &ButtonProps::default(),
        vec![("data-blocks-auth-dropdown-panel-submit", "")],
        vec![text("サインイン")],
    ));
    panel_children.push(auxiliary_links());

    let content = popover::content(
        OpenState::Open,
        Some(panel_id.as_str()),
        Some(title_id.as_str()),
        None,
        vec![("data-blocks-auth-dropdown-panel-panel", "")],
        panel_children,
    );
    let positioner = popover::positioner(
        OpenState::Open,
        vec![("data-blocks-auth-dropdown-panel-positioner", "")],
        vec![content],
    );

    popover::root(
        OpenState::Open,
        vec![("data-blocks-auth-dropdown-panel-popover", "")],
        vec![trigger, positioner],
    )
}

/// caption（並記された各レイアウトの見出し）。
fn caption(label: &'static str) -> Node {
    fandhe_frontend_core::p(
        vec![("class", "blocks-auth-dropdown-panel-caption")],
        vec![text(label)],
    )
}

/// 1 レイアウト分のナビバー本体を組み立てる。`narrow` は Demo 枠を
/// `max-inline-size: 22rem`（`< 24rem` の狭幅判定を確実に満たす幅）に
/// 固定するフラグ（[`super::super::navbar::navbar_with_search`] と同型）。
fn bar(variant: &'static str, narrow: bool) -> Node {
    let actions = div(
        vec![("data-blocks-auth-dropdown-panel-actions", "")],
        vec![help_menu(variant), sign_in_panel(variant)],
    );

    let mut frame_attrs = vec![("data-blocks-auth-dropdown-panel-shell", "")];
    if narrow {
        frame_attrs.push(("data-blocks-auth-dropdown-panel-frame", "narrow"));
    }

    div(
        frame_attrs,
        vec![header(
            vec![
                ("data-blocks-auth-dropdown-panel-root", ""),
                ("data-blocks-auth-dropdown-panel-variant", variant),
            ],
            vec![logo(), actions],
        )],
    )
}

/// `auth-dropdown-panel` の Demo 本体。広幅・狭幅の 2 レイアウトを縦に
/// 並記する純関数。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-auth-dropdown-panel-stack")],
        vec![
            caption("広幅（トリガー右揃えでパネルを絶対配置）"),
            bar("wide", false),
            caption("狭幅（< 24rem、パネルをトリガー直下へ全幅表示）"),
            bar("narrow", true),
        ],
    )
}
```

## 原案差分メモ

- 主参照 R0685 のみ（集約元 1 件）で、差分メモに記載すべき他集約元は
  ありません。
- `_/blocks-intake/` は本リポジトリの worktree に含まれないため対応表 ID
  のみを記載しており、実物ファイルへの参照は行っていません。
- 実機ブラウザでの `24rem` 境界・ライト/ダーク両テーマの目視確認は、
  サンドボックス制約のため未実施です。

関連情報: [Popover](../themes/popover.md) / [Menu](../themes/menu.md) /
[Field](../themes/field.md) / [Input](../themes/input.md) /
[Button](../themes/button.md) / [Link](../themes/link.md)
