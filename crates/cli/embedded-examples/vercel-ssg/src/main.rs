//! `fandhe-frontend-example-vercel-ssg`: SSG（`generate_pages`）→ Vercel
//! Build Output API → `vercel deploy --prebuilt` の正本サンプル
//! （イシュー #3290、親ツリー #3282 Phase 1 の決定を実演する）。
//!
//! # 役割・呼び出し文脈
//!
//! `docs/design/vercel-deployment-strategy.md` は Vercel 上での配布方式を
//! 案 c（SSG → Build Output API → `--prebuilt`）へ確定した（`vercel_runtime`
//! 1.x は起動時 panic・2.x は依存グラフ上限超過で不採用）。本サンプルは
//! この唯一の既定方式を動く形で示す。Vercel 側に Rust ツールチェーンは
//! 不要で、ローカルまたは CI でビルドした静的出力を配置するだけでよい。
//!
//! `examples/ssg-blog`（イシュー #501）と同じく
//! `fandhe_frontend_server::ssg::generate_pages` / `generate_assets` を使う。
//! 差分は出力先ディレクトリの構成（Vercel Build Output API v3 が要求する
//! `.vercel/output/config.json` + `.vercel/output/static/` レイアウト）と、
//! `config.json` の `routes` によるカスタム 404 の実演のみ。
//!
//! # 学べること
//!
//! - `generate_pages` による Build Output API `static/` ディレクトリの生成
//! - `generate_assets` による `config.json`（Build Output API v3）と
//!   `404.html` の生成
//! - `config.json` の `routes` で「ファイルシステムに一致しなければ 404」を
//!   表現する方法（`{"handle": "filesystem"}` の後続ルート）
//! - 古いビルド成果物を残したまま `--prebuilt` すると意図しない内容が
//!   公開されてしまうため、生成前に固定リテラルの出力先だけを削除する
//!   fail-closed な取り扱い（[`clean_output_dir`]）
//!
//! # セキュリティ不変条件（REQ-1・OWASP A01/A05）
//!
//! - HTML はすべて `fandhe_frontend_core` のノード木 API（`el` / `text` /
//!   タグヘルパー）と `render` を経由する。`raw_html()` も、タグ文字列の
//!   `format!` 組み立ても使わない（`coding-rust.md`「HTML 文字列の直接
//!   組み立て禁止」）。href 等の属性値はすべて固定リテラルで、`format!`
//!   の組み立ては行わない。本サンプルで `format!` を使うのは
//!   `<!DOCTYPE html>` の固定前置（[`generate_pages`] rustdoc が推奨する
//!   許容パターンと同じ）と、[`clean_output_dir`] の内部エラーメッセージ
//!   （HTML ではないため本規約の対象外）のみ。
//! - `config.json` の内容（[`CONFIG_JSON`]）は静的な定数リテラルで、
//!   ユーザー入力を一切混ぜない。
//! - 出力パスはすべて `generate_pages`/`generate_assets` の fail-closed
//!   検証（`..`・`/`・`\`・`.git` は拒否、全件事前検証で部分書き込みを
//!   しない）を経由する。本サンプル側で独自のパス組み立て・検証迂回は
//!   行わない。
//! - [`clean_output_dir`] が削除するのは固定リテラルの `.vercel/output`
//!   のみで、`.vercel/`（`vercel link` が作る `project.json` を含む）自体は
//!   消さない。対象がシンボリックリンクの場合は削除せずエラー終了する
//!   （fail-closed。symlink 経由でリポジトリ外を誤削除しない）。
//! - Deployment Protection（Vercel Authentication／SSO）・Basic 認証の
//!   運用は本サンプルのスコープ外（README 参照）。バイパス用トークン等の
//!   秘密情報はコード・README のいずれにも含めない。

#![forbid(unsafe_code)]

use fandhe_frontend_core::{a, el, h1, header, main_tag, p, render, text, Node};
use fandhe_frontend_server::ssg::{generate_assets, generate_pages, SsgError};
use std::path::Path;

