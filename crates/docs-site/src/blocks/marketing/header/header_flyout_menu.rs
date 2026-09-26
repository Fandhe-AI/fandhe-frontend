//! `header-flyout-menu` block（前半 #2855 で骨格・主要領域を実装済み、
//! 本ファイルは後半 #2856 が担当する残り領域・原稿仕上げまでを含む。
//! 親トラッキング #2854「Marketing / Header block 追加」配下、対応表 ID
//! R0981 を主参照とする合成例。集約元の差分（R0575/R0576/R0577/R0987/
//! R0988）の扱いは「3 形の並記」節・`site/blocks/header-flyout-menu.md`
//! の「原案差分メモ」節を参照（取得手段・ファイル名・内部コンポーネント
//! 識別子は記載しない契約、対応表 ID のみを記す）。
//!
//! **Marketing / Header カテゴリで最初の block**（イシュー #2734 の雛形を
//! 本 block 追加で卒業させた、`super`（`header/mod.rs`）参照）。
//!
//! # 使用部品
//!
//! `navigation-menu` / `button` / `icon` / `link` の 4 部品を合成する
//! （[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//!
//! # 3 形の並記（イシュー #2856）
//!
//! Demo は集約元のバリエーションを 1 ページに縦に並べて示す
//! （`footer_inline_nav` 前例と同型、caption `p` + 本体を組で繰り返す）。
//! 各形のルートには `data-blocks-header-flyout-menu-variant` を付与し、
//! フライアウトの value 名（trigger/content の id 導出元）を形ごとに
//! 一意にすることで、id 重複・宙に浮いた aria 参照を防いでいる
//! （`crates/docs-site/tests/blocks_contract.rs::
//! demo_output_has_no_dangling_aria_references_or_duplicate_ids` 参照）。
//!
//! | variant | 集約元 | 内容 |
//! |---|---|---|
//! | standard | R0981（主） | ロゴ・ナビ・アクション・「製品」フライアウト（Open、5 項目 + 補助 CTA）・「リソース」フライアウト（Closed、2 項目） |
//! | simple | R0575 | ナビ項目 1 件がテキストのみのドロップダウン（Open）、アクションはボタン 2 個（Outline + 既定、両方 disabled） |
//! | centered | R0577 | 48rem 以上でナビを中央寄せ、フライアウトは項目をアイコン付き 2 列グリッドで並べ末尾に補助 CTA |
//!
//! Demo に描画しない差分（`site/blocks/header-flyout-menu.md` の
//! 「原案差分メモ」節に記載）:
//! - R0576/R0988: モバイル drawer トリガー。狭幅ハンバーガーはこの相当だが
//!   開いた先の drawer 自体は無 JS のため描画しない。
//! - R0987: 2 項目のフライアウト。standard 形の「リソース」（2 項目・
//!   閉状態）と同じ構造のため独立インスタンス化しない。
//!
//! # 静的表示（無 JS、各 variant で 1 フライアウトを開いた状態で固定）
//!
//! docs サイトは JS ハイドレーションを行わないため、Demo は各 variant で
//! 主要なナビ項目のフライアウトを開いた状態のまま固定描画する。トリガーは
//! 押しても何も起きない no-op になるため、[`super::super::faq::
//! faq_accordion_centered`] 等の前例と同じ判断で `disabled: true`
//! （ネイティブ `disabled` 属性 + `aria-disabled="true"`）にしてフォーカス
//! 不能・操作不能であることを支援技術・キーボード双方に明示する。
//! `disabled_declarations()`（既定 `opacity: 0.5`）は [`LAYOUT_CSS`] で
//! 中和し、通常のナビ項目と同じ見た目に保つ（ハンバーガーボタン・主 CTA
//! ボタンの disabled も同様に中和する。主 CTA も遷移先・処理を持たない
//! ため同じ理由で `disabled: true` にする）。
//!
//! standard 形の「Resources」項目は閉じた状態（[`OpenState::Closed`]）で
//! 併記する。`content` は headless 層の fail-safe により `hidden` 属性が
//! 自動で付くが、`aria-controls`/`aria-labelledby` の参照先を欠落させない
//! ため、閉じていても content 自体は必ず描画する（showcase の修正ラウンド
//! と同じ教訓）。
//!
//! # 補助 CTA 行（イシュー #2856）
//!
//! standard 形の「製品」フライアウト・centered 形のフライアウトは、項目
//! リストの末尾に `[data-blocks-header-flyout-menu-panel-footer]` の行を
//! 持つ。中はアイコン付きの `link::root` 2 本（架空の文言）で、送信・
//! 遷移先の処理は持たない静的リンクである。
//!
//! # フライアウトの高さ確保（`.blocks-demo` の overflow-x: auto 対策）
//!
//! `crate::blocks` の `.blocks-demo` ラッパは横方向のみ `overflow-x: auto`
//! だが縦方向はブロック要素の自然な高さで確定するため、絶対配置される
//! フライアウトパネル（styled navigation-menu の `content` recipe が
//! `position: absolute; top: 100%` を持つ）がラッパの外へはみ出すと縦方向に
//! クリップされる。[`LAYOUT_CSS`] は `>= 48rem` でルートへ
//! `min-block-size` を与えてこれを防ぐ（狭い幅ではフライアウト自体を
//! 表示しないハンバーガー畳み構成のため不要）。
//!
//! # レスポンシブ（48rem 境界、mobile-first 直書き）
//!
//! 既存 block（例: [`super::super::contact::contact_split_form_image`]）と
//! 同じ mobile-first の `@media (min-width: 48rem)` 直書きを使う
//! （[`fandhe_frontend_pre_styled_ui::theme::Breakpoint::Md`] と同じ値。
//! container query の前例は本リポジトリに無く、本 block でも新規導入しない）。
//! 既定（狭い幅）はナビ・アクションを隠しハンバーガーのみを表示し、
//! `>= 48rem` で反転する。
//!
//! # CSS フックに data 属性を使う理由
//!
//! `button::button`/`button::icon_button`/`link::root`/`icon::icon` は
//! いずれも `drop_class_attr` により呼び出し側 `attrs` の `class` を黙って
//! 除去する契約を持つため、Demo 固有のスタイルフックは
//! `data-blocks-header-flyout-menu-*` 属性で渡す（headless 由来の
//! `navigation_menu::root`/`list`/`item`/`trigger`/`content`/`link` は
//! `drop_class_attr` を経由しないため `class` も使えるが、本 block では
//! 一貫性のため同じく data 属性に寄せる）。素の `header`/`div`/`span`/`p`
//! には `class` がそのまま効くため、それらは `.blocks-header-flyout-menu-*`
//! クラスセレクタを使う。
//!
//! # id 接頭辞
//!
//! `id`/`aria-controls`/`aria-labelledby` はいずれも
//! `blocks-header-flyout-menu-{value}-{trigger|content}` の形で項目値から
//! 一意に導出する（`crates/docs-site/tests/blocks_contract.rs::
//! demo_output_has_no_dangling_aria_references_or_duplicate_ids` が全 block
//! 横断で id 重複・宙に浮いた参照を検査する）。
//!
//! # `href` の方針・文言
//!
//! `href` は実在する自リポジトリ・組織の URL に限る（`linkcheck` が
//! fail-closed で検証する死リンク `href="#"` を使わない）。ナビ・
//! フライアウトの文言は架空の日本語（実在の企業名・個人情報は含まない）。
//!
//! # `<form>` を持たない・データ取得/送信を行わない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である。アクション行のボタンは `button::button` の既定
//! `type="button"` のまま送信先を持たない。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, el, header, p, span, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::navigation_menu::{self, NavigationMenuProps, OpenState};
use fandhe_frontend_pre_styled_ui::Size;

