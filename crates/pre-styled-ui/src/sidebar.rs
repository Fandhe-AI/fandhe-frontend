//! styled Sidebar（shadcn/ui `Sidebar` 相当。イシュー #2073、親 #2071、
//! 祖父トラッキング参照軸 #2001。headless 側 anatomy は #2072）。
//!
//! `fandhe_frontend_headless_ui::sidebar`（#2072）が出力する
//! `data-scope="sidebar"` の 22 slot（`provider`/`root`/`header`/`content`/
//! `footer`/`separator`/`input`/`group`/`group-label`/`group-content`/
//! `group-action`/`menu`/`menu-item`/`menu-button`/`menu-action`/
//! `menu-badge`/`menu-sub`/`menu-sub-item`/`menu-sub-button`/`rail`/
//! `trigger`/`inset`）へ、`variant`（sidebar/floating/inset）3 種・
//! `collapsible`（offcanvas/icon/none）3 種・`side`（left/right）2 種・
//! モバイル時 drawer 表示という shadcn/ui `Sidebar` の意匠を重ねる薄い
//! 委譲層である（[`crate::command`]/[`crate::item`] と同型の位置付け）。
//!
//! # 選択的 re-export（`pub use ...::*` を使わない理由）
//!
//! [`crate::command`]/[`crate::item`] と同型。22 パーツすべてを同名再定義
//! し（呼び出し側 `class` の除去は本モジュールの責務のため）、状態機械
//! [`Sidebar`]/[`SidebarAction`]/[`SidebarState`] と静的 props 型
//! （[`SidebarProps`]/[`SidebarCollapsible`]/[`SidebarVariant`]/
//! [`SidebarSide`]/[`SidebarMenuButtonProps`]/[`SidebarMenuButtonSize`]/
//! [`SidebarMenuButtonVariant`]/[`SidebarMenuSubButtonProps`]/
//! [`SidebarMenuSubButtonSize`]/[`DATA_STATE_EXPANDED`]/
//! [`DATA_STATE_COLLAPSED`]）を選択的に再エクスポートする。[`provider`]/
//! [`root`]/[`rail`]/[`trigger`] が `&Sidebar` を引数に取るため
//! （[`crate::drawer`] が `DisclosureAction`/`OpenState` を再エクスポートする
//! のと同じ理由）、本モジュールは状態機械を再エクスポートしない
//! [`crate::command`] とは異なり、状態機械型を含めて再エクスポートする
//! （[`crate::drawer`] と同型の判断）。
//!
//! # 責務境界（`docs/policy/intentional-non-adoption.md` §3.25 規則 1）
//!
//! Cmd/Ctrl+B のグローバルショートカット・モバイル判定（メディアクエリ）・
//! `menu-button` の tooltip hover 配線はアプリケーション/
//! `fandhe-frontend-wasm-full`（後続イシュー #2074）の責務として実装しない。
//! headless が出力する `data-*` を CSS セレクタとして参照するだけで見た目を
//! 切り替える。本モジュール自身は独自の `data-*` を一切出力しない
//! （[`menu_skeleton`] を含む）。
//!
//! # `--fandhe-sidebar-*`（scope 接頭辞）と `--fandhe-color-sidebar-*`（色トークン）
//!
//! イシュータイトルの `--fandhe-sidebar-*` は色については
//! `docs/design/color-token-system.md` §9.3 の決定に従い
//! `--fandhe-color-sidebar-*`（[`crate::theme::Theme::default`] が追加する
//! `sidebar-bg`/`sidebar-fg`/`sidebar-accent`/`sidebar-accent-fg`/
//! `sidebar-muted`/`sidebar-border`/`sidebar-focus-ring` の 7 ロール）を
//! 意味する。幅など非色トークンは本 [`recipe`] が宣言する scope 接頭辞
//! custom property（`--fandhe-sidebar-width`/`--fandhe-sidebar-width-icon`/
//! `--fandhe-sidebar-width-mobile`、[`crate::drawer`] の
//! `--fandhe-drawer-size` と同型のフォールバック付き）である。
//!
//! # icon 折りたたみ時のテキスト非表示・長いラベルのクリップ
//! （`menu-button` の装飾用ラッパー `<span>` の `overflow: hidden`）
//!
//! headless [`fandhe_frontend_headless_ui::sidebar::menu_button`] はテキスト
//! を `span` で包まない（`children` をそのまま流し込む）ため、本 recipe の
//! [`menu_button`] 関数（本ファイル）が headless へ渡す前に `children`
//! （アイコン + ラベル）を装飾用の無印 `<span>`（`data-part`/`data-scope` を
//! 持たない、headless anatomy 外の pre-styled-ui 専用要素。headless 側の
//! anatomy 変更を伴わないため下記「スコープ外」節の判断と矛盾しない）で
//! 1 段包む。`data-collapsible="icon"` かつ折りたたみ時・長いラベルの
//! どちらも、`menu-button` 本体ではなくこのラッパー側の `overflow: hidden` +
//! `white-space: nowrap` + `min-width: 0`（[`stylesheet`] 内
//! `SIDEBAR_MENU_BUTTON_LABEL_SELECTOR` 規則）で視覚的に切り落とす（イシュー
//! #2073 レビュー〔codex-review P1・Cursor Bugbot Medium〕対応。旧実装は
//! `menu-button` 自身へ `overflow: hidden`/`clip-path` を適用しており、
//! ボタン本体の背景・クリック領域・`:focus-visible` の outline まで一緒に
//! 失われる regression があった）。アイコンを `children` の最初の要素に置く
//! 呼び出し規約を前提とする。`justify-content: center` は使わない
//! （codex-review P1 指摘、[`stylesheet`] 内コメント参照）: ラッパー内で
//! children 全体をまとめて中央寄せすると、ラベルが長い場合にアイコン自体が
//! ラッパーの `overflow: hidden` の外側へ押し出されて切れてしまう。
//!
//! # モバイル + collapsed の詳細度調整（幅を固定し `transform` のみで開閉する）
//!
//! [`root`] の折りたたみ幅規則（`AttrEqAll([("data-state","collapsed"),
//! ("data-collapsible","offcanvas")])`/`AttrEqAll([("data-state","collapsed"),
//! ("data-collapsible","icon")])`、属性 4 個）は、`data-mobile` 単独条件
//! （属性 3 個）より詳細度が高いため、モバイル + collapsed ではこれらが
//! 優先されて「幅の増減」になってしまう。[`stylesheet`] が追記する raw CSS
//! （`[data-mobile][data-state="collapsed"]`、属性 4 個 + ソース順で後）が
//! 幅を `--fandhe-sidebar-width-mobile` に固定し、遷移対象を `transform`
//! のみにする（`offcanvas`/`icon` を問わずモバイル時は drawer 挙動に統一、
//! shadcn/ui 同様）。
//!
//! # backdrop を追加しない理由
//!
//! headless anatomy に `backdrop` パーツが無いため（headless モジュール doc
//! 参照）、本モジュールも追加しない（[`crate::command`] の `dialog` と同じ
//! 判断、下記「スコープ外」節参照）。
//!
//! # `menu-skeleton`（ローディング装飾、[`menu_skeleton`]）
//!
//! headless モジュール doc「`menu-skeleton` は本モジュールに置かない」節の
//! 申し送りどおり、本モジュールが [`menu_skeleton`] ヘルパを提供する
//! （`docs/policy/intentional-non-adoption.md` §3.25 規則 2: 装飾は Themes
//! 層の責務）。shadcn/ui のランダム幅実装は SSR 決定性を壊すため採用せず、
//! [`crate::skeleton::skeleton`]（`Circle`〔アイコン用、`show_icon` が
//! `true` のときのみ〕+ `Text`）を固定幅で合成する決定的な
//! [`fandhe_frontend_headless_ui::sidebar::menu_item`] を返す。幅の調整は
//! 呼び出し側が `--fandhe-skeleton-size` 等の CSS 変数で行う（意図的な
//! shadcn との差分）。自前の `data-*` は出力しない。
//!
//! # セキュリティ不変条件
//!
//! - 全出力は headless [`fandhe_frontend_headless_ui::sidebar`] →
//!   [`fandhe_frontend_core::render`] の既定エスケープ（REQ-1）を必ず
//!   経由する。`raw_html()` は使用しない。
//! - 呼び出し側 `class` は [`drop_class_attr`] で除去してから headless
//!   関数へ委譲する（22 パーツすべて）。
//! - [`stylesheet`] が組み立てる CSS 宣言・selector 断片はすべて
//!   コンパイル時静的リテラルであり、[`crate::css::decl`]/
//!   [`crate::css::serialize_rule`] の検証を通る値のみを使う。
//! - `data-scope`/`data-part` の呼び出し側偽装は headless
//!   [`fandhe_frontend_headless_ui::anatomy::Anatomy::part`] が除去する。
//!
//! # スコープ外
//!
//! - **#2074（`fandhe-frontend-wasm-full` 配線）**: Cmd/Ctrl+B・モバイル
//!   drawer 切替・`menu-button` の tooltip hover 配線（上記「責務境界」節
//!   参照）。
//! - **`/themes/sidebar/` の docs-site ページ・showcase Demo・`site/nav.toml`
//!   登録**: #2075。`crates/docs-site/tests/wrap_state.rs` の
//!   `THEMES_RECIPE_WITHOUT_PAGE` 暫定台帳が本イシューと #2075 の橋渡しを
//!   担う。
//! - **`backdrop` パーツの新設**: headless anatomy の変更を伴うため本
//!   イシューでは追随しない（上記「backdrop を追加しない理由」節参照）。
//! - **`menu-button` テキストのアイコン折りたたみ時の非表示を headless 側
//!   （`fandhe_frontend_headless_ui::sidebar::menu_button` の anatomy）で
//!   span 化する案**: 本イシューでは pre-styled-ui 層（[`menu_button`]、
//!   本ファイル）が装飾用ラッパー `<span>` を追加する方式に留め、headless
//!   anatomy 自体は変更しない（上記「icon 折りたたみ時のテキスト非表示・
//!   長いラベルのクリップ」節参照。イシュー #2073 レビュー対応でラッパー
//!   追加へ変更したのは pre-styled-ui 層内で完結する範囲であり、この
//!   スコープ外判断自体は維持している）。
//! - **`@media (min-width)` 対応**: breakpoint 機構
//!   （イシュー #2196/#2197）が未実装のため、レスポンシブなレイアウト
//!   切り替えは対象外。

use crate::class_attr::drop_class_attr;
use crate::css::{decl, serialize_rule};
use crate::recipe::{
    focus_ring_declarations, transition_declarations, FocusRingColor, FocusRingOffset,
    MotionDuration, SlotRecipe, StateCondition,
};
use crate::skeleton::{skeleton, SkeletonAnimation, SkeletonProps, SkeletonVariant};
use fandhe_frontend_headless_ui::fandhe_frontend_core::{el, Node};

