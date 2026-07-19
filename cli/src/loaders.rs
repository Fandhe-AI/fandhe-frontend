//! `app::Loader` trait（イシュー #346〜#349、`docs/design/loader-trait-design.md`）の
//! 実装（`impl Loader for <Type>`）をソース文字列から検出する軽量抽出器。
//!
//! `fw impact <symbol>` の `affected_files`（`impact.rs` が走査した結果）に
//! この抽出器を通し、影響が及ぶ `Loader` 実装の型名一覧（`affected_loaders`）を
//! 構築する（`impact.rs::affected_loader_types` から呼ばれる）。Loader は
//! SSR/SSG/CSR 三モード共通のデータ取得契約であり、`affected_routes` と同格の
//! 「ユーザー可視の契約面」として扱う（実装計画 §2 の判断根拠）。
//!
//! `routes.rs`（`rws-router-v1` 抽出器）と同じ方針を踏襲する:
//! 正規表現クレートを使わず手書きの文字列走査のみで行い、抽出結果は表示のみに
//! 用いて実行・評価はしない（`cli` 外部依存ゼロ方針・security.md A03 対策）。
//!
//! # 既知の限界
//!
//! - `impl Loader for <Type>` は単一行に収まる形（このリポジトリの実装
//!   （`app/src/lib.rs`）が一貫して用いる形）のみを検出対象とする。
//!   トレイト境界・型が複数行にまたがる `impl` ブロックは検出しない
//!   （AST 解析ベースの精密化はスコープ外、`docs/design/impact-analysis-design.md` §7
//!   と同じ立場。イシュー #379 で AST 化（syn 等）の採否を検討した結果
//!   非採用と確定しており、判断根拠は `docs/policy/intentional-non-adoption.md`
//!   §3.11 を参照。複数行 `impl` の非検出は下記テスト
//!   `does_not_detect_multiline_impl_loader` で現行仕様として固定している）。
//! - ジェネリック実装（`impl<T> Loader for Foo<T>`）はジェネリックパラメータ
//!   宣言・トレイト実装対象の生ジェネリック引数を読み飛ばし、基底の型名
//!   （`Foo`）のみを抽出する。

use crate::gate::position_is_inside_string_literal;
use crate::routes::{strip_comment_lines, truncate_before_test_cfg};

/// バイト `b` が Rust 識別子を構成し得るかを判定する（`routes.rs`/`impact.rs`
/// と同じ手書き判定、正規表現クレート不使用）。
fn is_ident_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

/// `s` の先頭が balanced な `<...>`（ジェネリックパラメータ・ジェネリック引数）
/// であれば読み飛ばして残りを返す。`<` 以外で始まる場合はそのまま返す
/// （ジェネリックを持たない `impl Loader for X` を無変換で通す）。
///
/// 深さカウントのみの簡易実装で、`<` `>` を演算子として使うコード
/// （比較演算子等）と紛れる可能性は `impl` 直後というごく限定された走査位置
/// でのみ本関数を呼ぶことで実用上十分な精度に抑える。
fn skip_optional_angle_brackets(s: &str) -> &str {
    let trimmed = s.trim_start();
    let Some(rest) = trimmed.strip_prefix('<') else {
        return s;
    };
    let mut depth = 1i32;
    for (idx, ch) in rest.char_indices() {
        match ch {
            '<' => depth += 1,
            '>' => {
                depth -= 1;
                if depth == 0 {
                    return &rest[idx + ch.len_utf8()..];
                }
            }
            _ => {}
        }
    }
    // 対応する `>` が見つからない（壊れた入力）場合は読み飛ばさず元の文字列を返す。
    // 後続のパースがここで失敗し、この `impl` 出現を黙ってスキップする
    // （誤検知よりは見逃しを許容する。過検知抑制の設計判断）。
    s
}

