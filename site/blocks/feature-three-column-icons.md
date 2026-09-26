# feature-three-column-icons

`heading` / `text` / `icon` / `link` / `card` の 5 部品を合成した、見出しの
下へアイコン・題名・説明を持つ feature を 3 列で並べる定番の feature
セクションです。

4 つの静的インスタンスを縦に並べています。1 つ目は中央寄せ見出し + CTA
リンクの下へ、小アイコン付きの項目 3 件を並べ各項目末尾に詳細リンクを
置く基準形です。2 つ目は左見出し・右リード文の 2 列見出し行の下へ、同じ
3 件を影付きカード（Elevated）へ収めて強調します。3 つ目は左寄せ見出しの
下へ、大アイコン付きの項目 6 件を 3 列 2 段で並べます（詳細リンクなし）。
4 つ目は中央見出しの下へ、アイコンを省いた項目 6 件を並べます。いずれも
幅 md（48rem）未満は 1 列です。詳細リンクの文言は項目ごとに固定の文言に
しており、同じ「詳しく見る」を並べません（WCAG 2.4.4/2.5.3 対応）。
文言・データはすべて架空のもので、データ取得・送信は行わない静的な表示例
です。`<form>` は使用しません。

## Rust コード

```rust
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

/// 項目 1 件分の架空データ（見出し・説明・自作アイコンのパス・詳細リンク
/// の accessible name）。
struct Feature {
    title: &'static str,
    body: &'static str,
    link_label: &'static str,
    icon_path_d: &'static str,
}

/// 項目 6 件（架空、実在の企業・製品とは無関係）。自作の単純な幾何パス
/// のみを使い、lucide 等の著作物は複製しない。先頭 3 件を A/B インスタンス
/// で使い回す。
const FEATURES: [Feature; 6] = [
    Feature {
        title: "決定的なビルド",
        body: "同じ入力からは常に同じ静的ファイルを生成します。",
        link_label: "決定的なビルドの詳細",
        icon_path_d: "M12 3l9 6-9 6-9-6z",
    },
    Feature {
        title: "型で表す構造",
        body: "スロットと props は Rust の型で表現されます。",
        link_label: "型で表す構造の詳細",
        icon_path_d: "M4 4h16v16H4z",
    },
    Feature {
        title: "既定エスケープ",
        body: "テキスト補間は既定でエスケープされます。",
        link_label: "既定エスケープの詳細",
        icon_path_d: "M12 21a9 9 0 100-18 9 9 0 000 18z",
    },
    Feature {
        title: "外部依存ゼロの描画コア",
        body: "描画コアは外部クレートに依存しません。",
        link_label: "外部依存ゼロの詳細",
        icon_path_d: "M12 2v8M8 6l4-4 4 4M4 14h16v8H4z",
    },
    Feature {
        title: "依存グラフ上限の管理",
        body: "依存パッケージ数・深さの上限を CI で機械検証します。",
        link_label: "依存グラフ上限の詳細",
        icon_path_d: "M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5",
    },
    Feature {
        title: "単一実行ファイル配布",
        body: "サーバーを 1 つのバイナリとして配布できます。",
        link_label: "単一実行ファイル配布の詳細",
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

/// 項目ごとの詳細リンク（accessible name は `link_label` そのまま、
/// モジュール doc「詳細リンクの accessible name」節参照）。
fn detail_link(f: &Feature) -> Node {
    link::root(
        REPO,
        &LinkProps {
            external: true,
            ..LinkProps::default()
        },
        vec![("data-blocks-feature-three-column-icons-link", "")],
        vec![text(f.link_label), arrow_icon()],
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
```

## 原案差分メモ

参照（対応表 ID R0935・R0073・R0074・R0472・R0996・R0099・R0100・R0937・
R0944・R0946。出典の固有名・ファイル名は記載しません）からの意図的な
差分は次のとおりです。

- R0935 を基準形（中央寄せ見出し + アイコン付き 3 列 feature）とし、
  R0099 の CTA リンクを見出し下へ添えて 1 つ目のインスタンスにしました。
- R0073 の「項目末尾に詳細リンク」を、1 つ目のインスタンスの各項目へ
  適用しました。詳細リンクの accessible name は項目ごとに固定の文言に
  し、`format!` によるラベル合成は行っていません。
- R0074・R0472・R0996 の「影付きカードに収める形」を、R0100 の 2 列
  見出し行と組み合わせて 2 つ目のインスタンスにしました。R0996 の背景
  画像装飾は使用部品に含まれないため持ち込みません。
- R0937 の「左寄せ + 大アイコン」と R0946 の「左寄せ + 6 件」を統合し、
  3 つ目のインスタンスにしました。詳細リンクは持たせません。
- R0944 の「アイコンなしの 6 件」を、中央寄せ見出しの下へ 4 つ目の
  インスタンスにしました。
- 4 インスタンスとも同じ 6 件の項目データ（先頭 3 件を A/B で使い回し）
  を使い、差分を配置・アイコン有無・カード有無のみで示しました。
- 見出しは `h3` に、項目題名はそれより 1 段下げて `h4` にしました（ページ
  側が `## Demo` として `h2` を出すため）。
- アイコンは自作の単純な幾何パスのみを使い、lucide 等の著作物は複製して
  いません。
