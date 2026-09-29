# auth-split-accent-panel

`fandhe-frontend-pre-styled-ui` の `field` / `input` / `button` / `link` /
`checkbox` / `separator` / `blockquote` / `avatar` / `icon` / `card` の
10 部品を合成した、アクセント面パネル付き分割サインインの実例です。Blocks
セクションは新規部品を追加するものではなく、既存の Themes/Primitives 部品
を組み合わせた実例集であることに注意してください（主参照は対応表 ID
R0689、集約元は R0698・R0699。出典の固有名・ファイル名は記載しません）。

左右 2 カラムの一方にサインインフォーム、もう一方に画像を持たないアクセント
色の面パネルを置く構成です。パネル内容が異なる 2 通り（顧客の声、アイコン
付き利点一覧 + カード入りフォーム）を並べて示し、後者ではパネルを左側へ
入れ替えて両方の配置パターンを実演します。狭い画面ではどちらのインスタンス
もパネルを隠し、フォームのみを 1 列表示にします。

本 Demo は静的な表示例であり、`<form>` 要素は一切持たず、データの取得・
送信・認証処理を行いません。ボタンは `type="button"` のまま送信先を持たず、
ログイン状態保持・利用規約同意の各チェックはネイティブ `disabled` により
常に未チェック表示のまま操作できません。インスタンス間の切替導線（サイン
アップ例へ／サインイン例へ）はページ内アンカーで実在する遷移先を指し、
死リンク（`href="#"`）ではありません。文言はすべて独自に書いた架空のもの
であり、実企業名・実クレデンシャル・PII を含みません。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::blockquote::{self, BlockquoteVariant};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::checkbox::{self, CheckboxProps};
use fandhe_frontend_pre_styled_ui::field::{self, FieldOrientation, FieldRootProps};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::input::{self, FieldIds, FieldProps, InputProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::separator::{self, SeparatorProps};
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// 架空の利点一覧文言（`dummy_assets` に該当ヘルパが無いためローカル
/// 定数。実在サービス・実企業名を含まない）。
const BENEFITS: [(&str, &str); 3] = [
    ("M5 12l4 4 10-10", "チームの作業をひとつの画面に集約"),
    (
        "M12 3l7 4v5c0 4-3 7-7 8-4-1-7-4-7-8V7z",
        "権限管理でデータを安全に保護",
    ),
    ("M4 12h12m0 0l-4-4m4 4l-4 4", "既存ツールとすぐに連携開始"),
];

/// 単純な幾何アイコン（実ブランドロゴを複製しない、モジュール doc
/// 「利点一覧のアイコン」節参照。`login_04::geo_icon` と同型だが
/// `pub(super)` で共有されていないためローカルに定義する）。
///
/// `icon` の svg root が固定する `fill="currentColor"`（塗りつぶし）は
/// チェック・矢印のような開いた path を面として潰してしまうため、
/// path 側で `fill="none"` + `stroke="currentColor"` に上書きし線画として
/// 描画する（塗りではなくストロークのアイコンにする）。
fn geo_icon(path_d: &'static str) -> Node {
    icon(
        &IconProps::default(),
        vec![("data-blocks-auth-split-accent-panel-benefit-icon", "")],
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

/// フォーム欄 1 個ぶんの `FieldProps` を組み立てる。
fn field_props(id: &'static str, required: bool) -> FieldProps<'static> {
    FieldProps {
        id,
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required,
        readonly: false,
        has_helper_text: false,
    }
}

/// 縦積み（label 上・control 下）の共通 orientation。
fn orientation() -> FieldRootProps {
    FieldRootProps {
        orientation: FieldOrientation::Vertical,
    }
}

/// 氏名から avatar フォールバック用のイニシャルを組み立てる
/// （`content_article::byline` と同型のロジック）。
fn initials(name: &str) -> String {
    name.split_whitespace()
        .filter_map(|part| part.chars().next())
        .collect()
}

/// インスタンス A（サインイン + 顧客の声パネル、主参照 R0689）。
fn signin_instance() -> Node {
    let email_field = field_props("blocks-auth-split-accent-panel-signin-email", true);
    let password_field = field_props("blocks-auth-split-accent-panel-signin-password", true);

    let remember_props = CheckboxProps {
        disabled: true,
        ..CheckboxProps::default()
    };

    let form = div(
        vec![("data-blocks-auth-split-accent-panel-form", "")],
        vec![field::group(
            vec![],
            vec![
                div(
                    vec![("class", "blocks-auth-split-accent-panel-intro")],
                    vec![
                        card::title(vec![], vec![text("サインイン")]),
                        card::description(vec![], vec![text("アカウント情報を入力してください。")]),
                    ],
                ),
                field::root(
                    &orientation(),
                    &email_field,
                    vec![("data-blocks-auth-split-accent-panel-field", "")],
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
                    &orientation(),
                    &password_field,
                    vec![("data-blocks-auth-split-accent-panel-field", "")],
                    vec![
                        field::label(&password_field, vec![], vec![text("パスワード")]),
                        input::input(
                            &InputProps::default(),
                            &password_field,
                            vec![("type", "password")],
                        ),
                    ],
                ),
                checkbox::root(
                    Size::Md,
                    ColorPalette::Accent,
                    &remember_props,
                    vec![("data-blocks-auth-split-accent-panel-remember", "")],
                    vec![
                        checkbox::hidden_input(
                            &remember_props,
                            "auth-split-accent-panel-signin-remember",
                            "on",
                            vec![],
                        ),
                        checkbox::control(
                            &remember_props,
                            vec![],
                            vec![checkbox::indicator(&remember_props, vec![], vec![])],
                        ),
                        checkbox::label(
                            &remember_props,
                            vec![],
                            vec![text("ログイン状態を保持する")],
                        ),
                    ],
                ),
                button::button(
                    &ButtonProps::default(),
                    vec![("data-blocks-auth-split-accent-panel-submit", "")],
                    vec![text("サインイン")],
                ),
                separator::separator(
                    &SeparatorProps::default(),
                    vec![("data-blocks-auth-split-accent-panel-switch-rule", "")],
                ),
                div(
                    vec![("class", "blocks-auth-split-accent-panel-switch")],
                    vec![
                        text("アカウントをお持ちでない方は "),
                        link::root(
                            "#blocks-auth-split-accent-panel-signup",
                            &LinkProps::default(),
                            vec![],
                            vec![text("サインアップ例へ")],
                        ),
                    ],
                ),
            ],
        )],
    );

    let panel = div(
        vec![("data-blocks-auth-split-accent-panel-panel", "")],
        vec![blockquote::root(
            BlockquoteVariant::default(),
            ColorPalette::default(),
            vec![("data-blocks-auth-split-accent-panel-quote", "")],
            vec![
                blockquote::content(vec![], vec![text(dummy_assets::TESTIMONIAL_QUOTES[0])]),
                blockquote::caption(
                    vec![("data-blocks-auth-split-accent-panel-caption", "")],
                    vec![
                        avatar::root(
                            &AvatarProps::default(),
                            vec![("data-blocks-auth-split-accent-panel-avatar", "")],
                            vec![avatar::fallback(
                                ImageStatus::Error,
                                vec![],
                                vec![text(initials(dummy_assets::PERSON_NAMES[0]))],
                            )],
                        ),
                        div(
                            vec![("class", "blocks-auth-split-accent-panel-byline")],
                            vec![
                                div(vec![], vec![text(dummy_assets::PERSON_NAMES[0])]),
                                div(
                                    vec![],
                                    vec![text(format!(
                                        "{} / {}",
                                        dummy_assets::JOB_TITLES[0],
                                        dummy_assets::COMPANY_NAMES[0]
                                    ))],
                                ),
                            ],
                        ),
                    ],
                ),
            ],
        )],
    );

    div(
        vec![
            ("data-blocks-auth-split-accent-panel-instance", "signin"),
            ("id", "blocks-auth-split-accent-panel-signin"),
        ],
        vec![form, panel],
    )
}

/// インスタンス B（カード入りサインアップ + 利点一覧パネル、R0698/R0699
/// を統合）。
fn signup_instance() -> Node {
    let name_field = field_props("blocks-auth-split-accent-panel-signup-name", true);
    let email_field = field_props("blocks-auth-split-accent-panel-signup-email", true);
    let password_field = field_props("blocks-auth-split-accent-panel-signup-password", true);

    let terms_props = CheckboxProps {
        disabled: true,
        ..CheckboxProps::default()
    };

    let form_card = card::root(
        CardProps::default(),
        vec![("data-blocks-auth-split-accent-panel-card", "")],
        vec![
            card::header(
                vec![],
                vec![
                    card::title(vec![], vec![text("アカウントを作成")]),
                    card::description(vec![], vec![text("数分でセットアップが完了します。")]),
                ],
            ),
            card::body(
                vec![],
                vec![field::group(
                    vec![],
                    vec![
                        field::root(
                            &orientation(),
                            &name_field,
                            vec![("data-blocks-auth-split-accent-panel-field", "")],
                            vec![
                                field::label(&name_field, vec![], vec![text("氏名")]),
                                input::input(
                                    &InputProps::default(),
                                    &name_field,
                                    vec![("type", "text"), ("placeholder", "山田 太郎")],
                                ),
                            ],
                        ),
                        field::root(
                            &orientation(),
                            &email_field,
                            vec![("data-blocks-auth-split-accent-panel-field", "")],
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
                            &orientation(),
                            &password_field,
                            vec![("data-blocks-auth-split-accent-panel-field", "")],
                            vec![
                                field::label(&password_field, vec![], vec![text("パスワード")]),
                                input::input(
                                    &InputProps::default(),
                                    &password_field,
                                    vec![("type", "password")],
                                ),
                            ],
                        ),
                        checkbox::root(
                            Size::Md,
                            ColorPalette::Accent,
                            &terms_props,
                            vec![("data-blocks-auth-split-accent-panel-terms", "")],
                            vec![
                                checkbox::hidden_input(
                                    &terms_props,
                                    "auth-split-accent-panel-signup-terms",
                                    "on",
                                    vec![],
                                ),
                                checkbox::control(
                                    &terms_props,
                                    vec![],
                                    vec![checkbox::indicator(&terms_props, vec![], vec![])],
                                ),
                                checkbox::label(
                                    &terms_props,
                                    vec![],
                                    vec![text("利用規約に同意します")],
                                ),
                            ],
                        ),
                        button::button(
                            &ButtonProps::default(),
                            vec![("data-blocks-auth-split-accent-panel-submit", "")],
                            vec![text("アカウントを作成")],
                        ),
                    ],
                )],
            ),
            card::footer(
                vec![],
                vec![div(
                    vec![("class", "blocks-auth-split-accent-panel-switch")],
                    vec![
                        text("すでにアカウントをお持ちの方は "),
                        link::root(
                            "#blocks-auth-split-accent-panel-signin",
                            &LinkProps::default(),
                            vec![],
                            vec![text("サインイン例へ")],
                        ),
                    ],
                )],
            ),
        ],
    );

    let benefits = el(
        "ul",
        vec![("class", "blocks-auth-split-accent-panel-benefits")],
        BENEFITS
            .iter()
            .map(|(path_d, label)| {
                el(
                    "li",
                    vec![("class", "blocks-auth-split-accent-panel-benefit")],
                    vec![geo_icon(path_d), text(*label)],
                )
            })
            .collect(),
    );

    let panel = div(
        vec![("data-blocks-auth-split-accent-panel-panel", "")],
        vec![div(vec![], vec![text("選ばれる理由")]), benefits],
    );

    div(
        vec![
            ("data-blocks-auth-split-accent-panel-instance", "signup"),
            ("id", "blocks-auth-split-accent-panel-signup"),
        ],
        vec![
            div(
                vec![("data-blocks-auth-split-accent-panel-form", "")],
                vec![form_card],
            ),
            panel,
        ],
    )
}

/// `auth-split-accent-panel` の Demo 本体。呼び出しごとに同一の `Node`
/// を返す純関数。
pub fn demo() -> Node {
    div(
        vec![("data-blocks-auth-split-accent-panel-stack", "")],
        vec![signin_instance(), signup_instance()],
    )
}
```

## 原案差分メモ

- 主参照 R0689 をインスタンス A（サインイン + 顧客の声パネル）とし、
  R0698（サインアップ版の入力項目差分）と R0699（利点一覧 + カード入り
  フォーム）をインスタンス B へ統合しています。
- 「アカウントをお持ちでない方は」「すでにアカウントをお持ちの方は」の
  切替導線は、遷移先を持たないリンク風ボタンではなくページ内アンカー
  （実在する `id`）にし、死リンクを避けています。
- ログイン状態保持・利用規約同意のチェックはいずれもネイティブ
  `disabled` による静的な未チェック表示です。
- 利点一覧のアイコンは実在ブランドのロゴ・商標を模さない自作の幾何図形
  です。
- ブラウザ実機確認は未実施、`cargo test` で代替しています。