// `provider`/`root`/`rail`/`trigger` が `&Sidebar` を引数に取るため、状態
// 機械型を選択的に再エクスポートする（モジュール doc「選択的 re-export」節、
// [`crate::drawer`] と同型の判断）。
pub use fandhe_frontend_headless_ui::sidebar::{
    Sidebar, SidebarAction, SidebarCollapsible, SidebarMenuButtonProps, SidebarMenuButtonSize,
    SidebarMenuButtonVariant, SidebarMenuSubButtonProps, SidebarMenuSubButtonSize, SidebarProps,
    SidebarSide, SidebarState, SidebarVariant, DATA_STATE_COLLAPSED, DATA_STATE_EXPANDED,
};

/// headless `sidebar` anatomy の `data-part` 一覧（`crates/headless-ui/src/
/// sidebar.rs` の `ANATOMY.part(...)` 呼び出しと同期させる契約。ずれると
/// [`stylesheet`] が一部パーツの CSS を出力しない fail-closed 側の不具合と
/// して現れるため、変更時は両ファイルを合わせて確認する）。
const SLOTS: &[&str] = &[
    "provider",
    "root",
    "header",
    "content",
    "footer",
    "separator",
    "input",
    "group",
    "group-label",
    "group-content",
    "group-action",
    "menu",
    "menu-item",
    "menu-button",
    "menu-action",
    "menu-badge",
    "menu-sub",
    "menu-sub-item",
    "menu-sub-button",
    "rail",
    "trigger",
    "inset",
];

/// [`menu_button`] が children を包む装飾用ラッパー `<span>`
/// （`data-part`/`data-scope` を持たない、headless anatomy 外の
/// pre-styled-ui 専用要素）を選択する CSS セレクタ。`menu-button` の
/// 唯一の直下子であるため子結合子 1 段で一意に特定できる（イシュー
/// #2073 レビュー対応、[`stylesheet`] 内コメント「長いラベルをガター
/// の手前でクリップする」節参照）。
const SIDEBAR_MENU_BUTTON_LABEL_SELECTOR: &str =
    r#"[data-scope="sidebar"][data-part="menu-button"] > span"#;

/// この styled Sidebar の既定 CSS を組み立てる（内部ヘルパ、[`stylesheet`]
/// のみが呼ぶ）。
// 折りたたみ時（`visibility: hidden` を併用する各 state 規則、`recipe()`
// の offcanvas collapsed `.state()` と `stylesheet()` の mobile collapsed
// `push()` の双方から参照される）が `visibility` の切り替わりを
// `width`/`transform` の遷移完了まで遅延させるための共通 `transition-delay`
// 上書き（`recipe()` 内 `root_transition` 定義部の doc 参照。`width`/
// `transform` の `delay` は base の `0s` のまま、`visibility` のみ
// `duration-normal` 分だけ遅延する）。`recipe()`/`stylesheet()` は別関数
// のためローカル変数を跨いで共有できず、モジュールスコープの `const` として
// 定義する（`decl()` は `const fn`）。
const ROOT_COLLAPSED_VISIBILITY_DELAY: crate::css::Declaration = decl(
    "transition-delay",
    "0s, 0s, var(--fandhe-motion-duration-normal)",
);

