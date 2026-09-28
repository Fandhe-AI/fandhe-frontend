//! `grid-list-contact-cards` block（イシュー #2919。Application/Grid List
//! カテゴリの最初の block、親トラッキング #2892「Blocks 目的別パーツ拡充」
//! 配下）。連絡先カードを 1〜4 列のグリッドに並べる合成例。対応表 ID
//! R0973（主参照、横型カード）と R0974（集約元、縦型カード）を集約する。
//! `_/blocks-intake/` の対応ファイルは本イシュー着手時点で本 worktree に
//! 存在しないため、原稿・本コメントには対応表 ID のみを記す
//! （`form_layout_two_column`〔イシュー #2916〕と同じ扱い）。
//!
//! # 使用部品
//!
//! `card` / `avatar` / `badge` / `button` / `icon` / `list` の 6 部品を
//! 合成する（[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//! 新しい UI 部品は追加しない。
//!
//! # horizontal / vertical の 2 インスタンスを並べる理由
//!
//! 主参照 R0973（[`horizontal_card`]、左に名前・役職・権限バッジ、右に
//! 小さいアバター）に対し、集約元 R0974（[`vertical_card`]、中央に大きい
//! アバター、その下に名前などを縦に積む）を並記する。`testimonial_card_grid`
//! と同型の判断で、1 つの Demo へ詰め込むより 2 例を並べる方が違いを
//! 一目で読み取れる。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `card::root`/`avatar::root`/`badge::badge`/`button::button`/
//! `icon::icon`/`list::root` はいずれも `drop_class_attr` により呼び出し側
//! `attrs` の `class` を黙って除去する契約を持つため、Demo 固有の
//! スタイルフックは `data-blocks-grid-list-contact-cards-*` 属性で渡す。
//! `card::body`/`card::footer`/`list::item`/素の `div` には `class` が
//! そのまま効くため、レイアウトは `.blocks-grid-list-contact-cards-*`
//! クラスセレクタを使う。ただし `card::body`/`card::footer`/`button::button`
//! が持つ recipe セレクタ（`[data-scope="card"][data-part="body"]`/
//! `[data-scope="card"][data-part="footer"]`/
//! `[data-scope="button"][data-part="root"]`、いずれも属性 2 個で
//! 詳細度 (0,2,0,0)）が `flex-direction`/padding/border-radius を宣言している
//! ため、単一クラス・単一属性セレクタ（詳細度 (0,1,0,0)）の上書きは詳細度
//! 負けで効かない（`.blocks-grid-list-contact-cards-main` が
//! `card::body` の `flex-direction: column` に負けて横型カードが縦積みに
//! なる不具合として実際に発生、イシュー #2919 レビュー指摘）。
//! `.blocks-grid-list-contact-cards-main`/`actions_footer`/`actions_footer`
//! 内ボタンの CSS フックは recipe セレクタと同じ要素へ複合セレクタ
//! （`[data-scope="card"][data-part="body"].blocks-grid-list-contact-cards-main`/
//! `[data-scope=...][data-part=...] .blocks-grid-list-contact-cards-actions`
//! 等、詳細度 (0,3,0,0)）で書き、カスケード順ではなく詳細度で確実に
//! 上書きする（`layout_css_contract` テスト参照）。
//!
//! # `@container` で列数を切り替える理由
//!
//! Demo 枠の幅はビューポート幅と一致しないため、`@media` ではなく
//! `form_layout_two_column`/`testimonial_card_grid` と同型の `@container`
//! を使う。各インスタンスのラッパーへ `container-type: inline-size;
//! container-name: blocks-grid-list-contact-cards;` を宣言し、`36rem`/
//! `52rem` で列数を 1 → 2 → 3 へ増やす。`vertical` インスタンスのみ
//! `68rem` でさらに 4 列へ増やす（横型カードは情報量が多く 4 列だと
//! 窮屈になるため据え置く）。`68rem` の縦型限定判定は
//! `[data-blocks-grid-list-contact-cards-variant="vertical"]
//! [data-blocks-grid-list-contact-cards-grid]`（コンテナー自身への祖先
//! 参照を含む子孫セレクタ）では一致しない。`@container` 内のスタイル規則は
//! クエリ対象のコンテナー自身ではなく子孫にのみ適用され、
//! `data-blocks-grid-list-contact-cards-variant` はコンテナーそのもの
//! （`container-type` を宣言する外側 `div`）に付与されているため
//! （イシュー #2919 レビュー指摘）。このため縦型判定はコンテナーの子孫で
//! ある grid 要素自身の属性
//! （`[data-blocks-grid-list-contact-cards-grid][data-blocks-grid-list-contact-cards-grid-variant="vertical"]`）
//! へ付け替えている。
//!
//! # 見出しを使わない理由
//!
//! ページ側が `## Demo` として `h2` を出す前提の上に、本 block はさらに
//! 見出しを重ねる構成にしない（`card-media-footer` 等、見出しを持たない
//! 既存 block と同じ判断）。状態ラベルは `text`（Muted）ではなく素の `div`
//! に留める最小構成とし、新規部品を増やさない。
//!
//! # アバター・アイコンの a11y
//!
//! avatar の画像は `alt=""`（隣接する氏名テキストと同じ情報を伝えるため
//! 装飾扱い）にし、無 JS のため `ImageStatus::Loaded` を明示する
//! （`testimonial_card_grid` と同じ判断）。メール・電話アイコンは自作の
//! 幾何図形（封筒・受話器を単純な線で描く）で `IconProps { label: None,
//! .. }`（`aria-hidden="true"`）にする。ボタン名（可視テキスト）で操作の
//! 意味は伝わるため、アイコン単独では情報を持たせない。
//!
//! # `mailto:`/`tel:` リンクを使わない理由
//!
//! Issue 本文がボタンでの表現を指定しているため、`href` を持つ `<a>` では
//! なく `button::button`（既定 `type="button"`）を使う。送信先を持たない
//! 静的な表示であり、暗黙の送信・遷移は起こらない。
//!
//! # 参照について
//!
//! 主参照は対応表 ID R0973、集約元は R0974。文言・配色・アイコンは独自に
//! 書く（他 block と同じライセンス上の転記制限）。実在の人物・企業名・PII
//! は使わない。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, AvatarShape, ImageStatus};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps, BadgeVariant};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::list::{self, ListType, ListVariant};
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// 連絡先 1 件分の静的データ（架空、実企業名・PII を含まない）。
struct Contact {
    name: &'static str,
    role: &'static str,
    permission: &'static str,
    palette: ColorPalette,
}

