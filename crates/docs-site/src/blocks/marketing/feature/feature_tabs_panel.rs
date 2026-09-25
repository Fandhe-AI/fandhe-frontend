//! `feature-tabs-panel` block（イシュー #2772。親イシュー #2771「Blocks
//! 目的別パーツ拡充ツリー」配下、区分 marketing / カテゴリ Feature に属する。
//! タブで切り替える feature セクションの合成例で、対応表の主参照 ID は
//! R1158（基準形）であり、集約元として R0104/R0478/R0479/R0481 を持つ
//! （取得手段・ファイル名・内部識別子は記載しない）。
//!
//! # #2772 と #2773 の分担
//!
//! 本イシューは骨格（[`demo`] のレイアウト root）と基準形 R1158 のみを
//! 実装する。ピル型タブ（R0478）・パネル内の交互配置（R0479）・中央見出しと
//! カードグリッド（R0481）・進捗バー付きトリガー（R0104）・モバイル縦積み
//! 代替形・複数状態の並記は後続の #2773 が [`variant_state`] を再利用して
//! 追加する。
//!
//! # 使用部品
//!
//! `badge` / `heading` / `text` / `image` の 4 部品を合成する（[`BLOCK`] の
//! `parts` に一致させる契約、`crates/docs-site/tests/blocks_nav.rs`/
//! `blocks_contract.rs` が検証する。実際に描画する部品だけを `parts` に
//! 載せ、#2773 が追加する `card`/`icon`/`button`/`progress` はそのイシューで
//! `parts` へ追記する）。`tabs`（`fandhe_frontend_pre_styled_ui::tabs`）は
//! 次節の理由により使わないため `parts` に含めない。
//!
//! # 無 JS での扱い（`tabs::tabs` を一切使わない静的タブ列）
//!
//! docs サイトは JS ハイドレーションを行わないため、選択したタブへ切り替える
//! 操作は実現できない。当初は [`PANELS`] の要素数（4 件）ぶんの `tabs::tabs`
//! インスタンスを選択状態違いで縦に並べていた（`pricing_tiers_morph`/
//! `sidebar_07` と同型の対処）が、この構成は `role="tab"`/`type="button"`/
//! `tabindex` を持つ操作可能に見えるトリガーボタンを反復して出しながら、
//! クリック・キーボード操作をしても実際には一切切り替わらない
//! （Codex P1 指摘 1 回目）。続いて実物の `tabs::tabs` インスタンスを
//! 1 個（[`PANELS`] 先頭のみ）へ減らし残り 3 パネルを非対話プレビューと
//! する是正を行ったが、その 1 個自体が依然として 4 個の `role="tab"`/
//! `tabindex` トリガーボタンを持ち、クリック・矢印キー操作をしても選択状態
//! が変わらない点は変わらないため、Codex P1 指摘 2 回目（同一箇所への
//! 再指摘）を受けた。この 2 回の指摘はいずれも「操作可能に見えるが実際
//! には切り替わらない要素を一切出さない」という 1 点に帰着するため、本
//! 是正では `tabs::tabs` の呼び出しを完全に取り除く。[`tab_strip`] は
//! `role`/`tabindex`/`<button>` を一切持たない `span` 列（選択中の項目は
//! `data-blocks-feature-tabs-panel-selected` 属性のみで視覚的に強調する）
//! とし、[`PANELS`] の 4 件それぞれについて「その項目を選択した場合の
//! 表示」（[`variant_state`]）を静的タブ列 + パネル本文の組として縦に
//! 併記する（`pricing_tiers_morph`/`sidebar_07` と同じ「複数状態併記」の
//! 発想を維持しつつ、各状態の描画手段を非対話な `span` へ統一する点が
//! 異なる）。これにより全パネル本文が常に可視のまま静的 HTML に現れ、かつ
//! 操作可能に見える要素を一切含まない（`docs/design/
//! docs-site-blocks-section.md` §19 参照）。
//!
//! # id を持たない
//!
//! `tabs::tabs` を使わないため、`TabsProps.id` に由来する
//! `aria-controls`/`aria-labelledby` 相互参照の id を発行する必要がない。
//! 本 block は他の大半の block と同じく id を一切使わない
//! （`crates/docs-site/tests/blocks_contract.rs::
//! demo_output_has_no_dangling_aria_references_or_duplicate_ids` の対象と
//! しては「id 0 件」として自明に通過する）。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `badge::badge`/`heading::heading`/`styled_text::text`/`image::image` は
//! いずれも `drop_class_attr` により呼び出し側 `attrs` の `class` を除去する
//! 契約を持つため、Demo 固有のスタイルフックは
//! `data-blocks-feature-tabs-panel-*` 属性で渡す。[`tab_strip`]・
//! [`variant_state`] は素の `fandhe_frontend_core::span`/`div` のみで
//! 組み立てるため `class` 属性をそのまま使える（[`LAYOUT_CSS`] 側で
//! `.blocks-feature-tabs-panel-*` クラスセレクタとして当てる）。
//!
//! - `.blocks-feature-tabs-panel-list` に `overflow-x: auto` を与え、狭い
//!   幅でタブ列がはみ出してページ全体が横スクロールするのを list 自身の
//!   横スクロールへ閉じ込める（モバイルでのタブ列横スクロール）。
//! - `.blocks-feature-tabs-panel-trigger` は下線タブ（[`TabsVariant::Line`]
//!   相当の見た目）を CSS のみで再現し、選択中の項目
//!   （`[data-blocks-feature-tabs-panel-selected]`）だけへ強調色の下線と
//!   太字を当てる。
//!
//! レイアウト root の class（`blocks-feature-tabs-panel-layout`）は
//! [`Block::demo_class`]（`blocks-feature-tabs-panel`）と意図的に別名にする
//! （既存 block と同じ Bugbot 教訓の回避）。
//!
//! # レスポンシブ（モバイルでのタブ列横スクロール・1 列↔2 列切り替え）
//!
//! `.blocks-feature-tabs-panel-row` は既定で flex column（テキストの下に
//! 画像）。`@media (min-width: 64rem)`（他の Marketing / Feature block と
//! 同じ [`fandhe_frontend_pre_styled_ui::recipe::Breakpoint::Lg`] のリテラル
//! 値。テーマの breakpoint トークンは `@media` 条件式の中では解決できない
//! ため直書きする）では 12 列 grid にし、テキストを 5 列・画像を 7 列へ
//! 割り当てる。DOM 順は常にテキスト → 画像で固定し（読み上げ順を変えない）、
//! 配置は `grid-column` 指定だけで行う。
//!
//! # `text` の名前衝突
//!
//! `fandhe_frontend_pre_styled_ui::text::text` と `fandhe_frontend_core::text`
//! が同名のため、styled 側を `styled_text` として取り込む（`crate::blocks`
//! 内の他 block と同じ回避方法）。
//!
//! # `<form>` を持たない・データ取得/送信を行わない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である。文言はすべて架空のもの（実企業名・実クレデンシャル・PII を
//! 含まない）。画像は [`crate::blocks::dummy_assets`] のビルド時生成
//! プレースホルダー SVG のみを使い、`alt=""`（装飾扱い）で出力する
//! （`data:` URI は使わない）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, span, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::image::{self, AspectRatio, ImageFit, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};

