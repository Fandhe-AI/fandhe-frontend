//! `changelog-timeline` block（イシュー #2820。親トラッキング #2807
//! 「Blocks マーケティング B」配下、対応表 ID R0043/R0047/R0048/R0041 の
//! 4 件を構造の参照元とする合成例。中央寄せの見出し + リード文の下に、
//! リリースを縦のタイムラインで並べる changelog）。取得手段・ファイル名・
//! 内部コンポーネント識別子は記載しない（`docs/design/motion-reference-
//! adoption-policy.md` §9 と同じライセンス上の転記制限）。
//!
//! **Marketing / Changelog カテゴリで 2 番目の block**（最初は
//! `changelog_accordion`、`super`（`changelog/mod.rs`）参照）。
//!
//! # 使用部品
//!
//! `heading` / `text` / `badge` / `timeline` / `list` / `image` / `link` の
//! 7 部品を合成する（[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//!
//! # 4 参照 ID の畳み込み方（Demo は 2 インスタンス）
//!
//! - **boxed インスタンス**（R0043 が主参照。R0047 は同じ 3 列構造で JS
//!   駆動のスクロール進捗バーを持つが、無 JS の docs サイトでは再現しない）:
//!   左列（日付 + version の枠）・中央のコネクタ（indicator + 縦線）・
//!   右列（本文）の 3 列構成。
//! - **pill インスタンス**（R0048）: indicator 自体を日付入りのピルに
//!   した outline 表現。左列を持たず 2 列（indicator | 本文）。
//! - R0041（version 列 + 変更一覧の 2 列構成）は独立した 3 つ目の Demo
//!   インスタンスとしては並べない。boxed インスタンスの狭幅表示（下記
//!   「狭い幅では左列を隠す」節）が「version + 変更一覧が主体の 2 列」に
//!   収束するため、この構造は狭幅表示と pill インスタンスの組み合わせで
//!   示す（[`site/blocks/changelog-timeline.md`] の「原案差分メモ」に明記）。
//!
//! # 静的表示（無 JS、架空データ固定）
//!
//! [`RELEASES`] は呼び出しごとに同一の `Node` を生成する純関数
//! （[`demo`]）が参照するだけの固定配列であり、状態機械・フォーム・
//! データ取得を一切持たない。
//!
//! # タイトルに `timeline::title` ではなく `heading` H4 を使う理由
//!
//! [`fandhe_frontend_pre_styled_ui::timeline::title`] は `<span>` を組み立てる
//! ため、見出し要素（`<h4>`）を内包できない。本 block はリリースタイトルを
//! 文書構造上の見出しとして扱いたいため、`timeline::title` は使わず
//! [`fandhe_frontend_pre_styled_ui::heading::heading`]（[`HeadingLevel::H4`]）
//! を直接 `timeline::content` の子として置く。
//!
//! # 狭い幅では左列を隠す（`data-blocks-changelog-timeline-inline-meta`）
//!
//! boxed インスタンスは広い幅で日付 + version を専用の左列に表示するが、
//! `@media (max-width: 47.99rem)`（`login_04` 等の前例と同じ閾値、
//! [`LAYOUT_CSS`] 参照）では左列を `display: none` にし、代わりに本文側
//! （`data-blocks-changelog-timeline-body`）内に同じ内容を複製した
//! `data-blocks-changelog-timeline-inline-meta` を表示へ切り替える。DOM 上は
//! 常に両方が存在するが、どの幅でも常に片方は `display: none` で支援技術
//! からも隠れるため、重複読み上げは起きない。
//!
//! # separator を最後のエントリで省く理由
//!
//! [`fandhe_frontend_pre_styled_ui::timeline`] モジュール doc
//! 「`showLastSeparator` 相当は実装しない」節のとおり、最終 item の
//! separator 非表示は呼び出し側の構成責務である。本 block も最後の
//! リリースでは [`timeline::separator`] を connector の子へ含めない。
//!
//! # id 属性を出力しない理由
//!
//! 2 インスタンス（boxed/pill）を同一ページへ並べるため、`id`/
//! `aria-controls`/`aria-labelledby` を出力すると id 重複や宙に浮いた
//! ARIA 参照を生みやすい。本 block はいずれの部品も `id` を要さない
//! 構成のため一切出力しない（`crates/docs-site/tests/blocks_contract.rs::
//! demo_output_has_no_dangling_aria_references_or_duplicate_ids` の対象）。
//!
//! # `text` の名前衝突
//!
//! `fandhe_frontend_pre_styled_ui::text::text`（`<p>` を組み立てる styled
//! パート関数）と `fandhe_frontend_core::text`（テキストノード生成関数）が
//! 同名のため、styled 側を `styled_text` として取り込む（`changelog_
//! accordion` 等、`crate::blocks` 内の他 block と同じ回避方法）。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `heading::heading` / `text::text` / `badge::badge` / `timeline::root` /
//! `list::root` / `image::image` / `link::root` はいずれも `drop_class_attr`
//! により呼び出し側 `attrs` の `class` を黙って除去する契約を持つため、
//! Demo 固有のスタイルフックは `data-blocks-changelog-timeline-*` 属性で
//! 渡す。`timeline` の非 root パーツ（`item`/`connector`/`separator`/
//! `indicator`/`content`）は `drop_class_attr` を経由しないため、こちらも
//! 同じ data-* 属性方式へ統一する（root/非 root で異なる手段を混在させない
//! ための判断）。素の `div`/`time` には `.blocks-changelog-timeline-*`
//! クラスセレクタを使う。
//!
//! # `<form>` を持たない・データ取得/送信を行わない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である。文言・バージョン番号・日付・変更点はすべて架空のもの（実在の
//! 製品・企業名・PII を含まない）。リンク先は固定の外部リポジトリ URL の
//! みで、ユーザー入力・動的な値は一切使わない。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::image::{self, AspectRatio, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::list::{self, ListType, ListVariant};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};
use fandhe_frontend_pre_styled_ui::timeline::{self, TimelineVariant};
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

