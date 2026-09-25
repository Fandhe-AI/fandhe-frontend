# feature-accordion-image

`fandhe-frontend-pre-styled-ui` の `badge` / `heading` / `text` /
`accordion` / `image` / `button` / `icon` の 7 部品を合成した、機能紹介
アコーディオンです。集約元は対応表 ID R0103（基準形。左列に見出しと
アコーディオン、右列に代表画像）と R0483（上部にカテゴリ切替ボタン列が
付き、選択中を `aria-pressed` で示す形）の 2 件で、本 Demo は 2 形を並記
します。Blocks セクションは新規部品を追加するものではなく、既存の
Themes/Primitives 部品を組み合わせた実例集であることに注意してください。

`md`（768px）未満では右列の画像を隠し、各項目の本文の中にインライン
画像を表示します。docs サイトは JS ハイドレーションを行わないため、
全項目を常時展開状態（`disabled` なトリガー）で固定表示し、開閉を
切り替える操作はできません。カテゴリ切替ボタンも同様に `disabled` +
`aria-pressed` の静的表示で、先頭カテゴリのみ選択済みの状態のまま
固定しています。

機能名・説明文・カテゴリ名はすべて架空のものです（実在の製品・企業名・
PII を含みません）。`<form>` は使用していません。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, span, text, Node};
use fandhe_frontend_pre_styled_ui::accordion::{
    self, item, item_content, item_indicator, item_trigger, AccordionProps, OpenState,
};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::image::{self, AspectRatio, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::Size;

/// 機能 1 件分のダミーデータ（架空、実在の製品・企業とは無関係）。
struct Feature {
    name: &'static str,
    description: &'static str,
}

/// 形 A（対応表 ID R0103）の識別キー（モジュール doc「id 接頭辞」節）。
const BASE: &str = "base";

/// 形 B（対応表 ID R0483）の識別キー。
const CATEGORY: &str = "category";

/// 形 A の機能一覧（架空、4 件）。全件が常時展開状態で固定描画される。
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

/// 形 B のカテゴリ 1 件分のラベル（架空、4 件。先頭が初期選択状態）。
const CATEGORIES: [&str; 4] = ["共同作業", "分析", "セキュリティ", "自動化"];

/// 形 B・先頭カテゴリ（[`CATEGORIES`]`[0]`）選択時の機能一覧（架空、3 件）。
/// 非選択カテゴリの機能は出力しない（hidden で抱えて到達不能にしない、
/// モジュール doc「カテゴリ切替ボタン列」節）。
const CATEGORY_FEATURES: [Feature; 3] = [
    Feature {
        name: "共有ワークスペース",
        description: "チーム全員が同じ作業台の上で、資料や進行状況を常に共有できます。",
    },
    Feature {
        name: "コメントとメンション",
        description: "気になる箇所に直接コメントを残し、担当者へ通知できます。",
    },
    Feature {
        name: "タスクの割り当て",
        description: "担当者と期限を明確にし、進捗をひと目で把握できます。",
    },
];

/// 各形の直前に置く短い形ラベル（`content_split_image` と同型、
/// `styled_text::text` の `Sm`/`Muted`）。
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

/// 見出しエリア（アイブロウ badge + heading + リード文）を組み立てる
/// （形ごとに文言を差し替えられるようパラメータ化する）。
fn header(eyebrow: &'static str, title: &'static str, lead: &'static str) -> Node {
    div(
        vec![("class", "blocks-feature-accordion-image-header")],
        vec![
            badge::badge(
                &BadgeProps::default(),
                vec![("data-blocks-feature-accordion-image-eyebrow", "")],
                vec![text(eyebrow)],
            ),
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
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

/// 機能の画像を組み立てる（`inline` が `true` のとき md 未満のインライン
/// 表示用フックを、`false` のとき右列表示用フックを付与する。
/// [`LAYOUT_CSS`] のブレークポイントに応じてどちらか一方だけが可視になる。
/// 両形で共有するため画像自体に `id` は持たせない）。
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
/// で固定するため、本文（[`item_content`]）は必ず出力される）。`key` は
/// 形キー（[`BASE`]/[`CATEGORY`]）で、2 形並記時の id 衝突を防ぐ
/// （モジュール doc「id 接頭辞」節）。
fn feature_item(key: &str, index: usize, feature: &Feature) -> Node {
    let state = OpenState::Open;
    let props = AccordionProps {
        disabled: true,
        ..AccordionProps::default()
    };
    let trigger_id = format!("blocks-feature-accordion-image-{key}-{index}-trigger");
    let content_id = format!("blocks-feature-accordion-image-{key}-{index}-content");

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

/// 機能一覧 1 本分のアコーディオン（`root` + 各 [`feature_item`]）を
/// 組み立てる。両形（[`variant_a`]/[`variant_b`]）で共有する
/// （モジュール doc「2 形を 1 つの Demo に並記する」節）。
fn accordion_list(key: &str, features: &[Feature]) -> Node {
    let items: Vec<Node> = features
        .iter()
        .enumerate()
        .map(|(index, feature)| feature_item(key, index, feature))
        .collect();

    accordion::root(
        Size::Md,
        &AccordionProps::default(),
        vec![("data-blocks-feature-accordion-image-root", "")],
        items,
    )
}

/// カテゴリ切替ボタン 1 個分の幾何線画アイコン（`careers_split_accordion::
/// geo_icon` と同型。取得元アイコンセットの path・識別子は複製しない）。
fn category_icon() -> Node {
    icon(
        &IconProps {
            size: Size::Sm,
            ..IconProps::default()
        },
        vec![],
        vec![el(
            "path",
            vec![
                ("d", "M4 7h16M4 12h10M4 17h16"),
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

/// カテゴリ切替ボタン 1 個（モジュール doc「カテゴリ切替ボタン列」節）。
/// `selected` が `true` のとき `Solid` variant + `aria-pressed="true"`、
/// それ以外は `Outline` variant + `aria-pressed="false"` を固定で持つ
/// （初期状態のまま切り替わらない静的表示、`disabled: true` で dead
/// control 化を防ぐ）。
fn category_button(label: &'static str, selected: bool) -> Node {
    let (variant, pressed) = if selected {
        (ButtonVariant::Solid, "true")
    } else {
        (ButtonVariant::Outline, "false")
    };
    button::button(
        &ButtonProps {
            variant,
            size: Size::Sm,
            disabled: true,
            ..ButtonProps::default()
        },
        vec![
            ("aria-pressed", pressed),
            ("data-blocks-feature-accordion-image-category", ""),
        ],
        vec![category_icon(), text(label)],
    )
}

/// カテゴリ切替ボタン列（形 B のみ）。[`CATEGORIES`] の先頭のみ選択済み
/// として固定し、選択中カテゴリの機能一覧（[`CATEGORY_FEATURES`]）と
/// 対応させる（モジュール doc「カテゴリ切替ボタン列」節）。
fn category_row() -> Node {
    let buttons: Vec<Node> = CATEGORIES
        .iter()
        .enumerate()
        .map(|(index, label)| category_button(label, index == 0))
        .collect();
    div(
        vec![
            ("class", "blocks-feature-accordion-image-categories"),
            ("role", "group"),
            ("aria-label", "機能カテゴリ"),
        ],
        buttons,
    )
}

/// 形 A（対応表 ID R0103、基準形）: 見出しエリア + アコーディオンの左列と
/// 代表画像の右列からなる 2 列構成。
fn variant_a() -> Node {
    let left = div(
        vec![("class", "blocks-feature-accordion-image-left")],
        vec![
            header(
                "機能紹介",
                "チームの作業をまとめて効率化",
                "各機能の詳細は以下でご確認いただけます。",
            ),
            accordion_list(BASE, &FEATURES),
        ],
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

/// 形 B（対応表 ID R0483）: 見出しの下にカテゴリ切替ボタン列（[`category_row`]）
/// を置いた上で、形 A と同じ 2 列構成（左列アコーディオン・右列代表画像）を
/// 続ける（モジュール doc「2 形を 1 つの Demo に並記する」節）。
fn variant_b() -> Node {
    let header_node = header(
        "カテゴリで探す",
        "機能をカテゴリから選んで確認",
        "気になるカテゴリを選ぶと、対応する機能の一覧が表示されます。",
    );
    let left = div(
        vec![("class", "blocks-feature-accordion-image-left")],
        vec![accordion_list(CATEGORY, &CATEGORY_FEATURES)],
    );
    let right = div(
        vec![("class", "blocks-feature-accordion-image-media-slot")],
        vec![feature_image(false)],
    );
    let grid = div(
        vec![("class", "blocks-feature-accordion-image-grid")],
        vec![left, right],
    );
    div(
        vec![("class", "blocks-feature-accordion-image-variant")],
        vec![header_node, category_row(), grid],
    )
}

/// `feature-accordion-image` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数（モジュール doc「静的アコーディオン」節）。形 A・形 B を
/// [`variant_label`] の見出し付きで縦に並記する（モジュール doc「2 形を
/// 1 つの Demo に並記する」節）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-feature-accordion-image-layout")],
        vec![
            variant_label("基準形（R0103）"),
            variant_a(),
            variant_label("カテゴリ切替ボタン列付き（R0483）"),
            variant_b(),
        ],
    )
}
```

## 原案差分メモ

- 集約元 2 件（対応表 ID R0103・R0483）の差分は、上部のカテゴリ切替
  ボタン列の有無です。R0483 はボタン列でカテゴリを選び、選択中を
  `aria-pressed` で示したうえで、対応するカテゴリの機能一覧を表示します。
  本 Demo はこの 2 形を `variant_label` の見出し付きで 1 つの Demo に
  並記し、差分をその場で比較できるようにしています（`content-split-
  image`/`cta-split-image` と同型の判断）。
- アコーディオンは無 JS のため状態機械を使わず、全項目を開いた状態で
  固定描画しています。当初は 1 件目だけを開き残りを閉じる設計でしたが、
  閉じた項目の説明文が視覚利用者・支援技術のいずれにも到達不能になり、
  かつリード文が「選択」という無 JS 下では実現しない操作を示唆する
  不整合があったため、`changelog-accordion`（イシュー #2818）と同型の
  「全件 open + disabled」へ是正しました（イシュー #2761 レビュー
  指摘）。全トリガーに `disabled` + `aria-disabled="true"` を付与し、
  押しても何も起きない操作要素にならないようにしています。集約元
  R0483 の「初期状態は 1 件目を開いた状態」という仕様からもこの点で
  意図的に逸脱しており、右列の代表画像は「選択中の項目に追従する画像」
  ではなく先頭機能（または選択中カテゴリの先頭機能）を指す固定の画像
  として扱っています。
- カテゴリ切替ボタン列（形 B）も同じ考え方で、押しても何も起きない
  有効な `<button>`（dead control）を作らないよう、全ボタンを
  `disabled: true` + `aria-pressed="true"|"false"` の静的表示にしています。
  非選択カテゴリの機能一覧は hidden で抱えず、そもそも出力していません
  （選択中カテゴリ 1 件分のみを常時表示）。
- `disabled` 由来の減光を打ち消す CSS セレクタ（アコーディオントリガー用
  `[data-scope="accordion"][data-part="item-trigger"][data-disabled]`・
  カテゴリボタン用 `[data-scope="button"][data-part="root"][data-disabled]`）
  は、それぞれ `.blocks-feature-accordion-image-left`/`.blocks-feature-
  accordion-image-categories` 配下への子孫結合子付きで書き、集約された
  `blocks.css` を読み込む他 block の disabled トリガー・disabled ボタン
  へ波及しないようスコープしています（イシュー #2761 レビュー指摘と
  同じ判断）。
- id/`aria-controls`/`aria-labelledby` は形キー（`base`/`category`）を
  挟んだ `blocks-feature-accordion-image-{key}-{index}-{trigger|content}`
  の形で導出し、2 形を 1 つの Demo に並記しても id が衝突しないように
  しています。
- セクション見出しは両形とも `h3`、各トリガーは `h4` で包んでいます。
- アイコン（カテゴリ切替ボタンの先頭アイコン）は自作の幾何線画であり、
  取得元のアイコンセットの path・識別子・配色・文言は持ち込んでいません。
- 画像はビルド時生成のプレースホルダー SVG（ダミー素材ヘルパ）に置き換
  えています。実在の製品画面は使用していません。
- 文言はすべて独自に作成した架空のものです。配色は既存のテーマトークン
  に従います。

## 関連情報

- [Badge](../themes/badge.md)
- [Heading](../themes/heading.md)
- [Text](../themes/text.md)
- [Accordion](../themes/accordion.md)
- [Image](../themes/image.md)
- [Button](../themes/button.md)
- [Icon](../themes/icon.md)
