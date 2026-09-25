//! `contact-split-form-image` block（イシュー #2831。親トラッキング #2807
//! 「Blocks マーケティング B」、ルート #2730「Blocks 目的別パーツ拡充
//! ツリー」配下）。左カラムに問い合わせフォーム、右カラムに画像を置いた
//! 2 カラムのお問い合わせ用ブロック。
//!
//! # 使用部品
//!
//! `heading` / `text` / `field` / `input` / `textarea` / `radio-group` /
//! `fieldset` / `separator` / `button` / `image` の 10 部品を合成する
//! （[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//! 新しい UI 部品は追加しない。
//!
//! # `<form>` を持たない・データ取得/送信を行わない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である。送信ボタンは [`fandhe_frontend_pre_styled_ui::button::button`]
//! の既定 `type="button"` のまま用いる（暗黙 submit も起きない）。予算の
//! radio group は先頭 1 件が選択済みの**静的な初期状態**のみを描く
//! （`docs/policy/intentional-non-adoption.md` §3.25、UI コンポーネント層は
//! アプリケーションロジックを内包しない）。文言はすべて独自に書いた架空の
//! ものであり、実企業名・実クレデンシャル・PII を含まない。
//!
//! # 予算 radio group をネイティブ disabled にする理由（レビュー是正）
//!
//! [`radio_group::item_hidden_input`] は有効なネイティブ
//! `<input type="radio">` であり、`disabled` を渡さない構成では docs
//! サイトが JS ハイドレーションを行わなくてもラベルクリック・キーボード
//! 操作でブラウザが `checked` をネイティブに切り替えてしまう。一方
//! `item`/`item_control`/`item_text` の見た目（`data-state="checked"`）は
//! SSR 時の `checked` 引数から固定生成されるため追従せず、実際に選択・
//! 送信される値と支援技術が認識する状態・カスタム radio の視覚表示が
//! 食い違う（静的な初期状態のみという上記契約にも反する）。
//! `changelog_accordion`（`crates/docs-site/src/blocks/marketing/changelog/
//! changelog_accordion.rs`）が `item_trigger` の同種の問題を
//! `AccordionProps { disabled: true, .. }` で解決した判断を踏襲し、
//! [`RadioGroupProps`] の `disabled: true` を [`root`]・全 [`budget_item`]
//! （`item`/`item_control`/`item_hidden_input`）へ共有する。ネイティブ
//! `disabled` 属性でフォーカス・操作を不能にし、状態が二度と変化しない
//! ことを構造的に保証する（クライアント側の状態配線を追加する代替案は
//! 採らない。無 JS の静的合成例という block 全体の設計方針に反するため）。
//! `disabled_declarations()`（既定 `opacity: 0.5` + `cursor: not-allowed`）は
//! [`LAYOUT_CSS`] で中和し、通常の radio group と同じ見た目に保つ
//! （`changelog_accordion` と同型の中和パターン）。
//!
//! # 参照について
//!
//! 主参照は対応表 ID R0065（フォーム + 画像の基本形）、集約元は R0859
//! （予算 radio の fieldset、画像は右半分の全高）。取得手段・ファイル名・
//! 出典名・内部識別子は記載しない（`contact_dialog_form`/`contact_image_info`
//! と同じライセンス上の転記制限、対応表 ID のみを記す）。取り込むのは
//! 領域の配置と部品構成という構造のみで、文言・配色・装飾・アイコンは
//! 独自に書く。参照との差分は末尾「原案差分メモ」（`site/blocks/
//! contact-split-form-image.md`）へ記載する。
//!
//! # 見出しレベル
//!
//! ページ側が `## Demo` として `h2` を出すため、見出しは `HeadingLevel::H3`
//! を使う（既存 block と同じ判断）。
//!
//! # `text` の名前衝突
//!
//! `fandhe_frontend_pre_styled_ui::text::text` と `fandhe_frontend_core::text`
//! が同名のため、styled 側を `styled_text` として取り込む（`crate::blocks`
//! 内の他 block と同じ回避方法）。
//!
//! # `drop_class_attr` と CSS フックの選び方
//!
//! `heading`/`text`/`image`/`button`/`field::root`/`fieldset::root`/
//! `radio_group::root`/`separator`/`textarea` はいずれも `drop_class_attr`
//! により呼び出し側 `attrs` の `class` を黙って除去する契約を持つため、
//! Demo 固有のスタイルフックは `data-blocks-contact-split-form-image-*`
//! 属性で渡し、[`LAYOUT_CSS`] 側も同じ属性セレクタで対応する。素の `div`
//! には `class` がそのまま効くため、レイアウトは
//! `.blocks-contact-split-form-image-*` クラスセレクタを使う。レイアウト
//! root の class（`blocks-contact-split-form-image-layout`）は
//! [`Block::demo_class`]（`blocks-contact-split-form-image`）と意図的に
//! 別名にする（既存 block と同じ Bugbot 教訓の回避）。
//!
//! # id / ARIA の方針
//!
//! id はすべて `blocks-contact-split-form-image-` を prefix に持つ
//! リテラル文字列で固定する（`format!` は使わない。`fieldset::legend` の
//! id 導出（`"{id}-legend"`）と一致する値をリテラルで直書きする、
//! `crate::blocks` モジュール doc「宙に浮いた ARIA 参照・id 重複を避ける」
//! 契約に従う静的な突合）。`has_helper_text: true` にした欄（電話番号・
//! お問い合わせ内容）には必ず [`field::helper_text`] を対で描く。予算
//! [`fieldset::root`] の `id` は `blocks-contact-split-form-image-budget`、
//! [`radio_group::root`] の `labelled_by` はその legend id
//! （`blocks-contact-split-form-image-budget-legend`）と一致させる。
//! [`radio_group::item_hidden_input`] の `name` はすべて
//! `blocks-contact-split-form-image-budget` に統一し、先頭の選択肢のみ
//! `checked` にする。
//!
//! # ブレークポイントをリテラルで直書きする理由
//!
//! テーマの breakpoint トークンは `@media` 条件式の中では解決できない
//! （CSS custom property は宣言側でのみ有効）ため、
//! [`fandhe_frontend_pre_styled_ui::recipe::Breakpoint::Md`]（768px =
//! 48rem）と一致するリテラル値を [`LAYOUT_CSS`] へ直書きする（既存 block と
//! 同じ判断）。狭幅（既定）は画像を隠しフォームを 1 列にし、`48rem` 以上で
//! 2 カラム grid・画像表示・名前欄 2 列・予算 radio 2×2・送信ボタン右寄せに
//! 切り替える。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::field::{
    self, FieldIds, FieldOrientation, FieldProps, FieldRootProps,
};
use fandhe_frontend_pre_styled_ui::fieldset::{self, FieldsetProps, FieldsetRootProps};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::image::{self, AspectRatio, ImageFit, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::input::{self, InputProps};
use fandhe_frontend_pre_styled_ui::radio_group::{self, RadioGroupProps};
use fandhe_frontend_pre_styled_ui::separator::{self, SeparatorProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};
use fandhe_frontend_pre_styled_ui::textarea::{self, TextareaProps};
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// 予算 radio group のネイティブ `<input>` の共通 `name`（モジュール doc
/// 「id / ARIA の方針」節）。
const BUDGET_NAME: &str = "blocks-contact-split-form-image-budget";

/// 予算 [`fieldset::root`] の `id`。legend の id は headless
/// [`fieldset::legend`] の導出規則（`"{id}-legend"`）に一致させて
/// リテラルで直書きする（`format!` は使わない）。
const BUDGET_FIELDSET_ID: &str = "blocks-contact-split-form-image-budget";
const BUDGET_LEGEND_ID: &str = "blocks-contact-split-form-image-budget-legend";

/// 姓・名 2 欄の行（狭幅は 1 列、`48rem` 以上は 2 列。モジュール doc
/// 「ブレークポイント」節）。
fn name_row(orientation: &FieldRootProps) -> Node {
    let first_name = FieldProps {
        id: "blocks-contact-split-form-image-first-name",
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: true,
        readonly: false,
        has_helper_text: false,
    };
    let last_name = FieldProps {
        id: "blocks-contact-split-form-image-last-name",
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: true,
        readonly: false,
        has_helper_text: false,
    };
    div(
        vec![("class", "blocks-contact-split-form-image-name-row")],
        vec![
            field::root(
                orientation,
                &first_name,
                vec![],
                vec![
                    field::label(&first_name, vec![], vec![text("姓")]),
                    input::input(
                        &InputProps::default(),
                        &first_name,
                        vec![
                            ("type", "text"),
                            ("placeholder", "山田"),
                            ("autocomplete", "family-name"),
                        ],
                    ),
                ],
            ),
            field::root(
                orientation,
                &last_name,
                vec![],
                vec![
                    field::label(&last_name, vec![], vec![text("名")]),
                    input::input(
                        &InputProps::default(),
                        &last_name,
                        vec![
                            ("type", "text"),
                            ("placeholder", "太郎"),
                            ("autocomplete", "given-name"),
                        ],
                    ),
                ],
            ),
        ],
    )
}

/// メールアドレス欄（必須、補助テキストなし）。
fn email_field(orientation: &FieldRootProps) -> Node {
    let email = FieldProps {
        id: "blocks-contact-split-form-image-email",
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: true,
        readonly: false,
        has_helper_text: false,
    };
    field::root(
        orientation,
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
    )
}

/// 電話番号欄（任意、補助テキスト付き。モジュール doc「id / ARIA の方針」
/// 節が述べる `has_helper_text` の実例）。
fn phone_field(orientation: &FieldRootProps) -> Node {
    let phone = FieldProps {
        id: "blocks-contact-split-form-image-phone",
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: true,
    };
    field::root(
        orientation,
        &phone,
        vec![],
        vec![
            field::label(&phone, vec![], vec![text("電話番号")]),
            input::input(
                &InputProps::default(),
                &phone,
                vec![
                    ("type", "tel"),
                    ("placeholder", "090-1234-5678"),
                    ("autocomplete", "tel"),
                ],
            ),
            field::helper_text(
                &phone,
                vec![],
                vec![text("任意です。折り返しのご連絡先としてご利用します。")],
            ),
        ],
    )
}

/// お問い合わせ内容欄（必須、補助テキスト付き）。
fn message_field(orientation: &FieldRootProps) -> Node {
    let message = FieldProps {
        id: "blocks-contact-split-form-image-message",
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: true,
        readonly: false,
        has_helper_text: true,
    };
    field::root(
        orientation,
        &message,
        vec![],
        vec![
            field::label(&message, vec![], vec![text("お問い合わせ内容")]),
            textarea::textarea(
                &TextareaProps::default(),
                &message,
                false,
                vec![
                    ("rows", "5"),
                    ("placeholder", "ご質問・ご要望をご記入ください"),
                ],
                vec![],
            ),
            field::helper_text(&message, vec![], vec![text("目安は 400 文字程度です。")]),
        ],
    )
}

/// 予算選択肢 1 件（`radio_group::item` 3 パーツの組み立て、モジュール doc
/// 「id / ARIA の方針」節）。
fn budget_item(
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
            radio_group::item_hidden_input(checked, props, Some(BUDGET_NAME), value, vec![]),
            radio_group::item_control(checked, props, vec![]),
            radio_group::item_text(checked, props, vec![], vec![text(label)]),
        ],
    )
}

