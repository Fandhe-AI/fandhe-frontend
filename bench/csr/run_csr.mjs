#!/usr/bin/env node
// フレームワーク横断 CSR ベンチマークの実行ハーネス。
//
// bench/csr/dist/<name>/ を 127.0.0.1 の使い捨て HTTP サーバーで配信し、
// playwright-core が起動するシステム chromium 上で create/update/clear の
// 実行時間を計測する。
//
// 計測境界（bench/PROTOCOL.md §2.2 参照）: 当初はダブル requestAnimationFrame
// で「描画完了」を近似していたが、この環境の chromium（snap 版）は headless
// でも rAF が約 60Hz（平均間隔 17.5ms）固定であり、`--disable-frame-rate-limit`
// / `--disable-gpu-vsync` を付けても解除されないことを実測で確認した。
// ダブル rAF 境界は常に約 33ms の vsync 床を伴い、全フレームワークが
// create ≈ 31〜33ms へ張り付いて判別力を失っていたため、計測境界を
// 「__bench[op]() の完了（各アプリが flushSync/nextTick 等で DOM 反映まで
// 保証する）+ offsetHeight 読み出しによる強制 layout flush」へ変更した
// （__benchMeasure 参照）。paint（実際の画面書き換え）は計測に含まない
// 既知の限界であり、rAF 待ちを廃したことで paint 完了の近似はできなくなる。
//
// イシュー #1377: 上記の一体計測はさらに「op 時間（DOM 反映まで）」と
// 「layout flush 時間」の 2 区間へ分離し、op_ms/layout_ms/total_ms の
// 3 系列を結果 JSON へ記録する（__benchMeasure 参照）。total_ms は従来
// 境界と同一定義で意味不変、比較 KPI は引き続き total_ms（既存
// create_ms/update_ms/clear_ms）とする。fandhe 改善イシュー群（#1371 配下）
// の改善追跡 KPI は op_ms（layout flush は全フレームワーク共通の床であり
// ハーネス側では改善不能）。詳細は bench/PROTOCOL.md §2.2 参照。
//
// 計測前に「既定エスケープ経路のみで label が安全にテキスト挿入されているか」
// を検証し（XSS 回帰の代理指標）、fail-closed で不合格を検知する。
//
// 統計（mean/p50/p95/min）は crates/xtask/src/bench_ssr.rs の
// percentile()（floor(p*(n-1)) 番目）と同一アルゴリズムで揃える。
import { chromium } from "playwright-core";
import { createServer } from "node:http";
import { execSync } from "node:child_process";
import { existsSync, readFileSync, realpathSync } from "node:fs";
import { dirname, extname, join, resolve, sep } from "node:path";
import { fileURLToPath } from "node:url";
import { ALL_FRAMEWORKS, parseFrameworkCliArgs } from "./frameworks.mjs";

const ROOT = dirname(fileURLToPath(import.meta.url));
const DIST = join(ROOT, "dist");

// join(DIST, name) の第 2 層防御（許可リスト照合が主、これは多層防御）:
// resolve 結果が DIST 配下でなければ即エラーにする。この dir は
// startStaticServer の配信 root になるため、ここを通らない値で dist 外の
// ディレクトリが chromium（--no-sandbox）へ配信されることを構築点でも
// 遮断する（.claude/rules/security.md A01。startStaticServer 側の境界検証
// は root からの脱出のみを防ぎ、root 自体の妥当性はここが担う）。
function frameworkDistDir(name) {
  const distRoot = resolve(DIST);
  const dir = resolve(distRoot, name);
  if (!dir.startsWith(distRoot + sep)) {
    throw new Error(`refusing to serve a directory outside dist root: ${dir}`);
  }
  return dir;
}

const MIME = {
  ".html": "text/html; charset=utf-8",
  ".js": "text/javascript; charset=utf-8",
  ".mjs": "text/javascript; charset=utf-8",
  ".json": "application/json; charset=utf-8",
  ".wasm": "application/wasm",
  ".css": "text/css; charset=utf-8",
};

