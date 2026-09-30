# list-title-meta

`list` / `badge` / `status` / `avatar` / `button` / `menu` / `link` /
`link-overlay` / `visually-hidden` の 9 部品を合成した、タイトル行＋メタ行
の 2 段構成を共通の骨格とするリストの合成例です。Blocks セクションは新規
部品を追加するものではなく、既存の Themes/Primitives 部品を組み合わせた
実例集であることに注意してください。

各行は 1 行目にタイトル（＋ステータス表示）、2 行目に日付・作成者などの
メタ情報を中黒区切りで並べます。右端には行ごとの付随要素を置きますが、
その内容だけが異なる 3 インスタンスを縦に並べて示します。

1. **表示ボタン + メニュー**: 行末に「表示」ボタンと三点メニュー（項目
   3 件 + 区切り線、閉状態固定）を置きます。狭いコンテナ幅では「表示」
   ボタンを隠し、メニュー内に同じ「表示」項目を残す構成ですが、本 Demo
   は無 JS の静的表示のためメニュー自体も `disabled` 固定です（実運用で
   `disabled` を外して配線する場合はこの限りではありません）。
2. **重ねたアバター + 件数**: 行末に重ねた小さなアバター 3 件と「+N」の
   残り人数表示を置きます。
3. **状態ドット + 環境バッジ**: 行全体をリンク化し、タイトルの隣に状態
   ドット、行末に環境バッジと矢印（装飾）を置きます。

いずれも静的な表示例であり、`<form>` 要素を持ちません。タイトル・日付・
環境名は独自に書いた架空の文言です。作成者名は架空の人名セットから
引用します。`menu`・行末ボタンは JS ハイドレーションを行わない本 docs
サイト向けに `disabled` 固定の静的表示です。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, span, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps, BadgeVariant};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::link_overlay;
use fandhe_frontend_pre_styled_ui::list::{self, ListType, ListVariant};
use fandhe_frontend_pre_styled_ui::menu::{self, OpenState};
use fandhe_frontend_pre_styled_ui::status::{self, StatusProps};
use fandhe_frontend_pre_styled_ui::visually_hidden;
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// 外部の実在 URL（`href="#"` は使わない、他 block と同型の判断）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// R1287（主参照）: 「表示」ボタン + 三点メニュー構成の 1 行分データ。
struct ActionRow {
    title: &'static str,
    status_label: &'static str,
    status_palette: ColorPalette,
    date_iso: &'static str,
    date_label: &'static str,
    author_index: usize,
    /// `menu::trigger`/`menu::content` を紐づける一意 id（行固定の定数、
    /// モジュール doc「id をページ内で一意にする理由」節）。
    menu_id: &'static str,
    menu_trigger_id: &'static str,
}

const ACTION_ROWS: [ActionRow; 4] = [
    ActionRow {
        title: "認証まわりのリファクタリング",
        status_label: "完了",
        status_palette: ColorPalette::Success,
        date_iso: "2026-09-20T10:00:00+09:00",
        date_label: "9 月 20 日",
        author_index: 0,
        menu_id: "blocks-list-title-meta-menu-0",
        menu_trigger_id: "blocks-list-title-meta-menu-trigger-0",
    },
    ActionRow {
        title: "検索インデックスの再構築",
        status_label: "進行中",
        status_palette: ColorPalette::Accent,
        date_iso: "2026-09-24T15:30:00+09:00",
        date_label: "9 月 24 日",
        author_index: 1,
        menu_id: "blocks-list-title-meta-menu-1",
        menu_trigger_id: "blocks-list-title-meta-menu-trigger-1",
    },
    ActionRow {
        title: "旧ダッシュボードの棚卸し",
        status_label: "アーカイブ",
        status_palette: ColorPalette::Neutral,
        date_iso: "2026-09-10T09:00:00+09:00",
        date_label: "9 月 10 日",
        author_index: 2,
        menu_id: "blocks-list-title-meta-menu-2",
        menu_trigger_id: "blocks-list-title-meta-menu-trigger-2",
    },
    ActionRow {
        title: "通知バッチのタイムアウト調査",
        status_label: "進行中",
        status_palette: ColorPalette::Accent,
        date_iso: "2026-09-26T18:15:00+09:00",
        date_label: "9 月 26 日",
        author_index: 3,
        menu_id: "blocks-list-title-meta-menu-3",
        menu_trigger_id: "blocks-list-title-meta-menu-trigger-3",
    },
];

