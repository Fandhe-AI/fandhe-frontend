//! `rws-dist-server` のビルドスクリプト。
//!
//! ワークスペース直下 `static/`（`wasm-thin`/`wasm-full` の埋め込み HTML 等が
//! 置かれる想定のディレクトリ、`.claude/rules/delegation-impl.md` 参照）を
//! 走査し、`(URL パス, ファイル内容)` の静的テーブルを `OUT_DIR` に生成する。
//! `src/assets.rs` がこの生成物を `include!` して配信に使う。
//!
//! # 外部依存ゼロの理由（REQ-3）
//!
//! `rust-embed` は依存グラフの深さを構造的に 8 まで押し上げ REQ-3（深さ 6 以内）
//! に違反する（`dist-server/Cargo.toml` の実測コメント参照）。本ファイルは
//! std のみで完結する自前実装とし、`build-dependencies` を一切追加しない。
//!
//! # 埋め込みは常時有効（TASK-9.1b のスコープ）
//!
//! 開発時にファイルシステムから直接読み込む・埋め込みを強制する force-embed
//! 切り替えは TASK-10.1（イシュー #105）のスコープであり、本タスクでは
//! debug/release を問わず常にコンパイル時埋め込みとする（実行時のファイル
//! システムアクセスが一切発生しないため、`assets.rs` の配信経路は構造的に
//! パストラバーサル不能になる）。
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    // `CARGO_MANIFEST_DIR` は `dist-server/` を指す。埋め込み対象の `static/` は
    // ワークスペースルート直下にあるため一段上がる。
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let static_dir = manifest_dir
        .parent()
        .expect("dist-server/ has a parent directory (workspace root)")
        .join("static");

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR"));
    let dest_path = out_dir.join("embedded_assets.rs");

    let mut entries = Vec::new();
    collect_files(&static_dir, &static_dir, &mut entries);
    // 生成物の並び順を実行間で安定させる（テーブルの diff が無意味に揺れない
    // ようにするため。lookup 自体は線形走査で順不同でも正しく動く）。
    entries.sort();

    let mut generated = String::new();
    generated.push_str(
        "/// build.rs が `static/` から生成した埋め込みテーブル。\n\
         /// `(URL パス, ファイル内容)` の組。`assets.rs::lookup` からのみ参照される。\n\
         pub static EMBEDDED_ASSETS: &[(&str, &[u8])] = &[\n",
    );
    for (url_path, abs_path) in &entries {
        // `include_bytes!` はこの生成ファイル（OUT_DIR 側）からの相対パス解決に
        // なるため、埋め込み元ファイルは絶対パスで記述する（相対パスのまま
        // 埋め込むと `dist-server/` 基準と `OUT_DIR` 基準がずれてビルドが壊れる）。
        generated.push_str(&format!(
            "    ({url_path:?}, include_bytes!({abs_path:?}) as &[u8]),\n",
            url_path = url_path,
            abs_path = abs_path.display().to_string(),
        ));
    }
    generated.push_str("];\n");

    fs::write(&dest_path, generated).expect("write OUT_DIR/embedded_assets.rs");

    // `static/` 配下の追加・変更・削除でテーブルを再生成する。
    println!("cargo:rerun-if-changed={}", static_dir.display());
    for (_, abs_path) in &entries {
        println!("cargo:rerun-if-changed={}", abs_path.display());
    }
}

/// `dir` 以下を再帰的に走査し、`(URL パス, 絶対パス)` を `out` へ積む。
///
/// `root` は URL パス片（`"/static/" + root からの相対パス`）の算出基準。
/// シンボリックリンクは `fs::metadata`（リンク先を辿る）で判定するため、
/// ワークスペース外を指すリンクを埋め込む事故を避けるには `static/` 配下に
/// 外部リンクを置かない運用を前提とする（現時点で `static/` は単一ファイル
/// のみで実害なし。将来の運用注意点としてここに残す）。
fn collect_files(root: &Path, dir: &Path, out: &mut Vec<(String, PathBuf)>) {
    let read_dir = match fs::read_dir(dir) {
        Ok(rd) => rd,
        Err(_) => return,
    };
    for entry in read_dir.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_files(root, &path, out);
            continue;
        }
        let relative = path
            .strip_prefix(root)
            .expect("walked path is under root")
            .to_string_lossy()
            .replace(std::path::MAIN_SEPARATOR, "/");
        let url_path = format!("/static/{relative}");
        out.push((url_path, path));
    }
}
