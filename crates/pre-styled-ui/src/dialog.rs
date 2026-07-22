//! styled Dialog（headless ラッパー第 1 弾、イシュー #551、親 #520/#545）。
//!
//! `fandhe_frontend_headless_ui::dialog`（イシュー #531）の Root / Trigger /
//! Backdrop / Positioner / Content / Title / Description / CloseTrigger
//! 8 anatomy パーツと [`fandhe_frontend_headless_ui::dialog::Dialog`] 状態機械を
//! そのまま再エクスポートし、[`stylesheet`] で既定 CSS を追加提供する。
//!
//! # 薄い委譲の根拠（本モジュールが新たな出力経路を持たない理由）
//!
//! headless 層の [`fandhe_frontend_headless_ui::anatomy::Anatomy::part`] は
//! 各パーツへ必ず `data-scope="dialog"` / `data-part="<slot>"` を付与する
//! （呼び出し側の偽装値は fail-closed で除去される、headless 側の既存保証）。
//! [`crate::recipe::SlotRecipe`] が生成する CSS のセレクタは
//! この `[data-scope][data-part]` 属性を直接ターゲットにするため、styled 層は
//! パーツ関数へ手を加えず再エクスポートするだけで既定スタイルを効かせられる
//! （クラス名注入を必要としない）。
//!
//! # data-state とスタイルの連動（イシュー #551 受け入れ条件）
//!
//! [`fandhe_frontend_headless_ui::state::Disclosure`] が出力する
//! `data-state="open"`/`"closed"`（headless 側の既存保証）に応じて
//! backdrop/content の見た目を切り替える CSS を [`stylesheet`] へ追加する
//! （[`state_css`] 参照）。[`crate::recipe::SlotRecipe`] は `data-scope`/
//! `data-part` セレクタのみを組み立てる API のため、`data-state` を含む
//! セレクタは [`crate::css::serialize_rule`] を直接使って組み立てる
//! （`SlotRecipe` と同じ fail-closed な宣言検証を経由する）。
//!
//! # 本イシューのスコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - variant（size 等）ごとのクラス切り替え・呼び出し側 `attrs` へのクラス
//!   注入は、#548 の `SlotRecipe::variant`/`variant_classes` を使えば追加可能
//!   だが、本イシュー（headless ラッパー第 1 弾）のスコープには含めない。
//! - フォーカストラップ・Escape キー閉鎖・外側クリック閉鎖・アニメーションは
//!   headless 層のドキュメント（`crates/headless-ui/src/dialog.rs`）で既に
//!   スコープ外と明記済みであり、本モジュールもそれを継承する。

use crate::css::{decl, serialize_rule};
use crate::recipe::SlotRecipe;

pub use fandhe_frontend_headless_ui::dialog::*;

/// headless `dialog` anatomy の `data-part` 一覧（`crates/headless-ui/src/dialog.rs`
/// の `ANATOMY.part(...)` 呼び出しと同期させる契約。ずれると [`stylesheet`] が
/// 一部パーツの CSS を出力しない fail-closed 側の不具合として現れるため、
/// 変更時は両ファイルを合わせて確認する）。
const SLOTS: &[&str] = &[
    "root",
    "trigger",
    "backdrop",
    "positioner",
    "content",
    "title",
    "description",
    "close-trigger",
];

/// この styled Dialog の既定 CSS を組み立てる（内部ヘルパ、[`stylesheet`] のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("dialog", SLOTS)
        .base(
            "backdrop",
            vec![
                decl("position", "fixed"),
                decl("inset", "0"),
                decl("background", "rgba(0, 0, 0, 0.4)"),
            ],
        )
        .base(
            "positioner",
            vec![
                decl("position", "fixed"),
                decl("inset", "0"),
                decl("display", "flex"),
                decl("align-items", "center"),
                decl("justify-content", "center"),
                decl("padding", "var(--fandhe-space-4)"),
            ],
        )
        .base(
            "content",
            vec![
                decl("background", "var(--fandhe-color-bg)"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("border-radius", "0.5rem"),
                decl("padding", "var(--fandhe-space-6)"),
                decl("max-width", "32rem"),
                decl("width", "100%"),
            ],
        )
        .base(
            "title",
            vec![
                decl("font-size", "var(--fandhe-font-font-size-lg)"),
                decl("font-weight", "var(--fandhe-font-font-weight-semibold)"),
                decl("margin", "0 0 var(--fandhe-space-2) 0"),
            ],
        )
        .base(
            "description",
            vec![
                decl("color", "var(--fandhe-color-fg-muted)"),
                decl("margin", "0"),
            ],
        )
        .base(
            "trigger",
            vec![
                decl("cursor", "pointer"),
                decl("color", "var(--fandhe-color-fg)"),
            ],
        )
        .base(
            "close-trigger",
            vec![
                decl("cursor", "pointer"),
                decl("color", "var(--fandhe-color-fg-muted)"),
            ],
        )
}

