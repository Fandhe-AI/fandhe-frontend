# onboarding-centered-steps

`fandhe-frontend-pre-styled-ui` の `steps` / `field` / `input` /
`native-select` / `checkbox-card` / `radio-card` / `card` / `avatar` /
`file-upload` / `checkbox` / `button` / `heading` 部品を合成した、中央寄せの
ステップ式オンボーディングフローの実例です。Blocks セクションは新規部品を
追加するものではなく、既存の Themes/Primitives 部品を組み合わせた実例集で
あることに注意してください（主参照は対応表 ID R0170、集約元は R0171・
R0172・R0179・R0180・R0181・R0182・R0183。出典の固有名・ファイル名は記載
しません）。

上部にロゴ + 4 段の進捗ステップ、中央カラムに見出し・説明文とステップ固有の
入力欄、下部に「戻る / 次へ」を配置した骨格を共通に、ステップの中身だけを
差し替えた 4 インスタンス（プロフィール入力・興味関心の選択・環境設定・
チーム招待）を縦に並べています。狭い幅ではカード群（興味関心・選択欄・
アプリ案内）が 1〜2 列へ自動的に折り返します。

本 Demo は静的な表示例であり、`<form>` 要素は一切持たず、データの取得・
送信・状態機械を行いません。ステップ番号・ラベルは操作可能な `<button>` に
せず、進捗表示のみの静的構造としています。下部の「戻る / 次へ」・興味関心
の `checkbox-card`・テーマ/利用目的の `radio-card`・招待メール CC の
`checkbox`・写真アップロードの `file-upload` はいずれも無 JS の docs サイト
で押しても何も起きない、または状態が視覚と食い違う操作になるため、ネイティブ
`disabled` で操作不能にした静的表示です。文言・招待リンク・メールアドレスは
すべて独自に書いた架空のものであり、実企業名・実クレデンシャル・PII を
含みません。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::checkbox::{self, CheckboxProps as PlainCheckboxProps};
use fandhe_frontend_pre_styled_ui::checkbox_card::{self, CheckboxProps, CheckedState};
use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::steps::Steps;
use fandhe_frontend_pre_styled_ui::field::{
    self, FieldIds, FieldOrientation, FieldProps, FieldRootProps,
};
use fandhe_frontend_pre_styled_ui::file_upload::{self, FileUploadProps};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::input::{self, InputProps};
use fandhe_frontend_pre_styled_ui::native_select::{self, NativeSelectProps};
use fandhe_frontend_pre_styled_ui::radio_card::{self, Orientation as RadioCardOrientation};
use fandhe_frontend_pre_styled_ui::steps;
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// 一意な id を組み立てる（`blocks-onboarding-centered-steps-<instance>-`
/// 接頭辞を共通化し、フィールド追加時の綴り間違いを防ぐ）。
fn field_id(instance: &str, suffix: &str) -> String {
    format!("blocks-onboarding-centered-steps-{instance}-{suffix}")
}

/// ロゴ（`avatar` の角丸 shape。`role="img"` + `aria-label` でアクセシブル
/// ネームを明示する、`page_heading_avatar.rs::profile_avatar` と同型）。
fn logo() -> Node {
    let company = dummy_assets::COMPANY_NAMES[0];
    avatar::root(
        &AvatarProps {
            size: Size::Lg,
            ..AvatarProps::default()
        },
        vec![
            ("role", "img"),
            ("aria-label", company),
            ("class", "blocks-onboarding-centered-steps-logo"),
        ],
        vec![
            avatar::image(ImageStatus::Loaded, dummy_assets::LOGO_SRC, "", vec![]),
            avatar::fallback(ImageStatus::Loaded, vec![], vec![text("\u{25a0}")]),
        ],
    )
}

