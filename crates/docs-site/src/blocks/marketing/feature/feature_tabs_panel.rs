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
//! 代替形・複数インスタンスの並記は後続の #2773 が [`tabs_panel`] を
//! 再利用して追加する。
//!
//! # 使用部品
//!
//! `badge` / `heading` / `text` / `tabs` / `image` の 5 部品を合成する
//! （[`BLOCK`] の `parts` に一致させる契約、`crates/docs-site/tests/
//! blocks_nav.rs`/`blocks_contract.rs` が検証する。実際に描画する部品だけを
//! `parts` に載せ、#2773 が追加する `card`/`icon`/`button`/`progress` は
//! そのイシューで `parts` へ追記する）。
//!
//! # 無 JS での扱い（初期タブ固定）
//!
//! docs サイトは JS ハイドレーションを行わないため、`tabs::tabs` が headless
//! 層のロジックに従い選択されていないパネルへ `hidden` を付与した状態で
//! 静的に描画する。初期タブ（[`PANELS`] の先頭）を選択済みとして固定し、
//! それ以外のタブは trigger だけが見える（`pricing_tiers_morph` と同型の
//! 判断）。
//!
//! # id 規約（重複 id 検知テストへの対応）
//!
//! `tabs::tabs` は `TabsProps.id` から `{id}-trigger-{value}` /
//! `{id}-content-{value}` を機械生成し `aria-controls`/`aria-labelledby` で
//! 相互参照するため、本 block は他の block と異なり id を持つ（`tabs` を
//! 使わない block の「id は一切使わない」方針の例外）。id の基底は
//! `blocks-feature-tabs-panel-<形の接尾辞>` に統一し、本イシューでは
//! `-basic` のみを使う。#2773 が追加するインスタンスは別接尾辞（`-pill`
//! 等）を使い、衝突を構造的に避ける。value は ASCII kebab-case とする。
//! `crates/docs-site/tests/blocks_contract.rs::
//! demo_output_has_no_dangling_aria_references_or_duplicate_ids` が id の
//! 重複・参照先欠落を検知する。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約と `tabs` の制約）
//!
//! `badge::badge`/`heading::heading`/`styled_text::text`/`image::image` は
//! いずれも `drop_class_attr` により呼び出し側 `attrs` の `class` を除去する
//! 契約を持つため、Demo 固有のスタイルフックは
//! `data-blocks-feature-tabs-panel-*` 属性で渡す。`tabs::tabs` は呼び出し側
//! から attrs を一切受け取らないため、tabs の各パーツへのスタイルは
//! [`LAYOUT_CSS`] 側でレイアウト root からの子孫セレクタ
//! （`[data-scope="tabs"][data-part="..."]`）として当てる:
//!
//! - `[data-part="list"]` に `overflow-x: auto` を与え、狭い幅でタブ列が
//!   はみ出してページ全体が横スクロールするのを list 自身の横スクロールへ
//!   閉じ込める（モバイルでのタブ列横スクロール）。`padding-bottom: 1px` は
//!   trigger の下線用 `margin-bottom: -1px` がスクロール領域の境界で
//!   クリップされる分を吸収する。
//! - `[data-part="trigger"]:focus-visible` の `outline-offset` を負値にし、
//!   スクロール領域の外側へ張り出すフォーカスリングがクリップされるのを
//!   防ぐ（内側に描く）。
//! - `[data-part="content"]` の `padding` を打ち消し、[`panel_row`] 側の
//!   余白と tabs recipe の content padding が二重に乗るのを防ぐ。
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
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::image::{self, AspectRatio, ImageFit, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::tabs::{
    self, ActivationMode, Orientation, TabItem, TabsProps, TabsVariant,
};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

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

/// [`PANELS`] から `tabs::tabs` の `items` を組み立てる（#2773 が variant を
/// 変えつつ再利用する共通ヘルパ）。
fn panel_items() -> Vec<TabItem<'static>> {
    PANELS
        .iter()
        .map(|data| TabItem {
            value: data.value,
            trigger: vec![text(data.label)],
            content: panel_row(data),
            disabled: false,
        })
        .collect()
}

/// `id`（呼び出し側が `blocks-feature-tabs-panel-<接尾辞>` の形で完全指定
/// する）・variant・選択中タブを引数に取る tabs 組み立てヘルパ（#2773 が
/// 形を追加する際に再利用する）。`id` を呼び出し側が組み立てる形にすることで
/// 「id の基底は常に `blocks-feature-tabs-panel-` で始まる」という
/// モジュール doc「id 規約」節の不変条件を、内部で接尾辞から静的文字列へ
/// 変換するテーブル（変換漏れがあっても素通りしてしまう）を持たずに
/// 呼び出し箇所ごとのリテラルとして機械的に確認できるようにする。
fn tabs_panel(id: &'static str, variant: TabsVariant, selected: &'static str) -> Node {
    tabs::tabs(
        variant,
        Size::Md,
        ColorPalette::Accent,
        &TabsProps {
            id,
            selected,
            orientation: Orientation::Horizontal,
            activation_mode: ActivationMode::Automatic,
            loop_focus: true,
            indicator: false,
        },
        panel_items(),
    )
}

