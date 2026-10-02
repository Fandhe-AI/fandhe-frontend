//! `list-container` block（イシュー #3227。Application/List カテゴリ）。
//! 主参照は対応表 ID R1047（区切り線付きの単純リスト）。R1048/R1052/R1050
//! を別インスタンスとして併記し、`item`/`card`/`separator` の 3 部品のみで
//! 「容器」バリエーションの一覧を組む。`_/blocks-intake/` の対応ファイルは
//! 本イシュー着手時点で本 worktree に存在しないため、原稿・本コメントには
//! 対応表 ID のみを記す（`list-title-meta`〔イシュー #3370〕と同じ扱い）。
//!
//! # 使用部品
//!
//! `item` / `card` / `separator` の 3 部品を合成する（[`BLOCK`] の `parts`
//! に一致させる契約）。新しい UI 部品は追加しない。
//!
//! # 4 形態と対応表 ID
//!
//! 1. **区切り線付きの単純リスト**（R1047、主参照）: `item::group` の中で
//!    `item::root` を [`separator::separator`] で区切って並べる。
//! 2. **外枠付きカード内のリスト**（R1048）: 1 と同じ骨格を
//!    `card::root(CardVariant::Elevated)` の中に収める。
//! 3. **枠線のみのカード内リスト**（R1052）: 2 と同じ骨格を
//!    `CardVariant::Outline` に差し替える（狭幅対応の集約元がないため
//!    インスタンスは 1 つのみ）。
//! 4. **項目ごとに独立したカード**（R1050）: `item::group` の中で各行を
//!    個別の `card::root(CardVariant::Elevated)` に包み、区切り線の代わりに
//!    `gap` で間隔を空ける。
//!
//! # 狭幅差分を `@container` + 幅制限インスタンスで静的に併記する理由
//! （R1049/R1051/R1053）
//!
//! 本 docs サイトは JS ハイドレーションを行わない静的出力のため、実際の
//! コンテナ幅リサイズは示せない。1・2・4 の各セクションは「通常幅」と
//! 「狭幅」（`.blocks-list-container-narrow` で `max-width` を制限した
//! パネル）の 2 インスタンスを並べ、[`LAYOUT_CSS`] の
//! `@container blocks-list-container (max-width: 30rem)` 規則が両方へ
//! 一貫して適用されることで狭幅時の見た目差分（角丸解除・全幅化・左右
//! 余白解除）を静的に併記する。3（R1052）は狭幅対応の集約元がないため
//! 単一インスタンスのみとする。通常幅パネルには `min-width: min(31rem,
//! 100%)`（コンテナクエリのしきい値 30rem を上回る値。ただし `100%` と
//! の `min()` で実ビューポート幅が 31rem を下回る場合は縮小を許す）を
//! 明示し、`flex-wrap` によるパネル自体の収縮でしきい値を割り込んで
//! 狭幅スタイルが誤適用されないよう固定する（`@container` はパネル自身
//! の実測幅を見るため、`flex-basis` だけでは収縮時にしきい値を下回り得
//! る）。Demo 枠の実効幅は `.docs-content` の `max-width: 46rem` から
//! `.blocks-demo` の左右 padding 3rem を引いた 43rem 程度（他 block の
//! 同種算出と同じ基準、`grid_list_logo_cards` 参照）。通常幅パネル
//! （30.5rem、30rem のコンテナクエリしきい値を上回る値）+ 狭幅パネル
//! （10rem）+ 間隔（1rem）の合計 41.5rem はこの 43rem に収まり、
//! `flex-wrap` による意図しない折り返しを起こさない（イシュー #3227
//! PR #3403 のレビュー指摘 P1 再対応）。`min-width: min(..., 100%)` は
//! 同時に、画面幅が 41.5rem を下回る場合に通常幅パネルが固定下限で横
//! スクロールを強制する問題（同 P2 指摘）も避ける。
//!
//! # `card::body` の padding を 0 にする理由
//!
//! `card::body` は既定で `--fandhe-card-padding` 分の内側余白を持つが、
//! 本 block はカード内側の余白を `item::root` 自身の既定 padding に委ね、
//! 区切り線がカード内壁まで到達する密なリストを志向するため、
//! `[data-blocks-list-container-body]` フックで `padding: 0` に上書きする。
//!
//! # `item::root` の既定角丸を解除する理由
//!
//! `item::root` は既定で `border-radius: var(--fandhe-radius-md)` を持つが、
//! 隣接行を区切り線でつなげる本 block の構成では角丸が不要（かつ
//! カード内では二重の角丸に見え得る）ため、`[data-blocks-list-container-row]`
//! フックで `border-radius: 0` に上書きする。左右余白の解除（R1053）は
//! 単純リストの行（`data-blocks-list-container-row="plain"`）のみに限定する
//! （カード内の行はカード自体の外枠が余白の境界を担うため対象外）。
//!
//! # `<form>` を使わない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない。対話要素（ボタン・リンク）も持たない静的な表示例である。
//!
//! # ダミー素材について
//!
//! 行のタイトル・説明文はすべて独自に書いた架空の文言であり、実在の
//! 人物・企業・プロジェクトとは無関係。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::card::{self, CardVariant};
use fandhe_frontend_pre_styled_ui::item::{self, ItemRootProps};
use fandhe_frontend_pre_styled_ui::separator::{self, SeparatorProps};

