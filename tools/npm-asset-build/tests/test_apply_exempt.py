#!/usr/bin/env python3
"""test_apply_exempt.py

役割: apply_exempt.py（イシュー #316）のオフライン回帰テスト。
tempfile 上に allowlist.toml / suggestions.toml フィクスチャを動的生成し、
半自動追記コマンドの検証・エスケープ・アトミック書き込み・終了コード契約
（0=適用完了 / 1=検証拒否 / 2=実行エラー）を検証する。
check_static_only.py 側のテスト（test_check_static_only.py）と同様、
弱体化・skip でごまかさないこと。
"""

from __future__ import annotations

import contextlib
import importlib.util
import io
import sys
import tempfile
import unittest
from pathlib import Path

SCRIPT_PATH = Path(__file__).resolve().parent.parent / "apply_exempt.py"

# apply_exempt.py は tools/ 配下の単体スクリプト（パッケージ化されていない）
# のため、importlib で直接モジュールとして読み込む。check_static_only.py の
# import（sys.path 経由の姉妹モジュール読み込み）が実行時に解決できるよう、
# 先に対象ディレクトリを sys.path へ入れておく。
sys.path.insert(0, str(SCRIPT_PATH.parent))
_spec = importlib.util.spec_from_file_location("apply_exempt", SCRIPT_PATH)
assert _spec is not None and _spec.loader is not None
apply_exempt_mod = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(apply_exempt_mod)


def run_capture_main(argv: list[str]) -> tuple[int, str]:
    """apply_exempt.main() を呼び出し、(exit_code, stdout+stderr) を返す。
    main() は sys.exit() するため SystemExit を捕捉する。"""
    buf = io.StringIO()
    old_argv = sys.argv
    sys.argv = ["apply_exempt.py"] + argv
    try:
        with contextlib.redirect_stdout(buf), contextlib.redirect_stderr(buf):
            try:
                apply_exempt_mod.main()
                code = 0
            except SystemExit as exc:
                code = exc.code if isinstance(exc.code, int) else 1
    finally:
        sys.argv = old_argv
    return code, buf.getvalue()


