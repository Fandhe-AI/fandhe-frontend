# cta-split-actions

`fandhe-frontend-pre-styled-ui` の `heading` / `text` / `button` / `card` /
`icon` 部品を合成した、見出しとボタン列を左右両端へ揃える CTA です。Blocks
セクションは新規部品を追加するものではなく、既存の Themes/Primitives 部品
を組み合わせた実例集であることに注意してください（主参照は対応表 ID
R0881、配色・訴求ポイントの差分は R0067 / R0071 / R0447 / R0451 / R0882
を集約。出典の固有名・ファイル名は記載しません）。

`64rem` 未満の幅では見出し領域の下にボタン列を縦に積み、`64rem` 以上で
見出しとボタン列を横並び・両端揃え（`justify-content: space-between`）に
切り替えます。配色違いとして 4 つのインスタンスを並べています。

1. 背景なし・2 行構成の見出し（2 行目のみアクセント色）+ ボタン 2 個。
2. 淡色（ブランド寄り）の面 + 訴求ポイント 3 点（チェックアイコン付き）+
   ボタン 2 個。
3. アクセント配色を反転した面 + 補足文 1 行 + ボタン 2 個。
4. アクセント色に塗った浮いたカードに収めた形 + ボタン 2 個。

本 Demo は静的な表示例であり、`<form>` 要素を一切持たず、データの取得・
送信・状態管理を行いません。ボタンはすべて `type="button"` のまま送信先を
持ちません。文言はすべて独自に書いた架空のものであり、実企業名・実
クレデンシャル・PII を含みません。

## Rust コード

```rust
use fandhe_frontend_core::{div, el, span, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps, CardVariant};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};
use fandhe_frontend_pre_styled_ui::Size;

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

/// 訴求ポイント一覧で使うチェックマーク風アイコン。
fn check_icon() -> Node {
    geo_icon("M5 13l4 4L19 7")
}

/// Primary/Secondary の 2 ボタン列（モジュール doc「tone 上書きの詳細度
/// (0,4,0)」節）。`data-blocks-cta-split-actions-primary`/`-secondary`
/// 属性を付与し、tone 面上での配色上書きの対象を識別できるようにする。
fn actions(primary_label: &'static str, secondary_label: &'static str) -> Node {
    div(
        vec![("class", "blocks-cta-split-actions-actions")],
        vec![
            button::button(
                &ButtonProps {
                    size: Size::Lg,
                    ..ButtonProps::default()
                },
                vec![("data-blocks-cta-split-actions-primary", "")],
                vec![text(primary_label)],
            ),
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Outline,
                    size: Size::Lg,
                    ..ButtonProps::default()
                },
                vec![("data-blocks-cta-split-actions-secondary", "")],
                vec![text(secondary_label)],
            ),
        ],
    )
}

/// 2 行構成の見出し（`plain` tone、モジュール doc「4 インスタンスで配色
/// 差分を表現する」節）。2 行目のみアクセント色にする（`h3` の内容モデルは
/// phrasing content のため、行の区切りには `div` ではなく `span` +
/// `display: block` を用いる）。
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
                vec![("class", "blocks-cta-split-actions-line")],
                vec![text(line1)],
            ),
            span(
                vec![
                    ("class", "blocks-cta-split-actions-line"),
                    ("data-blocks-cta-split-actions-line-accent", ""),
                ],
                vec![text(line2)],
            ),
        ],
    )
}

/// 訴求ポイント 1 件分（チェックアイコン + `text`）。
fn point(label: &str) -> Node {
    div(
        vec![("class", "blocks-cta-split-actions-point")],
        vec![
            check_icon(),
            styled_text::text(&TextProps::default(), vec![], vec![text(label)]),
        ],
    )
}

/// 行の共通ラッパー（見出し領域 + ボタン列、モジュール doc「レイアウトと
/// レスポンシブ」節）。`tone` は `plain`/`subtle`/`inverted` の 3 値のみを
/// 受け取る（`card` tone は [`instance_card`] が別経路で組み立てる）。
/// `copy_children` は見出し領域の子要素を直接並べる（`.blocks-cta-
/// split-actions-copy` の直接の子を複数にすることで `gap` が効くようにする
/// 契約。子を 1 個の無 class `div` で包んでしまうと `gap` を持つ flex
/// コンテナの子が実質 1 個になり効かなくなる不具合があったため、[`Vec<Node>`]
/// を直接受け取る設計にしている）。
fn row(tone: &'static str, copy_children: Vec<Node>, actions: Node) -> Node {
    div(
        vec![
            ("data-blocks-cta-split-actions-row", ""),
            ("data-blocks-cta-split-actions-tone", tone),
        ],
        vec![
            div(
                vec![("class", "blocks-cta-split-actions-copy")],
                copy_children,
            ),
            actions,
        ],
    )
}

/// 1 個目: 基準形（背景なし、モジュール doc「4 インスタンスで配色差分を
/// 表現する」節）。対応: R0881（主参照）。
fn instance_plain() -> Node {
    row(
        "plain",
        vec![two_line_heading(
            "新しいワークフローを、",
            "今日から始めましょう",
        )],
        actions("今すぐ始める", "詳しく見る"),
    )
}

