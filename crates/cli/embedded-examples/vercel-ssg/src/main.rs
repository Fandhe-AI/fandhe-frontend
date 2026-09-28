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
//! - Deployment Protection（Vercel Authentication／SSO）の運用は本サンプルの
//!   スコープ外（README 参照）。バイパス用トークン等の秘密情報はコード・
//!   README のいずれにも含めない。
//! - Basic 認証の Routing Middleware は既定では無効の opt-in 機能
//!   （[`BASIC_AUTH_FLAG_ENV`] が正確に `"1"` のときだけ有効、イシュー
//!   #3343）。**未設定**時のみ無効扱い（[`output_root_assets`] が
//!   [`CONFIG_JSON`] の 1 件のみを返し、`.vercel/output` はミドルウェア
//!   導入前とバイト単位で同一になる）。**設定済みだが `"1"` 以外**の値
//!   （空文字・`"1"` 以外の任意の値）は、誤って無保護の成果物を生成しない
//!   よう [`basic_auth_enabled`] がエラーを返しビルド自体を失敗させる
//!   （fail-closed。「未設定＝無効」と「設定済みだが不正な値＝エラー」を
//!   区別する）。

#![forbid(unsafe_code)]

use fandhe_frontend_core::{a, el, h1, header, main_tag, p, render, text, Node};
use fandhe_frontend_server::ssg::{generate_assets, generate_pages, SsgError};
use std::ffi::OsStr;
use std::path::Path;

/// Build Output API 形式の出力先ルート（固定リテラル）。
/// `vercel deploy --prebuilt` はこのディレクトリをそのままアップロードする。
const OUTPUT_ROOT: &str = ".vercel/output";
/// HTML 静的ファイルの出力先（`OUTPUT_ROOT` 配下、Build Output API 契約）。
const STATIC_DIR: &str = ".vercel/output/static";

/// Basic 認証 Routing Middleware の有効化フラグ（ビルド時専用）。
///
/// [`basic_auth_enabled`] が判定する: 未設定は無効、`"1"` は有効、
/// それ以外の設定済みの値（空文字・`"0"`・`"true"` 等）はビルドエラー
/// とする（fail-closed）。**この環境変数は
/// `cargo run` 実行時にのみ読まれ、Vercel のランタイム（デプロイ後の
/// Edge Runtime ミドルウェア）では一切読まれない**。ランタイムで実際に
/// 認証情報として使う `BASIC_AUTH_USER`/`BASIC_AUTH_PASSWORD`（Vercel の
/// 環境変数、[`MIDDLEWARE_INDEX_JS`] が `process.env` から読む）とは
/// 役割が異なる別物であることに注意する。
const BASIC_AUTH_FLAG_ENV: &str = "FANDHE_VERCEL_SSG_BASIC_AUTH";

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

/// [`BASIC_AUTH_FLAG_ENV`] 有効時の `config.json`。
///
/// `CONFIG_JSON` の `routes` 先頭（ヘッダールートより前）へ
/// `middlewarePath` ルートを 1 件挿入した内容で、静的ファイル・404
/// フォールバックを含む全パスがミドルウェアを通過する
/// （`docs/guides/deployment.md` 「Routing Middleware による Basic 認証」
/// §「仕組み」と同じ配置）。挿入するブロック自体は [`tests`] の
/// `config_json_with_basic_auth_matches_base_config_json_with_middleware_route_removed`
/// が `CONFIG_JSON` との乖離を回帰検証する。
const CONFIG_JSON_WITH_BASIC_AUTH: &str = r#"{
  "version": 3,
  "routes": [
    {
      "src": "/(.*)",
      "middlewarePath": "_middleware",
      "continue": true
    },
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

/// [`tests::config_json_with_basic_auth_matches_base_config_json_with_middleware_route_removed`]
/// が `CONFIG_JSON_WITH_BASIC_AUTH` から取り除く固定ブロック
/// （`CONFIG_JSON` との差分そのもの）。
#[cfg(test)]
const MIDDLEWARE_ROUTE_BLOCK: &str = "    {
      \"src\": \"/(.*)\",
      \"middlewarePath\": \"_middleware\",
      \"continue\": true
    },