/// Build Output API 形式の出力先ルート（固定リテラル）。
/// `vercel deploy --prebuilt` はこのディレクトリをそのままアップロードする。
const OUTPUT_ROOT: &str = ".vercel/output";
/// HTML 静的ファイルの出力先（`OUTPUT_ROOT` 配下、Build Output API 契約）。
const STATIC_DIR: &str = ".vercel/output/static";

/// ページのメタ情報（トップと各下位ページの共通データ）。
struct Page {
    /// `generate_pages` に渡すリクエストパス（`/` または `/pages/<slug>/`）。
    path: &'static str,
    /// `<title>` と見出しに使う表示名。
    title: &'static str,
    /// 本文段落。
    paragraphs: &'static [&'static str],
}

/// トップページ + 下位ページ 2 件。`default-escaping` は既定エスケープ
/// （REQ-1）の回帰テスト用に、意図的に XSS ペイロードを含む。
const PAGES: &[Page] = &[
    Page {
        path: "/",
        title: "Vercel SSG Example",
        paragraphs: &[
            "fandhe-frontend フレームワークの SSG 出力を Vercel Build Output API 形式で配置する正本サンプルです。",
        ],
    },
    Page {
        path: "/pages/about/",
        title: "About",
        paragraphs: &["cargo run --release で .vercel/output を生成し、vercel deploy --prebuilt で配置します。"],
    },
    Page {
        path: "/pages/default-escaping/",
        title: "<script>alert('xss')</script>",
        paragraphs: &["このページのタイトルは既定エスケープ（REQ-1）の回帰テストを兼ねます。"],
    },
];

/// Build Output API v3 の `config.json`。静的リテラルでユーザー入力を含まない。
///
/// - `{"handle": "filesystem"}`: 実在する静的ファイルへ通常どおり一致させる。
/// - 続く `{"src": "/(.*)", "status": 404, "dest": "/404.html"}`:
///   ファイルシステムに一致しなかった全パスへ 404 ステータスで
///   `/404.html` を返す（受け入れ基準 2）。
/// - 先頭のヘッダールート（`continue: true`）は OWASP A05 の防御を
///   上乗せする（インラインスタイルのみで CSP は追加しない。詳細は
///   [`layout`] の `<style>` を参照）。
const CONFIG_JSON: &str = r#"{
  "version": 3,
  "routes": [
    {
      "src": "/(.*)",
      "headers": {
        "X-Content-Type-Options": "nosniff",
        "Referrer-Policy": "strict-origin-when-cross-origin"
      },
      "continue": true
    },
    { "handle": "filesystem" },
    { "src": "/(.*)", "status": 404, "dest": "/404.html" }
  ]
}
"#;

/// 各ページ共通の骨格（`<html>` 全体）を組み立てる。
///
/// `examples/ssg-blog::layout` と同じ `@view-transition` 固定リテラルを
/// `text()` 経由で出力し、既定エスケープ経路を迂回しない。
fn layout(title: &str, main: Node) -> Node {
    let head = el(
        "head",
        vec![],
        vec![
            el("meta", vec![("charset", "utf-8")], vec![]),
            el(
                "meta",
                vec![
                    ("name", "viewport"),
                    ("content", "width=device-width, initial-scale=1"),
                ],
                vec![],
            ),
            el(
                "style",
                vec![],
                vec![text("@view-transition { navigation: auto; }")],
            ),
            el("title", vec![], vec![text(title)]),
        ],
    );
    let body = el(
        "body",
        vec![],
        vec![
            header(
                vec![],
                vec![a(vec![("href", "/")], vec![text("Vercel SSG Example")])],
            ),
            main,
        ],
    );
    el("html", vec![("lang", "ja")], vec![head, body])
}

/// `PAGES` から `generate_pages` へ渡す (リクエストパス, `Node`) 列を組み立てる。
fn build_pages() -> Vec<(String, Node)> {
    PAGES
        .iter()
        .map(|page| {
            let children: Vec<Node> = std::iter::once(h1(vec![], vec![text(page.title)]))
                .chain(
                    page.paragraphs
                        .iter()
                        .map(|paragraph| p(vec![], vec![text(*paragraph)])),
                )
                .collect();
            (
                page.path.to_string(),
                layout(page.title, main_tag(vec![], children)),
            )
        })
        .collect()
}

