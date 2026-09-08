//! Input Group コンポーネント（イシュー #2062、親 #2061）。
//!
//! shadcn/ui の Input Group（入力欄の前後にテキスト/アイコン/ボタンの addon
//! を付ける複合パターン）に相当する anatomy パーツ関数群を提供する。
//! chakra-ui の `InputGroup` は `input` 単体扱いで独立部品化されておらず、
//! ark-ui にも直接の相当物が無いため、本モジュールは shadcn/ui のみを
//! 参照元とする（イシュー #2062 計画「背景・目的」節）。
//!
//! anatomy は `root` / `addon` / `text` / `button` の 4 パーツ構成。
//! [`root`] は `role="group"` のコンテナであり、実際の `<input>`/`<textarea>`
//! は本モジュールが出力せず、呼び出し側が [`crate::field::input`]/
//! [`crate::field::textarea`] を `root` の子として合成する契約とする
//! （`.claude/rules/coding-rust.md` §UI 部品の責務境界、規則 1: 本クレートは
//! anatomy・アクセシビリティ・表示状態のみを担い、コントロールの再実装は
//! しない）。
//!
//! # 状態機械を持たない理由
//!
//! [`crate::field`]/[`crate::fieldset`] と同じ判断（両モジュール冒頭 doc
//! 参照）: `disabled`/`invalid` は呼び出し側アプリケーションが決める SSR
//! 静的な props であり、Input Group 自身が開閉のような内部状態遷移を持たない。
//!
//! # `merge_field_props` と `fieldset` の差分
//!
//! [`InputGroupProps::merge_field_props`] は `disabled` だけでなく `invalid`
//! も OR 伝播する。[`crate::fieldset::FieldsetProps::merge_field_props`] が
//! `invalid` を伝播しない（複数コントロールを束ねる 1 対多の構造であり、
//! グループ全体のエラーを個別コントロールへ誤って波及させないための意図的
//! な非伝播）のに対し、Input Group は「1 group = 1 control」（1 つの
//! `<input>`/`<textarea>` を addon で装飾する 1 対 1 の構造）であるため、
//! グループの `invalid` はそのまま唯一のコントロールの `invalid` と等価であり
//! OR 伝播しても意味論の齟齬が生じない。
//!
//! # 参考サイトとの意図的な差分
//!
//! - shadcn の addon クリックで input へフォーカスを移動する JS 挙動は
//!   `fandhe-frontend-wasm-full` の配線が担う領域であり、親 #2061 に
//!   wasm-full 側の sub-issue が存在しないため本イシューのスコープ外とする
//!   （out-of-scope-tracking 対応、PR 本文に記録）。
//! - shadcn `InputGroupButton` の `size`/`variant` は装飾軸であり、`.claude/
//!   rules/coding-rust.md` §UI 部品の責務境界・規則 2 に従い上層の
//!   `fandhe-frontend-pre-styled-ui`（#2063）の責務とする。
//! - [`addon`] に `role="group"` を重ねない: [`root`] が既に `role="group"`
//!   を持つため、その内側に addon 用の `group` ロールを重ねると支援技術へ
//!   「入れ子のグループ」という冗長な情報を伝えてしまう。
//! - shadcn の `data-slot` 語彙は本リポジトリの `data-scope`/`data-part`
//!   （[`crate::anatomy::Anatomy`]）へ写像する。
//!
//! # ラベル付け・説明の契約
//!
//! Input Group は入力欄への `aria-labelledby`/`aria-describedby` を一切
//! 付与しない。アクセシブルネームは [`crate::field::label`]、補足説明は
//! [`crate::field::helper_text`]/[`crate::field::error_text`] が担う既存の
//! 責務のままとする（重複付与による ID 参照の競合・保守分岐を避ける）。
//! [`text`] パートは `aria-label` を持たない可視テキストとして扱い、
//! addon 内のラベルは常にこの可視テキストを優先する。
//!
//! # セキュリティ不変条件
//!
//! - `id`/子ノード等の動的値はすべて [`fandhe_frontend_core::el`] の属性値・
//!   子ノードとして渡り、[`fandhe_frontend_core::render`] の既定エスケープ
//!   （REQ-1）を必ず経由する。本モジュールは `raw_html()` を使用しない。
//! - 属性名・`data-align` の値はすべて `&'static str` リテラルで固定されて
//!   おり、呼び出し側の動的文字列が属性名/`data-align` の値スロットへ
//!   混入する経路はない（[`InputGroupAlign::as_str`] 参照）。

