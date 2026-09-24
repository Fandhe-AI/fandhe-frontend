# changelog-stacked-list

`heading` / `text` / `badge` / `card` / `list` / `link-overlay` / `separator`
を合成した、リリースを縦に積む changelog です。

見出しとリード文の下に、区切り方の異なる 2 つの表示例を並べています。
1 つ目（罫線区切り、対応表 ID R0045・主参照）はタイトル・日付・タグ・
説明・変更点リストをエントリごとに `separator` で区切ります。2 つ目
（カード + 全面リンク、対応表 ID R0044・副参照）はエントリを `card` に
収め、カード全体を `link-overlay` でクリック可能にします（変更点リストは
省略します）。文言・バージョン番号・日付はすべて架空のもので、データ
取得・送信は行わない静的な表示例です。リンク先はすべてリポジトリへの
固定リンクです。`<form>` は使用しません。狭い幅でもタイトル側が縮み、
日付は折り返さず 1 行で表示されます。

## Rust コード

```rust
use fandhe_frontend_core::{article, div, el, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps, BadgeVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps, CardVariant};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::link_overlay::{self, overlay};
use fandhe_frontend_pre_styled_ui::list::{self, ListType, ListVariant};
use fandhe_frontend_pre_styled_ui::separator::{separator, SeparatorProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};

/// リンク先の固定外部 URL（モジュール doc「`href="#"` を使わない」節参照）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// リリース 1 件分のダミーデータ（架空、実在の製品・企業とは無関係）。
struct Release {
    version: &'static str,
    date_iso: &'static str,
    date_label: &'static str,
    title: &'static str,
    tags: &'static [&'static str],
    description: &'static str,
    changes: &'static [&'static str],
}

/// リリース一覧（架空、3 件）。1 件目はタイトルを長めにし、狭い幅でも
/// 日付が折り返さずタイトル側だけが縮むことを示す（モジュール doc「狭い
/// 幅でも日付を折り返さない」節）。
const RELEASES: [Release; 3] = [
    Release {
        version: "v3.4.0",
        date_iso: "2026-09-20",
        date_label: "2026年9月20日",
        title: "検索インデックスの決定性検証とページ分割の下準備を追加",
        tags: &["Feature", "Search"],
        description: "全文検索インデックスの生成手順を見直し、将来のセクション単位分割に備えた土台を整えました。",
        changes: &[
            "検索インデックスのビルド時サイズ計測を強化",
            "索引対象からナビゲーション専用ページを除外",
        ],
    },
    Release {
        version: "v3.3.2",
        date_iso: "2026-09-12",
        date_label: "2026年9月12日",
        title: "カード合成部品のフォーカスリング角丸追従を修正",
        tags: &["Fix", "Accessibility"],
        description: "カード全面リンク化時にフォーカスリングが角丸に追従しない不具合を修正しました。",
        changes: &["border-radius の継承連鎖を 2 段へ拡張"],
    },
    Release {
        version: "v3.3.1",
        date_iso: "2026-09-02",
        date_label: "2026年9月2日",
        title: "変更履歴ページのテンプレートを整理",
        tags: &["Chore"],
        description: "変更履歴の記述フォーマットを統一し、リリースごとの差分を追いやすくしました。",
        changes: &["タグの表記ゆれを解消", "日付表記を ISO 8601 に統一"],
    },
];

/// `<time datetime>` を組み立てる（モジュール doc「`<time datetime>` と
/// 表示日付の一致」節）。
fn release_date(iso: &str, label: &str) -> Node {
    el(
        "time",
        vec![
            ("class", "blocks-changelog-stacked-list-date"),
            ("datetime", iso),
        ],
        vec![text(label)],
    )
}

/// タイトル行（タイトル左・日付右の両端揃え）。
fn title_row(release: &Release, level: HeadingLevel) -> Node {
    div(
        vec![("class", "blocks-changelog-stacked-list-title-row")],
        vec![
            heading(
                level,
                &HeadingProps::default(),
                vec![("data-blocks-changelog-stacked-list-title", "")],
                vec![text(format!("{} — {}", release.version, release.title))],
            ),
            release_date(release.date_iso, release.date_label),
        ],
    )
}

/// タグの badge 群。
fn tag_row(release: &Release, featured: bool) -> Node {
    let mut badges: Vec<Node> = release
        .tags
        .iter()
        .map(|tag| {
            badge::badge(
                &BadgeProps::default(),
                vec![("data-blocks-changelog-stacked-list-tag", "")],
                vec![text(*tag)],
            )
        })
        .collect();
    if featured {
        badges.insert(
            0,
            badge::badge(
                &BadgeProps {
                    variant: BadgeVariant::Solid,
                    ..BadgeProps::default()
                },
                vec![("data-blocks-changelog-stacked-list-tag-latest", "")],
                vec![text("最新")],
            ),
        );
    }
    div(
        vec![("class", "blocks-changelog-stacked-list-tags")],
        badges,
    )
}

/// 罫線区切り表示（R0045、主参照）のリリース 1 件。
fn stacked_entry(release: &Release, featured: bool) -> Node {
    article(
        vec![("class", "blocks-changelog-stacked-list-entry")],
        vec![
            title_row(release, HeadingLevel::H4),
            tag_row(release, featured),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(release.description)],
            ),
            list::root(
                ListType::Unordered,
                ListVariant::Marker,
                vec![("data-blocks-changelog-stacked-list-changes", "")],
                release
                    .changes
                    .iter()
                    .map(|c| list::item(vec![], vec![text(*c)]))
                    .collect(),
            ),
        ],
    )
}

/// カード + 全面リンク化表示（R0044、副参照）のリリース 1 件。変更点
/// リストは省く（原稿「原案差分メモ」参照）。
fn card_entry(release: &Release) -> Node {
    card::root(
        CardProps {
            variant: CardVariant::Outline,
            ..CardProps::default()
        },
        vec![("data-blocks-changelog-stacked-list-card", "")],
        vec![link_overlay::root(
            vec![("data-blocks-changelog-stacked-list-card-link", "")],
            vec![
                card::body(
                    vec![],
                    vec![
                        title_row(release, HeadingLevel::H4),
                        tag_row(release, false),
                        styled_text::text(
                            &TextProps {
                                variant: TextVariant::Muted,
                                ..TextProps::default()
                            },
                            vec![],
                            vec![text(release.description)],
                        ),
                    ],
                ),
                overlay(REPO, vec![("aria-label", release.title)], vec![]),
            ],
        )],
    )
}

/// `changelog-stacked-list` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数（モジュール doc「2 つの表示例を並べる理由」節）。
pub fn demo() -> Node {
    let header = div(
        vec![("class", "blocks-changelog-stacked-list-header")],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    weight: HeadingWeight::Bold,
                },
                vec![],
                vec![text("変更履歴")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text("直近のリリース内容をまとめています。")],
            ),
        ],
    );

    let mut rows: Vec<Node> = Vec::new();
    for (index, release) in RELEASES.iter().enumerate() {
        rows.push(stacked_entry(release, index == 0));
        if index + 1 < RELEASES.len() {
            rows.push(separator(
                &SeparatorProps::default(),
                vec![("data-blocks-changelog-stacked-list-separator", "")],
            ));
        }
    }

    let stacked_example = div(
        vec![("class", "blocks-changelog-stacked-list-example")],
        vec![
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text("罫線区切り")],
            ),
            div(vec![("class", "blocks-changelog-stacked-list-rows")], rows),
        ],
    );

    let card_example = div(
        vec![("class", "blocks-changelog-stacked-list-example")],
        vec![
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text("カード（全面リンク）")],
            ),
            div(
                vec![("class", "blocks-changelog-stacked-list-cards")],
                RELEASES.iter().map(card_entry).collect(),
            ),
        ],
    );

    div(
        vec![("class", "blocks-changelog-stacked-list-layout")],
        vec![header, stacked_example, card_example],
    )
}
```

