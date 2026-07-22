//! Field コンポーネント（イシュー #538、親 #534、祖父 #525）。
//!
//! ark-ui の Field（`.claude/skills/ark-ui/references/components/form/field.md`）
//! に倣い、フォーム入力（input/textarea/select）・ラベル・補助テキスト・
//! エラーテキストを束ねるコンテナを anatomy パーツ関数群として提供する。
//!
//! anatomy は `root` / `label` / `input` / `textarea` / `select` /
//! `helper_text` / `error_text` / `required_indicator` の 8 パーツ構成
//! （ark-ui 準拠）。`input`/`textarea`/`select` は「1 Field = 1 コントロール」
//! という呼び出し側契約のもと、同じ [`FieldProps`] から同じ id
//! （`"{id}-control"`）を導出する 3 つの代替パーツであり、呼び出し側は
//! いずれか 1 つを選んで使う。
//!
//! # 状態機械を持たない理由
//!
//! [`crate::state::Disclosure`]/[`crate::state::SingleSelect`] は「開閉」
//! 「単一選択」という時間変化する状態を持つコンポーネント（Collapsible/
//! Accordion/Dialog/Popover/Tooltip 等）向けの共通状態機械である。Field の
//! `invalid`/`disabled`/`required`/`readonly` はフォームバリデーション・
//! 呼び出し側アプリケーションが決める SSR 静的な props であり、Field 自身が
//! 開閉のような内部状態遷移を持たない。そのため [`mod@crate::tabs`] と同じく
//! 「props から決定的にマークアップを組み立てる純粋関数群」として実装する
//! （状態機械の適用対象外という判断は PR 本文にも明記する）。
//!
//! # 呼び出し文脈
//!
//! - 上層の [`crate::anatomy::Anatomy`]・[`crate::aria`]・[`crate::data_attrs`]
//!   へ薄く委譲するのみで、独自の出力経路・独自のエスケープ処理は持たない。
//! - styled 層（`fandhe-frontend-pre-styled-ui`、#546）は本モジュールが
//!   出力する `data-scope="field"`/`data-part="..."` セレクタを前提にスタイル
//!   を当てる想定。
//! - クライアント側バリデーション連動（invalid フラグの動的更新・dispatch
//!   統合）は本イシューのスコープ外（後続イシュー・wasm 層の責務）。
//!
//! # セキュリティ不変条件
//!
//! - `id`/子ノード等の動的値はすべて [`fandhe_frontend_core::el`] の属性値・
//!   子ノードとして渡り、[`fandhe_frontend_core::render`] の既定エスケープ
//!   （REQ-1）を必ず経由する。本モジュールは `raw_html()` を使用しない。
//! - 属性名はすべて `&'static str` リテラルで固定されており、動的値が属性名
//!   スロットへ混入する経路はない。
//! - 派生 id（`"{id}-control"` 等）は `format!` で組み立てるが、これは属性値
//!   という**データ**の組み立てであり、`.claude/rules/coding-rust.md` が禁止
//!   する「HTML 文字列の直接組み立て」ではない（[`mod@crate::tabs`] の注記と同型）。
//! - `error_text`/`required_indicator` は非該当状態で `hidden` 存在属性を
//!   付与する fail-closed 描画とし、JS 不在の SSR でも誤表示しない。
//!
//! # ids カスタマイズ / autoresize / select readonly 解消（イシュー #602）
//!
//! - [`FieldIds`]（`FieldProps::ids`）: ark-ui `Field` の `ids: ElementIds`
//!   相当。既定（`FieldIds::default()`、全フィールド `None`）では従来どおり
//!   `"{id}-*"` 派生 id を使うが、個別に上書きすると `control_id`/`label_id`/
//!   `helper_text_id`/`error_text_id` の導出メソッドを経由する全参照箇所
//!   （label の `for`・各パーツの `id`・`aria-describedby` 合成）へ一貫伝播
//!   する。呼び出し側の再導出（別経路での id 組み立て）は作らない。
//! - [`textarea`] の `autoresize` 引数: SSR では実際の高さ調整はできないため
//!   `data-autoresize` 存在属性のみを出力する宣言的フックであり、実装は
//!   CSR/wasm 層（#580 系）または styled 層のセレクタの責務。
//! - `<select readonly>` は HTML 仕様上存在しない無効な属性であるため、
//!   [`select`] はネイティブ `readonly` を出力しない（`data-readonly` は
//!   styled 層セレクタ・CSR フック用に他コントロール同様に維持する）。実効的な
//!   読み取り専用化はアプリ側（disabled option 等）または CSR 層の責務。

