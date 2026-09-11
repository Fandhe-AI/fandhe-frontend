//! Button Group（shadcn/ui Button Group 相当）headless コンポーネント
//! （イシュー #2059、親 #2058、Phase 4 #2057、祖父トラッキング参照軸 #2001）。
//!
//! `docs/design/component-coverage-map.md` の shadcn/ui 参照軸（イシュー
//! #2004）にのみ存在し他 3 参照軸（ark-ui / chakra-ui / Radix）に対応が
//! ない部品を埋める。関連ボタンを角丸・境界線でひとつのグループに見せる
//! **静的なグループ化**を表現する `root` / `separator` / `text` の 3
//! anatomy パーツを提供する。
//!
//! # `role="group"` の静的グループであり `toolbar` と異なる
//!
//! [`crate::toolbar`](mod@crate::toolbar) は `role="toolbar"` + roving tabindex（矢印キーで
//! フォーカスが移動する複合ウィジェット）の状態機械を持つが、本モジュール
//! は shadcn/ui の Button Group 同様、子 `button` 要素のフォーカス順序は
//! ネイティブの Tab 順序に委ねる**静的なグループ**である。したがって
//! [`root`] は状態機械を持たず、[`crate::fieldset`](mod@crate::fieldset) と同じ理由
//! （SSR 時点で決まる静的な props のみで完結する）で自由関数のみで構成
//! する。
//!
//! # `role="group"` に `aria-orientation` を付与しない
//!
//! WAI-ARIA は `group` ロールへの `aria-orientation` を許可していない
//! （`mod@crate::toolbar::toggle_group` の PR #791 Bugbot 指摘と同じ
//! 判断）。向きの状態は `data-orientation` のみで表現し、CSS 側
//! （`fandhe-frontend-pre-styled-ui`、後続 #2060）が `flex-direction` 等を
//! 切り替える。
//!
//! # 先頭 / 末尾ボタンの角丸連結は CSS の責務
//!
//! 子 `button` が先頭 / 末尾かどうかの判定に専用の `data-*` は追加しない
//! （親 #2058 の方針）。`> :first-child` / `> :last-child` の CSS
//! セレクタで表現できるため、状態を持ち込んで anatomy を複雑化しない
//! （`.claude/rules/coding-rust.md` の UI 部品責務境界 §3.25 とも整合する
//! 判断: 連結角丸のような装飾は pre-styled-ui 側の責務）。
//!
//! # ネスト
//!
//! [`root`] はネスト（グループの中にグループ）を許容する。内側の
//! `root` は自身の `data-orientation` のみを持ち、外側の値へは影響しない
//! （[`crate::nav_list`](mod@crate::nav_list) のような入れ子コンテナと同様、`Anatomy::part`
//! が呼び出しごとに独立した属性列を組み立てるため自然に成立する）。
//!
//! # `separator` は `toolbar`/`action_bar` と同型の直交規則
//!
//! [`separator`] はグループ自身の向きと**直交**する `aria-orientation`/
//! `data-orientation` を出力する（横並びグループの区切り線は縦線になる
//! ため `vertical`。[`crate::toolbar::separator`]/[`crate::action_bar::separator`]
//! と同じ判断）。
//!
//! # `text` はラベル用の非ボタン要素
//!
//! shadcn/ui の `ButtonGroupText` 相当。[`text`] はネイティブ `<div>` を
//! 返す（`<label>` にしたい場合は呼び出し側が `attrs`/子ノードで工夫する
//! 想定であり、本モジュールは特定のセマンティック要素へ限定しない）。
//!
//! # 呼び出し文脈
//!
//! SSR/SSG は本モジュールの自由関数（[`root`]/[`separator`]/[`text`]、
//! いずれも純粋関数）を直接呼んで組み立てる。状態機械を持たないため
//! `fandhe-frontend-interactive`/hydration 属性（`data-hydrate-*`）は
//! 一切出力しない。`fandhe-frontend-pre-styled-ui` が本モジュールを呼んで
//! 連結角丸・境界線を持つスタイル済み Button Group を組み立てる想定
//! （#2060、本イシューのスコープ外）。
//!
//! # セキュリティ不変条件
//!
//! - 属性名（`role`/`aria-*`/`data-orientation`）はすべて `&'static str`
//!   リテラルで固定しており、動的値が属性名スロットへ混入する経路はない
//!   （[`crate::anatomy`](mod@crate::anatomy)/[`crate::aria`]/[`crate::data_attrs`] の
//!   既存不変条件をそのまま継承する）。
//! - 動的値（`label`/呼び出し側 `attrs`/`children`）は
//!   [`fandhe_frontend_core::render`] の既定エスケープを必ず経由する
//!   （REQ-1）。`raw_html()` は使用せず、HTML 文字列を直接組み立てない。
//! - **呼び出し側による予約キーのなりすまし除去**: `drop_reserved` が
//!   パート別の `*_RESERVED` 定数（ASCII 大文字小文字無視の完全一致）を
//!   使い、呼び出し側 `attrs` から `role`/`aria-*`/`data-orientation` 等、
//!   本モジュールが固定付与する属性名を除去してから固定値を合成する
//!   （`crate::toolbar::drop_reserved`/`crate::nav_list::drop_reserved`
//!   と同型のパターン）。`data-scope`/`data-part` の偽装は
//!   [`crate::anatomy::Anatomy::part`] が別途除去する。
//!
//! # スコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - pre-styled-ui 側の recipe（連結角丸・境界線・`:first-child`/
//!   `:last-child` スタイル）・golden テスト・docs サイト Themes ページは
//!   #2060 で扱う。
//! - wasm-full 側の配線は不要（静的グループであり、キーボード操作は
//!   ネイティブ `button` の Tab 順序のみで完結する）。