/// R1285: 重ねたアバター群 + 件数構成の 1 行分データ。
struct AvatarRow {
    title: &'static str,
    status_label: &'static str,
    status_palette: ColorPalette,
    date_iso: &'static str,
    date_label: &'static str,
    author_index: usize,
    /// 重ね表示するアバター 3 件分の [`dummy_assets::PERSON_NAMES`] 添字。
    avatar_indices: [usize; 3],
    /// 重ね表示から溢れた残り人数（「+N」表示、[`visually_hidden`] で
    /// 「他 N 名」を補う）。
    extra_count: u32,
}

const AVATAR_ROWS: [AvatarRow; 3] = [
    AvatarRow {
        title: "デザインレビュー会",
        status_label: "完了",
        status_palette: ColorPalette::Success,
        date_iso: "2026-09-22T14:00:00+09:00",
        date_label: "9 月 22 日",
        author_index: 4,
        avatar_indices: [0, 1, 2],
        extra_count: 2,
    },
    AvatarRow {
        title: "リリース判定ミーティング",
        status_label: "進行中",
        status_palette: ColorPalette::Accent,
        date_iso: "2026-09-25T11:00:00+09:00",
        date_label: "9 月 25 日",
        author_index: 5,
        avatar_indices: [1, 3, 5],
        extra_count: 4,
    },
    AvatarRow {
        title: "四半期振り返り",
        status_label: "アーカイブ",
        status_palette: ColorPalette::Neutral,
        date_iso: "2026-09-05T16:00:00+09:00",
        date_label: "9 月 5 日",
        author_index: 6,
        avatar_indices: [2, 4, 6],
        extra_count: 1,
    },
];

/// R1297: 状態ドット + 環境バッジ + 矢印構成の 1 行分データ。行全体を
/// [`link_overlay`] でリンク化するため、対話要素（`menu`/`button`）は
/// 置かない（モジュール doc「3 種の右端要素を 1 行に同居させない理由」
/// 節）。
struct DeployRow {
    title: &'static str,
    status_label: &'static str,
    status_palette: ColorPalette,
    env_label: &'static str,
    date_iso: &'static str,
    date_label: &'static str,
    author_index: usize,
}

const DEPLOY_ROWS: [DeployRow; 3] = [
    DeployRow {
        title: "web-frontend",
        status_label: "稼働中",
        status_palette: ColorPalette::Success,
        env_label: "本番",
        date_iso: "2026-09-27T08:00:00+09:00",
        date_label: "9 月 27 日",
        author_index: 0,
        // ↑ 環境名・サービス名は独自の架空文言（実在サービスとは無関係）。
    },
    DeployRow {
        title: "api-gateway",
        status_label: "稼働中",
        status_palette: ColorPalette::Success,
        env_label: "プレビュー",
        date_iso: "2026-09-27T09:30:00+09:00",
        date_label: "9 月 27 日",
        author_index: 1,
    },
    DeployRow {
        title: "batch-worker",
        status_label: "停止中",
        status_palette: ColorPalette::Neutral,
        env_label: "プレビュー",
        date_iso: "2026-09-24T21:00:00+09:00",
        date_label: "9 月 24 日",
        author_index: 2,
    },
];

