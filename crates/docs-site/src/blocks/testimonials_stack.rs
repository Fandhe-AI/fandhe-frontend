//! `testimonials-stack` block（イシュー #2548。トラッキング #2476/#2530。
//! Motion+ の testimonials 系レイアウトを参照し、Rust/CSS で再実装した
//! 合成例。積層した testimonial カードが前面から背面へ並び、前面カードが
//! 強調表示されるレイアウトを表す）。取得手段・ファイル名・内部
//! コンポーネント識別子は記載しない（購入者限定素材のライセンス上の
//! 転記制限、`docs/design/motion-reference-adoption-policy.md` §9 参照）。
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
//! `crates/pre-styled-ui/src/recipe::STAGGER_INDEX_VAR`（`motion` feature
//! 配下）と同名の CSS custom property を `:nth-child` セレクタで各カードへ
//! 直接代入する。**`pre-styled-ui` の `motion` feature は本クレートの
//! `Cargo.toml` で既に有効（`button_motion`／イシュー #2555 の実装が
//! 有効化済み。`dep:fandhe-animation` を新たに要求するのはあちら側であり
//! 本 Block が範囲を広げているわけではない）** であるため、値を
//! リテラルで複製するのではなく [`fandhe_frontend_pre_styled_ui::recipe::STAGGER_INDEX_VAR`]
//! を直接 import して `format!` でレイアウト CSS へ埋め込む（[`layout_css`]
//! 参照）。これにより値のドリフトはコンパイラが型レベルで防ぎ、
//! ソーステキスト突合による契約テストは不要になった。
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
//!
//! `data-state`（active/inactive）はカードの強調表示（`opacity`）**のみ**
//! を切り替える。積層順（`transform`/`z-index`）は本 Demo では `:nth-child`
//! が固定した `--fandhe-motion-stagger-index` の値で決まり、`data-state`
//! の書き換えに追従しない。前面カードを実際に入れ替える利用者側の実装は、
//! `data-state` に加えて各カードの `--fandhe-motion-stagger-index`
//! （[`fandhe_frontend_pre_styled_ui::recipe::stagger_index_style`] で
//! `style` 属性値を組み立てられる）も新しい積層順へ書き換える必要がある
//! （`docs/policy/intentional-non-adoption.md` §3.25 と同じ責務境界、
//! 上記「自動ローテーションは行わない」節参照）。
use super::{Block, BlockCategory, Part};
use fandhe_frontend_pre_styled_ui::recipe::STAGGER_INDEX_VAR;

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
    category: BlockCategory::Testimonial,
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

/// `testimonials_stack` 固有のレイアウト規則を組み立てる（`crate::blocks::
/// LAYOUT_CSS` doc「block 固有 CSS の置き場」節と同じ役割を担うが、他 block
/// の `pub(super) const LAYOUT_CSS: &str` とは異なり `pub(super) fn` である
/// （下記「`--fandhe-motion-stagger-index` を `format!` で埋め込む理由」
/// 節参照）。`super::stylesheet` から `&testimonials_stack::layout_css()`
/// として呼ばれ連結される。
///
/// # `--fandhe-motion-stagger-index` を `format!` で埋め込む理由
///
/// モジュール doc「積層オフセットに `--fandhe-motion-stagger-index` を
/// 使う理由」節参照。`pre-styled-ui` の `motion` feature は本クレートの
/// `Cargo.toml` で既に有効なため、[`STAGGER_INDEX_VAR`] を直接 import して
/// `format!` で埋め込む。値のリテラル複製・ソーステキスト突合による
/// ドリフト検知（旧`testimonials_stack_stagger_var_matches_pre_styled_ui_
/// recipe_source`）は不要になった（コンパイラが型レベルで一致を保証する）。
///
/// # `data-state` は積層順（`z-index`/`transform`）を変えない
///
/// 各カードの積層順は `:nth-child` が固定した [`STAGGER_INDEX_VAR`] の値
/// のみで決まる。`data-state`（active/inactive）はカードの `opacity`
/// （強調表示）だけを切り替える別軸であり、`data-state` を書き換えても
/// カードは DOM 上の位置（＝ `:nth-child` の順序）を変えない限り前面へ
/// 移動しない（モジュール doc「カード入れ替えの視覚表現」節参照）。
///
/// トランジションの duration/easing は固定 ms 値ではなく
/// `var(--fandhe-motion-duration-normal)`/`var(--fandhe-motion-easing-standard)`
/// を参照する（`blocks::stylesheet()` が `Theme::default()` を注入する
/// ため、これらのトークンは `blocks.css` の `:root` に既に定義され、
/// `prefers-reduced-motion: reduce` 下の 0ms 化にも自動的に追従する）。
///
/// `.blocks-testimonials-stack-meta`（`blockquote::caption` に付与する
/// 補助クラス）は `[data-scope="blockquote"][data-part="caption"]`
/// （詳細度 (0,2,0)、`crates/pre-styled-ui/src/recipe.rs` の base slot
/// セレクタ）と同じ要素へ適用されるため、単独クラス（詳細度 (0,1,0)）
/// では常に負ける。属性セレクタを含めて詳細度を (0,3,0) へ上げることで
/// `display: flex` を確実に適用する。
pub(super) fn layout_css() -> String {
    format!(
        ".blocks-testimonials-stack {{
  display: flex;
  justify-content: center;
}}
.blocks-testimonials-stack-stage {{
  display: grid;
  width: 100%;
  max-width: 26rem;
  margin-inline: auto;
  padding-block: 3rem 1.5rem;
}}
[data-blocks-testimonials-stack-card] {{
  grid-area: 1 / 1;
  transition: transform var(--fandhe-motion-duration-normal) var(--fandhe-motion-easing-standard), opacity var(--fandhe-motion-duration-normal) var(--fandhe-motion-easing-standard);
  transform: translateY(calc(var({stagger}, 0) * -1rem)) scale(calc(1 - var({stagger}, 0) * 0.06));
  z-index: calc(3 - var({stagger}, 0));
}}
[data-blocks-testimonials-stack-card][data-state=\"inactive\"] {{
  opacity: 0.6;
}}
.blocks-testimonials-stack-stage > [data-blocks-testimonials-stack-card]:nth-child(1) {{
  {stagger}: 0;
}}
.blocks-testimonials-stack-stage > [data-blocks-testimonials-stack-card]:nth-child(2) {{
  {stagger}: 1;
}}
.blocks-testimonials-stack-stage > [data-blocks-testimonials-stack-card]:nth-child(3) {{
  {stagger}: 2;
}}
[data-scope=\"blockquote\"][data-part=\"caption\"].blocks-testimonials-stack-meta {{
  display: flex;
  align-items: center;
  gap: 0.75rem;
  margin-top: 0.75rem;
}}
[data-blocks-testimonials-stack-avatar] {{
  flex-shrink: 0;
}}
.blocks-testimonials-stack-byline {{
  display: flex;
  flex-direction: column;
  font-size: 0.875rem;
}}
",
        stagger = STAGGER_INDEX_VAR,
    )
}
