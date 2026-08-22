#!/usr/bin/env bash
# bench/csr/fandhe/build.sh — フレームワーク横断 CSR ベンチマーク
# （bench/PROTOCOL.md §2.2/§3）向け fandhe-frontend wasm アプリのビルド手順。
#
# 役割: bench/csr/fandhe/（独立ワークスペース、glue クレート
# fandhe-bench-csr-wasm）を wasm32-unknown-unknown へビルドし、
# wasm-bindgen CLI で bench/csr/dist/fandhe/ へ index.html + glue JS
# （esbuild minify 済み）+ bootstrap.js（起動コード、同じく minify 済み）+
# .wasm（wasm-opt 済み）+ meta.json 一式を配置する。
#
# ツール整合性検証（バージョン固定・fail-closed）は
# templates/app/tools/wasm/build.sh と同一方式を踏襲する
# （.claude/rules/coding-rust.md・.claude/rules/ci.md「ツール前提の明示」）。
#
# 前提（正しさに関わる前提は fail-closed。黙示的にスキップしない）:
#   - rustup ターゲット wasm32-unknown-unknown が追加済みであること
#   - wasm-bindgen-cli が PATH 上にあり、Cargo.lock が解決した
#     wasm-bindgen クレートのバージョンと完全一致すること
#     （バージョン不一致は wasm-bindgen 自体の既知の制約により実行時に
#     壊れるため、ここで停止する）
#
# 後段の wasm-opt（size optimization）も既定で fail-closed とする:
# 未導入・バージョン不一致（WASM_OPT_EXPECTED_VERSION と不一致）は
# エラー停止する。wasm-opt の有無で fandhe の配布物サイズ（payload 計測
# 条件）が実行環境ごとに変わるのは bench/PROTOCOL.md の production 相当・
# 同一条件・再現性契約に違反するため（PR #1370 codex 第 4 巡レビュー
# 指摘 P1）。明示オプトアウト（BENCH_SKIP_WASM_OPT=1）のときのみ最適化
# なしで継続し、meta.json の "wasm_opt" へ "skipped" を記録して
# payload/measure.mjs 側が計測条件の差を結果へ明示できるようにする。
#
# シェル変数はすべてクォートする（.claude/rules/security.md A01/A03）。
set -euo pipefail

# wasm-opt（binaryen）の要求バージョン pin。`wasm-opt --version` の
# 第 3 フィールド（例: "wasm-opt version 116 (version_116)" の "116"）と
# 完全一致を要求する。pin の更新手順は bench/PROTOCOL.md §5 を参照。
WASM_OPT_EXPECTED_VERSION="116"

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" >/dev/null 2>&1 && pwd)"
crate_dir="${script_dir}"
manifest="${crate_dir}/Cargo.toml"
lock="${crate_dir}/Cargo.lock"
# meta.json の version は計測対象クレート（path 依存で実際にリンクされる
# fandhe-frontend-wasm-client）の現行バージョンを報告する
wasm_client_manifest="${crate_dir}/../../../crates/wasm-client/Cargo.toml"
out_dir="${script_dir}/../dist/fandhe"
# `CARGO_TARGET_DIR` が呼び出し側で設定されている場合はそれを尊重する
# （.claude/rules/ci.md の共有 CARGO_TARGET_DIR 対策と同じ配慮）。未設定時は
# cargo の既定挙動（manifest と同階層の target/）に合わせる。
target_dir="${CARGO_TARGET_DIR:-${crate_dir}/target}"

if [ ! -f "${manifest}" ]; then
  echo "error: ${manifest} not found." >&2
  exit 1
fi

# --- (a) 前提ツールの存在チェック ---
if ! command -v rustup >/dev/null 2>&1; then
  echo "error: rustup not found on PATH. Install rustup: https://rustup.rs/" >&2
  exit 1
fi

if ! rustup target list --installed | grep -qx "wasm32-unknown-unknown"; then
  echo "error: rustup target \`wasm32-unknown-unknown\` is not installed." >&2
  echo "fix: rustup target add wasm32-unknown-unknown" >&2
  exit 1
fi

if ! command -v wasm-bindgen >/dev/null 2>&1; then
  echo "error: wasm-bindgen-cli not found on PATH." >&2
  echo "fix: cargo install wasm-bindgen-cli --version <version-matching-Cargo.lock> --locked" >&2
  exit 1
fi

# --- (b) wasm-bindgen クレートのバージョンと CLI のバージョンの完全一致検証 ---
# Cargo.lock から動的に読むため、バージョン pin 箇所を新設しない
# （templates/app/tools/wasm/build.sh と同一方式）。Cargo.lock は通常
# 本ディレクトリにコミット済みだが、削除された場合の自己修復として
# 新規生成した lock は「今インストールされている CLI と同一バージョン」へ
# 明示的に pin する（cargo が crates.io 最新 0.2.x を解決してしまうと
# 意図せずバージョン不一致で fail するのを防ぐ、templates/app/tools/wasm/
# build.sh と同一方式）。
regenerated_lock=0
if [ ! -f "${lock}" ]; then
  cargo generate-lockfile --manifest-path "${manifest}"
  regenerated_lock=1
