//! anatomy（部品構成）ヘルパ: ark-ui 流の `data-scope` / `data-part` セレクタを
//! 各コンポーネントのパーツ生成に一律付与するための薄い委譲層。
//!
//! Phase 2（イシュー #525 配下: Accordion / Tabs / Dialog 等）の各コンポーネントは
//! 自身の `Anatomy` を 1 つ持ち、パーツごとに [`Anatomy::part`] を呼んで
//! [`fandhe_frontend_core::el`] へ委譲する。本モジュール自体は状態機械や
//! スタイリングを持たず、属性 Vec の組み立てのみを担う（イシュー #523 スコープ）。

use fandhe_frontend_core::{el, Node};

/// コンポーネント 1 種の anatomy（`data-scope` 固定値）を表す。
///
/// `scope` は `&'static str` に固定する（動的文字列を受け付けない）。
/// これは `crates/core/src/tags.rs` のタグ名/属性名リテラル固定と同型の
/// 判断であり、呼び出し側の動的値によって `data-scope` セレクタが
/// 差し替えられる余地（意図しないセレクタ注入）を型レベルで塞ぐ。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Anatomy {
    scope: &'static str,
}

/// [`Anatomy::new`] の関数形。コンポーネント定義側で
/// `const ANATOMY: Anatomy = anatomy("accordion");` のように使うことを想定する。
#[must_use]
pub const fn anatomy(scope: &'static str) -> Anatomy {
    Anatomy::new(scope)
}

impl Anatomy {
    /// `scope` を `data-scope` の値として固定した [`Anatomy`] を作る。
    #[must_use]
    pub const fn new(scope: &'static str) -> Self {
        Self { scope }
    }

    /// この anatomy の `data-scope` 値を返す。
    #[must_use]
    pub const fn scope(&self) -> &'static str {
        self.scope
    }

    /// パーツ 1 個のノードを組み立てる。
    ///
    /// `[("data-scope", self.scope), ("data-part", part)]` を属性列の先頭に
    /// 置いたうえで呼び出し側の `attrs` を後続に連結し、
    /// [`fandhe_frontend_core::el`] を 1 回呼ぶだけの薄い委譲である
    /// （`docs/api/component-api.md` §4 定義規則 1・2 準拠）。
    /// 属性値のエスケープ・属性名/タグ名のホワイトリスト検証は `el`/`render`
    /// 側の既存責務のままであり、本関数は新たな出力経路を持たない。
    ///
    /// 呼び出し側 `attrs` に `data-scope` / `data-part`（ASCII 大文字小文字を
    /// 無視して比較）が含まれる場合はその要素を除外する。フレームワークが
    /// 付与する anatomy 属性が常に優先されることを保証し、重複属性による
    /// 無効な HTML 出力・後勝ちの非決定的な描画を防ぐ（fail-closed）。
    #[must_use]
    pub fn part<'a>(
        &self,
        part: &'static str,
        tag: &'static str,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        let mut merged: Vec<(&'a str, &'a str)> = Vec::with_capacity(attrs.len() + 2);
        merged.push(("data-scope", self.scope));
        merged.push(("data-part", part));
        merged.extend(attrs.into_iter().filter(|(k, _)| {
            !k.eq_ignore_ascii_case("data-scope") && !k.eq_ignore_ascii_case("data-part")
        }));
        el(tag, merged, children)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};

    #[test]
    fn part_prepends_scope_and_part() {
        let a = anatomy("accordion");
        let node = a.part("item", "div", vec![], vec![text("hi")]);
        assert_eq!(
            render(&node),
            r#"<div data-scope="accordion" data-part="item">hi</div>"#
        );
    }

    #[test]
    fn part_appends_caller_attrs_after_anatomy_attrs() {
        let a = anatomy("tabs");
        let node = a.part("trigger", "button", vec![("data-state", "open")], vec![]);
        assert_eq!(
            render(&node),
            r#"<button data-scope="tabs" data-part="trigger" data-state="open"></button>"#
        );
    }

    #[test]
    fn part_drops_caller_supplied_scope_and_part_case_insensitively() {
        let a = anatomy("dialog");
        let node = a.part(
            "content",
            "div",
            vec![
                ("Data-Scope", "attacker"),
                ("DATA-PART", "attacker"),
                ("id", "x"),
            ],
            vec![],
        );
        // フレームワーク値（dialog/content）が勝ち、呼び出し側の偽装値は落ちる。
        assert_eq!(
            render(&node),
            r#"<div data-scope="dialog" data-part="content" id="x"></div>"#
        );
    }

    #[test]
    fn scope_accessor_returns_fixed_value() {
        assert_eq!(anatomy("menu").scope(), "menu");
    }

    #[test]
    fn part_matches_direct_el_call_shape() {
        // el() を直接呼んだ場合と同じ出力形になることを固定する（薄い委譲であることの確認）。
        let a = anatomy("checkbox");
        let via_part = a.part("root", "label", vec![("id", "c1")], vec![]);
        let via_el = el(
            "label",
            vec![
                ("data-scope", "checkbox"),
                ("data-part", "root"),
                ("id", "c1"),
            ],
            vec![],
        );
        assert_eq!(render(&via_part), render(&via_el));
    }
}
