//! `careers-split-accordion` block（イシュー #2816。親 #2807「Phase 2:
//! Blocks マーケティング B」配下、Marketing / Careers カテゴリの最初の
//! block。兄弟イシュー #2815「careers-card-grid」と同じカテゴリを担当する）。
//!
//! # 使用部品
//!
//! `heading` / `text` / `badge` / `accordion` / `icon` / `button` の 6 部品を
//! 合成する（[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_contract.rs` が検証する）。
//!
//! # レイアウト（md 以上で 2 カラム、狭幅で 1 カラム）
//!
//! `.blocks-careers-split-accordion-layout` は既定 1 列グリッドで、
//! `@media (min-width: 48rem)` 以上のとき左（見出し + 説明）/ 右（求人
//! アコーディオン一覧）の 2 カラムへ切り替える（[`LAYOUT_CSS`]）。狭幅では
//! DOM 順そのまま見出しの下にアコーディオンが縦続する（CSS 側の並び替え
//! は行わない）。
//!
//! # 無 JS のため全件展開で固定表示
//!
//! docs サイトは JS ハイドレーションを行わない。当初は先頭項目（`job-1`）
//! のみ [`OpenState::Open`] とし残り 3 件を `OpenState::Closed`（
//! `item_content` に `hidden`）で表示していたが、開閉を切り替える手段が
//! ない無 JS 環境ではこの `hidden` が恒久的な到達不能を意味し、閲覧者
//! （キーボード利用者を含む）が 2〜4 件目の求人説明・勤務地・雇用形態・
//! 応募ボタンへ一切到達できないという不具合だった（codex レビュー P1
//! 指摘、イシュー #2816）。「見かけの開閉可能な操作契約」と「実際には
//! 開閉できない静的表示」の不一致を解消するため、4 件すべてを
//! [`OpenState::Open`] で固定表示する（トリガーの `<button>`/
//! `aria-expanded` 自体は headless-ui の accordion anatomy をそのまま
//! 使うため残るが、これは見た目上のアコーディオン UI を維持しつつ
//! 中身を常時アクセス可能にするための選択であり、無 JS で押しても
//! 状態は変化しない）。`pricing_tiers_morph`/`sidebar_07` 等の他 block が
//! 採る「無 JS では代表状態を 1 つ固定表示する」判断は、複数状態を
//! 併記しない分にはコンテンツ到達性を損なわない場面（見た目の差分確認が
//! 目的の状態）でのみ有効であり、本 block のように閉状態が実データへの
//! 到達を阻む場合には適用しない。
//!
//! # トリガーを `h4` で包む理由（目次への漏れ防止）
//!
//! `crate::layout::inject_heading_anchors` はページ全体を再帰的に走査して
//! h2/h3 を目次へ採録する。本 block のセクション見出し（[`heading::heading`]
//! の `H3`）は目次対象のままでよいが、アコーディオン項目 4 件の見出し
//! ラッパーまで目次へ混入すると求人名がナビゲーションを埋めてしまうため、
//! `crates/docs-site/src/showcase.rs::accordion_section` の `h3` ラップとは
//! 異なり本 block は `h4` を使う（h2/h3 のみを対象とする収集契約の外側へ
//! 意図的に逃がす）。
//!
//! # id 規約
//!
//! `blocks-careers-split-accordion-job-{1..4}-trigger`/`-content` を固定で
//! 割り当てる。`aria-controls`/`aria-labelledby` の参照先はいずれも同一
//! Demo 内に実在し、`crates/docs-site/tests/blocks_contract.rs` の
//! `demo_output_has_no_dangling_aria_references_or_duplicate_ids` が
//! id 重複・参照切れの両方を横断的に固定する。
//!
//! # `<button>` の中身は phrasing content に限る
//!
//! トリガー（[`accordion::item_trigger`]）の子は `span` 2 個（ラベル +
//! インジケータ）のみで、`div`/見出しは入れない。応募ボタンの子も
//! `span`（ラベル）+ アイコンのみ。
//!
//! # `<form>` を使わない・実データを持たない・アプリロジックを持たない
//!
//! `crate::blocks` モジュール doc の不変条件どおり `<form>` を出力しない。
//! 応募ボタンは [`button::button`] の既定 `type="button"` のまま送信先を
//! 持たない静的な合成例であり、実際の応募処理は利用者側の Rust コードで
//! 実装する（`docs/policy/intentional-non-adoption.md` §3.25: UI コンポー
//! ネント層はアプリケーションロジックを内包しない）。職種・部署・勤務地・
//! 説明文はすべて架空のものであり、実企業名・実クレデンシャル・PII を
//! 含まない。
//!
//! # 参照元との違い
//!
//! 対応表 ID R0034（集約元 1 件）から構造だけを参照した。文言・配色・
//! 装飾・アイコンは持ち込まず、独自の日本語文言と自作の幾何線画アイコン
//! （chevron/pin/clock/arrow の 4 種、[`icon::icon`] + `el("path")`）で
//! 組み立てる。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `accordion::root`/`button::button`/`badge::badge`/`heading::heading`/
//! `text::text` は `drop_class_attr` により呼び出し側 `attrs` の `class` を
//! 黙って除去する契約を持つため、これらへの Demo 固有スタイルは
//! `data-blocks-careers-split-accordion-*` 属性で渡す。素の `div`/`span` と
//! headless の `item`/`item_trigger`/`item_content`（`class` をそのまま
//! 素通しする）側には通常のクラスセレクタを使う（`crate::blocks` モジュール
//! doc「CSS フックが `class` と `[data-*]` で混在する理由」節と同じ判断軸）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, el, span, text, Node};
use fandhe_frontend_pre_styled_ui::accordion::{self, AccordionProps, OpenState};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps, BadgeVariant};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::heading::{
    self, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// 求人 1 件分の静的データ（架空の職種・部署・説明・勤務地・雇用形態）。
struct Job {
    dept: &'static str,
    title: &'static str,
    summary: &'static str,
    location: &'static str,
    employment: &'static str,
}

/// 4 件の求人定義。無 JS のため全件を開いた状態で Demo 表示する
/// （モジュール doc「無 JS のため全件展開で固定表示」参照）。
const JOBS: &[Job] = &[
    Job {
        dept: "エンジニアリング",
        title: "バックエンドエンジニア",
        summary: "サーバーサイド API の設計・実装・運用を担当します。",
        location: "リモート（日本国内）",
        employment: "正社員",
    },
    Job {
        dept: "デザイン",
        title: "プロダクトデザイナー",
        summary: "利用者体験の調査からビジュアルデザインまで一貫して担当します。",
        location: "東京オフィス",
        employment: "正社員",
    },
    Job {
        dept: "セールス",
        title: "フィールドセールス",
        summary: "既存顧客との関係構築と新規商談の推進を担当します。",
        location: "大阪オフィス",
        employment: "契約社員",
    },
    Job {
        dept: "カスタマーサポート",
        title: "サポートスペシャリスト",
        summary: "問い合わせ対応とナレッジベースの整備を担当します。",
        location: "リモート（日本国内）",
        employment: "業務委託",
    },
];

/// 自作の単純な線画アイコン（`d` は呼び出し側が座標を選ぶ、
/// `sidebar_03::geo_icon`/`footer_newsletter::checkmark_icon` と同型）。
/// lucide 等の既存アイコンセットの path は複製しない。
fn geo_icon(path_d: &'static str) -> Node {
    icon(
        &IconProps {
            size: Size::Sm,
            ..IconProps::default()
        },
        vec![],
        vec![el(
            "path",
            vec![
                ("d", path_d),
                ("fill", "none"),
                ("stroke", "currentColor"),
                ("stroke-width", "2"),
                ("stroke-linecap", "round"),
                ("stroke-linejoin", "round"),
            ],
            vec![],
        )],
    )
}

/// シェブロン（下向き、アコーディオンの開閉インジケータ）。
fn chevron_icon() -> Node {
    geo_icon("M6 9l6 6 6-6")
}

/// ピン（勤務地メタ行）。
fn pin_icon() -> Node {
    geo_icon("M12 21s7-7.5 7-12a7 7 0 1 0-14 0c0 4.5 7 12 7 12zM12 11a2 2 0 1 0 0-4 2 2 0 0 0 0 4z")
}

/// 時計（雇用形態メタ行）。
fn clock_icon() -> Node {
    geo_icon("M12 7v5l3 3M12 21a9 9 0 1 0 0-18 9 9 0 0 0 0 18z")
}

/// 矢印（応募ボタン）。
fn arrow_icon() -> Node {
    geo_icon("M5 12h14M13 6l6 6-6 6")
}

/// メタ行 1 項目（アイコン + テキストの `span`）。
fn meta_item(item_icon: Node, label: &str) -> Node {
    span(
        vec![("data-blocks-careers-split-accordion-meta-item", "")],
        vec![item_icon, text(label)],
    )
}

/// 求人 1 件分の `item`（トリガー + 本文）を組み立てる。
fn job_item(index: usize, job: &Job, accordion_props: &AccordionProps) -> Node {
    // 無 JS のため開閉を切り替える手段がなく、`OpenState::Closed` にすると
    // `item_content` に `hidden` が付与され本文（勤務地・雇用形態・応募
    // ボタン）へ到達不能になる（P1 指摘、イシュー #2816）。全件を
    // `OpenState::Open` に固定し、求人内容を常時閲覧・操作可能にする。
    let state = OpenState::Open;
    let value = format!("job-{}", index + 1);
    let trigger_id = format!("blocks-careers-split-accordion-job-{}-trigger", index + 1);
    let content_id = format!("blocks-careers-split-accordion-job-{}-content", index + 1);
    let apply_label = format!("応募する（{}）", job.title);

    let trigger_label = span(
        vec![("class", "blocks-careers-split-accordion-trigger-label")],
        vec![
            span(vec![], vec![text(job.title)]),
            badge::badge(
                &BadgeProps {
                    variant: BadgeVariant::Subtle,
                    palette: ColorPalette::Neutral,
                    ..BadgeProps::default()
                },
                vec![("data-blocks-careers-split-accordion-dept", "")],
                vec![text(job.dept)],
            ),
        ],
    );

    let trigger = el(
        "h4",
        vec![("class", "blocks-careers-split-accordion-item-heading")],
        vec![accordion::item_trigger(
            state,
            false,
            accordion_props,
            value.as_str(),
            Some(trigger_id.as_str()),
            Some(content_id.as_str()),
            vec![],
            vec![
                trigger_label,
                accordion::item_indicator(
                    state,
                    false,
                    accordion_props,
                    vec![],
                    vec![chevron_icon()],
                ),
            ],
        )],
    );

    let body = div(
        vec![("class", "blocks-careers-split-accordion-body")],
        vec![
            styled_text::text(&TextProps::default(), vec![], vec![text(job.summary)]),
            div(
                vec![("class", "blocks-careers-split-accordion-meta")],
                vec![
                    meta_item(pin_icon(), job.location),
                    meta_item(clock_icon(), job.employment),
                ],
            ),
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Outline,
                    ..ButtonProps::default()
                },
                vec![
                    ("data-blocks-careers-split-accordion-apply", ""),
                    ("aria-label", apply_label.as_str()),
                ],
                vec![span(vec![], vec![text("応募する")]), arrow_icon()],
            ),
        ],
    );

    let content = accordion::item_content(
        state,
        false,
        accordion_props,
        Some(content_id.as_str()),
        Some(trigger_id.as_str()),
        vec![],
        vec![body],
    );

    accordion::item(
        state,
        false,
        accordion_props,
        vec![],
        vec![trigger, content],
    )
}