fi

if [ "${regenerated_lock}" -eq 1 ]; then
  installed_cli_version="$(wasm-bindgen --version | awk '{print $2}')"
  if [ -z "${installed_cli_version}" ]; then
    echo "error: could not determine the installed wasm-bindgen-cli version from \`wasm-bindgen --version\`." >&2
    exit 1
  fi
  if ! cargo update --manifest-path "${manifest}" -p wasm-bindgen --precise "${installed_cli_version}"; then
    echo "error: failed to pin the freshly generated ${lock} to wasm-bindgen ${installed_cli_version} (the installed wasm-bindgen-cli version)." >&2
    exit 1
  fi
fi

expected_version="$(awk '
  /^\[\[package\]\]$/ { in_pkg = 1; name = ""; next }
  in_pkg && /^name = "wasm-bindgen"$/ { name = "wasm-bindgen"; next }
  in_pkg && name == "wasm-bindgen" && /^version = / {
    gsub(/^version = "|"$/, "");
    print;
    exit
  }
  /^\[\[package\]\]$/ { next }
' "${lock}")"

if [ -z "${expected_version}" ]; then
  echo "error: could not determine the required wasm-bindgen-cli version from ${lock}" >&2
  echo "fix: verify Cargo.lock contains a [[package]] entry named \"wasm-bindgen\"" >&2
  exit 1
fi

installed_version="$(wasm-bindgen --version | awk '{print $2}')"

if [ "${installed_version}" != "${expected_version}" ]; then
  echo "error: wasm-bindgen-cli version mismatch: Cargo.lock resolves wasm-bindgen ${expected_version}, but \`wasm-bindgen --version\` reports ${installed_version}." >&2
  echo "fix: cargo install wasm-bindgen-cli --version ${expected_version} --locked" >&2
  exit 1
fi

# --- (c) wasm32 ビルド ---
cargo build \
  --manifest-path "${manifest}" \
  --target wasm32-unknown-unknown \
  --release

wasm_artifact="${target_dir}/wasm32-unknown-unknown/release/fandhe_bench_csr_wasm.wasm"
if [ ! -f "${wasm_artifact}" ]; then
  echo "error: expected build artifact not found: ${wasm_artifact}" >&2
  exit 1
fi

# --- (d) wasm-bindgen 後処理 ---
# 出力先を生成前に清掃し、旧成果物（過去のビルドが残した別名ファイル等）が
# payload 計測へ混入しないようにする（PR #1370 codex 第 4 巡レビュー指摘
# P1）。削除は固定パス out_dir のみ・glob 不使用で行い、`:?` ガードで
# 変数未定義時に空文字へ展開されて広域削除になる事故を防ぐ
# （.claude/rules/security.md の広域削除防止）。
rm -rf "${out_dir:?}"
mkdir -p "${out_dir}"
wasm-bindgen --target web --out-dir "${out_dir}" --out-name fandhe_bench "${wasm_artifact}"

# --- (e) wasm-opt によるサイズ最適化（既定 fail-closed、明示オプトアウトのみ skip） ---
# meta.json へ記録する wasm-opt 適用状態。適用時は検出したバージョン文字列
# （例: "116"）、BENCH_SKIP_WASM_OPT=1 のときのみ "skipped"。
bg_wasm="${out_dir}/fandhe_bench_bg.wasm"
if [ "${BENCH_SKIP_WASM_OPT:-0}" = "1" ]; then
  echo "warning: BENCH_SKIP_WASM_OPT=1 — skipping wasm-opt size optimization. The measured fandhe payload will NOT be production-equivalent (bench/PROTOCOL.md §2.3); meta.json records wasm_opt=skipped so payload/measure.mjs flags the result." >&2
  wasm_opt_meta="skipped"
else
  if ! command -v wasm-opt >/dev/null 2>&1; then
    echo "error: wasm-opt (binaryen) not found on PATH. The payload benchmark requires wasm-opt ${WASM_OPT_EXPECTED_VERSION} so the fandhe artifact size is measured under the same production-equivalent conditions on every machine (bench/PROTOCOL.md §2.3/§4)." >&2
    echo "fix: install binaryen ${WASM_OPT_EXPECTED_VERSION} (e.g. \`cargo install wasm-opt --version 0.${WASM_OPT_EXPECTED_VERSION}.1 --locked\`, or your distro's binaryen package pinned to ${WASM_OPT_EXPECTED_VERSION}), or set BENCH_SKIP_WASM_OPT=1 for an explicitly non-optimized build (recorded in meta.json)." >&2
    exit 1
  fi
  installed_wasm_opt_version="$(wasm-opt --version | awk '{print $3}')"
  if [ -z "${installed_wasm_opt_version}" ]; then
    echo "error: could not determine the installed wasm-opt version from \`wasm-opt --version\`." >&2
    exit 1
  fi
  if [ "${installed_wasm_opt_version}" != "${WASM_OPT_EXPECTED_VERSION}" ]; then
    echo "error: wasm-opt version mismatch: this harness pins wasm-opt ${WASM_OPT_EXPECTED_VERSION}, but \`wasm-opt --version\` reports ${installed_wasm_opt_version}. A version drift changes the optimized artifact size and breaks cross-run comparability." >&2
    echo "fix: install wasm-opt ${WASM_OPT_EXPECTED_VERSION}, or update the WASM_OPT_EXPECTED_VERSION pin in this script following bench/PROTOCOL.md §5 (update the pin, then re-measure ALL frameworks in the same run)." >&2
    exit 1
  fi
  opt_tmp="$(mktemp "${out_dir}/.fandhe_bench_bg.wasm-opt.XXXXXX")"
  if wasm-opt -Os "${bg_wasm}" -o "${opt_tmp}"; then
    chmod 644 "${opt_tmp}"
    mv "${opt_tmp}" "${bg_wasm}"
  else
    rm -f "${opt_tmp}"
    echo "error: wasm-opt failed while optimizing ${bg_wasm}" >&2
    exit 1
  fi
  wasm_opt_meta="${installed_wasm_opt_version}"
