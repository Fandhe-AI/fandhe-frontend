# auth-oauth-consent

`fandhe-frontend-pre-styled-ui` の `card` / `avatar` / `icon` / `list` /
`separator` / `button` / `link` / `heading` 部品を合成した、OAuth 連携の
同意画面（同意カード）の実例です。Blocks セクションは新規部品を追加する
ものではなく、既存の Themes/Primitives 部品を組み合わせた実例集であること
に注意してください（主参照・集約元ともに対応表 ID R0250、差分なし。出典の
固有名・ファイル名は記載しません）。

カード上部に連携元アプリ（Northwind Notes）と連携先アプリ（Aurora
Calendar）のアイコンを並べた見出し、区切り線を挟んで要求権限の一覧
（アイコン付き）、区切り線を挟んでアカウント表示（アバター・氏名・
メールアドレス・別アカウント切り替え）、カード下部に「拒否」「許可」の
2 ボタンを配置しています。カード下には連携の取り消し方法を案内する補足
リンクを置いています。狭い画面幅ではカードが親要素の幅いっぱいに広がり、
「拒否」「許可」ボタンは横並びから縦積みへ切り替わります（`40rem` を
境界値とします）。

本 Demo は静的な表示例であり、`<form>` 要素は一切持たず、実際の認可処理・
アクセストークンの発行・リダイレクトを行いません。「拒否」「許可」
ボタンは `type="button"` のまま送信先を持たず、`redirect_uri` /
`client_id` のような実パラメータを模した属性・URL も出力しません。
「別のアカウントを使う」ボタンも同様に遷移しません（無 JS の docs サイト
のため実際のアカウント切り替えは行いません）。アプリのアイコン・
アカウントのアバターはいずれもモノトーン抽象図形のプレースホルダーであり
実ブランドロゴではなく、権限アイコンも自作の単純幾何図形です。氏名・
メールアドレスはすべて独自に書いた架空のものであり、実在の人物・企業・
クレデンシャルとは無関係です。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::list::{self, ListType, ListVariant};
use fandhe_frontend_pre_styled_ui::separator::{self, SeparatorProps};
use fandhe_frontend_pre_styled_ui::Size;

/// 補足リンクの遷移先（外部の実在 URL、`href="#"` は使わない、
/// `page_heading_avatar.rs::REPO` と同型の判断。アクセス許可の仕組み自体を
/// 説明する実在ページは持たないため、遷移先がわかる文言「GitHub で見る」を
/// 可視テキストにする）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// 連携元/連携先アプリのアイコン（円形アバター）。`role_hint` は
/// `"source"`/`"target"` を CSS フックとして与え、実ブランドロゴは使わず
/// `dummy_assets::LOGO_SRC` を共通で使い回す。
fn app_avatar(name: &str, role_hint: &'static str) -> Node {
    let initial: String = name.chars().take(1).collect();
    avatar::root(
        &AvatarProps {
            size: Size::Lg,
            ..AvatarProps::default()
        },
        vec![
            ("role", "img"),
            ("aria-label", name),
            ("data-blocks-auth-oauth-consent-app", role_hint),
        ],
        vec![
            avatar::image(ImageStatus::Loaded, dummy_assets::LOGO_SRC, "", vec![]),
            avatar::fallback(ImageStatus::Loaded, vec![], vec![text(initial)]),
        ],
    )
}

