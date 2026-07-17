#!/usr/bin/env python3
"""test_check_static_only.py

役割: check_static_only.py（TASK-12.2b）のオフライン回帰テスト。
実 npm・ネットワークに依存せず、tempfile 上に node_modules フィクスチャを
動的生成して各ルール（R0〜R3）・終了コード契約・allowlist 免除機構を検証する。
XSS 回帰テスト同様、このテストを弱体化・skip でごまかさないこと
（.claude/rules/coding-rust.md のテスト規約の精神を Python テストにも適用）。

呼び出し文脈: このテストは check_static_only.py の公開関数 run() を直接呼び出し、
標準出力をキャプチャして VIOLATION/EXEMPTED 行のフォーマットと終了コードを検証する。
"""

from __future__ import annotations

import contextlib
import importlib.util
import io
import json
import os
import stat
import sys
import tempfile
import unittest
from pathlib import Path

SCRIPT_PATH = Path(__file__).resolve().parent.parent / "check_static_only.py"

# check_static_only.py は tools/ 配下の単体スクリプト（パッケージ化されていない）
# のため、importlib で直接モジュールとして読み込む。
_spec = importlib.util.spec_from_file_location("check_static_only", SCRIPT_PATH)
assert _spec is not None and _spec.loader is not None
check_static_only = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(check_static_only)


def write_package_json(pkg_dir: Path, data: dict) -> None:
    (pkg_dir / "package.json").write_text(json.dumps(data), encoding="utf-8")


def run_capture(args: list[str]) -> tuple[int, str]:
    """check_static_only.run() を呼び出し、(exit_code, stdout) を返す。"""
    buf = io.StringIO()
    with contextlib.redirect_stdout(buf):
        code = check_static_only.run(args)
    return code, buf.getvalue()


class BaseFixtureTest(unittest.TestCase):
    def setUp(self) -> None:
        self._tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self._tmp.cleanup)
        self.node_modules = Path(self._tmp.name) / "node_modules"
        self.node_modules.mkdir()

    def make_package(self, name: str, package_json: dict | None = None) -> Path:
        pkg_dir = self.node_modules / name
        pkg_dir.mkdir(parents=True)
        if package_json is not None:
            write_package_json(pkg_dir, package_json)
        return pkg_dir


class TestPassingCases(BaseFixtureTest):
    def test_css_font_license_only_package_passes(self) -> None:
        pkg = self.make_package("nice-css-pkg", {"name": "nice-css-pkg", "version": "1.0.0"})
        (pkg / "style.css").write_text("body{color:red}", encoding="utf-8")
        (pkg / "font.woff2").write_bytes(b"\x00\x01")
        (pkg / "LICENSE").write_text("MIT", encoding="utf-8")
        (pkg / "README.md").write_text("# readme", encoding="utf-8")

        code, out = run_capture(["--node-modules", str(self.node_modules)])
        self.assertEqual(code, 0, out)
        self.assertEqual(out, "")

    def test_scoped_package_passes(self) -> None:
        pkg = self.make_package("@scope/nice-pkg", {"name": "@scope/nice-pkg"})
        (pkg / "style.css").write_text("body{}", encoding="utf-8")

        code, out = run_capture(["--node-modules", str(self.node_modules)])
        self.assertEqual(code, 0, out)

    def test_main_pointing_to_css_is_allowed(self) -> None:
        self.make_package("css-only-main", {"main": "style.css"})
        (self.node_modules / "css-only-main" / "style.css").write_text("a{}", encoding="utf-8")

        code, out = run_capture(["--node-modules", str(self.node_modules)])
        self.assertEqual(code, 0, out)


