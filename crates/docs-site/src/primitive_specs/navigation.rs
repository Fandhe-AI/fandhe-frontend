//! Primitives（`fandhe-frontend-headless-ui`）Navigation カテゴリ 11 部品の
//! 原稿データ（イシュー #1028、親トラッキング #1035 Phase 5）。
//!
//! # 役割・呼び出し文脈
//!
//! [`crate::primitive_specs::SPEC_TABLES`] から参照される
//! `path -> ComponentPageSpec` テーブル（[`SPECS`]）を保持する専用モジュール。
//! 本モジュール自体は生成物へ直接寄与せず、
//! `crate::component_page::render_component_page` が `spec_for` 経由で
//! 読み取り、Demo（[`crate::primitive_showcase::navigation`]、イシュー
//! #1022）・Anatomy・`data-*` 属性表（いずれも Demo から機械導出）と合成して
//! 6 節ページ（Demo → Features → Anatomy → API Reference → Examples →
//! Accessibility）を組み立てる。CSS 変数表は Primitives 層では恒常的に省略
//! される（`docs/design/docs-site-primitives-themes-split.md` §5）。
//!
//! 対象 11 部品: action-bar / breadcrumb / link / link-overlay / menu /
//! menubar / nav-list / navigation-menu / pagination / tabs / toolbar。
//!
//! # 一次情報と根拠表記（推測混入の禁止）
//!
//! 全行の根拠は `crates/headless-ui/src/<module>.rs` の実ソースのみとする。
//! 各定数の doc コメントには兄弟カテゴリ（Forms A/B/C・Overlay/Disclosure・
//! Data Display/Utilities の各 `primitive_specs` サブモジュール）と同じ
//! 形式で `file:line`（範囲）を付す。関数名・引数名を主、行番号は実行時点の
//! スナップショットとして付す従の補助情報とし（リファクタで行番号が陳腐化
//! しても関数名からソースへ再到達できることを優先する。行番号のみを根拠に
//! しない）。根拠を示せない項目（架空の引数・ARIA 属性・キー割り当て）は
//! 一切書かない（`.claude/rules/out-of-scope-tracking.md`・
//! `docs/design/docs-site-component-pages.md` の一次情報規約）。
//!
//! # `arguments`（引数表）の形式
//!
//! `ArgRow::name` は `<関数名>(<引数名>)` 形式（例: `trigger(open)`）とし、
//! 部品 mod 名は省略する（表内で部品名が自明なため）。全パート関数に共通
//! する `attrs`/`children` は行として載せない（Features の散文または引数の
//! 説明文で必要に応じて触れる）。`default` 列は `#[default]` を持つ enum
//! のみ記入し、それ以外は空文字列のままとする。
//!
//! # `examples`（Examples）と Demo の切り口分離
//!
//! `crate::primitive_showcase::navigation` の Demo（#1022）とは異なる切り口
//! を選ぶ（同じ構成の再掲を避ける）。使用する API はすべて
//! `fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui`（イシュー
//! #693 方針、`crates/docs-site` は headless-ui へ直接依存しない）経由の
//! headless-ui 公開関数のみであり、pre-styled-ui の部品関数は呼ばない。
//! レンダラは `h2`/`h3` を出力しない（右カラム目次の汚染を避ける、過去
//! 事故 #980）。class 属性は付与しない（`primitives-showcase` は Demo 専用
//! ラッパ class であり、Examples 節へ pre-styled-ui 専用 class
//! （`showcase-row` 等）を持ち込むと `tests/site_css_contract.rs` の class
//! 契約に反する）。
//!
//! # `keyboard`（キーボード表）
//!
//! `crates/headless-ui/src/` の Navigation 11 モジュールにキー名リテラル
//! （`ArrowUp` 等）は 1 件も存在しない（キーイベント配線は headless-ui の
//! 対象外、各モジュールの module doc が明示する「スコープ外」節参照）。
//! 書けるのは (a) ネイティブ要素（`a[href]`/`button`）由来の
//! `Tab`/`Shift+Tab` フォーカス移動、(b) `tabs`/`toolbar`/`menubar` の
//! roving tabindex（フォーカス対象以外へ `tabindex="-1"` を付与する）という
//! マークアップ上決定される事実、(c) 「矢印キー等のキーイベント配線は
//! headless-ui の対象外であり、`ToolbarAction`/`MenubarAction`/
//! `PaginationAction` のような状態遷移 API として利用者側（wasm ランタイム
//! 層）へ委ねられる」という 1 行、の 3 種類のみである。
//!
//! # 責務境界（`docs/policy/intentional-non-adoption.md` §3.25）
//!
//! バリデーション・送信処理・データ整形・永続化を部品が担うかのような説明は
//! 書かない。`pagination::page_range` のようにソースに実在する表示構造計算は
//! 事実として記載するが、「ページング処理を行う」といった越境表現にしない。
//!
//! # セキュリティ不変条件（REQ-1）
//!
//! 本モジュールはリテラル `&'static str` のみで [`ArgRow`]/[`AriaRow`]/
//! [`KeyRow`] を構築し、`raw_html()` や HTML 文字列の直接組み立て
//! （`format!("<td>{}</td>", …)`）を一切行わない。Examples レンダラも
//! [`fandhe_frontend_core::el`]/[`fandhe_frontend_core::text`] とその
//! タグヘルパーのみで組み立てる。実際のエスケープは `component_page.rs`
//! 側の `table`/`td`/`text` ノード木経由で `render()` が行う。
//!
//! # スコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - `menu`（イシュー #1651）・`menubar`（イシュー #1652）はいずれも
//!   全 anatomy パーツを `primitive_showcase::navigation` の Demo で描画済み
//!   であり、`tests/primitive_showcase.rs::KNOWN_UNCOVERED` に未網羅
//!   エントリを持たない（旧記述「`trigger-item`/`checkbox-item`/…は
//!   Demo 側未網羅」は両イシューの Demo 拡充で解消済み）。

use fandhe_frontend_core::{code, el, p, pre, text, Node};
use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui as hui;
use hui::action_bar;
use hui::breadcrumb;
use hui::data_attrs::Orientation;
use hui::link;
use hui::link_overlay;
use hui::menu;
use hui::menubar;
use hui::nav_list;
use hui::navigation_menu;
use hui::pagination::{self, ItemMode};
use hui::tabs::{tabs, ActivationMode, TabItem, TabsProps};
use hui::toolbar;
use hui::OpenState;

use crate::component_page::{ArgRow, AriaRow, ComponentPageSpec, ExampleEntry, KeyRow};

/// Examples 節の共通ラッパー（`div` のみ、class を持ち込まない）。
fn example_wrap(children: Vec<Node>) -> Node {
    el("div", vec![], children)
}

// ---------------------------------------------------------------------
// action-bar（`crates/headless-ui/src/action_bar.rs`）
// ---------------------------------------------------------------------

/// Demo（`primitive_showcase::navigation::action_bar_section`）と異なり、
/// Separator を挟まず単一の CloseTrigger のみを持つ最小構成にする。
fn ex_action_bar() -> Node {
    let state = OpenState::Open;
    example_wrap(vec![action_bar::root(
        state,
        vec![],
        vec![action_bar::positioner(
            state,
            vec![],
            vec![action_bar::content(
                state,
                "Bulk edit",
                vec![],
                vec![action_bar::close_trigger(vec![], vec![text("Done")])],
            )],
        )],
    )])
}

const ACTION_BAR_CUSTOM_CSS_SNIPPET: &str = "\
[data-scope=\"action-bar\"][data-part=\"positioner\"] {\n  \
  position: fixed;\n  bottom: 1.5rem;\n  left: 50%;\n  transform: translateX(-50%);\n\
}\n\
[data-scope=\"action-bar\"][data-part=\"content\"][data-state=\"open\"] {\n  \
  display: flex;\n  align-items: center;\n  gap: 0.75rem;\n  padding: 0.75rem 1rem;\n  \
  border-radius: 8px;\n  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.2);\n\
}\n\
[data-scope=\"action-bar\"][data-part=\"close-trigger\"]:focus-visible {\n  \
  outline: 2px solid #2563eb;\n  outline-offset: 2px;\n\
}\n";

/// 自前 CSS の最小例（イシュー #1647）。`positioner` の画面下部固定配置・
/// `content[data-state="open"]` のレイアウト・`close-trigger` の
/// `:focus-visible` を data-scope/data-part 属性セレクタのみで組み立てる。
/// headless-ui 自体はスタイルを持たない（本クレートは `docs-site` の
/// showcase CSS〔`assets/primitives-showcase.css`〕へは一切追加しない。
/// `pre > code` のテキストとして提示するのみで、実際にページへ適用は
/// しない）。
fn action_bar_custom_css_example() -> Node {
    let state = OpenState::Open;
    let markup = action_bar::root(
        state,
        vec![],
        vec![action_bar::positioner(
            state,
            vec![],
            vec![action_bar::content(
                state,
                "Bulk edit",
                vec![],
                vec![action_bar::close_trigger(vec![], vec![text("Done")])],
            )],
        )],
    );
    el(
        "div",
        vec![],
        vec![
            markup,
            pre(
                vec![],
                vec![code(vec![], vec![text(ACTION_BAR_CUSTOM_CSS_SNIPPET)])],
            ),
        ],
    )
}

