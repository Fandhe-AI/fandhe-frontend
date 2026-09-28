//! `grid-list-file-thumbnails` block（イシュー #2920。Application/Grid List
//! カテゴリの最初の block）。画像サムネイルを 2〜4 列のグリッドに並べ、
//! 各サムネイルの下にファイル名とファイルサイズを表示する合成例。
//! 対応表 ID R0978（代表構成、1 件のみのため差分の並記はない）を参照元と
//! する。`_/blocks-intake/` の対応ファイルは本イシュー着手時点で本
//! worktree に存在しないため、原稿・本コメントには対応表 ID のみを記す
//! （`form-layout-two-column`〔イシュー #2916〕と同じ扱い）。
//!
//! # 使用部品
//!
//! `image` / `button` / `text` / `list` / `visually-hidden` の 5 部品を
//! 合成する（[`BLOCK`] の `parts` に一致させる契約）。新しい UI 部品は
//! 追加しない。
//!
//! # サムネイル全体をボタンにする理由
//!
//! Issue 要件「サムネイル全体を『詳細を表示』するボタンにし、ホバー時に
//! 強調表示する」を満たすため、[`button::button`]（`ButtonVariant::Plain`、
//! 背景・輪郭なしの最小装飾）で画像を包む。accessible name は画像内容を
//! 説明しないため、[`visually_hidden::root`] で「詳細を表示」の意図を
//! 伝えるラベルをボタン内へ追加する（画面上は非表示、スクリーンリーダー
//! のみに読み上げられる）。
//!
//! # コンテナクエリで列数を切り替える理由
//!
//! Demo 枠の幅はビューポート幅と一致しないため、`gallery-image-grid` の
//! `@media` 方式ではなく `form-layout-two-column`/
//! `description-list-horizontal` と同型の `@container` を使う。外側
//! スタックへ `container-type: inline-size; container-name:
//! blocks-grid-list-file-thumbnails;` を宣言し、既定（狭幅）は 2 列、
//! `min-width: 30rem` で 3 列、`min-width: 40rem` で 4 列へ切り替える。
//!
//! # 閾値を 48rem/64rem ではなく 30rem/40rem にする理由（PR #3364 codex(P1)
//! 指摘の是正）
//!
//! `.docs-content` の本文カラムは `max-width: 46rem`、Demo 枠
//! （`.blocks-demo`）は左右 `padding: 1.5rem` を持つため、本コンテナが
//! 実際に取り得る inline-size は最大でも 46rem − 3rem = 43rem である。
//! 当初案の `min-width: 48rem`/`64rem` はこの上限を上回っており、通常の
//! 掲載ページでは決して 3 列・4 列へ到達せず常に 2 列のまま固定される
//! （原稿が約束する 3〜4 列表示が実現しない不整合）。43rem 未満で確実に
//! 到達可能な `min-width: 30rem`/`40rem` へ引き下げ、実際のページ幅で
//! 3 列・4 列へ切り替わることを保証する。
//!
//! # サムネイル画像を 5 種で使い回す理由（PR #3364 Codex(P2) 指摘の是正）
//!
//! 本モジュールは 12 件全ファイルへ [`dummy_assets::PRODUCT_SRC`] のみを
//! 割り当てていたため、ファイルサムネイル一覧というデモ説明に反して見た目が
//! 単調に見える不整合があった。`dummy_assets` はいずれもモノトーンの抽象
//! 図形（実在の写真ではない）画像ヘルパを 5 種（商品・アバター・ロゴ・
//! スクリーンショット・背景タイル）持つため、新規アセットを追加せず
//! [`THUMBNAIL_SRCS`] でこの 5 種を `FILES` のインデックスに応じて循環割当
//! し、ファイルごとに異なるサムネイルへ見せる（`bento_two_column.rs` 等の
//! 既存 block と同型の使い回し）。
//!
//! # `alt=""` にする理由
//!
//! 同一プレースホルダー画像を複数枚並べる際、内容を伝えない同一文言の
//! `alt` を繰り返すとスクリーンリーダーで同じ文言が連呼される（WCAG
//! 1.1.1、`gallery-image-grid` 等で確認済みの教訓）ため、装飾用途として
//! `alt=""` を使う。accessible name はボタンの visually-hidden ラベルが
//! 単独で決める。
//!
//! # メタ情報段落へ `margin: 0` を付ける理由（PR #3364 codex(P2) 指摘の是正）
//!
//! `styled_text::text` は `<p>` を出力し、`.docs-content p` の下余白
//! （1.05rem）がファイル名・サイズの段落にも適用され、指定した
//! `gap: var(--fandhe-space-1)` より間隔が広がっていた。
//! `[data-blocks-grid-list-file-thumbnails-meta] p` へ `margin: 0` を付け、
//! 段落間隔を `gap` のみで制御する。
//!
//! # `<form>` を持たない・実データを持たない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo はフォーム・
//! 送信処理・データ取得を持たない静的な合成例である。ファイル名・サイズは
//! すべて架空のものであり、実企業名・実クレデンシャル・PII を含まない。
//! クリックで開く「詳細表示」の中身（ダイアログ等）は作らない
//! （`docs/policy/intentional-non-adoption.md` §3.25、UI コンポーネント層
//! はアプリケーションロジックを内包しない）。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `button::button`/`list::root`/`list::item`/`text::text`/`image::image`
//! はいずれも `drop_class_attr` により呼び出し側 `attrs` の `class` を
//! 黙って除去する契約を持つため、グリッド本体・セル・トリガー・画像への
//! スタイルフックは `data-blocks-grid-list-file-thumbnails-*` 属性で渡す。
//! 素の `div`/`p` にはクラスセレクタを使う。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, p, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::image::{self, AspectRatio, ImageProps};
use fandhe_frontend_pre_styled_ui::list::{self, ListType, ListVariant};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::visually_hidden;

