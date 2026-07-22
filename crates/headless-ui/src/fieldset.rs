//! Fieldset コンポーネント（イシュー #602、親 #578）。
//!
//! ark-ui の Fieldset（`.claude/skills/ark-ui/references/components/form/fieldset.md`）
//! に倣い、複数の [`crate::field`] をグループ化するネイティブ `<fieldset>`/
//! `<legend>` コンテナを anatomy パーツ関数群として提供する。
//!
//! anatomy は `root` / `legend` / `helper_text` / `error_text` の 4 パーツ
//! 構成（ark-ui 準拠）。`root` は `<fieldset>` 要素であり、`disabled` の
//! ネイティブ伝播（HTML 仕様により子コントロールが自動的に無効化される）を
//! 前提に、[`FieldsetProps::merge_field_props`] で [`crate::field::FieldProps`]
//! 側の data-*/ネイティブ属性表現とも整合させる。
//!
//! # 状態機械を持たない理由
//!
//! [`crate::field`] と同じ判断（同モジュール冒頭 doc 参照）: `disabled`/
//! `invalid` は呼び出し側アプリケーションが決める SSR 静的な props であり、
//! Fieldset 自身が開閉のような内部状態遷移を持たない。ark-ui の
//! `RootProvider`/`useFieldset` 相当（動的な状態共有）は本 SSR 純関数方式では
//! 非適用（out-of-scope-tracking 対応、PR 本文に記録）。
//!
//! # legend 連携
//!
//! [`legend`] はネイティブ `<legend>` 要素として `<fieldset>` 内の先頭に置く
//! ことを呼び出し側の契約とする。ネイティブ `<fieldset>`+`<legend>` の組が
//! HTML 仕様によりグループへアクセシブルネームを与えるため、追加の
//! `aria-labelledby` 等は不要（ブラウザ・支援技術が自動的に関連付ける）。
//!
//! # 呼び出し文脈
//!
//! - 上層の [`crate::anatomy::Anatomy`]・[`crate::aria`]・[`crate::data_attrs`]
//!   へ薄く委譲するのみで、独自の出力経路・独自のエスケープ処理は持たない。
//! - styled 層（`fandhe-frontend-pre-styled-ui`）は本モジュールが出力する
//!   `data-scope="fieldset"`/`data-part="..."` セレクタを前提にスタイルを
//!   当てる想定（#603 系、本イシューのスコープ外）。
//!
//! # セキュリティ不変条件
//!
//! - `id`/子ノード等の動的値はすべて [`fandhe_frontend_core::el`] の属性値・
//!   子ノードとして渡り、[`fandhe_frontend_core::render`] の既定エスケープ
//!   （REQ-1）を必ず経由する。本モジュールは `raw_html()` を使用しない。
//! - 属性名はすべて `&'static str` リテラルで固定されており、動的値が属性名
//!   スロットへ混入する経路はない。
//! - `error_text` は非該当状態（`!invalid`）で `hidden` 存在属性を付与する
//!   fail-closed 描画とし、JS 不在の SSR でも誤表示しない
//!   （[`crate::field::error_text`] と同型）。

use crate::anatomy::{anatomy, Anatomy};
use crate::field::FieldProps;
use fandhe_frontend_core::Node;

/// `data-scope="fieldset"` を固定した本コンポーネントの anatomy。
const ANATOMY: Anatomy = anatomy("fieldset");

