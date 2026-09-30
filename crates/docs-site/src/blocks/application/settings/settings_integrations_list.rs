//! `settings-integrations-list` block（イシュー #2993、親 #2992。Application /
//! Settings カテゴリ）。枠付き行リストで連携アプリを個人用・組織用の 2 グ
//! ループに分け、各行にロゴ・名前・接続状態・説明・詳細リンク・接続/解除
//! 操作を並べる骨格と主要領域のみを実装する。主参照 R0252（代表構成）、
//! 集約元 R0251（要望一覧 + 送信フォーム）/ R0254（末尾の空状態）/ R0255
//! （スイッチ展開 + API キー欄）。`_/blocks-intake/` の対応ファイルは本
//! イシュー着手時点で本 worktree に存在しないため、原稿・本コメントには
//! 対応表 ID のみを記す（`settings_billing_overview` と同じ扱い）。
//!
//! # 前半（本 block, #2993）と後半（#2994）の分担
//!
//! 本 block が持たないのは次の 3 領域で、後続 #2994 が同じ class/data-*
//! 命名規約の上へ追加する:
//!
//! - R0255: 行をスイッチで展開して表示する API キー欄
//! - R0251: 連携要望の一覧 + 送信フォーム
//! - R0254: 末尾の空状態枠
//!
//! # 使用部品
//!
//! `badge` / `button` / `separator` / `link` / `image` の 5 部品を合成する
//! （[`BLOCK`] の `parts` に一致させる契約、`blocks_nav.rs`/`blocks_contract.rs`
//! が検証する）。`image` は使用部品一覧に無いロゴ表示のため追加した
//! （`page_heading_avatar::logo_image` と同じ判断）。新しい UI 部品は
//! 追加しない。`switch`/`clipboard`/`input-group`/`input`/`empty-state` は
//! 本 block の使用部品ではなく、#2994 が追加する。
//!
//! # `class` と `data-*` の使い分け
//!
//! [`fandhe_frontend_pre_styled_ui::badge::badge`]・
//! [`fandhe_frontend_pre_styled_ui::button::button`]・
//! [`fandhe_frontend_pre_styled_ui::link::root`]・
//! [`fandhe_frontend_pre_styled_ui::separator::separator`]・
//! [`fandhe_frontend_pre_styled_ui::image::image`] はいずれも `drop_class_attr`
//! で呼び出し側 `class` を除去してから内部 variant クラスと合成するため、
//! これらへの CSS フックは `data-*` 属性で渡す
//! （`data-blocks-settings-integrations-list-*`）。レイアウト用ラッパー
//! （グループ・行・本文・操作領域）は素の `class="blocks-settings-integrations-list-*"`
//! を使う。
//!
//! # 狭幅切替はコンテナクエリで判定する
//!
//! Demo 枠の幅はビューポート幅と一致しないため、`settings_billing_overview`
//! 等と同型に `@container`（コンテナクエリ）で判定する。
//! `.blocks-settings-integrations-list-stack` へ `container-type: inline-size`
//! を宣言し、コンテナ幅が `40rem` 未満のとき行のグリッドを「ロゴ + 本文」を
//! 上段、「操作」を下段（説明の下）へ回す 2 行構成へ切り替える。
//!
//! # `<form>` を使わない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない静的表示のみで、送信処理・送信先は一切持たない。接続/解除
//! ボタンは `button::button` の既定 `type="button"` のまま用いる。
//!
//! # 詳細リンクの遷移先
//!
//! 架空アプリの詳細ページは存在しないため、`href="#"` を使わず実在 URL
//! （本リポジトリ `https://github.com/Fandhe-AI/fandhe-frontend`）へ
//! `LinkProps::external = true` でリンクする（PR #3440
//! `settings_integration_detail` と同じ判断。`external: true` は
//! `rel="noopener noreferrer"` を付与し reverse tabnabbing を防ぐ）。
//! リンク文言は遷移先の実態（本リポジトリの GitHub ページ）と一致させ
//! 「GitHub で見る」とする（近隣 block と同じ表記、PR #3441 Codex/Bugbot
//! 指摘の是正。「詳細を見る」は全アプリ同一の遷移先と矛盾するため使わない）。
//!
//! # ダミー素材について
//!
//! アプリ名（Lattice Notes / Harbor Calendar / Pulse Alerts / Ledger Sync /
//! Beacon Chat / Quarry Storage）・説明文はすべて架空で、実在の企業・製品・
//! 商標・PII は含まない。ロゴは共通ダミー画像
//! [`crate::blocks::dummy_assets::LOGO_SRC`] を使う。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
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

    // 接続/解除ボタンの可視ラベルは全行共通（「接続」/「解除」）のため、
    // 支援技術がどのアプリへの操作か区別できるよう `aria-label` へアプリ名を
    // 埋め込む（PR #3441 Codex 指摘）。
    let action_button = if item.connected {
        button(
            &ButtonProps {
                variant: ButtonVariant::Outline,
                ..ButtonProps::default()
            },
            vec![
                ("data-blocks-settings-integrations-list-action", ""),
                ("aria-label", &format!("{} の連携を解除", item.name)),
            ],
            vec![text("解除")],
        )
    } else {
        button(
            &ButtonProps {
                variant: ButtonVariant::Solid,
                ..ButtonProps::default()
            },
            vec![
                ("data-blocks-settings-integrations-list-action", ""),
                ("aria-label", &format!("{} と連携", item.name)),
            ],
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
                vec![text("GitHub で見る")],
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
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/settings-integrations-list/",
    title: "settings-integrations-list",
    category: BlockCategory::Settings,
    rust_source: "crates/docs-site/src/blocks/application/settings/settings_integrations_list.rs",
    demo_class: "blocks-settings-integrations-list",
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
            label: "Separator",
            path: "/themes/separator/",
        },
        Part {
            label: "Link",
            path: "/themes/link/",
        },
        Part {
            label: "Image",
            path: "/themes/image/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `settings_integrations_list` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節と同型）。
///
/// `group-title`（`h3`）/`list`（`ul`）/`row`（`li`）の 3 セレクタは、素の
/// `.blocks-settings-integrations-list-*` 単一クラス（詳細度 (0,1,0)）のみで
/// 宣言すると、サイト共通 typography（`site_theme.rs` の
/// `.docs-content h3`/`.docs-content ul,ol`/`.docs-content li`、いずれも
/// 詳細度 (0,1,1)）に負けて見出しサイズ・余白・リストの
/// ビュレット/パディング/`margin-block` が意図通りにならない
/// （PR #3441 Bugbot 指摘）。`.blocks-settings-integrations-list-stack`/
/// `-list` を祖先に持つ子孫セレクタへ書き換えてクラス数を 2 に増やし
/// （詳細度 (0,2,0)）、クラス数比較で `.docs-content h3` 等を確実に上回る
/// （型セレクタの有無に依存しないため、`site.css`/本 CSS の読み込み順序に
/// 左右されない）。`row` はさらに `margin-block: 0` を明示し、
/// `.docs-content li` の `margin-block` を打ち消して行間を `border-top`
/// のみに委ねる（image recipe base への `img[data-scope=...]` 上書きと同じ
/// 「サイト共通スタイルは変更せず block 側で限定的に上回る」判断）。
const LAYOUT_CSS: &str = "\
.blocks-settings-integrations-list-stack {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n  container-type: inline-size;\n  container-name: blocks-settings-integrations-list;\n}\n\
.blocks-settings-integrations-list-stack .blocks-settings-integrations-list-group-title {\n  font-size: var(--fandhe-font-font-size-sm);\n  font-weight: var(--fandhe-font-font-weight-semibold);\n  color: var(--fandhe-color-fg-muted);\n  margin: 0 0 var(--fandhe-space-2);\n}\n\
.blocks-settings-integrations-list-stack .blocks-settings-integrations-list-list {\n  list-style: none;\n  margin: 0;\n  padding: 0;\n  border: 1px solid var(--fandhe-color-border);\n  border-radius: var(--fandhe-radius-lg);\n  background: var(--fandhe-color-bg);\n}\n\
.blocks-settings-integrations-list-list .blocks-settings-integrations-list-row {\n  display: grid;\n  grid-template-columns: auto minmax(0, 1fr) auto;\n  grid-template-areas: \"logo body actions\";\n  align-items: center;\n  gap: var(--fandhe-space-4);\n  padding: var(--fandhe-space-4);\n  margin-block: 0;\n}\n\
.blocks-settings-integrations-list-row + .blocks-settings-integrations-list-row {\n  border-top: 1px solid var(--fandhe-color-border);\n}\n\
[data-blocks-settings-integrations-list-logo] {\n  grid-area: logo;\n}\n\
img[data-scope=\"image\"][data-blocks-settings-integrations-list-logo] {\n  width: 2.5rem;\n  height: 2.5rem;\n}\n\
.blocks-settings-integrations-list-body {\n  grid-area: body;\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1);\n}\n\
.blocks-settings-integrations-list-name {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n  font-weight: var(--fandhe-font-font-weight-medium);\n}\n\
.blocks-settings-integrations-list-description {\n  color: var(--fandhe-color-fg-muted);\n  font-size: var(--fandhe-font-font-size-sm);\n}\n\
.blocks-settings-integrations-list-actions {\n  grid-area: actions;\n}\n\
@container blocks-settings-integrations-list (max-width: 40rem) {\n  \
.blocks-settings-integrations-list-list .blocks-settings-integrations-list-row {\n    grid-template-columns: auto minmax(0, 1fr);\n    grid-template-areas: \"logo body\" \"logo actions\";\n  }\n  \
.blocks-settings-integrations-list-actions {\n    justify-self: start;\n  }\n\
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
            "data-scope=\"separator\"",
            "data-scope=\"image\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
    }

    #[test]
    fn demo_has_two_groups_and_six_rows() {
        let html = demo_html();
        assert_eq!(
            html.matches("blocks-settings-integrations-list-group-title")
                .count(),
            2
        );
        assert_eq!(
            html.matches("blocks-settings-integrations-list-row")
                .count(),
            // 各行の class 出現は 1 回だが、隣接セレクタ用の `+` 記述は
            // LAYOUT_CSS 側にのみ存在するため、demo 側は行要素数と一致する。
            6
        );
        assert!(html.contains("接続済み"));
        assert!(html.contains("未接続"));
        assert!(html.contains("接続"));
        assert!(html.contains("解除"));
    }

    #[test]
    fn no_form_submit_or_dead_links() {
        let html = demo_html();
        assert!(!html.contains("<form"));
        assert!(!html.contains("type=\"submit\""));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("src=\"data:"));
        assert!(!html.contains("<script"));
    }

    #[test]
    fn buttons_use_type_button_only() {
        let html = demo_html();
        assert_eq!(html.matches("<button").count(), 6);
        assert_eq!(html.matches("type=\"button\"").count(), 6);
    }

    #[test]
    fn detail_links_are_external_with_noopener() {
        let html = demo_html();
        assert_eq!(
            html.matches("rel=\"noopener noreferrer\"").count(),
            6,
            "each of the 6 rows should have one external detail link"
        );
    }

    #[test]
    fn detail_link_text_matches_destination() {
        // 遷移先が全アプリ共通で本リポジトリの GitHub ページである実態に
        // 合わせ、「詳細を見る」（詳細ページが実在するかのような誤解を招く
        // 文言）ではなく「GitHub で見る」を使う（PR #3441 Codex/Bugbot 指摘）。
        let html = demo_html();
        assert!(!html.contains("詳細を見る"));
        assert_eq!(html.matches("GitHub で見る").count(), 6);
    }

    #[test]
    fn action_buttons_have_app_specific_accessible_name() {
        // 「接続」「解除」だけでは支援技術上どのアプリへの操作か区別できない
        // ため、`aria-label` にアプリ名を含める（PR #3441 Codex 指摘）。
        let html = demo_html();
        for name in [
            "Lattice Notes",
            "Harbor Calendar",
            "Pulse Alerts",
            "Ledger Sync",
            "Beacon Chat",
            "Quarry Storage",
        ] {
            assert!(
                html.contains(&format!("aria-label=\"{name} ")),
                "expected an accessible action-button label mentioning {name}"
            );
        }
        assert_eq!(html.matches("aria-label=\"").count(), 6);
    }

    #[test]
    fn layout_css_is_safe_and_stacks_on_narrow_container() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("container-type: inline-size;"));
        assert!(
            LAYOUT_CSS.contains("@container blocks-settings-integrations-list (max-width: 40rem)")
        );
        assert!(LAYOUT_CSS.contains("grid-template-areas: \"logo body\" \"logo actions\""));
    }

    #[test]
    fn narrow_row_selector_specificity_matches_base_rule() {
        // `@container` 内の狭幅用行セレクタが通常ルール
        // `.blocks-settings-integrations-list-list .blocks-settings-integrations-list-row`
        // （詳細度 (0,2,0)）と同じ 2 クラスの子孫セレクタであることの回帰
        // ガード（PR #3441 Codex/Bugbot 指摘）。単一クラス（詳細度 (0,1,0)）
        // では通常ルールに負け、40rem 未満でもレイアウトが切り替わらない。
        assert!(LAYOUT_CSS.contains(
            "@container blocks-settings-integrations-list (max-width: 40rem) {\n  .blocks-settings-integrations-list-list .blocks-settings-integrations-list-row {"
        ));
    }

    #[test]
    fn logo_selector_specificity_beats_image_recipe_base() {
        // `[data-scope="image"][data-part="root"]`（詳細度 (0,2,0)）の
        // `height: auto`/`max-width: 100%` に負けないことの回帰ガード
        // （PR #3441 Bugbot 指摘、イシュー #2931 と同型）。
        assert!(LAYOUT_CSS
            .contains("img[data-scope=\"image\"][data-blocks-settings-integrations-list-logo]"));
    }

    #[test]
    fn typography_selectors_beat_docs_content_specificity() {
        // `.docs-content h3`/`.docs-content ul,ol`/`.docs-content li`
        // （いずれも詳細度 (0,1,1)）に負けないよう、group-title/list/row は
        // 2 クラスの子孫セレクタ（詳細度 (0,2,0)）で宣言する回帰ガード
        // （PR #3441 Bugbot 指摘）。単一クラスの旧セレクタが復活していない
        // ことも合わせて固定する。
        assert!(LAYOUT_CSS.contains(
            ".blocks-settings-integrations-list-stack .blocks-settings-integrations-list-group-title {"
        ));
        assert!(LAYOUT_CSS.contains(
            ".blocks-settings-integrations-list-stack .blocks-settings-integrations-list-list {"
        ));
        assert!(LAYOUT_CSS.contains(
            ".blocks-settings-integrations-list-list .blocks-settings-integrations-list-row {"
        ));
        assert!(!LAYOUT_CSS.contains("\n.blocks-settings-integrations-list-group-title {"));
        assert!(!LAYOUT_CSS.contains("\n.blocks-settings-integrations-list-list {"));
        assert!(LAYOUT_CSS.contains("margin-block: 0;"));
    }
}