/// 架空のファイル名とサイズのセット（12 件。12 は 2/3/4 いずれの列数でも
/// 割り切れる最小公倍数のため、2/3/4 列のいずれでも最終行が埋まる）。実在の
/// 人名・社名・PII は含まない。
const FILES: [(&str, &str); 12] = [
    ("IMG_4821.jpg", "3.9 MB"),
    ("harbor-sunset.png", "2.4 MB"),
    ("team-offsite-04.jpg", "5.1 MB"),
    ("product-mockup-v2.png", "1.8 MB"),
    ("mountain-trail.jpg", "4.6 MB"),
    ("workshop-notes.png", "0.9 MB"),
    ("studio-shelf.jpg", "3.2 MB"),
    ("river-bridge-evening.jpg", "6.0 MB"),
    ("conference-badge.png", "0.7 MB"),
    ("rooftop-garden.jpg", "4.1 MB"),
    ("archive-notes-scan.png", "1.2 MB"),
    ("lakeside-cabin.jpg", "5.5 MB"),
];

/// [`FILES`] のインデックスへ循環割当するサムネイル `src` の候補
/// （`dummy_assets` の 5 種すべて。PR #3364 Codex(P2) 指摘の是正、モジュール
/// doc 参照）。
const THUMBNAIL_SRCS: [&str; 5] = [
    dummy_assets::PRODUCT_SRC,
    dummy_assets::AVATAR_SRC,
    dummy_assets::LOGO_SRC,
    dummy_assets::SCREENSHOT_SRC,
    dummy_assets::BACKGROUND_SRC,
];

/// キャプション（見出し代わりの短い説明文）。
fn caption() -> Node {
    p(
        vec![("class", "blocks-grid-list-file-thumbnails-caption")],
        vec![text("最近アップロードした画像")],
    )
}

/// グリッド 1 セル分（サムネイル全体を「詳細を表示」ボタンにし、下へ
/// ファイル名・サイズを表示する）。`thumbnail_src` は [`THUMBNAIL_SRCS`] を
/// 呼び出し側が循環割当した値。
fn cell(name: &str, size: &str, thumbnail_src: &str) -> Node {
    let sr_label = format!("{name} の詳細を表示");
    let trigger = button::button(
        &ButtonProps {
            variant: ButtonVariant::Plain,
            ..ButtonProps::default()
        },
        vec![("data-blocks-grid-list-file-thumbnails-trigger", "")],
        vec![
            image::image(
                &ImageProps {
                    aspect_ratio: AspectRatio::Landscape,
                    ..ImageProps::new(thumbnail_src, "")
                },
                vec![("data-blocks-grid-list-file-thumbnails-image", "")],
            ),
            visually_hidden::root(vec![], vec![text(&sr_label)]),
        ],
    );
    let meta = div(
        vec![("data-blocks-grid-list-file-thumbnails-meta", "")],
        vec![
            styled_text::text(&TextProps::default(), vec![], vec![text(name)]),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    size: TextSize::Sm,
                    ..TextProps::default()
                },
                vec![],
                vec![text(size)],
            ),
        ],
    );
    list::item(
        vec![("data-blocks-grid-list-file-thumbnails-item", "")],
        vec![trigger, meta],
    )
}

