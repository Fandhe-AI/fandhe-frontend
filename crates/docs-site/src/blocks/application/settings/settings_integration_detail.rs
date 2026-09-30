//! `settings-integration-detail` block（イシュー #2989/#2990。親 #2988
//! 「連携アプリの詳細画面」。#2989 で骨格・主要領域を実装し、本イシュー
//! （#2990）で状態違いの並記〔接続済み/未接続〕・利点領域・関連連携カード
//! の充実・原稿の原案差分メモ仕上げを追加した）。Application / Settings
//! カテゴリ 2 件目の block（1 件目は `settings_billing_overview.rs`、
//! イシュー #2984）。
//!
//! # 使用部品
//!
//! `badge` / `button` / `link` / `list` / `separator` / `card` / `heading` /
//! `image` の 8 部品を合成する（[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//! 状態並記・利点領域・関連連携カードの充実はいずれもこの 8 部品の中で
//! 組めるため、`parts` は #2989 から変更しない。新しい UI 部品は追加しない。
//!
//! # 対応表 ID（`_/blocks-intake/` 不在の記録）
//!
//! `_/blocks-intake/` の対応ファイルは本イシュー着手時点で本 worktree に
//! 存在しないため、原稿・本コメントには対応表 ID のみを記す
//! （`settings_billing_overview.rs`・`profile_detail_datalist.rs` と同じ
//! 扱い）。
//!
//! # 2 版と集約元 ID の対応
//!
//! - **版 A（[`version_connected`]）**: R0249（主参照・代表構成）に対応。
//!   接続済み状態のバッジ・操作 2 個（「接続を解除」「設定を開く」）・
//!   最終同期を含むメタ情報・導入手順 section を持つ。
//! - **版 B（[`version_disconnected`]）**: R0256（利点 + 関連連携 + 作成
//!   導線）に対応。未接続状態のバッジ・操作 1 個（「接続する」）・
//!   最終同期を持たないメタ情報・「導入手順」の代わりに「利点」section を
//!   持つ。「導入手順」は接続後にしか意味を持たず、「利点」は接続前の
//!   説明として位置付けが異なるため、section の入れ替えで状態差と集約元差
//!   の両方を Demo から読み取れるようにしている。
//!
//! 関連する連携・作成導線の 2 section は両版で共有する（[`related_integrations_section`]・
//! [`create_cta_section`]）。
//!
//! # 2 版の並記と見出し階層
//!
//! `table_with_toolbar.rs`（複数版並記の先例）と同型で、版キャプション
//! （[`caption`]、`h2`）→ アプリ名（`h3`）→ section 見出し（`h4`）→
//! 関連連携カード・作成導線カードの表題（`h5`）の 4 階層で構造化する。
//! カード表題に [`fandhe_frontend_pre_styled_ui::card::title`]（`<h3>`
//! 固定）を使うと section 見出し（`h4`）より上位の階層になってしまうため、
//! `settings_integrations_grid.rs` が `card::title` を避けて
//! `heading(level, ..)` を使う判断と同型で、本 block も `card::title` を
//! 使わず [`heading`] で `h5` を明示する。
//!
//! # `class` と `data-*` の使い分け
//!
//! [`fandhe_frontend_pre_styled_ui::badge::badge`]・
//! [`fandhe_frontend_pre_styled_ui::button::button`]・
//! [`fandhe_frontend_pre_styled_ui::link::root`]・
//! [`fandhe_frontend_pre_styled_ui::list::root`]・
//! [`fandhe_frontend_pre_styled_ui::card::root`]・
//! [`fandhe_frontend_pre_styled_ui::image::image`] はいずれも
//! `drop_class_attr` で呼び出し側 `class` を除去してから内部 variant
//! クラスと合成するため、これらへの CSS フックは `data-*` 属性で渡す
//! （`data-blocks-settings-integration-detail-*`）。レイアウト用ラッパー
//! （素の `<div>`/`<dl>`/`<section>`）と `card::header`/`body`/`footer`/
//! `description`（variant を持たず `class` をそのまま連結する）は素の
//! `class="blocks-settings-integration-detail-*"` を使う
//! （`settings_billing_overview.rs`・`settings_integrations_grid.rs` と
//! 同型の判断）。
//!
//! # メタ情報は `<dl>` + ラベル・値ペアを包む `<div>` で組む
//!
//! HTML5 の `<dl>` content model は「1 個以上の `<dt>` に 1 個以上の
//! `<dd>` が続く組」を `<div>` で包むことを許容するため、横並び・狭幅での
//! 縦積み切り替えを 1 ペア単位の `<div>`（`meta_item`）で行う
//! （`data_list` 部品は使わず素の `<dl>` を直接組み立てる、モジュール doc
//! 冒頭「使用部品」に `data-list` を含めない理由）。未接続版（[`meta_row`]
//! の `connected: false`）は「最終同期」ペア自体を出力しない（値を
//! 「—」にする代替は採らない。未接続状態では同期が一度も発生していない
//! ため、項目の不在で状態を表す）。
//!
//! # 関連連携カード
//!
//! `card::header` にロゴ（[`logo`]、`image` + `dummy_assets::LOGO_SRC`）+
//! アプリ名（`h5`）、`card::body` に分類バッジ（`Subtle`）+ 説明文、
//! `card::footer` に外部リンクを配置する。ロゴは [`logo`] を名前引数化して
//! ヘッダーと共用し、[`Related`] の `const` 配列（3 件、両版で共有）から
//! [`related_card`] で組み立てる。
//!
//! # 狭幅では操作ボタン群を折り返し・メタ情報を縦積み・関連連携を 1 列にする
//! （`@container`）
//!
//! Demo 枠の幅はビューポート幅と一致しないため、`@container`
//! （コンテナクエリ）で判定する（`profile_detail_datalist.rs`・
//! `settings_billing_overview.rs` と同型のパターン）。[`LAYOUT_CSS`] の
//! ラッパー `.blocks-settings-integration-detail-stack` へ
//! `container-type: inline-size` を宣言し、コンテナ幅が `40rem` 未満の
//! とき、(1) ヘッダー右端の操作ボタン群の `margin-inline-start: 0`
//! を解除して全幅で下段へ折り返し、(2) メタ情報 `<dl>` を
//! `flex-direction: column` へ切り替えて各ラベル・値ペアを見出しの下へ
//! 縦積みにし、(3) 関連連携の grid を 1 列にする。
//!
//! # `<form>` を使わない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない静的表示のみで、送信処理・送信先は一切持たない。操作ボタンは
//! `button::button` の既定 `type="button"` のまま用いる。両版とも静的固定
//! 表示（無 JS）で、状態の切り替え自体はできない。
//!
//! # 関連連携・作成導線のリンクは実在の外部 URL を使う
//!
//! `href="#"` は使わない。`page_heading_avatar.rs`（イシュー #2931 codex
//! レビュー是正）と同型の判断で、可視テキストが遷移先を過大に主張しない
//! よう、関連連携カード・作成導線の外部リンクはいずれも本フレームワーク
//! リポジトリ（[`REPO`]）を指し、ラベルは統一して「GitHub で見る」とする
//! （「詳細を見る」「開発者ドキュメント」のような、実在しない特定ページを
//! 指すと誤認させるラベルは使わない）。
//!
//! # ダミー素材について
//!
//! アプリ名・開発元・分類・最終同期日時・対応プラン・機能一覧・導入手順・
//! 利点・関連連携の文言はすべて本ファイル内の架空値であり、実在の企業・
//! 製品・商標・PII を含まない。ロゴ画像はビルド時生成の同梱 SVG
//! （[`dummy_assets::LOGO_SRC`]）を使う（外部 URL・`data:` URI は使わない）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
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

