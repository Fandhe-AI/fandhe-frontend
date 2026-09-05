//! Switch（オン/オフ切り替え）headless コンポーネント（イシュー #537、親 #534）。
//!
//! ark-ui の Switch
//!（`.claude/skills/ark-ui/references/components/form/switch.md`）を
//! 参考に、Root / Control / Thumb / Label / HiddenInput の 5 anatomy パーツと、
//! [`crate::state::Checkable`] を埋め込んだチェック状態機械 [`Switch`] を
//! 提供する。
//!
//! # `data-state` 語彙について（[`crate::state::Checkable`] を埋め込む理由）
//!
//! [`crate::state::Disclosure`] の `data-state` 語彙は `"open"`/`"closed"` に
//! 固定されている（[`crate::state::OpenState`]）。Switch は ark-ui 準拠で
//! `"checked"`/`"unchecked"` を使うため、[`Disclosure`](crate::state::Disclosure)
//! ではなく [`crate::state::Checkable`] を埋め込む（[`crate::collapsible::Collapsible`]
//! が `Disclosure` を埋め込むのと同型の様式）。`"checked"/"unchecked"`
//! 状態機械は当初本モジュール内に個別実装していたが、`radio_group`/
//! `checkbox` との値語彙・dispatch 契約の分散を解消するため、イシュー
//! #595 で [`crate::state::Checkable`] へ共通化昇格した（本モジュールの
//! 公開 API・HTML 出力・hydration 属性は昇格前と完全互換）。
//!
//! # 呼び出し文脈
//!
//! SSR は本モジュールの自由関数（[`root`]/[`control`]/[`thumb`]/[`label`]/
//! [`hidden_input`]、純粋関数で完結）を直接呼んで組み立てる。CSR/hydration は
//! [`Switch`] を経由し、dispatch（`"check"`/`"uncheck"`/`"toggle"`）で状態
//! 遷移する。`fandhe-frontend-pre-styled-ui`（#546〜）が本モジュールを呼んで
//! スタイル済み Switch を組み立てる想定である。
//!
//! # フォーカスリング契約（`data-focus-visible`、イシュー #709）
//!
//! 実フォーカスは [`hidden_input`]（visually-hidden なネイティブ
//! `<input>`）が受けるため、視覚上の [`control`] へフォーカスリングを CSS
//! だけで伝播できない（[`root`] > [`control`] の兄弟配置であり
//! `:focus-within` も成立しない）。この静的表現として
//! [`crate::data_attrs::data_focus_visible`] を出力できる（契約は同関数の
//! doc を参照）。`fandhe-frontend-pre-styled-ui` の recipe（[`SlotRecipe::state`]
//! 相当）は同一要素上の属性有無でセレクタを組み立てるため
//! （`[data-scope="switch"][data-part="control"][data-focus-visible]`、
//! `crates/pre-styled-ui/src/switch.rs` 参照）、クライアントランタイム
//! （`fandhe-frontend-wasm-full` の focus 配線、
//! `crates/wasm-full/src/focus_visible.rs`）は [`hidden_input`] の
//! focusin/focusout と `:focus-visible` 判定に基づき、境界パーツ
//! （[`root`]）自身と、その配下で同じ `data-scope="switch"` を共有する
//! パーツ（[`control`]）の双方へ `data-focus-visible` を付け外しする
//! （単一要素にしか付与しないと `control` セレクタが一致しないため）。
//! SSR 初期マークアップでは常に属性なしで描画する。パーツ関数のシグネチャ
//! は変更しない（呼び出し側が `attrs` 引数へ `data_focus_visible(true)` を
//! 合成すれば静的掲示にも使える）。
//!
//! # 参考サイトとの意図的な差分（イシュー #1622）
//!
//! ark-ui / Radix Primitives の Switch と本実装の anatomy・`data-*`・
//! キーボード操作を突合した結果、以下は**意図的に**参考サイトへ合わせない
//! （`docs/policy/intentional-non-adoption.md` §3.25 規則 2 の一般化）。
//!
//! - **`data-hover`/`data-active`/`data-focus`**: ark-ui は Root/Control/
//!   Thumb/Label の全パーツへポインタ・フォーカスの DOM ローカル状態を
//!   `data-*` として出力するが、本実装はこれらを出力しない
//!   （`crates/headless-ui/src/checkbox.rs` #1602/#1874、
//!   `crates/headless-ui/src/radio_group.rs` #1886 と同じ判断軸）。CSS
//!   擬似クラス（`:hover`/`:active`/`:focus`）または `data-focus-visible`
//!   （wasm-full 配線）で代替する。
//! - **Enter キーでのトグル**: ark-ui / Radix はネイティブ実装（Radix は
//!   `button role="switch"`、ark-ui も内部でキー配線）で Space に加え
//!   Enter でもトグルするが、本実装は native
//!   `<input type="checkbox" role="switch">` をそのまま使うため Enter では
//!   反応しない（ブラウザ既定のチェックボックス操作）。WAI-ARIA APG の
//!   Switch パターンでは Enter は Optional 扱いであり、wasm-full 側での
//!   keydown 配線は別 issue 候補（本 issue のスコープ外）。
//! - **`readonly` 時の native トグル抑止**: `data-readonly` は表示用の
//!   状態掲示にとどまり、native checkbox のクリック・Space 操作を止める
//!   配線（`fandhe-frontend-wasm-full` の change 抑止）は持たない
//!   （`checkbox` も同じ未配線状態であり、統一的な後続対応が望ましい）。
//! - **Radix の `button role="switch"` パターン不採用**: 本実装は ark-ui /
//!   WAI-ARIA APG の「Switch Example Using HTML Checkbox Input」パターン
//!   （native checkbox + `role="switch"`）を採用しており、Radix 流の
//!   `<button>` ベース実装へは寄せない（native checked 状態が
//!   `aria-checked` へ自動マップされる利点を維持するため）。
//!
//! 一方、[`hidden_input`] へも他の 4 パーツと同じ `data-state`/
//! `data-disabled`/`data-invalid`/`data-required`/`data-readonly` を出力する
//! （ark-ui の HiddenInput は `data-state` を持たないが、`checkbox` の
//! `hidden_input` が同じ 5 属性を出力する契約に合わせる。値の真実源は
//! ネイティブ `checked`/`disabled`/`required` 属性であり、`data-*` は
//! CSS セレクタ用の補助掲示のため二重管理にはならない）。
//!
//! # セキュリティ不変条件
//!
//! - 属性名（`data-*`/`aria-*`/`type`/`role`/`name`/`checked`/`disabled`/
//!   `required`）はすべて `&'static str` リテラルで固定しており、動的値が
//!   属性名スロットへ混入する経路はない（[`crate::anatomy`]/[`crate::aria`]/
//!   [`crate::data_attrs`] の既存不変条件をそのまま継承する）。
//! - 動的値（`name`/`value`/呼び出し側 `attrs`/`children` テキスト）は
//!   [`fandhe_frontend_core::render`] の既定エスケープを必ず経由する。
//!   `raw_html()` は使用せず、HTML 文字列を直接組み立てない。
//! - `data-state` 値語彙（`"checked"`/`"unchecked"`）は [`crate::state`]
//!   （[`crate::state::checked_data_state`]）が一元管理し、本モジュールは
//!   パーツ関数間で分裂させない。
//! - hidden input は `<input type="checkbox" role="switch">`（WAI-ARIA APG
//!   の「Switch Example Using HTML Checkbox Input」パターン）。native の
//!   `checked` 状態がブラウザによって `aria-checked` へマップされるため、
//!   本モジュールは `aria-checked` を明示付与しない（二重読み上げ防止。
//!   `radio_group` の native input 方針と同型）。
//! - hydration 属性（`data-hydrate-checked`）はクライアント側で改ざんされ
//!   うる入力として扱う。[`Switch`] の
//!   [`fandhe_frontend_interactive::Hydrate`] 実装は [`crate::state::Checkable`]
//!   へ全委譲することで、panic せず `HydrateError` を返す既存保証をそのまま
//!   継承する。
//! - フレームワークが固定する属性（`data-scope`/`data-part`/`data-state`/
//!   `data-disabled`/`data-invalid`/`data-required`/`data-readonly`/
//!   `aria-hidden`/`type`/`role`/`checked`/`aria-checked`/`aria-invalid`/
//!   `name`/`value`/`disabled`/`required`）は呼び出し側 `attrs` に同名キー
//!   （ASCII 大文字小文字無視）が含まれていても fail-closed で除去し、
//!   フレームワーク値を優先する（`crates/headless-ui/src/checkbox.rs` の
//!   `STATE_RESERVED`/`drop_reserved` と同型の防御。イシュー #1622 で
//!   Switch にも導入した）。

