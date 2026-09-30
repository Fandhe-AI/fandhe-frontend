//! `settings-integrations-list` block（イシュー #2993/#2994、親 #2992。
//! Application / Settings カテゴリ）。枠付き行リストで連携アプリを表示する
//! block。集約元は R0251（要望一覧 + 送信フォーム）/ R0252（個人用・組織用
//! 2 グループの代表構成）/ R0254（末尾の空状態）/ R0255（スイッチ展開 +
//! API キー欄）の 4 件。`_/blocks-intake/` の対応ファイルは本イシュー着手
//! 時点で本 worktree に存在しないため、原稿・本コメントには対応表 ID のみを
//! 記す（`settings_billing_overview` と同じ扱い）。
//!
//! # 版 A〜D と集約元の対応（#2994 で仕上げ）
//!
//! 単一 Demo 内に 4 つの「版」を縦に並記し、集約元ごとの差分を読み取れる
//! ようにする（[`demo`] 参照）。
//!
//! - **版 A**（R0252、#2993 で実装済み）: 個人用・組織用の 2 グループ、
//!   各行に接続/解除ボタン
//! - **版 B**（R0255）: 単一グループ 3 行。操作領域が接続ボタンではなく
//!   `switch`（有効化トグル）で、先頭行のみ展開して `clipboard`（API キー
//!   欄）を表示する。残り 2 行は折りたたみのまま
//! - **版 C**（R0251）: 単一グループ 2 行の末尾に「連携の要望」領域
//!   （要望済み一覧 + `field`/`input-group` による要望送信欄）
//! - **版 D**（R0254）: 単一グループ 2 行の末尾に `empty_state`（末尾の
//!   空状態枠）
//!
//! # 使用部品
//!
//! `badge` / `button` / `separator` / `link` / `image`（#2993 分）に加え、
//! `switch` / `clipboard` / `field` / `input-group` / `input` /
//! `empty-state` / `heading`（#2994 で追加）の 12 部品を合成する（[`BLOCK`]
//! の `parts` に一致させる契約、`blocks_nav.rs`/`blocks_contract.rs` が
//! 検証する）。`image` は使用部品一覧に無いロゴ表示のため追加した
//! （`page_heading_avatar::logo_image` と同じ判断）。新しい UI 部品は
//! 追加しない。
//!
//! # `clipboard` root は版 B の展開行 1 個に限る
//!
//! [`fandhe_frontend_pre_styled_ui::clipboard`] モジュール doc の
//! 「1 root : 1 状態機械契約」（`settings_api_key_created` と同じ制約）に
//! 従い、本 Demo 全体で `clipboard::root` の呼び出しは版 B の先頭行
//! （展開状態）1 箇所のみとする。
//!
//! # スイッチは readonly + disabled で静的固定
//!
//! 版 B の `switch::root` は `pricing_seats_split` と同じ判断で
//! `SwitchProps { readonly: true, disabled: true, .. }` を使う。`readonly`
//! は `data-readonly` を出すのみで native トグル操作自体を止めないため、
//! `disabled: true` も併用して [`fandhe_frontend_pre_styled_ui::switch::hidden_input`]
//! へ native `disabled` を出力し実際に操作を止める。展開（先頭行）/折りたたみ
//! （残り 2 行）の両状態を同一リスト内に静的に並記する。
//!
//! # グループ/版見出しは `heading` を使う（TOC 混入回避）
//!
//! `heading::heading` は `data-scope="heading"` を持つため
//! `crate::layout::with_heading_anchors` の TOC 収集（`.docs-toc`）から
//! 除外される。#2993 時点の素の `h3` はこの除外対象外で TOC へ混入し得た
//! （`settings_api_keys_table` の Bugbot 指摘と同型）ため、本イシューで
//! グループ見出し・版見出しの双方を `heading` へ寄せた。CSS フックは
//! `class`（`heading` は `drop_class_attr` で呼び出し側 `class` を除去する）
//! ではなく `data-blocks-settings-integrations-list-group-title`/
//! `-version-title` の `data-*` 属性で渡す。
//!
//! # 要望フォームは `<form>` を持たず送信先も持たない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、版 C の要望送信欄は
//! `<form>` を出力しない静的表示のみで、送信処理・送信先は一切持たない。
//! 送信ボタンは `button::button` の既定 `type="button"` のまま用いる。
//!
//! # `class` と `data-*` の使い分け
//!
//! [`fandhe_frontend_pre_styled_ui`] の各パーツ関数はいずれも
//! `drop_class_attr` で呼び出し側 `class` を除去してから内部 variant クラス
//! と合成するため、これらへの CSS フックは `data-*` 属性で渡す
//! （`data-blocks-settings-integrations-list-*`）。レイアウト用ラッパー
//! （グループ・行・本文・操作領域・要望領域・空状態領域）は素の
//! `class="blocks-settings-integrations-list-*"` を使う。
//!
//! # 狭幅切替はコンテナクエリで判定する
//!
//! Demo 枠の幅はビューポート幅と一致しないため、`settings_billing_overview`
//! 等と同型に `@container`（コンテナクエリ）で判定する。
//! `.blocks-settings-integrations-list-stack` へ `container-type: inline-size`
//! を宣言し、コンテナ幅が `40rem` 未満のとき行のグリッドを「ロゴ + 本文」を
//! 上段、「操作」を中段、「API キー欄（版 B 展開行のみ）」を下段へ回す
//! 3 行構成へ切り替える。
//!
//! # 詳細リンクの遷移先
//!
//! 架空アプリの詳細ページは存在しないため、`href="#"` を使わず実在 URL
//! （本リポジトリ `https://github.com/Fandhe-AI/fandhe-frontend`）へ
//! `LinkProps::external = true` でリンクする（PR #3440
//! `settings_integration_detail` と同じ判断。`external: true` は
//! `rel="noopener noreferrer"` を付与し reverse tabnabbing を防ぐ）。
//! リンク文言は遷移先の実態（本リポジトリの GitHub ページ）と一致させ
//! 「GitHub で見る」とする（近隣 block と同じ表記、PR #3441 Codex/Bugbot
//! 指摘の是正。「詳細を見る」は全アプリ同一の遷移先と矛盾するため使わない）。
//!
//! # API キー値・ダミー素材について
//!
//! API キー値（[`API_KEY_DEMO`]）は `fd_demo_` 接頭辞の明白な架空パターン
//! で、実クレデンシャル形式・実企業名・PII を含まない。アプリ名
//! （Lattice Notes / Harbor Calendar / Pulse Alerts / Ledger Sync /
//! Beacon Chat / Quarry Storage）・要望済みアプリ名（Flow Board / Signal
//! Mail）・説明文はすべて架空で、実在の企業・製品・商標・PII は含まない。
//! ロゴは共通ダミー画像 [`crate::blocks::dummy_assets::LOGO_SRC`] を使う。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets::LOGO_SRC;
use fandhe_frontend_core::{div, li, text, ul, Node};
use fandhe_frontend_pre_styled_ui::badge::{badge, BadgeProps, BadgeVariant};
use fandhe_frontend_pre_styled_ui::button::{button, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::clipboard;
use fandhe_frontend_pre_styled_ui::empty_state::{self, EmptyStateProps, EmptyStateVariant};
use fandhe_frontend_pre_styled_ui::field::{
    self, FieldIds, FieldOrientation, FieldProps, FieldRootProps,
};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::image::{image, ImageFit, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::input::{self, InputProps};
use fandhe_frontend_pre_styled_ui::input_group::{self, InputGroupAlign, InputGroupProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::separator::{separator, SeparatorProps};
use fandhe_frontend_pre_styled_ui::switch::{self, SwitchProps};
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// 詳細リンクの遷移先（架空アプリのため本リポジトリへの外部リンクで代替、
/// モジュール doc「詳細リンクの遷移先」節参照）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// 版 B 展開行の API キーデモ値（モジュール doc「API キー値・ダミー素材に
/// ついて」節参照。`fd_demo_` 接頭辞の明白な架空パターンで実クレデンシャル
/// 形式と衝突しない）。
const API_KEY_DEMO: &str = "fd_demo_7f0c1a2e9b3d4f5a8c6e";

/// 連携アプリ 1 件分のデータ（名前・説明・接続状態）。
struct Integration {
    name: &'static str,
    description: &'static str,
    connected: bool,
}

/// グループ 1 件分（見出し + 所属アプリ一覧）。
struct Group {
    title: &'static str,
    items: &'static [Integration],
}

/// 個人用・組織用の 2 グループ（各 3 件、接続済み/未接続を混在させ両分岐を
/// Demo に出す）。版 B〜D も本配列の一部を再利用する（モジュール doc
/// 「版 A〜D と集約元の対応」節参照）。
const GROUPS: &[Group] = &[
    Group {
        title: "個人用",
        items: &[
            Integration {
                name: "Lattice Notes",
                description: "メモとドキュメントを同期するノートアプリ。",
                connected: true,
            },
            Integration {
                name: "Harbor Calendar",
                description: "予定を共有カレンダーへ反映する。",
                connected: false,
            },
            Integration {
                name: "Pulse Alerts",
                description: "重要な通知をモバイルへ転送する。",
                connected: true,
            },
        ],
    },
    Group {
        title: "組織用",
        items: &[
            Integration {
                name: "Ledger Sync",
                description: "経費データを会計システムへ同期する。",
                connected: false,
            },
            Integration {
                name: "Beacon Chat",
                description: "チームチャットへ更新情報を投稿する。",
                connected: true,
            },
            Integration {
                name: "Quarry Storage",
                description: "添付ファイルをクラウドストレージへ保管する。",
                connected: false,
            },
        ],
    },
];

/// 連携アプリのロゴ画像（共通ダミー画像、`data-*` で CSS フックを渡す）。
/// `image` recipe の base（`[data-scope="image"][data-part="root"]`、詳細度
/// (0,2,0)）に `height: auto`/`max-width: 100%` が乗るため、[`LAYOUT_CSS`]
/// 側は `img[data-scope="image"][data-blocks-settings-integrations-list-logo]`
/// （詳細度 (0,2,1)）で上回る（`page_heading_avatar::logo_image` と同じ
/// 判断、イシュー #2931 Bugbot 指摘の再発形）。
fn logo(name: &str) -> Node {
    image(
        &ImageProps {
            shape: ImageShape::Rounded,
            fit: ImageFit::Contain,
            ..ImageProps::new(LOGO_SRC, &format!("{name} のロゴ"))
        },
        vec![("data-blocks-settings-integrations-list-logo", "")],
    )
}

/// 接続状態バッジ（本文の `name_row` に埋め込む）。
fn status_badge(item: &Integration) -> Node {
    if item.connected {
        badge(
            &BadgeProps {
                variant: BadgeVariant::Subtle,
                ..BadgeProps::default()
            },
            vec![("data-blocks-settings-integrations-list-status", "")],
            vec![text("接続済み")],
        )
    } else {
        badge(
            &BadgeProps {
                variant: BadgeVariant::Outline,
                ..BadgeProps::default()
            },
            vec![("data-blocks-settings-integrations-list-status", "")],
            vec![text("未接続")],
        )
    }
}

/// 接続/解除操作ボタン（版 A/C/D の操作領域）。
///
/// 接続/解除ボタンの可視ラベルは全行共通（「接続」/「解除」）のため、
/// 支援技術がどのアプリへの操作か区別できるよう `aria-label` へアプリ名を
/// 埋め込む（PR #3441 Codex 指摘）。
fn action_button(item: &Integration) -> Node {
    if item.connected {
        button(
            &ButtonProps {
                variant: ButtonVariant::Outline,
                ..ButtonProps::default()
            },
            vec![
                ("data-blocks-settings-integrations-list-action", ""),
                ("aria-label", &format!("{} の連携を解除", item.name)),
            ],
            vec![text("解除")],
        )
    } else {
        button(
            &ButtonProps {
                variant: ButtonVariant::Solid,
                ..ButtonProps::default()
            },
            vec![
                ("data-blocks-settings-integrations-list-action", ""),
                ("aria-label", &format!("{} と連携", item.name)),
            ],
            vec![text("接続")],
        )
    }
}

/// 枠付きリストの 1 行（ロゴ・本文（名前 + 状態バッジ + 説明 + 詳細
/// リンク）・操作領域・任意の展開領域）。
///
/// `actions` は操作領域の中身（版 A/C/D は [`action_button`]、版 B は
/// `switch::root`）、`expanded` は版 B の展開行のみが持つ API キー欄
/// （`grid-area: key`）。
fn row_with(item: &Integration, actions: Node, expanded: Option<Node>) -> Node {
    let name_row = div(
        vec![("class", "blocks-settings-integrations-list-name")],
        vec![text(item.name), status_badge(item)],
    );

    let description_row = div(
        vec![("class", "blocks-settings-integrations-list-description")],
        vec![
            text(item.description),
            text(" "),
            link::root(
                REPO,
                &LinkProps {
                    external: true,
                    ..LinkProps::default()
                },
                vec![("data-blocks-settings-integrations-list-detail", "")],
                vec![text("GitHub で見る")],
            ),
        ],
    );

    let body = div(
        vec![("class", "blocks-settings-integrations-list-body")],
        vec![name_row, description_row],
    );

    let actions_wrap = div(
        vec![("class", "blocks-settings-integrations-list-actions")],
        vec![actions],
    );

    let mut children = vec![logo(item.name), body, actions_wrap];
    if let Some(expanded) = expanded {
        children.push(expanded);
    }

    li(
        vec![("class", "blocks-settings-integrations-list-row")],
        children,
    )
}

/// 版 A/C/D 共通の行（接続/解除ボタン、展開領域なし）。[`row_with`] の薄い
/// ラッパー。
fn row(item: &Integration) -> Node {
    row_with(item, action_button(item), None)
}

/// 1 グループ分（見出し + `ul` 行リスト）。版 A のみが使う（版 B〜D は単一
/// リストのため本関数を経由しない）。
fn group_section(group: &Group) -> Node {
    div(
        vec![("class", "blocks-settings-integrations-list-group")],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Sm,
                    ..HeadingProps::default()
                },
                vec![("data-blocks-settings-integrations-list-group-title", "")],
                vec![text(group.title)],
            ),
            ul(
                vec![("class", "blocks-settings-integrations-list-list")],
                group.items.iter().map(row).collect(),
            ),
        ],
    )
}

