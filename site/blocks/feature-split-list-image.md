# feature-split-list-image

`badge` / `heading` / `text` / `icon` / `card` / `image` / `button` /
`data-list` の 8 部品を合成した、左列に見出し・feature 一覧、右列に画像
（1 枚または複数枚の組）を置く 2 列 feature セクションです（ただしインス
タンス B のみ lg 以上で左右が入れ替わり、画像が左列・feature 一覧が右列
になります）。

4 つの静的インスタンスを縦に並べています。インスタンス A はアイコン付き
カード一覧（4 件・2 列）+ 縦長画像 1 枚、インスタンス B はアイコン付きの
行一覧（3 件）+ 正方形画像 1 枚（この 1 件だけ DOM 順が「画像 → 一覧」で、
lg 以上では画像が左列・一覧が右列になります）、インスタンス C は数値を
主役にしたカード（2 件）+ ボタン行 + 正方形画像 1 枚、インスタンス D は
定義リスト（4 件）+ 画像 4 枚の 2×2 グリッドです。
幅 lg（64rem）未満は一覧・画像を 1 列に畳み、lg 以上で 2 列へ切り替わり
ます。文言・データはすべて架空のもので、データ取得・送信は行わない静的
な表示例です。`<form>` は使用せず、ボタンは `type="button"` のまま送信先
を持ちません。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, text as core_text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::data_list::{self, DataListOrientation, DataListProps};
use fandhe_frontend_pre_styled_ui::heading::{
    self, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::image::{self, AspectRatio, ImageFit, ImageProps};
use fandhe_frontend_pre_styled_ui::text::{
    self as styled_text, TextProps, TextSize, TextVariant, TextWeight,
};
use fandhe_frontend_pre_styled_ui::Size;

/// 装飾用の自作幾何アイコン（lucide 等の既存アイコンセットの path を
/// 複製しないための単純図形、`careers_card_grid::geo_icon` と同型の判断）。
fn geo_icon(size: Size, path_d: &'static str) -> Node {
    icon(
        &IconProps {
            size,
            ..IconProps::default()
        },
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

/// 円の幾何アイコン（インスタンス A のカード見出しに添える）。
fn circle_icon() -> Node {
    geo_icon(Size::Md, "M12 3a9 9 0 100 18 9 9 0 000-18z")
}

/// 四角の幾何アイコン（インスタンス A のカード見出しに添える）。
fn square_icon() -> Node {
    geo_icon(Size::Md, "M4 4h16v16H4z")
}

/// 折れ線の幾何アイコン（インスタンス A のカード見出しに添える）。
fn zigzag_icon() -> Node {
    geo_icon(Size::Md, "M4 18l5-9 4 6 4-8 3 5")
}

/// 三角の幾何アイコン（インスタンス A のカード見出しに添える）。
fn triangle_icon() -> Node {
    geo_icon(Size::Md, "M12 4l8 16H4z")
}

/// 盾の幾何アイコン（インスタンス B のアイコン付き行に添える）。
fn shield_icon() -> Node {
    geo_icon(Size::Sm, "M12 3l7 3v5c0 5-3.5 8.5-7 10-3.5-1.5-7-5-7-10V6z")
}

/// 稲妻の幾何アイコン（インスタンス B のアイコン付き行に添える）。
fn bolt_icon() -> Node {
    geo_icon(Size::Sm, "M13 3L5 14h5l-1 7 8-11h-5z")
}

/// 歯車の幾何アイコン（インスタンス B のアイコン付き行に添える）。
fn gear_icon() -> Node {
    geo_icon(
        Size::Sm,
        "M12 8a4 4 0 100 8 4 4 0 000-8z M12 2v3 M12 19v3 M4.2 4.2l2.1 2.1 M17.7 17.7l2.1 2.1 M2 12h3 M19 12h3 M4.2 19.8l2.1-2.1 M17.7 6.3l2.1-2.1",
    )
}

/// 星の幾何アイコン（インスタンス D の定義リストに添える）。
fn star_icon() -> Node {
    geo_icon(
        Size::Sm,
        "M12 3l2.6 5.6 6.1.6-4.6 4.1 1.3 6-5.4-3.2-5.4 3.2 1.3-6-4.6-4.1 6.1-.6z",
    )
}

/// 波の幾何アイコン（インスタンス D の定義リストに添える）。
fn wave_icon() -> Node {
    geo_icon(Size::Sm, "M3 12c2-3 4-3 6 0s4 3 6 0 4-3 6 0")
}

/// 錠前の幾何アイコン（インスタンス D の定義リストに添える）。
fn lock_icon() -> Node {
    geo_icon(Size::Sm, "M6 11V8a6 6 0 1112 0v3 M5 11h14v9H5z")
}

/// 地球の幾何アイコン（インスタンス D の定義リストに添える）。
fn globe_icon() -> Node {
    geo_icon(
        Size::Sm,
        "M12 3a9 9 0 100 18 9 9 0 000-18z M3 12h18 M12 3c2.5 2.5 2.5 15.5 0 18 M12 3c-2.5 2.5-2.5 15.5 0 18",
    )
}

/// feature 一覧カード 1 件分の架空データ（インスタンス A）。
struct FeatureItem {
    icon_node: fn() -> Node,
    title: &'static str,
    body: &'static str,
}

/// インスタンス A のカードデータ（4 件）。
const FEATURES_A: [FeatureItem; 4] = [
    FeatureItem {
        icon_node: circle_icon,
        title: "決定的なビルド",
        body: "同じ入力からは常に同じ静的ファイルを生成します。",
    },
    FeatureItem {
        icon_node: square_icon,
        title: "型で表現する構造",
        body: "スロットと props は Rust の型で表現されます。",
    },
    FeatureItem {
        icon_node: zigzag_icon,
        title: "外部依存ゼロ",
        body: "描画コアは外部クレートに依存しません。",
    },
    FeatureItem {
        icon_node: triangle_icon,
        title: "単一実行ファイル配布",
        body: "SSR/SSG のいずれも単一バイナリへまとめられます。",
    },
];

/// インスタンス B のアイコン付き行データ（3 件）。
const FEATURES_B: [FeatureItem; 3] = [
    FeatureItem {
        icon_node: shield_icon,
        title: "既定エスケープ",
        body: "テキスト補間は既定でエスケープされます。",
    },
    FeatureItem {
        icon_node: bolt_icon,
        title: "無 JS の静的表示",
        body: "JS ハイドレーションを行わない決定的な表示です。",
    },
    FeatureItem {
        icon_node: gear_icon,
        title: "機械検証可能な構成",
        body: "構造マニフェストが依存関係を機械検証します。",
    },
];

/// インスタンス C の数値カードデータ（2 件）。
struct StatItem {
    value: &'static str,
    title: &'static str,
    body: &'static str,
}

const STATS_C: [StatItem; 2] = [
    StatItem {
        value: "14",
        title: "公開クレート",
        body: "描画コアから CLI までを個別クレートへ分割しています。",
    },
    StatItem {
        value: "60",
        title: "依存上限",
        body: "標準構成の依存パッケージ数は 60 件以内に収めます。",
    },
];

/// インスタンス D の定義リストデータ（4 件）。
struct DefinitionItem {
    icon_node: fn() -> Node,
    term: &'static str,
    detail: &'static str,
}

const DEFINITIONS_D: [DefinitionItem; 4] = [
    DefinitionItem {
        icon_node: star_icon,
        term: "既定エスケープ",
        detail: "迂回経路は明示的なオプトイン API に限られます。",
    },
    DefinitionItem {
        icon_node: wave_icon,
        term: "決定的な出力",
        detail: "同じ入力からは常に同じ静的ファイルを生成します。",
    },
    DefinitionItem {
        icon_node: lock_icon,
        term: "unsafe 境界の限定",
        detail: "描画コア・状態管理コアでは unsafe を使用しません。",
    },
    DefinitionItem {
        icon_node: globe_icon,
        term: "単一バイナリ配布",
        detail: "SSR/SSG を単一実行ファイルへまとめて配布できます。",
    },
];

/// セクション見出し（badge + H3 + リード文）を組み立てる。
fn section_header(eyebrow: &str, title: &str, lead: &str) -> Node {
    div(
        vec![("class", "blocks-feature-split-list-image-header")],
        vec![
            badge::badge(
                &BadgeProps::default(),
                vec![("data-blocks-feature-split-list-image-eyebrow", "")],
                vec![core_text(eyebrow)],
            ),
            heading::heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    weight: HeadingWeight::Bold,
                },
                vec![],
                vec![core_text(title)],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![("data-blocks-feature-split-list-image-lead", "")],
                vec![core_text(lead)],
            ),
        ],
    )
}

/// インスタンス A の feature カード 1 枚（アイコン + H4 見出し + 説明）。
fn feature_card(item: &FeatureItem) -> Node {
    card::root(
        CardProps::default(),
        vec![("data-blocks-feature-split-list-image-card", "")],
        vec![card::body(
            vec![("class", "blocks-feature-split-list-image-card-body")],
            vec![
                (item.icon_node)(),
                heading::heading(
                    HeadingLevel::H4,
                    &HeadingProps {
                        size: HeadingSize::Sm,
                        weight: HeadingWeight::Semibold,
                    },
                    vec![],
                    vec![core_text(item.title)],
                ),
                styled_text::text(
                    &TextProps {
                        size: TextSize::Sm,
                        variant: TextVariant::Muted,
                        ..TextProps::default()
                    },
                    vec![("data-blocks-feature-split-list-image-card-desc", "")],
                    vec![core_text(item.body)],
                ),
            ],
        )],
    )
}

/// インスタンス B のアイコン付き行 1 件（アイコン列 + (H4 + 説明)）。
fn feature_row(item: &FeatureItem) -> Node {
    div(
        vec![("class", "blocks-feature-split-list-image-row")],
        vec![
            div(
                vec![("data-blocks-feature-split-list-image-row-icon", "")],
                vec![(item.icon_node)()],
            ),
            div(
                vec![("class", "blocks-feature-split-list-image-row-text")],
                vec![
                    heading::heading(
                        HeadingLevel::H4,
                        &HeadingProps {
                            size: HeadingSize::Sm,
                            weight: HeadingWeight::Semibold,
                        },
                        vec![],
                        vec![core_text(item.title)],
                    ),
                    styled_text::text(
                        &TextProps {
                            size: TextSize::Sm,
                            variant: TextVariant::Muted,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![core_text(item.body)],
                    ),
                ],
            ),
        ],
    )
}

/// インスタンス C の数値カード 1 枚（大きな数値 + H4 見出し + 説明）。
fn stat_card(item: &StatItem) -> Node {
    card::root(
        CardProps::default(),
        vec![("data-blocks-feature-split-list-image-stat-card", "")],
        vec![card::body(
            vec![("class", "blocks-feature-split-list-image-card-body")],
            vec![
                styled_text::text(
                    &TextProps {
                        size: TextSize::Xl3,
                        weight: TextWeight::Bold,
                        ..TextProps::default()
                    },
                    vec![("data-blocks-feature-split-list-image-stat-value", "")],
                    vec![core_text(item.value)],
                ),
                heading::heading(
                    HeadingLevel::H4,
                    &HeadingProps {
                        size: HeadingSize::Sm,
                        weight: HeadingWeight::Semibold,
                    },
                    vec![],
                    vec![core_text(item.title)],
                ),
                styled_text::text(
                    &TextProps {
                        size: TextSize::Sm,
                        variant: TextVariant::Muted,
                        ..TextProps::default()
                    },
                    vec![],
                    vec![core_text(item.body)],
                ),
            ],
        )],
    )
}

/// インスタンス C 末尾のボタン行（送信先を持たない見た目のみの導線）。
fn stat_button_row() -> Node {
    div(
        vec![("class", "blocks-feature-split-list-image-button-row")],
        vec![
            button::button(
                &ButtonProps::default(),
                vec![],
                vec![core_text("詳しく見る")],
            ),
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Outline,
                    ..ButtonProps::default()
                },
                vec![],
                vec![core_text("資料をダウンロード")],
            ),
        ],
    )
}

