/**
 * ベースライン renderer（フレームワーク非依存）。
 *
 * vanilla ベースラインでも HTML 文字列の直接組み立ては行わず、最小の
 * ノード木構築（`el` / `text`）+ 既定エスケープ経由のシリアライズを通す
 * （リポジトリ規約「HTML 文字列の直接組み立て禁止・必ずノード木 API を
 * 使う」への準拠。エスケープを伴う手書き SSR の現実的な下限としても
 * 妥当なベースラインである）。シリアライザはテキストノード・属性値を
 * 必ずエスケープしてから出力し、エスケープ迂回経路（raw HTML 挿入）は
 * 提供しない。
 *
 * 依存パッケージを持たないため `getVersion()` は Node 自身のバージョンを
 * 返す（`bench/ssr/run_ssr.mjs` の `--framework vanilla` で単独実行可能）。
 * 各フレームワークの render 呼び出しコストを相対評価する基準点として使う。
 */
import { rowLabel } from "../lib/label.mjs";

const ESCAPE_MAP = {
  "&": "&amp;",
  "<": "&lt;",
  ">": "&gt;",
  '"': "&quot;",
  "'": "&#39;",
};

// 既定エスケープ（& < > " ' の 5 文字）。テキストノード・属性値の
// 双方がシリアライズ時に必ずこの関数を経由する（迂回経路なし）。
function escapeHtml(s) {
  return s.replace(/[&<>"']/g, (c) => ESCAPE_MAP[c]);
}

// --- 最小ノード木 API（コンストラクタ 2 種のみ） ---

// タグ名・属性名として許可する形（英小文字始まり + 英小文字/数字/ハイフン）。
// serialize はタグ名・属性名をエスケープせずそのまま出力するため、この
// 検証が「値のみエスケープすれば安全」という契約の前提になる。
const NAME_PATTERN = /^[a-z][a-z0-9-]*$/;

// 要素ノード。attrs は {name: value} の平坦なオブジェクト、children は
// ノード配列。value・children のテキストはシリアライズ時にエスケープされる。
//
// 契約: タグ名・属性名は呼び出し側リテラル（信頼済み定数）前提であり、
// エスケープ対象は値（テキストノード・属性値）のみ。前提が崩れた場合に
// 備え、タグ名・属性名は NAME_PATTERN に一致しなければ throw する
// （fail-closed。動的な名前を渡してシリアライザの非エスケープ出力へ
// 到達する誤用を構築時点で遮断する）。
function el(tag, attrs, children) {
  if (!NAME_PATTERN.test(tag)) {
    throw new Error(`invalid tag name (must match ${NAME_PATTERN}): ${tag}`);
  }
  for (const name of Object.keys(attrs)) {
    if (!NAME_PATTERN.test(name)) {
      throw new Error(`invalid attribute name (must match ${NAME_PATTERN}): ${name}`);
    }
  }
  return { kind: "element", tag, attrs, children };
}

// テキストノード。value はシリアライズ時に必ずエスケープされる
// （raw HTML を注入する手段は存在しない）。
function text(value) {
  return { kind: "text", value: String(value) };
}

// ノード木を HTML 文字列へシリアライズする唯一の出口。テキストノードと
// 属性値は escapeHtml を必ず通す。本ワークロードに void 要素は登場しない
// ため、要素は常に開始・終了タグの組で出力する。
function serialize(node) {
  if (node.kind === "text") {
    return escapeHtml(node.value);
  }
  let html = `<${node.tag}`;
  for (const [name, value] of Object.entries(node.attrs)) {
    html += ` ${name}="${escapeHtml(String(value))}"`;
  }
  html += ">";
  for (const child of node.children) {
    html += serialize(child);
  }
  html += `</${node.tag}>`;
  return html;
}

function renderRow(i) {
  return el("tr", {}, [
    el("td", {}, [text(i)]),
    el("td", {}, [text(rowLabel(i))]),
  ]);
}

export const name = "vanilla";

export function getVersion() {
  return process.version;
}

export function renderRows(rows) {
  const rowNodes = [];
  for (let i = 0; i < rows; i += 1) {
    rowNodes.push(renderRow(i));
  }
  const tree = el("html", {}, [
    el("body", {}, [
      el("header", {}, [el("h1", {}, [text("Benchmark")])]),
      el("table", { id: "bench-table" }, [el("tbody", {}, rowNodes)]),
      el("footer", {}, [el("p", {}, [text(`generated ${rows} rows`)])]),
    ]),
  ]);
  return serialize(tree);
}
