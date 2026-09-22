//! class 命名規約のヘルパ。
//!
//! 詳細な命名規約（部品ルート・パート・共通修飾・表示状態・カスタム
//! プロパティプレフィックス）は
//! `docs/design/wireframe-ui-architecture.md` §10 を正とする。

/// wireframe-ui が出力する全 class の共通プレフィックス。
pub const CLASS_PREFIX: &str = "fw-wire-";

/// `base` class に `modifiers` のうち `Some` の要素だけを半角スペースで
/// 連結した class 属性値を組み立てる。
///
/// 引数はいずれも `&'static str` に限定し、利用者入力（動的文字列）が
/// class へ流れ込む経路を型で塞ぐ（イシュー #2605 実装計画 §3.3・
/// `docs/policy/security.md` A03 対応）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_wireframe_ui::class_list;
///
/// let value = class_list("fw-wire-button", &[Some("fw-wire-size-md"), None, Some("fw-wire-bold")]);
/// assert_eq!(value, "fw-wire-button fw-wire-size-md fw-wire-bold");
/// ```
pub fn class_list(base: &'static str, modifiers: &[Option<&'static str>]) -> String {
    let mut out = String::from(base);
    for modifier in modifiers.iter().flatten() {
        out.push(' ');
        out.push_str(modifier);
    }
    out
}