class TestRule1PackageJson(BaseFixtureTest):
    def test_bin_field_violates(self) -> None:
        self.make_package("has-bin", {"bin": "./cli.js"})
        code, out = run_capture(["--node-modules", str(self.node_modules)])
        self.assertEqual(code, 1)
        self.assertIn("rule=R1-bin", out)
        self.assertIn("package=has-bin", out)

    def test_main_js_entry_violates(self) -> None:
        self.make_package("has-main-js", {"main": "index.js"})
        code, out = run_capture(["--node-modules", str(self.node_modules)])
        self.assertEqual(code, 1)
        self.assertIn("rule=R1-entry", out)

    def test_exports_nested_js_entry_violates(self) -> None:
        self.make_package(
            "has-exports",
            {"exports": {".": {"import": "./esm/index.mjs", "default": "./style.css"}}},
        )
        code, out = run_capture(["--node-modules", str(self.node_modules)])
        self.assertEqual(code, 1)
        self.assertIn("rule=R1-entry", out)

    def test_lifecycle_scripts_violate(self) -> None:
        self.make_package("has-scripts", {"scripts": {"postinstall": "node setup.js"}})
        code, out = run_capture(["--node-modules", str(self.node_modules)])
        self.assertEqual(code, 1)
        self.assertIn("rule=R1-scripts", out)
        self.assertIn("postinstall", out)

    def test_non_lifecycle_script_does_not_violate_r1_scripts(self) -> None:
        self.make_package("has-test-script", {"scripts": {"test": "echo ok"}})
        code, out = run_capture(["--node-modules", str(self.node_modules)])
        self.assertEqual(code, 0, out)


class TestRule2Extension(BaseFixtureTest):
    def test_js_file_violates(self) -> None:
        pkg = self.make_package("has-js-file")
        (pkg / "helper.js").write_text("console.log(1)", encoding="utf-8")
        code, out = run_capture(["--node-modules", str(self.node_modules)])
        self.assertEqual(code, 1)
        self.assertIn("rule=R2-ext", out)
        self.assertIn("file=helper.js", out)

    def test_dts_file_violates(self) -> None:
        pkg = self.make_package("has-dts")
        (pkg / "index.d.ts").write_text("export {}", encoding="utf-8")
        code, out = run_capture(["--node-modules", str(self.node_modules)])
        self.assertEqual(code, 1)
        self.assertIn("rule=R2-ext", out)

    def test_min_js_file_violates(self) -> None:
        pkg = self.make_package("has-min-js")
        (pkg / "bundle.min.js").write_text("!function(){}()", encoding="utf-8")
        code, out = run_capture(["--node-modules", str(self.node_modules)])
        self.assertEqual(code, 1)
        self.assertIn("rule=R2-ext", out)


class TestRule3Filesystem(BaseFixtureTest):
    def test_shebang_violates(self) -> None:
        pkg = self.make_package("has-shebang")
        f = pkg / "script.sh"
        f.write_text("#!/bin/sh\necho hi\n", encoding="utf-8")
        code, out = run_capture(["--node-modules", str(self.node_modules)])
        self.assertEqual(code, 1)
        self.assertIn("rule=R3-shebang", out)

    def test_exec_bit_violates(self) -> None:
        pkg = self.make_package("has-execbit")
        f = pkg / "data.txt"
        f.write_text("plain data", encoding="utf-8")
        f.chmod(f.stat().st_mode | stat.S_IXUSR)
        code, out = run_capture(["--node-modules", str(self.node_modules)])
        self.assertEqual(code, 1)
        self.assertIn("rule=R3-execbit", out)

    def test_svg_script_tag_violates(self) -> None:
        pkg = self.make_package("has-svg-script")
        f = pkg / "icon.svg"
        f.write_text('<svg><script>alert(1)</script></svg>', encoding="utf-8")
        code, out = run_capture(["--node-modules", str(self.node_modules)])
        self.assertEqual(code, 1)
        self.assertIn("rule=R3-svg-script", out)

    def test_svg_event_attribute_violates(self) -> None:
        pkg = self.make_package("has-svg-onload")
        f = pkg / "icon.svg"
        f.write_text('<svg onload="alert(1)"></svg>', encoding="utf-8")
        code, out = run_capture(["--node-modules", str(self.node_modules)])
        self.assertEqual(code, 1)
        self.assertIn("rule=R3-svg-script", out)

    def test_plain_svg_passes(self) -> None:
        pkg = self.make_package("has-plain-svg")
        f = pkg / "icon.svg"
        f.write_text('<svg><circle r="1"/></svg>', encoding="utf-8")
        code, out = run_capture(["--node-modules", str(self.node_modules)])
        self.assertEqual(code, 0, out)


