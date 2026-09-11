//! styled Questionnaire（headless ラッパー、イシュー #2119、親 #2116、
//! 祖父トラッキング #2057、参照軸 #2001）。
//!
//! [`fandhe_frontend_headless_ui::questionnaire`]（イシュー #2117）の
//! Root / Progress / Question / Prompt / Description / Options / Freeform /
//! Actions / Back / Next / Skip の 11 anatomy パーツへ、質問カード・進捗
//! ゲージ・選択肢の choice card 風表示・completed/upcoming の視覚差を
//! 重ねる薄い委譲層である（[`crate::steps`]/[`crate::marker`] と同型の
//! 構成）。`fandhe-frontend-wasm-full` の back/next/skip click → dispatch
//! 配線はイシュー #2118 で完了済みだが、本モジュールはその配線の変更を
//! 一切伴わない（静的 CSS・SSR マークアップの追加のみ）。
//!
//! # 全パーツが `state: &Questionnaire` を取る理由（headless 層に自由関数
//! がない）
//!
//! [`fandhe_frontend_headless_ui::questionnaire`] は
//! [`crate::steps`]（[`fandhe_frontend_headless_ui::steps::Steps`]）と同型に
//! 自由関数を一切持たず、すべて
//! [`fandhe_frontend_headless_ui::questionnaire::Questionnaire`] の
//! inherent メソッドとして提供される（`data-state`
//! （active/completed/upcoming）の判定に `count`/`step` の参照が毎回
//! 必要なため）。本モジュールも同型で、すべての styled パーツ関数が
//! `state: &Questionnaire` を受け取り、内部で `state.<part>(...)` へ委譲
//! する。
//!
//! `Questionnaire` 状態機械自体は再エクスポートしない（[`crate::steps`]
//! の `Steps` 非再エクスポートと同じ理由。呼び出し側 `state.root(...)`
//! 直呼びによる未スタイル描画事故を誘発するため）。状態管理・hydration が
//! 必要な呼び出し側は
//! [`fandhe_frontend_headless_ui::questionnaire::Questionnaire`] を直接
//! import し、実際の描画は本モジュールの styled パーツ関数を組み合わせて
//! 構築すること。[`QuestionProps`]/[`QuestionnaireAction`] の 2 型のみを
//! 選択的に再エクスポートする（[`crate::steps`] の `StepsAction` 再
//! エクスポートと同型）。
//!
//! # class 軸を持たない理由（`data-*` を参照するのみ）
//!
//! [`crate::marker`]/[`crate::attachment`] と同型の判断
//! （`docs/design/pre-styled-ui-data-attr-vocabulary.md` §2.2「役割 B:
//! 参照のみ」）。`size`/`colorPalette` 等の見た目クラス軸は持たず、全 11
//! パーツが headless の出力する `data-*`（`data-state`/`data-answered`/
//! `data-skipped`/`data-required`/`data-invalid`/`data-disabled`/
//! `data-complete`）を [`StateCondition::Attr`]/[`StateCondition::AttrEq`]
//! で参照するのみで見た目を切り替える。各パーツは呼び出し側 `class` を
//! `drop_class_attr` で除去してから委譲する（class 属性を自前で付与
//! しない）。
//!
//! # `question` slot の `hidden` 属性と base `display` の関係
//!
//! headless `question` は非 active（completed/upcoming）に必ず `hidden`
//! 属性を付与する（`crates/headless-ui/src/questionnaire.rs` 参照）。
//! `question` base は `display: flex` を持つため、`[hidden]{display:none}`
//! という UA 既定スタイルは `display: flex` に **上書きされてしまう**
//! （詳細度は属性セレクタと同等だが、後勝ちの CSS カスケードにより base
//! 規則が UA スタイルを上書きする）。そのため
//! `.state("question", StateCondition::Attr("hidden"), [display: none])`
//! を明示的に追加する（[`crate::tour`]/[`crate::dialog`] の positioner
//! 等、`display` を持つ base への `[hidden]` 明示規則と同型の対策）。
//!
//! # completed / upcoming の可視化契約（正直な契約）
//!
//! 上記のとおり非 active な question は常に `hidden` であるため、
//! `[data-state="completed"]`/`[data-state="upcoming"]` の state 規則
//! （枠色・破線表現等）が実際に可視化されるのは、**アプリケーション側が
//! 独自 CSS で `[hidden]` を打ち消して一覧表示する**（回答レビュー画面等）
//! 場合に限られる。既定表示は active な質問 1 件のみである。
//! `[data-state="active"]` は base と同値（既定表示）のため state 規則を
//! 書かない（[`crate::marker`] の「`neutral` = 既定、state 規則を書かない」
//! と同型の判断）。
//!
//! # progress の塗りと wasm-full 連携の制約
//!
//! headless `progress` は中身空の `div role="progressbar"` を返す。本
//! モジュールの [`progress`] は **styled [`crate::progress`] を入れ子に
//! しない**（`role="progressbar"` の二重化を避けるため。a11y 不整合
//! 対策）。代わりに [`Questionnaire::step`]/[`Questionnaire::count`] から
//! headless `progress` と同じ u128 拡張計算で百分率を求め、
//! `style="--fandhe-questionnaire-percent: N%"` を呼び出し側 `style` を
//! 除去したうえで合成する（[`crate::progress::range`] の `percent_style`/
//! `drop_style_attr` と同型のパターン）。CSS 側は `recipe` の
//! `linear-gradient` で塗り幅を表現する。
//!
//! **既知の制約**: `fandhe-frontend-wasm-full`（イシュー #2118）はクライ
//! アント側遷移で `aria-valuenow`/`aria-valuetext` を更新するが、本
//! custom property は更新しない。そのためステップ遷移直後はアプリ側の
//! 再描画（headless `Questionnaire` rustdoc が既に契約化している）まで
//! ゲージが古い値を示す（out-of-scope、後続イシューでの拡張余地として
//! 記録する）。
//!
//! # 選択肢の choice card 風表示（子孫セレクタ + 併用必須の注意）
//!
//! [`SlotRecipe`] は子孫セレクタを表現できないため、[`crate::marker`] の
//! [`stylesheet`] と同型に [`crate::css::serialize_rule`] で raw CSS を
//! [`stylesheet`] へ追記する。対象は `options` slot 配下の
//! `[data-scope="radio-group"][data-part="item"]`/
//! `[data-scope="checkbox-group"][data-part="item"]` の checked/disabled
//! 状態（[`crate::radio_card`] の item base に倣ったカード状の枠・パディ
//! ング）。**この規則を実効化するには、呼び出し側が [`options`] スロット
//! へ styled [`crate::radio_group`]/[`crate::checkbox_group`] の item を
//! 入れ子にし、かつ [`crate::radio_group::stylesheet`]/
//! [`crate::checkbox_group::stylesheet`] を併せて読み込む必要がある**
//! （本 [`stylesheet`] は item の基本規則を含まない。[`crate::marker`] の
//! 「`stylesheet` が separator の基本 CSS を含まない理由」節と同型の判断）。
//!
//! # 責務境界（`.claude/rules/coding-rust.md` §UI 部品の責務境界、
//! `docs/policy/intentional-non-adoption.md` §3.25）
//!
//! headless [`fandhe_frontend_headless_ui::questionnaire`] モジュール doc
//! 「責務境界」節をそのまま継承する。回答値の保持・検証（必須判定）・
//! 分岐・送信はアプリケーション責務であり、本モジュールは持たない。
//!
//! # セキュリティ不変条件
//!
//! - 全出力は headless `Questionnaire::<part>` →
//!   `fandhe_frontend_core::render` の既定エスケープ（REQ-1）を必ず
//!   経由する。`raw_html()` は使用せず、HTML 文字列を直接組み立てない。
//! - 呼び出し側 `class` は全 11 パーツで `drop_class_attr` により除去
//!   する。[`progress`] は加えて呼び出し側 `style` を除去し、自前の
//!   `style` 値は正規化済み `usize`（`0..=100`）の百分率のみから組み立て
//!   る（呼び出し側の動的値を `style` へ流さない）。
//! - CSS 宣言値はすべてコンパイル時静的リテラルで、[`crate::css::decl`]/
//!   [`crate::css::serialize_rule`] の検証を通る値のみ（raw CSS 追記部分を
//!   含む）。[`stylesheet`] は `<` を含まない（`</style` 断片が構成不能、
//!   [`crate::stylesheet::StyleSheet::push_css`] の fail-closed 検証を
//!   通る）。
//!
//! # スコープ外
//!
//! - `examples/headless-pre-styled-ui` への questionnaire 追加
//!   （crates.io バージョン依存のため未公開版を参照できない、
//!   [`crate::marker`] と同型の判断）。
//! - `fandhe-frontend-headless-ui`/`fandhe-frontend-wasm-full` の変更
//!   （`--fandhe-questionnaire-percent` の wasm-full 側更新を含む、上記
//!   「progress の塗りと wasm-full 連携の制約」節参照）。
//! - `root` の `data-orientation` を参照した横/縦のレイアウト差の実装
//!   （headless は hydration ラウンドトリップ用に出力するが、本 recipe は
//!   意図的に参照しない。`root` は常に縦積み）。
//! - `size`/`colorPalette` 軸の追加、[`SlotRecipe`] への子孫セレクタ DSL
//!   拡張。

