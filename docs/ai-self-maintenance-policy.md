# 自動適用と人間承認の境界ルール（TASK-13.6a・草案）

## 0. 本書のステータス

**本書は草案（DRAFT）です。** TASK-13.6（親イシュー #151、成果物本書）の
4h 分割の前半 **TASK-13.6a（本イシュー #152）** の成果物であり、境界ルール
そのものの**確定判断（人間レビューの反映）は後続の TASK-13.6b（#153）**が
担う。本書の記述はレビューを経て変更されうる。特に §7「未確定事項」に
列挙した論点は、#153 で人間が最終判断する前提で草案化している。

## 1. 目的とトレーサビリティ

- **関連要件**: REQ-13（AI 自己保守・改修のためのフック・ゲート機構、Must、
  `docs/spec/04-requirements.md`）の受け入れ基準 5 項目目:

  > 自動適用と人間承認の境界ルール（ゲート未通過は無条件拒否、
  > `breaking_risk: low` かつ影響ルートなしは自動適用候補、それ以外は
  > 人間承認必須）が、フレームワークの標準ドキュメントとして明文化されて
  > いること（PoC-7 の境界ルール節を踏襲）。

- **親タスク**: TASK-13.6「自動適用と人間承認の境界ルール明文化」
  （親イシュー #151、`docs/spec/05-tasks.md` 376 行目、成果物本書、
  担当区分「共同」— 境界ルールの妥当性は運用リスクに直結するため人間が
  確定し、文書化を共同で行う）。
- **原典**: `docs/spec/03-poc/ai-self-maintenance/README.md`
  「自動適用と人間承認の境界ルール」節（PoC-7 成功基準 4・疑問点 13 の
  解消として提案されたルール 4 項目）。
- **先行実装・関連設計文書**:
  - `docs/impact-analysis-design.md`（TASK-13.2a、`fw impact` の
    `breaking_risk` / `requires_human_approval` / `ambiguous` 判定設計）
  - `docs/gate-design.md`（TASK-13.3a、`fw gate` の 5 チェック・
    `gate_result` 判定設計。§4 が「本書（TASK-13.6）がゲートの終了コード・
    `gate_result` 値をそのまま前提として参照する」ことを既に明記している）
  - `docs/raw-html-review-gate.md`（`raw_html()` レビュー運用）
  - `docs/scenario-regression-design.md`（TASK-13.4、代表的改修シナリオ
    3 件の回帰実績）
  - `docs/cargo-deny-advisories.md`（`policy` チェックの `advisories`
    オフライン除外の扱い）

## 2. 用語と前提

本書は `fw structure` / `fw impact` / `fw gate`（`cli/` = `rws-cli`、
bin 名 `fw`）が出力する構造化データを判定材料とする。用語はすべて
既存設計文書で確定済みの定義をそのまま用い、本書で再定義しない
（判定ロジックの重複実装・二重管理を避けるため）。

| 用語 | 定義元 | 意味 |
|------|--------|------|
| `gate_result` | `docs/gate-design.md` §4、`cli/src/gate.rs` `aggregate` | `fw gate` の 5 チェック（`type_check` / `default_escape_check` / `lint` / `test` / `policy`）が**すべて**通過すれば `"PASS"`、1 件でも不合格なら `"BLOCKED"`。早期打ち切りはなく常に 5 チェック全件を実行する |
| `breaking_risk` | `docs/impact-analysis-design.md` §3.4、`cli/src/impact.rs` `judge_breaking_risk` | `fw impact <symbol>` が返す破壊的変更リスクの 3 段階分類（`high` / `medium` / `low`）。影響クレート数とクライアント境界クレート（`rws-wasm-client` / `rws-wasm-full` / `rws-wasm-thin`）への波及有無から決まる |
| `affected_routes` | `docs/impact-analysis-design.md` §3.5 | `fw impact` が返す、変更対象シンボルの影響を受けるルート定義一覧（ファイル単位の粗粒度突き合わせ）。空配列であれば「影響ルートなし」 |
| `ambiguous` | `docs/impact-analysis-design.md` §3.2 | 変更対象シンボルの定義元が複数見つかった状態（`true`）。定義元特定の前提が崩れているため、他の判定材料の信頼性も下がる |
| `requires_human_approval` | `docs/impact-analysis-design.md` §3.4、`cli/src/impact.rs` `requires_human_approval` | `fw impact` が返す単一の真偽値。`breaking_risk ∈ {high, medium}` または `affected_routes` 非空 または `ambiguous` のいずれかで `true` |

## 3. 境界ルール本体

REQ-13 受け入れ基準の 3 区分をそのまま境界ルールとして固定する。
**判定材料は `ImpactReport::requires_human_approval`（`fw impact` の
JSON 出力の `requires_human_approval` フィールド）を単一の情報源とし、
本書やそれを利用する AI エージェント側で判定ロジックを再実装しない**
（`docs/impact-analysis-design.md` の「判定材料は
`ImpactReport::requires_human_approval` のみを使う」契約と同一の方針を、
運用ルールの側でも踏襲する）。

