# content-article

`badge` / `heading` / `text` / `image` / `avatar` / `blockquote` / `separator`
の 7 部品を合成した、1 列の記事本文ブロックです。新しい UI 部品は追加して
いません。

静的な表示例であり、記事データの取得・整形・永続化は行いません。実際の
記事取得・レンダリングロジックの実装は利用者側の Rust コードに委ねます
（`docs/policy/intentional-non-adoption.md` §3.25 の責務境界）。

画像・人名は Blocks 共通のダミー素材ヘルパ（`dummy_assets`、docs サイト
内部の非公開ヘルパ）による架空のプレースホルダーです。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, p, span, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::blockquote::{self, BlockquoteVariant};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::image::{self, AspectRatio, ImageProps};
use fandhe_frontend_pre_styled_ui::separator::{self, SeparatorProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::ColorPalette;

/// メタ行（カテゴリ badge + 日付 + 区切り記号 + 読了時間）。
fn meta_row(category: &str, date: &str, read_minutes: u32) -> Node {
    div(
        vec![("class", "blocks-content-article-meta")],
        vec![
            badge::badge(
                &BadgeProps::default(),
                vec![("data-blocks-content-article-category", "")],
                vec![text(category)],
            ),
            styled_text::text(
                &TextProps {
                    size: TextSize::Sm,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(date)],
            ),
            span(vec![("aria-hidden", "true")], vec![text("\u{00B7}")]),
            styled_text::text(
                &TextProps {
                    size: TextSize::Sm,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(format!("{read_minutes} 分で読了"))],
            ),
        ],
    )
}

/// 著者行（fallback イニシャルの avatar + 氏名 + 肩書）。
fn byline(name: &str, role: &str) -> Node {
    let initials: String = name
        .split_whitespace()
        .filter_map(|part| part.chars().next())
        .collect();
    div(
        vec![("class", "blocks-content-article-byline")],
        vec![
            avatar::root(
                &AvatarProps::default(),
                vec![("data-blocks-content-article-avatar", "")],
                vec![avatar::fallback(
                    ImageStatus::Error,
                    vec![],
                    vec![text(initials)],
                )],
            ),
            div(
                vec![],
                vec![
                    styled_text::text(
                        &TextProps {
                            size: TextSize::Sm,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text(name)],
                    ),
                    styled_text::text(
                        &TextProps {
                            size: TextSize::Sm,
                            variant: TextVariant::Muted,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text(role)],
                    ),
                ],
            ),
        ],
    )
}

/// カバー画像（16:9、`data-blocks-content-article-cover` で全幅版と
/// 中央寄せ版の両方から CSS フックを掛ける）。
fn cover(alt: &str) -> Node {
    image::image(
        &ImageProps {
            aspect_ratio: AspectRatio::Video,
            ..ImageProps::new(dummy_assets::SCREENSHOT_SRC, alt)
        },
        vec![("data-blocks-content-article-cover", "")],
    )
}

/// 記事中の引用（R0873。出典を caption パーツで示す）。
fn pull_quote() -> Node {
    blockquote::root(
        BlockquoteVariant::default(),
        ColorPalette::default(),
        vec![("data-blocks-content-article-quote", "")],
        vec![
            blockquote::content(
                vec![],
                vec![p(
                    vec![],
                    vec![text(
                        "小さく試して確かめてから広げる、という順番だけは崩さない。\
                         それが結局いちばん早い。",
                    )],
                )],
            ),
            blockquote::caption(
                vec![],
                vec![text("— Ola Bergström、Operations Coordinator")],
            ),
        ],
    )
}

/// キャプション付き図版（R0873）。`image` パーツはキャプションを持たない
/// ため、`<figure>`/`<figcaption>` を素の HTML 意味論で組み立てる。
fn figure(alt: &str, caption: &str) -> Node {
    el(
        "figure",
        vec![("class", "blocks-content-article-figure")],
        vec![
            image::image(&ImageProps::new(dummy_assets::PRODUCT_SRC, alt), vec![]),
            el(
                "figcaption",
                vec![("class", "blocks-content-article-figcaption")],
                vec![text(caption)],
            ),
        ],
    )
}

/// 記事ヘッダー（メタ行 + 見出し + リード文 + 著者行）。中央寄せ版・全幅
/// 版の両方が共用する。
fn article_header() -> Node {
    div(
        vec![("class", "blocks-content-article-header")],
        vec![
            meta_row("運用", "2026-09-18", 7),
            heading::heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl3,
                    ..HeadingProps::default()
                },
                vec![],
                vec![text("小さく試して確かめる、という順番を崩さない理由")],
            ),
            styled_text::text(
                &TextProps::default(),
                vec![],
                vec![text(
                    "大きな変更ほど、いきなり全体へ適用したくなる。\
                     それでも一度立ち止まり、小さな範囲で確かめてから広げる \
                     運用に切り替えた経緯をまとめました。",
                )],
            ),
            byline("Ola Bergström", "Operations Coordinator"),
        ],
    )
}

