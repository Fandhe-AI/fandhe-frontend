// フレームワーク横断 CSR ベンチマークの比較対象一覧（正）。
//
// bench/PROTOCOL.md §1 の CSR 対象（7 種）と一致させる。run_csr.mjs
// （実行時間計測）と payload/measure.mjs（バンドルサイズ計測）の双方が
// この配列を import して使う。2 箇所で別々にハードコードすると、
// 一方だけ対象を増減したときに「一部だけ計測されているのに気付かない」
// fail-open（PR #1370 codex 再レビュー指摘、P1 x2）を招くため、
// 正本をここへ一元化する。
export const ALL_FRAMEWORKS = ["vanilla", "react", "preact", "vue", "svelte", "lit", "fandhe"];

// bench 各スクリプト（csr/build.mjs / csr/run_csr.mjs / payload/measure.mjs /
// ssr/run_ssr.mjs）共通の CLI 引数パーサ。契約（bench/PROTOCOL.md §3）:
//
// - `--framework` は直後に値必須。欠落（末尾・次要素が `--` 始まり）はエラー
//   （かつては `only=undefined` となり既定の全件実行へ fail-open していた、
//   PR #1370 codex 第 5 巡レビュー指摘 P1）。
// - 値は validNames との完全一致必須。不一致は既知リストを列挙してエラー。
//   パス構築（`join(DIST, name)` 等）より前に拒否することで、
//   `--framework ../../..` のような値による dist 外走査・配信
//   （パストラバーサル、同レビュー指摘 P0 x2）を遮断する。
// - `--framework` の重複指定はエラー（どちらが有効か曖昧なため）。
// - extraFlags に列挙されない未知の引数・オプションはエラー（fail-closed）。
//
// 成功時は `{ only, flags }`（only: 指定名 or null、flags: 指定された
// extraFlags の Set）、失敗時は `{ error }`（呼び出し側が stderr へ出力して
// `process.exitCode = 1` で終了する既存様式に合わせる）を返す。
// unknownValueHint は値不一致エラーへ追記する補足（build.mjs の
// 「fandhe は build.sh 担当」案内等）。
export function parseFrameworkCliArgs(argv, validNames, { extraFlags = [], unknownValueHint = "" } = {}) {
  let only = null;
  const flags = new Set();
  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === "--framework") {
      if (only !== null) {
        return { error: "duplicate --framework option (specify at most one framework)" };
      }
      const value = argv[i + 1];
      if (value === undefined || value.startsWith("--")) {
        return { error: `--framework requires a value (known: ${validNames.join(", ")})` };
      }
      if (!validNames.includes(value)) {
        return {
          error:
            `unknown --framework value: ${value} (known: ${validNames.join(", ")})` +
            (unknownValueHint ? `; ${unknownValueHint}` : ""),
        };
      }
      only = value;
      i += 1;
    } else if (extraFlags.includes(arg)) {
      if (flags.has(arg)) {
        return { error: `duplicate option: ${arg}` };
      }
      flags.add(arg);
    } else {
      return {
        error: `unknown argument: ${arg} (allowed: --framework <name>${extraFlags.length > 0 ? `, ${extraFlags.join(", ")}` : ""})`,
      };
    }
  }
  return { only, flags };
}