use crate::class_attr::drop_class_attr;
use crate::css::{decl, serialize_rule};
use crate::recipe::{
    disabled_declarations, focus_ring_declarations, hover_bg_muted, hover_bg_solid_with_fallback,
    hover_surface_declarations, transition_declarations, FocusRingColor, FocusRingOffset,
    MotionDuration, SlotRecipe, StateCondition,
};
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
use fandhe_frontend_headless_ui::questionnaire::Questionnaire;
pub use fandhe_frontend_headless_ui::questionnaire::{QuestionProps, QuestionnaireAction};

/// slot 一覧（headless [`fandhe_frontend_headless_ui::questionnaire`] の
/// anatomy と 1:1、11 パーツ）。
const SLOTS: &[&str] = &[
    "root",
    "progress",
    "question",
    "prompt",
    "description",
    "options",
    "freeform",
    "actions",
    "back",
    "next",
    "skip",
];

/// この styled Questionnaire の既定 CSS を組み立てる（内部ヘルパ、
/// [`stylesheet`] のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    let trigger_base = vec![
        decl("display", "inline-flex"),
        decl("align-items", "center"),
        decl("justify-content", "center"),
        decl("height", "var(--fandhe-size-control-height-md, 2.25rem)"),
        decl("padding", "0 var(--fandhe-space-4)"),
        decl("border-radius", "var(--fandhe-radius-md)"),
        decl("border", "1px solid transparent"),
        decl("font-weight", "var(--fandhe-font-font-weight-medium)"),
        decl("cursor", "pointer"),
    ];

    SlotRecipe::new("questionnaire", SLOTS)
        .base(
            "root",
            vec![
                decl("display", "flex"),
                decl("flex-direction", "column"),
                decl("gap", "var(--fandhe-space-4)"),
            ],
        )
        // 進捗トラック。塗り幅は `--fandhe-questionnaire-percent`（呼び出し
        // 側が [`progress`] 経由で `style` へ設定する）を参照する
        // `linear-gradient` で表現する（モジュール doc「progress の塗り」
        // 節参照）。
        .base(
            "progress",
            vec![
                decl("display", "block"),
                decl(
                    "height",
                    "var(--fandhe-questionnaire-progress-height, 0.375rem)",
                ),
                decl("border-radius", "var(--fandhe-radius-full, 999px)"),
                decl(
                    "background",
                    "linear-gradient(to right, var(--fandhe-color-accent) var(--fandhe-questionnaire-percent, 0%), var(--fandhe-color-bg-muted) 0)",
                ),
            ],
        )
        .state(
            "progress",
            StateCondition::Attr("data-complete"),
            vec![decl(
                "background",
                "linear-gradient(to right, var(--fandhe-color-success) var(--fandhe-questionnaire-percent, 0%), var(--fandhe-color-bg-muted) 0)",
            )],
        )
        // 質問カード。
        .base(
            "question",
            vec![
                decl("display", "flex"),
                decl("flex-direction", "column"),
                decl("gap", "var(--fandhe-space-3)"),
                decl("margin", "0"),
                decl("padding", "var(--fandhe-space-4)"),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "var(--fandhe-radius-lg)"),
                decl("background", "var(--fandhe-color-bg)"),
                decl("min-width", "0"),
            ],
        )
        // `[data-state="active"]` は base と同値のため state 規則を書かない
        // （モジュール doc「completed / upcoming の可視化契約」節参照）。
        // `[hidden]{display:none}` の UA 既定を `question` base の
        // `display: flex` が上書きするため、明示的に打ち消す
        // （モジュール doc「`question` slot の `hidden` 属性」節参照）。
        .state(
            "question",
            StateCondition::Attr("hidden"),
            vec![decl("display", "none")],
        )
        .state(
            "question",
            StateCondition::AttrEq("data-state", "completed"),
            vec![
                decl("border-color", "var(--fandhe-color-success)"),
                decl("background", "var(--fandhe-color-success-subtle)"),
            ],
        )
        .state(
            "question",
            StateCondition::AttrEq("data-state", "upcoming"),
            vec![
                decl("border-style", "dashed"),
                decl("color", "var(--fandhe-color-fg-muted)"),
            ],
        )
        .state(
            "question",
            StateCondition::Attr("data-answered"),
            vec![decl(
                "box-shadow",
                "inset 3px 0 0 var(--fandhe-color-accent)",
            )],
        )
        .state(
            "question",
            StateCondition::Attr("data-skipped"),
            vec![decl("opacity", "0.7"), decl("font-style", "italic")],
        )
        .state(
            "question",
            StateCondition::Attr("data-invalid"),
            vec![decl("border-color", "var(--fandhe-color-danger)")],
        )
        .base(
            "prompt",
            vec![
                decl("padding", "0"),
                decl("font-size", "var(--fandhe-font-font-size-md)"),
                decl("font-weight", "var(--fandhe-font-font-weight-semibold)"),
                decl("color", "var(--fandhe-color-fg)"),
            ],
        )
        .base(
            "description",
            vec![
                decl("font-size", "var(--fandhe-font-font-size-sm)"),
                decl("color", "var(--fandhe-color-fg-muted)"),
            ],
        )
        .base(
            "options",
            vec![decl("display", "grid"), decl("gap", "var(--fandhe-space-2)")],
        )
        .base(
            "freeform",
            vec![
                decl("display", "flex"),
                decl("flex-direction", "column"),
                decl("gap", "var(--fandhe-space-2)"),
            ],
        )
        .base(
            "actions",
            vec![
                decl("display", "flex"),
                decl("flex-wrap", "wrap"),
                decl("align-items", "center"),
                decl("gap", "var(--fandhe-space-2)"),
                decl("justify-content", "flex-end"),
            ],
        )
        .base("back", {
            let mut v = trigger_base.clone();
            v.push(decl("background", "transparent"));
            v.push(decl("border-color", "var(--fandhe-color-border)"));
            v.push(decl("color", "var(--fandhe-color-fg)"));
            v.extend(transition_declarations(
                "background, border-color, color",
                MotionDuration::Fast,
            ));
            v
        })
        .state(
            "back",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Outside),
        )
        .state("back", StateCondition::Hover, {
            let mut v = hover_surface_declarations();
            v.push(hover_bg_muted());
            v
        })
        .state(
            "back",
            StateCondition::Attr("disabled"),
            disabled_declarations(),
        )
        .state(
            "back",
            StateCondition::Attr("data-disabled"),
            disabled_declarations(),
        )
        .base("next", {
            let mut v = trigger_base.clone();
            v.push(decl("background", "var(--fandhe-color-accent)"));
            v.push(decl("border-color", "var(--fandhe-color-accent)"));
            v.push(decl("color", "var(--fandhe-color-accent-fg)"));
            v.extend(transition_declarations(
                "background, border-color, color",
                MotionDuration::Fast,
            ));
            v
        })
        .state(
            "next",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Outside),
        )
        .state("next", StateCondition::Hover, {
            let mut v = hover_surface_declarations();
            v.push(hover_bg_solid_with_fallback());
            v
        })
        .state(
            "next",
            StateCondition::Attr("disabled"),
            disabled_declarations(),
        )
        .state(
            "next",
            StateCondition::Attr("data-disabled"),
            disabled_declarations(),
        )
        .base("skip", {
            let mut v = trigger_base;
            v.push(decl("background", "transparent"));
            v.push(decl("border-color", "transparent"));
            v.push(decl("color", "var(--fandhe-color-fg-muted)"));
            v.extend(transition_declarations(
                "background, border-color, color",
                MotionDuration::Fast,
            ));
            v
        })
        .state(
            "skip",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Outside),
        )
        .state("skip", StateCondition::Hover, {
            let mut v = hover_surface_declarations();
            v.push(hover_bg_muted());
            v
        })
        .state(
            "skip",
            StateCondition::Attr("disabled"),
            disabled_declarations(),
        )
        .state(
            "skip",
            StateCondition::Attr("data-disabled"),
            disabled_declarations(),
        )
}

