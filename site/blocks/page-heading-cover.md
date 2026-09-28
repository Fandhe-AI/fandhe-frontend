# page-heading-cover

`fandhe-frontend-pre-styled-ui` の `image` / `avatar` / `heading` / `button` /
`icon` の 5 部品を合成した、カバー画像付きプロフィール見出しの実例です。
Blocks セクションは新規部品を追加するものではなく、既存の Themes/Primitives
部品を組み合わせた実例集であることに注意してください（主参照は対応表 ID
R1129 のみで、集約元差分はありません。出典の固有名・ファイル名は記載しません）。

上部に横長のカバー画像を配置し、その下端に重なる円形アバター、右に名前、
さらに右にメッセージ・電話の連絡系ボタン 2 個を並べています。狭い幅
（`40rem` 未満）ではアバター・名前・操作ボタンが縦積みになり、ボタンは
画面幅いっぱいに広がります。参照元の配色・文言・アイコンは持ち込まず、
既存の Blocks のトーンに揃えています。

本 Demo は静的な表示例であり、`<form>` 要素は一切持たず、データの取得・
送信・状態管理を行いません。ボタンは `type="button"` のまま送信先を持ちません。
文言・人名・画像はすべて架空のダミー素材であり、実企業名・実クレデンシャル・
PII を含みません。

## Rust コード

```rust
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::image::{self, ImageFit, ImageProps};
use fandhe_frontend_pre_styled_ui::Size;

/// 自作の単純な幾何アイコン（`page_heading_actions.rs::geo_icon` と同型。
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

/// 封筒アイコン（メッセージ送信操作）。
fn mail_icon() -> Node {
    geo_icon("M4 6h16v12H4z M4 6l8 7 8-7")
}

/// 受話器アイコン（電話操作）。
fn phone_icon() -> Node {
    geo_icon(
        "M6 3h4l2 5-2.5 2a11 11 0 0 0 5 5l2-2.5 5 2v4a2 2 0 0 1-2 2A16 16 0 0 1 4 5a2 2 0 0 1 2-2z",
    )
}

/// `page-heading-cover` の Demo 本体（呼び出しごとに同一の `Node` を返す
/// 純関数）。カバー画像 → アバター重なり + 名前 + 操作ボタン 2 個の 1
/// インスタンス構成（集約元が R1129 単独のため併記なし）。
pub fn demo() -> Node {
    let cover = div(
        vec![("data-blocks-page-heading-cover-cover", "")],
        vec![image::image(
            &ImageProps {
                fit: ImageFit::Cover,
                ..ImageProps::new(crate::blocks::dummy_assets::BACKGROUND_SRC, "")
            },
            vec![("data-blocks-page-heading-cover-cover-image", "")],
        )],
    );
    let avatar_node = avatar::root(
        &AvatarProps {
            size: Size::Xl,
            ..AvatarProps::default()
        },
        vec![("data-blocks-page-heading-cover-avatar", "")],
        vec![avatar::image(
            ImageStatus::Loaded,
            crate::blocks::dummy_assets::AVATAR_SRC,
            "",
            vec![],
        )],
    );
    let name = heading(
        HeadingLevel::H1,
        &HeadingProps {
            size: HeadingSize::Xl,
            ..HeadingProps::default()
        },
        vec![("data-blocks-page-heading-cover-name", "")],
        vec![text(crate::blocks::dummy_assets::PERSON_NAMES[0])],
    );
    let actions = div(
        vec![("data-blocks-page-heading-cover-actions", "")],
        vec![
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Outline,
                    ..ButtonProps::default()
                },
                vec![],
                vec![mail_icon(), text("メッセージ")],
            ),
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Outline,
                    ..ButtonProps::default()
                },
                vec![],
                vec![phone_icon(), text("電話")],
            ),
        ],
    );
    let header = div(
        vec![("data-blocks-page-heading-cover-header", "")],
        vec![avatar_node, name, actions],
    );
    div(
        vec![("class", "blocks-page-heading-cover-layout")],
        vec![cover, header],
    )
}
```

## 原案差分メモ

- 主参照は R1129 のみで、集約元差分はありません（併記インスタンスなし）。
- 参照元の配色・文言・アイコンは持ち込まず、既存 Blocks のトーンに揃えています。
- ボタンは押下先を持たない静的表示です。
- ブラウザ実機での目視確認は、サンドボックス環境の制約により未実施です
  （実装ステップ・検証方法は Issue 本文・実装計画を参照）。
