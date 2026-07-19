//! 束縛点ベースの最小更新: wasm32 配線層（イシュー #343）。
//!
//! [`crate::binding`]（DOM 非依存の純粋ロジック層）が定義する
//! [`crate::binding::BindingSpec`] / [`crate::binding::BindingSource`] を
//! 実 DOM（`web-sys`）へ接続する。DOM 変異は `set_text_content` /
//! `set_attribute` / `class_list`（`DomTokenList::toggle_with_force`）の
//! 3 種別に限定し、`set_inner_html` / `insert_adjacent_html` / `raw_html` を
//! **一切呼ばない**（`lib.rs` クレート docs の不変条件 1・2・4 を、束縛点駆動の
//! 汎用経路にも適用する）。
//!
//! `#[wasm_bindgen]` エクスポートはここでは追加しない。状態は Rust 側
//! （`rws_interactive::DirtyTracked` 実装 + `BindingSource` 実装）にのみ
//! 存在し、JS 表面（dispatch → `apply_update` の配線）は `#345`（`wasm-full`
//! への統合）のスコープである。本モジュールは `#345` が rlib 経由で消費できる
//! 公開 API として設計する。

use crate::binding::{BindingKind, BindingSource, BindingSpec, BoundValue};
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use web_sys::Element;

/// `data-bind-text` / `data-bind-attr` / `data-bind-class` の 3 マーカー属性を
/// 対象とする `query_selector_all` の CSS セレクタを組み立てる。
///
/// `rws_core::{BIND_TEXT_ATTR, BIND_ATTR_ATTR, BIND_CLASS_ATTR}`（#342）を
/// 参照して構築することで、SSR 出力側（core）とクライアント側走査の属性名
/// 契約を単一箇所（core の定数）に保つ。
fn binding_selector() -> String {
    format!(
        "[{}],[{}],[{}]",
        rws_core::BIND_TEXT_ATTR,
        rws_core::BIND_ATTR_ATTR,
        rws_core::BIND_CLASS_ATTR
    )
}

/// 束縛点対応表。起動時（ハイドレーション相当の初期化）に 1 回だけ
/// [`BindingTable::scan`] で構築し、以後の更新はこの表を再利用する
/// （設計書 §3.2 の「1 回だけ走査してメモリ上に保持する」方針）。
pub struct BindingTable {
    entries: Vec<(BindingSpec, Element)>,
}

impl BindingTable {
    /// `root` 配下を [`binding_selector`] で 1 回走査し、束縛点対応表を
    /// 構築する。
    ///
    /// `data-bind-text` / `data-bind-attr` / `data-bind-class` を同一要素が
    /// 複数持つ場合（設計書 §3.1）も、[`crate::binding::element_binding_specs`]
    /// が 1 要素分の 3 属性値からまとめて [`BindingSpec`] 列を構築するため、
    /// 対応表には要素ごとに 1〜複数エントリが積まれる。
    ///
    /// # Errors
    ///
    /// `query_selector_all` が失敗した場合に `Err` を返す。エラー文字列は
    /// 固定の英語文言とし内部状態を含めない（`lib.rs` 不変条件 6 を継承）。
    pub fn scan(root: &Element) -> Result<Self, JsValue> {
        let selector = binding_selector();
        let node_list = root
            .query_selector_all(&selector)
            .map_err(|_| JsValue::from_str("query_selector_all failed for binding markers"))?;

        let mut entries = Vec::new();
        for i in 0..node_list.length() {
            let Some(node) = node_list.get(i) else {
                continue;
            };
            let Ok(element) = node.dyn_into::<Element>() else {
                continue;
            };
            let bind_text = element.get_attribute(rws_core::BIND_TEXT_ATTR);
            let bind_attr = element.get_attribute(rws_core::BIND_ATTR_ATTR);
            let bind_class = element.get_attribute(rws_core::BIND_CLASS_ATTR);
            let specs = crate::binding::element_binding_specs(
                bind_text.as_deref(),
                bind_attr.as_deref(),
                bind_class.as_deref(),
            );
            for spec in specs {
                entries.push((spec, element.clone()));
            }
        }

        Ok(Self { entries })
    }

    /// `dirty` に含まれる field の束縛点のみへ、種別ごとの DOM API を適用する
    /// （設計書 §4.3「更新駆動の流れ」の中核）。
    ///
    /// field の照合は文字列比較で行う（`dirty` 側は `&'static str` の有限
    /// 集合、対応表側は DOM から読んだ実行時 `String`。設計書 §3.2 の実装
    /// 確定）。`source.bound_value(field)` が `None`、または束縛種別と値種別が
    /// 不一致（例: class 束縛に `BoundValue::Text`）の場合は当該束縛を
    /// no-op とする（fail-closed。panic しない）。
    ///
    /// 無関係な field（`dirty` に含まれない field）の束縛点には一切触れない
    /// ため、受け入れ条件 1（無関係ノードの DOM 変異なし）を満たす。
    pub fn apply_dirty(&self, dirty: &[&'static str], source: &impl BindingSource) {
        for (spec, element) in &self.entries {
            if !dirty.iter().any(|field| *field == spec.field) {
                continue;
            }
            let Some(value) = source.bound_value(&spec.field) else {
                continue;
            };
            apply_one(&spec.kind, element, &value);
        }
    }

    /// [`rws_interactive::DirtyTracked`] 実装から `dirty_fields()` を
    /// 読み出して [`Self::apply_dirty`] を呼ぶ便宜関数（設計書 §4.3
    /// 「`update()` 直後」フローの入口）。`C` は `DirtyTracked` と
    /// [`BindingSource`] の両方を実装する呼び出し側の状態コンポーネント型
    /// （`#345` の `wasm-full` 適用層が消費する想定）。
    pub fn apply_update<C>(&self, component: &C)
    where
        C: rws_interactive::DirtyTracked + BindingSource,
    {
        self.apply_dirty(component.dirty_fields(), component);
    }
}

/// 束縛点 1 件へ値を適用する（種別ごとの DOM API 分岐、設計書 §4.1）。
///
/// - `Text`: `set_text_content`（`Node.textContent`）。`BoundValue::Flag` が
///   渡された場合は `"true"`/`"false"` として出力する。
/// - `Attr(name)`: `set_attribute`。`BoundValue::Flag` も同様に文字列化する。
/// - `Class(name)`: `class_list().toggle_with_force`。`BoundValue::Text` が
///   渡された場合（型不一致）は no-op とする（fail-closed）。
fn apply_one(kind: &BindingKind, element: &Element, value: &BoundValue) {
    match kind {
        BindingKind::Text => {
            element.set_text_content(Some(&bound_value_as_text(value)));
        }
        BindingKind::Attr(name) => {
            let _ = element.set_attribute(name, &bound_value_as_text(value));
        }
        BindingKind::Class(name) => {
            if let BoundValue::Flag(flag) = value {
                let _ = element.class_list().toggle_with_force(name, *flag);
            }
            // BoundValue::Text は class 束縛と型不一致のため no-op（fail-closed）。
        }
    }
}

/// `BoundValue` をテキスト・属性出力用の文字列へ変換する。
///
/// `Flag` の文字列化は `"true"`/`"false"` に固定する（属性束縛が真偽値
/// フィールドを扱う場合の契約、例: `aria-pressed`）。
fn bound_value_as_text(value: &BoundValue) -> String {
    match value {
        BoundValue::Text(s) => s.clone(),
        BoundValue::Flag(flag) => flag.to_string(),
    }
}
