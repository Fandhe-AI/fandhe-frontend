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
//! 対象は accordion・action-bar・dialog・drawer・floating-panel・
//! hover-card・menu・menubar・navigation-menu・popover・tabs・toast・
//! toggle-tip・toolbar・tooltip・tour の 16 部品（トリガー起点の
//! オーバーレイ、または項目開閉のディスクロージャ系。toolbar はイシュー
//! #991、menubar はイシュー #992、navigation-menu はイシュー #993 で追加、
//! いずれも `showcase.rs` の Demo 登録込み）。`toggle`/`toggle-group`
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
//! # `Examples` 節を持たない理由
//!
//! `docs/design/docs-site-component-pages.md` §7 は `Examples` を任意の節と
//! 定めており、本 PR では 13 定数すべて `examples: &[]` とする（節は自動的に
//! 省略される）。バリエーション軸を持つ部品（`Size`/`ColorPalette`/
//! `ToastStatus` 等）への Examples 追加はレビュー負荷を抑えるための
//! フォローアップ課題として PR 本文に残す。
//!
//! # セキュリティ不変条件（REQ-1）
//!
//! 本モジュールはリテラル `&'static str` のみで [`crate::component_page::ArgRow`]/
//! [`crate::component_page::AriaRow`]/[`crate::component_page::KeyRow`] を
//! 構築し、`raw_html()` や HTML 文字列の直接組み立てを一切行わない。
//! 実際のエスケープは `component_page.rs` 側の `table`/`td`/`text` ノード
//! 木経由で `render()` が行う（`features_and_table_cells_escape_xss_payloads`
//! が既存フィクスチャで固定済み）。

use crate::component_page::{ArgRow, AriaRow, ComponentPageSpec, KeyRow};

/// `/themes/accordion/`（Interactive カテゴリ）。
///
/// 一次情報: `crates/pre-styled-ui/src/accordion.rs`（モジュール doc・
/// `root` シグネチャ）、`crates/headless-ui/src/accordion.rs`（`aria_expanded`/
/// `aria_controls`/`role="region"` の実出力テスト）。
pub const ACCORDION: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "高々 1 項目が開く single モード（Accordion）と、複数項目が同時に開く multiple モード（MultiAccordion）の 2 状態機械を提供する。",
        "開いている項目の item-trigger / item-indicator を data-state=\"open\" に連動してハイライトする。",
        "size variant（Sm/Md/Lg）を root へ付与し、item-trigger / item-content の padding を切り替える。",
        "item-trigger はキーボード操作時のみのフォーカスリング（:focus-visible）を持つ。",
    ],
    arguments: &[ArgRow {
        name: "size",
        kind: "Size",
        default: "Size::Md",
        description: "root へ付与するサイズ variant（Sm/Md/Lg）。item-trigger/item-content の padding を切り替える。",
    }],
    examples: &[],
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

/// `/themes/action-bar/`（Interactive カテゴリ）。
///
/// 一次情報: `crates/headless-ui/src/action_bar.rs`（モジュール doc・
/// `root` シグネチャ・`role="toolbar"`/`role="separator"` の実出力テスト）。
pub const ACTION_BAR: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "複数選択操作バー（chakra-ui ActionBar 相当）。Root / Positioner / Content / SelectionTrigger / Separator / CloseTrigger の 6 anatomy パーツを持つ。",
        "開閉は Disclosure を埋め込んだ状態機械 ActionBar が管理する。選択件数から open を自動導出する糖衣 API は持たず、「選択操作 → 開閉状態の決定」は呼び出し側の責務とする。",
        "content に role=\"toolbar\" と aria-label（選択件数などの読み上げ用ラベル、呼び出し側が指定）を固定付与する。",
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
            attribute: "role=\"toolbar\"",
            description: "content に付与。aria-label とセットで固定付与される。",
        },
        AriaRow {
            attribute: "role=\"separator\"",
            description: "separator に付与。aria-orientation=\"vertical\" を伴う。",
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
        "複数 Menu を水平（または垂直）に並べるコンテナ。Root / Menu / Trigger / Positioner / Content / Item / ItemGroup / ItemGroupLabel / Separator / SubTrigger / SubContent の 11 anatomy パーツを持つ。",
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
    examples: &[],
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
        "トリガー起点で開閉するナビゲーションパネル。Root / List / Item / Trigger / Content / Link の 6 anatomy パーツを持つ。",
        "高々 1 個の Trigger だけが開く状態機械（SingleSelect を埋め込んだ NavigationMenu）。dispatch は \"select\"/\"toggle\"/\"deselect\"。",
        "role は一切付与しない。root は素の nav の暗黙 ARIA role（navigation）に依拠し、role=\"menu\"/role=\"menuitem\" は付与しない（文書ナビを操作メニューと誤伝達しないための設計、nav_list と同じ判断）。",
        "アクティブリンクは aria-current=\"page\" + data-current で表す（role は付与しない）。",
        "data-motion（アニメーション方向の露出）・viewport 寸法測定は実装しない（intentional-non-adoption.md §3.25 規則 2 により headless 層へ持ち込まない設計判断）。",
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
            description: "item/trigger/content の開閉状態（Open/Closed）。",
        },
        ArgRow {
            name: "current",
            kind: "bool",
            default: "false",
            description: "link に付与。true のとき aria-current=\"page\" + data-current を出力する。",
        },
    ],
    examples: &[],
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
    ],
    demo: None,
};