/// steps item 1 件（番号 + ラベル）。トリガーボタンにしない理由はモジュール
/// doc 「steps 上部ナビを `trigger` ボタンにしない理由」節参照。
fn step_indicator_and_label(s: &Steps, index: usize, label: &str) -> Node {
    let item_attrs = if index == s.step() {
        vec![("aria-current", "step")]
    } else {
        vec![]
    };
    let mut children = vec![
        steps::indicator(s, index, vec![], vec![text((index + 1).to_string())]),
        el(
            "span",
            vec![("class", "blocks-onboarding-centered-steps-step-label")],
            vec![text(label)],
        ),
    ];
    if index + 1 < s.count() {
        children.push(steps::separator(s, index, vec![], vec![]));
    }
    steps::item(s, index, item_attrs, children)
}

/// 上部（ロゴ + 進捗ステップ）。
fn top_bar(s: &Steps) -> Node {
    let labels = ["プロフィール", "興味・関心", "環境設定", "招待"];
    let items: Vec<Node> = labels
        .iter()
        .enumerate()
        .map(|(index, label)| step_indicator_and_label(s, index, label))
        .collect();
    div(
        vec![("class", "blocks-onboarding-centered-steps-top")],
        vec![
            logo(),
            steps::root(
                Size::Md,
                ColorPalette::Accent,
                s,
                vec![],
                vec![steps::list(s, vec![], items)],
            ),
        ],
    )
}

/// 下部操作（戻る / 次へ）。全インスタンスで常にネイティブ disabled にする
/// 理由はモジュール doc 「`prev_trigger`/`next_trigger` を常にネイティブ
/// disabled にする理由」節参照。
fn nav_actions(s: &Steps, next_label: &'static str) -> Node {
    div(
        vec![("class", "blocks-onboarding-centered-steps-actions")],
        vec![
            steps::prev_trigger(
                s,
                vec![("disabled", ""), ("data-disabled", "")],
                vec![text("戻る")],
            ),
            steps::next_trigger(
                s,
                vec![("disabled", ""), ("data-disabled", "")],
                vec![text(next_label)],
            ),
        ],
    )
}

/// 通常フィールド（`field`/`input`）を組み立てる。
fn text_field<'a>(id: &'a str, label_text: &'a str, input_type: &'a str, value: &'a str) -> Node {
    let props = FieldProps {
        id,
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: false,
    };
    field::root(
        &FieldRootProps {
            orientation: FieldOrientation::Vertical,
        },
        &props,
        vec![("data-blocks-onboarding-centered-steps-field", "")],
        vec![
            field::label(&props, vec![], vec![text(label_text)]),
            input::input(
                &InputProps::default(),
                &props,
                vec![("type", input_type), ("value", value)],
            ),
        ],
    )
}

/// 読み取り専用フィールド（招待リンク行の `input`）。
fn readonly_field<'a>(id: &'a str, label_text: &'a str, value: &'a str) -> Node {
    let props = FieldProps {
        id,
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: false,
        readonly: true,
        has_helper_text: false,
    };
    field::root(
        &FieldRootProps {
            orientation: FieldOrientation::Vertical,
        },
        &props,
        vec![
            ("data-blocks-onboarding-centered-steps-field", ""),
            ("data-blocks-onboarding-centered-steps-invite-link", ""),
        ],
        vec![
            field::label(&props, vec![], vec![text(label_text)]),
            input::input(
                &InputProps::default(),
                &props,
                vec![("type", "text"), ("value", value)],
            ),
        ],
    )
}

