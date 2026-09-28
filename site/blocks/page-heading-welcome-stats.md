# page-heading-welcome-stats

`fandhe-frontend-pre-styled-ui` の `card` / `avatar` / `heading` / `text` /
`button` / `stat` / `separator` 部品を合成した、統計付きウェルカム見出し
カードの実例です。Blocks セクションは新規部品を追加するものではなく、
既存の Themes/Primitives 部品を組み合わせた実例集であることに注意して
ください（主参照は対応表 ID R1131。集約元は同一 1 件のみで差分はありませ
ん。出典の固有名・ファイル名は記載しません）。

カード上段に小さなアバターと挨拶文・名前・役職を並べ、右端にプロフィール
への導線ボタンを配置します。区切り線を挟んだ下段には、区切り線で 3 分割
した統計値の帯（完了タスク数・未読の通知数・進行中のプロジェクト数）を
表示します。`40rem` 未満の狭い幅では上段が縦積みになり、統計帯も 1 列の
縦並びへ切り替わります。

本 Demo は静的な表示例であり、`<form>` 要素は一切持たず、データの取得・
送信・状態管理を行いません。ボタンは `type="button"` のまま送信先を持ちま
せん。文言・数値はすべて独自に書いた架空のものであり、実企業名・実クレデ
ンシャル・PII を含みません。

## Rust コード

```rust
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::separator::{self, SeparatorProps};
use fandhe_frontend_pre_styled_ui::stat;
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};
use fandhe_frontend_pre_styled_ui::Size;

/// 1 件分の統計（ラベル・値）を `stat` パーツへ組み立てる。
fn stat_item(label: &'static str, value: &'static str) -> Node {
    stat::root(
        Size::Md,
        vec![("data-blocks-page-heading-welcome-stats-stat", "")],
        vec![
            stat::label(vec![], vec![text(label)]),
            stat::value_text(vec![], vec![text(value)]),
        ],
    )
}

/// `page-heading-welcome-stats` の Demo 本体（呼び出しごとに同一の
/// `Node` を返す純関数）。
pub fn demo() -> Node {
    let name = dummy_assets::PERSON_NAMES[0];
    let title = dummy_assets::JOB_TITLES[0];

    let identity = div(
        vec![("data-blocks-page-heading-welcome-stats-identity", "")],
        vec![
            avatar::root(
                &AvatarProps {
                    size: Size::Lg,
                    ..AvatarProps::default()
                },
                vec![],
                vec![avatar::image(
                    ImageStatus::Loaded,
                    dummy_assets::AVATAR_SRC,
                    "",
                    vec![],
                )],
            ),
            div(
                vec![("data-blocks-page-heading-welcome-stats-greeting", "")],
                vec![
                    styled_text::text(
                        &TextProps {
                            variant: TextVariant::Muted,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text("おはようございます")],
                    ),
                    heading(
                        HeadingLevel::H1,
                        &HeadingProps {
                            size: HeadingSize::Lg,
                            ..HeadingProps::default()
                        },
                        vec![],
                        vec![text(name)],
                    ),
                    styled_text::text(
                        &TextProps {
                            variant: TextVariant::Muted,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text(title)],
                    ),
                ],
            ),
        ],
    );

    let header = card::header(
        vec![("data-blocks-page-heading-welcome-stats-header", "")],
        vec![
            identity,
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Outline,
                    ..ButtonProps::default()
                },
                vec![],
                vec![text("プロフィールを見る")],
            ),
        ],
    );

    let stats = card::body(
        vec![("data-blocks-page-heading-welcome-stats-stats", "")],
        vec![
            stat_item("今週の完了タスク", "12"),
            stat_item("未読の通知", "3"),
            stat_item("進行中のプロジェクト", "5"),
        ],
    );

    let card_node = card::root(
        CardProps::default(),
        vec![],
        vec![
            header,
            separator::separator(&SeparatorProps::default(), vec![]),
            stats,
        ],
    );

    div(
        vec![("class", "blocks-page-heading-welcome-stats-layout")],
        vec![card_node],
    )
}
```

## 集約元との差分メモ

参照（対応表 ID R1131、基準形）からの意図的な差分は次のとおりです。

- 集約元は主参照と同一の 1 件のみで、差分はありません。
- アバター画像は共通のダミー素材（人物アバター SVG）を使いました。
- 名前・役職は既存 block と共通の架空のダミー文言セットを使いました。
- 統計 3 件の区切り線は `separator` 部品ではなく CSS の罫線で表現しま
  した。静的な `Node` は幅に応じて `aria-orientation` を切り替えられず、
  狭幅で意味論と見た目が食い違うため（本ファイルのモジュール doc参照）。
  `separator` 部品自体は上段/下段を区切る水平の 1 本として使っています。
- 数値・文言はすべて架空の値へ置き換えました。
- 配色・文言は既存のテーマトークンに従い、独自に書いた架空のものです。

関連情報: [Card](../themes/card.md) / [Avatar](../themes/avatar.md) /
[Heading](../themes/heading.md) / [Text](../themes/text.md) /
[Button](../themes/button.md) / [Stat](../themes/stat.md) /
[Separator](../themes/separator.md)
