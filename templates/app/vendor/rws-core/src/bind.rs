//! 束縛点マーキングの Node 木 API（イシュー #342）。
//!
//! `docs/design/dom-binding-update-design.md`（#340 設計確定書）第 3.1 節が
//! 凍結する SSR 出力形式（`data-bind-text="<field>"` /
//! `data-bind-attr="<attr>:<field>"` / `data-bind-class="<class>:<field>"`、
//! 複数束縛は空白区切りトークン）を生成するための薄いヘルパー群。
//! `rws-wasm-client`（#343）は起動時にこれらの属性を 1 回だけ走査し、
//! `field → DOM ノード` の束縛点対応表を構築する契約であり、本モジュールの
//! 出力形式が #343 の入力契約そのものである。
//!
//! # 既存防御への全面委譲（不変条件 1・2 の継承）
//!
//! 本モジュールは [`crate::el`] / [`crate::Node`] への委譲と文字列トークン
//! 合成のみを行い、独自の出力経路・独自のエスケープ処理・新しい検証ロジックを
//! 一切持たない。マーカー属性値・束縛テキスト値は既存の `render()`（`lib.rs`）
//! が行う属性値・テキストエスケープをそのまま経由する。フィールド名・属性名・
//! class 名はすべて `&'static str` に固定し、実行時の外部入力からの組み立てを
//! 型で遮断する（`crate::Node::Element::tag` と同じ設計原理、設計書 §3.3）。
//! ランタイムの assert/panic は追加しない。
//!
//! # スコープ外・引き継ぎ
//!
//! `data-bind-list` / `data-key`（keyed list）は #344 のスコープであり本
//! モジュールでは実装しない。束縛点対応表の構築・実 DOM への適用は #343 の
//! スコープ。トークン中の属性名の実行時検証（`on*` 属性・URL スキーム等）は
//! 消費側（#343）の契約であり、本モジュールは新たな検証機構を導入しない
//! （設計書 §9 不変条件 2 の残存リスク明記を継承）。

use crate::{el, text, Node};

/// テキスト束縛のマーカー属性名（SSR 出力に現れる契約値、設計書 §3.1 で凍結）。
pub const BIND_TEXT_ATTR: &str = "data-bind-text";
/// 属性束縛のマーカー属性名（同上）。
pub const BIND_ATTR_ATTR: &str = "data-bind-attr";
/// class 束縛のマーカー属性名（同上）。
pub const BIND_CLASS_ATTR: &str = "data-bind-class";

/// `"<attr>:<field>"` トークンを合成する（`data-bind-attr` 属性値用）。
///
/// 設計書 §3.1 が定める区切り文字 `:` を用いる。`attr`/`field` は
/// `&'static str` に固定されるため、実行時の外部入力から組み立てられることは
/// ない。トークンの出力時エスケープは呼び出し側が [`crate::el`] の属性値に
/// 渡した際に `render()`（`lib.rs`）が行う（本関数はエスケープ前の文字列
/// 合成のみを担う）。
///
/// # Examples
///
/// ```
/// use rws_core::bind_attr_token;
///
/// assert_eq!(bind_attr_token("aria-pressed", "liked"), "aria-pressed:liked");
/// ```
pub fn bind_attr_token(attr: &'static str, field: &'static str) -> String {
    format!("{attr}:{field}")
}

/// 複数の属性束縛を空白区切りで合成する（`data-bind-attr` 属性値用）。
///
/// 同一要素が複数属性を束縛する場合、[`bind_attr_token`] を個別に属性値へ
/// 割り当てると `data-bind-attr` 属性が要素内で重複し、ブラウザが先頭のみを
/// 採用して残りの束縛が黙って欠落する（設計書 §9 不変条件 6・fail-closed
/// 方針に反する）。本関数を使うことでマーカー属性の重複を構造的に防ぐ。
///
/// # Examples
///
/// ```
/// use rws_core::bind_attr_tokens;
///
/// assert_eq!(
///     bind_attr_tokens(&[("aria-pressed", "liked"), ("disabled", "busy")]),
///     "aria-pressed:liked disabled:busy"
/// );
/// ```
pub fn bind_attr_tokens(bindings: &[(&'static str, &'static str)]) -> String {
    bindings
        .iter()
        .map(|(attr, field)| bind_attr_token(attr, field))
        .collect::<Vec<_>>()
        .join(" ")
}

/// `"<class>:<field>"` トークンを合成する（`data-bind-class` 属性値用）。
///
/// [`bind_attr_token`] と同じ区切り文字規約に従う。
///
/// # Examples
///
/// ```
/// use rws_core::bind_class_token;
///
/// assert_eq!(bind_class_token("liked", "liked"), "liked:liked");
/// ```
pub fn bind_class_token(class: &'static str, field: &'static str) -> String {
    format!("{class}:{field}")
}

/// 複数の class 束縛を空白区切りで合成する（`data-bind-class` 属性値用）。
///
/// [`bind_attr_tokens`] と同じくマーカー属性重複を構造的に防ぐ。
///
/// # Examples
///
/// ```
/// use rws_core::bind_class_tokens;
///
/// assert_eq!(
///     bind_class_tokens(&[("liked", "liked"), ("busy", "loading")]),
///     "liked:liked busy:loading"
/// );
/// ```
pub fn bind_class_tokens(bindings: &[(&'static str, &'static str)]) -> String {
    bindings
        .iter()
        .map(|(class, field)| bind_class_token(class, field))
        .collect::<Vec<_>>()
        .join(" ")
}

