//! `auth-split-accent-panel` block（イシュー #2965。`application/auth` の
//! 5 件目。左右 2 カラムの一方にサインインフォーム、もう一方に画像を
//! 持たないアクセント色の面パネルを置く合成例。主参照は対応表 ID R0689、
//! 集約元は R0698（サインアップ版の入力項目差分）・R0699（利点一覧 +
//! カード入りフォーム）。取得手段・ファイル名・内部コンポーネント識別子は
//! 記載しない（`docs/design/motion-reference-adoption-policy.md` §9 と
//! 同じライセンス上の転記制限）。
//!
//! # 使用部品
//!
//! `field` / `input` / `button` / `link` / `checkbox` / `separator` /
//! `blockquote` / `avatar` / `icon` / `card` の 10 部品を合成する
//! （[`BLOCK`] の `parts` に一致させる契約、`crates/docs-site/tests/
//! blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//!
//! # 2 インスタンスの並記（両配置パターンを 1 ページで示す）
//!
//! Issue の要件「一方にフォーム・もう一方にアクセント面パネル（画像なし）」
//! を、パネル内容 2 通り（顧客の声／アイコン付き利点一覧）を並記して示す
//! （`content_with_testimonial`/`page_heading_avatar` 等、複数バリエーション
//! を 1 Demo に並べる既存 block と同型の構成判断）。
//!
//! - **A（`signin`）**: 左サインインフォーム・右アクセント面パネル
//!   （顧客の声、`blockquote` + `avatar`）。R0689 を主参照とする。
//! - **B（`signup`）**: 左アクセント面パネル・右カード入りサインアップ
//!   フォーム（利点一覧、`icon` 付き `ul`/`li`）。R0698 の入力項目差分と
//!   R0699 の「利点一覧 + カード入りフォーム」構成を統合する。パネルと
//!   フォームの左右入れ替えは CSS Grid の `order`（[`LAYOUT_CSS`]）で行い、
//!   DOM 順は両インスタンスとも「フォーム → パネル」のまま揃える
//!   （読み上げ順序をパネル位置で変えない）。
//!
//! 狭い画面（`@media (max-width: 47.99rem)`）ではどちらのインスタンスも
//! パネルを隠しフォームのみを 1 列表示にする（Issue 要件どおり）。
//!
//! # `<form>` を使わない・認証処理を持たない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力せず、送信ボタンは `button::button` の既定 `type="button"` のまま
//! 用いる。値は一切送信されず、認証処理も行わない静的な合成例である。
//! 文言はすべて架空のもの（実在の人物・企業・PII を含まない）。
//!
//! # 2 つの切替リンクはページ内アンカーにする（死リンク回避）
//!
//! 「アカウントをお持ちでない方は」「すでにアカウントをお持ちの方は」の
//! 切替導線は、遷移先を持たない `ButtonVariant::Link`（`login_04` の判断）
//! ではなく `link::root` の実在するページ内アンカー（`#blocks-
//! auth-split-accent-panel-signup`/`-signin`、対応する `id` を各インスタンス
//! ルートへ付与）で実装する。`content_article_toc.rs` のインライン目次
//! アンカーと同型で、`crate::linkcheck::check_links` の「`#fragment` のみ
//! → 同一ページの id 集合と突合」検証（イシュー #470）を満たす実在遷移先
//! であり、死リンク（`href="#"`）ではない。
//!
//! # チェックボックスを静的な未チェック表示にする理由
//!
//! 両インスタンスの同意/ログイン状態保持チェックは
//! `contact_centered_form::consent_checkbox` と同型に `CheckboxProps {
//! disabled: true, .. }` でネイティブ操作を不能にし、SSR 初期状態（未
//! チェック）が常にラベル・見た目と一致することを構造的に保証する
//! （同モジュール doc「同意チェックをネイティブ disabled にする理由」節
//! 参照）。`disabled_declarations()` の既定 `opacity: 0.5` は
//! [`LAYOUT_CSS`] で中和する。
//!
//! # 区切りはテキストなしの罫線のみ
//!
//! サインインボタンの直後には代替のサインイン手段（SSO 等）を持たない
//! ため、「または」等の代替操作を示唆するテキスト区切りは置かない
//! （レビュー指摘: テキストなしの区切りが暗示する代替操作が存在しない
//! 問題）。フォーム末尾の切替行の直前に置く罫線のみの
//! `separator::separator`（テキストなし）だけで区切りを満たす。
//!
//! # アクセント面パネルの色反転
//!
//! パネル背景は `var(--fandhe-color-accent)`、前景は
//! `var(--fandhe-color-accent-fg)` にし（`cta_centered.rs` の
//! `tone="accent"` と同型）、パネル内の `blockquote`/`icon`
//! recipe が持つ既定の色指定は [`LAYOUT_CSS`] のセレクタ結合（`[data-
//! scope=...][data-part=...][data-blocks-auth-split-accent-panel-*]`、
//! 詳細度を recipe 以上へ上げる）で `color: inherit` へ上書きする
//! （`login_04` の「recipe への勝ち方」と同型の判断）。`avatar` のみは
//! `color: inherit` だけでは背景が recipe 既定の `neutral-muted` の
//! ままコントラスト不足になる（レビュー指摘）ため、背景・前景を対で
//! `accent-fg` 地に `accent` 文字色へ反転させる（`accent`/`accent-fg`
//! はこのペア自体がコントラストを持つよう設計されたトークンのため、
//! 反転させても可読性が保たれる）。
//!
//! # signup 側フォーム列の padding 二重取り回避（レビュー指摘）
//!
//! `[data-blocks-auth-split-accent-panel-form]` は `space-8` の外側
//! padding を持つが、`card::root` の各スロット（header/body/footer）も
//! recipe 既定の padding を持つため、signin（`div` 直下にフォームを
//! 置くだけ）と異なり signup（`card::root` を内包）では二重に内側へ
//! 寄っていた（レビュー指摘）。`card` 側には `border`/`box-shadow` の
//! 除去のみを残し、signup インスタンス限定で外側フォーム列の padding を
//! [`LAYOUT_CSS`] の `[data-blocks-auth-split-accent-panel-instance=
//! "signup"] [data-blocks-auth-split-accent-panel-form]` で `0` に
//! 上書きし、カードのスロット padding のみを内側余白として残す。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `field::root` / `input::input` / `button::button` / `link::root` /
//! `checkbox::root` / `separator::separator` / `blockquote::root` /
//! `avatar::root` / `icon` / `card::root` はいずれも `drop_class_attr`
//! により呼び出し側 `attrs` の `class` を黙って除去する契約を持つため、
//! Demo 固有のスタイルフックは `data-blocks-auth-split-accent-panel-*`
//! 属性で渡す。素の `div`/`ul`/`li` には `class` がそのまま効くため、
//! それらは `.blocks-auth-split-accent-panel-*` クラスセレクタを使う。
//!
//! # 利点一覧のアイコン（実ブランドロゴを複製しない）
//!
//! `login_04::geo_icon` と同型の自作の単純幾何図形（チェック・盾・矢印を
//! 模した抽象パス）を用い、実在ブランドのロゴ・商標は持ち込まない。
//!
//! # `text` の名前衝突
//!
//! 本ファイルは `fandhe_frontend_pre_styled_ui::text` を使わないため
//! `styled_text` 別名は不要（`fandhe_frontend_core::text` のみを使う）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
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
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/auth-split-accent-panel/",
    title: "auth-split-accent-panel",
    category: BlockCategory::Auth,
    rust_source: "crates/docs-site/src/blocks/application/auth/auth_split_accent_panel.rs",
    demo_class: "blocks-auth-split-accent-panel",
    parts: &[
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
            label: "Link",
            path: "/themes/link/",
        },
        Part {
            label: "Checkbox",
            path: "/themes/checkbox/",
        },
        Part {
            label: "Separator",
            path: "/themes/separator/",
        },
        Part {
            label: "Blockquote",
            path: "/themes/blockquote/",
        },
        Part {
            label: "Avatar",
            path: "/themes/avatar/",
        },
        Part {
            label: "Icon",
            path: "/themes/icon/",
        },
        Part {
            label: "Card",
            path: "/themes/card/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `auth_split_accent_panel` 固有のレイアウト規則（`crate::blocks`
/// モジュール doc「CSS の置き場」節。他 block と同型で本ファイル
/// 内 private 定数として [`BLOCK`] の `layout_css`（[`LayoutCss::Static`]）
/// で自己申告し、`crate::blocks::stylesheet` が `all_blocks()` 走査で
/// `push_css` する）。
///
/// セレクタは `.blocks-auth-split-accent-panel-*` と
/// `[data-blocks-auth-split-accent-panel-*]`、および styled 部品の
/// `[data-scope=...]` 系セレクタへの上書き（モジュール doc「アクセント面
/// パネルの色反転」節参照）のみを用いる。
const LAYOUT_CSS: &str = "\
[data-blocks-auth-split-accent-panel-stack] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n}\n\
[data-blocks-auth-split-accent-panel-instance] {\n  display: grid;\n  grid-template-columns: repeat(2, minmax(0, 1fr));\n  border-radius: var(--fandhe-radius-lg);\n  overflow: hidden;\n  border: 1px solid var(--fandhe-color-border);\n}\n\
[data-blocks-auth-split-accent-panel-instance=\"signup\"] [data-blocks-auth-split-accent-panel-panel] {\n  order: -1;\n}\n\
[data-blocks-auth-split-accent-panel-form] {\n  padding: var(--fandhe-space-8);\n  min-width: 0;\n}\n\
[data-blocks-auth-split-accent-panel-instance=\"signup\"] [data-blocks-auth-split-accent-panel-form] {\n  padding: 0;\n}\n\
[data-scope=\"card\"][data-part=\"root\"][data-blocks-auth-split-accent-panel-card] {\n  border: none;\n  box-shadow: none;\n}\n\
.blocks-auth-split-accent-panel-intro {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1-5);\n  margin: 0 0 var(--fandhe-space-2);\n}\n\
[data-scope=\"checkbox\"][data-part=\"root\"][data-blocks-auth-split-accent-panel-remember][data-disabled],\n[data-scope=\"checkbox\"][data-part=\"root\"][data-blocks-auth-split-accent-panel-terms][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
[data-scope=\"button\"][data-part=\"root\"][data-blocks-auth-split-accent-panel-submit] {\n  width: 100%;\n}\n\
.blocks-auth-split-accent-panel-switch {\n  font-size: var(--fandhe-font-font-size-sm);\n  color: var(--fandhe-color-fg-muted);\n  text-align: center;\n}\n\
[data-blocks-auth-split-accent-panel-panel] {\n  background: var(--fandhe-color-accent);\n  color: var(--fandhe-color-accent-fg);\n  padding: var(--fandhe-space-8);\n  display: flex;\n  flex-direction: column;\n  justify-content: center;\n  gap: var(--fandhe-space-6);\n}\n\
[data-scope=\"blockquote\"][data-part=\"root\"][data-blocks-auth-split-accent-panel-quote] {\n  color: inherit;\n  border-color: var(--fandhe-color-accent-fg);\n}\n\
[data-blocks-auth-split-accent-panel-panel] [data-scope=\"blockquote\"][data-part=\"content\"] {\n  color: inherit;\n}\n\
[data-scope=\"blockquote\"][data-part=\"caption\"][data-blocks-auth-split-accent-panel-caption] {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-3);\n  color: inherit;\n}\n\
[data-scope=\"avatar\"][data-part=\"root\"][data-blocks-auth-split-accent-panel-avatar] {\n  background: var(--fandhe-color-accent-fg);\n  color: var(--fandhe-color-accent);\n}\n\
.blocks-auth-split-accent-panel-byline {\n  display: flex;\n  flex-direction: column;\n  font-size: var(--fandhe-font-font-size-sm);\n}\n\
.blocks-auth-split-accent-panel-benefits {\n  list-style: none;\n  margin: 0;\n  padding: 0;\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-auth-split-accent-panel-benefit {\n  display: flex;\n  align-items: flex-start;\n  gap: var(--fandhe-space-2);\n}\n\
[data-scope=\"icon\"][data-part=\"root\"][data-blocks-auth-split-accent-panel-benefit-icon] {\n  color: inherit;\n  width: 1.25rem;\n  height: 1.25rem;\n  flex-shrink: 0;\n}\n\
@media (max-width: 47.99rem) {\n  [data-blocks-auth-split-accent-panel-instance] {\n    grid-template-columns: 1fr;\n  }\n  [data-blocks-auth-split-accent-panel-panel] {\n    display: none;\n  }\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use crate::blocks::dummy_assets;
    use fandhe_frontend_core::render;

    /// Demo が期待する 10 種の部品を含むことを固定する。
    #[test]
    fn demo_composes_expected_parts() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"field\"",
            "data-scope=\"button\"",
            "data-scope=\"link\"",
            "data-scope=\"checkbox\"",
            "data-scope=\"separator\"",
            "data-scope=\"blockquote\"",
            "data-scope=\"avatar\"",
            "data-scope=\"icon\"",
            "data-scope=\"card\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert!(html.contains(r#"data-part="input""#));
    }

    /// `<form>` を出力しない・死リンク（`href="#"`）を持たないこと。
    #[test]
    fn demo_has_no_form_and_no_dead_link() {
        let html = render(&demo());
        assert!(!html.contains("<form"));
        assert!(!html.contains(r##"href="#""##));
        assert!(!html.contains("action="));
        assert!(!html.contains("src=\"data:"));
    }

    /// 入力欄の `type` 属性が期待どおり出力されていること。
    #[test]
    fn demo_declares_expected_input_types() {
        let html = render(&demo());
        assert!(html.contains(r#"type="email""#));
        assert!(html.contains(r#"type="password""#));
        assert!(html.contains(r#"type="checkbox""#));
    }

    /// アクセント面パネルは「画像なし」の契約どおり、`<img>` を出力せず
    /// イニシャルの avatar フォールバックのみを持つこと（モジュール doc
    /// 「2 インスタンスの並記」節の R0689 契約、[`crate::blocks::dummy_assets`]
    /// の `PERSON_NAMES[0]` から導出したイニシャルを表示する）。
    #[test]
    fn demo_panel_avatar_has_no_image() {
        let html = render(&demo());
        assert!(!html.contains("<img"));
        assert!(html.contains(&super::initials(dummy_assets::PERSON_NAMES[0])));
    }

    /// 2 インスタンス間のページ内アンカー（`href`）と対応する `id` が
    /// 両方存在すること（死リンク回避の実在遷移先であることを固定する）。
    #[test]
    fn switch_links_point_at_existing_instance_ids() {
        let html = render(&demo());
        for (href, id) in [
            (
                r##"href="#blocks-auth-split-accent-panel-signup""##,
                r##"id="blocks-auth-split-accent-panel-signup""##,
            ),
            (
                r##"href="#blocks-auth-split-accent-panel-signin""##,
                r##"id="blocks-auth-split-accent-panel-signin""##,
            ),
        ] {
            assert!(html.contains(href), "expected {href} in {html}");
            assert!(html.contains(id), "expected {id} in {html}");
        }
    }

    /// `id` 属性が重複しないこと（`content_centered_form` 等と同型の軽量
    /// 固定、横断検証は `blocks_contract.rs` の aria 検査が担う）。
    #[test]
    fn demo_output_has_no_duplicate_ids() {
        let html = render(&demo());
        let mut ids: Vec<&str> = Vec::new();
        let mut rest = html.as_str();
        while let Some(pos) = rest.find(r#"id=""#) {
            let after = &rest[pos + 4..];
            if let Some(end) = after.find('"') {
                ids.push(&after[..end]);
                rest = &after[end + 1..];
            } else {
                break;
            }
        }
        let unique: std::collections::BTreeSet<&str> = ids.iter().copied().collect();
        assert_eq!(ids.len(), unique.len(), "duplicate id in {ids:?}");
    }

    /// [`LAYOUT_CSS`] が `<` を含まず、狭幅ブレークポイントでパネルを
    /// 隠す規則を持つこと。
    #[test]
    fn layout_css_hides_panel_on_narrow_viewport() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("@media (max-width: 47.99rem)"));
        assert!(LAYOUT_CSS.contains("display: none;"));
    }
}