/// この styled Questionnaire が生成する静的 CSS 全量を返す（決定的。
/// [`crate::marker::stylesheet`] と同じ契約）。`options` slot 配下へ入れ子
/// にした [`crate::radio_group`]/[`crate::checkbox_group`] の item を
/// カード状に整形する子孫セレクタを追記する（モジュール doc「選択肢の
/// choice card 風表示」節参照。item 自体の基本規則は含まないため、
/// [`crate::radio_group::stylesheet`]/[`crate::checkbox_group::stylesheet`]
/// の併用が必須）。
#[must_use]
pub fn stylesheet() -> String {
    let mut out = recipe().css();

    const OPTIONS: &str = r#"[data-scope="questionnaire"][data-part="options"]"#;
    const RADIO_ITEM: &str = r#"[data-scope="radio-group"][data-part="item"]"#;
    const CHECKBOX_ITEM: &str = r#"[data-scope="checkbox-group"][data-part="item"]"#;

    let append = |rule: Option<String>, out: &mut String| {
        if let Some(css) = rule {
            if !out.is_empty() {
                out.push('\n');
            }
            out.push_str(&css);
        }
    };

    let card_base = vec![
        decl("display", "flex"),
        decl("align-items", "flex-start"),
        decl("gap", "var(--fandhe-space-2)"),
        decl("padding", "var(--fandhe-space-3)"),
        decl("border", "1px solid var(--fandhe-color-border)"),
        decl("border-radius", "var(--fandhe-radius-lg)"),
        decl("background", "var(--fandhe-color-bg)"),
        decl("cursor", "pointer"),
    ];
    let card_checked = vec![
        decl("border-color", "var(--fandhe-color-accent)"),
        decl("background", "var(--fandhe-color-accent-subtle)"),
    ];

    for item in [RADIO_ITEM, CHECKBOX_ITEM] {
        let selector = format!("{OPTIONS} {item}");
        append(serialize_rule(&selector, &card_base), &mut out);

        let checked_selector = format!("{OPTIONS} {item}[data-state=\"checked\"]");
        append(serialize_rule(&checked_selector, &card_checked), &mut out);

        let disabled_selector = format!("{OPTIONS} {item}[data-disabled]");
        append(
            serialize_rule(&disabled_selector, &disabled_declarations()),
            &mut out,
        );
    }

    out
}

