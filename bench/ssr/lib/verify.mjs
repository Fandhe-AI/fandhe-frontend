/**
 * 出力 HTML の fail-closed 検証（escape_ok / row_count_ok）。
 *
 * `runner.mjs` の `runFramework` から rows1k の最終出力に対して呼ばれる。
 * フレームワークごとにエスケープ表現が揺れる（例: `&quot;` vs `&#34;`、
 * `&#x27;` vs `&#39;`、テキストノードでは `>` を無害なため未エスケープの
 * まま出力する実装がある等）ため、「期待するエスケープ済み文字列との完全
 * 一致」ではなく、以下の 2 層 + 補助検査で判定する。
 *
 * かつての方式（生の `<script>alert(1)</script>` 完全一致の不在 + 実体参照の
 * 存在 + `alert(1)` 出現数）は、部分的なエスケープ漏れ（例: ラベルが
 * `<script>alert(1)&lt;/script&gt;` と出力され、生の開始タグだけで
 * script 要素が成立するケース）を成功扱いにする穴があった
 * （PR #1370 codex 第 6 巡レビュー P0 指摘）。現方式はこれを構造的に塞ぐ。
 * 外部 HTML パーサへの依存は追加しない（依存追加はユーザー承認事項）。
 *
 * 【第 1 層】生出力の `<` 許可リスト走査（注入要素の構造的排除）
 *   出力中のすべての `<` について、直後がワークロードの正当な構造
 *   （§2.1 の DOM 構造を成す許可タグの開始/終了タグ、またはフレームワークが
 *   挿入するハイドレーション用コメント `<!-- ... -->`）であることを要求する。
 *   許可リスト外の `<`（生の `<script`、`<img`、不明タグ、タグとして
 *   不成立の生 `<`）は 1 件でも escape_ok=false。これにより「開始タグだけ
 *   生」「終了タグだけ生」「別要素の注入」がすべて独立に検出される。
 *   許可タグ集合は 7 renderer の実出力に現れる最小集合（`ALLOWED_TAG_NAMES`
 *   参照）に絞り、実出力に現れない `<!DOCTYPE>` / `th` / `thead` 等は
 *   先回りで許可しない。コメントは svelte（`<!--[-->` / `<!--]-->`）と
 *   lit（`<!--lit-part ...-->` / `<!--/lit-part-->`）が実際に出力するため
 *   許可する（ラベル文字列は `!` を含まず、エスケープ漏れからコメント開始
 *   `<!--` が生成されることはないため、注入経路にはならない）。
 *
 * 【第 2 層】実体デコード後のラベル全文一致（内容の完全性）
 *   既知の実体参照のみをデコードした文字列に、期待ラベル全文
 *   `Row {i} & "quoted" 'single' <script>alert(1)</script>`（i = 0..rows-1）
 *   がちょうど 1 回ずつ出現することを検証する。旧方式の `alert(1)` 出現数
 *   検査より強く、表記ゆれ（`&quot;` vs `&#34;` vs 未エスケープの `"` 等）に
 *   影響されずに欠落・重複・内容改ざんを検出する。
 *
 * 【補助検査（旧方式から維持）】
 *   - `RAW_TAG` の不在: 第 1 層に厳密に包含される（生の `<script` は許可
 *     リスト外）が、検証の弱体化を避けるため維持する（コストは無視できる）。
 *   - `RAW_AMP_QUOTE` の不在・`UNESCAPED_AMPERSAND` の不在: `&` の
 *     エスケープ漏れは第 2 層では検出できない（ラベル原文自体が
 *     `& "quoted"` を含むため、未エスケープの `&` はデコード後も全文一致
 *     してしまう）。新方式に包含されない独立検査として必須のまま維持する。
 *   - `alert(1)` 出現数 = 行数: ラベル外の位置に紛れ込んだ余剰の
 *     `alert(1)`（第 2 層のラベル単位検査では捕まらない）を検出するため
 *     維持する。
 *
 * `row_count_ok` は `<tr` の出現回数が `expectedRows` と一致するかで判定する
 * （xtask 側 `verify` の `row_count_ok` と同一発想）。
 */

import { rowLabel } from "./label.mjs";

