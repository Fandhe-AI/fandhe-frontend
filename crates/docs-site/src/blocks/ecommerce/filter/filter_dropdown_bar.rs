//! `filter-dropdown-bar` block（イシュー #3046。親トラッキング #3024
//! 「Blocks EC」配下、Ecommerce / Filter カテゴリ）。主参照
//! R0815（中央見出し + 4 フィルタ）・集約元 R0816（左見出し + 3 フィルタ +
//! 件数バッジ）を対応表 ID とする合成例。`_/blocks-intake/` の対応ファイルは
//! 本イシュー着手時点で本 worktree に存在しないため、原稿・本コメントには
//! 対応表 ID のみを記す（`settings_export_data.rs` と同じ扱い）。
//!
//! # 構成（見出し + 並び替えメニュー + フィルタ群の横バー）
//!
//! 見出し（`heading`）の下に、並び替え `menu`（常時閉）と商品フィルタ
//! （`popover`）を横一列に並べたバーを 3 variant 分並記する。
//!
//! - `centered`（R0815）: 中央寄せ見出し + 4 フィルタ（カテゴリ / 色 /
//!   サイズ / 素材）。「色」フィルタの popover を開いた状態にし、
//!   checkbox 5 件のうち 1 件をチェック済みにする。件数バッジは付けない。
//! - `left`（R0816）: 左寄せ見出し + 3 フィルタ（カテゴリ / 色 / サイズ）。
//!   「サイズ」フィルタの popover を開いた状態にし、checkbox 一覧のうち
//!   2 件をチェック済みにする。「サイズ」トリガーへ選択件数バッジ
//!   （`badge`、値は checkbox のチェック数と一致させる）を付ける。
//! - `narrow`（R0816 の構成を `max-inline-size: 22rem` のシェルで再現）:
//!   `@container` によりフィルタ群を非表示にし、代わりに「フィルタ」
//!   ボタン 1 個 + 件数バッジを表示する（「狭い幅ではフィルタ群を 1 つの
//!   ボタンに畳む」要件の実演）。
//!
//! # 使用部品
//!
//! `heading` / `menu` / `popover` / `checkbox` / `button` / `badge` の
//! 6 部品を合成する（[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//! 新しい UI 部品は追加しない。
//!
//! # 静的表示（無 JS、トリガーはすべて `disabled: true`）
//!
//! docs サイトは JS ハイドレーションを行わないため、Demo は開閉状態を
//! 固定描画する（[`super::super::super::application::auth::
//! auth_dropdown_panel`] と同じ判断）。並び替えメニュー・各フィルタの
//! トリガー・畳みボタンはいずれも押しても何も起きないため
//! `disabled: true`（ネイティブ `disabled` + `aria-disabled="true"`）にし、
//! [`LAYOUT_CSS`] の `[data-disabled]` 複合セレクタで中和して通常状態と
//! 同じ見た目に保つ。
//!
//! # checkbox をネイティブ `disabled` にする理由
//!
//! `checkbox::hidden_input` は有効なネイティブ `<input>` であり、`disabled`
//! を渡さない構成では無 JS でもクリックで `checked` がネイティブに切り替わり、
//! SSR 固定の `control`/`indicator` 見た目と食い違う
//! （`settings_export_data.rs` と同型の判断）。全 checkbox へ
//! `disabled: true` を共有し、[`LAYOUT_CSS`] で中和しない（無効な操作である
//! ことを視覚的にも示す）。
//!
//! # 3 variant を並べる理由と `@container` の閾値の根拠
//!
//! Demo 枠の幅はビューポート幅と一致しないため `@container`（コンテナ
//! クエリ）で判定する。`[data-blocks-filter-dropdown-bar-shell]` へ
//! `container-type: inline-size` を宣言し、閾値を `30rem`
//! （`max-width: 29.99rem`）とする。docs サイトの `.docs-content` は最大幅
//! 46rem のため、`centered`/`left`（`max-inline-size` 未指定）はこの条件を
//! 満たさず通常のフィルタ群表示のまま、`narrow`（`max-inline-size: 22rem`
//! 固定）は確実にこの条件を満たして畳み表示へ切り替わる
//! （[`super::super::super::application::auth::auth_dropdown_panel`] と
//! 同型のインスタンス分離パターン）。
//!
//! # id と ARIA の一意性
//!
//! `id`/`aria-controls`/`aria-labelledby` はいずれも
//! `blocks-filter-dropdown-bar-{sort-trigger,sort-menu,<filter>-trigger,
//! <filter>-panel}-{variant}` の形で variant ごとに一意にする
//! （`crates/docs-site/tests/blocks_contract.rs::
//! demo_output_has_no_dangling_aria_references_or_duplicate_ids` 参照）。
//! 閉じたフィルタの `popover::trigger` は `aria-controls` に `None` を渡し、
//! 存在しないパネルを参照しない（未描画のパネル id への参照切れ防止）。
//!
//! # 高さ予約
//!
//! `.blocks-demo` は横方向のみ `overflow-x: auto` で縦方向は自然な高さに
//! 確定するため、絶対配置される開いた popover の `content`/`positioner`
//! がシェル外へはみ出すと縦方向にクリップされる
//! （`auth_dropdown_panel` と同じ教訓）。
//! `[data-blocks-filter-dropdown-bar-shell]` へ `min-block-size: 20rem` を
//! 宣言して高さを確保する。
//!
//! # 見出しレベル
//!
//! `heading::heading(HeadingLevel::H3, ..)` を使う。Demo 内に `h1`/`h2` を
//! 出さない（ページの `h1`・`## Demo` 見出しとの重複を避けるため、
//! `settings_export_data.rs` と同じ判断）。
//!
//! # `<form>` を持たない・フィルタ処理を持たない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である。すべてのトリガー・ボタンは `button`/`popover::trigger`/
//! `menu::trigger` の既定 `type="button"` のまま用い、`href="#"`・
//! `data:` URI・`mailto:` は使わない。文言はすべて架空のカテゴリ名・
//! フィルタ値であり、実在の商品・企業・PII は含まない。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, p, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{badge, BadgeProps, BadgeVariant};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::checkbox::{self, CheckboxProps, CheckedState};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::menu;
use fandhe_frontend_pre_styled_ui::popover::{self, OpenState};
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// 並び替えメニュー（常時閉、`disabled: true`）。`variant` は id の suffix
/// （モジュール doc「id と ARIA の一意性」節）。
fn sort_menu(variant: &str) -> Node {
    let trigger_id = format!("blocks-filter-dropdown-bar-sort-trigger-{variant}");
    let content_id = format!("blocks-filter-dropdown-bar-sort-menu-{variant}");
    let trigger = menu::trigger(
        OpenState::Closed,
        true,
        Some(content_id.as_str()),
        vec![
            ("id", trigger_id.as_str()),
            ("data-blocks-filter-dropdown-bar-sort-trigger", ""),
        ],
        vec![text("並び替え")],
    );
    let content = menu::content(
        OpenState::Closed,
        Some(content_id.as_str()),
        Some(trigger_id.as_str()),
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
    let positioner = menu::positioner(OpenState::Closed, vec![], vec![content]);
    menu::root(
        Size::Sm,
        OpenState::Closed,
        vec![],
        vec![trigger, positioner],
    )
}

/// 選択済みチェックボックス 1 件を組み立てる。`disabled: true` の理由は
/// モジュール doc「checkbox をネイティブ `disabled` にする理由」節参照。
fn filter_checkbox(
    name: &str,
    value: &'static str,
    label_text: &'static str,
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
        Size::Sm,
        ColorPalette::Accent,
        &props,
        vec![("data-blocks-filter-dropdown-bar-checkbox", "")],
        vec![
            checkbox::hidden_input(&props, name, value, vec![]),
            checkbox::control(
                &props,
                vec![],
                vec![checkbox::indicator(&props, vec![], vec![])],
            ),
            checkbox::label(&props, vec![], vec![text(label_text)]),
        ],
    )
}

