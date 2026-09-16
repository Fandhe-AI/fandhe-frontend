//! `game-ui-modal` block（イシュー #2552。トラッキング #2476/#2530/#2530
//! Phase 7 最終）。Motion+ `examples/game-ui`（公開カタログ上のゲーム風
//! UI カテゴリで唯一の実例、モーダル入場アニメーション 1 種）を参照し、
//! Rust/CSS で再実装した合成例。取得手段・ファイル名・内部コンポーネント
//! 識別子は記載しない（購入者限定素材のライセンス上の転記制限、
//! `docs/design/motion-reference-adoption-policy.md` §9 参照）。
//!
//! # 使用部品
//!
//! `dialog`（構造・入場アニメーション）/ `badge`（報酬表示）/ `button`
//! （操作）の 3 部品を合成する（`crate::blocks::Block::parts` に一致させる
//! 契約、`crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が
//! 検証する）。
//!
//! # 可否判定（イシュー #2552 の成果）
//!
//! 構成要素はすべて既存機能への写像で表現でき、DOM 計測・rAF・WAAPI を
//! 要する要素は無いため **Blocks 化**（新規部品なし）と判定した。
//! `wasm-full`/`frontend-animation`/`pre-styled-ui`/`headless-ui` は
//! 一切変更しない。
//!
//! | 構成要素 | 写像先 |
//! |---|---|
//! | 暗幕 + 中央パネルのモーダル構造 | [`dialog`] の root/backdrop/positioner/content/title/description/body/footer |
//! | scale + spring による入場 | [`motion::ZOOM_IN_KEYFRAMES_NAME`] + [`theme::SPRING_EASING_LINEAR`]（spring 近似 `linear()`、#2381） |
//! | 子要素（報酬行）の順送り出現 | [`STAGGER_INDEX_VAR`]/[`stagger_index_style`]（#2384）+ [`motion::SLIDE_FROM_BOTTOM_KEYFRAMES_NAME`] |
//! | 操作ボタン | [`button::button`]（Solid / Ghost） |
//! | 報酬・ステータス表示 | [`badge::badge`] |
//!
//! # `trigger` を置かない理由
//!
//! docs-site は JS ハイドレーションを行わない設計（CLAUDE.md）のため、
//! 本 Demo はモーダルが**既に開いた静的な初期状態**のみを描く
//! （`testimonials-stack`/`sidebar-07` と同じ設計判断）。
//! [`fandhe_frontend_pre_styled_ui::dialog::trigger`] は無 JS 下では
//! 開閉を切り替えられず表示上の意味を持たないため、意図的に置かない。
//!
//! # reduced-motion への追従
//!
//! duration はすべて `--fandhe-motion-duration-*` トークン参照であり
//! （生の `ms` 値を書かない）、`Theme::to_css` が
//! `prefers-reduced-motion: reduce` 下でこれらを 0ms 化する。keyframes は
//! [`motion::KEYFRAMES_CSS`] が同メディアクエリ内で opacity のみへ
//! 再定義済みであり、本 block は無限反復 `@keyframes`・scroll-driven な
//! アニメーションを使わないため個別の `@media` は不要。
use super::{Block, Part};
use fandhe_frontend_pre_styled_ui::motion::{
    SLIDE_FROM_BOTTOM_KEYFRAMES_NAME, ZOOM_IN_KEYFRAMES_NAME,
};
use fandhe_frontend_pre_styled_ui::recipe::STAGGER_INDEX_VAR;
use fandhe_frontend_pre_styled_ui::theme::SPRING_EASING_LINEAR;

// blocks-code:begin
use fandhe_frontend_core::{li, text, ul, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps, BadgeVariant};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::dialog::{self, ContentIds, DialogRole, OpenState};
use fandhe_frontend_pre_styled_ui::recipe::stagger_index_style;
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// 架空の報酬データ（実企業名・実クレデンシャルは使わない）。
const REWARDS: [(&str, ColorPalette); 3] = [
    ("+320 XP", ColorPalette::Success),
    ("レア装備 x1", ColorPalette::Info),
    ("称号「開拓者」", ColorPalette::Accent),
];

/// `game-ui-modal` の Demo 本体（既に開いた静的な初期状態のみ描く）。
pub fn demo() -> Node {
    let title_id = "blocks-game-ui-modal-title";
    let description_id = "blocks-game-ui-modal-description";

    let rewards: Vec<Node> = REWARDS
        .iter()
        .enumerate()
        .map(|(index, (label, palette))| {
            let style = stagger_index_style(index);
            li(
                vec![("data-blocks-game-ui-modal-reward", ""), ("style", &style)],
                vec![badge::badge(
                    &BadgeProps {
                        variant: BadgeVariant::Solid,
                        size: Size::Md,
                        palette: *palette,
                    },
                    vec![],
                    vec![text(*label)],
                )],
            )
        })
        .collect();

    dialog::root(
        Size::Md,
        OpenState::Open,
        vec![("data-blocks-game-ui-modal-root", "")],
        vec![
            dialog::backdrop(OpenState::Open, vec![], vec![]),
            dialog::positioner(
                OpenState::Open,
                vec![],
                vec![dialog::content(
                    OpenState::Open,
                    DialogRole::Dialog,
                    true,
                    ContentIds {
                        id: Some("blocks-game-ui-modal-content"),
                        labelledby: Some(title_id),
                        describedby: Some(description_id),
                    },
                    vec![("data-blocks-game-ui-modal-content", "")],
                    vec![
                        dialog::title(Some(title_id), vec![], vec![text("Quest Complete")]),
                        dialog::description(
                            Some(description_id),
                            vec![],
                            vec![text("討伐クエスト「北の遺跡」を制覇しました。")],
                        ),
                        dialog::body(vec![], vec![ul(vec![], rewards)]),
                        dialog::footer(
                            vec![],
                            vec![
                                button::button(
                                    &ButtonProps {
                                        variant: ButtonVariant::Ghost,
                                        ..ButtonProps::default()
                                    },
                                    vec![],
                                    vec![text("Later")],
                                ),
                                button::button(
                                    &ButtonProps::default(),
                                    vec![],
                                    vec![text("Claim rewards")],
                                ),
                            ],
                        ),
                    ],
                )],
            ),
        ],
    )
}
// blocks-code:end

