# onboarding-split-image

`fandhe-frontend-pre-styled-ui` の `steps` / `radio-card` / `checkbox-card` /
`image` / `button` / `heading` / `native-select` 部品を合成した、画像付き
分割オンボーディングの合成例です。Blocks セクションは新規部品を追加する
ものではなく、既存の Themes/Primitives 部品を組み合わせた実例集であること
に注意してください（対応表 ID R0174 を代表参照とし、R0175（チェックボック
スカード版）・R0176（見出し付きラジオカード 4 件）・R0177（曜日チェック
ボックスカード）を、下記 Demo 内で 4 形として併記します。出典の固有名・
ファイル名は記載しません）。

左カラムにロゴ・進捗（`steps`）・見出し・選択カード群・次へボタン、右
カラムに装飾画像を配置する 2 カラム構成です。狭幅（48rem 未満）では右
カラムの画像を隠し、左カラムのみを表示します。選択カードはすべて
`disabled: true` の静的表示固定（無 JS のためクリックしても選択状態は
変わりません）で、`<form>` 要素は出力せず、次へボタンは `type="button"`
のままで送信先・入力値検証・状態管理は一切持ちません
（`docs/policy/intentional-non-adoption.md` §3.25 の責務境界: UI
コンポーネント層はアプリケーションロジックを内包しません。実際の遷移・
送信処理を実装する場合は、利用者自身の Rust/JS コードで実装してください）。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::checkbox_card::{self, CheckboxProps, CheckedState};
use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::steps::Steps;
use fandhe_frontend_pre_styled_ui::field::FieldIds;
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::image::{self, ImageFit, ImageProps};
use fandhe_frontend_pre_styled_ui::input::FieldProps;
use fandhe_frontend_pre_styled_ui::native_select::{self, NativeSelectProps};
use fandhe_frontend_pre_styled_ui::radio_card::{self, Orientation};
use fandhe_frontend_pre_styled_ui::steps;
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// 4 つの形を区別する block ローカル列挙型（モジュール doc「id/name の
/// 一意性」節）。`format!`/`Box::leak` に頼らず、各アームへ属性値を直接
/// 書き下す。
#[derive(Clone, Copy, PartialEq, Eq)]
enum SplitImageVariant {
    Role,
    Interests,
    Plan,
    Schedule,
}

impl SplitImageVariant {
    /// CSS フック・[`SplitImageVariant`] 判定用の文字列値
    /// （`data-blocks-onboarding-split-image-variant` へ渡す）。
    fn key(self) -> &'static str {
        match self {
            SplitImageVariant::Role => "role",
            SplitImageVariant::Interests => "interests",
            SplitImageVariant::Plan => "plan",
            SplitImageVariant::Schedule => "schedule",
        }
    }

    /// 進捗（[`steps_row`]）における現在地（0 始まり、4 段中）。
    fn step_index(self) -> usize {
        match self {
            SplitImageVariant::Role => 0,
            SplitImageVariant::Interests => 1,
            SplitImageVariant::Plan => 2,
            SplitImageVariant::Schedule => 3,
        }
    }

    /// 見出し・説明文。
    fn heading_text(self) -> (&'static str, &'static str) {
        match self {
            SplitImageVariant::Role => (
                "利用目的を教えてください",
                "選択に応じて、次のステップの案内を最適化します。",
            ),
            SplitImageVariant::Interests => (
                "興味のある分野を選んでください",
                "複数選択できます。あとから設定で変更できます。",
            ),
            SplitImageVariant::Plan => (
                "プランを選んでください",
                "チーム規模も合わせて教えてください。",
            ),
            SplitImageVariant::Schedule => (
                "通知を受け取る曜日を選んでください",
                "選んだ曜日にまとめて更新情報をお届けします。",
            ),
        }
    }

    /// 次へボタンの文言（最終形のみ「開始する」で締める）。
    fn next_label(self) -> &'static str {
        match self {
            SplitImageVariant::Schedule => "開始する",
            _ => "次へ",
        }
    }
}

