//! イシュー #947（部品ページ充填 — Navigation / Data Display 系）の原稿データ。
//!
//! # 役割・呼び出し文脈
//!
//! [`crate::component_page::COMPONENT_SPECS`] から `path -> ComponentPageSpec`
//! の 1 タプルとして参照される定数群。[`crate::component_page`] 側の合成規則
//! （Demo → Features → Anatomy → API Reference → Examples → Accessibility の
//! 6 節、Anatomy・`data-*` 属性表・CSS 変数表は機械導出）に対し、本モジュールは
//! 原稿供給が必要な 5 フィールド（`features`/`arguments`/`examples`/
//! `keyboard`/`aria`）のみを埋める。
//!
//! # 一次情報
//!
//! `features`/`arguments`/`keyboard`/`aria` の各行は `crates/pre-styled-ui/src/`
//! （必要に応じて `crates/headless-ui/src/`）の実ソースを一次情報とし、各定数の
//! doc コメントに `file:line` 形式で根拠を付す。根拠を示せない行は掲載しない
//! （固有の ARIA/キーボード操作を持たない部品は、その旨を明示する 1 行のみを
//! `aria` に置く。`docs/design/docs-site-component-pages.md` §10・
//! `.claude/rules/out-of-scope-tracking.md` の「推測で補完しない」方針）。
//!
//! `examples` のレンダラは `fandhe_frontend_pre_styled_ui`（および、
//! `crates/docs-site` が直接依存しない headless-ui 型は
//! `fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui` 経由、
//! `docs/design/docs-site-component-pages.md` イシュー #693 方針）の公開 API
//! のみを呼び出して組み立てる。`showcase.rs`（イシュー #941 のレジストリ）の
//! 私有関数は再利用しない（#947 実装計画 §4「`showcase.rs` の私有関数は
//! 再利用不可」）。
//!
//! # セキュリティ不変条件（REQ-1）
//!
//! すべてのデータは `&'static str` リテラルであり、[`crate::component_page`]
//! 側で `fandhe_frontend_core::text()` 経由（既定エスケープ）にのみ出力される。
//! 本モジュールは `raw_html()` および HTML 文字列の直接組み立て
//! （`format!("<td>{}</td>", …)`）を一切使わない。Examples レンダラも
//! ノード木 API のみで組み立て、`docs-` 接頭辞の class を持ち込まない
//! （`tests/site_css_contract.rs::component_page_render_introduces_no_class_outside_the_contract`
//! が層 1 (c) 方向で回帰として固定する）。

use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::{
    alert,
    attachment::{self, AttachmentRootProps, AttachmentState, AttachmentVariant},
    avatar, badge, breadcrumb,
    bubble::{self, BubbleGroupPosition, BubbleRootProps, BubbleVariant},
    button::{button, ButtonProps, ButtonVariant},
    callout, card, carousel, color_swatch, data_list, empty_state, field, icon, image,
    item::{self, ItemMediaVariant, ItemRootProps},
    json_tree_view,
    marker::{self as marker, MarkerRootProps, MarkerTone, MarkerVariant},
    marquee,
    message::{self, MessageAlign, MessageRole, MessageRootProps},
    native_select, pagination, progress, scroll_area, separator,
    sidebar::{
        self, Sidebar, SidebarCollapsible, SidebarMenuButtonProps, SidebarProps, SidebarState,
        SidebarVariant,
    },
    skeleton, spinner, splitter, stat, status, steps, tab_nav, table, tag, timeline, tree_view,
    AlertProps, ColorPalette, OpenState, Orientation, Size,
};

use crate::component_page::{ArgRow, AriaRow, ComponentPageSpec, ExampleEntry, KeyRow};

// ---------------------------------------------------------------------
// Data Display（本文明示 15 件）
// ---------------------------------------------------------------------

/// `crates/pre-styled-ui/src/alert.rs`（`root` が `role="alert"` を固定
/// 付与、`AlertStatus`/`AlertVariant`/`Size` の 3 軸、イシュー #1553）。
fn ex_alert() -> Node {
    let props = AlertProps {
        status: alert::AlertStatus::Warning,
        ..AlertProps::default()
    };
    alert::root(
        &props,
        vec![],
        vec![
            alert::indicator(vec![], vec![]),
            alert::content(
                vec![],
                vec![
                    alert::title(vec![], vec![text("Heads up")]),
                    alert::description(vec![], vec![text("Something needs attention")]),
                ],
            ),
        ],
    )
}

/// `crates/pre-styled-ui/src/alert.rs`（shadcn `default` 相当。イシュー
/// #2043 の突合で status/variant 既存軸の組み合わせで再現できると判定した）。
fn ex_alert_neutral_surface() -> Node {
    alert::root(
        &AlertProps {
            status: alert::AlertStatus::Neutral,
            variant: alert::AlertVariant::Surface,
            ..AlertProps::default()
        },
        vec![],
        vec![
            alert::indicator(vec![], vec![]),
            alert::content(
                vec![],
                vec![
                    alert::title(vec![], vec![text("shadcn default 相当")]),
                    alert::description(vec![], vec![text("Neutral + Surface の組み合わせです")]),
                ],
            ),
        ],
    )
}

/// `crates/pre-styled-ui/src/alert.rs`（shadcn `destructive` 相当。イシュー
/// #2043 の突合で status/variant 既存軸の組み合わせで再現できると判定した）。
fn ex_alert_error_outline() -> Node {
    alert::root(
        &AlertProps {
            status: alert::AlertStatus::Error,
            variant: alert::AlertVariant::Outline,
            ..AlertProps::default()
        },
        vec![],
        vec![
            alert::indicator(vec![], vec![]),
            alert::content(
                vec![],
                vec![
                    alert::title(vec![], vec![text("shadcn destructive 相当")]),
                    alert::description(vec![], vec![text("Error + Outline の組み合わせです")]),
                ],
            ),
        ],
    )
}

/// `crates/pre-styled-ui/src/alert.rs`（title のみの構成、イシュー #2043）。
fn ex_alert_title_only() -> Node {
    alert::root(
        &AlertProps::default(),
        vec![],
        vec![alert::content(
            vec![],
            vec![alert::title(vec![], vec![text("title のみ")])],
        )],
    )
}

/// `crates/pre-styled-ui/src/alert.rs`（description のみの構成、
/// イシュー #2043）。
fn ex_alert_description_only() -> Node {
    alert::root(
        &AlertProps::default(),
        vec![],
        vec![alert::content(
            vec![],
            vec![alert::description(vec![], vec![text("description のみ")])],
        )],
    )
}

/// `crates/pre-styled-ui/src/alert.rs`（pre-styled-only `action` パート、
/// イシュー #2043。shadcn `AlertAction` 相当を root 末尾の button 併記で
/// 表現する）。
fn ex_alert_with_action() -> Node {
    alert::root(
        &AlertProps::default(),
        vec![],
        vec![
            alert::indicator(vec![], vec![]),
            alert::content(
                vec![],
                vec![
                    alert::title(vec![], vec![text("Update available")]),
                    alert::description(vec![], vec![text("A new version is ready to install.")]),
                ],
            ),
            alert::action(
                vec![],
                vec![button(
                    &ButtonProps {
                        variant: ButtonVariant::Outline,
                        size: Size::Sm,
                        ..ButtonProps::default()
                    },
                    vec![("type", "button")],
                    vec![text("Update")],
                )],
            ),
        ],
    )
}

pub(crate) const ALERT: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "AlertStatus（Info/Success/Warning/Error/Neutral、crates/pre-styled-ui/src/alert.rs）で 5 種の状態色を切り替える（イシュー #1553）",
        "AlertVariant（Subtle/Surface/Solid/Outline、既定 Subtle）で見た目のトーンを切り替える（イシュー #1553）",
        "size（Xs〜Xl、既定 Md）でパディング・フォントサイズ・indicator サイズを切り替える（イシュー #1553）",
        "root に WAI-ARIA live region の role=\"alert\" を状態に関わらず固定付与する（alert.rs）",
        "indicator/content/title/description/action の 5 パーツで見出し・本文・アクションを構造化できる（action は pre-styled-only レイアウトパート、イシュー #2043）",
    ],
    arguments: &[
        ArgRow {
            name: "status",
            kind: "AlertStatus",
            default: "Info",
            description: "見た目の状態色（alert.rs、#[default] は Info）。",
        },
        ArgRow {
            name: "variant",
            kind: "AlertVariant",
            default: "Subtle",
            description: "見た目のトーン（Subtle/Surface/Solid/Outline、イシュー #1553）。",
        },
        ArgRow {
            name: "size",
            kind: "Size",
            default: "Md",
            description: "サイズ（Xs〜Xl、イシュー #1553）。",
        },
    ],
    examples: &[
        ExampleEntry {
            title: "Warning",
            description: "Warning 状態の Alert を indicator + title/description で組み立てた例です。",
            render: ex_alert,
        },
        ExampleEntry {
            title: "Default (shadcn 相当)",
            description: "shadcn/ui の variant \"default\" に相当する Neutral + Surface の組み合わせです（イシュー #2043）。",
            render: ex_alert_neutral_surface,
        },
        ExampleEntry {
            title: "Destructive (shadcn 相当)",
            description: "shadcn/ui の variant \"destructive\" に相当する Error + Outline の組み合わせです（イシュー #2043）。",
            render: ex_alert_error_outline,
        },
        ExampleEntry {
            title: "Title only",
            description: "title のみを content に含めた最小構成です（イシュー #2043）。",
            render: ex_alert_title_only,
        },
        ExampleEntry {
            title: "Description only",
            description: "description のみを content に含めた最小構成です（イシュー #2043）。",
            render: ex_alert_description_only,
        },
        ExampleEntry {
            title: "With action",
            description: "shadcn/ui の AlertAction 相当を、pre-styled-only レイアウトパート action（root 末尾の button 併記）で表現した例です（イシュー #2043）。",
            render: ex_alert_with_action,
        },
    ],
    keyboard: &[],
    aria: &[AriaRow {
        attribute: "role=\"alert\"",
        description: "WAI-ARIA live region。status の値に関わらず固定で付与される（alert.rs）。",
    }],
    demo: None,
};

fn ex_avatar() -> Node {
    avatar::root(
        &avatar::AvatarProps::default(),
        vec![],
        vec![avatar::fallback(
            avatar::ImageStatus::Error,
            vec![],
            vec![text("FT")],
        )],
    )
}

/// `crates/pre-styled-ui/src/avatar.rs`（イシュー #2044、shadcn/ui 突合）:
/// 重なり表示 + `+N` の例。`group` + `stacked: true` root 2 個 +
/// `Subtle/Neutral` root（`+3` の fallback）の組み合わせで表現する
/// （独立パートを新設しない設計判断、avatar.rs モジュール冒頭 rustdoc
/// 「イシュー #2044 の shadcn/ui 突合」節参照）。
fn ex_avatar_group() -> Node {
    avatar::group(
        vec![],
        vec![
            avatar::root(
                &avatar::AvatarProps {
                    stacked: true,
                    ..avatar::AvatarProps::default()
                },
                vec![],
                vec![avatar::fallback(
                    avatar::ImageStatus::Error,
                    vec![],
                    vec![text("FT")],
                )],
            ),
            avatar::root(
                &avatar::AvatarProps {
                    stacked: true,
                    ..avatar::AvatarProps::default()
                },
                vec![],
                vec![avatar::fallback(
                    avatar::ImageStatus::Error,
                    vec![],
                    vec![text("NM")],
                )],
            ),
            avatar::root(
                &avatar::AvatarProps {
                    stacked: true,
                    ..avatar::AvatarProps::default()
                },
                vec![],
                vec![avatar::fallback(
                    avatar::ImageStatus::Error,
                    vec![],
                    vec![text("+3")],
                )],
            ),
        ],
    )
}

/// `crates/pre-styled-ui/src/avatar.rs`（イシュー #2044、shadcn/ui 突合）:
/// 右下の状態ドット（`badge`）の例。`with_badge: true` の root に
/// `AvatarBadgeProps`（既定 `Md`/`Accent`）の badge を子として重ねる。
fn ex_avatar_badge() -> Node {
    avatar::root(
        &avatar::AvatarProps {
            with_badge: true,
            ..avatar::AvatarProps::default()
        },
        vec![],
        vec![
            avatar::fallback(avatar::ImageStatus::Error, vec![], vec![text("FT")]),
            avatar::badge(&avatar::AvatarBadgeProps::default(), vec![], vec![]),
        ],
    )
}

pub(crate) const AVATAR: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "AvatarShape（Circle/Rounded/Square、crates/pre-styled-ui/src/avatar.rs）で外形を切り替える",
        "AvatarVariant（Subtle/Solid/Outline、イシュー #1554 で追加）で見た目バリアントを切り替える",
        "ColorPalette（6 値、既定 Neutral、イシュー #1554 で追加）で colorPalette 軸を切り替える",
        "ImageStatus に連動して image/fallback パーツの表示・非表示を CSS の [data-state=\"hidden\"] で切り替える",
        "image パーツは alt テキストを必須引数として要求する（avatar.rs 内 image 再エクスポート）",
        "group（イシュー #2044）で複数 root を重なり表示し、+N の残数表示は root(stacked) + fallback の組み合わせで表現する",
        "badge（イシュー #2044）で root 右下に状態ドットを絶対配置し、with_badge: true の root のみ overflow を解除する",
    ],
    arguments: &[
        ArgRow {
            name: "size",
            kind: "Size",
            default: "Md",
            description: "サイズ（avatar.rs の AvatarProps、#[default] は Md。イシュー #1554 で 24/32/40/48/56px へ是正）。",
        },
        ArgRow {
            name: "shape",
            kind: "AvatarShape",
            default: "Circle",
            description: "外形（avatar.rs の AvatarShape、#[default] は Circle）。",
        },
        ArgRow {
            name: "variant",
            kind: "AvatarVariant",
            default: "Subtle",
            description: "見た目バリアント（avatar.rs の AvatarVariant、#[default] は Subtle。イシュー #1554 で追加）。",
        },
        ArgRow {
            name: "palette",
            kind: "ColorPalette",
            default: "Neutral",
            description: "colorPalette 軸（avatar.rs の AvatarProps、#[default] は Neutral。イシュー #1554 で追加）。",
        },
        ArgRow {
            name: "stacked",
            kind: "bool",
            default: "false",
            description: "group 内で重なり表示するか（avatar.rs の AvatarProps、イシュー #2044 で追加）。",
        },
        ArgRow {
            name: "with_badge",
            kind: "bool",
            default: "false",
            description: "badge を子に持つか（avatar.rs の AvatarProps、イシュー #2044 で追加。true のときのみ overflow: visible を解除する）。",
        },
        ArgRow {
            name: "group()",
            kind: "fn",
            default: "-",
            description: "pre-styled-only group パート（avatar.rs、イシュー #2044）。headless-ui の anatomy には存在しない。",
        },
        ArgRow {
            name: "badge()",
            kind: "fn",
            default: "-",
            description: "pre-styled-only badge パート（avatar.rs、イシュー #2044）。AvatarBadgeProps（size: Md、palette: Accent）を取る。",
        },
        ArgRow {
            name: "AvatarBadgeProps",
            kind: "struct",
            default: "size: Md, palette: Accent",
            description: "badge() の設定（avatar.rs、イシュー #2044。size は Xs〜Xl の 5 段、palette は ColorPalette 6 値）。",
        },
    ],
    examples: &[
        ExampleEntry {
            title: "Fallback",
            description: "画像読み込み失敗（ImageStatus::Error）時のイニシャル表示例です。",
            render: ex_avatar,
        },
        ExampleEntry {
            title: "重なり表示 + +N",
            description: "group + stacked root 2 個 + +3 の fallback を組み合わせた重なり表示の例です（イシュー #2044）。",
            render: ex_avatar_group,
        },
        ExampleEntry {
            title: "状態 badge",
            description: "with_badge: true の root の右下に badge（既定 Accent）を重ねた例です（イシュー #2044）。",
            render: ex_avatar_badge,
        },
    ],
    keyboard: &[],
    aria: &[AriaRow {
        attribute: "(該当なし)",
        description: "root/image/fallback/group/badge は固有の role/aria-* を出力しない（avatar.rs 全文で role/aria-* を grep しても 0 件）。image パーツの alt テキストが代替情報を提供する。badge は装飾のためアクセシブルネームを持たず、必要な場合は呼び出し側が root の aria-label 等で供給する（イシュー #2044）。",
    }],
    demo: None,
};

fn ex_badge() -> Node {
    badge::badge(
        &badge::BadgeProps {
            variant: badge::BadgeVariant::Solid,
            ..badge::BadgeProps::default()
        },
        vec![],
        vec![text("New")],
    )
}

/// イシュー #2045: shadcn/ui With Icon 例に相当。`icon::icon` を子ノードと
/// して並べるだけで再現できる（badge.rs base の `gap` が既に対応済み）。
fn ex_badge_with_icon() -> Node {
    badge::badge(
        &badge::BadgeProps::default(),
        vec![],
        vec![
            icon::icon(
                &icon::IconProps {
                    size: Size::Xs,
                    label: None,
                    ..icon::IconProps::default()
                },
                vec![],
                vec![el(
                    "path",
                    vec![("d", "M9 16.17 4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z")],
                    vec![],
                )],
            ),
            text("Verified"),
        ],
    )
}

/// イシュー #2045: shadcn/ui `link`（`render` prop）相当。専用コンストラクタ
/// `badge::link` が `<a>` を組み立てる（badge.rs `link` rustdoc 参照）。
/// `href=""`（空文字列）は showcase.rs の href 中立性の作法に倣う
/// （`crate::linkcheck::check_links` が無条件許容する空文字列を使い、実
/// ページへ解決される href を持ち込まない）。
fn ex_badge_link() -> Node {
    badge::link(
        "",
        &badge::BadgeProps {
            variant: badge::BadgeVariant::Outline,
            ..badge::BadgeProps::default()
        },
        false,
        vec![],
        vec![text("As link")],
    )
}

pub(crate) const BADGE: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "BadgeVariant（Solid/Subtle/Outline/Surface/Plain、crates/pre-styled-ui/src/badge.rs。イシュー #1555 で Surface、#2045 で shadcn/ui `ghost` 相当の Plain を追加）で塗り方を切り替える",
        "colorPalette 軸（badge.rs の BadgeProps）でセマンティック色を選択する",
        "Subtle/Outline/Surface は 6 役割 palette の淡色トークンを消費する（badge.rs、イシュー #1555）",
        "role/aria-* は付与しない最小サブセット（badge.rs モジュール冒頭）",
        "badge::link（イシュー #2045）で `<a>` として組み立てられる。shadcn/ui `link` variant / render prop 相当で、`[href]`/`:focus-visible` state が badge.rs recipe に発火する",
    ],
    arguments: &[
        ArgRow {
            name: "variant",
            kind: "BadgeVariant",
            default: "Subtle",
            description: "塗り方（badge.rs の BadgeVariant、#[default] は Subtle。イシュー #1555 で Surface、#2045 で Plain を追加し 5 値）。",
        },
        ArgRow {
            name: "size",
            kind: "Size",
            default: "Md",
            description: "サイズ（badge.rs の BadgeProps）。",
        },
        ArgRow {
            name: "palette",
            kind: "ColorPalette",
            default: "Accent",
            description: "colorPalette 軸（badge.rs の BadgeProps）。",
        },
    ],
    examples: &[
        ExampleEntry {
            title: "Solid",
            description: "Solid variant の Badge です。",
            render: ex_badge,
        },
        ExampleEntry {
            title: "With icon",
            description: "アイコンを子ノードとして並べた Badge です（イシュー #2045）。",
            render: ex_badge_with_icon,
        },
        ExampleEntry {
            title: "As link",
            description: "badge::link で `<a>` として組み立てた Badge です（イシュー #2045）。",
            render: ex_badge_link,
        },
    ],
    keyboard: &[],
    aria: &[AriaRow {
        attribute: "(該当なし)",
        description: "chakra-ui v3 準拠の最小サブセットとして role/aria-* を付与しない（badge.rs モジュール冒頭）。",
    }],
    demo: None,
};

