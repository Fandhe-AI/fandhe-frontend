//! styled Input Group（イシュー #2063、親 #2061、祖父トラッキング #520。
//! headless 側 anatomy は #2062）。
//!
//! `fandhe_frontend_headless_ui::input_group`（#2062）が出力する
//! `data-scope="input-group"` の `root`/`addon`/`text`/`button` 4 slot へ、
//! shadcn/ui の Input Group（参照スクリーンショット
//! `docs/design/reference-screenshots/shadcn-input-group-{1,2,3}.png`）
//! の意匠（単一の枠線付きコンテナに枠線なし・透明背景の input/textarea を
//! 内包し、淡色 addon をその前後に配置する）を重ねる薄い委譲層である。
//!
//! # 選択的 re-export（`pub use ...::*` を使わない理由）
//!
//! 本モジュールは 4 パーツ（[`root`]/[`addon`]/[`text`]/[`button`]）を
//! すべて同名再定義する（見た目クラスは付与しないが、呼び出し側 `class` の
//! 除去は本モジュールの責務のため）。[`InputGroupAlign`]/[`InputGroupProps`]
//! のみを選択的に再エクスポートする。
//!
//! # 状態機械を持たない理由
//!
//! headless [`fandhe_frontend_headless_ui::input_group`] 自身が「props から
//! 決定的にマークアップを組み立てる純粋関数群」（状態機械なし、headless 側
//! モジュール doc 参照）として実装されているため、本モジュールもその設計を
//! そのまま継承する（[`crate::fieldset`] モジュール doc と同型の判断）。
//!
//! # 責務境界（`docs/policy/intentional-non-adoption.md` §3.25 規則 1）
//!
//! バリデーション処理・addon クリックで input へフォーカスを移す JS 配線
//! （`fandhe-frontend-wasm-full` 側の関心）は実装しない。headless が出力する
//! `data-*` を CSS セレクタとして参照するだけで見た目を切り替える。本モジュール
//! 自身は独自の `data-*` を一切出力しない。
//!
//! # variant 軸: 持たない
//!
//! `size`/`variant`/`color-palette` いずれの軸も提供しない
//! （`docs/design/pre-styled-ui-focus-ring-and-size-conventions.md` §4 (d)
//! 「子の寸法に従属するレイアウト部品」に該当。高さ・文字サイズは内側の
//! [`crate::input`](mod@crate::input)/[`crate::textarea`](mod@crate::textarea) の `size` に従属する）。このため
//! [`crate::visually_hidden`] と同型に、4 パーツとも見た目クラスを一切
//! 付与しない（呼び出し側 `class` は `drop_class_attr` で除去のみ行う）。
//!
//! # レイアウト設計（`flex-wrap` + `order` + `flex: 1 1 0%`）
//!
//! [`root`] は `display: flex; flex-wrap: wrap` のコンテナとする。
//! [`addon`] は `data-align` の値に応じて `order` で DOM 順に依存せず
//! 前後へ配置する（`inline-start`/`inline-end` は同一行、`block-start`/
//! `block-end` は `flex-basis: 100%` で改行させ textarea の上下へ配置する）。
//! 内側コントロール（headless `field::input`/`field::textarea`。下記
//! 「raw CSS 追記の理由」節）は `flex: 1 1 0%` とし、`flex-basis` を
//! **必ず `0%`**（`auto` にしない）にする: [`crate::input`](mod@crate::input)/
//! [`crate::textarea`](mod@crate::textarea) の base は `width: 100%` を持つため、`flex-grow`
//! の basis が `auto`（＝`width` を尊重）に解決されると input だけで
//! 1 行を占有し、inline addon が別行へ折り返されてしまう。
//!
//! # `:focus-within` リングを受容する理由（参照サイトとの意図的な差分）
//!
//! shadcn は `:has(control:focus-visible)` で input 自体のフォーカスに
//! リング表示を限定するが、[`crate::recipe::SlotRecipe`]/raw CSS のいずれも
//! `:has()` の先例を持たないため、本モジュールは [`root`] へ単純な
//! `StateCondition::FocusWithin` を使う。結果として [`addon`] 内の
//! [`button`] へフォーカスした場合も外周リングが点灯するが、[`button`] 自身
//! にも `FocusVisible` で inset リングを付けて外側リングと重ならないように
//! するため、キーボード操作上の弊害はない（意図的に受容、将来 `:has()` の
//! 先例が生まれた場合の再検討点）。
//!
//! # `root` に `disabled_declarations()` を付与しない理由
//!
//! 内側の [`crate::input`](mod@crate::input)/[`crate::textarea`](mod@crate::textarea) が `data-disabled` で自前の
//! `opacity: 0.5` を持つため、`root` にも同じ宣言を重ねると二重に薄くなる
//! （[`crate::fieldset`] の `root` と同じ判断）。`root` の `[data-disabled]`
//! は `cursor: not-allowed` のみ。
//!
//! # `button` の disabled で opacity を持たない理由
//!
//! [`button`] は現状すべての呼び出し箇所（`crates/docs-site` の showcase・
//! primitive-showcase・原稿）で [`addon`] の子として配置される
//! （shadcn/ui の `InputGroupButton` が常に `InputGroupAddon` の子である
//! 構成に倣う）。[`addon`] が `[data-disabled]` で自前の `opacity: 0.5` を
//! 持つため、内側の [`button`] にも同じ `opacity: 0.5` を重ねると
//! `0.5 * 0.5 = 0.25` まで減光されてしまい（addon 内の他パーツ、例えば
//! [`text`] は 0.5 のままで濃さが揃わない）、上記「`root` に
//! `disabled_declarations()` を付与しない理由」と同型の二重適用になる。
//! このため [`button`] の `[data-disabled]` は `cursor: not-allowed` のみ
//! を持ち、減光は外側 [`addon`] の `opacity: 0.5` に一本化する。
//!
//! # raw CSS 追記の理由（[`crate::recipe::SlotRecipe`] が子結合子を
//! 表現できないため）
//!
//! [`SlotRecipe`] はコンポーネント自身の slot にしか宣言を登録できず、
//! 子孫（内側の [`crate::input`](mod@crate::input)/[`crate::textarea`](mod@crate::textarea)）を対象にした宣言を
//! 組めない。このため [`stylesheet`] は [`crate::toggle_group::stylesheet`]/
//! [`crate::number_input::stylesheet`] と同型のパターンで、`recipe().css()`
//! の出力へ [`crate::css::serialize_rule`] を使った素の子結合子（`>`）
//! セレクタを追記する。対象は 2 セレクタ（`root > field::input`/
//! `root > field::textarea`）で、内側コントロールの枠線・角丸・背景を
//! リセットし（`root` 側 1 本の枠線に統一する）、`:focus-visible` の
//! `outline` も無効化する（リングは `root` の `:focus-within` が担う）。
//!
//! 追記した `border: 0` は `[data-scope="field"][data-part="input"]`
//! （[`crate::input`](mod@crate::input) の `variant` クラス経由の宣言、特異度
//! クラス 1 + 属性 2 = (0,3,0)）より高い特異度（`root`/`input` 双方の
//! `data-scope`/`data-part` 属性 4 個 = (0,4,0)）で確実に上書きする。
//! `:focus-visible` の `outline: none` も同様に、[`crate::input`](mod@crate::input) 自身の
//! `:focus-visible` 規則（属性 2 + 擬似 1 = (0,3,0)）より高い特異度
//! （属性 4 + 擬似 1 = (0,5,0)）で上書きする。
//!
//! 内側 input/textarea の `[data-invalid]` に対する `border-color` 上書き
//! は追記しない: 上記 `border: 0`（特異度 (0,4,0)）が [`crate::input`](mod@crate::input)/
//! [`crate::textarea`](mod@crate::textarea) の `[data-invalid] { border-color: ... }`
//! （特異度 (0,3,0)）に常に勝つため、内側コントロールの枠線は幅 0 のまま
//! であり `border-color` の上書きは dead CSS になる（本リポジトリは dead
//! な `[data-invalid]` セレクタを golden で否定する先例を持つ、
//! [`crate::fieldset`] 参照）。invalid の枠線表現は `root` 側 1 本に統一
//! される。
//!
//! # native_select / select を内包する追加リセット（イシュー #3123）
//!
//! 上記 2 セレクタに加え、[`crate::native_select`](mod@crate::native_select)
//! （`root > field::select`）と [`crate::select`](mod@crate::select)
//! （`root > select::root` の子孫 `select::trigger`）を対象にした追加リセット
//! を持つ。Blocks 取り込み対応表 R1035/R1036 が要求する「入力欄の前後へ
//! 通貨・国選択をインライン配置し外枠を 1 本にする」レイアウトのためで、
//! input/textarea と異なり `flex: 1 1 0%` は与えない（内容幅のまま
//! インライン配置する。input と対等に幅を分け合うと select だけで 1 行を
//! 占有しかねないため）。native_select 側は `flex: 0 1 auto; width: auto`
//! で base の `width: 100%` を打ち消す。select 側は `root` に
//! `flex: 0 1 auto` を与えたうえで、枠線・背景のリセットは `trigger`
//! （`control` を挟むため子孫結合子で指定）に対して行う。
//!
//! 受容する既知の上限（将来 opt-in を検討する余地はあるが現時点は対象外）:
//! - select だけを addon と並べる構成では内容幅のまま root 右側に余白が残る
//! - group 内の select `trigger` は特異度で hover 背景変化を失う
//!   （input/button と同じく group 内では hover 背景を持たない扱いとして
//!   受容する）
//! - `positioner`（dropdown 展開中）へのフォーカスでも `root` の
//!   `:focus-within` リングが点灯し続けるが、同リングは addon 内 `button`
//!   でも同様に受容済みであり同型の扱いとする
//!
//! # セキュリティ不変条件
//!
//! - 全出力は headless [`fandhe_frontend_headless_ui::input_group`] →
//!   `fandhe_frontend_core::render` の既定エスケープ（REQ-1）を必ず
//!   経由する。`raw_html()` は使用しない。
//! - 呼び出し側 `class` は `drop_class_attr` で除去してから headless
//!   関数へ委譲する（本モジュールが見た目クラスを付与しないため呼び出し側
//!   の生ペイロードがクラス名合成へ混入する経路自体がないが、`class`
//!   属性の偽装混入は一貫して防ぐ）。
//! - [`stylesheet`] が組み立てる CSS 宣言・selector 断片はすべて
//!   コンパイル時静的リテラルであり、[`crate::css::decl`]/
//!   [`crate::css::serialize_rule`] の `is_valid_value`/`is_valid_identifier`
//!   検証を通る値のみを使う（動的値を混入させない）。

