#!/usr/bin/env python3
"""check_static_only.py

役割: `tools/npm-asset-build/install.sh`（--ignore-scripts 強制の入口ラッパー）の
後段ゲート。REQ-12（docs/spec/04-requirements.md）受け入れ基準 2「取り込んだ
パッケージに実行可能コードを含まないこと」を、インストール後の node_modules を
走査して機械的に検証する。

背景: PoC-6（docs/spec/03-poc/npm-compat-feasibility/README.md）が示すとおり
`--ignore-scripts` は preinstall/install/postinstall の暗黙実行のみを防ぎ、
パッケージ内の明示的な require() やビルドプラグイン実行までは防げない。
本スクリプトはその隙間を埋める fail-closed な allowlist 方式の検証器であり、
判定ルールは #122（TASK-12.2a）の設計ドキュメント docs/npm-static-asset-rules.md
（本実装時点で未マージの場合は当該 PR #223 の内容を契約として使用）に従う。

呼び出し文脈: install.sh の実行後、CI から呼び出されることを想定する
（CI 統合自体は #124 / TASK-12.2c のスコープで後日接続される。本スクリプトは
単体で node_modules を検証できる状態までを提供する）。

セキュリティ不変条件:
  - allowlist（許可拡張子・許可フィールド）方式の既定拒否。denylist ではない
  - symlink は辿らず、それ自体を違反として報告する（パストラバーサル対策）
  - 検査対象ファイルの内容は shebang 判定（先頭 2 バイト）と SVG のテキスト
    検査のみに限定し、実行・eval・動的 import は一切行わない
  - allowlist.toml のパース失敗・不正エントリは fail-closed（exit 2）で拒否する

終了コード契約:
  0 = 全合格 / 1 = 違反あり / 2 = 実行エラー（パス不在・allowlist 不正等）
"""

from __future__ import annotations

import argparse
import json
import re
import stat
import sys
from pathlib import Path
from typing import Iterable, Iterator

# JS 実行エントリとみなす拡張子（main/module/browser/exports が指す先の判定に使う）。
JS_EXEC_EXTS = {".js", ".mjs", ".cjs", ".node"}

# node_modules 配下のファイルに許可する拡張子（静的アセット限定・既定拒否の allowlist）。
ALLOWED_EXTS = {
    ".css",
    ".woff",
    ".woff2",
    ".ttf",
    ".otf",
    ".eot",
    ".svg",
    ".png",
    ".jpg",
    ".jpeg",
    ".gif",
    ".webp",
    ".avif",
    ".ico",
    ".json",
    ".md",
    ".txt",
}

# 拡張子を持たないが許可するファイル名の前方一致プレフィックス（大文字小文字非区別）。
ALLOWED_NOEXT_PREFIXES = ("license", "notice", "readme")

# package.json の lifecycle スクリプト（存在自体が R1-scripts 違反）。
LIFECYCLE_SCRIPT_KEYS = {
    "preinstall",
    "install",
    "postinstall",
    "prepare",
    "prepublish",
    "prepublishOnly",
    "prepack",
    "postpack",
    "preuninstall",
    "postuninstall",
}

# allowlist.toml の rule フィールドとして許容する Rule ID（未知 ID は fail-closed）。
VALID_RULE_IDS = {
    "R0-symlink",
    "R1-bin",
    "R1-entry",
    "R1-scripts",
    "R2-ext",
    "R3-shebang",
    "R3-execbit",
    "R3-svg-script",
}

# node_modules 直下でパッケージ列挙の対象外とする npm 実装詳細ディレクトリ・ドットファイル。
IGNORED_TOPLEVEL_PREFIX = "."


class CheckError(Exception):
    """実行エラー（exit 2 系）を表す。パス不在・allowlist 不正等、検証対象自体が
    信頼できない状態を示し、fail-closed に倒すために使う。"""


def parse_args(argv: list[str]) -> argparse.Namespace:
    """CLI 引数を解析する。呼び出し元は install.sh 後段の CI ジョブ等を想定。"""
    parser = argparse.ArgumentParser(
        description="Verify that node_modules contains no executable code (REQ-12 acceptance criterion 2)."
    )
    group = parser.add_mutually_exclusive_group(required=True)
    group.add_argument("--node-modules", help="Path to a node_modules directory to scan.")
    group.add_argument("--dir", help="Project directory; <dir>/node_modules is derived.")
    parser.add_argument(
        "--allowlist",
        help="Path to allowlist.toml (per-package/per-rule exemptions). Not searched implicitly.",
    )
    return parser.parse_args(argv)


