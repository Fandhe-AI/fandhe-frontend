# filter-overlay-panel

`fandhe-frontend-pre-styled-ui` の `drawer` / `dialog` / `button` / `menu` /
`checkbox` / `fieldset` / `skeleton` 部品を合成した、商品一覧の上に並び替え
メニューとフィルタボタンの横バーを置き、フィルタボタンで開くパネル（カテゴリ・
価格帯・在庫の条件 + 解除・適用ボタン）を併せ持つ合成例です。Blocks セクションは
新規部品を追加するものではなく、既存の Themes 部品を組み合わせた実例集である
ことに注意してください（主参照・集約元はいずれも私有カタログ上の一件のみです。
出典の固有名・ファイル名は記載しません）。

集約元には画面端から出るドロワー形式と、中央に出るダイアログ形式の 2 系統が
あったため、両者の差分が分かるよう縦に併記しています。

本 Demo は静的な表示例であり、開閉・フォーカストラップ・Escape キー・並び替え・
絞り込みの実処理は一切扱いません。docs サイトは JS ハイドレーションを行わない
設計のため、パネルが既に開いた初期状態のみを固定して掲示します。`<form>` 要素は
出力せず、ボタンはすべて `type="button"` のままで、送信先・絞り込み処理・状態
管理は一切持ちません（`docs/policy/intentional-non-adoption.md` §3.25 の責務
境界: UI コンポーネント層はアプリケーションロジックを内包しません。実際に絞り
込み処理を実装する場合は、利用者自身の Rust/JS コードで実装してください）。

checkbox はすべてネイティブ `disabled` で固定しています。無 JS 環境では
ラベルクリックでチェック状態がネイティブに切り替わってしまい、見た目
（`data-state`）が追従しない不整合が生じるためです。

画面幅が狭いときはドロワーの幅を広げ、ダイアログの余白を縮め、解除・適用ボタンを
縦積み・全幅にし、商品一覧の列幅を縮めます（ダイアログ版は `dialog::footer` 自体も
縦方向・幅いっぱいに伸ばし、ボタン列が縮み幅にならないようにしています）。
各版のキャプションは、開いたパネルと背景が覆う枠の外側に置いています。

## Rust コード