use crate::class_attr::drop_class_attr;
use crate::css::{decl, serialize_rule};
use crate::recipe::{
    disabled_declarations, focus_ring_declarations, hover_bg_muted, hover_surface_declarations,
    transition_declarations, FocusRingColor, FocusRingOffset, MotionDuration, SlotRecipe,
    StateCondition,
};
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;

// headless 型のうち見た目クラスを付与しない本モジュールが必要とするのは
// props/align 列挙のみ（規約 A、`crate::lib` 「headless 再エクスポートの
// 形式規約（イシュー #1062）」節）。パーツ関数 4 件は呼び出し側 `class` の
// 除去を担うため同名再定義する。
pub use fandhe_frontend_headless_ui::input_group::{InputGroupAlign, InputGroupProps};

/// slot 一覧（headless [`fandhe_frontend_headless_ui::input_group`] の
/// anatomy と 1:1、4 パーツ）。
const SLOTS: &[&str] = &["root", "addon", "text", "button"];

/// この styled Input Group の既定 CSS を組み立てる（内部ヘルパ、
/// [`stylesheet`] のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    let mut root_base = vec![
        decl("position", "relative"),
        decl("display", "flex"),
        decl("flex-wrap", "wrap"),
        decl("align-items", "center"),
        decl("width", "100%"),
        decl("min-width", "0"),
        decl("box-sizing", "border-box"),
        decl("color", "var(--fandhe-color-fg)"),
        decl("background", "var(--fandhe-color-bg)"),
        decl("border", "1px solid var(--fandhe-color-border)"),
        decl("border-radius", "var(--fandhe-radius-md)"),
    ];
    root_base.extend(transition_declarations(
        "border-color, background",
        MotionDuration::Fast,
    ));

    let mut button_base = vec![
        decl("display", "inline-flex"),
        decl("align-items", "center"),
        decl("gap", "var(--fandhe-space-1)"),
        decl("box-sizing", "border-box"),
        decl("min-height", "var(--fandhe-size-control-height-xs, 2rem)"),
        decl("padding", "0 var(--fandhe-space-2)"),
        decl("border", "0"),
        decl("border-radius", "var(--fandhe-radius-sm)"),
        decl("background", "transparent"),
        decl("color", "var(--fandhe-color-fg)"),
        decl("font", "inherit"),
        decl("font-size", "var(--fandhe-font-font-size-sm)"),
        decl("cursor", "pointer"),
        hover_bg_muted(),
    ];
    button_base.extend(transition_declarations(
        "background, color",
        MotionDuration::Fast,
    ));

    SlotRecipe::new("input-group", SLOTS)
        .base("root", root_base)
        .base(
            "addon",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("justify-content", "center"),
                decl("gap", "var(--fandhe-space-2)"),
                decl("box-sizing", "border-box"),
                decl("padding-block", "var(--fandhe-space-1)"),
                decl("padding-inline", "var(--fandhe-space-2)"),
                decl("color", "var(--fandhe-color-fg-muted)"),
                decl("font-size", "var(--fandhe-font-font-size-sm)"),
                decl("font-weight", "var(--fandhe-font-font-weight-medium)"),
                decl("line-height", "var(--fandhe-font-line-height-normal)"),
                decl("user-select", "none"),
                decl("cursor", "text"),
            ],
        )
        .base(
            "text",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("gap", "var(--fandhe-space-1)"),
            ],
        )
        .base("button", button_base)
        .state(
            "root",
            StateCondition::FocusWithin,
            focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Outside),
        )
        .state(
            "root",
            StateCondition::Attr("data-invalid"),
            vec![decl("border-color", "var(--fandhe-color-danger)")],
        )
        .state(
            "root",
            StateCondition::Attr("data-disabled"),
            vec![decl("cursor", "not-allowed")],
        )
        .state(
            "addon",
            StateCondition::AttrEq("data-align", "inline-start"),
            vec![decl("order", "-1")],
        )
        .state(
            "addon",
            StateCondition::AttrEq("data-align", "inline-end"),
            vec![decl("order", "1")],
        )
        .state(
            "addon",
            StateCondition::AttrEq("data-align", "block-start"),
            vec![
                decl("order", "-1"),
                decl("flex-basis", "100%"),
                decl("justify-content", "flex-start"),
                decl("padding", "var(--fandhe-space-2) var(--fandhe-space-3) 0"),
            ],
        )
        .state(
            "addon",
            StateCondition::AttrEq("data-align", "block-end"),
            vec![
                decl("order", "1"),
                decl("flex-basis", "100%"),
                decl("justify-content", "flex-start"),
                decl("padding", "0 var(--fandhe-space-3) var(--fandhe-space-2)"),
            ],
        )
        .state(
            "addon",
            StateCondition::Attr("data-disabled"),
            disabled_declarations(),
        )
        .state(
            "button",
            StateCondition::Hover,
            hover_surface_declarations(),
        )
        .state(
            "button",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Inset),
        )
        .state(
            "button",
            StateCondition::Attr("data-disabled"),
            // `opacity: 0.5` を持たない理由は `button` に対する
            // モジュール doc「`button` の disabled で opacity を持たない
            // 理由」節を参照。ここでは `cursor: not-allowed` のみを適用する。
            vec![decl("cursor", "not-allowed")],
        )
}

