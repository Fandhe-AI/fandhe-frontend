# feature-tabs-panel

`badge` / `heading` / `text` / `image` の 4 部品を合成した、タブで切り替える
feature セクションです。見出しの下にタブ列を並べ、選んだタブに対応する
テキストと画像を表示します。docs サイトは JS ハイドレーションを行わない
ため、選択したタブへ切り替える操作は実現できません。当初は実物の `tabs`
コンポーネント（`role="tab"`・`tabindex` を持つ操作可能に見えるトリガー
ボタン）を使っていましたが、クリック・キーボード操作をしても実際には
切り替わらない点がアクセシビリティ契約に反するとの指摘（Codex P1）を
2 度受けたため、`tabs` コンポーネントの使用をやめ、`role`/`tabindex`/
`<button>` を一切持たない静的な `span` 列（選択中の項目だけを見た目で
強調）へ置き換えました。`PANELS` の 4 件それぞれについて「そのタブを
選択した場合の表示」を静的タブ列 + パネル本文の組として縦に併記し、
すべてのパネル本文が常に可視のまま静的 HTML に現れるようにしています。

文言はすべて架空のもので、データ取得・送信は行わない静的な表示例です。
`<form>` は使用しません。

## Rust コード

```rust
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
```

## 原案差分メモ

参照（対応表 ID R1158 基準形。集約元 R0104/R0478/R0479/R0481。出典の
固有名・ファイル名は記載しません）からの意図的な差分は次のとおりです。

- 本イシュー（#2772）では基準形（R1158）のみを実装しました。ピル型タブ
  （R0478）・パネル内でのテキスト/画像の交互配置（R0479）・中央見出し +
  カードグリッド（R0481）・トリガーへの説明 + 進捗バー付与（R0104）・
  モバイルでの全 feature 縦積み代替形は、続く #2773 が `variant_state` を
  再利用して追加します。
- 参照元の配色・装飾・アイコンは持ち込まず、既存の pre-styled-ui 部品の
  既定スタイルのみで構成しました。
- 見出しは 1 段下げて `h3`（パネル内は `h4`）にしました（ページ側が
  `## Demo` として `h2` を出すため）。
- 画像は `dummy_assets` のプレースホルダー + `alt=""`（装飾扱い）を使用し、
  実在のブランド・人物・企業とは無関係の架空データです。
- 無 JS 制約に従い、`tabs` コンポーネントは一切使わず、`role`/`tabindex`/
  `<button>` を持たない静的なタブ列（選択中の項目だけを
  `data-blocks-feature-tabs-panel-selected` 属性で強調）を `PANELS` の
  4 件それぞれに添えて縦に併記しました。当初はパネル数（4 件）ぶんの
  `tabs` インスタンスを選択状態違いで縦に並べていましたが（`pricing_
  tiers_morph`/`sidebar_07` と同型の対処）、この構成は操作可能に見えて
  実際には切り替わらないトリガーボタンを 4 インスタンス分反復して出して
  おり、UI のアクセシビリティ契約に反するとの指摘（Codex P1）を受けました。
  続けて実物の `tabs` インスタンスを 1 個へ減らし残り 3 パネルを非対話
  プレビューとする是正を行いましたが、その 1 個自体が依然として 4 個の
  操作可能に見えるトリガーボタンを持ち選択状態が変わらない点は変わらない
  ため、同一箇所への再指摘（Codex P1 2 回目）を受けました。2 回の指摘は
  いずれも「操作可能に見えるが実際には切り替わらない要素を一切出さない」
  という 1 点に帰着するため、`tabs` コンポーネントの呼び出し自体を完全に
  取り除く現在の構成へ是正しました。

関連部品: [Badge](../themes/badge.md) / [Heading](../themes/heading.md) /
[Text](../themes/text.md) / [Image](../themes/image.md)
