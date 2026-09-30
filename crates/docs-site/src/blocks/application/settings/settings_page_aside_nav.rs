//! `settings-page-aside-nav` block（イシュー #3001、親 #3000。Application /
//! Settings カテゴリ）。上段にアプリのナビバー、その下にページ見出し
//! （アバター + 氏名/メール + プランバッジ）、下段を「左ナビ + 本文」の
//! 2 カラムへ分割する設定ページの骨格を組み立てる。主参照 R0656（代表構成）
//! を軸に、R0657（区切り線 3 セクション版）・R0661（通知の配信先トグル）・
//! R0662（積み上げ 4 カード + 削除）を集約する予定だが、**この 3 点は本
//! イシューの対象外**（後続 #3002 で追加）。`_/blocks-intake/` の対応
//! ファイルは本イシュー着手時点で本 worktree に存在しないため、原稿・
//! 本コメントには対応表 ID のみを記す（`profile_detail_datalist`
//! 〔イシュー #2937〕と同じ扱い）。
//!
//! # 後続（#3002、対象外）
//!
//! - 通知の配信先トグル（`checkbox`、R0661）
//! - 危険な操作セクション（積み上げ 4 カード + 削除、R0662）
//! - 区切り線 3 セクション版（`separator`、R0657）の並記
//! - 原稿「原案差分メモ」節の仕上げ
//!
//! 上記に使う `checkbox`/`separator` は本 PR の [`BLOCK`] `parts` には
//! 含めない（未使用部品を列挙しない契約、`profile_detail_datalist` と
//! 同型）。
//!
//! # 使用部品
//!
//! `navigation-menu` / `nav-list` / `avatar` / `heading` / `text` /
//! `badge` / `card` / `field` / `input` / `input-group` / `native-select` /
//! `toggle-group` / `table` / `button` の 14 部品を合成する（[`BLOCK`] の
//! `parts` に一致させる契約、`blocks_nav.rs`/`blocks_contract.rs` が
//! 検証する）。
//!
//! # 構成
//!
//! 1. **ナビバー**: `navigation_menu::root`（trigger を持たないリンク
//!    項目のみ、[`header_mega_menu`] の閉じた `trigger` 回避と同じ
//!    理由で `item` + `link` のみ使う）
//! 2. **ページ見出し**: `avatar`（`Size::Lg`）+ 氏名（`heading` H2）/
//!    メール（`text`）+ プランバッジ
//! 3. **左ナビ**: `nav_list`（[`footer_inline_nav::primary_nav`] と同型の
//!    リンク列組み立て。現在項目の伝達は「左ナビの現在項目は
//!    `aria-current` を実在外部リンクへ付けない」節参照）
//! 4. **本文**: プロフィールカード（表示名・ユーザー名・タイムゾーン・
//!    アバター変更）+ プランカード（請求周期 toggle-group + 利用状況
//!    table）
//!
//! [`header_mega_menu`]: super::super::super::marketing::header::header_mega_menu
//! [`footer_inline_nav::primary_nav`]: super::super::super::marketing::footer::footer_inline_nav
//!
//! # `class` と `data-*` の使い分け
//!
//! `card::root`/`field::root`/`button::button`/`avatar::root`/
//! `toggle_group::root_with_props`/`native_select::native_select`/
//! `input_group::root`/`table::root` は `drop_class_attr` により呼び出し側
//! `class` を除去する契約を持つため、これらへの CSS フックは
//! `data-blocks-settings-page-aside-nav-*` 属性で渡す。`navigation_menu` の
//! 各パーツは `class` をそのまま透過する契約
//! （`header_mega_menu` モジュール doc 参照）ため、ナビバー本体には素の
//! `class` を使う。一方 `nav_list::root` は styled 版
//! （`fandhe_frontend_pre_styled_ui::nav_list`）が `drop_class_attr` で
//! 呼び出し側 `class` を除去する契約を持つため（`crate::breadcrumb`/
//! `crate::avatar` と同型の「`root` のみ再定義」構造、`nav_list` モジュール
//! doc「選択的 re-export」節参照）、左ナビの CSS フックは
//! `data-blocks-settings-page-aside-nav-aside` 属性で渡す（`item`/`list`/
//! `link` は headless そのままの再エクスポートで `class` を透過するが、
//! 本 block は `list`/`link` へ `class` を渡さず `data-*` 属性のみで
//! フックしている）。素の `div`/`card::header|body|footer` には
//! `class="blocks-settings-page-aside-nav-*"` を使う（`crate::blocks`
//! モジュール doc の CSS フック規則どおり）。
//!
//! # 狭幅では左ナビを本文の上へ横並びで移す（`@container`）
//!
//! Demo 枠の幅はビューポート幅と一致しないため `@container`
//! （コンテナクエリ）で判定する（`profile_detail_datalist` と同型の
//! パターン）。[`LAYOUT_CSS`] のラッパー `.blocks-settings-page-aside-nav-
//! stack` へ `container-type: inline-size` を宣言し、コンテナ幅が
//! `36rem` 未満のとき 2 カラム grid を 1 列へ切り替え、左ナビ
//! （`nav_list::list`）を縦積みから横並び（折り返し）へ変える。実際の
//! Demo 枠幅（`.docs-content` `max-width: 46rem` から `.blocks-demo` の
//! 左右 padding 各 `1.5rem` を差し引いた約 `43rem` が上限、
//! `grid_list_contact_cards` と同型の算出）を超える閾値（旧 `48rem`）では
//! デモ内で 2 カラム表示に一度も到達できないため（codex/Bugbot 指摘）、
//! 上限を下回る `36rem` へ変更した。
//!
//! # 氏名見出しはサイト共通の `h2` 罫線を持たない（`page_heading` slot）
//!
//! 氏名の `heading(HeadingLevel::H2, ...)` は `.docs-content h2`
//! （`site_theme.rs`）の `border-top`/`padding-top` をそのまま継承すると、
//! 文書の節区切りに見える罫線がページ見出しに紛れ込む
//! （`settings_item_cards` の `section_toolbar` と同型の Bugbot/codex
//! 指摘）。[`LAYOUT_CSS`] は `.blocks-settings-page-aside-nav-identity h2`
//! へ同パターンの `border-top: none; padding-top: 0; letter-spacing:
//! normal;` を当てる。
//!
//! # 左ナビの現在項目は `aria-current` を実在外部リンクへ付けない
//!
//! `nav_list::link` は `current: true` で `aria-current="page"` を付与する
//! が、これは「このリンク先が現在表示中のページである」ことを支援技術へ
//! 伝える契約である。本 Demo の「プロフィール」リンク先は実在する GitHub の
//! プロフィール設定ページであり、閲覧者が実際に見ているのは docs サイトの
//! この block デモページであるため、`aria-current="page"` を付けると遷移先
//! と矛盾した情報を支援技術に与えてしまう（codex 指摘）。このため
//! `nav_list::link` へは常に `current: false` を渡し、視覚的な強調は
//! `data-blocks-settings-page-aside-nav-current` 属性（[`LAYOUT_CSS`]）で
//! 行い、支援技術への現在地の伝達は
//! `fandhe_frontend_pre_styled_ui::visually_hidden::root` によるラベル末尾
//! の非表示テキスト（`sidebar_grouped_nav::current_page_label` と同型の
//! パターン）で行う。
//!
//! # 請求周期トグルは静的表示のみ（`disabled: true`）
//!
//! 無 JS の docs サイトでは押しても選択状態が変わらないため、
//! `ToggleGroupProps.disabled: true` を明示して「操作できない見た目
//! だけの切替」を防ぐ（`toggle_group::root`/`item` モジュール doc の
//! disabled 伝播契約参照）。タイムゾーンの `native_select` は JS 無しでも
//! 通常のネイティブ `<select>` として機能するため `disabled` を付けない。
//!
//! # `<form>` を使わない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない。保存・変更ボタンはすべて既定の `type="button"` のまま
//! 送信先を持たない静的表示のみである。
//!
//! # href の方針
//!
//! ナビバー・左ナビのリンク先は `href="#"` を使わず、ラベルの意味に
//! 対応する実在 URL（[`NAVBAR_LINKS`]/[`ASIDE_NAV_LINKS`]。GitHub の
//! リポジトリ内サブパス・`github.com/settings/*` 等）を個別に張る
//! （全項目を同一 URL（[`REPO`]）へ揃えるとラベルと遷移先が食い違う、
//! Codex レビュー指摘。`footer_inline_nav::NAV_LINKS` と同じ是正方針）。
//!
//! # ダミー素材について
//!
//! 氏名は `crate::blocks::dummy_assets::PERSON_NAMES` の架空セットを使い、
//! メールアドレスは `example.com` ドメインで独自に書く。実在の人物・
//! 企業・PII・クレデンシャルは含まない。アバター画像はビルド時生成の
//! 同梱 SVG（[`dummy_assets::AVATAR_SRC`]）を使う。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::badge::{badge, BadgeProps, BadgeVariant};
use fandhe_frontend_pre_styled_ui::button::{button, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::field::{
    self, FieldIds, FieldOrientation, FieldProps, FieldRootProps,
};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps};
use fandhe_frontend_pre_styled_ui::input::{self, InputProps};
use fandhe_frontend_pre_styled_ui::input_group::{self, InputGroupAlign, InputGroupProps};
use fandhe_frontend_pre_styled_ui::native_select::{self, NativeSelectProps};
use fandhe_frontend_pre_styled_ui::nav_list;
use fandhe_frontend_pre_styled_ui::navigation_menu::{self, NavigationMenuProps, OpenState};
use fandhe_frontend_pre_styled_ui::recipe::Size;
use fandhe_frontend_pre_styled_ui::table::{self, TableProps, TableVariant};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::toggle_group::{self, ToggleGroupProps, ToggleGroupVariant};
use fandhe_frontend_pre_styled_ui::visually_hidden;
use fandhe_frontend_pre_styled_ui::ColorPalette;

