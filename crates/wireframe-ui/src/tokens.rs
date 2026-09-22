//! モノクロトークン表。
//!
//! `ColorPalette`（pre-styled-ui）には依存しないグレースケール固定値
//! （`docs/design/wireframe-ui-architecture.md` §3「読みやすさ優先の
//! グレースケール」）。固定 light 値のみを持ち、`prefers-color-scheme`
//! による分岐は行わない（ダークモード対応は docs サイト側〔#2607〕の
//! 判断事項、イシュー #2605 実装計画 §8）。

/// トークン名と CSS 値の対。`:root` へのカスタムプロパティ出力は
/// [`css`] が本配列を唯一の正として生成する（二重管理しない）。
pub const TOKENS: [(&str, &str); 10] = [
    ("paper", "#ffffff"),
    ("fill-subtle", "#f5f5f5"),
    ("fill", "#e5e5e5"),
    ("line-subtle", "#cccccc"),
    ("line", "#8c8c8c"),
    ("ink-muted", "#595959"),
    ("ink", "#1f1f1f"),
    ("line-width", "1.5px"),
    ("radius", "4px"),
    (
        "font-family",
        "system-ui, -apple-system, \"Segoe UI\", sans-serif",
    ),
];

/// `TOKENS` を `:root { --fw-wire-<name>: <value>; ... }` 形式の CSS へ
/// 変換する（pre-styled-ui `Theme::to_css` の `:root` ブロック生成と
/// 同型の役割）。値はすべて定数由来であり利用者入力を含まない。
pub fn css() -> String {
    let mut out = String::from(":root {\n");
    for (name, value) in TOKENS {
        out.push_str("  --fw-wire-");
        out.push_str(name);
        out.push_str(": ");
        out.push_str(value);
        out.push_str(";\n");
    }
    out.push_str("}\n");
    out
}
