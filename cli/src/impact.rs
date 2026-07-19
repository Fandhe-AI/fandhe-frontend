//! 変更影響範囲解析（TASK-13.2a, #133、親 TASK-13.2 #132、REQ-13）の型定義と
//! 判定ロジック。`main.rs` の `impact` サブコマンド（TASK-13.2c, #135）から
//! [`analyze`] を呼び出して使われる。
//!
//! 本ファイルは TASK-13.2 の 5 分割サブタスク（#133 設計（本ファイル）/
//! #134 依存グラフ構築 / #135 コマンド実装 / #136 出力フォーマット（本ファイル） /
//! #137 テスト整備）が依拠する**単一の情報源**として、`fw impact <symbol>`
//! （`main.rs` の `impact` サブコマンド、TASK-13.2c, #135 が接続する）が返す
//! JSON の形（[`ImpactReport`]）と、`breaking_risk` / `requires_human_approval`
//! の判定を副作用のない純粋関数（[`judge_breaking_risk`] / [`requires_human_approval`]）
//! として切り出す。走査（定義元特定・使用箇所列挙・ルート突き合わせ）は本ファイルが
//! 実装する（TASK-13.2b, #134）。JSON シリアライズ（[`render_report`]、`verdict`
//! 文字列生成含む）も本ファイルが実装する（TASK-13.2d, #136、
//! `docs/design/impact-analysis-design.md` §3.5 の JSON スキーマに準拠、
//! `json_out::escape_str` を唯一の文字列エスケープ経路として使う）。CLI 接続
//! （`main.rs` へのディスパッチ・終了コード処理・[`render_report`] の呼び出し）は
//! #135 の担当で、[`analyze`] は `cargo metadata` の実行（`metadata::fetch`）を
//! 呼び出し元（#135 の CLI 層）に委ねる契約とする
//! （詳細は `docs/design/impact-analysis-design.md` §8 を参照）。
//!
//! アルゴリズムは PoC-7（`docs/spec/03-poc/ai-self-maintenance/tools/poc7_tool.py`
//! `cmd_impact`）を踏襲した「ファイル単位の粗粒度ヒューリスティック」であり、
//! AST 解析ベースの精密化は将来スコープとして明示的に見送る
//! （`docs/spec/05-tasks.md` TASK-13.2、`docs/design/impact-analysis-design.md` §7）。

use crate::component_boundary;
use crate::json_out;
use crate::metadata::MemberPackage;
use crate::routes;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::path::Path;

/// 破壊的変更リスクの粗粒度分類。
///
/// 判定は [`judge_breaking_risk`] が一元的に行う。PoC-7 と同じ 3 段階
/// （`high` / `medium` / `low`）を製品仕様として踏襲する
/// （`docs/design/impact-analysis-design.md` §3.4）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
/// 倒す（安全側・過検知容認、`docs/design/impact-analysis-design.md` §3.2）。
///
/// #134（依存グラフ構築）・#135（CLI 接続）はここを参照し、実測した
/// `affected_crates` との突き合わせに同じ定数を使う契約とする（二重管理しない）。
///
/// `judge_breaking_risk` からのみ参照される。
pub const CLIENT_BOUNDARY_CRATES: [&str; 3] = ["rws-wasm-client", "rws-wasm-full", "rws-wasm-thin"];

/// `high` 判定となる影響クレート数の下限（この件数**以上**で `high`）。
const HIGH_RISK_CRATE_THRESHOLD: usize = 3;

/// `medium` 判定となる影響クレート数の下限（この件数**以上**で `medium`）。
const MEDIUM_RISK_CRATE_THRESHOLD: usize = 1;

/// 影響を受けた 1 ファイルと、そのファイル内でシンボルが出現した行番号一覧。
///
/// 行番号は 1 始まり。#134 が走査結果として構築し、#136 が JSON の
/// `affected_files[].lines` として出力する契約。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AffectedFile {
    /// ワークスペースルート相対パス（絶対パス・環境情報を含めない、
    /// `docs/design/impact-analysis-design.md` §6 A09 対策）。
    pub file: String,
    /// シンボルが出現した行番号（1 始まり、昇順）。
    pub lines: Vec<usize>,
}

/// `fw impact <symbol>` の解析結果全体。
///
/// フィールド構成は PoC-7 の `cmd_impact` 戻り値（JSON キー）と互換を保ち、
/// 製品固有の判断（多重定義の扱い）を `ambiguous` として追加する
/// （`docs/design/impact-analysis-design.md` §3.3・§6.4 の対応表を参照）。
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
/// （`docs/design/structure-manifest.md` §4/§5、security.md A05）を踏襲し、
/// 定義元が見つからない場合も曖昧に成功させずエラーとする。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImpactError {
    /// シンボル名が Rust 識別子の形をしていない（[`validate_symbol`] 参照）。
    /// #135 はこれを使用法エラー（終了コード 2）として扱う契約。
    InvalidSymbol,
    /// ワークスペース全体を走査しても定義元が見つからなかった。
    /// #135 はこれを検証違反（終了コード 1）として扱う契約
    /// （`defined_in: null` で黙って成功させない）。
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
/// （`docs/design/impact-analysis-design.md` §6 A03 対策）。
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
/// 方針、`docs/design/impact-analysis-design.md` §3.2）。
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
/// PoC-7 の `cmd_impact` の判定式をそのまま踏襲する（`docs/design/impact-analysis-design.md`
/// §3.4）:
/// - `affected_crates` が [`HIGH_RISK_CRATE_THRESHOLD`] 件以上、または
///   [`CLIENT_BOUNDARY_CRATES`] のいずれかを含む → `High`
/// - 上記に該当せず [`MEDIUM_RISK_CRATE_THRESHOLD`] 件以上 → `Medium`
/// - 影響クレートなし → `Low`
///
/// #134 が走査結果（`affected_crates`）からこの関数を呼んで
/// [`ImpactReport::breaking_risk`] を確定する契約。
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
/// 定義元が曖昧（`ambiguous`）のいずれかで `true`（`docs/design/impact-analysis-design.md`
/// §3.4）。`ambiguous` は PoC-7 にない製品固有の追加条件: 定義元特定の前提
/// （シンボル 1 つに定義は 1 つ）が崩れている場合、他の判定材料
/// （`affected_crates` 等）自体の信頼性が下がるため、常に人間承認へ倒す
/// （安全側 / fail-closed、security.md A05）。
pub fn requires_human_approval(
    breaking_risk: BreakingRisk,
    affected_routes_is_empty: bool,
    ambiguous: bool,
) -> bool {
    ambiguous
        || matches!(breaking_risk, BreakingRisk::High | BreakingRisk::Medium)
        || !affected_routes_is_empty
}

