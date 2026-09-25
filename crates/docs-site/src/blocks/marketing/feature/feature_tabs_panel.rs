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
//! # 無 JS での扱い（実物の `tabs` は 1 個のみ・残りは非対話プレビュー）
//!
//! docs サイトは JS ハイドレーションを行わないため、`tabs::tabs` は headless
//! 層のロジックに従い選択されていないパネルへ `hidden` を付与した状態で
//! 静的に描画する。当初は [`PANELS`] の要素数（4 件）ぶんの `tabs::tabs`
//! インスタンスを選択状態違いで縦に並べていた（`pricing_tiers_morph`/
//! `sidebar_07` と同型の対処）が、この構成は `role="tab"`/`type="button"`/
//! `tabindex` を持つ操作可能に見えるトリガーボタンを 4 インスタンス
//! （計 16 個）反復して出しながら、クリック・キーボード操作をしても実際
//! には一切切り替わらない（Codex P1 指摘、`docs/design/
//! docs-site-blocks-section.md` §19 参照）。これは UI のアクセシビリティ
//! 契約に反するため、実物の `tabs::tabs`（他の Themes 部品ページの Demo と
//! 同じ「静的プレビュー」慣習に従う、操作できないこと自体は無 JS サイト
//! 全体で共通の既知の制約）は [`PANELS`] の先頭（`design`）を選択済みとした
//! **1 個だけ**を描画する。残り 3 パネルの内容は、`tabs` コンポーネントを
//! 複製せず（`role="tab"`/`tabindex`/`<button>` を一切持たない）見出し
//! キャプション付きの非対話表示（[`panel_state_preview`]）として静的に
//! 併記し、全パネル本文が `hidden` を経由せず常に可視のまま静的 HTML に
//! 現れるようにする。
//!
//! # id 規約（重複 id 検知テストへの対応）
//!
//! `tabs::tabs` は `TabsProps.id` から `{id}-trigger-{value}` /
//! `{id}-content-{value}` を機械生成し `aria-controls`/`aria-labelledby` で
//! 相互参照するため、本 block は他の block と異なり id を持つ（`tabs` を
//! 使わない block の「id は一切使わない」方針の例外）。id の基底は
//! `blocks-feature-tabs-panel-<形の接尾辞>` に統一し、本イシューでは
//! `-basic` のみを使う。実物の `tabs::tabs` インスタンスは（上記「無 JS
//! での扱い」節のとおり）1 個だけのため、インスタンスごとの識別子の
//! テーブルは持たない（[`panel_state_preview`] の非対話プレビューは
//! `tabs` を使わないため id を持たない）。#2773 が追加するインスタンスは
//! 別接尾辞（`-pill` 等）を使い、衝突を構造的に避ける。value は ASCII
//! kebab-case とする。`crates/docs-site/tests/blocks_contract.rs::
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
//! - `[data-part="trigger"]` の `margin-bottom` を pre-styled-ui 既定の
//!   `-1px` から `-2px` へ上書きする（Bugbot Low 指摘の是正）。`list` の
//!   `padding-bottom: 1px`（直前の箇条書き）は `list` の下端（＝下線を
//!   重ねたい `border-bottom: 1px` の位置）を 1px 押し下げるため、
//!   `trigger` 側の重ね量を素の `-1px` のままにすると選択中トリガーの
//!   2px 下線が `list` の 1px 罫線と重ならず 3px の二重線に見えてしまう。
//!   `padding-bottom` が押し下げた 1px 分を追加で相殺し、選択中トリガーの
//!   下線を `list` の罫線へ正しく重ねる。
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

