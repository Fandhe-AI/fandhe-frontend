# faq-tabbed-accordion

`fandhe-frontend-pre-styled-ui` の `heading` / `text` / `badge` / `tabs` /
`accordion` 部品を合成した、カテゴリ切替タブ + アコーディオンの FAQ
セクションです。Blocks セクションは新規部品を追加するものではなく、既存の
Themes/Primitives 部品を組み合わせた実例集であることに注意してください
（主参照は対応表 ID R0470。出典の固有名・ファイル名は記載しません）。

中央寄せの見出しエリアの下にカテゴリ切替の tabs を置き、選択中カテゴリの
FAQ を accordion で並べます。tabs は先頭カテゴリを選択した状態で描画し、
狭い幅ではタブ列だけが横スクロールします（ページ全体はスクロールしませ
ん）。

本 Demo は静的な表示例です。tabs は先頭カテゴリのみ選択・操作可能とし、
残りのカテゴリは `disabled` にしています（選択中タブのクリックは JS が
あっても no-op のため、見た目と挙動にずれが生じません）。各カテゴリの
accordion は全項目を常時展開（open）した状態で固定し、`disabled` により
開閉操作自体を無効化しています（理由は「原案差分メモ」節を参照）。
`<form>` 要素は一切持たず、データの取得・送信・状態管理を行いません。
文言はすべて独自に書いた架空のものであり、実企業名・実クレデンシャル・
PII を含みません。

## Rust コード

```rust
use fandhe_frontend_core::{div, el, span, text, Node};
use fandhe_frontend_pre_styled_ui::accordion::{
    self, item, item_content, item_indicator, item_trigger, AccordionProps, OpenState,
};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps, BadgeVariant};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::tabs::{
    self, ActivationMode, Orientation, TabItem, TabsProps, TabsVariant,
};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// カテゴリ 1 件分の型（value, label, その 3 件の Q&A）。
type Category = (
    &'static str,
    &'static str,
    [(&'static str, &'static str); 3],
);

/// カテゴリ別の架空 Q&A。実在の企業名・個人情報は含まない。
const CATEGORIES: [Category; 3] = [
    (
        "billing",
        "料金・契約",
        [
            (
                "無料プランでも試せますか",
                "主要な機能は無料プランでもお試しいただけます。",
            ),
            (
                "契約期間の縛りはありますか",
                "月単位でのご契約で、最低利用期間の縛りはありません。",
            ),
            (
                "支払い方法は何が使えますか",
                "主要なクレジットカードに対応しています。",
            ),
        ],
    ),
    (
        "features",
        "機能",
        [
            (
                "他ツールからデータを移行できますか",
                "主要なフォーマットでの書き出し・取り込みに対応しています。",
            ),
            (
                "API 連携はありますか",
                "REST API 経由で主要な操作を自動化できます。",
            ),
            (
                "利用人数の上限はありますか",
                "プランごとに上限が異なります。詳細はプラン一覧をご確認ください。",
            ),
        ],
    ),
    (
        "support",
        "サポート",
        [
            (
                "サポートへの問い合わせ方法を教えてください",
                "サポート窓口へメールでご連絡ください。通常 1 営業日以内に返信します。",
            ),
            (
                "障害情報はどこで確認できますか",
                "ステータスページで稼働状況を確認できます。",
            ),
            (
                "導入支援はありますか",
                "有償の導入支援プランをご用意しています。",
            ),
        ],
    ),
];

/// 見出しエリア（アイブロウ badge + 中央寄せ見出し + 説明文）。
fn header() -> Node {
    div(
        vec![("class", "blocks-faq-tabbed-accordion-header")],
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
                vec![text("カテゴリ別によくあるご質問")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text("知りたい内容のカテゴリを選んでください。")],
            ),
        ],
    )
}

/// FAQ 1 件分の accordion item（トリガー + 本文）。全件を
/// [`OpenState::Open`] + `disabled: true` で固定する（モジュール doc
/// 「アコーディオンは全件 open + disabled」節）。
fn faq_item(category: &str, index: usize, question: &str, answer: &str) -> Node {
    let state = OpenState::Open;
    let props = AccordionProps {
        disabled: true,
        ..AccordionProps::default()
    };
    let trigger_id = format!("blocks-faq-tabbed-accordion-{category}-{index}-trigger");
    let content_id = format!("blocks-faq-tabbed-accordion-{category}-{index}-content");

    item(
        state,
        false,
        &props,
        vec![],
        vec![
            el(
                "h4",
                vec![("class", "blocks-faq-tabbed-accordion-trigger-heading")],
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
                            vec![("class", "blocks-faq-tabbed-accordion-trigger-label")],
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

/// カテゴリ 1 件分の accordion（そのカテゴリの FAQ 全件を包む）。
fn category_accordion(category: &str, faqs: &[(&str, &str); 3]) -> Node {
    let items: Vec<Node> = faqs
        .iter()
        .enumerate()
        .map(|(index, (question, answer))| faq_item(category, index, question, answer))
        .collect();

    accordion::root(
        Size::Md,
        &AccordionProps::default(),
        vec![("data-blocks-faq-tabbed-accordion-root", "")],
        items,
    )
}

/// カテゴリ切替 tabs 本体。先頭カテゴリのみ選択・非 disabled とし、
/// 残りは disabled にする（モジュール doc「tabs は実物を 1 インスタンスだけ
/// 使う」節）。
fn category_tabs() -> Node {
    let items: Vec<TabItem<'_>> = CATEGORIES
        .iter()
        .enumerate()
        .map(|(index, (value, label, faqs))| TabItem {
            value,
            trigger: vec![text(*label)],
            content: vec![category_accordion(value, faqs)],
            disabled: index != 0,
        })
        .collect();

    div(
        vec![("class", "blocks-faq-tabbed-accordion-tabs")],
        vec![tabs::tabs(
            TabsVariant::Line,
            Size::Md,
            ColorPalette::Accent,
            &TabsProps {
                id: "blocks-faq-tabbed-accordion-tabs",
                selected: CATEGORIES[0].0,
                orientation: Orientation::Horizontal,
                activation_mode: ActivationMode::Automatic,
                loop_focus: true,
                indicator: false,
            },
            items,
        )],
    )
}

/// `faq-tabbed-accordion` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-faq-tabbed-accordion-layout")],
        vec![header(), category_tabs()],
    )
}
```