/// 基準形（R1158）1 タブ分のデータ（架空文言）。
struct PanelData {
    value: &'static str,
    label: &'static str,
    title: &'static str,
    body: &'static str,
    image_src: &'static str,
}

/// 基準形の 4 タブ分（#2773 が追加する形もこの配列を再利用できる）。
const PANELS: [PanelData; 4] = [
    PanelData {
        value: "design",
        label: "設計",
        title: "型で不変条件を保証する設計",
        body: "コンポーネント境界と状態遷移を型で表現し、実行時ではなくコンパイル時に誤りを検出します。",
        image_src: dummy_assets::SCREENSHOT_SRC,
    },
    PanelData {
        value: "integration",
        label: "連携",
        title: "既存システムへの段階的な組み込み",
        body: "部分埋め込みからフル機能構成まで、必要な範囲だけを選んで既存ページへ組み込めます。",
        image_src: dummy_assets::PRODUCT_SRC,
    },
    PanelData {
        value: "operations",
        label: "運用",
        title: "単一実行ファイルでの安定運用",
        body: "サーバーとアセットをひとまとめにし、Docker イメージ 1 枚で決定的にデプロイできます。",
        image_src: dummy_assets::BACKGROUND_SRC,
    },
    PanelData {
        value: "analytics",
        label: "分析",
        title: "ビルド成果物の可視化",
        body: "依存グラフとバンドルサイズを継続的に計測し、変化を CI 上で追跡できます。",
        image_src: dummy_assets::LOGO_SRC,
    },
];

