# docs-layout-toc-collapsible

`fandhe-frontend-pre-styled-ui` の `collapsible` / `link` / `button` /
`icon` 部品を合成した、狭い画面向けの開閉式ページ内目次の実例です。
Blocks セクションは新規部品を追加するものではなく、既存の
Themes/Primitives 部品を組み合わせた実例集であることに注意してください
（主参照は対応表 ID R0366。出典の固有名・ファイル名は記載しません）。

開閉トリガーには「目次」ラベルと現在の節名、開閉を示す下向き矢印
アイコンを並べ、開いた状態（例 A）と閉じた状態（例 B）を併記します。
節一覧は本文の見出しへのページ内リンクで、現在の節だけに
`aria-current="location"` を付けて強調しています。本 Demo は無 JS の
静的表示であり、初期状態をそれぞれ固定しています。

本 Demo は静的な表示例であり、`<form>` 要素は一切持たず、トリガーは
操作できない状態（`disabled`）に固定しています。本文末尾の
「ページの先頭へ」ボタンも `type="button"` のまま送信先を持たず、
トリガーと同じく `disabled` に固定しているため、クリック・キーボード
操作のいずれにも反応しません。文言はすべて独自に書いた架空のもの
であり、実企業名・実クレデンシャル・PII を含みません。

## Rust コード

```rust
use fandhe_frontend_core::{div, el, li, nav, p, span, text, ul, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::collapsible::{self, OpenState};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::Size;

/// 目次の節一覧（id スラッグ, ラベル）。単一の真実源。トリガーに出す
/// 現在の節名・`aria-current` を付けるリンク・本文見出し `id` は、
/// すべてここと [`CURRENT`] から導く。
const SECTIONS: &[(&str, &str)] = &[
    ("overview", "概要"),
    ("installation", "インストール"),
    ("configuration", "設定"),
    ("next-steps", "次のステップ"),
];

/// 現在の節（[`SECTIONS`] のインデックス）。
const CURRENT: usize = 1;

/// 本文見出し `id`（例 A のみに存在、両インスタンスの `href` が共有する）。
fn section_id(slug: &str) -> String {
    format!("blocks-docs-layout-toc-collapsible-sec-{slug}")
}

/// 下向き矢印アイコン（自作の単純な線分パス、実在アイコンセットは
/// 使わない。`action_panel_footer_bar::geo_icon` と同型）。
fn chevron_icon() -> Node {
    icon(
        &IconProps::default(),
        vec![],
        vec![el(
            "path",
            vec![
                ("d", "M6 9l6 6l6 -6"),
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

/// 上向き矢印アイコン（「ページの先頭へ」ボタン用）。
fn arrow_up_icon() -> Node {
    icon(
        &IconProps::default(),
        vec![],
        vec![el(
            "path",
            vec![
                ("d", "M12 19V5 M6 11l6 -6l6 6"),
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

/// 目次トリガー（ラベル「目次」+ 現在の節名 → indicator の 2 子構成）。
fn trigger(content_id: &str, state: OpenState) -> Node {
    let label = span(
        vec![("data-blocks-docs-layout-toc-collapsible-trigger-label", "")],
        vec![
            span(
                vec![(
                    "data-blocks-docs-layout-toc-collapsible-trigger-eyebrow",
                    "",
                )],
                vec![text("目次")],
            ),
            span(vec![], vec![text(SECTIONS[CURRENT].1)]),
        ],
    );
    collapsible::trigger(
        state,
        true,
        Some(content_id),
        vec![("data-blocks-docs-layout-toc-collapsible-trigger", "")],
        vec![
            label,
            collapsible::indicator(state, true, vec![], vec![chevron_icon()]),
        ],
    )
}

/// 目次の中身（節リンク一覧 + 「ページの先頭へ」ボタン）。
fn content(suffix: &str, content_id: &str, state: OpenState) -> Node {
    let items: Vec<Node> = SECTIONS
        .iter()
        .enumerate()
        .map(|(index, (slug, label))| {
            let href = format!("#{}", section_id(slug));
            let mut attrs: Vec<(&str, &str)> =
                vec![("data-blocks-docs-layout-toc-collapsible-link", "")];
            if index == CURRENT {
                attrs.push(("aria-current", "location"));
            }
            li(
                vec![],
                vec![link::root(
                    href.as_str(),
                    &LinkProps::default(),
                    attrs,
                    vec![text(*label)],
                )],
            )
        })
        .collect();

    let nav_label = format!("ページ内目次（例 {suffix}）");
    let toc_nav = nav(
        vec![("aria-label", nav_label.as_str())],
        vec![ul(vec![], items)],
    );

    // 「ページの先頭へ」は実際の送信先・スクロール処理を持たない静的な
    // 見本のため、トリガーと同じく `disabled: true` で固定し、押しても
    // キーボード操作でも反応しないことをネイティブ `disabled` 属性で
    // 構造的に保証する（[`trigger`] と同型の確定パターン）。既定の
    // `opacity: 0.5`/`cursor: not-allowed` は見本として読みにくくなるため
    // [`LAYOUT_CSS`] で打ち消す。
    let back_to_top = button::button(
        &ButtonProps {
            variant: ButtonVariant::Ghost,
            size: Size::Sm,
            disabled: true,
            ..ButtonProps::default()
        },
        vec![("data-blocks-docs-layout-toc-collapsible-back-to-top", "")],
        vec![arrow_up_icon(), text("ページの先頭へ")],
    );

    collapsible::content(
        state,
        false,
        Some(content_id),
        vec![("data-blocks-docs-layout-toc-collapsible-content", "")],
        vec![toc_nav, back_to_top],
    )
}

/// 目次インスタンス 1 件（`suffix` は `"a"`/`"b"` で `id` を分ける）。
fn toc(suffix: &'static str, state: OpenState) -> Node {
    let content_id = format!("blocks-docs-layout-toc-collapsible-{suffix}-content");
    collapsible::root(
        state,
        false,
        vec![("class", "blocks-docs-layout-toc-collapsible-instance")],
        vec![
            trigger(&content_id, state),
            content(suffix, &content_id, state),
        ],
    )
}

/// 本文（例 A のみに置く。目次リンクの参照先見出しを提供する）。
fn article_body() -> Node {
    let mut children: Vec<Node> = Vec::new();
    for (slug, label) in SECTIONS {
        let id = section_id(slug);
        children.push(heading(
            HeadingLevel::H3,
            &HeadingProps {
                size: HeadingSize::Md,
                weight: HeadingWeight::Semibold,
            },
            vec![("id", id.as_str())],
            vec![text(*label)],
        ));
        children.push(p(
            vec![],
            vec![text(
                "本文はプレースホルダーです。実際のページでは、ここに節ごとの \
                 解説が入ります。",
            )],
        ));
    }
    div(
        vec![("class", "blocks-docs-layout-toc-collapsible-body")],
        children,
    )
}

/// `docs-layout-toc-collapsible` の Demo 本体（例 A・B を縦に並べる。
/// 呼び出しごとに同一の `Node` を返す純関数）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-docs-layout-toc-collapsible-layout")],
        vec![
            toc("a", OpenState::Open),
            article_body(),
            toc("b", OpenState::Closed),
        ],
    )
}
```

