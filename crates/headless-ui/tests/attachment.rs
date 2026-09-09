//! `fandhe-frontend-headless-ui` の Attachment（[`attachment`] モジュール、
//! イシュー #2111）の公開 API 経由の統合テスト。
//!
//! `crates/headless-ui/src/attachment.rs` 内の `#[cfg(test)]` ユニット
//! テストが内部実装を含めた網羅を担うのに対し、本ファイルは
//! `fandhe-frontend-headless-ui` の公開 API（`lib.rs` の再エクスポート
//! `pub mod attachment;`）のみを経由し、`fandhe-frontend-pre-styled-ui`
//! （styled ラッパー、#2112）が実際に使う想定の外部からの利用形態
//! （variant/state/disabled の組み合わせ + media/content(name/meta)/
//! progress（`crate::progress::Progress` の入れ子）/ actions > action の
//! 全パーツ構成、`message::content` スロットへの入れ子）を固定する回帰
//! テスト（`tests/bubble.rs` と同型の位置付け）。

use fandhe_frontend_core::{render, text};
use fandhe_frontend_headless_ui::attachment::{
    self, AttachmentRootProps, AttachmentState, AttachmentVariant,
};
use fandhe_frontend_headless_ui::data_attrs::Orientation;
use fandhe_frontend_headless_ui::message;
use fandhe_frontend_headless_ui::progress::Progress;

/// root（variant×2・state×3・disabled あり/なしのうち代表値）+ media +
/// content(name/meta) + progress（`Progress` の root/track/range を入れ子）+
/// actions > action で、8 パーツ全ての `data-part` が出現し、全語彙値が
/// 出現し、`data-hydrate-` は attachment scope に閉じては出現しない
/// （`progress` へ入れ子にした `Progress` 自体は正当に hydration 属性を
/// 出す）ことを固定する。
#[test]
fn root_with_all_parts_has_all_eight_parts_and_full_vocabulary() {
    let inner = Progress::new(0.0, 100.0, Some(64.0), Orientation::Horizontal);
    let node = attachment::root(
        AttachmentRootProps {
            variant: AttachmentVariant::Image,
            state: AttachmentState::Uploading,
            disabled: false,
        },
        vec![],
        vec![
            attachment::media(vec![], vec![text("[image preview]")]),
            attachment::content(
                vec![],
                vec![
                    attachment::name(vec![], vec![text("report.pdf")]),
                    attachment::meta(vec![], vec![text("PDF · 128 KB · 64%")]),
                ],
            ),
            attachment::progress(
                vec![],
                vec![inner.root(
                    None,
                    vec![],
                    vec![inner.track(vec![], vec![inner.range(vec![], vec![])])],
                )],
            ),
            attachment::actions(
                vec![],
                vec![attachment::action(
                    "Delete report.pdf",
                    false,
                    vec![],
                    vec![],
                )],
            ),
        ],
    );
    let html = render(&node);

    assert!(html.contains(r#"data-scope="attachment""#));
    for part in [
        "root", "media", "content", "name", "meta", "progress", "actions", "action",
    ] {
        assert!(
            html.contains(&format!(r#"data-part="{part}""#)),
            "part={part} が出力されていない: html={html}"
        );
    }

    assert!(html.contains(r#"data-variant="image""#));
    assert!(html.contains(r#"data-state="uploading""#));
    assert!(!html.contains("data-disabled"));
    assert!(html.contains("report.pdf"));
    assert!(html.contains("128 KB"));
    assert!(html.contains(r#"aria-label="Delete report.pdf""#));
    assert!(html.contains(r#"type="button""#));

    // 入れ子にした Progress の scope は独立して残るが、パーツメソッド
    // （`Progress::root` 等）を直接呼ぶ本テストの構成では hydration 属性は
    // 出力されない（`Component::render_for_hydration` を経由したときのみ
    // `Hydrate::hydration_attrs` が付与される、`Progress::view`/
    // `Hydrate::hydration_attrs` rustdoc 参照）。attachment scope 側も
    // 状態機械を持たないため、ツリー全体で `data-hydrate-` は一切
    // 出現しない。
    assert!(html.contains(r#"data-scope="progress""#));
    assert!(!html.contains("data-hydrate-"));
}

/// `AttachmentRootProps::default()` が `file`/`idle`/enabled になることを
/// 公開 API 経由で固定する。
#[test]
fn default_root_props_render_file_idle_enabled() {
    let html = render(&attachment::root(
        AttachmentRootProps::default(),
        vec![],
        vec![],
    ));
    assert!(html.starts_with("<div"));
    assert!(html.contains(r#"data-variant="file""#));
    assert!(html.contains(r#"data-state="idle""#));
    assert!(!html.contains("data-disabled"));
}

/// `disabled: true` のとき `root` に `data-disabled` 存在属性が付き、
/// `action` の `disabled: true` はネイティブ `disabled` + `data-disabled`
/// の両方を出力することを固定する。
#[test]
fn disabled_root_and_action_expose_presence_and_native_attrs() {
    let root_html = render(&attachment::root(
        AttachmentRootProps {
            disabled: true,
            ..Default::default()
        },
        vec![],
        vec![],
    ));
    assert!(root_html.contains(r#"data-disabled="""#));

    let action_html = render(&attachment::action("Delete", true, vec![], vec![]));
    assert!(action_html.contains(r#"disabled="""#));
    assert!(action_html.contains(r#"data-disabled="""#));
}

/// `state` の 3 値（`idle`/`uploading`/`error`）が公開 API 経由で
/// それぞれ正しく反映されることを固定する。
#[test]
fn root_state_error_is_reflected() {
    let html = render(&attachment::root(
        AttachmentRootProps {
            state: AttachmentState::Error,
            ..Default::default()
        },
        vec![],
        vec![],
    ));
    assert!(html.contains(r#"data-state="error""#));
}

/// `message::content` スロット内へ複数の `attachment::root`（複数添付
/// ファイル）を入れ子にしても、それぞれの scope が独立して固定される
/// ことを固定する（会話 1 発言の中に複数の添付ファイルを並べる想定の
/// 回帰）。
#[test]
fn nesting_multiple_attachments_inside_message_content_keeps_scopes_independent() {
    let node = message::content(
        vec![],
        vec![
            attachment::root(
                AttachmentRootProps {
                    variant: AttachmentVariant::File,
                    ..Default::default()
                },
                vec![],
                vec![attachment::name(vec![], vec![text("a.txt")])],
            ),
            attachment::root(
                AttachmentRootProps {
                    variant: AttachmentVariant::Image,
                    ..Default::default()
                },
                vec![],
                vec![attachment::name(vec![], vec![text("b.png")])],
            ),
        ],
    );
    let html = render(&node);

    assert!(html.contains(r#"data-scope="message""#));
    assert!(html.contains(r#"data-scope="attachment""#));
    assert!(html.contains(r#"data-variant="file""#));
    assert!(html.contains(r#"data-variant="image""#));
    assert!(html.contains("a.txt"));
    assert!(html.contains("b.png"));
}
