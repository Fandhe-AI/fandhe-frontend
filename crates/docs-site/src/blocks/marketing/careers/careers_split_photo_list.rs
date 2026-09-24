//! `careers-split-photo-list` block（イシュー #2817。親トラッキング #2807
//! 「Blocks マーケティング B」配下、対応表 ID R0778 の 1 件のみを構造の
//! 参照元とする合成例。写真付き見出し + 求人リスト）。取得手段・
//! ファイル名・内部コンポーネント識別子は記載しない
//! （`docs/design/motion-reference-adoption-policy.md` §9 と同じライセンス
//! 上の転記制限）。参照元の分類は blog だが、右列が求人一覧のため careers
//! として扱う（構造のみ参照し、文言・配色・装飾・アイコンは持ち込まない）。
//!
//! # 使用部品
//!
//! `heading` / `text` / `image` / `separator` / `link` / `link-overlay` /
//! `visually-hidden` の 7 部品を合成する（[`BLOCK`] の `parts` に一致させる
//! 契約、`crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が
//! 検証する）。
//!
//! # レイアウト（lg = 64rem をブレークポイントとする理由）
//!
//! `< 64rem` は見出し・説明・写真の下に求人リストが続く 1 カラム、
//! `>= 64rem` で左に見出し・説明・写真、右に求人リストの 2 カラムへ切り
//! 替える。テーマの breakpoint トークンは `@media` 条件式の中では解決
//! できないため（CSS custom property は宣言側でのみ有効）、
//! `fandhe_frontend_pre_styled_ui::recipe::Breakpoint` の `Lg`（1024px =
//! 64rem）と一致するリテラル値を [`LAYOUT_CSS`] へ直書きする
//! （`blog_list_image` と同じ判断）。ルート grid class は [`demo`] 直下の
//! `blocks-careers-split-photo-list-layout` へ適用し、[`Block::demo_class`]
//! （`blocks-careers-split-photo-list`）とは意図的に別名にする
//! （`blog_list_image` モジュール doc「レイアウト」節と同じ Bugbot 教訓の
//! 回避）。
//!
//! # `dl`/`dt`/`dd` を再現しない（原稿「原案差分メモ」参照）
//!
//! 参照元は `dl` に sr-only の `dt` を置く構造だが、
//! [`visually_hidden::root`] は `<span>`（phrasing content）のため、`dt`
//! や見出しの子だけを隠すと親要素が flow 内に残り、flex の gap や見出しの
//! 余白が生じる。そこで求人 1 件は `div` と [`link_overlay::root`] で組み、
//! `visually_hidden::root` は可視の給与・勤務地テキストの内側に置いて
//! スクリーンリーダー向けのラベル（「給与：」「勤務地：」）を補う。
//! sr-only のリスト見出し（参照元の「求人一覧」の h3 相当）は作らない
//! （空の見出し要素による余白を避けるため。左列の見出しがセクション全体の
//! 見出しを担う）。
//!
//! # 罫線 = `separator`（`<hr>`）を求人の間に 2 本
//!
//! `<hr>` は `div` の直下に置く（`ul`/`li` は使わない。`<hr>` は
//! リストアイテムの直下に置けないため）。
//!
//! # 全面リンク = `link_overlay`
//!
//! [`link_overlay::overlay`] の `aria-label` は可視の職種名と一致させる。
//! 行の内側に他のクリック要素は置かない（overlay は `z-index: 0` で
//! 子要素を覆うため）。
//!
//! # `link` はリスト下の「すべての募集を見る」フッターリンク
//!
//! [`link_overlay::root`] の**外**、兄弟の位置に置く（overlay に覆われ
//! ないため）。参照元にある「すべての募集を見る」フッターに相当する
//! 構造である。
//!
//! # 写真（参照元との差分）
//!
//! [`AspectRatio::Landscape`]（4:3）を狭い幅・広い幅の両方で固定し、
//! `width: 100%` で列幅に合わせて縮める。参照元の 6:5 と lg 時の固定高
//! 34.5rem は再現しない（`AspectRatio` に 6:5 は無く、固定高は狭い親で
//! 比率を崩すため。原稿「原案差分メモ」参照）。`src` は
//! [`dummy_assets::BACKGROUND_SRC`]（`data:` URI は使わない、
//! `crate::blocks` モジュール doc「セキュリティ不変条件」節）。`alt=""`
//! の装飾扱いとする（抽象図形のプレースホルダーで情報を持たず、クリック
//! 範囲にも含めない）。
//!
//! # 見出しレベル
//!
//! ページ側が `## Demo` として `h2` を出すため、セクション見出しは
//! [`HeadingLevel::H3`]、職種名は [`HeadingLevel::H4`] にする
//! （`blog_list_image` と同じ判断）。
//!
//! # `href="#"` を使わない・`base_path` を受け取れない制約
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節と同じ判断軸で、
//! [`Block::demo`] は `fn() -> Node` のため `base_path` を受け取れない。
//! `linkcheck::check_links` は外部リンクを検証対象外とするため、リンク先は
//! すべて [`REPO`]（実在する GitHub リポジトリへの外部絶対 URL）に固定する
//! （`blog_list_image` と同じ先例判断）。
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
//! `heading::heading` / `text::text` / `image::image` /
//! `separator::separator` / `link::root` / `link_overlay::root` /
//! `visually_hidden::root` はいずれも `drop_class_attr` により呼び出し側
//! `attrs` の `class` を黙って除去する契約を持つため、Demo 固有のスタイル
//! フックは `data-blocks-careers-split-photo-list-*` 属性で渡し、
//! [`LAYOUT_CSS`] 側も同じ属性セレクタで対応する。素の `div`/`span` には
//! `class` がそのまま効くため、それらは従来どおり
//! `.blocks-careers-split-photo-list-*` クラスセレクタを使う。
//!
//! # `id`/`aria-describedby` を出力しない
//!
//! 宙に浮いた ARIA 参照や id 重複を構造的に避けるため、求人・給与・
//! 勤務地・写真のいずれも `id` を持たない。
//!
//! # `<form>` を持たない・応募処理を行わない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・送信処理・データ取得を持たない
//! 静的な合成例である。応募処理等のアプリケーションロジックは利用者側の
//! 責務（`docs/policy/intentional-non-adoption.md` §3.25）。文言・給与・
//! 勤務地はすべて架空のもの（実企業名・実クレデンシャル・PII を含まない）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, span, text, Node};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::image::{self, AspectRatio, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::link_overlay::{self, overlay};
use fandhe_frontend_pre_styled_ui::separator::{separator, SeparatorProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::visually_hidden;

/// リンク先の固定外部 URL（モジュール doc「`href="#"` を使わない」節参照）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// 求人 1 件分のダミーデータ（架空、実在の人物・企業とは無関係）。
struct Job {
    title: &'static str,
    description: &'static str,
    salary: &'static str,
    location: &'static str,
}

/// 求人一覧（架空、3 件）。
const JOBS: [Job; 3] = [
    Job {
        title: "バックエンドエンジニア",
        description: "描画コアとサーバーサイド API の設計・実装を担当します。",
        salary: "年収 600万〜900万円",
        location: "東京（リモート可）",
    },
    Job {
        title: "プロダクトデザイナー",
        description: "UI コンポーネント層のビジュアルデザインとアクセシビリティ検証を担当します。",
        salary: "年収 550万〜850万円",
        location: "大阪（リモート可）",
    },
    Job {
        title: "カスタマーサクセス",
        description: "導入企業への技術サポートとフィードバック収集を担当します。",
        salary: "年収 450万〜650万円",
        location: "フルリモート",
    },
];

/// 求人 1 件の行（見出し + 説明 + 給与・勤務地 + 全面リンク）。
///
/// `dl`/`dt`/`dd` を再現せず `div` + [`link_overlay::root`] で組む（モジュール
/// doc「`dl`/`dt`/`dd` を再現しない」節）。給与・勤務地はスクリーンリーダー
/// 向けラベルを [`visually_hidden::root`] で可視テキストの内側に補う。
/// `role="listitem"` を付与し、[`demo`] 側の `role="list"` コンテナと対で
/// 一覧構造をアクセシビリティツリーへ公開する（`<hr>` を `<li>` 直下に
/// 置けないため `ul`/`li` は使えず、ARIA role で代替する判断）。
fn job_item(job: &Job) -> Node {
    link_overlay::root(
        vec![
            ("data-blocks-careers-split-photo-list-job", ""),
            ("role", "listitem"),
        ],
        vec![
            heading(
                HeadingLevel::H4,
                &HeadingProps::default(),
                vec![],
                vec![text(job.title)],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(job.description)],
            ),
            div(
                vec![("class", "blocks-careers-split-photo-list-meta")],
                vec![
                    styled_text::text(
                        &TextProps {
                            size: TextSize::Sm,
                            ..TextProps::default()
                        },
                        vec![("data-blocks-careers-split-photo-list-salary", "")],
                        vec![
                            visually_hidden::root(vec![], vec![text("給与：")]),
                            text(job.salary),
                        ],
                    ),
                    span(
                        vec![
                            ("class", "blocks-careers-split-photo-list-dot"),
                            ("aria-hidden", "true"),
                        ],
                        vec![text("・")],
                    ),
                    styled_text::text(
                        &TextProps {
                            size: TextSize::Sm,
                            ..TextProps::default()
                        },
                        vec![("data-blocks-careers-split-photo-list-location", "")],
                        vec![
                            visually_hidden::root(vec![], vec![text("勤務地：")]),
                            text(job.location),
                        ],
                    ),
                ],
            ),
            overlay(REPO, vec![("aria-label", job.title)], vec![]),
        ],
    )
}

