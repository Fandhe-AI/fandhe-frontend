# feature-side-heading-grid

`badge` / `heading` / `text` / `icon` の 4 部品だけを合成した feature
セクションです。lg（64rem）以上ではセクション全体を 5 列 grid にし、左の
狭い列（2 列分）へ見出しと説明、右の広い列（3 列分）へ 2 列の feature
グリッドを配置します。lg 未満では見出しの下にグリッドがそのまま続く縦積み
になり、グリッド自体は幅 40rem（sm 相当）以上で 1 列から 2 列へ切り替わ
ります。

「アイコン付き 4 件の 2×2」と「チェック付き 6 件の 2 列」という 2 つの形
を縦に並べて示します。いずれもアイコン・チェックは意味を持たない装飾
（`aria-hidden`）として扱います。画像は使用せず、文言はすべて架空のもの
です。データ取得・送信は行わない静的な表示例で、`<form>` は使用しません。

## Rust コード

```rust
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::Size;

/// 形 A（アイコン付き 4 件）の 1 項目分。
struct IconFeature {
    title: &'static str,
    body: &'static str,
    icon_path_d: &'static str,
}

/// 形 B（チェック付き 6 件）の 1 項目分。
struct CheckFeature {
    title: &'static str,
    body: &'static str,
}

/// 形 A の架空データ 4 件（大アイコンは自作の単純な幾何パス、lucide 等の
/// 著作物は複製しない。`feature_expand::geo_icon` と同型の判断）。
const ICON_ITEMS: [IconFeature; 4] = [
    IconFeature {
        title: "決定的なビルド",
        body: "同じ入力からは常に同じ静的ファイルを生成します。",
        icon_path_d: "M4 4h16v16H4z",
    },
    IconFeature {
        title: "型で表す構造",
        body: "スロットと props は Rust の型で表現されます。",
        icon_path_d: "M12 3l9 6-9 6-9-6z",
    },
    IconFeature {
        title: "既定エスケープ",
        body: "テキスト補間は既定でエスケープされます。",
        icon_path_d: "M12 3v18M3 12h18",
    },
    IconFeature {
        title: "外部依存ゼロ",
        body: "描画コアは外部クレートに依存しません。",
        icon_path_d: "M12 2a10 10 0 100 20 10 10 0 000-20z",
    },
];

/// 形 B の架空データ 6 件（チェックは意味を持たない箇条マーカーとして
/// 全項目に一様に付ける、モジュール doc「アイコンを装飾扱いにする理由」
/// 節参照）。
const CHECK_ITEMS: [CheckFeature; 6] = [
    CheckFeature {
        title: "SSR / SSG / CSR 対応",
        body: "1 つのコンポーネント定義を複数のレンダリング方式で使えます。",
    },
    CheckFeature {
        title: "単一実行ファイル配布",
        body: "サーバーを 1 つのバイナリとして配布できます。",
    },
    CheckFeature {
        title: "状態管理コア同梱",
        body: "追加の外部ライブラリなしで状態を扱えます。",
    },
    CheckFeature {
        title: "アクセシビリティ配慮",
        body: "WAI-ARIA・キーボード操作を部品側で担保します。",
    },
    CheckFeature {
        title: "決定的な出力",
        body: "同一入力から常に同一のバイト列を生成します。",
    },
    CheckFeature {
        title: "依存グラフ上限の管理",
        body: "依存パッケージ数・深さの上限を CI で機械検証します。",
    },
];

/// 装飾用の自作幾何アイコン（`fill="none"` + `stroke="currentColor"` で
/// 線画として描く、`feature_expand::geo_icon` と同型の判断）。
fn geo_icon(path_d: &'static str, size: Size) -> Node {
    icon(
        &IconProps {
            size,
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

/// 装飾用のチェックマークアイコン（意味を持たない箇条マーカーとして
/// 全項目一様に付ける）。
fn check_icon() -> Node {
    geo_icon("M4 12l5 5L20 6", Size::Sm)
}

/// 左列（見出し + 説明）。形 A・形 B いずれの節でも使う共通パーツ。
fn side_heading(eyebrow: &'static str, title: &'static str, lead: &'static str) -> Node {
    div(
        vec![("class", "blocks-feature-side-heading-grid-heading")],
        vec![
            badge::badge(
                &BadgeProps::default(),
                vec![("data-blocks-feature-side-heading-grid-eyebrow", "")],
                vec![text(eyebrow)],
            ),
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
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
                vec![("data-blocks-feature-side-heading-grid-lead", "")],
                vec![text(lead)],
            ),
        ],
    )
}

/// 形 A の 1 項目（大アイコン + 項目見出し + 説明）。
fn icon_item(item: &IconFeature) -> Node {
    div(
        vec![("class", "blocks-feature-side-heading-grid-icon-item")],
        vec![
            geo_icon(item.icon_path_d, Size::Xl),
            heading(
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
                vec![text(item.body)],
            ),
        ],
    )
}

/// 形 B の 1 項目（チェック + 項目名 + 説明）。
fn check_item(item: &CheckFeature) -> Node {
    div(
        vec![("class", "blocks-feature-side-heading-grid-check-item")],
        vec![
            check_icon(),
            div(
                vec![("class", "blocks-feature-side-heading-grid-check-text")],
                vec![
                    heading(
                        HeadingLevel::H4,
                        &HeadingProps {
                            size: HeadingSize::Md,
                            ..HeadingProps::default()
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
                        vec![text(item.body)],
                    ),
                ],
            ),
        ],
    )
}

/// 1 つの節（見出し列 + グリッド列）を lg 5 列 grid（2:3）の枠へ組み立てる。
fn section(heading_col: Node, grid_items: Vec<Node>) -> Node {
    div(
        vec![("class", "blocks-feature-side-heading-grid-section")],
        vec![
            heading_col,
            div(
                vec![("class", "blocks-feature-side-heading-grid-grid")],
                grid_items,
            ),
        ],
    )
}

/// `feature-side-heading-grid` の Demo 本体。呼び出しごとに同一の `Node`
/// を返す純関数（他 block と同じ状態を持たない設計）。形 A・形 B の 2 節を
/// 縦に並べる（モジュール doc「2 つの形を縦に並べる」節参照）。
pub fn demo() -> Node {
    let section_a = section(
        side_heading(
            "機能",
            "主要な機能を一望する",
            "アイコンと短い説明で、代表的な 4 つの機能を紹介します。",
        ),
        ICON_ITEMS.iter().map(icon_item).collect(),
    );

    let section_b = section(
        side_heading(
            "含まれるもの",
            "標準で含まれる機能",
            "追加の設定なしで最初から使える機能の一覧です。",
        ),
        CHECK_ITEMS.iter().map(check_item).collect(),
    );

    div(
        vec![("class", "blocks-feature-side-heading-grid-layout")],
        vec![section_a, section_b],
    )
}
```

## 原案差分メモ

主参照（対応表 ID R0941、見出し 2 列 + 大アイコン 2×2 の基準形）と、
集約元 R0101（2 列 4 feature）・R0475（2 列チェック付きカード 6 枚）・
R0943（チェック付き 8 件の 2 列）からの意図的な差分は次のとおりです
（出典の固有名・ファイル名は記載しません）。

- R0943 の 8 件は R0475 と揃えて 6 件へ統一しました。
- R0475 のカード枠は付けません（`card` が使用部品に含まれないため、
  各項目は素の `div` のみで構成します）。
- 大アイコンは lucide 等の複製ではなく自作の単純な幾何パスです。
- 見出しは `h3`、項目見出しは `h4`（チェック項目のみ `h4` の `Md`
  サイズ）にしました（ページ側が `## Demo` として `h2` を出すため）。
- lg（64rem）未満は見出し列の下にグリッド列が続く縦積みにし、xl 以上の
  分割は持ち込みません。
- チェックは全項目が一様にチェック済みの箇条マーカーとして装飾扱い
  （`aria-hidden`）にし、対応/非対応を区別する意味は持たせません。
- 文言は独自に書いた架空のもの、配色・余白は既存のテーマトークンに
  そのまま従います。
