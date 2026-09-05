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
//! - `menu`/`menubar` の `trigger-item`/`checkbox-item`/`radio-item-group`/
//!   `radio-item`/`context-trigger`（menu）・`sub-trigger`/`sub-content`
//!   （menubar）は `tests/primitive_showcase.rs::KNOWN_UNCOVERED` に
//!   Demo 側の未網羅パートとして登録済みであり、本モジュールの Anatomy
//!   節（機械導出）には現れない。Features 節の散文では言及する（ソース上
//!   実在するため）が、Examples では新規に描画しない（Demo との切り口分離
//!   の趣旨と衝突しないよう、Anatomy に現れないパーツを Examples だけで
//!   実演することは避ける）。

use fandhe_frontend_core::{code, el, pre, text, Node};
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
        "close-trigger は呼び出し側 attrs に aria-label が無ければ既定値 \"close\"（zag.js popover の translations.closeTrigger 既定値、CLOSE_TRIGGER_ARIA_LABEL）を出力する（close_trigger 関数）。",
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
            description: "aria-label 未指定時は既定値 \"close\" を出力する（CLOSE_TRIGGER_ARIA_LABEL）。呼び出し側が指定すれば上書きされる。",
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
            description: "close-trigger の既定値。呼び出し側 attrs に aria-label があれば上書きされる。",
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

/// Demo（`breadcrumb_section`）は 3 項目のみで ellipsis を使わない。ここでは
/// `ellipsis` パーツ（`KNOWN_UNCOVERED` 登録済みの折り畳み表現）を実演する。
fn ex_breadcrumb() -> Node {
    example_wrap(vec![breadcrumb::root(
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
                breadcrumb::ellipsis(vec![]),
                breadcrumb::separator(vec![], vec![text("/")]),
                breadcrumb::item(
                    vec![],
                    vec![breadcrumb::current_link(vec![], vec![text("Current page")])],
                ),
            ],
        )],
    )])
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
    ],
    examples: &[ExampleEntry {
        title: "ellipsis を含む折り畳み表現",
        description: "中間項目を ellipsis パーツで省略し、先頭・末尾のみを表示する構成です。",
        render: ex_breadcrumb,
    }],
    keyboard: &[],
    aria: &[
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

/// Demo（`link_section`）は `external=true`（target/rel 付与あり）。ここでは
/// `external=false` の版（target/rel を出力しない、同一タブ内遷移）を実演
/// する。href は他ページとの内部リンク切れ検証
/// （`crate::linkcheck::check_links`）を避けるため、Demo と同じく
/// `example.com`（RFC 2606 予約ドメイン）を使う。
fn ex_link() -> Node {
    example_wrap(vec![link::root(
        "https://example.com/docs/guide",
        false,
        true,
        vec![],
        vec![text("Guide")],
    )])
}

/// `/primitives/link/`。
///
/// 一次情報: `crates/headless-ui/src/link.rs:1-54`（モジュール doc）、
/// `:72-100`（`root` シグネチャ）、`:66-67`（`target="_blank"`+
/// `rel="noopener noreferrer"` の不可分付与）、`:69`（`aria-current="page"`
/// の実出力）。
pub(super) const LINK: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "汎用インラインリンク。root（a）1 パーツのみの最小構成（chakra-ui Link 相当、link.rs モジュール doc）。",
        "external=true のとき target=\"_blank\"+rel=\"noopener noreferrer\" を不可分に付与する（reverse tabnabbing 対策。片方のみを付与できる API は公開しない）。",
        "current=true のとき aria-current=\"page\"+data-current を付与する（breadcrumb/nav_list と同じ語彙を共有）。",
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
    examples: &[ExampleEntry {
        title: "external=false の構成",
        description: "external を false にして target=\"_blank\"/rel を出力しない構成です。",
        render: ex_link,
    }],
    keyboard: &[],
    aria: &[AriaRow {
        attribute: "aria-current=\"page\"",
        description: "current 引数が true のときのみ付与する。",
    }],
    demo: None,
};

