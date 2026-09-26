# cta-feature-links

`fandhe-frontend-pre-styled-ui` の `heading` / `text` / `button` / `icon` /
`link-overlay` / `separator` 部品を合成した、見出し・説明文・CTA ボタンの
左列と、アイコン付きリンク項目 2 件の右列からなる 2 列 CTA です。Blocks
セクションは新規部品を追加するものではなく、既存の Themes/Primitives 部品
を組み合わせた実例集であることに注意してください（主参照は対応表 ID
R0070、右列のアイコン付き項目は R0072 を構造として集約。出典の固有名・
ファイル名は記載しません）。

`64rem` 未満の幅では 1 列（見出し + CTA が上、リンク項目が下）へ折り返し、
`64rem` 以上で左右 2 列になります。各リンク項目は `link-overlay` により
枠全体がクリック可能で、右端に矢印アイコンを添えています。

本 Demo は静的な表示例であり、`<form>` 要素を一切持たず、データの取得・
送信・状態管理を行いません。CTA ボタンは `type="button"` のまま送信先を
持たず、リンク項目の遷移先はいずれも本リポジトリへの固定 URL です。文言は
すべて独自に書いた架空のものであり、実企業名・実クレデンシャル・PII を
含みません。

## Rust コード

```rust
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::link_overlay::{self, overlay};
use fandhe_frontend_pre_styled_ui::separator::{separator, SeparatorProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};
use fandhe_frontend_pre_styled_ui::Size;

/// リンク先の固定外部 URL（モジュール doc「`href="#"` を使わない」節参照）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// 自作の幾何アイコン（線画。モジュール doc「アイコンは自作の抽象幾何
/// 図形」節参照）。`path` へ `fill="none"` + `stroke="currentColor"` を
/// 明示し、`icon` の `<svg>` 側が固定で持つ `fill="currentColor"`
/// （塗り面）を上書きして線画（ストローク）として描画する。
fn geo_icon(path_d: &'static str) -> Node {
    icon(
        &IconProps::default(),
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

/// 右端に置く矢印アイコン（全項目共通）。
fn arrow_icon() -> Node {
    geo_icon("M5 12h14M13 6l6 6-6 6")
}

/// リンク項目 1 件分（全面リンク、モジュール doc「全面リンク項目の構造」
/// 節）。`icon_path_d` は項目ごとの幾何アイコンの `d` 属性値。
fn feature_item(icon_path_d: &'static str, title: &str, description: &str) -> Node {
    link_overlay::root(
        vec![("data-blocks-cta-feature-links-item", "")],
        vec![
            div(
                vec![("class", "blocks-cta-feature-links-item-icon")],
                vec![geo_icon(icon_path_d)],
            ),
            div(
                vec![("class", "blocks-cta-feature-links-item-copy")],
                vec![
                    heading(
                        HeadingLevel::H4,
                        &HeadingProps::default(),
                        vec![],
                        vec![text(title)],
                    ),
                    styled_text::text(
                        &TextProps {
                            variant: TextVariant::Muted,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text(description)],
                    ),
                ],
            ),
            div(
                vec![("class", "blocks-cta-feature-links-item-arrow")],
                vec![arrow_icon()],
            ),
            overlay(REPO, vec![("aria-label", title)], vec![]),
        ],
    )
}

/// 左列（見出し + 説明文 + CTA ボタン）。
fn content_column() -> Node {
    div(
        vec![("class", "blocks-cta-feature-links-content")],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    ..HeadingProps::default()
                },
                vec![],
                vec![text("チームの成果を、そのまま次の意思決定へ")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(
                    "計測・共有・振り返りを 1 つの導線にまとめ、\
                     散らばった記録を集める手間をなくします。",
                )],
            ),
            button::button(
                &ButtonProps {
                    size: Size::Lg,
                    ..ButtonProps::default()
                },
                vec![],
                vec![text("今すぐ始める")],
            ),
        ],
    )
}

/// 右列（リンク項目 2 件 + 区切り線 1 本、モジュール doc「全面リンク項目の
/// 構造」節）。
fn links_column() -> Node {
    div(
        vec![("class", "blocks-cta-feature-links-links")],
        vec![
            feature_item(
                "M4 4h16v4H4zM4 12h10v8H4z",
                "レポートを共有する",
                "作成した記録をチームへワンクリックで届けます。",
            ),
            separator(&SeparatorProps::default(), vec![]),
            feature_item(
                "M12 3v18M3 12h18",
                "指標を組み合わせる",
                "複数の計測項目を並べて、変化の理由を追いやすくします。",
            ),
        ],
    )
}

/// `cta-feature-links` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-cta-feature-links-grid")],
        vec![content_column(), links_column()],
    )
}
```

**原案差分メモ**

参照（対応表 ID R0070 が主参照、R0072 を構造として集約。出典の固有名・
ファイル名は記載しません）からの意図的な差分は次のとおりです。

- R0072 の「アイコンを背景付きの箱に入れる」表現は、各リンク項目の
  アイコン枠として採り入れました。一方 R0072 が持つインラインの行動リンク
  （項目内の追加リンク）は Issue の使用部品に `link` が含まれないため
  追加していません。
- R0070 の枠付きカードは採らず、2 項目の間に水平の `separator` を 1 本
  挟んで区切る構成にしました（枠線を持たない分、罫線 1 本で境界を示す
  判断です）。
- 参照側が持つ 2 つ目の CTA ボタン（Secondary アクション）は省略し、
  Primary の 1 個のみにしました。
- 見出しレベルは `h2` から `h3`（セクション見出し）・`h4`（リンク項目の
  見出し）へ下げました（ページ側が `## Demo` として `h2` を出すため）。
- `href="#"` の死リンクは、すべてリポジトリへの固定外部 URL に置き換え
  ました。
- `id` や `aria-labelledby` は出力しません（宙に浮いた ARIA 参照・id 重複
  を構造的に避けるため）。
- 文言（見出し・説明文・リンク項目の見出しと説明）・アイコンの形・配色は
  すべて独自に書き直しました。
- ブレークポイントは `64rem`（`lg`）に固定しています。
