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

    def test_main_dot_path_violates_r1_entry(self) -> None:
        # 回帰: `"main": "."` はベース名が "." になり、素朴な "." in base 判定
        # では「拡張子あり」と誤判定されて非 JS 拡張子を導出してしまい R1-entry
        # をすり抜けていた（Node 解決規則の下ではディレクトリ参照も最終的に
        # .js に解決され得るため fail closed で検出する必要がある）。
        self.make_package("has-dot-main", {"main": "."})
        code, out = run_capture(["--node-modules", str(self.node_modules)])
        self.assertEqual(code, 1, out)
        self.assertIn("rule=R1-entry", out)


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

    def test_svg_xlink_href_javascript_scheme_violates(self) -> None:
        pkg = self.make_package("has-svg-xlink-href")
        f = pkg / "icon.svg"
        f.write_text('<svg><use xlink:href="javascript:alert(1)"/></svg>', encoding="utf-8")
        code, out = run_capture(["--node-modules", str(self.node_modules)])
        self.assertEqual(code, 1)
        self.assertIn("rule=R3-svg-script", out)

    def test_svg_href_data_scheme_violates(self) -> None:
        pkg = self.make_package("has-svg-href-data")
        f = pkg / "icon.svg"
        f.write_text(
            '<svg><a href="data:text/html,<script>alert(1)</script>"></a></svg>',
            encoding="utf-8",
        )
        code, out = run_capture(["--node-modules", str(self.node_modules)])
        self.assertEqual(code, 1)
        self.assertIn("rule=R3-svg-script", out)

    def test_svg_unquoted_href_javascript_scheme_violates(self) -> None:
        # 回帰: クォートを要求する正規表現は `href=javascript:...`（クォート
        # なし）を見逃していた。HTML/SVG パーサはクォートなし属性値を有効に
        # 受理するため、イベントハンドラ検出（クォートなしも受理）との非対称
        # を解消する。
        pkg = self.make_package("has-svg-unquoted-href")
        f = pkg / "icon.svg"
        f.write_text('<svg><a href=javascript:alert(1)></a></svg>', encoding="utf-8")
        code, out = run_capture(["--node-modules", str(self.node_modules)])
        self.assertEqual(code, 1, out)
        self.assertIn("rule=R3-svg-script", out)

    def test_svg_unquoted_xlink_href_data_scheme_violates(self) -> None:
        pkg = self.make_package("has-svg-unquoted-xlink-href")
        f = pkg / "icon.svg"
        f.write_text('<svg><use xlink:href=data:text/html,x></use></svg>', encoding="utf-8")
        code, out = run_capture(["--node-modules", str(self.node_modules)])
        self.assertEqual(code, 1, out)
        self.assertIn("rule=R3-svg-script", out)

    def test_svg_foreign_object_violates(self) -> None:
        pkg = self.make_package("has-svg-foreignobject")
        f = pkg / "icon.svg"
        f.write_text(
            '<svg><foreignObject><div xmlns="http://www.w3.org/1999/xhtml">x</div></foreignObject></svg>',
            encoding="utf-8",
        )
        code, out = run_capture(["--node-modules", str(self.node_modules)])
        self.assertEqual(code, 1)
        self.assertIn("rule=R3-svg-script", out)

    def test_plain_svg_passes(self) -> None:
        pkg = self.make_package("has-plain-svg")
        f = pkg / "icon.svg"
        f.write_text('<svg><circle r="1"/></svg>', encoding="utf-8")
        code, out = run_capture(["--node-modules", str(self.node_modules)])
        self.assertEqual(code, 0, out)


