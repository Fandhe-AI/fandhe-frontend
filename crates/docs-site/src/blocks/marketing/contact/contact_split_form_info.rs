//! `contact-split-form-info` block（イシュー #2832。sub-issue #2833
//! （骨格・主要領域・登録一式）・#2834（残りの領域・状態違いの並記・原稿の
//! 仕上げ）。親トラッキング #2807「Blocks マーケティング B」、ルート #2730
//! 「Blocks 目的別パーツ拡充ツリー」配下）。問い合わせフォーム + 連絡先
//! 情報の 2 カラムレイアウトを、主参照 R0441（左フォーム + 連絡先リンク
//! 3 件）・集約元 R0443（左右反転）・R0445（連絡先側 3 グループ + 区切り線）
//! の構造（領域配置・部品構成）だけを取り込んで合成する。取得手段・
//! ファイル名・出典名・内部識別子は記載しない（`contact_image_info`・
//! `contact_info_columns` と同じライセンス上の転記制限、対応表 ID のみを
//! 記す）。R0855（連絡先を `dl` で表示・格子模様の背景）は装飾（背景模様）
//! と使用部品外の要素（`dl`）を含むため取り込まない（原稿「原案差分メモ」
//! 参照）。
//!
//! # 使用部品
//!
//! `heading` / `text` / `badge` / `field` / `input` / `textarea` /
//! `checkbox` / `button` / `icon` / `separator` / `link` の 11 部品を
//! 合成する（[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//! 新しい UI 部品は追加しない。
//!
//! # 1 つの Demo に 2 つの形を縦に並べる
//!
//! [`content_split_image`](super::super::content::content_split_image) と
//! 同じ手法で、[`demo`] はルート `div.blocks-contact-split-form-info-stack`
//! の中に「形ラベル + 形の本体」の組を 2 つ縦に並べる。
//!
//! - **形 A**（[`variant_form_start`]、主参照 R0441）: lg（`64rem`）以上で
//!   「左にフォーム・右に連絡先情報（eyebrow badge + 見出し + 説明 +
//!   アイコン付きリンク 3 件）」。
//! - **形 B**（[`variant_form_end`]、集約元 R0443 の左右反転 + R0445 の
//!   3 グループ）: lg 以上で「左に連絡先情報（見出し + 説明 + 拠点/メール/
//!   SNS の 3 グループ、グループ間は区切り線）・右にフォーム」。
//!
//! 形ごとの左右の違いは新しい列挙型を作らず、block ローカルの属性
//! `data-blocks-contact-split-form-info-variant="form-start"|"form-end"` の
//! 値だけで表す（`contact_info_columns` の `variant` 属性と同じ考え方）。
//!
//! # DOM 順と視覚順（常に一致させる）
//!
//! 各形の DOM の順序は、lg 以上での視覚上の左右関係に揃える: 形 A
//! （`form-start`）は**フォーム → 区切り線 → 連絡先情報**（左にフォーム
//! を置くため）、形 B（`form-end`）は**連絡先情報 → 区切り線 →
//! フォーム**（左に連絡先情報を置くため）。[`LAYOUT_CSS`] は lg 以上で
//! 2 カラム grid へ切り替えるが、`grid-column`/`grid-row` による明示的な
//! 列の入れ替えは行わない（区切り線は `display: none` で grid の並びから
//! 除外されるため、残る 2 要素は grid の自動配置により DOM 順のまま
//! 1 列目・2 列目へ収まり、常に「DOM 順＝視覚順（左→右）」になる）。
//! 64rem 未満では両形とも `flex-direction: column` で DOM 順のまま縦へ
//! 積まれるため、形 A は「フォーム → 区切り線 → 連絡先情報」、形 B は
//! 「連絡先情報 → 区切り線 → フォーム」の順で表示される（形によって
//! 狭幅での積み順が異なる。以前の実装は狭幅の積み順を両形で揃えるために
//! lg 以上でのみ `grid-column`/`grid-row` を上書きしていたが、この上書きは
//! 見た目の左右（フォームが左）と DOM 順（連絡先情報が先）を食い違わせ、
//! 読み上げ順・Tab 順が視覚上の左→右と一致しない不具合
//! （WCAG 2.4.3 Focus Order 相当）を生んでいたため撤去した）。
//!
//! # 見出しレベル
//!
//! ページ側が `## Demo` として `h2` を出すため、各形の主見出しは
//! [`fandhe_frontend_pre_styled_ui::heading::HeadingLevel::H3`] を使う
//! （既存 block と同じ判断）。形 B の 3 グループ（拠点・メール・SNS）の
//! 小見出しは `heading` にすると見出しレベルが揺れるため、
//! `contact_info_columns` と同じ判断で `text`（`Sm`/`Medium`）へ意図的に
//! 畳む。
//!
//! # 表示バリアントの表現方法
//!
//! カラムの左右（`form-start`/`form-end`）は `pre-styled-ui`・本クレートの
//! いずれにも新しい列挙型を増やさず、block ローカルの
//! `data-blocks-contact-split-form-info-variant` 属性の値のみで表す
//! （`contact_info_columns` と同じ設計思想）。
//!
//! # `text` の名前衝突
//!
//! `fandhe_frontend_pre_styled_ui::text::text` と `fandhe_frontend_core::text`
//! が同名のため、styled 側を `styled_text` として取り込む（`crate::blocks`
//! 内の他 block と同じ回避方法）。
//!
//! # `drop_class_attr` と CSS フックの選び方
//!
//! `heading`/`text`/`badge`/`field::root`/`input`/`textarea`/
//! `checkbox::root`/`button`/`icon`/`separator`/`link::root` はいずれも
//! `drop_class_attr`（または同型の固定属性マージ）により呼び出し側
//! `attrs` の `class` を黙って除去する契約を持つため、Demo 固有のスタイル
//! フックは `data-blocks-contact-split-form-info-*` 属性で渡し、
//! [`LAYOUT_CSS`] 側も同じ属性セレクタで対応する。素の `div`/`ul`/`li` には
//! `class` がそのまま効くため、レイアウトは従来どおり
//! `.blocks-contact-split-form-info-*` クラスセレクタを使う。ルートの
//! class（`blocks-contact-split-form-info-stack`）は [`Block::demo_class`]
//! （`blocks-contact-split-form-info`）と意図的に別名にする（既存 block と
//! 同じ Bugbot 教訓の回避）。
//!
//! # ブレークポイント（lg=64rem をリテラル直書きする理由）
//!
//! テーマの breakpoint トークンは `@media` 条件式の中では解決できない
//! （CSS custom property は宣言側でのみ有効）ため、
//! [`fandhe_frontend_pre_styled_ui::recipe::Breakpoint::Lg`]（1024px =
//! 64rem）と一致するリテラル値を [`LAYOUT_CSS`] へ直書きする（
//! `contact_info_columns` と同じ判断）。
//!
//! # フォームの id/name の一意性
//!
//! [`fandhe_frontend_pre_styled_ui::input::FieldProps::id`] から
//! `-control`/`-label` の id が決まるため、形ごとに接頭辞を変える
//! （`blocks-contact-split-form-info-a-*`/`-b-*`）。checkbox の
//! `hidden_input` の `name` も形ごとに変える。`aria-describedby` 等が
//! 存在しない id を指さないよう、両形とも `has_helper_text: false` に
//! 固定する。
//!
//! # 連絡先の値とリンクの方針
//!
//! 電話は北米の架空番号用予約域 `555-01xx`、メールは RFC 2606 の予約
//! ドメイン `example.com` を使う（[`fandhe_frontend_core::url::is_safe_url`]
//! は `tel:`/`mailto:` の両スキームを許可する）。住所・拠点・SNS は
//! [`REPO`]（固定外部 URL、`external: true`）へ遷移させる。`href="#"` の
//! 死リンクは使わない。
//!
//! # アイコンは自作の単純図形
//!
//! 電話・メール・住所・SNS の各アイコンは `contact_image_info::geo_icon`・
//! `contact_info_columns::geo_icon` と同型の私有ヘルパ（`path` に
//! `fill="none"`・`stroke="currentColor"`・`stroke-width="2"`・丸い
//! cap/join）を本ファイル内へ複製する（private 関数のため共有できない）。
//! SNS アイコンは実在ブランドのグリフではなく抽象図形（3 つの円弧）に
//! する。
//!
//! # `<form>` を持たない・データ取得/送信を行わない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である（`docs/policy/intentional-non-adoption.md` §3.25）。ボタンは
//! すべて `button::button` の既定 `type="button"` のまま用いる。checkbox は
//! [`fandhe_frontend_pre_styled_ui::checkbox::CheckedState::Unchecked`]
//! 固定の静的表示であり、docs サイトは JS ハイドレーションを行わない設計
//! （CLAUDE.md）のため状態遷移は扱わない。文言・連絡先はすべて架空のもの
//! （実企業名・実クレデンシャル・PII を含まない）。
//!
//! # `id` の重複回避
//!
//! `crate::blocks` モジュール doc「宙に浮いた ARIA 参照・id 重複を避ける」
//! 契約に従い、checkbox には `id` を一切付与しない（`checkbox::root` 自体が
//! `<label>` のため `for`/`id` の明示連結が不要、`contact_dialog_form` の
//! ボタンと同じ判断）。フィールドの id は形ごとの接頭辞で一意にする。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, el, li, text, ul, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::checkbox::{self, CheckboxProps};
use fandhe_frontend_pre_styled_ui::field::{self, FieldOrientation, FieldRootProps};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::input::{self, FieldIds, FieldProps, InputProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::separator::{separator, SeparatorProps};
use fandhe_frontend_pre_styled_ui::text::{
    self as styled_text, TextProps, TextSize, TextVariant, TextWeight,
};
use fandhe_frontend_pre_styled_ui::textarea::{self, TextareaProps};
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// 住所・拠点・SNS リンクの固定外部 URL（モジュール doc「連絡先の値と
/// リンクの方針」節参照）。実在の地図サービス・SNS へは接続しない。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// 自作の幾何アイコン（線画。モジュール doc「アイコンは自作の単純図形」
/// 節参照）。`path` へ `fill="none"` + `stroke="currentColor"` を明示し、
/// `icon` の `<svg>` 側が固定で持つ `fill="currentColor"`（塗り面）を
/// 上書きして線画（ストローク）として描画する。
fn geo_icon(path_d: &'static str) -> Node {
    icon(
        &IconProps::default(),
        vec![],
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

/// 電話アイコン（角丸長方形の端末 + 上端の短い線）。
fn phone_icon() -> Node {
    geo_icon("M6 4h6v2H8v12h4v2H6z M9 6h1 M4 10c0 6 4 10 10 10")
}

/// メールアイコン（封筒 + V 字の折り返し線）。
fn mail_icon() -> Node {
    geo_icon("M3 6h18v12H3z M3 7l9 6 9-6")
}

/// 住所アイコン（ピン: 円 + 下向きの雫形）。
fn address_icon() -> Node {
    geo_icon(
        "M12 21s-7-6.5-7-11a7 7 0 0 1 14 0c0 4.5-7 11-7 11z M12 12a2 2 0 1 0 0-4 2 2 0 0 0 0 4z",
    )
}

/// SNS の抽象図形アイコン（実在ブランドのグリフを模さない、3 つの円弧）。
fn sns_icon() -> Node {
    geo_icon("M12 3a9 9 0 100 18 9 9 0 000-18z M8 9a4 4 0 018 0 M8 15a4 4 0 008 0")
}

/// 連絡先リンク 1 行（アイコン + ラベル、`contact_image_info::contact_link`
/// と同型）。
fn contact_link(href: &'static str, external: bool, glyph: Node, label: &'static str) -> Node {
    li(
        vec![],
        vec![link::root(
            href,
            &LinkProps {
                external,
                ..LinkProps::default()
            },
            vec![("data-blocks-contact-split-form-info-item", "")],
            vec![glyph, text(label)],
        )],
    )
}

/// 連絡先リンクの一覧（`ul`、形 A の 3 件・形 B の各グループで共用）。
fn contact_list(items: Vec<Node>) -> Node {
    ul(
        vec![("class", "blocks-contact-split-form-info-list")],
        items,
    )
}

/// 形 B のグループ 1 件分（小見出し `text`（Sm/Medium）+ 連絡先リンク一覧）。
/// 見出しレベルの揺れを避けるため `heading` ではなく `text` に畳む
/// （モジュール doc「見出しレベル」節、`contact_info_columns` と同じ判断）。
fn info_group(label: &'static str, items: Vec<Node>) -> Node {
    div(
        vec![("class", "blocks-contact-split-form-info-group")],
        vec![
            styled_text::text(
                &TextProps {
                    size: TextSize::Sm,
                    weight: TextWeight::Medium,
                    ..TextProps::default()
                },
                vec![("data-blocks-contact-split-form-info-group-label", "")],
                vec![text(label)],
            ),
            contact_list(items),
        ],
    )
}

/// 形 A の連絡先情報側（eyebrow badge + 見出し + 説明 + 連絡先リンク 3 件）。
fn info_form_start() -> Node {
    div(
        vec![
            ("class", "blocks-contact-split-form-info-info"),
            ("data-blocks-contact-split-form-info-info", ""),
        ],
        vec![
            badge::badge(
                &BadgeProps::default(),
                vec![("data-blocks-contact-split-form-info-eyebrow", "")],
                vec![text("お問い合わせ")],
            ),
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    ..HeadingProps::default()
                },
                vec![],
                vec![text("お気軽にご相談ください")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(
                    "フォームまたは下記の連絡先から直接お問い合わせいただけます。",
                )],
            ),
            contact_list(vec![
                contact_link("tel:+12025550100", false, phone_icon(), "+1 202-555-0100"),
                contact_link(
                    "mailto:hello@example.com",
                    false,
                    mail_icon(),
                    "hello@example.com",
                ),
                contact_link(REPO, true, address_icon(), "架空通り 1-2-3、サンプル区"),
            ]),
        ],
    )
}

/// 形 B の連絡先情報側（見出し + 説明 + 拠点/メール/SNS の 3 グループ、
/// グループ間は区切り線、集約元 R0445 の構造）。
fn info_form_end() -> Node {
    div(
        vec![
            ("class", "blocks-contact-split-form-info-info"),
            ("data-blocks-contact-split-form-info-info", ""),
        ],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    ..HeadingProps::default()
                },
                vec![],
                vec![text("窓口・拠点のご案内")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(
                    "拠点・メール・SNS のいずれからもお問い合わせいただけます。",
                )],
            ),
            div(
                vec![("class", "blocks-contact-split-form-info-groups")],
                vec![
                    info_group(
                        "拠点",
                        vec![
                            contact_link(REPO, true, address_icon(), "東京本社"),
                            contact_link(REPO, true, address_icon(), "大阪支社"),
                        ],
                    ),
                    separator(&SeparatorProps::default(), vec![]),
                    info_group(
                        "メール",
                        vec![
                            contact_link(
                                "mailto:sales@example.com",
                                false,
                                mail_icon(),
                                "sales@example.com",
                            ),
                            contact_link(
                                "mailto:support@example.com",
                                false,
                                mail_icon(),
                                "support@example.com",
                            ),
                        ],
                    ),
                    separator(&SeparatorProps::default(), vec![]),
                    info_group(
                        "SNS",
                        vec![
                            contact_link(REPO, true, sns_icon(), "コミュニティ"),
                            contact_link(REPO, true, sns_icon(), "更新情報"),
                            contact_link(REPO, true, sns_icon(), "フォーラム"),
                        ],
                    ),
                ],
            ),
        ],
    )
}

