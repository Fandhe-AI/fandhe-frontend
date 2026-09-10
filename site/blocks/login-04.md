# login-04

`fandhe-frontend-pre-styled-ui` の `card` / `field` / `input` / `button` /
`image` / `icon` 部品を合成した、shadcn/ui Blocks の `login-04`（フォーム +
画像の 2 カラムログインページ）に相当する合成例です。Blocks セクションは
新規部品を追加するものではなく、既存の Themes/Primitives 部品を組み合わせた
実例集であることに注意してください。

本 Demo は静的な表示例であり、`<form>` 要素を持たず、入力値の送信・検証・
認証処理を一切行いません（ボタンは `type="button"` のまま、「パスワードを
忘れた」「サインアップ」「利用規約」「プライバシーポリシー」はいずれも
ページ遷移しないリンク風ボタン、3 個のプロバイダボタンも送信先を持たない
静的なボタンです）。右列の画像はビルド時に生成されるプレースホルダー SVG
（`assets/image-demo.svg`）であり、実際の写真ではありません。実際の
ログインフォームを実装する場合は、送信処理・バリデーションを利用者自身の
Rust コードで書いてください（`docs/policy/intentional-non-adoption.md`
§3.25 の責務境界: UI コンポーネント層はアプリケーションロジックを内包
しません）。

## Rust コード

```rust
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, icon_button, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::field::{self, FieldOrientation, FieldRootProps};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::image::{image, ImageFit, ImageProps};
use fandhe_frontend_pre_styled_ui::input::{self, FieldIds, FieldProps, InputProps};

/// 右列プレースホルダー画像のパス（`crate::showcase::image_demo_svg` が
/// ビルド時に生成する `assets/image-demo.svg` への block ページからの
/// 相対参照。`showcase::IMAGE_DEMO_SRC` は `pub(crate)` のため参照できず、
/// Markdown 原稿へ転記するコード例としても自己完結させる必要があるため
/// ここでローカルに宣言する）。
const IMAGE_SRC: &str = "../../assets/image-demo.svg";

/// 自作の単純な矩形アイコン（実ブランドロゴを複製しない、モジュール doc
/// 「shadcn 側との構成上の判断」参照。`super::sidebar_03::geo_icon` と
/// 同型だが `pub(super)` で共有されていないためローカルに定義する）。
fn geo_icon(path_d: &'static str) -> Node {
    icon(
        &IconProps::default(),
        vec![],
        vec![el("path", vec![("d", path_d)], vec![])],
    )
}

/// `login-04` の Demo 本体。呼び出しごとに同一の `Node` を返す純関数。
pub fn demo() -> Node {
    let email_field = FieldProps {
        id: "blocks-login-04-email",
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: true,
        readonly: false,
        has_helper_text: false,
    };
    let password_field = FieldProps {
        id: "blocks-login-04-password",
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
    let link_button = ButtonProps {
        variant: ButtonVariant::Link,
        ..ButtonProps::default()
    };
    let outline_button = ButtonProps {
        variant: ButtonVariant::Outline,
        ..ButtonProps::default()
    };

    let form = div(
        vec![("data-blocks-login-04-form", "")],
        vec![field::group(
            vec![],
            vec![
                div(
                    vec![("class", "blocks-login-04-intro")],
                    vec![
                        card::title(vec![], vec![text("Welcome back")]),
                        card::description(vec![], vec![text("Login to your Acme Inc account")]),
                    ],
                ),
                field::root(
                    &orientation,
                    &email_field,
                    vec![("data-blocks-login-04-field", "")],
                    vec![
                        field::label(&email_field, vec![], vec![text("Email")]),
                        input::input(
                            &InputProps::default(),
                            &email_field,
                            vec![("type", "email"), ("placeholder", "m@example.com")],
                        ),
                    ],
                ),
                field::root(
                    &orientation,
                    &password_field,
                    vec![("data-blocks-login-04-field", "")],
                    vec![
                        div(
                            vec![("class", "blocks-login-04-password-row")],
                            vec![
                                field::label(&password_field, vec![], vec![text("Password")]),
                                button::button(
                                    &link_button,
                                    vec![],
                                    vec![text("Forgot your password?")],
                                ),
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
                    vec![("data-blocks-login-04-submit", "")],
                    vec![text("Login")],
                ),
                field::separator(
                    vec![("data-blocks-login-04-separator", "")],
                    vec![text("Or continue with")],
                ),
                div(
                    vec![("data-blocks-login-04-providers", "")],
                    vec![
                        icon_button(
                            &outline_button,
                            "Login with provider A",
                            vec![("data-blocks-login-04-provider", "")],
                            vec![geo_icon("M4 4h16v16H4z")],
                        ),
                        icon_button(
                            &outline_button,
                            "Login with provider B",
                            vec![("data-blocks-login-04-provider", "")],
                            vec![geo_icon("M12 3l9 18H3z")],
                        ),
                        icon_button(
                            &outline_button,
                            "Login with provider C",
                            vec![("data-blocks-login-04-provider", "")],
                            vec![geo_icon("M12 2a10 10 0 100 20 10 10 0 000-20z")],
                        ),
                    ],
                ),
                div(
                    vec![("class", "blocks-login-04-signup-row")],
                    vec![
                        text("Don't have an account? "),
                        button::button(&link_button, vec![], vec![text("Sign up")]),
                    ],
                ),
            ],
        )],
    );

    let image_column = div(
        vec![("data-blocks-login-04-image", "")],
        vec![image(
            &ImageProps {
                fit: ImageFit::Cover,
                ..ImageProps::new(IMAGE_SRC, "")
            },
            vec![("data-blocks-login-04-img", "")],
        )],
    );

    div(
        vec![("data-blocks-login-04-stack", "")],
        vec![
            card::root(
                CardProps::default(),
                vec![("data-blocks-login-04-card", "")],
                vec![card::body(
                    vec![("data-blocks-login-04-body", "")],
                    vec![form, image_column],
                )],
            ),
            div(
                vec![("class", "blocks-login-04-terms")],
                vec![
                    text("By clicking continue, you agree to our "),
                    button::button(&link_button, vec![], vec![text("Terms of Service")]),
                    text(" and "),
                    button::button(&link_button, vec![], vec![text("Privacy Policy")]),
                    text("."),
                ],
            ),
        ],
    )
}
```

