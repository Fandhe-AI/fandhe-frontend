#!/usr/bin/env node
/**
 * フレームワーク横断 SSR ベンチハーネスの実行入口。
 *
 * `crates/xtask/src/bench_ssr.rs`（`cargo run -p xtask --release --locked --
 * bench-ssr`）と同一ワークロード・同一統計アルゴリズムで、他フレームワーク
 * （react / preact / vue / solid / svelte / lit）と手書きベースライン
 * （vanilla）の SSR 性能を計測する。fandhe-frontend 自身は xtask 側の
 * 出力（JSON 1 行）をそのまま使う運用のため、本スクリプトには含めない。
 *
 * 使い方:
 *   node run_ssr.mjs                       # 全 7 renderer を順に実行
 *   node run_ssr.mjs --framework react      # react のみ実行
 *   node run_ssr.mjs --rows-10k-skip        # rows10k 計測を省略（動作確認用）
 *
 * renderer 間の相互干渉（グローバル状態汚染等）を避けるため各 renderer は
 * 独立モジュールとして設計しているが、起動オーバーヘッド削減のため
 * 同一プロセス内で順次実行する（計測対象は render 呼び出しのみのため
 * プロセス起動コストは計測に影響しない）。
 *
 * 出力: renderer 1 件につき JSON 1 行（stdout）。全件 escape_ok/row_count_ok
 * が true の場合のみ終了コード 0、いずれかが false なら 1（fail-closed）。
 */
import { runFramework, reportToJsonLine } from "./lib/runner.mjs";

const RENDERER_MODULES = {
  vanilla: "./renderers/vanilla.mjs",
  react: "./renderers/react.mjs",
  preact: "./renderers/preact.mjs",
  vue: "./renderers/vue.mjs",
  solid: "./renderers/solid.mjs",
  svelte: "./renderers/svelte.mjs",
  lit: "./renderers/lit.mjs",
};

// 出力順（stdout の 7 行の並び）を固定するための実行順。オブジェクトキー
// 順に依存せず明示する。
const FRAMEWORK_ORDER = [
  "vanilla",
  "react",
  "preact",
  "vue",
  "solid",
  "svelte",
  "lit",
];

function parseArgs(argv) {
  let framework = null;
  let rows10kSkip = false;
  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === "--framework") {
      framework = argv[i + 1];
      i += 1;
    } else if (arg === "--rows-10k-skip") {
      rows10kSkip = true;
    } else {
      throw new Error(`unknown argument: ${arg}`);
    }
  }
  return { framework, rows10kSkip };
}

async function main() {
  const { framework, rows10kSkip } = parseArgs(process.argv.slice(2));

  let targets = FRAMEWORK_ORDER;
  if (framework !== null) {
    if (!Object.hasOwn(RENDERER_MODULES, framework)) {
      process.stderr.write(
        `unknown --framework value: ${framework} (known: ${FRAMEWORK_ORDER.join(", ")})\n`,
      );
      process.exit(1);
    }
    targets = [framework];
  }

  let allOk = true;
  for (const name of targets) {
    const modulePath = RENDERER_MODULES[name];
    // eslint 等の静的解析非対象（動的 import 経路）。renderer ごとの
    // top-level await（例: svelte のコンパイル）は計測対象外の準備作業
    // として import 完了までに終わる。
    const renderer = await import(modulePath);
    const report = await runFramework(renderer, { rows10kSkip });
    process.stdout.write(`${reportToJsonLine(report)}\n`);
    if (!report.escape_ok || !report.row_count_ok) {
      allOk = false;
    }
  }

  process.exit(allOk ? 0 : 1);
}

main().catch((err) => {
  process.stderr.write(`bench-ssr fatal error: ${err.stack || err}\n`);
  process.exit(1);
});
