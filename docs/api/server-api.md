# fandhe-frontend-server SSG API

`fandhe-frontend-server` の `ssg` モジュール（`fandhe_frontend_server::ssg`）が
提供する静的サイト生成（SSG）API の公開契約をまとめます。SSR 応答の
ボディをそのまま静的ファイルへ書き出す設計（SSR/SSG 出力の一致）に
基づき、独自の HTML 組み立て・独自のエスケープ処理は行いません。
本ページは `fandhe-frontend-server` 0.2.0 以降を前提とします。

## 1. 公開 API 一覧

| API | 役割 | 出力先 |
|---|---|---|
| `generate(out_dir: &Path) -> Result<Vec<PathBuf>, SsgError>` | 既定 loader（`DemoItemsLoader` / `DemoItemDetailLoader`）で固定ルート表（`/`・`/items/{id}`）を書き出す | `<out_dir>/index.html`・`<out_dir>/items/<id>/index.html` |
| `generate_with(list_loader, detail_loader, out_dir: &Path) -> Result<Vec<PathBuf>, SsgError>` | loader を差し替え可能なジェネリック版。`crate::ssr::respond_with` の 200 応答ボディをそのまま書き出す | 同上 |
| `generate_pages(pages: &[(String, Node)], out_dir: &Path) -> Result<Vec<PathBuf>, SsgError>` | 任意の (リクエストパス, `Node`) 列を `fandhe_frontend_core::render` 経由で HTML 化して書き出す汎用 SSG API | `<path>/index.html` 固定 |
| `generate_assets(assets: &[(String, String)], out_dir: &Path) -> Result<Vec<PathBuf>, SsgError>` | 任意の (リクエストパス, コンテンツ文字列) 列を無加工で書き出す汎用アセット API | 任意のファイル名（拡張子の有無を問わない） |
| `SsgError` | 上記 4 API 共通のエラー型 | — |

いずれも `std::fs` のみで完結し、外部クレート（`tempfile` 等）に依存し
ません。戻り値は書き出したファイルの絶対パス一覧です。

## 2. 使い分け

- **HTML ページ**: `generate_pages` を使います。`fandhe_frontend_core::Node`
  木を渡すことで `fandhe_frontend_core::render` を経由し、既定エスケープ
  （テキスト補間・属性値のエスケープ）が構造的に適用されます。
- **非 HTML アセット**（`sitemap.xml` / `robots.txt` / `healthz` 等）:
  `generate_assets` を使います。コンテンツ文字列を無加工で書き出す
  ため、`generate_pages` より柔軟な任意のファイル名を扱えますが、既定
  エスケープは適用されません。

**`generate_assets` を HTML の組み立てに使わないでください。** `404.html`
のような HTML アセットを書き出す場合は、呼び出し側が
`format!("<!DOCTYPE html>\n{}", fandhe_frontend_core::render(&node))` の
ようにノード木 API 経由で文字列化してから `generate_assets` へ渡します
（HTML 文字列の直接組み立て禁止の原則には抵触しません。`generate_assets`
自身は HTML を組み立てず、呼び出し側が組み立て済みの文字列を受け取る
だけです）。

`sitemap.xml` に埋め込む URL 等、コンテンツ内部のエスケープ（XML
エスケープ等）は呼び出し側の責務です。`generate_assets` はエスケープを
一切行いません。

利用例は [ssg-blog サンプル](../../examples/ssg-blog/README.md)の
`src/main.rs`（`build_pages` / `build_assets` / `main`）を参照してくだ
さい。

## 3. パス検証規則（fail-closed 契約）

`generate_pages` と `generate_assets` はいずれも、書き出し前に**全件**の
パスを検証し、正規化後の出力先の重複も検出します。1 件でも検証・重複
判定に失敗した場合は**どのファイルも書き出さずに**エラーを返します
（部分成功で `dist/` を汚しません）。

| 項目 | `generate_pages` | `generate_assets` |
|---|---|---|
| 出力先 | `<path>/index.html` 固定 | 任意のファイル名 |
| 先頭 `/` | 必須 | 必須 |
| 末尾 `/` | 任意（`/guide/foo` と `/guide/foo/` は同一出力へ正規化） | 不可（アセットは常にファイルを指すため） |
| 空セグメント（`//`） | 拒否 | 拒否 |
| 全セグメント許可文字 | 英数字・`-`・`_` | 最終セグメント（ファイル名）・中間セグメント（ディレクトリ名）ともに英数字・`-`・`_`・`.` |
| ドットのみの名前（`.`/`..`/`...`） | 非許可文字集合のため構造的に拒否 | 位置を問わず構造的に拒否 |
| `..`/`.` トラバーサル | 拒否 | 拒否 |

