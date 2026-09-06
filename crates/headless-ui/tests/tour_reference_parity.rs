//! `tour`（イシュー #841）を ark-ui docs / 実装（`tour.anatomy.ts`・
//! `tour-actions.tsx`・`tour-control.tsx`）・zag.js
//! （`tour.anatomy.ts`・`tour.connect.ts`・`tour.machine.ts`）と突合した
//! 契約を fail-closed に固定する統合テスト（イシュー #1666、
//! `splitter_reference_parity.rs` と同型の立て付け）。Radix Primitives に
//! Tour 相当は存在しないため突合対象に含めない
//! （`docs/design/component-coverage-map.md` 参照）。
//!
//! # 突合結果（詳細は `crate::tour` モジュール doc「参照突合
//! （イシュー #1666）」節参照）
//!
//! - **是正**: `control`（ark-ui `data-part="control"`。docs 図の
//!   「Actions」に対応）パーツの追加、`content` への `tabindex="-1"`/
//!   `data-step` の追加、`action_trigger` への `data-type`/`disabled`/
//!   `data-disabled` の追加（[`TourTriggerKind`] 引数の破壊的変更）、
//!   `drop_reserved` による予約キーなりすまし除去。
//! - **非追随**: `role="alertdialog"`（`role="dialog"` を維持）、
//!   `aria-modal="true"`、content 自体の `aria-live`、ステップ種別
//!   `data-type`、`data-placement`/`data-side` on content/title/
//!   description、`data-nested`/`data-has-nested`、`dismissed` status、
//!   `arrow` の `hidden`、`close-trigger` の既定 `aria-label`。
//!
//! 公開 API（`fandhe_frontend_headless_ui::tour::{Tour, TourStep,
//! TourTriggerKind, ContentIds}`）のみを使い、`crate` 内部実装には
//! 依存しない。

use fandhe_frontend_core::render;
use fandhe_frontend_headless_ui::positioning::{Align, Placement, Side};
use fandhe_frontend_headless_ui::tour::{ContentIds, Tour, TourStep, TourTriggerKind};
use fandhe_frontend_interactive::dispatch;

/// `tag>` までを抜き出し開始タグのみを対象に属性を検査するヘルパ
/// （`splitter_reference_parity.rs` と同型のパターン）。
fn tag_slice<'a>(html: &'a str, needle: &str) -> &'a str {
    let start = html.find(needle).unwrap_or_else(|| {
        panic!("needle {needle:?} not found in html: {html}");
    });
    let end = html[start..].find('>').unwrap() + start;
    &html[start..end]
}

fn one_step() -> Vec<TourStep> {
    vec![TourStep {
        id: "s1".to_string(),
        target: Some("#a".to_string()),
        title: "One".to_string(),
        description: "first".to_string(),
        placement: Placement::new(Side::Bottom, Align::Center),
    }]
}

fn two_steps() -> Vec<TourStep> {
    vec![
        TourStep {
            id: "s1".to_string(),
            target: Some("#a".to_string()),
            title: "One".to_string(),
            description: "first".to_string(),
            placement: Placement::new(Side::Bottom, Align::Center),
        },
        TourStep {
            id: "s2".to_string(),
            target: Some("#b".to_string()),
            title: "Two".to_string(),
            description: "second".to_string(),
            placement: Placement::new(Side::Bottom, Align::Center),
        },
    ]
}

