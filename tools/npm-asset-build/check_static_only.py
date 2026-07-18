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
  - Rule 2（R2-ext）の免除は「パッケージ + ルール」単位では認めず、必ず
    「対象拡張子」または「個別ファイルパス」単位を要求する（§3.4）
  - 実行コード拡張子（.js/.mjs/.cjs/.node/.wasm・.min.js を含む）に対する
    R2-ext の拒否は、いかなる粒度の例外エントリでも免除不可（ハード拒否）
  - ネストした node_modules（<pkg>/node_modules/<dep>）は走査境界として除外し、
    子パッケージは独立した判定対象として別列挙する（§3.2。親パッケージの
    判定に子パッケージのファイルを混入させない）

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

# 実行コード拡張子（docs/npm-static-asset-rules.md §3.4）。
# R2-ext の拒否はこれらに対して、いかなる粒度の allowlist エントリでも免除不可
# （ハード拒否）とする。`.min.js` は `.js` とみなして判定する。
HARD_DENY_EXTS = {".js", ".mjs", ".cjs", ".node", ".wasm"}

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
    parser.add_argument(
        "--suggest-exempt",
        action="store_true",
        help=(
            "On violations, print a suggested [[exempt]] TOML snippet to stdout "
            "(does not write to any file; hard-deny violations are reported as "
            "not exemptable instead)."
        ),
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


def _hard_deny_ext_for_name(name: str) -> str | None:
    """name（ファイル名の末尾セグメント）が実行コード拡張子（HARD_DENY_EXTS）に
    該当する場合、正規化した拡張子を返す（`.min.js` は `.js` として扱う）。
    load_allowlist（免除エントリの事前拒否）と _check_extension（違反判定）の
    両方から呼ばれ、判定基準を単一箇所に集約する。"""
    lower = name.lower()
    if lower.endswith(".min.js"):
        return ".js"
    if "." not in lower:
        return None
    ext = "." + lower.rsplit(".", 1)[-1]
    return ext if ext in HARD_DENY_EXTS else None


# allowlist.toml の免除エントリ 1 件を表すキー。
#   - 非 R2-ext ルール: (package, rule, None) — パッケージ + ルール単位
#   - R2-ext ルール: (package, "R2-ext", ("ext", 拡張子)) または
#     (package, "R2-ext", ("file", パッケージ相対パス)) のいずれか一方のみ
#     （契約: docs/npm-static-asset-rules.md §3.4 「Rule 2 の免除は対象拡張子
#     or 個別ファイルパス単位を必須とする」）。
ExemptKey = tuple[str, str, tuple[str, str] | None]


def load_allowlist(path: str | None) -> dict[ExemptKey, str]:
    """allowlist.toml を読み込み、ExemptKey -> reason の免除表を返す。

    契約（docs/npm-static-asset-rules.md §3.4）:
      - reason 欠落・空、未知 rule ID、ワイルドカード的指定はすべてパースエラー
        として CheckError を送出する（呼び出し元 main() で exit 2 に変換）。
      - R2-ext の免除は「対象拡張子（ext）」または「個別ファイルパス（file）」の
        いずれか一方を必須とする（両方指定・両方欠落はどちらもエラー）。
      - 実行コード拡張子（HARD_DENY_EXTS）を対象とする R2-ext 免除エントリは、
        ext 指定・file 指定のいずれであってもエラーとする（ハード拒否の抜け道を
        allowlist 側からも構造的に塞ぐ）。
    ここでの安全側判断は「免除機構自体の設定ミスは、免除なしより危険（サイレント
    な野放図拡大）」という前提に基づく。
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

    exemptions: dict[ExemptKey, str] = {}
    for entry in data.get("exempt", []):
        if not isinstance(entry, dict):
            raise CheckError(f"invalid [[exempt]] entry (not a table): {entry!r}")

        package = entry.get("package")
        rule = entry.get("rule")
        reason = entry.get("reason")
        ext_field = entry.get("ext")
        file_field = entry.get("file")

        if not isinstance(package, str) or not package or "*" in package or "?" in package:
            raise CheckError(f"exempt entry has invalid or wildcard 'package': {entry!r}")
        if rule not in VALID_RULE_IDS:
            raise CheckError(f"exempt entry has unknown 'rule': {rule!r}")
        if not isinstance(reason, str) or not reason.strip():
            raise CheckError(
                f"exempt entry missing non-empty 'reason' for package={package} rule={rule}"
            )

        has_ext = isinstance(ext_field, str) and ext_field.strip() != ""
        has_file = isinstance(file_field, str) and file_field.strip() != ""

        if rule == "R2-ext":
            if has_ext == has_file:  # 両方指定・両方欠落のいずれもエラー
                raise CheckError(
                    "R2-ext exempt entry must specify exactly one of 'ext' or 'file' "
                    f"(package={package}): {entry!r}"
                )
            if has_ext:
                ext_norm = ext_field.strip().lower()
                if not ext_norm.startswith("."):
                    raise CheckError(
                        f"R2-ext exempt entry 'ext' must start with '.': {ext_field!r}"
                    )
                if ext_norm in HARD_DENY_EXTS or ext_norm == ".min.js":
                    raise CheckError(
                        f"R2-ext exemption for executable extension {ext_norm!r} is not "
                        "permitted (hard-deny per docs/npm-static-asset-rules.md §3.4): "
                        f"{entry!r}"
                    )
                key: ExemptKey = (package, rule, ("ext", ext_norm))
            else:
                file_norm = file_field.strip().replace("\\", "/")
                if _hard_deny_ext_for_name(file_norm) is not None:
                    raise CheckError(
                        f"R2-ext exemption for executable file {file_norm!r} is not "
                        "permitted (hard-deny per docs/npm-static-asset-rules.md §3.4): "
                        f"{entry!r}"
                    )
                key = (package, rule, ("file", file_norm))
        else:
            if has_ext or has_file:
                raise CheckError(
                    f"'ext'/'file' fields are only valid for rule=R2-ext: {entry!r}"
                )
            key = (package, rule, None)

        exemptions[key] = reason.strip()

    return exemptions


def _is_js_entry(value: object) -> bool:
    """package.json の main/module/browser/exports が指す値が JS 実行エントリを
    指しているかを再帰的に判定する（exports のネスト構造にも対応）。

    拡張子なしのパスは Node.js の解決アルゴリズムにより暗黙に .js とみなされ得る
    ため、安全側に倒して JS エントリ扱いとする（見逃し回避を優先）。

    ベース名が "." または ".." のパス（例: `"main": "."`）はディレクトリ参照
    であり拡張子を持つファイル名ではないため、`"." in base` による拡張子判定
    をすり抜けて false になってしまう抜け道を塞ぐ。Node の解決規則では
    ディレクトリ参照は package.json の main 解決等を経て最終的に .js に
    解決され得るため、これも拡張子なしパスと同様に安全側で JS エントリ扱い
    とする。
    """
    if isinstance(value, str):
        base = value.rsplit("/", 1)[-1]
        if base in (".", "..") or "." not in base:
            return True
        ext = "." + base.rsplit(".", 1)[-1].lower()
        return ext in JS_EXEC_EXTS
    if isinstance(value, dict):
        return any(_is_js_entry(v) for v in value.values())
    if isinstance(value, list):
        return any(_is_js_entry(v) for v in value)
    return False


# 1 件の違反を表す共通タプル形式: (rule_id, file, reason, detail)。
#   detail は run() での allowlist 照合キーの組み立てに使う:
#     - None: (package, rule) 単位で免除照合する（R2-ext 以外の全ルール）
#     - ("hard_deny",): 実行コード拡張子の R2-ext 違反。いかなる免除エントリでも
#       救済不可（docs/npm-static-asset-rules.md §3.4）
#     - ("ext", 拡張子) / ("file", パッケージ相対パス): R2-ext の免除照合キー
Violation = tuple[str, str, str, tuple[str, ...] | None]


def _check_package_json(pkg_json_path: Path) -> list[Violation]:
    """package.json 由来のルール（R1-bin / R1-entry / R1-scripts）を検査する。
    戻り値は Violation のリスト（file は常に "package.json"）。detail は常に
    None（(package, rule) 単位の免除照合）。"""
    violations: list[Violation] = []

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
            (
                "R1-bin",
                "package.json",
                'package.json declares a "bin" entry (executable command)',
                None,
            )
        )

    for field in ("main", "module", "browser", "exports"):
        if field in data and _is_js_entry(data[field]):
            violations.append(
                (
                    "R1-entry",
                    "package.json",
                    f'"{field}" field resolves to a JS execution entry',
                    None,
                )
            )

    scripts = data.get("scripts")
    if isinstance(scripts, dict):
        hits = sorted(k for k in scripts if k in LIFECYCLE_SCRIPT_KEYS)
        if hits:
            violations.append(
                (
                    "R1-scripts",
                    "package.json",
                    f"lifecycle script(s) present: {', '.join(hits)}",
                    None,
                )
            )

    return violations


def _check_extension(filename: str) -> tuple[str, tuple[str, ...] | None] | None:
    """R2-ext: allowlist にない拡張子・拡張子なしファイルを違反として返す
    （合格時は None）。既定拒否（許可リスト方式）。

    戻り値は (reason, detail) で、detail は run() での免除照合キーの組み立てに
    使う。実行コード拡張子（HARD_DENY_EXTS。`.min.js` を含む）は
    detail=("hard_deny",) を返し、いかなる allowlist エントリでも免除不可と
    する（docs/npm-static-asset-rules.md §3.4）。それ以外の拒否対象は
    detail=("ext", 拡張子) とし、対象拡張子単位の免除照合に使う。
    """
    lower = filename.lower()

    if "." not in lower:
        if lower.startswith(ALLOWED_NOEXT_PREFIXES):
            return None
        return (
            "file has no extension and is not an allowed LICENSE/NOTICE/README file",
            None,
        )

    hard_ext = _hard_deny_ext_for_name(filename)

    # 複合拡張子（.min.js / .d.ts）は単純拡張子判定より先に明示拒否する。
    if lower.endswith(".min.js"):
        return (
            'disallowed extension ".min.js" (minified JS execution code)',
            ("hard_deny",),
        )
    if lower.endswith(".d.ts"):
        return (
            'disallowed extension ".d.ts" (TypeScript declaration source)',
            ("ext", ".d.ts"),
        )

    ext = "." + lower.rsplit(".", 1)[-1]
    if ext in ALLOWED_EXTS:
        return None
    if hard_ext is not None:
        return (f'disallowed extension "{ext}"', ("hard_deny",))
    return (f'disallowed extension "{ext}"', ("ext", ext))


_SVG_SCRIPT_RE = re.compile(r"<script[\s>/]", re.IGNORECASE)
_SVG_EVENT_ATTR_RE = re.compile(r"\bon[a-zA-Z]+\s*=", re.IGNORECASE)
# href / xlink:href 属性値が javascript:/data: スキームで始まる場合を検出する
# （<a>/<use>/<image> 等、SVG 内で href 系属性を持ちうる要素すべてが対象）。
# クォート（"/'）は省略可能とする。HTML/SVG のパーサはクォートなし属性値
# （例: `href=javascript:alert(1)`）も有効な属性として受理するため、クォート
# ありの形式のみを要求すると、そのままクォートを外すだけで検査をすり抜けら
# れてしまう（on*= のイベントハンドラ検出はクォートなしも受理しており、
# ここだけ非対称になっていた）。
_SVG_DANGEROUS_HREF_RE = re.compile(
    r"(?:xlink:href|href)\s*=\s*[\"']?\s*(?:javascript|data)\s*:", re.IGNORECASE
)
# <foreignObject> 要素自体の存在を違反とする（HTML/スクリプトの持ち込み経路）。
_SVG_FOREIGN_OBJECT_RE = re.compile(r"<foreignObject[\s>/]", re.IGNORECASE)


def _check_svg_script(path: Path) -> str | None:
    """R3-svg-script: SVG 内のスクリプト混入経路を文字列検査で検出する
    （過検知許容・見逃し回避、正規表現による広めの一致。docs/npm-static-asset-rules.md
    §3.3 が列挙する 4 種の検査対象すべてをカバーする）。ファイルは読むのみで
    一切実行しない。"""
    try:
        content = path.read_text(encoding="utf-8", errors="ignore")
    except OSError:
        return None
    if _SVG_SCRIPT_RE.search(content):
        return "SVG contains an embedded <script> element"
    if _SVG_EVENT_ATTR_RE.search(content):
        return "SVG contains an inline event handler attribute (on*=)"
    if _SVG_DANGEROUS_HREF_RE.search(content):
        return "SVG contains a href/xlink:href attribute with a javascript:/data: scheme"
    if _SVG_FOREIGN_OBJECT_RE.search(content):
        return "SVG contains a <foreignObject> element"
    return None


# パッケージ自身の走査境界とする npm 実装ディレクトリ名（ネストした transitive
# 依存の配置場所）。docs/npm-static-asset-rules.md §3.2 の契約:
# 「あるパッケージの判定走査はこのディレクトリ自体を境界として除外し、配下は
# パッケージ一覧走査（enumerate_packages）が別途独立した判定対象として列挙する」。
NESTED_NODE_MODULES_DIR = "node_modules"


def _walk_no_follow(root: Path) -> Iterator[tuple[Path, bool]]:
    """root 配下を再帰的に走査するが symlink は辿らず、ネストした
    `node_modules` ディレクトリの手前で止める（§3.2 の走査境界契約）。

    yield: (path, is_symlink) のタプル。is_symlink=True の場合、呼び出し元は
    その場所を R0-symlink 違反として報告し、内部には決して降りない
    （パストラバーサル・意図しないファイル実行経路の混入を防ぐ不変条件）。
    `node_modules` という名前のディレクトリは yield されず内部にも降りない
    （enumerate_packages が子パッケージとして独立に列挙するため、ここで
    混入させると親パッケージの判定に無関係な transitive 依存のファイルが
    誤って計上される）。
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
            if entry.name == NESTED_NODE_MODULES_DIR:
                continue
            yield entry, False
            yield from _walk_no_follow(entry)
        else:
            yield entry, False


def _check_package_files(pkg_dir: Path) -> list[Violation]:
    """R0-symlink / R2-ext / R3-shebang / R3-execbit / R3-svg-script をパッケージ
    ディレクトリ配下の全ファイルに対して検査する（ネストした node_modules は
    _walk_no_follow が境界として除外済み）。戻り値は Violation のリスト。
    file はパッケージディレクトリからの相対パス文字列。"""
    violations: list[Violation] = []

    for path, is_symlink in _walk_no_follow(pkg_dir):
        rel = str(path.relative_to(pkg_dir))

        if is_symlink:
            violations.append(
                (
                    "R0-symlink",
                    rel,
                    "symlink entries are not permitted and are not followed",
                    None,
                )
            )
            continue
        if path.is_dir():
            continue

        try:
            mode = path.stat().st_mode
        except OSError:
            continue

        if mode & (stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH):
            violations.append(
                ("R3-execbit", rel, "file has an executable permission bit set", None)
            )

        try:
            with open(path, "rb") as f:
                head = f.read(2)
            if head == b"#!":
                violations.append(("R3-shebang", rel, "file begins with a shebang line", None))
        except OSError:
            pass

        ext_violation = _check_extension(path.name)
        if ext_violation is not None:
            ext_reason, ext_detail = ext_violation
            # R2-ext の免除照合は拡張子スコープ（ext_detail）に加えて、個別ファイル
            # パス（rel）単位でも照合できるようにするため file 側のキーも run() で
            # 別途試す。ここでは検出した detail をそのまま積む。
            violations.append(("R2-ext", rel, ext_reason, ext_detail))

        if path.name.lower().endswith(".svg"):
            svg_reason = _check_svg_script(path)
            if svg_reason is not None:
                violations.append(("R3-svg-script", rel, svg_reason, None))

    return violations


def check_package(name: str, pkg_dir: Path) -> list[Violation]:
    """1 パッケージ分の全ルールを検査する。呼び出し元 main() が name を
    VIOLATION/EXEMPTED 出力の package= に使う。"""
    violations = _check_package_json(pkg_dir / "package.json")
    violations.extend(_check_package_files(pkg_dir))
    return violations


def _list_one_node_modules_level(
    node_modules: Path,
) -> tuple[list[tuple[str, Path]], list[tuple[str, str, str, str, None]]]:
    """node_modules ディレクトリ 1 段分の直下を列挙する（scoped パッケージ含む）。
    ネストした node_modules の再帰探索は呼び出し元 enumerate_packages が行う
    （§3.2 の境界契約: このディレクトリ自体の列挙はどの階層でも同じロジックで
    行い、親パッケージの判定には混入させない）。

    戻り値: (packages, toplevel_violations)
      - packages: [(package_name, package_dir), ...]
      - toplevel_violations: [(package_name, rule_id, file, reason, None), ...]
    """
    packages: list[tuple[str, Path]] = []
    toplevel_violations: list[tuple[str, str, str, str, None]] = []

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
                (
                    entry.name,
                    "R0-symlink",
                    entry.name,
                    "symlink entries are not permitted and are not followed",
                    None,
                )
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
                        (
                            pkg_name,
                            "R0-symlink",
                            pkg_name,
                            "symlink entries are not permitted and are not followed",
                            None,
                        )
                    )
                    continue
                if sub.is_dir():
                    packages.append((pkg_name, sub))
        else:
            packages.append((entry.name, entry))

    return packages, toplevel_violations