/// 開いたフィルタ（`popover`、`trigger` は `disabled: true`）。`badge_count`
/// が `Some` のときトリガーへ件数バッジを添える。`variant`/`filter_key` は
/// id の suffix・接頭辞。
#[allow(clippy::too_many_arguments)]
fn open_filter(
    variant: &str,
    filter_key: &str,
    label_text: &'static str,
    options: &[(&'static str, &'static str, bool)],
    badge_count: Option<u8>,
) -> Node {
    let trigger_id = format!("blocks-filter-dropdown-bar-{filter_key}-trigger-{variant}");
    let panel_id = format!("blocks-filter-dropdown-bar-{filter_key}-panel-{variant}");

    let mut trigger_children = vec![text(label_text)];
    let mut trigger_attrs = vec![
        ("id", trigger_id.as_str()),
        ("data-blocks-filter-dropdown-bar-trigger", ""),
    ];
    let aria_label_value;
    if let Some(count) = badge_count {
        aria_label_value = format!("{label_text}（{count} 件選択中）");
        trigger_attrs.push(("aria-label", aria_label_value.as_str()));
        trigger_children.push(badge(
            &BadgeProps {
                variant: BadgeVariant::Subtle,
                ..BadgeProps::default()
            },
            vec![],
            vec![text(count.to_string())],
        ));
    }

    let trigger = popover::trigger(
        OpenState::Open,
        true,
        Some(panel_id.as_str()),
        trigger_attrs,
        trigger_children,
    );

    let checkboxes: Vec<Node> = options
        .iter()
        .map(|(value, label_text, checked)| {
            filter_checkbox(
                &format!("filter-{filter_key}-{variant}"),
                value,
                label_text,
                *checked,
            )
        })
        .collect();

    let content = popover::content(
        OpenState::Open,
        Some(panel_id.as_str()),
        Some(trigger_id.as_str()),
        None,
        vec![("data-blocks-filter-dropdown-bar-panel", "")],
        checkboxes,
    );
    let positioner = popover::positioner(
        OpenState::Open,
        vec![("data-blocks-filter-dropdown-bar-positioner", "")],
        vec![content],
    );

    popover::root(
        OpenState::Open,
        vec![("data-blocks-filter-dropdown-bar-popover", "")],
        vec![trigger, positioner],
    )
}