/// **profile**（step 0）: 表示名・役職・写真アップロード。
fn profile_step() -> Node {
    let name_id = field_id("profile", "name");
    let role_id = field_id("profile", "role");
    let role_props = FieldProps {
        id: &role_id,
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: false,
    };
    let role_field = field::root(
        &FieldRootProps {
            orientation: FieldOrientation::Vertical,
        },
        &role_props,
        vec![("data-blocks-onboarding-centered-steps-field", "")],
        vec![
            field::label(&role_props, vec![], vec![text("役職")]),
            native_select::native_select(
                &NativeSelectProps::default(),
                &role_props,
                vec![],
                vec![
                    el(
                        "option",
                        vec![("value", "engineer"), ("selected", "")],
                        vec![text("エンジニア")],
                    ),
                    el(
                        "option",
                        vec![("value", "designer")],
                        vec![text("デザイナー")],
                    ),
                    el(
                        "option",
                        vec![("value", "pm")],
                        vec![text("プロダクトマネージャー")],
                    ),
                ],
            ),
        ],
    );

    let photo_props = FileUploadProps {
        disabled: true,
        ..FileUploadProps::default()
    };
    let photo_upload = file_upload::root(
        Size::Md,
        &photo_props,
        false,
        vec![("data-blocks-onboarding-centered-steps-field", "")],
        vec![
            file_upload::label(&photo_props, vec![], vec![text("写真")]),
            div(
                vec![("class", "blocks-onboarding-centered-steps-photo-row")],
                vec![
                    avatar::root(
                        &AvatarProps {
                            size: Size::Lg,
                            ..AvatarProps::default()
                        },
                        vec![],
                        vec![avatar::fallback(
                            ImageStatus::Loaded,
                            vec![],
                            vec![text(
                                dummy_assets::PERSON_NAMES[0]
                                    .chars()
                                    .take(1)
                                    .collect::<String>(),
                            )],
                        )],
                    ),
                    file_upload::trigger(&photo_props, vec![], vec![text("写真を選択")]),
                    file_upload::hidden_input("image/*", false, &photo_props, vec![("hidden", "")]),
                ],
            ),
        ],
    );

    div(
        vec![("class", "blocks-onboarding-centered-steps-fields")],
        vec![
            text_field(&name_id, "表示名", "text", dummy_assets::PERSON_NAMES[0]),
            role_field,
            photo_upload,
        ],
    )
}

/// 興味関心 checkbox-card 1 件。
fn interest_card(
    name: &'static str,
    label: &'static str,
    description: &'static str,
    checked: bool,
) -> Node {
    let props = CheckboxProps {
        checked: if checked {
            CheckedState::Checked
        } else {
            CheckedState::Unchecked
        },
        disabled: true,
        ..CheckboxProps::default()
    };
    checkbox_card::root(
        Size::Md,
        ColorPalette::Accent,
        &props,
        vec![("data-blocks-onboarding-centered-steps-interest-card", "")],
        vec![
            checkbox_card::hidden_input(&props, name, "on", vec![]),
            checkbox_card::control(
                &props,
                vec![],
                vec![
                    checkbox_card::content(
                        &props,
                        vec![],
                        vec![
                            checkbox_card::label(&props, vec![], vec![text(label)]),
                            checkbox_card::description(&props, vec![], vec![text(description)]),
                        ],
                    ),
                    checkbox_card::indicator(
                        &props,
                        vec![],
                        vec![checkbox_card::indicator_check(&props, vec![], vec![])],
                    ),
                ],
            ),
        ],
    )
}

/// **interests**（step 1）: 興味関心 checkbox-card 6 件（2 件選択済み）。
fn interests_step() -> Node {
    let interests: [(&str, &str, &str, bool); 6] = [
        (
            "blocks-onboarding-centered-steps-interest-product",
            "プロダクト開発",
            "新機能の企画・設計に関わりたい。",
            true,
        ),
        (
            "blocks-onboarding-centered-steps-interest-design",
            "デザイン",
            "UI/UX の改善に興味がある。",
            false,
        ),
        (
            "blocks-onboarding-centered-steps-interest-data",
            "データ分析",
            "利用状況の分析・レポートに関わりたい。",
            true,
        ),
        (
            "blocks-onboarding-centered-steps-interest-marketing",
            "マーケティング",
            "集客・広報の施策に興味がある。",
            false,
        ),
        (
            "blocks-onboarding-centered-steps-interest-support",
            "カスタマーサポート",
            "利用者からの問い合わせ対応に関わりたい。",
            false,
        ),
        (
            "blocks-onboarding-centered-steps-interest-ops",
            "運用・インフラ",
            "安定稼働・監視の仕組みに興味がある。",
            false,
        ),
    ];
    div(
        vec![("class", "blocks-onboarding-centered-steps-cards")],
        interests
            .iter()
            .map(|(name, label, description, checked)| {
                interest_card(name, label, description, *checked)
            })
            .collect(),
    )
}

