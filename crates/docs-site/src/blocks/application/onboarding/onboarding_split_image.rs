//! `onboarding-split-image` block（イシュー #2980。親トラッキング #2951
//! 「Blocks 目的別パーツ拡充」配下、区分は application、カテゴリは
//! Onboarding。カテゴリ本体は本 block が最初のため #2980 でディレクトリ化
//! した、`docs/design/docs-site-blocks-section.md` §18 参照）。左カラムに
//! ロゴ・進捗・見出し・選択カード群・次へボタン、右カラムに装飾画像を置く
//! 分割オンボーディング。狭幅では右カラムを隠し左カラムのみを表示する。
//!
//! # 使用部品
//!
//! `steps` / `radio-card` / `checkbox-card` / `image` / `button` /
//! `heading` / `native-select` の 7 部品を合成する（[`BLOCK`] の `parts` に
//! 一致させる契約、`crates/docs-site/tests/blocks_nav.rs`/
//! `blocks_contract.rs` が検証する）。新しい UI 部品は追加しない。
//!
//! # 1 つの Demo に 4 つの形を縦に並べる
//!
//! Issue のレイアウト仕様（対応表 R0174 を代表参照とし、R0175〔チェック
//! ボックスカード版〕・R0176〔見出し付きラジオカード 4 件〕・R0177〔曜日
//! チェックボックスカード〕を差分として集約する）に従い、[`demo`] は
//! ルート `div.blocks-onboarding-split-image-stack`（[`Block::demo_class`]
//! とは意図的に別名、既存 block と同じ Bugbot 教訓の回避）の中に「形ラベル
//! と形の本体」の組を 4 つ縦に並べる。各形は
//! `data-blocks-onboarding-split-image-variant="<key>"`（`role`/
//! `interests`/`plan`/`schedule`）を持つ。DOM 順は「左カラム（フォーム）→
//! 右カラム（画像パネル）」で固定し、画像パネルは非インタラクティブな
//! 表示専用要素（フォーカス可能な子を持たない）なため Tab 順は視覚順と
//! 一致する。
//!
//! # 参照元と原案からの差分
//!
//! Issue #2980 のレイアウト仕様（ロゴ + 進捗 + 見出し + 選択カード群 +
//! 次へボタンの左カラム、装飾画像の右カラム、狭幅で画像非表示）から構成
//! した。具体的な参照ファイル・取得手段・内部識別子は記載しない
//! （`docs/design/motion-reference-adoption-policy.md` §9 と同じライセンス
//! 上の転記制限）。原案からの意図的な差分:
//!
//! - R0175〜R0177 を個別 block にせず、本 block 内の形として併記する
//!   （対応表の重複登録を避けるための集約判断、計画立案時点の方針）。
//! - 選択カードはすべて `disabled: true` の静的表示固定（無 JS のため
//!   クリックしても選択状態が変わらない構成、`card_form_footer::
//!   payment_item` と同じ判断）。
//! - `<form>` は出力せず、次へボタンは `button::button` の既定
//!   `type="button"` のまま用いる。
//!
//! # `id`/`name` の一意性
//!
//! ラジオカードの `name`・チェックボックスカードの `hidden_input` `name`・
//! `native_select` の `id` はいずれも形キー（`role`/`interests`/`plan`/
//! `schedule`）を接頭辞にして、4 形間で重複させない
//! （`demo_output_has_no_dangling_aria_references_or_duplicate_ids` が
//! 固定する不変条件）。
//!
//! # `drop_class_attr` と CSS フックの選び方
//!
//! `steps::root`/`radio_card::root`/`checkbox_card::root`/`image::image`/
//! `button::button`/`heading::heading`/`native_select::native_select` は
//! いずれも `drop_class_attr`（または同型の固定属性マージ）により呼び出し
//! 側 `attrs` の `class` を黙って除去する契約を持つため、Demo 固有のスタイル
//! フックは `data-blocks-onboarding-split-image-*` 属性で渡し、[`LAYOUT_CSS`]
//! 側も同じ属性セレクタで対応する。素の `div` には `class` がそのまま効く
//! ため、それらは `.blocks-onboarding-split-image-*` クラスセレクタを使う。
//!
//! # `<form>` を持たない・データ取得/送信を行わない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節・`docs/policy/intentional-non-adoption.md` §3.25 に従い、
//! 本 Demo はフォーム・状態機械・送信処理を持たない静的な合成例である。
//! 文言はすべて独自の架空ダミー（実企業名・実クレデンシャル・PII を含まない）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::checkbox_card::{self, CheckboxProps, CheckedState};
use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::steps::Steps;
use fandhe_frontend_pre_styled_ui::field::{self, FieldIds, FieldRootProps};
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
///
/// `steps::trigger` を使わない構成のため、既定では現在ステップを示す
/// `aria-current="step"` が出力されない（headless-ui `steps.rs` の設計上
/// `trigger` にのみ付与される）。スクリーンリーダーへ進捗を伝えるため、
/// 現在ステップの `item` へ `aria-current="step"` を明示付与する
/// （Codex レビュー指摘、イシュー #2980）。
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
                    let item_attrs = if index == variant.step_index() {
                        vec![("aria-current", "step")]
                    } else {
                        vec![]
                    };
                    steps::item(
                        &s,
                        index,
                        item_attrs,
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

    div(
        vec![],
        vec![
            cards,
            field::root(
                &FieldRootProps::default(),
                &team_size_props,
                vec![],
                vec![
                    field::label(&team_size_props, vec![], vec![text("チーム規模")]),
                    team_size,
                ],
            ),
        ],
    )
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
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/onboarding-split-image/",
    title: "onboarding-split-image",
    category: BlockCategory::Onboarding,
    rust_source: "crates/docs-site/src/blocks/application/onboarding/onboarding_split_image.rs",
    demo_class: "blocks-onboarding-split-image",
    parts: &[
        Part {
            label: "Steps",
            path: "/themes/steps/",
        },
        Part {
            label: "Radio Card",
            path: "/themes/radio-card/",
        },
        Part {
            label: "Checkbox Card",
            path: "/themes/checkbox-card/",
        },
        Part {
            label: "Image",
            path: "/themes/image/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Heading",
            path: "/themes/heading/",
        },
        Part {
            label: "Native Select",
            path: "/themes/native-select/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `onboarding_split_image` 固有のレイアウト規則
/// （`crate::blocks::LAYOUT_CSS` doc「block 固有 CSS の置き場」節）。
///
/// セレクタは `.blocks-onboarding-split-image-*` と
/// `[data-blocks-onboarding-split-image-*]` のみを用い、他 block や部品の
/// 素のセレクタへ影響させない。画像パネルは既定で非表示（`display: none`）
/// にし、`>= 48rem`（[`fandhe_frontend_pre_styled_ui::recipe::Breakpoint::Md`]
/// と一致するリテラル値、CSS custom property は `@media` 条件式内で解決
/// できないため直書きする、`auth_split_photo_testimonial` と同じ判断）で
/// 2 カラム grid へ切り替えて表示する。radio-card/checkbox-card は
/// `disabled: true` 固定のため、既定の `opacity: 0.5`（
/// [`fandhe_frontend_pre_styled_ui::recipe::disabled_declarations`]）を
/// 中和し「静的な選択済み表示」を薄く見せない（`card_form_footer`/
/// `auth_split_photo_testimonial` と同じ判断）。
const LAYOUT_CSS: &str = "\
.blocks-onboarding-split-image-stack {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-10);\n}\n\
.blocks-onboarding-split-image-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-onboarding-split-image-label {\n  font-size: var(--fandhe-font-font-size-sm);\n  color: var(--fandhe-color-fg-muted);\n}\n\
[data-blocks-onboarding-split-image-variant] {\n  display: grid;\n  grid-template-columns: minmax(0, 1fr);\n  border: 1px solid var(--fandhe-color-border);\n  border-radius: var(--fandhe-radius-lg);\n  overflow: hidden;\n}\n\
[data-blocks-onboarding-split-image-form] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n  padding: var(--fandhe-space-8);\n}\n\
[data-scope=\"image\"][data-part=\"root\"][data-blocks-onboarding-split-image-logo] {\n  width: 2rem;\n  height: 2rem;\n}\n\
[data-blocks-onboarding-split-image-steps] {\n  display: flex;\n}\n\
.blocks-onboarding-split-image-intro {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-onboarding-split-image-description {\n  color: var(--fandhe-color-fg-muted);\n}\n\
.blocks-onboarding-split-image-cards-label {\n  font-size: var(--fandhe-font-font-size-sm);\n  margin-bottom: var(--fandhe-space-2);\n}\n\
[data-blocks-onboarding-split-image-cards], [data-scope=\"radio-card\"][data-part=\"root\"][data-blocks-onboarding-split-image-cards] {\n  display: grid;\n  gap: var(--fandhe-space-3);\n  grid-template-columns: repeat(2, minmax(0, 1fr));\n}\n\
[data-blocks-onboarding-split-image-cards=\"days\"], [data-scope=\"radio-card\"][data-part=\"root\"][data-blocks-onboarding-split-image-cards=\"days\"] {\n  grid-template-columns: repeat(auto-fill, minmax(4.5rem, 1fr));\n}\n\
[data-blocks-onboarding-split-image-cards] [data-scope=\"radio-card\"][data-part=\"item\"][data-disabled] {\n  opacity: 1;\n}\n\
[data-blocks-onboarding-split-image-cards] [data-scope=\"checkbox-card\"][data-part=\"root\"][data-disabled] {\n  opacity: 1;\n}\n\
[data-scope=\"button\"][data-part=\"root\"][data-blocks-onboarding-split-image-next] {\n  width: 100%;\n}\n\
[data-blocks-onboarding-split-image-panel] {\n  display: none;\n  position: relative;\n  min-height: 16rem;\n}\n\
[data-scope=\"image\"][data-part=\"root\"][data-blocks-onboarding-split-image-photo] {\n  position: absolute;\n  inset: 0;\n  width: 100%;\n  height: 100%;\n}\n\
@media (min-width: 48rem) {\n  \
[data-blocks-onboarding-split-image-variant] {\n    grid-template-columns: repeat(2, minmax(0, 1fr));\n  }\n\
[data-blocks-onboarding-split-image-cards=\"three\"], [data-scope=\"radio-card\"][data-part=\"root\"][data-blocks-onboarding-split-image-cards=\"three\"] {\n    grid-template-columns: repeat(3, minmax(0, 1fr));\n  }\n\
[data-blocks-onboarding-split-image-panel] {\n    display: block;\n  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::{demo, dummy_assets, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// Demo が期待する 7 部品を出力すること、非対話制約（`<form>` 不在・
    /// `data:` URI 不在・チェック済みマーク不在〔`disabled` の
    /// hidden-input はネイティブ操作を封じるため、選択済み分の `checked`
    /// のみ許容する〕）を満たすことの単体回帰。
    #[test]
    fn demo_composes_expected_parts_and_avoids_forms() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"steps\"",
            "data-scope=\"radio-card\"",
            "data-scope=\"checkbox-card\"",
            "data-scope=\"image\"",
            "data-scope=\"button\"",
            "data-scope=\"heading\"",
            "data-part=\"select\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert_eq!(html.matches("<form").count(), 0);
        assert_eq!(html.matches("type=\"submit\"").count(), 0);
        assert_eq!(html.matches("<h2").count(), 0);
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("src=\"data:"));
        assert!(html.contains(dummy_assets::LOGO_SRC));
        assert!(html.contains(dummy_assets::BACKGROUND_SRC));
    }

    /// 4 形すべてが 1 回ずつ出力されること（モジュール doc「1 つの Demo に
    /// 4 つの形を縦に並べる」節）。
    #[test]
    fn demo_renders_all_four_variants() {
        let html = render(&demo());
        for key in ["role", "interests", "plan", "schedule"] {
            assert_eq!(
                html.matches(&format!(
                    "data-blocks-onboarding-split-image-variant=\"{key}\""
                ))
                .count(),
                1,
                "variant {key} should appear exactly once"
            );
        }
    }

    /// id の重複がないことを固定する（アクセシビリティ上の不変条件、
    /// `auth_split_photo_testimonial::demo_has_no_duplicate_ids` と同型）。
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

    /// [`LAYOUT_CSS`] が想定するブレークポイント・画像パネルの既定非表示・
    /// `<` を含まないことを固定する（REQ-1: `</style>` によるスタイル脱出を
    /// 防ぐ）。
    #[test]
    fn layout_css_declares_breakpoint_and_hides_panel_by_default() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("@media (min-width: 48rem)"));
        assert!(LAYOUT_CSS.contains(
            "[data-blocks-onboarding-split-image-panel] {\n  display: none;\n  position: relative;\n  min-height: 16rem;\n}"
        ));
        assert!(LAYOUT_CSS.contains("repeat(2, minmax(0, 1fr))"));
    }

    /// ルート class（`demo_class` とは別名）が `demo()` の出力へ実際に現れる
    /// こと（既存 block と同じ Bugbot 教訓の固定）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-onboarding-split-image-stack\""));
        assert_ne!(
            super::BLOCK.demo_class,
            "blocks-onboarding-split-image-stack"
        );
    }
}
