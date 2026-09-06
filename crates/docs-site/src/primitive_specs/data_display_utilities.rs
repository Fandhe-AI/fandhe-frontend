//! Primitives（`fandhe-frontend-headless-ui`）Data Display / Utilities 系
//! 10 部品ページの原稿データ（イシュー #1029、親トラッキング #1035
//! Phase 5）。
//!
//! # 役割・呼び出し文脈
//!
//! [`crate::primitive_specs::SPEC_TABLES`] から参照される
//! [`crate::component_page::ComponentPageSpec`] 定数群を保持する専用
//! モジュール。本モジュール自体は生成物へ直接寄与しない
//! （`component_page::render_component_page` が `spec_for` 経由で読み取り、
//! Demo〔[`crate::primitive_showcase::data_display_utilities`]〕・Anatomy・
//! `data-*` 属性表（いずれも機械導出、Primitives 層は CSS 変数表を持たない）
//! と合成して 6 節ページを組み立てる）。
//!
//! 対象は avatar・carousel・json-tree-view・scroll-area・skip-nav・
//! splitter・steps・tour・tree-view・visually-hidden の 10 部品
//! （`crates/docs-site/src/primitives_catalog.rs` の
//! `PrimitiveCategory::DataDisplayUtilities` 並び順と一致させる）。
//!
//! # Arguments 抽出規約（`/themes/` 側 #946 規約・`primitive_specs::overlay_disclosure`
//! （#1027）からの層固有の変更点を踏襲）
//!
//! `fandhe-frontend-headless-ui` の当該モジュールが公開する関数・
//! （`Carousel`/`Splitter`/`Steps`/`Tour` のような状態機械については）
//! メソッドの**型付き引数**を抽出元とする。`attrs: Vec<(&str, &str)>`/
//! `children: Vec<Node>` は全部品共通の定型引数のため原則除外するが、
//! [`visually_hidden::root`] のように公開関数がこの 2 引数のみを持つ場合は
//! 例外的にそのまま記載する（それ以外に本モジュールの API 面が存在しない
//! ため、除外すると `arguments` が空になり受け入れ条件 1 に反する）。
//! [`ArgRow`] には part 列が無いため、`<関数/メソッド名>: <引数名>` 形式で
//! `name` 列へ埋め込む（10 部品で表記を統一する、#1027 と同型）。
//!
//! # `keyboard` を 7/10 件で空にする理由（3 件のみ非空）
//!
//! 本カテゴリ 10 モジュールのうち `tabindex` を出力するのは
//! `scroll_area`（`viewport`、`crates/headless-ui/src/scroll_area.rs:68-71`）・
//! `skip_nav`（`content`、`crates/headless-ui/src/skip_nav.rs:100-107`）・
//! `splitter`（`resize_trigger`）の 3 件のみ（非テストソースの grep
//! 結果）。残り 7 件（avatar/carousel/json_tree_view/steps/tour/tree_view/
//! visually_hidden）は `tabindex` を一切出力せず、クリック・矢印キー等の実
//! DOM 配線は各モジュール doc の out-of-scope 節が
//! `fandhe-frontend-wasm-full` 後続イシューの責務と明示している
//! （`carousel.rs:71`/`steps.rs:75`/`tour.rs:89-91`/`tree_view.rs:74-77`
//! 参照。avatar/visually_hidden はそもそもキー操作の対象になる要素を
//! 持たない）。実装が焦点制御に関与しない部品へ「ArrowRight で次へ進む」の
//! ような未実装の対話を書くと利用者へ誤った安心を与えるため、該当 7 件は
//! `keyboard: &[]` を採用し、非空である `aria` 表のみで Accessibility 節を
//! 成立させる（`component_page.rs` の Accessibility 節省略規則参照。
//! scroll_area/skip_nav の 2 件は `tabindex` の**属性事実のみ**を `KeyRow`
//! に記載し、対話そのものは記載しない）。`splitter` は上記 2 件と異なり
//! Arrow/Home/End キーの DOM 配線がイシュー #1074 で
//! `fandhe-frontend-wasm-full` に実装済みのため、`keyboard` へ実装済みの
//! 対話（Arrow/Home/End）と未実装の対話（Shift+Arrow、`SplitterAction::
//! IncrementLarge`/`DecrementLarge` の状態機械のみ）を明確に区別して記載
//! する（イシュー #1664 参照突合）。
//!
//! # `avatar`/`visually_hidden` の Accessibility 節が空にならない理由
//!
//! `avatar`（`crates/headless-ui/src/avatar.rs`）・
//! `visually_hidden`（`crates/headless-ui/src/visually_hidden.rs`）は
//! いずれも `role`/`aria-*` を一切出力しない（`avatar.rs`/
//! `visually_hidden.rs` 全文で `role`/`aria-` grep 0 件、非テスト行のみで
//! 確認）。`aria` を空のままにすると Accessibility 節ごと省略されてしまう
//! ため、非付与の事実そのものを `AriaRow { attribute: "(該当なし)", .. }`
//! として明示する（`crate::component_specs_nav_data::AVATAR`/`BADGE` と同型の
//! 先例）。
//!
//! # セキュリティ不変条件（REQ-1）
//!
//! 本モジュールはリテラル `&'static str` のみで [`ArgRow`]/[`AriaRow`]/
//! [`KeyRow`] を構築し、`raw_html()` や HTML 文字列の直接組み立て
//! （`format!("<td>{}</td>", …)`）を一切行わない。実際のエスケープは
//! `component_page.rs` 側の `table`/`td`/`text` ノード木経由で `render()`
//! が行う。`examples` のレンダラは `fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui`
//! （イシュー #693 方針、`hui` エイリアス）経由の headless-ui パート関数
//! のみで組み立て、`fandhe_frontend_pre_styled_ui::` の部品関数（styled
//! 層）は一切呼ばない（受け入れ条件 3）。ダミー文字列は無害なもの
//! （`example.com` 等の予約ドメイン、架空の名前）に限る。

use fandhe_frontend_core::{code, div, p, pre, text, Node};
use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui as hui;
use hui::avatar::{self, ImageStatus};
use hui::carousel;
use hui::data_attrs::Orientation;
use hui::fandhe_frontend_interactive::Component;
use hui::json_tree_view::{self, JsonValue};
use hui::positioning::{Align, Placement, Side};
use hui::scroll_area;
use hui::skip_nav;
use hui::splitter;
use hui::steps::Steps;
use hui::tour::{Tour, TourAction, TourStep};
use hui::tree_view;
use hui::visually_hidden;
use hui::OpenState;

use crate::component_page::{ArgRow, AriaRow, ComponentPageSpec, ExampleEntry, KeyRow};

/// Examples 用の枠組み（`primitive_specs/forms_a.rs::wrap_example` /
/// `primitive_specs/forms_c_date_status.rs::wrap_example` と同型。
/// [`crate::primitive_showcase`] のデモ本体と同じ `primitives-demo-frame`/
/// `primitives-demo-note` class のみを使い、`h2`/`h3` は出さない）。
fn wrap_example(note: &'static str, body: Vec<Node>) -> Node {
    div(
        vec![],
        vec![
            p(vec![("class", "primitives-demo-note")], vec![text(note)]),
            div(vec![("class", "primitives-demo-frame")], body),
        ],
    )
}

// ---------------------------------------------------------------------
// Avatar（/primitives/avatar/）
// ---------------------------------------------------------------------

/// 一次情報: `crates/headless-ui/src/avatar.rs:1-120`（モジュール doc、
/// `data-state` 語彙・ARIA について・スコープ外・参考サイトとの突合）、
/// `:193-248`（`root`/`image`/`fallback` シグネチャ）、`:262-282`
/// （`Avatar::new`）。非テスト行で `role`/`aria-` の出力は 0 件。
fn ex_avatar_error_fallback() -> Node {
    let status = ImageStatus::Error;
    avatar::root(
        vec![],
        vec![
            avatar::image(
                status,
                "https://example.com/broken-avatar.png",
                "Ada Lovelace",
                vec![],
            ),
            avatar::fallback(status, vec![], vec![text("AL")]),
        ],
    )
}