/// メタ行（日付 + 作成者、中黒区切り）。`<time datetime>` の `iso`/`label`
/// は常に同じ日時を指す組にする不変条件（他 block と同型）。中黒（`\u{b7}`）
/// 自体は装飾として `aria-hidden` で読み上げから除外するが、支援技術に
/// 日付と作成者を区切る文字が一切渡らなくなるのを避けるため、
/// [`visually_hidden::root`] で読み上げ専用の区切りテキストを別途補う
/// （codex レビュー P2 指摘、discussion_r4118300487）。
fn meta_line(iso: &str, label: &str, author: &str) -> Node {
    el(
        "p",
        vec![("class", "blocks-list-title-meta-meta")],
        vec![
            el("time", vec![("datetime", iso)], vec![text(label)]),
            span(vec![("aria-hidden", "true")], vec![text(" \u{b7} ")]),
            visually_hidden::root(vec![], vec![text(", ")]),
            text(author),
        ],
    )
}

/// 本体（タイトル行 + メタ行）を組み立てる。
fn body(title_row: Node, meta: Node) -> Node {
    div(
        vec![("class", "blocks-list-title-meta-body")],
        vec![title_row, meta],
    )
}

/// タイトル行（タイトルノード + ステータス表示）を組み立てる。
fn title_row(title_node: Node, status_node: Node) -> Node {
    div(
        vec![("class", "blocks-list-title-meta-title-row")],
        vec![title_node, status_node],
    )
}

/// ステータス [`badge`] を組み立てる（R1287/R1285 の行で使用）。
fn status_badge(label: &str, palette: ColorPalette) -> Node {
    badge::badge(
        &BadgeProps {
            variant: BadgeVariant::Subtle,
            size: Size::Sm,
            palette,
            shape: None,
        },
        vec![],
        vec![text(label)],
    )
}

/// R1287 行末: 「表示」ボタン + 三点メニュー。ボタンは狭幅で隠れる
/// （[`LAYOUT_CSS`] の `@container` 参照）が、menu content に同じ「表示」
/// 項目があるため機能は失われない。
fn action_trailing(row: &ActionRow) -> Node {
    let view_button = button::button(
        &ButtonProps {
            variant: ButtonVariant::Outline,
            size: Size::Sm,
            disabled: true,
            ..ButtonProps::default()
        },
        vec![("data-blocks-list-title-meta-view-button", "")],
        vec![
            text("表示"),
            visually_hidden::root(vec![], vec![text(format!("、{}", row.title))]),
        ],
    );
    let menu_root = menu::root(
        Size::Sm,
        OpenState::Closed,
        vec![],
        vec![
            menu::trigger(
                OpenState::Closed,
                true,
                Some(row.menu_id),
                vec![("id", row.menu_trigger_id)],
                vec![
                    text("\u{2026}"),
                    visually_hidden::root(
                        vec![],
                        vec![text(format!("その他の操作、{}", row.title))],
                    ),
                ],
            ),
            menu::positioner(
                OpenState::Closed,
                vec![],
                vec![menu::content(
                    OpenState::Closed,
                    Some(row.menu_id),
                    Some(row.menu_trigger_id),
                    vec![],
                    vec![
                        menu::item("view", false, false, vec![], vec![text("表示")]),
                        menu::item("edit", false, false, vec![], vec![text("編集")]),
                        menu::separator(vec![], vec![]),
                        menu::item("archive", false, false, vec![], vec![text("アーカイブ")]),
                    ],
                )],
            ),
        ],
    );
    div(
        vec![("class", "blocks-list-title-meta-trailing")],
        vec![view_button, menu_root],
    )
}

/// R1287 行の `list::item`。
fn action_item(row: &ActionRow) -> Node {
    list::item(
        vec![("class", "blocks-list-title-meta-row")],
        vec![
            body(
                title_row(
                    link::root(REPO, &LinkProps::default(), vec![], vec![text(row.title)]),
                    status_badge(row.status_label, row.status_palette),
                ),
                meta_line(
                    row.date_iso,
                    row.date_label,
                    dummy_assets::PERSON_NAMES[row.author_index],
                ),
            ),
            action_trailing(row),
        ],
    )
}

