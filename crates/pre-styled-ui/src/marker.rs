//! styled Marker（shadcn/ui `Marker` 相当。イシュー #2115、親 #2113、
//! 祖父トラッキング参照軸 #2001。headless 側 anatomy は #2114）。
//!
//! `fandhe_frontend_headless_ui::marker`（#2114）が出力する
//! `data-scope="marker"` の 3 パーツ（`root`/`icon`/`content`）へ、会話
//! スレッド内のインライン注記行の意匠（インライン注記 = `note` 形態、
//! 行下の境界線 = `divider` 形態、中央ラベル + 左右の線 = `label`
//! 形態、色調ごとの文字色）を重ねる薄い委譲層である
//! （[`crate::bubble`]/[`crate::attachment`] と同型の構成）。
//!
//! # 選択的 re-export（`pub use ...::*` を使わない理由）
//!
//! [`crate::bubble`]/[`crate::attachment`] と同型。3 パーツすべて同名
//! 再定義し（呼び出し側 `class` の除去は本モジュールの責務のため）、
//! [`MarkerRootProps`]/[`MarkerVariant`]/[`MarkerTone`] の 3 型のみを
//! 選択的に再エクスポートする。
//!
//! # 状態機械を持たない理由
//!
//! headless [`fandhe_frontend_headless_ui::marker`](mod@fandhe_frontend_headless_ui::marker) 自身が状態機械を
//! 持たない静的な自由関数群であるため、本モジュールもその設計をそのまま
//! 継承する（[`crate::attachment`] モジュール doc と同型の判断）。
//!
//! # 責務境界（`.claude/rules/coding-rust.md` §UI 部品の責務境界、
//! `docs/policy/intentional-non-adoption.md` §3.25）
//!
//! ストリーミング中判定・注記の自動分類・タイムスタンプ整形といった
//! アプリケーションロジックは実装しない（規則 1）。headless が出力する
//! `data-*` を CSS セレクタとして参照するだけで見た目を切り替える。
//!
//! # `data-variant`/`data-tone`: headless の `data-*` を `AttrEq` で参照
//! する（[`crate::attachment`] と同型の意図的差分）
//!
//! headless `marker::root` は `data-variant`（`note`/`divider`/`label`）・
//! `data-tone`（`neutral`/`info`/`warning`/`danger`）を固定出力済み
//! （`crates/headless-ui/src/marker.rs`）。本モジュールはこれらを
//! [`StateCondition::AttrEq`] で**参照するのみ**とし、class ベースの
//! [`SlotRecipe::variant`]（`ColorPalette` 軸）を持たない
//! （`docs/design/pre-styled-ui-data-attr-vocabulary.md` §2.2「役割 B:
//! 参照のみ」）。したがって 3 パーツすべて見た目クラスを付与しない同名
//! 再定義であり、[`crate::bubble`]/[`crate::attachment`] と同じパターンを
//! 踏襲する。
//!
//! # 区切り線の描画方式（疑似要素を使わない）
//!
//! [`crate::recipe::SlotRecipe`]/[`StateCondition`] は疑似要素セレクタ
//! （`::before`/`::after`）を表現できず（[`crate::splitter`]/
//! [`crate::drawer`]/[`crate::link_overlay`] rustdoc に記録済み）、本
//! モジュールでも疑似要素は使わない。
//!
//! - **`Divider` 形態**（shadcn `border` 相当）: DOM を増やさず、`root`
//!   へ `border-bottom` を state 規則として付与する（headless モジュール
//!   doc が `[data-variant]` 条件 CSS 経路を明示許容）。
//! - **`Label` 形態**（shadcn `separator` 相当）: [`root`] が呼び出し側
//!   `children` を [`crate::separator::separator`]（horizontal, `Solid`）
//!   2 個で挟んでから headless `marker::root` へ委譲する（イシュー要件
//!   「区切り線は separator のパートを再利用する」の実装）。挟み込む
//!   separator には `aria-hidden="true"` を渡す（下記「アクセシビリティ」
//!   節参照）。[`crate::recipe::SlotRecipe`] は子孫セレクタを持てないため
//!   （`group`（`grid-template-columns: 1fr auto 1fr`、セル数固定）は
//!   icon + content で子が 2 個になり 3 列 grid が崩れるため流用できない）、
//!   separator が伸縮するための規則（`flex: 1 1 0%` 等）は [`stylesheet`]
//!   内で子孫セレクタの raw CSS として追記する。
//! - **`Note` 形態**（既定）: 線なし。base のみ。
//!
//! # `stylesheet` が separator の基本 CSS を含まない理由（`separator::css()` 併用必須）
//!
//! [`stylesheet`] は marker 自身の recipe と、`Label` 形態で挟み込む
//! separator の**伸縮・折返し抑止のための子孫セレクタ規則のみ**を返す
//! （上記「区切り線の描画方式」節）。[`crate::separator::separator`] 自体の
//! border-width・border-style・margin 等の基本規則
//! （[`crate::separator::css`]）は再宣言しない
//! （[`crate::checkbox_group`] が `hidden-input` slot を持たず
//! `crate::checkbox::stylesheet()` 併用を必須とするのと同型の判断。
//! `crate::separator` の recipe を本モジュールへ複製すると、`separator`
//! 側の変更とドリフトし二重管理になるため）。**`Label` 形態を利用する
//! 呼び出し側は、本モジュールの [`stylesheet`] に加えて
//! [`crate::separator::css`] も併せて読み込む必要がある**
//! （`crates/docs-site/src/showcase.rs` が両方を `push_css` する実例を
//! 参照）。`Note`/`Divider` 形態のみを使う場合はこの併用は不要（separator
//! を DOM に挿入しないため）。
//!
//! # `data-tone` 別の色（直接トークン）
//!
//! [`crate::json_tree_view`]/[`crate::message`] の先例に倣い、
//! [`StateCondition::AttrEq`] で直接トークンを参照する（`ColorPalette`
//! class 軸は使わない）。
//!
//! - base（= `neutral` 既定。state 規則を書かない。「neutral = 既定」）:
//!   `color: var(--fandhe-color-fg-muted)`、
//!   `--fandhe-marker-line: var(--fandhe-color-border)`
//! - `info`: `var(--fandhe-color-info-fg-subtle)` /
//!   `var(--fandhe-color-info-muted)`
//! - `warning`: `var(--fandhe-color-warning-fg-subtle)` /
//!   `var(--fandhe-color-warning-muted)`
//! - `danger`: `var(--fandhe-color-danger-fg-subtle)` /
//!   `var(--fandhe-color-danger-muted)`
//!
//! `--fandhe-marker-line` は `root` が公開する scope 接頭辞付き custom
//! property であり、`Divider`/`Label` 形態の線色（境界線・separator の
//! `border-color`）が tone の色調と連動するために参照する
//! （[`crate::separator`](mod@crate::separator) の `--fandhe-separator-height` と同型のパターン）。
//!
//! # `ColorPalette` 軸を持たない理由
//!
//! [`crate::bubble`]/[`crate::attachment`] と同じ判断（本イシューの
//! スコープに含まれない）。必要になれば非破壊的に追加提案する
//! （`.claude/rules/out-of-scope-tracking.md` 対応）。
//!
//! # アクセシビリティ
//!
//! - `Label` 形態で挟み込む [`crate::separator::separator`] へ
//!   `aria-hidden="true"` を渡す。理由: `hr role="separator"` が短い
//!   ラベルの前後で 2 回読み上げられるのを避ける装飾線であるため
//!   （headless [`fandhe_frontend_headless_ui::marker`](mod@fandhe_frontend_headless_ui::marker) モジュール doc
//!   「アクセシビリティ」節と対をなす判断）。[`crate::separator`](mod@crate::separator) の予約
//!   キー（`role`/`aria-orientation`/`data-orientation`/`class`）に
//!   `aria-hidden` は含まれず透過する。
//! - [`icon`]/[`content`] は headless 側の a11y 契約（`icon` の
//!   `aria-hidden="true"` 固定、`content` の素の `span`）をそのまま
//!   継承する。
//!
//! # `prefers-reduced-motion` を書かない理由
//!
//! [`crate::theme::Theme::to_css`] が duration トークンを 0ms へ一括
//! 上書きするため本モジュール側では書かない（[`crate::bubble`]/
//! [`crate::attachment`] と同型）。hover / focus / disabled は静的表示
//! 部品のため N/A。
//!
//! # セキュリティ不変条件
//!
//! - 全出力は headless [`fandhe_frontend_headless_ui::marker`](mod@fandhe_frontend_headless_ui::marker) →
//!   `fandhe_frontend_core::render` の既定エスケープ（REQ-1）を必ず
//!   経由する。`raw_html()` は使用しない。
//! - 呼び出し側 `class` は `drop_class_attr` で除去してから headless
//!   関数へ委譲する（3 パーツすべて）。`Label` 形態で挟み込む
//!   [`crate::separator::separator`] は固定 props（horizontal・`Solid`）
//!   と固定 `aria-hidden` 属性のみを渡し、呼び出し側の動的値を経由しない。
//! - [`stylesheet`] が組み立てる CSS 宣言はすべてコンパイル時静的
//!   リテラルであり、[`crate::css::decl`]/[`crate::css::serialize_rule`]
//!   の検証を通る値のみを使う（raw CSS 追記部分を含む）。
//!
//! # スコープ外
//!
//! - `fandhe-frontend-wasm-full` の配線（headless rustdoc「wasm-full 未
//!   配線」参照。静的部品、状態機械なし）。
//! - `ColorPalette` 軸の追加（上記「`ColorPalette` 軸を持たない理由」
//!   参照）。
//! - `examples/headless-pre-styled-ui` への marker 追加。
//! - 会話系共通語彙（`data-role`/`data-align`）への追随（headless 側で
//!   意図的不追随を決定済み）。

