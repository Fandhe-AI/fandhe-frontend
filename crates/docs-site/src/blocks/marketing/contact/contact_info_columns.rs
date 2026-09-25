//! `contact-info-columns` block（イシュー #2830。親トラッキング #2807
//! 「Blocks マーケティング B」配下、対応表 ID R0857（主参照）・R0856
//! （集約元）の 2 件を構造の参照元とする合成例。連絡先カラム一覧）。
//! 取得手段・ファイル名・内部コンポーネント識別子は記載しない
//! （`docs/design/motion-reference-adoption-policy.md` §9 と同じライセンス
//! 上の転記制限）。
//!
//! # 使用部品
//!
//! `heading` / `text` / `icon` / `link` の 4 部品のみを合成する（[`BLOCK`]
//! の `parts` に一致させる契約、`crates/docs-site/tests/blocks_nav.rs`/
//! `blocks_contract.rs` が検証する）。新しい UI 部品は追加しない。
//!
//! # 2 段構成（集約元との差分）
//!
//! 主参照（R0857）は角丸アイコンタイル + 見出し + 説明 + 矢印リンクの
//! 3 列、集約元（R0856）は左罫線付きの拠点住所 4 列という異なる見た目を
//! 持つ。要件（上部に見出しと説明、その下に窓口/拠点をカラムで並べる）を
//! 満たしつつ両方の構造を 1 つの Demo に収めるため、共通のヘッダー 1 つの
//! 下へ「お問い合わせ窓口」（角丸アイコンタイル 3 列、[`CHANNELS`]）・
//! 「拠点一覧」（左罫線 4 列、[`OFFICES`]）の 2 段を並べる。ヘッダーを
//! 1 つに留めるのは見出しの重複を避けるためで、各段の小見出しは
//! `heading` ではなく `text`（`Sm`/`Medium`）で表す（見出しレベルの
//! 節参照）。
//!
//! # 見出しレベル
//!
//! ページ側が `## Demo` として `h2` を出すため、セクション見出しは
//! `HeadingLevel::H3`、各カラムの窓口名/拠点名は `HeadingLevel::H4` にする
//! （`careers_card_grid`/`blog_list_image` と同じ判断）。段の小見出し
//! （「お問い合わせ窓口」「拠点一覧」）は `heading` にすると見出しレベルが
//! 揺れるため、意図的に `text` へ畳む。
//!
//! # 表示バリアントの表現方法
//!
//! カラムの見た目差（角丸アイコンタイル / 左罫線）は `pre-styled-ui`・
//! 本クレートのいずれにも新しい列挙型を増やさず、block ローカルの
//! `data-blocks-contact-info-columns-variant="tile"|"border"` 属性の値のみ
//! で表す（`accordion`/`progress` 等のワイヤーフレーム部品が部品ローカル
//! 列挙型で表示バリアントを表すのと同じ設計思想を、既存 UI 部品の合成例
//! である Blocks へ適用したもの）。
//!
//! # ブレークポイント（sm=40rem/lg=64rem をリテラル直書きする理由）
//!
//! テーマの breakpoint トークンは `@media` 条件式の中では解決できない
//! （CSS custom property は宣言側でのみ有効）ため、
//! `fandhe_frontend_pre_styled_ui::recipe::Breakpoint` の `Sm`（640px =
//! 40rem）・`Lg`（1024px = 64rem）と一致するリテラル値を [`LAYOUT_CSS`] へ
//! 直書きする（`careers_card_grid` の `Md` 直書きと同じ判断）。狭幅は 1 列、
//! `sm` 以上で 2 列、`lg` 以上で「お問い合わせ窓口」3 列・「拠点一覧」4 列
//! へ切り替える。
//!
//! # `text` の名前衝突
//!
//! `fandhe_frontend_pre_styled_ui::text::text`（`<p>` を組み立てる styled
//! パート関数）と `fandhe_frontend_core::text`（テキストノード生成関数）が
//! 同名のため、styled 側を `styled_text` として取り込む（`crate::blocks`
//! 内の他 block と同じ回避方法）。
//!
//! # 自作幾何アイコン（吹き出し・サポート・封筒・右矢印）
//!
//! lucide 等の既存アイコンセットの path を複製しないため、
//! `careers_card_grid::geo_icon` と同型の自作ヘルパ [`geo_icon`] で描く。
//! `path` へ `fill="none"` + `stroke="currentColor"` を明示し、
//! `icon::icon` の `<svg>` 側が固定で持つ `fill="currentColor"`（塗り面）を
//! 上書きして線画として描画する。いずれも隣に可視テキストがあるため装飾
//! 扱い（`IconProps::label` は `None` のまま、`aria-hidden="true"`）とする。
//!
//! # 同名リンクが複数並ぶ問題への対処
//!
//! 各カラムのリンクへ「（可視テキスト）（窓口名/拠点名）」形式の
//! `aria-label` を付与して区別する（可視テキストをアクセシブルネームに
//! 含める形なので WCAG 2.5.3 label-in-name に適合、`careers_card_grid` と
//! 同じ手段）。`id`/`aria-describedby` は出力しない（宙に浮いた ARIA
//! 参照・id 重複を構造的に避けるための既存 block と同じ判断）。
//!
//! # リンク先を固定リポジトリ URL にする・mailto/tel を使わない理由
//!
//! `crate::blocks` の他 block と同じく `link::root` + 固定 URL（[`REPO`]）
//! で「実際に押せる」導線を表す。`fandhe_frontend_core::is_safe_url` は
//! `mailto:`/`tel:` を許可するが、本クレートの block に前例がなく、
//! 架空とはいえ連絡先情報をリンク化しないことで安全側に倒す（連絡先の
//! 提示自体は `text` によるプレーンテキストで行う）。`href="#"` も使わない。
//!
//! # `<form>` を持たない・データ取得/送信を行わない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である。文言・窓口名・拠点住所はすべて架空のもの（実企業名・実クレデン
//! シャル・PII を含まない。住所の郵便番号はすべて `000-000X` の架空値）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

