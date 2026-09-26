# cta-centered

`fandhe-frontend-pre-styled-ui` の `heading` / `text` / `button` / `badge` /
`card` 部品を合成した、中央寄せの CTA バナーです。Blocks セクションは新規
部品を追加するものではなく、既存の Themes/Primitives 部品を組み合わせた
実例集であることに注意してください（主参照は対応表 ID R0878、配色・訴求点
の差分は R0068 / R0069 / R0446 / R0452 / R0876 / R0877 / R0879 / R0880 を
集約。出典の固有名・ファイル名は記載しません）。

見出し・リード文・ボタン列を常に中央寄せで縦に積みます。ボタン列は
`40rem` 未満の幅では縦積み、`40rem` 以上で横並び・中央寄せに切り替わり
ます。配色違いとして 5 つのインスタンスを並べています。

1. 背景なし・見出し + リード文 + ボタン 2 個。
2. 淡色面 + バッジ + 見出し + リード文 + ボタン 1 個。
3. アクセント色の全面背景 + 自作のロゴ枠 + 2 行構成の見出し + ボタン 2 個。
4. 前景色と背景色を反転したパネル（グラデーション装飾つき）+ ボタン 2 個。
5. アクセント色に塗った浮いたカードに収めた形 + ボタン 2 個。

本 Demo は静的な表示例であり、`<form>` 要素を一切持たず、データの取得・
送信・状態管理を行いません。ボタンはすべて `type="button"` のまま送信先を
持ちません。文言はすべて独自に書いた架空のものであり、実企業名・実
クレデンシャル・PII を含みません。

## Rust コード

```rust
use fandhe_frontend_core::{div, span, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps, BadgeVariant};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps, CardVariant};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};
use fandhe_frontend_pre_styled_ui::Size;

/// Primary/Secondary の 2 ボタン列（モジュール doc「tone 上書きの詳細度
/// (0,4,0)」節）。`data-blocks-cta-centered-primary`/`-secondary` 属性を
/// 付与し、tone 面上での配色上書きの対象を識別できるようにする。
/// `secondary` を `None` にすると Primary 1 個のみになる（`badge` tone）。
fn actions(primary_label: &'static str, secondary_label: Option<&'static str>) -> Node {
    let mut children = vec![button::button(
        &ButtonProps {
            size: Size::Lg,
            ..ButtonProps::default()
        },
        vec![("data-blocks-cta-centered-primary", "")],
        vec![text(primary_label)],
    )];
    if let Some(label) = secondary_label {
        children.push(button::button(
            &ButtonProps {
                variant: ButtonVariant::Outline,
                size: Size::Lg,
                ..ButtonProps::default()
            },
            vec![("data-blocks-cta-centered-secondary", "")],
            vec![text(label)],
        ));
    }
    div(vec![("class", "blocks-cta-centered-actions")], children)
}

/// リード文（[`TextVariant::Plain`]。`Muted` は自身に色を持つため、
/// `accent`/`dark` 面で継承色を上書きしコントラストが崩れる、
/// [`super::cta_split_actions`] と同じ判断）。
fn lead(label: &'static str) -> Node {
    styled_text::text(
        &TextProps {
            variant: TextVariant::Plain,
            ..TextProps::default()
        },
        vec![],
        vec![text(label)],
    )
}

/// セクション共通ラッパー（モジュール doc「レイアウトとレスポンシブ」
/// 節）。子要素は `Vec<Node>` で直接受け取り、`gap` が効く flex コンテナの
/// 直接の子を複数にする（[`super::cta_split_actions::row`] と同じ判断）。
fn section(tone: &'static str, children: Vec<Node>) -> Node {
    div(vec![("data-blocks-cta-centered-tone", tone)], children)
}

/// 2 行構成の見出し（`accent` tone。`h3` の内容モデルは phrasing content
/// のため、行の区切りには `div` ではなく `span` + `display: block` を
/// 用いる）。
fn two_line_heading(line1: &str, line2: &str) -> Node {
    heading(
        HeadingLevel::H3,
        &HeadingProps {
            size: HeadingSize::Xl2,
            ..HeadingProps::default()
        },
        vec![],
        vec![
            span(
                vec![("class", "blocks-cta-centered-line")],
                vec![text(line1)],
            ),
            span(
                vec![("class", "blocks-cta-centered-line")],
                vec![text(line2)],
            ),
        ],
    )
}

/// 自作のロゴ枠（モジュール doc「ロゴ枠は自作の抽象幾何図形」節）。
/// 実ブランドを模さない角丸枠 + 抽象幾何のみを CSS で描く。
fn logo_mark() -> Node {
    span(
        vec![
            ("class", "blocks-cta-centered-logo"),
            ("aria-hidden", "true"),
        ],
        vec![],
    )
}

/// 1 個目: 基準形（背景なし、モジュール doc「5 インスタンスで配色差分を
/// 表現する」節）。対応: R0878（主参照）、R0876（左寄せは配置差のみのため
/// 統合せず、原案差分メモへ記す）。
fn instance_plain() -> Node {
    section(
        "plain",
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    ..HeadingProps::default()
                },
                vec![],
                vec![text("新しい働き方を、今日から始めましょう")],
            ),
            lead("チームの規模を問わず、導入したその日から成果を実感できます。"),
            actions("今すぐ始める", Some("詳しく見る")),
        ],
    )
}

/// 2 個目: 淡色面 + バッジ。対応: R0068。
fn instance_badge() -> Node {
    section(
        "badge",
        vec![
            badge::badge(
                &BadgeProps {
                    variant: BadgeVariant::Subtle,
                    ..BadgeProps::default()
                },
                vec![],
                vec![text("新機能")],
            ),
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    ..HeadingProps::default()
                },
                vec![],
                vec![text("β版のご利用登録を受け付けています")],
            ),
            lead("先着枠のご案内は登録いただいた方から順にお送りします。"),
            actions("登録する", None),
        ],
    )
}

/// 3 個目: アクセント色の全面背景 + ロゴ枠 + 2 行見出し。対応: R0446
/// （反転配色）、R0880（配色差のみで統合）、R0069（ロゴ枠を代用）。
fn instance_accent() -> Node {
    section(
        "accent",
        vec![
            logo_mark(),
            two_line_heading("導入をご検討の方へ、", "個別相談を承ります"),
            lead("担当チームが要件をヒアリングし、最適な構成をご提案します。"),
            actions("相談する", Some("資料をもらう")),
        ],
    )
}

/// 4 個目: 反転配色パネル + グラデ装飾。対応: R0877（放射グラデは除外し
/// 面装飾のみ実装）、R0879（装飾の有無の差のみ）。
fn instance_dark() -> Node {
    section(
        "dark",
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    ..HeadingProps::default()
                },
                vec![],
                vec![text("次の四半期の計画を、いま固めませんか")],
            ),
            lead("導入事例をもとに、貴社に合わせた進め方をご提案します。"),
            actions("事例を見る", Some("問い合わせる")),
        ],
    )
}

/// 5 個目: アクセント色の浮いたカード。対応: R0452。カード面の余白は
/// [`card::body`] へ委ね、内側 section の padding は CSS 側で 0 に上書き
/// する（モジュール doc「tone 上書きの詳細度 (0,4,0)」節）。
fn instance_card() -> Node {
    let inner = section(
        "card",
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    ..HeadingProps::default()
                },
                vec![],
                vec![text("年間プランなら 2 か月分お得です")],
            ),
            lead("月額プランからいつでも切り替えられます。"),
            actions("年間プランを見る", Some("月額プランのまま続ける")),
        ],
    );
    card::root(
        CardProps {
            variant: CardVariant::Elevated,
            ..CardProps::default()
        },
        vec![("data-blocks-cta-centered-tone", "card")],
        vec![card::body(vec![], vec![inner])],
    )
}

/// `cta-centered` の Demo 本体。呼び出しごとに同一の `Node` を返す純関数
/// （グリッド用ラッパーの class を [`Block::demo_class`] と別名にする
/// 理由をモジュール doc「`drop_class_attr` と CSS フックの選び方」節
/// 参照）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-cta-centered-stack")],
        vec![
            instance_plain(),
            instance_badge(),
            instance_accent(),
            instance_dark(),
            instance_card(),
        ],
    )
}
```

