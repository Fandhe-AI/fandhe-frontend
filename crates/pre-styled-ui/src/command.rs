//! styled Command（shadcn/ui `Command`（cmdk 由来）相当。イシュー #2070、
//! 親 #2067、祖父トラッキング参照軸 #2001。headless 側 anatomy は #2068）。
//!
//! `fandhe_frontend_headless_ui::command`（#2068）が出力する
//! `data-scope="command"` の 10 slot（`root`/`input`/`list`/`empty`/`group`/
//! `group-heading`/`item`/`shortcut`/`separator`/`dialog`）へ、入力欄・
//! リスト・group 見出し・選択行の背景・shortcut の右寄せ・dialog 型の幅と
//! いう shadcn/ui `Command` の意匠を重ねる薄い委譲層である
//! （[`crate::item`]/[`crate::input_group`] と同型の位置付け）。
//!
//! # 選択的 re-export（`pub use ...::*` を使わない理由）
//!
//! [`crate::item`]/[`crate::combobox`] と同型。10 パーツすべてを同名再定義
//! し（呼び出し側 `class` の除去は本モジュールの責務のため）、
//! [`filter_items`]（純粋関数）と [`OpenState`]（`root`/`dialog`/`input` の
//! `state` 引数を本クレート単独で呼べるようにする、イシュー #685 の契約）
//! のみを選択的に再エクスポートする。状態機械
//! [`fandhe_frontend_headless_ui::command::Command`]/`CommandAction` は
//! [`crate::combobox`]/[`crate::select`]/[`crate::menu`] と同じ理由（本
//! モジュールが状態を持たない自由関数群に留まる設計判断、下記「状態機械を
//! 持たない理由」節）で再エクスポートしない。
//!
//! # 状態機械を持たない理由
//!
//! headless [`fandhe_frontend_headless_ui::command`] の 10 パーツ自由関数
//! （[`root`]/[`dialog`]/[`input`]/[`list`]/[`empty`]/[`group`]/
//! [`group_heading`]/[`item`]/[`shortcut`]/[`separator`]）はいずれも props
//! から決定的にマークアップを組み立てる純粋関数であり、状態機械
//! （`Command`）は呼び出し側が任意に埋め込む構成である。本モジュールも
//! この設計をそのまま継承し（[`crate::input_group`] モジュール doc と同型
//! の判断）、状態機械を独自にラップしない。
//!
//! # 責務境界（`docs/policy/intentional-non-adoption.md` §3.25 規則 1）
//!
//! 絞り込み配線（入力変更 → `"input"` dispatch → DOM 反映）・Enter 実行・
//! Cmd/Ctrl+K のグローバルショートカット・フォーカストラップは
//! アプリケーション/`fandhe-frontend-wasm-full`（後続イシュー #2069）の
//! 責務として実装しない。headless が出力する `data-*` を CSS セレクタとして
//! 参照するだけで見た目を切り替える。本モジュール自身は独自の `data-*` を
//! 一切出力しない。
//!
//! # 軸を持たない理由
//!
//! `size`/`variant`/`color-palette` いずれの軸も提供しない
//! （`docs/design/pre-styled-ui-focus-ring-and-size-conventions.md` §4 (d)
//! 「子の寸法に従属するレイアウト部品」に該当。headless にも shadcn/ui にも
//! 軸が無く、[`crate::input_group`] と同じ判断を踏襲する）。このため 10
//! パーツとも見た目クラスを一切付与しない（呼び出し側 `class` は
//! `drop_class_attr` で除去のみ行う）。
//!
//! # `empty` の表示切替 CSS（headless の SSR 決定性契約との対応）
//!
//! headless [`fandhe_frontend_headless_ui::command::empty`] は `present` が
//! `true` のときのみ `data-empty` 存在属性を出力し、`hidden` を一切付与し
//! ない（表示/非表示の切り替えは呼び出し側/本クレートの CSS の責務、
//! headless 側モジュール doc「`data-empty` の付与先と SSR 決定性」節
//! 参照）。本 `recipe` は `empty` slot を既定で `display: none` にし、
//! `[data-empty]` が付いているときのみ `display: block` へ切り替える
//! （逆方向〔既定可視 + `list[data-empty]` で非表示〕にしない理由:
//! [`empty`] は [`list`] の外・[`root`] の直接の子に置く配置制約
//! （headless モジュール doc参照）のため兄弟セレクタが必要になり
//! [`SlotRecipe`] では表現できない）。
//!
//! # `item` の選択表現（`data-selected` を hover が洗い流さないための specificity 対策）
//!
//! [`item`] は選択行の背景を `[data-selected]`（[`StateCondition::Attr`]）
//! で表現する。[`SlotRecipe::css`] は hover 規則を `@media (hover: hover)`
//! として常に末尾へ集約出力するため、素の [`StateCondition::Hover`] を
//! 併用すると `:hover:not([data-disabled])`（specificity (0,4,0)）が
//! `[data-selected]`（specificity (0,1,0)）より高く、かつソース順でも
//! hover が後になるため、選択行にポインタが乗ると選択色が hover の淡色
//! （muted）で上書きされコントラストが崩れる（[`crate::combobox`] の
//! `item` hover と同型の問題、[`StateCondition::HoverExceptAttr`] rustdoc
//! 参照）。本 `recipe` は
//! `.state("item", StateCondition::HoverExceptAttr("data-selected"), ...)`
//! を使い、選択行を hover 対象から除外する（`:not([data-disabled])` も
//! 併せて除外されるため disabled 行への対策は不要）。
//!
//! # `dialog` の `[hidden]`・幅トークン・backdrop 不在
//!
//! [`dialog`] は headless [`fandhe_frontend_headless_ui::command::dialog`]
//! が closed 時に付与する `hidden` 存在属性を確実に非表示化として機能
//! させるため、[`crate::dialog`] の `positioner` と同じく
//! `.state("dialog", StateCondition::Attr("hidden"), [display: none])` を
//! 明示登録する（base の `position: fixed` 等が UA 既定の
//! `[hidden] { display: none }` を詳細度で上書きしてしまうのを防ぐ
//! fail-closed、[`crate::dialog`] モジュール doc「closed 時の `positioner`
//! は必ず非表示化する」節参照）。幅は `--fandhe-command-dialog-max-width`
//! （既定 32rem、[`crate::dialog`] の `--fandhe-dialog-content-max-width`
//! と同じ命名規約）で決める。headless `dialog` パーツは単一要素であり
//! [`crate::dialog`] のような独立した `backdrop` パーツを持たない
//! （headless モジュール doc「`dialog` パーツを `crate::dialog` へ委譲しない
//! 理由」節参照）ため、本モジュールも backdrop を追加しない（意図的な
//! スコープ外、下記「スコープ外」節参照）。加えて `dialog` は
//! `overflow-y: auto`（横方向のみ `overflow-x: hidden`）とし、画面高が
//! 低く候補（`item`）が多い環境で `root`（`input` + `list`）の実高が
//! `dialog` の `max-height` を超えても末尾がクリップされたまま到達不能に
//! ならないようにする（`dialog` 自体がスクロールコンテナになる。フォーカス
//! リング指摘・#2241 codex レビュー対応）。
//!
//! # フォーカスリング（`input` の `outline: none` を `root` と `dialog`
//! の `:focus-within` で補う）
//!
//! [`input`] slot は `outline: none`（ブラウザ既定フォーカス枠の除去）を
//! 持つが、これを単独で置くと `input` へフォーカスしたときの視認手段が
//! 消える（フォーカスリング共通規約
//! `docs/design/pre-styled-ui-focus-ring-and-size-conventions.md` §3 の
//! 「祖先に canonical リングがある場合のみ許容」に対応する必要がある）。
//! 本 `recipe` は祖先 [`root`] の `:focus-within`（[`StateCondition::FocusWithin`]）
//! へ [`crate::recipe::focus_ring_declarations`]（`FocusRingColor::Token`、
//! `palette` 軸を持たないため。[`FocusRingOffset::Outside`]）を登録する
//! （[`crate::combobox`] が `control` の `:focus-within` へ付ける対策と
//! 同型。Tab で [`input`] へ移動したとき `root` の外枠にリングが表示され
//! フォーカス位置を視認できる）。単独 `root`（`dialog` の外）ではこの
//! `Outside` オフセットのリングがそのまま可視化される。
//!
//! **`dialog` 内では `root` のリングだけでは不十分（#2241 codex レビュー
//! P1 指摘）**: 公開 Examples のように `dialog` → `root` → `input` と入れ
//! 子にした構成では、`root` の `Outside` リングは `root` 自身の外側へ
//! はみ出して描画される。`dialog` は `padding: 0` かつ
//! `overflow-x: hidden`/`overflow-y: auto`（上記「`dialog` のスクロール」
//! 節参照）を持つため、この祖先のクリッピングコンテキストにより `root`
//! のリングは `dialog` の境界で見えなくなる（`outline` は自分自身の
//! `overflow` では切れないが、祖先の `overflow: hidden`/`auto` では
//! クリップされる CSS の性質）。このため本 `recipe` は [`dialog`]
//! 自身にも `:focus-within` の canonical リングを登録する（`dialog` 自体
//! の `overflow` はその要素自身の `outline` を切らないため、`dialog` の
//! 外枠として確実に可視化される）。`dialog` を使わない単独 `root` の
//! 構成では `dialog` slot 自体が存在せずこのセレクタは発火しないため、
//! 二重リングにはならない。
//!
//! # `shortcut` 内への `kbd` 合成
//!
//! [`shortcut`] は固定属性を持たない `span`（headless モジュール doc
//! 「`shortcut` のタグ」節参照）であり、本モジュールは API を増やさず
//! `shortcut` の `children` へ [`crate::kbd::kbd`] を渡す使い方で `kbd` を
//! 合成する（Demo・Examples・本 rustdoc の [`shortcut`] 参照）。`recipe`
//! は `shortcut` slot 自身に `margin-inline-start: auto` を与えて右寄せする
//! のみで、`kbd` 側の見た目には関与しない。
//!
//! # raw CSS 追記の理由（[`SlotRecipe`] が子結合子を表現できないため）
//!
//! [`crate::item`] と同型のパターンで、[`stylesheet`] は `recipe().css()`
//! の出力へ [`crate::css::serialize_rule`] を使った素の子結合子（`>`）
//! セレクタを 1 本追記する。対象は dialog 内に配置された root の二重枠
//! （`dialog` 自身の枠 + `root` の枠が二重に見えてしまう）を解除する
//! `[data-scope="command"][data-part="dialog"] > [data-scope="command"][data-part="root"]`
//! の 1 セレクタのみ（`serialize_rule` は selector 文字列を検証しないため
//! 静的リテラルのみを使う）。
//!
//! # セキュリティ不変条件
//!
//! - 全出力は headless [`fandhe_frontend_headless_ui::command`] →
//!   `fandhe_frontend_core::render` の既定エスケープ（REQ-1）を必ず
//!   経由する。`raw_html()` は使用しない。
//! - 呼び出し側 `class` は `drop_class_attr` で除去してから headless
//!   関数へ委譲する（10 パーツすべて）。
//! - [`stylesheet`] が組み立てる CSS 宣言・selector 断片はすべて
//!   コンパイル時静的リテラルであり、[`crate::css::decl`]/
//!   [`crate::css::serialize_rule`] の検証を通る値のみを使う。
//!
//! # スコープ外
//!
//! - **#2069（`fandhe-frontend-wasm-full` 配線）**: 絞り込み・実行フック・
//!   キーボード操作・フォーカストラップの配線（上記「責務境界」節参照）。
//! - **`dialog` backdrop の新設**: headless anatomy の変更を伴うため
//!   本イシューでは追随しない（上記「`dialog` の `[hidden]`・幅トークン・
//!   backdrop 不在」節参照）。
//! - **`Command`/`CommandAction` 状態機械の再エクスポート**:
//!   [`crate::combobox`]/[`crate::select`]/[`crate::menu`] と同じ判断
//!   （上記「状態機械を持たない理由」節参照）。
//! - **`examples/` への追加**: `embedded-examples` バイト一致同期・cli
//!   semver バンプ連鎖を誘発するため、本イシューでは対象外とする。

