//! 静的アセット配信の入口。開発 / 本番モードに応じてコンパイル時埋め込み
//! テーブル検索とファイルシステム直接読み込みを切り替える（TASK-10.1a、
//! イシュー #106）。`routes.rs` の `/static/` プレフィックス分岐からのみ
//! 呼ばれ、`main.rs`（HTTP 層）は本モジュールの型を直接扱わない。
//!
//! # モード切り替え（コンパイル時判定表）
//!
//! | ビルド条件 | モード | 実装 |
//! |-----------|--------|------|
//! | `debug_assertions` かつ `not(feature = "force-embed")` | [`AssetMode::DevFilesystem`] | [`dev_fs`]（実行時に `static/` から読む） |
//! | 上記以外（release、または `force-embed` 有効） | [`AssetMode::Embedded`] | [`embedded_lookup`]（`build.rs` 生成テーブルの完全一致検索） |
//!
//! [`lookup`] 自体が `cfg` で 2 実装に分岐しており、release ビルドには
//! `dev_fs` のファイルシステムアクセスコードが**構造的に含まれない**
//! （`force-embed` フィーチャーで debug ビルドのまま本番相当の埋め込み経路を
//! CI 検証できる、`dist-server/Cargo.toml` 参照）。
//!
//! `DevFilesystem` モードの [`lookup`] は `dev_fs::lookup` が `None`（未検出）を
//! 返した場合に [`embedded_lookup`] へフォールバックする。WASM ビルド成果物
//! （TASK-10.2b、イシュー #110。`dist-server/build.rs` 参照）はソースツリー
//! `static/` に実体を持たず `OUT_DIR` 完結で埋め込まれるため、この
//! フォールバックがないと dev モードで `/static/wasm/*` が 404 になる。
//!
//! # セキュリティ不変条件（パストラバーサル、REQ 系 OWASP A01）
//!
//! - [`embedded_lookup`] はコンパイル時に確定した固定テーブルへの完全一致検索
//!   のみを行い、実行時にファイルシステムへアクセスしない。`../` を含むパスや
//!   URL エンコードされたパストラバーサル試行はテーブル中のいずれのキーとも
//!   完全一致しないため常に `None`（404 相当）となる。
//! - [`dev_fs::lookup`] は「`static/` ルート配下のみ読める」ことを
//!   事前拒否（`..`・絶対パス成分・NUL 混入の検査）と事後検証
//!   （`fs::canonicalize` 後の `starts_with` によるルート内包含確認）の
//!   二重防御で保証する。パーセントデコードは行わない（`dev_fs` モジュール
//!   ドキュメント参照）。

use std::borrow::Cow;

include!(concat!(env!("OUT_DIR"), "/embedded_assets.rs"));

/// 静的アセット配信モード（コンパイル時に確定し、実行時には変化しない）。
///
/// `main.rs` の起動ログ・テストから [`active_mode`] 経由で参照される。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetMode {
    /// `build.rs` 生成テーブルへの完全一致検索のみで配信する（release、
    /// または `force-embed` フィーチャー有効時）。実行時のファイルシステム
    /// アクセスは発生しない。
    Embedded,
    /// `static/` ディレクトリから実行時に読み込む（debug かつ `force-embed`
    /// 無効時のみ）。開発時のリビルドなし反映を実現する（TASK-10.1b、
    /// イシュー #107 のスコープ）。
    DevFilesystem,
}

/// 現在のビルドで有効なアセット配信モードを返す。
///
/// `cfg!` はコンパイル時に確定する定数式であり、実行時分岐ではない
/// （[`lookup`] 自体は cfg 属性で 2 実装に分岐しており、本関数はその判定条件を
/// 起動ログ・テストへ公開するための問い合わせ用途）。
pub const fn active_mode() -> AssetMode {
    if cfg!(all(debug_assertions, not(feature = "force-embed"))) {
        AssetMode::DevFilesystem
    } else {
        AssetMode::Embedded
    }
}

/// URL パス（例: `"/static/view-transitions.js"`）から埋め込み済みバイト列を
/// 引く。一致しなければ `None`（呼び出し元が 404 を返す）。
///
/// モード非依存で常にコンパイルされる（[`AssetMode::DevFilesystem`] でも
/// [`dev_fs`] 単体テストの比較対象・CI の `force-embed` 検証対象として使う）。
pub fn embedded_lookup(url_path: &str) -> Option<&'static [u8]> {
    EMBEDDED_ASSETS
        .iter()
        .find(|(path, _)| *path == url_path)
        .map(|(_, bytes)| *bytes)
}

