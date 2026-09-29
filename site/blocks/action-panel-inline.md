# action-panel-inline

`fandhe-frontend-pre-styled-ui` の `card` / `heading` / `text` / `button` /
`switch` 部品を合成した、右側に操作を置くアクションパネルの実例です。
Blocks セクションは新規部品を追加するものではなく、既存の
Themes/Primitives 部品を組み合わせた実例集であることに注意してください
（主参照は対応表 ID R0733、集約元は R0734・R0735。出典の固有名・ファイル名
は記載しません）。

タイトルの下に説明文、その右側にボタンまたはトグルスイッチを横並びに
配置します。ボタンを右上へ固定した版も併記しており、こちらは
`fandhe-frontend-pre-styled-ui` の `card` が提供する `data-has-action`/
`action` スロット（見出しの右上にアクションを固定する既存 recipe）を
そのまま再利用しています。狭い幅（`40rem` 未満）では、左右並びの版の
操作が説明文の下へ回り込みます。

3 つのレイアウト差分を 1 block・3 インスタンス縦積みで並べています。
トグルスイッチは `checked`（オン）かつ `disabled` の初期状態で固定表示
される操作不能な例であり、本 Demo が無 JS であることとは無関係に
（native checkbox 自体が `disabled` のため）操作できません。

本 Demo は静的な表示例であり、`<form>` 要素は一切持たず、データの取得・
送信・状態管理を行いません。ボタンは `type="button"` のまま送信先を持たず、
トグルスイッチの隠し入力も送信先を持ちません。文言はすべて独自に書いた
架空のものであり、実企業名・実クレデンシャル・PII を含みません。

## Rust コード

```rust
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps, CardVariant};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::switch::{self, SwitchProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// 見出し（`<h3>`）。3 インスタンス共通で使う。
fn panel_heading(title: &'static str) -> Node {
    heading(
        HeadingLevel::H3,
        &HeadingProps {
            size: HeadingSize::Lg,
            ..HeadingProps::default()
        },
        vec![],
        vec![text(title)],
    )
}

/// 説明文（ミュート）。3 インスタンス共通で使う。
fn description(body: &'static str) -> Node {
    styled_text::text(
        &TextProps {
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![],
        vec![text(body)],
    )
}

/// `inline-button`（R0733 代表構成）: 見出し＋説明文の右側にボタン 1 個。
fn panel_inline_button() -> Node {
    card::root(
        CardProps::from(CardVariant::Outline),
        vec![("data-blocks-action-panel-inline-panel", "inline-button")],
        vec![
            card::header(vec![], vec![panel_heading("二要素認証")]),
            card::body(
                vec![("class", "blocks-action-panel-inline-row")],
                vec![
                    description("サインイン時にワンタイムコードの入力を求めます。"),
                    div(
                        vec![("data-blocks-action-panel-inline-action", "")],
                        vec![button::button(
                            &ButtonProps {
                                variant: ButtonVariant::Outline,
                                size: Size::Sm,
                                ..ButtonProps::default()
                            },
                            vec![],
                            vec![text("有効にする")],
                        )],
                    ),
                ],
            ),
        ],
    )
}

/// `top-right-button`（R0734 ボタン右上固定）: card の `data-has-action`/
/// `action` スロットをそのまま再利用する（本 block 側は独自 CSS を持たない、
/// モジュール doc「使用部品」節参照）。
fn panel_top_right_button() -> Node {
    card::root(
        CardProps::from(CardVariant::Outline),
        vec![("data-blocks-action-panel-inline-panel", "top-right-button")],
        vec![
            card::header(
                vec![("data-has-action", "")],
                vec![
                    panel_heading("請求先メールアドレス"),
                    card::action(
                        vec![],
                        vec![button::button(
                            &ButtonProps {
                                variant: ButtonVariant::Outline,
                                size: Size::Sm,
                                ..ButtonProps::default()
                            },
                            vec![],
                            vec![text("変更する")],
                        )],
                    ),
                ],
            ),
            card::body(
                vec![],
                vec![description("請求書と領収書の送付先を変更します。")],
            ),
        ],
    )
}

/// `inline-switch`（R0735 トグルスイッチ版）: `inline-button` と同じ行構造
/// で、右側をトグルスイッチへ差し替える。
fn panel_inline_switch() -> Node {
    let switch_props = SwitchProps {
        // モジュール doc「switch は `disabled: true` の `checked` 初期状態で
        // 固定する」節参照。native checkbox の操作を実際に抑止し、
        // 操作後の状態不一致（AT が伝える状態と `data-state` 固定表示の
        // 食い違い）を構造的に防ぐ。
        disabled: true,
        ..SwitchProps::default()
    };
    card::root(
        CardProps::from(CardVariant::Outline),
        vec![("data-blocks-action-panel-inline-panel", "inline-switch")],
        vec![
            card::header(vec![], vec![panel_heading("メール通知")]),
            card::body(
                vec![("class", "blocks-action-panel-inline-row")],
                vec![
                    description("週次レポートと重要なお知らせをメールで受け取ります。"),
                    div(
                        vec![("data-blocks-action-panel-inline-action", "")],
                        vec![switch::root(
                            Size::Md,
                            ColorPalette::Accent,
                            true,
                            &switch_props,
                            vec![],
                            vec![
                                switch::label(
                                    true,
                                    &switch_props,
                                    vec![],
                                    vec![text("通知を受け取る")],
                                ),
                                switch::hidden_input(
                                    "blocks-action-panel-inline-notify",
                                    "on",
                                    true,
                                    &switch_props,
                                    vec![],
                                ),
                                switch::control(
                                    true,
                                    &switch_props,
                                    vec![],
                                    vec![switch::thumb(true, &switch_props, vec![], vec![])],
                                ),
                            ],
                        )],
                    ),
                ],
            ),
        ],
    )
}

/// `action-panel-inline` の Demo 本体（3 レイアウトを縦積みで並記する。
/// 呼び出しごとに同一の `Node` を返す純関数）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-action-panel-inline-layout")],
        vec![
            panel_inline_button(),
            panel_top_right_button(),
            panel_inline_switch(),
        ],
    )
}
```

## 集約元との差分メモ

- 主参照（対応表 ID R0733）は説明文の右側にボタンを 1 個置いた代表構成
  （例 A: `inline-button`）です。
- 集約元（対応表 ID R0734）はボタンを見出しの右上へ固定した版（例 B:
  `top-right-button`）で、本 block では `card` の `data-has-action`/
  `action` スロットをそのまま再利用し、独自の配置 CSS を追加していません。
- 集約元（対応表 ID R0735）はトグルスイッチ版（例 C: `inline-switch`）で、
  例 A と同じ行構造のまま右側の操作をトグルスイッチに差し替えています。
  スイッチは `checked`（オン）かつ `disabled` の初期状態で固定された
  操作不能な表示例です（native checkbox 自体が `disabled` のため、無 JS
  であることとは無関係に操作を受け付けません）。
- いずれも `40rem` 未満で操作が説明文の下へ回り込み、`40rem` 以上で左右
  配置になります（`top-right-button` は card 自身の recipe が grid 配置を
  担うため対象外）。
- 文言・配色は既存のテーマトークンに従い、独自に書いた架空のものです。

関連情報: [Card](../themes/card.md) / [Heading](../themes/heading.md) /
[Text](../themes/text.md) / [Button](../themes/button.md) /
[Switch](../themes/switch.md)
