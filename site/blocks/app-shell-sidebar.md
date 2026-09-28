# app-shell-sidebar

`fandhe-frontend-pre-styled-ui` の `sidebar`（常設サイドバー、collapsible
なし。未読件数は組み込みの `menu_badge` で表示）/ `avatar`（チーム・
プロフィールのフォールバックイニシャル）/ `collapsible`（狭幅バーの
ハンバーガートリガー + 常時展開のモバイルナビパネル）/ `icon`（自作の
単純幾何アイコン）/ `heading`（メイン領域の見出し）を合成した、常設
サイドバー型アプリシェルの合成例です。Blocks セクションは
新規部品を追加するものではなく、既存の Themes/Primitives 部品を組み合わせた
実例集であることに注意してください。

デスクトップ幅では左側に幅広の常設サイドバー（ロゴ・アイコン付きメイン
ナビ・チーム一覧・下端のプロフィール）が表示され、右側がメイン領域
（見出し + 空のコンテンツ枠のみの骨格）になります。狭いコンテナ幅では
サイドバーが非表示になり、代わりに上部バー（ハンバーガー・画面名・
アバター）が表示されます。この切替は `data-mobile`（`position: fixed` の
ドロワー）ではなく、Demo 枠自体にコンテナクエリ（`@container`）を適用する
ことで実現しています。docs サイトは無 JS のためハンバーガーは常に
`disabled`（クリックしても何も起きない no-op）ですが、常時展開の
モバイルナビパネルへ `aria-controls` で関連付けているため、狭いコンテナ幅
でもナビ項目（画面一覧・チーム一覧）自体には静的に到達できます。

docs サイトは JS ハイドレーションを行わないため、狙う状態を **3 つの
インスタンスを縦に並べて** 静的に掲示します。

1. **Desktop**: 幅の制約なし。実ビューポートが狭ければコンテナクエリで
   自然に上部バーへ切り替わります。
2. **Desktop — brand surface**: 同じ構造のまま、面色トークン
   （`--fandhe-color-sidebar-*`）だけを `accent` 系へ差し替えたものです。
3. **Narrow**: Demo 枠の幅を固定し、常に上部バー表示（サイドバー非表示）を
   示します。

暗色（ダークモード）は専用インスタンスを設けていません。`sidebar` recipe の
面トークンはテーマのダークモード切り替えで自動的に追従するためです。

本 Demo は静的な表示例であり、`<form>` 要素を持たず、値の送信・検証・認証
処理・データ取得を一切行いません（無 JS 制約、
`docs/policy/intentional-non-adoption.md` §3.25 の責務境界: UI コンポーネント層
はアプリケーションロジックを内包しません）。ブランド名・チーム名（社名）・
氏名・役職はすべて架空のものであり、実企業名・実在人物・実クレデンシャル・
PII を含みません。アイコンは lucide 等の著作物ではなく自作の単純な幾何図形
です。

## Rust コード

```rust
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
```

## 集約元との差分メモ

- 暗色（ダークモード）: 専用インスタンスを設けず、テーマのダークモード
  切り替えで `sidebar` recipe の面トークンが自動的に追従することを利用
  しています。
- 背景色のみの差分: `--fandhe-color-sidebar-bg` の差し替えだけで表現でき、
  構造は変更していません。
- ブランド配色: Demo 2 番目のインスタンス（brand surface）で示しています。
- サイドバーの中身が空枠のもの: 本 block と骨格は同一で、ナビ・チーム
  一覧を空にするだけで再現できます。
- サイドバー幅違い: `sidebar` の `--fandhe-sidebar-width` トークンを上書き
  するだけで表現できます。
