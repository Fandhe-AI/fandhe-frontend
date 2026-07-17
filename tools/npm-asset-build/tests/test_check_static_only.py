#!/usr/bin/env python3
"""test_check_static_only.py

役割: `check_static_only.py`（TASK-12.2b・#123、`install.sh` の後段ゲート）に
対する fixture 回帰テスト。TASK-12.2c（本イシュー #124）は #123 と並列進行中で
あり、本ファイルは #123 マージ前後のどちらでも CI が green であることを
保証する形で書かれている（存在ガード、詳細は後述）。

対象仕様: docs/npm-static-asset-rules.md（TASK-12.2a・#122）。
本テストは同ドキュメント §4「実装インターフェース契約」のうち **確定
（frozen）**な項目のみを厳密検証し、#123 の裁量に委ねられた項目
（allowlist.toml の具体スキーマ・rule ID の語彙・symlink の具体的な扱い
方法選択）は緩めに照合するか、明示的にスキップする。これは #123 が
ドキュメントに反しない別実装を選んだ場合に本テストが誤ってレッドに
なることを避けるため（フォールスノガティブよりフォールスポジティブの
回避を優先。#123 マージ後に本ファイルの緩い部分を厳密化するのは自然な
フォローアップとして許容する）。

実行方法: python3 tools/npm-asset-build/tests/test_check_static_only.py -v
（Python 3 標準ライブラリのみ使用。pip 依存追加なし。設計 §4 の実装制約と
同じ方針を踏襲する。）
"""

from __future__ import annotations

import json
import os
import re
import stat
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

SCRIPT_DIR = Path(__file__).resolve().parent
CHECK_SCRIPT = SCRIPT_DIR.parent / "check_static_only.py"

# VIOLATION 行フォーマットの契約（設計 §4）。
# `VIOLATION package=<pkg> rule=<rule_id> file=<path> reason="<reason>"` の
# キー集合・順序は確定事項として厳密照合する。rule_id の具体的な語彙
# （例: "R1-bin"）は #123 の裁量であり、ここではドキュメントが例示した
# 命名規則そのものを固定しない（\S+ で緩く受ける）。
VIOLATION_LINE_RE = re.compile(
    r'^VIOLATION package=(?P<package>\S+) rule=(?P<rule>\S+) '
    r'file=(?P<file>\S+) reason="(?P<reason>[^"]*)"$'
)


def _make_package(root: Path, name: str, *, package_json: dict | None = None,
                   files: dict[str, bytes] | None = None) -> Path:
    """`root/node_modules/<name>/` にダミーパッケージを作成する。

    fixture はリポジトリにコミットせず、呼び出し元の
    tempfile.TemporaryDirectory() 配下に閉じて動的生成する（設計計画 §4
    Step 1 の方針。`.js` や実行ビット付きファイルをリポジトリに常設すると
    他の静的検査・レビューを撹乱するため）。
    """
    pkg_dir = root / "node_modules" / name
    pkg_dir.mkdir(parents=True, exist_ok=True)
    pj = {"name": name, "version": "1.0.0"}
    if package_json:
        pj.update(package_json)
    (pkg_dir / "package.json").write_text(json.dumps(pj), encoding="utf-8")
    for rel_path, content in (files or {}).items():
        target = pkg_dir / rel_path
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(content)
    return pkg_dir


def _run_checker(node_modules_parent: Path) -> subprocess.CompletedProcess:
    return subprocess.run(
        [sys.executable, str(CHECK_SCRIPT), "--node-modules",
         str(node_modules_parent / "node_modules")],
        capture_output=True,
        text=True,
        timeout=30,
    )