## 原案差分メモ

参照（対応表 ID R0045〔主〕・R0044〔副〕。出典の固有名・ファイル名は
記載しません）からの意図的な差分は次のとおりです。

- 見出しレベルを `h2`/`h3` から `h3`/`h4` へ 1 段下げました（ページ側が
  `## Demo` として `h2` を出すため）。
- 罫線区切り版とカード版を同一データから並べて示す構成にしました（参照は
  区切り方ごとに別々の changelog でしたが、使用部品の一致検証を満たしつつ
  違いを比較できるようにするための独自の判断です）。
- カード版は変更点リストを省略し、「最新」badge は罫線区切り版の先頭
  エントリにのみ付与しました（R0044 との差分）。
- タグは badge で表現し、重要度による色分けは行わず全件同じ見た目に
  しました。
- `href="#"` の死リンクをリポジトリへの固定外部 URL へ置き換えました。
- カード版のアクセシブルな名前は `link-overlay` の `overlay` が持つ
  `aria-label` で付与し、`id` 属性・`aria-describedby` は出力しません
  （宙に浮いた ARIA 参照・id 重複を構造的に避けるため）。
- 配色・余白・角丸は独自実装せず、既存のテーマトークンにそのまま従います。
- 日付の機械可読値（`datetime`）と表示値が食い違わないようにしています。
- 文言（見出し・リリースタイトル・説明・変更点）はすべて独自に書き直し
  ました。
