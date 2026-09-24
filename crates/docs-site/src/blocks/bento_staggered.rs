//! `bento-staggered` block（イシュー #2549。親 #2530「Phase 7: Motion+
//! 部品化」→ #2476「Motion/Motion+ 参照アニメーション充実」配下、Motion+
//! `sections/bento-grids` に相当する合成例で、`crate::blocks` モジュール doc
//! の契約を `pricing-usage-slider` に続いて 10 件目に実装する）。
//!
//! # 出典に関する注記（`docs/design/docs-site-blocks-section.md` §3 との関係）
//!
//! `docs-site-blocks-section.md` §3「掲載対象は 7 件で確定する」は
//! `#2088`〜`#2095` のツリー限定のスコープ記録であり、本 block は別系統
//! （Motion+ 参照系、#2530/#2476）からの純追加である（同文書の該当段落
//! 参照）。shadcn/ui ではなく Motion+ が出典であり、コードの転写ではなく
//! Rust/CSS での独自再実装である
//! （`docs/design/motion-reference-adoption-policy.md` §9 準拠）。
//!
//! # 使用部品
//!
//! `card`（各 bento セル）+ `icon`（装飾アイコン）を合成する（[`BLOCK`] の
//! `parts` に一致させる契約）。
//!
//! # scroll-driven stagger の実体（`animation-delay` ではなく `animation-range`）
//!
//! 各セルは `animation-timeline: view()`（[`crate::blocks`] モジュール doc
//! が参照する `SlotRecipe::write_scroll_reveal_blocks` と同型の
//! プログレッシブエンハンスメント）でビューポート進入時にフェード＋
//! 下方向スライドインする。`stagger`（順送り遅延）は
//! `fandhe_frontend_pre_styled_ui::recipe::stagger_index_style` が書き出す
//! `--fandhe-motion-stagger-index` を使うが、**`animation-delay`（時間軸）
//! ではなく `animation-range` の開始点オフセット（進行度軸）で表現する**。
//! `animation-timeline: view()` 配下で `animation-delay` を時間値のまま
//! 併用した場合の解釈は仕様上複雑で確証が持てないため、進行度軸のみで
//! 完結させる意図的な設計判断である（`--fandhe-motion-stagger-index` の
//! 値を entry range の開始点（`entry` 到達後どこまで進行してから発火するか）
//! へ直接乗せることで、後続セルほど発火が遅れる「順送り」を実現する）。
//!
//! # `.blocks-demo` の `overflow-x: auto` を打ち消す理由
//!
//! `crate::blocks::LAYOUT_CSS` の `.blocks-demo` は `overflow-x: auto` を
//! 宣言する。CSS Overflow 仕様上 `overflow-y` を明示しない場合、
//! `overflow-y` は `overflow-x` と同じ値へ強制される
//! （[CSS Overflow §2](https://www.w3.org/TR/css-overflow-3/#overflow-properties)）。
//! この結果 `.blocks-demo` 自身がスクロールコンテナ（`view()` タイムライン
//! の基準になり得る要素）になってしまい、実際にはスクロールしない小さな
//! デモ枠内では `animation-timeline: view()` が意図通り機能しない
//! （祖先スクロールコンテナが `.blocks-demo` 自身になり、要素はその枠内で
//! 最初から「entry 済み」になり得るため）。[`LAYOUT_CSS`] は本 block 限定
//! で明示的に `overflow: visible` へ打ち消し、ページ本体のビューポートを
//! 基準にする（`view()` の既定挙動）。
//!
//! # `content_height.rs`（wasm-full）を使わない理由
//!
//! `fandhe-frontend-wasm-full` の `content_height.rs` は JS ランタイム機構
//! であり、docs サイトは無 JS 前提（`crates/docs-site/tests/
//! no_js_contract.rs`）でハイドレーションを一切行わないため、文字通り
//! 再利用できない。本 block は `wasm-full`/`fandhe-frontend-animation` への
//! 変更を一切行わず、CSS のみで完結する。
//!
//! # hover 配線を新設しない理由
//!
//! `docs/design/motion-reference-adoption-policy.md` §4 は hover を A 群
//! （CSS `:hover` で足りる大半のケース）に分類し「既存実装済みの範囲のみで
//! 新規配線を追加しない」と定める。本 block は hover の新規配線を持たず、
//! scroll-driven reveal のみを実装する。
//!
//! # `<form>` を使わない・実データを持たない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない。機能名・説明文はすべて架空のものであり、実企業名・実サービス
//! 名・実クレデンシャル・PII を含まない。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `card::root` は `drop_class_attr` により呼び出し側 `attrs` の `class` を
//! 黙って除去する契約を持つが、`class` 以外の属性（`data-*`・`style`）は
//! そのまま連結される（`crate::class_attr::drop_class_attr` 参照）。この
//! ため本 block は `card::root` へラッパー `div` を追加せず、`data-*`/
//! `style` 属性を `card::root` の `attrs` に直接渡す（`crate::blocks`
//! モジュール doc「CSS フックが `class` と `[data-*]` で混在する理由」節と
//! 同じ判断軸。他 block と異なりラッパーが不要な点が差分）。

