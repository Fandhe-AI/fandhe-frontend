//! `bento-asymmetric-rows` block（イシュー #2745/#2746。親トラッキング
//! #2744「Blocks 目的別パーツ拡充ツリー」・Marketing/Bento カテゴリ配下、
//! 規模 L のため前半 #2745（骨格・基準形）と後半 #2746（本コミット、
//! 残りのバリエーション・状態表示・原稿仕上げ）へ分割済み）。
//!
//! # 使用部品
//!
//! `badge` / `heading` / `text` / `card` / `image` / `icon` の 6 部品を
//! 合成する（[`BLOCK`] の `parts` に一致させる契約）。新しい UI 部品は
//! 追加しない。
//!
//! # 4 バリエーションと対応表 ID の対応
//!
//! Demo は共通の見出しエリア 1 つの下に、キャプション付きで 4 つの
//! バリエーションを縦に並べる（`banner_full_width_bar` の複数インスタンス
//! 並記と同じ型）。本 block には開閉・切替の状態が無い（無 JS の静的
//! bento）ため、「状態違いの並記」はカードの見た目・列構成が異なる
//! バリエーションをキャプション付きで並べることとして扱う。
//!
//! 1. 基準形（R0769）: 6 列グリッドで 1 行目 4+2・2 行目 2+4（[`BASE_CELLS`]）。
//! 2. 分割形（R0412/R0770）: 6 列グリッドで 1 行目 3+3（`half` 2 枚）・
//!    2 行目 2+2+2（`narrow` 3 枚、[`SPLIT_CELLS`]）。
//! 3. ジグザグ形（R0411/R0415）: 3 列グリッドで 1 行目 2+1・2 行目 1+2、
//!    枠なし淡色カード（[`ZIGZAG_CELLS`]、[`card::CardVariant::Subtle`]）。
//!    R0411 の枠あり版は構造が枠なし版（R0415）と同じであるため 1
//!    インスタンスへ集約し、淡色側で見せる。
//! 4. 混在形（R0108）: 6 列グリッドで 4/2/3/3（`wide`/`narrow`/`half`/
//!    `half`、[`MIXED_CELLS`]）の上段 + 下段に 4 列の小さな feature 一覧
//!    （[`features`]）。元の並び（本文が先・画像が後、淡色カード）は
//!    親仕様の「画像が上」と [`card::CardVariant::Outline`] に統一する
//!    判断を採る。
//!
//! 取得手段・ファイル名・内部コンポーネント識別子は記載しない（購入者
//! 限定素材のライセンス上の転記制限、`docs/design/motion-reference-
//! adoption-policy.md` §9 と同じ方針。記載してよいのは対応表 ID のみ）。
//!
//! # 列モードと `wide`/`narrow`/`half` の意味づけ
//!
//! [`CellWidth`] の 3 値は列モード（[`GridColumns`]）に対する相対幅であり、
//! 属性値自体は列モードに関わらず同じ文字列
//! （`data-blocks-bento-asymmetric-rows-cell="wide"|"narrow"|"half"`）を
//! 使う。6 列モードでは `wide`=span 4・`narrow`=span 2・`half`=span 3、
//! 3 列モードでは `wide`=span 2・`narrow`=span 1（`half` は 3 列モードの
//! バリエーションでは使わない）に解決する。実際の `grid-column: span`
//! 切替は [`LAYOUT_CSS`] 側が `data-blocks-bento-asymmetric-rows-columns`
//! 属性を祖先セレクタへ併記することで行う（本 block ローカルの型のみで
//! 表現し `props::*` へは昇格しない、`alert::Severity`/
//! `progress::ProgressShape` と同じ判断軸）。
//!
//! # feature 一覧と `icon` の扱い
//!
//! 混在形（4 番目のバリエーション）の下段は `card` を使わない軽い一覧
//! （[`features`]）とし、各項目は `icon::icon`（自作の抽象チェックマーク
//! 図形。装飾用途のため `label: None` で `aria-hidden`）+ 太字の短い
//! タイトル + 説明文を縦に並べる。タイトルは見出し要素にせず
//! `styled_text::text`（[`TextWeight::Semibold`]）で表現し、文書アウト
//! ライン（`h3`/`h4`）を汚さない。
//!
//! # `drop_class_attr` を考慮した CSS フックの選び方
//!
//! `card::root` / `badge::badge` / `heading::heading` / `text::text`
//! （`styled_text` として import、`fandhe_frontend_core::text` との名前
//! 衝突を避けるため `blog_featured_article` と同じ判断）/ `image::image` /
//! `icon::icon` はいずれも `drop_class_attr` により呼び出し側 `attrs` の
//! `class` を黙って除去してから合成する契約を持つため、セルの幅区分・
//! 列モード・キャプション・feature 項目はすべて
//! `data-blocks-bento-asymmetric-rows-*` 属性で渡し、[`LAYOUT_CSS`] 側も
//! 同じ属性セレクタで対応する。`card::cover`/`card::body`・素の
//! `div`/`p` には `class` がそのまま効くため、それらは `class` で渡す。
//!
//! # セル本文の見出し・説明文の間隔
//!
//! `card::body` は `card::header` と異なり `gap` を持たない base スタイル
//! （`fandhe_frontend_pre_styled_ui::card` 参照）のため、`heading::heading`
//! と `card::description` をそのまま入れると余白なしで密着表示になる。
//! `card::title` を使わず素の `heading::heading` を消費する本 block では
//! `card::header` へ差し替える判断は採らず、`card::body` に
//! `blocks-bento-asymmetric-rows-body` class を付与して `gap` を持たせる
//! （[`LAYOUT_CSS`] 参照。Bugbot 指摘 #3162 で是正）。
//!
//! # 見出しレベル（H3/H4）の理由
//!
//! Demo は本文の `h2`「Demo」配下に挿入されるため、block 側の最上位見出しは
//! `<h3>` にする（`heading::heading` を素通しで消費）。`card::title` は
//! `<h3>` 固定でセル見出しに使うと文書アウトラインが重複するため、セルの
//! 見出しは `heading::heading(HeadingLevel::H4, ...)` にする（`card::title`
//! を使わない判断。`blog_featured_article` のグリッドカード見出しと同じ
//! 判断軸）。バリエーション間のキャプション（[`caption`]）は `h2`/`h3` を
//! 使わない素の `p`（右目次・折りたたみ目次に拾われないため、
//! `banner_full_width_bar`/`footer_newsletter` と同じ判断）。
//!
//! # レスポンシブをモバイルファースト（`min-width`）で書く理由
//!
//! 本 block の仕様は「md 未満 1 列・md 以上 2 列・lg 以上 6 列（または
//! 3 列ジグザグ）」という段階的な拡張であり、`max-width` 型の縮小記述より
//! `min-width` 型の拡張記述のほうが意図に忠実に読める。48rem/64rem は
//! `fandhe_frontend_pre_styled_ui::recipe::Breakpoint::Md`/`Lg` の
//! `min_width()`（768px/1024px、16px 基準の rem 換算）と一致する値であり、
//! 本ファイル末尾の `#[cfg(test)]` がドリフトを検知する。
//!
//! # `<form>` を使わない・実データを持たない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない。機能名・説明文はすべて架空のものであり、実企業名・実サービス
//! 名・実クレデンシャル・PII を含まない。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, p, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps, CardVariant};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::image::{self, AspectRatio, ImageProps};
use fandhe_frontend_pre_styled_ui::text::{
    self as styled_text, TextProps, TextSize, TextVariant, TextWeight,
};
use fandhe_frontend_pre_styled_ui::Size;