/// `/primitives/action-bar/`。
///
/// 一次情報: `crates/headless-ui/src/action_bar.rs`（モジュール doc §参照
/// 基準）、`root`/`content`/`close_trigger`/`separator` の各シグネチャと
/// doc コメント（`role="dialog"`/`data-expanded`/`tabindex="-1"`/
/// `aria-label` 既定値/`role="separator"` の実出力）、`ActionBar` 状態機械の
/// doc。参照基準は chakra-ui `overlays/action-bar` のみ（ark-ui・Radix
/// Primitives に ActionBar 相当は存在せず、`docs/design/component-coverage-map.md`
/// も chakra-ui 単独参照）。chakra-ui の ActionBar は独自の状態機械を持たず
/// Ark Popover（zag.js `popover.connect`）を再利用しており、本部品の属性
/// 仕様はその content/close-trigger の出力を基準に据える（イシュー #1647、
/// 参照基準・是正点・意図的差分の全量は action_bar.rs モジュール doc
/// §参照基準 を正とする）。
pub(super) const ACTION_BAR: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "複数選択操作バー（chakra-ui ActionBar 相当。実体は Ark Popover の再利用）。Root / Positioner / Content / SelectionTrigger / Separator / CloseTrigger の 6 anatomy パーツを持つ（action_bar.rs モジュール doc）。",
        "開閉は Disclosure を埋め込んだ状態機械 ActionBar が管理する。選択件数から open を自動導出する糖衣 API は持たず、「選択操作 → 開閉状態の決定」は呼び出し側の責務とする（action_bar.rs「選択件数から open を導出する糖衣 API は持たない」節）。",
        "content に role=\"dialog\"（非モーダル、aria-modal は付与しない）と aria-label（読み上げ用ラベル、呼び出し側が指定する必須引数）を固定付与する。参照基準（zag.js popover の content）に合わせイシュー #1647 で role=\"toolbar\"（roving tabindex を伴わない不完全な適用だった）から是正した（**破壊的変更**、content 関数）。",
        "content は開状態のときのみ data-expanded 存在属性を出力し、tabindex=\"-1\" を固定付与する（chakra autoFocus: false に対応。開時にフォーカスを自動移動しない。呼び出し側 attrs に tabindex があれば出力しない、content 関数）。",
        "close-trigger は呼び出し側 attrs に aria-label が無く、かつ children が空（可視テキストを持たないボタン）のときに限り既定値 \"close\"（zag.js popover の translations.closeTrigger 既定値、CLOSE_TRIGGER_ARIA_LABEL）を出力する（close_trigger 関数。children に可視テキストがあれば既定 aria-label は付与しない）。",
        "closed のとき positioner/content の双方に hidden 存在属性を付与し、SSR/no-JS でも閉状態を表現する（positioner/content 関数）。",
        "参照基準に存在する data-placement/data-side（placement variant）は本実装のスコープ外（`docs/policy/intentional-non-adoption.md` §3.25 規則 2、装飾・レイアウト計測は headless-ui へ持ち込まない）。外側クリックでの閉鎖（closeOnInteractOutside）は既定 false のまま opt-in 属性を持たない（選択操作のチェックボックス等が ActionBar の外側に存在するため、誤閉鎖を防ぐ安全側の判断）。",
    ],
    arguments: &[
        ArgRow {
            name: "root(state)",
            kind: "OpenState",
            default: "",
            description: "開閉状態（Open/Closed）。data-state へ反映される。",
        },
        ArgRow {
            name: "content(label)",
            kind: "&str",
            default: "",
            description: "role=\"dialog\" の aria-label（選択操作バーの読み上げ名、必須引数）。",
        },
        ArgRow {
            name: "selection_trigger",
            kind: "fn",
            default: "",
            description: "type=\"button\" を固定付与するボタンパーツ。選択件数テキストは呼び出し側が children で渡す。",
        },
        ArgRow {
            name: "close_trigger(attrs)",
            kind: "Vec<(&str, &str)>",
            default: "",
            description: "aria-label 未指定かつ children が空のときのみ既定値 \"close\" を出力する（CLOSE_TRIGGER_ARIA_LABEL）。呼び出し側が aria-label を指定するか children に可視テキストを渡せば上書き・不出力になる。",
        },
    ],
    examples: &[
        ExampleEntry {
            title: "CloseTrigger のみの最小構成",
            description: "SelectionTrigger/Separator を省き、Content 直下に CloseTrigger だけを配置した例です。",
            render: ex_action_bar,
        },
        ExampleEntry {
            title: "自前 CSS の最小例",
            description: "data-scope / data-part 属性セレクタと :focus-visible 擬似クラスで見た目を組み立てる最小例です。headless-ui 自体はスタイルを持ちません。",
            render: action_bar_custom_css_example,
        },
    ],
    keyboard: &[
        KeyRow {
            key: "Escape",
            description: "content が開いている間、バーを閉じる（`fandhe-frontend-wasm-full` の `OverlayKind::ActionBar` が配線。`data-close-on-escape=\"false\"` で opt-out 可能）。",
        },
        KeyRow {
            key: "Tab / Shift+Tab",
            description: "ネイティブ要素（button）由来のフォーカス移動。開時に自動でフォーカスは移動しない（tabindex=\"-1\"、chakra autoFocus: false に対応）。",
        },
    ],
    aria: &[
        AriaRow {
            attribute: "role=\"dialog\"",
            description: "content に固定付与（非モーダル、aria-modal は付与しない）。aria-label とセットで出力される。参照基準（zag.js popover）に合わせイシュー #1647 で role=\"toolbar\" から是正済み。",
        },
        AriaRow {
            attribute: "data-expanded",
            description: "content が開状態のときのみ出力される存在属性（zag.js popover の content と同じ語彙）。",
        },
        AriaRow {
            attribute: "aria-label=\"close\"",
            description: "close-trigger の既定値。呼び出し側 attrs に aria-label が無く、かつ children が空（可視テキストを持たない）のときに限り出力される。呼び出し側が aria-label を指定するか children に可視テキストを渡せば出力されない。",
        },
        AriaRow {
            attribute: "role=\"separator\" / aria-orientation=\"vertical\"",
            description: "separator に固定付与（ActionBar のボタン列は横並びのため区切り線は縦向きになる。参照基準は素の div であり、a11y 上の superset として維持する意図的差分）。",
        },
    ],
    demo: None,
};

// ---------------------------------------------------------------------
// breadcrumb（`crates/headless-ui/src/breadcrumb.rs`）
// ---------------------------------------------------------------------

/// Demo（`breadcrumb_section`）は個別パーツ手組みで 7 パーツ全てを実演する
/// （イシュー #1648、ellipsis も含め機械導出 Anatomy 表が網羅される）。ここ
/// では切り口を変え、[`breadcrumb::breadcrumb`] 利便ビルダーへカスタム
/// separator（`›`）を渡す構成を実演する。
fn ex_breadcrumb() -> Node {
    let items = [
        breadcrumb::BreadcrumbItem {
            label: "Home",
            href: "https://example.com/",
        },
        breadcrumb::BreadcrumbItem {
            label: "Docs",
            href: "https://example.com/docs/",
        },
        breadcrumb::BreadcrumbItem {
            label: "Breadcrumb",
            href: "https://example.com/docs/breadcrumb/",
        },
    ];
    example_wrap(vec![breadcrumb::breadcrumb(None, &items, || {
        vec![text("›")]
    })])
}

/// 自前 CSS の最小例。headless-ui 自体はスタイルを持たないため、利用者が
/// `data-scope`/`data-part`/`data-current` 属性セレクタで見た目を組み立てる
/// 例を示す（`CHECKBOX_CUSTOM_CSS_SNIPPET`〔forms_a.rs〕と同型、イシュー
/// #1648）。CSS はテキストノード（[`code`]/[`pre`]）として既定エスケープを
/// 経由し、`crate::primitive_showcase` の専用スタイルシート（`[data-scope=`/
/// `[data-part=` を持たない契約、`tests/site_css_contract.rs`）へは追加しない。
const BREADCRUMB_CUSTOM_CSS_SNIPPET: &str = "\
[data-scope=\"breadcrumb\"][data-part=\"list\"] {\n  \
  display: flex;\n  gap: 0.5rem;\n  list-style: none;\n  padding: 0;\n  margin: 0;\n\
}\n\
[data-scope=\"breadcrumb\"][data-part=\"link\"] {\n  \
  color: #2563eb;\n  text-decoration: none;\n\
}\n\
[data-scope=\"breadcrumb\"][data-part=\"link\"]:focus-visible {\n  \
  outline: 2px solid #2563eb;\n  outline-offset: 2px;\n\
}\n\
[data-scope=\"breadcrumb\"][data-part=\"current-link\"][data-current] {\n  \
  color: #111827;\n  font-weight: 600;\n\
}\n\
[data-scope=\"breadcrumb\"][data-part=\"separator\"],\n\
[data-scope=\"breadcrumb\"][data-part=\"ellipsis\"] {\n  \
  color: #6b7280;\n\
}\n";

fn ex_breadcrumb_custom_css() -> Node {
    let markup = breadcrumb::root(
        None,
        vec![],
        vec![breadcrumb::list(
            vec![],
            vec![
                breadcrumb::item(
                    vec![],
                    vec![breadcrumb::link(
                        "https://example.com/",
                        vec![],
                        vec![text("Home")],
                    )],
                ),
                breadcrumb::separator(vec![], vec![text("/")]),
                breadcrumb::item(
                    vec![],
                    vec![breadcrumb::current_link(vec![], vec![text("Docs")])],
                ),
            ],
        )],
    );
    example_wrap(vec![
        markup,
        pre(
            vec![],
            vec![code(vec![], vec![text(BREADCRUMB_CUSTOM_CSS_SNIPPET)])],
        ),
    ])
}

/// `/primitives/breadcrumb/`。
///
/// 一次情報: `crates/headless-ui/src/breadcrumb.rs:1-52`（モジュール doc）、
/// `:71-135`（`root`/`link`/`current_link`/`separator`/`ellipsis`
/// シグネチャ）、`:105`（`aria-current="page"` の実出力）、`:114`/`:126`
/// （`role="presentation"`+`aria-hidden="true"` の実出力）。
pub(super) const BREADCRUMB: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "パンくずナビゲーション。root（nav）/ list（ol）/ item（li）/ link（a）/ current-link（span）/ separator（li）/ ellipsis（li）の 7 anatomy パーツを持つ（breadcrumb.rs モジュール doc）。",
        "末尾項目のみ current_link（非対話の span、aria-current=\"page\"+data-current 固定付与）として描画し、それ以外は link（a、href 遷移可能）として描画する利便ビルダー breadcrumb() を提供する。",
        "separator/ellipsis はいずれも role=\"presentation\"+aria-hidden=\"true\" で装飾扱いとし、スクリーンリーダーの読み上げから除外する（separator/ellipsis 関数）。",
        "root の aria-label は省略時 \"breadcrumb\"（DEFAULT_ARIA_LABEL）が既定値になる。",
        "参照実体は chakra-ui の Breadcrumb のみ（ark-ui/Radix Primitives/Radix Themes に対応部品なし）。anatomy 7 パーツ・WAI-ARIA とも一致し差分なし。data-current は本リポジトリが link/nav_list/pagination と共有する独自語彙（イシュー #1648 参照突合）。",
    ],
    arguments: &[
        ArgRow {
            name: "root(aria_label_value)",
            kind: "Option<&str>",
            default: "",
            description: "None のとき既定値 \"breadcrumb\" を aria-label へ出力する。",
        },
        ArgRow {
            name: "current_link",
            kind: "fn",
            default: "",
            description: "aria-current=\"page\"+data-current を常に付与する span パーツ（末尾項目用）。",
        },
        ArgRow {
            name: "breadcrumb(items)",
            kind: "&[BreadcrumbItem]",
            default: "",
            description: "label/href の組から nav > ol > (li + li)* を決定的に組み立てる利便ビルダー。空配列でも panic しない。",
        },
        ArgRow {
            name: "separator(children)",
            kind: "Vec<Node>",
            default: "",
            description: "区切り表現は呼び出し側が children で与える（固定文言を持たず、\"/\"/\"›\" 等を自由に選べる）。",
        },
    ],
    examples: &[
        ExampleEntry {
            title: "利便ビルダー + カスタム separator",
            description: "breadcrumb() 利便ビルダーへ separator_children で \"›\" を渡す構成です。",
            render: ex_breadcrumb,
        },
        ExampleEntry {
            title: "自前 CSS の最小例",
            description: "data-scope / data-part / data-current 属性セレクタで自前 CSS を当てる最小例です。headless-ui 自体はスタイルを持ちません。",
            render: ex_breadcrumb_custom_css,
        },
    ],
    keyboard: &[
        KeyRow {
            key: "Tab / Shift+Tab",
            description: "ネイティブ a 要素（link パーツ）由来のフォーカス移動。current-link（span）は非対話でフォーカス対象外。独自キーハンドラは持たない（参照サイトにもキーボード操作表はない）。",
        },
        KeyRow {
            key: "Enter",
            description: "フォーカス中の link（a）をブラウザ既定動作で遷移する。",
        },
    ],
    aria: &[
        AriaRow {
            attribute: "aria-label=\"breadcrumb\"",
            description: "root の既定値（省略可能、WAI-ARIA APG \"Breadcrumb\" と同義）。",
        },
        AriaRow {
            attribute: "aria-current=\"page\"",
            description: "current_link に固定付与（末尾項目の非対話 span）。",
        },
        AriaRow {
            attribute: "role=\"presentation\" / aria-hidden=\"true\"",
            description: "separator/ellipsis に固定付与し装飾要素として扱う。",
        },
    ],
    demo: None,
};

