//! `settings-page-aside-nav` block（イシュー #3001/#3002、親 #3000。
//! Application / Settings カテゴリ）。上段にアプリのナビバー、その下に
//! ページ見出し（アバター + 氏名/メール + プランバッジ）、下段を
//! 「左ナビ + 本文」の 2 カラムへ分割する設定ページの骨格を組み立てる。
//! `_/blocks-intake/` の対応ファイルは本イシュー着手時点で本 worktree に
//! 存在しないため、原稿・本コメントには対応表 ID のみを記す
//! （`profile_detail_datalist`〔イシュー #2937〕と同じ扱い）。
//!
//! # 版 A / 版 B と集約元の対応
//!
//! Demo は単一 [`demo`] 内に版 A・版 B を縦に並記する
//! （`settings_integrations_list`〔#2994〕と同型のパターン、`heading`
//! H2 + [`version_title`] の `data-blocks-settings-page-aside-nav-
//! version-title` 属性で TOC 混入を回避する）。版見出しを H2 にする理由は
//! 「版見出しは H2（子見出しとの階層整合）」節参照。
//!
//! - **版 A（R0656 + R0661 + R0662）**: 「構成」節どおりの左ナビ + 本文
//!   2 カラムの本文を、プロフィール / プラン / 通知 / 危険な操作の
//!   4 カードへ拡張する（積み上げ 4 カード + 削除、R0662）。通知カードは
//!   配信先 `checkbox` 3 件を checked 2 + unchecked 1 の混在で静的表示する
//!   （配信先トグル、R0661）。
//! - **版 B（R0657）**: 版 A 直下に区切り線 3 セクション版を並記する。
//!   カード枠を持たず、プロフィール / 通知 / 危険な操作の 3 セクションを
//!   `separator::separator` 2 本で区切った本文のみで構成する（左ナビは
//!   再掲しない。`nav` ランドマーク `aria-label="設定項目"` の重複と
//!   現在項目マーカーの二重化を避けるため）。各セクションの中身
//!   （[`profile_fields`]/[`notification_fields`]/[`danger_body`]）は
//!   版 A と共有し、`id_prefix` で `id` 衝突を避ける（「id の一意化」
//!   節参照）。
//!
//! # id の一意化（版 A / 版 B で同じフィールドを再利用するため）
//!
//! [`profile_fields`]/[`notification_checkbox`] は `id_prefix: &str`
//! を取り、フィールド `id`（[`field_id`]）・checkbox `id`
//! （[`notification_checkbox`] 内で組み立てる）へ `-a-`/`-b-` を埋め込んで
//! 版 A/版 B 間の `id` 重複を防ぐ（`crates/docs-site/tests/
//! blocks_contract.rs` の id 重複禁止検査対策、`settings_notification_matrix`
//! の「幅広/狭幅で checkbox が重複する」ケースと同型の判断）。
//!
//! # 通知 checkbox をネイティブ disabled にする理由
//!
//! `checkbox::hidden_input` は有効なネイティブ `<input>` であり、`disabled`
//! を渡さない構成では docs サイトが JS ハイドレーションを行わなくても
//! ラベルクリック・キーボード操作でブラウザが `checked` をネイティブに
//! 切り替えてしまう一方、`control`/`indicator` の見た目（`data-state`）は
//! SSR 時の `checked` 引数から固定生成されるため追従しない
//! （`settings_notification_matrix`/`form_layout_stacked` と同型の判断）。
//! [`CheckboxProps`] の `disabled: true` を全 checkbox で共有し、ネイティブ
//! `disabled` 属性で操作を不能にして状態が二度と変化しないことを構造的に
//! 保証する。既定の `opacity: 0.5` + `cursor: not-allowed`
//! （`disabled_declarations()`）はそのまま残し、静的デモでも無効状態と
//! 分かる視覚表現を保つ（中和すると操作可能に見えるがクリックしても
//! 反応しない不整合になる、Codex レビュー指摘対応）。
//!
//! # `separator` は版 B のみで使う
//!
//! [`variant_b_main`] のみが `separator::separator`（既定 `Horizontal`/
//! `Solid`）を 2 本使い、3 セクションを区切る。版 A はカード枠
//! （`card::root`）が区切りの役目を兼ねるため `separator` を使わない。
//!
//! # 版見出しは H2（子見出しとの階層整合）
//!
//! [`version_title`] は H2 を使う（`settings_integrations_list::
//! version_title` は H3 だが、あちらは版の下に `card::title` のような
//! 固定見出しを持たないため階層問題が生じない。本 block は事情が異なる）。
//! 版 A の各カード見出しは `card::title`（`pre-styled-ui` 側で `<h3>` 固定、
//! `crate::card` モジュール doc 参照）、版 B の各セクション見出しは
//! [`plain_section`] 内の `heading(HeadingLevel::H3, ...)` であり、いずれも
//! 版見出し直下の子見出しである。版見出しを H3 のままにすると子見出しと
//! 同じ H3 になり、版という区切りが見出し階層に表れない（Codex レビュー
//! 指摘）。版見出しを H2 にすることで両版の子見出し（H3）が正しく 1 段
//! 下の階層に揃う。
//!
//! # 使用部品
//!
//! `navigation-menu` / `nav-list` / `avatar` / `heading` / `text` /
//! `badge` / `card` / `field` / `input` / `input-group` / `native-select` /
//! `toggle-group` / `table` / `button` / `checkbox` / `separator` の
//! 16 部品を合成する（[`BLOCK`] の `parts` に一致させる契約、
//! `blocks_nav.rs`/`blocks_contract.rs` が検証する）。
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
//! 4. **本文（版 A）**: プロフィールカード（表示名・ユーザー名・
//!    タイムゾーン・アバター変更）+ プランカード（請求周期 toggle-group +
//!    利用状況 table）+ 通知カード（配信先 `checkbox` 3 件）+
//!    危険な操作カード（説明 + 削除ボタン）
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
//! と矛盾した情報を支援技術に与えてしまう（codex 指摘）。この矛盾は
//! `aria-current` 属性そのものに限らず、`sidebar_grouped_nav::
//! current_page_label` と同型の非表示テキスト（"(current)"）で代替しても
//! 同じ支援技術への誤伝達になる（2 度目の codex 指摘: デモページ閲覧中に
//! 外部リンクを current item として "(current)" を付与するのは誤り）。
//! このため `nav_list::link` へは常に `current: false` を渡し、支援技術
//! 向けの現在地ラベルは一切付けない。「プロフィール」を強調表示したいだけ
//! の視覚上の意図は `data-blocks-settings-page-aside-nav-current` 属性
//! （[`LAYOUT_CSS`]）のみで表現する（`aria-*`/非表示テキストいずれも伴わ
//! ない、見た目だけの装飾。支援技術には他の項目と等価なリンクとして伝わる）。
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
use fandhe_frontend_pre_styled_ui::checkbox::{self, CheckboxProps, CheckedState};
use fandhe_frontend_pre_styled_ui::field::{
    self, FieldIds, FieldOrientation, FieldProps, FieldRootProps,
};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::input::{self, InputProps};
use fandhe_frontend_pre_styled_ui::input_group::{self, InputGroupAlign, InputGroupProps};
use fandhe_frontend_pre_styled_ui::native_select::{self, NativeSelectProps};
use fandhe_frontend_pre_styled_ui::nav_list;
use fandhe_frontend_pre_styled_ui::navigation_menu::{self, NavigationMenuProps, OpenState};
use fandhe_frontend_pre_styled_ui::recipe::Size;
use fandhe_frontend_pre_styled_ui::separator::{separator, SeparatorProps};
use fandhe_frontend_pre_styled_ui::table::{self, TableProps, TableVariant};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::toggle_group::{self, ToggleGroupProps, ToggleGroupVariant};
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
/// ため、GitHub 個人アカウント設定のうち誰でも自分のアカウントで開ける
/// `/settings/admin`（Account ページ、末尾に Delete account = Danger Zone
/// を含む）へ変更した。「プロフィール」の `/settings/profile` とは別の
/// 実在 URL であり、ラベルと遷移先が対応する（2 度目の Codex レビュー
/// 指摘: 「プロフィール」と同一 URL のままではラベルに対応する遷移先が
/// 無い）。現在項目（「プロフィール」）に `aria-current` を付けない理由・
/// 視覚的な現在地強調の方法は「左ナビの現在項目は `aria-current` を実在
/// 外部リンクへ付けない」節参照。
const ASIDE_NAV_LINKS: &[(&str, &str, bool)] = &[
    ("プロフィール", "https://github.com/settings/profile", true),
    ("アカウント", "https://github.com/settings/security", false),
    ("プラン", "https://github.com/settings/billing", false),
    ("通知", "https://github.com/settings/notifications", false),
    ("危険な操作", "https://github.com/settings/admin", false),
];

