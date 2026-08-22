/**
 * 出力 HTML の fail-closed 検証（escape_ok / row_count_ok）。
 *
 * フレームワークごとにエスケープ表現が揺れる（例: `&quot;` vs `&#34;`、
 * `&#x27;` vs `&#39;`、テキストノードでは `>` を無害なため未エスケープの
 * まま出力する実装がある等）ため、「期待するエスケープ済み文字列との完全
 * 一致」ではなく、以下 3 条件の頑健な組み合わせで判定する（PROTOCOL 指示の
 * 「頑健な方式で設計してよい」に基づく設計判断）。
 *
 * - `RAW_TAG`（生の `<script>alert(1)</script>`）が出力に一切現れないこと。
 *   HTML の `<` はテキストノード中で常に危険側の文字（新しい要素の開始と
 *   解釈されうる）であり、まっとうな既定エスケープ実装は例外なくこれを
 *   `&lt;` 等へ変換するため、この文字列が生で残っていれば構造的な
 *   エスケープ漏れ（XSS）と断定できる。
 * - `RAW_AMP_QUOTE`（生の `& "quoted"`、すなわち未エスケープの `&` の直後に
 *   スペース + `"` が続く並び）が出力に一切現れないこと。`&` がエスケープ
 *   された場合は必ず `&amp;` 等の実体参照に化けるため、この生の並びが
 *   残っていれば `&` のエスケープ漏れを検知できる。
 * - `alert(1)` という部分文字列の出現回数が `expectedRows` と一致すること。
 *   エスケープの表記ゆれに関わらず `(` `)` はエスケープ対象外でどの
 *   フレームワークでも素通りするため、この部分文字列は「各行のラベルが
 *   欠落・重複なく 1 回ずつ出力されている」ことを表記ゆれに影響されず
 *   検証できる不変条件として機能する。
 * - `&` の出現がすべて既知の実体参照（`&amp;` `&quot;` `&apos;` `&lt;`
 *   `&gt;` `&#NN;` `&#xHH;`）の先頭としてのみ現れること
 *   （`UNESCAPED_AMPERSAND` の否定先読み）。`RAW_AMP_QUOTE`（`& "quoted"`）
 *   だけでは `&` と `"` の両方が未エスケープの場合しか検知できず、`"` の
 *   みエスケープされ `&` だけが未エスケープに壊れた回帰
 *   （`Row 0 & &quot;quoted&quot; ...`）を見逃す。この独立した述語は
 *   `&quot;` vs `&#34;` / `&#x27;` vs `&#39;` のような正当な表記ゆれとは
 *   直交する（どちらも「既知の実体参照の先頭」として許容される）ため、
 *   表記ゆれ耐性を弱めずに `&` のエスケープ漏れのみを追加で検知できる。
 *
 * `row_count_ok` は `<tr` の出現回数が `expectedRows` と一致するかで判定する
 * （xtask 側 `verify` の `row_count_ok` と同一発想）。
 */

const RAW_TAG = "<script>alert(1)</script>";
const RAW_AMP_QUOTE = '& "quoted"';
const UNESCAPED_AMPERSAND =
  /&(?!(?:amp|quot|apos|lt|gt|#\d+|#x[0-9a-fA-F]+);)/;

export function verify(html, expectedRows) {
  const trCount = (html.match(/<tr/g) || []).length;
  const alertCount = (html.match(/alert\(1\)/g) || []).length;

  const escapeOk =
    !html.includes(RAW_TAG) &&
    !html.includes(RAW_AMP_QUOTE) &&
    !UNESCAPED_AMPERSAND.test(html) &&
    alertCount === expectedRows;
  const rowCountOk = trCount === expectedRows;

  return { escapeOk, rowCountOk };
}
