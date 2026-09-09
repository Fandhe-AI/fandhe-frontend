//! Overlay / Disclosure 系部品ページの原稿データ（イシュー #946、親 #928
//! Phase 4）。
//!
//! # 役割・呼び出し文脈
//!
//! [`crate::component_page::COMPONENT_SPECS`] レジストリから参照される
//! `ComponentPageSpec` 定数群を保持する専用モジュール。本モジュール自体は
//! 生成物へ直接寄与しない（`component_page::render_component_page` が
//! `spec_for` 経由で読み取り、Demo〔[`crate::showcase`]〕・Anatomy・
//! `data-*` 属性表・CSS 変数表（いずれも機械導出）と合成して 6 節ページを
//! 組み立てる）。
//!
//! 対象は accordion・action-bar・button-group・collapsible・dialog・
//! drawer・floating-panel・hover-card・menu・menubar・navigation-menu・
//! popover・tabs・toast・toggle-tip・toolbar・tooltip・tour の 18 部品
//! （トリガー起点のオーバーレイ、項目開閉のディスクロージャ系、または
//! role="group" の静的グループ化系。toolbar はイシュー #991、menubar は
//! イシュー #992、navigation-menu はイシュー #993、collapsible はイシュー
//! #1683、button-group はイシュー #2060（headless anatomy は #2059）で
//! 追加、いずれも `showcase.rs` の Demo 登録込み）。`toggle`/`toggle-group`
//! はショーケース CSS 未登録により Demo を持たないため（`crates/docs-site/src/showcase.rs`
//! を変更しないという #946 時点の受け入れ条件 4 の制約。#991/#992 の
//! Phase 8 には同制約は適用されない）、本モジュールには含めず
//! `site/themes/` の Markdown 側で完結させる（計画 §4.5 参照）。
//!
//! menubar のみ [`KeyRow`] を空にしていない（イシュー #992 実装計画で
//! 確定済みの判断）。他 14 部品はいずれも「フォーカストラップ・Escape
//! 閉鎖・キーボードナビゲーションは JS ランタイム側の責務としてスコープ
//! 外」（実 DOM のキー配線が未実装）という理由で空のままだが、menubar は
//! 「開いている Menu を跨いだ左右移動」が主題のため、実装済みの
//! `MenubarAction` variant（`crates/headless-ui/src/menubar.rs` の
//! `decode_action`）と 1:1 対応する行のみを記載し、各行の説明に「wasm 層
//! 実装」の注記を付けて実 DOM キー配線が本クレートのスコープ外である旨を
//! 明示する（`docs/design/docs-site-component-pages.md` の一次情報規約に
//! 従い、`decode_action` が受理するアクション名に対応しない架空のキー
//! 割り当ては書かない）。
//!
//! # 一次情報の所在（受け入れ条件 2 の裏付け、創作の禁止）
//!
//! 各定数の Features/Arguments/Accessibility は以下のソースからのみ導出
//! する（架空の引数・キー操作・ARIA 属性は書かない、`ComponentPageSpec`
//! フィールド 1 つでも根拠が取れない場合は空配列のまま残し節を省略させる、
//! `component_page.rs` の「節の省略規則」参照）。
//!
//! - Features: `crates/headless-ui/src/<mod>.rs` と
//!   `crates/pre-styled-ui/src/<mod>.rs` のモジュール doc（`//!`）
//! - Arguments: `crates/pre-styled-ui/src/<mod>.rs` の `pub fn root`/
//!   `pub fn tabs`/`pub fn group` 等のシグネチャ（`size`/`state`/
//!   `placement`/`status`/`palette`/`stage`/`delays` など型付き引数のみ。
//!   `attrs`/`children` は全部品共通の定型引数のため表には含めない）
//! - Accessibility: `crates/headless-ui/src/<mod>.rs` の `#[test]` が
//!   固定する実出力の `aria-*`/`role` アサーションのみ（キーボード操作は
//!   いずれの対象部品もモジュール doc で「フォーカストラップ・Escape
//!   閉鎖・キーボードナビゲーションは JS ランタイム側の責務としてスコープ
//!   外」と明記されており、確定したキー割り当てを本クレートのソースから
//!   裏付けられないため [`crate::component_page::KeyRow`] 表はいずれも
//!   空のまま省略する。フォーカスリング等スタイル層のみの挙動は
//!   Accessibility 節の対象外）
//!
//! # `Examples` 節を持たない理由（[`DIALOG`]/[`ACCORDION`]/[`COLLAPSIBLE`]/[`MENU`]/[`TOOLTIP`] を除く）
//!
//! `docs/design/docs-site-component-pages.md` §7 は `Examples` を任意の節と
//! 定めており、当初 PR（#946）では 13 定数すべて `examples: &[]` としていた
//! （節は自動的に省略される）。[`DIALOG`] はイシュー #1691 で alert-dialog
//! 構成（イシュー #1690）の掲示のため `Examples` 節（`ex_alert_dialog`）を
//! 追加した。[`ACCORDION`] はイシュー #2026 で shadcn/ui 突合の結果
//! multiple 開閉・トリガーへの icon/description 合成の掲示のため、
//! `Examples` 節（`ex_accordion_multiple`/`ex_accordion_trigger_composition`）
//! を追加した。[`COLLAPSIBLE`] はイシュー #2029 で shadcn/ui（Base UI）突合の
//! File Tree Example に相当する既存 API のみの合成デモ
//! （`ex_collapsible_nested_tree`）を追加した。[`MENU`] はイシュー #2033
//! で shadcn/ui 突合の合成パターン（グループ+ラベル+ショートカット・
//! checkbox/radio 項目、サブメニュー、inset/destructive 項目）を 3 つの
//! Examples（`ex_menu_group_checkable_shortcut`/`ex_menu_submenu`/
//! `ex_menu_inset_and_danger`）として追加した。[`TOOLTIP`] はイシュー #2041
//! で shadcn/ui の With Keyboard Shortcut Example に相当する、`content` の
//! children へテキストと [`kbd`] を並べる既存 API のみの合成デモ
//! （`ex_tooltip_with_kbd`）を追加した。他部品のバリエーション軸
//! （`Size`/`ColorPalette`/`ToastStatus` 等）への Examples 追加はレビュー
//! 負荷を抑えるためのフォローアップ課題として引き続き PR 本文に残す。
//!
//! # セキュリティ不変条件（REQ-1）
//!
//! 本モジュールはリテラル `&'static str` のみで [`crate::component_page::ArgRow`]/
//! [`crate::component_page::AriaRow`]/[`crate::component_page::KeyRow`] を
//! 構築し、`raw_html()` や HTML 文字列の直接組み立てを一切行わない。
//! 実際のエスケープは `component_page.rs` 側の `table`/`td`/`text` ノード
//! 木経由で `render()` が行う（`features_and_table_cells_escape_xss_payloads`
//! が既存フィクスチャで固定済み）。

use fandhe_frontend_core::{div, el, p, small, span, strong, text, Node};
use fandhe_frontend_pre_styled_ui::{
    accordion::{self, AccordionProps},
    avatar::{self, AvatarProps, ImageStatus},
    button::{button, ButtonProps, ButtonVariant},
    collapsible,
    dialog::{self, ContentIds, DialogRole},
    drawer::{self, DrawerPlacement},
    field::{self, FieldIds, FieldOrientation, FieldProps, FieldRootProps},
    hover_card::{self, HoverCardDelays},
    input::{self, InputProps},
    kbd::{kbd, KbdProps, KbdVariant},
    menu, menubar, navigation_menu, popover,
    text::{text as styled_text, TextProps, TextSize, TextWeight},
    toast::{self, ToastPlacement, ToastStatus},
    tooltip, ColorPalette, OpenState, Size,
};

use crate::component_page::{ArgRow, AriaRow, ComponentPageSpec, ExampleEntry, KeyRow};

/// `/themes/accordion/`（Interactive カテゴリ）。
///
/// 一次情報: `crates/pre-styled-ui/src/accordion.rs`（モジュール doc・
/// `root` シグネチャ）、`crates/headless-ui/src/accordion.rs`（`aria_expanded`/
/// `aria_controls`/`role="region"` の実出力テスト）。
pub const ACCORDION: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "高々 1 項目が開く single モード（Accordion）と、複数項目が同時に開く multiple モード（MultiAccordion）の 2 状態機械を提供する。",
        "開いている項目の item-trigger / item-indicator を data-state=\"open\" に連動してハイライトする。",
        "size variant（Xs/Sm/Md/Lg/Xl）を root へ付与し、item-trigger / item-content の padding を切り替える。",
        "item-trigger はキーボード操作時のみのフォーカスリング（:focus-visible）を持つ。",
        "開閉状態（data-state）は呼び出し側が SSR/SSG のビルド時に渡した値がそのまま出力される。JS ゼロ SSG（クライアント側 JavaScript を読み込まない構成）での挙動・ネイティブ details/summary への代替パターンは「JS ゼロ SSG での利用ガイド」（/guides/no-js-ssg/）を参照。",
    ],
    arguments: &[ArgRow {
        name: "size",
        kind: "Size",
        default: "Size::Md",
        description: "root へ付与するサイズ variant（Xs/Sm/Md/Lg/Xl）。item-trigger/item-content の padding を切り替える。",
    }],
    examples: &[
        ExampleEntry {
            title: "複数項目を同時に開く（multiple モード）",
            description: "shadcn/ui 突合（イシュー #2026）で確認した合成パターンです。実運用では MultiAccordion（fandhe_frontend_headless_ui::state::MultiSelect を埋め込んだ状態機械）が複数項目の同時開閉を管理します。この掲示は SSR 静的マークアップのため、2 項目とも data-state=\"open\" を固定表示しています。",
            render: ex_accordion_multiple,
        },
        ExampleEntry {
            title: "トリガーにアイコンと説明文を組み合わせる",
            description: "item_trigger の children は自由合成のため、追加引数なしでアイコン + タイトル + 説明文の組み合わせを表現できます（イシュー #2026）。item-trigger 直下は item_indicator と合わせて 2 要素に保ち、既存 CSS（space-between レイアウト）をそのまま活かしています。",
            render: ex_accordion_trigger_composition,
        },
    ],
    keyboard: &[],
    aria: &[
        AriaRow {
            attribute: "aria-expanded",
            description: "item-trigger に付与。項目の開閉状態（open で true）を表す。",
        },
        AriaRow {
            attribute: "aria-controls",
            description: "item-trigger に付与。controls が Some のとき対応する item-content の id を指す。",
        },
        AriaRow {
            attribute: "role=\"region\"",
            description: "item-content に付与。labelled_by が Some のとき aria-labelledby とセットで付与される。",
        },
    ],
    demo: None,
};

/// [`ACCORDION`] の Examples 節「複数項目を同時に開く（multiple モード）」
/// レンダラ（イシュー #2026）。
///
/// SSR 静的マークアップのため 2 項目とも `OpenState::Open` を固定表示する
/// （wasm 層が管理する実際の複数選択状態遷移はこの Demo の対象外）。
/// [`crate::showcase::accordion_section`] と同じページに描画されるため、
/// id は `showcase-acc-*` と衝突しない `showcase-multi-acc-*` を使う。
fn ex_accordion_multiple() -> Node {
    let props = AccordionProps::default();
    let make_item =
        |value: &'static str, idx: u32, question: &'static str, answer: &'static str| {
            let trigger_id = format!("showcase-multi-acc-trigger-{idx}");
            let content_id = format!("showcase-multi-acc-content-{idx}");
            accordion::item(
                OpenState::Open,
                false,
                &props,
                vec![],
                vec![
                    el(
                        "h3",
                        vec![],
                        vec![accordion::item_trigger(
                            OpenState::Open,
                            false,
                            &props,
                            value,
                            Some(trigger_id.as_str()),
                            Some(content_id.as_str()),
                            vec![],
                            vec![
                                text(question),
                                accordion::item_indicator(
                                    OpenState::Open,
                                    false,
                                    &props,
                                    vec![],
                                    vec![text("▾")],
                                ),
                            ],
                        )],
                    ),
                    accordion::item_content(
                        OpenState::Open,
                        false,
                        &props,
                        Some(content_id.as_str()),
                        Some(trigger_id.as_str()),
                        vec![],
                        vec![text(answer)],
                    ),
                ],
            )
        };
    accordion::root(
        Size::Md,
        &props,
        vec![],
        vec![
            make_item(
                "multi-1",
                1,
                "料金プランは変更できますか？",
                "いつでもアップグレード・ダウングレードできます。",
            ),
            make_item(
                "multi-2",
                2,
                "無料トライアルはありますか？",
                "14 日間の無料トライアルを提供しています。",
            ),
        ],
    )
}