/// 残り 3 パネル（[`PANELS`] の先頭以外）を、`tabs::tabs` を複製せず
/// 見出しキャプション付きの非対話表示として静的に併記する（モジュール doc
/// 「無 JS での扱い」節参照。Codex P1 指摘の是正: 操作可能に見えて実際には
/// 切り替わらないトリガーボタン〔`role="tab"`/`type="button"`/`tabindex`〕
/// を反復して出さない）。`panel_row` を直接呼ぶだけで `tabs::tabs`/
/// `ANATOMY` を一切経由しないため、`role`/`tabindex`/`<button>` を持たない。
fn panel_state_preview(data: &PanelData) -> Node {
    let mut children = vec![styled_text::text(
        &TextProps {
            size: TextSize::Sm,
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![],
        vec![text(format!(
            "「{}」タブを選択した場合のプレビュー",
            data.label
        ))],
    )];
    children.extend(panel_row(data));
    div(
        vec![("class", "blocks-feature-tabs-panel-preview")],
        children,
    )
}

/// 基準形（R1158）: 見出し + 下線タブ（[`TabsVariant::Line`]）+
/// テキスト/画像パネル。docs サイトは JS ハイドレーションを行わないため
/// （モジュール doc「無 JS での扱い」節）、実物の `tabs::tabs`（[`PANELS`]
/// 先頭の `design` を選択済みとする 1 個だけ）を描画したあと、残り 3
/// パネルは [`panel_state_preview`] による非対話プレビューとして併記する。
/// これにより 4 パネルすべての本文が常に可視のまま静的 HTML に現れる。
fn variant_basic() -> Node {
    let mut children = vec![
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
    ];
    for panel in &PANELS[1..] {
        children.push(panel_state_preview(panel));
    }
    div(
        vec![("class", "blocks-feature-tabs-panel-variant")],
        children,
    )
}

/// `feature-tabs-panel` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数（本イシューでは基準形 1 件のみを並べる。#2773 がこの配下へ
/// 追加の形の並記を続ける）。基準形自体は無 JS 対応のため、実物の
/// `tabs::tabs` は 1 個のみで残りは非対話プレビュー（[`variant_basic`]
/// 参照）。
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
.blocks-feature-tabs-panel-layout [data-scope=\"tabs\"][data-part=\"trigger\"] {\n  margin-bottom: -2px;\n}\n\
.blocks-feature-tabs-panel-layout [data-scope=\"tabs\"][data-part=\"trigger\"]:focus-visible {\n  outline-offset: calc(-1 * var(--fandhe-focus-ring-width, 2px));\n}\n\
.blocks-feature-tabs-panel-layout [data-scope=\"tabs\"][data-part=\"content\"] {\n  padding: 0;\n}\n\
.blocks-feature-tabs-panel-row {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n  padding-top: var(--fandhe-space-6);\n}\n\
.blocks-feature-tabs-panel-copy {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n  min-width: 0;\n}\n\
.blocks-feature-tabs-panel-preview {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-feature-tabs-panel-preview .blocks-feature-tabs-panel-row {\n  padding-top: 0;\n}\n\
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

    /// 実物の `tabs::tabs` インスタンスは 1 個だけで、選択中タブがちょうど
    /// 1 件・残り 3 件は `hidden` を持つこと（Codex P1 是正: 操作可能に見え
    /// て実際には切り替わらないトリガーボタンを複数インスタンス分反復して
    /// 出さない、モジュール doc「無 JS での扱い」節）。
    #[test]
    fn the_single_tabs_instance_has_exactly_one_active_trigger_and_hides_the_rest() {
        let html = render(&demo());
        // 実物の tabs インスタンスは 1 個のみ・4 trigger、選択中は 1 件。
        assert_eq!(html.matches("data-part=\"trigger\"").count(), 4);
        assert_eq!(html.matches("aria-selected=\"true\"").count(), 1);
        assert_eq!(html.matches("aria-selected=\"false\"").count(), 3);
        // content も 4 個のみ、非選択の 3 個が hidden。
        assert_eq!(html.matches("data-part=\"content\"").count(), 4);
        assert_eq!(html.matches(" hidden").count(), 3);
    }

    /// 実物の `tabs::tabs` は先頭パネル（`design`）1 個に限られ、残り 3
    /// パネルは `role="tab"`/`type="button"`/`tabindex` を持たない非対話
    /// プレビュー（[`super::panel_state_preview`]）として現れること
    /// （Codex P1 是正の回帰テスト: 操作できないトリガーボタンの反復を
    /// 防止する）。
    #[test]
    fn only_the_first_panel_is_a_real_tabs_instance_and_the_rest_are_non_interactive() {
        let html = render(&demo());
        // role="tab" は実物のインスタンス分（4 trigger）だけ現れる。
        assert_eq!(html.matches("role=\"tab\"").count(), 4);
        assert_eq!(html.matches("role=\"tabpanel\"").count(), 4);
        // 非対話プレビューは <button> を一切持ち込まない（実物の tabs
        // インスタンスが出す trigger 分の 4 個だけが `<button` として
        // 現れることを確認する。3 個のプレビューが追加の `<button` を
        // 持ち込んでいればここで検知される）。
        assert_eq!(
            html.matches("class=\"blocks-feature-tabs-panel-preview\"")
                .count(),
            3
        );
        assert_eq!(html.matches("<button").count(), 4);
    }

    /// 全パネルの本文見出しが、`hidden` を経由せず常に可視状態として静的
    /// HTML に現れること（codex P1 指摘の回帰テスト: 選択中以外のパネル
    /// 本文が静的ページから一切読めなくなっていた不具合の再発防止）。
    #[test]
    fn every_panel_title_is_always_visible() {
        let html = render(&demo());
        // 先頭パネル（design）は実物の tabs インスタンスで選択済み。
        let first = &super::PANELS[0];
        let content_marker = format!(
            "id=\"blocks-feature-tabs-panel-basic-content-{}\"",
            first.value
        );
        let content_start = html
            .find(&content_marker)
            .unwrap_or_else(|| panic!("missing content element for {}", first.value));
        let tag_end = html[content_start..]
            .find('>')
            .map(|i| content_start + i)
            .unwrap_or(html.len());
        assert!(
            !html[content_start..tag_end].contains("hidden"),
            "expected a visible (non-hidden) content element for panel {}",
            first.value
        );
        for panel in &super::PANELS {
            assert!(
                html.contains(panel.title),
                "expected panel title {:?} to appear in demo output",
                panel.title
            );
        }
    }

    /// id の接頭辞が `blocks-feature-tabs-panel-basic-` であり、実物の
    /// `tabs::tabs` インスタンス（先頭パネル分）の trigger/content id が
    /// 重複なく現れること（重複 id 検知テストへの対応、モジュール doc
    /// 「id 規約」節）。
    #[test]
    fn ids_use_the_expected_prefix_for_the_single_instance() {
        let html = render(&demo());
        for panel in &super::PANELS {
            assert!(html.contains(&format!(
                "id=\"blocks-feature-tabs-panel-basic-trigger-{}\"",
                panel.value
            )));
            assert!(html.contains(&format!(
                "id=\"blocks-feature-tabs-panel-basic-content-{}\"",
                panel.value
            )));
        }
    }

    /// [`LAYOUT_CSS`] が想定するタブ列の横スクロール・下線の重ね合わせ
    /// 補正（Bugbot Low 是正）・lg ブレークポイント・tabs パーツへの子孫
    /// セレクタを持つこと。
    #[test]
    fn layout_css_declares_scrollable_tablist_and_lg_grid() {
        assert!(LAYOUT_CSS.contains("overflow-x: auto"));
        assert!(LAYOUT_CSS.contains("margin-bottom: -2px"));
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
