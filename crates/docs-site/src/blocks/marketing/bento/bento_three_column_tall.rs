//! `bento-three-column-tall` block（イシュー #2748/#2749。親 #2747「Blocks
//! 目的別パーツ拡充ツリー」#2730 配下、両端のセルが縦 2 行にまたがる
//! 3 列 bento の骨格・主要領域（#2748）と、残り領域・状態表示・原稿の
//! 仕上げ（#2749）を実装する。
//!
//! # 使用部品
//!
//! `badge`（eyebrow）+ `heading`（見出し帯 H3・各セル見出し H4）+ `text`
//! （リード文・各セル説明文）+ `button`（CTA）+ `card`（セル本体）+
//! `image`（画像メディア）+ `code`（端末風の枠・コード表示枠の行）の
//! 7 部品を合成する（[`BLOCK`] の `parts` に一致させる契約）。
//!
//! # レイアウト（両端セルが縦 2 行にまたがる 3 列グリッド、2 形態を並記）
//!
//! `>= 64rem`（`fandhe_frontend_pre_styled_ui::recipe::Breakpoint::Lg` =
//! 1024px と一致するリテラル値、`blog_list_image`/
//! `blog_featured_with_list` と同じ判断。テーマの breakpoint トークンは
//! `@media` 条件式の中では解決できないため直書きする）で 3 列 2 行の
//! グリッドにする。本 Demo は 2 形態を並記する:
//!
//! - **基準形（`both-tall`）**: 1 枚目・4 枚目のセルを `grid-row: span 2`
//!   で縦 2 行へまたがせ、2 枚目・3 枚目を中央列の 1 行目・2 行目へ積む
//!   （4 枚セル）。
//! - **1 枚目のみ縦長（`start-tall`）**: 1 枚目のみ縦長のまま、右列
//!   （`end-top`/`end-bottom`）にも 2 枚を積む（5 枚セル）。
//!
//! `< 64rem` では両形態とも 1 列積みへ切り替え、`grid-row` を一切指定
//! しないため縦長セルも自動的に通常の高さへ戻る（明示的な打ち消し宣言を
//! 書く必要がない）。
//!
//! # 見出し帯 + CTA
//!
//! 見出し帯（eyebrow badge + 見出し + リード文）の右側（`>= 64rem`）に
//! CTA ボタンを 1 個配置する（`button::button`、`type="button"` 固定・
//! 遷移先を持たない）。2 形態で共有するヘッダーへ畳み込み、CTA は 1 回
//! だけ出す。
//!
//! # メディア 3 種（画像 / 端末風の枠 / コード表示枠）
//!
//! 各セルのカバー領域（`card::cover`）には次のいずれかを置く:
//!
//! - **画像**（[`image::image`]）: ビルド時生成のモノトーン抽象図形
//!   プレースホルダー。
//! - **端末風の枠**（[`Media::Terminal`]）: `hero_terminal` と同型の
//!   titlebar（dot 3 個）+ プロンプト + `code::code` の行を並べる。
//! - **コード表示枠**（[`Media::CodeWindow`]）: ファイル名タブ +
//!   `code::code` の行を並べる。
//!
//! いずれも `@keyframes`/`animation` を持たない静的表示である
//! （`hero_terminal` の時間軸 stagger とは異なり、本 Demo は無 JS
//! docs-site 上でも常に完成形が見える構成を優先する）。
//!
//! # 見出しレベルに `H3`/`H4` を使う理由
//!
//! ページ側が `## Demo` として `h2` を出すため、見出し帯は
//! `HeadingLevel::H3`、各セル見出しは `HeadingLevel::H4` にする
//! （`blog_list_image` と同じ判断）。`card::title` は `<h3>` 固定のため
//! セル見出しには使わず、`heading(HeadingLevel::H4, ...)` を `card::body`
//! 直下へ子として渡す。
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
//! `badge::badge`/`heading::heading`/`text::text`/`image::image`/
//! `card::root`/`code::code` はいずれも `drop_class_attr` により呼び出し側
//! `attrs` の `class` を黙って除去する契約を持つため、Demo 固有のスタイル
//! フックは `data-blocks-bento-three-column-tall-*` 属性で渡し、
//! [`LAYOUT_CSS`] 側も同じ属性セレクタで対応する。各セルの配置
//! （`grid-column`/`grid-row`）は `data-blocks-bento-three-column-tall-cell`
//! の値（`start`/`center-top`/`center-bottom`/`end`/`end-top`/
//! `end-bottom`）へ紐づける。`button::button` も同じ契約を持つため CTA は
//! `data-blocks-bento-three-column-tall-cta` を使う。素の `div`/`span`/`p`
//! には `class` がそのまま効くため、レイアウト用ラッパーは従来どおり
//! `.blocks-bento-three-column-tall-*` クラスセレクタを使う
//! （`crate::blocks` モジュール doc「CSS フックが `class` と `[data-*]` で
//! 混在する理由」節と同じ判断軸）。
//!
//! # `id` 属性・aria 参照を持たない
//!
//! `crates/docs-site/tests/blocks_contract.rs` の
//! `demo_output_has_no_dangling_aria_references_or_duplicate_ids` が
//! 全 block 横断で検証するため、本 Demo は `id`/`aria-controls`/
//! `aria-labelledby`/`aria-describedby` のいずれも出力しない。
//!
//! # `<form>` を持たない・実データを持たない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない。文言・画像・コマンド行・設定値はすべて架空のもの
//! （`dummy_assets` のビルド時生成プレースホルダーを含む）であり、実企業
//! 名・実クレデンシャル・PII を含まない。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, p, span, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::code::{self, CodeProps};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::image::{self, AspectRatio, ImageProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};