/// URL パスからアセットのバイト列を引く（[`routes.rs`](crate::routes) の
/// `/static/` プレフィックス分岐から呼ばれる公開入口）。
///
/// [`AssetMode::Embedded`] では [`embedded_lookup`] の借用をそのまま返し
/// （`Cow::Borrowed`）、[`AssetMode::DevFilesystem`] では [`dev_fs::lookup`]
/// が読み込んだ所有バイト列を返す（`Cow::Owned`）。呼び出し元はいずれの
/// モードでも同一シグネチャで扱える。
#[cfg(all(debug_assertions, not(feature = "force-embed")))]
pub fn lookup(url_path: &str) -> Option<Cow<'static, [u8]>> {
    // WASM 成果物（TASK-10.2b、イシュー #110）はソースツリー `static/` へ
    // 書き込まれず `build.rs` の OUT_DIR で完結する（再ビルドループ回避、
    // `build.rs` 冒頭ドキュメント参照）。そのため dev モードでもファイル
    // システムに実体が存在せず、`dev_fs::lookup` は常に `None` を返す。
    // `embedded_lookup`（コンパイル時固定テーブルの完全一致検索のみ・実行時
    // ファイルシステムアクセスなし）へフォールバックすることで、dev/release
    // 双方で `/static/wasm/*` を配信できるようにする。このフォールバックは
    // 既存のパストラバーサル不変条件（`embedded_lookup` は完全一致検索のみ）
    // を変えない — 新しい実行時 FS アクセス経路を追加するわけではない。
    dev_fs::lookup(url_path)
        .map(Cow::Owned)
        .or_else(|| embedded_lookup(url_path).map(Cow::Borrowed))
}

/// [`lookup`] の本番（[`AssetMode::Embedded`]）実装。`dev_fs` を一切参照
/// しないため、release バイナリにファイルシステム読み込みコードが
/// 含まれないことをコンパイラが構造的に保証する。
#[cfg(not(all(debug_assertions, not(feature = "force-embed"))))]
pub fn lookup(url_path: &str) -> Option<Cow<'static, [u8]>> {
    embedded_lookup(url_path).map(Cow::Borrowed)
}

/// 開発モード専用: `static/` ディレクトリを実行時に読み込む。
///
/// `#[cfg(all(debug_assertions, not(feature = "force-embed")))]` でゲートされ、
/// release ビルド・`force-embed` 有効時にはコンパイルされない
/// （[`AssetMode::Embedded`] のバイナリにファイルシステムアクセス経路が
/// 構造的に存在しないことの根拠）。
#[cfg(all(debug_assertions, not(feature = "force-embed")))]
mod dev_fs {
    use std::fs;
    use std::path::{Path, PathBuf};

    /// URL パスプレフィックス。この文字列で始まらないパスは即座に拒否する。
    const STATIC_PREFIX: &str = "/static/";