/// 実在の自リポジトリ URL（`href` の方針、モジュール doc参照）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";
/// 実在の自組織 URL（Company リンク用）。
const ORG: &str = "https://github.com/Fandhe-AI";

/// 「Product」フライアウトの項目一覧（アイコン path・ラベル・説明・
/// href の組。件数を単体テストから参照するため `const` 配列にする）。
const PRODUCT_FLYOUT_ITEMS: &[(&str, &str, &str, &str)] = &[
    (
        "M4 4h7v7H4zM13 4h7v7h-7zM4 13h7v7H4zM13 13h7v7h-7z",
        "ダッシュボード",
        "主要な指標をひと目で確認できる管理画面",
        REPO,
    ),
    (
        "M12 2l2.4 7.2H22l-6 4.6 2.3 7.2L12 16.4 5.7 21l2.3-7.2-6-4.6h7.6z",
        "自動化",
        "繰り返し作業をワークフローとして自動化",
        REPO,
    ),
    (
        "M4 12a8 8 0 1116 0 8 8 0 01-16 0zm8-5v5l3 3",
        "レポート",
        "チームの成果を定期レポートとして共有",
        REPO,
    ),
    (
        "M12 2l9 4.9v9.6L12 22l-9-4.9V6.9z",
        "連携",
        "他ツールとの連携でデータを一元管理",
        REPO,
    ),
    (
        "M12 2l7 3v6c0 4.9-3 8.7-7 11-4-2.3-7-6.1-7-11V5z",
        "セキュリティ",
        "アクセス権限と監査ログを一元管理",
        REPO,
    ),
];

/// 「Resources」フライアウトの項目一覧。
const RESOURCES_FLYOUT_ITEMS: &[(&str, &str, &str, &str)] = &[
    (
        "M6 2h9l3 3v17H6zM15 2v3h3",
        "ドキュメント",
        "導入手順と API リファレンス",
        REPO,
    ),
    (
        "M12 2a5 5 0 015 5 5 5 0 01-10 0 5 5 0 015-5zM4 22a8 8 0 0116 0",
        "コミュニティ",
        "利用者同士で情報交換できる場",
        REPO,
    ),
];

/// centered 形フライアウトの項目一覧（2 列グリッドで並べる、R0577）。
const PLATFORM_FLYOUT_ITEMS: &[(&str, &str, &str, &str)] = &[
    (
        "M4 4h16v4H4zM4 12h16v8H4z",
        "ワークスペース",
        "チームで共有する作業スペース",
        REPO,
    ),
    (
        "M12 2l9 4.9v9.6L12 22l-9-4.9V6.9z",
        "API",
        "外部サービスと連携する REST/gRPC API",
        REPO,
    ),
    (
        "M4 12a8 8 0 1116 0 8 8 0 01-16 0zm8-5v5l3 3",
        "モニタリング",
        "稼働状況をリアルタイムに可視化",
        REPO,
    ),
    (
        "M12 2l7 3v6c0 4.9-3 8.7-7 11-4-2.3-7-6.1-7-11V5z",
        "アクセス制御",
        "ロールベースの権限管理",
        REPO,
    ),
];

