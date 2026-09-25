# gallery-carousel

`badge` / `heading` / `text` / `carousel` / `image` / `button` / `icon` の 7 部品を合成した、作品を 1 枚ずつ送るギャラリー用カルーセルです。見出しの下に、画像を 1 枚ずつ送るカルーセルを配置します。

- 静的表示です。docs サイトは無 JS のため、状態機械・フォーム・データ取得を持たない固定描画にしています。1 枚目が選択済みの状態で描画し、実際にはスライドを送れないため前/次ボタン・インジケーターはいずれも無効状態で示します。
- 前/次ボタンは画像の外側に置き、カルーセルと横一列に横並びにしています（画像へ重ねる配置にはしていません）。
- 4 つの形を並べて、集約元の差分を読み取れるようにしています。
  - 1 枚ずつ送る基準形（対応表 ID R0510）
  - lg（`64rem`）以上で 2 枚同時表示する形（対応表 ID R0511）
  - lg 以上で 3 枚同時表示する形（対応表 ID R0512）
  - 次の画像の端を半透明で覗かせる形（対応表 ID R0502）
- 画像はビルド時生成のダミー素材（商品・背景・スクリーンショット・ロゴのプレースホルダー SVG）を使い回しています。`data:` URI・外部 URL は使いません。
- データ取得・送信は行わず、`<form>` は使いません。文言はすべて架空のものです。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::carousel::{self, Orientation};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::image::{image, AspectRatio, ImageFit, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};
use fandhe_frontend_pre_styled_ui::Size;

