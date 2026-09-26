# logo-cloud-split

`heading` / `text` / `image` / `link` の 4 部品を組み合わせた、
見出し左 + ロゴ 2 列グリッド右の合成例です。`lg` 未満では見出しの下に
ロゴが縦積みになります。新しい UI 部品は作らず、既存部品のみで構成して
います。無 JS の静的な表示で `<form>` は出力しません。CTA 2 本は
遷移先を持たない `<button>` ではなく `link::root` で組み立て、本
リポジトリの固定 URL へ実際に遷移します。

主参照は対応表 ID R1058、集約元は対応表 ID R0565・R0149・R0566・R0145・
R0567 の 5 件です（出典の固有名・ファイル名は記載しません）。ロゴ・社名は
すべて架空で、実在ブランドのロゴ・商標は使っていません。tagline も
「導入企業（デモ用の架空サンプル）」と明記し、見出し文言が実際の導入
実績ではないことを示します。

## Rust コード

```rust
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::image::{image, ImageFit, ImageProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps, LinkVariant};
use fandhe_frontend_pre_styled_ui::recipe::ColorPalette;
use fandhe_frontend_pre_styled_ui::text::{
    self as styled_text, TextProps, TextSize, TextVariant, TextWeight,
};

/// 各形の直前に置く短い形ラベル（`styled_text::text` の `Sm`/`Muted`、
/// `cta_split_image::variant_label` と同型）。
fn variant_label(label: &'static str) -> Node {
    styled_text::text(
        &TextProps {
            size: TextSize::Sm,
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![],
        vec![text(label)],
    )
}

/// ロゴ 1 件（同一の抽象バッジ SVG + 架空社名キャプション）。`bordered`
/// が true のとき淡色枠タイルへ収める（淡色枠タイル形の差分）。
fn logo_item(company: &'static str, bordered: bool) -> Node {
    // alt は空文字にする（隣接する `caption` が同じ社名を可視テキストとして
    // 持つため、`alt` にも同じ文字列を入れるとスクリーンリーダーが同名を
    // 二重読み上げしてしまう。`logo_cloud_marquee::logo` と同じ判断）。
    let logo_image = image(
        &ImageProps {
            fit: ImageFit::Contain,
            ..ImageProps::new(dummy_assets::LOGO_SRC, "")
        },
        vec![("data-blocks-logo-cloud-split-logo", "")],
    );
    let caption = styled_text::text(
        &TextProps {
            size: TextSize::Xs,
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![],
        vec![text(company)],
    );
    if bordered {
        div(
            vec![("data-blocks-logo-cloud-split-tile", "")],
            vec![logo_image, caption],
        )
    } else {
        div(vec![], vec![logo_image, caption])
    }
}

/// ロゴ 6 件を 2 列グリッドへ並べる（[`dummy_assets::COMPANY_NAMES`] の
/// 先頭 6 件を使う）。
fn logo_grid(bordered: bool) -> Node {
    div(
        vec![("class", "blocks-logo-cloud-split-grid")],
        dummy_assets::COMPANY_NAMES
            .iter()
            .take(6)
            .map(|company| logo_item(company, bordered))
            .collect(),
    )
}

/// 基準形・暗色固定形で共通の左列（tagline + 見出し + 説明 + CTA ボタン
/// 2 本 + GitHub リンク）。
fn copy_with_cta() -> Node {
    div(
        vec![("class", "blocks-logo-cloud-split-copy")],
        vec![
            styled_text::text(
                &TextProps {
                    size: TextSize::Sm,
                    weight: TextWeight::Medium,
                    ..TextProps::default()
                },
                vec![],
                vec![text("導入企業（デモ用の架空サンプル）")],
            ),
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    weight: HeadingWeight::Bold,
                },
                vec![],
                vec![text("多くのチームに選ばれています")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(
                    "様々な規模のチームが日々の開発にご利用いただいています。",
                )],
            ),
            div(
                vec![("class", "blocks-logo-cloud-split-cta")],
                vec![
                    link::root(
                        REPO,
                        &LinkProps {
                            variant: LinkVariant::Underline,
                            ..LinkProps::default()
                        },
                        vec![],
                        vec![text("GitHub で見る")],
                    ),
                    link::root(
                        REPO,
                        &LinkProps {
                            variant: LinkVariant::Underline,
                            palette: ColorPalette::Neutral,
                            ..LinkProps::default()
                        },
                        vec![],
                        vec![text("Star をつける")],
                    ),
                ],
            ),
        ],
    )
}

/// 基準形（R1058 主参照）: CTA 付き左列 + 枠なしロゴ 2 列グリッド。
fn variant_basic() -> Node {
    div(
        vec![
            ("class", "blocks-logo-cloud-split-row"),
            ("data-blocks-logo-cloud-split-row", ""),
        ],
        vec![copy_with_cta(), logo_grid(false)],
    )
}

/// 淡色枠タイル形（R0149/R0566/R0145 集約）: 見出し + 説明のみの左列
/// （CTA なし）+ 淡色枠タイルのロゴ 2 列グリッド。
fn variant_bordered() -> Node {
    let left = div(
        vec![("class", "blocks-logo-cloud-split-copy")],
        vec![
            styled_text::text(
                &TextProps {
                    size: TextSize::Sm,
                    weight: TextWeight::Medium,
                    ..TextProps::default()
                },
                vec![],
                vec![text("導入企業（デモ用の架空サンプル）")],
            ),
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    weight: HeadingWeight::Bold,
                },
                vec![],
                vec![text("信頼できるパートナー企業")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text("業界を代表する企業と協業しています。")],
            ),
        ],
    );
    div(
        vec![
            ("class", "blocks-logo-cloud-split-row"),
            ("data-blocks-logo-cloud-split-row", ""),
        ],
        vec![left, logo_grid(true)],
    )
}

/// 暗色固定形（R0567 集約）: 基準形と同じ左列構成を暗色面上に配置する
/// （モジュール doc「暗色固定」節）。
fn variant_dark() -> Node {
    div(
        vec![("data-blocks-logo-cloud-split-tone", "dark")],
        vec![div(
            vec![
                ("class", "blocks-logo-cloud-split-row"),
                ("data-blocks-logo-cloud-split-row", ""),
            ],
            vec![copy_with_cta(), logo_grid(true)],
        )],
    )
}

/// `logo-cloud-split` の Demo 本体（3 形を縦に並記）。呼び出しごとに同一の
/// `Node` を返す純関数。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-logo-cloud-split-layout")],
        vec![
            variant_label("基準形（R1058）"),
            variant_basic(),
            variant_label("淡色枠タイル形（R0149/R0566/R0145）"),
            variant_bordered(),
            variant_label("暗色固定形（R0567）"),
            variant_dark(),
        ],
    )
}
```