// ---------------------------------------------------------------------
// link-overlay（`crates/headless-ui/src/link_overlay.rs`）
// ---------------------------------------------------------------------

/// Demo（`link_overlay_section`）は overlay の子ノードに直接テキストを渡す
/// 単一リンク構成。ここでは module doc が推奨する「overlay へアクセシブル
/// ネームのみを与え、見出しは root の他の子ノードとして別途描画する」構成を
/// 実演する。
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

/// `/primitives/link-overlay/`。
///
/// 一次情報: `crates/headless-ui/src/link_overlay.rs:1-49`（モジュール doc
/// 「全面拡張の実装方針」`:12-30`・「呼び出し文脈」`:31-48` 節）、`:60-75`
/// （`root`/`overlay` シグネチャ）。
pub(super) const LINK_OVERLAY: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "カード全面クリック化。root（div、位置決めコンテキスト）/ overlay（a、カード全面へ拡張されるリンク）の 2 パーツ構成（chakra-ui LinkBox/LinkOverlay パターン相当、link_overlay.rs モジュール doc）。",
        "::before 疑似要素を使わず、overlay 自身を position: absolute; inset: 0; で root 全面へ展開する方式を採る（styled 層の CSS 責務、headless 層は CSS を持たない）。",
        "overlay がフローから外れるため、root の高さは overlay 以外の子ノード（見出し・説明文等の通常フロー要素）が確立する契約である（module doc「全面拡張の実装方針」）。",
        "単一リンクのみの overlay へ可視テキストを渡す代わりに、見出しを root の他の子ノードとして描画し overlay へは aria-label 等でアクセシブルネームのみを与える運用が推奨される。",
    ],
    arguments: &[ArgRow {
        name: "overlay(href)",
        kind: "&str",
        default: "",
        description: "遷移先 URL。危険な URL スキームは core の render() が属性ごと拒否する。",
    }],
    examples: &[ExampleEntry {
        title: "見出しを別描画し overlay へ aria-label のみ与える構成",
        description: "module doc が推奨する運用（可視見出しは root の子として描画し、overlay へはアクセシブルネームのみを渡す）を実演します。",
        render: ex_link_overlay,
    }],
    keyboard: &[],
    aria: &[AriaRow {
        attribute: "aria-label",
        description: "overlay 自身は aria-label を自動付与しない。単一リンクのみで完結する構成では呼び出し側が attrs 経由でアクセシブルネームを与える運用が推奨される（module doc 参照）。",
    }],
    demo: None,
};

// ---------------------------------------------------------------------
// menu（`crates/headless-ui/src/menu.rs`）
// ---------------------------------------------------------------------

/// Demo（`menu_section`）は ItemGroup + Separator の構成。ここでは
/// ItemGroup を使わず単純な Item 列のみの最小構成を実演する。
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