// --- chromium 実行パスの解決 ------------------------------------------------

function commandExists(cmd) {
  try {
    execSync(`command -v ${cmd}`, { stdio: "ignore" });
    return true;
  } catch {
    return false;
  }
}

function resolveChromiumPath() {
  const envPath = process.env.BENCH_CHROMIUM;
  if (envPath) {
    // 明示指定が不正なら既知パスへ silent fallback せずエラー停止する
    // （fallback すると「指定したつもりの chromium」と別バイナリで計測した
    // 結果が区別なく混ざる。fail-closed）。
    if (!existsSync(envPath)) {
      throw new Error(`BENCH_CHROMIUM=${envPath} not found`);
    }
    return envPath;
  }
  if (existsSync("/usr/bin/chromium-browser")) return "/usr/bin/chromium-browser";
  if (existsSync("/snap/bin/chromium")) return "/snap/bin/chromium";
  if (commandExists("google-chrome")) return "google-chrome";
  throw new Error(
    "chromium executable not found (checked BENCH_CHROMIUM env, /usr/bin/chromium-browser, /snap/bin/chromium, google-chrome)",
  );
}

// --- 静的ファイル配信（外部アクセスなし・127.0.0.1 バインドのみ） -----------

function startStaticServer(rootDir) {
  // symlink 境界検証（後述）の基準も realpath で揃えるため、root 自体を
  // 先に realpath 化しておく（dist が symlink 経由で参照されていても
  // 正しく配下判定できるようにする）。
  const root = realpathSync(resolve(rootDir));
  const server = createServer((req, res) => {
    // 不正なパーセントエンコーディングは decodeURIComponent が throw する。
    // 未捕捉のままハンドラ外へ逃すとサーバごと落ちるため 400 に落とす。
    let urlPath;
    try {
      urlPath = decodeURIComponent(req.url.split("?")[0]);
    } catch {
      res.writeHead(400);
      res.end();
      return;
    }
    if (urlPath === "/") urlPath = "/index.html";
    const filePath = resolve(join(root, urlPath));
    // パストラバーサル防止: 解決後のパスが配信ルートそのもの、または
    // 「ルート + セパレータ」で始まることを確認する（素の startsWith だと
    // 兄弟ディレクトリ名がルート名を接頭辞に持つ場合に境界外を許すため）。
    if (filePath !== root && !filePath.startsWith(root + sep)) {
      res.writeHead(403);
      res.end();
      return;
    }
    try {
      // symlink 経由の境界外配信防止: パス文字列上は root 配下でも、
      // dist 内に境界外を指す symlink が紛れ込んでいれば realpath は
      // 外へ出る。realpath（実体パス）でも root（realpath 済み）配下で
      // あることを検証する（.claude/rules/security.md A01）。
      const realPath = realpathSync(filePath);
      if (realPath !== root && !realPath.startsWith(root + sep)) {
        res.writeHead(403);
        res.end();
        return;
      }
      const body = readFileSync(realPath);
      const type = MIME[extname(filePath)] ?? "application/octet-stream";
      res.writeHead(200, { "Content-Type": type });
      res.end(body);
    } catch {
      res.writeHead(404);
      res.end();
    }
  });
  return new Promise((resolve) => {
    server.listen(0, "127.0.0.1", () => resolve(server));
  });
}

// --- 統計（xtask bench_ssr::percentile と同一アルゴリズム） -----------------

function percentile(sortedMs, p) {
  if (sortedMs.length === 0) return 0;
  const idx = Math.floor(p * (sortedMs.length - 1));
  return sortedMs[idx] ?? sortedMs[sortedMs.length - 1];
}

function statsFromDurations(durationsMs) {
  const sorted = [...durationsMs].sort((a, b) => a - b);
  const iters = sorted.length;
  const sum = sorted.reduce((a, b) => a + b, 0);
  return {
    iters,
    mean_ms: iters === 0 ? 0 : sum / iters,
    p50_ms: percentile(sorted, 0.5),
    p95_ms: percentile(sorted, 0.95),
    min_ms: iters === 0 ? 0 : sorted[0],
  };
}

