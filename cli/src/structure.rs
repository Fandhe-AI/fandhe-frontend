//! `structure.toml`（機械可読なプロジェクト構造マニフェスト、REQ-13）の
//! スキーマ v1 型定義と、マニフェスト内部の整合性検証（[`StructureManifest::validate`]）。
//!
//! TASK-13.1a（#128）のスコープはここまで。TOML テキストから本モジュールの型を
//! 構築する処理（TOML サブセットの手書きパーサ）は TASK-13.1b（#129）で実装し、
//! `cargo metadata` や実ディレクトリ・実クレートとの突き合わせ（実体との整合性）は
//! TASK-13.1c（#130）のスコープである。[`StructureManifest::validate`] は
//! **マニフェスト内部の宣言同士の整合性のみ**を検証し、ファイルシステムや
//! `cargo metadata` には一切アクセスしない（副作用なし・panic なし）。
//!
//! 設計の詳細・PoC-7 からの差分理由は `docs/structure-manifest.md` を参照。
//!
//! 本モジュールの公開 API は `fw main.rs` からまだ呼び出されない
//! （TOML パース・`structure` サブコマンドへの配線は TASK-13.1b/c のスコープ）。
//! テストからのみ使用される現時点の状態を明示するため `dead_code` を許容する。
#![allow(dead_code)]

/// ディレクトリ名として許可する文字集合の検証。
///
/// `^[a-z0-9_-]+$` 相当（正規表現クレートを使わず手書きで判定する。
/// `cli` は外部依存ゼロを維持する方針のため）。絶対パス・`..`・`/` を
/// 含む名前を拒否することで、TASK-13.1c 以降のファイル走査がワークスペース外へ
/// 出るパストラバーサル面を仕様段階で塞ぐ（OWASP A01/A05 対策）。
fn is_valid_directory_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_' || b == b'-')
}

/// ディレクトリの役割（閉じた語彙）。
///
/// PoC-7 の自由記述文字列を廃止し、`validate()` で機械的に判定できる
/// クローズドな語彙に固定する（`docs/structure-manifest.md` 差分理由参照）。
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
    const fn as_str(self) -> &'static str {
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
}

/// `[routing]` テーブル。ルート定義を許すディレクトリと、その抽出方式を宣言する。
///
/// PoC-7 の `handler_pattern`（マニフェスト由来の任意正規表現をツールが実行する
/// 設計）は本スキーマでは廃止した。`server/src/router.rs` が正規表現・
/// バックトラックを排した設計（DoS 耐性）であるのに対し、任意正規表現を
/// マニフェスト経由で実行させる経路は ReDoS・インジェクション面を広げるため
/// （`docs/structure-manifest.md` 差分理由参照）。抽出器自体の実装は TASK-13.1c
/// （#130）のスコープであり、ここでは組み込み抽出器の ID のみを宣言する。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoutingConfig {
    /// ルート定義を許すディレクトリ（`directories` キーを参照する）。
    pub definition_dir: String,
    /// 組み込み抽出器 ID（例: `"rws-router-v1"`）。自由記述の正規表現は持たない。
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
        }
    }
}

impl std::error::Error for ValidationError {}

impl StructureManifest {
    /// マニフェスト内部の宣言同士の整合性を検証する。
    ///
    /// ファイルシステム・`cargo metadata` へは一切アクセスしない純粋関数
    /// （実体との突き合わせは TASK-13.1c のスコープ、`docs/structure-manifest.md`
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

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
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
        }
    }

    /// このリポジトリの実構成に相当する正例（`structure.toml` の縮小版）。
    /// `validate()` が Ok になることの回帰テストを兼ねる。
    fn valid_manifest() -> StructureManifest {
        StructureManifest {
            version: 1,
            directories: vec![
                entry("core", Role::Core, Some("rws-core"), &[], &["app"]),
                entry("app", Role::Component, Some("rws-app"), &["core"], &[]),
            ],
            routing: Some(RoutingConfig {
                definition_dir: "app".to_string(),
                extractor: "rws-router-v1".to_string(),
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
        manifest
            .directories
            .push(entry("app", Role::Component, Some("rws-app-dup"), &[], &[]));
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
            extractor: "rws-router-v1".to_string(),
        });
        let errors = manifest.validate().unwrap_err();
        assert!(errors
            .iter()
            .any(|e| matches!(e, ValidationError::UnknownRoutingDefinitionDir(t) if t == "ghost")));
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
}
