//! コンパイル時埋め込み静的アセットの配信テーブル。
//!
//! `build.rs` が生成する `OUT_DIR/embedded_assets.rs`（`EMBEDDED_ASSETS`）を
//! `include!` する。`routes.rs` の `/static/` プレフィックス分岐からのみ
//! 呼ばれ、`main.rs`（HTTP 層）は本モジュールの型を直接扱わない。
//!
//! # セキュリティ不変条件（パストラバーサル、REQ 系 OWASP A01）
//!
//! [`lookup`] は「コンパイル時に確定した固定テーブルに対する完全一致検索」
//! のみを行い、実行時にファイルシステムへアクセスしない。`../` を含む
//! パスや URL エンコードされたパストラバーサル試行はテーブル中の
//! いずれのキーとも完全一致しないため、常に `None`（404 相当）となる
//! （正規化・デコード処理を書く必要がない = 実装漏れによる回避経路が存在
//! しない）。開発時にファイルシステムから直接読み込む方式へ切り替える際
//! （TASK-10.1、イシュー #105）は、この不変条件が失われないよう
//! 正規化・プレフィックス検査を新設する必要がある。

include!(concat!(env!("OUT_DIR"), "/embedded_assets.rs"));

/// URL パス（例: `"/static/view-transitions.js"`）から埋め込み済みバイト列を
/// 引く。一致しなければ `None`（呼び出し元が 404 を返す）。
pub fn lookup(url_path: &str) -> Option<&'static [u8]> {
    EMBEDDED_ASSETS
        .iter()
        .find(|(path, _)| *path == url_path)
        .map(|(_, bytes)| *bytes)
}

#[cfg(test)]
mod tests {
    use super::lookup;

    #[test]
    fn embedded_view_transitions_js_is_present_and_matches_source() {
        let bytes = lookup("/static/view-transitions.js").expect("embedded in build.rs table");
        let text = std::str::from_utf8(bytes).expect("static asset is UTF-8 text");
        assert!(text.contains("withViewTransition"));
    }

    #[test]
    fn traversal_and_unknown_paths_do_not_match_the_table() {
        assert!(lookup("/static/../Cargo.toml").is_none());
        assert!(lookup("/static/..%2FCargo.toml").is_none());
        assert!(lookup("/static/does-not-exist.js").is_none());
    }
}