/// `crates/pre-styled-ui/src/callout.rs:68`（`CalloutVariant` 3 バリアント）・
/// 同 `:300`（`root` が `role`/`aria-*` を一切付与しない）。
fn ex_callout() -> Node {
    let props = callout::CalloutProps {
        variant: callout::CalloutVariant::Surface,
        ..callout::CalloutProps::default()
    };
    callout::root(
        &props,
        vec![],
        vec![
            callout::icon(vec![], vec![]),
            callout::text(vec![], vec![text("Heads up: this is supplementary info")]),
        ],
    )
}

pub(crate) const CALLOUT: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "CalloutVariant（Soft/Surface/Outline、crates/pre-styled-ui/src/callout.rs:68-76）で見た目を切り替える",
        "colorPalette 軸（callout.rs:94-101）でセマンティック色を選択する",
        "size（xs〜xl、callout.rs:94-101）で padding / gap / 角丸 / 文字サイズが root の `--fandhe-callout-*` custom property を通じて連動する（イシュー #1556）",
        "root/icon/text の 3 パーツで補足情報を構造化できる（callout.rs 全文参照）",
        "alert と異なり role を一切付与しない静的な補足表示部品（callout.rs:1-18 module doc）",
    ],
    arguments: &[
        ArgRow {
            name: "variant",
            kind: "CalloutVariant",
            default: "Soft",
            description: "見た目（callout.rs:68-76、#[default] は Soft）。",
        },
        ArgRow {
            name: "size",
            kind: "Size",
            default: "Md",
            description: "サイズ（callout.rs:94-101）。padding / gap / 角丸 / 文字サイズが連動する（root の `--fandhe-callout-*`、イシュー #1556）。",
        },
        ArgRow {
            name: "palette",
            kind: "ColorPalette",
            default: "Accent",
            description: "colorPalette 軸（callout.rs:94-101）。",
        },
    ],
    examples: &[ExampleEntry {
        title: "Surface",
        description: "Surface variant の Callout を icon + text で組み立てた例です。",
        render: ex_callout,
    }],
    keyboard: &[],
    aria: &[AriaRow {
        attribute: "(該当なし)",
        description: "固有の role/aria-* を出力しない。alert と異なり live region ではないため支援技術へ割り込み通知をしない（callout.rs module doc 参照）。",
    }],
    demo: None,
};

fn ex_card() -> Node {
    let props = card::CardProps {
        variant: card::CardVariant::Elevated,
        ..card::CardProps::default()
    };
    card::root(
        props,
        vec![],
        vec![
            card::header(vec![], vec![card::title(vec![], vec![text("Title")])]),
            card::body(vec![], vec![text("Body")]),
            card::footer(vec![], vec![text("Footer")]),
        ],
    )
}

/// イシュー #2046: shadcn/ui 突合で純追加した `action` パーツ
/// （header 右上スロット）の例。`header` へ `data-has-action` を渡すと
/// grid 化され、action が右上 2 行にまたがって配置される。
fn ex_card_action_header() -> Node {
    card::root(
        card::CardProps::default(),
        vec![],
        vec![card::header(
            vec![("data-has-action", "")],
            vec![
                card::title(vec![], vec![text("Team plan")]),
                card::description(vec![], vec![text("3 members")]),
                card::action(
                    vec![],
                    vec![button(
                        &ButtonProps {
                            variant: ButtonVariant::Outline,
                            size: Size::Sm,
                            ..ButtonProps::default()
                        },
                        vec![],
                        vec![text("Manage")],
                    )],
                ),
            ],
        )],
    )
}

/// イシュー #2046: `cover` パーツ（cover image 枠）へ [`image::image`] を
/// 子として渡す合成パターンの例。
fn ex_card_cover_image() -> Node {
    card::root(
        card::CardProps::default(),
        vec![],
        vec![
            card::cover(
                vec![],
                vec![image::image(
                    &image::ImageProps {
                        aspect_ratio: image::AspectRatio::Video,
                        fit: image::ImageFit::Cover,
                        ..image::ImageProps::new(crate::showcase::IMAGE_DEMO_SRC, "cover image")
                    },
                    vec![],
                )],
            ),
            card::body(vec![], vec![card::title(vec![], vec![text("Cover image")])]),
        ],
    )
}

/// イシュー #2046: `data-bordered` opt-in 状態（footer の 1px 区切り線）の
/// 例。shadcn/ui の Header/Footer with Border Examples 相当。
fn ex_card_bordered_footer() -> Node {
    card::root(
        card::CardProps::default(),
        vec![],
        vec![
            card::header(vec![], vec![card::title(vec![], vec![text("Sign in")])]),
            card::body(vec![], vec![text("Email / Password フォームです。")]),
            card::footer(
                vec![("data-bordered", "")],
                vec![button(
                    &ButtonProps {
                        variant: ButtonVariant::Solid,
                        size: Size::Sm,
                        ..ButtonProps::default()
                    },
                    vec![],
                    vec![text("Continue")],
                )],
            ),
        ],
    )
}

pub(crate) const CARD: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "CardVariant（Elevated/Outline/Subtle、crates/pre-styled-ui/src/card.rs:80-106 付近）で見た目を切り替える",
        "size（xs〜xl）で padding / 角丸 / title の文字サイズが root の `--fandhe-card-*` custom property を通じて連動する（イシュー #1557。shadcn/ui の size=\"default\"/\"sm\" はこの 5 段で包含済みと判断し変更していない、イシュー #2046）",
        "header/body/footer/title/description/action/cover の 8 パーツでレイアウトを構造化する（action/cover はイシュー #2046 で shadcn/ui 突合により純追加）",
        "header へ `data-has-action` を渡すと grid 化され、action が右上 2 行にまたがって配置される（shadcn/ui `CardAction` 相当、イシュー #2046）",
        "cover へ image::image を子として渡すと cover image 枠になる（上端 2 角のみ root の角丸に沿ってクリップする、イシュー #2046）",
        "header/footer へ `data-bordered` を渡すと 1px の区切り線が付く opt-in 状態（既定は #1557 のとおり区切り線なし、イシュー #2046）",
        "純粋なレイアウトコンテナのため role/aria-* は付与しない（card.rs 冒頭）",
    ],
    arguments: &[
        ArgRow {
            name: "variant",
            kind: "CardVariant",
            default: "Outline",
            description: "見た目（#[default] は Outline）。",
        },
        ArgRow {
            name: "size",
            kind: "Size",
            default: "Md",
            description: "サイズ。padding / 角丸 / title の文字サイズが連動する（root の `--fandhe-card-*`、イシュー #1557）。",
        },
    ],
    examples: &[
        ExampleEntry {
            title: "Elevated card",
            description: "header/body/footer を組み合わせた Elevated variant の例です。",
            render: ex_card,
        },
        ExampleEntry {
            title: "Card with action",
            description: "header に data-has-action を付け action（button）を右上へ配置する例です。",
            render: ex_card_action_header,
        },
        ExampleEntry {
            title: "Card with cover image",
            description: "cover パーツへ image::image を子として渡す合成パターンです。",
            render: ex_card_cover_image,
        },
        ExampleEntry {
            title: "Card with bordered footer",
            description: "footer に data-bordered を付け 1px の区切り線を出す例です。",
            render: ex_card_bordered_footer,
        },
    ],
    keyboard: &[],
    aria: &[AriaRow {
        attribute: "(該当なし)",
        description: "純粋なレイアウトコンテナであり role/aria-* を付与しない（card.rs 冒頭）。",
    }],
    demo: None,
};

fn ex_data_list() -> Node {
    data_list::root(
        data_list::DataListProps {
            orientation: data_list::DataListOrientation::Horizontal,
            ..data_list::DataListProps::default()
        },
        vec![],
        vec![data_list::item(
            vec![],
            vec![
                data_list::item_label(vec![], vec![text("Name")]),
                data_list::item_value(vec![], vec![text("Alice")]),
            ],
        )],
    )
}

fn ex_data_list_bold() -> Node {
    data_list::root(
        data_list::DataListProps {
            variant: data_list::DataListVariant::Bold,
            ..data_list::DataListProps::default()
        },
        vec![],
        vec![data_list::item(
            vec![],
            vec![
                data_list::item_label(vec![], vec![text("Name")]),
                data_list::item_value(vec![], vec![text("Alice")]),
            ],
        )],
    )
}

pub(crate) const DATA_LIST: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "DataListOrientation（Vertical/Horizontal、crates/pre-styled-ui/src/data_list.rs:106-113）でラベル・値の並びを切り替える",
        "DataListVariant（Subtle/Bold、data_list.rs:129-136、イシュー #1559）でラベル・値の強調配色を切り替える",
        "size（Size::Xs〜Xl、既定 Md、data_list.rs recipe() の size_variants）で gap・font-size を段階的に切り替える",
        "item/item-label/item-value の 3 パーツで dl/dt/dd 構造を組み立てる",
        "orientation/variant/size の伝搬は root の CSS custom property 経由（通常の CSS 継承、data_list.rs モジュール doc）",
    ],
    arguments: &[
        ArgRow {
            name: "orientation",
            kind: "DataListOrientation",
            default: "Vertical",
            description: "並び方向（#[default] は Vertical）。",
        },
        ArgRow {
            name: "variant",
            kind: "DataListVariant",
            default: "Subtle",
            description: "見た目 variant（イシュー #1559。#[default] は Subtle。ラベル muted・値 fg / Bold はラベル fg+medium 太字・値 muted）。",
        },
        ArgRow {
            name: "size",
            kind: "Size",
            default: "Md",
            description: "サイズ variant（イシュー #1559。Xs〜Xl の 5 段、既定 Md）。",
        },
    ],
    examples: &[
        ExampleEntry {
            title: "Horizontal",
            description: "ラベル・値を横並び表示する Horizontal variant の例です。",
            render: ex_data_list,
        },
        ExampleEntry {
            title: "Bold",
            description: "ラベルを強調表示する Bold variant の例です。",
            render: ex_data_list_bold,
        },
    ],
    keyboard: &[],
    aria: &[AriaRow {
        attribute: "(該当なし)",
        description: "dl/dt/dd のネイティブ意味論のみで固有の role/aria-* は出力しない（data_list.rs 全文で role/aria-* を grep しても 0 件）。",
    }],
    demo: None,
};

fn ex_empty_state() -> Node {
    empty_state::root(
        &empty_state::EmptyStateProps::default(),
        vec![],
        vec![empty_state::content(
            vec![],
            vec![
                empty_state::title(vec![], vec![text("No results")]),
                empty_state::description(vec![], vec![text("Try a different search.")]),
            ],
        )],
    )
}

/// イシュー #2047: root `variant`（Outline）の例。
fn ex_empty_state_outline() -> Node {
    empty_state::root(
        &empty_state::EmptyStateProps {
            size: Size::Md,
            variant: empty_state::EmptyStateVariant::Outline,
        },
        vec![],
        vec![empty_state::content(
            vec![],
            vec![
                empty_state::title(vec![], vec![text("No results")]),
                empty_state::description(vec![], vec![text("Try a different search.")]),
            ],
        )],
    )
}

/// イシュー #2047: indicator `variant`（Boxed）の例。
fn ex_empty_state_boxed_indicator() -> Node {
    empty_state::root(
        &empty_state::EmptyStateProps::default(),
        vec![],
        vec![empty_state::content(
            vec![],
            vec![
                empty_state::indicator_with(
                    empty_state::EmptyStateIndicatorVariant::Boxed,
                    vec![],
                    vec![text("∅")],
                ),
                empty_state::title(vec![], vec![text("No projects yet")]),
                empty_state::description(
                    vec![],
                    vec![text("Create your first project to get started.")],
                ),
            ],
        )],
    )
}

pub(crate) const EMPTY_STATE: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "content/indicator/title/description/actions の 5 パーツで空状態の掲示を構造化する（crates/pre-styled-ui/src/empty_state.rs の recipe 関数）",
        "size variant（既定 Md）が root の `--fandhe-empty-state-*` custom property 経由で padding・gap・indicator/title/description の文字サイズを連動させる（empty_state.rs の recipe 関数、イシュー #1560）",
        "root の variant 軸（既定 Plain、Outline/Subtle を純追加）で破線枠・淡色背景を切り替える（イシュー #2047、shadcn/ui `Empty` 突合）",
        "indicator の variant 軸（indicator_with、既定 Plain、Boxed を純追加）で bg-muted の角丸タイル表示を切り替える（イシュー #2047）",
        "aria-* は付与しない（empty_state.rs 冒頭 doc コメント）",
    ],
    arguments: &[
        ArgRow {
            name: "size",
            kind: "Size",
            default: "Md",
            description: "root の custom property（`--fandhe-empty-state-*`）経由で padding・gap・indicator/title/description の文字サイズを連動させるサイズ（empty_state.rs の recipe 関数、イシュー #1560）。",
        },
        ArgRow {
            name: "variant",
            kind: "EmptyStateVariant",
            default: "Plain",
            description: "root の見た目 variant。Plain（既定、class 非出力）/ Outline（破線枠）/ Subtle（淡色単色背景）（イシュー #2047）。",
        },
        ArgRow {
            name: "indicator_with の variant",
            kind: "EmptyStateIndicatorVariant",
            default: "Plain",
            description: "indicator の見た目 variant。Plain（既定、indicator() と同一出力）/ Boxed（bg-muted の角丸タイル）（イシュー #2047）。",
        },
    ],
    examples: &[
        ExampleEntry {
            title: "Basic",
            description: "title + description のみで組み立てた最小構成の例です。",
            render: ex_empty_state,
        },
        ExampleEntry {
            title: "Outline",
            description: "root に破線枠を付ける Outline variant の例です（イシュー #2047）。",
            render: ex_empty_state_outline,
        },
        ExampleEntry {
            title: "Boxed indicator",
            description: "indicator に bg-muted の角丸タイルを付ける Boxed variant の例です（イシュー #2047）。",
            render: ex_empty_state_boxed_indicator,
        },
    ],
    keyboard: &[],
    aria: &[AriaRow {
        attribute: "(該当なし)",
        description: "aria-* は付与しない（empty_state.rs:5、root_has_no_role_attribute 回帰テストあり）。",
    }],
    demo: None,
};

/// JsonTreeView の Demo と同じ構成方針（`crates/docs-site/src/showcase.rs`
/// の `json_tree_view_section` を参考に、より小さいデータで再構築）。
/// `tree_view::root`/`label`/`tree` は json-tree-view/tree-view の両方が
/// 共有する headless anatomy であり、両モジュールとも `showcase.rs` と同じ
/// 呼び出し経路（`json_tree_view::expanded_to_depth`/`render_json` →
/// `tree_view::root`）を使う（json_tree_view.rs:42-49 の re-export 宣言）。
fn ex_json_tree_view() -> Node {
    let data = json_tree_view::JsonValue::Object(vec![
        (
            "name".to_string(),
            json_tree_view::JsonValue::String("fandhe-frontend".to_string()),
        ),
        ("stable".to_string(), json_tree_view::JsonValue::Bool(true)),
    ]);
    let tree = json_tree_view::expanded_to_depth(&data, 1);
    tree_view::root(
        Size::Md,
        vec![],
        vec![
            tree_view::label(vec![], vec![text("Package metadata")]),
            tree_view::tree(
                Some("Package metadata"),
                None,
                vec![],
                vec![json_tree_view::render_json(&tree, &data)],
            ),
        ],
    )
}

pub(crate) const JSON_TREE_VIEW: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "key/colon/value の 3 anatomy パーツ（data-scope=\"json-tree-view\"）を headless-ui からそのまま再エクスポートする。colon はイシュー #1661 で ark-ui/zag 突合により追加した区切りパーツで、pre-styled-ui は意図的に未スタイルのまま提供する（crates/pre-styled-ui/src/json_tree_view.rs:7-10）",
        "expanded_to_depth で ark-ui の defaultExpandedDepth 相当の初期展開状態を決定的に作る（json_tree_view.rs 冒頭 doc、showcase.rs:2057-2058 の利用例）",
        "値の型（string/number/boolean/null/array/object）ごとに [data-scope=\"json-tree-view\"][data-part=\"value\"][data-kind=\"...\"] で配色を切り替える（イシュー #1661 で data-kind の bool 表記を boolean へ変更、json_tree_view.rs:111-113）",
    ],
    arguments: &[],
    examples: &[ExampleEntry {
        title: "Package metadata",
        description: "name/stable の 2 フィールドを持つ JSON を深さ 1 まで展開した例です。",
        render: ex_json_tree_view,
    }],
    keyboard: &[],
    aria: &[AriaRow {
        attribute: "role=\"tree\" / role=\"treeitem\"",
        description: "headless-ui の TreeView anatomy を再利用するため role=\"tree\"/role=\"treeitem\" を持つ（crates/pre-styled-ui/src/tree_view.rs:45 の rustdoc 記述、json_tree_view.rs:7-8 が同 anatomy を再エクスポート）。",
    }],
    demo: None,
};

fn ex_progress() -> Node {
    use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::progress::Progress;
    use fandhe_frontend_pre_styled_ui::progress::ProgressProps;
    let p = Progress::new(0.0, 100.0, Some(65.0), Orientation::Horizontal);
    progress::root(
        &p,
        &ProgressProps::default(),
        Some("65%"),
        vec![],
        vec![
            p.label(vec![], vec![fandhe_frontend_core::text("Upload")]),
            p.value_text(vec![], vec![fandhe_frontend_core::text("65%")]),
            p.track(vec![], vec![progress::range(&p, vec![])]),
        ],
    )
}

fn ex_progress_circle() -> Node {
    use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::progress::Progress;
    use fandhe_frontend_pre_styled_ui::progress::ProgressProps;
    let p = Progress::new(0.0, 100.0, Some(65.0), Orientation::Horizontal);
    progress::root(
        &p,
        &ProgressProps::default(),
        Some("65%"),
        vec![],
        vec![p.circle(
            vec![],
            vec![
                p.circle_track(vec![], vec![]),
                p.circle_range(vec![], vec![]),
            ],
        )],
    )
}

// イシュー #2049: shadcn/ui "Label and Value" の既定表現（枠線なし中立
// トラック + label/value 併記）を、shadcn の TSX/Tailwind をそのまま複製
// せずノード木 API で書き直して再現する（`docs/policy/intentional-non-
// adoption.md` §4 相当の判断、`crates/pre-styled-ui/src/progress.rs`
// rustdoc「イシュー #2049: shadcn/ui との突合」節参照）。
fn ex_progress_plain() -> Node {
    use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::progress::Progress;
    use fandhe_frontend_pre_styled_ui::progress::{ProgressProps, ProgressVariant};
    let p = Progress::new(0.0, 100.0, Some(56.0), Orientation::Horizontal);
    let props = ProgressProps {
        variant: ProgressVariant::Plain,
        ..ProgressProps::default()
    };
    progress::root(
        &p,
        &props,
        Some("56%"),
        vec![("aria-labelledby", "progress-plain-label")],
        vec![
            p.label(
                vec![("id", "progress-plain-label")],
                vec![fandhe_frontend_core::text("Upload progress")],
            ),
            p.value_text(vec![], vec![fandhe_frontend_core::text("56%")]),
            p.track(vec![], vec![progress::range(&p, vec![])]),
        ],
    )
}

