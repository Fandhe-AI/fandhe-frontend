//! `structure.toml`（機械可読なプロジェクト構造マニフェスト、REQ-13）の
//! スキーマ v1 型定義・TOML テキストからのパース（[`parse`] / [`load`]）・
//! マニフェスト内部の整合性検証（[`StructureManifest::validate`]）。
//!
//! TASK-13.1a（#128）がスキーマ型定義と `validate()`（マニフェスト内部の宣言
//! 整合性のみを検証する純粋関数）を、TASK-13.1b（#129）が本ファイルの
//! [`parse`] / [`load`]（[`crate::toml`] の TOML サブセットパーサ結果を本モジュールの
//! 型へ変換するセマンティック検証）を提供する。`cargo metadata` や実ディレクトリ・
//! 実クレートとの突き合わせ（実体との整合性）は TASK-13.1c（#130）のスコープであり、
//! 本ファイルはファイルシステムへは [`load`] の読み込み以外でアクセスしない。
//!
//! 設計の詳細・PoC-7 からの差分理由は `docs/design/structure-manifest.md` を参照。
//!
//! `fw structure` サブコマンド（`main.rs`）からは [`load`] → [`StructureManifest::validate`]
//! の順で呼ばれ、双方が通ってはじめて TASK-13.1c の実体突き合わせ・JSON 出力へ進む
//! （`main.rs` 側の契約: パース・検証いずれかの失敗でも黙示的成功を返さない）。

/// `[directories.<name>]` の予約名: プロジェクトルート直下（ワークスペース
/// ルート自身）にクレートを直接配置する慣習を表す（`fw new`、イシュー #350/#351）。
///
/// 現行スキーマ（`^[a-z0-9_-]+$`）にはワークスペースルート自身を指す記号
/// （`.` 等）が無いため、`root` を予約名として正式化する（イシュー #353）。
/// `[directories.<name>]` のディレクトリ名 → 実ファイルシステムパスの解決は
/// [`dir_fs_path`] を単一の情報源とし、`main.rs`（実在確認）・`gate.rs`
/// （`default_escape_check` の走査対象解決）・`routes.rs`（`scan_root` の
/// 走査起点解決）・`impact.rs`（`member_dir_name` の逆変換）がいずれもこの
/// 定数・ヘルパーを参照する契約とする（個別特例の重複実装を防ぐ、
/// `docs/design/structure-manifest.md` §2.2 参照）。
pub(crate) const ROOT_DIR_KEY: &str = "root";

/// `[directories.<name>]` の `<name>` から実ファイルシステムパスを解決する
/// （[`ROOT_DIR_KEY`] の意味論の単一情報源、イシュー #353）。
///
/// `root`（予約名）は `project_dir` 自身（ワークスペースルート）へ写像し、
/// それ以外は従来どおり `project_dir.join(dir_name)` へ写像する。
/// `dir_name` は [`is_valid_directory_name`] を満たす値（呼び出し側で検証済み、
/// または `cargo metadata` から導出した安全な 1 段ディレクトリ名）を渡す契約
/// とし、本関数自体はパストラバーサル対策の検証を行わない（呼び出し元の
/// `routes::resolve_within_root` 等が別途境界検証を担う）。
pub(crate) fn dir_fs_path(project_dir: &std::path::Path, dir_name: &str) -> std::path::PathBuf {
    if dir_name == ROOT_DIR_KEY {
        project_dir.to_path_buf()
    } else {
        project_dir.join(dir_name)
    }
}

/// `[directories.<name>]` の宣言（`name` と任意の `path`）から実ファイルシステム
/// パスを解決する（イシュー #436、`crates/` 配下移設に伴う `dir_fs_path` の拡張）。
///
/// `path` が指定されていればワークスペースルート相対のその実配置（例:
/// `crates/core`）へ写像し、省略時は従来どおり [`dir_fs_path`] （`name` を
/// そのままディレクトリ名として使うフラット配置）へフォールバックする。
/// `entry.path` の値は [`StructureManifest::validate`] が事前に
/// [`is_valid_directory_name`] によるセグメント検証・段数上限・`root` への
/// 指定拒否を通過済みであることを呼び出し元の契約とする（本関数自体は
/// 検証を行わない。パストラバーサル対策は呼び出し元 `routes::resolve_within_root`
/// 等が別途境界検証を担う設計を [`dir_fs_path`] から引き継ぐ）。
pub(crate) fn dir_fs_path_for_entry(
    project_dir: &std::path::Path,
    entry: &DirectoryEntry,
) -> std::path::PathBuf {
    match &entry.path {
        Some(path) => {
            let mut fs_path = project_dir.to_path_buf();
            for segment in path.split('/') {
                fs_path.push(segment);
            }
            fs_path
        }
        None => dir_fs_path(project_dir, &entry.name),
    }
}

/// `path` キーの実配置パス制約: `/` 区切りで 1〜3 セグメント。
///
/// `crates/<name>` のような 2 段構成を想定しつつ、将来の深いネストは
/// 明示的に禁止する（無制限のネストは走査コスト・可読性の両面で
/// `docs/policy/intentional-non-adoption.md` の明示性・決定性の評価軸に反するため）。
pub(crate) const MAX_PATH_SEGMENTS: usize = 3;

/// `path` の各セグメントが [`is_valid_directory_name`] を満たし、段数が
/// [`MAX_PATH_SEGMENTS`] 以内であることを検証する（イシュー #436）。
///
/// `..`・絶対パス（先頭 `/`）・空セグメント（`//` や末尾 `/`）は
/// セグメント分割後に `is_valid_directory_name` が拒否する
/// （`..` は `.` を含む時点で文字集合外、空文字列は非空条件で拒否）。
fn is_valid_directory_path(path: &str) -> bool {
    if path.is_empty() || path.starts_with('/') || path.ends_with('/') {
        return false;
    }
    let segments: Vec<&str> = path.split('/').collect();
    !segments.is_empty()
        && segments.len() <= MAX_PATH_SEGMENTS
        && segments.iter().all(|seg| is_valid_directory_name(seg))
}

/// ディレクトリ名として許可する文字集合の検証。
///
/// `^[a-z0-9_-]+$` 相当（正規表現クレートを使わず手書きで判定する。
/// `cli` は外部依存ゼロを維持する方針のため）。絶対パス・`..`・`/` を
/// 含む名前を拒否することで、TASK-13.1c 以降のファイル走査がワークスペース外へ
/// 出るパストラバーサル面を仕様段階で塞ぐ（OWASP A01/A05 対策）。
pub(crate) fn is_valid_directory_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_' || b == b'-')
}

