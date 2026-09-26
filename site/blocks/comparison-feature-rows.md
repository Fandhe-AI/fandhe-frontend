# comparison-feature-rows

`heading` / `text` / `badge` / `icon` / `separator` の 5 部品を合成した、機能別の比較行レイアウトです。見出しブロックの下に、機能ごとの比較行を縦に並べます。

- 静的表示です。docs サイトは無 JS のため、状態機械・フォーム・データ取得を持たない固定描画にしています。
- 文言はすべて架空のものです。実在の企業名・製品名・競合名は使わず、中立的な「自社」「他社」ラベルにしています。
- データ取得・送信は行わず、`<form>` は使いません。
- 各行は「アイコン付きの機能名」「自社の説明」「他社の説明」の 3 要素で構成し、行と行の間は罫線（`separator` 部品）で区切ります（最後の行の後には置きません）。
- 狭い幅では 3 要素を縦積みにします。`48rem` 以上（md）で行を 2 列にし、機能名を全幅にして自社・他社を横並びにします。`64rem` 以上（lg）で行を 3 列にし、機能名・自社・他社を横一列に並べます。
- アイコンは参照元の SVG をコピーせず、円・四角・水平線 3 本の独自の抽象図形にしています。
- 集約元は 1 件です（対応表 ID R0438）。取り込んだのはレイアウト構造（見出しブロック + 機能行の縦積み・レスポンシブな列数切り替え・区切り線の使い方）のみで、文言・配色・装飾・アイコン・内部識別子は取り込んでいません。

## Rust コード

```rust
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps, BadgeVariant};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::separator::{self, SeparatorProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};
use fandhe_frontend_pre_styled_ui::ColorPalette;

/// 円のみの抽象アイコン（`stroke` のみで塗り面を持たない、`feature_expand::
/// geo_icon` と同型の対処）。
fn glyph_circle() -> Node {
    icon(
        &IconProps::default(),
        vec![],
        vec![el(
            "circle",
            vec![
                ("cx", "12"),
                ("cy", "12"),
                ("r", "8"),
                ("fill", "none"),
                ("stroke", "currentColor"),
                ("stroke-width", "2"),
            ],
            vec![],
        )],
    )
}

/// 四角の輪郭のみの抽象アイコン。
fn glyph_square() -> Node {
    icon(
        &IconProps::default(),
        vec![],
        vec![el(
            "path",
            vec![
                ("d", "M5 5h14v14H5z"),
                ("fill", "none"),
                ("stroke", "currentColor"),
                ("stroke-width", "2"),
                ("stroke-linejoin", "round"),
            ],
            vec![],
        )],
    )
}

/// 水平線 3 本の抽象アイコン（一覧・段階性を示す図形）。
fn glyph_lines() -> Node {
    icon(
        &IconProps::default(),
        vec![],
        vec![el(
            "path",
            vec![
                ("d", "M4 6h16M4 12h16M4 18h16"),
                ("fill", "none"),
                ("stroke", "currentColor"),
                ("stroke-width", "2"),
                ("stroke-linecap", "round"),
            ],
            vec![],
        )],
    )
}

/// 機能 1 行分のダミーデータ（架空、実在の製品・企業とは無関係）。
struct FeatureRow {
    glyph: fn() -> Node,
    name: &'static str,
    ours: &'static str,
    theirs: &'static str,
}

/// 比較行一覧（架空、3 行固定。検索インデックスの肥大を抑えるため件数を
/// 増やさない）。
const ROWS: [FeatureRow; 3] = [
    FeatureRow {
        glyph: glyph_circle,
        name: "同時編集",
        ours: "何人でも同時に編集でき、変更はリアルタイムに反映されます。",
        theirs: "同時編集は上位プランのみで、人数に上限があります。",
    },
    FeatureRow {
        glyph: glyph_square,
        name: "オフライン対応",
        ours: "オフラインでも編集を続けられ、復帰時に自動で同期します。",
        theirs: "オフライン編集には対応していません。",
    },
    FeatureRow {
        glyph: glyph_lines,
        name: "変更履歴",
        ours: "すべての変更履歴を無期限に保持し、いつでも復元できます。",
        theirs: "変更履歴の保持期間は 30 日間に限られます。",
    },
];