/// ご予算欄（fieldset + radio group、狭幅は縦積み・`48rem` 以上は 2×2。
/// 選択状態は先頭 1 件のみ固定した静的表示、モジュール doc「`<form>` を
/// 持たない」節）。ネイティブ操作で `checked` と視覚表示が食い違わない
/// よう `disabled: true` で固定する（モジュール doc「予算 radio group を
/// ネイティブ disabled にする理由」節）。
fn budget_fieldset() -> Node {
    let fieldset_props = FieldsetProps {
        id: BUDGET_FIELDSET_ID,
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
        vec![("data-blocks-contact-split-form-image-budget-fieldset", "")],
        vec![
            fieldset::legend(&fieldset_props, vec![], vec![text("ご予算")]),
            radio_group::root(
                Size::Md,
                ColorPalette::Accent,
                true,
                None,
                Some(BUDGET_LEGEND_ID),
                vec![("data-blocks-contact-split-form-image-budget-group", "")],
                vec![
                    budget_item(true, &radio_props, "under-10", "〜10 万円"),
                    budget_item(false, &radio_props, "10-to-50", "10 万〜50 万円"),
                    budget_item(false, &radio_props, "50-to-200", "50 万〜200 万円"),
                    budget_item(false, &radio_props, "over-200", "200 万円〜"),
                ],
            ),
        ],
    )
}

