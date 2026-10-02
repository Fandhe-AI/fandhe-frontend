//! `header-mega-menu` block（親トラッキング #2857、対応表 ID R0984 を主参照
//! とする合成例。前半 #2858 で骨格・全幅パネルを実装し、本イシュー #2859
//! （後半）で補助 CTA 帯・狭幅（メニュー展開時）状態の並記を仕上げる）。
//! 幅を制限したバー（ブランド / ナビ / アクション）の下に、ヘッダー全幅
//! まで広がるメガメニューパネルを持つヘッダー。Marketing / Header
//! カテゴリの 3 件目（`header_floating_pill`・`header_flyout_menu` に
//! 続く、`super`（`header/mod.rs`）参照）。
//!
//! # 使用部品
//!
//! `navigation-menu` / `button` / `icon` / `link` の 4 部品を合成する
//! （[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//!
//! # 静的表示（無 JS、唯一のドロップダウンを常時 open で固定）
//!
//! ドロップダウンを持つトップ項目は「プロダクト」1 件のみとし、
//! [`OpenState::Open`] で固定する。トリガーは押しても状態が変わらない
//! no-op になるため、[`super::header_flyout_menu`] と同じ判断で
//! `disabled: true`（ネイティブ `disabled` 属性 + `data-disabled`）
//! にしてフォーカス・クリック不能を明示する（レビュー是正: 見た目は
//! 「開閉可能なボタン」のまま実際には操作不能という食い違いを解消する。
//! `disabled_declarations()`（既定 `opacity: 0.5`）は [`LAYOUT_CSS`] で
//! 中和し、通常のトリガーと同じ見た目に保つ。open 固定の見出しとして
//! 自然に見せるための判断であり、CTA ボタンの disabled 表示（[`actions`]
//! 参照）とは異なり中和を維持する）。残りのトップ項目
//! （料金・ドキュメント）は [`navigation_menu::trigger`] を持たず
//! [`navigation_menu::item`] + [`navigation_menu::link`] のリンク項目
//! のみで構成する（閉じたままフォーカス可能だが操作しても何も起きない
//! trigger を作らないため。accordion 系 block が受けた「閉じた項目の
//! 本文へ到達できない」指摘を構造的に避ける）。この構成（[`nav`]）は
//! 幅広・狭幅の両インスタンスで共有し、`trigger`/`content` の `id` と
//! `aria-label` のみ差し替える（下記「狭幅インスタンスの並記」節）。
//!
//! # 全幅パネルの配置方法（`position: static` 上書きと包含ブロック）
//!
//! [`fandhe_frontend_pre_styled_ui::navigation_menu`] の recipe は
//! `root`/`item` に `position: relative` を、`content` に `position:
//! absolute; top: 100%; left: 0; min-width: 10rem;` を宣言する
//! （`crates/pre-styled-ui/src/navigation_menu.rs` 参照）。このままでは
//! `content` の包含ブロックが `item`（トリガー 1 個ぶんの幅）になり、
//! パネルがヘッダー全幅まで広がらない。[`LAYOUT_CSS`] は本 block の
//! スコープ内（`.blocks-header-mega-menu-layout` 子孫セレクタ、詳細度
//! (0,3,0)）に限定して `root`/`item` の `position` を `static` へ上書きし、
//! `.blocks-header-mega-menu-bar-wrap`（`position: relative`）を
//! `content` の包含ブロックへ格上げする（`.blocks-header-mega-menu-layout`
//! 自身ではなく `bar-wrap` を選ぶ理由: `layout` はバーの下にダミー本文
//! （[`page_placeholder`]）まで含むため、`layout` を包含ブロックにすると
//! `top: 100%` がダミー本文ぶんの高さを含めて計算され、パネルがバー直下
//! ではなくページ本文の下端に落ちてしまう。`bar-wrap` はバー 1 行のみを
//! 内包するため、`top: 100%` が常にバー直下を指す）。`content` 自身は
//! `inset-inline: 0; min-width: 0; padding: 0;` を追加宣言し、ヘッダー
//! 全幅へ広げる（`padding: 0` は本イシューで追加: 下記「補助 CTA 帯」節
//! 参照。`top`/`left`/`z-index` は recipe 既定のまま変更不要）。中身は
//! `.blocks-header-mega-menu-panel-inner` で `max-inline-size` + 中央寄せ
//! を与え、上のバーと同じ幅に揃える。
//!
//! # 補助 CTA 帯（[`panel_footer`]）
//!
//! パネル最下部に、パネル本体（`panel-inner`）とは別の帯として補助 CTA
//! （実在ページへのリンク 2 件）を配置する（本イシューで追加）。`content`
//! の `padding` を `0` へ上書きしたのは、この帯をパネルと同じヘッダー
//! 全幅の背景で見せるため（`panel-inner` 自身は従来どおり `padding` を
//! 持つため列のレイアウトは変わらない）。帯の中身
//! （`panel-footer-inner`）は `max-inline-size` + 中央寄せで上のバー・
//! `panel-inner` と幅を揃える。送信先を持たない no-op ボタンを増やすと
//! disabled 表示・到達性の論点が再発するため、帯のリンクは `button` では
//! なく実在ページへ遷移する [`link::root`] のみで構成する（`class` を
//! `drop_class_attr` が除去する契約のため、CSS フックは
//! `data-blocks-header-mega-menu-panel-footer-link` 属性で渡す。下記
//! 「CSS フックの選び方」節と同型の判断）。
//!
//! # 狭幅インスタンスの並記（[`mobile_preview`]）
//!
//! 無 JS の静的 Demo では開閉を伴うハンバーガーメニューを実装できない
//! （`crates/docs-site/src/blocks/marketing/contact/contact_centered_form.rs`
//! が `:has(:checked)` 等の CSS のみの状態同期を「block 全体の静的合成例
//! という設計方針に反する」として意図的に採らなかった判断を踏襲する）。
//! このため、狭幅ビューポートでの見え方を「メニュー展開時」の状態として
//! 常時表示する第 2 のインスタンスを並記する（本イシューで追加。
//! `@media` によるビューポート幅連動の `display` 切り替えは一切行わない
//! ため、`hidden` 属性も持たない）。メニューボタン（[`mobile_bar`]）は
//! 押しても状態が変わらない no-op のため、プロダクトトリガーと同じ判断で
//! `disabled: true` + `aria-expanded="true"` + `aria-controls`（展開済み
//! パネルの `id`）にして「既に展開済みで操作不能」であることを明示する。
//! 命名に `hamburger` の語は使わない（`class`/`data-*` は `menu-toggle`/
//! `mobile` を使う。[`super::header_flyout_menu`] の同型実演とは異なる
//! 独立した判断であり、下記「レスポンシブ」節の既存テスト
//! `narrow_layout_keeps_nav_and_actions_reachable` が `LAYOUT_CSS` に
//! `hamburger` を含まないことを固定している）。狭幅パネルの `nav`
//! ランドマークには幅広側と異なる `aria-label`（「メインメニュー
//! （狭幅）」）を付け、`navigation_menu::root` の `orientation` を
//! [`Orientation::Vertical`] にして `data-orientation="vertical"` を
//! 出力する（実際の縦並びは [`LAYOUT_CSS`] の `flex-direction: column`
//! 上書きが担う。`data-orientation` は SSR 静的属性のみで、視覚は担わない
//! `crates/headless-ui/src/navigation_menu.rs` の仕様どおり）。狭幅パネル
//! の `content` も `position: static` に上書きし通常のフローへ置く
//! （`.blocks-header-mega-menu-mobile` スコープ限定）。
//!
//! # `.blocks-demo` のはみ出し対策
//!
//! `crate::blocks::stylesheet` の `.blocks-demo` は `overflow-x: auto` を
//! 持つ（`blocks_stylesheet_declares_demo_frame_overflow` 契約）。絶対配置
//! パネルがフローに寄与しないため、`.blocks-header-mega-menu-page`
//! （ダミー本文枠）へ `min-block-size` を持たせ、展開済みパネルが
//! レイアウトボックスの内側に収まるようにする。補助 CTA 帯の追加でパネル
//! が縦に伸びる分、既定値・狭幅時の値をいずれも本イシューで引き上げる
//! （下記「レスポンシブ」節参照）。
//!
//! # レスポンシブ（`@media (max-width: 47.99rem)`）
//!
//! [`super::super::footer::footer_newsletter`] と同じブレークポイントを
//! 使う。幅広インスタンス（[`bar`]）は狭い幅でもナビ・アクション
//! （`.blocks-header-mega-menu-nav`/`.blocks-header-mega-menu-actions`）を
//! 非表示にせず、`flex-wrap: wrap` でバー内へ折り返して常時到達可能な
//! まま残す（レビュー是正: ハンバーガーに開閉処理を持たせず内容を隠すと、
//! 無 JS では畳んだ内容へ到達する手段が一切なくなる。accordion 系 block
//! が受けた「閉じた項目の本文へ到達できない」指摘と同型の問題であり、
//! ハンバーガーボタン自体を持たない構成でこれを避ける）。トップ項目
//! 一覧（`navigation_menu::list`、既定 `display: flex` で折り返さない）
//! 自体にも同じ幅で `flex-wrap: wrap` を上書きする（PR #3273 レビュー
//! 指摘（P2）是正: バー全体は折り返せても `list` 単体が横スクロールを
//! 要求していた不整合を解消する）。併せて、パネルが列見出しの下でスタック
//! （[`fandhe_frontend_pre_styled_ui::navigation_menu`] の `panel-inner`
//! グリッドが `auto-fit` で 1 列化する）して縦に伸びる分、
//! `.blocks-header-mega-menu-page` の `min-block-size` をこの幅でのみ
//! 引き上げる（PR #3273 Bugbot 指摘是正: 28rem のままだと常時展開パネル
//! がこの枠を超え、`.blocks-demo` の `overflow-x: auto`（CSS 仕様上
//! `overflow-y` も暗黙に `auto` へ計算される）でクリップ/内部スクロール
//! 化していた。本イシューで補助 CTA 帯ぶん値をさらに引き上げる）。上記の
//! ビューポート幅連動 reflow は幅広インスタンス（[`layout`]）専用であり、
//! 狭幅インスタンス（[`mobile_preview`]）はビューポート幅に関係なく常に
//! 「狭幅（メニュー展開時）」の見た目のまま並記する別の枠（上記
//! 「狭幅インスタンスの並記」節参照）。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `navigation_menu` のパート関数（`root`/`list`/`item`/`trigger`/
//! `item_indicator`/`content`/`link`）は呼び出し側 `attrs` の `class` を
//! 除去しない（`crates/headless-ui/src/navigation_menu.rs` の
//! `drop_reserved` は `data-*`/`aria-*`/`type`/`disabled` 系の予約キーのみを
//! 対象とし `class` を含まない）ため、`.blocks-header-mega-menu-nav` の
//! ような class フックがそのまま使える。一方 `button::button`/
//! `button::icon_button`/`link::root`/`icon::icon` は `drop_class_attr` で
//! 呼び出し側 `class` を常に除去する契約のため、これらへのフックは
//! `data-blocks-header-mega-menu-*` 属性で渡す
//! （[`super::super::faq::faq_accordion_centered`] と同型の判断）。
//!
//! # href の方針
//!
//! `href="#"` は使わない（横断テストが禁止する）。パネル項目・
//! ドキュメント・補助 CTA 帯はサイト内に実在する索引ページへの相対パス
//! （`../../`・`../../guides/`・`../../themes/`・`../../primitives/`・
//! `../../api/`・`../../examples/`）を使う（[`super::super::faq::faq_question_rows`]
//! と同型の判断）。「料金」は本サイトに料金情報の索引ページが無いため、
//! `site/nav.toml` に登録済みの料金関連 block（`../../blocks/
//! pricing-comparison-table/`）へ遷移させる（PR #3273 レビュー指摘（P2）
//! 是正: 当初 `../../themes/`（Themes 索引）を指していたが、表示名「料金」
//! と行き先が食い違い、料金情報に到達できなかった）。本サイトに実在する
//! ログインページは無いため、GitHub リンクのみ実在の外部 URL（[`REPO`]）
//! を使い、ラベルも行き先どおり「GitHub」とする（[`actions`] doc コメント
//! 参照）。
//!
//! # id 接頭辞
//!
//! `id`/`aria-controls`/`aria-labelledby` は開いたドロップダウン
//! （プロダクト、幅広・狭幅の各インスタンスに 1 件ずつ）と、狭幅の
//! 展開済みパネルにのみ必要であり、`blocks-header-mega-menu-products-
//! {trigger|content}`（幅広）・`blocks-header-mega-menu-mobile-products-
//! {trigger|content}`（狭幅）・`blocks-header-mega-menu-mobile-panel`
//! （狭幅パネル自体）の固定文字列で一意にする（複数項目に添字展開する
//! accordion 系 block とは異なり、`format!` を使わない。`crate::blocks`
//! モジュール doc「HTML 文字列の直接組み立て禁止」節参照）。
//!
//! # `text` の名前衝突
//!
//! `fandhe_frontend_pre_styled_ui::text::text`（`<p>` を組み立てる styled
//! パート関数）は本 block では使わないため衝突しない
//! （`fandhe_frontend_core::text` のみを import する）。
//!
//! # `<form>` を持たない・データ取得/送信を行わない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない。ボタンは既定の `type="button"`（`navigation_menu::trigger`
//! も `type="button"` 固定）のまま送信先を持たない。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, el, span, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::icon::{self, IconProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::navigation_menu::{self, NavigationMenuProps, OpenState};
use fandhe_frontend_pre_styled_ui::{Orientation, Size};

