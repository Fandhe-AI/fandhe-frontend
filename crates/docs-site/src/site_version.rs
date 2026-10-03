//! ヘッダーのバージョン badge に出す `fandhe-frontend-core` の版数（イシュー #3606）。
//!
//! # 役割・呼び出し文脈
//!
//! [`crate::layout`] のブランド部が `core v{version}` を表示する際の唯一の供給元。
//! `crates/core/Cargo.toml` を `include_str!` でコンパイル時に取り込むため、
//! 値は手書きされず、`build_site` の `repo_root` にも依存しない（テストフィクスチャの
//! ビルドを壊さない）。cargo が manifest の変更を追跡して再ビルドする。
//!
//! # セキュリティ・fail-closed
//!
//! 解析に失敗した場合や許可文字以外を含む場合は `None` を返し、badge は出さない
//! （ライブラリコードで panic しない）。出力側は `text()` を通して既定エスケープされる。

/// `fandhe-frontend-core` の manifest 全文（コンパイル時取り込み）。
const CORE_MANIFEST: &str = include_str!("../../core/Cargo.toml");

/// `fandhe-frontend-core` の `[package].version` を返す。解析不能なら `None`。
#[must_use]
pub fn core_version() -> Option<&'static str> {
    package_version(CORE_MANIFEST)
}

/// manifest 文字列の `[package]` テーブル内で最初の `version = "…"` を返す純関数。
/// 値は数字・英字・`.`・`-`・`+` のみ許可する。
fn package_version(manifest: &str) -> Option<&str> {
    let mut in_package = false;
    for line in manifest.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_package = line == "[package]";
            continue;
        }
        if !in_package {
            continue;
        }
        let Some(rest) = line.strip_prefix("version") else {
            continue;
        };
        let Some(rest) = rest.trim_start().strip_prefix('=') else {
            continue;
        };
        let rest = rest.trim();
        let inner = rest.strip_prefix('"')?;
        let end = inner.find('"')?;
        let value = &inner[..end];
        let valid = !value.is_empty()
            && value
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '+'));
        return valid.then_some(value);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn real_manifest_yields_a_plausible_version() {
        let v = core_version().expect("core manifest should have a package version");
        assert!(v.chars().next().is_some_and(|c| c.is_ascii_digit()));
        assert!(v.contains('.'));
    }

    #[test]
    fn ignores_versions_outside_package_table() {
        let m = "[dependencies]\nfoo = { version = \"9.9.9\" }\nversion = \"8.8.8\"\n[package]\nname = \"x\"\nversion = \"1.2.3\"\n";
        assert_eq!(package_version(m), Some("1.2.3"));
        assert_eq!(
            package_version("[dependencies]\nversion = \"1.0.0\"\n"),
            None
        );
    }

    #[test]
    fn rejects_malformed_or_unsafe_values() {
        assert_eq!(package_version("[package]\nversion = \"1.0\n"), None);
        assert_eq!(package_version("[package]\nversion = 1.0.0\n"), None);
        assert_eq!(package_version("[package]\nversion = \"1.0<b>\"\n"), None);
        assert_eq!(package_version("[package]\nversion = \"\"\n"), None);
    }
}