// ---------------------------------------------------------------------
// link（`crates/headless-ui/src/link.rs`）
// ---------------------------------------------------------------------

/// Demo（`link_section`）は 4 変種（通常/外部/現在ページ/文中インライン）を
/// 並べる（イシュー #1649）。ここでは Demo とは切り口を変え、
/// `external=false, current=false`（target/rel も aria-current/data-current
/// も出力しない最小構成）を単体で実演する。href は他ページとの内部リンク
/// 切れ検証（`crate::linkcheck::check_links`）を避けるため、Demo と同じく
/// `example.com`（RFC 2606 予約ドメイン）を使う。
fn ex_link() -> Node {
    example_wrap(vec![link::root(
        "https://example.com/docs/guide",
        false,
        false,
        vec![],
        vec![text("Guide")],
    )])
}

/// `current=true` の構成（`aria-current="page"`+`data-current` の実出力）。
/// 旧版（イシュー #1649 是正前）は本構成を「external=false の構成」という
/// 説明の下で実演しており、実演内容と説明が食い違っていた
/// （current 引数が true になっていたため）。本例で独立させて解消する。
fn ex_link_current() -> Node {
    example_wrap(vec![link::root(
        "https://example.com/docs/guide",
        false,
        true,
        vec![],
        vec![text("Guide (current page)")],
    )])
}

/// 文中インライン構成（chakra-ui のデモ「文中インラインリンク」相当）。
/// `link::root` は前後のテキストノードと並置してもエスケープ・anatomy 出力
/// が変わらないことを実演する。
fn ex_link_inline() -> Node {
    example_wrap(vec![el(
        "p",
        vec![],
        vec![
            text("See the "),
            link::root(
                "https://example.com/docs/guide",
                false,
                false,
                vec![],
                vec![text("guide")],
            ),
            text(" for setup steps."),
        ],
    )])
}

/// 自前 CSS の最小例。headless-ui 自体はスタイルを持たないため、利用者が
/// `data-scope`/`data-part`/`data-current`/`[aria-current="page"]` 属性
/// セレクタで見た目を組み立てる例を示す（`CHECKBOX_CUSTOM_CSS_SNIPPET`
/// 〔forms_a.rs〕と同型、イシュー #1649）。CSS はテキストノード
/// （[`code`]/[`pre`]）として既定エスケープを経由し、
/// `crate::primitive_showcase` の専用スタイルシート（`[data-scope=`/
/// `[data-part=` を持たない契約、`tests/site_css_contract.rs`）へは
/// 追加しない。
const LINK_CUSTOM_CSS_SNIPPET: &str = "\
[data-scope=\"link\"][data-part=\"root\"] {\n  \
  color: #2563eb;\n  text-decoration: none;\n\
}\n\
[data-scope=\"link\"][data-part=\"root\"]:hover {\n  \
  text-decoration: underline;\n\
}\n\
[data-scope=\"link\"][data-part=\"root\"]:focus-visible {\n  \
  outline: 2px solid #2563eb;\n  outline-offset: 2px;\n\
}\n\
[data-scope=\"link\"][data-part=\"root\"][aria-current=\"page\"] {\n  \
  color: #111827;\n  font-weight: 600;\n  text-decoration: none;\n\
}\n\
[data-scope=\"link\"][data-part=\"root\"][target=\"_blank\"]::after {\n  \
  content: \" ↗\";\n\
}\n";

fn ex_link_custom_css() -> Node {
    let markup = link::root(
        "https://example.com/docs/guide",
        false,
        true,
        vec![],
        vec![text("Guide (current page)")],
    );
    example_wrap(vec![
        markup,
        pre(
            vec![],
            vec![code(vec![], vec![text(LINK_CUSTOM_CSS_SNIPPET)])],
        ),
    ])
}

/// `/primitives/link/`。
///
/// 一次情報: `crates/headless-ui/src/link.rs:1-54`（モジュール doc）、
/// `:72-100`（`root` シグネチャ）、`:66-67`（`target="_blank"`+
/// `rel="noopener noreferrer"` の不可分付与）、`:69`（`aria-current="page"`
/// の実出力）。参考サイト突合（イシュー #1649）: chakra-ui Link
/// （`.agents/skills/chakra-ui/references/components/typography/link.md`）・
/// Radix Themes Link（`docs/design/radix-themes-survey.md:83`）はいずれも
/// スタイル prop のみの styled `a` で、Anatomy/Keyboard/`data-*` 節を持たない
/// （ark-ui・Radix Primitives に Link 相当は存在しない、
/// `docs/design/component-coverage-map.md:667`）。
pub(super) const LINK: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "汎用インラインリンク。root（a）1 パーツのみの最小構成（chakra-ui Link 相当、link.rs モジュール doc）。",
        "external=true のとき target=\"_blank\"+rel=\"noopener noreferrer\" を不可分に付与する（reverse tabnabbing 対策。片方のみを付与できる API は公開しない）。参考サイト（chakra-ui）は生の target/rel を利用者が渡す設計であり、本実装は API 側で不可分付与を保証する意図的差分。",
        "current=true のとき aria-current=\"page\"+data-current を付与する（breadcrumb/nav_list と同じ語彙を共有）。",
        "data-state/data-disabled/data-motion 等の状態 data-* は出力しない（§3.25 規則 2: 装飾・アニメーション関心を headless へ持ち込まない）。",
        "asChild（Slot 相当）・variant/colorPalette/size/underline/highContrast 等の装飾軸は本モジュールでは提供しない。asChild は intentional-non-adoption.md §3.25 表 Slot 行の保留により非採用、装飾軸は Themes 層 pre-styled-ui::link（/themes/link/）の責務。",
    ],
    arguments: &[
        ArgRow {
            name: "root(href)",
            kind: "&str",
            default: "",
            description: "遷移先 URL。危険な URL スキーム（javascript: 等）は core の render() が属性ごと拒否する。",
        },
        ArgRow {
            name: "root(external)",
            kind: "bool",
            default: "",
            description: "true のとき target=\"_blank\"+rel=\"noopener noreferrer\" を不可分に付与する。",
        },
        ArgRow {
            name: "root(current)",
            kind: "bool",
            default: "",
            description: "true のとき aria-current=\"page\"+data-current を付与する。",
        },
    ],
    examples: &[
        ExampleEntry {
            title: "external=false, current=false の構成",
            description: "external/current をともに false にして target=\"_blank\"/rel/aria-current/data-current のいずれも出力しない最小構成です。",
            render: ex_link,
        },
        ExampleEntry {
            title: "current=true の構成",
            description: "current を true にして aria-current=\"page\" と data-current を同時に出力する構成です。",
            render: ex_link_current,
        },
        ExampleEntry {
            title: "文中インライン",
            description: "段落テキストの中にリンクを混在させても anatomy 出力・エスケープが変わらないことを示す構成です。",
            render: ex_link_inline,
        },
        ExampleEntry {
            title: "自前 CSS の最小例",
            description: "data-scope/data-part/[aria-current=\"page\"] 属性セレクタで自前 CSS を当てる最小例です。headless-ui 自体はスタイルを持ちません。",
            render: ex_link_custom_css,
        },
    ],
    keyboard: &[KeyRow {
        key: "Tab / Shift+Tab",
        description: "ネイティブ要素（a[href]）由来のフォーカス移動（ブラウザ既定、headless-ui 側の配線なし）。",
    }, KeyRow {
        key: "Enter",
        description: "ネイティブ a[href] 由来の起動（リンク先へ遷移）。Space は起動しない（ブラウザ既定の a 要素の挙動）。",
    }],
    aria: &[
        AriaRow {
            attribute: "role / aria-*",
            description: "既定では付与しない。暗黙の link ロールに委ねる（参考サイトも独自付与しない）。",
        },
        AriaRow {
            attribute: "aria-current=\"page\"",
            description: "current 引数が true のときのみ data-current と同時に付与する。",
        },
        AriaRow {
            attribute: "rel=\"noopener noreferrer\"",
            description: "ARIA ではないがセキュリティ属性として併記する。external 引数が true のとき target=\"_blank\" と同時に付与する。",
        },
        AriaRow {
            attribute: "（危険スキーム時）",
            description: "javascript: 等の危険な URL スキームは core の render() が href 属性ごと拒否する。href を失った a はフォーカス不能になり、暗黙の link ロールも失う（fail-closed の意味論的帰結）。",
        },
    ],
    demo: None,
};

// ---------------------------------------------------------------------
// link-overlay（`crates/headless-ui/src/link_overlay.rs`）
// ---------------------------------------------------------------------

/// Demo（`link_overlay_section`）は「タイトル位置の overlay + 説明文 +
/// 内側の通常リンク」という chakra-ui のカードパターンを再現する
/// （イシュー #1650）。ここでは Demo と切り口を変え、module doc が推奨する
/// 「overlay へアクセシブルネームのみを与え、見出しは root の他の子ノード
/// として別途描画する」単一リンク構成を実演する。
fn ex_link_overlay() -> Node {
    example_wrap(vec![link_overlay::root(
        vec![],
        vec![
            el("h4", vec![], vec![text("Getting Started")]),
            text("A short summary of the guide."),
            link_overlay::overlay(
                "https://example.com/guides/getting-started/",
                vec![("aria-label", "Getting Started")],
                vec![],
            ),
        ],
    )])
}

/// chakra-ui LinkBox/LinkOverlay の代表例（タイトルが overlay 自身・
/// 説明文・内側リンクが残る構成）を実演する（イシュー #1650）。Demo
/// （`link_overlay_section`）と文言・内側リンクの有無等の切り口を変える。
fn ex_link_overlay_title_link() -> Node {
    example_wrap(vec![link_overlay::root(
        vec![],
        vec![
            p(
                vec![],
                vec![el(
                    "strong",
                    vec![],
                    vec![link_overlay::overlay(
                        "https://example.com/blog/framework-release-notes",
                        vec![],
                        vec![text("Framework release notes")],
                    )],
                )],
            ),
            p(
                vec![],
                vec![text(
                    "Highlights from the latest release, including breaking changes.",
                )],
            ),
            p(
                vec![],
                vec![el(
                    "a",
                    vec![("href", "https://example.com/authors/team")],
                    vec![text("By the framework team")],
                )],
            ),
        ],
    )])
}

