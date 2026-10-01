# category-grid-captioned

`fandhe-frontend-pre-styled-ui` の `heading` / `text` / `link` /
`link-overlay` / `image` の 5 部品を合成した、カテゴリ一覧（画像の下に
名称と説明）の構成例です。Blocks セクションは新規部品を追加するもので
はなく、既存の Themes/Primitives 部品を組み合わせた実例集であることに
注意してください（主参照は対応表 ID R0824「見出しの下に説明段落」。
出典の固有名・ファイル名は記載しません）。

カードは画像を上に、下の通常フローに名称と短い説明を置き、カード全体を
1 つのリンクにしています。画像は角丸矩形か円形で、狭い幅では 1〜2 列に
折り返します。データ・文言はすべて架空のものであり、`<form>` 要素は
一切持たず、無 JS の静的表示のみを行います。

Demo は「角丸矩形・説明あり」「真円・名称のみ」の 2 インスタンスを
キャプション付きで縦に並べています。詳細は「差分メモ」節を参照して
ください。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, text as core_text, Node};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::image::{self, AspectRatio, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::link_overlay::{self, overlay};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};

/// 遷移先の固定外部 URL（モジュール doc「リンク先の方針」節参照）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// 1 件分のカテゴリデータ（架空、実在の企業・ブランドとは無関係）。
struct Category {
    name: &'static str,
    /// インスタンス A のみで使う短い説明（インスタンス B では未参照）。
    description: &'static str,
}

/// インスタンス A（角丸・説明あり・6 件）のカテゴリデータ。
const CATEGORIES_A: [Category; 6] = [
    Category {
        name: "キッチン用品",
        description: "毎日の調理を助ける道具をそろえました。",
    },
    Category {
        name: "文房具",
        description: "書く・貼る・まとめるの基本を一式で。",
    },
    Category {
        name: "アウトドア",
        description: "週末の外出に役立つ装備を厳選。",
    },
    Category {
        name: "インテリア",
        description: "部屋の雰囲気を整える小物たち。",
    },
    Category {
        name: "ファッション小物",
        description: "コーディネートの仕上げに添える一品。",
    },
    Category {
        name: "ガーデニング",
        description: "育てる楽しさを支える道具一式。",
    },
];

/// インスタンス B（真円・名称のみ・6 件）のカテゴリデータ。
const CATEGORIES_B: [Category; 6] = [
    Category {
        name: "キッチン",
        description: "",
    },
    Category {
        name: "ステーショナリー",
        description: "",
    },
    Category {
        name: "アウトドア用品",
        description: "",
    },
    Category {
        name: "ホーム",
        description: "",
    },
    Category {
        name: "アクセサリー",
        description: "",
    },
    Category {
        name: "ガーデン",
        description: "",
    },
];

/// カテゴリカード 1 枚を組み立てる。`rounded` が `true` のとき角丸矩形
/// 画像 + 名称 + 説明（インスタンス A）、`false` のとき真円画像 + 名称の
/// み（インスタンス B）になる。
fn category_card(category: &Category, rounded: bool) -> Node {
    let mut image_props = ImageProps::new(dummy_assets::PRODUCT_SRC, "");
    image_props.aspect_ratio = AspectRatio::Square;
    image_props.shape = if rounded {
        ImageShape::Rounded
    } else {
        ImageShape::Circle
    };

    let name = heading::heading(
        HeadingLevel::H4,
        &HeadingProps {
            size: HeadingSize::Md,
            ..HeadingProps::default()
        },
        vec![("data-blocks-category-grid-captioned-name", "")],
        vec![core_text(category.name)],
    );

    let mut children = vec![
        image::image(
            &image_props,
            vec![("data-blocks-category-grid-captioned-image", "")],
        ),
        name,
    ];
    if rounded {
        children.push(styled_text::text(
            &TextProps {
                size: TextSize::Sm,
                variant: TextVariant::Muted,
                ..TextProps::default()
            },
            vec![("data-blocks-category-grid-captioned-description", "")],
            vec![core_text(category.description)],
        ));
    }
    children.push(overlay(
        REPO,
        vec![
            ("aria-label", category.name),
            ("data-blocks-category-grid-captioned-overlay", ""),
        ],
        vec![],
    ));

    link_overlay::root(
        vec![("data-blocks-category-grid-captioned-item", "")],
        children,
    )
}

