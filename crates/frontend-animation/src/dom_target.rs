//! DOM の style プロパティ・CSS カスタムプロパティへ書き込む
//! [`fandhe_animation::target::Target`] 実装（イシュー #2403/#2517）。
//!
//! # セキュリティ（A03: CSS injection）
//!
//! [`DomTarget::write`] は `CSSStyleDeclaration.setProperty(name, value)`
//! （2 引数 API）のみを使う。この API は `value` を単一のプロパティ値と
//! して扱い、`;` 等で追加の CSS 宣言を注入する経路を構造的に持たない
//! （生の CSS テキスト文字列を組み立てて代入する実装はしない）。
//! `name`/`unit` は呼び出し側（Rust コード）が渡す固定値であり、本
//! モジュールは `data-*` 属性等の信頼できない DOM 由来文字列をそのまま
//! `name` に使う経路を持たない。将来 `wasm-full` 側が `data-*` 属性から
//! プロパティ名を導出する場合は、許可リスト方式での検証が必要になる。

use fandhe_animation::target::Target;
use web_sys::HtmlElement;

enum Kind {
    /// 標準 CSS プロパティ（例: `opacity`）。数値へ付与する単位
    /// （`px`/`deg` 等、単位不要なら空文字）を保持する。
    Style { name: String, unit: String },
    /// CSS カスタムプロパティ（例: `--fandhe-motion-progress`）。
    Custom { name: String },
}

/// `HtmlElement` の style へ計算値を書き込む [`Target<f64>`] 実装。
pub struct DomTarget {
    element: HtmlElement,
    kind: Kind,
}

impl DomTarget {
    /// 標準 CSS プロパティへ書き込む `DomTarget` を作る。
    ///
    /// `unit` は値へ付与する単位（`"px"`/`"deg"` 等）。単位不要な
    /// プロパティ（`opacity` 等）は空文字を渡す。
    pub fn style_property(
        element: HtmlElement,
        name: impl Into<String>,
        unit: impl Into<String>,
    ) -> Self {
        Self {
            element,
            kind: Kind::Style {
                name: name.into(),
                unit: unit.into(),
            },
        }
    }

    /// CSS カスタムプロパティへ書き込む `DomTarget` を作る。
    pub fn custom_property(element: HtmlElement, name: impl Into<String>) -> Self {
        Self {
            element,
            kind: Kind::Custom { name: name.into() },
        }
    }
}

impl Target<f64> for DomTarget {
    fn write(&mut self, value: f64) {
        let style = self.element.style();
        // `set_property` の失敗（不正なプロパティ名等）は毎フレーム呼ばれる
        // 経路のため panic/Result 伝播せず黙って無視する（`Target::write`
        // の契約どおり）。
        match &self.kind {
            Kind::Style { name, unit } => {
                let _ = style.set_property(name, &format!("{value}{unit}"));
            }
            Kind::Custom { name } => {
                let _ = style.set_property(name, &value.to_string());
            }
        }
    }
}
