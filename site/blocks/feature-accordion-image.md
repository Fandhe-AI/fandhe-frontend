# feature-accordion-image

`fandhe-frontend-pre-styled-ui` の `badge` / `heading` / `text` /
`accordion` / `image` の 5 部品を合成した、対応表 ID R0103（基準形）に
相当する 2 列構成の機能紹介です。左列に見出しエリアとアコーディオンを、
右列に開いている項目に対応する画像を配置します。Blocks セクションは
新規部品を追加するものではなく、既存の Themes/Primitives 部品を組み合わ
せた実例集であることに注意してください。

`md`（768px）未満では右列の画像を隠し、開いている項目の本文の中にインラ
イン画像を表示します。docs サイトは JS ハイドレーションを行わないため、
表示は常に「1 件目が開いた状態」で固定され、項目を切り替える操作はでき
ません。

集約元にはもう 1 件（対応表 ID R0483。上部にカテゴリ切替ボタン列が付き、
選択中を `aria-pressed` で示す形）がありますが、規模が大きいため本 PR
では基準形（R0103）のみを実装し、カテゴリ切替ボタン列は後続イシューへ
回しています。

機能名・説明文はすべて架空のものです（実在の製品・企業名・PII を含みま
せん）。`<form>` は使用していません。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, span, text, Node};
use fandhe_frontend_pre_styled_ui::accordion::{
    self, item, item_content, item_indicator, item_trigger, AccordionProps, OpenState,
};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::image::{self, AspectRatio, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};
use fandhe_frontend_pre_styled_ui::Size;

/// 機能 1 件分のダミーデータ（架空、実在の製品・企業とは無関係）。
struct Feature {
    name: &'static str,
    description: &'static str,
}

/// 機能一覧（架空、4 件）。1 件目がモジュール doc「静的アコーディオン」
/// 節のとおり初期状態で開いている。
const FEATURES: [Feature; 4] = [
    Feature {
        name: "リアルタイム共同編集",
        description: "複数人が同じドキュメントを同時に編集し、変更が即座に反映されます。",
    },
    Feature {
        name: "バージョン履歴",
        description: "過去のすべての変更を遡って確認し、いつでも以前の状態に戻せます。",
    },
    Feature {
        name: "カスタムテンプレート",
        description: "よく使う構成をテンプレートとして保存し、次回から素早く再利用できます。",
    },
    Feature {
        name: "アクセス権限の管理",
        description: "閲覧・編集・管理者の 3 段階で、メンバーごとに権限を細かく設定できます。",
    },
];

/// 見出しエリア（アイブロウ badge + heading + リード文）を組み立てる。
fn header() -> Node {
    div(
        vec![("class", "blocks-feature-accordion-image-header")],
        vec![
            badge::badge(
                &BadgeProps::default(),
                vec![("data-blocks-feature-accordion-image-eyebrow", "")],
                vec![text("機能紹介")],
            ),
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    weight: HeadingWeight::Bold,
                },
                vec![],
                vec![text("チームの作業をまとめて効率化")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(
                    "項目を選ぶと、右側に対応する画面イメージが表示されます。",
                )],
            ),
        ],
    )
}

/// 開いている項目に対応する画像を組み立てる（`inline` が `true` のとき
/// md 未満のインライン表示用フックを、`false` のとき右列表示用フックを
/// 付与する。両者は同じ画像を指すが、[`LAYOUT_CSS`] のブレークポイントに
/// 応じてどちらか一方だけが可視になる）。
fn feature_image(inline: bool) -> Node {
    let hook = if inline {
        ("data-blocks-feature-accordion-image-inline-image", "")
    } else {
        ("data-blocks-feature-accordion-image-media", "")
    };
    image::image(
        &ImageProps {
            aspect_ratio: AspectRatio::Video,
            shape: ImageShape::Rounded,
            ..ImageProps::new(dummy_assets::SCREENSHOT_SRC, "")
        },
        vec![hook],
    )
}