/// 自前 CSS の最小例。headless-ui 自体はスタイルを持たないため、利用者が
/// `data-scope`/`data-part` 属性セレクタで `root` の位置決め・`overlay` の
/// 全面展開・内側リンクの前面化（chakra-ui の `LinkBox` が CSS の子孫
/// セレクタで行う挙動、headless 層は CSS を持たないため利用者 CSS の
/// 責務）を組み立てる例を示す（`LINK_CUSTOM_CSS_SNIPPET` と同型、イシュー
/// #1650）。CSS はテキストノード（[`code`]/[`pre`]）として既定エスケープを
/// 経由し、`crate::primitive_showcase` の専用スタイルシート（`[data-scope=`/
/// `[data-part=` を持たない契約、`tests/site_css_contract.rs`）へは追加
/// しない。
const LINK_OVERLAY_CUSTOM_CSS_SNIPPET: &str = "\
[data-scope=\"link-overlay\"][data-part=\"root\"] {\n  \
  position: relative;\n\
}\n\
[data-scope=\"link-overlay\"][data-part=\"overlay\"] {\n  \
  position: absolute;\n  inset: 0;\n  z-index: 0;\n\
}\n\
[data-scope=\"link-overlay\"][data-part=\"overlay\"]:focus-visible {\n  \
  outline: 2px solid #2563eb;\n  outline-offset: 2px;\n\
}\n\
[data-scope=\"link-overlay\"][data-part=\"root\"] a[href]:not([data-part=\"overlay\"]) {\n  \
  position: relative;\n  z-index: 1;\n\
}\n";

fn ex_link_overlay_custom_css() -> Node {
    // overlay は自前 CSS 例の CSS で position: absolute; inset: 0; となり
    // 通常フローから外れるため、可視タイトルを overlay の子ノードへ渡すと
    // root の高さを確立できない（module doc「全面拡張の実装方針」契約）。
    // ex_link_overlay と同様に、可視タイトルは通常フローの
    // 見出しとして別描画し、overlay へは aria-label のみでアクセシブル
    // ネームを与える（codex-review 指摘、イシュー #1650）。
    let markup = link_overlay::root(
        vec![],
        vec![
            p(
                vec![],
                vec![el("strong", vec![], vec![text("Custom CSS example")])],
            ),
            p(
                vec![],
                vec![el(
                    "a",
                    vec![("href", "https://example.com/authors/jane")],
                    vec![text("By Jane")],
                )],
            ),
            link_overlay::overlay(
                "https://example.com/articles/custom-css-example",
                vec![("aria-label", "Custom CSS example")],
                vec![],
            ),
        ],
    );
    example_wrap(vec![
        markup,
        pre(
            vec![],
            vec![code(vec![], vec![text(LINK_OVERLAY_CUSTOM_CSS_SNIPPET)])],
        ),
    ])
}

/// `/primitives/link-overlay/`。
///
/// 一次情報: `crates/headless-ui/src/link_overlay.rs:1-75`（モジュール doc
/// 「全面拡張の実装方針」`:12-29`・「参照突合」`:31-53`・「呼び出し文脈」
/// `:54-59` 節）、`:107-121`（`root`/`overlay` シグネチャ）。参考サイト突合
/// （イシュー #1650）: chakra-ui `LinkBox`/`LinkOverlay`
/// （`.agents/skills/chakra-ui/references/components/typography/link-overlay.md`）
/// は Anatomy/Keyboard/`data-*` 節を一切持たない styled 部品で、本実装の
/// `root`/`overlay` 2 パーツ・`data-scope`/`data-part` は参照側に概念が
/// 存在しない superset（過不足なし）。ark-ui の `link-overlay` ページは
/// 404 で実在せず、Radix Primitives/Radix Themes にも対応部品なし
/// （`docs/design/component-coverage-map.md:668`）。
pub(super) const LINK_OVERLAY: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "カード全面クリック化。root（div、位置決めコンテキスト）/ overlay（a、カード全面へ拡張されるリンク）の 2 パーツ構成（chakra-ui LinkBox/LinkOverlay パターン相当、link_overlay.rs モジュール doc）。",
        "::before 疑似要素を使わず、overlay 自身を position: absolute; inset: 0; で root 全面へ展開する方式を採る（styled 層の CSS 責務、headless 層は CSS を持たない）。",
        "overlay がフローから外れるため、root の高さは overlay 以外の子ノード（見出し・説明文等の通常フロー要素）が確立する契約である（module doc「全面拡張の実装方針」）。",
        "単一リンクのみの overlay へ可視テキストを渡す代わりに、見出しを root の他の子ノードとして描画し overlay へは aria-label 等でアクセシブルネームのみを与える運用が推奨される。",
        "data-state/data-disabled/data-motion 等の状態 data-* は出力しない（参考サイト〔chakra-ui〕も同様に状態 data-* を持たない、§3.25 規則 2: 装飾・アニメーション関心を headless へ持ち込まない）。",
        "参考サイト（chakra-ui LinkBox/LinkOverlay）は Anatomy/Keyboard/data-* 節を持たず、data-scope/data-part は本実装の superset。ark-ui・Radix Primitives・Radix Themes に対応部品なし。",
        "external（旧 isExternal）・asChild は非提供（意図的差分）。target/rel を attrs 経由で渡す場合は両方を同時に付与する運用を利用者側で行う（link::root の不可分保証は本部品には及ばない）。",
        "内側リンクの前面化（chakra-ui の LinkBox が子孫セレクタで行う挙動）は headless 層が CSS を持たないため利用者 CSS の責務（自前 CSS 例を参照）。",
        "overlay の DOM 位置がタブ順を決める（chakra-ui はタイトル位置に置く運用を推奨）。",
        "呼び出し側 attrs の href は予約キーとして除去される（同名なりすましの二重出力防止、イシュー #1650）。",
    ],
    arguments: &[ArgRow {
        name: "overlay(href)",
        kind: "&str",
        default: "",
        description: "遷移先 URL。危険な URL スキームは core の render() が属性ごと拒否する。",
    }],
    examples: &[
        ExampleEntry {
            title: "見出しを別描画し overlay へ aria-label のみ与える構成",
            description: "module doc が推奨する運用（可視見出しは root の子として描画し、overlay へはアクセシブルネームのみを渡す）を実演します。",
            render: ex_link_overlay,
        },
        ExampleEntry {
            title: "タイトル + 説明文 + 内側リンクの構成（chakra-ui パターン）",
            description: "参考サイト（chakra-ui LinkBox/LinkOverlay）の典型構成（タイトル位置の overlay・説明文・内側の通常リンク）を実演します。",
            render: ex_link_overlay_title_link,
        },
        ExampleEntry {
            title: "自前 CSS の最小例",
            description: "data-scope/data-part 属性セレクタで root の位置決め・overlay の全面展開・内側リンクの前面化を組み立てる最小例です。headless-ui 自体はスタイルを持ちません。",
            render: ex_link_overlay_custom_css,
        },
    ],
    keyboard: &[
        KeyRow {
            key: "Tab / Shift+Tab",
            description: "ネイティブ a[href]（overlay パーツ）由来のフォーカス移動。headless-ui 側の配線なし。root 内に内側リンクを併置する場合、フォーカス順は DOM 順に従う。",
        },
        KeyRow {
            key: "Enter",
            description: "フォーカス中の overlay（a）をブラウザ既定動作で遷移する。Space は起動しない（ブラウザ既定の a 要素の挙動）。",
        },
    ],
    aria: &[
        AriaRow {
            attribute: "role / aria-*",
            description: "既定では独自付与しない。暗黙の link ロールに委ねる（参考サイト〔chakra-ui〕も独自付与しない）。",
        },
        AriaRow {
            attribute: "aria-label",
            description: "overlay 自身は aria-label を自動付与しない。単一リンクのみで完結する構成では呼び出し側が attrs 経由でアクセシブルネームを与える運用が推奨される（module doc 参照）。",
        },
        AriaRow {
            attribute: "（危険スキーム時）",
            description: "javascript: 等の危険な URL スキームは core の render() が href 属性ごと拒否する。href を失った a はフォーカス不能になり、暗黙の link ロールも失う（fail-closed の意味論的帰結）。",
        },
        AriaRow {
            attribute: "（ポインタ操作の注記）",
            description: "overlay が root 全面へ absolute 展開されるため、root 内のテキストをポインタで選択しにくくなる（参考サイト〔chakra-ui〕にも同様の注記がある）。",
        },
    ],
    demo: None,
};

// ---------------------------------------------------------------------
// menu（`crates/headless-ui/src/menu.rs`）
// ---------------------------------------------------------------------

/// Demo（`menu_section`）は ItemGroup + Separator + CheckboxItem/
/// RadioItemGroup + サブメニューまで含む全機能構成。ここでは
/// ItemGroup を使わず単純な Item 列のみの最小構成を実演する（切り口分離）。
fn ex_menu() -> Node {
    let state = OpenState::Open;
    example_wrap(vec![menu::root(
        state,
        vec![],
        vec![
            menu::trigger(
                state,
                false,
                Some("ex-menu-content"),
                vec![],
                vec![text("Options")],
            ),
            menu::positioner(
                state,
                vec![],
                vec![menu::content(
                    state,
                    Some("ex-menu-content"),
                    None,
                    vec![],
                    vec![
                        menu::item("share", false, false, vec![], vec![text("Share")]),
                        menu::item("archive", true, false, vec![], vec![text("Archive")]),
                    ],
                )],
            ),
        ],
    )])
}

/// ItemIndicator/ItemText（イシュー #1651 で追加）を CheckboxItem/RadioItem
/// と組み合わせる例。
fn ex_menu_checkbox_and_radio() -> Node {
    let state = OpenState::Open;
    example_wrap(vec![menu::root(
        state,
        vec![],
        vec![
            menu::trigger(
                state,
                false,
                Some("ex-menu-check-content"),
                vec![],
                vec![text("View")],
            ),
            menu::positioner(
                state,
                vec![],
                vec![menu::content(
                    state,
                    Some("ex-menu-check-content"),
                    None,
                    vec![],
                    vec![
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
                            None,
                            vec![],
                            vec![
                                menu::radio_item(
                                    true,
                                    "light",
                                    false,
                                    false,
                                    vec![],
                                    vec![
                                        menu::item_indicator(true, vec![], vec![text("●")]),
                                        menu::item_text(false, false, vec![], vec![text("Light")]),
                                    ],
                                ),
                                menu::radio_item(
                                    false,
                                    "dark",
                                    false,
                                    false,
                                    vec![],
                                    vec![
                                        menu::item_indicator(false, vec![], vec![text("●")]),
                                        menu::item_text(false, false, vec![], vec![text("Dark")]),
                                    ],
                                ),
                            ],
                        ),
                    ],
                )],
            ),
        ],
    )])
}

/// `data-scope="menu"` セレクタで自前 CSS を当てる最小例（`LINK_CUSTOM_CSS_SNIPPET`
/// と同型、イシュー #1651）。`[hidden]` を持つパーツ（positioner/content/
/// item-indicator）へ `display: none` ガードを必ず含める（headless-ui は
/// `hidden` 存在属性のみで表示制御を行い、CSS 側の `display` は利用者が
/// 明示する契約のため）。
const MENU_CUSTOM_CSS_SNIPPET: &str = "\
[data-scope=\"menu\"][data-part=\"positioner\"][hidden],\n\
[data-scope=\"menu\"][data-part=\"content\"][hidden],\n\
[data-scope=\"menu\"][data-part=\"item-indicator\"][hidden] {\n  \
  display: none;\n\
}\n\
[data-scope=\"menu\"][data-part=\"item\"][data-highlighted] {\n  \
  background: #eff6ff;\n\
}\n\
[data-scope=\"menu\"][data-part=\"item\"][data-disabled] {\n  \
  color: #9ca3af;\n\
}\n\
[data-scope=\"menu\"][data-part=\"checkbox-item\"][data-state=\"checked\"] {\n  \
  font-weight: 600;\n\
}\n";

