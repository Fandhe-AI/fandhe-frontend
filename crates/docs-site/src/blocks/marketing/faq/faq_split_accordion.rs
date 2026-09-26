//! `faq-split-accordion` block（イシュー #2845。親トラッキング #2807
//! 「Phase 2: Blocks マーケティング B」配下、対応表 ID R0097 を主参照とし、
//! R0467 を差分として集約した合成例。左に見出し・説明・問い合わせ導線、
//! 右にカテゴリ区切りの FAQ アコーディオンを置く 2 カラム構成。取得手段・
//! ファイル名・内部コンポーネント識別子は記載しない
//! （[`super::faq_accordion_centered`] と同じライセンス上の転記制限、
//! 対応表 ID のみを記す）。
//!
//! # 使用部品
//!
//! `heading` / `text` / `accordion` / `button` の 4 部品を合成する
//! （[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//!
//! # レイアウト（lg 以上で 2 カラム、狭幅で 1 カラム）
//!
//! `.blocks-faq-split-accordion-layout` は既定 1 列グリッドで、
//! `@media (min-width: 64rem)`（`Breakpoint::Lg` 相当、blocks 全体で
//! 使用例のある値）以上のとき左（見出し + 説明 + 問い合わせボタン）/
//! 右（カテゴリ別 FAQ アコーディオン）の 2 カラムへ切り替える
//! （[`LAYOUT_CSS`]）。狭幅では DOM 順そのまま見出しの下にアコーディオン
//! が縦続する（CSS 側の並び替えは行わない、
//! [`super::super::careers::careers_split_accordion`] と同じ判断）。
//!
//! # 静的表示（無 JS、全項目を常時 open + disabled で固定）
//!
//! [`super::faq_accordion_centered`]・
//! [`super::super::changelog::changelog_accordion`]・
//! [`super::super::careers::careers_split_accordion`]・
//! [`super::super::feature::feature_accordion_image`] と同じ判断で全件を
//! [`OpenState::Open`] + `AccordionProps { disabled: true, .. }` で固定
//! 描画する。無 JS の docs サイトでは `item_trigger` が `disabled: false`
//! のフォーカス可能な `<button>` として出力されるため、閉じた項目を残す
//! とクリック・Enter/Space が no-op になり本文が事実上到達不能になる
//! （前例が受けた同一のレビュー指摘、
//! `docs/policy/intentional-non-adoption.md` の UI 部品責務境界にある
//! 「アクセシビリティ（WAI-ARIA・キーボード操作）」に反する）。是正として
//! ネイティブ `disabled` 属性・`aria-disabled="true"`（[`item_trigger`]）を
//! 出力してフォーカス不能・操作不能であることを支援技術・キーボード双方に
//! 明示する。`disabled_declarations()`（既定 `opacity: 0.5`）は
//! [`LAYOUT_CSS`] で中和し、通常の見出しと同じ見た目に保つ。
//!
//! # カテゴリ区切りの複数グループ（R0467 との差分）
//!
//! 主参照 R0097 は単一グループの FAQ 一覧を想定するが、集約元 R0467 は
//! 右カラムへ複数の accordion グループをカテゴリ小見出しで区切って並置
//! する構成を持つ。本 block は R0467 の構成を採用し、[`GROUPS`] で
//! カテゴリ名と Q&A の組を持ち、カテゴリごとに独立した `accordion::root`
//! を出力する。参照元との詳しい差は Demo を複数化せず
//! `site/blocks/faq-split-accordion.md` の「原案差分メモ」節に文章で記す。
//!
//! # トリガーの子を 2 個に保つ理由
//!
//! styled `accordion::root` の item-trigger recipe は
//! `justify-content: space-between` を前提に「直接の子 2 個」のレイアウト
//! を取る。そのため [`item_trigger`] の children はラベル領域（`span`）と
//! `item_indicator` の 2 個に保ち、間に他要素を挟まない
//! （[`super::faq_accordion_centered`] と同じ判断）。
//!
//! # id 接頭辞と ARIA 対応
//!
//! `id`/`aria-controls`/`aria-labelledby` はいずれも
//! `blocks-faq-split-accordion-{group}-{index}-{trigger|content}` の形で
//! グループ・項目添字から一意に導出する
//! （`crates/docs-site/tests/blocks_contract.rs::
//! demo_output_has_no_dangling_aria_references_or_duplicate_ids` が全
//! block 横断で id 重複・宙に浮いた参照を検査する）。
//!
//! # 見出しレベル（`H3`/`H4`/`h5`）
//!
//! ページ側が `## Demo` として `h2` を出すため、左カラムのセクション見出し
//! は [`HeadingLevel::H3`] にする。右カラムのカテゴリ小見出しは
//! [`HeadingLevel::H4`] とする。各トリガーは WAI-ARIA APG のアコーディオン
//! パターンに合わせて `<h5>` で包む（[`fandhe_frontend_core::el`] で直接
//! 組み立て、`heading::heading` は使わない。ページ本文の見出し階層に
//! 割り込ませない部品固有の構造要素であるため）。`h4`/`h5` は
//! `crate::layout::inject_heading_anchors` の h2/h3 のみを対象とする収集
//! 契約の外側であり、カテゴリ小見出しと質問文が目次を埋めない
//! （[`super::super::careers::careers_split_accordion`] と同じ判断）。
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
//! `heading::heading` / `text::text` / `button::button` / `accordion::root`
//! はいずれも `drop_class_attr` により呼び出し側 `attrs` の `class` を
//! 黙って除去する契約を持つため、Demo 固有のスタイルフックは
//! `data-blocks-faq-split-accordion-*` 属性で渡す（`accordion::item`/
//! `item_trigger`/`item_content`/`item_indicator` は `drop_class_attr` を
//! 経由しないため子孫セレクタでの上書きに寄せる）。素の `div`/`h5`/`span`
//! には `class` がそのまま効くため、それらは
//! `.blocks-faq-split-accordion-*` クラスセレクタを使う。
//!
//! # ルート class（`demo_class` との別名化）
//!
//! ルート class は `blocks-faq-split-accordion-layout` とし、
//! `demo_class`（`blocks-faq-split-accordion`）とは別名にする
//! （`blog_list_image` 等が受けた Bugbot 教訓の固定、
//! [`super::faq_accordion_centered`] と同じ判断）。
//!
//! # 文言
//!
//! `crate::blocks::dummy_assets` は `pub(crate)` のため使わない（Markdown
//! のコードフェンスが単体でも読めるコード例であり続けるため、
//! [`super::faq_accordion_centered`] と同じ判断）。架空の日本語 Q&A を
//! `const` 配列で持つ（実在の企業名・個人情報は含まない）。
//!
//! # `<form>` を持たない・データ取得/送信を行わない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である。問い合わせボタンは [`button::button`] の既定
//! `type="button"` のまま送信先を持たない。右カラム末尾の「もっと見る」
//! ボタンも `ButtonVariant::Outline` かつ既定 `type="button"` のまま
//! 遷移先を持たない。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
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
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/faq-split-accordion/",
    title: "faq-split-accordion",
    category: BlockCategory::Faq,
    rust_source: "crates/docs-site/src/blocks/marketing/faq/faq_split_accordion.rs",
    demo_class: "blocks-faq-split-accordion",
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
            label: "Accordion",
            path: "/themes/accordion/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `faq_split_accordion` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節。他 block と同型で `pub(super)` ではなく