/// フォーム領域 1 件分の 1 フィールド（`field::root` + `label` + 入力欄）。
/// `id` は呼び出し側が形ごとに一意なリテラルを渡す（モジュール doc
/// 「フォームの id/name の一意性」節、`format!`/`Box::leak` によるヒープ
/// 確保を避けるための `&'static str` 直接指定）。
fn text_field(id: &'static str, label: &'static str, kind: FieldKind, required: bool) -> Node {
    let field_props = FieldProps {
        id,
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required,
        readonly: false,
        has_helper_text: false,
    };
    let orientation = FieldRootProps {
        orientation: FieldOrientation::Vertical,
    };
    let control = match kind {
        FieldKind::Text { autocomplete } => input::input(
            &InputProps::default(),
            &field_props,
            vec![("type", "text"), ("autocomplete", autocomplete)],
        ),
        FieldKind::Email => input::input(
            &InputProps::default(),
            &field_props,
            vec![("type", "email"), ("autocomplete", "email")],
        ),
        FieldKind::Message => textarea::textarea(
            &TextareaProps::default(),
            &field_props,
            false,
            vec![
                ("rows", "4"),
                ("data-blocks-contact-split-form-info-message", ""),
            ],
            vec![],
        ),
    };
    field::root(
        &orientation,
        &field_props,
        vec![("data-blocks-contact-split-form-info-field", "")],
        vec![
            field::label(&field_props, vec![], vec![text(label)]),
            control,
        ],
    )
}

