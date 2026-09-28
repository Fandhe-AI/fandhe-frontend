//! `list-people` block（イシュー #2923。Application/List カテゴリの最初の
//! block、親トラッキング #2892「Blocks 目的別パーツ拡充」配下）。区切り線で
//! 仕切った縦のスタックリストに、人物 1 件ずつ（アバター・名前・メール +
//! 右側の役職・最終ログイン状態）を並べる合成例。対応表 ID R1283（主参照・
//! 代表構成）を軸に、R1284（行全体リンク）/ R1286（インラインリンク +
//! 三点メニュー）/ R1288（カード枠）/ R1289・R1294（2 カラム + 行末ボタン）/
//! R1290（ホバー面色・段階的余白）を 5 例へ集約する。`_/blocks-intake/`
//! の対応ファイルは本イシュー着手時点で本 worktree に存在しないため、原稿・
//! 本コメントには対応表 ID のみを記す（`card-form-footer`〔イシュー #2899〕
//! と同じ扱い）。R1291（最大幅の制限だけが違う）・R1292（役職・最終ログイン
//! 欄がない）は原稿の差分メモのみで扱う（実装は集約しない）。
//!
//! # 使用部品
//!
//! `list` / `avatar` / `link` / `link-overlay` / `button` / `menu` / `card` /
//! `status` / `visually-hidden` の 9 部品を合成する（[`BLOCK`] の `parts` に
//! 一致させる契約、`crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs`
//! が検証する）。新しい UI 部品は追加しない。
//!
//! # 狭幅で回り込む理由（`@container`）
//!
//! Demo 枠の幅はビューポート幅と一致しないため、`@media` ではなく
//! `form_layout_two_column`/`description_list_horizontal` と同型の
//! `@container` を使う。各パネル（`.blocks-list-people-panel`）へ
//! `container-type: inline-size; container-name: blocks-list-people;` を
//! 宣言し、`(min-width: 40rem)` のときだけメタ欄（役職 + 最終ログイン）を
//! 右寄せへ切り替える。既定（狭幅）は名前の下にメタ欄が縦積みになる
//! （隠さず回り込ませる、イシュー本文の要件）。2 カラム例（例 5）のパネル
//! は列幅がコンテナ幅の約半分になるため `container-name: blocks-list-people`
//! を持たせず（`.blocks-list-people-panel-columns`）、この 40rem クエリの
//! 対象外として常に縦積み（コンパクト形）に固定する（codex P1 指摘）。
//!
//! # list recipe の item 余白を上書きする理由
//!
//! `list::root(_, ListVariant::Plain, ..)` は recipe 側で
//! `[data-scope="list"][data-part="item"] { margin-block: var(--fandhe-space-1) }`
//! と `[data-part="root"].fd-list--variant-plain > [data-part="item"] {
//! align-items: flex-start }`（詳細度 (0,2,0)/(0,5,0)）を持つため、単一
//! class（(0,1,0)）の `.blocks-list-people-row` 規則はこれらに負け、行の
//! 余白が二重になり広幅でも縦中央揃えが効かない不具合があった
//! （Bugbot 指摘）。是正として行の基本規則（`margin-block: 0`・
//! `display: flex`・`flex-direction: column` 等）を
//! `:is(.blocks-list-people-panel, .blocks-list-people-panel-columns)
//! [data-scope="list"][data-part="root"].fd-list--variant-plain >
//! .blocks-list-people-row`（詳細度 (0,5,0)、list recipe と同着でも
//! `assets/pre-styled-ui.css` より後段の `assets/blocks.css` で宣言される
//! ため後勝ちする）へ書き直した。
//!
//! # 広幅時（40rem 以上）の横並び規則も同じ詳細度まで引き上げる理由
//!
//! 上記の是正で行の基本規則が詳細度 (0,5,0) になった一方、`@container
//! blocks-list-people (min-width: 40rem)` 内の横並び規則は単一 class
//! （`.blocks-list-people-row`、詳細度 (0,1,0)）のままだったため、基本規則
//! （(0,5,0)）にも list recipe の item 規則（`align-items: flex-start`、
//! 詳細度 (0,3,0)）にも詳細度で負け、広幅でも縦積みのまま崩れなかった
//! （Codex P1 指摘・Cursor High 指摘）。是正として横並び規則のセレクタを
//! 基本規則と同じ `.blocks-list-people-panel [data-scope="list"]
//! [data-part="root"].fd-list--variant-plain > .blocks-list-people-row`
//! （詳細度 (0,5,0)）へ書き直し、CSS ソース順で基本規則より後段に置くこと
//! で両方に打ち勝たせた。
//!
//! # メールアドレスの省略記号が効かない理由
//!
//! `.blocks-list-people-email` は `overflow: hidden` + `text-overflow:
//! ellipsis` のみで `white-space: nowrap` を欠いていたため、長いメール
//! アドレスは省略されず折り返してしまっていた（Cursor Low 指摘）。
//! `white-space: nowrap` を追加した。
//!
//! # グリッド時（例 5）の区切り線を先頭視覚行だけ消す理由
//!
//! 48rem 以上でグリッド化すると DOM 順の `.row + .row` セレクタでは各
//! グリッドセルの右列にだけ上罫線が付いてしまう（第 1 視覚行の右セルに
//! 誤って罫線が出る、Bugbot 指摘）。是正として 48rem クエリ内で
//! `.blocks-list-people-panel-columns .blocks-list-people-row + .blocks-list-people-row`
//! を一旦無効化し、`:nth-child(n+3)` で 3 件目以降の行にのみ上罫線を
//! 再適用する（5 人・奇数でも DOM 順 1,2 / 3,4 / 5 の配置で左右の罫線が
//! 揃う）。
//!
//! # 行全体リンク（例 2）と インラインリンク + menu（例 3）を分ける理由
//!
//! `<a>` の内側に `menu::trigger`（対話要素）を入れ子にすると HTML の
//! interactive content 制約に反するため、行全体リンク（[`link_overlay`]）と
//! インラインリンク + 三点メニューは同一行へ同居させず別 example にする
//! （`card_media_footer` の「`menu` の id をページ内で 1 つに限る理由」と
//! 同型の判断軸を、ここでは「同一行に押し込まない」へ適用したもの）。
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
//! codex レビュー指摘であり削除した。`list_title_meta`〔イシュー #3370〕の
//! 同型判断を踏襲する）。
//!
//! # `menu`/ボタンの id をページ内で一意にする理由
//!
//! `demo_output_has_no_dangling_aria_references_or_duplicate_ids`
//! （`crates/docs-site/tests/blocks_contract.rs`）が id 重複を fail-closed に
//! 検知するため、`menu::trigger`/`menu::content` の id・行末ボタンの
//! アクセシブル名（[`visually_hidden::root`] で「、{名前}」を追加して一意化
//! する）は行ごとに一意にする（`card_media_footer`/`form_layout_two_column`
//! と同型）。
//!
//! # `<form>` を使わない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない。ボタンは `button::button` の既定 `type="button"` のまま
//! 用い、送信処理・送信先は一切持たない。
//!
//! # ダミー素材について
//!
//! 人名・役職は `crate::blocks::dummy_assets::PERSON_NAMES`/`JOB_TITLES`
//! （架空セット）を、アバター画像は `dummy_assets::AVATAR_SRC` を使う。
//! メールアドレスは `example.com`（IANA 予約ドメイン）固定のリテラルで
//! 組み立て、実在の人物・企業・PII を含まない。
//!
//! # 参照について
//!
//! 主参照は対応表 ID R1283、集約元は R1284/R1286/R1288/R1289/R1290/R1291/
//! R1292/R1294（詳細は `site/blocks/list-people.md` の「集約元との差分
//! メモ」節）。文言・配色・アイコンは独自に書く（他 block と同じライセンス
//! 上の転記制限）。実在の人物・企業名・PII は使わない。
//!
//! # chevron が行クリックを奪う理由（Bugbot 指摘）
//!
//! 例 2・4 の chevron（`.blocks-list-people-chevron`）は装飾用の絶対配置
//! `<span aria-hidden>` だが、`link_overlay::overlay` より DOM 順で後に
//! あるため描画スタック上は overlay より手前に来て、chevron の矩形内の
//! クリックが overlay の `<a>` ではなく chevron 自身に当たり行クリック
//! ナビゲーションが機能しなかった。`pointer-events: none;` を追加し、
//! chevron を常にクリック透過にした。
//!
//! # 3 要素構成の行が広幅で中央分離する理由（Bugbot 指摘）
//!
//! 40rem クエリの `justify-content: space-between` は行の直接の flex
//! 子要素すべてへ等間隔配置を適用するため、例 3（body/meta/menu の 3
//! 要素構成）では meta と menu が両端へ引き離され、右寄せグループとして
//! まとまらなかった（`justify-content: space-between` は子要素数に依らず
//! 先頭と末尾を両端へ、残りを均等配置する仕様上の帰結）。是正として
//! meta と menu を `.blocks-list-people-trailing` 1 つの子要素へまとめ、
//! 行の直接の子を常に 2 つ（本体・trailing）に固定する。狭幅では
//! trailing 自体を縦積み（既定の `flex-direction: column`）にし、40rem
//! 以上でのみ横並びへ切り替える。
//!
//! # 行全体リンクの余白・クリック領域・chevron 重なりを root 側へ集約する
//! 理由（Bugbot 指摘・codex P1 指摘、2 回目の是正）
//!
//! 当初は「行（`<li>`、`.blocks-list-people-row-link`）に
//! `align-items: stretch` と `padding-inline-end` を持たせ、
//! `link-overlay::root` はその内側に収まる」構成だったが、3 つの不具合を
//! 生んでいた: (1) `align-items: stretch` が行の**全**子要素へ及ぶため、
//! 例 5（2 カラム）の「表示」ボタンまで行幅へ全幅化してしまう
//! （`.blocks-list-people-panel-columns` は 40rem クエリの対象外で
//! `flex-direction: row` へ切り替わらず、常にこの stretch の影響を受け
//! 続ける）。(2) `padding-inline-end` が行側にあると、`link-overlay::root`
//! はその内側（行の content box）に収まるため `inset: 0` の overlay も
//! root 幅までしか覆わず、予約余白（chevron の矩形がある領域）をクリック
//! しても overlay の `<a>` に当たらない。(3) chevron は root の子で
//! `position: relative` な root 自身の box 内に留まるが、root 自身は
//! 予約余白を持たないため、meta のテキストが root の右端（chevron が
//! `inset-inline-end` で陣取る位置のすぐ内側）まで届いてしまい重なる。
//!
//! 是正として `align-self: stretch`（行の `align-items` に依存せず常に
//! 行幅へ揃う）と `padding-inline-end: space-8`（メタ情報の予約余白）を
//! いずれも `link-overlay::root` 自身へ移した。`inset: 0` の overlay は
//! `root` の padding edge（＝ root の border box 全体）を覆うため、この
//! 予約余白ごとクリック可能になり (2) を解消する。meta は root の
//! content box（予約余白を除いた領域）までしか描画されないため chevron
//! と重ならず (3) も解消する。行自身は `align-items` を宣言しなくなり
//! list recipe の既定（`align-items: flex-start`）に委ねるため、例 5 の
//! ボタンは自然な幅のまま (1) も解消する。hover/focus-within 規則
//! （`.blocks-list-people-row-link` 側の `padding-inline` ショートハンド）
//! は別要素（`<li>` 自身、装飾用の背景余白）を触るだけになり、root の
//! `padding-inline-end` とは競合しなくなった。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps, CardVariant};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::link_overlay;
use fandhe_frontend_pre_styled_ui::list::{self, ListType, ListVariant};
use fandhe_frontend_pre_styled_ui::menu::{self, OpenState};
use fandhe_frontend_pre_styled_ui::status::{self, StatusProps};
use fandhe_frontend_pre_styled_ui::visually_hidden;
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// 外部の実在 URL（`href="#"` は使わない、他 block と同型の判断）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// 人物 1 件分の架空データ。
struct Person {
    /// [`dummy_assets::PERSON_NAMES`] への添字。
    name_index: usize,
    email: &'static str,
    /// [`dummy_assets::JOB_TITLES`] への添字。
    role_index: usize,
    presence: Presence,
}

