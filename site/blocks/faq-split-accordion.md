# faq-split-accordion

`fandhe-frontend-pre-styled-ui` の `heading` / `text` / `accordion` /
`button` 部品を合成した、見出し左 + アコーディオン右の 2 カラム FAQ
セクションです。Blocks セクションは新規部品を追加するものではなく、既存の
Themes/Primitives 部品を組み合わせた実例集であることに注意してください
（主参照は対応表 ID R0097、差分は R0467 を集約。出典の固有名・ファイル名は
記載しません）。

左カラムには見出し・説明文・問い合わせボタンを置き、右カラムにはカテゴリ
小見出しで区切った複数グループの FAQ アコーディオンと、末尾の
「もっと見る」ボタンを添えます。lg 以上の幅（`>= 64rem`）で左右 2 カラム
になり、狭い幅では DOM の順番どおり見出しの下にアコーディオンが続きます。

本 Demo は静的な表示例であり、アコーディオンは全項目を常時展開
（open）した状態で固定し、`disabled` により開閉操作自体を無効化してい
ます（理由は「原案差分メモ」節を参照）。`<form>` 要素は一切持たず、
データの取得・送信・状態管理を行いません。問い合わせボタン・「もっと
見る」ボタンはいずれも `type="button"` のまま送信先・遷移先を持ちません。
文言はすべて独自に書いた架空のものであり、実企業名・実クレデンシャル・
PII を含みません。

## Rust コード