/// ディレクトリの役割（閉じた語彙）。
///
/// PoC-7 の自由記述文字列を廃止し、`validate()` で機械的に判定できる
/// クローズドな語彙に固定する（`docs/design/structure-manifest.md` 差分理由参照）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    /// 外部依存ゼロの描画・状態管理コア（`core` / `interactive` 相当）。
    Core,
    /// 状態管理層（`interactive` 相当）。
    State,
    /// モード非依存の共通コンポーネント（`app` 相当）。
    Component,
    /// SSR/SSG/ルーティングのサーバーエントリポイント（`server` 相当）。
    ServerEntrypoint,
    /// CSR/ハイドレーションのクライアントエントリポイント（`wasm-*` 相当）。
    ClientEntrypoint,
    /// 単一バイナリ配布層（`dist-server` 相当）。
    Distribution,
    /// 静的アセット（HTML/CSS/JS グルー、対応クレートを持たない）。
    Asset,
    /// 開発者・CI 用ツール（`xtask` / `cli` 相当）。
    Tooling,
}

impl Role {
    /// マニフェストの文字列表現との対応（TASK-13.1b のパーサが本表と
    /// 同じ語彙を用いる契約。ここで一元管理し、パーサ側での語彙の
    /// 重複定義・食い違いを防ぐ）。
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Role::Core => "core",
            Role::State => "state",
            Role::Component => "component",
            Role::ServerEntrypoint => "server-entrypoint",
            Role::ClientEntrypoint => "client-entrypoint",
            Role::Distribution => "distribution",
            Role::Asset => "asset",
            Role::Tooling => "tooling",
        }
    }

    /// `role` キーの文字列表現を [`Role`] へ変換する（TASK-13.1b パーサからの唯一の
    /// 呼び出し経路）。未知の文字列は `None`（呼び出し側が `structure.toml` 由来の
    /// 位置情報を添えてエラー化する。fail-closed、`docs/design/structure-manifest.md` §2.2.1）。
    fn parse_str(s: &str) -> Option<Self> {
        match s {
            "core" => Some(Role::Core),
            "state" => Some(Role::State),
            "component" => Some(Role::Component),
            "server-entrypoint" => Some(Role::ServerEntrypoint),
            "client-entrypoint" => Some(Role::ClientEntrypoint),
            "distribution" => Some(Role::Distribution),
            "asset" => Some(Role::Asset),
            "tooling" => Some(Role::Tooling),
            _ => None,
        }
    }
}

/// `[directories.<name>]` テーブル 1 件分の宣言。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectoryEntry {
    /// ワークスペース相対ディレクトリ名（`directories.<name>` の `<name>`）。
    /// `^[a-z0-9_-]+$` を満たすことが前提（[`is_valid_directory_name`]）。
    pub name: String,
    pub role: Role,
    /// 対応クレート名。クレートを持たないディレクトリ（`static` 等）は
    /// `None` とする（PoC-7 の `crate = ""` 空文字表現は本スキーマでは廃止）。
    pub crate_name: Option<String>,
    pub description: String,
    /// 依存を許可する `directories` キーの一覧（宣言済みキーのみ参照可）。
    pub depends_on: Vec<String>,
    /// 被依存を許可する `directories` キーの一覧（宣言済みキーのみ参照可）。
    pub allowed_dependents: Vec<String>,
    /// ワークスペースルート相対の実配置パス（例: `crates/core`）。
    ///
    /// 省略時は従来どおり `name` をそのままディレクトリ名として使う
    /// フラット配置（`fw new` が生成するユーザープロジェクトの既定、
    /// イシュー #436）。論理名（`name`・`depends_on`・`allowed_dependents`・
    /// `[routing] definition_dir`）とは独立した実体解決専用のキーであり、
    /// 依存宣言の語彙には一切影響しない。
    pub path: Option<String>,
}

/// `[routing]` テーブル。ルート定義を許すディレクトリと、その抽出方式を宣言する。
///
/// PoC-7 の `handler_pattern`（マニフェスト由来の任意正規表現をツールが実行する
/// 設計）は本スキーマでは廃止した。`server/src/router.rs` が正規表現・
/// バックトラックを排した設計（DoS 耐性）であるのに対し、任意正規表現を
/// マニフェスト経由で実行させる経路は ReDoS・インジェクション面を広げるため
/// （`docs/design/structure-manifest.md` 差分理由参照）。抽出器自体の実装は TASK-13.1c
/// （#130）のスコープであり、ここでは組み込み抽出器の ID のみを宣言する。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoutingConfig {
    /// ルート定義を許すディレクトリ（`directories` キーを参照する）。
    pub definition_dir: String,
    /// 組み込み抽出器 ID（例: `"fandhe-frontend-router-v1"`）。自由記述の正規表現は持たない。
    pub extractor: String,
}

/// `structure.toml` v1 全体を表す型。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructureManifest {
    /// `[manifest] version`。前方互換判定の基盤（TASK-13.1b 以降が使用）。
    pub version: u32,
    pub directories: Vec<DirectoryEntry>,
    pub routing: Option<RoutingConfig>,
}