/// `fieldset` モジュールの各パーツ関数（[`root`]/[`legend`]/[`helper_text`]/
/// [`error_text`]）へ共通で渡す props。
///
/// `disabled`/`invalid` は ark-ui の `Fieldset.Root` が子パーツへ配布する
/// フラグと同じ意味論を持つ、SSR 時点の静的な状態である。
pub struct FieldsetProps<'a> {
    /// ベース id。`legend`/`helper_text`/`error_text` の決定的 id 生成
    /// （`"{id}-legend"`/`"{id}-helper-text"`/`"{id}-error-text"`）に使う。
    pub id: &'a str,
    /// グループ全体の無効化。`true` のとき `root`（`<fieldset>`）にネイティブ
    /// `disabled` 存在属性を付与する（HTML 仕様により内側の全コントロールが
    /// 自動的に無効化される）ほか `data-disabled` を付与する。
    pub disabled: bool,
    /// グループレベルの入力値が不正であることを示す。`true` のとき `root` に
    /// `data-invalid` を付与し、`error_text` を表示状態にする。個別 Field の
    /// `aria-invalid` へは伝播しない（[`merge_field_props`](FieldsetProps::merge_field_props)
    /// 参照。誤ったコントロール単位のエラー通知を避けるための意図的な判断）。
    pub invalid: bool,
    /// `helper_text` パーツを併用するかどうか。[`crate::field::FieldProps::has_helper_text`]
    /// と同じ役割で `aria-describedby` 合成に使う。
    pub has_helper_text: bool,
}

impl FieldsetProps<'_> {
    /// `legend` の id（`"{id}-legend"`）。
    #[must_use]
    fn legend_id(&self) -> String {
        format!("{}-legend", self.id)
    }

    /// `helper_text` の id（`"{id}-helper-text"`）。
    #[must_use]
    fn helper_text_id(&self) -> String {
        format!("{}-helper-text", self.id)
    }

    /// `error_text` の id（`"{id}-error-text"`）。
    #[must_use]
    fn error_text_id(&self) -> String {
        format!("{}-error-text", self.id)
    }

    /// Fieldset の `disabled` を内包する [`FieldProps`] へ OR 伝播する
    /// （イシュー #602）。
    ///
    /// ネイティブ `<fieldset disabled>` は子コントロールを HTML 仕様で
    /// 無効化するが、それは「実際の入力不可」であって、Field パーツの
    /// `data-disabled`/ネイティブ `disabled` 属性という**表現**は Field 側の
    /// 出力ロジック（[`FieldProps::disabled`]）が別途担うため、SSR 出力を
    /// 実際の無効化状態と整合させるには本メソッドで明示的に OR 伝播する
    /// 必要がある。`invalid` は伝播しない（本構造体 doc 参照）。
    #[must_use]
    pub fn merge_field_props<'a>(&self, mut field: FieldProps<'a>) -> FieldProps<'a> {
        field.disabled = field.disabled || self.disabled;
        field
    }
}

/// `disabled`/`invalid` フラグに対応する `data-disabled`/`data-invalid`
/// 存在属性をまとめて組み立てる。[`crate::field::state_data_attrs`] と同型の
/// 共通化（コピペによる将来のドリフトを避ける）。
#[must_use]
fn state_data_attrs(props: &FieldsetProps<'_>) -> Vec<(&'static str, &'static str)> {
    let mut attrs: Vec<(&'static str, &'static str)> = Vec::with_capacity(2);
    attrs.extend(crate::data_attrs::data_disabled(props.disabled));
    attrs.extend(crate::data_attrs::data_invalid(props.invalid));
    attrs
}

/// `aria-describedby` の合成則（[`crate::field::describedby_value`] と同型）。
///
/// `invalid` のとき error id を先頭に、`has_helper_text` のとき helper id を
/// 続けて空白区切りで連結する。どちらも無ければ `None` を返し、呼び出し側は
/// 属性自体を出力しない。
#[must_use]
fn describedby_value(props: &FieldsetProps<'_>) -> Option<String> {
    let mut ids: Vec<String> = Vec::with_capacity(2);
    if props.invalid {
        ids.push(props.error_text_id());
    }
    if props.has_helper_text {
        ids.push(props.helper_text_id());
    }
    if ids.is_empty() {
        None
    } else {
        Some(ids.join(" "))
    }
}

