#!/usr/bin/env python3
"""apply_exempt.py

役割: `check_static_only.py --suggest-exempt` が出力する `[[exempt]]` 雛形を、
**人間によるレビュー後**に allowlist.toml へ半自動で追記する独立コマンド
（イシュー #316・docs/npm-asset-build.md §3.4）。

背景: PR #311（イシュー #296）で導入された `--suggest-exempt` は「提案の
出力のみ・allowlist.toml への自動書き込みなし」という fail-closed 方針を
意図的に選んだ（A08 観点: チェッカー自身が免除機構を自動拡張できると、
サイレントな野放図拡大の経路になり得るため）。本コマンドはその方針を
変えず、"人間が提案をレビュー・編集して保存したファイルを、機械的な
ケアレスミス（転記漏れ・エスケープ崩れ）なしに反映する" ワンコマンドを
提供するだけであり、`check_static_only.py` 本体・`install.sh` からは
一切呼び出されない（自動連携しない）。

入力契約:
  - `--suggestions` はファイル全体が有効な TOML であることを要求する。
    `--suggest-exempt` の生出力（`VIOLATION ...` 行や hard-deny 注記等が
    混在する）をそのまま渡すと tomllib のパースに失敗し exit 2 で拒否
    される。これは意図的な設計であり、人間が VIOLATION 行を取り除き・
    reason を編集して保存する「レビューを経た」ファイルだけを受理する
    ためのゲートである。
  - トップレベルは `[[exempt]]` 配列のみを許可する（他のキーが混在する
    ファイルは拒否）。

検証:
  - エントリ検証は `check_static_only.validate_exempt_entries()` を再利用
    する（同一ディレクトリの import。検証規則の二重実装によるドリフトを
    避けるため、本ファイルではルールを再実装しない）。
  - `reason` が空・空白のみ・`TODO:` で始まる雛形のまま（未レビュー）の
    エントリは拒否する（人間レビューを強制する）。
  - ハード拒否拡張子（`.js`/`.mjs`/`.cjs`/`.node`/`.wasm`）への免除は
    `validate_exempt_entries()` 側で構造的に拒否される。

書き込み:
  - 既存 allowlist ファイルの生テキストへ、新規エントリの TOML 断片を
    追記した内容を組み立て、その内容を再度 tomllib でパースし
    `validate_exempt_entries()` で再検証してから書き込む（マージ後の
    内容が壊れていないことを書き込み前に確認する）。
  - 既存 allowlist に同一キー（package, rule, ext/file）のエントリが
    既にあれば SKIPPED として重複追記しない（冪等）。
  - 検証に 1 件でも失敗した場合、対象ファイルは一切変更しない
    （部分適用・破損 allowlist を残さない）。
  - 実ファイルへの反映は同一ディレクトリの一時ファイル経由で
    `os.replace()` によりアトミックに行う。

終了コード契約（check_static_only.py と同じ契約に揃える）:
  0 = 適用完了（適用 0 件の冪等成功を含む）
  1 = エントリ検証で拒否あり（提案ファイルに修正が必要）
  2 = 実行エラー（ファイル不在・TOML 構文エラー等）

テンプレート同梱（イシュー #316）: 本ファイルは
templates/default/tools/npm-asset-build/ へバイト同一のままコピー同梱される。
正本はこのファイル（tools/npm-asset-build/）であり、変更時は
tools/npm-asset-build/tests/test_template_sync.sh がドリフトを検知する。
"""

from __future__ import annotations

import argparse
import os
import sys
import tempfile
from pathlib import Path
from typing import Any

# check_static_only.py と同一ディレクトリに置かれる前提の姉妹スクリプト。
# 検証ロジック（validate_exempt_entries）・TOML エスケープ（_escape_toml_string）・
# 定数（VALID_RULE_IDS 等）を re-implement せず再利用する。
sys.path.insert(0, str(Path(__file__).resolve().parent))
import check_static_only as csc  # noqa: E402


class ApplyError(Exception):
    """実行エラー（exit 2）を表す。ファイル不在・TOML 構文エラー等、
    書き込み先・入力ファイルの状態自体を判定できない場合に送出する。"""


class ValidationRejected(Exception):
    """エントリ検証での拒否（exit 1）を表す。ファイル自体は正しい TOML だが、
    人間レビュー未了（TODO: reason）・fail-closed 規則違反等、提案内容に
    修正が必要な場合に送出する（check_static_only.py の exit 1 契約と揃える）。"""


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "Apply human-reviewed [[exempt]] suggestions (from "
            "check_static_only.py --suggest-exempt) into an allowlist.toml."
        )
    )
    parser.add_argument(
        "--suggestions",
        required=True,
        help="Path to a TOML file containing reviewed [[exempt]] entries.",
    )
    parser.add_argument(
        "--allowlist",
        required=True,
        help="Path to the allowlist.toml to update.",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Validate and print what would change, without writing the allowlist file.",
    )
    return parser.parse_args(argv)