/// 1 セルのカバー領域に置くメディア種別（画像 / 端末風の枠 / コード
/// 表示枠）。[`cell`] が `slot` ごとに 1 種類を選び分ける。
enum Media {
    /// ビルド時生成のモノトーン抽象図形プレースホルダー。
    Image {
        src: &'static str,
        aspect: AspectRatio,
    },
    /// `hero_terminal` と同型の端末風の枠（titlebar + プロンプト行）。
    Terminal { lines: &'static [&'static str] },
    /// ファイル名タブ付きのコード表示枠。
    CodeWindow {
        filename: &'static str,
        lines: &'static [&'static str],
    },
}

/// 1 枚分のセルデータ（架空の開発者向けプラットフォームの機能紹介、
/// 実企業名・実サービス名は含まない）。`slot` はグリッド内の配置を表す
/// 識別子で、[`LAYOUT_CSS`] の `[data-blocks-bento-three-column-tall-cell]`
/// セレクタの値と一致させる。
struct BentoCell {
    slot: &'static str,
    title: &'static str,
    description: &'static str,
    media: Media,
}

/// 基準形（`both-tall`、対応表 ID R0410 + R0768）: 両端セルが縦長で、
/// メディアは画像・端末風の枠・コード表示枠を混在させる（R0768 の
/// 「セルごとにメディアが異なる」を取り込む）。
const BOTH_TALL_CELLS: [BentoCell; 4] = [
    BentoCell {
        slot: "start",
        title: "統合ダッシュボード",
        description: "複数サービスの稼働状況を 1 画面へ集約して表示します。",
        media: Media::Image {
            src: dummy_assets::SCREENSHOT_SRC,
            aspect: AspectRatio::Portrait,
        },
    },
    BentoCell {
        slot: "center-top",
        title: "自動デプロイ",
        description: "コミットからビルド・検証・配信までを自動化します。",
        media: Media::Terminal {
            lines: &[
                "fw deploy --target staging",
                "pipeline run --target staging",
                "pipeline status",
            ],
        },
    },
    BentoCell {
        slot: "center-bottom",
        title: "チーム権限管理",
        description: "ロールごとに閲覧・操作範囲を細かく制御します。",
        media: Media::Image {
            src: dummy_assets::LOGO_SRC,
            aspect: AspectRatio::Video,
        },
    },
    BentoCell {
        slot: "end",
        title: "利用量アラート",
        description: "しきい値を超えた利用量を検知し即座に通知します。",
        media: Media::CodeWindow {
            filename: "alerts.toml",
            lines: &[
                "[alert.api_requests]",
                "threshold = 10000",
                "window = \"1h\"",
                "notify = [\"email\"]",
            ],
        },
    },
];