/// 定義元候補 1 件（クレート名 + ワークスペースルート相対ファイルパス）。
///
/// [`find_definitions`] の内部戻り値型。[`ImpactReport::defined_in_crates`] /
/// [`ImpactReport::defined_in_files`] は本構造体のリストから対応インデックスを
/// 保って構築される（`analyze` 参照）。
#[derive(Debug, Clone, PartialEq, Eq)]
struct Definition {
    crate_name: String,
    file: String,
}

/// `routes::ExtractError` を [`ImpactError::Scan`] に変換する。
///
/// ファイル列挙・走査は `routes::scan_root` / `routes::list_rs_files`
/// （パストラバーサル対策済み、`docs/design/impact-analysis-design.md` §3.3 必須事項）
/// のみを経由し、本ファイルが独自のパス解決・ファイル列挙を新設しないための
/// 変換窓口。
fn scan_err(e: routes::ExtractError) -> ImpactError {
    ImpactError::Scan(e.to_string())
}

/// workspace member のディレクトリ名（`routes::scan_root` 等に渡す 1 段の
/// ワークスペース相対ディレクトリ名）を `manifest_dir` から導出する。
///
/// `routes::resolve_within_root` は「ワークスペースルート直下 1 段のディレクトリ
/// 名」のみを受け付ける契約であるため、`manifest_dir` がワークスペースルート
/// 直下の単一ディレクトリでない場合（多段パス・ルート外）は fail-closed で
/// [`ImpactError::Scan`] を返す（`docs/design/impact-analysis-design.md` §6 A05 対策）。
fn member_dir_name(workspace_root: &Path, member: &MemberPackage) -> Result<String, ImpactError> {
    let rel = member
        .manifest_dir
        .strip_prefix(workspace_root)
        .map_err(|_| {
            ImpactError::Scan(format!(
                "member `{}` manifest_dir is not under workspace_root",
                member.name
            ))
        })?;
    let mut components = rel.components();
    let first = components.next().ok_or_else(|| {
        ImpactError::Scan(format!(
            "member `{}` manifest_dir equals workspace_root",
            member.name
        ))
    })?;
    if components.next().is_some() {
        return Err(ImpactError::Scan(format!(
            "member `{}` manifest_dir is not a single top-level directory",
            member.name
        )));
    }
    first
        .as_os_str()
        .to_str()
        .map(str::to_string)
        .ok_or_else(|| {
            ImpactError::Scan(format!(
                "member `{}` directory name is not valid UTF-8",
                member.name
            ))
        })
}

/// 走査で得た絶対パスをワークスペースルート相対パス（`/` 区切り）に正規化する。
///
/// 出力（[`AffectedFile::file`] 等）に絶対パス・環境情報を残さないための変換
/// （`docs/design/impact-analysis-design.md` §6 A09 対策）。`routes::resolve_within_root`
/// が返す走査起点は `canonicalize` 済みのため、比較対象の `workspace_root` 側も
/// 同じく `canonicalize` してから `strip_prefix` する。
fn to_workspace_relative(workspace_root: &Path, file: &Path) -> Result<String, ImpactError> {
    let canonical_root = std::fs::canonicalize(workspace_root)
        .map_err(|e| ImpactError::Scan(format!("{:?}", e.kind())))?;
    let rel = file
        .strip_prefix(&canonical_root)
        .map_err(|_| ImpactError::Scan("scanned file escaped workspace root".to_string()))?;
    rel.to_str()
        .map(|s| s.replace('\\', "/"))
        .ok_or_else(|| ImpactError::Scan("scanned file path is not valid UTF-8".to_string()))
}