use crate::anatomy::{anatomy, Anatomy};
use crate::field::FieldProps;
use fandhe_frontend_core::Node;

/// `data-scope="input-group"` を固定した本コンポーネントの anatomy。
const ANATOMY: Anatomy = anatomy("input-group");

/// `input_group` モジュールの各パーツ関数（[`root`]/[`addon`]/[`button`]）へ
/// 共通で渡す props。
///
/// `disabled`/`invalid` は shadcn/ui の Input Group が子パーツへ配布する
/// フラグと同じ意味論を持つ、SSR 時点の静的な状態である。
#[derive(Debug, Clone, Copy)]
pub struct InputGroupProps {
    /// グループ全体の無効化。`true` のとき `root`/`addon`/`button` に
    /// `data-disabled` を付与する（ネイティブ `disabled` の伝播は呼び出し側
    /// の `field::input`/`field::textarea`/[`button`] が個別に担う）。
    pub disabled: bool,
    /// 唯一のコントロールの入力値が不正であることを示す。`true` のとき
    /// `root` に `aria-invalid="true"`・`data-invalid` を、`addon` にも
    /// `data-invalid` を付与する。
    pub invalid: bool,
}

impl InputGroupProps {
    /// Input Group の `disabled`/`invalid` を内包する [`FieldProps`] へ
    /// OR 伝播する（イシュー #2062）。
    ///
    /// モジュール doc「`merge_field_props` と `fieldset` の差分」節の通り、
    /// `fieldset` と異なり `invalid` も伝播する（1 group = 1 control のため）。
    #[must_use]
    pub fn merge_field_props<'a>(&self, mut field: FieldProps<'a>) -> FieldProps<'a> {
        field.disabled = field.disabled || self.disabled;
        field.invalid = field.invalid || self.invalid;
        field
    }
}

/// `addon` パーツの配置（shadcn `InputGroupAddon align` と同じ語彙）。
///
/// `InlineStart`/`InlineEnd` は `<input>` の前後（横方向）、`BlockStart`/
/// `BlockEnd` は `<textarea>` の上下（縦方向）に addon を置く用途を想定する。
/// 既存の [`crate::positioning::Align`]（tooltip/popover 等の浮遊要素の配置）
/// と役割が異なるため独立した列挙型として定義する（名前衝突を避ける命名）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputGroupAlign {
    /// 行内方向の先頭（横書きで input の左側）。
    InlineStart,
    /// 行内方向の末尾（横書きで input の右側）。
    InlineEnd,
    /// ブロック方向の先頭（textarea の上側）。
    BlockStart,
    /// ブロック方向の末尾（textarea の下側）。
    BlockEnd,
}

impl InputGroupAlign {
    /// `data-align` の属性値文字列を返す。
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InlineStart => "inline-start",
            Self::InlineEnd => "inline-end",
            Self::BlockStart => "block-start",
            Self::BlockEnd => "block-end",
        }
    }
}

impl Default for InputGroupAlign {
    /// shadcn/ui の既定（`inline-start`）と一致させる。
    fn default() -> Self {
        Self::InlineStart
    }
}

/// `disabled`/`invalid` フラグに対応する `data-disabled`/`data-invalid`
/// 存在属性をまとめて組み立てる（`crate::fieldset` の同名の内部ヘルパと
/// 同型の共通化、コピペによる将来のドリフトを避ける）。
#[must_use]
fn state_data_attrs(props: &InputGroupProps) -> Vec<(&'static str, &'static str)> {
    let mut attrs: Vec<(&'static str, &'static str)> = Vec::with_capacity(2);
    attrs.extend(crate::data_attrs::data_disabled(props.disabled));
    attrs.extend(crate::data_attrs::data_invalid(props.invalid));
    attrs
}