fn recipe() -> SlotRecipe {
    let provider_base = vec![
        decl("display", "flex"),
        decl("min-height", "100svh"),
        decl("width", "100%"),
    ];

    let root_base = vec![
        decl("position", "relative"),
        decl("display", "flex"),
        decl("flex-direction", "column"),
        decl("box-sizing", "border-box"),
        decl("width", "var(--fandhe-sidebar-width, 16rem)"),
        decl("flex-shrink", "0"),
        decl("background", "var(--fandhe-color-sidebar-bg)"),
        decl("color", "var(--fandhe-color-sidebar-fg)"),
        decl(
            "border-inline-end",
            "1px solid var(--fandhe-color-sidebar-border)",
        ),
    ];
    // `root` の遷移対象は `width`/`transform` のみだったため、折りたたみ時に
    // 併用する `visibility: hidden`（offcanvas/mobile collapsed 規則参照）が
    // 即座に適用され、`width`/`transform` のアニメーションが完了する前に
    // 視覚的に消えてしまい閉じる動きが見えない（codex-review P2 指摘
    // 「閉じるアニメーションの終了まで可視性を維持する」/Cursor Bugbot
    // Medium 指摘「Collapse hides before animating」対応）。`visibility` は
    // 離散プロパティであり、遷移の値切り替わりは
    // `transition-delay + transition-duration`（= 100% 地点）で発生する
    // （[CSS Transitions] discrete animation type の仕様）。ここでは
    // `visibility` 自身の `duration` を `0s` に固定し、`delay` だけで
    // 切り替わりタイミングを制御する: 展開時（本 base 規則、`delay: 0s`）は
    // 即座に `visible` へ戻り、折りたたみ時（各 collapsed state 規則が
    // `transition-delay` を `width`/`transform` と同じ `duration-normal` へ
    // 上書き）は `width`/`transform` の遷移完了と同時に `hidden` へ切り替わる
    // （`transition-duration`/`-property`/`-timing-function` は longhand の
    // ままなので、collapsed state 側は `transition-delay` のみを再宣言すれば
    // 他のロングハンド値はカスケードで本 base 規則から引き継がれる）。
    let root_transition = vec![
        decl("transition-property", "width, transform, visibility"),
        decl(
            "transition-duration",
            "var(--fandhe-motion-duration-normal), var(--fandhe-motion-duration-normal), 0s",
        ),
        decl(
            "transition-timing-function",
            "var(--fandhe-motion-easing-standard), var(--fandhe-motion-easing-standard), linear",
        ),
        decl("transition-delay", "0s, 0s, 0s"),
    ];
    // 折りたたみ時の `transition-delay` 上書き値は `ROOT_COLLAPSED_VISIBILITY_DELAY`
    // （モジュールスコープの `const`、`recipe()`/`stylesheet()` 双方から
    // 参照するための定義、本モジュール冒頭の doc 参照）。

    let header_footer_base = vec![
        decl("display", "flex"),
        decl("flex-direction", "column"),
        decl("gap", "var(--fandhe-space-2)"),
        decl("padding", "var(--fandhe-space-2)"),
    ];

    let content_base = vec![
        decl("flex", "1"),
        decl("min-height", "0"),
        decl("overflow", "auto"),
        decl("display", "flex"),
        decl("flex-direction", "column"),
        decl("gap", "var(--fandhe-space-2)"),
    ];

    let separator_base = vec![
        decl("margin", "0 var(--fandhe-space-2)"),
        decl("border", "0"),
        decl("border-top", "1px solid var(--fandhe-color-sidebar-border)"),
    ];

    let input_base = vec![
        decl("display", "block"),
        decl("width", "100%"),
        decl("box-sizing", "border-box"),
        decl("height", "var(--fandhe-size-control-height-sm, 2rem)"),
        decl("padding", "0 var(--fandhe-space-2)"),
        decl("border", "1px solid var(--fandhe-color-sidebar-border)"),
        decl("border-radius", "var(--fandhe-radius-md)"),
        decl("background", "var(--fandhe-color-sidebar-bg)"),
        decl("color", "inherit"),
        decl("font", "inherit"),
        decl("font-size", "var(--fandhe-font-font-size-sm)"),
        decl("outline", "none"),
    ];

    let group_base = vec![
        decl("position", "relative"),
        decl("display", "flex"),
        decl("flex-direction", "column"),
        decl("padding", "var(--fandhe-space-2)"),
    ];

    let group_label_base = vec![
        decl("display", "flex"),
        decl("align-items", "center"),
        decl("height", "2rem"),
        decl("padding", "0 var(--fandhe-space-2)"),
        decl("font-size", "var(--fandhe-font-font-size-xs)"),
        decl("font-weight", "var(--fandhe-font-font-weight-medium)"),
        decl("color", "var(--fandhe-color-sidebar-fg)"),
        decl("opacity", "0.7"),
    ];

    let group_content_base = vec![
        decl("display", "flex"),
        decl("flex-direction", "column"),
        decl("gap", "var(--fandhe-space-1)"),
    ];

    let group_action_base = vec![
        decl("position", "absolute"),
        decl("inset-inline-end", "var(--fandhe-space-2)"),
        decl("top", "var(--fandhe-space-1)"),
        decl("width", "1.25rem"),
        decl("height", "1.25rem"),
        decl("display", "inline-flex"),
        decl("align-items", "center"),
        decl("justify-content", "center"),
        decl("border", "0"),
        decl("background", "transparent"),
        decl("border-radius", "var(--fandhe-radius-sm)"),
        decl("color", "inherit"),
        decl("cursor", "pointer"),
    ];

    let menu_base = vec![
        decl("list-style", "none"),
        decl("margin", "0"),
        decl("padding", "0"),
        decl("display", "flex"),
        decl("flex-direction", "column"),
        decl("gap", "var(--fandhe-space-1)"),
    ];

    let menu_item_base = vec![decl("position", "relative")];

    let menu_button_base = vec![
        // `display: flex` + `align-items: center` は [`menu_button`]（本
        // ファイル）が children を包む唯一の直下子（装飾用ラッパー
        // `<span>`、下記「長いラベルをガターの手前でクリップする」節参照）
        // を垂直中央に配置するためのもの。アイコン・ラベル間の `gap` は
        // ラッパー側（`SIDEBAR_MENU_BUTTON_LABEL_SELECTOR` 規則）が持つ
        // （子が 1 個のみのため `menu-button` 自身の `gap` は不要）。
        decl("display", "flex"),
        decl("align-items", "center"),
        decl("width", "100%"),
        decl("box-sizing", "border-box"),
        decl("height", "2rem"),
        decl("padding-block", "0"),
        decl("padding-inline-start", "var(--fandhe-space-2)"),
        // `menu-action`/`menu-badge` は `menu-button` と同じ `menu-item` 内で
        // 絶対配置される兄弟パーツ（`menu_action_base`/`menu_badge_base` の
        // コメント参照）であり、`SlotRecipe` は `:has()`/兄弟結合子を持たない
        // （`crate::button_group`/`crate::card` と同型の制約）ため、
        // `menu-button` 自身からは兄弟の有無を条件分岐できない。無条件に
        // `menu-action`/`menu-badge` 1 個分（幅 1.25rem + 右端オフセット
        // `--fandhe-space-2` + 隙間 `--fandhe-space-1`）の余白を予約して
        // ラベルの折り返し・切り落とし位置をずらし、長いラベルが
        // アクション/バッジと重なるのを防ぐ（codex-review P2 指摘対応）。
        decl(
            "padding-inline-end",
            "calc(var(--fandhe-space-2) + 1.25rem + var(--fandhe-space-1))",
        ),
        decl("border", "0"),
        decl("border-radius", "var(--fandhe-radius-md)"),
        decl("background", "transparent"),
        decl("color", "inherit"),
        decl("font", "inherit"),
        decl("font-size", "var(--fandhe-font-font-size-sm)"),
        decl("text-align", "start"),
        decl("text-decoration", "none"),
        decl("cursor", "pointer"),
    ];
    let menu_button_transition = transition_declarations("background, color", MotionDuration::Fast);

    let menu_action_base = vec![
        decl("position", "absolute"),
        decl("inset-inline-end", "var(--fandhe-space-2)"),
        // `menu-item`（`position: relative` の基準）は `menu-sub` を子に
        // 含みうるため、その合計高さに対する `top: 50%` では親
        // `menu-button` の外へずれる（codex-review P1 指摘）。`menu-button`
        // 自身の高さ（`menu_button_base` の `height: 2rem`）の半分を絶対値
        // で固定し、`menu-button` 単体の垂直中央に位置を保証する。
        decl("top", "1rem"),
        decl("transform", "translateY(-50%)"),
        decl("width", "1.25rem"),
        decl("height", "1.25rem"),
        decl("display", "inline-flex"),
        decl("align-items", "center"),
        decl("justify-content", "center"),
        decl("border", "0"),
        decl("background", "transparent"),
        decl("border-radius", "var(--fandhe-radius-sm)"),
        decl("color", "inherit"),
        decl("cursor", "pointer"),
    ];

    // `menu-badge` は `menu-item`（`<li>`、`menu_item_base` は
    // `position: relative` のみで flex コンテナではない）内で `menu-button`
    // の兄弟として配置される（headless `crates/headless-ui/src/sidebar.rs`
    // 参照）ため、`margin-inline-start: auto` は flex コンテナの子でなければ
    // 効果を持たず右寄せされない（codex-review P2 / Bugbot 指摘）。
    // `menu-action`/`group-action` と同型の絶対配置（`menu-item` の
    // `position: relative` を基準）へ切り替え、`menu-item` 側の変更なしに
    // 常に右端へ固定する。
    let menu_badge_base = vec![
        decl("position", "absolute"),
        decl("inset-inline-end", "var(--fandhe-space-2)"),
        // `menu_action_base` と同じ理由（`menu-item` が `menu-sub` を
        // 子に含みうるため `top: 50%` は使わない）で `menu-button` の
        // 高さ（2rem）の半分を絶対値で固定する。
        decl("top", "1rem"),
        decl("transform", "translateY(-50%)"),
        decl("font-size", "var(--fandhe-font-font-size-xs)"),
        decl("padding", "0 var(--fandhe-space-1)"),
        decl("border-radius", "var(--fandhe-radius-md)"),
        decl("min-width", "1.25rem"),
        decl("text-align", "center"),
        // `menu-badge` は件数表示のみの装飾用 span であり、`menu-button`
        // と同じ `menu-item` 内で絶対配置により右端へ重なる。
        // `pointer-events` を明示しないと既定値 `auto` のままクリックを
        // 奪い、兄弟の `menu-button` へイベントが伝播せず件数部分の
        // クリックで遷移・操作ができなくなる（codex-review P1 指摘）。
        decl("pointer-events", "none"),
    ];

    let menu_sub_base = vec![
        decl("list-style", "none"),
        decl("margin", "0"),
        decl("margin-inline-start", "var(--fandhe-space-3)"),
        decl("padding", "0"),
        decl("padding-inline-start", "var(--fandhe-space-2)"),
        decl(
            "border-inline-start",
            "1px solid var(--fandhe-color-sidebar-border)",
        ),
        decl("display", "flex"),
        decl("flex-direction", "column"),
        decl("gap", "var(--fandhe-space-1)"),
    ];

    let menu_sub_item_base = vec![decl("position", "relative")];

    let menu_sub_button_base = vec![
        decl("display", "flex"),
        decl("align-items", "center"),
        decl("gap", "var(--fandhe-space-2)"),
        decl("width", "100%"),
        decl("box-sizing", "border-box"),
        decl("height", "1.75rem"),
        decl("padding", "0 var(--fandhe-space-2)"),
        decl("border", "0"),
        decl("border-radius", "var(--fandhe-radius-md)"),
        decl("background", "transparent"),
        decl("color", "inherit"),
        decl("font", "inherit"),
        decl("font-size", "var(--fandhe-font-font-size-xs)"),
        decl("text-align", "start"),
        decl("text-decoration", "none"),
        decl("cursor", "pointer"),
        decl("overflow", "hidden"),
        decl("white-space", "nowrap"),
    ];

    let rail_base = vec![
        decl("position", "absolute"),
        decl("inset-block", "0"),
        decl("inset-inline-end", "-1rem"),
        decl("width", "1rem"),
        decl("border", "0"),
        decl("padding", "0"),
        decl("background", "transparent"),
        decl("cursor", "ew-resize"),
        // `rail` は `root`（`-1rem` オフセットで `root` のボーダーボックス外へ
        // はみ出す）の子孫であり `z-index` を持たない `position: absolute`
        // （`z-index: auto`）のため、`root`/`inset` がいずれも `z-index` を
        // 明示しない flex アイテム同士の場合、祖先の重ね順文脈次第では
        // 後続の flex 兄弟 `inset` が `rail` の描画領域を覆い、折りたたみ
        // グリップがクリック不能になる（Cursor Bugbot High 指摘「Rail
        // toggle sits under inset」対応）。`z-index: 1` を明示し、`inset`
        // （`z-index` 未指定 = `auto`）より確実に手前へ描画させる
        // （`crate::avatar` の badge ローカル z-index と同型の最小限定数）。
        decl("z-index", "1"),
    ];

    let trigger_base = vec![
        decl("display", "inline-flex"),
        decl("align-items", "center"),
        decl("justify-content", "center"),
        decl("width", "1.75rem"),
        decl("height", "1.75rem"),
        decl("border", "0"),
        decl("background", "transparent"),
        decl("border-radius", "var(--fandhe-radius-md)"),
        decl("color", "inherit"),
        decl("cursor", "pointer"),
    ];

    let inset_base = vec![
        decl("flex", "1"),
        decl("min-width", "0"),
        decl("display", "flex"),
        decl("flex-direction", "column"),
        decl("background", "var(--fandhe-color-bg)"),
    ];

    SlotRecipe::new("sidebar", SLOTS)
        .base("provider", provider_base)
        .base("root", root_base)
        .base("root", root_transition)
        .base("header", header_footer_base.clone())
        .base("content", content_base)
        .base("footer", header_footer_base)
        .base("separator", separator_base)
        .base("input", input_base)
        .base("group", group_base)
        .base("group-label", group_label_base)
        .base("group-content", group_content_base)
        .base("group-action", group_action_base)
        .base("menu", menu_base)
        .base("menu-item", menu_item_base)
        .base("menu-button", menu_button_base)
        .base("menu-button", menu_button_transition)
        .base("menu-action", menu_action_base)
        .base("menu-badge", menu_badge_base)
        .base("menu-sub", menu_sub_base)
        .base("menu-sub-item", menu_sub_item_base)
        .base("menu-sub-button", menu_sub_button_base)
        .base("rail", rail_base)
        .base("trigger", trigger_base)
        .base("inset", inset_base)
        // `root` の `data-side="right"` 反転（`rail` の位置反転は raw CSS）。
        .state(
            "root",
            StateCondition::AttrEq("data-side", "right"),
            vec![
                decl("order", "1"),
                decl("border-inline-end", "0"),
                decl(
                    "border-inline-start",
                    "1px solid var(--fandhe-color-sidebar-border)",
                ),
            ],
        )
        // `data-variant="floating"`: 浮遊する見た目。
        .state(
            "root",
            StateCondition::AttrEq("data-variant", "floating"),
            vec![
                decl("margin", "var(--fandhe-space-2)"),
                decl("border", "1px solid var(--fandhe-color-sidebar-border)"),
                decl("border-radius", "var(--fandhe-radius-lg)"),
                decl("box-shadow", "var(--fandhe-shadow-md)"),
                decl("height", "calc(100svh - var(--fandhe-space-4))"),
            ],
        )
        // `data-variant="inset"`: ページ背景に溶ける（`inset` パーツ側が
        // 面パネルになる、raw CSS 追記「inset variant の主領域」節参照）。
        .state(
            "root",
            StateCondition::AttrEq("data-variant", "inset"),
            vec![
                decl("margin", "var(--fandhe-space-2)"),
                decl("border", "0"),
                decl("background", "transparent"),
            ],
        )
        // 折りたたみ幅（icon）。`root` は `provider`（`display: flex`）の
        // flex アイテムであり、`min-width` は既定 `auto`（子孫の
        // 縮小不能な最小コンテンツ幅）のままだと header/footer 等の
        // テキストが `width` の縮小を無効化しレールへ折りたたまれない
        // （Bugbot 指摘「Icon collapse ignores content min-width」対応）。
        // `min-width` を `width` と同じトークンへ固定し、flexbox の
        // 既定縮小抑制を明示的に上書きする。`header`/`footer` 内のテキスト
        // クリップは `overflow-x: hidden` を `root` 自身へ持たせず、
        // `stylesheet()` が追記する raw CSS（`ICON_COLLAPSED > header`/
        // `> footer` の子結合子セレクタ）で `header`/`footer` パーツへ
        // 個別に付与する（codex-review P1 / Cursor Bugbot 指摘「Icon
        // collapse clips the rail」対応: `rail` パーツは `root` に対する
        // `position: absolute` + `inset-inline-end: -1rem` で `root` の
        // ボーダーボックス外に配置される開閉トグルであり、`root` に
        // `overflow-x: hidden` を付けると `rail` の操作領域自体が
        // クリップされ、折りたたみ後に再展開する手段が失われてしまう。
        // `content` は既に自前で `overflow: auto`（`content_base`）を
        // 持つため対象外でよい）。
        .state(
            "root",
            StateCondition::AttrEqAll(&[("data-state", "collapsed"), ("data-collapsible", "icon")]),
            vec![
                decl("width", "var(--fandhe-sidebar-width-icon, 3rem)"),
                decl("min-width", "var(--fandhe-sidebar-width-icon, 3rem)"),
            ],
        )
        // 折りたたみ幅（offcanvas）。`visibility: hidden` を併用する
        // （codex-review P1 指摘: `width: 0` + `overflow: hidden` だけでは
        // 子孫のリンク・入力・ボタンが Tab 順序・アクセシビリティツリーに
        // 残り続ける。headless 側は `hidden`/`inert` 属性を持たない
        // 契約〔モジュール doc「責務境界」節〕のため、pre-styled-ui 側の
        // CSS で操作対象から除外する。`visibility: hidden` は子孫が
        // `visibility: visible` を再宣言しない限り操作・読み上げ対象から
        // 外れ、`display: none` と異なりレイアウト崩れ〔幅 0 は既に
        // 別宣言で保証済み〕を伴わない）。
        .state(
            "root",
            StateCondition::AttrEqAll(&[
                ("data-state", "collapsed"),
                ("data-collapsible", "offcanvas"),
            ]),
            vec![
                decl("width", "0"),
                decl("border", "0"),
                decl("overflow", "hidden"),
                decl("visibility", "hidden"),
                // `root_transition`（base）の `visibility` 遷移 delay（`0s`）を
                // ここで `duration-normal` へ上書きし、`width`/`transform` の
                // 遷移が終わるまで `visibility: hidden` への切り替わりを
                // 遅延させる（codex-review P2 / Cursor Bugbot Medium 指摘
                // 「閉じるアニメーションの終了まで可視性を維持する」対応、
                // `root_transition` 定義部の doc 参照）。
                ROOT_COLLAPSED_VISIBILITY_DELAY,
                // `floating`/`inset` variant の `margin`（`var(--fandhe-
                // space-2)`）が残ったままだと `width: 0` でも `root` が
                // margin 分の領域を占め続け、main エリアが全幅を取れない
                // （Bugbot 指摘「Offcanvas collapse keeps variant margin」
                // 対応）。属性セレクタ 4 個は variant state（属性 1 個）
                // より詳細度が高いため、ここで `margin: 0` を上書きすれば
                // variant を問わず折りたたみ時は必ず margin が消える。
                decl("margin", "0"),
            ],
        )
        // `group-action`/`menu-action`: hover で背景・フォーカスリング。
        .state(
            "group-action",
            StateCondition::Hover,
            vec![decl("background", "var(--fandhe-color-sidebar-muted)")],
        )
        .state(
            "group-action",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Outside),
        )
        .state(
            "menu-action",
            StateCondition::Hover,
            vec![decl("background", "var(--fandhe-color-sidebar-muted)")],
        )
        .state(
            "menu-action",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Outside),
        )
        // `menu-button`: `data-variant="outline"` は選択（`data-active`）と
        // 同じ specificity（属性セレクタ 1 個分）の単一属性条件のため、
        // ソース順で後に登録した方が背景・文字色を上書きする。outline の
        // 既定背景を先に登録し、選択色（`data-active`）を後段へ置くことで
        // active な menu-button は常に選択色（背景・文字色）を保つ
        // （codex-review P1 指摘: outline+active で白背景に白文字化する
        // 不具合の是正。outline の `box-shadow` は active 側で上書きしない
        // ため境界線表現は維持される）。
        .state(
            "menu-button",
            StateCondition::AttrEq("data-variant", "outline"),
            vec![
                decl("background", "var(--fandhe-color-bg)"),
                decl("box-shadow", "0 0 0 1px var(--fandhe-color-sidebar-border)"),
            ],
        )
        .state(
            "menu-button",
            StateCondition::AttrEq("data-size", "sm"),
            vec![
                decl("height", "1.75rem"),
                decl("font-size", "var(--fandhe-font-font-size-xs)"),
            ],
        )
        .state(
            "menu-button",
            StateCondition::AttrEq("data-size", "lg"),
            vec![
                decl("height", "3rem"),
                // `padding-inline`（両端一括指定）は使わない（Bugbot 指摘
                // 「lg padding-inline overwrites action/badge gutter」）:
                // `menu_button_base` の `padding-inline-end` は
                // `menu-action`/`menu-badge` 1 個分の重なり回避ガター
                // （上記コメント参照）を予約しており、この state は
                // `menu_button_base` と同じ属性セレクタ 1 個分の specificity
                // でソース順のみが後段のため一括指定するとガター予約ごと
                // 上書きしてしまう。開始側だけを lg 用の余白へ広げ、終了側は
                // 同じガター予約式を lg のトークンで再宣言し維持する。
                decl("padding-inline-start", "var(--fandhe-space-3)"),
                decl(
                    "padding-inline-end",
                    "calc(var(--fandhe-space-3) + 1.25rem + var(--fandhe-space-1))",
                ),
            ],
        )
        // 選択色（`data-active`）を hover が洗い流さないよう
        // `HoverExceptAttr` で除外する（[`crate::command`] の `item` と同型）。
        // 上記コメントのとおり、outline の背景規則より後段に置いて
        // specificity 同点のソース順優先で選択色を勝たせる。
        .state(
            "menu-button",
            StateCondition::Attr("data-active"),
            vec![
                decl("background", "var(--fandhe-color-sidebar-accent)"),
                decl("color", "var(--fandhe-color-sidebar-accent-fg)"),
                decl("font-weight", "var(--fandhe-font-font-weight-medium)"),
            ],
        )
        .state(
            "menu-button",
            StateCondition::HoverExceptAttr("data-active"),
            vec![decl("background", "var(--fandhe-color-sidebar-muted)")],
        )
        .state(
            "menu-button",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Outside),
        )
        // `menu-sub-button`: `menu-button` と同型の選択色・サイズ規則。
        .state(
            "menu-sub-button",
            StateCondition::Attr("data-active"),
            vec![
                decl("background", "var(--fandhe-color-sidebar-accent)"),
                decl("color", "var(--fandhe-color-sidebar-accent-fg)"),
                decl("font-weight", "var(--fandhe-font-font-weight-medium)"),
            ],
        )
        .state(
            "menu-sub-button",
            StateCondition::HoverExceptAttr("data-active"),
            vec![decl("background", "var(--fandhe-color-sidebar-muted)")],
        )
        .state(
            "menu-sub-button",
            // 既定は `sm`（`data-size` 省略時と同じ `xs` 相当の文字サイズ）。
            // `md` 指定時のみ本文サイズへ拡大する（headless
            // `SidebarMenuSubButtonSize` の既定は `Sm`）。
            StateCondition::AttrEq("data-size", "md"),
            vec![decl("font-size", "var(--fandhe-font-font-size-sm)")],
        )
        .state(
            "menu-sub-button",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Outside),
        )
        // `trigger`/`rail`: hover・フォーカスリング。
        .state(
            "trigger",
            StateCondition::Hover,
            vec![decl("background", "var(--fandhe-color-sidebar-muted)")],
        )
        .state(
            "trigger",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Outside),
        )
        .state(
            "rail",
            StateCondition::Hover,
            vec![decl(
                "box-shadow",
                "inset 2px 0 0 var(--fandhe-color-sidebar-border)",
            )],
        )
        // `input`: canonical フォーカスリング（`--fandhe-color-sidebar-focus-ring`
        // で上書き）。
        .state("input", StateCondition::FocusVisible, {
            let mut declarations =
                focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Outside);
            declarations.push(decl(
                "outline-color",
                "var(--fandhe-color-sidebar-focus-ring)",
            ));
            declarations
        })
}