/// `careers-split-accordion` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。
pub fn demo() -> Node {
    let accordion_props = AccordionProps::default();

    let header = div(
        vec![("class", "blocks-careers-split-accordion-header")],
        vec![
            styled_text::text(
                &TextProps {
                    size: TextSize::Sm,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![("data-blocks-careers-split-accordion-tagline", "")],
                vec![text("採用情報")],
            ),
            heading::heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    weight: HeadingWeight::Bold,
                },
                vec![],
                vec![text("一緒にチームを育てる仲間を募集しています")],
            ),
            styled_text::text(
                &TextProps {
                    size: TextSize::Lg,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(
                    "募集中のポジションから、興味のある職種をご覧ください。",
                )],
            ),
        ],
    );

    let items: Vec<Node> = JOBS
        .iter()
        .enumerate()
        .map(|(index, job)| job_item(index, job, &accordion_props))
        .collect();

    let list = accordion::root(
        Size::Md,
        &accordion_props,
        vec![("data-blocks-careers-split-accordion-list", "")],
        items,
    );

    div(
        vec![("class", "blocks-careers-split-accordion-layout")],
        vec![header, list],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/careers-split-accordion/",
    title: "careers-split-accordion",
    category: BlockCategory::Careers,
    rust_source: "crates/docs-site/src/blocks/marketing/careers/careers_split_accordion.rs",
    demo_class: "blocks-careers-split-accordion",
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
            label: "Badge",
            path: "/themes/badge/",
        },
        Part {
            label: "Accordion",
            path: "/themes/accordion/",
        },
        Part {
            label: "Icon",
            path: "/themes/icon/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `careers_split_accordion` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節。他 block と同型で `pub(super)` ではなく
/// [`Block::layout_css`] 経由で `crate::blocks::stylesheet` から連結される）。
const LAYOUT_CSS: &str = "\
.blocks-careers-split-accordion-layout {\n  display: grid;\n  grid-template-columns: minmax(0, 1fr);\n  gap: var(--fandhe-space-8);\n}\n\
@media (min-width: 48rem) {\n  .blocks-careers-split-accordion-layout {\n    grid-template-columns: minmax(0, 1fr) minmax(0, 2fr);\n    align-items: start;\n  }\n}\n\
.blocks-careers-split-accordion-header {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n}\n\
[data-blocks-careers-split-accordion-tagline] {\n  color: var(--fandhe-color-accent-fg-subtle, var(--fandhe-color-accent));\n  text-transform: uppercase;\n  letter-spacing: 0.06em;\n}\n\
.blocks-careers-split-accordion-item-heading {\n  margin: 0;\n  font: inherit;\n}\n\
.blocks-careers-split-accordion-trigger-label {\n  display: inline-flex;\n  align-items: center;\n  flex-wrap: wrap;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-careers-split-accordion-body {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-careers-split-accordion-meta {\n  display: flex;\n  flex-wrap: wrap;\n  gap: var(--fandhe-space-4);\n}\n\
[data-blocks-careers-split-accordion-meta-item] {\n  display: inline-flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n}\n\
[data-blocks-careers-split-accordion-apply] {\n  align-self: flex-start;\n}\n";

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;

    #[test]
    fn demo_uses_expected_six_scopes() {
        let html = render(&demo());
        for scope in ["heading", "text", "badge", "accordion", "icon", "button"] {
            assert!(
                html.contains(&format!(r#"data-scope="{scope}""#)),
                "missing data-scope={scope} in demo output"
            );
        }
    }

    #[test]
    fn demo_has_four_items_all_open_for_no_js_content_reachability() {
        let html = render(&demo());
        assert_eq!(html.matches(r#"data-part="item""#).count(), 4);
        // 無 JS では開閉を切り替えられないため、4 件すべてを
        // `OpenState::Open` で固定表示する（P1 是正、イシュー #2816
        // モジュール doc「無 JS のため全件展開で固定表示」参照）。
        // `hidden` 属性が 1 件も出力されないことを閉状態が残っていない
        // ことの直接証拠として確認する。
        assert_eq!(html.matches(r#"aria-expanded="true""#).count(), 4);
        assert_eq!(html.matches(r#"aria-expanded="false""#).count(), 0);
        // `hidden=""` は `item_content` が closed のときのみ付与される
        // （`crates/headless-ui/src/accordion.rs::item_content` 参照）。
        // `aria-hidden="true"`（indicator の装飾用属性）とは区別する。
        assert!(!html.contains(r#"hidden="""#));
        assert!(html.contains("blocks-careers-split-accordion-job-1-trigger"));
    }

    #[test]
    fn demo_has_eight_type_button_elements() {
        let html = render(&demo());
        assert_eq!(html.matches(r#"type="button""#).count(), 8);
    }

    #[test]
    fn demo_never_leaks_form_or_data_uri_or_placeholder_anchor() {
        let html = render(&demo());
        assert!(!html.contains("<form"));
        assert!(!html.contains(r##"href="#""##));
        assert!(!html.contains("src=\"data:"));
    }

    #[test]
    fn demo_root_class_differs_from_registered_demo_class() {
        assert_ne!(BLOCK.demo_class, "blocks-careers-split-accordion-layout");
    }

    #[test]
    fn layout_css_declares_md_breakpoint_and_two_columns() {
        assert!(LAYOUT_CSS.contains("@media (min-width: 48rem)"));
        assert!(LAYOUT_CSS.contains("grid-template-columns: minmax(0, 1fr) minmax(0, 2fr)"));
    }

    #[test]
    fn every_aria_controls_and_labelledby_target_exists() {
        let html = render(&demo());
        for index in 1..=4 {
            let trigger_id = format!("blocks-careers-split-accordion-job-{index}-trigger");
            let content_id = format!("blocks-careers-split-accordion-job-{index}-content");
            assert!(html.contains(&format!(r#"id="{trigger_id}""#)));
            assert!(html.contains(&format!(r#"id="{content_id}""#)));
            assert!(html.contains(&format!(r#"aria-controls="{content_id}""#)));
            assert!(html.contains(&format!(r#"aria-labelledby="{trigger_id}""#)));
        }
    }
}