/// [`ACCORDION`] の Examples 節「トリガーにアイコンと説明文を組み合わせる」
/// レンダラ（イシュー #2026）。
///
/// `item_trigger` の `children` を `[wrapper, item_indicator]` の 2 要素に
/// 保つことで既存 CSS（`justify-content: space-between`）をそのまま活かす。
/// `wrapper`（`span`）にアイコン用装飾 `span`（`aria-hidden="true"`）と
/// ラベル用ブロック（`strong` でタイトル + `small` で説明文、色は既存の
/// `--fandhe-color-fg-muted` トークンのみ使用）を内包する。
fn ex_accordion_trigger_composition() -> Node {
    let props = AccordionProps::default();
    let trigger_id = "showcase-acc-composition-trigger";
    let content_id = "showcase-acc-composition-content";
    accordion::root(
        Size::Md,
        &props,
        vec![],
        vec![accordion::item(
            OpenState::Open,
            false,
            &props,
            vec![],
            vec![
                el(
                    "h3",
                    vec![],
                    vec![accordion::item_trigger(
                        OpenState::Open,
                        false,
                        &props,
                        "composition-1",
                        Some(trigger_id),
                        Some(content_id),
                        vec![],
                        vec![
                            span(
                                vec![(
                                    "style",
                                    "display:flex;align-items:center;gap:var(--fandhe-space-2);",
                                )],
                                vec![
                                    span(vec![("aria-hidden", "true")], vec![text("🔔")]),
                                    span(
                                        vec![("style", "display:flex;flex-direction:column;")],
                                        vec![
                                            strong(vec![], vec![text("通知設定")]),
                                            small(
                                                vec![(
                                                    "style",
                                                    "color: var(--fandhe-color-fg-muted);",
                                                )],
                                                vec![text(
                                                    "メール・アプリ通知の受信可否を管理します。",
                                                )],
                                            ),
                                        ],
                                    ),
                                ],
                            ),
                            accordion::item_indicator(
                                OpenState::Open,
                                false,
                                &props,
                                vec![],
                                vec![text("▾")],
                            ),
                        ],
                    )],
                ),
                accordion::item_content(
                    OpenState::Open,
                    false,
                    &props,
                    Some(content_id),
                    Some(trigger_id),
                    vec![],
                    vec![text(
                        "重要な更新のメール通知は既定で有効です。プッシュ通知は端末設定から個別に切り替えられます。",
                    )],
                ),
            ],
        )],
    )
}

/// `/themes/collapsible/`（Interactive カテゴリ）。
///
/// 一次情報: `crates/pre-styled-ui/src/collapsible.rs`（モジュール doc・
/// `stylesheet()` の 7 軸チェック節）と `crates/headless-ui/src/collapsible.rs`
/// （`root`/`trigger`/`indicator`/`content` シグネチャ・`aria-expanded`/
/// `aria-controls` の実出力テスト）。イシュー #1682/#1683/#2029（shadcn/ui
/// 突合による Examples 節「Nested navigation (file tree)」追加）。
pub const COLLAPSIBLE: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "単一の開閉パネル。Root / Trigger / Indicator / Content の 4 anatomy パーツを持つ。",
        "data-state（open/closed）を trigger の文字色強調・indicator の回転として視覚に反映する。",
        "data-disabled を 4 パート全てへ反映する（trigger のみネイティブ disabled 存在属性も伴う）。",
        "size/variant/colorPalette は提供しない（参照 4 サイト chakra-ui/Ark UI/Radix Primitives/shadcn-ui のいずれも Collapsible に持たないため）。",
        "closed のとき content は headless 層が付与する hidden 存在属性のみで隠れる（base では display を宣言せず、UA 既定の [hidden] { display: none } を上書きしない）。",
        "開閉時の高さアニメーション（Radix の collapsedHeight 相当）は意図的に非採用とする。理由はコンテンツ高さの実測が JS 前提という点だけでなく、headless 層が closed 時に content へ付与する hidden 存在属性を base 規則で上書きすると閉状態でも DOM 上へ再露出してしまう構造的な制約にもよる（shadcn/ui にも JS レスの代替実装は無い）。JS ゼロ SSG（クライアント側 JavaScript を読み込まない構成）での挙動は「JS ゼロ SSG での利用ガイド」（/guides/no-js-ssg/）を参照。",
        "shadcn/ui（Base UI）の Examples（Basic / Settings Panel / File Tree）はいずれも既存 API（root/trigger/indicator/content の再帰的な組み合わせ）で再現できる合成パターンであり、新規の variant/size/state 軸を追加しない（下記 Examples 節参照）。",
    ],
    arguments: &[
        ArgRow {
            name: "state",
            kind: "OpenState",
            default: "",
            description: "開閉状態（Open/Closed）。root/trigger/indicator/content の data-state へ反映される。",
        },
        ArgRow {
            name: "disabled",
            kind: "bool",
            default: "false",
            description: "無効状態。4 パート全ての data-disabled へ反映し、trigger にはネイティブ disabled 存在属性も付与する。",
        },
    ],
    examples: &[ExampleEntry {
        title: "Nested navigation (file tree)",
        description: "shadcn/ui（Base UI）の File Tree Example に相当する、collapsible を再帰的にネストした合成パターンです。フォルダ行はテキストの折り畳みトリガー（indicator は固定グリフ ▾ + data-state=\"open\" 時の回転で開閉方向を示す既存 CSS 規約に従う）、ファイル行はトリガーを持たない単なるテキストとして表現しています。新しい variant や data-* 語彙を追加せず、既存の root/trigger/indicator/content のみで構成しています。",
        render: ex_collapsible_nested_tree,
    }],
    keyboard: &[],
    aria: &[
        AriaRow {
            attribute: "aria-expanded",
            description: "trigger に付与。開閉状態を表す。",
        },
        AriaRow {
            attribute: "aria-controls",
            description: "trigger に付与。controls が Some のとき content の id を指す。",
        },
    ],
    demo: None,
};

/// [`COLLAPSIBLE`] の Examples 節「Nested navigation (file tree)」レンダラ
/// （イシュー #2029）。
///
/// shadcn/ui（Base UI）Collapsible ページの File Tree Example
/// （ChevronRightIcon + Folder/File アイコンでネストした複数 Collapsible）
/// を、既存 API のみで再現できることを示す合成デモ。フォルダ 1 件
/// （`src`）の中にフォルダ 2 件（開いた `components`・閉じた `utils`）と
/// ファイル 1 件（`main.rs`）を並べ、ネストしたフォルダの中にもファイルを
/// 置く（`docs/design/shadcn-reference-adoption-policy.md` §3 が
/// 「合わせない」と定める `data-slot` 等の Base UI 固有語彙・アイコン
/// フォント資産は使わず、indicator の子テキストは既存 CSS 規約（[`collapsible`]
/// の `recipe()`。単一固定グリフ `▾` + `indicator[data-state="open"]` の
/// `rotate(180deg)` で開閉方向を表す、[`crate::accordion`] と同型の規約）
/// にそのまま従う。グリフ自体を状態ごとに出し分けない）。
/// Demo（[`crate::showcase::collapsible_section`]、id `showcase-collapsible-*`）
/// と同じページに描画されるため、id は衝突しない
/// `showcase-collapsible-tree-*` を使う。
fn ex_collapsible_nested_tree() -> Node {
    let src_open = OpenState::Open;
    let components_open = OpenState::Open;
    let utils_closed = OpenState::Closed;

    collapsible::root(
        src_open,
        false,
        vec![],
        vec![
            collapsible::trigger(
                src_open,
                false,
                Some("showcase-collapsible-tree-src-content"),
                vec![],
                vec![
                    collapsible::indicator(src_open, false, vec![], vec![text("▾")]),
                    text("src"),
                ],
            ),
            collapsible::content(
                src_open,
                false,
                Some("showcase-collapsible-tree-src-content"),
                vec![],
                vec![
                    collapsible::root(
                        components_open,
                        false,
                        vec![],
                        vec![
                            collapsible::trigger(
                                components_open,
                                false,
                                Some("showcase-collapsible-tree-components-content"),
                                vec![],
                                vec![
                                    collapsible::indicator(
                                        components_open,
                                        false,
                                        vec![],
                                        vec![text("▾")],
                                    ),
                                    text("components"),
                                ],
                            ),
                            collapsible::content(
                                components_open,
                                false,
                                Some("showcase-collapsible-tree-components-content"),
                                vec![],
                                vec![
                                    div(vec![], vec![text("Button.rs")]),
                                    div(vec![], vec![text("Input.rs")]),
                                ],
                            ),
                        ],
                    ),
                    collapsible::root(
                        utils_closed,
                        false,
                        vec![],
                        vec![
                            collapsible::trigger(
                                utils_closed,
                                false,
                                Some("showcase-collapsible-tree-utils-content"),
                                vec![],
                                vec![
                                    collapsible::indicator(
                                        utils_closed,
                                        false,
                                        vec![],
                                        vec![text("▾")],
                                    ),
                                    text("utils"),
                                ],
                            ),
                            collapsible::content(
                                utils_closed,
                                false,
                                Some("showcase-collapsible-tree-utils-content"),
                                vec![],
                                vec![div(vec![], vec![text("helpers.rs")])],
                            ),
                        ],
                    ),
                    div(vec![], vec![text("main.rs")]),
                ],
            ),
        ],
    )
}

/// `/themes/action-bar/`（Interactive カテゴリ）。
///
/// 一次情報: `crates/headless-ui/src/action_bar.rs`（モジュール doc §参照
/// 基準・`root`/`content`/`close_trigger` シグネチャ・
/// `role="dialog"`/`data-expanded`/`role="separator"` の実出力テスト）。
/// 参照基準は chakra-ui のみ（Ark Popover の再利用、イシュー #1647 で
/// Primitives 側 `/primitives/action-bar/` と共通の突合を実施済み）。
pub const ACTION_BAR: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "複数選択操作バー（chakra-ui ActionBar 相当。実体は Ark Popover の再利用）。Root / Positioner / Content / SelectionTrigger / Separator / CloseTrigger の 6 anatomy パーツを持つ。",
        "開閉は Disclosure を埋め込んだ状態機械 ActionBar が管理する。選択件数から open を自動導出する糖衣 API は持たず、「選択操作 → 開閉状態の決定」は呼び出し側の責務とする。",
        "content に role=\"dialog\"（非モーダル）と aria-label（選択件数などの読み上げ用ラベル、呼び出し側が指定）を固定付与する。参照基準に合わせイシュー #1647 で role=\"toolbar\" から是正済み（**破壊的変更**）。開状態のみ data-expanded を、常時 tabindex=\"-1\" を付与する。",
        "close-trigger は呼び出し側が aria-label を指定せず、かつ children が空（可視テキストを持たないボタン）のときに限り既定値 \"close\" を出力する。",
    ],
    arguments: &[ArgRow {
        name: "state",
        kind: "OpenState",
        default: "",
        description: "開閉状態（Open/Closed）。root の data-state へ反映される。",
    }],
    examples: &[],
    keyboard: &[],
    aria: &[
        AriaRow {
            attribute: "role=\"dialog\"",
            description: "content に付与（非モーダル、aria-modal は付与しない）。aria-label とセットで固定付与される。",
        },
        AriaRow {
            attribute: "data-expanded",
            description: "content が開状態のときのみ付与される。",
        },
        AriaRow {
            attribute: "role=\"separator\"",
            description: "separator に付与。aria-orientation=\"vertical\" を伴う。",
        },
    ],
    demo: None,
};

