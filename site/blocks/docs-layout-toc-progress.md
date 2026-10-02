# docs-layout-toc-progress

ページ内目次（番号付き進捗トラック）。`link`/`text` の 2 部品のみで
合成する例で、主参照は対応表 ID R0367（集約元は同 1 件）。

先頭 10 項目に番号を付けて縦に並べ、左の縦トラックで現在節までの読み
進み位置を塗ります。目次自体は可視の見出しを持たず、`aria-label` で
アクセシブルネームのみを与えます。現在位置は Demo 内で固定した静的
表示で、scroll spy のような実行時更新は行いません（docs サイトは
無 JS）。`<form>` は出しません。文言はすべて独自に書いた架空のもので
あり、実企業名・実クレデンシャル・PII を含みません。

## Rust コード

```rust
use fandhe_frontend_core::{div, li, nav, ol, span, text, Node};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::recipe::ColorPalette;
use fandhe_frontend_pre_styled_ui::text::{
    self as styled_text, TextProps, TextVariant, TextWeight,
};

/// 本文の 1 節（目次項目と 1 対 1 対応する）。
struct Section {
    id: &'static str,
    number: &'static str,
    title: &'static str,
}

/// 本文の全 12 節（架空の目次項目）。先頭 [`TOC_LIMIT`] 件のみを目次に出す。
const SECTIONS: &[Section] = &[
    Section {
        id: "blocks-docs-layout-toc-progress-overview",
        number: "01",
        title: "概要",
    },
    Section {
        id: "blocks-docs-layout-toc-progress-install",
        number: "02",
        title: "インストール",
    },
    Section {
        id: "blocks-docs-layout-toc-progress-setup",
        number: "03",
        title: "初期設定",
    },
    Section {
        id: "blocks-docs-layout-toc-progress-routing",
        number: "04",
        title: "ルーティング",
    },
    Section {
        id: "blocks-docs-layout-toc-progress-state",
        number: "05",
        title: "状態管理",
    },
    Section {
        id: "blocks-docs-layout-toc-progress-styling",
        number: "06",
        title: "スタイル",
    },
    Section {
        id: "blocks-docs-layout-toc-progress-testing",
        number: "07",
        title: "テスト",
    },
    Section {
        id: "blocks-docs-layout-toc-progress-build",
        number: "08",
        title: "ビルド",
    },
    Section {
        id: "blocks-docs-layout-toc-progress-deploy",
        number: "09",
        title: "デプロイ",
    },
    Section {
        id: "blocks-docs-layout-toc-progress-operation",
        number: "10",
        title: "運用",
    },
    Section {
        id: "blocks-docs-layout-toc-progress-troubleshooting",
        number: "11",
        title: "トラブルシューティング",
    },
    Section {
        id: "blocks-docs-layout-toc-progress-appendix",
        number: "12",
        title: "付録",
    },
];

/// 目次に表示する先頭項目数（11・12 項目目は目次から打ち切られる）。
const TOC_LIMIT: usize = 10;

/// 現在位置として固定するインデックス（0 始まり、4 項目目）。
const CURRENT_INDEX: usize = 3;

/// 目次項目 1 件の進捗状態。
#[derive(Clone, Copy, PartialEq, Eq)]
enum ProgressState {
    Done,
    Current,
    Upcoming,
}

impl ProgressState {
    fn as_attr(self) -> &'static str {
        match self {
            ProgressState::Done => "done",
            ProgressState::Current => "current",
            ProgressState::Upcoming => "upcoming",
        }
    }
}

/// 目次（`nav` > `ol` > `li`）を組み立てる。マーカー `01`〜`10` は
/// `aria-hidden="true"` にし、`ol` の暗黙の順序と重複読み上げしない。
fn toc() -> Node {
    let items = SECTIONS
        .iter()
        .take(TOC_LIMIT)
        .enumerate()
        .map(|(index, section)| {
            let state = if index < CURRENT_INDEX {
                ProgressState::Done
            } else if index == CURRENT_INDEX {
                ProgressState::Current
            } else {
                ProgressState::Upcoming
            };
            let href = format!("#{}", section.id);
            let mut link_attrs: Vec<(&str, &str)> = vec![];
            if state == ProgressState::Current {
                link_attrs.push(("aria-current", "location"));
            }
            li(
                vec![
                    ("data-blocks-docs-layout-toc-progress-item", ""),
                    (
                        "data-blocks-docs-layout-toc-progress-item-state",
                        state.as_attr(),
                    ),
                ],
                vec![
                    span(
                        vec![
                            ("aria-hidden", "true"),
                            ("data-blocks-docs-layout-toc-progress-marker", ""),
                        ],
                        vec![text(section.number)],
                    ),
                    link::root(
                        &href,
                        &LinkProps {
                            palette: ColorPalette::Neutral,
                            ..LinkProps::default()
                        },
                        link_attrs,
                        vec![text(section.title)],
                    ),
                ],
            )
        })
        .collect::<Vec<_>>();
    nav(
        vec![
            ("aria-label", "このページの目次"),
            ("data-blocks-docs-layout-toc-progress-nav", ""),
        ],
        vec![ol(
            vec![("data-blocks-docs-layout-toc-progress-list", "")],
            items,
        )],
    )
}

/// 本文プレースホルダー列（全 [`SECTIONS`]。各節見出しに目次遷移先の `id`
/// を付ける）。
fn body() -> Node {
    let sections = SECTIONS
        .iter()
        .map(|section| {
            div(
                vec![],
                vec![
                    styled_text::text(
                        &TextProps {
                            weight: TextWeight::Semibold,
                            ..TextProps::default()
                        },
                        vec![("id", section.id)],
                        vec![text(section.title)],
                    ),
                    styled_text::text(
                        &TextProps {
                            variant: TextVariant::Muted,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text(
                            "節の本文はダミーのプレースホルダーです。実際のページでは解説が入ります。",
                        )],
                    ),
                ],
            )
        })
        .collect::<Vec<_>>();
    div(
        vec![("class", "blocks-docs-layout-toc-progress-body")],
        sections,
    )
}

/// `docs-layout-toc-progress` の Demo 本体（本文列 + 目次列。呼び出しごとに
/// 同一の `Node` を返す純関数）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-docs-layout-toc-progress-layout")],
        vec![body(), toc()],
    )
}
```

## 原案差分メモ

- 実アプリでは目次列に `position: sticky` を付け、scroll spy で
  `data-*-item-state` と `aria-current` を実行時に更新する想定です。
- 11 項目目以降は目次に出しません（先頭 10 項目のみの打ち切り表示）。
- 現在項目のトラックはマーカー中心までを塗って表現しています。
