# settings-item-cards

設定対象（認証方式・ロール・ログイン中のセッション）を 1 件 1 カードで縦に
並べた、カード列挙型の設定一覧です。各カードにアイコン・題名・説明・状態
バッジ・右端の操作ボタンを置き、`card` / `badge` / `icon` / `button` /
`heading` / `text` の 6 部品を合成します。Blocks は既存部品の合成例であり、
新しい UI 部品は追加しません。

主参照は対応表 ID R0266（代表構成）で、R0237（認証方式の 3 カード）・
R0265（ロール一覧 + 新規作成）を集約元とします（`_/blocks-intake/` の
対応ファイルは本 worktree に存在しないため、対応表 ID のみを記載します）。
認証方式・ロール・ログイン中のセッションの 3 節を並記することで集約元の
差分を表しています。ロール節・セッション節の見出し右には「ロールを作成」
「他のセッションをすべて終了」のツールバー操作ボタンを置き、現在の
セッションはバッジ（「このデバイス」）で区別しています。

デモ枠の幅が `40rem` 未満では各カードの操作ボタンが本文（題名・説明）の下へ
回り、`40rem` 以上で右端へ戻ります（コンテナクエリ判定、ページのビューポート
幅では判定しません）。操作ボタンは「管理者を編集」「東京 / ブラウザの
セッションを終了」のように対象を含む `aria-label` を持ち、同名ラベルが
複数カードに重複しても読み上げで区別できます。本 Demo は無 JS の静的表示の
みであり、`<form>` を含みません。
デバイス名・場所・氏名を示す情報はすべて独自に書いた架空のものであり、実在の
企業・人物・IP アドレス・クレデンシャル・PII を含みません。

## Rust コード

```rust
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{badge, BadgeProps, BadgeVariant};
use fandhe_frontend_pre_styled_ui::button::{button, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::recipe::ColorPalette;
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};
use fandhe_frontend_pre_styled_ui::Size;

/// 装飾用の自作幾何アイコン（`card_meta_cta::geo_icon` と同型。lucide 等の
/// 既存アイコンセットの path を複製しない単純図形）。
fn geo_icon(path_d: &'static str) -> Node {
    icon(
        &IconProps {
            size: Size::Md,
            ..IconProps::default()
        },
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

/// 鍵の幾何アイコン（パスワード認証）。
fn key_icon() -> Node {
    geo_icon("M15 7a4 4 0 10-3.874 5H4v4h2v-4h2v4h2v-2h3.126A4 4 0 0015 7z")
}

/// スマートフォンの幾何アイコン（認証アプリ）。
fn smartphone_icon() -> Node {
    geo_icon("M8 3h8a1 1 0 011 1v16a1 1 0 01-1 1H8a1 1 0 01-1-1V4a1 1 0 011-1z M11 18h2")
}

/// 盾の幾何アイコン（セキュリティキー）。
fn shield_icon() -> Node {
    geo_icon("M12 3l7 3v6c0 4.5-3 7.5-7 9-4-1.5-7-4.5-7-9V6z")
}

/// 人物の幾何アイコン（ロール）。
fn person_icon() -> Node {
    geo_icon("M12 12a4 4 0 100-8 4 4 0 000 8z M4 20c0-4.5 3.5-7 8-7s8 2.5 8 7")
}

/// ノート PC の幾何アイコン（ログイン中セッション）。
fn laptop_icon() -> Node {
    geo_icon("M5 5h14v10H5z M2 19h20 M9 19l1-2h4l1 2")
}

/// 設定対象カード 1 枚分のデータ（アイコン・題名・説明・状態バッジ・
/// 操作ボタン）。`action_aria_label` は同名の操作ボタン（「編集」「終了」等）
/// が複数カードに重複するため、対象を含む読み上げ名（例:「管理者を編集」）を
/// 個別に持たせる（P1 指摘対応、AGENTS.md UI a11y 観点2）。
struct SettingItem {
    icon: fn() -> Node,
    title: &'static str,
    description: &'static str,
    badge: Option<(&'static str, BadgeVariant, ColorPalette)>,
    action_label: &'static str,
    action_aria_label: &'static str,
    action_variant: ButtonVariant,
    action_palette: ColorPalette,
}

/// 1 枚の設定対象カード（アイコン + 題名/状態バッジ + 説明 + 操作ボタン。
/// [`LAYOUT_CSS`] の `-row` グリッドが並び順・狭幅時の折り返しを決める）。
fn item_card(item: &SettingItem) -> Node {
    let mut title_line: Vec<Node> = vec![heading(
        HeadingLevel::H3,
        &HeadingProps {
            size: HeadingSize::Md,
            weight: HeadingWeight::Semibold,
        },
        vec![],
        vec![text(item.title)],
    )];
    if let Some((label, variant, palette)) = item.badge {
        title_line.push(badge(
            &BadgeProps {
                variant,
                palette,
                ..BadgeProps::default()
            },
            vec![],
            vec![text(label)],
        ));
    }

    card::root(
        CardProps::default(),
        vec![("data-blocks-settings-item-cards-card", "")],
        vec![card::body(
            vec![],
            vec![div(
                // `card::body` は `[data-scope="card"][data-part="body"]`
                // （属性セレクタ 2 個、詳細度 0,2,0）で `display: flex;
                // flex-direction: column` を既定持ちしており、`class`
                // 1 個（詳細度 0,1,0）のグリッド化は詳細度で負けて
                // 適用されない（Bugbot 指摘）。そのため body 直下へ
                // 素の `div` を 1 枚はさみ、グリッド化はそちらへ適用する。
                vec![("class", "blocks-settings-item-cards-row")],
                vec![
                    (item.icon)(),
                    div(
                        vec![("class", "blocks-settings-item-cards-body")],
                        vec![
                            div(
                                vec![("class", "blocks-settings-item-cards-title-line")],
                                title_line,
                            ),
                            styled_text::text(
                                &TextProps {
                                    variant: TextVariant::Muted,
                                    ..TextProps::default()
                                },
                                vec![],
                                vec![text(item.description)],
                            ),
                        ],
                    ),
                    div(
                        vec![("class", "blocks-settings-item-cards-action")],
                        vec![button(
                            &ButtonProps {
                                variant: item.action_variant,
                                palette: item.action_palette,
                                ..ButtonProps::default()
                            },
                            vec![("aria-label", item.action_aria_label)],
                            vec![text(item.action_label)],
                        )],
                    ),
                ],
            )],
        )],
    )
}

