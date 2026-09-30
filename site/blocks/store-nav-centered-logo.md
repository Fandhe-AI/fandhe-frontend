# store-nav-centered-logo

`fandhe-frontend-pre-styled-ui` の `navigation-menu` / `link` / `button` /
`icon` / `image` 部品を合成した、中央にロゴを配置したストアナビゲーション
の実例です。Blocks セクションは新規部品を追加するものではなく、既存の
Themes/Primitives 部品を組み合わせた実例集であることに注意してください
（主参照は対応表 ID R1316、集約元は 1 件のみで差分なし。出典の固有名・
ファイル名は記載しません）。

上部帯を持たない 1 行バーで構成し、中央にブランドロゴ、左にカテゴリの
ナビゲーション、右に通貨・検索・アカウント・カートを配置しています。左の
カテゴリナビは「レディース」項目のメガメニューを常時展開した状態で固定し、
注目画像 3 枚とカテゴリ・ブランドの階層化した項目列を並べます。残りの
トップ項目（メンズ・ホーム用品）はトリガーを持たないリンクのみで構成し、
無 JS のドキュメントサイトでも本文へ到達できない閉じたトリガーを残しません。

本 Demo は静的な表示例であり、唯一のドロップダウン（レディース）を常時
展開（open）した状態で固定します。トリガーは押しても状態が変わらない
no-op のため `disabled` にして見た目と操作可否の食い違いを避けます。通貨・
検索・アカウント・カートも遷移先・送信処理を持たない no-op のため同様に
`disabled` にします。`<form>` 要素は一切持たず、データの取得・送信・
状態管理を行いません。ボタンは `type="button"` のまま送信先を持ちません。
狭い幅（`47.99rem` 以下）ではバーを 1 列へ折り返し、ナビ・アクションを
非表示にせず常時到達可能なまま残します（無 JS で開閉するメニューボタンは
実装できないため、内容を隠す構成は採りません）。文言・ブランド名・画像は
すべて独自に書いた架空のものであり、実企業名・実ブランド・PII を含みません。

この骨格・主要領域の実装は前半（本イシュー）の範囲であり、狭幅
（メニュー展開時）状態の並記は後半（`drawer` 部品を使用）で追加予定です。

## Rust コード