def resolve_node_modules(args: argparse.Namespace) -> Path:
    """検査対象の node_modules ルートを決定する。存在しない・ディレクトリでない場合は
    fail-closed（CheckError → exit 2）として扱う。"""
    if args.node_modules:
        candidate = Path(args.node_modules)
    else:
        candidate = Path(args.dir) / "node_modules"

    if not candidate.exists():
        raise CheckError(f"node_modules path not found: {candidate}")
    if not candidate.is_dir():
        raise CheckError(f"node_modules path is not a directory: {candidate}")
    return candidate


def load_allowlist(path: str | None) -> dict[tuple[str, str], str]:
    """allowlist.toml を読み込み、(package, rule) -> reason の免除表を返す。

    契約（docs/npm-static-asset-rules.md §4.3）: reason 欠落・空、未知 rule ID、
    ワイルドカード的指定はすべてパースエラーとして CheckError を送出する
    （呼び出し元 main() で exit 2 に変換）。ここでの安全側判断は「免除機構自体の
    設定ミスは、免除なしより危険（サイレントな野放図拡大）」という前提に基づく。
    """
    if path is None:
        return {}

    try:
        import tomllib
    except ImportError as exc:  # Python 3.10 以下には tomllib が存在しない
        raise CheckError(
            "--allowlist requires Python 3.11+ (tomllib not available in this interpreter)"
        ) from exc

    allowlist_path = Path(path)
    try:
        with open(allowlist_path, "rb") as f:
            data = tomllib.load(f)
    except OSError as exc:
        raise CheckError(f"failed to read allowlist file {allowlist_path}: {exc}") from exc
    except Exception as exc:  # tomllib.TOMLDecodeError 等
        raise CheckError(f"failed to parse allowlist file {allowlist_path}: {exc}") from exc

    exemptions: dict[tuple[str, str], str] = {}
    for entry in data.get("exempt", []):
        if not isinstance(entry, dict):
            raise CheckError(f"invalid [[exempt]] entry (not a table): {entry!r}")

        package = entry.get("package")
        rule = entry.get("rule")
        reason = entry.get("reason")

        if not isinstance(package, str) or not package or "*" in package or "?" in package:
            raise CheckError(f"exempt entry has invalid or wildcard 'package': {entry!r}")
        if rule not in VALID_RULE_IDS:
            raise CheckError(f"exempt entry has unknown 'rule': {rule!r}")
        if not isinstance(reason, str) or not reason.strip():
            raise CheckError(
                f"exempt entry missing non-empty 'reason' for package={package} rule={rule}"
            )

        exemptions[(package, rule)] = reason.strip()

    return exemptions


def _is_js_entry(value: object) -> bool:
    """package.json の main/module/browser/exports が指す値が JS 実行エントリを
    指しているかを再帰的に判定する（exports のネスト構造にも対応）。

    拡張子なしのパスは Node.js の解決アルゴリズムにより暗黙に .js とみなされ得る
    ため、安全側に倒して JS エントリ扱いとする（見逃し回避を優先）。
    """
    if isinstance(value, str):
        base = value.rsplit("/", 1)[-1]
        if "." not in base:
            return True
        ext = "." + base.rsplit(".", 1)[-1].lower()
        return ext in JS_EXEC_EXTS
    if isinstance(value, dict):
        return any(_is_js_entry(v) for v in value.values())
    if isinstance(value, list):
        return any(_is_js_entry(v) for v in value)
    return False