class TestNestedNodeModules(BaseFixtureTest):
    """§3.2 の走査境界契約: ネストした node_modules は親パッケージの判定に
    混入させず、独立した判定対象として別列挙する。"""

    def test_clean_parent_with_nested_js_dependency_passes(self) -> None:
        parent = self.make_package("clean-parent", {"name": "clean-parent"})
        (parent / "style.css").write_text("body{}", encoding="utf-8")

        nested_nm = parent / "node_modules"
        nested_dep = nested_nm / "nested-dep"
        nested_dep.mkdir(parents=True)
        write_package_json(nested_dep, {"name": "nested-dep"})
        (nested_dep / "index.js").write_text("console.log(1)", encoding="utf-8")

        code, out = run_capture(["--node-modules", str(self.node_modules)])
        # 親パッケージはクリーンなため合格するが、ネストした子パッケージ自体は
        # 独立した判定対象として別途違反報告される。
        self.assertEqual(code, 1, out)
        self.assertIn("package=nested-dep", out)
        self.assertIn("rule=R2-ext", out)
        self.assertNotIn("package=clean-parent", out)

    def test_nested_clean_dependency_does_not_fail_parent(self) -> None:
        parent = self.make_package("clean-parent2", {"name": "clean-parent2"})
        (parent / "style.css").write_text("body{}", encoding="utf-8")

        nested_nm = parent / "node_modules"
        nested_dep = nested_nm / "nested-clean-dep"
        nested_dep.mkdir(parents=True)
        write_package_json(nested_dep, {"name": "nested-clean-dep"})
        (nested_dep / "style.css").write_text("a{}", encoding="utf-8")

        code, out = run_capture(["--node-modules", str(self.node_modules)])
        self.assertEqual(code, 0, out)

    def test_node_modules_nested_under_subdirectory_is_still_enumerated(self) -> None:
        """node_modules が package root 直下ではなく、任意のサブディレクトリ
        （例: foo/lib/node_modules/evil/payload.js）の下に配置されるケース。
        walk 側の境界スキップと enumerate 側の探索の深さが食い違うと、
        payload.js がどちらの走査にも含まれず未検査のまま見逃される
        （fail-open の回帰）。"""
        parent = self.make_package("foo", {"name": "foo"})
        (parent / "style.css").write_text("body{}", encoding="utf-8")

        nested_dep = parent / "lib" / "node_modules" / "evil"
        nested_dep.mkdir(parents=True)
        write_package_json(nested_dep, {"name": "evil"})
        (nested_dep / "payload.js").write_text(
            "require('child_process').exec('id')", encoding="utf-8"
        )

        code, out = run_capture(["--node-modules", str(self.node_modules)])
        self.assertEqual(code, 1, out)
        self.assertIn("package=evil", out)
        self.assertIn("rule=R2-ext", out)
        self.assertIn("payload.js", out)
        self.assertNotIn("package=foo", out)

    def test_doubly_nested_node_modules_are_enumerated(self) -> None:
        parent = self.make_package("root-pkg", {"name": "root-pkg"})
        (parent / "style.css").write_text("body{}", encoding="utf-8")

        mid = parent / "node_modules" / "mid-pkg"
        mid.mkdir(parents=True)
        write_package_json(mid, {"name": "mid-pkg"})
        (mid / "style.css").write_text("a{}", encoding="utf-8")

        leaf = mid / "node_modules" / "leaf-pkg"
        leaf.mkdir(parents=True)
        write_package_json(leaf, {"name": "leaf-pkg"})
        (leaf / "bad.js").write_text("console.log(1)", encoding="utf-8")

        code, out = run_capture(["--node-modules", str(self.node_modules)])
        self.assertEqual(code, 1, out)
        self.assertIn("package=leaf-pkg", out)
        self.assertNotIn("package=root-pkg", out)
        self.assertNotIn("package=mid-pkg", out)


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

    def test_r2_ext_exempt_without_ext_or_file_is_fail_closed_exit_2(self) -> None:
        """§3.4: 「パッケージ + ルール」単位だけの R2-ext 免除は認めない。
        node_modules/evil-pkg/payload.js に require('child_process') 相当の
        実行コードを仕込み、粗い粒度の免除 1 行で合格させられる抜け道の再現。"""
        pkg = self.make_package("evil-pkg", {"name": "evil-pkg"})
        (pkg / "payload.js").write_text(
            "require('child_process').exec('id')", encoding="utf-8"
        )
        allowlist = self.write_allowlist(
            """
[[exempt]]
package = "evil-pkg"
rule = "R2-ext"
reason = "oops, too broad"
"""
        )
        code, out = run_capture(
            ["--node-modules", str(self.node_modules), "--allowlist", str(allowlist)]
        )
        self.assertEqual(code, 2, out)

    def test_r2_ext_exemption_for_executable_extension_is_fail_closed_exit_2(self) -> None:
        """実行コード拡張子（.js 等）に対する R2-ext 免除はハード拒否
        （ext 指定でも file 指定でも許可しない）。"""
        allowlist = self.write_allowlist(
            """
[[exempt]]
package = "evil-pkg"
rule = "R2-ext"
ext = ".js"
reason = "trying to exempt executable code"
"""
        )
        code, out = run_capture(
            ["--node-modules", str(self.node_modules), "--allowlist", str(allowlist)]
        )
        self.assertEqual(code, 2, out)

        allowlist2 = self.write_allowlist(
            """
[[exempt]]
package = "evil-pkg"
rule = "R2-ext"
file = "payload.js"
reason = "trying to exempt executable code via file path"
"""
        )
        code2, out2 = run_capture(
            ["--node-modules", str(self.node_modules), "--allowlist", str(allowlist2)]
        )
        self.assertEqual(code2, 2, out2)

    def test_unrelated_extension_exemption_does_not_leak_to_executable_code(
        self,
    ) -> None:
        """他拡張子（.dat）向けの ext 単位免除エントリが、同一パッケージ内の
        実行コード拡張子（.js）にまで波及しないことを確認する（免除の照合
        キーが拡張子単位で厳密に絞られていることの回帰テスト）。実行コード
        拡張子への免除エントリ自体は load_allowlist が parse 時点で
        exit 2 拒否するため、ここでは「無関係な免除の漏れ出し」を検証する。"""
        pkg = self.make_package("evil-pkg2", {"name": "evil-pkg2"})
        (pkg / "payload.js").write_text(
            "require('child_process').exec('id')", encoding="utf-8"
        )
        # ext 単位の非実行コード拡張子免除（.dat）はこのパッケージの .js には
        # 一切影響しないことを確認する。
        allowlist = self.write_allowlist(
            """
[[exempt]]
package = "evil-pkg2"
rule = "R2-ext"
ext = ".dat"
reason = "unrelated extension exemption must not affect .js"
"""
        )
        code, out = run_capture(
            ["--node-modules", str(self.node_modules), "--allowlist", str(allowlist)]
        )
        self.assertEqual(code, 1, out)
        self.assertIn("VIOLATION package=evil-pkg2 rule=R2-ext", out)
        self.assertIn("payload.js", out)

    def test_r2_ext_exemption_by_extension_scope_suppresses_only_that_extension(
        self,
    ) -> None:
        pkg = self.make_package("has-unknown-ext", {"name": "has-unknown-ext"})
        (pkg / "data.xyz").write_text("binary-ish data", encoding="utf-8")
        (pkg / "other.zzz").write_text("more data", encoding="utf-8")
        allowlist = self.write_allowlist(
            """
[[exempt]]
package = "has-unknown-ext"
rule = "R2-ext"
ext = ".xyz"
reason = "known safe vendor data format"
"""
        )
        code, out = run_capture(
            ["--node-modules", str(self.node_modules), "--allowlist", str(allowlist)]
        )
        self.assertEqual(code, 1, out)
        self.assertIn("EXEMPTED package=has-unknown-ext rule=R2-ext", out)
        self.assertIn("VIOLATION package=has-unknown-ext rule=R2-ext", out)
        self.assertIn("other.zzz", out)

    def test_r2_ext_exemption_by_file_path_scope(self) -> None:
        pkg = self.make_package("has-unknown-file", {"name": "has-unknown-file"})
        (pkg / "vendor.dat").write_text("binary-ish data", encoding="utf-8")
        allowlist = self.write_allowlist(
            """
[[exempt]]
package = "has-unknown-file"
rule = "R2-ext"
file = "vendor.dat"
reason = "known safe vendor data file"
"""
        )
        code, out = run_capture(
            ["--node-modules", str(self.node_modules), "--allowlist", str(allowlist)]
        )
        self.assertEqual(code, 0, out)
        self.assertIn("EXEMPTED package=has-unknown-file rule=R2-ext", out)

    def test_r2_ext_exempt_with_both_ext_and_file_is_fail_closed_exit_2(self) -> None:
        allowlist = self.write_allowlist(
            """
[[exempt]]
package = "pkg-a"
rule = "R2-ext"
ext = ".xyz"
file = "vendor.xyz"
reason = "both specified"
"""
        )
        code, out = run_capture(
            ["--node-modules", str(self.node_modules), "--allowlist", str(allowlist)]
        )
        self.assertEqual(code, 2, out)


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

    def test_reason_with_embedded_quotes_is_escaped_in_output(self) -> None:
        # 回帰: R1-entry の reason 文字列は
        # `f'"{field}" field resolves to a JS execution entry'` のように
        # フィールド名を生のダブルクォートで囲んでいるため、無エスケープで
        # `reason="..."` 契約に流し込むと途中でクォートが終端し、strict な
        # パーサ（`reason="(?:[^"\\]|\\.)*"` 相当）を壊してしまう。エスケープ
        # 後は埋め込まれた `"` が `\"` に変換され、契約上の 1 つの reason
        # フィールドとして復元可能であることを確認する。
        self.make_package("has-main-js", {"main": "index.js"})
        code, out = run_capture(["--node-modules", str(self.node_modules)])
        self.assertEqual(code, 1, out)
        line = out.strip()
        self.assertIn('reason="\\"main\\" field resolves to a JS execution entry"', line)
        # strict なパーサ相当の正規表現で reason フィールド全体を 1 つに復元できること。
        match = __import__("re").search(r'reason="((?:[^"\\]|\\.)*)"$', line)
        self.assertIsNotNone(match, line)
        restored = match.group(1).replace('\\"', '"').replace("\\\\", "\\")
        self.assertEqual(restored, '"main" field resolves to a JS execution entry')