/// 実在の GitHub リポジトリへの固定外部 URL（`href="#"` 等の非実在リンクを
/// 避けるための方針、`navbar_two_row`/`footer_inline_nav` と同型）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// アプリのナビバー 4 項目のラベルと遷移先。全項目が `REPO` 直下へ揃うと
/// ラベルと遷移先が食い違う（Codex レビュー指摘、`footer_inline_nav::
/// NAV_LINKS` と同じ是正）ため、ラベルの意味に対応する実在サブパスへ
/// 個別に張る。「設定」は当初リポジトリの `/settings`（管理権限を持つ
/// メンバーのみ閲覧できる）を指していたが、管理権限のない閲覧者には機能
/// しない（Codex レビュー指摘）ため、常にアクセス可能な GitHub 個人設定の
/// ランディングページへ変更した（`ASIDE_NAV_LINKS` の「プロフィール」と
/// 同一 URL になるが、両者は別ナビ内の別項目であり、いずれも「閲覧者
/// 自身が実際に開ける設定系ページ」という意味は保たれる）。
const NAVBAR_LINKS: &[(&str, &str, &str)] = &[
    ("home", "ホーム", REPO),
    (
        "projects",
        "プロジェクト",
        "https://github.com/Fandhe-AI/fandhe-frontend/projects",
    ),
    (
        "reports",
        "レポート",
        "https://github.com/Fandhe-AI/fandhe-frontend/pulse",
    ),
    ("settings", "設定", "https://github.com/settings/profile"),
];