use crate::anatomy::{anatomy, Anatomy};
use crate::aria::aria_hidden;
use crate::data_attrs::{data_disabled, data_invalid, data_readonly, data_required, data_state};
use crate::state::{checked_data_state, Checkable};
use fandhe_frontend_core::Node;
use fandhe_frontend_interactive::{Component, Hydrate, HydrateError};

/// Switch の anatomy（`data-scope="switch"`）。
const ANATOMY: Anatomy = anatomy("switch");

/// SSR 初期描画に必要な Switch の宣言的状態フラグ束
/// （[`crate::checkbox::CheckboxProps`] と同型。`checked` は含まない —
/// checked は各パーツ関数の第 1 引数として独立に受け取る既存様式を
/// [`Switch`] 昇格前から維持するため）。
///
/// イシュー #1622 で ark-ui / Radix Primitives との突合により新設した
/// （是正: `invalid`/`readonly` の追加、`disabled`/`required` の全パーツ
/// 反映への拡張）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SwitchProps {
    /// 無効化状態。`true` で全パーツへ `data-disabled` を、
    /// [`hidden_input`] へは native `disabled` も付与する。
    pub disabled: bool,
    /// 読み取り専用状態。`true` で全パーツへ `data-readonly` を付与する
    /// （native `readonly` 属性はチェックボックス/スイッチに意味を持たない
    /// ため付与しない。`crate::checkbox::CheckboxProps::readonly` と同じ
    /// 判断）。native トグル操作自体を抑止する配線は持たない
    /// （モジュール冒頭「参考サイトとの意図的な差分」節参照）。
    pub readonly: bool,
    /// 入力検証エラー状態。`true` で全パーツへ `data-invalid` を、
    /// [`hidden_input`] へは `aria-invalid="true"` も付与する。
    pub invalid: bool,
    /// 必須入力状態。`true` で全パーツへ `data-required` を、
    /// [`hidden_input`] へは native `required` も付与する。
    pub required: bool,
}