/// `/themes/button-group/`（Interactive カテゴリ）。
///
/// 一次情報: `crates/headless-ui/src/button_group.rs`（イシュー #2059、親
/// #2058。モジュール doc・`root`/`separator`/`text` シグネチャ・
/// `role="group"`/`role="separator"`/`aria-orientation` の実出力テスト）と
/// `crates/pre-styled-ui/src/button_group.rs`（イシュー #2060。recipe・raw
/// CSS 追記の設計根拠）。参照基準は shadcn/ui のみ（`docs/design/
/// component-coverage-map.md` shadcn/ui 参照軸、ark-ui / chakra-ui / Radix
/// のいずれにも対応部品がない）。
pub const BUTTON_GROUP: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "関連ボタンを角丸・境界線でひとつのグループに見せる静的なグループ化コンテナ（shadcn/ui Button Group 相当）。Root / Separator / Text の 3 anatomy パーツを持つ。",
        "`crate::toolbar` の roving tabindex（矢印キーでフォーカスが移動する複合ウィジェット）とは異なり、状態機械を持たない静的なグループである。子 button のフォーカス順序はネイティブの Tab 順序に委ねる。",
        "先頭・末尾以外の隣接要素の角丸・開始側境界線幅を pre-styled-ui 側の raw CSS 追記（`[data-scope=\"button-group\"][data-part=\"root\"] > <child>:not(:first-child)` 等）で無効化し、1 つの連結表示に見せる。対象は button / input / text / menu trigger / select trigger の 5 種。",
        "`data-orientation` による横並び（既定）/ 縦積みの切り替え（root に固定出力、`aria-orientation` は role=\"group\" へ許可されないため付与しない）。",
        "separator はグループ自身の向きと直交する `aria-orientation`/`data-orientation` を出力する（横並びグループの区切り線は縦線になる、`crate::toolbar::separator` と同じ判断）。",
        "ネスト（グループの中にグループ）を許容する。`:has()` の先例がないため、内側 root は角丸連結対象へ含めず、代わりに先頭以外の内側グループへ margin のみ付与する（shadcn の `:has(>[data-slot=button-group])` gap 相当の代替表現）。",
        "size / variant / color-palette いずれの軸も提供しない（子の寸法に従属するレイアウト部品、`docs/design/pre-styled-ui-focus-ring-and-size-conventions.md` §4 (d)）。バリデーション・送信処理等のアプリケーションロジックは内包しない（`.claude/rules/coding-rust.md` §3.25）。",
    ],
    arguments: &[
        ArgRow {
            name: "orientation",
            kind: "Orientation",
            default: "Orientation::Horizontal",
            description: "root の data-orientation に反映する向き（Horizontal/Vertical）。separator へは直交した値が渡る。",
        },
        ArgRow {
            name: "label",
            kind: "&str",
            default: "",
            description: "root に付与する aria-label（空文字列のときは省略）。",
        },
    ],
    examples: &[],
    keyboard: &[],
    aria: &[
        AriaRow {
            attribute: "role=\"group\"",
            description: "root に付与（aria-orientation は role=\"group\" に許可されないため付与しない）。",
        },
        AriaRow {
            attribute: "role=\"separator\" / aria-orientation",
            description: "separator に付与。グループ自身の向きと直交する値になる。",
        },
    ],
    demo: None,
};

/// `/themes/toolbar/`（Interactive カテゴリ）。
///
/// 一次情報: `crates/headless-ui/src/toolbar.rs`（モジュール doc・
/// `root`/`button`/`separator`/`toggle_group`/`toggle_item` シグネチャ・
/// `role="toolbar"`/`role="separator"`/`role="group"`/`aria-pressed` の
/// 実出力テスト）。
pub const TOOLBAR: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "ボタン・リンク・セパレータ・ToggleGroup を横方向（または縦方向）にグループ化する操作バー。Root / Button / Link / Separator / ToggleGroup / ToggleItem の 6 anatomy パーツを持つ。",
        "roving tabindex（focused/item_count/loop_focus/orientation の複合状態機械 Toolbar）。フォーカス対象の項目のみ tabindex=\"0\"、それ以外は tabindex=\"-1\" になる。",
        "disabled 項目もフォーカス順序から除外しない（WAI-ARIA APG の toolbar パターン推奨に従う意図的な設計。aria-disabled/data-disabled で操作不能のみを表す）。",
        "押下状態の管理は独自実装せず、既存の ToggleGroup/MultiToggleGroup 状態機械を再利用する。",
        "separator は toolbar 自身の向きと直交する aria-orientation を出力する（横向き toolbar のセパレータは縦線になる）。",
        "link は既存の Link コンポーネントへ完全委譲し、external 時の target=\"_blank\"/rel=\"noopener noreferrer\" を不可分に付与する。",
    ],
    arguments: &[
        ArgRow {
            name: "orientation",
            kind: "Orientation",
            default: "Orientation::Horizontal",
            description: "root の role=\"toolbar\" に付与する向き（Horizontal/Vertical）。aria-orientation/data-orientation の両方へ反映される。",
        },
        ArgRow {
            name: "label",
            kind: "&str",
            default: "",
            description: "root に付与する aria-label（空文字列のときは省略）。",
        },
        ArgRow {
            name: "focused",
            kind: "bool",
            default: "",
            description: "button/link/toggle-item に付与。true のとき tabindex=\"0\"、false のとき tabindex=\"-1\"（roving tabindex）。",
        },
    ],
    examples: &[],
    keyboard: &[],
    aria: &[
        AriaRow {
            attribute: "role=\"toolbar\" / aria-orientation",
            description: "root に付与。orientation 引数の値（horizontal/vertical）を反映する。",
        },
        AriaRow {
            attribute: "role=\"separator\" / aria-orientation",
            description: "separator に付与。toolbar 自身の向きと直交する値になる。",
        },
        AriaRow {
            attribute: "role=\"group\"",
            description: "toggle-group に付与（aria-orientation は role=\"group\" に許可されないため付与しない）。",
        },
        AriaRow {
            attribute: "aria-pressed",
            description: "toggle-item に付与。押下状態（true/false）を表す。",
        },
        AriaRow {
            attribute: "aria-disabled",
            description: "disabled な button/toggle-item に付与。ネイティブ disabled は付与せずフォーカス順序に残す。",
        },
    ],
    demo: None,
};

/// `/themes/menubar/`（Interactive カテゴリ）。
///
/// 一次情報: `crates/headless-ui/src/menubar.rs`（モジュール doc・
/// `root`/`menu`/`trigger`/`positioner`/`content`/`item`/`item_group`/
/// `item_group_label`/`separator`/`sub_trigger`/`sub_content` シグネチャ・
/// `role="menubar"`/`role="none"`/`role="menuitem"`/`role="menu"` の実出力
/// テスト・`Menubar::decode_action` のアクション名網羅）。
pub const MENUBAR: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "複数 Menu を水平（または垂直）に並べるコンテナ。headless-ui 側（`fandhe_frontend_headless_ui::menubar`）は Root / Menu / Trigger / Positioner / Content / Arrow / ArrowTip / Item / ItemText / ItemIndicator / ItemGroup / ItemGroupLabel / Separator / SubTrigger / SubContent / CheckboxItem / RadioItemGroup / RadioItem の 18 anatomy パーツを持つ（イシュー #1652 で参照突合し 11 → 18 パーツ）。Themes 側（本ページ）はイシュー #2034 で shadcn/ui と突合し、新設パートのうち item-text / item-indicator / checkbox-item / radio-item-group / radio-item の 5 パーツへ `SLOTS`/CSS 付与を追いつかせた（#1528 が申し送っていた分の解消）。arrow / arrow-tip は shadcn/ui のデモに矢印インジケータが視認できないため意図的に未着装のまま。",
        "roving tabindex（focused/trigger_count/open/loop_focus/orientation の複合状態機械 Menubar）。フォーカス対象のトリガーのみ tabindex=\"0\"、それ以外は tabindex=\"-1\" になる。",
        "開いている Menu を跨いだ左右移動: ある Menu が開いた状態で Next/Prev/First/Last/Focus アクションを送ると、フォーカス移動と同時に開く Menu も隣へ移る（menubar 特有の挙動、Toolbar の roving tabindex には無い）。",
        "menu パーツは role=\"none\" を固定付与し、role=\"menubar\" の子として menuitem/group 以外の要素を挟まないようにする（WAI-ARIA APG の menubar パターン）。",
        "サブメニューの開閉状態は Menubar 自身ではなく、呼び出し側が別途持つ Menu インスタンス（Disclosure 埋め込み）から SubTrigger/SubContent へ注入する。",
        "既存の menu モジュールの anatomy はそのまま再利用しない（data-scope=\"menubar\" を独自に持つ）。状態機械・値語彙（OpenState/aria/data-* ヘルパ）のみを再利用する。",
    ],
    arguments: &[
        ArgRow {
            name: "orientation",
            kind: "Orientation",
            default: "Orientation::Horizontal",
            description: "root の role=\"menubar\" に付与する向き（Horizontal/Vertical）。aria-orientation/data-orientation の両方へ反映される。",
        },
        ArgRow {
            name: "label",
            kind: "&str",
            default: "",
            description: "root に付与する aria-label（空文字列のときは省略）。",
        },
        ArgRow {
            name: "focused",
            kind: "bool",
            default: "",
            description: "trigger に付与。true のとき tabindex=\"0\"、false のとき tabindex=\"-1\"（roving tabindex）。",
        },
        ArgRow {
            name: "state",
            kind: "OpenState",
            default: "",
            description: "trigger/positioner/content/sub_trigger/sub_content の開閉状態（Open/Closed）。",
        },
    ],
    examples: &[ExampleEntry {
        title: "Keyboard shortcut suffix",
        description: "shadcn/ui の Menubar デモが item 末尾に表示する `⌘T` 風のキーボードショートカット表示に相当する合成パターンです。`SlotRecipe` は子孫セレクタを持たない（イシュー #708 で不採用確定）ため、新しい anatomy パートは追加せず、item の子として item_text（flex: 1 1 auto でラベルを引き伸ばす）と crate::kbd の kbd() を並べるだけで末尾寄せを実現しています（item 自身の CSS は変更していません）。",
        render: ex_menubar_shortcut_suffix,
    }],
    keyboard: &[
        KeyRow {
            key: "ArrowRight / ArrowLeft",
            description: "次/前のトリガーへフォーカスを移動する。ある Menu が開いていれば、開く Menu も追随する（MenubarAction::Next/Prev、wasm 層実装）。",
        },
        KeyRow {
            key: "Home / End",
            description: "先頭/末尾のトリガーへフォーカスを移動する（MenubarAction::First/Last、wasm 層実装）。",
        },
        KeyRow {
            key: "Enter / Space / ArrowDown",
            description: "フォーカス中のトリガーの Menu を開く（MenubarAction::Open、wasm 層実装）。",
        },
        KeyRow {
            key: "Escape",
            description: "開いている Menu を閉じる（MenubarAction::Close、wasm 層実装）。",
        },
    ],
    aria: &[
        AriaRow {
            attribute: "role=\"menubar\" / aria-orientation",
            description: "root に付与。orientation 引数の値（horizontal/vertical）を反映する。",
        },
        AriaRow {
            attribute: "role=\"none\"",
            description: "menu に付与。role=\"menubar\" の子として menuitem/group 以外を挟まないための WAI-ARIA APG 慣行。",
        },
        AriaRow {
            attribute: "role=\"menuitem\" / aria-haspopup=\"menu\" / aria-expanded",
            description: "trigger/sub-trigger に付与。開閉状態（this Menu の state、もしくはサブメニュー側の sub_state）を反映する。",
        },
        AriaRow {
            attribute: "role=\"menu\"",
            description: "content/sub-content に固定付与。aria-labelledby は labelledby 引数が Some のときのみ出力される。",
        },
        AriaRow {
            attribute: "role=\"menuitem\"",
            description: "item に固定付与。disabled 時のみ aria-disabled=\"true\" が付与される。",
        },
        AriaRow {
            attribute: "role=\"group\"",
            description: "item-group に固定付与。labelledby が Some のときのみ aria-labelledby が付与される。",
        },
    ],
    demo: None,
};

