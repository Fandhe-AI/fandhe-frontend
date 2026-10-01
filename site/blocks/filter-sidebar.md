# filter-sidebar

`fandhe-frontend-pre-styled-ui` の `heading` / `menu` / `link` /
`collapsible` / `fieldset` / `checkbox` / `drawer` / `button` / `skeleton`
部品を合成した、EC の商品一覧ページで使う「左サイドバーのフィルタ +
右の商品領域」の実例です。Blocks セクションは新規部品を追加するものでは
なく、既存の Themes/Primitives 部品を組み合わせた実例集であることに注意
してください（主参照は対応表 ID R0818、集約元は R0819 / R0621。出典の
固有名・ファイル名は記載しません）。

docs サイトは無 JS のため、並び替え・開閉・ドロワーの開閉は実際には
動作しない静的な表示例です。チェックボックスはすべてネイティブ
`disabled` で固定し、操作しても見た目と状態がずれません。

3 つの版を並べています。版 A（開閉式、先頭の群のみ展開）・版 B（常時
展開）・版 C（インスタンス幅を固定し、狭い幅ではサイドバーが隠れて
「フィルタ」ボタン + ドロワーに切り替わる表示）です。狭い幅（`40rem`
以下）への切り替えは `@container` で行っており、ビューポート幅ではなく
インスタンス自身の幅を基準にしています。

本 Demo は静的な表示例であり、`<form>` 要素は一切持たず、データの取得・
送信・状態管理を行いません。ボタンはすべて `type="button"` のまま送信先・
クリック後の挙動を持ちません。カテゴリ名・商品名・価格帯はすべて独自に
書いた架空のものであり、実企業名・実商品・PII を含みません。商品画像は
`skeleton` の占位表示のみです。

## Rust コード