/// 左ナビ（設定項目）5 件のラベルと遷移先。GitHub の実在する設定系ページ
/// （`github.com/settings/*`）へラベルの意味を合わせる。「危険な操作」は
/// 当初リポジトリの `/settings`（管理権限を持つメンバーのみ閲覧できる）を
/// 指していたが、管理権限のない閲覧者には機能しない（Codex レビュー指摘）
/// ため、常にアクセス可能な GitHub 個人設定のランディングページへ変更した
/// （「プロフィール」と同一 URL になる。危険な操作に対応する個人設定の
/// 個別サブページ名は GitHub 側の変更で流動的なため、URL の一意性よりも
/// 常時到達可能性を優先する）。現在項目（「プロフィール」）に
/// `aria-current` を付けない理由・視覚的な現在地強調の方法は「左ナビの
/// 現在項目は `aria-current` を実在外部リンクへ付けない」節参照。
const ASIDE_NAV_LINKS: &[(&str, &str, bool)] = &[
    ("プロフィール", "https://github.com/settings/profile", true),
    ("アカウント", "https://github.com/settings/security", false),
    ("プラン", "https://github.com/settings/billing", false),
    ("通知", "https://github.com/settings/notifications", false),
    ("危険な操作", "https://github.com/settings/profile", false),
];