def _load_toml_file(path: Path, *, label: str) -> dict[str, Any]:
    """TOML ファイルを読み込む。allowlist・suggestions の両方から共有される
    薄いラッパーで、check_static_only.load_allowlist と異なりファイル内容の
    dict をそのまま返す（呼び出し元がキー構成の妥当性を追加検証するため）。
    """
    try:
        import tomllib
    except ImportError as exc:  # Python 3.10 以下には tomllib が存在しない
        raise ApplyError(
            "apply_exempt.py requires Python 3.11+ (tomllib not available in this interpreter)"
        ) from exc

    try:
        with open(path, "rb") as f:
            return tomllib.load(f)
    except OSError as exc:
        raise ApplyError(f"failed to read {label} file {path}: {exc}") from exc
    except Exception as exc:  # tomllib.TOMLDecodeError 等
        raise ApplyError(
            f"failed to parse {label} file {path} as TOML: {exc} "
            "(hint: raw --suggest-exempt output containing 'VIOLATION ...' lines "
            "is not valid TOML — review and edit it first, per docs/npm-asset-build.md §3.4)"
        ) from exc


def _entry_key(entry: dict[str, Any]) -> tuple[str, str, tuple[str, str] | None]:
    """検証済みエントリから重複判定用のキーを組み立てる。

    正規化ロジック（ext の大文字小文字・file の区切り文字統一）自体は
    `check_static_only.exempt_entry_key()` に一本化されている（イシュー #316
    レビュー指摘: 本ファイルで独自に再実装すると `validate_exempt_entries()`
    側の正規化規則が将来変わった際にドリフトし得るため、ここでは正規化済みの
    ext/file を組み立てて渡すだけにとどめる）。
    """
    package = entry["package"]
    rule = entry["rule"]
    ext_field = entry.get("ext")
    file_field = entry.get("file")
    ext_norm = ext_field.strip().lower() if isinstance(ext_field, str) and ext_field.strip() else None
    file_norm = str(file_field).strip().replace("\\", "/") if ext_norm is None and file_field is not None else None
    return csc.exempt_entry_key(package, rule, ext_norm, file_norm)


def _render_exempt_block(entry: dict[str, Any]) -> str:
    """検証済みエントリ 1 件を allowlist.toml へ追記する TOML 断片へ整形する。
    値の埋め込みは check_static_only._escape_toml_string を再利用し、
    文字列連結による TOML 生成（インジェクション温床）を避ける。
    """
    lines = ["[[exempt]]"]
    lines.append(f'package = "{csc._escape_toml_string(entry["package"])}"')
    lines.append(f'rule = "{csc._escape_toml_string(entry["rule"])}"')
    if entry["rule"] == "R2-ext":
        ext_field = entry.get("ext")
        if isinstance(ext_field, str) and ext_field.strip():
            lines.append(f'ext = "{csc._escape_toml_string(ext_field.strip())}"')
        else:
            lines.append(f'file = "{csc._escape_toml_string(str(entry.get("file")).strip())}"')
    lines.append(f'reason = "{csc._escape_toml_string(entry["reason"].strip())}"')
    return "\n".join(lines) + "\n"


