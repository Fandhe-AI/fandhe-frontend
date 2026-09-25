# feature-vertical-tabs

`heading` / `text` / `tabs` / `image` / `icon` の 5 部品を合成した、左列に
縦並びの feature タブ、右列に選択中 feature の詳細（見出し・チェック付き
機能一覧・画像）を置くセクションです。

先頭タブ（build）を選択済みの状態で固定表示しています（docs サイトは
JS ハイドレーションを行わないため）。幅 lg（64rem）以上ではタブ列が左に
縦並び、パネルが右に表示されます。幅 lg 未満ではタブ列がパネルの上へ
移り、横スクロールできる横並びに切り替わります（この幅ではタブの
`aria-orientation` は `"vertical"` のまま残りますが、docs サイトは
JS ハイドレーションを行わないため実害はありません）。

文言・データはすべて架空のもので、データ取得・送信は行わない静的な表示
例です。`<form>` は使用せず、送信先を持ちません。

## Rust コード

```rust
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
```

## 原案差分メモ

参照（対応表 ID R0480。主参照であり他に集約元はないため、いわゆる
「集約元の差分」節は該当しません。出典の固有名・ファイル名は記載しません）
からの意図的な差分は次のとおりです。

- 参照元は横並び（Horizontal）のタブ + 見た目だけを縦並びに寄せる CSS
  という構成でしたが、本実装は `fandhe_frontend_pre_styled_ui::tabs` が
  既に持つ `data-orientation="vertical"` 相当の縦並び規則をそのまま
  採用しました（`aria-orientation` が実際のレイアウトと一致する意味論の
  正しさを優先した判断）。lg 未満のときだけ横並びへ CSS で上書きします。
- 参照元の背景帯・装飾・実際の文言は持ち込まず、文言はすべて独自の架空
  のもの（日本語）にしました。
- 見出しは `h3`/`h4` に下げました（ページ側が `## Demo` として `h2` を
  出すため）。
- 画像は `dummy_assets` のプレースホルダーと `alt=""`（装飾扱い）にしま
  した。
- チェックマークのアイコンは lucide 等の既存アイコンセットの path を
  複製しない自作の単純な線画です。
- 本イシュー（#2775）では骨格と主要領域（1 インスタンス・4 タブ）のみを
  実装しています。状態違いの並記・trigger アイコン等の仕上げ・画像主体の
  別パネル形は後続の #2776 で扱います。
