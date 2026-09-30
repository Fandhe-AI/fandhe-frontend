# settings-integrations-list

連携アプリを個人用・組織用の 2 グループに分け、枠付きの行リストで並べる設定画面向け
ブロックです。各行にロゴ・名前・接続状態バッジ・説明・詳細リンク・接続/解除ボタンを
配置します。`badge` / `button` / `separator` / `link` / `image` の 5 部品を合成します。
`image` は使用部品一覧に無いロゴ表示のために追加しました（`page-heading-avatar` と
同じ判断）。Blocks は既存部品の合成例であり、新しい UI 部品は追加しません。

主参照は対応表 ID R0252（代表構成）、集約元は R0251（要望一覧 + 送信フォーム）/
R0254（末尾の空状態）/ R0255（スイッチ展開 + API キー欄）です（`_/blocks-intake/`
の対応ファイルは本 worktree に存在しないため、対応表 ID のみを記載）。アプリ名・
説明文はすべて架空であり、実在の企業・製品・商標・PII を含みません。

本 Demo は無 JS の静的表示のみであり、`<form>` を含みません。詳細リンクは架空アプリの
ため `href="#"` ではなく、本リポジトリへの外部リンク（`rel="noopener noreferrer"` 付き）
を使っています。

**本 block は前半（イシュー #2993）です。** スイッチ展開 + API キー欄・要望送信
フォーム・末尾の空状態は後続のイシュー #2994 で追加します。

## Rust コード

