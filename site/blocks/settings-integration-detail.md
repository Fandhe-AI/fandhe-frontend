# settings-integration-detail

連携アプリの詳細画面向けブロックです。ヘッダー（ロゴ・アプリ名・接続状態
バッジ・操作ボタン）・メタ情報（開発元・分類・最終同期・対応プラン）・概要・
主な機能・導入手順・関連する連携・作成導線を縦に並べます。`badge` /
`button` / `link` / `list` / `separator` / `card` / `heading` / `image` の
8 部品を合成します。Blocks は既存部品の合成例であり、新しい UI 部品は
追加しません。

主参照は対応表 ID R0249、集約元は R0256（`_/blocks-intake/` の対応ファイルは
本 worktree に存在しないため、対応表 ID のみを記載）。アプリ名・開発元・
分類・最終同期日時・対応プラン・機能一覧・導入手順・関連連携の文言はすべて
架空のデータであり、実在の企業・製品・商標・PII を含みません。

本 Demo は無 JS の静的表示のみであり、`<form>` を含みません。関連連携・
作成導線のリンクは、実在しない特定ページを指すと誤認させるラベルを避け、
本フレームワークリポジトリへの外部リンク（ラベル「GitHub で見る」）として
います。

本ブロックは骨格・主要領域のみを実装した前半（イシュー #2989）です。
未接続状態との並記・「利点」領域の追加・関連連携カードの充実は後続
イシュー #2990 で対応します。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{badge, BadgeProps, BadgeVariant};
use fandhe_frontend_pre_styled_ui::button::{button, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps};
use fandhe_frontend_pre_styled_ui::image::{image, ImageFit, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::list::{self, ListType, ListVariant};
use fandhe_frontend_pre_styled_ui::recipe::Size;
use fandhe_frontend_pre_styled_ui::separator::{separator, SeparatorProps};

/// 架空アプリ名（連携先として表示する、実在製品ではない）。
const APP_NAME: &str = "Meridian Sync";

/// 関連連携・作成導線のリンク遷移先（実在の URL、`href="#"` は使わない。
/// モジュール doc「関連連携・作成導線のリンクは実在の外部 URL を使う」
/// 節参照）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// ヘッダー左のロゴ画像（`image` 部品、`page_heading_avatar.rs::logo_image`
/// と同型の判断）。
fn app_logo() -> Node {
    image(
        &ImageProps {
            shape: ImageShape::Rounded,
            fit: ImageFit::Contain,
            ..ImageProps::new(dummy_assets::LOGO_SRC, &format!("{APP_NAME} のロゴ"))
        },
        vec![("data-blocks-settings-integration-detail-logo", "")],
    )
}

/// ヘッダー行（ロゴ + アプリ名 + 短い説明 + 接続状態バッジ + 操作ボタン）。
fn header() -> Node {
    let identity = div(
        vec![("class", "blocks-settings-integration-detail-identity")],
        vec![
            div(
                vec![("class", "blocks-settings-integration-detail-title-row")],
                vec![
                    heading(
                        HeadingLevel::H2,
                        &HeadingProps::default(),
                        vec![],
                        vec![text(APP_NAME)],
                    ),
                    badge(
                        &BadgeProps {
                            variant: BadgeVariant::Subtle,
                            ..BadgeProps::default()
                        },
                        vec![("data-blocks-settings-integration-detail-badge", "")],
                        vec![text("接続済み")],
                    ),
                ],
            ),
            el(
                "p",
                vec![("class", "blocks-settings-integration-detail-summary")],
                vec![text(
                    "チームのタスク・通知をワークスペースへ自動で同期する連携アプリです。",
                )],
            ),
        ],
    );
    let actions = div(
        vec![("class", "blocks-settings-integration-detail-actions")],
        vec![
            button(
                &ButtonProps {
                    variant: ButtonVariant::Outline,
                    size: Size::Sm,
                    ..ButtonProps::default()
                },
                vec![("data-blocks-settings-integration-detail-button", "")],
                vec![text("接続を解除")],
            ),
            button(
                &ButtonProps {
                    variant: ButtonVariant::Solid,
                    size: Size::Sm,
                    ..ButtonProps::default()
                },
                vec![("data-blocks-settings-integration-detail-button", "")],
                vec![text("設定を開く")],
            ),
        ],
    );
    div(
        vec![("class", "blocks-settings-integration-detail-header")],
        vec![app_logo(), identity, actions],
    )
}

/// メタ情報 1 ペア（`<dt>`/`<dd>` を包む `<div>`。モジュール doc「メタ情報は
/// `<dl>` + ラベル・値ペアを包む `<div>` で組む」節参照）。
fn meta_item(label: &'static str, value: &'static str) -> Node {
    div(
        vec![("class", "blocks-settings-integration-detail-meta-item")],
        vec![
            el("dt", vec![], vec![text(label)]),
            el("dd", vec![], vec![text(value)]),
        ],
    )
}

/// メタ情報行（開発元・分類・最終同期・対応プランの 4 項目、素の
/// `<dl>`）。
fn meta_row() -> Node {
    el(
        "dl",
        vec![("class", "blocks-settings-integration-detail-meta")],
        vec![
            meta_item("開発元", "Quill & Meridian"),
            meta_item("分類", "生産性"),
            meta_item("最終同期", "2026-09-29 03:12"),
            meta_item("対応プラン", "Growth 以上"),
        ],
    )
}

/// 見出し（H3）+ 子要素 1 個の section。
fn section(title: &'static str, body: Node) -> Node {
    el(
        "section",
        vec![("class", "blocks-settings-integration-detail-section")],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps::default(),
                vec![],
                vec![text(title)],
            ),
            body,
        ],
    )
}

