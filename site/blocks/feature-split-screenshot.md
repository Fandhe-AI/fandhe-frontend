# feature-split-screenshot

`badge` / `heading` / `text` / `icon` / `image` / `code` の 6 部品を合成
した、テキスト列（eyebrow badge・見出し・リード文・アイコン付きの
インライン feature 3 件）と画像列（アプリ画面。列幅を超えてはみ出す）の
2 列構成の feature セクションです。

3 通りの見せ方（基準形・反転 + アクセント色パネル・タブ付きコード枠）を
1 つの Demo に縦に並べています。lg（64rem）未満では画像・コード枠が
テキストの下に来る 1 列表示になり、lg 以上でのみ左右の反転を行います。
画像は各セクションの境界で切り取られる形ではみ出します。文言はすべて
架空のもので、データ取得・送信は行わない静的な表示例です。`<form>` は
使用しません。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, span, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::code::{self, CodeProps};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::image::{self, AspectRatio, ImageFit, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};

/// 装飾用の自作幾何アイコン（lucide 等の著作物を複製しないための単純図形、
/// `feature_large_screenshot::geo_icon` と同型の判断）。線画（ストローク）
/// として描画するため `fill="none"` を明示し、`icon` の `<svg>` 側が固定で
/// 持つ `fill="currentColor"`（塗り面）を上書きする。
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

/// インライン feature 項目 1 件分（用語 + 説明 + アイコン形状）。
struct FeatureItem {
    term: &'static str,
    body: &'static str,
    icon_path_d: &'static str,
}

/// A/B 向けの 3 件分の架空データ。
const ITEMS: [FeatureItem; 3] = [
    FeatureItem {
        term: "自動同期",
        body: "端末間の差分を検出し、操作を待たずに反映します。",
        icon_path_d: "M12 2a10 10 0 100 20 10 10 0 000-20zM12 6v6l4 2",
    },
    FeatureItem {
        term: "権限管理",
        body: "役割ごとに閲覧・編集の範囲を割り当てられます。",
        icon_path_d: "M12 2l8 4v6c0 5-3.5 8-8 10-4.5-2-8-5-8-10V6z",
    },
    FeatureItem {
        term: "利用状況",
        body: "チームごとの利用傾向を継続的に記録します。",
        icon_path_d: "M4 20V10M10 20V4M16 20v-7M22 20V2",
    },
];

/// C（開発者向け）の 3 件分の架空データ。
const DEV_ITEMS: [FeatureItem; 3] = [
    FeatureItem {
        term: "型安全な API",
        body: "呼び出し側の型情報から補完と検証を行います。",
        icon_path_d: "M4 4h16v4H4zM4 10h10v4H4zM4 16h14v4H4z",
    },
    FeatureItem {
        term: "設定不要",
        body: "既定値のまま導入でき、追加の初期設定を要しません。",
        icon_path_d: "M12 2v6l4-3M4 13a8 8 0 1016 0",
    },
    FeatureItem {
        term: "拡張ポイント",
        body: "既存の処理を差し替えずに振る舞いを追加できます。",
        icon_path_d: "M8 12h8M12 8v8M4 4h6v6H4zM14 14h6v6h-6z",
    },
];

/// インライン feature 項目 1 件（アイコン + 太字の用語 + 説明を 1 行の
/// 流れで並べる）。
fn feature_row(item: &FeatureItem) -> Node {
    div(
        vec![("class", "blocks-feature-split-screenshot-feature")],
        vec![
            geo_icon(item.icon_path_d),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![("data-blocks-feature-split-screenshot-feature-text", "")],
                vec![
                    el("strong", vec![], vec![text(item.term)]),
                    text(" "),
                    text(item.body),
                ],
            ),
        ],
    )
}

