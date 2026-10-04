//! docs サイト全ページが `<head>` に出す meta CSP の単一情報源（イシュー #3678）。
//!
//! GitHub Pages はレスポンスヘッダを設定できないため、
//! `<meta http-equiv="Content-Security-Policy">` が唯一の配信手段になる。
//! `crate::layout::docs_page_with_layout`（通常ページ・トップ・404 の共通経路）が
//! [`CONTENT_SECURITY_POLICY`] を `charset`・`viewport` の直後に出し、
//! `tests/no_js_contract.rs` / `tests/layout_render.rs` が同じ定数を参照して
//! 位置・値・件数を固定する。手書きの複製を持たないため乖離しない。
//!
//! CSP は二次防御であり、一次防御（REQ-1 の既定エスケープ）は変えない。
//!
//! # ディレクティブの根拠
//!
//! - `default-src 'none'`: 明示しない取得先をすべて拒否する。`object-src` は
//!   このフォールバックで `'none'` になるため明示しない。
//! - `script-src 'self'`: `assets/theme-init.js`・`assets/site.js` のみ許可する
//!   （インライン script は #3676 で廃止済み）。`unsafe-inline`・`unsafe-eval`・
//!   ハッシュは使わない。
//! - `style-src 'self'`: 外部 stylesheet のみ許可する（インライン `<style>` は #3677 で廃止済み）。
//! - `style-src-attr 'unsafe-inline'`: 部品が出す `style` 属性（CSS 変数等）の
//!   ための唯一の緩和。残余リスクは style 注入による UI 偽装で、
//!   `img-src`・`connect-src` を同一オリジンに限ることで送出先を抑える。
//!   未対応ブラウザは `style-src 'self'` にフォールバックし `style` 属性を拒否する。
//! - `img-src 'self'`: 出力に `data:` 画像が残っていない（core の `is_safe_url` が
//!   `data:` の `src` を落とす）ため絞る。回帰は `no_js_contract.rs` が固定する。
//! - `font-src 'none'`: 外部フォントを読まない。
//! - `connect-src 'self'`: `site.js` の検索インデックス `fetch`（同一オリジン）のため。
//! - `base-uri 'none'` / `form-action 'none'`: `<base>` 書き換えとフォーム送信を禁止する。
//!
//! # meta 方式の限界
//!
//! `frame-ancestors`・`report-uri`・`report-to`・`sandbox` は meta では無効なので
//! 入れない（clickjacking は防げない）。meta より前の要素には CSP が効かない
//! ため、meta は `charset`・`viewport` の直後（`title` と全 `script`/`link` より前）に置く。
//!
//! リダイレクト案内ページ（`crate::redirect`）には付けない。script も stylesheet も
//! 持たない最小ページで、`meta refresh` との相性も考慮した判断である。
//! 設計の全体は `docs/design/docs-site-csp-policy.md` を参照。

/// 全本体ページの `<head>` に出す CSP 文字列（1 行・`; ` 区切り）。
///
/// 属性値としては通常の属性 API を通り、`'` は `&#x27;` にエスケープされて出力される
/// （ブラウザは CSP 解析前に実体参照を戻す）。テストの期待値は
/// `fandhe_frontend_core::escape_html` を通して組み立てること。
pub const CONTENT_SECURITY_POLICY: &str = "default-src 'none'; script-src 'self'; style-src 'self'; style-src-attr 'unsafe-inline'; img-src 'self'; font-src 'none'; connect-src 'self'; base-uri 'none'; form-action 'none'";

#[cfg(test)]
mod tests {
    use super::CONTENT_SECURITY_POLICY as CSP;

    fn directives() -> Vec<(&'static str, &'static str)> {
        CSP.split("; ")
            .map(|d| d.split_once(' ').expect("directive has a value"))
            .collect()
    }

    #[test]
    fn directive_names_and_values_are_fixed_in_order() {
        assert_eq!(
            directives(),
            vec![
                ("default-src", "'none'"),
                ("script-src", "'self'"),
                ("style-src", "'self'"),
                ("style-src-attr", "'unsafe-inline'"),
                ("img-src", "'self'"),
                ("font-src", "'none'"),
                ("connect-src", "'self'"),
                ("base-uri", "'none'"),
                ("form-action", "'none'"),
            ]
        );
    }

    #[test]
    fn forbidden_tokens_are_absent() {
        for bad in [
            "'unsafe-eval'",
            "'sha256-",
            "'nonce-",
            "frame-ancestors",
            "report-uri",
            "report-to",
            "sandbox",
            "\"",
            "<",
            ">",
            "\n",
            "data:",
            "*",
            "http:",
            "https:",
        ] {
            assert!(!CSP.contains(bad), "CSP must not contain {bad:?}");
        }
    }

    #[test]
    fn unsafe_inline_only_appears_in_style_src_attr() {
        for (name, value) in directives() {
            if name != "style-src-attr" {
                assert!(!value.contains("unsafe-inline"), "{name}");
            }
        }
    }
}