// イシュー #1689: #1688 で circle-range の indeterminate（[data-state="indeterminate"]）
// へ固定弧（円周の 1/4、stroke-dasharray）を追加した契約を Themes ページの
// Examples へ反映する。value=None（indeterminate）の circular Progress を
// 単独で掲示し、上の determinate 例（完全なリングにも回転にもならない
// 部分弧）との対比が原稿だけで読み取れるようにする。
fn ex_progress_circle_indeterminate() -> Node {
    use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::progress::Progress;
    use fandhe_frontend_pre_styled_ui::progress::ProgressProps;
    let p = Progress::new(0.0, 100.0, None, Orientation::Horizontal);
    progress::root(
        &p,
        &ProgressProps::default(),
        None,
        vec![],
        vec![p.circle(
            vec![],
            vec![
                p.circle_track(vec![], vec![]),
                p.circle_range(vec![], vec![]),
            ],
        )],
    )
}

pub(crate) const PROGRESS: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "track/range（linear）と circle/circle-track/circle-range（circular）はいずれも headless の inherent メソッドをそのまま呼ばせる契約（crates/pre-styled-ui/src/progress.rs テスト caller_headless_track_and_circle_parts_render_without_wrapper）",
        "value が None（indeterminate）のとき [data-state=\"indeterminate\"] でアニメーション（linear は横スライド・circular は回転）を付与し、prefers-reduced-motion: reduce で停止する",
        "ProgressProps（size/variant/color-palette の 3 軸）を root へ付与する。styled range() が --fandhe-progress-percent を determinate 時のみ付与する",
        "circle-range の [data-state=\"indeterminate\"] へ固定長の弧（--fandhe-progress-circumference = 2πr、stroke-dasharray で円周の 1/4）を与え、circle の回転と組み合わせて complete（完全リング）と視覚的に区別する。新規 @keyframes は追加せず reduced-motion 下でも弧が残る（crates/pre-styled-ui/src/progress.rs rustdoc「イシュー #1688: circle-range indeterminate の固定弧」節・テスト circle_range_indeterminate_state_declares_fixed_arc_dasharray）",
        "ProgressVariant::Plain（枠線なしの中立トラック）は shadcn/ui 既定表現を突合して補完した variant（イシュー #2049）。既存 Outline/Subtle の CSS 出力・既定 variant はバイト不変（crates/pre-styled-ui/src/progress.rs rustdoc「イシュー #2049: shadcn/ui との突合」節）",
    ],
    arguments: &[
        ArgRow {
            name: "props",
            kind: "&ProgressProps",
            default: "ProgressProps::default()",
            description: "size（既定 Md）/variant（既定 Outline、Outline/Subtle/Plain の 3 値）/palette（既定 Accent）の 3 軸をまとめた設定（progress.rs）。",
        },
        ArgRow {
            name: "aria_valuetext",
            kind: "Option<&str>",
            default: "None",
            description: "aria-valuetext へ渡す表示用テキスト（progress.rs）。",
        },
    ],
    examples: &[
        ExampleEntry {
            title: "Determinate (linear)",
            description: "value=65 の determinate linear progress（Track/Range）の例です。",
            render: ex_progress,
        },
        ExampleEntry {
            title: "Determinate (circular)",
            description: "value=65 の determinate circular progress（SVG）の例です。",
            render: ex_progress_circle,
        },
        ExampleEntry {
            title: "Indeterminate (circular)",
            description: "value=None の indeterminate circular progress の例です。circle 全体の回転に加え、circle-range へ円周の 1/4 分の固定弧（stroke-dasharray）を与え、complete（完全なリング）と区別します（イシュー #1688）。",
            render: ex_progress_circle_indeterminate,
        },
        ExampleEntry {
            title: "Label + Value (Plain variant)",
            description: "shadcn/ui 既定表現（枠線なしの中立トラック）に相当する ProgressVariant::Plain の例です。label + value 併記で shadcn の \"Label and Value\" 例をノード木 API で再現しています（イシュー #2049）。",
            render: ex_progress_plain,
        },
    ],
    keyboard: &[],
    aria: &[AriaRow {
        attribute: "aria-valuetext",
        description: "呼び出し側が渡した aria_valuetext を root（headless progress.root への委譲）へ出力する（progress.rs）。",
    }],
    demo: None,
};

fn ex_skeleton() -> Node {
    skeleton::skeleton(
        &skeleton::SkeletonProps {
            variant: skeleton::SkeletonVariant::Circle,
            ..Default::default()
        },
        vec![],
    )
}

/// shadcn/ui Avatar 例に相当（イシュー #2050）。Circle（サイズ上書き）+
/// 縦積みの Text 2 本を横並びで組み合わせる。追加 API 不要、`style` 属性の
/// 上書きのみで再現できる。
fn ex_skeleton_avatar() -> Node {
    div(
        vec![("style", "display: flex; align-items: center; gap: 1rem;")],
        vec![
            skeleton::skeleton(
                &skeleton::SkeletonProps {
                    variant: skeleton::SkeletonVariant::Circle,
                    ..Default::default()
                },
                vec![("style", "--fandhe-skeleton-size: 2.5rem;")],
            ),
            div(
                vec![(
                    "style",
                    "display: flex; flex-direction: column; gap: 0.5rem;",
                )],
                vec![
                    skeleton::skeleton(
                        &skeleton::SkeletonProps::default(),
                        vec![("style", "width: 9.5rem;")],
                    ),
                    skeleton::skeleton(
                        &skeleton::SkeletonProps::default(),
                        vec![("style", "width: 6.5rem;")],
                    ),
                ],
            ),
        ],
    )
}

/// shadcn/ui Card 例に相当（イシュー #2050）。`card::root`/`card::header`/
/// `card::body` と組み合わせ、Rect の `--fandhe-skeleton-height` を `auto`
/// へ上書きして `aspect-ratio` を効かせる。
fn ex_skeleton_card() -> Node {
    card::root(
        card::CardProps::default(),
        vec![("style", "max-width: 20rem;")],
        vec![
            card::header(
                vec![],
                vec![
                    skeleton::skeleton(
                        &skeleton::SkeletonProps::default(),
                        vec![("style", "width: 66%;")],
                    ),
                    skeleton::skeleton(
                        &skeleton::SkeletonProps::default(),
                        vec![("style", "width: 50%;")],
                    ),
                ],
            ),
            card::body(
                vec![],
                vec![skeleton::skeleton(
                    &skeleton::SkeletonProps {
                        variant: skeleton::SkeletonVariant::Rect,
                        ..Default::default()
                    },
                    vec![(
                        "style",
                        "aspect-ratio: 16 / 9; --fandhe-skeleton-height: auto;",
                    )],
                )],
            ),
        ],
    )
}

/// shadcn/ui Text 例に相当（イシュー #2050）。テキスト 3 本、最終行のみ
/// 幅を詰めて段落末尾らしさを表現する。
fn ex_skeleton_text() -> Node {
    div(
        vec![(
            "style",
            "display: flex; flex-direction: column; gap: 0.5rem; max-width: 20rem;",
        )],
        vec![
            skeleton::skeleton(&skeleton::SkeletonProps::default(), vec![]),
            skeleton::skeleton(&skeleton::SkeletonProps::default(), vec![]),
            skeleton::skeleton(
                &skeleton::SkeletonProps::default(),
                vec![("style", "width: 75%;")],
            ),
        ],
    )
}

/// shadcn/ui Form 例に相当（イシュー #2050）。「ラベル + 入力」の Rect
/// 組を 2 つ縦積みし、末尾にボタン相当の Rect を置く。
fn ex_skeleton_form() -> Node {
    fn field(label_style: &'static str) -> Node {
        div(
            vec![(
                "style",
                "display: flex; flex-direction: column; gap: 0.5rem;",
            )],
            vec![
                skeleton::skeleton(
                    &skeleton::SkeletonProps::default(),
                    vec![("style", label_style)],
                ),
                skeleton::skeleton(
                    &skeleton::SkeletonProps {
                        variant: skeleton::SkeletonVariant::Rect,
                        ..Default::default()
                    },
                    vec![("style", "--fandhe-skeleton-height: 2rem;")],
                ),
            ],
        )
    }
    div(
        vec![(
            "style",
            "display: flex; flex-direction: column; gap: 1.75rem; max-width: 20rem;",
        )],
        vec![
            field("width: 5rem;"),
            field("width: 6rem;"),
            skeleton::skeleton(
                &skeleton::SkeletonProps {
                    variant: skeleton::SkeletonVariant::Rect,
                    ..Default::default()
                },
                vec![("style", "width: 6rem; --fandhe-skeleton-height: 2rem;")],
            ),
        ],
    )
}

/// shadcn/ui Table 例に相当（イシュー #2050）。shadcn の実例どおり
/// `table` 部品は使わず、`flex` 行 5 本 × Text 3 本で再現する。
fn ex_skeleton_table() -> Node {
    fn row() -> Node {
        div(
            vec![("style", "display: flex; gap: 1rem;")],
            vec![
                skeleton::skeleton(
                    &skeleton::SkeletonProps::default(),
                    vec![("style", "flex: 1;")],
                ),
                skeleton::skeleton(
                    &skeleton::SkeletonProps::default(),
                    vec![("style", "width: 6rem;")],
                ),
                skeleton::skeleton(
                    &skeleton::SkeletonProps::default(),
                    vec![("style", "width: 5rem;")],
                ),
            ],
        )
    }
    div(
        vec![(
            "style",
            "display: flex; flex-direction: column; gap: 0.5rem; max-width: 24rem;",
        )],
        vec![row(), row(), row(), row(), row()],
    )
}

pub(crate) const SKELETON: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "SkeletonVariant（Text/Circle/Rect、crates/pre-styled-ui/src/skeleton.rs:192-200）で占位形状を切り替える",
        "SkeletonAnimation（Pulse/Shine/None、skeleton.rs:223-233、イシュー #1566）で第 2 軸のアニメーション種別を切り替える",
        "常に aria-hidden=\"true\" を固定付与する（skeleton.rs:420）",
        "呼び出し側が偽装した aria-hidden（大文字小文字問わず）も除去し常時 true へ一本化する（skeleton.rs:420、回帰テストは skeleton.rs:484）",
        "イシュー #2050 で shadcn/ui と突合、欠落 variant/state なし。shimmer は text 向け utility のため不採用、Card/Text/Form/Table 合成例を Examples へ追加",
    ],
    arguments: &[
        ArgRow {
            name: "variant",
            kind: "SkeletonVariant",
            default: "Text",
            description: "占位形状（skeleton.rs:192-200、#[default] は Text）。",
        },
        ArgRow {
            name: "animation",
            kind: "SkeletonAnimation",
            default: "Pulse",
            description: "アニメーション種別（skeleton.rs:223-233、#[default] は Pulse、イシュー #1566）。",
        },
    ],
    examples: &[
        ExampleEntry {
            title: "Circle",
            description: "アバター等の占位に使う Circle variant の例です。",
            render: ex_skeleton,
        },
        ExampleEntry {
            title: "Avatar",
            description: "shadcn/ui の Avatar 例に相当。Circle + 縦積み Text 2 本を横並びで組み合わせ、追加 API 不要で style 上書きのみで再現できます。",
            render: ex_skeleton_avatar,
        },
        ExampleEntry {
            title: "Card",
            description: "shadcn/ui の Card 例に相当。card::header/card::body と組み合わせ、Rect の --fandhe-skeleton-height を auto へ上書きして aspect-ratio を効かせます。",
            render: ex_skeleton_card,
        },
        ExampleEntry {
            title: "Text",
            description: "shadcn/ui の Text 例に相当。テキスト 3 本、最終行のみ幅を詰めて段落末尾らしさを表現します。",
            render: ex_skeleton_text,
        },
        ExampleEntry {
            title: "Form",
            description: "shadcn/ui の Form 例に相当。ラベル + 入力の Rect 組を 2 つ縦積みし、末尾にボタン相当の Rect を置きます。",
            render: ex_skeleton_form,
        },
        ExampleEntry {
            title: "Table",
            description: "shadcn/ui の Table 例に相当。table 部品は使わず、flex 行 5 本 × Text 3 本で再現します。",
            render: ex_skeleton_table,
        },
    ],
    keyboard: &[],
    aria: &[AriaRow {
        attribute: "aria-hidden=\"true\"",
        description: "常に固定付与される（呼び出し側の偽装値は除去、skeleton.rs:420、回帰テストは skeleton.rs:484）。",
    }],
    demo: None,
};

fn ex_spinner() -> Node {
    spinner::spinner(&spinner::SpinnerProps {
        label: "Loading products",
        ..spinner::SpinnerProps::default()
    })
}

/// shadcn/ui の Button 末尾配置例に相当（イシュー #2051）。`ButtonProps::
/// loading` は Spinner を子ノード先頭へ固定配置するため、末尾配置は
/// `spinner::spinner_decorative`（#2051 で公開 API 化）を呼び出し側が
/// `children` 末尾へ直接組み込むことで再現する。ボタン自身の状態伝達は
/// `aria-busy="true"`（`attrs` 経由で付与）が担うため、Spinner 側は
/// `aria-hidden="true"` の装飾専用のまま冗長なライブリージョンを重ねない。
fn ex_spinner_button() -> Node {
    div(
        vec![("style", "display: flex; gap: 0.75rem; flex-wrap: wrap;")],
        vec![
            button(
                &ButtonProps {
                    loading: true,
                    ..ButtonProps::default()
                },
                vec![],
                vec![text("Loading...")],
            ),
            button(
                &ButtonProps {
                    variant: ButtonVariant::Outline,
                    disabled: true,
                    ..ButtonProps::default()
                },
                vec![("aria-busy", "true")],
                vec![
                    text("Processing"),
                    spinner::spinner_decorative(Size::Sm, ColorPalette::Accent),
                ],
            ),
        ],
    )
}

/// shadcn/ui の Badge 合成例に相当（イシュー #2051）。装飾用途の
/// `spinner_decorative` をラベルテキストの前へ並べる。Badge 自体は
/// `role`/`aria-*` を持たないため、状態伝達は周囲のテキストが担う。
fn ex_spinner_badge() -> Node {
    div(
        vec![("style", "display: flex; gap: 0.5rem; flex-wrap: wrap;")],
        vec![
            badge::badge(
                &badge::BadgeProps {
                    variant: badge::BadgeVariant::Subtle,
                    ..badge::BadgeProps::default()
                },
                vec![],
                vec![
                    spinner::spinner_decorative(Size::Xs, ColorPalette::Accent),
                    text("Syncing"),
                ],
            ),
            badge::badge(
                &badge::BadgeProps {
                    variant: badge::BadgeVariant::Outline,
                    palette: ColorPalette::Info,
                    ..badge::BadgeProps::default()
                },
                vec![],
                vec![
                    spinner::spinner_decorative(Size::Xs, ColorPalette::Info),
                    text("Updating"),
                ],
            ),
        ],
    )
}

/// shadcn/ui の Empty state 合成例に相当（イシュー #2051）。`indicator` に
/// `role="status"` 付きの [`spinner::spinner`] をそのまま置き、title/
/// description/actions で処理内容とキャンセル導線を伝える。
fn ex_spinner_empty() -> Node {
    empty_state::root(
        &empty_state::EmptyStateProps::default(),
        vec![],
        vec![empty_state::content(
            vec![],
            vec![
                empty_state::indicator(
                    vec![],
                    vec![spinner::spinner(&spinner::SpinnerProps {
                        size: Size::Lg,
                        label: "Processing",
                        ..spinner::SpinnerProps::default()
                    })],
                ),
                empty_state::title(vec![], vec![text("Processing your request")]),
                empty_state::description(vec![], vec![text("This may take a few moments.")]),
                empty_state::actions(
                    vec![],
                    vec![button(
                        &ButtonProps {
                            variant: ButtonVariant::Outline,
                            ..ButtonProps::default()
                        },
                        vec![],
                        vec![text("Cancel")],
                    )],
                ),
            ],
        )],
    )
}

pub(crate) const SPINNER: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "role=\"status\" + aria-label（既定 \"Loading\"）でスクリーンリーダーへ読み込み中を伝える（crates/pre-styled-ui/src/spinner.rs）",
        "spinner_decorative は role/aria-label を持たず aria-hidden=\"true\" のみを付与する公開 API（Button 末尾配置・Badge・Input Group 等の合成用途、イシュー #2051 で pub(crate) から公開化、spinner.rs）",
        "size・colorPalette の 2 軸でサイズとセマンティック色を選択する（spinner.rs 冒頭）",
        "上・右 2 辺の弧で描画し、トラックは既定で透明（イシュー #1567、chakra-ui 基準）。--fandhe-spinner-track-color / --fandhe-spinner-thickness / --fandhe-spinner-duration の custom property で線色・線幅・回転速度を上書きできる",
        "style=\"--fandhe-palette: currentColor\" を指定すると shadcn/ui 相当の親文字色追随を既存 API のまま再現できる（イシュー #2051）",
        "prefers-reduced-motion: reduce 環境では回転アニメーションを停止する（イシュー #1567）",
    ],
    arguments: &[
        ArgRow {
            name: "size",
            kind: "Size",
            default: "Md",
            description: "サイズ variant（spinner.rs）。chakra-ui 5 段基準（xs=0.75rem/sm=1rem/md=1.25rem/lg=2rem/xl=2.5rem、イシュー #1567）。",
        },
        ArgRow {
            name: "palette",
            kind: "ColorPalette",
            default: "Accent",
            description: "colorPalette 軸（spinner.rs、イシュー #606）。",
        },
        ArgRow {
            name: "label",
            kind: "&str",
            default: "\"Loading\"",
            description: "aria-label に渡すラベル文字列（spinner.rs）。",
        },
    ],
    examples: &[
        ExampleEntry {
            title: "Custom label",
            description: "aria-label をカスタマイズした Spinner の例です。",
            render: ex_spinner,
        },
        ExampleEntry {
            title: "In button",
            description: "shadcn/ui の Button 先頭/末尾配置例に相当。先頭は ButtonProps::loading、末尾は spinner_decorative を children へ直接組み込みます。",
            render: ex_spinner_button,
        },
        ExampleEntry {
            title: "In badge",
            description: "shadcn/ui の Badge 合成例に相当。spinner_decorative をラベルテキストの前へ並べます。",
            render: ex_spinner_badge,
        },
        ExampleEntry {
            title: "In empty state",
            description: "shadcn/ui の Empty state 合成例に相当。indicator に role=\"status\" 付きの spinner を置きます。",
            render: ex_spinner_empty,
        },
    ],
    keyboard: &[],
    aria: &[AriaRow {
        attribute: "role=\"status\" + aria-label",
        description: "読み込み中であることをスクリーンリーダーへ伝える（spinner.rs）。",
    }],
    demo: None,
};

fn ex_stat() -> Node {
    stat::root(
        Size::Md,
        vec![],
        vec![
            stat::label(vec![], vec![text("Revenue")]),
            stat::value_text(
                vec![],
                vec![text("1,234"), stat::value_unit(vec![], vec![text("USD")])],
            ),
            stat::help_text(vec![], vec![stat::up_indicator(vec![]), text("12%")]),
        ],
    )
}

