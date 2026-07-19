# 意図的非採用の記録（仮想 DOM・ファイルベースルーティング・HMR・signal）と AI 開発前提の評価軸

**本文書のステータス**: 確定（イシュー #352）。

> **本書の位置づけ**: 本フレームワークは仮想 DOM・ファイルベースルーティング・HMR
> （Hot Module Replacement）・signal/store といった主流フロントエンド機能を
> 意図的に採用していないが、その判断根拠がこれまで `docs/spec/`（サブモジュール、
> 編集禁止）の断片的な記述と各設計文書に散在しており、単一の正式文書が存在
> しなかった。本書はこれを一本化し、AI エージェントが「主流だから」という理由
> だけで再導入を提案するリスク、および人間レビュアーが判断根拠を追えないリスク
> を低減することを目的とする。関連 REQ: REQ-10（開発時 DX）・REQ-11（WASM
> 完全方式によるクライアントインタラクション）・REQ-13（AI 自己保守・改修の
> ためのフック・ゲート機構）（`docs/spec/04-requirements.md`）。

## 1. 目的とトレーサビリティ

- **課題**: 非採用判断の根拠が散在し、AI・人間双方が追跡しづらい。
- **目的**: `docs/policy/` に「AI 開発・保守前提の評価軸」と、その軸による
  4 機能（仮想 DOM・ファイルベースルーティング・HMR・signal/store）の非採用
  判断・代替手段・再評価トリガーを記録し、CLAUDE.md / 関連 rules から参照
  可能にする。
- **対象外**: 本書は `docs/spec/` の内容を変更するものではない。仕様自体の
  変更が必要と判断された場合は、frontend-framework-spec リポジトリ側で
  提案する（`.claude/rules/out-of-scope-tracking.md` 準拠）。

## 2. AI 開発・保守前提の評価軸

本フレームワークは「AI 時代のセキュリティリスク低減」を中核価値とし、AI
エージェントによる継続的な開発・保守を前提に設計されている（CLAUDE.md
Overview）。主流機能の採用可否を判断する際は、性能・開発体験だけでなく、
以下 4 軸で AI エージェントにとっての扱いやすさを評価する。

### 明示性（Explicitness）

コードを読むだけで挙動・依存関係が判断できるか。暗黙の規約・「マジック」
（自動収集・命名規約による暗黙結合・実行時にしか解決されない依存）を排除
できているか。AI エージェントは実行時の挙動を推測ではなく静的な読み取りで
把握する必要があり、暗黙の規約はコンテキストウィンドウに収まらない「読まな
いと分からない前提」を増やす。

### 決定性（Determinism）

同一入力から同一出力が再現されるか。AI エージェントによる変更が正しいかを
検証する際、非決定的な挙動（実行順序依存・タイミング依存の再描画等）は
リグレッション判定を困難にする。

### 機械検証可能性（Machine-verifiability）

grep・`fw impact`（`docs/design/impact-analysis-design.md`）・静的解析・CI
ゲートで契約違反を機械的に検出できるか。人間のレビュー勘に頼らず、AI 生成
コードの妥当性を自動判定できることが、REQ-13（AI 自己保守・改修のための
フック・ゲート機構）の前提となる。

### コンテキスト消費（Context cost）

AI エージェントが変更の影響範囲を判断するために読み込むべきファイル・概念
の量が小さいか。ファイル配置規約や設定ファイルの暗黙的な意味づけへの依存が
大きいほど、AI が正しい変更範囲を特定するために必要なコンテキストが増え、
誤った変更（影響範囲の見落とし）のリスクが高まる。

これら 4 軸は独立ではなく相互に補強する関係にある（例: 明示的な API は
機械検証もしやすい）。以下 §3 の各項目はこの軸に沿って評価する。

## 3. 非採用項目の記録

### 3.1 仮想 DOM

- **概要**: 差分検出のための仮想 DOM ツリーを構築し、実 DOM との差分パッチ
  を適用する方式。React・Vue 等の主流フレームワークで広く採用されている。
- **一般的な採用動機**: 宣言的 UI 記述と実 DOM 更新の分離、部分更新による
  性能向上。
