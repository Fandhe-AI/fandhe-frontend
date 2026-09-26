# faq-accordion-centered

`fandhe-frontend-pre-styled-ui` の `heading` / `text` / `badge` /
`accordion` / `button` 部品を合成した、中央寄せの FAQ セクションです。
Blocks セクションは新規部品を追加するものではなく、既存の
Themes/Primitives 部品を組み合わせた実例集であることに注意してください
（主参照は対応表 ID R0095、差分は R0466 / R0925 を集約。出典の固有名・
ファイル名は記載しません）。

アイブロウ badge・見出し・説明文からなる中央寄せの見出しエリアの下に、
幅を絞った FAQ アコーディオンを置き、末尾に「まだ質問がありますか」の
問い合わせ導線を添えます。狭い幅ではアコーディオンが自然に幅いっぱいへ
広がります。

本 Demo は静的な表示例であり、アコーディオンは全項目を常時展開
（open）した状態で固定し、`disabled` により開閉操作自体を無効化してい
ます（理由は「原案差分メモ」節を参照）。`<form>` 要素は一切持たず、
データの取得・送信・状態管理を行いません。問い合わせボタンは
`type="button"` のまま送信先を持ちません。文言はすべて独自に書いた架空
のものであり、実企業名・実クレデンシャル・PII を含みません。

## Rust コード

```rust
use fandhe_frontend_core::{div, el, span, text, Node};
use fandhe_frontend_pre_styled_ui::accordion::{
    self, item, item_content, item_indicator, item_trigger, AccordionProps, OpenState,
};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps, BadgeVariant};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};
use fandhe_frontend_pre_styled_ui::Size;

/// 架空の Q&A 一覧（実在の企業名・個人情報は含まない）。
const FAQS: [(&str, &str); 5] = [
    (
        "無料プランでもすべての機能を試せますか",
        "主要な機能は無料プランでもお試しいただけます。データ保持期間・連携先の上限などは有料プランで拡張されます。",
    ),
    (
        "契約期間の縛りはありますか",
        "月単位でのご契約となり、最低利用期間の縛りはありません。いつでもプラン変更・解約が可能です。",
    ),
    (
        "他のツールからデータを移行できますか",
        "主要なフォーマットでのデータ書き出し・取り込みに対応しています。移行手順は導入ガイドをご参照ください。",
    ),
    (
        "サポートへの問い合わせ方法を教えてください",
        "サポート窓口のメールアドレスへご連絡ください。通常 1 営業日以内にご返信します。",
    ),
    (
        "支払い方法は何が使えますか",
        "主要なクレジットカードに対応しています。請求書払いをご希望の場合は個別にご相談ください。",
    ),
];

/// 見出しエリア（アイブロウ badge + 見出し + 説明文）。
fn header() -> Node {
    div(
        vec![("class", "blocks-faq-accordion-centered-header")],
        vec![
            badge::badge(
                &BadgeProps {
                    variant: BadgeVariant::Subtle,
                    ..BadgeProps::default()
                },
                vec![],
                vec![text("よくある質問")],
            ),
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
                vec![text("解決しない場合はお気軽にお問い合わせください。")],
            ),
        ],
    )
}

/// FAQ 1 件分の accordion item（トリガー + 本文）。全件を
/// [`OpenState::Open`] + `disabled: true` で固定する（モジュール doc
/// 「静的表示」節）。
fn faq_item(index: usize, question: &str, answer: &str) -> Node {
    let state = OpenState::Open;
    let props = AccordionProps {
        disabled: true,
        ..AccordionProps::default()
    };
    let trigger_id = format!("blocks-faq-accordion-centered-{index}-trigger");
    let content_id = format!("blocks-faq-accordion-centered-{index}-content");

    item(
        state,
        false,
        &props,
        vec![],
        vec![
            el(
                "h4",
                vec![("class", "blocks-faq-accordion-centered-trigger-heading")],
                vec![item_trigger(
                    state,
                    false,
                    &props,
                    question,
                    Some(trigger_id.as_str()),
                    Some(content_id.as_str()),
                    vec![],
                    vec![
                        span(
                            vec![("class", "blocks-faq-accordion-centered-trigger-label")],
                            vec![text(question)],
                        ),
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

/// 幅を絞った FAQ アコーディオン本体。
fn faq_list() -> Node {
    let items: Vec<Node> = FAQS
        .iter()
        .enumerate()
        .map(|(index, (question, answer))| faq_item(index, question, answer))
        .collect();

    div(
        vec![("class", "blocks-faq-accordion-centered-list")],
        vec![accordion::root(
            Size::Md,
            &AccordionProps::default(),
            vec![("data-blocks-faq-accordion-centered-root", "")],
            items,
        )],
    )
}

/// 問い合わせ導線（小見出し + 説明文 + ボタン）。
fn contact() -> Node {
    div(
        vec![("class", "blocks-faq-accordion-centered-contact")],
        vec![
            heading(
                HeadingLevel::H4,
                &HeadingProps::default(),
                vec![],
                vec![text("まだ質問がありますか")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text("チームが直接お答えします。お気軽にご連絡ください。")],
            ),
            button::button(&ButtonProps::default(), vec![], vec![text("お問い合わせ")]),
        ],
    )
}

/// `faq-accordion-centered` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数（モジュール doc「静的表示」節）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-faq-accordion-centered-layout")],
        vec![header(), faq_list(), contact()],
    )
}
```

## 原案差分メモ

- 全件を open + disabled に固定した静的表示にしました。参照元（対応表 ID
  R0095）は全閉または先頭 1 項目のみ open を想定していますが、無 JS の
  docs サイトでは閉じた項目のトリガーがクリック・Enter/Space に反応しない
  フォーカス可能な `<button>` として残り、本文が事実上到達不能になります
  （`changelog-accordion` 等、先行する block 3 件が受けた指摘と同じ判断）。
- R0466（先頭 1 項目目のみ open）は、この全件 open 方針にそのまま吸収
  されるため個別インスタンス化していません。
- R0925（問い合わせ導線なし）は、本 Demo から末尾の問い合わせセクション
  を省いた形に相当します。1 つの Demo に両形を並記するほどの構造差では
  ないため、本メモでの記載のみとしています。
- Q&A の項目・文言はすべて独自に書いた架空のものにしました。
- 開閉インジケータはアイコンを使わず、既存部品のテキスト「▾」にしまし
  た。
- 配色・余白・角丸は既存のテーマトークンに従っています。

関連情報: [Heading](../themes/heading.md) / [Text](../themes/text.md) /
[Badge](../themes/badge.md) / [Accordion](../themes/accordion.md) /
[Button](../themes/button.md)