/// 閉じたフィルタ（`popover`、`trigger` は `disabled: true`）。パネルを
/// 描画しないため `aria-controls` に `None` を渡し、参照切れを作らない
/// （モジュール doc「id と ARIA の一意性」節参照）。
fn closed_filter(variant: &str, filter_key: &str, label_text: &'static str) -> Node {
    let trigger_id = format!("blocks-filter-dropdown-bar-{filter_key}-trigger-{variant}");
    let trigger = popover::trigger(
        OpenState::Closed,
        true,
        None,
        vec![
            ("id", trigger_id.as_str()),
            ("data-blocks-filter-dropdown-bar-trigger", ""),
        ],
        vec![text(label_text)],
    );
    popover::root(
        OpenState::Closed,
        vec![("data-blocks-filter-dropdown-bar-popover", "")],
        vec![trigger],
    )
}

/// 狭幅で畳んだ「フィルタ」ボタン（`disabled: true`）。件数バッジを子に
/// 置く（「サイズ」フィルタのチェック数と一致させる、モジュール doc
/// 「構成」節の `narrow` variant 参照）。
fn collapsed_filter_button(variant: &str, count: u8) -> Node {
    button::button(
        &ButtonProps {
            variant: ButtonVariant::Outline,
            disabled: true,
            ..ButtonProps::default()
        },
        vec![
            ("data-blocks-filter-dropdown-bar-collapsed-trigger", ""),
            ("data-blocks-filter-dropdown-bar-variant-scoped", variant),
        ],
        vec![
            text("フィルタ"),
            badge(
                &BadgeProps {
                    variant: BadgeVariant::Subtle,
                    ..BadgeProps::default()
                },
                vec![],
                vec![text(count.to_string())],
            ),
        ],
    )
}

