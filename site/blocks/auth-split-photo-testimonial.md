# auth-split-photo-testimonial

`fandhe-frontend-pre-styled-ui` の `field` / `input` / `button` / `link` /
`checkbox` / `separator` / `image` / `blockquote` / `avatar` の 9 部品を
合成した、片側にサインイン/サインアップフォーム・もう片側に背景写真 +
暗幕 + 顧客の声（引用・氏名・肩書）を置く分割サインインの実例です。Blocks
セクションは新規部品を追加するものではなく、既存の Themes 部品を組み合わ
せた実例集であることに注意してください（レイアウト仕様は Issue #2966 の
記述から構成しており、出典の固有名・ファイル名は記載しません）。

1 つの Demo に「サインイン形」（左フォーム・右写真）と「サインアップ形」
（左写真・右フォーム）の 2 通りを縦に並べています。写真パネルは幅
48rem 未満では非表示になり、フォームのみの 1 カラム表示に切り替わります。
写真パネルは非インタラクティブな表示専用要素のため、形ごとに DOM 順を
入れ替えても Tab 順は常に視覚上の左→右の順と一致します。

本 Demo は静的な表示例であり、`<form>` 要素は一切持たず、データの取得・
送信・状態管理・認証処理を行いません。ボタンはすべて `type="button"` の
まま送信先を持たず、チェックボックスは未チェック固定の静的表示です。
ソーシャルログインボタンはアイコンなしのテキストボタンとし、実ブランド
名・ロゴは持ち込みません。文言・氏名・肩書はすべて独自に書いた架空の
ものであり、実企業名・実クレデンシャル・PII を含みません。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::blockquote::{self, BlockquoteVariant};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::checkbox::{self, CheckboxProps};
use fandhe_frontend_pre_styled_ui::field::{self, FieldOrientation, FieldRootProps};
use fandhe_frontend_pre_styled_ui::image::{self, ImageFit, ImageProps};
use fandhe_frontend_pre_styled_ui::input::{self, FieldIds, FieldProps, InputProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps, LinkVariant};
use fandhe_frontend_pre_styled_ui::separator::{self, SeparatorProps};
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// 補助リンク（パスワード再設定・アカウント切り替え導線）の固定外部 URL
/// （モジュール doc「参照元と原案からの差分」節参照。
/// `contact_split_form_info::REPO` と同じ判断で、実在の外部サービスへは
/// 接続せず本リポジトリ自身を指す）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// [`form_column`]/[`agree_checkbox`] が形（サインイン/サインアップ）を
/// 区別するための block ローカル列挙型（モジュール doc「id/name の
/// 一意性」節）。id/name の接頭辞を `format!`/`Box::leak` に頼らず、形ごとの
/// 全フィールド分の id リテラルをこの型の各アームへ直接書き下す。
#[derive(Clone, Copy, PartialEq, Eq)]
enum AuthVariant {
    SignIn,
    SignUp,
}

impl AuthVariant {
    /// CSS フック用の文字列値（[`variant_layout`] の
    /// `data-blocks-auth-split-photo-testimonial-variant` へ渡す）。
    fn attr(self) -> &'static str {
        match self {
            AuthVariant::SignIn => "sign-in",
            AuthVariant::SignUp => "sign-up",
        }
    }
}

/// ソーシャルログインボタン（アイコンなしのテキストボタン、モジュール doc
/// 「使用部品」節参照）。
fn provider_button(label: &'static str) -> Node {
    button::button(
        &ButtonProps {
            variant: ButtonVariant::Outline,
            ..ButtonProps::default()
        },
        vec![("data-blocks-auth-split-photo-testimonial-provider", "")],
        vec![text(label)],
    )
}

/// 「または」区切り線（[`fandhe_frontend_pre_styled_ui::separator`] の
/// `group`/`label`、Issue #2966 の使用部品指定 `separator` に従う）。
fn or_separator() -> Node {
    separator::group(
        vec![("data-blocks-auth-split-photo-testimonial-separator", "")],
        vec![
            separator::separator(&SeparatorProps::default(), vec![]),
            separator::label(vec![], vec![text("または")]),
            separator::separator(&SeparatorProps::default(), vec![]),
        ],
    )
}

/// 入力欄 1 個ぶん（`field::root` + `field::label` + `input::input`）。
fn text_field(
    id: &'static str,
    label_text: &'static str,
    input_type: &'static str,
    placeholder: Option<&'static str>,
) -> Node {
    let field_props = FieldProps {
        id,
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: true,
        readonly: false,
        has_helper_text: false,
    };
    let mut input_attrs: Vec<(&str, &str)> = vec![("type", input_type)];
    if let Some(placeholder) = placeholder {
        input_attrs.push(("placeholder", placeholder));
    }
    field::root(
        &FieldRootProps {
            orientation: FieldOrientation::Vertical,
        },
        &field_props,
        vec![("data-blocks-auth-split-photo-testimonial-field", "")],
        vec![
            field::label(&field_props, vec![], vec![text(label_text)]),
            input::input(&InputProps::default(), &field_props, input_attrs),
        ],
    )
}