use crate::anatomy::{anatomy, Anatomy};
use crate::aria::{aria_describedby, aria_hidden, aria_invalid};
use fandhe_frontend_core::Node;

/// `data-scope="field"` を固定した本コンポーネントの anatomy。
const ANATOMY: Anatomy = anatomy("field");

/// 派生 id（`"{id}-control"` 等）を個別に上書きするカスタム id 集合
/// （ark-ui `Field` の `ids: Partial<ElementIds>` 相当、イシュー #602）。
///
/// 各フィールドが `None` のとき（[`Default`]、既定挙動）は
/// [`FieldProps`] の `id` から `"{id}-*"` を導出する従来どおりの挙動になる。
/// `Some` を与えると該当パーツの id 導出メソッド（`FieldProps::control_id`
/// 等、いずれも非公開）がその値を返すようになり、`for`/`aria-describedby`
/// 等の参照箇所へも同じ値が一貫して伝播する（個別箇所での再導出は行わない）。
#[derive(Debug, Clone, Copy, Default)]
pub struct FieldIds<'a> {
    /// `root` パーツの `id` 属性。`None` のとき `root` は `id` を出力しない
    /// （従来どおり id なしで挙動不変）。
    pub root: Option<&'a str>,
    /// コントロール（input/textarea/select）の `id`。`label` の `for` 属性
    /// もこの値を参照する。
    pub control: Option<&'a str>,
    /// `label` パーツの `id`。
    pub label: Option<&'a str>,
    /// `helper_text` パーツの `id`。`aria-describedby` 合成にも使われる。
    pub helper_text: Option<&'a str>,
    /// `error_text` パーツの `id`。`aria-describedby` 合成にも使われる。
    pub error_text: Option<&'a str>,
}

/// `field` モジュールの各パーツ関数（[`root`]/[`label`]/[`input`]/[`textarea`]/
/// [`select`]/[`helper_text`]/[`error_text`]/[`required_indicator`]）へ
/// 共通で渡す props。
///
/// `invalid`/`disabled`/`required`/`readonly` は ark-ui の
/// `Field.Root` が子パーツへ配布するフラグと同じ意味論を持つ、SSR 時点の
/// 静的な状態である（動的な状態遷移は本イシューのスコープ外）。
pub struct FieldProps<'a> {
    /// ベース id。コントロール/label/helper_text/error_text の決定的 id 生成
    /// （`"{id}-control"`/`"{id}-label"`/`"{id}-helper-text"`/`"{id}-error-text"`）
    /// に使う。「1 Field = 1 コントロール」が呼び出し側の契約である。
    /// [`FieldIds`]（`ids` フィールド）で個別パーツの id を上書きしない限り
    /// この値から派生 id を導出する。
    pub id: &'a str,
    /// 派生 id の個別上書き（イシュー #602、[`FieldIds`] 参照）。既定
    /// （`FieldIds::default()`）では従来どおり `id` からの派生のみを使う。
    pub ids: FieldIds<'a>,
    /// フィールド全体の無効化。`true` のとき `root` に `data-disabled` を、
    /// コントロールパーツにネイティブ `disabled` 存在属性・`data-disabled`
    /// を付与する。
    pub disabled: bool,
    /// 入力値が不正であることを示す。`true` のとき `root`/コントロール
    /// パーツに `data-invalid` を、コントロールパーツに
    /// `aria-invalid="true"` を付与し、`error_text` を表示状態にする。
    pub invalid: bool,
    /// 必須入力。`true` のとき `root` に `data-required` を、コントロール
    /// パーツにネイティブ `required` 存在属性・`data-required` を付与し、
    /// `required_indicator` を表示状態にする。
    pub required: bool,
    /// 読み取り専用。`true` のとき `root` に `data-readonly` を、コントロール
    /// パーツにネイティブ `readonly` 存在属性・`data-readonly` を付与する。
    /// ただし [`select`] は HTML 仕様上 `<select readonly>` が無効なため
    /// ネイティブ属性は出力しない（`data-readonly` は出力する。モジュール
    /// doc 「select readonly 解消」節参照）。
    pub readonly: bool,
    /// `helper_text` パーツを併用するかどうか。`true` のとき
    /// `aria-describedby`（`invalid` なら error id を先頭に、続けて helper id
    /// を空白区切りで連結。詳細は [`input`] の rustdoc 参照）に helper id を
    /// 含める。呼び出し側が `helper_text` パーツを実際に描画するかどうかと
    /// 整合させることが呼び出し側の契約である。
    pub has_helper_text: bool,
}