/// `s` 先頭から Rust のパス識別子（`Foo` / `app::Loader` 等、`::` 区切り）を
/// 読み取り `(path, rest)` を返す。先頭が識別子として不正な場合は `None`。
fn parse_ident_path(s: &str) -> Option<(&str, &str)> {
    let s = s.trim_start();
    let bytes = s.as_bytes();
    if bytes.is_empty() || !(bytes[0].is_ascii_alphabetic() || bytes[0] == b'_') {
        return None;
    }
    let mut i = 0usize;
    while i < bytes.len() {
        if is_ident_byte(bytes[i]) {
            i += 1;
        } else if bytes[i] == b':' && bytes.get(i + 1) == Some(&b':') {
            i += 2;
        } else {
            break;
        }
    }
    if i == 0 {
        None
    } else {
        Some((&s[..i], &s[i..]))
    }
}

/// `impl` キーワード出現位置直後の残り文字列を解析し、`impl <trait> for <Type>`
/// の形で `<trait>` の最終セグメントが `Loader` と一致する場合に `<Type>` の
/// 最終セグメント（型名）を返す。一致しない・構文が崩れている場合は `None`
/// （`.route(` を含む無関係な呼び出しを黙ってスキップする `routes.rs` と同方針）。
fn parse_impl_for_loader(rest: &str) -> Option<String> {
    let s = skip_optional_angle_brackets(rest);
    let (trait_path, s) = parse_ident_path(s)?;
    let s = skip_optional_angle_brackets(s);
    let (keyword, s) = parse_ident_path(s)?;
    if keyword != "for" {
        return None;
    }
    let (type_path, _rest) = parse_ident_path(s)?;

    let trait_last = trait_path.rsplit("::").next().unwrap_or(trait_path);
    if trait_last != "Loader" {
        return None;
    }
    let type_last = type_path.rsplit("::").next().unwrap_or(type_path);
    Some(type_last.to_string())
}