/// `attrs` から `style`（ASCII 大文字小文字を無視）を除いた列を返す。
///
/// [`progress`] が `--fandhe-questionnaire-percent` を含む `style` を組み
/// 立てた後、呼び出し側 `attrs` を連結する前に使う dedup ヘルパ
/// （`crate::progress::drop_style_attr` と同型の判断。重複 `style` 属性に
/// よる後勝ちの非決定的なスタイル適用を防ぐ）。
fn drop_style_attr<'a>(attrs: Vec<(&'a str, &'a str)>) -> Vec<(&'a str, &'a str)> {
    attrs
        .into_iter()
        .filter(|(k, _)| !k.eq_ignore_ascii_case("style"))
        .collect()
}

/// `--fandhe-questionnaire-percent` custom property を設定する `style`
/// 属性値を組み立てる（動的値は `percent`（`0..=100` に正規化済み
/// `usize`）由来の 1 点のみ）。
fn percent_style(percent: usize) -> String {
    format!("--fandhe-questionnaire-percent: {percent}%")
}

/// styled root パーツ。呼び出し側 `class` を除去してから
/// [`Questionnaire::root`] へ委譲する。
#[must_use]
pub fn root<'a>(
    state: &Questionnaire,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    state.root(drop_class_attr(attrs), children)
}