class ApplyExemptTestCase(unittest.TestCase):
    def setUp(self) -> None:
        self._tmp = tempfile.TemporaryDirectory()
        self.tmp_path = Path(self._tmp.name)
        self.allowlist_path = self.tmp_path / "allowlist.toml"
        self.allowlist_path.write_text(
            "# allowlist.toml (fixture header comment)\n", encoding="utf-8"
        )

    def tearDown(self) -> None:
        self._tmp.cleanup()

    def _write_suggestions(self, content: str) -> Path:
        path = self.tmp_path / "suggestions.toml"
        path.write_text(content, encoding="utf-8")
        return path

    def test_normal_apply_appends_entry_and_exits_zero(self) -> None:
        """正常なレビュー済みエントリは allowlist に追記され exit 0 になる。"""
        suggestions = self._write_suggestions(
            '[[exempt]]\n'
            'package = "example-css-pkg"\n'
            'rule = "R1-scripts"\n'
            'reason = "vendored postcss config only, no runtime execution"\n'
        )
        code, out = run_capture_main(
            ["--suggestions", str(suggestions), "--allowlist", str(self.allowlist_path)]
        )
        self.assertEqual(code, 0)
        self.assertIn("APPLIED package=example-css-pkg rule=R1-scripts", out)
        final_text = self.allowlist_path.read_text(encoding="utf-8")
        self.assertIn('package = "example-css-pkg"', final_text)
        self.assertIn('rule = "R1-scripts"', final_text)
        # 元のヘッダコメントも保持されていること（既存内容の破壊なし）。
        self.assertIn("fixture header comment", final_text)

    def test_todo_reason_is_rejected_with_exit_one(self) -> None:
        """雛形の TODO: reason のまま（未レビュー）のエントリは exit 1 で拒否され、
        allowlist ファイルは無変更のままである。"""
        before = self.allowlist_path.read_text(encoding="utf-8")
        suggestions = self._write_suggestions(
            '[[exempt]]\n'
            'package = "other-pkg"\n'
            'rule = "R1-bin"\n'
            'reason = "TODO: describe why this exemption is safe for this package"\n'
        )
        code, out = run_capture_main(
            ["--suggestions", str(suggestions), "--allowlist", str(self.allowlist_path)]
        )
        self.assertEqual(code, 1)
        self.assertIn("TODO", out)
        self.assertEqual(self.allowlist_path.read_text(encoding="utf-8"), before)

    def test_hard_deny_extension_is_rejected_with_exit_one(self) -> None:
        """実行コード拡張子（.js 等）への R2-ext 免除は適用側でも拒否される
        （ハード拒否の抜け道を allowlist 側からも構造的に塞ぐ、§3.4）。"""
        before = self.allowlist_path.read_text(encoding="utf-8")
        suggestions = self._write_suggestions(
            '[[exempt]]\n'
            'package = "evil-pkg"\n'
            'rule = "R2-ext"\n'
            'ext = ".js"\n'
            'reason = "trust me"\n'
        )
        code, out = run_capture_main(
            ["--suggestions", str(suggestions), "--allowlist", str(self.allowlist_path)]
        )
        self.assertEqual(code, 1)
        self.assertIn("hard-deny", out)
        self.assertEqual(self.allowlist_path.read_text(encoding="utf-8"), before)

    def test_duplicate_entry_is_skipped_idempotently(self) -> None:
        """既存 allowlist に同一キーのエントリが既にあれば、二重追記せず
        SKIPPED として exit 0 になる（冪等）。"""
        suggestions = self._write_suggestions(
            '[[exempt]]\n'
            'package = "dup-pkg"\n'
            'rule = "R1-bin"\n'
            'reason = "first application"\n'
        )
        code1, _ = run_capture_main(
            ["--suggestions", str(suggestions), "--allowlist", str(self.allowlist_path)]
        )
        self.assertEqual(code1, 0)
        after_first = self.allowlist_path.read_text(encoding="utf-8")

        code2, out2 = run_capture_main(
            ["--suggestions", str(suggestions), "--allowlist", str(self.allowlist_path)]
        )
        self.assertEqual(code2, 0)
        self.assertIn("SKIPPED", out2)
        # 二重追記されていない（内容が変わらない）こと。
        self.assertEqual(self.allowlist_path.read_text(encoding="utf-8"), after_first)
        self.assertEqual(after_first.count('package = "dup-pkg"'), 1)

    def test_invalid_toml_with_violation_lines_is_rejected_exit_two(self) -> None:
        """`--suggest-exempt` の生出力のように VIOLATION 行が混在したファイルは
        TOML として不正なため exit 2 で拒否される（レビューを経ていない生出力の
        誤投入を防ぐ設計）。"""
        before = self.allowlist_path.read_text(encoding="utf-8")
        suggestions = self._write_suggestions(
            'VIOLATION package=evil-pkg rule=R2-ext file=x.js reason="not allowed"\n'
            '[[exempt]]\n'
            'package = "evil-pkg"\n'
            'rule = "R2-ext"\n'
            'ext = ".xyz"\n'
            'reason = "ok"\n'
        )
        code, out = run_capture_main(
            ["--suggestions", str(suggestions), "--allowlist", str(self.allowlist_path)]
        )
        self.assertEqual(code, 2)
        self.assertIn("Error", out)
        self.assertEqual(self.allowlist_path.read_text(encoding="utf-8"), before)

    def test_unexpected_top_level_key_is_rejected_exit_two(self) -> None:
        """`[[exempt]]` 以外のトップレベルキーが混在するファイルは拒否される。"""
        suggestions = self._write_suggestions(
            'title = "not a valid suggestions file"\n'
            '[[exempt]]\n'
            'package = "pkg"\n'
            'rule = "R1-bin"\n'
            'reason = "ok"\n'
        )
        code, out = run_capture_main(
            ["--suggestions", str(suggestions), "--allowlist", str(self.allowlist_path)]
        )
        self.assertEqual(code, 2)
        self.assertIn("unexpected key", out)

    def test_dry_run_does_not_modify_allowlist(self) -> None:
        """--dry-run は検証・出力のみ行い、allowlist ファイルを変更しない。"""
        before = self.allowlist_path.read_text(encoding="utf-8")
        suggestions = self._write_suggestions(
            '[[exempt]]\n'
            'package = "dryrun-pkg"\n'
            'rule = "R1-bin"\n'
            'reason = "static wrapper only"\n'
        )
        code, out = run_capture_main(
            [
                "--suggestions",
                str(suggestions),
                "--allowlist",
                str(self.allowlist_path),
                "--dry-run",
            ]
        )
        self.assertEqual(code, 0)
        self.assertIn("dry-run", out)
        self.assertEqual(self.allowlist_path.read_text(encoding="utf-8"), before)

    def test_escapes_special_characters_in_written_entry(self) -> None:
        """package 名にダブルクォート・バックスラッシュを含む場合でも、
        書き込まれる TOML が破壊されずパース可能であること
        （_escape_toml_string の再利用を検証する）。"""
        suggestions = self._write_suggestions(
            '[[exempt]]\n'
            'package = "weird\\\\pkg\\"name"\n'
            'rule = "R1-bin"\n'
            'reason = "contains \\"quotes\\" and a backslash \\\\"\n'
        )
        code, _ = run_capture_main(
            ["--suggestions", str(suggestions), "--allowlist", str(self.allowlist_path)]
        )
        self.assertEqual(code, 0)

        import tomllib

        data = tomllib.loads(self.allowlist_path.read_text(encoding="utf-8"))
        entries = data.get("exempt", [])
        self.assertEqual(len(entries), 1)
        self.assertEqual(entries[0]["package"], 'weird\\pkg"name')

    def test_no_entries_in_suggestions_is_a_noop_exit_zero(self) -> None:
        """空の [[exempt]] 配列（該当違反なし）は何もせず exit 0 になる。"""
        suggestions = self._write_suggestions("")
        before = self.allowlist_path.read_text(encoding="utf-8")
        code, out = run_capture_main(
            ["--suggestions", str(suggestions), "--allowlist", str(self.allowlist_path)]
        )
        self.assertEqual(code, 0)
        self.assertIn("nothing to apply", out)
        self.assertEqual(self.allowlist_path.read_text(encoding="utf-8"), before)

    def test_missing_allowlist_file_is_created(self) -> None:
        """allowlist.toml がまだ存在しない場合でも新規作成して適用できる。"""
        missing_path = self.tmp_path / "new_allowlist.toml"
        self.assertFalse(missing_path.exists())
        suggestions = self._write_suggestions(
            '[[exempt]]\n'
            'package = "fresh-pkg"\n'
            'rule = "R1-bin"\n'
            'reason = "new allowlist created from scratch"\n'
        )
        code, _ = run_capture_main(
            ["--suggestions", str(suggestions), "--allowlist", str(missing_path)]
        )
        self.assertEqual(code, 0)
        self.assertTrue(missing_path.exists())
        self.assertIn('package = "fresh-pkg"', missing_path.read_text(encoding="utf-8"))


if __name__ == "__main__":
    unittest.main()