/// 1 枚目のみ縦長（`start-tall`、対応表 ID R0107）: 見出し+CTA 行は共有
/// ヘッダーへ畳み込み済みのため、セル自体は基準形と重複しない架空の機能
/// 名を画像メディアで並べる。
const START_TALL_CELLS: [BentoCell; 5] = [
    BentoCell {
        slot: "start",
        title: "監視ビュー",
        description: "リクエスト数・エラー率をリアルタイムで確認できます。",
        media: Media::Image {
            src: dummy_assets::SCREENSHOT_SRC,
            aspect: AspectRatio::Portrait,
        },
    },
    BentoCell {
        slot: "center-top",
        title: "ビルドキャッシュ",
        description: "依存関係の再ビルドを省略し検証時間を短縮します。",
        media: Media::Image {
            src: dummy_assets::PRODUCT_SRC,
            aspect: AspectRatio::Video,
        },
    },
    BentoCell {
        slot: "center-bottom",
        title: "監査ログ",
        description: "操作履歴を改ざん不可能な形式で保存します。",
        media: Media::Image {
            src: dummy_assets::LOGO_SRC,
            aspect: AspectRatio::Video,
        },
    },
    BentoCell {
        slot: "end-top",
        title: "Webhook 連携",
        description: "外部サービスへイベントを即座に転送します。",
        media: Media::Image {
            src: dummy_assets::BACKGROUND_SRC,
            aspect: AspectRatio::Video,
        },
    },
    BentoCell {
        slot: "end-bottom",
        title: "API キー管理",
        description: "発行済みキーの利用範囲・有効期限を一覧管理します。",
        media: Media::Image {
            src: dummy_assets::PRODUCT_SRC,
            aspect: AspectRatio::Video,
        },
    },
];

/// 見出し帯（eyebrow badge + 見出し + リード文）。CTA ボタンは
/// [`header`] 側で右に添える。
fn intro() -> Node {
    div(
        vec![("class", "blocks-bento-three-column-tall-intro")],
        vec![
            badge::badge(
                &BadgeProps::default(),
                vec![],
                vec![text("プラットフォーム機能")],
            ),
            heading::heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    ..HeadingProps::default()
                },
                vec![],
                vec![text("必要な機能をひとつの基盤に")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(
                    "運用・デプロイ・権限管理をまとめて提供する、開発者向けプラットフォームの主要機能です。",
                )],
            ),
        ],
    )
}

/// 見出し帯 + CTA の行（2 形態で共有する 1 個のヘッダー）。
fn header() -> Node {
    div(
        vec![("class", "blocks-bento-three-column-tall-header")],
        vec![
            intro(),
            button::button(
                &ButtonProps::default(),
                vec![("data-blocks-bento-three-column-tall-cta", "")],
                vec![text("機能一覧を見る")],
            ),
        ],
    )
}

/// プロンプト記号（端末風の枠の各行先頭）。
fn prompt() -> Node {
    span(
        vec![("class", "blocks-bento-three-column-tall-prompt")],
        vec![text("$ ")],
    )
}

