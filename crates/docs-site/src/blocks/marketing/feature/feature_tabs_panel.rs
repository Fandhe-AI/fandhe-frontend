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
//! # 無 JS での扱い（全パネルを可視の複数インスタンスとして併記）
//!
//! docs サイトは JS ハイドレーションを行わないため、`tabs::tabs` は headless
//! 層のロジックに従い選択されていないパネルへ `hidden` を付与した状態で
//! 静的に描画する。単一インスタンスで初期タブだけを固定表示すると、残り
//! 3 パネルの内容は静的ページ上で一切読めなくなる（クリック・キーボード
//! 操作をしても JS 未配線のため表示が変わらない）。これを避けるため
//! `pricing_tiers_morph`（2 状態: 月額/年額）・`sidebar_07`
//! （2 状態: expanded/collapsed）と同型の対処として、[`PANELS`] の要素数
//! （4 件）ぶんの `tabs::tabs` インスタンスを縦に並べ、インスタンスごとに
//! 異なる `selected` を指定する。これにより各パネルの本文は必ずいずれか
//! 1 個のインスタンスで `hidden` なしの可視状態として静的 HTML に現れる
//! （tab バー自体は 4 回繰り返されるが、trigger 集合はどのインスタンスも
//! 同一であり、キャプション（[`variant_label`] 相当の短文）で「どのタブを
//! 選択した状態か」を示す）。
//!
//! # id 規約（重複 id 検知テストへの対応）
//!
//! `tabs::tabs` は `TabsProps.id` から `{id}-trigger-{value}` /
//! `{id}-content-{value}` を機械生成し `aria-controls`/`aria-labelledby` で
//! 相互参照するため、本 block は他の block と異なり id を持つ（`tabs` を
//! 使わない block の「id は一切使わない」方針の例外）。id の基底は
//! `blocks-feature-tabs-panel-<形の接尾辞>` に統一し、本イシューでは
//! `-basic` のみを使う。上記の複数インスタンス化に伴い、`-basic` 接尾辞の
//! 中でさらにインスタンスごとの識別子（[`PanelData::instance_id`]、
//! `-basic-design`/`-basic-integration`/`-basic-operations`/
//! `-basic-analytics` の 4 個。`id: &'static str` のため呼び出し箇所ごとの
//! リテラルとして持ち、`format!` によるテーブル変換は行わない）を持つ。
//! #2773 が追加するインスタンスは別接尾辞（`-pill` 等）を使い、衝突を
//! 構造的に避ける。value は ASCII kebab-case とする。
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

/// 基準形（R1158）1 タブ分のデータ（架空文言）。`instance_id` は本パネルを
/// 選択済みとして描画する `tabs::tabs` インスタンスの `TabsProps.id`
/// （モジュール doc「無 JS での扱い」「id 規約」節参照。`-basic` 接尾辞の
/// 中でパネルごとに一意な識別子を持つ）。
struct PanelData {
    value: &'static str,
    label: &'static str,
    title: &'static str,
    body: &'static str,
    image_src: &'static str,
    instance_id: &'static str,
}