/// フィールド id の共通接頭辞を付ける小さなヘルパ（綴り間違い防止、
/// `contact_centered_form::field_id` と同型）。
fn field_id(suffix: &str) -> String {
    format!("settings-page-aside-nav-{suffix}")
}

/// 縦積み（label 上・control 下）の共通 orientation。
fn vertical() -> FieldRootProps {
    FieldRootProps {
        orientation: FieldOrientation::Vertical,
    }
}

/// ユーザー名 `input_group` の共通 props（無効化・不正値なし）。
/// [`InputGroupProps`] は `Default` を実装しないため、明示構築する
/// （`hero_install_command::instance_b` と同型の判断）。
fn username_group_props() -> InputGroupProps {
    InputGroupProps {
        disabled: false,
        invalid: false,
    }
}

/// アプリのナビバー（`navigation_menu`、trigger を持たないリンク項目
/// 4 件のみ。モジュール doc「構成」節 1.）。
fn app_navbar() -> Node {
    let props = NavigationMenuProps::default();
    let children: Vec<Node> = NAVBAR_LINKS
        .iter()
        .map(|(value, label, href)| {
            navigation_menu::item(
                OpenState::Closed,
                false,
                &props,
                value,
                vec![],
                vec![navigation_menu::link(
                    href,
                    false,
                    vec![],
                    vec![text(*label)],
                )],
            )
        })
        .collect();
    navigation_menu::root(
        &props,
        "アプリケーション",
        vec![("class", "blocks-settings-page-aside-nav-navbar")],
        vec![navigation_menu::list(&props, vec![], children)],
    )
}

/// ページ見出し（アバター + 氏名/メール + プランバッジ、モジュール doc
/// 「構成」節 2.）。
fn page_heading(name: &'static str, email: &'static str) -> Node {
    div(
        vec![("class", "blocks-settings-page-aside-nav-heading")],
        vec![
            avatar::root(
                &AvatarProps {
                    size: Size::Lg,
                    ..AvatarProps::default()
                },
                vec![],
                vec![
                    avatar::image(ImageStatus::Loaded, dummy_assets::AVATAR_SRC, name, vec![]),
                    avatar::fallback(
                        ImageStatus::Loaded,
                        vec![],
                        vec![text(name.chars().take(1).collect::<String>())],
                    ),
                ],
            ),
            div(
                vec![("class", "blocks-settings-page-aside-nav-identity")],
                vec![
                    heading(
                        HeadingLevel::H2,
                        &HeadingProps::default(),
                        vec![],
                        vec![text(name)],
                    ),
                    styled_text::text(
                        &TextProps {
                            variant: TextVariant::Muted,
                            size: TextSize::Sm,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text(email)],
                    ),
                ],
            ),
            badge(
                &BadgeProps {
                    variant: BadgeVariant::Subtle,
                    ..BadgeProps::default()
                },
                vec![],
                vec![text("Pro プラン")],
            ),
        ],
    )
}