use crate::blocks::dummy_assets;

/// 外部リンク先（`href="#"` を使わないための固定 URL、`blog_list_image` と
/// 同じ判断）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// リリース 1 件分のダミーデータ（架空、実在の製品・企業とは無関係）。
struct Release {
    version: &'static str,
    date_iso: &'static str,
    date_label: &'static str,
    title: &'static str,
    tags: &'static [&'static str],
    image_src: Option<&'static str>,
    changes: &'static [&'static str],
}

/// リリース一覧（架空、4 件。画像は一部エントリのみに付け「任意の画像
/// スロット」であることを示す）。
const RELEASES: [Release; 4] = [
    Release {
        version: "v3.1.0",
        date_iso: "2026-09-20",
        date_label: "2026年9月20日",
        title: "タイムライン型 changelog を追加",
        tags: &["新機能"],
        image_src: Some(dummy_assets::SCREENSHOT_SRC),
        changes: &[
            "リリースを縦のタイムラインで並べる changelog レイアウトを追加",
            "日付入りピル表現の代替表示に対応",
        ],
    },
    Release {
        version: "v3.0.2",
        date_iso: "2026-09-12",
        date_label: "2026年9月12日",
        title: "狭い幅での表示崩れを修正",
        tags: &["修正"],
        image_src: None,
        changes: &["狭い幅で日付・version が本文と重なる表示崩れを修正"],
    },
    Release {
        version: "v3.0.1",
        date_iso: "2026-09-05",
        date_label: "2026年9月5日",
        title: "ダミー素材ヘルパを共通化",
        tags: &["改善", "内部"],
        image_src: Some(dummy_assets::PRODUCT_SRC),
        changes: &[
            "プレースホルダー画像・文言の生成をヘルパへ一元化",
            "block ごとの個別実装によるブレを解消",
        ],
    },
    Release {
        version: "v3.0.0",
        date_iso: "2026-08-28",
        date_label: "2026年8月28日",
        title: "changelog レイアウトを刷新",
        tags: &["新機能", "破壊的変更"],
        image_src: None,
        changes: &[
            "リリース単位のレイアウトを全面刷新",
            "変更点の種別タグ表示に対応",
        ],
    },
];