/// 右側メタ欄の在席状態表示。
enum Presence {
    /// 最終ログイン日時（`<time datetime>` + ラベル。`iso`/`label` は
    /// 常に同じ日時を指す組にする）。
    LastSeen {
        iso: &'static str,
        label: &'static str,
    },
    /// オンライン中（[`status::root`]、`ColorPalette::Success`）。
    Online,
}

const PEOPLE: [Person; 5] = [
    Person {
        name_index: 0,
        email: "haruto.fujimaki@example.com",
        role_index: 0,
        presence: Presence::Online,
    },
    Person {
        name_index: 1,
        email: "elena.vasquez@example.com",
        role_index: 1,
        presence: Presence::LastSeen {
            iso: "2026-09-27T09:14:00+09:00",
            label: "9 月 27 日 9:14",
        },
    },
    Person {
        name_index: 2,
        email: "kwame.boateng@example.com",
        role_index: 2,
        presence: Presence::LastSeen {
            iso: "2026-09-25T18:40:00+09:00",
            label: "9 月 25 日 18:40",
        },
    },
    Person {
        name_index: 3,
        email: "mei.lindqvist@example.com",
        role_index: 3,
        presence: Presence::Online,
    },
    Person {
        name_index: 4,
        email: "noor.al-sayed@example.com",
        role_index: 4,
        presence: Presence::LastSeen {
            iso: "2026-09-20T11:02:00+09:00",
            label: "9 月 20 日 11:02",
        },
    },
];