/// `root` パーツ（`fieldset`）。`disabled` 時にネイティブ `disabled` 存在属性
/// と `data-disabled`、`invalid` 時に `data-invalid` を付与する。
/// `aria-describedby` は `describedby_value`（非公開）の合成則に従う。
#[must_use]
pub fn root<'a>(
    props: &FieldsetProps<'_>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let described_by = describedby_value(props);
    let mut merged: Vec<(&str, &str)> = Vec::with_capacity(attrs.len() + 3);
    if props.disabled {
        merged.push(("disabled", ""));
    }
    merged.extend(state_data_attrs(props));
    if let Some(ref value) = described_by {
        merged.push(crate::aria::aria_describedby(value.as_str()));
    }
    merged.extend(attrs);
    ANATOMY.part("root", "fieldset", merged, children)
}

/// `legend` パーツ（`legend`）。`<fieldset>` 内の先頭に置くことでネイティブ
/// `<fieldset>`+`<legend>` の組がグループのアクセシブルネームを構成する
/// （モジュール doc「legend 連携」節参照。追加の `aria-labelledby` は不要）。
#[must_use]
pub fn legend(props: &FieldsetProps<'_>, attrs: Vec<(&str, &str)>, children: Vec<Node>) -> Node {
    let legend_id = props.legend_id();
    let mut merged: Vec<(&str, &str)> = vec![("id", legend_id.as_str())];
    merged.extend(state_data_attrs(props));
    merged.extend(attrs);
    ANATOMY.part("legend", "legend", merged, children)
}

/// `helper_text` パーツ（`span`）。補助説明文（バリデーション以外の
/// ヒント）を表示する。[`crate::field::helper_text`] と同型。
#[must_use]
pub fn helper_text(
    props: &FieldsetProps<'_>,
    attrs: Vec<(&str, &str)>,
    children: Vec<Node>,
) -> Node {
    let helper_id = props.helper_text_id();
    let mut merged: Vec<(&str, &str)> = vec![("id", helper_id.as_str())];
    merged.extend(state_data_attrs(props));
    merged.extend(attrs);
    ANATOMY.part("helper-text", "span", merged, children)
}

