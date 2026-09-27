//! `app-shell-sidebar` block（イシュー #2894。Application / App Shell
//! カテゴリの最初の block、`crate::blocks` モジュール doc の契約を
//! `login_01`/`dashboard_01`/`sidebar_07` 等に続いて実装する）。
//!
//! # 使用部品
//!
//! `sidebar`（常設サイドバー、collapsible なし。未読件数は `sidebar::
//! menu_badge` で表示、`menu_button` の予約余白と整合する組み込みの
//! 絶対配置に乗せる）/ `avatar`（チーム・プロフィールのフォールバック
//! イニシャル）/ `collapsible`（狭幅バーのハンバーガートリガー + 常時
//! 展開のモバイルナビパネル、無 JS 対応は「狭幅でもナビへ到達できる
//! ようにする」節参照）/ `icon`（自作の単純幾何アイコン）/ `heading`
//! （メイン領域の見出し）を合成する（[`BLOCK`] の `parts` に一致させる
//! 契約）。
//!
//! # 常設サイドバー ⇔ 上部バーの切替はコンテナクエリで行う
//!
//! `sidebar::provider`/`root` は `data-mobile` を付けるとドロワー相当の
//! `position: fixed` へ切り替わるが、これは Demo 枠から視覚的に逸脱する
//! （`sidebar_07` が同じ理由で不採用にした判断を継承）。本 block は
//! `data-mobile` を一切使わず、Demo 枠（`[data-blocks-app-shell-sidebar-
//! frame]`）へ `container-type: inline-size` を設定し、[`LAYOUT_CSS`] の
//! `@container` 規則だけでサイドバー非表示・上部バー表示を切り替える
//! （`header_flyout_menu` の `@container` 前例と同型）。`@container` は
//! コンテナ自身ではなく子孫のみに適用されるため、規則内ではコンテナ
//! （フレーム）を祖先セレクタとして再度要求せず、`[data-scope="sidebar"]
//! [data-part="root"]` を直接選択する（PR #3324 codex(P1) 再指摘の是正。
//! 祖先セレクタを重ねると規則が一致せず、狭幅でも常設サイドバーが非表示
//! にならなかった）。
//!
//! # 3 インスタンスを静的に並記する理由（無 JS）
//!
//! docs サイトは JS ハイドレーションを行わないため、動的な開閉・幅変化を
//! 実演できない。代わりに次の 3 インスタンスを縦に並べ、狙う状態を
//! それぞれ固定表示する。
//!
//! 1. `Desktop`: 幅制約なし。実ビューポートが狭ければコンテナクエリで
//!    自然に上部バーへ切り替わる。
//! 2. `Desktop — brand surface`: 同じ構造で、面色トークン
//!    （`--fandhe-color-sidebar-*`）だけを `accent` 系へ差し替える。
//! 3. `Narrow`: フレーム幅を [`LAYOUT_CSS`] で固定し、ビューポートに
//!    関係なく常に上部バー表示（サイドバー非表示）を示す。
//!
//! 暗色（ダークモード）は専用インスタンスを設けない。`sidebar` recipe の
//! 面トークンはテーマのダークモード切り替えで自動的に追従するため、
//! 静的な Demo でも別インスタンスを要しない。
//!
//! # `class` ではなく `data-*` で CSS フックを渡す
//!
//! `sidebar` の全パーツは呼び出し側 `attrs` の `class` を `drop_class_attr`
//! で黙って除去する契約を持つ（`sidebar_07` モジュール doc「CSS フックの
//! 選び方」節と同型）。本 block も統一して `data-blocks-app-shell-sidebar-*`
//! 属性で CSS フックを渡す。
//!
//! # アイコンは自作の単純幾何図形
//!
//! `sidebar_07::geo_icon` と同型の自作矩形アイコンのみを使う（実アイコン
//! セット由来の path データは複製しない）。
//!
//! # `<form>` を使わない・全データが架空
//!
//! `crate::blocks` モジュール doc の不変条件どおり `<form>` を出力しない。
//! ブランド名・チーム名・氏名・役職はすべて架空のもの（`dummy_assets` の
//! 架空セット、または本モジュール固有の架空値）であり、実企業名・実在
//! 人物・実クレデンシャル・PII を含まない。ナビ・チーム一覧・プロフィール
//! はいずれも静的な初期状態の掲示のみで、選択・送信・永続化・認証処理は
//! 行わない。ハンバーガーボタンの `aria-label` も「静的デモであり実際には
//! 開閉しない」ことと矛盾しない文言（`"Open navigation"`）にする。
//!
//! # 狭幅でもナビへ到達できるようにする（イシュー #2894 PR #3324 codex(P1)
//! 再指摘）
//!
//! `@container` で常設サイドバーを非表示にする一方、ハンバーガーを
//! `button::icon_button` の `disabled: true` のみで操作不能にすると、開いた
//! 先のパネルを一切描画しないため狭幅ではナビへ到達できなくなる
//! （[`super::super::marketing::header::header_simple_bar`] が同じ問題を
//! 是正した前例と同型の指摘）。同じ判断で、ハンバーガーは
//! `collapsible::trigger`（`OpenState::Open` + `disabled: true` 固定 =
//! 常時展開でクリックしても何も起きない）とし、`collapsible::content` の
//! 常時展開パネル（[`mobile_nav_panel`]）へ `aria-controls` で関連付ける。
//! パネルは [`main_nav`]/[`teams_group`] を（`header_simple_bar` の
//! `Node::clone()` 方式とは異なり）id 重複を避けるため suffix を分けて
//! 再呼び出しし、[`LAYOUT_CSS`] で `topbar` と同じ `@container` 条件下
//! でのみ表示する。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, nav, p, span, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::collapsible;
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::sidebar;
use fandhe_frontend_pre_styled_ui::sidebar::{
    Sidebar, SidebarCollapsible, SidebarMenuButtonProps, SidebarMenuButtonSize, SidebarProps,
    SidebarState,
};