/// `root` パーツ（`div`、`role="group"`）。唯一のコントロール
/// （呼び出し側が合成する [`crate::field::input`]/[`crate::field::textarea`]）
/// と addon 群を束ねるコンテナ。`invalid` のとき `aria-invalid="true"` を
/// 付与する（モジュール doc「ラベル付け・説明の契約」節の通り、
/// `aria-labelledby`/`aria-describedby` は付与しない）。
#[must_use]
pub fn root<'a>(
    props: &InputGroupProps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&str, &str)> = Vec::with_capacity(attrs.len() + 4);
    merged.push(crate::aria::role("group"));
    if props.invalid {
        merged.push(crate::aria::aria_invalid(true));
    }
    merged.extend(state_data_attrs(props));
    merged.extend(attrs);
    ANATOMY.part("root", "div", merged, children)
}

/// `addon` パーツ（`div`、`data-align`）。テキスト/アイコン/ボタンを
/// input/textarea の指定位置へ配置するためのコンテナ。`role="group"` は
/// 重ねない（モジュール doc「参考サイトとの意図的な差分」節参照）。
#[must_use]
pub fn addon<'a>(
    align: InputGroupAlign,
    props: &InputGroupProps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&str, &str)> = Vec::with_capacity(attrs.len() + 3);
    merged.push(("data-align", align.as_str()));
    merged.extend(state_data_attrs(props));
    merged.extend(attrs);
    ANATOMY.part("addon", "div", merged, children)
}

/// `text` パーツ（`span`）。addon 内の可視テキストラベル。`aria-label` は
/// 付与しない（モジュール doc「ラベル付け・説明の契約」節参照。
/// 可視テキストそのものが支援技術に読み上げられるため追加のラベル付けは
/// 不要かつ重複になる）。
#[must_use]
pub fn text<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("text", "span", attrs, children)
}