/// `error_text` パーツ（`span`）。`invalid` でないときは `hidden` 存在属性を
/// 付与する fail-closed 描画とし、JS 不在の SSR でも誤表示しない。
/// `aria-live="polite"` によりスクリーンリーダーへの通知を意図する
/// （[`crate::field::error_text`] と同型）。
#[must_use]
pub fn error_text(
    props: &FieldsetProps<'_>,
    attrs: Vec<(&str, &str)>,
    children: Vec<Node>,
) -> Node {
    let error_id = props.error_text_id();
    let mut merged: Vec<(&str, &str)> = vec![("id", error_id.as_str()), ("aria-live", "polite")];
    if !props.invalid {
        merged.push(("hidden", ""));
    }
    merged.extend(state_data_attrs(props));
    merged.extend(attrs);
    ANATOMY.part("error-text", "span", merged, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::field::{FieldIds, FieldProps};
    use fandhe_frontend_core::{render, text};

    fn base_props(id: &str) -> FieldsetProps<'_> {
        FieldsetProps {
            id,
            disabled: false,
            invalid: false,
            has_helper_text: false,
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
    fn root_reflects_disabled_and_invalid_flags() {
        let mut props = base_props("f");
        props.disabled = true;
        props.invalid = true;
        let html = render(&root(&props, vec![], vec![]));
        // invalid のため describedby_value が error id を合成し
        // aria-describedby も付与される（describedby_composition_three_cases
        // で網羅する合成則の一部）。
        assert_eq!(
            html,
            r#"<fieldset data-scope="fieldset" data-part="root" disabled="" data-disabled="" data-invalid="" aria-describedby="f-error-text"></fieldset>"#
        );
    }

    #[test]
    fn root_omits_flags_when_false() {
        let props = base_props("f");
        let html = render(&root(&props, vec![], vec![]));
        assert_eq!(
            html,
            r#"<fieldset data-scope="fieldset" data-part="root"></fieldset>"#
        );
    }

    #[test]
    fn legend_has_deterministic_id() {
        let props = base_props("f");
        let html = render(&legend(&props, vec![], vec![text("Address")]));
        assert_eq!(
            html,
            r#"<legend data-scope="fieldset" data-part="legend" id="f-legend">Address</legend>"#
        );
    }

    #[test]
    fn describedby_composition_three_cases() {
        // ケース 1: helper のみ。
        let mut props_helper_only = base_props("f");
        props_helper_only.has_helper_text = true;
        let html = render(&root(&props_helper_only, vec![], vec![]));
        assert!(html.contains(r#"aria-describedby="f-helper-text""#));

        // ケース 2: invalid + helper（error id が先頭）。
        let mut props_both = base_props("f");
        props_both.invalid = true;
        props_both.has_helper_text = true;
        let html = render(&root(&props_both, vec![], vec![]));
        assert!(html.contains(r#"aria-describedby="f-error-text f-helper-text""#));

        // ケース 3: どちらも無し → 属性自体を出力しない。
        let props_none = base_props("f");
        let html = render(&root(&props_none, vec![], vec![]));
        assert!(!html.contains("aria-describedby"));
    }

    #[test]
    fn error_text_fail_closed_hidden_defaults() {
        let props = base_props("f");
        let hidden_html = render(&error_text(&props, vec![], vec![text("bad")]));
        assert!(hidden_html.contains(r#"hidden="""#));
        assert!(hidden_html.contains(r#"aria-live="polite""#));

        let mut props_invalid = base_props("f");
        props_invalid.invalid = true;
        let visible_html = render(&error_text(&props_invalid, vec![], vec![text("bad")]));
        assert!(!visible_html.contains("hidden"));
    }

    #[test]
    fn helper_text_id_and_data_attrs() {
        let mut props = base_props("f");
        props.disabled = true;
        let html = render(&helper_text(&props, vec![], vec![text("hint")]));
        assert_eq!(
            html,
            r#"<span data-scope="fieldset" data-part="helper-text" id="f-helper-text" data-disabled="">hint</span>"#
        );
    }

    // --- merge_field_props（OR 伝播、イシュー #602） ---

    #[test]
    fn merge_field_props_or_propagates_disabled() {
        let fieldset_disabled = {
            let mut p = base_props("fs");
            p.disabled = true;
            p
        };
        let field_enabled = base_field_props("email");
        let merged = fieldset_disabled.merge_field_props(field_enabled);
        assert!(merged.disabled);
    }

    #[test]
    fn merge_field_props_both_false_stays_false() {
        let fieldset_enabled = base_props("fs");
        let field_enabled = base_field_props("email");
        let merged = fieldset_enabled.merge_field_props(field_enabled);
        assert!(!merged.disabled);
    }

    #[test]
    fn merge_field_props_field_side_true_is_preserved() {
        let fieldset_enabled = base_props("fs");
        let mut field_disabled = base_field_props("email");
        field_disabled.disabled = true;
        let merged = fieldset_enabled.merge_field_props(field_disabled);
        assert!(merged.disabled);
    }

    #[test]
    fn merge_field_props_does_not_propagate_invalid() {
        let mut fieldset_invalid = base_props("fs");
        fieldset_invalid.invalid = true;
        let field_valid = base_field_props("email");
        let merged = fieldset_invalid.merge_field_props(field_valid);
        assert!(!merged.invalid);
    }

    // --- XSS 回帰・anatomy 偽装除去（イシュー #602） ---

    #[test]
    fn xss_payload_in_id_and_children_is_escaped_on_render() {
        let payload_id = "x\" onmouseover=\"alert(1)";
        let props = base_props(payload_id);
        let html = render(&root(
            &props,
            vec![],
            vec![
                legend(&props, vec![], vec![text("<script>alert(1)</script>")]),
                helper_text(&props, vec![], vec![text("<script>alert(2)</script>")]),
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
        let props = base_props("f");
        let html = render(&root(
            &props,
            vec![("Data-Scope", "attacker"), ("DATA-PART", "attacker")],
            vec![],
        ));
        assert_eq!(
            html,
            r#"<fieldset data-scope="fieldset" data-part="root"></fieldset>"#
        );
    }
}