def _check_package_json(pkg_json_path: Path) -> list[tuple[str, str, str]]:
    """package.json 由来のルール（R1-bin / R1-entry / R1-scripts）を検査する。
    戻り値は (rule_id, file, reason) のリスト。file は常に "package.json"。"""
    violations: list[tuple[str, str, str]] = []

    if not pkg_json_path.is_file() or pkg_json_path.is_symlink():
        return violations

    try:
        data = json.loads(pkg_json_path.read_text(encoding="utf-8", errors="replace"))
    except (OSError, json.JSONDecodeError):
        # package.json が壊れている/読めない場合はこのルール群を判定不能として
        # スキップする（R2/R3 のファイル走査は別途継続されるため見逃しは限定的）。
        return violations

    if not isinstance(data, dict):
        return violations

    if "bin" in data:
        violations.append(
            ("R1-bin", "package.json", 'package.json declares a "bin" entry (executable command)')
        )

    for field in ("main", "module", "browser", "exports"):
        if field in data and _is_js_entry(data[field]):
            violations.append(
                ("R1-entry", "package.json", f'"{field}" field resolves to a JS execution entry')
            )

    scripts = data.get("scripts")
    if isinstance(scripts, dict):
        hits = sorted(k for k in scripts if k in LIFECYCLE_SCRIPT_KEYS)
        if hits:
            violations.append(
                ("R1-scripts", "package.json", f"lifecycle script(s) present: {', '.join(hits)}")
            )

    return violations


def _check_extension(filename: str) -> str | None:
    """R2-ext: allowlist にない拡張子・拡張子なしファイルを違反として返す
    （合格時は None）。既定拒否（許可リスト方式）。"""
    lower = filename.lower()

    if "." not in lower:
        if lower.startswith(ALLOWED_NOEXT_PREFIXES):
            return None
        return "file has no extension and is not an allowed LICENSE/NOTICE/README file"

    # 複合拡張子（.min.js / .d.ts）は単純拡張子判定より先に明示拒否する。
    if lower.endswith(".min.js"):
        return 'disallowed extension ".min.js" (minified JS execution code)'
    if lower.endswith(".d.ts"):
        return 'disallowed extension ".d.ts" (TypeScript declaration source)'

    ext = "." + lower.rsplit(".", 1)[-1]
    if ext in ALLOWED_EXTS:
        return None
    return f'disallowed extension "{ext}"'


_SVG_SCRIPT_RE = re.compile(r"<script[\s>/]", re.IGNORECASE)
_SVG_EVENT_ATTR_RE = re.compile(r"\bon[a-zA-Z]+\s*=", re.IGNORECASE)


def _check_svg_script(path: Path) -> str | None:
    """R3-svg-script: SVG 内の <script> 要素・イベントハンドラ属性を文字列検査で
    検出する（過検知許容・見逃し回避）。ファイルは読むのみで一切実行しない。"""
    try:
        content = path.read_text(encoding="utf-8", errors="ignore")
    except OSError:
        return None
    if _SVG_SCRIPT_RE.search(content):
        return "SVG contains an embedded <script> element"
    if _SVG_EVENT_ATTR_RE.search(content):
        return "SVG contains an inline event handler attribute (on*=)"
    return None


def _walk_no_follow(root: Path) -> Iterator[tuple[Path, bool]]:
    """root 配下を再帰的に走査するが symlink は辿らない。

    yield: (path, is_symlink) のタプル。is_symlink=True の場合、呼び出し元は
    その場所を R0-symlink 違反として報告し、内部には決して降りない
    （パストラバーサル・意図しないファイル実行経路の混入を防ぐ不変条件）。
    """
    try:
        entries = sorted(root.iterdir(), key=lambda p: p.name)
    except OSError:
        return
    for entry in entries:
        if entry.is_symlink():
            yield entry, True
            continue
        if entry.is_dir():
            yield entry, False
            yield from _walk_no_follow(entry)
        else:
            yield entry, False


def _check_package_files(pkg_dir: Path) -> list[tuple[str, str, str]]:
    """R0-symlink / R2-ext / R3-shebang / R3-execbit / R3-svg-script をパッケージ
    ディレクトリ配下の全ファイルに対して検査する。戻り値は (rule_id, file, reason)。
    file はパッケージディレクトリからの相対パス文字列。"""
    violations: list[tuple[str, str, str]] = []

    for path, is_symlink in _walk_no_follow(pkg_dir):
        rel = str(path.relative_to(pkg_dir))

        if is_symlink:
            violations.append(
                ("R0-symlink", rel, "symlink entries are not permitted and are not followed")
            )
            continue
        if path.is_dir():
            continue

        try:
            mode = path.stat().st_mode
        except OSError:
            continue

        if mode & (stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH):
            violations.append(("R3-execbit", rel, "file has an executable permission bit set"))

        try:
            with open(path, "rb") as f:
                head = f.read(2)
            if head == b"#!":
                violations.append(("R3-shebang", rel, "file begins with a shebang line"))
        except OSError:
            pass

        ext_reason = _check_extension(path.name)
        if ext_reason is not None:
            violations.append(("R2-ext", rel, ext_reason))

        if path.name.lower().endswith(".svg"):
            svg_reason = _check_svg_script(path)
            if svg_reason is not None:
                violations.append(("R3-svg-script", rel, svg_reason))

    return violations