/// 基準形の 4 タブ分（#2773 が追加する形もこの配列を再利用できる）。
const PANELS: [PanelData; 4] = [
    PanelData {
        value: "design",
        label: "設計",
        title: "型で不変条件を保証する設計",
        body: "コンポーネント境界と状態遷移を型で表現し、実行時ではなくコンパイル時に誤りを検出します。",
        image_src: dummy_assets::SCREENSHOT_SRC,
        instance_id: "blocks-feature-tabs-panel-basic-design",
    },
    PanelData {
        value: "integration",
        label: "連携",
        title: "既存システムへの段階的な組み込み",
        body: "部分埋め込みからフル機能構成まで、必要な範囲だけを選んで既存ページへ組み込めます。",
        image_src: dummy_assets::PRODUCT_SRC,
        instance_id: "blocks-feature-tabs-panel-basic-integration",
    },
    PanelData {
        value: "operations",
        label: "運用",
        title: "単一実行ファイルでの安定運用",
        body: "サーバーとアセットをひとまとめにし、Docker イメージ 1 枚で決定的にデプロイできます。",
        image_src: dummy_assets::BACKGROUND_SRC,
        instance_id: "blocks-feature-tabs-panel-basic-operations",
    },
    PanelData {
        value: "analytics",
        label: "分析",
        title: "ビルド成果物の可視化",
        body: "依存グラフとバンドルサイズを継続的に計測し、変化を CI 上で追跡できます。",
        image_src: dummy_assets::LOGO_SRC,
        instance_id: "blocks-feature-tabs-panel-basic-analytics",
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
/// テキスト/画像パネル。docs サイトは JS ハイドレーションを行わないため
/// （モジュール doc「無 JS での扱い」節）、[`PANELS`] の要素数ぶんの
/// `tabs::tabs` インスタンスをキャプション付きで縦に並べ、パネルごとに
/// 異なる `selected` を指定する。これにより 4 パネルすべての本文が
/// `hidden` なしの可視状態でいずれかのインスタンスに現れる
/// （`pricing_tiers_morph`/`sidebar_07` と同型の対処）。
fn variant_basic() -> Node {
    let mut children = vec![section_header(
        "機能紹介",
        "タブで切り替える機能セクション",
        "見出しの下にタブを並べ、選んだタブの内容だけを表示します。",
    )];
    for panel in &PANELS {
        children.push(styled_text::text(
            &TextProps {
                size: TextSize::Sm,
                variant: TextVariant::Muted,
                ..TextProps::default()
            },
            vec![],
            vec![text(format!("「{}」タブを選択した状態", panel.label))],
        ));
        children.push(tabs_panel(
            panel.instance_id,
            TabsVariant::Line,
            panel.value,
        ));
    }
    div(
        vec![("class", "blocks-feature-tabs-panel-variant")],
        children,
    )
}

/// `feature-tabs-panel` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数（本イシューでは基準形 1 件のみを並べる。#2773 がこの配下へ
/// 追加の形の並記を続ける）。基準形自体は無 JS 対応のため 4 インスタンス
/// （[`variant_basic`] 参照）を内包する。
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

    /// 4 インスタンス（[`super::PANELS`] の要素数ぶん）それぞれで選択中
    /// タブがちょうど 1 件・残り 3 件は `hidden` を持つこと（複数インスタンス
    /// 併記による無 JS 対応の不変条件、モジュール doc「無 JS での扱い」節）。
    #[test]
    fn each_instance_has_exactly_one_active_trigger_and_hides_the_rest() {
        let html = render(&demo());
        // 4 インスタンス × 4 trigger = 16、選択中は各インスタンス 1 件で計 4。
        assert_eq!(html.matches("data-part=\"trigger\"").count(), 16);
        assert_eq!(html.matches("aria-selected=\"true\"").count(), 4);
        assert_eq!(html.matches("aria-selected=\"false\"").count(), 12);
        // 4 インスタンス × 4 content = 16、非選択は各インスタンス 3 件で計 12。
        assert_eq!(html.matches("data-part=\"content\"").count(), 16);
        assert_eq!(html.matches(" hidden").count(), 12);
    }

    /// 全パネルの本文見出しが、少なくとも 1 インスタンスでは `hidden`
    /// なしの可視状態として静的 HTML に現れること（codex P1 指摘の回帰
    /// テスト: 単一インスタンス固定表示だと選択中以外のパネル本文が
    /// 静的ページから一切読めなくなっていた不具合の再発防止）。
    #[test]
    fn every_panel_title_is_visible_in_at_least_one_instance() {
        let html = render(&demo());
        for panel in &super::PANELS {
            let content_marker = format!("id=\"{}-content-{}\"", panel.instance_id, panel.value);
            let content_start = html
                .find(&content_marker)
                .unwrap_or_else(|| panic!("missing content element for {}", panel.value));
            // 当該 content 要素の開始タグ内（次の `>` まで）に hidden が
            // 無いことを確認する（同じタグ内の他属性を誤検知しないため、
            // タグの範囲だけを見る）。
            let tag_end = html[content_start..]
                .find('>')
                .map(|i| content_start + i)
                .unwrap_or(html.len());
            assert!(
                !html[content_start..tag_end].contains("hidden"),
                "expected a visible (non-hidden) content element for panel {}",
                panel.value
            );
            assert!(
                html.contains(panel.title),
                "expected panel title {:?} to appear in demo output",
                panel.title
            );
        }
    }

    /// id の接頭辞がすべて `blocks-feature-tabs-panel-basic-` であり、
    /// 4 インスタンス分の識別子（[`super::PanelData::instance_id`]）が
    /// 重複なく現れること（重複 id 検知テストへの対応、モジュール doc
    /// 「id 規約」節）。
    #[test]
    fn ids_use_the_expected_prefix_for_every_instance() {
        let html = render(&demo());
        for panel in &super::PANELS {
            assert!(html.contains(&format!(
                "id=\"{}-trigger-{}\"",
                panel.instance_id, panel.value
            )));
            assert!(html.contains(&format!(
                "id=\"{}-content-{}\"",
                panel.instance_id, panel.value
            )));
        }
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