/// 記事本文（本文段落 → 小見出し → 段落 → 引用 → 図版 → 区切り → 結び）。
/// 中央寄せ版・全幅版の両方が共用する。
fn article_body() -> Node {
    div(
        vec![("class", "blocks-content-article-body")],
        vec![
            styled_text::text(
                &TextProps::default(),
                vec![],
                vec![text(
                    "以前は変更のたびに全チームへ一斉に展開していましたが、\
                     手戻りが発生すると影響範囲が大きくなりすぎるという \
                     課題がありました。",
                )],
            ),
            styled_text::text(
                &TextProps::default(),
                vec![],
                vec![text(
                    "そこで、まず 1 チームだけで 2 週間試し、問題が無ければ \
                     翌週に対象を広げるという段階的な進め方へ変更しました。",
                )],
            ),
            heading::heading(
                HeadingLevel::H4,
                &HeadingProps::default(),
                vec![],
                vec![text("小さく始めたことで見えたもの")],
            ),
            styled_text::text(
                &TextProps::default(),
                vec![],
                vec![text(
                    "小さな範囲で試すと、想定していなかった手順の抜けや \
                     説明不足に早い段階で気付けます。広げる前に直せるため、\
                     結果として全体への展開が速く終わるようになりました。",
                )],
            ),
            pull_quote(),
            figure(
                "段階展開の進み方を示すプレースホルダー画像",
                "1 チーム → 対象チーム拡大 → 全体展開、の 3 段階で進める。",
            ),
            separator::separator(
                &SeparatorProps::default(),
                vec![("data-blocks-content-article-separator", "")],
            ),
            styled_text::text(
                &TextProps {
                    size: TextSize::Sm,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(
                    "次回は、段階展開のチェックリストをどう運用しているかを \
                     紹介します。",
                )],
            ),
        ],
    )
}

/// 基準形（R0631）: 中央寄せ 1 列に、列幅のカバー画像 + ヘッダー + 本文。
fn instance_centered() -> Node {
    div(
        vec![("class", "blocks-content-article-centered")],
        vec![
            article_header(),
            cover("記事カバー画像のプレースホルダー"),
            article_body(),
        ],
    )
}

/// 全幅バリエーション（R0632）: ヘッダー・本文は読書幅で左寄せのまま、
/// カバー画像だけを外枠いっぱいの全幅にする。
fn instance_full_bleed() -> Node {
    div(
        vec![("class", "blocks-content-article-full-bleed")],
        vec![
            div(
                vec![("class", "blocks-content-article-full-bleed-header")],
                vec![article_header()],
            ),
            cover("記事カバー画像のプレースホルダー（全幅）"),
            div(
                vec![("class", "blocks-content-article-full-bleed-body")],
                vec![article_body()],
            ),
        ],
    )
}

/// `content-article` の Demo 本体（基準形と全幅バリエーションの 2 インス
/// タンス併記、モジュール doc「2 インスタンス併記の理由」節参照）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-content-article")],
        vec![
            styled_text::text(
                &TextProps {
                    size: TextSize::Sm,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text("Centered article")],
            ),
            instance_centered(),
            styled_text::text(
                &TextProps {
                    size: TextSize::Sm,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text("Full-bleed cover, left-aligned body")],
            ),
            instance_full_bleed(),
        ],
    )
}
```

## 差分メモ

参照は対応表 ID R0631・R0632・R0873 の 3 件を集約元とします。参照元の
文言・配色・装飾・アイコンは持ち込まず、構造（領域配置・部品構成）のみを
取り、デモ文言は独自の日本語で書いています。

- **R0631（基準形）**: 中央寄せ 1 列に、ヘッダー（メタ行・見出し・リード
  文・著者行）→ 列幅のカバー画像 → 本文を並べます。1 インスタンス目
  （Centered article）で示します。
- **R0632（全幅バリエーション）**: カバー画像を外枠いっぱいの全幅にし、
  見出しと本文は読書幅で左寄せのまま据え置きます。2 インスタンス目
  （Full-bleed cover, left-aligned body）で示します。
- **R0873（引用・図版）**: 本文中に出典 caption 付きの引用（`blockquote`）
  と、キャプション付きの図版（`<figure>`/`<figcaption>` の素の HTML 意味論
  で組み立て）を含みます。両インスタンスの本文に共通で含めています。
- **2 インスタンス併記**: docs サイトは JS ハイドレーションを行わないため
  （`blog-featured-article`/`footer-newsletter` と同型）、基準形と全幅
  バリエーションを静的に併記しています。片方を選んで JS で切り替える
  機構は持ちません。
- **新規 `data-*` 語彙・新規部品は追加していません**: 既存の 7 部品と
  `drop_class_attr` を考慮した `data-blocks-content-article-*` 属性による
  CSS フックのみで構成しています。