/// `horizontal`（R0973）用の 6 件。
const CONTACTS_HORIZONTAL: [Contact; 6] = [
    Contact {
        name: "Haruto Fujimaki",
        role: "Product Designer",
        permission: "管理者",
        palette: ColorPalette::Success,
    },
    Contact {
        name: "Elena Vasquez",
        role: "Engineering Lead",
        permission: "編集者",
        palette: ColorPalette::Info,
    },
    Contact {
        name: "Kwame Boateng",
        role: "Customer Success Manager",
        permission: "閲覧者",
        palette: ColorPalette::Neutral,
    },
    Contact {
        name: "Mei Lindqvist",
        role: "Data Analyst",
        permission: "編集者",
        palette: ColorPalette::Info,
    },
    Contact {
        name: "Noor Al-Sayed",
        role: "Marketing Strategist",
        permission: "閲覧者",
        palette: ColorPalette::Neutral,
    },
    Contact {
        name: "Ola Bergström",
        role: "Operations Coordinator",
        permission: "管理者",
        palette: ColorPalette::Success,
    },
];

/// `vertical`（R0974）用の 8 件。
const CONTACTS_VERTICAL: [Contact; 8] = [
    Contact {
        name: "Priya Chandran",
        role: "Customer Success Manager",
        permission: "編集者",
        palette: ColorPalette::Info,
    },
    Contact {
        name: "Théo Marchetti",
        role: "Data Analyst",
        permission: "閲覧者",
        palette: ColorPalette::Neutral,
    },
    Contact {
        name: "Ingrid Solheim",
        role: "Product Manager",
        permission: "管理者",
        palette: ColorPalette::Success,
    },
    Contact {
        name: "Diego Alcantara",
        role: "Support Engineer",
        permission: "閲覧者",
        palette: ColorPalette::Neutral,
    },
    Contact {
        name: "Amara Okafor",
        role: "Sales Manager",
        permission: "編集者",
        palette: ColorPalette::Info,
    },
    Contact {
        name: "Lukas Hoffmann",
        role: "QA Engineer",
        permission: "閲覧者",
        palette: ColorPalette::Neutral,
    },
    Contact {
        name: "Saanvi Rao",
        role: "Content Strategist",
        permission: "編集者",
        palette: ColorPalette::Info,
    },
    Contact {
        name: "Marcus Lindberg",
        role: "Operations Analyst",
        permission: "管理者",
        palette: ColorPalette::Success,
    },
];