/// セルの幅区分（[`LAYOUT_CSS`] の `grid-column: span` を切り替える唯一の
/// 軸）。`props::*` へは昇格せず、本 block ローカルの列挙型とする
/// （`alert::Severity`/`progress::ProgressShape` と同じ判断軸）。列モード
/// （[`GridColumns`]）に対する相対幅であり、同じ属性値が列モードによって
/// 異なる `grid-column: span` へ解決される（モジュール doc「列モードと
/// `wide`/`narrow`/`half` の意味づけ」節参照）。
#[derive(Clone, Copy)]
enum CellWidth {
    /// 6 列中 4 列分・3 列中 2 列分。
    Wide,
    /// 6 列中 2 列分・3 列中 1 列分。
    Narrow,
    /// 6 列中 3 列分（3 列モードのバリエーションでは使わない）。
    Half,
}

impl CellWidth {
    /// [`LAYOUT_CSS`] の `[data-blocks-bento-asymmetric-rows-cell="..."]`
    /// セレクタと一致させる値。
    const fn value(self) -> &'static str {
        match self {
            CellWidth::Wide => "wide",
            CellWidth::Narrow => "narrow",
            CellWidth::Half => "half",
        }
    }
}

/// グリッドの列モード（[`grid`] ヘルパの引数）。`wide`/`narrow`/`half` の
/// 意味づけを列モードごとに切り替える（モジュール doc 参照）。
#[derive(Clone, Copy)]
enum GridColumns {
    /// 6 列グリッド（基準形・分割形・混在形）。
    Six,
    /// 3 列グリッド（ジグザグ形）。
    Three,
}

