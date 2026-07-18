//! 変更影響範囲解析（TASK-13.2a, #133、親 TASK-13.2 #132、REQ-13）の型定義と
//! 判定ロジック。`main.rs` の `impact` サブコマンド（TASK-13.2c, #135）から
//! [`analyze`] を呼び出して使われる。
//!
//! 本ファイルは TASK-13.2 の 5 分割サブタスク（#133 設計（本ファイル）/
//! #134 依存グラフ構築 / #135 コマンド実装（本ファイルの [`analyze`] 接続） /
//! #136 出力フォーマット / #137 テスト整備）が依拠する**単一の情報源**として、
//! `fw impact <symbol>` が返す JSON の形（[`ImpactReport`]）と、
//! `breaking_risk` / `requires_human_approval` の判定を副作用のない純粋関数
//! （[`judge_breaking_risk`] / [`requires_human_approval`]）として切り出す。
//!
//! **#134（依存グラフ構築・使用箇所走査）は本 PR 時点で未マージ**のため、
//! [`analyze`] は暫定的に常に [`ImpactError::Scan`] を返す fail-closed スタブ
//! （`docs/impact-analysis-design.md` §4 の「ケース B」）。#134 マージ後、
//! 実際の走査結果から [`ImpactReport`] を構築する実装へ #134 側が置き換える
//! 契約とする。黙示的成功（空レポートでの exit 0）には倒さない
//! （security.md A05）。
//!
//! アルゴリズムは PoC-7（`docs/spec/03-poc/ai-self-maintenance/tools/poc7_tool.py`
//! `cmd_impact`）を踏襲した「ファイル単位の粗粒度ヒューリスティック」であり、
//! AST 解析ベースの精密化は将来スコープとして明示的に見送る
//! （`docs/spec/05-tasks.md` TASK-13.2、`docs/impact-analysis-design.md` §7）。

use std::fmt;
use std::path::Path;

/// 破壊的変更リスクの粗粒度分類。
///
/// 判定は [`judge_breaking_risk`] が一元的に行う。PoC-7 と同じ 3 段階
/// （`high` / `medium` / `low`）を製品仕様として踏襲する
/// （`docs/impact-analysis-design.md` §3.4）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
// #134（依存グラフ構築）が `judge_breaking_risk` 経由でこれらの variant を
// 実際に構築するまで未使用。`analyze` の fail-closed スタブは `Ok` を返さない
// ため、現状は `BreakingRisk` の値自体が生成されない（撤去予定）。
#[allow(dead_code)]
pub enum BreakingRisk {
    High,
    Medium,
    Low,
}

impl BreakingRisk {
    /// JSON 出力（#136）が参照する固定文字列表現。
    pub const fn as_str(self) -> &'static str {
        match self {
            BreakingRisk::High => "high",
            BreakingRisk::Medium => "medium",
            BreakingRisk::Low => "low",
        }
    }
}

impl fmt::Display for BreakingRisk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// クライアント境界クレート（WASM 経由でブラウザに配布される 3 クレート）。
///
/// PoC-7 は `rws-wasm-client` 単独を「クライアント境界」として `high` 判定の
/// 追加条件に使っていたが、製品ワークスペースは WASM 配布クレートが
/// `rws-wasm-client` / `rws-wasm-full` / `rws-wasm-thin` の 3 つに分かれている
/// （`docs/spec/06-roadmap.md` MS-3〜MS-4）。いずれかへの波及でも `high` 側に
/// 倒す（安全側・過検知容認、`docs/impact-analysis-design.md` §3.2）。
///
/// #134（依存グラフ構築）・#135（CLI 接続）はここを参照し、実測した
/// `affected_crates` との突き合わせに同じ定数を使う契約とする（二重管理しない）。
///
/// `judge_breaking_risk` からのみ参照され、同関数は `analyze` の fail-closed
/// スタブ（`Ok` を返さない）からは呼ばれないため、#134 が走査結果を
/// `judge_breaking_risk` に渡すまで未使用（撤去予定）。
#[allow(dead_code)]
pub const CLIENT_BOUNDARY_CRATES: [&str; 3] = ["rws-wasm-client", "rws-wasm-full", "rws-wasm-thin"];

/// `high` 判定となる影響クレート数の下限（この件数**以上**で `high`）。
#[allow(dead_code)] // 上記 CLIENT_BOUNDARY_CRATES と同じ理由（#134 未接続）。
const HIGH_RISK_CRATE_THRESHOLD: usize = 3;

