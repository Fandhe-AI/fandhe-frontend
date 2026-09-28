# grid-list-logo-cards

`fandhe-frontend-pre-styled-ui` の `card` / `avatar` / `image` / `data-list`
/ `badge` / `menu` を合成した、取引先カードのグリッドです。Blocks
セクションは新規部品を追加するものではなく、既存の Themes/Primitives
部品を組み合わせた実例集であることに注意してください（主参照は対応表
ID R0979。出典の固有名・ファイル名は記載しません）。

各カードはヘッダー（ロゴまたはイニシャルアバター + 社名 + 三点メニュー）
と、下段の定義リスト（最新請求日・金額 + 支払状態バッジ）の 2 段で
構成します。ロゴを持たない取引先はイニシャル入りのアバターで代わりに
表示します。グリッドは幅に応じて 1〜3 列に自動で折り返します。

本 Demo は静的な表示例であり、`<form>` 要素は一切持たず、データの取得・
送信・状態管理を行いません。三点メニューは無 JS の docs サイトのため
閉じた状態で固定表示しています。文言・数値はすべて独自に書いた架空の
ものであり、実企業名・実クレデンシャル・PII を含みません。

## Rust コード

```rust
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
```

## 原案差分メモ

- 集約元は主参照（対応表 ID R0979）の 1 件のみです。参照元の文言・配色・
  アイコン意匠は持ち込まず、社名・金額・請求日・支払状態はすべて独自の
  架空データです（社名のみ `dummy_assets::COMPANY_NAMES` を再利用）。
- ロゴなしカードは `image` の代わりにイニシャル入りの `avatar::fallback`
  （`ImageStatus::Error`）で代用しています。実在ロゴ画像アセットへは
  依存しません。
- 三点メニューは無 JS の docs サイトのため `OpenState::Closed` の静的表示
  で固定しています。開閉状態機械・実際のポップアップ表示はクライアント
  配線層（wasm-full）の責務です。

関連情報: [Card](../themes/card.md) / [Avatar](../themes/avatar.md) /
[Image](../themes/image.md) / [Data List](../themes/data-list.md) /
[Badge](../themes/badge.md) / [Menu](../themes/menu.md)
