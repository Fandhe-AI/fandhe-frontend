//! DownloadTrigger（イシュー #828）headless コンポーネント。
//!
//! ark-ui `utilities/download-trigger.md` / chakra-ui `buttons/download-trigger.md`
//! 相当。両参考実装は JS（Blob 生成・非同期データ解決）でダウンロードを
//! 起動するが、本フレームワークは AI 時代のセキュリティリスク低減（プレーン
//! な HTML を尊重する既定方針）に基づき、`a[download]` 属性による**宣言的
//! トリガー**（JS 不要の静的部品）として実装する。
//!
//! `docs/design/component-coverage-map.md` で「保留」（根拠:
//! `docs/policy/intentional-non-adoption.md` §7「JS ランタイム固有 utilities
//! のうち静的実装可能なもの」行、#735）とされていたが、本イシュー #828 が
//! 同節の再評価トリガー「利用要望 issue の起票」を充足したため実装する
//! （既存静的部品群の実装パターンを踏襲、保留解除の記録は同 docs 参照）。
//!
//! [`mod@crate::link`] と同様、素の `a` 要素 1 パーツ（anatomy `root`）のみを
//! 提供する最小構成。開閉のような時間変化する内部状態を持たないため自由
//! 関数のみで構成する（[`crate::state`] の状態機械は適用しない）。
//!
//! # ark-ui/chakra-ui との対応表
//!
//! | 参考実装の prop | 本実装での扱い |
//! |---|---|
//! | `href`（解決済み URL 文字列。ark-ui は `url()`/`data`/`fileName` から実行時に組み立てる） | [`root`] の `href` 引数（呼び出し側があらかじめ配信 URL を渡す） |
//! | `fileName` | [`root`] の `file_name` 引数（`Some` で `download="<name>"`、`None` で `download=""`） |
//! | `data`（`Blob`/`ArrayBuffer`/非同期関数） | **非対応**。`Blob` 生成はクライアント JS 前提であり、本フレームワークの静的部品化方針の対象外 |
//! | `mimeType` | **非対応**（`data` 非対応と同じ理由。実ファイル配信時は配信側の `Content-Type` ヘッダで表現する） |
//!
//! # セキュリティ不変条件
//!
//! - `href`/`file_name`/呼び出し側 `attrs`/子ノードはすべて
//!   [`fandhe_frontend_core::el`] の属性値・子ノードとして渡り、
//!   [`fandhe_frontend_core::render`] の既定エスケープ（REQ-1）を必ず経由
//!   する。本モジュールは `raw_html()` を使用せず、HTML 文字列を直接組み立
//!   てない。
//! - `href` の URL スキーム検証は `fandhe_frontend_core::render` 側の既定
//!   経路（許可スキームのみを通す deny-by-default。`javascript:`/
//!   `vbscript:`/`data:`/`blob:` を含む不正な値は属性ごと出力されない
//!   fail-closed）に委譲し、独自の URL 検証を追加しない（[`crate::link`]
//!   と同じ整理）。ark-ui/chakra-ui の `Blob` ダウンロードと異なり、本実装
//!   は http/https/相対 URL による実ファイル配信のみを対象とする（`data:`/
//!   `blob:` スキームは安全側判断として拒否される）。
//! - ブラウザ仕様上、`download` 属性は same-origin（および `blob:`/`data:`）
//!   以外のリンクでは無視される（cross-origin の `href` では単なる通常の
//!   ナビゲーションとして扱われる）。呼び出し側はこの仕様上の制約を踏まえ
//!   て `href` を選ぶこと。
//!
//! # スコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - `examples/headless-pre-styled-ui` への追随・crates.io への公開は公開
//!   イシュー側のスコープ。
//! - json-tree-view（`docs/policy/intentional-non-adoption.md` §7 の同一行
//!   のもう一方）は引き続き保留。

use crate::anatomy::{anatomy, Anatomy};
use fandhe_frontend_core::Node;

/// DownloadTrigger の anatomy（`data-scope="download-trigger"`）。
const ANATOMY: Anatomy = anatomy("download-trigger");

/// `root` パーツ（`a[download]`）。唯一の anatomy パーツ。
///
/// - `file_name` が `Some(name)` のとき `download="<name>"` を付与する
///   （ダウンロード時のファイル名ヒント）。
/// - `file_name` が `None` のとき `download=""` を付与する（配信元のファイル
///   名を使う、ブラウザの `download` 属性の空文字列挙動）。
#[must_use]
pub fn root<'a>(
    href: &'a str,
    file_name: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&str, &str)> = vec![("href", href), ("download", file_name.unwrap_or(""))];
    merged.extend(attrs);
    ANATOMY.part("root", "a", merged, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};

    #[test]
    fn root_outputs_scope_part_href_and_download_with_file_name() {
        let html = render(&root(
            "/assets/report.pdf",
            Some("report.pdf"),
            vec![],
            vec![text("Download report")],
        ));
        assert!(html.starts_with("<a"));
        assert!(html.contains(r#"data-scope="download-trigger""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(html.contains(r#"href="/assets/report.pdf""#));
        assert!(html.contains(r#"download="report.pdf""#));
        assert!(html.contains(">Download report<"));
    }

    #[test]
    fn root_omits_file_name_emits_empty_download_attribute() {
        let html = render(&root("/assets/report.pdf", None, vec![], vec![]));
        assert!(html.contains(r#"download="""#));
    }

    #[test]
    fn caller_data_scope_and_part_spoofing_is_dropped() {
        let html = render(&root(
            "/assets/report.pdf",
            None,
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="download-trigger""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("attacker"));
    }

    // --- URL スキーム拒否（fail-closed、core の render() 経由） ---

    #[test]
    fn dangerous_url_schemes_are_rejected() {
        let dangerous_urls = [
            "javascript:alert(1)",
            "JaVaScRiPt:alert(1)",
            "java\tscript:alert(1)",
            "java\nscript:alert(1)",
            "\u{0}javascript:alert(1)",
            "data:text/html;base64,PHNjcmlwdD4=",
            "blob:https://example.com/uuid",
            "vbscript:msgbox(1)",
        ];
        for url in dangerous_urls {
            let html = render(&root(url, None, vec![], vec![]));
            assert!(
                !html.contains("href="),
                "危険な URL スキームなのに href 属性が出力されている: url={url:?}, html={html}"
            );
        }
    }

    // --- エスケープ回帰 ---

    #[test]
    fn href_attribute_breakout_payload_is_escaped() {
        let html = render(&root(
            "/assets/report.pdf\" onmouseover=\"alert(1)",
            None,
            vec![],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)\""));
    }

    #[test]
    fn file_name_attribute_breakout_payload_is_escaped() {
        let html = render(&root(
            "/assets/report.pdf",
            Some("report.pdf\" onmouseover=\"alert(1)"),
            vec![],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)\""));
    }

    #[test]
    fn children_script_payload_is_escaped() {
        let html = render(&root(
            "/assets/report.pdf",
            None,
            vec![],
            vec![text("<script>alert(1)</script>")],
        ));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
    }
}