/// `medium` 判定となる影響クレート数の下限（この件数**以上**で `medium`）。
#[allow(dead_code)] // 上記 CLIENT_BOUNDARY_CRATES と同じ理由（#134 未接続）。
const MEDIUM_RISK_CRATE_THRESHOLD: usize = 1;

/// 影響を受けた 1 ファイルと、そのファイル内でシンボルが出現した行番号一覧。
///
/// 行番号は 1 始まり。#134 が走査結果として構築し、#136 が JSON の
/// `affected_files[].lines` として出力する契約。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AffectedFile {
    /// ワークスペースルート相対パス（絶対パス・環境情報を含めない、
    /// `docs/impact-analysis-design.md` §6 A09 対策）。
    pub file: String,
    /// シンボルが出現した行番号（1 始まり、昇順）。
    pub lines: Vec<usize>,
}

/// `fw impact <symbol>` の解析結果全体。
///
/// フィールド構成は PoC-7 の `cmd_impact` 戻り値（JSON キー）と互換を保ち、
/// 製品固有の判断（多重定義の扱い）を `ambiguous` として追加する
/// （`docs/impact-analysis-design.md` §3.3・§6.4 の対応表を参照）。
/// JSON へのシリアライズは #136 が `json_out.rs` の方針
/// （`escape_str` を通す・値をそのまま埋め込まない）に従って実装する。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImpactReport {
    /// 解析対象シンボル名（Rust 識別子。[`validate_symbol`] 済みの値のみを許容）。
    pub symbol: String,
    /// 定義元クレート名の一覧。通常は 1 件。0 件はエラー（#135 が
    /// [`ImpactError::SymbolNotFound`] として拒否し、本構造体自体を作らない）。
    /// 2 件以上は [`ImpactReport::ambiguous`] を `true` にして構築する契約。
    pub defined_in_crates: Vec<String>,
    /// 定義元ファイル（ワークスペースルート相対）の一覧。
    /// `defined_in_crates` と対応するインデックスを持つ。
    pub defined_in_files: Vec<String>,
    /// 定義元が複数見つかった場合に `true`。多重定義は解析の前提
    /// （「シンボル 1 つに定義は 1 つ」）が崩れている状態であり、
    /// 自動判定を信頼せず人間承認へ倒す（[`requires_human_approval`] 参照）。
    pub ambiguous: bool,
    pub affected_files: Vec<AffectedFile>,
    /// 影響クレート名（重複なし・ソート済み、#134 が構築）。
    pub affected_crates: Vec<String>,
    /// 影響を受けるルート定義（`routes::extract_routes` の結果のうち、
    /// `defined_in` が `affected_files` に含まれるもの、#134 が構築）。
    pub affected_routes: Vec<String>,
    pub breaking_risk: BreakingRisk,
    pub requires_human_approval: bool,
}

/// `fw impact` 走査・検証段階の失敗。
///
/// `fw structure` / `fw gate` と同じ「黙示的成功を返さない」方針
/// （`docs/structure-manifest.md` §4/§5、security.md A05）を踏襲し、
/// 定義元が見つからない場合も曖昧に成功させずエラーとする。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImpactError {
    /// シンボル名が Rust 識別子の形をしていない（[`validate_symbol`] 参照）。
    /// #135 はこれを使用法エラー（終了コード 2）として扱う契約。
    InvalidSymbol,
    /// ワークスペース全体を走査しても定義元が見つからなかった。
    /// #135 はこれを検証違反（終了コード 1）として扱う契約
    /// （`defined_in: null` で黙って成功させない）。
    ///
    /// `analyze` の fail-closed スタブは常に `Scan` を返すため、#134 が実際の
    /// 走査を実装してこの variant を構築するまで未使用（撤去予定）。
    #[allow(dead_code)]
    SymbolNotFound,
    /// 走査中の I/O・パストラバーサル境界違反（`routes::ExtractError` 相当を
    /// #134 が包む想定のバリアント）。
    Scan(String),
}

impl fmt::Display for ImpactError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ImpactError::InvalidSymbol => {
                write!(f, "symbol must match ^[A-Za-z_][A-Za-z0-9_]*$")
            }
            ImpactError::SymbolNotFound => write!(f, "no definition found for symbol"),
            ImpactError::Scan(detail) => write!(f, "scan failed: {detail}"),
        }
    }
}

impl std::error::Error for ImpactError {}

