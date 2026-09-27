# デプロイガイド

fandhe-frontend の配布形態は主に 2 通りです。

- **静的出力（SSG）**: `fandhe_frontend_server::ssg::generate_pages` でページを
  静的 HTML として書き出し、任意の静的ホスティングへ配置します
  （正本サンプル: `examples/ssg-blog`）。
- **単一実行ファイル**: `fandhe-frontend-dist-server` で SSR/動的処理込みの
  単一バイナリをビルドし、Docker で配布します（REQ-9。正本サンプル:
  `examples/dist-server-docker`）。

本ガイドでは、このうち Vercel へのデプロイ方法を解説します。

## Vercel

### 方式の選び方

fandhe-frontend を Vercel にデプロイする際の推奨方式は 1 つだけです。

- **推奨: SSG → Vercel Build Output API → `vercel deploy --prebuilt`**
  （正本サンプル: `examples/vercel-ssg`）。`generate_pages`/`generate_assets`
  でローカルまたは CI であらかじめ静的出力を生成し、ビルド済み成果物を
  そのままデプロイします。Vercel 側に Rust ツールチェーンは不要です。

**SSR・動的処理は、Vercel の Rust ランタイム経由では現時点で対応していません。**
理由は次のとおりです。

- `vercel_runtime` 1.x（`vercel-community/rust` 系統）は依存先
  `lambda_runtime` が Vercel の Rust Function 実行環境に存在しない
  `AWS_LAMBDA_*` 系環境変数を必須として `expect` するため、起動直後に
  panic して全リクエストが HTTP 500 になります。
- `vercel_runtime` 2.x（`lambda_runtime` 非依存の独立実装）は Vercel 本番で
  正常応答を実測済みですが、依存木がフレームワーク側で定めた基準
  （60 件/深さ 6）を構造的に超過し、`build.rs` を持つ依存も多数追加されます。
- Vercel の Rust Function 実行自体もまだ Beta です。

判断の根拠・実測データの詳細は `docs/design/vercel-deployment-strategy.md`
（docs サイト非掲載のため、本ガイドではリンクにせずプレーンテキストで
参照します）を参照してください。

SSR・動的処理が必要な場合は、Vercel 以外のコンテナ実行基盤で
`examples/dist-server-docker` の単一バイナリを使ってください。

判断の目安は次のとおりです。

| ページ内容 | 推奨 |
|-----------|------|
| ビルド時に内容が確定する（ブログ・ドキュメント・マーケティングページ等） | `examples/vercel-ssg`（本ガイド） |
| リクエストごとの描画・フォーム受付・DB アクセス等が必要 | Vercel 以外のコンテナ基盤 + `examples/dist-server-docker` |

再評価条件（Vercel Rust ランタイムの GA 化、Vercel Container Images（Beta）の
GA 化、`fandhe-frontend-dist-server` の `$PORT`/`SIGTERM` 対応等）は
`docs/design/vercel-deployment-strategy.md` §7 を参照してください。

### デプロイ手順の要約

詳細な手順・生成物の説明は [`examples/vercel-ssg` の README](../../examples/vercel-ssg/README.md)
を参照してください。要約すると次の 3 ステップです。

```bash
# 0. サンプルディレクトリへ移動する（cargo run / vercel deploy は
#    このディレクトリ配下の Cargo.toml・.vercel/output を前提とする）
cd examples/vercel-ssg

# 1. プロジェクトを Vercel と紐付ける
vercel link

# 2. 静的出力を生成する（Vercel 側では実行されない）
cargo run --release

# 3. ビルド済み出力をそのままデプロイする
vercel deploy --prebuilt
# 本番デプロイの場合は --prod を付ける
```

### Deployment Protection（既定で有効）

新規に作成した Vercel プロジェクトでは、既定で Vercel Authentication
（Standard Protection、SSO によるデプロイ保護）が有効になっています。
チームのデフォルト設定で変更されている場合があるため、実際の有効・無効は
必ず後述の確認手順で確かめてください。

有効な間は、未認証のアクセスが Vercel のログイン画面へ誘導され、
アプリ側のルーティング（`config.json` の 404 フォールバックや、後述の
Routing Middleware を含む）へは到達しません。公式ドキュメントも
「Deployment Protection requires authentication for all requests,
including those to Routing Middleware.」（Deployment Protection はミドル
ウェアを含むすべてのリクエストに認証を要求する）と明記しています。

**Deployment Protection は、後述の Basic 認証よりも強い統制です。**
Basic 認証は Deployment Protection を無効化した（あるいは対象外の）
デプロイに対する軽量な門、または多層防御の 2 層目として位置づけてください。

