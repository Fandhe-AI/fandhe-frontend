# hero-search

`badge` / `heading` / `text` / `input-group` / `input` / `button` / `link` /
`icon` / `visually-hidden` を合成した、検索ボックス中心のヒーローの合成例です。

`<form>` を持たず、検索ボタンは `type="button"` のまま送信先・検索処理を
持ちません。実際の検索・遷移は利用者の Rust/JS コードで実装してください
（`docs/policy/intentional-non-adoption.md` §3.25）。

3 形態を上から順に並記しています。基準形は eyebrow badge + 見出し + リード文
+ 検索欄 + よく見られるトピックのリンク列、検索欄のみの最小形は見出しの下に
検索欄だけを置いたもの、タグライン付き見出しは `>= 48rem` で見出し左・検索欄
右の 2 カラム（`< 48rem` は縦積み）です。

## Rust コード

```rust
use fandhe_frontend_core::{div, el, p, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::field::{
    self, FieldIds, FieldOrientation, FieldProps, FieldRootProps,
};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::input::{self, InputProps};
use fandhe_frontend_pre_styled_ui::input_group::{self, InputGroupAlign, InputGroupProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps, LinkVariant};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::visually_hidden;
use fandhe_frontend_pre_styled_ui::Size;

/// 検索アイコン（円 + 柄の 2 path。装飾のため `aria-hidden="true"`、
/// モジュール doc「検索アイコンは自作の幾何アイコン」節）。
fn search_icon() -> Node {
    icon(
        &IconProps {
            size: Size::Sm,
            ..IconProps::default()
        },
        vec![],
        vec![el(
            "path",
            vec![
                ("d", "M11 4a7 7 0 1 0 0 14 7 7 0 0 0 0-14zm9 17-5.2-5.2"),
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

/// 検索欄 + 検索ボタンの入力グループ（モジュール doc「可視ラベルの
/// 代わりに」節）。`instance` はインスタンスごとに異なる `id` の suffix
/// （モジュール doc「`id` は 3 インスタンス分すべて別値にする」節）。
fn search_group(instance: &'static str, placeholder: &'static str) -> Node {
    let field_id = format!("blocks-hero-search-query-{instance}");
    let query_field = FieldProps {
        id: &field_id,
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: false,
    };
    let group_props = InputGroupProps {
        disabled: false,
        invalid: false,
    };

    div(
        vec![("class", "blocks-hero-search-row")],
        vec![field::root(
            &FieldRootProps {
                orientation: FieldOrientation::Vertical,
            },
            &query_field,
            vec![("data-blocks-hero-search-field", "")],
            vec![
                visually_hidden::root(
                    vec![],
                    vec![field::label(
                        &query_field,
                        vec![],
                        vec![text("ヘルプセンターを検索")],
                    )],
                ),
                input_group::root(
                    &group_props,
                    vec![("data-blocks-hero-search-group", "")],
                    vec![
                        input_group::addon(
                            InputGroupAlign::InlineStart,
                            &group_props,
                            vec![],
                            vec![search_icon()],
                        ),
                        input::input(
                            &InputProps::default(),
                            &query_field,
                            vec![
                                ("type", "search"),
                                ("autocomplete", "off"),
                                ("placeholder", placeholder),
                            ],
                        ),
                        input_group::addon(
                            InputGroupAlign::InlineEnd,
                            &group_props,
                            vec![],
                            vec![button::button(
                                &ButtonProps::default(),
                                vec![("data-blocks-hero-search-submit", "")],
                                vec![text("検索する")],
                            )],
                        ),
                    ],
                ),
            ],
        )],
    )
}

/// よく見られるトピックへのリンク列（`error_page_popular_links` と同型、
/// モジュール doc「トピックリンクは固定パス」節。`href="#"` は使わない）。
fn popular_topics() -> Node {
    let topics: [(&str, &str); 4] = [
        ("はじめに", "../../guides/"),
        ("料金プラン", "../../themes/badge/"),
        ("連携機能", "../../themes/input-group/"),
        ("キーボードショートカット", "../../primitives/"),
    ];
    div(
        vec![("class", "blocks-hero-search-topics")],
        std::iter::once(styled_text::text(
            &TextProps {
                size: TextSize::Sm,
                variant: TextVariant::Muted,
                ..TextProps::default()
            },
            vec![],
            vec![text("よく見られるトピック:")],
        ))
        .chain(topics.iter().map(|(label, href)| {
            link::root(
                href,
                &LinkProps {
                    variant: LinkVariant::Underline,
                    ..LinkProps::default()
                },
                vec![("data-blocks-hero-search-topic", "")],
                vec![text(*label)],
            )
        }))
        .collect(),
    )
}

/// 1 インスタンス分の「ビューポート枠」（`banner_cookie_consent::stage` と
/// 同型のキャプション付き併記）。
fn stage(caption: &'static str, inner: Node) -> Node {
    div(
        vec![("class", "blocks-hero-search-stage")],
        vec![
            p(
                vec![("class", "blocks-hero-search-caption")],
                vec![text(caption)],
            ),
            inner,
        ],
    )
}

/// 基準形（R0131）: eyebrow badge + 見出し + リード文 + 検索欄 +
/// トピックリンク列。
fn variant_base() -> Node {
    div(
        vec![("class", "blocks-hero-search-inner")],
        vec![
            badge::badge(
                &BadgeProps::default(),
                vec![("data-blocks-hero-search-eyebrow", "")],
                vec![text("ヘルプセンター")],
            ),
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl3,
                    ..HeadingProps::default()
                },
                vec![],
                vec![text("お探しの答えを、すぐに見つける")],
            ),
            styled_text::text(
                &TextProps {
                    size: TextSize::Lg,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text("記事・ガイド・リリースノートを横断して検索できます。")],
            ),
            search_group("base", "記事やガイドを検索"),
            popular_topics(),
        ],
    )
}

/// 検索欄のみの最小形（R0130）: 見出しの下に検索欄だけを置く。
fn variant_minimal() -> Node {
    div(
        vec![("class", "blocks-hero-search-inner")],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl3,
                    ..HeadingProps::default()
                },
                vec![],
                vec![text("何をお探しですか?")],
            ),
            search_group("minimal", "キーワードを入力"),
        ],
    )
}

/// タグライン付き見出し + 検索欄（R0192）: `>= 48rem` は見出し左・検索欄右
/// の 2 カラム、`< 48rem` は縦積み（[`LAYOUT_CSS`] の
/// `data-blocks-hero-search-layout="split"`）。
fn variant_tagline() -> Node {
    div(
        vec![
            ("class", "blocks-hero-search-inner"),
            ("data-blocks-hero-search-layout", "split"),
        ],
        vec![
            div(
                vec![("class", "blocks-hero-search-tagline-copy")],
                vec![
                    styled_text::text(
                        &TextProps {
                            size: TextSize::Sm,
                            variant: TextVariant::Muted,
                            ..TextProps::default()
                        },
                        vec![("data-blocks-hero-search-tagline", "")],
                        vec![text("ドキュメントを検索")],
                    ),
                    heading(
                        HeadingLevel::H3,
                        &HeadingProps {
                            size: HeadingSize::Xl2,
                            ..HeadingProps::default()
                        },
                        vec![],
                        vec![text("必要な情報に、まっすぐたどり着く")],
                    ),
                ],
            ),
            search_group("tagline", "ページを検索"),
        ],
    )
}

/// `hero-search` の Demo 本体。呼び出しごとに同一の `Node` を返す純関数
/// （モジュール doc「3 形を 1 つの Demo に並記する」節）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-hero-search-layout")],
        vec![
            stage("基準形（R0131）", variant_base()),
            stage("検索欄のみ（R0130）", variant_minimal()),
            stage("タグライン付き見出し（R0192）", variant_tagline()),
        ],
    )
}
```

## 集約元との差分メモ

- 集約元カタログの R0131（基準形: eyebrow badge + 見出し + リード文 + 検索欄
  + よく見られるトピックのリンク列）・R0130（検索欄のみの最小形）・
  R0192（タグライン付き見出し + 検索入力、旧 section-heading-with-input）
  の 3 形態を、Demo では上から「基準形（R0131）」「検索欄のみ（R0130）」
  「タグライン付き見出し（R0192）」の順にキャプション付きで併記しています。
- 3 形態とも検索欄自体の構成（虫眼鏡アイコン + テキスト入力 + 検索ボタン）
  は共通の `input-group` 合成です。
- タグライン付き見出し（R0192）のみ `>= 48rem` で見出し左・検索欄右の
  2 カラムに分かれ、`< 48rem` では縦積みになります。他の 2 形態は常に
  中央寄せの 1 カラムです。
- 検索処理・検索結果への遷移は本 Demo には含まれません。検索ボタンは
  `type="button"` のまま送信先を持たず、実際の実装は利用者側の Rust/JS
  コード（wasm-full 等）で行ってください。
- 文言・配色・装飾・アイコンは参照元から転記せず、独自に作成しています。