/// メール封筒アイコン（装飾、モジュール doc「アバター・アイコンの a11y」節）。
fn mail_icon() -> Node {
    icon(
        &IconProps {
            label: None,
            ..IconProps::default()
        },
        vec![],
        vec![el(
            "path",
            vec![
                ("d", "M4 6h16v12H4z M4 6l8 7 8-7"),
                ("fill", "none"),
                ("stroke", "currentColor"),
                ("stroke-width", "1.5"),
                ("stroke-linejoin", "round"),
                ("stroke-linecap", "round"),
            ],
            vec![],
        )],
    )
}

/// 電話受話器アイコン（装飾）。
fn phone_icon() -> Node {
    icon(
        &IconProps {
            label: None,
            ..IconProps::default()
        },
        vec![],
        vec![el(
            "path",
            vec![
                (
                    "d",
                    "M5 4h3l2 5-2.5 1.5a11 11 0 0 0 5 5L14 13l5 2v3a2 2 0 0 1-2 2A15 15 0 0 1 3 6a2 2 0 0 1 2-2z",
                ),
                ("fill", "none"),
                ("stroke", "currentColor"),
                ("stroke-width", "1.5"),
                ("stroke-linejoin", "round"),
                ("stroke-linecap", "round"),
            ],
            vec![],
        )],
    )
}

/// 権限バッジ 1 件（`Size::Sm`・`BadgeVariant::Subtle`）。
fn permission_badge(item: &Contact) -> Node {
    badge::badge(
        &BadgeProps {
            variant: BadgeVariant::Subtle,
            size: Size::Sm,
            palette: item.palette,
        },
        vec![],
        vec![text(item.permission)],
    )
}

/// メール・電話の 2 分割アクション行（横型・縦型共通、モジュール doc
/// 「`mailto:`/`tel:` リンクを使わない理由」節）。
fn actions_footer() -> Node {
    card::footer(
        vec![("class", "blocks-grid-list-contact-cards-actions")],
        vec![
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Ghost,
                    ..ButtonProps::default()
                },
                vec![("data-blocks-grid-list-contact-cards-action", "")],
                vec![mail_icon(), text("メール")],
            ),
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Ghost,
                    ..ButtonProps::default()
                },
                vec![("data-blocks-grid-list-contact-cards-action", "")],
                vec![phone_icon(), text("電話")],
            ),
        ],
    )
}

/// 横型カード 1 枚（R0973。左に名前・役職・権限バッジ、右に小さい
/// アバター）。
fn horizontal_card(item: &Contact) -> Node {
    card::root(
        CardProps::default(),
        vec![],
        vec![
            card::body(
                vec![("class", "blocks-grid-list-contact-cards-main")],
                vec![
                    div(
                        vec![("class", "blocks-grid-list-contact-cards-info")],
                        vec![
                            div(vec![], vec![text(item.name)]),
                            permission_badge(item),
                            div(
                                vec![("class", "blocks-grid-list-contact-cards-role")],
                                vec![text(item.role)],
                            ),
                        ],
                    ),
                    avatar::root(
                        &AvatarProps {
                            size: Size::Md,
                            shape: AvatarShape::Circle,
                            ..AvatarProps::default()
                        },
                        vec![("data-blocks-grid-list-contact-cards-avatar", "")],
                        vec![avatar::image(
                            ImageStatus::Loaded,
                            dummy_assets::AVATAR_SRC,
                            "",
                            vec![],
                        )],
                    ),
                ],
            ),
            actions_footer(),
        ],
    )
}

/// 縦型カード 1 枚（R0974。中央に大きいアバター、その下に名前などを縦に
/// 積む）。
fn vertical_card(item: &Contact) -> Node {
    card::root(
        CardProps::default(),
        vec![],
        vec![
            card::body(
                vec![("class", "blocks-grid-list-contact-cards-stack")],
                vec![
                    avatar::root(
                        &AvatarProps {
                            size: Size::Xl,
                            shape: AvatarShape::Circle,
                            ..AvatarProps::default()
                        },
                        vec![("data-blocks-grid-list-contact-cards-avatar", "")],
                        vec![avatar::image(
                            ImageStatus::Loaded,
                            dummy_assets::AVATAR_SRC,
                            "",
                            vec![],
                        )],
                    ),
                    div(vec![], vec![text(item.name)]),
                    div(
                        vec![("class", "blocks-grid-list-contact-cards-role")],
                        vec![text(item.role)],
                    ),
                    permission_badge(item),
                ],
            ),
            actions_footer(),
        ],
    )
}