/// テーマ・利用目的の radio-card 群 1 個分の item。同一グループの `name`
/// を共有しないとネイティブ `<input type="radio">` の排他選択が成立しない
/// ため、`name` を呼び出し側から明示的に受け取る（`value` を誤って
/// `name` に流用しない）。
fn preference_radio_item(
    checked: bool,
    name: &'static str,
    value: &'static str,
    label: &'static str,
) -> Node {
    radio_card::item(
        checked,
        true,
        value,
        vec![],
        vec![
            radio_card::item_hidden_input(checked, true, Some(name), value, vec![]),
            radio_card::item_control(
                checked,
                true,
                vec![],
                vec![
                    radio_card::item_indicator(checked, true, false, vec![]),
                    radio_card::item_content(
                        vec![],
                        vec![radio_card::item_text(vec![], vec![text(label)])],
                    ),
                ],
            ),
        ],
    )
}

/// テーマ・利用目的の radio-card 群（見出し + 2〜3 択、ネイティブ disabled）。
/// `aria-disabled` を明示付与する理由はモジュール doc
/// 「checkbox-card / radio-card / checkbox をネイティブ disabled にする
/// 理由」節参照。
fn preference_radio_group(label_id: String, label_text: &'static str, items: Vec<Node>) -> Node {
    div(
        vec![("class", "blocks-onboarding-centered-steps-radio-group")],
        vec![
            radio_card::label(Some(&label_id), vec![], vec![text(label_text)]),
            radio_card::root(
                Size::Sm,
                ColorPalette::Accent,
                true,
                Some(RadioCardOrientation::Horizontal),
                Some(&label_id),
                vec![("aria-disabled", "true")],
                items,
            ),
        ],
    )
}

/// `field`/`native_select` の選択欄 1 件を組み立てる（`id` を所有した
/// `String` のまま `FieldProps` へ借用させ、`Node` を組み立て終えるまでの間
/// だけ生かす。呼び出し側で `id` を先に確保することで `Box::leak` のような
/// ヒープリークを避ける）。
fn select_field(
    id: String,
    label_text: &'static str,
    options: Vec<(&'static str, &'static str)>,
) -> Node {
    let props = FieldProps {
        id: &id,
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: false,
    };
    field::root(
        &FieldRootProps {
            orientation: FieldOrientation::Vertical,
        },
        &props,
        vec![("data-blocks-onboarding-centered-steps-field", "")],
        vec![
            field::label(&props, vec![], vec![text(label_text)]),
            native_select::native_select(
                &NativeSelectProps::default(),
                &props,
                vec![],
                options
                    .into_iter()
                    .enumerate()
                    .map(|(index, (value, opt_label))| {
                        let mut attrs = vec![("value", value)];
                        if index == 0 {
                            attrs.push(("selected", ""));
                        }
                        el("option", attrs, vec![text(opt_label)])
                    })
                    .collect(),
            ),
        ],
    )
}

/// **preferences**（step 2）: テーマ・利用目的の radio-card + 3 つの選択欄。
fn preferences_step() -> Node {
    let theme_label_id = field_id("preferences", "theme-label");
    let theme_name = "blocks-onboarding-centered-steps-preferences-theme";
    let theme_group = preference_radio_group(
        theme_label_id,
        "テーマ",
        vec![
            preference_radio_item(false, theme_name, "light", "ライト"),
            preference_radio_item(true, theme_name, "dark", "ダーク"),
            preference_radio_item(false, theme_name, "system", "システムに合わせる"),
        ],
    );

    let purpose_label_id = field_id("preferences", "purpose-label");
    let purpose_name = "blocks-onboarding-centered-steps-preferences-purpose";
    let purpose_group = preference_radio_group(
        purpose_label_id,
        "主な利用目的",
        vec![
            preference_radio_item(true, purpose_name, "personal", "個人利用"),
            preference_radio_item(false, purpose_name, "team", "チームでの利用"),
            preference_radio_item(false, purpose_name, "learning", "学習・検証"),
        ],
    );

    let selects = div(
        vec![("class", "blocks-onboarding-centered-steps-selects")],
        vec![
            select_field(
                field_id("preferences", "team-size"),
                "チーム規模",
                vec![
                    ("1-10", "1〜10 名"),
                    ("11-50", "11〜50 名"),
                    ("51+", "51 名以上"),
                ],
            ),
            select_field(
                field_id("preferences", "industry"),
                "業種",
                vec![
                    ("software", "ソフトウェア"),
                    ("retail", "小売"),
                    ("education", "教育"),
                ],
            ),
            select_field(
                field_id("preferences", "timezone"),
                "タイムゾーン",
                vec![
                    ("jst", "日本標準時 (UTC+9)"),
                    ("utc", "協定世界時 (UTC)"),
                    ("pst", "太平洋標準時 (UTC-8)"),
                ],
            ),
        ],
    );

    div(
        vec![("class", "blocks-onboarding-centered-steps-fields")],
        vec![theme_group, purpose_group, selects],
    )
}