impl GridColumns {
    /// [`LAYOUT_CSS`] の
    /// `[data-blocks-bento-asymmetric-rows-columns="..."]` セレクタと
    /// 一致させる値。
    const fn value(self) -> &'static str {
        match self {
            GridColumns::Six => "six",
            GridColumns::Three => "three",
        }
    }
}

/// 1 枚分のセルデータ（架空の SaaS 機能名 + 1 行説明 + 幅区分）。
struct Cell {
    title: &'static str,
    description: &'static str,
    width: CellWidth,
}

/// 1 番目のバリエーション（基準形・R0769）: 1 行目 4+2、2 行目 2+4。
/// 並び順がグリッドの既定の自動配置（`grid-auto-flow: row`）と組み合わ
/// さってこの配置を作る不変条件を、本ファイル末尾の `#[cfg(test)]` が
/// 固定する。
const BASE_CELLS: [Cell; 4] = [
    Cell {
        title: "Unified Workspace",
        description: "散らばっていたツールを 1 つの画面に集約し、切り替えの手間を無くします。",
        width: CellWidth::Wide,
    },
    Cell {
        title: "Instant Handoff",
        description: "担当者の引き継ぎをワンクリックで完了します。",
        width: CellWidth::Narrow,
    },
    Cell {
        title: "Version History",
        description: "変更履歴を自動保存し、いつでも巻き戻せます。",
        width: CellWidth::Narrow,
    },
    Cell {
        title: "Cross-team Reporting",
        description: "部門をまたいだ進捗を 1 枚のレポートにまとめ、共有の手間を減らします。",
        width: CellWidth::Wide,
    },
];

/// 2 番目のバリエーション（分割形・R0412/R0770）: 1 行目 3+3（`half` 2
/// 枚）、2 行目 2+2+2（`narrow` 3 枚）。
const SPLIT_CELLS: [Cell; 5] = [
    Cell {
        title: "Shared Roadmap",
        description: "四半期の計画をチーム全員が同じ画面で追跡します。",
        width: CellWidth::Half,
    },
    Cell {
        title: "Automated Backups",
        description: "スナップショットを自動生成し、復元の手間を省きます。",
        width: CellWidth::Half,
    },
    Cell {
        title: "Custom Fields",
        description: "案件ごとに必要な項目だけを追加できます。",
        width: CellWidth::Narrow,
    },
    Cell {
        title: "Audit Trail",
        description: "誰が何を変更したかをいつでも確認できます。",
        width: CellWidth::Narrow,
    },
    Cell {
        title: "Bulk Actions",
        description: "複数件の操作をまとめて一度に実行します。",
        width: CellWidth::Narrow,
    },
];