/// [`StructureManifest::validate`] が返す整合性違反。
///
/// エラーメッセージはユーザー向け文字列の英語方針（`japanese-style.md`）に
/// 従い英語で表現する。内部パス以上の機微情報は含まない。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    /// `directories` が 1 件も宣言されていない。
    NoDirectories,
    /// ディレクトリ名が `^[a-z0-9_-]+$` を満たさない
    /// （絶対パス・`..`・パス区切りを含む名前など）。
    InvalidDirectoryName(String),
    /// `directories` に同名のエントリが複数宣言されている。
    ///
    /// 名前は参照解決で `HashSet<&str>` に集約されるため、重複したまま
    /// 通すと `find()` が最初に一致した要素にのみ束縛され、参照検証・
    /// 非対称性検証が意図しないエントリに対して判定される（内部一貫性が
    /// 実際には保証されない）。そのため重複自体を明示的に拒否する。
    DuplicateDirectoryName(String),
    /// `depends_on` / `allowed_dependents` が宣言済みキーを参照していない。
    UnknownReference {
        from: String,
        field: &'static str,
        target: String,
    },
    /// `depends_on` / `allowed_dependents` に自己参照が含まれる。
    SelfReference { name: String, field: &'static str },
    /// `depends_on` / `allowed_dependents` 内に重複要素がある。
    DuplicateReference {
        name: String,
        field: &'static str,
        target: String,
    },
    /// `role = "core"` のエントリが `depends_on` を持つ（REQ-3 の
    /// core 外部依存ゼロ規約をマニフェスト側にも反映する制約）。
    CoreRoleHasDependencies { name: String },
    /// `depends_on` と `allowed_dependents` の宣言が対称でない
    /// （A の `depends_on` に B があるのに B の `allowed_dependents` に A がない等）。
    AsymmetricDependency { from: String, to: String },
    /// `[routing] definition_dir` が宣言済み `directories` キーを参照していない。
    UnknownRoutingDefinitionDir(String),
    /// `path` の書式が不正（絶対パス・`..`・空セグメント・4 段以上・
    /// セグメントが `^[a-z0-9_-]+$` を満たさない、イシュー #436）。
    InvalidDirectoryPath { name: String, path: String },
    /// `path` が [`ROOT_DIR_KEY`]（`root`）エントリに指定されている。
    ///
    /// `root` は「ワークスペースルート自身」を表す予約名であり、実配置は
    /// 常にワークスペースルートに固定される。`path` で別の実体を指すことを
    /// 許すと `dir_fs_path` の `root` 特例の意味論と衝突するため拒否する。
    PathOnRootEntry { name: String },
    /// 複数の `directories` エントリが同一の実配置パスへ解決される
    /// （`name`/`path` の組み合わせ違いによる実体の重複、イシュー #436）。
    DuplicateResolvedPath {
        first: String,
        second: String,
        resolved: String,
    },
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::NoDirectories => {
                write!(f, "structure.toml: at least one `[directories.<name>]` entry is required")
            }
            ValidationError::InvalidDirectoryName(name) => {
                write!(
                    f,
                    "structure.toml: invalid directory name `{name}` (must match ^[a-z0-9_-]+$)"
                )
            }
            ValidationError::DuplicateDirectoryName(name) => {
                write!(f, "structure.toml: duplicate `[directories.{name}]` entry")
            }
            ValidationError::UnknownReference { from, field, target } => write!(
                f,
                "structure.toml: directories.{from}.{field} references unknown directory `{target}`"
            ),
            ValidationError::SelfReference { name, field } => write!(
                f,
                "structure.toml: directories.{name}.{field} must not reference itself"
            ),
            ValidationError::DuplicateReference { name, field, target } => write!(
                f,
                "structure.toml: directories.{name}.{field} contains duplicate entry `{target}`"
            ),
            ValidationError::CoreRoleHasDependencies { name } => write!(
                f,
                "structure.toml: directories.{name} has role = \"core\" but depends_on is not empty"
            ),
            ValidationError::AsymmetricDependency { from, to } => write!(
                f,
                "structure.toml: dependency between `{from}` and `{to}` is declared asymmetrically (directories.{from}.depends_on and directories.{to}.allowed_dependents disagree)"
            ),
            ValidationError::UnknownRoutingDefinitionDir(target) => write!(
                f,
                "structure.toml: [routing].definition_dir references unknown directory `{target}`"
            ),
            ValidationError::InvalidDirectoryPath { name, path } => write!(
                f,
                "structure.toml: directories.{name}.path `{path}` is invalid (must be 1-3 `/`-separated segments matching ^[a-z0-9_-]+$, no `..` or absolute paths)"
            ),
            ValidationError::PathOnRootEntry { name } => write!(
                f,
                "structure.toml: directories.{name} must not set `path` (the `root` reserved name always resolves to the workspace root itself)"
            ),
            ValidationError::DuplicateResolvedPath { first, second, resolved } => write!(
                f,
                "structure.toml: directories.{first} and directories.{second} both resolve to the same path `{resolved}`"
            ),
        }
    }
}

impl std::error::Error for ValidationError {}

impl StructureManifest {
    /// `directories.<name>` キーからワークスペースルート相対の実配置パスを
    /// 解決する（イシュー #436。`routes.rs`/`impact.rs`/`gate.rs` が走査対象・
    /// 実体解決を行う際の単一の情報源）。
    ///
    /// `name` が [`ROOT_DIR_KEY`] の場合はそのまま `"root"` を返す
    /// （`resolve_within_root` 等の呼び出し側が引き続き `ROOT_DIR_KEY` の
    /// 特例分岐で解決する契約、[`dir_fs_path`] の意味論を維持）。
    /// `name` が宣言されていない場合は `name` をそのまま返す（呼び出し元が
    /// 未知キーをこの時点までに検証済みである契約を前提とし、ここでの
    /// フォールバックはあくまで防御的措置）。
    pub(crate) fn resolved_dir_path(&self, name: &str) -> String {
        if name == ROOT_DIR_KEY {
            return name.to_string();
        }
        self.directories
            .iter()
            .find(|d| d.name == name)
            .and_then(|d| d.path.clone())
            .unwrap_or_else(|| name.to_string())
    }

    /// マニフェスト内部の宣言同士の整合性を検証する。
    ///
    /// ファイルシステム・`cargo metadata` へは一切アクセスしない純粋関数
    /// （実体との突き合わせは TASK-13.1c のスコープ、`docs/design/structure-manifest.md`
    /// 2.3 節参照）。検出した違反はすべて収集して返す（最初の 1 件で打ち切らない）。
    ///
    /// # Errors
    ///
    /// 1 件以上の [`ValidationError`] を検出した場合、それらをまとめて返す。
    pub fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        if self.directories.is_empty() {
            errors.push(ValidationError::NoDirectories);
            // directories が空なら以降の参照検証は無意味なため打ち切る。
            return Err(errors);
        }

        // 重複ディレクトリ名検出: known_names（HashSet）に集約する前に検出する。
        // 集約後は同名の 2 件目以降が握りつぶされ、以降の参照検証・非対称性検証が
        // `find()` で最初に一致した要素にのみ束縛されてしまうため、ここで先に弾く。
        {
            let mut seen_names: std::collections::HashSet<&str> = std::collections::HashSet::new();
            for dir in &self.directories {
                if !seen_names.insert(dir.name.as_str()) {
                    errors.push(ValidationError::DuplicateDirectoryName(dir.name.clone()));
                }
            }
        }

        let known_names: std::collections::HashSet<&str> =
            self.directories.iter().map(|d| d.name.as_str()).collect();

        for dir in &self.directories {
            if !is_valid_directory_name(&dir.name) {
                errors.push(ValidationError::InvalidDirectoryName(dir.name.clone()));
            }

            check_reference_list(
                &dir.name,
                "depends_on",
                &dir.depends_on,
                &known_names,
                &mut errors,
            );
            check_reference_list(
                &dir.name,
                "allowed_dependents",
                &dir.allowed_dependents,
                &known_names,
                &mut errors,
            );

            if matches!(dir.role, Role::Core) && !dir.depends_on.is_empty() {
                errors.push(ValidationError::CoreRoleHasDependencies {
                    name: dir.name.clone(),
                });
            }
        }

