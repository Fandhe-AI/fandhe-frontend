# TASK-6.3c 実証実行記録（#67）

## 位置づけ

TASK-6.3（#64）の 4h 単位分割サブタスクのうち、TASK-6.3c「実証実行と不具合修正」の実施記録です。
TASK-6.3b（`wasm-client/tests/hydration_browser.rs` の実装、PR #241 でマージ済み）の成果物を
実ブラウザで実行し、不具合の有無を確認します。

条件 1（Conditional Go）の解消判定・総括レポートは TASK-6.3d（#68）の成果物
`docs/reports/hydration-browser-report.md` を正とします。本ドキュメントは実行の一次記録であり、
判定文書ではありません。

## 実行内容

```
$ wasm-pack test --headless --chrome
```

（`wasm-client/` 配下、CHROMEDRIVER=/usr/bin/chromedriver、ブラウザ: /snap/bin/chromium）

## 結果

| 対象 | 件数 | 結果 |
|------|------|------|
| `tests/hydrate_smoke.rs`（TASK-6.2c） | 3 | 全 pass |
| `tests/hydration_browser.rs`（TASK-6.3b） | 6 | 全 pass |
| doc-tests（`src/lib.rs`） | 4 | 全 pass |

`hydration_browser.rs` の 6 テスト内訳（いずれも pass）:

- `mount_csr_reflects_same_render_output_as_ssr`
- `hydrate_does_not_rebuild_server_rendered_dom`
- `hydrate_toggles_liked_class_on_click_and_untoggles_on_second_click`
- `hydrate_preserves_pre_existing_liked_state`
- `re_hydrate_preserves_click_state_and_fires_exactly_once`
- `xss_payload_item_does_not_produce_script_element_in_real_dom`

## 不具合修正

今回の実行で新規の不具合は検出されませんでした。PR #241（TASK-6.3b 実装時）で修正済みの
2 件（ロケータのコンテナスコープ化、`hydrate()` 状態保持テストの stale DOM 参照修正）は
本ブランチに継承済みで、再発は確認されていません。

## 補足（ハーネス挙動）

`wasm-pack test --headless --chrome` 実行時に `Try find webdriver.json ... Not found` の
INFO ログが出力されますが、`webdriver.json` は capabilities のオプショナルなカスタマイズ
ファイルであり、未配置時はデフォルト capabilities にフォールバックするだけで、テスト結果
（全 pass）に影響はありません。不具合ではないため対応不要です。
