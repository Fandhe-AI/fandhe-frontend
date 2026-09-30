# settings-integrations-grid

連携アプリを 2 列のカードグリッドで並べる設定画面向けブロックです。各カードに
ロゴ・アプリ名・説明・接続状態バッジ・接続操作を配置し、`card` / `badge` /
`button` / `switch` / `link` / `image` / `heading` の 7 部品を合成します。
Blocks は既存部品の合成例であり、新しい UI 部品は追加しません。

主参照は対応表 ID R0244（代表構成）です。集約元 5 件（R0244〜R0247・R0253、
`_/blocks-intake/` の対応ファイルは本 worktree に存在しないため対応表 ID の
みを記載）の差分は、以下の 3 版として並記します。

- **版 A（接続済み/未接続のグループ見出し）**: R0244 主参照 + R0245（「表示」
  ボタン併記の 2 列）・R0246（説明文 3 行省略）を吸収
- **版 B（カテゴリ見出し + switch）**: R0247 対応
- **版 C（マーケットプレイス風: 導入数 + 認証済み印）**: R0253 対応

アプリ名・説明文・カテゴリ・導入数・認証済み状態はすべて架空のデータであり、
実在の企業・サービス名・人物・PII は含みません。ロゴ画像は全カード共通の
抽象バッジ SVG（ビルド時生成のプレースホルダー）です。

本 Demo は無 JS の静的表示のみであり、`<form>` を含みません。版 B の switch
は接続操作を表す静的固定表示（`disabled: true`）です。

## Rust コード