use crate::class_attr::drop_class_attr;
use crate::css::{decl, serialize_rule};
use crate::recipe::{SlotRecipe, StateCondition};
use crate::separator::{separator, SeparatorProps};
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;

// headless 型のうち見た目クラスを付与しない本モジュールが必要とするのは
// props/variant/tone 型のみ（`crate::attachment` と同型の規約）。パーツ
// 関数 3 件は呼び出し側 `class` の除去を担うため同名再定義する。
pub use fandhe_frontend_headless_ui::marker::{MarkerRootProps, MarkerTone, MarkerVariant};

/// slot 一覧（headless [`fandhe_frontend_headless_ui::marker`](mod@fandhe_frontend_headless_ui::marker) の
/// anatomy と 1:1、3 パーツ）。
const SLOTS: &[&str] = &["root", "icon", "content"];

/// この styled Marker の既定 CSS を組み立てる（内部ヘルパ、
/// [`stylesheet`] のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    let root_base = vec![
        decl("display", "flex"),
        decl("align-items", "center"),
        decl("gap", "var(--fandhe-space-2)"),
        decl("font-size", "var(--fandhe-font-font-size-xs)"),
        decl("color", "var(--fandhe-color-fg-muted)"),
        decl("--fandhe-marker-line", "var(--fandhe-color-border)"),
    ];

    let icon_base = vec![
        decl("display", "inline-flex"),
        decl("align-items", "center"),
        decl("flex-shrink", "0"),
        decl("line-height", "1"),
    ];

    let content_base = vec![decl("min-width", "0")];

    SlotRecipe::new("marker", SLOTS)
        .base("root", root_base)
        .base("icon", icon_base)
        .base("content", content_base)
        // `data-variant="divider"`: 行の下に境界線（モジュール doc
        // 「区切り線の描画方式」節参照）。
        .state(
            "root",
            StateCondition::AttrEq("data-variant", "divider"),
            vec![
                decl(
                    "border-bottom",
                    "1px solid var(--fandhe-marker-line, var(--fandhe-color-border))",
                ),
                decl("padding-bottom", "var(--fandhe-space-2)"),
            ],
        )
        // `data-variant="label"`: 中央寄せ（挟み込む separator の伸縮規則は
        // 子孫セレクタのため `stylesheet` 内の raw CSS で追記、モジュール
        // doc参照）。
        .state(
            "root",
            StateCondition::AttrEq("data-variant", "label"),
            vec![
                decl("justify-content", "center"),
                decl("text-align", "center"),
            ],
        )
        // `data-tone`（モジュール doc「`data-tone` 別の色」節参照。
        // `neutral` は base と同値のため state 規則を書かない）。
        .state(
            "root",
            StateCondition::AttrEq("data-tone", "info"),
            vec![
                decl("color", "var(--fandhe-color-info-fg-subtle)"),
                decl("--fandhe-marker-line", "var(--fandhe-color-info-muted)"),
            ],
        )
        .state(
            "root",
            StateCondition::AttrEq("data-tone", "warning"),
            vec![
                decl("color", "var(--fandhe-color-warning-fg-subtle)"),
                decl("--fandhe-marker-line", "var(--fandhe-color-warning-muted)"),
            ],
        )
        .state(
            "root",
            StateCondition::AttrEq("data-tone", "danger"),
            vec![
                decl("color", "var(--fandhe-color-danger-fg-subtle)"),
                decl("--fandhe-marker-line", "var(--fandhe-color-danger-muted)"),
            ],
        )
}

