# empty-state-invite-team

まだメンバーがいない状態を示す空状態と、その場でメールアドレスを入力して
招待できる入力欄、おすすめメンバー候補一覧を組み合わせたブロックです。
`empty-state` / `field` / `input-group` / `input` / `button` / `avatar` /
`item` / `text` の 8 部品を合成します。Blocks は既存部品の合成例であり、
新しい UI 部品は追加しません。

主参照は対応表 ID R0464（代表構成）で、R1394（メール招待 + 候補の縦一覧）・
R1395（招待 + 候補のグリッド）を集約しています。氏名・役職はすべて架空の
データであり、実在の人物・企業・PII は含みません。メールアドレスは
`example.com` ドメインのプレースホルダーです。アバター画像はビルド時生成の
同梱プレースホルダー SVG です。

本 Demo は無 JS の静的表示のみであり、`<form>` を含みません。招待ボタン・
追加ボタンはいずれも `type="button"` の静的な表示で、実際の送信処理・
追加処理は行いません。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, span, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::empty_state::{self, EmptyStateProps};
use fandhe_frontend_pre_styled_ui::field::{
    self, FieldIds, FieldOrientation, FieldProps, FieldRootProps,
};
use fandhe_frontend_pre_styled_ui::input::{self, InputProps};
use fandhe_frontend_pre_styled_ui::input_group::{self, InputGroupAlign, InputGroupProps};
use fandhe_frontend_pre_styled_ui::item::{
    self, ItemMediaVariant, ItemRootProps, ItemSize, ItemVariant,
};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::Size;

