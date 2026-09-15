# testimonials-stack

`fandhe-frontend-pre-styled-ui` の `card` / `blockquote` / `avatar` 部品を
合成した、Motion+（motiondivision/plus）の `sections/testimonials` にある
"testimonials-stack"（testimonial カードが積層し前面カードが強調表示される
レイアウト）に相当する合成例です。Blocks セクションは新規部品を追加する
ものではなく、既存の Themes/Primitives 部品を組み合わせた実例集であること
に注意してください。

本 Demo は静的な表示例であり、カードの自動入れ替え（Motion+ 側は JS で
ローテーションします）は行いません。docs サイトは JS ハイドレーションを
行わない設計のため、前面カード 1 枚・背面カード 2 枚を積層した初期状態
のみを固定して掲示します。実際に動的な切り替えを実装する場合は、
`data-state`（active/inactive）の書き換えを利用者自身の Rust/JS コードで
実装してください（`docs/policy/intentional-non-adoption.md` §3.25 の
責務境界: UI コンポーネント層はアプリケーションロジックを内包しません）。

## Rust コード

```rust
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
```

## Motion+ 参照上の判断

Motion+ は購入者限定素材のため、本節では取得手段・ファイル名・内部
コンポーネント識別子は記載しません（`docs/design/motion-reference-adoption-policy.md`
§9・`docs/design/shadcn-reference-adoption-policy.md` §4 と同型の転記
制限）。以下は Rust/CSS で再実装する際の判断のみを記します。

- **積層オフセットは `--fandhe-motion-stagger-index`**: `pre-styled-ui`
  が `motion` feature 配下で持つ CSS custom property 名（`stagger`
  ユーティリティ用途、#2384）を、docs-site 側では文字列リテラルとして
  直接記述しています。`pre-styled-ui` の `motion` feature（`dep:fandhe-animation`
  を有効化する）はここでは有効化していません（本 Block は静的な合成例
  であり、依存グラフを変更する範囲拡大が不要なため）。値のドリフトは
  `crates/docs-site/tests/blocks_contract.rs` の契約テストが
  `crates/pre-styled-ui/src/recipe.rs` のソーステキストと突合して
  fail-closed に検知します。
- **presence（入れ替え）は `data-state` + トークン参照 `transition` で
  表現**: `SlotRecipe::presence_transition` は `pre-styled-ui` 自身の
  `Theme::to_css` 生成パスに閉じた recipe-builder メソッドで、docs-site
  側の生 CSS へ後付けできません。代わりに各カードへ `data-state`
  （`active`/`inactive`）を付与し、`transition` の duration/easing を
  固定 ms 値ではなく `var(--fandhe-motion-duration-normal)`/
  `var(--fandhe-motion-easing-standard)` で参照する構成にしています。
- **`prefers-reduced-motion: reduce` は追加 `@media` なしで縮退**:
  `blocks::stylesheet()` は `Theme::default()` を注入するため、上記の
  duration トークンは `blocks.css` の `:root` に既に定義され、reduced
  motion 環境では `Theme::to_css` の既定出力が一括で 0ms 化します。
  `@keyframes`・scroll-driven な仕組みは使っていないため、本 Block
  固有の `@media` 追加は不要です。
- **カードは 3 枚固定・静的順**: 動的な index 書き戻し（wasm-full 側の
  `stagger_index::sync_stagger_index`、未実装）は使わず、`:nth-child(N)`
  で各カードへ `--fandhe-motion-stagger-index` を直接代入しています。

関連情報: [Card](../themes/card.md) / [Blockquote](../themes/blockquote.md) /
[Avatar](../themes/avatar.md)
