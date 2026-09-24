# banner-floating-card

`callout` / `icon` / `link` / `button` を合成した、浮いたカード型の告知の
合成例です。実運用では `position: fixed` + ページ端の padding を利用者が
用意することを想定しますが、この Demo は複数インスタンスを同一ページに
並べて掲示する構造のため、各インスタンスを「疑似ビューポート」（固定高の
`position: relative` ボックス）の中に置き、その内側でのみ
`position: absolute` の浮動レイヤで上端/下端へ貼り付けています。実際の
画面全体を基準にした `position: fixed` はこの Demo・生成 CSS のいずれにも
登場しません。

`<form>` を持たず、閉じるボタンは `type="button"` のまま送信先・削除処理
を持ちません。実際の開閉・表示の永続化は利用者の Rust/JS コードで実装
してください（`docs/policy/intentional-non-adoption.md` §3.25）。リンク先は
本リポジトリの実在 URL のみを使い、`href="#"` は使いません。

`40rem` 未満（Demo 枠幅ではなくビューポート幅が基準）では、浮動レイヤの
内側余白とカードの角丸・影を外し、画面端いっぱいに貼り付く全幅表示に
なります。

## Rust コード

```rust
use fandhe_frontend_core::{div, el, span, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::callout::{self, CalloutProps};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::Size;

/// 「Read the notes」リンクの遷移先。実在する自リポジトリの URL の定数
/// のみを使い、`href="#"` は使わない（footer 系 block と同型の判断）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// 先頭の告知アイコン（自作の単純な星形、`aria-hidden="true"` の装飾用途）。
fn announcement_icon() -> Node {
    icon(
        &IconProps {
            size: Size::Sm,
            ..IconProps::default()
        },
        vec![],
        vec![el(
            "path",
            vec![(
                "d",
                "M12 3l2.2 5.8L20 11l-5.8 2.2L12 19l-2.2-5.8L4 11l5.8-2.2z",
            )],
            vec![],
        )],
    )
}

/// 末尾の矢印アイコン（自作の単純な矢印、`aria-hidden="true"` の装飾用途）。
fn arrow_icon() -> Node {
    icon(
        &IconProps {
            size: Size::Sm,
            ..IconProps::default()
        },
        vec![],
        vec![el("path", vec![("d", "M5 12h13M13 6l6 6-6 6")], vec![])],
    )
}

/// カード本体（`callout::root` + アイコン + 本文 + 閉じるボタン）。
///
/// `dismiss_label` はインスタンスごとに閉じるボタンのアクセシブル名を
/// 変える（同一ページに複数カードを並べるため重複を避ける、`sidebar_07`
/// 等の既存慣行）。
fn card(dismiss_label: &'static str) -> Node {
    callout::root(
        &CalloutProps::default(),
        vec![("data-blocks-banner-floating-card-card", "")],
        vec![
            callout::icon(vec![], vec![announcement_icon()]),
            callout::text(
                vec![("class", "blocks-banner-floating-card-text")],
                vec![
                    span(
                        vec![("class", "blocks-banner-floating-card-badge")],
                        vec![text("New")],
                    ),
                    text("Blocks now ship with category navigation."),
                    link::root(
                        REPO,
                        &LinkProps::default(),
                        vec![("data-blocks-banner-floating-card-link", "")],
                        vec![text("Read the notes"), arrow_icon()],
                    ),
                ],
            ),
            button::close_button(
                &ButtonProps {
                    variant: ButtonVariant::Ghost,
                    size: Size::Sm,
                    ..ButtonProps::default()
                },
                dismiss_label,
                vec![("data-blocks-banner-floating-card-close", "")],
            ),
        ],
    )
}

/// Demo 1 セル分（キャプション + 疑似ビューポート + 浮動レイヤ + カード）。
///
/// `placement` は `"top"`/`"bottom"` のいずれか（[`LAYOUT_CSS`] の
/// `data-blocks-banner-floating-card-placement` セレクタと対応）。
/// `centered` が `true` のとき、浮動レイヤへ
/// `data-blocks-banner-floating-card-align="center"` を追加で付与する
/// （R0762 相当、中央寄せ）。
fn cell(
    caption: &'static str,
    placement: &'static str,
    centered: bool,
    dismiss_label: &'static str,
) -> Node {
    let mut layer_attrs = vec![("data-blocks-banner-floating-card-placement", placement)];
    if centered {
        layer_attrs.push(("data-blocks-banner-floating-card-align", "center"));
    }

    div(
        vec![("class", "blocks-banner-floating-card-cell")],
        vec![
            span(
                vec![("class", "blocks-banner-floating-card-caption")],
                vec![text(caption)],
            ),
            div(
                vec![("class", "blocks-banner-floating-card-viewport")],
                vec![div(
                    vec![("class", "blocks-banner-floating-card-layer")]
                        .into_iter()
                        .chain(layer_attrs)
                        .collect(),
                    vec![card(dismiss_label)],
                )],
            ),
        ],
    )
}

/// `banner-floating-card` の Demo 本体（3 セル: 下端・下端中央寄せ・上端）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-banner-floating-card-grid")],
        vec![
            cell("Bottom", "bottom", false, "Dismiss announcement"),
            cell("Bottom · centered", "bottom", true, "Dismiss announcement"),
            cell("Top", "top", false, "Dismiss announcement"),
        ],
    )
}
```

## 差分メモ

参照は対応表 ID R0761（基準形、下端に浮くバー）を中心に、R0409（上端配置）・
R0762（中央寄せ）の 2 件を同じ Demo の中へ集約します。参照元の文言・配色・
装飾・アイコンは持ち込まず、既存トーンでデモ文言を独自に書いています。
実装上の主な差分は次のとおりです。

- **固定配置は Demo 枠内の相対配置で表す**: 実運用の `position: fixed` は
  ビューポート全体を基準にするため、複数インスタンスを 1 ページへ並べる
  この Demo では使えません。代わりに固定高の疑似ビューポート
  （`position: relative` + `overflow: hidden`）の内側だけで
  `position: absolute` の浮動レイヤを上端/下端へ貼り付けています。
- **3 件を 1 つの Demo へ集約**: 下端（基準形）・下端中央寄せ・上端の
  3 セルを並べて掲示し、配置・中央寄せの違いを Demo と本原稿の両方から
  読み取れるようにしています。
- **ランドマーク role を付けない**: 1 ページに複数の告知カードを掲示する
  ため、`role="banner"` 等のランドマーク role は付与していません。
- **`<form>` 化・実送信をしない**: 閉じるボタンは `type="button"` のまま
  送信先を持たず、リンクは実在する自リポジトリの URL の定数のみを使い
  `href="#"` は使いません。
- **文言・配色は独自**: 参照元の配色・装飾・アイコンは踏襲せず、既存の
  `callout`/`link`/`button` のトーンに合わせたデモ文言・自作の単純幾何
  アイコンを新規に書いています。