/// 自前 CSS の最小例（イシュー #1659、`CALENDAR_CUSTOM_CSS_SNIPPET`
/// 〔`primitive_specs/forms_c_date_status.rs`〕と同型のパターン）。CSS は
/// テキストノード（[`code`]/[`pre`]）として既定エスケープを経由し、
/// `crate::primitive_showcase` の専用スタイルシート（`[data-scope=`/
/// `[data-part=` を持たない契約、`tests/site_css_contract.rs`）へは
/// 追加しない。
const AVATAR_CUSTOM_CSS_SNIPPET: &str = "\
[data-scope=\"avatar\"][data-part=\"root\"] {\n  \
  display: inline-block;\n  width: 3rem;\n  height: 3rem;\n  border-radius: 50%;\n  overflow: hidden;\n\
}\n\
[data-scope=\"avatar\"][data-part=\"image\"] {\n  \
  width: 100%;\n  height: 100%;\n  object-fit: cover;\n\
}\n\
[data-scope=\"avatar\"][data-part=\"fallback\"] {\n  \
  display: flex;\n  align-items: center;\n  justify-content: center;\n  \
  width: 100%;\n  height: 100%;\n  background: #e5e7eb;\n\
}\n\
[data-scope=\"avatar\"][data-part=\"image\"][data-state=\"hidden\"],\n\
[data-scope=\"avatar\"][data-part=\"fallback\"][data-state=\"hidden\"] {\n  \
  display: none;\n\
}\n";

/// Radix Primitives の `delayMs`（フォールバック表示遅延）を非採用とした
/// 代わりに、利用者が自前 CSS でどう円形アバターを組み立てるかを示す例
/// （イシュー #1659 差分メモ参照）。`Loaded`（実在アセット）と `Error`
/// （イニシャルフォールバック）の 2 インスタンスを並べ、
/// `data-state="hidden"` の多層防御（属性セレクタ + `hidden` 存在属性）を
/// 実演する。
fn ex_avatar_custom_css() -> Node {
    let loaded = ImageStatus::Loaded;
    let loaded_avatar = avatar::root(
        vec![],
        vec![
            avatar::image(
                loaded,
                crate::showcase::IMAGE_DEMO_SRC,
                "Naledi Khumalo",
                vec![],
            ),
            avatar::fallback(loaded, vec![], vec![text("NK")]),
        ],
    );
    let error = ImageStatus::Error;
    let error_avatar = avatar::root(
        vec![],
        vec![
            avatar::image(
                error,
                "https://example.com/missing-avatar.png",
                "Priya Das",
                vec![],
            ),
            avatar::fallback(error, vec![], vec![text("PD")]),
        ],
    );
    wrap_example(
        "利用者が data-scope / data-part / data-state 属性セレクタで自前 CSS を当てる最小例です。headless-ui 自体はスタイルを持ちません。",
        vec![
            loaded_avatar,
            error_avatar,
            pre(
                vec![],
                vec![code(vec![], vec![text(AVATAR_CUSTOM_CSS_SNIPPET)])],
            ),
        ],
    )
}

pub const AVATAR: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "画像読み込みステータス 3 値（ImageStatus::Loading/Loaded/Error）を管理する状態機械 Avatar を提供する（avatar.rs:135-182）。",
        "Root / Image / Fallback の 3 anatomy パーツで構成し、Loaded のときのみ image を表示、それ以外は fallback を表示する安全側の既定（avatar.rs:173-182,193-248）。",
        "image パーツは alt テキストを必須引数として要求することが実質的なアクセシビリティ担保であり、専用の role/aria-* は付与しない（avatar.rs:37-43）。",
        "ark-ui/Zag.js・Radix Primitives・Radix Themes・chakra-ui の 4 参照サイトと突合済み（イシュー #1659）。anatomy/data-*/ARIA は一致し是正なし。Radix の delayMs（表示遅延）・各パーツへの dir/id・イニシャル自動導出は意図的に非採用（avatar.rs:81-120）。",
    ],
    arguments: &[
        ArgRow {
            name: "image: src",
            kind: "&str",
            default: "",
            description: "画像 URL（avatar.rs:211-229、必須）。",
        },
        ArgRow {
            name: "image: alt",
            kind: "&str",
            default: "",
            description: "代替テキスト（avatar.rs:211-229、必須。実質的なアクセシビリティ担保）。",
        },
        ArgRow {
            name: "image/fallback: status",
            kind: "ImageStatus",
            default: "ImageStatus::Loading",
            description: "画像読み込みステータス。Loaded のときのみ image が可視（avatar.rs:140-148,173-182）。",
        },
        ArgRow {
            name: "Avatar::new: initial",
            kind: "ImageStatus",
            default: "ImageStatus::Loading",
            description: "状態機械の初期ステータス（avatar.rs:262-282）。",
        },
    ],
    examples: &[
        ExampleEntry {
            title: "Error fallback",
            description: "画像読み込み失敗（ImageStatus::Error）時のイニシャル表示例です。",
            render: ex_avatar_error_fallback,
        },
        ExampleEntry {
            title: "自前 CSS の最小例",
            description: "data-scope/data-part/data-state 属性セレクタで円形アバターのスタイルを当てる例です。",
            render: ex_avatar_custom_css,
        },
    ],
    keyboard: &[],
    aria: &[AriaRow {
        attribute: "(該当なし)",
        description: "root/image/fallback は固有の role/aria-* を出力しない（avatar.rs 全文の非テスト行で role/aria- grep 0 件）。image パーツの alt テキストのみが代替情報を提供する。参照 4 サイト（ark-ui/Zag.js・Radix Primitives・Radix Themes・chakra-ui）とも role/aria-* を付与しない点で一致する（イシュー #1659 突合）。",
    }],
    demo: None,
};

// ---------------------------------------------------------------------
// Carousel（/primitives/carousel/）
// ---------------------------------------------------------------------

/// 一次情報: `crates/headless-ui/src/carousel.rs:1-74`（モジュール doc、
/// 決定的な遷移規則・スコープ外）、`:85-208`（`root`/`prev_trigger`/
/// `next_trigger`/`item_group`/`item`/`indicator` シグネチャと
/// role/aria-* 出力）、`:266`（`Carousel::new`）。
fn ex_carousel_vertical_loop() -> Node {
    let orientation = Orientation::Vertical;
    carousel::root(
        orientation,
        "Featured photos (vertical, looping)",
        vec![],
        vec![
            carousel::item_group(
                vec![],
                vec![
                    carousel::item(0, 3, false, vec![], vec![text("Slide 1")]),
                    carousel::item(1, 3, false, vec![], vec![text("Slide 2")]),
                    carousel::item(2, 3, true, vec![], vec![text("Slide 3")]),
                ],
            ),
            carousel::control(
                vec![],
                vec![
                    carousel::prev_trigger(false, "Previous slide", vec![], vec![text("‹")]),
                    carousel::indicator_group(
                        vec![],
                        vec![
                            carousel::indicator(0, false, vec![]),
                            carousel::indicator(1, false, vec![]),
                            carousel::indicator(2, true, vec![]),
                        ],
                    ),
                    carousel::next_trigger(false, "Next slide", vec![], vec![text("›")]),
                ],
            ),
        ],
    )
}

pub const CAROUSEL: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "Root / Control / PrevTrigger / NextTrigger / ItemGroup / Item / IndicatorGroup / Indicator の 8 anatomy パーツを提供する（carousel.rs:1-10）。",
        "root は role=\"region\" + aria-roledescription=\"carousel\" + 呼び出し側指定の aria-label を固定出力する（carousel.rs:85-106）。",
        "item は role=\"group\" + aria-roledescription=\"slide\" + \"{n} of {m}\" 形式の aria-label を自動生成する（carousel.rs:167-186）。",
        "autoplay 非対応の初期実装のため item_group の aria-live は常に \"polite\" 固定（carousel.rs:66-70,154-165）。",
        "index==slide_count-1 かつ loop=true のとき Next は先頭へ循環し、prev/next とも disabled にならない決定的な遷移規則を持つ（carousel.rs:31-42）。",
    ],
    arguments: &[
        ArgRow {
            name: "root: orientation",
            kind: "Orientation",
            default: "",
            description: "carousel のレイアウト方向（carousel.rs:92-106）。",
        },
        ArgRow {
            name: "root: label",
            kind: "&str",
            default: "",
            description: "aria-label に出力する説明文（carousel.rs:92-106、必須）。",
        },
        ArgRow {
            name: "prev_trigger/next_trigger: disabled",
            kind: "bool",
            default: "false",
            description: "端かつ loop=false のとき true（native disabled + data-disabled、carousel.rs:115-152）。",
        },
        ArgRow {
            name: "item: index, count, current",
            kind: "usize, usize, bool",
            default: "",
            description: "スライド位置・総数・現在表示中かどうか（aria-label/data-current 生成元、carousel.rs:173-186）。",
        },
        ArgRow {
            name: "Carousel::new: index, slide_count, loop_, orientation",
            kind: "usize, usize, bool, Orientation",
            default: "",
            description: "状態機械の初期値（carousel.rs:266）。",
        },
    ],
    examples: &[ExampleEntry {
        title: "Vertical looping carousel",
        description: "縦方向レイアウト・loop 有効で最終スライドを表示した例です（Demo は水平・非 loop）。",
        render: ex_carousel_vertical_loop,
    }],
    keyboard: &[],
    aria: &[
        AriaRow {
            attribute: "role=\"region\" / aria-roledescription=\"carousel\" / aria-label",
            description: "root に固定出力する（carousel.rs:85-106）。",
        },
        AriaRow {
            attribute: "role=\"group\" / aria-roledescription=\"slide\" / aria-label",
            description: "item に固定出力し、aria-label は \"{n} of {m}\" 形式（carousel.rs:167-186）。",
        },
        AriaRow {
            attribute: "aria-live=\"polite\"",
            description: "item_group に固定出力する（carousel.rs:154-165）。",
        },
        AriaRow {
            attribute: "aria-label",
            description: "prev_trigger/next_trigger/indicator に呼び出し側指定または自動生成の説明文を出力する（carousel.rs:115-208）。",
        },
        AriaRow {
            attribute: "aria-current=\"true\"",
            description: "indicator が current のときのみ出力する（carousel.rs:194-208）。",
        },
    ],
    demo: None,
};

