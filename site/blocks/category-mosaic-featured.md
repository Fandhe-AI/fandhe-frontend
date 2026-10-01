# category-mosaic-featured

カテゴリ一覧ブロックです。先頭タイルを大きく扱うモザイク配置で、
2 列 × 2 行のグリッドの左列に先頭カテゴリを 2 行分、右列に残り 2 枚を
縦に積みます。`heading` / `link` / `link-overlay` / `image` / `button`
の 5 部品のみを使います。Blocks は既存部品の合成例であり、新しい UI
部品は追加しません。

主参照は対応表 ID R0821（見出しと全件リンクあり）です。見出しなし・
タイルごとのボタンの構成は R0037、見出しと一覧ボタン・縦長の画像は
R0605 を参照しています。

3 形を並べて掲載しています。**基準形**は見出し + 「すべてのカテゴリ」
リンクの下にモザイクグリッドを置きます。**見出しなし・タイルごとの
ボタン**形は見出し行を持たず、各タイルにボタンを添えます。**見出しと
一覧ボタン・縦長**形は見出し行の右側をボタンにし、グリッドを縦長
（行高 20rem）にします。

タイル全体を 1 本のリンク（`link-overlay`）にしているため、タイル内の
行動ラベルは装飾用の `span` とし、実際のリンクは `overlay` だけが
担います。見出しなし形のボタンは無 JS では動作しないため、常時
`disabled` で描画しています。

広い画面幅（40rem 以上）では 2 列グリッドへ切り替わり、先頭タイルが
2 行分を占めます。狭い画面幅では全タイルが縦 1 列・同じ高さに並び、
先頭タイルも特別扱いしません。

カテゴリ名・画像はすべて架空のデータであり、実在の企業・ブランド・
PII・実クレデンシャルは含みません。画像はビルド時生成の同梱
プレースホルダー SVG です。