/// 3 番目のバリエーション（ジグザグ形・R0411/R0415）: 3 列モードで
/// 1 行目 2+1、2 行目 1+2（[`GridColumns::Three`] + [`CardVariant::Subtle`]
/// と組み合わせて使う）。
const ZIGZAG_CELLS: [Cell; 4] = [
    Cell {
        title: "Quick Filters",
        description: "条件を組み合わせて必要な情報だけを絞り込みます。",
        width: CellWidth::Wide,
    },
    Cell {
        title: "Saved Views",
        description: "よく使う表示設定を保存していつでも呼び出せます。",
        width: CellWidth::Narrow,
    },
    Cell {
        title: "Inline Comments",
        description: "作業中の項目に直接コメントを残せます。",
        width: CellWidth::Narrow,
    },
    Cell {
        title: "Export to CSV",
        description: "表示中のデータをそのままファイルへ書き出せます。",
        width: CellWidth::Wide,
    },
];

/// 4 番目のバリエーション（混在形・R0108 上段）: 4/2/3/3
/// （`wide`/`narrow`/`half`/`half`）。下段は [`FEATURES`]/[`features`]。
const MIXED_CELLS: [Cell; 4] = [
    Cell {
        title: "Realtime Sync",
        description: "変更内容をチーム全員の画面へ即座に反映します。",
        width: CellWidth::Wide,
    },
    Cell {
        title: "Access Control",
        description: "役割ごとに閲覧・編集の権限を分けられます。",
        width: CellWidth::Narrow,
    },
    Cell {
        title: "Usage Insights",
        description: "利用状況を簡単なグラフで把握できます。",
        width: CellWidth::Half,
    },
    Cell {
        title: "Integration Hub",
        description: "外部サービスとの連携をひとつの画面で管理します。",
        width: CellWidth::Half,
    },
];

/// feature 一覧の 1 項目（アイコン + タイトル + 説明文）。
struct Feature {
    title: &'static str,
    description: &'static str,
}

/// 混在形（R0108）下段の 4 列 feature 一覧データ。
const FEATURES: [Feature; 4] = [
    Feature {
        title: "Fast Setup",
        description: "数分でチームの利用を開始できます。",
    },
    Feature {
        title: "Reliable Uptime",
        description: "安定した稼働を継続的に監視しています。",
    },
    Feature {
        title: "Flexible Plans",
        description: "チーム規模に合わせて契約内容を選べます。",
    },
    Feature {
        title: "Responsive Support",
        description: "問い合わせには迅速に対応します。",
    },
];

/// feature 一覧用の自作の抽象チェックマーク図形（装飾用途のため
/// `aria-hidden`）。参照元のアイコン形状・内部識別子は持ち込まない。
fn feature_icon() -> Node {
    icon(
        &IconProps {
            size: Size::Sm,
            label: None,
            ..IconProps::default()
        },
        vec![],
        vec![
            el(
                "circle",
                vec![
                    ("cx", "12"),
                    ("cy", "12"),
                    ("r", "9"),
                    ("fill", "none"),
                    ("stroke", "currentColor"),
                    ("stroke-width", "1.5"),
                ],
                vec![],
            ),
            el(
                "path",
                vec![
                    ("d", "M8 12.5l2.5 2.5L16 9"),
                    ("stroke", "currentColor"),
                    ("stroke-width", "1.5"),
                    ("stroke-linecap", "round"),
                    ("stroke-linejoin", "round"),
                    ("fill", "none"),
                ],
                vec![],
            ),
        ],
    )
}