/// この styled Sidebar が生成する静的 CSS 全量を返す（決定的。
/// [`crate::command::stylesheet`] と同じ契約）。[`SlotRecipe`] が子孫/兄弟/
/// 子結合子セレクタを表現できないため（モジュール doc参照）、以下の raw CSS
/// を追記する:
///
/// 1. icon 折りたたみ時に `group-label`/`menu-badge`/`menu-action`/
///    `group-action`/`menu-sub`/`input` を隠し、`menu-button` を中央寄せ・
///    パディング詰めする子孫セレクタ群。
/// 2. モバイル + collapsed 時の幅固定 + `transform` 切替（詳細度調整、
///    モジュール doc「モバイル + collapsed の詳細度調整」節参照）。
/// 3. `variant="inset"` の `inset` パーツを面パネル化する子結合子セレクタ
///    （side ごとに margin の向きを変える 2 本）。
/// 4. `side="right"` の `rail` 位置反転（子結合子セレクタ）。
/// 5. icon 折りたたみ時の `header`/`footer` テキストクリップ（子結合子
///    セレクタ。`root` 自身へ `overflow-x: hidden` を付けると `rail`
///    〔`root` に対する絶対配置で `root` のボーダーボックス外へはみ出す
///    開閉トグル〕がクリップされ再展開不能になるため、`root` ではなく
///    `header`/`footer` パーツへ個別に付与する）。
/// 6. `menu-action`/`menu-badge` 併用時のオフセット分離（兄弟結合子 2 本）
///    と `menu-button` 終了側余白の 2 パーツ分拡張（`:has()` 併用の子
///    結合子。codex-review P2 指摘対応）。
/// 7. `rail` hover ラインの物理端/RTL/side 反転 3 規則（子結合子セレクタの
///    ため [`SlotRecipe::state`] の `Hover` を経由できず、`@media
///    (hover: hover)` 集約 + `:not([data-disabled])` を raw CSS 側で
///    手動併記する。Cursor Bugbot Low 指摘対応）。
#[must_use]
pub fn stylesheet() -> String {
    let mut out = recipe().css();
    let mut push = |selector: &str, declarations: &[crate::css::Declaration]| {
        if let Some(rule) = serialize_rule(selector, declarations) {
            if !out.is_empty() {
                out.push('\n');
            }
            out.push_str(&rule);
        }
    };

    const ICON_COLLAPSED: &str = r#"[data-scope="sidebar"][data-part="root"][data-state="collapsed"][data-collapsible="icon"]"#;

    push(
        &format!("{ICON_COLLAPSED} [data-scope=\"sidebar\"][data-part=\"group-label\"]"),
        &[decl("display", "none")],
    );
    push(
        &format!("{ICON_COLLAPSED} [data-scope=\"sidebar\"][data-part=\"menu-badge\"]"),
        &[decl("display", "none")],
    );
    push(
        &format!("{ICON_COLLAPSED} [data-scope=\"sidebar\"][data-part=\"menu-action\"]"),
        &[decl("display", "none")],
    );
    push(
        &format!("{ICON_COLLAPSED} [data-scope=\"sidebar\"][data-part=\"group-action\"]"),
        &[decl("display", "none")],
    );
    push(
        &format!("{ICON_COLLAPSED} [data-scope=\"sidebar\"][data-part=\"menu-sub\"]"),
        &[decl("display", "none")],
    );
    push(
        &format!("{ICON_COLLAPSED} [data-scope=\"sidebar\"][data-part=\"input\"]"),
        &[decl("display", "none")],
    );
    push(
        &format!("{ICON_COLLAPSED} [data-scope=\"sidebar\"][data-part=\"menu-button\"]"),
        &[
            // `justify-content: center` は使わない（codex-review P1 指摘）:
            // [`menu_button`]（本ファイル）が children（アイコン + テキスト）
            // をまとめて包む装飾用ラッパー `<span>` は既定 `justify-content:
            // flex-start` のためアイコンは常にラッパー左端に残る。`center`
            // にすると `padding: 0` で幅 2rem に縮小されたボタン内でアイコン
            // 自体がラッパーの `overflow: hidden` の外側へ押し出されて
            // 切れてしまう（イシュー #2073 レビュー対応で `menu-button`
            // 自身への `clip-path` を撤回し `overflow: hidden` をラッパー
            // 側へ移した後も同じ理由で `flex-start` を維持する）。ラッパーの
            // 幅は `menu-button` のコンテンツ box（`padding: 0` により本
            // 規則適用時は幅 2rem 全域）と一致するため、はみ出す文字列側は
            // ラッパーの `overflow: hidden`（`SIDEBAR_MENU_BUTTON_LABEL_
            // SELECTOR` 規則）で視覚的に切り落とされる（モジュール doc
            // 「icon 折りたたみ時のテキスト非表示」節の意図どおりの挙動）。
            decl("padding", "0"),
            decl("width", "2rem"),
        ],
    );

    // `menu-action`/`menu-badge` の併用時のオフセット分離（codex-review P2
    // 指摘「アクションとバッジを併用すると表示が重なる」対応）。両パーツは
    // `menu_action_base`/`menu_badge_base` でそれぞれ独立に
    // `inset-inline-end: var(--fandhe-space-2)` の絶対配置を持ち、
    // `menu-item` に両方を同時に配置すると同一位置（右端・垂直中央）へ
    // 重なる。`SlotRecipe` は兄弟結合子を表現できないため（モジュール doc
    // 参照）raw CSS で分離する。呼び出し規約（本ファイルのテスト・
    // `crates/docs-site/src/primitive_showcase/navigation.rs` の実例が示す
    // `menu-button` → `menu-action` → `menu-badge` の DOM 順）に対応する
    // 一般兄弟結合子（`~`）を使い、後方のパーツを前方のパーツ 1 個分
    // （幅 1.25rem + 隙間 `--fandhe-space-1`）だけ内側へ押し出す。順序を
    // 入れ替えて呼び出された場合（`menu-badge` が先・`menu-action` が後）
    // でも重ならないよう、両方向のセレクタを併記する。
    push(
        r#"[data-scope="sidebar"][data-part="menu-action"] ~ [data-scope="sidebar"][data-part="menu-badge"]"#,
        &[decl(
            "inset-inline-end",
            "calc(var(--fandhe-space-2) + 1.25rem + var(--fandhe-space-1))",
        )],
    );
    push(
        r#"[data-scope="sidebar"][data-part="menu-badge"] ~ [data-scope="sidebar"][data-part="menu-action"]"#,
        &[decl(
            "inset-inline-end",
            "calc(var(--fandhe-space-2) + 1.25rem + var(--fandhe-space-1))",
        )],
    );
    // `menu-button` の終了側余白（`menu_button_base` の
    // `padding-inline-end`）は 1 パーツ分のみを見込んでいるため、
    // `menu-action`/`menu-badge` を併用する `menu-item` では 2 パーツ分の
    // 余白へ拡張し、長いラベルが分離後のバッジ/アクションとも重ならない
    // ようにする（codex-review P2 指摘「menu-button の終了側余白も両方の
    // 幅に合わせる」対応）。`:has()` は本クレートで先例のある標準機能
    // （`crate::list` 参照）であり、両パーツの共存を子結合子越しに検知する。
    push(
        r#"[data-scope="sidebar"][data-part="menu-item"]:has(> [data-scope="sidebar"][data-part="menu-action"]):has(> [data-scope="sidebar"][data-part="menu-badge"]) > [data-scope="sidebar"][data-part="menu-button"]"#,
        &[decl(
            "padding-inline-end",
            "calc(var(--fandhe-space-2) + 2 * 1.25rem + 2 * var(--fandhe-space-1))",
        )],
    );
    // icon 折りたたみ時は `menu-action`/`menu-badge`（両パーツとも
    // `{ICON_COLLAPSED} [data-part="group-action"/...]` 系の非表示規則の
    // 対象外だが、`ICON_COLLAPSED [data-part="menu-button"]` 規則が
    // `padding: 0` でラベル用の余白予約自体を解除する設計、上記
    // `push(&format!("{ICON_COLLAPSED} ... menu-button"), ...)` 参照）でも、
    // 直上の `:has()` 規則（属性 8 個相当・`:has()` 2 個分の内部
    // セレクタ specificity を含む）が `ICON_COLLAPSED` 経由の規則
    // （属性 6 個相当）より詳細度で勝るため `padding-inline-end` が
    // 2 パーツ分のまま残留し、幅 2rem に縮小されたボタンから
    // 3.5rem 相当の余白がはみ出す（codex-review P1 指摘「折りたたみ時は
    // アクション・バッジ用の追加余白を除外する」対応。Cursor Bugbot High
    // 指摘「Icon collapse clips action-badge buttons」と実質同根）。
    // `ICON_COLLAPSED` を子結合子の前段に連結し、直上の `:has()` 規則より
    // 詳細度を高くした専用の上書き規則で `padding-inline-end` を明示的に
    // `0` へ戻す（`ICON_COLLAPSED [data-part="menu-button"]` の
    // `padding: 0` ショートハンドは詳細度で負けて無効化されているため、
    // ロングハンド `padding-inline-end` だけを個別に再上書きする）。
    push(
        &format!(
            "{ICON_COLLAPSED} [data-scope=\"sidebar\"][data-part=\"menu-item\"]:has(> [data-scope=\"sidebar\"][data-part=\"menu-action\"]):has(> [data-scope=\"sidebar\"][data-part=\"menu-badge\"]) > [data-scope=\"sidebar\"][data-part=\"menu-button\"]"
        ),
        &[decl("padding-inline-end", "0")],
    );

    // 長いラベルをガター（`menu-action`/`menu-badge` 予約領域）の手前で
    // クリップする（codex-review P2 指摘対応）。イシュー #2073 レビュー
    // （codex-review P1・Cursor Bugbot Medium、いずれも `menu-button` 全体
    // への `clip-path: inset()` 適用を指摘）を受け、旧実装（`menu-button`
    // 自身への `clip-path` 適用）を撤回した。CSS Masking の仕様上
    // `clip-path` は要素のペイント・ヒットテスト対象を丸ごと切り落とす
    // ため、ボタン本体の背景・クリック領域・`:focus-visible` の
    // outline（`outline-offset` により border-box 外側に描画される）まで
    // 一緒に失われる（WCAG 2.4.7 Focus Visible 違反の回帰、かつ
    // `menu-badge` の `pointer-events: none`〔`menu_badge_base` 参照〕に
    // よる `menu-button` へのクリック透過も破壊する）。
    //
    // 代わりに [`menu_button`]（本ファイル、headless
    // `fandhe_frontend_headless_ui::sidebar::menu_button` への薄い委譲
    // 関数）が `children` 全体を装飾用の無印 `<span>`（`data-part`/
    // `data-scope` を持たない、headless anatomy 外の pre-styled-ui 専用
    // ラッパー。`docs/policy/intentional-non-adoption.md` §3.25 規則 2
    // 「装飾・レイアウト計測は Themes 層の責務」に従う）で包んでから
    // headless へ渡す。このラッパー span 自身に `overflow: hidden` +
    // `white-space: nowrap` + `min-width: 0` を適用する（下記
    // `SIDEBAR_MENU_BUTTON_LABEL_SELECTOR` 規則）。`overflow: hidden` の
    // クリップ境界は要素自身の padding box 外側端であり、ラッパー span は
    // `padding` を持たないため、境界は `menu-button` の**コンテンツ box**
    // （`padding-inline-end` で予約したガターを除いた領域）と一致する。
    // これにより `menu-button` 本体は一切クリップされないまま
    // （背景・クリック領域・outline を保持）、ラベルだけがガター手前で
    // 正しく切り落とされる。`padding-inline-end` の計算式（1 パーツ分・
    // 2 パーツ分・icon 折りたたみ時 `0`）は変更なしで維持し、この
    // ガター確保がそのままラッパーのクリップ境界として機能する。
    push(
        SIDEBAR_MENU_BUTTON_LABEL_SELECTOR,
        &[
            decl("display", "flex"),
            decl("align-items", "center"),
            decl("gap", "var(--fandhe-space-2)"),
            decl("min-width", "0"),
            decl("flex", "1 1 auto"),
            decl("overflow", "hidden"),
            decl("white-space", "nowrap"),
        ],
    );

    // モバイル表示中は固定オーバーレイになる。`floating`/`inset` variant が
    // 設定する `margin`/`border`/`border-radius`/`background`/`height` は
    // デスクトップの浮遊・面パネル表現専用であり、モバイルオーバーレイ
    // では全幅・全高のドロワーへリセットする必要がある（Bugbot 指摘:
    // リセットしないと variant の見た目がドロワーへ残存する）。
    // `:not([data-collapsible="none"])`（Cursor Bugbot Medium 指摘対応）:
    // `data-collapsible="none"`（閉じる手段を持たない常時表示契約、
    // モジュール doc「`data-collapsible`」節）のサイドバーへこの規則を
    // 適用すると、閉じるトリガーが無いまま `position: fixed` の恒久
    // オーバーレイになり、in-flow の `inset`（`variant="inset"` の主領域）
    // が下へ広がってメイン領域を覆い隠し続けてしまう
    // （`crates/pre-styled-ui/src/sidebar.rs#L563-L580, #L830-L837` 指摘）。
    // 本規則自体を対象外にすることで、`collapsible="none"` はモバイルでも
    // 通常の in-flow 表示（`root_base`/variant state が決める幅・位置）を
    // 保つ。属性 4 個（`data-scope`/`data-part`/`data-mobile`/`:not(...)`)
    // は variant 系 state（属性 3 個）より詳細度で優先する。
    push(
        r#"[data-scope="sidebar"][data-part="root"][data-mobile]:not([data-collapsible="none"])"#,
        &[
            decl("position", "fixed"),
            decl("inset-block", "0"),
            decl("inset-inline-start", "0"),
            decl("width", "var(--fandhe-sidebar-width-mobile, 18rem)"),
            decl("height", "100%"),
            decl("margin", "0"),
            decl("border", "0"),
            decl("border-radius", "0"),
            decl("background", "var(--fandhe-color-sidebar-bg)"),
            decl("z-index", "var(--fandhe-z-index-modal, 1001)"),
            decl("box-shadow", "var(--fandhe-shadow-lg)"),
        ],
    );

    // モバイル + `side="right"`: 開閉状態（`data-state`）に関係なく右側へ
    // ドッキングする（属性 4 個 = `data-mobile` 単独の base 規則、属性 3 個
    // より優先。codex-review P1 / Bugbot 指摘: expanded 時に base 規則の
    // `inset-inline-start: 0` が残って左に表示されてしまう不具合の是正。
    // 下記「モバイル + collapsed」規則は開閉に応じた `transform` の向きの
    // みを上書きし、位置（`inset-inline-*`）はこの規則が単独の情報源と
    // なる）。
    push(
        r#"[data-scope="sidebar"][data-part="root"][data-side="right"][data-mobile]"#,
        &[
            decl("inset-inline-start", "auto"),
            decl("inset-inline-end", "0"),
        ],
    );

    // モバイル + collapsed: `root` の折りたたみ幅規則（属性 4 個）に対し
    // `data-mobile` 単独（属性 3 個）は詳細度で負けるため、本規則（属性 4 個
    // + ソース順で後）で幅を固定し `transform` のみで開閉する（モジュール
    // doc「モバイル + collapsed の詳細度調整」節参照）。`visibility: hidden`
    // は上記 offcanvas collapsed 規則と同じ理由（codex-review P1 指摘）で
    // 併記する: `transform` による画面外への移動だけでは子孫が Tab 順序・
    // アクセシビリティツリーに残る。`:not([data-collapsible="none"])` は
    // codex-review P1 再指摘の是正: `data-state`/`data-collapsible` は
    // headless 側で独立に決まる（headless モジュール doc「`data-state`/
    // `data-collapsible`」節）ため `collapsible="none"`（常時表示・折りた
    // たまない契約）でも `data-state="collapsed"` になり得る。この限定が
    // 無いと `collapsible="none"` のサイドバーがモバイルで画面外へ退避し
    // 消えてしまい、「collapsible="none" は常時表示」という公開契約に反する。
    push(
        r#"[data-scope="sidebar"][data-part="root"][data-mobile][data-state="collapsed"]:not([data-collapsible="none"])"#,
        &[
            decl("width", "var(--fandhe-sidebar-width-mobile, 18rem)"),
            decl("transform", "translateX(-100%)"),
            decl("visibility", "hidden"),
            // `root_transition`（base）の `visibility` 遷移 delay を
            // `duration-normal` へ上書きし、`transform` の退避アニメーション
            // が終わるまで可視性を維持する（`root_collapsed_visibility_delay`
            // 定義部の doc・上記 offcanvas collapsed 規則と同じ理由）。
            ROOT_COLLAPSED_VISIBILITY_DELAY,
        ],
    );
    // `side="right"` + collapsed: 位置は上記の側指定専用規則が既に固定
    // 済みのため、ここでは開閉の `transform` 方向のみを右側向けへ上書き
    // する。`:not([data-collapsible="none"])` は上記規則と同じ理由
    // （codex-review P1 指摘: `collapsible="none"` はモバイルでも常時表示
    // という公開契約〔headless モジュール doc「`data-collapsible`」節、
    // `data-state` は `collapsible` の値と独立に外部から与えられうる〕の
    // ため、退避規則を offcanvas/icon 専用に限定する）で併記する。
    push(
        r#"[data-scope="sidebar"][data-part="root"][data-side="right"][data-mobile][data-state="collapsed"]:not([data-collapsible="none"])"#,
        &[decl("transform", "translateX(100%)")],
    );
    // `side`（既定=inline-start 側/`right`=inline-end 側）は本モジュール
    // 全体で `inset-inline-*`/`border-inline-*` 等の論理プロパティのみを
    // 用いて表現している（`root` 展開時の `border-inline-end`・
    // `data-side="right"` 時の `border-inline-start` 反転が例）が、
    // `transform: translateX()` には論理方向の等価物が無く物理方向のまま
    // 固定されるため `dir="rtl"` 文書では退避方向が逆転してしまう
    // （codex-review 指摘「Mobile drawer slides the wrong way」対応）。
    // [`crate::scroll_area`] の `:dir(rtl)` 併記と同型に、rtl 文書向けの
    // 符号反転規則をソース順で後に追記し上書きする。
    push(
        r#"[data-scope="sidebar"][data-part="root"][data-mobile][data-state="collapsed"]:not([data-collapsible="none"]):dir(rtl)"#,
        &[decl("transform", "translateX(100%)")],
    );
    push(
        r#"[data-scope="sidebar"][data-part="root"][data-side="right"][data-mobile][data-state="collapsed"]:not([data-collapsible="none"]):dir(rtl)"#,
        &[decl("transform", "translateX(-100%)")],
    );

    // `variant="inset"` の主領域（`inset` パーツを面パネル化する）。
    push(
        r#"[data-scope="sidebar"][data-part="provider"][data-variant="inset"] > [data-scope="sidebar"][data-part="inset"]"#,
        &[
            decl("margin", "var(--fandhe-space-2)"),
            decl("margin-inline-start", "0"),
            decl("border-radius", "var(--fandhe-radius-lg)"),
            decl("box-shadow", "var(--fandhe-shadow-sm)"),
            decl("background", "var(--fandhe-color-bg)"),
        ],
    );
    // マージン反転は `variant="inset"` の面パネル化規則（直前の push）が
    // 付けた `margin-inline-start: 0` を打ち消して逆側に付け替えるための
    // ものであり、`sidebar`/`floating` 等ほかの variant では `inset` パーツ
    // に margin を持たせない（Bugbot 指摘: `data-variant` を問わず適用する
    // と非 inset variant にも意図しない margin が付いてしまう）。セレクタへ
    // `[data-variant="inset"]` を明示して対象を絞る。
    push(
        r#"[data-scope="sidebar"][data-part="provider"][data-variant="inset"][data-side="right"] > [data-scope="sidebar"][data-part="inset"]"#,
        &[
            decl("margin-inline-start", "var(--fandhe-space-2)"),
            decl("margin-inline-end", "0"),
        ],
    );

    // `variant="inset"` + `collapsible="icon"` の折りたたみ時、`inset` 主
    // パネルの margin を復元する（Bugbot 指摘「Inset panel margin not
    // restored」対応）。上記 2 規則は展開時の既定・`side="right"` 反転のみを
    // 扱い、`root`（ナビレール）が 3rem 幅へ縮む折りたたみ時の margin
    // 調整を持たない。`root` に隣接する側（既定 left は
    // `margin-inline-start`、`side="right"` は `margin-inline-end`）は
    // 展開時の面パネル化規則が `0` にした値のままだと `inset` パネルが
    // 縮んだレールへ密着してしまうため、折りたたみ時のみ
    // `--fandhe-space-2` へ戻す（shadcn/ui `SidebarInset` の
    // `peer-data-[state=collapsed]:peer-data-[variant=inset]:ml-2` と
    // 同型の挙動）。`collapsible="offcanvas"` は `root` 自体を
    // `visibility: hidden` + 幅 0 にして完全に消す設計（[`recipe`] 参照）
    // のため margin 復元は不要（レールが無いのでそのまま隙間を詰めてよい）
    // であり本規則の対象に含めない。
    push(
        r#"[data-scope="sidebar"][data-part="provider"][data-variant="inset"][data-state="collapsed"][data-collapsible="icon"] > [data-scope="sidebar"][data-part="inset"]"#,
        &[decl("margin-inline-start", "var(--fandhe-space-2)")],
    );
    push(
        r#"[data-scope="sidebar"][data-part="provider"][data-variant="inset"][data-side="right"][data-state="collapsed"][data-collapsible="icon"] > [data-scope="sidebar"][data-part="inset"]"#,
        &[decl("margin-inline-end", "var(--fandhe-space-2)")],
    );

    // `menu-action`/`menu-badge` の垂直中央位置を `menu-button` の
    // `data-size` に追随させる（既定 `base` の `top: 1rem` は `menu-button`
    // の既定高さ 2rem の半分。Bugbot・codex-review 指摘「Action offset
    // ignores button size」対応）。`SlotRecipe` は兄弟結合子を持たない
    // （モジュール内複数箇所のコメント参照）ため、ここで一般兄弟結合子
    // （`~`）による raw CSS として追記する。呼び出し規約（`menu-button` を
    // 先に置き、`menu-action`/`menu-badge` を後続の兄弟として置く。
    // `crates/docs-site/src/primitive_showcase/navigation.rs` の実例
    // 参照）を前提とする。
    const SM_HEIGHT_HALF: &str = "0.875rem"; // 1.75rem / 2
    const LG_HEIGHT_HALF: &str = "1.5rem"; // 3rem / 2
    for part in ["menu-action", "menu-badge"] {
        push(
            &format!(
                r#"[data-scope="sidebar"][data-part="menu-button"][data-size="sm"] ~ [data-scope="sidebar"][data-part="{part}"]"#
            ),
            &[decl("top", SM_HEIGHT_HALF)],
        );
        push(
            &format!(
                r#"[data-scope="sidebar"][data-part="menu-button"][data-size="lg"] ~ [data-scope="sidebar"][data-part="{part}"]"#
            ),
            &[decl("top", LG_HEIGHT_HALF)],
        );
    }

    // `side="right"` の `rail` 位置反転。
    push(
        r#"[data-scope="sidebar"][data-part="root"][data-side="right"] > [data-scope="sidebar"][data-part="rail"]"#,
        &[
            decl("inset-inline-end", "auto"),
            decl("inset-inline-start", "-1rem"),
        ],
    );

    // `rail` hover ラインの物理端/RTL/side 反転（Cursor Bugbot Low 指摘
    // 「Rail hover indicator uses physical left offset」対応）。
    // `box-shadow` に論理方向の等価物（`inset-inline-*` 相当）が無いため、
    // `:dir(rtl)` 併記（本モジュール「モバイル + collapsed の詳細度調整」
    // 節・`translateX` の RTL 反転と同型）で 4 通り（side 2 種 × dir 2 種）
    // を明示する。基本規則（`.state("rail", StateCondition::Hover, ...)`、
    // `inset 2px 0 0`）は `root` の inline-end 側（既定 side、LTR での物理
    // 右端）に `rail` がドッキングし、その `rail` の inline-start 端（LTR
    // では物理左端）が `root` との継ぎ目になる場合にのみ正しい。以下は
    // それ以外の 3 通り（継ぎ目が物理右端になる場合）を上書きする。
    //
    // 子結合子セレクタのため `SlotRecipe::state` の `Hover`
    // （`@media (hover: hover)` 集約 + `:not([data-disabled])`、イシュー
    // #1425）を経由できず raw CSS で書く必要があるが、この 3 規則も同じ
    // hover 規約に従わせる（Cursor Bugbot Low 再指摘対応）。タッチ端末は
    // `hover: hover` に一致しないため `@media` で括らないと、タップ後も
    // `:hover` 状態が貼り付いたまま `side="right"`/RTL レールの
    // indicator が残留する。`:not([data-disabled])` も基本規則と揃え、
    // `rail` に `data-disabled` を持たせた場合は無効化する。実際の
    // `@media` ブロック合成は `push` クロージャの可変借用が関数末尾まで
    // 生存するため（後続の header/footer クリップ規則が `push` を再利用
    // する）、`out` への直接書き込みを関数末尾（`push` の最終利用後）へ
    // 先送りする（下記 `rail_hover_media` 参照）。
    const RAIL_HOVER_OVERRIDES: [(&str, &str); 3] = [
        (
            // `data-side="right"` の `root` では `rail` が `root` の
            // inline-start 側（上記位置反転規則）にドッキングし、`rail`
            // の inline-end 端が継ぎ目になる。LTR では inline-end = 物理
            // 右端のため、ヒントは `rail` の右端（`inset -2px 0 0`）に
            // 出す必要がある。
            r#"[data-scope="sidebar"][data-part="root"][data-side="right"] > [data-scope="sidebar"][data-part="rail"]:hover:not([data-disabled])"#,
            "inset -2px 0 0 var(--fandhe-color-sidebar-border)",
        ),
        (
            // 既定 side（`data-side` 未指定）+ RTL: inline-start = 物理
            // 右端のため、継ぎ目は `rail` の物理右端に移る。
            r#"[data-scope="sidebar"][data-part="root"] > [data-scope="sidebar"][data-part="rail"]:hover:not([data-disabled]):dir(rtl)"#,
            "inset -2px 0 0 var(--fandhe-color-sidebar-border)",
        ),
        (
            // `data-side="right"` + RTL: inline-end = 物理左端のため、
            // 継ぎ目は `rail` の物理左端に戻る（属性 7 個相当で上記 2
            // 規則より詳細度が高く、`data-side="right"` かつ RTL の場合
            // に確実に優先する）。
            r#"[data-scope="sidebar"][data-part="root"][data-side="right"] > [data-scope="sidebar"][data-part="rail"]:hover:not([data-disabled]):dir(rtl)"#,
            "inset 2px 0 0 var(--fandhe-color-sidebar-border)",
        ),
    ];

    // icon 折りたたみ時の `header`/`footer` テキストクリップ。`root` は
    // `position: relative` の基準要素であり、`rail`（`position: absolute`
    // + `inset-inline-end: -1rem`）は `root` のボーダーボックス外へ意図的に
    // はみ出して配置される開閉トグルである。`overflow-x: hidden` を `root`
    // 自身へ付けると `rail` の描画・クリック領域ごとクリップされ、折りたた
    // んだ後に再展開する手段が失われる（codex-review P1 / Cursor Bugbot
    // 指摘「Icon collapse clips the rail」対応）。クリップ対象を `header`/
    // `footer`（`header_footer_base` は自身に `overflow` を持たず、`root`
    // の `min-width` 固定〔上記 [`recipe`] 参照〕だけでは内部テキストが
    // 折りたたみ幅を超えてレール外へはみ出して見えてしまう）へ個別に
    // 限定する。`content` は既に自前で `overflow: auto`（`content_base`）
    // を持つため対象に含めない。
    push(
        &format!("{ICON_COLLAPSED} > [data-scope=\"sidebar\"][data-part=\"header\"]"),
        &[decl("overflow-x", "hidden")],
    );
    push(
        &format!("{ICON_COLLAPSED} > [data-scope=\"sidebar\"][data-part=\"footer\"]"),
        &[decl("overflow-x", "hidden")],
    );

    // `RAIL_HOVER_OVERRIDES`（上記「`rail` hover ラインの物理端/RTL/side
    // 反転」節）を `@media (hover: hover)` 配下へ集約して追記する。`push`
    // クロージャは `out` を可変借用したままここまで生存するため（直前まで
    // 呼び出しが続く）、この 1 箇所へ集約することで `out` への直接書き込み
    // と `push` の可変借用が同時に競合しないようにする。
    let mut rail_hover_media = String::new();
    for (selector, box_shadow) in RAIL_HOVER_OVERRIDES {
        if let Some(rule) = serialize_rule(selector, &[decl("box-shadow", box_shadow)]) {
            rail_hover_media.push_str(&rule);
            rail_hover_media.push('\n');
        }
    }
    if !rail_hover_media.is_empty() {
        if !out.is_empty() {
            out.push('\n');
        }
        out.push_str("@media (hover: hover) {\n");
        for line in rail_hover_media.trim_end_matches('\n').lines() {
            if line.is_empty() {
                out.push('\n');
            } else {
                out.push_str("  ");
                out.push_str(line);
                out.push('\n');
            }
        }
        out.push_str("}\n");
    }

    out
}