/// 左ナビ（設定項目、モジュール doc「構成」節 3.）。「プロフィール」だけ
/// 現在項目とする（`footer_inline_nav::primary_nav` と同型のリンク列
/// 組み立て）が、`nav_list::link` の `current` 引数には常に `false` を渡し
/// `aria-current` は付けない。視覚的な強調は
/// `data-blocks-settings-page-aside-nav-current` 属性（[`LAYOUT_CSS`]）、
/// 支援技術への伝達は [`current_item_label`] の非表示テキストで行う
/// （理由は「左ナビの現在項目は `aria-current` を実在外部リンクへ付けない」
/// 節参照）。
fn aside_nav() -> Node {
    let children: Vec<Node> = ASIDE_NAV_LINKS
        .iter()
        .map(|(label, href, current)| {
            let link_attrs = if *current {
                vec![("data-blocks-settings-page-aside-nav-current", "")]
            } else {
                vec![]
            };
            nav_list::item(
                vec![],
                vec![nav_list::link(
                    href,
                    false,
                    link_attrs,
                    current_item_label(label, *current),
                )],
            )
        })
        .collect();
    nav_list::root(
        "設定項目",
        vec![("data-blocks-settings-page-aside-nav-aside", "")],
        vec![nav_list::list(vec![], children)],
    )
}

/// `aside_nav` 各リンクのラベル組み立て。`current` のときのみ
/// visually-hidden な "(current)" をラベル末尾へ加え、`aria-current` を
/// 使わずに現在地を支援技術へ伝える（`sidebar_grouped_nav::
/// current_page_label` と同型のパターン）。
fn current_item_label(label: &'static str, current: bool) -> Vec<Node> {
    let mut children = vec![text(label)];
    if current {
        children.push(visually_hidden::root(vec![], vec![text(" (current)")]));
    }
    children
}