/// 本ファイル内 private 定数として `super::stylesheet` 経由の `push_css` で
/// 連結される）。
///
/// セレクタは `.blocks-faq-split-accordion-*` と
/// `[data-blocks-faq-split-accordion-*]`、および styled accordion の
/// `[data-scope="accordion"]` 系セレクタへの子孫結合子付き上書き
/// （disabled 中和、モジュール doc「静的表示」節）のみを用い、他 block や
/// 部品の素のセレクタへ影響させない。
const LAYOUT_CSS: &str = "\
.blocks-faq-split-accordion-layout {\n  display: grid;\n  grid-template-columns: minmax(0, 1fr);\n  gap: var(--fandhe-space-8);\n}\n\
@media (min-width: 64rem) {\n  .blocks-faq-split-accordion-layout {\n    grid-template-columns: minmax(0, 1fr) minmax(0, 2fr);\n    align-items: start;\n  }\n}\n\
.blocks-faq-split-accordion-intro {\n  display: flex;\n  flex-direction: column;\n  align-items: flex-start;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-faq-split-accordion-faqs {\n  display: flex;\n  flex-direction: column;\n  align-items: stretch;\n  gap: var(--fandhe-space-8);\n}\n\
.blocks-faq-split-accordion-group {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-faq-split-accordion-more {\n  align-self: flex-start;\n}\n\
.blocks-faq-split-accordion-faqs [data-scope=\"accordion\"][data-part=\"item-trigger\"][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
.blocks-faq-split-accordion-trigger-heading {\n  margin: 0;\n  font-size: inherit;\n  font-weight: inherit;\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, GROUPS, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    fn total_faqs() -> usize {
        GROUPS.iter().map(|(_, items)| items.len()).sum()
    }

    /// Demo が期待する 4 種の部品・非対話制約を満たすことの単体回帰
    /// （`crates/docs-site/tests/blocks_contract.rs` の横断検査と重複し
    /// 過ぎない範囲での個別固定）。
    #[test]
    fn demo_composes_expected_parts() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"accordion\"",
            "data-scope=\"button\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert!(html.contains(r#"type="button""#));
        assert!(!html.contains("<form"));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("src=\"data:"));
    }

    /// 全件 open・全件 disabled（`item-content` の `data-state="closed"` を
    /// 一切出力しない）ことを固定する（モジュール doc「静的表示」節）。
    #[test]
    fn demo_renders_all_items_open_and_disabled() {
        let html = render(&demo());
        let total = total_faqs();
        assert_eq!(
            html.matches(r#"data-part="item" data-state="open""#)
                .count(),
            total,
            "html={html}"
        );
        assert_eq!(
            html.matches("item-content\" data-state=\"closed\"").count(),
            0,
            "html={html}"
        );
        assert_eq!(
            html.matches(r#"aria-disabled="true""#).count(),
            total,
            "html={html}"
        );
    }

    /// カテゴリ区切りの複数グループ（R0467 との差分、モジュール doc
    /// 「カテゴリ区切りの複数グループ」節）を固定する。
    #[test]
    fn demo_has_two_category_groups() {
        let html = render(&demo());
        assert_eq!(
            html.matches(r#"data-scope="accordion" data-part="root""#)
                .count(),
            GROUPS.len()
        );
        assert!(html.matches("<h4").count() >= GROUPS.len());
    }

    /// [`LAYOUT_CSS`] が lg ブレークポイント以上で 2 カラムに切り替わり、
    /// disabled トリガーの既定 `opacity: 0.5` を中和すること（モジュール doc
    /// 「レイアウト」節）。
    #[test]
    fn layout_css_switches_to_two_columns_at_lg() {
        assert!(LAYOUT_CSS.contains("@media (min-width: 64rem)"));
        assert!(LAYOUT_CSS.contains("minmax(0, 2fr)"));
        assert!(LAYOUT_CSS.contains(
            r#".blocks-faq-split-accordion-faqs [data-scope="accordion"][data-part="item-trigger"][data-disabled] {"#
        ));
        assert!(LAYOUT_CSS.contains("opacity: 1;"));
    }

    /// ルート class（`demo_class` とは別名）が [`demo`] の出力へ実際に
    /// 現れること（`blog_list_image` と同じ Bugbot 教訓の固定）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-faq-split-accordion-layout\""));
        assert_ne!(super::BLOCK.demo_class, "blocks-faq-split-accordion-layout");
    }
}
