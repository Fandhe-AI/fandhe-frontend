//! `cart-dialog` block（イシュー #3025。Ecommerce / Cart カテゴリ）。
//! カートを開くボタンと、画面中央に出るダイアログ（タイトル・閉じる
//! ボタン・数量選択/削除付き商品行・淡い面の集計・右寄せの次へボタン）を
//! 無 JS の静的開状態で示す。主参照は対応表 ID R1251（集約元 R0680）。
//! `_/blocks-intake/` の対応ファイルは本イシュー着手時点で本 worktree に
//! 存在しないため、原稿・本コメントには対応表 ID のみを記し、レイアウトは
//! issue のレイアウト仕様に従って独自に組む
//! （`cart_two_column_summary.rs` と同じ扱い）。
//!
//! # 使用部品
//!
//! `dialog` / `button` / `image` / `text` / `native-select` / `separator` /
//! `data-list` の 7 部品を合成する（[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//! 新しい UI 部品は追加しない。
//!
//! # 開く・閉じる・削除・レジに進むは無 JS で no-op のため `disabled`
//!
//! docs サイトは JS ハイドレーションを行わない設計（CLAUDE.md）のため、
//! [`fandhe_frontend_pre_styled_ui::dialog::trigger`]/[`close_trigger`] を
//! 押しても開閉は切り替わらない。本 block は issue のレイアウト仕様で
//! 「開くボタン」「閉じるボタン」の明示を求めているため、
//! `contact_dialog_form.rs`（trigger/close-trigger を置かない判断）ではなく
//! `store_nav_centered_logo.rs` の判断を採り、両ボタンを置いたうえで
//! ネイティブ `disabled` 属性 + `data-disabled` でフォーカス・クリック
//! 不能を明示する（押せない操作をキーボード・支援技術利用者に実行可能な
//! ものとして提示しない）。
//!
//! 同じ理由で「削除」「レジに進む」ボタンも `disabled` にする。送信先・
//! 削除処理を持たない静的デモで押下可能なまま残すと無反応になり、
//! 開く・閉じるボタンに適用した上記の判断と矛盾するため
//! （[`fandhe_frontend_pre_styled_ui::native_select`] の数量選択を
//! `disabled` にする判断は次節を参照）。
//!
//! # 数量 `select` も `disabled`（集計再計算を持たないため）
//!
//! 数量 `select` は操作可能なままだと、無 JS の静的デモでは選択を変えても
//! [`SUMMARY_ROWS`] の小計・合計が追随せず表示が矛盾する。
//! `cart_two_column_summary.rs::qty_control` が入荷待ち商品で
//! `FieldProps::disabled = true` にする判断と同型で、本 block は全行を
//! `disabled` にして集計との矛盾を避ける。
//!
//! # `aria-modal` を false にする理由
//!
//! 静的なデモは閉じる機構を実際には持たず、ダイアログの外側に説明・
//! コード・ナビゲーションがある。支援技術が外側を無視しないよう、表示の
//! 実態と一致させて `aria-modal` は false にする
//! （`contact_dialog_form.rs`/`game_ui_modal.rs` と同じ判断）。
//!
//! # `<form>` を使わない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない静的表示のみで、送信処理・送信先は一切持たない。数量選択は
//! `select` の初期選択値のみを示す静的表示で、削除・次へ進むの各ボタンは
//! `button::button` の既定 `type="button"` のまま用いる。リンクは置かず、
//! `href="#"` も使わない。
//!
//! # 数量 `select` の id・アクセシブルネーム
//!
//! 商品行ごとに `FieldProps::id` を一意にし（`.../qty-<n>`）、
//! `aria-label` も商品名を含めて行ごとに区別する
//! （`cart_two_column_summary.rs` と同型の判断）。実際に出力される
//! `<select>` の `id` は `fandhe_frontend_headless_ui::field` の派生規則に
//! より `"{id}-control"` になる。
//!
//! # ダミー素材について
//!
//! 商品名・属性・価格は本ファイル内の架空データ（実在のブランド・商品・
//! PII を含まない）で持つ。商品画像はビルド時生成の同梱 SVG
//! （[`dummy_assets::PRODUCT_SRC`]）を使う（外部 URL・`data:` URI は
//! 使わない）。`alt` は空文字列（`""`）とし、商品名テキストが隣接して
//! 可視のためアクセシブルネームは商品名テキストが担う
//! （`cart_two_column_summary.rs` と同型の判断）。
//!
//! # 集計は淡い背景面の `data_list`
//!
//! 小計・送料・合計の 3 行を `data_list::root`（横並び）で組み、合計行は
//! `data-blocks-cart-dialog-total` で罫線 + 太字強調する
//! （`cart_two_column_summary.rs::summary_totals` と同型のパターン）。
//! 集計面全体は `[data-blocks-cart-dialog-summary]` へ淡い背景色・角丸を
//! 当てる。
//!
//! # 狭幅ではダイアログが全幅に近づく（`@container`）
//!
//! Demo 枠の幅はビューポート幅と一致しないため、`@container`
//! （コンテナクエリ）で判定する（`cart_two_column_summary.rs` と同型の
//! パターン）。[`LAYOUT_CSS`] のラッパー `.blocks-cart-dialog-stack` へ
//! `container-type: inline-size` を宣言し、コンテナ幅が `36rem` 未満の
//! とき `positioner` の padding を縮め、`content` の `max-width` を全幅へ
//! 近づけ、商品行コントロールと次へボタンを縦積み・全幅にする。
//!
//! # 固定オーバーレイのデモ枠内中和
//!
//! `dialog::backdrop`/`positioner` は本来 `position: fixed; inset: 0` の
//! ビューポート全体オーバーレイだが、Blocks の掲示は `.blocks-demo` 枠内へ
//! 収める必要がある。本 block スコープ（`.blocks-cart-dialog` 配下）に
//! 限定した属性セレクタで中和する（`contact_dialog_form.rs` と同型。
//! `positioner`（実コンテンツ側）の高さで共通の位置指定祖先の高さを決め、
//! `backdrop` をそれに追随させる構成も同型）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::button::{button, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::data_list::{self, DataListOrientation, DataListProps};
use fandhe_frontend_pre_styled_ui::dialog::{self, ContentIds, DialogRole, OpenState};
use fandhe_frontend_pre_styled_ui::image::{image, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::native_select::{
    native_select, FieldIds, FieldProps, NativeSelectProps,
};
use fandhe_frontend_pre_styled_ui::recipe::Size;
use fandhe_frontend_pre_styled_ui::separator::{separator, SeparatorProps};
use fandhe_frontend_pre_styled_ui::text::{
    self as styled_text, TextProps, TextSize, TextVariant, TextWeight,
};

/// [`dialog::content`] の `id`（[`dialog::trigger`] の `controls` と対）。
const CONTENT_ID: &str = "blocks-cart-dialog-content";

/// [`dialog::title`] の `id`（[`dialog::content`] の `labelledby` と対）。
const TITLE_ID: &str = "blocks-cart-dialog-title";

/// 架空の商品行データ（商品名, 属性表示, 価格表示, 初期選択数量）。
/// 実在のブランド・商品・PII は含まない。
const CART_ITEMS: &[(&str, &str, &str, u8)] = &[
    (
        "エルゴノミック メッシュチェア",
        "カラー: グレー / サイズ: M",
        "¥24,800",
        1,
    ),
    (
        "ノイズキャンセリング ヘッドホン",
        "カラー: ブラック",
        "¥18,200",
        1,
    ),
];

/// 集計行（ラベル, 値）。最終行（合計）だけ [`summary`] 側で強調用の
/// `data-*` を追加する。
const SUMMARY_ROWS: &[(&str, &str)] = &[("小計", "¥43,000"), ("送料", "¥600"), ("合計", "¥43,600")];

/// 指定した初期選択数量 `selected` の 1〜5 の `<option>` 列を組み立てる
/// （`cart_two_column_summary.rs::qty_options` と同型）。
fn qty_options(selected: u8) -> Vec<Node> {
    (1..=5u8)
        .map(|n| {
            let mut attrs = vec![("value", n.to_string())];
            if n == selected {
                attrs.push(("selected", String::new()));
            }
            el(
                "option",
                attrs.iter().map(|(k, v)| (*k, v.as_str())).collect(),
                vec![text(n.to_string())],
            )
        })
        .collect()
}

/// 商品行 1 件（サムネイル + 名称/属性 + 価格/数量選択/削除）。`index` は
/// 0 始まりで、数量 `select` の一意な `id` の派生に使う
/// （`cart_two_column_summary.rs::item_row` と同型）。
fn item_row(index: usize, name: &str, attrs: &str, price: &str, qty: u8) -> Node {
    let field_id = format!("blocks-cart-dialog-qty-{}", index + 1);
    let qty_aria_label = format!("{name} の数量");
    let field = FieldProps {
        id: &field_id,
        ids: FieldIds::default(),
        disabled: true,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: false,
    };
    div(
        vec![("class", "blocks-cart-dialog-item")],
        vec![
            image(
                &ImageProps {
                    shape: ImageShape::Rounded,
                    ..ImageProps::new(dummy_assets::PRODUCT_SRC, "")
                },
                vec![("data-blocks-cart-dialog-thumb", "")],
            ),
            div(
                vec![("class", "blocks-cart-dialog-item-body")],
                vec![
                    styled_text::text(
                        &TextProps {
                            weight: TextWeight::Medium,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text(name)],
                    ),
                    styled_text::text(
                        &TextProps {
                            variant: TextVariant::Muted,
                            size: TextSize::Sm,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text(attrs)],
                    ),
                ],
            ),
            div(
                vec![("class", "blocks-cart-dialog-item-controls")],
                vec![
                    styled_text::text(
                        &TextProps {
                            weight: TextWeight::Medium,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text(price)],
                    ),
                    native_select(
                        &NativeSelectProps::default(),
                        &field,
                        vec![("aria-label", qty_aria_label.as_str())],
                        qty_options(qty),
                    ),
                    button(
                        &ButtonProps {
                            variant: ButtonVariant::Ghost,
                            size: Size::Sm,
                            ..ButtonProps::default()
                        },
                        vec![("disabled", ""), ("data-disabled", "")],
                        vec![text("削除")],
                    ),
                ],
            ),
        ],
    )
}

/// 商品行と行間の `separator` を束ねたリスト。
fn item_list() -> Node {
    let mut children = Vec::new();
    for (index, (name, attrs, price, qty)) in CART_ITEMS.iter().enumerate() {
        if index > 0 {
            children.push(separator(&SeparatorProps::default(), vec![]));
        }
        children.push(item_row(index, name, attrs, price, *qty));
    }
    div(vec![("class", "blocks-cart-dialog-items")], children)
}

/// 集計行 1 件（`data_list::item` + `item-label` + `item-value`）。最終行
/// （合計）だけ強調用の `data-blocks-cart-dialog-total` を付与する
/// （`cart_two_column_summary.rs::summary_row` と同型）。
fn summary_row(label: &str, value: &str, emphasize: bool) -> Node {
    let attrs = if emphasize {
        vec![("data-blocks-cart-dialog-total", "")]
    } else {
        vec![]
    };
    data_list::item(
        attrs,
        vec![
            data_list::item_label(vec![], vec![text(label)]),
            data_list::item_value(vec![], vec![text(value)]),
        ],
    )
}

/// 淡い背景面の集計（小計・送料・合計）。
fn summary() -> Node {
    let last = SUMMARY_ROWS.len() - 1;
    let rows = SUMMARY_ROWS
        .iter()
        .enumerate()
        .map(|(i, (label, value))| summary_row(label, value, i == last))
        .collect();
    div(
        vec![("data-blocks-cart-dialog-summary", "")],
        vec![data_list::root(
            DataListProps {
                orientation: DataListOrientation::Horizontal,
                ..DataListProps::default()
            },
            vec![],
            rows,
        )],
    )
}

/// `cart-dialog` の Demo 本体。カートを開くボタン（右寄せ）+ 中央に開いた
/// 静的なダイアログで構成する。呼び出しごとに同一の `Node` を返す純関数。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-cart-dialog-stack")],
        vec![
            div(
                vec![("class", "blocks-cart-dialog-bar")],
                vec![dialog::trigger(
                    OpenState::Open,
                    Some(CONTENT_ID),
                    vec![
                        ("disabled", ""),
                        ("data-disabled", ""),
                        ("data-blocks-cart-dialog-open", ""),
                    ],
                    vec![text("カート（2）")],
                )],
            ),
            dialog::root(
                Size::Lg,
                OpenState::Open,
                vec![("data-blocks-cart-dialog-root", "")],
                vec![
                    dialog::backdrop(OpenState::Open, vec![], vec![]),
                    dialog::positioner(
                        OpenState::Open,
                        vec![],
                        vec![dialog::content(
                            OpenState::Open,
                            DialogRole::Dialog,
                            false,
                            ContentIds {
                                id: Some(CONTENT_ID),
                                labelledby: Some(TITLE_ID),
                                describedby: None,
                            },
                            vec![],
                            vec![
                                dialog::close_trigger(
                                    vec![
                                        ("aria-label", "閉じる"),
                                        ("disabled", ""),
                                        ("data-disabled", ""),
                                    ],
                                    vec![text("×")],
                                ),
                                dialog::title(
                                    Some(TITLE_ID),
                                    vec![],
                                    vec![text("ショッピングカート")],
                                ),
                                dialog::body(
                                    vec![("data-blocks-cart-dialog-body", "")],
                                    vec![item_list()],
                                ),
                                summary(),
                                dialog::footer(
                                    vec![],
                                    vec![button(
                                        &ButtonProps::default(),
                                        vec![
                                            ("data-blocks-cart-dialog-next", ""),
                                            ("disabled", ""),
                                            ("data-disabled", ""),
                                        ],
                                        vec![text("レジに進む")],
                                    )],
                                ),
                            ],
                        )],
                    ),
                ],
            ),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/cart-dialog/",
    title: "cart-dialog",
    category: BlockCategory::Cart,
    rust_source: "crates/docs-site/src/blocks/ecommerce/cart/cart_dialog.rs",
    demo_class: "blocks-cart-dialog",
    parts: &[
        Part {
            label: "Dialog",
            path: "/themes/dialog/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Image",
            path: "/themes/image/",
        },
        Part {
            label: "Text",
            path: "/themes/text/",
        },
        Part {
            label: "Native Select",
            path: "/themes/native-select/",
        },
        Part {
            label: "Separator",
            path: "/themes/separator/",
        },
        Part {
            label: "Data List",
            path: "/themes/data-list/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `cart_dialog` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS` doc
/// 「block 固有 CSS の置き場」節と同型、`contact_dialog_form.rs` の
/// 固定オーバーレイ中和パターンを踏襲）。
const LAYOUT_CSS: &str = "\
.blocks-cart-dialog.blocks-demo {\n  overflow: visible;\n}\n\
.blocks-cart-dialog-stack {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n  container-type: inline-size;\n  container-name: blocks-cart-dialog;\n}\n\
.blocks-cart-dialog-bar {\n  display: flex;\n  justify-content: flex-end;\n}\n\
[data-blocks-cart-dialog-root] {\n  position: relative;\n}\n\
.blocks-cart-dialog [data-scope=\"dialog\"] h2 {\n  border-top: none;\n  padding-top: 0;\n  letter-spacing: normal;\n}\n\
.blocks-cart-dialog [data-scope=\"dialog\"][data-part=\"backdrop\"] {\n  position: absolute;\n  inset: 0;\n  z-index: auto;\n  border-radius: var(--fandhe-radius-lg);\n  background: var(--fandhe-color-bg-subtle);\n}\n\
.blocks-cart-dialog [data-scope=\"dialog\"][data-part=\"positioner\"] {\n  position: relative;\n  inset: auto;\n  z-index: auto;\n  width: 100%;\n  min-height: 24rem;\n  display: flex;\n  align-items: center;\n  justify-content: center;\n  padding: var(--fandhe-space-6);\n}\n\
.blocks-cart-dialog [data-scope=\"dialog\"][data-part=\"body\"] {\n  max-height: none;\n  overflow: visible;\n}\n\
.blocks-cart-dialog-items {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-cart-dialog-item {\n  display: flex;\n  gap: var(--fandhe-space-4);\n  align-items: flex-start;\n}\n\
[data-blocks-cart-dialog-thumb] {\n  width: 5rem;\n  height: 5rem;\n  flex: none;\n}\n\
.blocks-cart-dialog-item-body {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1);\n  flex: 1 1 auto;\n  min-width: 0;\n}\n\
.blocks-cart-dialog-item-controls {\n  display: flex;\n  flex-direction: column;\n  align-items: flex-end;\n  gap: var(--fandhe-space-2);\n}\n\
[data-blocks-cart-dialog-summary] {\n  background: var(--fandhe-color-bg-subtle);\n  border-radius: var(--fandhe-radius-md);\n  padding: var(--fandhe-space-4);\n  margin-block-start: var(--fandhe-space-4);\n}\n\
[data-blocks-cart-dialog-summary] [data-blocks-cart-dialog-total] {\n  border-top: 1px solid var(--fandhe-color-border);\n  padding-top: var(--fandhe-space-2);\n  font-weight: var(--fandhe-font-weight-bold, 700);\n}\n\
@container blocks-cart-dialog (max-width: 36rem) {\n  \
.blocks-cart-dialog [data-scope=\"dialog\"][data-part=\"positioner\"] {\n    padding: var(--fandhe-space-2);\n  }\n  \
.blocks-cart-dialog [data-scope=\"dialog\"][data-part=\"content\"] {\n    max-width: 100%;\n  }\n  \
.blocks-cart-dialog-item {\n    flex-wrap: wrap;\n  }\n  \
.blocks-cart-dialog-item-controls {\n    align-items: flex-start;\n    width: 100%;\n  }\n  \
[data-blocks-cart-dialog-next] {\n    width: 100%;\n  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::{demo, CART_ITEMS, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    fn demo_html() -> String {
        render(&demo())
    }

    #[test]
    fn demo_composes_expected_parts() {
        let html = demo_html();
        for scope in [
            "data-scope=\"dialog\" data-part=\"trigger\"",
            "data-scope=\"dialog\" data-part=\"backdrop\"",
            "data-scope=\"dialog\" data-part=\"positioner\"",
            "data-scope=\"dialog\" data-part=\"content\"",
            "data-scope=\"dialog\" data-part=\"title\"",
            "data-scope=\"dialog\" data-part=\"close-trigger\"",
            "data-scope=\"dialog\" data-part=\"body\"",
            "data-scope=\"dialog\" data-part=\"footer\"",
            "data-scope=\"image\"",
            "data-scope=\"text\"",
            "data-scope=\"field\"",
            "data-scope=\"separator\"",
            "data-scope=\"data-list\"",
            "data-scope=\"button\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
        assert_eq!(html.matches("<select").count(), 2);
        assert_eq!(html.matches("<option").count(), 10);
        assert_eq!(html.matches("selected=\"\"").count(), 2);
        assert_eq!(html.matches("data-scope=\"separator\"").count(), 1);
        // 削除ボタン 2 + 次へボタン 1 = 3（trigger/close-trigger は dialog スコープ）。
        assert_eq!(html.matches("data-scope=\"button\"").count(), 3);
    }

    #[test]
    fn no_form_submit_or_dead_links() {
        let html = demo_html();
        assert!(!html.contains("<form"));
        assert!(!html.contains("type=\"submit\""));
        assert!(!html.contains("src=\"data:"));
        assert!(!html.contains("<script"));
        assert!(!html.contains("href=\"#\""));
        assert!(html.contains("../../assets/blocks-demo-product.svg"));
    }

    #[test]
    fn buttons_are_type_button() {
        let html = demo_html();
        let button_count = html.matches("<button").count();
        let type_button_count = html.matches("type=\"button\"").count();
        assert_eq!(button_count, type_button_count);
        // trigger + close-trigger + 削除 2 + 次へ = 5。
        assert_eq!(button_count, 5);
    }

    #[test]
    fn all_interactive_controls_are_disabled_noop() {
        let html = demo_html();
        assert_eq!(
            html.matches("aria-controls=\"blocks-cart-dialog-content\"")
                .count(),
            1
        );
        assert_eq!(html.matches("id=\"blocks-cart-dialog-content\"").count(), 1);
        // trigger + close-trigger + 数量 select 2 + 削除 2 + レジに進む 1 = 7。
        // いずれも送信先・集計再計算を持たない静的デモのため、操作可能な
        // まま無反応にしないよう全て disabled にする（モジュール doc参照）。
        assert_eq!(html.matches(" disabled=\"\"").count(), 7);
    }

    #[test]
    fn dialog_is_open_static_and_non_modal() {
        let html = demo_html();
        assert!(html.contains("aria-modal=\"false\""));
        assert!(html.contains("data-state=\"open\""));
        assert!(!html.contains(" hidden>"));
        assert!(!html.contains(" hidden "));
    }

    #[test]
    fn select_ids_are_unique_and_labelled() {
        let html = demo_html();
        for n in 1..=2 {
            let needle = format!("id=\"blocks-cart-dialog-qty-{n}-control\"");
            assert_eq!(
                html.matches(needle.as_str()).count(),
                1,
                "expected exactly one {needle}"
            );
        }
        for (name, ..) in CART_ITEMS {
            let needle = format!("aria-label=\"{name} の数量\"");
            assert_eq!(
                html.matches(needle.as_str()).count(),
                1,
                "expected exactly one {needle}"
            );
        }
    }

    #[test]
    fn layout_css_is_safe_and_goes_full_width_on_narrow_container() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("container-type: inline-size;"));
        assert!(LAYOUT_CSS.contains("@container blocks-cart-dialog (max-width: 36rem)"));
        assert!(LAYOUT_CSS.contains("position: relative;\n  inset: auto;"));
        assert!(LAYOUT_CSS.contains("max-height: none;"));
    }
}