/// `<time datetime>` を組み立てる（機械可読な ISO 値と表示値は常に同じ日を
/// 指す組にする不変条件、`changelog_accordion`/`blog_list_image` と同じ
/// 判断）。
fn release_date(iso: &str, label: &str) -> Node {
    el(
        "time",
        vec![
            ("class", "blocks-changelog-timeline-date"),
            ("datetime", iso),
        ],
        vec![text(label)],
    )
}

/// 日付 + version の枠（boxed インスタンスの左列、および狭幅時に本文内へ
/// 表示する複製の共通実体）。
fn release_meta(release: &Release) -> Node {
    div(
        vec![("class", "blocks-changelog-timeline-meta")],
        vec![
            fandhe_frontend_core::span(
                vec![("class", "blocks-changelog-timeline-version")],
                vec![text(release.version)],
            ),
            release_date(release.date_iso, release.date_label),
        ],
    )
}

/// タグ badge 群。
fn release_tags(release: &Release) -> Node {
    div(
        vec![("class", "blocks-changelog-timeline-tags")],
        release
            .tags
            .iter()
            .map(|tag| {
                badge::badge(
                    &BadgeProps::default(),
                    vec![("data-blocks-changelog-timeline-tag", "")],
                    vec![text(*tag)],
                )
            })
            .collect(),
    )
}

/// 変更点リスト。
fn release_changes(release: &Release) -> Node {
    list::root(
        ListType::Unordered,
        ListVariant::Marker,
        vec![("data-blocks-changelog-timeline-changes", "")],
        release
            .changes
            .iter()
            .map(|change| list::item(vec![], vec![text(*change)]))
            .collect(),
    )
}

/// boxed インスタンス 1 リリース分の `timeline::item`。3 列（左列は日付
/// と version、中央は connector、右列は本文）。[`RELEASES`] は新しい順
/// （先頭が最新）のため、`is_first` は最新エントリの indicator を
/// `"current"` にする判定に、`is_last` は最終（最古）エントリで
/// `separator` を省く判定に使う（モジュール doc「separator を最後の
/// エントリで省く理由」節）。
fn boxed_item(release: &Release, is_first: bool, is_last: bool) -> Node {
    let side = timeline::content(
        vec![("data-blocks-changelog-timeline-side", "")],
        vec![release_meta(release)],
    );

    let mut connector_children = vec![timeline::indicator(
        vec![("data-state", if is_first { "current" } else { "complete" })],
        vec![],
    )];
    if !is_last {
        connector_children.push(timeline::separator(
            vec![("data-state", "complete")],
            vec![],
        ));
    }
    let connector = timeline::connector(vec![], connector_children);

    let mut body_children = vec![
        div(
            vec![("data-blocks-changelog-timeline-inline-meta", "")],
            vec![release_meta(release)],
        ),
        heading(
            HeadingLevel::H4,
            &HeadingProps {
                size: HeadingSize::Lg,
                weight: HeadingWeight::Semibold,
            },
            vec![],
            vec![text(release.title)],
        ),
        release_tags(release),
    ];
    if let Some(src) = release.image_src {
        body_children.push(div(
            vec![("class", "blocks-changelog-timeline-figure")],
            vec![image::image(
                &ImageProps {
                    aspect_ratio: AspectRatio::Video,
                    shape: ImageShape::Rounded,
                    ..ImageProps::new(src, "")
                },
                vec![("data-blocks-changelog-timeline-image", "")],
            )],
        ));
    }
    body_children.push(release_changes(release));
    body_children.push(link::root(
        REPO,
        &LinkProps {
            external: true,
            ..LinkProps::default()
        },
        vec![],
        vec![text("リリースノートを見る")],
    ));
    let body = timeline::content(
        vec![("data-blocks-changelog-timeline-body", "")],
        body_children,
    );

    timeline::item(vec![], vec![side, connector, body])
}

