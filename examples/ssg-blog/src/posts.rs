//! `examples/ssg-blog` の固定記事データ。
//!
//! `src/main.rs` の `build_pages()` から呼ばれ、`generate_pages` に渡す
//! 出力ページ集合（`/` と各記事詳細ページ）の元データとなる。実データソース
//! （CMS・ファイルシステム上の Markdown 等）を持たない最小サンプルのため、
//! `all_posts()` は決定的な固定配列を返す（`fandhe_frontend_app::demo_items`
//! の実証パターンを踏襲）。

/// 1 記事分のデータ。`slug` は出力パス（`/posts/<slug>/`）の一部となるため
/// `fandhe_frontend_server::ssg::generate_pages` のパス検証ホワイトリスト
/// （英数字・`-`・`_`）を満たす静的リテラルのみを使う契約とする。
pub(crate) struct Post {
    pub(crate) slug: &'static str,
    pub(crate) title: &'static str,
    pub(crate) paragraphs: &'static [&'static str],
}

/// 固定記事 3 件を返す。
///
/// 2 件目のタイトルへ意図的な XSS ペイロード `<script>alert('xss')</script>`
/// を含め、`tests/ssg_output.rs` の既定エスケープ回帰（REQ-1）の入力に使う。
/// ペイロードは title/本文にのみ置き、`slug` には置かない（`slug` は
/// `generate_pages` の出力パスへ直接使われ、パス検証のホワイトリストで
/// 拒否されるため）。
pub(crate) fn all_posts() -> Vec<Post> {
    vec![
        Post {
            slug: "hello-ssg",
            title: "SSG ブログサンプルへようこそ",
            paragraphs: &[
                "このサンプルは fandhe_frontend_server::ssg::generate_pages を使い、\
                 記事一覧と各記事詳細を静的 HTML として dist/ へ書き出します。",
                "generate_pages は書き出し前に全ページのパスを検証し、\
                 1 件でも不正なパスがあれば何も書き出さずに失敗します（fail-closed）。",
            ],
        },
        Post {
            slug: "default-escaping",
            title: "<script>alert('xss')</script>",
            paragraphs: &[
                "このタイトルは既定エスケープ（REQ-1）を実演する意図的な \
                 XSS ペイロードです。",
                "ノード木 API（el / text）で組み立てた HTML は render() を \
                 経由するため、生の <script> タグとしては出力されません。",
            ],
        },
        Post {
            slug: "view-transitions",
            title: "View Transitions で滑らかな画面遷移",
            paragraphs: &[
                "各ページの <head> には @view-transition { navigation: auto; } を \
                 固定リテラルとして出力し、Cross-Document View Transitions を \
                 有効化しています。",
                "この CSS はユーザー入力を含まないため text() 経由でも \
                 既定エスケープ経路を迂回しません。",
            ],
        },
    ]
}