// ---------------------------------------------------------------------
// JSON Tree View（/primitives/json-tree-view/）
// ---------------------------------------------------------------------

/// 一次情報: `crates/headless-ui/src/json_tree_view.rs:1-93`（モジュール
/// doc、`tree_view`（#753）の派生であることの位置づけ・データモデル・
/// out-of-scope）、`:194-223`（`key`/`value`/`render_json` シグネチャ）、
/// `:347`（`expanded_to_depth`）。role/aria-* は `tree_view` 側
/// （`branch`/`item`/`tree`/`branch_content`）が出力し、本モジュール固有の
/// `key`/`value` パーツは role/aria-* を持たない。
fn ex_json_tree_view_collapsed_array() -> Node {
    let value = JsonValue::Array(vec![
        JsonValue::Number(1.0),
        JsonValue::Number(2.0),
        JsonValue::String("three".to_string()),
    ]);
    let tree = json_tree_view::expanded_to_depth(&value, 0);
    json_tree_view::render_json(&tree, &value)
}

/// 自前 CSS の最小例（イシュー #1661、`AVATAR_CUSTOM_CSS_SNIPPET` と同型の
/// パターン）。CSS はテキストノード（[`code`]/[`pre`]）として既定
/// エスケープを経由し、`crate::primitive_showcase` の専用スタイルシート
/// （`[data-scope=`/`[data-part=` を持たない契約、`tests/site_css_contract.rs`）
/// へは追加しない。json-tree-view スコープ（`key`/`colon`/`value`）と
/// tree-view スコープ（構造部）の 2 スコープが併存する事実を正直に示す
/// （両方に触れなければ見た目が完成しないことを明示する）。
const JSON_TREE_VIEW_CUSTOM_CSS_SNIPPET: &str = "\
[data-scope=\"tree-view\"][data-part=\"branch-content\"][hidden] {\n  \
  display: none;\n\
}\n\
[data-scope=\"tree-view\"][data-part=\"branch-indent-guide\"] {\n  \
  width: 1rem;\n\
}\n\
[data-scope=\"json-tree-view\"][data-part=\"key\"] {\n  \
  color: #7c3aed;\n  font-family: monospace;\n\
}\n\
[data-scope=\"json-tree-view\"][data-part=\"colon\"] {\n  \
  color: #6b7280;\n\
}\n\
[data-scope=\"json-tree-view\"][data-part=\"value\"][data-kind=\"string\"] {\n  \
  color: #059669;\n  font-family: monospace;\n\
}\n\
[data-scope=\"json-tree-view\"][data-part=\"value\"][data-kind=\"number\"] {\n  \
  color: #2563eb;\n  font-family: monospace;\n\
}\n";

/// [`JSON_TREE_VIEW_CUSTOM_CSS_SNIPPET`] を実演する例（Collapsed array と
/// 同じ `depth=0` の配列を使い回す）。
fn ex_json_tree_view_custom_css() -> Node {
    let value = JsonValue::Array(vec![
        JsonValue::Number(1.0),
        JsonValue::String("two".to_string()),
    ]);
    let tree = json_tree_view::expanded_to_depth(&value, 1);
    wrap_example(
        "json-tree-view スコープ（key/colon/value）と tree-view スコープ（構造部）の両方に data-scope / data-part / data-kind 属性セレクタで自前 CSS を当てる最小例です。headless-ui 自体はスタイルを持ちません。",
        vec![
            json_tree_view::render_json(&tree, &value),
            pre(
                vec![],
                vec![code(
                    vec![],
                    vec![text(JSON_TREE_VIEW_CUSTOM_CSS_SNIPPET)],
                )],
            ),
        ],
    )
}

pub const JSON_TREE_VIEW: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "実装済み crate::tree_view（イシュー #753）の 12 anatomy パーツ・状態機械 TreeView を再利用し、JSON 固有の key/colon/value 3 パーツを branch-text/item-text の内側へ入れ子にして追加する（json_tree_view.rs:1-35、colon はイシュー #1661 で ark-ui/zag 突合により追加）。",
        "Object は HashMap ではなく挿入順を保持する Vec<(String, JsonValue)> で表現し、render_json の出力が決定的（バイト単位で一致）である（json_tree_view.rs:31-37,118-136）。",
        "ノード識別子に RFC 6901 JSON Pointer を用い、key に `/`/`~` を含むデータでも一意性が壊れない（json_tree_view.rs:39-46,180-192）。",
        "role/aria-*（role=\"tree\"/\"treeitem\"/\"group\"、aria-expanded/aria-selected/aria-level/aria-posinset/aria-setsize）はすべて crate::tree_view のパーツ関数から継承する（json_tree_view.rs:16-24）。",
    ],
    arguments: &[
        ArgRow {
            name: "render_json: tree",
            kind: "&TreeView",
            default: "",
            description: "展開・選択状態（json_tree_view.rs:221-223）。",
        },
        ArgRow {
            name: "render_json: root",
            kind: "&JsonValue",
            default: "",
            description: "描画対象の JSON 風データ木（json_tree_view.rs:221-223）。",
        },
        ArgRow {
            name: "expanded_to_depth: depth",
            kind: "usize",
            default: "",
            description: "この深さまでのブランチを展開済みにした TreeView を生成する（json_tree_view.rs:347）。",
        },
        ArgRow {
            name: "value: kind",
            kind: "&'static str",
            default: "",
            description: "JsonValue::kind() が返す固定語彙（\"null\"/\"boolean\"/\"number\"/\"string\"/\"array\"/\"object\"）のみを受け取る data-kind 属性値（イシュー #1661 で \"bool\" から \"boolean\" へ変更、json_tree_view.rs:138-153,201-209）。",
        },
    ],
    examples: &[
        ExampleEntry {
            title: "Collapsed array",
            description: "depth=0（全ブランチ折りたたみ）で描画した配列の例です（Demo は depth=2 まで展開済みのオブジェクト）。",
            render: ex_json_tree_view_collapsed_array,
        },
        ExampleEntry {
            title: "Custom CSS for key/colon/value",
            description: "本モジュールが型別配色を提供しない headless 部品のため、キー・区切り・値の 3 パーツへ自前 CSS を当てる最小例です（json-tree-view スコープのみを対象とし、構造部の tree-view スコープには触れません）。",
            render: ex_json_tree_view_custom_css,
        },
    ],
    // イシュー #1661: tree-view 構造部（branch/branch-control/item 等）は
    // json_tree_view でも data-scope="tree-view" のまま出力されるため、
    // `fandhe-frontend-wasm-full` の `keynav.rs::handle_tree_view_keydown`
    // （`TREE_VIEW_TREE_SELECTOR`/`TREE_VIEW_ITEM_SELECTOR` で
    // `[data-scope="tree-view"]` を探索）がそのまま適用される（継承元:
    // `crates/wasm-full/src/keynav.rs`、イシュー #1072）。
    keyboard: &[
        KeyRow {
            key: "ArrowDown / ArrowUp",
            description: "可視かつ disabled でない treeitem 間で 1 件ずつフォーカス移動する（折りたたみ中の子孫はスキップ、非循環）。",
        },
        KeyRow {
            key: "ArrowRight",
            description: "閉じたブランチは展開する。開いたブランチは最初の子へフォーカス移動する。葉ノードでは no-op。",
        },
        KeyRow {
            key: "ArrowLeft",
            description: "開いたブランチは折りたたむ。それ以外（葉ノード・閉じたブランチ）は親ブランチへフォーカス移動する（ルート直下では no-op）。",
        },
        KeyRow {
            key: "Home / End",
            description: "可視かつ disabled でない最初/最後の treeitem へフォーカス移動する。",
        },
        KeyRow {
            key: "Enter / Space",
            description: "葉ノードは選択（select）、ブランチは開閉（toggle）を発火する。",
        },
        KeyRow {
            key: "印字可能文字",
            description: "typeahead: 直近の入力から一致する treeitem へフォーカス移動する。",
        },
        KeyRow {
            key: "Escape",
            description: "typeahead バッファをリセットするのみ（TreeView は常時展開のツリーであり閉じる操作を持たない）。",
        },
    ],
    aria: &[
        AriaRow {
            attribute: "role=\"tree\" / role=\"treeitem\" / role=\"group\"",
            description: "crate::tree_view の tree/branch・item/branch_content から継承する（tree_view.rs:125-259）。",
        },
        AriaRow {
            attribute: "aria-expanded / aria-selected / aria-level / aria-posinset / aria-setsize",
            description: "crate::tree_view の branch/item から継承する（tree_view.rs:159-303）。",
        },
    ],
    demo: None,
};