        // 対称性検証: depends_on と allowed_dependents は宣言の両面であり、
        // 片方だけの宣言（片落ち）を見逃さないよう双方向に突き合わせる。
        for dir in &self.directories {
            // depends_on → allowed_dependents 方向。
            for target in &dir.depends_on {
                let Some(target_dir) = self.directories.iter().find(|d| &d.name == target) else {
                    // 未知参照は上の check_reference_list で既に記録済みのためスキップ。
                    continue;
                };
                if !target_dir.allowed_dependents.iter().any(|n| n == &dir.name) {
                    errors.push(ValidationError::AsymmetricDependency {
                        from: dir.name.clone(),
                        to: target.clone(),
                    });
                }
            }
            // allowed_dependents → depends_on 方向（逆方向。片側だけの
            // 欠落を見逃さないため、depends_on 側からの探索だけでなく
            // allowed_dependents 側からも突き合わせる）。
            for accessor in &dir.allowed_dependents {
                let Some(accessor_dir) = self.directories.iter().find(|d| &d.name == accessor)
                else {
                    // 未知参照は上の check_reference_list で既に記録済みのためスキップ。
                    continue;
                };
                if !accessor_dir.depends_on.iter().any(|n| n == &dir.name) {
                    errors.push(ValidationError::AsymmetricDependency {
                        from: accessor.clone(),
                        to: dir.name.clone(),
                    });
                }
            }
        }

        if let Some(routing) = &self.routing {
            if !known_names.contains(routing.definition_dir.as_str()) {
                errors.push(ValidationError::UnknownRoutingDefinitionDir(
                    routing.definition_dir.clone(),
                ));
            }
        }