/// [`MENUBAR`] の Examples 節「Keyboard shortcut suffix」レンダラ
/// （イシュー #2034）。
///
/// shadcn/ui の Menubar デモが item 末尾に表示するキーボードショートカット
/// （`⌘T` 風）に相当する合成パターン。新規 anatomy パートを追加せず、
/// item の子として item_text（`flex: 1 1 auto`）+ kbd を並べるだけで末尾
/// 寄せを実現する（`crates/pre-styled-ui/src/menubar.rs` モジュール doc
/// 「イシュー #2034」節参照）。
fn ex_menubar_shortcut_suffix() -> Node {
    menubar::item(
        "new-tab",
        false,
        false,
        vec![],
        vec![
            menubar::item_text(false, false, vec![], vec![text("New Tab")]),
            kbd(&KbdProps::default(), vec![], vec![text("⌘T")]),
        ],
    )
}

/// `/themes/navigation-menu/`（Interactive カテゴリ）。
///
/// 一次情報: `crates/headless-ui/src/navigation_menu.rs`（モジュール doc・
/// `root`/`list`/`item`/`trigger`/`content`/`link` シグネチャ・
/// `aria-expanded`/`aria-controls`/`aria-labelledby`/`aria-current` の
/// 実出力テスト・role 非出力の固定テスト）、
/// `crates/pre-styled-ui/src/navigation_menu.rs`（モジュール doc）。
///
/// `keyboard: &[]` とする理由: `decode_action` に方向系 variant を持たず
/// （[`crate::state::SingleSelect`] の `"select"`/`"toggle"`/`"deselect"`
/// のみ）、確定したキー割り当てを本クレートのソースから裏付けられない
/// ため（本モジュール冒頭の rustdoc「menubar のみ [`KeyRow`] を空にして
/// いない」の記述はそのまま不変。navigation-menu も他 14 部品と同じく
/// 空のまま）。
pub const NAVIGATION_MENU: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "トリガー起点で開閉するナビゲーションパネル。Root / List / Item / Trigger / ItemIndicator / Content / Link / Indicator の 8 anatomy パーツを持つ（イシュー #1654 で ItemIndicator を新設し 6 → 7 パーツへ、イシュー #2187 でルートレベルの Indicator を新設し 7 → 8 パーツへ拡張）。",
        "高々 1 個の Trigger だけが開く状態機械（SingleSelect を埋め込んだ NavigationMenu）。dispatch は \"select\"/\"toggle\"/\"deselect\"。",
        "role は一切付与しない。root は素の nav の暗黙 ARIA role（navigation）に依拠し、role=\"menu\"/role=\"menuitem\" は付与しない（文書ナビを操作メニューと誤伝達しないための設計、nav_list と同じ判断）。",
        "アクティブリンクは aria-current=\"page\" + data-current で表す（role は付与しない）。",
        "data-motion（アニメーション方向の露出）・viewport 寸法測定は実装しない（intentional-non-adoption.md §3.25 規則 2 により headless 層へ持ち込まない設計判断）。",
        "indicator（イシュー #2187）は開いている Trigger の下でスライドするポインタ。座標（x/width/y/height）は --fandhe-navigation-menu-indicator-* の CSS 変数契約で表現し、既定 0px フォールバックのため実座標が書き込まれない間は不可視。実座標の書き込みは wasm-full の責務（#2208/#2209 系の後続）。",
    ],
    arguments: &[
        ArgRow {
            name: "label",
            kind: "&str",
            default: "",
            description: "root に付与する aria-label（必須引数）。",
        },
        ArgRow {
            name: "state",
            kind: "OpenState",
            default: "",
            description: "item/trigger/content/indicator の開閉状態（Open/Closed）。",
        },
        ArgRow {
            name: "current",
            kind: "bool",
            default: "false",
            description: "link に付与。true のとき aria-current=\"page\" + data-current を出力する。",
        },
        ArgRow {
            name: "value",
            kind: "Option<&str>",
            default: "None",
            description: "indicator に付与。Some のときのみ data-value として開いている項目値を出力する。",
        },
    ],
    examples: &[ExampleEntry {
        title: "Title + description link grid",
        description: "shadcn/ui の \"Components\" パネル相当のタイトル + 説明文リンク合成パターンです。`SlotRecipe` は子孫セレクタを持たない（イシュー #708 で不採用確定）ため、新しい anatomy パートは追加せず、link の子として crate::text（太字タイトル + 淡色サイズの説明文）を並べるだけで再現しています（link/content 自身の CSS は変更していません。2 列グリッド配置も content へ渡す style 属性のみで実現）。",
        render: ex_navigation_menu_title_description_link,
    }],
    keyboard: &[],
    aria: &[
        AriaRow {
            attribute: "aria-label",
            description: "root に付与。root の role は素の nav の暗黙 role（navigation）に依拠し明示付与しない。",
        },
        AriaRow {
            attribute: "aria-expanded / aria-controls",
            description: "trigger に付与。開閉状態（true/false）と content との関連付け（controls が Some のときのみ）を表す。role は付与しない。",
        },
        AriaRow {
            attribute: "aria-labelledby",
            description: "content に付与（labelled_by が Some のときのみ）。role は付与しない。",
        },
        AriaRow {
            attribute: "aria-current=\"page\"",
            description: "link に付与（current が true のときのみ）。role は付与しない。",
        },
        AriaRow {
            attribute: "aria-hidden=\"true\"",
            description: "item-indicator/indicator に固定付与。トリガーの aria-expanded から開閉状態が既に伝わるための装飾専用要素（item-indicator はイシュー #2035、indicator はイシュー #2187）。",
        },
    ],
    demo: None,
};

/// [`NAVIGATION_MENU`] の Examples 節「Title + description link grid」
/// レンダラ（イシュー #2035）。
///
/// shadcn/ui の "Components" パネル相当のタイトル + 説明文リンク合成。
/// 新規 anatomy パートを追加せず、link の子として crate::text
/// （太字タイトル + 小サイズの説明文）を並べるだけで再現する
/// （`crates/pre-styled-ui/src/navigation_menu.rs` モジュール doc
/// 「shadcn/ui 突合（イシュー #2035）」節参照）。
fn ex_navigation_menu_title_description_link() -> Node {
    navigation_menu::link(
        "",
        false,
        vec![("style", "display: block; padding: var(--fandhe-space-2);")],
        vec![
            styled_text(
                &TextProps {
                    weight: TextWeight::Semibold,
                    ..TextProps::default()
                },
                vec![],
                vec![text("Analytics")],
            ),
            styled_text(
                &TextProps {
                    size: TextSize::Sm,
                    ..TextProps::default()
                },
                vec![],
                vec![text("利用状況・パフォーマンスを可視化するダッシュボード。")],
            ),
        ],
    )
}

/// `/themes/dialog/`（Interactive カテゴリ）。
///
/// 一次情報: `crates/pre-styled-ui/src/dialog.rs`（モジュール doc・`root`
/// シグネチャ）、`crates/headless-ui/src/dialog.rs`（`aria-haspopup`/
/// `role="dialog"`・`role="alertdialog"`/`aria-modal` の実出力テスト）。
pub const DIALOG: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "headless-ui 由来の Root / Trigger / Backdrop / Positioner / Content / Title / Description / CloseTrigger の 8 anatomy パーツに加え、pre-styled-only の footer パート（イシュー #1690）・body パート（イシュー #2030）を持つモーダルダイアログ（data-scope=\"dialog\" 配下は headless 8 パート + pre-styled-only 2 パートの計 10 件）。",
        "DialogRole（Dialog/Alertdialog）で role=\"dialog\"/role=\"alertdialog\" を出し分ける。",
        "size variant（Xs/Sm/Md/Lg/Xl、既定 Md、イシュー #1714）で root の寸法を切り替える。",
        "フォーカストラップ・Escape キーでの閉鎖・外側クリックでの閉鎖は JS ランタイム側の責務であり、本レイヤーは SSR/属性出力のみを担う。",
        "alert-dialog（確認ダイアログ）構成: 独立部品や新しい variant 軸ではなく、role=\"alertdialog\"（DialogRole::Alertdialog）+ footer（アクション列レイアウト）+ button（Solid/Danger と Outline の組み合わせ）で表現する（イシュー #1690。role=\"alertdialog\" の dialog は wasm-full 層が外側クリックでの閉鎖を既定で無効化する）。footer 自体は送信・閉鎖等のアプリケーションロジックを持たないレイアウト専用パートである。",
        "スクロール可能コンテンツ: body パート（shadcn/ui〔Base UI スタイル〕突合、イシュー #2030）は title/description（見出し）と footer（アクション列）を content 内で固定したまま、本文だけを縦スクロールさせるレイアウト専用パートである（`overflow-y: auto` / `max-height: 50vh`）。呼び出し側の任意判断によるオプトインパートであり、使わない既存の呼び出しには影響しない。",
    ],
    arguments: &[
        ArgRow {
            name: "size",
            kind: "Size",
            default: "Size::Md",
            description: "root へ付与するサイズ variant（Xs/Sm/Md/Lg/Xl、イシュー #1714）。",
        },
        ArgRow {
            name: "state",
            kind: "OpenState",
            default: "",
            description: "開閉状態（Open/Closed）。root/content の data-state へ反映される。",
        },
    ],
    examples: &[
        ExampleEntry {
            title: "Alert dialog",
            description: "DialogRole::Alertdialog（role=\"alertdialog\"）と footer（イシュー #1690、pre-styled-only のアクション列パート）に、既存の button を Solid/ColorPalette::Danger（破壊的確認）と Outline（キャンセル）の組み合わせで構成した確認ダイアログの例です。role=\"alertdialog\" の dialog は wasm-full 層が外側クリックでの閉鎖を既定で無効化します。footer はレイアウト専用パートであり送信・閉鎖の配線は持ちません（close_trigger は併用せず省略しています）。",
            render: ex_alert_dialog,
        },
        ExampleEntry {
            title: "Share link (custom close button)",
            description: "shadcn/ui（Base UI スタイル）の「Custom Close Button」デモ（イシュー #2030、親 #2025）に対応する構成です。content 右上のアイコン専用 close-trigger は使わず、footer 内の通常の button（ButtonVariant::Outline）で平文の \"Close\" ボタンを併設します。close-trigger のアイコン専用契約（codex-review #1795）と SlotRecipe の子孫セレクタ非対応（イシュー #708）により、footer 内で close-trigger を機能配線済みボタンとして再利用することはできないため、既存の footer + button の組み合わせのみで表現しています。",
            render: ex_share_link_custom_close_button,
        },
    ],
    keyboard: &[],
    aria: &[
        AriaRow {
            attribute: "aria-haspopup=\"dialog\"",
            description: "trigger に固定付与。",
        },
        AriaRow {
            attribute: "aria-expanded",
            description: "trigger に付与。開閉状態を表す。",
        },
        AriaRow {
            attribute: "aria-controls",
            description: "trigger に付与。controls が Some のとき content の id を指す。",
        },
        AriaRow {
            attribute: "role=\"dialog\" / role=\"alertdialog\"",
            description: "content に付与。DialogRole に応じて出し分けられる。",
        },
        AriaRow {
            attribute: "aria-modal",
            description: "content に付与。モーダルかどうかを表す真偽値。",
        },
        AriaRow {
            attribute: "aria-labelledby / aria-describedby",
            description: "content に付与。対応する title/description が設定されているときのみ出力される。",
        },
    ],
    demo: None,
};

