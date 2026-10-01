# incentives-inline-strip

`fandhe-frontend-pre-styled-ui` の `icon` / `text` / `visually-hidden`
部品を合成した、特典紹介を 1 行の帯で見せる実例です。Blocks セクションは
新規部品を追加するものではなく、既存の Themes/Primitives 部品を組み合わせた
実例集であることに注意してください（主参照は対応表 ID R1022、集約元は
R0560。出典の固有名・ファイル名は記載しません）。

アイコンと短いタイトルの組を横一行に並べ、説明文は付けません。集約元の
差分を 2 通りの例として並べています。

- 例 A: 広い幅では 1 行中央寄せ、狭い幅では自然に折り返します
- 例 B: 1 行のまま並べ、はみ出した分は横スクロールで見せます。横スクロール
  枠は `tabindex="0"` + `role="region"` + `aria-label` を持ち、マウスに
  依存せずキーボードの矢印キーでもスクロールできます

本 Demo は静的な表示例であり、`<form>` 要素も JavaScript も一切持ちません。
帯に見える見出しがないため、各例の先頭にはスクリーンリーダー向けの導入文
（`visually-hidden`）を置いています。文言はすべて独自に書いた架空のもの
であり、実企業名・実クレデンシャル・PII を含みません。

## Rust コード

```rust
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextWeight};
use fandhe_frontend_pre_styled_ui::visually_hidden;
use fandhe_frontend_pre_styled_ui::Size;

/// 特典 1 件分の架空データ（タイトルと自作アイコンのパス）。
struct Incentive {
    title: &'static str,
    icon_path_d: &'static str,
}

/// 特典 5 件（架空、実在の企業・製品とは無関係）。自作の単純な幾何パスのみ
/// を使い、lucide 等の著作物は複製しない。
const INCENTIVES: [Incentive; 5] = [
    Incentive {
        title: "送料無料",
        icon_path_d: "M3 7l9-4 9 4-9 4-9-4zM3 7v10l9 4 9-4V7M12 11v10",
    },
    Incentive {
        title: "30 日間返品",
        icon_path_d: "M4 4v6h6M4 10a8 8 0 1012-6",
    },
    Incentive {
        title: "安全なお支払い",
        icon_path_d: "M12 2l8 4v6c0 5-3.5 9-8 10-4.5-1-8-5-8-10V6z",
    },
    Incentive {
        title: "24 時間サポート",
        icon_path_d: "M12 21a9 9 0 100-18 9 9 0 000 18zM12 7v5l4 2",
    },
    Incentive {
        title: "ギフト包装",
        icon_path_d: "M3 9h18v4H3zM5 9v10h14V9M12 9v10M9 9c0-2 1-4 3-4s3 2 3 4M15 9c0-2-1-4-3-4",
    },
];

/// 装飾用の自作幾何アイコン（`fill="none"` + `stroke="currentColor"` の
/// 線画、`feature_three_column_icons::geo_icon` と同型の判断）。
fn geo_icon(path_d: &'static str) -> Node {
    icon(
        &IconProps {
            size: Size::Md,
            ..IconProps::default()
        },
        vec![("data-blocks-incentives-inline-strip-icon", "")],
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

/// 特典 1 件（アイコン → タイトル）を `li` で組み立てる。
fn item(i: &Incentive) -> Node {
    el(
        "li",
        vec![("class", "blocks-incentives-inline-strip-item")],
        vec![
            geo_icon(i.icon_path_d),
            styled_text::text(
                &TextProps {
                    size: TextSize::Sm,
                    weight: TextWeight::Medium,
                    ..TextProps::default()
                },
                vec![("data-blocks-incentives-inline-strip-title", "")],
                vec![text(i.title)],
            ),
        ],
    )
}

/// 注記行（インスタンスの原案差分の要約）。
fn note(body: &'static str) -> Node {
    styled_text::text(
        &TextProps {
            size: TextSize::Xs,
            ..TextProps::default()
        },
        vec![("data-blocks-incentives-inline-strip-note", "")],
        vec![text(body)],
    )
}

/// 見出しのない帯の先頭に置く、スクリーンリーダー向け導入文 + 特典一覧
/// （`ul`）。`variant` が `Some("scroll")` のとき横スクロール 1 行構成
/// （インスタンス B）、`None` のとき中央寄せ・折り返し構成（インスタンス A）
/// になる。
fn strip(hidden_label: &'static str, variant: Option<&'static str>) -> Vec<Node> {
    let mut ul_attrs = vec![("class", "blocks-incentives-inline-strip-list")];
    if let Some(v) = variant {
        ul_attrs.push(("data-variant", v));
    }
    let list = el(
        "ul",
        ul_attrs,
        INCENTIVES.iter().map(item).collect::<Vec<Node>>(),
    );
    vec![
        visually_hidden::root(vec![], vec![text(hidden_label)]),
        list,
    ]
}

/// `incentives-inline-strip` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数（他 block と同じ状態を持たない設計）。
#[must_use]
pub fn demo() -> Node {
    let mut instance_a_children = vec![note(
        "中央寄せ + 折り返し。広い幅では 1 行中央寄せ、狭い幅では自然に折り返す。",
    )];
    instance_a_children.extend(strip("ご購入特典", None));
    let instance_a = div(
        vec![("class", "blocks-incentives-inline-strip-instance")],
        instance_a_children,
    );

    let mut instance_b_children = vec![note(
        "横スクロール 1 行。広い幅では 1 行に収まり、狭い幅でははみ出した分を横スクロールで見せる。",
    )];
    let scroll_list = el(
        "div",
        vec![
            ("class", "blocks-incentives-inline-strip-scroller"),
            ("tabindex", "0"),
            ("role", "region"),
            ("aria-label", "特典一覧（横にスクロールできます）"),
        ],
        strip("ご購入特典", Some("scroll")),
    );
    instance_b_children.push(scroll_list);
    let instance_b = div(
        vec![("class", "blocks-incentives-inline-strip-instance")],
        instance_b_children,
    );

    div(
        vec![("class", "blocks-incentives-inline-strip-layout")],
        vec![instance_a, instance_b],
    )
}
```

## 原案差分メモ

- 例 A は集約元 R0560 に対応し、広い幅では中央寄せ、収まらなければ
  `flex-wrap: wrap` で自然に折り返す構成を表します。
- 例 B は主参照 R1022 に対応し、`flex-wrap: nowrap` + `overflow-x: auto`
  で 1 行のまま並べ、はみ出した分を横スクロールで見せる構成を表します。
  横スクロール枠には `tabindex="0"` + `role="region"` + `aria-label` を
  付け、キーボード操作でもスクロールできるようにしています。
- 参照元が持つ装飾・配色・説明文は持ち込まず、アイコンは自作の単純な
  幾何パスのみを使い、タイトルも独自に書いた架空の文言（送料無料・
  30 日間返品・安全なお支払い・24 時間サポート・ギフト包装）に差し替えて
  います。
- 帯に見える `h2`/`h3` 等の見出しはないため、各例の先頭へ
  `visually-hidden` でスクリーンリーダー向けの導入文（「ご購入特典」）を
  置いています。
- 狭い幅での折り返し・横スクロールの切り替えは無 JS のため CSS のみで
  表現しています。実際のブラウザでの表示確認は本 Demo では行っていません。

関連情報: [Icon](../themes/icon.md) / [Text](../themes/text.md) /
[Visually Hidden](../themes/visually-hidden.md)