/// caption（並記された各 variant の見出し）。
fn caption(label: &'static str) -> Node {
    p(
        vec![("class", "blocks-filter-dropdown-bar-caption")],
        vec![text(label)],
    )
}

/// `centered`（R0815）: 中央寄せ見出し + 4 フィルタ、「色」を開状態にする。
fn centered_bar() -> Node {
    let variant = "centered";
    let heading_node = heading(
        HeadingLevel::H3,
        &HeadingProps {
            size: HeadingSize::Lg,
            weight: HeadingWeight::Bold,
        },
        vec![],
        vec![text("すべての商品")],
    );
    let bar = div(
        vec![("data-blocks-filter-dropdown-bar-bar", "")],
        vec![
            sort_menu(variant),
            div(
                vec![("data-blocks-filter-dropdown-bar-filters", "")],
                vec![
                    closed_filter(variant, "category", "カテゴリ"),
                    open_filter(
                        variant,
                        "color",
                        "色",
                        &[
                            ("black", "ブラック", false),
                            ("white", "ホワイト", true),
                            ("navy", "ネイビー", false),
                            ("beige", "ベージュ", false),
                            ("green", "グリーン", false),
                        ],
                        None,
                    ),
                    closed_filter(variant, "size", "サイズ"),
                    closed_filter(variant, "material", "素材"),
                ],
            ),
        ],
    );
    div(
        vec![
            ("data-blocks-filter-dropdown-bar-shell", ""),
            ("data-blocks-filter-dropdown-bar-variant", variant),
        ],
        vec![heading_node, bar],
    )
}

/// `left`/`narrow` 共通の「サイズ」フィルタ選択肢（`value`, `label`,
/// `checked`）。両 variant の選択状態を単一の正から導出し、畳みボタンの
/// 件数バッジ（[`narrow_bar`]）が `left` の実チェック数（`open_filter` の
/// checkbox 一覧）と食い違わないようにする（Codex P2 指摘の是正）。
const SIZE_OPTIONS: &[(&str, &str, bool)] = &[
    ("s", "S", true),
    ("m", "M", true),
    ("l", "L", false),
    ("xl", "XL", false),
];

/// [`SIZE_OPTIONS`] のうちチェック済みの件数。
fn size_options_checked_count() -> u8 {
    SIZE_OPTIONS
        .iter()
        .filter(|(_, _, checked)| *checked)
        .count() as u8
}

/// `left`（R0816）: 左寄せ見出し + 3 フィルタ、「サイズ」を開状態にし
/// [`SIZE_OPTIONS`] のチェック数を件数バッジに添える。
fn left_bar() -> Node {
    let variant = "left";
    let heading_node = heading(
        HeadingLevel::H3,
        &HeadingProps {
            size: HeadingSize::Lg,
            weight: HeadingWeight::Bold,
        },
        vec![],
        vec![text("アウター")],
    );
    let bar = div(
        vec![("data-blocks-filter-dropdown-bar-bar", "")],
        vec![
            sort_menu(variant),
            div(
                vec![("data-blocks-filter-dropdown-bar-filters", "")],
                vec![
                    closed_filter(variant, "category", "カテゴリ"),
                    closed_filter(variant, "color", "色"),
                    open_filter(
                        variant,
                        "size",
                        "サイズ",
                        SIZE_OPTIONS,
                        Some(size_options_checked_count()),
                    ),
                ],
            ),
        ],
    );
    div(
        vec![
            ("data-blocks-filter-dropdown-bar-shell", ""),
            ("data-blocks-filter-dropdown-bar-variant", variant),
        ],
        vec![heading_node, bar],
    )
}

