//! CSS 宣言の低レベル表現とシリアライズを担うモジュール。
//!
//! [`recipe`](crate::recipe) モジュールが組み立てる slot recipe の「規則
//! （セレクタ + 宣言列）→ 静的 CSS 文字列」変換のうち、宣言単体の検証・
//! シリアライズ責務のみを本モジュールが持つ。テーマトークン層（イシュー
//! #547）が将来 CSS カスタムプロパティ値（`var(--fd-color-primary)` 等）を
//! 生成する際にも、本モジュールの検証規則（`--` プレフィックス許容）を
//! そのまま再利用できるよう、recipe 固有の概念（scope/part/variant）を
//! 一切持ち込まない中立な部品として設計している。
//!
//! # fail-closed 方針（`.claude/rules/coding-rust.md` の panic 回避規約）
//!
//! `crates/core/src/lib.rs` が不正なタグ名・属性名を「panic させず出力から
//! スキップ」する規約を踏襲し、本モジュールも不正なプロパティ名・値を
//! 黙ってスキップする（呼び出し側に `Result` を強制しない）。CSS は
//! スタイル表現であり 1 宣言の欠落が機能を壊さないため、この規約は
//! `raw_html()` のような明示的オプトインを必要とする既定エスケープ（REQ-1）
//! とは独立している。

/// CSS 宣言 1 件（`property: value;`）。
///
/// プロパティ名・値ともに `&'static str` に固定し、動的文字列を受け付けない。
/// これは `crates/headless-ui/src/anatomy.rs` の `Anatomy { scope: &'static str }`
/// と同型の判断であり、呼び出し側の実行時入力が CSS 規則へ直接混入する経路を
/// 型レベルで塞ぐ（`decl()` はソースコード中のリテラルからのみ構築される）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Declaration {
    property: &'static str,
    value: &'static str,
}

/// [`Declaration`] を構築する。
///
/// 実際に CSS へ出力されるかどうかは [`is_valid_property`] / [`is_valid_value`]
/// の検証を通過するかに依存する（本関数自体は検証を行わない）。検証は
/// シリアライズ時（[`crate::recipe::SlotRecipe::css`]）に一括で行い、
/// 不正な宣言は fail-closed にスキップする。
#[must_use]
pub const fn decl(property: &'static str, value: &'static str) -> Declaration {
    Declaration { property, value }
}

impl Declaration {
    /// プロパティ名を返す。
    #[must_use]
    pub const fn property(&self) -> &'static str {
        self.property
    }

    /// 値を返す。
    #[must_use]
    pub const fn value(&self) -> &'static str {
        self.value
    }
}

