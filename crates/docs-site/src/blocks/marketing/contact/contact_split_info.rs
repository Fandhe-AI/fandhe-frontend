//! `contact-split-info` block（イシュー #2835。親トラッキング #2730
//! 「Blocks 目的別パーツ拡充」配下、対応表 ID R0854（主参照）・R0442
//! （集約元）の 2 件を構造の参照元とする合成例。見出し左 + 連絡先情報右）。
//! 取得手段・ファイル名・内部コンポーネント識別子は記載しない
//! （`docs/design/motion-reference-adoption-policy.md` §9 と同じライセンス
//! 上の転記制限）。
//!
//! # 使用部品
//!
//! `heading` / `text` / `card` / `icon` / `link` の 5 部品のみを合成する
//! （[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//! 新しい UI 部品は追加しない。
//!
//! # 2 行構成（集約元との差分）
//!
//! 主参照（R0854）は「連絡先」「拠点」の 2 段 × 右側に淡色カードを 2×2 に
//! 並べる構成、集約元（R0442）は左に見出し + SNS 風リンク列・右に連絡先
//! リンクという構成を持つ。要件（`lg` 以上で左見出し + 右情報の 2 カラム、
//! カテゴリが複数あるときはペアを縦に繰り返す）を満たしつつ両方の構造を
//! 1 つの Demo に収めるため、1 行目（「連絡先」）に集約元の SNS 風リンク列
//! を左側へ組み込み、右側は主参照どおり窓口カード 2×2 とする。2 行目
//! （「拠点」）は主参照の 2 段目（拠点情報）に対応し、SNS 列を持たない
//! ことで「行ごとに左側の内容量が違う」ことを示す（[`split_row`]）。
//!
//! # 見出しレベル
//!
//! ページ側が `## Demo` として `h2` を出すため、各行の左見出しは
//! `HeadingLevel::H3` にする。カード内の項目名・拠点名は `card::title`
//! （`<h3>` 固定）を使うと行見出しと同じレベルになって階層が崩れるため、
//! 意図的に `card::body` の中で `heading(HeadingLevel::H4, …)` を使う
//! （`contact_info_columns` の窓口名/拠点名と同じ判断）。
//!
//! # SNS 風リンクの汎用化（R0442 からの意図的な逸脱）
//!
//! R0442 は実在 SNS ブランドのアイコン・名称を用いるが、本 block では
//! 実在プラットフォーム名もロゴも一切使わず、「公式ブログ」「コミュニティ」
//! 「ニュースレター」「開発者フォーラム」という汎用ラベルと自作の抽象幾何
//! アイコン（[`geo_icon`]）で表す（実在ブランドのロゴ・商標を模した SVG を
//! 持ち込まない不変条件、`wireframe-ui` の `brand`/`icon::brand` と同じ
//! 判断軸）。
//!
//! # ブレークポイント（sm=40rem/lg=64rem をリテラル直書きする理由）
//!
//! テーマの breakpoint トークンは `@media` 条件式の中では解決できない
//! （CSS custom property は宣言側でのみ有効）ため、
//! `fandhe_frontend_pre_styled_ui::recipe::Breakpoint` の `Sm`（640px =
//! 40rem）・`Lg`（1024px = 64rem）と一致するリテラル値を [`LAYOUT_CSS`] へ
//! 直書きする（`contact_info_columns` と同じ判断）。狭幅は左見出みの下へ
//! 情報を 1 列に積み、`sm` 以上で右側カード列を 2 列、`lg` 以上で行全体を
//! 「左見出し + 右情報」の 2 カラムへ切り替える。
//!
//! # `text` の名前衝突
//!
//! `fandhe_frontend_pre_styled_ui::text::text`（`<p>` を組み立てる styled
//! パート関数）と `fandhe_frontend_core::text`（テキストノード生成関数）が
//! 同名のため、styled 側を `styled_text` として取り込む（`crate::blocks`
//! 内の他 block と同じ回避方法）。
//!
//! # 自作幾何アイコン
//!
//! lucide 等の既存アイコンセットの path を複製しないため、
//! `contact_info_columns::geo_icon` と同型の自作ヘルパ [`geo_icon`] で描く。
//! `path` へ `fill="none"` + `stroke="currentColor"` を明示し、
//! `icon::icon` の `<svg>` 側が固定で持つ `fill="currentColor"`（塗り面）を
//! 上書きして線画として描画する。いずれも隣に可視テキストがあるため装飾
//! 扱い（`IconProps::label` は `None` のまま、`aria-hidden="true"`）とする。
//!
//! # 同じ可視テキストのリンクが複数並ぶ問題への対処
//!
//! 拠点カードのリンクは 4 件とも同じ可視テキストのため、
//! 「（可視テキスト）（拠点名）」形式の `aria-label` を付与して区別する
//! （WCAG 2.5.3 label-in-name に適合、`contact_info_columns` と同じ手段）。
//! SNS 風リンクは 4 件とも可視テキストが相異なるため `aria-label` は付与
//! しない。`id`/`aria-describedby` は出力しない。
//!
//! # リンク先を固定リポジトリ URL にする・mailto/tel を使わない理由
//!
//! `crate::blocks` の他 block と同じく `link::root` + 固定 URL（[`REPO`]）
//! で「実際に押せる」導線を表す。連絡先カード（電話/メール/所在地/受付
//! 時間）はプレーンテキストのみで表示しリンク化しない。`href="#"` も
//! 使わない（`contact_info_columns` と同じ安全側判断）。
//!
//! # 表示文言と遷移先の食い違い是正（codex 再レビュー是正）
//!
//! 当初、拠点カードのリンクは「地図を見る」、SNS 風リンクは「公式ブログ」
//! 等の可視テキストのまま [`REPO`] へ遷移していたが、拠点ごと・
//! プラットフォームごとに異なる内容を示す複数のリンクがすべて同一の
//! GitHub リポジトリへ遷移してしまい、表示内容と遷移先が食い違うという
//! 指摘（codex 再レビュー）を受けた。架空の拠点・SNS それぞれに対応する
//! 実在の個別 URL を捏造することはできない（`crate::blocks` モジュール doc
//! の安全側判断）ため、`error_page_centered.rs`「サポートへの導線」節と
//! 同じ方針（実在する destination を維持しつつ、表示文言を遷移先が
//! わかる内容へ変更する）を採り、拠点カードのリンク文言を「GitHub で
//! 見る」、SNS 風リンクの可視テキストへ「（GitHub）」を付記する形に変更
//! した。`aria-label` も新しい可視テキストに合わせて更新済み。
//!
//! # `<form>` を持たない・データ取得/送信を行わない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である。文言・連絡先・拠点住所はすべて架空のもの（実企業名・実クレデン
//! シャル・PII を含まない。住所の郵便番号はすべて `000-000X` の架空値）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::card::{self, CardVariant};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps, LinkVariant};
use fandhe_frontend_pre_styled_ui::recipe::ColorPalette;
use fandhe_frontend_pre_styled_ui::text::{
    self as styled_text, TextProps, TextSize, TextVariant, TextWeight,
};
use fandhe_frontend_pre_styled_ui::Size;

