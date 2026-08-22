#!/usr/bin/env node
/**
 * `verify.mjs` の負のセルフテスト（検証の検証）。
 *
 * テストランナー依存なしの node 単独実行（`node bench/ssr/lib/verify.test.mjs`）
 * で完結し、全ケース期待どおりなら exit 0、1 件でも期待と異なれば失敗内容を
 * stderr へ出力して exit 1 で終了する（fail-closed）。
 *
 * 目的: escape_ok 判定が「部分的なエスケープ漏れ」（PR #1370 codex 第 6 巡
 * レビュー P0 指摘。例: 開始タグのみ生の `<script>alert(1)&lt;/script&gt;`）を
 * 成功扱いにしないことを、正常系の非退行（偽陽性なし）とあわせて機械固定する。
 * 正常系の土台には依存パッケージゼロの vanilla renderer の実出力を使い、
 * そこへ文字列置換で欠陥を注入する。
 */

import { verify } from "./verify.mjs";
import { renderRows } from "../renderers/vanilla.mjs";

const ROWS = 50;

// vanilla の正常出力。ラベルは
// `Row {i} &amp; &quot;quoted&quot; &#39;single&#39; &lt;script&gt;alert(1)&lt;/script&gt;`
// の形で全 5 文字がエスケープされている。
const GOOD = renderRows(ROWS);

/**
 * ケース定義: { name, html, rows, expectEscapeOk }。
 * escape_ok の判定のみを対象とする（row_count_ok は既存検査のまま不変）。
 */
const CASES = [
  {
    // codex 指摘の実例: 開始タグのみ生（`<script>` が生で script 要素が成立）
    name: "partial leak: raw opening <script> only",
    html: GOOD.replaceAll("&lt;script&gt;", "<script>"),
    expectEscapeOk: false,
  },
  {
    // 終了タグのみ生
    name: "partial leak: raw closing </script> only",
    html: GOOD.replaceAll("&lt;/script&gt;", "</script>"),
    expectEscapeOk: false,
  },
  {
    // 許可リスト外要素の注入
    name: "injected <img onerror>",
    html: GOOD.replace("<tbody>", "<tbody><img src=x onerror=alert(1)>"),
    expectEscapeOk: false,
  },
  {
    // `&lt;` はあるが alert 部分が丸ごと欠落（内容改ざん）
    name: "content tampering: alert(1) removed",
    html: GOOD.replaceAll("alert(1)", ""),
    expectEscapeOk: false,
  },
  {
    // `&` のみ未エスケープ（第 2 層では検出できず補助検査が担う退行）
    name: "partial leak: raw ampersand only",
    html: GOOD.replaceAll("&amp;", "&"),
    expectEscapeOk: false,
  },
  {
    // 行ラベルの重複（Row 0 のラベルで Row 1 のラベルを上書き）
    name: "duplicated row label",
    html: GOOD.replace("Row 1 &amp;", "Row 0 &amp;"),
    expectEscapeOk: false,
  },
  {
    // 許可リスト外の生 `<`（タグとして不成立でも危険側へ倒す）
    name: "stray raw < in text",
    html: GOOD.replace("generated", "gen < erated"),
    expectEscapeOk: false,
  },
  {
    // 正常出力（vanilla renderer 相当）
    name: "well-escaped output passes (vanilla style)",
    html: GOOD,
    expectEscapeOk: true,
  },
  {
    // 表記ゆれ耐性: テキストノードで `>` `"` `'` を未エスケープのまま出す
    // solid / svelte 相当の正当な出力を偽陽性で落とさない
    name: "well-escaped output passes (solid style: bare > \" ')",
    html: GOOD.replaceAll("&gt;", ">")
      .replaceAll("&quot;", '"')
      .replaceAll("&#39;", "'"),
    expectEscapeOk: true,
  },
  {
    // 表記ゆれ耐性: lit / svelte 相当のハイドレーション用コメントを
    // 偽陽性で落とさない
    name: "well-escaped output passes (with hydration comments)",
    html: GOOD.replace("<tbody>", "<!--[--><tbody><!--lit-part-->").replace(
      "</tbody>",
      "<!--/lit-part--></tbody><!--]-->",
    ),
    expectEscapeOk: true,
  },
];

let failures = 0;
for (const { name, html, expectEscapeOk } of CASES) {
  const { escapeOk } = verify(html, ROWS);
  if (escapeOk !== expectEscapeOk) {
    failures += 1;
    process.stderr.write(
      `FAIL: ${name}: escapeOk=${escapeOk}, expected ${expectEscapeOk}\n`,
    );
  } else {
    process.stdout.write(`ok: ${name}\n`);
  }
}

if (failures > 0) {
  process.stderr.write(`verify.test.mjs: ${failures}/${CASES.length} case(s) failed\n`);
  process.exitCode = 1;
} else {
  process.stdout.write(`verify.test.mjs: all ${CASES.length} cases passed\n`);
}
