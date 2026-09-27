# testimonial-centered-quote

`blockquote` / `avatar` / `icon` / `text` の合成例（既存部品のみで組んだ、
中央寄せの単一推薦文セクションです）。Blocks セクションは新規部品を追加
するものではなく、既存の Themes/Primitives 部品を組み合わせた実例集で
あることに注意してください。

中央に上部の飾り（ロゴ風アイコン・引用アイコン・星評価のいずれか、または
なし）、その下に大きな引用文、さらにその下へアバター・氏名・役職を並べる
レイアウトです。狭い幅でも中央寄せのまま、`48rem` 未満では引用文の
文字サイズを 1 段下げます。4 variant（ロゴマーク形・引用アイコン形・
ロゴバッジ形・星評価形）を静的に縦へ並記しています。星評価・ロゴバッジは
いずれも装飾扱いで、隣接する氏名テキストがアクセシブルネームを担います。
本 Demo は静的な表示例であり、`<form>` 要素を一切持たず、送信処理・データ
取得を行いません。

## Rust コード

```rust
use crate::blocks::dummy_assets::{
    AVATAR_SRC, COMPANY_NAMES, JOB_TITLES, PERSON_NAMES, TESTIMONIAL_QUOTES,
};
use fandhe_frontend_core::{div, el, p, text as core_text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarBadgeProps, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::blockquote::{self, BlockquoteVariant};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::text::{
    self as styled_text, TextProps, TextSize, TextVariant, TextWeight,
};
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// 装飾用の抽象ロゴマーク（菱形。実在ブランドのロゴを模さない自作 path）。
const LOGO_PATH: &str = "M12 2L22 12L12 22L2 12Z";

/// 装飾用の引用符アイコン（二重引用符を抽象化した自作 path）。
const QUOTE_PATH: &str =
    "M3 9c0-4 3-6 6-6v3c-2 0-3 1-3 3v1h3v7H3V9zm11 0c0-4 3-6 6-6v3c-2 0-3 1-3 3v1h3v7h-6V9z";

/// 装飾用の星アイコン（5 点星の自作 path）。
const STAR_PATH: &str =
    "M12 2l2.9 6.1 6.7.9-4.9 4.6 1.3 6.6L12 17l-5.9 3.2 1.3-6.6-4.9-4.6 6.7-.9L12 2z";

/// 上部の飾り・著者表示を切り替える 4 variant。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Variant {
    /// ロゴ風幾何アイコン + 縦積み著者（R0354 主参照）。
    Base,
    /// 引用符アイコン + 1 行キャプション（R0358）。
    QuoteIcon,
    /// アバター右下にロゴバッジ（R0359）。
    LogoBadge,
    /// 星 5 個（R1364）。
    Stars,
}

impl Variant {
    /// `data-blocks-testimonial-centered-quote-variant` の値。
    fn attr(self) -> &'static str {
        match self {
            Self::Base => "base",
            Self::QuoteIcon => "quote-icon",
            Self::LogoBadge => "logo-badge",
            Self::Stars => "stars",
        }
    }
}

/// 装飾用の幾何図形アイコン 1 個（`label: None` = `aria-hidden`）。
fn geo_icon(size: Size, d: &str) -> Node {
    icon(
        &IconProps {
            size,
            label: None,
            ..IconProps::default()
        },
        vec![],
        vec![el("path", vec![("d", d)], vec![])],
    )
}

/// 星 5 個の並び（ラッパー自体がアクセシブルネームを持つ装飾グループ）。
fn star_rating() -> Node {
    div(
        vec![
            ("role", "img"),
            ("aria-label", "5 段階中 5 の評価"),
            ("data-blocks-testimonial-centered-quote-stars", ""),
        ],
        (0..5).map(|_| geo_icon(Size::Sm, STAR_PATH)).collect(),
    )
}

/// variant ごとの上部マーク（`None` なら描画しない = `logo-badge`）。
fn top_mark(variant: Variant) -> Option<Node> {
    match variant {
        Variant::Base => Some(geo_icon(Size::Lg, LOGO_PATH)),
        Variant::QuoteIcon => Some(geo_icon(Size::Lg, QUOTE_PATH)),
        Variant::LogoBadge => None,
        Variant::Stars => Some(star_rating()),
    }
}

/// 氏名（強調）+ 役職・社名（弱色）の 2 行テキスト。
fn name_and_role(name: &str, role_title: &str, company: &str) -> Vec<Node> {
    vec![
        styled_text::text(
            &TextProps {
                weight: TextWeight::Semibold,
                ..TextProps::default()
            },
            vec![],
            vec![core_text(name)],
        ),
        styled_text::text(
            &TextProps {
                size: TextSize::Sm,
                variant: TextVariant::Muted,
                ..TextProps::default()
            },
            vec![],
            vec![core_text(format!("{role_title}, {company}"))],
        ),
    ]
}

/// アバター画像（`alt=""`、装飾扱い。氏名テキストがアクセシブルネームを
/// 担うため画像自体には意味づけしない）。
fn avatar_image() -> Node {
    avatar::image(ImageStatus::Loaded, AVATAR_SRC, "", vec![])
}

/// variant ごとの著者表示（`blockquote::caption`）。
fn author_display(variant: Variant, name: &str, role_title: &str, company: &str) -> Node {
    match variant {
        Variant::QuoteIcon => blockquote::caption(
            vec![("data-blocks-testimonial-centered-quote-inline", "")],
            name_and_role(name, role_title, company),
        ),
        Variant::LogoBadge => {
            let mut children = vec![avatar::root(
                &AvatarProps {
                    with_badge: true,
                    ..AvatarProps::default()
                },
                vec![],
                vec![
                    avatar_image(),
                    avatar::badge(
                        &AvatarBadgeProps::default(),
                        vec![],
                        vec![geo_icon(Size::Xs, LOGO_PATH)],
                    ),
                ],
            )];
            children.extend(name_and_role(name, role_title, company));
            blockquote::caption(
                vec![("class", "blocks-testimonial-centered-quote-meta")],
                children,
            )
        }
        Variant::Base | Variant::Stars => {
            let mut children = vec![avatar::root(
                &AvatarProps::default(),
                vec![],
                vec![avatar_image()],
            )];
            children.extend(name_and_role(name, role_title, company));
            blockquote::caption(
                vec![("class", "blocks-testimonial-centered-quote-meta")],
                children,
            )
        }
    }
}

/// 並記の見出し（`header_simple_bar::caption` と同型）。
fn caption_label(label: &str) -> Node {
    p(
        vec![("class", "blocks-testimonial-centered-quote-caption")],
        vec![core_text(label)],
    )
}

/// testimonial 1 件分（上部マーク + blockquote + 著者表示）を組み立てる。
fn testimonial(variant: Variant, quote: &str, name: &str, role_title: &str, company: &str) -> Node {
    let mut children = Vec::new();
    if let Some(mark) = top_mark(variant) {
        children.push(div(
            vec![("data-blocks-testimonial-centered-quote-mark", "")],
            vec![mark],
        ));
    }
    children.push(blockquote::root(
        BlockquoteVariant::default(),
        ColorPalette::default(),
        vec![("data-blocks-testimonial-centered-quote-quote", "")],
        vec![
            blockquote::content(vec![], vec![core_text(quote)]),
            author_display(variant, name, role_title, company),
        ],
    ));
    div(
        vec![
            ("class", "blocks-testimonial-centered-quote-layout"),
            (
                "data-blocks-testimonial-centered-quote-variant",
                variant.attr(),
            ),
        ],
        children,
    )
}

/// `testimonial-centered-quote` の Demo 本体。呼び出しごとに同一の `Node`
/// を返す純関数。4 variant を静的に縦に並記する。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-testimonial-centered-quote-stack")],
        vec![
            caption_label("ロゴマーク＋縦積み著者"),
            testimonial(
                Variant::Base,
                TESTIMONIAL_QUOTES[0],
                PERSON_NAMES[0],
                JOB_TITLES[0],
                COMPANY_NAMES[0],
            ),
            caption_label("引用アイコン＋1 行キャプション"),
            testimonial(
                Variant::QuoteIcon,
                TESTIMONIAL_QUOTES[1],
                PERSON_NAMES[1],
                JOB_TITLES[1],
                COMPANY_NAMES[1],
            ),
            caption_label("アバター右下にロゴバッジ"),
            testimonial(
                Variant::LogoBadge,
                TESTIMONIAL_QUOTES[2],
                PERSON_NAMES[2],
                JOB_TITLES[2],
                COMPANY_NAMES[2],
            ),
            caption_label("星評価付き"),
            testimonial(
                Variant::Stars,
                TESTIMONIAL_QUOTES[3],
                PERSON_NAMES[3],
                JOB_TITLES[3],
                COMPANY_NAMES[3],
            ),
        ],
    )
}
```

## 集約元との差分メモ

- R0354（主参照）を基準形（ロゴマーク形）とし、R0724・R1359（上にロゴ）も
  同一形として統合しました。
- R0358（引用アイコン + 1 行キャプション）は「引用アイコン形」として
  独立したインスタンスにしています。アバターは持たず、氏名・役職を
  横並びの 1 行で表示します。
- R0359（アバター右下にロゴバッジ）は「ロゴバッジ形」として独立した
  インスタンスにしています。バッジは装飾扱いで、隣接する氏名テキストが
  アクセシブルネームを担います。
- R1364（星 5 個）は「星評価形」として独立したインスタンスにしています。
  星は装飾扱いで、ラッパー自体に `role="img"` + `aria-label` を付与して
  います。
- 参照元の文言・配色・実ブランドロゴ・実写真は持ち込まず、架空の引用文・
  人名・役職・社名（`crate::blocks::dummy_assets`）と自作の幾何図形
  アイコンへ置き換えました。

関連情報: [Blockquote](../themes/blockquote.md) /
[Avatar](../themes/avatar.md) / [Icon](../themes/icon.md) /
[Text](../themes/text.md)