/// 各行のダミーデータ（タイトル + 説明、架空の文言）。
const ROWS: [(&str, &str); 4] = [
    (
        "週次レポートの自動生成",
        "毎週月曜 9 時に集計結果を配信します。",
    ),
    (
        "在庫アラートのしきい値見直し",
        "欠品率を抑えるため通知条件を再設定します。",
    ),
    (
        "サポート返信テンプレートの整理",
        "よくある問い合わせへの定型文を見直します。",
    ),
    (
        "バックアップ保存先の切り替え",
        "月次バックアップの保存先を新ストレージへ移行します。",
    ),
];

/// 1 行分の `item::root`。`variant` は
/// `[data-blocks-list-container-row]` フックの値で、`"plain"`
/// （単純リスト行、R1053 の左右余白解除対象）と `"framed"`（カード内・
/// 独立カード行）を区別する（モジュール doc「`item::root` の既定角丸を
/// 解除する理由」節参照）。
fn row(title: &str, description: &str, variant: &str) -> Node {
    item::root(
        ItemRootProps::default(),
        vec![("data-blocks-list-container-row", variant)],
        vec![item::content(
            vec![],
            vec![
                item::title(vec![], vec![text(title)]),
                item::description(vec![], vec![text(description)]),
            ],
        )],
    )
}

/// [`ROWS`] を `variant` フック付きの行 + [`separator::separator`] で
/// 交互に並べた `Vec<Node>`（行数 - 1 本の区切り線）。
fn separated_rows(variant: &str) -> Vec<Node> {
    let mut nodes = Vec::with_capacity(ROWS.len() * 2 - 1);
    for (i, (title, description)) in ROWS.iter().enumerate() {
        if i > 0 {
            nodes.push(separator::separator(&SeparatorProps::default(), vec![]));
        }
        nodes.push(row(title, description, variant));
    }
    nodes
}

/// R1047: 区切り線付きの単純リスト。
fn plain_list() -> Node {
    item::group("シンプルなリスト", vec![], separated_rows("plain"))
}

/// R1048/R1052: `variant` のカードで包んだ区切り線付きリスト。
fn framed_card_list(variant: CardVariant, label: &'static str) -> Node {
    card::root(
        variant,
        vec![("data-blocks-list-container-card", "")],
        vec![card::body(
            vec![("data-blocks-list-container-body", "")],
            vec![item::group(label, vec![], separated_rows("framed"))],
        )],
    )
}

/// R1050: 項目ごとに独立したカード（区切り線の代わりに `gap` で分離する）。
fn independent_cards() -> Node {
    let cards: Vec<Node> = ROWS
        .iter()
        .map(|(title, description)| {
            card::root(
                CardVariant::Elevated,
                vec![("data-blocks-list-container-card", "")],
                vec![card::body(
                    vec![("data-blocks-list-container-body", "")],
                    vec![row(title, description, "framed")],
                )],
            )
        })
        .collect();
    item::group(
        "項目ごとのカード",
        vec![("data-blocks-list-container-cards", "")],
        cards,
    )
}

/// パネル外枠（`@container` の名前付きコンテナ）。`narrow` が `true` なら
/// 幅を制限した狭幅インスタンス用クラスを追加する（モジュール doc「狭幅
/// 差分を `@container` + 幅制限インスタンスで静的に併記する理由」節）。
fn panel(narrow: bool, node: Node) -> Node {
    let class = if narrow {
        "blocks-list-container-panel blocks-list-container-narrow"
    } else {
        "blocks-list-container-panel"
    };
    div(vec![("class", class)], vec![node])
}

/// 通常幅・狭幅の 2 インスタンスを横並びで示す。
fn pair(normal: Node, narrow: Node) -> Node {
    div(
        vec![("class", "blocks-list-container-pair")],
        vec![panel(false, normal), panel(true, narrow)],
    )
}