/// `careers-split-photo-list` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数（モジュール doc「レイアウト」節）。
pub fn demo() -> Node {
    let intro = div(
        vec![("class", "blocks-careers-split-photo-list-intro")],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    weight: HeadingWeight::Bold,
                },
                vec![],
                vec![text("一緒にプロダクトを育てる仲間を募集しています")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(
                    "描画コアからドキュメントサイトまで、フレームワーク全体を横断して開発する仲間を探しています。",
                )],
            ),
            div(
                vec![("class", "blocks-careers-split-photo-list-figure")],
                vec![image::image(
                    &ImageProps {
                        aspect_ratio: AspectRatio::Landscape,
                        shape: ImageShape::Rounded,
                        ..ImageProps::new(dummy_assets::BACKGROUND_SRC, "")
                    },
                    vec![("data-blocks-careers-split-photo-list-photo", "")],
                )],
            ),
        ],
    );

    let list_items: Vec<Node> = JOBS
        .iter()
        .enumerate()
        .flat_map(|(index, job)| {
            let mut nodes = vec![job_item(job)];
            if index + 1 < JOBS.len() {
                nodes.push(separator(
                    &SeparatorProps::default(),
                    vec![("data-blocks-careers-split-photo-list-separator", "")],
                ));
            }
            nodes
        })
        .collect();

    let jobs = div(
        vec![("class", "blocks-careers-split-photo-list-jobs")],
        vec![
            div(
                vec![
                    ("class", "blocks-careers-split-photo-list-list"),
                    ("role", "list"),
                ],
                list_items,
            ),
            div(
                vec![("class", "blocks-careers-split-photo-list-footer")],
                vec![link::root(
                    REPO,
                    &LinkProps::default(),
                    vec![("data-blocks-careers-split-photo-list-all-link", "")],
                    vec![
                        text("すべての募集を見る"),
                        span(vec![("aria-hidden", "true")], vec![text("→")]),
                    ],
                )],
            ),
        ],
    );

    div(
        vec![("class", "blocks-careers-split-photo-list-layout")],
        vec![intro, jobs],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/careers-split-photo-list/",
    title: "careers-split-photo-list",
    category: BlockCategory::Careers,
    rust_source: "crates/docs-site/src/blocks/marketing/careers/careers_split_photo_list.rs",
    demo_class: "blocks-careers-split-photo-list",
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
            label: "Image",
            path: "/themes/image/",
        },
        Part {
            label: "Separator",
            path: "/themes/separator/",
        },
        Part {
            label: "Link",
            path: "/themes/link/",
        },
        Part {
            label: "Link Overlay",
            path: "/themes/link-overlay/",
        },
        Part {
            label: "Visually Hidden",
            path: "/themes/visually-hidden/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `careers_split_photo_list` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節。他 block と同型で `pub(super)` ではなく
/// 本ファイル内 private 定数として `super::stylesheet` 経由の `push_css` で
/// 連結される）。
///
/// セレクタは `.blocks-careers-split-photo-list-*` と
/// `[data-blocks-careers-split-photo-list-*]` のみを用い、他 block や部品の
/// 素のセレクタへ影響させない（`blog_list_image` と同じ名前空間分離）。
const LAYOUT_CSS: &str = "\
.blocks-careers-split-photo-list-layout {\n  display: grid;\n  gap: var(--fandhe-space-12);\n}\n\
.blocks-careers-split-photo-list-intro {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-careers-split-photo-list-figure {\n  width: 100%;\n}\n\
[data-blocks-careers-split-photo-list-photo] {\n  display: block;\n  width: 100%;\n}\n\
.blocks-careers-split-photo-list-jobs {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n}\n\
.blocks-careers-split-photo-list-list {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n}\n\
[data-blocks-careers-split-photo-list-job] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n  padding-block: var(--fandhe-space-2);\n}\n\
[data-blocks-careers-split-photo-list-separator] {\n  margin: 0;\n}\n\
.blocks-careers-split-photo-list-meta {\n  display: flex;\n  flex-wrap: wrap;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-careers-split-photo-list-dot {\n  color: var(--fandhe-color-fg-muted);\n}\n\
@media (min-width: 64rem) {\n  .blocks-careers-split-photo-list-layout {\n    grid-template-columns: repeat(2, minmax(0, 1fr));\n    gap: var(--fandhe-space-16);\n    align-items: start;\n  }\n}\n";

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
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"image\"",
            "data-scope=\"separator\"",
            "data-scope=\"link\"",
            "data-scope=\"link-overlay\"",
            "data-scope=\"visually-hidden\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert_eq!(
            html.matches("data-blocks-careers-split-photo-list-job")
                .count(),
            3,
            "demo should render exactly 3 job rows"
        );
        assert_eq!(
            html.matches("data-blocks-careers-split-photo-list-separator")
                .count(),
            2,
            "demo should render exactly 2 separators (N-1 for 3 jobs)"
        );
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("<form"));
        assert!(!html.contains("src=\"data:"));
        assert!(!html.contains("id=\""));
        assert!(!html.contains("<ul"));
    }

    /// [`LAYOUT_CSS`] が想定するブレークポイント条件・2 カラムを持つこと。
    #[test]
    fn layout_css_declares_lg_breakpoint_and_two_columns() {
        assert!(LAYOUT_CSS.contains("@media (min-width: 64rem)"));
        assert!(LAYOUT_CSS.contains("grid-template-columns"));
    }

    /// ルート grid class（`demo_class` とは別名）が `demo()` の出力へ
    /// 実際に現れること（モジュール doc「レイアウト」節の Bugbot 教訓の
    /// 固定、`blog_list_image` と同じ回帰）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-careers-split-photo-list-layout\""));
        assert_ne!(
            super::BLOCK.demo_class,
            "blocks-careers-split-photo-list-layout"
        );
    }
}