use super::{Block, BlockCategory, Part};

// blocks-code:begin
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
// blocks-code:end

/// [`super::BLOCKS`] へ登録するレジストリエントリ。
pub const BLOCK: Block = Block {
    path: "/blocks/bento-staggered/",
    title: "bento-staggered",
    category: BlockCategory::Bento,
    rust_source: "crates/docs-site/src/blocks/bento_staggered.rs",
    demo_class: "blocks-bento-staggered",
    parts: &[
        Part {
            label: "Card",
            path: "/themes/card/",
        },
        Part {
            label: "Icon",
            path: "/themes/icon/",
        },
    ],
    demo,
};

/// `bento_staggered` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS` doc
/// 「block 固有 CSS の置き場」節。他 block と同型で `pub(super)` として
/// `super::stylesheet` から連結される）。
///
/// ドリフト防止（モジュール doc「scroll-driven stagger の実体」節が参照する
/// 定数とのリテラル一致）は本ファイル末尾の `#[cfg(test)]` が検証する。
pub(super) const LAYOUT_CSS: &str = "\
.blocks-demo.blocks-bento-staggered {\n  overflow: visible;\n}\n\
.blocks-bento-staggered-grid {\n  display: grid;\n  grid-template-columns: repeat(4, 1fr);\n  grid-auto-rows: 1fr;\n  gap: 1rem;\n}\n\
[data-blocks-bento-staggered-hero] {\n  grid-column: span 2;\n  grid-row: span 2;\n}\n\
[data-blocks-bento-staggered-icon-wrap] {\n  display: inline-flex;\n  align-items: center;\n  justify-content: center;\n  width: 2.5rem;\n  height: 2.5rem;\n  border-radius: var(--fandhe-radius-md, 0.375rem);\n  background: var(--fandhe-color-bg-subtle);\n  margin-bottom: 0.5rem;\n}\n\
@media (max-width: 39.99rem) {\n  .blocks-bento-staggered-grid {\n    grid-template-columns: repeat(2, 1fr);\n  }\n  [data-blocks-bento-staggered-hero] {\n    grid-column: span 2;\n    grid-row: span 1;\n  }\n}\n\
@supports (animation-timeline: view()) {\n  [data-blocks-bento-staggered-item] {\n    animation-name: fd-motion-slide-from-bottom;\n    animation-timing-function: linear;\n    animation-fill-mode: backwards;\n    animation-timeline: view();\n    animation-range: entry calc(10% + var(--fandhe-motion-stagger-index, 0) * 8%) entry 100%;\n  }\n}\n\
@media (prefers-reduced-motion: reduce) {\n  [data-blocks-bento-staggered-item] {\n    animation: none;\n  }\n}\n";

#[cfg(test)]
mod tests {
    use super::LAYOUT_CSS;
    use fandhe_frontend_pre_styled_ui::motion::SLIDE_FROM_BOTTOM_KEYFRAMES_NAME;
    use fandhe_frontend_pre_styled_ui::recipe::STAGGER_INDEX_VAR;

    /// [`LAYOUT_CSS`] が参照する `@keyframes` 名・stagger var 名が、
    /// `motion`/`recipe` モジュール側の定数と実際に一致していること
    /// （モジュール doc「scroll-driven stagger の実体」節の設計が
    /// 手書き文字列のドリフトで崩れないことを固定する）。
    #[test]
    fn layout_css_references_the_shared_keyframes_name_and_stagger_var() {
        assert!(LAYOUT_CSS.contains(SLIDE_FROM_BOTTOM_KEYFRAMES_NAME));
        assert!(LAYOUT_CSS.contains(STAGGER_INDEX_VAR));
    }
}