/// トリガーのラベル領域（機能名のみ。トリガーの直接の子は本要素と
/// `item_indicator` の 2 個に保つ、モジュール doc「トリガーの子を 2 個に
/// 保つ理由」節）。
fn trigger_label(feature: &Feature) -> Node {
    span(
        vec![("class", "blocks-feature-accordion-image-trigger-label")],
        vec![text(feature.name)],
    )
}

/// 機能 1 件分の accordion item を組み立てる（モジュール doc「静的
/// アコーディオン」節の方式）。
fn feature_item(index: usize, feature: &Feature, open: bool) -> Node {
    let state = if open {
        OpenState::Open
    } else {
        OpenState::Closed
    };
    let props = AccordionProps {
        disabled: true,
        ..AccordionProps::default()
    };
    let trigger_id = format!("blocks-feature-accordion-image-{index}-trigger");
    let content_id = format!("blocks-feature-accordion-image-{index}-content");
    let controls = if open {
        Some(content_id.as_str())
    } else {
        None
    };

    let trigger = el(
        "h4",
        vec![("class", "blocks-feature-accordion-image-trigger-heading")],
        vec![item_trigger(
            state,
            false,
            &props,
            feature.name,
            Some(trigger_id.as_str()),
            controls,
            vec![],
            vec![
                trigger_label(feature),
                item_indicator(state, false, &props, vec![], vec![text("▾")]),
            ],
        )],
    );

    let mut children = vec![trigger];
    if open {
        children.push(item_content(
            state,
            false,
            &props,
            Some(content_id.as_str()),
            Some(trigger_id.as_str()),
            vec![],
            vec![div(
                vec![("class", "blocks-feature-accordion-image-body")],
                vec![
                    styled_text::text(
                        &TextProps::default(),
                        vec![],
                        vec![text(feature.description)],
                    ),
                    feature_image(true),
                ],
            )],
        ));
    }

    item(state, false, &props, vec![], children)
}

/// `feature-accordion-image` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数（モジュール doc「静的アコーディオン」節）。
pub fn demo() -> Node {
    let items: Vec<Node> = FEATURES
        .iter()
        .enumerate()
        .map(|(index, feature)| feature_item(index, feature, index == 0))
        .collect();

    let list = accordion::root(
        Size::Md,
        &AccordionProps::default(),
        vec![("data-blocks-feature-accordion-image-root", "")],
        items,
    );

    let left = div(
        vec![("class", "blocks-feature-accordion-image-left")],
        vec![header(), list],
    );

    let right = div(
        vec![("class", "blocks-feature-accordion-image-media-slot")],
        vec![feature_image(false)],
    );

    div(
        vec![("class", "blocks-feature-accordion-image-grid")],
        vec![left, right],
    )
}
```

## 原案差分メモ（暫定版、#2762 で仕上げます）

- 集約元 2 件のうち、上部にカテゴリ切替ボタン列が付く形（対応表 ID
  R0483）は本 PR では実装せず、後続イシュー #2762 へ回しました。JS を
  使わないサイトで `aria-pressed` を持つボタンをどう静的表示するかの
  判断が必要なためです。
- アコーディオンは無 JS のため状態機械を使わず、1 件目だけを開いた状態
  で固定描画しています。全トリガーに `disabled` + `aria-disabled="true"`
  を付与し、押しても何も起きない操作要素にならないようにしています
  （`changelog-accordion`・`careers-split-accordion` の既存の教訓を
  踏まえた対処）。
- 閉じた項目は本文（`item-content`）自体を出力しません。`hidden` 属性
  付きの到達できない本文ノードを DOM に残さないためです。
- セクション見出しは `h3`、各トリガーは `h4` で包んでいます。
- 画像はビルド時生成のプレースホルダー SVG（ダミー素材ヘルパ）に置き換
  えています。実在の製品画面は使用していません。
- 文言は独自に作成した架空のものです。配色は既存のテーマトークンに
  従います。

## 関連情報

- [Badge](../themes/badge.md)
- [Heading](../themes/heading.md)
- [Text](../themes/text.md)
- [Accordion](../themes/accordion.md)
- [Image](../themes/image.md)