pub(crate) const STAT: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "label/value-text/value-unit/help-text/up-indicator/down-indicator の 6 パーツで指標表示を構造化する（crates/pre-styled-ui/src/stat.rs:314-362）",
        "up-indicator/down-indicator は装飾用途のため aria-hidden=\"true\" を固定付与する（stat.rs:17, 348-362）",
        "呼び出し側が aria-hidden を渡してもフレームワーク値の後に連結される（stat.rs:442-449）",
    ],
    arguments: &[ArgRow {
        name: "size",
        kind: "Size",
        default: "Md",
        description: "root（dl）のサイズ（xs〜xl、既定 md。chakra-ui の sm/md/lg は本実装の Sm/Md/Lg に対応、stat.rs:158-256）。",
    }],
    examples: &[ExampleEntry {
        title: "Revenue",
        description: "value-unit と up-indicator を組み合わせた指標表示の例です。",
        render: ex_stat,
    }],
    keyboard: &[],
    aria: &[AriaRow {
        attribute: "aria-hidden=\"true\"（up-indicator/down-indicator のみ）",
        description: "装飾用途の増減インジケータに固定付与される（stat.rs:17, 348-362）。root/label/value-text 自体は固有の ARIA を出力しない。",
    }],
    demo: None,
};

fn ex_status() -> Node {
    status::root(
        &status::StatusProps {
            palette: ColorPalette::Success,
            ..status::StatusProps::default()
        },
        vec![],
        vec![status::indicator(vec![]), text("Online")],
    )
}

pub(crate) const STATUS: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "colorPalette 軸（status.rs の `StatusProps::palette` フィールド）でセマンティック色を選択する",
        "size 軸（Xs〜Xl、status.rs の `StatusProps::size` フィールド）でドット径（--fandhe-status-dot-size）と文字サイズが連動する",
        "root/indicator の 2 パーツのみで構成する最小部品",
        "role=\"status\"（WAI-ARIA live region）は付与しない設計（status.rs のクレート先頭 doc コメント、非同期の状態遷移がある場合は呼び出し側が明示的に足す）",
    ],
    arguments: &[
        ArgRow {
            name: "size",
            kind: "Size",
            default: "Md",
            description: "サイズ軸（status.rs の `StatusProps::size` フィールド）。ドット径と文字サイズが連動する。",
        },
        ArgRow {
            name: "palette",
            kind: "ColorPalette",
            default: "Accent",
            description: "colorPalette 軸（status.rs の `StatusProps::palette` フィールド）。",
        },
    ],
    examples: &[ExampleEntry {
        title: "Success",
        description: "Success パレットで \"Online\" を表示する例です。",
        render: ex_status,
    }],
    keyboard: &[],
    aria: &[AriaRow {
        attribute: "(該当なし)",
        description: "role=\"status\" は付与しない設計。非同期の状態遷移を伴う場合は呼び出し側が明示的に role/aria-live を足す契約（status.rs のクレート先頭 doc コメント、テストで role= の非出現を固定）。",
    }],
    demo: None,
};

fn ex_table() -> Node {
    table::root(
        table::TableProps {
            striped: true,
            ..table::TableProps::default()
        },
        vec![],
        vec![
            table::header(
                vec![],
                vec![table::row(
                    vec![],
                    vec![table::column_header(vec![], vec![text("Name")])],
                )],
            ),
            table::body(
                vec![],
                vec![table::row(
                    vec![],
                    vec![table::cell(vec![], vec![text("Alice")])],
                )],
            ),
        ],
    )
}

fn ex_table_selectable_rows() -> Node {
    table::root(
        table::TableProps {
            interactive: true,
            ..table::TableProps::default()
        },
        vec![],
        vec![
            table::header(
                vec![],
                vec![table::row(
                    vec![],
                    vec![table::column_header(vec![], vec![text("Name")])],
                )],
            ),
            table::body(
                vec![],
                vec![
                    table::row(
                        vec![("data-selected", "")],
                        vec![table::cell(vec![], vec![text("Alice")])],
                    ),
                    table::row(vec![], vec![table::cell(vec![], vec![text("Bob")])]),
                ],
            ),
        ],
    )
}

fn ex_table_aligned_footer() -> Node {
    table::root(
        table::TableProps::default(),
        vec![],
        vec![
            table::header(
                vec![],
                vec![table::row(
                    vec![],
                    vec![
                        table::column_header(vec![], vec![text("Invoice")]),
                        table::column_header(vec![("data-align", "end")], vec![text("Amount")]),
                    ],
                )],
            ),
            table::body(
                vec![],
                vec![table::row(
                    vec![],
                    vec![
                        table::cell(vec![], vec![text("INV001")]),
                        table::cell(vec![("data-align", "end")], vec![text("$250.00")]),
                    ],
                )],
            ),
            table::footer(
                vec![],
                vec![table::row(
                    vec![],
                    vec![
                        table::cell(vec![], vec![text("Total")]),
                        table::cell(vec![("data-align", "end")], vec![text("$250.00")]),
                    ],
                )],
            ),
        ],
    )
}

fn ex_table_scroll_area() -> Node {
    table::scroll_area(
        vec![("style", "--fandhe-table-scroll-max-height: 8rem")],
        vec![table::root(
            table::TableProps {
                sticky_header: true,
                ..table::TableProps::default()
            },
            vec![],
            vec![
                table::header(
                    vec![],
                    vec![table::row(
                        vec![],
                        vec![table::column_header(vec![], vec![text("Name")])],
                    )],
                ),
                table::body(
                    vec![],
                    vec![
                        table::row(vec![], vec![table::cell(vec![], vec![text("Alice")])]),
                        table::row(vec![], vec![table::cell(vec![], vec![text("Bob")])]),
                        table::row(vec![], vec![table::cell(vec![], vec![text("Carol")])]),
                    ],
                ),
            ],
        )],
    )
}

pub(crate) const TABLE: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "TableVariant（Line/Outline、crates/pre-styled-ui/src/table.rs:172-186）で外枠・区切り線を切り替える（イシュー #1572 で Outline の行罫線・ヘッダー背景を chakra-ui/Radix Themes 基準へ是正）",
        "size（Size、Xs〜Xl の 5 段）でセルの padding/font-size を切り替える（padding は --fandhe-space-* トークン、イシュー #1572）",
        "striped（bool）で本文行の背景を交互に変える（table.rs「striped の実装」節）",
        "sticky_header（bool、イシュー #1571）で column-header（th）を position: sticky にする（table.rs「sticky ヘッダーの実装」節）",
        "interactive（bool、イシュー #2052、shadcn/ui 突合）で row に bg-muted の hover 背景を付ける（opt-in、chakra-ui interactive 由来。table.rs「interactive 軸（行 hover）」節）",
        "data-selected（呼び出し側が付与する共有語彙、イシュー #2052）で row を選択状態（accent-subtle/accent-fg-subtle）にする（table.rs「data-selected 行状態」節）",
        "data-align（呼び出し側が付与する共有語彙、start/center/end、イシュー #2052）で cell/column-header の text-align を切り替える（headless positioning.rs と共有する既存語彙、table.rs「data-align セル整列」節）",
        "scroll_area（イシュー #1572、chakra Table.ScrollArea 相当）で root を overflow: auto のスクロール枠に包み、sticky_header と組み合わせて見出し行を固定できる（table.rs「scroll-area パーツ」節）",
        "caption は font-weight: medium・font-size: xs・text-align: inherit（chakra-ui 基準、イシュー #1572）",
        "column_header は scope=\"col\" を関数側で固定し呼び出し側の偽装を除去する（table.rs セキュリティ不変条件節、COLUMN_HEADER_RESERVED）",
        "column_header/cell は aria-sort 等の呼び出し側属性をそのまま通過させる（ソート・選択の生産は headless data-table〔#2124〕の責務、table.rs「スコープ外」節）",
    ],
    arguments: &[
        ArgRow {
            name: "variant",
            kind: "TableVariant",
            default: "Line",
            description: "外枠・区切り線の見た目（table.rs「variant について」節）。",
        },
        ArgRow {
            name: "size",
            kind: "Size",
            default: "Md",
            description: "セルの padding/font-size（table.rs「variant について」節）。",
        },
        ArgRow {
            name: "striped",
            kind: "bool",
            default: "false",
            description: "縞模様表示の有無（table.rs「striped の実装」節）。",
        },
        ArgRow {
            name: "sticky_header",
            kind: "bool",
            default: "false",
            description: "column-header（th）を position: sticky にする（イシュー #1571、table.rs「sticky ヘッダーの実装」節）。",
        },
        ArgRow {
            name: "interactive",
            kind: "bool",
            default: "false",
            description: "row に bg-muted の hover 背景を付ける（イシュー #2052、table.rs「interactive 軸（行 hover）」節）。",
        },
    ],
    examples: &[
        ExampleEntry {
            title: "Striped",
            description: "striped=true・header/body を組み合わせた例です。",
            render: ex_table,
        },
        ExampleEntry {
            title: "Selectable rows",
            description: "interactive=true と行の data-selected を組み合わせた例です（イシュー #2052）。",
            render: ex_table_selectable_rows,
        },
        ExampleEntry {
            title: "Aligned total footer",
            description: "column_header/cell の data-align=\"end\" で金額列を右寄せし、footer に合計行を持たせた例です（イシュー #2052）。",
            render: ex_table_aligned_footer,
        },
        ExampleEntry {
            title: "Scroll area",
            description: "scroll_area で包み、sticky_header=true と組み合わせた例です（イシュー #1572）。",
            render: ex_table_scroll_area,
        },
    ],
    keyboard: &[],
    aria: &[
        AriaRow {
            attribute: "scope=\"col\"（column-header）",
            description: "column_header が固定付与するテーブル見出しの意味論属性（呼び出し側の偽装は除去、table.rs セキュリティ不変条件節）。role/aria-* 自体はネイティブ table 要素の意味論に委ねており固有の出力はない。",
        },
        AriaRow {
            attribute: "aria-sort（column-header、呼び出し側付与）",
            description: "ソート状態の意味論属性。table.rs は通過させるのみで生産しない（生産は headless data-table〔#2124〕の責務、イシュー #2052、table.rs「スコープ外」節）。",
        },
    ],
    demo: None,
};

fn ex_tag() -> Node {
    tag::root(
        &tag::TagProps {
            variant: tag::TagVariant::Outline,
            ..tag::TagProps::default()
        },
        vec![],
        vec![
            tag::label(vec![], vec![text("beta")]),
            tag::close_trigger(Some("Remove beta tag"), vec![], vec![]),
        ],
    )
}

pub(crate) const TAG: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "TagVariant（Solid/Subtle/Outline/Surface、crates/pre-styled-ui/src/tag.rs。イシュー #1573 で Surface を追加）で塗り方を切り替える",
        "colorPalette 軸（tag.rs の TagProps）でセマンティック色を選択する",
        "Subtle/Outline/Surface は 6 役割 palette の淡色トークンを消費する（tag.rs、イシュー #1573）",
        "close_trigger（<button type=\"button\">）で削除可能な Tag を構成できる（tag.rs）",
        "close_trigger は hover 面・キーボードフォーカスリングを持つ（イシュー #1573。Solid variant ではリング色を --fandhe-palette-fg へ切り替え、背景との同化を避ける）",
        "close_trigger 自体は children/aria-label を持たないため呼び出し側が視覚内容とアクセシブルネームを渡す責務を持つ（tag.rs）",
    ],
    arguments: &[
        ArgRow {
            name: "variant",
            kind: "TagVariant",
            default: "Subtle",
            description: "塗り方（tag.rs の TagVariant、#[default] は Subtle。イシュー #1573 で Surface を追加）。",
        },
        ArgRow {
            name: "size",
            kind: "Size",
            default: "Md",
            description: "サイズ（tag.rs の TagProps）。",
        },
        ArgRow {
            name: "palette",
            kind: "ColorPalette",
            default: "Accent",
            description: "colorPalette 軸（tag.rs の TagProps）。",
        },
    ],
    examples: &[ExampleEntry {
        title: "Removable",
        description: "close_trigger を付与した削除可能な Tag の例です。",
        render: ex_tag,
    }],
    keyboard: &[],
    aria: &[AriaRow {
        attribute: "(呼び出し側責務)",
        description: "close_trigger の aria-label・視覚内容（×等）は呼び出し側が渡す責務であり、本部品自体は固有の ARIA を固定出力しない（tag.rs）。",
    }],
    demo: None,
};

fn ex_tree_view() -> Node {
    use fandhe_frontend_pre_styled_ui::fandhe_frontend_interactive::dispatch;
    let mut view = tree_view::TreeView::default();
    dispatch(&mut view, "expand", "src");
    let nodes = vec![tree_view::TreeNode::new("src", "src")
        .with_children(vec![tree_view::TreeNode::new("src/lib.rs", "lib.rs")])];
    let children = view.render_nodes(&nodes);
    tree_view::root(
        Size::Md,
        vec![],
        vec![
            tree_view::label(vec![], vec![text("Project files")]),
            tree_view::tree(Some("Project files"), None, vec![], children),
        ],
    )
}

pub(crate) const TREE_VIEW: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "role=\"tree\"/role=\"treeitem\" の WAI-ARIA Tree パターンを headless-ui が提供する（crates/pre-styled-ui/src/tree_view.rs:45）",
        "branch-content の展開・折りたたみは [hidden] 属性と CSS の詳細度制御で表現する（tree_view.rs 参照）",
        "disabled/selected 状態を data-disabled/data-selected で表現しフォーカス可視化と連動する（tree_view.rs 参照）",
        "size（xs/sm/md/lg/xl、既定 md）が行密度・文字サイズを切り替える。root スコープの CSS custom property 経由で子孫パーツへ継承される（イシュー #1578）",
        "hover 面・キーボードフォーカスリングを持つ（イシュー #1578。選択行の背景は hover で洗い流されない）",
    ],
    arguments: &[ArgRow {
        name: "size",
        kind: "Size",
        default: "Md",
        description: "行密度・文字サイズを切り替える（tree_view.rs の root）。",
    }],
    examples: &[ExampleEntry {
        title: "Expanded branch",
        description: "\"src\" ブランチを展開した状態で固定表示する例です。",
        render: ex_tree_view,
    }],
    keyboard: &[],
    aria: &[AriaRow {
        attribute: "role=\"tree\" / role=\"treeitem\"",
        description: "WAI-ARIA Tree View パターンに従い headless-ui が付与する（tree_view.rs:45）。",
    }],
    demo: None,
};

// ---------------------------------------------------------------------
// Data Display（設計 §5 規則 4 による追加 4 件）
// ---------------------------------------------------------------------

fn ex_color_swatch() -> Node {
    color_swatch::color_swatch(
        &color_swatch::ColorSwatchProps {
            value: color_swatch::Color::from_rgb(color_swatch::Rgb::new(0x3b, 0x82, 0xf6)),
            shape: color_swatch::SwatchShape::Circle,
            ..color_swatch::ColorSwatchProps::default()
        },
        vec![],
        vec![],
    )
}

pub(crate) const COLOR_SWATCH: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "SwatchShape（Square/Circle/Rounded）で外形を切り替える（color_swatch.rs `recipe` 節）",
        "検証済み Color 型のみを受け取り、--fd-swatch-color custom property 経由で色を出力する（color_swatch.rs モジュール冒頭「色値は検証済み型経由のみ」節）",
        "呼び出し側の class/style/data-scope/data-part 偽装はすべて除去する（color_swatch.rs テスト caller_class_and_style_attrs_are_dropped_not_duplicated 等）",
        "内側 1px の輪郭リング（box-shadow: inset）で淡色・低アルファ色でも外形が判別できる（イシュー #1558、color_swatch.rs モジュール冒頭「参照サイト比較」節）",
    ],
    arguments: &[
        ArgRow {
            name: "value",
            kind: "Color",
            default: "opaque black",
            description: "表示する色（検証済み型のみ受け取る、color_swatch.rs モジュール冒頭「色値は検証済み型経由のみ」節）。",
        },
        ArgRow {
            name: "size",
            kind: "Size",
            default: "Md",
            description: "サイズ（Xs〜Xl の 5 段、chakra-ui 同名段の実寸に整合。イシュー #1558）。",
        },
        ArgRow {
            name: "shape",
            kind: "SwatchShape",
            default: "Rounded",
            description: "外形（color_swatch.rs `SwatchShape` 節）。",
        },
    ],
    examples: &[ExampleEntry {
        title: "Circle",
        description: "円形の ColorSwatch の例です。",
        render: ex_color_swatch,
    }],
    keyboard: &[],
    aria: &[AriaRow {
        attribute: "(該当なし)",
        description: "純粋な色見本表示であり固有の role/aria-* を出力しない（color_swatch.rs 全文で role/aria-* を grep しても 0 件）。",
    }],
    demo: None,
};

fn ex_icon() -> Node {
    icon::icon(
        &icon::IconProps {
            label: Some("Search"),
            ..icon::IconProps::default()
        },
        vec![],
        vec![el("path", vec![("d", "M12 2L2 22h20z")], vec![])],
    )
}

pub(crate) const ICON: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "label が Some のとき role=\"img\" + aria-label を付与し意味のあるアイコンとして扱う（icon.rs `icon` 節）",
        "label が None（既定）のとき装飾用途とみなし aria-hidden=\"true\" を付与する（icon.rs `icon` 節）",
        "fill=\"currentColor\" を固定付与し祖先の color プロパティで着色する（icon.rs `icon` 節）",
    ],
    arguments: &[
        ArgRow {
            name: "size",
            kind: "Size",
            default: "Md",
            description: "サイズ（Xs〜Xl の 5 段、chakra-ui 同名段の実寸に整合。イシュー #1561）。",
        },
        ArgRow {
            name: "label",
            kind: "Option<&str>",
            default: "None",
            description: "アクセシブルネーム（icon.rs `IconProps` 節）。",
        },
        ArgRow {
            name: "view_box",
            kind: "&str",
            default: "\"0 0 24 24\"",
            description: "viewBox 属性値（icon.rs `IconProps` 節）。",
        },
    ],
    examples: &[ExampleEntry {
        title: "Labelled",
        description: "label=\"Search\" を指定し role=\"img\" として扱う例です。",
        render: ex_icon,
    }],
    keyboard: &[],
    aria: &[AriaRow {
        attribute: "role=\"img\" + aria-label（label が Some の場合）/ aria-hidden=\"true\"（None の場合）",
        description: "label の有無で意味のあるアイコンか装飾用途かを切り替える（icon.rs `icon` 節）。",
    }],
    demo: None,
};

fn ex_image() -> Node {
    image::image(
        &image::ImageProps {
            fit: image::ImageFit::Contain,
            aspect_ratio: image::AspectRatio::Square,
            shape: image::ImageShape::Rounded,
            ..image::ImageProps::new(crate::showcase::IMAGE_DEMO_SRC, "製品写真")
        },
        vec![],
    )
}

pub(crate) const IMAGE: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "ImageFit（Cover/Contain/Fill/ScaleDown/NoFit、crates/pre-styled-ui/src/image.rs:76-107）で object-fit を切り替える",
        "AspectRatio（Auto/Square/Landscape/Portrait/Video、image.rs:113-141）で aspect-ratio を切り替える（イシュー #1562 で Landscape(4:3)/Portrait(3:4) を追加）",
        "ImageShape（Square/Rounded/Circle、image.rs:150-172）で角丸を切り替える（イシュー #1562 で新設。radius トークン `--fandhe-radius-none`/`-md`/`-full` 経由）",
        "base に height: auto を持つ（image.rs、イシュー #1562）。max-width: 100% による縮小時に縦横比を保つ",
        "src は既定エスケープ + is_safe_url 検証を経由し危険なスキーム（javascript: 等・data: 等）は出力自体を落とす（image.rs テスト dangerous_src_scheme_is_not_output_but_sibling_attrs_survive）",
    ],
    arguments: &[
        ArgRow {
            name: "src",
            kind: "&str",
            default: "(必須)",
            description: "画像 URL（image.rs:258、is_safe_url 検証を経由）。",
        },
        ArgRow {
            name: "alt",
            kind: "&str",
            default: "(必須)",
            description: "代替テキスト（image.rs:261、空文字列も明示的な選択として許容）。",
        },
        ArgRow {
            name: "fit",
            kind: "ImageFit",
            default: "Cover",
            description: "object-fit（image.rs:76-107, 263）。",
        },
        ArgRow {
            name: "aspect_ratio",
            kind: "AspectRatio",
            default: "Auto",
            description: "aspect-ratio（image.rs:113-141, 265）。",
        },
        ArgRow {
            name: "shape",
            kind: "ImageShape",
            default: "Square",
            description: "角丸（image.rs:150-172, 267。イシュー #1562 で新設）。",
        },
    ],
    examples: &[ExampleEntry {
        title: "Square contain, rounded",
        description: "Contain fit + Square aspect ratio + Rounded shape の例です。",
        render: ex_image,
    }],
    keyboard: &[],
    aria: &[AriaRow {
        attribute: "alt（必須引数）",
        description: "代替テキストを img 要素の alt として必ず出力する（image.rs:261）。role/aria-* 自体は固有の出力を持たない。",
    }],
    demo: None,
};

