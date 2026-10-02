# api-reference-param-accordion

`fandhe-frontend-pre-styled-ui` の `heading` / `text` / `badge` / `code` /
`accordion` 部品を合成した、API パラメータ一覧の開閉式表示です。Blocks
セクションは新規部品を追加するものではなく、既存の Themes/Primitives
部品を組み合わせた実例集であることに注意してください（主参照は対応表 ID
R0197。出典の固有名・ファイル名は記載しません）。

枠の上端に列見出し行（「名前」「型」）を置き、その下にパラメータ 1 件
ごとの開閉式の行を並べます。開いた行には説明と既定値を表示します。静的
表示では全行を開いた状態にしています。枠・見出し行・各行の角丸は
外枠（アコーディオンの root）1 つでまとめています。

本 Demo は静的な表示例であり、全行を `disabled` により開閉操作自体を
無効化しています（理由は「原案差分メモ」節を参照）。`<form>` 要素は一切
持たず、データの取得・送信・状態管理を行いません。パラメータの名前・
型・説明・既定値はすべて独自に書いた架空のものであり、実在する API・
実企業名・実クレデンシャル・PII を含みません。

## Rust コード

```rust
use fandhe_frontend_core::{div, el, span, text, Node};
use fandhe_frontend_pre_styled_ui::accordion::{
    self, item, item_content, item_indicator, item_trigger, AccordionProps, OpenState,
};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps, BadgeVariant};
use fandhe_frontend_pre_styled_ui::code::{self, CodeProps};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::Size;

/// 架空の API パラメータ 1 件（名前・型・必須か・説明・既定値）。
struct Param {
    name: &'static str,
    ty: &'static str,
    required: bool,
    description: &'static str,
    default: Option<&'static str>,
}

/// 架空のクエリパラメータ一覧（実在サービスの API 名は含まない）。
const PARAMS: [Param; 4] = [
    Param {
        name: "limit",
        ty: "integer",
        required: false,
        description: "1 ページあたりの最大件数。",
        default: Some("20"),
    },
    Param {
        name: "cursor",
        ty: "string",
        required: false,
        description: "前回応答の next_cursor を渡すとその続きから取得する。",
        default: None,
    },
    Param {
        name: "order",
        ty: "\"asc\" | \"desc\"",
        required: false,
        description: "作成日時順のソート方向。",
        default: Some("\"desc\""),
    },
    Param {
        name: "project_id",
        ty: "string",
        required: true,
        description: "対象プロジェクトの識別子。",
        default: None,
    },
];

/// 枠の上端に置く列見出し行（「名前」「型」）。トリガーのラベル
/// grid（[`param_trigger_label`]）と同じ列幅比を共有する。
fn columns_header() -> Node {
    div(
        vec![("class", "blocks-api-reference-param-accordion-columns")],
        vec![
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    size: TextSize::Sm,
                    ..TextProps::default()
                },
                vec![("class", "blocks-api-reference-param-accordion-col-name")],
                vec![text("名前")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    size: TextSize::Sm,
                    ..TextProps::default()
                },
                vec![("class", "blocks-api-reference-param-accordion-col-type")],
                vec![text("型")],
            ),
        ],
    )
}

/// トリガーのラベル（名前 + 必須/任意バッジ、型）。
fn param_trigger_label(param: &Param) -> Node {
    span(
        vec![(
            "class",
            "blocks-api-reference-param-accordion-trigger-label",
        )],
        vec![
            span(
                vec![("class", "blocks-api-reference-param-accordion-name")],
                vec![
                    code::code(&CodeProps::default(), vec![], vec![text(param.name)]),
                    badge::badge(
                        &BadgeProps {
                            variant: if param.required {
                                BadgeVariant::Solid
                            } else {
                                BadgeVariant::Subtle
                            },
                            ..BadgeProps::default()
                        },
                        vec![],
                        vec![text(if param.required { "必須" } else { "任意" })],
                    ),
                ],
            ),
            span(
                vec![("class", "blocks-api-reference-param-accordion-type")],
                vec![code::code(
                    &CodeProps::default(),
                    vec![],
                    vec![text(param.ty)],
                )],
            ),
        ],
    )
}

/// 説明 + 既定値（`description`/`code` の 2 行）。
fn param_detail(param: &Param) -> Node {
    div(
        vec![],
        vec![
            styled_text::text(&TextProps::default(), vec![], vec![text(param.description)]),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![
                    text("既定値: "),
                    match param.default {
                        Some(default) => {
                            code::code(&CodeProps::default(), vec![], vec![text(default)])
                        }
                        None => text("既定値なし"),
                    },
                ],
            ),
        ],
    )
}

/// パラメータ 1 件分の accordion item。全行 [`OpenState::Open`] +
/// `disabled: true` 固定（モジュール doc「静的表示」節）。無 JS で開閉が
/// 機能しないため、全行 open にして説明・既定値を読める状態にする
/// （`faq_accordion_centered` と同じ判断、イシュー #3100 PR #3542 指摘）。
fn param_item(index: usize, param: &Param) -> Node {
    let state = OpenState::Open;
    let props = AccordionProps {
        disabled: true,
        ..AccordionProps::default()
    };
    let trigger_id = format!("blocks-api-reference-param-accordion-{index}-trigger");
    let content_id = format!("blocks-api-reference-param-accordion-{index}-content");

    item(
        state,
        false,
        &props,
        vec![],
        vec![
            el(
                "h4",
                vec![(
                    "class",
                    "blocks-api-reference-param-accordion-trigger-heading",
                )],
                vec![item_trigger(
                    state,
                    false,
                    &props,
                    param.name,
                    Some(trigger_id.as_str()),
                    Some(content_id.as_str()),
                    vec![],
                    vec![
                        param_trigger_label(param),
                        item_indicator(state, false, &props, vec![], vec![text("▾")]),
                    ],
                )],
            ),
            item_content(
                state,
                false,
                &props,
                Some(content_id.as_str()),
                Some(trigger_id.as_str()),
                vec![],
                vec![param_detail(param)],
            ),
        ],
    )
}

/// 列見出し行 + パラメータ行を内包する外枠（`accordion::root` 1 つに
/// まとめる、モジュール doc「外枠をまとめる方法」節）。
fn params_root() -> Node {
    let mut children: Vec<Node> = vec![columns_header()];
    children.extend(
        PARAMS
            .iter()
            .enumerate()
            .map(|(index, param)| param_item(index, param)),
    );

    accordion::root(
        Size::Md,
        &AccordionProps::default(),
        vec![("data-blocks-api-reference-param-accordion-root", "")],
        children,
    )
}

/// `api-reference-param-accordion` の Demo 本体。呼び出しごとに同一の
/// `Node` を返す純関数（モジュール doc「静的表示」節）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-api-reference-param-accordion-layout")],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    ..HeadingProps::default()
                },
                vec![],
                vec![text("リクエストパラメータ")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(
                    "各パラメータの説明と既定値を開いた状態で表示しています。",
                )],
            ),
            params_root(),
        ],
    )
}
```