/// 全パーツ共通の `data-state`/`data-disabled`/`data-invalid`/`data-required`/
/// `data-readonly` 属性列を組み立てる非公開ヘルパ
/// （`crate::checkbox::state_attrs` と同型）。
fn state_attrs(checked: bool, props: &SwitchProps) -> Vec<(&'static str, &'static str)> {
    let mut attrs: Vec<(&'static str, &'static str)> =
        vec![data_state(checked_data_state(checked))];
    attrs.extend(data_disabled(props.disabled));
    attrs.extend(data_invalid(props.invalid));
    attrs.extend(data_required(props.required));
    attrs.extend(data_readonly(props.readonly));
    attrs
}

/// [`state_attrs`] が全パーツへ一律付与する属性キー一覧。呼び出し側 `attrs`
/// にこれらと同名キーが含まれていても fail-closed で除去する対象
/// （`crate::checkbox::STATE_RESERVED` と同型）。
const STATE_RESERVED: &[&str] = &[
    "data-state",
    "data-disabled",
    "data-invalid",
    "data-required",
    "data-readonly",
];

/// 呼び出し側 `attrs` からフレームワーク固定キー（ASCII 大文字小文字無視）を
/// 除外する（`crate::checkbox::drop_reserved` と同型）。
fn drop_reserved<'a>(
    attrs: Vec<(&'a str, &'a str)>,
    reserved: &'static [&'static str],
) -> Vec<(&'a str, &'a str)> {
    attrs
        .into_iter()
        .filter(|(k, _)| !reserved.iter().any(|r| k.eq_ignore_ascii_case(r)))
        .collect()
}

/// Root パーツ（`label`）。
///
/// 内包する [`hidden_input`] との暗黙のラベル関連付けを成立させるため
/// `<label>` 要素を使う（`for`/`id` の配線が不要になる。ark-ui と同じ方針）。
#[must_use]
pub fn root<'a>(
    checked: bool,
    props: &SwitchProps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, STATE_RESERVED);
    let mut merged = state_attrs(checked, props);
    merged.extend(attrs);
    ANATOMY.part("root", "label", merged, children)
}

