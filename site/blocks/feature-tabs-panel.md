# feature-tabs-panel

`badge` / `heading` / `text` / `tabs` / `image` の 5 部品を合成した、
タブで切り替える feature セクションです。見出しの下に下線タブ
（`TabsVariant::Line`）を並べ、選んだタブのパネルだけにテキストと画像を
表示します。docs サイトは JS ハイドレーションを行わないため、選択中の
パネル以外は `tabs` が `hidden` を付けて静的に描画します。単一インスタンス
だけを固定表示すると残りのパネルの内容が一切読めなくなるため、パネルの
数だけ `tabs` インスタンスを選択状態違いでキャプション付きに縦へ並べ、
すべてのパネル本文がいずれかのインスタンスで可視のまま静的 HTML に
現れるようにしています。

文言はすべて架空のもので、データ取得・送信は行わない静的な表示例です。
`<form>` は使用しません。

## Rust コード

```rust
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
```

## 原案差分メモ

参照（対応表 ID R1158 基準形。集約元 R0104/R0478/R0479/R0481。出典の
固有名・ファイル名は記載しません）からの意図的な差分は次のとおりです。

- 本イシュー（#2772）では基準形（R1158）のみを実装しました。ピル型タブ
  （R0478）・パネル内でのテキスト/画像の交互配置（R0479）・中央見出し +
  カードグリッド（R0481）・トリガーへの説明 + 進捗バー付与（R0104）・
  モバイルでの全 feature 縦積み代替形・状態違いの並記は、続く #2773 が
  `tabs_panel`・`panel_items` を再利用して追加します。
- 参照元の配色・装飾・アイコンは持ち込まず、既存の pre-styled-ui 部品の
  既定スタイルのみで構成しました。
- 見出しは 1 段下げて `h3`（パネル内は `h4`）にしました（ページ側が
  `## Demo` として `h2` を出すため）。
- 画像は `dummy_assets` のプレースホルダー + `alt=""`（装飾扱い）を使用し、
  実在のブランド・人物・企業とは無関係の架空データです。
- 無 JS 制約に従い、パネル数（4 件）ぶんの `tabs` インスタンスを選択状態
  違いでキャプション付きに縦へ並べました。単一インスタンスだけを初期タブ
  固定で描画すると、非選択の 3 パネルは `hidden` のまま静的ページから
  一切読めなくなるため（`pricing_tiers_morph`/`sidebar_07` と同型の対処）。

関連部品: [Badge](../themes/badge.md) / [Heading](../themes/heading.md) /
[Text](../themes/text.md) / [Tabs](../themes/tabs.md) /
[Image](../themes/image.md)
