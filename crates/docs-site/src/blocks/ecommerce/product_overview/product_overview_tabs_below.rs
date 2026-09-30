//! `product-overview-tabs-below` block（前半骨格はイシュー #3074〔PR
//! #3470〕、残り領域・状態違いの並記・原稿仕上げは #3075。親 #3073「Blocks
//! ecommerce/Product Overview カテゴリ新設」配下、対応表 ID R1179 を主参照
//! とする合成例。取得手段・ファイル名・内部コンポーネント識別子は記載
//! しない契約〔`faq_tabbed_accordion` モジュール doc冒頭と同型〕）。
//!
//! # 使用部品
//!
//! `image` / `heading` / `text` / `rating-group` / `button` / `list` /
//! `link` / `avatar` の 8 部品のみを合成する（[`BLOCK`] の `parts` に
//! 一致させる契約、`crates/docs-site/tests/blocks_nav.rs`/
//! `blocks_contract.rs` が検証する）。「よくある質問」「利用条件」パネルも
//! 既存 8 部品（`heading`/`text`/`list`）のみで組み立て、新規部品は
//! 追加しない。
//!
//! # 実物の `tabs::tabs` を使わない（`faq_tabbed_accordion`/
//! `feature_tabs_panel` と同じ判断）
//!
//! docs サイトは無 JS のため、実物の `tabs::tabs` を置くと「操作できる
//! ように見えて切り替わらない」トリガーと `hidden` パネルが残る
//! （既存 block が Codex/Bugbot 指摘を受けて是正済みの経路）。本 block も
//! [`static_tab_list`] で `role`/`tabindex`/`<button>` を持たない非対話の
//! `div` 列を置き、選択中の「レビュー」パネルはタブ列とは別に可視見出し
//! （`H3`）を持つ節として直下に描画する。残り 2 タブ（「よくある質問」
//! 「利用条件」）は [`panel_preview`] が `faq_tabbed_accordion` の
//! `category_preview` と同型の「『〜』タブを選択した場合のプレビュー」
//! キャプション付きで常時可視・縦積みに併記する（`hidden` は一切
//! 使わない）。[`BLOCK::parts`] に `Tabs` は含めない（実際に呼ばない部品を
//! 掲げない、`faq_tabbed_accordion`/`feature_tabs_panel` と同じ判断）。
//!
//! # 状態違いの並記（原案差分・集約元差分の扱い）
//!
//! 対応表の主参照 R1179 は 1 件のみで参照由来の差分は無い
//! （`_/blocks-intake/` は本イシュー着手時点でも worktree に存在せず、
//! `ai_chat_code_preview`/`page_heading_avatar` と同じ扱いで対応表 ID の
//! みを記す）。[`demo`] は `ai_chat_code_preview` の `demo` と同型に
//! [`caption`] 付きで 2 版を縦に並べる。版は本 block 固有の**状態違い**で
//! ある:
//!
//! 1. **代表構成**（[`Variant`] の `in_stock: true, rated: true`）:
//!    在庫あり・評価 4.0（86 件）・レビュー 3 件（前半 PR の Demo と同一）。
//! 2. **在庫切れ・評価なし**（`in_stock: false, rated: false`）: 購入
//!    ボタンのラベルを「在庫切れ」に差し替え、上段へ在庫注記を追加。評価
//!    行は「まだ評価はありません」に差し替え、レビューパネルは「レビュー
//!    はまだありません。」に差し替える。「よくある質問」「利用条件」の
//!    プレビューは両版で同一。
//!
//! 3 つ目の案（狭幅版の並記）は採らない: 本 block の 2 カラム切替は
//! `LAYOUT_CSS` の `@media (min-width: 48rem)` viewport クエリであり、
//! `ai_chat_code_preview` の `@container` クエリと異なりページ内の並記
//! （リサイズなしでの視覚確認）では再現できないため。
//!
//! 版 2（[`Variant`] の `rated: false`）は `rating_group::root` を
//! 一切出力しないため id が発生せず、`ai_chat_code_preview` の
//! `suffix` 接尾辞方式は不要（`rating_label_ids_are_unique` テストが各 id
//! ちょうど 1 件のままであることで衝突なしを証明する）。将来 3 版目で
//! 評価付き版を増やす場合は `suffix` 方式の導入を検討すること。
//!
//! # 購入ボタンは disabled の静的表示
//!
//! 無 JS のため押しても何も起きないボタンを操作可能なまま残さない
//! （`feature_tabs_panel` の CTA と同じ判断）。`ButtonProps { disabled:
//! true, .. }` で固定する。
//!
//! # 共有リンクは架空の固定 href
//!
//! 実在 SNS サービス名を持ち込まず「共有 A」「共有 B」の架空ラベルにし、
//! href は本リポジトリの固定 URL（`REPO` 定数、他 block と同型）にする。
//!
//! # id を block 固有定数にする理由
//!
//! 商品の評価ラベル・レビュー各件の評価ラベルはいずれも
//! `rating_group::label` の `id` を持つため、`crates/docs-site/tests/
//! blocks_contract.rs::demo_output_has_no_dangling_aria_references_or_
//! duplicate_ids` の「id の非重複」検証のため block 固有の接頭辞付き定数
//! （[`RATING_LABEL_ID`]）とレビュー件別の動的 id（[`review_rating_label_id`]）
//! にする。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `heading`/`text`/`image`/`button`/`avatar`/`link::root`/`list::root`/
//! `rating_group::root` はいずれも `drop_class_attr` により呼び出し側
//! `class` を黙って除去する契約を持つため、これらへの CSS フックは
//! `data-*` 属性で渡し、素の `div` には `class` をそのまま使う
//! （`crate::blocks` モジュール doc「CSS フックが `class` と `[data-*]` で
//! 混在する理由」節参照）。
//!
//! # `<form>` を持たない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節に従い、本 Demo
//! はフォーム・送信処理・状態機械を持たない静的な合成例である。商品名・
//! レビュー本文・人名はすべて架空のもの（実企業・PII・クレデンシャルを
//! 含まない）。