// --- ページ内計測ヘルパー ----------------------------------------------------

// 検証（validate）専用: 操作 + ダブル rAF 待ち（描画完了の近似）。
// 計測経路（__benchMeasure）からは意図的に外している（上記モジュール
// コメント参照）。
async function doubleRaf(page) {
  await page.evaluate(
    () => new Promise((resolve) => requestAnimationFrame(() => requestAnimationFrame(() => resolve()))),
  );
}

// 計測境界: __bench[op]() の完了（Promise を返すアプリは await で同期適用を
// 保証する）を待ってから、bench-table の offsetHeight を読み出して
// style/layout を強制 flush する（rAF を挟まないため、paint 完了までは
// 含まない）。
//
// イシュー #1377: 親トラッキング #1371 の区間プロファイル実測で、CSR total
// の最大要素は layout flush（create で約 10ms、全フレームワーク共通の床）
// であり、wasm 側の改善が効くのは op 部分（DOM 反映までの時間）のみと
// 判明したため、performance.now() を 3 点（開始前 t0 / __bench[op]() 完了
// 直後 t1 / offsetHeight 読み出し後 t2）で取り、op_ms（t1 - t0、DOM 反映
// まで）・layout_ms（t2 - t1、強制 layout flush）・total_ms（t2 - t0、
// 従来境界と同一・意味不変）の 3 区間へ分離する。この計測点はハーネス
// 共通ヘルパー（全フレームワーク一律）にのみ存在し、fandhe 専用の計測点・
// 分岐は作らない（公平性維持、bench/PROTOCOL.md §4 参照）。1 反復内では
// op_ms + layout_ms === total_ms が成り立つが、統計値（mean/p50/p95/min）は
// 系列ごとに独立集計するため percentile の加法性はない（bench/PROTOCOL.md
// §2.2 参照）。
async function installMeasureHelper(page) {
  await page.evaluate(() => {
    window.__benchMeasure = async (op) => {
      const t0 = performance.now();
      await window.__bench[op]();
      const t1 = performance.now();
      void document.getElementById("bench-table").offsetHeight;
      const t2 = performance.now();
      return { op_ms: t1 - t0, layout_ms: t2 - t1, total_ms: t2 - t0 };
    };
  });
}

// __benchMeasure の返り値（{op_ms, layout_ms, total_ms}）をそのまま返す。
// timedRuns 側で 3 系列に分けて収集する。
async function measureOnce(page, op) {
  return page.evaluate((o) => window.__benchMeasure(o), op);
}

// 検証用: 操作 + ダブル rAF 待ち（描画完了の近似、validate() 専用）。
async function callBench(page, op) {
  await page.evaluate((o) => window.__bench[o](), op);
  await doubleRaf(page);
}

// 計測ループ内の未計測な状態リセット用: __bench[op]() の完了を待ち、
// __benchMeasure と同じく offsetHeight で強制 layout flush する
// （rAF は挟まない）。この flush を省略すると、未計測の before 手順で
// 生じた pending な style/layout 再計算が後続の計測対象 op の
// offsetHeight 読み出し時にまとめて実行されてしまい、before 側のコストが
// 計測値へ混入する（実測で lit の clear が異常値化する形で顕在化した）。
async function settleOp(page, op) {
  await page.evaluate(async (o) => {
    await window.__bench[o]();
    void document.getElementById("bench-table").offsetHeight;
  }, op);
}

// reps 回、op を計測する。before が指定された場合は計測前（未計測扱い）に
// 一度呼んで状態を整える（例: create 計測前に毎回 clear してから測る）。
// measureOnce が返す {op_ms, layout_ms, total_ms} を系列ごとの配列へ分けて
// 集計する（イシュー #1377。op/layout 分離計測、bench/PROTOCOL.md §2.2）。
async function timedRuns(page, op, reps, before) {
  const durations = { op: [], layout: [], total: [] };
  for (let i = 0; i < reps; i += 1) {
    if (before) await settleOp(page, before);
    const { op_ms, layout_ms, total_ms } = await measureOnce(page, op);
    durations.op.push(op_ms);
    durations.layout.push(layout_ms);
    durations.total.push(total_ms);
  }
  return durations;
}