fn ex_timeline() -> Node {
    timeline::root(
        timeline::TimelineVariant::Outline,
        Size::Md,
        ColorPalette::Accent,
        vec![],
        vec![
            timeline::item(
                vec![],
                vec![
                    timeline::connector(
                        vec![],
                        vec![
                            timeline::indicator(vec![], vec![]),
                            timeline::separator(vec![], vec![]),
                        ],
                    ),
                    timeline::content(
                        vec![],
                        vec![
                            timeline::title(vec![], vec![text("Started")]),
                            timeline::description(vec![], vec![text("2026-01-01")]),
                        ],
                    ),
                ],
            ),
            timeline::item(
                vec![],
                vec![
                    timeline::connector(vec![], vec![timeline::indicator(vec![], vec![])]),
                    timeline::content(vec![], vec![timeline::title(vec![], vec![text("Now")])]),
                ],
            ),
        ],
    )
}

pub(crate) const TIMELINE: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "TimelineVariant（Solid/Subtle/Outline/Plain、crates/pre-styled-ui/src/timeline.rs:232-247）で indicator の塗り方を切り替える",
        "item/connector/separator/indicator/content/title/description の 7 パーツで年表を構造化する（timeline.rs:711-756）",
        "最終 item は separator を組み込まないことで非表示にする契約（showLastSeparator 相当は実装しない、timeline.rs:48-55）",
        "content は縦積み flex で title（sm / medium、size 連動の font-size）・description（xs / muted）の型階層を持つ（イシュー #1576）",
        "呼び出し側が indicator/separator へ data-state=\"complete\"/\"current\" を付与すると完了区間・現在位置のスタイルが適用される（recipe 側は子孫セレクタを持たないため、値は呼び出し側の構成責務。イシュー #1575、timeline.rs:107-118）",
    ],
    arguments: &[
        ArgRow {
            name: "variant",
            kind: "TimelineVariant",
            default: "Solid",
            description: "indicator の塗り方（timeline.rs:232-247）。",
        },
        ArgRow {
            name: "palette",
            kind: "ColorPalette",
            default: "Accent",
            description: "colorPalette 軸（timeline.rs:659）。",
        },
    ],
    examples: &[ExampleEntry {
        title: "Two events",
        description: "2 件のイベントを Outline variant で表示する例です（最終 item は separator を省略）。",
        render: ex_timeline,
    }],
    keyboard: &[],
    aria: &[AriaRow {
        attribute: "(該当なし)",
        description: "ol/li のネイティブ意味論のみで固有の role/aria-* は出力しない（timeline.rs 全文で role/aria-* を grep しても 0 件）。",
    }],
    demo: None,
};

// ---------------------------------------------------------------------
// Interactive（navigation 系、本文明示 5 件）
// ---------------------------------------------------------------------

fn ex_breadcrumb() -> Node {
    breadcrumb::root(
        Size::Md,
        breadcrumb::BreadcrumbVariant::Underline,
        None,
        vec![],
        vec![breadcrumb::list(
            vec![],
            vec![
                breadcrumb::item(
                    vec![],
                    vec![breadcrumb::link("../table/", vec![], vec![text("Docs")])],
                ),
                breadcrumb::separator(vec![], vec![text("/")]),
                breadcrumb::item(
                    vec![],
                    vec![breadcrumb::current_link(vec![], vec![text("Components")])],
                ),
            ],
        )],
    )
}

pub(crate) const BREADCRUMB: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "BreadcrumbVariant（Plain/Underline、crates/pre-styled-ui/src/breadcrumb.rs:266-272）でリンクの下線表示を切り替える",
        "root は既定で aria-label=\"breadcrumb\" を付与する（breadcrumb.rs:152-154）",
        "current_link は aria-current=\"page\"、separator は role=\"presentation\" を固定付与する（breadcrumb.rs:184-188）",
    ],
    arguments: &[
        ArgRow {
            name: "variant",
            kind: "BreadcrumbVariant",
            default: "Plain",
            description: "リンクの下線表示（breadcrumb.rs:266-272）。",
        },
        ArgRow {
            name: "aria_label_value",
            kind: "Option<&str>",
            default: "None",
            description: "None の場合は既定値 \"breadcrumb\" を使う（breadcrumb.rs:122-134, 152-154）。",
        },
    ],
    examples: &[ExampleEntry {
        title: "Two levels",
        description: "Docs → Components の 2 階層パンくずの例です。",
        render: ex_breadcrumb,
    }],
    keyboard: &[],
    aria: &[
        AriaRow {
            attribute: "aria-label=\"breadcrumb\"（root、既定値）",
            description: "root へ既定で付与される（breadcrumb.rs:122-134, 152-154）。",
        },
        AriaRow {
            attribute: "aria-current=\"page\"（current_link） / role=\"presentation\"（separator）",
            description: "現在ページと区切り記号に付与される（breadcrumb.rs:184-188）。",
        },
    ],
    demo: None,
};

/// `crates/pre-styled-ui/src/tab_nav.rs:171`（`root` が `aria-label` を必須
/// 引数として要求）・`:198-209`（`link` が `current` に応じて
/// `aria-current="page"` + `data-current` を付与）。href は
/// `crate::linkcheck::check_links` の突合対象のため、実在ページへ解決
/// する相対パス（`/themes/` 配下の兄弟ページ。イシュー #1017 で
/// `/components/` から移行）を使う。
fn ex_tab_nav() -> Node {
    tab_nav::root(
        Size::Md,
        "Section navigation",
        vec![],
        vec![
            tab_nav::link("../tabs/", true, vec![], vec![text("Tabs")]),
            tab_nav::link("../nav-list/", false, vec![], vec![text("Nav List")]),
            tab_nav::link("../menubar/", false, vec![], vec![text("Menubar")]),
        ],
    )
}

pub(crate) const TAB_NAV: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "role=\"tablist\"/role=\"tab\" を出力しない。素の nav/a の暗黙 ARIA ロール（navigation/link）のみを使うナビゲーションリンク集合（crates/pre-styled-ui/src/tab_nav.rs:1-20）",
        "現在ページは aria-current=\"page\" + data-current で示す（tab_nav.rs:198-209）",
        "見た目は自前の宣言列から生成する（イシュー #1541 で crate::tabs との共有を解消）。size 軸（xs〜xl、既定 md）を持ち、color-palette 軸は非提供（tab_nav.rs 参照）",
    ],
    arguments: &[
        ArgRow {
            name: "size",
            kind: "Size",
            default: "Md",
            description: "サイズ（イシュー #1541 で追加、tab_nav.rs 参照）。",
        },
        ArgRow {
            name: "label",
            kind: "&str",
            default: "",
            description: "root に付与する aria-label（必須引数、tab_nav.rs:171）。",
        },
        ArgRow {
            name: "href",
            kind: "&str",
            default: "",
            description: "link の href（tab_nav.rs:198）。",
        },
        ArgRow {
            name: "current",
            kind: "bool",
            default: "false",
            description: "true のとき aria-current=\"page\" + data-current を付与する（tab_nav.rs:198-209）。",
        },
    ],
    examples: &[ExampleEntry {
        title: "Three links",
        description: "Tabs / Nav List / Menubar の 3 リンクのうち Tabs を現在ページとした例です。",
        render: ex_tab_nav,
    }],
    keyboard: &[KeyRow {
        key: "Tab / Shift+Tab",
        description: "通常のリンクとしてフォーカス移動する。矢印キーによる roving tabindex は持たない（crate::tabs との決定的な差）。",
    }],
    aria: &[
        AriaRow {
            attribute: "aria-label（root、必須引数）",
            description: "landmark のアクセシブルネームを常に持つ（tab_nav.rs:171）。",
        },
        AriaRow {
            attribute: "aria-current=\"page\"（link、current=true のとき）",
            description: "現在ページを示す（tab_nav.rs:198-209）。",
        },
        AriaRow {
            attribute: "role",
            description: "一切出力しない（tab_nav.rs:18）。",
        },
    ],
    demo: None,
};

fn ex_carousel() -> Node {
    carousel::root(
        Size::Md,
        Orientation::Horizontal,
        "Products",
        vec![],
        vec![],
    )
}

pub(crate) const CAROUSEL: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "orientation（Horizontal/Vertical）に応じて item-group の transform 軸を translateX/translateY に切り替える（crates/pre-styled-ui/src/carousel.rs:390-398）",
        "label 引数を root の aria-label へそのまま出力する（carousel.rs:277-318, 321-322）",
        "選択・チェック状態を示す部品ではないため colorPalette 軸を提供しない（carousel.rs テスト carousel_stylesheet_never_consumes_color_palette_axis）",
        "--fandhe-carousel-item-basis（既定 100%）を root へ設定すると item の flex-basis と item-group の横方向 transform 係数の両方に反映され、複数スライドの同時表示に対応する（イシュー #2028、shadcn/ui の Sizes 例に相当。carousel.rs の recipe() item/item-group base）",
    ],
    arguments: &[
        ArgRow {
            name: "orientation",
            kind: "Orientation",
            default: "Horizontal",
            description: "スクロール方向（carousel.rs:277-318）。",
        },
        ArgRow {
            name: "label",
            kind: "&str",
            default: "(必須)",
            description: "root の aria-label に渡すラベル（carousel.rs:277-318, 321-322）。",
        },
    ],
    examples: &[ExampleEntry {
        title: "Root only",
        description: "aria-label=\"Products\" を持つ Carousel root の最小構成例です。",
        render: ex_carousel,
    }],
    keyboard: &[],
    aria: &[AriaRow {
        attribute: "aria-label（root、呼び出し側必須引数）",
        description: "root へそのまま出力される（carousel.rs:277-318, 321-322）。",
    }],
    demo: None,
};

fn ex_pagination() -> Node {
    pagination::root(
        Size::Md,
        ColorPalette::Accent,
        "pagination",
        vec![],
        vec![pagination::item(
            pagination::ItemMode::Button,
            1,
            true,
            false,
            vec![],
            vec![text("1")],
        )],
    )
}

/// [`PAGINATION`] の Examples 節「Prev/Next with label」レンダラ
/// （イシュー #2036、shadcn/ui 突合。shadcn の Prev/Next は「‹ Previous」
/// 「Next ›」というアイコン+テキスト表示だが、headless 層の
/// `prev_trigger`/`next_trigger` は children を固定しない設計のため、
/// 呼び出し側が単一テキストノード（`calendar::prev_trigger`/
/// `next_trigger` が確立した慣例、`crates/docs-site/src/showcase.rs` の
/// `‹`/`›` 単一テキストノードと同型）を渡すだけで再現できる。複数
/// children（アイコン部品 + テキスト）による横並びは採用せず、
/// `prev-trigger`/`next-trigger` の `gap` 追加という recipe 変更を発生
/// させない（`pagination.rs` モジュール rustdoc「shadcn/ui 突合」節参照）。
fn ex_pagination_prev_next_label() -> Node {
    pagination::root(
        Size::Md,
        ColorPalette::Accent,
        "pagination",
        vec![],
        vec![
            pagination::prev_trigger(
                pagination::ItemMode::Button,
                false,
                vec![],
                vec![text("\u{2039} Previous")],
            ),
            pagination::item(
                pagination::ItemMode::Button,
                1,
                false,
                false,
                vec![],
                vec![text("1")],
            ),
            pagination::item(
                pagination::ItemMode::Button,
                2,
                true,
                false,
                vec![],
                vec![text("2")],
            ),
            pagination::item(
                pagination::ItemMode::Button,
                3,
                false,
                false,
                vec![],
                vec![text("3")],
            ),
            pagination::next_trigger(
                pagination::ItemMode::Button,
                false,
                vec![],
                vec![text("Next \u{203a}")],
            ),
        ],
    )
}

/// [`PAGINATION`] の Examples 節「Rows per page + Select」レンダラ
/// （イシュー #2036、shadcn/ui 突合。shadcn の Pagination はデータテーブル
/// 用フッターとして「Rows per page」ラベル付き Select + Prev/Next のみ
/// （ページ番号なし）という合成パターンも提供する。`native_select` が
/// 既に存在するため、既存部品の組み合わせで再現する
/// （`pagination.rs` モジュール rustdoc「shadcn/ui 突合」節参照。
/// `field::label`/`native_select::native_select` の組み合わせは
/// `crates/docs-site/src/component_specs/forms.rs::ex_native_select_with_label_and_helper`
/// と同型）。
fn ex_pagination_rows_per_page() -> Node {
    let f = native_select::FieldProps {
        id: "example-pagination-rows-per-page",
        ids: native_select::FieldIds::default(),
        disabled: false,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: false,
    };
    div(
        vec![],
        vec![
            field::label(&f, vec![], vec![text("Rows per page")]),
            native_select::native_select(
                &native_select::NativeSelectProps::default(),
                &f,
                vec![],
                vec![
                    el("option", vec![("value", "10")], vec![text("10")]),
                    el(
                        "option",
                        vec![("value", "25"), ("selected", "")],
                        vec![text("25")],
                    ),
                    el("option", vec![("value", "50")], vec![text("50")]),
                ],
            ),
            pagination::root(
                Size::Md,
                ColorPalette::Accent,
                "rows per page pagination",
                vec![],
                vec![
                    pagination::prev_trigger(
                        pagination::ItemMode::Button,
                        false,
                        vec![],
                        vec![text("\u{2039} Previous")],
                    ),
                    pagination::next_trigger(
                        pagination::ItemMode::Button,
                        false,
                        vec![],
                        vec![text("Next \u{203a}")],
                    ),
                ],
            ),
        ],
    )
}

pub(crate) const PAGINATION: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "item は data-selected マーカー + aria-current=\"page\" で現在ページを表す（crates/pre-styled-ui/src/pagination.rs:35-38）",
        "item 自体には class を付与しない（root のみへクラスが付く複合部品の variant 統一方針、pagination.rs テスト reexported_item_is_not_given_variant_classes）",
        "root は <nav> 要素として出力される（pagination.rs テスト root_outputs_scope_and_part）",
    ],
    arguments: &[
        ArgRow {
            name: "palette",
            kind: "ColorPalette",
            default: "Accent",
            description: "colorPalette 軸（pagination.rs:447-460）。",
        },
        ArgRow {
            name: "aria_label",
            kind: "&str",
            default: "(必須)",
            description: "root（nav）の aria-label（pagination.rs:447-460, 528-534）。",
        },
    ],
    examples: &[
        ExampleEntry {
            title: "Current page",
            description: "選択中ページ 1 件のみを表示する最小構成例です。",
            render: ex_pagination,
        },
        ExampleEntry {
            title: "Prev/Next with label",
            description: "shadcn/ui の「‹ Previous」「Next ›」表示を、prev_trigger/next_trigger へ単一テキストノードを渡すだけで再現する例です（イシュー #2036）。",
            render: ex_pagination_prev_next_label,
        },
        ExampleEntry {
            title: "Rows per page + Select",
            description: "shadcn/ui のデータテーブル用フッター（Rows per page ラベル付き Select + Prev/Next のみ、ページ番号なし）を既存部品の組み合わせで再現する例です（イシュー #2036）。",
            render: ex_pagination_rows_per_page,
        },
    ],
    keyboard: &[],
    aria: &[AriaRow {
        attribute: "aria-current=\"page\"（選択中 item） / data-selected",
        description: "現在ページを示すマーカー（pagination.rs:35-38）。root（nav）は呼び出し側指定の aria-label を持つ。",
    }],
    demo: None,
};

fn ex_splitter() -> Node {
    use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::splitter::{
        PanelSpec, Splitter,
    };
    let state = Splitter::new(
        &[
            PanelSpec::new(50.0, 0.0, 100.0),
            PanelSpec::new(50.0, 0.0, 100.0),
        ],
        Orientation::Horizontal,
    );
    splitter::root(
        Size::Md,
        ColorPalette::Accent,
        &state,
        false,
        vec![],
        vec![
            splitter::panel(&state, 0, "panel-a", vec![], vec![text("A")]),
            splitter::resize_trigger(&state, 0, "panel-a", "panel-b", false, vec![], vec![]),
            splitter::panel(&state, 1, "panel-b", vec![], vec![text("B")]),
        ],
    )
}

// イシュー #2038: shadcn/ui `resizable` の `withHandle` prop 相当。
// `resize_trigger_indicator` を `resize_trigger` の children に渡すと
// ハンドル中央にグリップ表現が付く（呼ぶ／呼ばないの 2 択で shadcn の
// `withHandle` あり／なしに対応する既存 API、新規 API 追加なし）。
fn ex_splitter_with_handle() -> Node {
    use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::splitter::{
        PanelSpec, Splitter,
    };
    let state = Splitter::new(
        &[
            PanelSpec::new(50.0, 0.0, 100.0),
            PanelSpec::new(50.0, 0.0, 100.0),
        ],
        Orientation::Horizontal,
    );
    splitter::root(
        Size::Md,
        ColorPalette::Accent,
        &state,
        false,
        vec![],
        vec![
            splitter::panel(&state, 0, "panel-wh-a", vec![], vec![text("A")]),
            splitter::resize_trigger(
                &state,
                0,
                "panel-wh-a",
                "panel-wh-b",
                false,
                vec![],
                vec![splitter::resize_trigger_indicator(vec![], vec![])],
            ),
            splitter::panel(&state, 1, "panel-wh-b", vec![], vec![text("B")]),
        ],
    )
}

// イシュー #2038: shadcn/ui デフォルトデモ（One | (Two / Three)）と同型の
// 入れ子構成。panel の children に別の splitter::root（内側 vertical）を
// そのまま渡すだけで再現できる合成パターン（新規 API 不要）。
fn ex_splitter_nested() -> Node {
    use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::splitter::{
        PanelSpec, Splitter,
    };
    let outer = Splitter::new(
        &[
            PanelSpec::new(40.0, 20.0, 80.0),
            PanelSpec::new(60.0, 20.0, 80.0),
        ],
        Orientation::Horizontal,
    );
    let inner = Splitter::new(
        &[
            PanelSpec::new(50.0, 0.0, 100.0),
            PanelSpec::new(50.0, 0.0, 100.0),
        ],
        Orientation::Vertical,
    );
    let inner_demo = splitter::root(
        Size::Md,
        ColorPalette::Accent,
        &inner,
        false,
        vec![("style", "border: none; border-radius: 0; height: 100%;")],
        vec![
            splitter::panel(&inner, 0, "panel-nested-inner-a", vec![], vec![text("Two")]),
            splitter::resize_trigger(
                &inner,
                0,
                "panel-nested-inner-a",
                "panel-nested-inner-b",
                false,
                vec![],
                vec![],
            ),
            splitter::panel(
                &inner,
                1,
                "panel-nested-inner-b",
                vec![],
                vec![text("Three")],
            ),
        ],
    );
    splitter::root(
        Size::Md,
        ColorPalette::Accent,
        &outer,
        false,
        vec![("style", "height: 12rem;")],
        vec![
            splitter::panel(&outer, 0, "panel-nested-outer-a", vec![], vec![text("One")]),
            splitter::resize_trigger(
                &outer,
                0,
                "panel-nested-outer-a",
                "panel-nested-outer-b",
                false,
                vec![],
                vec![],
            ),
            splitter::panel(&outer, 1, "panel-nested-outer-b", vec![], vec![inner_demo]),
        ],
    )
}