use crate::blocks::dummy_assets;
use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, el_owned, h3, p, section, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::button::{button, ButtonProps};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::image::{image, AspectRatio, ImageProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::list::{self, ListType, ListVariant};
use fandhe_frontend_pre_styled_ui::rating_group::{
    self, RatingGroup, RatingGroupProps, RatingItemFlags,
};
use fandhe_frontend_pre_styled_ui::recipe::ColorPalette;
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::Size;

/// 商品の評価ラベル（`rating_group::label` の `id`）。
const RATING_LABEL_ID: &str = "blocks-product-overview-tabs-below-rating-label";

/// 本リポジトリの固定 URL（共有リンクの href、他 block と同型の判断）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// レビュー 1 件分のダミーデータ（人名は [`dummy_assets::PERSON_NAMES`] の
/// 索引、評価は 5 段階、本文は架空のもの）。
const REVIEWS: [(usize, u32, &str); 3] = [
    (
        0,
        5,
        "手首の負担が減り、長時間の作業がかなり楽になりました。",
    ),
    (1, 4, "質感は良いですが、もう少し大きいサイズも欲しいです。"),
    (
        2,
        5,
        "梱包も丁寧で、届いてすぐに使い始められました。おすすめです。",
    ),
];

/// レビュー `index` 件目の評価ラベル id（全件を通じて一意にする、
/// モジュール doc「id を block 固有定数にする理由」節）。
fn review_rating_label_id(index: usize) -> String {
    format!("blocks-product-overview-tabs-below-review-{index}-rating-label")
}

/// 「よくある質問」パネルのダミー Q&A（架空文、モジュール doc「使用部品」
/// 節）。
const FAQS: [(&str, &str); 3] = [
    (
        "サイズは選べますか",
        "現在は 90 × 40 cm の 1 サイズのみの展開です。",
    ),
    (
        "水洗いはできますか",
        "表面は撥水コーティングのため軽い汚れは水拭きで対応できます。丸洗いは推奨していません。",
    ),
    (
        "滑り止めは付いていますか",
        "裏面全体に滑り止め加工を施しており、デスク上でずれにくい仕様です。",
    ),
];

/// 「利用条件」パネルの箇条書き項目（架空文）。
const TERMS: [&str; 3] = [
    "返品は到着後 14 日以内、未使用・未開封の商品に限り受け付けます。",
    "初期不良は 1 年間の保証対象です。通常使用による経年劣化は対象外です。",
    "配送は国内のみの対応です。離島・一部地域は追加日数がかかる場合があります。",
];

/// Demo 版ごとの状態フラグ（モジュール doc「状態違いの並記」節）。
/// `in_stock`/`rated` の 2 bool のみを持ち、列挙型・builder は作らない
/// （現状 2 版のみのため）。
struct Variant {
    /// 在庫あり（false のとき購入ボタンを「在庫切れ」に差し替える）。
    in_stock: bool,
    /// 評価集計あり（false のとき「まだ評価はありません」に差し替え、
    /// レビューパネルも「レビューはまだありません。」に差し替える）。
    rated: bool,
}

/// カテゴリラベルのみを装飾として示す非対話タブ列（モジュール doc「実物の
/// `tabs::tabs` を使わない」節）。`role`/`tabindex`/`<button>` を一切持たず、
/// `selected` に一致するラベルだけ `data-state="active"` にする。各ラベルは
/// `aria-hidden` で支援技術のツリーから除外する。
fn static_tab_list(selected: &str) -> Node {
    let labels = [
        ("reviews", "レビュー"),
        ("faq", "よくある質問"),
        ("terms", "利用条件"),
    ];
    el_owned(
        "div",
        vec![(
            "class".to_string(),
            "blocks-product-overview-tabs-below-tablist".to_string(),
        )],
        labels
            .iter()
            .map(|(value, label)| {
                let state = if *value == selected {
                    "active"
                } else {
                    "inactive"
                };
                el_owned(
                    "div",
                    vec![
                        (
                            "class".to_string(),
                            "blocks-product-overview-tabs-below-tab".to_string(),
                        ),
                        ("data-state".to_string(), state.to_string()),
                        ("aria-hidden".to_string(), "true".to_string()),
                    ],
                    vec![text(*label)],
                )
            })
            .collect(),
    )
}

/// 商品画像（左列）。
fn product_image() -> Node {
    image(
        &ImageProps {
            aspect_ratio: AspectRatio::Square,
            ..ImageProps::new(dummy_assets::PRODUCT_SRC, "デスクマット Pro の商品画像")
        },
        vec![("data-blocks-product-overview-tabs-below-image", "")],
    )
}

/// 商品の評価（readonly の静的表示。無 JS のため操作不能、
/// `card_meta_cta::product_card` と同じ判断）。`rated` が false の版
/// （モジュール doc「状態違いの並記」節）は `rating_group::root` を
/// 一切出力せず「まだ評価はありません」の文だけを返す。
fn product_rating(rated: bool) -> Node {
    if !rated {
        return styled_text::text(
            &TextProps {
                variant: TextVariant::Muted,
                ..TextProps::default()
            },
            vec![("data-blocks-product-overview-tabs-below-rating", "")],
            vec![text("まだ評価はありません")],
        );
    }
    let props = RatingGroupProps {
        disabled: false,
        readonly: true,
        required: false,
    };
    let state = RatingGroup::new(5, Some(4), true);
    let label = rating_group::label(
        &props,
        Some(RATING_LABEL_ID),
        vec![],
        vec![text("評価 4.0（86 件）")],
    );
    let items: Vec<Node> = (1..=state.count())
        .map(|i| {
            rating_group::item(
                i,
                RatingItemFlags {
                    checked: state.is_checked(i),
                    highlighted: state.is_highlighted(i),
                    disabled: false,
                    readonly: true,
                },
                &format!("{i} star{}", if i == 1 { "" } else { "s" }),
                vec![],
                vec![],
            )
        })
        .collect();
    let control = rating_group::control(&props, Some(RATING_LABEL_ID), vec![], items);
    rating_group::root(
        Size::Sm,
        ColorPalette::Accent,
        &props,
        vec![("data-blocks-product-overview-tabs-below-rating", "")],
        vec![label, control],
    )
}

/// 特徴リストの 1 行。
fn feature_item(label: &str) -> Node {
    list::item(vec![], vec![text(label)])
}

/// 共有リンク 1 件（架空ラベル、固定 href、モジュール doc「共有リンクは
/// 架空の固定 href」節）。
fn share_link(label: &str) -> Node {
    link::root(REPO, &LinkProps::default(), vec![], vec![text(label)])
}

/// 在庫切れ注記（`variant.in_stock` が false のときのみ [`info_column`]
/// 冒頭へ挿入する、モジュール doc「状態違いの並記」節）。
fn stock_notice() -> Node {
    styled_text::text(
        &TextProps {
            variant: TextVariant::Muted,
            size: TextSize::Sm,
            ..TextProps::default()
        },
        vec![],
        vec![text("現在在庫切れです。入荷時期は未定です。")],
    )
}

/// 上段・情報列（商品名 + 評価 + 説明 + 購入ボタン + 特徴リスト + 共有
/// リンク）。`variant` により評価表示・購入ボタンのラベルが切り替わる
/// （モジュール doc「状態違いの並記」節）。
fn info_column(variant: &Variant) -> Node {
    let mut children = vec![heading(
        HeadingLevel::H2,
        &HeadingProps {
            size: HeadingSize::Xl2,
            weight: HeadingWeight::Semibold,
        },
        vec![],
        vec![text("デスクマット Pro")],
    )];
    if !variant.in_stock {
        children.push(stock_notice());
    }
    children.push(product_rating(variant.rated));
    children.push(styled_text::text(
        &TextProps {
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![],
        vec![text(
            "手首の負担を抑える傾斜構造の作業用デスクマットです。撥水コーティング表面で日常使いにも適しています。",
        )],
    ));
    children.push(button(
        &ButtonProps {
            disabled: true,
            ..ButtonProps::default()
        },
        vec![("data-blocks-product-overview-tabs-below-cta", "")],
        vec![text(if variant.in_stock {
            "カートに追加"
        } else {
            "在庫切れ"
        })],
    ));
    children.push(list::root(
        ListType::default(),
        ListVariant::Plain,
        vec![("data-blocks-product-overview-tabs-below-features", "")],
        vec![
            feature_item("サイズ：90 × 40 cm"),
            feature_item("素材：撥水コーティング表面"),
            feature_item("お届け目安：3〜5 営業日"),
        ],
    ));
    children.push(div(
        vec![("class", "blocks-product-overview-tabs-below-share")],
        vec![share_link("共有 A"), share_link("共有 B")],
    ));
    div(
        vec![("class", "blocks-product-overview-tabs-below-info")],
        children,
    )
}

/// 上段（商品画像 + 情報列、狭幅では縦積み、[`LAYOUT_CSS`] 参照）。
fn top_section(variant: &Variant) -> Node {
    div(
        vec![("class", "blocks-product-overview-tabs-below-top")],
        vec![product_image(), info_column(variant)],
    )
}

/// レビュー 1 件分（アバター + 氏名 + 評価 + 本文）。
fn review_item((index, rating, body): (usize, u32, &str)) -> Node {
    let name = dummy_assets::PERSON_NAMES[index % dummy_assets::PERSON_NAMES.len()];
    let initials: String = name
        .split_whitespace()
        .filter_map(|part| part.chars().next())
        .collect();
    let label_id = review_rating_label_id(index);

    let props = RatingGroupProps {
        disabled: false,
        readonly: true,
        required: false,
    };
    let state = RatingGroup::new(5, Some(rating), true);
    let label = rating_group::label(
        &props,
        Some(label_id.as_str()),
        vec![],
        vec![text(format!("評価 {rating}.0"))],
    );
    let items: Vec<Node> = (1..=state.count())
        .map(|i| {
            rating_group::item(
                i,
                RatingItemFlags {
                    checked: state.is_checked(i),
                    highlighted: state.is_highlighted(i),
                    disabled: false,
                    readonly: true,
                },
                &format!("{i} star{}", if i == 1 { "" } else { "s" }),
                vec![],
                vec![],
            )
        })
        .collect();
    let control = rating_group::control(&props, Some(label_id.as_str()), vec![], items);
    let rating_node = rating_group::root(
        Size::Sm,
        ColorPalette::Accent,
        &props,
        vec![],
        vec![label, control],
    );

    div(
        vec![("class", "blocks-product-overview-tabs-below-review")],
        vec![
            avatar::root(
                &AvatarProps {
                    size: Size::Sm,
                    ..AvatarProps::default()
                },
                vec![],
                vec![avatar::fallback(
                    ImageStatus::Error,
                    vec![],
                    vec![text(initials)],
                )],
            ),
            div(
                vec![("class", "blocks-product-overview-tabs-below-review-body")],
                vec![
                    div(
                        vec![("class", "blocks-product-overview-tabs-below-review-name")],
                        vec![text(name)],
                    ),
                    rating_node,
                    styled_text::text(
                        &TextProps {
                            size: TextSize::Sm,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text(body)],
                    ),
                ],
            ),
        ],
    )
}

/// 版キャプション（`ai_chat_code_preview::caption` と同型、素の `<p>` で
/// `heading` 部品を使わない）。
fn caption(label: &str) -> Node {
    p(
        vec![("class", "blocks-product-overview-tabs-below-caption")],
        vec![text(label)],
    )
}

/// 非選択タブ（「よくある質問」「利用条件」）のプレビュー併記
/// （`faq_tabbed_accordion::category_preview` と同型の
/// 「『〜』タブを選択した場合のプレビュー」キャプション付き常時可視節、
/// モジュール doc「実物の `tabs::tabs` を使わない」節参照）。
fn panel_preview(label: &str, body: Node) -> Node {
    div(
        vec![("class", "blocks-product-overview-tabs-below-preview")],
        vec![
            h3(
                vec![],
                vec![text(format!("「{label}」タブを選択した場合のプレビュー"))],
            ),
            body,
        ],
    )
}

/// 「よくある質問」パネル本体（質問は `H4`/`Sm` の見出し、回答は
/// `styled_text::text`、[`FAQS`] 定数参照）。
fn faq_panel() -> Node {
    div(
        vec![("class", "blocks-product-overview-tabs-below-faq")],
        FAQS.iter()
            .map(|(question, answer)| {
                div(
                    vec![("class", "blocks-product-overview-tabs-below-faq-item")],
                    vec![
                        heading(
                            HeadingLevel::H4,
                            &HeadingProps {
                                size: HeadingSize::Sm,
                                ..HeadingProps::default()
                            },
                            vec![],
                            vec![text(*question)],
                        ),
                        styled_text::text(
                            &TextProps {
                                size: TextSize::Sm,
                                ..TextProps::default()
                            },
                            vec![],
                            vec![text(*answer)],
                        ),
                    ],
                )
            })
            .collect(),
    )
}

/// 「利用条件」パネル本体（導入文 + 箇条書き 3 項目、[`TERMS`] 定数参照）。
fn terms_panel() -> Node {
    div(
        vec![],
        vec![
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text("ご購入前に以下の利用条件をご確認ください。")],
            ),
            list::root(
                ListType::default(),
                ListVariant::Marker,
                vec![],
                TERMS
                    .iter()
                    .map(|term| list::item(vec![], vec![text(*term)]))
                    .collect(),
            ),
        ],
    )
}