/// プロフィールカード（表示名・ユーザー名・タイムゾーン・アバター変更、
/// モジュール doc「構成」節 4.）。
fn profile_card() -> Node {
    let display_name_id = field_id("display-name");
    let username_id = field_id("username");
    let timezone_id = field_id("timezone");
    let display_name = FieldProps {
        id: &display_name_id,
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: true,
        readonly: false,
        has_helper_text: false,
    };
    let username = FieldProps {
        id: &username_id,
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: true,
        readonly: false,
        has_helper_text: false,
    };
    let timezone = FieldProps {
        id: &timezone_id,
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: false,
    };

    card::root(
        CardProps::default(),
        vec![("data-blocks-settings-page-aside-nav-card", "")],
        vec![
            card::header(
                vec![],
                vec![
                    card::title(vec![], vec![text("プロフィール")]),
                    card::description(
                        vec![],
                        vec![text("表示名・ユーザー名・タイムゾーンを設定します。")],
                    ),
                ],
            ),
            card::body(
                vec![],
                vec![div(
                    vec![("class", "blocks-settings-page-aside-nav-fields")],
                    vec![
                        field::root(
                            &vertical(),
                            &display_name,
                            vec![],
                            vec![
                                field::label(&display_name, vec![], vec![text("表示名")]),
                                input::input(
                                    &InputProps::default(),
                                    &display_name,
                                    vec![("type", "text"), ("value", "Kwame Boateng")],
                                ),
                            ],
                        ),
                        field::root(
                            &vertical(),
                            &username,
                            vec![],
                            vec![
                                field::label(&username, vec![], vec![text("ユーザー名")]),
                                input_group::root(
                                    &username_group_props(),
                                    vec![],
                                    vec![
                                        input_group::addon(
                                            InputGroupAlign::InlineStart,
                                            &username_group_props(),
                                            vec![],
                                            vec![input_group::text(vec![], vec![text("@")])],
                                        ),
                                        input::input(
                                            &InputProps::default(),
                                            &username,
                                            vec![("type", "text"), ("value", "kwame-boateng")],
                                        ),
                                    ],
                                ),
                            ],
                        ),
                        field::root(
                            &vertical(),
                            &timezone,
                            vec![],
                            vec![
                                field::label(&timezone, vec![], vec![text("タイムゾーン")]),
                                native_select::native_select(
                                    &NativeSelectProps::default(),
                                    &timezone,
                                    vec![],
                                    vec![
                                        // Asia/Tokyo は夏時間を持たないため固定オフセット表記で
                                        // 通年正しいが、Europe/London・America/New_York は夏時間で
                                        // オフセットが変動する（London: UTC+0/+1、New_York:
                                        // UTC-5/-4）ため固定オフセットを付けない
                                        // （Codex レビュー指摘: 通年不正確な固定表記の是正）。
                                        select_option("Asia/Tokyo", "Asia/Tokyo（UTC+9）"),
                                        select_option("Europe/London", "Europe/London"),
                                        select_option("America/New_York", "America/New_York"),
                                    ],
                                ),
                            ],
                        ),
                        div(
                            vec![("class", "blocks-settings-page-aside-nav-avatar-row")],
                            vec![
                                avatar::root(
                                    &AvatarProps {
                                        size: Size::Sm,
                                        ..AvatarProps::default()
                                    },
                                    vec![],
                                    vec![
                                        avatar::image(
                                            ImageStatus::Loaded,
                                            dummy_assets::AVATAR_SRC,
                                            "Kwame Boateng",
                                            vec![],
                                        ),
                                        avatar::fallback(
                                            ImageStatus::Loaded,
                                            vec![],
                                            vec![text("K")],
                                        ),
                                    ],
                                ),
                                button(
                                    &ButtonProps {
                                        variant: ButtonVariant::Outline,
                                        size: Size::Sm,
                                        ..ButtonProps::default()
                                    },
                                    vec![],
                                    vec![text("画像を変更")],
                                ),
                            ],
                        ),
                    ],
                )],
            ),
            card::footer(
                vec![("class", "blocks-settings-page-aside-nav-actions")],
                vec![
                    button(&ButtonProps::default(), vec![], vec![text("保存")]),
                    button(
                        &ButtonProps {
                            variant: ButtonVariant::Ghost,
                            ..ButtonProps::default()
                        },
                        vec![],
                        vec![text("キャンセル")],
                    ),
                ],
            ),
        ],
    )
}

/// `<option>` 1 件（`native_select` の子 `Node`）。
fn select_option(value: &str, label: &str) -> Node {
    fandhe_frontend_core::el("option", vec![("value", value)], vec![text(label)])
}

/// プランカード（請求周期 toggle-group + 利用状況 table、モジュール doc
/// 「構成」節 4.）。
fn plan_card() -> Node {
    let toggle_props = ToggleGroupProps {
        disabled: true,
        ..ToggleGroupProps::default()
    };
    let toggle = toggle_group::root_with_props(
        Size::Sm,
        ToggleGroupVariant::Outline,
        ColorPalette::Accent,
        &toggle_props,
        None,
        vec![("aria-label", "請求周期")],
        vec![
            toggle_group::item(
                &toggle_props,
                true,
                false,
                false,
                "monthly",
                vec![],
                vec![text("月払い")],
            ),
            toggle_group::item(
                &toggle_props,
                false,
                false,
                false,
                "yearly",
                vec![],
                vec![text("年払い")],
            ),
        ],
    );

    let usage_table = table::root(
        TableProps {
            variant: TableVariant::Outline,
            ..TableProps::default()
        },
        vec![("data-blocks-settings-page-aside-nav-table", "")],
        vec![
            table::caption(vec![], vec![text("利用状況")]),
            table::header(
                vec![],
                vec![table::row(
                    vec![],
                    vec![
                        table::column_header(vec![], vec![text("項目")]),
                        table::column_header(vec![], vec![text("現在")]),
                        table::column_header(vec![], vec![text("上限")]),
                    ],
                )],
            ),
            table::body(
                vec![],
                vec![
                    usage_row("プロジェクト数", "12", "50"),
                    usage_row("メンバー数", "6", "20"),
                    usage_row("ストレージ", "18 GB", "100 GB"),
                ],
            ),
        ],
    );

    card::root(
        CardProps::default(),
        vec![("data-blocks-settings-page-aside-nav-card", "")],
        vec![
            card::header(
                vec![("data-has-action", "")],
                vec![
                    card::title(vec![], vec![text("プラン")]),
                    card::action(
                        vec![],
                        vec![badge(
                            &BadgeProps {
                                variant: BadgeVariant::Outline,
                                ..BadgeProps::default()
                            },
                            vec![],
                            vec![text("現在: Pro")],
                        )],
                    ),
                ],
            ),
            card::body(
                vec![],
                vec![div(
                    vec![("class", "blocks-settings-page-aside-nav-fields")],
                    vec![toggle, usage_table],
                )],
            ),
            card::footer(
                vec![("class", "blocks-settings-page-aside-nav-actions")],
                vec![button(
                    &ButtonProps::default(),
                    vec![],
                    vec![text("プランを変更")],
                )],
            ),
        ],
    )
}