def check_package(name: str, pkg_dir: Path) -> list[tuple[str, str, str]]:
    """1 パッケージ分の全ルールを検査する。呼び出し元 main() が name を
    VIOLATION/EXEMPTED 出力の package= に使う。"""
    violations = _check_package_json(pkg_dir / "package.json")
    violations.extend(_check_package_files(pkg_dir))
    return violations


def enumerate_packages(
    node_modules: Path,
) -> tuple[list[tuple[str, Path]], list[tuple[str, str, str, str]]]:
    """node_modules 直下を列挙し、通常パッケージ（scoped 含む）と、直下に置かれた
    symlink（それ自体が R0-symlink 違反）を分けて返す。

    戻り値: (packages, toplevel_violations)
      - packages: [(package_name, package_dir), ...]
      - toplevel_violations: [(package_name, rule_id, file, reason), ...]
    """
    packages: list[tuple[str, Path]] = []
    toplevel_violations: list[tuple[str, str, str, str]] = []

    try:
        entries = sorted(node_modules.iterdir(), key=lambda p: p.name)
    except OSError as exc:
        raise CheckError(f"failed to list node_modules: {exc}") from exc

    for entry in entries:
        # `.bin`（npm 実行ラッパーの symlink 集約ディレクトリ）や `.package-lock.json`
        # 等の npm 実装詳細はパッケージではないため列挙対象から除外する。
        if entry.name.startswith(IGNORED_TOPLEVEL_PREFIX):
            continue

        if entry.is_symlink():
            toplevel_violations.append(
                (entry.name, "R0-symlink", entry.name, "symlink entries are not permitted and are not followed")
            )
            continue

        if not entry.is_dir():
            continue

        if entry.name.startswith("@"):
            try:
                sub_entries = sorted(entry.iterdir(), key=lambda p: p.name)
            except OSError:
                continue
            for sub in sub_entries:
                if sub.name.startswith(IGNORED_TOPLEVEL_PREFIX):
                    continue
                pkg_name = f"{entry.name}/{sub.name}"
                if sub.is_symlink():
                    toplevel_violations.append(
                        (pkg_name, "R0-symlink", pkg_name, "symlink entries are not permitted and are not followed")
                    )
                    continue
                if sub.is_dir():
                    packages.append((pkg_name, sub))
        else:
            packages.append((entry.name, entry))

    return packages, toplevel_violations


def run(argv: list[str]) -> int:
    """スクリプト本体。exit code (0/1/2) を返す。main() から呼ばれ、CheckError は
    ここで exit 2 相当の戻り値に変換される。"""
    args = parse_args(argv)

    try:
        node_modules = resolve_node_modules(args)
        exemptions = load_allowlist(args.allowlist)
        packages, toplevel_violations = enumerate_packages(node_modules)
    except CheckError as exc:
        print(f"Error: {exc}", file=sys.stderr)
        return 2

    all_violations: list[tuple[str, str, str, str]] = list(toplevel_violations)
    for name, pkg_dir in packages:
        for rule, file, reason in check_package(name, pkg_dir):
            all_violations.append((name, rule, file, reason))

    violation_count = 0
    for name, rule, file, reason in all_violations:
        key = (name, rule)
        if key in exemptions:
            print(f'EXEMPTED package={name} rule={rule} reason="{exemptions[key]}"')
            continue
        print(f'VIOLATION package={name} rule={rule} file={file} reason="{reason}"')
        violation_count += 1

    return 1 if violation_count else 0


def main() -> None:
    sys.exit(run(sys.argv[1:]))


if __name__ == "__main__":
    main()