/// [`text_field`] のコントロール種別（`clippy::too_many_arguments` 回避の
/// ための小さな列挙型、この block 内限定のローカル定義）。
enum FieldKind {
    Text { autocomplete: &'static str },
    Email,
    Message,
}

/// [`form`]/[`agree_checkbox`] が形 A・形 B を区別するための block ローカル
/// 列挙型（モジュール doc「フォームの id/name の一意性」節）。id/name の
/// 接頭辞を `&'static str` の合成（`format!`/`Box::leak`）に頼らず、形ごとの
/// 全フィールド分の id リテラルをこの型の各アームへ直接書き下す。データを
/// 持たない unit variant のみのため `Copy` にし、`form` 内で複数回 `match`
/// できるようにする。
#[derive(Clone, Copy, PartialEq, Eq)]
enum FormVariant {
    /// 形 A（`form-start`）。会社名フィールドを含む。
    A,
    /// 形 B（`form-end`）。会社名フィールドを含まない。
    B,
}

/// 同意チェックボックス（未チェック固定の静的表示、モジュール doc
/// 「`<form>` を持たない」節参照）。
fn agree_checkbox(variant: FormVariant) -> Node {
    let props = CheckboxProps::default();
    let name = match variant {
        FormVariant::A => "blocks-contact-split-form-info-a-agree",
        FormVariant::B => "blocks-contact-split-form-info-b-agree",
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
            checkbox::label(&props, vec![], vec![text("プライバシーポリシーに同意する")]),
        ],
    )
}