```rust
use crate::blocks::dummy_assets::LOGO_SRC;
use fandhe_frontend_core::{div, h3, li, text, ul, Node};
use fandhe_frontend_pre_styled_ui::badge::{badge, BadgeProps, BadgeVariant};
use fandhe_frontend_pre_styled_ui::button::{button, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::image::{image, ImageFit, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::separator::{separator, SeparatorProps};

/// 詳細リンクの遷移先（架空アプリのため本リポジトリへの外部リンクで代替、
/// モジュール doc「詳細リンクの遷移先」節参照）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// 連携アプリ 1 件分のデータ（名前・説明・接続状態）。
struct Integration {
    name: &'static str,
    description: &'static str,
    connected: bool,
}

/// グループ 1 件分（見出し + 所属アプリ一覧）。
struct Group {
    title: &'static str,
    items: &'static [Integration],
}

/// 個人用・組織用の 2 グループ（各 3 件、接続済み/未接続を混在させ両分岐を
/// Demo に出す）。
const GROUPS: &[Group] = &[
    Group {
        title: "個人用",
        items: &[
            Integration {
                name: "Lattice Notes",
                description: "メモとドキュメントを同期するノートアプリ。",
                connected: true,
            },
            Integration {
                name: "Harbor Calendar",
                description: "予定を共有カレンダーへ反映する。",
                connected: false,
            },
            Integration {
                name: "Pulse Alerts",
                description: "重要な通知をモバイルへ転送する。",
                connected: true,
            },
        ],
    },
    Group {
        title: "組織用",
        items: &[
            Integration {
                name: "Ledger Sync",
                description: "経費データを会計システムへ同期する。",
                connected: false,
            },
            Integration {
                name: "Beacon Chat",
                description: "チームチャットへ更新情報を投稿する。",
                connected: true,
            },
            Integration {
                name: "Quarry Storage",
                description: "添付ファイルをクラウドストレージへ保管する。",
                connected: false,
            },
        ],
    },
];

/// 連携アプリのロゴ画像（共通ダミー画像、`data-*` で CSS フックを渡す）。
/// `image` recipe の base（`[data-scope="image"][data-part="root"]`、詳細度
/// (0,2,0)）に `height: auto`/`max-width: 100%` が乗るため、[`LAYOUT_CSS`]
/// 側は `img[data-scope="image"][data-blocks-settings-integrations-list-logo]`
/// （詳細度 (0,2,1)）で上回る（`page_heading_avatar::logo_image` と同じ
/// 判断、イシュー #2931 Bugbot 指摘の再発形）。
fn logo(name: &str) -> Node {
    image(
        &ImageProps {
            shape: ImageShape::Rounded,
            fit: ImageFit::Contain,
            ..ImageProps::new(LOGO_SRC, &format!("{name} のロゴ"))
        },
        vec![("data-blocks-settings-integrations-list-logo", "")],
    )
}

/// 枠付きリストの 1 行（ロゴ・本文（名前 + 状態バッジ + 説明 + 詳細
/// リンク）・接続/解除操作）。
fn row(item: &Integration) -> Node {
    let status_badge = if item.connected {
        badge(
            &BadgeProps {
                variant: BadgeVariant::Subtle,
                ..BadgeProps::default()
            },
            vec![("data-blocks-settings-integrations-list-status", "")],
            vec![text("接続済み")],
        )
    } else {
        badge(
            &BadgeProps {
                variant: BadgeVariant::Outline,
                ..BadgeProps::default()
            },
            vec![("data-blocks-settings-integrations-list-status", "")],
            vec![text("未接続")],
        )
    };

    let action_button = if item.connected {
        button(
            &ButtonProps {
                variant: ButtonVariant::Outline,
                ..ButtonProps::default()
            },
            vec![("data-blocks-settings-integrations-list-action", "")],
            vec![text("解除")],
        )
    } else {
        button(
            &ButtonProps {
                variant: ButtonVariant::Solid,
                ..ButtonProps::default()
            },
            vec![("data-blocks-settings-integrations-list-action", "")],
            vec![text("接続")],
        )
    };

    let name_row = div(
        vec![("class", "blocks-settings-integrations-list-name")],
        vec![text(item.name), status_badge],
    );

    let description_row = div(
        vec![("class", "blocks-settings-integrations-list-description")],
        vec![
            text(item.description),
            text(" "),
            link::root(
                REPO,
                &LinkProps {
                    external: true,
                    ..LinkProps::default()
                },
                vec![("data-blocks-settings-integrations-list-detail", "")],
                vec![text("詳細を見る")],
            ),
        ],
    );

    let body = div(
        vec![("class", "blocks-settings-integrations-list-body")],
        vec![name_row, description_row],
    );

    let actions = div(
        vec![("class", "blocks-settings-integrations-list-actions")],
        vec![action_button],
    );

    li(
        vec![("class", "blocks-settings-integrations-list-row")],
        vec![logo(item.name), body, actions],
    )
}

/// 1 グループ分（見出し + `ul` 行リスト）。
fn group_section(group: &Group) -> Node {
    div(
        vec![("class", "blocks-settings-integrations-list-group")],
        vec![
            h3(
                vec![("class", "blocks-settings-integrations-list-group-title")],
                vec![text(group.title)],
            ),
            ul(
                vec![("class", "blocks-settings-integrations-list-list")],
                group.items.iter().map(row).collect(),
            ),
        ],
    )
}

/// `settings-integrations-list` の Demo 本体。呼び出しごとに同一の `Node`
/// を返す純関数。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-settings-integrations-list-stack")],
        vec![
            group_section(&GROUPS[0]),
            separator(
                &SeparatorProps::default(),
                vec![("data-blocks-settings-integrations-list-divider", "")],
            ),
            group_section(&GROUPS[1]),
        ],
    )
}
```

## 原案差分メモ

前半（本イシュー）で扱う差分:

- 個人用・組織用の 2 グループへ分け、見出し + 区切り線（`separator`）で分離
- 接続状態を状態バッジ（接続済み: Subtle / 未接続: Outline）と接続/解除ボタンの
  variant 切り替えで表現し、両分岐を Demo に出す
- コンテナ幅が 40rem 未満のとき、接続/解除ボタンを説明の下（2 行目）へ回す

後半（イシュー #2994）へ送る差分:

- R0255: 行をスイッチで展開して表示する API キー欄（`switch` + `clipboard`）
- R0251: 連携要望の一覧 + 送信フォーム（`input-group` + `input`、送信処理は持たない
  静的表示）
- R0254: 末尾の空状態枠（`empty-state`）

関連情報: [Badge](../themes/badge.md) / [Button](../themes/button.md) /
[Separator](../themes/separator.md) / [Link](../themes/link.md) /
[Image](../themes/image.md)