// ウォームアップ: create→clear を rounds 回、未計測で往復する
// （JIT ウォームアップ・初回描画コストの計測混入を避ける）。
async function warmup(page, rounds) {
  for (let i = 0; i < rounds; i += 1) {
    await settleOp(page, "create");
    await settleOp(page, "clear");
  }
}

// --- 検証（fail-closed） -----------------------------------------------------

async function validate(page) {
  await callBench(page, "clear");
  const scriptCountBaseline = await page.evaluate(() => document.scripts.length);

  await callBench(page, "create");
  const rowCountAfterCreate = await page.evaluate(
    () => document.querySelectorAll("#bench-table tbody tr").length,
  );
  const sampleLabel = await page.evaluate(() => {
    const trs = document.querySelectorAll("#bench-table tbody tr");
    const target = trs[500] ?? trs[0];
    return target ? target.children[1]?.textContent ?? null : null;
  });
  const scriptCountAfterCreate = await page.evaluate(() => document.scripts.length);
  // label 文字列（`<script>alert(1)</script>` 等のタグ様文字列を含む）が
  // innerHTML 経由で要素として注入されていないことを確認する。innerHTML
  // 経由で挿入された <script> は仕様上実行されないため「実行されたか」を
  // 検知する手段（例: window.__xssExecuted のようなフラグ）はどのワーク
  // ロードもそもそも設定せず常に無意味な true になる（PR #1370 Bugbot
  // 指摘）。代わりに「要素として注入されていない＝テキストとして
  // 保持されている」ことを直接検証する。
  const injectedElementFound = await page.evaluate(
    () => document.querySelector("#bench-table script, #bench-table img, #bench-table svg") !== null,
  );

  await callBench(page, "update");
  const updatedCount = await page.evaluate(() => {
    const trs = document.querySelectorAll("#bench-table tbody tr");
    let count = 0;
    for (const tr of trs) {
      const label = tr.children[1]?.textContent ?? "";
      if (label.endsWith(" !!!")) count += 1;
    }
    return count;
  });

  await callBench(page, "clear");
  const rowCountAfterClear = await page.evaluate(
    () => document.querySelectorAll("#bench-table tbody tr").length,
  );

  const rowsOk = rowCountAfterCreate === 1000 && updatedCount === 100 && rowCountAfterClear === 0;
  const escapeOk =
    typeof sampleLabel === "string" &&
    sampleLabel.includes("<script>alert(1)</script>") &&
    scriptCountAfterCreate === scriptCountBaseline &&
    !injectedElementFound;

  return { rowsOk, escapeOk };
}

// --- フレームワーク 1 件分の実行 --------------------------------------------