/// CSS 識別子（scope / slot / axis / variant 値）として許容する形式かどうかを
/// 判定する: `[a-z][a-z0-9-]*` の 1 文字以上。
///
/// クラス名・属性値の一部として組み込まれるため、`recipe` モジュールの
/// クラス名生成（`fd-{scope}--{axis}-{value}`）とセレクタ生成
/// （`[data-scope="..."][data-part="..."]`）の両方がこの関数で入力を検証する。
#[must_use]
pub fn is_valid_identifier(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_ascii_lowercase() => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

/// CSS プロパティ名として許容する形式かどうかを判定する。
///
/// 通常のプロパティ（`color` 等）に加え、カスタムプロパティ（`--fd-color-primary`
/// のような `--` プレフィックス）を許容する。これはイシュー #547 のテーマ
/// トークンが CSS カスタムプロパティとして宣言・参照される前提を見越した
/// ものであり、本イシュー（#548）ではカスタムプロパティの生成元は持たない
/// （値としての `var(--fd-*)` 参照のみを想定、`is_valid_value` 側で許容）。
#[must_use]
pub fn is_valid_property(s: &str) -> bool {
    let stripped = s.strip_prefix("--").unwrap_or(s);
    is_valid_identifier(stripped)
}

/// CSS 宣言値として許容できるかどうかを判定する。
///
/// `{` `}` `;` はセレクタ/宣言境界を破壊し得るため拒否し、`<` は下流
/// （#552 examples 等）が生成 CSS を `<style>` へインライン埋め込みした際に
/// `</style>` を経由して HTML コンテキストへ脱出する経路を断つために拒否する
/// （セキュリティ上の不変条件、`.claude/rules/security.md` 参照）。制御文字
/// （改行・タブ等）も出力書式の決定性を壊すため拒否する。空値も無意味な
/// 宣言として拒否する。
#[must_use]
pub fn is_valid_value(s: &str) -> bool {
    !s.is_empty()
        && !s
            .chars()
            .any(|c| c.is_control() || matches!(c, '{' | '}' | ';' | '<'))
}

/// 1 つの CSS 規則（セレクタ + 宣言列）を凍結書式でシリアライズする。
///
/// 出力書式（golden テストの前提、変更しない）:
/// - `<selector> {\n`
/// - 宣言 1 件につき `  <property>: <value>;\n`（インデント 2 スペース）
/// - `}\n`
///
/// 不正なプロパティ名・値を持つ宣言は fail-closed でスキップする（規則自体は
/// 出力する。宣言が 1 件も残らない場合は空の規則 `<selector> {\n}\n` を出力する
/// のではなく、規則ごと省略する。呼び出し側の `SlotRecipe::css` がこの関数を
/// 「有効な宣言が 1 件もない場合は呼ばない」前提で使う）。
#[must_use]
pub fn serialize_rule(selector: &str, declarations: &[Declaration]) -> Option<String> {
    let valid: Vec<&Declaration> = declarations
        .iter()
        .filter(|d| is_valid_property(d.property) && is_valid_value(d.value))
        .collect();
    if valid.is_empty() {
        return None;
    }
    let mut out = String::new();
    out.push_str(selector);
    out.push_str(" {\n");
    for d in valid {
        out.push_str("  ");
        out.push_str(d.property);
        out.push_str(": ");
        out.push_str(d.value);
        out.push_str(";\n");
    }
    out.push_str("}\n");
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identifier_accepts_lowercase_kebab() {
        assert!(is_valid_identifier("tabs"));
        assert!(is_valid_identifier("size-sm"));
        assert!(is_valid_identifier("a1"));
    }

    #[test]
    fn identifier_rejects_empty_and_invalid_chars() {
        assert!(!is_valid_identifier(""));
        assert!(!is_valid_identifier("Tabs"));
        assert!(!is_valid_identifier("1abc"));
        assert!(!is_valid_identifier("ta_bs"));
        assert!(!is_valid_identifier("ta bs"));
    }

    #[test]
    fn property_accepts_custom_property_prefix() {
        assert!(is_valid_property("color"));
        assert!(is_valid_property("--fd-color-primary"));
        assert!(!is_valid_property("--"));
        assert!(!is_valid_property("Color"));
    }

    #[test]
    fn value_rejects_structural_and_control_chars() {
        assert!(is_valid_value("var(--fd-color-primary)"));
        assert!(!is_valid_value(""));
        assert!(!is_valid_value("red; } .evil {"));
        assert!(!is_valid_value("</style><script>"));
        assert!(!is_valid_value("red\n"));
    }

    #[test]
    fn serialize_rule_formats_and_skips_invalid_declarations() {
        let decls = vec![
            decl("color", "red"),
            decl("Bad Prop", "1px"),
            decl("padding", "1px; } .evil {"),
        ];
        let out = serialize_rule("[data-scope=\"tabs\"][data-part=\"root\"]", &decls).unwrap();
        assert_eq!(
            out,
            "[data-scope=\"tabs\"][data-part=\"root\"] {\n  color: red;\n}\n"
        );
    }

    #[test]
    fn serialize_rule_returns_none_when_all_declarations_invalid() {
        let decls = vec![decl("Bad", "1px")];
        assert!(serialize_rule(".x", &decls).is_none());
    }
}
