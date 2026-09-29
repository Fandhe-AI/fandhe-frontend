# card-heading-basic

`fandhe-frontend-pre-styled-ui` の `card` / `heading` / `text` / `avatar` /
`button` / `menu` / `link` 部品を合成した、カード上部の最小構成区画見出しの
実例です。Blocks セクションは新規部品を追加するものではなく、既存の
Themes/Primitives 部品を組み合わせた実例集であることに注意してください
（主参照は対応表 ID R0808、集約元は R0809〜R0813。出典の固有名・ファイル名は
記載しません）。

6 形を縦に並べています（`data-blocks-card-heading-basic-variant` で区別
できます）。

1. 題名のみ（基準形）
2. 題名 + 操作ボタン
3. アバター + 輪郭ボタン 2 個
4. 題名 + 説明文 + 操作ボタン + テキストリンク
5. 題名 + 説明文のみ
6. アバター + メタ情報 + 三点メニュー

下罫線の有無は形により異なり、`data-blocks-card-heading-basic-divider`
属性で表現しています（1・2・4 に付与、3・5・6 には付与しません）。

本 Demo は静的な表示例であり、`<form>` 要素は一切持たず、データの取得・
送信・状態管理を行いません。ボタンは `type="button"` のまま送信先を持たず、
三点メニューは閉じた状態の固定表示です（開閉には `fandhe-frontend-wasm-full`
の JS 配線が必要で、docs サイトは JS ハイドレーションを行いません）。文言は
すべて独自に書いた架空のものであり、実企業名・実クレデンシャル・PII を
含みません。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::menu::{self, OpenState};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};
use fandhe_frontend_pre_styled_ui::Size;

/// テキストリンクの遷移先（外部の実在 URL、`href="#"` は使わない。
/// モジュール doc「リンクは `href="#"` を使わない」節参照）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// 見出し（H3）。docs ページ自体が H1 を持つため block 内は H2 以下とする
/// （`card_heading_toolbar.rs` と同型の判断）。
fn title(label: &str) -> Node {
    heading::heading(
        HeadingLevel::H3,
        &HeadingProps {
            size: HeadingSize::Md,
            ..HeadingProps::default()
        },
        vec![],
        vec![text(label)],
    )
}

/// 説明文・メタ情報用の淡色テキスト。
fn muted(label: &str) -> Node {
    styled_text::text(
        &TextProps {
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![],
        vec![text(label)],
    )
}

/// アバター（円形）。氏名をアクセシブルネームとして `root` へ直接付与し、
/// フォールバックは氏名の先頭 1 文字を表示する（`page_heading_avatar.rs::
/// profile_avatar` と同型）。
fn person_avatar(name: &str) -> Node {
    let initial: String = name.chars().take(1).collect();
    avatar::root(
        &AvatarProps {
            size: Size::Md,
            ..AvatarProps::default()
        },
        vec![("role", "img"), ("aria-label", name)],
        vec![
            avatar::image(ImageStatus::Loaded, dummy_assets::AVATAR_SRC, "", vec![]),
            avatar::fallback(ImageStatus::Loaded, vec![], vec![text(initial)]),
        ],
    )
}

/// 三点メニュー（無 JS のため閉じた状態で固定する。モジュール doc
/// 「三点メニューは無 JS のため閉じた状態で固定する」節参照）。
fn overflow_menu() -> Node {
    const CONTENT_ID: &str = "blocks-card-heading-basic-menu";
    let trigger = menu::trigger(
        OpenState::Closed,
        false,
        Some(CONTENT_ID),
        vec![("aria-label", "その他の操作")],
        vec![text("\u{2026}")],
    );
    let content = menu::content(
        OpenState::Closed,
        Some(CONTENT_ID),
        None,
        vec![],
        vec![
            menu::item("edit", false, false, vec![], vec![text("編集する")]),
            menu::item(
                "archive",
                false,
                false,
                vec![],
                vec![text("アーカイブする")],
            ),
            menu::separator(vec![], vec![]),
            menu::item("delete", false, false, vec![], vec![text("削除する")]),
        ],
    );
    let positioner = menu::positioner(OpenState::Closed, vec![], vec![content]);
    menu::root(
        Size::Sm,
        OpenState::Closed,
        vec![],
        vec![trigger, positioner],
    )
}

/// カード本文（点線枠の短いプレースホルダ。見出しが「カード上部」で
/// あることを示すためのダミー本文）。
fn placeholder_body() -> Node {
    card::body(
        vec![],
        vec![div(
            vec![("data-blocks-card-heading-basic-placeholder", "")],
            vec![muted("ここにカードの本文が入ります。")],
        )],
    )
}

/// カード 1 枚（見出し行 + プレースホルダ本文）を組み立てる。`divider` は
/// 下罫線の有無（モジュール doc「下罫線の有無は形により異なる」節参照）。
fn panel(variant: &'static str, divider: bool, header_content: Node) -> Node {
    let mut header_attrs: Vec<(&str, &str)> = vec![("data-blocks-card-heading-basic-header", "")];
    if divider {
        header_attrs.push(("data-blocks-card-heading-basic-divider", ""));
    }
    card::root(
        CardProps::default(),
        vec![("data-blocks-card-heading-basic-variant", variant)],
        vec![
            card::header(header_attrs, vec![header_content]),
            placeholder_body(),
        ],
    )
}

/// 1. 題名のみ（R0808・主参照）。
fn variant_title() -> Node {
    panel("title", true, title("プロジェクト概要"))
}

/// 2. 題名 + 操作ボタン。
fn variant_title_actions() -> Node {
    let header = div(
        vec![("data-blocks-card-heading-basic-header-row", "")],
        vec![
            title("チームメンバー"),
            button::button(&ButtonProps::default(), vec![], vec![text("招待する")]),
        ],
    );
    panel("title-actions", true, header)
}

/// 3. アバター + 見出し群、右に輪郭ボタン 2 個。
fn variant_avatar_outline() -> Node {
    let name = dummy_assets::PERSON_NAMES[0];
    let group = div(
        vec![("data-blocks-card-heading-basic-group", "")],
        vec![
            person_avatar(name),
            div(
                vec![("data-blocks-card-heading-basic-stack", "")],
                vec![title(name), muted("プラットフォームチーム")],
            ),
        ],
    );
    let actions = div(
        vec![("data-blocks-card-heading-basic-actions", "")],
        vec![
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Outline,
                    ..ButtonProps::default()
                },
                vec![],
                vec![text("メッセージ")],
            ),
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Outline,
                    ..ButtonProps::default()
                },
                vec![],
                vec![text("プロフィール")],
            ),
        ],
    );
    let header = div(
        vec![("data-blocks-card-heading-basic-header-row", "")],
        vec![group, actions],
    );
    panel("avatar-outline", false, header)
}