/// この styled Input Group が生成する静的 CSS 全量を返す（決定的。
/// [`crate::fieldset::css`] と同じ契約）。子孫（[`crate::input`](mod@crate::input)/
/// [`crate::textarea`](mod@crate::textarea)/[`crate::native_select`](mod@crate::native_select)/
/// [`crate::select`](mod@crate::select)）を対象にした raw CSS 追記を含む
/// （モジュール doc「raw CSS 追記の理由」節・「native_select / select を
/// 内包する追加リセット」節参照）。
#[must_use]
pub fn stylesheet() -> String {
    let mut out = recipe().css();

    const ROOT: &str = r#"[data-scope="input-group"][data-part="root"]"#;
    const INPUT: &str = r#"[data-scope="field"][data-part="input"]"#;
    const TEXTAREA: &str = r#"[data-scope="field"][data-part="textarea"]"#;

    let mut push_rule = |selector: &str, decls: &[crate::css::Declaration]| {
        if let Some(rule) = serialize_rule(selector, decls) {
            if !out.is_empty() {
                out.push('\n');
            }
            out.push_str(&rule);
        }
    };

    for control in [INPUT, TEXTAREA] {
        push_rule(
            &format!("{ROOT} > {control}"),
            &[
                decl("flex", "1 1 0%"),
                decl("min-width", "0"),
                decl("border", "0"),
                decl("border-radius", "0"),
                decl("background", "transparent"),
                decl("box-shadow", "none"),
            ],
        );
        push_rule(
            &format!("{ROOT} > {control}:focus-visible"),
            &[decl("outline", "none")],
        );
    }

    // native_select（`data-scope="field" data-part="select"`）は input と
    // 異なりインライン幅（内容幅）のまま並べる。モジュール doc「native_select
    // / select を内包する追加リセット」節参照。
    const NATIVE_SELECT: &str = r#"[data-scope="field"][data-part="select"]"#;
    push_rule(
        &format!("{ROOT} > {NATIVE_SELECT}"),
        &[
            decl("flex", "0 1 auto"),
            decl("width", "auto"),
            decl("min-width", "0"),
            decl("border", "0"),
            decl("border-radius", "0"),
            decl("background", "transparent"),
            decl("box-shadow", "none"),
        ],
    );
    push_rule(
        &format!("{ROOT} > {NATIVE_SELECT}:focus-visible"),
        &[decl("outline", "none")],
    );

    // styled select（`data-scope="select"`）は `root` にインライン幅の
    // flex を与え、枠線・背景は `control` を挟んだ子孫の `trigger` へ
    // リセットを当てる（同節参照）。
    const SELECT_ROOT: &str = r#"[data-scope="select"][data-part="root"]"#;
    const SELECT_TRIGGER: &str = r#"[data-scope="select"][data-part="trigger"]"#;
    push_rule(
        &format!("{ROOT} > {SELECT_ROOT}"),
        &[decl("flex", "0 1 auto"), decl("min-width", "0")],
    );
    push_rule(
        &format!("{ROOT} > {SELECT_ROOT} {SELECT_TRIGGER}"),
        &[
            decl("border", "0"),
            decl("border-radius", "0"),
            decl("background", "transparent"),
            decl("box-shadow", "none"),
        ],
    );
    push_rule(
        &format!("{ROOT} > {SELECT_ROOT} {SELECT_TRIGGER}:focus-visible"),
        &[decl("outline", "none")],
    );

    out
}