/// ワークスペース全 member を走査し、トップレベル公開宣言としてシンボルを
/// 定義しているファイルを列挙する（定義元特定、`docs/design/impact-analysis-design.md`
/// §3.1）。
///
/// 列挙は `routes::scan_root` / `routes::list_rs_files`（シンボリックリンク
/// 非追従・ワークスペースルート限定）のみを使い、各ファイル内容を
/// `component_boundary::extract_from_source` に通してトップレベル `pub`
/// 宣言と突き合わせる。0 件は [`ImpactError::SymbolNotFound`]（黙示的成功を
/// 返さない、security.md A05）。2 件以上は全候補をそのまま返し、呼び出し元
/// （[`analyze`]）が `ambiguous = true` として扱う。
fn find_definitions(
    workspace_root: &Path,
    members: &[MemberPackage],
    symbol: &str,
) -> Result<Vec<Definition>, ImpactError> {
    let mut definitions = Vec::new();
    for member in members {
        let dir_name = member_dir_name(workspace_root, member)?;
        let target = routes::scan_root(workspace_root, &dir_name).map_err(scan_err)?;
        for file in routes::list_rs_files(&target).map_err(scan_err)? {
            let content = std::fs::read_to_string(&file)
                .map_err(|e| ImpactError::Scan(format!("{:?}", e.kind())))?;
            let found = component_boundary::extract_from_source(&content)
                .into_iter()
                .any(|s| s.name == symbol);
            if found {
                let rel = to_workspace_relative(workspace_root, &file)?;
                definitions.push(Definition {
                    crate_name: member.name.clone(),
                    file: rel,
                });
            }
        }
    }
    if definitions.is_empty() {
        return Err(ImpactError::SymbolNotFound);
    }
    Ok(definitions)
}

/// ワークスペース全 member を走査し、シンボルの使用箇所（定義元ファイル自身を
/// 除く）を行番号付きで列挙する（使用箇所走査、`docs/design/impact-analysis-design.md`
/// §3.2）。
///
/// 行単位に [`contains_symbol_at_boundary`] を適用する。コメント・文字列リテラル
/// 内のヒットも除外しない（PoC-7 と同じ「過検知容認」方針）。ヒットのある
/// ファイルのみを返し、ファイルパス昇順にソートする。
fn scan_usages(
    workspace_root: &Path,
    members: &[MemberPackage],
    symbol: &str,
    definition_files: &[String],
) -> Result<Vec<AffectedFile>, ImpactError> {
    let mut affected: Vec<AffectedFile> = Vec::new();
    for member in members {
        let dir_name = member_dir_name(workspace_root, member)?;
        let target = routes::scan_root(workspace_root, &dir_name).map_err(scan_err)?;
        for file in routes::list_rs_files(&target).map_err(scan_err)? {
            let rel = to_workspace_relative(workspace_root, &file)?;
            if definition_files.iter().any(|d| d == &rel) {
                continue;
            }
            let content = std::fs::read_to_string(&file)
                .map_err(|e| ImpactError::Scan(format!("{:?}", e.kind())))?;
            let lines: Vec<usize> = content
                .lines()
                .enumerate()
                .filter(|(_, line)| contains_symbol_at_boundary(line, symbol))
                .map(|(idx, _)| idx + 1)
                .collect();
            if !lines.is_empty() {
                affected.push(AffectedFile { file: rel, lines });
            }
        }
    }
    affected.sort_by(|a, b| a.file.cmp(&b.file));
    Ok(affected)
}

/// `affected_files` を含む workspace member（`seeds`）から、逆依存（そのクレートに
/// 依存している他クレート）を BFS で辿った推移閉包を返す（`seeds` 自身を含む）。
///
/// 「A に依存する B が影響を受けるなら、B にさらに依存する C も波及候補として
/// 保守的に含める」（`docs/design/impact-analysis-design.md` §5）を実装する純粋関数。
/// I/O を行わないため単体テストで `MemberPackage` を直接構築して検証できる。
fn reverse_dependency_closure(
    members: &[MemberPackage],
    seeds: &BTreeSet<String>,
) -> BTreeSet<String> {
    // dep 名 -> それに依存している member 名一覧（逆隣接マップ）。
    let mut reverse: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for member in members {
        for dep in &member.normal_workspace_deps {
            reverse
                .entry(dep.as_str())
                .or_default()
                .push(member.name.as_str());
        }
    }

    let mut closure: BTreeSet<String> = seeds.clone();
    let mut queue: Vec<String> = seeds.iter().cloned().collect();
    while let Some(current) = queue.pop() {
        if let Some(dependents) = reverse.get(current.as_str()) {
            for dependent in dependents {
                if closure.insert((*dependent).to_string()) {
                    queue.push((*dependent).to_string());
                }
            }
        }
    }
    closure
}

/// `rel_file`（ワークスペースルート相対パス）が属する workspace member 名を返す。
///
/// パスの先頭 1 段（ディレクトリ名）を [`member_dir_name`] の結果と突き合わせる。
/// 一致する member がない場合（走査対象外のファイル等）は `None`。
fn crate_name_for_file(
    workspace_root: &Path,
    members: &[MemberPackage],
    rel_file: &str,
) -> Result<Option<String>, ImpactError> {
    let first_component = rel_file.split('/').next().unwrap_or("");
    for member in members {
        let dir_name = member_dir_name(workspace_root, member)?;
        if dir_name == first_component {
            return Ok(Some(member.name.clone()));
        }
    }
    Ok(None)
}

/// `affected_files` の内容を `routes::extract_routes_from_source` に通し、
/// 抽出されたルートの `path` を重複除去・昇順ソートして返す
/// （影響ルート突き合わせ、`docs/design/impact-analysis-design.md` §3.2）。
///
/// ファイル単位の粗粒度突き合わせであり、`structure.toml` の
/// `[routing].definition_dir` 宣言には依存しない（過検知容認）。
fn affected_route_paths(
    workspace_root: &Path,
    affected_files: &[AffectedFile],
) -> Result<Vec<String>, ImpactError> {
    let canonical_root = std::fs::canonicalize(workspace_root)
        .map_err(|e| ImpactError::Scan(format!("{:?}", e.kind())))?;
    let mut routes_found: BTreeSet<String> = BTreeSet::new();
    for affected in affected_files {
        let abs = canonical_root.join(&affected.file);
        let content = std::fs::read_to_string(&abs)
            .map_err(|e| ImpactError::Scan(format!("{:?}", e.kind())))?;
        for route in routes::extract_routes_from_source(&content) {
            routes_found.insert(route.path);
        }
    }
    Ok(routes_found.into_iter().collect())
}