`generate_pages` の全セグメント検証・`generate_assets` の最終セグメント
検証はいずれも `generate`/`generate_with` の `Item::id` 検証と同じ
ホワイトリスト方式を土台にしており、二重管理を避けています。

## 4. `.well-known` の許可範囲

`generate_assets` の中間ディレクトリセグメントは、ファイル名と同じ
「英数字・`-`・`_`・`.`」の許可文字集合に加えて**ドット始まりの名前**
（`.well-known` 等）を許可します。これにより `/.well-known/security.txt`
のような RFC 8615 well-known URI 配下へのアセット出力が可能です。

一方で次の 2 点は安全側に倒し、常に拒否します。

- **ドットのみの名前**（`.`/`..`/`...`）: 位置を問わず構造的に拒否し、
  トラバーサル不可の不変条件を維持します。
- **`.git`**（ASCII 大文字小文字非区別。末尾の `.` を trim してから比較
  するため `.git.` のような表記も含めて拒否します）: `out_dir` が git
  worktree（gh-pages デプロイ等）である場合に `.git/config` や
  `.git/hooks/...` への書き出しを防ぐ defense-in-depth です。

## 5. エラー（`SsgError`）

| variant | 発生条件 |
|---|---|
| `UnsafeItemId(String)` | `Item::id` に `..`・`/`・`\` 等の非許可文字を含む（`generate`/`generate_with` の固定ルート表限定） |
| `CreateDir { path, source }` | 出力先ディレクトリの作成（`fs::create_dir_all`）に失敗した |
| `WriteFile { path, source }` | ファイル書き込み（`fs::write`）に失敗した |
| `RouteNotFound(String)` | 固定ルート表とハンドラの不整合（通常到達しない） |
| `UnexpectedStatus { path, status }` | SSR ルートが 200 以外の非 500 ステータスを返した |
| `LoaderError { path }` | loader がデータ解決に失敗した（一覧列挙・各ルート描画のいずれか） |
| `UnsafePagePath(String)` | `generate_pages`/`generate_assets` に渡したパスが検証を通らなかった |
| `DuplicatePagePath(String)` | `generate_pages`/`generate_assets` の複数パスが正規化後に同じ出力先を指した |

`SsgError` の `Display` はいずれの variant も呼び出し元が渡したパス
文字列・ステータスコードのみを含み、`Loader::Error` の内部詳細（内部
パス・接続情報等）は一切含めません。

## 6. セキュリティ不変条件

- **パストラバーサル対策（OWASP A01）**: 出力ファイルパスは全セグメント
  をホワイトリスト検証したうえで `out_dir` 配下に限定して構成します。
  `..`/`.`/空セグメントを含むパスは構造的に拒否され、`out_dir.join(..)`
  した結果が `out_dir` 外を指す経路は存在しません。
- **fail-closed**: `generate_pages`/`generate_assets` は書き出し前の全件
  事前検証・重複判定を行い、1 件でも不正・重複があればファイルを 1 つも
  書き出さずにエラーを返します。loader 解決の失敗（`generate`/
  `generate_with`）も即座にビルドを失敗させ、部分成功で握りつぶしません。
- **機微情報の非露出**: `SsgError::Display` は `Loader::Error` の値を
  含みません。
- **panic しない**: `unwrap`/`expect`/`panic!` を使わず、書き込み・検証の
  失敗はすべて `Result`（`SsgError`）として呼び出し元へ伝えます。
- **呼び出し間の出力衝突は検出対象外という caveat**: `generate_pages` /
  `generate`/`generate_with` と `generate_assets` を同一 `out_dir` へ併用
  した場合、重複検出は 1 回の呼び出し内でしか効きません（例:
  `generate_assets` の `/index.html` と `generate_pages` の `/` が衝突
  しても検出されません）。同一 `out_dir` へ複数 API を併用する場合は、
  呼び出し側で出力パスが重ならないことを確認してください。

## 7. 関連ドキュメント

- [fandhe-frontend-app API](./app-api.md) — `Loader` トレイト・`Item` 型
- [ルーター パスマッチング](./router-path-matching.md) — `respond_with`
  が内部で使う `Router` の仕様
- [JS ゼロ SSG での利用ガイド](../guides/no-js-ssg.md) — `generate_pages`
  で書き出した静的サイトのクライアント側挙動
- [ssg-blog サンプル](../../examples/ssg-blog/README.md) — `generate_pages`
  / `generate_assets` の実装例