";

/// ミドルウェア Function の `.vc-config.json`（Build Output API v3、
/// Edge Runtime 関数の宣言）。`envVarsInUse` は実際に読む環境変数名の
/// 宣言のみで、値は含まない。
const MIDDLEWARE_VC_CONFIG_JSON: &str = r#"{
  "runtime": "edge",
  "entrypoint": "index.js",
  "envVarsInUse": ["BASIC_AUTH_USER", "BASIC_AUTH_PASSWORD"]
}
"#;

/// Basic 認証 Routing Middleware 本体（Edge Runtime、`config.json` の
/// `middlewarePath: "_middleware"` から起動される）。
///
/// `docs/guides/deployment.md` 「Routing Middleware による Basic 認証」の
/// スニペットと同一内容を保つ（乖離させない。ガイド側も本定数に合わせて
/// 更新済み）。
///
/// - fail-closed: `BASIC_AUTH_USER`/`BASIC_AUTH_PASSWORD` のいずれかが
///   未設定・空文字なら常に 503 を返す（`WWW-Authenticate` は付けない。
///   認証情報の入力を促さないため）。
/// - 資格情報の比較は SHA-256 ダイジェストの固定長バイト列同士を
///   最後まで走査して XOR 累積する。入力の長さで早期リターンせず、
///   ユーザー名・パスワードの両方を必ず比較してから結果を結合する
///   （タイミング攻撃・長さの漏えいを避ける、OWASP A02/A07）。
/// - `Authorization` ヘッダー・資格情報は一切ログ出力しない。
const MIDDLEWARE_INDEX_JS: &str = r##"/**
 * Routing Middleware（Vercel Build Output API v3、`config.json` の
 * `middlewarePath` から起動される）。Basic 認証で全パスを保護する。
 *
 * fail-closed: `BASIC_AUTH_USER`/`BASIC_AUTH_PASSWORD` のいずれかが
 * 未設定・空文字の場合は誤設定とみなし、常に 503 で拒否する（認証情報の
 * 入力を促す 401 は返さない）。
 */
export default async function middleware(request) {
  const user = process.env.BASIC_AUTH_USER;
  const password = process.env.BASIC_AUTH_PASSWORD;

  if (!user || !password) {
    return new Response('Basic auth is not configured', { status: 503 });
  }

  const unauthorized = () =>
    new Response('Unauthorized', {
      status: 401,
      headers: {
        'WWW-Authenticate': 'Basic realm="Restricted", charset="UTF-8"',
      },
    });

  const header = request.headers.get('authorization') || '';
  const match = header.match(/^Basic\s+(.+)$/);
  if (!match) {
    return unauthorized();
  }

  let decoded;
  try {
    // atob() はバイト列を Latin-1 として文字列化するだけで UTF-8 デコード
    // を行わない。BASIC_AUTH_USER/PASSWORD に日本語等の非 ASCII 文字を
    // 設定した場合に備え、バイト列へ戻してから UTF-8 として明示的に
    // デコードする。
    const binary = atob(match[1]);
    const bytes = Uint8Array.from(binary, (c) => c.charCodeAt(0));
    // fatal: true を指定しないと不正な UTF-8 バイト列が既定で U+FFFD
    // （置換文字）へ静かに置換され、不正なバイト列を含む資格情報が
    // 正規の資格情報と偶然一致してしまう危険がある。不正な UTF-8 は
    // 例外を投げさせ、下の catch で確実に拒否する。
    decoded = new TextDecoder('utf-8', { fatal: true }).decode(bytes);
  } catch {
    // 不正な base64 / UTF-8 は認証情報を読み取れないため拒否する。
    return unauthorized();
  }

  const separatorIndex = decoded.indexOf(':');
  if (separatorIndex === -1) {
    return unauthorized();
  }
  const givenUser = decoded.slice(0, separatorIndex);
  const givenPassword = decoded.slice(separatorIndex + 1);

  // `||` の短絡評価は使わない: 短絡すると givenUser が一致しない場合に
  // givenPassword 側の比較が実行されず、比較回数（延いては処理時間）が
  // 入力によって変わってタイミング攻撃の手がかりになり得る。両方を必ず
  // 比較してから真偽値を結合する。
  const userMatches = await constantTimeEqual(givenUser, user);
  const passwordMatches = await constantTimeEqual(givenPassword, password);
  if (!userMatches || !passwordMatches) {
    return unauthorized();
  }

  // 認証成功: 後続のルーティング（静的ファイル・404 フォールバック）へ
  // 処理を渡す（vercel/examples の Routing Middleware 実装と同じ規約）。
  const response = new Response();
  response.headers.set('x-middleware-next', '1');
  return response;
}