/// `fw impact <symbol>` の解析エンジン本体（TASK-13.2b, #134）。
///
/// [`validate_symbol`] → [`find_definitions`] → [`scan_usages`] → seeds 構築 →
/// [`reverse_dependency_closure`] → [`affected_route_paths`] →
/// [`judge_breaking_risk`] / [`requires_human_approval`] の順で [`ImpactReport`]
/// を構築する（`docs/design/impact-analysis-design.md` §3〜§5）。
///
/// `cargo metadata` の実行（`metadata::fetch`）は呼び出し元の CLI 層（#135）が
/// 行う契約とし、本関数は `&[MemberPackage]` を受け取る。これにより単体テストは
/// `cargo` プロセスを起動せず `MemberPackage` を直接構築して検証できる。
///
/// # Errors
///
/// `symbol` が Rust 識別子の形をしていない場合に [`ImpactError::InvalidSymbol`]、
/// 定義元が 1 件も見つからない場合に [`ImpactError::SymbolNotFound`]、
/// 走査中の I/O・パストラバーサル境界違反時に [`ImpactError::Scan`] を返す。
pub(crate) fn analyze(
    workspace_root: &Path,
    members: &[MemberPackage],
    symbol: &str,
) -> Result<ImpactReport, ImpactError> {
    validate_symbol(symbol)?;

    let definitions = find_definitions(workspace_root, members, symbol)?;
    let ambiguous = definitions.len() > 1;
    let defined_in_crates: Vec<String> = definitions.iter().map(|d| d.crate_name.clone()).collect();
    let defined_in_files: Vec<String> = definitions.iter().map(|d| d.file.clone()).collect();

    let affected_files = scan_usages(workspace_root, members, symbol, &defined_in_files)?;

    let mut seeds: BTreeSet<String> = BTreeSet::new();
    for affected in &affected_files {
        if let Some(crate_name) = crate_name_for_file(workspace_root, members, &affected.file)? {
            seeds.insert(crate_name);
        }
    }
    let affected_crates: Vec<String> = reverse_dependency_closure(members, &seeds)
        .into_iter()
        .collect();

    let affected_routes = affected_route_paths(workspace_root, &affected_files)?;

    let breaking_risk = judge_breaking_risk(&affected_crates);
    let approval_required =
        requires_human_approval(breaking_risk, affected_routes.is_empty(), ambiguous);

    Ok(ImpactReport {
        symbol: symbol.to_string(),
        defined_in_crates,
        defined_in_files,
        ambiguous,
        affected_files,
        affected_crates,
        affected_routes,
        breaking_risk,
        requires_human_approval: approval_required,
    })
}

/// 人間可読な判定要約（`docs/design/impact-analysis-design.md` §3.5 の `verdict`
/// フィールド）を固定 2 値で生成する。
///
/// 判定材料は `ImpactReport::requires_human_approval` のみを使う
/// （`judge_breaking_risk` / `requires_human_approval` の判定ロジックを
/// ここで二重実装しない）。文言は英語で確定する: PoC-7 は日本語 2 値
/// （「要人間承認」/「自動適用可」）だが、`japanese-style.md` は
/// 「ユーザー向け文字列は仕様で指定がない限り英語」と規定し、
/// `docs/design/impact-analysis-design.md` §3.5 の「PoC-7 互換」はフィールドの
/// 存在・意味・2 値構造の互換と解釈する（判断 D1、同 §3.5 に追記）。
fn verdict_text(requires_human_approval: bool) -> &'static str {
    if requires_human_approval {
        "requires human approval (impact spans multiple crates or public routes)"
    } else {
        "auto-applicable (impact is limited; automatic application allowed subject to gate pass)"
    }
}

/// `Vec<String>`（`defined_in_crates` / `defined_in_files`）を JSON スカラーへ
/// 単数化する（`docs/design/impact-analysis-design.md` §3.5: 「複数なら先頭要素、
/// `ambiguous` 参照」）。多重定義時も先頭要素をそのまま出力し、多重定義の
/// 事実自体は `ambiguous` フィールドが伝える契約とするため、`ambiguous` の値
/// では分岐しない。`analyze` は 0 件を `SymbolNotFound` として拒否するため
/// 実運用で空になることはないが、防御的に空の場合は `null` を返し
/// panic させない（PoC-7 スキーマの `string | null` 互換）。
fn scalar_or_null(values: &[String]) -> String {
    match values.first() {
        Some(v) => json_out::quoted(v),
        None => "null".to_string(),
    }
}

