//! `feature-expand` block（イシュー #2549。親 #2530「Phase 7: Motion+
//! 部品化」→ #2476「Motion/Motion+ 参照アニメーション充実」配下、Motion+
//! `sections/bento-grids` に相当する合成例で、`crate::blocks` モジュール doc
//! の契約を `bento-staggered` に続いて 11 件目に実装する。両 block は同じ
//! Motion+ セクションの異なる 2 パターン（scroll-driven stagger /
//! hover-expand）を再現する）。
//!
//! # 使用部品
//!
//! `card`（各機能カード）+ `icon`（装飾アイコン）+ `button`（詳細リンク風
//! CTA）を合成する（[`BLOCK`] の `parts` に一致させる契約）。
//!
//! # hover 配線を新設しない（CSS のみで実装する）理由
//!
//! `docs/design/motion-reference-adoption-policy.md` §4 は hover を A 群
//! （CSS `:hover` で足りる大半のケース）に分類し「既存実装済みの範囲のみで
//! 新規配線を追加しない」と定める。本 block はマウス操作の hover に加え
//! キーボード操作の `:focus-within`（後述）を CSS だけで扱い、
//! `fandhe-frontend-wasm-full`/`fandhe-frontend-animation` への新規配線を
//! 一切行わない。
//!
//! # `content_height.rs`（wasm-full）を使わず `grid-template-rows: 0fr → 1fr`
//! を使う理由
//!
//! `fandhe-frontend-wasm-full` の `content_height.rs` は JS ランタイム機構
//! （実測 `scrollHeight` を CSS カスタムプロパティへ書く hydration 配線）
//! であり、docs サイトは無 JS 前提（`crates/docs-site/tests/
//! no_js_contract.rs`）でハイドレーションを一切行わないため、文字通り
//! 再利用できない。代わりに `height: auto` への遷移不能問題を JS 計測なし
//! で解く標準テクニック（`grid-template-rows: 0fr` → `1fr` の
//! `transition`）を使う。子要素へ `min-height: 0` を明示するのは、grid
//! item の既定 `min-height: auto`（コンテンツの内在サイズを下回らない）に
//! よって `0fr` の収縮が効かなくなるのを避けるため。
//!
//! # reduced-motion の個別対応が不要な理由（`bento-staggered` との対比）
//!
//! `bento-staggered` の `animation-timeline`/`animation-range` は
//! `crate::theme::Theme::to_css` の既定 reduced-motion 対応（`--fandhe-
//! motion-duration-*` トークンの 0ms 化）では止められないため個別
//! `@media (prefers-reduced-motion: reduce)` が必須だった
//! （`bento_staggered` モジュール doc 参照）。一方本 block の遷移 duration
//! は `var(--fandhe-motion-duration-normal)` トークン参照そのものであり、
//! `Theme::to_css` の既定出力が同トークンを一括 0ms 化するため追加の
//! `@media` は不要である。
//!
//! # `button` を各カードへ必ず配置する理由（キーボード到達性）
//!
//! `:hover` のみでは非マウス操作者（キーボード操作者・スクリーンリーダー
//! 利用者）が展開内容へ到達できない。各カードへ [`button::button`]
//! （`Ghost` variant、`type="button"` のまま送信先を持たない）を必ず含め、
//! `:focus-within` の対象にする（[`LAYOUT_CSS`] のセレクタ参照）。展開/
//! 非展開いずれの状態でも「extra」領域の内容は DOM 上に常在し
//! `aria-hidden`/`hidden` で隠していないため、スクリーンリーダー利用者は
//! 視覚状態に関わらず全文を読み取れる。
//!
//! # `<form>` を使わない・実データを持たない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない。機能名・説明文はすべて架空のものであり、実企業名・実サービス
//! 名・実クレデンシャル・PII を含まない。ボタンは `type="button"` のまま
//! 送信先を持たない静的な合成例であり、実際の遷移処理は利用者自身の
//! Rust コードで実装する（`docs/policy/intentional-non-adoption.md` §3.25）。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `card::root`/`button::button` は `drop_class_attr` により呼び出し側
//! `attrs` の `class` を黙って除去する契約を持つため、Demo 固有スタイルは
//! `data-blocks-feature-expand-*` 属性で渡し、[`LAYOUT_CSS`] 側も同じ属性
//! セレクタで対応する（`crate::blocks` モジュール doc「CSS フックが
//! `class` と `[data-*]` で混在する理由」節参照）。一方
//! `card::header`/`title`/`description`/`body`（variant を持たず `attrs`
//! をそのまま連結する）と素の `div` には `class` がそのまま効くため、
//! それらは従来どおりクラスセレクタを使う。

use super::{Block, BlockCategory, Part};

// blocks-code:begin
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
// blocks-code:end

/// [`super::BLOCKS`] へ登録するレジストリエントリ。
pub const BLOCK: Block = Block {
    path: "/blocks/feature-expand/",
    title: "feature-expand",
    category: BlockCategory::Feature,
    rust_source: "crates/docs-site/src/blocks/feature_expand.rs",
    demo_class: "blocks-feature-expand",
    parts: &[
        Part {
            label: "Card",
            path: "/themes/card/",
        },
        Part {
            label: "Icon",
            path: "/themes/icon/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
    ],
    demo,
};

/// `feature_expand` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS` doc
/// 「block 固有 CSS の置き場」節。他 block と同型で `pub(super)` として
/// `super::stylesheet` から連結される）。
pub(super) const LAYOUT_CSS: &str = "\
.blocks-feature-expand-grid {\n  display: grid;\n  grid-template-columns: repeat(3, 1fr);\n  gap: 1rem;\n}\n\
@media (max-width: 47.99rem) {\n  .blocks-feature-expand-grid {\n    grid-template-columns: 1fr;\n  }\n}\n\
[data-blocks-feature-expand-wrap] {\n  display: grid;\n  grid-template-rows: 0fr;\n  transition: grid-template-rows var(--fandhe-motion-duration-normal);\n  overflow: hidden;\n}\n\
[data-blocks-feature-expand-extra] {\n  min-height: 0;\n  overflow: hidden;\n  display: flex;\n  flex-direction: column;\n  gap: 0.5rem;\n  padding-top: 0.5rem;\n}\n\
[data-blocks-feature-expand-item]:hover [data-blocks-feature-expand-wrap],\n[data-blocks-feature-expand-item]:focus-within [data-blocks-feature-expand-wrap] {\n  grid-template-rows: 1fr;\n}\n";

#[cfg(test)]
mod tests {
    use super::LAYOUT_CSS;

    /// hover と focus-within の双方に同じ展開規則を適用していること
    /// （モジュール doc「`button` を各カードへ必ず配置する理由」節が言う
    /// キーボード到達性の CSS 側担保、手書き文字列のドリフト検知）。
    #[test]
    fn layout_css_expands_on_hover_and_focus_within() {
        assert!(LAYOUT_CSS.contains(":hover [data-blocks-feature-expand-wrap]"));
        assert!(LAYOUT_CSS.contains(":focus-within [data-blocks-feature-expand-wrap]"));
        assert!(LAYOUT_CSS.contains("var(--fandhe-motion-duration-normal)"));
    }
}