/// 見出しブロック（タグライン `badge` + セクション見出し + リード文）。
fn intro() -> Node {
    div(
        vec![("class", "blocks-comparison-feature-rows-intro")],
        vec![
            badge::badge(&BadgeProps::default(), vec![], vec![text("比較")]),
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    weight: HeadingWeight::Bold,
                },
                vec![],
                vec![text("機能で比較する")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text("主要な機能を自社・他社で並べて確認できます。")],
            ),
        ],
    )
}

/// 機能名セル（アイコン + `heading H4`）。
fn feature_cell(row: &FeatureRow) -> Node {
    div(
        vec![("data-blocks-comparison-feature-rows-feature", "")],
        vec![
            (row.glyph)(),
            heading(
                HeadingLevel::H4,
                &HeadingProps {
                    size: HeadingSize::Md,
                    weight: HeadingWeight::Semibold,
                },
                vec![],
                vec![text(row.name)],
            ),
        ],
    )
}

/// 自社/他社の説明セル（列ラベル `badge` + 説明 `text`）。
fn party_cell(
    hook: &'static str,
    label: &'static str,
    variant: BadgeVariant,
    description: &str,
) -> Node {
    div(
        vec![(hook, "")],
        vec![
            badge::badge(
                &BadgeProps {
                    variant,
                    palette: if hook.ends_with("ours") {
                        ColorPalette::Accent
                    } else {
                        ColorPalette::Neutral
                    },
                    ..BadgeProps::default()
                },
                vec![],
                vec![text(label)],
            ),
            styled_text::text(&TextProps::default(), vec![], vec![text(description)]),
        ],
    )
}

/// 機能 1 行分（機能名・自社・他社の 3 要素）。
fn feature_row(row: &FeatureRow) -> Node {
    div(
        vec![("data-blocks-comparison-feature-rows-row", "")],
        vec![
            feature_cell(row),
            party_cell(
                "data-blocks-comparison-feature-rows-ours",
                "自社",
                BadgeVariant::Solid,
                row.ours,
            ),
            party_cell(
                "data-blocks-comparison-feature-rows-theirs",
                "他社",
                BadgeVariant::Outline,
                row.theirs,
            ),
        ],
    )
}

/// `comparison-feature-rows` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数（[`crate::blocks`] モジュール doc「静的表示」節）。
pub fn demo() -> Node {
    let last_index = ROWS.len().saturating_sub(1);
    let mut list_children: Vec<Node> = Vec::new();
    for (index, row) in ROWS.iter().enumerate() {
        list_children.push(feature_row(row));
        if index != last_index {
            list_children.push(separator::separator(
                &SeparatorProps::default(),
                vec![("data-blocks-comparison-feature-rows-separator", "")],
            ));
        }
    }

    div(
        vec![("data-blocks-comparison-feature-rows-root", "")],
        vec![
            intro(),
            div(
                vec![("data-blocks-comparison-feature-rows-list", "")],
                list_children,
            ),
        ],
    )
}
```

**原案差分メモ**

- アイコンは参照元の SVG path をコピーせず、円・四角・水平線 3 本の自作の抽象図形に差し替えました。
- タグライン（「比較」）は `badge` で表現しました。
- 見出しレベルは、ページ側の `## Demo` に合わせてセクション見出しを `h3`、機能名見出しを `h4` にしています。
- 行間の罫線は `separator` 部品で描いています。
- 自社・他社のラベルは `badge` で示し、自社側を強調配色（`Solid` + `Accent`）、他社側を中立配色（`Outline` + `Neutral`）にして視覚的に区別しています。
- md（`48rem` 以上）の 2 列段階を追加しました。機能名を全幅にして自社・他社を横並びにする表現です。
- 配色・余白・角丸は既存のテーマトークンに従っています。
- 文言（機能名・自社/他社の説明）はすべて独自に書きました。実在の企業名・製品名・競合名は使っていません。