use fandhe_frontend_core::{div, el, text, Node};
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
/// 複製しないための単純図形、`careers_card_grid::geo_icon` と同型）。
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

/// 吹き出しの幾何アイコン（相談窓口）。
fn bubble_icon() -> Node {
    geo_icon("M4 5h16v9H10l-4 4v-4H4V5z")
}

/// 円 + 疑問符の幾何アイコン（サポート窓口）。
fn support_icon() -> Node {
    geo_icon(
        "M12 3a9 9 0 100 18 9 9 0 000-18z M9.5 9a2.5 2.5 0 015 0c0 1.5-1.5 2-2.5 3 M12 16.5v.01",
    )
}

/// 封筒の幾何アイコン（取材・提携窓口）。
fn envelope_icon() -> Node {
    geo_icon("M4 6h16v12H4V6z M4 7l8 6 8-6")
}

/// 右矢印の幾何アイコン（各リンク末尾）。
fn arrow_icon() -> Node {
    geo_icon("M4 12h16 M13 5l7 7-7 7")
}

/// 窓口 1 件分のダミーデータ（架空、実在の企業・人物とは無関係）。
struct Channel {
    title: &'static str,
    description: &'static str,
    link_label: &'static str,
    icon_fn: fn() -> Node,
}

/// 窓口一覧（架空、3 件。主参照 R0857 の角丸アイコンタイル 3 列に対応）。
const CHANNELS: [Channel; 3] = [
    Channel {
        title: "導入のご相談",
        description: "導入検討中のお客様からのご相談を承ります。",
        link_label: "お問い合わせはこちら",
        icon_fn: bubble_icon,
    },
    Channel {
        title: "技術サポート",
        description: "導入後の技術的なお問い合わせに対応します。",
        link_label: "サポート窓口へ",
        icon_fn: support_icon,
    },
    Channel {
        title: "取材・提携のご相談",
        description: "取材・協業に関するご相談を受け付けています。",
        link_label: "お問い合わせはこちら",
        icon_fn: envelope_icon,
    },
];

/// 拠点 1 件分のダミーデータ（架空、実在の企業・住所とは無関係。
/// 郵便番号はすべて `000-000X` の架空値）。
struct Office {
    name: &'static str,
    line1: &'static str,
    line2: &'static str,
}

/// 拠点一覧（架空、4 件。集約元 R0856 の左罫線付き拠点住所 4 列に対応）。
const OFFICES: [Office; 4] = [
    Office {
        name: "東京オフィス",
        line1: "〒000-0001 東京都千代田区架空 1-2-3",
        line2: "本社機能・営業窓口",
    },
    Office {
        name: "大阪オフィス",
        line1: "〒000-0002 大阪府大阪市架空 4-5-6",
        line2: "西日本エリア担当",
    },
    Office {
        name: "福岡オフィス",
        line1: "〒000-0003 福岡県福岡市架空 7-8-9",
        line2: "九州エリア担当",
    },
    Office {
        name: "札幌オフィス",
        line1: "〒000-0004 北海道札幌市架空 10-11",
        line2: "北日本エリア担当",
    },
];