/// 氏名からイニシャル（先頭文字 + 姓の頭文字）を組み立てる（`avatar::fallback`
/// 用、`grid_list_compact_tiles` と同型のヘルパ）。
fn initials(name: &str) -> String {
    name.split_whitespace()
        .filter_map(|part| part.chars().next())
        .collect()
}

/// アバター 1 件（`avatar::image` + `avatar::fallback`）。
fn person_avatar(name: &str) -> Node {
    avatar::root(
        &AvatarProps {
            size: Size::Md,
            ..AvatarProps::default()
        },
        vec![],
        vec![
            avatar::image(ImageStatus::Loaded, dummy_assets::AVATAR_SRC, "", vec![]),
            avatar::fallback(ImageStatus::Loaded, vec![], vec![text(initials(name))]),
        ],
    )
}

/// 在席状態の表示ノード（メタ欄の 1 パーツ）。
fn presence_node(presence: &Presence) -> Node {
    match presence {
        Presence::LastSeen { iso, label } => div(
            vec![("class", "blocks-list-people-presence")],
            vec![
                text("最終ログイン: "),
                el("time", vec![("datetime", *iso)], vec![text(*label)]),
            ],
        ),
        Presence::Online => status::root(
            &StatusProps {
                size: Size::Sm,
                palette: ColorPalette::Success,
            },
            vec![],
            vec![status::indicator(vec![]), text("オンライン")],
        ),
    }
}