### ルール 1: ゲート未通過 → 無条件拒否

`fw gate` の `gate_result` が `"BLOCKED"` の変更は、**承認フロー以前の
段階で無条件に拒否**する。人間の承認では覆せない。

- 対象: 型エラー（`type_check`）・未エスケープ出力（`default_escape_check`
  / `lint` の `raw_html()` 未レビュー呼び出し検出）・禁止依存の追加
  （`policy`）・テスト失敗（`test`）のいずれか 1 件でも不合格であれば
  該当する。
- 根拠: これらはいずれも機械的に検出可能であり、人間承認を介する必要すら
  ない最も低コストな安全弁である（PoC-7 原典の境界ルール 1）。
- **承認フローがエスケープ検査のバイパス経路にならないことの明示**:
  `gate_result: BLOCKED` を人間が「承認」することで変更を適用する経路は
  存在しない。`fw gate` を再実行して `PASS` に変えることでのみ次段階
  （ルール 2/3 の判定）に進める。

### ルール 2: ゲート通過 かつ breaking_risk: low かつ 影響ルートなし かつ ambiguous でない → 自動適用「候補」

`gate_result: "PASS"` かつ `fw impact` の判定結果が
`requires_human_approval: false`（= `breaking_risk: "low"` かつ
`affected_routes` が空配列かつ `ambiguous: false`）である変更は、
**自動適用候補**とする。

- 該当例（設計上の想定。`docs/spec/03-poc/ai-self-maintenance/README.md`
  の 3 シナリオではこの区分に実例はなかった）: 単一クレート内で完結し、
  他クレートのソースを一切参照されない変更（例: 内部ヘルパー関数の実装
  詳細変更で、影響が同一クレート内・非公開ルートに限定されテストが通る
  もの）。
- **既定動作は「候補提示のみ」とする**（草案時点の安全側の提案。
  §7 未確定事項を参照）。すなわち、「自動適用候補」と判定された変更を
  無人で実適用まで進めるか、常に人間の最終トリガーを要求するかは、
  本草案では後者（候補提示に留める）を既定として提案し、確定判断は
  #153 に引き継ぐ。

### ルール 3: それ以外 → 人間承認必須

`gate_result: "PASS"` だが `requires_human_approval: true`
（= `breaking_risk` が `medium` / `high`、または `affected_routes` が
非空、または `ambiguous`）の変更は、**人間の承認を必須**とする。

- `docs/spec/03-poc/ai-self-maintenance/README.md` の 3 シナリオ
  （バグ修正・UI 改善・機能追加）はすべてこの区分に該当した実績を持つ
  （コンポーネント境界・ルーティングに触れる変更は本質的にレビュー対象に
  なりやすいため、想定どおりの結果）。
- `ambiguous: true`（定義元多重）の場合は、`breaking_risk` や
  `affected_routes` の値にかかわらず常にこの区分へ倒す（fail-closed。
  `docs/impact-analysis-design.md` §3.2 の設計判断をそのまま踏襲）。

### 判定表（要約）

| `gate_result` | `requires_human_approval` | 区分 | 扱い |
|---|---|---|---|
| `BLOCKED` | — | ルール 1 | 無条件拒否（人間承認では覆せない） |
| `PASS` | `false`（= `breaking_risk: low` かつ `affected_routes` 空 かつ `ambiguous` でない） | ルール 2 | 自動適用候補（既定: 候補提示のみ、§7 参照） |
| `PASS` | `true` | ルール 3 | 人間承認必須 |

## 4. 運用フロー（AI エージェント視点）

AI エージェントがプロダクトの補修・改善・機能追加を行う際の想定フロー
（機構の提供のみが REQ-13 のスコープであり、AI エージェント本体の実装は
対象外。本書はこのフローが従うべき境界ルールを定めるものであり、
エージェントの実装仕様書ではない）。

```
1. fw impact <symbol> を実行し、breaking_risk / affected_routes /
   ambiguous / requires_human_approval を取得する
2. 変更を適用する（ワーキングツリーへの反映。適用前に impact を取ることで
   変更前の影響範囲を把握する。適用後に fw gate を実行する運用を想定）
3. fw gate --project <dir> を実行し、gate_result を取得する
4. gate_result == "BLOCKED" → ルール 1（無条件拒否）。
   AI エージェントへ差し戻し、報告された failing checks を修正して 3 に戻る
5. gate_result == "PASS" かつ requires_human_approval == false
   → ルール 2（自動適用候補として提示。実適用の可否は §7 の未確定事項）
6. gate_result == "PASS" かつ requires_human_approval == true
   → ルール 3（人間承認を要求し、承認を得るまで適用を保留する）
```

## 5. 実装トレーサビリティ