def _find_nested_node_modules_dirs(pkg_dir: Path) -> Iterator[Path]:
    """pkg_dir 配下の**任意の深さ**にある実ディレクトリ `node_modules` を境界
    （_walk_no_follow がファイル走査を止める境界と同じもの）ごとに 1 つずつ
    yield する。

    _walk_no_follow は名前が `node_modules` であるディレクトリを深さに関係なく
    スキップするため（例: `foo/lib/node_modules/evil/payload.js` も境界の対象）、
    ここでの探索もそれと深さの扱いを一致させないと、`foo/lib/node_modules/`
    配下のファイルがどちらの走査にも含まれず未検査のまま見逃される
    （fail-closed の趣旨に反する）。

    symlink は辿らない。symlink である node_modules 自体は、この関数ではなく
    _walk_no_follow が通常のファイル走査中に R0-symlink 違反として報告する
    （symlink は名前を問わず walk 側で真っ先に検出されるため、ここで二重に
    報告しない）。
    """
    try:
        entries = sorted(pkg_dir.iterdir(), key=lambda p: p.name)
    except OSError:
        return
    for entry in entries:
        if entry.is_symlink():
            continue
        if not entry.is_dir():
            continue
        if entry.name == NESTED_NODE_MODULES_DIR:
            yield entry
            continue
        yield from _find_nested_node_modules_dirs(entry)