/// [`super::BLOCKS`] へ登録するレジストリエントリ。
pub const BLOCK: Block = Block {
    path: "/blocks/game-ui-modal/",
    title: "game-ui-modal",
    rust_source: "crates/docs-site/src/blocks/game_ui_modal.rs",
    demo_class: "blocks-game-ui-modal",
    parts: &[
        Part {
            label: "Dialog",
            path: "/themes/dialog/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Badge",
            path: "/themes/badge/",
        },
    ],
    demo,
};

/// `game_ui_modal` 固有のレイアウト規則を組み立てる（`crate::blocks::
/// LAYOUT_CSS` doc「block 固有 CSS の置き場」節と同じ役割）。`super::stylesheet`
/// から `&game_ui_modal::layout_css()` として呼ばれ連結される。
///
/// # デモ枠内での掲示（`showcase.rs` の中和と同型、`bento_staggered` 先例）
///
/// `dialog::positioner`/`backdrop` は本来 `position: fixed; inset: 0` の
/// ビューポート全体オーバーレイだが、Blocks の掲示は `.blocks-demo` 枠内へ
/// 収める必要がある。本 block スコープ（`.blocks-game-ui-modal` 配下）に
/// 限定した属性セレクタで `position: relative`・`inset: auto` へ差し替え、
/// ゲーム画面風の暗色グラデーション背景を持つ枠として positioner を
/// 前面に重ねる。`.blocks-demo.blocks-game-ui-modal` は scale の overshoot
/// が枠でクリップされないよう `overflow: visible` にする
/// （`bento_staggered` 先例と同じ理由）。`[data-blocks-game-ui-modal-root]`
/// へ `position: relative` を設定し、`position: absolute; inset: 0` の
/// positioner の包含ブロックをこのデモ枠自身にする（root 直下の兄弟である
/// backdrop の `position: relative` は positioner の基準にならないため
/// 別途必要、PR #2587 レビュー指摘）。`dialog::title` の `h2` は
/// `.docs-content` のタイポグラフィ（`border-top`/`padding-top`/
/// `letter-spacing`）を継承してしまうため、`showcase.rs` の
/// pre-styled-showcase と同型のリセットを併せて適用する。
///
/// # 入場アニメーション
///
/// content には [`ZOOM_IN_KEYFRAMES_NAME`] + [`SPRING_EASING_LINEAR`]
/// （spring 近似）を適用する。既存の `presence_transition`（`content` の
/// `transition` + `@starting-style`、#2387）と同方向（opacity/scale の
/// フェードイン）であり、本 block の `@keyframes` 入場と併走しても
/// 破綻しない。
///
/// 報酬行は [`SLIDE_FROM_BOTTOM_KEYFRAMES_NAME`] を [`STAGGER_INDEX_VAR`]
/// による段階遅延（`stagger_delay_declaration` と同型の `calc()` 式を
/// 直書き、`testimonials_stack::layout_css` 先例）で順送りに表示する。
pub(super) fn layout_css() -> String {
    format!(
        ".blocks-game-ui-modal.blocks-demo {{
  overflow: visible;
}}
[data-blocks-game-ui-modal-root] {{
  position: relative;
}}
.blocks-game-ui-modal [data-scope=\"dialog\"] h2 {{
  border-top: none;
  padding-top: 0;
  letter-spacing: normal;
}}
.blocks-game-ui-modal [data-scope=\"dialog\"][data-part=\"backdrop\"] {{
  position: relative;
  inset: auto;
  z-index: auto;
  width: 100%;
  min-height: 20rem;
  border-radius: 0.5rem;
  background: linear-gradient(160deg, #1b1033 0%, #05070f 100%);
}}
.blocks-game-ui-modal [data-scope=\"dialog\"][data-part=\"positioner\"] {{
  position: absolute;
  inset: 0;
  z-index: auto;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 1.5rem;
}}
.blocks-game-ui-modal [data-scope=\"dialog\"][data-part=\"content\"] {{
  position: relative;
  animation: {zoom_in} var(--fandhe-motion-duration-slower) {spring} both;
  border: 2px solid var(--fandhe-color-border-accent, var(--fandhe-color-border));
  box-shadow: var(--fandhe-shadow-lg);
}}
[data-blocks-game-ui-modal-reward] {{
  list-style: none;
  animation: {slide_up} var(--fandhe-motion-duration-slow) var(--fandhe-motion-easing-emphasized) both;
  animation-delay: calc(var({stagger}, 0) * var(--fandhe-motion-duration-normal));
}}
",
        zoom_in = ZOOM_IN_KEYFRAMES_NAME,
        spring = SPRING_EASING_LINEAR,
        slide_up = SLIDE_FROM_BOTTOM_KEYFRAMES_NAME,
        stagger = STAGGER_INDEX_VAR,
    )
}