/// 進捗表示（[`Steps`] 4 段。`trigger`/`content`/`separator` は置かず、
/// 番号インジケータのみを横に並べる。主見出しは [`SplitImageVariant::
/// heading_text`] 側で別途出すため、`item` 直下に本文は持たない）。
fn steps_row(variant: SplitImageVariant) -> Node {
    let s = Steps::new(4, variant.step_index(), Orientation::Horizontal);
    steps::root(
        Size::Sm,
        ColorPalette::Accent,
        &s,
        vec![("data-blocks-onboarding-split-image-steps", "")],
        vec![steps::list(
            &s,
            vec![],
            (0..4)
                .map(|index| {
                    steps::item(
                        &s,
                        index,
                        vec![],
                        vec![steps::indicator(
                            &s,
                            index,
                            vec![],
                            vec![text((index + 1).to_string())],
                        )],
                    )
                })
                .collect(),
        )],
    )
}

/// ロゴ画像（左カラム上部、モジュール doc「使用部品」節）。
fn logo() -> Node {
    image::image(
        &ImageProps::new(dummy_assets::LOGO_SRC, "サービスロゴ"),
        vec![("data-blocks-onboarding-split-image-logo", "")],
    )
}

/// radio-card 1 件（[`radio_card::item`] 系 4 パーツをまとめる、
/// `card_form_footer::payment_item` と同型）。`description` が `None` の
/// ときは `item_description` を出さない（`Plan` 形では見出し付き、`Role`
/// 形では見出しのみの 2 バリエーションを 1 関数で表す）。
fn radio_card_item(
    name: &'static str,
    checked: bool,
    value: &'static str,
    label: &'static str,
    description: Option<&'static str>,
) -> Node {
    let mut content_children = vec![radio_card::item_text(vec![], vec![text(label)])];
    if let Some(description) = description {
        content_children.push(radio_card::item_description(
            vec![],
            vec![text(description)],
        ));
    }
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
                    radio_card::item_content(vec![], content_children),
                ],
            ),
        ],
    )
}

/// radio-card グループ（見出し + `radio_card::root`、モジュール doc
/// 「id/name の一意性」節）。`columns` は [`LAYOUT_CSS`] の
/// `[data-blocks-onboarding-split-image-cards]` セレクタが読む列数フック。
fn radio_card_group(
    labelled_by: &'static str,
    label_text: &'static str,
    columns: &'static str,
    items: Vec<Node>,
) -> Node {
    div(
        vec![],
        vec![
            radio_card::label(Some(labelled_by), vec![], vec![text(label_text)]),
            radio_card::root(
                Size::Sm,
                ColorPalette::Accent,
                true,
                None::<Orientation>,
                Some(labelled_by),
                vec![("data-blocks-onboarding-split-image-cards", columns)],
                items,
            ),
        ],
    )
}

/// checkbox-card 1 件（[`checkbox_card`] 6 パーツをまとめる）。常に
/// `disabled: true` の静的表示固定（モジュール doc「原案からの差分」節）。
fn checkbox_card_item(
    name: &'static str,
    checked: bool,
    value: &'static str,
    label: &'static str,
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
        Size::Sm,
        ColorPalette::Accent,
        &props,
        vec![],
        vec![
            checkbox_card::hidden_input(&props, name, value, vec![]),
            checkbox_card::control(
                &props,
                vec![],
                vec![checkbox_card::indicator(
                    &props,
                    vec![],
                    vec![checkbox_card::indicator_check(&props, vec![], vec![])],
                )],
            ),
            checkbox_card::content(
                &props,
                vec![],
                vec![checkbox_card::label(&props, vec![], vec![text(label)])],
            ),
        ],
    )
}

/// checkbox-card グループ（見出し + 列数フック付きグリッド）。
fn checkbox_card_group(label_text: &'static str, columns: &'static str, items: Vec<Node>) -> Node {
    div(
        vec![],
        vec![
            div(
                vec![("class", "blocks-onboarding-split-image-cards-label")],
                vec![text(label_text)],
            ),
            div(
                vec![("data-blocks-onboarding-split-image-cards", columns)],
                items,
            ),
        ],
    )
}

