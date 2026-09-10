//! `login-04` block（イシュー #2093。shadcn/ui Blocks の `login-04`
//! 〔フォーム + 画像の 2 カラムログインページ〕に相当する合成例。
//! `login_01`（1 カラム）に続く 2 件目のログイン系 block）。
//!
//! # 使用部品
//!
//! `card`（構造・2 カラム grid 化）/ `field`（`group`/`root`/`separator`）+
//! `input`（メール・パスワード入力）/ `button`（Solid 送信・リンク風
//! アクション・Outline アイコンボタン 3 個）/ `image`（右カラムの
//! プレースホルダー画像）/ `icon`（プロバイダボタンの自作幾何アイコン）の
//! 6 部品を合成する（[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//!
//! # `<form>` を使わない・認証処理を持たない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力せず、送信ボタンは `button::button` の既定 `type="button"` のまま
//! 用いる。「パスワードを忘れた」「サインアップ」「利用規約」「プライバシー
//! ポリシー」の各リンクは遷移先を持たないため `link::root` の `href="#"`
//! ではなく、見た目だけをリンク風にする `button::ButtonVariant::Link`
//! （`<button type="button">` のまま）を使う（`login_01` と同一判断）。
//! 値は一切送信されず、認証処理も行わない静的な合成例である。
//!
//! # shadcn 側との構成上の判断（#2093 で確定）
//!
//! - **見出しは `card::title`（`<h3>`）+ `card::description` で表現する**:
//!   `crate::layout` の TOC 収集（`with_heading_anchors`）は Card の
//!   `title`（`<h3>`）を含む部品内部の見出しを部分木ごと走査対象外とする
//!   （`crate::layout` モジュール doc「一部（例: … Card の title `h3`）」
//!   節参照）ため、`card::title` は右目次・折りたたみ目次を汚染しない。
//!   `login_01` も `card::header` の `title`/`description` のみで構成して
//!   おり同じ判断（`h1`/`h2` は使わない）。
//! - **右列画像は `showcase::image_demo_svg` が生成するビルド時アセット**
//!   （`../../assets/image-demo.svg`）を相対パス参照する。`data:` URI は
//!   `fandhe_frontend_core::is_safe_url` が拒否し属性ごと欠落する
//!   （イシュー #1562 と同型の判断）ため使わない。
//! - **プロバイダボタン 3 個は実ブランドロゴを複製しない**: shadcn 側は
//!   Apple/Google/Meta のロゴ入り Outline `IconButton` を持つが、実企業名・
//!   商標ロゴは持ち込まない方針（`docs/design/docs-site-blocks-section.md`
//!   §8）のため、`sidebar_03::geo_icon` と同型の自作の単純幾何図形へ
//!   置換した `icon_button`（`ButtonVariant::Outline`）3 個とする。
//!   `aria-label` はそれぞれ「Login with provider A/B/C」という一般的な
//!   ラベルを付与する。
//! - **「Or continue with」区切りは `field::separator` を採用**:
//!   shadcn `FieldSeparator`（#2276 で headless-ui/pre-styled-ui へ追加
//!   済み）の直接の対応物であり、`field::group` 内の縦積みリズムを崩さない。
//! - **見出し文言「Acme Inc」**: `docs/design/docs-site-blocks-section.md`
//!   §8 の「架空のプレースホルダー」規則に従い、shadcn 側の文言をそのまま
//!   踏襲する（慣用的な架空社名であり実在企業を指さない）。
//! - **リンク類はすべて `ButtonVariant::Link` の `button::button`**:
//!   `<form>` を使わない・死リンクを出さない方針（`login_01` と同一判断）。
//! - **補足文（サインアップ行・規約行）は素の `div`**: `field::helper_text`
//!   は `has_helper_text: true` と aria 配線を要求し用途が異なるため使わず、
//!   `login_01` の `blocks-login-01-signup-row` と同型の `div` にする。
use super::{Block, Part};

// blocks-code:begin
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
// blocks-code:end

/// [`super::BLOCKS`] へ登録するレジストリエントリ。
pub const BLOCK: Block = Block {
    path: "/blocks/login-04/",
    title: "login-04",
    rust_source: "crates/docs-site/src/blocks/login_04.rs",
    demo_class: "blocks-login-04",
    parts: &[
        Part {
            label: "Card",
            path: "/themes/card/",
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
            label: "Image",
            path: "/themes/image/",
        },
        Part {
            label: "Icon",
            path: "/themes/icon/",
        },
    ],
    demo,
};