class TestSymlink(BaseFixtureTest):
    def test_symlinked_package_reported_not_followed(self) -> None:
        real_target = Path(self._tmp.name) / "outside-target"
        real_target.mkdir()
        (real_target / "evil.js").write_text("console.log('evil')", encoding="utf-8")

        link = self.node_modules / "linked-pkg"
        os.symlink(real_target, link)

        code, out = run_capture(["--node-modules", str(self.node_modules)])
        self.assertEqual(code, 1)
        self.assertIn("rule=R0-symlink", out)
        # symlink 先の中身（evil.js）は辿られていないため R2-ext 違反は出ない。
        self.assertNotIn("evil.js", out)

    def test_symlinked_file_inside_package_reported_not_followed(self) -> None:
        pkg = self.make_package("pkg-with-symlink-file")
        outside_file = Path(self._tmp.name) / "outside.js"
        outside_file.write_text("console.log('x')", encoding="utf-8")
        os.symlink(outside_file, pkg / "linked.js")

        code, out = run_capture(["--node-modules", str(self.node_modules)])
        self.assertEqual(code, 1)
        self.assertIn("rule=R0-symlink", out)


class TestAllowlist(BaseFixtureTest):
    def write_allowlist(self, content: str) -> Path:
        path = Path(self._tmp.name) / "allowlist.toml"
        path.write_text(content, encoding="utf-8")
        return path

    def test_valid_exemption_suppresses_violation_and_exits_zero(self) -> None:
        self.make_package("has-scripts", {"scripts": {"postinstall": "node setup.js"}})
        allowlist = self.write_allowlist(
            """
[[exempt]]
package = "has-scripts"
rule = "R1-scripts"
reason = "known false positive for test fixture"
"""
        )
        code, out = run_capture(
            ["--node-modules", str(self.node_modules), "--allowlist", str(allowlist)]
        )
        self.assertEqual(code, 0, out)
        self.assertIn("EXEMPTED package=has-scripts rule=R1-scripts", out)

    def test_missing_reason_is_fail_closed_exit_2(self) -> None:
        self.make_package("pkg-a", {"bin": "./x.js"})
        allowlist = self.write_allowlist(
            """
[[exempt]]
package = "pkg-a"
rule = "R1-bin"
reason = ""
"""
        )
        code, out = run_capture(
            ["--node-modules", str(self.node_modules), "--allowlist", str(allowlist)]
        )
        self.assertEqual(code, 2)

    def test_unknown_rule_is_fail_closed_exit_2(self) -> None:
        allowlist = self.write_allowlist(
            """
[[exempt]]
package = "pkg-a"
rule = "R99-nonexistent"
reason = "bogus"
"""
        )
        code, out = run_capture(
            ["--node-modules", str(self.node_modules), "--allowlist", str(allowlist)]
        )
        self.assertEqual(code, 2)

    def test_wildcard_package_is_fail_closed_exit_2(self) -> None:
        allowlist = self.write_allowlist(
            """
[[exempt]]
package = "*"
rule = "R1-bin"
reason = "too broad"
"""
        )
        code, out = run_capture(
            ["--node-modules", str(self.node_modules), "--allowlist", str(allowlist)]
        )
        self.assertEqual(code, 2)


class TestExitCodeContract(BaseFixtureTest):
    def test_missing_node_modules_path_exits_2(self) -> None:
        missing = Path(self._tmp.name) / "does-not-exist"
        code, out = run_capture(["--node-modules", str(missing)])
        self.assertEqual(code, 2)

    def test_dir_flag_derives_node_modules(self) -> None:
        project_dir = Path(self._tmp.name)
        code, out = run_capture(["--dir", str(project_dir)])
        self.assertEqual(code, 0, out)

    def test_violation_output_format(self) -> None:
        self.make_package("bad-pkg", {"bin": "./cli.js"})
        code, out = run_capture(["--node-modules", str(self.node_modules)])
        self.assertEqual(code, 1)
        line = out.strip()
        self.assertRegex(
            line,
            r'^VIOLATION package=bad-pkg rule=R1-bin file=package\.json reason=".*"$',
        )


if __name__ == "__main__":
    unittest.main()