// ---------------------------------------------------------------------
// Scroll Area（/primitives/scroll-area/）
// ---------------------------------------------------------------------

/// 一次情報: `crates/headless-ui/src/scroll_area.rs:1-90`（モジュール doc、
/// スコープ外・参考サイトとの突合〔イシュー #1662〕）、`:95-183`
/// （予約キー定数・`drop_reserved`・`root`/`viewport`/`content`/
/// `scrollbar`/`thumb`/`corner` シグネチャと `tabindex`/`aria-hidden` 出力）。
fn ex_scroll_area_horizontal() -> Node {
    scroll_area::root(
        vec![],
        vec![
            scroll_area::viewport(
                vec![],
                vec![scroll_area::content(
                    vec![],
                    vec![text("Wide scrollable content…")],
                )],
            ),
            scroll_area::scrollbar(
                Orientation::Horizontal,
                vec![],
                vec![scroll_area::thumb(Orientation::Horizontal, vec![], vec![])],
            ),
            scroll_area::corner(vec![], vec![]),
        ],
    )
}

/// 自前 CSS の最小例（イシュー #1662、`AVATAR_CUSTOM_CSS_SNIPPET` と同型の
/// パターン）。CSS はテキストノード（[`code`]/[`pre`]）として既定エスケープ
/// を経由し、`crate::primitive_showcase` の専用スタイルシート（`[data-scope=`/
/// `[data-part=` を持たない契約、`tests/site_css_contract.rs`）へは追加
/// しない。
///
/// `scrollbar`/`thumb`/`corner` パーツは JS によるスクロール位置追従・
/// drag 操作を実装していない静的マークアップ（`crates/headless-ui/
/// src/scroll_area.rs` モジュール doc「スコープ外」節）であり、これらを
/// 表示したままネイティブスクロールバーを隠すと、スクロール位置表示・
/// マウス操作の両方を失う（codex-review P1 指摘、イシュー #1662）。
/// `crates/pre-styled-ui/src/scroll_area.rs` の実装契約と同じく、
/// `scrollbar`/`thumb`/`corner` は非表示のまま維持し、ネイティブ
/// スクロールバー自体を標準プロパティ（`scrollbar-width`/
/// `scrollbar-color`）+ `::-webkit-scrollbar` 系疑似要素で装飾する
/// 最小構成を示す。
const SCROLL_AREA_CUSTOM_CSS_SNIPPET: &str = "\
[data-scope=\"scroll-area\"][data-part=\"root\"] {\n  \
  position: relative;\n\
}\n\
[data-scope=\"scroll-area\"][data-part=\"viewport\"] {\n  \
  overflow: auto;\n  height: 8rem;\n  \
  scrollbar-width: thin;\n  scrollbar-color: #9ca3af transparent;\n\
}\n\
[data-scope=\"scroll-area\"][data-part=\"viewport\"]::-webkit-scrollbar {\n  \
  width: 0.5rem;\n  height: 0.5rem;\n\
}\n\
[data-scope=\"scroll-area\"][data-part=\"viewport\"]::-webkit-scrollbar-track {\n  \
  background: transparent;\n\
}\n\
[data-scope=\"scroll-area\"][data-part=\"viewport\"]::-webkit-scrollbar-thumb {\n  \
  background: #9ca3af;\n  border-radius: 9999px;\n\
}\n\
[data-scope=\"scroll-area\"][data-part=\"scrollbar\"],\n\
[data-scope=\"scroll-area\"][data-part=\"thumb\"],\n\
[data-scope=\"scroll-area\"][data-part=\"corner\"] {\n  \
  display: none;\n\
}\n";

fn ex_scroll_area_custom_css() -> Node {
    let items: Vec<Node> = (1..=6)
        .map(|n| fandhe_frontend_core::li(vec![], vec![text(format!("Item {n}"))]))
        .collect();
    let demo = scroll_area::root(
        vec![],
        vec![
            scroll_area::viewport(
                vec![],
                vec![scroll_area::content(
                    vec![],
                    vec![fandhe_frontend_core::ul(vec![], items)],
                )],
            ),
            scroll_area::scrollbar(
                Orientation::Vertical,
                vec![],
                vec![scroll_area::thumb(Orientation::Vertical, vec![], vec![])],
            ),
            scroll_area::corner(vec![], vec![]),
        ],
    );
    wrap_example(
        "利用者が data-scope/data-part 属性セレクタでネイティブスクロールバーを装飾する最小例です（scrollbar/thumb/corner パーツは静的マークアップのため非表示のまま維持します）。headless-ui 自体はスタイルを持ちません。",
        vec![
            demo,
            pre(
                vec![],
                vec![code(vec![], vec![text(SCROLL_AREA_CUSTOM_CSS_SNIPPET)])],
            ),
        ],
    )
}

pub const SCROLL_AREA: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "Root / Viewport / Content / Scrollbar / Thumb / Corner の 6 anatomy パーツを提供し、CSS overflow を主体とするスクロール領域を表現する（scroll_area.rs:1-10）。ark-ui/Zag.js の anatomy（6 パーツ）と完全一致（イシュー #1662 突合）。",
        "viewport は WAI 慣行（矢印キー/Page キーでフォーカス済み要素をスクロールできる）に従い tabindex=\"0\" を固定出力する（scroll_area.rs:139-146）。SSR では overflow の有無を判定できないため常時付与する安全側の設計（WCAG 2.1.1）。",
        "scrollbar/corner はネイティブスクロールバーと意味が重複する装飾要素のため aria-hidden=\"true\" を固定出力する（scroll_area.rs:157-183）。",
        "呼び出し側 attrs による固定属性（tabindex/aria-hidden/data-orientation）のなりすまし・重複出力を drop_reserved で除去する（scroll_area.rs:116-127、イシュー #1662）。",
        "ark-ui/Zag.js・Radix Primitives・chakra-ui・Radix Themes の 4 参照サイトと突合済み（イシュー #1662）。anatomy は増減なし。参照側の data-overflow-*/data-at-*/data-hover/data-scrolling/data-dragging・Radix の data-state（いずれも DOM 計測・ポインタ操作由来）は SSR で真の値を決定できないため意図的に非採用（`docs/policy/intentional-non-adoption.md` §3.25 規則 2）。Zag.js の role=\"presentation\" も、viewport が tabindex=\"0\" でフォーカス可能なため WAI-ARIA 1.2 §5.4 により無視される値として非採用。",
    ],
    arguments: &[
        ArgRow {
            name: "scrollbar/thumb: orientation",
            kind: "Orientation",
            default: "",
            description: "data-orientation 属性値（scroll_area.rs:157-176）。値語彙は vertical/horizontal の 2 値のみ。",
        },
        ArgRow {
            name: "viewport: attrs, children",
            kind: "Vec<(&str, &str)>, Vec<Node>",
            default: "vec![], vec![]",
            description: "tabindex=\"0\" は固定出力のため呼び出し側から指定する引数ではない（drop_reserved が除去、scroll_area.rs:139-146）。読み上げ名が必要な場合は aria-label/aria-labelledby を attrs へ付与することを推奨する。",
        },
    ],
    examples: &[
        ExampleEntry {
            title: "Horizontal scroll area",
            description: "水平方向のスクロールバー配置例です（Demo は垂直・水平の両軸 + corner）。",
            render: ex_scroll_area_horizontal,
        },
        ExampleEntry {
            title: "自前 CSS の最小例",
            description: "data-scope/data-part/data-orientation 属性セレクタでスクロールバー表現を当てる例です。",
            render: ex_scroll_area_custom_css,
        },
    ],
    keyboard: &[
        KeyRow {
            key: "Tab",
            description: "viewport は tabindex=\"0\" を固定出力するため通常の Tab 順序に含まれる（scroll_area.rs:139-146）。",
        },
        KeyRow {
            key: "↑ / ↓ / ← / →",
            description: "フォーカス済み viewport 上でのネイティブスクロール（本モジュールは独自のキーイベントリスナを持たず、ブラウザ既定動作に委ねる。Radix Primitives docs も同じ方針を明記、イシュー #1662 突合）。",
        },
        KeyRow {
            key: "PageUp / PageDown / Home / End",
            description: "ネイティブスクロールのページ単位・端への移動（ブラウザ既定動作、独自リスナなし）。",
        },
        KeyRow {
            key: "Space / Shift+Space",
            description: "ネイティブスクロールの前方/後方ページ送り（ブラウザ既定動作、独自リスナなし）。",
        },
    ],
    aria: &[
        AriaRow {
            attribute: "aria-hidden=\"true\"",
            description: "scrollbar/corner に固定出力する（scroll_area.rs:157-183）。ネイティブスクロールバーとの意味重複を明示する目的（両参照サイトにはない本実装独自の付与、イシュー #1662）。",
        },
        AriaRow {
            attribute: "(該当なし)",
            description: "root/viewport/content は role/aria-* を出力しない。Zag.js の role=\"presentation\" は、viewport が tabindex=\"0\" でフォーカス可能なため WAI-ARIA 1.2 §5.4 により UA に無視される値であり、Radix Primitives（role 非付与）に整合する形で追加していない（イシュー #1662 突合）。",
        },
    ],
    demo: None,
};