/// `/primitives/menu/`。
///
/// 一次情報: `crates/headless-ui/src/menu.rs:1-84`（モジュール doc）、
/// `:104-471`（`root`/`trigger`/`content`/`item`/`item_group`/`separator`/
/// `trigger_item`/`checkbox_item`/`radio_item` シグネチャ）、`:114`/`:185`/
/// `:226`/`:255`/`:369`（`aria-haspopup="menu"`/`role="menu"`/
/// `role="menuitem"`/`role="group"`/`aria-checked` の実出力）。
pub(super) const MENU: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "トリガー起点のオーバーレイ + アクション項目リスト。Root / Trigger / Indicator / Positioner / Content / Arrow / ArrowTip / Item / ItemGroup / ItemGroupLabel / Separator / TriggerItem / ContextTrigger / CheckboxItem / RadioItemGroup / RadioItem の 16 anatomy パーツを持つ（menu.rs モジュール doc）。",
        "サブメニューは親 Menu インスタンスの content 内に子 Menu インスタンス由来の TriggerItem/Positioner/Content を入れ子で配置して表現し、親子双方に aria-haspopup=\"menu\" を付与する（「haspopup 連鎖」でネストを支援技術へ伝える）。",
        "CheckboxItem/RadioItemGroup は Menu の開閉状態とは独立した checked 状態機械（MenuCheckboxItem/MenuRadioItemGroup、Checkable/SingleSelect を埋め込む）を持つ。",
        "ContextTrigger は右クリック起点のトリガーであり、ARIA 属性を一切付与しない（右クリックは SSR/no-JS では成立せず、ARIA を付けると JS なしで実現できない操作性を誤って約束するため）。",
        "開閉は Disclosure を埋め込んだ状態機械 Menu が管理する（dispatch は \"open\"/\"close\"/\"toggle\"）。",
    ],
    arguments: &[
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
            name: "checkbox_item(checked)",
            kind: "bool",
            default: "",
            description: "role=\"menuitemcheckbox\" の aria-checked（true/false のみ、indeterminate 非対応）。",
        },
    ],
    examples: &[ExampleEntry {
        title: "ItemGroup を使わない単純な Item 列",
        description: "ItemGroup/Separator を省き、Item のみを並べた最小構成です。",
        render: ex_menu,
    }],
    keyboard: &[],
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

/// `/primitives/menubar/`。
///
/// 一次情報: `crates/headless-ui/src/menubar.rs:1-124`（モジュール doc）、
/// `:168-411`（`root`/`menu`/`trigger`/`content`/`sub_trigger`/
/// `sub_content` シグネチャ）、`:163`/`:187`/`:280`/`:254`
/// （`role="menubar"`/`role="none"`/`role="menuitem"`/`role="menu"` の
/// 実出力）、`:412-462`/`:463-762`（`MenubarAction`/`Menubar`、`:717` の
/// `decode_action` の doc）。
pub(super) const MENUBAR: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "複数 Menu を水平（または垂直）に並べるコンテナ。Root / Menu / Trigger / Positioner / Content / Item / ItemGroup / ItemGroupLabel / Separator / SubTrigger / SubContent の 11 anatomy パーツを持つ（menubar.rs モジュール doc）。",
        "roving tabindex（focused/trigger_count/open/loop_focus/orientation の複合状態機械 Menubar）。フォーカス対象のトリガーのみ tabindex=\"0\"、それ以外は tabindex=\"-1\" になる。",
        "開いている Menu を跨いだ左右移動: ある Menu が開いた状態で Next/Prev/First/Last/Focus を dispatch すると、フォーカス移動と同時に開く Menu も隣へ移る（menubar 特有の挙動、toolbar の roving tabindex にはない）。",
        "menu パーツは role=\"none\" を固定付与し、role=\"menubar\" の子として menuitem/group 以外の要素を挟まないようにする（WAI-ARIA APG の menubar パターン）。",
        "既存の menu モジュールの anatomy はそのまま再利用せず data-scope=\"menubar\" を独自に持つ。状態機械の値語彙（OpenState/aria/data-* ヘルパ）のみを再利用する。",
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
    ],
    examples: &[ExampleEntry {
        title: "2 個の Menu を並べた構成",
        description: "File（open）/ Edit（closed）の 2 つの Menu を並べ、複数 Menu をまたぐ roving tabindex の構造を示します。",
        render: ex_menubar,
    }],
    keyboard: &[
        KeyRow {
            key: "Tab / Shift+Tab",
            description: "ネイティブ button 要素の暗黙フォーカス移動（roving tabindex により tabindex=\"0\" のトリガーのみが Tab 順序に含まれる）。",
        },
        KeyRow {
            key: "(矢印キー等)",
            description: "ArrowRight/Left・Home/End・Enter/Space・Escape 等のキーイベント配線は headless-ui の対象外。MenubarAction（Next/Prev/First/Last/Focus/Open/Close/Toggle）として状態遷移 API を提供し、実 DOM のキー配線は wasm ランタイム層の責務とする（menubar.rs「スコープ外」節）。",
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