/// 下段・全幅タブ（静的タブ列 + レビュー一覧パネル + 「よくある質問」
/// 「利用条件」プレビュー併記、モジュール doc「実物の `tabs::tabs` を
/// 使わない」節参照）。`variant.rated` が false のときレビューパネルは
/// 一覧の代わりに「レビューはまだありません。」の文を返す。
fn tabs_section(variant: &Variant) -> Node {
    let reviews_body: Node = if variant.rated {
        div(
            vec![("class", "blocks-product-overview-tabs-below-review-list")],
            REVIEWS.iter().copied().map(review_item).collect(),
        )
    } else {
        styled_text::text(
            &TextProps::default(),
            vec![],
            vec![text("レビューはまだありません。")],
        )
    };
    div(
        vec![("class", "blocks-product-overview-tabs-below-tabs")],
        vec![
            static_tab_list("reviews"),
            div(
                vec![("class", "blocks-product-overview-tabs-below-reviews")],
                vec![h3(vec![], vec![text("レビュー")]), reviews_body],
            ),
            panel_preview("よくある質問", faq_panel()),
            panel_preview("利用条件", terms_panel()),
        ],
    )
}

/// 1 版分の shell（上段 2 カラム + 下段全幅タブ）を組み立てる
/// （モジュール doc「状態違いの並記」節）。
fn shell(variant: &Variant) -> Node {
    div(
        vec![("class", "blocks-product-overview-tabs-below-layout")],
        vec![top_section(variant), tabs_section(variant)],
    )
}