use crate::class_attr::drop_class_attr;
use crate::css::{decl, serialize_rule};
use crate::recipe::{
    focus_ring_declarations, FocusRingColor, FocusRingOffset, SlotRecipe, StateCondition,
};
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;

// headless 型のうち本モジュールが必要とするのは純粋関数 [`filter_items`]
// と `state` 引数の型 [`OpenState`] のみ（`crate::combobox`/`crate::select`
// と同型の規約）。パーツ関数 10 件は呼び出し側 `class` の除去を担うため
// 同名再定義する。
pub use fandhe_frontend_headless_ui::command::filter_items;
pub use fandhe_frontend_headless_ui::state::OpenState;

/// slot 一覧（headless [`fandhe_frontend_headless_ui::command`] の anatomy
/// と 1:1、10 パーツ）。
const SLOTS: &[&str] = &[
    "root",
    "input",
    "list",
    "empty",
    "group",
    "group-heading",
    "item",
    "shortcut",
    "separator",
    "dialog",
];

/// この styled Command の既定 CSS を組み立てる（内部ヘルパ、[`stylesheet`]
/// のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    let root_base = vec![
        decl("display", "flex"),
        decl("flex-direction", "column"),
        decl("width", "100%"),
        decl("min-width", "0"),
        decl("box-sizing", "border-box"),
        decl("overflow", "hidden"),
        decl("background", "var(--fandhe-color-bg)"),
        decl("color", "var(--fandhe-color-fg)"),
        decl("border", "1px solid var(--fandhe-color-border)"),
        decl("border-radius", "var(--fandhe-radius-md)"),
    ];

    let input_base = vec![
        decl("display", "block"),
        decl("width", "100%"),
        decl("box-sizing", "border-box"),
        decl("padding", "var(--fandhe-space-3)"),
        decl("border", "0"),
        decl("border-bottom", "1px solid var(--fandhe-color-border)"),
        decl("background", "transparent"),
        decl("color", "inherit"),
        decl("font", "inherit"),
        decl("font-size", "var(--fandhe-font-font-size-sm)"),
        decl("outline", "none"),
    ];

    let list_base = vec![
        decl(
            "max-height",
            "var(--fandhe-command-list-max-height, 18.75rem)",
        ),
        decl("overflow-y", "auto"),
        decl("overflow-x", "hidden"),
        decl("padding", "var(--fandhe-space-1)"),
    ];

    let empty_base = vec![
        decl("display", "none"),
        decl("padding", "var(--fandhe-space-6) 0"),
        decl("text-align", "center"),
        decl("font-size", "var(--fandhe-font-font-size-sm)"),
        decl("color", "var(--fandhe-color-fg-muted)"),
    ];

    let group_base = vec![
        decl("padding", "var(--fandhe-space-1)"),
        decl("color", "var(--fandhe-color-fg)"),
    ];

    let group_heading_base = vec![
        decl("padding", "var(--fandhe-space-1) var(--fandhe-space-2)"),
        decl("font-size", "var(--fandhe-font-font-size-xs)"),
        decl("font-weight", "var(--fandhe-font-font-weight-medium)"),
        decl("color", "var(--fandhe-color-fg-muted)"),
    ];

    let item_base = vec![
        decl("display", "flex"),
        decl("align-items", "center"),
        decl("gap", "var(--fandhe-space-2)"),
        decl("padding", "var(--fandhe-space-2)"),
        decl("border-radius", "var(--fandhe-radius-sm)"),
        decl("font-size", "var(--fandhe-font-font-size-sm)"),
        decl("cursor", "default"),
        decl("user-select", "none"),
        decl("outline", "none"),
        decl(
            "transition",
            "background var(--fandhe-motion-duration-fast, 150ms) ease, color var(--fandhe-motion-duration-fast, 150ms) ease",
        ),
    ];

    let shortcut_base = vec![
        decl("margin-inline-start", "auto"),
        decl("display", "inline-flex"),
        decl("align-items", "center"),
        decl("gap", "var(--fandhe-space-1)"),
        decl("font-size", "var(--fandhe-font-font-size-xs)"),
        decl("letter-spacing", "0.1em"),
        decl("color", "var(--fandhe-color-fg-muted)"),
    ];

    let separator_base = vec![
        decl("height", "1px"),
        decl("margin", "var(--fandhe-space-1) 0"),
        decl("background", "var(--fandhe-color-border)"),
    ];

    let dialog_base = vec![
        decl("position", "fixed"),
        decl("top", "50%"),
        decl("left", "50%"),
        decl("transform", "translate(-50%, -50%)"),
        decl("width", "100%"),
        decl("max-width", "var(--fandhe-command-dialog-max-width, 32rem)"),
        decl("max-height", "calc(100vh - var(--fandhe-space-8))"),
        decl("box-sizing", "border-box"),
        decl("padding", "0"),
        decl("overflow-y", "auto"),
        decl("overflow-x", "hidden"),
        decl("z-index", "var(--fandhe-z-index-modal, 1001)"),
        decl("background", "var(--fandhe-color-bg)"),
        decl("border", "1px solid var(--fandhe-color-border)"),
        decl("border-radius", "var(--fandhe-radius-lg)"),
        decl("box-shadow", "var(--fandhe-shadow-lg)"),
        decl("outline", "none"),
    ];

    SlotRecipe::new("command", SLOTS)
        .base("root", root_base)
        .base("input", input_base)
        .base("list", list_base)
        .base("empty", empty_base)
        .base("group", group_base)
        .base("group-heading", group_heading_base)
        .base("item", item_base)
        .base("shortcut", shortcut_base)
        .base("separator", separator_base)
        .base("dialog", dialog_base)
        .state(
            // `input` の `outline: none`（フォーカス位置の視認手段が
            // 消える）に対し、祖先 `root` の `:focus-within` へ canonical
            // フォーカスリングを登録する（[`crate::combobox`] が `control`
            // の `:focus-within` へ付ける対策と同型。モジュール doc
            // 「フォーカスリング」節参照）。
            "root",
            StateCondition::FocusWithin,
            focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Outside),
        )
        .state(
            // `dialog` 内では `root` の `Outside` リングが祖先 `dialog` の
            // `padding: 0`/`overflow-x: hidden`/`overflow-y: auto` に
            // よってクリップされ見えなくなる（モジュール doc「`dialog`
            // 内では `root` のリングだけでは不十分」節参照、#2241 codex
            // レビュー P1 対応）。`dialog` 自身の `:focus-within` へも
            // canonical リングを登録し、`dialog` を使わない構成では
            // `dialog` slot 自体が無くこのルールが発火しないため二重
            // リングにはならない。
            "dialog",
            StateCondition::FocusWithin,
            focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Outside),
        )
        .state(
            "empty",
            StateCondition::Attr("data-empty"),
            vec![decl("display", "block")],
        )
        .state(
            "item",
            StateCondition::Attr("data-selected"),
            vec![
                decl("background", "var(--fandhe-color-bg-muted)"),
                decl("color", "var(--fandhe-color-fg)"),
            ],
        )
        .state(
            "item",
            StateCondition::Attr("data-disabled"),
            vec![decl("opacity", "0.5"), decl("cursor", "not-allowed")],
        )
        .state(
            "item",
            // 選択行の背景を hover が洗い流さないよう除外する
            // （モジュール doc「`item` の選択表現」節参照）。
            StateCondition::HoverExceptAttr("data-selected"),
            vec![decl("background", "var(--fandhe-color-bg-subtle)")],
        )
        .state(
            "dialog",
            // closed 時の `hidden` を確実に非表示化させる
            // （モジュール doc「`dialog` の `[hidden]`」節参照）。
            StateCondition::Attr("hidden"),
            vec![decl("display", "none")],
        )
}