/**
 * 固定長（SHA-256 ダイジェスト）の定数時間比較。
 *
 * 入力文字列同士の長さ比較による早期リターンは行わない: 両文字列を
 * SHA-256 でハッシュ化してから、常に 32 バイト分を最後まで XOR 累積する
 * ことで、入力の長さに比較回数・処理時間が依存しないようにする
 * （長さそのものが漏れる情報になり得るため）。
 */
async function constantTimeEqual(a, b) {
  const digestA = new Uint8Array(
    await crypto.subtle.digest('SHA-256', new TextEncoder().encode(a)),
  );
  const digestB = new Uint8Array(
    await crypto.subtle.digest('SHA-256', new TextEncoder().encode(b)),
  );
  let diff = 0;
  for (let i = 0; i < digestA.length; i += 1) {
    diff |= digestA[i] ^ digestB[i];
  }
  return diff === 0;
}
"##;

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

/// [`BASIC_AUTH_FLAG_ENV`] の値から有効・無効を判定する（純関数）。
///
/// **未設定**（`None`）の場合のみ既定で `false`（無効）を返す。値が
/// **設定済み**の場合は、正確に `"1"` なら `Ok(true)`、それ以外
/// （空文字・`"0"`・`"true"`・前後に空白や改行を含む `"1"` 系〔` 1`・
/// `1\n` 等〕）はすべて `Err` を返す。誤って `"true"`/`"1 "` のような
/// 値を設定した場合に、意図せず [`output_root_assets`] が無保護
/// （ミドルウェアなし）の成果物を生成してビルドを成功させてしまう
/// ことを防ぐ fail-closed な判定（イシュー #3343、P1 指摘対応）。
/// 「未設定＝無効」と「設定済みだが不正な値＝エラー」を区別する。
fn basic_auth_enabled(value: Option<&OsStr>) -> Result<bool, String> {
    match value {
        None => Ok(false),
        Some(v) if v == OsStr::new("1") => Ok(true),
        Some(v) => Err(format!(
            "{BASIC_AUTH_FLAG_ENV} is set to an unexpected value ({v:?}); \
             set it to \"1\" to enable Basic auth middleware, or leave it \
             unset to disable it"
        )),
    }
}