## 原案差分メモ

- 参照元（対応表 ID R0197）の「開閉式の行」はそのまま採用しつつ、静的
  表示では全行を open にしました。原案は「先頭 1 行だけ open」でしたが、
  無 JS の静的 Demo で全行を `disabled` にすると閉じた行の説明・既定値に
  利用者が到達できなくなるため（PR #3542 指摘）、`faq-accordion-centered`
  等と同じ「全件 open」方針へ変更しています。
- 閉じた行も含め全行を `disabled` にしています。無 JS の docs サイトでは
  閉じた行のトリガーがクリック・Enter/Space に反応しないフォーカス可能な
  `<button>` として残ると、本文（説明・既定値）が事実上到達不能になるため
  です（`faq-accordion-centered` 等、先行する block が受けた指摘と同じ
  判断）。
- 列見出し行（「名前」「型」）はアコーディオンの `root` の最初の子として
  置き、トリガーのラベルと同じ `grid-template-columns` を共有すること
  で、見出しと各行の列を揃えています。
- 開閉インジケータはアイコンを使わず、既存部品のテキスト「▾」にしました。
- パラメータの名前・型・説明・既定値はすべて独自に書いた架空のものです。
- 配色・余白・角丸は既存のテーマトークンに従っています。

関連情報: [Heading](../themes/heading.md) / [Text](../themes/text.md) /
[Badge](../themes/badge.md) / [Code](../themes/code.md) /
[Accordion](../themes/accordion.md)