/// コントロールパーツの種別（イシュー #602）。
///
/// `<select readonly>` が HTML 仕様上無効な属性であるため、
/// [`FieldProps::control_attrs`] がネイティブ `readonly` を出力するか否かを
/// 分岐するためだけに使う内部区分。これ以外の属性則（`disabled`/`required`/
/// `aria-invalid`/data-*）は 3 種で共通のため列挙子はこの 1 点にのみ影響する。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ControlKind {
    Input,
    Textarea,
    Select,
}

impl FieldProps<'_> {
    /// コントロール（input/textarea/select）が共有する id
    /// （既定 `"{id}-control"`、[`FieldIds::control`] で上書き可能）。
    /// label の `for` 属性もこの id を参照する。
    #[must_use]
    fn control_id(&self) -> String {
        self.ids
            .control
            .map(str::to_string)
            .unwrap_or_else(|| format!("{}-control", self.id))
    }

    /// label の id（既定 `"{id}-label"`、[`FieldIds::label`] で上書き可能）。
    #[must_use]
    fn label_id(&self) -> String {
        self.ids
            .label
            .map(str::to_string)
            .unwrap_or_else(|| format!("{}-label", self.id))
    }

    /// helper_text の id（既定 `"{id}-helper-text"`、[`FieldIds::helper_text`]
    /// で上書き可能）。
    #[must_use]
    fn helper_text_id(&self) -> String {
        self.ids
            .helper_text
            .map(str::to_string)
            .unwrap_or_else(|| format!("{}-helper-text", self.id))
    }

    /// error_text の id（既定 `"{id}-error-text"`、[`FieldIds::error_text`]
    /// で上書き可能）。
    #[must_use]
    fn error_text_id(&self) -> String {
        self.ids
            .error_text
            .map(str::to_string)
            .unwrap_or_else(|| format!("{}-error-text", self.id))
    }

    /// コントロールパーツ（input/textarea/select）に共通する属性列を
    /// 組み立てる（`id`・ネイティブ存在属性・`aria-invalid`・
    /// `aria-describedby`・data-* 4 種）。`extra_attrs` は呼び出し側が
    /// 追加で渡す属性（`name`/`type`/`value` 等）で、末尾に連結する。
    ///
    /// `kind` が [`ControlKind::Select`] のときのみネイティブ `readonly` の
    /// 出力を止める（select readonly 解消、イシュー #602）。それ以外の
    /// 属性則は 3 種共通。
    ///
    /// `self` は戻り値へ実際には借用を持ち越さない（`state_data_attrs`/
    /// `aria_invalid` はいずれも `'static` を返す）ため、`self` は出力
    /// ライフタイム `'a` に紐付けない（無名ライフタイムで受ける）。これに
    /// より [`input`]/[`textarea`]/[`select`] を [`root`]/[`label`] と同じ
    /// `&FieldProps<'_>` で受けられ、長寿命の props と短寿命の
    /// `extra_attrs` を組み合わせても不要な借用チェッカーエラーに当たらない
    /// （PR #567 レビュー指摘の是正）。
    fn control_attrs<'a>(
        &self,
        kind: ControlKind,
        control_id: &'a str,
        extra_attrs: Vec<(&'a str, &'a str)>,
    ) -> Vec<(&'a str, &'a str)> {
        let mut attrs: Vec<(&str, &str)> = vec![("id", control_id)];
        if self.disabled {
            attrs.push(("disabled", ""));
        }
        if self.required {
            attrs.push(("required", ""));
        }
        if self.readonly && kind != ControlKind::Select {
            attrs.push(("readonly", ""));
        }
        if self.invalid {
            attrs.push(aria_invalid(true));
        }
        attrs.extend(state_data_attrs(self));
        attrs.extend(extra_attrs);
        attrs
    }
}