/// 実在の自リポジトリ URL（`header_simple_bar` の「`href` の方針」節と
/// 同型。`href="#"` は `linkcheck` が拒否する死リンクのため使わない）。
/// メインナビ項目・モバイルナビパネル双方の唯一のリンク先とし、狭幅で
/// キーボード利用者が到達した先を実際に操作可能なリンクにする（codex(P2)
/// 指摘、イシュー #2894 PR #3324。`href: None` の `menu_button` は無 JS 環境
/// では動作のない `<button>` になり、モバイルナビパネル経由の唯一の到達
/// 手段が塞がれてしまう）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// 自作の単純な矩形アイコン（`sidebar_07::geo_icon` と同型。lucide 等の
/// 著作物を複製しないためのモジュール doc「アイコンは自作」節参照）。
fn geo_icon(path_d: &'static str) -> Node {
    icon(
        &IconProps::default(),
        vec![],
        vec![el("path", vec![("d", path_d)], vec![])],
    )
}

/// ハンバーガーアイコン（3 本線）。`icon` は `fill="currentColor"` 固定
/// （`stroke` を持たない）ため、面積を持たない線分パス（`M4 6h16` 等）は
/// 描画されない（Bugbot 指摘、PR #3324）。3 本の細い矩形（塗り面）として
/// 描く（`header_simple_bar::hamburger_icon` と同型の対処）。
fn hamburger_icon() -> Node {
    icon(
        &IconProps::default(),
        vec![],
        vec![el(
            "path",
            vec![("d", "M4 6h16v2H4zM4 11h16v2H4zM4 16h16v2H4z")],
            vec![],
        )],
    )
}

/// サイドバー header（ロゴ + 架空ブランド名）。
fn brand_header() -> Node {
    sidebar::header(
        vec![],
        vec![div(
            vec![("data-blocks-app-shell-sidebar-brand", "")],
            vec![
                geo_icon("M4 4h16v16H4z"),
                span(vec![], vec![text("Northshelf Console")]),
            ],
        )],
    )
}

/// メインナビ 1 行（icon + ラベル + 任意の未読 badge）。`sidebar::
/// menu_badge` は `menu_item` 直下で `menu_button` の兄弟として絶対配置
/// される組み込みパーツで、`menu_button` 側が `padding-inline-end` へ
/// 同じ 1 個分の余白をあらかじめ予約しているため、ラベルと重ならず
/// 折り返しもしない（`crate::pre_styled_ui::sidebar::menu_badge_base`
/// 参照）。汎用 `badge::badge` を素の兄弟として置いていた旧実装は
/// `menu_item` が flex コンテナでないため右寄せ・折り返し防止のいずれも
/// 効かなかった（Bugbot 指摘、PR #3324）。
fn nav_item(
    icon_path: &'static str,
    label: &'static str,
    active: bool,
    count: Option<&str>,
) -> Node {
    let button = sidebar::menu_button(
        &SidebarMenuButtonProps {
            href: Some(REPO),
            active,
            ..Default::default()
        },
        Some(geo_icon(icon_path)),
        vec![],
        vec![text(label)],
    );
    let mut children = vec![button];
    if let Some(count) = count {
        children.push(sidebar::menu_badge(vec![], vec![text(count)]));
    }
    sidebar::menu_item(vec![], children)
}