/// マーカー付き箇条書きの 1 項目。
fn list_item(text_value: &'static str) -> Node {
    list::item(vec![], vec![text(text_value)])
}

/// 概要 section（H3 + 段落 2 つ）。
fn overview_section() -> Node {
    section(
        "概要",
        div(
            vec![],
            vec![
                el(
                    "p",
                    vec![],
                    vec![text(format!(
                        "{APP_NAME} は、外部のタスク管理ツールとワークスペースを\
                         双方向に同期する連携アプリです。担当者の割り当て・\
                         期限・完了状態の変更が双方に自動反映されます。",
                    ))],
                ),
                el(
                    "p",
                    vec![],
                    vec![text(
                        "チーム単位で有効化でき、同期対象のプロジェクトは\
                         あとから絞り込めます。",
                    )],
                ),
            ],
        ),
    )
}

/// 機能一覧 section（H3 + 箇条書き、マーカー付き）。
fn features_section() -> Node {
    let items = [
        "タスクの担当者・期限・完了状態を双方向に自動同期",
        "コメントの相互転記",
        "同期対象プロジェクトの絞り込み",
        "同期履歴の閲覧",
        "同期エラー発生時の通知",
    ];
    section(
        "主な機能",
        list::root(
            ListType::Unordered,
            ListVariant::Marker,
            vec![("data-blocks-settings-integration-detail-list", "")],
            items.iter().map(|item| list_item(item)).collect(),
        ),
    )
}

/// 導入手順 section（H3 + 番号付き手順）。
fn setup_section() -> Node {
    let steps = [
        "「設定を開く」から同期するプロジェクトを選択する",
        "同期頻度（リアルタイム・1 時間ごと・手動）を選ぶ",
        "同期対象外にしたいタスクの除外条件を設定する",
    ];
    section(
        "導入手順",
        list::root(
            ListType::Ordered,
            ListVariant::Marker,
            vec![("data-blocks-settings-integration-detail-list", "")],
            steps.iter().map(|step| list_item(step)).collect(),
        ),
    )
}

