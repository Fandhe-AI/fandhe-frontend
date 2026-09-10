//! styled Bubble（shadcn/ui `Bubble` 相当。イシュー #2109、親 #2107、
//! 祖父トラッキング参照軸 #2001。headless 側 anatomy は #2108）。
//!
//! `fandhe_frontend_headless_ui::bubble`（#2108）が出力する
//! `data-scope="bubble"` の 6 slot（`root`/`content`/`reactions`/
//! `reaction`/`collapse-trigger`/`collapse-content`）へ、チャット吹き出し
//! 1 個の意匠（塗り・枠線・無装飾の 3 形態、連続発言の角丸連結、
//! リアクションチップ、折りたたみのフェード）を重ねる薄い委譲層である
//! （[`crate::message`] と同型の構成）。
//!
//! # 選択的 re-export（`pub use ...::*` を使わない理由）
//!
//! [`crate::message`]/[`crate::item`] と同型。6 パーツすべてを同名再定義
//! し（呼び出し側 `class` の除去は本モジュールの責務のため）、
//! [`BubbleRootProps`]/[`BubbleVariant`]/[`BubbleGroupPosition`]/
//! [`MessageAlign`]/[`OpenState`] の 5 型のみを選択的に再エクスポートする。
//! `MessageAlign` は会話系部品が共有する語彙であり第 2 の align 列挙型を
//! 作らない（headless [`mod@fandhe_frontend_headless_ui::bubble`] の
//! モジュール doc「会話系 4 部品の共通語彙への追随」参照）。`OpenState` は
//! [`crate::collapsible`] と同じ再エクスポート方式で、呼び出し側が
//! headless-ui を直接依存せずに済むようにする。
//!
//! # 状態機械を持たない理由
//!
//! headless [`mod@fandhe_frontend_headless_ui::bubble`] 自身が状態機械を
//! 持たない静的な自由関数群であるため、本モジュールもその設計をそのまま
//! 継承する（[`crate::message`] モジュール doc と同型の判断）。
//!
//! # 責務境界（`docs/policy/intentional-non-adoption.md` §3.25）
//!
//! リアクションの押下・集計・トグル、`data-group-position` の算出
//! （「リスト中の何番目か」「前後の発言者が同じか」の突き合わせ）、
//! 折りたたみのトグル自体はアプリケーションロジックであり実装しない
//! （規則 1）。headless が出力する `data-*` を CSS セレクタとして参照する
//! だけで見た目を切り替える。
//!
//! # `data-variant`/`data-align`/`data-group-position`/`data-selected`/
//! `data-state`: headless の `data-*` を `AttrEq`/`Attr`/`AttrEqAll` で
//! 参照する（[`crate::message`] と同型の意図的差分）
//!
//! headless `bubble::root` は `data-variant`（`solid`/`outline`/`plain`）・
//! `data-align`（`start`/`end`）・`data-group-position`
//! （`single`/`first`/`middle`/`last`）を、`bubble::reaction` は
//! `data-selected`（存在属性）を、`bubble::collapse_trigger`/
//! `bubble::collapse_content` は `data-state`（`open`/`closed`）を固定
//! 出力済み（`crates/headless-ui/src/bubble.rs`）。本モジュールはこれらを
//! [`StateCondition::AttrEq`]/[`StateCondition::Attr`]/
//! [`StateCondition::AttrEqAll`] で**参照するのみ**とし、class ベースの
//! [`SlotRecipe::variant`] を持たない
//! （`docs/design/pre-styled-ui-data-attr-vocabulary.md` §2.2「役割 B:
//! 参照のみ」）。したがって 6 パーツすべて見た目クラスを付与しない同名
//! 再定義であり、[`crate::message`]/[`crate::item`] と同じパターンを
//! 踏襲する。
//!
//! # `ColorPalette` 軸を持たない理由（custom property フックで対応）
//!
//! headless rustdoc「`data-variant`（shadcn の 7 色調を 3 形態へ縮約）」が
//! 述べるとおり、実際の色調選択（アクセント・破壊的操作色等）は
//! `ColorPalette` 軸（イシュー #1678）の責務と位置づけられているが、本
//! イシューのスコープ（variant 3 種・角丸連結・reaction・フェード）には
//! 含まれない。`root` は `--fandhe-bubble-bg`/`--fandhe-bubble-fg`/
//! `--fandhe-bubble-border` の 3 custom property を既定値付きで宣言する
//! のみとし、これらを呼び出し側が上書きするフックとして残す
//! （軸追加は後続提案、`.claude/rules/out-of-scope-tracking.md` 対応）。
//!
//! # 角丸連結（`AttrEqAll` を採用する理由）
//!
//! 連続発言の角丸連結は `data-align`（`start`/`end`）×
//! `data-group-position`（`first`/`middle`/`last`。`single` は base のまま
//! 据え置き）の複合条件で決まるが、`root` 自身の 2 属性の組み合わせのみで
//! 表現できるため、[`crate::message`]（子結合子・`:has()` 等の raw CSS
//! 追記）とは異なり [`SlotRecipe::state`] の [`StateCondition::AttrEqAll`]
//! （[`crate::tour`]/[`crate::tabs`] の side×align 組み合わせと同型）だけで
//! 完結する。raw CSS 追記は不要（`stylesheet()` は `recipe().css()` を
//! そのまま返す）。角丸連結の算出対象（何番目の発言か）を求める計算自体は
//! 利用者責務のまま変わらない（headless 契約を継承、上記責務境界節参照）。
//!
//! # 高さトランジション（イシュー #2192 の共通機構を適用、#2282）
//!
//! 当初（イシュー #2109）は `collapse-content` の `opacity` を
//! `data-state` で 0/1 切り替える構成だったが、headless
//! `collapse_content` は closed 時に `hidden` 属性を出力し
//! `display: none` は transition を無効化するため
//! （[`crate::dialog`] の backdrop と同じ前提）、opacity 遷移が実際に
//! 見えるのはクライアントランタイムが `hidden` を外す前後で
//! `data-state` を切り替える場合のみであり、SSR 単独では即時表示・
//! 即時非表示になっていた（既知の「フェードの限界」）。
//!
//! [`crate::collapsible`]/[`crate::accordion`] が採用済みの案 C
//! （`docs/design/collapsible-height-animation.md`、`hidden` 契約を維持
//! したまま `@starting-style` + `transition-behavior: allow-discrete`
//! と `fandhe-frontend-wasm-full` の実測高さ CSS 変数
//! （`--fandhe-content-height`）を組み合わせる方式）の共通機構
//! （[`crate::recipe::SlotRecipe::content_height_transition`]、
//! イシュー #2192）を `collapse-content` slot へそのまま適用した
//! （`recipe()` 末尾参照）。
//!
//! `collapse-content` の高さトランジションが実際に成立するかどうかは、
//! (1) `hidden` を切り替える**トリガーの配線**と (2) 共通 preset が
//! 出す CSS 自体の**遷移条件**という別々の 2 点に依存する（`content_
//! height_transition` の CSS が要求する条件は codex レビュー指摘により
//! ここで明確化する。`crates/pre-styled-ui/src/recipe.rs` の
//! `content_height_open_declarations`/`supports_not_calc_size_height_
//! state` rustdoc 参照）。
//!
//! - **トリガー未配線（既知の限界）**: `crates/wasm-full/src/headless.rs`
//!   の `MAPPING_TABLE` に `(bubble, collapse-trigger)` の行が無く
//!   （headless [`mod@fandhe_frontend_headless_ui::bubble`] rustdoc
//!   「wasm-full 未配線」節参照）、`wire_headless_component` 経由の
//!   クリックでは `hidden` の切り替え自体が dispatch されない。呼び出し
//!   側が独自に `data-state`/`hidden` を切り替える経路を用意する必要が
//!   ある（実運用上の限界、`.claude/rules/out-of-scope-tracking.md`
//!   対応候補）。
//! - **`calc-size()` 対応ブラウザ**: `hidden` の切り替えだけで高さ
//!   トランジションが成立する。`fandhe-frontend-wasm-full` の
//!   `content_height` 同期（`--fandhe-content-height` への実測値
//!   書き込み）は**不要**であり、同期しなくても `height:
//!   calc-size(auto, size)` が定常状態を `auto` 相当として扱いつつ
//!   `0 → 定常値` の遷移を自動で成立させる。
//! - **`calc-size()` 未対応ブラウザ**: 共通 preset の `@supports not
//!   (height: calc-size(auto, size))` ブロックが `transition: none`
//!   相当を適用するため、`content_height` 同期の有無に関わらず高さ
//!   トランジションは成立せず、`var(--fandhe-content-height, auto)`
//!   の `auto` フォールバックによる `hidden` の即時切替（閉固定）の
//!   ままになる。
//!
//! preset の base 2 個目ブロックは既存の `opacity` transition 宣言
//! （`transition-property: opacity` 等）より後に登録されるため
//! `transition-property`/`transition-duration`/`transition-timing-function`
//! の longhand を上書きし、`opacity` は遷移対象から外れて `data-state`
//! による離散切替になる（フェードは実質廃止）。既存の `opacity` state
//! 規則・base 規則は golden 純追加原則により削除しない（下記
//! `stylesheet_fades_collapse_content_and_hides_hidden` テスト参照。
//! テキストとしては残るためブロック自体は PASS するが実際の見た目は
//! 高さトランジションに従う）。
//!
//! 既知の限界: `root` の `gap: var(--fandhe-space-1)` は
//! `collapse-content` が `display: none` へ到達した瞬間に消えるため、
//! 閉じ切りの直前に軽微な段差（レイアウトのずれ）が生じる
//! （対策は本イシュー外、`.claude/rules/out-of-scope-tracking.md`
//! 対応候補）。
//!
//! `prefers-reduced-motion` は `Theme::to_css` が duration トークンを
//! 0ms へ一括上書きするため本モジュール側では書かない
//! （[`crate::calendar`] と同型）。
//!
//! # セキュリティ不変条件
//!
//! - 全出力は headless [`mod@fandhe_frontend_headless_ui::bubble`] →
//!   [`fandhe_frontend_core::render`] の既定エスケープ（REQ-1）を必ず
//!   経由する。`raw_html()` は使用しない。
//! - 呼び出し側 `class` は [`drop_class_attr`] で除去してから headless
//!   関数へ委譲する（6 パーツすべて）。
//! - [`stylesheet`] が組み立てる CSS 宣言はすべてコンパイル時静的
//!   リテラルであり、[`crate::css::decl`] の検証を通る値のみを使う。
//!
//! # スコープ外
//!
//! - `fandhe-frontend-wasm-full` の `MAPPING_TABLE` への
//!   `(bubble, collapse-trigger)` → `"toggle"` 配線
//!   （headless rustdoc「wasm-full 未配線」参照）。
//! - `ColorPalette` 軸の追加（上記「`ColorPalette` 軸を持たない理由」
//!   参照）。
//! - `examples/headless-pre-styled-ui` への bubble 追加。
//! - marker は #2115 で Themes 化済み（`data-role`/`data-align` を持たない
//!   設計のため語彙追随は不要、`crate::marker` モジュール doc参照）。
//!   attachment は #2112 で同様に Themes 化済み（`crate::attachment`
//!   モジュール doc参照）。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{
    focus_ring_declarations, transition_declarations, FocusRingColor, FocusRingOffset,
    MotionDuration, SlotRecipe, StateCondition,
};
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;