/// pill インスタンス 1 リリース分の `timeline::item`。2 列（indicator |
/// 本文）で、indicator 自体を日付入りのピルにする（R0048）。左列（日付 +
/// version 専用枠）は持たない。`is_first`/`is_last` の意味は [`boxed_item`]
/// と同じ（[`RELEASES`] は新しい順）。
fn pill_item(release: &Release, is_first: bool, is_last: bool) -> Node {
    let mut connector_children = vec![timeline::indicator(
        vec![("data-state", if is_first { "current" } else { "complete" })],
        vec![release_date(release.date_iso, release.date_label)],
    )];
    if !is_last {
        connector_children.push(timeline::separator(
            vec![("data-state", "complete")],
            vec![],
        ));
    }
    let connector = timeline::connector(vec![], connector_children);

    let body = timeline::content(
        vec![("data-blocks-changelog-timeline-body", "")],
        vec![
            fandhe_frontend_core::span(
                vec![("class", "blocks-changelog-timeline-version")],
                vec![text(release.version)],
            ),
            heading(
                HeadingLevel::H4,
                &HeadingProps {
                    size: HeadingSize::Lg,
                    weight: HeadingWeight::Semibold,
                },
                vec![],
                vec![text(release.title)],
            ),
            release_tags(release),
            release_changes(release),
        ],
    );

    timeline::item(vec![], vec![connector, body])
}