/// フォーム本体（`field::group` で縦積みにまとめる。モジュール doc「使用
/// 部品」節の 10 部品すべてがここへ現れる）。
fn form() -> Node {
    let orientation = FieldRootProps {
        orientation: FieldOrientation::Vertical,
    };
    div(
        vec![("class", "blocks-contact-split-form-image-form")],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    ..HeadingProps::default()
                },
                vec![],
                vec![text("お問い合わせ")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(
                    "ご質問・ご相談をお送りください。担当者が折り返しご連絡します。",
                )],
            ),
            field::group(
                vec![],
                vec![
                    name_row(&orientation),
                    email_field(&orientation),
                    phone_field(&orientation),
                    message_field(&orientation),
                    budget_fieldset(),
                    separator::separator(&SeparatorProps::default(), vec![]),
                    div(
                        vec![("class", "blocks-contact-split-form-image-actions")],
                        vec![button::button(
                            &ButtonProps::default(),
                            vec![("data-blocks-contact-split-form-image-submit", "")],
                            vec![text("送信する")],
                        )],
                    ),
                ],
            ),
        ],
    )
}

/// 画像領域（右カラム、`48rem` 以上でのみ表示・全高。モジュール doc
/// 「ブレークポイント」節）。
fn media() -> Node {
    div(
        vec![("class", "blocks-contact-split-form-image-media")],
        vec![image::image(
            &ImageProps {
                fit: ImageFit::Cover,
                aspect_ratio: AspectRatio::Auto,
                shape: ImageShape::Square,
                ..ImageProps::new(dummy_assets::BACKGROUND_SRC, "")
            },
            vec![("data-blocks-contact-split-form-image-image", "")],
        )],
    )
}