// headless 型のうち見た目クラスを付与しない本モジュールが必要とするのは
// props/variant/align/group-position/state 型のみ（`crate::message` と
// 同型の規約）。パーツ関数 6 件は呼び出し側 `class` の除去を担うため
// 同名再定義する。
pub use fandhe_frontend_headless_ui::bubble::{
    BubbleGroupPosition, BubbleRootProps, BubbleVariant,
};
pub use fandhe_frontend_headless_ui::message::MessageAlign;
pub use fandhe_frontend_headless_ui::state::OpenState;

/// slot 一覧（headless [`mod@fandhe_frontend_headless_ui::bubble`] の
/// anatomy と 1:1、6 パーツ）。
const SLOTS: &[&str] = &[
    "root",
    "content",
    "reactions",
    "reaction",
    "collapse-trigger",
    "collapse-content",
];

/// この styled Bubble の既定 CSS を組み立てる（内部ヘルパ、[`stylesheet`]
/// のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    let root_base = vec![
        decl("display", "flex"),
        decl("flex-direction", "column"),
        decl("gap", "var(--fandhe-space-1)"),
        decl("align-self", "flex-start"),
        decl("max-width", "var(--fandhe-bubble-max-width, 32rem)"),
        decl("min-width", "0"),
        decl("padding", "var(--fandhe-space-2) var(--fandhe-space-3)"),
        decl("border-radius", "var(--fandhe-radius-2xl)"),
        decl("background", "var(--fandhe-bubble-bg)"),
        decl("color", "var(--fandhe-bubble-fg)"),
        decl("border", "1px solid var(--fandhe-bubble-border)"),
        decl("font-size", "var(--fandhe-font-font-size-sm)"),
        decl("overflow-wrap", "anywhere"),
        // 既定値（`data-variant="solid"` 相当。shadcn `default` = accent
        // 塗りに対応する初期値。`variant` state 節参照）。
        decl("--fandhe-bubble-bg", "var(--fandhe-color-accent)"),
        decl("--fandhe-bubble-fg", "var(--fandhe-color-accent-fg)"),
        decl("--fandhe-bubble-border", "transparent"),
    ];

    let content_base = vec![decl("min-width", "0")];

    let reactions_base = vec![
        decl("display", "flex"),
        decl("flex-wrap", "wrap"),
        decl("gap", "var(--fandhe-space-1)"),
        decl("align-self", "flex-end"),
    ];

    let reaction_base = vec![
        decl("display", "inline-flex"),
        decl("align-items", "center"),
        decl("gap", "var(--fandhe-space-1)"),
        decl("padding", "0 var(--fandhe-space-2)"),
        decl("border-radius", "var(--fandhe-radius-full)"),
        decl("border", "1px solid var(--fandhe-color-border)"),
        decl("background", "var(--fandhe-color-bg)"),
        decl("color", "var(--fandhe-color-fg)"),
        decl("font-size", "var(--fandhe-font-font-size-xs)"),
        decl("line-height", "1.4"),
    ];

    // `collapse-trigger` はボタンリセット + 下線テキストで折りたたみ操作
    // であることを示す（[`crate::breadcrumb`] の `link` と同型のフォーカス
    // リング・hover・transition 構成）。`color: inherit` により solid
    // variant（accent 塗り）上でもコントラストを保つ。
    let collapse_trigger_base = vec![
        decl("display", "inline-flex"),
        decl("align-items", "center"),
        decl("gap", "var(--fandhe-space-1)"),
        decl("padding", "0"),
        decl("border", "0"),
        decl("background", "transparent"),
        decl("color", "inherit"),
        decl("font-size", "var(--fandhe-font-font-size-xs)"),
        decl("cursor", "pointer"),
        decl("text-decoration", "underline"),
        decl("border-radius", "var(--fandhe-radius-sm, 0.25rem)"),
    ];

    let collapse_content_base = vec![
        decl("font-size", "var(--fandhe-font-font-size-xs)"),
        decl("opacity", "1"),
    ];

    SlotRecipe::new("bubble", SLOTS)
        .base("root", root_base)
        .base("content", content_base)
        .base("reactions", reactions_base)
        .base("reaction", reaction_base)
        .base(
            "reaction",
            transition_declarations("background, border-color, color", MotionDuration::Fast),
        )
        .base("collapse-trigger", collapse_trigger_base)
        .base(
            "collapse-trigger",
            transition_declarations("color, opacity", MotionDuration::Fast),
        )
        .base("collapse-content", collapse_content_base)
        .base(
            "collapse-content",
            transition_declarations("opacity", MotionDuration::Fast),
        )
        // `data-variant`（3 形態、モジュール doc「`ColorPalette` 軸を
        // 持たない理由」参照）。`solid` は base の既定値と同じのため
        // 明示規則を golden へ残す（意図の可読性のため）。
        .state(
            "root",
            StateCondition::AttrEq("data-variant", "solid"),
            vec![
                decl("--fandhe-bubble-bg", "var(--fandhe-color-accent)"),
                decl("--fandhe-bubble-fg", "var(--fandhe-color-accent-fg)"),
                decl("--fandhe-bubble-border", "transparent"),
            ],
        )
        .state(
            "root",
            StateCondition::AttrEq("data-variant", "outline"),
            vec![
                decl("--fandhe-bubble-bg", "var(--fandhe-color-bg)"),
                decl("--fandhe-bubble-fg", "var(--fandhe-color-fg)"),
                decl("--fandhe-bubble-border", "var(--fandhe-color-border)"),
            ],
        )
        .state(
            "root",
            StateCondition::AttrEq("data-variant", "plain"),
            vec![
                decl("--fandhe-bubble-bg", "transparent"),
                decl("--fandhe-bubble-fg", "var(--fandhe-color-fg)"),
                decl("--fandhe-bubble-border", "transparent"),
                decl("padding-inline", "0"),
                decl("max-width", "100%"),
            ],
        )
        // `data-align="end"`（右寄せ）。
        .state(
            "root",
            StateCondition::AttrEq("data-align", "end"),
            vec![
                decl("align-self", "flex-end"),
                decl("margin-inline-start", "auto"),
            ],
        )
        // 連続発言の角丸連結（`data-align` × `data-group-position` の
        // 複合条件、モジュール doc「角丸連結（`AttrEqAll` を採用する
        // 理由）」参照）。`single` は base の角丸のまま据え置くため規則を
        // 持たない。
        .state(
            "root",
            StateCondition::AttrEqAll(&[("data-align", "start"), ("data-group-position", "first")]),
            vec![decl("border-end-start-radius", "var(--fandhe-radius-sm)")],
        )
        .state(
            "root",
            StateCondition::AttrEqAll(&[
                ("data-align", "start"),
                ("data-group-position", "middle"),
            ]),
            vec![
                decl("border-start-start-radius", "var(--fandhe-radius-sm)"),
                decl("border-end-start-radius", "var(--fandhe-radius-sm)"),
            ],
        )
        .state(
            "root",
            StateCondition::AttrEqAll(&[("data-align", "start"), ("data-group-position", "last")]),
            vec![decl("border-start-start-radius", "var(--fandhe-radius-sm)")],
        )
        .state(
            "root",
            StateCondition::AttrEqAll(&[("data-align", "end"), ("data-group-position", "first")]),
            vec![decl("border-end-end-radius", "var(--fandhe-radius-sm)")],
        )
        .state(
            "root",
            StateCondition::AttrEqAll(&[("data-align", "end"), ("data-group-position", "middle")]),
            vec![
                decl("border-start-end-radius", "var(--fandhe-radius-sm)"),
                decl("border-end-end-radius", "var(--fandhe-radius-sm)"),
            ],
        )
        .state(
            "root",
            StateCondition::AttrEqAll(&[("data-align", "end"), ("data-group-position", "last")]),
            vec![decl("border-start-end-radius", "var(--fandhe-radius-sm)")],
        )
        // `reaction` の選択状態（モジュール doc「headless の `data-*` を
        // 参照する」節参照）。
        .state(
            "reaction",
            StateCondition::Attr("data-selected"),
            vec![
                decl("background", "var(--fandhe-color-accent-subtle)"),
                decl("border-color", "var(--fandhe-color-accent)"),
                decl("color", "var(--fandhe-color-accent-fg-subtle)"),
            ],
        )
        // `collapse-trigger` のフォーカスリング・hover（[`crate::breadcrumb`]
        // の `link` と同型。`Token`: bubble は `ColorPalette` 軸を持たない
        // 部品。`Outside`: `collapse-trigger` の祖先に `overflow: hidden`
        // を持つ slot がないため）。
        .state(
            "collapse-trigger",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Outside),
        )
        .state(
            "collapse-trigger",
            StateCondition::Hover,
            vec![decl("opacity", "0.8")],
        )
        // `collapse-content` のフェード（モジュール doc「フェードの限界」
        // 参照）。
        .state(
            "collapse-content",
            StateCondition::AttrEq("data-state", "open"),
            vec![decl("opacity", "1")],
        )
        .state(
            "collapse-content",
            StateCondition::AttrEq("data-state", "closed"),
            vec![decl("opacity", "0")],
        )
        .state(
            "collapse-content",
            StateCondition::Attr("hidden"),
            vec![decl("display", "none")],
        )
        // 高さトランジション（イシュー #2192 の共通機構を適用、#2282）。
        // base 2 個目ブロック・`[hidden]` state・`@starting-style` を
        // 一括登録する（モジュール doc「高さトランジション」節参照）。
        .content_height_transition("collapse-content", MotionDuration::Normal)
}