本 Demo は `<form>` を含みません。送信処理・状態管理は一切持ちません。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::image::{image, ImageFit, ImageProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::link_overlay::{self, overlay};

/// リンク先の固定外部 URL（`category_carousel::REPO` と同じ方針。
/// `Block::demo` は `base_path` を受け取れないため、サイト内リンクではなく
/// 固定の外部 URL を使う）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// 架空のカテゴリ 3 件（実在の企業・商標とは無関係）。`CATEGORIES[0]` を
/// 常に先頭（featured）タイルとして扱う。
const CATEGORIES: [(&str, &str); 3] = [
    ("季節の器", dummy_assets::PRODUCT_SRC),
    ("旅の道具", dummy_assets::BACKGROUND_SRC),
    ("書斎の小物", dummy_assets::SCREENSHOT_SRC),
];

/// 各形の直前に置く短い形ラベル（`text` 部品を使わない素の `p`。
/// `category_carousel::variant_label` の `text` 版と同型の役割）。
fn variant_label(label: &'static str) -> Node {
    div(
        vec![("class", "blocks-category-mosaic-featured-variant-label")],
        vec![text(label)],
    )
}

/// 見出し行（R0821/R0605）。左にセクション見出し、右に `link`（全件
/// リンク）または `button`（一覧ボタン）を置き、狭幅では折り返す。
/// `right` が `None`（B 形）のときは見出しだけになる。
fn header(title: &'static str, right: Option<Node>) -> Node {
    let mut children = vec![heading(
        HeadingLevel::H3,
        &HeadingProps {
            size: HeadingSize::Xl2,
            weight: HeadingWeight::Bold,
        },
        vec![],
        vec![text(title)],
    )];
    if let Some(node) = right {
        children.push(node);
    }
    div(
        vec![("class", "blocks-category-mosaic-featured-header")],
        children,
    )
}

/// カテゴリタイル 1 件。`featured` が `true`（先頭タイル）のときだけ
/// `data-…-featured` を付け、モザイク配置（§「モザイクグリッドの配置」）の
/// フックにする。`per_tile_button` が `true`（B 形）のとき、行動ラベルの
/// `span` の代わりに常時 `disabled` の `button` を添える。`heading_level`
/// はタイル名見出しのレベル（モジュール doc「B 形タイル見出しのレベル」節
/// 参照、A/C 形は `H4`・B 形は `H3`）。
fn tile(
    name: &'static str,
    src: &'static str,
    featured: bool,
    per_tile_button: bool,
    heading_level: HeadingLevel,
) -> Node {
    let mut root_attrs = vec![("data-blocks-category-mosaic-featured-tile", "")];
    if featured {
        root_attrs.push(("data-blocks-category-mosaic-featured-featured", ""));
    }

    let action: Node = if per_tile_button {
        button::button(
            &ButtonProps {
                variant: ButtonVariant::Plain,
                disabled: true,
                ..ButtonProps::default()
            },
            vec![("data-blocks-category-mosaic-featured-tile-button", "")],
            vec![text("見てみる")],
        )
    } else {
        div(
            vec![
                ("class", "blocks-category-mosaic-featured-action"),
                ("aria-hidden", "true"),
            ],
            vec![text("見てみる")],
        )
    };

    link_overlay::root(
        root_attrs,
        vec![
            image(
                &ImageProps {
                    fit: ImageFit::Cover,
                    ..ImageProps::new(src, "")
                },
                vec![("data-blocks-category-mosaic-featured-image", "")],
            ),
            div(
                vec![
                    ("class", "blocks-category-mosaic-featured-scrim"),
                    ("aria-hidden", "true"),
                ],
                vec![],
            ),
            div(
                vec![("class", "blocks-category-mosaic-featured-content")],
                vec![
                    heading(
                        heading_level,
                        &HeadingProps::default(),
                        vec![("data-blocks-category-mosaic-featured-name", "")],
                        vec![text(name)],
                    ),
                    action,
                ],
            ),
            overlay(
                REPO,
                vec![
                    ("aria-label", name),
                    ("data-blocks-category-mosaic-featured-overlay", ""),
                ],
                vec![],
            ),
        ],
    )
}

/// 3 枚のタイルを並べたグリッド（1 枚目のみ `featured`）。`tall` が
/// `true`（C 形）のとき行高を広げる修飾 class を添える。
fn grid(tall: bool, per_tile_button: bool) -> Node {
    let mut class = "blocks-category-mosaic-featured-grid".to_string();
    if tall {
        class.push_str(" blocks-category-mosaic-featured-grid--tall");
    }
    // B 形（`per_tile_button`）はヘッダー行（`H3`）を持たないため、タイル
    // 名見出しを `H3` に上げて見出しレベルスキップを避ける（モジュール doc
    // 「B 形タイル見出しのレベル」節参照）。A/C 形は `H3` ヘッダーの下に
    // 並ぶため `H4` のまま。
    let heading_level = if per_tile_button {
        HeadingLevel::H3
    } else {
        HeadingLevel::H4
    };
    let tiles: Vec<Node> = CATEGORIES
        .iter()
        .enumerate()
        .map(|(i, (name, src))| tile(name, src, i == 0, per_tile_button, heading_level))
        .collect();
    div(vec![("class", &class)], tiles)
}

/// A 基準形（対応表 ID R0821）。見出し + 全件リンク + 行動ラベルが `span`
/// のグリッド。
fn variant_a() -> Node {
    div(
        vec![("class", "blocks-category-mosaic-featured-section")],
        vec![
            header(
                "カテゴリから選ぶ",
                Some(link::root(
                    REPO,
                    &LinkProps::default(),
                    vec![],
                    vec![text("すべてのカテゴリ")],
                )),
            ),
            grid(false, false),
        ],
    )
}

/// B 見出しなし・タイルごとのボタン（対応表 ID R0037）。
fn variant_b() -> Node {
    div(
        vec![("class", "blocks-category-mosaic-featured-section")],
        vec![grid(false, true)],
    )
}

/// C 見出しと一覧ボタン・縦長（対応表 ID R0605）。
fn variant_c() -> Node {
    div(
        vec![("class", "blocks-category-mosaic-featured-section")],
        vec![
            header(
                "カテゴリから選ぶ",
                Some(button::button(
                    &ButtonProps {
                        variant: ButtonVariant::Plain,
                        disabled: true,
                        ..ButtonProps::default()
                    },
                    vec![("data-blocks-category-mosaic-featured-header-button", "")],
                    vec![text("一覧を見る")],
                )),
            ),
            grid(true, false),
        ],
    )
}

/// `category-mosaic-featured` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数（`crate::blocks` モジュール doc「静的表示」節）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-category-mosaic-featured-layout")],
        vec![
            variant_label("基準形（対応表 ID R0821。見出し + 全件リンク）"),
            variant_a(),
            variant_label("見出しなし・タイルごとのボタン（対応表 ID R0037）"),
            variant_b(),
            variant_label("見出しと一覧ボタン・縦長（対応表 ID R0605）"),
            variant_c(),
        ],
    )
}
```

## 原案差分メモ

- 集約元 3 件（R0821 / R0037 / R0605）はいずれも Demo 内で見た目の
  差分として並記しています。見出しと全件リンク（R0821）、見出しなし・
  タイルごとのボタン（R0037）、見出しと一覧ボタン・縦長画像
  （R0605）をそれぞれ別インスタンスとして掲載しています。
- イシュー本文が指定する 5 部品（`heading` / `link` / `link-overlay` /
  `image` / `button`）のみを使い、`text` 部品は使いません。形ラベル・
  タイル内の行動ラベルは素の `div`/`span` で出力しています。
- タイル全体を `link-overlay` の下に置き、実際の遷移リンクは `overlay`
  （DOM の最後、最前面）が一本だけ担う設計のため、タイル内の行動ラベル
  は `a`/`button` にせず装飾用の `span`（`aria-hidden="true"`）に
  留めています。同じ遷移先への入れ子リンク・二重リンクを避ける判断は
  `link-overlay` の rustdoc「入れ子リンクの前面化は非採用」節と同じです。
- R0037 のタイルごとのボタンは無 JS では動作しないため、
  `category-carousel`/`gallery-carousel` と同じ判断で常時 `disabled`
  にしています。R0037・R0605 のボタンは `ButtonVariant::Plain`（輪郭・
  背景なし）にしており、タイル全体が既にクリック可能な領域の内側に
  重なる以上、`Outline` のような縁取り付きの見た目が「別に押せる
  ボタンがある」という誤認を招くためです（codex-review #3498 P2 指摘）。
- B 形タイルボタン（画像・暗幕の上に重なる）は祖先の
  `color: var(--fandhe-color-bg)` を `color: inherit` で引き継ぎ、暗幕上
  でも読める文字色にしています（codex-review #3498 P1 指摘、
  `promo-collection-cards` の `cta-secondary` と同じ解法）。
- B 形はヘッダー行（`H3`）を持たないため、タイル名見出しを `H3` に
  上げています（A/C 形は `H3` ヘッダーの下に並ぶため `H4` のまま）。
  `H4` 固定のままだと周囲ページの見出し階層を飛び越える見出しレベル
  スキップになるためです（Bugbot 指摘）。
- R0605 の縦長の画像は、グリッドの行高（`grid-auto-rows`）を 14rem から
  20rem へ広げることで表現しています。
- モザイク配置（先頭タイルが 2 行分）は `grid-row: span 2` と
  `grid-auto-flow` 既定の自動配置の組み合わせで実現しており、明示的な
  `grid-template-areas` は使っていません。
- ダークテーマでは画像上の暗幕用トークン（`--fandhe-color-fg`/
  `--fandhe-color-bg`）が反転するため「明るい幕に暗い文字」になります。
  これは `promo-collection-cards` 等の既存 block と同じ既知の挙動です。