/// 版見出し（H3、`heading` 部品。モジュール doc「グループ/版見出しは
/// `heading` を使う」節参照）。
fn version_title(label: &str) -> Node {
    heading(
        HeadingLevel::H3,
        &HeadingProps {
            size: HeadingSize::Sm,
            ..HeadingProps::default()
        },
        vec![("data-blocks-settings-integrations-list-version-title", "")],
        vec![text(label)],
    )
}

/// 版 B 展開行の API キー欄（`clipboard`、Demo 全体で唯一の
/// `clipboard::root` 呼び出し。モジュール doc「`clipboard` root は版 B の
/// 展開行 1 個に限る」節参照）。
fn api_key_clipboard() -> Node {
    const INPUT_ID: &str = "blocks-settings-integrations-list-api-key";
    div(
        vec![("class", "blocks-settings-integrations-list-key")],
        vec![clipboard::root(
            API_KEY_DEMO,
            false,
            vec![("data-blocks-settings-integrations-list-clipboard", "")],
            vec![
                clipboard::label(false, Some(INPUT_ID), vec![], vec![text("API キー")]),
                clipboard::control(
                    false,
                    vec![],
                    vec![
                        clipboard::input(API_KEY_DEMO, false, vec![("id", INPUT_ID)]),
                        clipboard::trigger(
                            false,
                            vec![],
                            vec![
                                clipboard::indicator(false, false, vec![], vec![text("コピー")]),
                                clipboard::indicator(true, false, vec![], vec![text("コピー済み")]),
                            ],
                        ),
                    ],
                ),
            ],
        )],
    )
}