/// 「role」形のカード群（利用目的、`radio-card` 3 択・3 列。R0174 代表）。
fn role_cards() -> Node {
    radio_card_group(
        "blocks-onboarding-split-image-role-label",
        "利用目的",
        "three",
        vec![
            radio_card_item(
                "blocks-onboarding-split-image-role-purpose",
                true,
                "personal",
                "個人利用",
                None,
            ),
            radio_card_item(
                "blocks-onboarding-split-image-role-purpose",
                false,
                "team",
                "チーム利用",
                None,
            ),
            radio_card_item(
                "blocks-onboarding-split-image-role-purpose",
                false,
                "enterprise",
                "全社導入",
                None,
            ),
        ],
    )
}

/// 「interests」形のカード群（興味分野、`checkbox-card` 6 択・3 列。
/// R0175）。
fn interests_cards() -> Node {
    let items = [
        ("design", "デザイン", true),
        ("engineering", "エンジニアリング", true),
        ("marketing", "マーケティング", false),
        ("sales", "セールス", false),
        ("support", "サポート", false),
        ("data", "データ分析", false),
    ];
    checkbox_card_group(
        "興味のある分野",
        "three",
        items
            .into_iter()
            .map(|(value, label, checked)| {
                checkbox_card_item(
                    "blocks-onboarding-split-image-interests-selection",
                    checked,
                    value,
                    label,
                )
            })
            .collect(),
    )
}

/// 「plan」形のカード群（見出し付きラジオカード 4 件・2 列 + チーム規模
/// select。R0176）。
fn plan_cards() -> Node {
    let cards = radio_card_group(
        "blocks-onboarding-split-image-plan-label",
        "プラン",
        "two",
        vec![
            radio_card_item(
                "blocks-onboarding-split-image-plan-tier",
                false,
                "free",
                "Free",
                Some("個人での試用に。主要機能を制限付きで利用できます。"),
            ),
            radio_card_item(
                "blocks-onboarding-split-image-plan-tier",
                true,
                "pro",
                "Pro",
                Some("小規模チーム向け。全機能を無制限に利用できます。"),
            ),
            radio_card_item(
                "blocks-onboarding-split-image-plan-tier",
                false,
                "team",
                "Team",
                Some("複数チーム向け。権限管理と監査ログが付きます。"),
            ),
            radio_card_item(
                "blocks-onboarding-split-image-plan-tier",
                false,
                "enterprise",
                "Enterprise",
                Some("全社導入向け。専任サポートと SLA が付きます。"),
            ),
        ],
    );

    let team_size_id = "blocks-onboarding-split-image-plan-team-size";
    let team_size_props = FieldProps {
        id: team_size_id,
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: false,
    };
    let team_size = native_select::native_select(
        &NativeSelectProps::default(),
        &team_size_props,
        vec![],
        vec![
            el(
                "option",
                vec![("value", "1-10"), ("selected", "")],
                vec![text("1〜10 人")],
            ),
            el("option", vec![("value", "11-50")], vec![text("11〜50 人")]),
            el(
                "option",
                vec![("value", "51-plus")],
                vec![text("51 人以上")],
            ),
        ],
    );

    div(vec![], vec![cards, team_size])
}

/// 「schedule」形のカード群（通知曜日、`checkbox-card` 7 択・auto-fill
/// グリッド。R0177）。
fn schedule_cards() -> Node {
    let items = [
        ("mon", "月", true),
        ("tue", "火", false),
        ("wed", "水", true),
        ("thu", "木", false),
        ("fri", "金", false),
        ("sat", "土", false),
        ("sun", "日", false),
    ];
    checkbox_card_group(
        "通知を受け取る曜日",
        "days",
        items
            .into_iter()
            .map(|(value, label, checked)| {
                checkbox_card_item(
                    "blocks-onboarding-split-image-schedule-day",
                    checked,
                    value,
                    label,
                )
            })
            .collect(),
    )
}