```rust
use crate::blocks::dummy_assets::{self, COMPANY_NAMES};
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{badge, BadgeProps, BadgeVariant};
use fandhe_frontend_pre_styled_ui::button::{button, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardVariant};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps};
use fandhe_frontend_pre_styled_ui::image::{image, ImageProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::recipe::{ColorPalette, Size};
use fandhe_frontend_pre_styled_ui::switch::{self, SwitchProps};

/// 「リポジトリを見る」リンクの遷移先（本 block 専用のプレースホルダー実践、
/// モジュール doc「死リンクを避ける」節参照）。
const DETAIL_URL: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// 連携アプリ 1 件分のデータ（アプリ名・説明・接続状態・カテゴリ・
/// 導入数・認証済みフラグ、いずれも架空値）。
struct Integration {
    name: &'static str,
    description: &'static str,
    connected: bool,
    category: &'static str,
    installs: &'static str,
    verified: bool,
}

/// 架空の連携アプリ 6 件（[`COMPANY_NAMES`] を名前に流用）。接続済み/
/// 未接続 3 件ずつ、カテゴリは「コミュニケーション」「データ連携」の
/// 2 分類（版 A/B/C いずれも同一データセットを参照する）。
const INTEGRATIONS: &[Integration] = &[
    Integration {
        name: COMPANY_NAMES[0],
        description: "チームチャットへ通知を自動で集約する連携です。未読の見落としを防ぎます。",
        connected: true,
        category: "コミュニケーション",
        installs: "3,200+",
        verified: true,
    },
    Integration {
        name: COMPANY_NAMES[1],
        description:
            "ファイルをクラウドストレージへ自動同期する連携です。共有リンクも自動更新します。",
        connected: true,
        category: "データ連携",
        installs: "1,800+",
        verified: true,
    },
    Integration {
        name: COMPANY_NAMES[2],
        description: "カレンダーの予定変更をチームの共有スケジュールへ即座に反映します。",
        connected: false,
        category: "コミュニケーション",
        installs: "950+",
        verified: false,
    },
    Integration {
        name: COMPANY_NAMES[3],
        description: "タスクの進捗をかんばんボードへ同期し、担当者の負荷を可視化します。",
        connected: false,
        category: "コミュニケーション",
        installs: "640+",
        verified: false,
    },
    Integration {
        name: COMPANY_NAMES[4],
        description: "請求データを会計ソフトへ自動連携し、月次締め作業の手入力を削減します。",
        connected: true,
        category: "データ連携",
        installs: "2,100+",
        verified: true,
    },
    Integration {
        name: COMPANY_NAMES[5],
        description: "在庫情報を出荷管理システムへ連携し、欠品リスクを早期に検知します。",
        connected: false,
        category: "データ連携",
        installs: "410+",
        verified: false,
    },
];

/// カードロゴ（全カード共通の抽象バッジ SVG、装飾のため `alt` は空。
/// アプリ名は隣接する `heading` が可視テキストとして担う）。
fn logo() -> Node {
    image(
        &ImageProps::new(dummy_assets::LOGO_SRC, ""),
        vec![("data-blocks-settings-integrations-grid-logo", "")],
    )
}

/// 接続状態バッジ（接続済み: Success Subtle・未接続: Neutral Subtle）。
fn status_badge(connected: bool) -> Node {
    let (palette, label) = if connected {
        (ColorPalette::Success, "接続済み")
    } else {
        (ColorPalette::Neutral, "未接続")
    };
    badge(
        &BadgeProps {
            variant: BadgeVariant::Subtle,
            palette,
            ..BadgeProps::default()
        },
        vec![],
        vec![text(label)],
    )
}

/// 接続/解除ボタン（未接続: Outline「接続」・接続済み: Ghost「解除」）。
fn connect_button(app: &Integration) -> Node {
    let (variant, label, aria_label) = if app.connected {
        (
            ButtonVariant::Ghost,
            "解除",
            format!("{} の連携を解除", app.name),
        )
    } else {
        (
            ButtonVariant::Outline,
            "接続",
            format!("{} に接続", app.name),
        )
    };
    action_button(variant, label, &aria_label)
}

/// カードの操作ボタン（可視ラベルは全カード共通のため、`aria-label` に
/// アプリ名を含めて支援技術が操作対象を区別できるようにする。可視ラベルを
/// そのまま含めて WCAG 2.5.3 Label in Name も満たす。
/// `settings_integrations_list` と同じ扱い、PR #3442 レビュー指摘の是正）。
fn action_button(variant: ButtonVariant, label: &str, aria_label: &str) -> Node {
    button(
        &ButtonProps {
            variant,
            size: Size::Sm,
            ..ButtonProps::default()
        },
        vec![("aria-label", aria_label)],
        vec![text(label)],
    )
}

/// 「リポジトリを見る」リンク（[`DETAIL_URL`] を指す、死リンクにしない）。
/// 全カード共通で同じ URL へ遷移するため、リンク名は遷移先の実体
/// （リポジトリ）に合わせる（モジュール doc「死リンクを避ける」節参照）。
fn detail_link() -> Node {
    link::root(
        DETAIL_URL,
        &LinkProps {
            external: true,
            ..LinkProps::default()
        },
        vec![],
        vec![text("リポジトリを見る")],
    )
}

/// 全版共通のカード footer（左: リンク・右: 操作群）。操作が複数でも
/// `-actions` でまとめて右端へ寄せ、`space-between` が要素間へ均等に
/// 余白を配らないようにする（PR #3442 レビュー指摘の是正）。
fn card_footer_row(actions: Vec<Node>) -> Node {
    div(
        vec![("class", "blocks-settings-integrations-grid-footer-row")],
        vec![
            detail_link(),
            div(
                vec![("class", "blocks-settings-integrations-grid-actions")],
                actions,
            ),
        ],
    )
}

/// ロゴ + アプリ名見出しの行（見出しレベルは版の見出し階層に合わせて
/// 呼び出し側が渡す）。
fn identity(app: &Integration, level: HeadingLevel) -> Node {
    div(
        vec![("class", "blocks-settings-integrations-grid-identity")],
        vec![
            logo(),
            heading(
                level,
                &HeadingProps::default(),
                vec![],
                vec![text(app.name)],
            ),
        ],
    )
}

/// 版 A のカード（R0245「表示」ボタン併記・R0246 説明文 3 行省略を含む）。
fn card_grouped_by_status(app: &Integration) -> Node {
    let view_button = action_button(
        ButtonVariant::Plain,
        "表示",
        &format!("{} を表示", app.name),
    );
    card::root(
        CardVariant::Outline,
        vec![("data-blocks-settings-integrations-grid-card", "")],
        vec![
            card::header(
                vec![("data-has-action", "")],
                vec![
                    identity(app, HeadingLevel::H5),
                    card::action(vec![], vec![status_badge(app.connected)]),
                ],
            ),
            card::body(
                vec![],
                vec![card::description(
                    vec![(
                        "class",
                        "blocks-settings-integrations-grid-description-clamp",
                    )],
                    vec![text(app.description)],
                )],
            ),
            card::footer(
                vec![],
                vec![card_footer_row(vec![view_button, connect_button(app)])],
            ),
        ],
    )
}

/// 版 B のカード（R0247。接続操作を switch で表す静的固定表示）。
fn card_grouped_by_category(app: &Integration) -> Node {
    let switch_props = SwitchProps {
        disabled: true,
        ..SwitchProps::default()
    };
    let status_label = if app.connected {
        "接続中"
    } else {
        "未接続"
    };
    let integration_switch = switch::root(
        Size::Sm,
        ColorPalette::Accent,
        app.connected,
        &switch_props,
        vec![],
        vec![
            switch::label(
                app.connected,
                &switch_props,
                vec![],
                vec![text(status_label)],
            ),
            // アクセシブルネームにアプリ名を含める（`root` の `<label>` に
            // よる暗黙の関連付けは `switch::label` の可視テキスト
            // 「接続中」/「未接続」のみを拾い、隣接する `heading`（アプリ名）
            // は関連付け対象外のため、支援技術では複数カード間で状態の
            // 主体を識別できない。`aria-label` で明示上書きして区別する
            // （PR #3442 レビュー指摘の是正、Codex P2）。
            switch::hidden_input(
                "blocks-settings-integrations-grid-toggle",
                app.name,
                app.connected,
                &switch_props,
                vec![("aria-label", &format!("{}: {status_label}", app.name))],
            ),
            switch::control(
                app.connected,
                &switch_props,
                vec![],
                vec![switch::thumb(app.connected, &switch_props, vec![], vec![])],
            ),
        ],
    );

    card::root(
        CardVariant::Outline,
        vec![("data-blocks-settings-integrations-grid-card", "")],
        vec![
            card::header(vec![], vec![identity(app, HeadingLevel::H5)]),
            card::body(
                vec![],
                vec![card::description(vec![], vec![text(app.description)])],
            ),
            card::footer(vec![], vec![card_footer_row(vec![integration_switch])]),
        ],
    )
}

/// 版 C のカード（R0253。導入数テキスト + 認証済みバッジ、操作は
/// 「インストール」ボタン）。
fn card_marketplace(app: &Integration) -> Node {
    let install_button = action_button(
        ButtonVariant::Solid,
        "インストール",
        &format!("{} をインストール", app.name),
    );

    let mut badges = vec![div(
        vec![("class", "blocks-settings-integrations-grid-installs")],
        vec![text(format!("導入数 {}", app.installs))],
    )];
    if app.verified {
        badges.push(badge(
            &BadgeProps {
                variant: BadgeVariant::Subtle,
                palette: ColorPalette::Info,
                ..BadgeProps::default()
            },
            vec![],
            vec![text("認証済み")],
        ));
    }

    card::root(
        CardVariant::Outline,
        vec![("data-blocks-settings-integrations-grid-card", "")],
        vec![
            card::header(vec![], vec![identity(app, HeadingLevel::H4)]),
            card::body(
                vec![],
                vec![
                    card::description(vec![], vec![text(app.description)]),
                    div(
                        vec![(
                            "class",
                            "blocks-settings-integrations-grid-marketplace-meta",
                        )],
                        badges,
                    ),
                ],
            ),
            card::footer(vec![], vec![card_footer_row(vec![install_button])]),
        ],
    )
}

/// グループ見出し（H4）+ カードグリッドの合成（版 A/B 共通ヘルパ）。
fn group(title: &str, cards: Vec<Node>) -> Node {
    div(
        vec![("class", "blocks-settings-integrations-grid-group")],
        vec![
            heading(
                HeadingLevel::H4,
                &HeadingProps::default(),
                vec![],
                vec![text(title)],
            ),
            div(
                vec![("class", "blocks-settings-integrations-grid-grid")],
                cards,
            ),
        ],
    )
}

/// 版見出し（H3）+ 本体の合成。Demo 節の `h2` 直下で 3 版を見分けられる
/// ようにする（PR #3442 レビュー指摘の是正）。見出し階層は 版（H3）→
/// グループ（H4）→ カード（H5）で、グループを持たない版 C はカードを H4
/// とする。
fn version(title: &str, body: Vec<Node>) -> Node {
    div(
        vec![("class", "blocks-settings-integrations-grid-version")],
        std::iter::once(heading(
            HeadingLevel::H3,
            &HeadingProps::default(),
            vec![],
            vec![text(title)],
        ))
        .chain(body)
        .collect(),
    )
}

/// 版 A: 接続済み/未接続でグループ見出しを分ける構成（R0244 主参照 +
/// R0245/R0246 を吸収）。
fn version_grouped_by_status() -> Node {
    let (connected, not_connected): (Vec<&Integration>, Vec<&Integration>) =
        INTEGRATIONS.iter().partition(|app| app.connected);

    version(
        "版 A: 接続状態別",
        vec![
            group(
                "接続済み",
                connected.into_iter().map(card_grouped_by_status).collect(),
            ),
            group(
                "未接続",
                not_connected
                    .into_iter()
                    .map(card_grouped_by_status)
                    .collect(),
            ),
        ],
    )
}

/// 版 B: カテゴリ見出し + switch で接続状態を表す構成（R0247）。
fn version_grouped_by_category() -> Node {
    let categories = ["コミュニケーション", "データ連携"];
    let groups = categories.map(|category| {
        let cards = INTEGRATIONS
            .iter()
            .filter(|app| app.category == category)
            .map(card_grouped_by_category)
            .collect();
        group(category, cards)
    });

    version("版 B: カテゴリ別", groups.into())
}

/// 版 C: マーケットプレイス風（導入数 + 認証済み印、R0253）。グループ
/// 見出しを持たず単一グリッドで並べる。
fn version_marketplace() -> Node {
    version(
        "版 C: マーケットプレイス",
        vec![div(
            vec![("class", "blocks-settings-integrations-grid-grid")],
            INTEGRATIONS.iter().map(card_marketplace).collect(),
        )],
    )
}

/// `settings-integrations-grid` の Demo 本体（3 版を縦に並記）。呼び出し
/// ごとに同一の `Node` を返す純関数。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-settings-integrations-grid-stack")],
        vec![
            version_grouped_by_status(),
            version_grouped_by_category(),
            version_marketplace(),
        ],
    )
}
```