/// `generate_assets` に渡す (リクエストパス, コンテンツ文字列) 列を組み立てる。
///
/// `config.json` と、有効時のみのミドルウェア Function 2 ファイルを
/// `OUTPUT_ROOT`（`.vercel/output`）配下へ 1 回の `generate_assets` 呼び
/// 出しでまとめて書き出す（呼び出し元 [`main`] 参照）。全件を事前検証して
/// から一括で書き込む `generate_assets` の fail-closed 性を、この 3
/// ファイル（有効時）にも及ぼすため、複数回の呼び出しに分割しない。
///
/// 無効時（`basic_auth` が `false`）は [`CONFIG_JSON`] の 1 件のみを返し、
/// 出力内容は本機能導入前とバイト単位で同一になる
/// （`tests::output_root_assets_disabled_returns_only_config_json` 参照）。
fn output_root_assets(basic_auth: bool) -> Vec<(String, String)> {
    if !basic_auth {
        return vec![("/config.json".to_string(), CONFIG_JSON.to_string())];
    }
    vec![
        (
            "/config.json".to_string(),
            CONFIG_JSON_WITH_BASIC_AUTH.to_string(),
        ),
        (
            "/functions/_middleware.func/.vc-config.json".to_string(),
            MIDDLEWARE_VC_CONFIG_JSON.to_string(),
        ),
        (
            "/functions/_middleware.func/index.js".to_string(),
            MIDDLEWARE_INDEX_JS.to_string(),
        ),
    ]
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
/// 対象またはその親要素（`.vercel` 自体を含む）がシンボリックリンクの場合は
/// fail-closed でエラーを返し、削除しない。`.vercel/output` 自体の
/// `symlink_metadata` だけでは、親の `.vercel` が外部ディレクトリへの
/// シンボリックリンクであるケースを検出できず、そのリンクを辿った先の
/// `output` を意図せず削除してしまうため、パスの構成要素を先頭から順に
/// 検証する。
fn clean_output_dir() -> std::io::Result<()> {
    let path = Path::new(OUTPUT_ROOT);

    let mut ancestor = std::path::PathBuf::new();
    for component in path.components() {
        ancestor.push(component);
        match std::fs::symlink_metadata(&ancestor) {
            Ok(meta) if meta.file_type().is_symlink() => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("refusing to remove symlink at {}", ancestor.display()),
                ));
            }
            Ok(_) => {}
            // 途中の構成要素が存在しなければ、削除対象自体も存在しないため
            // 削除するものがない（正常終了）。
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(err) => return Err(err),
        }
    }

    std::fs::remove_dir_all(path)
}

/// [`main`] の失敗理由をまとめる薄いラッパー（`Display` のみを利用者へ見せ、
/// I/O エラーの内部詳細以外の機微情報は含めない、`security.md` 準拠）。
#[derive(Debug)]
enum BuildError {
    Clean(std::io::Error),
    Ssg(SsgError),
    /// [`basic_auth_enabled`] が不正な環境変数値を検出した場合
    /// （イシュー #3343）。無保護の成果物を誤って生成しないよう、
    /// ビルド自体を失敗させる。
    BasicAuthFlag(String),
}