/// この styled Command が生成する静的 CSS 全量を返す（決定的。
/// [`crate::item::stylesheet`] と同じ契約）。dialog 内 root の二重枠を
/// 解除する raw CSS 追記を含む（モジュール doc「raw CSS 追記の理由」節
/// 参照）。
#[must_use]
pub fn stylesheet() -> String {
    let mut out = recipe().css();

    const DIALOG_ROOT: &str =
        r#"[data-scope="command"][data-part="dialog"] > [data-scope="command"][data-part="root"]"#;
    if let Some(rule) = serialize_rule(
        DIALOG_ROOT,
        &[
            decl("border", "0"),
            decl("border-radius", "0"),
            decl("box-shadow", "none"),
        ],
    ) {
        if !out.is_empty() {
            out.push('\n');
        }
        out.push_str(&rule);
    }

    out
}

/// styled `root` パーツを組み立てる。見た目クラスは付与せず（モジュール doc
/// 「軸を持たない理由」節参照）、呼び出し側 `class` を `drop_class_attr`
/// で除去してから [`fandhe_frontend_headless_ui::command::root`] へそのまま
/// 委譲する。
#[must_use]
pub fn root<'a>(
    state: OpenState,
    empty: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    fandhe_frontend_headless_ui::command::root(state, empty, drop_class_attr(attrs), children)
}

