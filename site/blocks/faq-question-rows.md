# faq-question-rows

`fandhe-frontend-pre-styled-ui` の `heading` / `text` / `separator` /
`link` 部品を合成した、質問左・回答右の行型 FAQ セクションです。Blocks
セクションは新規部品を追加するものではなく、既存の Themes/Primitives
部品を組み合わせた実例集であることに注意してください（主参照は対応表
ID R0926、差分は R0469 を集約。出典の固有名・ファイル名は記載しません）。

見出しの下に Q&A を 1 行ずつ罫線で区切って縦に並べます。各行の中は、
質問を左 5/12・回答を右 7/12 に配置し、狭い幅では自然に縦積みへ切り替
わります。回答文中には実利用時に差し替えるべきリンクを含めることがで
きます（サンプルのリンク先はこのサイト内の実在するページです）。

本 Demo は静的な表示例であり、データの取得・送信・状態管理を行いませ
ん。`<form>` 要素は一切持ちません。文言はすべて独自に書いた架空のもの
であり、実企業名・実クレデンシャル・PII を含みません。

## Rust コード

```rust
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps, LinkVariant};
use fandhe_frontend_pre_styled_ui::separator::{self, SeparatorProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};

/// 回答内リンク（href・ラベル）。`None` の行はリンクを持たない。
type AnswerLink<'a> = Option<(&'a str, &'a str)>;

/// 架空の Q&A 一覧（実在の企業名・個人情報は含まない）。質問 / 回答前半 /
/// 任意のリンク / 回答後半、の 4 要素タプル。
const FAQS: [(&str, &str, AnswerLink<'static>, &str); 5] = [
    (
        "無料プランでもすべての機能を試せますか",
        "主要な機能は無料プランでもお試しいただけます。詳しい制限事項は",
        Some(("../../guides/", "導入ガイド")),
        "をご覧ください。",
    ),
    (
        "契約期間の縛りはありますか",
        "月単位でのご契約となり、最低利用期間の縛りはありません。いつでもプラン変更・解約が可能です。",
        None,
        "",
    ),
    (
        "他のツールからデータを移行できますか",
        "主要なフォーマットでのデータ書き出し・取り込みに対応しています。移行手順は",
        Some(("../../guides/", "導入ガイド")),
        "の該当ページをご参照ください。",
    ),
    (
        "API から利用できますか",
        "はい。対応エンドポイントの一覧は",
        Some(("../../api/", "API Reference")),
        "にまとめています。",
    ),
    (
        "支払い方法は何が使えますか",
        "主要なクレジットカードに対応しています。請求書払いをご希望の場合は個別にご相談ください。",
        None,
        "",
    ),
];

/// 見出しエリア（見出し + 補足文）。
fn header() -> Node {
    div(
        vec![("class", "blocks-faq-question-rows-header")],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    ..HeadingProps::default()
                },
                vec![],
                vec![text("よくいただくご質問")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text("導入前によく寄せられるご質問をまとめました。")],
            ),
        ],
    )
}

/// 回答段落の children を組み立てる（前半 + 任意のインラインリンク +
/// 後半）。
fn answer_children(lead: &str, link_target: AnswerLink<'_>, tail: &str) -> Vec<Node> {
    let mut children = vec![text(lead)];
    if let Some((href, label)) = link_target {
        children.push(link::root(
            href,
            &LinkProps {
                variant: LinkVariant::Underline,
                ..LinkProps::default()
            },
            vec![],
            vec![text(label)],
        ));
    }
    if !tail.is_empty() {
        children.push(text(tail));
    }
    children
}

/// FAQ 1 行分（上罫線 + 質問 + 回答の 2 列行）。
fn faq_row(question: &str, lead: &str, link_target: AnswerLink<'_>, tail: &str) -> Vec<Node> {
    vec![
        separator::separator(
            &SeparatorProps::default(),
            vec![("data-blocks-faq-question-rows-rule", "")],
        ),
        div(
            vec![("class", "blocks-faq-question-rows-row")],
            vec![
                heading(
                    HeadingLevel::H4,
                    &HeadingProps::default(),
                    vec![("data-blocks-faq-question-rows-question", "")],
                    vec![text(question)],
                ),
                div(
                    vec![("class", "blocks-faq-question-rows-answer")],
                    vec![styled_text::text(
                        &TextProps::default(),
                        vec![],
                        answer_children(lead, link_target, tail),
                    )],
                ),
            ],
        ),
    ]
}

/// Q&A 一覧本体（各行の上に罫線を引く、R0469 方式）。
fn faq_list() -> Node {
    let items: Vec<Node> = FAQS
        .iter()
        .flat_map(|(question, lead, link_target, tail)| faq_row(question, lead, *link_target, tail))
        .collect();

    div(vec![("class", "blocks-faq-question-rows-list")], items)
}

/// `faq-question-rows` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-faq-question-rows-layout")],
        vec![header(), faq_list()],
    )
}
```

## 原案差分メモ

- 罫線は各行の上に 1 本引く方式（対応表 ID R0469）を採用しました。主参照
  （R0926）との違いは最初の行の上の罫線の有無だけで、並べて見せるほどの
  構造差ではないため 1 つの Demo に統合しています。
- 回答内のインラインリンクは R0469 由来の特徴です。5 行中 3 行にリンク
  を入れ、リンクなしの行も残すことで両方の書き方ができることを示してい
  ます。
- `<dl>`/`<dt>`/`<dd>` は使いませんでした。`<hr>`（罫線）を `dl` の直接
  の子にも `dt`/`dd` をまとめた `div` グループの中にも置けないこと、
  `dt` の中に見出し要素を置けないことが理由です。行は素の `div` にし、
  質問は `<h4>` 見出しにすることで、見出しナビゲーションから質問へ直接
  飛べるようにしています。
- 文言は独自に書いた架空のものです。配色・余白は既存のテーマトークンに
  従っています。アイコンは使っていません。

関連情報: [Heading](../themes/heading.md) / [Text](../themes/text.md) /
[Separator](../themes/separator.md) / [Link](../themes/link.md)