## shadcn 側との差分メモ

shadcn/ui `login-04`（`apps/v4/registry/new-york-v4/blocks/login-04/`）との
突合により、以下の判断で構成しています（#2093 で確定）。

- **`<form>` を使わない**: 本サイトは無 JS 前提で Enter キーの暗黙 submit を
  避けるため（`crate::layout` モジュール doc）、`<form>` 要素は出力せず
  `div` で構造化しています。送信ボタンは `type="button"` のまま、
  「パスワードを忘れた」「サインアップ」「利用規約」「プライバシー
  ポリシー」は死リンク（`href="#"`）ではなく見た目だけリンク風の
  `ButtonVariant::Link` ボタンとして実装しています。
- **見出し「Welcome back」は `card::title`（`h3`）を使う**: `card::title` +
  `card::description` を用いています。`card::title` は `h3` を生成します
  が、`data-scope` 属性を持つ部品 anatomy の内部要素であるため
  `crate::layout::with_heading_anchors`（docs サイトの右目次・折りたたみ
  目次の見出し収集）が `data-scope` を持つ部分木を丸ごと走査対象外とし、
  この `h3` は目次へ混入しません（`login-01` と同一判断）。
- **プロバイダロゴ（Apple/Google/Meta）→ 自作の単純幾何図形**: shadcn 側は
  各社ロゴ入り Outline `IconButton` を持ちますが、実企業名・商標ロゴは
  持ち込まない方針（`docs/design/docs-site-blocks-section.md` §8）のため、
  `sidebar-03` と同型の自作幾何アイコン + 一般的な `aria-label`
  （「Login with provider A/B/C」）へ置換しています。
- **「Or continue with」区切りは `field::separator` を採用**: shadcn
  `FieldSeparator`（#2276 で追加）の直接の対応物であり、`field::group`
  内の縦積みリズムを崩さないため採用しました。
- **右列画像は `data:` URI ではなくビルド時生成アセットへの相対パス**:
  `src="data:"` は `fandhe_frontend_core::is_safe_url` の検証で属性ごと
  欠落するため（イシュー #1562 と同型の判断）、`assets/image-demo.svg`
  （`showcase::image_demo_svg` がビルド時に生成する共通プレースホルダー）を
  相対パス参照しています。
- **文言「Acme Inc」**: `docs/design/docs-site-blocks-section.md` §8 の
  「架空のプレースホルダー」規則に従い、shadcn 側の文言をそのまま踏襲して
  います（慣用的な架空社名であり実在企業を指しません）。
- **レスポンシブ**: `< 768px` で右列画像を非表示にし 1 カラムへ切り替える
  shadcn 側の `hidden md:block` 相当を `@media (max-width: 47.99rem)` で
  再現しています。
- **フォント・色はテーマトークン準拠**: レイアウト（2 カラム構成・カード幅
  上限・角丸クリップ・ボタン配置）は shadcn 側スクリーンショットと
  一致させ、配色・タイポグラフィは本リポジトリの `Theme` トークンをそのまま
  用いています。

関連情報: [Card](../themes/card.md) / [Field](../themes/field.md) /
[Input](../themes/input.md) / [Button](../themes/button.md) /
[Image](../themes/image.md) / [Icon](../themes/icon.md)