/// 404 ページの `Node` を組み立てる。
fn not_found_page() -> Node {
    layout(
        "404 Not Found",
        main_tag(
            vec![],
            vec![
                h1(vec![], vec![text("404 Not Found")]),
                p(vec![], vec![text("お探しのページは見つかりませんでした。")]),
            ],
        ),
    )
}

/// `generate_assets` に渡す (リクエストパス, コンテンツ文字列) 列を組み立てる。
///
/// `config.json` は `OUTPUT_ROOT` 直下、`404.html` は `STATIC_DIR` 配下へ
/// 出力先ディレクトリを分けて呼び出す（呼び出し元 [`main`] 参照）ため、
/// 本関数はそれぞれの単一要素スライスを返す 2 つの小さな関数に分割する。
fn config_json_asset() -> Vec<(String, String)> {
    vec![("/config.json".to_string(), CONFIG_JSON.to_string())]
}

fn not_found_asset() -> Vec<(String, String)> {
    let body = format!("<!DOCTYPE html>\n{}", render(&not_found_page()));
    vec![("/404.html".to_string(), body)]
}

/// 生成前に古い `.vercel/output` を削除する（OWASP A05: 古いビルド成果物が
/// 混ざって公開されるのを防ぐ）。
///
/// `.vercel/output` はサンプル自身の固定リテラルパスのみを対象とし、
/// `.vercel/`（`vercel link` が作る `project.json` を含む）自体は残す。
/// 対象がシンボリックリンクの場合は fail-closed でエラーを返し、削除しない
/// （リンク先を辿った意図しない削除を避ける）。
fn clean_output_dir() -> std::io::Result<()> {
    let path = Path::new(OUTPUT_ROOT);
    match std::fs::symlink_metadata(path) {
        Ok(meta) if meta.file_type().is_symlink() => Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("refusing to remove symlink at {OUTPUT_ROOT}"),
        )),
        Ok(_) => std::fs::remove_dir_all(path),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(err) => Err(err),
    }
}

/// [`main`] の失敗理由をまとめる薄いラッパー（`Display` のみを利用者へ見せ、
/// I/O エラーの内部詳細以外の機微情報は含めない、`security.md` 準拠）。
#[derive(Debug)]
enum BuildError {
    Clean(std::io::Error),
    Ssg(SsgError),
}

impl std::fmt::Display for BuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BuildError::Clean(err) => write!(f, "failed to clean {OUTPUT_ROOT}: {err}"),
            BuildError::Ssg(err) => write!(f, "{err}"),
        }
    }
}

impl From<SsgError> for BuildError {
    fn from(err: SsgError) -> Self {
        BuildError::Ssg(err)
    }
}

/// CLI エントリポイント。
///
/// 1. `clean_output_dir` で古い `.vercel/output` を削除する。
/// 2. `generate_pages` で HTML ページを `STATIC_DIR` へ書き出す。
/// 3. `generate_assets` で `404.html`（`STATIC_DIR`）と `config.json`
///    （`OUTPUT_ROOT`）を書き出す。
///
/// 成功時は書き出したファイルパスを 1 行ずつ標準出力へ、失敗時はエラーを
/// 標準エラーへ出力して非ゼロ終了する（`unwrap`/`panic!` は使わない）。
fn main() {
    let result = clean_output_dir()
        .map_err(BuildError::Clean)
        .and_then(|()| Ok(generate_pages(&build_pages(), Path::new(STATIC_DIR))?))
        .and_then(|pages| {
            let not_found = generate_assets(&not_found_asset(), Path::new(STATIC_DIR))?;
            let config = generate_assets(&config_json_asset(), Path::new(OUTPUT_ROOT))?;
            Ok(pages
                .into_iter()
                .chain(not_found)
                .chain(config)
                .collect::<Vec<_>>())
        });

    match result {
        Ok(written) => {
            for path in written {
                println!("{}", path.display());
            }
        }
        Err(err) => {
            eprintln!("failed to generate Vercel Build Output API tree: {err}");
            std::process::exit(1);
        }
    }
}