/// テキスト束縛付き要素を構築する。
///
/// `attrs` の末尾へ `data-bind-text="<field>"` を決定的な順序（呼び出し側
/// 属性の後）で付加し、子は `Node::Text(value)` の 1 つのみとする
/// （設計書 §3.1「要素の唯一のテキスト子ノード」という不変条件を、構築の
/// 時点で構造的に保証する）。属性値・テキスト値のエスケープは既存の
/// `render()`（`lib.rs`）へ全面委譲し、新しいエスケープ処理を持たない。
///
/// `field` は `&'static str` に固定する（設計書 §3.3 の設計原理）。
///
/// 未使用時（本関数を呼ばない既存の `el`/`div`/`text` 等によるノード構築）の
/// `render()` 出力には一切影響しない（設計書 §3.3 の凍結条件）。
///
/// # Examples
///
/// ```
/// use rws_core::{bind_text, render};
///
/// let node = bind_text("span", vec![("class", "count")], "counter", "0");
/// assert_eq!(
///     render(&node),
///     r#"<span class="count" data-bind-text="counter">0</span>"#
/// );
/// ```
pub fn bind_text(
    tag: &'static str,
    attrs: Vec<(&str, &str)>,
    field: &'static str,
    value: impl Into<String>,
) -> Node {
    // 呼び出し元 `attrs` に既存の `data-bind-text` マーカーが含まれる場合、
    // 除去せず末尾に追加すると `render()` が 2 つのマーカーを出力してしまい、
    // HTML パース時は先頭のマーカーのみが有効になって `field` が黙って
    // 無視される不整合が生じる（Bugbot 指摘 df17d3a2-8566-438a-9459-3016e25667c5）。
    // 一意性を保証するため、まず既存エントリを除去してから新しい値を追加する。
    let mut attrs = attrs;
    attrs.retain(|(name, _)| *name != BIND_TEXT_ATTR);
    attrs.push((BIND_TEXT_ATTR, field));
    el(tag, attrs, vec![text(value)])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render;

    #[test]
    fn bind_attr_token_formats_single_binding() {
        assert_eq!(
            bind_attr_token("aria-pressed", "liked"),
            "aria-pressed:liked"
        );
    }

    #[test]
    fn bind_attr_tokens_joins_multiple_bindings_with_space() {
        assert_eq!(
            bind_attr_tokens(&[("aria-pressed", "liked"), ("disabled", "busy")]),
            "aria-pressed:liked disabled:busy"
        );
    }

    #[test]
    fn bind_class_token_formats_single_binding() {
        assert_eq!(bind_class_token("liked", "liked"), "liked:liked");
    }

    #[test]
    fn bind_class_tokens_joins_multiple_bindings_with_space() {
        assert_eq!(
            bind_class_tokens(&[("liked", "liked"), ("busy", "loading")]),
            "liked:liked busy:loading"
        );
    }

    #[test]
    fn bind_text_ssr_output_is_byte_exact() {
        let node = bind_text("span", vec![("class", "count")], "counter", "0");
        assert_eq!(
            render(&node),
            r#"<span class="count" data-bind-text="counter">0</span>"#
        );
    }

    #[test]
    fn bind_text_marker_attr_is_appended_after_caller_attrs() {
        // マーカー属性の出力位置（呼び出し側 attrs の後）を固定する回帰テスト。
        let node = bind_text("div", vec![("id", "x"), ("class", "y")], "field", "value");
        assert_eq!(
            render(&node),
            r#"<div id="x" class="y" data-bind-text="field">value</div>"#
        );
    }

    #[test]
    fn bind_text_render_is_deterministic() {
        let node = bind_text("span", vec![], "counter", "0");
        assert_eq!(render(&node), render(&node));
    }

    #[test]
    fn bind_text_marker_attr_passes_attr_name_whitelist() {
        // data-bind-text は既存 is_valid_attr_name（英数字・`-`・`_`・`:`）を
        // 素通しで通過する形式であることを固定する（設計書 §3.1）。
        let node = bind_text("span", vec![], "counter", "0");
        let html = render(&node);
        assert!(
            html.contains("data-bind-text=\"counter\""),
            "マーカー属性がホワイトリスト検証でスキップされた: {html}"
        );
    }

    #[test]
    fn bind_text_without_attrs_renders_marker_only() {
        let node = bind_text("span", vec![], "counter", "0");
        assert_eq!(render(&node), r#"<span data-bind-text="counter">0</span>"#);
    }

    #[test]
    fn bind_text_replaces_existing_bind_text_marker_in_attrs() {
        // `attrs` に既存の `data-bind-text` が含まれていても、`render()` が
        // 2 つのマーカーを出力して先頭のみ有効になる不整合を防ぐため、
        // 新しい `field` の値で一意に上書きされることを検証する。
        let node = bind_text(
            "span",
            vec![("data-bind-text", "old-field")],
            "new-field",
            "0",
        );
        assert_eq!(
            render(&node),
            r#"<span data-bind-text="new-field">0</span>"#
        );
    }
}