/// フォーム領域（氏名・メール・（形 A のみ会社名）・本文 + 同意チェック +
/// 送信ボタン。形ごとに [`FormVariant`] で id/name を分離する、モジュール
/// doc「フォームの id/name の一意性」節）。
fn form(variant: FormVariant) -> Node {
    let mut fields = vec![text_field(
        match variant {
            FormVariant::A => "blocks-contact-split-form-info-a-name",
            FormVariant::B => "blocks-contact-split-form-info-b-name",
        },
        "氏名",
        FieldKind::Text {
            autocomplete: "name",
        },
        true,
    )];
    if matches!(variant, FormVariant::A) {
        fields.push(text_field(
            "blocks-contact-split-form-info-a-company",
            "会社名（任意）",
            FieldKind::Text {
                autocomplete: "organization",
            },
            false,
        ));
    }
    fields.push(text_field(
        match variant {
            FormVariant::A => "blocks-contact-split-form-info-a-email",
            FormVariant::B => "blocks-contact-split-form-info-b-email",
        },
        "メールアドレス",
        FieldKind::Email,
        true,
    ));
    fields.push(text_field(
        match variant {
            FormVariant::A => "blocks-contact-split-form-info-a-message",
            FormVariant::B => "blocks-contact-split-form-info-b-message",
        },
        "お問い合わせ内容",
        FieldKind::Message,
        true,
    ));

    div(
        vec![
            ("class", "blocks-contact-split-form-info-form"),
            ("data-blocks-contact-split-form-info-form", ""),
        ],
        vec![
            field::group(vec![], fields),
            agree_checkbox(variant),
            button::button(
                &ButtonProps::default(),
                vec![("data-blocks-contact-split-form-info-submit", "")],
                vec![text("送信")],
            ),
        ],
    )
}

