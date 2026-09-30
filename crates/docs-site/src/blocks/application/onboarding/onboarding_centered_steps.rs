//! `onboarding-centered-steps` block（イシュー #2978。親トラッキング #2951
//! 「Blocks 目的別パーツ拡充」配下）。対応表 ID R0170 を主参照とし、R0171
//! （興味関心 checkbox card 群）・R0172（テーマ選択 radio card）・R0179
//! （写真アップロード）・R0180（アプリ案内カード）・R0181（招待リンク +
//! メール招待行）・R0182（利用目的 radio card）・R0183（3 つの選択欄）を
//! 集約した合成例。上部にロゴ + 進捗ステップ、中央カラムに見出し・説明 +
//! ステップ固有の入力、下部に「戻る / 次へ」を持つ中央寄せオンボーディング
//! フローで、骨格は共通のまま各ステップの中身だけを差し替える。
//! `_/blocks-intake/` の対応ファイルは本イシュー着手時点で本 worktree に
//! 存在しないため、原稿・本コメントには対応表 ID のみを記す
//! （`page_heading_avatar.rs`〔イシュー #2931〕・`list_title_meta.rs`〔イシュー
//! #2925〕と同じ扱い）。
//!
//! Application / Onboarding カテゴリ最初の block（`onboarding.rs` の空雛形を
//! `onboarding/mod.rs` へディレクトリ化した「カテゴリの卒業」、
//! `docs/design/docs-site-blocks-section.md` §18 参照）。
//!
//! # 使用部品
//!
//! `steps` / `field` / `input` / `native-select` / `checkbox-card` /
//! `radio-card` / `card` / `avatar` / `file-upload` / `checkbox` / `button` /
//! `heading` の 12 部品のみを合成する（[`BLOCK`] の `parts` に一致させる
//! 契約、`crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が
//! 検証する）。新しい UI 部品は追加しない。
//!
//! # 4 インスタンスで各ステップを併記する（無 JS のため静的併記）
//!
//! `Steps::new(4, step_index, Orientation::Horizontal)` を変えた 4
//! インスタンスを縦に並べ、各インスタンスは現在ステップの内容のみを描画
//! する（[`fandhe_frontend_headless_ui::steps::Steps::content`] が非現在
//! index を `hidden` にする契約のため、そもそも非表示パネルを作らない）。
//!
//! - **profile**（step 0・主参照 R0170/R0179）: 表示名 + 役職の `field`/
//!   `input`/`native-select` と、写真アップロード（`file-upload` +
//!   `avatar` プレビュー）。
//! - **interests**（step 1・R0171）: 興味関心を選ぶ `checkbox-card` 6 件
//!   （2 件を選択済みで固定）。
//! - **preferences**（step 2・R0172/R0182/R0183）: テーマ選択・利用目的の
//!   `radio-card` 2 群と、チーム規模・業種・タイムゾーンの `native-select`
//!   3 件。
//! - **invite**（step 3・R0181/R0180）: 招待リンク（読み取り専用 `input` +
//!   コピー `button`）・メール招待欄 2 行・招待メール CC の `checkbox`・
//!   モバイルアプリ案内の `card` 2 件。
//!
//! # steps 上部ナビを `trigger` ボタンにしない理由
//!
//! `steps::trigger` は実 `<button>` であり、無 JS の docs サイトでは押しても
//! 何も起きない dead control になる（`empty_state_setup_steps.rs`・
//! `error_page_centered.rs` PR #3212 codex レビューと同型の判断）。番号・
//! ラベルは `item` 直下へ [`step_indicator_and_label`] で静的に組み立て、
//! 現在ステップにのみ `aria-current="step"` を明示付与する
//! （`empty_state_setup_steps::step` と同型）。
//!
//! # `prev_trigger`/`next_trigger` を常にネイティブ disabled にする理由
//!
//! [`fandhe_frontend_headless_ui::steps::Steps::prev_trigger`]/
//! `next_trigger` は実 `<button>` で、`step == 0`/`step == count` の境界
//! でのみ自動的に `disabled` を付与する仕様のため、中間ステップでは押しても
//! 何も起きない dead control になる。`gallery_carousel.rs` が
//! `carousel::prev_trigger`/`next_trigger` を全インスタンスで常に
//! `disabled: true` にした判断（`disabled`/`data-disabled` は
//! `PREV_NEXT_RESERVED` の予約対象外であり、呼び出し側が `attrs` 経由で
//! 追加の無効化理由を上書きできる設計）を踏襲し、[`nav_actions`] は全
//! インスタンスで両ボタンへ明示 `("disabled", ""), data-disabled` を渡す。
//! `gallery_carousel` と同型に disabled の減光（`opacity: 0.5`）は中和しない
//! （操作不能であることを視覚的にも伝える）。
//!
//! # checkbox-card / radio-card / checkbox をネイティブ disabled にする理由
//!
//! `checkbox_card::hidden_input`/`radio_card::item_hidden_input`/
//! `checkbox::hidden_input` はいずれも有効なネイティブ `<input>` であり、
//! `disabled` を渡さない構成では docs サイトが JS ハイドレーションを
//! 行わなくてもラベルクリック・キーボード操作でブラウザが `checked` を
//! ネイティブに切り替えてしまう一方、`indicator`/`item-indicator` の見た目は
//! SSR 時の `checked` 引数から固定生成されるため追従しない
//! （`form_layout_stacked.rs`/`pricing_single_split.rs` と同型の判断）。
//! すべて `disabled: true` を共有し、[`LAYOUT_CSS`] で
//! `opacity: 1`/`cursor: default` に中和する。`radio_card::root` は
//! `disabled` から `aria-disabled` を自動付与しないため
//! [`preference_radio_group`] で明示付与する（`pricing_single_split.rs`
//! と同型）。
//!
//! # file-upload をネイティブ disabled にする理由
//!
//! `file_upload::hidden_input` は通常クリック不可視のまま `trigger`/
//! `dropzone` から `click()` 転送する JS 配線（`fandhe-frontend-wasm-full`）
//! を前提とするが、docs サイトは JS ハイドレーションを行わない。
//! `form_layout_stacked.rs` と同型に `FileUploadProps { disabled: true, .. }`
//! を渡し、`hidden_input` へ `hidden` 存在属性を付与し、`trigger` をネイティブ
//! disabled にする。file-upload は checkbox 系と異なり disabled の減光を
//! 中和しない（`form_layout_stacked.rs` と同じ判断: 中和すると「操作可能に
//! 見えるが実は無反応」という不整合に逆戻りするため）。
//!
//! # `<form>` を持たない・送信処理を持たない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo はフォーム・
//! 状態機械を持たない静的な合成例である。ボタンは `button::button`/
//! `file_upload::trigger` の既定 `type="button"` のまま送信先を持たず、
//! 実際の送信処理・バリデーションは利用者自身の Rust/JS コードで実装する
//! （`docs/policy/intentional-non-adoption.md` §3.25）。招待リンクの
//! 「コピー」ボタン（[`invite_step`]）とアプリ案内カードの「詳しく見る」
//! ボタン（[`app_card`]）は JS 配線・遷移先を持たないため、押しても何も
//! 起きない要素を操作可能に見せないよう `ButtonProps { disabled: true, .. }`
//! を固定する（`list_people.rs` と同型の判断。減光は中和しない）。
//!
//! # ダミー素材について
//!
//! ロゴ・アバターは `crate::blocks::dummy_assets::LOGO_SRC`/`AVATAR_SRC`
//! （モノトーン抽象図形の SVG）、社名は `dummy_assets::COMPANY_NAMES` を
//! 使う。招待リンク・メールアドレスは `example.com` 配下の架空値
//! （`href="#"` は使わない。招待リンクはリンク化せず `input` の `value` に
//! 留める）。文言はすべて独自の日本語で、実在の人物・企業・クレデンシャル・
//! ストア名とは無関係。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
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
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/onboarding-centered-steps/",
    title: "onboarding-centered-steps",
    category: BlockCategory::Onboarding,
    rust_source: "crates/docs-site/src/blocks/application/onboarding/onboarding_centered_steps.rs",
    demo_class: "blocks-onboarding-centered-steps",
    parts: &[
        Part {
            label: "Steps",
            path: "/themes/steps/",
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
            label: "Native Select",
            path: "/themes/native-select/",
        },
        Part {
            label: "Checkbox Card",
            path: "/themes/checkbox-card/",
        },
        Part {
            label: "Radio Card",
            path: "/themes/radio-card/",
        },
        Part {
            label: "Card",
            path: "/themes/card/",
        },
        Part {
            label: "Avatar",
            path: "/themes/avatar/",
        },
        Part {
            label: "File Upload",
            path: "/themes/file-upload/",
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
            label: "Heading",
            path: "/themes/heading/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `onboarding_centered_steps` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節と同型）。セレクタは
/// `.blocks-onboarding-centered-steps-*` と
/// `[data-blocks-onboarding-centered-steps-*]`、および本 block スコープ配下の
/// `checkbox`/`checkbox-card`/`radio-card`/`file-upload` disabled 中和セレクタ
/// のみを用いる。`[data-scope="..."]` のような属性セレクタは block 固有の
/// 接頭辞を持たず他ページの同名部品にも波及するため、必ず
/// `.blocks-onboarding-centered-steps-layout` を子孫結合子で前置してこの
/// block の DOM 配下に限定する（`checkbox-card`/`checkbox` は呼び出し側の
/// `data-blocks-onboarding-centered-steps-*` マーカー属性で個別スコープ
/// 済みのためこの前置は不要）。
const LAYOUT_CSS: &str = "\
.blocks-onboarding-centered-steps-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n}\n\
.blocks-onboarding-centered-steps-panel {\n  display: flex;\n  flex-direction: column;\n  align-items: center;\n  gap: var(--fandhe-space-6);\n  max-width: 40rem;\n  margin-inline: auto;\n}\n\
.blocks-onboarding-centered-steps-top {\n  display: flex;\n  flex-direction: column;\n  align-items: center;\n  gap: var(--fandhe-space-4);\n  width: 100%;\n}\n\
.blocks-onboarding-centered-steps-logo {\n  flex-shrink: 0;\n}\n\
.blocks-onboarding-centered-steps-layout [data-scope=\"steps\"][data-part=\"root\"] {\n  width: 100%;\n}\n\
.blocks-onboarding-centered-steps-step-label {\n  font-size: var(--fandhe-font-size-sm);\n}\n\
.blocks-onboarding-centered-steps-body {\n  width: 100%;\n  text-align: start;\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-onboarding-centered-steps-description {\n  margin: 0;\n  color: var(--fandhe-color-fg-muted);\n}\n\
.blocks-onboarding-centered-steps-fields {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-onboarding-centered-steps-photo-row {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-onboarding-centered-steps-cards {\n  display: grid;\n  grid-template-columns: repeat(auto-fit, minmax(11rem, 1fr));\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-onboarding-centered-steps-radio-group {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-onboarding-centered-steps-selects {\n  display: grid;\n  grid-template-columns: repeat(auto-fit, minmax(11rem, 1fr));\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-onboarding-centered-steps-invite-row {\n  display: flex;\n  flex-wrap: wrap;\n  align-items: flex-end;\n  gap: var(--fandhe-space-2);\n}\n\
[data-blocks-onboarding-centered-steps-invite-link] {\n  flex: 1 1 16rem;\n}\n\
.blocks-onboarding-centered-steps-apps {\n  display: grid;\n  grid-template-columns: repeat(auto-fit, minmax(11rem, 1fr));\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-onboarding-centered-steps-actions {\n  display: flex;\n  justify-content: space-between;\n  flex-wrap: wrap;\n  gap: var(--fandhe-space-2);\n  width: 100%;\n}\n\
[data-scope=\"checkbox-card\"][data-part=\"root\"][data-blocks-onboarding-centered-steps-interest-card][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
.blocks-onboarding-centered-steps-layout [data-scope=\"radio-card\"][data-part=\"item\"][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
[data-scope=\"checkbox\"][data-part=\"root\"][data-blocks-onboarding-centered-steps-cc-checkbox][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, BLOCK, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    fn demo_html() -> String {
        render(&demo())
    }

    #[test]
    fn demo_composes_expected_parts() {
        let html = demo_html();
        for scope in [
            "data-scope=\"steps\"",
            "data-scope=\"field\"",
            "data-scope=\"checkbox-card\"",
            "data-scope=\"radio-card\"",
            "data-scope=\"card\"",
            "data-scope=\"avatar\"",
            "data-scope=\"file-upload\"",
            "data-scope=\"checkbox\"",
            "data-scope=\"button\"",
            "data-scope=\"heading\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
        assert!(html.contains(r#"data-part="input""#));
        assert!(html.contains(r#"data-part="select""#));
    }

    #[test]
    fn demo_wires_all_four_instance_hooks() {
        let html = demo_html();
        for instance_name in ["profile", "interests", "preferences", "invite"] {
            assert!(html.contains(&format!(
                "data-blocks-onboarding-centered-steps-instance=\"{instance_name}\""
            )));
        }
    }

    #[test]
    fn no_form_semantics_or_unsafe_markup() {
        let html = demo_html();
        assert!(!html.contains("<form"));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("src=\"data:"));
    }

    #[test]
    fn ids_have_no_duplicates_and_no_dangling_aria_references() {
        let html = demo_html();
        let mut ids: Vec<&str> = Vec::new();
        for chunk in html.split("id=\"").skip(1) {
            if let Some(end) = chunk.find('"') {
                ids.push(&chunk[..end]);
            }
        }
        let mut sorted = ids.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(ids.len(), sorted.len(), "id が重複している: {ids:?}");

        for chunk in html.split("aria-labelledby=\"").skip(1) {
            if let Some(end) = chunk.find('"') {
                let target = &chunk[..end];
                assert!(
                    ids.contains(&target),
                    "aria-labelledby={target} が参照する id が存在しない"
                );
            }
        }
    }

    #[test]
    fn buttons_are_all_type_button() {
        let html = demo_html();
        let count_open = html.matches("<button").count();
        let count_typed = html.matches(r#"type="button""#).count();
        assert!(count_open > 0);
        assert_eq!(count_open, count_typed);
    }

    #[test]
    fn only_current_step_content_is_visible_per_instance() {
        let html = demo_html();
        // 各インスタンスは現在ステップのみを描画するため、非表示
        // (`hidden`/`data-state="closed"`) な steps content パネルは
        // 存在しない（現在ステップのみを描画する設計、モジュール doc
        // 「4 インスタンスで各ステップを併記する」節参照）。
        assert!(!html.contains(r#"data-part="content" data-state="closed""#));
        assert_eq!(html.matches("aria-current=\"step\"").count(), 4);
    }

    #[test]
    fn nav_actions_are_natively_disabled_on_every_instance() {
        let html = demo_html();
        // 4 インスタンス × (prev + next) = 8 個すべてが disabled。
        assert_eq!(html.matches(r#"data-part="prev-trigger""#).count(), 4);
        assert_eq!(html.matches(r#"data-part="next-trigger""#).count(), 4);
        // 各インスタンスの prev/next trigger 2 個が常に disabled になる
        // ことの下限チェック（他要素の disabled 分は許容し、8 個以上を
        // 確認する）。
        assert!(html.matches(" disabled=\"\"").count() >= 8);
    }

    #[test]
    fn layout_css_is_safe_and_uses_auto_fit_without_media_or_hiding() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(!LAYOUT_CSS.contains("display: none"));
        assert!(LAYOUT_CSS.contains("auto-fit"));
    }

    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = demo_html();
        assert!(html.contains("class=\"blocks-onboarding-centered-steps-layout\""));
        assert_ne!(BLOCK.demo_class, "blocks-onboarding-centered-steps-layout");
    }

    #[test]
    fn demo_is_deterministic() {
        assert_eq!(demo_html(), demo_html());
    }

    #[test]
    fn uses_shared_dummy_assets() {
        let html = demo_html();
        assert!(html.contains(super::dummy_assets::LOGO_SRC));
    }
}
