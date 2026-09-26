# section-heading-stacked

`fandhe-frontend-pre-styled-ui` の `badge` / `heading` / `text` /
`breadcrumb` の 4 部品のみを合成した、縦積みのセクション見出しです。
Blocks セクションは新規部品を追加するものではなく、既存の Themes/
Primitives 部品を組み合わせた実例集であることに注意してください（主参照は
対応表 ID R0513、集約元は対応表 ID R0193/R0186/R0994/R0995/R0524/R0997/
R0998/R0528/R0526/R0527/R0529/R0999/R1000/R0459/R0460/R0230 の 17 件で
す。出典の固有名・ファイル名は記載しません）。

先頭要素（タグライン badge / eyebrow / パンくず / なし）→大見出し →
説明文の順に縦へ積む形を、6 通り並べて示しています。

- **A. 基準形**: タグライン badge + 見出し + 説明文、中央寄せ。
- **B. 最小形**: 見出し + 説明文のみ、中央寄せ。
- **C. eyebrow 付き**: アクセント色の強調テキスト + 見出し + 説明文。
  48rem 未満は中央寄せ、48rem 以上では左寄せに切り替わります。
- **D. パンくず付き**: パンくず + 見出し + 説明文、常に左寄せ。
- **E. 暗色固定背景**: eyebrow + 見出し + 説明文、中央寄せ。badge は
  全 variant が自前の配色を持ち反転面での可読性を保証しないため置いて
  いません。
- **F. 暗色固定背景・パンくず付き**: 暗色背景とパンくずを組み合わせ、
  常に左寄せ。

CTA ボタン・背景画像・装飾ブラー・下に置く空の内容枠は持ち込みません
（CTA ボタンは `hero-editorial-stagger` が別途担当します）。ページ見出し
（h1）とセクション見出し（h2）の違いは、見た目を変えずに `heading` の
`level` 引数だけを差し替えることで表現できます（本 Demo は `## Demo` の
下に置くため見出しレベルはすべて `h3` にしています）。文言はすべて独自に
書いた架空のものです。パンくずのリンク先は実際にサイト内を指す相対パス
（Home/Blocks/現在ページ）で、`href="#"` は使いません。`<form>` 要素・
ボタン・状態機械は一切持たず、送信処理・データ取得は行いません。

## Rust コード