/// 1 枚のカードを `list::item` へ包む（モジュール doc「CSS フックの選び方」
/// 節。`ListVariant::Plain` の item は `display: flex; align-items:
/// flex-start` になり item 自身は親 grid セルいっぱいに伸びるが、その
/// 唯一の子である card は主軸方向（既定 row）にサイズが content 依存の
/// まま先頭寄せに残る。`blocks-grid-list-contact-cards-item` 側は
/// `min-width: 0`（オーバーフロー対策）のみを持ち、card 側のセレクタへ
/// `flex: 1; width: 100%` を持たせてグリッドセルいっぱいへ伸長させる）。
fn card_item(card: Node) -> Node {
    list::item(
        vec![("class", "blocks-grid-list-contact-cards-item")],
        vec![card],
    )
}

/// インスタンス 1 件分（状態ラベル + グリッド）を組み立てる。
fn instance(variant: &'static str, label: &'static str, cards: Vec<Node>) -> Node {
    div(
        vec![("data-blocks-grid-list-contact-cards-variant", variant)],
        vec![
            div(
                vec![("class", "blocks-grid-list-contact-cards-label")],
                vec![text(label)],
            ),
            list::root(
                ListType::Unordered,
                ListVariant::Plain,
                vec![
                    ("data-blocks-grid-list-contact-cards-grid", ""),
                    ("data-blocks-grid-list-contact-cards-grid-variant", variant),
                ],
                cards,
            ),
        ],
    )
}