/// styled progress パーツ。`state.step()`/`state.count()` から
/// headless `progress` と同じ u128 拡張計算で百分率を求め、
/// `--fandhe-questionnaire-percent` を含む `style` を組み立てて委譲する
/// （モジュール doc「progress の塗りと wasm-full 連携の制約」節参照。
/// styled [`crate::progress`] は入れ子にしない）。呼び出し側 `class`/
/// `style` はいずれも除去する。
#[must_use]
pub fn progress<'a>(
    state: &Questionnaire,
    label: &'a str,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let percent = (state.step() as u128 * 100 / state.count() as u128) as usize;
    let style = percent_style(percent);
    let mut merged: Vec<(&str, &str)> = vec![("style", style.as_str())];
    merged.extend(drop_style_attr(drop_class_attr(attrs)));
    state.progress(label, merged, children)
}

/// styled question パーツ。実体は [`Questionnaire::question`] へそのまま
/// 委譲する（呼び出し側 `class` は除去する）。
#[must_use]
pub fn question<'a>(
    state: &Questionnaire,
    index: usize,
    props: QuestionProps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    state.question(index, props, drop_class_attr(attrs), children)
}

/// styled prompt パーツ。実体は [`Questionnaire::prompt`] へそのまま委譲
/// する（呼び出し側 `class` は除去する）。
#[must_use]
pub fn prompt<'a>(
    state: &Questionnaire,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    state.prompt(drop_class_attr(attrs), children)
}