/// この styled Marker が生成する静的 CSS 全量を返す（決定的。
/// [`crate::attachment::stylesheet`] と同じ契約）。`data-variant="label"`
/// 時に挟み込む [`crate::separator::separator`] の伸縮・中央ラベルの
/// 折返し抑止は `root`/separator/`content` にまたがる子孫セレクタのため
/// [`crate::recipe::SlotRecipe::state`]（単一 slot 前提）では表現できず、
/// raw CSS として追記する（モジュール doc「区切り線の描画方式」節参照）。
/// `Label` 形態を利用する場合は [`crate::separator::css`] も併せて読み込む
/// 必要がある（本関数は separator 自体の基本規則を含まない。モジュール doc
/// 「`stylesheet` が separator の基本 CSS を含まない理由」節参照）。
#[must_use]
pub fn stylesheet() -> String {
    let mut out = recipe().css();

    const ROOT: &str = r#"[data-scope="marker"][data-part="root"]"#;
    const CONTENT: &str = r#"[data-scope="marker"][data-part="content"]"#;
    const SEPARATOR_ROOT: &str = r#"[data-scope="separator"][data-part="root"]"#;

    let label_root = format!(r#"{ROOT}[data-variant="label"]"#);

    let append = |rule: Option<String>, out: &mut String| {
        if let Some(css) = rule {
            if !out.is_empty() {
                out.push('\n');
            }
            out.push_str(&css);
        }
    };

    // Label 形態: 挟み込んだ separator を残り幅いっぱいへ伸縮させる
    // （separator base の `flex-shrink: 0` と horizontal variant の
    // `width: 100%` を上書きしないと flex 内で伸縮しない）。線色は
    // `--fandhe-marker-line`（tone 連動）に合わせる。
    let separator_selector = format!("{label_root} > {SEPARATOR_ROOT}");
    append(
        serialize_rule(
            &separator_selector,
            &[
                decl("flex", "1 1 0%"),
                decl("width", "auto"),
                decl(
                    "border-color",
                    "var(--fandhe-marker-line, var(--fandhe-color-border))",
                ),
            ],
        ),
        &mut out,
    );

    // Label 形態: 中央ラベルの折返しを抑止する（separator::label と同じ
    // `white-space: nowrap`、モジュール doc参照）。
    let content_selector = format!("{label_root} > {CONTENT}");
    append(
        serialize_rule(&content_selector, &[decl("white-space", "nowrap")]),
        &mut out,
    );

    out
}

/// styled `root` パーツを組み立てる。見た目クラスは付与せず（モジュール
/// doc「headless の `data-*` を参照する」節参照）、呼び出し側 `class` を
/// `drop_class_attr` で除去してから
/// [`fandhe_frontend_headless_ui::marker::root`] へ委譲する。`Label`
/// 形態のときのみ `children` を [`crate::separator::separator`] 2 個
/// （`aria-hidden="true"`）で挟む（モジュール doc「区切り線の描画方式」
/// 節参照）。
#[must_use]
pub fn root<'a>(
    props: MarkerRootProps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_class_attr(attrs);
    if props.variant == MarkerVariant::Label {
        let decorative_separator =
            || separator(&SeparatorProps::default(), vec![("aria-hidden", "true")]);
        let mut wrapped: Vec<Node> = Vec::with_capacity(children.len() + 2);
        wrapped.push(decorative_separator());
        wrapped.extend(children);
        wrapped.push(decorative_separator());
        fandhe_frontend_headless_ui::marker::root(props, attrs, wrapped)
    } else {
        fandhe_frontend_headless_ui::marker::root(props, attrs, children)
    }
}