/// 利用状況 table の 1 行（項目名 + 現在値 + 上限値）。
fn usage_row(label: &'static str, current: &'static str, limit: &'static str) -> Node {
    table::row(
        vec![],
        vec![
            table::row_header(vec![], vec![text(label)]),
            table::cell(vec![], vec![text(current)]),
            table::cell(vec![], vec![text(limit)]),
        ],
    )
}

/// `settings-page-aside-nav` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-settings-page-aside-nav-stack")],
        vec![
            app_navbar(),
            page_heading("Kwame Boateng", "kwame.boateng@example.com"),
            div(
                vec![("class", "blocks-settings-page-aside-nav-layout")],
                vec![
                    aside_nav(),
                    div(
                        vec![("class", "blocks-settings-page-aside-nav-main")],
                        vec![profile_card(), plan_card()],
                    ),
                ],
            ),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/settings-page-aside-nav/",
    title: "settings-page-aside-nav",
    category: BlockCategory::Settings,
    rust_source: "crates/docs-site/src/blocks/application/settings/settings_page_aside_nav.rs",
    demo_class: "blocks-settings-page-aside-nav",
    parts: &[
        Part {
            label: "Navigation Menu",
            path: "/themes/navigation-menu/",
        },
        Part {
            label: "Nav List",
            path: "/themes/nav-list/",
        },
        Part {
            label: "Avatar",
            path: "/themes/avatar/",
        },
        Part {
            label: "Heading",
            path: "/themes/heading/",
        },
        Part {
            label: "Text",
            path: "/themes/text/",
        },
        Part {
            label: "Badge",
            path: "/themes/badge/",
        },
        Part {
            label: "Card",
            path: "/themes/card/",
        },
        Part {
            label: "Field",
            path: "/themes/field/",
        },
        Part {
            label: "Input",
            path: "/themes/input/",
        },
        Part {
            label: "Input Group",
            path: "/themes/input-group/",
        },
        Part {
            label: "Native Select",
            path: "/themes/native-select/",
        },
        Part {
            label: "Toggle Group",
            path: "/themes/toggle-group/",
        },
        Part {
            label: "Table",
            path: "/themes/table/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `settings_page_aside_nav` 固有のレイアウト規則（`crate::blocks::mod`
/// モジュール doc「block 固有 CSS の置き場」節と同型）。
const LAYOUT_CSS: &str = "\
.blocks-settings-page-aside-nav-stack {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n  container-type: inline-size;\n  container-name: blocks-settings-page-aside-nav;\n}\n\
.blocks-settings-page-aside-nav-navbar {\n  display: flex;\n}\n\
.blocks-settings-page-aside-nav-navbar [data-scope=\"navigation-menu\"][data-part=\"list\"] {\n  display: flex;\n  flex-wrap: wrap;\n  gap: var(--fandhe-space-4);\n  list-style: none;\n  margin: 0;\n  padding: 0;\n}\n\
.blocks-settings-page-aside-nav-heading {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-4);\n  flex-wrap: wrap;\n}\n\
.blocks-settings-page-aside-nav-identity {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1);\n  min-width: 0;\n}\n\
.blocks-settings-page-aside-nav-identity h2 {\n  border-top: none;\n  padding-top: 0;\n  letter-spacing: normal;\n}\n\
.blocks-settings-page-aside-nav-layout {\n  display: grid;\n  grid-template-columns: 14rem minmax(0, 1fr);\n  gap: var(--fandhe-space-8);\n  align-items: start;\n}\n\
[data-blocks-settings-page-aside-nav-aside] [data-scope=\"nav-list\"][data-part=\"list\"] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1);\n}\n\
[data-blocks-settings-page-aside-nav-aside] [data-scope=\"nav-list\"][data-part=\"link\"][data-blocks-settings-page-aside-nav-current] {\n  color: var(--fandhe-color-accent, var(--fandhe-color-fg));\n  font-weight: var(--fandhe-font-font-weight-medium);\n}\n\
.blocks-settings-page-aside-nav-main {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n  min-width: 0;\n}\n\
.blocks-settings-page-aside-nav-fields {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-settings-page-aside-nav-avatar-row {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-settings-page-aside-nav-actions {\n  display: flex;\n  gap: var(--fandhe-space-2);\n  flex-wrap: wrap;\n}\n\
@container blocks-settings-page-aside-nav (max-width: 36rem) {\n  \
.blocks-settings-page-aside-nav-layout {\n    grid-template-columns: minmax(0, 1fr);\n  }\n  \
[data-blocks-settings-page-aside-nav-aside] [data-scope=\"nav-list\"][data-part=\"list\"] {\n    flex-direction: row;\n    flex-wrap: wrap;\n    gap: var(--fandhe-space-2);\n  }\n\
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
            "data-scope=\"navigation-menu\"",
            "data-scope=\"nav-list\"",
            "data-scope=\"avatar\"",
            "data-scope=\"badge\"",
            "data-scope=\"card\"",
            "data-scope=\"field\"",
            "data-scope=\"input-group\"",
            "data-scope=\"toggle-group\"",
            "data-scope=\"table\"",
            "data-scope=\"button\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
        assert_eq!(html.matches("<h2").count(), 1);
        assert_eq!(html.matches("<h3").count(), 2);
    }

    #[test]
    fn no_form_submit_or_dead_links() {
        let html = demo_html();
        assert!(!html.contains("<form"));
        assert!(!html.contains("type=\"submit\""));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("src=\"data:"));
        assert!(!html.contains("<script"));
        assert!(html.contains("../../assets/blocks-demo-avatar.svg"));
    }

    #[test]
    fn layout_css_is_safe_and_stacks_on_narrow_container() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("container-type: inline-size;"));
        assert!(LAYOUT_CSS.contains("@container blocks-settings-page-aside-nav (max-width: 36rem)"));
    }

    #[test]
    fn page_heading_resets_docs_content_h2_rule() {
        assert!(LAYOUT_CSS.contains(".blocks-settings-page-aside-nav-identity h2"));
        assert!(LAYOUT_CSS.contains("border-top: none;"));
    }

    #[test]
    fn aside_nav_marks_current_item_without_aria_current() {
        let html = demo_html();
        // `aria-current` は実在外部リンクへ付けない（codex 指摘対応）。現在
        // 項目の伝達は視覚的な `data-*` 属性 + 非表示テキストで行う。
        assert!(!html.contains("aria-current=\"page\""));
        assert_eq!(
            html.matches("data-blocks-settings-page-aside-nav-current")
                .count(),
            1
        );
        assert!(html.contains("(current)"));
    }

    #[test]
    fn toggle_group_is_static() {
        let html = demo_html();
        assert!(html.contains("data-scope=\"toggle-group\" data-part=\"root\""));
        assert!(html.contains("data-disabled=\"\""));
    }
}