/// 認証方式節の 3 カード（集約元 R0237）。
const AUTH_METHOD_ITEMS: &[SettingItem] = &[
    SettingItem {
        icon: key_icon,
        title: "パスワード",
        description: "サインイン時に使うパスワードです。",
        badge: Some(("有効", BadgeVariant::Subtle, ColorPalette::Success)),
        action_label: "変更",
        action_aria_label: "パスワードを変更",
        action_variant: ButtonVariant::Outline,
        action_palette: ColorPalette::Accent,
    },
    SettingItem {
        icon: smartphone_icon,
        title: "認証アプリ",
        description: "ワンタイムコードによる 2 段階認証です。",
        badge: Some(("未設定", BadgeVariant::Outline, ColorPalette::Warning)),
        action_label: "設定する",
        action_aria_label: "認証アプリを設定する",
        action_variant: ButtonVariant::Solid,
        action_palette: ColorPalette::Accent,
    },
    SettingItem {
        icon: shield_icon,
        title: "セキュリティキー",
        description: "物理キーによる 2 段階認証です。",
        badge: Some(("未設定", BadgeVariant::Outline, ColorPalette::Warning)),
        action_label: "追加する",
        action_aria_label: "セキュリティキーを追加する",
        action_variant: ButtonVariant::Solid,
        action_palette: ColorPalette::Accent,
    },
];

/// ロール節の 3 カード（集約元 R0265）。
const ROLE_ITEMS: &[SettingItem] = &[
    SettingItem {
        icon: person_icon,
        title: "管理者",
        description: "請求・メンバー管理を含む全操作が可能です。",
        badge: None,
        action_label: "編集",
        action_aria_label: "管理者を編集",
        action_variant: ButtonVariant::Outline,
        action_palette: ColorPalette::Accent,
    },
    SettingItem {
        icon: person_icon,
        title: "編集者",
        description: "コンテンツの作成・更新が可能です。",
        badge: None,
        action_label: "編集",
        action_aria_label: "編集者を編集",
        action_variant: ButtonVariant::Outline,
        action_palette: ColorPalette::Accent,
    },
    SettingItem {
        icon: person_icon,
        title: "閲覧者",
        description: "閲覧のみが可能です。",
        badge: None,
        action_label: "編集",
        action_aria_label: "閲覧者を編集",
        action_variant: ButtonVariant::Outline,
        action_palette: ColorPalette::Accent,
    },
];

