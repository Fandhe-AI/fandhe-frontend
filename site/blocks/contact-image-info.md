# contact-image-info

`fandhe-frontend-pre-styled-ui` の `heading` / `text` / `image` / `icon` /
`link` の 5 部品を合成した、画像 + 連絡先リンクのお問い合わせ用ブロックです。
Blocks セクションは新規部品を追加するものではなく、既存の Themes/Primitives
部品を組み合わせた実例集であることに注意してください（主参照は対応表 ID
R0444。取り込んだのは領域配置・部品構成の構造だけで、文言・配色・装飾・
アイコン形状は持ち込んでいません。出典の固有名・ファイル名は記載しません）。

幅 `48rem`（`md`）以上では左に角丸の画像、右に「タグライン・見出し・説明文」
と連絡先リンク 3 件（電話・メール・住所）を縦に並べた 2 列レイアウトに
なります。`48rem` 未満では画像の下に見出しとリンクが縦に積まれます。

本 Demo は静的な表示例であり、`<form>` 要素を一切持たず、データの取得・
送信・状態管理を行いません。電話・メールのリンクは `tel:`/`mailto:` の
実プロトコルリンクですが、番号・アドレスはいずれも架空値（電話は北米の
架空番号用予約域 555-01xx、メールは RFC 2606 の予約ドメイン
`example.com`）です。住所リンクの遷移先は本リポジトリへの固定外部 URL で
あり、実在の地図サービスへは接続しません。文言はすべて独自に書いた架空の
ものであり、実企業名・実クレデンシャル・PII を含みません。画像はビルド時
生成のプレースホルダー SVG（`alt=""`）です。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, li, text, ul, Node};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::image::{self, AspectRatio, ImageFit, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};

/// 住所リンクの固定外部 URL（モジュール doc「連絡先リンクの `href` 方針」
/// 節参照）。実在の地図サービスへは接続しない。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// 自作の幾何アイコン（線画。モジュール doc「アイコンは自作の単純図形」
/// 節参照）。`path` へ `fill="none"` + `stroke="currentColor"` を明示し、
/// `icon` の `<svg>` 側が固定で持つ `fill="currentColor"`（塗り面）を
/// 上書きして線画（ストローク）として描画する。
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

/// 電話アイコン（角丸長方形の端末 + 上端の短い線）。
fn phone_icon() -> Node {
    geo_icon("M6 4h6v2H8v12h4v2H6z M9 6h1 M4 10c0 6 4 10 10 10")
}

/// メールアイコン（封筒 + V 字の折り返し線）。
fn mail_icon() -> Node {
    geo_icon("M3 6h18v12H3z M3 7l9 6 9-6")
}

/// 住所アイコン（ピン: 円 + 下向きの雫形）。
fn address_icon() -> Node {
    geo_icon(
        "M12 21s-7-6.5-7-11a7 7 0 0 1 14 0c0 4.5-7 11-7 11z M12 12a2 2 0 1 0 0-4 2 2 0 0 0 0 4z",
    )
}

/// 画像領域（角丸の正方形画像。モジュール doc「使用部品」節）。
fn media() -> Node {
    div(
        vec![("class", "blocks-contact-image-info-media")],
        vec![image::image(
            &ImageProps {
                fit: ImageFit::Cover,
                aspect_ratio: AspectRatio::Square,
                shape: ImageShape::Rounded,
                ..ImageProps::new(dummy_assets::BACKGROUND_SRC, "")
            },
            vec![("data-blocks-contact-image-info-image", "")],
        )],
    )
}

/// 連絡先リンク 1 行（モジュール doc「連絡先リンクの `href` 方針」節）。
fn contact_link(href: &'static str, external: bool, glyph: Node, label: &'static str) -> Node {
    li(
        vec![],
        vec![link::root(
            href,
            &LinkProps {
                external,
                ..LinkProps::default()
            },
            vec![("data-blocks-contact-image-info-item", "")],
            vec![glyph, text(label)],
        )],
    )
}

/// テキスト領域（タグライン + 見出し + 説明文 + 連絡先リンク 3 件）。
fn info() -> Node {
    div(
        vec![("class", "blocks-contact-image-info-info")],
        vec![
            styled_text::text(
                &TextProps {
                    size: TextSize::Sm,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![("data-blocks-contact-image-info-tagline", "")],
                vec![text("お問い合わせ")],
            ),
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    ..HeadingProps::default()
                },
                vec![],
                vec![text("お気軽にご連絡ください")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text("平日 2 営業日以内にご返信します。")],
            ),
            ul(
                vec![("class", "blocks-contact-image-info-list")],
                vec![
                    contact_link("tel:+15550100", false, phone_icon(), "+1 555-0100"),
                    contact_link(
                        "mailto:hello@example.com",
                        false,
                        mail_icon(),
                        "hello@example.com",
                    ),
                    contact_link(REPO, true, address_icon(), "架空通り 1-2-3、サンプル区"),
                ],
            ),
        ],
    )
}

/// `contact-image-info` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数。DOM 順は画像 → テキスト領域（狭幅の 1 列表示で画像が上に
/// 来るようにするため）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-contact-image-info-layout")],
        vec![media(), info()],
    )
}
```

## 原案差分メモ

参照（対応表 ID R0444、出典の固有名・ファイル名は記載しません）からの
意図的な差分は次のとおりです。

- 見出しは参照側相当ではなく `h3`（ページ側が `## Demo` として `h2` を出す
  ため）にしています。
- 連絡先リンクの遷移先は、参照側の死リンク相当を `tel:`/`mailto:` の実
  プロトコルリンク（番号・アドレスは架空値）と、住所用の固定外部 URL
  （本リポジトリ）へ置き換えました。`href="#"` の死リンクは使いません。
- アイコンは lucide 等の著作物を複製しない自作の幾何アイコン（線画）に
  しています。
- 画像はビルド時生成のプレースホルダー SVG（`alt=""`）で、装飾用の背景・
  実写風の合成は持ち込んでいません。
- 文言（タグライン・見出し・説明文・連絡先ラベル）はすべて独自に書き
  直しました。
- 配色・余白・角丸は既存のテーマトークンに従っています。
- ブレークポイントは `48rem`（`md`）に固定しています。
- `id` や `aria-labelledby` は出力しません（宙に浮いた ARIA 参照・id 重複
  を構造的に避けるため）。