/// 基準形（R1158）: 見出し + 下線タブ（[`TabsVariant::Line`]）+
/// テキスト/画像パネル。
fn variant_basic() -> Node {
    div(
        vec![("class", "blocks-feature-tabs-panel-variant")],
        vec![
            section_header(
                "機能紹介",
                "タブで切り替える機能セクション",
                "見出しの下にタブを並べ、選んだタブの内容だけを表示します。",
            ),
            tabs_panel(
                "blocks-feature-tabs-panel-basic",
                TabsVariant::Line,
                PANELS[0].value,
            ),
        ],
    )
}

/// `feature-tabs-panel` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数（本イシューでは基準形 1 件のみを並べる。#2773 がこの配下へ
/// 追加の形・複数インスタンスの並記を続ける）。
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
            label: "Tabs",
            path: "/themes/tabs/",
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
/// doc「block 固有 CSS の置き場」節）。素の `div` へのフックは
/// `.blocks-feature-tabs-panel-*` クラスセレクタ、`tabs` パーツへのフックは
/// `[data-scope="tabs"][data-part="..."]` の子孫セレクタで行い、他 block や
/// 部品の素のセレクタへ影響させない（モジュール doc「CSS フックの選び方」
/// 節参照）。
const LAYOUT_CSS: &str = "\
.blocks-feature-tabs-panel-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-10);\n}\n\
.blocks-feature-tabs-panel-variant {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n}\n\
.blocks-feature-tabs-panel-header {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n  align-items: start;\n  max-width: 40rem;\n}\n\
.blocks-feature-tabs-panel-layout [data-scope=\"tabs\"][data-part=\"list\"] {\n  overflow-x: auto;\n  overflow-y: hidden;\n  padding-bottom: 1px;\n}\n\
.blocks-feature-tabs-panel-layout [data-scope=\"tabs\"][data-part=\"trigger\"]:focus-visible {\n  outline-offset: calc(-1 * var(--fandhe-focus-ring-width, 2px));\n}\n\
.blocks-feature-tabs-panel-layout [data-scope=\"tabs\"][data-part=\"content\"] {\n  padding: 0;\n}\n\
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

    /// Demo が期待する 5 部品を出力すること、非対話制約（`<form>` 不在・
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
            "data-scope=\"tabs\"",
            "data-scope=\"image\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert!(html.contains("type=\"button\""));
        for absent in ["<form", "src=\"data:", "href=\"#\""] {
            assert!(
                !html.contains(absent),
                "demo output should never contain {absent}"
            );
        }
    }

    /// 選択中タブがちょうど 1 件で、残りは `hidden` を持つこと（無 JS
    /// 固定表示の不変条件）。
    #[test]
    fn exactly_one_trigger_is_active_and_others_are_hidden() {
        let html = render(&demo());
        assert_eq!(html.matches("aria-selected=\"true\"").count(), 1);
        assert_eq!(
            html.matches("data-part=\"trigger\"").count(),
            html.matches("aria-selected=\"false\"").count() + 1
        );
        assert_eq!(html.matches("data-part=\"content\"").count(), 4);
        assert_eq!(html.matches(" hidden").count(), 3);
    }

    /// id の接頭辞がすべて `blocks-feature-tabs-panel-` であること
    /// （重複 id 検知テストへの対応、モジュール doc「id 規約」節）。
    #[test]
    fn ids_use_the_expected_prefix() {
        let html = render(&demo());
        assert!(html.contains("id=\"blocks-feature-tabs-panel-basic-trigger-design\""));
        assert!(html.contains("id=\"blocks-feature-tabs-panel-basic-content-design\""));
    }

    /// [`LAYOUT_CSS`] が想定するタブ列の横スクロール・lg ブレークポイント・
    /// tabs パーツへの子孫セレクタを持つこと。
    #[test]
    fn layout_css_declares_scrollable_tablist_and_lg_grid() {
        assert!(LAYOUT_CSS.contains("overflow-x: auto"));
        assert!(LAYOUT_CSS.contains("@media (min-width: 64rem)"));
        assert!(LAYOUT_CSS.contains("[data-scope=\"tabs\"][data-part=\"list\"]"));
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