/// `changelog-timeline` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数（モジュール doc「静的表示」節）。boxed（R0043/R0047）・pill
/// （R0048）の 2 インスタンスを縦に並べる。
pub fn demo() -> Node {
    let header = div(
        vec![("class", "blocks-changelog-timeline-header")],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    weight: HeadingWeight::Bold,
                },
                vec![],
                vec![text("更新履歴")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text("各リリースの変更点を時系列で確認できます。")],
            ),
        ],
    );

    let last_index = RELEASES.len().saturating_sub(1);

    let boxed_items: Vec<Node> = RELEASES
        .iter()
        .enumerate()
        .map(|(index, release)| boxed_item(release, index == 0, index == last_index))
        .collect();
    let boxed = timeline::root(
        TimelineVariant::Solid,
        Size::Sm,
        ColorPalette::default(),
        vec![("data-blocks-changelog-timeline-variant", "boxed")],
        boxed_items,
    );

    let pill_items: Vec<Node> = RELEASES
        .iter()
        .enumerate()
        .map(|(index, release)| pill_item(release, index == 0, index == last_index))
        .collect();
    let pill = timeline::root(
        TimelineVariant::Outline,
        Size::Sm,
        ColorPalette::default(),
        vec![("data-blocks-changelog-timeline-variant", "pill")],
        pill_items,
    );

    div(
        vec![("class", "blocks-changelog-timeline-layout")],
        vec![header, boxed, pill],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/changelog-timeline/",
    title: "changelog-timeline",
    category: BlockCategory::Changelog,
    rust_source: "crates/docs-site/src/blocks/marketing/changelog/changelog_timeline.rs",
    demo_class: "blocks-changelog-timeline",
    parts: &[
        Part {
            label: "Heading",
            path: "/themes/heading/",
        },
        Part {
            label: "Text",
            path: "/themes/text/",
        },
        Part {
            label: "Badge",
            path: "/themes/badge/",
        },
        Part {
            label: "Timeline",
            path: "/themes/timeline/",
        },
        Part {
            label: "List",
            path: "/themes/list/",
        },
        Part {
            label: "Image",
            path: "/themes/image/",
        },
        Part {
            label: "Link",
            path: "/themes/link/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `changelog_timeline` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節）。
///
/// # 3 列化（boxed インスタンス限定）
///
/// styled `timeline::item` の既定 recipe は 2 列
/// （`indicator-size 1fr`、[`timeline`] モジュール doc「variant 3 軸」
/// 節参照）だが、boxed インスタンスは「日付+version 枠 | connector |
/// 本文」の 3 列を要件とする。子孫結合子付きセレクタ（詳細度 0,4,0）で
/// 確実に recipe（0,2,0）へ勝たせる。
///
/// # 狭幅（`@media (max-width: 47.99rem)`）
///
/// `login_04` 等の前例と同じ閾値。boxed インスタンスを 2 列（indicator |
/// 本文）へ戻し、左列（`data-blocks-changelog-timeline-side`）を隠し
/// 本文内複製（`data-blocks-changelog-timeline-inline-meta`）を表示へ
/// 切り替える。
///
/// # pill インスタンス（`data-blocks-changelog-timeline-variant="pill"`）
///
/// `item` の `grid-template-columns` を `auto minmax(0, 1fr)` へ上書きし
/// （既定の `indicator-size 1fr` では固定幅の第 1 トラックへ日付ラベルが
/// 収まらない）、indicator を「幅・高さ自動 + 横長 padding + 角丸全周」の
/// ピル形状へ上書きし、日付ラベル用に `white-space: nowrap` と小さめの
/// font-size を与える。
const LAYOUT_CSS: &str = "\
.blocks-changelog-timeline-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-10);\n  width: 100%;\n}\n\
.blocks-changelog-timeline-header {\n  display: flex;\n  flex-direction: column;\n  align-items: center;\n  text-align: center;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-changelog-timeline-layout [data-blocks-changelog-timeline-variant=\"boxed\"] [data-scope=\"timeline\"][data-part=\"item\"] {\n  grid-template-columns: 12rem var(--fandhe-timeline-indicator-size, 1.5rem) minmax(0, 1fr);\n}\n\
.blocks-changelog-timeline-layout [data-blocks-changelog-timeline-variant=\"boxed\"] [data-blocks-changelog-timeline-side] {\n  grid-column: 1;\n}\n\
.blocks-changelog-timeline-layout [data-blocks-changelog-timeline-variant=\"boxed\"] [data-scope=\"timeline\"][data-part=\"connector\"] {\n  grid-column: 2;\n}\n\
.blocks-changelog-timeline-layout [data-blocks-changelog-timeline-variant=\"boxed\"] [data-blocks-changelog-timeline-body] {\n  grid-column: 3;\n}\n\
.blocks-changelog-timeline-layout [data-blocks-changelog-timeline-inline-meta] {\n  display: none;\n}\n\
@media (max-width: 47.99rem) {\n  .blocks-changelog-timeline-layout [data-blocks-changelog-timeline-variant=\"boxed\"] [data-scope=\"timeline\"][data-part=\"item\"] {\n    grid-template-columns: var(--fandhe-timeline-indicator-size, 1.5rem) minmax(0, 1fr);\n  }\n  .blocks-changelog-timeline-layout [data-blocks-changelog-timeline-variant=\"boxed\"] [data-blocks-changelog-timeline-side] {\n    display: none;\n  }\n  .blocks-changelog-timeline-layout [data-blocks-changelog-timeline-variant=\"boxed\"] [data-scope=\"timeline\"][data-part=\"connector\"] {\n    grid-column: 1;\n  }\n  .blocks-changelog-timeline-layout [data-blocks-changelog-timeline-variant=\"boxed\"] [data-blocks-changelog-timeline-body] {\n    grid-column: 2;\n  }\n  .blocks-changelog-timeline-layout [data-blocks-changelog-timeline-inline-meta] {\n    display: flex;\n    flex-direction: column;\n    margin-bottom: var(--fandhe-space-2);\n  }\n}\n\
.blocks-changelog-timeline-meta {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1);\n  padding: var(--fandhe-space-3);\n  border: 1px solid var(--fandhe-color-border);\n  border-radius: var(--fandhe-radius-lg);\n}\n\
.blocks-changelog-timeline-version {\n  font-weight: var(--fandhe-font-weight-bold, 700);\n}\n\
.blocks-changelog-timeline-date {\n  color: var(--fandhe-color-fg-muted);\n  font-size: var(--fandhe-font-font-size-sm, 0.875rem);\n}\n\
.blocks-changelog-timeline-tags {\n  display: flex;\n  flex-wrap: wrap;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-changelog-timeline-figure {\n  width: 100%;\n  max-width: 40rem;\n}\n\
[data-blocks-changelog-timeline-image] {\n  display: block;\n  width: 100%;\n}\n\
.blocks-changelog-timeline-layout [data-blocks-changelog-timeline-variant=\"pill\"] [data-scope=\"timeline\"][data-part=\"item\"] {\n  grid-template-columns: auto minmax(0, 1fr);\n}\n\
.blocks-changelog-timeline-layout [data-blocks-changelog-timeline-variant=\"pill\"] [data-scope=\"timeline\"][data-part=\"indicator\"] {\n  width: auto;\n  height: auto;\n  padding: var(--fandhe-space-1) var(--fandhe-space-3);\n  border-radius: var(--fandhe-radius-full, 9999px);\n  white-space: nowrap;\n  font-size: var(--fandhe-font-font-size-xs);\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS, RELEASES};
    use fandhe_frontend_core::render;

    /// Demo が期待する 7 種の部品を出力すること。
    #[test]
    fn demo_composes_expected_parts() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"badge\"",
            "data-scope=\"timeline\"",
            "data-scope=\"list\"",
            "data-scope=\"image\"",
            "data-scope=\"link\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
    }

    /// boxed・pill 双方の variant フックが出力されていること。
    #[test]
    fn demo_wires_both_variant_hooks() {
        let html = render(&demo());
        assert!(html.contains(r#"data-blocks-changelog-timeline-variant="boxed""#));
        assert!(html.contains(r#"data-blocks-changelog-timeline-variant="pill""#));
    }

    /// 各インスタンスの separator 数が「エントリ数 - 1」であること
    /// （最後のエントリで separator を省く契約、モジュール doc参照）。
    #[test]
    fn demo_omits_separator_on_last_entry_per_instance() {
        let html = render(&demo());
        let expected = RELEASES.len() - 1;
        assert_eq!(
            html.matches(r#"data-part="separator""#).count(),
            expected * 2,
            "each of the 2 instances should render exactly {expected} separators; html={html}"
        );
    }

    /// 非対話・安全性の不変条件（`<form>`・`href="#"`・`data:` URI・`id=`
    /// 属性を持たないこと。モジュール doc「id 属性を出力しない理由」節）。
    #[test]
    fn demo_never_contains_forbidden_markup() {
        let html = render(&demo());
        for absent in ["<form", "href=\"#\"", "src=\"data:", "id=\""] {
            assert!(!html.contains(absent), "demo should never contain {absent}");
        }
    }

    /// [`LAYOUT_CSS`] が狭幅切り替えと 3 列化のセレクタを持つこと。
    #[test]
    fn layout_css_has_responsive_and_three_column_rules() {
        assert!(LAYOUT_CSS.contains("@media (max-width: 47.99rem)"));
        assert!(LAYOUT_CSS.contains(
            r#"[data-blocks-changelog-timeline-variant="boxed"] [data-scope="timeline"][data-part="item"] {"#
        ));
    }

    /// ルート class（`demo_class` とは別名）が [`demo`] の出力へ実際に
    /// 現れること（`blog_list_image` と同じ Bugbot 教訓の固定）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-changelog-timeline-layout\""));
        assert_ne!(super::BLOCK.demo_class, "blocks-changelog-timeline-layout");
    }
}