/// 見出しエリア（eyebrow badge + `<h3>` + リード文）。
fn header() -> Node {
    div(
        vec![("class", "blocks-bento-asymmetric-rows-header")],
        vec![
            badge::badge(
                &BadgeProps::default(),
                vec![("data-blocks-bento-asymmetric-rows-eyebrow", "")],
                vec![text("Platform")],
            ),
            heading::heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    ..HeadingProps::default()
                },
                vec![("data-blocks-bento-asymmetric-rows-title", "")],
                vec![text("チームの仕事をひとつの流れにまとめる")],
            ),
            styled_text::text(
                &TextProps {
                    size: TextSize::Md,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![("data-blocks-bento-asymmetric-rows-lead", "")],
                vec![text(
                    "分断されがちな作業を、幅の異なるカードで用途ごとに見渡せるようにしました。",
                )],
            ),
        ],
    )
}

/// 1 行のキャプション（`h2`/`h3` は使わない。右目次・折りたたみ目次が
/// 拾ってしまうため、`banner_full_width_bar`/`footer_newsletter` と同じ
/// 判断で素の `p` を使う）。
fn caption(label: &str) -> Node {
    p(
        vec![("data-blocks-bento-asymmetric-rows-caption", "")],
        vec![text(label)],
    )
}

/// 1 枚分の bento セル（`card`。画像を上、見出しと説明を下に置く）。
/// `variant` はカードの見た目（ジグザグ形のみ [`CardVariant::Subtle`]）。
fn cell(item: &Cell, variant: CardVariant) -> Node {
    card::root(
        CardProps {
            variant,
            ..CardProps::default()
        },
        vec![("data-blocks-bento-asymmetric-rows-cell", item.width.value())],
        vec![
            card::cover(
                vec![("class", "blocks-bento-asymmetric-rows-cover")],
                vec![image::image(
                    &ImageProps {
                        aspect_ratio: AspectRatio::Landscape,
                        // 全セルが同一のプレースホルダー画像を使い、内容を
                        // 伝えない同一文言の alt を持たせると支援技術で
                        // 同じ文言が繰り返し読み上げられる（WCAG 1.1.1
                        // 違反、Bugbot 指摘 #3162）。装飾用途として空の
                        // alt にする（`blog_list_image` /
                        // `bento_three_column_tall` / `login_04` の
                        // 同型プレースホルダー画像と同じ判断）。
                        ..ImageProps::new(dummy_assets::SCREENSHOT_SRC, "")
                    },
                    vec![],
                )],
            ),
            card::body(
                vec![("class", "blocks-bento-asymmetric-rows-body")],
                vec![
                    heading::heading(
                        HeadingLevel::H4,
                        &HeadingProps::default(),
                        vec![],
                        vec![text(item.title)],
                    ),
                    card::description(vec![], vec![text(item.description)]),
                ],
            ),
        ],
    )
}

/// 1 個の bento グリッド（`columns` に応じて `wide`/`narrow`/`half` の
/// 意味づけが変わる、モジュール doc「列モードと `wide`/`narrow`/`half`
/// の意味づけ」節参照）。`variant` は全セル共通のカード見た目。
fn grid(columns: GridColumns, variant: CardVariant, cells: &[Cell]) -> Node {
    let rendered: Vec<Node> = cells.iter().map(|item| cell(item, variant)).collect();
    div(
        vec![
            ("class", "blocks-bento-asymmetric-rows-grid"),
            ("data-blocks-bento-asymmetric-rows-columns", columns.value()),
        ],
        rendered,
    )
}