impl std::fmt::Display for BuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BuildError::Clean(err) => write!(f, "failed to clean {OUTPUT_ROOT}: {err}"),
            BuildError::Ssg(err) => write!(f, "{err}"),
            BuildError::BasicAuthFlag(msg) => write!(f, "{msg}"),
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
///    （`OUTPUT_ROOT`。[`BASIC_AUTH_FLAG_ENV`] が `"1"` のときはミドル
///    ウェア Function 2 ファイルも同時に）を書き出す。
///
/// 成功時は書き出したファイルパスを 1 行ずつ標準出力へ、失敗時はエラーを
/// 標準エラーへ出力して非ゼロ終了する（`unwrap`/`panic!` は使わない）。
fn main() {
    let result = basic_auth_enabled(std::env::var_os(BASIC_AUTH_FLAG_ENV).as_deref())
        .map_err(BuildError::BasicAuthFlag)
        .and_then(|basic_auth| {
            clean_output_dir()
                .map_err(BuildError::Clean)
                .and_then(|()| Ok(generate_pages(&build_pages(), Path::new(STATIC_DIR))?))
                .and_then(|pages| build_assets(basic_auth, pages))
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

/// `main` から呼ばれる、[`STATIC_DIR`] へのページ生成後の残り処理
/// （404 ページ・`config.json`・有効時のミドルウェアの書き出し）を
/// まとめた関数。`basic_auth` の判定（[`basic_auth_enabled`]）が
/// エラーの場合はそもそも呼ばれない。
fn build_assets(
    basic_auth: bool,
    pages: Vec<std::path::PathBuf>,
) -> Result<Vec<std::path::PathBuf>, BuildError> {
    let not_found = generate_assets(&not_found_asset(), Path::new(STATIC_DIR))?;
    let config = generate_assets(&output_root_assets(basic_auth), Path::new(OUTPUT_ROOT))?;
    Ok(pages
        .into_iter()
        .chain(not_found)
        .chain(config)
        .collect::<Vec<_>>())
}

/// [`basic_auth_enabled`]・[`output_root_assets`]・
/// `CONFIG_JSON_WITH_BASIC_AUTH` の単体テスト（イシュー #3343）。
/// CLI ブラックボックステスト（両モードの `.vercel/output/` 生成結果）は
/// `tests/build_output.rs` を参照。
#[cfg(test)]
mod tests {
    use super::*;

    /// フラグの真理値表: 未設定は無効（`Ok(false)`）、`"1"` は有効
    /// （`Ok(true)`）、それ以外の設定済みの値はすべてエラー（fail-closed。
    /// 「未設定＝無効」と「設定済みだが不正な値＝エラー」を区別する、
    /// イシュー #3343 P1 指摘対応）。
    #[test]
    fn basic_auth_enabled_truth_table() {
        assert_eq!(basic_auth_enabled(None), Ok(false));
        assert!(basic_auth_enabled(Some(OsStr::new(""))).is_err());
        assert!(basic_auth_enabled(Some(OsStr::new("0"))).is_err());
        assert!(basic_auth_enabled(Some(OsStr::new("true"))).is_err());
        assert!(basic_auth_enabled(Some(OsStr::new(" 1"))).is_err());
        assert!(basic_auth_enabled(Some(OsStr::new("1 "))).is_err());
        assert!(basic_auth_enabled(Some(OsStr::new("1\n"))).is_err());
        assert_eq!(basic_auth_enabled(Some(OsStr::new("1"))), Ok(true));
    }

    /// `CONFIG_JSON_WITH_BASIC_AUTH` から先頭の middlewarePath ルート
    /// ブロックを取り除くと `CONFIG_JSON` と完全一致することを固定する
    /// （2 定数が意図せず乖離しないための回帰）。
    #[test]
    fn config_json_with_basic_auth_matches_base_config_json_with_middleware_route_removed() {
        let reduced = CONFIG_JSON_WITH_BASIC_AUTH.replacen(MIDDLEWARE_ROUTE_BLOCK, "", 1);
        assert_eq!(reduced, CONFIG_JSON);
    }

    /// 無効時は `config.json` 1 件のみが返り、内容は既存の `CONFIG_JSON`
    /// と同一であることを固定する（バイト同一性の前提）。
    #[test]
    fn output_root_assets_disabled_returns_only_config_json() {
        let assets = output_root_assets(false);
        assert_eq!(
            assets,
            vec![("/config.json".to_string(), CONFIG_JSON.to_string())]
        );
    }

    /// 有効時は `config.json`（middlewarePath 入り）+ ミドルウェア 2
    /// ファイルの計 3 件が返ることを固定する。
    #[test]
    fn output_root_assets_enabled_returns_middleware_files() {
        let assets = output_root_assets(true);
        let paths: Vec<&str> = assets.iter().map(|(path, _)| path.as_str()).collect();
        assert_eq!(
            paths,
            vec![
                "/config.json",
                "/functions/_middleware.func/.vc-config.json",
                "/functions/_middleware.func/index.js",
            ]
        );
        assert_eq!(assets[0].1, CONFIG_JSON_WITH_BASIC_AUTH);
        assert_eq!(assets[1].1, MIDDLEWARE_VC_CONFIG_JSON);
        assert_eq!(assets[2].1, MIDDLEWARE_INDEX_JS);
    }
}
