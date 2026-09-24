//! `cursor-hover-cards` block（イシュー #2542。親 #2530「Phase 7: Motion+
//! 部品化」配下、Motion+ Cursor 由来の合成例で `cta_signup_celebrate` に
//! 続く 14 件目の実装）。
//!
//! # 使用部品
//!
//! `card`（hover 対象を持つカード 3 枚）のみを合成する（[`BLOCK`] の
//! `parts` に一致させる契約、`crates/docs-site/tests/blocks_nav.rs`/
//! `blocks_contract.rs` が検証する）。カスタムカーソル要素本体
//! （`fandhe_frontend_pre_styled_ui::cursor::cursor`）は独立部品ページを
//! 持たないため `parts` には含めない（`cta_banner_magnetic` が
//! `data-fandhe-magnetic` を `parts` へ含めないのと同型の判断）。
//!
//! # cursor 効果の実体（`fandhe-frontend-wasm-full` の `cursor` feature）
//!
//! `data-fandhe-cursor-target`（バリアント名 `"ring"`）・
//! `data-fandhe-cursor-target-label`・`data-fandhe-cursor-target-magnetic`
//! （値なし存在属性）はいずれも `crates/wasm-full/src/cursor.rs`
//! （イシュー #2542）が消費する opt-in マーカーである。実アプリで
//! `wasm-full` の `cursor` feature（既定 on）を有効にすると、カード上に
//! ポインタが乗ったときカスタムカーソルがリング形状へ変化し・ラベルを
//! 表示し・（3 枚目のみ）カード中心へ吸着する。
//!
//! # no-JS（docs サイト）での静的表示（重要な既知の制約）
//!
//! docs サイトは JS ハイドレーションを一切行わない
//! （`crate::blocks` モジュール doc 参照）。このため本 Demo では
//! `pointermove` が一切発生せず、カーソルの追従・hover バリアント変化は
//! 起きない（`fandhe_frontend_pre_styled_ui::cursor::CURSOR_CSS` の
//! `:not([data-fandhe-cursor-state])` 規則により、カーソル要素自体は
//! `display: none` のまま表示されない）。本 Demo はマークアップと opt-in
//! 属性の使い方を示す静的な実例である（`cta_banner_magnetic` と同型の
//! 既知の制約）。
//!
//! # `<form>` を使わない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `card::root` は `drop_class_attr` により呼び出し側 `attrs` の `class`
//! を黙って除去する契約を持つため、グリッドレイアウトは `div` ラッパーの
//! 素の `class` で組み、カード個別の余白調整は不要なため追加の `data-*`
//! フックは持たない（`cta_banner_magnetic` の「CSS フックが `class` と
//! `[data-*]` で混在する理由」節と異なり、本 block はカード自体の見た目を
//! 変更しないため単純である）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps, CardVariant};
use fandhe_frontend_pre_styled_ui::cursor;

/// `cursor-hover-cards` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数。
pub fn demo() -> Node {
    let plain_card = card::root(
        CardProps::from(CardVariant::Outline),
        vec![("data-fandhe-cursor-target", "ring")],
        vec![
            card::title(vec![], vec![text("Explore")]),
            card::description(vec![], vec![text("Hover to see the cursor change shape.")]),
        ],
    );
    let labeled_card = card::root(
        CardProps::from(CardVariant::Outline),
        vec![
            ("data-fandhe-cursor-target", "ring"),
            ("data-fandhe-cursor-target-label", "View"),
        ],
        vec![
            card::title(vec![], vec![text("View details")]),
            card::description(
                vec![],
                vec![text("Hover to see a label attached to the cursor.")],
            ),
        ],
    );
    let magnetic_card = card::root(
        CardProps::from(CardVariant::Outline),
        vec![
            ("data-fandhe-cursor-target", "ring"),
            ("data-fandhe-cursor-target-label", "Focus"),
            ("data-fandhe-cursor-target-magnetic", ""),
        ],
        vec![
            card::title(vec![], vec![text("Magnetic focus")]),
            card::description(
                vec![],
                vec![text("Hover to see the cursor snap to the card center.")],
            ),
        ],
    );

    div(
        vec![("data-blocks-cursor-hover-cards-grid", "")],
        vec![
            plain_card,
            labeled_card,
            magnetic_card,
            cursor::cursor(vec![]),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/cursor-hover-cards/",
    title: "cursor-hover-cards",
    category: BlockCategory::Card,
    rust_source: "crates/docs-site/src/blocks/application/card/cursor_hover_cards.rs",
    demo_class: "blocks-cursor-hover-cards",
    parts: &[Part {
        label: "Card",
        path: "/themes/card/",
    }],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `cursor_hover_cards` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節。`login_01` 等と同型で `pub(super)`
/// として `super::stylesheet` から連結される）。
const LAYOUT_CSS: &str = "\
.blocks-cursor-hover-cards {\n  padding: 3rem 1.5rem;\n}\n\
[data-blocks-cursor-hover-cards-grid] {\n  display: grid;\n  grid-template-columns: repeat(auto-fit, minmax(14rem, 1fr));\n  gap: 1rem;\n}\n";
