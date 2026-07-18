# 自動適用と人間承認の境界ルール（AI 自己保守・改修ポリシー）

**本文書のステータス**: 確定（TASK-13.6b、Issue #153）。

> **本書の位置づけ**: 本書は TASK-13.6（親イシュー、`docs/spec/05-tasks.md` 376 行目）
> の成果物であり、`docs/spec/03-poc/ai-self-maintenance/README.md`（PoC-7）が提案した
> 「自動適用と人間承認の境界ルール」を、REQ-13 の受け入れ基準
> （`docs/spec/04-requirements.md` REQ-13 節）が求める「フレームワークの標準ドキュメント」
> として正式化・確定したものである。TASK-13.6 は 2 段階に分割されている。
>
> - **TASK-13.6a（Issue #152）**: 境界ルール草案の作成
> - **TASK-13.6b（Issue #153・本書）**: レビュー反映・確定
>
> 着手時点で TASK-13.6a の草案が `origin/main`・関連 PR・関連ブランチのいずれにも
> 存在しなかったため、本書は PoC-7 原文・REQ-13・製品実装（`cli/src/impact.rs`・
> `cli/src/gate.rs`）を一次情報源として新規作成し、そのまま仕様準拠レビュー・
> セキュリティレビューの観点を反映して確定させたものである。TASK-13.6a（#152）が
> 別途進行し草案が作られていた場合、本書と内容が重複する可能性がある。#152 側の
> 重複作業回避を提案する。

## 1. 目的とトレーサビリティ

- **関連要件**: REQ-13（AI 自己保守・改修のためのフック・ゲート機構、Must、
  `docs/spec/04-requirements.md`）の受け入れ基準のうち、次の項目に対応する。

  > 自動適用と人間承認の境界ルール（ゲート未通過は無条件拒否、`breaking_risk: low`
  > かつ影響ルートなしは自動適用候補、それ以外は人間承認必須）が、フレームワークの
  > 標準ドキュメントとして明文化されていること（PoC-7 の境界ルール節を踏襲）。

- **土台**: `docs/spec/03-poc/ai-self-maintenance/README.md`
  「自動適用と人間承認の境界ルール（成功基準 4、疑問点 13 の解消）」節（100〜107 行目）。
- **実装との対応**: 境界ルールの判定ロジックは製品実装として既に確定済みである。
  本書はその判定ロジックを「運用ポリシー」として文書化するものであり、
  判定ロジック自体の一次情報源は次の 2 ファイルである。
  - `cli/src/impact.rs` の `judge_breaking_risk` / `requires_human_approval`
    （`breaking_risk` 判定・人間承認要否判定の純粋関数、設計は `docs/impact-analysis-design.md`）
  - `cli/src/gate.rs` の `run_gate` / 5 種の検証チェック
    （ゲート合否判定、設計は `docs/gate-design.md`）
- **関連文書**: `docs/gate-design.md`（`fw gate` 判定ルール設計）・
  `docs/impact-analysis-design.md`（`fw impact` 判定ルール設計）・
  `docs/cargo-deny-advisories.md`（`cargo-deny` オンライン CI 運用）・
  `docs/dependency-graph-policy.md`（依存グラフ上限値運用ポリシー）・
  `docs/raw-html-review-gate.md`（`raw_html()` レビューゲート）。
- **関連テスト**: `cli/tests/negative_cases.rs`（型エラー・未エスケープ出力・
  禁止依存追加の 3 負例が `BLOCKED` になることの回帰テスト、TASK-13.5）。

## 2. 境界ルール（確定）

AI エージェントがプロダクトへの変更を提案したとき、`fw gate` の実行結果
（`gate_result`）と `fw impact <symbol>` の実行結果（`breaking_risk` /
`affected_routes` / `ambiguous`）を判断材料として、次の 4 ルールに従う。

### ルール 1: ゲート未通過（`gate_result: BLOCKED`）は無条件に自動適用しない

`fw gate` が実行する 5 種の検証チェック（`type_check` / `default_escape_check` /
`lint` / `test` / `policy`）のうち 1 件でも不合格であれば、変更は承認フロー以前の
段階で AI エージェントへ差し戻す。人間の承認を介する必要すらない、最もコストの低い
安全弁である。