| 本書のルール | 対応実装 |
|---|---|
| ルール 1（ゲート未通過は無条件拒否） | `cli/src/gate.rs` `run_gate`（`gate_result` の生成）・`aggregate`（5 チェック全件実行・1 件でも不合格なら `BLOCKED`）。回帰: `cli/tests/gate_integration.rs`・`cli/tests/negative_cases.rs`（`type_error_blocks_gate_with_type_check_failure` / `unescaped_raw_html_call_blocks_gate_with_escape_check_failure` / `banned_dependency_blocks_gate_with_policy_failure`） |
| ルール 2・3 の判定材料（`breaking_risk` / `affected_routes` / `ambiguous` / `requires_human_approval`） | `cli/src/impact.rs` `judge_breaking_risk`・`requires_human_approval`（純粋関数）。回帰: `cli/src/impact.rs` 内ユニットテスト（`judge_breaking_risk_*`）、`docs/impact-analysis-design.md` §3.4 判定境界一覧 |
| 3 シナリオでの区分実績（ルール 3 該当の実例） | `cli/tests/scenarios/`（TASK-13.4b/c/d、`docs/scenario-regression-design.md`） |
| `policy` チェックの `advisories` オフライン除外 | `docs/cargo-deny-advisories.md`（ルール 1 の `policy` チェックは `bans`/`licenses`/`sources` のみをオフラインで実行し、`advisories` は CI 側のネットワークアクセス前提で補完する） |
| 既定エスケープ検査の 3 層体制 | `docs/gate-design.md` §2.2（`#[expect(clippy::disallowed_methods, reason = "ESCAPE-REVIEWED: ...")]` 属性 + `clippy.toml` `disallowed-methods` + ブランケット抑止監査。PoC-7 のマーカーコメント方式はイシュー #157 の指摘で偽装可能と判明し廃止済み） |

## 6. 限界の明示

`docs/spec/04-requirements.md` REQ-13 の「制約・宿題」および
`docs/gate-design.md` §7・`docs/impact-analysis-design.md` §7 の限界を、
運用ルールの利用者向けに改めて明示する（過度な安全性訴求を避ける観点、
PoC-2 の要件示唆と整合）。

- **ゲート通過は安全性の「必要条件」であって「十分条件」ではない**。
  `fw gate` が検出できるのは構文・ポリシー・エスケープ・テスト網羅の
  範囲に限定され、**意味的な脆弱性・ロジック誤りの検出はスコープ外**
  である（PoC-2・PoC-7 の結論）。ルール 2（自動適用候補）・ルール 3
  （人間承認必須）のいずれの区分でも、「ゲートを通過した」ことは
  「意味的に正しい」ことを保証しない。
- **影響範囲判定はファイル単位の粗粒度ヒューリスティック**である。
  シンボル単位で「どのルートハンドラが実際にそのシンボルを呼ぶか」まで
  は追跡せず、無関係なルートも「影響あり」として過検知しうる（安全側の
  設計）。AST 解析ベースの精密化（関数呼び出しグラフ解析）は将来スコープ
  であり本書の対象外（`docs/impact-analysis-design.md` §7）。
- **AI エージェント本体の実装は REQ-13 の対象外**。本書は機構（フック・
  ゲート）が従うべき境界ルールの提供に限定される。
- **`cargo-deny` の `advisories`（既知脆弱性 DB 照合）はオフラインゲート
  対象外**。`policy` チェックは `bans`/`licenses`/`sources` に限定され、
  `advisories` は CI 環境でのネットワークアクセスを前提に別途補完する
  （`docs/cargo-deny-advisories.md`）。

## 7. 未確定事項（TASK-13.6b への引き継ぎ）

以下は本草案（TASK-13.6a）では安全側の暫定案を示すのみとし、確定判断は
TASK-13.6b（#153）の人間レビューで行う。

1. **「自動適用候補」（ルール 2）の既定動作**: 「候補提示のみ（人間が
   最終トリガーを引く）」とするか、「実適用まで自動で進める」とするか。
   本草案では安全側の**「候補提示のみ」を既定として提案**する。実運用で
   自動化の範囲を広げる場合も、対象を「単一クレート内完結・非公開 API
   のみ・既存テストで担保されている」等にさらに絞り込む追加条件の要否を
   #153 で検討する。
2. **ルール 2 のシナリオ実例の不足**: PoC-7 の 3 シナリオ・TASK-13.4 の
   回帰シナリオのいずれも「自動適用候補」区分の実例を持たない
   （すべてルール 3 に該当）。実運用開始後、ルール 2 に該当する実際の
   変更パターンを収集し、想定どおりに安全側へ倒れているかを事後検証する
   運用（メトリクス収集等）を設けるかどうかは #153 で判断する。
3. **人間承認の記録・監査方法**: ルール 3 で要求する「人間承認」を
   どのような形式（PR レビュー承認・別途の承認ログ等）で記録し、
   監査可能性（REQ-13 非機能要件）を担保するかは本書では規定していない。
   #153 でリポジトリの運用（`implement-issue` 等の承認フロー）との
   対応関係を確定する。
4. **`ambiguous` 解消後の再判定運用**: 定義元が複数見つかった
   （`ambiguous: true`）場合にルール 3 へ倒すことは確定済みだが、
   多重定義の解消（リネーム等）自体を AI エージェントに任せてよいか、
   常に人間が介入すべきかは未確定。#153 で判断する。