/// メインナビ群（label なしの `group`、架空の画面 5 件）。
fn main_nav() -> Node {
    sidebar::group(
        None,
        vec![],
        vec![sidebar::group_content(
            vec![],
            vec![sidebar::menu(
                vec![],
                vec![
                    nav_item("M4 4h16v16H4z", "Dashboard", true, None),
                    nav_item(
                        "M4 4h7v7H4zM13 4h7v7h-7zM4 13h7v7H4zM13 13h7v7h-7z",
                        "Inbox",
                        false,
                        Some("12"),
                    ),
                    nav_item("M5 3h14v18H5z", "Reports", false, None),
                    nav_item(
                        "M4 4h16v4H4zM4 10h16v4H4zM4 16h16v4H4z",
                        "Projects",
                        false,
                        None,
                    ),
                    nav_item(
                        "M12 2a10 10 0 1 0 .001 20.001A10 10 0 0 0 12 2z",
                        "Settings",
                        false,
                        None,
                    ),
                ],
            )],
        )],
    )
}

/// チーム 1 件（avatar フォールバックのイニシャル + 架空社名）。
fn team_item(suffix: &str, company: &'static str) -> Node {
    let initial = company
        .chars()
        .next()
        .map_or_else(|| "?".to_string(), |c| c.to_uppercase().collect::<String>());
    sidebar::menu_item(
        vec![],
        vec![sidebar::menu_button(
            &SidebarMenuButtonProps {
                href: None,
                active: false,
                ..Default::default()
            },
            Some(avatar::root(
                &AvatarProps::default(),
                vec![],
                vec![avatar::fallback(
                    ImageStatus::Error,
                    vec![],
                    vec![text(initial)],
                )],
            )),
            vec![("data-blocks-app-shell-sidebar-team", suffix)],
            vec![text(company)],
        )],
    )
}

/// `Teams` グループ（架空社名 3 件、[`dummy_assets::COMPANY_NAMES`] から
/// 先頭 3 件を採る）。
fn teams_group(suffix: &str) -> Node {
    let label_id = format!("blocks-app-shell-sidebar-teams-label-{suffix}");
    let items: Vec<Node> = dummy_assets::COMPANY_NAMES[..3]
        .iter()
        .map(|company| team_item(suffix, company))
        .collect();
    sidebar::group(
        Some(label_id.as_str()),
        vec![],
        vec![
            sidebar::group_label(Some(label_id.as_str()), vec![], vec![text("Teams")]),
            sidebar::group_content(vec![], vec![sidebar::menu(vec![], items)]),
        ],
    )
}

/// footer のプロフィール行（avatar フォールバック + 架空氏名・役職）。
/// `SidebarMenuButtonSize::Lg`（`height: 3rem`）を指定する（cursor(Medium)
/// 指摘、PR #3324。既定の `height: 2rem` では氏名 + 役職の 2 行
/// （`line-height: 1.2` で約 2.1rem）と avatar がボタン内で欠落・はみ出す）。
fn profile_footer() -> Node {
    let name = dummy_assets::PERSON_NAMES[0];
    let title = dummy_assets::JOB_TITLES[0];
    let initials: String = name
        .split_whitespace()
        .filter_map(|w| w.chars().next())
        .collect();
    sidebar::footer(
        vec![],
        vec![sidebar::menu(
            vec![],
            vec![sidebar::menu_item(
                vec![],
                vec![sidebar::menu_button(
                    &SidebarMenuButtonProps {
                        href: None,
                        active: false,
                        size: SidebarMenuButtonSize::Lg,
                        ..Default::default()
                    },
                    Some(avatar::root(
                        &AvatarProps::default(),
                        vec![],
                        vec![avatar::fallback(
                            ImageStatus::Error,
                            vec![],
                            vec![text(initials)],
                        )],
                    )),
                    vec![],
                    vec![span(
                        vec![("data-blocks-app-shell-sidebar-profile", "")],
                        vec![
                            span(vec![], vec![text(name)]),
                            span(vec![], vec![text(title)]),
                        ],
                    )],
                )],
            )],
        )],
    )
}

/// 常設サイドバー本体（header/content/footer、collapsible なし）。
/// `nav_label` はインスタンスごとに一意にする（cursor(Medium) 指摘、
/// PR #3324。Desktop/brand-surface の 2 インスタンスは同一
/// container query 幅では両方可視のため、同一 aria-label だと
/// ランドマーク名が重複する）。
fn app_sidebar(
    state: &Sidebar,
    props: &SidebarProps,
    suffix: &str,
    root_id: &str,
    nav_label: &str,
) -> Node {
    sidebar::root(
        state,
        props,
        nav_label,
        Some(root_id),
        vec![],
        vec![
            brand_header(),
            sidebar::content(vec![], vec![main_nav(), teams_group(suffix)]),
            profile_footer(),
        ],
    )
}