const RAW_TAG = "<script>alert(1)</script>";
const RAW_AMP_QUOTE = '& "quoted"';
const UNESCAPED_AMPERSAND =
  /&(?!(?:amp|quot|apos|lt|gt|#\d+|#x[0-9a-fA-F]+);)/;

// 第 1 層の許可タグ集合。7 renderer（vanilla / react / preact / vue / solid /
// svelte / lit）の実出力に現れるタグの全集合であり、これ以上増やさない
// （`head` は react のみが出力する）。新 renderer 追加で正当なタグが増えた
// 場合のみ、実出力を確認したうえでここへ追記する。
const ALLOWED_TAG_NAMES = [
  "html",
  "head",
  "body",
  "header",
  "h1",
  "table",
  "tbody",
  "tr",
  "td",
  "footer",
  "p",
];

// `<` 位置に sticky（y フラグ）で適用し、`</?タグ名` + 直後がタグ終端類
// （空白・`/`・`>`）であることまでを検査する。属性列は後続の
// rawStructureOk 側で「次の `>` まで `<` を含まない」ことのみ要求する
// （属性値の妥当性はワークロード上検証対象外。タグ名は大文字小文字不問）。
const ALLOWED_TAG_RE = new RegExp(
  `<\\/?(?:${ALLOWED_TAG_NAMES.join("|")})(?=[\\s/>])`,
  "iy",
);

/**
 * 第 1 層: 出力中のすべての `<` が許可タグまたはコメントの開始であることを
 * 走査で検証する。1 件でも許可リスト外なら false（fail-closed）。
 */
function rawStructureOk(html) {
  let pos = html.indexOf("<");
  while (pos !== -1) {
    if (html.startsWith("<!--", pos)) {
      // コメント: 対応する `-->` まで読み飛ばす（未終端は不正）。
      const end = html.indexOf("-->", pos + 4);
      if (end === -1) {
        return false;
      }
      pos = html.indexOf("<", end + 3);
      continue;
    }
    ALLOWED_TAG_RE.lastIndex = pos;
    if (!ALLOWED_TAG_RE.test(html)) {
      return false;
    }
    const nameEnd = ALLOWED_TAG_RE.lastIndex;
    const gt = html.indexOf(">", nameEnd);
    if (gt === -1) {
      return false;
    }
    // タグ名の後から `>` までの属性領域に `<` が現れないこと（タグの
    // 途中に別のタグ開始が食い込む形の壊れた出力を弾く）。終了タグには
    // 属性を認めない（空白のみ許容）。
    const between = html.slice(nameEnd, gt);
    if (between.includes("<")) {
      return false;
    }
    if (html[pos + 1] === "/" && between.trim() !== "") {
      return false;
    }
    pos = html.indexOf("<", gt + 1);
  }
  return true;
}

// 第 2 層のデコード対象: 既知の named entity 5 種 + 数値文字参照
// （10 進 / 16 進）。未知の named entity はデコードせずそのまま残す
// （デコーダを HTML 仕様の完全実装にしない意図的な最小化。ワークロードの
// エスケープ結果として現れうる表現はこれで尽きる）。
const NAMED_ENTITIES = {
  amp: "&",
  lt: "<",
  gt: ">",
  quot: '"',
  apos: "'",
};
const ENTITY_RE = /&(?:(amp|lt|gt|quot|apos)|#(\d+)|#[xX]([0-9a-fA-F]+));/g;

/** 既知の実体参照のみをデコードする（未知・不正なものは原文のまま残す）。 */
function decodeKnownEntities(s) {
  return s.replace(ENTITY_RE, (whole, named, dec, hex) => {
    if (named !== undefined) {
      return NAMED_ENTITIES[named.toLowerCase()];
    }
    const cp = parseInt(dec !== undefined ? dec : hex, dec !== undefined ? 10 : 16);
    // Unicode 範囲外・サロゲート単体は不正な参照としてデコードしない
    // （String.fromCodePoint の throw を避ける fail-safe）。
    if (!Number.isFinite(cp) || cp > 0x10ffff || (cp >= 0xd800 && cp <= 0xdfff)) {
      return whole;
    }
    return String.fromCodePoint(cp);
  });
}

/**
 * 第 2 層: デコード済み文字列に期待ラベル全文（i = 0..expectedRows-1）が
 * ちょうど 1 回ずつ出現することを検証する。
 *
 * rows が大きくても実用時間で終わるよう、全ラベル × 全文 indexOf の
 * O(rows × n) 走査は避け、次の 2 検査の組み合わせで「ちょうど 1 回ずつ」を
 * 導く（鳩の巣原理）:
 *   (a) ラベル共通末尾 `LABEL_SUFFIX` の出現総数 = expectedRows（単一パス）
 *   (b) 各ラベル全文が少なくとも 1 回出現（renderer は行順に出力するため
 *       前回一致位置からの cursor 走査で全体 O(n)。順序が異なる出力にも
 *       位置制約なしの indexOf でフォールバックし、偽陽性にしない）
 * 各ラベル全文の出現 1 件は共通末尾の出現 1 件を必ず含み、相異なるラベルは
 * 同一の末尾出現を共有できない（`Row {i}` の相互非包含）ため、(a)+(b) から
 * 「各ラベルちょうど 1 回・ラベル外の末尾出現ゼロ」が従う。
 */
const LABEL_SUFFIX = rowLabel(0).slice("Row 0".length);

function labelsExactlyOnce(decoded, expectedRows) {
  // (a) 共通末尾の出現総数（重複・余剰・欠落を総量で拘束する）
  let suffixCount = 0;
  let p = decoded.indexOf(LABEL_SUFFIX);
  while (p !== -1) {
    suffixCount += 1;
    p = decoded.indexOf(LABEL_SUFFIX, p + LABEL_SUFFIX.length);
  }
  if (suffixCount !== expectedRows) {
    return false;
  }

  // (b) 各ラベル全文の存在（行順前提の cursor 走査 + 順不同フォールバック）
  let cursor = 0;
  for (let i = 0; i < expectedRows; i += 1) {
    const label = rowLabel(i);
    let idx = decoded.indexOf(label, cursor);
    if (idx === -1) {
      idx = decoded.indexOf(label);
    }
    if (idx === -1) {
      return false;
    }
    cursor = idx + label.length;
  }
  return true;
}

export function verify(html, expectedRows) {
  const trCount = (html.match(/<tr/g) || []).length;
  const alertCount = (html.match(/alert\(1\)/g) || []).length;

  const escapeOk =
    rawStructureOk(html) &&
    labelsExactlyOnce(decodeKnownEntities(html), expectedRows) &&
    !html.includes(RAW_TAG) &&
    !html.includes(RAW_AMP_QUOTE) &&
    !UNESCAPED_AMPERSAND.test(html) &&
    alertCount === expectedRows;
  const rowCountOk = trCount === expectedRows;

  return { escapeOk, rowCountOk };
}