/// 氏名の先頭 1 文字をアバターのフォールバック表示に使う（`list_people`
/// と同型のヘルパ）。フォールバックの表示文字は先頭 1 文字のみで
/// 支援技術に氏名が伝わらないため、`root` へ `aria-label` で氏名全体を
/// 供給する（`avatar::badge` doc「アクセシブルネームが必要な場合は
/// 呼び出し側が `root` の `aria-label` 等で供給する」節と同型の判断、
/// codex レビュー指摘）。`avatar::root` が出力する `<div>` は役割を持たず
/// `aria-label` だけでは支援技術がアクセシブルネームとして採用しない
/// （role なし要素の name computation の対象外になり得るため）ため、
/// `role="img"` を明示付与して氏名全体をアクセシブルネームとして確実に
/// 伝える（codex/Bugbot レビュー指摘、イシュー #2925）。
fn initial_avatar(name: &str) -> Node {
    let initial: String = name.chars().take(1).collect();
    avatar::root(
        &AvatarProps {
            size: Size::Xs,
            stacked: true,
            ..AvatarProps::default()
        },
        vec![("role", "img"), ("aria-label", name)],
        vec![avatar::fallback(
            ImageStatus::default(),
            vec![],
            vec![text(initial)],
        )],
    )
}

/// R1285 行末: 重ねたアバター群 + 件数。
fn avatars_trailing(row: &AvatarRow) -> Node {
    let avatars: Vec<Node> = row
        .avatar_indices
        .iter()
        .map(|&i| initial_avatar(dummy_assets::PERSON_NAMES[i]))
        .collect();
    div(
        vec![("class", "blocks-list-title-meta-trailing")],
        vec![
            avatar::group(vec![], avatars),
            span(
                vec![("class", "blocks-list-title-meta-avatar-count")],
                vec![
                    span(
                        vec![("aria-hidden", "true")],
                        vec![text(format!("+{}", row.extra_count))],
                    ),
                    visually_hidden::root(
                        vec![],
                        vec![text(format!("、他 {} 名", row.extra_count))],
                    ),
                ],
            ),
        ],
    )
}

/// R1285 行の `list::item`。
fn avatar_item(row: &AvatarRow) -> Node {
    list::item(
        vec![("class", "blocks-list-title-meta-row")],
        vec![
            body(
                title_row(
                    link::root(REPO, &LinkProps::default(), vec![], vec![text(row.title)]),
                    status_badge(row.status_label, row.status_palette),
                ),
                meta_line(
                    row.date_iso,
                    row.date_label,
                    dummy_assets::PERSON_NAMES[row.author_index],
                ),
            ),
            avatars_trailing(row),
        ],
    )
}

/// R1297 行の状態表示（状態ドット + ラベル。色だけで状態を伝えないよう
/// [`visually_hidden`] でラベルを補う）。
fn deploy_status(row: &DeployRow) -> Node {
    status::root(
        &StatusProps {
            size: Size::Sm,
            palette: row.status_palette,
        },
        vec![],
        vec![status::indicator(vec![]), text(row.status_label)],
    )
}

/// R1297 行末: 環境バッジ + 矢印（装飾、`aria-hidden`）。
fn deploy_trailing(row: &DeployRow) -> Node {
    div(
        vec![("class", "blocks-list-title-meta-trailing")],
        vec![
            badge::badge(
                &BadgeProps {
                    variant: BadgeVariant::Outline,
                    size: Size::Sm,
                    palette: ColorPalette::Neutral,
                    shape: None,
                },
                vec![],
                vec![text(row.env_label)],
            ),
            span(
                vec![
                    ("class", "blocks-list-title-meta-arrow"),
                    ("aria-hidden", "true"),
                ],
                vec![text("\u{2192}")],
            ),
        ],
    )
}