/// `narrow`: `left` と同じ構成を `max-inline-size: 22rem` のシェルで
/// 再現し、`@container` でフィルタ群を畳みボタンへ切り替える（モジュール
/// doc「`@container` の閾値の根拠」節参照）。畳みボタンの件数バッジは
/// [`SIZE_OPTIONS`] のチェック数（`left` と同一の正）から導出し、実際の
/// 選択状態と常に一致させる。
fn narrow_bar() -> Node {
    let variant = "narrow";
    let heading_node = heading(
        HeadingLevel::H3,
        &HeadingProps {
            size: HeadingSize::Lg,
            weight: HeadingWeight::Bold,
        },
        vec![],
        vec![text("アウター")],
    );
    let bar = div(
        vec![("data-blocks-filter-dropdown-bar-bar", "")],
        vec![
            sort_menu(variant),
            div(
                vec![
                    ("data-blocks-filter-dropdown-bar-filters", ""),
                    ("data-blocks-filter-dropdown-bar-frame", "narrow"),
                ],
                vec![
                    closed_filter(variant, "category", "カテゴリ"),
                    closed_filter(variant, "color", "色"),
                    closed_filter(variant, "size", "サイズ"),
                ],
            ),
            collapsed_filter_button(variant, size_options_checked_count()),
        ],
    );
    div(
        vec![
            ("data-blocks-filter-dropdown-bar-shell", ""),
            ("data-blocks-filter-dropdown-bar-variant", variant),
            ("data-blocks-filter-dropdown-bar-frame", "narrow"),
        ],
        vec![heading_node, bar],
    )
}

