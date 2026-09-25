//! `gallery-carousel` block（イシュー #2777。親トラッキング #2730
//! 「Blocks 目的別パーツ拡充ツリー」/ Phase 親 #2738「マーケティング A」
//! 配下、対応表 ID R0510（基準形）/ R0511（lg で 2 枚同時表示）/
//! R0512（lg で 3 枚同時表示）/ R0502（次の画像の端を半透明で覗かせる）の
//! 4 件を構造の参照元とする合成例。取得手段・ファイル名・内部コンポーネント
//! 識別子は記載しない（`docs/design/motion-reference-adoption-policy.md`
//! §9 と同じライセンス上の転記制限）。
//!
//! **Marketing / Gallery カテゴリで最初の block**（`super`（`gallery/mod.rs`）
//! 参照。カテゴリ雛形からの卒業も本 block が担う）。
//!
//! # 使用部品
//!
//! `badge` / `heading` / `text` / `carousel` / `image` / `button` の 6 部品
//! に加え、[`chevron`] の前後トリガーが空ボタンにならないよう `icon` も
//! 合成する（[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する。
//! 「原案差分メモ」節に採用理由を記載）。
//!
//! # レイアウト構造（`## Demo` 見出しに続く 4 インスタンス）
//!
//! 見出しブロック（タグライン `badge` + セクション見出し `heading H3` +
//! リード文 `text` + 基準形にのみ添える `button`）の下に、`carousel` を
//! 「prev-trigger | 画像表示領域 | next-trigger」の横一列で配置する
//! （headless `control` パーツが 3 要素を束ねる）。画像表示領域は
//! `.blocks-gallery-carousel-viewport`（`min-width: 0; overflow: hidden`）で
//! クリップした静止領域とし、その内側で headless `item-group` が
//! `carousel::item`（各 [`image::image`]）を横一列に並べる。無 JS の静的
//! 表示のため、`--fandhe-carousel-index` は既定値 `0`（headless `item_group`
//! 自由関数は `style` を出力しない）にフォールバックし、常に 1 枚目が
//! 選択済みの状態で描画される。無 JS のためスライド送りを実際には配線
//! できず、操作可能に見えるボタンが動作しないと誤解を招く
//! （レビュー指摘 P1）ため、`prev-trigger`・`next-trigger`・`indicator`
//! はいずれもネイティブ `disabled` + `data-disabled`（`indicator` は
//! ネイティブ `disabled` のみ、headless-ui にモジュールレベルの
//! `data-disabled` 出力はない）を伴い、常時操作不能な状態で描画される
//! （[`gallery`] 関数 doc 参照）。
//!
//! # 4 形の差分（集約元の対応表 ID を並記する理由）
//!
//! 静的な docs サイトのため、実際にドラッグ・クリックで送る挙動は
//! 示せない（`crate::blocks` モジュール doc「静的表示」節）。代わりに
//! `--fandhe-carousel-item-basis`（[`fandhe_frontend_pre_styled_ui::carousel`]
//! が新設した CSS カスタムプロパティ）の値を変えた 4 インスタンスを並べ、
//! 見た目の差分だけで挙動の違いを読み取れるようにする:
//!
//! - **A 基準形（R0510）**: `per-view` 属性値 `"1"`。既定の `basis: 100%`
//!   のまま、1 枚だけ表示する。
//! - **B lg で 2 枚同時表示（R0511）**: `per-view` 属性値 `"2"`。
//!   `>= 64rem`（[`LAYOUT_CSS`] 参照）で `basis: 50%` へ切り替える。
//! - **C lg で 3 枚同時表示（R0512）**: `per-view` 属性値 `"3"`。
//!   `>= 64rem` で `basis: 33.3333%` へ切り替える。
//! - **D 次の画像の端を半透明で覗かせる（R0502）**: `peek` 属性を持ち、
//!   `basis: 83.3333%` に固定して次スライドの端を常時のぞかせる。
//!   2 枚目（`index == 1`）のスライドにのみ `data-blocks-gallery-carousel-
//!   dim` を付与し `opacity` を下げる近似で「半透明」を表現する（実際の
//!   Embla 実装のような連続的なフェードは行わない、既知の近似）。
//!
//! # 前後ボタンを画像の外側へ配置する理由
//!
//! 参照元は前後ボタンを画像の上に重ねて配置するが、本 block は
//! `headless::carousel::control` が元々持つ横並びコンテナ構造をそのまま
//! 使い、`prev-trigger | viewport | next-trigger` を単純な `flex` 横並びに
//! する（イシュー本文の明示要件「前/次ボタンは画像の外側に置き、
//! カルーセルと横並びにする」）。画像へオーバーレイする絶対配置は行わない。
//!
//! # `chevron` アイコンと `icon` 部品の採用理由
//!
//! `prev_trigger`/`next_trigger` は子要素を渡さないと見た目が空の
//! ボタンになる。参照元の SVG path はコピーせず、
//! `feature_split_image::geo_icon` と同型の単純な山形アイコンを自前の
//! `d` で描く。トリガー自身が `aria-label` を持つため、アイコンは装飾
//! 扱い（`IconProps::default()` の `label: None` により `aria-hidden`）。
//! これにより使用部品が [`BLOCK`] の `parts` で 7 種（イシュー本文記載の
//! 6 部品 + Icon）になる。
//!
//! # `id`/`aria-labelledby` を出力しない理由
//!
//! [`carousel::root`] の `label` 引数は `aria-label` を直接出力するため
//! `id`/`aria-labelledby` の紐付けを必要としない。複数インスタンス
//! （4 形）を 1 ページに置いても id 重複が起きない
//! （`changelog_timeline`/`comparison_feature_rows` と同じ判断）。
//!
//! # スライド間の余白を `item` の padding で表現する理由
//!
//! [`fandhe_frontend_pre_styled_ui::carousel`] の recipe は
//! スライド間の gap を意図的に持たない（`translateX` の百分率計算が
//! 崩れるため、モジュール doc「参考サイト基準への調整」節参照）。本 block
//! では `item` slot 自身に `box-sizing: border-box` + `padding-inline` を
//! 与えることで、`flex-basis` に含まれる形で見た目の余白を作る
//! （[`LAYOUT_CSS`] 参照。`flex-basis` の幾何計算自体は変更しない）。
//!
//! # `<form>` を持たない・データ取得/送信を行わない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である。画像は [`dummy_assets`] のビルド時生成 SVG（相対パス）のみを
//! 使い、`data:` URI・外部 URL は使わない。文言はすべて架空のもの
//! （実在の企業名・人名・PII を含まない）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
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
        .map(|i| carousel::indicator(Orientation::Horizontal, i, i == 0, vec![("disabled", "")]))
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
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/gallery-carousel/",
    title: "gallery-carousel",
    category: BlockCategory::Gallery,
    rust_source: "crates/docs-site/src/blocks/marketing/gallery/gallery_carousel.rs",
    demo_class: "blocks-gallery-carousel",
    parts: &[
        Part {
            label: "Badge",
            path: "/themes/badge/",
        },
        Part {
            label: "Heading",
            path: "/themes/heading/",
        },
        Part {
            label: "Text",
            path: "/themes/text/",
        },
        Part {
            label: "Carousel",
            path: "/themes/carousel/",
        },
        Part {
            label: "Image",
            path: "/themes/image/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Icon",
            path: "/themes/icon/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `gallery_carousel` 固有のレイアウト規則。前後トリガーと viewport の
/// 横並び・viewport のクリップ・スライド間の余白（`item` padding）・
/// per-view/peek に応じた `--fandhe-carousel-item-basis` 切り替えを担う
/// （モジュール doc「4 形の差分」「スライド間の余白」節参照）。
///
/// レビュー指摘対応（P1）: `[data-part="indicator"]:disabled` の減光規則
/// を追加する。[`fandhe_frontend_pre_styled_ui::carousel`] は
/// `prev-trigger`/`next-trigger` の `[data-disabled]` 減光（canonical
/// `disabled_declarations()`）は持つが `indicator` 分は持たない
/// （pre-styled-ui 側は zag.js 同様 indicator に disabled 概念を
/// 持ち込まない設計のため）。本 block は `indicator` へネイティブ
/// `disabled` を独自付与する（[`gallery`] 関数 doc 参照）ため、対応する
/// 見た目の減光は block 固有 CSS 側で補う（pre-styled-ui 側の変更は
/// 本 PR のスコープ外）。
const LAYOUT_CSS: &str = "\
.blocks-gallery-carousel-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-10);\n  width: 100%;\n}\n\
.blocks-gallery-carousel-section {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n}\n\
.blocks-gallery-carousel-header {\n  display: flex;\n  flex-direction: column;\n  align-items: flex-start;\n  gap: var(--fandhe-space-3);\n}\n\
[data-scope=\"carousel\"][data-part=\"root\"][data-blocks-gallery-carousel-root] {\n  padding: var(--fandhe-space-1);\n}\n\
[data-scope=\"carousel\"][data-part=\"control\"][data-blocks-gallery-carousel-control] {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-gallery-carousel-viewport {\n  min-width: 0;\n  flex: 1;\n  overflow: hidden;\n}\n\
[data-scope=\"carousel\"][data-part=\"item\"][data-blocks-gallery-carousel-slide] {\n  box-sizing: border-box;\n  padding-inline: var(--fandhe-space-2);\n}\n\
[data-scope=\"image\"][data-part=\"root\"][data-blocks-gallery-carousel-image] {\n  display: block;\n  width: 100%;\n}\n\
[data-scope=\"carousel\"][data-part=\"indicator-group\"][data-blocks-gallery-carousel-indicators] {\n  margin-top: var(--fandhe-space-4);\n}\n\
[data-scope=\"carousel\"][data-part=\"indicator\"]:disabled {\n  opacity: 0.5;\n  cursor: not-allowed;\n}\n\
[data-scope=\"carousel\"][data-part=\"root\"][data-blocks-gallery-carousel-peek] {\n  --fandhe-carousel-item-basis: 83.3333%;\n}\n\
[data-scope=\"carousel\"][data-part=\"item\"][data-blocks-gallery-carousel-dim] {\n  opacity: 0.5;\n}\n\
@media (min-width: 64rem) {\n  [data-scope=\"carousel\"][data-part=\"root\"][data-blocks-gallery-carousel-per-view=\"2\"] {\n    --fandhe-carousel-item-basis: 50%;\n  }\n  [data-scope=\"carousel\"][data-part=\"root\"][data-blocks-gallery-carousel-per-view=\"3\"] {\n    --fandhe-carousel-item-basis: 33.3333%;\n  }\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, BLOCK, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// Demo は呼び出しごとに同一の `Node` を返す純関数であること
    /// （`crate::blocks` モジュール doc「静的表示」節）。
    #[test]
    fn demo_is_deterministic() {
        assert_eq!(render(&demo()), render(&demo()));
    }

    /// Demo が期待する 7 種の部品を出力すること。
    #[test]
    fn demo_composes_expected_parts() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"badge\"",
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"carousel\"",
            "data-scope=\"image\"",
            "data-scope=\"button\"",
            "data-scope=\"icon\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
    }

    /// 4 形 × 6 枚 = 24 枚の画像が出力されること。
    #[test]
    fn demo_renders_24_images_across_four_variants() {
        let html = render(&demo());
        assert_eq!(html.matches("<img").count(), 24);
    }

    /// carousel の WAI-ARIA carousel パターン（`aria-roledescription`）が
    /// 4 インスタンス分出力されること。
    #[test]
    fn demo_wires_four_carousel_instances() {
        let html = render(&demo());
        assert_eq!(html.matches("aria-roledescription=\"carousel\"").count(), 4);
    }

    /// 各インスタンスで 1 枚目が選択済み（`data-current`）で、対応する
    /// インジケーターも `aria-current` を持つこと（静的表示の不変条件）。
    #[test]
    fn demo_selects_first_slide_by_default() {
        let html = render(&demo());
        assert_eq!(html.matches("data-current").count(), 8);
        assert_eq!(html.matches("aria-current=\"true\"").count(), 4);
    }

    /// prev-trigger は先頭スライド固定のため、4 インスタンスすべてで
    /// 無効化されること。
    #[test]
    fn demo_disables_prev_trigger_on_every_instance() {
        let html = render(&demo());
        assert_eq!(html.matches("aria-label=\"前の画像\"").count(), 4);
        // `data-disabled` は prev-trigger・next-trigger の 2 パーツ ×
        // 4 インスタンス分（レビュー指摘 P1: 静的デモのため next-trigger
        // も常時無効化する、`gallery` 関数 doc 参照）。
        assert_eq!(html.matches("data-disabled").count(), 8);
    }

    /// レビュー指摘対応（P1）: 無 JS の静的デモでは next-trigger・
    /// indicator を操作可能に見せてはならないため、`next-trigger` も
    /// prev-trigger と同様に無効化され、全 `indicator` にネイティブ
    /// `disabled` が付与されること。
    #[test]
    fn demo_disables_next_trigger_and_all_indicators_on_every_instance() {
        let html = render(&demo());
        assert_eq!(html.matches("aria-label=\"次の画像\"").count(), 4);
        // indicator は 4 インスタンス合計 6 枚 × 4 = 24 件。
        let total_indicators: usize = [6, 6, 6, 6].iter().sum();
        assert_eq!(
            html.matches("data-scope=\"carousel\" data-part=\"indicator\"")
                .count(),
            total_indicators
        );
        // ネイティブ `disabled=""` は prev-trigger 4 + next-trigger 4 +
        // indicator 24 の合計 32 件出力される（headless-ui は indicator に
        // `data-disabled` を出力しない設計のため、`data-disabled` の総数
        //〔上のテストで検証済みの 8 件〕には indicator 分を含まない）。
        // 先頭に半角スペースを含めて検索し、`data-disabled=""` の末尾
        // 部分文字列（ハイフンの前に空白は無い）との誤マッチを避ける。
        assert_eq!(
            html.matches(" disabled=\"\"").count(),
            4 + 4 + total_indicators
        );
    }

    /// D 形（覗かせる）だけが半透明近似のフックをちょうど 1 件持つこと。
    #[test]
    fn demo_dims_exactly_one_slide_for_peek_variant() {
        let html = render(&demo());
        assert_eq!(
            html.matches("data-blocks-gallery-carousel-dim=\"\"")
                .count(),
            1
        );
    }

    /// 非対話・安全性の不変条件（`<form>`・`href="#"`・`data:` URI・
    /// `id=` 属性・`aria-labelledby` を持たないこと）。
    #[test]
    fn demo_never_contains_forbidden_markup() {
        let html = render(&demo());
        for absent in [
            "<form",
            "href=\"#\"",
            "src=\"data:",
            "id=\"",
            "aria-labelledby",
        ] {
            assert!(!html.contains(absent), "demo should never contain {absent}");
        }
    }

    /// ダミー画像素材 4 種すべてを使用していること。
    #[test]
    fn demo_uses_all_dummy_image_sources() {
        let html = render(&demo());
        for src in [
            crate::blocks::dummy_assets::PRODUCT_SRC,
            crate::blocks::dummy_assets::BACKGROUND_SRC,
            crate::blocks::dummy_assets::SCREENSHOT_SRC,
            crate::blocks::dummy_assets::LOGO_SRC,
        ] {
            assert!(html.contains(src), "demo should reference {src}");
        }
    }

    /// [`LAYOUT_CSS`] が viewport クリップ・lg ブレークポイント・peek 幅・
    /// 半透明宣言を持つこと。
    #[test]
    fn layout_css_declares_viewport_clip_and_per_view_rules() {
        assert!(LAYOUT_CSS.contains("overflow: hidden"));
        assert!(LAYOUT_CSS.contains("@media (min-width: 64rem)"));
        assert!(LAYOUT_CSS.contains("--fandhe-carousel-item-basis: 50%"));
        assert!(LAYOUT_CSS.contains("--fandhe-carousel-item-basis: 33.3333%"));
        assert!(LAYOUT_CSS.contains("--fandhe-carousel-item-basis: 83.3333%"));
        assert!(LAYOUT_CSS.contains("opacity"));
        assert!(!LAYOUT_CSS.contains('<'));
    }

    /// レイアウト用ルート class（`.blocks-gallery-carousel-layout`）が
    /// `demo_class`（`blocks-gallery-carousel`）と異なること（既存 block と
    /// 同じ Bugbot 教訓: デモ枠と配置レイアウトの class を分離する）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        assert_ne!(BLOCK.demo_class, "blocks-gallery-carousel-layout");
    }
}