/// styled `provider` パーツを組み立てる。
#[must_use]
pub fn provider<'a>(
    state: &Sidebar,
    props: &SidebarProps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    fandhe_frontend_headless_ui::sidebar::provider(state, props, drop_class_attr(attrs), children)
}

/// styled `root` パーツを組み立てる。
#[must_use]
pub fn root<'a>(
    state: &Sidebar,
    props: &SidebarProps,
    label: &'a str,
    id: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    fandhe_frontend_headless_ui::sidebar::root(
        state,
        props,
        label,
        id,
        drop_class_attr(attrs),
        children,
    )
}

/// styled `header` パーツを組み立てる。
#[must_use]
pub fn header<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    fandhe_frontend_headless_ui::sidebar::header(drop_class_attr(attrs), children)
}

/// styled `content` パーツを組み立てる。
#[must_use]
pub fn content<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    fandhe_frontend_headless_ui::sidebar::content(drop_class_attr(attrs), children)
}

/// styled `footer` パーツを組み立てる。
#[must_use]
pub fn footer<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    fandhe_frontend_headless_ui::sidebar::footer(drop_class_attr(attrs), children)
}

/// styled `separator` パーツを組み立てる。
#[must_use]
pub fn separator<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    fandhe_frontend_headless_ui::sidebar::separator(drop_class_attr(attrs), children)
}

