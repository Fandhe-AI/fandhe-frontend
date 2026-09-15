# bento-staggered

`fandhe-frontend-pre-styled-ui` の `card` / `icon` 部品を合成した、
Motion+ `sections/bento-grids` に相当する scroll-driven stagger の合成例
です。Blocks セクションは新規部品を追加するものではなく、既存の Themes/
Primitives 部品を組み合わせた実例集であることに注意してください。

本 Demo は Motion+ のコードを転写したものではなく、CSS
（`animation-timeline: view()` + `animation-range`）のみで独自に再実装した
ものです。各セルは `--fandhe-motion-stagger-index` custom property を
entry range の開始点オフセットへ乗せることで「順送り」を表現します
（`animation-delay` は使いません。詳細はモジュール doc「scroll-driven
stagger の実体」節を参照してください）。対応ブラウザではスクロールに
応じて各セルがフェード＋下方向スライドインし、非対応ブラウザでは
`@supports` ブロックごと無視されるため常に通常表示のまま安全に劣化します
（JS 不要）。`prefers-reduced-motion: reduce` ではアニメーションが完全に
無効化されます。

## Rust コード

```rust
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps, CardVariant};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::recipe::stagger_index_style;
use fandhe_frontend_pre_styled_ui::Size;

/// 装飾用の自作幾何アイコン（lucide 等の著作物を複製しないための単純図形、
/// `sidebar_07::geo_icon` と同型の判断）。`children` は呼び出し側が組み立てる
/// `path`/`circle`/`rect` 等の SVG 子ノード。
fn geo_icon(children: Vec<Node>) -> Node {
    icon(&IconProps::default(), vec![], children)
}

/// 1 枚分のセルデータ（架空の SaaS 機能名 + 1 行説明 + アイコン子ノード）。
struct BentoItem {
    title: &'static str,
    description: &'static str,
    icon_children: fn() -> Vec<Node>,
}

const ITEMS: [BentoItem; 6] = [
    BentoItem {
        title: "Realtime Sync",
        description: "複数デバイス間の状態を数百ミリ秒以内に同期します。",
        icon_children: || {
            vec![el(
                "path",
                vec![("d", "M12 3v6l4-3-4-3zM12 21v-6l-4 3 4 3z")],
                vec![],
            )]
        },
    },
    BentoItem {
        title: "Smart Search",
        description: "自然文クエリからインデックス済みデータを検索します。",
        icon_children: || {
            vec![
                el(
                    "circle",
                    vec![
                        ("cx", "10"),
                        ("cy", "10"),
                        ("r", "6"),
                        ("fill", "none"),
                        ("stroke", "currentColor"),
                        ("stroke-width", "2"),
                    ],
                    vec![],
                ),
                el(
                    "path",
                    vec![
                        ("d", "M15 15l6 6"),
                        ("stroke", "currentColor"),
                        ("stroke-width", "2"),
                    ],
                    vec![],
                ),
            ]
        },
    },
    BentoItem {
        title: "Access Control",
        description: "ロールベースの権限管理で機密データを保護します。",
        icon_children: || {
            vec![el(
                "path",
                vec![("d", "M12 2l8 4v6c0 5-3.5 8-8 10-4.5-2-8-5-8-10V6z")],
                vec![],
            )]
        },
    },
    BentoItem {
        title: "Automation Rules",
        description: "トリガーと条件を組み合わせた業務フローを自動化します。",
        icon_children: || vec![el("path", vec![("d", "M4 12h6l2-4 4 8 2-4h2")], vec![])],
    },
    BentoItem {
        title: "Usage Insights",
        description: "利用状況を可視化し、傾向を素早く把握できます。",
        icon_children: || {
            vec![
                el(
                    "rect",
                    vec![("x", "4"), ("y", "12"), ("width", "3"), ("height", "8")],
                    vec![],
                ),
                el(
                    "rect",
                    vec![("x", "10"), ("y", "8"), ("width", "3"), ("height", "12")],
                    vec![],
                ),
                el(
                    "rect",
                    vec![("x", "16"), ("y", "4"), ("width", "3"), ("height", "16")],
                    vec![],
                ),
            ]
        },
    },
    BentoItem {
        title: "Global CDN",
        description: "世界各地のエッジノードから低遅延で配信します。",
        icon_children: || {
            vec![el(
                "path",
                vec![("d", "M12 2a10 10 0 100 20 10 10 0 000-20zM2 12h20M12 2c2.5 2.5 4 6.2 4 10s-1.5 7.5-4 10c-2.5-2.5-4-6.2-4-10s1.5-7.5 4-10z")],
                vec![],
            )]
        },
    },
];

/// 1 枚分の bento セル（`card`）を組み立てる。`index` は 0 始まりの
/// 表示順であり、`--fandhe-motion-stagger-index` へそのまま渡す
/// （モジュール doc「scroll-driven stagger の実体」節参照）。`is_hero`
/// は最初のセルのみ `true` で、`data-blocks-bento-staggered-hero` を付与し
/// [`LAYOUT_CSS`] 側の `grid-column`/`grid-row` span へつなげる。
fn bento_card(item: &BentoItem, index: usize, is_hero: bool) -> Node {
    let style = stagger_index_style(index);
    let mut attrs: Vec<(&str, &str)> = vec![
        ("data-blocks-bento-staggered-item", ""),
        ("style", style.as_str()),
    ];
    if is_hero {
        attrs.push(("data-blocks-bento-staggered-hero", ""));
    }
    card::root(
        CardProps {
            variant: CardVariant::Outline,
            size: Size::Md,
        },
        attrs,
        vec![
            card::header(
                vec![],
                vec![
                    div(
                        vec![("data-blocks-bento-staggered-icon-wrap", "")],
                        vec![geo_icon((item.icon_children)())],
                    ),
                    card::title(vec![], vec![text(item.title)]),
                ],
            ),
            card::body(
                vec![],
                vec![card::description(vec![], vec![text(item.description)])],
            ),
        ],
    )
}

/// `bento-staggered` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数。1 枚目（`Realtime Sync`）を hero（2x2 span）として配置し、
/// 残り 5 枚を通常サイズで並べる。
pub fn demo() -> Node {
    let cells: Vec<Node> = ITEMS
        .iter()
        .enumerate()
        .map(|(i, item)| bento_card(item, i, i == 0))
        .collect();
    div(vec![("class", "blocks-bento-staggered-grid")], cells)
}
```