/// `contact-split-form-image` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。DOM 順はフォーム → 画像（狭幅の 1 列表示でフォームが常に
/// 上に来るようにするため、画像は CSS で非表示にする）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-contact-split-form-image-layout")],
        vec![form(), media()],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/contact-split-form-image/",
    title: "contact-split-form-image",
    category: BlockCategory::Contact,
    rust_source: "crates/docs-site/src/blocks/marketing/contact/contact_split_form_image.rs",
    demo_class: "blocks-contact-split-form-image",
    parts: &[
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
            label: "Textarea",
            path: "/themes/textarea/",
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
            label: "Separator",
            path: "/themes/separator/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Image",
            path: "/themes/image/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `contact_split_form_image` 固有のレイアウト規則（`crate::blocks::
/// LAYOUT_CSS` doc「block 固有 CSS の置き場」節、`contact_image_info`/
/// `contact_dialog_form` と同型で本ファイル内 private 定数として
/// `super::stylesheet` 経由の `push_css` で連結される）。
///
/// セレクタは `.blocks-contact-split-form-image-*` と
/// `[data-blocks-contact-split-form-image-*]` のみを用い、他 block や部品の
/// 素のセレクタへ影響させない（既存 block と同じ名前空間分離）。狭幅
/// （既定）は画像を隠して 1 列、`48rem` 以上で 2 カラム grid・名前欄 2 列・
/// 予算 radio 2×2・送信ボタン右寄せに切り替える（モジュール doc
/// 「ブレークポイントをリテラルで直書きする理由」節）。
///
/// 予算 radio group の grid 化フックは `[data-blocks-contact-split-form-
/// image-budget-group]` 単独（属性セレクタ 1 個、詳細度 (0,1,0)）ではなく
/// `[data-scope="radio-group"][data-part="root"][data-blocks-contact-split-
/// form-image-budget-group]`（属性セレクタ 3 個、詳細度 (0,3,0)）を使う
/// （レビュー是正、`contact_dialog_form` の `[data-scope="field"][data-
/// part="textarea"][data-blocks-contact-dialog-form-message]` と同型の
/// 前例）。`radio_group::stylesheet` の `root` base 規則が
/// `[data-scope="radio-group"][data-part="root"]`（属性セレクタ 2 個、
/// 詳細度 (0,2,0)）で `display: flex` を宣言しており、単独属性セレクタ
/// では `@media` 内で上書きしても詳細度で負けて `display: grid` が
/// 適用されない（`@media` 内外は詳細度比較に影響しない）。
///
/// 予算 radio group はネイティブ操作不能にするため `disabled: true`
/// （モジュール doc「予算 radio group をネイティブ disabled にする理由」
/// 節）で描くが、これに伴い `item` slot が `disabled_declarations()`
/// （`opacity: 0.5` + `cursor: not-allowed`）を受けるため、
/// `.blocks-contact-split-form-image-form [data-scope="radio-group"]
/// [data-part="item"][data-disabled]`（クラス祖先 + 属性セレクタ 3 個、
/// 詳細度 (0,4,0) で `[data-scope="radio-group"][data-part="item"]
/// [data-disabled]`〔属性セレクタ 3 個、詳細度 (0,3,0)〕を上回る）で
/// `opacity: 1`/`cursor: default` へ中和する（`changelog_accordion` の
/// `item-trigger[data-disabled]` 中和と同型）。
const LAYOUT_CSS: &str = "\
.blocks-contact-split-form-image-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n  align-items: stretch;\n}\n\
.blocks-contact-split-form-image-form {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n  min-width: 0;\n}\n\
.blocks-contact-split-form-image-name-row {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
[data-blocks-contact-split-form-image-budget-fieldset] {\n  display: flex;\n  flex-direction: column;\n}\n\
[data-scope=\"radio-group\"][data-part=\"root\"][data-blocks-contact-split-form-image-budget-group] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-contact-split-form-image-form [data-scope=\"radio-group\"][data-part=\"item\"][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
.blocks-contact-split-form-image-actions {\n  display: flex;\n  justify-content: flex-end;\n}\n\
.blocks-contact-split-form-image-actions [data-scope=\"button\"] {\n  width: 100%;\n}\n\
.blocks-contact-split-form-image-media {\n  display: none;\n}\n\
@media (min-width: 48rem) {\n  \
.blocks-contact-split-form-image-layout {\n    display: grid;\n    grid-template-columns: repeat(2, minmax(0, 1fr));\n    gap: var(--fandhe-space-8);\n  }\n  \
.blocks-contact-split-form-image-name-row {\n    flex-direction: row;\n  }\n  \
.blocks-contact-split-form-image-name-row > [data-scope=\"field\"] {\n    flex: 1 1 0%;\n  }\n  \
[data-scope=\"radio-group\"][data-part=\"root\"][data-blocks-contact-split-form-image-budget-group] {\n    display: grid;\n    grid-template-columns: repeat(2, minmax(0, 1fr));\n  }\n  \
.blocks-contact-split-form-image-actions {\n    justify-content: flex-end;\n  }\n  \
.blocks-contact-split-form-image-actions [data-scope=\"button\"] {\n    width: auto;\n  }\n  \
.blocks-contact-split-form-image-media {\n    display: block;\n    height: 100%;\n  }\n  \
[data-scope=\"image\"][data-part=\"root\"][data-blocks-contact-split-form-image-image] {\n    display: block;\n    width: 100%;\n    height: 100%;\n  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::{demo, dummy_assets, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    fn demo_html() -> String {
        render(&demo())
    }

    /// Demo が使用部品（heading/text/field/input/textarea/radio-group/
    /// fieldset/separator/button/image）の anatomy をすべて実際に出力して
    /// いることを固定する（`contact_dialog_form`/`contact_image_info` と
    /// 同型）。
    #[test]
    fn demo_composes_expected_parts() {
        let html = demo_html();
        for scope in [
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"field\" data-part=\"root\"",
            "data-scope=\"field\" data-part=\"input\"",
            "data-scope=\"field\" data-part=\"textarea\"",
            "data-scope=\"radio-group\"",
            "data-scope=\"fieldset\"",
            "data-scope=\"separator\"",
            "data-scope=\"button\"",
            "data-scope=\"image\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
        assert_eq!(
            html.matches("data-scope=\"field\" data-part=\"input\"")
                .count(),
            4,
            "demo should render exactly 4 input parts (first/last name, email, phone)"
        );
        assert_eq!(
            html.matches("data-part=\"legend\"").count(),
            1,
            "demo should render exactly 1 legend part"
        );
        assert!(
            html.contains("role=\"radiogroup\""),
            "demo should render a radiogroup role"
        );
    }

    /// `<form>` を出力せず、ボタンが `type="button"` の 1 個のみであることを
    /// 固定する（`crate::blocks` モジュール doc の不変条件）。
    #[test]
    fn demo_has_no_form_and_exactly_one_button_type_button() {
        let html = demo_html();
        assert!(!html.contains("<form"), "demo should never contain <form");
        for absent in ["type=\"submit\"", "action=", "href=", "src=\"data:"] {
            assert!(!html.contains(absent), "demo should never contain {absent}");
        }
        assert_eq!(
            html.matches("type=\"button\"").count(),
            1,
            "demo should have exactly 1 type=\"button\" button (submit)"
        );
    }

    /// name / email / phone / message の `label[for]` と、各コントロールの
    /// `id` が一致することを固定する（アクセシビリティ上の関連付け不変
    /// 条件）。
    #[test]
    fn labels_point_at_their_controls() {
        let html = demo_html();
        for (label_for, control_id) in [
            (
                "for=\"blocks-contact-split-form-image-first-name-control\"",
                "id=\"blocks-contact-split-form-image-first-name-control\"",
            ),
            (
                "for=\"blocks-contact-split-form-image-last-name-control\"",
                "id=\"blocks-contact-split-form-image-last-name-control\"",
            ),
            (
                "for=\"blocks-contact-split-form-image-email-control\"",
                "id=\"blocks-contact-split-form-image-email-control\"",
            ),
            (
                "for=\"blocks-contact-split-form-image-phone-control\"",
                "id=\"blocks-contact-split-form-image-phone-control\"",
            ),
            (
                "for=\"blocks-contact-split-form-image-message-control\"",
                "id=\"blocks-contact-split-form-image-message-control\"",
            ),
        ] {
            assert!(html.contains(label_for), "missing {label_for}");
            assert!(html.contains(control_id), "missing {control_id}");
        }
    }

    /// 補助テキスト（電話番号・お問い合わせ内容）の `aria-describedby` が
    /// 対応する `helper-text` の `id` と一致することを固定する。
    #[test]
    fn helper_texts_are_described_by_their_controls() {
        let html = demo_html();
        for (control_id, helper_id) in [
            (
                "id=\"blocks-contact-split-form-image-phone-control\"",
                "id=\"blocks-contact-split-form-image-phone-helper-text\"",
            ),
            (
                "id=\"blocks-contact-split-form-image-message-control\"",
                "id=\"blocks-contact-split-form-image-message-helper-text\"",
            ),
        ] {
            assert!(html.contains(control_id), "missing {control_id}");
            assert!(html.contains(helper_id), "missing {helper_id}");
        }
        assert!(
            html.contains("aria-describedby=\"blocks-contact-split-form-image-phone-helper-text\"")
        );
        assert!(html
            .contains("aria-describedby=\"blocks-contact-split-form-image-message-helper-text\""));
    }

    /// ご予算 radiogroup の `aria-labelledby` が legend の `id` と一致する
    /// ことを固定する（モジュール doc「id / ARIA の方針」節）。
    #[test]
    fn budget_radiogroup_is_labelled_by_its_legend() {
        let html = demo_html();
        assert!(html.contains("id=\"blocks-contact-split-form-image-budget-legend\""));
        assert!(html.contains("aria-labelledby=\"blocks-contact-split-form-image-budget-legend\""));
    }

    /// 予算 radio group の 4 件がすべて同じ `name` を共有し、先頭 1 件のみ
    /// `checked` の静的な初期状態であることを固定する。
    #[test]
    fn budget_radio_items_share_name_and_first_is_checked() {
        let html = demo_html();
        assert_eq!(
            html.matches("name=\"blocks-contact-split-form-image-budget\"")
                .count(),
            4,
            "all 4 budget radio inputs should share the same name"
        );
        assert_eq!(html.matches(" checked").count(), 1);
    }

    /// 予算 radio group はネイティブ `disabled` により操作不能であること、
    /// つまり `checked`/視覚表示が永続的に食い違わないことを固定する
    /// （codex-review P1 是正、モジュール doc「予算 radio group をネイティブ
    /// disabled にする理由」節）。
    #[test]
    fn budget_radio_group_is_natively_disabled() {
        let html = demo_html();
        assert_eq!(
            html.matches(" disabled=\"\"").count(),
            4,
            "all 4 budget radio inputs should carry the native disabled attribute"
        );
        assert_eq!(
            html.matches("data-disabled=\"\"").count(),
            13,
            "root (1) + item/item-control/item-text (3 per option x 4 options); item_hidden_input carries the native disabled attribute instead of data-disabled"
        );
        assert!(
            html.contains("aria-disabled=\"true\""),
            "the radiogroup root should also be marked aria-disabled"
        );
    }

    /// [`LAYOUT_CSS`] が予算 radio group の disabled 化に伴う視覚中和規則
    /// （`opacity: 1`/`cursor: default`）を持つことを固定する（codex-review
    /// P1 是正に伴う追加）。
    #[test]
    fn layout_css_neutralizes_disabled_budget_item_opacity() {
        assert!(LAYOUT_CSS.contains(
            ".blocks-contact-split-form-image-form [data-scope=\"radio-group\"][data-part=\"item\"][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}"
        ));
    }

    /// 予算 radio group の grid 化フックが `[data-scope="radio-group"]
    /// [data-part="root"]` を含む高詳細度セレクタであることを固定する
    /// （Bugbot 指摘是正: 単独属性セレクタでは `radio_group::stylesheet` の
    /// `root` base 規則〔`display: flex`、詳細度 (0,2,0)〕に詳細度で負けて
    /// `48rem` 以上でも `display: grid` が適用されなかった）。
    #[test]
    fn budget_grid_hook_outranks_radio_group_root_recipe_specificity() {
        assert!(LAYOUT_CSS.contains(
            "[data-scope=\"radio-group\"][data-part=\"root\"][data-blocks-contact-split-form-image-budget-group]"
        ));
        assert!(!LAYOUT_CSS.contains("\n[data-blocks-contact-split-form-image-budget-group] {"));
    }

    /// [`LAYOUT_CSS`] が想定するブレークポイント条件・2 列 grid・2×2
    /// grid・画像の表示切り替えを持つことを固定する。
    #[test]
    fn layout_css_declares_breakpoint_and_grids() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("@media (min-width: 48rem)"));
        assert!(LAYOUT_CSS.contains("repeat(2, minmax(0, 1fr))"));
        assert!(
            LAYOUT_CSS.contains(".blocks-contact-split-form-image-media {\n  display: none;\n}")
        );
    }

    /// ルート grid class（`demo_class` とは別名）が `demo()` の出力へ実際に
    /// 現れること（既存 block と同じ Bugbot 教訓の固定）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = demo_html();
        assert!(html.contains("class=\"blocks-contact-split-form-image-layout\""));
        assert_ne!(
            super::BLOCK.demo_class,
            "blocks-contact-split-form-image-layout"
        );
    }

    /// dummy_assets の src を参照していること。
    #[test]
    fn media_uses_dummy_asset_src() {
        let html = demo_html();
        assert!(html.contains(dummy_assets::BACKGROUND_SRC));
    }
}