```rust
use fandhe_frontend_core::{div, p, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::checkbox::{self, CheckboxProps, CheckedState};
use fandhe_frontend_pre_styled_ui::dialog::{self, ContentIds as DialogContentIds};
use fandhe_frontend_pre_styled_ui::drawer::{self, DrawerPlacement};
use fandhe_frontend_pre_styled_ui::fieldset::{self, FieldsetProps, FieldsetRootProps};
use fandhe_frontend_pre_styled_ui::menu::{self, OpenState};
use fandhe_frontend_pre_styled_ui::skeleton::{skeleton, SkeletonProps, SkeletonVariant};
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// 1 バリアント分の id 一式（drawer 版・dialog 版で重複しないよう
/// インスタンスごとに個別の定数を持つ、モジュール doc「2 バリアント併記」
/// 節参照）。すべて接頭辞 `blocks-filter-overlay-panel-{drawer|dialog}-`
/// を持つ。
struct VariantIds {
    title: &'static str,
    description: &'static str,
    content: &'static str,
    sort_trigger: &'static str,
    sort_content: &'static str,
    fieldset_category: &'static str,
    fieldset_price: &'static str,
    fieldset_stock: &'static str,
}

const DRAWER_IDS: VariantIds = VariantIds {
    title: "blocks-filter-overlay-panel-drawer-title",
    description: "blocks-filter-overlay-panel-drawer-description",
    content: "blocks-filter-overlay-panel-drawer-content",
    sort_trigger: "blocks-filter-overlay-panel-drawer-sort-trigger",
    sort_content: "blocks-filter-overlay-panel-drawer-sort-content",
    fieldset_category: "blocks-filter-overlay-panel-drawer-fieldset-category",
    fieldset_price: "blocks-filter-overlay-panel-drawer-fieldset-price",
    fieldset_stock: "blocks-filter-overlay-panel-drawer-fieldset-stock",
};

const DIALOG_IDS: VariantIds = VariantIds {
    title: "blocks-filter-overlay-panel-dialog-title",
    description: "blocks-filter-overlay-panel-dialog-description",
    content: "blocks-filter-overlay-panel-dialog-content",
    sort_trigger: "blocks-filter-overlay-panel-dialog-sort-trigger",
    sort_content: "blocks-filter-overlay-panel-dialog-sort-content",
    fieldset_category: "blocks-filter-overlay-panel-dialog-fieldset-category",
    fieldset_price: "blocks-filter-overlay-panel-dialog-fieldset-price",
    fieldset_stock: "blocks-filter-overlay-panel-dialog-fieldset-stock",
};

/// 並び替えメニュー + フィルタボタンの横バー（ドロワー版・ダイアログ版
/// 共通ヘルパ）。並び替えメニューは無 JS のため `OpenState::Closed` 固定
/// （モジュール doc「menu を閉じた状態で固定する理由」節参照）。
fn toolbar(ids: &VariantIds) -> Node {
    let sort_trigger = menu::trigger(
        OpenState::Closed,
        false,
        Some(ids.sort_content),
        vec![("id", ids.sort_trigger)],
        vec![text("並び替え")],
    );
    let sort_items = vec![
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
    ];
    let sort_content = menu::content(
        OpenState::Closed,
        Some(ids.sort_content),
        Some(ids.sort_trigger),
        vec![],
        sort_items,
    );
    let sort_positioner = menu::positioner(OpenState::Closed, vec![], vec![sort_content]);
    let sort_menu = menu::root(
        Size::Sm,
        OpenState::Closed,
        vec![],
        vec![sort_trigger, sort_positioner],
    );
    let filter_button = button::button(
        &ButtonProps {
            variant: ButtonVariant::Outline,
            ..ButtonProps::default()
        },
        vec![("data-blocks-filter-overlay-panel-filter-button", "")],
        vec![text("フィルタ")],
    );
    div(
        vec![("class", "blocks-filter-overlay-panel-toolbar")],
        vec![sort_menu, filter_button],
    )
}

/// 商品一覧の占位（`skeleton` 6 件、ドロワー版・ダイアログ版共通）。
fn product_grid() -> Node {
    let mut cards = Vec::with_capacity(6);
    for _ in 0..6 {
        cards.push(div(
            vec![("class", "blocks-filter-overlay-panel-product-card")],
            vec![
                skeleton(
                    &SkeletonProps {
                        variant: SkeletonVariant::Rect,
                        ..SkeletonProps::default()
                    },
                    vec![("data-blocks-filter-overlay-panel-product-image", "")],
                ),
                skeleton(&SkeletonProps::default(), vec![]),
                skeleton(
                    &SkeletonProps::default(),
                    vec![("data-blocks-filter-overlay-panel-product-price", "")],
                ),
            ],
        ));
    }
    div(vec![("class", "blocks-filter-overlay-panel-grid")], cards)
}

/// ネイティブ disabled の checkbox 1 件を組み立てる（モジュール doc
/// 「checkbox をネイティブ disabled にする理由」節参照）。
fn filter_checkbox(
    id_hook: &'static str,
    name: &'static str,
    label_text: &str,
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
    checkbox::root(
        Size::Md,
        ColorPalette::Accent,
        &props,
        vec![("data-blocks-filter-overlay-panel-checkbox", id_hook)],
        vec![
            checkbox::hidden_input(&props, name, "on", vec![]),
            checkbox::control(
                &props,
                vec![],
                vec![checkbox::indicator(&props, vec![], vec![])],
            ),
            checkbox::label(&props, vec![], vec![text(label_text.to_string())]),
        ],
    )
}

/// 1 フィルタグループ（`fieldset` + checkbox 群）を組み立てる。
fn filter_group(id: &'static str, legend_text: &str, items: &[(&'static str, &str, bool)]) -> Node {
    let props = FieldsetProps {
        id,
        disabled: false,
        invalid: false,
        has_helper_text: false,
    };
    let mut children: Vec<Node> = vec![fieldset::legend(
        &props,
        vec![],
        vec![text(legend_text.to_string())],
    )];
    for (index, (name, label_text, checked)) in items.iter().enumerate() {
        let hook: &'static str = if index == 0 { "first" } else { "" };
        children.push(filter_checkbox(hook, name, label_text, *checked));
    }
    fieldset::root(
        &FieldsetRootProps::default(),
        &props,
        vec![("data-blocks-filter-overlay-panel-fieldset", "")],
        children,
    )
}

/// フィルタ群（カテゴリ/価格帯/在庫の 3 グループ）。
fn filter_groups(ids: &VariantIds) -> Node {
    div(
        vec![("data-blocks-filter-overlay-panel-body", "")],
        vec![
            filter_group(
                ids.fieldset_category,
                "カテゴリ",
                &[
                    ("category-tops", "トップス", true),
                    ("category-bottoms", "ボトムス", false),
                    ("category-shoes", "シューズ", false),
                ],
            ),
            filter_group(
                ids.fieldset_price,
                "価格帯",
                &[
                    ("price-under-3000", "3,000 円未満", false),
                    ("price-3000-10000", "3,000〜10,000 円", true),
                    ("price-over-10000", "10,000 円以上", false),
                ],
            ),
            filter_group(
                ids.fieldset_stock,
                "在庫",
                &[("stock-only", "在庫ありのみ", false)],
            ),
        ],
    )
}

/// 解除・適用ボタン列（ドロワー版・ダイアログ版共通）。
fn panel_actions() -> Node {
    div(
        vec![("data-blocks-filter-overlay-panel-footer", "")],
        vec![
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Outline,
                    ..ButtonProps::default()
                },
                vec![],
                vec![text("解除")],
            ),
            button::button(&ButtonProps::default(), vec![], vec![text("適用")]),
        ],
    )
}

/// ドロワー版インスタンス（画面端から全高で開く）。
fn drawer_instance() -> Node {
    let ids = &DRAWER_IDS;
    let content = drawer::content(
        drawer::OpenState::Open,
        DrawerPlacement::Start,
        false,
        drawer::ContentIds {
            id: Some(ids.content),
            labelledby: Some(ids.title),
            describedby: Some(ids.description),
        },
        vec![],
        vec![
            drawer::title(Some(ids.title), vec![], vec![text("フィルタ")]),
            drawer::description(
                Some(ids.description),
                vec![],
                vec![text("条件を選んで商品を絞り込みます。")],
            ),
            filter_groups(ids),
            panel_actions(),
        ],
    );
    let positioner = drawer::positioner(
        drawer::OpenState::Open,
        DrawerPlacement::Start,
        vec![],
        vec![content],
    );
    let backdrop = drawer::backdrop(drawer::OpenState::Open, vec![], vec![]);
    let drawer_node = drawer::root(
        Size::Md,
        drawer::OpenState::Open,
        DrawerPlacement::Start,
        vec![],
        vec![backdrop, positioner],
    );
    // キャプションはオーバーレイ（`inset: 0`）が覆う枠の外に置く。
    div(
        vec![("class", "blocks-filter-overlay-panel-instance")],
        vec![
            p(vec![], vec![text("ドロワー版（画面端から開く）")]),
            div(
                vec![
                    ("class", "blocks-filter-overlay-panel-variant"),
                    ("data-blocks-filter-overlay-panel-variant", "drawer"),
                ],
                vec![toolbar(ids), product_grid(), drawer_node],
            ),
        ],
    )
}

/// ダイアログ版インスタンス（画面中央・幅制限で開く）。
fn dialog_instance() -> Node {
    let ids = &DIALOG_IDS;
    let content = dialog::content(
        dialog::OpenState::Open,
        dialog::DialogRole::Dialog,
        false,
        DialogContentIds {
            id: Some(ids.content),
            labelledby: Some(ids.title),
            describedby: Some(ids.description),
        },
        vec![],
        vec![
            dialog::title(Some(ids.title), vec![], vec![text("フィルタ")]),
            dialog::description(
                Some(ids.description),
                vec![],
                vec![text("条件を選んで商品を絞り込みます。")],
            ),
            dialog::body(vec![], vec![filter_groups(ids)]),
            dialog::footer(vec![], vec![panel_actions()]),
        ],
    );
    let positioner = dialog::positioner(dialog::OpenState::Open, vec![], vec![content]);
    let backdrop = dialog::backdrop(dialog::OpenState::Open, vec![], vec![]);
    let dialog_node = dialog::root(
        Size::Md,
        dialog::OpenState::Open,
        vec![("data-blocks-filter-overlay-panel-dialog-root", "")],
        vec![backdrop, positioner],
    );
    // キャプションはオーバーレイ（`inset: 0`）が覆う枠の外に置く。
    div(
        vec![("class", "blocks-filter-overlay-panel-instance")],
        vec![
            p(vec![], vec![text("ダイアログ版（中央に開く）")]),
            div(
                vec![
                    ("class", "blocks-filter-overlay-panel-variant"),
                    ("data-blocks-filter-overlay-panel-variant", "dialog"),
                ],
                vec![toolbar(ids), product_grid(), dialog_node],
            ),
        ],
    )
}

/// `filter-overlay-panel` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数。ドロワー版を先頭に、2 インスタンスを縦に並べる。
#[must_use]
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-filter-overlay-panel-layout")],
        vec![drawer_instance(), dialog_instance()],
    )
}
```

