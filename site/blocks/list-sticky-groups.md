# list-sticky-groups

`list` / `avatar` / `heading` / `scroll-area` の 4 部品を合成した、頭文字ごとにグループ化した人物ディレクトリの実例です。Blocks セクションは新規部品を追加するものではなく、既存の Themes/Primitives 部品を組み合わせた実例集であることに注意してください（主参照は対応表 ID R1293。出典の固有名・ファイル名は記載しません）。

- 静的表示です。docs サイトは無 JS のため、状態機械・フォーム・データ取得を持たない固定描画にしています。
- 文言・人名・メールアドレスはすべて架空のものです（メールは予約ドメイン `example.com` を使い、`mailto:` リンクにはしていません）。
- データ取得・送信は行わず、`<form>` は使いません。ボタンも置きません。
- 各グループの見出し（頭文字）は `position: sticky` のみで実現しており、固定高（20rem）のスクロール領域内でスクロールするとその場で貼り付き・押し出しの挙動を確かめられます。
- スクロール領域は `scroll-area` の Viewport（`role="region"` + `aria-label` 付き）で、キーボードでもスクロールできます。
- アバターは画像を使わず、名前の頭文字を fallback 表示にしています。

## Rust コード

```rust
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::list::{self, ListType, ListVariant};
use fandhe_frontend_pre_styled_ui::scroll_area;
use fandhe_frontend_pre_styled_ui::Size;

/// 頭文字グループ順の人物ディレクトリ（架空。頭文字と (名前, メール) の組の
/// 配列）。あらかじめ頭文字順に並べた状態で持ち、実行時にソート・グループ
/// 化は行わない。スクロールが確実に起きる件数（6 グループ・合計 16 人）に
/// している。
const GROUPS: &[(&str, &[(&str, &str)])] = &[
    (
        "A",
        &[
            ("安藤 明", "akira.ando@example.com"),
            ("荒木 愛子", "aiko.araki@example.com"),
        ],
    ),
    (
        "C",
        &[
            ("千葉 智也", "tomoya.chiba@example.com"),
            ("近藤 千夏", "chinatsu.kondo@example.com"),
            ("千田 治郎", "jiro.chida@example.com"),
        ],
    ),
    (
        "F",
        &[
            ("藤原 文子", "fumiko.fujiwara@example.com"),
            ("福田 太郎", "taro.fukuda@example.com"),
        ],
    ),
    (
        "M",
        &[
            ("松本 まどか", "madoka.matsumoto@example.com"),
            ("三浦 実", "minoru.miura@example.com"),
            ("森田 美咲", "misaki.morita@example.com"),
            ("宮本 学", "manabu.miyamoto@example.com"),
        ],
    ),
    (
        "S",
        &[
            ("佐々木 進", "susumu.sasaki@example.com"),
            ("清水 幸子", "sachiko.shimizu@example.com"),
            ("杉山 聡", "satoshi.sugiyama@example.com"),
        ],
    ),
    (
        "Y",
        &[
            ("山口 洋子", "yoko.yamaguchi@example.com"),
            ("吉田 裕也", "yuya.yoshida@example.com"),
        ],
    ),
];

/// 名前の先頭 1 文字をアバターのフォールバック表示に使う
/// （`content_article`/`list_narrow_activity` と同じ判断）。
fn initial_avatar(name: &str) -> Node {
    let initial: String = name
        .split_whitespace()
        .filter_map(|part| part.chars().next())
        .collect();
    avatar::root(
        &AvatarProps {
            size: Size::Sm,
            ..AvatarProps::default()
        },
        vec![],
        vec![avatar::fallback(
            ImageStatus::Error,
            vec![],
            vec![text(initial)],
        )],
    )
}

/// 1 行（アバター + 名前 + メール）を組み立てる。
fn person_row(name: &str, email: &str) -> Node {
    list::item(
        vec![("data-blocks-list-sticky-groups-row", "")],
        vec![
            initial_avatar(name),
            div(
                vec![],
                vec![
                    div(
                        vec![("data-blocks-list-sticky-groups-name", "")],
                        vec![text(name)],
                    ),
                    div(
                        vec![("data-blocks-list-sticky-groups-email", "")],
                        vec![text(email)],
                    ),
                ],
            ),
        ],
    )
}

/// 1 グループ（sticky 見出し + 人物一覧）を組み立てる。
fn group_section(initial: &str, people: &[(&str, &str)]) -> Node {
    div(
        vec![("data-blocks-list-sticky-groups-group", "")],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Sm,
                    ..HeadingProps::default()
                },
                vec![("data-blocks-list-sticky-groups-heading", "")],
                vec![text(initial)],
            ),
            list::root(
                ListType::Unordered,
                ListVariant::Plain,
                vec![],
                people
                    .iter()
                    .map(|(name, email)| person_row(name, email))
                    .collect(),
            ),
        ],
    )
}

/// `list-sticky-groups` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数（モジュール doc「静的表示」節）。
pub fn demo() -> Node {
    div(
        vec![("data-blocks-list-sticky-groups-frame", "")],
        vec![scroll_area::root(
            vec![("data-blocks-list-sticky-groups-scroll", "")],
            vec![scroll_area::viewport(
                vec![("role", "region"), ("aria-label", "People directory")],
                vec![scroll_area::content(
                    vec![],
                    GROUPS
                        .iter()
                        .map(|(initial, people)| group_section(initial, people))
                        .collect(),
                )],
            )],
        )],
    )
}
```

## 原案差分メモ

- 集約元は R1293（代表構成）の 1 件のみです。並記や他 ID との合成は行っていません。
- 参照元の文言・配色・アイコンは使用せず、独自に書いています。実在の人物・企業名等は含みません。
- アバターは画像を使わず、名前の頭文字を fallback 表示にしています。
- メールアドレスは `link` 部品を使わずプレーンテキストで表示し、予約ドメイン `example.com` のみを使っています。