/// アプリ案内カード 1 件（`card` の header/body/footer + Outline `button`）。
fn app_card(platform: &'static str, description: &'static str) -> Node {
    card::root(
        CardProps::default(),
        vec![("data-blocks-onboarding-centered-steps-app-card", "")],
        vec![
            card::header(vec![], vec![card::title(vec![], vec![text(platform)])]),
            card::body(vec![], vec![text(description)]),
            card::footer(
                vec![],
                vec![button::button(
                    &ButtonProps {
                        variant: ButtonVariant::Outline,
                        size: Size::Sm,
                        disabled: true,
                        ..ButtonProps::default()
                    },
                    vec![],
                    vec![text("詳しく見る")],
                )],
            ),
        ],
    )
}

/// **invite**（step 3）: 招待リンク・メール招待・CC チェック・アプリ案内。
fn invite_step() -> Node {
    let link_id = field_id("invite", "link");
    let link_row = div(
        vec![("class", "blocks-onboarding-centered-steps-invite-row")],
        vec![
            readonly_field(
                &link_id,
                "招待リンク",
                "https://example.com/invite/8f2c1a9d",
            ),
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Outline,
                    disabled: true,
                    ..ButtonProps::default()
                },
                vec![],
                vec![text("コピー")],
            ),
        ],
    );

    let email_1 = field_id("invite", "email-1");
    let email_2 = field_id("invite", "email-2");

    let cc_props = PlainCheckboxProps {
        disabled: true,
        ..PlainCheckboxProps::default()
    };
    let cc_checkbox = checkbox::root(
        Size::Md,
        ColorPalette::Accent,
        &cc_props,
        vec![("data-blocks-onboarding-centered-steps-cc-checkbox", "")],
        vec![
            checkbox::hidden_input(
                &cc_props,
                "blocks-onboarding-centered-steps-invite-cc",
                "on",
                vec![],
            ),
            checkbox::control(
                &cc_props,
                vec![],
                vec![checkbox::indicator(&cc_props, vec![], vec![])],
            ),
            checkbox::label(&cc_props, vec![], vec![text("招待メールに自分を CC する")]),
        ],
    );

    let apps = div(
        vec![("class", "blocks-onboarding-centered-steps-apps")],
        vec![
            app_card("iOS 版", "App Store からダウンロードできます。"),
            app_card("Android 版", "Google Play からダウンロードできます。"),
        ],
    );

    div(
        vec![("class", "blocks-onboarding-centered-steps-fields")],
        vec![
            link_row,
            text_field(
                &email_1,
                "招待するメールアドレス",
                "email",
                "sato@example.com",
            ),
            text_field(
                &email_2,
                "招待するメールアドレス",
                "email",
                "suzuki@example.com",
            ),
            cc_checkbox,
            apps,
        ],
    )
}