// ---------------------------------------------------------------------
// Skip Nav（/primitives/skip-nav/）
// ---------------------------------------------------------------------

/// 一次情報: `crates/headless-ui/src/skip_nav.rs:1-63`（モジュール doc、
/// href の構成・契約属性の除去）、`:75-108`（`DEFAULT_ID`/`link`/`content`
/// シグネチャ）。role/aria-* の出力は非テスト行で 0 件。
fn ex_skip_nav_custom_id() -> Node {
    div(
        vec![],
        vec![
            skip_nav::link(
                "primitives-main",
                vec![],
                vec![text("Skip to primitives content")],
            ),
            skip_nav::content(
                "primitives-main",
                vec![],
                vec![text("Primitives content starts here.")],
            ),
        ],
    )
}

/// 自前 CSS の最小例（イシュー #1663、`AVATAR_CUSTOM_CSS_SNIPPET` と同型の
/// パターン）。CSS はテキストノード（[`code`]/[`pre`]）として既定エスケープ
/// を経由し、`crate::primitive_showcase` の専用スタイルシート（`[data-scope=`/
/// `[data-part=` を持たない契約、`tests/site_css_contract.rs`）へは追加
/// しない。headless-ui 自体はスタイルを持たないため、chakra-ui の
/// `SkipNavContent` inline `style={{ outline: 0 }}`（headless 層では
/// `docs/policy/intentional-non-adoption.md` §3.25 規則 2 により非採用、
/// イシュー #1663 突合結果）に相当する調整も含め、利用者側 CSS で示す。
const SKIP_NAV_CUSTOM_CSS_SNIPPET: &str = "\
[data-scope=\"skip-nav\"][data-part=\"link\"] {\n  \
  position: absolute;\n  width: 1px;\n  height: 1px;\n  margin: -1px;\n  \
  overflow: hidden;\n  clip: rect(0 0 0 0);\n  border: 0;\n\
}\n\
[data-scope=\"skip-nav\"][data-part=\"link\"]:focus-visible {\n  \
  clip: auto;\n  width: auto;\n  height: auto;\n  position: fixed;\n  \
  top: 1rem;\n  left: 1rem;\n  padding: 0.5rem 0.75rem;\n  \
  background: #fff;\n  outline: 2px solid #2563eb;\n\
}\n\
[data-scope=\"skip-nav\"][data-part=\"content\"] {\n  \
  outline: none;\n\
}\n";

/// [`SKIP_NAV_CUSTOM_CSS_SNIPPET`] を実演する例（`AVATAR_CUSTOM_CSS_SNIPPET`
/// の実演関数と同型）。id はページ骨格の `DEFAULT_ID`（`"fandhe-skip-nav"`）
/// や他の例の `"primitives-main"` と衝突しない第 3 の値
/// （`"primitives-custom-css-target"`）を使い、`href` の解決先が曖昧に
/// ならないようにする。
fn ex_skip_nav_custom_css() -> Node {
    div(
        vec![],
        vec![
            skip_nav::link(
                "primitives-custom-css-target",
                vec![],
                vec![text("Skip to content (custom CSS)")],
            ),
            skip_nav::content(
                "primitives-custom-css-target",
                vec![],
                vec![text("Content reachable via the styled skip link.")],
            ),
            pre(
                vec![],
                vec![code(vec![], vec![text(SKIP_NAV_CUSTOM_CSS_SNIPPET)])],
            ),
        ],
    )
}

pub const SKIP_NAV: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "link / content の 2 anatomy パーツで WCAG 2.1 SC 2.4.1 Bypass Blocks を実現する（skip_nav.rs:1-14）。",
        "link は任意の URL を呼び出し側から受け取らず、常に `#<id>`（フラグメントのみ）を内部で組み立てるためスキーム注入経路を持たない（skip_nav.rs:16-22,77-91）。",
        "content は id/tabindex を、link は href を、それぞれ呼び出し側 attrs に同名キー（大文字小文字を無視）があっても fail-closed に除去してから合成する（skip_nav.rs:24-32,82-108）。",
        "DEFAULT_ID 定数（\"fandhe-skip-nav\"）を提供し、ページ全体に 1 個だけ配置する典型利用を想定する（skip_nav.rs:71-75）。",
        "chakra-ui（唯一の参照軸、Ark UI は該当ページ 404・Radix Primitives / Radix Themes に該当部品なし）と突合済み（イシュー #1663）。anatomy / data-* / ARIA は増減なし。chakra-ui の SkipNavContent が出力する inline outline: 0 は装飾のため headless 層では非採用（`docs/policy/intentional-non-adoption.md` §3.25 規則 2、Themes 版が CSS で担当）。",
    ],
    arguments: &[
        ArgRow {
            name: "link: id",
            kind: "&str",
            default: "skip_nav::DEFAULT_ID",
            description: "スキップ先 id。href=\"#<id>\" として出力する（skip_nav.rs:81-91）。",
        },
        ArgRow {
            name: "content: id",
            kind: "&str",
            default: "skip_nav::DEFAULT_ID",
            description: "id 属性値。link の href と対にする（skip_nav.rs:100-108）。",
        },
    ],
    examples: &[
        ExampleEntry {
            title: "Custom id",
            description: "DEFAULT_ID ではなく呼び出し側指定の id（\"primitives-main\"）を使う例です。",
            render: ex_skip_nav_custom_id,
        },
        ExampleEntry {
            title: "Custom CSS",
            description: "利用者が data-scope/data-part 属性セレクタで見た目を当てる最小例です。headless-ui 自体はスタイルを持ちません。",
            render: ex_skip_nav_custom_css,
        },
    ],
    keyboard: &[
        KeyRow {
            key: "Tab (link)",
            description: "独自のキーリスナを持たず、link はページ先頭に置かれた通常の <a> としてブラウザの既定 Tab 順序に従いフォーカス可能になる（chakra-ui docs の「できるだけ DOM 先頭に置く」前提と一致、イシュー #1663 突合結果）。",
        },
        KeyRow {
            key: "Enter (link, focused)",
            description: "ネイティブのフラグメント遷移で href=\"#<id>\" へジャンプする。独自のキーイベントリスナは持たずブラウザ既定動作に委ねる（skip_nav.rs:16-22、イシュー #1663 突合結果）。",
        },
        KeyRow {
            key: "Tab (content, after link activation)",
            description: "content は tabindex=\"-1\" を固定出力するため通常の Tab 順序には含まれず、link クリック後のプログラム的フォーカス移動のみを許可する（skip_nav.rs:93-108）。",
        },
    ],
    aria: &[AriaRow {
        attribute: "(該当なし)",
        description: "link/content は固有の role/aria-* を出力しない（skip_nav.rs 全文の非テスト行で role/aria- grep 0 件）。chakra-ui も role/aria-* を付与せず整合する（イシュー #1663 突合結果）。",
    }],
    demo: None,
};

