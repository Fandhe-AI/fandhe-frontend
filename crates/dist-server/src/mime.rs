//! 拡張子から `Content-Type` を引く固定表。
//!
//! `mime_guess` クレートの代替として自前実装する（REQ-3: 依存グラフ上限。
//! `dist-server/Cargo.toml` の実測コメント参照）。`assets.rs` の静的アセット
//! 配信のみから呼ばれ、リクエスト由来の文字列を一切受け取らない
//! （拡張子はコンパイル時に確定した埋め込みテーブルのパスから抽出するため、
//! ヘッダインジェクションの入力経路にならない）。

/// パス（例: `"/static/view-transitions.js"`）の拡張子から `Content-Type` を返す。
///
/// 未知の拡張子・拡張子なしの場合は `application/octet-stream`（`mime_guess` の
/// 既定挙動を踏襲）。返り値は常に `&'static str` の固定文言であり、
/// リクエスト文字列をそのままヘッダへ反映することはない。
pub fn content_type_for_path(path: &str) -> &'static str {
    let extension = path.rsplit('.').next().filter(|ext| *ext != path);
    match extension {
        Some("html") => "text/html; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("js") | Some("mjs") => "text/javascript; charset=utf-8",
        Some("wasm") => "application/wasm",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("ico") => "image/x-icon",
        Some("json") => "application/json; charset=utf-8",
        Some("txt") => "text/plain; charset=utf-8",
        _ => "application/octet-stream",
    }
}

#[cfg(test)]
mod tests {
    use super::content_type_for_path;

    #[test]
    fn known_extensions_map_to_expected_content_type() {
        assert_eq!(
            content_type_for_path("/static/view-transitions.js"),
            "text/javascript; charset=utf-8"
        );
        assert_eq!(
            content_type_for_path("/static/app.wasm"),
            "application/wasm"
        );
        assert_eq!(
            content_type_for_path("/static/style.css"),
            "text/css; charset=utf-8"
        );
    }

    #[test]
    fn unknown_or_missing_extension_falls_back_to_octet_stream() {
        assert_eq!(
            content_type_for_path("/static/README"),
            "application/octet-stream"
        );
        assert_eq!(
            content_type_for_path("/static/archive.tar.gz"),
            "application/octet-stream"
        );
    }
}
