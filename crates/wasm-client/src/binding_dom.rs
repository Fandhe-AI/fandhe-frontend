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
//! （`fandhe_frontend_interactive::DirtyTracked` 実装 + `BindingSource` 実装）にのみ
//! 存在し、JS 表面（dispatch → `apply_update` の配線）は `#345`（`wasm-full`
//! への統合）のスコープである。本モジュールは `#345` が rlib 経由で消費できる
//! 公開 API として設計する。

use crate::binding::{BindingKind, BindingSource, BindingSpec, BoundValue};
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use web_sys::{Element, HtmlInputElement};

/// `data-bind-text` / `data-bind-attr` / `data-bind-class` の 3 マーカー属性を
/// 対象とする `query_selector_all` の CSS セレクタを組み立てる。
///
/// `fandhe_frontend_core::{BIND_TEXT_ATTR, BIND_ATTR_ATTR, BIND_CLASS_ATTR}`（#342）を
/// 参照して構築することで、SSR 出力側（core）とクライアント側走査の属性名
/// 契約を単一箇所（core の定数）に保つ。
fn binding_selector() -> String {
    format!(
        "[{}],[{}],[{}]",
        fandhe_frontend_core::BIND_TEXT_ATTR,
        fandhe_frontend_core::BIND_ATTR_ATTR,
        fandhe_frontend_core::BIND_CLASS_ATTR
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
            let bind_text = element.get_attribute(fandhe_frontend_core::BIND_TEXT_ATTR);
            let bind_attr = element.get_attribute(fandhe_frontend_core::BIND_ATTR_ATTR);
            let bind_class = element.get_attribute(fandhe_frontend_core::BIND_CLASS_ATTR);
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

    /// [`fandhe_frontend_interactive::DirtyTracked`] 実装から `dirty_fields()` を
    /// 読み出して [`Self::apply_dirty`] を呼ぶ便宜関数（設計書 §4.3
    /// 「`update()` 直後」フローの入口）。`C` は `DirtyTracked` と
    /// [`BindingSource`] の両方を実装する呼び出し側の状態コンポーネント型
    /// （`#345` の `wasm-full` 適用層が消費する想定）。
    pub fn apply_update<C>(&self, component: &C)
    where
        C: fandhe_frontend_interactive::DirtyTracked + BindingSource,
    {
        self.apply_dirty(component.dirty_fields(), component);
    }

    /// `field` を対象とする束縛点が対応表に 1 件でも存在するか
    /// （イシュー #1120）。
    ///
    /// `fandhe-frontend-wasm-full` の `Runtime`（構造フォールバック、
    /// `lib.rs::apply_update_for_dirty`）が、ある dirty field が
    /// 「束縛点でもキーワード list でも処理されなかった」ことを判定する
    /// ための前提 API。画面遷移のような束縛点・keyed list のいずれにも
    /// 対応しない DOM 構造変化を検知し、全再描画フォールバックを発動する
    /// トリガー判定に使う（本クレート自体は判定結果を使わず、事実の
    /// 有無だけを返す）。
    pub fn has_field(&self, field: &str) -> bool {
        self.entries
            .iter()
            .any(|(spec, _)| spec.field.as_str() == field)
    }
}

/// 束縛点 1 件へ値を適用する（種別ごとの DOM API 分岐、設計書 §4.1）。
///
/// - `Text`: `set_text_content`（`Node.textContent`）。`BoundValue::Flag` が
///   渡された場合は `"true"`/`"false"` として出力する。
/// - `Attr(name)`: `set_attribute`。`BoundValue::Flag` も同様に文字列化する。
///   `render_into`（`fandhe-frontend-core`）と同一の URL スキーム検証（`srcset` の
///   カンマ区切り候補分割検証を含む）・イベントハンドラ属性ブロックを
///   適用する（イシュー #373。SSR 初期描画と実 DOM 直接更新の両経路に
///   同一の XSS 対策保証を持たせる契約。詳細は
///   `docs/policy/attribute-output-policy.md`）。
/// - `Class(name)`: `class_list().toggle_with_force`。`BoundValue::Text` が
///   渡された場合（型不一致）は no-op とする（fail-closed）。
fn apply_one(kind: &BindingKind, element: &Element, value: &BoundValue) {
    match kind {
        BindingKind::Text => {
            element.set_text_content(Some(&bound_value_as_text(value)));
        }
        BindingKind::Attr(name) => {
            // イベントハンドラ属性（`on*`）は束縛対象にしない。束縛点は
            // SSR 側 `render_into`（fandhe-frontend-core）が事前に発行したものに限られる
            // 契約だが、`fandhe_frontend_core` 側で `on*` は出力されないため対応表にも
            // 現れない想定である。ここでは二重の fail-closed 防御として
            // 同じ判定を適用する。
            if fandhe_frontend_core::is_event_handler_attr(name) {
                return;
            }
            let text = bound_value_as_text(value);
            // URL を受ける属性（`href`/`src` 等）は許可スキーム検証を通過
            // した値のみ反映する。不合格の場合は書き込まず、既存属性が
            // 残る不整合を避けるため `remove_attribute` で除去する
            // （fail-closed。古い安全値の残存にも決定的な挙動を与える）。
            if fandhe_frontend_core::is_url_attr(name) && !fandhe_frontend_core::is_safe_url(&text)
            {
                let _ = element.remove_attribute(name);
                return;
            }
            // `srcset` はカンマ区切りの URL 候補を持つ特殊構文のため
            // `is_url_attr` の対象外（`URL_ATTRS` に非該当）。`render_into`
            // と同一の `is_safe_srcset` で候補分割検証する（イシュー #373
            // レビュー指摘対応: 従来は `render_into` にのみ実装されており
            // 本経路では未検証だった）。
            if name.eq_ignore_ascii_case("srcset") && !fandhe_frontend_core::is_safe_srcset(&text) {
                let _ = element.remove_attribute(name);
                return;
            }
            let _ = element.set_attribute(name, &text);
            // `set_attribute("value", ...)` は HTML 属性（初期値）のみを
            // 更新し、ブラウザの live value プロパティ（`HTMLInputElement.value`）
            // には反映されない（DOM 仕様上の既知の非対称性）。これにより
            // 例えば「項目追加後に入力欄をクリアする」操作が `set_attribute`
            // だけでは効かない（イシュー #345、
            // `docs/design/dom-binding-update-design.md` #345 実装確定節）。
            // 対象要素が `HtmlInputElement` の場合のみプロパティも同期する。
            if name == "value" {
                if let Ok(input) = element.clone().dyn_into::<HtmlInputElement>() {
                    // 現在値と等しい場合は no-op とする（入力中の自己反映で
                    // キャレット位置が飛ぶ事故を防ぐ等値ガード。ユーザーが
                    // 入力途中の値と一致する限り `set_value` を呼ばない）。
                    if input.value() != text {
                        input.set_value(&text);
                    }
                }
            }
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