/// `disabled`/`invalid`/`required`/`readonly` フラグに対応する
/// `data-disabled`/`data-invalid`/`data-required`/`data-readonly` 存在属性を
/// まとめて組み立てる。`root`/`label`/`control_attrs`/`helper_text`/
/// `error_text`/`required_indicator` の 6 箇所が同一のフラグ集合を data-*
/// へ写像するため、コピペによる将来のドリフト（フラグ変更時の同期漏れ）を
/// 避けるために共通化する。
#[must_use]
fn state_data_attrs(props: &FieldProps<'_>) -> Vec<(&'static str, &'static str)> {
    let mut attrs: Vec<(&'static str, &'static str)> = Vec::with_capacity(4);
    attrs.extend(crate::data_attrs::data_disabled(props.disabled));
    attrs.extend(crate::data_attrs::data_invalid(props.invalid));
    attrs.extend(crate::data_attrs::data_required(props.required));
    attrs.extend(crate::data_attrs::data_readonly(props.readonly));
    attrs
}

/// `aria-describedby` の合成則（zag.js Field の実装意味論を SSR 静的に写像）。
///
/// `invalid` のとき error id を先頭に、`has_helper_text` のとき helper id を
/// 続けて空白区切りで連結する。どちらも無ければ `None` を返し、呼び出し側は
/// 属性自体を出力しない。
#[must_use]
fn describedby_value(props: &FieldProps<'_>) -> Option<String> {
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

/// `root` パーツ（`div`）。`disabled`/`invalid`/`required`/`readonly` の
/// data-* フラグを反映する。[`FieldIds::root`] が `Some` のときのみ `id`
/// 属性を出力する（`None` のときは従来どおり id なし、イシュー #602）。
#[must_use]
pub fn root<'a>(
    props: &FieldProps<'_>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let root_id = props.ids.root;
    let mut merged: Vec<(&str, &str)> = Vec::with_capacity(attrs.len() + 5);
    if let Some(id) = root_id {
        merged.push(("id", id));
    }
    merged.extend(state_data_attrs(props));
    merged.extend(attrs);
    ANATOMY.part("root", "div", merged, children)
}

/// `label` パーツ（`label`）。`for`/`id` はコントロール/label の id と
/// 決定的に対応する。
#[must_use]
pub fn label(props: &FieldProps<'_>, attrs: Vec<(&str, &str)>, children: Vec<Node>) -> Node {
    let control_id = props.control_id();
    let label_id = props.label_id();
    let mut merged: Vec<(&str, &str)> =
        vec![("for", control_id.as_str()), ("id", label_id.as_str())];
    merged.extend(state_data_attrs(props));
    merged.extend(attrs);
    ANATOMY.part("label", "label", merged, children)
}