/// 各形の直前に置く短い形ラベル（`content_split_image::variant_label` と
/// 同型）。
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

/// 1 つの形のレイアウト骨格。`first`/`second` は lg 以上での視覚上の
/// 左→右の順（モジュール doc「DOM 順と視覚順」節）で渡す（呼び出し側が
/// 形ごとに正しい順で渡す契約であり、DOM 順は常に `first, 区切り線,
/// second` になる）。`variant_attr` は `"form-start"`/`"form-end"` の
/// いずれか（[`FormVariant`] とは別の、CSS フック用の文字列値）。
fn variant_layout(variant_attr: &'static str, first: Node, second: Node) -> Node {
    div(
        vec![
            ("class", "blocks-contact-split-form-info-layout"),
            ("data-blocks-contact-split-form-info-variant", variant_attr),
        ],
        vec![
            first,
            separator(
                &SeparatorProps::default(),
                vec![("data-blocks-contact-split-form-info-divider", "")],
            ),
            second,
        ],
    )
}

/// `contact-split-form-info` の Demo 本体（形 A・形 B を縦に並記する）。
/// 呼び出しごとに同一の `Node` を返す純関数（モジュール doc「1 つの Demo に
/// 2 つの形を縦に並べる」節）。形 A は視覚上フォームが左のため
/// `variant_layout` へ「フォーム, 連絡先情報」の順で渡し、形 B は連絡先
/// 情報が左のため「連絡先情報, フォーム」の順で渡す（モジュール doc
/// 「DOM 順と視覚順」節、DOM 順＝視覚順を維持する契約）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-contact-split-form-info-stack")],
        vec![
            variant_label("左にフォーム・右に連絡先情報（R0441 基準形）"),
            variant_layout("form-start", form(FormVariant::A), info_form_start()),
            variant_label("左に連絡先情報（3 グループ）・右にフォーム（R0443/R0445）"),
            variant_layout("form-end", info_form_end(), form(FormVariant::B)),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/contact-split-form-info/",
    title: "contact-split-form-info",
    category: BlockCategory::Contact,
    rust_source: "crates/docs-site/src/blocks/marketing/contact/contact_split_form_info.rs",
    demo_class: "blocks-contact-split-form-info",
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
            label: "Badge",
            path: "/themes/badge/",
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
            label: "Checkbox",
            path: "/themes/checkbox/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Icon",
            path: "/themes/icon/",
        },
        Part {
            label: "Separator",
            path: "/themes/separator/",
        },
        Part {
            label: "Link",
            path: "/themes/link/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `contact_split_form_info` 固有のレイアウト規則
/// （`crate::blocks::LAYOUT_CSS` doc「block 固有 CSS の置き場」節、
/// `contact_info_columns` と同型）。
///
/// セレクタは `.blocks-contact-split-form-info-*` と
/// `[data-blocks-contact-split-form-info-*]` のみを用い、他 block や部品の
/// 素のセレクタへ影響させない。
const LAYOUT_CSS: &str = "\
.blocks-contact-split-form-info-stack {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-12);\n}\n\
.blocks-contact-split-form-info-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n}\n\
[data-blocks-contact-split-form-info-info] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n  min-width: 0;\n}\n\
[data-blocks-contact-split-form-info-form] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n  min-width: 0;\n}\n\
.blocks-contact-split-form-info-groups {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-contact-split-form-info-group {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n}\n\
[data-blocks-contact-split-form-info-group-label] {\n  color: var(--fandhe-color-accent);\n  text-transform: uppercase;\n  letter-spacing: 0.05em;\n}\n\
.blocks-contact-split-form-info-list {\n  list-style: none;\n  margin: 0;\n  padding: 0;\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n}\n\
[data-scope=\"link\"][data-part=\"root\"][data-blocks-contact-split-form-info-item] {\n  display: inline-flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n}\n\
[data-blocks-contact-split-form-info-field] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n}\n\
[data-scope=\"field\"][data-part=\"textarea\"][data-blocks-contact-split-form-info-message] {\n  min-height: 8rem;\n}\n\
@media (min-width: 64rem) {\n  \
[data-scope=\"separator\"][data-blocks-contact-split-form-info-divider] {\n    display: none;\n  }\n\
  .blocks-contact-split-form-info-layout {\n    display: grid;\n    grid-template-columns: repeat(2, minmax(0, 1fr));\n    gap: var(--fandhe-space-12);\n    align-items: start;\n  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS, REPO};
    use fandhe_frontend_core::render;

    /// Demo が使用部品（11 部品）の anatomy をすべて実際に出力していること
    /// を固定する。
    #[test]
    fn demo_composes_expected_parts() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"badge\"",
            "data-scope=\"field\" data-part=\"root\"",
            "data-scope=\"field\" data-part=\"input\"",
            "data-scope=\"field\" data-part=\"textarea\"",
            "data-scope=\"checkbox\"",
            "data-scope=\"button\"",
            "data-scope=\"icon\"",
            "data-scope=\"separator\"",
            "data-scope=\"link\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert!(html.contains("href=\"tel:"));
        assert!(html.contains("href=\"mailto:"));
        assert_eq!(html.matches(&format!("href=\"{REPO}\"")).count(), 6);
        assert_eq!(html.matches("type=\"button\"").count(), 2);
        assert_eq!(
            html.matches("data-scope=\"separator\"").count(),
            4,
            "demo should render exactly 4 separators (2 outer dividers + 2 group separators in form B)"
        );
        assert_eq!(
            html.matches("data-blocks-contact-split-form-info-variant=\"form-start\"")
                .count(),
            1
        );
        assert_eq!(
            html.matches("data-blocks-contact-split-form-info-variant=\"form-end\"")
                .count(),
            1
        );
    }

    /// 形 A（`form-start`）・形 B（`form-end`）のいずれも、lg 以上での
    /// 視覚上の左→右の順と DOM 順が一致することを固定する（モジュール
    /// doc「DOM 順と視覚順」節の回帰ガード。P1 修正: 以前は `form-start`
    /// のみ視覚順〔フォームが左〕と DOM 順〔連絡先情報が先〕が食い違い、
    /// 読み上げ順・Tab 順が見た目と一致しない不具合〔WCAG 2.4.3 相当〕が
    /// あった）。形 A は「フォーム → 連絡先情報」、形 B は「連絡先情報 →
    /// フォーム」の順で出現するはずである。
    #[test]
    fn demo_dom_order_matches_visual_order_for_both_variants() {
        let html = render(&demo());
        let form_positions: Vec<usize> = html
            .match_indices("data-blocks-contact-split-form-info-form")
            .map(|(idx, _)| idx)
            .collect();
        let info_positions: Vec<usize> = html
            .match_indices("data-blocks-contact-split-form-info-info")
            .map(|(idx, _)| idx)
            .collect();
        assert_eq!(
            form_positions.len(),
            2,
            "form 内部フックは form-start・form-end の 2 回出現するはず"
        );
        assert_eq!(
            info_positions.len(),
            2,
            "info 内部フックは form-start・form-end の 2 回出現するはず"
        );
        // 形 A（先に出現するレイアウト）: フォームが先（視覚上左のため）。
        assert!(
            form_positions[0] < info_positions[0],
            "形 A は DOM 順でフォームが連絡先情報より先であるべき（視覚上フォームが左のため）"
        );
        // 形 B（後に出現するレイアウト）: 連絡先情報が先（視覚上左のため）。
        assert!(
            info_positions[1] < form_positions[1],
            "形 B は DOM 順で連絡先情報がフォームより先であるべき（視覚上連絡先情報が左のため）"
        );
    }

    /// 非対話・XSS 回帰の不変条件（`crate::blocks` モジュール doc）を固定
    /// する。
    #[test]
    fn demo_has_no_form_or_unsafe_output() {
        let html = render(&demo());
        for absent in [
            "<form",
            "type=\"submit\"",
            "action=",
            "href=\"#\"",
            "src=\"data:",
            " checked",
        ] {
            assert!(
                !html.contains(absent),
                "demo output should never contain {absent}"
            );
        }
        // `form=\"` は avoid リストに含めない: `data-blocks-contact-split-
        // form-info-form` 等の自前フック属性名が偶然 `form="` を部分文字列
        // として含むため（`<input form="...">` のような外部 form 関連付け
        // 属性は別途使っていない。`<form` の非存在チェックで実体はカバー
        // 済み）。同じ理由で `"checked"` ではなく `" checked"`（先頭空白
        // 付き）を使う: `data-state="unchecked"` が部分文字列 `"checked"`
        // を含むため。
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

    /// [`LAYOUT_CSS`] が想定するブレークポイント・2 列 grid・divider 非表示を
    /// 持ち、`<` を含まないことを固定する（REQ-1: `</style>` によるスタイル
    /// 脱出を防ぐ）。
    #[test]
    fn layout_css_declares_breakpoint_and_grid() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("@media (min-width: 64rem)"));
        assert!(LAYOUT_CSS.contains("repeat(2, minmax(0, 1fr))"));
        assert!(LAYOUT_CSS.contains(
            "[data-scope=\"separator\"][data-blocks-contact-split-form-info-divider] {\n    display: none;\n  }"
        ));
    }

    /// [`LAYOUT_CSS`] が `form-start`/`form-end` の列位置を
    /// `grid-column`/`grid-row` で明示的に上書きしないことを固定する
    /// （モジュール doc「DOM 順と視覚順」節の回帰ガード。かつて `form-start`
    /// のみをこの上書きで視覚上左へ移動しており、DOM 順〔連絡先情報が先〕
    /// と視覚順〔フォームが左〕が食い違って Tab 順が見た目と一致しない
    /// 不具合〔WCAG 2.4.3 相当〕があった。列位置は [`demo`] 側が
    /// `variant_layout` へ渡す `first`/`second` の順序〔grid 自動配置〕
    /// のみで決まる契約を、この文字列非存在で固定する）。
    #[test]
    fn layout_css_does_not_override_grid_column_placement() {
        assert!(!LAYOUT_CSS.contains("grid-column"));
        assert!(!LAYOUT_CSS.contains("grid-row"));
    }

    /// ルート grid class（`demo_class` とは別名）が `demo()` の出力へ実際に
    /// 現れること（既存 block と同じ Bugbot 教訓の固定）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-contact-split-form-info-stack\""));
        assert_ne!(
            super::BLOCK.demo_class,
            "blocks-contact-split-form-info-stack"
        );
    }
}
