//! `grid-list-logo-cards` block（イシュー #2921。親トラッキング #2892
//! 「Blocks アプリケーション A」配下、Application/Grid List カテゴリ
//! 最初の block）。取引先カードを並べるグリッドの合成例。主参照は対応表
//! ID R0979 のみで、集約元との差分は下記「原案差分メモ」相当の各節に
//! 記す（`site/blocks/grid-list-logo-cards.md` の「原案差分メモ」節と
//! 対応する）。
//!
//! # 使用部品
//!
//! `card` / `avatar` / `image` / `data-list` / `badge` / `menu` の 6 部品
//! のみを合成する（[`BLOCK`] の `parts` に一致させる契約）。新しい UI
//! 部品は追加しない。
//!
//! # ロゴの有無による出し分け
//!
//! [`Client::has_logo`] が `true` の取引先は [`image::image`]（装飾画像の
//! ため `alt` は空文字。隣接する社名テキストが実質的なラベルを担う）で
//! ロゴを表示し、`false` の取引先は [`avatar::root`] + イニシャル
//! [`avatar::fallback`]（`ImageStatus::Error`）で代用する
//! （`blog_grid_text::author` と同じ「画像アセットへ依存しないアバター」
//! の判断）。
//!
//! # 1〜3 列のグリッド
//!
//! [`LAYOUT_CSS`] は `grid-template-columns: repeat(auto-fill,
//! minmax(min(100%, 13rem), 1fr))` のみでレスポンシブな列数（狭い幅では
//! 1 列・広い幅では 2〜3 列）を実現する。カード最小幅はイシュー #2921 の
//! レビュー指摘を受け、Demo 枠の実効幅（`.docs-content` の
//! `max-width: 46rem` から `.blocks-demo` の左右 padding 3rem を引いた
//! 43rem 程度）で 3 列（3 × 13rem + 2 × `--fandhe-space-6`〔1.5rem〕=
//! 42rem）が収まるよう 16rem から調整した（`@container` 等による列数上限の
//! 明示指定は行わない）。
//!
//! # 三点メニューは無 JS のため閉じた状態で固定する
//!
//! 開閉状態機械・実際のポップアップ表示はクライアント配線層（wasm-full）の
//! 責務であり、無 JS の docs サイトでは `OpenState::Closed` の静的表示のみを
//! 描画する（`card_heading_toolbar::overflow_menu` と同型の構成）。
//!
//! # `<form>` を持たない・データ取得/送信を行わない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である。三点メニューの項目はナビゲーション用途を模すのみで実際の送信先
//! を持たない（`docs/policy/intentional-non-adoption.md` §3.25）。
//!
//! # デモデータは架空
//!
//! 社名は [`crate::blocks::dummy_assets::COMPANY_NAMES`] から取り、金額・
//! 請求日・支払状態はすべて独自に書いた架空の値である（実企業名・実
//! クレデンシャル・PII を含まない）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, li, span, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, AvatarShape, ImageStatus};
use fandhe_frontend_pre_styled_ui::badge::{badge, BadgeProps, BadgeVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps, CardVariant};
use fandhe_frontend_pre_styled_ui::data_list::{self, DataListOrientation, DataListProps};
use fandhe_frontend_pre_styled_ui::image::{image, AspectRatio, ImageFit, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::menu::{self, OpenState};
use fandhe_frontend_pre_styled_ui::recipe::ColorPalette;
use fandhe_frontend_pre_styled_ui::Size;

/// 支払状態（[`ColorPalette`] へ写像するための固有 enum）。
#[derive(Clone, Copy)]
enum PaymentStatus {
    Paid,
    Unpaid,
    Overdue,
}

impl PaymentStatus {
    fn label(self) -> &'static str {
        match self {
            Self::Paid => "支払済み",
            Self::Unpaid => "未払い",
            Self::Overdue => "期限超過",
        }
    }

    fn palette(self) -> ColorPalette {
        match self {
            Self::Paid => ColorPalette::Success,
            Self::Unpaid => ColorPalette::Warning,
            Self::Overdue => ColorPalette::Danger,
        }
    }
}

/// 取引先 1 件分のデモデータ。
struct Client {
    name: &'static str,
    initials: &'static str,
    has_logo: bool,
    invoice_date: &'static str,
    invoice_date_label: &'static str,
    amount: &'static str,
    status: PaymentStatus,
    /// menu の `content`/`trigger` を結ぶ id（カード内で一意、
    /// `format!` を使わず定数化する。モジュール doc「三点メニュー」節参照）。
    menu_id: &'static str,
}