/// ソース文字列から `impl Loader for <Type>` / `impl <path>::Loader for <Type>`
/// を検出し、型名（重複除去・昇順ソート済み）を返す。
///
/// 前処理は `routes.rs` の `extract_routes_from_source` と同じ 2 点
/// （行コメント除去・`#[cfg(test)]` 以降の切り捨て）に加え、
/// [`position_is_inside_string_literal`] で文字列リテラル内の疑似マッチ
/// （`"impl Loader for Evil"` のような文字列定数）を除外する。
///
/// `impact.rs`（TASK-13.2 の走査エンジンの後継、イシュー #353）が
/// `affected_files` の内容をこの関数に通し `affected_loaders` を構築する
/// ために `pub(crate)` として公開する。抽出結果は表示のみに用い、実行・評価は
/// 一切行わない（security.md A03 対策）。
pub(crate) fn extract_loader_impls_from_source(content: &str) -> Vec<String> {
    let filtered = strip_comment_lines(truncate_before_test_cfg(content));
    let mut results: Vec<String> = Vec::new();

    for line in filtered.lines() {
        let bytes = line.as_bytes();
        let needle = b"impl";
        let mut i = 0usize;
        while i + needle.len() <= bytes.len() {
            let is_word_boundary_before = i == 0 || !is_ident_byte(bytes[i - 1]);
            let is_word_boundary_after =
                i + needle.len() >= bytes.len() || !is_ident_byte(bytes[i + needle.len()]);
            if &bytes[i..i + needle.len()] == needle
                && is_word_boundary_before
                && is_word_boundary_after
                && !position_is_inside_string_literal(line, i)
            {
                if let Some(type_name) = parse_impl_for_loader(&line[i + needle.len()..]) {
                    results.push(type_name);
                }
            }
            i += 1;
        }
    }

    results.sort();
    results.dedup();
    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_plain_loader_impl() {
        let src = "impl Loader for DemoItemsLoader {\n    fn load(&self) {}\n}\n";
        assert_eq!(
            extract_loader_impls_from_source(src),
            vec!["DemoItemsLoader".to_string()]
        );
    }

    #[test]
    fn detects_path_qualified_trait() {
        let src = "impl app::Loader for DemoItemDetailLoader {\n}\n";
        assert_eq!(
            extract_loader_impls_from_source(src),
            vec!["DemoItemDetailLoader".to_string()]
        );
    }

    #[test]
    fn detects_generic_loader_impl() {
        let src = "impl<T: Clone> Loader for Wrapper<T> {\n}\n";
        assert_eq!(
            extract_loader_impls_from_source(src),
            vec!["Wrapper".to_string()]
        );
    }

    #[test]
    fn detects_multiple_impls_deduplicated_and_sorted() {
        let src =
            "impl Loader for BLoader {}\nimpl Loader for ALoader {}\nimpl Loader for ALoader {}\n";
        assert_eq!(
            extract_loader_impls_from_source(src),
            vec!["ALoader".to_string(), "BLoader".to_string()]
        );
    }

    #[test]
    fn ignores_unrelated_trait_impls() {
        let src = "impl Display for DemoItemsLoader {\n}\nimpl Clone for X {}\n";
        assert!(extract_loader_impls_from_source(src).is_empty());
    }

    #[test]
    fn ignores_partial_identifier_match() {
        // `LoaderX` は末尾セグメントが `Loader` と一致しないため対象外
        // （部分一致による誤検知の非回帰）。
        let src = "impl LoaderX for Something {}\n";
        assert!(extract_loader_impls_from_source(src).is_empty());
    }

    #[test]
    fn ignores_occurrence_inside_string_literal() {
        let src = "let s = \"impl Loader for Evil\";\n";
        assert!(
            extract_loader_impls_from_source(src).is_empty(),
            "string literal occurrence must not be treated as a real impl"
        );
    }

    #[test]
    fn ignores_occurrence_inside_line_comment() {
        let src = "// impl Loader for CommentedOut\nimpl Loader for RealOne {}\n";
        assert_eq!(
            extract_loader_impls_from_source(src),
            vec!["RealOne".to_string()]
        );
    }

    #[test]
    fn ignores_occurrence_after_cfg_test() {
        let src = "impl Loader for BeforeTest {}\n#[cfg(test)]\nmod tests {\n    impl Loader for InTestFixture {}\n}\n";
        assert_eq!(
            extract_loader_impls_from_source(src),
            vec!["BeforeTest".to_string()]
        );
    }

    #[test]
    fn does_not_detect_multiline_impl_loader() {
        // #379 characterization test: 偽陰性（見逃し）の実例を固定する。
        // トレイト境界・型名が改行で分割された `impl` ブロックは単一行走査
        // では検出できない（rustdoc「既知の限界」参照、
        // `docs/policy/intentional-non-adoption.md` §3.11）。本リポジトリの
        // コード規約（単一行 `impl`）が事実上この偽陰性の発生を抑えている。
        let src = "impl Loader\n    for MultilineLoader\n{\n}\n";
        assert!(
            extract_loader_impls_from_source(src).is_empty(),
            "multi-line impl header is not detected by single-line scanning (known false negative)"
        );
    }

    #[test]
    fn does_not_panic_on_arbitrary_input() {
        let inputs = ["", "impl", "impl ", "impl<", "impl Loader for", "\0\0\0"];
        for input in inputs {
            let _ = extract_loader_impls_from_source(input);
        }
    }

    #[test]
    fn extracts_real_app_crate_loader_impls() {
        // 統合的な回帰テスト: このリポジトリの実 `app/src/lib.rs` から
        // 実際に Loader 実装を抽出できること（`docs/design/loader-trait-design.md`
        // の実装、`DemoItemsLoader` / `DemoItemDetailLoader`）。
        let workspace_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("cli/ has a parent workspace root");
        let content = std::fs::read_to_string(workspace_root.join("app/src/lib.rs"))
            .expect("app/src/lib.rs should be readable");
        let found = extract_loader_impls_from_source(&content);
        assert!(found.contains(&"DemoItemsLoader".to_string()));
        assert!(found.contains(&"DemoItemDetailLoader".to_string()));
    }
}