// ---------------------------------------------------------------------
// Splitter（/primitives/splitter/）
// ---------------------------------------------------------------------

/// 一次情報: `crates/headless-ui/src/splitter.rs:1-76`（モジュール doc、
/// `aria-orientation` の向き・決定的な正規化・スコープ外）、`:210-287`
/// （`root`/`panel`/`resize_trigger`/`resize_trigger_indicator`
/// シグネチャと role/aria-*/tabindex 出力）、`:352`（`Splitter::new`）。
fn ex_splitter_vertical_three_panels() -> Node {
    let orientation = Orientation::Vertical;
    splitter::root(
        orientation,
        false,
        vec![],
        vec![
            splitter::panel("sp-top", 0, orientation, vec![], vec![text("Top")]),
            splitter::resize_trigger(
                orientation,
                "0",
                "100",
                "33",
                "sp-top",
                "sp-middle",
                false,
                vec![],
                vec![splitter::resize_trigger_indicator(vec![], vec![])],
            ),
            splitter::panel("sp-middle", 1, orientation, vec![], vec![text("Middle")]),
            splitter::resize_trigger(
                orientation,
                "0",
                "100",
                "66",
                "sp-middle",
                "sp-bottom",
                false,
                vec![],
                vec![splitter::resize_trigger_indicator(vec![], vec![])],
            ),
            splitter::panel("sp-bottom", 2, orientation, vec![], vec![text("Bottom")]),
        ],
    )
}

pub const SPLITTER: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "Root / Panel / ResizeTrigger / ResizeTriggerIndicator の 4 anatomy パーツと、パネルサイズ状態機械 Splitter を提供する（splitter.rs:1-11）。",
        "resize_trigger は role=\"separator\" + aria-valuemin/aria-valuemax/aria-valuenow + aria-orientation（パネルレイアウトと逆向き）+ aria-controls（隣接 2 パネルの id）+ data-id（\"<leading>:<trailing>\"）を常に出力する。",
        "panel は data-index（パネル序数）・data-id（id の写し）を出力する（イシュー #1664 で ark-ui docs の data-* 表と突合し追加）。",
        "disabled=true のとき tabindex=\"-1\" + aria-disabled、false のとき tabindex=\"0\" を出力する。",
        "SplitterAction::IncrementLarge/DecrementLarge（zag.js keyboardResizeBy 既定値 ×10 相当）を状態機械として提供する（イシュー #1664、DOM 配線は未実装）。",
        "呼び出し側 attrs からの role/aria-*/tabindex/data-*/id のなりすまし・重複出力を drop_reserved で除去する（イシュー #1664）。",
        "Splitter::new はパネル数 2 未満・非有限値・制約矛盾等の実現不能構成を既定（2 パネル 50/50）へ fail-closed にフォールバックする（splitter.rs:33-45,194-208）。",
    ],
    arguments: &[
        ArgRow {
            name: "root: orientation, disabled",
            kind: "Orientation, bool",
            default: "",
            description: "パネルレイアウトの向きと無効状態。",
        },
        ArgRow {
            name: "panel: id, index",
            kind: "&str, usize",
            default: "",
            description: "resize_trigger の aria-controls 先となる id（必須）と data-index として出力するパネル序数（イシュー #1664 で index を追加、破壊的変更）。",
        },
        ArgRow {
            name: "resize_trigger: min, max, now, leading_id, trailing_id, disabled",
            kind: "&str, &str, &str, &str, &str, bool",
            default: "",
            description: "aria-valuemin/aria-valuemax/aria-valuenow と、隣接 2 パネルの id（aria-controls/data-id へ出力）・tabindex 切替の元（イシュー #1664 で controls 単一引数を leading_id/trailing_id へ置換、破壊的変更）。",
        },
        ArgRow {
            name: "Splitter::new: panels, orientation",
            kind: "&[PanelSpec], Orientation",
            default: "",
            description: "パネル構成（size/min/max）と向き。fail-closed に正規化する（splitter.rs:352）。",
        },
    ],
    examples: &[ExampleEntry {
        title: "Vertical 3-panel splitter",
        description: "縦方向レイアウトで 3 パネル・2 セパレータを組んだ例です（Demo は水平 2 パネル）。",
        render: ex_splitter_vertical_three_panels,
    }],
    keyboard: &[
        KeyRow {
            key: "Tab",
            description: "resize_trigger は disabled=false のとき tabindex=\"0\" で通常の Tab 順序に入り、disabled=true のとき tabindex=\"-1\" で除外される。",
        },
        KeyRow {
            key: "Arrow（軸別）",
            description: "SplitterAction::Increment/Decrement（ステップ 1%）として状態遷移する。イシュー #1074 で fandhe-frontend-wasm-full の splitter モジュールが DOM keydown 配線済み。",
        },
        KeyRow {
            key: "Home / End",
            description: "SplitterAction::SetToMin/SetToMax として状態遷移する。DOM 配線済み（イシュー #1074）。",
        },
        KeyRow {
            key: "Shift+Arrow",
            description: "SplitterAction::IncrementLarge/DecrementLarge（ステップ 10%）として状態機械のみ提供する。fandhe-frontend-wasm-full の DOM 配線は未実装（イシュー #1664 時点、別 Issue 起票対象）。",
        },
    ],
    aria: &[
        AriaRow {
            attribute: "role=\"separator\"",
            description: "resize_trigger に固定出力する。",
        },
        AriaRow {
            attribute: "aria-valuemin / aria-valuemax / aria-valuenow",
            description: "先行パネルのサイズ%（有限性検証・クランプ済み）を出力する。",
        },
        AriaRow {
            attribute: "aria-orientation",
            description: "セパレータ自体の向き（パネルレイアウトと逆向き、WAI-ARIA APG 準拠。zag.js の非反転出力とは非同値、イシュー #1664 参照突合）。",
        },
        AriaRow {
            attribute: "aria-controls",
            description: "隣接 2 パネルの id（\"<leading> <trailing>\"、イシュー #1664 で先行パネルのみから拡張）。",
        },
        AriaRow {
            attribute: "aria-disabled=\"true\"",
            description: "disabled=true のときのみ出力する。",
        },
    ],
    demo: None,
};

// ---------------------------------------------------------------------
// Steps（/primitives/steps/）
// ---------------------------------------------------------------------

/// 一次情報: `crates/headless-ui/src/steps.rs:1-76`（モジュール doc、
/// 状態モデル・out-of-scope）、`:153-388`（`Steps::new`/`root`/`list`/
/// `item`/`trigger`/`indicator`/`separator`/`content`/`completed_content`/
/// `prev_trigger`/`next_trigger` シグネチャと role/aria-* 出力）。
fn ex_steps_completed() -> Node {
    let steps = Steps::new(2, 2, Orientation::Horizontal);
    steps.root(
        vec![],
        vec![
            steps.list(
                vec![],
                vec![
                    steps.item(
                        0,
                        vec![],
                        vec![steps.trigger(
                            0,
                            vec![],
                            vec![steps.indicator(0, vec![], vec![text("1")])],
                        )],
                    ),
                    steps.separator(0, vec![], vec![]),
                    steps.item(
                        1,
                        vec![],
                        vec![steps.trigger(
                            1,
                            vec![],
                            vec![steps.indicator(1, vec![], vec![text("2")])],
                        )],
                    ),
                ],
            ),
            steps.completed_content(vec![], vec![text("All steps completed.")]),
        ],
    )
}

/// 一次情報: `crates/headless-ui/src/steps.rs`（`Steps::trigger`/
/// `Steps::content`/`Steps::completed_content` の `data-orientation` 加算、
/// イシュー #1665 参照突合）。
fn ex_steps_vertical() -> Node {
    let steps = Steps::new(3, 1, Orientation::Vertical);
    steps.root(
        vec![],
        vec![
            steps.list(
                vec![],
                vec![
                    steps.item(
                        0,
                        vec![],
                        vec![steps.trigger(
                            0,
                            vec![],
                            vec![steps.indicator(0, vec![], vec![text("1")])],
                        )],
                    ),
                    steps.separator(0, vec![], vec![]),
                    steps.item(
                        1,
                        vec![],
                        vec![steps.trigger(
                            1,
                            vec![],
                            vec![steps.indicator(1, vec![], vec![text("2")])],
                        )],
                    ),
                    steps.separator(1, vec![], vec![]),
                    steps.item(
                        2,
                        vec![],
                        vec![steps.trigger(
                            2,
                            vec![],
                            vec![steps.indicator(2, vec![], vec![text("3")])],
                        )],
                    ),
                ],
            ),
            steps.progress(vec![], vec![]),
            steps.content(1, vec![], vec![text("Step 2 content")]),
        ],
    )
}