/// 各形の直前に置く短い形ラベル（`styled_text::text` の `Sm`/`Muted`）。
/// 本イシューでは基準形（R1158）の 1 件のみを出す（#2773 が形を追加する
/// 際に再利用する）。
fn variant_label(label: &'static str) -> Node {
    styled_text::text(
        &TextProps {
            size: TextSize::Sm,
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![],
        vec![text(label)],
    )
}

/// セクション見出し（badge + 見出し + 説明文）。ページ側が `## Demo` として
/// `h2` を出すため、見出しは `HeadingLevel::H3` にする。
fn section_header(eyebrow: &'static str, title: &'static str, lead: &'static str) -> Node {
    div(
        vec![("class", "blocks-feature-tabs-panel-header")],
        vec![
            badge::badge(&BadgeProps::default(), vec![], vec![text(eyebrow)]),
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl3,
                    weight: HeadingWeight::Bold,
                },
                vec![],
                vec![text(title)],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(lead)],
            ),
        ],
    )
}

/// パネル内容（テキスト列 → 画像列、DOM 順は固定）。
fn panel_row(data: &PanelData) -> Vec<Node> {
    vec![div(
        vec![("class", "blocks-feature-tabs-panel-row")],
        vec![
            div(
                vec![("class", "blocks-feature-tabs-panel-copy")],
                vec![
                    heading(
                        HeadingLevel::H4,
                        &HeadingProps::default(),
                        vec![],
                        vec![text(data.title)],
                    ),
                    styled_text::text(
                        &TextProps {
                            variant: TextVariant::Muted,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text(data.body)],
                    ),
                ],
            ),
            div(
                vec![("class", "blocks-feature-tabs-panel-media")],
                vec![image::image(
                    &ImageProps {
                        fit: ImageFit::Cover,
                        aspect_ratio: AspectRatio::Landscape,
                        shape: ImageShape::Rounded,
                        ..ImageProps::new(data.image_src, "")
                    },
                    vec![("data-blocks-feature-tabs-panel-image", "")],
                )],
            ),
        ],
    )]
}

/// 静的なタブ列（[`PANELS`] のラベルを並べただけの `span` 列）。
/// `role`/`tabindex`/`<button>` を一切持たないため、無 JS サイトで
/// 操作可能に見えて実際には切り替わらない要素を持ち込まない（モジュール doc
/// 「無 JS での扱い」節、Codex P1 指摘 2 回分の是正）。選択中の項目だけを
/// `data-blocks-feature-tabs-panel-selected` 属性で視覚的に強調する。
fn tab_strip(selected: &str) -> Node {
    span(
        vec![("class", "blocks-feature-tabs-panel-list")],
        PANELS
            .iter()
            .map(|data| {
                let mut attrs = vec![("class", "blocks-feature-tabs-panel-trigger")];
                if data.value == selected {
                    attrs.push(("data-blocks-feature-tabs-panel-selected", ""));
                }
                span(attrs, vec![text(data.label)])
            })
            .collect(),
    )
}