/// シンボル入力を Rust 識別子（`^[A-Za-z_][A-Za-z0-9_]*$`）に検証する。
///
/// 正規表現クレートを使わず手書きの文字種判定のみで行う（`cli` 外部依存
/// ゼロ方針・`coding-rust.md`）。ここを通らない入力（空文字・記号・数字始まり等）
/// は #135 が使用法エラー（終了コード 2）として拒否する契約であり、
/// 検証を経ないシンボル文字列がシェル・ファイル走査へ渡ることはない
/// （`docs/impact-analysis-design.md` §6 A03 対策）。
///
/// # Errors
///
/// 識別子の形をしていない場合に [`ImpactError::InvalidSymbol`] を返す。
pub fn validate_symbol(symbol: &str) -> Result<(), ImpactError> {
    let mut chars = symbol.chars();
    let starts_ok = matches!(chars.next(), Some(c) if c.is_ascii_alphabetic() || c == '_');
    let rest_ok = chars.all(|c| c.is_ascii_alphanumeric() || c == '_');
    if starts_ok && rest_ok {
        Ok(())
    } else {
        Err(ImpactError::InvalidSymbol)
    }
}

/// `haystack` 中で `symbol` が識別子境界（前後が `[A-Za-z0-9_]` でない位置）に
/// 一致する箇所が 1 つ以上あるかを判定する。
///
/// 正規表現クレート（`\bsymbol\b` 相当）を使わず手書きで行う
/// （`cli` 外部依存ゼロ方針）。#134 の使用箇所列挙（行単位の出現判定）が
/// この関数を呼ぶ想定の共有ロジック。
///
/// コメント・文字列リテラル内の一致も除外しない（PoC-7 と同じ「過検知容認」
/// 方針、`docs/impact-analysis-design.md` §3.2）。
///
/// #134 の使用箇所走査が本関数を呼ぶまで（本 PR の `analyze` スタブからは
/// 呼ばれない）未使用（撤去予定）。
#[allow(dead_code)]
pub fn contains_symbol_at_boundary(haystack: &str, symbol: &str) -> bool {
    if symbol.is_empty() {
        return false;
    }
    let bytes = haystack.as_bytes();
    let sym_bytes = symbol.as_bytes();
    let is_ident_byte = |b: u8| b.is_ascii_alphanumeric() || b == b'_';

    let mut start = 0usize;
    while let Some(rel_pos) = haystack[start..].find(symbol) {
        let pos = start + rel_pos;
        let end = pos + sym_bytes.len();
        let before_ok = pos == 0 || !is_ident_byte(bytes[pos - 1]);
        let after_ok = end >= bytes.len() || !is_ident_byte(bytes[end]);
        if before_ok && after_ok {
            return true;
        }
        // 境界不一致でも次の出現を探す（例: "abcfoo" 内の "foo" は境界不一致だが
        // 同一行に独立した "foo" が別途あるかもしれない）。
        start = pos + 1;
        if start >= bytes.len() {
            break;
        }
    }
    false
}

/// 影響クレート数・クライアント境界クレートへの波及から `breaking_risk` を判定する。
///
/// PoC-7 の `cmd_impact` の判定式をそのまま踏襲する（`docs/impact-analysis-design.md`
/// §3.4）:
/// - `affected_crates` が [`HIGH_RISK_CRATE_THRESHOLD`] 件以上、または
///   [`CLIENT_BOUNDARY_CRATES`] のいずれかを含む → `High`
/// - 上記に該当せず [`MEDIUM_RISK_CRATE_THRESHOLD`] 件以上 → `Medium`
/// - 影響クレートなし → `Low`
///
/// #134 が走査結果（`affected_crates`）からこの関数を呼んで
/// [`ImpactReport::breaking_risk`] を確定する契約。
///
/// `analyze` の fail-closed スタブ（本 PR 時点）からは呼ばれないため未使用
/// （撤去予定）。
#[allow(dead_code)]
pub fn judge_breaking_risk(affected_crates: &[String]) -> BreakingRisk {
    let touches_client_boundary = affected_crates
        .iter()
        .any(|c| CLIENT_BOUNDARY_CRATES.contains(&c.as_str()));
    if affected_crates.len() >= HIGH_RISK_CRATE_THRESHOLD || touches_client_boundary {
        BreakingRisk::High
    } else if affected_crates.len() >= MEDIUM_RISK_CRATE_THRESHOLD {
        BreakingRisk::Medium
    } else {
        BreakingRisk::Low
    }
}