fn ex_menu_custom_css() -> Node {
    let state = OpenState::Open;
    let markup = menu::root(
        state,
        vec![],
        vec![
            menu::trigger(
                state,
                false,
                Some("ex-menu-css-content"),
                vec![],
                vec![text("Actions")],
            ),
            menu::positioner(
                state,
                vec![],
                vec![menu::content(
                    state,
                    Some("ex-menu-css-content"),
                    None,
                    vec![],
                    vec![
                        menu::item("rename", false, true, vec![], vec![text("Rename")]),
                        menu::item("delete", true, false, vec![], vec![text("Delete")]),
                    ],
                )],
            ),
        ],
    );
    example_wrap(vec![
        markup,
        pre(
            vec![],
            vec![code(vec![], vec![text(MENU_CUSTOM_CSS_SNIPPET)])],
        ),
    ])
}

/// `/primitives/menu/`。
///
/// 一次情報: `crates/headless-ui/src/menu.rs:1-84`（モジュール doc、
/// イシュー #1651 で「参考サイトとの意図的な差分」節を追加）、
/// `:104-471`（`root`/`trigger`/`content`/`item`/`item_text`/
/// `item_indicator`/`item_group`/`separator`/`trigger_item`/`checkbox_item`/
/// `radio_item` シグネチャ）、`:114`/`:185`/`:226`/`:255`/`:369`
/// （`aria-haspopup="menu"`/`role="menu"`/`role="menuitem"`/`role="group"`/
/// `aria-checked` の実出力）。キーボードの一次情報は
/// `crates/wasm-full/src/keynav.rs`（`handle_menu_or_select_trigger_keydown`/
/// `matching_keydown_target`/`wire_keynav`）、click 合成の一次情報は
/// `crates/wasm-full/src/headless.rs`（`MAPPING_TABLE`）。
pub(super) const MENU: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "トリガー起点のオーバーレイ + アクション項目リスト。Root / Trigger / Indicator / Positioner / Content / Arrow / ArrowTip / Item / ItemText / ItemIndicator / ItemGroup / ItemGroupLabel / Separator / TriggerItem / ContextTrigger / CheckboxItem / RadioItemGroup / RadioItem の 18 anatomy パーツを持つ（menu.rs モジュール doc。ItemText/ItemIndicator はイシュー #1651 で参考サイト〔ark-ui/Radix Primitives/chakra-ui〕と突合し追加）。",
        "各パーツ関数は呼び出し側 attrs に含まれる固定属性キー（data-state/role/aria-* 等）を除去してから合成する（drop_reserved、イシュー #1651）。id/aria-labelledby/aria-controls のような Option 引数経由の正規キーは除去対象に含まない。",
        "サブメニューは親 Menu インスタンスの content 内に子 Menu インスタンス由来の TriggerItem/Positioner/Content を入れ子で配置して表現し、親子双方に aria-haspopup=\"menu\" を付与する（「haspopup 連鎖」でネストを支援技術へ伝える）。",
        "CheckboxItem/RadioItemGroup は Menu の開閉状態とは独立した checked 状態機械（MenuCheckboxItem/MenuRadioItemGroup、Checkable/SingleSelect を埋め込む）を持つ。",
        "ContextTrigger は右クリック起点のトリガーであり、ARIA 属性を一切付与しない（右クリックは SSR/no-JS では成立せず、ARIA を付けると JS なしで実現できない操作性を誤って約束するため）。",
        "開閉は Disclosure を埋め込んだ状態機械 Menu が管理する（dispatch は \"open\"/\"close\"/\"toggle\"）。",
        "参考サイトとの意図的な差分（イシュー #1651）: Portal（DOM 配置）/ data-placement・data-side・data-align の positioner 集約 / data-orientation（content・item）/ chakra ItemCommand（ショートカット表示）/ asChild はいずれも非採用。キーボードは trigger にフォーカスを留めたまま aria-activedescendant + data-highlighted で仮想フォーカスを表現する設計（#583）のため、Escape 後の「trigger へのフォーカス復帰」は構造的に不要で、Tab は無配線（zag は Tab で閉じるが本実装は閉じない）。",
        "checkbox-item/radio-item への Enter/Space（click 合成による checked トグル dispatch）は crates/wasm-full/src/headless.rs の MAPPING_TABLE に行が無く未実装（イシュー #1651 時点）。",
    ],
    arguments: &[
        ArgRow {
            name: "trigger(disabled)",
            kind: "bool",
            default: "",
            description: "true のとき native disabled 存在属性 + data-disabled を付与する。",
        },
        ArgRow {
            name: "trigger(controls)",
            kind: "Option<&str>",
            default: "",
            description: "Some のとき aria-controls で content と関連付ける。",
        },
        ArgRow {
            name: "item(value)",
            kind: "&str",
            default: "",
            description: "data-value として出力される項目識別値。",
        },
        ArgRow {
            name: "item(disabled)",
            kind: "bool",
            default: "",
            description: "true のとき aria-disabled=\"true\"+data-disabled を付与する（div ベースのため native disabled は持たない）。",
        },
        ArgRow {
            name: "item(highlighted)",
            kind: "bool",
            default: "",
            description: "キーボードナビゲーションのフォーカス位置。true のとき data-highlighted を付与する。",
        },
        ArgRow {
            name: "item_text(disabled) / item_text(highlighted)",
            kind: "bool",
            default: "",
            description: "親 item 系パーツの状態を data-disabled/data-highlighted として装飾用に複製する（checked 状態は持たない）。",
        },
        ArgRow {
            name: "item_indicator(checked)",
            kind: "bool",
            default: "",
            description: "checked_data_state を data-state へ反映し、false のとき hidden 存在属性を付与する。aria-hidden=\"true\" を固定付与（装飾アイコン、二重読み上げ防止）。",
        },
        ArgRow {
            name: "trigger_item(sub_state)",
            kind: "OpenState",
            default: "",
            description: "このトリガーが開閉するサブメニュー側の状態（呼び出し側は子 Menu インスタンスの state() を注入する）。",
        },
        ArgRow {
            name: "checkbox_item(checked) / radio_item(checked)",
            kind: "bool",
            default: "",
            description: "role=\"menuitemcheckbox\"/\"menuitemradio\" の aria-checked（true/false のみ、indeterminate 非対応）。",
        },
    ],
    examples: &[
        ExampleEntry {
            title: "ItemGroup を使わない単純な Item 列",
            description: "ItemGroup/Separator を省き、Item のみを並べた最小構成です。",
            render: ex_menu,
        },
        ExampleEntry {
            title: "CheckboxItem/RadioItem + ItemIndicator/ItemText",
            description: "ItemIndicator/ItemText（イシュー #1651 で追加）を CheckboxItem・RadioItemGroup と組み合わせる例です。",
            render: ex_menu_checkbox_and_radio,
        },
        ExampleEntry {
            title: "自前 CSS の最小例",
            description: "data-scope/data-part 属性セレクタで自前 CSS を当てる最小例です。headless-ui 自体はスタイルを持ちません。",
            render: ex_menu_custom_css,
        },
    ],
    keyboard: &[
        KeyRow {
            key: "ArrowDown / ArrowUp / Enter / Space（closed の trigger）",
            description: "Menu を開き、先頭（ArrowUp は末尾）の非 disabled 項目を highlight する（handle_menu_or_select_trigger_keydown、crates/wasm-full/src/keynav.rs）。",
        },
        KeyRow {
            key: "ArrowDown / ArrowUp / Home / End（open）",
            description: "highlight 中の項目を次/前/先頭/末尾の非 disabled 項目へ移動する（data-highlighted + content の aria-activedescendant を更新）。既定は循環なし、content の data-loop-focus=\"true\" で有効化できる。",
        },
        KeyRow {
            key: "Enter / Space（open、highlight 中の項目）",
            description: "highlight 中の項目へ click を合成する。item は利用者の click ハンドラへ、trigger-item は \"toggle\" を dispatch する。checkbox-item/radio-item は crates/wasm-full/src/headless.rs::MAPPING_TABLE に対応行が無く、checked トグルは dispatch されない（未実装）。",
        },
        KeyRow {
            key: "印字可能文字",
            description: "typeahead（350ms バッファ）。item-text 子があればそのテキストを優先してマッチする。",
        },
        KeyRow {
            key: "ArrowRight",
            description: "highlight 中の項目が trigger-item かつ非 disabled・サブメニューが解決できるときのみサブメニューを展開する（それ以外は menubar 層のトリガー間移動へフォールバックする合図を返す、イシュー #1073）。",
        },
        KeyRow {
            key: "ArrowLeft",
            description: "サブメニュー内で親 trigger-item へ復帰しサブメニューを閉じる（トップレベルでは menubar 層のトリガー間移動へフォールバックする合図を返す）。",
        },
        KeyRow {
            key: "Escape",
            description: "最上位オーバーレイを閉じる（overlay::close_on_escape_for、data-close-on-escape=\"false\" で無効化できる）。本実装はフォーカスを trigger から離さない設計のため、参考サイトの「Escape 後に trigger へフォーカス復帰」と結果同等になる。",
        },
        KeyRow {
            key: "Tab / Shift+Tab",
            description: "配線なし（ブラウザ既定）。zag は Tab で閉じるが本実装は閉じない。",
        },
    ],
    aria: &[
        AriaRow {
            attribute: "aria-haspopup=\"menu\"",
            description: "trigger（およびサブメニューの trigger-item）に固定付与。",
        },
        AriaRow {
            attribute: "aria-expanded / aria-controls",
            description: "trigger に付与。開閉状態と対応する content の id を表す。",
        },
        AriaRow {
            attribute: "role=\"menu\"",
            description: "content に固定付与。aria-labelledby は labelledby が Some のときのみ出力される。",
        },
        AriaRow {
            attribute: "role=\"menuitem\" / role=\"menuitemcheckbox\" / role=\"menuitemradio\"",
            description: "item/checkbox-item/radio-item にそれぞれ固定付与。",
        },
        AriaRow {
            attribute: "role=\"group\"",
            description: "item-group/radio-item-group に固定付与。labelledby が Some のときのみ aria-labelledby が付与される。",
        },
        AriaRow {
            attribute: "aria-checked",
            description: "checkbox-item/radio-item に付与（true/false のみ、indeterminate 非対応）。",
        },
        AriaRow {
            attribute: "aria-hidden=\"true\"",
            description: "item-indicator に固定付与（装飾アイコン、親 checkbox-item/radio-item 自身の aria-checked と重複読み上げしないため）。",
        },
        AriaRow {
            attribute: "aria-orientation=\"horizontal\"",
            description: "separator に固定付与。",
        },
        AriaRow {
            attribute: "aria-disabled=\"true\"",
            description: "item/trigger-item/checkbox-item/radio-item の disabled 時に付与（div ベースのため native disabled は持たない）。",
        },
        AriaRow {
            attribute: "aria-activedescendant",
            description: "content に付与（実行時に crates/wasm-full/src/keynav.rs が書く。SSR 出力は行わない）。",
        },
        AriaRow {
            attribute: "（ContextTrigger の ARIA 省略）",
            description: "右クリックは SSR/no-JS では成立せず、ARIA を付けると JS なしで実現できない操作性を誤って約束するため、context-trigger は ARIA 属性を一切持たない。",
        },
    ],
    demo: None,
};

// ---------------------------------------------------------------------
// menubar（`crates/headless-ui/src/menubar.rs`）
// ---------------------------------------------------------------------