/// 1 状態分（[`PANELS`] のいずれか 1 件を選択した場合）の表示。キャプション +
/// 静的タブ列（[`tab_strip`]）+ パネル本文（[`panel_row`]）を並べる。無 JS
/// サイトでは切り替えができないため、[`PANELS`] の 4 件それぞれをこの関数で
/// 縦に並べ、全パネルの内容を静的に併記する（`pricing_tiers_morph`/
/// `sidebar_07` と同型の「複数状態併記」対処。`tabs::tabs` は一切使わず、
/// 静的なタブ列を都度添えるだけに変える点がモジュール doc「無 JS での扱い」
/// 節で述べる是正の要点である）。
fn variant_state(data: &PanelData) -> Node {
    let mut children = vec![
        styled_text::text(
            &TextProps {
                size: TextSize::Sm,
                variant: TextVariant::Muted,
                ..TextProps::default()
            },
            vec![],
            vec![text(format!("「{}」を選択した場合の表示", data.label))],
        ),
        tab_strip(data.value),
    ];
    children.extend(panel_row(data));
    div(vec![("class", "blocks-feature-tabs-panel-state")], children)
}

/// 基準形（R1158）: 見出し + 下線タブ（見た目のみ CSS で再現）+
/// テキスト/画像パネル。無 JS サイトでは切り替えができないため
/// （モジュール doc「無 JS での扱い」節）、[`PANELS`] の 4 件それぞれを
/// 選択状態違いの [`variant_state`] として縦に併記する。
fn variant_basic() -> Node {
    let mut children = vec![section_header(
        "機能紹介",
        "タブで切り替える機能セクション",
        "見出しの下にタブを並べ、選んだタブの内容だけを表示します。",
    )];
    children.extend(PANELS.iter().map(variant_state));
    div(
        vec![("class", "blocks-feature-tabs-panel-variant")],
        children,
    )
}

