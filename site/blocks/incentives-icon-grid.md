# incentives-icon-grid

`heading` / `text` / `icon` / `image` / `card` / `item` / `visually-hidden`
の 7 部品を合成した、特典（送料無料・返品保証・ギフト包装・ポイント還元
など）をアイコンやイラスト付きの項目グリッドで紹介する合成例です。

5 つの静的インスタンスを縦に並べています。1 つ目は見出し + 導入文の下へ、
枠付きカード 3 件（アイコン・題名・説明）を並べる基準形です。2 つ目は
見出しを視覚的に隠し、淡色の角丸枠で囲んだアイコン付き項目を 4 件、
カードなしで並べます（見出し自体はアクセシビリティツリーに残ります）。
3 つ目は中央寄せの見出しの下へ、アイコン上・テキスト中央の項目を 3 件
並べる簡潔な配置です。4 つ目は見出しを持たず、アイコン左・文章右の横並び
を 3 行並べます。5 つ目は淡色パネルの中に中央見出しと装飾イラストを
4 点並べます。いずれも幅が狭いときは 1〜2 列に自動で折り返します。
`<form>` やボタン・リンクは使わず、文言・画像はすべて架空のもの（実在の
企業名や個人情報は含みません）を独自に書いた静的な表示例です。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps, CardVariant};
use fandhe_frontend_pre_styled_ui::heading::{
    self, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::image::{image, ImageProps};
use fandhe_frontend_pre_styled_ui::item::{self, ItemMediaVariant, ItemRootProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::visually_hidden;
use fandhe_frontend_pre_styled_ui::Size;

/// 特典 1 件分の架空データ（題名・説明・自作アイコンのパス）。
struct Perk {
    title: &'static str,
    body: &'static str,
    icon_path_d: &'static str,
}

/// 特典 4 件（架空、実在の企業・製品とは無関係）。自作の単純な幾何パス
/// のみを使い、lucide 等の著作物は複製しない。先頭 3 件を A/C/D インス
/// タンスで使い回す。
const PERKS: [Perk; 4] = [
    Perk {
        title: "送料無料",
        body: "一定金額以上のご注文で、配送料は一切かかりません。",
        icon_path_d: "M3 7h11v9H3zM14 10h4l3 3v3h-7z\
            M7 20a2 2 0 100-4 2 2 0 000 4zM17 20a2 2 0 100-4 2 2 0 000 4z",
    },
    Perk {
        title: "30 日間返品保証",
        body: "ご注文から 30 日以内であれば理由を問わず返品できます。",
        icon_path_d: "M4 4v6h6M4 10a8 8 0 1114.93 4.93",
    },
    Perk {
        title: "ギフト包装",
        body: "ご注文時にチェックするだけで、包装紙とメッセージカードを添えます。",
        icon_path_d: "M3 10h18v10H3zM3 10V6h18v4M12 2l3 4H9z\
            M12 10v10",
    },
    Perk {
        title: "ポイント還元",
        body: "ご購入金額に応じてポイントが貯まり、次回以降の買い物に使えます。",
        icon_path_d: "M12 2l2.6 5.3 5.9.9-4.3 4.1 1 5.8L12 15.9 6.8 18l1-5.8-4.3-4.1 5.9-.9z",
    },
];

/// 装飾用の自作幾何アイコン（`fill="none"` + `stroke="currentColor"` の
/// 線画、`feature_three_column_icons::geo_icon` と同型の判断）。
fn geo_icon(path_d: &'static str, size: Size, attrs: Vec<(&'static str, &'static str)>) -> Node {
    icon(
        &IconProps {
            size,
            ..IconProps::default()
        },
        attrs,
        vec![el(
            "path",
            vec![
                ("d", path_d),
                ("fill", "none"),
                ("stroke", "currentColor"),
                ("stroke-width", "2"),
                ("stroke-linecap", "round"),
                ("stroke-linejoin", "round"),
            ],
            vec![],
        )],
    )
}

/// 中央寄せの導入見出し（A/C/E 用）。`sr_only` が `true` のとき見出し文言を
/// [`visually_hidden::root`] で包み、アクセシビリティツリーにのみ残す
/// （モジュール doc「見出しを視覚的に隠す配置」節参照）。
fn header(title: &'static str, lead: Option<&'static str>, sr_only: bool) -> Node {
    let heading_children = if sr_only {
        vec![visually_hidden::root(vec![], vec![text(title)])]
    } else {
        vec![text(title)]
    };
    let mut children = vec![heading::heading(
        HeadingLevel::H3,
        &HeadingProps {
            size: HeadingSize::Xl2,
            weight: HeadingWeight::Bold,
        },
        if sr_only {
            vec![("data-blocks-incentives-icon-grid-sr-heading", "")]
        } else {
            vec![]
        },
        heading_children,
    )];
    if let Some(lead) = lead {
        children.push(styled_text::text(
            &TextProps {
                variant: TextVariant::Muted,
                ..TextProps::default()
            },
            vec![("data-blocks-incentives-icon-grid-lead", "")],
            vec![text(lead)],
        ));
    }
    let mut wrapper_attrs = vec![("class", "blocks-incentives-icon-grid-header")];
    if sr_only {
        wrapper_attrs.push(("data-sr-only", ""));
    }
    div(wrapper_attrs, children)
}

/// 項目題名 + 説明（アイコン・カードの有無は呼び出し側が組み立てる）。
fn perk_text(p: &Perk) -> Vec<Node> {
    vec![
        heading::heading(
            HeadingLevel::H4,
            &HeadingProps::default(),
            vec![],
            vec![text(p.title)],
        ),
        styled_text::text(
            &TextProps {
                size: TextSize::Sm,
                variant: TextVariant::Muted,
                ..TextProps::default()
            },
            vec![("data-blocks-incentives-icon-grid-desc", "")],
            vec![text(p.body)],
        ),
    ]
}

/// A 用: 枠付きカードへ収めたアイコン付き項目。
fn perk_card(p: &Perk) -> Node {
    let mut body_children = vec![geo_icon(
        p.icon_path_d,
        Size::Lg,
        vec![("data-blocks-incentives-icon-grid-icon", "")],
    )];
    body_children.extend(perk_text(p));
    card::root(
        CardProps {
            variant: CardVariant::Outline,
            ..CardProps::default()
        },
        vec![("data-blocks-incentives-icon-grid-card", "")],
        vec![card::body(
            vec![("data-blocks-incentives-icon-grid-card-body", "")],
            body_children,
        )],
    )
}

/// B 用: 淡色の角丸枠で囲んだアイコン + 題名 + 説明（カードなし）。
fn perk_badge_icon(p: &Perk) -> Node {
    let mut children = vec![div(
        vec![("class", "blocks-incentives-icon-grid-badge")],
        vec![geo_icon(
            p.icon_path_d,
            Size::Md,
            vec![("data-blocks-incentives-icon-grid-icon", "")],
        )],
    )];
    children.extend(perk_text(p));
    div(
        vec![
            ("class", "blocks-incentives-icon-grid-item"),
            ("data-align", "center"),
        ],
        children,
    )
}

/// C 用: 中央寄せ（アイコン上・テキスト中央、カードなし・バッジなし）。
fn perk_centered(p: &Perk) -> Node {
    let mut children = vec![geo_icon(
        p.icon_path_d,
        Size::Lg,
        vec![("data-blocks-incentives-icon-grid-icon", "")],
    )];
    children.extend(perk_text(p));
    div(
        vec![
            ("class", "blocks-incentives-icon-grid-item"),
            ("data-align", "center"),
        ],
        children,
    )
}

/// D 用: アイコン左・文章右の横並び 1 行（`item` anatomy）。
fn perk_item_row(p: &Perk) -> Node {
    item::root(
        ItemRootProps::default(),
        vec![("data-blocks-incentives-icon-grid-item-row", "")],
        vec![
            item::media(
                ItemMediaVariant::Icon,
                vec![],
                vec![geo_icon(p.icon_path_d, Size::Md, vec![])],
            ),
            item::content(
                vec![],
                vec![
                    item::title(vec![], vec![text(p.title)]),
                    item::description(vec![], vec![text(p.body)]),
                ],
            ),
        ],
    )
}

/// E 用: 淡色パネル内の装飾イラスト 1 枚 + 項目題名（アイコン・説明は
/// 持たず、パネルが特典紹介であることを見出しと題名のみで示す簡潔形）。
fn perk_illustration(p: &Perk) -> Node {
    div(
        vec![
            ("class", "blocks-incentives-icon-grid-item"),
            ("data-align", "center"),
        ],
        vec![
            image(
                &ImageProps::new(dummy_assets::PRODUCT_SRC, ""),
                vec![("data-blocks-incentives-icon-grid-illustration", "")],
            ),
            heading::heading(
                HeadingLevel::H4,
                &HeadingProps::default(),
                vec![],
                vec![text(p.title)],
            ),
        ],
    )
}

/// 注記行（各インスタンスの原案差分の要約、`data-*` フックのみで class を
/// 持たない共通パーツ）。
fn note(body: &'static str) -> Node {
    styled_text::text(
        &TextProps {
            size: TextSize::Xs,
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![("data-blocks-incentives-icon-grid-note", "")],
        vec![text(body)],
    )
}

/// 1 インスタンス分（任意の見出し + 注記 + グリッド）を組み立てる。
fn instance(
    header_node: Option<Node>,
    note_body: &'static str,
    grid_items: Vec<Node>,
    extra_attrs: Vec<(&'static str, &'static str)>,
) -> Node {
    let mut attrs = vec![("class", "blocks-incentives-icon-grid-instance")];
    attrs.extend(extra_attrs);
    let mut children = Vec::new();
    if let Some(header_node) = header_node {
        children.push(header_node);
    }
    children.push(note(note_body));
    children.push(div(
        vec![("class", "blocks-incentives-icon-grid-grid")],
        grid_items,
    ));
    div(attrs, children)
}

/// `incentives-icon-grid` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数（他 block と同じ状態を持たない設計）。
#[must_use]
pub fn demo() -> Node {
    let instance_a = instance(
        Some(header(
            "ご注文いただくみなさまへの特典",
            Some("当店でのお買い物にいつも付いてくる、3 つの特典です。"),
            false,
        )),
        "基準形。見出し + 導入文 + 枠付きカード（Outline）3 件。",
        PERKS[..3].iter().map(perk_card).collect(),
        vec![],
    );

    let instance_b = instance(
        Some(header("特典一覧（視覚的に非表示）", None, true)),
        "見出しを視覚的に隠し、淡色の角丸枠で囲んだアイコン付き項目を 4 件並べる。カードなし。",
        PERKS.iter().map(perk_badge_icon).collect(),
        vec![("data-count", "4")],
    );

    let instance_c = instance(
        Some(header(
            "シンプルに伝える特典",
            Some("アイコンを中央に置き、テキストも中央寄せにした簡潔な配置です。"),
            false,
        )),
        "中央見出し。アイコン上・テキスト中央の 3 件（カード・バッジなし）。",
        PERKS[..3].iter().map(perk_centered).collect(),
        vec![],
    );

    let instance_d = instance(
        None,
        "見出しなし。アイコン左・文章右の横並び 3 件（item anatomy）。",
        PERKS[..3].iter().map(perk_item_row).collect(),
        vec![("data-layout", "rows")],
    );

    let instance_e = instance(
        Some(header(
            "届いてからも楽しい",
            Some("特典の様子を、イラストでも紹介します。"),
            false,
        )),
        "淡色パネル内に中央見出し + 装飾イラスト 4 点（alt は空、装飾画像）。",
        PERKS.iter().map(perk_illustration).collect(),
        vec![("data-panel", "subtle"), ("data-count", "4")],
    );

    div(
        vec![("class", "blocks-incentives-icon-grid-layout")],
        vec![instance_a, instance_b, instance_c, instance_d, instance_e],
    )
}
```

## 原案差分メモ

- 対応表 ID R0555〜R0559・R1016〜R1021（主参照 R1017）の集約元 11 件を、
  レイアウト仕様に沿って 5 インスタンスへ統合しました。
- インスタンス A は主参照 R1017 と R0555 を代表させた枠付きカード構成、
  B は R0557/R0556/R1021/R1020 を集約した枠なし・淡色バッジ構成、C は
  R0558/R1018 を集約した中央寄せ構成、D は R0559 を反映したアイコン左・
  文章右の横並び構成、E は R1019/R1016 を集約した淡色パネル + イラスト
  構成です。
- B の見出しは `visually_hidden::root` で視覚的に隠していますが、見出し
  要素自体（`h3`）はアクセシビリティツリーに残ります（`span` で `h3` を
  包むのではなく、`h3` の中へ `visually_hidden::root` を入れる構成）。
- 参照元の文言・配色・アイコン意匠は持ち込まず、デモ文言（送料無料・
  返品保証・ギフト包装・ポイント還元）はすべて独自に書いた架空のものです。
- `<form>`・ボタン・リンクは使わず、状態も持たない静的な表示例です。
- ブラウザでの実機確認（狭幅・広幅の折り返し、ライト/ダーク両テーマ）は
  サンドボックス制約により未実施です。`cargo test` による出力検証のみで
  代替しました。
