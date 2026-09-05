//! `download_trigger::root`（イシュー #828）の公開 API 経由統合テスト。
//!
//! `crates/headless-ui/src/download_trigger.rs` 側のユニットテストが
//! 値ごとの詳細な属性検証を行っているのに対し、本ファイルは参考サイト
//! （ark-ui `utilities/download-trigger.md` / chakra-ui
//! `buttons/download-trigger.md`。Radix Primitives に DownloadTrigger 相当
//! は存在しない）との突合契約（イシュー #1628）に絞る。
//!
//! 参考サイトの DownloadTrigger は Blob 生成・非同期データ解決を行う JS
//! ユーティリティで、Anatomy 節・Accessibility 節（Keyboard Interactions
//! 含む）を持たない `<button type="button">` 1 要素である（ローカル一次
//! 資料 `.agents/skills/ark-ui/references/utilities/download-trigger.md` /
//! `.agents/skills/chakra-ui/references/components/buttons/download-trigger.md`
//! に Anatomy/Accessibility 節が存在しないことを根拠とする間接証拠）。
//! 本実装は `a[href][download]` による静的トリガーへ置き換えているため、
//! 要素種別・`data-scope`/`data-part`（本実装側の superset）・キーボード
//! 操作（`a` は Enter のみ起動、Space は起動しない）は**意図的差分**であり、
//! 一方で「状態を表す `data-*` を一切出力しない」「`role`/`aria-*` を
//! 付与しない」点は参考サイトと一致する。以下はこの一致点・意図的差分の
//! 双方を fail-closed に固定する。

use fandhe_frontend_core::{render, text};
use fandhe_frontend_headless_ui::download_trigger::root;

/// anatomy は `root` 1 パーツのみ（`a[download]`）であり、`data-part=` の
/// 出現は 1 回に限られることを固定する（参考サイトに Anatomy 節が無く
/// パート分割の概念自体が存在しないことに対応する最小構成）。
#[test]
fn reference_anatomy_is_single_root_part_on_anchor() {
    let html = render(&root(
        "https://example.com/report.pdf",
        Some("report.pdf"),
        vec![],
        vec![text("Download report")],
    ));
    assert!(html.starts_with("<a"), "root は a 要素で始まる: {html}");
    assert!(html.contains(r#"data-scope="download-trigger""#));
    assert!(html.contains(r#"data-part="root""#));
    assert_eq!(
        html.matches("data-part=").count(),
        1,
        "download-trigger の anatomy は root 1 パーツのみ: {html}"
    );
}

/// `download` 属性は `file_name` の有無にかかわらず常に出力される
/// （`Some` はファイル名ヒント、`None` は空文字列＝配信元ファイル名を使う
/// ブラウザの挙動）ことを固定する。
#[test]
fn download_attribute_is_always_present() {
    let with_name = render(&root(
        "https://example.com/report.pdf",
        Some("report.pdf"),
        vec![],
        vec![],
    ));
    assert!(with_name.contains(r#"download="report.pdf""#));

    let without_name = render(&root(
        "https://example.com/report.pdf",
        None,
        vec![],
        vec![],
    ));
    assert!(without_name.contains(r#"download="""#));
}

/// 参照 headless（ark-ui/zag.js）の DownloadTrigger は開閉・非同期解決の
/// いずれも `data-*` として露出しない（内部で Promise 解決を待つのみ）。
/// 本実装も時間変化する内部状態を持たないため、状態を表す `data-*` を
/// 一切出力しないことを固定する（§3.25 規則 2: 装飾・アニメーション関心を
/// headless へ持ち込まない不変条件も含む）。
#[test]
fn no_state_data_attributes_are_emitted() {
    let html = render(&root(
        "https://example.com/report.pdf",
        Some("report.pdf"),
        vec![],
        vec![],
    ));
    for forbidden in [
        "data-state",
        "data-disabled",
        "data-invalid",
        "data-readonly",
        "data-orientation",
        "data-placement",
        "data-motion",
    ] {
        assert!(
            !html.contains(forbidden),
            "{forbidden} を含むべきでない: {html}"
        );
    }
}

/// 参考サイトは `role`/`aria-*` を付与しない（`button` のネイティブ
/// セマンティクスに委ねる）。本実装も `type=` を含め独自のロール・ARIA
/// 属性を付与せず、`a[href]` の暗黙 `link` ロールに委ねることを固定する。
#[test]
fn no_role_or_aria_or_type_is_emitted() {
    let html = render(&root(
        "https://example.com/report.pdf",
        Some("report.pdf"),
        vec![],
        vec![],
    ));
    assert!(!html.contains("role="), "{html}");
    assert!(!html.contains("aria-"), "{html}");
    assert!(!html.contains("type="), "{html}");
}

/// 無効化が必要な場合は呼び出し側 `attrs` で `aria-disabled`/`tabindex` を
/// 明示的に渡す経路（pre-styled 側 rustdoc が案内する無効化手段）が
/// そのまま透過することを固定する。
#[test]
fn caller_attrs_for_disabled_semantics_pass_through() {
    let html = render(&root(
        "https://example.com/report.pdf",
        Some("report.pdf"),
        vec![("aria-disabled", "true"), ("tabindex", "-1")],
        vec![],
    ));
    assert!(html.contains(r#"aria-disabled="true""#));
    assert!(html.contains(r#"tabindex="-1""#));
}

/// 呼び出し側 `attrs` に `data-scope`/`data-part` の偽装値を混入させても
/// anatomy 側の正規値で上書きされ fail-closed に除去されることを固定する
/// （`tests/fieldset.rs` の同型テストと同じ不変条件）。
#[test]
fn data_scope_and_part_spoofing_is_dropped() {
    let html = render(&root(
        "https://example.com/report.pdf",
        Some("report.pdf"),
        vec![("data-scope", "attacker"), ("data-part", "attacker")],
        vec![],
    ));
    assert!(html.contains(r#"data-scope="download-trigger""#));
    assert!(html.contains(r#"data-part="root""#));
    assert!(!html.contains("attacker"));
}