/// テキスト列（eyebrow badge + 見出し + リード文 + インライン feature
/// 一覧）。`eyebrow` が `None` のとき badge は出力しない。
fn copy_column(eyebrow: Option<&str>, title: &str, lead: &str, features: &[FeatureItem]) -> Node {
    let mut children: Vec<Node> = Vec::new();
    if let Some(eyebrow) = eyebrow {
        children.push(badge::badge(
            &BadgeProps::default(),
            vec![("data-blocks-feature-split-screenshot-eyebrow", "")],
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
        vec![("data-blocks-feature-split-screenshot-lead", "")],
        vec![text(lead)],
    ));
    children.push(div(
        vec![("class", "blocks-feature-split-screenshot-features")],
        features.iter().map(feature_row).collect(),
    ));

    div(
        vec![("class", "blocks-feature-split-screenshot-text")],
        children,
    )
}

/// アプリ画面のプレースホルダー画像（列幅より大きい固定幅で、
/// [`LAYOUT_CSS`] 側がセクション境界での切り取りを行う）。装飾扱いのため
/// `alt=""`。
fn screenshot() -> Node {
    image::image(
        &ImageProps {
            fit: ImageFit::Cover,
            aspect_ratio: AspectRatio::Auto,
            shape: ImageShape::Rounded,
            ..ImageProps::new(dummy_assets::SCREENSHOT_SRC, "")
        },
        vec![("data-blocks-feature-split-screenshot-image", "")],
    )
}

/// A: 基準形（テキスト列 → 画像列。画像は右へはみ出す）。
fn section_a() -> Node {
    div(
        vec![("class", "blocks-feature-split-screenshot-section")],
        vec![
            copy_column(
                Some("プラットフォーム"),
                "画面のまま特長を伝える",
                "実際の操作画面と、そこから得られる主要な特長をあわせて紹介します。",
                &ITEMS,
            ),
            div(
                vec![("class", "blocks-feature-split-screenshot-media")],
                vec![screenshot()],
            ),
        ],
    )
}

/// B: 反転 + アクセント色パネル（lg 以上で画像列が左。パネルの外側へ
/// はみ出す）。
fn section_b() -> Node {
    div(
        vec![
            ("class", "blocks-feature-split-screenshot-section"),
            ("data-blocks-feature-split-screenshot-reverse", ""),
        ],
        vec![
            copy_column(
                None,
                "反転レイアウトでも読みやすく",
                "画像とテキストの位置を入れ替えても、同じ構成のまま伝えられます。",
                &ITEMS,
            ),
            div(
                vec![("class", "blocks-feature-split-screenshot-media")],
                vec![div(
                    vec![("class", "blocks-feature-split-screenshot-panel")],
                    vec![screenshot()],
                )],
            ),
        ],
    )
}

/// C: タブ付きコード枠（テキスト列 → 開発者向けのコード表示枠）。
fn section_c() -> Node {
    let snippet = "let app = App::new();\napp.mount(\"#root\");\napp.run();\n";
    div(
        vec![("class", "blocks-feature-split-screenshot-section")],
        vec![
            copy_column(
                None,
                "コードで組み込む",
                "既存のアプリへ数行を追加するだけで導入できます。",
                &DEV_ITEMS,
            ),
            div(
                vec![("class", "blocks-feature-split-screenshot-media")],
                vec![div(
                    vec![("class", "blocks-feature-split-screenshot-code-frame")],
                    vec![
                        div(
                            vec![("class", "blocks-feature-split-screenshot-tabs")],
                            vec![
                                span(
                                    vec![("data-blocks-feature-split-screenshot-tab-active", "")],
                                    vec![text("main.rs")],
                                ),
                                span(vec![], vec![text("Cargo.toml")]),
                            ],
                        ),
                        el(
                            "pre",
                            vec![("class", "blocks-feature-split-screenshot-pre")],
                            vec![code::code(
                                &CodeProps::default(),
                                vec![("data-blocks-feature-split-screenshot-code", "")],
                                vec![text(snippet)],
                            )],
                        ),
                    ],
                )],
            ),
        ],
    )
}

/// `feature-split-screenshot` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数（他 block と同じ状態を持たない設計）。ルート class は
/// `demo_class`（`blocks-feature-split-screenshot`）とは別名にする。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-feature-split-screenshot-layout")],
        vec![section_a(), section_b(), section_c()],
    )
}
```

## 原案差分メモ

参照（対応表 ID R0932・R0936・R0939・R0942。出典の固有名・ファイル名は
記載しません）からの意図的な差分は次のとおりです。

- 基準は R0932 とし、R0936（画像左の反転版）と R0939（アクセント色
  パネルに載せる形）を 1 つの B セクションへ統合しました。どちらも画像列
  の見せ方の差分であり、独立の Demo に分けるほどの構造差ではないと
  判断したためです。
- R0942（タブ付きコード枠）は C セクションとして再現しました。`tabs`
  部品は使用部品一覧に含まれず、JS ハイドレーションを行わない docs
  サイトでは操作もできないため、タブ列は素の `span` 2 つによる静的表示
  にとどめ、`role="tab"` 等の操作可能を示す ARIA は付けていません。
- 参照元が使う負の `margin-inline` によるはみ出しは採らず、各セクション
  のルートへ `overflow: hidden` を付けて画像をセクションの境界で切り取る
  方式にしました（Demo 枠が横スクロールする副作用を避けるため）。
- アクセント色パネルには画像だけを置きテキストを含めないため、反転面での
  文字色のコントラスト問題は発生しません。
- 画像は `dummy_assets` のプレースホルダー + `alt=""`（装飾扱い）を使い
  ます。
- 見出しは 1 段下げて `h3` にしました（ページ側が `## Demo` として `h2`
  を出すため）。インライン feature 項目は見出し要素にせず、太字の用語 +
  説明の 1 行表記にしています。
- コードスニペットは架空の Rust 風コードで、トークン・URL・メール
  アドレスに見える文字列は含みません。
- 文言・配色は独自のもの、または既存のテーマトークンにそのまま従います。