/// styled `icon` パーツを組み立てる。
#[must_use]
pub fn icon<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    fandhe_frontend_headless_ui::marker::icon(drop_class_attr(attrs), children)
}

/// styled `content` パーツを組み立てる。
#[must_use]
pub fn content<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    fandhe_frontend_headless_ui::marker::content(drop_class_attr(attrs), children)
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
        assert!(a.contains(r#"[data-scope="marker"][data-part="root"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let out = stylesheet();
        assert!(!out.contains("</style"));
        assert!(!out.contains('<'));
    }

    #[test]
    fn stylesheet_declares_variant_and_tone_rules() {
        let out = stylesheet();
        assert!(out.contains(r#"[data-variant="divider"]"#));
        assert!(out.contains(r#"[data-variant="label"]"#));
        assert!(out.contains(r#"[data-tone="info"]"#));
        assert!(out.contains(r#"[data-tone="warning"]"#));
        assert!(out.contains(r#"[data-tone="danger"]"#));
        // `neutral` は base と同値のため state 規則を書かない。
        assert!(!out.contains(r#"[data-tone="neutral"]"#));
        // class ベースの variant/tone クラス（`fd-marker--` 等）は生成しない
        // （モジュール doc参照）。
        assert!(!out.contains("fd-marker--"));
    }

    #[test]
    fn stylesheet_declares_label_separator_descendant_rules() {
        let out = stylesheet();
        assert!(out.contains(
            r#"[data-scope="marker"][data-part="root"][data-variant="label"] > [data-scope="separator"][data-part="root"]"#
        ));
        assert!(out.contains("flex: 1 1 0%;"));
    }

    #[test]
    fn root_connects_to_headless_marker_scope() {
        let html = render(&root(MarkerRootProps::default(), vec![], vec![]));
        assert!(html.contains(r#"data-scope="marker" data-part="root""#));
        assert!(html.starts_with("<div"));
    }

    #[test]
    fn all_parts_connect_to_headless_marker_scope() {
        let icon_html = render(&icon(vec![], vec![core_text("icon")]));
        assert!(icon_html.contains(r#"data-scope="marker" data-part="icon""#));

        let content_html = render(&content(vec![], vec![core_text("Today")]));
        assert!(content_html.contains(r#"data-scope="marker" data-part="content""#));
    }

    #[test]
    fn note_and_divider_variants_do_not_wrap_with_separator() {
        for variant in [MarkerVariant::Note, MarkerVariant::Divider] {
            let props = MarkerRootProps {
                variant,
                ..Default::default()
            };
            let html = render(&root(props, vec![], vec![content(vec![], vec![])]));
            assert!(
                !html.contains(r#"data-scope="separator""#),
                "{variant:?} -> {html}"
            );
        }
    }

    #[test]
    fn label_variant_wraps_children_with_decorative_separators() {
        let props = MarkerRootProps {
            variant: MarkerVariant::Label,
            ..Default::default()
        };
        let html = render(&root(
            props,
            vec![],
            vec![content(vec![], vec![core_text("2026-09-10")])],
        ));
        assert_eq!(
            html.matches(r#"data-scope="separator" data-part="root""#)
                .count(),
            2,
            "{html}"
        );
        assert_eq!(html.matches(r#"aria-hidden="true""#).count(), 2, "{html}");
        assert!(html.contains("2026-09-10"));
    }

    #[test]
    fn caller_class_is_dropped_on_every_part() {
        let html = render(&root(
            MarkerRootProps::default(),
            vec![("class", "evil")],
            vec![
                icon(vec![("class", "evil")], vec![]),
                content(vec![("class", "evil")], vec![]),
            ],
        ));
        assert!(!html.contains("evil"));
        assert_eq!(html.matches("class=\"").count(), 0);
    }
}