- **無効化前に必ず確認すること**: `examples/vercel-ssg` 本体は後述の
  Routing Middleware（Basic 認証）を組み込んでいません。Deployment
  Protection を無効化すると、ミドルウェア未導入のデプロイ（既存デプロイを
  含む）は**認証なしで誰でも閲覧できる状態**になります。「無効化しても
  既存デプロイが全拒否される」わけではなく、逆に**無保護で公開される**
  点を混同しないでください。Basic 認証で保護したい場合は、無効化する前に
  次の順序で進めてください。
  1. 後述の「Routing Middleware による Basic 認証」の手順でミドルウェア
     組み込み済みの新しいデプロイを作成し、`BASIC_AUTH_USER` /
     `BASIC_AUTH_PASSWORD` を設定した状態で `vercel deploy --prebuilt`
     が完了していることを確認する。
  2. 保護したい旧デプロイ（ミドルウェア未導入のまま残っているもの）を
     ダッシュボードで洗い出し、公開のままでよいか判断する。公開すべきで
     なければ `vercel remove <deployment-url>` で**削除する**。Vercel の
     デプロイ URL はデプロイごとに不変であり、ミドルウェア入りの内容で
     再デプロイしても旧デプロイの URL は別に存在し続け無保護のまま残る
     ため、「再デプロイして置き換える」は保護の代替にならない。旧 URL
     を無効化する手段は削除のみと理解してください。
  3. 上記が済んでから Project Settings → Deployment Protection →
     Vercel Authentication のトグルを無効にして保存する。
- **確認手順**（`<deployment-url>` は実際のデプロイ URL に読み替え）:

  ```bash
  # Deployment Protection が有効なら 200 以外
  # （Vercel のログインへ誘導する応答）になる
  curl -sI "https://<deployment-url>/"

  # 存在しないパスで 404（config.json の 404 フォールバック）を確認する
  curl -sI "https://<deployment-url>/does-not-exist"
  ```

  Deployment Protection を無効化した後の期待値は、Basic 認証ミドルウェアの
  有無で変わります。

  ```bash
  # Basic 認証ミドルウェア未導入のデプロイ:
  # 無効化後は認証なしで 200 になる（= 無保護で公開されている）
  curl -sI "https://<deployment-url>/"

  # Basic 認証ミドルウェア導入後のデプロイ:
  # 認証情報なしだと 401（Unauthorized）になる
  curl -sI "https://<deployment-url>/"

  # 正しい認証情報を付けると 200 になる（パスワードをコマンドライン引数に
  # 直接書かないため、`-u "<user>"` のみ指定して curl の対話プロンプトで
  # 入力する）
  curl -sI -u "<user>" "https://<deployment-url>/"

  # 503 の場合は BASIC_AUTH_USER / BASIC_AUTH_PASSWORD が未設定
  # （fail-closed。後述の「環境変数の反映範囲と旧デプロイ」参照）
  ```

- Protection Bypass for Automation 等のバイパス用トークンは、リポジトリ・
  README・CI ログのいずれにも書かないでください。

### Routing Middleware による Basic 認証

Deployment Protection を無効化した公開デプロイに、追加の軽量な認証を
かけたい場合は、Build Output API の Routing Middleware で Basic 認証を
実装できます。**`examples/vercel-ssg` 本体はこの認証を組み込んでいません**
（別イシューでの検討対象です）。ここでは既存の `generate_assets` 呼び出しに
追記する形で拡張する手順を示します。

#### 仕組み

`config.json` の `routes` 配列の**先頭**（`{"handle": "filesystem"}` より前）
に `middlewarePath` ルートを置くと、静的ファイル・404 ページを含む全パスが
このミドルウェアを通過します。

```jsonc
{
  "version": 3,
  "routes": [
    {
      "src": "/(.*)",
      "middlewarePath": "_middleware",
      "continue": true
    },
    {
      "src": "/(.*)",
      "headers": { "...": "..." },
      "continue": true
    },
    { "handle": "filesystem" },
    { "src": "/(.*)", "status": 404, "dest": "/404.html" }
  ]
}
```

ミドルウェア自体は Edge Runtime 関数として配置します（Vercel は Function
自体の Edge Runtime を Node.js への移行を一般に推奨していますが、Routing
Middleware は Build Output API 上「Edge Runtime 関数」として仕様化されて
いるため、ここでは `runtime: "edge"` が正しい選択です）。

#### 生成物の追加

`examples/vercel-ssg` の `clean_output_dir()` は毎回 `.vercel/output` を
削除するため、手書きファイルを直接置いても消えてしまいます。追加する
ファイルは `generate_assets` で `OUTPUT_ROOT`（`.vercel/output`）配下へ
書き出してください。追加するのは次の 3 点です。

- `/functions/_middleware.func/.vc-config.json`
- `/functions/_middleware.func/index.js`
- `middlewarePath` ルートを先頭に加えた `config.json`

`.vc-config.json` の例です。

```json
{
  "runtime": "edge",
  "entrypoint": "index.js",
  "envVarsInUse": ["BASIC_AUTH_USER", "BASIC_AUTH_PASSWORD"]
}
```

`index.js` の要件は次のとおりです（Edge Runtime は `process.env` で環境
変数へアクセスできます）。

- `BASIC_AUTH_USER` / `BASIC_AUTH_PASSWORD`（名前は例）を `process.env` から
  読み、**どちらかが未設定・空文字なら常に拒否**します（fail-closed）。
  誤設定時は 503 を返し、`WWW-Authenticate` は付けません（認証情報の入力を
  促さないため）。
