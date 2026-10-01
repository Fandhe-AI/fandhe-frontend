# category-carousel

カテゴリ一覧ブロックです。見出し行（左にセクション見出し、右に
一覧ページへのリンク）の下へ、カテゴリタイル（画像 + 名称）を横一列に
並べます。`heading` / `link` / `carousel` / `card` / `image` / `button` /
`link-overlay` の 7 部品に加え、前後トリガーが空ボタンにならないよう
`icon` も合成します。Blocks は既存部品の合成例であり、新しい UI 部品は
追加しません。

主参照は対応表 ID R0604 です。見出し行 + すべて見るリンクの構成は
R0638、狭幅カルーセル・広幅グリッドへの切り替えは R0606、横スクロール
版は R0825 を参照しています。

3 形を並べて掲載しています。**基準形**は 2〜3 枚同時表示のカルーセルで、
前後ボタン・ドット（indicator）を持ちます。**狭幅カルーセル・広幅
グリッド**形は、広い画面幅ではカルーセルをやめて 3 列グリッド表示へ
切り替え、下に「もっと見る」ボタンを添えます。**横スクロール**形は
カルーセル部品を使わず、前後ボタン・ドットを持たないスクロール領域
だけでタイルを送り、広い画面幅では 5 列グリッドへ切り替えます。

本 Demo は無 JS の静的表示のみのため、基準形・狭幅カルーセル形の前後
ボタン・ドット・「もっと見る」ボタンはいずれも常時無効状態（disabled）
で描画しています。横スクロール形はボタンを持たず、`role="region"` +
`aria-label` を付けたスクロール領域だけで操作可能です。

カテゴリ名・画像はすべて架空のデータであり、実在の企業・ブランド・
PII・実クレデンシャルは含みません。画像はビルド時生成の同梱
プレースホルダー SVG です。

本 Demo は `<form>` を含みません。送信処理・状態管理は一切持ちません。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::carousel::{self, Orientation};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::image::{image, AspectRatio, ImageFit, ImageProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::link_overlay::{self, overlay};
use fandhe_frontend_pre_styled_ui::Size;

/// リンク先の固定外部 URL（`blog_overlay_cards::REPO` と同じ方針。
/// `Block::demo` は `base_path` を受け取れないため、サイト内リンクではなく
/// 固定の外部 URL を使う）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// 架空のカテゴリ 6 件（実在の企業・商標とは無関係）。`src` は
/// [`dummy_assets`] の 4 種（`AVATAR_SRC` を除く）を循環させる。
const CATEGORIES: [(&str, &str); 6] = [
    ("生活雑貨", dummy_assets::PRODUCT_SRC),
    ("キッチン", dummy_assets::BACKGROUND_SRC),
    ("文房具", dummy_assets::SCREENSHOT_SRC),
    ("アウトドア", dummy_assets::LOGO_SRC),
    ("インテリア", dummy_assets::PRODUCT_SRC),
    ("ファッション小物", dummy_assets::BACKGROUND_SRC),
];

/// 各形の直前に置く短い形ラベル（`gallery_carousel::variant_label` と
/// 同型）。
fn variant_label(label: &'static str) -> Node {
    use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};
    styled_text::text(
        &TextProps {
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![],
        vec![text(label)],
    )
}

/// 自作の幾何アイコン（線画。`gallery_carousel::chevron` と同型）。
fn chevron(path_d: &'static str) -> Node {
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

fn chevron_left() -> Node {
    chevron("M15 18l-6-6 6-6")
}

fn chevron_right() -> Node {
    chevron("M9 18l6-6-6-6")
}

/// 見出し行（R0638）。左にセクション見出し、右に一覧ページへのリンクを
/// 置き、狭幅では折り返す。
fn header(title: &'static str, link_label: &'static str) -> Node {
    div(
        vec![("class", "blocks-category-carousel-header")],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    weight: HeadingWeight::Bold,
                },
                vec![],
                vec![text(title)],
            ),
            link::root(REPO, &LinkProps::default(), vec![], vec![text(link_label)]),
        ],
    )
}

/// カテゴリタイル 1 件（`card` > `link_overlay::root` > 画像 + 名称 +
/// `overlay`）。モジュール doc「`alt` を空文字列にする理由」参照。
fn category_tile(name: &'static str, src: &'static str) -> Node {
    card::root(
        CardProps::default(),
        vec![("data-blocks-category-carousel-card", "")],
        vec![link_overlay::root(
            vec![("data-blocks-category-carousel-link", "")],
            vec![
                image(
                    &ImageProps {
                        fit: ImageFit::Cover,
                        aspect_ratio: AspectRatio::Square,
                        ..ImageProps::new(src, "")
                    },
                    vec![("data-blocks-category-carousel-image", "")],
                ),
                heading(
                    HeadingLevel::H4,
                    &HeadingProps::default(),
                    vec![("data-blocks-category-carousel-name", "")],
                    vec![text(name)],
                ),
                overlay(
                    REPO,
                    vec![
                        ("aria-label", name),
                        ("data-blocks-category-carousel-overlay", ""),
                    ],
                    vec![],
                ),
            ],
        )],
    )
}