/// 版ラベル（`hero_email_signup::variant_label` と同型）。[`fandhe_frontend_pre_styled_ui::text::text`]
/// を使い、使用部品一覧（[`crate::blocks`] の `parts`）へ Text を含める。
fn variant_label(label: &'static str) -> Node {
    styled_text::text(
        &TextProps {
            size: TextSize::Sm,
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![],
        vec![text(label)],
    )
}

/// 自作の幾何アイコン（線画。モジュール doc「アイコンは自作の単純図形」
/// 節参照）。装飾のため `aria-hidden="true"` を付与する。
fn geo_icon(view_box: &'static str, path_d: &'static str) -> Node {
    el(
        "svg",
        vec![
            ("viewBox", view_box),
            ("width", "40"),
            ("height", "40"),
            ("aria-hidden", "true"),
        ],
        vec![el(
            "path",
            vec![
                ("d", path_d),
                ("fill", "none"),
                ("stroke", "currentColor"),
                ("stroke-width", "1.5"),
                ("stroke-linecap", "round"),
                ("stroke-linejoin", "round"),
            ],
            vec![],
        )],
    )
}

/// 空状態の装飾アイコン（人物シルエット + プラス記号）。
fn invite_icon() -> Node {
    geo_icon(
        "0 0 24 24",
        "M9 11a3 3 0 1 0 0-6 3 3 0 0 0 0 6z M3 20c0-3.3 2.7-6 6-6s6 2.7 6 6 M18 8v6 M15 11h6",
    )
}

/// 「追加済み」の印に添える小さいチェックマーク。
fn check_icon() -> Node {
    el(
        "svg",
        vec![
            ("viewBox", "0 0 16 16"),
            ("width", "14"),
            ("height", "14"),
            ("aria-hidden", "true"),
        ],
        vec![el(
            "path",
            vec![
                ("d", "M3 8.5l3 3 7-7"),
                ("fill", "none"),
                ("stroke", "currentColor"),
                ("stroke-width", "1.5"),
                ("stroke-linecap", "round"),
                ("stroke-linejoin", "round"),
            ],
            vec![],
        )],
    )
}

/// メールアドレス入力 + 招待送信ボタンを一体化した入力グループ
/// （`hero_email_signup::signup_group` と同型）。`instance` はインスタンス
/// ごとに異なる `id` の suffix（モジュール doc「`id` は 2 インスタンス分
/// すべて別値にする」節）。
fn invite_field(instance: &'static str) -> Node {
    let field_id = format!("blocks-empty-state-invite-team-email-{instance}");
    let email_field = FieldProps {
        id: &field_id,
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: true,
        readonly: false,
        has_helper_text: false,
    };
    let group_props = InputGroupProps {
        disabled: false,
        invalid: false,
    };

    field::root(
        &FieldRootProps {
            orientation: FieldOrientation::Vertical,
        },
        &email_field,
        vec![("data-blocks-empty-state-invite-team-field", "")],
        vec![
            field::label(&email_field, vec![], vec![text("メールアドレス")]),
            input_group::root(
                &group_props,
                vec![("data-blocks-empty-state-invite-team-group", "")],
                vec![
                    input::input(
                        &InputProps::default(),
                        &email_field,
                        vec![
                            ("type", "email"),
                            ("autocomplete", "email"),
                            ("placeholder", "member@example.com"),
                        ],
                    ),
                    input_group::addon(
                        InputGroupAlign::InlineEnd,
                        &group_props,
                        vec![("data-blocks-empty-state-invite-team-addon", "")],
                        vec![button::button(
                            &ButtonProps::default(),
                            vec![("data-blocks-empty-state-invite-team-submit", "")],
                            vec![text("招待を送る")],
                        )],
                    ),
                ],
            ),
        ],
    )
}

/// おすすめメンバー候補 1 件（アバター + 氏名・役職 + 追加操作）。`added`
/// が `true` のときは追加ボタンの代わりに「追加済み」の印を出す。
fn candidate_item(name: &'static str, job_title: &'static str, added: bool) -> Node {
    let action = if added {
        span(
            vec![("class", "blocks-empty-state-invite-team-added")],
            vec![check_icon(), text("追加済み")],
        )
    } else {
        button::button(
            &ButtonProps {
                variant: ButtonVariant::Outline,
                size: Size::Sm,
                ..ButtonProps::default()
            },
            vec![("aria-label", &format!("{name} を追加"))],
            vec![text("追加")],
        )
    };

    item::root(
        ItemRootProps {
            variant: ItemVariant::Outline,
            size: ItemSize::Sm,
            ..ItemRootProps::default()
        },
        vec![("data-blocks-empty-state-invite-team-candidate", "")],
        vec![
            item::media(
                ItemMediaVariant::Default,
                vec![],
                vec![avatar::root(
                    &AvatarProps {
                        size: Size::Sm,
                        ..AvatarProps::default()
                    },
                    vec![],
                    vec![
                        avatar::image(ImageStatus::Loaded, dummy_assets::AVATAR_SRC, name, vec![]),
                        avatar::fallback(
                            ImageStatus::Loaded,
                            vec![],
                            vec![text(name.chars().take(1).collect::<String>())],
                        ),
                    ],
                )],
            ),
            item::content(
                vec![],
                vec![
                    item::title(vec![], vec![text(name)]),
                    item::description(vec![], vec![text(job_title)]),
                ],
            ),
            item::actions(vec![], vec![action]),
        ],
    )
}

/// 候補一覧（`layout` は `"list"`/`"grid"`、[`LAYOUT_CSS`] 側の
/// `[data-layout]` セレクタで列数を切り替える）。
fn candidates(layout: &'static str, people: &[(usize, bool)]) -> Node {
    let items = people
        .iter()
        .map(|&(index, added)| {
            let name = dummy_assets::PERSON_NAMES[index];
            let job_title = dummy_assets::JOB_TITLES[index % dummy_assets::JOB_TITLES.len()];
            candidate_item(name, job_title, added)
        })
        .collect();
    div(
        vec![
            ("class", "blocks-empty-state-invite-team-candidates"),
            ("data-layout", layout),
        ],
        items,
    )
}

/// 空状態 + 招待行 + 候補一覧の 1 インスタンス分を組み立てる。
fn invite_block(instance: &'static str, layout: &'static str, people: &[(usize, bool)]) -> Node {
    div(
        vec![("class", "blocks-empty-state-invite-team-instance")],
        vec![
            empty_state::root(
                &EmptyStateProps::default(),
                vec![("data-blocks-empty-state-invite-team-empty", "")],
                vec![
                    empty_state::indicator(vec![], vec![invite_icon()]),
                    empty_state::content(
                        vec![],
                        vec![
                            empty_state::title(vec![], vec![text("まだメンバーがいません")]),
                            empty_state::description(
                                vec![],
                                vec![text(
                                    "チームメンバーをメールで招待するか、おすすめの候補から追加してください。",
                                )],
                            ),
                            empty_state::actions(
                                vec![("data-blocks-empty-state-invite-team-field-wrap", "")],
                                vec![invite_field(instance)],
                            ),
                        ],
                    ),
                ],
            ),
            candidates(layout, people),
        ],
    )
}

/// `empty-state-invite-team` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数（モジュール doc「2 版と集約元の対応」節）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-empty-state-invite-team-stack")],
        vec![
            variant_label("代表構成 + 縦一覧（R0464/R1394）"),
            invite_block("list", "list", &[(0, false), (1, true), (2, false)]),
            variant_label("グリッド候補（R1395）"),
            invite_block(
                "grid",
                "grid",
                &[(3, false), (4, true), (5, false), (6, false)],
            ),
        ],
    )
}
```

## 原案差分メモ

- **版 A（代表構成 + 縦一覧、R0464/R1394）**: 空状態（アイコン・見出し・
  説明）の下に招待用の入力欄を 1 行配置し、その下へ候補 3 名を縦一覧
  （`data-layout="list"`）で並べます。
- **版 B（グリッド、R1395）**: 版 A と同じ骨格のまま、候補を 4 名へ増やし
  2 列グリッド（`data-layout="grid"`）で並べます。
- 追加済みの候補は「追加」ボタンの代わりにチェックマーク付きの「追加済み」
  表示に切り替わります（各版 1 名ずつ）。
- 狭幅（コンテナ幅 36rem 未満）では招待入力欄の入力とボタンが縦積みに
  折り返し、版 B のグリッドは 1 列へ切り替わります（`@container` による
  コンテナクエリ判定）。
- 空状態のアイコン・追加済みのチェックマークは自作の線画（SVG path）で、
  参照元由来のアイコンセットではありません。

関連情報: [Empty State](../themes/empty-state.md) / [Field](../themes/field.md) /
[Input Group](../themes/input-group.md) / [Input](../themes/input.md) /
[Button](../themes/button.md) / [Avatar](../themes/avatar.md) /
[Item](../themes/item.md) / [Text](../themes/text.md)