/// 見出し付きセクションを組み立てる（`list_title_meta` と同型のヘルパ）。
fn section(label: &'static str, node: Node) -> Node {
    div(
        vec![("class", "blocks-list-container-section")],
        vec![
            el(
                "h3",
                vec![("class", "blocks-list-container-section-title")],
                vec![text(label)],
            ),
            node,
        ],
    )
}

/// `list-container` の Demo 本体。呼び出しごとに同一の `Node` を返す純関数。
/// 主参照（R1047）を先頭に、R1048/R1052/R1050 をラベル付き見出しで区切って
/// 並べる。
#[must_use]
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-list-container-layout")],
        vec![
            section("区切り線付きの単純リスト", pair(plain_list(), plain_list())),
            section(
                "外枠付きカード内のリスト",
                pair(
                    framed_card_list(CardVariant::Elevated, "外枠付きカードのリスト"),
                    framed_card_list(CardVariant::Elevated, "外枠付きカードのリスト"),
                ),
            ),
            section(
                "枠線のみのカード内リスト",
                panel(
                    false,
                    framed_card_list(CardVariant::Outline, "枠線のみのカードのリスト"),
                ),
            ),
            section(
                "項目ごとに独立したカード",
                pair(independent_cards(), independent_cards()),
            ),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/list-container/",
    title: "list-container",
    category: BlockCategory::List,
    rust_source: "crates/docs-site/src/blocks/application/list/list_container.rs",
    demo_class: "blocks-list-container",
    parts: &[
        Part {
            label: "Item",
            path: "/themes/item/",
        },
        Part {
            label: "Card",
            path: "/themes/card/",
        },
        Part {
            label: "Separator",
            path: "/themes/separator/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `list_container` 固有のレイアウト規則（`crate::blocks`
/// モジュール doc「CSS の置き場」節と同型）。
const LAYOUT_CSS: &str = "\
.blocks-list-container-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n}\n\
.blocks-list-container-section {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-list-container-section-title {\n  margin: 0;\n  font-size: var(--fandhe-font-font-size-sm);\n  font-weight: var(--fandhe-font-font-weight-medium);\n  color: var(--fandhe-color-fg-muted);\n}\n\
.blocks-list-container-pair {\n  display: flex;\n  flex-wrap: wrap;\n  align-items: flex-start;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-list-container-panel {\n  flex: 1 1 20rem;\n  min-width: min(30.5rem, 100%);\n  container-type: inline-size;\n  container-name: blocks-list-container;\n}\n\
.blocks-list-container-narrow {\n  flex: 0 1 10rem;\n  max-width: 10rem;\n  min-width: 0;\n}\n\
[data-scope=\"card\"][data-part=\"body\"][data-blocks-list-container-body] {\n  padding: 0;\n}\n\
[data-scope=\"item\"][data-part=\"group\"][data-blocks-list-container-cards] {\n  gap: var(--fandhe-space-4);\n}\n\
[data-scope=\"item\"][data-part=\"root\"][data-blocks-list-container-row] {\n  border-radius: 0;\n}\n\
@container blocks-list-container (max-width: 30rem) {\n  \
[data-scope=\"card\"][data-part=\"root\"][data-blocks-list-container-card] {\n    border-radius: 0;\n    border-inline-width: 0;\n  }\n  \
[data-scope=\"item\"][data-part=\"root\"][data-blocks-list-container-row=\"plain\"] {\n    padding-inline: 0;\n  }\n\
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
            "data-scope=\"item\"",
            "data-scope=\"card\"",
            "data-scope=\"separator\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
        // 単純リスト: 3 区切り線 x2 インスタンス、カード内リスト: 3 区切り線
        // x2（Elevated）+ 3（Outline）、独立カードは区切り線を持たない。
        assert_eq!(html.matches("role=\"separator\"").count(), 6 + 6 + 3);
        // card root: Elevated x2 + Outline x1 + 独立カード 4 行 x2 インスタンス。
        assert_eq!(
            html.matches("data-scope=\"card\" data-part=\"root\"")
                .count(),
            2 + 1 + 4 * 2
        );
    }

    #[test]
    fn narrow_instances_are_marked() {
        let html = demo_html();
        assert_eq!(html.matches("blocks-list-container-narrow").count(), 3);
    }

    #[test]
    fn no_form_or_unsafe_markup() {
        let html = demo_html();
        assert!(!html.contains("<form"));
        assert!(!html.contains("<script"));
        assert!(!html.contains("src=\"data:"));
        assert!(!html.contains("href="));
    }

    #[test]
    fn layout_css_is_safe() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("@container blocks-list-container (max-width: 30rem)"));
        assert!(LAYOUT_CSS.contains("border-radius: 0;"));
    }

    #[test]
    fn demo_is_deterministic() {
        assert_eq!(demo_html(), demo_html());
    }
}
