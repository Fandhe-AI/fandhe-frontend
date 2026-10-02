//! `auth-split-photo-testimonial` block（イシュー #2966。親トラッキング
//! #2730「Blocks 目的別パーツ拡充ツリー」配下、区分は application、
//! カテゴリは Auth）。片側にサインイン/サインアップフォーム、もう片側に
//! 背景写真 + 暗幕 + 顧客の声（引用・氏名・肩書）を置く 2 カラム block。
//!
//! # 使用部品
//!
//! `field` / `input` / `button` / `link` / `checkbox` / `separator` /
//! `image` / `blockquote` / `avatar` / `heading` の 10 部品を合成する
//! （[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//! `text`/`icon` は使用部品に含まれないため、説明文・ソーシャルログイン
//! ボタンは素の `div`（レイアウト用 CSS クラスのみ）とアイコンなしの
//! テキストボタンで表現する（実ブランド名・ロゴを持ち込まない方針とも
//! 整合する）。フォーム見出し（「おかえりなさい」/「アカウントを作成」）は
//! pre-styled-ui `heading::heading`（`HeadingLevel::H3`）で意味づける（PR
//! #3418 レビュー指摘対応・Bugbot: 素の `el("h2", ...)` は
//! `data-scope` 祖先を持たないため `crate::layout::with_heading_anchors`
//! の `h2`/`h3` 収集から除外されず、Demo 見出しが右目次（`.docs-toc`）へ
//! 混入していた。`heading` は `data-scope="heading"` を持つため同関数の
//! 「`data-scope` を持つ要素の部分木は収集除外」規則に構造的に乗る。
//! `contact_form_testimonial` 等の既存 block と同じ手法）。
//!
//! # 1 つの Demo に 2 つの形（サインイン/サインアップ）を縦に並べる
//!
//! `contact_split_form_info` と同じ手法で、[`demo`] はルート
//! `div.blocks-auth-split-photo-testimonial-stack`（[`Block::demo_class`]
//! とは意図的に別名、既存 block と同じ Bugbot 教訓の回避）の中に
//! 「形ラベル + 形の本体」の組を 2 つ縦に並べる。
//!
//! - **形 A（サインイン）**: `data-blocks-auth-split-photo-testimonial-
//!   variant="sign-in"`。DOM 順は「フォーム → 写真パネル」（`>= 48rem` で
//!   左フォーム・右写真）。
//! - **形 B（サインアップ）**: `variant="sign-up"`。DOM 順は「写真パネル →
//!   フォーム」（`>= 48rem` で左写真・右フォーム）。写真パネルは
//!   非インタラクティブな表示専用要素（フォーカス可能な子を持たない）
//!   なため、DOM 順を形ごとに入れ替えても Tab 順は視覚順とそのまま一致する
//!   （`order`/`grid-column` による明示的な列の入れ替えは行わない）。
//!
//! # 参照元と原案からの差分
//!
//! Issue #2966 のレイアウト仕様（写真+暗幕+推薦文の片側パネル、区切り線を
//! 挟んだソーシャルログイン + 入力欄の縦積みフォーム）から構成した。
//! 具体的な参照ファイル・取得手段・内部識別子は記載しない
//! （`docs/design/motion-reference-adoption-policy.md` §9 と同じライセンス
//! 上の転記制限）。原案からの意図的な差分:
//!
//! - ソーシャルログインはアイコンなしのテキストボタン（`icon` 部品を
//!   使用部品に含めないため、実ブランドロゴも複製しない）。
//! - 暗幕の配色は `--fandhe-color-fg`/`--fandhe-color-bg` の反転ペア
//!   （[`blog_overlay_cards`](super::super::blog::blog_overlay_cards) と
//!   同型）。
//! - 補助リンク: パスワード再設定（サインイン形「パスワードをお忘れですか」）
//!   は遷移先を持たないため `link::root` の `href="#"` ではなく
//!   `ButtonVariant::Link` の `<button type="button">` を使う（`auth_dropdown_panel`・
//!   `login_04` と同じ判断。PR #3418 レビュー指摘対応:
//!   固定外部 URL〔本リポジトリ自身〕へ遷移する以前の実装は、文言
//!   「パスワードをお忘れですか」と無関係のリポジトリへ遷移する動作が
//!   食い違っていた）。アカウント切り替え導線（サインアップ形「すでに
//!   アカウントをお持ちの方はこちら」）は Demo 内にサインイン形が実在
//!   するため、サインイン形コンテナの `id`（[`AuthVariant::id`]）への
//!   同一ページ内アンカー（`href="#..."`、`external: false`）にする（PR
//!   #3418 レビュー指摘対応。文言と遷移先の不一致——遷移すると謳いながら
//!   実際には別ページへ飛ぶ——を防ぐ）。
//! - チェックボックスは未チェック固定の静的表示（状態遷移は扱わない）。
//! - `<form>` は出力せず、送信ボタンは `button::button` の既定
//!   `type="button"` のまま用いる。
//!
//! # `text` の名前衝突
//!
//! `fandhe_frontend_core::text`（テキストノード生成関数）のみを使い、
//! `fandhe_frontend_pre_styled_ui::text` モジュール（styled Text 部品）は
//! 使用部品に含まれないため import しない（名前衝突は発生しない）。
//!
//! # `drop_class_attr` と CSS フックの選び方
//!
//! `field::root`/`input::input`/`button::button`/`link::root`/
//! `checkbox::root`/`separator::separator`/`image::image`/
//! `blockquote::root`/`avatar::root` はいずれも `drop_class_attr`（または
//! 同型の固定属性マージ）により呼び出し側 `attrs` の `class` を黙って除去
//! する契約を持つため、Demo 固有のスタイルフックは
//! `data-blocks-auth-split-photo-testimonial-*` 属性で渡し、[`LAYOUT_CSS`]
//! 側も同じ属性セレクタで対応する。素の `div`・`blockquote::content`/
//! `caption`・`separator::group`/`label` には `class` がそのまま効くため、
//! それらは `.blocks-auth-split-photo-testimonial-*` クラスセレクタを使う。
//!
//! # 詳細度の罠（recipe への勝ち方）
//!
//! 部品 recipe（詳細度 (0,2,0)）に確実に勝つため、上書きは
//! `[data-scope="…"][data-part="…"][data-blocks-auth-split-photo-
//! testimonial-*]` の 3 セレクタ構成（詳細度 (0,3,0)）で行う
//! （`login_04`/`contact_form_testimonial` と同型の判断）。
//!
//! # `<form>` を持たない・データ取得/送信を行わない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節・`docs/policy/intentional-non-adoption.md` §3.25 に従い、
//! 本 Demo はフォーム・状態機械・認証処理を持たない静的な合成例である。
//! ボタンはすべて `button::button` の既定 `type="button"` のまま用い、
//! チェックボックスは `CheckedState::Unchecked` 固定である。文言・氏名・
//! 肩書はすべて架空のもの（実企業名・実クレデンシャル・PII を含まない）。
//!
//! # `id`/`name` の一意性
//!
//! [`fandhe_frontend_pre_styled_ui::input::FieldProps::id`] から
//! `-control`/`-label` の id が決まるため、形ごとに接頭辞を変える
//! （`blocks-auth-split-photo-testimonial-signin-*`/`-signup-*`）。
//! checkbox の `hidden_input` の `name` も形ごとに変える。両形とも
//! `has_helper_text: false` に固定し、存在しない id を指す
//! `aria-describedby` を出さない。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::blockquote::{self, BlockquoteVariant};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::checkbox::{self, CheckboxProps};
use fandhe_frontend_pre_styled_ui::field::{self, FieldOrientation, FieldRootProps};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::image::{self, ImageFit, ImageProps};
use fandhe_frontend_pre_styled_ui::input::{self, FieldIds, FieldProps, InputProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps, LinkVariant};
use fandhe_frontend_pre_styled_ui::separator::{self, SeparatorProps};
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

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

    /// [`variant_layout`] が variant コンテナへ付与する `id`（フラグメント
    /// アンカーの遷移先、モジュール doc「補助リンク」節参照）。
    fn id(self) -> &'static str {
        match self {
            AuthVariant::SignIn => "blocks-auth-split-photo-testimonial-sign-in",
            AuthVariant::SignUp => "blocks-auth-split-photo-testimonial-sign-up",
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
///
/// `disabled` の既定 CSS（opacity: 0.5 / cursor: not-allowed）は
/// `[data-blocks-auth-split-photo-testimonial-agree][data-disabled]` を
/// [`LAYOUT_CSS`] で中和する（`auth_split_accent_panel::agree_checkbox`
/// と同じ判断・同型のセレクタ、PR #3418 レビュー指摘対応）。「未チェック
/// 固定の静的表示」という意図の伝達に薄い見た目は不要なため。
fn agree_checkbox(name: &'static str, label_text: &'static str) -> Node {
    let props = CheckboxProps {
        disabled: true,
        ..CheckboxProps::default()
    };
    checkbox::root(
        Size::Sm,
        ColorPalette::Accent,
        &props,
        vec![("data-blocks-auth-split-photo-testimonial-agree", "")],
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

/// アカウント切り替え導線（サインアップ形限定、`link::root`。モジュール doc
/// 「補助リンク」節参照）。サインイン形の「パスワードをお忘れですか」は
/// 遷移先を持たないため本関数を使わず [`form_column`] 内で直接
/// `ButtonVariant::Link` の button を組む。
fn helper_link(label_text: &'static str, href: &str, external: bool) -> Node {
    link::root(
        href,
        &LinkProps {
            external,
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

    let (checkbox_name, checkbox_label) = match variant {
        AuthVariant::SignIn => (
            "blocks-auth-split-photo-testimonial-signin-remember",
            "ログイン状態を保持する",
        ),
        AuthVariant::SignUp => (
            "blocks-auth-split-photo-testimonial-signup-agree",
            "利用規約に同意する",
        ),
    };

    // helper: サインイン形は遷移先を持たないパスワード再設定のため
    // `ButtonVariant::Link` の `<button type="button">`（`auth_dropdown_panel`
    // と同じ判断）。サインアップ形は Demo 内に実在するサインイン形への
    // 同一ページ内アンカー（モジュール doc「補助リンク」節参照）。
    let helper = match variant {
        AuthVariant::SignIn => button::button(
            &ButtonProps {
                variant: ButtonVariant::Link,
                ..ButtonProps::default()
            },
            vec![],
            vec![text("パスワードをお忘れですか")],
        ),
        AuthVariant::SignUp => helper_link(
            "すでにアカウントをお持ちの方はこちら",
            &format!("#{}", AuthVariant::SignIn.id()),
            false,
        ),
    };

    div(
        vec![("data-blocks-auth-split-photo-testimonial-form", "")],
        vec![
            div(
                vec![("class", "blocks-auth-split-photo-testimonial-intro")],
                vec![
                    heading(
                        HeadingLevel::H3,
                        &HeadingProps {
                            size: HeadingSize::Xl,
                            weight: HeadingWeight::Bold,
                        },
                        vec![("data-blocks-auth-split-photo-testimonial-title", "")],
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
                vec![helper],
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
                vec![
                    (
                        "data-blocks-auth-split-photo-testimonial-variant",
                        variant.attr(),
                    ),
                    ("id", variant.id()),
                ],
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
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/auth-split-photo-testimonial/",
    title: "auth-split-photo-testimonial",
    category: BlockCategory::Auth,
    rust_source: "crates/docs-site/src/blocks/application/auth/auth_split_photo_testimonial.rs",
    demo_class: "blocks-auth-split-photo-testimonial",
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
            label: "Image",
            path: "/themes/image/",
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
            label: "Heading",
            path: "/themes/heading/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `auth_split_photo_testimonial` 固有のレイアウト規則
/// （`crate::blocks` モジュール doc「CSS の置き場」節）。
///
/// セレクタは `.blocks-auth-split-photo-testimonial-*` と
/// `[data-blocks-auth-split-photo-testimonial-*]` のみを用い、他 block や
/// 部品の素のセレクタへ影響させない。写真パネルは既定で非表示
/// （`display: none`）にし、`>= 48rem`（
/// [`fandhe_frontend_pre_styled_ui::recipe::Breakpoint::Md`] と一致する
/// リテラル値、CSS custom property は `@media` 条件式内で解決できないため
/// 直書きする、`login_04`/`contact_split_form_info` と同じ判断）で 2 カラム
/// grid へ切り替えて表示する。
///
/// フォーム見出しは pre-styled-ui `heading::heading` を使うため、
/// `.docs-content h2` のサイト typography（`border-top`・上余白等）が
/// 漏れ込む問題は起きない（`heading` recipe が `data-scope="heading"` の
/// 属性セレクタ経由でスタイルを持ち、素の `h2` 要素セレクタには一致しない
/// ため、詳細度勝負を要しない。モジュール doc「使用部品」節参照）。
const LAYOUT_CSS: &str = "\
.blocks-auth-split-photo-testimonial-stack {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-10);\n}\n\
.blocks-auth-split-photo-testimonial-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-auth-split-photo-testimonial-label {\n  font-size: var(--fandhe-font-font-size-sm);\n  color: var(--fandhe-color-fg-muted);\n}\n\
[data-blocks-auth-split-photo-testimonial-variant] {\n  display: grid;\n  grid-template-columns: minmax(0, 1fr);\n  border: 1px solid var(--fandhe-color-border);\n  border-radius: var(--fandhe-radius-lg);\n  overflow: hidden;\n  min-height: 28rem;\n}\n\
[data-blocks-auth-split-photo-testimonial-form] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n  padding: var(--fandhe-space-8);\n}\n\
.blocks-auth-split-photo-testimonial-intro {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-auth-split-photo-testimonial-description {\n  color: var(--fandhe-color-fg-muted);\n}\n\
.blocks-auth-split-photo-testimonial-providers {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n}\n\
[data-scope=\"button\"][data-part=\"root\"][data-blocks-auth-split-photo-testimonial-provider] {\n  width: 100%;\n}\n\
.blocks-auth-split-photo-testimonial-fields {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
[data-scope=\"button\"][data-part=\"root\"][data-blocks-auth-split-photo-testimonial-submit] {\n  width: 100%;\n}\n\
.blocks-auth-split-photo-testimonial-helper {\n  font-size: var(--fandhe-font-font-size-sm);\n  text-align: center;\n}\n\
[data-scope=\"checkbox\"][data-part=\"root\"][data-blocks-auth-split-photo-testimonial-agree][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
[data-blocks-auth-split-photo-testimonial-panel] {\n  display: none;\n  position: relative;\n}\n\
[data-scope=\"image\"][data-part=\"root\"][data-blocks-auth-split-photo-testimonial-photo] {\n  position: absolute;\n  inset: 0;\n  width: 100%;\n  height: 100%;\n}\n\
.blocks-auth-split-photo-testimonial-scrim {\n  position: absolute;\n  inset: 0;\n  background: var(--fandhe-color-fg);\n  opacity: 0.75;\n}\n\
[data-scope=\"blockquote\"][data-part=\"root\"][data-blocks-auth-split-photo-testimonial-quote] {\n  position: relative;\n  color: var(--fandhe-color-bg);\n  padding: var(--fandhe-space-8);\n  height: 100%;\n  display: flex;\n  flex-direction: column;\n  justify-content: flex-end;\n  gap: var(--fandhe-space-4);\n  --fandhe-blockquote-caption-fg: var(--fandhe-color-bg);\n}\n\
[data-scope=\"blockquote\"][data-part=\"caption\"].blocks-auth-split-photo-testimonial-meta {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-auth-split-photo-testimonial-byline {\n  display: flex;\n  flex-direction: column;\n  font-size: var(--fandhe-font-font-size-sm);\n}\n\
@media (min-width: 48rem) {\n  \
[data-blocks-auth-split-photo-testimonial-variant] {\n    grid-template-columns: repeat(2, minmax(0, 1fr));\n  }\n\
[data-blocks-auth-split-photo-testimonial-panel] {\n    display: block;\n  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::{demo, dummy_assets, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// Demo が期待する 10 部品を出力すること、非対話制約（`<form>` 不在・
    /// `data:` URI 不在・`href="#"` 不在・チェック済みマーク不在）を満たす
    /// ことの単体回帰。フォーム見出しが `heading` 部品（`data-scope`
    /// を持つ）で意味づけられ、素の `<h2` を出力しないこと（PR #3418
    /// レビュー指摘対応・TOC 混入の回帰固定）も合わせて固定する。
    #[test]
    fn demo_composes_expected_parts_and_avoids_forms() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"field\"",
            "data-scope=\"button\"",
            "data-scope=\"link\"",
            "data-scope=\"checkbox\"",
            "data-scope=\"separator\"",
            "data-scope=\"image\"",
            "data-scope=\"blockquote\"",
            "data-scope=\"avatar\"",
            "data-scope=\"heading\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert!(html.contains("data-part=\"input\""));
        assert_eq!(html.matches("<form").count(), 0);
        assert_eq!(html.matches("type=\"submit\"").count(), 0);
        assert_eq!(html.matches("<h2").count(), 0);
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("src=\"data:"));
        assert!(!html.contains(" checked"));
        assert!(html.contains(dummy_assets::AVATAR_SRC));
        assert!(html.contains(dummy_assets::BACKGROUND_SRC));
    }

    /// サインイン形・サインアップ形の両方が出力されること（[`BLOCK`] の
    /// モジュール doc「1 つの Demo に 2 つの形を縦に並べる」節）。
    #[test]
    fn demo_renders_both_variants() {
        let html = render(&demo());
        assert_eq!(
            html.matches("data-blocks-auth-split-photo-testimonial-variant=\"sign-in\"")
                .count(),
            1
        );
        assert_eq!(
            html.matches("data-blocks-auth-split-photo-testimonial-variant=\"sign-up\"")
                .count(),
            1
        );
    }

    /// サインアップ形の「すでにアカウントをお持ちの方はこちら」がサインイン
    /// 形コンテナの実在 `id` を指す同一ページ内アンカーであること（PR #3418
    /// レビュー指摘対応。文言と遷移先の不一致・死リンクの回避を固定する。
    /// `auth_split_accent_panel::switch_links_point_at_existing_instance_ids`
    /// と同型）。
    #[test]
    fn signup_helper_link_points_at_existing_signin_id() {
        let html = render(&demo());
        assert!(html.contains(r##"href="#blocks-auth-split-photo-testimonial-sign-in""##));
        assert!(html.contains(r#"id="blocks-auth-split-photo-testimonial-sign-in""#));
        // 同一ページ内アンカーは `external` を立てない（リンク先が外部
        // ドメインでないことの固定）。
        assert_eq!(
            html.matches(r##"href="#blocks-auth-split-photo-testimonial-sign-in""##)
                .count(),
            1
        );
    }

    /// id の重複がないことを固定する（アクセシビリティ上の不変条件）。
    #[test]
    fn demo_has_no_duplicate_ids() {
        let html = render(&demo());
        let mut ids = Vec::new();
        let mut rest = html.as_str();
        while let Some(idx) = rest.find("id=\"") {
            let after = &rest[idx + 4..];
            let end = after.find('"').expect("id attribute should be closed");
            ids.push(&after[..end]);
            rest = &after[end + 1..];
        }
        let mut sorted = ids.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(
            sorted.len(),
            ids.len(),
            "demo output should not contain duplicate id attributes: {ids:?}"
        );
    }

    /// [`LAYOUT_CSS`] が想定するブレークポイント・写真パネルの既定非表示・
    /// `<` を含まないことを固定する（REQ-1: `</style>` によるスタイル脱出を
    /// 防ぐ）。
    #[test]
    fn layout_css_declares_breakpoint_and_hides_panel_by_default() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("@media (min-width: 48rem)"));
        assert!(LAYOUT_CSS.contains(
            "[data-blocks-auth-split-photo-testimonial-panel] {\n  display: none;\n  position: relative;\n}"
        ));
        assert!(LAYOUT_CSS.contains("repeat(2, minmax(0, 1fr))"));
    }

    /// `agree_checkbox` の `disabled` 既定 CSS（opacity: 0.5 相当）を
    /// [`LAYOUT_CSS`] が中和すること（PR #3418 レビュー指摘対応。
    /// `data-blocks-auth-split-photo-testimonial-agree` 属性が実際に出力
    /// されることも合わせて固定する）。
    #[test]
    fn layout_css_neutralizes_disabled_checkbox_opacity() {
        assert!(LAYOUT_CSS.contains(
            "[data-scope=\"checkbox\"][data-part=\"root\"][data-blocks-auth-split-photo-testimonial-agree][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}"
        ));
        let html = render(&demo());
        assert!(html.contains("data-blocks-auth-split-photo-testimonial-agree"));
    }

    /// ルート class（`demo_class` とは別名）が `demo()` の出力へ実際に現れる
    /// こと（既存 block と同じ Bugbot 教訓の固定）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-auth-split-photo-testimonial-stack\""));
        assert_ne!(
            super::BLOCK.demo_class,
            "blocks-auth-split-photo-testimonial-stack"
        );
    }
}