/// 本体（アバター + 名前/メール）。行全体リンク example では `name_node`
/// にインラインリンクを差し込まず素のテキストのまま使う（リンクは行全体か
/// 名前のみのどちらか一方に限る、モジュール doc「行全体リンクと
/// インラインリンクを分ける理由」節）。
fn body(name: &str, email: &str, name_node: Node) -> Node {
    div(
        vec![("class", "blocks-list-people-body")],
        vec![
            person_avatar(name),
            div(
                vec![("class", "blocks-list-people-identity")],
                vec![
                    name_node,
                    div(
                        vec![("class", "blocks-list-people-email")],
                        vec![text(email)],
                    ),
                ],
            ),
        ],
    )
}

/// メタ欄（役職 + 在席状態）。
fn meta(role: &str, presence: &Presence) -> Node {
    div(
        vec![("class", "blocks-list-people-meta")],
        vec![
            div(vec![("class", "blocks-list-people-role")], vec![text(role)]),
            presence_node(presence),
        ],
    )
}

/// 例 1: 代表構成（R1283）。リンクなし、行末操作なし。
fn example_representative() -> Node {
    let rows: Vec<Node> = PEOPLE
        .iter()
        .map(|person| {
            let name = dummy_assets::PERSON_NAMES[person.name_index];
            let role = dummy_assets::JOB_TITLES[person.role_index];
            list::item(
                vec![("class", "blocks-list-people-row")],
                vec![
                    body(
                        name,
                        person.email,
                        div(vec![("class", "blocks-list-people-name")], vec![text(name)]),
                    ),
                    meta(role, &person.presence),
                ],
            )
        })
        .collect();
    div(
        vec![("class", "blocks-list-people-panel")],
        vec![list::root(
            ListType::Unordered,
            ListVariant::Plain,
            vec![],
            rows,
        )],
    )
}

/// 例 2: 行全体リンク + ホバー面色 + 段階的余白（R1284/R1290）。
fn example_row_link() -> Node {
    let rows: Vec<Node> = PEOPLE
        .iter()
        .map(|person| {
            let name = dummy_assets::PERSON_NAMES[person.name_index];
            let role = dummy_assets::JOB_TITLES[person.role_index];
            list::item(
                vec![(
                    "class",
                    "blocks-list-people-row blocks-list-people-row-link",
                )],
                vec![link_overlay::root(
                    vec![],
                    vec![
                        body(
                            name,
                            person.email,
                            div(vec![("class", "blocks-list-people-name")], vec![text(name)]),
                        ),
                        meta(role, &person.presence),
                        link_overlay::overlay(REPO, vec![("aria-label", name)], vec![]),
                        el(
                            "span",
                            vec![
                                ("class", "blocks-list-people-chevron"),
                                ("aria-hidden", "true"),
                            ],
                            vec![text("\u{203a}")],
                        ),
                    ],
                )],
            )
        })
        .collect();
    div(
        vec![("class", "blocks-list-people-panel")],
        vec![list::root(
            ListType::Unordered,
            ListVariant::Plain,
            vec![],
            rows,
        )],
    )
}

