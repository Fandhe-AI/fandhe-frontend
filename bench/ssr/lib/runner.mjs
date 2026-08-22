/**
 * 1 フレームワーク分の計測を実行し、JSON 1 行契約に整形する。
 *
 * `crates/xtask/src/bench_ssr.rs` の `measure_rows`/`Report::to_json_line` と
 * 同一のワークロード規模・統計アルゴリズムを踏襲する（rows1k=1,000 行
 * ウォームアップ 20 回・計測 100 回、rows10k=10,000 行 ウォームアップ 2 回・
 * 計測 10 回）。計測対象は各 renderer の `renderRows(rows)`（render to
 * string 呼び出しのみ）とし、ノード/コンポーネントツリーの定義自体は
 * renderer 側の設計に委ねる（PROTOCOL 指示: フレームワーク API 構造上
 * 呼び出しごとに再実行されるものはそのままでよい）。
 */
import { performance } from "node:perf_hooks";
import { statsFromDurations } from "./stats.mjs";
import { verify } from "./verify.mjs";

const ROWS_1K = 1_000;
const ROWS_1K_WARMUP = 20;
const ROWS_1K_ITERS = 100;

const ROWS_10K = 10_000;
const ROWS_10K_WARMUP = 2;
const ROWS_10K_ITERS = 10;

/**
 * `renderRows(rows)`（同期でも `Promise` を返す非同期でもよい）を
 * `warmup` 回ウォームアップした後 `iters` 回計測し、統計サマリと最終回の
 * HTML（検証用。ワークロードは決定的なためどの回の出力も同一）を返す。
 */
async function measureRows(renderRows, rows, warmup, iters) {
  for (let i = 0; i < warmup; i += 1) {
    await renderRows(rows);
  }

  const durationsMs = [];
  let lastHtml = "";
  for (let i = 0; i < iters; i += 1) {
    const start = performance.now();
    const html = await renderRows(rows);
    const elapsedMs = performance.now() - start;
    durationsMs.push(elapsedMs);
    lastHtml = html;
  }

  return { stats: statsFromDurations(durationsMs), html: lastHtml };
}

/**
 * 1 フレームワーク（`renderer`: `{ name, getVersion, renderRows }`）を計測し、
 * `run_ssr.mjs` が stdout へ出力する JSON 1 行分のオブジェクトを返す。
 *
 * - `rows10kSkip` が真の場合、rows10k 計測をスキップし `null` を出力する
 *   （動作確認を素早く回すためのオプトイン、既定は計測する）。
 */
export async function runFramework(renderer, { rows10kSkip = false } = {}) {
  const { stats: rows1k, html: html1k } = await measureRows(
    renderer.renderRows,
    ROWS_1K,
    ROWS_1K_WARMUP,
    ROWS_1K_ITERS,
  );

  const rows10k = rows10kSkip
    ? null
    : (await measureRows(renderer.renderRows, ROWS_10K, ROWS_10K_WARMUP, ROWS_10K_ITERS)).stats;

  const { escapeOk, rowCountOk } = verify(html1k, ROWS_1K);

  return {
    framework: renderer.name,
    version: renderer.getVersion(),
    mode: "ssr",
    workload_schema_version: 1,
    rows1k,
    rows10k,
    html_bytes_1k: Buffer.byteLength(html1k, "utf8"),
    escape_ok: escapeOk,
    row_count_ok: rowCountOk,
    // NODE_ENV を notes へ明示記録する（run_ssr.mjs が production を代入
    // する契約。react/vue が dev ビルドで計測される事故を結果 JSON 上で
    // 機械検知できるようにするため、実際の値をそのまま記録する）。
    notes: `node ${process.version}; NODE_ENV=${process.env.NODE_ENV}`,
  };
}

export function reportToJsonLine(report) {
  return JSON.stringify(report);
}
