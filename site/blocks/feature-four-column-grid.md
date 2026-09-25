# feature-four-column-grid

`heading` / `text` / `card` / `icon` / `link-overlay` / `highlight` の 6 部品を
合成した、中央寄せ見出し + 導入文の下へアイコン付き feature カードを
4 枚並べるグリッドです。

3 つの静的インスタンスを縦に並べています。1 つ目は md（48rem）以上で
2 列・lg（64rem）以上で 4 列に広がる基準形、2 つ目は lg 幅になっても
2×2 のまま列数を増やさない配置、3 つ目はカード全体がリンクになる全面
リンクカードです（右上の矢印アイコンは装飾で、実際のリンク意味は
カードタイトルの `aria-label` が担います）。いずれも幅 md 未満は 1 列
です。文言・データはすべて架空のもので、データ取得・送信は行わない
静的な表示例です。`<form>` は使用しません。

## Rust コード

```rust
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::heading::{
    self, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::highlight::{highlight, HighlightProps};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::link_overlay::{self, overlay};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::Size;

/// リンク先の固定外部 URL（モジュール doc「リンク先の方針」節参照）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// カード 1 件分の架空データ（見出し・説明・自作アイコンのパス）。
struct FeatureCard {
    title: &'static str,
    body: &'static str,
    icon_path_d: &'static str,
}

/// カード 4 件（架空、実在の企業・製品とは無関係）。自作の単純な幾何パス
/// のみを使い、lucide 等の著作物は複製しない。
const CARDS: [FeatureCard; 4] = [
    FeatureCard {
        title: "決定的なビルド",
        body: "同じ入力からは常に同じ静的ファイルを生成します。",
        icon_path_d: "M12 3l9 6-9 6-9-6z",
    },
    FeatureCard {
        title: "型で表す構造",
        body: "スロットと props は Rust の型で表現されます。",
        icon_path_d: "M4 4h16v16H4z",
    },
    FeatureCard {
        title: "既定エスケープ",
        body: "テキスト補間は既定でエスケープされます。",
        icon_path_d: "M12 21a9 9 0 100-18 9 9 0 000 18z",
    },
    FeatureCard {
        title: "外部依存ゼロの描画コア",
        body: "描画コアは外部クレートに依存しません。",
        icon_path_d: "M12 2v8M8 6l4-4 4 4M4 14h16v8H4z",
    },
];

/// 装飾用の自作幾何アイコン（`fill="none"` + `stroke="currentColor"` の
/// 線画、`feature_side_heading_grid::geo_icon` と同型の判断）。
fn geo_icon(path_d: &'static str, attrs: Vec<(&'static str, &'static str)>) -> Node {
    icon(
        &IconProps {
            size: Size::Lg,
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

/// 右上向きの装飾用矢印アイコン（全面リンクカード用、`banner_floating_card
/// ::arrow_icon` と同型の自作パス）。
fn top_right_arrow_icon() -> Node {
    icon(
        &IconProps {
            size: Size::Sm,
            ..IconProps::default()
        },
        vec![("data-blocks-feature-four-column-grid-arrow", "")],
        vec![el(
            "path",
            vec![
                ("d", "M7 17L17 7M9 7h8v8"),
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

/// 中央寄せの導入部（見出しの一部を `highlight` で強調 + リード文）。
fn header(
    title_prefix: &'static str,
    highlighted: &'static str,
    title_suffix: &'static str,
) -> Node {
    div(
        vec![("class", "blocks-feature-four-column-grid-header")],
        vec![
            heading::heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl3,
                    weight: HeadingWeight::Bold,
                },
                vec![],
                vec![
                    text(title_prefix),
                    highlight(
                        &HighlightProps {
                            query: &[highlighted],
                            ..HighlightProps::default()
                        },
                        vec![],
                        highlighted,
                    ),
                    text(title_suffix),
                ],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![("data-blocks-feature-four-column-grid-lead", "")],
                vec![text("4 つの特長をアイコン付きカードで紹介する構成例です。")],
            ),
        ],
    )
}

/// カード 1 枚（アイコン → 見出し → 説明）を組み立てる。`overlay_href` が
/// `Some` のとき `link_overlay` でカード全体をクリック可能にする
/// （R0106 系インスタンス用）。
fn feature_card(data: &FeatureCard, overlay_href: Option<&'static str>) -> Node {
    let body: Vec<Node> = vec![
        geo_icon(
            data.icon_path_d,
            vec![("data-blocks-feature-four-column-grid-icon", "")],
        ),
        heading::heading(
            HeadingLevel::H4,
            &HeadingProps::default(),
            vec![],
            vec![text(data.title)],
        ),
        styled_text::text(
            &TextProps {
                size: TextSize::Sm,
                variant: TextVariant::Muted,
                ..TextProps::default()
            },
            vec![("data-blocks-feature-four-column-grid-desc", "")],
            vec![text(data.body)],
        ),
    ];

    let inner = match overlay_href {
        None => body,
        Some(href) => {
            let mut linked = body;
            linked.push(top_right_arrow_icon());
            linked.push(overlay(
                href,
                vec![
                    ("aria-label", data.title),
                    ("data-blocks-feature-four-column-grid-overlay", ""),
                ],
                vec![],
            ));
            vec![link_overlay::root(
                vec![("data-blocks-feature-four-column-grid-link", "")],
                linked,
            )]
        }
    };

    card::root(
        CardProps::default(),
        vec![("data-blocks-feature-four-column-grid-card", "")],
        vec![card::body(
            vec![("data-blocks-feature-four-column-grid-card-body", "")],
            inner,
        )],
    )
}

/// 1 インスタンス分（導入部 + カードグリッド）を組み立てる。`lg_columns_2`
/// が `true` のとき lg でも 2 列（R0945 相当）のまま、`false` なら lg で
/// 4 列（基準形/全面リンク形）になる。`overlay_href` は R0106 相当の全面
/// リンクカードにのみ渡す。
fn instance(
    header_node: Node,
    note: &'static str,
    lg_columns_2: bool,
    overlay_href: Option<&'static str>,
) -> Node {
    let mut grid_attrs = vec![("class", "blocks-feature-four-column-grid-grid")];
    if lg_columns_2 {
        grid_attrs.push(("data-columns", "2"));
    }
    let cards: Vec<Node> = CARDS
        .iter()
        .map(|c| feature_card(c, overlay_href))
        .collect();

    div(
        vec![("class", "blocks-feature-four-column-grid-instance")],
        vec![
            header_node,
            styled_text::text(
                &TextProps {
                    size: TextSize::Xs,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![("data-blocks-feature-four-column-grid-note", "")],
                vec![text(note)],
            ),
            div(grid_attrs, cards),
        ],
    )
}

/// `feature-four-column-grid` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数（他 block と同じ状態を持たない設計）。
#[must_use]
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-feature-four-column-grid-layout")],
        vec![
            instance(
                header("アイコンで示す", "4 つの特長", "（基準形）"),
                "md 2 列 → lg 4 列の基準形。",
                false,
                None,
            ),
            instance(
                header("狭い密度で示す", "4 つの特長", "（lg でも 2×2）"),
                "lg 幅になっても 2×2 のまま列数を増やさない配置例。",
                true,
                None,
            ),
            instance(
                header("カード全体で示す", "4 つの特長", "（全面リンク）"),
                "カード全体がリンクになる配置例。",
                false,
                Some(REPO),
            ),
        ],
    )
}
```

## 原案差分メモ

参照（対応表 ID R0490・R0106・R0945。出典の固有名・ファイル名は記載
しません）からの意図的な差分は次のとおりです。

- R0490 を基準形（中央寄せ見出し + アイコン付き feature カード 4 枚、
  md 2 列/lg 4 列）とし、最初のインスタンスにしました。
- R0945 の「lg でも 2×2 に留める」列数違いを 2 つ目のインスタンスに
  し、`[data-columns="2"]` 属性で lg のグリッド上書きのみを行いました
  （カードデータ自体は基準形と共通です）。
- R0106 の「4 枚とも全面リンクカード」を 3 つ目のインスタンスにし、
  `link-overlay` でカード全体をクリック可能にしました。リンク先は
  実在の外部 URL（このリポジトリ自体）を使い、`href="#"` の死リンクは
  使いません。
- 3 インスタンスとも同じ 4 件のカードデータを使い回し、差分を配置
  （列数・全面リンク化）のみで示しました。
- 見出しの一部を `highlight` で強調し、`h3`/`h4` に下げました（ページ側
  が `## Demo` として `h2` を出すため）。
- カードのアイコンは自作の単純な幾何パス（正三角形・正方形・円・矢印）
  のみを使い、lucide 等の著作物は複製していません。
