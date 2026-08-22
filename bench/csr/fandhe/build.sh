#!/usr/bin/env bash
# bench/csr/fandhe/build.sh — フレームワーク横断 CSR ベンチマーク
# （bench/PROTOCOL.md §2.2/§3）向け fandhe-frontend wasm アプリのビルド手順。
#
# 役割: bench/csr/fandhe/（独立ワークスペース、glue クレート
# fandhe-bench-csr-wasm）を wasm32-unknown-unknown へビルドし、
# wasm-bindgen CLI で bench/csr/dist/fandhe/ へ index.html + glue JS +
# .wasm + meta.json 一式を配置する。
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
# 一方、後段の wasm-opt（size optimization）は成果物の**正しさに影響しない**
# ため未導入環境では停止せず warning を出して素通りする（soft-skip）。
#
# シェル変数はすべてクォートする（.claude/rules/security.md A01/A03）。
set -euo pipefail

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
mkdir -p "${out_dir}"
wasm-bindgen --target web --out-dir "${out_dir}" --out-name fandhe_bench "${wasm_artifact}"

# --- (e) wasm-opt によるサイズ最適化（soft-skip） ---
bg_wasm="${out_dir}/fandhe_bench_bg.wasm"
if command -v wasm-opt >/dev/null 2>&1; then
  opt_tmp="$(mktemp "${out_dir}/.fandhe_bench_bg.wasm-opt.XXXXXX")"
  if wasm-opt -Os "${bg_wasm}" -o "${opt_tmp}"; then
    chmod 644 "${opt_tmp}"
    mv "${opt_tmp}" "${bg_wasm}"
  else
    rm -f "${opt_tmp}"
    echo "error: wasm-opt failed while optimizing ${bg_wasm}" >&2
    exit 1
  fi
else
  echo "warning: wasm-opt not found on PATH; skipping size optimization (output correctness unaffected, but bundle size will be larger than optimized builds)." >&2
fi

# --- (f) index.html の配置（script src プレースホルダの実ファイル名置換） ---
sed 's/__WASM_GLUE__/fandhe_bench.js/' "${script_dir}/index.html" >"${out_dir}/index.html"

# --- (g) meta.json（bench/PROTOCOL.md §1 の framework/version 表記に対応） ---
wasm_client_version="$(awk -F'"' '/^version = /{print $2; exit}' "${wasm_client_manifest}")"
if [ -z "${wasm_client_version}" ]; then
  echo "error: could not determine fandhe-frontend-wasm-client version from ${wasm_client_manifest}" >&2
  exit 1
fi
printf '{"framework":"fandhe-frontend","version":"%s"}\n' "${wasm_client_version}" >"${out_dir}/meta.json"

echo "fandhe-frontend CSR bench build complete: ${out_dir}"
