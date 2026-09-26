# cta-split-image

`fandhe-frontend-pre-styled-ui` の `badge` / `heading` / `text` / `button` /
`image` / `card` / `icon` / `link` 部品を合成した、画像付きの分割 CTA です。
Blocks セクションは新規部品を追加するものではなく、既存の Themes/Primitives
部品を組み合わせた実例集であることに注意してください（主参照は対応表 ID
R0883。R0448・R0450・R0875・R0884・R0885 を構造として集約。出典の固有名・
ファイル名は記載しません）。

Demo は次の 4 形を並記しています。

- **画像 1 枚 + コピー列**（R0883 基準形）: 片側に画像、反対側に
  eyebrow・見出し・本文・CTA ボタン 1 つ。
- **暗色 / アクセント色のカード**（R0875/R0450/R0448 集約）: カードの端まで
  画像を全面に広げ、暗色・アクセント色の 2 トーンで描画します。
- **チェック付き項目**（R0884）: カード内に写真 + 見出し + 本文 + チェック
  付き項目 6 件 + 誘導リンク 1 本。
- **タイル配置**（R0885）: 画像 1 枚の代わりに 2×2 のタイル 4 枚。

いずれの形も幅 `64rem`（`lg`）未満では 1 列（画像が上、コピー列が下）へ
折り返し、`64rem` 以上で左右 2 列になります。

本 Demo は静的な表示例であり、`<form>` 要素を一切持たず、データの取得・
送信・状態管理を行いません。CTA ボタンは `type="button"` のまま送信先を
持たず、誘導リンクの遷移先は本リポジトリへの固定外部 URL です。文言は
すべて独自に書いた架空のものであり、実企業名・実クレデンシャル・PII を
含みません。画像はビルド時生成のプレースホルダー SVG（`alt=""`）です。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, li, text, ul, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::image::{self, ImageFit, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::Size;

/// リンク先の固定外部 URL（モジュール doc「link の `href` を固定の外部絶対
/// URL にする」節参照）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// 自作の幾何アイコン（線画。モジュール doc「C: チェック付き項目の
/// アイコン」節参照）。`path` へ `fill="none"` + `stroke="currentColor"` を
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

/// チェックマークの幾何アイコン。
fn check_icon() -> Node {
    geo_icon("M5 13l4 4L19 7")
}

/// 各形の直前に置く短い形ラベル（`styled_text::text` の `Sm`/`Muted`）。
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

/// A/D で共通のコピー列（eyebrow badge + 見出し + 本文 + CTA ボタン 1 つ）。
fn copy_column(
    eyebrow: &'static str,
    title: &'static str,
    body: &'static str,
    cta: &'static str,
) -> Node {
    div(
        vec![("class", "blocks-cta-split-image-copy")],
        vec![
            badge::badge(&BadgeProps::default(), vec![], vec![text(eyebrow)]),
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    ..HeadingProps::default()
                },
                vec![],
                vec![text(title)],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(body)],
            ),
            button::button(
                &ButtonProps {
                    size: Size::Lg,
                    ..ButtonProps::default()
                },
                vec![],
                vec![text(cta)],
            ),
        ],
    )
}

/// 形 A（R0883 基準形）: 画像 1 枚（片側）+ コピー列（反対側）。DOM 順は
/// 画像 → コピー列（lg 未満の 1 列表示で画像が上に来るようにするため）。
fn variant_basic() -> Node {
    div(
        vec![("class", "blocks-cta-split-image-basic-grid")],
        vec![
            image::image(
                &ImageProps {
                    fit: ImageFit::Cover,
                    shape: ImageShape::Rounded,
                    ..ImageProps::new(dummy_assets::SCREENSHOT_SRC, "")
                },
                vec![("data-blocks-cta-split-image-basic-image", "")],
            ),
            copy_column(
                "新機能",
                "導入から公開まで、迷わず進める",
                "テンプレートと雛形を組み合わせ、初期構築にかかる時間を短縮します。",
                "今すぐ試す",
            ),
        ],
    )
}