/// 狭幅時のみ表示する上部バー（ハンバーガー + 画面名 + avatar）。
/// ハンバーガーは常時展開の [`mobile_nav_panel`]（`panel_id`）を
/// `aria-controls` で指す（モジュール doc「狭幅でもナビへ到達できる
/// ようにする」節参照）。
fn topbar(suffix: &str, panel_id: &str) -> Node {
    let name = dummy_assets::PERSON_NAMES[0];
    let initials: String = name
        .split_whitespace()
        .filter_map(|w| w.chars().next())
        .collect();
    div(
        vec![("data-blocks-app-shell-sidebar-topbar", suffix)],
        vec![
            collapsible::trigger(
                collapsible::OpenState::Open,
                true,
                Some(panel_id),
                vec![
                    ("aria-label", "Open navigation"),
                    ("data-blocks-app-shell-sidebar-toggle", ""),
                ],
                vec![hamburger_icon()],
            ),
            span(
                vec![("data-blocks-app-shell-sidebar-screen-name", "")],
                vec![text("Dashboard")],
            ),
            avatar::root(
                &AvatarProps::default(),
                vec![],
                vec![avatar::fallback(
                    ImageStatus::Error,
                    vec![],
                    vec![text(initials)],
                )],
            ),
        ],
    )
}

/// 常時展開のモバイルナビパネル（狭幅専用、[`topbar`] のハンバーガーが
/// `aria-controls` で指す）。`main_nav`/`teams_group` を常設サイドバー側
/// とは別の suffix で再呼び出しし、id 重複を避ける（モジュール doc「狭幅
/// でもナビへ到達できるようにする」節参照。`header_simple_bar` の
/// `Node::clone()` 方式は使わない）。[`LAYOUT_CSS`] は [`topbar`] と同じ
/// `@container` 条件下でのみ表示する。
///
/// `disabled` は `false` を渡す（PR #3324 Bugbot/codex 再指摘）。この
/// パネル自体は常時展開の静的表示であり操作不能にする対象ではない。
/// `disabled: true` にすると headless 層が `data-disabled` を出力し、
/// pre-styled-ui の `[data-scope="collapsible"][data-part="content"]
/// [data-disabled]` 規則（`disabled_declarations()`、`opacity: 0.5`）が
/// パネル全体（Dashboard/Inbox/チーム行）へ波及して無効表示に見えてしまう
/// （操作不能にすべきなのは [`topbar`] のトリガーのみ）。
/// `nav_label` は `<nav>` ランドマークの `aria-label`。狭幅時は
/// [`app_sidebar`] の `sidebar::root` が `@container` で非表示になり
/// ナビゲーションへ到達できなくなるため、代替経路であるこのパネル
/// 自体を `<nav>` として囲み唯一のナビゲーションランドマークにする
/// （codex(P1) 指摘、PR #3324。`collapsible::content` は素の `<div>` を
/// 生成するのみでランドマーク要素を持たない）。
fn mobile_nav_panel(suffix: &str, panel_id: &str, nav_label: &str) -> Node {
    collapsible::content(
        collapsible::OpenState::Open,
        false,
        Some(panel_id),
        vec![("data-blocks-app-shell-sidebar-mobile-nav", "")],
        vec![nav(
            vec![("aria-label", nav_label)],
            vec![main_nav(), teams_group(suffix)],
        )],
    )
}

/// メイン領域（見出し + 空のコンテンツ枠）。
fn main_area() -> Node {
    div(
        vec![("data-blocks-app-shell-sidebar-main", "")],
        vec![
            heading::heading(
                // block ページ側が既に `## Demo` として h2 を出すため、
                // アウトライン上はそれより 1 段下げた H3 にする
                // （`contact_split_form_image`/`contact_image_info` 等
                // 他 block と同じ判断、cursor(Low) 指摘 PR #3324）。
                HeadingLevel::H3,
                &HeadingProps::default(),
                vec![],
                vec![text("Dashboard")],
            ),
            div(
                vec![("data-blocks-app-shell-sidebar-placeholder", "")],
                vec![],
            ),
        ],
    )
}