use crate::anatomy::{anatomy, Anatomy};
use crate::aria::{aria_label, aria_orientation, role};
use crate::data_attrs::{data_orientation, Orientation};
use fandhe_frontend_core::Node;

/// Button Group の anatomy（`data-scope="button-group"`）。
const ANATOMY: Anatomy = anatomy("button-group");

/// [`root`] が固定付与する予約キー。
const ROOT_RESERVED: &[&str] = &["role", "data-orientation", "aria-label"];

/// [`separator`] が固定付与する予約キー。
const SEPARATOR_RESERVED: &[&str] = &["role", "aria-orientation", "data-orientation"];

/// [`text`] は現状固定属性を持たないが、将来の追加に備え対称性のため
/// 空の予約キー定数を用意する（[`drop_reserved`] は空スライスでも安全に
/// 動作する）。
const TEXT_RESERVED: &[&str] = &[];

/// 呼び出し側 `attrs` から予約キー（本モジュールが固定付与する属性名）を
/// 除去する（ASCII 大文字小文字無視の完全一致）。`fandhe_frontend_core::el`
/// は属性の重複除去をしないため、これを経由しない呼び出しは状態属性の
/// なりすましを許してしまう（[`crate::toolbar::drop_reserved`] と同型）。
fn drop_reserved<'a>(
    attrs: Vec<(&'a str, &'a str)>,
    reserved: &'static [&'static str],
) -> Vec<(&'a str, &'a str)> {
    attrs
        .into_iter()
        .filter(|(k, _)| !reserved.iter().any(|r| k.eq_ignore_ascii_case(r)))
        .collect()
}

/// Root パーツ（`div`）。`role="group"` + `data-orientation` を固定出力
/// する（`aria-orientation` は付与しない、モジュール doc 参照）。`label`
/// は動的値であり [`fandhe_frontend_core::render`] の既定エスケープを
/// 経由して `aria-label` へ出力する（空文字列のときは省略する。
/// `aria-labelledby` を使いたい場合は呼び出し側が `attrs` 経由で渡す）。
/// ネスト（`root` の子に別の `root`）を許容する。
#[must_use]
pub fn root<'a>(
    orientation: Orientation,
    label: &'a str,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, ROOT_RESERVED);
    let mut merged: Vec<(&'a str, &'a str)> = vec![role("group"), data_orientation(orientation)];
    if !label.is_empty() {
        merged.push(aria_label(label));
    }
    merged.extend(attrs);
    ANATOMY.part("root", "div", merged, children)
}

/// Separator パーツ（`div`）。`role="separator"` + グループ自身の向きと
/// **直交**する `aria-orientation`/`data-orientation` を固定出力する
/// （横並びグループのセパレータは縦線になるため `vertical`、
/// [`crate::toolbar::separator`] と同じ判断）。
#[must_use]
pub fn separator<'a>(
    group_orientation: Orientation,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let orthogonal = match group_orientation {
        Orientation::Horizontal => Orientation::Vertical,
        Orientation::Vertical => Orientation::Horizontal,
    };
    let attrs = drop_reserved(attrs, SEPARATOR_RESERVED);
    let mut merged: Vec<(&'a str, &'a str)> = vec![
        role("separator"),
        aria_orientation(orthogonal),
        data_orientation(orthogonal),
    ];
    merged.extend(attrs);
    ANATOMY.part("separator", "div", merged, children)
}