```rust
use fandhe_frontend_core::{aside, div, li, nav, p, section, span, text, ul, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::checkbox::{self, CheckboxProps, CheckedState};
use fandhe_frontend_pre_styled_ui::collapsible;
use fandhe_frontend_pre_styled_ui::drawer::{
    self, CloseTriggerVariant, ContentIds, DrawerPlacement, OpenState,
};
use fandhe_frontend_pre_styled_ui::fieldset::{self, FieldsetProps, FieldsetRootProps};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::menu::{self, OpenState as MenuOpenState};
use fandhe_frontend_pre_styled_ui::skeleton::{skeleton, SkeletonProps, SkeletonVariant};
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// カテゴリ一覧（架空）。`(表示名,)` のみを持つ。
const CATEGORIES: &[&str] = &["すべて", "トップス", "ボトムス", "アウター", "小物"];

/// 1 件のチェックボックス選択肢（架空）。
struct Option_ {
    value: &'static str,
    label: &'static str,
    checked: bool,
}

/// カラー群の選択肢。
const COLOR_OPTIONS: &[Option_] = &[
    Option_ {
        value: "navy",
        label: "ネイビー",
        checked: true,
    },
    Option_ {
        value: "beige",
        label: "ベージュ",
        checked: false,
    },
    Option_ {
        value: "black",
        label: "ブラック",
        checked: false,
    },
];

/// サイズ群の選択肢。
const SIZE_OPTIONS: &[Option_] = &[
    Option_ {
        value: "s",
        label: "S",
        checked: false,
    },
    Option_ {
        value: "m",
        label: "M",
        checked: true,
    },
    Option_ {
        value: "l",
        label: "L",
        checked: false,
    },
];

/// 価格帯群の選択肢。
const PRICE_OPTIONS: &[Option_] = &[
    Option_ {
        value: "under-5000",
        label: "〜5,000 円",
        checked: false,
    },
    Option_ {
        value: "5000-to-15000",
        label: "5,000〜15,000 円",
        checked: false,
    },
    Option_ {
        value: "over-15000",
        label: "15,000 円〜",
        checked: false,
    },
];

/// チェックボックス 1 件（無 JS のためネイティブ `disabled` で固定、
/// モジュール doc「無 JS・到達性の方針」節）。
fn checkbox_item(name: &str, opt: &Option_) -> Node {
    let props = CheckboxProps {
        checked: if opt.checked {
            CheckedState::Checked
        } else {
            CheckedState::Unchecked
        },
        disabled: true,
        ..CheckboxProps::default()
    };
    checkbox::root(
        Size::Sm,
        ColorPalette::Accent,
        &props,
        vec![("data-blocks-filter-sidebar-checkbox", "")],
        vec![
            checkbox::hidden_input(&props, name, opt.value, vec![]),
            checkbox::control(
                &props,
                vec![],
                vec![checkbox::indicator(&props, vec![], vec![])],
            ),
            checkbox::label(&props, vec![], vec![text(opt.label)]),
        ],
    )
}

/// フィルタ群 1 つ（カラー/サイズ/価格帯）を `collapsible` で開閉式に
/// 包む（版 A/C、`form_layout_property_panel::panel_theme_collapsible` と
/// 同型）。
fn accordion_group(
    name: &str,
    label: &'static str,
    content_id: &str,
    open: bool,
    options: &[Option_],
) -> Node {
    let state = if open {
        collapsible::OpenState::Open
    } else {
        collapsible::OpenState::Closed
    };
    let items: Vec<Node> = options.iter().map(|opt| checkbox_item(name, opt)).collect();
    collapsible::root(
        state,
        true,
        vec![],
        vec![
            collapsible::trigger(
                state,
                true,
                Some(content_id),
                vec![],
                vec![
                    text(label),
                    collapsible::indicator(state, true, vec![], vec![]),
                ],
            ),
            collapsible::content(
                state,
                true,
                Some(content_id),
                vec![("data-blocks-filter-sidebar-group", "")],
                items,
            ),
        ],
    )
}

/// フィルタ群 1 つを `fieldset` + `legend` で常時展開する（版 B）。
fn fieldset_group(id: &str, name: &str, label: &'static str, options: &[Option_]) -> Node {
    let fieldset_props = FieldsetProps {
        id,
        disabled: false,
        invalid: false,
        has_helper_text: false,
    };
    let mut children: Vec<Node> =
        vec![fieldset::legend(&fieldset_props, vec![], vec![text(label)])];
    children.extend(options.iter().map(|opt| checkbox_item(name, opt)));
    fieldset::root(
        &FieldsetRootProps::default(),
        &fieldset_props,
        vec![("data-blocks-filter-sidebar-group", "")],
        children,
    )
}

/// カテゴリナビ（同インスタンスの商品領域 id へのフラグメント、
/// モジュール doc「`href` の方針」節）。
///
/// 本 Demo は実際のカテゴリ別フィルタリングを持たず、商品領域は単一の
/// 固定カードセットのため、全カテゴリを同一 href のリンクにすると
/// 「カテゴリを選んでも同じ場所へ移動する」という誤解を招く
/// （PR #3504 レビュー指摘）。先頭の「すべて」のみ実際のリンク
/// （`aria-current="page"` で選択中を示す）とし、残りは行き先を持たない
/// プレーンテキストとして示す。
fn category_nav(products_id: &str) -> Node {
    let href = format!("#{products_id}");
    let items: Vec<Node> = CATEGORIES
        .iter()
        .enumerate()
        .map(|(i, label)| {
            let child = if i == 0 {
                link::root(
                    href.as_str(),
                    &LinkProps::default(),
                    vec![("aria-current", "page")],
                    vec![text(*label)],
                )
            } else {
                span(vec![], vec![text(*label)])
            };
            li(vec![], vec![child])
        })
        .collect();
    nav(
        vec![
            ("aria-label", "カテゴリ"),
            ("class", "blocks-filter-sidebar-category-nav"),
        ],
        vec![ul(vec![], items)],
    )
}

/// フィルタパネル本体（カテゴリナビ + 3 群 + クリアボタン）。
/// `prefix` はサイドバー/ドロワーで異なる値を渡し id を一意にする
/// （モジュール doc「ID の一意性」節相当）。
fn filter_panel(prefix: &str, products_id: &str, accordion: bool) -> Node {
    let color_name = format!("{prefix}-color");
    let size_name = format!("{prefix}-size");
    let price_name = format!("{prefix}-price");
    let color_id = format!("{prefix}-color-content");
    let size_id = format!("{prefix}-size-content");
    let price_id = format!("{prefix}-price-content");
    let groups: Vec<Node> = if accordion {
        // 無 JS では disabled な trigger を開閉できず、closed な群は
        // `hidden` 属性で選択肢ごと到達不能になる（モジュール doc
        // 「無 JS・到達性の方針」節）。3 群すべてを open 固定する。
        vec![
            accordion_group(&color_name, "カラー", &color_id, true, COLOR_OPTIONS),
            accordion_group(&size_name, "サイズ", &size_id, true, SIZE_OPTIONS),
            accordion_group(&price_name, "価格帯", &price_id, true, PRICE_OPTIONS),
        ]
    } else {
        vec![
            fieldset_group(&color_id, &color_name, "カラー", COLOR_OPTIONS),
            fieldset_group(&size_id, &size_name, "サイズ", SIZE_OPTIONS),
            fieldset_group(&price_id, &price_name, "価格帯", PRICE_OPTIONS),
        ]
    };
    let mut children = vec![category_nav(products_id)];
    children.extend(groups);
    children.push(button::button(
        &ButtonProps {
            variant: ButtonVariant::Outline,
            size: Size::Sm,
            ..ButtonProps::default()
        },
        vec![],
        vec![text("条件をクリア")],
    ));
    div(vec![("class", "blocks-filter-sidebar-panel")], children)
}

/// 商品カード 1 枚（画像占位 + テキスト占位 2 行）。
fn product_card() -> Node {
    div(
        vec![("class", "blocks-filter-sidebar-card")],
        vec![
            skeleton(
                &SkeletonProps {
                    variant: SkeletonVariant::Rect,
                    ..SkeletonProps::default()
                },
                vec![("data-blocks-filter-sidebar-card-image", "")],
            ),
            skeleton(&SkeletonProps::default(), vec![]),
            skeleton(&SkeletonProps::default(), vec![]),
        ],
    )
}

/// 商品領域（6 枚のカード）。
fn product_area(products_id: &str) -> Node {
    section(
        vec![
            ("id", products_id),
            ("aria-label", "商品一覧"),
            ("class", "blocks-filter-sidebar-products"),
        ],
        (0..6).map(|_| product_card()).collect(),
    )
}

/// 並び替えメニュー（`settings_page_tabs::key_actions_menu` と同型、
/// `OpenState::Closed` 固定）。
fn sort_menu(prefix: &str) -> Node {
    let content_id = format!("{prefix}-sort-content");
    let trigger = menu::trigger(
        MenuOpenState::Closed,
        true,
        Some(content_id.as_str()),
        vec![],
        vec![text("並び替え")],
    );
    let content = menu::content(
        MenuOpenState::Closed,
        Some(content_id.as_str()),
        None,
        vec![],
        vec![
            menu::item(
                "recommended",
                false,
                false,
                vec![],
                vec![text("おすすめ順")],
            ),
            menu::item("newest", false, false, vec![], vec![text("新着順")]),
            menu::item(
                "price-asc",
                false,
                false,
                vec![],
                vec![text("価格の安い順")],
            ),
            menu::item(
                "price-desc",
                false,
                false,
                vec![],
                vec![text("価格の高い順")],
            ),
        ],
    );
    let positioner = menu::positioner(MenuOpenState::Closed, vec![], vec![content]);
    menu::root(
        Size::Sm,
        MenuOpenState::Closed,
        vec![],
        vec![trigger, positioner],
    )
}

/// フィルタドロワーのトリガー（ヘッダー行に置く）。
fn drawer_trigger(content_id: &str) -> Node {
    drawer::trigger(
        OpenState::Open,
        Some(content_id),
        vec![("class", "blocks-filter-sidebar-drawer-trigger")],
        vec![text("フィルタ")],
    )
}

/// ヘッダー行（見出し + ドロワートリガー + 並び替えメニュー）。
fn header_row(prefix: &str, heading_text: &'static str, drawer_content_id: &str) -> Node {
    div(
        vec![("class", "blocks-filter-sidebar-header")],
        vec![
            heading::heading(
                HeadingLevel::H3,
                &HeadingProps::default(),
                vec![],
                vec![text(heading_text)],
            ),
            div(
                vec![("class", "blocks-filter-sidebar-header-actions")],
                vec![drawer_trigger(drawer_content_id), sort_menu(prefix)],
            ),
        ],
    )
}

/// フィルタドロワー（狭い幅のみ表示。モジュール doc「ドロワーを枠内に
/// 収める理由」節）。
fn filter_drawer(
    prefix: &str,
    products_id: &str,
    content_id: &str,
    title_id: &str,
    accordion: bool,
) -> Node {
    drawer::root(
        Size::Sm,
        OpenState::Open,
        DrawerPlacement::Start,
        vec![("class", "blocks-filter-sidebar-drawer")],
        vec![
            drawer::backdrop(OpenState::Open, vec![], vec![]),
            drawer::positioner(
                OpenState::Open,
                DrawerPlacement::Start,
                vec![],
                vec![drawer::content(
                    OpenState::Open,
                    DrawerPlacement::Start,
                    false,
                    ContentIds {
                        id: Some(content_id),
                        labelledby: Some(title_id),
                        describedby: None,
                    },
                    vec![],
                    vec![
                        drawer::title(Some(title_id), vec![], vec![text("フィルタ")]),
                        filter_panel(&format!("{prefix}-drawer"), products_id, accordion),
                        drawer::close_trigger_with_variant(
                            CloseTriggerVariant::Text,
                            vec![],
                            vec![text("閉じる")],
                        ),
                    ],
                )],
            ),
        ],
    )
}

/// 1 インスタンス分の組み立て（見出し行 + 本体グリッド + ドロワー）。
fn instance(prefix: &str, heading_text: &'static str, accordion: bool) -> Node {
    let products_id = format!("blocks-filter-sidebar-{prefix}-products");
    let drawer_content_id = format!("blocks-filter-sidebar-{prefix}-drawer-content");
    let drawer_title_id = format!("blocks-filter-sidebar-{prefix}-drawer-title");
    div(
        vec![("class", "blocks-filter-sidebar-instance")],
        vec![
            header_row(prefix, heading_text, &drawer_content_id),
            div(
                vec![("class", "blocks-filter-sidebar-body")],
                vec![
                    aside(
                        vec![
                            ("aria-label", "商品フィルタ"),
                            ("class", "blocks-filter-sidebar-aside"),
                        ],
                        vec![filter_panel(prefix, &products_id, accordion)],
                    ),
                    product_area(&products_id),
                ],
            ),
            filter_drawer(
                prefix,
                &products_id,
                &drawer_content_id,
                &drawer_title_id,
                accordion,
            ),
        ],
    )
}

/// 版キャプション（素の `p` + class、styled `text` は `class` を落とす
/// ため使わない）。
fn version_caption(label: &'static str) -> Node {
    p(
        vec![("class", "blocks-filter-sidebar-caption")],
        vec![text(label)],
    )
}

/// `filter-sidebar` の Demo 本体。呼び出しごとに決定的な `Node` を返す
/// 純関数。版 A/B/C（モジュール doc「3 版の対応」節）を縦に並べる。
#[must_use]
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-filter-sidebar-wrap")],
        vec![
            version_caption("開閉式フィルタ（R0818、代表構成）"),
            instance("a", "新着アイテム", true),
            version_caption("常時展開フィルタ（R0819）"),
            instance("b", "新着アイテム", false),
            version_caption("狭い幅: サイドバーが隠れドロワーに切り替わる（R0621）"),
            div(
                vec![("class", "blocks-filter-sidebar-narrow-stage")],
                vec![instance("c", "新着アイテム", true)],
            ),
        ],
    )
}
```