pub const STEPS: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "Root / List / Item / Trigger / Indicator / Separator / Content / CompletedContent / PrevTrigger / NextTrigger / Progress の 11 anatomy パーツを提供する（steps.rs:1-11。Progress はイシュー #1665 の参照突合で新設）。",
        "count/step から complete/current/incomplete の 3 状態を導出し、data-state・data-complete/-current/-incomplete へ一元反映する（steps.rs:24-34,89-121）。",
        "current な item の trigger のみ aria-current=\"step\" を付与する（steps.rs:58-61,249-269）。",
        "trigger/content/completed-content は data-orientation を出力する（イシュー #1665 参照突合で加算。呼び出し側 CSS が単独パートのみでレイアウト条件化できるようにする）。",
        "separator は role=\"separator\" + aria-hidden=\"true\" で装飾要素として a11y ツリーから除外する（steps.rs:288-310）。",
        "prev_trigger/next_trigger は境界（step==0/step==count）で disabled + data-disabled 属性を付与する（イシュー #1665 で data-disabled を加算。本リポジトリの disabled 語彙統一）。",
        "progress は role=\"progressbar\" + aria-valuemin/aria-valuemax/aria-valuenow/aria-valuetext を出力し、percent==100（全 step 完了）のときのみ data-complete を付与する（イシュー #1665 新設）。",
        "呼び出し側 attrs からの role/aria-*/data-*/type/hidden のなりすまし・重複出力を drop_reserved で除去する（イシュー #1665、toolbar/splitter と同型）。",
    ],
    arguments: &[
        ArgRow {
            name: "Steps::new: count, step, orientation",
            kind: "usize, usize, Orientation",
            default: "",
            description: "全 step 数・現在位置・向き。fail-closed に正規化する（steps.rs:123-133,164-171）。",
        },
        ArgRow {
            name: "item/trigger/indicator/separator/content: index",
            kind: "usize",
            default: "",
            description: "0..count の step インデックス（3 状態・aria-current 判定の元、steps.rs:224-334）。",
        },
        ArgRow {
            name: "progress: attrs, children",
            kind: "Vec<(&str, &str)>, Vec<Node>",
            default: "",
            description: "percent（step * 100 / count）を aria-valuenow/aria-valuetext へ出力する progressbar パーツ（イシュー #1665 新設）。",
        },
    ],
    examples: &[
        ExampleEntry {
            title: "All steps completed",
            description: "count==step（全 step 完了）で completed_content が表示される例です（Demo は 2 番目の step が current）。",
            render: ex_steps_completed,
        },
        ExampleEntry {
            title: "Vertical orientation",
            description: "Orientation::Vertical で trigger/content/progress の data-orientation=\"vertical\" が反映される例です。",
            render: ex_steps_vertical,
        },
    ],
    keyboard: &[
        KeyRow {
            key: "Tab / Shift+Tab",
            description: "trigger/prev-trigger/next-trigger はネイティブ button のためフォーカス順序に入る（disabled 境界では native disabled によりフォーカス順序から除外される）。",
        },
        KeyRow {
            key: "Enter / Space",
            description: "ネイティブ button の既定挙動（click イベント発火）のみ機能する。`fandhe-frontend-wasm-full` の headless::MAPPING_TABLE に \"steps\" scope は登録されておらず、trigger/prev-trigger/next-trigger の click から dispatch（\"goto\"/\"prev\"/\"next\"）への実配線は未実装（イシュー #1665 時点、別 Issue 起票対象）。",
        },
    ],
    aria: &[
        AriaRow {
            attribute: "aria-current=\"step\"",
            description: "current な item の trigger のみに出力する（steps.rs:249-269）。",
        },
        AriaRow {
            attribute: "role=\"separator\" / aria-hidden=\"true\"",
            description: "separator に固定出力する（装飾要素、steps.rs:288-310）。",
        },
        AriaRow {
            attribute: "data-disabled",
            description: "prev-trigger/next-trigger の境界（step==0/step==count）で native disabled と併せて出力する（イシュー #1665 加算）。",
        },
        AriaRow {
            attribute: "role=\"progressbar\" / aria-valuemin / aria-valuemax / aria-valuenow / aria-valuetext",
            description: "progress パーツに固定出力する（イシュー #1665 新設）。",
        },
    ],
    demo: None,
};

// ---------------------------------------------------------------------
// Tour（/primitives/tour/）
// ---------------------------------------------------------------------

/// 一次情報: `crates/headless-ui/src/tour.rs:1-94`（モジュール doc、
/// スコープ・状態モデル・out-of-scope）、`:220-458`（`Tour::new`/`content`/
/// `title`/`description`/`progress_text` シグネチャと role/aria-* 出力）。
fn ex_tour_second_step() -> Node {
    let mut tour = Tour::new(vec![
        TourStep {
            id: "step-1".to_string(),
            target: Some("#docs-toc-heading".to_string()),
            title: "Page navigation".to_string(),
            description: "Use this menu to jump between sections.".to_string(),
            placement: Placement::new(Side::Bottom, Align::Center),
        },
        TourStep {
            id: "step-2".to_string(),
            target: Some("#docs-search".to_string()),
            title: "Full-text search".to_string(),
            description: "Use this box to search the whole site.".to_string(),
            placement: Placement::new(Side::Top, Align::Start),
        },
    ]);
    tour.update(TourAction::Start);
    tour.update(TourAction::Next);
    tour.root(
        vec![],
        vec![
            tour.backdrop(vec![], vec![]),
            tour.spotlight(vec![], vec![]),
            tour.positioner(
                vec![],
                vec![
                    tour.arrow(vec![], vec![tour.arrow_tip(vec![], vec![])]),
                    tour.content(
                        hui::tour::ContentIds {
                            id: Some("tour-content-2"),
                            labelledby: Some("tour-title-2"),
                            describedby: Some("tour-desc-2"),
                        },
                        vec![],
                        vec![
                            tour.title(
                                Some("tour-title-2"),
                                vec![],
                                vec![text("Full-text search")],
                            ),
                            tour.description(
                                Some("tour-desc-2"),
                                vec![],
                                vec![text("Use this box to search the whole site.")],
                            ),
                            tour.progress_text(vec![], vec![text("Step 2 of 2")]),
                            tour.action_trigger(vec![], vec![text("Finish")]),
                            tour.close_trigger(vec![], vec![text("×")]),
                        ],
                    ),
                ],
            ),
        ],
    )
}

pub const TOUR: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "Root / Backdrop / Spotlight / Positioner / Arrow / ArrowTip / Content / Title / Description / ProgressText / CloseTrigger / ActionTrigger の 12 anatomy パーツを提供する（tour.rs:1-11）。",
        "Idle/Active{step}/Skipped/Completed の決定的な状態機械を持ち、終端状態からのいずれのアクションも no-op（一度終了したツアーは暗黙に再開しない、tour.rs:34-52）。",
        "content は role=\"dialog\" を固定出力し、ContentIds が Some のときのみ aria-labelledby/aria-describedby を出力する（tour.rs:371-395）。",
        "progress_text は aria-live=\"polite\" を固定出力し、ステップ遷移を支援技術へ読み上げさせる（tour.rs:431-439）。",
        "対象要素の実座標追従・スポットライト実測値注入・target セレクタの実解決は fandhe-frontend-wasm-full の後続イシューのスコープ（本モジュールは data-target 出力と静的な data-side/data-align のみ、tour.rs:13-21,87-94）。",
    ],
    arguments: &[
        ArgRow {
            name: "Tour::new: steps",
            kind: "Vec<TourStep>",
            default: "",
            description: "ツアーの全ステップ（初期状態は常に Idle、tour.rs:238-243）。",
        },
        ArgRow {
            name: "content: ids",
            kind: "ContentIds",
            default: "ContentIds::default()",
            description: "id/aria-labelledby/aria-describedby の関連付け先（tour.rs:195-205,375-395）。",
        },
        ArgRow {
            name: "title/description: id",
            kind: "Option<&str>",
            default: "None",
            description: "Some のとき content の labelledby/describedby と対にする（tour.rs:397-429）。",
        },
    ],
    examples: &[ExampleEntry {
        title: "Second step (2-step tour)",
        description: "2 ステップ構成で 2 番目の step まで進めた例です（Demo は 1 ステップのみ）。",
        render: ex_tour_second_step,
    }],
    keyboard: &[],
    aria: &[
        AriaRow {
            attribute: "role=\"dialog\"",
            description: "content に固定出力する（tour.rs:371-395）。",
        },
        AriaRow {
            attribute: "aria-labelledby / aria-describedby",
            description: "ContentIds が Some のときのみ content に出力する（tour.rs:383-391）。",
        },
        AriaRow {
            attribute: "aria-live=\"polite\"",
            description: "progress_text に固定出力する（tour.rs:431-439）。",
        },
    ],
    demo: None,
};