/// 各形の直前に置く短い形ラベル（`feature_split_image::variant_label` と
/// 同型）。
fn variant_label(label: &'static str) -> Node {
    styled_text::text(
        &TextProps {
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![],
        vec![text(label)],
    )
}

/// 自作の幾何アイコン（線画。`feature_split_image::geo_icon` と同型。
/// `path` へ `fill="none"` + `stroke="currentColor"` を明示し、`icon` の
/// `<svg>` 側が固定で持つ塗り面を上書きして線画として描画する）。
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

/// 左向き山形（prev-trigger 用）。
fn chevron_left() -> Node {
    chevron("M15 18l-6-6 6-6")
}

/// 右向き山形（next-trigger 用）。
fn chevron_right() -> Node {
    chevron("M9 18l6-6-6-6")
}

/// 見出しブロック（タグライン `badge` + セクション見出し + リード文 +
/// 任意の CTA `button`）。`with_action` は基準形にのみ `true` を渡し、
/// 使用部品の一覧に `button` を実際に登場させる。
///
/// レビュー指摘対応（P1、PR #3215 codex 再指摘）: 本 Demo は無 JS の
/// 静的合成例であり `<form>`・実際の遷移先を持たない
/// （モジュール doc「`<form>` を持たない・データ取得/送信を行わない」節）。
/// `gallery` 関数の `prev-trigger`/`next-trigger`/`indicator` と同じ理由
/// （動作しないインタラクション要素をクリック可能に見せない）で、この
/// CTA ボタンにも `disabled: true` を渡し、常時操作不能な状態で描画する。
fn header(
    eyebrow: &'static str,
    title: &'static str,
    body: &'static str,
    with_action: bool,
) -> Node {
    let mut children: Vec<Node> = vec![
        badge::badge(&BadgeProps::default(), vec![], vec![text(eyebrow)]),
        heading(
            HeadingLevel::H3,
            &HeadingProps {
                size: HeadingSize::Xl2,
                weight: HeadingWeight::Bold,
            },
            vec![],
            vec![text(title)],
        ),
        styled_text::text(&TextProps::default(), vec![], vec![text(body)]),
    ];
    if with_action {
        children.push(button::button(
            &ButtonProps {
                variant: ButtonVariant::Outline,
                disabled: true,
                ..ButtonProps::default()
            },
            vec![],
            vec![text("すべての作品を見る")],
        ));
    }
    div(vec![("class", "blocks-gallery-carousel-header")], children)
}

/// スライド 1 枚分（`carousel::item` + [`image::image`]）。`dim` が
/// `true` のとき半透明近似用の `data-blocks-gallery-carousel-dim` を
/// 付与する（D 形の 2 枚目のみ）。
///
/// レビュー指摘対応（P2）: 作品ギャラリーの画像は装飾ではなく
/// カルーセルの主要コンテンツであるため、`blog_grid_image::article_card`
/// と同型の判断で `alt` を空文字列にしない。個々の作品を区別できる
/// 実データを持たない静的デモのため、1-origin の連番を差し込んだ
/// `"作品{n}の画像"` を alt として与える（実企業名・PII は含まない）。
fn slide(index: usize, count: usize, dim: bool) -> Node {
    let sources = [
        dummy_assets::PRODUCT_SRC,
        dummy_assets::BACKGROUND_SRC,
        dummy_assets::SCREENSHOT_SRC,
        dummy_assets::LOGO_SRC,
    ];
    let src = sources[index % sources.len()];
    let alt = format!("作品{}の画像", index + 1);
    let mut attrs: Vec<(&str, &str)> = vec![("data-blocks-gallery-carousel-slide", "")];
    if dim {
        attrs.push(("data-blocks-gallery-carousel-dim", ""));
    }
    carousel::item(
        Orientation::Horizontal,
        index,
        count,
        index == 0,
        attrs,
        vec![image(
            &ImageProps {
                fit: ImageFit::Cover,
                aspect_ratio: AspectRatio::Video,
                shape: ImageShape::Rounded,
                ..ImageProps::new(src, &alt)
            },
            vec![("data-blocks-gallery-carousel-image", "")],
        )],
    )
}

/// carousel 1 個分（control 行 + indicator 群）を組み立てる。`label` は
/// [`carousel::root`] の `aria-label` に渡す意味のある文言、`count` は
/// スライド枚数、`variant_attr` は per-view/peek を表す `data-*`、
/// `dim_next` は D 形専用（2 枚目を半透明近似にする）フラグ。
///
/// レビュー指摘対応（P1）: docs サイトは無 JS でスライド送りを実装
/// しない（モジュール doc「`<form>` を持たない・データ取得/送信を行わ
/// ない」節）ため、`next-trigger`・`indicator` を操作可能に見える状態
/// のまま放置すると、クリック・キーボード操作をしても表示が変わらない
/// 動作しないインタラクション要素になってしまう。`prev-trigger` が
/// 既に「先頭スライドで disabled」という headless 契約上の理由で
/// ネイティブ `disabled` + `data-disabled` を出力していたことに揃え、
/// `next-trigger` にも常時 `disabled: true` を渡し、`indicator` にも
/// 呼び出し側 `attrs`（headless-ui `RESERVED` に含まれない）経由で
/// ネイティブ `disabled` を明示付与する。これにより全トリガー・
/// インジケーターがキーボード操作も含めて実際に操作不能になり、
/// 静的デモであることが見た目（`disabled_declarations()` の減光）と
/// 挙動の両面で伝わる。
fn gallery(
    label: &'static str,
    count: usize,
    variant_attr: (&'static str, &'static str),
    dim_next: bool,
) -> Node {
    let slides: Vec<Node> = (0..count)
        .map(|i| slide(i, count, dim_next && i == 1))
        .collect();
    let indicators: Vec<Node> = (0..count)
        .map(|i| {
            carousel::indicator(
                Orientation::Horizontal,
                i,
                i == 0,
                vec![
                    ("disabled", ""),
                    ("data-blocks-gallery-carousel-indicator", ""),
                ],
            )
        })
        .collect();

    carousel::root(
        Size::Md,
        Orientation::Horizontal,
        label,
        vec![("data-blocks-gallery-carousel-root", ""), variant_attr],
        vec![
            carousel::control(
                Orientation::Horizontal,
                vec![("data-blocks-gallery-carousel-control", "")],
                vec![
                    carousel::prev_trigger(
                        Orientation::Horizontal,
                        true,
                        "前の画像",
                        vec![],
                        vec![chevron_left()],
                    ),
                    div(
                        vec![("class", "blocks-gallery-carousel-viewport")],
                        vec![carousel::item_group(
                            Orientation::Horizontal,
                            vec![],
                            slides,
                        )],
                    ),
                    carousel::next_trigger(
                        Orientation::Horizontal,
                        true,
                        "次の画像",
                        vec![],
                        vec![chevron_right()],
                    ),
                ],
            ),
            carousel::indicator_group(
                Orientation::Horizontal,
                vec![("data-blocks-gallery-carousel-indicators", "")],
                indicators,
            ),
        ],
    )
}

/// A 基準形（対応表 ID R0510）。
fn variant_basic() -> Node {
    div(
        vec![("class", "blocks-gallery-carousel-section")],
        vec![
            header(
                "ギャラリー",
                "作品を 1 点ずつ紹介",
                "厳選した作品を 1 枚ずつ送って紹介します。",
                true,
            ),
            gallery(
                "作品ギャラリー（1 枚表示）",
                6,
                ("data-blocks-gallery-carousel-per-view", "1"),
                false,
            ),
        ],
    )
}

/// B lg で 2 枚同時表示（対応表 ID R0511）。
fn variant_two_up() -> Node {
    div(
        vec![("class", "blocks-gallery-carousel-section")],
        vec![
            header(
                "ギャラリー",
                "広い画面では 2 点並べて表示",
                "lg 以上の画面幅では 2 点を同時に表示します。",
                false,
            ),
            gallery(
                "作品ギャラリー（2 枚表示）",
                6,
                ("data-blocks-gallery-carousel-per-view", "2"),
                false,
            ),
        ],
    )
}

/// C lg で 3 枚同時表示（対応表 ID R0512）。
fn variant_three_up() -> Node {
    div(
        vec![("class", "blocks-gallery-carousel-section")],
        vec![
            header(
                "ギャラリー",
                "広い画面では 3 点並べて表示",
                "lg 以上の画面幅では 3 点を同時に表示します。",
                false,
            ),
            gallery(
                "作品ギャラリー（3 枚表示）",
                6,
                ("data-blocks-gallery-carousel-per-view", "3"),
                false,
            ),
        ],
    )
}

/// D 次の画像の端を半透明で覗かせる（対応表 ID R0502）。
fn variant_peek() -> Node {
    div(
        vec![("class", "blocks-gallery-carousel-section")],
        vec![
            header(
                "ギャラリー",
                "次の作品をちらりと見せる",
                "次に表示される作品の端を薄く覗かせます。",
                false,
            ),
            gallery(
                "作品ギャラリー（次の画像を覗かせる表示）",
                6,
                ("data-blocks-gallery-carousel-peek", ""),
                true,
            ),
        ],
    )
}

/// `gallery-carousel` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数（`crate::blocks` モジュール doc「静的表示」節）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-gallery-carousel-layout")],
        vec![
            variant_label("1 枚ずつ送る（対応表 ID R0510 基準形）"),
            variant_basic(),
            variant_label("lg 以上で 2 枚同時表示（対応表 ID R0511）"),
            variant_two_up(),
            variant_label("lg 以上で 3 枚同時表示（対応表 ID R0512）"),
            variant_three_up(),
            variant_label("次の画像の端を半透明で覗かせる（対応表 ID R0502）"),
            variant_peek(),
        ],
    )
}
```

## 原案差分メモ

- 参照元は前後ボタンを画像の上に重ねて配置していましたが、画像の外側の横並び（`prev-trigger | 画像表示領域 | next-trigger`）へ簡略化しました。headless の `control` パーツが元々持つ横並びコンテナ構造をそのまま利用しています。
- スライドの送りは、`fandhe-frontend-pre-styled-ui` の carousel recipe が持つ CSS カスタムプロパティ `--fandhe-carousel-index` の既定値 `0` による 1 枚目固定です。JS によるドラッグ・クリック送りは docs サイトのスコープ外（無 JS 制約）のため実演しません。
- lg（`64rem`）以上での 2 枚・3 枚同時表示は、carousel recipe の `--fandhe-carousel-item-basis`（既定 `100%`）を `50%`/`33.3333%` へ切り替えることで表現しています。
- 「次の画像の端を半透明で覗かせる」形は、`--fandhe-carousel-item-basis` を `83.3333%` に固定し、2 枚目のスライドにだけ `opacity: 0.5` を与える近似です。実際の連続的なフェード効果は再現していません。
- 2 枚・3 枚同時表示の形では、末尾側の表示領域に空白が残り得ます（carousel の `index` が `0..件数-1` を素朴に走査するのみで、末尾のクランプ調整を持たないための既知の制約です）。
- 前後トリガーには子要素が必要なため、参照元の SVG を使わず自作の単純な山形アイコン（`icon` 部品）を追加しました。使用部品がイシュー本文記載の 6 部品に `icon` を加えた 7 部品になっています。
- スライド間の余白は、`item` スロット自身の `padding-inline` で表現しています（carousel recipe はスライド間の gap を意図的に持たないため、`flex-basis` の幾何計算を崩さない形にしています）。
- 画像はビルド時生成のダミー素材ヘルパ（商品・背景・スクリーンショット・ロゴの 4 種）を巡回して使い回しています。
- `id`/`aria-labelledby` は使わず、carousel の `label` 引数（`aria-label` に直接出力）だけで各インスタンスを区別しています。
- 作品画像の `alt` は空文字列にせず「作品{n}の画像」を与えています（装飾ではなく主要コンテンツのため）。
- 無 JS の静的デモではスライド送りを実装できないため、`next-trigger`・`indicator` も `prev-trigger` と同様にネイティブ `disabled` を付与し、操作しても表示が変わらない要素として見せないようにしています。
- 基準形の CTA「すべての作品を見る」も遷移先・クリック処理を持たない静的デモのため、他のインタラクション要素と同様に `disabled: true` を渡して操作不能な状態で描画しています。
- `indicator` の disabled 減光 CSS（`[data-part="indicator"]:disabled`）は、`blocks.css` が全 block 共通のスタイルシートであるため、本 block 固有の `data-blocks-gallery-carousel-indicator` 属性でスコープし、他の block の disabled indicator へ波及しないようにしています。