/// Demo（`menubar_section`）は単一 Menu（File）のみ。ここでは 2 つ目の
/// Menu（Edit、closed）を並べ、複数 Menu をまたぐ roving tabindex の構造を
/// 実演する。
fn ex_menubar() -> Node {
    let orientation = Orientation::Horizontal;
    let open = OpenState::Open;
    let closed = OpenState::Closed;
    example_wrap(vec![menubar::root(
        orientation,
        "Example menu",
        vec![],
        vec![
            menubar::menu(
                open,
                vec![],
                vec![menubar::trigger(
                    true,
                    open,
                    false,
                    false,
                    0,
                    None,
                    vec![],
                    vec![text("File")],
                )],
            ),
            menubar::menu(
                closed,
                vec![],
                vec![menubar::trigger(
                    false,
                    closed,
                    false,
                    false,
                    1,
                    None,
                    vec![],
                    vec![text("Edit")],
                )],
            ),
        ],
    )])
}

/// checkbox-item/radio-item-group/radio-item + item-indicator/item-text の
/// 組み立てを実演する（menu の `ex_menu`（checkbox/radio 節）と同型、
/// イシュー #1652）。
fn ex_menubar_checkbox_and_radio() -> Node {
    let open = OpenState::Open;
    example_wrap(vec![menubar::root(
        Orientation::Horizontal,
        "Example menu",
        vec![],
        vec![menubar::menu(
            open,
            vec![],
            vec![
                menubar::trigger(
                    true,
                    open,
                    false,
                    false,
                    0,
                    Some("ex-menubar-checkbox-radio-content"),
                    vec![],
                    vec![text("View")],
                ),
                menubar::positioner(
                    open,
                    vec![],
                    vec![menubar::content(
                        open,
                        Some("ex-menubar-checkbox-radio-content"),
                        None,
                        vec![],
                        vec![
                            menubar::checkbox_item(
                                true,
                                "word-wrap",
                                false,
                                false,
                                vec![],
                                vec![
                                    menubar::item_indicator(true, vec![], vec![text("✓")]),
                                    menubar::item_text(
                                        false,
                                        false,
                                        vec![],
                                        vec![text("Word Wrap")],
                                    ),
                                ],
                            ),
                            menubar::separator(vec![], vec![]),
                            menubar::radio_item_group(
                                None,
                                vec![],
                                vec![
                                    menubar::radio_item(
                                        true,
                                        "grid",
                                        false,
                                        false,
                                        vec![],
                                        vec![
                                            menubar::item_indicator(true, vec![], vec![text("●")]),
                                            menubar::item_text(
                                                false,
                                                false,
                                                vec![],
                                                vec![text("Grid")],
                                            ),
                                        ],
                                    ),
                                    menubar::radio_item(
                                        false,
                                        "list",
                                        false,
                                        false,
                                        vec![],
                                        vec![
                                            menubar::item_indicator(false, vec![], vec![text("●")]),
                                            menubar::item_text(
                                                false,
                                                false,
                                                vec![],
                                                vec![text("List")],
                                            ),
                                        ],
                                    ),
                                ],
                            ),
                        ],
                    )],
                ),
            ],
        )],
    )])
}

/// `data-scope="menubar"` セレクタで自前 CSS を当てる最小例
/// （`MENU_CUSTOM_CSS_SNIPPET` と同型、イシュー #1652）。`[hidden]` を持つ
/// パーツ（positioner/content/sub-content/item-indicator）へ
/// `display: none` ガードを必ず含める。
const MENUBAR_CUSTOM_CSS_SNIPPET: &str = "\
[data-scope=\"menubar\"][data-part=\"positioner\"][hidden],\n\
[data-scope=\"menubar\"][data-part=\"content\"][hidden],\n\
[data-scope=\"menubar\"][data-part=\"sub-content\"][hidden],\n\
[data-scope=\"menubar\"][data-part=\"item-indicator\"][hidden] {\n  \
  display: none;\n\
}\n\
[data-scope=\"menubar\"][data-part=\"trigger\"][data-state=\"open\"] {\n  \
  background: #eff6ff;\n\
}\n\
[data-scope=\"menubar\"][data-part=\"item\"][data-highlighted] {\n  \
  background: #eff6ff;\n\
}\n\
[data-scope=\"menubar\"][data-part=\"item\"][data-disabled] {\n  \
  color: #9ca3af;\n\
}\n\
[data-scope=\"menubar\"][data-part=\"checkbox-item\"][data-state=\"checked\"] {\n  \
  font-weight: 600;\n\
}\n";

fn ex_menubar_custom_css() -> Node {
    let open = OpenState::Open;
    let markup = menubar::root(
        Orientation::Horizontal,
        "Example menu",
        vec![],
        vec![menubar::menu(
            open,
            vec![],
            vec![
                menubar::trigger(
                    true,
                    open,
                    false,
                    false,
                    0,
                    Some("ex-menubar-css-content"),
                    vec![],
                    vec![text("File")],
                ),
                menubar::positioner(
                    open,
                    vec![],
                    vec![menubar::content(
                        open,
                        Some("ex-menubar-css-content"),
                        None,
                        vec![],
                        vec![
                            menubar::item("new", false, true, vec![], vec![text("New")]),
                            menubar::item("close", true, false, vec![], vec![text("Close")]),
                        ],
                    )],
                ),
            ],
        )],
    );
    example_wrap(vec![
        markup,
        pre(
            vec![],
            vec![code(vec![], vec![text(MENUBAR_CUSTOM_CSS_SNIPPET)])],
        ),
    ])
}

/// `/primitives/menubar/`。
///
/// 一次情報: `crates/headless-ui/src/menubar.rs:1-210`（モジュール doc、
/// イシュー #1652 で「参考サイトとの意図的な差分」節を追加）、
/// `:225-786`（`root`/`menu`/`trigger`/`positioner`/`content`/`arrow`/
/// `arrow_tip`/`item`/`item_text`/`item_indicator`/`item_group`/
/// `item_group_label`/`separator`/`sub_trigger`/`sub_content`/
/// `checkbox_item`/`radio_item_group`/`radio_item` シグネチャ）、`role`
/// 実出力は各パーツ関数内（`role("menubar")`/`role("none")`/
/// `role("menuitem")`/`role("menu")`/`role("menuitemcheckbox")`/
/// `role("menuitemradio")`/`role("group")`/`role("separator")`）、
/// `:703`（`MenubarAction`）、`:754`（`Menubar`）、`:1009`（`decode_action`）。
pub(super) const MENUBAR: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "複数 Menu を水平（または垂直）に並べるコンテナ。Root / Menu / Trigger / Positioner / Content / Arrow / ArrowTip / Item / ItemText / ItemIndicator / ItemGroup / ItemGroupLabel / Separator / SubTrigger / SubContent / CheckboxItem / RadioItemGroup / RadioItem の 18 anatomy パーツを持つ（menubar.rs モジュール doc。Arrow/ArrowTip/ItemText/ItemIndicator/CheckboxItem/RadioItemGroup/RadioItem はイシュー #1652 で Radix Primitives Menubar と突合し追加、11 → 18 パーツ）。",
        "roving tabindex（focused/trigger_count/open/loop_focus/orientation の複合状態機械 Menubar）。フォーカス対象のトリガーのみ tabindex=\"0\"、それ以外は tabindex=\"-1\" になる。",
        "開いている Menu を跨いだ左右移動: ある Menu が開いた状態で Next/Prev/First/Last/Focus を dispatch すると、フォーカス移動と同時に開く Menu も隣へ移る（menubar 特有の挙動、toolbar の roving tabindex にはない）。",
        "menu パーツは role=\"none\" を固定付与し、role=\"menubar\" の子として menuitem/group 以外の要素を挟まないようにする（WAI-ARIA APG の menubar パターン）。",
        "既存の menu モジュールの anatomy はそのまま再利用せず data-scope=\"menubar\" を独自に持つ。状態機械の値語彙（OpenState/aria/data-* ヘルパ）と checkbox_item/radio_item の checked 値語彙（checked_data_state）のみを再利用する。",
        "全パーツで呼び出し側 attrs による固定属性（role/aria-*/data-*/tabindex）の偽装を drop_reserved で除去する（イシュー #1652、A05）。",
    ],
    arguments: &[
        ArgRow {
            name: "root(orientation)",
            kind: "Orientation",
            default: "Orientation::Horizontal",
            description: "role=\"menubar\" に付与する向き。aria-orientation/data-orientation の両方へ反映される。",
        },
        ArgRow {
            name: "root(label)",
            kind: "&str",
            default: "",
            description: "root に付与する aria-label（空文字列のときは省略）。",
        },
        ArgRow {
            name: "trigger(focused)",
            kind: "bool",
            default: "",
            description: "true のとき tabindex=\"0\"、false のとき tabindex=\"-1\"（roving tabindex）。",
        },
        ArgRow {
            name: "trigger(state)",
            kind: "OpenState",
            default: "",
            description: "この Menu 自身の開閉状態。aria-expanded/data-state へ反映される。",
        },
        ArgRow {
            name: "checkbox_item(checked) / radio_item(checked)",
            kind: "bool",
            default: "",
            description: "checked/unchecked を role=\"menuitemcheckbox\"/\"menuitemradio\" の aria-checked と data-state（checked_data_state 経由）へ反映する。",
        },
        ArgRow {
            name: "item_indicator(checked)",
            kind: "bool",
            default: "",
            description: "data-state（checked_data_state）へ反映し、unchecked のとき hidden 存在属性を付与する。aria-hidden=\"true\" を固定付与。",
        },
        ArgRow {
            name: "item_text(disabled, highlighted)",
            kind: "bool, bool",
            default: "",
            description: "親 item 系パーツの状態を data-disabled/data-highlighted として装飾用に複製する。",
        },
        ArgRow {
            name: "sub_trigger(sub_state)",
            kind: "OpenState",
            default: "",
            description: "このトリガーが開閉するサブメニュー側の開閉状態（親 Menubar の state ではない）。aria-expanded/data-state へ反映する。",
        },
    ],
    examples: &[
        ExampleEntry {
            title: "2 個の Menu を並べた構成",
            description: "File（open）/ Edit（closed）の 2 つの Menu を並べ、複数 Menu をまたぐ roving tabindex の構造を示します。",
            render: ex_menubar,
        },
        ExampleEntry {
            title: "checkbox-item / radio-item-group を含む構成",
            description: "checkbox-item（Word Wrap）と radio-item-group（Grid/List）を含む View メニューの構成例です。",
            render: ex_menubar_checkbox_and_radio,
        },
        ExampleEntry {
            title: "自前 CSS を当てる最小例",
            description: "data-scope=\"menubar\" セレクタで hover/highlighted/disabled/checked のスタイルを当てる最小 CSS 例です。",
            render: ex_menubar_custom_css,
        },
    ],
    keyboard: &[
        KeyRow {
            key: "ArrowRight / ArrowLeft（trigger）",
            description: "隣の trigger へフォーカスを移動する（open-follows-focus: ある Menu が開いていれば開く Menu も追随する）。垂直 menubar では ArrowDown/ArrowUp が同じ役割を担う（menubar.rs「開いている Menu を跨いだ左右移動」節）。",
        },
        KeyRow {
            key: "ArrowDown / Space / Enter（trigger、closed）",
            description: "Menu を開き、先頭の非 disabled 項目を data-highlighted で仮想フォーカスする（実 DOM フォーカスは trigger に留まる）。",
        },
        KeyRow {
            key: "ArrowUp（trigger、closed）",
            description: "Menu を開き、末尾の非 disabled 項目を仮想フォーカスする。",
        },
        KeyRow {
            key: "ArrowDown / ArrowUp（content 内）",
            description: "項目間で data-highlighted を移動する（disabled はスキップする、`fandhe-frontend-wasm-full` の `keynav::highlight_next_index` が `step_non_disabled` へ委譲する）。",
        },
        KeyRow {
            key: "ArrowRight / ArrowLeft（sub-trigger）",
            description: "ArrowRight でサブメニューを展開、ArrowLeft で親トリガーへ復帰する（水平配置時。垂直配置は軸が入れ替わる）。",
        },
        KeyRow {
            key: "Home / End",
            description: "content 内の先頭 / 末尾の非 disabled 項目へ仮想フォーカスを移動する（WAI-ARIA APG 準拠、Radix Menubar にはない拡張）。",
        },
        KeyRow {
            key: "印字可能文字（typeahead）",
            description: "item-text の子テキストをラベルとして前方一致検索し、一致する項目へ仮想フォーカスを移動する（WAI-ARIA APG 準拠）。",
        },
        KeyRow {
            key: "Escape",
            description: "開いている Menu を閉じる。trigger からフォーカスを離さない設計のため、フォーカス復帰は構造的に成立する。",
        },
        KeyRow {
            key: "Tab / Shift+Tab",
            description: "無配線（ブラウザ既定に委ねる）。roving tabindex により tabindex=\"0\" のトリガーのみが Tab 順序に含まれる。",
        },
    ],
    aria: &[
        AriaRow {
            attribute: "role=\"menubar\" / aria-orientation",
            description: "root に固定付与。orientation 引数の値を反映する。",
        },
        AriaRow {
            attribute: "role=\"none\"",
            description: "menu に固定付与（menubar の子として menuitem/group 以外を挟まないための WAI-ARIA APG 慣行）。",
        },
        AriaRow {
            attribute: "role=\"menuitem\" / aria-haspopup=\"menu\" / aria-expanded",
            description: "trigger/sub-trigger に付与。開閉状態（この Menu の state、もしくはサブメニューの sub_state）を反映する。",
        },
        AriaRow {
            attribute: "role=\"menu\"",
            description: "content/sub-content に固定付与。aria-labelledby は labelledby が Some のときのみ出力される。",
        },
        AriaRow {
            attribute: "role=\"menuitemcheckbox\" / role=\"menuitemradio\" / aria-checked",
            description: "checkbox-item/radio-item に付与。checked/unchecked の 2 値のみ（indeterminate は扱わない）。",
        },
        AriaRow {
            attribute: "aria-hidden=\"true\"",
            description: "item-indicator に固定付与（装飾アイコンであり、親 checkbox-item/radio-item 自身の aria-checked が checked 状態を既に伝達するため）。",
        },
    ],
    demo: None,
};