/// `data-state`（open/closed）に連動する CSS を組み立てる（内部ヘルパ、
/// [`stylesheet`] のみが呼ぶ）。`backdrop`/`content` の開閉状態に応じた
/// 見た目の切り替えを固定する（イシュー #551 受け入れ条件）。
fn state_css() -> String {
    let mut out = String::new();
    if let Some(css) = serialize_rule(
        r#"[data-scope="dialog"][data-part="backdrop"][data-state="open"]"#,
        &[decl("opacity", "1")],
    ) {
        out.push_str(&css);
    }
    if let Some(css) = serialize_rule(
        r#"[data-scope="dialog"][data-part="backdrop"][data-state="closed"]"#,
        &[decl("opacity", "0")],
    ) {
        out.push_str(&css);
    }
    if let Some(css) = serialize_rule(
        r#"[data-scope="dialog"][data-part="content"][data-state="open"]"#,
        &[decl("transform", "scale(1)")],
    ) {
        out.push_str(&css);
    }
    if let Some(css) = serialize_rule(
        r#"[data-scope="dialog"][data-part="content"][data-state="closed"]"#,
        &[decl("transform", "scale(0.95)")],
    ) {
        out.push_str(&css);
    }
    out
}

/// この styled Dialog が生成する静的 CSS 全量を返す（決定的。同一プロセス内で
/// 複数回呼んでも常にバイト単位で同一の文字列を返す、[`SlotRecipe::css`](crate::recipe::SlotRecipe::css)
/// の契約をそのまま継承する）。base 規則（[`recipe`]）の後に `data-state`
/// 連動規則（[`state_css`]）を連結する。
///
/// 呼び出し元は返り値を静的 `.css` ファイルとして配信する利用形態を想定し、
/// `<style>` タグへの埋め込み（`raw_html()` が必要になる）は本クレートの
/// 責務外（[`crate`] 冒頭の不変条件を参照）。
#[must_use]
pub fn stylesheet() -> String {
    recipe().css() + &state_css()
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;
    use fandhe_frontend_headless_ui::state::OpenState;

    #[test]
    fn stylesheet_is_deterministic_and_targets_data_scope_selectors() {
        let a = stylesheet();
        let b = stylesheet();
        assert_eq!(a, b);
        assert!(a.contains(r#"[data-scope="dialog"][data-part="content"]"#));
        assert!(a.contains(r#"[data-scope="dialog"][data-part="backdrop"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    #[test]
    fn reexported_root_renders_with_headless_anatomy_attrs() {
        // 再エクスポートされたパーツ関数が headless 層と同一の出力になることを固定する
        // （薄い委譲であることの回帰。呼び出し文脈: pre-styled-ui 経由でも
        // headless の data-scope/data-part 契約が保たれる）。
        let html = render(&root(OpenState::Closed, vec![], vec![]));
        assert!(html.contains(r#"data-scope="dialog""#));
        assert!(html.contains(r#"data-part="root""#));
    }

    #[test]
    fn stylesheet_links_data_state_to_style_open_and_closed() {
        // イシュー #551 受け入れ条件: 「headless 層の data-state とスタイルの
        // 連動テスト（[data-state='open'] セレクタ等）」を固定する。
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="dialog"][data-part="backdrop"][data-state="open"]"#));
        assert!(css.contains(r#"[data-scope="dialog"][data-part="backdrop"][data-state="closed"]"#));
        assert!(css.contains(r#"[data-scope="dialog"][data-part="content"][data-state="open"]"#));
    }

    #[test]
    fn ssr_and_hydration_round_trip_via_reexported_dialog_state_machine() {
        // イシュー #551 受け入れ条件: 「SSR / hydration 両経路の動作確認」を
        // 再エクスポートされた `Dialog`（headless の Component/Hydrate 実装を
        // そのまま継承）経由で固定する。
        use fandhe_frontend_interactive::{dispatch, render_for_hydration, Hydrate};

        let mut d = Dialog::default();
        assert_eq!(d.state(), OpenState::Closed);

        // SSR: 状態なし初期描画には data-hydrate-* が出ない。
        let ssr_html = render(&d.root(vec![], vec![]));
        assert!(ssr_html.contains(r#"data-state="closed""#));

        // dispatch で開閉し、hydration 属性へ反映されることを確認する。
        assert!(dispatch(&mut d, "open", ""));
        let hydrate_html = render(&render_for_hydration(&d));
        assert!(hydrate_html.contains(r#"data-hydrate-state="open""#));

        // クライアント側の改ざん耐性のある復元経路が Dialog 経由でも機能する。
        let restored = Dialog::from_hydration_attrs(&d.hydration_attrs()).unwrap();
        assert_eq!(restored.state(), OpenState::Open);
    }
}