/// `input` パーツ（`input`）。`extra_attrs` に `type`/`name`/`value` 等、
/// 呼び出し側が必要とする追加属性を渡す。
///
/// `aria-describedby` は次の合成則で決定的に組み立てる（zag.js Field の
/// 実装意味論を SSR 静的に写像）: `props.invalid` のとき error id を先頭に、
/// `props.has_helper_text` のとき helper id を続けて空白区切りで連結する。
/// どちらも `false` なら属性自体を出力しない。[`textarea`]/[`select`] も
/// 同じ合成則に従う。
#[must_use]
pub fn input<'a>(props: &FieldProps<'_>, extra_attrs: Vec<(&'a str, &'a str)>) -> Node {
    let control_id = props.control_id();
    let described_by = describedby_value(props);
    let mut attrs = props.control_attrs(ControlKind::Input, &control_id, extra_attrs);
    if let Some(ref value) = described_by {
        attrs.push(aria_describedby(value.as_str()));
    }
    ANATOMY.part("input", "input", attrs, vec![])
}

/// `textarea` パーツ（`textarea`）。[`input`] と同一の属性則に従う。
///
/// `autoresize`: ark-ui `Field.Textarea` の `autoresize` 相当（イシュー
/// #602）。`true` のとき `data-autoresize=""` 存在属性を付与する宣言的
/// フックのみを出力する。SSR は実際の高さ調整を行わない（実装は CSR/wasm 層
/// （#580 系）または styled 層のセレクタの責務。モジュール doc 参照）。
#[must_use]
pub fn textarea<'a>(
    props: &FieldProps<'_>,
    autoresize: bool,
    extra_attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let control_id = props.control_id();
    let described_by = describedby_value(props);
    let mut attrs = props.control_attrs(ControlKind::Textarea, &control_id, extra_attrs);
    if let Some(ref value) = described_by {
        attrs.push(aria_describedby(value.as_str()));
    }
    if autoresize {
        attrs.push(("data-autoresize", ""));
    }
    ANATOMY.part("textarea", "textarea", attrs, children)
}

/// `select` パーツ（`select`）。[`input`] と同一の属性則に従うが、
/// `props.readonly` が `true` でもネイティブ `readonly` 属性は出力しない
/// （HTML 仕様上 `<select readonly>` は無効な属性のため、イシュー #602 で
/// 解消。`data-readonly` は他コントロールと同様に出力する）。実効的な
/// 読み取り専用化はアプリ側（disabled option 等）または CSR 層の責務。
///
/// `option` 子ノードは呼び出し側が [`fandhe_frontend_core::el`]（例:
/// `el("option", ..., ...)`）で組み立てて `children` に渡す（`core` が
/// `select` ショートカットタグを意図的に持たない経緯は `crates/core/src/tags.rs`
/// 冒頭 doc 参照）。本関数は `field::select` とモジュール修飾で呼ばれるため、
/// `core` の他タグ関数との混同リスクは低い。
#[must_use]
pub fn select<'a>(
    props: &FieldProps<'_>,
    extra_attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let control_id = props.control_id();
    let described_by = describedby_value(props);
    let mut attrs = props.control_attrs(ControlKind::Select, &control_id, extra_attrs);
    if let Some(ref value) = described_by {
        attrs.push(aria_describedby(value.as_str()));
    }
    ANATOMY.part("select", "select", attrs, children)
}

/// `helper_text` パーツ（`span`）。補助説明文（バリデーション以外の
/// ヒント）を表示する。
#[must_use]
pub fn helper_text(props: &FieldProps<'_>, attrs: Vec<(&str, &str)>, children: Vec<Node>) -> Node {
    let helper_id = props.helper_text_id();
    let mut merged: Vec<(&str, &str)> = vec![("id", helper_id.as_str())];
    merged.extend(state_data_attrs(props));
    merged.extend(attrs);
    ANATOMY.part("helper-text", "span", merged, children)
}

/// `error_text` パーツ（`span`）。`invalid` でないときは `hidden` 存在属性を
/// 付与する fail-closed 描画とし、JS 不在の SSR でも誤表示しない。
/// `aria-live="polite"` によりスクリーンリーダーへの通知を意図する。
#[must_use]
pub fn error_text(props: &FieldProps<'_>, attrs: Vec<(&str, &str)>, children: Vec<Node>) -> Node {
    let error_id = props.error_text_id();
    let mut merged: Vec<(&str, &str)> = vec![("id", error_id.as_str()), ("aria-live", "polite")];
    if !props.invalid {
        merged.push(("hidden", ""));
    }
    merged.extend(state_data_attrs(props));
    merged.extend(attrs);
    ANATOMY.part("error-text", "span", merged, children)
}