/// styled `dialog` パーツを組み立てる。
#[must_use]
pub fn dialog<'a>(
    state: OpenState,
    label: &'a str,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    fandhe_frontend_headless_ui::command::dialog(state, label, drop_class_attr(attrs), children)
}

/// styled `input` パーツを組み立てる。
#[must_use]
pub fn input<'a>(
    state: OpenState,
    value: &'a str,
    list_id: &'a str,
    activedescendant: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
) -> Node {
    fandhe_frontend_headless_ui::command::input(
        state,
        value,
        list_id,
        activedescendant,
        drop_class_attr(attrs),
    )
}

/// styled `list` パーツを組み立てる。
#[must_use]
pub fn list<'a>(
    id: &'a str,
    label: &'a str,
    empty: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    fandhe_frontend_headless_ui::command::list(id, label, empty, drop_class_attr(attrs), children)
}

/// styled `empty` パーツを組み立てる。表示/非表示は `present` に応じた
/// `data-empty` の有無で切り替わる（モジュール doc「`empty` の表示切替
/// CSS」節参照）。
#[must_use]
pub fn empty<'a>(present: bool, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    fandhe_frontend_headless_ui::command::empty(present, drop_class_attr(attrs), children)
}

/// styled `group` パーツを組み立てる。
#[must_use]
pub fn group<'a>(
    labelledby: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    fandhe_frontend_headless_ui::command::group(labelledby, drop_class_attr(attrs), children)
}