/// A/B 共通のカルーセル本体（control 行 + indicator 群）。`extra_root_attr`
/// は B 形のみが持つグリッド切り替え用フック。
fn category_carousel(
    label: &'static str,
    extra_root_attr: Option<(&'static str, &'static str)>,
) -> Node {
    let items: Vec<Node> = CATEGORIES
        .iter()
        .enumerate()
        .map(|(i, (name, src))| {
            carousel::item(
                Orientation::Horizontal,
                i,
                CATEGORIES.len(),
                i == 0,
                vec![("data-blocks-category-carousel-tile", "")],
                vec![category_tile(name, src)],
            )
        })
        .collect();
    let indicators: Vec<Node> = (0..CATEGORIES.len())
        .map(|i| {
            carousel::indicator(
                Orientation::Horizontal,
                i,
                i == 0,
                vec![
                    ("disabled", ""),
                    ("data-blocks-category-carousel-indicator", ""),
                ],
            )
        })
        .collect();

    let mut root_attrs: Vec<(&str, &str)> = vec![("data-blocks-category-carousel-root", "")];
    if let Some(attr) = extra_root_attr {
        root_attrs.push(attr);
    }

    carousel::root(
        Size::Md,
        Orientation::Horizontal,
        label,
        root_attrs,
        vec![
            carousel::control(
                Orientation::Horizontal,
                vec![("data-blocks-category-carousel-control", "")],
                vec![
                    carousel::prev_trigger(
                        Orientation::Horizontal,
                        true,
                        "前のカテゴリ",
                        vec![],
                        vec![chevron_left()],
                    ),
                    div(
                        vec![("class", "blocks-category-carousel-viewport")],
                        vec![carousel::item_group(Orientation::Horizontal, vec![], items)],
                    ),
                    carousel::next_trigger(
                        Orientation::Horizontal,
                        true,
                        "次のカテゴリ",
                        vec![],
                        vec![chevron_right()],
                    ),
                ],
            ),
            carousel::indicator_group(
                Orientation::Horizontal,
                vec![("data-blocks-category-carousel-indicators", "")],
                indicators,
            ),
        ],
    )
}

/// A 基準形（対応表 ID R0604）。
fn variant_a() -> Node {
    div(
        vec![("class", "blocks-category-carousel-section")],
        vec![
            header("カテゴリから探す", "すべてのカテゴリを見る"),
            category_carousel("カテゴリ一覧（2〜3 枚表示）", None),
        ],
    )
}

/// B 狭幅カルーセル・広幅グリッド（対応表 ID R0606）。
fn variant_b() -> Node {
    div(
        vec![("class", "blocks-category-carousel-section")],
        vec![
            header("カテゴリから探す", "すべてのカテゴリを見る"),
            category_carousel(
                "カテゴリ一覧（広い画面ではグリッド表示）",
                Some(("data-blocks-category-carousel-grid-mode", "")),
            ),
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Outline,
                    disabled: true,
                    ..ButtonProps::default()
                },
                vec![("data-blocks-category-carousel-cta", "")],
                vec![text("カテゴリをもっと見る")],
            ),
        ],
    )
}

/// C 横スクロール（対応表 ID R0825）。`carousel` 部品を使わず、前後ボタン・
/// ドットを持たないスクロール領域のみで送る。
fn variant_c() -> Node {
    let tiles: Vec<Node> = CATEGORIES
        .iter()
        .take(5)
        .map(|(name, src)| {
            div(
                vec![("class", "blocks-category-carousel-scroll-item")],
                vec![category_tile(name, src)],
            )
        })
        .collect();
    div(
        vec![("class", "blocks-category-carousel-section")],
        vec![
            header("カテゴリから探す", "すべてのカテゴリを見る"),
            div(
                vec![
                    ("class", "blocks-category-carousel-scroll"),
                    ("role", "region"),
                    ("aria-label", "カテゴリ一覧（横スクロール）"),
                ],
                tiles,
            ),
        ],
    )
}

/// `category-carousel` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数（`crate::blocks` モジュール doc「静的表示」節）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-category-carousel-layout")],
        vec![
            variant_label("基準形（対応表 ID R0604。既定 2 枚・lg 以上で 3 枚表示）"),
            variant_a(),
            variant_label("狭幅カルーセル・広幅グリッド（対応表 ID R0606）"),
            variant_b(),
            variant_label("横スクロール（対応表 ID R0825。lg 以上で 5 列グリッド）"),
            variant_c(),
        ],
    )
}
```

## 原案差分メモ

- 集約元 4 件（R0604 / R0638 / R0606 / R0825）はいずれも Demo 内で
  見た目の差分として並記しています。見出し行（R0638）は 3 形共通、
  基準形の表示枚数切り替え（R0604）・狭幅カルーセル／広幅グリッドの
  切り替え（R0606）・横スクロールと広幅 5 列化（R0825）はそれぞれ別
  インスタンスとして掲載しています。
- イシュー本文が指定する 7 部品に加え、前後トリガー（`prev-trigger`/
  `next-trigger`）が空ボタンにならないよう `icon` を合成しています
  （`gallery-carousel` の前例と同じ判断）。
- 横スクロール形（R0825）は `carousel` 部品を使いません。無 JS の
  静的表示では実際のドラッグ送りを示せないため、ネイティブのスクロール
  バーで送れる単純な `overflow-x: auto` 領域とし、前後ボタン・ドットは
  置きません。
- R0825 の参照元は広い画面幅で 5 列、R0606 の参照元は広い画面幅で
  3 列へ切り替えます。列数の違いはそのまま踏襲し、横スクロール形のみ
  5 列、狭幅カルーセル・広幅グリッド形のみ 3 列としています。
- ドット（indicator）は任意要件ですが、基準形・狭幅カルーセル形の
  両方に揃えて添えています（横スクロール形には置きません）。
