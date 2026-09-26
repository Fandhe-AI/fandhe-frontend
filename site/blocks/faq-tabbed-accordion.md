# faq-tabbed-accordion

`fandhe-frontend-pre-styled-ui` の `heading` / `text` / `badge` / `accordion`
部品を合成した、カテゴリ別 FAQ セクションです。Blocks セクションは新規
部品を追加するものではなく、既存の Themes/Primitives 部品を組み合わせた
実例集であることに注意してください（主参照は対応表 ID R0470。出典の固有
名・ファイル名は記載しません）。

中央寄せの見出しエリアの下にカテゴリラベルを示す非対話のタブ列を置き、
先頭カテゴリの FAQ を「「{カテゴリ名}」の質問と回答」というキャプション
付きの accordion で並べます。無 JS の docs サイトでは実物の `tabs` で
カテゴリを切り替える経路が作れないため、タブ列は見た目のみを示す装飾
（クリック・キーボード操作はできません、ラベルは `aria-hidden` で装飾扱い
です）とし、残り 2 カテゴリの FAQ は「「{カテゴリ名}」タブを選択した場合
のプレビュー」というキャプション付きの accordion として下に併記します。
カテゴリ名はいずれもタブ列だけでなくキャプションとして可視・支援技術から
読める位置に存在し、3 カテゴリ全件の FAQ が常に閲覧できます。狭い幅では
タブ列だけが横スクロールします（ページ全体はスクロールしません）。

本 Demo は静的な表示例です。各カテゴリの accordion は全項目を常時展開
（open）した状態で固定し、`disabled` により開閉操作自体を無効化していま
す（理由は「原案差分メモ」節を参照）。`<form>` 要素は一切持たず、データ
の取得・送信・状態管理を行いません。文言はすべて独自に書いた架空のもの
であり、実企業名・実クレデンシャル・PII を含みません。

## Rust コード

```rust
use fandhe_frontend_core::{div, el, el_owned, span, text, Node};
use fandhe_frontend_pre_styled_ui::accordion::{
    self, item, item_content, item_indicator, item_trigger, AccordionProps, OpenState,
};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps, BadgeVariant};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::Size;

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
                vec![text(
                    "カテゴリ別によくある質問と回答をまとめて掲載しています。",
                )],
            ),
        ],
    )
}

/// カテゴリラベルのみを装飾として示す非対話タブ列（モジュール doc「実物の
/// `tabs::tabs` を使わない」節）。`role`/`tabindex`/`<button>` を一切持たず、
/// クリック・キーボード操作が可能に見えるトリガーを残さない。各ラベルは
/// `aria-hidden` で支援技術のツリーから除外する（内容は下の見出し付き
/// accordion 群が別途提供するため、情報が欠落しない）。
fn static_tab_list(selected: &str) -> Node {
    el_owned(
        "div",
        vec![
            (
                "class".to_string(),
                "blocks-faq-tabbed-accordion-tablist".to_string(),
            ),
            (
                "id".to_string(),
                "blocks-faq-tabbed-accordion-tablist".to_string(),
            ),
        ],
        CATEGORIES
            .iter()
            .map(|(value, label, _)| {
                let state = if *value == selected {
                    "active"
                } else {
                    "inactive"
                };
                el_owned(
                    "div",
                    vec![
                        (
                            "class".to_string(),
                            "blocks-faq-tabbed-accordion-tab".to_string(),
                        ),
                        ("data-state".to_string(), state.to_string()),
                        ("aria-hidden".to_string(), "true".to_string()),
                    ],
                    vec![text(*label)],
                )
            })
            .collect(),
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

/// 非選択カテゴリの「選択した場合のプレビュー」キャプション付き accordion
/// （モジュール doc「実物の `tabs::tabs` を使わない」節、
/// `feature_tabs_panel::panel_state_preview` と同型）。
fn category_preview(label: &str, category: &str, faqs: &[(&str, &str); 3]) -> Node {
    div(
        vec![("class", "blocks-faq-tabbed-accordion-preview")],
        vec![
            styled_text::text(
                &TextProps {
                    size: TextSize::Sm,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(format!("「{label}」タブを選択した場合のプレビュー"))],
            ),
            category_accordion(category, faqs),
        ],
    )
}

/// 先頭（選択中）カテゴリのキャプション付き accordion（PR #3268 レビュー
/// 是正 2 件目・3 件目、Codex P1 + Bugbot Medium）。[`static_tab_list`] の
/// ラベルは `aria-hidden` で装飾扱いにしているため、先頭カテゴリの本文を
/// [`category_accordion`] のみで描画すると「料金・契約」の名称がどこにも
/// 支援技術から読める形で存在しなくなる（後続 2 カテゴリは
/// [`category_preview`] のキャプションが同じ役割を担っていた）。
/// [`category_preview`] と対になる可視キャプションを付け、全カテゴリで
/// 「カテゴリ名がどこかに可視テキストとして存在する」構成を揃える。
fn category_current(label: &str, category: &str, faqs: &[(&str, &str); 3]) -> Node {
    div(
        vec![("class", "blocks-faq-tabbed-accordion-preview")],
        vec![
            styled_text::text(
                &TextProps {
                    size: TextSize::Sm,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(format!("「{label}」の質問と回答"))],
            ),
            category_accordion(category, faqs),
        ],
    )
}

/// `faq-tabbed-accordion` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。先頭カテゴリを [`static_tab_list`] の選択状態として示し
/// [`category_current`] でキャプション付きの本文を描画したあと、残り
/// 2 カテゴリを [`category_preview`] で併記する（3 カテゴリ全件が常に
/// 可視、かつ全カテゴリ名が可視キャプションとして存在する）。
pub fn demo() -> Node {
    let (first_value, first_label, first_faqs) = CATEGORIES[0];
    let mut children = vec![
        header(),
        static_tab_list(first_value),
        category_current(first_label, first_value, &first_faqs),
    ];
    for (value, label, faqs) in &CATEGORIES[1..] {
        children.push(category_preview(label, value, faqs));
    }
    div(
        vec![("class", "blocks-faq-tabbed-accordion-layout")],
        children,
    )
}
```