/// インスタンス A の導入部（左寄せ見出し・説明段落・「すべてのカテゴリを
/// 見る」リンク）。
fn intro_a() -> Node {
    div(
        vec![("class", "blocks-category-grid-captioned-intro")],
        vec![
            div(
                vec![],
                vec![
                    heading::heading(
                        HeadingLevel::H3,
                        &HeadingProps::default(),
                        vec![("data-blocks-category-grid-captioned-heading", "")],
                        vec![core_text("人気のカテゴリ")],
                    ),
                    styled_text::text(
                        &TextProps {
                            size: TextSize::Md,
                            variant: TextVariant::Muted,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![core_text(
                            "よく選ばれているカテゴリから、お探しの商品を見つけられます。",
                        )],
                    ),
                ],
            ),
            link::root(
                REPO,
                &LinkProps::default(),
                vec![],
                vec![core_text("すべてのカテゴリを見る")],
            ),
        ],
    )
}

/// インスタンス B の導入部（中央寄せ見出しのみ、説明は持たない）。
fn intro_b() -> Node {
    div(
        vec![
            ("class", "blocks-category-grid-captioned-intro"),
            ("data-align", "center"),
        ],
        vec![heading::heading(
            HeadingLevel::H3,
            &HeadingProps::default(),
            vec![("data-blocks-category-grid-captioned-heading", "")],
            vec![core_text("カテゴリから探す")],
        )],
    )
}

/// 1 インスタンス分（導入部 + グリッド）を組み立てる。`rounded` が `true`
/// のとき角丸矩形画像（インスタンス A）、`false` のとき真円画像
/// （インスタンス B）になる。
fn instance(intro: Node, categories: &[Category], rounded: bool) -> Node {
    let cards: Vec<Node> = categories
        .iter()
        .map(|category| category_card(category, rounded))
        .collect();
    let grid_class = if rounded {
        "blocks-category-grid-captioned-grid"
    } else {
        "blocks-category-grid-captioned-grid blocks-category-grid-captioned-grid-wide"
    };
    div(
        vec![("class", "blocks-category-grid-captioned-instance")],
        vec![intro, div(vec![("class", grid_class)], cards)],
    )
}

/// `category-grid-captioned` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数（モジュール doc「2 インスタンスで示す差分」節参照）。
#[must_use]
pub fn demo() -> Node {
    let note_a = styled_text::text(
        &TextProps {
            size: TextSize::Xs,
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![],
        vec![core_text(
            "左寄せの導入部・角丸矩形画像・名称と説明の構成例。",
        )],
    );
    let note_b = styled_text::text(
        &TextProps {
            size: TextSize::Xs,
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![],
        vec![core_text("中央寄せの導入部・真円画像・名称のみの構成例。")],
    );

    div(
        vec![("class", "blocks-category-grid-captioned")],
        vec![
            note_a,
            instance(intro_a(), &CATEGORIES_A, true),
            note_b,
            instance(intro_b(), &CATEGORIES_B, false),
        ],
    )
}
```

## 差分メモ

- 主参照は対応表 ID R0824（見出しの下に説明段落）です。インスタンス A
  （角丸矩形・名称と説明）が R0824 と R0822（3 列、名称と説明）の構成を
  表します。
- インスタンス B（真円・名称のみ）は R0609（円形画像 6 枚、中央見出し）
  と R0040（中央見出し、正方形 4 枚、名称のみ）を畳み込んだものです。
  R0040 の 4 枚・正方形は、枚数と画像形状の差し替えで再現できます。
- R0607（縦長画像と下段テキスト）は Demo に並記していません。
  `image::ImageProps` の `aspect_ratio` を `AspectRatio::Portrait` に
  差し替えれば縦長画像になります。
- ブレークポイントは既定 2 列、`40rem` 以上で 3 列、インスタンス B のみ
  `64rem` 以上で 6 列に広がります（テーマの `Breakpoint::Sm`/`Lg` と
  一致するリテラル値を直書きしています）。
- 状態を持たない静的な合成例のため、状態違いの並記は行っていません。

関連情報: [Heading](../themes/heading.md) / [Text](../themes/text.md) /
[Link](../themes/link.md) / [Link Overlay](../themes/link-overlay.md) /
[Image](../themes/image.md)
