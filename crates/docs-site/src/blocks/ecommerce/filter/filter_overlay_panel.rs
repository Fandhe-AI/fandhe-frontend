//! `filter-overlay-panel` block（イシュー #3048。親トラッキング #3024
//! 「Blocks Ecommerce 拡充ツリー」phase:5 配下）。Ecommerce / Filter
//! カテゴリ最初の block。「並び替えメニューとフィルタボタンの横バー、
//! その下の商品一覧、フィルタボタンで開くパネル（フィルタ群と適用・
//! 解除ボタン）」という EC の典型画面を、無 JS の静的な実例として掲示する。
//!
//! # 使用部品
//!
//! `drawer` / `dialog` / `button` / `menu` / `checkbox` / `fieldset` /
//! `skeleton` の 7 部品のみを合成する（[`BLOCK`] の `parts` に一致させる
//! 契約、`crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が
//! 検証する）。新しい UI 部品は追加しない。
//!
//! # 2 バリアント併記（ドロワー版・ダイアログ版）
//!
//! 集約元は画面端から出るドロワー版と中央に出るダイアログ版の 2 系統が
//! あり、両者の差分を読み取れるよう縦に併記する（取得手段・ファイル名・
//! 内部コンポーネント識別子は記載しない、
//! `docs/design/motion-reference-adoption-policy.md` §9 と同じ転記制限）。
//! `drawer`/`dialog` いずれも id はインスタンスごとに一意にする（id 重複・
//! 宙ぶらり aria 参照の回避、`blocks_contract.rs` 参照）。
//!
//! # 静的表示・`trigger`/`close_trigger` を置かない理由
//!
//! docs サイトは JS ハイドレーションを行わない設計（CLAUDE.md）のため、
//! 本 Demo はパネルが**既に開いた静的な初期状態**のみを描く
//! （`contact_dialog_form`/`game_ui_modal` と同型の判断）。
//! `drawer::trigger`/`dialog::trigger`・`close_trigger` 系はいずれも無 JS
//! 下では開閉を切り替えられず表示上の意味を持たないため、意図的に置かない。
//! フィルタボタン（横バー側）も `drawer::trigger` を使わず素の `button`
//! にする（無 JS では開閉できず `aria-expanded` が実挙動を約束して
//! しまうため）。開閉・フォーカストラップ・Escape 等の挙動は一切扱わない。
//!
//! # `aria-modal` を false にする理由
//!
//! 静的なデモは閉じる機構を持たず、パネルの外側に横バー・商品一覧・
//! 説明・コードがある。支援技術が外側を無視しないよう、表示の実態と
//! 一致させて `aria-modal` は false にする（`contact_dialog_form` と
//! 同じ判断）。
//!
//! # checkbox をネイティブ disabled にする理由
//!
//! `checkbox::hidden_input` は有効なネイティブ `<input>` であり、`disabled`
//! を渡さない構成では JS ハイドレーションなしでもラベルクリックで
//! `checked` がネイティブに切り替わってしまう一方、`control`/`indicator`
//! の見た目は SSR 時の `checked` 引数から固定生成され追従しないため、
//! 静的な初期状態のみという block 全体の設計方針に反する
//! （`form_layout_stacked` と同型の判断）。[`CheckboxProps`] の
//! `disabled: true` を全パーツへ共有し、`disabled_declarations()`（既定
//! `opacity: 0.5` + `cursor: not-allowed`）は [`LAYOUT_CSS`] で中和して
//! 通常の checkbox と同じ見た目に保つ。
//!
//! # menu を閉じた状態で固定する理由
//!
//! 開閉状態機械・実際のポップアップ表示はクライアント配線層（wasm-full）の
//! 責務であり、無 JS の docs サイトでは `OpenState::Closed` の静的表示の
//! みを描画する（`page_heading_avatar.rs::overflow_menu` と同型）。
//!
//! # `<form>` を使わない・送信処理を持たない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力せず、ボタンはすべて `button::button` の既定 `type="button"` の
//! まま用いる。並び替え・絞り込み・適用/解除の実処理は一切持たない静的な
//! 合成例である（`docs/policy/intentional-non-adoption.md` §3.25、UI
//! コンポーネント層はアプリケーションロジックを内包しない）。
//!
//! # 参照について
//!
//! 主参照・集約元はいずれも私有カタログ上の 1 件のみであり、取得手段・
//! ファイル名・内部コンポーネント識別子は記載しない。参照から取り込むのは
//! 構造（領域の配置と部品構成）のみで、文言・配色・装飾・アイコンは
//! 独自に書く。
//!
//! # `drawer` に `body`/`footer` パートが無いことへの対応
//!
//! [`fandhe_frontend_pre_styled_ui::drawer`] の anatomy は dialog と異なり
//! `body`/`footer` パートを持たない（`crates/pre-styled-ui/src/drawer.rs`
//! 冒頭 rustdoc「本イシューのスコープ外」節参照）。このためパネル内の
//! フィルタ群・適用/解除ボタン列は `data-blocks-filter-overlay-panel-*`
//! フック付きの素の `div` で組む。
//!
//! # `dialog::content`/`dialog::body` の高さ制約とスクロール
//!
//! [`fandhe_frontend_pre_styled_ui::dialog::body`] の既定 `max-height: 50vh`
//! はビューポート単位のため、本 Demo の固定サイズ枠
//! （`.blocks-filter-overlay-panel-variant`、`overflow: hidden`）内では
//! 意味を持たない。[`LAYOUT_CSS`] は `max-height: 50vh` の数値のみを
//! 打ち消し、`dialog::content` 側に `display: flex; flex-direction:
//! column; max-height: 100%`（positioner の実高さを上限とする）を与えた
//! うえで `dialog::body` を `flex: 1 1 auto; min-height: 0; overflow-y:
//! auto` に据え直す。画面高が低く全フィルタ群が収まらない場合は
//! `dialog::body` がスクロール領域として機能し、枠からのはみ出し・
//! `overflow: hidden` によるボタン到達不能（イシュー #3048 PR #3503
//! レビュー指摘）を防ぐ。`dialog::content` は既定 `padding` を持つため
//! `box-sizing: border-box` も明示し、`max-height: 100%` の算出が
//! padding を含めないまま端が欠ける事故（同 PR レビュー指摘）を防ぐ。
//!
//! # `drawer::content` 側のスクロール範囲限定
//!
//! [`fandhe_frontend_pre_styled_ui::drawer::content`] の既定
//! `overflow-y: auto` は `content` 全体（タイトル・フィルタ群・
//! 操作ボタン列すべて）にかかるため、内容超過時に操作ボタンごと
//! スクロールして到達しづらくなる（イシュー #3048 PR #3503 レビュー
//! 指摘）。`[data-blocks-filter-overlay-panel-body]`（フィルタ群）側に
//! `flex: 1 1 auto; min-height: 0; overflow-y: auto` を与え、
//! `drawer::content` の flex column 内でフィルタ群のみを可変・
//! スクロール領域とし、操作ボタン列は定位置のまま残す。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
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
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/filter-overlay-panel/",
    title: "filter-overlay-panel",
    category: BlockCategory::Filter,
    rust_source: "crates/docs-site/src/blocks/ecommerce/filter/filter_overlay_panel.rs",
    demo_class: "blocks-filter-overlay-panel",
    parts: &[
        Part {
            label: "Drawer",
            path: "/themes/drawer/",
        },
        Part {
            label: "Dialog",
            path: "/themes/dialog/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Menu",
            path: "/themes/menu/",
        },
        Part {
            label: "Checkbox",
            path: "/themes/checkbox/",
        },
        Part {
            label: "Fieldset",
            path: "/themes/fieldset/",
        },
        Part {
            label: "Skeleton",
            path: "/themes/skeleton/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `filter_overlay_panel` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節、`contact_dialog_form`/`form_layout_stacked`
/// と同型）。drawer/dialog の固定オーバーレイをデモ枠内へ収める中和規則と、
/// checkbox disabled の中和規則を持つ。
const LAYOUT_CSS: &str = "\
.blocks-filter-overlay-panel.blocks-demo {\n  overflow: visible;\n}\n\
.blocks-filter-overlay-panel-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n}\n\
.blocks-filter-overlay-panel-instance {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-filter-overlay-panel-variant {\n  position: relative;\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n  min-height: 32rem;\n  overflow: hidden;\n  border: 1px solid var(--fandhe-color-border);\n  border-radius: var(--fandhe-radius-lg);\n  padding: var(--fandhe-space-4);\n}\n\
.blocks-filter-overlay-panel-toolbar {\n  display: flex;\n  justify-content: space-between;\n  align-items: center;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-filter-overlay-panel-grid {\n  display: grid;\n  grid-template-columns: repeat(auto-fill, minmax(10rem, 1fr));\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-filter-overlay-panel-product-card {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n}\n\
[data-scope=\"skeleton\"][data-part=\"root\"][data-blocks-filter-overlay-panel-product-image] {\n  aspect-ratio: 1 / 1;\n  width: 100%;\n}\n\
[data-scope=\"skeleton\"][data-part=\"root\"][data-blocks-filter-overlay-panel-product-price] {\n  max-width: 50%;\n}\n\
.blocks-filter-overlay-panel [data-scope=\"drawer\"][data-part=\"backdrop\"], .blocks-filter-overlay-panel [data-scope=\"dialog\"][data-part=\"backdrop\"] {\n  position: absolute;\n  inset: 0;\n  z-index: auto;\n}\n\
.blocks-filter-overlay-panel [data-scope=\"drawer\"][data-part=\"positioner\"], .blocks-filter-overlay-panel [data-scope=\"dialog\"][data-part=\"positioner\"] {\n  position: absolute;\n  inset: 0;\n  z-index: auto;\n}\n\
.blocks-filter-overlay-panel [data-scope=\"dialog\"][data-part=\"positioner\"] {\n  display: flex;\n  align-items: center;\n  justify-content: center;\n  padding: var(--fandhe-space-6);\n}\n\
.blocks-filter-overlay-panel [data-scope=\"drawer\"][data-part=\"content\"] {\n  height: 100%;\n  max-width: 100%;\n  display: flex;\n  flex-direction: column;\n}\n\
.blocks-filter-overlay-panel [data-scope=\"dialog\"][data-part=\"content\"] {\n  box-sizing: border-box;\n  max-width: 24rem;\n  max-height: 100%;\n  display: flex;\n  flex-direction: column;\n}\n\
.blocks-filter-overlay-panel [data-scope=\"drawer\"] h2, .blocks-filter-overlay-panel [data-scope=\"dialog\"] h2 {\n  border-top: none;\n  padding-top: 0;\n  letter-spacing: normal;\n}\n\
.blocks-filter-overlay-panel [data-scope=\"dialog\"][data-part=\"body\"] {\n  flex: 1 1 auto;\n  min-height: 0;\n  max-height: none;\n  overflow-y: auto;\n}\n\
[data-blocks-filter-overlay-panel-body] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n  flex: 1 1 auto;\n  min-height: 0;\n  overflow-y: auto;\n}\n\
[data-blocks-filter-overlay-panel-footer] {\n  display: flex;\n  justify-content: flex-end;\n  gap: var(--fandhe-space-3);\n}\n\
[data-scope=\"checkbox\"][data-part=\"root\"][data-blocks-filter-overlay-panel-checkbox][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
@media (max-width: 47.99rem) {\n  .blocks-filter-overlay-panel [data-scope=\"drawer\"][data-part=\"content\"] {\n    --fandhe-drawer-size: 85%;\n  }\n  .blocks-filter-overlay-panel [data-scope=\"dialog\"][data-part=\"positioner\"] {\n    padding: var(--fandhe-space-3);\n  }\n  .blocks-filter-overlay-panel [data-scope=\"dialog\"][data-part=\"footer\"] {\n    flex-direction: column;\n    align-items: stretch;\n  }\n  [data-blocks-filter-overlay-panel-footer] {\n    flex-direction: column-reverse;\n  }\n  [data-blocks-filter-overlay-panel-footer] [data-scope=\"button\"] {\n    width: 100%;\n  }\n  .blocks-filter-overlay-panel-grid {\n    grid-template-columns: repeat(auto-fill, minmax(8rem, 1fr));\n  }\n}\n";

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;

    fn demo_html() -> String {
        render(&demo())
    }

    /// Demo が使用部品（drawer/dialog/button/menu/checkbox/fieldset/
    /// skeleton）の anatomy をすべて実際に出力していることを固定する
    /// （`contact_dialog_form` 先例と同型）。
    #[test]
    fn demo_composes_expected_parts() {
        let html = demo_html();
        for scope in [
            "data-scope=\"drawer\" data-part=\"backdrop\"",
            "data-scope=\"drawer\" data-part=\"positioner\"",
            "data-scope=\"drawer\" data-part=\"content\"",
            "data-scope=\"drawer\" data-part=\"title\"",
            "data-scope=\"dialog\" data-part=\"backdrop\"",
            "data-scope=\"dialog\" data-part=\"positioner\"",
            "data-scope=\"dialog\" data-part=\"content\"",
            "data-scope=\"dialog\" data-part=\"body\"",
            "data-scope=\"dialog\" data-part=\"footer\"",
            "data-scope=\"menu\" data-part=\"trigger\"",
            "data-scope=\"checkbox\" data-part=\"root\"",
            "data-scope=\"fieldset\" data-part=\"root\"",
            "data-scope=\"skeleton\" data-part=\"root\"",
            "data-scope=\"button\" data-part=\"root\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
    }

    /// `<form>` を出力せず、ボタンがすべて `type="button"` であることを
    /// 固定する（`crate::blocks` モジュール doc の不変条件）。
    #[test]
    fn demo_has_no_form_and_only_button_type_buttons() {
        let html = demo_html();
        assert!(!html.contains("<form"), "demo should never contain <form");
        for absent in ["type=\"submit\"", "form=\"", "action=", "src=\"data:"] {
            assert!(!html.contains(absent), "demo should never contain {absent}");
        }
        let count_open = html.matches("<button").count();
        let count_typed = html.matches("type=\"button\"").count();
        assert!(count_open > 0);
        assert!(count_typed >= count_open, "html={html}");
    }

    /// 静的な開状態・非モーダル（`aria-modal=false`）・menu 閉状態である
    /// ことを固定する（モジュール doc参照）。
    #[test]
    fn panels_are_open_static_and_non_modal() {
        let html = demo_html();
        assert_eq!(html.matches("aria-modal=\"false\"").count(), 2);
        assert!(html.contains("data-placement=\"start\""));
        assert_eq!(html.matches("aria-expanded=\"false\"").count(), 2);
    }

    /// checkbox がすべてネイティブ `disabled` で固定されていることを
    /// 固定する（モジュール doc「checkbox をネイティブ disabled にする
    /// 理由」節参照）。
    #[test]
    fn checkboxes_are_natively_disabled() {
        let html = demo_html();
        // 3 グループ × 2 インスタンス分の checkbox hidden input。
        assert_eq!(html.matches(" disabled=\"\"").count(), 14);
    }

    /// [`LAYOUT_CSS`] が固定オーバーレイの中和・checkbox disabled 中和・
    /// 狭幅ブレークポイントを含み、`<` を含まないことを固定する
    /// （REQ-1: `</style>` によるスタイル脱出を防ぐ）。
    #[test]
    fn layout_css_neutralizes_fixed_overlay_and_has_no_angle_bracket() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("position: absolute;\n  inset: 0;"));
        assert!(LAYOUT_CSS.contains(
            "[data-scope=\"checkbox\"][data-part=\"root\"][data-blocks-filter-overlay-panel-checkbox][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}"
        ));
        assert!(LAYOUT_CSS.contains("@media (max-width: 47.99rem)"));
    }

    /// 各版のキャプションがオーバーレイ（`inset: 0`）を持つ
    /// `position: relative` の枠の外（兄弟要素）にあり、覆われないことを
    /// 固定する。
    #[test]
    fn captions_are_outside_overlay_frame() {
        let html = demo_html();
        for (caption, kind) in [
            ("ドロワー版（画面端から開く）", "drawer"),
            ("ダイアログ版（中央に開く）", "dialog"),
        ] {
            let caption_at = html.find(caption).expect("caption present");
            let frame_at = html
                .find(&format!(
                    "data-blocks-filter-overlay-panel-variant=\"{kind}\""
                ))
                .expect("frame present");
            assert!(
                caption_at < frame_at,
                "{kind}: caption should precede frame"
            );
            let between = &html[caption_at..frame_at];
            assert!(
                between.contains("</p><div class=\"blocks-filter-overlay-panel-variant\""),
                "{kind}: caption should be a sibling before the frame: {between}"
            );
        }
        assert_eq!(
            html.matches("class=\"blocks-filter-overlay-panel-instance\"")
                .count(),
            2
        );
        assert!(
            LAYOUT_CSS.contains(".blocks-filter-overlay-panel-variant {\n  position: relative;")
        );
    }

    /// 狭幅時にダイアログ版の footer が縦方向・stretch になり、解除・適用
    /// ボタン列が縮み幅にならず全幅の縦積みになることを固定する。
    #[test]
    fn dialog_footer_stretches_actions_on_narrow_viewport() {
        let html = demo_html();
        assert!(html.contains("data-scope=\"dialog\" data-part=\"footer\""));
        let media = &LAYOUT_CSS[LAYOUT_CSS
            .find("@media (max-width: 47.99rem)")
            .expect("media query present")..];
        assert!(media.contains(
            ".blocks-filter-overlay-panel [data-scope=\"dialog\"][data-part=\"footer\"] {\n    flex-direction: column;\n    align-items: stretch;\n  }"
        ));
        assert!(media.contains(
            "[data-blocks-filter-overlay-panel-footer] [data-scope=\"button\"] {\n    width: 100%;\n  }"
        ));
    }

    /// id の重複がないことを固定する（drawer/dialog 併記による id 衝突の
    /// 回帰ガード）。
    #[test]
    fn ids_have_no_duplicates() {
        let html = demo_html();
        let mut ids: Vec<&str> = Vec::new();
        for chunk in html.split("id=\"").skip(1) {
            if let Some(end) = chunk.find('"') {
                ids.push(&chunk[..end]);
            }
        }
        let mut sorted = ids.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(ids.len(), sorted.len(), "id が重複している: {ids:?}");
    }

    #[test]
    fn demo_is_deterministic() {
        assert_eq!(demo_html(), demo_html());
    }
}