- `type_check`: 型チェック（`cargo check --locked`）
- `default_escape_check`: 既定エスケープ検査。`raw_html()` の呼び出しに
  `#[expect(clippy::disallowed_methods, reason = "ESCAPE-REVIEWED: ...")]`
  によるレビュー済み明示がない場合に不合格とする（`docs/gate-design.md` §2.2・§5）。
  **この検査は REQ-1（既定エスケープ）の唯一の許容迂回経路であり、迂回そのものを
  「推奨手順」として運用しない。** `raw_html()` を使う変更は、常にこのチェックによる
  人間レビュー（`ESCAPE-REVIEWED:` 属性を書く行為自体が人間の明示的な承認記録となる）
  を経由することを前提とする。
- `lint`: `cargo clippy --locked -- -D warnings`
- `test`: `cargo test --locked`
- `policy`: `cargo deny check bans licenses sources`。**`advisories`（既知脆弱性 DB
  照合）はネットワークアクセスが前提のため、オフラインのローカルゲート実行では対象外
  とする**（`docs/cargo-deny-advisories.md`）。これは「オフラインでは省略してよい」
  という意味ではなく、オフライン環境では `bans` / `licenses` / `sources` の 3 チェックに
  限定した上で結果を確定させる、という運用上の切り分けである。`advisories` を含めた
  完全な検証は、ネットワークアクセスが保証される CI 環境（`docs/cargo-deny-advisories.md`
  記載の運用手順）で別途実行することを前提とし、ローカルゲート通過だけをもって
  「既知脆弱性が無いこと」を保証したとは扱わない。
  また `deny.toml` 自体が読み込めない場合も `policy` を起動せず即座に不合格とする
  （fail-closed、`cli/src/gate.rs` `policy_check`）。

### ルール 2: ゲート通過かつ `breaking_risk: low` かつ `affected_routes` が空の変更は自動適用候補とする

`fw impact <symbol>` の判定で `breaking_risk: low`（影響クレート数 0）かつ
`affected_routes` が空である変更は、単一クレート内で完結し他クレートのソースを
一切参照されない変更として、自動適用候補になり得る。ただし「候補」であって
無条件の自動適用を意味しない点に注意する。次の 2 条件がいずれも成立する場合に限る。

- `fw gate` が `PASS` を返している（ルール 1 の前提）
- `fw impact` が `ambiguous: false` を返している（ルール 3 参照。シンボルの定義元が
  一意に特定できている）

### ルール 3: ゲート通過かつ `breaking_risk: medium/high`、または `affected_routes` が非空、または定義元が曖昧（`ambiguous: true`）の変更は人間の承認を必須とする

製品実装の判定式（`cli/src/impact.rs` `requires_human_approval`）は次の論理和である。

```text
requires_human_approval =
    ambiguous
    OR breaking_risk in {High, Medium}
    OR affected_routes is not empty
```

- `breaking_risk` は影響クレート数から判定する（`judge_breaking_risk`）。
  影響クレートが `rws-wasm-client` / `rws-wasm-full` / `rws-wasm-thin`
  （クライアント境界クレート）のいずれかを含む場合は、影響クレート数に関わらず
  常に `High` へ倒す（ブラウザへ配布される境界への波及は安全側で扱う、
  `docs/impact-analysis-design.md` §3.2）。
- **`ambiguous`（シンボルの多重定義）は PoC-7 原文にない製品固有の追加条件である。**
  シンボル 1 つに対して定義元が複数見つかった場合、「シンボル 1 つに定義は 1 つ」
  という判定の前提が崩れているため、`breaking_risk` や `affected_routes` の値に
  関わらず、常に人間承認へ倒す（fail-closed。定義元特定を誤ると他の判定材料自体の
  信頼性が下がるため）。**この拡張はルール 2 の「自動適用候補」の範囲を狭める方向
  にのみ働き、PoC-7 原文より自動適用を広げる方向の緩和は一切行っていない。**

### ルール 4: 意味的な脆弱性・ロジック誤りはゲート・影響範囲解析のいずれの対象外であることを明示する