/// `breaking_risk` / 影響ルート有無 / 多重定義から人間承認要否を判定する。
///
/// `breaking_risk` が `High` / `Medium`、影響ルートが 1 件以上、または
/// 定義元が曖昧（`ambiguous`）のいずれかで `true`（`docs/impact-analysis-design.md`
/// §3.4）。`ambiguous` は PoC-7 にない製品固有の追加条件: 定義元特定の前提
/// （シンボル 1 つに定義は 1 つ）が崩れている場合、他の判定材料
/// （`affected_crates` 等）自体の信頼性が下がるため、常に人間承認へ倒す
/// （安全側 / fail-closed、security.md A05）。
///
/// `analyze` の fail-closed スタブ（本 PR 時点）からは呼ばれないため未使用
/// （撤去予定）。
#[allow(dead_code)]
pub fn requires_human_approval(
    breaking_risk: BreakingRisk,
    affected_routes_is_empty: bool,
    ambiguous: bool,
) -> bool {
    ambiguous
        || matches!(breaking_risk, BreakingRisk::High | BreakingRisk::Medium)
        || !affected_routes_is_empty
}

/// 変更影響範囲解析のエントリポイント（TASK-13.2b, #134 が実装する走査 API）。
///
/// `main.rs` の `impact` サブコマンド（TASK-13.2c, #135）がここを呼ぶ。呼び出し前に
/// `symbol` は [`validate_symbol`] を通過済みであることを CLI 層の契約とする
/// （本関数は再検証しない）。`project_dir` の境界（ワークスペースルート外へ
/// 脱出しない）は `routes::resolve_within_root` / `routes::scan_root` 相当の
/// 仕組みを #134 が再利用する契約であり、本関数はそれを迂回しない
/// （`docs/impact-analysis-design.md` §3.3・§6 A01 対策）。
///
/// # 現状（#134 未マージの間の暫定実装）
///
/// 本 PR（#135）の時点で #134（依存グラフ構築・使用箇所走査）は未マージのため、
/// 本関数は常に `Err(ImpactError::Scan(_))` を返す fail-closed スタブである。
/// 走査せずに黙って空レポートの成功（exit 0）へ倒すと、CLI 呼び出し元
/// （CI・AI 自己保守フック）が「解析未実装」を「影響なし」と誤認しうるため、
/// 決してそちらには倒さない（security.md A05）。#134 マージ後、この関数の中身は
/// 実際の依存グラフ構築・使用箇所走査による [`ImpactReport`] 構築へ置き換わる
/// 契約とする（シグネチャは変更しない想定）。
///
/// # Errors
///
/// 現状は常に `Err(ImpactError::Scan(_))`（#134 未接続の暫定実装のため）。
/// #134 実装後は [`ImpactError::SymbolNotFound`]（定義元が見つからない）も返しうる。
pub fn analyze(_project_dir: &Path, symbol: &str) -> Result<ImpactReport, ImpactError> {
    // `symbol` は #134 実装後に走査の起点として使われる想定。現状は未使用だが
    // シグネチャを確定させるため引数として保持する（余剰引数警告の抑止に
    // `let _ =` は使わず、契約上の意味をコメントで示すのみに留める）。
    let _ = symbol;
    Err(ImpactError::Scan(
        "impact analysis (TASK-13.2b / #134) not yet integrated".to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- validate_symbol ---

    #[test]
    fn validate_symbol_accepts_plain_identifier() {
        assert!(validate_symbol("render").is_ok());
        assert!(validate_symbol("_private").is_ok());
        assert!(validate_symbol("Render123").is_ok());
    }

    #[test]
    fn validate_symbol_rejects_empty() {
        assert_eq!(validate_symbol(""), Err(ImpactError::InvalidSymbol));
    }

    #[test]
    fn validate_symbol_rejects_leading_digit() {
        assert_eq!(validate_symbol("1render"), Err(ImpactError::InvalidSymbol));
    }

    #[test]
    fn validate_symbol_rejects_non_identifier_characters() {
        assert_eq!(validate_symbol("render()"), Err(ImpactError::InvalidSymbol));
        assert_eq!(validate_symbol("a-b"), Err(ImpactError::InvalidSymbol));
        assert_eq!(validate_symbol("a b"), Err(ImpactError::InvalidSymbol));
        assert_eq!(
            validate_symbol("std::render"),
            Err(ImpactError::InvalidSymbol)
        );
    }

    // --- contains_symbol_at_boundary ---

    #[test]
    fn boundary_matches_standalone_symbol() {
        assert!(contains_symbol_at_boundary("pub fn render() {}", "render"));
        assert!(contains_symbol_at_boundary("let x = render(y);", "render"));
    }

    #[test]
    fn boundary_matches_symbol_at_line_start_or_end() {
        assert!(contains_symbol_at_boundary("render", "render"));
        assert!(contains_symbol_at_boundary("(render)", "render"));
    }

    #[test]
    fn boundary_rejects_substring_of_longer_identifier() {
        assert!(!contains_symbol_at_boundary("render_all()", "render"));
        assert!(!contains_symbol_at_boundary("prerender()", "render"));
        assert!(!contains_symbol_at_boundary("rendering", "render"));
    }

    #[test]
    fn boundary_finds_match_after_non_boundary_occurrence() {
        // "prerender" は境界不一致だが、同じ行に独立した "render" もあるケース。
        assert!(contains_symbol_at_boundary("prerender render()", "render"));
    }

    #[test]
    fn boundary_empty_symbol_never_matches() {
        assert!(!contains_symbol_at_boundary("anything", ""));
    }

    // --- judge_breaking_risk ---

    #[test]
    fn judge_breaking_risk_zero_crates_is_low() {
        assert_eq!(judge_breaking_risk(&[]), BreakingRisk::Low);
    }

    #[test]
    fn judge_breaking_risk_one_crate_is_medium() {
        let crates = vec!["rws-core".to_string()];
        assert_eq!(judge_breaking_risk(&crates), BreakingRisk::Medium);
    }

    #[test]
    fn judge_breaking_risk_two_crates_is_medium() {
        let crates = vec!["rws-core".to_string(), "rws-app".to_string()];
        assert_eq!(judge_breaking_risk(&crates), BreakingRisk::Medium);
    }

    #[test]
    fn judge_breaking_risk_three_crates_is_high() {
        let crates = vec![
            "rws-core".to_string(),
            "rws-app".to_string(),
            "rws-server".to_string(),
        ];
        assert_eq!(judge_breaking_risk(&crates), BreakingRisk::High);
    }

    #[test]
    fn judge_breaking_risk_single_wasm_client_crate_is_high() {
        let crates = vec!["rws-wasm-client".to_string()];
        assert_eq!(judge_breaking_risk(&crates), BreakingRisk::High);
    }

    #[test]
    fn judge_breaking_risk_single_wasm_full_crate_is_high() {
        let crates = vec!["rws-wasm-full".to_string()];
        assert_eq!(judge_breaking_risk(&crates), BreakingRisk::High);
    }

    #[test]
    fn judge_breaking_risk_single_wasm_thin_crate_is_high() {
        let crates = vec!["rws-wasm-thin".to_string()];
        assert_eq!(judge_breaking_risk(&crates), BreakingRisk::High);
    }

    // --- requires_human_approval ---

    #[test]
    fn approval_not_required_for_low_risk_no_routes_unambiguous() {
        assert!(!requires_human_approval(BreakingRisk::Low, true, false));
    }

    #[test]
    fn approval_required_for_medium_risk() {
        assert!(requires_human_approval(BreakingRisk::Medium, true, false));
    }

    #[test]
    fn approval_required_for_high_risk() {
        assert!(requires_human_approval(BreakingRisk::High, true, false));
    }

    #[test]
    fn approval_required_when_routes_affected_even_if_low_risk() {
        assert!(requires_human_approval(BreakingRisk::Low, false, false));
    }

    #[test]
    fn approval_required_when_ambiguous_even_if_low_risk_no_routes() {
        assert!(requires_human_approval(BreakingRisk::Low, true, true));
    }

    // --- BreakingRisk::as_str / Display ---

    #[test]
    fn breaking_risk_as_str_matches_poc7_json_values() {
        assert_eq!(BreakingRisk::High.as_str(), "high");
        assert_eq!(BreakingRisk::Medium.as_str(), "medium");
        assert_eq!(BreakingRisk::Low.as_str(), "low");
    }

    #[test]
    fn breaking_risk_display_matches_as_str() {
        assert_eq!(BreakingRisk::High.to_string(), "high");
    }

    // --- analyze（#134 未接続の間の fail-closed スタブ） ---

    /// #134 がマージされるまで `analyze` は常に `Scan` エラーを返す契約
    /// （黙示的成功に倒さない、security.md A05）。有効なシンボル名を渡しても
    /// 成功しないことをここで固定し、将来 #134 接続時に本テストが
    /// 意図的に落ちる（=置き換えを検知できる）ようにする。
    #[test]
    fn analyze_stub_never_succeeds_even_for_valid_symbol() {
        let result = analyze(Path::new("."), "render");
        assert!(matches!(result, Err(ImpactError::Scan(_))));
    }
}