/// [`DIALOG`] の Examples 節「Alert dialog」レンダラ（イシュー #1691）。
///
/// `dialog.rs` rustdoc「alert-dialog 構成」節（イシュー #1690）が確定した
/// 5 要素のうち、本 crate から到達可能な 4 要素（role・footer・button の
/// variant/palette 組み合わせ・close_trigger の要否判断）を組み合わせる
/// （外側クリック非閉鎖は wasm-full 層の既定挙動でありノード木には現れない）。
/// Demo（[`crate::showcase::dialog_section`]）と同じページに描画されるため、
/// id は `showcase-dialog-*` と衝突しない `showcase-alert-dialog-*` を使う。
/// `close_trigger` は併用せず省略する（rustdoc 項目 5 の Radix 流の選択）。
fn ex_alert_dialog() -> Node {
    div(
        vec![],
        vec![
            dialog::trigger(
                OpenState::Open,
                Some("showcase-alert-dialog-content"),
                vec![],
                vec![text("Delete file")],
            ),
            dialog::root(
                Size::Md,
                OpenState::Open,
                vec![],
                vec![
                    dialog::backdrop(OpenState::Open, vec![], vec![]),
                    dialog::positioner(
                        OpenState::Open,
                        vec![],
                        vec![dialog::content(
                            OpenState::Open,
                            DialogRole::Alertdialog,
                            true,
                            ContentIds {
                                id: Some("showcase-alert-dialog-content"),
                                labelledby: Some("showcase-alert-dialog-title"),
                                describedby: Some("showcase-alert-dialog-desc"),
                            },
                            vec![],
                            vec![
                                dialog::title(
                                    Some("showcase-alert-dialog-title"),
                                    vec![],
                                    vec![text("Delete this file?")],
                                ),
                                dialog::description(
                                    Some("showcase-alert-dialog-desc"),
                                    vec![],
                                    vec![text(
                                        "この操作は取り消せません。ファイルは完全に削除されます。",
                                    )],
                                ),
                                dialog::footer(
                                    vec![],
                                    vec![
                                        button(
                                            &ButtonProps {
                                                variant: ButtonVariant::Outline,
                                                ..ButtonProps::default()
                                            },
                                            vec![],
                                            vec![text("Cancel")],
                                        ),
                                        button(
                                            &ButtonProps {
                                                variant: ButtonVariant::Solid,
                                                palette: ColorPalette::Danger,
                                                ..ButtonProps::default()
                                            },
                                            vec![],
                                            vec![text("Delete")],
                                        ),
                                    ],
                                ),
                            ],
                        )],
                    ),
                ],
            ),
        ],
    )
}

