# faq-static-grid

`fandhe-frontend-pre-styled-ui` の `heading` / `text` / `link` / `button`
部品を合成した、開閉 UI を持たない常時表示の FAQ グリッドです。Blocks
セクションは新規部品を追加するものではなく、既存の Themes/Primitives
部品を組み合わせた実例集であることに注意してください（主参照は対応表 ID
R0927、差分は R0096 / R0468 / R0928 / R0929 / R0930 を集約。出典の固有名・
ファイル名は記載しません）。

左寄せの見出しとリード文（問い合わせ導線を含むインラインリンク）の下に、
「質問の小見出し + 回答段落」の組を常時表示のまま並べます。グリッドの
列数は狭い幅で 1 列、`sm`（40rem）以上で 2 列、`lg`（64rem）以上で 3 列に
切り替わります。末尾には送信先を持たない問い合わせボタンを 2 個添えます。

本 Demo は静的な表示例であり、`accordion` 等の開閉可能な部品を一切使わず、
質問と回答を最初から並べて表示します。`<form>` 要素は一切持たず、データの
取得・送信・状態管理を行いません。ボタンはいずれも `type="button"` の
まま送信先を持ちません。文言はすべて独自に書いた架空のものであり、実企業
名・実クレデンシャル・PII を含みません。

## Rust コード

```rust
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps, LinkVariant};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};

/// 架空の Q&A 一覧（実在の企業名・個人情報は含まない）。3 列 × 2 行が
/// 埋まる 6 件（[`LAYOUT_CSS`] の `lg` 以上 3 列との整合、モジュール doc
/// 「レイアウト」節）。
const FAQS: [(&str, &str); 6] = [
    (
        "アカウントは何人まで招待できますか",
        "プランごとに上限人数が異なります。上限に達した場合はプラン変更でメンバー数を拡張できます。",
    ),
    (
        "利用データのバックアップは自動で行われますか",
        "日次で自動バックアップを取得しています。手動でのエクスポートもいつでも可能です。",
    ),
    (
        "無料トライアル期間はありますか",
        "登録から 14 日間、主要機能を無料でお試しいただけます。クレジットカード登録は不要です。",
    ),
    (
        "プランのダウングレードはいつでもできますか",
        "次回更新日からのダウングレードが可能です。日割りでの即時反映は行っておりません。",
    ),
    (
        "API の利用回数に上限はありますか",
        "プランごとに月間の呼び出し回数上限を設けています。詳細は料金ページをご確認ください。",
    ),
    (
        "退会時にデータは削除されますか",
        "退会手続き完了後、一定期間の保持を経てデータを削除します。保持期間中は再開により復元できます。",
    ),
];

/// 導入部（左寄せの見出し + 問い合わせ導線を含むリード文）。
fn header() -> Node {
    div(
        vec![("class", "blocks-faq-static-grid-header")],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    ..HeadingProps::default()
                },
                vec![],
                vec![text("よくある質問")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![
                    text("こちらで解決しない場合は、"),
                    link::root(
                        REPO,
                        &LinkProps {
                            variant: LinkVariant::Underline,
                            ..LinkProps::default()
                        },
                        vec![],
                        vec![text("お問い合わせフォーム")],
                    ),
                    text("よりご連絡ください。"),
                ],
            ),
        ],
    )
}

/// FAQ 1 件分（質問の小見出し + 回答段落。開閉 UI は持たず常時表示）。
fn faq_entry(question: &str, answer: &str) -> Node {
    div(
        vec![("class", "blocks-faq-static-grid-item")],
        vec![
            heading(
                HeadingLevel::H4,
                &HeadingProps::default(),
                vec![],
                vec![text(question)],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(answer)],
            ),
        ],
    )
}

/// 常時表示の FAQ グリッド本体（狭幅 1 列 → `sm` 2 列 → `lg` 3 列、
/// モジュール doc「レイアウト」節）。
fn faq_grid() -> Node {
    div(
        vec![("class", "blocks-faq-static-grid-grid")],
        FAQS.iter()
            .map(|(question, answer)| faq_entry(question, answer))
            .collect(),
    )
}

/// 末尾の問い合わせボタン行（R0468 の「下部ボタン 2 個」を集約）。
fn contact() -> Node {
    div(
        vec![("class", "blocks-faq-static-grid-actions")],
        vec![
            button::button(&ButtonProps::default(), vec![], vec![text("お問い合わせ")]),
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Outline,
                    ..ButtonProps::default()
                },
                vec![],
                vec![text("資料をダウンロード")],
            ),
        ],
    )
}

/// `faq-static-grid` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数（モジュール doc「開閉 UI を持たない」節）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-faq-static-grid-layout")],
        vec![header(), faq_grid(), contact()],
    )
}
```

## 原案差分メモ

- 主参照（R0927）は左寄せの導入部 + `sm` 2 列 / `lg` 3 列のグリッドで、
  本 Demo はそのまま踏襲しています。
- R0928 / R0930（中央寄せの導入部）は、`header` の class に
  `align-items: center; text-align: center` を足した形に相当します。1 つの
  Demo に両形を並記するほどの構造差ではないため、本メモでの記載のみと
  しています。
- R0929 / R0930（2 列までのグリッド）は、`lg` 側の `@media` 規則を外し
  常に `sm` 以上で 2 列のまま据え置いた形に相当します。
- R0468（下部ボタン 2 個）は、末尾の `contact` セクションへそのまま
  取り込みました。
- R0096 も同型の「開閉なし FAQ グリッド」構造の差分として、上記の列数・
  導入部の寄せ方のバリエーションに吸収されます。
- Q&A の項目・文言はすべて独自に書いた架空のものにしました。
- アイコンは使わず、既存部品の組み合わせのみで構成しています。
- 配色・余白・角丸は既存のテーマトークンに従っています。

関連情報: [Heading](../themes/heading.md) / [Text](../themes/text.md) /
[Link](../themes/link.md) / [Button](../themes/button.md)