/// 例 3: インラインリンク + 三点メニュー（R1286）。`menu` の id は行ごとに
/// 一意にする（モジュール doc「id をページ内で一意にする理由」節）。
fn example_inline_link_menu() -> Node {
    let rows: Vec<Node> = PEOPLE
        .iter()
        .enumerate()
        .map(|(i, person)| {
            let name = dummy_assets::PERSON_NAMES[person.name_index];
            let role = dummy_assets::JOB_TITLES[person.role_index];
            let content_id = format!("blocks-list-people-menu-{i}");
            let trigger_id = format!("blocks-list-people-menu-trigger-{i}");
            // 5 件の menu::trigger が同一 aria-label だとスクリーンリーダー
            // 利用時に区別できないため、行末ボタンと同様に氏名を付与して
            // 一意にする（Low 指摘）。
            let trigger_label = format!("その他の操作、{name}");
            let menu_root = menu::root(
                Size::Sm,
                OpenState::Closed,
                vec![],
                vec![
                    menu::trigger(
                        OpenState::Closed,
                        true,
                        Some(content_id.as_str()),
                        vec![
                            ("id", trigger_id.as_str()),
                            ("aria-label", trigger_label.as_str()),
                        ],
                        vec![text("\u{2026}")],
                    ),
                    menu::positioner(
                        OpenState::Closed,
                        vec![],
                        vec![menu::content(
                            OpenState::Closed,
                            Some(content_id.as_str()),
                            Some(trigger_id.as_str()),
                            vec![],
                            vec![
                                menu::item(
                                    "profile",
                                    false,
                                    false,
                                    vec![],
                                    vec![text("プロフィールを見る")],
                                ),
                                menu::item(
                                    "message",
                                    false,
                                    false,
                                    vec![],
                                    vec![text("メッセージを送る")],
                                ),
                                menu::separator(vec![], vec![]),
                                menu::item("remove", false, false, vec![], vec![text("削除")]),
                            ],
                        )],
                    ),
                ],
            );
            list::item(
                vec![("class", "blocks-list-people-row")],
                vec![
                    body(
                        name,
                        person.email,
                        link::root(REPO, &LinkProps::default(), vec![], vec![text(name)]),
                    ),
                    div(
                        vec![("class", "blocks-list-people-trailing")],
                        vec![meta(role, &person.presence), menu_root],
                    ),
                ],
            )
        })
        .collect();
    div(
        vec![("class", "blocks-list-people-panel")],
        vec![list::root(
            ListType::Unordered,
            ListVariant::Plain,
            vec![],
            rows,
        )],
    )
}

/// 例 4: カード枠（R1288）。例 2 と同じ行構成をカードへ入れる。
fn example_card() -> Node {
    card::root(
        CardProps::from(CardVariant::Outline),
        vec![],
        vec![example_row_link()],
    )
}

/// 例 5: 2 カラム + 行末「表示」ボタン（R1289/R1294）。ボタンは disabled
/// 固定の静的表示（モジュール doc「menu/ボタンを disabled に固定する理由」
/// 節）。
fn example_two_column() -> Node {
    let rows: Vec<Node> = PEOPLE
        .iter()
        .map(|person| {
            let name = dummy_assets::PERSON_NAMES[person.name_index];
            let role = dummy_assets::JOB_TITLES[person.role_index];
            list::item(
                vec![("class", "blocks-list-people-row")],
                vec![
                    body(
                        name,
                        person.email,
                        div(vec![("class", "blocks-list-people-name")], vec![text(name)]),
                    ),
                    meta(role, &person.presence),
                    button::button(
                        &ButtonProps {
                            variant: ButtonVariant::Outline,
                            size: Size::Sm,
                            disabled: true,
                            ..ButtonProps::default()
                        },
                        vec![],
                        vec![
                            text("表示"),
                            visually_hidden::root(vec![], vec![text(format!("、{name}"))]),
                        ],
                    ),
                ],
            )
        })
        .collect();
    div(
        vec![("class", "blocks-list-people-two-column")],
        vec![div(
            vec![("class", "blocks-list-people-panel-columns")],
            vec![list::root(
                ListType::Unordered,
                ListVariant::Plain,
                vec![],
                rows,
            )],
        )],
    )
}