/// `filter-dropdown-bar` の Demo 本体。3 variant を caption 付きで縦に
/// 並記する純関数。ルート class は `demo_class`（`BLOCK` 参照）とは別名
/// にする（二重適用防止、`auth_dropdown_panel.rs` と同じ判断）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-filter-dropdown-bar-stack")],
        vec![
            caption("中央見出し + 4 フィルタ（「色」を展開）"),
            centered_bar(),
            caption("左寄せ見出し + 3 フィルタ + 件数バッジ（「サイズ」を展開）"),
            left_bar(),
            caption("狭幅（< 30rem、フィルタ群を「フィルタ」ボタン 1 個へ畳む）"),
            narrow_bar(),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/filter-dropdown-bar/",
    title: "filter-dropdown-bar",
    category: BlockCategory::Filter,
    rust_source: "crates/docs-site/src/blocks/ecommerce/filter/filter_dropdown_bar.rs",
    demo_class: "blocks-filter-dropdown-bar",
    parts: &[
        Part {
            label: "Heading",
            path: "/themes/heading/",
        },
        Part {
            label: "Menu",
            path: "/themes/menu/",
        },
        Part {
            label: "Popover",
            path: "/themes/popover/",
        },
        Part {
            label: "Checkbox",
            path: "/themes/checkbox/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Badge",
            path: "/themes/badge/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `filter_dropdown_bar` 固有のレイアウト規則（`crate::blocks`
/// モジュール doc「CSS の置き場」節）。`--fandhe-*` トークンのみ使用し、
/// 生値は幅・rem 指定のみに限る。
///
/// # シェルの高さ確保・popover recipe を上書きする詳細度
///
/// モジュール doc「高さ予約」節参照。popover/menu/button recipe の disabled
/// state（`[data-scope="..."][data-part="trigger"または"root"][data-disabled]`、
/// 詳細度 `(0,3,0)`）を上書きするため、`data-scope`/`data-part` を含む複合
/// セレクタへ block 固有属性を追加して常に 1 個以上上回る詳細度にする
/// （`auth_dropdown_panel.rs` と同じ判断）。畳みボタン
/// （[`collapsed_filter_button`]）は button recipe（`data-part="root"`）の
/// ため、同じ理由で `[data-scope="button"][data-part="root"]` を併記する。
///
/// # 開いた popover の配置
///
/// popover recipe 既定は `positioner { left: 0; top: 100%; }` のため、右端
/// に近いフィルタのトリガーからだと水平オーバーフローし得る。`right: 0;
/// left: auto;` へ上書きしトリガー右揃えにする。`content` は
/// `inline-size: 14rem` の固定幅で checkbox 一覧を縦並びにする。
/// positioner の上書きセレクタは `[data-blocks-filter-dropdown-bar-positioner]`
/// （[`open_filter`] が付与）を併記して本 block 固有の popover インスタンス
/// へ限定する。block 固有属性なしの `[data-scope="popover"][data-part=
/// "positioner"]` は `assets/blocks.css` へ全 block 共通で連結されるため、
/// 他 block の popover 配置（既定の左揃え）まで右揃えへ書き換えてしまう
/// （Codex P1 指摘の是正）。
///
/// # `@container` の閾値（`narrow` variant 限定）
///
/// モジュール doc「3 variant を並べる理由と `@container` の閾値の根拠」節
/// 参照。`30rem` 未満でフィルタ群を非表示にし畳みボタンを表示する。この
/// 非表示規則は `[data-blocks-filter-dropdown-bar-frame="narrow"]` を
/// 持つ variant へ限定する。畳みボタン（[`collapsed_filter_button`]）を
/// 持たない `centered`/`left` variant まで適用すると、Demo 枠が 30rem
/// 未満になった際にフィルタ群が代替操作なしに消える（Codex P1 指摘の
/// 是正）。限定の付け方は、コンテナ自身（`[data-blocks-filter-dropdown-
/// bar-shell]`）を祖先セレクタに含めず、`frame="narrow"` を
/// `[data-blocks-filter-dropdown-bar-filters]`（畳み対象の子要素）自身へ
/// 直接付与して判定する（コンテナクエリは対象要素の祖先コンテナを評価する
/// 仕様のため、コンテナ自身を祖先セレクタの一部にする記法は紛らわしく、
/// 子要素自身に属性を持たせたほうが「このコンテナがこの条件を満たす」と
/// 「この子要素を対象にする」を分離でき明確、という判断。Codex P1
/// 指摘の是正）。
const LAYOUT_CSS: &str = "\
.blocks-filter-dropdown-bar-stack {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n}\n\
.blocks-filter-dropdown-bar-caption {\n  margin: 0;\n  font-size: var(--fandhe-font-font-size-sm, 0.875rem);\n  color: var(--fandhe-color-fg-muted);\n}\n\
[data-blocks-filter-dropdown-bar-shell] {\n  container-type: inline-size;\n  container-name: blocks-filter-dropdown-bar;\n  min-block-size: 20rem;\n  padding: var(--fandhe-space-4);\n  border: 1px solid var(--fandhe-color-border);\n  border-radius: var(--fandhe-radius-md);\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
[data-blocks-filter-dropdown-bar-shell][data-blocks-filter-dropdown-bar-frame=\"narrow\"] {\n  max-inline-size: 22rem;\n}\n\
[data-blocks-filter-dropdown-bar-shell][data-blocks-filter-dropdown-bar-variant=\"centered\"] [data-scope=\"heading\"] {\n  text-align: center;\n}\n\
[data-blocks-filter-dropdown-bar-bar] {\n  position: relative;\n  display: flex;\n  align-items: center;\n  justify-content: space-between;\n  flex-wrap: wrap;\n  gap: var(--fandhe-space-3);\n  padding-block-end: var(--fandhe-space-3);\n  border-bottom: 1px solid var(--fandhe-color-border);\n}\n\
[data-blocks-filter-dropdown-bar-filters] {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n  flex-wrap: wrap;\n}\n\
[data-scope=\"menu\"][data-part=\"trigger\"][data-blocks-filter-dropdown-bar-sort-trigger][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
[data-scope=\"popover\"][data-part=\"trigger\"][data-blocks-filter-dropdown-bar-trigger][data-disabled] {\n  opacity: 1;\n  cursor: default;\n  display: inline-flex;\n  align-items: center;\n  gap: var(--fandhe-space-1);\n}\n\
[data-scope=\"button\"][data-part=\"root\"][data-blocks-filter-dropdown-bar-collapsed-trigger][data-disabled] {\n  opacity: 1;\n  cursor: default;\n  display: none;\n}\n\
[data-scope=\"popover\"][data-part=\"positioner\"][data-blocks-filter-dropdown-bar-positioner] {\n  left: auto;\n  right: 0;\n}\n\
[data-scope=\"popover\"][data-part=\"content\"][data-blocks-filter-dropdown-bar-panel] {\n  inline-size: 14rem;\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n}\n\
@container blocks-filter-dropdown-bar (max-width: 29.99rem) {\n  \
[data-blocks-filter-dropdown-bar-filters][data-blocks-filter-dropdown-bar-frame=\"narrow\"] {\n    display: none;\n  }\n  \
[data-scope=\"button\"][data-part=\"root\"][data-blocks-filter-dropdown-bar-collapsed-trigger][data-disabled] {\n    display: inline-flex;\n    align-items: center;\n    gap: var(--fandhe-space-1);\n  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// 6 部品の `data-scope` が揃い、`type="button"` があり、`<form>`・
    /// `href="#"`・`data:` src・`mailto:` を持たないこと。
    #[test]
    fn demo_composes_expected_parts_and_has_no_form() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"heading\"",
            "data-scope=\"menu\"",
            "data-scope=\"popover\"",
            "data-scope=\"checkbox\"",
            "data-scope=\"button\"",
            "data-scope=\"badge\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert!(html.contains(r#"type="button""#));
        assert!(!html.contains("<form"));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("src=\"data:"));
        assert!(!html.contains("mailto:"));
    }

    /// 3 variant が並記され、caption が 3 件あること。
    #[test]
    fn demo_renders_three_variants_with_captions() {
        let html = render(&demo());
        for variant in ["centered", "left", "narrow"] {
            assert!(
                html.contains(&format!(
                    "data-blocks-filter-dropdown-bar-variant=\"{variant}\""
                )),
                "variant {variant} should render"
            );
        }
        assert_eq!(
            html.matches("blocks-filter-dropdown-bar-caption").count(),
            3
        );
        assert_eq!(
            html.matches("data-blocks-filter-dropdown-bar-shell")
                .count(),
            3
        );
    }

    /// `narrow` インスタンスのみ frame 属性を持つこと（shell 自身と、
    /// コンテナクエリの対象である `filters` 子要素の 2 箇所、
    /// `centered`/`left` には 0 件）。
    #[test]
    fn only_narrow_instance_has_the_frame_attribute() {
        let html = render(&demo());
        assert_eq!(
            html.matches("data-blocks-filter-dropdown-bar-frame=\"narrow\"")
                .count(),
            2
        );
    }

    /// 開いた popover がちょうど 2 件（`centered` の色、`left` のサイズ）、
    /// `menu` はすべて閉じていること。
    #[test]
    fn exactly_two_filters_are_open_and_all_menus_are_closed() {
        let html = render(&demo());
        assert_eq!(
            html.matches(r#"data-scope="popover" data-part="root" data-state="open""#)
                .count(),
            2
        );
        assert_eq!(
            html.matches(r#"data-scope="menu" data-part="root" data-state="open""#)
                .count(),
            0
        );
        assert_eq!(
            html.matches(r#"data-scope="menu" data-part="root" data-state="closed""#)
                .count(),
            3
        );
    }

    /// `left` variant のチェック済み checkbox 数が badge の「2」と
    /// 一致すること。
    #[test]
    fn left_variant_checked_checkbox_count_matches_badge() {
        let html = render(&demo());
        // centered（色・white 1 件）+ left（サイズ・s/m 2 件）で計 3 件が
        // チェック済みになる（`hidden_input` は 1 checkbox につき `checked`
        // 存在属性 1 個）。
        assert_eq!(
            html.matches(r#" checked"#).count(),
            3,
            "white(centered) + s,m(left) = 3"
        );
        // `left` のサイズ件数バッジはチェック数と同じ「2」を表示する。
        assert!(html.contains(r#"data-scope="badge""#));
        assert!(html.contains(">2<"));
    }

    /// variant ごとに `aria-controls`/`id`/`aria-labelledby` の参照先が
    /// 存在し一意であること。
    #[test]
    fn panel_references_are_unique_and_consistent() {
        let html = render(&demo());
        for (variant, filter_key) in [("centered", "color"), ("left", "size")] {
            let panel_id = format!("blocks-filter-dropdown-bar-{filter_key}-panel-{variant}");
            let trigger_id = format!("blocks-filter-dropdown-bar-{filter_key}-trigger-{variant}");
            assert_eq!(
                html.matches(&format!(r#"aria-controls="{panel_id}""#))
                    .count(),
                1
            );
            assert_eq!(html.matches(&format!(r#"id="{panel_id}""#)).count(), 1);
            assert_eq!(
                html.matches(&format!(r#"aria-labelledby="{trigger_id}""#))
                    .count(),
                1
            );
            assert_eq!(html.matches(&format!(r#"id="{trigger_id}""#)).count(), 1);
        }
    }

    /// 閉じたフィルタが `aria-controls` を持たないこと（参照切れ防止）。
    #[test]
    fn closed_filters_have_no_aria_controls() {
        let html = render(&demo());
        // centered の category/size/material、left/narrow の category/color
        // (/size for narrow) はいずれも closed_filter（`aria-controls` に
        // `None` を渡す）由来のため `aria-controls` を出力しない。本 Demo で
        // `aria-controls` を持つのは常時 `Some` を渡す並び替え menu トリガー
        // 3 件（centered/left/narrow）+ 開いた popover 2 件（centered の色・
        // left のサイズ）の計 5 件のみであることを件数で確認する。
        assert_eq!(html.matches("aria-controls").count(), 5);
    }

    /// [`LAYOUT_CSS`] がコンテナクエリ・高さ予約・閾値を満たすこと。
    #[test]
    fn layout_css_has_container_query_and_guards() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("container-type: inline-size;"));
        assert!(LAYOUT_CSS.contains("min-block-size: 20rem;"));
        assert!(LAYOUT_CSS.contains("@container blocks-filter-dropdown-bar (max-width: 29.99rem)"));
        assert!(LAYOUT_CSS.contains("opacity: 1;"));
        assert!(LAYOUT_CSS.contains("display: none;"));
    }

    /// `@container` 内のフィルタ群非表示セレクタが、コンテナ自身
    /// （`[data-blocks-filter-dropdown-bar-shell]`）を祖先セレクタに
    /// 含めず、畳み対象の子要素（`[data-blocks-filter-dropdown-bar-filters]`）
    /// 自身の `frame="narrow"` 属性のみで判定すること（Codex P1 指摘の
    /// 是正。祖先コンテナ自身をセレクタへ含める記法は、コンテナクエリが
    /// 祖先コンテナを評価する仕様と紛らわしいため採らない）。
    #[test]
    fn narrow_hide_selector_targets_filters_child_directly() {
        assert!(LAYOUT_CSS.contains(
            "[data-blocks-filter-dropdown-bar-filters][data-blocks-filter-dropdown-bar-frame=\"narrow\"] {\n    display: none;\n  }"
        ));
        assert!(!LAYOUT_CSS.contains(
            "[data-blocks-filter-dropdown-bar-shell][data-blocks-filter-dropdown-bar-frame=\"narrow\"] [data-blocks-filter-dropdown-bar-filters]"
        ));
    }

    /// ルート class（`demo_class` とは別名）が [`demo`] の出力へ実際に
    /// 現れること。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-filter-dropdown-bar-stack\""));
        assert_ne!(super::BLOCK.demo_class, "blocks-filter-dropdown-bar-stack");
    }

    /// 呼び出しごとに同一の `Node` を返す決定的な関数であること。
    #[test]
    fn demo_is_deterministic() {
        assert_eq!(render(&demo()), render(&demo()));
    }
}