def enumerate_packages(
    node_modules: Path,
) -> tuple[list[tuple[str, Path]], list[tuple[str, str, str, str, None]]]:
    """node_modules を再帰的に列挙する。

    契約（docs/npm-static-asset-rules.md §3.2）: あるパッケージ `foo` の配下に
    ネストした `node_modules/bar/`（深さは問わない。`foo/node_modules/bar/` は
    もちろん `foo/lib/node_modules/bar/` のような配置も対象）が存在する場合、
    `bar` は `foo` の判定走査（_check_package_files が _walk_no_follow の境界で
    除外する）には含めず、ここで独立した判定対象として別列挙する。ネストは
    多段（bar がさらに node_modules を持つ等）にも対応するため BFS で辿る。

    戻り値: (packages, toplevel_violations) — 形式は _list_one_node_modules_level
    と同じ（全階層分を合算したもの）。
    """
    all_packages: list[tuple[str, Path]] = []
    all_toplevel_violations: list[tuple[str, str, str, str, None]] = []

    queue: list[Path] = [node_modules]
    seen: set[Path] = set()

    while queue:
        current = queue.pop(0)
        try:
            resolved = current.resolve()
        except OSError:
            resolved = current
        if resolved in seen:
            continue
        seen.add(resolved)

        packages, toplevel_violations = _list_one_node_modules_level(current)
        all_toplevel_violations.extend(toplevel_violations)

        for name, pkg_dir in packages:
            all_packages.append((name, pkg_dir))
            for nested_nm in _find_nested_node_modules_dirs(pkg_dir):
                queue.append(nested_nm)

    return all_packages, all_toplevel_violations


