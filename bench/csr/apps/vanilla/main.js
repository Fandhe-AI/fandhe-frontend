// vanilla（素の DOM API）による CSR ベンチアプリ。
// 他フレームワークとの比較基準（フレームワークオーバーヘッドなしの下限値）。
// createElement/textContent のみを使い、innerHTML 系 API は一切使わない
// （既定エスケープに相当する「テキストとして挿入」を DOM API レベルで保証する）。
import { generateRows, updateRows } from "../../common/rowData.mjs";

let rows = [];

function tbodyEl() {
  return document.querySelector("#bench-table tbody");
}

// rows の内容を tbody へ全再構築する。create は DocumentFragment へ一括構築してから
// 1 回の appendChild でコミットし、レイアウトスラッシングを避ける。
function render() {
  const tbody = tbodyEl();
  const frag = document.createDocumentFragment();
  for (const row of rows) {
    const tr = document.createElement("tr");
    const tdId = document.createElement("td");
    tdId.textContent = String(row.id);
    const tdLabel = document.createElement("td");
    tdLabel.textContent = row.label;
    tr.appendChild(tdId);
    tr.appendChild(tdLabel);
    frag.appendChild(tr);
  }
  tbody.textContent = "";
  tbody.appendChild(frag);
}

function create() {
  rows = generateRows(1000);
  render();
}

function update() {
  rows = updateRows(rows);
  render();
}

function clear() {
  rows = [];
  render();
}

function main() {
  window.__bench = { create, update, clear };
  document.querySelector("#create").addEventListener("click", create);
  document.querySelector("#update").addEventListener("click", update);
  document.querySelector("#clear").addEventListener("click", clear);
}

main();
