// Lit 3 による CSR ベンチアプリ。カスタム要素（LitElement）は使わず、
// lit-html の render() + repeat ディレクティブを直接 tbody へ適用する
// （Shadow DOM を挟まないぶん、他フレームワークと同じ light DOM 上で
// querySelector による検証ができる）。
import { html, render } from "lit";
import { repeat } from "lit/directives/repeat.js";
import { generateRows, updateRows } from "../../common/rowData.mjs";

let rows = [];

function view() {
  return html`${repeat(
    rows,
    (row) => row.id,
    (row) => html`<tr>
      <td>${row.id}</td>
      <td>${row.label}</td>
    </tr>`,
  )}`;
}

function draw() {
  const tbody = document.querySelector("#bench-table tbody");
  render(view(), tbody);
}

function create() {
  rows = generateRows(1000);
  draw();
}

function update() {
  rows = updateRows(rows);
  draw();
}

function clear() {
  rows = [];
  draw();
}

function main() {
  draw();
  window.__bench = { create, update, clear };
  document.querySelector("#create").addEventListener("click", create);
  document.querySelector("#update").addEventListener("click", update);
  document.querySelector("#clear").addEventListener("click", clear);
}

main();