        // `path` キーの検証（イシュー #436）: 書式・`root` エントリへの指定拒否・
        // 解決先の重複を fail-closed で検出する。解決先の衝突検出は
        // `dir_fs_path_for_entry` 相当のロジックを文字列レベルで再現し、
        // ファイルシステムへは一切アクセスしない（本関数は純粋関数のまま）。
        {
            let mut resolved_paths: std::collections::HashMap<String, &str> =
                std::collections::HashMap::new();
            for dir in &self.directories {
                let Some(path) = &dir.path else {
                    // `path` 省略時は `name` がそのまま実体（フラット配置）。
                    let resolved = dir.name.clone();
                    if let Some(existing) = resolved_paths.insert(resolved.clone(), &dir.name) {
                        errors.push(ValidationError::DuplicateResolvedPath {
                            first: existing.to_string(),
                            second: dir.name.clone(),
                            resolved,
                        });
                    }
                    continue;
                };

                if dir.name == ROOT_DIR_KEY {
                    errors.push(ValidationError::PathOnRootEntry {
                        name: dir.name.clone(),
                    });
                    continue;
                }

                if !is_valid_directory_path(path) {
                    errors.push(ValidationError::InvalidDirectoryPath {
                        name: dir.name.clone(),
                        path: path.clone(),
                    });
                    continue;
                }

                if let Some(existing) = resolved_paths.insert(path.clone(), &dir.name) {
                    errors.push(ValidationError::DuplicateResolvedPath {
                        first: existing.to_string(),
                        second: dir.name.clone(),
                        resolved: path.clone(),
                    });
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// [`parse`] / [`load`] の失敗を表す。TOML 構文レベルの失敗（[`crate::toml`]）と
/// セマンティック検証（必須キー欠落・未知キー・`role` 未知語彙等）の失敗を区別する。
///
/// [`StructureManifest::validate`] の [`ValidationError`] とは責務が異なる:
/// こちらは「TOML テキスト → 型」の変換段階、`validate()` は「型 → 宣言整合性」の
/// 検証段階を担う（呼び出し順は `load`/`parse` → `validate` が契約）。
#[derive(Debug, Clone, PartialEq)]
pub enum StructureError {
    /// 下層の TOML 構文パース（[`crate::toml::parse`]）に失敗した。
    Toml(crate::toml::TomlError),
    /// セマンティック検証（必須キー欠落・型不一致・未知キー・未知 `role` 等）に失敗した。
    Semantic(String),
}

impl std::fmt::Display for StructureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StructureError::Toml(e) => write!(f, "structure.toml is not valid TOML: {e}"),
            StructureError::Semantic(msg) => {
                write!(f, "structure.toml failed schema validation: {msg}")
            }
        }
    }
}

impl std::error::Error for StructureError {}

impl From<crate::toml::TomlError> for StructureError {
    fn from(e: crate::toml::TomlError) -> Self {
        StructureError::Toml(e)
    }
}

fn semantic_err(message: impl Into<String>) -> StructureError {
    StructureError::Semantic(message.into())
}

/// `input` を `structure.toml` としてパースし [`StructureManifest`] へ変換する
/// （TASK-13.1b、`fw structure` の唯一の TOML → 型 変換経路）。
///
/// 変換のみを行い、`directories` 間の宣言整合性（依存の対称性等）は検証しない。
/// 呼び出し側（`main.rs` の `run_structure`）は本関数の成功後、必ず
/// [`StructureManifest::validate`] を呼んで宣言整合性を確認する契約とする。
///
/// # Errors
///
/// TOML 構文が壊れている場合・スキーマ上必須のキーが欠落している場合・
/// 未知のキー / 未知の `role` 値が含まれる場合に [`StructureError`] を返す
/// （fail-closed。`docs/design/structure-manifest.md` §2.1/§2.2 の方針に従う）。
pub fn parse(input: &str) -> Result<StructureManifest, StructureError> {
    let doc = crate::toml::parse(input)?;

    let version = parse_manifest_table(&doc)?;

    let mut directories: Vec<DirectoryEntry> = Vec::new();
    let mut routing: Option<RoutingConfig> = None;

    for (path, table) in doc.entries() {
        match path {
            [] => {
                // トップレベル直書きキーはこのスキーマでは許可しない
                // （`[manifest]` / `[directories.<name>]` / `[routing]` のみ許可）。
                if !table.is_empty() {
                    return Err(semantic_err(
                        "top-level keys outside `[manifest]` / `[directories.*]` / `[routing]` are not supported",
                    ));
                }
            }
            [seg] if seg == "manifest" => {
                // parse_manifest_table で処理済み。
            }
            [seg] if seg == "routing" => {
                routing = Some(parse_routing_table(table)?);
            }
            [seg, name] if seg == "directories" => {
                directories.push(parse_directory_table(name, table)?);
            }
            other => {
                return Err(semantic_err(format!(
                    "unknown table `[{}]` (expected `[manifest]`, `[directories.<name>]`, or `[routing]`)",
                    other.join(".")
                )));
            }
        }
    }

    if directories.is_empty() {
        return Err(semantic_err(
            "at least one `[directories.<name>]` table is required",
        ));
    }

    Ok(StructureManifest {
        version,
        directories,
        routing,
    })
}

/// 指定パスから `structure.toml` を読み込みパースする（`fw structure` からの
/// 唯一のファイル入出力経路）。パスはそのまま `std::fs::read_to_string` に渡し、
/// 正規化・シンボリックリンク展開は行わない（走査時のパストラバーサル対策は
/// 呼び出し側 — TASK-13.1c のファイル走査ロジック — の責務とする）。
///
/// # Errors
///
/// ファイル読み込みに失敗した場合・[`parse`] が失敗した場合に [`StructureError`] を返す。
pub fn load(path: &std::path::Path) -> Result<StructureManifest, StructureError> {
    let content = std::fs::read_to_string(path).map_err(|e| {
        semantic_err(format!(
            "failed to read structure.toml: {}",
            io_error_kind_message(&e)
        ))
    })?;
    parse(&content)
}

/// `std::io::Error` の `Display` は OS 依存メッセージ（まれに内部パスを含み得る）を
/// 出すため、機微情報露出を避けて `ErrorKind` ベースの定型メッセージに丸める
/// （security.md: エラーメッセージへの内部情報漏えい防止）。
fn io_error_kind_message(e: &std::io::Error) -> String {
    format!("{:?}", e.kind())
}

/// `[manifest]` テーブルを検証し `version` を取り出す。
fn parse_manifest_table(doc: &crate::toml::Document) -> Result<u32, StructureError> {
    let table = doc
        .table(&["manifest"])
        .ok_or_else(|| semantic_err("`[manifest]` table is required"))?;
    reject_unknown_keys(table, &["version"], "manifest")?;
    let version_value = table
        .iter()
        .find(|(k, _)| k == "version")
        .map(|(_, v)| v)
        .ok_or_else(|| semantic_err("`manifest` is missing required key `version`"))?;
    match version_value {
        crate::toml::Value::Integer(n) if *n >= 0 => Ok(*n as u32),
        _ => Err(semantic_err(
            "`manifest.version` must be a non-negative integer",
        )),
    }
}

/// `[directories.<name>]` テーブル 1 件を検証しつつ [`DirectoryEntry`] へ変換する。
fn parse_directory_table(
    name: &str,
    table: &[(String, crate::toml::Value)],
) -> Result<DirectoryEntry, StructureError> {
    let context = format!("directories.{name}");
    const ALLOWED_KEYS: &[&str] = &[
        "role",
        "crate",
        "description",
        "depends_on",
        "allowed_dependents",
        "path",
    ];
    reject_unknown_keys(table, ALLOWED_KEYS, &context)?;

    let role_str = get_str(table, "role", &context)?;
    let role = Role::parse_str(role_str).ok_or_else(|| {
        semantic_err(format!(
            "`{context}.role` has unknown value `{role_str}` (expected one of: core, state, component, server-entrypoint, client-entrypoint, distribution, asset, tooling)"
        ))
    })?;
    // `crate` はキー省略可（`static` 等クレートを持たないディレクトリ）。
    // `docs/design/structure-manifest.md` 2.2.2 節: 空文字列表現は廃止し「キー省略」に統一。
    // `crate = ""` はスキーマ上非許可（キー省略が正）のため、ここで明示的に拒否する。
    // 拒否しないまま通すと、後段の workspace-member 突き合わせ（main.rs の
    // `check_crate_and_dependency_consistency`）まで不正なマニフェストが素通りし、
    // 曖昧な失敗（空文字列に一致する workspace member は存在しないため誤検出は防げるが、
    // 意図が不明瞭なまま検証を通過してしまう）につながる
    // （レビュー指摘 #127: 空 crate 文字列を拒否するセマンティックチェックがなかった）。
    if let Some(value) = table.iter().find(|(k, _)| k == "crate").map(|(_, v)| v) {
        if value.as_str() == Some("") {
            return Err(semantic_err(format!(
                "`{context}.crate` must not be an empty string (omit the key instead)"
            )));
        }
    }
    let crate_name = get_optional_str(table, "crate", &context)?.map(str::to_string);
    let description = get_str(table, "description", &context)?.to_string();
    let depends_on = get_optional_string_array(table, "depends_on", &context)?;
    let allowed_dependents = get_optional_string_array(table, "allowed_dependents", &context)?;
    let path = get_optional_str(table, "path", &context)?.map(str::to_string);

    Ok(DirectoryEntry {
        name: name.to_string(),
        role,
        crate_name,
        description,
        depends_on,
        allowed_dependents,
        path,
    })
}

/// `[routing]` テーブルを検証しつつ [`RoutingConfig`] へ変換する。
fn parse_routing_table(
    table: &[(String, crate::toml::Value)],
) -> Result<RoutingConfig, StructureError> {
    const ALLOWED_KEYS: &[&str] = &["definition_dir", "extractor"];
    reject_unknown_keys(table, ALLOWED_KEYS, "routing")?;
    Ok(RoutingConfig {
        definition_dir: get_str(table, "definition_dir", "routing")?.to_string(),
        extractor: get_str(table, "extractor", "routing")?.to_string(),
    })
}

fn get_str<'a>(
    table: &'a [(String, crate::toml::Value)],
    key: &str,
    context: &str,
) -> Result<&'a str, StructureError> {
    let value = table
        .iter()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v)
        .ok_or_else(|| semantic_err(format!("`{context}` is missing required key `{key}`")))?;
    value
        .as_str()
        .ok_or_else(|| semantic_err(format!("`{context}.{key}` must be a string")))
}

fn get_optional_str<'a>(
    table: &'a [(String, crate::toml::Value)],
    key: &str,
    context: &str,
) -> Result<Option<&'a str>, StructureError> {
    match table.iter().find(|(k, _)| k == key).map(|(_, v)| v) {
        None => Ok(None),
        Some(value) => value
            .as_str()
            .map(Some)
            .ok_or_else(|| semantic_err(format!("`{context}.{key}` must be a string"))),
    }
}