## 原案差分メモ

- 主参照（対応表 ID R0366）のみを集約元とし、トリガーに「現在の節名」+
  「開閉を示す矢印アイコン」を並べる構成をそのまま踏襲しています。
- トリガー・「ページの先頭へ」ボタンはいずれも無 JS のため `disabled`
  に固定し、押しても状態は変わりません。既定の半透明表示
  （`opacity: 0.5`）は見本として読みにくいため CSS で打ち消しています
  （打ち消しセレクタは `data-scope`/`data-part` を含め、各 recipe の
  disabled 規則と同等以上の詳細度にしています）。
- 目次リンクの現在地強調には `link` の `current`（`aria-current="page"`）
  ではなく、`attrs` で直接渡した `aria-current="location"` を使って
  います。`current` はページ単位の現在地を表すため、ページ内の節には
  意味論上そぐわないと判断しました。
- 目次の開閉トリガーに `position: sticky` は付けていません。Demo 枠が
  横スクロールコンテナ（`overflow-x: auto`）になるため、Demo 内では
  意図どおりに効かないためです。実際のページでは目次トリガーや目次
  本体に `sticky` を付けられます。
- docs サイト本体のスクロールスパイが使う `class="docs-toc"` は、本
  block のクラス名と衝突しないよう使っていません。
- 実際のブラウザでの開閉操作・矢印の回転確認は本 Demo では行って
  いません（無 JS の静的表示のため）。

関連情報: [Collapsible](../themes/collapsible.md) / [Link](../themes/link.md) /
[Button](../themes/button.md) / [Icon](../themes/icon.md)
