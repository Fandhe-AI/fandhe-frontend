//! `templates/embed/embed.html`（TASK-7.1a・#52）に対する回帰テスト。
//!
//! REQ-7 の受け入れ基準「最小埋め込み構成とフルスタック構成がコンポーネント
//! ロジックに一切分岐を持たない同一関数を呼び出すこと」を成立させる、
//! 最小埋め込み側の製品版テンプレートの構造的不変条件を機械的に固定する。
//! `docs/hydration-api.md` 第 3 節で凍結された `mount_csr` 呼び出し契約への
//! 準拠と、既定エスケープ迂回経路（`innerHTML` 直接代入・`document.write`・
//! 外部 CDN 参照）が入り込んでいないことを検証する
//! （`.claude/rules/security.md` サプライチェーン対策・REQ-1 非弱体化）。
//!
//! 既存の `template_deny_config.rs` 等と同じ流儀（`workspace_root()` から
//! 相対パスでファイルを読み、行ベースの単純な文字列一致で検証する）に従う。
//! 外部 HTML パーサは追加しない（REQ-3・xtask 外部依存ゼロ方針）。

use std::path::PathBuf;

/// workspace ルート（`xtask/` の親ディレクトリ）の絶対パスを返す。
fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask/ には親ディレクトリ（workspace ルート）が存在する")
        .to_path_buf()
}

fn embed_html_path() -> PathBuf {
    workspace_root().join("templates/embed/embed.html")
}

fn read_embed_html() -> String {
    std::fs::read_to_string(embed_html_path())
        .expect("templates/embed/embed.html の読み込みに失敗した")
}

#[test]
fn embed_html_exists() {
    let path = embed_html_path();
    assert!(
        path.is_file(),
        "TASK-7.1a の成果物 templates/embed/embed.html が見つからない: {}",
        path.display()
    );
}

#[test]
fn embed_html_is_complete_document() {
    let contents = read_embed_html();
    assert!(
        contents.contains("<!DOCTYPE html>"),
        "完全な HTML 文書であることを示す <!DOCTYPE html> が見つからない"
    );
    assert!(
        contents.contains(r#"<meta charset="utf-8">"#),
        "文字コード宣言 <meta charset=\"utf-8\"> が見つからない"
    );
}

#[test]
fn embed_html_has_mount_point_div() {
    let contents = read_embed_html();
    assert!(
        contents.contains(r#"<div id="app-list">"#),
        "マウントポイント <div id=\"app-list\"> が見つからない"
    );
}

#[test]
fn embed_html_calls_mount_csr_via_frozen_contract() {
    let contents = read_embed_html();

    assert!(
        contents.contains(r#"<script type="module">"#),
        "<script type=\"module\"> が見つからない"
    );
    assert!(
        contents.contains("rws_wasm_client.js"),
        "docs/hydration-api.md 第 3 節が定める rws-wasm-client のグルー \
         コード（rws_wasm_client.js）からの import が見つからない"
    );
    assert!(
        contents.contains("mount_csr")
            && contents.contains("init")
            && contents.contains("await init()"),
        "init() を await してから mount_csr を呼ぶ凍結契約どおりの \
         呼び出し順が見つからない"
    );
    // mount_csr へ渡す root_id は、マウントポイント <div id="app-list"> と
    // 一致していなければハイドレーション対象を取り違える。
    assert!(
        contents.contains(r#"mount_csr("app-list")"#),
        "mount_csr の引数 \"app-list\" がマウントポイントの id と一致していない"
    );
}

#[test]
fn embed_html_does_not_reference_external_origins() {
    let contents = read_embed_html();
    for scheme in ["http://", "https://"] {
        assert!(
            !contents.contains(scheme),
            "外部 URL 参照（{scheme}）が見つかった。WASM/JS は同一オリジンの \
             相対パスで配信する方針（.claude/rules/security.md サプライ \
             チェーン対策）に反する"
        );
    }
}

#[test]
fn embed_html_does_not_bypass_default_escaping() {
    let contents = read_embed_html();
    for forbidden in ["innerHTML", "document.write"] {
        assert!(
            !contents.contains(forbidden),
            "既定エスケープ経路を迂回しうる {forbidden} の使用が見つかった。\
             描画は mount_csr → rws_core::render() の既定エスケープ経路のみを \
             通る必要がある（REQ-1 非弱体化）"
        );
    }
}

#[test]
fn embedding_guide_references_embed_html_template() {
    let guide_path = workspace_root().join("docs/embedding-guide.md");
    let guide_contents = std::fs::read_to_string(&guide_path)
        .unwrap_or_else(|_| panic!("{} の読み込みに失敗した", guide_path.display()));
    assert!(
        guide_contents.contains("templates/embed/embed.html"),
        "docs/embedding-guide.md が templates/embed/embed.html へ言及して \
         いない（テンプレートとガイドの相互参照が担保されていない）"
    );
}
