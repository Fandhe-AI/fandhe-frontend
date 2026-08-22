// フレームワーク横断 CSR ベンチマークの行データ生成・更新ロジック。
//
// 全フレームワークのアプリ（apps/<name>/main.*）が本モジュールを import し、
// create/update の入力データを完全に同一（id 採番・label 文字列）にすることで、
// フレームワーク間の計測差分が「同じワークロードに対する描画コストの差」だけに
// なるようにする（ワークロード自体の差異を測定ノイズに混ぜない）。
//
// label には HTML の特殊文字（&, ", ', <script> タグ）を意図的に含める。
// 各フレームワークは textContent 相当の既定エスケープ経路のみで label を
// 描画する契約であり、run_csr.mjs はこの文字列がタグとして解釈されず
// 生のテキストとして DOM に現れることを検証する（XSS 回帰の代理指標）。

/**
 * 行 i の label 文字列を生成する。
 * @param {number} i - 行インデックス（0 始まり）
 * @returns {string}
 */
export function makeLabel(i) {
  return `Row ${i} & "quoted" 'single' <script>alert(1)</script>`;
}

/**
 * count 件の行データ ({id, label}) を生成する。
 * id は 0..count-1 の連番とし、キー付きリスト描画のキーとして使う。
 * @param {number} count
 * @returns {{id: number, label: string}[]}
 */
export function generateRows(count) {
  const rows = new Array(count);
  for (let i = 0; i < count; i += 1) {
    rows[i] = { id: i, label: makeLabel(i) };
  }
  return rows;
}

/**
 * 10 行ごと（id % 10 === 0）の label 末尾へ ' !!!' を追記した新しい配列を返す。
 * 元の配列・行オブジェクトは変更しない（各フレームワークの不変更新パターンに
 * 合わせるため）。
 * @param {{id: number, label: string}[]} rows
 * @returns {{id: number, label: string}[]}
 */
export function updateRows(rows) {
  return rows.map((row) => (row.id % 10 === 0 ? { ...row, label: `${row.label} !!!` } : row));
}
