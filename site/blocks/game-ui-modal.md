# game-ui-modal

`dialog` / `badge` / `button` を合成した、Motion+ `examples/game-ui`
（ゲーム風 UI カテゴリの実例）参照のモーダル入場アニメーション例です。
新規部品ではなく既存部品の合成であり、`<form>` は使わず、無 JS のため
モーダルが既に開いた状態のみを固定表示します。

## Rust コード

```rust
use fandhe_frontend_core::{li, text, ul, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps, BadgeVariant};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::dialog::{self, ContentIds, DialogRole, OpenState};
use fandhe_frontend_pre_styled_ui::recipe::stagger_index_style;
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// 架空の報酬データ（実企業名・実クレデンシャルは使わない）。
const REWARDS: [(&str, ColorPalette); 3] = [
    ("+320 XP", ColorPalette::Success),
    ("レア装備 x1", ColorPalette::Info),
    ("称号「開拓者」", ColorPalette::Accent),
];

/// `game-ui-modal` の Demo 本体（既に開いた静的な初期状態のみ描く）。
pub fn demo() -> Node {
    let title_id = "blocks-game-ui-modal-title";
    let description_id = "blocks-game-ui-modal-description";

    let rewards: Vec<Node> = REWARDS
        .iter()
        .enumerate()
        .map(|(index, (label, palette))| {
            let style = stagger_index_style(index);
            li(
                vec![("data-blocks-game-ui-modal-reward", ""), ("style", &style)],
                vec![badge::badge(
                    &BadgeProps {
                        variant: BadgeVariant::Solid,
                        size: Size::Md,
                        palette: *palette,
                    },
                    vec![],
                    vec![text(*label)],
                )],
            )
        })
        .collect();

    dialog::root(
        Size::Md,
        OpenState::Open,
        vec![("data-blocks-game-ui-modal-root", "")],
        vec![
            dialog::backdrop(OpenState::Open, vec![], vec![]),
            dialog::positioner(
                OpenState::Open,
                vec![],
                vec![dialog::content(
                    OpenState::Open,
                    DialogRole::Dialog,
                    // 静的デモは閉じる機構を持たず外側に説明・コード・
                    // ナビゲーションがあるため、表示実態と一致させ
                    // aria-modal は false にする（支援技術が外側を
                    // 無視しないようにする、イシュー #2552 レビュー指摘）。
                    false,
                    ContentIds {
                        id: Some("blocks-game-ui-modal-content"),
                        labelledby: Some(title_id),
                        describedby: Some(description_id),
                    },
                    vec![("data-blocks-game-ui-modal-content", "")],
                    vec![
                        dialog::title(Some(title_id), vec![], vec![text("Quest Complete")]),
                        dialog::description(
                            Some(description_id),
                            vec![],
                            vec![text("討伐クエスト「北の遺跡」を制覇しました。")],
                        ),
                        dialog::body(vec![], vec![ul(vec![], rewards)]),
                        dialog::footer(
                            vec![],
                            vec![
                                button::button(
                                    &ButtonProps {
                                        variant: ButtonVariant::Ghost,
                                        ..ButtonProps::default()
                                    },
                                    vec![],
                                    vec![text("Later")],
                                ),
                                button::button(
                                    &ButtonProps::default(),
                                    vec![],
                                    vec![text("Claim rewards")],
                                ),
                            ],
                        ),
                    ],
                )],
            ),
        ],
    )
}
```

Motion+ は購入者限定素材のため取得手段・内部識別子は記載しません
（設計判断の詳細は `docs/design/docs-site-blocks-section.md` §16）。

関連情報: [Dialog](../themes/dialog.md) / [Button](../themes/button.md) /
[Badge](../themes/badge.md)
