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
// production 相当ビルドでの計測を保証するため、renderer の dynamic import
// より前（本モジュールの評価時点）に NODE_ENV を明示代入する。
// react-dom/server と @vue/server-renderer は NODE_ENV 分岐で dev ビルド
// （余分な検証・警告コード入り）へフォールバックするため、未設定のまま
// 実行すると SSR 実行時間が dev ビルドのものになってしまう。他 6 種
// （vanilla / preact / solid / svelte / lit）は NODE_ENV に依存しない。
// 設定値は runner.mjs が notes へ `NODE_ENV=<値>` として記録し、
// 再発（未設定計測）を結果 JSON 上で機械検知できるようにする。
// なお下の静的 import（runner.mjs）は hoisting によりこの代入より先に
// 評価されるが、renderer 本体は main() 内の dynamic import でのみ読み
// 込むため、NODE_ENV 分岐を持つモジュールの評価は必ずこの代入の後になる。
process.env.NODE_ENV = "production";

import { runFramework, reportToJsonLine } from "./lib/runner.mjs";
// CLI 引数の検証（--framework の値必須・許可リスト照合・重複/未知引数の
// 拒否）は CSR/payload 側と共通のパーサ（bench/csr/frameworks.mjs）を
// 相対 import で共有し、重複実装しない（bench/PROTOCOL.md §3）。
import { parseFrameworkCliArgs } from "../csr/frameworks.mjs";

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

async function main() {
  // --framework の値欠落は かつて framework=undefined として既定の全件実行へ
  // fail-open していた（PR #1370 codex 第 5 巡レビュー指摘 P1 と同族）。
  // 値必須・FRAMEWORK_ORDER との完全一致・重複/未知引数の拒否を共通パーサで
  // renderer の import より前に fail-closed に検証する。
  const parsed = parseFrameworkCliArgs(process.argv.slice(2), FRAMEWORK_ORDER, {
    extraFlags: ["--rows-10k-skip"],
  });
  if (parsed.error) {
    process.stderr.write(`${parsed.error}\n`);
    process.exitCode = 1;
    return;
  }
  const framework = parsed.only;
  const rows10kSkip = parsed.flags.has("--rows-10k-skip");

  let targets = FRAMEWORK_ORDER;
  if (framework !== null) {
    // parseFrameworkCliArgs が許可リスト照合済みだが、RENDERER_MODULES との
    // 対応欠落（リスト間ドリフト）を検知する多層防御として残す。
    if (!Object.hasOwn(RENDERER_MODULES, framework)) {
      process.stderr.write(
        `unknown --framework value: ${framework} (known: ${FRAMEWORK_ORDER.join(", ")})\n`,
      );
      process.exitCode = 1;
      return;
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

  // run_csr.mjs / measure.mjs と同様式: process.exit() の即時終了ではなく
  // exitCode + 自然終了で stdout の flush を待つ。
  if (!allOk) {
    process.exitCode = 1;
  }
}

main().catch((err) => {
  process.stderr.write(`bench-ssr fatal error: ${err.stack || err}\n`);
  process.exitCode = 1;
});