/// 版 B（R0255）: 有効化スイッチで行を展開し API キー欄を表示する版。
///
/// 先頭行のみ `checked: true` で展開し [`api_key_clipboard`] を表示する。
/// 残り 2 行は `checked: false` で折りたたみのまま（展開/折りたたみの両
/// 状態を同一リスト内に並記する）。スイッチは `readonly` + `disabled` で
/// 静的固定する（モジュール doc「スイッチは readonly + disabled で静的
/// 固定」節参照）。
fn version_switch_keys() -> Node {
    let items = GROUPS[0].items;
    let switch_props = SwitchProps {
        readonly: true,
        disabled: true,
        ..SwitchProps::default()
    };
    let rows: Vec<Node> = items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let checked = i == 0;
            let hidden_input_name = format!("blocks-settings-integrations-list-enabled-{i}");
            let switch_node = switch::root(
                Size::Md,
                ColorPalette::Accent,
                checked,
                &switch_props,
                vec![("data-blocks-settings-integrations-list-switch", "")],
                vec![
                    switch::label(
                        checked,
                        &switch_props,
                        vec![],
                        vec![text(format!("{} を有効化", item.name))],
                    ),
                    switch::hidden_input(&hidden_input_name, "on", checked, &switch_props, vec![]),
                    switch::control(
                        checked,
                        &switch_props,
                        vec![],
                        vec![switch::thumb(checked, &switch_props, vec![], vec![])],
                    ),
                ],
            );
            let expanded = checked.then(api_key_clipboard);
            row_with(item, switch_node, expanded)
        })
        .collect();
    ul(
        vec![("class", "blocks-settings-integrations-list-list")],
        rows,
    )
}