/// styled `group-heading` パーツを組み立てる。
#[must_use]
pub fn group_heading<'a>(
    id: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    fandhe_frontend_headless_ui::command::group_heading(id, drop_class_attr(attrs), children)
}

/// styled `item` パーツを組み立てる。選択行の背景は `selected` に応じた
/// `data-selected` の有無で切り替わる（モジュール doc「`item` の選択表現」
/// 節参照）。
#[must_use]
pub fn item<'a>(
    selected: bool,
    disabled: bool,
    value: &'a str,
    id: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    fandhe_frontend_headless_ui::command::item(
        selected,
        disabled,
        value,
        id,
        drop_class_attr(attrs),
        children,
    )
}

/// styled `shortcut` パーツを組み立てる。呼び出し側は `children` へ
/// [`crate::kbd::kbd`] を渡すことで `kbd` を合成できる（モジュール doc
/// 「`shortcut` 内への `kbd` 合成」節参照）。
#[must_use]
pub fn shortcut<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    fandhe_frontend_headless_ui::command::shortcut(drop_class_attr(attrs), children)
}

/// styled `separator` パーツを組み立てる。
#[must_use]
pub fn separator<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    fandhe_frontend_headless_ui::command::separator(drop_class_attr(attrs), children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text as core_text};

    #[test]
    fn stylesheet_is_deterministic_and_targets_data_scope_selectors() {
        let a = stylesheet();
        let b = stylesheet();
        assert_eq!(a, b);
        assert!(a.contains(r#"[data-scope="command"][data-part="root"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let out = stylesheet();
        assert!(!out.contains("</style"));
        assert!(!out.contains('<'));
    }

    #[test]
    fn stylesheet_toggles_empty_visibility_via_data_empty() {
        let out = stylesheet();
        assert!(out.contains(r#"[data-scope="command"][data-part="empty"] {"#));
        assert!(out.contains(r#"[data-scope="command"][data-part="empty"][data-empty] {"#));
        assert!(out.contains("display: none;"));
        assert!(out.contains("display: block;"));
    }

    #[test]
    fn stylesheet_selected_item_background_and_hover_exclusion() {
        let out = stylesheet();
        assert!(out.contains(r#"[data-scope="command"][data-part="item"][data-selected] {"#));
        assert!(out.contains(":hover:not([data-disabled]):not([data-selected])"));
    }

    #[test]
    fn stylesheet_dialog_hidden_and_double_border_removal() {
        let out = stylesheet();
        assert!(out.contains(r#"[data-scope="command"][data-part="dialog"][hidden] {"#));
        assert!(out.contains(
            r#"[data-scope="command"][data-part="dialog"] > [data-scope="command"][data-part="root"] {"#
        ));
    }

    #[test]
    fn stylesheet_never_generates_class_based_variant_classes() {
        let out = stylesheet();
        assert!(!out.contains("fd-command--"));
    }

    #[test]
    fn root_connects_to_headless_command_scope() {
        let html = render(&root(OpenState::Open, false, vec![], vec![]));
        assert!(html.contains(r#"data-scope="command" data-part="root""#));
    }

    #[test]
    fn all_parts_connect_to_headless_command_scope() {
        let dialog_html = render(&dialog(OpenState::Closed, "Command Menu", vec![], vec![]));
        assert!(dialog_html.contains(r#"data-scope="command" data-part="dialog""#));

        let input_html = render(&input(OpenState::Open, "ca", "list-1", None, vec![]));
        assert!(input_html.contains(r#"data-scope="command" data-part="input""#));

        let list_html = render(&list("list-1", "Suggestions", false, vec![], vec![]));
        assert!(list_html.contains(r#"data-scope="command" data-part="list""#));

        let empty_html = render(&empty(true, vec![], vec![core_text("No results")]));
        assert!(empty_html.contains(r#"data-scope="command" data-part="empty""#));
        assert!(empty_html.contains("data-empty"));

        let group_html = render(&group(Some("heading-1"), vec![], vec![]));
        assert!(group_html.contains(r#"data-scope="command" data-part="group""#));

        let group_heading_html = render(&group_heading(
            Some("heading-1"),
            vec![],
            vec![core_text("Suggestions")],
        ));
        assert!(group_heading_html.contains(r#"data-scope="command" data-part="group-heading""#));

        let item_html = render(&item(
            true,
            false,
            "calendar",
            Some("item-1"),
            vec![],
            vec![core_text("Calendar")],
        ));
        assert!(item_html.contains(r#"data-scope="command" data-part="item""#));
        assert!(item_html.contains("data-selected"));

        let shortcut_html = render(&shortcut(vec![], vec![core_text("⌘K")]));
        assert!(shortcut_html.contains(r#"data-scope="command" data-part="shortcut""#));

        let separator_html = render(&separator(vec![], vec![]));
        assert!(separator_html.contains(r#"data-scope="command" data-part="separator""#));
    }

    #[test]
    fn caller_class_is_dropped_on_every_part() {
        let root_node = root(
            OpenState::Open,
            false,
            vec![("class", "evil")],
            vec![
                input(OpenState::Open, "", "list-1", None, vec![("class", "evil")]),
                list(
                    "list-1",
                    "Suggestions",
                    false,
                    vec![("class", "evil")],
                    vec![group(
                        Some("heading-1"),
                        vec![("class", "evil")],
                        vec![
                            group_heading(
                                Some("heading-1"),
                                vec![("class", "evil")],
                                vec![core_text("Suggestions")],
                            ),
                            item(
                                false,
                                false,
                                "calendar",
                                None,
                                vec![("class", "evil")],
                                vec![
                                    core_text("Calendar"),
                                    shortcut(vec![("class", "evil")], vec![]),
                                ],
                            ),
                            separator(vec![("class", "evil")], vec![]),
                        ],
                    )],
                ),
                empty(false, vec![("class", "evil")], vec![]),
            ],
        );
        // `dialog` も見た目クラスを付与しない 10 パーツ目のため、
        // ここで root を包んで固定する（他 9 パーツと同じ契約を漏らさず
        // 検証する）。
        let html = render(&dialog(
            OpenState::Open,
            "Command Menu",
            vec![("class", "evil")],
            vec![root_node],
        ));
        assert!(!html.contains("evil"));
        assert_eq!(html.matches("class=\"").count(), 0);
    }

    #[test]
    fn filter_items_is_reexported_and_filters_case_insensitively() {
        let items: &[(&str, &str)] = &[("calendar", "Calendar"), ("search", "Search Emoji")];
        let filtered = filter_items(items, "CAL");
        assert_eq!(filtered, vec![("calendar", "Calendar")]);
    }
}