async function runFramework(browser, name, chromiumNote) {
  const distDir = frameworkDistDir(name);
  if (!existsSync(join(distDir, "index.html"))) {
    console.error(`[run] ${name}: skip (not built)`);
    return null;
  }

  // meta.json はビルダー（build.mjs / fandhe は build.sh）が必ず書く契約
  // ファイル。不在・パース不能・version 欠落はビルド成果物が壊れている
  // 証拠なので、payload/measure.mjs と同じく fail-closed でエラーにする
  // （version=unknown のまま計測を続ける fail-soft は、どのバージョンを
  // 測ったのか結果 JSON から復元できなくなる）。
  const metaPath = join(distDir, "meta.json");
  if (!existsSync(metaPath)) {
    throw new Error(`meta.json not found: ${metaPath} — rebuild (bench/csr/build.mjs, or bench/csr/fandhe/build.sh for fandhe)`);
  }
  let meta;
  try {
    meta = JSON.parse(readFileSync(metaPath, "utf8"));
  } catch {
    throw new Error(`meta.json is not valid JSON: ${metaPath} — rebuild (bench/csr/build.mjs, or bench/csr/fandhe/build.sh for fandhe)`);
  }
  if (typeof meta.version !== "string" || meta.version === "") {
    throw new Error(`meta.json lacks the "version" field: ${metaPath} — rebuild (bench/csr/build.mjs, or bench/csr/fandhe/build.sh for fandhe)`);
  }
  const version = meta.version;

  const server = await startStaticServer(distDir);
  const { port } = server.address();

  const context = await browser.newContext();
  const page = await context.newPage();

  try {
    await page.goto(`http://127.0.0.1:${port}/index.html`, { waitUntil: "load" });
    // window.__bench は同期的な <script> 実行で即座に用意できるアプリ
    // （vanilla/lit 等）もあれば、fandhe のように module スクリプト
    // （bootstrap.js）内で `await init()`（wasm 初期化）を経てから代入する
    // アプリもある。
    // load イベント完了だけでは後者が未定義のままになり得るため、
    // window.__bench.{create,update,clear} が揃うまで明示的に待つ。
    await page.waitForFunction(
      () =>
        typeof window.__bench === "object" &&
        typeof window.__bench.create === "function" &&
        typeof window.__bench.update === "function" &&
        typeof window.__bench.clear === "function",
    );
    await installMeasureHelper(page);

    const { rowsOk, escapeOk } = await validate(page);

    await warmup(page, 5);

    const REPS = 25;
    const createDurations = await timedRuns(page, "create", REPS, "clear");
    // update は毎回、未計測の create（settleOp 経由。layout flush 込みの
    // リセット）で未更新の 1,000 行へ戻してから計測する。before を
    // 付けずに 25 回連続で update を適用すると、対象行の label へ
    // ` !!!` が累積し（1 回目 +4 文字 → 25 回目で +100 文字）、
    // PROTOCOL §2.2 が定義する「100 行へ ` !!!` を追記」という同一
    // ワークロードを毎回計測できなくなる（PR #1370 codex レビュー
    // 指摘 P1）。fandhe の bench_update も同様に累積する実装だが、
    // create がフル初期状態を再構築するためこのハーネス側の変更のみで
    // 全フレームワークに一律で効く。
    const updateDurations = await timedRuns(page, "update", REPS, "create");
    const clearDurations = await timedRuns(page, "clear", REPS, "create");

    // 既存キー（create_ms/update_ms/clear_ms）は total 系列＝従来境界と
    // 同一定義のまま名前・形とも不変で、比較 KPI として維持する
    // （bench/PROTOCOL.md §2.2）。*_op_ms/*_layout_ms はイシュー #1377 で
    // 追加した改善追跡用の分離計測キー（キー追加のみの後方互換）。
    return {
      framework: name,
      version,
      mode: "csr",
      workload_schema_version: 1,
      create_ms: statsFromDurations(createDurations.total),
      update_ms: statsFromDurations(updateDurations.total),
      clear_ms: statsFromDurations(clearDurations.total),
      create_op_ms: statsFromDurations(createDurations.op),
      create_layout_ms: statsFromDurations(createDurations.layout),
      update_op_ms: statsFromDurations(updateDurations.op),
      update_layout_ms: statsFromDurations(updateDurations.layout),
      clear_op_ms: statsFromDurations(clearDurations.op),
      clear_layout_ms: statsFromDurations(clearDurations.layout),
      rows_ok: rowsOk,
      escape_ok: escapeOk,
      notes: chromiumNote,
    };
  } finally {
    await context.close();
    await new Promise((resolve) => server.close(resolve));
  }
}