/// デモ取引先 4 件（ロゴあり 2 件・ロゴなし 2 件、状態 3 種を混在させる）。
const CLIENTS: &[Client] = &[
    Client {
        name: dummy_assets::COMPANY_NAMES[0],
        initials: "LS",
        has_logo: true,
        invoice_date: "2026-09-01",
        invoice_date_label: "2026年9月1日",
        amount: "¥340,000",
        status: PaymentStatus::Paid,
        menu_id: "grid-list-logo-cards-menu-1",
    },
    Client {
        name: dummy_assets::COMPANY_NAMES[1],
        initials: "VF",
        has_logo: false,
        invoice_date: "2026-09-10",
        invoice_date_label: "2026年9月10日",
        amount: "¥128,500",
        status: PaymentStatus::Unpaid,
        menu_id: "grid-list-logo-cards-menu-2",
    },
    Client {
        name: dummy_assets::COMPANY_NAMES[2],
        initials: "QM",
        has_logo: true,
        invoice_date: "2026-08-15",
        invoice_date_label: "2026年8月15日",
        amount: "¥76,200",
        status: PaymentStatus::Overdue,
        menu_id: "grid-list-logo-cards-menu-3",
    },
    Client {
        name: dummy_assets::COMPANY_NAMES[3],
        initials: "TC",
        has_logo: false,
        invoice_date: "2026-09-20",
        invoice_date_label: "2026年9月20日",
        amount: "¥512,000",
        status: PaymentStatus::Paid,
        menu_id: "grid-list-logo-cards-menu-4",
    },
];

/// カードヘッダー左側（ロゴ or イニシャルアバター + 社名）。
fn logo_or_avatar(client: &Client) -> Node {
    if client.has_logo {
        image(
            &ImageProps {
                fit: ImageFit::Contain,
                aspect_ratio: AspectRatio::Square,
                shape: ImageShape::Rounded,
                ..ImageProps::new(dummy_assets::LOGO_SRC, "")
            },
            vec![("data-blocks-grid-list-logo-cards-logo", "")],
        )
    } else {
        avatar::root(
            &AvatarProps {
                size: Size::Md,
                shape: AvatarShape::Rounded,
                ..AvatarProps::default()
            },
            vec![("data-blocks-grid-list-logo-cards-logo", "")],
            vec![avatar::fallback(
                ImageStatus::Error,
                vec![],
                vec![text(client.initials)],
            )],
        )
    }
}

/// 三点メニュー（無 JS のため閉じた状態で固定する。モジュール doc
/// 「三点メニューは無 JS のため閉じた状態で固定する」節参照）。
fn overflow_menu(client: &Client) -> Node {
    let label = format!("{} の操作", client.name);
    let trigger = menu::trigger(
        OpenState::Closed,
        false,
        Some(client.menu_id),
        vec![("aria-label", &label)],
        vec![text("\u{2026}")],
    );
    let content = menu::content(
        OpenState::Closed,
        Some(client.menu_id),
        None,
        vec![],
        vec![
            menu::item("view", false, false, vec![], vec![text("詳細を見る")]),
            menu::item(
                "edit-invoice",
                false,
                false,
                vec![],
                vec![text("請求書を編集")],
            ),
            menu::separator(vec![], vec![]),
            menu::item("archive", false, false, vec![], vec![text("アーカイブ")]),
        ],
    );
    let positioner = menu::positioner(OpenState::Closed, vec![], vec![content]);
    menu::root(
        Size::Sm,
        OpenState::Closed,
        vec![],
        vec![trigger, positioner],
    )
}

/// カード下段（`data-list` による最新請求日・金額 + 支払状態バッジ）。
fn client_data_list(client: &Client) -> Node {
    let invoice_date_item = data_list::item(
        vec![],
        vec![
            data_list::item_label(vec![], vec![text("最新請求日")]),
            data_list::item_value(
                vec![],
                vec![el(
                    "time",
                    vec![("datetime", client.invoice_date)],
                    vec![text(client.invoice_date_label)],
                )],
            ),
        ],
    );
    let amount_item = data_list::item(
        vec![],
        vec![
            data_list::item_label(vec![], vec![text("金額")]),
            data_list::item_value(
                vec![("data-blocks-grid-list-logo-cards-amount-row", "")],
                vec![
                    span(
                        vec![("data-blocks-grid-list-logo-cards-amount", "")],
                        vec![text(client.amount)],
                    ),
                    badge(
                        &BadgeProps {
                            variant: BadgeVariant::Subtle,
                            size: Size::Sm,
                            palette: client.status.palette(),
                        },
                        vec![],
                        vec![text(client.status.label())],
                    ),
                ],
            ),
        ],
    );
    data_list::root(
        DataListProps {
            orientation: DataListOrientation::Vertical,
            ..DataListProps::default()
        },
        vec![],
        vec![invoice_date_item, amount_item],
    )
}

