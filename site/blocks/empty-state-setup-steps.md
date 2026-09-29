# empty-state-setup-steps

`fandhe-frontend-pre-styled-ui` の `empty-state` / `steps` / `button` /
`icon` 部品を合成した、導入手順付きの大きめの空状態の合成例です。Blocks
セクションは新規部品を追加するものではなく、既存の Themes/Primitives 部品
を組み合わせた実例集であることに注意してください（対応表 ID R0376 のみを
参照します。集約元は主参照と同一 1 件のため、文言・構成に差分は
ありません。出典の固有名・ファイル名は記載しません）。

空状態（アイコン・見出し・説明・「最初のプロジェクトを作成」ボタン）の
下に、導入手順を 3 段（番号・見出し・説明）並べます。作成ボタンは空状態
の直下に 1 個だけで、手順の各段は表示専用で操作要素を持ちません。画面幅が
`40rem` 未満では手順を縦並びに折り返します。

本 Demo は静的な表示例であり、最初の手順のみを「現在地」として色分けし、
残り 2 段は「未着手」の色のまま固定表示します。`<form>` 要素は出力せず、
ボタンは `type="button"` のままで、送信先・入力値検証・状態管理は一切
持ちません（`docs/policy/intentional-non-adoption.md` §3.25 の責務境界:
UI コンポーネント層はアプリケーションロジックを内包しません。実際の
手順進行・送信処理を実装する場合は、利用者自身の Rust/JS コードで
実装してください）。

## Rust コード

```rust
use fandhe_frontend_core::{div, el, strong, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::empty_state::{
    self, EmptyStateIndicatorVariant, EmptyStateProps,
};
use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::steps::Steps;
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::recipe::ColorPalette;
use fandhe_frontend_pre_styled_ui::steps;
use fandhe_frontend_pre_styled_ui::{Orientation, Size};

/// 自作の幾何アイコン（角丸の四角に「+」を重ねた単純な折れ線、
/// `page_heading_meta::geo_icon` と同型で [`icon::icon`] へ独自の `path`/
/// `rect` を渡すのみ）。`aria-hidden="true"` の装飾用 SVG で、実在
/// ブランドのロゴ・商標は模さない。
fn folder_icon() -> Node {
    icon(
        &IconProps::default(),
        vec![],
        vec![
            el(
                "rect",
                vec![
                    ("x", "3"),
                    ("y", "5"),
                    ("width", "18"),
                    ("height", "14"),
                    ("rx", "2"),
                    ("fill", "none"),
                    ("stroke", "currentColor"),
                    ("stroke-width", "1.5"),
                ],
                vec![],
            ),
            el(
                "path",
                vec![
                    ("d", "M12 10v6M9 13h6"),
                    ("fill", "none"),
                    ("stroke", "currentColor"),
                    ("stroke-width", "1.5"),
                    ("stroke-linecap", "round"),
                ],
                vec![],
            ),
        ],
    )
}

/// 導入手順 1 段分（番号インジケータ + 見出し + 説明）を組み立てる内部
/// ヘルパ。`steps::trigger`/`content` を使わず `item` 直下へ静的な構造を
/// 置く（モジュール冒頭「`steps::trigger`/`content`/`separator` を置かない
/// 理由」節参照）。
fn step<'a>(s: &Steps, index: usize, heading: &'a str, description: &'a str) -> Node {
    steps::item(
        s,
        index,
        vec![],
        vec![
            steps::indicator(s, index, vec![], vec![text((index + 1).to_string())]),
            div(
                vec![("data-blocks-empty-state-setup-steps-step-body", "")],
                vec![
                    div(
                        vec![("data-blocks-empty-state-setup-steps-step-title", "")],
                        vec![strong(vec![], vec![text(heading)])],
                    ),
                    div(vec![], vec![text(description)]),
                ],
            ),
        ],
    )
}

/// `empty-state-setup-steps` の Demo 本体（大きめの空状態 + 導入手順 3 段。
/// 最初の手順を current・残りを incomplete で固定する）。呼び出しごとに
/// 同一の `Node` を返す純関数。
pub fn demo() -> Node {
    let s = Steps::new(3, 0, Orientation::Horizontal);

    let message = empty_state::root(
        &EmptyStateProps {
            size: Size::Lg,
            ..EmptyStateProps::default()
        },
        vec![("data-blocks-empty-state-setup-steps-message", "")],
        vec![empty_state::content(
            vec![],
            vec![
                empty_state::indicator_with(
                    EmptyStateIndicatorVariant::Boxed,
                    vec![],
                    vec![folder_icon()],
                ),
                empty_state::title(vec![], vec![text("まだプロジェクトがありません")]),
                empty_state::description(
                    vec![],
                    vec![text(
                        "最初のプロジェクトを作成すると、ここに一覧が表示されます。",
                    )],
                ),
                empty_state::actions(
                    vec![],
                    vec![button::button(
                        &ButtonProps::default(),
                        vec![],
                        vec![text("最初のプロジェクトを作成")],
                    )],
                ),
            ],
        )],
    );

    let steps_root = steps::root(
        Size::Md,
        ColorPalette::Accent,
        &s,
        vec![("data-blocks-empty-state-setup-steps-steps", "")],
        vec![steps::list(
            &s,
            vec![],
            vec![
                step(
                    &s,
                    0,
                    "プロジェクトを作成",
                    "名前と説明を入力して、新しいプロジェクトを作ります。",
                ),
                step(
                    &s,
                    1,
                    "メンバーを招待",
                    "メールアドレスでチームメンバーを招待します。",
                ),
                step(
                    &s,
                    2,
                    "最初のタスクを登録",
                    "着手するタスクを追加して作業を始めます。",
                ),
            ],
        )],
    );

    div(
        vec![("class", "blocks-empty-state-setup-steps-root")],
        vec![message, steps_root],
    )
}
```

## 原案差分メモ

参照（対応表 ID R0376。出典の固有名・ファイル名は記載しません）から
取り込んだのは構造（領域の配置と部品構成）のみです。集約元は主参照と
同一 1 件のため、原案からの差分はありません。文言・アイコンはすべて
独自に書き下ろしています。

- 空状態のアイコンは参照元のアイコンを転記せず、独自の幾何学的な
  フォルダ + 折れ線の SVG を描いています。
- 手順は `steps::trigger`（実 `<button>`）ではなく `item` 直下へ番号・
  見出し・説明を静的に組み立てています。無 JS の docs サイトでは押しても
  何も起きない操作要素を増やさないための判断です。
- 手順の説明は `steps::content` を使わず素の `div` で常時表示しています
  （`content` は非 current の段が `data-state="closed"` で隠れてしまうため）。
- 文言（見出し・説明文・ボタンラベル）はすべて独自の架空ダミーです。
  実在の人物・企業・実クレデンシャルは一切含みません。

関連情報: [Empty State](../themes/empty-state.md) / [Steps](../themes/steps.md) /
[Button](../themes/button.md) / [Icon](../themes/icon.md)
