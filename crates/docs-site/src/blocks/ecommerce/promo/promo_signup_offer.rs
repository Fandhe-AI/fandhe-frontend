//! `promo-signup-offer` block（Ecommerce / Promo カテゴリ）。角丸カードを
//! 画像列（片側）と登録フォーム（反対側）の 2 列へ分け、画像付きの登録
//! 特典カードを合成する。
//!
//! # 出典に関する注記
//!
//! 主参照は対応表 ID R0346（基準形）。集約元は R0345（カテゴリ選択の
//! radio group）・R0347（左右反転 + 同意文）・R0348（フォーム列の中央寄せ）
//! の 3 件。取得手段・ファイル名・出典名・内部識別子は記載しない（既存
//! block と同じライセンス上の転記制限、対応表 ID のみを記す）。取り込むのは
//! 領域の配置と部品構成という構造のみで、文言・配色・装飾は独自に書く。
//!
//! # 使用部品
//!
//! `card` / `heading` / `text` / `field` / `input` / `button` / `image` /
//! `radio-group` / `fieldset` / `link` の 10 部品を合成する（[`BLOCK`] の
//! `parts` に一致させる契約、`crates/docs-site/tests/blocks_nav.rs`/
//! `blocks_contract.rs` が検証する）。新しい UI 部品は追加しない。
//!
//! # 4 形を 1 つの Demo に並記する
//!
//! 既存 block（例: `cta_split_image`）と同型に、4 形を [`variant_label`] で
//! 見出しを付けながら [`demo`] 1 つの中へ縦に並べる。
//!
//! | 形 | 対応 ID | 内容 |
//! |----|---------|------|
//! | A 基準形 | R0346 | 画像列（左）+ ロゴ・見出し・説明・メール登録（右） |
//! | B カテゴリ選択 | R0345 | A に fieldset + radio-group（興味のあるカテゴリ）を追加 |
//! | C 左右反転 + 同意文 | R0347 | 画像列を右へ（`48rem` 以上で `order` を入れ替え）。ボタンの下に同意文 |
//! | D 中央寄せ | R0348 | フォーム列の内容を中央寄せ |
//!
//! # `<form>` を持たない・データ取得/送信を行わない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である。登録ボタンは [`button::button`] の既定 `type="button"` のまま
//! 用いる（暗黙 submit は起きない）。興味カテゴリの radio group は先頭 1 件
//! が選択済みの**静的な初期状態**のみを描く（`docs/policy/
//! intentional-non-adoption.md` §3.25、UI コンポーネント層はアプリケーション
//! ロジックを内包しない）。文言はすべて独自に書いた架空のものであり、実
//! 企業名・実クレデンシャル・PII を含まない。
//!
//! # 興味カテゴリ radio group をネイティブ disabled にする理由
//!
//! [`radio_group::item_hidden_input`] は有効なネイティブ
//! `<input type="radio">` であり、`disabled` を渡さない構成では docs サイト
//! が JS ハイドレーションを行わなくてもラベルクリック・キーボード操作で
//! ブラウザが `checked` をネイティブに切り替えてしまう。一方
//! `item`/`item_control`/`item_text` の見た目（`data-state="checked"`）は
//! SSR 時の `checked` 引数から固定生成されるため追従せず、実際に選択・送信
//! される値と支援技術が認識する状態・カスタム radio の視覚表示が食い違う
//! （静的な初期状態のみという上記契約にも反する）。`contact_split_form_image`
//! が同種の問題を `RadioGroupProps { disabled: true, .. }` で解決した判断を
//! 踏襲する。`disabled: true` に伴い `item` slot が
//! [`fandhe_frontend_pre_styled_ui::recipe::disabled_declarations`]
//! （既定 `opacity: 0.5` + `cursor: not-allowed`）を受けるため、
//! `contact_split_form_image` と同型に [`LAYOUT_CSS`] で中和し、他 block と
//! 見た目をそろえる（`[data-blocks-promo-signup-offer-interest-fieldset]
//! [data-scope="radio-group"][data-part="item"][data-disabled]` で
//! `opacity: 1`/`cursor: default` へ上書き）。
//!
//! # `drop_class_attr` を踏まえた CSS フックの選び方
//!
//! `card::root`/`card::body`/`heading::heading`/`styled_text::text`/
//! `image::image`/`field::root`/`input::input`/`button::button`/
//! `fieldset::root`/`radio_group::root`/`link::root` はいずれも
//! `drop_class_attr` により呼び出し側 `attrs` の `class` を黙って除去する
//! 契約を持つため、Demo 固有のスタイルフックは
//! `data-blocks-promo-signup-offer-*` 属性で渡し、[`LAYOUT_CSS`] 側も同じ
//! 属性セレクタで対応する。素の `div` には `class` がそのまま効くため、
//! 配置用コンテナは `.blocks-promo-signup-offer-*` クラスセレクタを使う。
//! レイアウト root の class（`blocks-promo-signup-offer-layout`）は
//! [`Block::demo_class`]（`blocks-promo-signup-offer`）と意図的に別名にする
//! （既存 block と同じ Bugbot 教訓の回避）。
//!
//! # id / ARIA の方針
//!
//! メール欄の `id` は 4 インスタンス間で重複させないよう、variant ごとの
//! リテラル定数（`-a`〜`-d` の suffix、`format!` は使わない）で固定する
//! （`demo_output_has_no_dangling_aria_references_or_duplicate_ids` 契約）。
//! B の興味カテゴリ [`fieldset::root`] の `id` は
//! `blocks-promo-signup-offer-interest`、[`radio_group::root`] の
//! `labelled_by` はその legend の id 導出規則（`"{id}-legend"`）と一致する
//! `blocks-promo-signup-offer-interest-legend` をリテラルで直書きする。
//!
//! # ブレークポイントをリテラルで直書きする理由
//!
//! テーマの breakpoint トークンは `@media` 条件式の中では解決できない
//! （CSS custom property は宣言側でのみ有効）ため、
//! [`fandhe_frontend_pre_styled_ui::recipe::Breakpoint::Md`]（768px =
//! 48rem）と一致するリテラル値を [`LAYOUT_CSS`] へ直書きする（既存 block と
//! 同じ判断）。狭幅（既定）は画像を隠し 1 列、`48rem` 以上で 2 列 grid・
//! 画像表示（縦長の画像列）へ切り替える。C は `48rem` 以上でのみ画像側へ
//! `order: 2` を与えて左右を入れ替える（狭幅では常に画像が上に来る）。
//!
//! # 同意文のリンク文言
//!
//! [`consent_text`] のリンク先は [`REPO`]（本リポジトリの固定外部 URL）で
//! あり、実在の利用規約ページではない。遷移先と矛盾する「利用規約」の
//! ようなリンク文言は使わず、遷移先（プロジェクトのリポジトリ）と一致する
//! 「ご利用にあたっての注意事項」という語を使う。
//!
//! # 同意文はボタンの下に置く
//!
//! モジュール doc「4 形を 1 つの Demo に並記する」節の対応表どおり、形 C の
//! 同意文は登録ボタンの**下**に表示する契約のため、[`offer_card`] は
//! ボタン直前に差し込む `extra`（形 B の fieldset 用）とは別に、ボタン
//! 直後に差し込む `trailing`（形 C の同意文用）を受け取る。
//!
//! # 見出しレベル
//!
//! ページ側が `## Demo` として `h2` を出すため、全ての見出しは
//! `HeadingLevel::H3` を使う。
//!
//! # `text` の名前衝突
//!
//! `fandhe_frontend_pre_styled_ui::text::text` と `fandhe_frontend_core::text`
//! が同名のため、styled 側を `styled_text` として取り込む（`crate::blocks`
//! 内の他 block と同じ回避方法）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::field::{
    self, FieldIds, FieldOrientation, FieldProps, FieldRootProps,
};
use fandhe_frontend_pre_styled_ui::fieldset::{self, FieldsetProps, FieldsetRootProps};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::image::{self, ImageFit, ImageProps};
use fandhe_frontend_pre_styled_ui::input::{self, InputProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::radio_group::{self, RadioGroupProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// リンク先の固定外部 URL（同意文リンク、モジュール doc「見出しレベル」節に
/// 準じ死リンク `href="#"` は使わない既存方針）。実体は本リポジトリであり
/// 実在の利用規約ページではないため、リンク文言は「利用規約」のような
/// 法的文書を指す語ではなく、遷移先と矛盾しない一般的な語にする
/// （モジュール doc「同意文のリンク文言」節参照）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// 興味カテゴリ fieldset の `id`。legend の id は headless
/// [`fieldset::legend`] の導出規則（`"{id}-legend"`）に一致させてリテラルで
/// 直書きする（`format!` は使わない）。
const INTEREST_FIELDSET_ID: &str = "blocks-promo-signup-offer-interest";
const INTEREST_LEGEND_ID: &str = "blocks-promo-signup-offer-interest-legend";
/// 興味カテゴリ radio group のネイティブ `<input>` の共通 `name`。
const INTEREST_NAME: &str = "blocks-promo-signup-offer-interest";

/// 各形の直前に置く短い形ラベル（`styled_text::text` の `Sm`/`Muted`）。
fn variant_label(label: &'static str) -> Node {
    styled_text::text(
        &TextProps {
            size: TextSize::Sm,
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![],
        vec![text(label)],
    )
}

/// 興味カテゴリ選択肢 1 件（`radio_group::item` 3 パーツの組み立て）。
fn interest_item(
    checked: bool,
    props: &RadioGroupProps,
    value: &'static str,
    label: &'static str,
) -> Node {
    radio_group::item(
        checked,
        props,
        value,
        vec![],
        vec![
            radio_group::item_hidden_input(checked, props, Some(INTEREST_NAME), value, vec![]),
            radio_group::item_control(checked, props, vec![]),
            radio_group::item_text(checked, props, vec![], vec![text(label)]),
        ],
    )
}

/// 形 B（R0345）が追加する興味カテゴリ fieldset + radio group（ネイティブ
/// 操作不能にする理由はモジュール doc「興味カテゴリ radio group をネイティブ
/// disabled にする理由」節）。
fn interest_fieldset() -> Node {
    let fieldset_props = FieldsetProps {
        id: INTEREST_FIELDSET_ID,
        disabled: false,
        invalid: false,
        has_helper_text: false,
    };
    let radio_props = RadioGroupProps {
        disabled: true,
        ..RadioGroupProps::default()
    };
    fieldset::root(
        &FieldsetRootProps::default(),
        &fieldset_props,
        vec![("data-blocks-promo-signup-offer-interest-fieldset", "")],
        vec![
            fieldset::legend(&fieldset_props, vec![], vec![text("興味のあるカテゴリ")]),
            radio_group::root(
                Size::Md,
                ColorPalette::Accent,
                true,
                None,
                Some(INTEREST_LEGEND_ID),
                vec![("data-blocks-promo-signup-offer-interest-group", "")],
                vec![
                    interest_item(true, &radio_props, "apparel", "アパレル"),
                    interest_item(false, &radio_props, "home", "生活雑貨"),
                    interest_item(false, &radio_props, "food", "食品"),
                ],
            ),
        ],
    )
}

/// 形 C（R0347）が末尾に添える同意文（text + link）。
fn consent_text() -> Node {
    styled_text::text(
        &TextProps {
            size: TextSize::Sm,
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![("data-blocks-promo-signup-offer-consent", "")],
        vec![
            text("登録すると"),
            link::root(
                REPO,
                &LinkProps {
                    external: true,
                    ..LinkProps::default()
                },
                vec![],
                vec![text("ご利用にあたっての注意事項")],
            ),
            text("に同意したものとみなされます。"),
        ],
    )
}

/// カード 1 枚分（画像列 + 本文列）を組み立てる。`layout` は
/// `data-blocks-promo-signup-offer-layout` へ渡す属性値
/// （`"default"`/`"reverse"`/`"centered"`）で、[`LAYOUT_CSS`] が CSS フックに
/// 使う。`email_id` は variant ごとに一意なリテラルを呼び出し側から渡す
/// （モジュール doc「id / ARIA の方針」節）。`extra` はボタンの**上**（興味
/// カテゴリ fieldset 用）、`trailing` はボタンの**下**（同意文用）に差し込む
/// （モジュール doc「同意文はボタンの下に置く」節）。
fn offer_card(
    layout: &'static str,
    email_id: &'static str,
    extra: Vec<Node>,
    trailing: Vec<Node>,
) -> Node {
    let email = FieldProps {
        id: email_id,
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: true,
        readonly: false,
        has_helper_text: false,
    };

    let mut body_children = vec![
        image::image(
            &ImageProps::new(dummy_assets::LOGO_SRC, ""),
            vec![("data-blocks-promo-signup-offer-logo", "")],
        ),
        heading::heading(
            HeadingLevel::H3,
            &HeadingProps {
                size: HeadingSize::Xl2,
                ..HeadingProps::default()
            },
            vec![],
            vec![text("先行登録で特典をゲット")],
        ),
        styled_text::text(
            &TextProps {
                variant: TextVariant::Muted,
                ..TextProps::default()
            },
            vec![],
            vec![text(
                "メールアドレスをご登録いただくと、次回のお買い物でお使いいただける特典をお送りします。",
            )],
        ),
        field::root(
            &FieldRootProps {
                orientation: FieldOrientation::Vertical,
            },
            &email,
            vec![],
            vec![
                field::label(&email, vec![], vec![text("メールアドレス")]),
                input::input(
                    &InputProps::default(),
                    &email,
                    vec![
                        ("type", "email"),
                        ("placeholder", "you@example.com"),
                        ("autocomplete", "email"),
                    ],
                ),
            ],
        ),
    ];
    body_children.extend(extra);
    body_children.push(button::button(
        &ButtonProps::default(),
        vec![("data-blocks-promo-signup-offer-submit", "")],
        vec![text("今すぐ登録する")],
    ));
    body_children.extend(trailing);

    let media = div(
        vec![("class", "blocks-promo-signup-offer-media")],
        vec![image::image(
            &ImageProps {
                fit: ImageFit::Cover,
                ..ImageProps::new(dummy_assets::PRODUCT_SRC, "")
            },
            vec![("data-blocks-promo-signup-offer-image", "")],
        )],
    );
    let body = card::body(
        vec![("data-blocks-promo-signup-offer-body", "")],
        body_children,
    );

    card::root(
        CardProps::default(),
        vec![
            ("data-blocks-promo-signup-offer-card", ""),
            ("data-blocks-promo-signup-offer-layout", layout),
        ],
        vec![media, body],
    )
}

/// `promo-signup-offer` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数（他 block と同じ契約）。4 形を縦に並べる（モジュール doc「4 形を
/// 1 つの Demo に並記する」節）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-promo-signup-offer-layout")],
        vec![
            variant_label("基準形"),
            offer_card(
                "default",
                "blocks-promo-signup-offer-email-a",
                vec![],
                vec![],
            ),
            variant_label("カテゴリ選択"),
            offer_card(
                "default",
                "blocks-promo-signup-offer-email-b",
                vec![interest_fieldset()],
                vec![],
            ),
            variant_label("左右反転 + 同意文"),
            offer_card(
                "reverse",
                "blocks-promo-signup-offer-email-c",
                vec![],
                vec![consent_text()],
            ),
            variant_label("中央寄せ"),
            offer_card(
                "centered",
                "blocks-promo-signup-offer-email-d",
                vec![],
                vec![],
            ),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/promo-signup-offer/",
    title: "promo-signup-offer",
    category: BlockCategory::Promo,
    rust_source: "crates/docs-site/src/blocks/ecommerce/promo/promo_signup_offer.rs",
    demo_class: "blocks-promo-signup-offer",
    parts: &[
        Part {
            label: "Card",
            path: "/themes/card/",
        },
        Part {
            label: "Heading",
            path: "/themes/heading/",
        },
        Part {
            label: "Text",
            path: "/themes/text/",
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
            label: "Radio Group",
            path: "/themes/radio-group/",
        },
        Part {
            label: "Fieldset",
            path: "/themes/fieldset/",
        },
        Part {
            label: "Link",
            path: "/themes/link/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `promo_signup_offer` 固有のレイアウト規則（`--fandhe-*` トークンのみ
/// 使用）。ブレークポイントは
/// [`fandhe_frontend_pre_styled_ui::recipe::Breakpoint::Md`]（768px =
/// 48rem）と一致するリテラル値を直書きする（モジュール doc
/// 「ブレークポイントをリテラルで直書きする理由」節参照）。
const LAYOUT_CSS: &str = "\
.blocks-promo-signup-offer-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n}\n\
[data-scope=\"card\"][data-part=\"root\"][data-blocks-promo-signup-offer-card] {\n  padding: 0;\n  overflow: hidden;\n}\n\
.blocks-promo-signup-offer-media {\n  display: none;\n}\n\
[data-scope=\"image\"][data-part=\"root\"][data-blocks-promo-signup-offer-image] {\n  display: block;\n  width: 100%;\n  height: 100%;\n}\n\
[data-scope=\"card\"][data-part=\"body\"][data-blocks-promo-signup-offer-body] {\n  display: flex;\n  flex-direction: column;\n  align-items: flex-start;\n  gap: var(--fandhe-space-4);\n  padding: var(--fandhe-space-6);\n}\n\
[data-scope=\"image\"][data-part=\"root\"][data-blocks-promo-signup-offer-logo] {\n  width: 2.5rem;\n  height: 2.5rem;\n}\n\
[data-blocks-promo-signup-offer-interest-fieldset] [data-scope=\"radio-group\"][data-part=\"item\"][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
[data-blocks-promo-signup-offer-layout=\"centered\"] [data-scope=\"card\"][data-part=\"body\"][data-blocks-promo-signup-offer-body] {\n  align-items: center;\n  text-align: center;\n}\n\
@media (min-width: 48rem) {\n  \
[data-scope=\"card\"][data-part=\"root\"][data-blocks-promo-signup-offer-card] {\n    display: grid;\n    grid-template-columns: repeat(2, minmax(0, 1fr));\n  }\n  \
.blocks-promo-signup-offer-media {\n    display: block;\n    min-height: 20rem;\n  }\n  \
[data-blocks-promo-signup-offer-layout=\"reverse\"] .blocks-promo-signup-offer-media {\n    order: 2;\n  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;

    /// [`demo`] が 10 部品すべての anatomy を実際に出力し、画像 8 枚
    /// （4 形 × 画像 + ロゴ）・ボタン 4 個を持ち、禁止パターン
    /// （`<form>`/`data:` URI/死リンク）を含まないことを固定する。
    #[test]
    fn demo_composes_expected_parts_and_avoids_forms() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"card\"",
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"field\" data-part=\"root\"",
            "data-scope=\"field\" data-part=\"input\"",
            "data-scope=\"button\"",
            "data-scope=\"image\"",
            "data-scope=\"radio-group\"",
            "data-scope=\"fieldset\"",
            "data-scope=\"link\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
        assert_eq!(
            html.matches("<img").count(),
            8,
            "should render exactly 8 images (4 cards x (product + logo))"
        );
        assert_eq!(
            html.matches("type=\"button\"").count(),
            4,
            "should render exactly 4 submit buttons (1 per variant)"
        );
        assert!(!html.contains("<form"), "demo should never contain <form");
        for absent in ["href=\"#\"", "src=\"data:", "type=\"submit\"", "action="] {
            assert!(!html.contains(absent), "demo should never contain {absent}");
        }
    }

    /// 4 形が互いに区別可能であること（layout 属性値・radio 選択肢件数・
    /// 同意文リンク件数）を固定する。
    #[test]
    fn variants_are_distinguishable() {
        let html = render(&demo());
        assert!(html.contains("data-blocks-promo-signup-offer-layout=\"default\""));
        assert!(html.contains("data-blocks-promo-signup-offer-layout=\"reverse\""));
        assert!(html.contains("data-blocks-promo-signup-offer-layout=\"centered\""));
        assert_eq!(
            html.matches(&format!("name=\"{INTEREST_NAME}\"")).count(),
            3,
            "the category-select variant should render exactly 3 interest radio options"
        );
        assert_eq!(
            html.matches("data-blocks-promo-signup-offer-consent")
                .count(),
            1,
            "only the reverse variant should render the consent text"
        );
        assert_eq!(
            html.matches(&format!("href=\"{REPO}\"")).count(),
            1,
            "should render exactly 1 link to REPO (consent text)"
        );
    }

    /// 形 C の同意文は登録ボタンの**下**に表示する契約（モジュール doc
    /// 「同意文はボタンの下に置く」節）を固定する。ボタン・同意文の
    /// マーカーがこの順に出現することを位置比較で検証する。
    #[test]
    fn consent_text_appears_after_submit_button() {
        let html = render(&demo());
        let submit_pos = html
            .find("data-blocks-promo-signup-offer-submit")
            .expect("submit button marker should be present");
        let consent_pos = html
            .find("data-blocks-promo-signup-offer-consent")
            .expect("consent text marker should be present");
        assert!(
            submit_pos < consent_pos,
            "consent text should be rendered after (below) the submit button"
        );
    }

    /// 興味カテゴリ radio group をネイティブ disabled にすることで受ける
    /// `disabled_declarations()`（`opacity: 0.5`/`cursor: not-allowed`）を
    /// [`LAYOUT_CSS`] が中和し、他 block と同じ見た目（`opacity: 1`/
    /// `cursor: default`）になることを固定する（モジュール doc「興味
    /// カテゴリ radio group をネイティブ disabled にする理由」節）。
    #[test]
    fn disabled_interest_radio_items_are_visually_neutralized() {
        assert!(LAYOUT_CSS.contains(
            "[data-blocks-promo-signup-offer-interest-fieldset] [data-scope=\"radio-group\"][data-part=\"item\"][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}"
        ));
    }

    /// [`LAYOUT_CSS`] が 48rem ブレークポイント・画像の表示切り替え・
    /// 左右反転の order 切替を持つことを固定する。
    #[test]
    fn layout_css_hides_media_below_md() {
        assert!(LAYOUT_CSS.contains("@media (min-width: 48rem)"));
        assert!(LAYOUT_CSS.contains(".blocks-promo-signup-offer-media {\n  display: none;\n}"));
        assert!(LAYOUT_CSS.contains(
            "[data-blocks-promo-signup-offer-layout=\"reverse\"] .blocks-promo-signup-offer-media {\n    order: 2;\n  }"
        ));
        assert!(LAYOUT_CSS.contains("var(--fandhe-"));
        assert!(!LAYOUT_CSS.contains('#'));
        assert!(!LAYOUT_CSS.contains("white"));
    }

    /// ルート class が [`demo`] の出力へ実際に現れ、かつ `BLOCK.demo_class`
    /// とは異なること（既存 Blocks と同じ教訓）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("blocks-promo-signup-offer-layout"));
        assert_ne!("blocks-promo-signup-offer-layout", BLOCK.demo_class);
    }
}