/// 1 インスタンス分の全体（frame > provider(root + inset) の構造）。
/// `surface` は `"default"`/`"brand"`（[`LAYOUT_CSS`] のセレクタと一致
/// させる、面色トークンの差し替え）。`narrow` は `true` のとき
/// [`LAYOUT_CSS`] がフレーム幅を固定して常に上部バー表示にする。
/// `instance_label` は [`demo`] のキャプションと対応させ、常設サイドバー
/// ・モバイルナビパネルいずれの `<nav>`/`sidebar::root` の `aria-label`
/// にも使う一意な接尾辞（cursor(Medium) 指摘、PR #3324）。
fn shell(suffix: &str, surface: &'static str, narrow: bool, instance_label: &str) -> Node {
    let state = Sidebar::new(SidebarState::Expanded);
    let props = SidebarProps {
        collapsible: SidebarCollapsible::None,
        ..SidebarProps::default()
    };
    let root_id = format!("blocks-app-shell-sidebar-root-{suffix}");
    let panel_id = format!("blocks-app-shell-sidebar-mobile-nav-{suffix}");
    let nav_label = format!("Main navigation — {instance_label}");
    let provider = sidebar::provider(
        &state,
        &props,
        vec![],
        vec![
            app_sidebar(&state, &props, suffix, &root_id, &nav_label),
            sidebar::inset(
                vec![],
                vec![
                    topbar(suffix, &panel_id),
                    mobile_nav_panel(&format!("{suffix}-mobile"), &panel_id, &nav_label),
                    main_area(),
                ],
            ),
        ],
    );

    let mut frame_attrs: Vec<(&str, &str)> = vec![
        ("data-blocks-app-shell-sidebar-frame", ""),
        ("data-blocks-app-shell-sidebar-surface", surface),
    ];
    if narrow {
        frame_attrs.push(("data-blocks-app-shell-sidebar-narrow", ""));
    }
    div(frame_attrs, vec![provider])
}