    /// `static/` ディレクトリの絶対パスを返す。
    ///
    /// `build.rs`（`dist-server/build.rs`）と同一の基準
    /// （`CARGO_MANIFEST_DIR` の親 + `static`）を用いる。開発ビルドは
    /// ソースツリー上での実行が前提であり、`CARGO_MANIFEST_DIR` は
    /// コンパイル時に埋め込まれる定数のためリクエスト入力に依存しない
    /// （`parent()` が `None` になるのは `CARGO_MANIFEST_DIR` がファイル
    /// システムルートの場合のみで、cargo ワークスペース構成では実質
    /// 到達不能）。ただし `coding-rust.md`「ライブラリコードでの
    /// `unwrap()`/`expect()`/`panic!` を避ける」に従い、`parent()` 不在時も
    /// パニックせず `static/`（カレントディレクトリ相対）へフォールバック
    /// する。フォールバック時は本来の `static/` を指さない可能性があるが、
    /// [`resolve_under_root`] が `canonicalize` 失敗を `None`（404）に丸める
    /// ため安全側に倒れる。
    fn static_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .map(|workspace_root| workspace_root.join("static"))
            .unwrap_or_else(|| PathBuf::from("static"))
    }

    /// URL パスから `static/` ルート配下のファイルを読み込む。
    ///
    /// # セキュリティ不変条件（OWASP A01 パストラバーサル、二重防御）
    ///
    /// 1. **事前拒否**（文字列検査。パーセントデコードは行わない —
    ///    `..%2f` 等はデコードされないため単なる不明ファイル名として
    ///    扱われ 404 になる。デコード処理を書かない = 実装漏れによる
    ///    回避経路を作らない方針）:
    ///    - `STATIC_PREFIX` で始まらない → 拒否
    ///    - 除去後の相対部分が空、`..` 成分を含む、絶対パス/ルート成分
    ///      （`/` 始まり・Windows ドライブ文字等）を含む、NUL を含む
    ///      のいずれか → 拒否
    /// 2. **事後検証**: 結合後パスを `fs::canonicalize` し、`static/` ルート
    ///    （同じく canonicalize 済み）配下（`starts_with`）であることを
    ///    確認する。シンボリックリンク経由でルート外へ脱出する試みは
    ///    ここで遮断される（1 だけでは防げない）。
    ///
    /// 読み込み失敗（NotFound・権限不足・canonicalize 失敗等）はすべて
    /// `None` に丸め、エラー詳細・内部絶対パスをレスポンスへ露出しない
    /// （`security.md`「機微情報の露出」）。
    pub fn lookup(url_path: &str) -> Option<Vec<u8>> {
        resolve_under_root(&static_root(), url_path)
    }

    /// [`lookup`] の本体。`root` を引数化することで、実運用の `static_root()`
    /// に依存せず一時ディレクトリ上でシンボリックリンク脱出ケースを
    /// 単体テストできるようにする（`static_root()` は `env!` 定数に固定され
    /// テスト側から差し替えられないため）。
    fn resolve_under_root(root: &Path, url_path: &str) -> Option<Vec<u8>> {
        let relative = url_path.strip_prefix(STATIC_PREFIX)?;
        if !is_safe_relative_path(relative) {
            return None;
        }

        let candidate = root.join(relative);

        // canonicalize はファイルが実在しない場合も失敗するため、存在確認と
        // シンボリックリンク解決を同時に行う。失敗は「404 として扱う」に
        // 丸め、OS エラー種別・絶対パスは呼び出し元へ伝えない。
        let canonical_root = fs::canonicalize(root).ok()?;
        let canonical_candidate = fs::canonicalize(&candidate).ok()?;

        if !canonical_candidate.starts_with(&canonical_root) {
            // 事前拒否をすり抜けたシンボリックリンク等によるルート外脱出。
            // 事後検証が最終防衛線として遮断する。
            return None;
        }

        fs::read(&canonical_candidate).ok()
    }

    /// `static/` からの相対パス片が安全か検査する（事前拒否、上記 doc 参照）。
    fn is_safe_relative_path(relative: &str) -> bool {
        if relative.is_empty() || relative.contains('\0') {
            return false;
        }
        let path = Path::new(relative);
        if path.is_absolute() {
            return false;
        }
        path.components()
            .all(|component| matches!(component, std::path::Component::Normal(_)))
    }

    #[cfg(test)]
    mod tests {
        use super::lookup;

        #[test]
        fn reads_existing_static_file_matching_source_on_disk() {
            let bytes = lookup("/static/view-transitions.js")
                .expect("static/view-transitions.js exists in workspace tree");
            let text = std::str::from_utf8(&bytes).expect("static asset is UTF-8 text");
            assert!(text.contains("withViewTransition"));
        }

        #[test]
        fn unknown_file_returns_none() {
            assert!(lookup("/static/does-not-exist.js").is_none());
        }

        #[test]
        fn dot_dot_traversal_is_rejected_before_filesystem_access() {
            assert!(lookup("/static/../Cargo.toml").is_none());
            assert!(lookup("/static/../../etc/passwd").is_none());
        }

        #[test]
        fn percent_encoded_traversal_is_not_decoded_and_therefore_not_found() {
            // パーセントデコードを行わない方針（doc 参照）: `..%2F` はそのまま
            // 「..%2FCargo.toml」というファイル名として解決を試み、
            // static/ 配下に存在しないため 404 になる。
            assert!(lookup("/static/..%2FCargo.toml").is_none());
        }

        #[test]
        fn absolute_path_component_is_rejected() {
            assert!(lookup("/static//etc/passwd").is_none());
        }

        #[test]
        fn empty_relative_path_is_rejected() {
            assert!(lookup("/static/").is_none());
        }

        #[test]
        fn missing_static_prefix_is_rejected() {
            assert!(lookup("/other/view-transitions.js").is_none());
        }

        #[test]
        fn dotdot_relative_path_is_rejected_by_pre_check() {
            // `..` 成分を含むため `is_safe_relative_path`（事前拒否）で
            // 弾かれる。canonicalize + starts_with（事後検証）まで到達しない
            // ケース。その独立した検証は下記
            // `symlink_escaping_static_root_is_rejected_by_canonicalize_check`
            // が担う（レビュー指摘: 本テストが事後検証を検証していると誤認
            // させる命名・コメントだったため是正）。
            assert!(lookup("/static/sub/../../Cargo.toml").is_none());
        }

        /// 事前拒否（`..` 成分検査）をすり抜けるシンボリックリンク経由の
        /// ルート外脱出を、事後検証（`fs::canonicalize` + `starts_with`）が
        /// 独立して遮断することを固定する回帰テスト。
        ///
        /// `resolve_under_root` に一時ディレクトリを注入し、
        /// `static_root()`（`env!` 定数に固定され差し替え不能）に依存せず
        /// 検証する。`std::env::temp_dir` 配下に一時ディレクトリを作成し、
        /// テスト終了時に必ず削除する（他クレートの `build.rs` と同様、
        /// 追加の外部依存は使わない std のみの実装）。
        #[test]
        #[cfg(unix)]
        fn symlink_escaping_static_root_is_rejected_by_canonicalize_check() {
            use std::fs;
            use std::os::unix::fs::symlink;

            let temp_root = std::env::temp_dir().join(format!(
                "rws-dist-server-dev-fs-symlink-test-{}-{:?}",
                std::process::id(),
                std::thread::current().id()
            ));
            let fake_static = temp_root.join("static");
            let outside_secret = temp_root.join("outside").join("secret.txt");

            fs::create_dir_all(&fake_static).expect("create fake static/ dir");
            fs::create_dir_all(outside_secret.parent().expect("has parent"))
                .expect("create outside/ dir");
            fs::write(&outside_secret, b"should not be reachable").expect("write secret file");

            // `static/leak` → `../outside/secret.txt`（ルート外）へのリンク。
            // 事前拒否は `leak` という 1 コンポーネントの相対パスしか見ないため
            // 通過し、事後検証（canonicalize 後の starts_with）でのみ弾かれる。
            let link_path = fake_static.join("leak");
            symlink(&outside_secret, &link_path).expect("create escaping symlink");

            let result = super::resolve_under_root(&fake_static, "/static/leak");

            // 後始末はアサート前に行わない（失敗時に温度がわかる状態を残す
            // 意図はないが、`fs::remove_dir_all` 自体の失敗でテストの本題が
            // 隠れないよう、まず判定してから片付ける）。
            assert!(
                result.is_none(),
                "symlink escaping static/ root must be rejected by canonicalize+starts_with check"
            );

            let _ = fs::remove_dir_all(&temp_root);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::embedded_lookup;

    #[test]
    fn embedded_view_transitions_js_is_present_and_matches_source() {
        let bytes =
            embedded_lookup("/static/view-transitions.js").expect("embedded in build.rs table");
        let text = std::str::from_utf8(bytes).expect("static asset is UTF-8 text");
        assert!(text.contains("withViewTransition"));
    }

    #[test]
    fn traversal_and_unknown_paths_do_not_match_the_table() {
        assert!(embedded_lookup("/static/../Cargo.toml").is_none());
        assert!(embedded_lookup("/static/..%2FCargo.toml").is_none());
        assert!(embedded_lookup("/static/does-not-exist.js").is_none());
    }

    // `active_mode()` は cfg 条件どおりの値を返すことを両モードぶん固定する。
    // 通常の `cargo test`（debug・force-embed 無効）は DevFilesystem 側のみ、
    // `--features force-embed` / `--release` は Embedded 側のみコンパイル
    // されるため、それぞれのビルド構成で片方の assert のみが有効になる。
    #[cfg(all(debug_assertions, not(feature = "force-embed")))]
    #[test]
    fn active_mode_is_dev_filesystem_in_debug_build_without_force_embed() {
        assert_eq!(super::active_mode(), super::AssetMode::DevFilesystem);
    }

    #[cfg(not(all(debug_assertions, not(feature = "force-embed"))))]
    #[test]
    fn active_mode_is_embedded_in_release_or_force_embed_build() {
        assert_eq!(super::active_mode(), super::AssetMode::Embedded);
    }
}