/// simple 形フライアウトの項目一覧（テキストのみ、R0575）。
const SERVICES_TEXT_ITEMS: &[(&str, &str)] = &[
    ("導入支援", REPO),
    ("運用代行", REPO),
    ("トレーニング", REPO),
];

/// 装飾用の幾何図形アイコン（`label: None`）を組み立てる（自作 SVG、実在
/// ブランドのロゴを模さない）。
fn geo_icon(size: Size, d: &str) -> Node {
    icon(
        &IconProps {
            size,
            label: None,
            ..IconProps::default()
        },
        vec![],
        vec![el("path", vec![("d", d)], vec![])],
    )
}

/// シェブロン（開閉インジケータ）用の装飾アイコン。
fn chevron_icon() -> Node {
    geo_icon(Size::Sm, "M6 9l6 6 6-6")
}

/// ハンバーガー（3 本線）アイコン。
fn hamburger_icon() -> Node {
    geo_icon(Size::Md, "M3 6h18v2H3zM3 11h18v2H3zM3 16h18v2H3z")
}

/// ロゴ（幾何図形アイコン + ブランド名テキスト）。実在ブランドのロゴを
/// 模さない自作の SVG。
fn logo() -> Node {
    link::root(
        REPO,
        &LinkProps::default(),
        vec![("data-blocks-header-flyout-menu-logo", "")],
        vec![
            geo_icon(Size::Md, "M4 4h16v6H4zM4 14h16v6H4z"),
            span(
                vec![("class", "blocks-header-flyout-menu-brand")],
                vec![text("Fandhe Frontend")],
            ),
        ],
    )
}

/// フライアウト 1 件分の項目（アイコン + ラベル + 説明の 3 要素）。
fn flyout_item(icon_path: &str, label: &str, description: &str, href: &str) -> Node {
    navigation_menu::link(
        href,
        false,
        vec![("data-blocks-header-flyout-menu-item", "")],
        vec![
            geo_icon(Size::Md, icon_path),
            div(
                vec![("class", "blocks-header-flyout-menu-item-text")],
                vec![
                    span(
                        vec![("class", "blocks-header-flyout-menu-item-label")],
                        vec![text(label)],
                    ),
                    p(
                        vec![("class", "blocks-header-flyout-menu-item-description")],
                        vec![text(description)],
                    ),
                ],
            ),
        ],
    )
}

/// アイコン + ラベル + 説明の項目一覧を [`flyout_item`] へ写像する
/// （`nav_item_with_flyout` へ渡す `items` の組み立て用）。
fn flyout_items(items: &[(&str, &str, &str, &str)]) -> Vec<Node> {
    items
        .iter()
        .map(|(icon_path, label, description, href)| {
            flyout_item(icon_path, label, description, href)
        })
        .collect()
}

/// テキストのみの簡易フライアウト項目（R0575。アイコン・説明を持たない）。
fn flyout_text_item(label: &str, href: &str) -> Node {
    navigation_menu::link(
        href,
        false,
        vec![("data-blocks-header-flyout-menu-text-item", "")],
        vec![text(label)],
    )
}

/// フライアウト下部の補助 CTA 行 1 本（アイコン付き `link::root`、
/// モジュール doc「補助 CTA 行」節）。
fn footer_link(icon_path: &str, label: &str, href: &str) -> Node {
    link::root(
        href,
        &LinkProps::default(),
        vec![("data-blocks-header-flyout-menu-panel-footer-link", "")],
        vec![geo_icon(Size::Sm, icon_path), text(label)],
    )
}

/// 補助 CTA 行のコンテナ（2 本のリンクを横並びで持つ）。
fn panel_footer() -> Node {
    div(
        vec![("data-blocks-header-flyout-menu-panel-footer", "")],
        vec![
            footer_link("M5 12h14M13 6l6 6-6 6", "デモを見る", REPO),
            footer_link("M2 12h20M12 2v20", "営業に相談する", REPO),
        ],
    )
}

/// caption（並記された各形の見出し、`footer_inline_nav` 前例と同型）。
fn caption(label: &str) -> Node {
    p(
        vec![("class", "blocks-header-flyout-menu-caption")],
        vec![text(label)],
    )
}

/// トリガー付きナビ項目（フライアウトを持つ）。`disabled: true` 固定
/// （モジュール doc「静的表示」節）。`footer`（補助 CTA 行）・
/// `columns`（パネルの列数、centered 形の 2 列グリッド用）は任意。
fn nav_item_with_flyout(
    props: &NavigationMenuProps,
    value: &str,
    label: &str,
    state: OpenState,
    items: Vec<Node>,
    footer: Option<Node>,
    columns: Option<&'static str>,
) -> Node {
    let trigger_id = format!("blocks-header-flyout-menu-{value}-trigger");
    let content_id = format!("blocks-header-flyout-menu-{value}-content");

    let mut panel_attrs = vec![("data-blocks-header-flyout-menu-panel", "")];
    if let Some(cols) = columns {
        panel_attrs.push(("data-blocks-header-flyout-menu-columns", cols));
    }

    let mut panel_children = items;
    if let Some(footer_node) = footer {
        panel_children.push(footer_node);
    }

    navigation_menu::item(
        state,
        false,
        props,
        value,
        vec![],
        vec![
            navigation_menu::trigger(
                state,
                true,
                value,
                Some(trigger_id.as_str()),
                Some(content_id.as_str()),
                vec![],
                vec![
                    span(vec![], vec![text(label)]),
                    navigation_menu::item_indicator(
                        state,
                        props,
                        value,
                        vec![],
                        vec![chevron_icon()],
                    ),
                ],
            ),
            navigation_menu::content(
                state,
                props,
                value,
                Some(content_id.as_str()),
                Some(trigger_id.as_str()),
                panel_attrs,
                panel_children,
            ),
        ],
    )
}

