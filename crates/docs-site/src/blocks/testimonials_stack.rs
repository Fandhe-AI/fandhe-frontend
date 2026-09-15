//! `testimonials-stack` block（イシュー #2548。トラッキング #2476/#2530。
//! Motion+ `sections/testimonials` の "testimonials-stack"（積層した
//! testimonial カードが前面から背面へ並び、前面カードが強調表示される
//! レイアウト）を Rust/CSS で再実装した合成例）。
//!
//! # 使用部品
//!
//! `card`（構造）/ `blockquote`（引用文・出典）/ `avatar`（イニシャル
//! fallback のみ、`image` パートは使わない）の 3 部品を合成する
//! （`crate::blocks::Block::parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//!
//! # 自動ローテーションは行わない
//!
//! docs-site は JS ハイドレーションを行わない設計（CLAUDE.md）のため、
//! 本 Demo は 3 枚のカードを積層した**静的な初期状態のみ**を描く。
//! Motion+ 側が JS で行うカードの自動入れ替えは、実装する場合は利用者
//! 自身の Rust/JS 配線に委ねる（`docs/policy/intentional-non-adoption.md`
//! §3.25 と同じ責務境界）。
//!
//! # 積層オフセットに `--fandhe-motion-stagger-index` を使う理由
//!
//! `crates/pre-styled-ui/src/recipe.rs` の `STAGGER_INDEX_VAR`
//! （`motion` feature 配下）と同名の CSS custom property を `:nth-child`
//! セレクタで各カードへ直接代入する。**`pre-styled-ui` の `motion`
//! feature はここでは有効化しない**（`dep:fandhe-animation` を有効化し
//! `structure.toml` の依存グラフ宣言を要する変更範囲拡大になるため）。
//! 代わりに `crates/pre-styled-ui/tests/stagger_index_var_drift.rs` と
//! 同型の「値のリテラル複製 + ソーステキスト突合」契約を本クレート側にも
//! 追加する（`crates/docs-site/tests/blocks_contract.rs` の
//! `testimonials_stack_stagger_var_matches_pre_styled_ui_recipe_source`）。
//!
//! # カード入れ替えの視覚表現（`presence_transition` を使わない理由）
//!
//! `SlotRecipe::presence_transition` は `pre-styled-ui` 自身の
//! `Theme::to_css` 生成パスに閉じた recipe-builder メソッドであり、
//! docs-site 側の生 CSS へ後付けできない。同じ設計思想（`transition` +
//! `data-state` 属性 + `--fandhe-motion-duration-*`/
//! `--fandhe-motion-easing-*` トークン参照）を `blocks.css` の生 CSS で
//! 再現する（`prefers-reduced-motion: reduce` 下は `Theme::to_css` が
//! これらのトークンを 0ms 化するため、個別の `@media` は不要）。
use super::{Block, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::blockquote::{self, BlockquoteVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::ColorPalette;

/// 1 件の testimonial（架空の合成データ。実企業名・実人物は使わない）。
struct Testimonial {
    quote: &'static str,
    name: &'static str,
    role: &'static str,
    initials: &'static str,
}

const TESTIMONIALS: [Testimonial; 3] = [
    Testimonial {
        quote: "\"導入から一週間で、チーム全体のレビュー待ち時間が半分になりました。\"",
        name: "Maya Chen",
        role: "Product Lead, Northbridge Labs",
        initials: "MC",
    },
    Testimonial {
        quote: "\"既定エスケープのおかげで、レビューでの指摘事項が明らかに減りました。\"",
        name: "Diego Alvarez",
        role: "Staff Engineer, Riverton Systems",
        initials: "DA",
    },
    Testimonial {
        quote: "\"単一バイナリで配布できる点が、運用チームにとても好評です。\"",
        name: "Priya Nair",
        role: "Platform Manager, Aurora Cloudworks",
        initials: "PN",
    },
];

/// testimonial 1 件分のカードを組み立てる。`index` は積層順
/// （0 が最前面）で、`data-state` の active/inactive を決める。
fn testimonial_card(index: usize, item: &Testimonial) -> Node {
    let state = if index == 0 { "active" } else { "inactive" };
    card::root(
        CardProps::default(),
        vec![
            ("data-blocks-testimonials-stack-card", ""),
            ("data-state", state),
        ],
        vec![card::body(
            vec![],
            vec![blockquote::root(
                BlockquoteVariant::default(),
                ColorPalette::default(),
                vec![],
                vec![
                    blockquote::content(vec![], vec![text(item.quote)]),
                    blockquote::caption(
                        vec![("class", "blocks-testimonials-stack-meta")],
                        vec![
                            avatar::root(
                                &AvatarProps::default(),
                                vec![("data-blocks-testimonials-stack-avatar", "")],
                                vec![avatar::fallback(
                                    ImageStatus::Error,
                                    vec![],
                                    vec![text(item.initials)],
                                )],
                            ),
                            div(
                                vec![("class", "blocks-testimonials-stack-byline")],
                                vec![
                                    div(vec![], vec![text(item.name)]),
                                    div(vec![], vec![text(item.role)]),
                                ],
                            ),
                        ],
                    ),
                ],
            )],
        )],
    )
}

/// `testimonials-stack` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数。3 枚のカードを積層表示する静的な初期状態のみを描く
/// （モジュール doc「自動ローテーションは行わない」節）。
pub fn demo() -> Node {
    let cards: Vec<Node> = TESTIMONIALS
        .iter()
        .enumerate()
        .map(|(index, item)| testimonial_card(index, item))
        .collect();

    div(
        vec![("class", "blocks-testimonials-stack")],
        vec![div(
            vec![("class", "blocks-testimonials-stack-stage")],
            cards,
        )],
    )
}
// blocks-code:end

/// [`super::BLOCKS`] へ登録するレジストリエントリ。
pub const BLOCK: Block = Block {
    path: "/blocks/testimonials-stack/",
    title: "testimonials-stack",
    rust_source: "crates/docs-site/src/blocks/testimonials_stack.rs",
    demo_class: "blocks-testimonials-stack",
    parts: &[
        Part {
            label: "Card",
            path: "/themes/card/",
        },
        Part {
            label: "Blockquote",
            path: "/themes/blockquote/",
        },
        Part {
            label: "Avatar",
            path: "/themes/avatar/",
        },
    ],
    demo,
};

/// `testimonials_stack` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節。他 block と同型で `pub(super)` として
/// `super::stylesheet` から連結される）。
///
/// # `--fandhe-motion-stagger-index` の値をリテラルで直接書く理由
///
/// モジュール doc「積層オフセットに `--fandhe-motion-stagger-index` を
/// 使う理由」節参照。`crates/docs-site/tests/blocks_contract.rs` の
/// `testimonials_stack_stagger_var_matches_pre_styled_ui_recipe_source`
/// が `crates/pre-styled-ui/src/recipe.rs` の `STAGGER_INDEX_VAR` 定義と
/// このリテラルのドリフトを fail-closed に検知する。
///
/// トランジションの duration/easing は固定 ms 値ではなく
/// `var(--fandhe-motion-duration-normal)`/`var(--fandhe-motion-easing-standard)`
/// を参照する（`blocks::stylesheet()` が `Theme::default()` を注入する
/// ため、これらのトークンは `blocks.css` の `:root` に既に定義され、
/// `prefers-reduced-motion: reduce` 下の 0ms 化にも自動的に追従する）。
pub(super) const LAYOUT_CSS: &str = "\
.blocks-testimonials-stack {\n  display: flex;\n  justify-content: center;\n}\n\
.blocks-testimonials-stack-stage {\n  display: grid;\n  width: 100%;\n  max-width: 26rem;\n  margin-inline: auto;\n  padding-block: 1.5rem 3rem;\n}\n\
[data-blocks-testimonials-stack-card] {\n  grid-area: 1 / 1;\n  transition: transform var(--fandhe-motion-duration-normal) var(--fandhe-motion-easing-standard), opacity var(--fandhe-motion-duration-normal) var(--fandhe-motion-easing-standard);\n  transform: translateY(calc(var(--fandhe-motion-stagger-index, 0) * -1rem)) scale(calc(1 - var(--fandhe-motion-stagger-index, 0) * 0.06));\n  z-index: calc(3 - var(--fandhe-motion-stagger-index, 0));\n}\n\
[data-blocks-testimonials-stack-card][data-state=\"inactive\"] {\n  opacity: 0.6;\n}\n\
.blocks-testimonials-stack-stage > [data-blocks-testimonials-stack-card]:nth-child(1) {\n  --fandhe-motion-stagger-index: 0;\n}\n\
.blocks-testimonials-stack-stage > [data-blocks-testimonials-stack-card]:nth-child(2) {\n  --fandhe-motion-stagger-index: 1;\n}\n\
.blocks-testimonials-stack-stage > [data-blocks-testimonials-stack-card]:nth-child(3) {\n  --fandhe-motion-stagger-index: 2;\n}\n\
.blocks-testimonials-stack-meta {\n  display: flex;\n  align-items: center;\n  gap: 0.75rem;\n  margin-top: 0.75rem;\n}\n\
.blocks-testimonials-stack-byline {\n  display: flex;\n  flex-direction: column;\n  font-size: 0.875rem;\n}\n";