def apply_exempt(suggestions_path: Path, allowlist_path: Path, *, dry_run: bool) -> tuple[int, str]:
    """本体処理。(exit_code, message) を返す（main() が stdout/stderr へ振り分ける）。"""
    suggestions_data = _load_toml_file(suggestions_path, label="suggestions")
    unknown_keys = set(suggestions_data.keys()) - {"exempt"}
    if unknown_keys:
        raise ApplyError(
            f"suggestions file must contain only a top-level 'exempt' array; "
            f"found unexpected key(s): {sorted(unknown_keys)!r} "
            "(this usually means raw --suggest-exempt output was pasted without review)"
        )
    raw_new_entries = suggestions_data.get("exempt", [])
    # 「exempt キーが未指定・空配列」（適用対象なし、正常系）と「exempt が
    # 存在するがリスト以外の型（文字列・テーブル等、明確な不正形式）」を
    # 混同しない。後者は他の不正 suggestions 入力と同様に fail-closed で
    # ApplyError（exit code 2）として拒否する（イシュー #316 Bugbot 指摘対応）。
    if not isinstance(raw_new_entries, list):
        raise ApplyError(
            "suggestions file's top-level 'exempt' key must be an array of tables; "
            f"found type {type(raw_new_entries).__name__}"
        )
    if not raw_new_entries:
        return 0, "No [[exempt]] entries found in suggestions file; nothing to apply."

    # 未レビュー（reason が空・雛形の TODO: のまま）のエントリは、検証エラー
    # として明示的に拒否する（validate_exempt_entries は reason の非空しか
    # 見ないため、ここで人間レビューの強制を追加する）。
    for entry in raw_new_entries:
        if not isinstance(entry, dict):
            continue
        reason = entry.get("reason")
        if isinstance(reason, str) and reason.strip().upper().startswith("TODO:"):
            raise ValidationRejected(
                f"entry for package={entry.get('package')!r} rule={entry.get('rule')!r} "
                "still has a template 'TODO: ...' reason — edit it to describe why the "
                "exemption is safe before applying (human review required)"
            )

    # 新規エントリ単体の検証（既存 allowlist と無関係に、まずそれ自体が
    # fail-closed 規則を満たすかを確認する）。
    try:
        csc.validate_exempt_entries(raw_new_entries)
    except csc.CheckError as exc:
        raise ValidationRejected(f"suggestions file failed validation: {exc}") from exc

    if allowlist_path.exists():
        allowlist_raw_text = allowlist_path.read_text(encoding="utf-8")
    else:
        allowlist_raw_text = ""
    allowlist_data = _load_toml_file(allowlist_path, label="allowlist") if allowlist_path.exists() else {}
    existing_raw_entries = allowlist_data.get("exempt", []) if allowlist_path.exists() else []
    try:
        existing_exemptions = csc.validate_exempt_entries(existing_raw_entries)
    except csc.CheckError as exc:
        raise ApplyError(f"existing allowlist file failed validation: {exc}") from exc

    existing_keys = set(existing_exemptions.keys())
    new_keys_seen: set[tuple[str, str, tuple[str, str] | None]] = set()
    blocks_to_append: list[str] = []
    applied: list[str] = []
    skipped: list[str] = []
    for entry in raw_new_entries:
        key = _entry_key(entry)
        label = f"package={entry.get('package')} rule={entry.get('rule')}"
        if key in existing_keys or key in new_keys_seen:
            skipped.append(label)
            continue
        new_keys_seen.add(key)
        blocks_to_append.append(_render_exempt_block(entry))
        applied.append(label)

    if not blocks_to_append:
        return 0, f"Nothing new to apply (all {len(skipped)} entrie(s) already present; SKIPPED)."

    # 既存本文の末尾トレイリング改行を正規化してから、空行 1 行を挟んで新規
    # ブロックを追記する（既存ファイルの改行有無に関わらず結合結果を安定させる）。
    if allowlist_raw_text.strip():
        merged_text = allowlist_raw_text.rstrip("\n") + "\n\n" + "\n".join(blocks_to_append)
    else:
        merged_text = "\n".join(blocks_to_append)

    # 書き込み前の最終再検証: マージ後のテキストが有効な TOML であり、かつ
    # validate_exempt_entries を通ることを、実ファイルへ触れる前に確認する
    # （ここで失敗すれば allowlist_path は一切変更されない）。
    try:
        import tomllib

        reparsed = tomllib.loads(merged_text)
    except Exception as exc:  # tomllib.TOMLDecodeError 等
        raise ApplyError(f"internal error: merged allowlist content failed to re-parse as TOML: {exc}") from exc
    try:
        csc.validate_exempt_entries(reparsed.get("exempt", []))
    except csc.CheckError as exc:
        raise ApplyError(f"internal error: merged allowlist content failed re-validation: {exc}") from exc

    summary_lines = [f"APPLIED {label}" for label in applied] + [f"SKIPPED {label} (duplicate)" for label in skipped]
    message = "\n".join(summary_lines)

    if dry_run:
        return 0, message + "\n(dry-run: allowlist file not modified)"

    # 一時ファイル経由の os.replace によるアトミック置換。検証失敗時は
    # ここに到達しないため、対象ファイルは常に「無変更」か「完全に新しい
    # 内容」のいずれかであり、部分書き込みの破損 allowlist を残さない。
    allowlist_path.parent.mkdir(parents=True, exist_ok=True)
    fd, tmp_path_str = tempfile.mkstemp(
        dir=str(allowlist_path.parent), prefix=f".{allowlist_path.name}.", suffix=".tmp"
    )
    tmp_path = Path(tmp_path_str)
    try:
        with os.fdopen(fd, "w", encoding="utf-8") as f:
            f.write(merged_text)
        os.replace(tmp_path, allowlist_path)
    except BaseException:
        tmp_path.unlink(missing_ok=True)
        raise

    return 0, message


def main() -> None:
    args = parse_args(sys.argv[1:])
    try:
        exit_code, message = apply_exempt(
            Path(args.suggestions), Path(args.allowlist), dry_run=args.dry_run
        )
    except ValidationRejected as exc:
        print(f"Rejected: {exc}", file=sys.stderr)
        sys.exit(1)
    except ApplyError as exc:
        print(f"Error: {exc}", file=sys.stderr)
        sys.exit(2)
    print(message)
    sys.exit(exit_code)


if __name__ == "__main__":
    main()
