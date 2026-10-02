# api-reference-param-list

`fandhe-frontend-pre-styled-ui` の `heading` / `badge` / `text` / `link` /
`code` / `separator` 部品を合成した、API パラメータ一覧の縦並び表示です。
Blocks セクションは新規部品を追加するものではなく、既存の
Themes/Primitives 部品を組み合わせた実例集であることに注意してください
（主参照は対応表 ID R0194、集約元に R0195 を含みます。出典の固有名・
ファイル名は記載しません）。

見出しの下に、区切り線で仕切ったパラメータ項目を縦に並べます。各項目は
「名前・型バッジ・必須バッジ」の行と説明文で構成し、列挙型の項目は
さらに許可値をバッジで並べます。名前の横には項目へのアンカーリンクを
置き、hover とフォーカスのときだけ表示します。

本 Demo は静的な表示例であり、`<form>` 要素は一切持たず、データの取得・
送信・状態管理を行いません。パラメータの名前・型・説明・許可値はすべて
独自に書いた架空のものであり、実在する API・実企業名・実クレデンシャル・
PII を含みません。

## Rust コード

```rust
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps, BadgeVariant};
use fandhe_frontend_pre_styled_ui::code::{self, CodeProps};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::recipe::ColorPalette;
use fandhe_frontend_pre_styled_ui::separator::{separator, SeparatorProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};

/// 架空のクエリパラメータ 1 件（名前・id・href・型・必須か・説明・許可値）。
/// `id`/`href` はアンカーリンクと見出しの対応表の唯一の情報源。
struct Param {
    name: &'static str,
    id: &'static str,
    href: &'static str,
    ty: &'static str,
    required: bool,
    description: &'static str,
    allowed: &'static [&'static str],
}

/// 架空のクエリパラメータ一覧（実在サービスの API 名は含まない）。
const PARAMS: [Param; 4] = [
    Param {
        name: "project_id",
        id: "blocks-api-reference-param-list-project-id",
        href: "#blocks-api-reference-param-list-project-id",
        ty: "string",
        required: true,
        description: "対象プロジェクトの識別子。",
        allowed: &[],
    },
    Param {
        name: "status",
        id: "blocks-api-reference-param-list-status",
        href: "#blocks-api-reference-param-list-status",
        ty: "enum",
        required: true,
        description: "取得対象の状態。",
        allowed: &["active", "archived"],
    },
    Param {
        name: "sort",
        id: "blocks-api-reference-param-list-sort",
        href: "#blocks-api-reference-param-list-sort",
        ty: "enum",
        required: false,
        description: "並び順の基準。",
        allowed: &["created", "updated", "name"],
    },
    Param {
        name: "limit",
        id: "blocks-api-reference-param-list-limit",
        href: "#blocks-api-reference-param-list-limit",
        ty: "integer",
        required: false,
        description: "1 ページあたりの最大件数。",
        allowed: &[],
    },
];

/// 許可値バッジの並び（列挙型の項目のみ出力する）。
fn allowed_values_row(allowed: &[&'static str]) -> Option<Node> {
    if allowed.is_empty() {
        return None;
    }
    let mut children: Vec<Node> = vec![styled_text::text(
        &TextProps {
            variant: TextVariant::Muted,
            size: TextSize::Sm,
            ..TextProps::default()
        },
        vec![],
        vec![text("許可値:")],
    )];
    children.extend(allowed.iter().map(|value| {
        badge::badge(
            &BadgeProps {
                variant: BadgeVariant::Surface,
                ..BadgeProps::default()
            },
            vec![],
            vec![text(*value)],
        )
    }));
    Some(div(
        vec![("data-blocks-api-reference-param-list-values", "")],
        children,
    ))
}

/// パラメータ 1 件分（名前・バッジ行 + 説明 + 任意の許可値行）。
fn param_item(param: &Param) -> Node {
    // `aria-label` に `param.name` を含める。全項目で固定文言にすると
    // スクリーンリーダーのリンク一覧上でどのパラメータへのリンクか
    // 区別できないため（イシュー #3101 レビュー指摘）。
    let anchor_label = format!("{} 項目へのリンク", param.name);
    let mut meta_children: Vec<Node> = vec![
        code::code(&CodeProps::default(), vec![], vec![text(param.name)]),
        link::root(
            param.href,
            &LinkProps::default(),
            vec![
                ("aria-label", anchor_label.as_str()),
                ("data-blocks-api-reference-param-list-anchor", ""),
            ],
            vec![text("#")],
        ),
        badge::badge(
            &BadgeProps {
                variant: BadgeVariant::Outline,
                ..BadgeProps::default()
            },
            vec![],
            vec![text(param.ty)],
        ),
    ];
    if param.required {
        meta_children.push(badge::badge(
            &BadgeProps {
                variant: BadgeVariant::Subtle,
                palette: ColorPalette::Danger,
                ..BadgeProps::default()
            },
            vec![],
            vec![text("必須")],
        ));
    }

    let mut children: Vec<Node> = vec![
        heading(
            HeadingLevel::H4,
            &HeadingProps {
                size: HeadingSize::Md,
                ..HeadingProps::default()
            },
            vec![("id", param.id)],
            meta_children,
        ),
        styled_text::text(&TextProps::default(), vec![], vec![text(param.description)]),
    ];
    if let Some(values_row) = allowed_values_row(param.allowed) {
        children.push(values_row);
    }

    div(
        vec![("data-blocks-api-reference-param-list-item", "")],
        children,
    )
}

/// パラメータ一覧（項目の間にだけ `separator` を置く。末尾には置かない）。
fn param_list() -> Node {
    let mut children: Vec<Node> = Vec::new();
    for (index, param) in PARAMS.iter().enumerate() {
        if index > 0 {
            children.push(separator(&SeparatorProps::default(), vec![]));
        }
        children.push(param_item(param));
    }
    div(
        vec![("data-blocks-api-reference-param-list-list", "")],
        children,
    )
}

/// `api-reference-param-list` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。
pub fn demo() -> Node {
    div(
        vec![("data-blocks-api-reference-param-list-layout", "")],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    ..HeadingProps::default()
                },
                vec![],
                vec![text("クエリパラメータ")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text("一覧取得エンドポイントで指定できる条件です。")],
            ),
            param_list(),
        ],
    )
}
```

## 集約元との差分メモ

- R0194（主参照）の名前横アンカーリンクと型バッジを、名前・バッジの行へ
  そのまま統合しました。
- R0195 の必須表示と許可値バッジを同じ行・項目内に統合しました。必須
  バッジは必須の項目のみ、許可値バッジは列挙型の項目のみ出力します。
- 2 件の原案を個別インスタンスとして並記せず、単一の一覧へ統合しました
  （id の重複増加・検索インデックス容量増を避けるため）。
- 開閉インジケータ等の対話要素は持たず、無 JS の静的一覧のみで表現して
  います。
- パラメータの名前・型・説明・許可値はすべて独自に書いた架空のものです。

関連情報: [Heading](../themes/heading.md) / [Badge](../themes/badge.md) /
[Text](../themes/text.md) / [Link](../themes/link.md) /
[Code](../themes/code.md) / [Separator](../themes/separator.md)