/// 装飾用の自作幾何アイコン（lucide 等の既存アイコンセットの path を
/// 複製しないための単純図形、`contact_info_columns::geo_icon` と同型）。
fn geo_icon(path_d: &'static str) -> Node {
    icon(
        &IconProps {
            size: Size::Sm,
            ..IconProps::default()
        },
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

/// 受話器の幾何アイコン（電話）。
fn phone_icon() -> Node {
    geo_icon("M5 4h4l2 5-2 1a11 11 0 006 6l1-2 5 2v4a2 2 0 01-2 2A16 16 0 013 6a2 2 0 012-2z")
}

/// 封筒の幾何アイコン（メール）。
fn mail_icon() -> Node {
    geo_icon("M4 6h16v12H4V6z M4 7l8 6 8-6")
}

/// 位置ピンの幾何アイコン（所在地/拠点）。
fn pin_icon() -> Node {
    geo_icon("M12 21s7-7.5 7-12a7 7 0 10-14 0c0 4.5 7 12 7 12z M12 11a2 2 0 100-4 2 2 0 000 4z")
}

/// 時計の幾何アイコン（受付時間）。
fn clock_icon() -> Node {
    geo_icon("M12 3a9 9 0 100 18 9 9 0 000-18z M12 7v5l4 2")
}

/// 折り目付き文書の幾何アイコン（公式ブログ）。
fn doc_icon() -> Node {
    geo_icon("M6 4h9l3 3v13H6V4z M15 4v3h3 M9 11h6 M9 14h6 M9 17h4")
}

/// 円 2 つ + 弧 2 つの幾何アイコン（コミュニティ）。
fn cluster_icon() -> Node {
    geo_icon(
        "M9 10a3 3 0 100-6 3 3 0 000 6z M15 10a3 3 0 100-6 3 3 0 000 6z \
         M4 19c0-3 2.5-5 5-5s5 2 5 5 M10 19c0-2.5 2-4 5-4s5 1.5 5 4",
    )
}

/// 紙飛行機の幾何アイコン（ニュースレター）。
fn paperplane_icon() -> Node {
    geo_icon("M3 11l18-7-7 18-3-7-8-4z")
}

/// 吹き出し 2 つの幾何アイコン（開発者フォーラム）。
fn bubbles_icon() -> Node {
    geo_icon("M4 5h13v8H8l-4 4V5z M14 9h6v7l-3-3h-3V9z")
}

/// 連絡先窓口 1 件分のダミーデータ（架空、実在の企業・人物とは無関係）。
struct ContactItem {
    title: &'static str,
    value: &'static str,
    icon_fn: fn() -> Node,
}

/// 連絡先窓口一覧（架空、4 件。主参照 R0854 の 1 段目の 2×2 カードに対応）。
const CONTACT_ITEMS: [ContactItem; 4] = [
    ContactItem {
        title: "電話",
        value: "050-0000-0001（架空の番号）",
        icon_fn: phone_icon,
    },
    ContactItem {
        title: "メール",
        value: "info[at]example.invalid（架空のアドレス）",
        icon_fn: mail_icon,
    },
    ContactItem {
        title: "所在地",
        value: "〒000-0005 東京都架空区架空 1-1",
        icon_fn: pin_icon,
    },
    ContactItem {
        title: "受付時間",
        value: "平日 9:00〜18:00（架空の営業時間）",
        icon_fn: clock_icon,
    },
];

/// 拠点 1 件分のダミーデータ（架空、実在の企業・住所とは無関係。
/// 郵便番号はすべて `000-000X` の架空値）。
struct Office {
    name: &'static str,
    line1: &'static str,
    line2: &'static str,
}

/// 拠点一覧（架空、4 件。主参照 R0854 の 2 段目の 2×2 カードに対応）。
const OFFICES: [Office; 4] = [
    Office {
        name: "東京本社",
        line1: "〒000-0006 東京都架空区架空 2-2-2",
        line2: "受付: 平日 9:00〜18:00",
    },
    Office {
        name: "大阪支社",
        line1: "〒000-0007 大阪府架空市架空 3-3-3",
        line2: "受付: 平日 9:00〜17:30",
    },
    Office {
        name: "名古屋支社",
        line1: "〒000-0008 愛知県架空市架空 4-4-4",
        line2: "受付: 平日 9:00〜17:30",
    },
    Office {
        name: "福岡支社",
        line1: "〒000-0009 福岡県架空市架空 5-5-5",
        line2: "受付: 平日 9:00〜17:00",
    },
];

/// SNS 風リンク 1 件分のダミーデータ（架空、実在プラットフォーム名・ロゴは
/// 使わない汎用ラベル + 自作幾何アイコンの組）。
struct SocialLink {
    label: &'static str,
    icon_fn: fn() -> Node,
}

/// SNS 風リンク一覧（架空、4 件。集約元 R0442 の左見出し下 SNS 列に対応）。
const SOCIAL_LINKS: [SocialLink; 4] = [
    SocialLink {
        label: "公式ブログ",
        icon_fn: doc_icon,
    },
    SocialLink {
        label: "コミュニティ",
        icon_fn: cluster_icon,
    },
    SocialLink {
        label: "ニュースレター",
        icon_fn: paperplane_icon,
    },
    SocialLink {
        label: "開発者フォーラム",
        icon_fn: bubbles_icon,
    },
];

/// 連絡先窓口カード 1 件（淡色カード + アイコン + 項目名 + 値）。
/// リンクを持たないプレーンな情報表示（モジュール doc「リンク先を固定
/// リポジトリ URL にする」節）。
fn info_card(item: &ContactItem) -> Node {
    card::root(
        CardVariant::Subtle,
        vec![],
        vec![card::body(
            vec![("data-blocks-contact-split-info-card", "")],
            vec![
                div(
                    vec![("data-blocks-contact-split-info-card-icon", "")],
                    vec![(item.icon_fn)()],
                ),
                heading(
                    HeadingLevel::H4,
                    &HeadingProps {
                        size: HeadingSize::Md,
                        weight: HeadingWeight::Semibold,
                    },
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
                    vec![text(item.value)],
                ),
            ],
        )],
    )
}

/// 拠点カード 1 件（淡色カード + 拠点名 + 住所 2 行 + リンク）。
fn office_card(office: &Office) -> Node {
    let aria_label = format!("GitHub で見る（{}）", office.name);
    card::root(
        CardVariant::Subtle,
        vec![],
        vec![card::body(
            vec![("data-blocks-contact-split-info-card", "")],
            vec![
                heading(
                    HeadingLevel::H4,
                    &HeadingProps {
                        size: HeadingSize::Md,
                        weight: HeadingWeight::Semibold,
                    },
                    vec![],
                    vec![text(office.name)],
                ),
                div(
                    vec![("class", "blocks-contact-split-info-address")],
                    vec![
                        styled_text::text(
                            &TextProps {
                                size: TextSize::Sm,
                                variant: TextVariant::Muted,
                                ..TextProps::default()
                            },
                            vec![],
                            vec![text(office.line1)],
                        ),
                        styled_text::text(
                            &TextProps {
                                size: TextSize::Sm,
                                variant: TextVariant::Muted,
                                ..TextProps::default()
                            },
                            vec![],
                            vec![text(office.line2)],
                        ),
                    ],
                ),
                link::root(
                    REPO,
                    &LinkProps {
                        variant: LinkVariant::Underline,
                        palette: ColorPalette::Neutral,
                        ..LinkProps::default()
                    },
                    vec![
                        ("aria-label", aria_label.as_str()),
                        ("data-blocks-contact-split-info-link", ""),
                    ],
                    vec![text("GitHub で見る")],
                ),
            ],
        )],
    )
}

/// SNS 風リンク 1 件（アイコン + 汎用ラベル + 遷移先の明示）。可視テキスト
/// が 4 件とも相異なるため `aria-label` は付与しない（モジュール doc参照）。
fn social_link(link: &SocialLink) -> Node {
    link::root(
        REPO,
        &LinkProps {
            variant: LinkVariant::Underline,
            palette: ColorPalette::Neutral,
            ..LinkProps::default()
        },
        vec![("data-blocks-contact-split-info-social-link", "")],
        vec![(link.icon_fn)(), text(format!("{}（GitHub）", link.label))],
    )
}

/// 「左見出し + 右情報」の行 1 件を組み立てる。`tagline`/`social` は
/// 任意（1 行目のみ実演、モジュール doc「2 行構成」節）。
fn split_row(
    tagline: Option<&'static str>,
    heading_text: &'static str,
    description: &'static str,
    social: Option<Vec<Node>>,
    right: Node,
) -> Node {
    let mut left_children: Vec<Node> = vec![];
    if let Some(tagline) = tagline {
        left_children.push(styled_text::text(
            &TextProps {
                size: TextSize::Sm,
                weight: TextWeight::Medium,
                ..TextProps::default()
            },
            vec![("data-blocks-contact-split-info-tagline", "")],
            vec![text(tagline)],
        ));
    }
    left_children.push(heading(
        HeadingLevel::H3,
        &HeadingProps {
            size: HeadingSize::Xl2,
            weight: HeadingWeight::Bold,
        },
        vec![],
        vec![text(heading_text)],
    ));
    left_children.push(styled_text::text(
        &TextProps {
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![],
        vec![text(description)],
    ));
    if let Some(social) = social {
        left_children.push(div(
            vec![("class", "blocks-contact-split-info-social")],
            social,
        ));
    }
    let left = div(
        vec![("class", "blocks-contact-split-info-left")],
        left_children,
    );

    div(
        vec![("data-blocks-contact-split-info-row", "")],
        vec![left, right],
    )
}

/// `contact-split-info` の Demo 本体（「連絡先」行 + 「拠点」行の 2 行、
/// それぞれ左見出し + 右側 2×2 カード列）。呼び出しごとに同一の `Node` を
/// 返す純関数。
pub fn demo() -> Node {
    let social_links: Vec<Node> = SOCIAL_LINKS.iter().map(social_link).collect();

    let contact_grid = div(
        vec![("class", "blocks-contact-split-info-grid")],
        CONTACT_ITEMS.iter().map(info_card).collect(),
    );
    let contact_row = split_row(
        Some("お問い合わせ"),
        "ご相談・ご質問はこちらから",
        "各種お問い合わせは下記の窓口までご連絡ください。オンラインでの発信は以下のチャンネルもご利用いただけます。",
        Some(social_links),
        contact_grid,
    );

    let offices_grid = div(
        vec![("class", "blocks-contact-split-info-grid")],
        OFFICES.iter().map(office_card).collect(),
    );
    let offices_row = split_row(
        None,
        "拠点一覧",
        "各拠点の所在地と受付時間の目安です。",
        None,
        offices_grid,
    );

    div(
        vec![("class", "blocks-contact-split-info-layout")],
        vec![contact_row, offices_row],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/contact-split-info/",
    title: "contact-split-info",
    category: BlockCategory::Contact,
    rust_source: "crates/docs-site/src/blocks/marketing/contact/contact_split_info.rs",
    demo_class: "blocks-contact-split-info",
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
            label: "Card",
            path: "/themes/card/",
        },
        Part {
            label: "Icon",
            path: "/themes/icon/",
        },
        Part {
            label: "Link",
            path: "/themes/link/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `contact_split_info` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節と同型で `pub(super)` ではなく本ファイル
/// 内 private 定数として `super::stylesheet` 経由の `push_css` で連結
/// される）。
///
/// セレクタは `.blocks-contact-split-info-*` と
/// `[data-blocks-contact-split-info-*]` のみを用い、他 block や部品の
/// 素のセレクタへ影響させない（`contact_info_columns` と同じ名前空間
/// 分離）。
///
/// # ルート class を `demo_class` と別名にする理由
///
/// [`Block::demo_class`] は `blocks-contact-split-info` だが、`demo()` が
/// 返すルート `div` の class は `blocks-contact-split-info-layout` という
/// 別名にする（`contact_info_columns` と同じ Bugbot 教訓の回避。ページ側が
/// `demo_class` を `.blocks-demo` の隣に付与するラッパーと block 自身の
/// レイアウトルートを区別するため）。
const LAYOUT_CSS: &str = "\
.blocks-contact-split-info-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-12);\n}\n\
[data-blocks-contact-split-info-row] {\n  display: grid;\n  grid-template-columns: 1fr;\n  gap: var(--fandhe-space-6);\n}\n\
.blocks-contact-split-info-left {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n  max-width: 28rem;\n}\n\
[data-blocks-contact-split-info-tagline] {\n  color: var(--fandhe-color-accent);\n  text-transform: uppercase;\n  letter-spacing: 0.05em;\n}\n\
.blocks-contact-split-info-social {\n  display: flex;\n  flex-wrap: wrap;\n  gap: var(--fandhe-space-4);\n  margin-top: var(--fandhe-space-2);\n}\n\
[data-blocks-contact-split-info-social-link] {\n  display: inline-flex;\n  align-items: center;\n  gap: var(--fandhe-space-1);\n}\n\
.blocks-contact-split-info-grid {\n  display: grid;\n  grid-template-columns: 1fr;\n  gap: var(--fandhe-space-4);\n}\n\
[data-blocks-contact-split-info-card] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n}\n\
[data-blocks-contact-split-info-card-icon] {\n  color: var(--fandhe-color-accent);\n}\n\
.blocks-contact-split-info-address {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1);\n}\n\
[data-blocks-contact-split-info-link] {\n  display: inline-flex;\n  align-items: center;\n  gap: var(--fandhe-space-1);\n}\n\
@media (min-width: 40rem) {\n  .blocks-contact-split-info-grid {\n    grid-template-columns: repeat(2, minmax(0, 1fr));\n  }\n}\n\
@media (min-width: 64rem) {\n  [data-blocks-contact-split-info-row] {\n    grid-template-columns: minmax(0, 1fr) minmax(0, 2fr);\n    align-items: start;\n  }\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS, REPO};
    use fandhe_frontend_core::render;

    /// Demo が使用部品（heading/text/card/icon/link）の anatomy をすべて
    /// 実際に出力していることと、行数・カード数・リンク先を固定する
    /// （`contact_info_columns_composes_expected_parts` と同型）。
    #[test]
    fn demo_composes_expected_parts() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"card\"",
            "data-scope=\"icon\"",
            "data-scope=\"link\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert_eq!(
            html.matches("data-blocks-contact-split-info-row").count(),
            2,
            "demo should render exactly 2 rows (contact / offices)"
        );
        assert_eq!(
            html.matches("data-blocks-contact-split-info-card=\"\"")
                .count(),
            8,
            "demo should render exactly 8 cards (4 contact items + 4 offices)"
        );
        assert_eq!(
            html.matches(&format!("href=\"{REPO}\"")).count(),
            8,
            "4 office links + 4 social links should point to the fixed repository URL"
        );
        assert!(html.contains("GitHub で見る（東京本社）"));
        assert!(html.contains("公式ブログ（GitHub）"));
    }

    /// 非対話・XSS 回帰の不変条件（`crate::blocks` モジュール doc）を固定
    /// する。
    #[test]
    fn demo_has_no_form_or_unsafe_output() {
        let html = render(&demo());
        for absent in [
            "<form",
            "type=\"submit\"",
            "href=\"#\"",
            "src=\"data:",
            "id=\"",
            "mailto:",
            "tel:",
        ] {
            assert!(!html.contains(absent), "demo should never contain {absent}");
        }
    }

    /// [`LAYOUT_CSS`] が想定するブレークポイント・列数を持ち、`<` を含まない
    /// こと（REQ-1: `</style>` によるスタイル脱出を防ぐ）。
    #[test]
    fn layout_css_declares_breakpoints_and_column_counts() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("@media (min-width: 40rem)"));
        assert!(LAYOUT_CSS.contains("@media (min-width: 64rem)"));
        assert!(LAYOUT_CSS.contains("repeat(2"));
    }

    /// ルート class（`demo_class` とは別名）が `demo()` の出力へ実際に
    /// 現れること（モジュール doc「ルート class を `demo_class` と別名に
    /// する理由」節の固定、`contact_info_columns` と同じ回帰）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-contact-split-info-layout\""));
        assert_ne!(super::BLOCK.demo_class, "blocks-contact-split-info-layout");
    }
}