pub(crate) const SPLITTER: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "panel は --fandhe-splitter-size custom property を通じてのみ動的な flex-basis を伝える唯一のパーツ（crates/pre-styled-ui/src/splitter.rs:688-697）",
        "resize_trigger は role=\"separator\" + aria-controls を固定付与する（splitter.rs:930-935）",
        "panel_index が範囲外の場合は style 属性自体を省略する fail-closed 動作（splitter.rs:694-697, 908-913）",
        "resize_trigger_indicator は resize_trigger の children として渡したときのみ描画される（shadcn/ui withHandle prop 相当の合成パターン、イシュー #2038）",
        "resize_trigger は ::after で視覚上の太さを変えずに当たり判定のみを外側へ拡張する（data-orientation で拡張方向を切り替え、--fandhe-splitter-hit-extension で拡張幅を上書き可、イシュー #2202）",
    ],
    arguments: &[ArgRow {
        name: "disabled",
        kind: "bool",
        default: "false",
        description: "root/resize_trigger の無効化状態（splitter.rs:672-686）。",
    }],
    examples: &[
        ExampleEntry {
            title: "Two panels",
            description: "50/50 の 2 パネルと resize_trigger 1 個の例です。",
            render: ex_splitter,
        },
        ExampleEntry {
            title: "With handle",
            description: "resize_trigger_indicator を渡すとハンドル中央にグリップ表現が付きます。",
            render: ex_splitter_with_handle,
        },
        ExampleEntry {
            title: "Nested",
            description: "panel の children に別の splitter::root を渡すと入れ子構成になります。",
            render: ex_splitter_nested,
        },
    ],
    keyboard: &[],
    aria: &[AriaRow {
        attribute: "role=\"separator\" + aria-controls（resize_trigger）",
        description: "リサイズハンドルが操作対象パネルを aria-controls で指し示す（splitter.rs:930-935）。",
    }],
    demo: None,
};

fn ex_steps() -> Node {
    use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::steps::Steps;
    let state = Steps::new(3, 1, Orientation::Horizontal);
    steps::root(
        Size::Md,
        ColorPalette::Accent,
        &state,
        vec![],
        vec![steps::list(
            &state,
            vec![],
            vec![
                steps::item(
                    &state,
                    0,
                    vec![],
                    vec![
                        steps::trigger(
                            &state,
                            0,
                            vec![],
                            vec![steps::indicator(&state, 0, vec![], vec![])],
                        ),
                        steps::separator(&state, 0, vec![], vec![]),
                    ],
                ),
                steps::item(
                    &state,
                    1,
                    vec![],
                    vec![steps::trigger(
                        &state,
                        1,
                        vec![],
                        vec![steps::indicator(&state, 1, vec![], vec![])],
                    )],
                ),
            ],
        )],
    )
}

pub(crate) const STEPS: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "trigger は現在ステップに aria-current=\"step\" を固定付与する（crates/pre-styled-ui/src/steps.rs テスト list_item_trigger_indicator_separator_delegate_to_headless、行 1256）",
        "separator は role=\"separator\" を持つ（steps.rs テスト同上、行 1258）",
        "indicator の data-state（current/complete）で見た目を切り替える（steps.rs:1134-1142）",
    ],
    arguments: &[],
    examples: &[ExampleEntry {
        title: "3 steps, step 2 current",
        description: "3 ステップ中 2 番目が current の状態を固定表示する例です。",
        render: ex_steps,
    }],
    keyboard: &[],
    aria: &[AriaRow {
        attribute: "aria-current=\"step\"（trigger） / role=\"separator\"（separator）",
        description: "現在ステップと区切りの意味論（steps.rs テスト list_item_trigger_indicator_separator_delegate_to_headless、行 1256, 1258）。",
    }],
    demo: None,
};

// ---------------------------------------------------------------------
// Utilities（本文明示 3 件）
// ---------------------------------------------------------------------

fn ex_marquee() -> Node {
    marquee::marquee(
        &marquee::MarqueeProps {
            label: Some("Breaking news"),
            ..marquee::MarqueeProps::default()
        },
        vec![],
        vec![marquee::item(vec![], vec![text("Breaking news ticker")])],
    )
}

pub(crate) const MARQUEE: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "content パーツを内部で 2 回複製しシームレスループを実現する（crates/pre-styled-ui/src/marquee.rs:1296-1298）",
        "root:hover/:focus-within で常時一時停止する CSS を持つ（marquee.rs テスト css_output_declares_hover_and_focus_within_pause）",
        "prefers-reduced-motion: reduce で (1) アニメーションを停止し、(2) 複製 content を除去し、(3) 可視コピーを折り返して全文表示し、(4) 両端フェードを解除する（イシュー #1583、marquee.rs テスト css_output_declares_reduced_motion_media_query）",
        "両端フェードは root の mask-image（--fandhe-marquee-fade、既定 0px = 無効）で opt-in 提供する（イシュー #1582、marquee.rs テスト css_output_declares_root_mask_image_fade）",
        "root の内側余白は --fandhe-marquee-padding（既定 0）の custom property hook で opt-in 提供する（イシュー #1583、marquee.rs テスト css_output_declares_root_padding_hook）",
    ],
    arguments: &[
        ArgRow {
            name: "decorative",
            kind: "bool",
            default: "false",
            description: "true なら root へ aria-hidden=\"true\" を付与する（marquee.rs:184-191, 1341-1354）。",
        },
        ArgRow {
            name: "label",
            kind: "Option<&str>",
            default: "None",
            description: "decorative が false のときのみ有効な aria-label（marquee.rs:191, 1355-1357）。",
        },
    ],
    examples: &[ExampleEntry {
        title: "Labelled",
        description: "aria-label 付きの非装飾 Marquee の例です。",
        render: ex_marquee,
    }],
    keyboard: &[],
    aria: &[AriaRow {
        attribute: "aria-hidden=\"true\"（decorative=true の root と複製 content） / aria-label（非 decorative時の root）",
        description: "decorative/label の組み合わせで root の意味論を切り替える（marquee.rs:1341-1357）。複製した 2 個目の content は常に aria-hidden=\"true\" + inert を持つ（marquee.rs:1360-1385）。",
    }],
    demo: None,
};

fn ex_scroll_area() -> Node {
    let items: Vec<Node> = (1..=5)
        .map(|i| el("p", vec![], vec![text(format!("Row {i}"))]))
        .collect();
    scroll_area::root(
        vec![(
            "style",
            "height: 6rem; width: 12rem; border: 1px solid var(--fandhe-color-border);",
        )],
        vec![scroll_area::viewport(
            vec![],
            vec![scroll_area::content(vec![], items)],
        )],
    )
}

/// `/themes/scroll-area/` の Examples 節其の 2（イシュー #2054、shadcn/ui
/// 突合）: 線 – テキスト区切りのタグリスト。shadcn の Tags リスト例
/// （`h4` + 行ごとの `Separator`）を、既存の `separator::separator` のみで
/// 再現する。
fn ex_scroll_area_tags() -> Node {
    let tags: Vec<Node> = (1..=5)
        .flat_map(|i| {
            vec![
                el("p", vec![], vec![text(format!("Tag {i}"))]),
                separator::separator(&separator::SeparatorProps::default(), vec![]),
            ]
        })
        .collect();
    scroll_area::root(
        vec![(
            "style",
            "height: 8rem; width: 12rem; border: 1px solid var(--fandhe-color-border);",
        )],
        vec![scroll_area::viewport(
            vec![],
            vec![scroll_area::content(
                vec![],
                std::iter::once(el("h4", vec![], vec![text("Tags")]))
                    .chain(tags)
                    .collect(),
            )],
        )],
    )
}

/// `/themes/scroll-area/` の Examples 節其の 3（イシュー #2054）: 横スクロール
/// （shadcn `ScrollBar orientation="horizontal"` 相当）。`viewport` へ
/// `data-orientation="horizontal"` を付与すると `content` が `flex` 化される。
fn ex_scroll_area_horizontal() -> Node {
    let figures: Vec<Node> = (1..=4)
        .map(|i| {
            el(
                "figure",
                vec![("style", "margin: 0; flex: none;")],
                vec![
                    image::image(
                        &image::ImageProps {
                            fit: image::ImageFit::Cover,
                            ..image::ImageProps::new(
                                crate::showcase::IMAGE_DEMO_SRC,
                                "横スクロールデモ画像",
                            )
                        },
                        vec![("style", "width: 6rem; height: 4.5rem;")],
                    ),
                    el("figcaption", vec![], vec![text(format!("Photo {i}"))]),
                ],
            )
        })
        .collect();
    scroll_area::root(
        vec![(
            "style",
            "width: 12rem; border: 1px solid var(--fandhe-color-border);",
        )],
        vec![scroll_area::viewport(
            vec![("data-orientation", "horizontal")],
            vec![scroll_area::content(
                vec![(
                    "style",
                    "gap: var(--fandhe-space-4); padding: var(--fandhe-space-4);",
                )],
                figures,
            )],
        )],
    )
}

/// `/themes/scroll-area/` の Examples 節其の 4（イシュー #2054）: RTL。
/// `root` へ `dir="rtl"` をそのまま透過させる（headless 層は独自の RTL
/// 処理を持たず、ブラウザネイティブの `dir` 属性へ委ねる）。
fn ex_scroll_area_rtl() -> Node {
    let items: Vec<Node> = (1..=5)
        .map(|i| el("p", vec![], vec![text(format!("行 {i}"))]))
        .collect();
    scroll_area::root(
        vec![
            ("dir", "rtl"),
            (
                "style",
                "height: 6rem; width: 12rem; border: 1px solid var(--fandhe-color-border);",
            ),
        ],
        vec![scroll_area::viewport(
            vec![("data-fade", "")],
            vec![scroll_area::content(vec![], items)],
        )],
    )
}

pub(crate) const SCROLL_AREA: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "headless 層は anatomy（data-scope/data-part）と tabindex=\"0\" のみを提供し、CSS overflow が実際のスクロール可能性を担う",
        "::-webkit-scrollbar 系規則でカスタムスクロールバーの見た目を表現する",
        "thumb 色は custom property --fandhe-scroll-area-thumb-bg（既定 fg-subtle トークン。WCAG 非テキストコントラスト基準 3:1 を満たす、イシュー #1584 PR #1858 是正）で一元化されており、root へ再定義するだけで scrollbar-color と ::-webkit-scrollbar-thumb 双方の色を揃って変更できる",
        "viewport の hover 時・キーボードフォーカス時の双方で thumb 色を --fandhe-scroll-area-thumb-hover-bg（既定 fg）へ強調する（常時表示 + hover/focus 強調。タッチ端末でも thumb が不可視にならないよう hover-reveal は既定にしない、イシュー #1584）",
        "フォーカスリングは focus_ring_declarations（Token/Inset）による canonical 表現で、root の overflow: hidden 内に収まるよう内側描画にしている（イシュー #1584）",
        "variant（chakra の hover/always・Radix Themes の size/type 相当）は提供しない。custom property の上書きで同等の見た目を利用側から再現できる",
        "横スクロール: viewport へ data-orientation=\"horizontal\" を付与すると content が display: flex; width: max-content; になる（shadcn/ui ScrollBar orientation=\"horizontal\" 相当、イシュー #2054）",
        "端フェード: viewport へ data-fade を付与すると mask-image による端フェードが opt-in する。animation-timeline: scroll() 対応ブラウザでは @supports 配下でスクロール量に応じてフェード端を動的に切り替え、非対応ブラウザは両端固定フェードへ graceful degradation する（shadcn/ui utils/scroll-fade 相当、イシュー #2054）",
        "RTL 対応: data-fade + data-orientation=\"horizontal\" 併用時は :dir(rtl) でグラデーション方向を反転する（イシュー #2054）",
        "JS によるスクロール位置追従は対象外（showcase.rs の scroll_area_section 記述と同方針）",
    ],
    arguments: &[],
    examples: &[
        ExampleEntry {
            title: "Scrollable list",
            description: "固定高さのビューポート内に 5 行を表示するスクロール領域の例です。",
            render: ex_scroll_area,
        },
        ExampleEntry {
            title: "Tags",
            description: "線区切りのタグリストの例です（イシュー #2054、shadcn/ui Tags 例相当）。",
            render: ex_scroll_area_tags,
        },
        ExampleEntry {
            title: "Horizontal scroll",
            description: "data-orientation=\"horizontal\" による横スクロールの例です（イシュー #2054）。",
            render: ex_scroll_area_horizontal,
        },
        ExampleEntry {
            title: "RTL + fade",
            description: "dir=\"rtl\" と data-fade を組み合わせた例です（イシュー #2054）。",
            render: ex_scroll_area_rtl,
        },
    ],
    keyboard: &[],
    aria: &[AriaRow {
        attribute: "tabindex=\"0\"（viewport）",
        description: "キーボードフォーカス可能にするのみで、固有の role/aria-* は出力しない。",
    }],
    demo: None,
};

fn ex_separator() -> Node {
    separator::separator(
        &separator::SeparatorProps {
            orientation: Orientation::Vertical,
            variant: separator::SeparatorVariant::Dashed,
        },
        vec![],
    )
}

/// `/themes/separator/` の Examples 節其の 2（イシュー #2053）: 線 – テキスト
/// – 線のラベル付き合成（`group`/`label`、chakra-ui の HStack + Text 合成
/// 相当、shadcn/ui 突合で補完）。
fn ex_separator_labeled() -> Node {
    separator::group(
        vec![("style", "width: 16rem;")],
        vec![
            separator::separator(&separator::SeparatorProps::default(), vec![]),
            separator::label(vec![], vec![text("OR")]),
            separator::separator(&separator::SeparatorProps::default(), vec![]),
        ],
    )
}

pub(crate) const SEPARATOR: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "orientation が role=\"separator\"（固定） + aria-orientation + data-orientation + variant クラスの 3 箇所へ連動する（crates/pre-styled-ui/src/separator.rs:10, 17）",
        "SeparatorVariant（Solid/Dashed/Dotted、separator.rs:113-121。Dotted はイシュー #1585 で追加）で罫線種別を切り替える",
        "罫線の太さは --fandhe-separator-thickness（既定 1px の custom property、イシュー #1585）の上書きで変更する。size 軸は Phase 0 規約（docs/design/pre-styled-ui-focus-ring-and-size-conventions.md §4 (d)）により非提供（separator.rs:24-31）",
        "role/aria-orientation/data-orientation は呼び出し側の偽装を除去し常にフレームワーク値へ一本化する（separator.rs:270-277、skeleton の aria-hidden 除去と同型）",
        "group/label（イシュー #2053、shadcn/ui 突合で補完）: pre-styled-only のラベル付き区切り線パート。線 – テキスト – 線を display: grid（1fr auto 1fr）で合成する（chakra-ui の HStack + Text 合成相当、horizontal 専用契約）",
    ],
    arguments: &[
        ArgRow {
            name: "orientation",
            kind: "Orientation",
            default: "Horizontal",
            description: "向き（separator.rs:156-161）。",
        },
        ArgRow {
            name: "variant",
            kind: "SeparatorVariant",
            default: "Solid",
            description: "罫線種別（solid/dashed/dotted、separator.rs:113-121, 159-161）。",
        },
    ],
    examples: &[
        ExampleEntry {
            title: "Vertical dashed",
            description: "縦向き・破線の Separator の例です。",
            render: ex_separator,
        },
        ExampleEntry {
            title: "Labeled",
            description: "group/label によるラベル付き区切り線の例です（イシュー #2053）。",
            render: ex_separator_labeled,
        },
    ],
    keyboard: &[],
    aria: &[AriaRow {
        attribute: "role=\"separator\" + aria-orientation",
        description: "orientation と連動し常に固定出力される（separator.rs:10, 17, 58-59）。",
    }],
    demo: None,
};

/// `/themes/item/` の Examples 節其の 1: variant/media variant の組み合わせ
/// のうち代表 1 件（`crates/pre-styled-ui/src/item.rs` の `root`/`media`/
/// `content`/`title`/`description` パーツを組み合わせる標準形）。
fn ex_item_default() -> Node {
    item::root(
        ItemRootProps::default(),
        vec![],
        vec![
            item::media(ItemMediaVariant::Icon, vec![], vec![text("IC")]),
            item::content(
                vec![],
                vec![
                    item::title(vec![], vec![text("Default item")]),
                    item::description(
                        vec![],
                        vec![text("A generic list row with media and text.")],
                    ),
                ],
            ),
        ],
    )
}

/// `/themes/item/` の Examples 節其の 2: `href` を渡した root（`a` として
/// 描画され hover 背景・`:focus-visible` リングが付く）+ `actions` パーツの
/// 組み合わせ。`href=""`（空文字列）は linkcheck 中立性を保つための既存
/// showcase デモと同型のパターン（`crate::showcase::item_section` 参照）。
fn ex_item_with_link_and_actions() -> Node {
    item::root(
        ItemRootProps {
            href: Some(""),
            ..Default::default()
        },
        vec![],
        vec![
            item::media(ItemMediaVariant::Icon, vec![], vec![text("GH")]),
            item::content(
                vec![],
                vec![
                    item::title(vec![], vec![text("fandhe-frontend")]),
                    item::description(vec![], vec![text("Linked item with an action.")]),
                ],
            ),
            item::actions(vec![], vec![text("→")]),
        ],
    )
}