/// 関連連携 1 枚のカード（`card::root` Outline + タイトル + 説明 +
/// 外部リンク）。
fn related_card(name: &'static str, desc: &'static str) -> Node {
    card::root(
        CardProps::default(),
        vec![("data-blocks-settings-integration-detail-card", "")],
        vec![card::body(
            vec![],
            vec![
                card::title(vec![], vec![text(name)]),
                card::description(vec![], vec![text(desc)]),
                link::root(
                    REPO,
                    &LinkProps {
                        external: true,
                        ..LinkProps::default()
                    },
                    vec![("data-blocks-settings-integration-detail-link", "")],
                    vec![text("GitHub で見る")],
                ),
            ],
        )],
    )
}

/// 関連する連携 section（H3 + カード 3 枚の grid）。
fn related_integrations_section() -> Node {
    section(
        "関連する連携",
        div(
            vec![("class", "blocks-settings-integration-detail-related-grid")],
            vec![
                related_card(
                    "Verdant Foundry Chat",
                    "チームチャットへ同期通知を転送します。",
                ),
                related_card(
                    "Trellisworks Board",
                    "カンバンボードとタスクを相互同期します。",
                ),
                related_card(
                    "Northshelf Calendar",
                    "同期対象タスクの期限をカレンダーへ反映します。",
                ),
            ],
        ),
    )
}

/// 作成導線（H3 なしの単独カード。「独自の連携を作成」+ 説明 +
/// footer にボタン + 外部リンク）。
fn create_cta_section() -> Node {
    card::root(
        CardProps::default(),
        vec![("data-blocks-settings-integration-detail-cta", "")],
        vec![
            card::body(
                vec![],
                vec![
                    card::title(vec![], vec![text("独自の連携を作成")]),
                    card::description(
                        vec![],
                        vec![text(
                            "公開 API を使って、社内ツール向けの独自連携を構築できます。",
                        )],
                    ),
                ],
            ),
            card::footer(
                vec![],
                vec![
                    button(
                        &ButtonProps {
                            variant: ButtonVariant::Solid,
                            size: Size::Sm,
                            ..ButtonProps::default()
                        },
                        vec![("data-blocks-settings-integration-detail-button", "")],
                        vec![text("連携を作成")],
                    ),
                    link::root(
                        REPO,
                        &LinkProps {
                            external: true,
                            ..LinkProps::default()
                        },
                        vec![("data-blocks-settings-integration-detail-link", "")],
                        vec![text("GitHub で見る")],
                    ),
                ],
            ),
        ],
    )
}

/// section 間の区切り線。
fn section_separator() -> Node {
    separator(&SeparatorProps::default(), vec![])
}

/// `settings-integration-detail` の Demo 本体。呼び出しごとに同一の
/// `Node` を返す純関数。
#[must_use]
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-settings-integration-detail-stack")],
        vec![
            header(),
            meta_row(),
            section_separator(),
            overview_section(),
            section_separator(),
            features_section(),
            section_separator(),
            setup_section(),
            section_separator(),
            related_integrations_section(),
            create_cta_section(),
        ],
    )
}
```

## 原案差分メモ

- 主参照 R0249 の 1 版のみを実装しています。未接続状態との並記は後続
  イシュー #2990 で対応します。
- 集約元 R0256 が持つ「利点」領域は本イシューでは実装していません（#2990）。
  関連連携カードは骨格として 3 枚の最小構成のみです（#2990 で充実予定）。
- `_/blocks-intake/` の対応ファイルは本イシュー着手時点で本 worktree に
  存在しないため、対応表 ID（R0249・R0256）のみを記載しています。
- ブラウザでの実機確認（40rem 前後の幅切替・ライト/ダーク表示）は
  サンドボックス環境の制約により未実施です。

関連情報: [Badge](../themes/badge.md) / [Button](../themes/button.md) /
[Link](../themes/link.md) / [List](../themes/list.md) /
[Separator](../themes/separator.md) / [Card](../themes/card.md) /
[Heading](../themes/heading.md) / [Image](../themes/image.md)