/// 作成導線（H4 なしの単独カード。「独自の連携を作成」（`h5`）+ 説明 +
/// footer にボタン + 外部リンク、両版で共有。`card::title` を使わない
/// 理由は [`related_card`] rustdoc と同じ）。
fn create_cta_section() -> Node {
    card::root(
        CardProps::default(),
        vec![("data-blocks-settings-integration-detail-cta", "")],
        vec![
            card::body(
                vec![],
                vec![
                    heading(
                        HeadingLevel::H5,
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
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/settings-integration-detail/",
    title: "settings-integration-detail",
    category: BlockCategory::Settings,
    rust_source: "crates/docs-site/src/blocks/application/settings/settings_integration_detail.rs",
    demo_class: "blocks-settings-integration-detail",
    parts: &[
        Part {
            label: "Badge",
            path: "/themes/badge/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Link",
            path: "/themes/link/",
        },
        Part {
            label: "List",
            path: "/themes/list/",
        },
        Part {
            label: "Separator",
            path: "/themes/separator/",
        },
        Part {
            label: "Card",
            path: "/themes/card/",
        },
        Part {
            label: "Heading",
            path: "/themes/heading/",
        },
        Part {
            label: "Image",
            path: "/themes/image/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `settings_integration_detail` 固有のレイアウト規則
/// （`crate::blocks::LAYOUT_CSS` doc「block 固有 CSS の置き場」節と同型）。
const LAYOUT_CSS: &str = "\
.blocks-settings-integration-detail-stack {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-12);\n  container-type: inline-size;\n  container-name: blocks-settings-integration-detail;\n}\n\
.blocks-settings-integration-detail-version {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n}\n\
[data-scope=\"heading\"][data-blocks-settings-integration-detail-caption] {\n  border-top: none;\n  padding-top: 0;\n  margin: 0;\n  color: var(--fandhe-color-fg-muted);\n}\n\
.blocks-settings-integration-detail-header {\n  display: flex;\n  flex-wrap: wrap;\n  align-items: flex-start;\n  gap: var(--fandhe-space-4);\n}\n\
img[data-scope=\"image\"][data-blocks-settings-integration-detail-logo] {\n  width: 3rem;\n  height: 3rem;\n  flex-shrink: 0;\n}\n\
.blocks-settings-integration-detail-identity {\n  display: flex;\n  flex: 1 1 16rem;\n  flex-direction: column;\n  gap: var(--fandhe-space-1);\n  min-width: 0;\n}\n\
.blocks-settings-integration-detail-title-row {\n  display: flex;\n  flex-wrap: wrap;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-settings-integration-detail-title-row [data-scope=\"heading\"] {\n  border-top: none;\n  padding-top: 0;\n  margin: 0;\n}\n\
.blocks-settings-integration-detail-identity .blocks-settings-integration-detail-summary {\n  margin: 0;\n  color: var(--fandhe-color-fg-muted);\n}\n\
.blocks-settings-integration-detail-actions {\n  display: flex;\n  flex-wrap: wrap;\n  gap: var(--fandhe-space-2);\n  margin-inline-start: auto;\n}\n\
.blocks-settings-integration-detail-meta {\n  margin: 0;\n  display: flex;\n  flex-wrap: wrap;\n  gap: var(--fandhe-space-6);\n}\n\
.blocks-settings-integration-detail-meta-item {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1);\n  min-width: 0;\n}\n\
.blocks-settings-integration-detail-meta-item dt {\n  margin: 0;\n  font-size: var(--fandhe-font-font-size-sm);\n  color: var(--fandhe-color-fg-muted);\n}\n\
.blocks-settings-integration-detail-meta-item dd {\n  margin: 0;\n}\n\
.blocks-settings-integration-detail-section {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-settings-integration-detail-section [data-scope=\"heading\"] {\n  margin: 0;\n}\n\
.blocks-settings-integration-detail-related-grid {\n  display: grid;\n  grid-template-columns: repeat(3, 1fr);\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-settings-integration-detail-card-header {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-settings-integration-detail-card-header [data-scope=\"heading\"] {\n  margin: 0;\n}\n\
img[data-scope=\"image\"][data-blocks-settings-integration-detail-card-logo] {\n  width: 2rem;\n  height: 2rem;\n  flex-shrink: 0;\n}\n\
[data-blocks-settings-integration-detail-cta] [data-scope=\"heading\"] {\n  margin: 0;\n}\n\
@container blocks-settings-integration-detail (max-width: 40rem) {\n  \
.blocks-settings-integration-detail-actions {\n    margin-inline-start: 0;\n    width: 100%;\n  }\n  \
.blocks-settings-integration-detail-meta {\n    flex-direction: column;\n  }\n  \
.blocks-settings-integration-detail-related-grid {\n    grid-template-columns: 1fr;\n  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    fn demo_html() -> String {
        render(&demo())
    }

    #[test]
    fn demo_composes_expected_parts() {
        let html = demo_html();
        for scope in [
            "data-scope=\"badge\"",
            "data-scope=\"button\"",
            "data-scope=\"link\"",
            "data-scope=\"list\"",
            "data-scope=\"separator\"",
            "data-scope=\"card\"",
            "data-scope=\"heading\"",
            "data-scope=\"image\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
        // 版キャプション（h2）2 個。
        assert_eq!(html.matches("<h2").count(), 2);
        // アプリ名（h3）2 個（版 A・B 各 1）。
        assert_eq!(html.matches("<h3").count(), 2);
        // section() の見出し（h4）: 版 A（概要・主な機能・導入手順・
        // 関連する連携 = 4）+ 版 B（概要・主な機能・利点・関連する連携 = 4）。
        assert_eq!(html.matches("<h4").count(), 8);
        // 関連連携カード表題（3 枚）+ 作成導線表題（1 枚）を版ごとに
        // heading(H5) で出力（`card::title` は使わない）。
        assert_eq!(html.matches("<h5").count(), 8);
        // 導入手順（番号付き）は版 A のみ。
        assert_eq!(html.matches("<ol").count(), 1);
        // 主な機能（版 A・B 共通）+ 利点（版 B のみ）。
        assert_eq!(html.matches("<ul").count(), 3);
        // 操作ボタン: 版 A（2）+ 版 B（1）+ 作成導線（版ごと 1、計 2）。
        assert_eq!(html.matches("<button").count(), 5);
        assert_eq!(
            html.matches("data-blocks-settings-integration-detail-card=\"\"")
                .count(),
            6
        );
        // ロゴ: ヘッダー（版ごと 1、計 2）+ 関連連携カード（版ごと 3、計 6）。
        assert_eq!(html.matches(super::dummy_assets::LOGO_SRC).count(), 8);
    }

    #[test]
    fn both_states_are_rendered() {
        let html = demo_html();
        for text in [
            "接続済み",
            "未接続",
            "接続する",
            "接続を解除",
            "利点",
            "導入手順",
        ] {
            assert!(html.contains(text), "demo should contain {text}");
        }
        assert_eq!(html.matches("利点").count(), 1);
        assert_eq!(html.matches("導入手順").count(), 1);
        assert_eq!(
            html.matches("data-blocks-settings-integration-detail-caption")
                .count(),
            2
        );
    }

    #[test]
    fn related_cards_have_logo_badge_and_link() {
        let html = demo_html();
        // 版ごとにカード内で image/badge/link が 3 組ずつ、計 8（badge は
        // header のバッジ〔版ごと 1〕分を含む）。
        assert_eq!(html.matches("data-scope=\"image\"").count(), 8);
        assert_eq!(html.matches("data-scope=\"badge\"").count(), 8);
        // link は関連連携カード（版ごと 3）+ 作成導線（版ごと 1）で計 8。
        assert_eq!(html.matches("data-scope=\"link\"").count(), 8);
    }

    #[test]
    fn no_card_title_h3_inside_versions() {
        // `card::title`（`<h3>` 固定）を使わない契約（モジュール doc
        // 「2 版の並記と見出し階層」節参照）。アプリ名の `h3` は
        // `data-part="title"` を持たない素の `heading` 呼び出しのため、
        // `data-part="title"` は一切出力されない。
        let html = demo_html();
        assert!(!html.contains("data-part=\"title\""));
    }

    #[test]
    fn no_form_submit_or_dead_links() {
        let html = demo_html();
        assert!(!html.contains("<form"));
        assert!(!html.contains("type=\"submit\""));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("<script"));
        assert!(!html.contains("src=\"data:"));
    }

    #[test]
    fn layout_css_is_safe_and_stacks_on_narrow_container() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("container-type: inline-size;"));
        assert!(LAYOUT_CSS.contains(".blocks-settings-integration-detail-version"));
        assert!(
            LAYOUT_CSS.contains("@container blocks-settings-integration-detail (max-width: 40rem)")
        );
    }

    /// `.docs-content h2`（`site_theme.rs` の文書タイポグラフィ、
    /// `border-top`/`padding-top`/`margin`）が版キャプションへ漏れ出さない
    /// ことを固定する（`title_row` に対する PR #3440 の対策の版キャプション
    /// 版、モジュール doc「2 版の並記と見出し階層」節参照）。
    #[test]
    fn caption_overrides_docs_content_h2() {
        assert!(LAYOUT_CSS.contains(
            "[data-scope=\"heading\"][data-blocks-settings-integration-detail-caption] {\n  border-top: none;\n  padding-top: 0;\n  margin: 0;"
        ));
    }

    /// `.docs-content h3`/`.docs-content p`（`site_theme.rs` の文書
    /// タイポグラフィ、`margin`）がヘッダー識別行（ロゴ + アプリ名）へ
    /// 漏れ出さないことを固定する（Bugbot 指摘、PR #3440。アプリ名は
    /// #2990 で H2 から H3 へ 1 段下げたが、セレクタ自体は変更不要）。
    #[test]
    fn header_heading_and_summary_override_docs_content_typography() {
        assert!(LAYOUT_CSS.contains(
            ".blocks-settings-integration-detail-title-row [data-scope=\"heading\"] {\n  border-top: none;\n  padding-top: 0;\n  margin: 0;\n}"
        ));
        assert!(LAYOUT_CSS.contains(
            ".blocks-settings-integration-detail-identity .blocks-settings-integration-detail-summary {\n  margin: 0;"
        ));
    }

    /// `.docs-content h4`（`site_theme.rs` の文書タイポグラフィ、
    /// `margin-top: 1.4rem`）が節見出し（概要・主な機能・導入手順・利点・
    /// 関連する連携）へ漏れ出さないことを固定する（Codex 指摘、PR #3440。
    /// section 見出しは #2990 で H3 から H4 へ 1 段下げたが、セレクタ自体は
    /// 変更不要）。
    #[test]
    fn section_heading_overrides_docs_content_h4_margin() {
        assert!(LAYOUT_CSS.contains(
            ".blocks-settings-integration-detail-section [data-scope=\"heading\"] {\n  margin: 0;\n}"
        ));
    }

    /// `.docs-content h5`（`site_theme.rs` の文書タイポグラフィ、
    /// `margin-top: 1.2rem`）が関連連携カード・作成導線カードの表題へ
    /// 漏れ出さないことを固定する。
    #[test]
    fn card_heading_overrides_docs_content_h5_margin() {
        assert!(LAYOUT_CSS.contains(
            ".blocks-settings-integration-detail-card-header [data-scope=\"heading\"] {\n  margin: 0;\n}"
        ));
        assert!(LAYOUT_CSS.contains(
            "[data-blocks-settings-integration-detail-cta] [data-scope=\"heading\"] {\n  margin: 0;\n}"
        ));
    }

    #[test]
    fn logo_image_alt_names_the_app() {
        let html = demo_html();
        assert!(html.contains(&format!(r#"alt="{} のロゴ""#, super::APP_NAME)));
    }
}