/// `ImpactReport` を `docs/design/impact-analysis-design.md` §3.5 の JSON スキーマに
/// 従い 1 行の JSON へシリアライズする（`fw structure` の
/// `json_out::render` と同じくパイプ処理・他ツール読み込み前提で
/// pretty-print はしない）。
///
/// #135（CLI 接続）が `analyze()` 成功時に stdout へ出力する契約。
/// マニフェスト由来ではなくワークスペース走査由来の文字列
/// （`symbol` / ファイルパス / クレート名・ルートパス）も、利用者の
/// ソースコード内容に起因し得る任意文字列であることに変わりはないため、
/// 全ての文字列値は `json_out::quoted`（内部で `escape_str` を通す）
/// 経由でのみ埋め込む（security.md A08、JSON インジェクション対策）。
pub(crate) fn render_report(report: &ImpactReport) -> String {
    let mut buf = String::new();
    buf.push('{');

    buf.push_str("\"symbol\":");
    buf.push_str(&json_out::quoted(&report.symbol));

    buf.push_str(",\"defined_in_crate\":");
    buf.push_str(&scalar_or_null(&report.defined_in_crates));

    buf.push_str(",\"defined_in_file\":");
    buf.push_str(&scalar_or_null(&report.defined_in_files));

    buf.push_str(",\"ambiguous\":");
    buf.push_str(if report.ambiguous { "true" } else { "false" });

    buf.push_str(",\"affected_files\":[");
    for (i, affected) in report.affected_files.iter().enumerate() {
        if i > 0 {
            buf.push(',');
        }
        buf.push('{');
        buf.push_str("\"file\":");
        buf.push_str(&json_out::quoted(&affected.file));
        buf.push_str(",\"lines\":");
        buf.push_str(&json_out::usize_array(&affected.lines));
        buf.push('}');
    }
    buf.push(']');

    buf.push_str(",\"affected_crates\":");
    buf.push_str(&json_out::string_array(&report.affected_crates));

    buf.push_str(",\"affected_routes\":");
    buf.push_str(&json_out::string_array(&report.affected_routes));

    buf.push_str(",\"breaking_risk\":");
    buf.push_str(&json_out::quoted(report.breaking_risk.as_str()));

    buf.push_str(",\"requires_human_approval\":");
    buf.push_str(if report.requires_human_approval {
        "true"
    } else {
        "false"
    });

    buf.push_str(",\"verdict\":");
    buf.push_str(&json_out::quoted(verdict_text(
        report.requires_human_approval,
    )));

    buf.push('}');
    buf
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

    // --- #134 解析エンジン本体（find_definitions / scan_usages /
    // reverse_dependency_closure / analyze）のテスト用擬似ワークスペース ---
    //
    // `cargo metadata` を起動せず `MemberPackage` を直接構築できることを
    // `analyze` の契約として確認するため、実ファイルシステム上に最小限の
    // workspace member ディレクトリ（`<root>/<name>/src/*.rs`）を作る。

    /// 各テストが専有する一時ディレクトリを作り、終了時に削除するガード。
    struct TempWorkspace {
        root: std::path::PathBuf,
    }

    impl TempWorkspace {
        fn new(label: &str) -> Self {
            let root = std::env::temp_dir().join(format!(
                "fw-impact-test-{label}-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .expect("system clock should be after epoch")
                    .as_nanos()
            ));
            let _ = std::fs::remove_dir_all(&root);
            std::fs::create_dir_all(&root).expect("create temp workspace root");
            TempWorkspace { root }
        }

        /// `<root>/<crate_name>/src/<file_name>` にソースを書き込み、対応する
        /// [`MemberPackage`] を返す。
        fn write_member(
            &self,
            crate_name: &str,
            file_name: &str,
            content: &str,
            deps: &[&str],
        ) -> MemberPackage {
            let manifest_dir = self.root.join(crate_name);
            let src = manifest_dir.join("src");
            std::fs::create_dir_all(&src).expect("create member src dir");
            std::fs::write(src.join(file_name), content).expect("write member source file");
            MemberPackage {
                name: crate_name.to_string(),
                manifest_dir,
                normal_workspace_deps: deps.iter().map(|d| d.to_string()).collect(),
            }
        }
    }

    impl Drop for TempWorkspace {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.root);
        }
    }

    // --- find_definitions ---

    #[test]
    fn find_definitions_detects_single_definition() {
        let ws = TempWorkspace::new("find-single");
        let member = ws.write_member(
            "crate-a",
            "lib.rs",
            "pub fn render() -> String { String::new() }\n",
            &[],
        );
        let defs = find_definitions(&ws.root, &[member], "render").expect("should find");
        assert_eq!(defs.len(), 1);
        assert_eq!(defs[0].crate_name, "crate-a");
        assert_eq!(defs[0].file, "crate-a/src/lib.rs");
    }

    #[test]
    fn find_definitions_ignores_indented_declarations() {
        let ws = TempWorkspace::new("find-indented");
        let member = ws.write_member(
            "crate-a",
            "lib.rs",
            "mod inner {\n    pub fn render() {}\n}\n",
            &[],
        );
        let err = find_definitions(&ws.root, &[member], "render").unwrap_err();
        assert_eq!(err, ImpactError::SymbolNotFound);
    }

    #[test]
    fn find_definitions_returns_not_found_when_absent() {
        let ws = TempWorkspace::new("find-absent");
        let member = ws.write_member("crate-a", "lib.rs", "pub fn other() {}\n", &[]);
        let err = find_definitions(&ws.root, &[member], "render").unwrap_err();
        assert_eq!(err, ImpactError::SymbolNotFound);
    }

    #[test]
    fn find_definitions_returns_all_candidates_when_ambiguous() {
        let ws = TempWorkspace::new("find-ambiguous");
        let a = ws.write_member("crate-a", "lib.rs", "pub fn render() {}\n", &[]);
        let b = ws.write_member("crate-b", "lib.rs", "pub fn render() {}\n", &[]);
        let defs = find_definitions(&ws.root, &[a, b], "render").expect("should find both");
        assert_eq!(defs.len(), 2);
    }

    // --- scan_usages ---

    #[test]
    fn scan_usages_collects_sorted_one_indexed_line_numbers() {
        let ws = TempWorkspace::new("usages-lines");
        let member = ws.write_member(
            "crate-a",
            "caller.rs",
            "fn a() {\n    render();\n}\nfn b() {\n    render();\n}\n",
            &[],
        );
        let affected = scan_usages(&ws.root, &[member], "render", &[]).expect("should scan");
        assert_eq!(affected.len(), 1);
        assert_eq!(affected[0].file, "crate-a/src/caller.rs");
        assert_eq!(affected[0].lines, vec![2, 5]);
    }

    #[test]
    fn scan_usages_excludes_definition_file_itself() {
        let ws = TempWorkspace::new("usages-exclude-def");
        let member = ws.write_member(
            "crate-a",
            "lib.rs",
            "pub fn render() {}\nfn helper() {\n    render();\n}\n",
            &[],
        );
        let definition_files = vec!["crate-a/src/lib.rs".to_string()];
        let affected =
            scan_usages(&ws.root, &[member], "render", &definition_files).expect("should scan");
        assert!(
            affected.is_empty(),
            "definition file itself must be excluded from usages: {affected:?}"
        );
    }

    // --- reverse_dependency_closure ---

    #[test]
    fn reverse_dependency_closure_includes_direct_dependents() {
        let a = MemberPackage {
            name: "rws-core".to_string(),
            manifest_dir: std::path::PathBuf::from("/ws/core"),
            normal_workspace_deps: vec![],
        };
        let b = MemberPackage {
            name: "rws-app".to_string(),
            manifest_dir: std::path::PathBuf::from("/ws/app"),
            normal_workspace_deps: vec!["rws-core".to_string()],
        };
        let members = [a, b];
        let mut seeds = BTreeSet::new();
        seeds.insert("rws-core".to_string());
        let closure = reverse_dependency_closure(&members, &seeds);
        assert_eq!(
            closure,
            BTreeSet::from(["rws-core".to_string(), "rws-app".to_string()])
        );
    }

    #[test]
    fn reverse_dependency_closure_includes_transitive_dependents() {
        let a = MemberPackage {
            name: "rws-core".to_string(),
            manifest_dir: std::path::PathBuf::from("/ws/core"),
            normal_workspace_deps: vec![],
        };
        let b = MemberPackage {
            name: "rws-app".to_string(),
            manifest_dir: std::path::PathBuf::from("/ws/app"),
            normal_workspace_deps: vec!["rws-core".to_string()],
        };
        let c = MemberPackage {
            name: "rws-server".to_string(),
            manifest_dir: std::path::PathBuf::from("/ws/server"),
            normal_workspace_deps: vec!["rws-app".to_string()],
        };
        let members = [a, b, c];
        let mut seeds = BTreeSet::new();
        seeds.insert("rws-core".to_string());
        let closure = reverse_dependency_closure(&members, &seeds);
        assert_eq!(
            closure,
            BTreeSet::from([
                "rws-core".to_string(),
                "rws-app".to_string(),
                "rws-server".to_string(),
            ])
        );
    }

    #[test]
    fn reverse_dependency_closure_excludes_unrelated_crates() {
        let a = MemberPackage {
            name: "rws-core".to_string(),
            manifest_dir: std::path::PathBuf::from("/ws/core"),
            normal_workspace_deps: vec![],
        };
        let unrelated = MemberPackage {
            name: "rws-wasm-client".to_string(),
            manifest_dir: std::path::PathBuf::from("/ws/wasm-client"),
            normal_workspace_deps: vec![],
        };
        let members = [a, unrelated];
        let mut seeds = BTreeSet::new();
        seeds.insert("rws-core".to_string());
        let closure = reverse_dependency_closure(&members, &seeds);
        assert_eq!(closure, BTreeSet::from(["rws-core".to_string()]));
    }

    #[test]
    fn reverse_dependency_closure_empty_seeds_is_empty() {
        let a = MemberPackage {
            name: "rws-core".to_string(),
            manifest_dir: std::path::PathBuf::from("/ws/core"),
            normal_workspace_deps: vec![],
        };
        let members = [a];
        let closure = reverse_dependency_closure(&members, &BTreeSet::new());
        assert!(closure.is_empty());
    }

    // --- analyze (統括) ---

    #[test]
    fn analyze_rejects_invalid_symbol() {
        let ws = TempWorkspace::new("analyze-invalid-symbol");
        let members: Vec<MemberPackage> = vec![];
        let err = analyze(&ws.root, &members, "not an ident").unwrap_err();
        assert_eq!(err, ImpactError::InvalidSymbol);
    }

    #[test]
    fn analyze_low_risk_when_no_usages() {
        let ws = TempWorkspace::new("analyze-low-risk");
        let member = ws.write_member("crate-a", "lib.rs", "pub fn render() {}\n", &[]);
        let report = analyze(&ws.root, &[member], "render").expect("should analyze");
        assert_eq!(report.defined_in_crates, vec!["crate-a".to_string()]);
        assert!(!report.ambiguous);
        assert!(report.affected_files.is_empty());
        assert_eq!(report.breaking_risk, BreakingRisk::Low);
        assert!(!report.requires_human_approval);
    }

    #[test]
    fn analyze_reflects_affected_crates_and_client_boundary_risk() {
        let ws = TempWorkspace::new("analyze-crates-risk");
        let core = ws.write_member("rws-core", "lib.rs", "pub fn render() {}\n", &[]);
        let wasm_client = ws.write_member(
            "rws-wasm-client",
            "lib.rs",
            "fn use_it() {\n    render();\n}\n",
            &["rws-core"],
        );
        let report = analyze(&ws.root, &[core, wasm_client], "render").expect("should analyze");
        // affected_crates は「使用箇所を含むクレート」を seed とした逆依存閉包
        // （docs/design/impact-analysis-design.md §5）であり、定義元クレート自体は
        // 別途使用されていない限り含まれない。ここでは rws-wasm-client のみが
        // 使用箇所を持ち、それに依存する他クレートは存在しないため単独になる。
        assert_eq!(report.affected_crates, vec!["rws-wasm-client".to_string()]);
        assert_eq!(report.breaking_risk, BreakingRisk::High);
        assert!(report.requires_human_approval);
    }

    #[test]
    fn analyze_marks_affected_routes_and_requires_approval() {
        let ws = TempWorkspace::new("analyze-routes");
        let core = ws.write_member("rws-core", "lib.rs", "pub fn item_detail() {}\n", &[]);
        let server = ws.write_member(
            "rws-server",
            "router.rs",
            "fn build() {\n    Router::new().route(\"/items/:id\", item_detail)?;\n}\n",
            &["rws-core"],
        );
        let report = analyze(&ws.root, &[core, server], "item_detail").expect("should analyze");
        assert_eq!(report.affected_routes, vec!["/items/:id".to_string()]);
        assert!(report.requires_human_approval);
    }

    #[test]
    fn analyze_reports_ambiguous_and_requires_approval_when_multiply_defined() {
        let ws = TempWorkspace::new("analyze-ambiguous");
        let a = ws.write_member("crate-a", "lib.rs", "pub fn render() {}\n", &[]);
        let b = ws.write_member("crate-b", "lib.rs", "pub fn render() {}\n", &[]);
        let report = analyze(&ws.root, &[a, b], "render").expect("should analyze");
        assert!(report.ambiguous);
        assert!(report.requires_human_approval);
        assert_eq!(report.defined_in_crates.len(), 2);
    }

    // --- fail-closed ---

    #[test]
    fn member_dir_name_rejects_nested_manifest_dir() {
        let workspace_root = std::path::Path::new("/ws");
        let member = MemberPackage {
            name: "nested".to_string(),
            manifest_dir: std::path::PathBuf::from("/ws/group/nested"),
            normal_workspace_deps: vec![],
        };
        let err = member_dir_name(workspace_root, &member).unwrap_err();
        assert!(matches!(err, ImpactError::Scan(_)));
    }

    #[test]
    fn member_dir_name_rejects_manifest_dir_outside_workspace_root() {
        let workspace_root = std::path::Path::new("/ws");
        let member = MemberPackage {
            name: "outside".to_string(),
            manifest_dir: std::path::PathBuf::from("/elsewhere/outside"),
            normal_workspace_deps: vec![],
        };
        let err = member_dir_name(workspace_root, &member).unwrap_err();
        assert!(matches!(err, ImpactError::Scan(_)));
    }

    /// レビュー指摘 #127 相当の保証（`routes.rs` の
    /// `list_rs_files_does_not_follow_symlinked_directory` が実体テスト済み）を
    /// `analyze` 経由で確認する: シンボリックリンク越しのワークスペース**外**
    /// ファイルは走査結果（延いては定義元特定）に含まれない。
    ///
    /// 判別可能性を確保するため、リンク先には対象シンボルと**同名**の
    /// `pub fn render()` を置く。もし実装がシンボリックリンクを辿ってしまえば
    /// `defined_in_files` が 2 件（`ambiguous = true`）になり検出できる
    /// （リンク先に別名シンボルを置くと、辿っても辿らなくても定義元が
    /// 1 件のままとなり検出できない旧版のテストの不備を修正）。
    #[cfg(unix)]
    #[test]
    fn analyze_does_not_follow_symlinked_directory_out_of_workspace() {
        let ws = TempWorkspace::new("analyze-symlink");
        let member = ws.write_member("crate-a", "lib.rs", "pub fn render() {}\n", &[]);

        // ワークスペースルート（`ws.root`）とは兄弟関係にある、真にルート外の
        // ディレクトリにリンク先を用意する（`ws.root` 配下だと "workspace 外" の
        // 検証にならないため）。
        let outside = std::env::temp_dir().join(format!(
            "fw-impact-test-analyze-symlink-outside-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after epoch")
                .as_nanos()
        ));
        std::fs::create_dir_all(&outside).expect("create outside dir");
        std::fs::write(outside.join("secret.rs"), "pub fn render() {}\n")
            .expect("write outside file");
        std::os::unix::fs::symlink(&outside, ws.root.join("crate-a").join("src").join("linked"))
            .expect("create symlink");

        let report = analyze(&ws.root, &[member], "render").expect("should analyze");
        assert_eq!(
            report.defined_in_files.len(),
            1,
            "symlinked directory outside the workspace root must not be followed: {:?}",
            report.defined_in_files
        );
        assert!(!report.ambiguous);

        let _ = std::fs::remove_dir_all(&outside);
    }

    // --- 不正シンボル ---

    #[test]
    fn analyze_rejects_symbol_with_invalid_characters() {
        let ws = TempWorkspace::new("analyze-invalid-chars");
        let members: Vec<MemberPackage> = vec![];
        let err = analyze(&ws.root, &members, "render()").unwrap_err();
        assert_eq!(err, ImpactError::InvalidSymbol);
    }

    // --- render_report（TASK-13.2d, #136: JSON 出力フォーマット） ---

    /// テスト用の最小 `ImpactReport` を組み立てるヘルパー。各テストは
    /// 検証したいフィールドのみ引数で上書きする。
    fn sample_report() -> ImpactReport {
        ImpactReport {
            symbol: "render".to_string(),
            defined_in_crates: vec!["rws-core".to_string()],
            defined_in_files: vec!["core/src/lib.rs".to_string()],
            ambiguous: false,
            affected_files: vec![AffectedFile {
                file: "app/src/main.rs".to_string(),
                lines: vec![3, 10],
            }],
            affected_crates: vec!["rws-app".to_string()],
            affected_routes: vec!["/items/:id".to_string()],
            breaking_risk: BreakingRisk::Medium,
            requires_human_approval: true,
        }
    }

    #[test]
    fn render_report_matches_expected_json_key_order_and_shape() {
        let report = sample_report();
        let json = render_report(&report);
        assert_eq!(
            json,
            "{\"symbol\":\"render\",\
             \"defined_in_crate\":\"rws-core\",\
             \"defined_in_file\":\"core/src/lib.rs\",\
             \"ambiguous\":false,\
             \"affected_files\":[{\"file\":\"app/src/main.rs\",\"lines\":[3,10]}],\
             \"affected_crates\":[\"rws-app\"],\
             \"affected_routes\":[\"/items/:id\"],\
             \"breaking_risk\":\"medium\",\
             \"requires_human_approval\":true,\
             \"verdict\":\"requires human approval (impact spans multiple crates or public routes)\"}"
        );
    }

    #[test]
    fn render_report_singularizes_defined_in_when_ambiguous() {
        let mut report = sample_report();
        report.defined_in_crates = vec!["rws-core".to_string(), "rws-app".to_string()];
        report.defined_in_files = vec!["core/src/lib.rs".to_string(), "app/src/lib.rs".to_string()];
        report.ambiguous = true;
        let json = render_report(&report);
        assert!(json.contains("\"defined_in_crate\":\"rws-core\""));
        assert!(json.contains("\"defined_in_file\":\"core/src/lib.rs\""));
        assert!(json.contains("\"ambiguous\":true"));
        // 先頭要素のみが出力され、2 件目（"rws-app" 単体としての値）が
        // defined_in_crate の値としては現れないこと。
        assert!(!json.contains("\"defined_in_crate\":\"rws-app\""));
    }

    #[test]
    fn render_report_defensively_nulls_empty_defined_in_vecs() {
        let mut report = sample_report();
        report.defined_in_crates = vec![];
        report.defined_in_files = vec![];
        let json = render_report(&report);
        assert!(json.contains("\"defined_in_crate\":null"));
        assert!(json.contains("\"defined_in_file\":null"));
    }

    #[test]
    fn render_report_renders_empty_affected_collections_as_empty_arrays() {
        let mut report = sample_report();
        report.affected_files = vec![];
        report.affected_crates = vec![];
        report.affected_routes = vec![];
        report.breaking_risk = BreakingRisk::Low;
        report.requires_human_approval = false;
        let json = render_report(&report);
        assert!(json.contains("\"affected_files\":[]"));
        assert!(json.contains("\"affected_crates\":[]"));
        assert!(json.contains("\"affected_routes\":[]"));
        assert!(json.contains("\"breaking_risk\":\"low\""));
        assert!(json.contains(
            "\"verdict\":\"auto-applicable (impact is limited; automatic application allowed subject to gate pass)\""
        ));
    }

    #[test]
    fn verdict_text_has_fixed_two_values() {
        assert_eq!(
            verdict_text(true),
            "requires human approval (impact spans multiple crates or public routes)"
        );
        assert_eq!(
            verdict_text(false),
            "auto-applicable (impact is limited; automatic application allowed subject to gate pass)"
        );
    }

    #[test]
    fn render_report_lines_are_unquoted_numeric_arrays_in_order() {
        let mut report = sample_report();
        report.affected_files = vec![AffectedFile {
            file: "a.rs".to_string(),
            lines: vec![1, 2, 42],
        }];
        let json = render_report(&report);
        assert!(json.contains("\"lines\":[1,2,42]"));
    }

    #[test]
    fn render_report_escapes_json_injection_attempts_in_string_values() {
        let mut report = sample_report();
        report.symbol = "render".to_string();
        report.affected_files = vec![AffectedFile {
            file: "weird\"}, \"injected\": true, \"x\":\"\\n.rs".to_string(),
            lines: vec![1],
        }];
        report.affected_crates = vec!["crate\"with\\quotes".to_string()];
        report.affected_routes = vec!["/a\"b".to_string()];
        let json = render_report(&report);

        // エスケープ済みの `"` `\` を含む文字列としてそのまま値に現れ、
        // JSON 構造（キー数・ネスト）を壊す形では出現しないこと。
        assert!(json.contains("\\\"injected\\\""));
        assert!(json.contains("crate\\\"with\\\\quotes"));
        assert!(json.contains("/a\\\"b"));
        // 注入されたキーがトップレベルのキーとして解釈されていないこと
        // （エスケープされた文字列値の内部にのみ現れる）。
        assert!(!json.contains("\"injected\":true"));
    }
}