## shadcn / Motion+ 側との構成上の判断

- **出典は Motion+ であり shadcn/ui ではありません**: `docs/design/
  docs-site-blocks-section.md` §3「掲載対象は 7 件で確定する」は
  `#2088`〜`#2095` のツリー限定のスコープであり、本 block は別系統
  （Motion+ 参照系、親トラッキング #2530/#2476）からの純追加です。
- **`animation-delay` ではなく `animation-range` で stagger を表現**:
  `animation-timeline: view()` 配下で `animation-delay` を時間値のまま
  併用した場合の解釈は仕様上複雑で確証が持てないため、進行度軸の
  `animation-range` の開始点オフセットのみで完結させています。
- **`fandhe-frontend-wasm-full` の `content_height.rs` は使いません**:
  docs サイトは JS ハイドレーションを一切行わないため、この JS ランタイム
  機構は文字通り再利用できません。本 block は CSS のみで完結します。
- **hover の新規配線は行いません**: `docs/design/
  motion-reference-adoption-policy.md` §4 が hover を A 群（CSS のみで
  足りる）に分類しているため、既存実装済みの範囲のみで新規配線を
  追加していません。
- **機能名・説明は架空**: 実企業名・実サービス名・実クレデンシャル・PII を
  含みません。

関連情報: [Card](../themes/card.md) / [Icon](../themes/icon.md)
