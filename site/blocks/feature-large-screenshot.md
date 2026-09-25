# feature-large-screenshot

`badge` / `heading` / `text` / `button` / `image` / `icon` の 6 部品を合成
した、中央寄せの見出しとリード文の下にアプリ画面の大きな画像を全幅で置き、
その下に feature 一覧を並べる 1 列構成の feature セクションです。

画像の見せ方 3 通り（下端フェード・枠付き・暗色パネル）を 1 つの Demo に
縦に並べています。フェードは面の背景色へ向かう `linear-gradient` で作り、
一覧を省いた最小形も含みます。feature 一覧は幅 md（48rem）未満で 1 列に
なります。文言はすべて架空のもので、データ取得・送信は行わない静的な
表示例です。`<form>` は使用しません。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::image::{self, AspectRatio, ImageFit, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};

/// 装飾用の自作幾何アイコン（lucide 等の著作物を複製しないための単純図形、
/// `feature_expand::geo_icon` と同型の判断）。線画（ストローク）として
/// 描画するため `fill="none"` を明示し、`icon` の `<svg>` 側が固定で持つ
/// `fill="currentColor"`（塗り面）を上書きする。
fn geo_icon(path_d: &'static str) -> Node {
    icon(
        &IconProps::default(),
        vec![],
        vec![el(
            "path",
            vec![
                ("d", path_d),
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

/// feature 項目 1 件分（見出し + 説明 + アイコン形状）。
struct FeatureItem {
    title: &'static str,
    body: &'static str,
    icon_path_d: &'static str,
}

/// A（基準形）向けの 6 件分の架空データ。C はこの先頭 3 件（`&ITEMS[..3]`）
/// を再利用する。
const ITEMS: [FeatureItem; 6] = [
    FeatureItem {
        title: "リアルタイム同期",
        body: "変更内容は即座に全メンバーへ反映され、手動更新は不要です。",
        icon_path_d: "M12 2a10 10 0 100 20 10 10 0 000-20zM12 6v6l4 2",
    },
    FeatureItem {
        title: "きめ細かな権限",
        body: "リソース単位で閲覧・編集・共有の範囲を割り当てられます。",
        icon_path_d: "M12 2l8 4v6c0 5-3.5 8-8 10-4.5-2-8-5-8-10V6z",
    },
    FeatureItem {
        title: "監査ログ",
        body: "誰が何をいつ変更したかを追跡し、後から検索できます。",
        icon_path_d: "M4 4h16v4H4zM4 10h10v4H4zM4 16h14v4H4z",
    },
    FeatureItem {
        title: "自動バックアップ",
        body: "日次でスナップショットを取得し、任意の時点へ復元できます。",
        icon_path_d: "M12 2v6l4-3M4 13a8 8 0 1016 0",
    },
    FeatureItem {
        title: "外部連携",
        body: "既存のツールとイベント連携し、通知や同期を自動化します。",
        icon_path_d: "M8 12h8M12 8v8M4 4h6v6H4zM14 14h6v6h-6z",
    },
    FeatureItem {
        title: "使用状況の可視化",
        body: "チームごとの利用傾向をダッシュボードで確認できます。",
        icon_path_d: "M4 20V10M10 20V4M16 20v-7M22 20V2",
    },
];

/// 中央寄せのセクション見出し（eyebrow badge + 見出し + リード文）。
/// `eyebrow` が `None` のとき badge は出力しない（B が使う）。
fn section_header(eyebrow: Option<&str>, title: &str, lead: &str) -> Node {
    let mut children: Vec<Node> = Vec::new();
    if let Some(eyebrow) = eyebrow {
        children.push(badge::badge(
            &BadgeProps::default(),
            vec![("data-blocks-feature-large-screenshot-eyebrow", "")],
            vec![text(eyebrow)],
        ));
    }
    children.push(heading(
        HeadingLevel::H3,
        &HeadingProps {
            size: HeadingSize::Xl3,
            weight: HeadingWeight::Bold,
        },
        vec![],
        vec![text(title)],
    ));
    children.push(styled_text::text(
        &TextProps {
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![("data-blocks-feature-large-screenshot-lead", "")],
        vec![text(lead)],
    ));

    div(
        vec![("class", "blocks-feature-large-screenshot-header")],
        children,
    )
}

/// 大きな画面画像（`variant` は `fade`/`bordered`/`panel` のいずれかで、
/// [`LAYOUT_CSS`] 側の見せ方の切り替えに使う）。装飾扱いのため `alt=""`。
fn screenshot(variant: &'static str) -> Node {
    div(
        vec![
            ("class", "blocks-feature-large-screenshot-media"),
            ("data-blocks-feature-large-screenshot-variant", variant),
        ],
        vec![image::image(
            &ImageProps {
                fit: ImageFit::Cover,
                aspect_ratio: AspectRatio::Auto,
                shape: ImageShape::Rounded,
                ..ImageProps::new(dummy_assets::SCREENSHOT_SRC, "")
            },
            vec![("data-blocks-feature-large-screenshot-image", "")],
        )],
    )
}

/// feature 項目 1 件（アイコン + 見出し + 説明）。
fn feature_item(item: &FeatureItem) -> Node {
    div(
        vec![("class", "blocks-feature-large-screenshot-item")],
        vec![
            geo_icon(item.icon_path_d),
            heading(
                HeadingLevel::H4,
                &HeadingProps::default(),
                vec![],
                vec![text(item.title)],
            ),
            styled_text::text(
                &TextProps {
                    size: TextSize::Sm,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![("data-blocks-feature-large-screenshot-desc", "")],
                vec![text(item.body)],
            ),
        ],
    )
}

/// feature 一覧（3 列 grid、md 未満は 1 列。R1154 の帯画像 + 定義リストを
/// ここへ統合した）。
fn feature_grid(items: &[FeatureItem]) -> Node {
    div(
        vec![("class", "blocks-feature-large-screenshot-grid")],
        items.iter().map(feature_item).collect(),
    )
}

/// A: 基準形（eyebrow + 見出し + リード文 → 下端フェード付き画像 →
/// feature 6 件の grid）。
fn fade_section() -> Node {
    div(
        vec![
            ("class", "blocks-feature-large-screenshot-section"),
            (
                "data-blocks-feature-large-screenshot-section-variant",
                "fade",
            ),
        ],
        vec![
            section_header(
                Some("導入事例"),
                "現場の画面をそのまま見せる",
                "実際のダッシュボード画面と、そこから得られる主要な特長をあわせて紹介します。",
            ),
            screenshot("fade"),
            feature_grid(&ITEMS),
        ],
    )
}

/// B: 最小形（見出し + リード文 + ボタン行 → 枠付き画像。一覧は持たない）。
fn bordered_section() -> Node {
    div(
        vec![
            ("class", "blocks-feature-large-screenshot-section"),
            (
                "data-blocks-feature-large-screenshot-section-variant",
                "bordered",
            ),
        ],
        vec![
            section_header(
                None,
                "まずは画面を見てみる",
                "導入前に、実際の操作画面を確認できます。",
            ),
            div(
                vec![("class", "blocks-feature-large-screenshot-actions")],
                vec![
                    button::button(
                        &ButtonProps::default(),
                        vec![("type", "button")],
                        vec![text("デモを見る")],
                    ),
                    button::button(
                        &ButtonProps {
                            variant: ButtonVariant::Outline,
                            ..ButtonProps::default()
                        },
                        vec![("type", "button")],
                        vec![text("資料をダウンロード")],
                    ),
                ],
            ),
            screenshot("bordered"),
        ],
    )
}

/// C: 暗色パネル（反転配色の中に見出し + リード文 → 画像 → feature 3 件の
/// grid）。
fn panel_section() -> Node {
    div(
        vec![
            ("class", "blocks-feature-large-screenshot-section"),
            (
                "data-blocks-feature-large-screenshot-section-variant",
                "panel",
            ),
        ],
        vec![
            section_header(
                None,
                "暗い画面でも見やすく",
                "ダークテーマのアプリでも、同じ構成で特長を伝えられます。",
            ),
            screenshot("panel"),
            feature_grid(&ITEMS[..3]),
        ],
    )
}

/// `feature-large-screenshot` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数（他 block と同じ状態を持たない設計）。ルート class は
/// `demo_class`（`blocks-feature-large-screenshot`）とは別名にする。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-feature-large-screenshot-layout")],
        vec![fade_section(), bordered_section(), panel_section()],
    )
}
```

## 原案差分メモ

参照（対応表 ID R0933・R0473・R0488・R0934・R0938・R1154。出典の固有名・
ファイル名は記載しません）からの意図的な差分は次のとおりです。

- 基準は R0933 とし、R0473 / R0488 / R0934 / R0938 / R1154 を 1 つの
  Demo へ統合しました（3 つのセクションを縦に並べる形）。
- R0473 が使う 16:9 の `AspectRatio::Video` ではなく `AspectRatio::Auto`
  を使い、プレースホルダー画像の元の比率のまま全幅表示しています。
- R0488 の最小形と R0934 の枠付き・一覧なしの構成は、1 つの B セクション
  （見出し + リード文 + ボタン行 → 枠付き画像、一覧なし）へ統合しました。
- R0938 の暗色パネルは C セクションとして再現しました。反転配色の中では
  `badge` を置かず（全 variant が反転面での可読性を保証しないため）、
  `icon`/`heading`/`text` の色継承のみで構成しています。
- R1154 が持つ上部の帯画像と定義リスト 6 項目は独立の Demo にせず、A の
  feature 6 件 grid へ統合しました。使用部品 6 つに `<dl>` は含まれない
  ため、定義リストは使いません。
- 参照元にある負の `margin-inline` による画像のはみ出しは採らず、画像は
  `width: 100%` の全幅のみに留めています（Demo 枠が横スクロールする
  副作用を避けるため）。
- フェードは `linear-gradient` の行き先をテーマの背景色（`fade` は
  `--fandhe-color-bg`、`panel` はパネル面の `--fandhe-color-fg`）へ一致
  させ、画像の背後にある面の色とフェードの行き先を揃えました。
- 見出しは 1 段下げて `h3`、feature 項目見出しは `h4` にしました（ページ側
  が `## Demo` として `h2` を出すため）。
- 画像は `dummy_assets` のプレースホルダー + `alt=""`（装飾扱い）を使い
  ます。
- 文言・配色は独自のもの、または既存のテーマトークンにそのまま従います。