/// `feature-tabs-panel` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数（本イシューでは基準形 1 件のみを並べる。#2773 がこの配下へ
/// 追加の形の並記を続ける）。基準形自体は無 JS 対応のため、実物の
/// `tabs::tabs` は一切使わず静的なタブ列（[`variant_basic`] 参照）のみで
/// 構成する。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-feature-tabs-panel-layout")],
        vec![
            variant_label("下線タブ + テキスト/画像パネル（対応表 ID R1158 基準形）"),
            variant_basic(),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/feature-tabs-panel/",
    title: "feature-tabs-panel",
    category: BlockCategory::Feature,
    rust_source: "crates/docs-site/src/blocks/marketing/feature/feature_tabs_panel.rs",
    demo_class: "blocks-feature-tabs-panel",
    parts: &[
        Part {
            label: "Badge",
            path: "/themes/badge/",
        },
        Part {
            label: "Heading",
            path: "/themes/heading/",
        },
        Part {
            label: "Text",
            path: "/themes/text/",
        },
        Part {
            label: "Image",
            path: "/themes/image/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `feature_tabs_panel` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節）。素の `div`/`span` へのフックは
/// `.blocks-feature-tabs-panel-*` クラスセレクタのみで行い、他 block や
/// 部品の素のセレクタへ影響させない（モジュール doc「CSS フックの選び方」
/// 節参照）。
const LAYOUT_CSS: &str = "\
.blocks-feature-tabs-panel-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-10);\n}\n\
.blocks-feature-tabs-panel-variant {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n}\n\
.blocks-feature-tabs-panel-header {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n  align-items: start;\n  max-width: 40rem;\n}\n\
.blocks-feature-tabs-panel-list {\n  display: flex;\n  flex-wrap: nowrap;\n  overflow-x: auto;\n  overflow-y: hidden;\n  gap: var(--fandhe-space-4);\n  border-bottom: 1px solid var(--fandhe-color-border);\n}\n\
.blocks-feature-tabs-panel-trigger {\n  display: inline-flex;\n  align-items: center;\n  white-space: nowrap;\n  padding: var(--fandhe-space-2) var(--fandhe-space-1);\n  margin-bottom: -1px;\n  color: var(--fandhe-color-fg-muted);\n  border-bottom: 2px solid transparent;\n}\n\
.blocks-feature-tabs-panel-trigger[data-blocks-feature-tabs-panel-selected] {\n  color: var(--fandhe-color-fg);\n  font-weight: var(--fandhe-font-weight-medium);\n  border-bottom-color: var(--fandhe-color-accent);\n}\n\
.blocks-feature-tabs-panel-state {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-feature-tabs-panel-row {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n  padding-top: var(--fandhe-space-6);\n}\n\
.blocks-feature-tabs-panel-copy {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n  min-width: 0;\n}\n\
[data-scope=\"image\"][data-part=\"root\"][data-blocks-feature-tabs-panel-image] {\n  display: block;\n  width: 100%;\n}\n\
@media (min-width: 64rem) {\n  \
.blocks-feature-tabs-panel-row {\n    display: grid;\n    grid-template-columns: repeat(12, minmax(0, 1fr));\n    column-gap: var(--fandhe-space-10);\n    align-items: center;\n  }\n  \
.blocks-feature-tabs-panel-row > .blocks-feature-tabs-panel-copy {\n    grid-column: 1 / span 5;\n    grid-row: 1;\n  }\n  \
.blocks-feature-tabs-panel-row > .blocks-feature-tabs-panel-media {\n    grid-column: 7 / span 6;\n    grid-row: 1;\n  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// Demo が期待する 4 部品を出力すること、非対話制約（`<form>` 不在・
    /// `data:` URI 不在・`href="#"` 不在）を満たすことの単体回帰
    /// （`crates/docs-site/tests/blocks_contract.rs` の横断検査と重複し
    /// 過ぎない範囲での個別固定）。
    #[test]
    fn demo_composes_expected_parts_and_avoids_forms() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"badge\"",
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"image\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        for absent in ["<form", "src=\"data:", "href=\"#\""] {
            assert!(
                !html.contains(absent),
                "demo output should never contain {absent}"
            );
        }
    }

    /// Demo 出力が操作可能に見える要素（`<button`・`role="tab"`・
    /// `tabindex`・`aria-selected`）を一切含まないこと（Codex P1 指摘
    /// 2 回分の回帰テスト: docs サイトは JS ハイドレーションを行わないため、
    /// クリック・キーボード操作をしても実際には切り替わらないトリガー
    /// ボタンを出してはならない、モジュール doc「無 JS での扱い」節）。
    #[test]
    fn demo_output_has_no_interactive_looking_elements() {
        let html = render(&demo());
        for forbidden in ["<button", "role=\"tab\"", "tabindex", "aria-selected"] {
            assert!(
                !html.contains(forbidden),
                "demo output should never contain {forbidden}"
            );
        }
    }

    /// [`PANELS`] の 4 件すべてについて、静的タブ列（[`super::tab_strip`]）が
    /// 選択中の項目 1 件だけへ強調属性を付与し、パネル本文の見出しが
    /// 常に可視のまま静的 HTML に現れること。
    #[test]
    fn every_panel_state_highlights_exactly_one_trigger_and_stays_visible() {
        let html = render(&demo());
        assert_eq!(
            html.matches("data-blocks-feature-tabs-panel-selected")
                .count(),
            super::PANELS.len(),
            "expected exactly one selected trigger per panel state"
        );
        for panel in &super::PANELS {
            assert!(
                html.contains(panel.title),
                "expected panel title {:?} to appear in demo output",
                panel.title
            );
            assert!(
                html.contains(panel.label),
                "expected panel label {:?} to appear in demo output",
                panel.label
            );
        }
    }

    /// [`LAYOUT_CSS`] が想定するタブ列の横スクロール・選択中トリガーの
    /// 強調・lg ブレークポイントを持つこと。
    #[test]
    fn layout_css_declares_scrollable_tablist_and_lg_grid() {
        assert!(LAYOUT_CSS.contains("overflow-x: auto"));
        assert!(LAYOUT_CSS.contains("[data-blocks-feature-tabs-panel-selected]"));
        assert!(LAYOUT_CSS.contains("@media (min-width: 64rem)"));
        assert!(LAYOUT_CSS.contains("repeat(12, minmax(0, 1fr))"));
    }

    /// ルート class（`demo_class` とは別名）が `demo()` の出力へ実際に
    /// 現れること（既存 block と同じ Bugbot 教訓の固定）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-feature-tabs-panel-layout\""));
        assert_ne!(super::BLOCK.demo_class, "blocks-feature-tabs-panel-layout");
    }
}