/// `button` パーツ（`button type="button"`）。フォーム暗黙送信を起こさない
/// よう `type="button"` を固定する（[`crate::fieldset::root`] のネイティブ
/// `disabled` 伝播とは異なり、Input Group の `button` は addon 側の独立した
/// 操作要素であるため個別に `disabled` を受け取る）。
#[must_use]
pub fn button<'a>(
    props: &InputGroupProps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&str, &str)> = Vec::with_capacity(attrs.len() + 4);
    merged.push(("type", "button"));
    if props.disabled {
        merged.push(("disabled", ""));
    }
    merged.extend(state_data_attrs(props));
    merged.extend(attrs);
    ANATOMY.part("button", "button", merged, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::field::{FieldIds, FieldProps};
    use fandhe_frontend_core::{render, text as core_text};

    fn base_props() -> InputGroupProps {
        InputGroupProps {
            disabled: false,
            invalid: false,
        }
    }

    fn base_field_props(id: &str) -> FieldProps<'_> {
        FieldProps {
            id,
            ids: FieldIds::default(),
            disabled: false,
            invalid: false,
            required: false,
            readonly: false,
            has_helper_text: false,
        }
    }

    #[test]
    fn root_omits_flags_when_false() {
        let props = base_props();
        let html = render(&root(&props, vec![], vec![]));
        assert_eq!(
            html,
            r#"<div data-scope="input-group" data-part="root" role="group"></div>"#
        );
    }

    #[test]
    fn root_reflects_disabled_and_invalid_flags() {
        let mut props = base_props();
        props.disabled = true;
        props.invalid = true;
        let html = render(&root(&props, vec![], vec![]));
        assert_eq!(
            html,
            r#"<div data-scope="input-group" data-part="root" role="group" aria-invalid="true" data-disabled="" data-invalid=""></div>"#
        );
    }

    #[test]
    fn addon_supports_all_four_align_values() {
        let props = base_props();
        for (align, expected) in [
            (InputGroupAlign::InlineStart, "inline-start"),
            (InputGroupAlign::InlineEnd, "inline-end"),
            (InputGroupAlign::BlockStart, "block-start"),
            (InputGroupAlign::BlockEnd, "block-end"),
        ] {
            let html = render(&addon(align, &props, vec![], vec![]));
            assert_eq!(
                html,
                format!(
                    r#"<div data-scope="input-group" data-part="addon" data-align="{expected}"></div>"#
                )
            );
        }
    }

    #[test]
    fn addon_default_align_is_inline_start() {
        assert_eq!(InputGroupAlign::default(), InputGroupAlign::InlineStart);
    }

    #[test]
    fn addon_reflects_disabled_and_invalid_flags() {
        let mut props = base_props();
        props.disabled = true;
        props.invalid = true;
        let html = render(&addon(InputGroupAlign::InlineEnd, &props, vec![], vec![]));
        assert_eq!(
            html,
            r#"<div data-scope="input-group" data-part="addon" data-align="inline-end" data-disabled="" data-invalid=""></div>"#
        );
    }

    #[test]
    fn text_has_no_aria_label() {
        let html = render(&text(vec![], vec![core_text("$")]));
        assert_eq!(
            html,
            r#"<span data-scope="input-group" data-part="text">$</span>"#
        );
        assert!(!html.contains("aria-label"));
    }

    #[test]
    fn button_is_native_button_type_and_reflects_disabled() {
        let props = base_props();
        let html = render(&button(&props, vec![], vec![core_text("Clear")]));
        assert_eq!(
            html,
            r#"<button data-scope="input-group" data-part="button" type="button">Clear</button>"#
        );

        let mut disabled_props = base_props();
        disabled_props.disabled = true;
        let disabled_html = render(&button(&disabled_props, vec![], vec![core_text("Clear")]));
        assert_eq!(
            disabled_html,
            r#"<button data-scope="input-group" data-part="button" type="button" disabled="" data-disabled="">Clear</button>"#
        );
    }

    // --- merge_field_props（OR 伝播、イシュー #2062） ---

    #[test]
    fn merge_field_props_or_propagates_disabled() {
        let group_disabled = InputGroupProps {
            disabled: true,
            invalid: false,
        };
        let field_enabled = base_field_props("email");
        let merged = group_disabled.merge_field_props(field_enabled);
        assert!(merged.disabled);
    }

    #[test]
    fn merge_field_props_or_propagates_invalid() {
        let group_invalid = InputGroupProps {
            disabled: false,
            invalid: true,
        };
        let field_valid = base_field_props("email");
        let merged = group_invalid.merge_field_props(field_valid);
        assert!(merged.invalid);
    }

    #[test]
    fn merge_field_props_both_false_stays_false() {
        let group_enabled = base_props();
        let field_enabled = base_field_props("email");
        let merged = group_enabled.merge_field_props(field_enabled);
        assert!(!merged.disabled);
        assert!(!merged.invalid);
    }

    #[test]
    fn merge_field_props_field_side_true_is_preserved() {
        let group_enabled = base_props();
        let mut field_disabled = base_field_props("email");
        field_disabled.disabled = true;
        field_disabled.invalid = true;
        let merged = group_enabled.merge_field_props(field_disabled);
        assert!(merged.disabled);
        assert!(merged.invalid);
    }

    // --- XSS 回帰・anatomy 偽装除去（イシュー #2062） ---

    #[test]
    fn xss_payload_in_children_and_attrs_is_escaped_on_render() {
        let props = base_props();
        let html = render(&root(
            &props,
            vec![("id", "x\" onmouseover=\"alert(1)")],
            vec![
                text(vec![], vec![core_text("<script>alert(1)</script>")]),
                button(&props, vec![], vec![core_text("<script>alert(2)</script>")]),
            ],
        ));
        assert!(!html.contains("<script>alert"));
        assert!(!html.contains("onmouseover=\"alert"));
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
        assert!(html.contains("&lt;script&gt;alert(2)&lt;/script&gt;"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn caller_supplied_data_scope_and_part_are_dropped_fail_closed() {
        let props = base_props();
        let html = render(&root(
            &props,
            vec![("Data-Scope", "attacker"), ("DATA-PART", "attacker")],
            vec![],
        ));
        assert_eq!(
            html,
            r#"<div data-scope="input-group" data-part="root" role="group"></div>"#
        );
    }
}