/// styled `root` パーツを組み立てる。見た目クラスは付与せず（モジュール doc
/// 「variant 軸: 持たない」節参照）、呼び出し側 `class` を `drop_class_attr`
/// で除去してから [`fandhe_frontend_headless_ui::input_group::root`] へ
/// そのまま委譲する。
#[must_use]
pub fn root<'a>(
    props: &InputGroupProps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    fandhe_frontend_headless_ui::input_group::root(props, drop_class_attr(attrs), children)
}

/// styled `addon` パーツを組み立てる。[`root`] と同じく見た目クラスを
/// 付与しない。
#[must_use]
pub fn addon<'a>(
    align: InputGroupAlign,
    props: &InputGroupProps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    fandhe_frontend_headless_ui::input_group::addon(align, props, drop_class_attr(attrs), children)
}

/// styled `text` パーツを組み立てる。[`root`] と同じく見た目クラスを
/// 付与しない。
#[must_use]
pub fn text<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    fandhe_frontend_headless_ui::input_group::text(drop_class_attr(attrs), children)
}

/// styled `button` パーツを組み立てる。[`root`] と同じく見た目クラスを
/// 付与しない。
#[must_use]
pub fn button<'a>(
    props: &InputGroupProps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    fandhe_frontend_headless_ui::input_group::button(props, drop_class_attr(attrs), children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text as core_text};

    fn base_props() -> InputGroupProps {
        InputGroupProps {
            disabled: false,
            invalid: false,
        }
    }

    #[test]
    fn stylesheet_is_deterministic_and_targets_data_scope_selectors() {
        let a = stylesheet();
        let b = stylesheet();
        assert_eq!(a, b);
        assert!(a.contains(r#"[data-scope="input-group"][data-part="root"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let out = stylesheet();
        assert!(!out.contains("</style"));
        assert!(!out.contains('<'));
    }

    #[test]
    fn root_connects_to_headless_input_group_scope() {
        let props = base_props();
        let html = render(&root(&props, vec![], vec![]));
        assert!(html.contains(r#"data-scope="input-group" data-part="root""#));
    }

    #[test]
    fn addon_text_button_connect_to_headless_input_group_scope() {
        let props = base_props();
        let addon_html = render(&addon(InputGroupAlign::InlineStart, &props, vec![], vec![]));
        assert!(addon_html.contains(r#"data-scope="input-group" data-part="addon""#));

        let text_html = render(&text(vec![], vec![core_text("$")]));
        assert!(text_html.contains(r#"data-scope="input-group" data-part="text""#));

        let button_html = render(&button(&props, vec![], vec![core_text("Clear")]));
        assert!(button_html.contains(r#"data-scope="input-group" data-part="button""#));
    }

    #[test]
    fn caller_class_is_dropped_on_every_part() {
        let props = base_props();
        let html = render(&root(
            &props,
            vec![("class", "evil")],
            vec![
                addon(
                    InputGroupAlign::InlineStart,
                    &props,
                    vec![("class", "evil")],
                    vec![text(vec![("class", "evil")], vec![core_text("$")])],
                ),
                button(&props, vec![("class", "evil")], vec![core_text("Clear")]),
            ],
        ));
        assert!(!html.contains("evil"));
        assert_eq!(html.matches("class=\"").count(), 0);
    }

    #[test]
    fn root_disabled_state_has_no_opacity_declaration() {
        let out = stylesheet();
        let root_disabled_rule = r#"[data-scope="input-group"][data-part="root"][data-disabled] {
  cursor: not-allowed;
}
"#;
        assert!(out.contains(root_disabled_rule));
        assert!(!out.contains(
            r#"[data-scope="input-group"][data-part="root"][data-disabled] {
  opacity: 0.5;"#
        ));
    }

    #[test]
    fn stylesheet_contains_all_four_align_state_rules() {
        let out = stylesheet();
        for align in ["inline-start", "inline-end", "block-start", "block-end"] {
            assert!(out.contains(&format!(
                r#"[data-scope="input-group"][data-part="addon"][data-align="{align}"]"#
            )));
        }
    }

    #[test]
    fn stylesheet_appends_inner_control_reset_rules() {
        let out = stylesheet();
        assert!(out.contains(
            r#"[data-scope="input-group"][data-part="root"] > [data-scope="field"][data-part="input"]"#
        ));
        assert!(out.contains(
            r#"[data-scope="input-group"][data-part="root"] > [data-scope="field"][data-part="textarea"]"#
        ));
        assert!(out.contains(":focus-visible"));
        assert!(out.contains("outline: none;"));
    }

    /// native_select（`data-scope="field" data-part="select"`）/ styled
    /// select（`data-scope="select"`）を内包するための追加リセット
    /// （イシュー #3123）が新セレクタとして追記され、`:focus-visible` の
    /// outline 無効化も効くことを固定する。
    #[test]
    fn stylesheet_appends_native_select_and_select_reset_rules() {
        let out = stylesheet();
        assert!(out.contains(
            r#"[data-scope="input-group"][data-part="root"] > [data-scope="field"][data-part="select"]"#
        ));
        assert!(out.contains(
            r#"[data-scope="input-group"][data-part="root"] > [data-scope="select"][data-part="root"]"#
        ));
        assert!(out.contains(
            r#"[data-scope="input-group"][data-part="root"] > [data-scope="select"][data-part="root"] [data-scope="select"][data-part="trigger"]"#
        ));
        assert!(out.matches("outline: none;").count() >= 3);
    }
}