/// 形 B（R0448/R0450/R0875 集約）: 暗色またはアクセント色のカードへ収め、
/// 画像をカード端まで全面に広げる。`tone`（`"dark"`/`"accent"`）で 2
/// インスタンス描画する（モジュール doc「B: カードの端まで全面に画像を
/// 広げる方法」節）。
fn variant_card(
    tone: &'static str,
    title: &'static str,
    body: &'static str,
    cta: &'static str,
) -> Node {
    let copy = card::body(
        vec![("data-blocks-cta-split-image-card-copy", "")],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    ..HeadingProps::default()
                },
                vec![],
                vec![text(title)],
            ),
            styled_text::text(&TextProps::default(), vec![], vec![text(body)]),
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Ghost,
                    size: Size::Lg,
                    ..ButtonProps::default()
                },
                vec![("data-blocks-cta-split-image-cta", "")],
                vec![text(cta)],
            ),
        ],
    );

    let media = div(
        vec![("class", "blocks-cta-split-image-card-media")],
        vec![image::image(
            &ImageProps {
                fit: ImageFit::Cover,
                ..ImageProps::new(dummy_assets::SCREENSHOT_SRC, "")
            },
            vec![("data-blocks-cta-split-image-card-image", "")],
        )],
    );

    card::root(
        CardProps::default(),
        vec![
            ("data-blocks-cta-split-image-card", ""),
            ("data-blocks-cta-split-image-tone", tone),
        ],
        vec![media, copy],
    )
}

/// チェック付き項目 1 件分（チェックマーク + 短い文言）。
fn checklist_item(label: &'static str) -> Node {
    li(
        vec![("class", "blocks-cta-split-image-checklist-item")],
        vec![check_icon(), text(label)],
    )
}

/// チェック付き項目 6 件（架空文言）。
const CHECKLIST_ITEMS: [&str; 6] = [
    "既定エスケープ済みの HTML 出力",
    "外部依存ゼロの描画コア",
    "SSR / SSG / CSR の切り替え",
    "型で保証されたコンポーネント境界",
    "決定的なビルド成果物",
    "単一実行ファイルでの配布",
];

/// 形 C（R0884）: カード内に写真 + 見出し + 本文 + チェック付き項目 6 件 +
/// 誘導リンク 1 本。
fn variant_checklist() -> Node {
    let media = div(
        vec![("class", "blocks-cta-split-image-checklist-media")],
        vec![image::image(
            &ImageProps {
                fit: ImageFit::Cover,
                shape: ImageShape::Rounded,
                ..ImageProps::new(dummy_assets::PRODUCT_SRC, "")
            },
            vec![("data-blocks-cta-split-image-checklist-image", "")],
        )],
    );

    let copy = div(
        vec![("class", "blocks-cta-split-image-checklist-copy")],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    ..HeadingProps::default()
                },
                vec![],
                vec![text("導入前に確認したいポイント")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(
                    "既存のチームでもそのまま採用できるよう、次の点を満たしています。",
                )],
            ),
            ul(
                vec![("class", "blocks-cta-split-image-checklist-list")],
                CHECKLIST_ITEMS
                    .iter()
                    .map(|label| checklist_item(label))
                    .collect(),
            ),
            link::root(
                REPO,
                &LinkProps {
                    external: true,
                    ..LinkProps::default()
                },
                vec![],
                vec![text("導入事例を見る")],
            ),
        ],
    );

    card::root(
        CardProps::default(),
        vec![("data-blocks-cta-split-image-checklist-card", "")],
        vec![div(
            vec![("class", "blocks-cta-split-image-checklist-grid")],
            vec![media, copy],
        )],
    )
}