/// 左カラム（ロゴ + 進捗 + 見出し + カード群 + 次へボタン）。
fn form_column(variant: SplitImageVariant, cards: Node) -> Node {
    let (title, description) = variant.heading_text();
    div(
        vec![("data-blocks-onboarding-split-image-form", "")],
        vec![
            logo(),
            steps_row(variant),
            div(
                vec![("class", "blocks-onboarding-split-image-intro")],
                vec![
                    heading(
                        HeadingLevel::H3,
                        &HeadingProps {
                            size: HeadingSize::Xl,
                            weight: HeadingWeight::Bold,
                        },
                        vec![("data-blocks-onboarding-split-image-title", "")],
                        vec![text(title)],
                    ),
                    div(
                        vec![("class", "blocks-onboarding-split-image-description")],
                        vec![text(description)],
                    ),
                ],
            ),
            cards,
            button::button(
                &ButtonProps::default(),
                vec![("data-blocks-onboarding-split-image-next", "")],
                vec![text(variant.next_label())],
            ),
        ],
    )
}

/// 右カラム（装飾画像、狭幅では非表示。表示専用でフォーカス可能な子を
/// 持たない、モジュール doc「1 つの Demo に 4 つの形を縦に並べる」節）。
fn image_panel() -> Node {
    div(
        vec![("data-blocks-onboarding-split-image-panel", "")],
        vec![image::image(
            &ImageProps {
                fit: ImageFit::Cover,
                ..ImageProps::new(dummy_assets::BACKGROUND_SRC, "")
            },
            vec![("data-blocks-onboarding-split-image-photo", "")],
        )],
    )
}

/// 1 つの形のレイアウト骨格（形ラベル + 左カラム・右カラムを DOM 順どおり
/// に渡す）。
fn variant_layout(variant: SplitImageVariant, label: &'static str, cards: Node) -> Node {
    div(
        vec![("class", "blocks-onboarding-split-image-layout")],
        vec![
            div(
                vec![("class", "blocks-onboarding-split-image-label")],
                vec![text(label)],
            ),
            div(
                vec![("data-blocks-onboarding-split-image-variant", variant.key())],
                vec![form_column(variant, cards), image_panel()],
            ),
        ],
    )
}

/// `onboarding-split-image` の Demo 本体（role/interests/plan/schedule の
/// 4 形を縦に並記する、モジュール doc「参照元と原案からの差分」節参照）。
/// 呼び出しごとに同一の `Node` を返す純関数。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-onboarding-split-image-stack")],
        vec![
            variant_layout(
                SplitImageVariant::Role,
                "利用目的（radio-card）",
                role_cards(),
            ),
            variant_layout(
                SplitImageVariant::Interests,
                "興味分野（checkbox-card）",
                interests_cards(),
            ),
            variant_layout(
                SplitImageVariant::Plan,
                "プラン選択（見出し付き radio-card + native-select）",
                plan_cards(),
            ),
            variant_layout(
                SplitImageVariant::Schedule,
                "通知曜日（checkbox-card）",
                schedule_cards(),
            ),
        ],
    )
}
```

## 原案差分メモ

- 対応表 ID R0175（チェックボックスカード版）・R0176（見出し付きラジオ
  カード 4 件）・R0177（曜日チェックボックスカード）を個別 block にせず、
  本 block 内の「role」「interests」「plan」「schedule」4 形として Demo
  内に縦に併記しています（対応表の重複登録を避けるための集約判断）。
- 文言（利用目的・興味分野・プラン名・曜日等）はすべて独自の架空ダミー
  です。実企業名・実クレデンシャル・PII は含みません。
- 選択カード（radio-card/checkbox-card）は常に `disabled: true` の静的
  表示固定です。クリックしても選択状態は変わりません。
- `<form>` 要素・送信先・入力値検証・状態管理は持ちません。