/// [`Media`] をカバー領域の子ノードへ変換する。画像はそのまま
/// [`image::image`] へ委譲し、端末風の枠・コード表示枠は 1 行ずつ
/// `code::code` で描く静的表示にする（`@keyframes`/`animation` は使わない）。
fn media(item: &Media) -> Node {
    match item {
        Media::Image { src, aspect } => image::image(
            &ImageProps {
                aspect_ratio: *aspect,
                ..ImageProps::new(src, "")
            },
            vec![],
        ),
        Media::Terminal { lines } => {
            let dot = || {
                span(
                    vec![("class", "blocks-bento-three-column-tall-dot")],
                    vec![],
                )
            };
            let line_nodes: Vec<Node> = lines
                .iter()
                .map(|line| {
                    div(
                        vec![("class", "blocks-bento-three-column-tall-line")],
                        vec![
                            prompt(),
                            code::code(&CodeProps::default(), vec![], vec![text(*line)]),
                        ],
                    )
                })
                .collect();
            div(
                vec![("data-blocks-bento-three-column-tall-media", "terminal")],
                vec![
                    div(
                        vec![("class", "blocks-bento-three-column-tall-titlebar")],
                        vec![dot(), dot(), dot()],
                    ),
                    div(
                        vec![("class", "blocks-bento-three-column-tall-lines")],
                        line_nodes,
                    ),
                ],
            )
        }
        Media::CodeWindow { filename, lines } => {
            let line_nodes: Vec<Node> = lines
                .iter()
                .map(|line| {
                    div(
                        vec![("class", "blocks-bento-three-column-tall-line")],
                        vec![code::code(&CodeProps::default(), vec![], vec![text(*line)])],
                    )
                })
                .collect();
            div(
                vec![("data-blocks-bento-three-column-tall-media", "code")],
                vec![
                    div(
                        vec![("class", "blocks-bento-three-column-tall-tabbar")],
                        vec![span(
                            vec![("class", "blocks-bento-three-column-tall-tab")],
                            vec![text(*filename)],
                        )],
                    ),
                    div(
                        vec![("class", "blocks-bento-three-column-tall-lines")],
                        line_nodes,
                    ),
                ],
            )
        }
    }
}

/// 1 枚分の bento セル（`card` + カバーメディア + 見出し/説明）を組み立
/// てる。
fn cell(item: &BentoCell) -> Node {
    card::root(
        CardProps::default(),
        vec![("data-blocks-bento-three-column-tall-cell", item.slot)],
        vec![
            card::cover(
                vec![("data-blocks-bento-three-column-tall-cover", "")],
                vec![media(&item.media)],
            ),
            card::body(
                vec![("class", "blocks-bento-three-column-tall-body")],
                vec![
                    heading::heading(
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
                        vec![],
                        vec![text(item.description)],
                    ),
                ],
            ),
        ],
    )
}

/// 1 形態ぶんの並記ブロック（キャプション + グリッド）。`key` は
/// [`LAYOUT_CSS`] の `data-blocks-bento-three-column-tall-variant` 値。
fn variant(key: &'static str, caption: &'static str, cells: &[BentoCell]) -> Node {
    let cell_nodes: Vec<Node> = cells.iter().map(cell).collect();
    div(
        vec![
            ("class", "blocks-bento-three-column-tall-variant"),
            ("data-blocks-bento-three-column-tall-variant", key),
        ],
        vec![
            p(
                vec![("class", "blocks-bento-three-column-tall-caption")],
                vec![text(caption)],
            ),
            div(
                vec![("class", "blocks-bento-three-column-tall-grid")],
                cell_nodes,
            ),
        ],
    )
}

