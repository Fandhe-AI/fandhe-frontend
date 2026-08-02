#!/usr/bin/env bash
# Conventional Commits 形式（`.claude/rules/conventional-commits.md`）を
# シェル正規表現で検証する commit-msg フック本体。
#
# npm 依存の commitlint は導入しない（REQ-12 の `--ignore-scripts` 方針・
# サプライチェーン対策を踏まえ、npm 経路を新たに増やさない判断）。
#
# 呼び出し元: lefthook.yml の commit-msg フック（第 1 引数にコミット
# メッセージファイルのパスが渡る lefthook の仕様に合わせる）。
#
# 許容形式:
#   <type>(<scope>)[!]: <要約>
#   例: feat(core): テキスト補間の既定エスケープを製品仕様として固定
#       feat(core)!: render() の戻り値型を変更
# type は .claude/rules/conventional-commits.md の一覧
# （feat/fix/docs/style/refactor/perf/test/build/ci/chore）に限定する。
# scope は英小文字・数字・ハイフンのみ許容する。
#
# Merge commit・Revert commit・fixup!/squash! commit は Conventional
# Commits 形式を要求せず素通りさせる（git 標準操作・作業中コミットを
# 不必要にブロックしないため）。
set -euo pipefail

msg_file="${1:-}"
if [ -z "${msg_file}" ] || [ ! -f "${msg_file}" ]; then
  echo "commit-msg-check: commit message file not found: ${msg_file}" >&2
  exit 1
fi

# コメント行（`#` 始まり）・空行を除いた最初の非空行を件名として扱う。
subject=""
while IFS= read -r line; do
  case "${line}" in
    '#'*) continue ;;
    '') continue ;;
    *) subject="${line}"; break ;;
  esac
done < "${msg_file}"

if [ -z "${subject}" ]; then
  echo "commit-msg-check: empty commit message subject" >&2
  exit 1
fi

case "${subject}" in
  Merge\ *|Revert\ *|fixup!\ *|squash!\ *)
    exit 0
    ;;
esac

pattern='^(feat|fix|docs|style|refactor|perf|test|build|ci|chore)\(([a-z0-9-]+)\)!?: .+'
if [[ "${subject}" =~ ${pattern} ]]; then
  exit 0
fi

cat >&2 <<EOF
commit-msg-check: commit message does not follow Conventional Commits format.

  subject: ${subject}

expected: <type>(<scope>)[!]: <summary>
  type  = feat|fix|docs|style|refactor|perf|test|build|ci|chore
  scope = lowercase letters, digits, hyphens (e.g. core, cli, docs-site)

example: feat(core): テキスト補間の既定エスケープを製品仕様として固定

see .claude/rules/conventional-commits.md for details.
EOF
exit 1