/// Control パーツ（`span`）。トラック/つまみの見た目を担う装飾用パーツ。
///
/// 意味論（オン/オフ）は [`hidden_input`] の native checkbox が担うため、
/// `aria-hidden="true"` を固定付与し、支援技術の重複読み上げを防ぐ
/// （`radio_group` の control と同じ最小主義）。
#[must_use]
pub fn control<'a>(
    checked: bool,
    props: &SwitchProps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(drop_reserved(attrs, STATE_RESERVED), &["aria-hidden"]);
    let mut merged = state_attrs(checked, props);
    merged.push(aria_hidden(true));
    merged.extend(attrs);
    ANATOMY.part("control", "span", merged, children)
}

/// Thumb パーツ（`span`）。装飾用パーツ。
#[must_use]
pub fn thumb<'a>(
    checked: bool,
    props: &SwitchProps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, STATE_RESERVED);
    let mut merged = state_attrs(checked, props);
    merged.extend(attrs);
    ANATOMY.part("thumb", "span", merged, children)
}

/// Label パーツ（`span`）。ラベルテキストを表示する装飾用パーツ
/// （意味論的なラベル関連付けは [`root`] の `<label>` 要素が担う）。
#[must_use]
pub fn label<'a>(
    checked: bool,
    props: &SwitchProps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, STATE_RESERVED);
    let mut merged = state_attrs(checked, props);
    merged.extend(attrs);
    ANATOMY.part("label", "span", merged, children)
}

/// フレームワークが `hidden_input` に固定する属性キー一覧
/// （呼び出し側 `attrs` からの偽装を fail-closed で除外する対象。
/// `crate::checkbox::HIDDEN_INPUT_RESERVED` と同型）。
const HIDDEN_INPUT_RESERVED: &[&str] = &[
    "type",
    "role",
    "checked",
    "aria-checked",
    "aria-invalid",
    "name",
    "value",
    "disabled",
    "required",
];

/// HiddenInput パーツ（`input type="checkbox" role="switch"`）。
///
/// WAI-ARIA APG の「Switch Example Using HTML Checkbox Input」パターンに
/// 従い、native checkbox の checked 状態でオン/オフの意味論・フォーム送信
/// を担う（`aria-checked` は自動マップされるため明示付与しない）。
/// `checked`/`disabled`/`required` は存在属性として `true` のときのみ
/// 出力する（ark-ui 流の boolean 属性規約、[`crate::data_attrs`] と同型）。
/// `props.invalid` のとき `aria-invalid="true"` を付与する
/// （`crate::checkbox::hidden_input` と同型。イシュー #1622 で追加）。
#[must_use]
pub fn hidden_input<'a>(
    name: &'a str,
    value: &'a str,
    checked: bool,
    props: &SwitchProps,
    attrs: Vec<(&'a str, &'a str)>,
) -> Node {
    let attrs = drop_reserved(drop_reserved(attrs, STATE_RESERVED), HIDDEN_INPUT_RESERVED);
    let mut merged = state_attrs(checked, props);
    merged.push(("type", "checkbox"));
    merged.push(("role", "switch"));
    merged.push(("name", name));
    merged.push(("value", value));
    if checked {
        merged.push(("checked", ""));
    }
    if props.invalid {
        merged.push(("aria-invalid", "true"));
    }
    if props.disabled {
        merged.push(("disabled", ""));
    }
    if props.required {
        merged.push(("required", ""));
    }
    merged.extend(attrs);
    ANATOMY.part("hidden-input", "input", merged, Vec::new())
}

/// Switch のアクション（WASM 境界の文字列 dispatch と
/// [`Switch::decode_action`] で接続する）。payload は使用しない。
///
/// [`crate::state::CheckableAction`] の互換 re-export（イシュー #595 で
/// [`crate::state::Checkable`] へ状態機械を昇格した後も、既存利用箇所の
/// `SwitchAction::Check` 等の記法をそのまま使えるようにする）。
pub use crate::state::CheckableAction as SwitchAction;