/// [`DIALOG`] の Examples 節「Share link (custom close button)」レンダラ
/// （イシュー #2030、親 #2025、shadcn/ui〔Base UI スタイル〕「Custom Close
/// Button」デモとの突合）。
///
/// content 右上のアイコン専用 close-trigger（イシュー #1795 でアイコン専用
/// 契約に固定済み）は使わず、footer 内の通常の button
/// （`ButtonVariant::Outline`）で平文の "Close" ボタンを併設する。
/// `close_trigger` を footer 内で機能配線済みボタンとして再利用すること
/// は、(1) close-trigger のアイコン専用契約、(2)
/// [`fandhe_frontend_pre_styled_ui`] の `SlotRecipe` が子孫セレクタ機構を
/// 持たない（イシュー #708）ことの 2 点により本イシュー単体では実現でき
/// ないため、既存 API（footer + button）のみで表現している（本イシューの
/// スコープ外、`crates/pre-styled-ui/src/dialog.rs` モジュール冒頭 rustdoc
/// 「close-trigger のフッター内再利用」節参照）。
/// Demo（[`crate::showcase::dialog_section`]）・[`ex_alert_dialog`] と
/// 同一ページに描画されるため、id は両者と衝突しない
/// `showcase-share-dialog-*` を使う。
fn ex_share_link_custom_close_button() -> Node {
    div(
        vec![],
        vec![
            dialog::trigger(
                OpenState::Open,
                Some("showcase-share-dialog-content"),
                vec![],
                vec![text("Share")],
            ),
            dialog::root(
                Size::Md,
                OpenState::Open,
                vec![],
                vec![
                    dialog::backdrop(OpenState::Open, vec![], vec![]),
                    dialog::positioner(
                        OpenState::Open,
                        vec![],
                        vec![dialog::content(
                            OpenState::Open,
                            DialogRole::Dialog,
                            true,
                            ContentIds {
                                id: Some("showcase-share-dialog-content"),
                                labelledby: Some("showcase-share-dialog-title"),
                                describedby: Some("showcase-share-dialog-desc"),
                            },
                            vec![],
                            vec![
                                dialog::title(
                                    Some("showcase-share-dialog-title"),
                                    vec![],
                                    vec![text("Share link")],
                                ),
                                dialog::description(
                                    Some("showcase-share-dialog-desc"),
                                    vec![],
                                    vec![text("このリンクを知っている人は誰でも閲覧できます。")],
                                ),
                                // close-trigger は併用しない（アイコン専用
                                // 契約と footer 内再利用不可のため）。
                                // 平文の "Close" ボタンのみを footer に置く。
                                dialog::footer(
                                    vec![],
                                    vec![button(
                                        &ButtonProps {
                                            variant: ButtonVariant::Outline,
                                            ..ButtonProps::default()
                                        },
                                        vec![],
                                        vec![text("Close")],
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

/// `/themes/drawer/`（Interactive カテゴリ）。
///
/// 一次情報: `crates/pre-styled-ui/src/drawer.rs`（モジュール doc・`root`
/// シグネチャ）、`crates/headless-ui/src/drawer.rs`（Dialog パターンの
/// 変種であることの記述・`role="dialog"`/`aria-modal` の実出力テスト）。
pub const DRAWER: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "画面端からスライドインするパネル。WAI-ARIA 上は Dialog パターンの変種であり、新規状態機械を作らず crate::dialog の状態機械（Disclosure）をそのまま再利用する。",
        "Root / Trigger / Backdrop / Positioner / Content / Title / Description / CloseTrigger の 8 anatomy パーツ（dialog と同一構成）。",
        "DrawerPlacement（Start/End/Top/Bottom、既定 End）で画面のどの端から出現するかを data-placement として root/positioner/content へ出力する。",
        "size variant で寸法を切り替える。",
    ],
    arguments: &[
        ArgRow {
            name: "size",
            kind: "Size",
            default: "Size::Md",
            description: "root へ付与するサイズ variant（Xs/Sm/Md/Lg/Xl）。",
        },
        ArgRow {
            name: "state",
            kind: "OpenState",
            default: "",
            description: "開閉状態（Open/Closed）。dialog と同じ Disclosure 状態機械を再利用する。",
        },
        ArgRow {
            name: "placement",
            kind: "DrawerPlacement",
            default: "DrawerPlacement::End",
            description: "画面のどの端から出現するか（Start/End/Top/Bottom）。data-placement として出力される。",
        },
    ],
    examples: &[ExampleEntry {
        title: "Footer action row (bottom placement)",
        description: "shadcn/ui の Sheet/Drawer が持つフッター相当のアクション配置を、既存 API のみで再現した合成パターンです（イシュー #2031）。headless anatomy に専用 footer パートが存在しないため（description 直後の通常行として掲示する制約は drawer.rs rustdoc の「本イシューのスコープ外」節を継承）、description の直後に Cancel/Save の 2 ボタンを並べています。placement=\"bottom\" を掲示し、DrawerPlacement の 4 方向のうち Demo（end）とは異なる方向を示します。",
        render: ex_drawer_footer_bottom,
    }],
    keyboard: &[],
    aria: &[
        AriaRow {
            attribute: "aria-haspopup=\"dialog\"",
            description: "trigger に固定付与。",
        },
        AriaRow {
            attribute: "aria-expanded / aria-controls",
            description: "trigger に付与。dialog と同一の意味論。",
        },
        AriaRow {
            attribute: "role=\"dialog\"",
            description: "content に固定付与。",
        },
        AriaRow {
            attribute: "aria-modal",
            description: "content に付与。モーダルかどうかを表す真偽値。",
        },
        AriaRow {
            attribute: "aria-labelledby / aria-describedby",
            description: "content に付与。対応する title/description が設定されているときのみ出力される。",
        },
    ],
    demo: None,
};

/// [`DRAWER`] の Examples 節「Footer action row (bottom placement)」レンダラ
/// （イシュー #2031、shadcn/ui `Sheet`/`Drawer` との突合）。
///
/// headless drawer の anatomy は dialog と同一の 8 パーツのみで footer
/// パートを持たない（`crates/pre-styled-ui/src/drawer.rs` rustdoc「本
/// イシューのスコープ外」節、イシュー #1695 で確定済みの制約）ため、
/// description 直後の通常行として Cancel/Save を並べる合成パターンを示す
/// （`showcase::drawer_section` の Demo と同型。ID は Demo と衝突しない
/// `showcase-drawer-footer-example-*` を使う）。placement は Demo（end）と
/// 異なる bottom を掲示し、DrawerPlacement の網羅性を示す。
fn ex_drawer_footer_bottom() -> Node {
    div(
        vec![],
        vec![
            drawer::trigger(
                OpenState::Open,
                Some("showcase-drawer-footer-example-content"),
                vec![],
                vec![text("Open bottom drawer")],
            ),
            drawer::root(
                Size::Md,
                OpenState::Open,
                DrawerPlacement::Bottom,
                vec![],
                vec![
                    drawer::backdrop(OpenState::Open, vec![], vec![]),
                    drawer::positioner(
                        OpenState::Open,
                        DrawerPlacement::Bottom,
                        vec![],
                        vec![drawer::content(
                            OpenState::Open,
                            DrawerPlacement::Bottom,
                            true,
                            ContentIds {
                                id: Some("showcase-drawer-footer-example-content"),
                                labelledby: Some("showcase-drawer-footer-example-title"),
                                describedby: Some("showcase-drawer-footer-example-desc"),
                            },
                            vec![],
                            vec![
                                drawer::title(
                                    Some("showcase-drawer-footer-example-title"),
                                    vec![],
                                    vec![text("Filters")],
                                ),
                                drawer::description(
                                    Some("showcase-drawer-footer-example-desc"),
                                    vec![],
                                    vec![text("画面下端からスライドインするパネルの例です。")],
                                ),
                                // headless anatomy に専用 footer パートが存在
                                // しないため、description 直後に通常の行として
                                // 掲示する。`.showcase-row` は掲示用レイアウト
                                // のみを担い、製品 CSS には footer 規則を持ち
                                // 込まない（showcase.rs の他 drawer 例と同型）。
                                div(
                                    vec![("class", "showcase-row")],
                                    vec![
                                        button(
                                            &ButtonProps {
                                                variant: ButtonVariant::Outline,
                                                ..ButtonProps::default()
                                            },
                                            vec![],
                                            vec![text("Cancel")],
                                        ),
                                        button(&ButtonProps::default(), vec![], vec![text("Save")]),
                                    ],
                                ),
                                drawer::close_trigger(
                                    vec![("aria-label", "Close")],
                                    vec![text("×")],
                                ),
                            ],
                        )],
                    ),
                ],
            ),
        ],
    )
}

/// `/themes/floating-panel/`（Interactive カテゴリ）。
///
/// 一次情報: `crates/headless-ui/src/floating_panel.rs`（モジュール doc・
/// `root` シグネチャ・`role="dialog"`（`aria-modal` 非出力）の実出力
/// テスト）。
pub const FLOATING_PANEL: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "ドラッグ移動・リサイズ可能な浮遊パネル。Root / Trigger / Positioner / Content / Header / Title / Control / StageTrigger / CloseTrigger / Body の 10 anatomy パーツを持つ。",
        "開閉（crate::state::Disclosure 埋め込み）に加え、default/minimized/maximized の 3 値を持つ独自状態 Stage を管理する。",
        "content は非モーダル overlay のため role=\"dialog\" のみを付与し、aria-modal は出力しない（ユーザーは他の要素を操作し続けられる）。",
        "座標は --fandhe-x / --fandhe-y CSS 変数として出力する（実際のドラッグ操作の DOM 配線は wasm-full 側の後続スコープ）。",
    ],
    arguments: &[
        ArgRow {
            name: "state",
            kind: "OpenState",
            default: "",
            description: "開閉状態（Open/Closed）。",
        },
        ArgRow {
            name: "stage",
            kind: "Stage",
            default: "",
            description: "パネルの表示段階（default/minimized/maximized）。data-stage へ反映される。",
        },
    ],
    examples: &[],
    keyboard: &[],
    aria: &[
        AriaRow {
            attribute: "aria-haspopup=\"dialog\"",
            description: "trigger に固定付与。",
        },
        AriaRow {
            attribute: "aria-expanded / aria-controls",
            description: "trigger に付与。開閉状態と対応する content の id を表す。",
        },
        AriaRow {
            attribute: "role=\"dialog\"",
            description: "content に固定付与。aria-modal は出力しない（非モーダル overlay）。",
        },
        AriaRow {
            attribute: "aria-labelledby",
            description: "content に付与。対応する title が設定されているときのみ出力される。",
        },
    ],
    demo: None,
};

/// `/themes/hover-card/`（Interactive カテゴリ）。
///
/// 一次情報: `crates/headless-ui/src/hover_card.rs`（モジュール doc・
/// `root`/`trigger` シグネチャ・hover card 専用パターンが WAI-ARIA に
/// 存在しないため `aria-expanded` 等を付与しないことの記述と実測テスト）。
/// イシュー #2032（shadcn/ui 突合による Examples 節「User profile preview」
/// 追加）。
pub const HOVER_CARD: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "リンク先プレビュー等、hover / focus で開閉するオーバーレイ。Root / Trigger / Positioner / Content / Arrow / ArrowTip の 6 anatomy パーツを持つ。",
        "trigger はリンク先プレビュー用途の a 要素であり、javascript: 等の危険スキームは fandhe-frontend-core の URL スキーム検証が除去する。",
        "HoverCardDelays（既定 open_ms: 600 / close_ms: 300）を data-open-delay / data-close-delay として root へ出力する。実際の hover/focus タイマー駆動は wasm-full 側の後続スコープ。",
        "WAI-ARIA に hover card 専用パターンは存在しないため、trigger へ aria-expanded / aria-controls / aria-haspopup は付与しない。",
        "shadcn/ui（Base UI）の Basic Example（Avatar + ユーザー名/説明文の合成カード）は content の children へ avatar::root/fallback を並べるだけの既存 API のみで再現できる合成パターンであり、新規の variant/size/state 軸を追加しない（下記 Examples 節参照）。",
    ],
    arguments: &[
        ArgRow {
            name: "state",
            kind: "OpenState",
            default: "",
            description: "開閉状態（Open/Closed）。",
        },
        ArgRow {
            name: "delays",
            kind: "HoverCardDelays",
            default: "HoverCardDelays::default()（open_ms: 600, close_ms: 300）",
            description: "hover/focus の開閉遅延（ms）。data-open-delay / data-close-delay として出力される決定的な SSR 設定値。",
        },
    ],
    examples: &[ExampleEntry {
        title: "User profile preview",
        description: "shadcn/ui（Base UI）の Basic Example に相当する、avatar と説明テキストを組み合わせた合成パターンです。avatar は fandhe-frontend-pre-styled-ui::avatar の root/fallback をそのまま使い、外部画像を読み込まないフォールバック文字（イニシャル）のみで表示します。新しい variant や data-* 語彙を追加せず、既存の hover_card::content の自由な children と avatar の既存 API のみで構成しています。",
        render: ex_hover_card_user_preview,
    }],
    keyboard: &[],
    aria: &[AriaRow {
        attribute: "aria-hidden=\"true\"",
        description: "positioner に付与。閉じている間、支援技術から内容を隠す。",
    }],
    demo: None,
};

/// [`HOVER_CARD`] の Examples 節「User profile preview」レンダラ
/// （イシュー #2032）。
///
/// shadcn/ui（Base UI）Hover Card ページの Basic Example（Avatar +
/// ユーザー名/説明文の合成カード）を、既存 API のみで再現できることを示す
/// 合成デモ。avatar は外部フェッチ・404 を発生させないよう
/// `avatar::fallback`（イニシャル文字列のみ、画像なし）で表示する
/// （`crates/docs-site/src/showcase.rs` の `avatar_section` が使う
/// `data:` URI 固定パターンと同じ「外部 URL を使わない」方針）。
/// 横並び（row）配置は `content` への `display: flex` 追加という既存 CSS
/// 出力の変更を伴うため、単一部品の合成パターン補完という本イシューの
/// スコープでは行わず、avatar と説明テキストは既定の縦積みのまま配置する
/// （[`fandhe_frontend_pre_styled_ui::hover_card`] モジュール doc「shadcn/ui 突合」節参照）。
/// Demo（[`crate::showcase::hover_card_section`]）と同じページに描画
/// されるため、id は衝突しない `showcase-hover-card-user-preview-*` を使う。
fn ex_hover_card_user_preview() -> Node {
    let open = OpenState::Open;
    let delays = HoverCardDelays::default();

    hover_card::root(
        open,
        delays,
        vec![],
        vec![
            hover_card::trigger(
                open,
                Some("https://fandhe-frontend.example/users/ada"),
                vec![],
                vec![text("@ada")],
            ),
            hover_card::positioner(
                open,
                vec![],
                vec![hover_card::content(
                    open,
                    Some("showcase-hover-card-user-preview-content"),
                    vec![],
                    vec![
                        avatar::root(
                            &AvatarProps::default(),
                            vec![],
                            vec![avatar::fallback(
                                ImageStatus::Error,
                                vec![],
                                vec![text("AL")],
                            )],
                        ),
                        div(
                            vec![],
                            vec![
                                p(vec![], vec![strong(vec![], vec![text("Ada Lovelace")])]),
                                p(
                                    vec![],
                                    vec![text("Sample profile for the hover card demo.")],
                                ),
                            ],
                        ),
                    ],
                )],
            ),
        ],
    )
}

/// `/themes/menu/`（Interactive カテゴリ）。
///
/// 一次情報: `crates/pre-styled-ui/src/menu.rs`（モジュール doc・`root`
/// シグネチャ）、`crates/headless-ui/src/menu.rs`（`aria-haspopup="menu"`/
/// `role="menu"`/`role="menuitem"`/`role="group"` の実出力テスト）。
pub const MENU: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "トリガー起点のオーバーレイ + アクション項目リスト。Root / Trigger / Indicator / Positioner / Content / Arrow / ArrowTip / Item / ItemText / ItemIndicator / ItemGroup / ItemGroupLabel / Separator / TriggerItem / ContextTrigger / CheckboxItem / RadioItemGroup / RadioItem の 18 anatomy パーツを持つ。",
        "サブメニューは親 Menu インスタンスの content 内に子 Menu インスタンス由来の trigger_item / positioner / content を入れ子で配置して表現し、親子双方に aria-haspopup=\"menu\" を付与する。",
        "CheckboxItem / RadioItemGroup は開閉状態とは独立した checked 状態機械（MenuCheckboxItem / MenuRadioItemGroup）を持つ。",
        "size variant で root/content の padding を切り替える。",
        "ItemText / ItemIndicator は headless anatomy には #1651 時点で存在していたが、pre-styled-ui 側の再エクスポート・CSS 着装漏れをイシュー #2033（shadcn/ui 突合）で補完した。ショートカット表示は新規 anatomy パートを追加せず、独立部品 kbd との合成パターンで実現する。",
    ],
    arguments: &[
        ArgRow {
            name: "size",
            kind: "Size",
            default: "Size::Md",
            description: "root へ付与するサイズ variant。root/content の padding を切り替える。",
        },
        ArgRow {
            name: "state",
            kind: "OpenState",
            default: "",
            description: "開閉状態（Open/Closed）。",
        },
    ],
    examples: &[
        ExampleEntry {
            title: "グループ・checkbox/radio 項目・ショートカット",
            description: "item_group + item_group_label によるグループ化、checkbox_item / radio_item によるチェック可能な項目、kbd との合成によるショートカット表示を組み合わせた例です。",
            render: ex_menu_group_checkable_shortcut,
        },
        ExampleEntry {
            title: "サブメニュー",
            description: "trigger_item + 入れ子の positioner/content で子 Menu インスタンスを埋め込み、サブメニューを表現する例です。",
            render: ex_menu_submenu,
        },
        ExampleEntry {
            title: "inset 項目・destructive 項目",
            description: "アイコンを持たない項目のテキスト位置を揃える data-inset と、危険操作を示す data-danger（pre-styled-only の存在属性、item() の attrs 経由で付与）を組み合わせた例です。",
            render: ex_menu_inset_and_danger,
        },
    ],
    keyboard: &[],
    aria: &[
        AriaRow {
            attribute: "aria-haspopup=\"menu\"",
            description: "trigger（およびサブメニューの trigger_item）に固定付与。",
        },
        AriaRow {
            attribute: "aria-expanded / aria-controls",
            description: "trigger に付与。開閉状態と対応する content の id を表す。",
        },
        AriaRow {
            attribute: "role=\"menu\"",
            description: "content に固定付与。aria-labelledby は trigger の id が設定されているときのみ出力される。",
        },
        AriaRow {
            attribute: "role=\"menuitem\"",
            description: "item に固定付与。disabled 時のみ aria-disabled=\"true\" が付与される。",
        },
        AriaRow {
            attribute: "role=\"group\"",
            description: "item_group に固定付与。labelledby が Some のときのみ aria-labelledby が付与される。",
        },
    ],
    demo: None,
};

// --- イシュー #2033（shadcn/ui 突合）: MENU の Examples 節 ---
//
// `crates/pre-styled-ui/src/menu.rs` の再エクスポート済み API のみで
// 組み立てる（`format!` によるマークアップ直接組み立てはしない、REQ-1
// 遵守）。ネストした positioner のフロー内配置中和は
// `crates/docs-site/src/showcase.rs` の `.pre-styled-showcase
// [data-scope="menu"][data-part="positioner"]` セレクタが本ページ全体の
// スタイルシートにも含まれるため、Examples 節でも成立する。

fn ex_menu_group_checkable_shortcut() -> Node {
    menu::root(
        Size::Md,
        OpenState::Open,
        vec![],
        vec![
            menu::trigger(
                OpenState::Open,
                false,
                Some("spec-menu-checkable-content"),
                vec![],
                vec![text("Options")],
            ),
            menu::positioner(
                OpenState::Open,
                vec![],
                vec![menu::content(
                    OpenState::Open,
                    Some("spec-menu-checkable-content"),
                    None,
                    vec![],
                    vec![
                        menu::item_group(
                            Some("spec-menu-checkable-group"),
                            vec![],
                            vec![
                                menu::item_group_label(
                                    Some("spec-menu-checkable-group"),
                                    vec![],
                                    vec![text("Edit")],
                                ),
                                menu::item(
                                    "save",
                                    false,
                                    false,
                                    vec![],
                                    vec![
                                        menu::item_text(false, false, vec![], vec![text("Save")]),
                                        kbd(
                                            &KbdProps {
                                                variant: KbdVariant::Subtle,
                                                ..KbdProps::default()
                                            },
                                            vec![],
                                            vec![text("⌘S")],
                                        ),
                                    ],
                                ),
                            ],
                        ),
                        menu::separator(vec![], vec![]),
                        menu::checkbox_item(
                            true,
                            "word-wrap",
                            false,
                            false,
                            vec![],
                            vec![
                                menu::item_indicator(true, vec![], vec![text("✓")]),
                                menu::item_text(false, false, vec![], vec![text("Word wrap")]),
                            ],
                        ),
                        menu::radio_item_group(
                            Some("spec-menu-radio-group"),
                            vec![],
                            vec![
                                menu::item_group_label(
                                    Some("spec-menu-radio-group"),
                                    vec![],
                                    vec![text("Theme")],
                                ),
                                menu::radio_item(
                                    true,
                                    "dark",
                                    false,
                                    false,
                                    vec![],
                                    vec![
                                        menu::item_indicator(true, vec![], vec![text("●")]),
                                        menu::item_text(false, false, vec![], vec![text("Dark")]),
                                    ],
                                ),
                                menu::radio_item(
                                    false,
                                    "light",
                                    false,
                                    false,
                                    vec![],
                                    vec![
                                        menu::item_indicator(false, vec![], vec![text("●")]),
                                        menu::item_text(false, false, vec![], vec![text("Light")]),
                                    ],
                                ),
                            ],
                        ),
                    ],
                )],
            ),
        ],
    )
}

fn ex_menu_submenu() -> Node {
    menu::root(
        Size::Md,
        OpenState::Open,
        vec![],
        vec![
            menu::trigger(
                OpenState::Open,
                false,
                Some("spec-menu-submenu-content"),
                vec![],
                vec![text("File")],
            ),
            menu::positioner(
                OpenState::Open,
                vec![],
                vec![menu::content(
                    OpenState::Open,
                    Some("spec-menu-submenu-content"),
                    None,
                    vec![],
                    vec![
                        menu::item("new", false, false, vec![], vec![text("New")]),
                        menu::trigger_item(
                            OpenState::Open,
                            false,
                            false,
                            Some("spec-menu-submenu-sub-content"),
                            vec![],
                            vec![
                                menu::item_text(false, false, vec![], vec![text("Share")]),
                                text("›"),
                            ],
                        ),
                        menu::positioner(
                            OpenState::Open,
                            vec![],
                            vec![menu::content(
                                OpenState::Open,
                                Some("spec-menu-submenu-sub-content"),
                                None,
                                vec![],
                                vec![
                                    menu::item("email", false, false, vec![], vec![text("Email")]),
                                    menu::item(
                                        "link",
                                        false,
                                        false,
                                        vec![],
                                        vec![text("Copy link")],
                                    ),
                                ],
                            )],
                        ),
                    ],
                )],
            ),
        ],
    )
}

fn ex_menu_inset_and_danger() -> Node {
    menu::root(
        Size::Md,
        OpenState::Open,
        vec![],
        vec![
            menu::trigger(
                OpenState::Open,
                false,
                Some("spec-menu-inset-danger-content"),
                vec![],
                vec![text("Account")],
            ),
            menu::positioner(
                OpenState::Open,
                vec![],
                vec![menu::content(
                    OpenState::Open,
                    Some("spec-menu-inset-danger-content"),
                    None,
                    vec![],
                    vec![
                        // アイコン/インジケータを持たない項目。
                        menu::item(
                            "profile",
                            false,
                            false,
                            vec![("data-inset", "")],
                            vec![text("Profile")],
                        ),
                        menu::item(
                            "settings",
                            false,
                            false,
                            vec![("data-inset", "")],
                            vec![text("Settings")],
                        ),
                        menu::separator(vec![], vec![]),
                        menu::item(
                            "delete-account",
                            false,
                            false,
                            vec![("data-danger", "")],
                            vec![text("Delete account")],
                        ),
                    ],
                )],
            ),
        ],
    )
}

/// `/themes/popover/`（Interactive カテゴリ）。
///
/// 一次情報: `crates/headless-ui/src/popover.rs`（モジュール doc・
/// `root`/`trigger`/`content` シグネチャ・`aria-haspopup="dialog"`/
/// `role="dialog"` の実出力テスト）。イシュー #2037（shadcn/ui 突合による
/// Examples 節「Dimensions form」追加）。
pub const POPOVER: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "トリガー起点のオーバーレイ。Root / Trigger / Anchor / Positioner / Arrow / ArrowTip / Content / Title / Description / CloseTrigger / Indicator の 11 anatomy パーツを持つ。",
        "開閉は Disclosure を埋め込んだ状態機械 Popover が管理する。",
        "content に role=\"dialog\" を固定付与し、title / description が設定されているときのみ aria-labelledby / aria-describedby をセットで付与する。",
        "shadcn/ui（Base UI）の With Form Example（Field/Input を内包する Content）は content の children へ fandhe-frontend-pre-styled-ui::field/input の既存 API を並べるだけの既存合成パターンであり、新規の variant/size/state 軸を追加しない（下記 Examples 節参照）。",
    ],
    arguments: &[ArgRow {
        name: "state",
        kind: "OpenState",
        default: "",
        description: "開閉状態（Open/Closed）。root/content の data-state へ反映される。",
    }],
    examples: &[ExampleEntry {
        title: "Dimensions form",
        description: "shadcn/ui（Base UI）の With Form Example に相当する、Field/Input を組み合わせたフォーム内包パターンです。バリデーション・送信処理は実装せず、fandhe-frontend-pre-styled-ui::field/input の既存 API のみで静的な入力欄を並べています。新しい variant や data-* 語彙は追加していません。",
        render: ex_popover_dimensions_form,
    }],
    keyboard: &[],
    aria: &[
        AriaRow {
            attribute: "aria-haspopup=\"dialog\"",
            description: "trigger に固定付与。",
        },
        AriaRow {
            attribute: "aria-expanded / aria-controls",
            description: "trigger に付与。開閉状態と対応する content の id を表す。",
        },
        AriaRow {
            attribute: "role=\"dialog\"",
            description: "content に固定付与。",
        },
        AriaRow {
            attribute: "aria-labelledby / aria-describedby",
            description: "content に付与。対応する title/description が設定されているときのみ出力される。",
        },
    ],
    demo: None,
};

/// [`POPOVER`] の Examples 節「Dimensions form」レンダラ（イシュー #2037）。
///
/// shadcn/ui（Base UI）Popover ページの With Form Example（`Field`/
/// `FieldGroup`/`FieldLabel`/`Input` を内包する Content）を、既存 API のみで
/// 再現できることを示す合成デモ。バリデーション・送信処理は一切実装しない
/// （`docs/policy/intentional-non-adoption.md` §3.25 規則 1）。Width/Max.
/// width/Height/Max. height の 4 フィールドを
/// [`FieldOrientation::Horizontal`] で横並びに配置し、値は静的リテラルの
/// `value` 属性のみで表す。Demo（[`crate::showcase::popover_section`]）と
/// 同じページに描画されるため、id は衝突しない
/// `showcase-popover-dimensions-*` を使う
/// （[`fandhe_frontend_pre_styled_ui::popover`] モジュール doc「shadcn/ui
/// 突合」節参照）。
fn ex_popover_dimensions_form() -> Node {
    let open = OpenState::Open;
    let content_id = "showcase-popover-dimensions-content";
    let title_id = "showcase-popover-dimensions-title";

    let dimension_field = |id: &'static str, label_text: &'static str, value: &'static str| {
        let field_props = FieldProps {
            id,
            ids: FieldIds::default(),
            disabled: false,
            invalid: false,
            required: false,
            readonly: false,
            has_helper_text: false,
        };
        field::root(
            &FieldRootProps {
                orientation: FieldOrientation::Horizontal,
            },
            &field_props,
            vec![],
            vec![
                field::label(&field_props, vec![], vec![text(label_text)]),
                input::input(&InputProps::default(), &field_props, vec![("value", value)]),
            ],
        )
    };

    popover::root(
        open,
        vec![],
        vec![
            popover::trigger(
                open,
                false,
                Some(content_id),
                vec![],
                vec![text("Edit dimensions")],
            ),
            popover::positioner(
                open,
                vec![],
                vec![popover::content(
                    open,
                    Some(content_id),
                    Some(title_id),
                    None,
                    vec![],
                    vec![
                        popover::title(Some(title_id), vec![], vec![text("Dimensions")]),
                        div(
                            vec![],
                            vec![
                                dimension_field(
                                    "showcase-popover-dimensions-width",
                                    "Width",
                                    "100%",
                                ),
                                dimension_field(
                                    "showcase-popover-dimensions-max-width",
                                    "Max. width",
                                    "300px",
                                ),
                                dimension_field(
                                    "showcase-popover-dimensions-height",
                                    "Height",
                                    "25px",
                                ),
                                dimension_field(
                                    "showcase-popover-dimensions-max-height",
                                    "Max. height",
                                    "none",
                                ),
                            ],
                        ),
                    ],
                )],
            ),
        ],
    )
}

/// `/themes/tabs/`（Interactive カテゴリ）。
///
/// 一次情報: `crates/pre-styled-ui/src/tabs.rs`（モジュール doc・`tabs`
/// シグネチャ）、`crates/headless-ui/src/tabs.rs`（`role="tablist"`/
/// `"tab"`/`"tabpanel"`・`aria-selected`・相互参照する `aria-controls`/
/// `aria-labelledby` の実出力テスト）。
pub const TABS: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "WAI-ARIA APG の Tabs パターン（role=\"tablist\"/\"tab\"/\"tabpanel\"、aria-selected、相互参照する aria-controls/aria-labelledby、roving tabindex）に準拠したマークアップを組み立てる。",
        "root / list / trigger / content の 4 パーツに加え、選択タブの位置を示す装飾パーツ indicator（opt-in）を持つ 5 パーツ構成。",
        "ActivationMode（Automatic/Manual）と Orientation（Horizontal/Vertical）を data-activation-mode / data-orientation として出力する。",
        "size / color-palette variant で root にクラスを付与する。",
        "data-orientation=\"vertical\" 時は list の下線を右罫線へ、trigger の下線を右側の強調線へ切り替えて縦並び表示する（イシュー #1542）。",
        "hover（背景・文字色）・disabled（data-disabled、半透明表示）・focus-visible（trigger/content 双方）を視覚的に反映する（イシュー #1542）。",
        "variant で下線スタイル（Line、既定）とセグメント/ピル型スタイル（Enclosed、shadcn/ui 既定 variant 相当。list を淡色の角丸コンテナに、選択中 trigger を白背景 + 微小な影で浮き上がらせる）を選べる（イシュー #2039）。",
    ],
    arguments: &[
        ArgRow {
            name: "variant",
            kind: "TabsVariant",
            default: "TabsVariant::Line",
            description: "root へ付与する見た目 variant（Line/Enclosed）。",
        },
        ArgRow {
            name: "size",
            kind: "Size",
            default: "Size::Md",
            description: "root へ付与するサイズ variant。",
        },
        ArgRow {
            name: "palette",
            kind: "ColorPalette",
            default: "",
            description: "root へ付与する配色 variant。",
        },
        ArgRow {
            name: "props",
            kind: "&TabsProps<'_>",
            default: "",
            description: "id/selected/orientation/activation_mode/loop_focus/indicator を束ねる SSR 静的設定値。",
        },
    ],
    examples: &[],
    keyboard: &[],
    aria: &[
        AriaRow {
            attribute: "role=\"tablist\" / aria-orientation",
            description: "list に付与。",
        },
        AriaRow {
            attribute: "role=\"tab\" / aria-selected / aria-controls",
            description: "trigger に付与。選択状態と対応する content の id を表す。",
        },
        AriaRow {
            attribute: "role=\"tabpanel\" / aria-labelledby",
            description: "content に付与。対応する trigger の id を指す。",
        },
    ],
    demo: None,
};