/// Text パーツ（`div`）。shadcn/ui の `ButtonGroupText` 相当のラベル用
/// 非ボタン要素（モジュール doc 参照）。固定付与属性を持たない。
#[must_use]
pub fn text<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let attrs = drop_reserved(attrs, TEXT_RESERVED);
    ANATOMY.part("text", "div", attrs, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;

    #[test]
    fn root_horizontal_has_group_role_and_no_aria_orientation() {
        let node = root(Orientation::Horizontal, "Actions", vec![], vec![]);
        let html = render(&node);
        assert!(html.contains(r#"data-scope="button-group""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(html.contains(r#"role="group""#));
        assert!(html.contains(r#"data-orientation="horizontal""#));
        assert!(html.contains(r#"aria-label="Actions""#));
        assert!(!html.contains("aria-orientation"));
    }

    #[test]
    fn root_vertical_sets_data_orientation() {
        let node = root(Orientation::Vertical, "", vec![], vec![]);
        let html = render(&node);
        assert!(html.contains(r#"data-orientation="vertical""#));
        assert!(!html.contains("aria-label"));
    }

    #[test]
    fn root_empty_label_omits_aria_label() {
        let node = root(Orientation::Horizontal, "", vec![], vec![]);
        let html = render(&node);
        assert!(!html.contains("aria-label"));
    }

    #[test]
    fn separator_orientation_is_orthogonal_to_group() {
        let horizontal_group_sep = render(&separator(Orientation::Horizontal, vec![], vec![]));
        assert!(horizontal_group_sep.contains(r#"role="separator""#));
        assert!(horizontal_group_sep.contains(r#"aria-orientation="vertical""#));
        assert!(horizontal_group_sep.contains(r#"data-orientation="vertical""#));

        let vertical_group_sep = render(&separator(Orientation::Vertical, vec![], vec![]));
        assert!(vertical_group_sep.contains(r#"aria-orientation="horizontal""#));
        assert!(vertical_group_sep.contains(r#"data-orientation="horizontal""#));
    }

    #[test]
    fn text_part_has_no_fixed_attrs() {
        let node = text(vec![("data-testid", "label")], vec![]);
        let html = render(&node);
        assert!(html.contains(r#"data-scope="button-group""#));
        assert!(html.contains(r#"data-part="text""#));
        assert!(html.contains(r#"data-testid="label""#));
    }

    #[test]
    fn nested_root_keeps_independent_orientation() {
        let inner = root(Orientation::Vertical, "", vec![], vec![]);
        let outer = root(Orientation::Horizontal, "", vec![], vec![inner]);
        let html = render(&outer);
        // 外側の data-orientation="horizontal" が先に、内側の
        // data-orientation="vertical" が後に出現し、双方が独立して
        // 保持されていることを確認する。
        let horizontal_idx = html.find(r#"data-orientation="horizontal""#).unwrap();
        let vertical_idx = html.find(r#"data-orientation="vertical""#).unwrap();
        assert!(horizontal_idx < vertical_idx);
    }

    #[test]
    fn root_drops_impersonated_reserved_keys_case_insensitively() {
        let node = root(
            Orientation::Horizontal,
            "Actions",
            vec![
                ("Role", "toolbar"),
                ("DATA-ORIENTATION", "vertical"),
                ("aria-label", "Fake"),
                ("data-testid", "kept"),
            ],
            vec![],
        );
        let html = render(&node);
        assert!(html.contains(r#"role="group""#));
        assert!(!html.contains("toolbar"));
        assert!(html.contains(r#"data-orientation="horizontal""#));
        assert!(html.contains(r#"aria-label="Actions""#));
        assert!(!html.contains("Fake"));
        assert!(html.contains(r#"data-testid="kept""#));
    }

    #[test]
    fn separator_drops_impersonated_reserved_keys() {
        let node = separator(
            Orientation::Horizontal,
            vec![("role", "button"), ("aria-orientation", "horizontal")],
            vec![],
        );
        let html = render(&node);
        assert!(html.contains(r#"role="separator""#));
        assert!(html.contains(r#"aria-orientation="vertical""#));
        assert!(!html.contains(r#"role="button""#));
    }

    #[test]
    fn anatomy_scope_and_part_spoofing_is_removed() {
        let node = root(
            Orientation::Horizontal,
            "",
            vec![("data-scope", "evil"), ("data-part", "evil")],
            vec![],
        );
        let html = render(&node);
        assert!(html.contains(r#"data-scope="button-group""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("evil"));
    }

    #[test]
    fn xss_payload_in_label_and_children_is_escaped_on_render() {
        let payload = "\"><script>alert(1)</script>";
        let node = root(
            Orientation::Horizontal,
            payload,
            vec![("id", payload)],
            vec![text(vec![], vec![fandhe_frontend_core::text(payload)])],
        );
        let html = render(&node);
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;"));
    }
}