/// Switch の開閉（オン/オフ）状態機械。
///
/// [`crate::state::Checkable`]（#595 で昇格した共通チェック状態機械）を
/// フィールドとして埋め込み（[`crate::collapsible::Collapsible`] が
/// [`crate::state::Disclosure`] を埋め込むのと同じ様式）、`data-state` と
/// 実際のチェック状態の整合を型レベルで保証する入口として、各パーツ関数
/// （[`root`]/[`control`]/[`thumb`]/[`label`]/[`hidden_input`]）へ
/// `self.is_checked()` を注入する利便メソッドを提供する。SSR での自由関数
/// 直接利用（本型を経由しない構成）も引き続き可能。`Default` は未チェック
/// （SSR の状態なし初期描画に対応する既定値）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Switch {
    checkable: Checkable,
}

impl Switch {
    /// `data-hydrate-checked` 属性名のフィールド部分
    /// （`docs/api/hydration-state-format.md` の `<field>` 命名規約に従う。
    /// [`Checkable::FIELD_CHECKED`] と同一値であり、hydration 属性名
    /// `data-hydrate-checked` は昇格前後で不変）。
    pub const FIELD_CHECKED: &'static str = Checkable::FIELD_CHECKED;

    /// 指定した初期状態で Switch を生成する。
    #[must_use]
    pub fn new(checked: bool) -> Self {
        Self {
            checkable: Checkable::new(checked),
        }
    }

    /// 現在チェックされているかどうか。
    #[must_use]
    pub fn is_checked(&self) -> bool {
        self.checkable.is_checked()
    }

    /// 現在の `data-state` 属性値（`"checked"`/`"unchecked"`）。
    #[must_use]
    pub fn data_state(&self) -> &'static str {
        self.checkable.data_state()
    }

    /// [`root`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn root<'a>(
        &self,
        props: &SwitchProps,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        root(self.checkable.is_checked(), props, attrs, children)
    }

    /// [`control`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn control<'a>(
        &self,
        props: &SwitchProps,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        control(self.checkable.is_checked(), props, attrs, children)
    }

    /// [`thumb`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn thumb<'a>(
        &self,
        props: &SwitchProps,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        thumb(self.checkable.is_checked(), props, attrs, children)
    }

    /// [`label`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn label<'a>(
        &self,
        props: &SwitchProps,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        label(self.checkable.is_checked(), props, attrs, children)
    }

    /// [`hidden_input`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn hidden_input<'a>(
        &self,
        name: &'a str,
        value: &'a str,
        props: &SwitchProps,
        attrs: Vec<(&'a str, &'a str)>,
    ) -> Node {
        hidden_input(name, value, self.checkable.is_checked(), props, attrs)
    }
}

impl Component for Switch {
    type Action = SwitchAction;

    fn update(&mut self, action: SwitchAction) {
        self.checkable.update(action);
    }

    /// 共通契約（`data-state` 整合・hydration ルート）のみを表す最小正準
    /// ビュー（root > control(thumb)、`name`/`value` を要する
    /// [`hidden_input`] は含めない）。公開 UI としての利用は想定しない
    /// （実際の UI 構築は §パーツ関数群を呼び出し側が組み合わせる）。
    fn view(&self) -> Node {
        let props = SwitchProps::default();
        self.root(
            &props,
            Vec::new(),
            vec![control(
                self.checkable.is_checked(),
                &props,
                Vec::new(),
                vec![thumb(
                    self.checkable.is_checked(),
                    &props,
                    Vec::new(),
                    Vec::new(),
                )],
            )],
        )
    }

    fn decode_action(name: &str, payload: &str) -> Option<SwitchAction> {
        Checkable::decode_action(name, payload)
    }
}

impl Hydrate for Switch {
    fn hydration_attrs(&self) -> Vec<(String, String)> {
        self.checkable.hydration_attrs()
    }