fn get_optional_string_array(
    table: &[(String, crate::toml::Value)],
    key: &str,
    context: &str,
) -> Result<Vec<String>, StructureError> {
    match table.iter().find(|(k, _)| k == key).map(|(_, v)| v) {
        None => Ok(Vec::new()),
        Some(value) => value
            .as_string_array()
            .map(|items| items.into_iter().map(str::to_string).collect())
            .ok_or_else(|| semantic_err(format!("`{context}.{key}` must be a string array"))),
    }
}

/// テーブル内に許可リスト外のキーがないか検証する（未知キーの fail-closed 拒否）。
fn reject_unknown_keys(
    table: &[(String, crate::toml::Value)],
    allowed: &[&str],
    context: &str,
) -> Result<(), StructureError> {
    for (key, _) in table {
        if !allowed.contains(&key.as_str()) {
            return Err(semantic_err(format!("`{context}` has unknown key `{key}`")));
        }
    }
    Ok(())
}

/// `depends_on` / `allowed_dependents` 共通の参照検証（未知参照・自己参照・重複）。
///
/// [`StructureManifest::validate`] から `depends_on` と `allowed_dependents` の
/// 双方に同一ロジックを適用するための内部ヘルパー。
fn check_reference_list(
    owner: &str,
    field: &'static str,
    list: &[String],
    known_names: &std::collections::HashSet<&str>,
    errors: &mut Vec<ValidationError>,
) {
    let mut seen: std::collections::HashSet<&str> = std::collections::HashSet::new();
    for target in list {
        if target == owner {
            errors.push(ValidationError::SelfReference {
                name: owner.to_string(),
                field,
            });
            continue;
        }
        if !known_names.contains(target.as_str()) {
            errors.push(ValidationError::UnknownReference {
                from: owner.to_string(),
                field,
                target: target.clone(),
            });
            continue;
        }
        if !seen.insert(target.as_str()) {
            errors.push(ValidationError::DuplicateReference {
                name: owner.to_string(),
                field,
                target: target.clone(),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(
        name: &str,
        role: Role,
        crate_name: Option<&str>,
        depends_on: &[&str],
        allowed_dependents: &[&str],
    ) -> DirectoryEntry {
        DirectoryEntry {
            name: name.to_string(),
            role,
            crate_name: crate_name.map(str::to_string),
            description: format!("{name} directory"),
            depends_on: depends_on.iter().map(|s| s.to_string()).collect(),
            allowed_dependents: allowed_dependents.iter().map(|s| s.to_string()).collect(),
            path: None,
        }
    }

    /// このリポジトリの実構成に相当する正例（`structure.toml` の縮小版）。
    /// `validate()` が Ok になることの回帰テストを兼ねる。
    fn valid_manifest() -> StructureManifest {
        StructureManifest {
            version: 1,
            directories: vec![
                entry(
                    "core",
                    Role::Core,
                    Some("fandhe-frontend-core"),
                    &[],
                    &["app"],
                ),
                entry(
                    "app",
                    Role::Component,
                    Some("fandhe-frontend-app"),
                    &["core"],
                    &[],
                ),
            ],
            routing: Some(RoutingConfig {
                definition_dir: "app".to_string(),
                extractor: "fandhe-frontend-router-v1".to_string(),
            }),
        }
    }

    #[test]
    fn valid_manifest_passes() {
        assert_eq!(valid_manifest().validate(), Ok(()));
    }

    #[test]
    fn empty_directories_is_rejected() {
        let manifest = StructureManifest {
            version: 1,
            directories: vec![],
            routing: None,
        };
        assert_eq!(
            manifest.validate(),
            Err(vec![ValidationError::NoDirectories])
        );
    }

    #[test]
    fn invalid_directory_name_is_rejected() {
        let mut manifest = valid_manifest();
        manifest.directories[0].name = "../etc".to_string();
        // 名前を変えたので depends_on/allowed_dependents 側の参照も不整合になる
        // （未知参照・非対称）が、目的の検証対象（無効名検出）が含まれることのみ確認する。
        let errors = manifest.validate().unwrap_err();
        assert!(errors
            .iter()
            .any(|e| matches!(e, ValidationError::InvalidDirectoryName(n) if n == "../etc")));
    }

    #[test]
    fn duplicate_directory_name_is_rejected() {
        let mut manifest = valid_manifest();
        // "app" と同名のエントリを追加する（依存関係は空にして、この
        // テストが検出したい重複名の検証のみを対象にする）。
        manifest.directories.push(entry(
            "app",
            Role::Component,
            Some("fandhe-frontend-app-dup"),
            &[],
            &[],
        ));
        let errors = manifest.validate().unwrap_err();
        assert!(errors
            .iter()
            .any(|e| matches!(e, ValidationError::DuplicateDirectoryName(n) if n == "app")));
    }

    #[test]
    fn unknown_reference_is_rejected() {
        let mut manifest = valid_manifest();
        manifest.directories[1].depends_on.push("ghost".to_string());
        let errors = manifest.validate().unwrap_err();
        assert!(errors.iter().any(|e| matches!(
            e,
            ValidationError::UnknownReference { from, field, target }
                if from == "app" && *field == "depends_on" && target == "ghost"
        )));
    }

    #[test]
    fn self_reference_is_rejected() {
        let mut manifest = valid_manifest();
        manifest.directories[1].depends_on.push("app".to_string());
        let errors = manifest.validate().unwrap_err();
        assert!(errors.iter().any(
            |e| matches!(e, ValidationError::SelfReference { name, field }
                if name == "app" && *field == "depends_on")
        ));
    }

    #[test]
    fn duplicate_reference_is_rejected() {
        let mut manifest = valid_manifest();
        manifest.directories[1].depends_on.push("core".to_string());
        let errors = manifest.validate().unwrap_err();
        assert!(errors.iter().any(|e| matches!(
            e,
            ValidationError::DuplicateReference { name, field, target }
                if name == "app" && *field == "depends_on" && target == "core"
        )));
    }

    #[test]
    fn core_role_with_dependencies_is_rejected() {
        let mut manifest = valid_manifest();
        manifest.directories[0].depends_on.push("app".to_string());
        let errors = manifest.validate().unwrap_err();
        assert!(errors.iter().any(
            |e| matches!(e, ValidationError::CoreRoleHasDependencies { name } if name == "core")
        ));
    }

    #[test]
    fn asymmetric_dependency_is_rejected() {
        let mut manifest = valid_manifest();
        // core.allowed_dependents から app を外し、app.depends_on だけが core を指す片落ちにする。
        manifest.directories[0].allowed_dependents.clear();
        let errors = manifest.validate().unwrap_err();
        assert!(errors.iter().any(|e| matches!(
            e,
            ValidationError::AsymmetricDependency { from, to } if from == "app" && to == "core"
        )));
    }

    #[test]
    fn asymmetric_dependency_is_rejected_from_allowed_dependents_side() {
        // 逆方向（allowed_dependents にはあるが depends_on にない）の片落ちを検出できることを
        // 確認する回帰テスト。forward 方向（depends_on 起点）は asymmetric_dependency_is_rejected
        // で既にカバーしている。
        let mut manifest = valid_manifest();
        manifest
            .directories
            .push(entry("tool", Role::Tooling, None, &[], &[]));
        // app が tool からの依存を許可すると宣言するが、tool.depends_on 側には
        // 対応するエントリを追加しない（片落ち）。
        manifest.directories[1]
            .allowed_dependents
            .push("tool".to_string());
        let errors = manifest.validate().unwrap_err();
        assert!(errors.iter().any(|e| matches!(
            e,
            ValidationError::AsymmetricDependency { from, to } if from == "tool" && to == "app"
        )));
    }

    #[test]
    fn unknown_routing_definition_dir_is_rejected() {
        let mut manifest = valid_manifest();
        manifest.routing = Some(RoutingConfig {
            definition_dir: "ghost".to_string(),
            extractor: "fandhe-frontend-router-v1".to_string(),
        });
        let errors = manifest.validate().unwrap_err();
        assert!(errors
            .iter()
            .any(|e| matches!(e, ValidationError::UnknownRoutingDefinitionDir(t) if t == "ghost")));
    }

    /// イシュー #436: `path` キーを持つエントリが `crates/<name>` へ正しく
    /// 解決されること（正常系）。
    #[test]
    fn path_key_valid_two_segment_passes_validation() {
        let mut manifest = valid_manifest();
        manifest.directories[0].path = Some("crates/core".to_string());
        manifest.directories[1].path = Some("crates/app".to_string());
        assert_eq!(manifest.validate(), Ok(()));
    }

    #[test]
    fn path_key_rejects_dot_dot_traversal() {
        let mut manifest = valid_manifest();
        manifest.directories[0].path = Some("../etc".to_string());
        let errors = manifest.validate().unwrap_err();
        assert!(errors.iter().any(|e| matches!(
            e,
            ValidationError::InvalidDirectoryPath { name, .. } if name == "core"
        )));
    }

    #[test]
    fn path_key_rejects_absolute_path() {
        let mut manifest = valid_manifest();
        manifest.directories[0].path = Some("/etc/passwd".to_string());
        let errors = manifest.validate().unwrap_err();
        assert!(errors.iter().any(|e| matches!(
            e,
            ValidationError::InvalidDirectoryPath { name, .. } if name == "core"
        )));
    }

    #[test]
    fn path_key_rejects_uppercase_segment() {
        let mut manifest = valid_manifest();
        manifest.directories[0].path = Some("Crates/core".to_string());
        let errors = manifest.validate().unwrap_err();
        assert!(errors.iter().any(|e| matches!(
            e,
            ValidationError::InvalidDirectoryPath { name, .. } if name == "core"
        )));
    }

    #[test]
    fn path_key_rejects_four_or_more_segments() {
        let mut manifest = valid_manifest();
        manifest.directories[0].path = Some("a/b/c/d".to_string());
        let errors = manifest.validate().unwrap_err();
        assert!(errors.iter().any(|e| matches!(
            e,
            ValidationError::InvalidDirectoryPath { name, .. } if name == "core"
        )));
    }

    #[test]
    fn path_key_on_root_entry_is_rejected() {
        let mut manifest = valid_manifest();
        manifest.directories[0].name = ROOT_DIR_KEY.to_string();
        manifest.directories[0].path = Some("crates/core".to_string());
        // `depends_on`/`allowed_dependents` の参照先を書き換えたので、この
        // テストが確認したい `PathOnRootEntry` 検出のみに着目する。
        let errors = manifest.validate().unwrap_err();
        assert!(errors.iter().any(
            |e| matches!(e, ValidationError::PathOnRootEntry { name } if name == ROOT_DIR_KEY)
        ));
    }

    #[test]
    fn path_key_duplicate_resolved_path_is_rejected() {
        let mut manifest = valid_manifest();
        manifest.directories[0].path = Some("crates/shared".to_string());
        manifest.directories[1].path = Some("crates/shared".to_string());
        let errors = manifest.validate().unwrap_err();
        assert!(errors
            .iter()
            .any(|e| matches!(e, ValidationError::DuplicateResolvedPath { resolved, .. } if resolved == "crates/shared")));
    }

    #[test]
    fn dir_fs_path_for_entry_uses_path_key_when_present() {
        let mut manifest = valid_manifest();
        manifest.directories[0].path = Some("crates/core".to_string());
        let project_dir = std::path::Path::new("/tmp/some-project");
        assert_eq!(
            dir_fs_path_for_entry(project_dir, &manifest.directories[0]),
            project_dir.join("crates").join("core")
        );
    }

    #[test]
    fn dir_fs_path_for_entry_falls_back_to_name_when_path_absent() {
        let manifest = valid_manifest();
        let project_dir = std::path::Path::new("/tmp/some-project");
        assert_eq!(
            dir_fs_path_for_entry(project_dir, &manifest.directories[0]),
            project_dir.join("core")
        );
    }

    #[test]
    fn parse_accepts_path_key() {
        let input = r#"
[manifest]
version = 1

[directories.core]
role = "core"
crate = "fandhe-frontend-core"
description = "desc"
path = "crates/core"
"#;
        let manifest = parse(input).expect("path key should be accepted by parser");
        assert_eq!(manifest.directories[0].path.as_deref(), Some("crates/core"));
    }

    #[test]
    fn dir_fs_path_maps_root_key_to_project_dir_itself() {
        let project_dir = std::path::Path::new("/tmp/some-project");
        assert_eq!(dir_fs_path(project_dir, ROOT_DIR_KEY), project_dir);
    }

    #[test]
    fn dir_fs_path_maps_normal_name_to_nested_dir() {
        let project_dir = std::path::Path::new("/tmp/some-project");
        assert_eq!(dir_fs_path(project_dir, "app"), project_dir.join("app"));
    }

    #[test]
    fn role_as_str_round_trips_expected_vocabulary() {
        assert_eq!(Role::Core.as_str(), "core");
        assert_eq!(Role::State.as_str(), "state");
        assert_eq!(Role::Component.as_str(), "component");
        assert_eq!(Role::ServerEntrypoint.as_str(), "server-entrypoint");
        assert_eq!(Role::ClientEntrypoint.as_str(), "client-entrypoint");
        assert_eq!(Role::Distribution.as_str(), "distribution");
        assert_eq!(Role::Asset.as_str(), "asset");
        assert_eq!(Role::Tooling.as_str(), "tooling");
    }

    /// このリポジトリの実 `structure.toml` を縮小したフィクスチャ
    /// （TASK-13.1b, #129 の `parse()` 回帰テスト。サブモジュール未初期化環境や
    /// リポジトリルート外での `cargo test` 実行にも依存しないよう、実ファイルを
    /// 読まずインラインで保持する）。
    const FIXTURE: &str = r#"
[manifest]
version = 1

[directories.core]
role = "core"
crate = "fandhe-frontend-core"
description = "外部依存ゼロのレンダリングコア"
allowed_dependents = ["app"]

[directories.app]
role = "component"
crate = "fandhe-frontend-app"
description = "共通コンポーネント"
depends_on = ["core"]

[directories.static]
role = "asset"
description = "静的アセット（対応クレートなし）"

[routing]
definition_dir = "app"
extractor = "fandhe-frontend-router-v1"
"#;

    #[test]
    fn parse_fixture_succeeds_and_validates() {
        let manifest = parse(FIXTURE).expect("フィクスチャはスキーマを満たす");
        assert_eq!(manifest.version, 1);
        assert_eq!(manifest.directories.len(), 3);
        let core = manifest
            .directories
            .iter()
            .find(|d| d.name == "core")
            .unwrap();
        assert_eq!(core.role, Role::Core);
        assert_eq!(core.crate_name.as_deref(), Some("fandhe-frontend-core"));
        let static_dir = manifest
            .directories
            .iter()
            .find(|d| d.name == "static")
            .unwrap();
        // `crate` キー省略時は `None`（PoC-7 の空文字列表現は本スキーマでは廃止済み）。
        assert_eq!(static_dir.crate_name, None);
        assert!(static_dir.depends_on.is_empty());
        assert_eq!(
            manifest.routing,
            Some(RoutingConfig {
                definition_dir: "app".to_string(),
                extractor: "fandhe-frontend-router-v1".to_string(),
            })
        );
        // parse() は変換のみを行う契約であり、`validate()` は呼び出し側の責務。
        assert_eq!(manifest.validate(), Ok(()));
    }

    #[test]
    fn parse_rejects_missing_manifest_table() {
        let input = r#"
[directories.core]
role = "core"
description = "desc"
"#;
        assert!(matches!(parse(input), Err(StructureError::Semantic(_))));
    }

    #[test]
    fn parse_rejects_missing_required_key() {
        let missing_description = r#"
[manifest]
version = 1

[directories.core]
role = "core"
"#;
        assert!(parse(missing_description).is_err());
    }

    #[test]
    fn parse_rejects_unknown_key() {
        let unknown_key = r#"
[manifest]
version = 1

[directories.core]
role = "core"
description = "desc"
unexpected = "value"
"#;
        assert!(parse(unknown_key).is_err());
    }

    #[test]
    fn parse_rejects_unknown_role() {
        let unknown_role = r#"
[manifest]
version = 1

[directories.core]
role = "not-a-role"
description = "desc"
"#;
        assert!(parse(unknown_role).is_err());
    }

    #[test]
    fn parse_rejects_empty_directories() {
        let no_dirs = r#"
[manifest]
version = 1
"#;
        assert!(parse(no_dirs).is_err());
    }

    /// レビュー指摘 #127: `crate = ""` はスキーマ上非許可（キー省略が正）だが、
    /// `get_optional_str` 経由で `Some("")` へマッピングされるだけで拒否されて
    /// いなかった。空文字列は明示的にセマンティックエラーとして拒否する。
    #[test]
    fn parse_rejects_empty_crate_string() {
        let empty_crate = r#"
[manifest]
version = 1

[directories.core]
role = "core"
crate = ""
description = "desc"
"#;
        let result = parse(empty_crate);
        assert!(
            matches!(result, Err(StructureError::Semantic(_))),
            "empty `crate` string must be rejected as a semantic error, got {result:?}"
        );
    }

    #[test]
    fn parse_propagates_toml_syntax_errors() {
        let bad_toml = "[[directories]]\nrole = \"core\"\n";
        assert!(matches!(parse(bad_toml), Err(StructureError::Toml(_))));
    }

    #[test]
    fn parse_allows_missing_routing_table() {
        // `[routing]` はスキーマ上任意（このリポジトリの `structure.toml` は
        // 常に持つが、`routing: Option<RoutingConfig>` の型が示す通り
        // ルート定義を持たないプロジェクトも許容する）。
        let no_routing = r#"
[manifest]
version = 1

[directories.core]
role = "core"
description = "desc"
"#;
        let manifest = parse(no_routing).unwrap();
        assert_eq!(manifest.routing, None);
    }

    #[test]
    fn parse_does_not_panic_on_arbitrary_input() {
        let inputs = [
            "",
            "[",
            "]",
            "[directories]",
            "[directories.]",
            "key",
            "\0\0\0",
        ];
        for input in inputs {
            let _ = parse(input);
        }
    }

    #[test]
    fn load_reports_error_for_missing_file_without_leaking_path() {
        let result = load(std::path::Path::new(
            "/nonexistent/path/does-not-exist/structure.toml",
        ));
        assert!(result.is_err());
        let message = result.unwrap_err().to_string();
        assert!(!message.contains("/nonexistent/path"));
    }

    /// このリポジトリのルート `structure.toml`（TASK-13.1a/d の参照マニフェスト）が
    /// 実際に `parse()` → `validate()` を通過することの統合的な回帰テスト
    /// （`cli/tests/structure_integration.rs` の単体版。単体テストとしても
    /// 早期に壊れたスキーマ変更を検知できるようここにも置く）。
    #[test]
    fn parses_and_validates_repository_root_manifest() {
        // このテストバイナリは `crates/cli/` 配下でビルドされるため、2 段の
        // 親ディレクトリでワークスペースルートを得る（イシュー #436）。
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|p| p.parent())
            .expect("crates/cli/ has a workspace root two levels up")
            .join("structure.toml");
        let manifest = load(&root).expect("repository root structure.toml must parse");
        assert_eq!(
            manifest.validate(),
            Ok(()),
            "repository root structure.toml must satisfy internal validation"
        );
    }
}