```rust
use fandhe_frontend_core::{div, el, span, text, Node};
use fandhe_frontend_pre_styled_ui::accordion::{
    self, item, item_content, item_indicator, item_trigger, AccordionProps, OpenState,
};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};
use fandhe_frontend_pre_styled_ui::Size;

/// カテゴリ別の架空 Q&A 一覧（実在の企業名・個人情報は含まない）。
/// カテゴリ名と (質問, 回答) の組。
const GROUPS: [(&str, &[(&str, &str)]); 2] = [
    (
        "料金・契約",
        &[
            (
                "無料プランでもすべての機能を試せますか",
                "主要な機能は無料プランでもお試しいただけます。データ保持期間・連携先の上限などは有料プランで拡張されます。",
            ),
            (
                "契約期間の縛りはありますか",
                "月単位でのご契約となり、最低利用期間の縛りはありません。いつでもプラン変更・解約が可能です。",
            ),
            (
                "支払い方法は何が使えますか",
                "主要なクレジットカードに対応しています。請求書払いをご希望の場合は個別にご相談ください。",
            ),
        ],
    ),
    (
        "アカウント・サポート",
        &[
            (
                "アカウントを複数人で共有できますか",
                "メンバーを招待して同一ワークスペースを共有できます。権限はロールごとに設定可能です。",
            ),
            (
                "サポートへの問い合わせ方法を教えてください",
                "サポート窓口のメールアドレスへご連絡ください。通常 1 営業日以内にご返信します。",
            ),
            (
                "退会後にデータは削除されますか",
                "退会申請の受理後、定められた保持期間を経てすべてのデータを削除します。",
            ),
        ],
    ),
];

/// 左カラム（見出し・説明・問い合わせボタン）。
fn intro() -> Node {
    div(
        vec![("class", "blocks-faq-split-accordion-intro")],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    ..HeadingProps::default()
                },
                vec![],
                vec![text("ご利用前に気になることをまとめました")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text("カテゴリ別によくある質問をまとめています。解決しない場合はお気軽にお問い合わせください。")],
            ),
            button::button(&ButtonProps::default(), vec![], vec![text("お問い合わせ")]),
        ],
    )
}

/// FAQ 1 件分の accordion item（トリガー + 本文）。全件を
/// [`OpenState::Open`] + `disabled: true` で固定する（モジュール doc
/// 「静的表示」節）。
fn faq_item(group: usize, index: usize, question: &str, answer: &str) -> Node {
    let state = OpenState::Open;
    let props = AccordionProps {
        disabled: true,
        ..AccordionProps::default()
    };
    let trigger_id = format!("blocks-faq-split-accordion-{group}-{index}-trigger");
    let content_id = format!("blocks-faq-split-accordion-{group}-{index}-content");

    item(
        state,
        false,
        &props,
        vec![],
        vec![
            el(
                "h5",
                vec![("class", "blocks-faq-split-accordion-trigger-heading")],
                vec![item_trigger(
                    state,
                    false,
                    &props,
                    question,
                    Some(trigger_id.as_str()),
                    Some(content_id.as_str()),
                    vec![],
                    vec![
                        span(vec![], vec![text(question)]),
                        item_indicator(state, false, &props, vec![], vec![text("▾")]),
                    ],
                )],
            ),
            item_content(
                state,
                false,
                &props,
                Some(content_id.as_str()),
                Some(trigger_id.as_str()),
                vec![],
                vec![styled_text::text(
                    &TextProps::default(),
                    vec![],
                    vec![text(answer)],
                )],
            ),
        ],
    )
}

/// カテゴリ 1 件分（小見出し + accordion）。
fn faq_group(group: usize, title: &str, items: &[(&str, &str)]) -> Node {
    let nodes: Vec<Node> = items
        .iter()
        .enumerate()
        .map(|(index, (question, answer))| faq_item(group, index, question, answer))
        .collect();

    div(
        vec![("class", "blocks-faq-split-accordion-group")],
        vec![
            heading(
                HeadingLevel::H4,
                &HeadingProps::default(),
                vec![],
                vec![text(title)],
            ),
            accordion::root(
                Size::Md,
                &AccordionProps::default(),
                vec![("data-blocks-faq-split-accordion-root", "")],
                nodes,
            ),
        ],
    )
}

/// 右カラム（カテゴリ区切りの FAQ アコーディオン一覧 + もっと見るボタン）。
fn faq_column() -> Node {
    let groups: Vec<Node> = GROUPS
        .iter()
        .enumerate()
        .map(|(group, (title, items))| faq_group(group, title, items))
        .collect();

    let mut children = groups;
    children.push(div(
        vec![("class", "blocks-faq-split-accordion-more")],
        vec![button::button(
            &ButtonProps {
                variant: ButtonVariant::Outline,
                ..ButtonProps::default()
            },
            vec![],
            vec![text("もっと見る")],
        )],
    ));

    div(vec![("class", "blocks-faq-split-accordion-faqs")], children)
}

/// `faq-split-accordion` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数（モジュール doc「静的表示」節）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-faq-split-accordion-layout")],
        vec![intro(), faq_column()],
    )
}
```

## 原案差分メモ

- 全件を open + disabled に固定した静的表示にしました。参照元（対応表 ID
  R0097）は単一グループの一覧を想定していますが、無 JS の docs サイトでは
  閉じた項目のトリガーがクリック・Enter/Space に反応しないフォーカス可能な
  `<button>` として残り、本文が事実上到達不能になります（`faq-accordion-
  centered` 等、先行する block が受けた指摘と同じ判断）。
- 主参照 R0097 は右カラムを単一グループの FAQ 一覧として構成しますが、
  集約元 R0467 は右カラムへ複数の accordion グループをカテゴリ小見出しで
  区切って並置する構成を持ちます。本 Demo は R0467 の構成を採用し、
  「料金・契約」「アカウント・サポート」の 2 グループをカテゴリ小見出しで
  区切って並べました。
- 「もっと見る」ボタンは遷移先を持たない `type="button"` のままにしまし
  た。実際の一覧遷移・ページネーションは利用者側の Rust コードで実装する
  想定です。
- 開閉インジケータはアイコンを使わず、既存部品のテキスト「▾」にしまし
  た。
- 配色・余白・角丸は既存のテーマトークンに従っています。

関連情報: [Heading](../themes/heading.md) / [Text](../themes/text.md) /
[Accordion](../themes/accordion.md) / [Button](../themes/button.md)
