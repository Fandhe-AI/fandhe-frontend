# faq-split-static

`fandhe-frontend-pre-styled-ui` の `heading` / `text` / `link` 部品を合成
した、左に見出し + リード文、右に開閉のない Q&A 一覧を並べる 2 カラムの
FAQ セクションです。Blocks セクションは新規部品を追加するものではなく、
既存の Themes/Primitives 部品を組み合わせた実例集であることに注意して
ください（参照は対応表 ID R0924 のみ。出典の固有名・ファイル名は記載し
ません）。

`lg`（64rem）以上では左 5/12 に見出しとリード文、右 7/12 に質問と回答の
ペアを縦に並べて常時表示します。`lg` 未満では見出しの下に一覧が続く 1 列
の縦積みになります。`faq-accordion-centered` と異なり開閉 UI は持たず、
すべての質問・回答を常時展開の見出し + 段落として表示します。

本 Demo は静的な表示例であり、`<form>` 要素は一切持たず、データの取得・
送信・状態管理を行いません。リード文中の問い合わせリンクは自プロジェク
トの実在リポジトリ URL に固定しており、架空の窓口 URL は使いません。
文言はすべて独自に書いた架空のものであり、実企業名・実クレデンシャル・
PII を含みません。

## Rust コード

```rust
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps, LinkVariant};
use fandhe_frontend_pre_styled_ui::recipe::ColorPalette;
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};

/// 自プロジェクトの実在リポジトリ URL（架空の問い合わせ先を捏造しない）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// 架空の Q&A 一覧（実在の企業名・個人情報は含まない）。
const FAQS: [(&str, &str); 6] = [
    (
        "チームメンバーを何人まで招待できますか",
        "招待人数に上限はありません。権限はオーナー・編集者・閲覧者の 3 段階から選べます。",
    ),
    (
        "既存のプロジェクトを後から取り込めますか",
        "はい。標準的なエクスポート形式に対応しており、取り込み後も履歴を保ったまま編集を続けられます。",
    ),
    (
        "モバイル環境でも利用できますか",
        "ブラウザからそのままご利用いただけます。専用アプリのインストールは不要です。",
    ),
    (
        "セキュリティ認証の取得状況を教えてください",
        "第三者機関による定期的な監査を受けています。詳細な資料はご要望に応じてお渡しします。",
    ),
    (
        "プラン変更後の請求はどうなりますか",
        "変更内容は次回請求サイクルから反映されます。日割り計算での差額精算にも対応しています。",
    ),
    (
        "解約時にデータはどうなりますか",
        "解約後も一定期間はデータを保持し、期間内であればいつでも書き出しが可能です。",
    ),
];

/// 左列（見出し + リード文 + 問い合わせリンク）。
fn intro() -> Node {
    div(
        vec![("class", "blocks-faq-split-static-intro")],
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
                    text("ご不明な点がこちらで解決しない場合は、"),
                    link::root(
                        REPO,
                        &LinkProps {
                            variant: LinkVariant::Underline,
                            palette: ColorPalette::Neutral,
                            ..LinkProps::default()
                        },
                        vec![],
                        vec![text("GitHub のリポジトリ")],
                    ),
                    text("からお問い合わせください。"),
                ],
            ),
        ],
    )
}

/// Q&A 1 件分（質問見出し + 回答段落）。開閉状態を持たない常時展開。
fn faq_item(question: &str, answer: &str) -> Node {
    div(
        vec![("class", "blocks-faq-split-static-item")],
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

/// 右列（Q&A 一覧を縦に並べる）。
fn faq_list() -> Node {
    div(
        vec![("class", "blocks-faq-split-static-list")],
        FAQS.iter()
            .map(|(question, answer)| faq_item(question, answer))
            .collect(),
    )
}

/// `faq-split-static` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数（モジュール doc「`<form>` を持たない」節）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-faq-split-static-layout")],
        vec![intro(), faq_list()],
    )
}
```

## 原案差分メモ

- 対応表 ID R0924 のみを参照し、複数原案の集約は発生していません。
- 開閉 UI は持たず、質問・回答を常時展開の見出し + 段落として表示しま
  す。`faq-accordion-centered`（開閉可能なアコーディオン）とは構造上
  明確に別の block とし、disabled 固定等の対処は不要です。
- 問い合わせリンクの遷移先は自プロジェクトの実在リポジトリ URL に固定
  し、可視テキストを「GitHub のリポジトリ」として遷移先が分かるように
  しました。`mailto:`・`tel:`・`href="#"` は使っていません。
- 参照元の文言・配色・装飾・アイコンは持ち込まず、Q&A の項目・文言は
  すべて独自に書いた架空のものにしました。
- 配色・余白・角丸は既存のテーマトークンに従っています。

関連情報: [Heading](../themes/heading.md) / [Text](../themes/text.md) /
[Link](../themes/link.md)