/// `grid-list-file-thumbnails` の Demo 本体。呼び出しごとに同一の `Node`
/// を返す純関数。
pub fn demo() -> Node {
    let items = FILES
        .iter()
        .enumerate()
        .map(|(i, (name, size))| cell(name, size, THUMBNAIL_SRCS[i % THUMBNAIL_SRCS.len()]))
        .collect();
    let grid = list::root(
        ListType::Unordered,
        ListVariant::Plain,
        vec![("data-blocks-grid-list-file-thumbnails-grid", "")],
        items,
    );
    div(
        vec![("class", "blocks-grid-list-file-thumbnails-stack")],
        vec![caption(), grid],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ。
pub const BLOCK: Block = Block {
    path: "/blocks/grid-list-file-thumbnails/",
    title: "grid-list-file-thumbnails",
    category: BlockCategory::GridList,
    rust_source: "crates/docs-site/src/blocks/application/grid_list/grid_list_file_thumbnails.rs",
    demo_class: "blocks-grid-list-file-thumbnails",
    parts: &[
        Part {
            label: "Image",
            path: "/themes/image/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Text",
            path: "/themes/text/",
        },
        Part {
            label: "List",
            path: "/themes/list/",
        },
        Part {
            label: "Visually Hidden",
            path: "/themes/visually-hidden/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `grid_list_file_thumbnails` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節）。セレクタは
/// `.blocks-grid-list-file-thumbnails-*` と
/// `[data-blocks-grid-list-file-thumbnails-*]` のみを用いる。
const LAYOUT_CSS: &str = "\
.blocks-grid-list-file-thumbnails-stack {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n  container-type: inline-size;\n  container-name: blocks-grid-list-file-thumbnails;\n}\n\
.blocks-grid-list-file-thumbnails-stack .blocks-grid-list-file-thumbnails-caption {\n  margin: 0;\n  color: var(--fandhe-color-fg-muted);\n  font-size: var(--fandhe-font-size-sm);\n}\n\
[data-scope=\"list\"][data-part=\"root\"][data-blocks-grid-list-file-thumbnails-grid] {\n  display: grid;\n  grid-template-columns: repeat(2, minmax(0, 1fr));\n  gap: var(--fandhe-space-4);\n  margin: 0;\n  padding: 0;\n  list-style: none;\n}\n\
[data-scope=\"list\"][data-part=\"root\"][data-blocks-grid-list-file-thumbnails-grid] > [data-scope=\"list\"][data-part=\"item\"][data-blocks-grid-list-file-thumbnails-item] {\n  display: flex;\n  flex-direction: column;\n  align-items: stretch;\n  gap: var(--fandhe-space-2);\n  margin-block: 0;\n  min-width: 0;\n}\n\
[data-scope=\"button\"][data-part=\"root\"][data-blocks-grid-list-file-thumbnails-trigger] {\n  display: block;\n  width: 100%;\n  height: auto;\n  min-height: 0;\n  padding: 0;\n  overflow: hidden;\n  border-radius: var(--fandhe-radius-md);\n}\n\
[data-scope=\"image\"][data-part=\"root\"][data-blocks-grid-list-file-thumbnails-image] {\n  display: block;\n  width: 100%;\n  transition: opacity var(--fandhe-motion-duration-fast, 150ms) var(--fandhe-motion-easing-standard, ease);\n}\n\
[data-blocks-grid-list-file-thumbnails-trigger]:hover [data-blocks-grid-list-file-thumbnails-image] {\n  opacity: 0.75;\n}\n\
[data-blocks-grid-list-file-thumbnails-meta] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1);\n  min-width: 0;\n}\n\
[data-blocks-grid-list-file-thumbnails-meta] p {\n  margin: 0;\n  overflow: hidden;\n  text-overflow: ellipsis;\n  white-space: nowrap;\n}\n\
@media (prefers-reduced-motion: reduce) {\n  [data-scope=\"image\"][data-part=\"root\"][data-blocks-grid-list-file-thumbnails-image] {\n    transition: none;\n  }\n}\n\
@container blocks-grid-list-file-thumbnails (min-width: 30rem) {\n  [data-scope=\"list\"][data-part=\"root\"][data-blocks-grid-list-file-thumbnails-grid] {\n    grid-template-columns: repeat(3, minmax(0, 1fr));\n  }\n}\n\
@container blocks-grid-list-file-thumbnails (min-width: 40rem) {\n  [data-scope=\"list\"][data-part=\"root\"][data-blocks-grid-list-file-thumbnails-grid] {\n    grid-template-columns: repeat(4, minmax(0, 1fr));\n  }\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, FILES, LAYOUT_CSS, THUMBNAIL_SRCS};
    use fandhe_frontend_core::render;

    /// Demo が期待する部品・構造・非対話制約を満たしていることの単体
    /// 回帰（`crates/docs-site/tests/blocks_contract.rs` の横断検査と重複
    /// し過ぎない範囲での個別固定）。
    #[test]
    fn demo_composes_expected_parts_and_avoids_forms() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"image\"",
            "data-scope=\"button\"",
            "data-scope=\"text\"",
            "data-scope=\"list\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert_eq!(html.matches("<img").count(), FILES.len());
        assert_eq!(
            html.matches("type=\"button\"").count(),
            FILES.len(),
            "each trigger must be a native type=\"button\""
        );
        assert_eq!(html.matches("<li").count(), FILES.len());
        for (name, size) in FILES {
            assert!(html.contains(name), "file name {name} should be rendered");
            assert!(html.contains(size), "file size {size} should be rendered");
        }
        assert!(!html.contains("<form"));
        assert!(!html.contains("<script"));
        assert!(!html.contains("src=\"data:"));
        // 素の `id="` 部分一致は自前の `data-blocks-...-grid` 属性名末尾
        // （`...grid="..."` は文字列として `id="` を部分文字列に含む）と
        // 衝突するため、先頭空白付きで判定する
        // （`crates/docs-site/tests/blocks_contract.rs` の既存パターンと
        // 同型）。
        assert!(!html.contains(" id=\""));
    }

    /// `demo()` が決定的（呼び出しごとに同じ `Node`）であること。
    #[test]
    fn demo_is_deterministic() {
        assert_eq!(render(&demo()), render(&demo()));
    }

    /// PR #3364 Codex(P2) 指摘の回帰: 12 件のファイルへ同一プレースホルダー
    /// 画像のみを割り当てず、[`THUMBNAIL_SRCS`] の全種を使い回すこと。
    #[test]
    fn demo_cycles_through_all_thumbnail_srcs() {
        let html = render(&demo());
        for src in THUMBNAIL_SRCS {
            assert!(
                html.contains(src),
                "demo output should reference thumbnail src {src}"
            );
        }
    }

    /// [`LAYOUT_CSS`] がコンテナクエリ（2 → 3 → 4 列）と `:hover` 強調を
    /// 含むこと。
    #[test]
    fn layout_css_declares_container_query_column_steps_and_hover() {
        assert!(LAYOUT_CSS.contains("container-type: inline-size"));
        assert!(LAYOUT_CSS.contains("grid-template-columns: repeat(2, minmax(0, 1fr));"));
        assert!(
            LAYOUT_CSS.contains("@container blocks-grid-list-file-thumbnails (min-width: 30rem)")
        );
        assert!(
            LAYOUT_CSS.contains("@container blocks-grid-list-file-thumbnails (min-width: 40rem)")
        );
        assert!(LAYOUT_CSS.contains(":hover"));
    }

    /// PR #3364 codex(P2) 指摘の回帰: メタ情報の段落が `.docs-content p` の
    /// 既定下余白を持ち込まないよう `margin: 0` を明示していること。
    #[test]
    fn layout_css_resets_meta_paragraph_margin() {
        assert!(
            LAYOUT_CSS.contains("[data-blocks-grid-list-file-thumbnails-meta] p {\n  margin: 0;\n")
        );
    }
}
