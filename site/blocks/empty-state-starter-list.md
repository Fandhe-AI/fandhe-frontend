# empty-state-starter-list

`fandhe-frontend-pre-styled-ui` の `item` / `icon` / `heading` / `text` /
`separator` / `link` の 6 部品を合成した、開始候補（テンプレート等）を
縦一覧で並べる空状態の合成例です。Blocks セクションは新規部品を追加する
ものではなく、既存の Themes/Primitives 部品を組み合わせた実例集である
ことに注意してください（主参照は対応表 ID R0465、集約元は R1397 です。
出典の固有名・ファイル名は記載しません）。

見出し + 説明の下に、区切り線付きの縦リストを並べます。各行はアイコン・
題名・説明・行末シェブロンで構成され、行全体がリンクです。リストは幅に
かかわらず常に縦積みで、横並びへ崩れることはありません。末尾には
「別の始め方」への導線リンクを置きます。Demo の各行 `href` はサイト内の
索引ページを指しますが、実際の利用時は自分のテンプレート一覧・作成
ページの URL へ差し替えてください。

版 A（代表構成、R0465）は開始候補 4 行、版 B（R1397 寄り）は開始候補
3 行 + 行末シェブロンを強調する `data-variant="chevron-emphasis"` を
並記しています。差分の詳細は下記「原案差分メモ」を参照してください。

本 Demo は静的な表示例で、`<form>` 要素を出力せず、遷移処理・送信処理・
状態管理は一切持ちません。文言はすべて独自の架空ダミーで、実在の人物・
企業・実クレデンシャルは含みません。

## Rust コード