/// トリガーを持たない素のリンク項目（Pricing/Company）。
fn nav_link_item(props: &NavigationMenuProps, value: &str, label: &str, href: &str) -> Node {
    navigation_menu::item(
        OpenState::Closed,
        false,
        props,
        value,
        vec![],
        vec![navigation_menu::link(
            href,
            false,
            vec![],
            vec![text(label)],
        )],
    )
}

/// 横並びナビ本体（standard 形、R0981）。
fn main_nav() -> Node {
    let props = NavigationMenuProps::default();
    navigation_menu::root(
        &props,
        "Main",
        vec![("data-blocks-header-flyout-menu-nav", "")],
        vec![navigation_menu::list(
            &props,
            vec![],
            vec![
                nav_item_with_flyout(
                    &props,
                    "product",
                    "製品",
                    OpenState::Open,
                    flyout_items(PRODUCT_FLYOUT_ITEMS),
                    Some(panel_footer()),
                    None,
                ),
                nav_item_with_flyout(
                    &props,
                    "resources",
                    "リソース",
                    OpenState::Closed,
                    flyout_items(RESOURCES_FLYOUT_ITEMS),
                    None,
                    None,
                ),
                nav_link_item(&props, "pricing", "料金", REPO),
                nav_link_item(&props, "company", "会社概要", ORG),
            ],
        )],
    )
}

/// アクション行（ログイン + 主 CTA）。無 JS デモのため主 CTA も
/// ハンバーガーと同じ理由（モジュール doc「静的表示」節）で
/// `disabled: true` 固定にし、フォーカス・クリック不能を明示する。
fn actions() -> Node {
    div(
        vec![("data-blocks-header-flyout-menu-actions", "")],
        vec![
            link::root(REPO, &LinkProps::default(), vec![], vec![text("ログイン")]),
            button::button(
                &ButtonProps {
                    disabled: true,
                    ..ButtonProps::default()
                },
                vec![("data-blocks-header-flyout-menu-cta", "")],
                vec![text("使ってみる")],
            ),
        ],
    )
}

/// simple 形のアクション行（R0575。ボタン 2 個、どちらも `disabled: true`）。
fn actions_simple() -> Node {
    div(
        vec![("data-blocks-header-flyout-menu-actions", "")],
        vec![
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Outline,
                    disabled: true,
                    ..ButtonProps::default()
                },
                vec![("data-blocks-header-flyout-menu-cta-secondary", "")],
                vec![text("ログイン")],
            ),
            button::button(
                &ButtonProps {
                    disabled: true,
                    ..ButtonProps::default()
                },
                vec![("data-blocks-header-flyout-menu-cta", "")],
                vec![text("使ってみる")],
            ),
        ],
    )
}

/// ハンバーガーボタン（狭い幅専用、押しても何も起きないため
/// `disabled: true` 固定。`aria-controls` は開く先を描画しないため付けない）。
fn hamburger() -> Node {
    button::icon_button(
        &ButtonProps {
            variant: ButtonVariant::Ghost,
            disabled: true,
            ..ButtonProps::default()
        },
        "Open main menu",
        vec![("data-blocks-header-flyout-menu-toggle", "")],
        vec![hamburger_icon()],
    )
}

/// standard 形（R0981 主参照）。
fn standard_header() -> Node {
    header(
        vec![
            ("class", "blocks-header-flyout-menu-layout"),
            ("data-blocks-header-flyout-menu-root", ""),
            ("data-blocks-header-flyout-menu-variant", "standard"),
        ],
        vec![logo(), main_nav(), actions(), hamburger()],
    )
}

/// simple 形（R0575。テキストのみのドロップダウン + ボタン 2 個）。
fn simple_header() -> Node {
    let props = NavigationMenuProps::default();
    let nav = navigation_menu::root(
        &props,
        "Main (text only)",
        vec![("data-blocks-header-flyout-menu-nav", "")],
        vec![navigation_menu::list(
            &props,
            vec![],
            vec![nav_item_with_flyout(
                &props,
                "services",
                "サービス",
                OpenState::Open,
                SERVICES_TEXT_ITEMS
                    .iter()
                    .map(|(label, href)| flyout_text_item(label, href))
                    .collect(),
                None,
                None,
            )],
        )],
    );
    header(
        vec![
            ("class", "blocks-header-flyout-menu-layout"),
            ("data-blocks-header-flyout-menu-root", ""),
            ("data-blocks-header-flyout-menu-variant", "simple"),
        ],
        vec![logo(), nav, actions_simple(), hamburger()],
    )
}

