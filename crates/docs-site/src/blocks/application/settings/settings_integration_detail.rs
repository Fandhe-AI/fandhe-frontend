//! `settings-integration-detail` block（イシュー #2989。親 #2988
//! 「連携アプリの詳細画面」の前半。骨格・主要領域のみを実装し、状態違いの
//! 並記〔未接続/接続済み〕・利点領域・原稿の原案差分メモ仕上げは後続
//! #2990 へ送る）。Application / Settings カテゴリ 2 件目の block
//! （1 件目は `settings_billing_overview.rs`、イシュー #2984）。
//!
//! # 使用部品
//!
//! `badge` / `button` / `link` / `list` / `separator` / `card` / `heading` /
//! `image` の 8 部品を合成する（[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//! 親イシューの想定 7 部品に対し、ロゴ表示のため `image` を 1 件追加した
//! （`page_heading_avatar.rs::logo_image` が会社ロゴに `image::image` +
//! `dummy_assets::LOGO_SRC` を使う既存判断に揃える）。新しい UI 部品は
//! 追加しない。
//!
//! # 対応表 ID（`_/blocks-intake/` 不在の記録）
//!
//! 主参照 R0249（代表構成）・集約元 R0256（利点 + 関連連携 + 作成導線）。
//! `_/blocks-intake/` の対応ファイルは本イシュー着手時点で本 worktree に
//! 存在しないため、原稿・本コメントには対応表 ID のみを記す
//! （`settings_billing_overview.rs`・`profile_detail_datalist.rs` と同じ
//! 扱い）。R0256 由来の「利点」領域・関連連携カードの充実は #2990 へ送る
//! （下記「本 block が持たない領域（#2990 へ送る）」節参照）。
//!
//! # 版は 1 つのみ（状態並記は #2990）
//!
//! 他の block（`profile_detail_datalist.rs` 等）が持つ「複数版の並記」は
//! 本 block では行わない。接続済み状態のみを描画し、未接続状態との並記は
//! #2990 で追加する。
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
//! （素の `<div>`/`<dl>`/`<section>`）は素の
//! `class="blocks-settings-integration-detail-*"` を使う
//! （`settings_billing_overview.rs` と同型の判断）。
//!
//! # メタ情報は `<dl>` + ラベル・値ペアを包む `<div>` で組む
//!
//! HTML5 の `<dl>` content model は「1 個以上の `<dt>` に 1 個以上の
//! `<dd>` が続く組」を `<div>` で包むことを許容するため、横並び・狭幅での
//! 縦積み切り替えを 1 ペア単位の `<div>`（`meta_item`）で行う
//! （`data_list` 部品は使わず素の `<dl>` を直接組み立てる、モジュール doc
//! 冒頭「使用部品」に `data-list` を含めない理由）。
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
//! `button::button` の既定 `type="button"` のまま用いる。
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
//! # 本 block が持たない領域（#2990 へ送る）
//!
//! - 未接続状態との並記（状態違いの複数版）
//! - R0256 由来の「利点」領域の追加
//! - 関連連携カードの充実（現状は骨格として 3 枚の最小構成のみ）
//! - 原稿「原案差分メモ」の集約元差分の仕上げ・ブラウザ実機確認記録
//!
//! # ダミー素材について
//!
//! アプリ名・開発元・分類・最終同期日時・対応プラン・機能一覧・導入手順・
//! 関連連携の文言はすべて本ファイル内の架空値であり、実在の企業・製品・
//! 商標・PII を含まない。ロゴ画像はビルド時生成の同梱 SVG
//! （[`dummy_assets::LOGO_SRC`]）を使う（外部 URL・`data:` URI は使わない）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
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
        "「設定を開く」から連携アプリの認証を完了する",
        "同期するプロジェクトを選択する",
        "同期頻度（リアルタイム・1 時間ごと・手動）を選ぶ",
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
.blocks-settings-integration-detail-stack {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n  container-type: inline-size;\n  container-name: blocks-settings-integration-detail;\n}\n\
.blocks-settings-integration-detail-header {\n  display: flex;\n  flex-wrap: wrap;\n  align-items: flex-start;\n  gap: var(--fandhe-space-4);\n}\n\
img[data-scope=\"image\"][data-blocks-settings-integration-detail-logo] {\n  width: 3rem;\n  height: 3rem;\n  flex-shrink: 0;\n}\n\
.blocks-settings-integration-detail-identity {\n  display: flex;\n  flex: 1 1 16rem;\n  flex-direction: column;\n  gap: var(--fandhe-space-1);\n  min-width: 0;\n}\n\
.blocks-settings-integration-detail-title-row {\n  display: flex;\n  flex-wrap: wrap;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-settings-integration-detail-title-row [data-scope=\"heading\"] {\n  border-top: none;\n  padding-top: 0;\n}\n\
.blocks-settings-integration-detail-identity .blocks-settings-integration-detail-summary {\n  margin: 0;\n  color: var(--fandhe-color-fg-muted);\n}\n\
.blocks-settings-integration-detail-actions {\n  display: flex;\n  flex-wrap: wrap;\n  gap: var(--fandhe-space-2);\n  margin-inline-start: auto;\n}\n\
.blocks-settings-integration-detail-meta {\n  margin: 0;\n  display: flex;\n  flex-wrap: wrap;\n  gap: var(--fandhe-space-6);\n}\n\
.blocks-settings-integration-detail-meta-item {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1);\n  min-width: 0;\n}\n\
.blocks-settings-integration-detail-meta-item dt {\n  margin: 0;\n  font-size: var(--fandhe-font-font-size-sm);\n  color: var(--fandhe-color-fg-muted);\n}\n\
.blocks-settings-integration-detail-meta-item dd {\n  margin: 0;\n}\n\
.blocks-settings-integration-detail-section {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-settings-integration-detail-related-grid {\n  display: grid;\n  grid-template-columns: repeat(3, 1fr);\n  gap: var(--fandhe-space-4);\n}\n\
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
        assert_eq!(html.matches("<h2").count(), 1);
        // section() の見出し 4 個（概要・主な機能・導入手順・関連する連携） +
        // `card::title`（`<h3>` を出力する）4 個（related_card 3 枚 + cta 1 枚）。
        assert_eq!(html.matches("<h3").count(), 8);
        assert_eq!(html.matches("<ol").count(), 1);
        assert_eq!(html.matches("<ul").count(), 1);
        assert_eq!(html.matches("<button").count(), 3);
        assert_eq!(
            html.matches("data-blocks-settings-integration-detail-card=\"\"")
                .count(),
            3
        );
        assert!(html.contains(super::dummy_assets::LOGO_SRC));
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
        assert!(
            LAYOUT_CSS.contains("@container blocks-settings-integration-detail (max-width: 40rem)")
        );
    }

    /// `.docs-content h2`/`.docs-content p`（`site_theme.rs` の文書タイポグラフィ、
    /// `border-top`/`padding-top`/`margin`）がヘッダー識別行（ロゴ + アプリ名）へ
    /// 漏れ出さないことを固定する（Bugbot 指摘、PR #3440）。
    #[test]
    fn header_heading_and_summary_override_docs_content_typography() {
        assert!(LAYOUT_CSS.contains(
            ".blocks-settings-integration-detail-title-row [data-scope=\"heading\"] {\n  border-top: none;\n  padding-top: 0;\n}"
        ));
        assert!(LAYOUT_CSS.contains(
            ".blocks-settings-integration-detail-identity .blocks-settings-integration-detail-summary {\n  margin: 0;"
        ));
    }

    #[test]
    fn logo_image_alt_names_the_app() {
        let html = demo_html();
        assert!(html.contains(&format!(r#"alt="{} のロゴ""#, super::APP_NAME)));
    }
}
