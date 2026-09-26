//! `feature-three-column-icons` block（イシュー #3225。親トラッキングは
//! #2738「Blocks マーケティング A」→ #2730）。見出しの下へアイコン・題名・
//! 説明を持つ feature を 3 列で並べる定番の feature セクションの合成例
//! （取得手段・ファイル名・内部コンポーネント識別子は記載しない。対応表
//! ID は原案差分メモにのみ記す）。
//!
//! # 使用部品
//!
//! `heading` / `text` / `icon` / `link` / `card` の 5 部品のみを合成する
//! （[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//! 新規 UI 部品・外部依存は追加しない。
//!
//! # 4 インスタンスへの統合
//!
//! 集約元の差分を 4 インスタンスへ統合し、Demo に並べて示す（詳細は原稿
//! 側「原案差分メモ」節）。
//!
//! - **A**: 中央見出し + CTA。3 件・小アイコン・詳細リンクあり。カードなし
//! - **B**: 2 列の見出し行（左見出し・右リード文）。3 件を影付きカード
//!   （`CardVariant::Elevated`）へ収め、各カードに詳細リンクを置く
//! - **C**: 左寄せ見出し。6 件を大アイコンで並べる。詳細リンクなし
//! - **D**: 中央見出し。6 件をアイコンなしで並べる
//!
//! # 詳細リンクの accessible name（WCAG 2.4.4 / 2.5.3 対応）
//!
//! 全項目の詳細リンクは同一の固定 URL（[`REPO`]、リポジトリのトップ
//! ページ）へ遷移する。項目ごとに異なる説明ページは存在しないため、
//! 「〜の詳細」のように専用ページの存在を示唆するラベルにはしない
//! （実際の遷移先と食い違うため）。代わりに `f.title` を埋め込んだ
//! 「`<title>` を GitHub で見る」を accessible name にする: 見える文字列
//! をそのまま使い（2.5.3 Label in Name）、項目ごとに異なる固定文字列に
//! なるため同じラベルが並ばず読み上げ上も区別できる（2.4.4 Link
//! Purpose）うえ、実遷移先（GitHub リポジトリ）と矛盾しない。
//!
//! # アイコンを装飾扱いにする理由
//!
//! 項目見出し・説明が既に意味を伝えているため、装飾アイコンは
//! [`IconProps::label`] を `None` にする（`feature_side_heading_grid::
//! geo_icon` と同型の判断）。自作の単純な幾何パスのみを使い、lucide 等の
//! 著作物は複製しない。
//!
//! # リンク先の方針
//!
//! `Block::demo` は `fn() -> Node` で `base_path` を受け取れない
//! （`crate::blocks` モジュール doc「`<form>` を使わない」節と同じ制約）。
//! 前例（`contact_image_info`/`changelog_timeline`）と同じく、固定の実在
//! 外部 URL `https://github.com/Fandhe-AI/fandhe-frontend` を使い、
//! `href="#"` の死リンクは使わない。`LinkProps::external: true` により
//! `target="_blank"` + `rel="noopener noreferrer"` は headless 層が付与
//! する（reverse tabnabbing 対策）。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `heading::heading`/`styled_text::text`/`card::root`/`icon::icon`/
//! `link::root` はいずれも `drop_class_attr` により呼び出し側 `attrs` の
//! `class` を黙って除去する契約を持つため、Demo 固有のスタイルフックは
//! `data-blocks-feature-three-column-icons-*` 属性で渡す。素の `div` には
//! `class` がそのまま効くため `.blocks-feature-three-column-icons-*`
//! クラスセレクタを使う。レイアウト root の class
//! （`blocks-feature-three-column-icons-layout`）は [`Block::demo_class`]
//! （`blocks-feature-three-column-icons`）とは意図的に別名にする（先行
//! block で得た Bugbot 教訓の踏襲）。
//!
//! # 見出しレベル
//!
//! ページ側が `## Demo` として `h2` を出すため、各インスタンスの導入
//! 見出しは `HeadingLevel::H3`、項目題名はそれより 1 段下げて
//! `HeadingLevel::H4` にする。
//!
//! # `<form>` を持たない・依存追加なし
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である。文言はすべて架空のもの（実企業名・実クレデンシャル・PII を
//! 含まない）。新規 UI 部品・新規外部クレート依存は追加しない。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps, CardVariant};
use fandhe_frontend_pre_styled_ui::heading::{
    self, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::Size;

/// リンク先の固定外部 URL（モジュール doc「リンク先の方針」節参照）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// 項目 1 件分の架空データ（見出し・説明・自作アイコンのパス）。詳細
/// リンクの accessible name はモジュール doc「詳細リンクの accessible
/// name」節のとおり `title` から合成するため、専用フィールドは持たない。
struct Feature {
    title: &'static str,
    body: &'static str,
    icon_path_d: &'static str,
}

/// 項目 6 件（架空、実在の企業・製品とは無関係）。自作の単純な幾何パス
/// のみを使い、lucide 等の著作物は複製しない。先頭 3 件を A/B インスタンス
/// で使い回す。
const FEATURES: [Feature; 6] = [
    Feature {
        title: "決定的なビルド",
        body: "同じ入力からは常に同じ静的ファイルを生成します。",
        icon_path_d: "M12 3l9 6-9 6-9-6z",
    },
    Feature {
        title: "型で表す構造",
        body: "スロットと props は Rust の型で表現されます。",
        icon_path_d: "M4 4h16v16H4z",
    },
    Feature {
        title: "既定エスケープ",
        body: "テキスト補間は既定でエスケープされます。",
        icon_path_d: "M12 21a9 9 0 100-18 9 9 0 000 18z",
    },
    Feature {
        title: "外部依存ゼロの描画コア",
        body: "描画コアは外部クレートに依存しません。",
        icon_path_d: "M12 2v8M8 6l4-4 4 4M4 14h16v8H4z",
    },
    Feature {
        title: "依存グラフ上限の管理",
        body: "依存パッケージ数・深さの上限を CI で機械検証します。",
        icon_path_d: "M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5",
    },
    Feature {
        title: "単一実行ファイル配布",
        body: "サーバーを 1 つのバイナリとして配布できます。",
        icon_path_d: "M6 3h12v6H6zM6 15h12v6H6zM9 9h6v6H9z",
    },
];

/// 装飾用の自作幾何アイコン（`fill="none"` + `stroke="currentColor"` の
/// 線画、`feature_side_heading_grid::geo_icon` と同型の判断）。
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

/// 詳細リンクへ添える装飾用の小さな矢印アイコン
/// （`feature_four_column_grid::top_right_arrow_icon` と同型の自作パス）。
fn arrow_icon() -> Node {
    icon(
        &IconProps {
            size: Size::Xs,
            ..IconProps::default()
        },
        vec![("data-blocks-feature-three-column-icons-arrow", "")],
        vec![el(
            "path",
            vec![
                ("d", "M5 12h14M13 6l6 6-6 6"),
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

/// 項目ごとの詳細リンク（accessible name は `title` から合成する、
/// モジュール doc「詳細リンクの accessible name」節参照）。
fn detail_link(f: &Feature) -> Node {
    link::root(
        REPO,
        &LinkProps {
            external: true,
            ..LinkProps::default()
        },
        vec![("data-blocks-feature-three-column-icons-link", "")],
        vec![text(format!("{} を GitHub で見る", f.title)), arrow_icon()],
    )
}

/// 装飾アイコンの有無・サイズ（3 値のため bool ではなく enum にする）。
#[derive(Clone, Copy)]
enum IconMode {
    None,
    Small,
    Large,
}

/// 項目 1 件（アイコン → 見出し → 説明 → 任意の詳細リンク）を組み立てる。
fn feature_item(f: &Feature, icon_mode: IconMode, with_link: bool) -> Node {
    let mut children: Vec<Node> = Vec::new();
    match icon_mode {
        IconMode::None => {}
        IconMode::Small => children.push(geo_icon(
            f.icon_path_d,
            Size::Md,
            vec![("data-blocks-feature-three-column-icons-icon", "")],
        )),
        IconMode::Large => children.push(geo_icon(
            f.icon_path_d,
            Size::Xl,
            vec![("data-blocks-feature-three-column-icons-icon", "")],
        )),
    }
    children.push(heading::heading(
        HeadingLevel::H4,
        &HeadingProps::default(),
        vec![],
        vec![text(f.title)],
    ));
    children.push(styled_text::text(
        &TextProps {
            size: TextSize::Sm,
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![("data-blocks-feature-three-column-icons-desc", "")],
        vec![text(f.body)],
    ));
    if with_link {
        children.push(detail_link(f));
    }
    div(
        vec![("class", "blocks-feature-three-column-icons-item")],
        children,
    )
}

/// 影付きカードへ収めた項目（形 B 用、`CardVariant::Elevated`）。
fn feature_card(f: &Feature) -> Node {
    card::root(
        CardProps {
            variant: CardVariant::Elevated,
            ..CardProps::default()
        },
        vec![("data-blocks-feature-three-column-icons-card", "")],
        vec![card::body(
            vec![("data-blocks-feature-three-column-icons-card-body", "")],
            vec![feature_item(f, IconMode::Small, true)],
        )],
    )
}

/// 見出しの寄せ（`-header[data-align="start"]` の CSS フック）。
#[derive(Clone, Copy)]
enum Align {
    Center,
    Start,
}

/// 中央/左寄せの見出し（形 A/C/D 用）。`cta` が `Some` のとき見出し下へ
/// CTA リンクを置く（詳細リンクとは別の CSS フック
/// `-cta` を使い、詳細リンク件数の固定テストと衝突させない）。
fn header(
    title: &'static str,
    lead: &'static str,
    align: Align,
    cta: Option<(&'static str, &'static str)>,
) -> Node {
    let mut attrs = vec![("class", "blocks-feature-three-column-icons-header")];
    if matches!(align, Align::Start) {
        attrs.push(("data-align", "start"));
    }
    let mut children = vec![
        heading::heading(
            HeadingLevel::H3,
            &HeadingProps {
                size: HeadingSize::Xl3,
                weight: HeadingWeight::Bold,
            },
            vec![],
            vec![text(title)],
        ),
        styled_text::text(
            &TextProps {
                variant: TextVariant::Muted,
                ..TextProps::default()
            },
            vec![("data-blocks-feature-three-column-icons-lead", "")],
            vec![text(lead)],
        ),
    ];
    if let Some((href_label, url)) = cta {
        children.push(link::root(
            url,
            &LinkProps {
                external: true,
                ..LinkProps::default()
            },
            vec![("data-blocks-feature-three-column-icons-cta", "")],
            vec![text(href_label)],
        ));
    }
    div(attrs, children)
}

/// 2 列の見出し行（形 B 用。左に見出し、右にリード文。md 以上で横並び、
/// 未満は縦積み）。
fn header_split(title: &'static str, lead: &'static str) -> Node {
    div(
        vec![("class", "blocks-feature-three-column-icons-header-split")],
        vec![
            heading::heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl3,
                    weight: HeadingWeight::Bold,
                },
                vec![],
                vec![text(title)],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![(
                    "data-blocks-feature-three-column-icons-header-split-lead",
                    "",
                )],
                vec![text(lead)],
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
        vec![("data-blocks-feature-three-column-icons-note", "")],
        vec![text(body)],
    )
}

/// 1 インスタンス分（導入部 + 注記 + 3 列グリッド）を組み立てる。
fn instance(header_node: Node, note_body: &'static str, grid_items: Vec<Node>) -> Node {
    div(
        vec![("class", "blocks-feature-three-column-icons-instance")],
        vec![
            header_node,
            note(note_body),
            div(
                vec![("class", "blocks-feature-three-column-icons-grid")],
                grid_items,
            ),
        ],
    )
}

/// `feature-three-column-icons` の Demo 本体。呼び出しごとに同一の `Node`
/// を返す純関数（他 block と同じ状態を持たない設計）。
#[must_use]
pub fn demo() -> Node {
    let instance_a = instance(
        header(
            "3 つの特長で紹介する",
            "アイコンと詳細リンク付きの、もっとも定番の 3 列構成です。",
            Align::Center,
            Some(("すべての特長を見る", REPO)),
        ),
        "中央見出し + CTA。小アイコン + 詳細リンク付きの 3 列。カードなし。",
        FEATURES[..3]
            .iter()
            .map(|f| feature_item(f, IconMode::Small, true))
            .collect(),
    );

    let instance_b = instance(
        header_split(
            "カードで示す特長",
            "同じ 3 件を、影付きカードへ収めて強調する配置例です。",
        ),
        "2 列の見出し行 + 影付きカード（Elevated）3 枚。各カードに詳細リンク。",
        FEATURES[..3].iter().map(feature_card).collect(),
    );

    let instance_c = instance(
        header(
            "左寄せで示す特長",
            "6 つの特長を、大きめのアイコンとともに 2 段で並べます。",
            Align::Start,
            None,
        ),
        "左寄せ見出し。大アイコンの 6 件を 3 列 2 段で並べる。詳細リンクなし。",
        FEATURES
            .iter()
            .map(|f| feature_item(f, IconMode::Large, false))
            .collect(),
    );

    let instance_d = instance(
        header(
            "簡潔に示す特長",
            "アイコンを省き、題名と説明だけで 6 件を並べます。",
            Align::Center,
            None,
        ),
        "中央見出し。アイコンなしの 6 件を 3 列 2 段で並べる。",
        FEATURES
            .iter()
            .map(|f| feature_item(f, IconMode::None, false))
            .collect(),
    );

    div(
        vec![("class", "blocks-feature-three-column-icons-layout")],
        vec![instance_a, instance_b, instance_c, instance_d],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/feature-three-column-icons/",
    title: "feature-three-column-icons",
    category: BlockCategory::Feature,
    rust_source: "crates/docs-site/src/blocks/marketing/feature/feature_three_column_icons.rs",
    demo_class: "blocks-feature-three-column-icons",
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
        Part {
            label: "Card",
            path: "/themes/card/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `feature_three_column_icons` 固有のレイアウト規則（`crate::blocks::
/// LAYOUT_CSS` doc「block 固有 CSS の置き場」節）。セレクタは
/// `.blocks-feature-three-column-icons-*` と `[data-blocks-feature-three-
/// column-icons-*]` のみを用い、他 block や部品の素のセレクタへ影響させ
/// ない（`feature_four_column_grid` と同じ名前空間分離）。ブレークポイント
/// のリテラル 48rem は `recipe::Breakpoint::Md`（768px）と一致させる
/// （テーマの breakpoint トークンは `@media` 条件の中では解決できない
/// ため）。
const LAYOUT_CSS: &str = "\
.blocks-feature-three-column-icons-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-10);\n}\n\
.blocks-feature-three-column-icons-instance {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n}\n\
.blocks-feature-three-column-icons-header {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n  align-items: center;\n  text-align: center;\n  max-width: 40rem;\n  margin-inline: auto;\n}\n\
.blocks-feature-three-column-icons-header[data-align=\"start\"] {\n  align-items: flex-start;\n  text-align: left;\n  margin-inline: 0;\n}\n\
[data-scope=\"text\"][data-part=\"root\"][data-blocks-feature-three-column-icons-lead] {\n  margin: 0;\n}\n\
.blocks-feature-three-column-icons-header-split {\n  display: grid;\n  grid-template-columns: minmax(0, 1fr);\n  gap: var(--fandhe-space-2);\n  align-items: end;\n}\n\
[data-scope=\"text\"][data-part=\"root\"][data-blocks-feature-three-column-icons-header-split-lead] {\n  margin: 0;\n}\n\
[data-scope=\"text\"][data-part=\"root\"][data-blocks-feature-three-column-icons-note] {\n  margin: 0;\n  text-align: center;\n}\n\
.blocks-feature-three-column-icons-grid {\n  display: grid;\n  grid-template-columns: minmax(0, 1fr);\n  gap: var(--fandhe-space-6);\n}\n\
.blocks-feature-three-column-icons-item {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n  min-width: 0;\n  height: 100%;\n}\n\
[data-scope=\"text\"][data-part=\"root\"][data-blocks-feature-three-column-icons-desc] {\n  margin: 0;\n}\n\
[data-scope=\"link\"][data-part=\"root\"][data-blocks-feature-three-column-icons-link] {\n  display: inline-flex;\n  align-items: center;\n  gap: var(--fandhe-space-1);\n  margin-top: auto;\n}\n\
[data-blocks-feature-three-column-icons-card] {\n  height: 100%;\n}\n\
[data-blocks-feature-three-column-icons-card-body] {\n  gap: var(--fandhe-space-3);\n}\n\
@media (min-width: 48rem) {\n  \
.blocks-feature-three-column-icons-grid {\n    grid-template-columns: repeat(3, minmax(0, 1fr));\n  }\n  \
.blocks-feature-three-column-icons-header-split {\n    grid-template-columns: repeat(2, minmax(0, 1fr));\n  }\n\
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
            "data-scope=\"link\"",
            "data-scope=\"card\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        // A(3) + B(3) + C(6) + D(6) = 18 項目。
        assert_eq!(
            html.matches("class=\"blocks-feature-three-column-icons-item\"")
                .count(),
            18
        );
        // B のみ 3 枚の影付きカード。
        assert_eq!(
            html.matches("data-blocks-feature-three-column-icons-card=\"\"")
                .count(),
            3
        );
        // 詳細リンクは A(3) + B(3) = 6 件（CTA・カードなし C/D は含まない）。
        assert_eq!(
            html.matches("data-blocks-feature-three-column-icons-link=\"\"")
                .count(),
            6
        );
        // アイコンは A(3・小) + B(3・小) + C(6・大) = 12 件（D はアイコン
        // なし）。
        assert_eq!(
            html.matches("data-blocks-feature-three-column-icons-icon=\"\"")
                .count(),
            12
        );
        // CTA は A のみ 1 件。
        assert_eq!(
            html.matches("data-blocks-feature-three-column-icons-cta=\"\"")
                .count(),
            1
        );
        assert!(!html.contains("<form"));
        assert!(!html.contains("<button"));
        assert!(!html.contains("type=\"submit\""));
        assert!(!html.contains("id=\""));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("src=\"data:"));
    }

    /// 詳細リンク・CTA のいずれも `external: true` により
    /// `target="_blank"` と `rel="noopener noreferrer"` が付与されること
    /// （reverse tabnabbing 対策の固定）。
    #[test]
    fn links_are_external_with_safe_rel() {
        let html = render(&demo());
        assert_eq!(html.matches("target=\"_blank\"").count(), 7);
        assert_eq!(html.matches("rel=\"noopener noreferrer\"").count(), 7);
    }

    /// [`LAYOUT_CSS`] が想定する md ブレークポイント・3 列切り替えを持つ
    /// こと。
    #[test]
    fn layout_css_declares_md_three_column_grid() {
        assert!(LAYOUT_CSS.contains("@media (min-width: 48rem)"));
        assert!(LAYOUT_CSS.contains("repeat(3, minmax(0, 1fr))"));
        assert!(LAYOUT_CSS.contains("[data-align=\"start\"]"));
    }

    /// ルート class（`demo_class` とは別名）が `demo()` の出力へ実際に
    /// 現れること（`feature_four_column_grid` と同じ Bugbot 教訓の固定）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-feature-three-column-icons-layout\""));
        assert_ne!(
            super::BLOCK.demo_class,
            "blocks-feature-three-column-icons-layout"
        );
    }
}