- **評価軸での評価**:
  - 明示性: 差分検出アルゴリズム（reconciliation）はフレームワーク内部の
    ブラックボックスであり、どの DOM ノードが実際に更新されるかはランタイム
    の挙動を追わないと分からない。
  - 決定性: key の付け方・コンポーネントツリーの形状によって差分検出結果が
    変わりうる。
  - 機械検証可能性: 差分検出ロジック自体の正しさを grep や静的解析で検証
    することは難しい。
  - コンテキスト消費: 仮想 DOM の実装（core）自体が大きな依存となり、
    `core` の外部依存ゼロ方針（`.claude/rules/coding-rust.md`）と相反する。
- **本フレームワークでの代替**: 仮想 DOM 非採用の設計根拠と移行計画は
  `docs/design/dom-binding-update-design.md`（イシュー #340、「実 DOM 束縛点
  更新・keyed list の設計確定」）に本書より詳細な形で確定している。構造変化
  （リストの増減・並べ替え）にも仮想 DOM 的な汎用 diff を採用せず、keyed
  list という単一の専用経路に限定する設計である。実装状況は以下のとおり
  クレートごとに異なる（本書執筆時点）。
  - `rws-wasm-client`（`data-hydrate="like"` 等の最小ハイドレーション）:
    「束縛点最小更新」がすでに一般化実装済み。`data-bind-text` /
    `data-bind-attr` / `data-bind-class` 属性を 1 回走査して束縛点対応表を
    構築し、`set_text_content` / `set_attribute` / `class_list` の 3 種別
    に限定した DOM 変異のみを行う（`wasm-client/src/binding.rs` ・
    `wasm-client/src/binding_dom.rs`、イシュー #343、`docs/design/dom-binding-update-design.md`
    §3）。
  - `rws-wasm-full`（状態機械つきの既定インタラクション）: 現時点では
    `paint()` が `web_sys::Element::set_inner_html` によるイベント単位の
    領域再描画を行う（`wasm-full/src/dom.rs`）。以下の設計制約でリスクと
    コストを抑えている（`docs/design/wasm-full-architecture.md` 第 7 節・
    不変条件表）。
    - イベント委譲配線（`click` / `input`）をマウント時に 1 回だけルート
      要素へ登録する（`Closure` の都度 `forget` によるリークを構造的に
      回避）。
    - `input` イベント中は再描画を行わない（フォーカス・キャレット位置の
      破棄を避けるため）。
    - `paint()` が `set_inner_html` へ渡す文字列は必ず `rws_core::render()`
      の既定エスケープ済み出力である（REQ-1 の不変条件、
      `.claude/rules/coding-rust.md` の既定エスケープ厳守と一致）。
    - `wasm-client` が既に守っている最小更新路線への一般化はイシュー #345
      「`set_inner_html` 全置換を束縛点更新 + keyed list へ置換」として
      追跡中であり、本書執筆時点で未着手（open）。keyed list プリミティブ
      自体（イシュー #344）も同様に未着手（open）である。
  - 性能実測: `docs/ci/perf-browser-harness.md` / `docs/reports/perf-browser-report.md`
    （REQ-11 の受け入れ基準としての実ブラウザ計測）。
- **再評価トリガー**: 仮想 DOM の再導入検討は、束縛点更新 + keyed list への
  一般化（イシュー #344・#345）が完了してもなお perf-browser ゲート
  （`docs/ci/perf-browser-harness.md`）で REQ-11 の受け入れ基準を継続的に
  満たせず、かつ設計制約の追加調整では構造的に解消できないと判断された場合
  に限る（`docs/design/dom-binding-update-design.md` の移行計画が完了する
  前の再導入提案は本トリガーの対象外）。

### 3.2 ファイルベースルーティング

- **概要**: ディレクトリ・ファイル配置（例: `pages/about.tsx`）からルート
  定義を自動生成する方式。Next.js 等で広く採用されている。
- **一般的な採用動機**: ルート定義の記述省略、ディレクトリ構造とルート
  構造の一致による直感的な把握。