/// `grid-list-contact-cards` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。horizontal/vertical の 2 インスタンスを縦に並べる
/// （モジュール doc「horizontal / vertical の 2 インスタンスを並べる理由」
/// 節）。
pub fn demo() -> Node {
    let horizontal_cards: Vec<Node> = CONTACTS_HORIZONTAL
        .iter()
        .map(|item| card_item(horizontal_card(item)))
        .collect();
    let vertical_cards: Vec<Node> = CONTACTS_VERTICAL
        .iter()
        .map(|item| card_item(vertical_card(item)))
        .collect();

    div(
        vec![("class", "blocks-grid-list-contact-cards-layout")],
        vec![
            instance("horizontal", "横型カード（主参照）", horizontal_cards),
            instance("vertical", "縦型カード（集約元）", vertical_cards),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/grid-list-contact-cards/",
    title: "grid-list-contact-cards",
    category: BlockCategory::GridList,
    rust_source: "crates/docs-site/src/blocks/application/grid_list/grid_list_contact_cards.rs",
    demo_class: "blocks-grid-list-contact-cards",
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
            label: "Badge",
            path: "/themes/badge/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Icon",
            path: "/themes/icon/",
        },
        Part {
            label: "List",
            path: "/themes/list/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `grid_list_contact_cards` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節と同型）。既定（狭幅）は 1 列、`36rem`
/// 以上で 2 列、`52rem` 以上で 3 列、`vertical` インスタンスのみ `68rem`
/// 以上でさらに 4 列へ増やす（モジュール doc「`@container` で列数を
/// 切り替える理由」節）。
const LAYOUT_CSS: &str = "\
.blocks-grid-list-contact-cards-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-10);\n}\n\
.blocks-grid-list-contact-cards-layout > [data-blocks-grid-list-contact-cards-variant] {\n  container-type: inline-size;\n  container-name: blocks-grid-list-contact-cards;\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-grid-list-contact-cards-label {\n  color: var(--fandhe-color-fg-muted);\n  font-size: 0.875rem;\n}\n\
[data-blocks-grid-list-contact-cards-grid] {\n  display: grid;\n  grid-template-columns: minmax(0, 1fr);\n  gap: var(--fandhe-space-6);\n  margin: 0;\n  padding: 0;\n}\n\
.blocks-grid-list-contact-cards-item {\n  min-width: 0;\n}\n\
.blocks-grid-list-contact-cards-item [data-scope=\"card\"][data-part=\"root\"] {\n  display: flex;\n  flex: 1;\n  flex-direction: column;\n  width: 100%;\n  height: 100%;\n  overflow: hidden;\n}\n\
[data-scope=\"card\"][data-part=\"body\"].blocks-grid-list-contact-cards-main {\n  display: flex;\n  flex-direction: row;\n  justify-content: space-between;\n  align-items: center;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-grid-list-contact-cards-info {\n  display: flex;\n  flex-direction: column;\n  align-items: flex-start;\n  gap: var(--fandhe-space-1);\n  min-width: 0;\n}\n\
.blocks-grid-list-contact-cards-role {\n  color: var(--fandhe-color-fg-muted);\n  font-size: 0.875rem;\n}\n\
.blocks-grid-list-contact-cards-stack {\n  display: flex;\n  flex-direction: column;\n  align-items: center;\n  text-align: center;\n  gap: var(--fandhe-space-2);\n}\n\
[data-blocks-grid-list-contact-cards-avatar] {\n  flex-shrink: 0;\n}\n\
[data-scope=\"card\"][data-part=\"footer\"].blocks-grid-list-contact-cards-actions {\n  display: flex;\n  padding: 0;\n  border-top: 1px solid var(--fandhe-color-border);\n  margin-top: auto;\n}\n\
[data-scope=\"button\"][data-part=\"root\"][data-blocks-grid-list-contact-cards-action] {\n  flex: 1;\n  border-radius: 0;\n  display: flex;\n  align-items: center;\n  justify-content: center;\n  gap: var(--fandhe-space-2);\n}\n\
[data-blocks-grid-list-contact-cards-action] + [data-blocks-grid-list-contact-cards-action] {\n  border-inline-start: 1px solid var(--fandhe-color-border);\n}\n\
@container blocks-grid-list-contact-cards (min-width: 36rem) {\n  \
[data-blocks-grid-list-contact-cards-grid] {\n    grid-template-columns: repeat(2, minmax(0, 1fr));\n  }\n\
}\n\
@container blocks-grid-list-contact-cards (min-width: 52rem) {\n  \
[data-blocks-grid-list-contact-cards-grid] {\n    grid-template-columns: repeat(3, minmax(0, 1fr));\n  }\n\
}\n\
@container blocks-grid-list-contact-cards (min-width: 68rem) {\n  \
[data-blocks-grid-list-contact-cards-grid][data-blocks-grid-list-contact-cards-grid-variant=\"vertical\"] {\n    grid-template-columns: repeat(4, minmax(0, 1fr));\n  }\n\
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
            "data-scope=\"avatar\"",
            "data-scope=\"badge\"",
            "data-scope=\"button\"",
            "data-scope=\"list\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
        assert!(
            html.contains("<svg"),
            "demo should render icon svg elements"
        );
        assert!(html.contains(crate::blocks::dummy_assets::AVATAR_SRC));
    }

    #[test]
    fn no_form_semantics_or_dead_links() {
        let html = demo_html();
        assert_eq!(html.matches(r#"type="button""#).count(), 2 * (6 + 8));
        assert!(!html.contains("<form"));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("href="));
        assert!(!html.contains("src=\"data:"));
    }

    #[test]
    fn both_variants_are_present_exactly_once() {
        let html = demo_html();
        for variant in ["horizontal", "vertical"] {
            let attr = format!(r#"data-blocks-grid-list-contact-cards-variant="{variant}""#);
            assert_eq!(
                html.matches(&attr).count(),
                1,
                "variant {variant} should appear exactly once"
            );
        }
    }

    #[test]
    fn card_and_list_counts_match_instances() {
        let html = demo_html();
        assert_eq!(
            html.matches("data-scope=\"card\" data-part=\"root\"")
                .count(),
            6 + 8
        );
        assert_eq!(html.matches("<ul").count(), 2);
    }

    #[test]
    fn layout_css_contract() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("container-type: inline-size;"));
        assert!(LAYOUT_CSS.contains("@container blocks-grid-list-contact-cards (min-width: 36rem)"));
        assert!(LAYOUT_CSS.contains("@container blocks-grid-list-contact-cards (min-width: 52rem)"));
        assert!(LAYOUT_CSS.contains("@container blocks-grid-list-contact-cards (min-width: 68rem)"));
        assert!(LAYOUT_CSS.contains("repeat(4, minmax(0, 1fr));"));
        // 縦型限定の 4 列判定は grid 要素自身の属性で行う（コンテナー自身
        // への祖先参照を含む子孫セレクタは @container 内で一致しないため、
        // モジュール doc「`@container` で列数を切り替える理由」節参照）。
        assert!(LAYOUT_CSS.contains(
            "[data-blocks-grid-list-contact-cards-grid][data-blocks-grid-list-contact-cards-grid-variant=\"vertical\"]"
        ));
        // horizontal card の main は card::body recipe（詳細度 (0,2,0,0)）に
        // 勝つため複合セレクタ（詳細度 (0,3,0,0)）で flex-direction を
        // 明示上書きする（モジュール doc「CSS フックの選び方」節参照）。
        assert!(LAYOUT_CSS.contains(
            "[data-scope=\"card\"][data-part=\"body\"].blocks-grid-list-contact-cards-main {\n  display: flex;\n  flex-direction: row;"
        ));
    }

    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = demo_html();
        assert!(html.contains("class=\"blocks-grid-list-contact-cards-layout\""));
        assert_ne!(
            super::BLOCK.demo_class,
            "blocks-grid-list-contact-cards-layout"
        );
    }
}