/// `/themes/toast/`（Interactive カテゴリ）。
///
/// 一次情報: `crates/pre-styled-ui/src/toast.rs`（モジュール doc・
/// `group`/`root` シグネチャ）、`crates/headless-ui/src/toast.rs`
/// （`role="region"`+`aria-label`・`role="status"`+`aria-live` の実出力
/// テスト）。
pub const TOAST: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "一時的な通知の queue 表示。group（live region）/ root（通知 1 件）/ title / description / action-trigger / close-trigger の 6 anatomy パーツを持つ。",
        "複数通知を有界なキューとして管理する状態機械 Toaster を提供する（Disclosure/SingleSelect のいずれにも写像できないため Component/Hydrate を直接実装する）。",
        "aria-live は ToastStatus から決定的に導出する（Error のみ assertive、他は polite）。aria-atomic=\"true\" を併用し通知全体を単位として読み上げさせる。",
        "placement（6 語彙、既定 BottomEnd）/ status（Info/Success/Warning/Error、既定 Info）の 2 軸 variant。",
    ],
    arguments: &[
        ArgRow {
            name: "placement",
            kind: "ToastPlacement",
            default: "ToastPlacement::BottomEnd",
            description: "group（live region）の表示位置（6 語彙）。",
        },
        ArgRow {
            name: "label",
            kind: "&str",
            default: "",
            description: "group の role=\"region\" に付与する aria-label（読み上げ用ラベル）。",
        },
        ArgRow {
            name: "status",
            kind: "ToastStatus",
            default: "ToastStatus::Info",
            description: "通知 1 件（root）の状態（Info/Success/Warning/Error）。aria-live の緊急度導出にも使われる。",
        },
    ],
    examples: &[ExampleEntry {
        title: "Description のみ（タイトルなし）",
        description: "title を省略し description のみで構成する合成パターン。既存 anatomy のみで再現でき、CSS 変更は不要（イシュー #2040、shadcn/ui 突合）。",
        render: ex_toast_description_only,
    }],
    keyboard: &[],
    aria: &[
        AriaRow {
            attribute: "role=\"region\" / aria-label",
            description: "group に付与。label 引数がそのまま aria-label になる。",
        },
        AriaRow {
            attribute: "role=\"status\" / aria-atomic=\"true\" / aria-live",
            description: "root に付与。aria-live は status から決定的に導出される（Error のみ assertive、他は polite）。",
        },
    ],
    demo: None,
};

