# feature-expand

`fandhe-frontend-pre-styled-ui` の `card` / `icon` / `button` 部品を合成
した、Motion+ `sections/bento-grids` に相当する hover-expand の合成例
です。Blocks セクションは新規部品を追加するものではなく、既存の Themes/
Primitives 部品を組み合わせた実例集であることに注意してください。

各カードは常時表示の短い説明に加え、hover または（キーボード操作の）
`:focus-within` で詳細説明とボタンが展開表示されます。展開/非展開いずれの
状態でも詳細説明は DOM 上に常在するため、スクリーンリーダー利用者は視覚
状態に関わらず全文を読み取れます。遷移は `grid-template-rows: 0fr → 1fr`
という CSS のみの標準テクニックで実装しており、`fandhe-frontend-wasm-full`
の `content_height.rs`（JS ランタイム機構）は使いません（docs サイトは
JS ハイドレーションを一切行わないため）。

## Rust コード

```rust
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps, CardVariant};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::Size;

/// 装飾用の自作幾何アイコン（lucide 等の著作物を複製しないための単純図形、
/// `bento_staggered::geo_icon` と同型の判断）。
///
/// `path` へ `fill="none"` + `stroke="currentColor"` を明示し、`icon` の
/// `<svg>` 側が固定で持つ `fill="currentColor"`（塗り面）を上書きして
/// 線画（ストローク）として描画する。`ITEMS` の一部（Live Dashboards 等）
/// の `icon_path_d` は複数の独立した開いた線分（例:
/// `"M4 20V10M10 20V4M16 20v-7M22 20V2"`）で構成され、囲まれた面積を
/// 持たないため塗り面（`fill`）のみでは何も描画されない
/// （`bento_staggered::geo_icon` の `Smart Search` アイテムが同じ理由で
/// `circle`/`path` へ個別に `stroke` を上書きしているのと同型の対処）。
fn geo_icon(path_d: &'static str) -> Node {
    icon(
        &IconProps::default(),
        vec![],
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

/// 1 枚分のカードデータ（架空の SaaS 機能名 + 常時表示の短い説明 +
/// hover/focus 時のみ見える詳細説明）。
struct FeatureItem {
    title: &'static str,
    summary: &'static str,
    detail: &'static str,
    icon_path_d: &'static str,
}

const ITEMS: [FeatureItem; 6] = [
    FeatureItem {
        title: "Instant Search",
        summary: "入力と同時に検索結果を返します。",
        detail: "インデックスをメモリ上に保持し、数万件規模のデータでも \
                  100ms 未満で検索結果を返します。表記の揺れも吸収します。",
        icon_path_d: "M12 2a10 10 0 100 20 10 10 0 000-20z",
    },
    FeatureItem {
        title: "Role-Based Access",
        summary: "ロールごとに閲覧・編集範囲を制御します。",
        detail: "組織単位・チーム単位でロールを定義し、リソースごとに \
                  閲覧・編集・削除の権限を細かく割り当てられます。",
        icon_path_d: "M12 2l8 4v6c0 5-3.5 8-8 10-4.5-2-8-5-8-10V6z",
    },
    FeatureItem {
        title: "Workflow Automation",
        summary: "定型作業をトリガーとルールで自動化します。",
        detail: "イベントの発生を検知し、条件分岐と外部連携を組み合わせた \
                  一連の処理を人手を介さず自動実行します。",
        icon_path_d: "M4 12h6l2-4 4 8 2-4h2",
    },
    FeatureItem {
        title: "Live Dashboards",
        summary: "主要指標をリアルタイムに可視化します。",
        detail: "複数データソースを 1 画面に集約し、更新のたびに \
                  グラフ・表を自動的に再描画します。",
        icon_path_d: "M4 20V10M10 20V4M16 20v-7M22 20V2",
    },
    FeatureItem {
        title: "API Webhooks",
        summary: "外部システムへイベントを即時通知します。",
        detail: "登録した URL へイベント発生時に署名付きペイロードを送信し、 \
                  再送・失敗検知の仕組みも標準で備えます。",
        icon_path_d: "M12 2v6l4 4-4 4v6M4 12h4M16 12h4",
    },
    FeatureItem {
        title: "Audit Trail",
        summary: "誰が何をいつ変更したかを記録します。",
        detail: "全ての変更操作を改ざん検知可能な形式で保存し、期間・ \
                  操作者・対象での絞り込み検索に対応します。",
        icon_path_d: "M6 2h9l5 5v15H6zM15 2v5h5",
    },
];

/// 1 枚分の機能カードを組み立てる。
fn feature_card(item: &FeatureItem) -> Node {
    card::root(
        CardProps {
            variant: CardVariant::Outline,
            size: Size::Md,
        },
        vec![("data-blocks-feature-expand-item", "")],
        vec![
            card::header(
                vec![],
                vec![
                    geo_icon(item.icon_path_d),
                    card::title(vec![], vec![text(item.title)]),
                ],
            ),
            card::body(
                vec![],
                vec![
                    card::description(vec![], vec![text(item.summary)]),
                    div(
                        vec![("data-blocks-feature-expand-wrap", "")],
                        vec![div(
                            vec![("data-blocks-feature-expand-extra", "")],
                            vec![
                                card::description(vec![], vec![text(item.detail)]),
                                button::button(
                                    &ButtonProps {
                                        variant: ButtonVariant::Ghost,
                                        ..ButtonProps::default()
                                    },
                                    vec![],
                                    vec![text("Learn more")],
                                ),
                            ],
                        )],
                    ),
                ],
            ),
        ],
    )
}

/// `feature-expand` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数。6 枚を均等グリッドで並べる。
pub fn demo() -> Node {
    let cards: Vec<Node> = ITEMS.iter().map(feature_card).collect();
    div(vec![("class", "blocks-feature-expand-grid")], cards)
}
```

## shadcn / Motion+ 側との構成上の判断

- **出典は Motion+ であり shadcn/ui ではありません**: `docs/design/
  docs-site-blocks-section.md` §3「掲載対象は 7 件で確定する」は
  `#2088`〜`#2095` のツリー限定のスコープであり、本 block は別系統
  （Motion+ 参照系、親トラッキング #2530/#2476）からの純追加です。
- **hover の新規配線は行いません**: `docs/design/
  motion-reference-adoption-policy.md` §4 が hover を A 群（CSS のみで
  足りる）に分類しているため、`:hover`/`:focus-within` はいずれも既存の
  CSS 機構のみで実装し、`fandhe-frontend-wasm-full`/
  `fandhe-frontend-animation` への新規配線を行っていません。
- **`content_height.rs` を使わない理由**: JS ランタイム機構であり、無 JS の
  docs サイトでは文字通り再利用できません。代わりに `grid-template-rows:
  0fr → 1fr` の CSS のみの標準テクニックを使用しています。
- **キーボード到達性**: 各カードへ `button::button`（`Ghost` variant）を
  必ず配置し、`:focus-within` の対象にしています。マウス操作者・キーボード
  操作者の双方が同等に展開内容へ到達できます。
- **機能名・説明は架空**: 実企業名・実サービス名・実クレデンシャル・PII を
  含みません。ボタンは送信先を持たない静的な合成例です。

関連情報: [Card](../themes/card.md) / [Icon](../themes/icon.md) /
[Button](../themes/button.md)