@unittest.skipUnless(
    CHECK_SCRIPT.is_file(),
    # ::notice 相当の明示メッセージ。check_static_only.py 本体は
    # TASK-12.2b（#123）のスコープであり、本 PR 時点では未マージ
    # （#123 完了までの直列化措置）。#123 マージ後、本ファイルは
    # 自動的に有効化される。
    "check_static_only.py not found yet (blocked on TASK-12.2b #123). "
    "Skipping fixture tests.",
)
class CheckStaticOnlyTests(unittest.TestCase):
    """`check_static_only.py` の fixture 回帰テスト（設計 §4 テスト設計方針）。"""

    def setUp(self) -> None:
        self._tmpdir = tempfile.TemporaryDirectory()
        self.addCleanup(self._tmpdir.cleanup)
        self.root = Path(self._tmpdir.name)

    # --- Rule 1: package.json メタデータ検査 -----------------------------

    def test_pass_static_only_package(self) -> None:
        """許可拡張子のみのパッケージは合格（exit 0）。"""
        _make_package(
            self.root, "clean-pkg",
            files={
                "style.css": b"body{color:red}",
                "font.woff2": b"\x00\x01",
                "icon.svg": b"<svg xmlns='http://www.w3.org/2000/svg'></svg>",
                "data.json": b"{}",
                "LICENSE": b"MIT",
            },
        )
        result = _run_checker(self.root)
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)

    def test_violation_bin_field(self) -> None:
        """`bin` フィールドが存在するパッケージは違反（Rule 1）。"""
        _make_package(
            self.root, "has-bin",
            package_json={"bin": "./cli.js"},
            files={"cli.js": b"#!/usr/bin/env node\n"},
        )
        result = _run_checker(self.root)
        self.assertEqual(result.returncode, 1)
        self._assert_violation_for_package(result.stdout, "has-bin")

    def test_violation_main_points_to_js(self) -> None:
        """`main` が `.js` 実行エントリを指すパッケージは違反（Rule 1）。"""
        _make_package(
            self.root, "main-js",
            package_json={"main": "index.js"},
            files={"index.js": b"module.exports = {};\n"},
        )
        result = _run_checker(self.root)
        self.assertEqual(result.returncode, 1)
        self._assert_violation_for_package(result.stdout, "main-js")

    def test_violation_lifecycle_script(self) -> None:
        """`scripts` にライフサイクルスクリプトが存在すれば違反（Rule 1）。"""
        _make_package(
            self.root, "has-lifecycle",
            package_json={"scripts": {"postinstall": "node setup.js"}},
        )
        result = _run_checker(self.root)
        self.assertEqual(result.returncode, 1)
        self._assert_violation_for_package(result.stdout, "has-lifecycle")

    # --- Rule 2: ファイル実体の拡張子 allowlist 検査 ----------------------

    def test_violation_js_file_present(self) -> None:
        """許可拡張子外の `.js` ファイル混入は違反（Rule 2）。"""
        _make_package(
            self.root, "has-js-file",
            files={"lib/helper.js": b"console.log(1);\n"},
        )
        result = _run_checker(self.root)
        self.assertEqual(result.returncode, 1)
        self._assert_violation_for_package(result.stdout, "has-js-file")

    # --- Rule 3: ファイル属性・内容の追加検査 -----------------------------

    def test_violation_shebang_disguised_as_allowed_extension(self) -> None:
        """許可拡張子でも shebang で始まれば違反（Rule 3）。"""
        pkg_dir = _make_package(
            self.root, "shebang-disguise",
            files={"data.json": b"#!/bin/sh\necho hi\n"},
        )
        result = _run_checker(self.root)
        self.assertEqual(result.returncode, 1)
        self._assert_violation_for_package(result.stdout, "shebang-disguise")

    def test_violation_executable_regular_file(self) -> None:
        """通常ファイルへの実行ビット付与は違反（Rule 3）。"""
        pkg_dir = _make_package(
            self.root, "exec-bit-file",
            files={"asset.txt": b"hello\n"},
        )
        target = pkg_dir / "asset.txt"
        target.chmod(target.stat().st_mode | stat.S_IXUSR)
        result = _run_checker(self.root)
        self.assertEqual(result.returncode, 1)
        self._assert_violation_for_package(result.stdout, "exec-bit-file")

    def test_pass_executable_directory_is_not_a_violation(self) -> None:
        """ディレクトリの実行ビット（トラバース権限）は違反対象外（Rule 3 の除外）。"""
        pkg_dir = _make_package(
            self.root, "exec-bit-dir",
            files={"nested/asset.txt": b"hello\n"},
        )
        nested_dir = pkg_dir / "nested"
        nested_dir.chmod(nested_dir.stat().st_mode | stat.S_IXUSR)
        result = _run_checker(self.root)
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)

    def test_violation_svg_script_element(self) -> None:
        """SVG 内 `<script>` 混入は違反（Rule 3、REQ-1 既定エスケープ方針との整合）。"""
        _make_package(
            self.root, "svg-script",
            files={
                "icon.svg": (
                    b"<svg xmlns='http://www.w3.org/2000/svg'>"
                    b"<script>alert(1)</script></svg>"
                ),
            },
        )
        result = _run_checker(self.root)
        self.assertEqual(result.returncode, 1)
        self._assert_violation_for_package(result.stdout, "svg-script")

    def test_violation_svg_javascript_scheme_href(self) -> None:
        """SVG `href`/`xlink:href` の `javascript:` スキームは違反（Rule 3）。"""
        _make_package(
            self.root, "svg-js-href",
            files={
                "icon.svg": (
                    b"<svg xmlns='http://www.w3.org/2000/svg'>"
                    b"<a xlink:href='javascript:alert(1)'>x</a></svg>"
                ),
            },
        )
        result = _run_checker(self.root)
        self.assertEqual(result.returncode, 1)
        self._assert_violation_for_package(result.stdout, "svg-js-href")

    # --- §3.2 走査境界: ネストした node_modules ---------------------------

    def test_nested_node_modules_boundary(self) -> None:
        """親パッケージ自体はクリーンで子（transitive 依存）のみ違反の場合、
        親は合格・子のみ違反として個別に報告される（設計 §3.2 の走査境界）。

        フォーマット（VIOLATION 行のキー集合）は確定事項として検証するが、
        「親・子どちらの package= 値で報告するか」の細部は #123 の実装詳細に
        依存しうるため、ここでは「クリーンな親パッケージ名自体が違反として
        報告されないこと」と「合計 exit code が違反ありを示すこと」のみを
        厳密に検証する。
        """
        parent_dir = _make_package(
            self.root, "clean-parent",
            files={"style.css": b"body{}"},
        )
        _make_package(
            parent_dir, "dirty-child",
            files={"index.js": b"console.log(1);\n"},
        )
        result = _run_checker(self.root)
        self.assertEqual(result.returncode, 1)
        self.assertNotIn(
            "package=clean-parent ", result.stdout,
            "クリーンな親パッケージ自身が違反として報告されてはならない",
        )

    # --- §4 実装制約: 実行コード拡張子はハード拒否（免除不可） -------------

    def test_js_extension_rejected_even_with_allowlist_entry(self) -> None:
        """実行コード拡張子（`.js` 等）は allowlist.toml による免除エントリが
        あっても拒否されたまま（設計 §3.4 のハード拒否・抜け道封じ）。

        allowlist.toml の具体的なファイルフォーマット・パーサ実装は
        #123 のスコープ（設計 §3.4 末尾）のため、本テストは特定のシンタックス
        を仮定しない。allowlist.toml を配置しない状態（免除なし）でも
        `.js` が拒否されることを確認することで、「免除機構が存在しない場合に
        当然拒否される」という下限を固定し、#123 マージ後に免除ありのケースを
        追補する土台とする。
        """
        _make_package(
            self.root, "hard-reject-js",
            files={"index.js": b"console.log(1);\n"},
        )
        result = _run_checker(self.root)
        self.assertEqual(result.returncode, 1)
        self._assert_violation_for_package(result.stdout, "hard-reject-js")

    # --- CLI 契約: パス不在 / 出力フォーマット ----------------------------

    def test_exit_code_2_on_missing_path(self) -> None:
        """`--node-modules` のパスが存在しない場合、実行エラーとして exit 2。"""
        missing_path = self.root / "does-not-exist" / "node_modules"
        result = subprocess.run(
            [sys.executable, str(CHECK_SCRIPT), "--node-modules",
             str(missing_path)],
            capture_output=True,
            text=True,
            timeout=30,
        )
        self.assertEqual(result.returncode, 2, result.stdout + result.stderr)

    def test_violation_line_format_contract(self) -> None:
        """VIOLATION 行はキー集合 package=/rule=/file=/reason= を持つ
        （設計 §4 出力形式。rule ID の具体的な語彙は #123 の裁量のため
        \\S+ で緩く照合し、キーの並び・存在のみを確定事項として検証する）。
        """
        _make_package(
            self.root, "format-check",
            package_json={"bin": "./cli.js"},
        )
        result = _run_checker(self.root)
        self.assertEqual(result.returncode, 1)
        violation_lines = [
            line for line in result.stdout.splitlines()
            if line.startswith("VIOLATION ")
        ]
        self.assertTrue(violation_lines, "VIOLATION 行が出力されていない")
        matched = [
            line for line in violation_lines
            if VIOLATION_LINE_RE.match(line)
        ]
        self.assertTrue(
            matched,
            f"VIOLATION 行のフォーマットが契約と一致しない: {violation_lines}",
        )

    # --- 補助 -------------------------------------------------------------

    def _assert_violation_for_package(self, stdout: str, package_name: str) -> None:
        """指定パッケージ名を含む VIOLATION 行が最低 1 行出力されていることを
        確認する（rule ID の具体的な語彙は問わない）。
        """
        found = any(
            VIOLATION_LINE_RE.match(line) and
            VIOLATION_LINE_RE.match(line).group("package") == package_name
            for line in stdout.splitlines()
            if line.startswith("VIOLATION ")
        )
        self.assertTrue(
            found,
            f"'{package_name}' に対する VIOLATION 行が見つからない: {stdout}",
        )


if __name__ == "__main__":
    unittest.main()