/// `TOAST.examples` のレンダラ（イシュー #2040）。title を省略し
/// description のみで構成できることを示す合成パターン。`toast` モジュール
/// の既存公開 API のみで再現でき、新規 CSS は一切追加しない。
fn ex_toast_description_only() -> Node {
    toast::group(
        ToastPlacement::BottomEnd,
        "Notifications",
        vec![],
        vec![toast::root(
            ToastStatus::Success,
            vec![],
            vec![
                toast::description(vec![], vec![text("設定を保存しました。")]),
                toast::close_trigger(vec![("aria-label", "Close")], vec![text("×")]),
            ],
        )],
    )
}

/// `/themes/toggle-tip/`（Interactive カテゴリ）。
///
/// 一次情報: `crates/headless-ui/src/toggle_tip.rs`（モジュール doc・
/// `root`/`trigger` シグネチャ・`aria-expanded`/`aria-controls` の実出力
/// テストと `aria-haspopup`/`role="tooltip"` を付与しないことの記述）。
pub const TOGGLE_TIP: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "クリック開閉の小型ヒント。Root / Trigger / Positioner / Content / Arrow / ArrowTip の 6 anatomy パーツを持つ。",
        "見た目は Tooltip（小型・非モーダル）、挙動は Popover（クリックで開閉し明示的に閉じるまで持続）の変種として位置づけられる。",
        "trigger / content のいずれにも role=\"tooltip\" を付与しない（tooltip・popover 双方と異なる独自の ARIA 表現）。",
        "開閉は Disclosure を埋め込んだ状態機械 ToggleTip が管理する。",
    ],
    arguments: &[ArgRow {
        name: "state",
        kind: "OpenState",
        default: "",
        description: "開閉状態（Open/Closed）。root/content の data-state へ反映される。",
    }],
    examples: &[],
    keyboard: &[],
    aria: &[
        AriaRow {
            attribute: "aria-expanded",
            description: "trigger に付与。開閉状態を表す（aria-haspopup は付与しない）。",
        },
        AriaRow {
            attribute: "aria-controls",
            description: "trigger に付与。controls が Some のとき content の id を指す。",
        },
    ],
    demo: None,
};

/// `/themes/tooltip/`（Interactive カテゴリ）。
///
/// 一次情報: `crates/headless-ui/src/tooltip.rs`（モジュール doc・
/// `root`/`trigger`/`content` シグネチャ・`aria-describedby`/
/// `role="tooltip"` の実出力テスト）。イシュー #2041 で shadcn/ui
/// （Base UI）との突合を行い、`positioner` の `data-side` 属性と
/// kbd 併記の合成パターン（下記 Examples 節参照）を追記した。
pub const TOOLTIP: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "吹き出しヒント。Root / Trigger / Positioner / Content / Arrow / ArrowTip の 6 anatomy パーツを持つ。",
        "WAI-ARIA tooltip パターンに従い、trigger は aria-describedby で content と関連付ける（aria-expanded / aria-controls は使わない）。content 側が role=\"tooltip\" を持つ。",
        "openDelay / closeDelay（表示・非表示までの遅延タイマー）は wasm-full 側の後続スコープ。",
        "開閉は Disclosure を埋め込んだ状態機械 Tooltip が管理する。",
        "positioner は data-side 属性（top（既定）/ bottom / left / right）で表示位置を切り替えられる（イシュー #2041、shadcn/ui の side prop 相当。実座標追従ではなく静的フォールバックのみ）。",
        "content の children へテキストと fandhe-frontend-pre-styled-ui::kbd を組み合わせるキーボードショートカット併記パターンが可能（イシュー #2041、shadcn/ui の With Keyboard Shortcut Example 相当。下記 Examples 節参照）。",
    ],
    arguments: &[
        ArgRow {
            name: "state",
            kind: "OpenState",
            default: "",
            description: "開閉状態（Open/Closed）。root/content の data-state へ反映される。",
        },
        ArgRow {
            name: "positioner の attrs",
            kind: "Vec<(&str, &str)>",
            default: "",
            description: "positioner へ透過する属性。data-side=\"bottom\"/\"left\"/\"right\" を渡すと表示位置の静的フォールバックが切り替わる（未指定は top 相当）。left/right は root 幅が trigger 幅に一致する文脈（flex アイテム等で shrink-wrap される場合）でのみ trigger に隣接する位置になる。",
        },
    ],
    examples: &[ExampleEntry {
        title: "With keyboard shortcut",
        description: "shadcn/ui（Base UI）の With Keyboard Shortcut Example に相当する、content の children へテキストと kbd を組み合わせる合成パターンです。新しい variant や data-* 語彙を追加せず、既存の tooltip::content の自由な children と fandhe-frontend-pre-styled-ui::kbd の既存 API のみで構成しています（Ctrl+P のような複数 kbd 連結〔KbdGroup 相当〕は本リポジトリに対応するラッパーが無いため単一 kbd の再現に留めています）。",
        render: ex_tooltip_with_kbd,
    }],
    keyboard: &[],
    aria: &[
        AriaRow {
            attribute: "aria-describedby",
            description: "trigger に付与。content の id が設定されているときのみ出力される。",
        },
        AriaRow {
            attribute: "role=\"tooltip\"",
            description: "content に固定付与。",
        },
    ],
    demo: None,
};

/// [`TOOLTIP`] の Examples 節「With keyboard shortcut」レンダラ（イシュー
/// #2041）。
///
/// shadcn/ui（Base UI）Tooltip ページの With Keyboard Shortcut Example
/// （`apps/v4/examples/base/kbd-tooltip.tsx` の `Save Changes <Kbd>S</Kbd>`）
/// に相当する、`content` の children へテキストと [`kbd`] を組み合わせる
/// 合成パターンを、新しい variant/data-* を追加せず既存 API のみで再現する
/// （[`fandhe_frontend_pre_styled_ui::tooltip`] モジュール doc「イシュー
/// #2041 の shadcn/ui 突合」節参照）。Demo（[`crate::showcase::tooltip_section`]、
/// `showcase-tooltip-content`）と同じページに描画されるため、id は衝突
/// しない `showcase-tooltip-kbd-*` を使う。
fn ex_tooltip_with_kbd() -> Node {
    let open = OpenState::Open;
    let content_id = "showcase-tooltip-kbd-content";

    tooltip::root(
        open,
        vec![],
        vec![
            tooltip::trigger(open, false, Some(content_id), vec![], vec![text("Save")]),
            tooltip::positioner(
                open,
                vec![],
                vec![tooltip::content(
                    open,
                    Some(content_id),
                    vec![],
                    vec![
                        text("Save Changes "),
                        kbd(&KbdProps::default(), vec![], vec![text("S")]),
                    ],
                )],
            ),
        ],
    )
}

/// `/themes/tour/`（Interactive カテゴリ）。
///
/// 一次情報: `crates/pre-styled-ui/src/tour.rs`（モジュール doc・`root`
/// シグネチャ）、`crates/headless-ui/src/tour.rs`（`role="dialog"`/
/// `aria-labelledby`/`aria-describedby`・`progress_text` の
/// `aria-live="polite"` の実出力テスト）。
pub const TOUR: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "オンボーディング向けステップガイド。Root / Backdrop / Spotlight / Positioner / Arrow / ArrowTip / Content / Title / Description / ProgressText / Control / CloseTrigger / ActionTrigger の 13 anatomy パーツを持つ（イシュー #1666 で Control を追加。styled ラッパ・専用 CSS は Themes 側の後続）。",
        "open/closed の 2 値に加え skipped/completed という終端状態を持つ独自状態機械 Tour（Disclosure/SingleSelect のいずれにも写像できないため Component/Hydrate を直接実装する）。",
        "TourStep::target は DOM 解決を行わず data-target 属性としてエスケープ済みで出力するのみ（実座標追従は wasm-full 側の後続スコープ）。",
        "color-palette variant で root にクラスを付与する。",
    ],
    arguments: &[ArgRow {
        name: "palette",
        kind: "ColorPalette",
        default: "",
        description: "root へ付与する配色 variant。",
    }],
    examples: &[],
    keyboard: &[],
    aria: &[
        AriaRow {
            attribute: "role=\"dialog\" / aria-labelledby / aria-describedby",
            description: "content に付与。ContentIds で指定した title/description の id を指す。",
        },
        AriaRow {
            attribute: "aria-live=\"polite\"",
            description: "progress_text に固定付与。ステップ進捗テキストの更新を読み上げさせる。",
        },
    ],
    demo: None,
};