/// `login_04` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS` doc
/// 「block 固有 CSS の置き場」節。他 block と同型で `pub(super)` として
/// `super::stylesheet` から連結される）。
///
/// # `@media` の初使用
///
/// 既存 block（`login_01`/`dashboard_01`/`sidebar_07`/`sidebar_03`）は
/// `@media` を持たないが、`StyleSheet::push_css` の検証は `<`・NUL のみを
/// 拒否するため通る。`< 768px` で右列画像を隠し 1 カラムへ切り替える
/// shadcn 側の `hidden md:block` 相当をブレークポイントとして再現する
/// （目視確認は実装計画 §4 手順 11 参照）。
///
/// # `[data-blocks-login-04-body]` の詳細度を Card recipe 以上にする
///
/// `card::body` が出力する要素は `[data-scope="card"][data-part="body"]`
/// （詳細度 (0,2,0)）を併せ持つため、block 側セレクタを単独の
/// `[data-blocks-login-04-body]`（詳細度 (0,1,0)）のままにすると Card
/// recipe の既定 `display: flex`/`padding` に負けて `display: grid` や
/// `grid-template-columns` が適用されず 2 カラム表示が成立しない
/// （PR #2292 codex-review 指摘、イシュー #2093）。このためセレクタを
/// `[data-scope="card"][data-part="body"][data-blocks-login-04-body]`
/// （詳細度 (0,3,0)）へ結合し、Card recipe を確実に上書きする。
/// `@media (max-width: 47.99rem)` 側の再定義も同じ詳細度に揃える
/// （揃えないと縮小時のみ Card recipe が再び優先されてしまうため）。
///
/// # `[data-blocks-login-04-img]` の詳細度を Image recipe 以上にする
///
/// `image::image` が出力する要素は `[data-scope="image"][data-part="root"]`
/// （詳細度 (0,2,0)）で `height: auto` を持つため、block 側セレクタを単独の
/// `[data-blocks-login-04-img]`（詳細度 (0,1,0)）のままにすると Image
/// recipe の `height: auto` に負けて `height: 100%` が適用されず、2 カラム
/// 表示時に画像が右列全体を覆わず下部に余白が残る（PR #2292 codex-review /
/// Cursor Bugbot 指摘、イシュー #2093）。Card body と同様にセレクタを
/// `[data-scope="image"][data-part="root"][data-blocks-login-04-img]`
/// （詳細度 (0,3,0)）へ結合し、Image recipe を確実に上書きする。
pub(super) const LAYOUT_CSS: &str = "\
.blocks-login-04 {\n  display: flex;\n  justify-content: center;\n  align-items: center;\n  min-height: 28rem;\n}\n\
[data-blocks-login-04-stack] {\n  display: flex;\n  flex-direction: column;\n  align-items: center;\n  gap: 1.5rem;\n  width: 100%;\n}\n\
[data-blocks-login-04-card] {\n  width: 100%;\n  max-width: 56rem;\n  overflow: hidden;\n}\n\
[data-scope=\"card\"][data-part=\"body\"][data-blocks-login-04-body] {\n  padding: 0;\n  display: grid;\n  grid-template-columns: 1fr 1fr;\n}\n\
[data-blocks-login-04-form] {\n  padding: 2rem;\n  display: flex;\n  flex-direction: column;\n}\n\
[data-blocks-login-04-field] {\n  display: flex;\n  flex-direction: column;\n  gap: 0.5rem;\n}\n\
.blocks-login-04-intro {\n  text-align: center;\n  margin: 0 0 0.5rem;\n}\n\
.blocks-login-04-password-row {\n  display: flex;\n  align-items: baseline;\n  justify-content: space-between;\n  gap: 0.5rem;\n}\n\
[data-blocks-login-04-submit] {\n  width: 100%;\n}\n\
[data-blocks-login-04-providers] {\n  display: grid;\n  grid-template-columns: repeat(3, 1fr);\n  gap: 1rem;\n}\n\
.blocks-login-04-signup-row {\n  font-size: 0.875rem;\n  text-align: center;\n  color: var(--fandhe-color-fg-muted);\n}\n\
[data-blocks-login-04-image] {\n  position: relative;\n  background: var(--fandhe-color-bg-subtle);\n  min-height: 100%;\n}\n\
[data-scope=\"image\"][data-part=\"root\"][data-blocks-login-04-img] {\n  position: absolute;\n  inset: 0;\n  width: 100%;\n  height: 100%;\n}\n\
.blocks-login-04-terms {\n  font-size: 0.75rem;\n  text-align: center;\n  color: var(--fandhe-color-fg-muted);\n  max-width: 56rem;\n}\n\
@media (max-width: 47.99rem) {\n  [data-scope=\"card\"][data-part=\"body\"][data-blocks-login-04-body] {\n    grid-template-columns: 1fr;\n  }\n  [data-blocks-login-04-image] {\n    display: none;\n  }\n}\n";