```rust
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{badge, BadgeProps};
use fandhe_frontend_pre_styled_ui::breadcrumb::{self, BreadcrumbVariant};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::recipe::Size;
use fandhe_frontend_pre_styled_ui::text::{
    self as styled_text, TextProps, TextSize, TextVariant, TextWeight,
};

/// Demo 内の各形の上に付ける区別ラベル（`cta_split_image::variant_label`
/// と同型）。
fn variant_label(label: &'static str) -> Node {
    styled_text::text(
        &TextProps {
            size: TextSize::Sm,
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![],
        vec![text(label)],
    )
}

/// eyebrow（アクセント色の強調テキスト、C/E で使用）。
fn eyebrow(label: &'static str) -> Node {
    styled_text::text(
        &TextProps {
            size: TextSize::Sm,
            weight: TextWeight::Semibold,
            ..TextProps::default()
        },
        vec![("data-blocks-section-heading-stacked-eyebrow", "")],
        vec![text(label)],
    )
}

/// D/F で使うパンくず（Home → Blocks → 現在ページ）。`aria_label` は
/// D/F の 2 インスタンスを区別する値を渡す。
fn breadcrumb_nav(aria_label: &'static str) -> Node {
    breadcrumb::root(
        Size::Md,
        BreadcrumbVariant::default(),
        Some(aria_label),
        vec![],
        vec![breadcrumb::list(
            vec![],
            vec![
                breadcrumb::item(
                    vec![],
                    vec![breadcrumb::link("../../", vec![], vec![text("Home")])],
                ),
                breadcrumb::separator(vec![], vec![text("/")]),
                breadcrumb::item(
                    vec![],
                    vec![breadcrumb::link("../", vec![], vec![text("Blocks")])],
                ),
                breadcrumb::separator(vec![], vec![text("/")]),
                breadcrumb::item(
                    vec![],
                    vec![breadcrumb::current_link(
                        vec![],
                        vec![text("Section Heading")],
                    )],
                ),
            ],
        )],
    )
}

/// 縦積みのセクション見出し 1 件を組み立てる（本 block の中核ヘルパ）。
///
/// - `align`: `"center"`（既定）/`"responsive"`（48rem 未満は中央、以上は
///   左寄せ）/`"start"`（常に左寄せ）。
/// - `tone`: `None`（通常面）/`Some("dark")`（暗色固定背景）。
/// - `lead`: 見出しの直前に置く先頭要素（badge/eyebrow/breadcrumb）。
///   `None` は B（最小形）用。
fn stacked(
    align: &'static str,
    tone: Option<&'static str>,
    lead: Option<Node>,
    title: &'static str,
    description: &'static str,
) -> Node {
    let mut attrs = vec![("data-blocks-section-heading-stacked-align", align)];
    if let Some(tone) = tone {
        attrs.push(("data-blocks-section-heading-stacked-tone", tone));
    }

    let mut children: Vec<Node> = vec![];
    if let Some(lead) = lead {
        children.push(lead);
    }
    children.push(heading(
        HeadingLevel::H3,
        &HeadingProps {
            size: HeadingSize::Xl2,
            weight: HeadingWeight::Bold,
        },
        vec![],
        vec![text(title)],
    ));
    children.push(styled_text::text(
        &TextProps {
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![("data-blocks-section-heading-stacked-desc", "")],
        vec![text(description)],
    ));

    div(attrs, children)
}

/// `section-heading-stacked` の Demo 本体（6 形を縦に並べる）。呼び出し
/// ごとに同一の `Node` を返す純関数。
pub fn demo() -> Node {
    let a = stacked(
        "center",
        None,
        Some(badge(
            &BadgeProps::default(),
            vec![("data-blocks-section-heading-stacked-badge", "")],
            vec![text("お知らせ")],
        )),
        "見出しの縦積みレイアウト",
        "タグライン・大見出し・説明文を中央に積む、最も基本的なセクション見出しの形です。",
    );

    let b = stacked(
        "center",
        None,
        None,
        "シンプルな見出しだけの形",
        "先頭要素を持たず、見出しと説明文だけを中央に積む最小構成です。",
    );

    let c = stacked(
        "responsive",
        None,
        Some(eyebrow("特集")),
        "eyebrow 付きの見出し",
        "48rem 未満では中央寄せ、48rem 以上では左寄せに切り替わります。",
    );

    let d = stacked(
        "start",
        None,
        Some(breadcrumb_nav("Breadcrumb example")),
        "パンくず付きの見出し",
        "現在地を示すパンくずを先頭に置き、常に左寄せで表示します。",
    );

    let e = stacked(
        "center",
        Some("dark"),
        Some(eyebrow("特集")),
        "暗色背景の見出し",
        "固定の暗色背景の上に、eyebrow・見出し・説明文を中央に積みます。",
    );

    let f = stacked(
        "start",
        Some("dark"),
        Some(breadcrumb_nav("Breadcrumb example (dark)")),
        "暗色背景・パンくず付きの見出し",
        "暗色背景とパンくずを組み合わせ、常に左寄せで表示します。",
    );

    div(
        vec![("class", "blocks-section-heading-stacked-layout")],
        vec![
            div(
                vec![],
                vec![variant_label("A. 基準形（badge・中央寄せ）"), a],
            ),
            div(vec![], vec![variant_label("B. 最小形（中央寄せ）"), b]),
            div(
                vec![],
                vec![variant_label("C. eyebrow 付き（48rem 以上で左寄せ）"), c],
            ),
            div(vec![], vec![variant_label("D. パンくず付き（左寄せ）"), d]),
            div(
                vec![],
                vec![variant_label("E. 暗色固定背景（中央寄せ）"), e],
            ),
            div(
                vec![],
                vec![variant_label("F. 暗色固定背景・パンくず付き（左寄せ）"), f],
            ),
        ],
    )
}
```

## 原案差分メモ

主参照は対応表 ID R0513（タグライン badge + 見出し + 説明文、中央寄せ）
です。集約元 17 件のうち、変種として実装したものは Demo 上の A〜F の
違いとして読み取れます（対応関係は上記「6 通り」の一覧を参照）。

以下は独立の形として実装せず、本メモでのみ扱います。

- **R0999/R1000（背景画像・装飾ブラー）**: 持ち込んでいません。テーマの
  面色（暗色面は E/F、通常面は A〜D）だけで表現しています。
- **R0459/R0460（下に置く空の内容枠）**: 本 block は見出しセクション単体
  の合成例のため、下部の空カード・プレースホルダー枠は持ち込んでいません。
- **R0230（h2 版の見出し）**: h1/h2 の切り替えは `heading` の `level`
  引数だけの差し替えで表現できるため、独立の形にはしていません（見た目
  〔`size`/`weight`〕は変えません）。