/// styled `input` パーツを組み立てる。
#[must_use]
pub fn input<'a>(attrs: Vec<(&'a str, &'a str)>) -> Node {
    fandhe_frontend_headless_ui::sidebar::input(drop_class_attr(attrs))
}

/// styled `group` パーツを組み立てる。
#[must_use]
pub fn group<'a>(
    labelledby: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    fandhe_frontend_headless_ui::sidebar::group(labelledby, drop_class_attr(attrs), children)
}

/// styled `group-label` パーツを組み立てる。
#[must_use]
pub fn group_label<'a>(
    id: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    fandhe_frontend_headless_ui::sidebar::group_label(id, drop_class_attr(attrs), children)
}

/// styled `group-content` パーツを組み立てる。
#[must_use]
pub fn group_content<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    fandhe_frontend_headless_ui::sidebar::group_content(drop_class_attr(attrs), children)
}

/// styled `group-action` パーツを組み立てる。
#[must_use]
pub fn group_action<'a>(
    label: &'a str,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    fandhe_frontend_headless_ui::sidebar::group_action(label, drop_class_attr(attrs), children)
}

/// styled `menu` パーツを組み立てる。
#[must_use]
pub fn menu<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    fandhe_frontend_headless_ui::sidebar::menu(drop_class_attr(attrs), children)
}

