# feature-accordion-image

`fandhe-frontend-pre-styled-ui` の `badge` / `heading` / `text` /
`accordion` / `image` の 5 部品を合成した、対応表 ID R0103（基準形）に
相当する 2 列構成の機能紹介です。左列に見出しエリアとアコーディオンを、
右列に代表画像を配置します。Blocks セクションは新規部品を追加するもの
ではなく、既存の Themes/Primitives 部品を組み合わせた実例集であること
に注意してください。

`md`（768px）未満では右列の画像を隠し、各項目の本文の中にインライン
画像を表示します。docs サイトは JS ハイドレーションを行わないため、
全項目を常時展開状態（`disabled` なトリガー）で固定表示し、開閉を
切り替える操作はできません。

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

/// 機能一覧（架空、4 件）。全件がモジュール doc「静的アコーディオン」
/// 節のとおり常時展開状態で固定描画される。
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
                vec![text("各機能の詳細は以下でご確認いただけます。")],
            ),
        ],
    )
}

/// 機能の画像を組み立てる（`inline` が `true` のとき md 未満のインライン
/// 表示用フックを、`false` のとき右列表示用フックを付与する。
/// [`LAYOUT_CSS`] のブレークポイントに応じてどちらか一方だけが可視になる）。
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
/// アコーディオン」節の方式。全件を [`OpenState::Open`] + `disabled: true`
/// で固定するため、本文（[`item_content`]）は必ず出力される）。
fn feature_item(index: usize, feature: &Feature) -> Node {
    let state = OpenState::Open;
    let props = AccordionProps {
        disabled: true,
        ..AccordionProps::default()
    };
    let trigger_id = format!("blocks-feature-accordion-image-{index}-trigger");
    let content_id = format!("blocks-feature-accordion-image-{index}-content");

    let trigger = el(
        "h4",
        vec![("class", "blocks-feature-accordion-image-trigger-heading")],
        vec![item_trigger(
            state,
            false,
            &props,
            feature.name,
            Some(trigger_id.as_str()),
            Some(content_id.as_str()),
            vec![],
            vec![
                trigger_label(feature),
                item_indicator(state, false, &props, vec![], vec![text("▾")]),
            ],
        )],
    );

    let content = item_content(
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
    );

    item(state, false, &props, vec![], vec![trigger, content])
}

/// `feature-accordion-image` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数（モジュール doc「静的アコーディオン」節）。
pub fn demo() -> Node {
    let items: Vec<Node> = FEATURES
        .iter()
        .enumerate()
        .map(|(index, feature)| feature_item(index, feature))
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
- アコーディオンは無 JS のため状態機械を使わず、全項目を開いた状態で
  固定描画しています。当初は 1 件目だけを開き残りを閉じる設計でしたが、
  閉じた項目の説明文が視覚利用者・支援技術のいずれにも到達不能になり、
  かつリード文が「選択」という無 JS 下では実現しない操作を示唆する
  不整合があったため、`changelog-accordion`（イシュー #2818）と同型の
  「全件 open + disabled」へ是正しました（イシュー #2761 レビュー
  指摘）。全トリガーに `disabled` + `aria-disabled="true"` を付与し、
  押しても何も起きない操作要素にならないようにしています。
- `disabled` 由来の減光を打ち消す CSS セレクタ
  （`[data-scope="accordion"][data-part="item-trigger"][data-disabled]`）
  は `.blocks-feature-accordion-image-left` 配下への子孫結合子付きで
  書き、集約された `blocks.css` を読み込む他 block の disabled
  accordion トリガーへ波及しないようスコープしています（同レビュー
  指摘）。
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