/// フィールド id の共通接頭辞を付ける小さなヘルパ（綴り間違い防止、
/// `contact_centered_form::field_id` と同型）。`id_prefix` は版 A/版 B の
/// `id` 衝突を避けるための追加接頭辞（モジュール doc「id の一意化」節
/// 参照）。
fn field_id(id_prefix: &str, suffix: &str) -> String {
    format!("settings-page-aside-nav-{id_prefix}-{suffix}")
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
/// `aria-current` は付けない。ここでの `highlighted` は「プロフィール」を
/// 視覚的に目立たせるだけの装飾フラグであり、`data-blocks-settings-page-
/// aside-nav-current` 属性（[`LAYOUT_CSS`]）のみに反映する。支援技術向け
/// の現在地ラベルは付けない（理由は「左ナビの現在項目は `aria-current` を
/// 実在外部リンクへ付けない」節参照。非表示テキストによる代替も同節の
/// 2 度目の指摘により不採用）。
fn aside_nav() -> Node {
    let children: Vec<Node> = ASIDE_NAV_LINKS
        .iter()
        .map(|(label, href, highlighted)| {
            let link_attrs = if *highlighted {
                vec![("data-blocks-settings-page-aside-nav-current", "")]
            } else {
                vec![]
            };
            nav_list::item(
                vec![],
                vec![nav_list::link(href, false, link_attrs, vec![text(*label)])],
            )
        })
        .collect();
    nav_list::root(
        "設定項目",
        vec![("data-blocks-settings-page-aside-nav-aside", "")],
        vec![nav_list::list(vec![], children)],
    )
}

/// プロフィールフィールド一式（表示名・ユーザー名・タイムゾーン・
/// アバター変更、モジュール doc「構成」節 4.）。版 A（`profile_card`）・
/// 版 B（[`variant_b_main`]）の双方から `id_prefix` を変えて呼ばれる
/// （モジュール doc「id の一意化」節参照）。
fn profile_fields(id_prefix: &str) -> Vec<Node> {
    let display_name_id = field_id(id_prefix, "display-name");
    let username_id = field_id(id_prefix, "username");
    let timezone_id = field_id(id_prefix, "timezone");
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
                        avatar::fallback(ImageStatus::Loaded, vec![], vec![text("K")]),
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
    ]
}

