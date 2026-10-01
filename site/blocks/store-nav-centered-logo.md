# store-nav-centered-logo

`fandhe-frontend-pre-styled-ui` の `navigation-menu` / `link` / `button` /
`icon` / `image` / `drawer` 部品を合成した、中央にロゴを配置したストア
ナビゲーションの実例です。Blocks セクションは新規部品を追加するものでは
なく、既存の Themes/Primitives 部品を組み合わせた実例集であることに注意
してください（主参照は対応表 ID R1316、集約元は 1 件のみで差分なし。
出典の固有名・ファイル名は記載しません）。

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
文言・ブランド名・画像はすべて独自に書いた架空のものであり、実企業名・
実ブランド・PII を含みません。

幅広・狭幅（メニュー展開時）の 2 状態をキャプション付きで並記します。
幅広インスタンスは狭い幅（`47.99rem` 以下）でもバーを 1 列へ折り返し、
ナビ・アクションを非表示にせず常時到達可能なまま残します（無 JS で開閉
するメニューボタンは実装できないため、内容を隠す構成は採りません）。
狭幅インスタンスはメニューボタン（`disabled` + `aria-expanded="true"` の
静的状態）と、`drawer` 部品で組んだ展開済みメニュー（`hidden` を持たず
非モーダル）を常時表示し、狭い画面での見え方を実演します。

## Rust コード