fi

# --- (f) JS の minify と index.html / bootstrap.js の配置 ---
# 他 6 フレームワークは bench/csr/build.mjs が esbuild minify:true で
# bundle.js を生成し、その minify 済み JS が実行時間・payload の両計測に
# 使われる。fandhe だけ非 minify の JS（wasm-bindgen glue + 起動コード）を
# 配ると同一条件比較（bench/PROTOCOL.md §2.3/§4）が崩れるため、
# wasm-bindgen glue（fandhe_bench.js）と bootstrap.js の双方へ同じ esbuild で
# minify（--format=esm、バンドルなしの transform のみ）を適用する。
# バンドルしないため import 指定子（bootstrap.js → ./fandhe_bench.js、
# glue 内の import.meta.url 経由の .wasm 解決）はそのまま保存される。
# esbuild は bench/csr/node_modules（npm ci --ignore-scripts 導入済み）の
# ものを使い、未導入は fail-closed で停止する（.claude/rules/ci.md
# 「ツール前提の明示」）。
esbuild_bin="${script_dir}/../node_modules/.bin/esbuild"
if [ ! -x "${esbuild_bin}" ]; then
  echo "error: esbuild not found at ${esbuild_bin}. The fandhe glue/bootstrap JS must be minified with the same esbuild the other frameworks use (bench/PROTOCOL.md §2.3)." >&2
  echo "fix: cd \"${script_dir}/..\" && npm ci --ignore-scripts" >&2
  exit 1
fi

# 一時ファイルは .js サフィックス必須（esbuild は入力の loader を拡張子で
# 推論するため。GNU mktemp の --suffix を使う）。
glue_js="${out_dir}/fandhe_bench.js"
glue_min_tmp="$(mktemp --suffix=.js "${out_dir}/.fandhe_bench.min.XXXXXX")"
if "${esbuild_bin}" "${glue_js}" --minify --format=esm --outfile="${glue_min_tmp}" --allow-overwrite --log-level=warning; then
  chmod 644 "${glue_min_tmp}"
  mv "${glue_min_tmp}" "${glue_js}"
else
  rm -f "${glue_min_tmp}"
  echo "error: esbuild failed while minifying ${glue_js}" >&2
  exit 1
fi

# bootstrap.js: __WASM_GLUE__ プレースホルダを wasm-bindgen --out-name
# （fandhe_bench.js）へ置換してから minify して配置する。
bootstrap_tmp="$(mktemp --suffix=.js "${out_dir}/.bootstrap.XXXXXX")"
sed 's/__WASM_GLUE__/fandhe_bench.js/' "${script_dir}/bootstrap.js" >"${bootstrap_tmp}"
if "${esbuild_bin}" "${bootstrap_tmp}" --minify --format=esm --outfile="${out_dir}/bootstrap.js" --log-level=warning; then
  chmod 644 "${out_dir}/bootstrap.js"
  rm -f "${bootstrap_tmp}"
else
  rm -f "${bootstrap_tmp}"
  echo "error: esbuild failed while minifying bootstrap.js" >&2
  exit 1
fi

# index.html は起動コードを持たない共通骨格のみ（bootstrap.js を参照）の
# ため、置換なしでそのままコピーする。
cp "${script_dir}/index.html" "${out_dir}/index.html"

# --- (g) meta.json（bench/PROTOCOL.md §1 の framework/version 表記に対応） ---
# "wasm_opt" は payload/measure.mjs が読む契約フィールド（§2.3）:
# 適用時はバージョン文字列、BENCH_SKIP_WASM_OPT=1 のときのみ "skipped"。
wasm_client_version="$(awk -F'"' '/^version = /{print $2; exit}' "${wasm_client_manifest}")"
if [ -z "${wasm_client_version}" ]; then
  echo "error: could not determine fandhe-frontend-wasm-client version from ${wasm_client_manifest}" >&2
  exit 1
fi
printf '{"framework":"fandhe-frontend","version":"%s","wasm_opt":"%s"}\n' "${wasm_client_version}" "${wasm_opt_meta}" >"${out_dir}/meta.json"

echo "fandhe-frontend CSR bench build complete: ${out_dir}"