## 原案差分メモ

- 当初は実物の `tabs::tabs` を 1 インスタンスだけ使い、先頭カテゴリの
  みを選択・非 disabled、残りを disabled にする構成にしていました。
  しかしこの構成でも、無 JS の docs サイトでは非選択カテゴリのパネルが
  `hidden` のまま到達不能で、見出し下の案内文「知りたい内容のカテゴリを
  選んでください」が指す操作を実行する経路がありませんでした
  （Codex P1 3 件・Bugbot Medium 1 件の指摘、PR #3268）。
  `feature-tabs-panel` / `feature-vertical-tabs` が同種の指摘を受けて
  採った方針（実物の `tabs` を使わず非対話の視覚的タブ列 + 全カテゴリ
  本文の常時可視化）へ切り替え、3 カテゴリ全件の FAQ が常に到達可能な
  静的表示にしました。
- 各カテゴリの accordion は全件を open + disabled で固定した静的表示に
  しました。原案の「全閉」表示は、無 JS の docs サイトでは閉じた項目の
  トリガーがクリック・Enter/Space に反応しないフォーカス可能な
  `<button>` として残り、本文が事実上到達不能になるため採用していません
  （`faq-accordion-centered` 等、先行する block が受けた指摘と同じ判断）。
- カテゴリ・Q&A の項目・文言はすべて独自に書いた架空のものにしました。
- 配色・余白・角丸は既存のテーマトークンに従っています。
- （2 回目のレビュー是正）見出し下の案内文「知りたい内容のカテゴリを
  選んでください」は、操作不能な非対話タブ列への操作を促す文言のまま
  残っており、静的な全件表示と矛盾していました。案内文を「カテゴリ別に
  よくある質問と回答をまとめて掲載しています。」へ変更しました。あわせて
  先頭カテゴリ（「料金・契約」）だけがキャプションを持たず、カテゴリ名が
  `aria-hidden` なタブ列にしか存在しない状態だったため、残り 2 カテゴリと
  同様に可視キャプション「「{カテゴリ名}」の質問と回答」を付けました
  （Codex P1 2 件・Bugbot Medium 1 件の指摘、PR #3268）。

関連情報: [Heading](../themes/heading.md) / [Text](../themes/text.md) /
[Badge](../themes/badge.md) / [Accordion](../themes/accordion.md)
