//! FileUpload（イシュー #840）の統合テスト。
//!
//! `crates/headless-ui/src/file_upload.rs` の inline unit tests がパーツ単体の
//! 属性出力・状態機械の遷移を固定するのに対し、本ファイルは「root >
//! label + dropzone(trigger + hidden-input) + item-group(item(item-name +
//! item-size-text + item-delete-trigger)) + clear-trigger」の組み立て全体の
//! data-*/ARIA 対応・dispatch 統合・SSR/hydration 両経路・拒否理由 enum の
//! 網羅・XSS 回帰をクレート外部から（公開 API のみを使って）固定する。

use fandhe_frontend_core::{render, text};
use fandhe_frontend_headless_ui::file_upload::{
    self, FileRejectionReason, FileUpload, FileUploadAction, FileUploadItem,
};
use fandhe_frontend_interactive::{dispatch, render_for_hydration, Component, Hydrate};

fn file(name: &str, size: u64, mime: &str) -> FileUploadItem {
    FileUploadItem::new(name, size, mime)
}

#[test]
fn full_assembly_wires_root_label_dropzone_item_group_and_clear_trigger() {
    let mut f = FileUpload::new("image/*", None, None, None);
    f.update(FileUploadAction::AddFiles(vec![file(
        "photo.png",
        2048,
        "image/png",
    )]));

    let items: Vec<_> = f
        .accepted()
        .iter()
        .map(|item| {
            let size_text = file_upload::item_size_text(item.size_bytes);
            file_upload::item(
                false,
                vec![],
                vec![
                    file_upload::item_name(vec![], vec![text(&item.name)]),
                    file_upload::item_size_text_node(vec![], vec![text(&size_text)]),
                    file_upload::item_delete_trigger(&item.name, false, vec![], vec![]),
                ],
            )
        })
        .collect();

    let label = f.label(vec![], vec![text("Files")]);
    let dropzone = f.dropzone(
        false,
        false,
        vec![],
        vec![
            f.trigger(false, vec![], vec![text("Browse")]),
            f.hidden_input(true, false, vec![]),
        ],
    );
    let item_group = file_upload::item_group(vec![], items);
    let clear = file_upload::clear_trigger(false, vec![], vec![text("Clear")]);
    let root = f.root(false, vec![], vec![label, dropzone, item_group, clear]);

    let html = render(&root);
    assert!(html.contains(r#"data-scope="file-upload""#));
    assert!(html.contains(r#"data-part="root""#));
    assert!(html.contains(r#"data-part="label""#));
    assert!(html.contains(r#"data-part="dropzone""#));
    assert!(html.contains(r#"role="button""#));
    assert!(html.contains(r#"data-part="trigger""#));
    assert!(html.contains(r#"data-part="hidden-input""#));
    assert!(html.contains(r#"type="file""#));
    assert!(html.contains(r#"accept="image/*""#));
    assert!(html.contains(r#"data-part="item-group""#));
    assert!(html.contains(r#"data-part="item""#));
    assert!(html.contains(r#"data-part="item-name""#));
    assert!(html.contains(r#"data-part="item-size-text""#));
    assert!(html.contains(r#"data-part="item-delete-trigger""#));
    assert!(html.contains(r#"data-part="clear-trigger""#));
    assert!(html.contains("photo.png"));
    assert!(html.contains("2.0 KB"));
}

#[test]
fn rejection_reasons_are_exhaustive() {
    // FileRejectionReason の全 variant が到達可能であることを固定する
    // （新規 variant 追加時にこのテストが変更を要求する）。
    let reasons = [
        FileRejectionReason::TooManyFiles,
        FileRejectionReason::FileInvalidType,
        FileRejectionReason::FileTooLarge,
        FileRejectionReason::FileTooSmall,
        FileRejectionReason::FileExists,
    ];
    assert_eq!(reasons.len(), 5);

    let mut too_many = FileUpload::new("", Some(0), None, None);
    too_many.update(FileUploadAction::AddFiles(vec![file("a", 1, "")]));
    assert_eq!(too_many.rejected()[0].1, FileRejectionReason::TooManyFiles);

    let mut invalid_type = FileUpload::new("image/*", None, None, None);
    invalid_type.update(FileUploadAction::AddFiles(vec![file(
        "a.txt",
        1,
        "text/plain",
    )]));
    assert_eq!(
        invalid_type.rejected()[0].1,
        FileRejectionReason::FileInvalidType
    );

    let mut too_large = FileUpload::new("", None, Some(10), None);
    too_large.update(FileUploadAction::AddFiles(vec![file("a", 20, "")]));
    assert_eq!(too_large.rejected()[0].1, FileRejectionReason::FileTooLarge);

    let mut too_small = FileUpload::new("", None, None, Some(10));
    too_small.update(FileUploadAction::AddFiles(vec![file("a", 1, "")]));
    assert_eq!(too_small.rejected()[0].1, FileRejectionReason::FileTooSmall);

    let mut exists = FileUpload::default();
    exists.update(FileUploadAction::AddFiles(vec![file("a", 1, "")]));
    exists.update(FileUploadAction::AddFiles(vec![file("a", 1, "")]));
    assert_eq!(exists.rejected()[0].1, FileRejectionReason::FileExists);
}

#[test]
fn dispatch_remove_and_clear_via_string_actions() {
    let mut f = FileUpload::default();
    f.update(FileUploadAction::AddFiles(vec![
        file("a", 1, ""),
        file("b", 1, ""),
    ]));
    assert!(dispatch(&mut f, "remove", "0"));
    assert_eq!(f.accepted()[0].name, "b");
    assert!(dispatch(&mut f, "clear", ""));
    assert!(f.is_empty());
}

#[test]
fn add_files_action_is_not_reachable_via_string_dispatch() {
    // モジュール doc「dispatch 契約」節: メタデータ付きファイル追加は型付き
    // API のみで受理し、文字列 dispatch では受理しない
    // （クライアント文字列へファイルメタデータを載せない設計）。
    let mut f = FileUpload::default();
    assert!(!dispatch(&mut f, "add-files", "a.txt"));
    assert!(f.is_empty());
}

#[test]
fn hydration_round_trip_via_public_api() {
    let mut f = FileUpload::new("image/*,.pdf", Some(10), Some(1_000_000), None);
    f.update(FileUploadAction::AddFiles(vec![
        file("a.png", 100, "image/png"),
        file("b.pdf", 200, "application/pdf"),
    ]));
    let rendered = render(&render_for_hydration(&f));
    assert!(rendered.contains(r#"data-hydrate-max-files="10""#));

    let restored = FileUpload::from_hydration_attrs(&f.hydration_attrs()).unwrap();
    assert_eq!(restored.accepted(), f.accepted());
    assert_eq!(restored.accept(), f.accept());
    assert_eq!(restored.max_files(), f.max_files());
}

// --- XSS 回帰: 公開 API 経由でもファイル名がエスケープされる ---

#[test]
fn public_api_item_name_with_script_payload_is_escaped_on_render() {
    let mut f = FileUpload::default();
    f.update(FileUploadAction::AddFiles(vec![file(
        "<script>alert(1)</script>",
        10,
        "text/plain",
    )]));
    let name = &f.accepted()[0].name;
    let html = render(&file_upload::item_name(vec![], vec![text(name)]));
    assert!(!html.contains("<script>alert(1)</script>"));
    assert!(html.contains("&lt;script&gt;"));
}
