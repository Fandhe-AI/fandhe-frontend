//! `list-title-meta` block（イシュー #2925。親トラッキング #2892「Blocks
//! 目的別パーツ拡充」配下）。タイトル行＋メタ情報行の 2 段構成を共通の
//! 行骨格とし、右端の付随要素だけを 3 種類に差し替えたリストを並べる
//! 合成例。対応表 ID R1287（主参照・「表示」ボタン + 三点メニュー構成）を
//! 軸に、R1285（重ねたアバター群 + 件数）/ R1297（状態ドット + 環境
//! バッジ + 矢印）を 3 インスタンスへ集約する。`_/blocks-intake/` の
//! 対応ファイルは本イシュー着手時点で本 worktree に存在しないため、原稿・
//! 本コメントには対応表 ID のみを記す（`card-form-footer`〔イシュー
//! #2899〕・`list-people`〔イシュー #2923〕と同じ扱い）。
//!
//! # 使用部品
//!
//! `list` / `badge` / `status` / `avatar` / `button` / `menu` / `link` /
//! `link-overlay` / `visually-hidden` の 9 部品を合成する（[`BLOCK`] の
//! `parts` に一致させる契約、`crates/docs-site/tests/blocks_nav.rs`/
//! `blocks_contract.rs` が検証する）。新しい UI 部品は追加しない。
//!
//! # 3 種の右端要素を 1 行に同居させない理由
//!
//! `link_overlay`（行全体リンク、R1297 の行で使用）と `menu`/`button`
//! （対話要素、R1287/R1285 の行で使用）は同じ行へ同居できない。`<a>` の
//! 内側に対話要素を入れ子にすると HTML の interactive content 制約に反し、
//! かつ `link_overlay::overlay` は `position: absolute; inset: 0;` で
//! 行全体を覆うため、他の対話要素をクリック不能にしてしまう
//! （`list_people`〔イシュー #2923〕の同型判断を踏襲）。そのため本 Demo は
//! 3 種を 1 行へ混ぜず、同じ行骨格（タイトル行 + メタ行）で右端だけを
//! 差し替えた 3 インスタンスとして縦に並べる。
//!
//! # `menu`/ボタンを disabled に固定する理由
//!
//! 本 Demo は無 JS の docs サイトで静的な初期状態のみを示す（JS
//! ハイドレーションを行わない）。押しても何も起きない要素を操作可能に
//! 見せないため、`menu::trigger` の `disabled: true`（第 2 引数）・行末
//! 「表示」ボタンの `ButtonProps { disabled: true, .. }` を固定する。
//! `[data-disabled]` の既定 `opacity: 0.5; cursor: not-allowed;` は
//! 中和しない（無効な操作を有効に見せてはならないため。以前の版は
//! `opacity: 1; cursor: default;` で打ち消していたが、これは「押しても
//! 何も起きない要素を操作可能に見せない」という本節冒頭の意図と矛盾する
//! codex レビュー指摘であり削除した）。「表示」ボタンが隠れる狭幅の
//! コンテナでも `menu` content に同じ「表示」項目があるため見た目の
//! 到達手段は失われないが、`menu::trigger` 自体も本 Demo では
//! `disabled: true` の静的表示であり実際には操作できない（本 Demo が
//! 無 JS の静的表示に限定されるための一般的な制約であり、実運用で
//! `disabled` を外して配線する場合はこの限りではない旨を
//! `site/blocks/list-title-meta.md` に明記する）。
//!
//! # `menu`/ボタンの id をページ内で一意にする理由
//!
//! `demo_output_has_no_dangling_aria_references_or_duplicate_ids`
//! （`crates/docs-site/tests/blocks_contract.rs`）が id 重複を fail-closed に
//! 検知するため、`menu::trigger`/`menu::content` の id は行ごとに固定の
//! `&'static str` 定数として持つ（`format!` で作る `String` の寿命問題を
//! 避けるため、`list_people` と異なり配列の添字ではなく行データへ直接 id
//! 文字列を持たせる）。行末ボタン・`menu::trigger` のアクセシブル名は
//! いずれも [`visually_hidden::root`] で「、{タイトル}」を追加して行ごとに
//! 一意化する（`menu::trigger` の可視テキストは全行共通の「…」のみのため、
//! `aria-label` 固定文言のままでは行を移動する利用者がどのタスクの操作か
//! 区別できない、codex レビュー指摘・イシュー #2925）。id と異なり
//! アクセシブル名は `children` 経由（`text(format!(...))`）で供給できる
//! ため寿命問題は生じない。
//!
//! # `href="#"` を使わない
//!
//! 他 block と同型の判断により、外部の実在 URL（[`REPO`]）を固定で使う。
//!
//! # `<form>` を使わない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない。ボタンは `button::button` の既定 `type="button"` のまま
//! 用い、送信処理・送信先は一切持たない。
//!
//! # ダミー素材について
//!
//! 作成者名は `crate::blocks::dummy_assets::PERSON_NAMES`（架空セット）を
//! 使う。タイトル・日付・環境名はすべて独自に書いた架空の文言であり、
//! 実在の人物・企業・プロジェクトとは無関係。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
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
/// は常に同じ日時を指す組にする不変条件（他 block と同型）。
fn meta_line(iso: &str, label: &str, author: &str) -> Node {
    el(
        "p",
        vec![("class", "blocks-list-title-meta-meta")],
        vec![
            el("time", vec![("datetime", iso)], vec![text(label)]),
            span(vec![("aria-hidden", "true")], vec![text(" \u{b7} ")]),
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
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/list-title-meta/",
    title: "list-title-meta",
    category: BlockCategory::List,
    rust_source: "crates/docs-site/src/blocks/application/list/list_title_meta.rs",
    demo_class: "blocks-list-title-meta",
    parts: &[
        Part {
            label: "List",
            path: "/themes/list/",
        },
        Part {
            label: "Badge",
            path: "/themes/badge/",
        },
        Part {
            label: "Status",
            path: "/themes/status/",
        },
        Part {
            label: "Avatar",
            path: "/themes/avatar/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Menu",
            path: "/themes/menu/",
        },
        Part {
            label: "Link",
            path: "/themes/link/",
        },
        Part {
            label: "Link Overlay",
            path: "/themes/link-overlay/",
        },
        Part {
            label: "Visually Hidden",
            path: "/themes/visually-hidden/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `list_title_meta` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS` doc
/// 「block 固有 CSS の置き場」節と同型）。
///
/// # 行の骨格をどこに置くか
///
/// R1287/R1285 の行は `list::item`（`<li>`）自身に `blocks-list-title-meta-row`
/// クラスを直接付与できる（`list::item` は `drop_class_attr` を経由しない
/// ため呼び出し側 `class` がそのまま残る）。R1297 の行は `list::item` の下に
/// `link_overlay::root` を挟むが、`link_overlay::root` は `drop_class_attr`
/// で呼び出し側 `class` を必ず落とすため、`.blocks-list-title-meta-panel`
/// を起点にした属性セレクタ `[data-scope="link-overlay"][data-part="root"]`
/// でスコープする（`list::root` の呼び出し側 `class` も同じ理由で落ちる
/// ため、パネル外枠には別途 `div` を巻いている、[`panel`] 参照）。
/// [`deploy_item`] の `list::item` 自体には `blocks-list-title-meta-row`
/// を付与しない（内側の `link_overlay::root` と二重適用になり
/// `padding-block` が 2 倍になるため。詳細は [`deploy_item`] 参照）。
///
/// # `list::item` の grid 化が Plain variant の `display: flex` に負けない理由
///
/// `pre-styled-ui` の `ListVariant::Plain` は
/// `[data-scope="list"][data-part="root"].fd-list--variant-plain > [data-scope="list"][data-part="item"]`
/// （5 セレクタ分の詳細度）へ `display: flex` を宣言する。単なる
/// `.blocks-list-title-meta-row { display: grid; }`（詳細度 1）はこれに
/// 負けて grid が無効化される。そのためこのセレクタは
/// `.blocks-list-title-meta-panel [data-scope="list"][data-part="root"]
/// [data-scope="list"][data-part="item"].blocks-list-title-meta-row`
/// （詳細度 6）まで持ち上げ、CSS ソース順に頼らず確実に勝つようにする。
///
/// # `list` recipe の item 余白の上書き
///
/// `list` recipe は item に既定 `margin-block: var(--fandhe-space-1)` を
/// 与えるが、本 block は区切り線（`border-top`）を挟む密な一覧を志向する
/// ため `margin-block: 0` へ上書きする（`list_narrow_activity`/`list_people`
/// と同型の判断）。
const LAYOUT_CSS: &str = "\
.blocks-list-title-meta-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n}\n\
.blocks-list-title-meta-section {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-list-title-meta-section-title {\n  margin: 0;\n  font-size: var(--fandhe-font-font-size-sm);\n  font-weight: var(--fandhe-font-font-weight-medium);\n  color: var(--fandhe-color-fg-muted);\n}\n\
.blocks-list-title-meta-panel {\n  display: flex;\n  flex-direction: column;\n  container-type: inline-size;\n  container-name: blocks-list-title-meta;\n}\n\
.blocks-list-title-meta-panel [data-scope=\"list\"][data-part=\"item\"] {\n  margin-block: 0;\n}\n\
.blocks-list-title-meta-panel [data-scope=\"list\"][data-part=\"item\"] + [data-scope=\"list\"][data-part=\"item\"] {\n  border-top: 1px solid var(--fandhe-color-border);\n}\n\
.blocks-list-title-meta-panel [data-scope=\"list\"][data-part=\"root\"] [data-scope=\"list\"][data-part=\"item\"].blocks-list-title-meta-row,\n.blocks-list-title-meta-panel [data-scope=\"link-overlay\"][data-part=\"root\"] {\n  display: grid;\n  grid-template-columns: minmax(0, 1fr) auto;\n  align-items: center;\n  gap: var(--fandhe-space-4);\n  padding-block: var(--fandhe-space-4);\n  width: 100%;\n}\n\
.blocks-list-title-meta-body {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1);\n  min-width: 0;\n}\n\
.blocks-list-title-meta-title-row {\n  display: flex;\n  flex-wrap: wrap;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n  min-width: 0;\n}\n\
.blocks-list-title-meta-title-text {\n  font-weight: var(--fandhe-font-font-weight-medium);\n}\n\
.blocks-list-title-meta-meta {\n  margin: 0;\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-1);\n  color: var(--fandhe-color-fg-muted);\n  font-size: var(--fandhe-font-font-size-sm);\n  overflow: hidden;\n  text-overflow: ellipsis;\n  white-space: nowrap;\n}\n\
.blocks-list-title-meta-trailing {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-3);\n  flex-shrink: 0;\n}\n\
.blocks-list-title-meta-avatar-count {\n  font-size: var(--fandhe-font-font-size-sm);\n  color: var(--fandhe-color-fg-muted);\n}\n\
.blocks-list-title-meta-arrow {\n  color: var(--fandhe-color-fg-muted);\n}\n\
.blocks-list-title-meta-panel [data-blocks-list-title-meta-view-button] {\n  display: none;\n}\n\
@container blocks-list-title-meta (min-width: 40rem) {\n  \
.blocks-list-title-meta-panel [data-blocks-list-title-meta-view-button] {\n    display: inline-flex;\n  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    fn demo_html() -> String {
        render(&demo())
    }

    #[test]
    fn demo_composes_expected_parts() {
        let html = demo_html();
        for scope in [
            "data-scope=\"list\"",
            "data-scope=\"badge\"",
            "data-scope=\"status\"",
            "data-scope=\"avatar\"",
            "data-scope=\"button\"",
            "data-scope=\"menu\"",
            "data-scope=\"link\"",
            "data-scope=\"link-overlay\"",
            "data-scope=\"visually-hidden\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
    }

    #[test]
    fn demo_wires_all_three_variant_hooks() {
        let html = demo_html();
        for variant in ["actions", "avatars", "deploy"] {
            assert!(html.contains(&format!(
                "data-blocks-list-title-meta-variant=\"{variant}\""
            )));
        }
    }

    #[test]
    fn no_form_semantics_or_dead_links() {
        let html = demo_html();
        assert!(!html.contains("<form"));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("src=\"data:"));
    }

    #[test]
    fn menu_ids_have_no_duplicates() {
        let html = demo_html();
        let mut ids: Vec<&str> = Vec::new();
        for chunk in html.split("id=\"").skip(1) {
            if let Some(end) = chunk.find('"') {
                ids.push(&chunk[..end]);
            }
        }
        let mut sorted = ids.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(ids.len(), sorted.len(), "id が重複している: {ids:?}");
    }

    #[test]
    fn all_buttons_are_type_button() {
        let html = demo_html();
        // headless-ui の button/menu トリガーはいずれも既定 `type="button"`
        // を固定する契約（誤って `submit` になっていないことの回帰固定）。
        let count_open = html.matches("<button").count();
        let count_typed = html.matches("type=\"button\"").count();
        assert!(count_open > 0);
        assert!(count_typed >= count_open, "html={html}");
    }

    #[test]
    fn has_time_element_with_datetime() {
        let html = demo_html();
        assert!(html.contains("<time datetime="));
    }

    #[test]
    fn layout_css_is_safe_and_declares_container_query() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("container-type: inline-size;"));
        assert!(LAYOUT_CSS.contains("@container blocks-list-title-meta (min-width: 40rem)"));
    }

    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = demo_html();
        assert!(html.contains("class=\"blocks-list-title-meta-layout\""));
        assert_ne!(super::BLOCK.demo_class, "blocks-list-title-meta-layout");
    }

    #[test]
    fn demo_is_deterministic() {
        assert_eq!(demo_html(), demo_html());
    }
}