/// 取引先カード 1 枚。
fn client_card(client: &Client) -> Node {
    let card = card::root(
        CardProps::from(CardVariant::Outline),
        vec![],
        vec![
            card::header(
                vec![("data-has-action", "")],
                vec![
                    div(
                        vec![("data-blocks-grid-list-logo-cards-heading", "")],
                        vec![
                            logo_or_avatar(client),
                            card::title(vec![], vec![text(client.name)]),
                        ],
                    ),
                    card::action(vec![], vec![overflow_menu(client)]),
                ],
            ),
            card::body(vec![], vec![client_data_list(client)]),
        ],
    );
    li(vec![], vec![card])
}

/// `grid-list-logo-cards` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。
pub fn demo() -> Node {
    let cards: Vec<Node> = CLIENTS.iter().map(client_card).collect();
    div(
        vec![("class", "blocks-grid-list-logo-cards-layout")],
        vec![el(
            "ul",
            vec![
                ("class", "blocks-grid-list-logo-cards-grid"),
                ("role", "list"),
            ],
            cards,
        )],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/grid-list-logo-cards/",
    title: "grid-list-logo-cards",
    category: BlockCategory::GridList,
    rust_source: "crates/docs-site/src/blocks/application/grid_list/grid_list_logo_cards.rs",
    demo_class: "blocks-grid-list-logo-cards",
    parts: &[
        Part {
            label: "Card",
            path: "/themes/card/",
        },
        Part {
            label: "Avatar",
            path: "/themes/avatar/",
        },
        Part {
            label: "Image",
            path: "/themes/image/",
        },
        Part {
            label: "Data List",
            path: "/themes/data-list/",
        },
        Part {
            label: "Badge",
            path: "/themes/badge/",
        },
        Part {
            label: "Menu",
            path: "/themes/menu/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `grid_list_logo_cards` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節と同型）。セレクタはすべて
/// `.blocks-grid-list-logo-cards-*` か `[data-blocks-grid-list-logo-cards-*]`
/// の名前空間に収める。
const LAYOUT_CSS: &str = "\
.blocks-grid-list-logo-cards-grid {\n  list-style: none;\n  margin: 0;\n  padding: 0;\n  display: grid;\n  gap: var(--fandhe-space-6);\n  grid-template-columns: repeat(auto-fill, minmax(min(100%, 13rem), 1fr));\n}\n\
[data-blocks-grid-list-logo-cards-heading] {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-3);\n}\n\
[data-blocks-grid-list-logo-cards-logo] {\n  width: 2.5rem;\n  height: 2.5rem;\n  flex-shrink: 0;\n}\n\
[data-scope=\"data-list\"][data-part=\"item-value\"][data-blocks-grid-list-logo-cards-amount-row] {\n  justify-content: space-between;\n}\n\
[data-blocks-grid-list-logo-cards-amount] {\n  font-weight: var(--fandhe-font-font-weight-bold);\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, CLIENTS, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    fn demo_html() -> String {
        render(&demo())
    }

    #[test]
    fn demo_composes_expected_parts() {
        let html = demo_html();
        for scope in [
            "data-scope=\"card\"",
            "data-scope=\"avatar\"",
            "data-scope=\"image\"",
            "data-scope=\"data-list\"",
            "data-scope=\"badge\"",
            "data-scope=\"menu\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
    }

    #[test]
    fn dl_count_matches_client_count() {
        let html = demo_html();
        assert_eq!(html.matches("<dl").count(), CLIENTS.len());
    }

    #[test]
    fn logo_image_count_matches_has_logo_clients() {
        let html = demo_html();
        let logo_count = CLIENTS.iter().filter(|c| c.has_logo).count();
        assert_eq!(html.matches("<img").count(), logo_count);
    }

    #[test]
    fn avatar_fallback_shows_initials_for_no_logo_clients() {
        let html = demo_html();
        for client in CLIENTS.iter().filter(|c| !c.has_logo) {
            assert!(
                html.contains(client.initials),
                "avatar fallback should show initials for {}",
                client.name
            );
        }
    }

    #[test]
    fn all_three_status_badges_present() {
        let html = demo_html();
        for label in ["支払済み", "未払い", "期限超過"] {
            assert!(html.contains(label), "demo should contain badge {label}");
        }
    }

    #[test]
    fn no_form_or_unsafe_markup() {
        let html = demo_html();
        for absent in ["<form", "<script", "src=\"data:", "href=\"#\""] {
            assert!(!html.contains(absent), "demo should never contain {absent}");
        }
    }

    #[test]
    fn menu_content_ids_are_unique() {
        let mut ids: Vec<&str> = CLIENTS.iter().map(|c| c.menu_id).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), CLIENTS.len());
    }

    #[test]
    fn layout_css_is_safe_and_defines_grid() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("display: grid"));
        assert!(LAYOUT_CSS.contains("grid-template-columns"));
    }
}