/// styled `menu-item` パーツを組み立てる。
#[must_use]
pub fn menu_item<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    fandhe_frontend_headless_ui::sidebar::menu_item(drop_class_attr(attrs), children)
}

/// styled `menu-button` パーツを組み立てる。アイコンを `children` の最初の
/// 要素に置く呼び出し規約を前提とする（モジュール doc「icon 折りたたみ時の
/// テキスト非表示」節参照）。
///
/// `children`（アイコン + ラベル）は headless
/// [`fandhe_frontend_headless_ui::sidebar::menu_button`] へ渡す前に、本関数が
/// 装飾用の無印 `<span>`（`data-part`/`data-scope` を持たない、headless
/// anatomy 外の pre-styled-ui 専用ラッパー。`docs/policy/
/// intentional-non-adoption.md` §3.25 規則 2「装飾・レイアウト計測は Themes
/// 層の責務」に従う）で 1 段包む。headless anatomy 自体は変更しない
/// （headless 側でラベルを `span` 化する改善はモジュール doc「スコープ外」
/// 節のとおり別イシュー）。この 1 段ラップにより、長いラベルの視覚的な
/// クリップ（[`stylesheet`] 内コメント「長いラベルをガターの手前で
/// クリップする」節）を `menu-button` 本体ではなくラッパー側に限定でき、
/// ボタンの背景・クリック領域・`:focus-visible` の outline を一切損なわない
/// （イシュー #2073 レビュー〔codex-review P1・Cursor Bugbot Medium〕対応）。
#[must_use]
pub fn menu_button<'a>(
    props: &SidebarMenuButtonProps<'a>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let wrapped = vec![el("span", vec![], children)];
    fandhe_frontend_headless_ui::sidebar::menu_button(props, drop_class_attr(attrs), wrapped)
}

/// styled `menu-action` パーツを組み立てる。
#[must_use]
pub fn menu_action<'a>(
    label: &'a str,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    fandhe_frontend_headless_ui::sidebar::menu_action(label, drop_class_attr(attrs), children)
}

/// styled `menu-badge` パーツを組み立てる。
#[must_use]
pub fn menu_badge<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    fandhe_frontend_headless_ui::sidebar::menu_badge(drop_class_attr(attrs), children)
}

/// styled `menu-sub` パーツを組み立てる。
#[must_use]
pub fn menu_sub<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    fandhe_frontend_headless_ui::sidebar::menu_sub(drop_class_attr(attrs), children)
}

/// styled `menu-sub-item` パーツを組み立てる。
#[must_use]
pub fn menu_sub_item<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    fandhe_frontend_headless_ui::sidebar::menu_sub_item(drop_class_attr(attrs), children)
}

/// styled `menu-sub-button` パーツを組み立てる。
#[must_use]
pub fn menu_sub_button<'a>(
    props: &SidebarMenuSubButtonProps<'a>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    fandhe_frontend_headless_ui::sidebar::menu_sub_button(props, drop_class_attr(attrs), children)
}

/// styled `rail` パーツを組み立てる。
#[must_use]
pub fn rail<'a>(
    state: &Sidebar,
    label: &'a str,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    fandhe_frontend_headless_ui::sidebar::rail(state, label, drop_class_attr(attrs), children)
}

/// styled `trigger` パーツを組み立てる。
#[must_use]
pub fn trigger<'a>(
    state: &Sidebar,
    label: &'a str,
    controls: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    fandhe_frontend_headless_ui::sidebar::trigger(
        state,
        label,
        controls,
        drop_class_attr(attrs),
        children,
    )
}

/// styled `inset` パーツを組み立てる。
#[must_use]
pub fn inset<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    fandhe_frontend_headless_ui::sidebar::inset(drop_class_attr(attrs), children)
}

