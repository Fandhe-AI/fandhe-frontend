//! `settings-integrations-grid` block（イシュー #2991。Application /
//! Settings カテゴリ）。連携アプリを 2 列のカードグリッドで並べ、集約元
//! 5 件（主参照 R0244）の差分を 3 版として並記する。`_/blocks-intake/` の
//! 対応ファイルは本イシュー着手時点で本 worktree に存在しないため、
//! 原稿・本コメントには対応表 ID のみを記す（`settings-billing-overview`
//! 〔イシュー #2984〕・`profile-detail-datalist`〔イシュー #2937〕と同じ
//! 扱い）。
//!
//! # 使用部品
//!
//! `card` / `badge` / `button` / `switch` / `link` / `image` / `heading` の
//! 7 部品を合成する（[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//! 新しい UI 部品は追加しない。
//!
//! # 3 版と集約元 ID の対応
//!
//! - **版 A（[`version_grouped_by_status`]）**: R0244（代表構成）を主参照
//!   とし、「接続済み」「未接続」のグループ見出しで分ける。R0245（「表示」
//!   ボタン付き 2 列）は接続操作ボタンに加え「表示」ボタンを並べることで、
//!   R0246（説明文 3 行省略）は `card::description` へ 3 行 `line-clamp`
//!   を適用することでそれぞれ吸収する。
//! - **版 B（[`version_grouped_by_category`]）**: R0247（カテゴリ別 +
//!   スイッチ付き）に対応。接続状態を接続ボタンではなく [`switch::root`]
//!   で表す。
//! - **版 C（[`version_marketplace`]）**: R0253（導入数 + 認証済み印）に
//!   対応。導入数テキストと「認証済み」バッジ（[`ColorPalette::Info`]）を
//!   添え、操作ボタンを「インストール」にする。
//!
//! # switch を静的固定表示にする理由
//!
//! docs サイトは無 JS のため、版 B の switch は初期状態を固定した静的表示
//! とする。[`fandhe_frontend_pre_styled_ui::switch::SwitchProps::disabled`]
//! を `true` にする（`ai_chat_playground::param_switch` と同型の判断）。
//! `readonly` は併用しない（native トグル操作自体を抑止する意味は持たず、
//! `disabled` 単独で「操作不能な固定表示」の意図を過不足なく表せるため）。
//!
//! # `<form>` を使わない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない静的表示のみで、送信処理・送信先は一切持たない。操作ボタンは
//! `button::button` の既定 `type="button"` のまま用いる。
//!
//! # 死リンクを避ける（`href="#"` を使わない）
//!
//! 各カードのリンクは実在する外部 URL（`https://github.com/Fandhe-AI/fandhe-frontend`、
//! marketing 系 footer block の `REPO` 定数と同じプレースホルダー実践）を
//! 指す。架空データのためアプリ個別の詳細ページを持たず、全カード共通で
//! 同じ URL へ遷移する。リンク名は「詳細を見る」ではなく実際の遷移先に
//! 合わせて「リポジトリを見る」とし、アプリ個別ページへ遷移するという
//! 誤った期待を与えない（PR #3442 レビュー指摘の是正、Codex P2）。
//!
//! # `class` と `data-*` の使い分け
//!
//! [`fandhe_frontend_pre_styled_ui::card::root`]・
//! [`fandhe_frontend_pre_styled_ui::badge::badge`]・
//! [`fandhe_frontend_pre_styled_ui::button::button`]・
//! [`fandhe_frontend_pre_styled_ui::switch::root`]・
//! [`fandhe_frontend_pre_styled_ui::link::root`]・
//! [`fandhe_frontend_pre_styled_ui::image::image`] はいずれも
//! `drop_class_attr` で呼び出し側 `class` を除去してから内部 variant
//! クラスと合成するため、これらへの CSS フックは `data-*` 属性で渡す
//! （`data-blocks-settings-integrations-grid-*`）。レイアウト用ラッパー・
//! `card::body`/`description` は素の
//! `class="blocks-settings-integrations-grid-*"` を使う。
//!
//! # 狭幅の 1 列化は `@container`
//!
//! Demo 枠の幅はビューポート幅と一致しないため、`@container`（コンテナ
//! クエリ）で判定する（`settings_billing_overview`・`profile_detail_datalist`
//! 等と同型のパターン）。
//!
//! # ダミー素材について
//!
//! アプリ名は `crate::blocks::dummy_assets::COMPANY_NAMES`（架空社名 6 件）
//! を流用し、説明文・カテゴリ・導入数・認証済み状態は本ファイル内の架空値
//! である。ロゴは `dummy_assets::LOGO_SRC`（抽象バッジ SVG、実在ブランド
//! ロゴではない）を全カード共通で使う。実在の企業・サービス名・人物・PII
//! は含まない。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
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
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/settings-integrations-grid/",
    title: "settings-integrations-grid",
    category: BlockCategory::Settings,
    rust_source: "crates/docs-site/src/blocks/application/settings/settings_integrations_grid.rs",
    demo_class: "blocks-settings-integrations-grid",
    parts: &[
        Part {
            label: "Card",
            path: "/themes/card/",
        },
        Part {
            label: "Badge",
            path: "/themes/badge/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Switch",
            path: "/themes/switch/",
        },
        Part {
            label: "Link",
            path: "/themes/link/",
        },
        Part {
            label: "Image",
            path: "/themes/image/",
        },
        Part {
            label: "Heading",
            path: "/themes/heading/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `settings_integrations_grid` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節と同型）。
const LAYOUT_CSS: &str = "\
.blocks-settings-integrations-grid-stack {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-10);\n  container-type: inline-size;\n  container-name: blocks-settings-integrations-grid;\n}\n\
.blocks-settings-integrations-grid-version {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n}\n\
.blocks-settings-integrations-grid-group {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-settings-integrations-grid-grid {\n  display: grid;\n  grid-template-columns: repeat(2, minmax(0, 1fr));\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-settings-integrations-grid-identity {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-3);\n}\n\
[data-scope=\"image\"][data-part=\"root\"][data-blocks-settings-integrations-grid-logo] {\n  inline-size: 2.5rem;\n  block-size: 2.5rem;\n  max-inline-size: none;\n  flex-shrink: 0;\n}\n\
.blocks-settings-integrations-grid-description-clamp {\n  display: -webkit-box;\n  -webkit-line-clamp: 3;\n  -webkit-box-orient: vertical;\n  overflow: hidden;\n}\n\
.blocks-settings-integrations-grid-footer-row {\n  display: flex;\n  align-items: center;\n  justify-content: space-between;\n  gap: var(--fandhe-space-3);\n  flex: 1;\n  min-width: 0;\n}\n\
.blocks-settings-integrations-grid-actions {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-settings-integrations-grid-installs {\n  font-size: var(--fandhe-font-font-size-sm);\n  color: var(--fandhe-color-fg-muted);\n}\n\
.blocks-settings-integrations-grid-marketplace-meta {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n  margin-top: var(--fandhe-space-2);\n}\n\
@container blocks-settings-integrations-grid (max-width: 36rem) {\n  \
.blocks-settings-integrations-grid-grid {\n    grid-template-columns: 1fr;\n  }\n\
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
            "data-scope=\"card\"",
            "data-scope=\"badge\"",
            "data-scope=\"button\"",
            "data-scope=\"switch\"",
            "data-scope=\"link\"",
            "data-scope=\"image\"",
            "data-scope=\"heading\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
    }

    #[test]
    fn demo_has_eighteen_cards_across_three_versions() {
        let html = demo_html();
        assert_eq!(
            html.matches("data-blocks-settings-integrations-grid-card=\"\"")
                .count(),
            18,
            "3 versions x 6 integrations should render 18 cards"
        );
    }

    #[test]
    fn no_form_dead_link_or_data_uri() {
        let html = demo_html();
        assert!(!html.contains("<form"));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("src=\"data:"));
        assert!(!html.contains("<script"));
        assert!(!html.contains("type=\"submit\""));
    }

    #[test]
    fn buttons_use_type_button_only() {
        let html = demo_html();
        let button_count = html.matches("<button").count();
        assert!(button_count > 0);
        assert_eq!(html.matches("type=\"button\"").count(), button_count);
    }

    #[test]
    fn version_b_switches_are_disabled_static() {
        let html = demo_html();
        assert_eq!(
            html.matches("data-scope=\"switch\" data-part=\"hidden-input\"")
                .count(),
            6,
            "version B should render exactly 6 switches (one per integration)"
        );
        // 静的固定表示のため hidden-input は必ず disabled を持つ。
        let hidden_input_positions: Vec<_> = html
            .match_indices("data-scope=\"switch\" data-part=\"hidden-input\"")
            .collect();
        for (start, _) in hidden_input_positions {
            let window_end = (start + 400).min(html.len());
            assert!(
                html[start..window_end].contains("disabled"),
                "switch hidden-input should be disabled for static display"
            );
        }
    }

    #[test]
    fn action_buttons_name_their_integration() {
        let html = demo_html();
        // 属性値は既定エスケープを経るため、比較側も `&` をエスケープする。
        let escaped = |name: &str| name.replace('&', "&amp;");
        for app in super::INTEGRATIONS {
            for label in ["表示", "インストール"] {
                let aria = format!("aria-label=\"{} を{label}\"", escaped(app.name));
                assert!(html.contains(&aria), "missing {aria}");
            }
        }
        for app in super::INTEGRATIONS {
            let aria = if app.connected {
                format!("aria-label=\"{} の連携を解除\"", escaped(app.name))
            } else {
                format!("aria-label=\"{} に接続\"", escaped(app.name))
            };
            assert!(html.contains(&aria), "missing {aria}");
        }
        // 操作ボタン 18 個（版 A 12・版 C 6）+ 版 B の switch 6 個がすべて
        // アプリ名入りの aria-label を持つ。
        assert_eq!(html.matches("<button").count(), 18);
        assert_eq!(html.matches("aria-label=\"").count(), 24);
    }

    #[test]
    fn demo_labels_each_version_and_groups_footer_actions() {
        let html = demo_html();
        for title in [
            "版 A: 接続状態別",
            "版 B: カテゴリ別",
            "版 C: マーケットプレイス",
        ] {
            assert!(html.contains(title), "missing version heading {title}");
        }
        assert_eq!(
            html.matches("class=\"blocks-settings-integrations-grid-actions\"")
                .count(),
            18
        );
        assert!(LAYOUT_CSS.contains(
            "[data-scope=\"image\"][data-part=\"root\"][data-blocks-settings-integrations-grid-logo]"
        ));
    }

    #[test]
    fn layout_css_is_safe_and_stacks_on_narrow_container() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("container-type: inline-size;"));
        assert!(
            LAYOUT_CSS.contains("@container blocks-settings-integrations-grid (max-width: 36rem)")
        );
    }
}
