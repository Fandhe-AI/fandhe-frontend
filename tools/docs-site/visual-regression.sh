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
# 撮影マトリクスの追随（イシュー #1033）:
# - イシュー #1017/#1018 で部品ページ URL が `/components/<kebab>/` から
#   `/themes/<kebab>/` へ移行し、`/components/<kebab>/` は
#   `meta refresh` による移転案内へ置き換わった。#960 時点のマトリクスが
#   `/components/button/` `/components/dialog/` `/components/pre-styled-ui/`
#   を撮っていたままだと、実際には「移転先ページ」の証跡になり
#   ラベルと内容が食い違う（meta refresh は `script-src 'none'` の下でも
#   発火するため no-JS 撮影でも移転先が写る）。#1033 でマトリクスを
#   Primitives（`/primitives/`）/ Themes（`/themes/`）の 2 層構成へ
#   追随させた。旧 `/components/...` を撮る P2/P3/P7 相当は
#   `/themes/...` へ差し替え、Primitives 代表（accordion・dialog・
#   `data-*` 表を持たない部品の代表 visually-hidden）・両索引ページを追加した。
# - 計画時点では「`/components/button/` no-JS」を撮れば meta refresh 追従後の
#   `/themes/button/` が end-to-end 証跡として写る想定だったが、実装時の
#   実測で **headless chromium の `--screenshot` は CSP
#   `script-src 'none'` 配信下で `<meta http-equiv="refresh">` ページを開く
#   と無期限にハングする**ことを確認した（同一ページを CSP 無しの通常配信
#   で開く、または同じ CSP 配信で通常ページ（非リダイレクト）を開く場合は
#   数秒で成功する。CSP + meta-refresh の組み合わせのみが再現条件）。
#   40 秒 `timeout` を掛けても exit 124 で確実にハングし、リトライで解消
#   しないため、この撮影は本スクリプトのマトリクスに含めない（1 枚のために
#   全体を無期限にブロックする fail-closed 経路を持ち込まない判断）。
#   観点 6（旧 URL 互換）の end-to-end 確認は、meta refresh・canonical・
#   robots noindex・静的 `<a>` の 4 要素を HTML 直接検証する手段（既存
#   `crates/docs-site/tests/redirects.rs` / `no_js_contract.rs`）で担保する。
#   詳細は `docs/reports/docs-site-redesign-regression-report.md` §13/§16。
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

if ! command -v ss >/dev/null 2>&1; then
  echo "environment error: ss (iproute2) not found in PATH. Required to verify that a spawned server process (not some other process) actually holds the listening socket on its assigned port before proceeding (see start_server_on_free_port below)." >&2
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

# ---- ポート選択とサーバ起動（並列実行対策）----
# 「空きポートを probe → 後で bind」という 2 段階だと、probe と実際の bind
# の間に別プロセス（同スクリプトの並列実行や無関係なプロセス）が同じポートを
# 取得できてしまい（TOCTOU）、light/dark/no-JS の 3 サーバが互いに同一ポート
# へ衝突しうる（イシュー #960 PR #1006 Bugbot 指摘）。probe と bind を分離
# せず、候補ポートへ実サーバ自身を起動させて bind 成否をそのまま判定する
# ことで、外部から見た「空き」判定と実際の予約を単一の atomic な操作にする。
# bind に失敗した python プロセスは即座に終了するため、次候補へ機械的に
# フォールバックできる。
# out_var へ結果ポートを直接代入し（`printf -v`）、PIDS へ直接 append する。
# `PORT_X="$(start_server_on_free_port ...)"` のようにコマンド置換で包むと
# 関数全体がサブシェルで実行され、内部での `PIDS+=(...)` が親シェルへ反映
# されず cleanup trap から見えなくなる（サブシェルはコピーオンライトの変数
# スコープを持ち、更新は呼び出し元に伝播しない）ため、この形は避ける。
#
# 「ポートで何かが listen している」の確認だけでは不十分（`/dev/tcp/...`
# 接続確認は他プロセス — 既にそのポートを占有している別サーバ — への
# 接続にも成功してしまう）。今まさに起動した子プロセス自身がそのポートの
# listen ソケットを保持しているかを `ss -ltnp` の pid フィールドで照合し、
# 「自分が bind できたこと」を「誰かが listen していること」から区別する
# （既存の占有者がいる状況を子プロセスの bind 成功と誤認しないため。
# イシュー #960 PR #1006 Bugbot 指摘の再発防止）。
#
# `ss` は出力幅を端末幅（TIOCGWINSZ、非 TTY の場合は `COLUMNS` 環境変数を
# 参照するフォールバックを含む）に合わせて折り返すため、パイプ経由の実行
# （非 TTY）では `users:(("...",pid=...))` フィールドが `127.0.0.1:<port>`
# と同一行に収まらず次行へ折り返されることがある。その場合 2 段の
# `grep -F` が同一行一致を前提とするため一致せず、正常に bind できた
# サーバが「起動していない」と誤判定されて kill・次候補ポートへ浪費され
# 続ける（イシュー #960 PR #1006 Bugbot 指摘）。`COLUMNS` を十分大きい値に
# 固定して `ss` へ渡し、1 ソケット 1 行に収まることを保証することで、
# 既存の「同一行一致」前提のまま折り返しを起こさせない。
port_owned_by_pid() {
  local port="$1" pid="$2"
  COLUMNS=1000 ss -ltnp 2>/dev/null | grep -F "127.0.0.1:${port} " | grep -qF "pid=${pid},"
}