/// この styled Bubble が生成する静的 CSS 全量を返す（決定的。
/// [`crate::message::stylesheet`] と同じ契約）。角丸連結は
/// [`StateCondition::AttrEqAll`] のみで表現できるため raw CSS 追記は
/// 不要（モジュール doc「角丸連結」節参照）。
#[must_use]
pub fn stylesheet() -> String {
    recipe().css()
}

/// styled `root` パーツを組み立てる。見た目クラスは付与せず（モジュール
/// doc「headless の `data-*` を参照する」節参照）、呼び出し側 `class` を
/// [`drop_class_attr`] で除去してから
/// [`fandhe_frontend_headless_ui::bubble::root`] へそのまま委譲する。
#[must_use]
pub fn root<'a>(
    props: BubbleRootProps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    fandhe_frontend_headless_ui::bubble::root(props, drop_class_attr(attrs), children)
}

/// styled `content` パーツを組み立てる。
#[must_use]
pub fn content<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    fandhe_frontend_headless_ui::bubble::content(drop_class_attr(attrs), children)
}

/// styled `reactions` パーツを組み立てる。`label` は headless
/// [`fandhe_frontend_headless_ui::bubble::reactions`] が既定エスケープを
/// 経由して `aria-label` へ出力する（本モジュールは再エスケープしない）。
#[must_use]
pub fn reactions<'a>(label: &'a str, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    fandhe_frontend_headless_ui::bubble::reactions(label, drop_class_attr(attrs), children)
}

