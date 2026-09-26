//! `faq-split-static` block（イシュー #2846。親トラッキング #2730
//! 「Blocks 目的別パーツ拡充」配下、対応表 ID R0924 のみを参照する合成例。
//! 左に見出し + リード文、右に開閉のない Q&A 一覧を並べる 2 カラム FAQ。
//! 取得手段・ファイル名・内部コンポーネント識別子は記載しない
//! ([`super::faq_accordion_centered`] と同じライセンス上の転記制限、対応表
//! ID のみを記す）。
//!
//! # 使用部品
//!
//! `heading` / `text` / `link` の 3 部品のみを合成する（[`BLOCK`] の
//! `parts` に一致させる契約、`crates/docs-site/tests/blocks_nav.rs`/
//! `blocks_contract.rs` が検証する）。新しい UI 部品は追加しない
//! （イシュー本文の明示制約）。
//!
//! # [`super::faq_accordion_centered`] との違い（開閉 UI を持たない）
//!
//! [`super::faq_accordion_centered`] は `accordion` で開閉可能な一覧を
//! 表すが、本 block は Q&A を常時展開の静的な見出し + 段落の並びとして
//! 表す。そのため disabled 固定・`aria-controls` 等のアコーディオン固有の
//! 対処は不要（開閉状態自体が存在しない）。
//!
//! # レイアウト（`lg` で 5fr/7fr の 2 カラム、それ未満は縦積み）
//!
//! [`super::super::section_heading::section_heading_split`] と同じく
//! テーマの breakpoint トークンは `@media` 条件式の中では解決できないため、
//! `fandhe_frontend_pre_styled_ui::recipe::Breakpoint::Lg`（1024px = 64rem）
//! と一致するリテラル値を [`LAYOUT_CSS`] へ直書きする。`lg` 未満はルートが
//! 単一列の grid となり、左の見出しエリア・右の一覧が自然に縦積みになる
//! （個別の `flex-direction` 切り替えを要しない）。
//!
//! # 見出しレベル
//!
//! ページ側が `## Demo` として `h2` を出すため、左列の見出しは
//! [`HeadingLevel::H3`]。各質問は [`HeadingLevel::H4`] で、ページ本文の
//! 見出し階層（`h2` → `h3` → `h4`）に沿う。
//!
//! # 問い合わせリンク（固定リポジトリ URL、`mailto:`/`tel:`/`href="#"` を
//! 使わない理由）
//!
//! [`super::super::contact::contact_split_info`] と同じく、リンク先は
//! 架空の問い合わせ窓口 URL を捏造せず [`REPO`]（自プロジェクトの実在
//! リポジトリ）へ固定する。可視テキストは遷移先が分かる「GitHub の
//! リポジトリ」とし、`href="#"` は使わない。
//!
//! # `text` の名前衝突
//!
//! `fandhe_frontend_pre_styled_ui::text::text`（`<p>` を組み立てる styled
//! パート関数）と `fandhe_frontend_core::text`（テキストノード生成関数）が
//! 同名のため、styled 側を `styled_text` として取り込む（`crate::blocks`
//! 内の他 block と同じ回避方法）。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `heading::heading` / `text::text` / `link::root` はいずれも
//! `drop_class_attr` により呼び出し側 `attrs` の `class` を黙って除去する
//! ため、レイアウト用の class は素の `div` にのみ付与する
//! （`.blocks-faq-split-static-*` クラスセレクタ）。
//!
//! # 文言
//!
//! `crate::blocks::dummy_assets` は `pub(crate)` のため使わない
//! （[`super::faq_accordion_centered`] と同じ判断）。架空の日本語 Q&A を
//! `const` 配列で持つ（実在の企業名・個人情報は含まない。
//! [`super::faq_accordion_centered`] とは別の質問文にし重複を避ける）。
//!
//! # `<form>` を持たない・データ取得/送信を行わない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
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
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/faq-split-static/",
    title: "faq-split-static",
    category: BlockCategory::Faq,
    rust_source: "crates/docs-site/src/blocks/marketing/faq/faq_split_static.rs",
    demo_class: "blocks-faq-split-static",
    parts: &[
        Part {
            label: "Heading",
            path: "/themes/heading/",
        },
        Part {
            label: "Text",
            path: "/themes/text/",
        },
        Part {
            label: "Link",
            path: "/themes/link/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `faq_split_static` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節）。セレクタは
/// `.blocks-faq-split-static-*` のみを用い、他 block や部品の素のセレクタへ
/// 影響させない。
const LAYOUT_CSS: &str = "\
.blocks-faq-split-static-layout {\n  display: grid;\n  gap: var(--fandhe-space-10);\n}\n\
.blocks-faq-split-static-intro {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-faq-split-static-list {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n}\n\
.blocks-faq-split-static-item {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n}\n\
@media (min-width: 64rem) {\n  .blocks-faq-split-static-layout {\n    grid-template-columns: minmax(0, 5fr) minmax(0, 7fr);\n    align-items: start;\n  }\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, FAQS, LAYOUT_CSS, REPO};
    use fandhe_frontend_core::render;

    /// Demo が期待する 3 種の部品・非対話制約を満たすことの単体回帰
    /// （`crates/docs-site/tests/blocks_contract.rs` の横断検査と重複し
    /// 過ぎない範囲での個別固定）。
    #[test]
    fn demo_composes_expected_parts() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"link\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert!(!html.contains("<form"));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("src=\"data:"));
        assert!(!html.contains("mailto:"));
        assert!(!html.contains("tel:"));
    }

    /// 質問数と H4 の出現数が [`FAQS`] の件数に一致すること（開閉のない
    /// 常時展開の一覧であることの固定）。
    #[test]
    fn demo_renders_all_faq_items_as_static_headings() {
        let html = render(&demo());
        assert_eq!(html.matches("<h4 ").count(), FAQS.len(), "html={html}");
        for (question, answer) in FAQS {
            assert!(
                html.contains(question),
                "html should contain question {question}"
            );
            assert!(html.contains(answer), "html should contain answer {answer}");
        }
    }

    /// リンク先が [`REPO`] に固定され、実際に押せる URL であること。
    #[test]
    fn demo_link_targets_repo_url() {
        let html = render(&demo());
        assert_eq!(
            html.matches(&format!("href=\"{REPO}\"")).count(),
            1,
            "html={html}"
        );
    }

    /// [`LAYOUT_CSS`] が `lg`（64rem）で 5fr/7fr の 2 カラムへ切り替わる
    /// ことを固定する（モジュール doc「レイアウト」節）。
    #[test]
    fn layout_css_switches_to_two_columns_at_lg() {
        assert!(LAYOUT_CSS.contains("@media (min-width: 64rem)"));
        assert!(LAYOUT_CSS.contains("minmax(0, 5fr) minmax(0, 7fr)"));
    }

    /// ルート class（`demo_class` とは別名）が [`demo`] の出力へ実際に
    /// 現れること（`faq_accordion_centered` 等と同じ Bugbot 教訓の固定）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-faq-split-static-layout\""));
        assert_ne!(super::BLOCK.demo_class, "blocks-faq-split-static-layout");
    }
}