    fn from_hydration_attrs(attrs: &[(String, String)]) -> Result<Self, HydrateError> {
        Ok(Self {
            checkable: Checkable::from_hydration_attrs(attrs)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};
    use fandhe_frontend_interactive::{dispatch, render_for_hydration};

    fn plain() -> SwitchProps {
        SwitchProps::default()
    }

    fn disabled() -> SwitchProps {
        SwitchProps {
            disabled: true,
            ..SwitchProps::default()
        }
    }

    fn invalid_required_readonly() -> SwitchProps {
        SwitchProps {
            invalid: true,
            required: true,
            readonly: true,
            ..SwitchProps::default()
        }
    }

    // --- 各パーツの data-scope/data-part/data-state 出力 ---

    #[test]
    fn root_outputs_scope_part_and_state() {
        let html = render(&root(false, &plain(), vec![], vec![]));
        assert!(html.contains(r#"data-scope="switch""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(html.contains(r#"data-state="unchecked""#));
        assert!(html.contains("<label"));
        assert!(!html.contains("data-disabled"));
    }

    #[test]
    fn root_checked_true_outputs_checked_state() {
        let html = render(&root(true, &plain(), vec![], vec![]));
        assert!(html.contains(r#"data-state="checked""#));
    }

    #[test]
    fn root_disabled_true_adds_data_disabled() {
        let html = render(&root(true, &disabled(), vec![], vec![]));
        assert!(html.contains(r#"data-disabled="""#));
    }

    #[test]
    fn control_outputs_scope_part_state_and_aria_hidden() {
        let html = render(&control(true, &plain(), vec![], vec![]));
        assert!(html.contains(r#"data-scope="switch""#));
        assert!(html.contains(r#"data-part="control""#));
        assert!(html.contains(r#"data-state="checked""#));
        assert!(html.contains(r#"aria-hidden="true""#));
        assert!(!html.contains("aria-checked"));
        assert!(!html.contains("data-disabled"));
    }

    #[test]
    fn control_disabled_true_adds_data_disabled() {
        let html = render(&control(false, &disabled(), vec![], vec![]));
        assert!(html.contains(r#"data-disabled="""#));
    }

    #[test]
    fn thumb_outputs_scope_part_and_state_only() {
        let html = render(&thumb(true, &plain(), vec![], vec![]));
        assert!(html.contains(r#"data-scope="switch""#));
        assert!(html.contains(r#"data-part="thumb""#));
        assert!(html.contains(r#"data-state="checked""#));
    }

    #[test]
    fn label_outputs_scope_part_and_state() {
        let html = render(&label(false, &plain(), vec![], vec![text("Airplane mode")]));
        assert!(html.contains(r#"data-scope="switch""#));
        assert!(html.contains(r#"data-part="label""#));
        assert!(html.contains(r#"data-state="unchecked""#));
        assert!(html.contains("Airplane mode"));
    }

    #[test]
    fn hidden_input_outputs_type_role_name_value() {
        let html = render(&hidden_input("wifi", "on", false, &plain(), vec![]));
        assert!(html.contains(r#"data-scope="switch""#));
        assert!(html.contains(r#"data-part="hidden-input""#));
        assert!(html.contains("<input"));
        assert!(html.contains(r#"type="checkbox""#));
        assert!(html.contains(r#"role="switch""#));
        assert!(html.contains(r#"name="wifi""#));
        assert!(html.contains(r#"value="on""#));
        assert!(html.contains(r#"data-state="unchecked""#));
        assert!(!html.contains(r#"checked="""#));
        assert!(!html.contains("disabled"));
        assert!(!html.contains("required"));
    }

    #[test]
    fn hidden_input_checked_disabled_required_are_present_attrs() {
        let mut props = disabled();
        props.required = true;
        let html = render(&hidden_input("wifi", "on", true, &props, vec![]));
        assert!(html.contains(r#"checked="""#));
        assert!(html.contains(r#"disabled="""#));
        assert!(html.contains(r#"required="""#));
        assert!(html.contains(r#"data-required="""#));
    }

    #[test]
    fn hidden_input_omits_boolean_attrs_when_false() {
        let html = render(&hidden_input("wifi", "on", false, &plain(), vec![]));
        assert!(!html.contains(r#"checked="""#));
        assert!(!html.contains(r#"disabled="""#));
        assert!(!html.contains(r#"required="""#));
        assert!(!html.contains("data-required"));
    }

    #[test]
    fn hidden_input_invalid_true_adds_aria_invalid() {
        let mut props = plain();
        props.invalid = true;
        let html = render(&hidden_input("wifi", "on", false, &props, vec![]));
        assert!(html.contains(r#"aria-invalid="true""#));
        assert!(html.contains(r#"data-invalid="""#));
    }

    #[test]
    fn hidden_input_valid_omits_aria_invalid() {
        let html = render(&hidden_input("wifi", "on", false, &plain(), vec![]));
        assert!(!html.contains("aria-invalid"));
    }

    // --- イシュー #1622: SwitchProps の全パーツ反映 ---

    #[test]
    fn root_reflects_invalid_readonly_required() {
        let html = render(&root(false, &invalid_required_readonly(), vec![], vec![]));
        assert!(html.contains(r#"data-invalid="""#));
        assert!(html.contains(r#"data-readonly="""#));
        assert!(html.contains(r#"data-required="""#));
    }

    #[test]
    fn control_reflects_invalid_readonly_required() {
        let html = render(&control(
            false,
            &invalid_required_readonly(),
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-invalid="""#));
        assert!(html.contains(r#"data-readonly="""#));
        assert!(html.contains(r#"data-required="""#));
    }

    #[test]
    fn thumb_reflects_disabled_invalid_readonly_required() {
        let mut props = invalid_required_readonly();
        props.disabled = true;
        let html = render(&thumb(false, &props, vec![], vec![]));
        assert!(html.contains(r#"data-disabled="""#));
        assert!(html.contains(r#"data-invalid="""#));
        assert!(html.contains(r#"data-readonly="""#));
        assert!(html.contains(r#"data-required="""#));
    }

    #[test]
    fn label_reflects_disabled_invalid_readonly_required() {
        let mut props = invalid_required_readonly();
        props.disabled = true;
        let html = render(&label(false, &props, vec![], vec![]));
        assert!(html.contains(r#"data-disabled="""#));
        assert!(html.contains(r#"data-invalid="""#));
        assert!(html.contains(r#"data-readonly="""#));
        assert!(html.contains(r#"data-required="""#));
    }

    #[test]
    fn hidden_input_reflects_readonly_as_data_attr_without_native_readonly() {
        let html = render(&hidden_input(
            "wifi",
            "on",
            false,
            &invalid_required_readonly(),
            vec![],
        ));
        assert!(html.contains(r#"data-readonly="""#));
        assert!(!html.contains(r#" readonly"#));
    }

    // --- Anatomy::part fail-closed 回帰（呼び出し側 attrs の data-scope/data-part 偽装除去） ---

    #[test]
    fn caller_supplied_scope_and_part_are_dropped() {
        let html = render(&root(
            false,
            &plain(),
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="switch""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("attacker"));
    }

    // --- イシュー #1622: 予約キー除去（呼び出し側 attrs による状態偽装の防止） ---

    #[test]
    fn reserved_state_keys_in_caller_attrs_are_dropped_on_root() {
        let html = render(&root(
            false,
            &plain(),
            vec![
                ("data-state", "checked"),
                ("DATA-DISABLED", ""),
                ("data-invalid", ""),
                ("data-required", ""),
                ("data-readonly", ""),
            ],
            vec![],
        ));
        assert!(html.contains(r#"data-state="unchecked""#));
        assert_eq!(html.matches("data-state").count(), 1);
        assert!(!html.contains("data-disabled"));
        assert!(!html.contains("data-invalid"));
        assert!(!html.contains("data-required"));
        assert!(!html.contains("data-readonly"));
    }

    #[test]
    fn reserved_keys_in_caller_attrs_are_dropped_on_hidden_input() {
        let html = render(&hidden_input(
            "wifi",
            "on",
            false,
            &plain(),
            vec![
                ("type", "text"),
                ("ROLE", "textbox"),
                ("checked", "checked"),
                ("aria-checked", "true"),
                ("aria-invalid", "true"),
                ("name", "attacker"),
                ("value", "attacker"),
                ("disabled", ""),
                ("required", ""),
            ],
        ));
        assert!(html.contains(r#"type="checkbox""#));
        assert!(html.contains(r#"role="switch""#));
        assert!(html.contains(r#"name="wifi""#));
        assert!(html.contains(r#"value="on""#));
        assert!(!html.contains("aria-checked"));
        assert!(!html.contains("aria-invalid"));
        assert!(!html.contains("checked=\"checked\""));
        assert!(!html.contains("attacker"));
        assert!(!html.contains(r#"disabled="""#));
        assert!(!html.contains(r#"required="""#));
    }

    #[test]
    fn control_aria_hidden_cannot_be_overridden_by_caller() {
        let html = render(&control(
            false,
            &plain(),
            vec![("aria-hidden", "false")],
            vec![],
        ));
        assert!(html.contains(r#"aria-hidden="true""#));
        assert_eq!(html.matches("aria-hidden").count(), 1);
    }

    // --- Switch: dispatch 統合 ---

    #[test]
    fn switch_default_is_unchecked() {
        assert!(!Switch::default().is_checked());
    }

    #[test]
    fn switch_dispatch_toggle_changes_data_state() {
        let mut s = Switch::default();
        let props = SwitchProps::default();
        assert!(render(&s.root(&props, vec![], vec![])).contains(r#"data-state="unchecked""#));

        assert!(dispatch(&mut s, "toggle", ""));
        assert!(render(&s.root(&props, vec![], vec![])).contains(r#"data-state="checked""#));
        assert!(render(&s.control(&props, vec![], vec![])).contains(r#"data-state="checked""#));
        assert!(render(&s.thumb(&props, vec![], vec![])).contains(r#"data-state="checked""#));
        assert!(render(&s.label(&props, vec![], vec![])).contains(r#"data-state="checked""#));
        assert!(render(&s.hidden_input("wifi", "on", &props, vec![])).contains(r#"checked="""#));
    }

    #[test]
    fn switch_dispatch_check_and_uncheck() {
        let mut s = Switch::default();
        assert!(dispatch(&mut s, "check", ""));
        assert!(s.is_checked());
        assert!(dispatch(&mut s, "uncheck", ""));
        assert!(!s.is_checked());
    }

    #[test]
    fn switch_dispatch_ignores_unknown_action() {
        let mut s = Switch::new(true);
        assert!(!dispatch(&mut s, "no_such_action", "x"));
        assert!(s.is_checked());
    }

    // --- Switch: SSR 状態なし初期描画 ---

    #[test]
    fn switch_default_ssr_view_has_no_hydrate_attr() {
        let rendered = render(&Switch::default().view());
        assert!(rendered.contains(r#"data-state="unchecked""#));
        assert!(!rendered.contains("data-hydrate-"));
    }

    // --- Switch: hydration 経路 ---

    #[test]
    fn switch_hydration_round_trip() {
        let s = Switch::new(true);
        let rendered = render(&render_for_hydration(&s));
        assert!(rendered.contains(r#"data-hydrate-checked="checked""#));

        let restored = Switch::from_hydration_attrs(&s.hydration_attrs()).unwrap();
        assert_eq!(restored, s);
    }

    #[test]
    fn switch_from_hydration_attrs_missing_attr_does_not_panic() {
        let err = Switch::from_hydration_attrs(&[]).unwrap_err();
        assert_eq!(
            err,
            HydrateError::MissingAttr("data-hydrate-checked".to_string())
        );
    }

    #[test]
    fn switch_from_hydration_attrs_invalid_value_does_not_panic() {
        for bogus in ["CHECKED", "<script>alert(1)</script>", ""] {
            let attrs = vec![("data-hydrate-checked".to_string(), bogus.to_string())];
            let err = Switch::from_hydration_attrs(&attrs).unwrap_err();
            assert!(matches!(err, HydrateError::InvalidValue { .. }));
        }
    }

    // --- XSS 回帰: name/value/呼び出し側 attrs/children にペイロードを渡してもエスケープされる ---

    const ATTR_BREAK_PAYLOAD: &str = "\" onmouseover=\"alert(1)";

    #[test]
    fn hidden_input_name_value_payload_is_escaped_on_render() {
        let html = render(&hidden_input(
            ATTR_BREAK_PAYLOAD,
            ATTR_BREAK_PAYLOAD,
            false,
            &plain(),
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn caller_attrs_payload_is_escaped_on_render() {
        let html = render(&root(
            false,
            &plain(),
            vec![("data-testid", ATTR_BREAK_PAYLOAD)],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
    }

    #[test]
    fn children_text_is_escaped_on_render() {
        let html = render(&label(
            true,
            &plain(),
            vec![],
            vec![text("<script>alert(1)</script>")],
        ));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn switch_xss_payload_in_hydration_checked_is_rejected_not_rendered() {
        // data-hydrate-checked はサーバーが state_str() から生成する固定語彙の
        // みを出力するため攻撃者が任意値を注入する経路はないが、クライアント
        // 改ざん入力の復元経路（from_hydration_attrs）が未知値を拒否することを
        // Switch 経由でも固定する。
        let attrs = vec![(
            "data-hydrate-checked".to_string(),
            "<script>alert(1)</script>".to_string(),
        )];
        let err = Switch::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }
}