## 原案差分メモ

参照（私有カタログ上のドロワー形式 1 件・ダイアログ形式 1 件。出典の固有名・
ファイル名は記載しません）から取り込んだのは構造（領域の配置と部品構成）のみ
であり、次の点を独自に設計・変更しています。

- ドロワー・ダイアログともに開閉トリガーを持たず、既に開いた静的な初期状態の
  みを描きます（原案は開閉可能なインタラクティブなパネルですが、docs サイトは
  JS ハイドレーションを行わない設計のため）。横バーのフィルタボタンも通常の
  ボタンとして描き、トリガーとしての開閉配線は持ちません。
- `aria-modal` はいずれも `false` にしています（原案はモーダル相当）。静的な
  デモは閉じる機構を持たず、パネルの外側に横バー・商品一覧・説明・コードが
  あるため、支援技術が外側を無視しないよう表示の実態に合わせました。
- `drawer` には `body`/`footer` に相当する anatomy パートが無いため、フィルタ
  群・解除/適用ボタン列は独自のフックを付けた `div` で組んでいます（`dialog`
  側は既存の `body`/`footer` パートをそのまま使用）。
- `dialog::body` の既定 CSS（`max-height: 50vh; overflow-y: auto`）は打ち消し、
  パネル内部の `div` 側で `overflow-y: auto` を与えて縦スクロールを担わせて
  います。
- 商品一覧は商品カードの代わりに `skeleton` の占位を 6 件並べています（実際の
  商品データ・画像アセットには依存しません）。
- checkbox はすべてネイティブ `disabled` で固定し、既定の `opacity: 0.5` を
  打ち消して通常の checkbox と同じ見た目にしています。
- 並び替えメニューは無 JS のため `OpenState::Closed` の静的表示に固定して
  います（`page-heading-avatar` の三点メニューと同型の判断）。
- 文言（見出し・説明文・フィルタ項目・ボタンラベル）はすべて独自に書き直し
  ました。実在の商品・企業・PII は一切含みません。

関連情報: [Drawer](../themes/drawer.md) / [Dialog](../themes/dialog.md) /
[Button](../themes/button.md) / [Menu](../themes/menu.md) /
[Checkbox](../themes/checkbox.md) / [Fieldset](../themes/fieldset.md) /
[Skeleton](../themes/skeleton.md)
