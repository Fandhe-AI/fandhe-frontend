# changelog-timeline

`heading` / `text` / `badge` / `timeline` / `list` / `image` / `link` の 7 部品を合成した、タイムライン型の changelog レイアウトです。中央寄せの見出し・リード文の下に、リリースを縦のタイムラインで並べます。

- 静的表示です。docs サイトは無 JS のため、状態機械・フォーム・データ取得を持たない固定描画にしています。
- 文言・バージョン番号・日付・変更点はすべて架空のものです。
- データ取得・送信は行わず、`<form>` は使いません。
- 狭い幅（48rem 未満）では、日付・version 専用の左列を隠し、代わりに本文の中に同じ内容を表示します。
- Demo には 2 種類のインスタンスを並べています。上が日付+version 枠・connector・本文の 3 列構成（対応表 ID R0043、R0047 も同じ構造ですが JS で動くスクロール進捗バーは再現していません）、下が indicator 自体を日付入りのピルにした outline 表現（対応表 ID R0048）です。
- 集約元は 4 件です（対応表 ID R0043・R0047・R0048・R0041）。取り込んだのは領域配置・部品構成・状態の見せ方といった構造のみで、文言・配色・装飾・アイコン・内部識別子は取り込んでいません。

## Rust コード

```rust
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::image::{self, AspectRatio, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::list::{self, ListType, ListVariant};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};
use fandhe_frontend_pre_styled_ui::timeline::{self, TimelineVariant};
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

use crate::blocks::dummy_assets;

/// 外部リンク先（`href="#"` を使わないための固定 URL、`blog_list_image` と
/// 同じ判断）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// リリース 1 件分のダミーデータ（架空、実在の製品・企業とは無関係）。
struct Release {
    version: &'static str,
    date_iso: &'static str,
    date_label: &'static str,
    title: &'static str,
    tags: &'static [&'static str],
    image_src: Option<&'static str>,
    changes: &'static [&'static str],
}

/// リリース一覧（架空、4 件。画像は一部エントリのみに付け「任意の画像
/// スロット」であることを示す）。
const RELEASES: [Release; 4] = [
    Release {
        version: "v3.1.0",
        date_iso: "2026-09-20",
        date_label: "2026年9月20日",
        title: "タイムライン型 changelog を追加",
        tags: &["新機能"],
        image_src: Some(dummy_assets::SCREENSHOT_SRC),
        changes: &[
            "リリースを縦のタイムラインで並べる changelog レイアウトを追加",
            "日付入りピル表現の代替表示に対応",
        ],
    },
    Release {
        version: "v3.0.2",
        date_iso: "2026-09-12",
        date_label: "2026年9月12日",
        title: "狭い幅での表示崩れを修正",
        tags: &["修正"],
        image_src: None,
        changes: &["狭い幅で日付・version が本文と重なる表示崩れを修正"],
    },
    Release {
        version: "v3.0.1",
        date_iso: "2026-09-05",
        date_label: "2026年9月5日",
        title: "ダミー素材ヘルパを共通化",
        tags: &["改善", "内部"],
        image_src: Some(dummy_assets::PRODUCT_SRC),
        changes: &[
            "プレースホルダー画像・文言の生成をヘルパへ一元化",
            "block ごとの個別実装によるブレを解消",
        ],
    },
    Release {
        version: "v3.0.0",
        date_iso: "2026-08-28",
        date_label: "2026年8月28日",
        title: "changelog レイアウトを刷新",
        tags: &["新機能", "破壊的変更"],
        image_src: None,
        changes: &[
            "リリース単位のレイアウトを全面刷新",
            "変更点の種別タグ表示に対応",
        ],
    },
];

/// `<time datetime>` を組み立てる（機械可読な ISO 値と表示値は常に同じ日を
/// 指す組にする不変条件、`changelog_accordion`/`blog_list_image` と同じ
/// 判断）。
fn release_date(iso: &str, label: &str) -> Node {
    el(
        "time",
        vec![
            ("class", "blocks-changelog-timeline-date"),
            ("datetime", iso),
        ],
        vec![text(label)],
    )
}

/// 日付 + version の枠（boxed インスタンスの左列、および狭幅時に本文内へ
/// 表示する複製の共通実体）。
fn release_meta(release: &Release) -> Node {
    div(
        vec![("class", "blocks-changelog-timeline-meta")],
        vec![
            fandhe_frontend_core::span(
                vec![("class", "blocks-changelog-timeline-version")],
                vec![text(release.version)],
            ),
            release_date(release.date_iso, release.date_label),
        ],
    )
}

/// タグ badge 群。
fn release_tags(release: &Release) -> Node {
    div(
        vec![("class", "blocks-changelog-timeline-tags")],
        release
            .tags
            .iter()
            .map(|tag| {
                badge::badge(
                    &BadgeProps::default(),
                    vec![("data-blocks-changelog-timeline-tag", "")],
                    vec![text(*tag)],
                )
            })
            .collect(),
    )
}

/// 変更点リスト。
fn release_changes(release: &Release) -> Node {
    list::root(
        ListType::Unordered,
        ListVariant::Marker,
        vec![("data-blocks-changelog-timeline-changes", "")],
        release
            .changes
            .iter()
            .map(|change| list::item(vec![], vec![text(*change)]))
            .collect(),
    )
}

/// boxed インスタンス 1 リリース分の `timeline::item`。3 列（左列は日付
/// と version、中央は connector、右列は本文）。[`RELEASES`] は新しい順
/// （先頭が最新）のため、`is_first` は最新エントリの indicator を
/// `"current"` にする判定に、`is_last` は最終（最古）エントリで
/// `separator` を省く判定に使う（モジュール doc「separator を最後の
/// エントリで省く理由」節）。
fn boxed_item(release: &Release, is_first: bool, is_last: bool) -> Node {
    let side = timeline::content(
        vec![("data-blocks-changelog-timeline-side", "")],
        vec![release_meta(release)],
    );

    let mut connector_children = vec![timeline::indicator(
        vec![("data-state", if is_first { "current" } else { "complete" })],
        vec![],
    )];
    if !is_last {
        connector_children.push(timeline::separator(
            vec![("data-state", "complete")],
            vec![],
        ));
    }
    let connector = timeline::connector(vec![], connector_children);

    let mut body_children = vec![
        div(
            vec![("data-blocks-changelog-timeline-inline-meta", "")],
            vec![release_meta(release)],
        ),
        heading(
            HeadingLevel::H4,
            &HeadingProps {
                size: HeadingSize::Lg,
                weight: HeadingWeight::Semibold,
            },
            vec![],
            vec![text(release.title)],
        ),
        release_tags(release),
    ];
    if let Some(src) = release.image_src {
        body_children.push(div(
            vec![("class", "blocks-changelog-timeline-figure")],
            vec![image::image(
                &ImageProps {
                    aspect_ratio: AspectRatio::Video,
                    shape: ImageShape::Rounded,
                    ..ImageProps::new(src, "")
                },
                vec![("data-blocks-changelog-timeline-image", "")],
            )],
        ));
    }
    body_children.push(release_changes(release));
    body_children.push(link::root(
        REPO,
        &LinkProps {
            external: true,
            ..LinkProps::default()
        },
        vec![],
        vec![text("リリースノートを見る")],
    ));
    let body = timeline::content(
        vec![("data-blocks-changelog-timeline-body", "")],
        body_children,
    );

    timeline::item(vec![], vec![side, connector, body])
}

/// pill インスタンス 1 リリース分の `timeline::item`。2 列（indicator |
/// 本文）で、indicator 自体を日付入りのピルにする（R0048）。左列（日付 +
/// version 専用枠）は持たない。`is_first`/`is_last` の意味は [`boxed_item`]
/// と同じ（[`RELEASES`] は新しい順）。
fn pill_item(release: &Release, is_first: bool, is_last: bool) -> Node {
    let mut connector_children = vec![timeline::indicator(
        vec![("data-state", if is_first { "current" } else { "complete" })],
        vec![release_date(release.date_iso, release.date_label)],
    )];
    if !is_last {
        connector_children.push(timeline::separator(
            vec![("data-state", "complete")],
            vec![],
        ));
    }
    let connector = timeline::connector(vec![], connector_children);

    let body = timeline::content(
        vec![("data-blocks-changelog-timeline-body", "")],
        vec![
            fandhe_frontend_core::span(
                vec![("class", "blocks-changelog-timeline-version")],
                vec![text(release.version)],
            ),
            heading(
                HeadingLevel::H4,
                &HeadingProps {
                    size: HeadingSize::Lg,
                    weight: HeadingWeight::Semibold,
                },
                vec![],
                vec![text(release.title)],
            ),
            release_tags(release),
            release_changes(release),
        ],
    );

    timeline::item(vec![], vec![connector, body])
}

/// `changelog-timeline` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数（モジュール doc「静的表示」節）。boxed（R0043/R0047）・pill
/// （R0048）の 2 インスタンスを縦に並べる。
pub fn demo() -> Node {
    let header = div(
        vec![("class", "blocks-changelog-timeline-header")],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    weight: HeadingWeight::Bold,
                },
                vec![],
                vec![text("更新履歴")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text("各リリースの変更点を時系列で確認できます。")],
            ),
        ],
    );

    let last_index = RELEASES.len().saturating_sub(1);

    let boxed_items: Vec<Node> = RELEASES
        .iter()
        .enumerate()
        .map(|(index, release)| boxed_item(release, index == 0, index == last_index))
        .collect();
    let boxed = timeline::root(
        TimelineVariant::Solid,
        Size::Sm,
        ColorPalette::default(),
        vec![("data-blocks-changelog-timeline-variant", "boxed")],
        boxed_items,
    );

    let pill_items: Vec<Node> = RELEASES
        .iter()
        .enumerate()
        .map(|(index, release)| pill_item(release, index == 0, index == last_index))
        .collect();
    let pill = timeline::root(
        TimelineVariant::Outline,
        Size::Sm,
        ColorPalette::default(),
        vec![("data-blocks-changelog-timeline-variant", "pill")],
        pill_items,
    );

    div(
        vec![("class", "blocks-changelog-timeline-layout")],
        vec![header, boxed, pill],
    )
}
```

## 原案差分メモ

- R0041 の 2 列構成（version 列 + 変更一覧）は Demo に並べていません。boxed インスタンスの狭幅表示（左列を隠し version + 変更一覧が主体の 2 列へ切り替わる）と pill インスタンスの組み合わせで、この構造を示しています。
- R0047 が持つ JS 駆動のスクロール進捗バーは、無 JS の docs サイトでは再現していません。
- 見出しレベルを 1 段下げました（ページ側の `## Demo` に合わせるため、セクション見出しは `h3`、各リリースタイトルは `h4`）。
- リリースタイトルは `timeline::title`（`<span>`）ではなく `heading`（`<h4>`）を使いました。文書構造上の見出しとして扱いたいためです。
- アイコンは使わず、装飾は既存部品のみで構成しています。
- 画像はモノトーンの共通ダミー素材にし、一部のリリースのみに付けて任意スロットであることを示しました。
- 配色・余白・角丸は既存のテーマトークンに従っています。
- 日付には機械可読な `datetime` 属性を付けています。
- 文言・バージョン番号・変更点はすべて独自に書きました。
