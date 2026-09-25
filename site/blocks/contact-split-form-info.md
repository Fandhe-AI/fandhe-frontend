# contact-split-form-info

`fandhe-frontend-pre-styled-ui` の `heading` / `text` / `badge` / `field` /
`input` / `textarea` / `checkbox` / `button` / `icon` / `separator` /
`link` の 11 部品を合成した、問い合わせフォーム + 連絡先情報の 2 カラム
ブロックです。Blocks セクションは新規部品を追加するものではなく、既存の
Themes/Primitives 部品を組み合わせた実例集であることに注意してください
（主参照は対応表 ID R0441、集約元は R0443・R0445・R0855 の 3 件です。
取り込んだのは領域配置・部品構成の構造だけで、文言・配色・装飾・アイコン
形状は持ち込んでいません。出典の固有名・ファイル名は記載しません）。

本 Demo は 1 つの Demo の中に 2 つの形を縦に並べています。

- **形 A**（主参照 R0441）: 幅 `64rem`（`lg`）以上では左にフォーム、右に
  連絡先情報（お問い合わせバッジ + 見出し + 説明文 + 電話・メール・住所の
  アイコン付きリンク 3 件）を配置した 2 カラムになります。
- **形 B**（集約元 R0443 の左右反転 + R0445 の 3 グループ）: `lg` 以上では
  左に連絡先情報、右にフォームを配置します。連絡先情報側は拠点・メール・
  SNS の 3 グループに分かれ、グループ間を区切り線で分けています。

どちらの形も `64rem` 未満の狭い幅では、連絡先情報 → 区切り線 → フォームの
順に縦へ積まれます（DOM の順序は常にこの順で固定です）。形 A は `lg` 以上
のとき、この DOM の順序と見た目の順序（フォームが左）が意図的にずれます
（読み上げ順・Tab 順は常に DOM の順序で一貫させるため）。

本 Demo は静的な表示例であり、`<form>` 要素を一切持たず、データの取得・
送信・状態管理を行いません。ボタンはすべて `type="button"` です。同意
チェックボックスは未チェック固定の静的表示です。電話・メールのリンクは
`tel:`/`mailto:` の実プロトコルリンクですが、番号・アドレスはいずれも
架空値（電話は北米の架空番号用予約域 555-01xx、メールは RFC 2606 の予約
ドメイン `example.com`）です。住所・拠点・SNS リンクの遷移先は本リポジトリ
への固定外部 URL であり、実在の地図サービス・SNS へは接続しません。SNS の
アイコンは実在ブランドのグリフを模さない抽象図形です。文言はすべて独自に
書いた架空のものであり、実企業名・実クレデンシャル・PII を含みません。

## Rust コード

```rust
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
                contact_link("tel:+15550100", false, phone_icon(), "+1 555-0100"),
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

/// 1 つの形のレイアウト骨格（連絡先情報 → 区切り線 → フォームの DOM 順、
/// モジュール doc「DOM 順と視覚順」節）。`variant_attr` は
/// `"form-start"`/`"form-end"` のいずれか（[`FormVariant`] とは別の、CSS
/// フック用の文字列値）。
fn variant_layout(variant_attr: &'static str, info: Node, form: Node) -> Node {
    div(
        vec![
            ("class", "blocks-contact-split-form-info-layout"),
            ("data-blocks-contact-split-form-info-variant", variant_attr),
        ],
        vec![
            info,
            separator(
                &SeparatorProps::default(),
                vec![("data-blocks-contact-split-form-info-divider", "")],
            ),
            form,
        ],
    )
}

/// `contact-split-form-info` の Demo 本体（形 A・形 B を縦に並記する）。
/// 呼び出しごとに同一の `Node` を返す純関数（モジュール doc「1 つの Demo に
/// 2 つの形を縦に並べる」節）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-contact-split-form-info-stack")],
        vec![
            variant_label("左にフォーム・右に連絡先情報（R0441 基準形）"),
            variant_layout("form-start", info_form_start(), form(FormVariant::A)),
            variant_label("左に連絡先情報（3 グループ）・右にフォーム（R0443/R0445）"),
            variant_layout("form-end", info_form_end(), form(FormVariant::B)),
        ],
    )
}
```

## 原案差分メモ

参照（対応表 ID R0441 が主参照、R0443・R0445・R0855 が集約元。出典の固有名・
ファイル名は記載しません）からの意図的な差分は次のとおりです。

- R0441 の構造を形 A として取り込みました。左にフォーム、右に連絡先情報
  （電話・メール・住所のアイコン付きリンク 3 件）という領域配置のみを
  参照し、文言・配色・装飾・アイコン形状は独自に書き直しました。
- R0443 の左右反転を形 B として取り込みました。新しい列挙型は追加せず、
  block ローカルの `data-blocks-contact-split-form-info-variant` 属性の値
  （`"form-start"`/`"form-end"`）だけで表現しています。
- R0445 の連絡先側 3 グループ構成を形 B の連絡先情報側として取り込みました。
  拠点・メール・SNS の 3 グループに区切り線を挟んでいますが、グループの
  小見出しは見出しレベルの揺れを避けるため `heading` ではなく `text`
  （`Sm`/`Medium`）にしています。
- R0855 の連絡先を `dl` で表示する構成・格子模様の背景は取り込んでいません。
  `dl` 要素は本 Demo の使用部品（11 部品）に含まれず、格子模様の背景は
  装飾であり構造ではないためです。
- 見出しは参照側相当ではなく `h3`（ページ側が `## Demo` として `h2` を出す
  ため）にしています。
- 連絡先リンクの遷移先は、参照側の死リンク相当を `tel:`/`mailto:` の実
  プロトコルリンク（番号・アドレスは架空値）と、住所・拠点・SNS 用の固定
  外部 URL（本リポジトリ）へ置き換えました。`href="#"` の死リンクは使い
  ません。
- SNS アイコンは実在ブランドのグリフを模さない自作の抽象図形（円弧）に
  しています。電話・メール・住所のアイコンも lucide 等の既存アイコン
  セットの path を複製しない自作の幾何アイコン（線画）です。
- フォームは氏名・（形 A のみ会社名）・メールアドレス・お問い合わせ内容と
  同意チェックボックス・送信ボタンのみの最小構成にしています。`<form>`
  要素は持たず、送信先・検証・状態機械は一切実装していません（構造は
  `field`/`input`/`textarea`/`checkbox`/`button` の既存部品のみで組み、
  ボタンは `type="button"` 固定です）。
- 文言（見出し・説明文・連絡先ラベル・グループ名）はすべて独自に書き
  直しました。
- 配色・余白・角丸は既存のテーマトークンに従っています。
- ブレークポイントは `64rem`（`lg`）に固定しています。
- `id`/`name` は形ごとに接頭辞（`a`/`b`）で分離し、重複しないようにして
  います。checkbox 自体には `id` を付与しません（`checkbox::root` が
  `<label>` のため `for`/`id` の明示連結が不要なため）。