/// 権限アイコン（自作の単純幾何パス、装飾専用）。`login_04.rs::geo_icon` と
/// 同型で、実ブランドロゴ・既存アイコンセットを複製しない。
fn scope_icon(path_d: &'static str) -> Node {
    icon(
        &IconProps {
            size: Size::Sm,
            ..IconProps::default()
        },
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

/// 要求権限 1 件（アイコン + タイトル + 説明）。`list::indicator` は常に
/// `aria-hidden="true"` を固定するため、アイコンは装飾として扱う
/// （`pricing_upgrade_card.rs::check_icon` と同型）。
fn scope_item(path_d: &'static str, title: &'static str, desc: &'static str) -> Node {
    list::item(
        vec![],
        vec![
            list::indicator(vec![], vec![scope_icon(path_d)]),
            div(
                vec![("class", "blocks-auth-oauth-consent-scope-body")],
                vec![
                    el("span", vec![], vec![text(title)]),
                    el(
                        "span",
                        vec![("class", "blocks-auth-oauth-consent-scope-desc")],
                        vec![text(desc)],
                    ),
                ],
            ),
        ],
    )
}

/// アカウント表示欄のアバター（円形、小サイズ）。
fn account_avatar(name: &str) -> Node {
    let initial: String = name.chars().take(1).collect();
    avatar::root(
        &AvatarProps {
            size: Size::Sm,
            ..AvatarProps::default()
        },
        vec![("role", "img"), ("aria-label", name)],
        vec![
            avatar::image(ImageStatus::Loaded, dummy_assets::AVATAR_SRC, "", vec![]),
            avatar::fallback(ImageStatus::Loaded, vec![], vec![text(initial)]),
        ],
    )
}

/// `auth-oauth-consent` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数。
#[must_use]
pub fn demo() -> Node {
    let name = dummy_assets::PERSON_NAMES[0];

    let apps_row = div(
        vec![("class", "blocks-auth-oauth-consent-apps")],
        vec![
            app_avatar("Northwind Notes", "source"),
            scope_icon("M5 12h14M13 6l6 6-6 6"),
            app_avatar("Aurora Calendar", "target"),
        ],
    );

    let heading_node = heading::heading(
        HeadingLevel::H3,
        &HeadingProps {
            size: HeadingSize::Lg,
            ..HeadingProps::default()
        },
        vec![],
        vec![text(
            "Northwind Notes が Aurora Calendar へのアクセスを求めています",
        )],
    );

    let scopes = list::root(
        ListType::Unordered,
        ListVariant::Plain,
        vec![("data-blocks-auth-oauth-consent-scopes", "")],
        vec![
            scope_item(
                "M12 12a4 4 0 100-8 4 4 0 000 8zM4 20c0-3.3 3.6-6 8-6s8 2.7 8 6",
                "プロフィール情報",
                "氏名とアイコン画像を読み取ります",
            ),
            scope_item(
                "M4 5h16v14H4zM4 7l8 6 8-6",
                "カレンダーの予定",
                "予定の一覧を読み取り、新規作成します",
            ),
            scope_item(
                "M6 3v4M18 3v4M4 9h16M5 5h14a1 1 0 011 1v13a1 1 0 01-1 1H5a1 1 0 01-1-1V6a1 1 0 011-1z",
                "空き時間",
                "会議の空き状況を確認します",
            ),
        ],
    );

    let account_row = div(
        vec![("class", "blocks-auth-oauth-consent-account")],
        vec![
            account_avatar(name),
            div(
                vec![],
                vec![
                    el("span", vec![], vec![text(name)]),
                    el(
                        "span",
                        vec![("class", "blocks-auth-oauth-consent-email")],
                        vec![text("yamada@example.com")],
                    ),
                ],
            ),
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Link,
                    ..ButtonProps::default()
                },
                vec![],
                vec![text("別のアカウントを使う")],
            ),
        ],
    );

    let card_node = card::root(
        CardProps::default(),
        vec![("data-blocks-auth-oauth-consent-card", "")],
        vec![
            card::header(
                vec![],
                vec![
                    apps_row,
                    heading_node,
                    card::description(vec![], vec![text("続行すると、次の権限が付与されます")]),
                ],
            ),
            card::body(
                vec![],
                vec![
                    separator::separator(
                        &SeparatorProps::default(),
                        vec![("data-blocks-auth-oauth-consent-divider", "")],
                    ),
                    scopes,
                    separator::separator(
                        &SeparatorProps::default(),
                        vec![("data-blocks-auth-oauth-consent-divider", "")],
                    ),
                    account_row,
                ],
            ),
            card::footer(
                vec![("data-blocks-auth-oauth-consent-actions", "")],
                vec![
                    button::button(
                        &ButtonProps {
                            variant: ButtonVariant::Outline,
                            ..ButtonProps::default()
                        },
                        vec![],
                        vec![text("拒否")],
                    ),
                    button::button(&ButtonProps::default(), vec![], vec![text("許可")]),
                ],
            ),
        ],
    );

    let help_row = div(
        vec![("class", "blocks-auth-oauth-consent-help")],
        vec![
            text("この連携はいつでも設定画面から取り消せます。"),
            link::root(
                REPO,
                &LinkProps {
                    external: true,
                    ..LinkProps::default()
                },
                vec![],
                vec![text("アクセス許可の仕組みを GitHub で見る")],
            ),
        ],
    );

    div(
        vec![("class", "blocks-auth-oauth-consent-layout")],
        vec![card_node, help_row],
    )
}
```

## 原案差分メモ

- 主参照・集約元ともに対応表 ID R0250 の 1 件のみで、差分はありません。
  `_/blocks-intake/` の対応ファイルは本イシュー着手時点で本 worktree に
  存在しないため、レイアウト仕様（イシュー本文の文言）のみから構成して
  います。
- イシュー本文が使用部品として `separator` を明記しているため、見出し↔
  一覧・一覧↔アカウント表示の 2 箇所へ `separator` 部品を実体として
  置いています。一覧項目どうしの区切り線は `separator` 部品を `<li>` の
  中へ挟まずセマンティクスを壊さないため、CSS の隣接兄弟セレクタで
  描いています。
- 実際の認可・トークン発行・リダイレクトは行わず、静的な初期状態のみを
  示します。連携元/連携先アプリ名・氏名・メールアドレスはすべて独自の
  架空データです。
- ブラウザでの実機確認（`40rem` 前後のコンテナ幅切替・ライト/ダーク両
  テーマ）はサンドボックス制約により未実施です。cargo test による出力
  検証のみで代替しました。