## 原案差分メモ

- tabs は実物の `tabs::tabs` を 1 インスタンスだけ使い、先頭カテゴリの
  みを選択・非 disabled、残りを disabled にしました。`feature-tabs-panel`
  / `feature-vertical-tabs` は「非選択パネルが見えないのに操作できそうな
  トリガーが残る」という指摘を受けて実物の tabs を使うのをやめた前例が
  ありますが、本 block はイシューが使用部品に `tabs` を挙げているため、
  全タブをフォーカス可能にする代わりに非選択タブを disabled にすること
  で同じ問題を避けています。
- 各カテゴリの accordion は全件を open + disabled で固定した静的表示に
  しました。原案の「全閉」表示は、無 JS の docs サイトでは閉じた項目の
  トリガーがクリック・Enter/Space に反応しないフォーカス可能な
  `<button>` として残り、本文が事実上到達不能になるため採用していません
  （`faq-accordion-centered` 等、先行する block が受けた指摘と同じ判断）。
- 非選択カテゴリのパネルは `tabs::tabs` の仕様どおり `hidden` で出力し、
  実際の FAQ をそのまま入れています（実利用と同じマークアップにするた
  め）。状態違いを並記する構成は採っていません。
- カテゴリ・Q&A の項目・文言はすべて独自に書いた架空のものにしました。
- 配色・余白・角丸は既存のテーマトークンに従っています。

関連情報: [Heading](../themes/heading.md) / [Text](../themes/text.md) /
[Badge](../themes/badge.md) / [Tabs](../themes/tabs.md) /
[Accordion](../themes/accordion.md)