**原案差分メモ**

参照（対応表 ID R0878 が主参照、R0068 / R0069 / R0446 / R0452 / R0876 /
R0877 / R0879 / R0880 を配色・訴求点の差分として集約。出典の固有名・
ファイル名は記載しません）からの意図的な差分は次のとおりです。

- R0878 の中央寄せ基準形を `plain` tone として実装しました。背景を持たず、
  見出し + リード文 + ボタン 2 個で構成します。
- R0876 は見出し左寄せの原案ですが、配置差のみのため個別インスタンス化
  せず、`plain` tone に統合しています（実装は中央寄せのみ）。
- R0068 の淡色面 + バッジ構成を `badge` tone として実装しました。
- R0446 の反転配色と R0880 の配色差は、アクセント色の全面背景として
  `accent` tone へ統合しました。
- R0069 のロゴ・ブランドマークは自作の抽象幾何図形（角丸枠 + グラデーション）
  で代用しています。実ブランドのロゴ・商標は持ち込んでいません。
- R0877 の放射グラデーション装飾は除外し、`dark` tone には線形グラデーション
  の面装飾のみを実装しました。R0879 とは装飾の有無のみが差分です。
- R0452 のアクセント色の浮いたカードは `card` tone として、`card`（
  `CardVariant::Elevated`）のカード面をアクセント色へ上書きして表現しま
  した。
- 見出しレベルはすべて `h3` に固定しています（ページ側が `## Demo` として
  `h2` を出すため）。
- `id` や `aria-labelledby` は出力しません（宙に浮いた ARIA 参照・id 重複
  を構造的に避けるため）。
- 文言（見出し・リード文・ボタンラベル）・ロゴ枠の形・配色はすべて独自に
  書き直しました。実在ブランドの文言・配色・装飾・ロゴは持ち込んでいません。