/// 混在形（4 番目のバリエーション）下段の 4 列 feature 一覧。`card` は
/// 使わず、アイコン + 太字タイトル + 説明文を縦に並べる軽い一覧にする
/// （モジュール doc「feature 一覧と `icon` の扱い」節参照）。
fn features() -> Node {
    let items: Vec<Node> = FEATURES
        .iter()
        .map(|item| {
            div(
                vec![("data-blocks-bento-asymmetric-rows-feature", "")],
                vec![
                    div(
                        vec![("class", "blocks-bento-asymmetric-rows-feature-title")],
                        vec![
                            feature_icon(),
                            styled_text::text(
                                &TextProps {
                                    size: TextSize::Sm,
                                    weight: TextWeight::Semibold,
                                    ..TextProps::default()
                                },
                                vec![],
                                vec![text(item.title)],
                            ),
                        ],
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
            )
        })
        .collect();
    div(
        vec![("class", "blocks-bento-asymmetric-rows-features")],
        items,
    )
}

/// `bento-asymmetric-rows` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。見出しエリアの下に、キャプション付きで 4 つのバリエー
/// ション（モジュール doc「4 バリエーションと対応表 ID の対応」節）を
/// 縦に並べる。ルートの class は [`BLOCK::demo_class`]（外側の
/// `.blocks-demo` ラッパへ付与される）と意図的に別名にする。同名だと
/// ラッパ div にも [`LAYOUT_CSS`] の `display: flex` 規則が二重適用
/// されてしまうため（`banner_full_width_bar`/`footer_newsletter` の
/// `-stack` 命名と同じ判断）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-bento-asymmetric-rows-stack")],
        vec![
            header(),
            caption("6 columns · 4+2 / 2+4"),
            grid(GridColumns::Six, CardVariant::Outline, &BASE_CELLS),
            caption("6 columns · 3+3 / 2+2+2"),
            grid(GridColumns::Six, CardVariant::Outline, &SPLIT_CELLS),
            caption("3 columns · 2+1 / 1+2 zigzag · subtle cards"),
            grid(GridColumns::Three, CardVariant::Subtle, &ZIGZAG_CELLS),
            caption("6 columns · 4/2/3/3 + 4-column feature list"),
            grid(GridColumns::Six, CardVariant::Outline, &MIXED_CELLS),
            features(),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/bento-asymmetric-rows/",
    title: "bento-asymmetric-rows",
    category: BlockCategory::Bento,
    rust_source: "crates/docs-site/src/blocks/marketing/bento/bento_asymmetric_rows.rs",
    demo_class: "blocks-bento-asymmetric-rows",
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
            label: "Card",
            path: "/themes/card/",
        },
        Part {
            label: "Image",
            path: "/themes/image/",
        },
        Part {
            label: "Icon",
            path: "/themes/icon/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `bento_asymmetric_rows` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節。他 block と同型で `super::stylesheet`
/// から連結される）。48rem/64rem の根拠はモジュール doc「レスポンシブを
/// モバイルファーストで書く理由」節参照（本ファイル末尾の `#[cfg(test)]`
/// が `Breakpoint::Md`/`Lg` とのドリフトを検知する）。
///
/// 3 列モードの `wide`/`narrow` 上書き（`.blocks-bento-asymmetric-rows-
/// grid[data-blocks-bento-asymmetric-rows-columns="three"]
/// [data-blocks-bento-asymmetric-rows-cell="wide"]`、詳細度 (0,3,0)）は
/// 6 列既定の `[data-blocks-bento-asymmetric-rows-cell="wide"]`（詳細度
/// (0,1,0)）より必ず優先される。`card` recipe base は `grid-column` を
/// 宣言しないため、この 2 段構成以外に競合する規則は無い。
const LAYOUT_CSS: &str = "\
.blocks-bento-asymmetric-rows-stack {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n}\n\
.blocks-bento-asymmetric-rows-header {\n  display: flex;\n  flex-direction: column;\n  align-items: flex-start;\n  gap: var(--fandhe-space-2);\n}\n\
[data-blocks-bento-asymmetric-rows-caption] {\n  margin: 0;\n  font-size: var(--fandhe-font-font-size-sm, 0.875rem);\n  font-weight: var(--fandhe-font-font-weight-medium, 500);\n  color: var(--fandhe-color-fg-muted);\n}\n\
.blocks-bento-asymmetric-rows-grid {\n  display: grid;\n  grid-template-columns: 1fr;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-bento-asymmetric-rows-cover img {\n  width: 100%;\n}\n\
.blocks-bento-asymmetric-rows-body {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1-5);\n}\n\
.blocks-bento-asymmetric-rows-features {\n  display: grid;\n  grid-template-columns: 1fr;\n  gap: var(--fandhe-space-4);\n}\n\
[data-blocks-bento-asymmetric-rows-feature] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1);\n}\n\
.blocks-bento-asymmetric-rows-feature-title {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n}\n\
@media (min-width: 48rem) {\n  .blocks-bento-asymmetric-rows-grid {\n    grid-template-columns: repeat(2, minmax(0, 1fr));\n  }\n  [data-blocks-bento-asymmetric-rows-cell] {\n    grid-column: span 1;\n  }\n  .blocks-bento-asymmetric-rows-features {\n    grid-template-columns: repeat(2, minmax(0, 1fr));\n  }\n}\n\
@media (min-width: 64rem) {\n  .blocks-bento-asymmetric-rows-grid {\n    grid-template-columns: repeat(6, minmax(0, 1fr));\n  }\n  [data-blocks-bento-asymmetric-rows-cell=\"wide\"] {\n    grid-column: span 4;\n  }\n  [data-blocks-bento-asymmetric-rows-cell=\"narrow\"] {\n    grid-column: span 2;\n  }\n  [data-blocks-bento-asymmetric-rows-cell=\"half\"] {\n    grid-column: span 3;\n  }\n  .blocks-bento-asymmetric-rows-grid[data-blocks-bento-asymmetric-rows-columns=\"three\"] {\n    grid-template-columns: repeat(3, minmax(0, 1fr));\n  }\n  .blocks-bento-asymmetric-rows-grid[data-blocks-bento-asymmetric-rows-columns=\"three\"] [data-blocks-bento-asymmetric-rows-cell=\"wide\"] {\n    grid-column: span 2;\n  }\n  .blocks-bento-asymmetric-rows-grid[data-blocks-bento-asymmetric-rows-columns=\"three\"] [data-blocks-bento-asymmetric-rows-cell=\"narrow\"] {\n    grid-column: span 1;\n  }\n  .blocks-bento-asymmetric-rows-features {\n    grid-template-columns: repeat(4, minmax(0, 1fr));\n  }\n}\n";

#[cfg(test)]
mod tests {
    use super::{
        demo, CellWidth, BASE_CELLS, FEATURES, LAYOUT_CSS, MIXED_CELLS, SPLIT_CELLS, ZIGZAG_CELLS,
    };
    use fandhe_frontend_core::render;
    use fandhe_frontend_pre_styled_ui::recipe::Breakpoint;

    /// [`LAYOUT_CSS`] の 48rem/64rem が
    /// `fandhe_frontend_pre_styled_ui::recipe::Breakpoint::Md`/`Lg` の
    /// `min_width()`（768px/1024px、16px 基準で 48rem/64rem）と実際に
    /// 一致し、6 列（span 4/2/3）・3 列（`repeat(3, ...)`・span 2/1）の
    /// 両方の規則を含むこと（モジュール doc「レスポンシブをモバイル
    /// ファーストで書く理由」節が参照する対応のドリフト検知）。
    #[test]
    fn layout_css_declares_both_breakpoints_and_spans() {
        assert_eq!(Breakpoint::Md.min_width(), "768px");
        assert_eq!(Breakpoint::Lg.min_width(), "1024px");
        // 768px / 16 = 48rem, 1024px / 16 = 64rem（`Breakpoint::min_width`
        // の rustdoc・`docs/design/pre-styled-ui-scale-tokens.md` §3.6 と
        // 同じ px→rem 換算）。
        assert!(LAYOUT_CSS.contains("@media (min-width: 48rem)"));
        assert!(LAYOUT_CSS.contains("@media (min-width: 64rem)"));
        assert!(LAYOUT_CSS.contains("grid-column: span 4"));
        assert!(LAYOUT_CSS.contains("grid-column: span 2"));
        assert!(LAYOUT_CSS.contains("grid-column: span 3"));
        assert!(LAYOUT_CSS.contains("grid-column: span 1"));
        assert!(LAYOUT_CSS.contains("repeat(3, minmax(0, 1fr))"));
        assert!(LAYOUT_CSS.contains("repeat(4, minmax(0, 1fr))"));
        assert!(LAYOUT_CSS.contains(
            "[data-blocks-bento-asymmetric-rows-columns=\"three\"] [data-blocks-bento-asymmetric-rows-cell=\"wide\"]"
        ));
    }

    /// 4 バリエーションそれぞれの幅区分の並びが、モジュール doc「4
    /// バリエーションと対応表 ID の対応」節が定める配置と一致すること。
    #[test]
    fn variant_cells_declare_expected_width_sequences() {
        let widths = |cells: &[super::Cell]| -> Vec<&str> {
            cells.iter().map(|c| c.width.value()).collect()
        };
        assert_eq!(
            widths(&BASE_CELLS),
            vec!["wide", "narrow", "narrow", "wide"]
        );
        assert_eq!(
            widths(&SPLIT_CELLS),
            vec!["half", "half", "narrow", "narrow", "narrow"]
        );
        assert_eq!(
            widths(&ZIGZAG_CELLS),
            vec!["wide", "narrow", "narrow", "wide"]
        );
        assert_eq!(widths(&MIXED_CELLS), vec!["wide", "narrow", "half", "half"]);
        assert!(matches!(BASE_CELLS[0].width, CellWidth::Wide));
    }

    /// [`demo`] の出力が、4 バリエーション合計 17 セル（4+5+4+4、#3145 の
    /// 「件数はハードコードせず `const` 配列の長さから導出する」方針を
    /// テスト側でも踏襲）・feature 4 件・`fd-card--variant-subtle` はジグ
    /// ザグ形の 4 件だけ・3 列モードはちょうど 1 件・`<form>`/`<script>`/
    /// `data:` URI を含まないことを固定する。
    #[test]
    fn demo_renders_expected_variant_totals() {
        let total_cells =
            BASE_CELLS.len() + SPLIT_CELLS.len() + ZIGZAG_CELLS.len() + MIXED_CELLS.len();
        assert_eq!(total_cells, 17);

        let html = render(&demo());
        assert_eq!(
            html.matches("data-blocks-bento-asymmetric-rows-cell=\"")
                .count(),
            total_cells
        );
        assert_eq!(
            html.matches("data-blocks-bento-asymmetric-rows-columns=\"three\"")
                .count(),
            1
        );
        assert_eq!(
            html.matches("fd-card--variant-subtle").count(),
            ZIGZAG_CELLS.len()
        );
        assert_eq!(
            html.matches("data-blocks-bento-asymmetric-rows-caption=\"\"")
                .count(),
            4
        );
        assert_eq!(
            html.matches("data-blocks-bento-asymmetric-rows-feature=\"\"")
                .count(),
            FEATURES.len()
        );
        assert_eq!(html.matches("data-scope=\"icon\"").count(), FEATURES.len());
        assert_eq!(html.matches("aria-hidden=\"true\"").count(), FEATURES.len());
        assert!(!html.contains(r#"role="img""#));
        assert!(!html.contains("<form"));
        assert!(!html.contains("<script"));
        assert!(!html.contains("src=\"data:"));
    }

    /// `demo()` は決定的（2 回の `render` が一致する）。
    #[test]
    fn demo_is_deterministic() {
        assert_eq!(render(&demo()), render(&demo()));
    }
}