/// `/themes/dialog/`（Interactive カテゴリ）。
///
/// 一次情報: `crates/pre-styled-ui/src/dialog.rs`（モジュール doc・`root`
/// シグネチャ）、`crates/headless-ui/src/dialog.rs`（`aria-haspopup`/
/// `role="dialog"`・`role="alertdialog"`/`aria-modal` の実出力テスト）。
pub const DIALOG: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "Root / Trigger / Backdrop / Positioner / Content / Title / Description / CloseTrigger の 8 anatomy パーツを持つモーダルダイアログ。",
        "DialogRole（Dialog/AlertDialog）で role=\"dialog\"/role=\"alertdialog\" を出し分ける。",
        "size variant（Sm/Md/Lg）で root の寸法を切り替える。",
        "フォーカストラップ・Escape キーでの閉鎖・外側クリックでの閉鎖は JS ランタイム側の責務であり、本レイヤーは SSR/属性出力のみを担う。",
    ],
    arguments: &[
        ArgRow {
            name: "size",
            kind: "Size",
            default: "Size::Md",
            description: "root へ付与するサイズ variant（Sm/Md/Lg）。",
        },
        ArgRow {
            name: "state",
            kind: "OpenState",
            default: "",
            description: "開閉状態（Open/Closed）。root/content の data-state へ反映される。",
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
            description: "root へ付与するサイズ variant（Sm/Md/Lg）。",
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
    examples: &[],
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
pub const HOVER_CARD: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "リンク先プレビュー等、hover / focus で開閉するオーバーレイ。Root / Trigger / Positioner / Content / Arrow / ArrowTip の 6 anatomy パーツを持つ。",
        "trigger はリンク先プレビュー用途の a 要素であり、javascript: 等の危険スキームは fandhe-frontend-core の URL スキーム検証が除去する。",
        "HoverCardDelays（既定 open_ms: 600 / close_ms: 300）を data-open-delay / data-close-delay として root へ出力する。実際の hover/focus タイマー駆動は wasm-full 側の後続スコープ。",
        "WAI-ARIA に hover card 専用パターンは存在しないため、trigger へ aria-expanded / aria-controls / aria-haspopup は付与しない。",
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
    examples: &[],
    keyboard: &[],
    aria: &[AriaRow {
        attribute: "aria-hidden=\"true\"",
        description: "positioner に付与。閉じている間、支援技術から内容を隠す。",
    }],
    demo: None,
};

/// `/themes/menu/`（Interactive カテゴリ）。
///
/// 一次情報: `crates/pre-styled-ui/src/menu.rs`（モジュール doc・`root`
/// シグネチャ）、`crates/headless-ui/src/menu.rs`（`aria-haspopup="menu"`/
/// `role="menu"`/`role="menuitem"`/`role="group"` の実出力テスト）。
pub const MENU: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "トリガー起点のオーバーレイ + アクション項目リスト。Root / Trigger / Indicator / Positioner / Content / Arrow / ArrowTip / Item / ItemGroup / ItemGroupLabel / Separator / TriggerItem / ContextTrigger / CheckboxItem / RadioItemGroup / RadioItem の 16 anatomy パーツを持つ。",
        "サブメニューは親 Menu インスタンスの content 内に子 Menu インスタンス由来の trigger_item / positioner / content を入れ子で配置して表現し、親子双方に aria-haspopup=\"menu\" を付与する。",
        "CheckboxItem / RadioItemGroup は開閉状態とは独立した checked 状態機械（MenuCheckboxItem / MenuRadioItemGroup）を持つ。",
        "size variant で root/content の padding を切り替える。",
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
    examples: &[],
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

/// `/themes/popover/`（Interactive カテゴリ）。
///
/// 一次情報: `crates/headless-ui/src/popover.rs`（モジュール doc・
/// `root`/`trigger`/`content` シグネチャ・`aria-haspopup="dialog"`/
/// `role="dialog"` の実出力テスト）。
pub const POPOVER: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "トリガー起点のオーバーレイ。Root / Trigger / Anchor / Positioner / Arrow / ArrowTip / Content / Title / Description / CloseTrigger / Indicator の 11 anatomy パーツを持つ。",
        "開閉は Disclosure を埋め込んだ状態機械 Popover が管理する。",
        "content に role=\"dialog\" を固定付与し、title / description が設定されているときのみ aria-labelledby / aria-describedby をセットで付与する。",
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
    ],
    arguments: &[
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
    examples: &[],
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
/// `role="tooltip"` の実出力テスト）。
pub const TOOLTIP: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "吹き出しヒント。Root / Trigger / Positioner / Content / Arrow / ArrowTip の 6 anatomy パーツを持つ。",
        "WAI-ARIA tooltip パターンに従い、trigger は aria-describedby で content と関連付ける（aria-expanded / aria-controls は使わない）。content 側が role=\"tooltip\" を持つ。",
        "openDelay / closeDelay（表示・非表示までの遅延タイマー）は wasm-full 側の後続スコープ。",
        "開閉は Disclosure を埋め込んだ状態機械 Tooltip が管理する。",
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

/// `/themes/tour/`（Interactive カテゴリ）。
///
/// 一次情報: `crates/pre-styled-ui/src/tour.rs`（モジュール doc・`root`
/// シグネチャ）、`crates/headless-ui/src/tour.rs`（`role="dialog"`/
/// `aria-labelledby`/`aria-describedby`・`progress_text` の
/// `aria-live="polite"` の実出力テスト）。
pub const TOUR: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "オンボーディング向けステップガイド。Root / Backdrop / Spotlight / Positioner / Arrow / ArrowTip / Content / Title / Description / ProgressText / CloseTrigger / ActionTrigger の 12 anatomy パーツを持つ。",
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
