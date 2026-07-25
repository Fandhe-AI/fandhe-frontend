#!/usr/bin/env bash
# docs サイト（crates/docs-site）の刷新後の見た目を実ブラウザで撮影する
# ビジュアル回帰スクリプト（イシュー #960）。
#
# 呼び出し元:
# - ローカル開発者・test-runner が手動実行し、`docs/reports/assets/docs-site-960/`
#   へコピーする証跡画像を生成する（CI ジョブ化はしない。chromium 常設を
#   self-hosted runner に前提できないため、`docs/ci/ci-runner-requirements.md`
#   の未解決要件に依存する CI 化は見送っている）。
#
# 何をするか:
# 1. `cargo run -p fandhe-frontend-docs-site -- --out <dir>` で実サイトを
#    ビルドする（内蔵 linkcheck が fail-closed のため、成功が全ページ生成の
#    保証になる）。
# 2. ビルド出力を複製し、全 `*.html` の `<html lang="ja">` を
#    `<html lang="ja" data-theme="dark">` へ置換したダーク変種を用意する
#    （`crate::script` のテーマトグルが設定するのと同一の属性経路。
#    `prefers-color-scheme` エミュレーションは chromium CLI に無いため
#    採用しない。両者の対応関係は証跡ページに明記する）。
# 3. `Content-Security-Policy: script-src 'none'` を返す簡易 HTTP サーバで
#    ライト変種を配信し、JS 無効環境相当の描画を撮影できるようにする
#    （`--blink-settings=scriptEnabled=false` は headless で無音失敗するため
#    使わない、イシュー #960 計画 §2 実測 3）。
# 4. 3 通りの配信（ライト / ダーク / no-JS）× 撮影マトリクスで
#    `chromium --headless --screenshot=...` を実行し、1 枚ごとにファイル
#    存在・サイズ > 0 を検証する（chromium は撮影失敗時も 0 終了し得るため、
#    無音失敗を成功と誤記録しない）。
#
# セキュリティ考慮（security.md / OWASP Top 10）:
# - 配信サーバは `127.0.0.1` バインドのみ（`0.0.0.0` にしない、A01）。
# - 出力先は絶対パスかつパス要素にドット始まりディレクトリを含まないことを
#   検証する（worktree 配下 `.claude/...` へ撮影すると snap の AppArmor に
#   より chromium が無音で書き込み失敗するため、イシュー #960 計画 §2.1）。
# - `rm -rf` は `${VAR:?}` ガード付きで変数未設定時の暴発を防ぐ。
# - manifest には `$HOME`（ユーザー名を含む絶対パス）を残さず、出力ディレクトリ
#   相対パスのみを記録する。
set -euo pipefail

# ---- 前提ツールの存在チェック（fail-closed、ci.md のツール前提明示方針と同型）----

CHROMIUM_BIN=""
for candidate in chromium chromium-browser; do
  if command -v "$candidate" >/dev/null 2>&1; then
    CHROMIUM_BIN="$candidate"
    break
  fi
done
if [ -z "$CHROMIUM_BIN" ]; then
  echo "environment error: chromium (or chromium-browser) not found in PATH. Install a Chromium build capable of --headless --screenshot before running this script." >&2
  exit 1
fi

if ! command -v python3 >/dev/null 2>&1; then
  echo "environment error: python3 not found in PATH. Required to serve the built dist directories (including the no-JS CSP variant)." >&2
  exit 1
fi

if ! command -v cargo >/dev/null 2>&1; then
  echo "environment error: cargo not found in PATH. Required to build the docs site via 'cargo run -p fandhe-frontend-docs-site'." >&2
  exit 1
fi

# ---- 出力先の決定と fail-closed 検証 ----
# イシュー本文の `_/shots/` はこのスクリプトが動く典型的な worktree
# （`.claude/worktrees/<name>/`）配下で解決すると `.claude` を含み、snap の
# AppArmor により chromium が書き込めない（無音失敗）。絶対パス・非ドット
# パス要素の既定値へ固定し、違反時は明示エラーで停止する。
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
TIMESTAMP="$(date +%Y%m%d-%H%M%S)"
OUT_DIR="${DOCS_SITE_SHOTS_DIR:-$HOME/fandhe-docs-site-visual/$TIMESTAMP}"