start_server_on_free_port() {
  local out_var="$1" start_port="$2" log_file="$3"
  shift 3
  local port="$start_port"
  for _ in $(seq 1 50); do
    "$@" "$port" > "$log_file" 2>&1 &
    local pid=$!
    # bind 成否の判定: プロセスが生存したまま、当該ポートの listen ソケットを
    # 自分自身の pid で保持するまで待つ。プロセスが早期終了していれば
    # ポート衝突（か他の起動エラー）とみなし次のポートへ進む。python の
    # bind エラーは通常 100ms 未満で反映されるため、待機上限（2 秒）を
    # 大きく超えて粘らない。
    local ready=""
    for _ in $(seq 1 20); do
      if ! kill -0 "$pid" 2>/dev/null; then
        break
      fi
      if port_owned_by_pid "$port" "$pid"; then
        ready=1
        break
      fi
      sleep 0.1
    done
    if [ -n "$ready" ]; then
      PIDS+=("$pid")
      printf -v "$out_var" '%s' "$port"
      return 0
    fi
    # bind できなかった（またはまだ listen していない）候補は破棄する。
    kill "$pid" >/dev/null 2>&1 || true
    wait "$pid" 2>/dev/null || true
    port=$((port + 1))
  done
  echo "environment error: could not start a server starting at port $start_port (log: $log_file)" >&2
  return 1
}

# csp_server.py は引数順が `port docroot` 固定（既存の CSP_SERVER_PY 参照）。
# start_server_on_free_port は候補ポートを常に末尾へ渡すため、docroot を
# 事前束縛したラッパー関数で引数順を吸収する。
# `exec` で python3 に置き換える（単なる呼び出しにすると、このラッパー関数を
# バックグラウンド実行するサブシェルの PID が `$!` として返り、実際に
# listen ソケットを保持する python3 の PID と食い違う。start_server_on_free_port
# の `port_owned_by_pid` は `$!` の PID で `ss` を照合するため、`exec` せずに
# 子プロセスとして起動すると常時ミスマッチしフォールバックし続けてしまう）。
run_csp_server() {
  local docroot="$1" port="$2"
  exec python3 "$CSP_SERVER_PY" "$port" "$docroot"
}

start_server_on_free_port PORT_LIGHT 8931 "$LOG_DIR/server-light.log" \
  python3 -m http.server --bind 127.0.0.1 --directory "$SERVE_LIGHT"
start_server_on_free_port PORT_DARK 8941 "$LOG_DIR/server-dark.log" \
  python3 -m http.server --bind 127.0.0.1 --directory "$SERVE_DARK"
start_server_on_free_port PORT_NOJS 8951 "$LOG_DIR/server-nojs.log" \
  run_csp_server "$SERVE_LIGHT"

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

# P1: トップ（D・G・E の一次証跡、既存継続）
for w_h in "1440x900" "768x1024" "375x812"; do
  w="${w_h%x*}"; h="${w_h#*x}"
  shoot "p1-top-${w}-light" "/fandhe-frontend/" "$w" "$h" light js "$PORT_LIGHT"
  shoot "p1-top-${w}-dark" "/fandhe-frontend/" "$w" "$h" dark js "$PORT_DARK"
done

# P2: Themes 部品ページ（accordion。CSS 変数表を持つ代表、観点 5・8・9）
for w_h in "1440x900" "768x1024" "375x812"; do
  w="${w_h%x*}"; h="${w_h#*x}"
  shoot "p2-themes-accordion-${w}-light" "/fandhe-frontend/themes/accordion/" "$w" "$h" light js "$PORT_LIGHT"
  shoot "p2-themes-accordion-${w}-dark" "/fandhe-frontend/themes/accordion/" "$w" "$h" dark js "$PORT_DARK"
done

# P3: Themes の Overlay 代表（dialog。F のテーブル横スクロール証跡の継続）
for w_h in "1440x900" "375x812"; do
  w="${w_h%x*}"; h="${w_h#*x}"
  shoot "p3-themes-dialog-${w}-light" "/fandhe-frontend/themes/dialog/" "$w" "$h" light js "$PORT_LIGHT"
  shoot "p3-themes-dialog-${w}-dark" "/fandhe-frontend/themes/dialog/" "$w" "$h" dark js "$PORT_DARK"