`fw gate` の 5 チェック・`fw impact` の影響範囲解析は、いずれも「構文・ポリシー・
エスケープ・テスト網羅・依存ポリシーの範囲」に限定される。**ゲート通過・自動適用
候補判定は「安全性の必要条件」であって「十分条件」ではない。** 次の限界を明示する。

- 意味的に誤ったロジック（既存テストがカバーしていない不具合、要件との乖離）は、
  ゲートを通過しても検出されない。ゲート通過は「機械的に検出可能な既知の危険パターン
  が無いこと」を意味するに留まる（`docs/spec/03-poc/ai-self-maintenance/README.md`
  「発見事項」1・`docs/impact-analysis-design.md` の PoC-2 整合の記述と同じ結論）。
- 影響範囲解析（`fw impact`）はファイル単位の粗粒度ヒューリスティックであり、
  シンボル単位の呼び出しグラフ解析は行わない。無関係なファイル・ルートも
  「影響あり」として過検知し得るが、これは安全側（見落としより過検知を許容する）
  の設計判断であり、精度向上（呼び出しグラフ解析・AST 解析への発展）は将来スコープ
  として明示的に見送っている（`docs/impact-analysis-design.md` §7、
  `docs/spec/05-tasks.md` TASK-13.2 の指示）。
- 「自動適用候補」（ルール 2）は、あくまで機械的な安全弁を通過したという意味であり、
  自動適用を実行する AI エージェント本体の実装・運用判断は REQ-13 のスコープ外
  （`docs/spec/04-requirements.md` REQ-13「制約・宿題」節）である。本書は機構
  （フック・ゲート）が提示する判断材料と境界条件を定義するに留まり、実際に
  「自動適用する／しない」を実行するオペレーター（人間・AI エージェント運用主体）
  の最終判断を代替しない。

## 3. 判定材料の出力形式（参照）

`fw gate` と `fw impact` はいずれも JSON を標準出力へ返す。境界ルールの適用可否は、
両コマンドの出力にある次のフィールドの組み合わせで機械的に判定できる。

| コマンド | フィールド | 本書での参照箇所 |
|---------|-----------|-----------------|
| `fw gate` | `gate_result`（`"PASS"` \| `"BLOCKED"`） | ルール 1 |
| `fw impact <symbol>` | `breaking_risk`（`"high"` \| `"medium"` \| `"low"`） | ルール 2・3 |
| `fw impact <symbol>` | `affected_routes`（配列） | ルール 2・3 |
| `fw impact <symbol>` | `ambiguous`（真偽値） | ルール 2・3 |
| `fw impact <symbol>` | `requires_human_approval`（真偽値、上記 3 者から一元的に算出済み） | ルール 2・3 の合成結果。個別フィールドを再判定せず本フィールドをそのまま使ってよい |

出力の詳細スキーマ・終了コード規約は `docs/gate-design.md` §4・
`docs/impact-analysis-design.md` §3.5 を参照する。

## 4. 監査可能性

`fw gate` / `fw impact` の判定結果は JSON として標準出力へ返るため、AI エージェントの
実行ログ・CI ログに残すことで判定根拠を事後監査できる。判定結果自体に絶対パス・
環境情報（ホスト名・ユーザー名等）を含めない設計とし（`docs/impact-analysis-design.md`
§6 A09 対策）、ログへ機微情報が混入しないことを、両コマンドの出力仕様として担保する。

## 5. 適用範囲・スコープ外

- 本書が定義する境界ルールは `fw gate` / `fw impact` の判定結果を判断材料とする
  運用ポリシーであり、判定ロジック自体の実装変更（`cli/src/impact.rs` /
  `cli/src/gate.rs`）は本書のスコープ外である。判定ロジックに変更が入った場合は、
  本書 §2 の記述が実装と乖離しないよう追随して更新する必要がある。
- AI エージェント本体（自動適用を実行するオペレーター）の実装は REQ-13 のスコープ外
  であり、本書もその実装方式を規定しない。
- 意味的検証（AST 解析ベースの精密化、ロジック誤りの自動検出）の実現方式は
  本書のスコープ外とし、将来タスクとして別途起票する（`docs/impact-analysis-design.md`
  §7 に将来スコープとして記載済み）。