```rust
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::icon::{self, IconProps};
use fandhe_frontend_pre_styled_ui::item::{self, ItemMediaVariant, ItemRootProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::separator::{self, SeparatorProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};

const STACK_CLASS: &str = "blocks-empty-state-starter-list-stack";
const INSTANCE_CLASS: &str = "blocks-empty-state-starter-list-instance";
const HEADER_CLASS: &str = "blocks-empty-state-starter-list-header";
const LIST_CLASS: &str = "blocks-empty-state-starter-list-list";
const FOOTER_CLASS: &str = "blocks-empty-state-starter-list-footer";

const HEADING_ATTR: &str = "data-blocks-empty-state-starter-list-heading";
const DESCRIPTION_ATTR: &str = "data-blocks-empty-state-starter-list-description";
const ROW_ATTR: &str = "data-blocks-empty-state-starter-list-row";
const MEDIA_ATTR: &str = "data-blocks-empty-state-starter-list-media";
const CHEVRON_ATTR: &str = "data-blocks-empty-state-starter-list-chevron";
const RULE_ATTR: &str = "data-blocks-empty-state-starter-list-rule";
const FOOTER_TEXT_ATTR: &str = "data-blocks-empty-state-starter-list-footer-text";
const FOOTER_LINK_ATTR: &str = "data-blocks-empty-state-starter-list-footer-link";

/// 開始候補 1 行分のデータ（モジュール doc「href の方針」節参照。
/// `icon_path_d` は `stroke` 系の自作幾何アイコンの `d` 属性値）。
struct Starter {
    href: &'static str,
    title: &'static str,
    description: &'static str,
    icon_path_d: &'static str,
}

/// 開始候補一覧（サイト内に実在する索引ページのみを指す）。
const STARTERS: [Starter; 4] = [
    Starter {
        href: "../../guides/",
        title: "空のプロジェクト",
        description: "最小構成から自分で組み立てます。",
        icon_path_d: "M12 4v16M4 12h16",
    },
    Starter {
        href: "../../examples/",
        title: "サンプル一式から始める",
        description: "動く実装例をコピーして手を加えます。",
        icon_path_d: "M4 4h16v16H4z M4 9h16",
    },
    Starter {
        href: "../../primitives/",
        title: "部品カタログから組む",
        description: "構造だけの部品を積み上げて作ります。",
        icon_path_d: "M4 4h7v7H4z M13 13h7v7h-7z",
    },
    Starter {
        href: "../../themes/",
        title: "スタイル済みテーマから始める",
        description: "見た目まで整った部品をそのまま使います。",
        icon_path_d: "M4 4h16v6H4z M4 14h16v6H4z",
    },
];

/// 線画（stroke）の自作幾何アイコンを組み立てる
/// （`error_page_popular_links::stroke_icon` と同型のパターン。装飾用途
/// のため `IconProps::label` は付けない）。
fn stroke_icon(path_d: &'static str) -> Node {
    icon::icon(
        &IconProps::default(),
        vec![],
        vec![el(
            "path",
            vec![
                ("d", path_d),
                ("fill", "none"),
                ("stroke", "currentColor"),
                ("stroke-width", "1.5"),
                ("stroke-linecap", "round"),
                ("stroke-linejoin", "round"),
            ],
            vec![],
        )],
    )
}

/// 行末のシェブロンアイコン（全行共通、装飾用途）。
fn chevron_icon() -> Node {
    stroke_icon("m9 6 6 6-6 6")
}

/// 開始候補 1 行分を組み立てる（行全体を `item::root` の `<a>` にする。
/// モジュール doc「行全体がリンクになる仕組み」節参照）。
fn starter_row(starter: &Starter) -> Node {
    item::root(
        ItemRootProps {
            href: Some(starter.href),
            ..ItemRootProps::default()
        },
        vec![(ROW_ATTR, "")],
        vec![
            item::media(
                ItemMediaVariant::Icon,
                vec![(MEDIA_ATTR, "")],
                vec![stroke_icon(starter.icon_path_d)],
            ),
            item::content(
                vec![],
                vec![
                    item::title(vec![], vec![text(starter.title)]),
                    item::description(vec![], vec![text(starter.description)]),
                ],
            ),
            item::actions(vec![(CHEVRON_ATTR, "")], vec![chevron_icon()]),
        ],
    )
}

/// 開始候補一覧を組み立てる（行と行の間にのみ `separator` を挟み、末尾
/// には置かない。モジュール doc「`list` を使わない理由」節参照）。
///
/// 各行は `role="listitem"` を付けた `div` で包み、一覧全体のコンテナには
/// `role="list"` を付ける（支援技術へ一覧・項目数を伝えるための素の
/// ARIA 属性付与。モジュール doc「`list` を使わない理由」節参照）。
/// 区切り線（`separator::separator`、`role="separator"`）は `role="list"`
/// の直接の子には置かず、2 行目以降の `listitem` の内側（`starter_row`
/// の手前）へ入れ子にする。`role="list"` の直接の子を `listitem` のみに
/// 揃えることで、支援技術が一覧の項目数を `listitem` 数どおりに計算
/// できるようにする（P1 是正）。
fn starter_list(rows: &[Starter]) -> Node {
    let mut children = Vec::with_capacity(rows.len());
    for (index, starter) in rows.iter().enumerate() {
        let mut item_children = Vec::with_capacity(2);
        if index > 0 {
            item_children.push(separator::separator(
                &SeparatorProps::default(),
                vec![(RULE_ATTR, "")],
            ));
        }
        item_children.push(starter_row(starter));
        children.push(div(vec![("role", "listitem")], item_children));
    }
    div(vec![("class", LIST_CLASS), ("role", "list")], children)
}

/// 見出し + 説明 + 開始候補一覧 + 末尾の別導線リンクの 1 インスタンス分を
/// 組み立てる（モジュール doc「2 版並記と差分」節）。
fn starter_instance(
    variant: Option<&'static str>,
    title: &'static str,
    description: &'static str,
    rows: &[Starter],
    footer_text: &'static str,
    footer_href: &'static str,
    footer_label: &'static str,
) -> Node {
    let header = div(
        vec![("class", HEADER_CLASS)],
        vec![
            heading::heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Lg,
                    ..HeadingProps::default()
                },
                vec![(HEADING_ATTR, "")],
                vec![text(title)],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![(DESCRIPTION_ATTR, "")],
                vec![text(description)],
            ),
        ],
    );

    let footer = div(
        vec![("class", FOOTER_CLASS)],
        vec![
            styled_text::text(
                &TextProps {
                    size: TextSize::Sm,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![(FOOTER_TEXT_ATTR, "")],
                vec![text(footer_text)],
            ),
            link::root(
                footer_href,
                &LinkProps::default(),
                vec![(FOOTER_LINK_ATTR, "")],
                vec![text(footer_label)],
            ),
        ],
    );

    let mut attrs = vec![("class", INSTANCE_CLASS)];
    if let Some(v) = variant {
        attrs.push(("data-variant", v));
    }
    div(attrs, vec![header, starter_list(rows), footer])
}

/// `empty-state-starter-list` の Demo 本体（版 A・版 B の 2 インスタンスを
/// 並記する。モジュール doc「2 版並記と差分」節）。呼び出しごとに同一の
/// `Node` を返す純関数。
#[must_use]
pub fn demo() -> Node {
    div(
        vec![("class", STACK_CLASS)],
        vec![
            starter_instance(
                None,
                "プロジェクトを開始",
                "テンプレートを選ぶと、すぐに編集を始められます。",
                &STARTERS[..4],
                "テンプレートを使わずに始めることもできます。",
                "../../guides/",
                "空のプロジェクトから始める →",
            ),
            starter_instance(
                Some("chevron-emphasis"),
                "何から作りますか",
                "候補から選ぶと、初期設定を省略できます。",
                &STARTERS[..3],
                "他の始め方を見る",
                "../../api/",
                "API から直接組み立てる →",
            ),
        ],
    )
}
```

## 原案差分メモ

参照（主参照 R0465・集約元 R1397。出典の固有名・ファイル名は記載しません）
から取り込んだのは構造（見出し + 説明・区切り線付きの縦リスト・行末
シェブロン・末尾の別導線リンク）のみです。

- `list`（`<ul>`/`<li>`）は使用部品に含めず、素の `div` コンテナへ
  `item::root` と `separator::separator` を交互に配置しています。
  `error-page-popular-links` の `list::item` 直下配置とは異なる組み方です。
- 区切り線は `separator` 部品で実現し、行と行の間にのみ挿入します
  （行数分ではなく行数 − 1 本。末尾には置きません）。
- 版 A（代表構成、R0465）は開始候補 4 行 + 各行に説明文を添えます。
  版 B（R1397 寄り）は開始候補 3 行とし、`data-variant="chevron-emphasis"`
  で行末シェブロンの色をアクセントカラーへ強調します。
- 狭幅でも縦リストのままにするため、メディアクエリ・コンテナクエリの
  分岐を持たせず、常に `flex-direction: column` の構造で固定しています。
- アイコンは参照元を転記せず、独自の線画（stroke）SVG で描いています。
- 文言（見出し・説明文・リンクラベル）はすべて独自の架空ダミーです。
  実在の人物・企業・実クレデンシャルは一切含みません。

関連情報: [Item](../themes/item.md) / [Icon](../themes/icon.md) /
[Heading](../themes/heading.md) / [Text](../themes/text.md) /
[Separator](../themes/separator.md) / [Link](../themes/link.md)
