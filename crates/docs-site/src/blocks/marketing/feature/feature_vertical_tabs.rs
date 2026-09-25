//! `feature-vertical-tabs` block（イシュー #2775。親 #2774「Blocks 目的別
//! パーツ拡充ツリー」配下、規模 L のため #2775（本イシュー、骨格と主要
//! 領域）/ #2776（残り・仕上げ）の 2 sub-issue へ分割済み）。左列に縦に
//! 並んだ feature タブ、右列に選択中 feature の詳細（見出し・チェック付き
//! 機能一覧・画像）を置く合成例。対応表 ID は R0480（主参照。他に集約元は
//! なく「集約元の差分」は該当しない）のみを記す（取得手段・ファイル名・
//! 内部コンポーネント識別子は記載しない）。
//!
//! # #2775 と #2776 の分担
//!
//! 本イシュー（#2775）では骨格と主要領域を実装する: レイアウト root・
//! セクション見出し（`heading` H3 + `text` リード文）・4 タブの
//! [`vertical_tabs`]（先頭タブ選択済み・[`Orientation::Vertical`]）・各
//! trigger のタイトル + 短い説明・各パネルの見出し（`heading` H4）+
//! チェック付き機能一覧（`icon` + タイトル + 説明）+ 画像（`image`）。これで
//! [`BLOCK`] の `parts` が申告する Heading / Text / Tabs / Image / Icon の
//! 5 部品すべてが実際に描画される。
//!
//! 後続の #2776 では次を扱う: 状態違いの並記（2 件目のタブを選択した別
//! インスタンス）・trigger 先頭のアイコン等の仕上げの装飾・画像主体の別
//! パネル形・原稿「原案差分メモ」節の本記述（本イシューでは暫定版のみ）。
//!
//! # 使用部品
//!
//! `heading` / `text` / `tabs` / `image` / `icon` の 5 部品を合成する
//! （[`BLOCK`] の `parts` に一致させる契約、`crates/docs-site/tests/
//! blocks_nav.rs`/`blocks_contract.rs` が検証する）。新規 UI 部品は追加
//! しない。
//!
//! # 無 JS での扱い
//!
//! docs サイトは JS ハイドレーションを行わない（`crates/docs-site/tests/
//! no_js_contract.rs`）ため、先頭タブ（`build`）を選択済みの固定状態で
//! 描画する。非選択パネルは headless `tabs` が付与する `hidden` 属性で
//! 隠れる（`crates/headless-ui/src/tabs.rs`）。
//!
//! # id 規約
//!
//! 基底 id は `blocks-feature-vertical-tabs-<接尾辞>` とする。本イシューは
//! `-basic`（唯一のインスタンス）のみを持つ。
//!
//! # `Orientation::Vertical` を採用する理由（参照元の「Horizontal + 見た目
//! だけ CSS」は採らない）
//!
//! `fandhe_frontend_pre_styled_ui::tabs` の recipe（イシュー #1542/#2039）は
//! `data-orientation="vertical"` に対して root の `display: flex`・list の
//! 縦積み + `border-inline-end`・trigger の `border-inline-end` + 選択中の
//! 強調線・content の `flex: 1` をすでに持つ。lg（64rem）以上の主表示は
//! この recipe だけで賄えるうえ、`aria-orientation="vertical"` が実際の
//! レイアウトと一致する意味論として正しい。**lg 未満だけ** [`LAYOUT_CSS`]
//! が横並びへ上書きする（下記「レスポンシブ」節）。
//!
//! # レスポンシブ（64rem をブレークポイントとする理由・`aria-orientation`
//! のトレードオフ）
//!
//! `< 64rem`（lg 未満）はタブ列をパネルの上へ移して横並び（横スクロール）
//! にする。`>= 64rem` で左の縦タブ列 + 右のパネルの 2 列へ切り替える。
//! テーマの breakpoint トークンは `@media` 条件式の中では解決できない
//! （CSS custom property は宣言側でのみ有効）ため、
//! [`fandhe_frontend_pre_styled_ui::recipe::Breakpoint`] の `Lg`（1024px =
//! 64rem）と一致するリテラル値 `63.99rem`/`64rem` を [`LAYOUT_CSS`] へ直書き
//! する（`feature_expand`/`feature_split_list_image` と同じ判断）。
//!
//! **トレードオフ**: lg 未満はタブが横に並ぶが `aria-orientation` は
//! `"vertical"` のまま残る。hydration されたページでは矢印キーの移動軸
//! （上下）と見た目の軸（左右）が食い違うが、docs サイトは JS
//! ハイドレーションを一切行わないため実害はない。
//!
//! # trigger 内は phrasing content だけで組む
//!
//! headless の trigger は `<button type="button">` で、許される内容は
//! phrasing content だけである（`heading::heading` の `<h*>` や
//! `styled_text::text` の `<p>` は trigger の中では使えない）。[`trigger_body`]
//! は `span[data-blocks-feature-vertical-tabs-trigger-body]` の中に、
//! タイトル用の `span[data-blocks-feature-vertical-tabs-trigger-title]` と
//! 説明用の `span[data-blocks-feature-vertical-tabs-trigger-desc]` を置き、
//! core の `span`/`text` のみで組む。trigger の base 規則が持つ
//! `white-space: nowrap` を [`LAYOUT_CSS`] で `normal` へ上書きし、`gap` +
//! `flex-direction: column` で縦に積む。lg 未満の横並び時は、trigger 幅の
//! 肥大化を避けるため説明文（`trigger-desc`）を非表示にする（タイトルだけ
//! のタブになる）。
//!
//! # CSS フックの選び方・詳細度の方針
//!
//! `heading`/`text`/`image`/`icon` はいずれも `drop_class_attr` により
//! 呼び出し側 `attrs` の `class` を黙って除去する契約を持つため、Demo
//! 固有のスタイルフックは `data-blocks-feature-vertical-tabs-*` 属性で
//! 渡す。`tabs`（[`fandhe_frontend_pre_styled_ui::tabs::tabs`]）は root への
//! attrs 注入点を持たないため、レイアウト root の class
//! （[`Block::demo_class`] とは別名の `blocks-feature-vertical-tabs-layout`）
//! を起点にした子孫セレクタ（`.blocks-feature-vertical-tabs-layout
//! [data-scope="tabs"]...`）で上書きする。recipe の
//! `[data-scope][data-part][data-orientation]` 系規則（詳細度 (0,3,0)）・
//! `[data-scope][data-part][data-state][data-orientation]`（(0,4,0)）に
//! 確実に勝つため、上書きは子孫セレクタで 1 クラス分の詳細度を追加する
//! （それぞれ (0,4,0)・(0,5,0) になる）。`image`（recipe 詳細度 (0,2,0)）
//! への上書きも同様に `[data-scope="image"][data-part="root"][data-blocks-
//! feature-vertical-tabs-image]` の 3 セレクタ構成（(0,3,0)）で行う
//! （`feature_split_list_image`/`feature_image_cards` と同型の判断）。
//!
//! # `text` の名前衝突
//!
//! `fandhe_frontend_pre_styled_ui::text::text` と
//! `fandhe_frontend_core::text` が同名のため、styled 側を `styled_text`
//! として取り込む（`crate::blocks` 内の他 block と同じ回避方法）。
//!
//! # 自作幾何アイコン
//!
//! lucide 等の既存アイコンセットの path を複製しないため、
//! `feature_split_list_image::geo_icon` と同型の自作ヘルパ [`check_icon`]
//! （チェックマーク、`path` へ `fill="none"` + `stroke="currentColor"` を
//! 明示し `icon::icon` の `<svg>` 側が固定で持つ `fill="currentColor"`
//! （塗り面）を上書きして線画として描画する）で描く。隣に可視テキストが
//! あるため装飾扱い（[`IconProps::label`] は `None` のまま、
//! `aria-hidden="true"`）とする。
//!
//! # alt を空文字列にする理由
//!
//! いずれの画像も隣接する見出し・機能一覧で内容が伝わる装飾用途のため、
//! `alt=""` にする（`feature_alternating_rows`/`feature_split_list_image`
//! の前例と同じ判断）。
//!
//! # `<form>` を持たない・データ取得/送信を行わない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である。trigger は headless 由来の `type="button"` で送信先を持たない。
//! 文言はすべて架空のもの（実企業名・実クレデンシャル・PII を含まない）。
//! 画像は [`crate::blocks::dummy_assets`] の各定数（いずれもビルド時生成の
//! プレースホルダー SVG）を使い分け、`alt=""` で出力する。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, span, text as core_text, Node};
use fandhe_frontend_pre_styled_ui::heading::{
    self, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::image::{self, AspectRatio, ImageFit, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::tabs::{
    self, ActivationMode, Orientation, TabItem, TabsProps, TabsVariant,
};
use fandhe_frontend_pre_styled_ui::text::{
    self as styled_text, TextProps, TextSize, TextVariant, TextWeight,
};
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// チェックマークの自作幾何アイコン（機能一覧の各項目に添える、
/// `feature_split_list_image::geo_icon` と同型の判断）。
fn check_icon() -> Node {
    icon(
        &IconProps {
            size: Size::Sm,
            ..IconProps::default()
        },
        vec![],
        vec![fandhe_frontend_core::el(
            "path",
            vec![
                ("d", "M5 12l4 4L19 7"),
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

/// 機能一覧 1 項目分の架空データ。
struct DetailPoint {
    title: &'static str,
    body: &'static str,
}

/// タブ 1 枚分の架空データ（trigger のタイトル/説明 + パネルの見出し/機能
/// 一覧 3 件/画像）。
struct FeatureTab {
    /// タブ識別 value（ASCII kebab-case）。
    value: &'static str,
    /// trigger タイトル。
    title: &'static str,
    /// trigger の短い説明。
    summary: &'static str,
    /// パネル見出し。
    panel_title: &'static str,
    points: [DetailPoint; 3],
    image_src: &'static str,
}

/// 4 タブ分のデータ（画像はタブごとに異なる `dummy_assets` 定数を割り当てる）。
const FEATURES: [FeatureTab; 4] = [
    FeatureTab {
        value: "build",
        title: "ビルド",
        summary: "型で表現された構造から静的ファイルを組み立てます。",
        panel_title: "決定的なビルド",
        points: [
            DetailPoint {
                title: "外部依存ゼロの描画コア",
                body: "描画コアは外部クレートに依存しません。",
            },
            DetailPoint {
                title: "型で表現するスロットと props",
                body: "コンポーネントの構造は Rust の型で表現されます。",
            },
            DetailPoint {
                title: "決定的な出力",
                body: "同じ入力からは常に同じ静的ファイルを生成します。",
            },
        ],
        image_src: dummy_assets::PRODUCT_SRC,
    },
    FeatureTab {
        value: "deploy",
        title: "デプロイ",
        summary: "単一実行ファイルへまとめて配布できます。",
        panel_title: "単一バイナリ配布",
        points: [
            DetailPoint {
                title: "SSR/SSG を単一実行ファイルへ",
                body: "サーバー機能を単一バイナリへまとめられます。",
            },
            DetailPoint {
                title: "Docker 想定の配布形態",
                body: "コンテナイメージへそのまま組み込めます。",
            },
            DetailPoint {
                title: "オフライン決定性",
                body: "配布物は同一構成から常に同じ内容になります。",
            },
        ],
        image_src: dummy_assets::SCREENSHOT_SRC,
    },
    FeatureTab {
        value: "observe",
        title: "観測",
        summary: "機械検証可能な構成で挙動を追跡します。",
        panel_title: "機械検証可能な構成",
        points: [
            DetailPoint {
                title: "構造マニフェストによる検証",
                body: "依存関係を構造マニフェストが機械検証します。",
            },
            DetailPoint {
                title: "無 JS の静的表示",
                body: "JS ハイドレーションを行わない決定的な表示です。",
            },
            DetailPoint {
                title: "依存グラフ上限の遵守",
                body: "標準構成の依存パッケージ数は上限内に収めます。",
            },
        ],
        image_src: dummy_assets::BACKGROUND_SRC,
    },
    FeatureTab {
        value: "secure",
        title: "保護",
        summary: "既定エスケープと限定された unsafe 境界で守ります。",
        panel_title: "既定エスケープと安全な境界",
        points: [
            DetailPoint {
                title: "既定エスケープ",
                body: "テキスト補間は既定でエスケープされます。",
            },
            DetailPoint {
                title: "限定された unsafe 境界",
                body: "描画コア・状態管理コアでは unsafe を使用しません。",
            },
            DetailPoint {
                title: "明示的なオプトイン API",
                body: "エスケープの迂回経路は明示的な API に限られます。",
            },
        ],
        image_src: dummy_assets::LOGO_SRC,
    },
];

/// セクション見出し（heading H3 + リード文）を組み立てる。ページ側が
/// `## Demo` として `h2` を出すため H3 にする（`feature_split_list_image`
/// と同じ判断）。
fn section_header() -> Node {
    div(
        vec![("class", "blocks-feature-vertical-tabs-header")],
        vec![
            heading::heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    weight: HeadingWeight::Bold,
                },
                vec![],
                vec![core_text("機能を切り替えて確認する")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![("data-blocks-feature-vertical-tabs-lead", "")],
                vec![core_text(
                    "左のタブを選ぶと、対応する機能の詳細が右側に表示されます。",
                )],
            ),
        ],
    )
}

/// trigger の中身（span のみ、phrasing content 制約に従う。モジュール doc
/// 「trigger 内は phrasing content だけで組む」節参照）。
fn trigger_body(tab: &FeatureTab) -> Vec<Node> {
    vec![span(
        vec![("data-blocks-feature-vertical-tabs-trigger-body", "")],
        vec![
            span(
                vec![("data-blocks-feature-vertical-tabs-trigger-title", "")],
                vec![core_text(tab.title)],
            ),
            span(
                vec![("data-blocks-feature-vertical-tabs-trigger-desc", "")],
                vec![core_text(tab.summary)],
            ),
        ],
    )]
}

/// 機能一覧 1 項目（チェックアイコン + タイトル + 説明）。
fn detail_point(point: &DetailPoint) -> Node {
    div(
        vec![("class", "blocks-feature-vertical-tabs-point")],
        vec![
            check_icon(),
            div(
                vec![("class", "blocks-feature-vertical-tabs-point-text")],
                vec![
                    styled_text::text(
                        &TextProps {
                            weight: TextWeight::Bold,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![core_text(point.title)],
                    ),
                    styled_text::text(
                        &TextProps {
                            size: TextSize::Sm,
                            variant: TextVariant::Muted,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![core_text(point.body)],
                    ),
                ],
            ),
        ],
    )
}

/// パネル（content）の中身（見出し H4 + 機能一覧 + 画像。DOM 順は常に
/// 「見出し → 一覧 → 画像」で固定し `order` は使わない）。
fn panel_body(tab: &FeatureTab) -> Vec<Node> {
    vec![
        heading::heading(
            HeadingLevel::H4,
            &HeadingProps {
                size: HeadingSize::Lg,
                weight: HeadingWeight::Semibold,
            },
            vec![],
            vec![core_text(tab.panel_title)],
        ),
        div(
            vec![("class", "blocks-feature-vertical-tabs-points")],
            tab.points.iter().map(detail_point).collect(),
        ),
        image::image(
            &ImageProps {
                fit: ImageFit::Cover,
                aspect_ratio: AspectRatio::Landscape,
                shape: ImageShape::Rounded,
                ..ImageProps::new(tab.image_src, "")
            },
            vec![("data-blocks-feature-vertical-tabs-image", "")],
        ),
    ]
}

/// 縦並び Tabs 本体を組み立てる（#2776 が並記のために再利用できる共通
/// ヘルパ。`id` は呼び出し側がリテラルで完全指定する）。
fn vertical_tabs(id: &'static str, selected: &'static str) -> Node {
    let items: Vec<TabItem<'static>> = FEATURES
        .iter()
        .map(|tab| TabItem {
            value: tab.value,
            trigger: trigger_body(tab),
            content: panel_body(tab),
            disabled: false,
        })
        .collect();
    let props = TabsProps {
        id,
        selected,
        orientation: Orientation::Vertical,
        activation_mode: ActivationMode::Automatic,
        loop_focus: true,
        indicator: false,
    };
    tabs::tabs(
        TabsVariant::Line,
        Size::Md,
        ColorPalette::Accent,
        &props,
        items,
    )
}

/// `feature-vertical-tabs` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数（他 block と同じ状態を持たない設計）。
#[must_use]
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-feature-vertical-tabs-layout")],
        vec![
            section_header(),
            vertical_tabs("blocks-feature-vertical-tabs-basic", "build"),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/feature-vertical-tabs/",
    title: "feature-vertical-tabs",
    category: BlockCategory::Feature,
    rust_source: "crates/docs-site/src/blocks/marketing/feature/feature_vertical_tabs.rs",
    demo_class: "blocks-feature-vertical-tabs",
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
            label: "Tabs",
            path: "/themes/tabs/",
        },
        Part {
            label: "Image",
            path: "/themes/image/",
        },
        Part {
            label: "Icon",
            path: "/themes/icon/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `feature_vertical_tabs` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節）。セレクタは
/// `.blocks-feature-vertical-tabs-*` と `[data-blocks-feature-vertical-tabs-*]`
/// に加え、`tabs`（root attrs 注入点を持たない）への上書きに限り
/// `.blocks-feature-vertical-tabs-layout [data-scope="tabs"]...` の子孫
/// セレクタを用いる（モジュール doc「CSS フックの選び方・詳細度の方針」
/// 節）。他 block や部品の素のセレクタへは影響させない。
const LAYOUT_CSS: &str = "\
.blocks-feature-vertical-tabs-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n}\n\
.blocks-feature-vertical-tabs-header {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n}\n\
[data-scope=\"text\"][data-part=\"root\"][data-blocks-feature-vertical-tabs-lead] {\n  margin: 0;\n}\n\
.blocks-feature-vertical-tabs-layout [data-scope=\"tabs\"][data-part=\"root\"] {\n  display: flex;\n  gap: var(--fandhe-space-8);\n  align-items: flex-start;\n}\n\
.blocks-feature-vertical-tabs-layout [data-scope=\"tabs\"][data-part=\"list\"][data-orientation=\"vertical\"] {\n  flex: 0 0 auto;\n  max-width: 20rem;\n}\n\
.blocks-feature-vertical-tabs-layout [data-scope=\"tabs\"][data-part=\"content\"][data-orientation=\"vertical\"] {\n  flex: 1;\n  min-width: 0;\n}\n\
.blocks-feature-vertical-tabs-layout [data-scope=\"tabs\"][data-part=\"trigger\"][data-orientation=\"vertical\"] {\n  white-space: normal;\n  text-align: start;\n  align-items: flex-start;\n}\n\
[data-blocks-feature-vertical-tabs-trigger-body] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1);\n}\n\
[data-blocks-feature-vertical-tabs-trigger-title] {\n  font-weight: var(--fandhe-font-font-weight-semibold);\n}\n\
[data-blocks-feature-vertical-tabs-trigger-desc] {\n  color: var(--fandhe-color-fg-muted);\n  font-size: var(--fandhe-font-font-size-sm);\n}\n\
.blocks-feature-vertical-tabs-points {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n  margin-block: var(--fandhe-space-4) 0;\n}\n\
.blocks-feature-vertical-tabs-point {\n  display: flex;\n  align-items: flex-start;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-feature-vertical-tabs-point-text {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1);\n}\n\
[data-scope=\"image\"][data-part=\"root\"][data-blocks-feature-vertical-tabs-image] {\n  display: block;\n  width: 100%;\n  margin-top: var(--fandhe-space-4);\n}\n\
@media (max-width: 63.99rem) {\n  \
.blocks-feature-vertical-tabs-layout [data-scope=\"tabs\"][data-part=\"root\"] {\n    flex-direction: column;\n  }\n  \
.blocks-feature-vertical-tabs-layout [data-scope=\"tabs\"][data-part=\"list\"][data-orientation=\"vertical\"] {\n    flex-direction: row;\n    max-width: none;\n    overflow-x: auto;\n    overflow-y: hidden;\n    border-inline-end: 0;\n    border-bottom: 1px solid var(--fandhe-color-border);\n    padding-bottom: 1px;\n  }\n  \
.blocks-feature-vertical-tabs-layout [data-scope=\"tabs\"][data-part=\"trigger\"][data-orientation=\"vertical\"] {\n    border-inline-end: 0;\n    margin-inline-end: 0;\n    border-bottom: 2px solid transparent;\n    margin-bottom: -1px;\n    flex-shrink: 0;\n    outline-offset: -2px;\n  }\n  \
.blocks-feature-vertical-tabs-layout [data-scope=\"tabs\"][data-part=\"trigger\"][data-state=\"active\"][data-orientation=\"vertical\"] {\n    border-bottom-color: var(--fandhe-palette, var(--fandhe-color-accent));\n  }\n  \
[data-blocks-feature-vertical-tabs-trigger-desc] {\n    display: none;\n  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// Demo が期待する部品・構造・非対話制約を満たしていることの単体
    /// 回帰（`crates/docs-site/tests/blocks_contract.rs` の横断検査と重複
    /// し過ぎない範囲での個別固定）。
    #[test]
    fn demo_composes_expected_parts_and_avoids_forms() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"tabs\"",
            "data-scope=\"image\"",
            "data-scope=\"icon\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert!(html.contains("type=\"button\""));
        assert!(!html.contains("<form"));
        assert!(!html.contains("src=\"data:"));
    }

    /// 先頭タブ（`build`）のみが選択済み・4 パネル中 3 パネルが `hidden`
    /// であること（無 JS 前提の静的固定表示、モジュール doc「無 JS での
    /// 扱い」節）。
    #[test]
    fn demo_selects_first_tab_and_hides_other_panels() {
        let html = render(&demo());
        assert_eq!(html.matches("aria-selected=\"true\"").count(), 1);
        assert_eq!(html.matches("aria-selected=\"false\"").count(), 3);
        assert_eq!(html.matches(" hidden").count(), 3);
    }

    /// `data-orientation="vertical"`/`aria-orientation="vertical"` が
    /// 出力されること（モジュール doc「`Orientation::Vertical` を採用する
    /// 理由」節）。
    #[test]
    fn demo_declares_vertical_orientation() {
        let html = render(&demo());
        assert!(html.contains("data-orientation=\"vertical\""));
        assert!(html.contains("aria-orientation=\"vertical\""));
    }

    /// id の接頭辞が想定どおりであること。
    #[test]
    fn demo_uses_expected_id_prefix() {
        let html = render(&demo());
        assert!(html.contains("id=\"blocks-feature-vertical-tabs-basic-trigger-build\""));
        assert!(html.contains("id=\"blocks-feature-vertical-tabs-basic-content-build\""));
    }

    /// trigger（`<button>...</button>` の区間）が phrasing content のみで
    /// 構成されること（モジュール doc「trigger 内は phrasing content だけ
    /// で組む」節の不変条件）。
    #[test]
    fn trigger_button_contains_no_block_level_elements() {
        let html = render(&demo());
        let mut start = 0;
        let mut checked = 0;
        while let Some(open_rel) = html[start..].find("<button") {
            let open = start + open_rel;
            let open_end = html[open..].find('>').map(|i| open + i + 1).unwrap();
            let close = html[open_end..]
                .find("</button>")
                .map(|i| open_end + i)
                .unwrap();
            let inner = &html[open_end..close];
            assert!(
                !inner.contains("<p"),
                "trigger must not contain <p>: {inner}"
            );
            assert!(
                !inner.contains("<h"),
                "trigger must not contain heading tags: {inner}"
            );
            start = close + "</button>".len();
            checked += 1;
        }
        assert_eq!(checked, 4);
    }

    /// [`LAYOUT_CSS`] が lg 未満の横並び上書き・詳細度確保の子孫セレクタを
    /// 持つこと。
    #[test]
    fn layout_css_declares_lg_breakpoint_overrides() {
        assert!(LAYOUT_CSS.contains("@media (max-width: 63.99rem)"));
        assert!(LAYOUT_CSS.contains(
            "[data-scope=\"tabs\"][data-part=\"list\"][data-orientation=\"vertical\"] {\n    flex-direction: row;"
        ));
        assert!(LAYOUT_CSS.contains("overflow-x: auto;"));
        assert!(LAYOUT_CSS
            .contains("[data-blocks-feature-vertical-tabs-trigger-desc] {\n    display: none;"));
    }

    /// ルート class（`demo_class` とは別名）が `demo()` の出力へ実際に
    /// 現れること（`feature_split_list_image`/`feature_image_cards` と
    /// 同じ Bugbot 教訓の固定）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-feature-vertical-tabs-layout\""));
        assert_ne!(
            super::BLOCK.demo_class,
            "blocks-feature-vertical-tabs-layout"
        );
    }
}
