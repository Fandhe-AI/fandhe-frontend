# デプロイガイド

fandhe-frontend の配布形態は主に 2 通りです。

- **静的出力（SSG）**: `fandhe_frontend_server::ssg::generate_pages` でページを
  静的 HTML として書き出し、任意の静的ホスティングへ配置します
  （正本サンプル: `examples/ssg-blog`）。
- **単一実行ファイル**: `fandhe-frontend-dist-server` で SSR/動的処理込みの
  単一バイナリをビルドし、Docker で配布します（REQ-9。正本サンプル:
  `examples/dist-server-docker`）。

本ガイドでは、このうち Vercel へのデプロイ方法を解説します。Vercel には
静的配置（SSG）と SSR（Vercel Container Images 経由）の 2 通りの方式が
あります。

## Vercel

### 方式の選び方

fandhe-frontend を Vercel にデプロイする方式は、静的配置と SSR の
2 通りです。

- **静的配置（既定・推奨）: SSG → Vercel Build Output API →
  `vercel deploy --prebuilt`**（案 c、正本サンプル: `examples/vercel-ssg`）。
  `generate_pages`/`generate_assets` でローカルまたは CI であらかじめ
  静的出力を生成し、ビルド済み成果物をそのままデプロイします。Vercel 側に
  Rust ツールチェーンは不要で、Beta 機能にも依存しません。
- **SSR（Beta 依存）: Vercel Container Images**（案 d、正本サンプル:
  `examples/vercel-ssr`）。`Dockerfile.vercel` のコンテナイメージを Vercel
  上で実行し、`fandhe-frontend-dist-server`（0.3.4 以降。`PORT` 対応・
  graceful shutdown 対応を含む）でリクエストごとの描画を行います。

選び方の原則は次のとおりです。静的出力で足りるなら案 c を推奨します。
リクエストごとの描画が必要な場合にだけ案 d を選び、Beta 依存の影響を
SSR 用途に限定してください。

案 d を選ぶ場合は次の点に注意してください。

- Container Images は公式に Beta であり、チームでの有効化（権限）が
  必要な場合があります
- Vercel プロジェクトの環境変数 `PORT` に **1024 以上**（例 `3100`）を
  設定する必要があります（イメージは非 root の `USER 65532:65532` で
  動くため、1024 未満のポートに bind できません。既定の `80` は使えません）
- `Dockerfile.vercel` は `FANDHE_FRONTEND_BIND_ADDR` を設定しません
  （設定すると `PORT` より優先されてしまうためです）
- SIGTERM を受けると最大 25 秒かけて処理中の接続を終えます（Vercel の
  猶予 30 秒以内です）
- 静的アセットと WASM は出荷しません
- Vercel 実機での検証はまだ行っていません（イシュー #3339）

### Vercel の Rust ランタイム（`vercel_runtime`）を使わない理由

SSR を Vercel で行う場合は、上記の案 d（Vercel Container Images）を
使います。Vercel の公式 Rust ランタイムである `vercel_runtime` は、
次の理由から採用していません。

- `vercel_runtime` 1.x（`vercel-community/rust` 系統）は依存先
  `lambda_runtime` が Vercel の Rust Function 実行環境に存在しない
  `AWS_LAMBDA_*` 系環境変数を必須として `expect` するため、起動直後に
  panic して全リクエストが HTTP 500 になります。
- `vercel_runtime` 2.x（`lambda_runtime` 非依存の独立実装）は Vercel 本番で
  正常応答を実測済みですが、依存木がフレームワーク側で定めた基準
  （60 件/深さ 6）を構造的に超過し、`build.rs` を持つ依存も多数追加されます。
- Vercel の Rust Function 実行自体もまだ Beta です。