- **評価軸での評価**:
  - 明示性: ルートとファイルパスの対応関係が命名規約（暗黙のマジック）
    に依存し、コードを読むだけでは全ルート一覧を把握できない。
  - 決定性: ファイルシステムの走査順序・命名規則の解釈がフレームワーク
    バージョン間で変わりうる。
  - 機械検証可能性: 「このルートはどこで定義されているか」を特定するには
    ディレクトリ走査規約の知識が前提となり、grep 1 回では完結しない。
  - コンテキスト消費: AI エージェントがルート一覧を把握するには全
    ディレクトリ構造を読み込む必要があり、宣言的テーブル 1 ファイルを読む
    より消費コンテキストが大きい。
- **本フレームワークでの代替**: 宣言的な `Router` テーブル
  （`server/src/router.rs`、TASK-7.2b）。`Router::route(pattern, handler)`
  の builder パターンでルート一覧を 1 箇所に明示し、`Router::resolve` で
  解決する。パターン不正（先頭 `/` 欠落・空セグメント等）は `panic!` せず
  `RouterError` を返す設計。パスパターンの照合仕様は
  `docs/api/router-path-matching.md` に文書化されている。宣言的テーブルは
  `fw impact`（`docs/design/impact-analysis-design.md`）によるシンボル単位
  の影響解析と相性がよく、grep 1 回でルート一覧・影響範囲を特定できる。
- **再評価トリガー**: 人間開発者の比重が増加し、ルート定義の記述量削減
  （ファイル配置による省略）が機械検証可能性・明示性より優先されると
  プロジェクト運営判断で明確に位置づけられた場合。

### 3.3 HMR（Hot Module Replacement）/ dev サーバー

- **概要**: ソース変更をブラウザの状態を保ったまま即時反映する開発サーバー
  機構。webpack-dev-server・Vite 等で広く採用されている。
- **一般的な採用動機**: 開発時の反復速度向上、状態保持による確認コスト
  削減。
- **評価軸での評価**:
  - 明示性: モジュール差し替え時にどの状態が保持され、どの状態がリセット
    されるかはランタイムの実装詳細に依存し、事前に静的に判断しにくい。
  - 決定性: 差し替え順序・依存モジュールの再評価タイミングによって同一
    変更でも異なる見え方になりうる。
  - 機械検証可能性: 「変更が正しく反映されたか」を機械的に判定する基準を
    HMR 自体は提供しない（人間の目視確認が前提になりやすい）。
  - コンテキスト消費: HMR ランタイム自体が複雑な状態管理を持ち、AI が
    ビルドパイプラインの挙動を把握するための追加コンテキストとなる。
- **本フレームワークでの代替**: REQ-10（開発時 DX、
  `docs/spec/04-requirements.md`）が定める「本番差分ビルド反映 5 秒以内」
  ゲート。`dist-server/benches/rebuild_latency.rs` による rebuild latency
  計測が CI ジョブ（「REQ-10 rebuild latency (5s limit)」）として組み込まれ、
  実測値は `docs/reports/rebuild-latency-acceptance-report.md`
  （0.571〜0.597 秒）に記録されている。状態保持は行わず、決定的な
  フルリビルド + 高速反映という機械検証可能な基準に置き換えている。ブラウザ
  上の動作確認は `docs/guides/browser-testing.md` /
  `docs/ci/perf-browser-harness.md` の自動検証で補う。
- **再評価トリガー**: 人間の対話的な UI 微調整（試行錯誤を伴うスタイル
  調整等）が開発ワークフローの主となり、5 秒ゲートでの反復では実務上
  不十分と判断された場合。

### 3.4 signal / store

- **概要**: 細粒度リアクティブな状態プリミティブ（signal）や集中管理
  ストア（store）による状態管理。Solid.js の signal・Redux/Vuex の
  store 等が代表例。
- **一般的な採用動機**: 状態変化の追跡・依存解決の自動化、状態更新の
  細粒度化による再描画コスト削減。
- **評価軸での評価**:
  - 明示性: signal 間の依存関係はランタイムの自動追跡に委ねられ、コード
    を読むだけではどの signal がどの副作用を引き起こすか判断しにくい。
  - 決定性: 依存追跡の実行順序（バッチング・スケジューリング）により
    同一の更新シーケンスでも実行順が変わりうる実装がある。
  - 機械検証可能性: 「この state 変更で何が起きるか」を静的に列挙する
    ことが難しく、実行時デバッグに頼りやすい。
  - コンテキスト消費: リアクティブグラフ全体を把握しないと変更影響を
    判断できず、AI が読むべきコンテキストが増える。