/// `/themes/item/` の `ComponentPageSpec`（イシュー #2066）。
/// `crates/pre-styled-ui/src/item.rs` が headless
/// [`fandhe_frontend_headless_ui::item`]（#2065）の 10 パーツ
/// （root/media/content/title/description/actions/header/footer/group/
/// separator）へ shadcn/ui `Item` 相当の意匠を重ねる薄い委譲層であることに
/// 対応する。Demo 節は本 spec ではなく `crate::showcase::COMPONENT_PAGES`
/// の `item_section`（`demo: None` のまま [`ComponentPageSpec::demo`] は
/// 未使用、`crate::component_page::generated_content` の優先順位参照）。
pub(crate) const ITEM: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "ItemVariant（Default/Outline/Muted、item.rs recipe() の data-variant 参照）で見た目を切り替える",
        "ItemSize（Default/Sm）で padding/gap を縮小する（item.rs data-size=\"sm\" 参照）",
        "root/media/content/title/description/actions/header/footer/group/separator の 10 パーツで media + title/description + actions からなる汎用リスト行を構造化する",
        "root へ href を渡すと div ではなく a として描画され、ポインタ・下線解除・hover 背景・:focus-visible リングが付く（item.rs StateCondition::Attr(\"href\") 参照）",
        "media は ItemMediaVariant（Default/Icon/Image）で固定サイズ・背景・角丸を切り替え、Image variant では子 img を object-fit: cover でトリミングする",
        "group は複数 root の縦並びコンテナで、separator を挟んで区切る",
        "バリデーション・送信処理・データ整形はこの部品では実装しない（docs/policy/intentional-non-adoption.md §3.25 規則 1、item.rs モジュール doc「責務境界」節）",
    ],
    arguments: &[
        ArgRow {
            name: "variant",
            kind: "ItemVariant",
            default: "Default",
            description: "見た目（#[default] は Default。Outline は枠線、Muted は背景色を切り替える）。",
        },
        ArgRow {
            name: "size",
            kind: "ItemSize",
            default: "Default",
            description: "サイズ（#[default] は Default。Sm は padding/gap を縮小する 2 値軸、item.rs モジュール doc参照）。",
        },
        ArgRow {
            name: "href",
            kind: "Option<&str>",
            default: "None",
            description: "root を a として描画するリンク先（headless 層がスキーム検証・external の target/rel 付与を担う）。",
        },
        ArgRow {
            name: "external",
            kind: "bool",
            default: "false",
            description: "true の場合 target=\"_blank\" と rel の安全な組を不可分付与する（headless 層に委譲）。",
        },
    ],
    examples: &[
        ExampleEntry {
            title: "Default item",
            description: "media・title・description を組み合わせた標準形の例です。",
            render: ex_item_default,
        },
        ExampleEntry {
            title: "Linked item with actions",
            description: "href 付き root（a として描画）と actions パーツを組み合わせる例です。",
            render: ex_item_with_link_and_actions,
        },
    ],
    keyboard: &[],
    aria: &[AriaRow {
        attribute: "(該当なし、href 指定時は a のネイティブ意味論)",
        description: "root/media/content 等は role/aria-* を独自付与しないレイアウト用パーツであり、href を渡した場合のみ a のネイティブなリンク意味論に委ねる（item.rs モジュール doc参照）。",
    }],
    demo: None,
};

// ---------------------------------------------------------------------
// Message（イシュー #2106、親 #2104。headless anatomy は #2105）
// ---------------------------------------------------------------------

/// `/themes/message/` の Examples 節其の 1: user/assistant の 1 往復
/// （`data-align` の 2 値を両方出現させる）。
fn ex_message_conversation_turn() -> Node {
    // headless message::root は role="listitem" を固定付与し、role="list"
    // （またはそれと同等）の親を required context として要求する
    // （headless message.rs モジュール doc「role="listitem"/role="list"」
    // 参照）。ここでは同一グループとして視覚的にまとめる意図（余白詰め・
    // avatar 省略）はないため message::group ではなく、role="list" のみを
    // 付与した素の div でこの契約を満たす。
    div(
        vec![("role", "list")],
        vec![
            message::root(
                MessageRootProps {
                    role: MessageRole::User,
                    align: MessageAlign::End,
                    ..Default::default()
                },
                vec![],
                vec![
                    message::avatar(
                        vec![],
                        vec![avatar::root(
                            &avatar::AvatarProps::default(),
                            vec![],
                            vec![avatar::fallback(
                                avatar::ImageStatus::Error,
                                vec![],
                                vec![text("YOU")],
                            )],
                        )],
                    ),
                    message::content(vec![], vec![text("What is the release checklist?")]),
                ],
            ),
            message::root(
                MessageRootProps {
                    role: MessageRole::Assistant,
                    align: MessageAlign::Start,
                    ..Default::default()
                },
                vec![],
                vec![
                    message::avatar(
                        vec![],
                        vec![avatar::root(
                            &avatar::AvatarProps::default(),
                            vec![],
                            vec![avatar::fallback(
                                avatar::ImageStatus::Error,
                                vec![],
                                vec![text("AI")],
                            )],
                        )],
                    ),
                    message::content(
                        vec![],
                        vec![text("Bump the version, run the golden tests, open a PR.")],
                    ),
                ],
            ),
        ],
    )
}

/// `/themes/message/` の Examples 節其の 2: `group` による連続発言のまとめ
/// （raw CSS 追記で 2 件目以降の avatar を省略・余白を詰める実演）。
fn ex_message_consecutive_group() -> Node {
    message::group(
        "Conversation",
        vec![],
        vec![
            message::root(
                MessageRootProps {
                    role: MessageRole::Assistant,
                    ..Default::default()
                },
                vec![],
                vec![
                    message::avatar(
                        vec![],
                        vec![avatar::root(
                            &avatar::AvatarProps::default(),
                            vec![],
                            vec![avatar::fallback(
                                avatar::ImageStatus::Error,
                                vec![],
                                vec![text("AI")],
                            )],
                        )],
                    ),
                    message::content(vec![], vec![text("Here is the first part.")]),
                ],
            ),
            message::root(
                MessageRootProps {
                    role: MessageRole::Assistant,
                    ..Default::default()
                },
                vec![],
                vec![
                    message::avatar(
                        vec![],
                        vec![avatar::root(
                            &avatar::AvatarProps::default(),
                            vec![],
                            vec![avatar::fallback(
                                avatar::ImageStatus::Error,
                                vec![],
                                vec![text("AI")],
                            )],
                        )],
                    ),
                    message::content(vec![], vec![text("...and a follow-up detail.")]),
                ],
            ),
        ],
    )
}

/// `/themes/message/` の Examples 節其の 3: `data-loading`/`data-error` の
/// 表示状態（応答待ち・送信失敗）。
fn ex_message_loading_and_error() -> Node {
    // 上と同じ理由（role="listitem" の required context）で role="list"
    // を付与する。
    div(
        vec![("role", "list")],
        vec![
            message::root(
                MessageRootProps {
                    role: MessageRole::Assistant,
                    loading: true,
                    ..Default::default()
                },
                vec![],
                vec![message::content(vec![], vec![text("Thinking...")])],
            ),
            message::root(
                MessageRootProps {
                    role: MessageRole::User,
                    align: MessageAlign::End,
                    error: true,
                    ..Default::default()
                },
                vec![],
                vec![message::content(
                    vec![],
                    vec![text("Message failed to send.")],
                )],
            ),
        ],
    )
}

/// `/themes/message/`（イシュー #2106、親 #2104。headless anatomy は
/// #2105）の原稿データ。`role`/`align`/`loading`/`error` は headless の
/// `data-*` を `AttrEq`/`Attr` で参照するのみで class 軸を持たない
/// （`message.rs` モジュール doc「role / align / loading / error の表現」
/// 節参照）。
pub(crate) const MESSAGE: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "MessageRole（User/Assistant/System、message.rs recipe() の data-role 参照）で発言者ごとに content の背景・文字色を切り替える",
        "MessageAlign（Start/End）で root の水平整列を切り替える（data-role とは独立した軸、headless message.rs モジュール doc「会話系 4 部品の共通語彙」参照）",
        "root/avatar/header/content/footer/group の 6 パーツで会話 1 発言を構造化する",
        "loading/error の bool props で data-loading/data-error 存在属性を切り替え、応答待ち・送信失敗を見た目のみで表す（判定・再送はアプリ責務）",
        "group は連続発言をまとめるコンテナで、2 件目以降の root の余白を詰め avatar を非表示にする raw CSS 追記を持つ（message.rs モジュール doc「raw CSS 追記の理由」節参照）",
        "バリデーション・送信処理・Markdown レンダリングはこの部品では実装しない（docs/policy/intentional-non-adoption.md §3.25 規則 1、message.rs モジュール doc「責務境界」節）",
    ],
    arguments: &[
        ArgRow {
            name: "role",
            kind: "MessageRole",
            default: "User",
            description: "発言者の役割（User/Assistant/System。data-role として出力され、content の背景・文字色を切り替える）。",
        },
        ArgRow {
            name: "align",
            kind: "MessageAlign",
            default: "Start",
            description: "root の水平整列（Start/End。role から独立した軸で、center は持たない）。",
        },
        ArgRow {
            name: "loading",
            kind: "bool",
            default: "false",
            description: "true の場合 data-loading 存在属性を付与し、root を半透明化する（応答待ちの表示のみ）。",
        },
        ArgRow {
            name: "error",
            kind: "bool",
            default: "false",
            description: "true の場合 data-error 存在属性を付与し、content の背景・文字色・枠線を危険色へ切り替える（送信失敗の表示のみ）。",
        },
    ],
    examples: &[
        ExampleEntry {
            title: "Conversation turn",
            description: "user（align=end）と assistant（align=start）の 1 往復の例です。",
            render: ex_message_conversation_turn,
        },
        ExampleEntry {
            title: "Consecutive assistant messages in a group",
            description: "group で連続発言をまとめると、2 件目以降の avatar が省略され余白が詰まります。",
            render: ex_message_consecutive_group,
        },
        ExampleEntry {
            title: "Loading and error states",
            description: "data-loading（応答待ち）と data-error（送信失敗）の表示状態の例です。",
            render: ex_message_loading_and_error,
        },
    ],
    keyboard: &[],
    aria: &[
        AriaRow {
            attribute: "role=\"listitem\" (root)",
            description: "root は role=\"listitem\" を固定付与する（headless message.rs モジュール doc「role=\"listitem\"/role=\"list\"」節参照）。",
        },
        AriaRow {
            attribute: "role=\"list\" / aria-label (group)",
            description: "group は role=\"list\" を固定付与し、渡した label が空でなければ aria-label へ出力する（root の listitem に対する required context）。",
        },
        AriaRow {
            attribute: "(付与しない) aria-live / aria-busy",
            description: "ストリーミング通知・応答待ちの読み上げはアプリ固有の UX 判断のため、本モジュールは data-loading/data-error の見た目のみを担い aria-live/aria-busy は付与しない（headless message.rs モジュール doc「aria-live/aria-busy を付けない理由」節参照）。",
        },
    ],
    demo: None,
};

// ---------------------------------------------------------------------
// Bubble（イシュー #2109、親 #2107。headless anatomy は #2108）
// ---------------------------------------------------------------------

/// `/themes/bubble/` の Examples 節其の 1: solid（start）と outline（end）
/// の 1 往復（`data-align` の 2 値・`data-variant` の 2 値を出現させる）。
fn ex_bubble_conversation() -> Node {
    div(
        vec![],
        vec![
            bubble::root(
                BubbleRootProps {
                    variant: BubbleVariant::Solid,
                    align: MessageAlign::Start,
                    group_position: BubbleGroupPosition::Single,
                },
                vec![],
                vec![bubble::content(
                    vec![],
                    vec![text("How do I center a div?")],
                )],
            ),
            bubble::root(
                BubbleRootProps {
                    variant: BubbleVariant::Outline,
                    align: MessageAlign::End,
                    group_position: BubbleGroupPosition::Single,
                },
                vec![],
                vec![bubble::content(vec![], vec![text("Use flexbox.")])],
            ),
        ],
    )
}

/// `/themes/bubble/` の Examples 節其の 2: 連続発言の角丸連結
/// （`data-group-position` の `first`/`middle`/`last` を出現させる）。
fn ex_bubble_group_position() -> Node {
    div(
        vec![],
        vec![
            bubble::root(
                BubbleRootProps {
                    variant: BubbleVariant::Solid,
                    align: MessageAlign::Start,
                    group_position: BubbleGroupPosition::First,
                },
                vec![],
                vec![bubble::content(vec![], vec![text("Here is the plan:")])],
            ),
            bubble::root(
                BubbleRootProps {
                    variant: BubbleVariant::Solid,
                    align: MessageAlign::Start,
                    group_position: BubbleGroupPosition::Middle,
                },
                vec![],
                vec![bubble::content(vec![], vec![text("1. Bump the version.")])],
            ),
            bubble::root(
                BubbleRootProps {
                    variant: BubbleVariant::Solid,
                    align: MessageAlign::Start,
                    group_position: BubbleGroupPosition::Last,
                },
                vec![],
                vec![bubble::content(vec![], vec![text("2. Open a PR.")])],
            ),
        ],
    )
}

/// `/themes/bubble/` の Examples 節其の 3: `plain` variant・`reactions`/
/// `reaction`（selected あり/なし）・`collapse-trigger`/`collapse-content`
/// （open/closed）を出現させる。
fn ex_bubble_reactions_and_collapse() -> Node {
    div(
        vec![],
        vec![
            bubble::root(
                BubbleRootProps {
                    variant: BubbleVariant::Outline,
                    align: MessageAlign::End,
                    group_position: BubbleGroupPosition::Single,
                },
                vec![],
                vec![
                    bubble::content(vec![], vec![text("That should do it.")]),
                    bubble::reactions(
                        "2 reactions",
                        vec![],
                        vec![
                            bubble::reaction(true, vec![], vec![text("\u{1f44d}")]),
                            bubble::reaction(false, vec![], vec![text("\u{2764}")]),
                        ],
                    ),
                    bubble::collapse_trigger(
                        OpenState::Open,
                        Some("bubble-example-detail"),
                        vec![],
                        vec![text("Hide details")],
                    ),
                    bubble::collapse_content(
                        OpenState::Open,
                        Some("bubble-example-detail"),
                        vec![],
                        vec![text("Sent 09:41 \u{b7} Edited")],
                    ),
                ],
            ),
            bubble::root(
                BubbleRootProps {
                    variant: BubbleVariant::Plain,
                    align: MessageAlign::Start,
                    group_position: BubbleGroupPosition::Single,
                },
                vec![],
                vec![
                    bubble::content(vec![], vec![text("Thanks!")]),
                    bubble::collapse_trigger(
                        OpenState::Closed,
                        None,
                        vec![],
                        vec![text("Show details")],
                    ),
                    bubble::collapse_content(
                        OpenState::Closed,
                        None,
                        vec![],
                        vec![text("Sent 09:42")],
                    ),
                ],
            ),
        ],
    )
}

/// `/themes/attachment/` の Examples 節其の 1: `file` 形態（idle）と
/// `media`/`content`/`actions` パーツを出現させる。
fn ex_attachment_file() -> Node {
    attachment::root(
        AttachmentRootProps {
            variant: AttachmentVariant::File,
            state: AttachmentState::Idle,
            disabled: false,
        },
        vec![],
        vec![
            attachment::media(vec![], vec![text("\u{1f4c4}")]),
            attachment::content(
                vec![],
                vec![
                    attachment::name(vec![], vec![text("report.pdf")]),
                    attachment::meta(vec![], vec![text("PDF \u{b7} 128 KB")]),
                ],
            ),
            attachment::actions(
                vec![],
                vec![attachment::action(
                    "Delete report.pdf",
                    false,
                    vec![],
                    vec![text("\u{2715}")],
                )],
            ),
        ],
    )
}

/// `/themes/attachment/` の Examples 節其の 2: `image` 形態
/// （uploading）と `progress` スロットへ入れ子にした styled Progress を
/// 出現させる。
fn ex_attachment_image_uploading() -> Node {
    use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::progress::Progress;
    let p = Progress::new(0.0, 100.0, Some(64.0), Orientation::Horizontal);
    attachment::root(
        AttachmentRootProps {
            variant: AttachmentVariant::Image,
            state: AttachmentState::Uploading,
            disabled: false,
        },
        vec![],
        vec![
            attachment::media(
                vec![],
                vec![fandhe_frontend_core::img(
                    vec![
                        ("src", crate::showcase::IMAGE_DEMO_SRC),
                        ("alt", "photo.png のプレビュー"),
                    ],
                    vec![],
                )],
            ),
            attachment::content(
                vec![],
                vec![
                    attachment::name(vec![], vec![text("photo.png")]),
                    attachment::meta(vec![], vec![text("PNG \u{b7} 2.4 MB \u{b7} 64%")]),
                ],
            ),
            attachment::progress(
                vec![],
                vec![fandhe_frontend_pre_styled_ui::progress::root(
                    &p,
                    &fandhe_frontend_pre_styled_ui::progress::ProgressProps::default(),
                    Some("64%"),
                    vec![],
                    vec![p.track(
                        vec![],
                        vec![fandhe_frontend_pre_styled_ui::progress::range(&p, vec![])],
                    )],
                )],
            ),
        ],
    )
}

/// `/themes/attachment/` の Examples 節其の 3: `error` 状態と
/// `disabled`（root/action 双方）を出現させる。
fn ex_attachment_error() -> Node {
    attachment::root(
        AttachmentRootProps {
            variant: AttachmentVariant::File,
            state: AttachmentState::Error,
            disabled: true,
        },
        vec![],
        vec![
            attachment::media(vec![], vec![text("\u{1f4c4}")]),
            attachment::content(
                vec![],
                vec![
                    attachment::name(vec![], vec![text("archive.zip")]),
                    attachment::meta(vec![], vec![text("Upload failed")]),
                ],
            ),
            attachment::actions(
                vec![],
                vec![attachment::action(
                    "Retry archive.zip",
                    true,
                    vec![],
                    vec![text("\u{21bb}")],
                )],
            ),
        ],
    )
}

/// `/themes/bubble/`（イシュー #2109、親 #2107。headless anatomy は
/// #2108）の原稿データ。`variant`/`align`/`group-position`/`selected`/
/// `state` は headless の `data-*` を `AttrEq`/`Attr`/`AttrEqAll` で参照
/// するのみで class 軸を持たない（`bubble.rs` モジュール doc参照）。
pub(crate) const BUBBLE: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "BubbleVariant（Solid/Outline/Plain、bubble.rs recipe() の data-variant 参照）で塗り・枠線・無装飾の 3 形態を切り替える",
        "MessageAlign（Start/End、会話系部品共通語彙）で root の水平整列を切り替える",
        "BubbleGroupPosition（Single/First/Middle/Last）で連続発言の隣接辺の角丸を連結する（算出はアプリ側の責務）",
        "root/content/reactions/reaction/collapse-trigger/collapse-content の 6 パーツで吹き出し 1 個を構造化する",
        "reactions/reaction はリアクションチップの表示のみを担う非対話パーツで、押下・集計・トグルは実装しない（docs/policy/intentional-non-adoption.md §3.25 規則 1）",
        "collapse-trigger/collapse-content は OpenState の open/closed を data-state・hidden・opacity フェードへ反映する（closed 時は hidden により display: none を伴うため、フェード遷移が見えるのはクライアントランタイムが hidden を外す前後のみ）",
        "root は --fandhe-bubble-bg/--fandhe-bubble-fg/--fandhe-bubble-border の 3 custom property を公開し、ColorPalette 軸を持たない代わりに呼び出し側が色を上書きできる",
    ],
    arguments: &[
        ArgRow {
            name: "variant",
            kind: "BubbleVariant",
            default: "Solid",
            description: "見た目の形態（Solid/Outline/Plain。data-variant として出力され、背景・文字色・枠線を切り替える）。",
        },
        ArgRow {
            name: "align",
            kind: "MessageAlign",
            default: "Start",
            description: "root の水平整列（Start/End。会話系部品共通語彙、center は持たない）。",
        },
        ArgRow {
            name: "group_position",
            kind: "BubbleGroupPosition",
            default: "Single",
            description: "連続発言中の位置（Single/First/Middle/Last。align と組み合わせて隣接辺の角丸を潰す。何番目かの算出は利用者責務）。",
        },
        ArgRow {
            name: "selected",
            kind: "bool",
            default: "false",
            description: "reaction の選択状態（true の場合 data-selected 存在属性を付与し、背景・枠線・文字色を強調する）。",
        },
        ArgRow {
            name: "state",
            kind: "OpenState",
            default: "Closed",
            description: "collapse-trigger/collapse-content の開閉状態（Open/Closed。data-state・aria-expanded・hidden へ反映する）。",
        },
        ArgRow {
            name: "controls / id",
            kind: "Option<&str>",
            default: "None",
            description: "collapse-trigger の aria-controls と collapse-content の id を関連付ける識別子（Some のときのみ出力）。",
        },
        ArgRow {
            name: "label",
            kind: "&str",
            default: "\"\"",
            description: "reactions の aria-label（空文字列のときは省略）。",
        },
    ],
    examples: &[
        ExampleEntry {
            title: "Conversation turn",
            description: "solid（align=start）と outline（align=end）の 1 往復の例です。",
            render: ex_bubble_conversation,
        },
        ExampleEntry {
            title: "Consecutive group position",
            description: "group-position（first/middle/last）で連続発言の隣接辺の角丸が連結される例です。",
            render: ex_bubble_group_position,
        },
        ExampleEntry {
            title: "Reactions and collapse",
            description: "reactions/reaction（selected あり/なし）と collapse-trigger/collapse-content（open/closed）の例です。",
            render: ex_bubble_reactions_and_collapse,
        },
    ],
    keyboard: &[
        KeyRow {
            key: "Space / Enter",
            description: "collapse-trigger はネイティブ button[type=\"button\"] として描画され、ブラウザ標準の click 発火に従う（wasm-full の toggle 配線は本イシューのスコープ外）。",
        },
    ],
    aria: &[
        AriaRow {
            attribute: "role=\"group\" / aria-label (reactions)",
            description: "reactions は role=\"group\" を固定付与し、渡した label が空でなければ aria-label へ出力する（bubble.rs モジュール doc「reactions/reaction」節参照）。",
        },
        AriaRow {
            attribute: "aria-expanded / aria-controls / data-state (collapse-trigger)",
            description: "collapse-trigger は開閉状態を aria-expanded と data-state に同期させ、controls を渡すと aria-controls で collapse-content と関連付ける。",
        },
        AriaRow {
            attribute: "hidden (collapse-content)",
            description: "collapse-content は closed のとき hidden 存在属性を付与し、JS なしの SSR でも閉状態を表現する。",
        },
    ],
    demo: None,
};