def _escape_output_value(value: str) -> str:
    """VIOLATION/EXEMPTED 出力行の `reason="..."` フィールドに埋め込む値を
    エスケープする。reason 文字列（"bin" 等の拡張子・フィールド名を含む
    メッセージ）には生の `"` が含まれ得るため、無エスケープで出力すると
    `reason="..."` 契約（ダブルクォート区切り）を破壊し、下流の strict な
    パーサを壊す（バックスラッシュ・ダブルクォートの双方を \\ でエスケープ
    する最小限の実装）。"""
    return value.replace("\\", "\\\\").replace('"', '\\"')


def _escape_toml_string(value: str) -> str:
    """TOML 基本文字列（`"..."`）へ埋め込む値を TOML v1.0 §5.2.2 のエスケープ
    規則に従ってエスケープする。

    背景: `_print_exempt_suggestion` が出力する package/ext/file の値は
    `enumerate_packages`/`check_package` 経由で node_modules 配下の実ファイル
    パス（npm パッケージの tarball エントリ名）から得られ、攻撃者が自由に
    制御できる。ダブルクォート・バックスラッシュだけでなく改行・制御文字も
    無エスケープで埋め込むと、提案 TOML 断片の構文破壊や追加行（偽の
    `[[exempt]]` ブロック等）の注入を許してしまう。人間がレビューしてそのまま
    allowlist.toml に貼り付ける前提の雛形であるため、ここで確実にエスケープし、
    レビュー時の見落としが fail-closed ゲートの無力化に直結しないようにする。
    """
    out: list[str] = []
    for ch in value:
        if ch == "\\":
            out.append("\\\\")
        elif ch == '"':
            out.append('\\"')
        elif ch == "\b":
            out.append("\\b")
        elif ch == "\t":
            out.append("\\t")
        elif ch == "\n":
            out.append("\\n")
        elif ch == "\f":
            out.append("\\f")
        elif ch == "\r":
            out.append("\\r")
        elif ord(ch) < 0x20 or ord(ch) == 0x7F:
            out.append(f"\\u{ord(ch):04x}")
        else:
            out.append(ch)
    return "".join(out)