/// `bento-three-column-tall` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数（見出し帯 + CTA、両端が縦長の基準形、1 枚目のみ縦長の 2
/// 形態を並記する、モジュール doc「レイアウト」節）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-bento-three-column-tall")],
        vec![
            header(),
            variant("both-tall", "両端のセルが縦長（基準形）", &BOTH_TALL_CELLS),
            variant("start-tall", "1 枚目のみ縦長", &START_TALL_CELLS),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/bento-three-column-tall/",
    title: "bento-three-column-tall",
    category: BlockCategory::Bento,
    rust_source: "crates/docs-site/src/blocks/marketing/bento/bento_three_column_tall.rs",
    demo_class: "blocks-bento-three-column-tall",
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
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Card",
            path: "/themes/card/",
        },
        Part {
            label: "Image",
            path: "/themes/image/",
        },
        Part {
            label: "Code",
            path: "/themes/code/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `bento_three_column_tall` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節。他 block と同型で本ファイル内 private
/// 定数として `super::stylesheet` 経由の `push_css` で連結される）。
///
/// セレクタは `.blocks-bento-three-column-tall-*` と
/// `[data-blocks-bento-three-column-tall-*]` のみを用い、他 block や
/// 部品の素のセレクタへ影響させない。生の hex 色は使わずテーマトークン
/// （`--fandhe-*`）のみを参照する。
const LAYOUT_CSS: &str = "\
.blocks-bento-three-column-tall {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n}\n\
.blocks-bento-three-column-tall-header {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-bento-three-column-tall-intro {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n  max-width: 40rem;\n}\n\
.blocks-bento-three-column-tall-variant {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-bento-three-column-tall-caption {\n  margin: 0;\n  font-size: var(--fandhe-font-font-size-sm);\n  color: var(--fandhe-color-fg-muted);\n}\n\
.blocks-bento-three-column-tall-grid {\n  display: grid;\n  grid-template-columns: minmax(0, 1fr);\n  gap: var(--fandhe-space-4);\n}\n\
[data-blocks-bento-three-column-tall-cell] {\n  display: flex;\n  flex-direction: column;\n  height: 100%;\n}\n\
.blocks-bento-three-column-tall-body {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n  flex: 1;\n}\n\
[data-blocks-bento-three-column-tall-media] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n  min-height: 10rem;\n  height: 100%;\n  box-sizing: border-box;\n  padding: var(--fandhe-space-4);\n  background: var(--fandhe-color-bg-muted);\n  font-family: var(--fandhe-font-font-mono, monospace);\n  font-size: var(--fandhe-font-font-size-sm);\n}\n\
.blocks-bento-three-column-tall-titlebar {\n  display: flex;\n  gap: 0.4rem;\n}\n\
.blocks-bento-three-column-tall-dot {\n  display: inline-block;\n  width: 0.6rem;\n  height: 0.6rem;\n  border-radius: 999px;\n  background: currentColor;\n  opacity: 0.35;\n}\n\
.blocks-bento-three-column-tall-tabbar {\n  display: flex;\n  border-bottom: 1px solid var(--fandhe-color-border);\n  padding-bottom: var(--fandhe-space-2);\n}\n\
.blocks-bento-three-column-tall-tab {\n  font-size: var(--fandhe-font-font-size-sm);\n  opacity: 0.8;\n}\n\
.blocks-bento-three-column-tall-lines {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-bento-three-column-tall-line {\n  display: flex;\n  align-items: baseline;\n}\n\
.blocks-bento-three-column-tall-prompt {\n  opacity: 0.6;\n  margin-right: 0.25rem;\n}\n\
@media (min-width: 64rem) {\n  .blocks-bento-three-column-tall-header {\n    flex-direction: row;\n    justify-content: space-between;\n    align-items: flex-end;\n  }\n  [data-blocks-bento-three-column-tall-cta] {\n    flex-shrink: 0;\n  }\n  .blocks-bento-three-column-tall-grid {\n    grid-template-columns: repeat(3, minmax(0, 1fr));\n    grid-template-rows: repeat(2, auto);\n  }\n  [data-blocks-bento-three-column-tall-cell=\"start\"] {\n    grid-column: 1;\n    grid-row: 1 / span 2;\n  }\n  [data-blocks-bento-three-column-tall-cell=\"center-top\"] {\n    grid-column: 2;\n    grid-row: 1;\n  }\n  [data-blocks-bento-three-column-tall-cell=\"center-bottom\"] {\n    grid-column: 2;\n    grid-row: 2;\n  }\n  [data-blocks-bento-three-column-tall-cell=\"end\"] {\n    grid-column: 3;\n    grid-row: 1 / span 2;\n  }\n  [data-blocks-bento-three-column-tall-cell=\"end-top\"] {\n    grid-column: 3;\n    grid-row: 1;\n  }\n  [data-blocks-bento-three-column-tall-cell=\"end-bottom\"] {\n    grid-column: 3;\n    grid-row: 2;\n  }\n  [data-blocks-bento-three-column-tall-cell=\"start\"] [data-blocks-bento-three-column-tall-cover],\n  [data-blocks-bento-three-column-tall-cell=\"end\"] [data-blocks-bento-three-column-tall-cover] {\n    flex: 1;\n  }\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// Demo が期待する部品・構造・非対話制約を満たしていることの単体
    /// 回帰（`crates/docs-site/tests/blocks_contract.rs` の横断検査と重複
    /// し過ぎない範囲での個別固定）。
    #[test]
    fn demo_composes_expected_parts_and_avoids_dead_links() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"badge\"",
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"button\"",
            "data-scope=\"card\" data-part=\"root\"",
            "data-scope=\"image\"",
            "data-scope=\"code\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert_eq!(
            html.matches("data-scope=\"card\" data-part=\"root\"")
                .count(),
            9,
            "bento-three-column-tall should render exactly 9 cards (4 + 5)"
        );
        for (slot, expected) in [
            ("start", 2),
            ("center-top", 2),
            ("center-bottom", 2),
            ("end", 1),
            ("end-top", 1),
            ("end-bottom", 1),
        ] {
            let needle = format!("data-blocks-bento-three-column-tall-cell=\"{slot}\"");
            assert_eq!(
                html.matches(&needle).count(),
                expected,
                "demo output should place {expected} cell(s) at slot {slot}"
            );
        }
        assert_eq!(
            html.matches("data-blocks-bento-three-column-tall-media=\"terminal\"")
                .count(),
            1,
            "demo output should render exactly one terminal-style media"
        );
        assert_eq!(
            html.matches("data-blocks-bento-three-column-tall-media=\"code\"")
                .count(),
            1,
            "demo output should render exactly one code-window media"
        );
        assert_eq!(
            html.matches("type=\"button\"").count(),
            1,
            "demo output should render exactly one CTA button"
        );
        for variant_key in ["both-tall", "start-tall"] {
            let needle = format!("data-blocks-bento-three-column-tall-variant=\"{variant_key}\"");
            assert!(
                html.contains(&needle),
                "demo output should render the {variant_key} variant"
            );
        }
        for absent in ["<form", "<script", "src=\"data:", " id=\"", "href=\"#\""] {
            assert!(
                !html.contains(absent),
                "bento-three-column-tall should never contain {absent}"
            );
        }
    }

    /// [`LAYOUT_CSS`] が想定するブレークポイント条件・グリッド配置・
    /// 追加した `end-top`/`end-bottom` の配置規則を持ち、
    /// `@keyframes`/`animation` を一切使わない静的表示であること。
    #[test]
    fn layout_css_declares_lg_breakpoint_and_span_two_cells() {
        assert!(LAYOUT_CSS.contains("@media (min-width: 64rem)"));
        assert!(LAYOUT_CSS.contains("grid-template-columns: repeat(3"));
        assert!(LAYOUT_CSS.contains("grid-row: 1 / span 2"));
        assert!(LAYOUT_CSS.contains("[data-blocks-bento-three-column-tall-cell"));
        assert!(LAYOUT_CSS.contains("[data-blocks-bento-three-column-tall-cell=\"end-top\"]"));
        assert!(LAYOUT_CSS.contains("[data-blocks-bento-three-column-tall-cell=\"end-bottom\"]"));
        assert!(
            LAYOUT_CSS.contains("[data-blocks-bento-three-column-tall-cta]"),
            "CTA の data-* フックには対応するスタイルルールを持たせる"
        );
        assert!(
            !LAYOUT_CSS.contains("@keyframes"),
            "bento-three-column-tall media should be a static display without animation"
        );
        assert!(
            !LAYOUT_CSS.contains("animation"),
            "bento-three-column-tall media should be a static display without animation"
        );
    }
}