/// 4. 題名 + 説明文 + 操作ボタン + テキストリンク。
fn variant_title_description_actions() -> Node {
    let stack = div(
        vec![("data-blocks-card-heading-basic-stack", "")],
        vec![
            title("リリースノート"),
            muted("直近の更新内容と既知の不具合をまとめています。"),
        ],
    );
    let actions = div(
        vec![("data-blocks-card-heading-basic-actions", "")],
        vec![
            button::button(&ButtonProps::default(), vec![], vec![text("公開する")]),
            link::root(
                REPO,
                &LinkProps {
                    external: true,
                    ..LinkProps::default()
                },
                vec![],
                vec![text("GitHub で見る")],
            ),
        ],
    );
    let header = div(
        vec![("data-blocks-card-heading-basic-header-row", "")],
        vec![stack, actions],
    );
    panel("title-description-actions", true, header)
}

/// 5. 題名 + 説明文のみ（操作なし）。
fn variant_title_description() -> Node {
    let stack = div(
        vec![("data-blocks-card-heading-basic-stack", "")],
        vec![
            title("利用状況"),
            muted("今月のアクセス数と利用傾向の概要です。"),
        ],
    );
    panel("title-description", false, stack)
}

/// 6. アバター + メタ情報（投稿者名・日時） + 三点メニュー。
fn variant_avatar_meta_menu() -> Node {
    let name = dummy_assets::PERSON_NAMES[1];
    let group = div(
        vec![("data-blocks-card-heading-basic-group", "")],
        vec![
            person_avatar(name),
            div(
                vec![("data-blocks-card-heading-basic-stack", "")],
                vec![title(name), muted("9 月 26 日に投稿")],
            ),
        ],
    );
    let header = div(
        vec![("data-blocks-card-heading-basic-header-row", "")],
        vec![group, overflow_menu()],
    );
    panel("avatar-meta-menu", false, header)
}

/// `card-heading-basic` の Demo 本体（6 形を縦積みで併記する。呼び出し
/// ごとに同一の `Node` を返す純関数）。
#[must_use]
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-card-heading-basic-layout")],
        vec![
            variant_title(),
            variant_title_actions(),
            variant_avatar_outline(),
            variant_title_description_actions(),
            variant_title_description(),
            variant_avatar_meta_menu(),
        ],
    )
}
```

## 集約元との差分メモ

- 主参照（対応表 ID R0808）は「題名のみ」の基準形（`title`）です。集約元
  （対応表 ID R0809〜R0813）はそれぞれ操作ボタン・アバター・説明文・
  メニューの有無が異なる派生形で、本 Demo では次の 6 インスタンスとして
  併記しています（`data-blocks-card-heading-basic-variant` の値）。
  - `title`: 題名のみ（R0808・主参照）
  - `title-actions`: 題名 + 操作ボタン
  - `avatar-outline`: アバター + 輪郭ボタン 2 個
  - `title-description-actions`: 題名 + 説明文 + 操作ボタン + テキストリンク
  - `title-description`: 題名 + 説明文のみ
  - `avatar-meta-menu`: アバター + メタ情報 + 三点メニュー
- 下罫線は `title`・`title-actions`・`title-description-actions` の 3 形に
  付与し、`avatar-outline`・`title-description`・`avatar-meta-menu` の
  3 形には付与していません。
- テキストリンクの遷移先は実在の公開リポジトリ URL です。特定の応募内容・
  記事等の実在しないページを指すような固有のラベルは避け、遷移先がわかる
  「GitHub で見る」を可視テキストにしています。
- 三点メニューは閉じた状態の固定表示です。トリガーを押しても開きません
  （wasm-full の JS 配線がある実アプリでは操作できます）。
- 文言・配色は既存のテーマトークンに従い、独自に書いた架空のものです。

関連情報: [Card](../themes/card.md) / [Heading](../themes/heading.md) /
[Text](../themes/text.md) / [Avatar](../themes/avatar.md) /
[Button](../themes/button.md) / [Menu](../themes/menu.md) /
[Link](../themes/link.md)