/// styled `reaction` パーツを組み立てる。
#[must_use]
pub fn reaction<'a>(selected: bool, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    fandhe_frontend_headless_ui::bubble::reaction(selected, drop_class_attr(attrs), children)
}

/// styled `collapse-trigger` パーツを組み立てる。`controls` は headless
/// [`fandhe_frontend_headless_ui::bubble::collapse_trigger`] が既定
/// エスケープを経由して `aria-controls` へ出力する。
#[must_use]
pub fn collapse_trigger<'a>(
    state: OpenState,
    controls: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    fandhe_frontend_headless_ui::bubble::collapse_trigger(
        state,
        controls,
        drop_class_attr(attrs),
        children,
    )
}

/// styled `collapse-content` パーツを組み立てる。`id` は headless
/// [`fandhe_frontend_headless_ui::bubble::collapse_content`] が既定
/// エスケープを経由して `id` へ出力する。
#[must_use]
pub fn collapse_content<'a>(
    state: OpenState,
    id: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    fandhe_frontend_headless_ui::bubble::collapse_content(
        state,
        id,
        drop_class_attr(attrs),
        children,
    )
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
        assert!(a.contains(r#"[data-scope="bubble"][data-part="root"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let out = stylesheet();
        assert!(!out.contains("</style"));
        assert!(!out.contains('<'));
    }

    #[test]
    fn stylesheet_declares_variant_align_group_position_and_state_rules() {
        let out = stylesheet();
        assert!(out.contains(r#"[data-variant="solid"]"#));
        assert!(out.contains(r#"[data-variant="outline"]"#));
        assert!(out.contains(r#"[data-variant="plain"]"#));
        assert!(out.contains(r#"[data-align="end"]"#));
        assert!(out.contains(r#"[data-align="start"][data-group-position="first"]"#));
        assert!(out.contains(r#"[data-align="start"][data-group-position="middle"]"#));
        assert!(out.contains(r#"[data-align="start"][data-group-position="last"]"#));
        assert!(out.contains(r#"[data-align="end"][data-group-position="first"]"#));
        assert!(out.contains(r#"[data-align="end"][data-group-position="middle"]"#));
        assert!(out.contains(r#"[data-align="end"][data-group-position="last"]"#));
        assert!(out.contains("[data-selected]"));
        assert!(out.contains(r#"[data-state="open"]"#));
        assert!(out.contains(r#"[data-state="closed"]"#));
        assert!(out.contains("[hidden]"));
        // class ベースの variant/align/group-position クラス（
        // `fd-bubble--` 等）は生成しない（モジュール doc参照）。
        assert!(!out.contains("fd-bubble--"));
    }

    #[test]
    fn stylesheet_fades_collapse_content_and_hides_hidden() {
        // イシュー #2282 以降、実際の開閉挙動は下記
        // `collapse_content_declares_height_transition_and_hidden_collapse`
        // が固定する高さトランジションへ移行済み（`opacity` state・
        // `transition-property: opacity` の宣言テキスト自体は golden
        // 純追加原則で残るため PASS するが、`data-state` による離散切替
        // になる。モジュール doc「高さトランジション」節参照）。
        let out = stylesheet();
        assert!(out.contains("opacity: 0;"));
        assert!(out.contains("display: none;"));
        assert!(out.contains("transition-property: opacity"));
    }

    #[test]
    fn collapse_content_declares_height_transition_and_hidden_collapse() {
        // イシュー #2192 の共通機構（`content_height_transition`）を
        // `collapse-content` slot へ適用したことを固定する
        // （[`crate::collapsible`] の
        // `content_declares_height_transition_and_hidden_collapse` と同型。
        // 値の詳細は recipe.rs 側の
        // `content_height_transition_preset_registers_base_state_and_starting_style`
        // が固定するため、本テストは「bubble の collapse-content slot に
        // 実際に適用されている」ことのみを確認する）。
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="bubble"][data-part="collapse-content"][hidden] {"#));
        assert!(css.contains("@starting-style {"));
        assert!(css.contains("transition-behavior: allow-discrete;"));
        assert!(css.contains("height: var(--fandhe-content-height, auto);"));
    }

    #[test]
    fn root_connects_to_headless_bubble_scope() {
        let html = render(&root(BubbleRootProps::default(), vec![], vec![]));
        assert!(html.contains(r#"data-scope="bubble" data-part="root""#));
        assert!(html.starts_with("<div"));
    }

    #[test]
    fn all_parts_connect_to_headless_bubble_scope() {
        let content_html = render(&content(vec![], vec![core_text("hi")]));
        assert!(content_html.contains(r#"data-scope="bubble" data-part="content""#));

        let reactions_html = render(&reactions("3 reactions", vec![], vec![]));
        assert!(reactions_html.contains(r#"data-scope="bubble" data-part="reactions""#));
        assert!(reactions_html.contains(r#"aria-label="3 reactions""#));

        let reaction_html = render(&reaction(true, vec![], vec![]));
        assert!(reaction_html.contains(r#"data-scope="bubble" data-part="reaction""#));
        assert!(reaction_html.contains(r#"data-selected="""#));

        let trigger_html = render(&collapse_trigger(
            OpenState::Closed,
            Some("bubble-collapse-1"),
            vec![],
            vec![],
        ));
        assert!(trigger_html.contains(r#"data-scope="bubble" data-part="collapse-trigger""#));
        assert!(trigger_html.contains(r#"aria-controls="bubble-collapse-1""#));

        let content_panel_html = render(&collapse_content(
            OpenState::Open,
            Some("bubble-collapse-1"),
            vec![],
            vec![core_text("detail")],
        ));
        assert!(content_panel_html.contains(r#"data-scope="bubble" data-part="collapse-content""#));
        assert!(content_panel_html.contains(r#"id="bubble-collapse-1""#));
    }

    #[test]
    fn caller_class_is_dropped_on_every_part() {
        let html = render(&root(
            BubbleRootProps::default(),
            vec![("class", "evil")],
            vec![
                content(vec![("class", "evil")], vec![]),
                reactions(
                    "",
                    vec![("class", "evil")],
                    vec![reaction(false, vec![("class", "evil")], vec![])],
                ),
                collapse_trigger(OpenState::Closed, None, vec![("class", "evil")], vec![]),
                collapse_content(OpenState::Closed, None, vec![("class", "evil")], vec![]),
            ],
        ));
        assert!(!html.contains("evil"));
        assert_eq!(html.matches("class=\"").count(), 0);
    }
}