/// 要望済みアプリ 2 件（名前 + 検討状況バッジ）。
const REQUESTED_APPS: &[(&str, &str, BadgeVariant)] = &[
    ("Flow Board", "検討中", BadgeVariant::Subtle),
    ("Signal Mail", "予定", BadgeVariant::Outline),
];

/// 要望済みアプリの小リスト（版 C 要望領域の一部）。
fn requested_apps_list() -> Node {
    ul(
        vec![("class", "blocks-settings-integrations-list-requested-list")],
        REQUESTED_APPS
            .iter()
            .map(|(name, status, variant)| {
                li(
                    vec![],
                    vec![
                        text(*name),
                        badge(
                            &BadgeProps {
                                variant: *variant,
                                ..BadgeProps::default()
                            },
                            vec![(
                                "data-blocks-settings-integrations-list-requested-status",
                                "",
                            )],
                            vec![text(*status)],
                        ),
                    ],
                )
            })
            .collect(),
    )
}

/// 版 C（R0251）: 連携要望の一覧 + 要望送信フォームを末尾に持つ版。
///
/// `<form>` は使わず送信先も持たない静的表示（モジュール doc「要望フォーム
/// は `<form>` を持たず送信先も持たない」節参照）。
fn version_request_form() -> Node {
    const FIELD_ID: &str = "blocks-settings-integrations-list-request-app";
    let field_props = FieldProps {
        id: FIELD_ID,
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: false,
    };
    let group_props = InputGroupProps {
        disabled: false,
        invalid: false,
    };

    let request_area = li(
        vec![("class", "blocks-settings-integrations-list-request")],
        vec![
            requested_apps_list(),
            field::root(
                &FieldRootProps {
                    orientation: FieldOrientation::Vertical,
                },
                &field_props,
                vec![],
                vec![
                    field::label(&field_props, vec![], vec![text("連携してほしいアプリ")]),
                    input_group::root(
                        &group_props,
                        vec![],
                        vec![
                            input::input(
                                &InputProps::default(),
                                &field_props,
                                vec![("type", "text"), ("placeholder", "例: Slack, Notion")],
                            ),
                            input_group::addon(
                                InputGroupAlign::InlineEnd,
                                &group_props,
                                vec![],
                                vec![button(
                                    &ButtonProps::default(),
                                    vec![],
                                    vec![text("要望を送る")],
                                )],
                            ),
                        ],
                    ),
                ],
            ),
        ],
    );

    let mut rows: Vec<Node> = GROUPS[1].items[0..2].iter().map(row).collect();
    rows.push(request_area);
    ul(
        vec![("class", "blocks-settings-integrations-list-list")],
        rows,
    )
}