/// centered 形（R0577。48rem 以上でナビを中央寄せ、フライアウトは
/// アイコン付き項目を 2 列グリッドで並べ末尾に補助 CTA を持つ）。
fn centered_header() -> Node {
    let props = NavigationMenuProps::default();
    let nav = navigation_menu::root(
        &props,
        "Main (centered)",
        vec![("data-blocks-header-flyout-menu-nav", "")],
        vec![navigation_menu::list(
            &props,
            vec![],
            vec![
                nav_item_with_flyout(
                    &props,
                    "platform",
                    "プラットフォーム",
                    OpenState::Open,
                    flyout_items(PLATFORM_FLYOUT_ITEMS),
                    Some(panel_footer()),
                    Some("2"),
                ),
                nav_link_item(&props, "docs", "ドキュメント", REPO),
                nav_link_item(&props, "about", "会社概要", ORG),
            ],
        )],
    );
    header(
        vec![
            ("class", "blocks-header-flyout-menu-layout"),
            ("data-blocks-header-flyout-menu-root", ""),
            ("data-blocks-header-flyout-menu-variant", "centered"),
        ],
        vec![logo(), nav, actions(), hamburger()],
    )
}

/// `header-flyout-menu` の Demo 本体。集約元のバリエーション（standard/
/// simple/centered）を縦に並記する（モジュール doc「3 形の並記」節、
/// `footer_inline_nav` 前例と同型）。呼び出しごとに同一の `Node` を返す
/// 純関数。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-header-flyout-menu-stack")],
        vec![
            caption("標準形"),
            standard_header(),
            caption("テキストのみ"),
            simple_header(),
            caption("中央寄せ"),
            centered_header(),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/header-flyout-menu/",
    title: "header-flyout-menu",
    category: BlockCategory::Header,
    rust_source: "crates/docs-site/src/blocks/marketing/header/header_flyout_menu.rs",
    demo_class: "blocks-header-flyout-menu",
    parts: &[
        Part {
            label: "Navigation Menu",
            path: "/themes/navigation-menu/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Icon",
            path: "/themes/icon/",
        },
        Part {
            label: "Link",
            path: "/themes/link/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `header_flyout_menu` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節。他 block と同型で `pub(super)` では
/// なく本ファイル内 private 定数として `super::stylesheet` 経由の
/// `push_css` で連結される）。
///
/// セレクタは `.blocks-header-flyout-menu-*` と
/// `[data-blocks-header-flyout-menu-*]`、および styled navigation-menu /
/// styled button の `[data-scope]`/`[data-part]` セレクタとの複合セレクタ
/// （disabled 中和・レスポンシブ切り替え、モジュール doc「静的表示」節）
/// のみを用い、他 block や部品の素のセレクタへ影響させない。
///
/// # CSS 特異性（Bugbot 指摘の是正、モジュール doc「レスポンシブ」節）
///
/// `navigation-menu`/`button` の recipe は `root`/`link` slot へ
/// `[data-scope="..."][data-part="..."]`（属性セレクタ 2 個、詳細度
/// `(0,2,0)`）の無条件 base 宣言・`(0,3,0)` の disabled state 宣言を持つ
/// （`crates/pre-styled-ui/src/navigation_menu.rs`/`button.rs` 参照）。
/// これらは `styled navigation-menu`/`styled button` を素通しする本 Demo
/// では headless 由来の `data-scope`/`data-part` と Demo 固有の
/// `data-blocks-header-flyout-menu-*` が同一要素に併記されるため、
/// 単一属性セレクタ（詳細度 `(0,1,0)`）の上書きは常にこれらへ負ける
/// （後勝ちの記述順に関係なく詳細度が低いため）。本 CSS は
/// `data-scope`/`data-part` を明示的に含めた複合セレクタ（recipe 側より
/// 属性セレクタ数を必ず 1 個以上多くする）でこれを解決する。対象は
/// レスポンシブ切り替え（nav の `[data-part="root"]`・ハンバーガーの
/// `[data-part="root"]`）・disabled 中和（ハンバーガー・主 CTA・
/// フライアウトトリガーの各 `[data-disabled]` state）・フライアウト項目の
/// アイコン縦位置（`link` の `align-items` base 宣言）・centered 形の
/// nav 中央寄せ（次節）の 5 種。
///
/// # centered 形のナビ中央寄せ（codex/Bugbot 指摘の是正、イシュー #2856）
///
/// 当初は `[data-blocks-header-flyout-menu-root][...variant="centered"]`
/// へ `justify-content: center` を宣言していたが、これは
/// `.blocks-header-flyout-menu-layout`（`display: flex`）の**行全体**を
/// 中央寄せする宣言であり、ロゴ・ナビ・アクション・ハンバーガーの 4 要素
/// がひとかたまりとして中央へ寄るだけで、ナビ自体の中心はヘッダー中心
/// からロゴ幅・アクション幅の差分だけずれる（48rem 以上でナビを中央寄せ
/// する、というモジュール doc の表の記述を満たさない）。是正として、
/// ルート側は `justify-content` を持たず（`.blocks-header-flyout-menu-layout`
/// の既定 `space-between` のまま）、nav 要素自身へ `flex: 1;
/// justify-content: center` を宣言する。ロゴ・アクション+ハンバーガーの
/// 両端に挟まれた残り空間の中でナビ自身の内容（`list`）が中央寄せされる
/// （ロゴ幅とアクション+ハンバーガー幅がおおむね対称な本 Demo の構図では
/// ヘッダー全体の視覚中心に近づく）。
const LAYOUT_CSS: &str = "\
.blocks-header-flyout-menu-stack {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n}\n\
.blocks-header-flyout-menu-caption {\n  margin: 0;\n  font-size: var(--fandhe-font-font-size-sm, 0.875rem);\n  color: var(--fandhe-color-fg-muted);\n}\n\
.blocks-header-flyout-menu-layout {\n  display: flex;\n  align-items: center;\n  justify-content: space-between;\n  gap: var(--fandhe-space-4);\n  inline-size: 100%;\n}\n\
[data-scope=\"navigation-menu\"][data-part=\"root\"][data-blocks-header-flyout-menu-nav] {\n  display: none;\n}\n\
[data-blocks-header-flyout-menu-actions] {\n  display: none;\n  align-items: center;\n  gap: var(--fandhe-space-3);\n}\n\
[data-scope=\"button\"][data-part=\"root\"][data-blocks-header-flyout-menu-toggle] {\n  display: inline-flex;\n}\n\
[data-scope=\"button\"][data-part=\"root\"][data-blocks-header-flyout-menu-toggle][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
[data-scope=\"button\"][data-part=\"root\"][data-blocks-header-flyout-menu-cta][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
[data-scope=\"button\"][data-part=\"root\"][data-blocks-header-flyout-menu-cta-secondary][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
.blocks-header-flyout-menu-brand {\n  font-weight: var(--fandhe-font-font-weight-medium);\n}\n\
[data-blocks-header-flyout-menu-panel] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1);\n  inline-size: min(26rem, 80vw);\n}\n\
[data-blocks-header-flyout-menu-panel][hidden] {\n  display: none;\n}\n\
[data-blocks-header-flyout-menu-panel][data-blocks-header-flyout-menu-columns=\"2\"] {\n  display: grid;\n  grid-template-columns: repeat(2, minmax(0, 1fr));\n  gap: var(--fandhe-space-3);\n  inline-size: min(36rem, 90vw);\n}\n\
[data-blocks-header-flyout-menu-panel][data-blocks-header-flyout-menu-columns] > [data-blocks-header-flyout-menu-panel-footer] {\n  grid-column: 1 / -1;\n}\n\
[data-blocks-header-flyout-menu-panel-footer] {\n  display: flex;\n  gap: var(--fandhe-space-3);\n  padding-block-start: var(--fandhe-space-2);\n  margin-block-start: var(--fandhe-space-1);\n  border-block-start: 1px solid var(--fandhe-color-border);\n  background: var(--fandhe-color-bg-subtle);\n}\n\
[data-scope=\"navigation-menu\"][data-part=\"link\"][data-blocks-header-flyout-menu-item] {\n  align-items: flex-start;\n}\n\
.blocks-header-flyout-menu-item-text {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1);\n}\n\
.blocks-header-flyout-menu-item-label {\n  font-weight: var(--fandhe-font-font-weight-medium);\n}\n\
.blocks-header-flyout-menu-item-description {\n  margin: 0;\n  color: var(--fandhe-color-fg-muted);\n  font-size: var(--fandhe-font-font-size-sm, 0.875rem);\n}\n\
[data-blocks-header-flyout-menu-root] [data-scope=\"navigation-menu\"][data-part=\"trigger\"][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
@media (min-width: 48rem) {\n  \
[data-scope=\"navigation-menu\"][data-part=\"root\"][data-blocks-header-flyout-menu-nav] {\n    display: block;\n  }\n  \
[data-blocks-header-flyout-menu-actions] {\n    display: flex;\n  }\n  \
[data-scope=\"button\"][data-part=\"root\"][data-blocks-header-flyout-menu-toggle] {\n    display: none;\n  }\n  \
.blocks-header-flyout-menu-layout {\n    align-items: flex-start;\n    min-block-size: 24rem;\n  }\n  \
[data-blocks-header-flyout-menu-root][data-blocks-header-flyout-menu-variant=\"centered\"] {\n    gap: var(--fandhe-space-8);\n  }\n  \
[data-blocks-header-flyout-menu-root][data-blocks-header-flyout-menu-variant=\"centered\"] [data-scope=\"navigation-menu\"][data-part=\"root\"][data-blocks-header-flyout-menu-nav] {\n    flex: 1;\n    display: flex;\n    justify-content: center;\n  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS, PRODUCT_FLYOUT_ITEMS};
    use fandhe_frontend_core::render;

    /// Demo が期待する 4 種の部品・非対話制約を満たすことの単体回帰
    /// （`crates/docs-site/tests/blocks_contract.rs` の横断検査と重複し
    /// 過ぎない範囲での個別固定）。
    #[test]
    fn demo_composes_expected_parts() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"navigation-menu\"",
            "data-scope=\"button\"",
            "data-scope=\"icon\"",
            "data-scope=\"link\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert!(html.contains(r#"type="button""#));
        assert!(!html.contains("<form"));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("src=\"data:"));
    }

    /// 3 形すべてが並記されること（モジュール doc「3 形の並記」節）。
    #[test]
    fn demo_renders_all_three_variants() {
        let html = render(&demo());
        for variant in ["standard", "simple", "centered"] {
            assert!(
                html.contains(&format!(
                    "data-blocks-header-flyout-menu-variant=\"{variant}\""
                )),
                "variant {variant} should render, html={html}"
            );
        }
        assert_eq!(html.matches("blocks-header-flyout-menu-caption").count(), 3);
    }

    /// 各 variant でちょうど 1 個のフライアウトが開いた状態で固定され、
    /// standard 形は開いている Product content 内のリンク数が項目数と
    /// 一致し、閉じている Resources content には `hidden` が付くこと
    /// （モジュール doc「静的表示」節）。variant の描画順は
    /// standard → simple → centered で決定的なため（[`demo`]）、各 variant
    /// の開始位置でスライスして個別に検証する。
    #[test]
    fn each_variant_renders_exactly_one_open_flyout() {
        let html = render(&demo());
        let variants = ["standard", "simple", "centered"];
        let mut starts: Vec<usize> = variants
            .iter()
            .map(|variant| {
                html.find(&format!(
                    "data-blocks-header-flyout-menu-variant=\"{variant}\""
                ))
                .unwrap_or_else(|| panic!("variant {variant} should render"))
            })
            .collect();
        starts.push(html.len());

        for (i, variant) in variants.iter().enumerate() {
            let slice = &html[starts[i]..starts[i + 1]];
            assert_eq!(
                slice.matches(r#"aria-expanded="true""#).count(),
                1,
                "variant={variant} slice={slice}"
            );
        }

        let standard_slice = &html[starts[0]..starts[1]];
        assert_eq!(
            standard_slice.matches(r#"aria-expanded="false""#).count(),
            1,
            "standard_slice={standard_slice}"
        );

        let product_content_start = standard_slice
            .find(r#"id="blocks-header-flyout-menu-product-content""#)
            .expect("product content should render");
        let resources_content_start = standard_slice
            .find(r#"id="blocks-header-flyout-menu-resources-content""#)
            .expect("resources content should render");
        assert!(
            product_content_start < resources_content_start,
            "product content should render before resources content"
        );

        let open_slice = &standard_slice[product_content_start..resources_content_start];
        assert_eq!(
            open_slice
                .matches("data-scope=\"navigation-menu\" data-part=\"link\"")
                .count(),
            PRODUCT_FLYOUT_ITEMS.len(),
            "open_slice={open_slice}"
        );

        let closed_slice = &standard_slice[resources_content_start..];
        assert!(
            closed_slice.contains("hidden"),
            "closed content should carry hidden attribute"
        );
    }

    /// standard 形（Product フライアウト）・centered 形（Platform
    /// フライアウト）の末尾に補助 CTA 行が現れること（モジュール doc
    /// 「補助 CTA 行」節）。simple 形はテキストのみの項目で CTA 行を
    /// 持たない。
    #[test]
    fn standard_and_centered_flyouts_have_cta_footer() {
        let html = render(&demo());
        assert_eq!(
            html.matches("data-blocks-header-flyout-menu-panel-footer=\"\"")
                .count(),
            2,
            "standard・centered の 2 箇所に補助 CTA 行が現れるべき, html={html}"
        );
        assert_eq!(
            html.matches("data-blocks-header-flyout-menu-panel-footer-link")
                .count(),
            4,
            "補助 CTA 行はそれぞれ 2 本のリンクを持つべき, html={html}"
        );
    }

    /// simple 形の項目がアイコンを持たないこと（テキストのみ、R0575）。
    #[test]
    fn simple_flyout_items_have_no_icon() {
        let html = render(&demo());
        let services_start = html
            .find(r#"id="blocks-header-flyout-menu-services-content""#)
            .expect("services content should render");
        // 同じ simple variant のアクション行（`hamburger`/`actions_simple`
        // のアイコン）を巻き込まないよう、次に来る actions 行の手前までに
        // 限定する（`hamburger_icon` も `data-scope="icon"` を持つため）。
        let content_end = html[services_start..]
            .find("data-blocks-header-flyout-menu-actions")
            .map(|offset| services_start + offset)
            .expect("actions row should follow services content");
        let slice = &html[services_start..content_end];
        assert!(
            !slice.contains("data-scope=\"icon\""),
            "simple flyout should not render icons, slice={slice}"
        );
        assert!(slice.contains("data-blocks-header-flyout-menu-text-item"));
    }

    /// simple 形のアクション行がボタン 2 個（Outline + 既定）で、どちらも
    /// disabled であること（モジュール doc「3 形の並記」節）。
    #[test]
    fn simple_variant_has_two_disabled_buttons() {
        let html = render(&demo());
        let simple_start = html
            .find("data-blocks-header-flyout-menu-variant=\"simple\"")
            .expect("simple variant should render");
        let centered_start = html
            .find("data-blocks-header-flyout-menu-variant=\"centered\"")
            .expect("centered variant should render");
        let slice = &html[simple_start..centered_start];
        assert!(slice.contains("data-blocks-header-flyout-menu-cta-secondary"));
        assert!(slice.contains("data-blocks-header-flyout-menu-cta=\"\""));
        assert_eq!(
            slice.matches("aria-disabled=\"true\"").count(),
            3,
            "hamburger + 2 action buttons should be disabled, slice={slice}"
        );
    }

    /// centered 形のパネルが 2 列グリッドの data 属性・CSS 規則を持つこと。
    #[test]
    fn centered_panel_uses_two_column_grid() {
        let html = render(&demo());
        assert!(html.contains(r#"data-blocks-header-flyout-menu-columns="2""#));
        assert!(LAYOUT_CSS.contains(
            "[data-blocks-header-flyout-menu-panel][data-blocks-header-flyout-menu-columns=\"2\"] {\n  display: grid;"
        ));
    }

    /// centered 形は行全体ではなく nav 自身が中央寄せされること（codex/
    /// Bugbot 指摘の是正回帰、[`LAYOUT_CSS`] doc「centered 形のナビ中央
    /// 寄せ」節）。ルート（`.blocks-header-flyout-menu-layout`）に
    /// `justify-content: center` を持たせない（既定の `space-between` の
    /// まま）ことと、nav 要素自身に `flex: 1; justify-content: center` が
    /// 宣言されることの両方を固定する。
    #[test]
    fn centered_variant_centers_nav_itself_not_the_whole_row() {
        assert!(
            !LAYOUT_CSS.contains(
                "[data-blocks-header-flyout-menu-root][data-blocks-header-flyout-menu-variant=\"centered\"] {\n    justify-content: center;"
            ),
            "root should not re-center the whole flex row, LAYOUT_CSS={LAYOUT_CSS}"
        );
        assert!(LAYOUT_CSS.contains(
            "[data-blocks-header-flyout-menu-root][data-blocks-header-flyout-menu-variant=\"centered\"] [data-scope=\"navigation-menu\"][data-part=\"root\"][data-blocks-header-flyout-menu-nav] {\n    flex: 1;\n    display: flex;\n    justify-content: center;\n  }"
        ));
    }

    /// centered 形の 2 列グリッドで `panel_footer` が両列にまたがること
    /// （codex/Bugbot 指摘の是正回帰）。
    #[test]
    fn centered_panel_footer_spans_both_grid_columns() {
        assert!(LAYOUT_CSS.contains(
            "[data-blocks-header-flyout-menu-panel][data-blocks-header-flyout-menu-columns] > [data-blocks-header-flyout-menu-panel-footer] {\n  grid-column: 1 / -1;\n}"
        ));
    }

    /// [`LAYOUT_CSS`] が 48rem 境界・ハンバーガー切り替え・disabled 中和を
    /// 持つこと（モジュール doc「レスポンシブ」節）。
    #[test]
    fn layout_css_switches_to_hamburger_on_narrow_viewports() {
        assert!(LAYOUT_CSS.contains("@media (min-width: 48rem)"));
        assert!(LAYOUT_CSS.contains(
            "[data-scope=\"button\"][data-part=\"root\"][data-blocks-header-flyout-menu-toggle] {\n    display: none;\n  }"
        ));
        assert!(LAYOUT_CSS.contains("opacity: 1;"));
    }

    /// [`LAYOUT_CSS`] が recipe の `[data-scope]`/`[data-part]` セレクタと
    /// 同等以上の詳細度でレスポンシブ切り替え・disabled 中和・アイコン
    /// 縦位置を上書きすること（Bugbot 指摘の是正回帰、[`LAYOUT_CSS`] doc
    /// 「CSS 特異性」節）。
    #[test]
    fn layout_css_overrides_have_sufficient_specificity() {
        assert!(LAYOUT_CSS.contains(
            "[data-scope=\"navigation-menu\"][data-part=\"root\"][data-blocks-header-flyout-menu-nav] {\n  display: none;\n}"
        ));
        assert!(LAYOUT_CSS.contains(
            "[data-scope=\"button\"][data-part=\"root\"][data-blocks-header-flyout-menu-toggle][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}"
        ));
        assert!(LAYOUT_CSS.contains(
            "[data-scope=\"button\"][data-part=\"root\"][data-blocks-header-flyout-menu-cta][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}"
        ));
        assert!(LAYOUT_CSS.contains(
            "[data-scope=\"navigation-menu\"][data-part=\"link\"][data-blocks-header-flyout-menu-item] {\n  align-items: flex-start;\n}"
        ));
        assert!(LAYOUT_CSS
            .contains("[data-blocks-header-flyout-menu-panel][hidden] {\n  display: none;\n}"));
    }

    /// 主 CTA（「使ってみる」）が `disabled` で描画され、フォーカス・
    /// クリック不能であること（codex P1 指摘の是正回帰）。
    #[test]
    fn cta_button_is_disabled() {
        let html = render(&demo());
        let attr_start = html
            .find("data-blocks-header-flyout-menu-cta")
            .expect("cta button should render");
        let tag_start = html[..attr_start]
            .rfind("<button")
            .expect("cta button opening tag should precede its attribute");
        let tag_end = html[tag_start..]
            .find('>')
            .expect("cta opening tag should close")
            + tag_start;
        let cta_tag = &html[tag_start..tag_end];
        assert!(cta_tag.contains("disabled"), "cta_tag={cta_tag}");
        assert!(
            cta_tag.contains(r#"aria-disabled="true""#),
            "cta_tag={cta_tag}"
        );
    }

    /// ハンバーガーボタンが `aria-label` を持つこと。
    #[test]
    fn hamburger_has_accessible_label() {
        let html = render(&demo());
        assert!(html.contains(r#"aria-label="Open main menu""#));
    }

    /// ルート class（`demo_class` とは別名）が [`demo`] の出力へ実際に
    /// 現れること（`blog_list_image` と同じ Bugbot 教訓の固定）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-header-flyout-menu-layout\""));
        assert_ne!(super::BLOCK.demo_class, "blocks-header-flyout-menu-layout");
    }
}