/// 窓口カラム 1 件（角丸アイコンタイル + 窓口名 + 説明 + 矢印付きリンク）。
fn channel_column(channel: &Channel) -> Node {
    let aria_label = format!("{}（{}）", channel.link_label, channel.title);
    div(
        vec![
            ("data-blocks-contact-info-columns-column", ""),
            ("data-blocks-contact-info-columns-variant", "tile"),
        ],
        vec![
            div(
                vec![("data-blocks-contact-info-columns-tile", "")],
                vec![(channel.icon_fn)()],
            ),
            heading(
                HeadingLevel::H4,
                &HeadingProps {
                    size: HeadingSize::Md,
                    weight: HeadingWeight::Semibold,
                },
                vec![],
                vec![text(channel.title)],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(channel.description)],
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
                    ("data-blocks-contact-info-columns-link", ""),
                ],
                vec![text(channel.link_label), arrow_icon()],
            ),
        ],
    )
}

/// 拠点カラム 1 件（左罫線 + 拠点名 + 住所 2 行 + 矢印付きリンク）。
fn office_column(office: &Office) -> Node {
    let aria_label = format!("地図を見る（{}）", office.name);
    div(
        vec![
            ("data-blocks-contact-info-columns-column", ""),
            ("data-blocks-contact-info-columns-variant", "border"),
        ],
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
                vec![("class", "blocks-contact-info-columns-address")],
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
                    ("data-blocks-contact-info-columns-link", ""),
                ],
                vec![text("地図を見る"), arrow_icon()],
            ),
        ],
    )
}