/// styled description パーツ。実体は [`Questionnaire::description`] へ
/// そのまま委譲する（呼び出し側 `class` は除去する）。
#[must_use]
pub fn description<'a>(
    state: &Questionnaire,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    state.description(drop_class_attr(attrs), children)
}

/// styled options パーツ（純スロット）。呼び出し側が styled
/// [`crate::radio_group`]/[`crate::checkbox_group`] の item を入れ子にする
/// 契約（モジュール doc「選択肢の choice card 風表示」節参照。呼び出し側
/// `class` は除去する）。
#[must_use]
pub fn options<'a>(
    state: &Questionnaire,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    state.options(drop_class_attr(attrs), children)
}

/// styled freeform パーツ（純スロット）。呼び出し側が styled
/// [`crate::field::label`]/[`crate::textarea::textarea`] を入れ子にする
/// 契約（呼び出し側 `class` は除去する）。
#[must_use]
pub fn freeform<'a>(
    state: &Questionnaire,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    state.freeform(drop_class_attr(attrs), children)
}

/// styled actions パーツ。実体は [`Questionnaire::actions`] へそのまま
/// 委譲する（呼び出し側 `class` は除去する）。
#[must_use]
pub fn actions<'a>(
    state: &Questionnaire,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    state.actions(drop_class_attr(attrs), children)
}

/// styled back パーツ。実体は [`Questionnaire::back`] へそのまま委譲する
/// （呼び出し側 `class` は除去する）。
#[must_use]
pub fn back<'a>(
    state: &Questionnaire,
    disabled: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    state.back(disabled, drop_class_attr(attrs), children)
}

/// styled next パーツ。実体は [`Questionnaire::next`] へそのまま委譲する
/// （呼び出し側 `class` は除去する）。
#[must_use]
pub fn next<'a>(
    state: &Questionnaire,
    disabled: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    state.next(disabled, drop_class_attr(attrs), children)
}