/// 版 D（R0254）: 末尾に空状態の枠を持つ版。
fn version_empty_state() -> Node {
    let empty_area = li(
        vec![("class", "blocks-settings-integrations-list-empty")],
        vec![empty_state::root(
            &EmptyStateProps {
                variant: EmptyStateVariant::Outline,
                ..EmptyStateProps::default()
            },
            vec![],
            vec![empty_state::content(
                vec![],
                vec![
                    empty_state::title(vec![], vec![text("ほかの連携アプリを探す")]),
                    empty_state::description(
                        vec![],
                        vec![text("カタログから追加の連携アプリを探して接続できます。")],
                    ),
                    empty_state::actions(
                        vec![],
                        vec![button(
                            &ButtonProps {
                                variant: ButtonVariant::Outline,
                                ..ButtonProps::default()
                            },
                            vec![],
                            vec![text("アプリを追加")],
                        )],
                    ),
                ],
            )],
        )],
    );

    let mut rows: Vec<Node> = GROUPS[1].items[1..3].iter().map(row).collect();
    rows.push(empty_area);
    ul(
        vec![("class", "blocks-settings-integrations-list-list")],
        rows,
    )
}

/// `settings-integrations-list` の Demo 本体。呼び出しごとに同一の `Node`
/// を返す純関数。版 A〜D を縦に並記する（モジュール doc「版 A〜D と集約元の
/// 対応」節参照）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-settings-integrations-list-stack")],
        vec![
            version_title("版 A: 個人用・組織用グループ（R0252）"),
            group_section(&GROUPS[0]),
            separator(
                &SeparatorProps::default(),
                vec![("data-blocks-settings-integrations-list-divider", "")],
            ),
            group_section(&GROUPS[1]),
            separator(
                &SeparatorProps::default(),
                vec![("data-blocks-settings-integrations-list-divider", "")],
            ),
            version_title("版 B: 有効化スイッチと API キー欄（R0255）"),
            version_switch_keys(),
            separator(
                &SeparatorProps::default(),
                vec![("data-blocks-settings-integrations-list-divider", "")],
            ),
            version_title("版 C: 連携要望フォーム付き（R0251）"),
            version_request_form(),
            separator(
                &SeparatorProps::default(),
                vec![("data-blocks-settings-integrations-list-divider", "")],
            ),
            version_title("版 D: 空状態付き（R0254）"),
            version_empty_state(),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/settings-integrations-list/",
    title: "settings-integrations-list",
    category: BlockCategory::Settings,
    rust_source: "crates/docs-site/src/blocks/application/settings/settings_integrations_list.rs",
    demo_class: "blocks-settings-integrations-list",
    parts: &[
        Part {
            label: "Badge",
            path: "/themes/badge/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Separator",
            path: "/themes/separator/",
        },
        Part {
            label: "Link",
            path: "/themes/link/",
        },
        Part {
            label: "Image",
            path: "/themes/image/",
        },
        Part {
            label: "Switch",
            path: "/themes/switch/",
        },
        Part {
            label: "Clipboard",
            path: "/themes/clipboard/",
        },
        Part {
            label: "Field",
            path: "/themes/field/",
        },
        Part {
            label: "Input Group",
            path: "/themes/input-group/",
        },
        Part {
            label: "Input",
            path: "/themes/input/",
        },
        Part {
            label: "Empty State",
            path: "/themes/empty-state/",
        },
        Part {
            label: "Heading",
            path: "/themes/heading/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `settings_integrations_list` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節と同型）。
///
/// `list`（`ul`）/`row`（`li`）の 2 セレクタは、素の
/// `.blocks-settings-integrations-list-*` 単一クラス（詳細度 (0,1,0)）のみで
/// 宣言すると、サイト共通 typography（`site_theme.rs` の
/// `.docs-content ul,ol`/`.docs-content li`、いずれも詳細度 (0,1,1)）に負けて
/// リストのビュレット/パディング/`margin-block` が意図通りにならない
/// （PR #3441 Bugbot 指摘）。`.blocks-settings-integrations-list-stack`/
/// `-list` を祖先に持つ子孫セレクタへ書き換えてクラス数を 2 に増やし
/// （詳細度 (0,2,0)）、クラス数比較で `.docs-content ul,ol`/`li` を確実に
/// 上回る（型セレクタの有無に依存しないため、`site.css`/本 CSS の読み込み
/// 順序に左右されない）。`row` はさらに `margin-block: 0` を明示し、
/// `.docs-content li` の `margin-block` を打ち消して行間を `border-top`
/// のみに委ねる（image recipe base への `img[data-scope=...]` 上書きと同じ
/// 「サイト共通スタイルは変更せず block 側で限定的に上回る」判断）。
///
/// グループ見出し・版見出しは `h3` 直書きから `heading` 部品（#2994）へ
/// 移行したため、CSS フックは `class` ではなく `data-*` 属性で渡す
/// （モジュール doc「グループ/版見出しは `heading` を使う」節参照）。
/// `heading` の base 規則（`[data-scope="heading"][data-part="root"]`、
/// 詳細度 (0,2,0)）の `margin: 0` を上書きするため、`.blocks-settings-
/// integrations-list-stack` を祖先に持つ 3 クラス相当の子孫セレクタ
/// （1 class + 2 attribute selector、詳細度 (0,3,0)）を使い、読み込み順序に
/// 依存せず確実に上回る。
const LAYOUT_CSS: &str = "\
.blocks-settings-integrations-list-stack {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n  container-type: inline-size;\n  container-name: blocks-settings-integrations-list;\n}\n\
.blocks-settings-integrations-list-stack [data-scope=\"heading\"][data-blocks-settings-integrations-list-version-title] {\n  margin: 0 0 var(--fandhe-space-3);\n}\n\
.blocks-settings-integrations-list-stack [data-scope=\"heading\"][data-blocks-settings-integrations-list-group-title] {\n  color: var(--fandhe-color-fg-muted);\n  margin: 0 0 var(--fandhe-space-2);\n}\n\
.blocks-settings-integrations-list-stack .blocks-settings-integrations-list-list {\n  list-style: none;\n  margin: 0;\n  padding: 0;\n  border: 1px solid var(--fandhe-color-border);\n  border-radius: var(--fandhe-radius-lg);\n  background: var(--fandhe-color-bg);\n}\n\
.blocks-settings-integrations-list-list .blocks-settings-integrations-list-row {\n  display: grid;\n  grid-template-columns: auto minmax(0, 1fr) auto;\n  grid-template-areas: \"logo body actions\" \"logo key key\";\n  align-items: center;\n  gap: var(--fandhe-space-4);\n  padding: var(--fandhe-space-4);\n  margin-block: 0;\n}\n\
.blocks-settings-integrations-list-row + .blocks-settings-integrations-list-row {\n  border-top: 1px solid var(--fandhe-color-border);\n}\n\
[data-blocks-settings-integrations-list-logo] {\n  grid-area: logo;\n}\n\
img[data-scope=\"image\"][data-blocks-settings-integrations-list-logo] {\n  width: 2.5rem;\n  height: 2.5rem;\n}\n\
.blocks-settings-integrations-list-body {\n  grid-area: body;\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1);\n}\n\
.blocks-settings-integrations-list-name {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n  font-weight: var(--fandhe-font-font-weight-medium);\n}\n\
.blocks-settings-integrations-list-description {\n  color: var(--fandhe-color-fg-muted);\n  font-size: var(--fandhe-font-font-size-sm);\n}\n\
.blocks-settings-integrations-list-actions {\n  grid-area: actions;\n}\n\
.blocks-settings-integrations-list-key {\n  grid-area: key;\n  padding-top: var(--fandhe-space-2);\n}\n\
.blocks-settings-integrations-list-list .blocks-settings-integrations-list-request {\n  padding: var(--fandhe-space-4);\n  border-top: 1px solid var(--fandhe-color-border);\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n  margin-block: 0;\n}\n\
.blocks-settings-integrations-list-requested-list {\n  list-style: none;\n  margin: 0;\n  padding: 0;\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1);\n  font-size: var(--fandhe-font-font-size-sm);\n}\n\
.blocks-settings-integrations-list-requested-list li {\n  display: flex;\n  align-items: center;\n  justify-content: space-between;\n  gap: var(--fandhe-space-2);\n  margin-block: 0;\n}\n\
.blocks-settings-integrations-list-list .blocks-settings-integrations-list-empty {\n  padding: var(--fandhe-space-4);\n  border-top: 1px solid var(--fandhe-color-border);\n  margin-block: 0;\n}\n\
@container blocks-settings-integrations-list (max-width: 40rem) {\n  \
.blocks-settings-integrations-list-list .blocks-settings-integrations-list-row {\n    grid-template-columns: auto minmax(0, 1fr);\n    grid-template-areas: \"logo body\" \"logo actions\" \"logo key\";\n  }\n  \
.blocks-settings-integrations-list-actions {\n    justify-self: start;\n  }\n\
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
            "data-scope=\"badge\"",
            "data-scope=\"button\"",
            "data-scope=\"link\"",
            "data-scope=\"separator\"",
            "data-scope=\"image\"",
            "data-scope=\"switch\"",
            "data-scope=\"clipboard\"",
            "data-scope=\"field\"",
            "data-scope=\"input-group\"",
            "data-scope=\"empty-state\"",
            "data-scope=\"heading\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
    }

    #[test]
    fn demo_has_two_groups_and_thirteen_rows() {
        let html = demo_html();
        assert_eq!(
            html.matches("data-blocks-settings-integrations-list-group-title")
                .count(),
            2
        );
        assert_eq!(
            html.matches("blocks-settings-integrations-list-row")
                .count(),
            // 版 A(6) + 版 B(3) + 版 C(2) + 版 D(2) = 13。各行の class 出現は
            // 1 回だが、隣接セレクタ用の `+` 記述は LAYOUT_CSS 側にのみ
            // 存在するため、demo 側は行要素数と一致する。
            13
        );
        assert!(html.contains("接続済み"));
        assert!(html.contains("未接続"));
        assert!(html.contains("接続"));
        assert!(html.contains("解除"));
    }

    #[test]
    fn no_form_submit_or_dead_links() {
        let html = demo_html();
        assert!(!html.contains("<form"));
        assert!(!html.contains("type=\"submit\""));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("src=\"data:"));
        assert!(!html.contains("<script"));
    }

    #[test]
    fn buttons_use_type_button_only() {
        let html = demo_html();
        // 接続/解除ボタン 10（版 A:6・版 C:2・版 D:2）+ clipboard trigger 1 +
        // 要望送信ボタン 1 + 空状態アクションボタン 1 = 13。
        assert_eq!(html.matches("<button").count(), 13);
        assert_eq!(html.matches("type=\"button\"").count(), 13);
    }

    #[test]
    fn detail_links_are_external_with_noopener() {
        let html = demo_html();
        assert_eq!(
            html.matches("rel=\"noopener noreferrer\"").count(),
            13,
            "each of the 13 rows should have one external detail link"
        );
    }

    #[test]
    fn detail_link_text_matches_destination() {
        // 遷移先が全アプリ共通で本リポジトリの GitHub ページである実態に
        // 合わせ、「詳細を見る」（詳細ページが実在するかのような誤解を招く
        // 文言）ではなく「GitHub で見る」を使う（PR #3441 Codex/Bugbot 指摘）。
        let html = demo_html();
        assert!(!html.contains("詳細を見る"));
        assert_eq!(html.matches("GitHub で見る").count(), 13);
    }

    #[test]
    fn action_buttons_have_app_specific_accessible_name() {
        // 「接続」「解除」だけでは支援技術上どのアプリへの操作か区別できない
        // ため、`aria-label` にアプリ名を含める（PR #3441 Codex 指摘）。
        // 版 B はスイッチ操作のため対象外（接続/解除ボタンを持たない）。
        let html = demo_html();
        for name in [
            "Lattice Notes",
            "Harbor Calendar",
            "Pulse Alerts",
            "Ledger Sync",
            "Beacon Chat",
            "Quarry Storage",
        ] {
            assert!(
                html.contains(&format!("aria-label=\"{name} ")),
                "expected an accessible action-button label mentioning {name}"
            );
        }
    }

    #[test]
    fn switch_rows_show_both_expanded_and_collapsed_states() {
        // 版 B: 先頭行のみ展開（checked）、残り 2 行は折りたたみ
        // （unchecked）のまま同一リスト内に並記する。
        let html = demo_html();
        assert_eq!(
            html.matches("data-part=\"control\" data-state=\"checked\"")
                .count(),
            1
        );
        assert_eq!(
            html.matches("data-part=\"control\" data-state=\"unchecked\"")
                .count(),
            2
        );
        assert_eq!(
            html.matches("data-scope=\"clipboard\" data-part=\"root\"")
                .count(),
            1,
            "clipboard root は 1 root : 1 状態機械契約のため Demo 全体で 1 個のみ"
        );
    }

    #[test]
    fn switch_is_statically_fixed() {
        // readonly + disabled で native トグル操作自体を止める
        // （モジュール doc「スイッチは readonly + disabled で静的固定」節）。
        let html = demo_html();
        assert!(html.contains("data-readonly"));
        for line in html.split("<input") {
            if line.contains("role=\"switch\"") {
                assert!(
                    line.contains("disabled"),
                    "switch hidden_input should carry native disabled: {line}"
                );
            }
        }
    }

    #[test]
    fn request_area_has_labelled_input_and_no_form() {
        let request_html = render(&super::version_request_form());
        assert!(!request_html.contains("<form"));
        assert!(request_html.contains("<label"));
        assert!(
            request_html.contains("for=\"blocks-settings-integrations-list-request-app-control\"")
        );
        assert!(
            request_html.contains("id=\"blocks-settings-integrations-list-request-app-control\"")
        );
        assert!(request_html.trim_end().ends_with("</li></ul>"));
    }

    #[test]
    fn empty_state_is_last_item_of_its_list() {
        let empty_html = render(&super::version_empty_state());
        assert!(empty_html.contains("data-scope=\"empty-state\" data-part=\"root\""));
        assert!(empty_html.trim_end().ends_with("</li></ul>"));
    }

    #[test]
    fn headings_use_pre_styled_heading_not_raw_h3() {
        let html = demo_html();
        assert!(!html.contains("<h3 class=\"blocks-"));
        assert!(html.contains("data-blocks-settings-integrations-list-version-title"));
        assert!(html.contains("data-blocks-settings-integrations-list-group-title"));
    }

    #[test]
    fn layout_css_is_safe_and_stacks_on_narrow_container() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("container-type: inline-size;"));
        assert!(
            LAYOUT_CSS.contains("@container blocks-settings-integrations-list (max-width: 40rem)")
        );
        assert!(LAYOUT_CSS.contains("grid-template-areas: \"logo body\" \"logo actions\""));
    }

    #[test]
    fn layout_css_declares_key_area_on_wide_and_narrow() {
        assert!(LAYOUT_CSS.contains("grid-template-areas: \"logo body actions\" \"logo key key\";"));
        assert!(LAYOUT_CSS
            .contains("grid-template-areas: \"logo body\" \"logo actions\" \"logo key\";"));
        assert!(LAYOUT_CSS.contains(".blocks-settings-integrations-list-key {\n  grid-area: key;"));
    }

    #[test]
    fn narrow_row_selector_specificity_matches_base_rule() {
        // `@container` 内の狭幅用行セレクタが通常ルール
        // `.blocks-settings-integrations-list-list .blocks-settings-integrations-list-row`
        // （詳細度 (0,2,0)）と同じ 2 クラスの子孫セレクタであることの回帰
        // ガード（PR #3441 Codex/Bugbot 指摘）。単一クラス（詳細度 (0,1,0)）
        // では通常ルールに負け、40rem 未満でもレイアウトが切り替わらない。
        assert!(LAYOUT_CSS.contains(
            "@container blocks-settings-integrations-list (max-width: 40rem) {\n  .blocks-settings-integrations-list-list .blocks-settings-integrations-list-row {"
        ));
    }

    #[test]
    fn logo_selector_specificity_beats_image_recipe_base() {
        // `[data-scope="image"][data-part="root"]`（詳細度 (0,2,0)）の
        // `height: auto`/`max-width: 100%` に負けないことの回帰ガード
        // （PR #3441 Bugbot 指摘、イシュー #2931 と同型）。
        assert!(LAYOUT_CSS
            .contains("img[data-scope=\"image\"][data-blocks-settings-integrations-list-logo]"));
    }

    #[test]
    fn typography_selectors_beat_docs_content_specificity() {
        // `.docs-content ul,ol`/`.docs-content li`（いずれも詳細度 (0,1,1)）
        // に負けないよう list/row は 2 クラスの子孫セレクタ（詳細度
        // (0,2,0)）で宣言する回帰ガード（PR #3441 Bugbot 指摘）。単一クラスの
        // 旧セレクタが復活していないことも合わせて固定する。グループ見出し・
        // 版見出しは #2994 で `heading` 部品（`data-scope="heading"`）へ
        // 移行したため、対応するセレクタも子孫 data 属性形へ変わる
        // （詳細度 (0,3,0)、モジュール doc「グループ/版見出しは `heading`
        // を使う」節参照）。
        assert!(LAYOUT_CSS.contains(
            ".blocks-settings-integrations-list-stack [data-scope=\"heading\"][data-blocks-settings-integrations-list-group-title] {"
        ));
        assert!(LAYOUT_CSS.contains(
            ".blocks-settings-integrations-list-stack .blocks-settings-integrations-list-list {"
        ));
        assert!(LAYOUT_CSS.contains(
            ".blocks-settings-integrations-list-list .blocks-settings-integrations-list-row {"
        ));
        assert!(!LAYOUT_CSS.contains("\n.blocks-settings-integrations-list-group-title {"));
        assert!(!LAYOUT_CSS.contains("\n.blocks-settings-integrations-list-list {"));
        assert!(LAYOUT_CSS.contains("margin-block: 0;"));
    }
}