// ---------------------------------------------------------------------
// nav-list（`crates/headless-ui/src/nav_list.rs`）
// ---------------------------------------------------------------------

/// Demo（`nav_list_section`）は heading + 2 リンクの単一グループ構成。ここ
/// では heading を省いた最小構成（見出しなしのフラットなリンクリスト）を
/// 実演する。
fn ex_nav_list() -> Node {
    example_wrap(vec![nav_list::root(
        "API Reference",
        vec![],
        vec![nav_list::list(
            vec![],
            vec![
                nav_list::item(
                    vec![],
                    vec![nav_list::link(
                        "https://example.com/api/core/",
                        false,
                        vec![],
                        vec![text("core")],
                    )],
                ),
                nav_list::item(
                    vec![],
                    vec![nav_list::link(
                        "https://example.com/api/server/",
                        false,
                        vec![],
                        vec![text("server")],
                    )],
                ),
            ],
        )],
    )])
}

/// `/primitives/nav-list/`。
///
/// 一次情報: `crates/headless-ui/src/nav_list.rs:1-67`（モジュール doc
/// 「role を一切付与しない」`:16-30` 節）、`:81-121`（`root`/`heading`/
/// `list`/`item`/`link` シグネチャ）、`:64-67`（`navigation_menu` との
/// 使い分け節）、`:136`/`:143`/`:148`（role 非出力の実出力テスト）。
pub(super) const NAV_LIST: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "文書ナビ向け Link リスト。root（nav）/ heading（h2）/ list（ul）/ item（li）/ link（a）の 5 anatomy パーツを持つ（nav_list.rs モジュール doc）。",
        "5 パーツいずれも role 属性を一切付与しない。素の nav/h2/ul/li/a の暗黙 ARIA ロールをそのまま使うことが、menu ロールへの誤読を避ける本部品の存在理由そのものである（module doc「role を一切付与しない」節）。",
        "root の aria-label（label 引数）は必須引数であり、複数 nav ランドマークが存在する文書でスクリーンリーダー利用者がランドマーク間を区別できるようにする。",
        "navigation_menu との使い分けの軸は role の有無ではなくディスクロージャの有無（本部品は状態機械を持たない静的なリンク集、navigation_menu は Trigger/Content で開閉するパネル）。",
    ],
    arguments: &[
        ArgRow {
            name: "root(label)",
            kind: "&str",
            default: "",
            description: "root（nav）へ付与する aria-label（必須引数）。",
        },
        ArgRow {
            name: "link(current)",
            kind: "bool",
            default: "",
            description: "true のとき aria-current=\"page\"+data-current を付与する（role は付与しない）。",
        },
    ],
    examples: &[ExampleEntry {
        title: "heading を省いたフラットなリンクリスト",
        description: "heading パーツを使わず、見出しなしのリンクリストのみで構成した例です。",
        render: ex_nav_list,
    }],
    keyboard: &[],
    aria: &[
        AriaRow {
            attribute: "aria-label",
            description: "root（nav）に必須付与。role は一切付与しない（module doc「role を一切付与しない」節）。",
        },
        AriaRow {
            attribute: "aria-current=\"page\"",
            description: "link に付与（current が true のときのみ）。role は付与しない。",
        },
    ],
    demo: None,
};

// ---------------------------------------------------------------------
// navigation-menu（`crates/headless-ui/src/navigation_menu.rs`）
// ---------------------------------------------------------------------

/// Demo（`navigation_menu_section`）は 1 項目 open + 1 項目 closed。ここでは
/// plan 方針どおり全項目 closed 版を実演する。
fn ex_navigation_menu() -> Node {
    let closed = OpenState::Closed;
    example_wrap(vec![navigation_menu::root(
        "Resources",
        vec![],
        vec![navigation_menu::list(
            vec![],
            vec![navigation_menu::item(
                closed,
                false,
                vec![],
                vec![navigation_menu::trigger(
                    closed,
                    false,
                    "guides",
                    None,
                    None,
                    vec![],
                    vec![text("Guides")],
                )],
            )],
        )],
    )])
}

/// `/primitives/navigation-menu/`。
///
/// 一次情報: `crates/headless-ui/src/navigation_menu.rs:1-93`（モジュール
/// doc「`data-motion`・viewport 測定を実装しない」`:35-91` 節）、
/// `:111-232`（`root`/`item`/`trigger`/`content`/`link` シグネチャ）、
/// `:16`/`:28`（role を明示付与しない・`nav_list` との使い分け節）、
/// `:391-483`（`aria-expanded`/`aria-controls`/`aria-labelledby`/
/// `aria-current` の実出力テスト）。
pub(super) const NAVIGATION_MENU: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "トリガー起点で開閉するナビゲーションパネル。Root / List / Item / Trigger / Content / Link の 6 anatomy パーツを持つ（navigation_menu.rs モジュール doc）。",
        "高々 1 個の Trigger だけが開く状態機械（SingleSelect を埋め込んだ NavigationMenu、dispatch は \"select\"/\"toggle\"/\"deselect\"）。",
        "role は一切付与しない。root は素の nav の暗黙 ARIA role（navigation）に依拠し、role=\"menu\"/role=\"menuitem\" は付与しない（nav_list と同じ判断、文書ナビを操作メニューと誤伝達しないための設計）。",
        "data-motion（アニメーション方向の露出）・viewport 寸法測定は実装しない（intentional-non-adoption.md §3.25 規則 2 により headless 層へ持ち込まない設計判断、module doc「data-motion・viewport 測定を実装しない」節）。",
    ],
    arguments: &[
        ArgRow {
            name: "root(label)",
            kind: "&str",
            default: "",
            description: "root（nav）へ付与する aria-label（必須引数）。",
        },
        ArgRow {
            name: "trigger(state)",
            kind: "OpenState",
            default: "",
            description: "項目の開閉状態。aria-expanded/data-state へ反映される。",
        },
        ArgRow {
            name: "link(current)",
            kind: "bool",
            default: "false",
            description: "true のとき aria-current=\"page\"+data-current を出力する。",
        },
    ],
    examples: &[ExampleEntry {
        title: "全項目 closed の初期表示",
        description: "SSR の初期描画時点を想定し、すべての Trigger/Content を closed 状態にした構成です。",
        render: ex_navigation_menu,
    }],
    keyboard: &[],
    aria: &[
        AriaRow {
            attribute: "aria-expanded / aria-controls",
            description: "trigger に付与。開閉状態と content との関連付け（controls が Some のときのみ）を表す。role は付与しない。",
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

// ---------------------------------------------------------------------
// pagination（`crates/headless-ui/src/pagination.rs`）
// ---------------------------------------------------------------------

/// Demo（`pagination_section`）は Button モード中心（一部 Link）。ここでは
/// plan 方針どおり `ItemMode::Link` のみで統一した SSR/SEO 向け構成を実演
/// する。
fn ex_pagination() -> Node {
    example_wrap(vec![pagination::root(
        "Search results pagination",
        vec![],
        vec![
            pagination::prev_trigger(
                ItemMode::Link {
                    href: "https://example.com/search?page=1",
                },
                true,
                vec![],
                vec![text("Prev")],
            ),
            pagination::item(
                ItemMode::Link {
                    href: "https://example.com/search?page=1",
                },
                true,
                false,
                vec![],
                vec![text("1")],
            ),
            pagination::item(
                ItemMode::Link {
                    href: "https://example.com/search?page=2",
                },
                false,
                false,
                vec![],
                vec![text("2")],
            ),
            pagination::next_trigger(
                ItemMode::Link {
                    href: "https://example.com/search?page=2",
                },
                false,
                vec![],
                vec![text("Next")],
            ),
        ],
    )])
}

/// `/primitives/pagination/`。
///
/// 一次情報: `crates/headless-ui/src/pagination.rs:1-69`（モジュール doc）、
/// `:124-193`（`page_range`）、`:209-326`（`root`/`item`/`ellipsis`/
/// `prev_trigger`/`next_trigger` シグネチャ）、`:216`/`:261`/`:272`
/// （`aria-current="page"`/`data-selected`/`aria-hidden`/`aria-disabled`
/// の実出力）、`:327-340`/`:341-546`（`PaginationAction`/`Pagination` の
/// doc）。
pub(super) const PAGINATION: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "ページ送り。Root / Item / Ellipsis / PrevTrigger / NextTrigger の 5 anatomy パーツを持つ（pagination.rs モジュール doc）。",
        "page_range は総件数・ページサイズ・現在ページ・sibling/boundary 件数から省略記号を含むページ列を O(boundary_count + sibling_count) で決定的に導出する純粋関数（total_pages を全列挙しないため巨大 count 入力でも有界、DoS 対策）。",
        "Button（クリックで dispatch する SPA 向け）/ Link（href 起点の SSR・SEO 向け）の 2 つの ItemMode に両対応する。",
        "data-state を持たない（連続的なページ位置を扱うため）。現在ページは item の aria-current=\"page\"/data-selected、端到達は prev_trigger/next_trigger の disabled/data-disabled で表現する。",
    ],
    arguments: &[
        ArgRow {
            name: "root(aria_label)",
            kind: "&str",
            default: "",
            description: "root（nav）へ付与する aria-label。既定値相当は呼び出し側が明示指定する契約。",
        },
        ArgRow {
            name: "item(mode)",
            kind: "ItemMode",
            default: "",
            description: "Button（button type=\"button\"）/ Link（a href）のいずれで描画するかを選ぶ。",
        },
        ArgRow {
            name: "item(current)",
            kind: "bool",
            default: "",
            description: "true のとき aria-current=\"page\"+data-selected を付与する。",
        },
        ArgRow {
            name: "item(disabled)",
            kind: "bool",
            default: "",
            description: "Button モードのみネイティブ disabled を出力。両モード共通で aria-disabled/data-disabled を出力する。",
        },
    ],
    examples: &[ExampleEntry {
        title: "ItemMode::Link のみで構成した SSR/SEO 向けページ送り",
        description: "全パーツを ItemMode::Link に統一し、href 遷移によるページ送りを実演します。",
        render: ex_pagination,
    }],
    keyboard: &[],
    aria: &[
        AriaRow {
            attribute: "aria-current=\"page\" / data-selected",
            description: "item に付与（current が true のときのみ）。",
        },
        AriaRow {
            attribute: "aria-hidden=\"true\"",
            description: "ellipsis に固定付与。",
        },
        AriaRow {
            attribute: "aria-disabled",
            description: "item/prev-trigger/next-trigger に付与（disabled が true のときのみ）。Button モードのみ併せてネイティブ disabled を出力する。",
        },
    ],
    demo: None,
};