case "$OUT_DIR" in
  /*) ;;
  *)
    echo "environment error: DOCS_SITE_SHOTS_DIR (or the default) must be an absolute path, got: $OUT_DIR" >&2
    exit 1
    ;;
esac

IFS='/' read -r -a out_dir_parts <<< "$OUT_DIR"
for part in "${out_dir_parts[@]}"; do
  case "$part" in
    .*)
      if [ -n "$part" ]; then
        echo "environment error: output path must not contain a dot-prefixed component (snap AppArmor silently blocks chromium screenshots under e.g. .claude/...): $OUT_DIR" >&2
        exit 1
      fi
      ;;
  esac
done

mkdir -p "$OUT_DIR"
echo "output directory: $OUT_DIR"

SERVE_LIGHT="$OUT_DIR/serve-light"
SERVE_DARK="$OUT_DIR/serve-dark"
SHOTS_DIR="$OUT_DIR/shots"
LOG_DIR="$OUT_DIR/logs"
mkdir -p "$SERVE_LIGHT" "$SERVE_DARK" "$SHOTS_DIR" "$LOG_DIR"

MANIFEST="$OUT_DIR/manifest.tsv"
printf 'file\turl\twidth\theight\ttheme\tjs\tbytes\tsha256\n' > "$MANIFEST"

# ---- ポート選択（並列実行対策。既定から空きを探す）----
find_free_port() {
  local start="$1"
  local port="$start"
  for _ in $(seq 1 50); do
    if ! (exec 3<>"/dev/tcp/127.0.0.1/$port") 2>/dev/null; then
      echo "$port"
      return 0
    fi
    exec 3>&- 2>/dev/null || true
    port=$((port + 1))
  done
  echo "environment error: could not find a free localhost port starting at $start" >&2
  return 1
}

PORT_LIGHT="$(find_free_port 8931)"
PORT_DARK="$(find_free_port 8941)"
PORT_NOJS="$(find_free_port 8951)"

# ---- ビルド（ライト版）----
echo "building docs site (light) -> $SERVE_LIGHT/fandhe-frontend"
( cd "$REPO_ROOT" && cargo run --locked -p fandhe-frontend-docs-site -- --out "$SERVE_LIGHT/fandhe-frontend" ) \
  > "$LOG_DIR/build-light.log" 2>&1
echo "build (light) ok"

# ---- ダーク変種の生成 ----
# `<html lang="ja">` を `<html lang="ja" data-theme="dark">` へ置換する。
# `find -exec sed -i` は非マッチでも成功終了するため、置換前後の件数一致を
# fail-closed に検証する（不一致のままだとライト画像がダーク証跡として
# 混入する。chromium の無音失敗と同じ失敗クラス）。
cp -r "$SERVE_LIGHT/fandhe-frontend" "$SERVE_DARK/fandhe-frontend"
html_count_before="$(find "$SERVE_DARK/fandhe-frontend" -name '*.html' | wc -l)"
find "$SERVE_DARK/fandhe-frontend" -name '*.html' -print0 \
  | xargs -0 sed -i 's/<html lang="ja">/<html lang="ja" data-theme="dark">/'
html_count_after_dark="$(grep -rl 'data-theme="dark"' "$SERVE_DARK/fandhe-frontend" --include='*.html' | wc -l)"
if [ "$html_count_before" -ne "$html_count_after_dark" ]; then
  echo "environment error: dark variant replacement mismatch (html files: $html_count_before, data-theme=dark files: $html_count_after_dark). Refusing to proceed with a possibly-mixed evidence set." >&2
  exit 1
fi
echo "dark variant ok ($html_count_after_dark/$html_count_before pages)"

# ---- サーバ起動（127.0.0.1 のみ、CSP 付き no-JS サーバは Python で自作）----

CSP_SERVER_PY="$OUT_DIR/csp_server.py"
cat > "$CSP_SERVER_PY" <<'PYEOF'
"""JS 無効環境相当の描画を撮影するための最小 HTTP サーバ。

すべてのレスポンスへ `Content-Security-Policy: script-src 'none'` を付与し、
`assets/site.js`（外部 <script defer> のみ）を実行不能にする。
`--blink-settings=scriptEnabled=false` は headless chromium で撮影自体が
無音失敗するため使わず（イシュー #960 計画 §2 実測 3）、CSP による無効化を
採用する。127.0.0.1 のみへバインドし外部公開しない（security.md A01）。
"""
import http.server
import sys


class CspHandler(http.server.SimpleHTTPRequestHandler):
    def end_headers(self):
        self.send_header("Content-Security-Policy", "script-src 'none'")
        super().end_headers()

    def log_message(self, format, *args):
        pass


if __name__ == "__main__":
    port = int(sys.argv[1])
    docroot = sys.argv[2]
    handler = lambda *args, **kwargs: CspHandler(*args, directory=docroot, **kwargs)
    server = http.server.ThreadingHTTPServer(("127.0.0.1", port), handler)
    server.serve_forever()
PYEOF

PIDS=()
cleanup() {
  for pid in "${PIDS[@]:-}"; do
    kill "$pid" >/dev/null 2>&1 || true
  done
}
trap cleanup EXIT

python3 -m http.server "$PORT_LIGHT" --bind 127.0.0.1 --directory "$SERVE_LIGHT" \
  > "$LOG_DIR/server-light.log" 2>&1 &
PIDS+=("$!")
python3 -m http.server "$PORT_DARK" --bind 127.0.0.1 --directory "$SERVE_DARK" \
  > "$LOG_DIR/server-dark.log" 2>&1 &
PIDS+=("$!")
python3 "$CSP_SERVER_PY" "$PORT_NOJS" "$SERVE_LIGHT" \
  > "$LOG_DIR/server-nojs.log" 2>&1 &
PIDS+=("$!")

# サーバ起動待ち（固定 sleep は避け、ポート待受を確認する）。
wait_for_port() {
  local port="$1"
  for _ in $(seq 1 50); do
    if (exec 3<>"/dev/tcp/127.0.0.1/$port") 2>/dev/null; then
      exec 3>&- 2>/dev/null || true
      return 0
    fi
    sleep 0.1
  done
  echo "environment error: server on port $port did not become ready" >&2
  return 1
}
wait_for_port "$PORT_LIGHT"
wait_for_port "$PORT_DARK"
wait_for_port "$PORT_NOJS"

# ---- 撮影 ----
# 1 枚ごとにファイル存在・サイズ > 0 を検証する（chromium はスクリーンショット
# 失敗時も 0 終了し得るため、無音失敗を成功と誤記録しない）。
shoot() {
  local name="$1" path="$2" width="$3" height="$4" theme="$5" js="$6" port="$7"
  local url="http://127.0.0.1:${port}${path}"
  local out_file="$SHOTS_DIR/${name}.png"

  "$CHROMIUM_BIN" \
    --headless --disable-gpu --no-sandbox \
    --window-size="${width},${height}" \
    --screenshot="$out_file" \
    "$url" \
    > "$LOG_DIR/${name}.chromium.log" 2>&1 || true

  if [ ! -s "$out_file" ]; then
    echo "environment error: screenshot missing or empty for $name (url=$url). chromium can exit 0 on silent failure; treat this as fail-closed." >&2
    exit 1
  fi

  local bytes
  bytes="$(stat -c '%s' "$out_file" 2>/dev/null || stat -f '%z' "$out_file")"
  local sha
  sha="$(sha256sum "$out_file" | cut -d' ' -f1)"
  printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' \
    "shots/${name}.png" "$url" "$width" "$height" "$theme" "$js" "$bytes" "$sha" \
    >> "$MANIFEST"
  echo "shot ok: $name ($bytes bytes)"
}

# P1: トップ（D・G・E の一次証跡）
for w_h in "1440x900" "768x1024" "375x812"; do
  w="${w_h%x*}"; h="${w_h#*x}"
  shoot "p1-top-${w}-light" "/fandhe-frontend/" "$w" "$h" light js "$PORT_LIGHT"
  shoot "p1-top-${w}-dark" "/fandhe-frontend/" "$w" "$h" dark js "$PORT_DARK"
done

# P2: 部品ページ（Forms 代表、button）
for w_h in "1440x900" "768x1024" "375x812"; do
  w="${w_h%x*}"; h="${w_h#*x}"
  shoot "p2-button-${w}-light" "/fandhe-frontend/components/button/" "$w" "$h" light js "$PORT_LIGHT"
  shoot "p2-button-${w}-dark" "/fandhe-frontend/components/button/" "$w" "$h" dark js "$PORT_DARK"
done

# P3: 部品ページ（Overlay 代表、dialog。F のテーブル横スクロール証跡）
for w_h in "1440x900" "375x812"; do
  w="${w_h%x*}"; h="${w_h#*x}"
  shoot "p3-dialog-${w}-light" "/fandhe-frontend/components/dialog/" "$w" "$h" light js "$PORT_LIGHT"
  shoot "p3-dialog-${w}-dark" "/fandhe-frontend/components/dialog/" "$w" "$h" dark js "$PORT_DARK"
done

# P4: API Reference（C・F）
for w_h in "1440x900" "375x812"; do
  w="${w_h%x*}"; h="${w_h#*x}"
  shoot "p4-api-headless-ui-${w}-light" "/fandhe-frontend/api/headless-ui-api/" "$w" "$h" light js "$PORT_LIGHT"
  shoot "p4-api-headless-ui-${w}-dark" "/fandhe-frontend/api/headless-ui-api/" "$w" "$h" dark js "$PORT_DARK"
done

# P5: Guides（レスポンシブ確認、1440/375 のみ）
shoot "p5-quickstart-1440-light" "/fandhe-frontend/getting-started/quickstart/" 1440 900 light js "$PORT_LIGHT"
shoot "p5-quickstart-375-light" "/fandhe-frontend/getting-started/quickstart/" 375 812 light js "$PORT_LIGHT"

# P6: Examples（レスポンシブ確認、1440/375 のみ）
shoot "p6-ssr-routing-1440-light" "/fandhe-frontend/examples/ssr-routing/" 1440 900 light js "$PORT_LIGHT"
shoot "p6-ssr-routing-375-light" "/fandhe-frontend/examples/ssr-routing/" 375 812 light js "$PORT_LIGHT"

# P7: コンポーネント索引（A の一次証跡）
shoot "p7-components-index-1440-light" "/fandhe-frontend/components/pre-styled-ui/" 1440 900 light js "$PORT_LIGHT"
shoot "p7-components-index-375-light" "/fandhe-frontend/components/pre-styled-ui/" 375 812 light js "$PORT_LIGHT"

# N1: no-JS トップ（3 幅）
for w_h in "1440x900" "768x1024" "375x812"; do
  w="${w_h%x*}"; h="${w_h#*x}"
  shoot "n1-top-nojs-${w}" "/fandhe-frontend/" "$w" "$h" light nojs "$PORT_NOJS"
done

# N2: no-JS 部品ページ（折りたたみサイドバーが :checked のみで辿れるか）
shoot "n2-button-nojs-375" "/fandhe-frontend/components/button/" 375 812 light nojs "$PORT_NOJS"

echo "---"
echo "manifest: $MANIFEST"
total_bytes="$(du -cb "$SHOTS_DIR" 2>/dev/null | tail -1 | cut -f1)"
file_count="$(find "$SHOTS_DIR" -name '*.png' | wc -l)"
echo "total: ${file_count} files, ${total_bytes} bytes"
if [ "$file_count" -gt 40 ] || [ "$total_bytes" -gt 4718592 ]; then
  echo "warning: shot budget exceeded (>40 files or >4.5MB). Trim per plan §4.3 reduction order before committing." >&2
fi