- **本フレームワークでの代替**: `rws-interactive` の action-dispatch 単一
  状態機械（`interactive/src/lib.rs`）。`Component::view` の出力は
  `rws_core::Node` のみを経由し既定エスケープを必ず通す。状態遷移は
  `dispatch` 関数 1 箇所に集約され、未知のアクション名は no-op となる
  安全側フォールバックを規約化している（同ファイル冒頭の不変条件コメント
  1〜7 参照）。ハイドレーション属性の契約は `docs/api/interactive-api.md`
  ・`docs/api/hydration-state-format.md` に文書化されている。単一の
  `dispatch` 関数と明示的な action 列挙により、状態遷移の全体像を 1 ファイル
  から機械的に把握できる。
- **再評価トリガー**: アプリケーションの状態グラフの規模が拡大し、
  単一状態機械での全再評価コストが実測で性能受け入れ基準を超えることが
  確認された場合。

## 4. 運用（再導入提案時の手続き）

上記 4 項目のいずれかを再導入したいと判断した場合、以下を Issue・PR に
明記する。

1. §2 の評価軸 4 項目（明示性・決定性・機械検証可能性・コンテキスト消費）
   について、再導入後の設計がどう評価されるかを個別に記述する。
2. 該当項目の再評価トリガー（§3 各節）が実際に充足していることを、実測
   データ・受け入れ基準の未達実績等の根拠とともに示す。
3. 既存の不変条件（既定エスケープ・`forbid(unsafe_code)`・依存上限
   60 件/深さ 6・`core` 外部依存ゼロ、`.claude/rules/coding-rust.md`）
   を弱めない設計であることを示す。
4. 仕様（`docs/spec/`）の変更を伴う場合は、本リポジトリではなく
   frontend-framework-spec リポジトリ側で提案する。

## 5. 参照

- `docs/design/dom-binding-update-design.md`（イシュー #340、束縛点更新・
  keyed list の設計確定書。仮想 DOM 非採用の設計根拠・#341〜#345 の移行計画）
- `docs/design/wasm-full-architecture.md`（イベント委譲・`set_inner_html`
  再描画の設計制約、REQ-1/REQ-11 不変条件）
- `docs/spec/03-poc/differentiation-analysis/README.md`（PoC-1、Leptos/
  Dioxus/Yew/Sycamore の差別化分析。Sycamore の fine-grained reactivity
  採用に関する記述を含む）
- `docs/ci/perf-browser-harness.md` / `docs/reports/perf-browser-report.md`
  （REQ-11 性能実測）
- `server/src/router.rs`（宣言的 `Router` テーブル、TASK-7.2b）
- `docs/api/router-path-matching.md`（パスパターン照合仕様）
- `docs/design/impact-analysis-design.md`（`fw impact` シンボル単位影響
  解析）
- `docs/spec/04-requirements.md`（REQ-10・REQ-11・REQ-13）
- `dist-server/benches/rebuild_latency.rs` /
  `docs/reports/rebuild-latency-acceptance-report.md`（rebuild latency
  実測）
- `docs/guides/browser-testing.md`（ブラウザ自動検証）
- `interactive/src/lib.rs`（`AppState` / `dispatch` / action 単一状態
  機械）
- `docs/api/interactive-api.md` / `docs/api/hydration-state-format.md`
  （`rws-interactive` API・ハイドレーション状態フォーマット）

## 6. スコープ外（放置しない事項）

- `rws-wasm-full` への束縛点更新 + keyed list の一般化（イシュー #344・
  #345）自体の実装は本書のスコープ外であり、追跡状況の記録にとどめる。
  実装は既存イシューで追跡済みのため新規起票は不要。
- 評価軸（§2）を `fw gate` 等の機械ゲートへ組み込む自動化は本書のスコープ
  外。必要と判断された場合は別イシューとして提案する。
