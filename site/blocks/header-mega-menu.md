# header-mega-menu

`fandhe-frontend-pre-styled-ui` の `navigation-menu` / `button` / `icon` /
`link` 部品を合成した、全幅メガメニュー付きヘッダーです。Blocks
セクションは新規部品を追加するものではなく、既存の Themes/Primitives
部品を組み合わせた実例集であることに注意してください（主参照は対応表 ID
R0984。出典の固有名・ファイル名は記載しません）。

幅を制限したバー（ブランド / ナビ / アクション）の下に、ヘッダー全幅まで
広がるドロップダウンパネルを持ちます。パネルはアイコン付き項目を複数列に
並べた構成に加え、下部に実在ページへの補助 CTA 帯（リンク 2 件）を持ちます。
無 JS の静的表示のためハンバーガーへの開閉切り替えは持たず、幅広インスタンス
は狭い幅ではバー内でナビ・アクションを折り返して常時到達可能なまま残します。
加えて、狭いビューポートでの見え方を「幅広（プロダクトを展開）」「狭幅
（メニュー展開時）」の 2 状態として並記します。狭幅側のメニューボタンは
既に展開済み（`aria-expanded="true"`）で操作不能（`disabled`）な状態を
静的に示すのみで、実際の開閉処理は持ちません。

本 Demo は静的な表示例であり、両インスタンスとも唯一のドロップダウン
（プロダクト）を常時展開（open）した状態で固定します。トリガーを持たない
トップ項目（料金・ドキュメント）はリンクのみで構成し、無 JS のドキュメント
サイトでも本文へ到達できない閉じたトリガーを残しません。補助 CTA 帯は
送信先を持たない `button` ではなく実在ページへ遷移する `link` のみで
構成します。`<form>` 要素は一切持たず、データの取得・送信・状態管理を
行いません。ボタンは `type="button"` のまま送信先を持ちません。文言・
ブランド名はすべて独自に書いた架空のものであり、実企業名・実クレデンシャル・
PII を含みません。

## Rust コード

```rust
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
```

## 原案差分メモ

主参照（R0984）は開閉可能な JS 実装のメガメニューを想定しますが、本
Demo は無 JS の静的合成例のため以下の差分があります。

- 開閉・hover 展開を持たず、唯一のドロップダウン（プロダクト）を常時
  open 固定し、そのトリガーは `disabled` にして操作不能を明示します。
- ドロップダウンはプロダクト 1 件のみで、料金・ドキュメントはリンクのみの
  項目です。
- 「狭幅ではハンバーガーに畳む」という一般的な実装は、ビューポート幅に
  連動した自動切り替えではなく、「狭幅（メニュー展開時）」インスタンスの
  並記で表現します。メニューボタンは `disabled` + `aria-expanded="true"`
  の静的状態のみを示します。
- 幅広インスタンスは狭い画面でバー・ナビ一覧を折り返し、常時到達可能な
  まま残します。
- 主 CTA ボタン（「無料で始める」）は遷移先を持たない `disabled` 表示です。
  一方、パネル下部の補助 CTA 帯は実在ページへのリンクのみで構成します。
- 文言・ロゴ・配色はすべて独自に書いた架空のものです。

関連情報: [Navigation Menu](../themes/navigation-menu.md) /
[Button](../themes/button.md) / [Icon](../themes/icon.md) /
[Link](../themes/link.md)