```rust
use fandhe_frontend_core::{div, el, span, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::drawer::{self, ContentIds, DrawerPlacement};
use fandhe_frontend_pre_styled_ui::icon::{self, IconProps};
use fandhe_frontend_pre_styled_ui::image::{self, ImageProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::navigation_menu::{self, NavigationMenuProps, OpenState};
use fandhe_frontend_pre_styled_ui::{Orientation, Size};

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

/// 狭幅インスタンス（[`mobile_preview`]）側のレディーストリガー `id`
/// （モジュール冒頭 rustdoc「狭幅インスタンスの並記」節）。
const MOBILE_WOMEN_TRIGGER_ID: &str = "blocks-store-nav-centered-logo-mobile-women-trigger";
/// [`MOBILE_WOMEN_TRIGGER_ID`] と対になる `content` の `id`。
const MOBILE_WOMEN_CONTENT_ID: &str = "blocks-store-nav-centered-logo-mobile-women-content";
/// 狭幅インスタンスの展開済み drawer 本体（`content` パート）の `id`
/// （メニュートグルボタンの `aria-controls` が参照する）。
const MOBILE_DRAWER_CONTENT_ID: &str = "blocks-store-nav-centered-logo-mobile-drawer";
/// [`MOBILE_DRAWER_CONTENT_ID`] の `aria-labelledby` が参照する `title`
/// の `id`。
const MOBILE_DRAWER_TITLE_ID: &str = "blocks-store-nav-centered-logo-mobile-drawer-title";

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
/// `trigger_id`/`content_id` は呼び出し側（幅広 [`category_nav`] 呼び出し・
/// 狭幅 [`category_nav`] 呼び出し）ごとに異なる固定文字列を渡す
/// （モジュール冒頭 rustdoc「id 接頭辞」節）。
fn women_item(props: &NavigationMenuProps, trigger_id: &str, content_id: &str) -> Node {
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
                Some(trigger_id),
                Some(content_id),
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
                Some(content_id),
                Some(trigger_id),
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

/// カテゴリナビ（「レディース」のメガメニュー + リンク項目 2 件）。幅広
/// バー（[`bar`]）・狭幅インスタンス（[`mobile_drawer`]）の双方から
/// `orientation`・`class`・`aria-label`・`id` 接頭辞のみを差し替えて
/// 呼ばれる共通部品（モジュール冒頭 rustdoc「狭幅インスタンスの並記」
/// 節）。
fn category_nav(
    props: &NavigationMenuProps,
    class: &str,
    aria_label: &str,
    trigger_id: &str,
    content_id: &str,
) -> Node {
    navigation_menu::root(
        props,
        aria_label,
        vec![("class", class)],
        vec![navigation_menu::list(
            props,
            vec![],
            vec![
                women_item(props, trigger_id, content_id),
                link_item(props, "men", "メンズ", "../../themes/"),
                link_item(props, "home", "ホーム用品", "../../guides/"),
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
        vec![
            category_nav(
                &NavigationMenuProps::default(),
                "blocks-store-nav-centered-logo-nav",
                "カテゴリ",
                WOMEN_TRIGGER_ID,
                WOMEN_CONTENT_ID,
            ),
            brand(),
            actions(),
        ],
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

/// 幅広インスタンス（1 行バー + ダミー本文）。[`demo`] が狭幅インスタンス
/// （[`mobile_preview`]）と並べて描画する。
fn layout() -> Node {
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

/// 状態並記の見出し（モジュール冒頭 rustdoc「狭幅インスタンスの並記」
/// 節）。
fn state_label(label: &str) -> Node {
    span(
        vec![("class", "blocks-store-nav-centered-logo-state-label")],
        vec![text(label)],
    )
}

/// 狭幅インスタンスのバー（ブランド + メニュートグルボタン）。トリガーは
/// 押しても状態が変わらない no-op のため `disabled: true` にする
/// （モジュール冒頭 rustdoc「狭幅インスタンスの並記」節）。
/// `drawer::trigger` が `state`/`controls` から `aria-haspopup="dialog"`・
/// `aria-expanded="true"`・`aria-controls` を自動出力するため、これらを
/// `attrs` へ重複して渡さない。
fn mobile_bar() -> Node {
    div(
        vec![("class", "blocks-store-nav-centered-logo-mobile-bar")],
        vec![
            brand(),
            drawer::trigger(
                OpenState::Open,
                Some(MOBILE_DRAWER_CONTENT_ID),
                vec![
                    ("aria-label", "メニュー"),
                    ("disabled", ""),
                    ("data-disabled", ""),
                    ("data-blocks-store-nav-centered-logo-menu-toggle", ""),
                ],
                vec![line_icon("M3 6h18M3 12h18M3 18h18")],
            ),
        ],
    )
}

/// 狭幅インスタンスの展開済み drawer（`root` + backdrop/positioner/content
/// の anatomy、モジュール冒頭 rustdoc「狭幅インスタンスの並記」節）。
/// 常時 [`OpenState::Open`] で固定するため `hidden` 属性を持たない。
/// `modal` は `false`（静的デモは閉じる機構を持たず外側に説明・コード・
/// ナビゲーションがあるため、表示実態と一致させる）。`drawer::root` は
/// `drop_class_attr` で呼び出し側 `class` を除去するため、スコープ用の
/// class フックは呼び出し元（[`mobile_preview`]）の外側ラッパーへ付ける
/// （モジュール冒頭 rustdoc「CSS フックの選び方」節）。
fn mobile_drawer() -> Node {
    let state = OpenState::Open;
    let placement = DrawerPlacement::Start;
    drawer::root(
        Size::Sm,
        state,
        placement,
        vec![],
        vec![
            mobile_bar(),
            div(
                vec![("class", "blocks-store-nav-centered-logo-mobile-panel-wrap")],
                vec![
                    drawer::backdrop(state, vec![], vec![]),
                    drawer::positioner(
                        state,
                        placement,
                        vec![],
                        vec![drawer::content(
                            state,
                            placement,
                            false,
                            ContentIds {
                                id: Some(MOBILE_DRAWER_CONTENT_ID),
                                labelledby: Some(MOBILE_DRAWER_TITLE_ID),
                                describedby: None,
                            },
                            vec![],
                            vec![
                                drawer::close_trigger(
                                    vec![
                                        ("aria-label", "閉じる"),
                                        ("disabled", ""),
                                        ("data-disabled", ""),
                                    ],
                                    vec![line_icon("M6 6l12 12M18 6L6 18")],
                                ),
                                drawer::title(
                                    Some(MOBILE_DRAWER_TITLE_ID),
                                    vec![],
                                    vec![text("メニュー")],
                                ),
                                category_nav(
                                    &NavigationMenuProps {
                                        orientation: Orientation::Vertical,
                                    },
                                    "blocks-store-nav-centered-logo-mobile-nav",
                                    "カテゴリ（狭幅）",
                                    MOBILE_WOMEN_TRIGGER_ID,
                                    MOBILE_WOMEN_CONTENT_ID,
                                ),
                                actions(),
                            ],
                        )],
                    ),
                ],
            ),
        ],
    )
}

/// 狭幅インスタンス全体（ブランド + メニュートグル + 展開済み drawer）。
/// [`demo`] が幅広インスタンス（[`layout`]）と並べて描画する。
fn mobile_preview() -> Node {
    div(
        vec![("class", "blocks-store-nav-centered-logo-mobile")],
        vec![mobile_drawer()],
    )
}

/// `store-nav-centered-logo` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数（モジュール doc「静的表示」節）。幅広インスタンス
/// （[`layout`]）と狭幅（メニュー展開時）インスタンス（[`mobile_preview`]）
/// を見出し付きで並記する（本イシューで追加、モジュール冒頭 rustdoc
/// 「狭幅インスタンスの並記」節）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-store-nav-centered-logo-states")],
        vec![
            state_label("幅広（レディースを展開）"),
            layout(),
            state_label("狭幅（メニュー展開時）"),
            mobile_preview(),
        ],
    )
}
```

## 原案差分メモ

主参照（R1316）は開閉可能なメガメニュー・狭幅ハンバーガーを想定します
が、本 Demo は無 JS のため以下の差分があります。

- ドロップダウンはレディース 1 件のみを常時 open 固定し、そのトリガーは
  `disabled` にして操作不能を明示します。
- 「狭幅ではハンバーガーに畳む」という一般的な実装は、ビューポート幅に
  連動した自動切り替えではなく、「狭幅（メニュー展開時）」インスタンスの
  並記で表現します。メニューボタン・閉じるボタンは `disabled` の静的状態
  のみを示します。
- 通貨・検索・アカウント・カートは遷移先・送信処理を持たない no-op のため
  `disabled` にします。
- 文言・ロゴ・画像はすべて独自に書いた架空のものです。

集約元は主参照 1 件のみであり、並記が必要な集約元差分はありません。

関連情報: [Navigation Menu](../themes/navigation-menu.md) /
[Link](../themes/link.md) / [Button](../themes/button.md) /
[Icon](../themes/icon.md) / [Image](../themes/image.md) /
[Drawer](../themes/drawer.md)
