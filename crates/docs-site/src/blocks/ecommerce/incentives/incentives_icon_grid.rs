//! `incentives-icon-grid` block（イシュー #3050。Ecommerce 区分で最初の
//! block であり、本 PR で `ecommerce/incentives` カテゴリを雛形から卒業
//! させた）。アイコン・イラスト付きの特典項目をグリッドで並べる、
//! ストア共通の「特典紹介」セクションの合成例（取得手段・ファイル名・
//! 内部コンポーネント識別子は記載しない。対応表 ID は原稿「原案差分
//! メモ」節にのみ記す）。
//!
//! # 使用部品
//!
//! `heading` / `text` / `icon` / `image` / `card` / `item` /
//! `visually_hidden` の 7 部品のみを合成する（[`BLOCK`] の `parts` に
//! 一致させる契約、`crates/docs-site/tests/blocks_nav.rs`/
//! `blocks_contract.rs` が検証する）。新規 UI 部品・外部依存は追加しない。
//!
//! # 5 インスタンスへの統合
//!
//! 集約元 11 件の差分を 5 インスタンスへ統合し、Demo に並べて示す
//! （詳細は原稿側「原案差分メモ」節）。
//!
//! - **A**: 見出し + 導入文 + 枠付きカード（`CardVariant::Outline`）3 件。
//!   各カードはアイコン・項目題名・説明を縦に積む基準形
//! - **B**: 見出しを視覚的に隠し（[`fandhe_frontend_pre_styled_ui::
//!   visually_hidden`]）、枠なしで中央寄せ・淡色の角丸枠で囲んだアイコンの
//!   4 件を並べる
//! - **C**: 中央寄せ見出し + 中央寄せ（アイコン上・テキスト中央）3 件
//! - **D**: アイコン左・文章右の横並び 3 件（`item::root` +
//!   `item::media(ItemMediaVariant::Icon)` + `item::content`）
//! - **E**: 淡色パネル（`--fandhe-color-bg-muted`）内に中央見出し +
//!   装飾イラスト 4 点（[`fandhe_frontend_pre_styled_ui::image`]、
//!   `alt=""` の装飾画像）
//!
//! パネル（E）・アイコン枠（B）の背景は `--fandhe-color-bg-subtle` では
//! なく `--fandhe-color-bg-muted` を使う。Blocks デモ枠自身
//! （`.blocks-demo`）が `--fandhe-color-bg-subtle` を背景に持つため、
//! 同トークンのままだとパネル・アイコン枠がデモ背景に埋もれて境界が
//! 見分けられない（イシュー #3050 PR #3505 レビュー指摘）。`bg-muted` は
//! `bg-subtle` より 1 段濃い定義済みトークン（`theme.rs` の
//! `DEFAULT_COLORS`）であり、デモ背景との対比を確保する。
//!
//! # 見出しレベル
//!
//! ページ側が `## Demo` として `h2` を出すため、各インスタンスの導入
//! 見出しは `HeadingLevel::H3`、項目題名はそれより 1 段下げて
//! `HeadingLevel::H4` にする（D の項目題名は `item::title`（`div`）の
//! ままとする。headless `item` anatomy の固定仕様であり本 block 側の
//! 選択ではない）。
//!
//! # 見出しを視覚的に隠す配置（インスタンス B）
//!
//! [`fandhe_frontend_pre_styled_ui::visually_hidden::root`] は `span` を
//! 出力するため、`h3` を `span` で包むと見出し要素の内容モデルに反する。
//! 本 block では逆に `h3` の**中へ** `visually_hidden::root` を入れる
//! （`heading::heading(H3, …, vec![text("…")])` ではなく
//! `heading::heading(H3, …, vec![visually_hidden::root(vec![], vec![text("…")])])`）。
//! これにより見出し要素自体はアクセシビリティツリーに残りつつ、視覚的には
//! 1px 四方へ縮小される。`h3` 自体が生む余白は `margin: 0` の CSS で潰す。
//!
//! `h3` を包む `.blocks-incentives-icon-grid-header` 自体は、中身が
//! 1px に縮んでも親（`.blocks-incentives-icon-grid-instance`、
//! `display: flex` + `gap`）の flex item であり続けるため、`gap` 分の
//! 余白がそのまま残ってしまう（イシュー #3050 PR #3505 レビュー指摘）。
//! sr-only のときはラッパー自身にも `data-sr-only` を付けて
//! `position: absolute` にし、flex レイアウトから完全に外す
//! （見た目は元から 1px なので位置の指定自体は無関係、flex item
//! として数えられなくなることが目的）。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `heading::heading`/`styled_text::text`/`icon::icon`/`card::root`/
//! `card::body`/`item::*`/`visually_hidden::root`/`image::image` はいずれも
//! `drop_class_attr` により呼び出し側 `attrs` の `class` を黙って除去する
//! 契約を持つため、Demo 固有のスタイルフックは
//! `data-blocks-incentives-icon-grid-*` 属性で渡す（`feature_three_column_icons`
//! と同型の判断）。素の `div` には `class` がそのまま効くため
//! `.blocks-incentives-icon-grid-*` クラスセレクタを使う。レイアウト root の
//! class（`blocks-incentives-icon-grid-layout`）は [`Block::demo_class`]
//! （`blocks-incentives-icon-grid`）とは意図的に別名にする（先行 block で
//! 得た Bugbot 教訓の踏襲）。
//!
//! # アイコン・画像を装飾扱いにする理由
//!
//! 項目題名・説明が既に意味を伝えているため、アイコンは
//! [`IconProps::label`] を `None` にする（`feature_three_column_icons::
//! geo_icon` と同型の判断）。自作の単純な幾何パスのみを使い、lucide 等の
//! 著作物は複製しない。E のイラストは [`dummy_assets::PRODUCT_SRC`] を
//! `alt=""` で渡す装飾画像（`category_grid_overlay` 等と同型）。
//!
//! # `<form>`・リンク・`id` を持たない理由
//!
//! 特典紹介は静的な説明セクションであり、送信処理・個別の詳細ページを
//! 持たないため `<form>`・`<a href>`・`id` 属性のいずれも出力しない
//! （`crate::blocks` モジュール doc「`<form>` を使わない」節に加え、
//! 死リンク・ARIA 参照の重複を生まない最小構成にするための判断）。
//!
//! # セキュリティ不変条件・依存追加なし
//!
//! `crate::blocks` モジュール doc「セキュリティ不変条件」節に従い、本
//! Demo はフォーム・状態機械を持たない静的な合成例である。文言はすべて
//! 架空のもの（実企業名・実クレデンシャル・PII を含まない）。新規 UI 部品・
//! 新規外部クレート依存は追加しない。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps, CardVariant};
use fandhe_frontend_pre_styled_ui::heading::{
    self, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::image::{image, ImageProps};
use fandhe_frontend_pre_styled_ui::item::{self, ItemMediaVariant, ItemRootProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::visually_hidden;
use fandhe_frontend_pre_styled_ui::Size;

/// 特典 1 件分の架空データ（題名・説明・自作アイコンのパス）。
struct Perk {
    title: &'static str,
    body: &'static str,
    icon_path_d: &'static str,
}

/// 特典 4 件（架空、実在の企業・製品とは無関係）。自作の単純な幾何パス
/// のみを使い、lucide 等の著作物は複製しない。先頭 3 件を A/C/D インス
/// タンスで使い回す。
const PERKS: [Perk; 4] = [
    Perk {
        title: "送料無料",
        body: "一定金額以上のご注文で、配送料は一切かかりません。",
        icon_path_d: "M3 7h11v9H3zM14 10h4l3 3v3h-7z\
            M7 20a2 2 0 100-4 2 2 0 000 4zM17 20a2 2 0 100-4 2 2 0 000 4z",
    },
    Perk {
        title: "30 日間返品保証",
        body: "ご注文から 30 日以内であれば理由を問わず返品できます。",
        icon_path_d: "M4 4v6h6M4 10a8 8 0 1114.93 4.93",
    },
    Perk {
        title: "ギフト包装",
        body: "ご注文時にチェックするだけで、包装紙とメッセージカードを添えます。",
        icon_path_d: "M3 10h18v10H3zM3 10V6h18v4M12 2l3 4H9z\
            M12 10v10",
    },
    Perk {
        title: "ポイント還元",
        body: "ご購入金額に応じてポイントが貯まり、次回以降の買い物に使えます。",
        icon_path_d: "M12 2l2.6 5.3 5.9.9-4.3 4.1 1 5.8L12 15.9 6.8 18l1-5.8-4.3-4.1 5.9-.9z",
    },
];

/// 装飾用の自作幾何アイコン（`fill="none"` + `stroke="currentColor"` の
/// 線画、`feature_three_column_icons::geo_icon` と同型の判断）。
fn geo_icon(path_d: &'static str, size: Size, attrs: Vec<(&'static str, &'static str)>) -> Node {
    icon(
        &IconProps {
            size,
            ..IconProps::default()
        },
        attrs,
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

/// 中央寄せの導入見出し（A/C/E 用）。`sr_only` が `true` のとき見出し文言を
/// [`visually_hidden::root`] で包み、アクセシビリティツリーにのみ残す
/// （モジュール doc「見出しを視覚的に隠す配置」節参照）。
fn header(title: &'static str, lead: Option<&'static str>, sr_only: bool) -> Node {
    let heading_children = if sr_only {
        vec![visually_hidden::root(vec![], vec![text(title)])]
    } else {
        vec![text(title)]
    };
    let mut children = vec![heading::heading(
        HeadingLevel::H3,
        &HeadingProps {
            size: HeadingSize::Xl2,
            weight: HeadingWeight::Bold,
        },
        if sr_only {
            vec![("data-blocks-incentives-icon-grid-sr-heading", "")]
        } else {
            vec![]
        },
        heading_children,
    )];
    if let Some(lead) = lead {
        children.push(styled_text::text(
            &TextProps {
                variant: TextVariant::Muted,
                ..TextProps::default()
            },
            vec![("data-blocks-incentives-icon-grid-lead", "")],
            vec![text(lead)],
        ));
    }
    let mut wrapper_attrs = vec![("class", "blocks-incentives-icon-grid-header")];
    if sr_only {
        wrapper_attrs.push(("data-sr-only", ""));
    }
    div(wrapper_attrs, children)
}

/// 項目題名 + 説明（アイコン・カードの有無は呼び出し側が組み立てる）。
fn perk_text(p: &Perk) -> Vec<Node> {
    vec![
        heading::heading(
            HeadingLevel::H4,
            &HeadingProps::default(),
            vec![],
            vec![text(p.title)],
        ),
        styled_text::text(
            &TextProps {
                size: TextSize::Sm,
                variant: TextVariant::Muted,
                ..TextProps::default()
            },
            vec![("data-blocks-incentives-icon-grid-desc", "")],
            vec![text(p.body)],
        ),
    ]
}

/// A 用: 枠付きカードへ収めたアイコン付き項目。
fn perk_card(p: &Perk) -> Node {
    let mut body_children = vec![geo_icon(
        p.icon_path_d,
        Size::Lg,
        vec![("data-blocks-incentives-icon-grid-icon", "")],
    )];
    body_children.extend(perk_text(p));
    card::root(
        CardProps {
            variant: CardVariant::Outline,
            ..CardProps::default()
        },
        vec![("data-blocks-incentives-icon-grid-card", "")],
        vec![card::body(
            vec![("data-blocks-incentives-icon-grid-card-body", "")],
            body_children,
        )],
    )
}

/// B 用: 淡色の角丸枠で囲んだアイコン + 題名 + 説明（カードなし）。
fn perk_badge_icon(p: &Perk) -> Node {
    let mut children = vec![div(
        vec![("class", "blocks-incentives-icon-grid-badge")],
        vec![geo_icon(
            p.icon_path_d,
            Size::Md,
            vec![("data-blocks-incentives-icon-grid-icon", "")],
        )],
    )];
    children.extend(perk_text(p));
    div(
        vec![
            ("class", "blocks-incentives-icon-grid-item"),
            ("data-align", "center"),
        ],
        children,
    )
}

/// C 用: 中央寄せ（アイコン上・テキスト中央、カードなし・バッジなし）。
fn perk_centered(p: &Perk) -> Node {
    let mut children = vec![geo_icon(
        p.icon_path_d,
        Size::Lg,
        vec![("data-blocks-incentives-icon-grid-icon", "")],
    )];
    children.extend(perk_text(p));
    div(
        vec![
            ("class", "blocks-incentives-icon-grid-item"),
            ("data-align", "center"),
        ],
        children,
    )
}

/// D 用: アイコン左・文章右の横並び 1 行（`item` anatomy）。
fn perk_item_row(p: &Perk) -> Node {
    item::root(
        ItemRootProps::default(),
        vec![("data-blocks-incentives-icon-grid-item-row", "")],
        vec![
            item::media(
                ItemMediaVariant::Icon,
                vec![],
                vec![geo_icon(p.icon_path_d, Size::Md, vec![])],
            ),
            item::content(
                vec![],
                vec![
                    item::title(vec![], vec![text(p.title)]),
                    item::description(vec![], vec![text(p.body)]),
                ],
            ),
        ],
    )
}

/// E 用: 淡色パネル内の装飾イラスト 1 枚 + 項目題名（アイコン・説明は
/// 持たず、パネルが特典紹介であることを見出しと題名のみで示す簡潔形）。
fn perk_illustration(p: &Perk) -> Node {
    div(
        vec![
            ("class", "blocks-incentives-icon-grid-item"),
            ("data-align", "center"),
        ],
        vec![
            image(
                &ImageProps::new(dummy_assets::PRODUCT_SRC, ""),
                vec![("data-blocks-incentives-icon-grid-illustration", "")],
            ),
            heading::heading(
                HeadingLevel::H4,
                &HeadingProps::default(),
                vec![],
                vec![text(p.title)],
            ),
        ],
    )
}

/// 注記行（各インスタンスの原案差分の要約、`data-*` フックのみで class を
/// 持たない共通パーツ）。
fn note(body: &'static str) -> Node {
    styled_text::text(
        &TextProps {
            size: TextSize::Xs,
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![("data-blocks-incentives-icon-grid-note", "")],
        vec![text(body)],
    )
}

/// 1 インスタンス分（任意の見出し + 注記 + グリッド）を組み立てる。
fn instance(
    header_node: Option<Node>,
    note_body: &'static str,
    grid_items: Vec<Node>,
    extra_attrs: Vec<(&'static str, &'static str)>,
) -> Node {
    let mut attrs = vec![("class", "blocks-incentives-icon-grid-instance")];
    attrs.extend(extra_attrs);
    let mut children = Vec::new();
    if let Some(header_node) = header_node {
        children.push(header_node);
    }
    children.push(note(note_body));
    children.push(div(
        vec![("class", "blocks-incentives-icon-grid-grid")],
        grid_items,
    ));
    div(attrs, children)
}

/// `incentives-icon-grid` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数（他 block と同じ状態を持たない設計）。
#[must_use]
pub fn demo() -> Node {
    let instance_a = instance(
        Some(header(
            "ご注文いただくみなさまへの特典",
            Some("当店でのお買い物にいつも付いてくる、3 つの特典です。"),
            false,
        )),
        "基準形。見出し + 導入文 + 枠付きカード（Outline）3 件。",
        PERKS[..3].iter().map(perk_card).collect(),
        vec![],
    );

    let instance_b = instance(
        Some(header("特典一覧（視覚的に非表示）", None, true)),
        "見出しを視覚的に隠し、淡色の角丸枠で囲んだアイコン付き項目を 4 件並べる。カードなし。",
        PERKS.iter().map(perk_badge_icon).collect(),
        vec![("data-count", "4")],
    );

    let instance_c = instance(
        Some(header(
            "シンプルに伝える特典",
            Some("アイコンを中央に置き、テキストも中央寄せにした簡潔な配置です。"),
            false,
        )),
        "中央見出し。アイコン上・テキスト中央の 3 件（カード・バッジなし）。",
        PERKS[..3].iter().map(perk_centered).collect(),
        vec![],
    );

    let instance_d = instance(
        None,
        "見出しなし。アイコン左・文章右の横並び 3 件（item anatomy）。",
        PERKS[..3].iter().map(perk_item_row).collect(),
        vec![("data-layout", "rows")],
    );

    let instance_e = instance(
        Some(header(
            "届いてからも楽しい",
            Some("特典の様子を、イラストでも紹介します。"),
            false,
        )),
        "淡色パネル内に中央見出し + 装飾イラスト 4 点（alt は空、装飾画像）。",
        PERKS.iter().map(perk_illustration).collect(),
        vec![("data-panel", "subtle"), ("data-count", "4")],
    );

    div(
        vec![("class", "blocks-incentives-icon-grid-layout")],
        vec![instance_a, instance_b, instance_c, instance_d, instance_e],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/incentives-icon-grid/",
    title: "incentives-icon-grid",
    category: BlockCategory::Incentives,
    rust_source: "crates/docs-site/src/blocks/ecommerce/incentives/incentives_icon_grid.rs",
    demo_class: "blocks-incentives-icon-grid",
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
            label: "Icon",
            path: "/themes/icon/",
        },
        Part {
            label: "Image",
            path: "/themes/image/",
        },
        Part {
            label: "Card",
            path: "/themes/card/",
        },
        Part {
            label: "Item",
            path: "/themes/item/",
        },
        Part {
            label: "Visually Hidden",
            path: "/themes/visually-hidden/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `incentives_icon_grid` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節）。セレクタは
/// `.blocks-incentives-icon-grid-*` と
/// `[data-blocks-incentives-icon-grid-*]` のみを用い、他 block や部品の
/// 素のセレクタへ影響させない（`feature_three_column_icons` と同じ名前
/// 空間分離）。グリッドは `repeat(auto-fit, minmax(min(100%, 12rem), 1fr))`
/// で組み、デモ枠の幅に応じて自動で列数が変わる（`@media` 不要でも狭い
/// 幅では 1〜2 列へ落ちる）。4 件グリッド（B/E）が 2 列止まりで収まり
/// 切らないぶんは 4 列へ揃えたいが、Blocks デモ枠は `.docs-content` の
/// 最大幅（約 46rem）のカラム内にあり、ビューポート幅での `@media` では
/// デモの実表示幅を反映できない（画面幅が広くても列自体は狭いまま
/// 4 列固定になり `minmax(min(100%, 12rem), …)` の折り返しが失われる）。
/// そのため `.blocks-incentives-icon-grid-instance` へ
/// `container-type: inline-size` を張り、実コンテナ幅で判定する
/// `@container` へ置き換える（`product_overview_image_grid` と同型）。
/// クエリ対象は `.blocks-incentives-icon-grid-instance` 自身ではなく、
/// その**子**（`.blocks-incentives-icon-grid-grid`）である点に注意
/// （コンテナクエリは対象要素自身が確立するコンテナを参照できない仕様
/// であり、祖先セレクタに `container-type` を張った要素自身ではなく
/// descendant を対象にする必要がある）。
///
/// しきい値は `@container` が実際に参照する**コンテナ自身（`.instance`）
/// の content-box 幅**で計算する（ビューポート幅や `.docs-content` の
/// `max-width` そのものではない）。`.instance` の祖先は
/// `.docs-content`（`max-width: 46rem`） → `.blocks-demo`（`padding:
/// 1.5rem` 両側 = 3rem） → `.instance` 自身（`padding:
/// var(--fandhe-space-6)` = 1.5rem 両側 = 3rem）の 2 段の padding を
/// 挟むため、`.instance` が到達し得る content-box 幅の上限は
/// `46rem − 3rem − 3rem = 40rem` である。旧しきい値 52.5rem
/// （`4 * 12rem + 3 * 1.5rem`、列幅 12rem 基準）はこの 40rem 上限を
/// 超えており、デモ枠内では構造的に到達不能だった（4 列表示が常に
/// 未発動のまま死んでいたバグ、イシュー #3050 PR #3505 レビュー指摘）。
/// 到達可能な列幅へ縮小し、しきい値 36rem（`4 * 7.875rem + 3 *
/// 1.5rem`）に変更する。40rem 上限に対し十分な余裕（4rem）を残すことで
/// ブラウザごとの端数処理・スクロールバー幅のぶれでも確実に到達する。
/// 対象は 4 件インスタンス（B/E）のみに限定するため `data-count="4"` を
/// 付与し、3 件インスタンス（A/C）には付けない（3 件のまま
/// `repeat(4, …)` を当てると 4 列目が空トラックのまま残り、3 枚の
/// 中央寄せが崩れるため）。
const LAYOUT_CSS: &str = "\
.blocks-incentives-icon-grid-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-10);\n}\n\
.blocks-incentives-icon-grid-instance {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n  padding: var(--fandhe-space-6);\n  border-radius: var(--fandhe-radius-lg);\n  container-type: inline-size;\n  container-name: blocks-incentives-icon-grid;\n}\n\
.blocks-incentives-icon-grid-instance[data-panel=\"subtle\"] {\n  background: var(--fandhe-color-bg-muted);\n}\n\
.blocks-incentives-icon-grid-header {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n  align-items: center;\n  text-align: center;\n  max-width: 40rem;\n  margin-inline: auto;\n}\n\
.blocks-incentives-icon-grid-header[data-sr-only] {\n  position: absolute;\n}\n\
h3[data-blocks-incentives-icon-grid-sr-heading] {\n  margin: 0;\n}\n\
[data-scope=\"text\"][data-part=\"root\"][data-blocks-incentives-icon-grid-lead] {\n  margin: 0;\n}\n\
[data-scope=\"text\"][data-part=\"root\"][data-blocks-incentives-icon-grid-note] {\n  margin: 0;\n  text-align: center;\n}\n\
.blocks-incentives-icon-grid-grid {\n  display: grid;\n  grid-template-columns: repeat(auto-fit, minmax(min(100%, 12rem), 1fr));\n  gap: var(--fandhe-space-6);\n}\n\
.blocks-incentives-icon-grid-instance[data-layout=\"rows\"] .blocks-incentives-icon-grid-grid {\n  grid-template-columns: minmax(0, 1fr);\n}\n\
.blocks-incentives-icon-grid-item {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n  min-width: 0;\n}\n\
.blocks-incentives-icon-grid-item[data-align=\"center\"] {\n  align-items: center;\n  text-align: center;\n}\n\
.blocks-incentives-icon-grid-badge {\n  display: inline-flex;\n  align-items: center;\n  justify-content: center;\n  padding: var(--fandhe-space-3);\n  border-radius: var(--fandhe-radius-full);\n  background: var(--fandhe-color-bg-muted);\n}\n\
[data-scope=\"text\"][data-part=\"root\"][data-blocks-incentives-icon-grid-desc] {\n  margin: 0;\n}\n\
[data-blocks-incentives-icon-grid-card] {\n  height: 100%;\n}\n\
[data-blocks-incentives-icon-grid-card-body] {\n  gap: var(--fandhe-space-3);\n}\n\
[data-blocks-incentives-icon-grid-item-row] {\n  align-items: flex-start;\n}\n\
[data-blocks-incentives-icon-grid-illustration] {\n  max-width: 8rem;\n}\n\
@container blocks-incentives-icon-grid (min-width: 36rem) {\n  \
.blocks-incentives-icon-grid-instance[data-count=\"4\"] .blocks-incentives-icon-grid-grid {\n    grid-template-columns: repeat(4, minmax(0, 1fr));\n  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// Demo が期待する部品・構造・非対話制約を満たしていること
    /// （`crates/docs-site/tests/blocks_contract.rs` の横断検査と重複し
    /// 過ぎない範囲での個別固定）。
    #[test]
    fn demo_composes_expected_parts_and_avoids_forms() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"icon\"",
            "data-scope=\"image\"",
            "data-scope=\"card\"",
            "data-scope=\"item\"",
            "data-scope=\"visually-hidden\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        // B(4) + C(3) + D(3、item-row として別計上) + E(4) = 14 項目
        // （A はカードのため `.blocks-incentives-icon-grid-item` class を
        // 持たず、D は `.blocks-incentives-icon-grid-item` class を持たない
        // ため `data-blocks-incentives-icon-grid-item-row` を別途数える）。
        let plain_items = html
            .matches("class=\"blocks-incentives-icon-grid-item\"")
            .count();
        let item_rows = html
            .matches("data-blocks-incentives-icon-grid-item-row=\"\"")
            .count();
        assert_eq!(plain_items + item_rows, 14);
        assert_eq!(item_rows, 3);
        // カードは A のみ 3 枚。
        assert_eq!(
            html.matches("data-blocks-incentives-icon-grid-card=\"\"")
                .count(),
            3
        );
        // visually-hidden の見出しは B のみ 1 件。
        assert_eq!(html.matches("data-scope=\"visually-hidden\"").count(), 1);
        // イラストは E のみ 4 件。
        assert_eq!(
            html.matches("data-blocks-incentives-icon-grid-illustration=\"\"")
                .count(),
            4
        );
        assert!(!html.contains("<form"));
        assert!(!html.contains("<button"));
        assert!(!html.contains("type=\"submit\""));
        assert!(!html.contains("id=\""));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("src=\"data:"));
    }

    /// イラスト画像は装飾扱い（`alt=""`）であること。
    #[test]
    fn illustrations_are_decorative() {
        let html = render(&demo());
        assert_eq!(html.matches("alt=\"\"").count(), 4);
    }

    /// [`LAYOUT_CSS`] が想定する auto-fit グリッド・コンテナクエリでの
    /// 4 列固定（4 件インスタンス限定）を持つこと（デモ実幅を無視する
    /// ビューポート `@media` には戻さない固定）。しきい値 36rem は
    /// `.instance` が到達し得る content-box 幅の上限 40rem
    /// （`.docs-content` の `max-width: 46rem` から `.blocks-demo`・
    /// `.instance` 自身の padding 計 6rem を引いた値）以下であること
    /// （モジュール doc 「しきい値は…」節参照、イシュー #3050 PR #3505
    /// レビュー指摘の回帰防止）。
    #[test]
    fn layout_css_declares_auto_fit_grid_and_container_query_breakpoint() {
        assert!(LAYOUT_CSS.contains("auto-fit"));
        assert!(LAYOUT_CSS.contains("container-type: inline-size;"));
        assert!(LAYOUT_CSS.contains("@container blocks-incentives-icon-grid (min-width: 36rem)"));
        assert!(LAYOUT_CSS.contains("[data-count=\"4\"]"));
        assert!(LAYOUT_CSS.contains("repeat(4, minmax(0, 1fr))"));
        assert!(!LAYOUT_CSS.contains("@media"));
        // 新しきい値は `.instance` が実際に到達し得る content-box 幅の
        // 上限（46rem − blocks-demo padding 3rem − instance padding
        // 3rem = 40rem）を超えないこと（超えると旧バグの再発＝常に
        // 到達不能になる）。
        const INSTANCE_CONTENT_WIDTH_CAP_REM: f64 = 40.0;
        const THRESHOLD_REM: f64 = 36.0;
        const { assert!(THRESHOLD_REM <= INSTANCE_CONTENT_WIDTH_CAP_REM) };
    }

    /// ルート class（`demo_class` とは別名）が `demo()` の出力へ実際に
    /// 現れること（`feature_three_column_icons` と同じ Bugbot 教訓の固定）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-incentives-icon-grid-layout\""));
        assert_ne!(
            super::BLOCK.demo_class,
            "blocks-incentives-icon-grid-layout"
        );
    }
}