判断の根拠・実測データの詳細は
[`docs/design/vercel-deployment-strategy.md`](https://github.com/Fandhe-AI/fandhe-frontend/blob/main/docs/design/vercel-deployment-strategy.md)
を参照してください。

判断の目安は次のとおりです。

| ページ内容 | 推奨 |
|-----------|------|
| ビルド時に内容が確定する（ブログ・ドキュメント・マーケティングページ等） | `examples/vercel-ssg`（案 c、推奨） |
| リクエストごとの描画が必要で、Vercel 上で動かす | `examples/vercel-ssr`（案 d、Container Images・Beta） |
| リクエストごとの描画が必要で、Vercel 以外のコンテナ基盤で動かす | `examples/dist-server-docker` |

再評価条件（Vercel Rust ランタイムの GA 化、Vercel Container Images（Beta）の
GA 化等）は
[`docs/design/vercel-deployment-strategy.md`](https://github.com/Fandhe-AI/fandhe-frontend/blob/main/docs/design/vercel-deployment-strategy.md)
§7 を参照してください。

### デプロイ手順の要約

#### 静的配置（vercel-ssg）

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

#### SSR（vercel-ssr）

詳細な手順は [`examples/vercel-ssr` の README](../../examples/vercel-ssr/README.md)
を参照してください（Vercel プロジェクト設定・`vercel deploy` の内容は
README の「Vercel へのデプロイ」節が正です）。要約すると次のとおりです。

```bash
# 0. サンプルディレクトリへ移動する
cd examples/vercel-ssr

# 1. Vercel プロジェクト設定で PORT を 1024 以上（例 3100）に設定する
#    （Project Settings → Environment Variables）

# 2. プロジェクトを Vercel と紐付ける
vercel link

# 3. デプロイする（build step が Dockerfile.vercel を自動検出してビルドする）
vercel deploy
# 本番デプロイの場合は --prod を付ける
vercel deploy --prod
```

`--prebuilt` は付けません（`.vercel/output/` の事前ビルド成果物を使う
Build Output API 経路〔案 c〕向けのオプションで、`Dockerfile.vercel` の
build step とは無関係です）。

### Deployment Protection（既定で有効）

Deployment Protection は静的配置（案 c・vercel-ssg）・SSR（案 d・
vercel-ssr）のどちらのデプロイにも適用されます。以下の説明は主に
vercel-ssg を前提としていますが、Vercel Authentication 自体の挙動
（未認証アクセスの扱い）は両方式で共通です。

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

- **無効化前に必ず確認すること**（vercel-ssg の場合）: `examples/vercel-ssg`
  本体は後述の Routing Middleware（Basic 認証）を opt-in（既定では無効。
  ビルド時に `FANDHE_VERCEL_SSG_BASIC_AUTH=1` を指定したときだけ組み込ま
  れます）で組み込み済みです。フラグなしでビルドしたデプロイにはミドル
  ウェアが含まれないため、Deployment Protection を無効化すると、そのまま
  では**認証なしで誰でも閲覧できる状態**になります。「無効化しても
  既存デプロイが全拒否される」わけではなく、逆に**無保護で公開される**
  点を混同しないでください。Basic 認証で保護したい場合は、無効化する前に
  次の順序で進めてください。
  1. 後述の「Routing Middleware による Basic 認証」の手順で
     `FANDHE_VERCEL_SSG_BASIC_AUTH=1` を指定してビルドした新しいデプロイ
     を作成し、`BASIC_AUTH_USER` / `BASIC_AUTH_PASSWORD` を設定した状態で
     `vercel deploy --prebuilt` が完了していることを確認する。
  2. **Deployment Protection を無効化する前に、Basic 認証が実際に効いて
     いることを確認する**（後述の「Deployment Protection が有効なままの
     Basic 認証確認（必須）」）。未認証で 401、正しい資格情報で 200 に
     ならない場合は、次の手順へ進まずミドルウェアの組み込み・
     `config.json` の `middlewarePath` 設定・環境変数の設定を見直して
     再デプロイし、確認をやり直す。
  3. 保護したい旧デプロイ（ミドルウェア未導入のまま残っているもの）を
     ダッシュボードで洗い出し、公開のままでよいか判断する。公開すべきで
     なければ `vercel remove <deployment-url>` で**削除する**。Vercel の
     デプロイ URL はデプロイごとに不変であり、ミドルウェア入りの内容で
     再デプロイしても旧デプロイの URL は別に存在し続け無保護のまま残る
     ため、「再デプロイして置き換える」は保護の代替にならない。旧 URL
     を無効化する手段は削除のみと理解してください。
  4. 上記が済んでから Project Settings → Deployment Protection →
     Vercel Authentication のトグルを無効にして保存する。
- **確認手順**（`<deployment-url>` は実際のデプロイ URL に読み替え、
  404 の確認は vercel-ssg（`config.json` の 404 フォールバック）の場合）:

  ```bash
  # Deployment Protection が有効なら 200 以外
  # （Vercel のログインへ誘導する応答）になる
  curl -sI "https://<deployment-url>/"

  # 存在しないパスで 404（config.json の 404 フォールバック、vercel-ssg の場合）を確認する
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

#### Deployment Protection が有効なままの Basic 認証確認（必須）

前述の「無効化前に必ず確認すること」手順 2 に対応する検証です。
Deployment Protection（Vercel Authentication）が有効な間は、未認証の
リクエストは Vercel のログインへ誘導する応答（302 リダイレクトまたは
401）になり、Basic 認証ミドルウェアまで到達しません。この状態のまま
素の `curl` で 401/200 を確認しても、返ってくるのは Vercel 側の応答で
あってミドルウェアの応答ではないため、確認したことになりません。

Deployment Protection を無効化する前にミドルウェアの認証結果だけを
検証するには、Protection Bypass for Automation
（[公式ドキュメント](https://vercel.com/docs/deployment-protection/methods-to-bypass-deployment-protection/protection-bypass-automation)）
を使って Deployment Protection のチェックを迂回し、認証なしでデプロイへ
アクセスします。ミドルウェアまで実際に到達したかどうかは、後述の手順で
返る応答が `WWW-Authenticate` ヘッダー付きの 401（ミドルウェア自身が
発行した応答）であることによって確認します。バイパス用シークレットは
Project Settings → Deployment Protection →
「Protection Bypass for Automation」で生成でき、`vercel deploy` が
作成するデプロイには `VERCEL_AUTOMATION_BYPASS_SECRET` という名前の
環境変数として自動的に設定されます（利用側コードでの参照は不要で、
検証用リクエストのヘッダー値として使うだけです）。

- **手順**（`<deployment-url>` は実際のデプロイ URL、シークレットは
  シェル履歴に残らないよう `read -s` 等で変数に読み込んでから使う）:

  ```bash
  read -s -p 'VERCEL_AUTOMATION_BYPASS_SECRET: ' BYPASS_SECRET
  echo

  # 1. 未認証（Authorization ヘッダーなし）が 401 になることを確認する。
  #    -D - でレスポンスヘッダーも出力し、Basic 認証ミドルウェア自身が
  #    返した 401 であることを WWW-Authenticate ヘッダーで確認する
  #    （Deployment Protection 自身の 401/302 と区別するため）。
  curl -sS -o /dev/null -D - -w '%{http_code}\n' \
    -H "x-vercel-protection-bypass: ${BYPASS_SECRET}" \
    "https://<deployment-url>/" | grep -Ei '^(HTTP|www-authenticate)|^401$'

  # 2. 正しい資格情報を付けると 200 になることを確認する
  #    （パスワードをコマンドライン引数に直接書かないため、`-u "<user>"`
  #    のみ指定して curl の対話プロンプトで入力する）。
  curl -sS -o /dev/null -w '%{http_code}\n' \
    -H "x-vercel-protection-bypass: ${BYPASS_SECRET}" \
    -u "<user>" \
    "https://<deployment-url>/"
  ```

  期待値どおりにならない場合（401 が返らない、`WWW-Authenticate` ヘッダー
  が確認できない、正しい資格情報でも 200 にならない、503 になる等）は、
  **Deployment Protection を無効化せず**、`config.json` の
  `middlewarePath` ルートの位置・`.vc-config.json`・
  `BASIC_AUTH_USER`/`BASIC_AUTH_PASSWORD` の設定を見直してから再デプロイ
  し、この確認をやり直してください。
- production では確認せず、まず preview デプロイ
  （`vercel deploy --prebuilt`、`--prod` なし）で確認することを推奨
  します。本番へ影響を与えずに確認できます。
- Protection Bypass for Automation のシークレットは、リポジトリ・
  README・CI ログのいずれにも書かないでください（前述のバイパス用
  トークンに関する注意と同じです）。

### Routing Middleware による Basic 認証

本節の手順は Build Output API（`config.json` の `routes`・`--prebuilt`、
案 c・vercel-ssg）を前提とします。Container Images（案 d・vercel-ssr）の
デプロイには未検証のため、そのまま適用しないでください。

Deployment Protection を無効化した公開デプロイに、追加の軽量な認証を
かけたい場合は、Build Output API の Routing Middleware で Basic 認証を
実装できます。**`examples/vercel-ssg` 本体はこの認証を opt-in で組み込み
済みです**（既定では無効。ビルド時に環境変数
`FANDHE_VERCEL_SSG_BASIC_AUTH` を正確に `1` に設定したときだけ有効化され
ます。詳細は [`examples/vercel-ssg` の README](../../examples/vercel-ssg/README.md)
の「Basic 認証（opt-in、既定では無効）」を参照してください）。以下では
仕組みと生成される内容を説明します（自前のプロジェクトへ同等の実装を
移植する際の参考にもなります）。

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

`examples/vercel-ssg` では、ビルド時に環境変数
`FANDHE_VERCEL_SSG_BASIC_AUTH=1` を指定するだけで、`src/main.rs` の
`output_root_assets()` が次の 3 点を `OUTPUT_ROOT`（`.vercel/output`）配下へ
`generate_assets` 経由で書き出します（フラグなしの既定ビルドではこの 3 点
は生成されず、`config.json` は導入前とバイト単位で同一のままです）。

- `/functions/_middleware.func/.vc-config.json`
- `/functions/_middleware.func/index.js`
- `middlewarePath` ルートを先頭に加えた `config.json`

自前のプロジェクトへ同等の実装を移植する場合は、`clean_output_dir()` が
毎回 `.vercel/output` を削除する構成である点に注意してください。手書き
ファイルを直接置いても消えてしまうため、追加ファイルは同じく
`generate_assets` で書き出す形にする必要があります。

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
- 資格情報の比較は入力の長さで早期リターンしない定数時間比較（後述）を
  使い、ユーザー名・パスワードの両方を必ず比較してから結果を結合します
  （`||` の短絡評価で比較回数を減らしません）。

以下は `examples/vercel-ssg` の `src/main.rs::MIDDLEWARE_INDEX_JS` と同一
内容です（乖離させないため、実装を変更した場合は両方を更新してください）。

```js
/**
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
```

#### 環境変数の登録

対話プロンプトで値を入力します（シェル履歴に値を残さないため、
コマンドライン引数や `echo | vercel env add` は避けてください）。

前述の「Deployment Protection（既定で有効）」の手順 1・2（ミドルウェア
組み込み済みデプロイの作成と、「Deployment Protection が有効なままの
Basic 認証確認（必須）」）は、いずれも `vercel deploy --prebuilt`
（`--prod` なし、すなわち preview デプロイ）でミドルウェアの動作を
確認する前提です。preview 環境に
`BASIC_AUTH_USER`/`BASIC_AUTH_PASSWORD` が未設定のままだと、ミドルウェアは
fail-closed の設計どおり常に 503 を返し、Basic 認証を確認できません。
そのため **preview への登録は任意ではなく必須**です。production のみに
登録して確認したい場合は、preview 登録を省略する代わりに `--prod` 付きで
デプロイして確認してください。

```bash
# 検証に使うデプロイ種別（preview / production）の両方に登録する。
# 少なくとも「動作確認に実際に使う環境」への登録は省略できない。
vercel env add BASIC_AUTH_USER production
vercel env add BASIC_AUTH_PASSWORD production

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
- [`examples/vercel-ssr` の README](../../examples/vercel-ssr/README.md)
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
- [Protection Bypass for Automation](https://vercel.com/docs/deployment-protection/methods-to-bypass-deployment-protection/protection-bypass-automation)
- [vercel env](https://vercel.com/docs/cli/env)
- [vercel remove](https://vercel.com/docs/cli/remove)
- [Managing environment variables](https://vercel.com/docs/environment-variables/managing-environment-variables)
- [Container Images](https://vercel.com/docs/functions/container-images)
- [Container Registry](https://vercel.com/docs/container-registry)
