/**
 * `node_modules/<pkg>/package.json` を直接読み取り、実インストール
 * バージョンを解決する。
 *
 * `import(pkg + '/package.json', { with: { type: 'json' } })` や
 * `createRequire(...)(pkg + '/package.json')` は `lit` のように
 * `package.json` サブパスを `exports` フィールドで公開していないパッケージで
 * `ERR_PACKAGE_PATH_NOT_EXPORTED` になるため使わない。`node_modules`
 * 直下のディレクトリ構成（`bench/ssr/node_modules/<pkg>/package.json`）を
 * 直接読むことで、`exports` フィールドの公開有無に依存せず全パッケージ
 * （スコープ付き `@lit-labs/ssr` を含む）を統一的に解決できる。
 */
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import path from "node:path";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const NODE_MODULES_DIR = path.join(__dirname, "..", "node_modules");

export function pkgVersion(pkgName) {
  const pkgJsonPath = path.join(NODE_MODULES_DIR, pkgName, "package.json");
  const pkg = JSON.parse(readFileSync(pkgJsonPath, "utf8"));
  return pkg.version;
}