done

# P4: Primitives 部品ページ（accordion。unstyled デモの判読性、観点 4・8・9）
for w_h in "1440x900" "768x1024" "375x812"; do
  w="${w_h%x*}"; h="${w_h#*x}"
  shoot "p4-primitives-accordion-${w}-light" "/fandhe-frontend/primitives/accordion/" "$w" "$h" light js "$PORT_LIGHT"
  shoot "p4-primitives-accordion-${w}-dark" "/fandhe-frontend/primitives/accordion/" "$w" "$h" dark js "$PORT_DARK"
done

# P5: Primitives の Overlay 代表（dialog、観点 4・8）
for w_h in "1440x900" "375x812"; do
  w="${w_h%x*}"; h="${w_h#*x}"
  shoot "p5-primitives-dialog-${w}-light" "/fandhe-frontend/primitives/dialog/" "$w" "$h" light js "$PORT_LIGHT"
  shoot "p5-primitives-dialog-${w}-dark" "/fandhe-frontend/primitives/dialog/" "$w" "$h" dark js "$PORT_DARK"
done

# P6: `data-*` 表を持たない 13 部品の代表（visually-hidden、light のみ）。
# 表が生成されない部品でもページとして破綻しないことの視覚的裏付け
# （観点 4 の「合格（導出規則の説明付き）」の裏付け証跡。ダーク変種は
# ここでは撮らず枚数バジェットを優先する。ダークモードの網羅は P2/P4/P5
# が担う）。
shoot "p6-primitives-visually-hidden-1440-light" "/fandhe-frontend/primitives/visually-hidden/" 1440 900 light js "$PORT_LIGHT"

# P7: Primitives 索引（観点 1・9）
shoot "p7-primitives-index-1440-light" "/fandhe-frontend/primitives/" 1440 900 light js "$PORT_LIGHT"
shoot "p7-primitives-index-375-light" "/fandhe-frontend/primitives/" 375 812 light js "$PORT_LIGHT"

# P8: Themes 索引（観点 1・9）
shoot "p8-themes-index-1440-light" "/fandhe-frontend/themes/" 1440 900 light js "$PORT_LIGHT"
shoot "p8-themes-index-375-light" "/fandhe-frontend/themes/" 375 812 light js "$PORT_LIGHT"

# P9: API Reference（既存継続、C・F）
shoot "p9-api-headless-ui-1440-light" "/fandhe-frontend/api/headless-ui-api/" 1440 900 light js "$PORT_LIGHT"
shoot "p9-api-headless-ui-1440-dark" "/fandhe-frontend/api/headless-ui-api/" 1440 900 dark js "$PORT_DARK"

# 既存継続分（Guides / Examples のレスポンシブ確認）は 1440 light のみへ縮約し、
# #1033 で増えた Primitives/Themes 分と合わせても枚数バジェット（40 枚 /
# 4.5MB）を超えないようにする。
shoot "p10-quickstart-1440-light" "/fandhe-frontend/getting-started/quickstart/" 1440 900 light js "$PORT_LIGHT"
shoot "p11-ssr-routing-1440-light" "/fandhe-frontend/examples/ssr-routing/" 1440 900 light js "$PORT_LIGHT"

# N1: no-JS トップ（3 幅、既存継続）
for w_h in "1440x900" "768x1024" "375x812"; do
  w="${w_h%x*}"; h="${w_h#*x}"
  shoot "n1-top-nojs-${w}" "/fandhe-frontend/" "$w" "$h" light nojs "$PORT_NOJS"
done

# N2: no-JS Primitives 部品ページ（狭幅でのサイドバー到達性、観点 3・9）
shoot "n2-primitives-accordion-nojs-375" "/fandhe-frontend/primitives/accordion/" 375 812 light nojs "$PORT_NOJS"

# N3（旧 URL の移転案内、`/components/button/`）は採らない。CSP
# `script-src 'none'` 配信下で `<meta http-equiv="refresh">` ページを開くと
# headless chromium の `--screenshot` が無期限にハングすることを実測で
# 確認した（本ファイル冒頭コメント参照）。観点 6 は HTML 直接検証と既存
# テスト（`redirects.rs` / `no_js_contract.rs`）で担保する。

echo "---"
echo "manifest: $MANIFEST"
total_bytes="$(du -cb "$SHOTS_DIR" 2>/dev/null | tail -1 | cut -f1)"
file_count="$(find "$SHOTS_DIR" -name '*.png' | wc -l)"
echo "total: ${file_count} files, ${total_bytes} bytes"
if [ "$file_count" -gt 40 ] || [ "$total_bytes" -gt 4718592 ]; then
  echo "warning: shot budget exceeded (>40 files or >4.5MB). Trim per plan §4.3 reduction order before committing." >&2
fi