/// `contact-info-columns` の Demo 本体（見出し + 窓口カラム 3 列 + 拠点
/// カラム 4 列）。呼び出しごとに同一の `Node` を返す純関数
/// （モジュール doc「2 段構成」節）。
pub fn demo() -> Node {
    let header = div(
        vec![("class", "blocks-contact-info-columns-header")],
        vec![
            styled_text::text(
                &TextProps {
                    size: TextSize::Sm,
                    weight: TextWeight::Medium,
                    ..TextProps::default()
                },
                vec![("data-blocks-contact-info-columns-tagline", "")],
                vec![text("お問い合わせ")],
            ),
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    weight: HeadingWeight::Bold,
                },
                vec![],
                vec![text("用件に合わせた窓口をお選びください")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(
                    "各種お問い合わせは以下の窓口から、拠点へのご訪問は下記の一覧からご確認ください。",
                )],
            ),
        ],
    );

    let channels_label = styled_text::text(
        &TextProps {
            size: TextSize::Sm,
            weight: TextWeight::Medium,
            ..TextProps::default()
        },
        vec![("data-blocks-contact-info-columns-section-label", "")],
        vec![text("お問い合わせ窓口")],
    );
    let channels = div(
        vec![("class", "blocks-contact-info-columns-channels")],
        CHANNELS.iter().map(channel_column).collect(),
    );
    let channels_section = div(
        vec![("class", "blocks-contact-info-columns-section")],
        vec![channels_label, channels],
    );

    let offices_label = styled_text::text(
        &TextProps {
            size: TextSize::Sm,
            weight: TextWeight::Medium,
            ..TextProps::default()
        },
        vec![("data-blocks-contact-info-columns-section-label", "")],
        vec![text("拠点一覧")],
    );
    let offices = div(
        vec![("class", "blocks-contact-info-columns-offices")],
        OFFICES.iter().map(office_column).collect(),
    );
    let offices_section = div(
        vec![("class", "blocks-contact-info-columns-section")],
        vec![offices_label, offices],
    );

    div(
        vec![("class", "blocks-contact-info-columns-layout")],
        vec![header, channels_section, offices_section],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/contact-info-columns/",
    title: "contact-info-columns",
    category: BlockCategory::Contact,
    rust_source: "crates/docs-site/src/blocks/marketing/contact/contact_info_columns.rs",
    demo_class: "blocks-contact-info-columns",
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
            label: "Link",
            path: "/themes/link/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `contact_info_columns` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節、`careers_card_grid` と同型で
/// `pub(super)` ではなく本ファイル内 private 定数として
/// `super::stylesheet` 経由の `push_css` で連結される）。
///
/// セレクタは `.blocks-contact-info-columns-*` と
/// `[data-blocks-contact-info-columns-*]` のみを用い、他 block や部品の
/// 素のセレクタへ影響させない（`careers_card_grid` と同じ名前空間分離）。
///
/// # ルート class を `demo_class` と別名にする理由
///
/// [`Block::demo_class`] は `blocks-contact-info-columns` だが、
/// `demo()` が返すルート `div` の class は `blocks-contact-info-columns-layout`
/// という別名にする（`careers_card_grid`/`blog_list_image` と同じ
/// Bugbot 教訓の回避。ページ側が `demo_class` を `.blocks-demo` の隣に
/// 付与するラッパーと block 自身のレイアウトルートを区別するため）。
const LAYOUT_CSS: &str = "\
.blocks-contact-info-columns-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-10);\n}\n\
.blocks-contact-info-columns-header {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n  max-width: 42rem;\n}\n\
[data-blocks-contact-info-columns-tagline] {\n  color: var(--fandhe-color-accent);\n  text-transform: uppercase;\n  letter-spacing: 0.05em;\n}\n\
[data-blocks-contact-info-columns-section-label] {\n  color: var(--fandhe-color-accent);\n  text-transform: uppercase;\n  letter-spacing: 0.05em;\n}\n\
.blocks-contact-info-columns-section {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-contact-info-columns-channels {\n  display: grid;\n  grid-template-columns: 1fr;\n  gap: var(--fandhe-space-8);\n}\n\
.blocks-contact-info-columns-offices {\n  display: grid;\n  grid-template-columns: 1fr;\n  gap: var(--fandhe-space-8);\n}\n\
[data-blocks-contact-info-columns-column] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n}\n\
[data-blocks-contact-info-columns-tile] {\n  display: inline-flex;\n  align-items: center;\n  justify-content: center;\n  width: var(--fandhe-space-10);\n  height: var(--fandhe-space-10);\n  border-radius: var(--fandhe-radius-lg);\n  background: var(--fandhe-color-bg-subtle);\n  color: var(--fandhe-color-accent);\n}\n\
[data-blocks-contact-info-columns-variant=\"border\"] {\n  border-left: 1px solid var(--fandhe-color-border);\n  padding-left: var(--fandhe-space-4);\n}\n\
.blocks-contact-info-columns-address {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1);\n}\n\
[data-blocks-contact-info-columns-link] {\n  display: inline-flex;\n  align-items: center;\n  gap: var(--fandhe-space-1);\n}\n\
@media (min-width: 40rem) {\n  .blocks-contact-info-columns-channels {\n    grid-template-columns: repeat(2, minmax(0, 1fr));\n  }\n  .blocks-contact-info-columns-offices {\n    grid-template-columns: repeat(2, minmax(0, 1fr));\n  }\n}\n\
@media (min-width: 64rem) {\n  .blocks-contact-info-columns-channels {\n    grid-template-columns: repeat(3, minmax(0, 1fr));\n  }\n  .blocks-contact-info-columns-offices {\n    grid-template-columns: repeat(4, minmax(0, 1fr));\n  }\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS, REPO};
    use fandhe_frontend_core::render;

    /// Demo が使用部品（heading/text/icon/link）の anatomy をすべて実際に
    /// 出力していることと、カラム数・variant の内訳・リンク先を固定する
    /// （`careers_card_grid_demo_composes_expected_parts` と同型）。
    #[test]
    fn demo_composes_expected_parts() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"icon\"",
            "data-scope=\"link\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert_eq!(
            html.matches("data-blocks-contact-info-columns-variant=\"tile\"")
                .count(),
            3,
            "demo should render exactly 3 tile-variant columns"
        );
        assert_eq!(
            html.matches("data-blocks-contact-info-columns-variant=\"border\"")
                .count(),
            4,
            "demo should render exactly 4 border-variant columns"
        );
        assert_eq!(
            html.matches(&format!("href=\"{REPO}\"")).count(),
            7,
            "each channel/office column should link to the fixed repository URL"
        );
        assert!(html.contains("お問い合わせはこちら（導入のご相談）"));
        assert!(html.contains("地図を見る（東京オフィス）"));
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
        assert!(LAYOUT_CSS.contains("repeat(3"));
        assert!(LAYOUT_CSS.contains("repeat(4"));
    }

    /// ルート grid class（`demo_class` とは別名）が `demo()` の出力へ
    /// 実際に現れること（モジュール doc「ルート class を `demo_class` と
    /// 別名にする理由」節の固定、`careers_card_grid` と同じ回帰）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-contact-info-columns-layout\""));
        assert_ne!(
            super::BLOCK.demo_class,
            "blocks-contact-info-columns-layout"
        );
    }
}