/// パネル項目 1 件（タイトル, 説明, href, アイコンの `path` `d`）。href は
/// サイト内に実在する索引ページへの相対パス（モジュール冒頭 rustdoc「href
/// の方針」節）。アイコンは PR #3273 レビュー指摘（P2）是正: モジュール
/// doc・本定数のコメントが言う「アイコン付き項目」を実際に描画する
/// （[`item_icon`] 参照。`error_page_popular_links::stroke_icon` と同型の
/// 装飾用線画）。
type PanelItem = (&'static str, &'static str, &'static str, &'static str);

/// パネルの列 1 件（列見出し, 項目 3 件）。
type PanelColumn = (&'static str, [PanelItem; 3]);

/// プロダクトパネルの列一覧（列見出し + アイコン付き項目 3 件 × 2 列）。
const PANEL_COLUMNS: [PanelColumn; 2] = [
    (
        "分析",
        [
            (
                "ダッシュボード",
                "利用状況をひと目で把握できる可視化パネル。",
                "../../themes/",
                "M4 4h16v12H4zM8 20h8M12 16v4",
            ),
            (
                "レポート",
                "定期集計を自動で生成するレポート機能。",
                "../../guides/",
                "M6 3h9l3 3v15H6zM8 10h8M8 14h8M8 18h5",
            ),
            (
                "アラート",
                "しきい値超過を通知する監視機能。",
                "../../primitives/",
                "M12 3a6 6 0 0 0-6 6c0 5-2 6-2 6h16s-2-1-2-6a6 6 0 0 0-6-6zM10 19a2 2 0 0 0 4 0",
            ),
        ],
    ),
    (
        "連携",
        [
            (
                "API",
                "外部システムと連携するための拡張ポイント。",
                "../../api/",
                "M8 6 3 12l5 6M16 6l5 6-5 6",
            ),
            (
                "サンプル集",
                "構成別の実装サンプルへの索引。",
                "../../examples/",
                "M4 4h7v7H4zM13 4h7v7h-7zM4 13h7v7H4zM13 13h7v7h-7z",
            ),
            (
                "導入ガイド",
                "はじめての導入手順をまとめたガイド。",
                "../../guides/",
                "M12 3v18M4 8l8-5 8 5M4 16l8 5 8-5",
            ),
        ],
    ),
];

/// 幅広インスタンスで唯一開いた状態で固定するトリガーの `id`（モジュール
/// 冒頭 rustdoc「id 接頭辞」節。項目が 1 件のみのため `format!` による
/// 添字展開は行わない）。
const PRODUCTS_TRIGGER_ID: &str = "blocks-header-mega-menu-products-trigger";
/// [`PRODUCTS_TRIGGER_ID`] と対になる `content` の `id`。
const PRODUCTS_CONTENT_ID: &str = "blocks-header-mega-menu-products-content";

/// 狭幅インスタンス（[`mobile_preview`]）側のプロダクトトリガー `id`
/// （モジュール冒頭 rustdoc「狭幅インスタンスの並記」節）。
const MOBILE_PRODUCTS_TRIGGER_ID: &str = "blocks-header-mega-menu-mobile-products-trigger";
/// [`MOBILE_PRODUCTS_TRIGGER_ID`] と対になる `content` の `id`。
const MOBILE_PRODUCTS_CONTENT_ID: &str = "blocks-header-mega-menu-mobile-products-content";
/// 狭幅インスタンスの展開済みパネル自体の `id`（メニュートグルボタンの
/// `aria-controls` が参照する）。
const MOBILE_PANEL_ID: &str = "blocks-header-mega-menu-mobile-panel";

/// リポジトリ実 URL（`href` の方針）。本サイトに実在するログインページは
/// 無いため、遷移先はこの実在の外部 URL を使う。ただし [`actions`] の
/// リンク文言は「ログイン」ではなく行き先どおり「GitHub」とする（PR #3273
/// レビュー指摘: 「ログイン」という文言のまま GitHub リポジトリへ飛ばすと
/// リンク名と実際の行き先が食い違い、ログイン画面に到達すると誤認させる。
/// [`super::header_flyout_menu`] の同型リンクは同じ不一致を抱えたまま
/// 既に main へマージ済みで、本 block の範囲外のため別途追跡する
/// （`.claude/rules/out-of-scope-tracking.md`）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// ブランドロゴ（装飾用の幾何アイコン、菱形）。実在ブランドのロゴ・
/// 商標を模さない独自の単純図形（`docs/design/wireframe-ui-architecture.md`
/// と同じ判断軸）。
fn brand_icon() -> Node {
    icon::icon(
        &IconProps {
            label: None,
            ..IconProps::default()
        },
        vec![],
        vec![el("path", vec![("d", "M12 2L22 12L12 22L2 12Z")], vec![])],
    )
}

/// ブランド領域（アイコン + 架空のブランド名）。幅広バー（[`bar`]）・
/// 狭幅バー（[`mobile_bar`]）の双方から呼ばれる共通部品。
fn brand() -> Node {
    div(
        vec![("class", "blocks-header-mega-menu-brand")],
        vec![brand_icon(), span(vec![], vec![text("Nimbus Studio")])],
    )
}

/// パネル項目・補助 CTA 帯・メニュートグルボタンで共有する線画アイコン
/// （装飾用途。[`error_page_popular_links`] の `stroke_icon` と同型の
/// パターンで、`icon::icon` の `currentColor` 継承に任せ生の色リテラルは
/// 持ち込まない）。
///
/// [`error_page_popular_links`]: crate::blocks::marketing::error_page::error_page_popular_links
fn item_icon(path_d: &str) -> Node {
    icon::icon(
        &IconProps {
            size: Size::Sm,
            label: None,
            ..IconProps::default()
        },
        vec![],
        vec![el(
            "path",
            vec![
                ("d", path_d),
                ("fill", "none"),
                ("stroke", "currentColor"),
                ("stroke-width", "2"),
                ("stroke-linecap", "round"),
                ("stroke-linejoin", "round"),
            ],
            vec![],
        )],
    )
}

/// パネル 1 列分（列見出し + アイコン付き項目 3 件）。
fn panel_column(heading: &str, items: &[PanelItem; 3]) -> Node {
    let links: Vec<Node> = items
        .iter()
        .map(|(title, description, href, icon_path_d)| {
            navigation_menu::link(
                href,
                false,
                vec![("class", "blocks-header-mega-menu-panel-link")],
                vec![
                    div(
                        vec![("class", "blocks-header-mega-menu-panel-link-header")],
                        vec![
                            item_icon(icon_path_d),
                            span(
                                vec![("class", "blocks-header-mega-menu-panel-link-title")],
                                vec![text(*title)],
                            ),
                        ],
                    ),
                    span(
                        vec![("class", "blocks-header-mega-menu-panel-link-description")],
                        vec![text(*description)],
                    ),
                ],
            )
        })
        .collect();
    let mut children = vec![span(
        vec![("class", "blocks-header-mega-menu-panel-column-heading")],
        vec![text(heading)],
    )];
    children.extend(links);
    div(
        vec![("class", "blocks-header-mega-menu-panel-column")],
        children,
    )
}

/// 補助 CTA 帯のリンク 1 本（モジュール冒頭 rustdoc「補助 CTA 帯」節）。
/// `link::root` は `drop_class_attr` で `class` を除去するため、CSS フックは
/// `data-blocks-header-mega-menu-panel-footer-link` 属性で渡す。
fn footer_link(href: &str, label: &str, icon_path_d: &str) -> Node {
    link::root(
        href,
        &LinkProps::default(),
        vec![("data-blocks-header-mega-menu-panel-footer-link", "")],
        vec![item_icon(icon_path_d), text(label)],
    )
}

/// パネル下部の補助 CTA 帯（モジュール冒頭 rustdoc「補助 CTA 帯」節）。
/// 実在ページへのリンク 2 件のみで構成し、送信先を持たない `button` は
/// 使わない。幅広（[`products_item`] の幅広インスタンス）・狭幅
/// （狭幅インスタンス）の両パネルへ同一の内容を配置する。
fn panel_footer() -> Node {
    div(
        vec![("class", "blocks-header-mega-menu-panel-footer")],
        vec![div(
            vec![("class", "blocks-header-mega-menu-panel-footer-inner")],
            vec![
                span(
                    vec![("class", "blocks-header-mega-menu-panel-footer-text")],
                    vec![text(
                        "導入を検討中ですか。まずは無料プランから始められます。",
                    )],
                ),
                div(
                    vec![("class", "blocks-header-mega-menu-panel-footer-links")],
                    vec![
                        footer_link(
                            "../../guides/",
                            "導入ガイドを見る",
                            "M12 3v18M4 8l8-5 8 5M4 16l8 5 8-5",
                        ),
                        footer_link(REPO, "GitHub で見る", "M8 6 3 12l5 6M16 6l5 6-5 6"),
                    ],
                ),
            ],
        )],
    )
}

/// 「プロダクト」トップ項目（唯一のドロップダウン、常時 open 固定）。
/// `trigger_id`/`content_id` を引数化し、幅広・狭幅の各インスタンスから
/// 異なる `id` の組で呼び出す（モジュール冒頭 rustdoc「id 接頭辞」節）。
fn products_item(props: &NavigationMenuProps, trigger_id: &str, content_id: &str) -> Node {
    let state = OpenState::Open;
    let columns: Vec<Node> = PANEL_COLUMNS
        .iter()
        .map(|(heading, items)| panel_column(heading, items))
        .collect();

    navigation_menu::item(
        state,
        false,
        props,
        "products",
        vec![],
        vec![
            navigation_menu::trigger(
                state,
                true,
                "products",
                Some(trigger_id),
                Some(content_id),
                vec![],
                vec![
                    text("プロダクト"),
                    navigation_menu::item_indicator(
                        state,
                        props,
                        "products",
                        vec![],
                        vec![text("▾")],
                    ),
                ],
            ),
            navigation_menu::content(
                state,
                props,
                "products",
                Some(content_id),
                Some(trigger_id),
                vec![],
                vec![
                    div(
                        vec![("class", "blocks-header-mega-menu-panel-inner")],
                        columns,
                    ),
                    panel_footer(),
                ],
            ),
        ],
    )
}

/// トリガーを持たない、リンクのみのトップ項目（料金・ドキュメント）。
fn link_item(props: &NavigationMenuProps, value: &str, label: &str, href: &str) -> Node {
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

/// バー内のナビゲーション（メガメニュー本体）。幅広（[`bar`]）・狭幅
/// （[`mobile_panel`]）の両インスタンスから、`class`/`aria-label`/
/// トリガー・content の `id` の組を変えて呼び出す共通実装
/// （モジュール冒頭 rustdoc「静的表示」節）。
fn nav(
    props: &NavigationMenuProps,
    class_name: &str,
    aria_label: &str,
    trigger_id: &str,
    content_id: &str,
) -> Node {
    navigation_menu::root(
        props,
        aria_label,
        vec![("class", class_name)],
        vec![navigation_menu::list(
            props,
            vec![],
            vec![
                products_item(props, trigger_id, content_id),
                link_item(
                    props,
                    "pricing",
                    "料金",
                    "../../blocks/pricing-comparison-table/",
                ),
                link_item(props, "docs", "ドキュメント", "../../guides/"),
            ],
        )],
    )
}

/// バー右側のアクション（GitHub リンク + CTA ボタン）。幅広バー
/// （[`bar`]）・狭幅パネル（[`mobile_panel`]）の双方から呼ばれる共通部品。
/// PR #3273 レビュー指摘（P2）是正: 当初「ログイン」ラベルで [`REPO`]
/// （GitHub リポジトリ）へ遷移させていたが、本サイトに実在するログイン
/// ページは無く、リンク名（ログイン）と実際の行き先（GitHub）が食い違って
/// いた。行き先を変えずラベルを実態（GitHub リポジトリ）に合わせて是正
/// する（[`REPO`] の doc コメント参照）。CTA（「無料で始める」）は遷移先・
/// 送信処理を持たない no-op のため、`disabled: true` にしてフォーカス・
/// クリック不能を明示する（`disabled_declarations()`〔既定 `opacity:
/// 0.5`〕は中和せずそのまま適用し、操作できない CTA だと見た目でも分かる
/// よう無効表示のまま残す。レビュー指摘是正: 中和すると押せる見た目の
/// まま実際には押せない食い違いが残っていた）。
fn actions() -> Node {
    div(
        vec![("class", "blocks-header-mega-menu-actions")],
        vec![
            link::root(REPO, &LinkProps::default(), vec![], vec![text("GitHub")]),
            button::button(
                &ButtonProps {
                    disabled: true,
                    ..ButtonProps::default()
                },
                vec![("data-blocks-header-mega-menu-cta", "")],
                vec![text("無料で始める")],
            ),
        ],
    )
}

/// 幅を制限したバー（ブランド / ナビ / アクション）。狭い幅では
/// ハンバーガーで畳まず、[`LAYOUT_CSS`] の `flex-wrap` でバー内へ折り返す
/// （モジュール冒頭 rustdoc「レスポンシブ」節）。
fn bar() -> Node {
    div(
        vec![("class", "blocks-header-mega-menu-bar")],
        vec![
            brand(),
            nav(
                &NavigationMenuProps::default(),
                "blocks-header-mega-menu-nav",
                "メインメニュー",
                PRODUCTS_TRIGGER_ID,
                PRODUCTS_CONTENT_ID,
            ),
            actions(),
        ],
    )
}

/// ダミーのページ本文（`.blocks-demo` のはみ出し対策、モジュール冒頭
/// rustdoc 参照）。
fn page_placeholder() -> Node {
    div(
        vec![("class", "blocks-header-mega-menu-page")],
        vec![text(
            "ページ本文（ダミー）。展開済みパネルの下に十分な高さを確保するための枠。",
        )],
    )
}

/// 幅広インスタンス（バー + ダミー本文）。旧 `demo()` の出力そのもの
/// （本イシューで [`demo`] が幅広・狭幅の 2 状態を並記する構成へ変わった
/// ため、幅広分をこの関数へ切り出した）。
fn layout() -> Node {
    div(
        vec![("class", "blocks-header-mega-menu-layout")],
        vec![
            div(
                vec![("class", "blocks-header-mega-menu-bar-wrap")],
                vec![bar()],
            ),
            page_placeholder(),
        ],
    )
}

/// 状態並記の見出し（モジュール冒頭 rustdoc「狭幅インスタンスの並記」
/// 節）。
fn state_label(label: &str) -> Node {
    span(
        vec![("class", "blocks-header-mega-menu-state-label")],
        vec![text(label)],
    )
}

/// 狭幅インスタンスのバー（ブランド + メニュートグルボタン）。ボタンは
/// 押しても状態が変わらない no-op のため `disabled: true` にし、既に
/// 展開済みであることを `aria-expanded="true"` + `aria-controls` で示す
/// （モジュール冒頭 rustdoc「狭幅インスタンスの並記」節）。
fn mobile_bar() -> Node {
    div(
        vec![("class", "blocks-header-mega-menu-mobile-bar")],
        vec![
            brand(),
            button::icon_button(
                &ButtonProps {
                    variant: ButtonVariant::Ghost,
                    disabled: true,
                    ..ButtonProps::default()
                },
                "メニュー",
                vec![
                    ("aria-expanded", "true"),
                    ("aria-controls", MOBILE_PANEL_ID),
                    ("data-blocks-header-mega-menu-menu-toggle", ""),
                ],
                vec![item_icon("M3 6h18M3 12h18M3 18h18")],
            ),
        ],
    )
}

/// 狭幅インスタンスの展開済みパネル（[`nav`] の縦並び構成 + [`actions`]）。
/// `hidden` を持たず常時表示する（モジュール冒頭 rustdoc「狭幅
/// インスタンスの並記」節）。
fn mobile_panel() -> Node {
    let props = NavigationMenuProps {
        orientation: Orientation::Vertical,
    };
    div(
        vec![
            ("id", MOBILE_PANEL_ID),
            ("class", "blocks-header-mega-menu-mobile-panel"),
        ],
        vec![
            nav(
                &props,
                "blocks-header-mega-menu-mobile-nav",
                "メインメニュー（狭幅）",
                MOBILE_PRODUCTS_TRIGGER_ID,
                MOBILE_PRODUCTS_CONTENT_ID,
            ),
            actions(),
        ],
    )
}

/// 狭幅インスタンス全体（バー + 展開済みパネル）。[`demo`] が幅広
/// インスタンス（[`layout`]）と並べて描画する。
fn mobile_preview() -> Node {
    div(
        vec![("class", "blocks-header-mega-menu-mobile")],
        vec![mobile_bar(), mobile_panel()],
    )
}

/// `header-mega-menu` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数（モジュール doc「静的表示」節）。幅広インスタンス
/// （[`layout`]）と狭幅（メニュー展開時）インスタンス（[`mobile_preview`]）
/// を見出し付きで並記する（本イシューで追加、モジュール冒頭 rustdoc
/// 「狭幅インスタンスの並記」節）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-header-mega-menu-states")],
        vec![
            state_label("幅広（プロダクトを展開）"),
            layout(),
            state_label("狭幅（メニュー展開時）"),
            mobile_preview(),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/header-mega-menu/",
    title: "header-mega-menu",
    category: BlockCategory::Header,
    rust_source: "crates/docs-site/src/blocks/marketing/header/header_mega_menu.rs",
    demo_class: "blocks-header-mega-menu",
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

/// `header_mega_menu` 固有のレイアウト規則（`crate::blocks`
/// モジュール doc「CSS の置き場」節。他 block と同型で `pub(super)` ではなく
/// 本ファイル内 private 定数として `super::stylesheet` 経由の `push_css`
/// で連結される）。
///
/// セレクタは `.blocks-header-mega-menu-*`、
/// `[data-blocks-header-mega-menu-*]`、および `.blocks-header-mega-menu-
/// layout`/`.blocks-header-mega-menu-mobile` を祖先に持つ
/// `[data-scope="navigation-menu"]` 系セレクタへの子孫結合子付き上書き
/// （全幅パネル化・狭幅の縦並び化、モジュール冒頭 rustdoc「全幅パネル
/// の配置方法」「狭幅インスタンスの並記」節）のみを用い、他 block や
/// 部品の素のセレクタへ影響させない。値はすべて `var(--fandhe-*)`
/// トークンで書き、生の色リテラルは使わない。
const LAYOUT_CSS: &str = "\
.blocks-header-mega-menu-states {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-header-mega-menu-state-label {\n  display: block;\n  font-size: var(--fandhe-font-font-size-sm);\n  font-weight: 600;\n  color: var(--fandhe-color-fg-muted);\n}\n\
.blocks-header-mega-menu-bar-wrap {\n  position: relative;\n  background: var(--fandhe-color-bg);\n  border-bottom: 1px solid var(--fandhe-color-border);\n}\n\
.blocks-header-mega-menu-bar {\n  display: flex;\n  align-items: center;\n  justify-content: space-between;\n  gap: var(--fandhe-space-4);\n  max-inline-size: 64rem;\n  margin-inline: auto;\n  padding: var(--fandhe-space-3) var(--fandhe-space-4);\n}\n\
.blocks-header-mega-menu-brand {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n  font-weight: 600;\n  white-space: nowrap;\n}\n\
.blocks-header-mega-menu-layout [data-scope=\"navigation-menu\"][data-part=\"root\"] {\n  flex: 1;\n}\n\
.blocks-header-mega-menu-layout [data-scope=\"navigation-menu\"][data-part=\"root\"],\n\
.blocks-header-mega-menu-layout [data-scope=\"navigation-menu\"][data-part=\"item\"] {\n  position: static;\n}\n\
.blocks-header-mega-menu-layout [data-scope=\"navigation-menu\"][data-part=\"content\"] {\n  inset-inline: 0;\n  min-width: 0;\n  padding: 0;\n}\n\
.blocks-header-mega-menu-panel-inner {\n  max-inline-size: 64rem;\n  margin-inline: auto;\n  display: grid;\n  grid-template-columns: repeat(auto-fit, minmax(min(14rem, 100%), 1fr));\n  gap: var(--fandhe-space-6);\n  padding: var(--fandhe-space-2) var(--fandhe-space-4);\n}\n\
.blocks-header-mega-menu-panel-column {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1);\n}\n\
.blocks-header-mega-menu-panel-column-heading {\n  display: block;\n  font-weight: 600;\n  font-size: var(--fandhe-font-font-size-sm);\n  color: var(--fandhe-color-fg-muted);\n  margin-block-end: var(--fandhe-space-2);\n}\n\
.blocks-header-mega-menu-panel-link[data-scope=\"navigation-menu\"][data-part=\"link\"] {\n  display: flex;\n  flex-direction: column;\n  align-items: flex-start;\n  gap: var(--fandhe-space-1);\n}\n\
.blocks-header-mega-menu-panel-link-header {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-header-mega-menu-panel-link-title {\n  font-weight: 600;\n}\n\
.blocks-header-mega-menu-panel-link-description {\n  font-size: var(--fandhe-font-font-size-sm);\n  color: var(--fandhe-color-fg-muted);\n}\n\
.blocks-header-mega-menu-panel-footer {\n  border-block-start: 1px solid var(--fandhe-color-border);\n  background: var(--fandhe-color-bg-muted);\n}\n\
.blocks-header-mega-menu-panel-footer-inner {\n  max-inline-size: 64rem;\n  margin-inline: auto;\n  display: flex;\n  align-items: center;\n  justify-content: space-between;\n  flex-wrap: wrap;\n  gap: var(--fandhe-space-3);\n  padding: var(--fandhe-space-3) var(--fandhe-space-4);\n}\n\
.blocks-header-mega-menu-panel-footer-text {\n  font-size: var(--fandhe-font-font-size-sm);\n  color: var(--fandhe-color-fg-muted);\n}\n\
.blocks-header-mega-menu-panel-footer-links {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-4);\n}\n\
[data-blocks-header-mega-menu-panel-footer-link] {\n  display: inline-flex;\n  align-items: center;\n  gap: var(--fandhe-space-1);\n  color: var(--fandhe-color-fg);\n  font-size: var(--fandhe-font-font-size-sm);\n  font-weight: 600;\n}\n\
.blocks-header-mega-menu-actions {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-3);\n  white-space: nowrap;\n}\n\
.blocks-header-mega-menu-layout [data-scope=\"navigation-menu\"][data-part=\"trigger\"][data-disabled],\n\
.blocks-header-mega-menu-mobile [data-scope=\"navigation-menu\"][data-part=\"trigger\"][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
.blocks-header-mega-menu-page {\n  min-block-size: 34rem;\n  padding: var(--fandhe-space-6) var(--fandhe-space-4);\n  color: var(--fandhe-color-fg-muted);\n}\n\
.blocks-header-mega-menu-mobile {\n  max-inline-size: 24rem;\n  border: 1px solid var(--fandhe-color-border);\n  border-radius: var(--fandhe-radius-md);\n  background: var(--fandhe-color-bg);\n  overflow: hidden;\n}\n\
.blocks-header-mega-menu-mobile-bar {\n  display: flex;\n  align-items: center;\n  justify-content: space-between;\n  gap: var(--fandhe-space-3);\n  padding: var(--fandhe-space-3) var(--fandhe-space-4);\n  border-bottom: 1px solid var(--fandhe-color-border);\n}\n\
.blocks-header-mega-menu-mobile-panel {\n  padding-block-end: var(--fandhe-space-2);\n}\n\
.blocks-header-mega-menu-mobile [data-scope=\"navigation-menu\"][data-part=\"item\"] {\n  position: static;\n}\n\
.blocks-header-mega-menu-mobile [data-scope=\"navigation-menu\"][data-part=\"list\"] {\n  display: flex;\n  flex-direction: column;\n  align-items: stretch;\n  gap: var(--fandhe-space-1);\n  padding: var(--fandhe-space-2) var(--fandhe-space-4);\n}\n\
.blocks-header-mega-menu-mobile [data-scope=\"navigation-menu\"][data-part=\"content\"] {\n  position: static;\n  padding: 0;\n  margin-top: var(--fandhe-space-2);\n  border: none;\n  box-shadow: none;\n}\n\
.blocks-header-mega-menu-mobile .blocks-header-mega-menu-actions {\n  flex-wrap: wrap;\n  padding: var(--fandhe-space-2) var(--fandhe-space-4);\n}\n\
@media (max-width: 47.99rem) {\n  .blocks-header-mega-menu-bar {\n    flex-wrap: wrap;\n  }\n  .blocks-header-mega-menu-nav[data-scope=\"navigation-menu\"][data-part=\"root\"] {\n    flex-basis: 100%;\n  }\n  .blocks-header-mega-menu-layout [data-scope=\"navigation-menu\"][data-part=\"list\"] {\n    flex-wrap: wrap;\n  }\n  .blocks-header-mega-menu-actions {\n    flex-basis: 100%;\n  }\n  .blocks-header-mega-menu-page {\n    min-block-size: 50rem;\n  }\n}\n";

#[cfg(test)]
mod tests {
    use super::{
        demo, LAYOUT_CSS, MOBILE_PANEL_ID, MOBILE_PRODUCTS_CONTENT_ID, MOBILE_PRODUCTS_TRIGGER_ID,
        PRODUCTS_CONTENT_ID, PRODUCTS_TRIGGER_ID, REPO,
    };
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

    /// 幅広・狭幅の各インスタンスで唯一のドロップダウン（プロダクト）が
    /// 常時 open で固定され、`hidden` を持たないこと（モジュール doc
    /// 「静的表示」節）。閉じた trigger（`aria-expanded="false"` の
    /// navigation-menu trigger）は存在しないこと（リンクのみの項目は
    /// trigger を持たないため）。本イシューで狭幅インスタンスを追加した
    /// ため、件数は幅広分の 2 倍（open item・content・trigger 各 2 件）に
    /// なる。
    #[test]
    fn demo_renders_open_dropdowns_in_both_states() {
        let html = render(&demo());
        assert_eq!(
            html.matches(r#"data-part="item" data-state="open""#)
                .count(),
            2,
            "html={html}"
        );
        assert_eq!(html.matches("data-part=\"content\"").count(), 2);
        assert!(!html.contains(r#"data-part="content" data-state="closed""#));
        assert_eq!(
            html.matches(r#"data-part="trigger""#).count(),
            2,
            "リンクのみの項目は trigger を持たない: html={html}"
        );
        assert!(!html.contains(r#"data-part="trigger" aria-expanded="false""#));
        for (trigger_id, content_id) in [
            (PRODUCTS_TRIGGER_ID, PRODUCTS_CONTENT_ID),
            (MOBILE_PRODUCTS_TRIGGER_ID, MOBILE_PRODUCTS_CONTENT_ID),
        ] {
            assert!(html.contains(&format!("id=\"{trigger_id}\"")));
            assert!(html.contains(&format!("aria-controls=\"{content_id}\"")));
            assert!(html.contains(&format!("id=\"{content_id}\"")));
            assert!(html.contains(&format!("aria-labelledby=\"{trigger_id}\"")));
        }
    }

    /// 指定した `trigger_id` を持つトリガーが `disabled` で描画され、見た目は
    /// 開閉可能なボタンのまま実際は操作不能という食い違いが無いことを検査
    /// する共通ヘルパ（[`products_triggers_are_disabled`] から幅広・狭幅
    /// 双方へ適用する）。
    fn assert_trigger_disabled(html: &str, trigger_id: &str) {
        let trigger_start = html
            .find(&format!("id=\"{trigger_id}\""))
            .unwrap_or_else(|| panic!("trigger id {trigger_id} should be present"));
        let trigger_tag = &html[..trigger_start];
        let tag_start = trigger_tag
            .rfind("<button")
            .expect("trigger should be a <button>");
        let trigger_tag = &html[tag_start..html[tag_start..].find('>').unwrap() + tag_start];
        assert!(
            trigger_tag.contains("disabled=\"\""),
            "trigger_tag={trigger_tag}"
        );
        assert!(
            trigger_tag.contains(r#"data-disabled="""#),
            "trigger_tag={trigger_tag}"
        );
        assert!(
            trigger_tag.contains(r#"aria-expanded="true""#),
            "trigger_tag={trigger_tag}"
        );
    }

    /// 幅広・狭幅双方のプロダクトトリガーが `disabled` で描画されること
    /// （P1 是正の継承、モジュール doc「静的表示」節）。
    #[test]
    fn products_triggers_are_disabled() {
        let html = render(&demo());
        assert_trigger_disabled(&html, PRODUCTS_TRIGGER_ID);
        assert_trigger_disabled(&html, MOBILE_PRODUCTS_TRIGGER_ID);
    }

    /// 「GitHub」リンクの遷移先が実在の外部 URL（[`REPO`]）であり、ラベルが
    /// 行き先どおり「GitHub」であること（P2 是正: 「ログイン」ラベルの
    /// まま GitHub リポジトリへ飛ばすとリンク名と行き先が食い違う、
    /// `actions` doc コメント参照）。
    #[test]
    fn github_link_label_matches_its_repo_destination() {
        let html = render(&demo());
        assert!(html.contains(&format!(r#"href="{REPO}""#)));
        assert!(html.contains(">GitHub<"));
        assert!(!html.contains(">ログイン<"));
    }

    /// CTA（「無料で始める」）が幅広・狭幅双方で `disabled` で描画され、
    /// フォーカス・クリック不能であること（P2 是正の継承: 遷移先・送信
    /// 処理を持たない no-op ボタンが操作可能に見える食い違いを解消する）。
    #[test]
    fn cta_buttons_are_disabled() {
        let html = render(&demo());
        let marker = "data-blocks-header-mega-menu-cta";
        let mut search_from = 0;
        let mut found = 0;
        while let Some(rel_idx) = html[search_from..].find(marker) {
            let cta_start = search_from + rel_idx;
            let cta_tag_start = html[..cta_start].rfind("<button").unwrap();
            let cta_tag_end = html[cta_tag_start..].find('>').unwrap() + cta_tag_start;
            let cta_tag = &html[cta_tag_start..cta_tag_end];
            assert!(cta_tag.contains("disabled"), "cta_tag={cta_tag}");
            assert!(
                cta_tag.contains(r#"aria-disabled="true""#),
                "cta_tag={cta_tag}"
            );
            found += 1;
            search_from = cta_start + marker.len();
        }
        assert_eq!(
            found, 2,
            "CTA marker should appear once per instance (wide + narrow)"
        );
    }

    /// CTA の disabled 表示を [`LAYOUT_CSS`] で中和しないこと（レビュー
    /// 指摘是正の継承: 中和すると `disabled_declarations()` 既定の
    /// `opacity: 0.5` が打ち消され、操作できない CTA が押せる見た目の
    /// まま残っていた。トリガー側の中和〔`assert_trigger_disabled`
    /// と対になる `[data-part="trigger"][data-disabled]` 上書き〕は
    /// open 固定の見出しとして自然に見せるための別判断であり残す）。
    #[test]
    fn cta_disabled_style_is_not_neutralized() {
        assert!(!LAYOUT_CSS.contains("data-blocks-header-mega-menu-cta"));
    }

    /// 狭い幅でもナビ・アクションを非表示にせず、ハンバーガーの
    /// `display: none`/`inline-flex` 切り替えを持たないこと（P1 是正:
    /// 開閉処理のないハンバーガーで内容を隠すと無 JS では到達不能になる）。
    #[test]
    fn narrow_layout_keeps_nav_and_actions_reachable() {
        assert!(!LAYOUT_CSS.contains("hamburger"));
        assert!(!LAYOUT_CSS.contains(
            ".blocks-header-mega-menu-nav[data-scope=\"navigation-menu\"][data-part=\"root\"] {\n    display: none;\n  }"
        ));
        assert!(!LAYOUT_CSS.contains(".blocks-header-mega-menu-actions {\n    display: none;\n  }"));
        assert!(LAYOUT_CSS.contains("flex-wrap: wrap;"));
    }

    /// 狭い幅ではトップ項目一覧（`navigation_menu::list`）自体も折り返し、
    /// 横スクロールを要求しないこと（PR #3273 レビュー指摘（P2）是正:
    /// バー（`.blocks-header-mega-menu-bar`）が折り返せても `list` 単体は
    /// `display: flex` のまま非折り返しだった不整合を解消する）。
    #[test]
    fn narrow_layout_wraps_nav_list_items() {
        assert!(LAYOUT_CSS.contains(
            "@media (max-width: 47.99rem) {\n  .blocks-header-mega-menu-bar {\n    flex-wrap: wrap;\n  }\n  .blocks-header-mega-menu-nav[data-scope=\"navigation-menu\"][data-part=\"root\"] {\n    flex-basis: 100%;\n  }\n  .blocks-header-mega-menu-layout [data-scope=\"navigation-menu\"][data-part=\"list\"] {\n    flex-wrap: wrap;\n  }"
        ));
    }

    /// 狭い幅では常時展開パネルがスタックして縦に伸びる分、
    /// `.blocks-header-mega-menu-page` の `min-block-size` を引き上げること
    /// （PR #3273 Bugbot 指摘是正の継承、本イシューで補助 CTA 帯ぶんの
    /// 高さをさらに引き上げて `50rem` へ更新: `.blocks-demo` の
    /// `overflow-x: auto` により縦方向がクリップ/内部スクロール化する）。
    #[test]
    fn narrow_layout_reserves_more_height_for_stacked_panel() {
        assert!(
            LAYOUT_CSS.contains(".blocks-header-mega-menu-page {\n    min-block-size: 50rem;\n  }")
        );
    }

    /// パネル項目・補助 CTA 帯・メニュートグルにアイコンが実際に描画される
    /// こと（PR #3273 レビュー指摘（P2）是正の継承）。本イシューで幅広・
    /// 狭幅の 2 インスタンス構成へ変わったため、icon scope の件数は
    /// (ブランド 1 + パネル項目 6 + 補助 CTA 帯 2) × 2 インスタンス
    /// + メニュートグル 1（狭幅のみ）= 19 件。
    #[test]
    fn panel_items_render_icons() {
        let html = render(&demo());
        assert_eq!(
            html.matches("data-scope=\"icon\"").count(),
            19,
            "html={html}"
        );
        assert!(LAYOUT_CSS.contains(".blocks-header-mega-menu-panel-link-header"));
    }

    /// パネルリンクの見出し + 説明が中央寄せではなく左揃えの列で並ぶこと
    /// （Bugbot 是正: `navigation_menu::link` recipe の `align-items:
    /// center` が column 化後も残っていた）。
    #[test]
    fn panel_link_overrides_recipe_align_items() {
        assert!(LAYOUT_CSS.contains(
            ".blocks-header-mega-menu-panel-link[data-scope=\"navigation-menu\"][data-part=\"link\"] {\n  display: flex;\n  flex-direction: column;\n  align-items: flex-start;"
        ));
    }

    /// [`LAYOUT_CSS`] が全幅パネル化（`position: static` 上書き・
    /// `inset-inline: 0`）とレスポンシブ切り替えを持つこと。
    #[test]
    fn layout_css_makes_panel_full_width() {
        assert!(LAYOUT_CSS.contains("position: static;"));
        assert!(LAYOUT_CSS.contains("inset-inline: 0;"));
        assert!(LAYOUT_CSS.contains("@media (max-width: 47.99rem)"));
    }

    /// パネル項目グリッドの列最小幅が `min(14rem, 100%)` で頭打ちされる
    /// こと（レビュー指摘是正: `minmax(14rem, 1fr)` のままだと `.blocks-demo`
    /// の枠が 14rem を下回る幅で列がはみ出し横スクロールを要求していた）。
    #[test]
    fn panel_grid_columns_clamp_to_available_width() {
        assert!(LAYOUT_CSS.contains("minmax(min(14rem, 100%), 1fr)"));
        assert!(!LAYOUT_CSS.contains("minmax(14rem, 1fr)"));
    }

    /// ルート class（`demo_class` とは別名）が [`demo`] の出力へ実際に
    /// 現れること（`blog_list_image` と同じ Bugbot 教訓の固定）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-header-mega-menu-layout\""));
        assert_ne!(super::BLOCK.demo_class, "blocks-header-mega-menu-layout");
    }

    /// 補助 CTA 帯（[`panel_footer`]）が幅広・狭幅の両パネルへ配置され、
    /// 中身がすべて `<a>`（`link::root`）で構成されること。送信先を持たない
    /// `button` を帯へ混ぜていないことを、CSS フックの直前タグが必ず
    /// `<a ` から始まる構造的検査で固定する（モジュール冒頭 rustdoc
    /// 「補助 CTA 帯」節）。
    #[test]
    fn panel_footer_band_renders_links_only() {
        let html = render(&demo());
        let marker = "data-blocks-header-mega-menu-panel-footer-link";
        let mut idx = 0;
        let mut count = 0;
        while let Some(rel) = html[idx..].find(marker) {
            let pos = idx + rel;
            let tag_start = html[..pos].rfind('<').unwrap();
            assert!(
                html[tag_start..].starts_with("<a "),
                "補助 CTA 帯のリンクは <a> で描画されるべき: {}",
                &html[tag_start..(tag_start + 20).min(html.len())]
            );
            count += 1;
            idx = pos + marker.len();
        }
        assert_eq!(count, 4, "html={html}");
        assert!(html.contains("導入ガイドを見る"));
    }

    /// 狭幅インスタンスのメニュートグルボタンが、既に展開済みであることを
    /// `aria-expanded="true"` + `aria-controls`（展開済みパネルの `id`）で
    /// 示し、かつ操作不能（`disabled`）であること（モジュール冒頭 rustdoc
    /// 「狭幅インスタンスの並記」節）。
    #[test]
    fn narrow_preview_menu_toggle_is_expanded_and_disabled() {
        let html = render(&demo());
        let marker_pos = html
            .find("data-blocks-header-mega-menu-menu-toggle")
            .expect("menu toggle marker should be present");
        let tag_start = html[..marker_pos].rfind("<button").unwrap();
        let tag_end = html[tag_start..].find('>').unwrap() + tag_start;
        let tag = &html[tag_start..tag_end];
        assert!(tag.contains(r#"aria-expanded="true""#), "tag={tag}");
        assert!(
            tag.contains(&format!(r#"aria-controls="{MOBILE_PANEL_ID}""#)),
            "tag={tag}"
        );
        assert!(tag.contains("disabled"), "tag={tag}");
        assert!(html.contains(&format!("id=\"{MOBILE_PANEL_ID}\"")));
    }

    /// 狭幅インスタンスの展開済みパネルが `hidden` を持たず、
    /// [`LAYOUT_CSS`] もビューポート幅に連動した `display: none` 切り替え
    /// を持たないこと（モジュール冒頭 rustdoc「狭幅インスタンスの並記」
    /// 節。前半 #2858 のレビュー是正〔無 JS では畳んだ内容へ到達できない〕
    /// を再発させないための固定）。
    #[test]
    fn narrow_preview_panel_is_always_visible() {
        let html = render(&demo());
        let panel_marker = format!("id=\"{MOBILE_PANEL_ID}\"");
        let panel_start = html
            .find(&panel_marker)
            .expect("mobile panel id should be present");
        let tag_start = html[..panel_start].rfind("<div").unwrap();
        let tag_end = html[tag_start..].find('>').unwrap() + tag_start;
        let tag = &html[tag_start..tag_end];
        assert!(
            !tag.contains("hidden"),
            "狭幅パネルは常時表示のため hidden を持たない: tag={tag}"
        );
        assert!(!LAYOUT_CSS.contains("display: none"));
    }
}