// ---------------------------------------------------------------------
// Tree View（/primitives/tree-view/）
// ---------------------------------------------------------------------

/// 一次情報: `crates/headless-ui/src/tree_view.rs:1-89`（モジュール doc、
/// out-of-scope）、`:112-324`（`root`/`label`/`tree`/`branch`/
/// `branch_control`/`branch_indicator`/`branch_text`/`branch_content`/
/// `branch_indent_guide`/`item`/`item_text`/`item_indicator` シグネチャと
/// role/aria-* 出力）、`:486`（`TreeView::render_nodes`）。
fn ex_tree_view_closed_branch() -> Node {
    let closed = OpenState::Closed;
    tree_view::root(
        vec![],
        vec![
            tree_view::label(vec![], vec![text("Project files")]),
            tree_view::tree(
                Some("Project files"),
                None,
                vec![],
                vec![tree_view::branch(
                    closed,
                    "src",
                    false,
                    false,
                    "1",
                    "1",
                    "1",
                    "0",
                    vec![],
                    vec![
                        tree_view::branch_control(
                            closed,
                            false,
                            false,
                            vec![],
                            vec![
                                tree_view::branch_indicator(closed, vec![], vec![text("▸")]),
                                tree_view::branch_text(vec![], vec![text("src")]),
                            ],
                        ),
                        tree_view::branch_content(
                            closed,
                            vec![],
                            vec![
                                tree_view::branch_indent_guide(vec![], vec![]),
                                tree_view::item(
                                    "lib.rs",
                                    false,
                                    false,
                                    "2",
                                    "1",
                                    "1",
                                    "1",
                                    vec![],
                                    vec![
                                        tree_view::item_indicator(false, vec![], vec![]),
                                        tree_view::item_text(vec![], vec![text("lib.rs")]),
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

pub const TREE_VIEW: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "Root / Label / Tree / Branch / BranchControl / BranchIndicator / BranchText / BranchContent / BranchIndentGuide / Item / ItemText / ItemIndicator の 12 anatomy パーツを提供する（tree_view.rs:1-10）。",
        "展開集合（MultiSelect）+ 選択値（SingleSelect）を合成した状態機械 TreeView を提供し、TreeView::render_nodes が深さ・aria-posinset/aria-setsize を再帰的に計算する（tree_view.rs:425-488）。",
        "tree は role=\"tree\"、branch/item は role=\"treeitem\"、branch_content は role=\"group\" を固定出力する WAI-ARIA APG Tree パターン準拠（tree_view.rs:125-259,269-303）。",
        "branch/item は disabled=true のとき aria-disabled=\"true\" を対で付与する（ネイティブ disabled を持たない role=\"treeitem\" のための代替、tree_view.rs:186-193,295-300）。",
        "キーボードナビゲーション・typeahead は SSR 静的マークアップに寄与しない CSR 挙動層のスコープ外（tree_view.rs:72-77）。",
    ],
    arguments: &[
        ArgRow {
            name: "tree: aria_label_text, aria_labelledby_id",
            kind: "Option<&str>, Option<&str>",
            default: "None, None",
            description: "tree のアクセシブルな名前（いずれか片方が Some を推奨、tree_view.rs:125-146）。",
        },
        ArgRow {
            name: "branch/item: level, posinset, setsize, depth",
            kind: "&str, &str, &str, &str",
            default: "",
            description: "呼び出し側が usize から文字列化した aria-level/aria-posinset/aria-setsize/data-depth（tree_view.rs:150-156）。",
        },
        ArgRow {
            name: "TreeView::render_nodes: nodes",
            kind: "&[TreeNode]",
            default: "",
            description: "決定的な静的木を現在の展開・選択状態で再帰描画する（tree_view.rs:486-488）。",
        },
    ],
    examples: &[ExampleEntry {
        title: "Closed branch",
        description: "branch が折りたたまれ、branch_content が hidden の例です（Demo は展開済み）。",
        render: ex_tree_view_closed_branch,
    }],
    keyboard: &[],
    aria: &[
        AriaRow {
            attribute: "role=\"tree\" / role=\"treeitem\" / role=\"group\"",
            description: "tree/branch・item/branch_content に固定出力する（tree_view.rs:125-259,269-303）。",
        },
        AriaRow {
            attribute: "aria-expanded",
            description: "branch のみ（open/closed を反映、tree_view.rs:173-176）。",
        },
        AriaRow {
            attribute: "aria-selected / aria-level / aria-posinset / aria-setsize",
            description: "branch/item 双方に出力する（tree_view.rs:177-180,286-290）。",
        },
        AriaRow {
            attribute: "aria-disabled=\"true\"",
            description: "disabled=true のときのみ branch/item に出力する（tree_view.rs:186-193,295-300）。",
        },
    ],
    demo: None,
};

// ---------------------------------------------------------------------
// Visually Hidden（/primitives/visually-hidden/）
// ---------------------------------------------------------------------

/// 一次情報: `crates/headless-ui/src/visually_hidden.rs:1-46`（モジュール
/// doc、`aria-hidden` を付けない不変条件）、`:50-61`（`root` シグネチャ）。
/// 非テスト行で `role`/`aria-` の出力は 0 件。
fn ex_visually_hidden_status_label() -> Node {
    div(
        vec![],
        vec![
            visually_hidden::root(vec![], vec![text("Build status: ")]),
            text("✓ Passing"),
        ],
    )
}

pub const VISUALLY_HIDDEN: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "Root（span）の 1 anatomy パーツのみで構成する最小の状態非依存パーツ（visually_hidden.rs:50-61）。",
        "視覚的には隠すが支援技術（スクリーンリーダー）には読ませ続けるテキストコンテナであり、装飾要素とは逆に aria-hidden を意図的に付与しない（visually_hidden.rs:11-21）。",
        "styled 層（fandhe-frontend-pre-styled-ui::visually_hidden）は本モジュールが出力する data-scope=\"visually-hidden\"/data-part=\"root\" セレクタを前提に clip 手法の CSS を当てる（visually_hidden.rs:23-29）。",
    ],
    arguments: &[
        ArgRow {
            name: "root: attrs",
            kind: "Vec<(&str, &str)>",
            default: "vec![]",
            description: "呼び出し側が追加する属性。data-scope/data-part は本パーツが固定出力するため上書きされない（visually_hidden.rs:58-61）。root が唯一の公開関数であり attrs/children 以外の型付き引数を持たない。",
        },
        ArgRow {
            name: "root: children",
            kind: "Vec<Node>",
            default: "vec![]",
            description: "視覚的には隠すがスクリーンリーダーには読み上げさせるテキスト・ノード（visually_hidden.rs:58-61）。",
        },
    ],
    examples: &[ExampleEntry {
        title: "Status label prefix",
        description: "視覚的には記号のみを見せつつ、スクリーンリーダーには前置ラベルを読ませる例です。",
        render: ex_visually_hidden_status_label,
    }],
    keyboard: &[],
    aria: &[AriaRow {
        attribute: "(該当なし)",
        description: "root は固有の role/aria-* を出力しない（visually_hidden.rs 全文の非テスト行で role/aria- grep 0 件）。装飾要素の aria-hidden=\"true\" 固定付与パターンとは逆に、本コンポーネントは aria-hidden を意図的に付与しない（visually_hidden.rs:11-21）。",
    }],
    demo: None,
};

/// 本カテゴリ 10 部品の `path -> ComponentPageSpec` テーブル
/// （`crate::primitive_specs::SPEC_TABLES` へ集約される、#1027 と同型）。
/// 並び順は `crate::primitives_catalog::PrimitiveCategory::DataDisplayUtilities`
/// のカタログ順と一致させる。
pub const SPECS: &[(&str, ComponentPageSpec)] = &[
    ("/primitives/avatar/", AVATAR),
    ("/primitives/carousel/", CAROUSEL),
    ("/primitives/json-tree-view/", JSON_TREE_VIEW),
    ("/primitives/scroll-area/", SCROLL_AREA),
    ("/primitives/skip-nav/", SKIP_NAV),
    ("/primitives/splitter/", SPLITTER),
    ("/primitives/steps/", STEPS),
    ("/primitives/tour/", TOUR),
    ("/primitives/tree-view/", TREE_VIEW),
    ("/primitives/visually-hidden/", VISUALLY_HIDDEN),
];