/// 同意/記憶チェックボックス（未チェック固定の静的表示、モジュール doc
/// 「`<form>` を持たない」節参照）。`name`/`label_text` は呼び出し側が
/// 形（サインイン/サインアップ）ごとに一意な値を渡す契約であり、本関数
/// 自体は形を区別しない。
///
/// `disabled: true`（PR #3418 レビュー指摘対応）: ネイティブ `hidden_input`
/// はクリック・キーボードで操作可能な一方、視覚上の `indicator` は
/// レンダリング時の `CheckedState::Unchecked` に固定されたまま更新されない
/// （docs-site は無 JS 制約〔`crate` モジュール doc 参照〕で hydration を
/// 行わないため）。「未チェック固定の静的表示」という意図を `disabled` で
/// 実際に操作不能化し、見た目と状態の食い違いを構造的に防ぐ。
fn agree_checkbox(name: &'static str, label_text: &'static str) -> Node {
    let props = CheckboxProps {
        disabled: true,
        ..CheckboxProps::default()
    };
    checkbox::root(
        Size::Sm,
        ColorPalette::Accent,
        &props,
        vec![],
        vec![
            checkbox::hidden_input(&props, name, "on", vec![]),
            checkbox::control(
                &props,
                vec![],
                vec![checkbox::indicator(&props, vec![], vec![])],
            ),
            checkbox::label(&props, vec![], vec![text(label_text)]),
        ],
    )
}

/// 補助リンク（`link::root`、常に外部の固定 URL、モジュール doc
/// 「参照元と原案からの差分」節参照）。
fn helper_link(label_text: &'static str) -> Node {
    link::root(
        REPO,
        &LinkProps {
            external: true,
            variant: LinkVariant::Underline,
            palette: ColorPalette::Neutral,
            ..LinkProps::default()
        },
        vec![],
        vec![text(label_text)],
    )
}

/// フォーム列（見出し + 説明 → ソーシャルログイン → 区切り線 → 入力欄 →
/// チェックボックス → 送信 → 補助リンクの縦積み）。
fn form_column(variant: AuthVariant) -> Node {
    let (title, description, submit_label) = match variant {
        AuthVariant::SignIn => (
            "おかえりなさい",
            "アカウントにサインインして続行してください。",
            "サインイン",
        ),
        AuthVariant::SignUp => (
            "アカウントを作成",
            "必要事項を入力してアカウントを作成してください。",
            "アカウントを作成",
        ),
    };

    let mut fields = Vec::new();
    if matches!(variant, AuthVariant::SignUp) {
        fields.push(text_field(
            "blocks-auth-split-photo-testimonial-signup-name",
            "氏名",
            "text",
            None,
        ));
    }
    fields.push(text_field(
        match variant {
            AuthVariant::SignIn => "blocks-auth-split-photo-testimonial-signin-email",
            AuthVariant::SignUp => "blocks-auth-split-photo-testimonial-signup-email",
        },
        "メールアドレス",
        "email",
        Some("m@example.com"),
    ));
    fields.push(text_field(
        match variant {
            AuthVariant::SignIn => "blocks-auth-split-photo-testimonial-signin-password",
            AuthVariant::SignUp => "blocks-auth-split-photo-testimonial-signup-password",
        },
        "パスワード",
        "password",
        None,
    ));

    let (checkbox_name, checkbox_label, helper_label) = match variant {
        AuthVariant::SignIn => (
            "blocks-auth-split-photo-testimonial-signin-remember",
            "ログイン状態を保持する",
            "パスワードをお忘れですか",
        ),
        AuthVariant::SignUp => (
            "blocks-auth-split-photo-testimonial-signup-agree",
            "利用規約に同意する",
            "すでにアカウントをお持ちの方はこちら",
        ),
    };

    div(
        vec![("data-blocks-auth-split-photo-testimonial-form", "")],
        vec![
            div(
                vec![("class", "blocks-auth-split-photo-testimonial-intro")],
                vec![
                    el(
                        "h2",
                        vec![("class", "blocks-auth-split-photo-testimonial-title")],
                        vec![text(title)],
                    ),
                    div(
                        vec![("class", "blocks-auth-split-photo-testimonial-description")],
                        vec![text(description)],
                    ),
                ],
            ),
            div(
                vec![("class", "blocks-auth-split-photo-testimonial-providers")],
                vec![
                    provider_button("プロバイダ A で続行"),
                    provider_button("プロバイダ B で続行"),
                ],
            ),
            or_separator(),
            div(
                vec![("class", "blocks-auth-split-photo-testimonial-fields")],
                fields,
            ),
            agree_checkbox(checkbox_name, checkbox_label),
            button::button(
                &ButtonProps::default(),
                vec![("data-blocks-auth-split-photo-testimonial-submit", "")],
                vec![text(submit_label)],
            ),
            div(
                vec![("class", "blocks-auth-split-photo-testimonial-helper")],
                vec![helper_link(helper_label)],
            ),
        ],
    )
}