/// `/themes/attachment/`（イシュー #2112、親 #2110。headless anatomy は
/// #2111）の原稿データ。`variant`/`state`/`disabled` は headless の
/// `data-*` を `AttrEq`/`Attr` で参照するのみで class 軸を持たない
/// （`attachment.rs` モジュール doc参照）。
pub(crate) const ATTACHMENT: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "AttachmentVariant（File/Image、attachment.rs recipe() の data-variant 参照）で横並びの行カード・縦積みのサムネイルカードを切り替える",
        "AttachmentState（Idle/Uploading/Error）で通常表示・アップロード進行中・失敗時の枠色/文字色を切り替える",
        "root/media/content/name/meta/progress/actions/action の 8 パーツで添付ファイル 1 件を構造化する",
        "image 形態の actions は既定で隠れ、root への hover/focus-within で表示する（タッチ端末等 hover 機構を持たない端末では opacity: 0 の既定非表示規則自体が @media (hover: hover) 配下に限定されるため常時表示のまま残る、attachment.rs モジュール doc「actions の hover 表示とタッチ端末対策」節参照）",
        "progress は attachment scope の単純なスロットで、呼び出し側が styled Progress（root/track/range）を入れ子にする契約",
        "name/meta は整形済み文字列を受け取るだけのスロットで、byte → KB 変換等の数値整形は実装しない（docs/policy/intentional-non-adoption.md §3.25 規則 1）",
        "action は type=\"button\" 固定・label が空文字列でないときのみ aria-label・disabled はネイティブ disabled + data-disabled の両方に反映する",
    ],
    arguments: &[
        ArgRow {
            name: "variant",
            kind: "AttachmentVariant",
            default: "File",
            description: "表示形態（File/Image。data-variant として出力され、横並びの行カードと縦積みのサムネイルカードを切り替える）。",
        },
        ArgRow {
            name: "state",
            kind: "AttachmentState",
            default: "Idle",
            description: "アップロード状態（Idle/Uploading/Error。data-state として出力され、Error は枠色・meta 文字色を切り替える）。",
        },
        ArgRow {
            name: "disabled",
            kind: "bool",
            default: "false",
            description: "root の data-disabled（true の場合 opacity: 0.5 + cursor: not-allowed を適用）。",
        },
        ArgRow {
            name: "label",
            kind: "&str",
            default: "\"\"",
            description: "action の aria-label（空文字列のときは省略）。",
        },
        ArgRow {
            name: "disabled (action)",
            kind: "bool",
            default: "false",
            description: "action 自身の disabled（root の disabled とは独立。ネイティブ disabled + data-disabled の両方に反映し、cursor: not-allowed のみを適用する）。",
        },
    ],
    examples: &[
        ExampleEntry {
            title: "File variant",
            description: "横並びの行カード（idle）と media/content/actions パーツの例です。",
            render: ex_attachment_file,
        },
        ExampleEntry {
            title: "Image variant uploading",
            description: "縦積みのサムネイルカード（uploading）と progress スロットへ入れ子にした styled Progress の例です。",
            render: ex_attachment_image_uploading,
        },
        ExampleEntry {
            title: "Error and disabled",
            description: "アップロード失敗時（error）と disabled 状態の例です。",
            render: ex_attachment_error,
        },
    ],
    keyboard: &[
        KeyRow {
            key: "Tab / Shift+Tab",
            description: "action はネイティブ button[type=\"button\"] として描画され、ブラウザ標準のフォーカス順序・click 発火に従う（wasm-full の配線は不要、静的部品）。",
        },
    ],
    aria: &[
        AriaRow {
            attribute: "type=\"button\" / aria-label (action)",
            description: "action は type=\"button\" を固定付与し（フォーム内配置時の意図しない submit を防ぐ）、渡した label が空文字列でなければ aria-label へ出力する。",
        },
        AriaRow {
            attribute: "disabled / data-disabled (action)",
            description: "action の disabled 引数はネイティブ disabled 属性と data-disabled の両方へ反映する。",
        },
    ],
    demo: None,
};

/// `/themes/marker/` の Examples 節其の 1: `note` 形態（既定・neutral）。
fn ex_marker_note() -> Node {
    marker::root(
        MarkerRootProps {
            variant: MarkerVariant::Note,
            tone: MarkerTone::Neutral,
        },
        vec![],
        vec![
            marker::icon(vec![], vec![text("\u{25cf}")]),
            marker::content(vec![], vec![text("System note")]),
        ],
    )
}

/// `/themes/marker/` の Examples 節其の 2: `divider` 形態（行下の境界線）。
fn ex_marker_divider() -> Node {
    marker::root(
        MarkerRootProps {
            variant: MarkerVariant::Divider,
            tone: MarkerTone::Info,
        },
        vec![],
        vec![
            marker::icon(vec![], vec![text("\u{25cf}")]),
            marker::content(vec![], vec![text("New messages")]),
        ],
    )
}

/// `/themes/marker/` の Examples 節其の 3: `label` 形態（左右へ
/// separator パーツを挟んだ中央ラベル）と warning/danger tone の例。
fn ex_marker_label_tones() -> Node {
    let warning = marker::root(
        MarkerRootProps {
            variant: MarkerVariant::Label,
            tone: MarkerTone::Warning,
        },
        vec![],
        vec![marker::content(vec![], vec![text("Draft")])],
    );
    let danger = marker::root(
        MarkerRootProps {
            variant: MarkerVariant::Label,
            tone: MarkerTone::Danger,
        },
        vec![],
        vec![marker::content(vec![], vec![text("Failed")])],
    );
    div(vec![], vec![warning, danger])
}

/// `/themes/marker/`（イシュー #2115、親 #2113。headless anatomy は
/// #2114）の原稿データ。`variant`/`tone` は headless の `data-*` を
/// `AttrEq` で参照するのみで class 軸を持たない（`marker.rs` モジュール
/// doc参照）。
pub(crate) const MARKER: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "MarkerVariant（Note/Divider/Label、marker.rs recipe() の data-variant 参照）でインライン注記・行下の境界線・中央ラベル + 左右の線の 3 形態を切り替える",
        "MarkerTone（Neutral/Info/Warning/Danger、ColorPalette と同名 4 値）で注記の文字色・線色を切り替える",
        "root/icon/content の 3 パーツで注記行 1 個を構造化する（icon は装飾スロットとして aria-hidden=\"true\" を固定付与する）",
        "Label 形態は呼び出し側 children を styled Separator（horizontal・Solid、aria-hidden=\"true\"）2 個で挟んでから headless へ委譲する。区切り線は疑似要素を使わず DOM 上の separator パート再利用で描画する（marker.rs モジュール doc「区切り線の描画方式」節参照）",
        "Label 形態を使う場合は marker::stylesheet() に加えて separator::css() も併せて読み込む必要がある。marker::stylesheet() は separator 自体の border-width・border-style・margin 等の基本規則を含まない（marker.rs モジュール doc「stylesheet が separator の基本 CSS を含まない理由」節参照）",
        "ストリーミング中判定・注記の自動分類・タイムスタンプ整形は実装しない（docs/policy/intentional-non-adoption.md §3.25 規則 1）。root へ role=\"status\" 等を付けたい場合は呼び出し側が attrs で渡す",
    ],
    arguments: &[
        ArgRow {
            name: "variant",
            kind: "MarkerVariant",
            default: "Note",
            description: "表示形態（Note/Divider/Label。data-variant として出力される）。",
        },
        ArgRow {
            name: "tone",
            kind: "MarkerTone",
            default: "Neutral",
            description: "色調（Neutral/Info/Warning/Danger。data-tone として出力され、文字色・線色を切り替える）。",
        },
    ],
    examples: &[
        ExampleEntry {
            title: "Note variant",
            description: "インライン注記表示（既定・neutral）の例です。",
            render: ex_marker_note,
        },
        ExampleEntry {
            title: "Divider variant",
            description: "行下に境界線を伴う表示（info tone）の例です。",
            render: ex_marker_divider,
        },
        ExampleEntry {
            title: "Label variant with tones",
            description: "中央ラベル + 左右の線（warning/danger tone）の例です。",
            render: ex_marker_label_tones,
        },
    ],
    keyboard: &[
        KeyRow {
            key: "(N/A)",
            description: "静的な表示専用部品でありフォーカス可能な要素を持たない（wasm-full の配線は不要）。",
        },
    ],
    aria: &[
        AriaRow {
            attribute: "aria-hidden=\"true\" (icon)",
            description: "icon は装飾スロットとして aria-hidden=\"true\" を固定付与する。呼び出し側の aria-hidden=\"false\" 偽装は予約キー除去で無効化する。",
        },
        AriaRow {
            attribute: "role (root、未固定)",
            description: "root は role を固定付与しない。ストリーミング中の注記へ role=\"status\" を付けたい場合は呼び出し側が attrs で渡す運用とする。",
        },
        AriaRow {
            attribute: "aria-hidden=\"true\" (Label 形態の separator)",
            description: "Label 形態で挟み込む separator は装飾線として aria-hidden=\"true\" を持ち、支援技術による重複読み上げを避ける。",
        },
    ],
    demo: None,
};

// ---------------------------------------------------------------------
// Interactive: Sidebar（イシュー #2075）
// ---------------------------------------------------------------------

/// `crates/pre-styled-ui/src/sidebar.rs`（headless anatomy 22 パーツへ
/// variant/collapsible/side の意匠を重ねる薄い委譲層、イシュー #2073）。
/// 展開状態・group（label/action/content > menu）・menu-item（button/
/// action/badge）を 1 例に統合する。
fn ex_sidebar_expanded_with_group() -> Node {
    let state = Sidebar::new(SidebarState::Expanded);
    let props = SidebarProps::default();
    let menu = sidebar::menu(
        vec![],
        vec![sidebar::menu_item(
            vec![],
            vec![
                sidebar::menu_button(
                    &SidebarMenuButtonProps {
                        href: Some(""),
                        active: true,
                        ..Default::default()
                    },
                    None,
                    vec![],
                    vec![text("Dashboard")],
                ),
                sidebar::menu_action("Pin Dashboard", vec![], vec![]),
                sidebar::menu_badge(vec![], vec![text("3")]),
            ],
        )],
    );
    let group = sidebar::group(
        Some("example-sidebar-platform-label"),
        vec![],
        vec![
            sidebar::group_label(
                Some("example-sidebar-platform-label"),
                vec![],
                vec![text("Platform")],
            ),
            sidebar::group_content(vec![], vec![menu]),
        ],
    );
    let root = sidebar::root(
        &state,
        &props,
        "Main navigation",
        None,
        vec![],
        vec![
            sidebar::header(vec![], vec![text("Acme Inc")]),
            sidebar::content(vec![], vec![group]),
            sidebar::footer(vec![], vec![text("Ada Lovelace")]),
        ],
    );
    sidebar::provider(&state, &props, vec![], vec![root])
}

/// `SidebarCollapsible::Icon` による折りたたみ例。折りたたみ幅でも
/// `trigger` は可視のまま残る（`docs/policy/intentional-non-adoption.md`
/// §3.25 規則 1 により実際のトグル配線は `fandhe-frontend-wasm-full` の
/// 責務、静的掲示のためここでは `data-state="collapsed"` 固定）。
fn ex_sidebar_icon_collapsed() -> Node {
    let state = Sidebar::new(SidebarState::Collapsed);
    let props = SidebarProps {
        collapsible: SidebarCollapsible::Icon,
        ..SidebarProps::default()
    };
    let root = sidebar::root(
        &state,
        &props,
        "Main navigation (icon collapsed)",
        None,
        vec![],
        vec![sidebar::trigger(
            &state,
            "Toggle sidebar",
            None,
            vec![],
            vec![],
        )],
    );
    sidebar::provider(&state, &props, vec![], vec![root])
}

/// `SidebarVariant::Inset` の例。`inset` パーツを `provider` の直接の子に
/// 置く（`[data-variant="inset"] > [data-part="inset"]` 規則が直接子のみを
/// 対象にするため、`sidebar.rs` モジュール doc 参照）。
fn ex_sidebar_inset_variant() -> Node {
    let state = Sidebar::new(SidebarState::Expanded);
    let props = SidebarProps {
        variant: SidebarVariant::Inset,
        ..SidebarProps::default()
    };
    let root = sidebar::root(
        &state,
        &props,
        "Inset navigation",
        None,
        vec![],
        vec![sidebar::content(
            vec![],
            vec![sidebar::menu(
                vec![],
                vec![sidebar::menu_item(
                    vec![],
                    vec![sidebar::menu_button(
                        &SidebarMenuButtonProps::default(),
                        None,
                        vec![],
                        vec![text("Home")],
                    )],
                )],
            )],
        )],
    );
    sidebar::provider(
        &state,
        &props,
        vec![],
        vec![root, sidebar::inset(vec![], vec![text("Page content")])],
    )
}

pub(crate) const SIDEBAR: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "provider/root/header/content/footer/separator/input/group/group-label/group-content/group-action/menu/menu-item/menu-button/menu-action/menu-badge/menu-sub/menu-sub-item/menu-sub-button/rail/trigger/inset の 22 パーツでアプリシェル用サイドバーを構造化する",
        "SidebarVariant（Sidebar/Floating/Inset）・SidebarCollapsible（Offcanvas/Icon/None）・SidebarSide（Left/Right）はいずれも class 軸を持たず、headless が出力する data-variant/data-collapsible/data-side を CSS 属性セレクタとして参照するのみで見た目を切り替える",
        "--fandhe-sidebar-width / --fandhe-sidebar-width-icon / --fandhe-sidebar-width-mobile が展開幅・icon 折りたたみ幅・モバイル drawer 幅を制御し、--fandhe-color-sidebar-* 7 ロールが配色を担う",
        "icon 折りたたみ時、menu-button のラベルテキストは clip 手法で視覚的に非表示化しつつアクセシブルネームは維持する（WCAG 4.1.2）",
        "menu_skeleton はローディング装飾用ヘルパーで、ランダム幅を持たない決定的な固定幅の skeleton を返す",
        "Cmd/Ctrl+B のグローバルショートカット・モバイル判定（メディアクエリ）・menu-button の tooltip hover 配線はこの部品では実装しない（docs/policy/intentional-non-adoption.md §3.25 規則 1、fandhe-frontend-wasm-full の責務）",
    ],
    arguments: &[
        ArgRow {
            name: "state",
            kind: "&Sidebar",
            default: "",
            description: "provider/root/rail/trigger が共有する状態機械（SidebarState::Expanded/Collapsed）。",
        },
        ArgRow {
            name: "props",
            kind: "&SidebarProps",
            default: "SidebarProps::default()",
            description: "collapsible（Offcanvas/Icon/None）・variant（Sidebar/Floating/Inset）・side（Left/Right）・mobile（bool）をまとめた静的設定。",
        },
        ArgRow {
            name: "label",
            kind: "&str",
            default: "",
            description: "root（nav）の必須 aria-label。",
        },
        ArgRow {
            name: "id",
            kind: "Option<&str>",
            default: "None",
            description: "root の id。trigger の controls から aria-controls で関連付ける場合に指定する。",
        },
        ArgRow {
            name: "icon",
            kind: "Option<Node>",
            default: "None",
            description: "menu_button 固有の装飾アイコン引数（children の位置規約ではなく明示引数、PR #2245 対応）。",
        },
        ArgRow {
            name: "controls",
            kind: "Option<&str>",
            default: "None",
            description: "trigger が aria-controls として参照する root の id。",
        },
        ArgRow {
            name: "show_icon",
            kind: "bool",
            default: "",
            description: "menu_skeleton がアイコン用の円形 skeleton を先頭に合成するかどうか。",
        },
    ],
    examples: &[
        ExampleEntry {
            title: "Expanded with a group",
            description: "展開状態で group（label/action/content > menu）と menu-item（button/action/badge）を構造化する例です。",
            render: ex_sidebar_expanded_with_group,
        },
        ExampleEntry {
            title: "Icon-collapsed",
            description: "SidebarCollapsible::Icon による折りたたみ状態（data-state=\"collapsed\"）の例です。",
            render: ex_sidebar_icon_collapsed,
        },
        ExampleEntry {
            title: "Inset variant",
            description: "SidebarVariant::Inset。inset パーツを provider の直接の子として配置します。",
            render: ex_sidebar_inset_variant,
        },
    ],
    keyboard: &[
        KeyRow {
            key: "Tab / Shift+Tab",
            description: "trigger・menu-button・menu-sub-button 間のフォーカス移動（ブラウザ既定のフォーカス順序）。",
        },
        KeyRow {
            key: "Enter / Space",
            description: "trigger の開閉トグル（実際の開閉配線は fandhe-frontend-wasm-full、イシュー #2074 が担う）。",
        },
        KeyRow {
            key: "Cmd/Ctrl+B",
            description: "サイドバーの開閉ショートカット（本部品には実装されず、fandhe-frontend-wasm-full の後続配線、#2074 が担う）。",
        },
    ],
    aria: &[
        AriaRow {
            attribute: "aria-label (root)",
            description: "root（nav）の必須アクセシブルネーム。",
        },
        AriaRow {
            attribute: "aria-expanded / aria-controls (trigger)",
            description: "trigger は開閉状態と操作対象の root を伝える。",
        },
        AriaRow {
            attribute: "aria-current=\"page\" (menu-button/menu-sub-button)",
            description: "href を持ち active な menu-button/menu-sub-button（a 要素）にのみ付与される。",
        },
        AriaRow {
            attribute: "role=\"group\" / aria-labelledby (group)",
            description: "group は role=\"group\" を固定出力し、labelledby が Some のときのみ aria-labelledby を併記する。",
        },
        AriaRow {
            attribute: "tabindex=\"-1\" (rail)",
            description: "rail はマウス専用のドラッグ/クリック領域でありキーボードフォーカスの対象外。",
        },
        AriaRow {
            attribute: "aria-describedby (menu-button)",
            description: "tooltip との関連付けにのみ用いる（describedby が Some のときのみ出力）。",
        },
    ],
    demo: None,
};