/// `product-overview-tabs-below` の Demo 本体。2 版（代表構成・在庫切れ・
/// 評価なし）をキャプション付きで縦に並べる（モジュール doc「状態違いの
/// 並記」節）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-product-overview-tabs-below-demo")],
        vec![
            caption("代表構成"),
            section(
                vec![],
                vec![shell(&Variant {
                    in_stock: true,
                    rated: true,
                })],
            ),
            caption("在庫切れ・評価なし"),
            section(
                vec![],
                vec![shell(&Variant {
                    in_stock: false,
                    rated: false,
                })],
            ),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/product-overview-tabs-below/",
    title: "product-overview-tabs-below",
    category: BlockCategory::ProductOverview,
    rust_source:
        "crates/docs-site/src/blocks/ecommerce/product_overview/product_overview_tabs_below.rs",
    demo_class: "blocks-product-overview-tabs-below",
    parts: &[
        Part {
            label: "Image",
            path: "/themes/image/",
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
            label: "Rating Group",
            path: "/themes/rating-group/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "List",
            path: "/themes/list/",
        },
        Part {
            label: "Link",
            path: "/themes/link/",
        },
        Part {
            label: "Avatar",
            path: "/themes/avatar/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `product_overview_tabs_below` 固有のレイアウト規則（`crate::blocks::
/// LAYOUT_CSS` doc「block 固有 CSS の置き場」節と同型）。
///
/// ルート class（`-layout`）は [`Block::demo_class`] と意図的に別名にする
/// （`faq_tabbed_accordion`/`card_meta_cta` と同じ Bugbot 教訓の回避）。
/// 上段は既定 1 カラム、`min-width: 48rem` で 2 カラムへ切り替える
/// （ブレークポイントトークンは `@media` 内で解決できないため rem 直書き、
/// `contact_image_info` と同じ判断）。タブ列は下線型で `overflow-x: auto`
/// による横スクロールのみを許す。
const LAYOUT_CSS: &str = "\
.blocks-product-overview-tabs-below-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n}\n\
.blocks-product-overview-tabs-below-top {\n  display: grid;\n  grid-template-columns: 1fr;\n  gap: var(--fandhe-space-6);\n}\n\
@media (min-width: 48rem) {\n  .blocks-product-overview-tabs-below-top {\n    grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);\n  }\n}\n\
[data-blocks-product-overview-tabs-below-image] {\n  width: 100%;\n}\n\
.blocks-product-overview-tabs-below-info {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n  align-items: flex-start;\n}\n\
[data-blocks-product-overview-tabs-below-cta] {\n  margin-top: var(--fandhe-space-2);\n}\n\
.blocks-product-overview-tabs-below-share {\n  display: flex;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-product-overview-tabs-below-tabs {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-product-overview-tabs-below-tablist {\n  display: flex;\n  flex-wrap: nowrap;\n  overflow-x: auto;\n  overflow-y: hidden;\n  gap: var(--fandhe-space-2);\n  border-bottom: 1px solid var(--fandhe-color-border);\n  padding-bottom: 1px;\n}\n\
.blocks-product-overview-tabs-below-tab {\n  display: inline-flex;\n  align-items: center;\n  flex-shrink: 0;\n  padding: var(--fandhe-space-2) var(--fandhe-space-4);\n  font-size: var(--fandhe-font-font-size-sm);\n  font-weight: var(--fandhe-font-font-weight-medium);\n  white-space: nowrap;\n  color: var(--fandhe-color-fg-muted);\n  border-bottom: 2px solid transparent;\n  margin-bottom: -2px;\n  cursor: default;\n}\n\
.blocks-product-overview-tabs-below-tab[data-state=\"active\"] {\n  color: var(--fandhe-color-fg);\n  border-bottom-color: var(--fandhe-color-accent);\n}\n\
.blocks-product-overview-tabs-below-review-list {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-product-overview-tabs-below-review {\n  display: flex;\n  gap: var(--fandhe-space-3);\n  align-items: flex-start;\n}\n\
.blocks-product-overview-tabs-below-review-body {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1);\n}\n\
.blocks-product-overview-tabs-below-review-name {\n  font-weight: var(--fandhe-font-font-weight-medium);\n}\n\
.blocks-product-overview-tabs-below-demo {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n}\n\
.blocks-product-overview-tabs-below-caption {\n  margin: 0;\n  font-size: var(--fandhe-font-font-size-sm);\n  color: var(--fandhe-color-fg-muted);\n}\n\
.blocks-product-overview-tabs-below-preview {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n  padding-top: var(--fandhe-space-4);\n  border-top: 1px solid var(--fandhe-color-border);\n}\n\
.blocks-product-overview-tabs-below-faq {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-product-overview-tabs-below-faq-item {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1);\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS, RATING_LABEL_ID};
    use fandhe_frontend_core::render;

    /// Demo が [`crate::blocks::Block::parts`] と一致する 8 部品すべてを
    /// 出力すること。
    #[test]
    fn demo_composes_expected_parts() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"image\"",
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"rating-group\"",
            "data-scope=\"button\"",
            "data-scope=\"list\"",
            "data-scope=\"link\"",
            "data-scope=\"avatar\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert!(!html.contains("<form"));
    }

    /// 実物の `tabs::tabs` を使わないため `role="tab"`/`hidden` パネルが
    /// 一切現れないこと（モジュール doc「実物の `tabs::tabs` を使わない」
    /// 節）。
    #[test]
    fn demo_has_no_real_tabs_or_hidden_panels() {
        let html = render(&demo());
        assert!(!html.contains(r#"role="tab""#));
        assert!(!html.contains(r#"role="tabpanel""#));
        assert!(!html.contains("hidden=\"\""));
        assert!(html.contains("class=\"blocks-product-overview-tabs-below-tablist\""));
    }

    /// 静的タブ列は「レビュー」のみ `data-state="active"` であること
    /// （2 版分、1 版ごとに 1 件）。
    #[test]
    fn static_tab_list_marks_only_reviews_as_active() {
        let html = render(&demo());
        assert_eq!(html.matches("data-state=\"active\"").count(), 2);
        assert!(html.contains("レビュー"));
        assert!(html.contains("よくある質問"));
        assert!(html.contains("利用条件"));
    }

    /// 選択中「レビュー」パネルの見出しと「よくある質問」「利用条件」
    /// プレビューの見出しが `aria-hidden` なタブ列の外に可視・支援技術から
    /// 到達可能な形で存在すること（1 版あたり 3 見出し × 2 版 = 6 件）。
    #[test]
    fn reviews_heading_is_visible_outside_aria_hidden_tab_list() {
        let html = render(&demo());
        assert!(html.contains("<h3"));
        assert_eq!(html.matches("<h3").count(), 6);
    }

    /// 購入ボタンが `type="button"` かつ `disabled` の静的表示であること
    /// （モジュール doc「購入ボタンは disabled の静的表示」節）。
    #[test]
    fn purchase_button_is_type_button_and_disabled() {
        let html = render(&demo());
        assert!(html.contains(r#"type="button""#));
        assert!(html.contains("disabled"));
        assert!(html.contains("カートに追加"));
    }

    /// [`LAYOUT_CSS`] が上段 2 カラムのブレークポイントとタブ列の
    /// 横スクロールを宣言すること。
    #[test]
    fn layout_css_declares_two_column_top_and_scrollable_tablist() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("@media (min-width: 48rem)"));
        assert!(LAYOUT_CSS.contains(".blocks-product-overview-tabs-below-tablist {"));
    }

    /// [`LAYOUT_CSS`] が版キャプション・プレビュー節のセレクタを宣言する
    /// こと（モジュール doc「状態違いの並記」節・「実物の `tabs::tabs` を
    /// 使わない」節）。
    #[test]
    fn layout_css_declares_caption_and_preview() {
        assert!(LAYOUT_CSS.contains(".blocks-product-overview-tabs-below-caption {"));
        assert!(LAYOUT_CSS.contains(".blocks-product-overview-tabs-below-preview {"));
        assert!(LAYOUT_CSS.contains(".blocks-product-overview-tabs-below-faq {"));
    }

    /// ルート class（`demo_class` とは別名）が [`demo`] の出力へ実際に
    /// 現れること。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-product-overview-tabs-below-layout\""));
        assert_ne!(
            super::BLOCK.demo_class,
            "blocks-product-overview-tabs-below-layout"
        );
    }

    /// 評価ラベルの id が重複しないこと（商品 1 件 + レビュー 3 件 = 4 件、
    /// モジュール doc「id を block 固有定数にする理由」節）。版 2 は
    /// `rating_group::root` を一切出力しないため id を持たず、全体でも
    /// 各 id ちょうど 1 件のまま衝突しない（モジュール doc「状態違いの
    /// 並記」節）。
    #[test]
    fn rating_label_ids_are_unique() {
        let html = render(&demo());
        assert_eq!(
            html.matches(&format!("id=\"{RATING_LABEL_ID}\"")).count(),
            1
        );
        for index in 0..3 {
            let id = super::review_rating_label_id(index);
            assert_eq!(html.matches(&format!("id=\"{id}\"")).count(), 1);
        }
    }

    /// [`demo`] が 2 版のキャプション（「代表構成」「在庫切れ・評価なし」）
    /// を持つこと（モジュール doc「状態違いの並記」節）。
    #[test]
    fn demo_renders_two_captioned_variants() {
        let html = render(&demo());
        assert_eq!(
            html.matches("blocks-product-overview-tabs-below-caption")
                .count(),
            2
        );
        assert!(html.contains("代表構成"));
        assert!(html.contains("在庫切れ・評価なし"));
    }

    /// 在庫切れ・評価なし版は購入ボタンのラベルを「在庫切れ」に差し替え、
    /// 評価を「まだ評価はありません」、レビューパネルを「レビューは
    /// まだありません。」に差し替えること。代表構成版の `rating-group`
    /// はそのまま残る（[`rating_label_ids_are_unique`] が衝突なしを
    /// 別途固定する）。
    #[test]
    fn out_of_stock_variant_swaps_cta_label_and_hides_rating() {
        let html = render(&demo());
        assert_eq!(html.matches("在庫切れ").count(), 3); // 版キャプション + CTA ラベル + 在庫注記
        assert!(html.contains("まだ評価はありません"));
        assert!(html.contains("レビューはまだありません。"));
        // 代表構成版の rating-group（商品 1 + レビュー 3 = 4 インスタンス
        // × root/label/control/item5 の 8 要素 = 32 件）は維持される。
        assert_eq!(html.matches("data-scope=\"rating-group\"").count(), 32);
    }

    /// 「よくある質問」「利用条件」パネルが `hidden` を伴わず常時可視で
    /// あり、両版に併記されること（モジュール doc「実物の `tabs::tabs`
    /// を使わない」節）。
    #[test]
    fn faq_and_terms_panels_are_always_visible() {
        let html = render(&demo());
        assert!(!html.contains("hidden=\"\""));
        for (question, answer) in super::FAQS {
            assert!(html.contains(question), "html should contain {question}");
            assert!(html.contains(answer), "html should contain {answer}");
        }
        for term in super::TERMS {
            assert!(html.contains(term), "html should contain {term}");
        }
        assert_eq!(
            html.matches("「よくある質問」タブを選択した場合のプレビュー")
                .count(),
            2
        );
        assert_eq!(
            html.matches("「利用条件」タブを選択した場合のプレビュー")
                .count(),
            2
        );
    }
}