/// 2 個目: 淡色ブランド面 + 訴求ポイント 3 点。対応: R0882（淡色面）、
/// R0067 / R0071（訴求ポイント。2 点版・2 列配置は「原案差分メモ」節へ
/// 記す）。
fn instance_subtle() -> Node {
    let copy_children = vec![
        heading(
            HeadingLevel::H3,
            &HeadingProps {
                size: HeadingSize::Xl2,
                ..HeadingProps::default()
            },
            vec![],
            vec![text("チームの導入を、まるごとサポートします")],
        ),
        div(
            vec![("class", "blocks-cta-split-actions-points")],
            vec![
                point("初期設定は担当者が同席してご案内します"),
                point("既存データの移行を無償でお手伝いします"),
                point("導入後 30 日間はいつでも解約できます"),
            ],
        ),
    ];
    row(
        "subtle",
        copy_children,
        actions("プランを選ぶ", "資料をもらう"),
    )
}

/// 3 個目: 反転配色。対応: R0447。補足文は [`TextVariant::Muted`]（自身に
/// `color: var(--fandhe-color-fg-muted)` を持つ）ではなく
/// [`TextVariant::Plain`]（`color` 宣言なし）を使う。`inverted` tone は
/// 祖先の `[data-blocks-cta-split-actions-tone="inverted"]` で
/// `color: var(--fandhe-color-bg)` を与えるため、子側は継承させる必要が
/// あり、`Muted` を使うと `fg-muted` で上書きされてコントラストが崩れる
/// （codex/Bugbot 指摘、イシュー #2758 PR レビュー）。
fn instance_inverted() -> Node {
    let copy_children = vec![
        heading(
            HeadingLevel::H3,
            &HeadingProps {
                size: HeadingSize::Xl2,
                ..HeadingProps::default()
            },
            vec![],
            vec![text("次の四半期の計画を、いま固めませんか")],
        ),
        styled_text::text(
            &TextProps {
                variant: TextVariant::Plain,
                ..TextProps::default()
            },
            vec![],
            vec![text(
                "担当チームが要件をヒアリングし、導入案をご提示します。",
            )],
        ),
    ];
    row("inverted", copy_children, actions("相談する", "事例を見る"))
}

/// 4 個目: アクセント色の浮いたカード。対応: R0451。カード面の余白は
/// [`card::body`] へ委ね、行のレイアウトは [`row`] と同じ属性構成
/// （`tone` は付けず、tone 上書きはカード側の
/// `data-blocks-cta-split-actions-tone="card"` から祖先セレクタで届かせる、
/// モジュール doc「tone 上書きの詳細度 (0,4,0)」節）。
fn instance_card() -> Node {
    let copy = div(
        vec![("class", "blocks-cta-split-actions-copy")],
        vec![heading(
            HeadingLevel::H3,
            &HeadingProps {
                size: HeadingSize::Xl2,
                ..HeadingProps::default()
            },
            vec![],
            vec![text("年間プランなら 2 か月分お得です")],
        )],
    );
    let row_inner = div(
        vec![("data-blocks-cta-split-actions-row", "")],
        vec![copy, actions("年間プランを見る", "月額プランのまま続ける")],
    );
    card::root(
        CardProps {
            variant: CardVariant::Elevated,
            ..CardProps::default()
        },
        vec![("data-blocks-cta-split-actions-tone", "card")],
        vec![card::body(vec![], vec![row_inner])],
    )
}

/// `cta-split-actions` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数（グリッド用ラッパーの class は [`Block::demo_class`] と別名にする
/// 理由をモジュール doc「`drop_class_attr` と CSS フックの選び方」節参照）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-cta-split-actions-stack")],
        vec![
            instance_plain(),
            instance_subtle(),
            instance_inverted(),
            instance_card(),
        ],
    )
}
```

## 原案差分メモ

参照（対応表 ID R0881 が主参照、R0067 / R0071 / R0447 / R0451 / R0882 を
配色・訴求ポイントの差分として集約。出典の固有名・ファイル名は記載しま
せん）からの意図的な差分は次のとおりです。

- R0881 の「見出しとボタン列を両端に置く」基準形を `plain` tone として
  実装しました。見出しは 2 行構成にし、2 行目のみアクセント色にしていま
  す。
- R0882 の淡色（ブランド寄り）の面は `subtle` tone として、
  `--fandhe-color-accent-subtle` の背景色で表現しました。
- R0067 / R0071 が持つ訴求ポイント（短い訴求点の一覧）は `subtle` tone の
  見出し下にチェックアイコン付きの一覧として 3 点統合しました。R0067 の
  2 点版・R0071 の 2 列配置は、Demo の肥大化を避けるため個別インスタンス
  化せず、この 1 インスタンスへ統合しています。
- R0447 の反転配色は `inverted` tone として、前景色と背景色を入れ替えて
  表現しました。
- R0451 のアクセント色の浮いたカードは `card` tone として、`card`（
  `CardVariant::Elevated`）のカード面をアクセント色へ上書きして表現しま
  した。
- レスポンシブの境界は `64rem`（`lg`）に固定しています。`64rem` 未満では
  見出し領域の下にボタン列を縦に積み、`64rem` 以上で横並び・両端揃えに
  なります。
- 見出しレベルはすべて `h3` にしています（ページ側が `## Demo` として
  `h2` を出すため）。
- `id` や `aria-labelledby` は出力しません（宙に浮いた ARIA 参照・id 重複
  を構造的に避けるため）。
- 文言（見出し・訴求ポイント・ボタンラベル）・アイコンの形・配色は
  すべて独自に書き直しました。実在ブランドの文言・配色・装飾・アイコン
  は持ち込んでいません。