/// タイル 4 枚（同じ絵が並ばないよう 4 種のダミー素材を使う）。
fn tiles() -> Node {
    let sources = [
        dummy_assets::PRODUCT_SRC,
        dummy_assets::AVATAR_SRC,
        dummy_assets::LOGO_SRC,
        dummy_assets::BACKGROUND_SRC,
    ];
    div(
        vec![("class", "blocks-cta-split-image-tiles")],
        sources
            .iter()
            .map(|src| {
                image::image(
                    &ImageProps {
                        fit: ImageFit::Cover,
                        shape: ImageShape::Rounded,
                        ..ImageProps::new(src, "")
                    },
                    vec![("data-blocks-cta-split-image-tile-image", "")],
                )
            })
            .collect(),
    )
}

/// 形 D（R0885）: 画像 1 枚の代わりに 2×2 のタイル 4 枚 + コピー列。
fn variant_tiles() -> Node {
    div(
        vec![("class", "blocks-cta-split-image-tiles-grid")],
        vec![
            tiles(),
            copy_column(
                "ギャラリー",
                "実際の画面をひとまとめに確認",
                "複数の切り口から成果物を見せたいときに、タイル配置で並べて提示できます。",
                "詳しく見る",
            ),
        ],
    )
}

/// `cta-split-image` の Demo 本体。呼び出しごとに同一の `Node` を返す純
/// 関数（モジュール doc「4 形を 1 つの Demo に並記する」節）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-cta-split-image-layout")],
        vec![
            variant_label("画像 1 枚 + コピー列（R0883 基準形）"),
            variant_basic(),
            variant_label("暗色カード + 全面画像（R0875/R0450 集約）"),
            variant_card(
                "dark",
                "リリースノートを自動でまとめる",
                "コミット履歴から変更点を抽出し、公開前に要点だけを確認できます。",
                "詳細を見る",
            ),
            variant_label("アクセント色カード + 全面画像（R0448 集約）"),
            variant_card(
                "accent",
                "チームの合言葉を、そのまま画面へ",
                "配色トークンを差し替えるだけで、複数ブランドの画面を作り分けられます。",
                "始める",
            ),
            variant_label("チェック付き項目（R0884）"),
            variant_checklist(),
            variant_label("タイル配置（R0885）"),
            variant_tiles(),
        ],
    )
}
```

**原案差分メモ**

参照（対応表 ID R0883 が主参照、R0448/R0450/R0875/R0884/R0885 を構造として
集約。出典の固有名・ファイル名は記載しません）からの意図的な差分は次の
とおりです。

- 3 種類のカード形（暗色パネル、反転カード、アクセント色カード）は、単一の
  `variant_card(tone, ...)` 関数の `tone` 引数（`"dark"`/`"accent"`）へ集約
  しました。参照側が個別に持つ配色・装飾の細部は再現していません。
- 「画像をカードの端まで全面に広げる」表現は、`card::root` へ
  `padding: 0; overflow: hidden; border: 0;` を上書きし、画像とコピー列を
  横並びの 2 トラックとして扱うことで実現しました（絶対配置・負の
  マージンは使いません）。
- 見出しは参照側の `h2` 相当ではなく `h3`（ページ側が `## Demo` として
  `h2` を出すため）にしています。eyebrow は `badge` で表現しました。
- チェック付き項目のアイコンは、lucide 等の著作物を複製しない自作の
  幾何チェックマーク（折れ線）にしています。
- 画像はすべてビルド時生成のプレースホルダー SVG（`alt=""`）で、装飾用の
  背景グラデーション・実写風の合成は持ち込んでいません。
- 文言（eyebrow・見出し・本文・チェック項目・リンク文言）・配色トークンの
  割り当ては、すべて独自に書き直しました。
- `href="#"` の死リンクは、誘導リンクをリポジトリへの固定外部 URL に
  置き換えました。
- `id` や `aria-labelledby` は出力しません（宙に浮いた ARIA 参照・id 重複
  を構造的に避けるため）。
- ブレークポイントは `64rem`（`lg`）に固定しています。