/// styled skip パーツ。実体は [`Questionnaire::skip`] へそのまま委譲する
/// （呼び出し側 `class` は除去する）。
#[must_use]
pub fn skip<'a>(
    state: &Questionnaire,
    disabled: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    state.skip(disabled, drop_class_attr(attrs), children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};
    use fandhe_frontend_headless_ui::Orientation;

    fn q(count: usize, step: usize) -> Questionnaire {
        Questionnaire::new(count, step, Orientation::Horizontal)
    }

    // --- stylesheet 決定性・脱出不能・全 slot 出現 ---

    #[test]
    fn stylesheet_is_deterministic() {
        assert_eq!(stylesheet(), stylesheet());
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    #[test]
    fn stylesheet_covers_all_slots() {
        let css = stylesheet();
        for slot in SLOTS {
            let selector = format!(r#"[data-scope="questionnaire"][data-part="{slot}"]"#);
            assert!(css.contains(&selector), "missing selector for slot {slot}");
        }
    }

    #[test]
    fn stylesheet_declares_completed_and_upcoming_but_not_active() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-state="completed"]"#));
        assert!(css.contains(r#"[data-state="upcoming"]"#));
        assert!(!css.contains(r#"[data-state="active"]"#));
    }

    #[test]
    fn stylesheet_declares_hidden_display_none_for_question() {
        let css = stylesheet();
        assert!(css.contains("[hidden]"));
        assert!(css.contains("display: none"));
    }

    #[test]
    fn stylesheet_declares_descendant_choice_card_rules() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="questionnaire"][data-part="options"] [data-scope="radio-group"][data-part="item"]"#));
        assert!(css.contains(r#"[data-scope="questionnaire"][data-part="options"] [data-scope="checkbox-group"][data-part="item"]"#));
    }

    #[test]
    fn stylesheet_declares_no_class_axis() {
        assert!(!stylesheet().contains("fd-questionnaire--"));
    }

    // --- パーツ関数: scope/part・class 除去・委譲確認 ---

    #[test]
    fn root_connects_to_scope_and_drops_class() {
        let s = q(3, 1);
        let html = render(&root(&s, vec![("class", "evil")], vec![]));
        assert!(html.contains(r#"data-scope="questionnaire""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("evil"));
    }

    #[test]
    fn progress_sets_percent_style_and_drops_caller_style() {
        let s = q(4, 1);
        let html = render(&progress(&s, "", vec![("style", "evil: 1")], vec![]));
        assert!(html.contains("--fandhe-questionnaire-percent: 25%"));
        assert!(!html.contains("evil"));
        assert_eq!(html.matches("style=").count(), 1);
    }

    #[test]
    fn question_connects_and_drops_class() {
        let s = q(3, 1);
        let html = render(&question(
            &s,
            1,
            QuestionProps::default(),
            vec![("class", "evil")],
            vec![],
        ));
        assert!(html.contains(r#"data-part="question""#));
        assert!(!html.contains("evil"));
    }

    #[test]
    fn slot_parts_connect_and_drop_class() {
        let s = q(3, 0);
        for (html, part) in [
            (
                render(&prompt(&s, vec![("class", "evil")], vec![text("hi")])),
                "prompt",
            ),
            (
                render(&description(&s, vec![("class", "evil")], vec![])),
                "description",
            ),
            (
                render(&options(&s, vec![("class", "evil")], vec![])),
                "options",
            ),
            (
                render(&freeform(&s, vec![("class", "evil")], vec![])),
                "freeform",
            ),
            (
                render(&actions(&s, vec![("class", "evil")], vec![])),
                "actions",
            ),
        ] {
            assert!(html.contains(&format!(r#"data-part="{part}""#)));
            assert!(!html.contains("evil"), "{part} leaked caller class");
        }
    }

    #[test]
    fn triggers_connect_and_drop_class() {
        let s = q(3, 1);
        for (html, part) in [
            (
                render(&back(&s, false, vec![("class", "evil")], vec![])),
                "back",
            ),
            (
                render(&next(&s, false, vec![("class", "evil")], vec![])),
                "next",
            ),
            (
                render(&skip(&s, false, vec![("class", "evil")], vec![])),
                "skip",
            ),
        ] {
            assert!(html.contains(&format!(r#"data-part="{part}""#)));
            assert!(!html.contains("evil"), "{part} leaked caller class");
        }
    }

    // --- XSS 回帰: 呼び出し側 attrs/children/label にペイロードを渡しても
    // エスケープされる ---

    const ATTR_BREAK_PAYLOAD: &str = "\" onmouseover=\"alert(1)";

    #[test]
    fn caller_attrs_payload_is_escaped_on_render() {
        let s = q(3, 1);
        let html = render(&root(&s, vec![("data-testid", ATTR_BREAK_PAYLOAD)], vec![]));
        assert!(!html.contains("onmouseover=\"alert(1)"));
    }

    #[test]
    fn children_text_is_escaped_on_render() {
        let s = q(3, 1);
        let html = render(&prompt(&s, vec![], vec![text("<script>alert(1)</script>")]));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn progress_label_payload_is_escaped_on_render() {
        let s = q(3, 1);
        let html = render(&progress(&s, ATTR_BREAK_PAYLOAD, vec![], vec![]));
        assert!(!html.contains("onmouseover=\"alert(1)"));
    }
}