```rust
use fandhe_frontend_core::{div, el, span, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::icon::{self, IconProps};
use fandhe_frontend_pre_styled_ui::image::{self, ImageProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::navigation_menu::{self, NavigationMenuProps, OpenState};
use fandhe_frontend_pre_styled_ui::Size;

/// パネルの注目画像 1 件（見出し, 補足, href, alt）。href はサイト内に
/// 実在する索引ページへの相対パス（モジュール冒頭 rustdoc「href の方針」
/// 節）。画像は [`dummy_assets::PRODUCT_SRC`]（モノトーン抽象図形の
/// SVG）を使い回す。
type FeaturedItem = (&'static str, &'static str, &'static str);

/// パネルの列 1 件（列見出し, 項目〔表示名, href〕4 件）。
type PanelColumn = (&'static str, [(&'static str, &'static str); 4]);

/// 「レディース」パネルの注目画像 3 件。
const FEATURED_ITEMS: [FeaturedItem; 3] = [
    (
        "新作アウター",
        "今季の新作をまとめてチェック",
        "../../themes/",
    ),
    (
        "定番トップス",
        "毎日使えるベーシックアイテム",
        "../../guides/",
    ),
    (
        "アクセサリー",
        "コーディネートのアクセントに",
        "../../primitives/",
    ),
];

/// 「レディース」パネルの階層化された項目列 2 列。
const PANEL_COLUMNS: [PanelColumn; 2] = [
    (
        "カテゴリ",
        [
            ("トップス", "../../themes/"),
            ("ボトムス", "../../primitives/"),
            ("シューズ", "../../api/"),
            ("バッグ", "../../examples/"),
        ],
    ),
    (
        "ブランド",
        [
            ("アトリエノア", "../../guides/"),
            ("ルミエールベーシック", "../../themes/"),
            ("ノームアンドコー", "../../primitives/"),
            ("ソレイユスタジオ", "../../examples/"),
        ],
    ),
];

/// 唯一開いた状態で固定するトリガーの `id`（モジュール冒頭 rustdoc「id
/// 接頭辞」節。項目が 1 件のみのため `format!` による添字展開は行わない）。
const WOMEN_TRIGGER_ID: &str = "blocks-store-nav-centered-logo-women-trigger";
/// [`WOMEN_TRIGGER_ID`] と対になる `content` の `id`。
const WOMEN_CONTENT_ID: &str = "blocks-store-nav-centered-logo-women-content";

/// パネル項目・中央ロゴ・アクションで共有する線画アイコン（装飾用途、
/// `icon::icon` の `currentColor` 継承に任せ生の色リテラルは持ち込まない）。
fn line_icon(path_d: &str) -> Node {
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

/// 中央ロゴ領域（アイコン + 架空のブランド名、実在サイトの索引ページへの
/// リンク）。実在ブランドのロゴ・商標を模さない独自の単純図形
/// （`docs/design/wireframe-ui-architecture.md` と同じ判断軸）。
fn brand() -> Node {
    link::root(
        "../../",
        &LinkProps::default(),
        vec![("data-blocks-store-nav-centered-logo-brand", "")],
        vec![
            line_icon("M12 3a9 9 0 1 0 0 18a9 9 0 0 0 0-18zM8 12h8M12 8v8"),
            span(vec![], vec![text("Solaris Mart")]),
        ],
    )
}

/// 注目画像 1 件（画像 + 見出し + 補足）。
fn featured_link((title, note, href): &FeaturedItem) -> Node {
    navigation_menu::link(
        href,
        false,
        vec![("class", "blocks-store-nav-centered-logo-featured")],
        vec![
            image::image(
                &ImageProps::new(dummy_assets::PRODUCT_SRC, title),
                vec![("data-blocks-store-nav-centered-logo-featured-image", "")],
            ),
            span(
                vec![("class", "blocks-store-nav-centered-logo-featured-title")],
                vec![text(*title)],
            ),
            span(
                vec![("class", "blocks-store-nav-centered-logo-featured-note")],
                vec![text(*note)],
            ),
        ],
    )
}

/// パネル 1 列分（列見出し + リンク項目 4 件）。
fn panel_column((heading, items): &PanelColumn) -> Node {
    let links: Vec<Node> = items
        .iter()
        .map(|(label, href)| {
            navigation_menu::link(
                href,
                false,
                vec![("class", "blocks-store-nav-centered-logo-panel-link")],
                vec![text(*label)],
            )
        })
        .collect();
    let mut children = vec![span(
        vec![("class", "blocks-store-nav-centered-logo-panel-heading")],
        vec![text(*heading)],
    )];
    children.extend(links);
    div(
        vec![("class", "blocks-store-nav-centered-logo-panel-column")],
        children,
    )
}

/// 「レディース」トップ項目（唯一のドロップダウン、常時 open 固定）。
fn women_item(props: &NavigationMenuProps) -> Node {
    let state = OpenState::Open;
    let featured: Vec<Node> = FEATURED_ITEMS.iter().map(featured_link).collect();
    let columns: Vec<Node> = PANEL_COLUMNS.iter().map(panel_column).collect();

    navigation_menu::item(
        state,
        false,
        props,
        "women",
        vec![],
        vec![
            navigation_menu::trigger(
                state,
                true,
                "women",
                Some(WOMEN_TRIGGER_ID),
                Some(WOMEN_CONTENT_ID),
                vec![],
                vec![
                    text("レディース"),
                    navigation_menu::item_indicator(state, props, "women", vec![], vec![text("▾")]),
                ],
            ),
            navigation_menu::content(
                state,
                props,
                "women",
                Some(WOMEN_CONTENT_ID),
                Some(WOMEN_TRIGGER_ID),
                vec![],
                vec![div(
                    vec![("class", "blocks-store-nav-centered-logo-panel-inner")],
                    vec![
                        div(
                            vec![("class", "blocks-store-nav-centered-logo-featured-row")],
                            featured,
                        ),
                        div(
                            vec![("class", "blocks-store-nav-centered-logo-panel-columns")],
                            columns,
                        ),
                    ],
                )],
            ),
        ],
    )
}

/// トリガーを持たない、リンクのみのトップ項目（メンズ・ホーム用品）。
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

/// 左側のカテゴリナビ（「レディース」のメガメニュー + リンク項目 2 件）。
fn category_nav() -> Node {
    let props = NavigationMenuProps::default();
    navigation_menu::root(
        &props,
        "カテゴリ",
        vec![("class", "blocks-store-nav-centered-logo-nav")],
        vec![navigation_menu::list(
            &props,
            vec![],
            vec![
                women_item(&props),
                link_item(&props, "men", "メンズ", "../../themes/"),
                link_item(&props, "home", "ホーム用品", "../../guides/"),
            ],
        )],
    )
}

/// 右側のアクション（通貨・検索・アカウント・カート）。いずれも遷移先・
/// 送信処理を持たない no-op のため `disabled: true` にする（モジュール
/// 冒頭 rustdoc「`<form>` を持たない」節）。
fn actions() -> Node {
    div(
        vec![(
            "class",
            "blocks-store-nav-centered-logo-actions",
        )],
        vec![
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Ghost,
                    disabled: true,
                    ..ButtonProps::default()
                },
                vec![(
                    "data-blocks-store-nav-centered-logo-currency",
                    "",
                )],
                vec![text("JPY")],
            ),
            button::icon_button(
                &ButtonProps {
                    variant: ButtonVariant::Ghost,
                    disabled: true,
                    ..ButtonProps::default()
                },
                "検索",
                vec![(
                    "data-blocks-store-nav-centered-logo-search",
                    "",
                )],
                vec![line_icon(
                    "M10 4a6 6 0 1 0 0 12a6 6 0 0 0 0-12zM20 20l-4.35-4.35",
                )],
            ),
            button::icon_button(
                &ButtonProps {
                    variant: ButtonVariant::Ghost,
                    disabled: true,
                    ..ButtonProps::default()
                },
                "アカウント",
                vec![(
                    "data-blocks-store-nav-centered-logo-account",
                    "",
                )],
                vec![line_icon("M12 12a4 4 0 1 0 0-8 4 4 0 0 0 0 8zM4 20c0-4 4-6 8-6s8 2 8 6")],
            ),
            div(
                vec![(
                    "class",
                    "blocks-store-nav-centered-logo-cart",
                )],
                vec![
                    button::icon_button(
                        &ButtonProps {
                            variant: ButtonVariant::Ghost,
                            disabled: true,
                            ..ButtonProps::default()
                        },
                        "カート",
                        vec![(
                            "data-blocks-store-nav-centered-logo-cart-button",
                            "",
                        )],
                        vec![line_icon("M3 4h2l2 10h10l2-7H6M9 20a1 1 0 1 0 0-2a1 1 0 0 0 0 2ZM17 20a1 1 0 1 0 0-2a1 1 0 0 0 0 2Z")],
                    ),
                    span(
                        vec![(
                            "class",
                            "blocks-store-nav-centered-logo-cart-count",
                        )],
                        vec![text("2")],
                    ),
                ],
            ),
        ],
    )
}

/// 1 行バー（左: カテゴリナビ / 中央: ロゴ / 右: アクション）。
fn bar() -> Node {
    div(
        vec![("class", "blocks-store-nav-centered-logo-bar")],
        vec![category_nav(), brand(), actions()],
    )
}

/// ダミーのページ本文（`.blocks-demo` のはみ出し対策、モジュール冒頭
/// rustdoc参照）。
fn page_placeholder() -> Node {
    div(
        vec![("class", "blocks-store-nav-centered-logo-page")],
        vec![text(
            "ページ本文（ダミー）。展開済みパネルの下に十分な高さを確保するための枠。",
        )],
    )
}

/// `store-nav-centered-logo` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数（モジュール doc「静的表示」節）。前半 #3094 の範囲では
/// 幅広インスタンス 1 種のみを描画する（狭幅の並記は後半 #3095 で追加）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-store-nav-centered-logo-layout")],
        vec![
            div(
                vec![("class", "blocks-store-nav-centered-logo-bar-wrap")],
                vec![bar()],
            ),
            page_placeholder(),
        ],
    )
}
```