/// R1297 行の `list::item`。行全体を [`link_overlay`] でリンク化するため
/// タイトルは素のテキストのまま使う（インラインリンクと行全体リンクを
/// 同居させない、モジュール doc「3 種の右端要素を 1 行に同居させない
/// 理由」節）。`list::item` 自体には `blocks-list-title-meta-row` を
/// 付与しない（grid レイアウト・`padding-block` は内側の
/// `link_overlay::root` 側にのみ属性セレクタで適用する契約。両方へ
/// 付与すると `padding-block` が二重適用され行の高さが 2 倍になる、
/// [`LAYOUT_CSS`] doc「行の骨格をどこに置くか」節参照）。
fn deploy_item(row: &DeployRow) -> Node {
    list::item(
        vec![],
        vec![link_overlay::root(
            vec![],
            vec![
                body(
                    title_row(
                        div(
                            vec![("class", "blocks-list-title-meta-title-text")],
                            vec![text(row.title)],
                        ),
                        deploy_status(row),
                    ),
                    meta_line(
                        row.date_iso,
                        row.date_label,
                        dummy_assets::PERSON_NAMES[row.author_index],
                    ),
                ),
                deploy_trailing(row),
                link_overlay::overlay(REPO, vec![("aria-label", row.title)], vec![]),
            ],
        )],
    )
}

/// 3 インスタンス共通のパネル外枠（`@container` の名前付きコンテナ、
/// イシュー本文の「狭い幅では『表示』ボタンを隠す」要件をここで宣言
/// する）。
fn panel(variant: &str, aria_label: &str, rows: Vec<Node>) -> Node {
    div(
        vec![
            ("class", "blocks-list-title-meta-panel"),
            ("data-blocks-list-title-meta-variant", variant),
        ],
        vec![list::root(
            ListType::Unordered,
            ListVariant::Plain,
            vec![("aria-label", aria_label)],
            rows,
        )],
    )
}

fn actions_instance() -> Node {
    panel(
        "actions",
        "最近のタスク",
        ACTION_ROWS.iter().map(action_item).collect(),
    )
}

fn avatars_instance() -> Node {
    panel(
        "avatars",
        "最近の会議",
        AVATAR_ROWS.iter().map(avatar_item).collect(),
    )
}

fn deploy_instance() -> Node {
    panel(
        "deploy",
        "デプロイ状況",
        DEPLOY_ROWS.iter().map(deploy_item).collect(),
    )
}

/// 見出し付きセクションを組み立てる（`list_people` と同型のヘルパ）。
fn section(label: &'static str, node: Node) -> Node {
    div(
        vec![("class", "blocks-list-title-meta-section")],
        vec![
            el(
                "h3",
                vec![("class", "blocks-list-title-meta-section-title")],
                vec![text(label)],
            ),
            node,
        ],
    )
}

/// `list-title-meta` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数。主参照（R1287）を先頭に、3 インスタンスをラベル付き見出しで
/// 区切って並べる。
#[must_use]
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-list-title-meta-layout")],
        vec![
            section("表示ボタン + メニュー", actions_instance()),
            section("重ねたアバター + 件数", avatars_instance()),
            section("状態ドット + 環境バッジ", deploy_instance()),
        ],
    )
}
```

## 原案差分メモ

- 主参照は R1287（表示ボタン + 三点メニュー）です。R1285（重ねたアバター +
  件数）・R1297（状態ドット + 環境バッジ + 矢印）を別インスタンスとして
  併記し、1 行への混在は行っていません（`link-overlay` と `menu`/`button`
  は同じ行に同居できないため）。
- 実データ取得・メニュー開閉・ボタン押下は行わず、静的な初期状態のみを
  示します。タイトル・日付・環境名・作成者名はすべて独自の架空データです。
- ブラウザでの実機確認（コンテナ幅切替・ホバー面色・ライト/ダーク両テーマ）
  はサンドボックス制約により未実施です。cargo test による出力検証のみで
  代替しました。
