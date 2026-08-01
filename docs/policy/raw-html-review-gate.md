# `raw_html()` レビューゲート運用ガイド（イシュー #157/#159/#299/#315）

`fandhe_frontend_core::raw_html()` は REQ-1（既定エスケープ）の唯一の許容迂回経路である。
本ドキュメントは、その使用をレビュー済みとして宣言する手順と、`fw gate` /
CI が構成する 3 層の検出体制を説明する。方式選定の背景・脅威モデルは
`docs/design/raw-html-lint-design.md` を参照。CI `clippy` ジョブの `--all-targets`
拡張（テストターゲット内呼び出しの検出対象化）はイシュー #299 で行い、
`fw gate` の `lint` チェックへの同拡張（検出境界一致）はイシュー #315 で
行った。

## 1. オプトインの書式

`raw_html()` を呼び出す文の直前（または同一行）に、`clippy::disallowed_methods`
への `#[expect(...)]` 属性を付与する。

```rust
pub fn render_trusted_fragment() -> String {
    #[expect(clippy::disallowed_methods, reason = "ESCAPE-REVIEWED: 固定の信頼済み HTML 片。外部入力を含まない")]
    let node = fandhe_frontend_core::raw_html("<b>trusted</b>");
    fandhe_frontend_core::render(&node)
}
```

- `reason` 文字列には必ず `ESCAPE-REVIEWED:` から始まるレビュー根拠を書く
  （「なぜ既定エスケープを迂回してよいか」を後続の読み手・レビュアーが
  ソース上だけで判断できるようにする、`.claude/rules/code-comment-style.md`
  準拠）。
- 属性は Rust の statement 属性として呼び出し文（式文・`let` 文）へ直接付与
  できる（`rustc 1.96.0` で動作確認済み。関数全体に付与してもよいが、
  影響範囲を呼び出し 1 件に絞るため呼び出し文への付与を推奨する）。
- `rustfmt` による折り返しは受理される（イシュー #1116）。`reason = "..."` が
  長く rustfmt が属性を複数行へ折り返しても（例: `#[expect(\n
  clippy::disallowed_methods,\n    reason = "ESCAPE-REVIEWED: ..."\n)]`）、
  `default_escape_check`（保険層）は `[`/`]` の括弧バランスで属性グループを
  判定するため `#[rustfmt::skip]` の追加は不要。`#[rustfmt::skip]` 等の
  重ね掛けも、スタック中のいずれかの属性グループが両マーカーを含めば
  受理される。属性と呼び出しの間に空行・コメント・無関係なコード行を挟んだ
  場合は受理されない。
- `#[allow(clippy::disallowed_methods)]` は使わない。`#[expect]` は「lint が
  実際に発火することを期待する」属性であり、対応する `raw_html()` 呼び出しが
  将来削除・変更されて lint が発火しなくなった場合に
  `unfulfilled_lint_expectations` として警告が出る。レビュー済みマーカーが
  実体を失ったまま放置される（陳腐化）ことを防ぐため、`#[allow]` ではなく
  必ず `#[expect]` を使うこと。

## 2. 禁止事項: ブランケット抑止

ファイル・モジュール冒頭に次のような内部属性を書き、ファイル全体で
`disallowed_methods` を無効化することは禁止する。

```rust
#![allow(clippy::disallowed_methods)]   // 禁止
#![expect(clippy::disallowed_methods)]  // 禁止
```

これらは呼び出し個別のレビューではなく、主防御そのものの一括無効化に相当する。
`fw gate` の `default_escape_check` はこれを独立の違反として検出し、
`file:line` 付きで報告する（`crates/cli/src/gate.rs` の
`BLANKET_DISALLOWED_METHODS_MARKERS`）。レビューが必要な呼び出しには、
必ず 1. の呼び出し単位の `#[expect(...)]` を使うこと。

## 3. 3 層の検出体制

| 層 | 実行タイミング | 実体 |
|----|---------------|------|
| 主防御 | `fw gate` の `lint` チェック／CI `clippy` ジョブ | `fw gate` の `lint` はイシュー #315 で `cargo clippy --locked --all-targets -p <crate>... -- -D warnings` へ拡張済み。CI `clippy` ジョブはイシュー #299 で `cargo clippy --workspace --all-targets --locked -- -D warnings` へ拡張済み。両者とも `--all-targets` によりテストターゲット（`#[cfg(test)]` / `tests/` 配下）内の呼び出しを検出対象に含み、検出範囲は一致する（旧記述「CI が superset」は解消済み）。いずれも workspace ルート `clippy.toml`（`disallowed-methods`）に基づき、コンパイラのパス解決を通じてコメント偽装・リネーム import 経由の呼び出しも検出する |
| 保険層 | `fw gate` の `default_escape_check` | テキスト走査。呼び出し開始行自体、または呼び出し直前に隙間なく連なる属性グループ列のいずれか 1 つに `#[expect(clippy::disallowed_methods, reason = "ESCAPE-REVIEWED: ...")]` が含まれることを受理条件とする（rustfmt の複数行折り返し・`#[rustfmt::skip]` 等の重ね掛けも受理、イシュー #1116。単独のコメントは受理しない） |
| 監査層 | `fw gate` の `default_escape_check` | ブランケット抑止属性の一律検出（2. 参照） |

`lint` チェックは `clippy.toml` の存在・`fandhe_frontend_core::raw_html` エントリの包含を
起動前に検証し、欠落時は `cargo clippy` を起動せず即 failed とする
（`clippy.toml` が消されると検出が沈黙し黙示的 PASS になる穴を塞ぐ、
security.md A05 fail-closed）。

## 4. 新規プロジェクトへの波及

`fw` が生成する標準プロジェクトテンプレート（`templates/default/`）には
workspace ルートと同一内容の `clippy.toml` を同梱済みである
（`templates/default/clippy.toml`）。生成直後の空プロジェクトの段階から
ポリシーが有効になっており、`fandhe-frontend-core` への依存・`raw_html()` の使用を
始めた時点で検出が機能する。

## 5. レビュー手順

1. `raw_html()` の使用が必要になった場合、まず既定エスケープ経由（`text()`/
   `el()` 等）で代替できないかを検討する。
2. 代替できない場合、呼び出し文に `#[expect(clippy::disallowed_methods,
   reason = "ESCAPE-REVIEWED: <根拠>")]` を付与する。
3. コミット・PR 作成前に security-auditor によるレビュー（`reason` の妥当性・
   信頼できない入力を渡していないかの確認）を必須とする
   （`.claude/rules/security.md`）。
4. `fw gate --project .` を実行し、`lint`・`default_escape_check` の両方が
   `passed: true` であることを確認する。