class TestSuggestExempt(BaseFixtureTest):
    """`--suggest-exempt`（イシュー #296: install.sh の allowlist 自動連携）が
    生成する提案出力の回帰テスト。allowlist.toml への自動書き込みは一切
    行わない契約のため、ここでは stdout への出力内容のみを検証する。"""

    def test_no_suggestion_when_no_violations(self) -> None:
        pkg = self.make_package("clean-pkg", {"name": "clean-pkg"})
        (pkg / "style.css").write_text("body{}", encoding="utf-8")

        code, out = run_capture(
            ["--node-modules", str(self.node_modules), "--suggest-exempt"]
        )
        self.assertEqual(code, 0, out)
        self.assertEqual(out, "")

    def test_flag_absent_suppresses_suggestion_even_with_violation(self) -> None:
        self.make_package("has-bin", {"bin": "./cli.js"})
        code, out = run_capture(["--node-modules", str(self.node_modules)])
        self.assertEqual(code, 1)
        self.assertNotIn("[[exempt]]", out)

    def test_package_rule_scoped_violation_suggests_exempt_snippet(self) -> None:
        self.make_package("has-bin", {"bin": "./cli.js"})
        code, out = run_capture(
            ["--node-modules", str(self.node_modules), "--suggest-exempt"]
        )
        self.assertEqual(code, 1, out)
        self.assertIn("[[exempt]]", out)
        self.assertIn('package = "has-bin"', out)
        self.assertIn('rule = "R1-bin"', out)
        self.assertIn('reason = "TODO:', out)
        # R1-bin は package+rule 単位の免除のため ext/file フィールドは出力しない。
        self.assertNotIn("ext =", out)
        self.assertNotIn("file =", out)

    def test_r2_ext_non_hard_violation_suggests_ext_scoped_snippet(self) -> None:
        pkg = self.make_package("has-dts", {"name": "has-dts"})
        (pkg / "types.d.ts").write_text("export {};", encoding="utf-8")

        code, out = run_capture(
            ["--node-modules", str(self.node_modules), "--suggest-exempt"]
        )
        self.assertEqual(code, 1, out)
        self.assertIn("[[exempt]]", out)
        self.assertIn('package = "has-dts"', out)
        self.assertIn('rule = "R2-ext"', out)
        self.assertIn('ext = ".d.ts"', out)

    def test_hard_deny_violation_reports_not_exemptable_instead_of_snippet(self) -> None:
        self.make_package("has-js", {"main": "index.js"})
        (self.node_modules / "has-js" / "index.js").write_text(
            "module.exports = {};", encoding="utf-8"
        )

        code, out = run_capture(
            ["--node-modules", str(self.node_modules), "--suggest-exempt"]
        )
        self.assertEqual(code, 1, out)
        # R2-ext のハード拒否（実行コード拡張子）はいかなる免除エントリでも
        # 救済不可であることを明示し、[[exempt]] 雛形を出力してはならない
        # （docs/policy/npm-static-asset-rules.md §3.4）。
        self.assertIn("cannot be exempted", out)
        self.assertNotIn("[[exempt]]\npackage = \"has-js\"\nrule = \"R2-ext\"", out)

    def test_escape_toml_string_escapes_quotes_backslash_and_newline(self) -> None:
        """`_escape_toml_string`（TOML v1.0 §5.2.2 準拠）の単体テスト。node_modules
        配下の実ファイル名（攻撃者制御下にあり得る）が TOML 文字列に安全に
        埋め込めることを検証する。"""
        escaped = check_static_only._escape_toml_string('evil".\n[[exempt]]\npackage = "x')
        self.assertEqual(
            escaped,
            'evil\\".\\n[[exempt]]\\npackage = \\"x',
        )
        # エスケープ後の文字列に生の改行・生のダブルクォートが残らないこと。
        self.assertNotIn("\n", escaped)
        self.assertNotIn('"', escaped.replace('\\"', ""))

    def test_r2_ext_violation_with_quote_in_filename_produces_parseable_toml(self) -> None:
        """ファイル名にダブルクォートを含む R2-ext 違反でも、提案された
        `[[exempt]]` 断片が構文的に壊れず tomllib で正しくパースできること
        （file フィールドの値が元のファイル名と一致すること）を検証する。"""
        # 既存の VIOLATION ログ行（file={file} 無エスケープ）は本テストのスコープ
        # 外（#296 以前からの既知の挙動）のため、ペイロードには "[[exempt]]" と
        # いう文字列自体を含めず、`--suggest-exempt` が生成する [[exempt]] 断片
        # 側のみを対象に検証する。
        # 拡張子なしファイル（"." を含まない）にすることで、R2-ext 違反の免除
        # 照合が ext 単位ではなく file（個別ファイルパス）単位になるようにする。
        pkg = self.make_package("has-quote-file", {"name": "has-quote-file"})
        malicious_name = 'weird"file\ninjected = "value'
        (pkg / malicious_name).write_text("data", encoding="utf-8")

        code, out = run_capture(
            ["--node-modules", str(self.node_modules), "--suggest-exempt"]
        )
        self.assertEqual(code, 1, out)

        import tomllib

        snippet_start = out.index("[[exempt]]")
        parsed = tomllib.loads(out[snippet_start:])
        self.assertEqual(len(parsed["exempt"]), 1)
        self.assertEqual(parsed["exempt"][0]["package"], "has-quote-file")
        self.assertEqual(parsed["exempt"][0]["file"], malicious_name)
        # 注入を狙った偽の [[exempt]] ブロックが独立したテーブルとして
        # 追加されていないこと（テーブルは 1 件のみ）。
        self.assertEqual(out.count("[[exempt]]"), 1)

    def test_hard_deny_comment_with_newline_and_quote_does_not_inject_line(self) -> None:
        """`_print_exempt_suggestion` の hard_deny 注記（`#` コメント）出力を
        直接検証する。既存の VIOLATION ログ行（file={file} 無エスケープ）は
        本指摘のスコープ外（#296 以前からの既知の挙動）のため関与させず、
        `--suggest-exempt` 由来の提案出力のみを対象とする。

        改行・ダブルクォートを含むファイル名でも、出力が単一の `#` コメント行に
        収まり、注入マーカーが独立した非コメント行として出力されないことを
        検証する。"""
        malicious_file = 'evil".js\nINJECTED-MARKER = true'

        buf = io.StringIO()
        with contextlib.redirect_stdout(buf):
            check_static_only._print_exempt_suggestion(
                "has-evil-js", "R2-ext", ("hard_deny",), malicious_file
            )
        out = buf.getvalue()

        self.assertIn("cannot be exempted", out)
        lines = out.splitlines()
        # 出力は 1 行のコメントに収まっていること（改行注入によって行が
        # 分割されていない）。
        self.assertEqual(len(lines), 1)
        self.assertTrue(lines[0].startswith("#"))
        # 生のダブルクォート・生の改行がエスケープされていること。
        self.assertNotIn("\n", out.rstrip("\n"))

    def test_already_exempted_violation_does_not_suggest_again(self) -> None:
        self.make_package("has-bin", {"bin": "./cli.js"})
        allowlist_path = Path(self._tmp.name) / "allowlist.toml"
        allowlist_path.write_text(
            '[[exempt]]\npackage = "has-bin"\nrule = "R1-bin"\nreason = "already reviewed"\n',
            encoding="utf-8",
        )

        code, out = run_capture(
            [
                "--node-modules",
                str(self.node_modules),
                "--allowlist",
                str(allowlist_path),
                "--suggest-exempt",
            ]
        )
        self.assertEqual(code, 0, out)
        self.assertIn("EXEMPTED package=has-bin rule=R1-bin", out)
        self.assertNotIn("[[exempt]]", out)


if __name__ == "__main__":
    unittest.main()
