//! `feature-accordion-image` block（イシュー #2761。親イシュー #2760
//! 「feature-accordion-image を Blocks に追加する」配下、規模 L のため
//! 前半 #2761（本コミット、骨格・主要領域）と後半 #2762（カテゴリ切替
//! ボタン列・状態違いの並記・原稿仕上げ）へ分割済み。集約元は対応表 ID
//! R0103（基準形。本イシューで実装）と R0483（上部にカテゴリ切替ボタン列が
//! 付く形、#2762 で実装）の 2 件。取得手段・ファイル名・内部コンポーネント
//! 識別子は記載しない（`docs/design/motion-reference-adoption-policy.md`
//! §9 と同じライセンス上の転記制限。記載してよいのは対応表 ID のみ）。
//!
//! # 使用部品
//!
//! `badge` / `heading` / `text` / `accordion` / `image` の 5 部品を合成する
//! （[`BLOCK`] の `parts` に一致させる契約、`crates/docs-site/tests/
//! blocks_nav.rs`/`blocks_contract.rs` が検証する）。`button`/`icon` は
//! カテゴリ切替ボタン列を実装する #2762 で使い始める（`parts` には実際に
//! 描画する部品だけを載せる、`bento_asymmetric_rows` が icon を後半から
//! 使い始めた前例と同じ判断）。
//!
//! # レイアウト
//!
//! 左列に見出しエリア（アイブロウ badge + heading + リード文）とアコー
//! ディオンを縦に並べ、右列に先頭機能の代表画像を 1 枚置く 2 列構成
//! （全項目が常時展開のため、右列は「選択中の項目に追従する画像」では
//! なく先頭機能を指す固定の代表画像として扱う）。`md`（[`Breakpoint::
//! Md`]、768px/48rem）未満では右列を隠し、各項目の本文の中にインライン
//! 画像を表示する（項目ごとに自分の画像を持つ）。
//!
//! # 静的アコーディオン（JS を使わない、全項目を常時展開で固定）
//!
//! docs サイトは JS ハイドレーションを行わないため、状態機械を経由せず
//! [`fandhe_frontend_pre_styled_ui::accordion`] の自由関数を直接呼び、
//! [`FEATURES`] の全件を [`OpenState::Open`] として固定描画する
//! （`changelog_accordion` イシュー #2818 と同型の設計。当初は 1 件目
//! のみ open・残りを `OpenState::Closed`（`item_content` 自体を
//! 出力しない）としていたが、閉じた項目の説明文が視覚利用者・支援技術の
//! いずれにも一切到達不能になり、かつ見出しリード文「項目を選ぶと」の
//! 案内文言が無 JS 下では実現しない操作を示唆してしまう不整合があった
//! （イシュー #2761 レビュー指摘。UI コンポーネント層のアクセシビリティ
//! 責務違反、`docs/policy/intentional-non-adoption.md` §3.25 参照）。
//! 是正として `careers_split_accordion` イシュー #2816・`changelog_
//! accordion` イシュー #2818 の前例に倣い、全件を open + disabled の
//! 「非操作の機能一覧」として再設計した。
//!
//! - [`AccordionProps`] の `disabled: true` を全パーツで共有する。これに
//!   より全 `item_trigger` がネイティブ `disabled` + `aria-disabled="true"`
//!   を持ち、フォーカス・操作ともに不能になる（押しても何も起きない
//!   ボタンを作らない）。`disabled` 由来の減光（`opacity: 0.5`）は
//!   [`LAYOUT_CSS`] で中和し、通常表示に戻す。
//! - 全項目が [`OpenState::Open`] であり、`item_trigger` に `id`/
//!   `controls` を持たせ、`item_content` を `id`/`labelled_by` 付きで
//!   出力する（閉じた項目・到達不能な本文は存在しない）。
//!
//! # トリガーの子を 2 個に保つ理由
//!
//! recipe の item-trigger は `justify-content: space-between` を前提に
//! 「直接の子 2 個」のレイアウトを取る（`changelog_accordion` と同じ）。
//! そのため [`item_trigger`] の children はラベル領域（`span`）と
//! `item_indicator` の 2 個に固定する。
//!
//! # 見出しレベル（`H3`/`h4`）
//!
//! ページ側が `## Demo` として `h2` を出すため、セクション見出しは
//! [`HeadingLevel::H3`] にする。各トリガーは WAI-ARIA APG のアコーディオン
//! パターンに合わせて `<h4>` で包む（[`fandhe_frontend_core::el`] で直接
//! 組み立て、`heading::heading` は使わない。ページ本文の見出し階層に
//! 割り込ませない部品固有の構造要素であるため）。
//!
//! # `text` の名前衝突
//!
//! `fandhe_frontend_pre_styled_ui::text::text`（`<p>` を組み立てる styled
//! パート関数）と `fandhe_frontend_core::text`（テキストノード生成関数）が
//! 同名のため、styled 側を `styled_text` として取り込む（`crate::blocks`
//! 内の他 block と同じ回避方法）。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `heading::heading` / `text::text` / `badge::badge` / `accordion::root` /
//! `image::image` はいずれも `drop_class_attr` により呼び出し側 `attrs` の
//! `class` を黙って除去する契約を持つため、Demo 固有のスタイルフックは
//! `data-blocks-feature-accordion-image-*` 属性で渡す。`accordion::item`/
//! `item_trigger`/`item_content`/`item_indicator` は `drop_class_attr` を
//! 経由しないため、素の `div`/`h4`/`span` と合わせて `class` か子孫
//! セレクタを使う（`changelog_accordion` と同じ判断）。
//!
//! # id 接頭辞
//!
//! `id`/`aria-controls`/`aria-labelledby` はいずれも
//! `blocks-feature-accordion-image-{index}-{trigger|content}` の形で項目の
//! 添字から一意に導出する（`crates/docs-site/tests/blocks_contract.rs::
//! demo_output_has_no_dangling_aria_references_or_duplicate_ids` が全
//! block 横断で id 重複・宙に浮いた参照を検査する）。
//!
//! # `<form>` を持たない・文言は架空
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である。機能名・説明文はすべて架空のもの（実在の製品・企業名・PII を
//! 含まない）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
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
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/feature-accordion-image/",
    title: "feature-accordion-image",
    category: BlockCategory::Feature,
    rust_source: "crates/docs-site/src/blocks/marketing/feature/feature_accordion_image.rs",
    demo_class: "blocks-feature-accordion-image",
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
            label: "Accordion",
            path: "/themes/accordion/",
        },
        Part {
            label: "Image",
            path: "/themes/image/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `feature_accordion_image` 固有のレイアウト規則（`crate::blocks::
/// LAYOUT_CSS` doc「block 固有 CSS の置き場」節。他 block と同型で
/// `pub(super)` ではなく本ファイル内 private 定数として `super::stylesheet`
/// 経由の `push_css` で連結される）。
///
/// モバイルファーストで書く: md（[`Breakpoint::Md`]、768px/48rem）未満は
/// 1 列とし、右列（`.blocks-feature-accordion-image-media-slot`）を隠して
/// 各項目のインライン画像（`[data-blocks-feature-accordion-image-inline-
/// image]`）を見せる。`@media (min-width: 48rem)` 以降は 2 列にし、右列を
/// 見せてインライン画像を隠す（本ファイル末尾の `#[cfg(test)]` が
/// `Breakpoint::Md` とのドリフトを検知する）。インライン画像は
/// `pre_styled_ui::image::image` の root 要素（`data-scope="image"
/// data-part="root"`）そのものに `data-blocks-feature-accordion-image-
/// inline-image` を併せ持たせているため、md+ で隠す規則は属性セレクタ
/// 単体（詳細度 (0,1,0)）ではなく `[data-scope="image"][data-part="root"]`
/// も併記した詳細度 (0,3,0) で書き、Image recipe の基底規則
/// `[data-scope="image"][data-part="root"] { display: block }`（詳細度
/// (0,2,0)）に確実に勝たせる（PR #3188 Bugbot 指摘で是正）。
///
/// 右列の代表画像（`[data-blocks-feature-accordion-image-media]`）にも
/// `width: 100%` を明示する。`image::image` の root は `max-width: 100%`
/// のみを持ち固有の横幅を指定しないため、`ImageProps::new` に渡す
/// スクリーンショット SVG の intrinsic size（`viewBox` 200x140）のまま
/// 描画され、md 以降で列いっぱいに広がらない（PR #3188 Bugbot 指摘で
/// 是正）。
///
/// `disabled` による減光の中和（モジュール doc「静的アコーディオン」節）:
/// `accordion::stylesheet` の `disabled_declarations()`（既定
/// `opacity: 0.5`）は「操作できない要素」の既定表現だが、本 block は
/// トリガー自体を disabled にしているだけで通常の機能一覧として見せる
/// ため、`opacity: 1`・`cursor: default` へ上書きする。詳細度はクラス
/// セレクタ 1 個 + 属性セレクタ 3 個（1,3,0）で recipe 側（0,2,0 相当）に
/// 確実に勝たせる（`changelog_accordion` と同じ考え方）。
///
/// セレクタはすべて `.blocks-feature-accordion-image*` か
/// `[data-blocks-feature-accordion-image-*]` の名前空間に閉じる。
/// `[data-scope="accordion"][data-part="item-trigger"][data-disabled]` の
/// 中和も名前空間の外に出さないよう、必ず `.blocks-feature-accordion-
/// image-left` 配下への子孫結合子付きで書く（集約された `blocks.css` を
/// 読み込む全ページで他 block の disabled accordion トリガーの見た目まで
/// 書き換えてしまわないため。`changelog_accordion` が `.blocks-changelog-
/// accordion-list` 配下へ子孫結合子付きで書くのと同じ判断、イシュー
/// #2761 レビュー指摘で是正）。
const LAYOUT_CSS: &str = "\
.blocks-feature-accordion-image-grid {\n  display: grid;\n  grid-template-columns: 1fr;\n  gap: var(--fandhe-space-8);\n  width: 100%;\n}\n\
.blocks-feature-accordion-image-left {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n  min-width: 0;\n}\n\
.blocks-feature-accordion-image-header {\n  display: flex;\n  flex-direction: column;\n  align-items: flex-start;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-feature-accordion-image-media-slot {\n  display: none;\n}\n\
[data-blocks-feature-accordion-image-media] {\n  display: block;\n  width: 100%;\n}\n\
[data-blocks-feature-accordion-image-inline-image] {\n  display: block;\n  width: 100%;\n  margin-top: var(--fandhe-space-3);\n}\n\
.blocks-feature-accordion-image-trigger-heading {\n  margin: 0;\n  font-size: inherit;\n  font-weight: inherit;\n}\n\
.blocks-feature-accordion-image-trigger-label {\n  flex: 1;\n  min-width: 0;\n  font-weight: var(--fandhe-font-weight-medium, 500);\n}\n\
.blocks-feature-accordion-image-body {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n  padding-top: var(--fandhe-space-1);\n}\n\
.blocks-feature-accordion-image-left [data-scope=\"accordion\"][data-part=\"item-trigger\"][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
@media (min-width: 48rem) {\n  .blocks-feature-accordion-image-grid {\n    grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);\n    align-items: start;\n  }\n  .blocks-feature-accordion-image-media-slot {\n    display: block;\n    position: sticky;\n    top: var(--fandhe-space-4);\n  }\n  [data-scope=\"image\"][data-part=\"root\"][data-blocks-feature-accordion-image-inline-image] {\n    display: none;\n  }\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, FEATURES, LAYOUT_CSS};
    use fandhe_frontend_core::render;
    use fandhe_frontend_pre_styled_ui::recipe::Breakpoint;

    /// [`LAYOUT_CSS`] の 48rem が `Breakpoint::Md.min_width()`（768px）と
    /// 実際に一致すること（モジュール doc「レイアウト規則」節が参照する
    /// 対応のドリフト検知）。
    #[test]
    fn layout_css_breakpoint_matches_pre_styled_ui_md() {
        assert_eq!(Breakpoint::Md.min_width(), "768px");
        assert!(LAYOUT_CSS.contains("@media (min-width: 48rem)"));
    }

    /// 全項目が `open` で、`closed` の項目が無いこと（モジュール doc
    /// 「静的アコーディオン」節、イシュー #2761 レビュー指摘の是正）。
    #[test]
    fn all_items_are_open() {
        let html = render(&demo());
        assert_eq!(
            html.matches(r#"data-part="item" data-state="open""#)
                .count(),
            FEATURES.len(),
            "html={html}"
        );
        assert_eq!(
            html.matches(r#"data-part="item" data-state="closed""#)
                .count(),
            0,
            "html={html}"
        );
    }

    /// すべてのトリガーが disabled（ネイティブ属性 + `aria-disabled`）で
    /// あること（モジュール doc「静的アコーディオン」節）。
    #[test]
    fn all_triggers_are_disabled() {
        let html = render(&demo());
        assert_eq!(
            html.matches(r#"data-part="item-trigger""#).count(),
            FEATURES.len(),
            "html={html}"
        );
        assert_eq!(
            html.matches(r#"aria-disabled="true""#).count(),
            FEATURES.len(),
            "html={html}"
        );
        assert_eq!(html.matches(" disabled=\"\"").count(), FEATURES.len());
    }

    /// 全項目に `aria-controls` と `item-content` があり、`hidden` な
    /// 到達不能ノードを一切残さないこと（モジュール doc「静的
    /// アコーディオン」節、イシュー #2761 レビュー指摘の是正）。
    #[test]
    fn all_items_have_content_and_aria_controls() {
        let html = render(&demo());
        assert_eq!(
            html.matches("aria-controls=").count(),
            FEATURES.len(),
            "html={html}"
        );
        assert_eq!(
            html.matches(r#"data-part="item-content""#).count(),
            FEATURES.len(),
            "html={html}"
        );
        assert_eq!(html.matches(" hidden=\"\"").count(), 0, "html={html}");
    }

    /// 右列の画像がちょうど 1 枚（代表画像）であり、インライン画像は
    /// 全項目分（`FEATURES.len()` 枚）出力されること（本 block は全項目が
    /// 常時展開のため、各項目が自分のインライン画像を持つ）。
    #[test]
    fn exactly_one_media_slot_image_and_inline_image_per_feature() {
        let html = render(&demo());
        assert_eq!(
            html.matches("data-blocks-feature-accordion-image-media=\"\"")
                .count(),
            1,
            "html={html}"
        );
        assert_eq!(
            html.matches("data-blocks-feature-accordion-image-inline-image=\"\"")
                .count(),
            FEATURES.len(),
            "html={html}"
        );
    }

    /// `data:` URI・`<form` を持ち込まないこと（A05）。
    #[test]
    fn demo_output_has_no_form_or_data_uri() {
        let html = render(&demo());
        assert!(!html.contains("<form"));
        assert!(!html.contains("src=\"data:"));
    }

    /// 見出しが `h3`、トリガーが `h4` で包まれていること。
    #[test]
    fn heading_and_trigger_levels() {
        let html = render(&demo());
        assert!(html.contains("<h3"));
        assert!(html.contains("class=\"blocks-feature-accordion-image-trigger-heading\""));
    }
}