/// `list-people` の Demo 本体。呼び出しごとに同一の `Node` を返す純関数。
/// 5 例をラベル付き見出しで区切って並べる。
#[must_use]
pub fn demo() -> Node {
    let section = |label: &'static str, node: Node| -> Node {
        div(
            vec![("class", "blocks-list-people-section")],
            vec![
                el(
                    "h3",
                    vec![("class", "blocks-list-people-section-title")],
                    vec![text(label)],
                ),
                node,
            ],
        )
    };
    div(
        vec![("class", "blocks-list-people-layout")],
        vec![
            section("代表構成", example_representative()),
            section("行全体リンク", example_row_link()),
            section("インラインリンク + メニュー", example_inline_link_menu()),
            section("カード枠", example_card()),
            section("2 カラム + 行末ボタン", example_two_column()),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/list-people/",
    title: "list-people",
    category: BlockCategory::List,
    rust_source: "crates/docs-site/src/blocks/application/list/list_people.rs",
    demo_class: "blocks-list-people",
    parts: &[
        Part {
            label: "List",
            path: "/themes/list/",
        },
        Part {
            label: "Avatar",
            path: "/themes/avatar/",
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
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Menu",
            path: "/themes/menu/",
        },
        Part {
            label: "Card",
            path: "/themes/card/",
        },
        Part {
            label: "Status",
            path: "/themes/status/",
        },
        Part {
            label: "Visually Hidden",
            path: "/themes/visually-hidden/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `list_people` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS` doc
/// 「block 固有 CSS の置き場」節と同型）。
const LAYOUT_CSS: &str = "\
.blocks-list-people-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-10);\n}\n\
.blocks-list-people-section {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-list-people-section-title {\n  margin: 0;\n  font-size: var(--fandhe-font-font-size-sm);\n  font-weight: var(--fandhe-font-font-weight-medium);\n  color: var(--fandhe-color-fg-muted);\n}\n\
.blocks-list-people-panel {\n  display: flex;\n  flex-direction: column;\n  container-type: inline-size;\n  container-name: blocks-list-people;\n}\n\
.blocks-list-people-panel-columns {\n  display: flex;\n  flex-direction: column;\n}\n\
:is(.blocks-list-people-panel, .blocks-list-people-panel-columns) [data-scope=\"list\"][data-part=\"root\"].fd-list--variant-plain > .blocks-list-people-row {\n  margin-block: 0;\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n  padding-block: var(--fandhe-space-4);\n  position: relative;\n}\n\
.blocks-list-people-row + .blocks-list-people-row {\n  border-block-start: 1px solid var(--fandhe-color-border);\n}\n\
.blocks-list-people-row-link {\n  transition: background-color 0.15s ease;\n}\n\
.blocks-list-people-row-link:hover, .blocks-list-people-row-link:focus-within {\n  background-color: var(--fandhe-color-bg-subtle);\n  padding-inline: var(--fandhe-space-3);\n}\n\
.blocks-list-people-row-link > [data-scope=\"link-overlay\"][data-part=\"root\"] {\n  display: flex;\n  flex-direction: column;\n  flex: 1;\n  align-self: stretch;\n  min-width: 0;\n  gap: var(--fandhe-space-3);\n  padding-inline-end: var(--fandhe-space-8);\n}\n\
.blocks-list-people-body {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-3);\n  min-width: 0;\n}\n\
.blocks-list-people-identity {\n  display: flex;\n  flex-direction: column;\n  min-width: 0;\n}\n\
.blocks-list-people-name {\n  font-weight: var(--fandhe-font-font-weight-medium);\n}\n\
.blocks-list-people-identity > [data-scope=\"link\"][data-part=\"root\"] {\n  font-weight: var(--fandhe-font-font-weight-medium);\n}\n\
.blocks-list-people-email {\n  font-size: var(--fandhe-font-font-size-sm);\n  color: var(--fandhe-color-fg-muted);\n  white-space: nowrap;\n  overflow: hidden;\n  text-overflow: ellipsis;\n}\n\
.blocks-list-people-meta {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1);\n  font-size: var(--fandhe-font-font-size-sm);\n  color: var(--fandhe-color-fg-muted);\n}\n\
.blocks-list-people-presence {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-1);\n}\n\
.blocks-list-people-trailing {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-list-people-chevron {\n  position: absolute;\n  inset-inline-end: var(--fandhe-space-2);\n  top: 50%;\n  transform: translateY(-50%);\n  color: var(--fandhe-color-fg-muted);\n  pointer-events: none;\n}\n\
@container blocks-list-people (min-width: 40rem) {\n  \
.blocks-list-people-panel [data-scope=\"list\"][data-part=\"root\"].fd-list--variant-plain > .blocks-list-people-row {\n    flex-direction: row;\n    align-items: center;\n    justify-content: space-between;\n  }\n  \
.blocks-list-people-row-link > [data-scope=\"link-overlay\"][data-part=\"root\"] {\n    flex-direction: row;\n    align-items: center;\n    justify-content: space-between;\n  }\n  \
.blocks-list-people-meta {\n    align-items: flex-end;\n  }\n  \
.blocks-list-people-trailing {\n    flex-direction: row;\n    align-items: center;\n    gap: var(--fandhe-space-3);\n  }\n\
}\n\
.blocks-list-people-two-column {\n  container-type: inline-size;\n  container-name: blocks-list-people-two-column;\n}\n\
@container blocks-list-people-two-column (min-width: 48rem) {\n  \
.blocks-list-people-two-column [data-scope=\"list\"][data-part=\"root\"] {\n    display: grid;\n    grid-template-columns: repeat(2, minmax(0, 1fr));\n    column-gap: var(--fandhe-space-8);\n  }\n  \
.blocks-list-people-panel-columns .blocks-list-people-row + .blocks-list-people-row {\n    border-block-start: none;\n  }\n  \
.blocks-list-people-panel-columns .blocks-list-people-row:nth-child(n+3) {\n    border-block-start: 1px solid var(--fandhe-color-border);\n  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::{demo, example_inline_link_menu, example_two_column, LAYOUT_CSS, PEOPLE};
    use fandhe_frontend_core::render;

    fn demo_html() -> String {
        render(&demo())
    }

    #[test]
    fn demo_composes_expected_parts() {
        let html = demo_html();
        for scope in [
            "data-scope=\"list\"",
            "data-scope=\"avatar\"",
            "data-scope=\"link\"",
            "data-scope=\"link-overlay\"",
            "data-scope=\"button\"",
            "data-scope=\"menu\"",
            "data-scope=\"card\"",
            "data-scope=\"status\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
        assert!(html.contains("data-scope=\"visually-hidden\""));
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
    fn has_time_element_with_datetime() {
        let html = demo_html();
        assert!(html.contains("<time datetime="));
    }

    #[test]
    fn layout_css_is_safe_and_reflows_on_wide_container() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("container-type: inline-size;"));
        assert!(LAYOUT_CSS.contains("@container blocks-list-people (min-width: 40rem)"));
        assert!(LAYOUT_CSS.contains(":hover"));
    }

    #[test]
    fn layout_css_row_rule_outranks_list_recipe_plain_item_rule() {
        // list recipe の item 規則（`[data-part="root"].fd-list--variant-plain >
        // [data-part="item"] { align-items: flex-start }`、詳細度 (0,5,0)）に
        // 行の余白（margin-block 二重化）を上書きされないための回帰
        // （Bugbot 指摘）。`align-items` は行自身では宣言せず list recipe の
        // 既定（`flex-start`）へ委ねる（2 カラム例のボタン全幅化を防ぐ、
        // Medium 指摘。クリック領域の確保は `link-overlay::root` 自身の
        // `align-self: stretch` が担う、下記テスト参照）。
        assert!(LAYOUT_CSS.contains(
            "[data-scope=\"list\"][data-part=\"root\"].fd-list--variant-plain > .blocks-list-people-row {\n  margin-block: 0;\n  display: flex;\n  flex-direction: column;\n  gap:"
        ));
        assert!(!LAYOUT_CSS.contains("align-items: stretch"));
    }

    #[test]
    fn layout_css_wide_row_override_matches_base_rule_specificity() {
        // 40rem 以上の行横並び規則（`.blocks-list-people-row { flex-direction:
        // row; ... }`）は class 単体（詳細度 (0,1,0)）のままだと、狭幅の
        // 基本規則（`.blocks-list-people-panel [data-scope="list"]
        // [data-part="root"].fd-list--variant-plain > .blocks-list-people-row`、
        // 詳細度 (0,5,0)）にも list recipe の item 規則（`align-items:
        // flex-start`、詳細度 (0,3,0)）にも負けて横並びへ切り替わらない
        // （Codex P1 指摘・Cursor High 指摘、Bugbot 指摘と同根）。基本規則と
        // 同じ詳細度 (0,5,0) のセレクタへ書き直し、CSS ソース順で後勝ちさせる
        // ことで両方に打ち勝つ回帰。
        assert!(LAYOUT_CSS.contains(
            "@container blocks-list-people (min-width: 40rem) {\n  .blocks-list-people-panel [data-scope=\"list\"][data-part=\"root\"].fd-list--variant-plain > .blocks-list-people-row {\n    flex-direction: row;"
        ));
    }

    #[test]
    fn layout_css_email_truncates_with_nowrap() {
        // `white-space: nowrap` が無いと折り返してしまい `overflow: hidden` +
        // `text-overflow: ellipsis` が効かない（Cursor Low 指摘）。
        assert!(LAYOUT_CSS.contains(
            ".blocks-list-people-email {\n  font-size: var(--fandhe-font-font-size-sm);\n  color: var(--fandhe-color-fg-muted);\n  white-space: nowrap;\n  overflow: hidden;\n  text-overflow: ellipsis;\n}"
        ));
    }

    #[test]
    fn layout_css_two_column_grid_has_first_row_border_exception() {
        // 48rem 以上のグリッドで先頭視覚行（1・2 番目のセル）にだけ上罫線が
        // 付かないための回帰（Bugbot 指摘）。
        assert!(LAYOUT_CSS.contains("@container blocks-list-people-two-column (min-width: 48rem)"));
        assert!(LAYOUT_CSS.contains(".blocks-list-people-row:nth-child(n+3)"));
    }

    #[test]
    fn two_column_example_panel_is_not_the_narrow_wrap_container() {
        // 2 カラム例のパネルは `container-name: blocks-list-people`（40rem
        // クエリの対象）を持たず、幅に依らず常に縦積みへ固定される
        // ための回帰（codex P1 指摘）。
        let html = render(&example_two_column());
        assert!(html.contains("blocks-list-people-panel-columns\""));
        assert!(!html.contains("blocks-list-people-panel\""));
    }

    #[test]
    fn layout_css_chevron_is_click_transparent() {
        // 装飾用 chevron が overlay より DOM 順で後にあり、pointer-events:
        // none がないと行クリックナビゲーションを奪ってしまう
        // （Bugbot 指摘）。
        assert!(LAYOUT_CSS.contains(
            ".blocks-list-people-chevron {\n  position: absolute;\n  inset-inline-end: var(--fandhe-space-2);\n  top: 50%;\n  transform: translateY(-50%);\n  color: var(--fandhe-color-fg-muted);\n  pointer-events: none;\n}"
        ));
    }

    #[test]
    fn layout_css_row_link_reserves_space_for_chevron_on_root() {
        // chevron の絶対配置矩形が末尾のメタ情報と重ならないよう、
        // `link-overlay::root` 自身に予約余白（`padding-inline-end`）を
        // 持たせる回帰（Bugbot 指摘・codex P1 指摘、2 回目の是正）。
        // 行（`<li>`）側に持たせると overlay の `inset: 0` が root 幅までしか
        // 覆わず余白部分がクリックできない不具合があったため、root 自身へ
        // 移した（モジュール doc「行全体リンクの余白・クリック領域・chevron
        // 重なりを root 側へ集約する理由」節）。`align-self: stretch` も同じ
        // 規則内で root に付与し、行の `align-items` に依存せず常に行幅へ
        // 揃える。hover/focus-within 規則（`<li>` 側の `padding-inline`
        // ショートハンド）は別要素を触るだけになり、この予約余白とは
        // 競合しない。
        assert!(LAYOUT_CSS.contains(
            ".blocks-list-people-row-link > [data-scope=\"link-overlay\"][data-part=\"root\"] {\n  display: flex;\n  flex-direction: column;\n  flex: 1;\n  align-self: stretch;\n  min-width: 0;\n  gap: var(--fandhe-space-3);\n  padding-inline-end: var(--fandhe-space-8);\n}"
        ));
    }

    #[test]
    fn inline_link_menu_example_groups_meta_and_menu_into_trailing_wrapper() {
        // 例 3 の行は body/trailing の 2 要素構成に固定し、meta と menu が
        // 3 要素の space-between で中央分離しないようにする回帰
        // （Bugbot 指摘）。
        let html = render(&example_inline_link_menu());
        assert!(html.contains("class=\"blocks-list-people-trailing\""));
    }

    #[test]
    fn two_column_example_button_does_not_stretch_full_width() {
        // 行が `align-items` を宣言しなくなった（list recipe の既定
        // `flex-start` に委ねる）ことで、例 5 の「表示」ボタンが行幅へ
        // 全幅化しない回帰（Medium 指摘）。
        assert!(!LAYOUT_CSS.contains("align-items: stretch"));
        let html = render(&example_two_column());
        assert!(html.contains("data-scope=\"button\""));
    }

    #[test]
    fn inline_link_menu_example_trigger_labels_are_unique_per_row() {
        // 5 件の menu::trigger が同一 aria-label だとスクリーンリーダー
        // 利用時に区別できない回帰（Low 指摘）。氏名を含めて一意にする。
        let html = render(&example_inline_link_menu());
        let mut labels: Vec<&str> = Vec::new();
        for chunk in html.split("aria-label=\"").skip(1) {
            if let Some(end) = chunk.find('"') {
                labels.push(&chunk[..end]);
            }
        }
        assert_eq!(labels.len(), PEOPLE.len(), "行数と同数の aria-label が必要");
        let mut sorted = labels.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(
            labels.len(),
            sorted.len(),
            "menu::trigger の aria-label が重複している: {labels:?}"
        );
        for label in &labels {
            assert!(
                label.starts_with("その他の操作、"),
                "aria-label は氏名付きで一意化する: {label}"
            );
        }
    }

    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = demo_html();
        assert!(html.contains("class=\"blocks-list-people-layout\""));
        assert_ne!(super::BLOCK.demo_class, "blocks-list-people-layout");
    }
}