async function main() {
  // 引数検証は共通パーサへ委譲する（値必須・ALL_FRAMEWORKS との完全一致・
  // 重複/未知引数の拒否、bench/PROTOCOL.md §3）。配信 root のパス構築・
  // ブラウザ起動より前に不正値を拒否することがパストラバーサル遮断の
  // 主防御になる（PR #1370 codex 第 5 巡レビュー指摘 P0）。
  const parsed = parseFrameworkCliArgs(process.argv.slice(2), ALL_FRAMEWORKS);
  if (parsed.error) {
    console.error(`[run] ${parsed.error}`);
    process.exitCode = 1;
    return;
  }
  const only = parsed.only;
  const targets = only ? [only] : ALL_FRAMEWORKS;

  const chromiumPath = resolveChromiumPath();
  // 計測経路は rAF を経由しないため vsync 量子化の影響を受けない
  // （上記モジュールコメント参照）。`--disable-frame-rate-limit` /
  // `--disable-gpu-vsync` はこの環境の chromium（snap 版）では rAF の
  // 固定 60Hz を解除できないことを実測で確認済みのため付与しない
  // （validate() のダブル rAF 待ちは検証用途のみで計測境界には現れない）。
  const browser = await chromium.launch({
    executablePath: chromiumPath,
    headless: true,
    // --no-sandbox: snap 版 chromium はサンドボックス起動に失敗するため
    // 無効化する。本ハーネスは 127.0.0.1 配信の自前ページのみを開く
    // ローカル専用ツールで、外部入力・外部サイトを扱わないため許容する。
    args: ["--no-sandbox", "--disable-dev-shm-usage"],
  });
  // 計測に使った chromium の実バージョンを結果 JSON の notes へ記録する
  // （数値はブラウザバージョンに依存するため〔bench/PROTOCOL.md §4〕、
  // パスだけでは結果の再現条件を復元できない）。
  const chromiumNote = `chromium ${chromiumPath} version ${browser.version()}`;

  let anyFailed = false;
  const measuredNames = [];
  const skippedNames = [];
  try {
    for (const name of targets) {
      let result;
      try {
        result = await runFramework(browser, name, chromiumNote);
      } catch (err) {
        // 1 フレームワークの実行時エラー（壊れた dist・meta.json 契約違反・
        // ページ内例外等）で残りの計測結果まで失わないよう、該当
        // フレームワークのみ失敗として stderr へ報告し、残りは続行する。
        // 終了コードは anyFailed 経由で必ず 1 になる（fail-closed は不変）。
        console.error(`[run] ${name}: FAILED (${err.message ?? err})`);
        anyFailed = true;
        continue;
      }
      if (result === null) {
        skippedNames.push(name);
        continue;
      }
      measuredNames.push(name);
      console.log(JSON.stringify(result));
      if (!result.rows_ok || !result.escape_ok) anyFailed = true;
    }
  } finally {
    await browser.close();
  }

  // 既定実行（--framework 未指定）では ALL_FRAMEWORKS 全 7 種の dist が
  // 揃い測定完了することを必須とする。runFramework() は未ビルド対象を
  // null で返すだけであり、これをループ側が黙って除外すると 7 種中
  // 1 件でも測定できれば終了コード 0 になってしまう（fail-open、
  // PR #1370 codex 再レビュー指摘 P1）。--framework <name> による
  // 明示的な部分実行のときだけ、その 1 件のみの成功を許可する
  // （payload/measure.mjs と同じ「既定は全種必須・--framework は例外」
  // という契約、bench/PROTOCOL.md §2.2 参照）。
  if (!only && skippedNames.length > 0) {
    console.error(
      `[run] missing framework(s) under default full run (${measuredNames.length}/${ALL_FRAMEWORKS.length} built): ` +
        `${skippedNames.join(", ")} — build them first (bench/csr/build.mjs, ` +
        `bench/csr/fandhe/build.sh), or pass --framework <name> for an explicit partial run`,
    );
    process.exitCode = 1;
    return;
  }
  // --framework <name> 指定時に対象自体が未ビルドの場合（skippedNames に
  // 1 件のみ入り measuredNames が空になる）も、同様に fail-closed とする。
  if (measuredNames.length === 0) {
    console.error(`[run] no framework was measured (target: ${targets.join(", ")})`);
    process.exitCode = 1;
    return;
  }

  if (anyFailed) {
    console.error("[run] one or more frameworks failed validation (rows_ok/escape_ok)");
    process.exitCode = 1;
  }
}

main().catch((err) => {
  console.error(err);
  process.exitCode = 1;
});