/// セッション節の 3 カード（主参照 R0266・代表構成。1 枚目に「このデバイス」
/// バッジを付け現在のセッションを区別する）。
const SESSION_ITEMS: &[SettingItem] = &[
    SettingItem {
        icon: laptop_icon,
        title: "東京 / ブラウザ",
        description: "最終アクセス：たった今",
        badge: Some(("このデバイス", BadgeVariant::Subtle, ColorPalette::Info)),
        action_label: "終了",
        action_aria_label: "東京 / ブラウザのセッションを終了",
        action_variant: ButtonVariant::Outline,
        action_palette: ColorPalette::Danger,
    },
    SettingItem {
        icon: smartphone_icon,
        title: "大阪 / モバイルアプリ",
        description: "最終アクセス：2 時間前",
        badge: None,
        action_label: "終了",
        action_aria_label: "大阪 / モバイルアプリのセッションを終了",
        action_variant: ButtonVariant::Outline,
        action_palette: ColorPalette::Danger,
    },
    SettingItem {
        icon: laptop_icon,
        title: "福岡 / ブラウザ",
        description: "最終アクセス：3 日前",
        badge: None,
        action_label: "終了",
        action_aria_label: "福岡 / ブラウザのセッションを終了",
        action_variant: ButtonVariant::Outline,
        action_palette: ColorPalette::Danger,
    },
];

/// 節の見出し（H2）とツールバー操作ボタン（任意）を横並びにする。
fn section_toolbar(title: &str, action_label: Option<&str>) -> Node {
    let mut children = vec![heading(
        HeadingLevel::H2,
        &HeadingProps {
            size: HeadingSize::Md,
            weight: HeadingWeight::Semibold,
        },
        vec![],
        vec![text(title)],
    )];
    if let Some(label) = action_label {
        children.push(button(&ButtonProps::default(), vec![], vec![text(label)]));
    }
    div(
        vec![("class", "blocks-settings-item-cards-toolbar")],
        children,
    )
}

/// 1 節分（ツールバー + カード列）を組み立てる。
fn section(title: &str, action_label: Option<&str>, items: &[SettingItem]) -> Node {
    let cards: Vec<Node> = items.iter().map(item_card).collect();
    div(
        vec![],
        vec![
            section_toolbar(title, action_label),
            div(vec![("class", "blocks-settings-item-cards-list")], cards),
        ],
    )
}

/// `settings-item-cards` の Demo 本体（認証方式・ロール・セッションの 3 節を
/// 縦に並べる）。
#[must_use]
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-settings-item-cards-layout")],
        vec![
            section("認証方式", None, AUTH_METHOD_ITEMS),
            section("ロール", Some("ロールを作成"), ROLE_ITEMS),
            section(
                "ログイン中のセッション",
                Some("他のセッションをすべて終了"),
                SESSION_ITEMS,
            ),
        ],
    )
}
```

## 原案差分メモ

- 主参照は R0266（セッション代表構成）です。R0237（認証方式の 3 カード）・
  R0265（ロール一覧 + 新規作成）を別節として併記し、集約元の差分を表して
  います。
- 現在のセッションはバッジ（「このデバイス」、`Info` palette）で区別し、
  終了ボタンは `Danger` palette にしています。ロール節の編集ボタンは
  危険操作ではないため `Accent` palette のままです。
- 狭幅（`40rem` 未満）では各カードの操作ボタンが題名・説明の下へ回り、
  `40rem` 以上で右端へ戻ります。これは本 block 側の CSS が担っています
  （`card`/`button` 部品自体の機能ではありません）。
- 実データ取得・ボタン押下・セッション終了処理は行わず、静的な初期状態の
  みを示します。デバイス名・場所・役割名・氏名はすべて独自の架空データ
  です。
- ブラウザでの実機確認（`40rem` 前後の幅切替・ライト/ダーク両テーマ）は
  未実施です。cargo test による出力検証のみで代替しました。
