# settings-integration-detail

連携アプリの詳細画面向けブロックです。ヘッダー（ロゴ・アプリ名・接続状態
バッジ・操作ボタン）・メタ情報（開発元・分類・最終同期・対応プラン）・概要・
主な機能・導入手順または利点・関連する連携・作成導線を縦に並べます。
`badge` / `button` / `link` / `list` / `separator` / `card` / `heading` /
`image` の 8 部品を合成します。Blocks は既存部品の合成例であり、新しい
UI 部品は追加しません。

接続済み（版 A、対応表 ID R0249 主参照）と未接続（版 B、対応表 ID R0256）
の 2 版を並記します（`_/blocks-intake/` の対応ファイルは本 worktree に
存在しないため、対応表 ID のみを記載）。アプリ名・開発元・分類・
最終同期日時・対応プラン・機能一覧・導入手順・利点・関連連携の文言は
すべて架空のデータであり、実在の企業・製品・商標・PII を含みません。

本 Demo は無 JS の静的表示のみであり、`<form>` を含みません。関連連携・
作成導線のリンクは、実在しない特定ページを指すと誤認させるラベルを避け、
本フレームワークリポジトリへの外部リンク（ラベル「GitHub で見る」）として
います。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{badge, BadgeProps, BadgeVariant};
use fandhe_frontend_pre_styled_ui::button::{button, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::image::{image, ImageFit, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::list::{self, ListType, ListVariant};
use fandhe_frontend_pre_styled_ui::recipe::{ColorPalette, Size};
use fandhe_frontend_pre_styled_ui::separator::{separator, SeparatorProps};

/// 架空アプリ名（連携先として表示する、実在製品ではない）。
const APP_NAME: &str = "Meridian Sync";

/// 関連連携・作成導線のリンク遷移先（実在の URL、`href="#"` は使わない。
/// モジュール doc「関連連携・作成導線のリンクは実在の外部 URL を使う」
/// 節参照）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// 関連する連携 1 件分のデータ（両版で共有する `const` 配列 [`RELATED`]
/// の要素、モジュール doc「関連連携カード」節参照）。
struct Related {
    name: &'static str,
    category: &'static str,
    desc: &'static str,
}

/// 関連する連携 3 件（架空のアプリ名・分類・説明文）。
const RELATED: [Related; 3] = [
    Related {
        name: "Verdant Foundry Chat",
        category: "チーム連絡",
        desc: "チームチャットへ同期通知を転送します。",
    },
    Related {
        name: "Trellisworks Board",
        category: "プロジェクト管理",
        desc: "カンバンボードとタスクを相互同期します。",
    },
    Related {
        name: "Northshelf Calendar",
        category: "カレンダー",
        desc: "同期対象タスクの期限をカレンダーへ反映します。",
    },
];

/// ロゴ画像（`image` 部品、`page_heading_avatar.rs::logo_image` と同型の
/// 判断）。`name` を引数化してヘッダー（[`header`]）と関連連携カード
/// （[`related_card`]）の双方で共用する。
fn logo(name: &str, data_attr: (&'static str, &'static str)) -> Node {
    image(
        &ImageProps {
            shape: ImageShape::Rounded,
            fit: ImageFit::Contain,
            ..ImageProps::new(dummy_assets::LOGO_SRC, &format!("{name} のロゴ"))
        },
        vec![data_attr],
    )
}

/// ヘッダー行（ロゴ + アプリ名 + 短い説明 + 接続状態バッジ + 操作ボタン）。
/// `connected` で状態バッジ・操作ボタンの構成を切り替える
/// （モジュール doc「2 版と集約元 ID の対応」節参照）。
fn header(connected: bool) -> Node {
    let (badge_variant, badge_palette, badge_label) = if connected {
        (BadgeVariant::Subtle, ColorPalette::Success, "接続済み")
    } else {
        (BadgeVariant::Outline, ColorPalette::Neutral, "未接続")
    };
    let identity = div(
        vec![("class", "blocks-settings-integration-detail-identity")],
        vec![
            div(
                vec![("class", "blocks-settings-integration-detail-title-row")],
                vec![
                    heading(
                        HeadingLevel::H3,
                        &HeadingProps::default(),
                        vec![],
                        vec![text(APP_NAME)],
                    ),
                    badge(
                        &BadgeProps {
                            variant: badge_variant,
                            palette: badge_palette,
                            ..BadgeProps::default()
                        },
                        vec![("data-blocks-settings-integration-detail-badge", "")],
                        vec![text(badge_label)],
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
    let actions = if connected {
        div(
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
        )
    } else {
        div(
            vec![("class", "blocks-settings-integration-detail-actions")],
            vec![button(
                &ButtonProps {
                    variant: ButtonVariant::Solid,
                    size: Size::Sm,
                    ..ButtonProps::default()
                },
                vec![("data-blocks-settings-integration-detail-button", "")],
                vec![text("接続する")],
            )],
        )
    };
    div(
        vec![("class", "blocks-settings-integration-detail-header")],
        vec![
            logo(
                APP_NAME,
                ("data-blocks-settings-integration-detail-logo", ""),
            ),
            identity,
            actions,
        ],
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

/// メタ情報行（素の `<dl>`）。`connected` が `false` のときは「最終同期」
/// ペア自体を出力しない（モジュール doc「メタ情報は `<dl>` + ラベル・値
/// ペアを包む `<div>` で組む」節参照）。
fn meta_row(connected: bool) -> Node {
    let mut items = vec![
        meta_item("開発元", "Quill & Meridian"),
        meta_item("分類", "生産性"),
    ];
    if connected {
        items.push(meta_item("最終同期", "2026-09-29 03:12"));
    }
    items.push(meta_item("対応プラン", "Growth 以上"));
    el(
        "dl",
        vec![("class", "blocks-settings-integration-detail-meta")],
        items,
    )
}

/// 見出し（H4）+ 子要素 1 個の section（モジュール doc「2 版の並記と見出し
/// 階層」節参照。版キャプションが `h2`・アプリ名が `h3` のため、section
/// 見出しは 1 段下げた `h4` とする）。
fn section(title: &'static str, body: Node) -> Node {
    el(
        "section",
        vec![("class", "blocks-settings-integration-detail-section")],
        vec![
            heading(
                HeadingLevel::H4,
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

/// 概要 section（H4 + 段落 2 つ、両版で共有）。
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

/// 機能一覧 section（H4 + 箇条書き、マーカー付き、両版で共有）。
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

/// 導入手順 section（H4 + 番号付き手順、版 A（接続済み）のみで使う。
/// 接続後にしか意味を持たない手順のため、版 B（未接続）では
/// [`benefits_section`] に差し替える、モジュール doc「2 版と集約元 ID の
/// 対応」節参照）。
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

/// 利点 section（H4 + 箇条書き、マーカー付き、版 B（未接続）のみで使う。
/// 集約元 R0256 由来の領域。「主な機能」（機能そのものの列挙）とは観点を
/// 分け、導入によって得られる成果・効果を書く、モジュール doc「2 版と
/// 集約元 ID の対応」節参照）。
fn benefits_section() -> Node {
    let items = [
        "手作業の転記がなくなり、担当者変更の反映漏れを防げます",
        "同期履歴が残るため、いつ何が変わったかを後から追跡できます",
        "同期対象プロジェクトを絞り込めるため、無関係な通知を避けられます",
        "同期エラー発生時に通知が届くため、気づかないまま同期が止まる事態を防げます",
    ];
    section(
        "利点",
        list::root(
            ListType::Unordered,
            ListVariant::Marker,
            vec![("data-blocks-settings-integration-detail-list", "")],
            items.iter().map(|item| list_item(item)).collect(),
        ),
    )
}

/// 関連連携 1 枚のカード（`card::root` Outline + ロゴ + アプリ名（`h5`）+
/// 分類バッジ + 説明 + 外部リンク。モジュール doc「関連連携カード」節
/// 参照。`card::title`（`<h3>` 固定）は section 見出し（`h4`）より上位の
/// 階層になってしまうため使わない）。
fn related_card(related: &Related) -> Node {
    card::root(
        CardProps::default(),
        vec![("data-blocks-settings-integration-detail-card", "")],
        vec![
            card::header(
                vec![("class", "blocks-settings-integration-detail-card-header")],
                vec![
                    logo(
                        related.name,
                        ("data-blocks-settings-integration-detail-card-logo", ""),
                    ),
                    heading(
                        HeadingLevel::H5,
                        &HeadingProps::default(),
                        vec![],
                        vec![text(related.name)],
                    ),
                ],
            ),
            card::body(
                vec![],
                vec![
                    badge(
                        &BadgeProps {
                            variant: BadgeVariant::Subtle,
                            ..BadgeProps::default()
                        },
                        vec![],
                        vec![text(related.category)],
                    ),
                    card::description(vec![], vec![text(related.desc)]),
                ],
            ),
            card::footer(
                vec![],
                vec![link::root(
                    REPO,
                    &LinkProps {
                        external: true,
                        ..LinkProps::default()
                    },
                    vec![("data-blocks-settings-integration-detail-link", "")],
                    vec![text("GitHub で見る")],
                )],
            ),
        ],
    )
}

/// 関連する連携 section（H4 + カード 3 枚の grid、[`RELATED`] から組み立て、
/// 両版で共有）。
fn related_integrations_section() -> Node {
    section(
        "関連する連携",
        div(
            vec![("class", "blocks-settings-integration-detail-related-grid")],
            RELATED.iter().map(related_card).collect(),
        ),
    )
}

/// 作成導線（単独カード。「独自の連携を作成」（`h4`）+ 説明 + footer に
/// ボタン + 外部リンク、両版で共有。「関連する連携」（h4）と対等な独立導線
/// のため表題も `h4` に揃える（モジュール doc「2 版の並記と見出し階層」
/// 節参照、Codex 指摘・PR #3458）。`card::title` を使わない理由は
/// [`related_card`] rustdoc と同じ）。
fn create_cta_section() -> Node {
    card::root(
        CardProps::default(),
        vec![("data-blocks-settings-integration-detail-cta", "")],
        vec![
            card::body(
                vec![],
                vec![
                    heading(
                        HeadingLevel::H4,
                        &HeadingProps::default(),
                        vec![],
                        vec![text("独自の連携を作成")],
                    ),
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

/// 版のキャプション（見出し）。`h2`（アプリ名 `h3` の親階層）として
/// 構造化する（`table_with_toolbar.rs::caption` と同型のパターン、
/// モジュール doc「2 版の並記と見出し階層」節参照）。`heading` は `class`
/// を `drop_class_attr` 経由で除去するため、スタイルフックには
/// `data-blocks-settings-integration-detail-caption` を使う。
fn caption(label: &'static str) -> Node {
    heading(
        HeadingLevel::H2,
        &HeadingProps {
            size: HeadingSize::Sm,
            ..HeadingProps::default()
        },
        vec![("data-blocks-settings-integration-detail-caption", "")],
        vec![text(label)],
    )
}

/// 版 A（接続済み、R0249 主参照）。ヘッダー・メタ情報・導入手順が
/// 版 B と異なり、他 4 section（概要・主な機能・関連する連携・作成導線）は
/// 共有する。
fn version_connected() -> Node {
    div(
        vec![("class", "blocks-settings-integration-detail-version")],
        vec![
            caption("接続済み"),
            header(true),
            meta_row(true),
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

/// 版 B（未接続、R0256）。ヘッダー・メタ情報・末尾 section（利点）が
/// 版 A と異なる。
fn version_disconnected() -> Node {
    div(
        vec![("class", "blocks-settings-integration-detail-version")],
        vec![
            caption("未接続"),
            header(false),
            meta_row(false),
            section_separator(),
            overview_section(),
            section_separator(),
            features_section(),
            section_separator(),
            benefits_section(),
            section_separator(),
            related_integrations_section(),
            create_cta_section(),
        ],
    )
}

/// `settings-integration-detail` の Demo 本体（版 A/B を並記。呼び出し
/// ごとに同一の `Node` を返す純関数、モジュール doc「2 版の並記と見出し
/// 階層」節参照）。
#[must_use]
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-settings-integration-detail-stack")],
        vec![version_connected(), version_disconnected()],
    )
}
```

## 原案差分メモ

- **版 A（接続済み、R0249 主参照）**: バッジ「接続済み」（Subtle/Success
  palette）+ 操作 2 個（「接続を解除」「設定を開く」）。メタ情報に
  「最終同期」を含む。「導入手順」section（番号付き手順 3 件）まで。
- **版 B（未接続、R0256）**: バッジ「未接続」（Outline/Neutral palette）+
  操作 1 個（「接続する」）。メタ情報から「最終同期」ペア自体を省く
  （「—」等の代替値は使わない）。「導入手順」の代わりに R0256 由来の
  「利点」section（箇条書き 4 項目、成果・効果の観点）を置く。「導入手順」
  は接続後にしか意味を持たず、「利点」は接続前の説明として位置付けが
  異なるため、section の入れ替えで状態差と集約元差の両方を Demo から
  読み取れるようにしている。
- 版キャプション（`h2`）を新設し、見出し階層をアプリ名（`h3`）→ section
  見出し（`h4`）→ 関連連携カードの表題（`h5`）まで 1 段ずつ下げた。カード
  表題は `card::title`（`<h3>` 固定）を使うと section 見出しより上位の
  階層になってしまうため、`heading(H5, ..)` を明示的に使う（`card::title`
  は一切出力しない）。作成導線カードの表題は「関連する連携」と対等な
  独立導線として section 見出しと同じ `h4` に揃える（`create_cta_section()`
  実装の `HeadingLevel::H4` 呼び出しに合わせる）。
- 関連連携カードを充実させた: `card::header` にロゴ + アプリ名（`h5`）、
  `card::body` に分類バッジ（Subtle）+ 説明文、`card::footer` に外部
  リンクを配置する。3 件のデータは両版で共有する。
- 関連連携・作成導線の外部リンクは実在 URL（本フレームワークリポジトリ）
  へ統一している（変更なし）。
- 狭幅（コンテナ幅 40rem 未満）では、(1) 操作ボタン群の折り返し、(2) メタ
  情報の縦積み、(3) 関連連携 grid の 1 列化、の 3 点を切り替える
  （`@container`、変更なし）。
- `_/blocks-intake/` の対応ファイルは本イシュー着手時点で本 worktree に
  存在しないため、対応表 ID（R0249・R0256）のみを記載しています。
- ブラウザでの実機確認（40rem 前後の幅切替・ライト/ダーク表示）は
  サンドボックス環境の制約により未実施です。

関連情報: [Badge](../themes/badge.md) / [Button](../themes/button.md) /
[Link](../themes/link.md) / [List](../themes/list.md) /
[Separator](../themes/separator.md) / [Card](../themes/card.md) /
[Heading](../themes/heading.md) / [Image](../themes/image.md)