/// 4 ステップ共通の骨格。`step_index` の `Steps` を組み立て、`body` へ渡した
/// ステップ固有の中身を中央カラムへ差し込む。
fn instance(
    instance_name: &'static str,
    step_index: usize,
    title: &'static str,
    description: &'static str,
    next_label: &'static str,
    body: Node,
) -> Node {
    let s = Steps::new(
        4,
        step_index,
        fandhe_frontend_pre_styled_ui::Orientation::Horizontal,
    );
    div(
        vec![
            ("class", "blocks-onboarding-centered-steps-panel"),
            (
                "data-blocks-onboarding-centered-steps-instance",
                instance_name,
            ),
        ],
        vec![
            top_bar(&s),
            steps::content(
                &s,
                step_index,
                vec![("class", "blocks-onboarding-centered-steps-body")],
                vec![
                    heading(
                        HeadingLevel::H3,
                        &HeadingProps {
                            size: HeadingSize::Xl,
                            ..HeadingProps::default()
                        },
                        vec![],
                        vec![text(title)],
                    ),
                    el(
                        "p",
                        vec![("class", "blocks-onboarding-centered-steps-description")],
                        vec![text(description)],
                    ),
                    body,
                ],
            ),
            nav_actions(&s, next_label),
        ],
    )
}

/// `onboarding-centered-steps` の Demo 本体。呼び出しごとに同一の `Node`
/// を返す純関数。4 インスタンスを縦に並べる。
#[must_use]
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-onboarding-centered-steps-layout")],
        vec![
            instance(
                "profile",
                0,
                "プロフィールを設定しましょう",
                "表示名・役職・写真を登録します。あとから変更できます。",
                "次へ",
                profile_step(),
            ),
            instance(
                "interests",
                1,
                "興味のある分野を教えてください",
                "選んだ内容に合わせて、おすすめの機能をご案内します。",
                "次へ",
                interests_step(),
            ),
            instance(
                "preferences",
                2,
                "利用環境を設定しましょう",
                "テーマや利用目的、チーム規模に合わせて表示を最適化します。",
                "次へ",
                preferences_step(),
            ),
            instance(
                "invite",
                3,
                "チームを招待しましょう",
                "招待リンクを共有するか、メールアドレスで直接招待できます。",
                "はじめる",
                invite_step(),
            ),
        ],
    )
}
```

## 原案差分メモ

- 主参照は R0170（上部ロゴ + 進捗ステップ、中央に見出し・説明 + ステップ
  固有の入力、下部に戻る/次への骨格）です。集約元は R0171（興味関心
  checkbox-card 群）・R0172（テーマ選択 radio-card）・R0179（写真
  アップロード）・R0180（アプリ案内カード）・R0181（招待リンク + メール
  招待行）・R0182（利用目的 radio-card）・R0183（3 つの選択欄）で、これらを
  4 インスタンス（profile / interests / preferences / invite）へ分けて
  併記しています。
- steps 上部ナビは押しても何も起きない `<button>`（dead control）を避け、
  番号 + ラベルの静的構造とし、現在ステップにのみ `aria-current="step"` を
  付与しています。下部の「戻る / 次へ」は `steps` headless の
  `prev_trigger`/`next_trigger` を使いつつ、中間ステップでは自動的に
  `disabled` にならないため、`gallery_carousel` と同型の判断で全インスタンス
  常に明示 `disabled` にしています。
- 興味関心の `checkbox-card`・テーマ/利用目的の `radio-card`・招待メール CC
  の `checkbox`・写真の `file-upload` はいずれもネイティブ `disabled` に
  固定し（ラベルクリック等でブラウザが状態をネイティブに切り替えて
  SSR 時の見た目と食い違うのを防ぐため）、`checkbox-card`/`radio-card`/
  `checkbox` は減光（`opacity`）を中和して通常の見た目に、`file-upload` は
  操作不能であることが伝わるよう減光をそのまま残しています。
- 実データ取得・ステップ遷移・送信処理は行わず、静的な初期状態のみを
  示します。表示名・役職・招待リンク・メールアドレス・アプリ案内の文言は
  すべて独自の架空データです（`example.com` 配下の値、実企業名・実
  クレデンシャル・ストア名は含みません）。
- ブラウザでの実機確認（40rem 前後の幅でカード列が 1〜2 列へ折り返す挙動・
  ライト/ダーク両テーマ）はサンドボックス制約により未実施です。cargo test
  による出力検証のみで代替しました。