def _print_exempt_suggestion(name: str, rule: str, detail: tuple[str, ...] | None, file: str) -> None:
    """`--suggest-exempt` 向けに、1 件の違反へ対応する allowlist.toml の
    `[[exempt]]` 雛形（または免除不可の注記）を stdout へ出力する。

    契約: allowlist.toml への書き込みは一切行わない（提案のみ・fail-closed
    原則の維持）。呼び出し元 run() は「実際に免除されなかった違反」に対して
    のみこれを呼ぶ（EXEMPTED 済みの違反には出力しない）。

    セキュリティ注記: name/file は node_modules 配下の実ファイル名由来で
    攻撃者制御下にあり得るため、TOML 文字列に埋め込む箇所は
    `_escape_toml_string` を通す。hard_deny の注記行（`#` コメント）も、
    改行を含むファイル名によって出力に無関係な追加行が注入されるのを防ぐため
    同様にエスケープする（この行は TOML 文字列ではないが、改行注入を防ぐ
    目的でエスケープ表記を流用する）。
    """
    if detail is not None and detail[0] == "hard_deny":
        print(
            f"# package={_escape_toml_string(name)} rule={rule} "
            f"file={_escape_toml_string(file)}: hard-deny executable "
            "extension — cannot be exempted (docs/npm-static-asset-rules.md §3.4)"
        )
        return

    print("[[exempt]]")
    print(f'package = "{_escape_toml_string(name)}"')
    print(f'rule = "{rule}"')
    if rule == "R2-ext":
        if detail is not None and detail[0] == "ext":
            print(f'ext = "{_escape_toml_string(detail[1])}"')
        else:
            print(f'file = "{_escape_toml_string(file)}"')
    print('reason = "TODO: describe why this exemption is safe for this package"')


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

    all_violations: list[tuple[str, str, str, str, tuple[str, ...] | None]] = list(
        toplevel_violations
    )
    for name, pkg_dir in packages:
        for rule, file, reason, detail in check_package(name, pkg_dir):
            all_violations.append((name, rule, file, reason, detail))

    violation_count = 0
    for name, rule, file, reason, detail in all_violations:
        # R2-ext のハード拒否（実行コード拡張子）はいかなる免除エントリでも
        # 救済しない（docs/npm-static-asset-rules.md §3.4）。
        if detail is not None and detail[0] == "hard_deny":
            print(
                f'VIOLATION package={name} rule={rule} file={file} '
                f'reason="{_escape_output_value(reason)}"'
            )
            violation_count += 1
            if args.suggest_exempt:
                _print_exempt_suggestion(name, rule, detail, file)
            continue

        if rule == "R2-ext":
            # R2-ext: 拡張子単位（detail が ("ext", ...) の場合）またはファイル
            # パス単位のどちらかの免除エントリに一致すれば免除する。拡張子なし
            # ファイル（detail=None）はファイルパス単位でのみ免除可能。
            ext_key: ExemptKey | None = (
                (name, rule, ("ext", detail[1])) if detail is not None and detail[0] == "ext" else None
            )
            file_key: ExemptKey = (name, rule, ("file", file))
            reason_found = (exemptions.get(ext_key) if ext_key is not None else None) or exemptions.get(
                file_key
            )
            if reason_found is not None:
                print(
                    f'EXEMPTED package={name} rule={rule} '
                    f'reason="{_escape_output_value(reason_found)}"'
                )
                continue
            print(
                f'VIOLATION package={name} rule={rule} file={file} '
                f'reason="{_escape_output_value(reason)}"'
            )
            violation_count += 1
            if args.suggest_exempt:
                _print_exempt_suggestion(name, rule, detail, file)
            continue

        key = (name, rule, None)
        if key in exemptions:
            print(
                f'EXEMPTED package={name} rule={rule} '
                f'reason="{_escape_output_value(exemptions[key])}"'
            )
            continue
        print(
            f'VIOLATION package={name} rule={rule} file={file} '
            f'reason="{_escape_output_value(reason)}"'
        )
        violation_count += 1
        if args.suggest_exempt:
            _print_exempt_suggestion(name, rule, detail, file)

    return 1 if violation_count else 0


def main() -> None:
    sys.exit(run(sys.argv[1:]))


if __name__ == "__main__":
    main()
