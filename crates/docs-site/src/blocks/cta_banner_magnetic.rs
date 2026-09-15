//! `cta-banner-magnetic` block（イシュー #2550。親 #2530「Phase 7: Motion+
//! 部品化」配下、Motion+ `sections/cta-sections` 由来の合成例で、
//! `crate::blocks` モジュール doc の契約を `login_01`〜`pricing_usage_slider`
//! に続いて 10 件目に実装する）。
//!
//! # 使用部品
//!
//! `button`（CTA、`data-fandhe-magnetic` opt-in 属性を付与）のみを合成する
//! （[`BLOCK`] の `parts` に一致させる契約、`crates/docs-site/tests/
//! blocks_nav.rs`/`blocks_contract.rs` が検証する）。見出し・説明文は
//! `heading`/素の `p` で構成し、`Part` には含めない（`login_04` 等の既存
//! block と同じ「合成の主役になる部品のみ `parts` へ列挙する」方針）。
//!
//! # magnetic 効果の実体（`fandhe-frontend-wasm-full` の `magnetic` feature）
//!
//! `data-fandhe-magnetic`（値なし存在属性）は
//! `crates/wasm-full/src/magnetic.rs`（イシュー #2550）が消費する opt-in
//! マーカーである。実アプリで `wasm-full` の `magnetic` feature（既定 on）
//! を有効にすると、ポインタの移動に追従してボタンが吸い付くように動く
//! （`fandhe_frontend_animation::magnetic::compute_pull`/`write_offset` が
//! `--fandhe-motion-magnetic-x`/`-y` の 2 個の CSS カスタムプロパティを
//! 書き込み、下記 [`LAYOUT_CSS`] の `transform: translate(var(..))` が
//! それを消費する設計）。
//!
//! # no-JS（docs サイト）での静的表示（重要な既知の制約）
//!
//! docs サイトは JS ハイドレーションを一切行わない
//! （`crate::blocks` モジュール doc 参照）。このため本 Demo では
//! `pointermove` が一切発生せず、実際のポインタ追従は起きない。本 Demo は
//! マークアップと opt-in 属性の使い方（`data-fandhe-magnetic` の付与位置・
//! `--fandhe-motion-magnetic-x`/`-y` を消費する CSS フックの書き方）を示す
//! 静的な実例である。`sidebar_07`/`pricing_tiers_morph` のような「複数状態
//! を静的に併記する」対処は、本 block では効果自体が離散的な状態を持たない
//! （連続的なポインタ追従）ため適用できず、単一の静的インスタンスのみを
//! 掲載する。
//!
//! # `<form>` を使わない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない。CTA ボタンは `button::button` の既定 `type="button"` のまま
//! 送信先を持たない静的な合成例である。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `button::button` は `drop_class_attr` により呼び出し側 `attrs` の
//! `class` を黙って除去する契約を持つため、CTA ボタンの CSS フックは
//! `data-blocks-cta-banner-magnetic-cta` 属性で渡し、[`LAYOUT_CSS`] 側も
//! 同じ属性セレクタで対応する（`crate::blocks` モジュール doc「CSS フックが
//! `class` と `[data-*]` で混在する理由」節参照）。`heading::heading` も
//! `drop_class_attr` を経由するため見出しの余白調整も同様に `data-*` で
//! 渡す。素の `div`/`p` には `class` がそのまま効く。

use super::{Block, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, p, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::Size;

/// `cta-banner-magnetic` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数。
pub fn demo() -> Node {
    let title = heading(
        HeadingLevel::H3,
        &HeadingProps {
            size: HeadingSize::Xl,
            weight: HeadingWeight::Bold,
        },
        vec![("data-blocks-cta-banner-magnetic-title", "")],
        vec![text("Ready to get started?")],
    );
    let description = p(
        vec![("class", "blocks-cta-banner-magnetic-description")],
        vec![text(
            "Join teams already shipping faster with a framework built for AI-era security.",
        )],
    );
    let cta = button::button(
        &ButtonProps {
            size: Size::Lg,
            ..ButtonProps::default()
        },
        vec![
            ("data-blocks-cta-banner-magnetic-cta", ""),
            ("data-fandhe-magnetic", ""),
        ],
        vec![text("Get started")],
    );

    div(
        vec![("data-blocks-cta-banner-magnetic-banner", "")],
        vec![title, description, cta],
    )
}
// blocks-code:end

/// [`super::BLOCKS`] へ登録するレジストリエントリ。
pub const BLOCK: Block = Block {
    path: "/blocks/cta-banner-magnetic/",
    title: "cta-banner-magnetic",
    rust_source: "crates/docs-site/src/blocks/cta_banner_magnetic.rs",
    demo_class: "blocks-cta-banner-magnetic",
    parts: &[Part {
        label: "Button",
        path: "/themes/button/",
    }],
    demo,
};

/// `cta_banner_magnetic` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節。`login_01` 等と同型で `pub(super)`
/// として `super::stylesheet` から連結される）。
///
/// `[data-blocks-cta-banner-magnetic-cta]` の `transform: translate(...)` は
/// `fandhe_frontend_animation::magnetic::write_offset` が書き込む 2 個の
/// CSS カスタムプロパティ（既定値 `0px`、未配線環境でも安全に無効値へ
/// フォールバックする `var(.., 0px)` 形式）を消費する。`transition` は
/// 既存の motion トークン（`--fandhe-motion-duration-fast`/
/// `--fandhe-motion-easing-standard`、`Theme::to_css` の既定出力）を使う
/// ため、`prefers-reduced-motion: reduce` 下で `Theme::to_css` が一括 0 化
/// する既存機構がそのまま効く（個別の `@media` 追加は不要、
/// `crates/frontend-animation/src/magnetic.rs` モジュール doc参照）。
pub(super) const LAYOUT_CSS: &str = "\
.blocks-cta-banner-magnetic {\n  display: flex;\n  flex-direction: column;\n  align-items: center;\n  gap: 1rem;\n  text-align: center;\n  padding: 3rem 1.5rem;\n}\n\
[data-blocks-cta-banner-magnetic-banner] {\n  display: flex;\n  flex-direction: column;\n  align-items: center;\n  gap: 1rem;\n  max-width: 32rem;\n}\n\
.blocks-cta-banner-magnetic-description {\n  color: var(--fandhe-color-fg-muted);\n}\n\
[data-blocks-cta-banner-magnetic-cta] {\n  transform: translate(var(--fandhe-motion-magnetic-x, 0px), var(--fandhe-motion-magnetic-y, 0px));\n  transition-property: transform;\n  transition-duration: var(--fandhe-motion-duration-fast);\n  transition-timing-function: var(--fandhe-motion-easing-standard);\n}\n";