// ---------------------------------------------------------------------
// tabs（`crates/headless-ui/src/tabs.rs`）
// ---------------------------------------------------------------------

/// Demo（`tabs_section`）は Horizontal + indicator あり。ここでは plan
/// 方針どおり Vertical orientation・indicator なしの構成を実演する。
fn ex_tabs() -> Node {
    let props = TabsProps {
        id: "primitives-tabs-example",
        selected: "profile",
        orientation: Orientation::Vertical,
        activation_mode: ActivationMode::Manual,
        loop_focus: true,
        indicator: false,
    };
    let items = vec![
        TabItem {
            value: "profile",
            trigger: vec![text("Profile")],
            content: vec![text("Profile panel content.")],
            disabled: false,
        },
        TabItem {
            value: "billing",
            trigger: vec![text("Billing")],
            content: vec![text("Billing panel content.")],
            disabled: true,
        },
    ];
    example_wrap(vec![tabs(&props, items)])
}

/// `/primitives/tabs/`。
///
/// 一次情報: `crates/headless-ui/src/tabs.rs:1-60`（モジュール doc）、
/// `:137-297`（`TabsProps`/`tabs`/`tabs_with_root_attrs` シグネチャ）、
/// `:178-180`（`role="tablist"`/`"tab"`/`"tabpanel"` の実出力）、`:154-165`
/// （roving tabindex）、`:95-121`（`ActivationMode` の doc）。
pub(super) const TABS: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "WAI-ARIA APG の Tabs パターン（role=\"tablist\"/\"tab\"/\"tabpanel\"、aria-selected、相互参照する aria-controls/aria-labelledby、roving tabindex）に準拠したマークアップを組み立てる（tabs.rs モジュール doc）。",
        "root / list / trigger / content の 4 パーツに加え、選択タブの位置を示す装飾パーツ indicator（opt-in、TabsProps::indicator）を持つ 5 パーツ構成。",
        "ActivationMode（Automatic/Manual）を data-activation-mode として出力し、wasm-full 側のキーボード操作時の活性化挙動分岐に使われる（Automatic が既定）。",
        "props.selected がどの value とも一致しない場合、または一致した項目が disabled の場合は全 trigger/content が inactive として決定的に描画される（panic しない）。",
    ],
    arguments: &[
        ArgRow {
            name: "props(orientation)",
            kind: "Orientation",
            default: "",
            description: "data-orientation（root/list/trigger/content 共通）・list の aria-orientation の双方に反映する。",
        },
        ArgRow {
            name: "props(activation_mode)",
            kind: "ActivationMode",
            default: "Automatic",
            description: "list へ data-activation-mode として出力する（フォーカス移動と同時に活性化 or Enter/Space で活性化）。",
        },
        ArgRow {
            name: "props(loop_focus)",
            kind: "bool",
            default: "",
            description: "list へ data-loop-focus として出力する（Arrow キーで端から反対端へ循環するか）。",
        },
        ArgRow {
            name: "TabItem(disabled)",
            kind: "bool",
            default: "",
            description: "true のとき disabled 属性・data-disabled・aria-disabled=\"true\" を trigger に付与し、roving tabindex のフォールバック候補からも除外する。",
        },
    ],
    examples: &[ExampleEntry {
        title: "Vertical orientation・Manual activation・indicator なし",
        description: "Demo とは異なる軸（縦向き・手動活性化・indicator 省略）の組み合わせを実演します。",
        render: ex_tabs,
    }],
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

// ---------------------------------------------------------------------
// toolbar（`crates/headless-ui/src/toolbar.rs`）
// ---------------------------------------------------------------------

/// Demo（`toolbar_section`）は Horizontal。ここでは plan 方針どおり
/// Vertical orientation 版を実演する。
fn ex_toolbar() -> Node {
    let orientation = Orientation::Vertical;
    example_wrap(vec![toolbar::root(
        orientation,
        "View options",
        vec![],
        vec![
            toolbar::button(true, false, vec![], vec![text("List view")]),
            toolbar::button(false, false, vec![], vec![text("Grid view")]),
            toolbar::separator(orientation, vec![], vec![]),
            toolbar::toggle_item(
                false,
                false,
                true,
                "compact",
                vec![],
                vec![text("Compact (disabled)")],
            ),
        ],
    )])
}

/// `/primitives/toolbar/`。
///
/// 一次情報: `crates/headless-ui/src/toolbar.rs:1-91`（モジュール doc
/// 「呼び出し文脈」`:12` 節）、`:135-284`（`root`/`button`/`link`/
/// `separator`/`toggle_group`/`toggle_item` シグネチャ）、`:130`/`:220`/
/// `:239`/`:258`（`role="toolbar"`/`role="separator"`/`role="group"`/
/// `aria-pressed` の実出力）、`:285-319`/`:320-524`（`ToolbarAction`/
/// `Toolbar` の doc）。
pub(super) const TOOLBAR: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "ボタン・リンク・セパレータ・ToggleGroup を横方向（または縦方向）にグループ化する操作バー。Root / Button / Link / Separator / ToggleGroup / ToggleItem の 6 anatomy パーツを持つ（toolbar.rs モジュール doc）。",
        "roving tabindex（focused/item_count/loop_focus/orientation の複合状態機械 Toolbar）。フォーカス対象の項目のみ tabindex=\"0\"、それ以外は tabindex=\"-1\" になる。",
        "disabled 項目もフォーカス順序から除外しない（WAI-ARIA APG の toolbar パターン推奨に従う意図的な設計。aria-disabled/data-disabled で操作不能のみを表す）。",
        "link パーツは既存の link::root へ完全委譲し、external 時の target=\"_blank\"/rel=\"noopener noreferrer\" の不可分付与ロジックを再導出しない。",
        "separator は toolbar 自身の向きと直交する aria-orientation を出力する（横向き toolbar のセパレータは縦線になる）。",
    ],
    arguments: &[
        ArgRow {
            name: "root(orientation)",
            kind: "Orientation",
            default: "Orientation::Horizontal",
            description: "role=\"toolbar\" に付与する向き。aria-orientation/data-orientation の両方へ反映される。",
        },
        ArgRow {
            name: "root(label)",
            kind: "&str",
            default: "",
            description: "root に付与する aria-label（空文字列のときは省略）。",
        },
        ArgRow {
            name: "button(focused)",
            kind: "bool",
            default: "",
            description: "true のとき tabindex=\"0\"、false のとき tabindex=\"-1\"（roving tabindex）。button/link/toggle_item 共通の引数。",
        },
        ArgRow {
            name: "toggle_item(pressed)",
            kind: "bool",
            default: "",
            description: "aria-pressed/data-state（pressed_data_state 経由）へ反映される押下状態。",
        },
    ],
    examples: &[ExampleEntry {
        title: "Vertical orientation の構成",
        description: "縦方向の Toolbar で Separator の aria-orientation が水平（直交）に切り替わる例です。",
        render: ex_toolbar,
    }],
    keyboard: &[
        KeyRow {
            key: "Tab / Shift+Tab",
            description: "ネイティブ button/a 要素の暗黙フォーカス移動（roving tabindex により tabindex=\"0\" の項目のみが Tab 順序に含まれる）。",
        },
        KeyRow {
            key: "(矢印キー等)",
            description: "項目間移動のキーイベント配線は headless-ui の対象外。ToolbarAction（Next/Prev/First/Last/Focus）として状態遷移 API を提供し、実 DOM のキー配線は wasm ランタイム層の責務とする（toolbar.rs モジュール doc「呼び出し文脈」節）。",
        },
    ],
    aria: &[
        AriaRow {
            attribute: "role=\"toolbar\" / aria-orientation",
            description: "root に固定付与。orientation 引数の値を反映する。",
        },
        AriaRow {
            attribute: "role=\"separator\" / aria-orientation",
            description: "separator に固定付与。toolbar 自身の向きと直交する値になる。",
        },
        AriaRow {
            attribute: "role=\"group\"",
            description: "toggle-group に固定付与（aria-orientation は role=\"group\" に許可されないため付与しない）。",
        },
        AriaRow {
            attribute: "aria-pressed",
            description: "toggle-item に付与。押下状態（true/false）を表す。",
        },
    ],
    demo: None,
};

/// Navigation カテゴリ（11 部品）の `path -> ComponentPageSpec` テーブル。
/// 並び順は `crate::primitives_catalog::PRIMITIVES` の Navigation カテゴリの
/// 並びに合わせる。
pub const SPECS: &[(&str, ComponentPageSpec)] = &[
    ("/primitives/action-bar/", ACTION_BAR),
    ("/primitives/breadcrumb/", BREADCRUMB),
    ("/primitives/link/", LINK),
    ("/primitives/link-overlay/", LINK_OVERLAY),
    ("/primitives/menu/", MENU),
    ("/primitives/menubar/", MENUBAR),
    ("/primitives/nav-list/", NAV_LIST),
    ("/primitives/navigation-menu/", NAVIGATION_MENU),
    ("/primitives/pagination/", PAGINATION),
    ("/primitives/tabs/", TABS),
    ("/primitives/toolbar/", TOOLBAR),
];