/// プロフィールカード（版 A、モジュール doc「構成」節 4.）。
fn profile_card() -> Node {
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
                    profile_fields("a"),
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

/// 版見出し（H2、`heading` 部品。モジュール doc「版見出しは H2（子見出し
/// との階層整合）」節参照。TOC 混入回避のため専用 `data-*` 属性を付ける）。
fn version_title(label: &str) -> Node {
    heading(
        HeadingLevel::H2,
        &HeadingProps {
            size: HeadingSize::Sm,
            ..HeadingProps::default()
        },
        vec![("data-blocks-settings-page-aside-nav-version-title", "")],
        vec![text(label)],
    )
}

/// 通知配信先 checkbox 1 件（モジュール doc「通知 checkbox をネイティブ
/// disabled にする理由」節参照）。`id_prefix` は版 A/版 B の `id` 一意化用。
fn notification_checkbox(
    id_prefix: &str,
    suffix: &str,
    label: &'static str,
    checked: bool,
) -> Node {
    let props = CheckboxProps {
        checked: if checked {
            CheckedState::Checked
        } else {
            CheckedState::Unchecked
        },
        disabled: true,
        ..CheckboxProps::default()
    };
    let id = format!("settings-page-aside-nav-{id_prefix}-notification-{suffix}");
    checkbox::root(
        Size::Md,
        ColorPalette::Accent,
        &props,
        vec![("data-blocks-settings-page-aside-nav-checkbox", "")],
        vec![
            checkbox::hidden_input(
                &props,
                "notification-channel",
                "on",
                vec![("id", id.as_str())],
            ),
            checkbox::control(
                &props,
                vec![],
                vec![checkbox::indicator(&props, vec![], vec![])],
            ),
            checkbox::label(&props, vec![], vec![text(label)]),
        ],
    )
}

/// 通知の配信先 checkbox 3 件（メール・ブラウザ通知はチェック済み、
/// モバイルプッシュは未チェック。状態並記の意図は `checked: bool` 引数に
/// 明示する）。`role="group"` + `aria-label` で checkbox 群をひとまとまりの
/// フォームコントロール群として支援技術へ伝える（`fieldset` は本 block の
/// 使用部品外のため使わない）。
fn notification_fields(id_prefix: &str) -> Node {
    div(
        vec![
            ("class", "blocks-settings-page-aside-nav-checkboxes"),
            ("role", "group"),
            ("aria-label", "通知の配信先"),
        ],
        vec![
            notification_checkbox(id_prefix, "email", "メール", true),
            notification_checkbox(id_prefix, "browser", "ブラウザ通知", true),
            notification_checkbox(id_prefix, "push", "モバイルプッシュ", false),
        ],
    )
}

/// 通知カード（版 A、モジュール doc「構成」節 4.）。
fn notification_card() -> Node {
    card::root(
        CardProps::default(),
        vec![("data-blocks-settings-page-aside-nav-card", "")],
        vec![
            card::header(
                vec![],
                vec![
                    card::title(vec![], vec![text("通知")]),
                    card::description(vec![], vec![text("通知を受け取る配信先を選択します。")]),
                ],
            ),
            card::body(vec![], vec![notification_fields("a")]),
            card::footer(
                vec![("class", "blocks-settings-page-aside-nav-actions")],
                vec![button(&ButtonProps::default(), vec![], vec![text("保存")])],
            ),
        ],
    )
}

/// 危険な操作の説明 + 削除ボタン（版 A・版 B 共有。`id` を持たないため
/// `id_prefix` は不要）。削除ボタンは `ButtonVariant::Outline` +
/// `ColorPalette::Danger` で危険操作であることを視覚的に示す。
fn danger_body() -> Vec<Node> {
    vec![
        styled_text::text(
            &TextProps {
                variant: TextVariant::Muted,
                size: TextSize::Sm,
                ..TextProps::default()
            },
            vec![],
            vec![text(
                "アカウントを削除すると、すべてのデータが完全に失われ元に戻せません。",
            )],
        ),
        button(
            &ButtonProps {
                variant: ButtonVariant::Outline,
                palette: ColorPalette::Danger,
                ..ButtonProps::default()
            },
            vec![("data-blocks-settings-page-aside-nav-danger", "")],
            vec![text("アカウントを削除")],
        ),
    ]
}

/// 危険な操作カード（版 A、モジュール doc「構成」節 4.）。
fn danger_card() -> Node {
    card::root(
        CardProps::default(),
        vec![("data-blocks-settings-page-aside-nav-card", "")],
        vec![
            card::header(vec![], vec![card::title(vec![], vec![text("危険な操作")])]),
            card::body(
                vec![("class", "blocks-settings-page-aside-nav-fields")],
                danger_body(),
            ),
        ],
    )
}

/// 版 B（区切り線 3 セクション版）の 1 セクション（見出し + 本文）。
/// カード枠を持たない素の `div`/`heading` で組み立てる（モジュール doc
/// 「版 A / 版 B と集約元の対応」節参照）。
fn plain_section(title: &str, body: Vec<Node>) -> Node {
    let mut children = vec![heading(
        HeadingLevel::H3,
        &HeadingProps {
            size: HeadingSize::Sm,
            ..HeadingProps::default()
        },
        vec![],
        vec![text(title)],
    )];
    children.extend(body);
    div(
        vec![("class", "blocks-settings-page-aside-nav-plain-section")],
        children,
    )
}

/// 版 B の本文一式（プロフィール / 通知 / 危険な操作の 3 セクションを
/// `separator` 2 本で区切る、モジュール doc「`separator` は版 B のみで
/// 使う」節参照）。
fn variant_b_main() -> Vec<Node> {
    vec![
        plain_section(
            "プロフィール",
            vec![div(
                vec![("class", "blocks-settings-page-aside-nav-fields")],
                profile_fields("b"),
            )],
        ),
        separator(&SeparatorProps::default(), vec![]),
        plain_section("通知", vec![notification_fields("b")]),
        separator(&SeparatorProps::default(), vec![]),
        plain_section(
            "危険な操作",
            vec![div(
                vec![("class", "blocks-settings-page-aside-nav-fields")],
                danger_body(),
            )],
        ),
    ]
}

/// `settings-page-aside-nav` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。版 A（積み上げ 4 カード）・版 B（区切り線 3 セクション）を
/// 縦に並記する（モジュール doc「版 A / 版 B と集約元の対応」節参照）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-settings-page-aside-nav-stack")],
        vec![
            app_navbar(),
            page_heading("Kwame Boateng", "kwame.boateng@example.com"),
            version_title("版 A: 積み上げ 4 カード（R0656 / R0661 / R0662）"),
            div(
                vec![("class", "blocks-settings-page-aside-nav-layout")],
                vec![
                    aside_nav(),
                    div(
                        vec![("class", "blocks-settings-page-aside-nav-main")],
                        vec![
                            profile_card(),
                            plan_card(),
                            notification_card(),
                            danger_card(),
                        ],
                    ),
                ],
            ),
            version_title("版 B: 区切り線 3 セクション（R0657）"),
            div(
                vec![("class", "blocks-settings-page-aside-nav-variant-b")],
                variant_b_main(),
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
        Part {
            label: "Checkbox",
            path: "/themes/checkbox/",
        },
        Part {
            label: "Separator",
            path: "/themes/separator/",
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
.blocks-settings-page-aside-nav-checkboxes {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-settings-page-aside-nav-plain-section {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-settings-page-aside-nav-variant-b {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n}\n\
[data-scope=\"heading\"][data-blocks-settings-page-aside-nav-version-title] {\n  margin: 0 0 var(--fandhe-space-3);\n  border-top: none;\n  padding-top: 0;\n  letter-spacing: normal;\n}\n\
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
            "data-scope=\"checkbox\"",
            "data-scope=\"separator\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
        // ページ見出し（氏名）1 + 版見出し 2（版 A/版 B、モジュール doc
        // 「版見出しは H2（子見出しとの階層整合）」節参照）= 3。
        assert_eq!(html.matches("<h2").count(), 3);
        // 版 A カード title 4（プロフィール/プラン/通知/危険な操作）+ 版 B
        // セクション見出し 3（プロフィール/通知/危険な操作）= 7。
        assert_eq!(html.matches("<h3").count(), 7);
    }

    #[test]
    fn checkboxes_are_natively_disabled_with_mixed_states() {
        let html = demo_html();
        // 版 A 3 件 + 版 B 3 件 = 6 件。全件ネイティブ disabled。
        assert_eq!(html.matches("type=\"checkbox\"").count(), 6);
        // `" disabled=\"\""` は checkbox 6 件 + プランカードの請求周期
        // toggle-group 2 件（月払い/年払い、既存の `ToggleGroupProps.
        // disabled: true`）の合計 8 件。
        assert_eq!(html.matches(" disabled=\"\"").count(), 8);
        // メール・ブラウザ通知はチェック済み（版ごとに 2 件）= 4 件。
        assert_eq!(html.matches("checked=\"\"").count(), 4);
        assert!(html.contains("data-state=\"checked\""));
        assert!(html.contains("data-state=\"unchecked\""));
    }

    #[test]
    fn separator_appears_only_in_variant_b() {
        let html = demo_html();
        assert_eq!(html.matches("data-scope=\"separator\"").count(), 2);
    }

    #[test]
    fn element_ids_are_unique() {
        let html = demo_html();
        let mut ids: Vec<&str> = Vec::new();
        let mut rest = html.as_str();
        while let Some(start) = rest.find("id=\"") {
            let after = &rest[start + 4..];
            let end = after.find('"').expect("unterminated id attribute");
            ids.push(&after[..end]);
            rest = &after[end + 1..];
        }
        let mut sorted = ids.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(ids.len(), sorted.len(), "duplicate id found among: {ids:?}");
    }

    #[test]
    fn danger_action_is_outline_danger_button() {
        let html = demo_html();
        assert_eq!(
            html.matches("data-blocks-settings-page-aside-nav-danger")
                .count(),
            2
        );
        assert!(html.contains("アカウントを削除"));
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
        // checkbox の既定 `opacity: 0.5` + `cursor: not-allowed`
        // （disabled_declarations()）を中和しない（モジュール doc「通知
        // checkbox をネイティブ disabled にする理由」節参照。中和すると
        // 無効状態が視覚的に分からなくなる、Codex レビュー指摘対応）。
        assert!(!LAYOUT_CSS.contains(
            "[data-scope=\"checkbox\"][data-part=\"root\"][data-blocks-settings-page-aside-nav-checkbox][data-disabled]"
        ));
    }

    #[test]
    fn page_heading_resets_docs_content_h2_rule() {
        assert!(LAYOUT_CSS.contains(".blocks-settings-page-aside-nav-identity h2"));
        assert!(LAYOUT_CSS.contains("border-top: none;"));
    }

    #[test]
    fn version_title_resets_docs_content_h2_rule() {
        // 版見出し（H2）が `.docs-content h2`（site_theme.rs）由来の
        // `border-top`/`padding-top` を継承しないことを固定する
        // （Codex レビュー指摘対応）。
        let rule_start = LAYOUT_CSS
            .find("[data-scope=\"heading\"][data-blocks-settings-page-aside-nav-version-title]")
            .expect("version-title rule must exist");
        let rule = &LAYOUT_CSS[rule_start..];
        assert!(rule.starts_with(
            "[data-scope=\"heading\"][data-blocks-settings-page-aside-nav-version-title] {\n  margin: 0 0 var(--fandhe-space-3);\n  border-top: none;\n  padding-top: 0;\n  letter-spacing: normal;\n}"
        ));
    }

    #[test]
    fn aside_nav_marks_current_item_without_aria_current() {
        let html = demo_html();
        // `aria-current` は実在外部リンクへ付けない（codex 指摘対応）。強調
        // 表示は視覚専用の `data-*` 属性のみで行い、支援技術向けの現在地
        // ラベル（非表示テキスト含む）は付けない（2 度目の codex 指摘対応）。
        assert!(!html.contains("aria-current=\"page\""));
        assert_eq!(
            html.matches("data-blocks-settings-page-aside-nav-current")
                .count(),
            1
        );
        assert!(!html.contains("(current)"));
    }

    #[test]
    fn toggle_group_is_static() {
        let html = demo_html();
        assert!(html.contains("data-scope=\"toggle-group\" data-part=\"root\""));
        assert!(html.contains("data-disabled=\"\""));
    }
}