- 成功時は `x-middleware-next` ヘッダーを持つ `Response` を返し、後続の
  ルーティング（静的ファイル・404 フォールバック）へ処理を渡します。
- 失敗時は 401 と `WWW-Authenticate: Basic realm="Restricted",
  charset="UTF-8"` を返します。
- `Authorization` ヘッダー・認証情報を `console.log` 等でログ出力しません。

```js
/**
 * Routing Middleware（Vercel Build Output API v3、`config.json` の
 * `middlewarePath` から起動される）。Basic 認証で全パスを保護する。
 *
 * fail-closed: `BASIC_AUTH_USER`/`BASIC_AUTH_PASSWORD` のいずれかが
 * 未設定・空文字の場合は誤設定とみなし、常に 503 で拒否する（認証情報の
 * 入力を促す 401 は返さない）。
 */
export default function middleware(request) {
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
  const userMatches = constantTimeEqual(givenUser, user);
  const passwordMatches = constantTimeEqual(givenPassword, password);
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
 * ベストエフォートの定数時間比較。タイミング攻撃を完全には排除しないが、
 * 単純な `===` 比較より漏洩する情報を減らす。
 */
function constantTimeEqual(a, b) {
  if (a.length !== b.length) {
    // 長さの不一致自体も情報になり得るが、ここでは簡潔さを優先する
    // （ベストエフォート）。
    return false;
  }
  let diff = 0;
  for (let i = 0; i < a.length; i += 1) {
    diff |= a.charCodeAt(i) ^ b.charCodeAt(i);
  }
  return diff === 0;
}
```

#### 環境変数の登録

対話プロンプトで値を入力します（シェル履歴に値を残さないため、
コマンドライン引数や `echo | vercel env add` は避けてください）。

```bash
vercel env add BASIC_AUTH_USER production
vercel env add BASIC_AUTH_PASSWORD production

# preview 環境でも保護したい場合は同様に追加する
vercel env add BASIC_AUTH_USER preview
vercel env add BASIC_AUTH_PASSWORD preview
```

#### 注意点

- Vercel は HTTPS を強制するため、Basic 認証（平文相当のエンコードのみ）
  でも通信経路上は保護されます。
- Basic 認証は軽量な制限です。機微データの保護には前述の Deployment
  Protection 等を使ってください。
- 認証情報はリポジトリ・`vercel.json`・README のいずれにも書かないで
  ください。

### 環境変数の反映範囲と旧デプロイ

環境変数の追加・変更は、**その変更より後に作成したデプロイにだけ**
反映されます。稼働中のデプロイは、変更前の値を保持し続けます（Vercel の
挙動: [Managing environment variables](https://vercel.com/docs/environment-variables/managing-environment-variables)）。
変更を反映するには、`vercel deploy --prebuilt`（本番は `--prod`）で
再デプロイしてください。

デプロイごとの URL（`*.vercel.app` の生成 URL）は不変で、作成時点の値を
保持します。認証情報をローテーションしても、旧デプロイは旧い認証情報を
受け付け続けます。

- **対処**: `vercel remove <deployment-url>` またはダッシュボードから
  旧デプロイを削除してください。
- **fail-closed の帰結**: これは**ミドルウェアが組み込まれているデプロイ**
  に限った話です。環境変数を設定する前にそのデプロイを作成した場合、
  ミドルウェアが常に 503 を返す恒久的な全拒否状態になります（前述の
  「無効化前に必ず確認すること」で扱った、ミドルウェア自体を含まない
  デプロイが無保護で公開される状態とは別の問題です）。環境変数を設定して
  から再デプロイしてください。

### 関連リンク

- [`examples/vercel-ssg` の README](../../examples/vercel-ssg/README.md)
- [`examples/ssg-blog` の README](../../examples/ssg-blog/README.md)
- [`examples/dist-server-docker` の README](../../examples/dist-server-docker/README.md)
- [サンプル集](examples.md)
- [Server API リファレンス](../api/server-api.md)

外部の公式ドキュメント（出典）:

- [Build Output API](https://vercel.com/docs/build-output-api)
- [Build Output Configuration（routes / middlewarePath）](https://vercel.com/docs/build-output-api/configuration)
- [Vercel Primitives（Function with Edge Runtime / `.vc-config.json`）](https://vercel.com/docs/build-output-api/primitives)
- [Features（Routing Middleware）](https://vercel.com/docs/build-output-api/features)
- [Edge Runtime](https://vercel.com/docs/functions/runtimes/edge)
- [Deployment Protection](https://vercel.com/docs/deployment-protection)
- [Vercel Authentication](https://vercel.com/docs/deployment-protection/methods-to-protect-deployments/vercel-authentication)
- [vercel env](https://vercel.com/docs/cli/env)
- [vercel remove](https://vercel.com/docs/cli/remove)
- [Managing environment variables](https://vercel.com/docs/environment-variables/managing-environment-variables)
