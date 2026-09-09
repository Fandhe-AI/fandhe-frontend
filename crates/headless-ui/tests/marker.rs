//! `fandhe-frontend-headless-ui` の Marker（[`marker`] モジュール、
//! イシュー #2114）の公開 API 経由の統合テスト。
//!
//! `crates/headless-ui/src/marker.rs` 内の `#[cfg(test)]` ユニットテストが
//! 内部実装を含めた網羅を担うのに対し、本ファイルは
//! `fandhe-frontend-headless-ui` の公開 API（`lib.rs` の再エクスポート
//! `pub mod marker;`）のみを経由し、`fandhe-frontend-pre-styled-ui`
//! （styled ラッパー、#2115）が実際に使う想定の外部からの利用形態
//! （variant×3・tone×4 の全語彙値・root/icon/content の全パーツ構成、
//! `message::content` スロットへの入れ子）を固定する回帰テスト
//! （`tests/attachment.rs` と同型の位置付け）。

use fandhe_frontend_core::{render, text};
use fandhe_frontend_headless_ui::marker::{self, MarkerRootProps, MarkerTone, MarkerVariant};
use fandhe_frontend_headless_ui::message;

/// root（variant×3・tone×4 の全組み合わせ）+ icon + content で、3 パーツ
/// 全ての `data-part` が出現し、全語彙値が出現し、`data-hydrate-` は
/// 一切出現しないことを固定する。
#[test]
fn root_with_all_parts_covers_full_vocabulary() {
    let variants = [
        MarkerVariant::Note,
        MarkerVariant::Divider,
        MarkerVariant::Label,
    ];
    let tones = [
        MarkerTone::Neutral,
        MarkerTone::Info,
        MarkerTone::Warning,
        MarkerTone::Danger,
    ];

    for variant in variants {
        for tone in tones {
            let node = marker::root(
                MarkerRootProps { variant, tone },
                vec![],
                vec![
                    marker::icon(vec![], vec![text("*")]),
                    marker::content(vec![], vec![text("Conversation compacted")]),
                ],
            );
            let html = render(&node);

            assert!(html.contains(r#"data-scope="marker""#));
            for part in ["root", "icon", "content"] {
                assert!(
                    html.contains(&format!(r#"data-part="{part}""#)),
                    "part={part} が出力されていない: html={html}"
                );
            }
            assert!(html.contains(&format!(r#"data-variant="{}""#, variant.as_str())));
            assert!(html.contains(&format!(r#"data-tone="{}""#, tone.as_str())));
            assert!(html.contains(r#"aria-hidden="true""#));
            assert!(html.contains("Conversation compacted"));
            assert!(!html.contains("data-hydrate-"));
        }
    }
}

/// `MarkerRootProps::default()` が `note`/`neutral` になることを公開 API
/// 経由で固定する。
#[test]
fn default_root_props_render_note_neutral() {
    let html = render(&marker::root(MarkerRootProps::default(), vec![], vec![]));
    assert!(html.starts_with("<div"));
    assert!(html.contains(r#"data-variant="note""#));
    assert!(html.contains(r#"data-tone="neutral""#));
}

/// `divider`/`label` variant でも headless 層は線要素（`hr` /
/// `role="separator"`）を出力しないことを固定する（区切り線の再利用は
/// `fandhe-frontend-pre-styled-ui` 側、#2115 の責務）。
#[test]
fn divider_and_label_variants_do_not_emit_a_line_element() {
    for variant in [MarkerVariant::Divider, MarkerVariant::Label] {
        let html = render(&marker::root(
            MarkerRootProps {
                variant,
                ..Default::default()
            },
            vec![],
            vec![],
        ));
        assert!(!html.contains("<hr"));
        assert!(!html.contains(r#"role="separator""#));
    }
}

/// `message::content` スロット内へ複数の `marker::root`（複数の注記行）を
/// 入れ子にしても、それぞれの scope が独立して固定されることを固定する
/// （会話スレッド内に複数のシステム注記を並べる想定の回帰）。
#[test]
fn nesting_multiple_markers_inside_message_content_keeps_scopes_independent() {
    let node = message::content(
        vec![],
        vec![
            marker::root(
                MarkerRootProps {
                    variant: MarkerVariant::Note,
                    ..Default::default()
                },
                vec![],
                vec![marker::content(vec![], vec![text("Explored 4 files")])],
            ),
            marker::root(
                MarkerRootProps {
                    variant: MarkerVariant::Label,
                    tone: MarkerTone::Info,
                },
                vec![],
                vec![marker::content(vec![], vec![text("Today")])],
            ),
        ],
    );
    let html = render(&node);

    assert!(html.contains(r#"data-scope="message""#));
    assert!(html.contains(r#"data-scope="marker""#));
    assert!(html.contains(r#"data-variant="note""#));
    assert!(html.contains(r#"data-variant="label""#));
    assert!(html.contains(r#"data-tone="info""#));
    assert!(html.contains("Explored 4 files"));
    assert!(html.contains("Today"));
}