/// `app-shell-sidebar` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数。3 インスタンスを `data-blocks-app-shell-sidebar-stack` の下へ
/// 縦に並べる（モジュール doc「3 インスタンスを静的に並記する理由」参照）。
pub fn demo() -> Node {
    div(
        vec![("data-blocks-app-shell-sidebar-stack", "")],
        vec![
            p(
                vec![("data-blocks-app-shell-sidebar-caption", "")],
                vec![text("Desktop")],
            ),
            shell("desktop", "default", false, "Desktop"),
            p(
                vec![("data-blocks-app-shell-sidebar-caption", "")],
                vec![text("Desktop — brand surface")],
            ),
            shell("brand", "brand", false, "Desktop — brand surface"),
            p(
                vec![("data-blocks-app-shell-sidebar-caption", "")],
                vec![text("Narrow")],
            ),
            shell("narrow", "default", true, "Narrow"),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/app-shell-sidebar/",
    title: "app-shell-sidebar",
    category: BlockCategory::AppShell,
    rust_source: "crates/docs-site/src/blocks/application/app_shell/app_shell_sidebar.rs",
    demo_class: "blocks-app-shell-sidebar",
    parts: &[
        Part {
            label: "Sidebar",
            path: "/themes/sidebar/",
        },
        Part {
            label: "Avatar",
            path: "/themes/avatar/",
        },
        Part {
            label: "Collapsible",
            path: "/themes/collapsible/",
        },
        Part {
            label: "Icon",
            path: "/themes/icon/",
        },
        Part {
            label: "Heading",
            path: "/themes/heading/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// [`super::mod@self`] の `stylesheet()` が `push_css` する block 固有の
/// レイアウト CSS（`docs/design/docs-site-blocks-section.md` §10 追記節に
/// 従い、並列進行する他 block との `mod.rs::LAYOUT_CSS` 追記衝突を避け
/// 本モジュール側の定数へ分離する）。
///
/// `[data-blocks-app-shell-sidebar-frame]` へ `container-type: inline-size`
/// を設定し、`@container` 規則だけで常設サイドバー⇔上部バーを切り替える
/// （モジュール doc「常設サイドバー ⇔ 上部バーの切替はコンテナクエリで
/// 行う」節参照。`header_flyout_menu` の前例と同型）。面色（`surface`）は
/// リテラル色を書かず `--fandhe-color-accent*` トークン参照のみで
/// `--fandhe-color-sidebar-*` を上書きする。
///
/// `[data-blocks-app-shell-sidebar-mobile-nav]` は単独の属性セレクタ
/// （詳細度 `(0,1,0)`）ではなく `[data-scope="collapsible"][data-part=
/// "content"][data-blocks-app-shell-sidebar-mobile-nav]`（`(0,3,0)`）で
/// 記述する（Bugbot/codex(P2) 再指摘、PR #3324。pre-styled-ui の
/// collapsible content base 規則が同じ `(0,2,0)` の `margin-top`/
/// `padding`/`border`/`border-radius` を常時宣言しており、単独属性
/// セレクタでは詳細度で負けてカード風の余白・枠線・角丸が残っていた。
/// `header_simple_bar` の「CSS 特異性」節と同型の対処。`margin-top`/
/// `border`/`border-radius` を明示的に `0` へ戻し、フラットな
/// パネル境界線（`border-bottom` のみ）を保つ）。
const LAYOUT_CSS: &str = "\
.blocks-demo.blocks-app-shell-sidebar {\n  padding: 0;\n}\n\
[data-blocks-app-shell-sidebar-stack] {\n  display: flex;\n  flex-direction: column;\n  gap: 1rem;\n}\n\
[data-blocks-app-shell-sidebar-caption] {\n  margin: 0;\n  font-size: var(--fandhe-font-font-size-sm, 0.875rem);\n  font-weight: var(--fandhe-font-font-weight-medium, 500);\n  color: var(--fandhe-color-fg-muted);\n}\n\
[data-blocks-app-shell-sidebar-frame] {\n  container-type: inline-size;\n  container-name: blocks-app-shell-sidebar;\n  border: 1px solid var(--fandhe-color-border);\n  border-radius: var(--fandhe-radius-lg);\n  overflow: hidden;\n}\n\
[data-blocks-app-shell-sidebar-frame][data-blocks-app-shell-sidebar-narrow] {\n  max-inline-size: 22rem;\n}\n\
[data-blocks-app-shell-sidebar-frame] > [data-scope=\"sidebar\"][data-part=\"provider\"] {\n  min-height: 28rem;\n  height: auto;\n}\n\
[data-blocks-app-shell-sidebar-frame][data-blocks-app-shell-sidebar-surface=\"brand\"] [data-scope=\"sidebar\"][data-part=\"root\"] {\n  --fandhe-color-sidebar-bg: var(--fandhe-color-accent);\n  --fandhe-color-sidebar-fg: var(--fandhe-color-accent-fg);\n  --fandhe-color-sidebar-accent: var(--fandhe-color-accent-emphasized);\n  --fandhe-color-sidebar-accent-fg: var(--fandhe-color-accent-fg);\n  --fandhe-color-sidebar-border: var(--fandhe-color-accent-emphasized);\n  --fandhe-color-sidebar-muted: var(--fandhe-color-accent-emphasized);\n}\n\
[data-blocks-app-shell-sidebar-brand] {\n  display: flex;\n  align-items: center;\n  gap: 0.5rem;\n  padding: var(--fandhe-space-2, 0.5rem);\n  font-weight: var(--fandhe-font-font-weight-semibold, 600);\n}\n\
[data-blocks-app-shell-sidebar-profile] {\n  display: flex;\n  flex-direction: column;\n  overflow: hidden;\n  line-height: 1.2;\n  text-align: start;\n}\n\
[data-blocks-app-shell-sidebar-topbar] {\n  display: none;\n  align-items: center;\n  gap: 0.75rem;\n  padding: 0.75rem 1rem;\n  border-bottom: 1px solid var(--fandhe-color-border);\n}\n\
[data-scope=\"collapsible\"][data-part=\"trigger\"][data-blocks-app-shell-sidebar-toggle][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
[data-blocks-app-shell-sidebar-screen-name] {\n  margin-inline-end: auto;\n  font-weight: var(--fandhe-font-font-weight-medium, 500);\n}\n\
[data-scope=\"collapsible\"][data-part=\"content\"][data-blocks-app-shell-sidebar-mobile-nav] {\n  display: none;\n  flex-direction: column;\n  gap: 0.25rem;\n  margin-top: 0;\n  padding: 0.5rem 1rem 1rem;\n  border: 0;\n  border-radius: 0;\n  border-bottom: 1px solid var(--fandhe-color-border);\n}\n\
[data-blocks-app-shell-sidebar-main] {\n  display: flex;\n  flex-direction: column;\n  gap: 1rem;\n  padding: 1.5rem;\n}\n\
[data-blocks-app-shell-sidebar-placeholder] {\n  min-height: 16rem;\n  border: 2px dashed var(--fandhe-color-border);\n  border-radius: var(--fandhe-radius-lg);\n}\n\
@container blocks-app-shell-sidebar (max-width: 40rem) {\n  \
[data-scope=\"sidebar\"][data-part=\"root\"] {\n    display: none;\n  }\n  \
[data-blocks-app-shell-sidebar-topbar] {\n    display: flex;\n  }\n  \
[data-scope=\"collapsible\"][data-part=\"content\"][data-blocks-app-shell-sidebar-mobile-nav] {\n    display: flex;\n  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// Demo が期待する部品を実際に出力し、非対話制約
    /// （`<form>`/`data:` 不在）を満たすことの単体回帰
    /// （`crates/docs-site/tests/blocks_contract.rs` の横断検査と重複
    /// し過ぎない範囲での個別固定）。
    #[test]
    fn demo_composes_expected_parts_and_has_no_form_or_data_uri() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"sidebar\"",
            "data-scope=\"avatar\"",
            "data-scope=\"collapsible\"",
            "data-scope=\"heading\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert!(
            html.contains("data-part=\"menu-badge\""),
            "demo should render the unread count via sidebar::menu_badge"
        );
        assert!(!html.contains("<form"), "demo should not emit <form>");
        assert!(
            !html.contains("src=\"data:"),
            "demo should not emit data: URIs"
        );
    }

    /// 3 インスタンス（caption・frame・topbar）が並記されていること。
    #[test]
    fn demo_has_three_instances() {
        let html = render(&demo());
        assert_eq!(
            html.matches("data-blocks-app-shell-sidebar-caption")
                .count(),
            3,
            "demo should render exactly 3 captions"
        );
        assert_eq!(
            html.matches("data-blocks-app-shell-sidebar-frame").count(),
            3,
            "demo should render exactly 3 frames"
        );
        assert_eq!(
            html.matches("data-blocks-app-shell-sidebar-topbar").count(),
            3,
            "demo should render exactly 3 topbars (hidden by default via CSS)"
        );
        assert_eq!(
            html.matches("data-blocks-app-shell-sidebar-mobile-nav")
                .count(),
            3,
            "demo should render exactly 3 always-open mobile nav panels \
             (reachable in narrow @container widths, hidden by CSS otherwise)"
        );
        assert_eq!(
            html.matches("data-blocks-app-shell-sidebar-surface=\"brand\"")
                .count(),
            1,
            "demo should render exactly 1 brand-surface instance"
        );
        assert_eq!(
            html.matches("data-blocks-app-shell-sidebar-narrow").count(),
            1,
            "demo should render exactly 1 narrow instance"
        );
    }

    /// ハンバーガーが `type="button"`・`aria-label`・`disabled`（無 JS の
    /// no-op を明示）を持ち、常時展開の [`super::mobile_nav_panel`] を
    /// `aria-controls` で指すこと（狭幅でもナビへ到達可能、イシュー #2894
    /// PR #3324 codex(P1) 再指摘の是正）。
    #[test]
    fn hamburger_is_disabled_trigger_controlling_reachable_panel() {
        let html = render(&demo());
        assert!(
            html.contains(r#"aria-label="Open navigation""#),
            "hamburger button should have an aria-label"
        );
        assert!(
            html.contains(r#"type="button""#),
            "buttons should stay type=\"button\" (no implicit form submit)"
        );
        assert_eq!(
            html.matches("data-blocks-app-shell-sidebar-toggle").count(),
            3,
            "demo should render exactly 3 hamburger triggers"
        );
        assert!(
            html.contains(r#"aria-controls="blocks-app-shell-sidebar-mobile-nav-desktop""#),
            "hamburger trigger should control its instance's mobile nav panel via aria-controls"
        );
    }

    /// 常時展開のモバイルナビパネル自体は `disabled` にしないこと（PR #3324
    /// Bugbot/codex 再指摘）。`data-disabled` を持つと pre-styled-ui の
    /// disabled 視覚規則（`opacity: 0.5`）がパネル全体へ波及し、
    /// Dashboard/Inbox/チーム行が無効表示に見えてしまう。無効化すべきは
    /// [`super::topbar`] のトリガーのみで、常時展開の静的パネル自体は
    /// 操作対象ではない。
    #[test]
    fn mobile_nav_panel_content_is_not_disabled() {
        let html = render(&demo());
        for instance in ["desktop", "brand", "narrow"] {
            let marker = format!(r#"id="blocks-app-shell-sidebar-mobile-nav-{instance}""#);
            let pos = html
                .find(&marker)
                .unwrap_or_else(|| panic!("mobile nav panel for {instance} should render its id"));
            // 開始タグの直前 200 バイト程度に data-disabled が現れないことを
            // 確認する（`content` の属性は `data-state`/`data-disabled`/`id`
            // の順で組み立てられるため、id の直前を見れば足りる）。
            let window_start = pos.saturating_sub(200);
            let window = &html[window_start..pos];
            assert!(
                !window.contains("data-disabled"),
                "mobile nav panel content ({instance}) should not carry \
                 data-disabled, or its disabled-state CSS \
                 (opacity: 0.5) dims the whole panel"
            );
        }
    }

    /// 狭幅表示（`@container` 適用時）で `sidebar::root` が `display: none`
    /// になっても、[`super::mobile_nav_panel`] 自身が `<nav>` ランドマーク
    /// を持ちナビゲーションへ到達できること（codex(P1) 指摘、PR #3324）。
    #[test]
    fn mobile_nav_panel_has_own_nav_landmark() {
        let html = render(&demo());
        for instance in ["desktop", "brand", "narrow"] {
            let marker = format!(r#"id="blocks-app-shell-sidebar-mobile-nav-{instance}""#);
            let panel_start = html
                .find(&marker)
                .unwrap_or_else(|| panic!("mobile nav panel for {instance} should render its id"));
            // `<nav>` はパネル開始タグ直後（main_nav/teams_group より前）に
            // 現れるはずなので、id 直後 200 バイト程度の窓に限定して探す
            // （他インスタンスの `<nav>` を誤って拾わないため）。
            let window = &html[panel_start..(panel_start + 200).min(html.len())];
            assert!(
                window.contains("<nav"),
                "mobile nav panel for {instance} should wrap its items in <nav>"
            );
        }
    }

    /// 3 インスタンスの `sidebar::root`/モバイルナビ `<nav>` が、いずれも
    /// 一意な `aria-label` を持つこと（同一名のランドマークが複数同時に
    /// 可視化されるのを防ぐ、cursor(Medium) 指摘 PR #3324）。
    #[test]
    fn nav_landmarks_have_unique_aria_labels() {
        let html = render(&demo());
        for label in [
            "Main navigation — Desktop",
            "Main navigation — Desktop — brand surface",
            "Main navigation — Narrow",
        ] {
            let needle = format!(r#"aria-label="{label}""#);
            assert!(
                html.contains(&needle),
                "expected a unique aria-label {needle:?} in demo output"
            );
        }
    }

    /// `LAYOUT_CSS` が `@container` の狭幅切替とブランド面色の上書きを持ち、
    /// 色リテラルを含まないこと。
    #[test]
    fn layout_css_has_container_query_and_token_only_brand_surface() {
        assert!(
            LAYOUT_CSS.contains("@container blocks-app-shell-sidebar (max-width: 40rem)"),
            "LAYOUT_CSS should define the narrow-width container query"
        );
        assert!(
            LAYOUT_CSS.contains("--fandhe-color-sidebar-bg: var(--fandhe-color-accent)"),
            "LAYOUT_CSS should override sidebar bg via a theme token, not a literal color"
        );
        assert!(
            !LAYOUT_CSS.contains('#'),
            "LAYOUT_CSS should not contain literal hex color values"
        );
    }

    /// メインナビ項目（常設サイドバー・モバイルナビパネル双方）が実在の
    /// 自リポジトリ URL を `href` に持つ操作可能なリンクであること
    /// （`href="#"` は使わない。codex(P2) 指摘、PR #3324）。
    #[test]
    fn main_nav_items_are_reachable_links() {
        let html = render(&demo());
        assert!(
            html.matches(r#"href="https://github.com/Fandhe-AI/fandhe-frontend""#)
                .count()
                >= 10,
            "main nav items across the 3 sidebar instances and 3 mobile nav \
             panels (5 items each) should render real hrefs"
        );
        assert!(
            !html.contains(r##"href="#""##),
            "demo should not use dead href=\"#\" links (linkcheck rejects them)"
        );
    }

    /// `LAYOUT_CSS` の mobile-nav 規則が pre-styled-ui の collapsible
    /// content base 規則（`(0,2,0)`）へ詳細度で勝つ複合セレクタ（`(0,3,0)`）
    /// で書かれ、カード風の余白・枠線・角丸を明示的に打ち消すこと
    /// （Bugbot/codex(P2) 再指摘、PR #3324）。
    #[test]
    fn mobile_nav_layout_css_outranks_collapsible_content_recipe() {
        assert!(
            LAYOUT_CSS.contains(
                r#"[data-scope="collapsible"][data-part="content"][data-blocks-app-shell-sidebar-mobile-nav]"#
            ),
            "mobile-nav rule should be compounded with the collapsible content \
             part selector to win specificity over the base recipe"
        );
        assert!(
            LAYOUT_CSS.contains("margin-top: 0")
                && LAYOUT_CSS.contains("border: 0")
                && LAYOUT_CSS.contains("border-radius: 0"),
            "mobile-nav rule should explicitly reset the card-like spacing/border \
             the collapsible content base recipe always declares"
        );
    }

    /// footer のプロフィール行が `data-size="lg"`（`height: 3rem`）を持つこと
    /// （cursor(Medium) 指摘、PR #3324。既定の 2rem では氏名 + 役職の 2 行 +
    /// avatar がボタン内に収まらない）。
    #[test]
    fn profile_footer_menu_button_uses_lg_size() {
        let html = render(&demo());
        assert_eq!(
            html.matches(r#"data-size="lg""#).count(),
            3,
            "each of the 3 instances should render its profile menu-button at size=lg"
        );
    }
}
