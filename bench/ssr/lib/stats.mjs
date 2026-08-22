/**
 * 統計計算（百分位・平均・最小値）。
 *
 * `crates/xtask/src/bench_ssr.rs` の `percentile`/`stats_from_durations` と
 * 同一アルゴリズム（線形補間なし・`floor(p * (n - 1))` 番目）にすることで、
 * xtask（fandhe-frontend 自身の SSR 計測）と本ハーネス（他フレームワーク）の
 * 出力を同じ基準で横並び比較できるようにする。
 */

/**
 * ソート済み `sortedMs` から百分位を求める（`floor(p * (n - 1))` 番目、
 * 線形補間なし）。空配列は 0 を返す（`iters >= 1` を呼び出し元が保証する
 * ため実運用では到達しないが、防御的に扱う）。
 */
export function percentile(sortedMs, p) {
  if (sortedMs.length === 0) {
    return 0;
  }
  const idx = Math.floor(p * (sortedMs.length - 1));
  return sortedMs[idx];
}

/**
 * 表示・出力用に小数第 4 位へ丸める。`crates/xtask/src/bench_ssr.rs` の
 * `stats_to_json`（`{:.4}` フォーマット）と桁数を揃え、xtask 側の出力
 * （JSON 数値として同じ精度）と横並び比較したときの見た目のノイズを
 * 減らす（値の丸め誤差自体は計測の意味を変えない範囲）。
 */
function round4(v) {
  return Math.round(v * 10_000) / 10_000;
}

/**
 * 計測済みの経過時間（ミリ秒）配列から `iters`/`mean_ms`/`p50_ms`/`p95_ms`/
 * `min_ms` を計算する。
 */
export function statsFromDurations(durationsMs) {
  const sorted = [...durationsMs].sort((a, b) => a - b);
  const iters = sorted.length;
  const sum = sorted.reduce((acc, v) => acc + v, 0);
  const mean_ms = iters === 0 ? 0 : sum / iters;
  const min_ms = iters === 0 ? 0 : sorted[0];
  return {
    iters,
    mean_ms: round4(mean_ms),
    p50_ms: round4(percentile(sorted, 0.5)),
    p95_ms: round4(percentile(sorted, 0.95)),
    min_ms: round4(min_ms),
  };
}