## 原案差分メモ

- **版 A（R0244 主参照 + R0245/R0246）**: 接続済み/未接続でグループ見出しを
  分け、各カードに接続操作ボタンに加えて「表示」ボタンを併記します
  （R0245）。説明文は 3 行で省略します（R0246、`-webkit-line-clamp`）。
- **版 B（R0247）**: カテゴリ見出し（コミュニケーション/データ連携）で分け、
  接続状態の表現を接続ボタンではなく switch にします。docs サイトは無 JS
  のため switch は初期状態を固定した静的表示です（`disabled: true`）。
- **版 C（R0253）**: グループ見出しを持たない単一グリッドで、導入数テキスト
  と「認証済み」バッジ（該当するアプリのみ）を添え、操作ボタンを
  「インストール」にします。
- Demo では各版に版見出し（「版 A: 接続状態別」等）を付けます。各カードの
  操作ボタンは可視ラベルが全カード共通のため、アクセシブルネーム
  （`aria-label`）にアプリ名を含めます。
- 各カードの「リポジトリを見る」リンクは、架空データのため実在の詳細ページを
  持たず全カード共通で同じ URL へ遷移します。`href="#"` の死リンクにはせず、
  marketing 系 footer block と同じプレースホルダー実践（フレームワークの
  リポジトリ URL）を使い、リンク名も実際の遷移先に合わせています。
- 狭幅（コンテナ幅 36rem 未満）ではカードグリッドが 1 列へ切り替わります
  （`@container` によるコンテナクエリ判定）。

関連情報: [Card](../themes/card.md) / [Badge](../themes/badge.md) /
[Button](../themes/button.md) / [Switch](../themes/switch.md) /
[Link](../themes/link.md) / [Image](../themes/image.md) /
[Heading](../themes/heading.md)
