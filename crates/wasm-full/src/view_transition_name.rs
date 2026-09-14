//! keyed list の行等、値ごとに一意な `view-transition-name` を実行時に
//! 割り当てるユーティリティ（イシュー #2515）。
//!
//! # 背景・責務境界
//!
//! `fandhe-frontend-pre-styled-ui::recipe::view_transition_name_declaration`
//! （`motion` feature 配下）が担う「固定値」の静的なケースとは別系統。
//! 本モジュールは実行時に決まる値（keyed_list のキー等）を対象とし、
//! `Declaration::value` の `&'static str` 制約により pre-styled-ui 側では
//! 表現できないケースを補う。`stagger_index`/`content_height` と同じ、
//! 純粋層（native `cargo test` で検証可能）+ 配線層
//! （`#[cfg(target_arch = "wasm32")]`）の 2 層構成。
//!
//! `Runtime::mount`/`hydrate`・`apply_update_for_dirty` からの自動呼び出しは
//! **持たない**（`stagger_index` と異なり DOM 順位置ではなく呼び出し側の
//! 業務キーに基づく値のため、いつ・どの要素に割り当てるかはアプリ側が
//! 決める）。呼び出し側（アプリのイベントハンドラ・keyed_list 行構築後の
//! 独自配線コード等）が直接呼ぶ公開 API として提供する。
//!
//! # セキュリティ不変条件（REQ-1・security.md A03）
//!
//! `name` は呼び出し側由来の任意文字列（keyed_list のキー等、攻撃者制御の
//! 可能性がある値）であるため、[`is_valid_view_transition_name`] による
//! 許可リスト検証を経ない値は DOM へ一切書き込まない（fail-closed）。
//! CSS への文字列連結（`format!` 等）は行わず、検証済み文字列をそのまま
//! `CSSStyleDeclaration::set_property` の第 2 引数へ渡す（`;`/`}` 等による
//! 追加宣言の注入を許可リスト側で構造的に防ぐ）。

/// [`set_view_transition_name`] が受理する識別子形式かどうかを判定する。
///
/// `[a-z][a-z0-9-]*` の許可リストに加え、CSS 側で特別な意味を持つ予約語
/// （`none`/CSS-wide keywords）を拒否する。`view-transition-name: none`
/// は「名前なし」を意味する予約語であり、キーの文字列表現として `"none"`
/// を意図しない形で無効化しないための拒否。
#[must_use]
pub fn is_valid_view_transition_name(name: &str) -> bool {
    const RESERVED: [&str; 6] = [
        "none",
        "inherit",
        "initial",
        "unset",
        "revert",
        "revert-layer",
    ];
    if RESERVED.contains(&name) {
        return false;
    }
    let mut chars = name.chars();
    match chars.next() {
        Some(c) if c.is_ascii_lowercase() => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

#[cfg(all(target_arch = "wasm32", feature = "view-transition-name"))]
mod wiring {
    use super::is_valid_view_transition_name;
    use web_sys::HtmlElement;

    /// `element` の inline style へ `view-transition-name: <name>` を書き込む。
    ///
    /// `name` が [`is_valid_view_transition_name`] を満たさない場合は
    /// 何も書き込まず `false` を返す（fail-closed、既存の値も保持される）。
    /// 書き込みに成功したら `true` を返す。
    pub fn set_view_transition_name(element: &HtmlElement, name: &str) -> bool {
        if !is_valid_view_transition_name(name) {
            return false;
        }
        element
            .style()
            .set_property("view-transition-name", name)
            .is_ok()
    }
}

#[cfg(all(target_arch = "wasm32", feature = "view-transition-name"))]
pub use wiring::set_view_transition_name;

#[cfg(test)]
mod tests {
    use super::is_valid_view_transition_name;

    #[test]
    fn accepts_lowercase_kebab() {
        assert!(is_valid_view_transition_name("logo"));
        assert!(is_valid_view_transition_name("row-42"));
    }

    #[test]
    fn rejects_reserved_keywords() {
        assert!(!is_valid_view_transition_name("none"));
        assert!(!is_valid_view_transition_name("inherit"));
        assert!(!is_valid_view_transition_name("unset"));
    }

    #[test]
    fn rejects_invalid_charset_and_injection_attempts() {
        assert!(!is_valid_view_transition_name(""));
        assert!(!is_valid_view_transition_name("Logo"));
        assert!(!is_valid_view_transition_name("1abc"));
        assert!(!is_valid_view_transition_name("-abc"));
        assert!(!is_valid_view_transition_name("a b"));
        assert!(!is_valid_view_transition_name("a;color:red"));
        assert!(!is_valid_view_transition_name("a}body{color:red"));
    }
}