## 原案差分メモ

- 主参照 R0818（カテゴリリンク + 開閉式フィルタ群 3 つ、先頭のみ open）を
  軸にし、集約元 R0819（常時展開フィルタ）・R0621（広い幅は固定サイド
  バー、狭い幅はドロワー）を版 B・版 C としてそれぞれ独立したインスタンス
  で並記しています。
- 版 A・版 C の `collapsible` と版 B の `fieldset` はいずれも
  `disabled: true`/ネイティブ `disabled` で固定し、docs サイトの無 JS
  制約の下でも操作後の状態と見た目がずれません（押しても何も起きない
  静的な見本です）。
- カテゴリリンクは `href="#"` を避け、同じインスタンスの商品領域 id への
  ページ内フラグメントにしています。「カテゴリを選ぶと商品一覧へ移る」
  という意味と一致させ、`linkcheck` が同一ページの id 集合と突き合わせて
  検証できるようにするためです。
- 狭い幅での切り替え（サイドバー非表示 + 「フィルタ」ボタン + ドロワー
  表示）は、各インスタンスが独立した `@container` を持つことで、幅に
  応じてどちらが実際に使われるかを決定的に示しています。ドロワーの
  固定オーバーレイ（`position: fixed`）は Demo 枠内に収まるよう中和
  しています。
- ブラウザでの実機確認（`40rem` 前後のコンテナ幅切替・ライト/ダーク両
  テーマ）はサンドボックス制約により未実施です。`cargo test` による
  出力検証のみで代替しました。

## 関連情報

- [Heading](../themes/heading.md)
- [Menu](../themes/menu.md)
- [Link](../themes/link.md)
- [Collapsible](../themes/collapsible.md)
- [Fieldset](../themes/fieldset.md)
- [Checkbox](../themes/checkbox.md)
- [Drawer](../themes/drawer.md)
- [Button](../themes/button.md)
- [Skeleton](../themes/skeleton.md)
