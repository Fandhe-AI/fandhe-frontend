# action-panel-footer-bar

`fandhe-frontend-pre-styled-ui` の `button` / `button-group` / `text` /
`separator` / `icon` の 5 部品を合成した、セクション末尾の操作バーの実例
です。Blocks セクションは新規部品を追加するものではなく、既存の
Themes/Primitives 部品を組み合わせた実例集であることに注意してください
（主参照 ID R0647・R0648・R0649 の 3 例を併記します。出典の固有名・
ファイル名は記載しません）。

上罫線付きの細い帯をセクション末尾に置き、左に補足テキスト、右にボタン群
（表示・編集・保存等）を並べる構成です。3 インスタンスを併記します。

- A（代表構成）: 補足テキストが上・操作列が下（狭幅）、Outline「表示」
  「編集」の button-group + 単体 Solid「保存」+ アイコンのみの「その他の
  操作」ボタン
- B（積み順・ボタン順違い）: 狭幅では操作列が先に積まれ、主ボタン
  （「保存」）を先頭に置く並び順違い
- C（主・副ボタンの購入導線）: Solid「購入する」+ Outline「カートに追加」
  の 2 ボタンのみ。狭幅では各ボタンを全幅にして縦積み

アイコンのみの「その他の操作」ボタンには支援技術向けのアクセシブルネーム
（`aria-label`）を付与しています。参照元の配色・文言・アイコンは持ち込ま
ず、既存の Blocks のトーンに揃えています。

本 Demo は静的な表示例であり、`<form>` 要素は一切持たず、データの取得・
送信・状態管理を行いません。ボタンは `type="button"` のまま送信先を持ち
ません。文言はすべて架空のダミーであり、実企業名・実クレデンシャル・PII
を含みません。

## Rust コード

```rust
use fandhe_frontend_core::{div, el, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::button_group::{self, Orientation};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::separator::{self, SeparatorProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};

/// 自作の単純な幾何アイコン（`page_heading_cover.rs::geo_icon` と同型。
/// モジュール doc「アイコンは自作の単純幾何図形」参照）。開いた線分のみの
/// パスのため `fill="none"` + `stroke="currentColor"` でアウトライン描画
/// にする。
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

/// 縦三点アイコン（「その他の操作」の装飾。実在アイコンセットは使わない）。
fn more_icon() -> Node {
    geo_icon("M12 6v.01 M12 12v.01 M12 18v.01")
}

/// 1 インスタンス分の帯を組み立てる（上罫線 + 補足テキスト + 操作列）。
fn instance(stack: &'static str, note: &'static str, actions: Vec<Node>) -> Node {
    let rule = separator::separator(
        &SeparatorProps::default(),
        vec![("data-blocks-action-panel-footer-bar-rule", "")],
    );
    let note_node = styled_text::text(
        &TextProps {
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![("data-blocks-action-panel-footer-bar-note", "")],
        vec![fandhe_frontend_core::text(note)],
    );
    let actions_node = div(
        vec![("data-blocks-action-panel-footer-bar-actions", "")],
        actions,
    );
    let bar = div(
        vec![
            ("data-blocks-action-panel-footer-bar-bar", ""),
            ("data-blocks-action-panel-footer-bar-stack", stack),
        ],
        vec![note_node, actions_node],
    );
    div(
        vec![("data-blocks-action-panel-footer-bar-instance", "")],
        vec![rule, bar],
    )
}

/// `action-panel-footer-bar` の Demo 本体（呼び出しごとに同一の `Node` を
/// 返す純関数）。3 インスタンス（R0647/R0648/R0649）を縦に併記する。
pub fn demo() -> Node {
    // A: 代表構成（R0647）。補足が上・操作が下、Outline 2 個の button-group
    // + 単体 Solid「保存」+ アイコンのみ「その他の操作」。
    let example_a = instance(
        "note-first",
        "最終保存: 2 分前（下書き）",
        vec![
            button_group::root(
                Orientation::Horizontal,
                "レコードの操作",
                vec![],
                vec![
                    button::button(
                        &ButtonProps {
                            variant: ButtonVariant::Outline,
                            ..ButtonProps::default()
                        },
                        vec![],
                        vec![fandhe_frontend_core::text("表示")],
                    ),
                    button::button(
                        &ButtonProps {
                            variant: ButtonVariant::Outline,
                            ..ButtonProps::default()
                        },
                        vec![],
                        vec![fandhe_frontend_core::text("編集")],
                    ),
                ],
            ),
            button::button(
                &ButtonProps::default(),
                vec![],
                vec![fandhe_frontend_core::text("保存")],
            ),
            button::icon_button(
                &ButtonProps::default(),
                "その他の操作",
                vec![],
                vec![more_icon()],
            ),
        ],
    );

    // B: 積み順・ボタン順違い（R0648）。狭幅では操作列が先、主ボタン
    // （保存）を先頭に置く。
    let example_b = instance(
        "actions-first",
        "変更は自動で保存されません",
        vec![
            button::button(
                &ButtonProps::default(),
                vec![],
                vec![fandhe_frontend_core::text("保存")],
            ),
            button_group::root(
                Orientation::Horizontal,
                "レコードの操作",
                vec![],
                vec![
                    button::button(
                        &ButtonProps {
                            variant: ButtonVariant::Outline,
                            ..ButtonProps::default()
                        },
                        vec![],
                        vec![fandhe_frontend_core::text("編集")],
                    ),
                    button::button(
                        &ButtonProps {
                            variant: ButtonVariant::Outline,
                            ..ButtonProps::default()
                        },
                        vec![],
                        vec![fandhe_frontend_core::text("表示")],
                    ),
                ],
            ),
        ],
    );

    // C: 主・副ボタンの購入導線（R0649）。狭幅では各ボタンを全幅にして
    // 縦積み。
    let example_c = instance(
        "full-width",
        "合計 ¥4,800（税込）・送料無料",
        vec![
            button::button(
                &ButtonProps::default(),
                vec![],
                vec![fandhe_frontend_core::text("購入する")],
            ),
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Outline,
                    ..ButtonProps::default()
                },
                vec![],
                vec![fandhe_frontend_core::text("カートに追加")],
            ),
        ],
    );

    div(
        vec![("class", "blocks-action-panel-footer-bar-layout")],
        vec![example_a, example_b, example_c],
    )
}
```

## 原案差分メモ

- 主参照は R0647（代表構成）で、集約元 R0648（狭幅での積み順・ボタン順
  違い）・R0649（主・副ボタンの購入導線）を別インスタンスとして併記して
  います。
- 参照元の配色・文言・アイコンは持ち込まず、既存 Blocks のトーンに
  揃えています。
- ボタンは押下先を持たない静的表示です。
- 狭幅（`40rem` 未満）の積み順・全幅化は CSS のみで表現しており、
  ブラウザ実機での目視確認はサンドボックス環境の制約により未実施です
  （実装ステップ・検証方法は Issue 本文・実装計画を参照）。

関連情報: [Button](../themes/button.md) /
[Button Group](../themes/button-group.md) / [Text](../themes/text.md) /
[Separator](../themes/separator.md) / [Icon](../themes/icon.md)