/// 写真パネル（背景写真 + 暗幕 + 顧客の声、`quote_index`/`person_index` で
/// 形ごとに別人物の推薦文を出す）。
fn photo_panel(quote_index: usize, person_index: usize) -> Node {
    div(
        vec![("data-blocks-auth-split-photo-testimonial-panel", "")],
        vec![
            image::image(
                &ImageProps {
                    fit: ImageFit::Cover,
                    ..ImageProps::new(dummy_assets::BACKGROUND_SRC, "")
                },
                vec![("data-blocks-auth-split-photo-testimonial-photo", "")],
            ),
            div(
                vec![("class", "blocks-auth-split-photo-testimonial-scrim")],
                vec![],
            ),
            blockquote::root(
                BlockquoteVariant::default(),
                ColorPalette::default(),
                vec![("data-blocks-auth-split-photo-testimonial-quote", "")],
                vec![
                    blockquote::content(
                        vec![],
                        vec![text(dummy_assets::TESTIMONIAL_QUOTES[quote_index])],
                    ),
                    blockquote::caption(
                        vec![("class", "blocks-auth-split-photo-testimonial-meta")],
                        vec![
                            avatar::root(
                                &AvatarProps::default(),
                                vec![("data-blocks-auth-split-photo-testimonial-avatar", "")],
                                vec![avatar::image(
                                    ImageStatus::Loaded,
                                    dummy_assets::AVATAR_SRC,
                                    "",
                                    vec![],
                                )],
                            ),
                            div(
                                vec![("class", "blocks-auth-split-photo-testimonial-byline")],
                                vec![
                                    div(
                                        vec![],
                                        vec![text(dummy_assets::PERSON_NAMES[person_index])],
                                    ),
                                    div(
                                        vec![],
                                        vec![text(
                                            dummy_assets::JOB_TITLES
                                                [person_index % dummy_assets::JOB_TITLES.len()],
                                        )],
                                    ),
                                ],
                            ),
                        ],
                    ),
                ],
            ),
        ],
    )
}

/// 1 つの形のレイアウト骨格（フォーム列・写真パネルを DOM 順どおりに渡す、
/// モジュール doc「1 つの Demo に 2 つの形を縦に並べる」節参照）。
fn variant_layout(variant: AuthVariant, label: &'static str, first: Node, second: Node) -> Node {
    div(
        vec![("class", "blocks-auth-split-photo-testimonial-layout")],
        vec![
            div(
                vec![("class", "blocks-auth-split-photo-testimonial-label")],
                vec![text(label)],
            ),
            div(
                vec![(
                    "data-blocks-auth-split-photo-testimonial-variant",
                    variant.attr(),
                )],
                vec![first, second],
            ),
        ],
    )
}

/// `auth-split-photo-testimonial` の Demo 本体（サインイン形・サインアップ形
/// を縦に並記する）。呼び出しごとに同一の `Node` を返す純関数。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-auth-split-photo-testimonial-stack")],
        vec![
            variant_layout(
                AuthVariant::SignIn,
                "サインイン（左フォーム・右写真）",
                form_column(AuthVariant::SignIn),
                photo_panel(0, 0),
            ),
            variant_layout(
                AuthVariant::SignUp,
                "サインアップ（左写真・右フォーム）",
                photo_panel(1, 1),
                form_column(AuthVariant::SignUp),
            ),
        ],
    )
}
```

## 原案差分メモ

- レイアウト仕様は Issue #2966 の記述（片側フォーム・片側写真+暗幕+顧客の
  声、9 部品構成）から構成しており、特定の参照ファイル・内部識別子は転記
  しません。
- ソーシャルログインは `icon` 部品を使用部品に含めないため、アイコンなしの
  テキストボタンにしています。実ブランドロゴの複製は行いません。
- 暗幕の配色は `--fandhe-color-fg`/`--fandhe-color-bg` の反転ペア（既存
  block `blog-overlay-cards` と同型の判断）です。
- 補助リンク（パスワード再設定・アカウント切り替え導線）はすべて固定外部
  URL（本リポジトリ自身）へ遷移させ、`href="#"` の死リンクは使いません。
- チェックボックスは未チェック固定・`<form>` 非出力・送信処理なしの静的
  表示です。