/// `menu-item` 1 行分のローディング装飾（モジュール doc「`menu-skeleton`」
/// 節参照）。`show_icon` が `true` のとき先頭にアイコン用の円形
/// skeleton を、続けて固定幅のテキスト用 skeleton を合成する。決定的で
/// ランダム幅を持たない（shadcn/ui との意図的な差分）。自前の `data-*` は
/// 出力しない。
///
/// # 行内レイアウト（`style` 属性、Bugbot/codex-review 指摘「Menu skeleton
/// stacks instead of row」対応）
///
/// [`crate::skeleton::skeleton`] の `root` は既定 `display: block` の
/// `div` であり、`menu-item`（`<li>`）が並べる 2 個（アイコン + テキスト）
/// はそのままだと縦積みになる。[`recipe`] の `menu-item` slot を
/// `display: flex` へ変えて解決する案は不採用: モジュール doc・
/// `menu_action_base`/`menu_badge_base` のコメントが明記するとおり
/// `menu-item` は一般の利用パターンで `menu-sub`（複数行の子メニュー）を
/// 直接の子として縦に積む構成も取りうるため、slot 全体を row へ変えると
/// その用途が壊れる（`crate::button_group`/`crate::card` と同型に
/// `:has()`/兄弟結合子を持たない [`SlotRecipe`] の制約）。本関数が返す
/// `menu-item` は常にこの 2 個の skeleton のみを子に持つ自己完結した
/// 構造のため、[`recipe`] を変更せず戻り値自身にだけ `style` 属性で
/// `display: flex` を付与し、他の実 `menu-item`（`menu-sub` 併用を含む）
/// へは一切影響させない。
#[must_use]
pub fn menu_skeleton<'a>(show_icon: bool, attrs: Vec<(&'a str, &'a str)>) -> Node {
    let mut children = Vec::new();
    if show_icon {
        children.push(skeleton(
            &SkeletonProps {
                variant: SkeletonVariant::Circle,
                animation: SkeletonAnimation::default(),
            },
            vec![],
        ));
    }
    children.push(skeleton(&SkeletonProps::default(), vec![]));

    // 呼び出し側の `style`（モジュール doc「menu-skeleton」節が案内する
    // `--fandhe-skeleton-size` 等のカスタムプロパティ上書きの契約経路）と
    // 本関数固定のレイアウト宣言（`display:flex` 他）を単一の `style`
    // 属性へ統合する（codex-review P1 / Bugbot 指摘: 別々の
    // `("style", ...)` エントリをそのまま両方 attrs へ積むと同名属性が
    // 2 個出力される無効な HTML になり、HTML パーサーは先勝ち（後続の
    // 重複属性を無視）のため呼び出し側の指定が常に無効化されていた）。
    // `drop_class_attr` と異なりここでは `style` を丸ごと落とさない
    // （他の一部部品が使う `drop_style_attr`〔フレームワーク側固定・呼び
    // 出し側破棄〕とは意図的に異なる判断: `menu_skeleton` は
    // カスタムプロパティ経由の寸法上書きが唯一の公開 API のため、
    // 呼び出し側宣言を破棄すると契約そのものが機能しなくなる）。
    let caller_style = attrs
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case("style"))
        .map(|(_, v)| *v)
        .filter(|v| !v.is_empty());
    let base_style = "display:flex;align-items:center;gap:var(--fandhe-space-2)";
    let combined_style = match caller_style {
        Some(v) => format!("{base_style};{v}"),
        None => base_style.to_string(),
    };

    let mut merged: Vec<(&str, &str)> = vec![("style", combined_style.as_str())];
    merged.extend(
        drop_class_attr(attrs)
            .into_iter()
            .filter(|(k, _)| !k.eq_ignore_ascii_case("style")),
    );
    fandhe_frontend_headless_ui::sidebar::menu_item(merged, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text as core_text};

    fn expanded() -> Sidebar {
        Sidebar::new(SidebarState::Expanded)
    }

    #[test]
    fn stylesheet_is_deterministic_and_targets_data_scope_selectors() {
        let a = stylesheet();
        let b = stylesheet();
        assert_eq!(a, b);
        assert!(a.contains(r#"[data-scope="sidebar"][data-part="root"] {"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let out = stylesheet();
        assert!(!out.contains("</style"));
        assert!(!out.contains('<'));
    }

    #[test]
    fn stylesheet_never_generates_class_based_variant_classes() {
        let out = stylesheet();
        assert!(!out.contains("fd-sidebar--"));
    }

    #[test]
    fn all_parts_connect_to_headless_sidebar_scope() {
        let state = expanded();
        let props = SidebarProps::default();

        let html = render(&provider(&state, &props, vec![], vec![]));
        assert!(html.contains(r#"data-scope="sidebar" data-part="provider""#));

        let html = render(&root(&state, &props, "App sidebar", None, vec![], vec![]));
        assert!(html.contains(r#"data-scope="sidebar" data-part="root""#));

        let html = render(&header(vec![], vec![]));
        assert!(html.contains(r#"data-scope="sidebar" data-part="header""#));

        let html = render(&content(vec![], vec![]));
        assert!(html.contains(r#"data-scope="sidebar" data-part="content""#));

        let html = render(&footer(vec![], vec![]));
        assert!(html.contains(r#"data-scope="sidebar" data-part="footer""#));

        let html = render(&separator(vec![], vec![]));
        assert!(html.contains(r#"data-scope="sidebar" data-part="separator""#));

        let html = render(&input(vec![]));
        assert!(html.contains(r#"data-scope="sidebar" data-part="input""#));

        let html = render(&group(None, vec![], vec![]));
        assert!(html.contains(r#"data-scope="sidebar" data-part="group""#));

        let html = render(&group_label(None, vec![], vec![]));
        assert!(html.contains(r#"data-scope="sidebar" data-part="group-label""#));

        let html = render(&group_content(vec![], vec![]));
        assert!(html.contains(r#"data-scope="sidebar" data-part="group-content""#));

        let html = render(&group_action("Add", vec![], vec![]));
        assert!(html.contains(r#"data-scope="sidebar" data-part="group-action""#));

        let html = render(&menu(vec![], vec![]));
        assert!(html.contains(r#"data-scope="sidebar" data-part="menu""#));

        let html = render(&menu_item(vec![], vec![]));
        assert!(html.contains(r#"data-scope="sidebar" data-part="menu-item""#));

        let html = render(&menu_button(
            &SidebarMenuButtonProps::default(),
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="sidebar" data-part="menu-button""#));

        let html = render(&menu_action("Remove", vec![], vec![]));
        assert!(html.contains(r#"data-scope="sidebar" data-part="menu-action""#));

        let html = render(&menu_badge(vec![], vec![]));
        assert!(html.contains(r#"data-scope="sidebar" data-part="menu-badge""#));

        let html = render(&menu_sub(vec![], vec![]));
        assert!(html.contains(r#"data-scope="sidebar" data-part="menu-sub""#));

        let html = render(&menu_sub_item(vec![], vec![]));
        assert!(html.contains(r#"data-scope="sidebar" data-part="menu-sub-item""#));

        let html = render(&menu_sub_button(
            &SidebarMenuSubButtonProps::default(),
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="sidebar" data-part="menu-sub-button""#));

        let html = render(&rail(&state, "Toggle sidebar", vec![], vec![]));
        assert!(html.contains(r#"data-scope="sidebar" data-part="rail""#));

        let html = render(&trigger(&state, "Toggle sidebar", None, vec![], vec![]));
        assert!(html.contains(r#"data-scope="sidebar" data-part="trigger""#));

        let html = render(&inset(vec![], vec![]));
        assert!(html.contains(r#"data-scope="sidebar" data-part="inset""#));

        let html = render(&menu_skeleton(true, vec![]));
        assert!(html.contains(r#"data-scope="sidebar" data-part="menu-item""#));
        assert!(!html.contains("data-sidebar"));
    }

    #[test]
    fn caller_class_is_dropped_on_every_part() {
        let state = expanded();
        let props = SidebarProps::default();
        let html = render(&provider(
            &state,
            &props,
            vec![("class", "evil")],
            vec![
                root(
                    &state,
                    &props,
                    "App sidebar",
                    None,
                    vec![("class", "evil")],
                    vec![
                        header(vec![("class", "evil")], vec![]),
                        content(
                            vec![("class", "evil")],
                            vec![group(
                                None,
                                vec![("class", "evil")],
                                vec![
                                    group_label(None, vec![("class", "evil")], vec![]),
                                    group_action("Add", vec![("class", "evil")], vec![]),
                                    group_content(
                                        vec![("class", "evil")],
                                        vec![menu(
                                            vec![("class", "evil")],
                                            vec![menu_item(
                                                vec![("class", "evil")],
                                                vec![
                                                    menu_button(
                                                        &SidebarMenuButtonProps::default(),
                                                        vec![("class", "evil")],
                                                        vec![],
                                                    ),
                                                    menu_action(
                                                        "Remove",
                                                        vec![("class", "evil")],
                                                        vec![],
                                                    ),
                                                    menu_badge(vec![("class", "evil")], vec![]),
                                                    menu_sub(
                                                        vec![("class", "evil")],
                                                        vec![menu_sub_item(
                                                            vec![("class", "evil")],
                                                            vec![menu_sub_button(
                                                                &SidebarMenuSubButtonProps::default(
                                                                ),
                                                                vec![("class", "evil")],
                                                                vec![],
                                                            )],
                                                        )],
                                                    ),
                                                ],
                                            )],
                                        )],
                                    ),
                                ],
                            )],
                        ),
                        footer(vec![("class", "evil")], vec![]),
                        separator(vec![("class", "evil")], vec![]),
                        input(vec![("class", "evil")]),
                        rail(&state, "Toggle sidebar", vec![("class", "evil")], vec![]),
                        trigger(
                            &state,
                            "Toggle sidebar",
                            None,
                            vec![("class", "evil")],
                            vec![],
                        ),
                    ],
                ),
                inset(vec![("class", "evil")], vec![core_text("main")]),
            ],
        ));
        assert!(!html.contains("evil"));
        assert_eq!(html.matches("class=\"").count(), 0);
    }

    // codex-review P1 / Bugbot 指摘の回帰: `menu_skeleton` は呼び出し側
    // `style`（`--fandhe-skeleton-size` 等のカスタムプロパティ上書き、
    // モジュール doc「menu-skeleton」節の契約）を単一の `style` 属性へ
    // 統合しなければならない。別々の `style` 属性が 2 個出力されると HTML
    // パーサーは先勝ちで後続を無視するため、呼び出し側の指定が常に無効化
    // される（fix 前の実際の不具合）。
    #[test]
    fn menu_skeleton_merges_caller_style_into_single_attribute() {
        let html = render(&menu_skeleton(
            true,
            vec![("style", "--fandhe-skeleton-size: 4rem")],
        ));
        assert_eq!(html.matches("style=\"").count(), 1);
        assert!(html.contains("--fandhe-skeleton-size: 4rem"));
        assert!(html.contains("display:flex"));
    }

    #[test]
    fn menu_skeleton_without_caller_style_still_outputs_single_style_attribute() {
        let html = render(&menu_skeleton(false, vec![]));
        assert_eq!(html.matches("style=\"").count(), 1);
        assert!(html.contains("display:flex"));
    }
}