/// 是正: ark-ui `data-part="control"`（docs 図の「Actions」相当）が
/// 出力される。
#[test]
fn control_part_is_output_with_scope_and_state() {
    let t = Tour::new(one_step());
    let html = render(&t.control(vec![], vec![]));
    assert!(html.contains(r#"data-scope="tour""#));
    let tag = tag_slice(&html, r#"data-part="control""#);
    assert!(tag.contains(r#"data-state="closed""#));
}

/// 是正: content は `tabindex="-1"` を固定付与する（zag
/// `content.connect.ts` 準拠）。
#[test]
fn content_has_tabindex_minus_one() {
    let t = Tour::new(one_step());
    let html = render(&t.content(ContentIds::default(), vec![], vec![]));
    let tag = tag_slice(&html, r#"data-part="content""#);
    assert!(tag.contains(r#"tabindex="-1""#));
}

/// 是正: Active 時のみ現在ステップの `id` を `data-step` として出力する。
#[test]
fn content_outputs_data_step_only_when_active() {
    let mut t = Tour::new(one_step());
    let idle_html = render(&t.content(ContentIds::default(), vec![], vec![]));
    assert!(!tag_slice(&idle_html, r#"data-part="content""#).contains("data-step"));

    dispatch(&mut t, "start", "");
    let active_html = render(&t.content(ContentIds::default(), vec![], vec![]));
    assert!(tag_slice(&active_html, r#"data-part="content""#).contains(r#"data-step="s1""#));
}

/// 非追随: `role="dialog"` を維持し `alertdialog` へは切り替えない
/// （WAI-ARIA `alertdialog` は即時応答を要する警告向けであり、
/// オンボーディング案内は該当しないため）。
#[test]
fn content_keeps_role_dialog_not_alertdialog() {
    let t = Tour::new(one_step());
    let html = render(&t.content(ContentIds::default(), vec![], vec![]));
    let tag = tag_slice(&html, r#"data-part="content""#);
    assert!(tag.contains(r#"role="dialog""#));
    assert!(!tag.contains("alertdialog"));
}

/// 非追随: `aria-modal="true"` は付与しない（wasm-full にフォーカス
/// トラップ配線がまだ無く、SSR でトラップされていない状態を偽って
/// 主張しないため）。
#[test]
fn content_does_not_output_aria_modal() {
    let mut t = Tour::new(one_step());
    dispatch(&mut t, "start", "");
    let html = render(&t.content(ContentIds::default(), vec![], vec![]));
    assert!(!html.contains("aria-modal"));
}

/// 非追随: content 自体は `aria-live` を持たない（`progress-text` のみが
/// 持つ既存方針を維持、二重ライブリージョンの重複読み上げを避ける）。
#[test]
fn content_has_no_aria_live_progress_text_does() {
    let t = Tour::new(one_step());
    let content_html = render(&t.content(ContentIds::default(), vec![], vec![]));
    assert!(!tag_slice(&content_html, r#"data-part="content""#).contains("aria-live"));

    let progress_html = render(&t.progress_text(vec![], vec![]));
    assert!(tag_slice(&progress_html, r#"data-part="progress-text""#).contains("aria-live"));
}

/// 是正: action-trigger は `kind` に応じた `data-type` を出力する。
#[test]
fn action_trigger_outputs_data_type() {
    let t = Tour::new(one_step());
    let html = render(&t.action_trigger(TourTriggerKind::Skip, vec![], vec![]));
    assert!(tag_slice(&html, r#"data-part="action-trigger""#).contains(r#"data-type="skip""#));
}

/// 非追随: `data-type` はステップ種別（tooltip/dialog/floating/wait）の
/// 意味では出さない（本状態機械はステップ種別を初版スコープ外としている）。
#[test]
fn action_trigger_data_type_is_not_step_type_vocabulary() {
    let t = Tour::new(one_step());
    for kind in [
        TourTriggerKind::Next,
        TourTriggerKind::Prev,
        TourTriggerKind::Skip,
        TourTriggerKind::Complete,
        TourTriggerKind::Custom,
    ] {
        let html = render(&t.action_trigger(kind, vec![], vec![]));
        let tag = tag_slice(&html, r#"data-part="action-trigger""#);
        for step_type in ["tooltip", "dialog", "floating", "wait"] {
            assert!(!tag.contains(&format!(r#"data-type="{step_type}""#)));
        }
    }
}

/// 是正: `prev` は dispatch が no-op になる境界（`step == 0`）でのみ
/// disabled になる。
#[test]
fn prev_disabled_only_at_dispatch_no_op_boundary() {
    let mut t = Tour::new(two_steps());
    dispatch(&mut t, "start", "");
    let first = render(&t.action_trigger(TourTriggerKind::Prev, vec![], vec![]));
    assert!(tag_slice(&first, r#"data-part="action-trigger""#).contains("disabled"));

    dispatch(&mut t, "next", "");
    let second = render(&t.action_trigger(TourTriggerKind::Prev, vec![], vec![]));
    assert!(!tag_slice(&second, r#"data-part="action-trigger""#).contains("disabled"));
}

/// 非追随: 本状態機械では最終 step の `"next"` が `Completed` へ遷移する
/// 有効な操作であるため、zag の `!hasNextStep` 判定とは異なり `next` を
/// 最終 step でも disabled にしない。
#[test]
fn next_is_never_disabled_even_at_last_step() {
    let mut t = Tour::new(two_steps());
    dispatch(&mut t, "start", "");
    dispatch(&mut t, "next", "");
    let html = render(&t.action_trigger(TourTriggerKind::Next, vec![], vec![]));
    assert!(!tag_slice(&html, r#"data-part="action-trigger""#).contains("disabled"));
}

/// 非追随: content/title/description に `data-placement`/`data-side` は
/// 出力しない（`positioning::placement_attrs` は positioner のみへ出力する
/// 既存設計、popover #1642 と同判断）。
#[test]
fn placement_attrs_are_positioner_only() {
    let mut t = Tour::new(one_step());
    dispatch(&mut t, "start", "");
    let content_html = render(&t.content(ContentIds::default(), vec![], vec![]));
    assert!(!tag_slice(&content_html, r#"data-part="content""#).contains("data-side"));

    let title_html = render(&t.title(None, vec![], vec![]));
    assert!(!tag_slice(&title_html, r#"data-part="title""#).contains("data-side"));

    let positioner_html = render(&t.positioner(vec![], vec![]));
    assert!(tag_slice(&positioner_html, r#"data-part="positioner""#).contains("data-side"));
}