## 原案差分メモ

- 主参照（R1058）・集約元 5 件（R0565・R0149・R0566・R0145・R0567）を
  基準形 / 淡色枠タイル形 / 暗色固定形の 3 形へ統合しました。
- 淡色枠タイル形は R0149・R0566・R0145 を集約し、CTA を持たない
  見出しのみの左列（R0145 の差分）を採用しています。
- 「横並び」（R0565）は Demo を増やさず、基準形のグリッド列数を変える
  だけで表現できるためこのメモに留めています。
- 暗色固定形（R0567）は現在のテーマ（ライト/ダーク）に関わらず常に
  暗色面で表示します。ライト/ダーク切替に追随する仕組みではありません。
- 文言・社名はすべて架空です。ロゴは同一の抽象バッジ画像で統一し、
  実在ブランドのロゴ・商標は使っていません。
- 配色は基本的にテーマトークン（`--fandhe-color-*`）経由ですが、暗色固定形
  （R0567）のみライト/ダーク切替に追随しない固定色が必要なため、
  `--fandhe-color-fg`/`--fandhe-color-bg` のライトテーマ値相当（`#111111`/
  `#ffffff`/`#f7f7f7`）を意図的な例外としてリテラル直書きしています。
- レイアウトは `lg`（64rem）未満で 1 列（見出しの下にロゴ）、`lg` 以上で
  左見出し + 右ロゴの 2 カラムに切り替わります。

関連情報:
[Heading](../themes/heading.md) /
[Text](../themes/text.md) /
[Image](../themes/image.md) /
[Link](../themes/link.md)