/// `required_indicator` パーツ（`span`）。装飾目的の印であるため
/// `aria-hidden="true"` を常に付与し、`required` でないときは `hidden`
/// 存在属性を付与する fail-closed 描画とする。
#[must_use]
pub fn required_indicator(
    props: &FieldProps<'_>,
    attrs: Vec<(&str, &str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&str, &str)> = vec![aria_hidden(true)];
    if !props.required {
        merged.push(("hidden", ""));
    }
    merged.extend(state_data_attrs(props));
    merged.extend(attrs);
    ANATOMY.part("required-indicator", "span", merged, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};

    fn base_props(id: &str) -> FieldProps<'_> {
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
    fn root_reflects_all_four_flags() {
        let mut props = base_props("f");
        props.disabled = true;
        props.invalid = true;
        props.required = true;
        props.readonly = true;
        let node = root(&props, vec![], vec![]);
        let html = render(&node);
        assert_eq!(
            html,
            r#"<div data-scope="field" data-part="root" data-disabled="" data-invalid="" data-required="" data-readonly=""></div>"#
        );
    }

    #[test]
    fn root_omits_flags_when_false() {
        let props = base_props("f");
        let node = root(&props, vec![], vec![]);
        assert_eq!(
            render(&node),
            r#"<div data-scope="field" data-part="root"></div>"#
        );
    }

    #[test]
    fn label_for_matches_control_id() {
        let props = base_props("f");
        let node = label(&props, vec![], vec![text("Name")]);
        assert_eq!(
            render(&node),
            r#"<label data-scope="field" data-part="label" for="f-control" id="f-label">Name</label>"#
        );
    }

    #[test]
    fn input_native_presence_attrs_and_aria_invalid_only_when_true() {
        let mut props = base_props("f");
        props.disabled = true;
        props.required = true;
        props.readonly = true;
        props.invalid = true;
        let node = input(&props, vec![("type", "text")]);
        let html = render(&node);
        assert!(html.contains(r#"id="f-control""#));
        assert!(html.contains(r#"disabled="""#));
        assert!(html.contains(r#"required="""#));
        assert!(html.contains(r#"readonly="""#));
        assert!(html.contains(r#"aria-invalid="true""#));
        assert!(html.contains(r#"type="text""#));
    }

    #[test]
    fn input_omits_native_attrs_and_aria_invalid_when_all_false() {
        let props = base_props("f");
        let node = input(&props, vec![]);
        let html = render(&node);
        assert!(!html.contains("disabled"));
        assert!(!html.contains("required"));
        assert!(!html.contains("readonly"));
        assert!(!html.contains("aria-invalid"));
    }

    #[test]
    fn describedby_composition_helper_only() {
        let mut props = base_props("f");
        props.has_helper_text = true;
        let node = input(&props, vec![]);
        let html = render(&node);
        assert!(html.contains(r#"aria-describedby="f-helper-text""#));
    }

    #[test]
    fn describedby_composition_invalid_and_helper() {
        let mut props = base_props("f");
        props.invalid = true;
        props.has_helper_text = true;
        let node = input(&props, vec![]);
        let html = render(&node);
        assert!(html.contains(r#"aria-describedby="f-error-text f-helper-text""#));
    }

    #[test]
    fn describedby_composition_neither_omits_attribute() {
        let props = base_props("f");
        let node = input(&props, vec![]);
        let html = render(&node);
        assert!(!html.contains("aria-describedby"));
    }

    #[test]
    fn textarea_and_select_share_input_attribute_rules() {
        let mut props = base_props("f");
        props.invalid = true;
        let ta = render(&textarea(&props, false, vec![], vec![]));
        assert!(ta.contains(r#"data-scope="field" data-part="textarea""#));
        assert!(ta.contains(r#"id="f-control""#));
        assert!(ta.contains(r#"aria-invalid="true""#));

        let sel = render(&select(&props, vec![], vec![]));
        assert!(sel.contains(r#"data-scope="field" data-part="select""#));
        assert!(sel.contains(r#"id="f-control""#));
        assert!(sel.contains(r#"aria-invalid="true""#));
    }

    // --- select readonly 解消（イシュー #602） ---

    #[test]
    fn select_omits_native_readonly_but_keeps_data_readonly() {
        let mut props = base_props("f");
        props.readonly = true;
        let sel_html = render(&select(&props, vec![], vec![]));
        // `data-readonly=""` を含む文字列全体には部分文字列として
        // `readonly=""` が含まれてしまうため、ネイティブ属性（先頭が空白
        // かクォートで区切られる） `" readonly=\"\""` の非存在で検証する。
        assert!(!sel_html.contains(r#" readonly="""#));
        assert!(sel_html.contains(r#"data-readonly=""#));

        // input/textarea は従来どおりネイティブ readonly を出力する（回帰）。
        let input_html = render(&input(&props, vec![]));
        assert!(input_html.contains(r#"readonly="""#));
        let ta_html = render(&textarea(&props, false, vec![], vec![]));
        assert!(ta_html.contains(r#"readonly="""#));
    }

    // --- autoresize（イシュー #602） ---

    #[test]
    fn textarea_autoresize_true_emits_data_autoresize() {
        let props = base_props("f");
        let html = render(&textarea(&props, true, vec![], vec![]));
        assert!(html.contains(r#"data-autoresize=""#));
    }

    #[test]
    fn textarea_autoresize_false_omits_data_autoresize() {
        let props = base_props("f");
        let html = render(&textarea(&props, false, vec![], vec![]));
        assert!(!html.contains("data-autoresize"));
    }

    // --- FieldIds（イシュー #602） ---

    #[test]
    fn ids_all_none_preserves_default_derivation() {
        let props = base_props("f");
        let root_html = render(&root(&props, vec![], vec![]));
        assert_eq!(
            root_html,
            r#"<div data-scope="field" data-part="root"></div>"#
        );
        let label_html = render(&label(&props, vec![], vec![text("Name")]));
        assert_eq!(
            label_html,
            r#"<label data-scope="field" data-part="label" for="f-control" id="f-label">Name</label>"#
        );
    }

    #[test]
    fn ids_control_override_propagates_to_input_and_label_for() {
        let mut props = base_props("f");
        props.ids.control = Some("custom-control");
        let input_html = render(&input(&props, vec![]));
        assert!(input_html.contains(r#"id="custom-control""#));
        let label_html = render(&label(&props, vec![], vec![text("Name")]));
        assert!(label_html.contains(r#"for="custom-control""#));
        // label 自身の id は上書き対象外（別フィールド）のため従来どおり。
        assert!(label_html.contains(r#"id="f-label""#));
    }

    #[test]
    fn ids_error_and_helper_override_propagate_to_describedby() {
        let mut props = base_props("f");
        props.invalid = true;
        props.has_helper_text = true;
        props.ids.error_text = Some("custom-error");
        props.ids.helper_text = Some("custom-helper");
        let html = render(&input(&props, vec![]));
        assert!(html.contains(r#"aria-describedby="custom-error custom-helper""#));

        let error_html = render(&error_text(&props, vec![], vec![text("bad")]));
        assert!(error_html.contains(r#"id="custom-error""#));
        let helper_html = render(&helper_text(&props, vec![], vec![text("hint")]));
        assert!(helper_html.contains(r#"id="custom-helper""#));
    }

    #[test]
    fn ids_root_some_emits_id_none_omits_it() {
        let props_default = base_props("f");
        let default_html = render(&root(&props_default, vec![], vec![]));
        assert!(!default_html.contains(" id="));

        let mut props_with_root_id = base_props("f");
        props_with_root_id.ids.root = Some("custom-root");
        let with_id_html = render(&root(&props_with_root_id, vec![], vec![]));
        assert!(with_id_html.contains(r#"id="custom-root""#));
    }

    #[test]
    fn helper_text_id_and_data_attrs() {
        let mut props = base_props("f");
        props.disabled = true;
        let node = helper_text(&props, vec![], vec![text("hint")]);
        assert_eq!(
            render(&node),
            r#"<span data-scope="field" data-part="helper-text" id="f-helper-text" data-disabled="">hint</span>"#
        );
    }

    #[test]
    fn error_text_hidden_when_valid_visible_when_invalid() {
        let props_valid = base_props("f");
        let hidden_html = render(&error_text(&props_valid, vec![], vec![text("bad")]));
        assert!(hidden_html.contains(r#"hidden="""#));
        assert!(hidden_html.contains(r#"aria-live="polite""#));

        let mut props_invalid = base_props("f");
        props_invalid.invalid = true;
        let visible_html = render(&error_text(&props_invalid, vec![], vec![text("bad")]));
        assert!(!visible_html.contains("hidden"));
    }

    #[test]
    fn required_indicator_hidden_unless_required_and_always_aria_hidden() {
        let props_optional = base_props("f");
        let hidden_html = render(&required_indicator(
            &props_optional,
            vec![],
            vec![text("*")],
        ));
        assert!(hidden_html.contains(r#"hidden="""#));
        assert!(hidden_html.contains(r#"aria-hidden="true""#));

        let mut props_required = base_props("f");
        props_required.required = true;
        let visible_html = render(&required_indicator(
            &props_required,
            vec![],
            vec![text("*")],
        ));
        assert!(!visible_html.contains(r#"hidden="""#));
        assert!(visible_html.contains(r#"aria-hidden="true""#));
    }

    // --- XSS 回帰: id/attrs/children に攻撃者制御文字列が入っても既定エスケープが効く ---

    #[test]
    fn xss_payload_in_id_and_children_is_escaped_on_render() {
        let payload_id = "x\" onmouseover=\"alert(1)";
        let props = base_props(payload_id);
        let node = root(
            &props,
            vec![],
            vec![
                label(&props, vec![], vec![text("<script>alert(1)</script>")]),
                input(&props, vec![("value", "<script>alert(2)</script>")]),
            ],
        );
        let html = render(&node);
        assert!(!html.contains("<script>alert"));
        assert!(!html.contains("onmouseover=\"alert"));
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
        assert!(html.contains("&lt;script&gt;alert(2)&lt;/script&gt;"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn caller_supplied_data_scope_and_part_are_dropped_fail_closed() {
        let props = base_props("f");
        let node = root(
            &props,
            vec![("Data-Scope", "attacker"), ("DATA-PART", "attacker")],
            vec![],
        );
        assert_eq!(
            render(&node),
            r#"<div data-scope="field" data-part="root"></div>"#
        );
    }

    #[test]
    fn xss_payload_in_ids_override_is_escaped_on_render() {
        // FieldIds（イシュー #602）で上書きされた id 値も root/control/label と
        // 同じ既定エスケープ経路（render()）を通ることを固定する。
        let payload = "x\" onmouseover=\"alert(1)";
        let mut props = base_props("f");
        props.ids.root = Some(payload);
        props.ids.control = Some(payload);
        props.ids.label = Some(payload);
        props.ids.helper_text = Some(payload);
        props.ids.error_text = Some(payload);
        props.invalid = true;
        props.has_helper_text = true;

        let html = render(&root(
            &props,
            vec![],
            vec![
                label(&props, vec![], vec![text("Name")]),
                input(&props, vec![]),
                helper_text(&props, vec![], vec![text("hint")]),
                error_text(&props, vec![], vec![text("bad")]),
            ],
        ));
        assert!(!html.contains("onmouseover=\"alert"));
        assert!(html.contains("&quot;"));
    }
}
