//! `link::root`（イシュー #756）の公開 API 経由統合テスト。
//!
//! `crates/headless-ui/src/link.rs` 側のユニットテストが値ごとの詳細な
//! 属性検証を行っているのに対し、本ファイルは参考サイト（chakra-ui
//! `typography/link.md` / Radix Themes Link。ark-ui・Radix Primitives に
//! Link 相当は存在しない、`docs/design/component-coverage-map.md` 参照）
//! との突合契約（イシュー #1649）に絞る。
//!
//! 参考サイトの Link はいずれもスタイル prop（chakra: `variant` /
//! `colorPalette` / `asChild`。Radix Themes: `size` / `weight` /
//! `underline` / `color` / `highContrast` / `truncate` / `wrap` / `trim` /
//! `asChild`）のみを持つ styled `a` であり、Anatomy 節・Keyboard
//! Interactions 節・`data-*` 語彙・独自 ARIA 付与のいずれも持たない
//! （ローカル一次資料 `.agents/skills/chakra-ui/references/components/
//! typography/link.md`、`docs/design/radix-themes-survey.md:83` 参照）。
//! 本実装の `data-scope`/`data-part`/`data-current` は superset であり、
//! `external` の `target`+`rel` 不可分付与（reverse tabnabbing 対策）は
//! 参考サイトが利用者へ委ねている挙動を API 側で保証する意図的差分で
//! ある。以下はこの一致点・意図的差分の双方を fail-closed に固定する。

use fandhe_frontend_core::{render, text};
use fandhe_frontend_headless_ui::link::root;

/// anatomy は `root` 1 パーツのみ（`a`）であり、`data-part=` の出現は
/// 1 回に限られることを固定する（参考サイトに Anatomy 節が無くパート
/// 分割の概念自体が存在しないことに対応する最小構成）。
#[test]
fn reference_anatomy_is_single_root_part_on_anchor() {
    let html = render(&root(
        "https://example.com/docs",
        false,
        false,
        vec![],
        vec![text("Docs")],
    ));
    assert!(html.starts_with("<a"), "root は a 要素で始まる: {html}");
    assert!(html.contains(r#"data-scope="link""#));
    assert!(html.contains(r#"data-part="root""#));
    assert_eq!(
        html.matches("data-part=").count(),
        1,
        "link の anatomy は root 1 パーツのみ: {html}"
    );
}

/// 参照側（chakra-ui / Radix Themes）は状態を表す `data-*` を一切持たない。
/// 本実装も `external=false, current=false` の既定状態では `data-current`
/// を含め状態系 `data-*` を一切出力しないことを固定する（§3.25 規則 2:
/// 装飾・アニメーション関心を headless へ持ち込まない不変条件も含む）。
#[test]
fn no_state_data_attributes_by_default() {
    let html = render(&root(
        "https://example.com/docs",
        false,
        false,
        vec![],
        vec![],
    ));
    for forbidden in [
        "data-state",
        "data-disabled",
        "data-invalid",
        "data-orientation",
        "data-motion",
        "data-current",
    ] {
        assert!(
            !html.contains(forbidden),
            "{forbidden} を含むべきでない: {html}"
        );
    }
    assert_eq!(
        html.matches("data-").count(),
        2,
        "data-scope/data-part 以外の data-* を出力しない: {html}"
    );
}

/// 参考サイトは `role`/`aria-*` を独自付与せず、ネイティブ `a` の暗黙
/// `link` ロールに委ねる。本実装も既定状態では `role=`/`aria-` を一切
/// 出力しないことを固定する。
#[test]
fn no_role_or_aria_unless_current() {
    let html = render(&root(
        "https://example.com/docs",
        false,
        false,
        vec![],
        vec![],
    ));
    assert!(!html.contains("role="), "{html}");
    assert!(!html.contains("aria-"), "{html}");
}

/// `current=true` のとき `aria-current="page"` と `data-current` が
/// 同時に出力されることを固定する（片方のみが出る経路がないことの
/// 保証）。
#[test]
fn current_adds_aria_current_page_and_data_current_together() {
    let html = render(&root(
        "https://example.com/docs",
        false,
        true,
        vec![],
        vec![],
    ));
    assert!(html.contains(r#"aria-current="page""#));
    assert!(html.contains("data-current"));
}

/// `external` の `target="_blank"` + `rel="noopener noreferrer"` は
/// 不可分に付与・省略されることを固定する（参考実装〔chakra-ui〕は生の
/// `target`/`rel` を利用者が渡す設計であり、本実装は reverse tabnabbing
/// 対策を API 側で保証する意図的差分）。
#[test]
fn external_target_and_rel_are_inseparable() {
    let external = render(&root(
        "https://example.com/docs",
        true,
        false,
        vec![],
        vec![],
    ));
    assert!(external.contains(r#"target="_blank""#));
    assert!(external.contains(r#"rel="noopener noreferrer""#));

    let internal = render(&root(
        "https://example.com/docs",
        false,
        false,
        vec![],
        vec![],
    ));
    assert!(!internal.contains("target="));
    assert!(!internal.contains("rel="));
}

/// 呼び出し側 `attrs` で任意の属性（自前 CSS のフック用の `class` 等）が
/// そのまま透過することを固定する（参考サイトはスタイル prop を持つが、
/// 本実装は headless 契約によりスタイルレスのまま `class` 等の受け口を
/// 提供するのみであることの確認）。
#[test]
fn caller_attrs_pass_through() {
    let html = render(&root(
        "https://example.com/docs",
        false,
        false,
        vec![("class", "x")],
        vec![],
    ));
    assert!(html.contains(r#"class="x""#));
}

/// 呼び出し側 `attrs` に `data-scope`/`data-part` の偽装値を混入させても
/// anatomy 側の正規値で上書きされ fail-closed に除去されることを固定する
/// （`tests/download_trigger.rs` の同型テストと同じ不変条件）。
#[test]
fn caller_cannot_spoof_scope_or_part() {
    let html = render(&root(
        "https://example.com/docs",
        false,
        false,
        vec![("data-scope", "attacker"), ("data-part", "attacker")],
        vec![],
    ));
    assert!(html.contains(r#"data-scope="link""#));
    assert!(html.contains(r#"data-part="root""#));
    assert!(!html.contains("attacker"));
}

/// 危険な URL スキームでは `href` 属性ごと出力されない（既定エスケープ
/// 経路の fail-closed 拒否）。参考サイトはネイティブ `a` そのままで同種の
/// 保証を持たないため、本実装の superset な安全性として固定する。
#[test]
fn dangerous_href_scheme_drops_href_attribute() {
    let html = render(&root("javascript:alert(1)", false, false, vec![], vec![]));
    assert!(
        !html.contains("href="),
        "危険な URL スキームなのに href 属性が出力されている: {html}"
    );
}