/// インスタンス D の定義リスト 1 項目（アイコン + 短い見出し / 説明）。
fn definition_item(item: &DefinitionItem) -> Node {
    data_list::item(
        vec![],
        vec![
            data_list::item_label(
                vec![("class", "blocks-feature-split-list-image-dt")],
                vec![
                    (item.icon_node)(),
                    styled_text::text(
                        &TextProps {
                            weight: TextWeight::Bold,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![core_text(item.term)],
                    ),
                ],
            ),
            data_list::item_value(
                vec![],
                vec![styled_text::text(
                    &TextProps {
                        variant: TextVariant::Muted,
                        ..TextProps::default()
                    },
                    vec![],
                    vec![core_text(item.detail)],
                )],
            ),
        ],
    )
}

/// 画像 1 枚を組み立てる（`alt=""`、装飾扱い）。
fn split_image(src: &str, aspect_ratio: AspectRatio) -> Node {
    image::image(
        &ImageProps {
            fit: ImageFit::Cover,
            aspect_ratio,
            ..ImageProps::new(src, "")
        },
        vec![("data-blocks-feature-split-list-image-image", "")],
    )
}

/// インスタンス A・C・D 共通の「一覧 → 画像」2 列レイアウト。
fn split_list_then_image(list: Node, image_col: Node) -> Node {
    div(
        vec![("class", "blocks-feature-split-list-image-split")],
        vec![list, image_col],
    )
}

/// インスタンス B 専用の「画像 → 一覧」2 列レイアウト（本モジュール doc
/// 「DOM 順と視覚順」節）。
fn split_image_then_list(image_col: Node, list: Node) -> Node {
    div(
        vec![("class", "blocks-feature-split-list-image-split")],
        vec![image_col, list],
    )
}

/// インスタンス A（カード一覧 + 縦長画像 1 枚）。
fn instance_a() -> Node {
    let header = section_header(
        "特長",
        "カード一覧と画像で特長を紹介する",
        "各カードはアイコン・見出し・説明の順に積み、画面幅に応じて 1〜2 列へ切り替わります。",
    );
    let grid = div(
        vec![("class", "blocks-feature-split-list-image-card-grid")],
        FEATURES_A.iter().map(feature_card).collect(),
    );
    let list = div(
        vec![("class", "blocks-feature-split-list-image-list")],
        vec![header, grid],
    );
    let image_col = split_image(dummy_assets::PRODUCT_SRC, AspectRatio::Portrait);
    div(
        vec![("class", "blocks-feature-split-list-image-instance")],
        vec![split_list_then_image(list, image_col)],
    )
}

/// インスタンス B（アイコン付き行 3 件 + 正方形画像 1 枚、DOM 順は画像が先）。
fn instance_b() -> Node {
    let header = section_header(
        "できること",
        "アイコン付きの一覧で要点を伝える",
        "各行はアイコン・短い見出し・説明の 3 要素で揃えています。",
    );
    let rows = div(
        vec![("class", "blocks-feature-split-list-image-row-list")],
        FEATURES_B.iter().map(feature_row).collect(),
    );
    let list = div(
        vec![("class", "blocks-feature-split-list-image-list")],
        vec![header, rows],
    );
    let image_col = split_image(dummy_assets::SCREENSHOT_SRC, AspectRatio::Square);
    div(
        vec![("class", "blocks-feature-split-list-image-instance")],
        vec![split_image_then_list(image_col, list)],
    )
}

/// インスタンス C（数値カード 2 枚 + ボタン行 + 正方形画像 1 枚）。
fn instance_c() -> Node {
    let header = section_header(
        "実績",
        "数値で語る特長とボタン導線",
        "数値カードの下に、資料へのボタン導線を並べています。",
    );
    let grid = div(
        vec![("class", "blocks-feature-split-list-image-card-grid")],
        STATS_C.iter().map(stat_card).collect(),
    );
    let list = div(
        vec![("class", "blocks-feature-split-list-image-list")],
        vec![header, grid, stat_button_row()],
    );
    let image_col = split_image(dummy_assets::BACKGROUND_SRC, AspectRatio::Square);
    div(
        vec![("class", "blocks-feature-split-list-image-instance")],
        vec![split_list_then_image(list, image_col)],
    )
}

/// インスタンス D（定義リスト 4 項目 + 画像 4 枚の 2×2 グリッド）。
fn instance_d() -> Node {
    let header = section_header(
        "詳細",
        "定義リストと画像の組で詳細を示す",
        "用語ごとの短い説明を、画像 4 枚の組と並べています。",
    );
    let items: Vec<Node> = DEFINITIONS_D.iter().map(definition_item).collect();
    let dl = data_list::root(
        DataListProps {
            orientation: DataListOrientation::Vertical,
            ..DataListProps::default()
        },
        vec![],
        items,
    );
    let list = div(
        vec![("class", "blocks-feature-split-list-image-list")],
        vec![header, dl],
    );
    let image_grid = div(
        vec![("class", "blocks-feature-split-list-image-image-grid")],
        vec![
            split_image(dummy_assets::PRODUCT_SRC, AspectRatio::Square),
            split_image(dummy_assets::SCREENSHOT_SRC, AspectRatio::Square),
            split_image(dummy_assets::BACKGROUND_SRC, AspectRatio::Square),
            split_image(dummy_assets::LOGO_SRC, AspectRatio::Square),
        ],
    );
    div(
        vec![("class", "blocks-feature-split-list-image-instance")],
        vec![split_list_then_image(list, image_grid)],
    )
}

/// 各インスタンスに添える差分注記（`feature_image_cards` の `note_a`/
/// `note_b` と同じ形）。
fn note(body: &str) -> Node {
    styled_text::text(
        &TextProps {
            size: TextSize::Xs,
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![],
        vec![core_text(body)],
    )
}

/// `feature-split-list-image` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数（他 block と同じ状態を持たない設計）。
#[must_use]
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-feature-split-list-image-layout")],
        vec![
            note("カード一覧（4 件・2 列）+ 縦長画像 1 枚。一覧 → 画像の順。"),
            instance_a(),
            note("アイコン付き行（3 件）+ 正方形画像 1 枚。画像 → 一覧の順（狭幅で画像が上、lg で画像が左）。"),
            instance_b(),
            note("数値カード（2 件）+ ボタン行 + 正方形画像 1 枚。一覧 → 画像の順。"),
            instance_c(),
            note("定義リスト（4 件）+ 画像 4 枚の 2×2 グリッド。一覧 → 画像の順。"),
            instance_d(),
        ],
    )
}
```

## 原案差分メモ

参照（対応表 ID R0486・R0105・R0102・R0484・R0477・R0489・R0487・
R1155・R1157・R1161。出典の固有名・ファイル名は記載しません）からの
意図的な差分は次のとおりです。

- R0486 を基準形（左列一覧 + 右列画像の 2 列 feature セクション）とし、
  インスタンス A にしました。
- R0105 の「狭幅で画像が上・lg で画像が右」は DOM 順を変えずに済む
  インスタンス A・C・D の並びで代替し、CTA（ボタン導線）はインスタンス C
  へ集約しました。
- R0102・R0484・R0477 を統合してインスタンス B（アイコン付き行一覧）に
  しました。R0477 が持つ縦長画像は、インスタンス A で既に使っているため
  インスタンス B では正方形画像で代替しています。DOM 順のみ「画像 →
  一覧」に入れ替え、狭幅で画像が上・lg で画像が左になるようにしました。
- R0489・R0487 を統合してインスタンス C（数値カード + ボタン行）にしま
  した。
- R1155（6 項目）・R1157（全高画像）・R1161（大小 3 枚の段違い画像）を
  統合し、インスタンス D では項目数を 4 件へ縮約し、画像は均等な 2×2
  グリッド（4 枚）で代替しました。
- 参照元の背景帯・装飾・実際の文言は持ち込まず、文言はすべて独自の架空
  のもの（日本語）にしました。
- 見出しは `h3`/`h4` に下げました（ページ側が `## Demo` として `h2` を
  出すため）。インスタンス D の定義リストの短い見出しは、`dt` が見出し
  要素を子孫に持てないため太字テキストで表しています。
- 画像は `dummy_assets` のプレースホルダーと `alt=""`（装飾扱い）にしま
  した。
- 文言・配色・余白は独自のもの、または既存のテーマトークンにそのまま
  従います。
