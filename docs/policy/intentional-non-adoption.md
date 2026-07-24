# 意図的非採用の記録（仮想 DOM・ファイルベースルーティング・HMR・signal）と AI 開発前提の評価軸

**本文書のステータス**: 確定（イシュー #352）。§3.5〜§3.9 は非採用確定
（イシュー #373）。§3.10 は非採用確定（イシュー #376）。§3.11 は非採用確定
（イシュー #379）。§3.12 は非採用確定（イシュー #381）。§3.13〜§3.15 は
非採用確定（イシュー #377）。§3.16〜§3.18 は非採用確定（イシュー #399、
出典 PR #386 / #390）。§3.19 は非採用確定（イシュー #405、出典 PR #383 /
`docs/design/wasm-full-architecture.md` §10）。§3.20〜§3.21 は非採用確定
（イシュー #639、出典 `docs/design/anchor-positioning-design.md` §4.3・
§4.5 / PR #613）。§3.21 は、CSS Anchor Positioning の progressive
enhancement（`@supports` 段階適用）のフォールバック設計案・非採用の論点
整理をイシュー #644（出典: 同書 §4.5a）でも参照されている（判断・節本文は
変更なし）。§3.22〜§3.24 は非採用確定（イシュー #735、出典
`docs/design/component-coverage-map.md`（イシュー #734）の保留・意図的
非採用プレースホルダ行の評価）。ただし §3.24 のうち Marquee は #831 で、
§3.22 のうち AngleSlider は #842 でそれぞれ §4 の再導入手続きに基づき
再導入済み（chakra `Theme` コンポーネント・ImageCropper・SignaturePad・
RichTextEditor は非採用のまま変更しない）。§7 は、イシュー #735 で非採用
ではなく保留のまま維持すると判断した項目群の再評価トリガーの記録である。

**節番号の採番規則**（イシュー #398、§3.6〜§3.8 の重複発覚を受けて明記）:

- 節番号（`### 3.N`）は本文書全体で一意とする。
- 新規節の追加は §3 の**末尾へ「既存最大番号 + 1」で追記のみ**とし、途中
  挿入・欠番の再利用・既存節のリナンバーは行わない。
- 並行ブランチのマージで番号衝突が生じた場合は、**後からマージする側**が
  自ブランチの節を末尾番号へ再採番してからマージする。
- 番号変更が不可避な場合は本リポジトリ全体を grep（`intentional-non-adoption`
  および `§3.`）して参照箇所（他の設計文書・コード内コメント含む）を追随
  させる。

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
  非採用判断・代替手段・再評価トリガーを記録し、CLAUDE.md / 関連 rules から
  参照可能にする。当初 4 機能（仮想 DOM・ファイルベースルーティング・HMR・
  signal/store、イシュー #352）に加え、§3.5〜§3.9 で属性値の URL スキーム
  検証・属性出力ポリシー（イシュー #373）、§3.10 で `wasm-thin` への束縛点
  更新・keyed list 方針の適用可否（イシュー #376）、§3.11 で `fw impact` の
  AST 解析ベース精密化（syn 等）の採否（イシュー #379）、§3.12 で評価軸の
  機械ゲート化の採否（イシュー #381）、§3.13〜§3.15 で Loader の async 化・
  キャッシュ / 再検証・複数 loader 合成の採否（イシュー #377）、§3.16〜§3.18
  で束縛点整合性の型付き強制・検証ユーティリティ横展開・`<base href>` 等の
  間接的 URL 制御対策の採否（イシュー #399、出典 PR #386 / #390）、§3.19 で
  `fandhe-frontend-wasm-client`（最小ハイドレーション方式）側のクライアントルーティング
  対応の採否（イシュー #405、出典 PR #383 の out-of-scope 節）、§3.20〜§3.21
  で anchor positioning 実装（イシュー #590）における Floating UI 相当の
  高度 positioning middleware 群・CSS Anchor Positioning（Web 標準）の採否
  （イシュー #639、出典 `docs/design/anchor-positioning-design.md` §4.3・
  §4.5 / PR #613）を追加記録する。§3.21 は、CSS Anchor Positioning の
  progressive enhancement 設計検討（イシュー #644、出典: 同書 §4.5a）の
  結果（判断は非採用のまま変更なし）もあわせて参照する。§3.22〜§3.24 で
  `docs/design/component-coverage-map.md`（イシュー #734）が前方参照の
  ままにしていた保留・意図的非採用プレースホルダ行のうち、高度入力系
  UI 部品（image-cropper 等）・JS ランタイム固有 utilities（portal 等）・
  その他 UI 部品（marquee 等）を非採用確定する（イシュー #735）。同イシュー
  では、残る保留項目（date-time 系・charts 全般等）の再評価トリガーを
  §7 にまとめて記録する。
- **対象外**: 本書は `docs/spec/` の内容を変更するものではない。仕様自体の
  変更が必要と判断された場合は、fandhe-frontend-spec リポジトリ側で
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
  - `fandhe-frontend-wasm-client`（`data-hydrate="like"` 等の最小ハイドレーション）:
    「束縛点最小更新」がすでに一般化実装済み。`data-bind-text` /
    `data-bind-attr` / `data-bind-class` 属性を 1 回走査して束縛点対応表を
    構築し、`set_text_content` / `set_attribute` / `class_list` の 3 種別
    に限定した DOM 変異のみを行う（`crates/wasm-client/src/binding.rs` ・
    `crates/wasm-client/src/binding_dom.rs`、イシュー #343、`docs/design/dom-binding-update-design.md`
    §3）。
  - `fandhe-frontend-wasm-full`（状態機械つきの既定インタラクション）: 現時点では
    `paint()` が `web_sys::Element::set_inner_html` によるイベント単位の
    領域再描画を行う（`crates/wasm-full/src/dom.rs`）。以下の設計制約でリスクと
    コストを抑えている（`docs/design/wasm-full-architecture.md` 第 7 節・
    不変条件表）。
    - イベント委譲配線（`click` / `input`）をマウント時に 1 回だけルート
      要素へ登録する（`Closure` の都度 `forget` によるリークを構造的に
      回避）。
    - `input` イベント中は再描画を行わない（フォーカス・キャレット位置の
      破棄を避けるため）。
    - `paint()` が `set_inner_html` へ渡す文字列は必ず `fandhe_frontend_core::render()`
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
  （`crates/server/src/router.rs`、TASK-7.2b）。`Router::route(pattern, handler)`
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
  ゲート。`crates/dist-server/benches/rebuild_latency.rs` による rebuild latency
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
- **本フレームワークでの代替**: `fandhe-frontend-interactive` の action-dispatch 単一
  状態機械（`crates/interactive/src/lib.rs`）。`Component::view` の出力は
  `fandhe_frontend_core::Node` のみを経由し既定エスケープを必ず通す。状態遷移は
  `dispatch` 関数 1 箇所に集約され、未知のアクション名は no-op となる
  安全側フォールバックを規約化している（同ファイル冒頭の不変条件コメント
  1〜7 参照）。ハイドレーション属性の契約は `docs/api/interactive-api.md`
  ・`docs/api/hydration-state-format.md` に文書化されている。単一の
  `dispatch` 関数と明示的な action 列挙により、状態遷移の全体像を 1 ファイル
  から機械的に把握できる。
- **再評価トリガー**: アプリケーションの状態グラフの規模が拡大し、
  単一状態機械での全再評価コストが実測で性能受け入れ基準を超えることが
  確認された場合。

### 3.5 WHATWG URL 準拠のフル URL パーサ（属性値 URL スキーム検証、イシュー #373）

- **概要**: `href`/`src` 等の属性値を WHATWG URL Standard 準拠のフルパーサ
  （オーソリティ・ポート・パス正規化まで含む）で解析し、スキーム判定を行う
  方式。
- **一般的な採用動機**: ブラウザの実パース挙動との完全な一致、エッジケース
  （IDN・パーセントエンコーディング等）の網羅。
- **評価軸での評価**:
  - 明示性: フルパーサの内部状態機械は複雑で、どの入力がどう解釈されるかを
    コードを読むだけで把握するのは難しい。
  - 決定性: パーサ自体は決定的だが、実装が大きいほど挙動の全体像を検証者が
    追いにくくなる。
  - 機械検証可能性: 巨大な状態機械は grep・簡易静的解析での契約確認に向かない。
  - コンテキスト消費: `core` 外部依存ゼロ（不変条件 7）を維持する場合は自前
    実装が必要になり、コンテキスト消費・保守コストが本イシューの脅威（URL
    スキーム経由の XSS）の重大度に対して過大。
- **本フレームワークでの代替**: `crates/core/src/url.rs` の `is_safe_url`（スキーム
  抽出のみを行う最小実装、外部依存ゼロ）。スキーム判定のみで
  `javascript:`/`data:`/`vbscript:` 等の脅威は遮断できるため、フルパースの
  必要性がない。
- **再評価トリガー**: 許可スキーム判定（現行実装）では防げない実攻撃パターン
  が XSS 回帰テスト（`crates/core/tests/xss_escape.rs`）で実証された場合。

### 3.6 `data:` URL の部分許可（`data:image/*` 等）

- **概要**: `data:` スキームのうち画像 MIME タイプ（`data:image/png` 等）
  のみを許可リストへ追加する方式。
- **一般的な採用動機**: 画像インライン埋め込みのユースケースを損なわない。
- **評価軸での評価**:
  - 明示性: MIME タイプ判定はデータ URI 内の文字列パースに依存し、
    `data:image/svg+xml`（SVG は script 実行可能）のような境界事例の扱いが
    自明でない。
  - 決定性: MIME タイプ文字列の大文字小文字・空白・パラメータ表記ゆれの
    正規化規則を固定しないと判定がぶれる。
  - 機械検証可能性: `data:text/html` との識別誤りが起きた場合、それを静的に
    検出する仕組みが別途必要になる。
  - コンテキスト消費: MIME タイプの許可リスト管理が新たな設定面を増やす。
- **本フレームワークでの代替**: v1 では `data:` を一律拒否する（`is_safe_url`
  の許可リストに含めない）。画像埋め込みが必要な場合は SSR/SSG 側で通常の
  `src="/path/to/image"` を使う運用を前提とする。
- **再評価トリガー**: 画像インライン埋め込みの実需要が REQ ベースで確定した場合。

### 3.7 `style` 属性の CSS サニタイザ

- **概要**: `style` 属性値を CSS パーサで解析し、危険なプロパティ・値
  （`expression()`・`url(javascript:...)` 等の歴史的ベクタ）を除去する方式。
- **一般的な採用動機**: CSS コンテキスト経由のデータ流出・レガシーブラウザ
  向け脆弱ベクタへの対策。
- **評価軸での評価**:
  - 明示性: CSS 構文解析は文法が広く、除去対象の網羅性を一目で把握しづらい。
  - 決定性: CSS パーサの実装差でエッジケースの扱いが変わりうる。
  - 機械検証可能性: 構文解析ベースの判定は grep 等の軽量な機械チェックでは
    代替できない。
  - コンテキスト消費: `core` 外部依存ゼロを維持する場合は CSS パーサの自前
    実装が必要になり、本イシューのスコープ（URL スキーム対策）に対して
    過大な投資になる。
- **本フレームワークでの代替**: 属性値エスケープ（`escape_html_into`）が
  breakout（`"` による属性境界の脱出）を既に防止済み。現代ブラウザでは
  `expression()` 等の CSS 経由コード実行は既に廃止されている。
- **再評価トリガー**: `style` 属性経由の実害あるベクタが判明した場合、または
  利用者コードへの `style` 属性動的値の需要が確定した場合。

### 3.8 インラインイベントハンドラ（`on*`）の許可付きサポート

- **概要**: `on*` 属性を一律拒否せず、許可リスト・サニタイズ付きで出力を
  許容する方式。
- **一般的な採用動機**: 既存 HTML 資産との統合、フレームワーク外コードとの
  互換性。
- **評価軸での評価**:
  - 明示性: `data-hydrate`/`data-bind-*` の束縛点方式（本フレームワークの
    正規経路）と `on*` インライン JS が並存すると、イベント処理の入口が
    2 系統になり、どちらが実際に発火するかをコードから追いにくくなる。
  - 決定性: 2 系統の配線が同一要素に重複した場合の優先順位を決定的に定義
    する必要が生じ、設計が複雑化する。
  - 機械検証可能性: dispatch モデルは `data-action`/`data-payload` の
    grep 可能な契約で検証できるが、`on*` 許可はその機械検証可能性を弱める。
  - コンテキスト消費: 2 系統のインタラクションモデルを AI エージェントが
    把握するコストが増える。
- **本フレームワークでの代替**: `data-hydrate`/`data-bind-*`（束縛点マーキング、
  イシュー #342/#343）による dispatch モデルを唯一の正規経路とする。`on*`
  相当のカスタム属性が必要な場合は `data-*` 属性を使う運用とする。
- **再評価トリガー**: フレームワーク外 HTML との統合要件で `on*` 出力が必須と
  確定した場合。

### 3.9 利用者定義の許可スキーム拡張 API（allowlist 差し替え）

- **概要**: `is_safe_url` の許可スキームリストを利用者コードから設定可能に
  する API（例: `configure_allowed_schemes(&["ftp"])`）。
- **一般的な採用動機**: `ftp:` 等、v1 の固定リストに含まれないスキームを
  必要とするユースケースへの対応。
- **評価軸での評価**:
  - 明示性: 設定可能な許可リストは、実際にどのスキームが許可されているかを
    コード読み取りだけでは判断できなくする（設定ファイル・呼び出し順序に
    依存する）。
  - 決定性: 「常に同じ規則」という REQ-1 型の予測可能性（既定エスケープが
    設定で弱められない設計）を、URL 検証側でも維持する必要がある。
  - 機械検証可能性: 固定リストなら grep で正リストの内容を機械的に確認
    できるが、実行時設定はその確認を不可能にする。
  - コンテキスト消費: 「どこで設定されたか」を追う必要が生じ、影響範囲の
    特定コストが増える。
- **本フレームワークでの代替**: `crates/core/src/url.rs::URL_ATTRS` と許可スキームは
  core 内の固定定数 1 箇所を正とする。v1 は固定リストのみ。
- **再評価トリガー**: `ftp:` 等の追加スキーム需要が Issue で確定した場合
  （その際も追加はリスト拡張のみとし、差し替え API は再検討する）。

### 3.10 `wasm-thin`（薄い JS グルー方式）への束縛点更新・keyed list の適用

- **概要**: `fandhe-frontend-wasm-full` / `fandhe-frontend-wasm-client` で採用済みの「束縛点更新
  （`data-bind-*`）+ keyed list（`data-key`）」方針（§3.1、
  `docs/design/dom-binding-update-design.md`）を、`fandhe-frontend-wasm-thin`（オプトイン
  の薄い JS グルー方式、`docs/design/opt-in-thin-js-glue.md`）の JS グルー側
  更新経路にも一般化するかどうかの検討（イシュー #345 out-of-scope 節から
  イシュー #376 として起票）。
- **検討した構成案とコスト**:

| 案 | 構成 | 却下理由 |
|----|------|---------|
| A: JS グルー側実装 | JS グルーが `data-bind-*` 走査・`textContent`/`setAttribute` 適用・keyed diff を実装する | JS 実効 LOC が PoC-3 ルーブリック上限（40 行 = 「中」）を大きく超過する。更新ロジック全体が Rust の型検査・`cargo test`・REQ-13 の AI 自己保守ゲートの到達範囲外へ移動し、`docs/design/opt-in-thin-js-glue.md` §3.1（(c) XSS 保証一貫性の減衰）・§3.2（(d) AI 生成検証の到達範囲）の制約が構造的に悪化する |
| B: WASM が diff 操作列を返し JS が適用する | `apply()` の戻り値を「HTML 文字列」から「操作列（JSON 等）」へ変更する | 「`initial_html()` / `apply()` の戻り値のみを `innerHTML` に設定する」という JS グルー規範（同 §5 不変条件 1・2）に反する新たな DOM 書き換え経路の新設になる。公開 API 凍結表（同 §4.2）の破壊的変更でもある |
| C: `fandhe-frontend-wasm-client` の束縛点適用層（`crates/wasm-client/src/binding_dom.rs` 等）へ依存する | `wasm-thin` が `web-sys` 依存の DOM 適用層を取り込む | 「`web-sys` 非依存・文字列 in・文字列 out の純粋計算」という `wasm-thin` の存在意義（`crates/wasm-thin/src/lib.rs` クレートドキュメント、`opt-in-thin-js-glue.md` §4.2）が消滅し、既定方式である `fandhe-frontend-wasm-full` と同型化してしまう。その要件であれば選定フローチャート（同 §2「位置づけ — 既定とオプトイン」）に従い `wasm-full` を使うべきである |

  - 4 軸評価（§2）: 案 A・B・C はいずれも更新ロジックの一部または全部を
    機械検証不能な JS 層へ移す、または `wasm-thin` の存在意義（明示的な
    「文字列 in・文字列 out」契約）を崩すため、明示性・決定性・機械検証
    可能性・コンテキスト消費のいずれの軸でも悪化する。
  - 補足事実: イシュー #345 以降、`fandhe-frontend-interactive` の `AppState::view()` /
    `render_html` が `wasm-thin` からも呼ばれる共通コードのため、
    `wasm-thin` の出力 HTML にも `data-bind-*` / `data-key` /
    `data-hydrate-item-ids` マーカーが**含まれる**。ただし `wasm-thin` の
    更新経路は `apply()` 戻り値の全置換 `innerHTML` 代入
    （`opt-in-thin-js-glue.md` §4.2・§5）であるため、これらのマーカーは
    `wasm-thin` 経路では**不活性（inert）**であり、無害である（属性値は
    既定エスケープ済み）。この形は `crates/wasm-thin/tests/thin_runtime.rs` の
    `demo_boundary_layer_smoke` が既に検証済みである。
- **本フレームワークでの代替**: 既定方式である `fandhe-frontend-wasm-full` を使う
  （`opt-in-thin-js-glue.md` §2 の選定フローチャートに従う）。DOM ノード
  同一性保持（フォーカス・IME・アニメーション維持）が必要なユースケースは、
  そもそも `wasm-thin` の想定選定範囲外である。
- **再評価トリガー**: 以下のいずれかが実測・仕様変更で確認された場合に限る。
  - (a) 実ブラウザ計測（`docs/ci/perf-browser-harness.md`）で `wasm-thin`
    経路の全置換再描画が REQ-11 の受け入れ基準を継続的に満たせないと実測で
    確認された場合。
  - (b) オプトイン採用者の移行ユースケースで DOM ノード同一性保持（フォーカ
    ス・IME・アニメーション）が必須となり、かつ `wasm-full` への移行が
    成立しない場合。
  - (c) 仕様（REQ-11）側で全クライアント経路への束縛点更新適用が必須化
    された場合（この場合は fandhe-frontend-spec リポジトリ側での提案が
    前提となる）。
- **XSS 回帰テストの位置付け**: Rust 側文字列出力の XSS 回帰は
  `crates/wasm-thin/tests/thin_runtime.rs`（native）の
  `apply_escapes_script_payload` / `apply_escapes_attribute_breaking_payload`
  / `demo_boundary_layer_smoke` が引き続き担保する。本節の非採用判断により
  更新経路は「既定エスケープ済み HTML の全置換」単一のままであり、これらの
  テストの削除・弱体化・追加はいずれも不要である
  （`.claude/rules/coding-rust.md`「XSS 回帰テストは削除・弱体化しない」）。
  JS グルー結合（実ブラウザ）の XSS 検証は
  `docs/design/xss-escape-wasm-test-design.md` §9 の判断（v1 スコープ外）を
  維持し、`opt-in-thin-js-glue.md` の制約明記（§3.1・§5）で担保する。

### 3.11 `fw impact` の AST 解析ベース精密化（syn 等、イシュー #379）

- **概要**: `fw impact`（`crates/cli/src/impact.rs` / `crates/cli/src/loaders.rs`）が
  行うシンボル定義元特定・使用箇所走査・`Loader` 実装抽出を、正規表現
  不使用・手書き文字列走査のヒューリスティックから `syn` 等の AST
  （抽象構文木）解析クレートへ置き換える方式。`docs/design/impact-analysis-design.md`
  §7 が将来スコープとして残していた検討タスク（PR #366 対象外節）。
- **一般的な採用動機**: コメント・文字列リテラル内の偶発的なシンボル出現
  を構文的に除外できる（過検知の低減）。`use X as Y` の別名・複数行に
  またがる宣言・マクロ生成シンボル等、識別子境界一致では追跡できない
  構造をインポート解決・構文木走査で正しく捕捉できる（見逃しの低減）。
- **実例収集（費用対効果評価の根拠）**: `crates/cli/src/impact.rs` /
  `crates/cli/src/loaders.rs` の `#[cfg(test)]` に「#379 characterization
  tests」として固定した現行仕様の実例。
  - 偽陽性（過検知・安全側）:
    `scan_usages_counts_occurrence_inside_comment_as_usage` /
    `scan_usages_counts_occurrence_inside_string_literal_as_usage`
    （コメント・文字列リテラル内の出現も使用箇所として数える）。
  - 偽陰性（見逃し）: `scan_usages_misses_alias_reexport_call_site`
    （`pub use crate::render as draw;` の再エクスポート宣言自身は
    検出されるが、実際の呼び出し箇所 `draw()` は文字列上 `render` を
    含まないため境界一致では追跡できない）。
    `does_not_detect_multiline_impl_loader`（`impl Loader\n    for X`
    のようにトレイト境界・型名が改行で分割された `impl` は単一行走査
    では検出できない）。
  - 偽陽性は `requires_human_approval` を承認要側へ倒すのみで安全側
    （fail-closed）である一方、偽陰性の主要因（複数行 `impl`・トップ
    レベル以外の `pub` 宣言）は本リポジトリのコード規約
    （単一行 `impl`・トップレベル `pub` 定義）で実質的に抑制されている
    （`find_definitions_ignores_indented_declarations` が既存挙動として
    固定済み）。
- **syn 導入時の依存影響実測**（使い捨てスクラッチプロジェクトでの実測、
  イシュー #379 受け入れ条件 2 に対応、`Cargo.toml`/`Cargo.lock` への
  痕跡は残していない）: `cargo add syn --features full` で追加される
  パッケージは `syn` / `proc-macro2` / `quote` / `unicode-ident` の
  **4 件**（依存深さ 3、`cargo tree --edges normal` 実測）。うち
  `proc-macro2` / `quote` の 2 件は `build.rs` を持つ。`cli`（`fandhe-frontend-cli`）
  は REQ-3「標準サーバー構成 60 件以内・深さ 6 以内」の直接の計測対象
  （`xtask` の依存グラフ計測基準）ではないが、`cli` 自体が現在
  「外部依存ゼロ」（`crates/cli/Cargo.toml` 冒頭コメント）であるため、
  4 件・深さ 3・`build.rs` 2 件はいずれもゼロからの純増となる。
- **評価軸での評価**:
  - 明示性: 手書き走査は `impact.rs` / `loaders.rs` 内で完結し、判定
    ロジック全体を `cli` の外部依存なしに読み切れる。`syn` 導入は
    Rust 構文木の型（`syn::Item` 列挙・`syn::UseTree` 等）への理解を
    前提化し、外部クレートの API サーフェスを読者に要求する。
  - 決定性: 双方とも決定的な走査であり、この軸での差はない（公平に
    記載）。
  - 機械検証可能性: 現行の過検知は fail-closed（`requires_human_approval`
    を承認要側へ倒す）であり、REQ-13 の AI 自己保守ゲート機構
    （人間承認を安全側の既定とする設計）と整合する。AST 化で得られる
    精度向上は主に「承認要 → 承認不要」方向の変化であり、ゲートを
    緩める投資になる。見逃しが残る限り「AST 化したから承認を省略して
    よい」という運用判断はできず、精度向上の実利は限定的である。
  - コンテキスト消費: `syn`（+ `proc-macro2` / `quote` / `unicode-ident`、
    上記実測）の追加はサプライチェーン脅威面（`.claude/rules/security.md`・
    PoC-2 脅威モデル）とビルドコスト（`build.rs` 2 件）を増やし、
    `cli` の「外部依存ゼロ」という単純な不変条件（読者が確認すべき
    依存面がゼロ）を失わせる。
- **本フレームワークでの代替**: 過検知容認（fail-closed）+ 人間承認への
  安全側フォールバック（`requires_human_approval`、
  `docs/design/impact-analysis-design.md` §3.4）+ 本リポジトリのコード
  規約（単一行 `impl`・トップレベル `pub` 定義）による偽陰性主要因の
  実質抑制 + 上記 characterization テストによる現行仕様の回帰的固定。
  `cli` の外部依存ゼロ方針（`crates/cli/Cargo.toml` 冒頭コメント）を維持する。
- **採用時の手続き（現時点では非該当）**: 将来 AST 化を採用する場合は
  `cargo metadata` で `cli` への実際の依存影響を確認し、ユーザー承認を
  得る（§4 の再導入手続きに準拠）。
- **再評価トリガー**: 以下のいずれかが確認された場合に限る。
  1. 過検知・見逃しが `requires_human_approval` / `breaking_risk` の
     誤判定として AI 自己保守フックの実運用で具体的な手戻り・障害を
     反復的に引き起こした実績が確認された場合。
  2. `fw impact` の適用対象が本リポジトリのコード規約に従わない外部
     プロジェクト（`fw new` 生成物の多様化を含む）へ拡大し、規約前提の
     偽陰性抑制が成立しなくなった場合。
  3. 外部依存ゼロ方針を維持したまま利用可能な構文解析手段（Rust 標準・
     rustc 安定 API 等）が現実的になった場合。
  4. `cli` の外部依存ゼロ方針自体がユーザー判断で変更された場合。

### 3.12 評価軸の機械ゲート化（イシュー #381）

- **概要**: §2 の AI 開発・保守前提の評価軸（明示性・決定性・機械検証
  可能性・コンテキスト消費）4 軸を `fw gate` 等の機械ゲートへ組み込み、
  専用の PASS/FAIL チェックとして自動判定する構想。PR #364 の対象外節
  から切り出されたイシュー #381 の検討結果。
- **機械判定可能項目の洗い出し（受け入れ条件 1）**: 4 軸のうち機械判定
  可能な下位項目と、既存の機械的担保の対応は以下のとおり。

  | 軸 | 機械判定可能な下位項目 | 既存の機械的担保（強制箇所） | 新チェック追加の要否 |
  |---|---|---|---|
  | 明示性 | grep 可能な API 使用（`raw_html()` の明示レビュー宣言） | `fw gate` の `lint` チェック（clippy `disallowed-methods` 主防御）+ `default_escape_check`（テキスト走査の保険層）+ ブランケット抑止監査の 3 層（`docs/design/gate-design.md` §2.2） | 不要（担保済み） |
  | 明示性 | 宣言的構成（クレート一覧・ルート表の単一情報源化） | `structure.toml` を唯一の情報源とする `fw gate` 全体の設計（`docs/design/gate-design.md` §2）・宣言的 `Router` テーブル（`crates/server/src/router.rs`）とそのテスト | 不要（担保済み） |
  | 決定性 | 同一入力 → 同一出力（SSR/SSG バイト一致・再実行一致） | `crates/server/tests/ssr_ssg_parity.rs`（`generate_is_deterministic_across_runs` ほか）→ `fw gate` の `test` チェック（`cargo test --locked -p <crate>`）経由で gate に既に接続済み | 不要（担保済み） |
  | 決定性 | 検証入力（依存グラフ）の固定 | `type_check` / `lint` / `test` 共通の `--locked` 付与（`docs/design/gate-design.md` §2・§5 A06） | 不要（担保済み） |
  | 機械検証可能性 | 契約違反の静的・機械的検出 | `fw gate` 6 チェックそのもの + `fw impact`（`docs/design/impact-analysis-design.md`）+ CI（`deny.yml`・XSS 回帰連携） | 不要（この軸は gate の存在意義そのものであり、専用チェックの追加は自己言及的な重複） |
  | コンテキスト消費 | 依存グラフ上限（60 件 / 深さ 6）という代理指標 | xtask の依存グラフ自動計測（`docs/policy/dependency-graph-policy.md`・CI） | 不要（担保済み） |
  | コンテキスト消費 | 「AI が読むべきファイル・概念の量」の直接計測 | なし | **追加不能**（決定的な PASS/FAIL 閾値を設計できず、ヒューリスティック判定は gate の決定性原則・環境エラー区別（`docs/design/gate-design.md` §2.3a）と両立しない） |

- **判断（非採用）**: 上記洗い出しの結果、機械判定可能な下位項目は
  すべて既存の機械的担保（`fw gate` の 6 チェック・既存回帰テスト・
  xtask 依存グラフ計測）で強制済みであり、新チェックの追加は既存
  チェックとの二重管理（単一情報源の崩壊）になる。残る「コンテキスト
  消費の直接計測」は決定的な PASS/FAIL 基準を設計できずヒューリスティック
  判定にならざるを得ず、gate 自身の設計原則（決定的判定・fail-closed・
  環境エラーとコード起因 FAIL の区別、`docs/design/gate-design.md` §2.3a・
  §3）と矛盾するため gate へ載せるべきでない。`fw gate` のチェック
  構成・JSON 出力契約は PoC-7 互換として固定されており（同 §4）、
  チェック追加は AI 自己保守フック・CI の利用契約に波及するため、
  必要性が顕在化していない現段階での契約変更はリスクのみが大きい。
  この判断はイシュー #353（新 API へのチェック追加の非採用、
  `docs/design/gate-design.md` §7）と同型であり、本イシューも同パターン
  に従う。コード・`fw gate` の振る舞い・JSON 契約はいずれも変更しない。
- **再評価トリガー**: 以下のいずれかが確認された場合に限る。
  1. 既存担保（3 層エスケープ検査・`ssr_ssg_parity` 決定性テスト・依存
     グラフ計測）をすり抜けた軸違反のリグレッションが実際に発生し、
     機械ゲートの不在が原因と特定された場合。
  2. `fw gate` へ静的検証チェックを追加する共通基盤が別要因で導入され
     （例: イシュー #380「束縛点整合性の静的検証手段の検討」が採用に
     至った場合）、評価軸チェックの追加コストが JSON 契約の安定性を
     損なわずに小さくなった場合。
  3. チェック構成・JSON 契約（PoC-7 互換）の改定が他要因で避けられ
     なくなった場合（改定時に評価軸チェック搭載を同時再検討する。イシュー
     #401 で 5→6 チェックへ改定済みだが評価軸自体の再検討には至らず、
     本トリガーは維持する）。

### 3.13 async loader

- **概要**: `fandhe-frontend-app` の `Loader` trait（`docs/design/loader-trait-design.md`、
  イシュー #346）が持つ同期 `fn load` を `async fn load` へ拡張し、外部 I/O を
  伴う loader をブロッキングなしで実行できるようにする案（イシュー #377）。
- **一般的な採用動機**: DB・HTTP API 呼び出し等の外部 I/O を伴う loader を
  想定した場合、同期実行は呼び出しスレッドをブロックする。
- **評価軸での評価**:
  - 明示性: `async fn load` 自体は明示可能だが、async ランタイムの実行モデル
    （タスクスケジューリング・`Send` 境界）はコードを読むだけでは判断しにくい
    副作用を持ち込む。
  - 決定性: async ランタイムのタスクスケジューリング順序は実装依存であり、
    SSR/SSG バイト一致という現行の構造的保証と緊張関係になりうる。
  - 機械検証可能性: 同期 `fn load` は `cargo test` で決定的に検証できるが、
    async 化すると `tokio::test` 等のテストハーネス依存が増え検証経路が
    複雑化する。
  - コンテキスト消費: async ランタイム導入は依存グラフに新規サブツリーを
    追加し、AI が把握すべき依存関係の範囲を広げる。
- **本フレームワークでの代替**: 同期 `fn load`（`crates/app/src/lib.rs:121`）を v1
  契約として維持する。現行の loader 実装（`DemoItemsLoader`・
  `DemoItemDetailLoader`）はいずれも固定デモデータを返す純関数であり外部 I/O を
  伴わないため、async 化を正当化する実需要が存在しない。加えて
  `docs/api/app-api.md` 第 9 節が記録するとおり `dist-server` は axum 不採用の
  実測根拠（`tokio-macros → syn → quote → proc-macro2 → unicode-ident` の連鎖が
  深さ 7〜9 に達し REQ-3 に違反）を持ち、`fandhe-frontend-app`/`fandhe-frontend-server` の外部依存ゼロ
  方針と両立する async ランタイムが現構成に存在しない。詳細は
  `docs/design/loader-extension-design.md` 第 3 節を参照。
- **再評価トリガー**: 以下の 3 条件をすべて満たした場合。
  1. 外部 I/O を伴う loader の要望が顕在化する。
  2. REQ-3（依存 60 件以内・深さ 6 以内）内に収まる async 構成を
     `cargo metadata` で実測確認できる。
  3. ユーザー承認を得る。

### 3.14 loader キャッシュ / 再検証（revalidation）

- **概要**: `Loader` 解決結果を一定期間再利用し、重複する I/O コストを削減する
  仕組み（イシュー #377）。TTL ベースの再利用・stale-while-revalidate 等が
  一般的な実装例。
- **一般的な採用動機**: 同一 loader 呼び出しの重複コスト削減。
- **評価軸での評価**:
  - 明示性: TTL・無効化タイミングはキャッシュ実装の内部状態に依存し、
    「このリクエストで loader が実際に呼ばれたか」がコードを読むだけでは
    判断しにくくなる。
  - 決定性: 「同一入力 → 同一出力」という現行の決定性を、TTL 経過時刻や
    無効化イベントという非決定要素で弱める。SSR/SSG バイト一致の構造的保証と
    緊張関係になる。
  - 機械検証可能性: キャッシュヒット/ミスの分岐は実行時刻・呼び出し順序に
    依存し、`cargo test` での決定的な網羅検証が難しくなる。
  - コンテキスト消費: キャッシュ層の存在は「loader が呼ばれるたびに最新
    データが返る」という単純な読み下しを崩し、AI が追加でキャッシュの
    生存期間・無効化条件を把握する必要が生じる。
- **本フレームワークでの代替**: 現行の三モード解決シーケンス（SSR: リクエスト
  時に毎回解決する純関数的経路 / SSG: ビルド時に 1 回だけ解決 / CSR 初期表示:
  ハイドレーション状態注入の再利用で loader 再実行なし）にはキャッシュが挟まる
  自然な箇所が存在しないため、キャッシュを導入しない。加えてキャッシュは
  stale データ配信・キャッシュポイズニング（キー設計不備による他ユーザー
  データ混入）という新規リスク面を持ち込む（`security.md`「セキュリティ設定
  ミス」観点）。詳細は `docs/design/loader-extension-design.md` 第 4 節を参照。
- **再評価トリガー**: 以下のいずれかが実測で確認された場合。
  - クライアント側ルーティング（画面遷移機構）の導入後、同一データの再取得
    コストが実測で性能受け入れ基準（REQ-11）を満たせないことが確認された場合。
  - 外部 I/O を伴う loader（§3.13）が導入され、その I/O コストが実測で問題化
    した場合。
  - 再評価時は stale データ配信・キャッシュポイズニングへの緩和策（キー設計・
    無効化契約）を再導入判断に必ず含める。

### 3.15 複数 loader 合成の専用 API

- **概要**: 単一ページが複数のデータソースを必要とする場合に、それぞれを
  別々の loader として書き結果を結合するための専用コンビネータ API（例:
  `and_then`/`zip`/`Loader::combine`）の新設（イシュー #377）。
- **一般的な採用動機**: 複数データソースの並列解決・結合を簡潔に書きたい
  需要（React Router の複数 loader 並列解決、GraphQL のフィールド結合等）。
- **評価軸での評価**:
  - 明示性: 専用コンビネータ API を新設すると、合成の意味論（並列か直列か・
    部分失敗時の挙動）を利用者が API ドキュメントを読んで理解する必要が
    生じる。
  - 決定性: 既存 `Loader` trait を素直に `impl` する合成であれば、実行順序・
    エラー伝播は通常の Rust コードと同じ決定性を持つ。
  - 機械検証可能性: 新規コンビネータは新規の型検査規則・テストパターンを
    要する。既存 trait のままの合成は既存の `assemble_list_page`/
    `assemble_detail_page`（`docs/api/app-api.md` §3.1）の型接続検証がそのまま
    機能する。
  - コンテキスト消費: 新規 API 概念（コンビネータの命名・意味論）の追加は
    AI がその API 固有の規約を追加で学習する必要を生じさせる。既存 trait の
    ままなら「`impl Loader` の書き方」という単一の学習対象で足りる。
- **本フレームワークでの代替**: 専用合成 API は新設せず、既存 `Loader` trait
  の 1 つの `impl`（内部で複数 loader を呼び、`Output` を結合構造体として
  返す）として合成 loader を書く規約を採用する。いずれかの内側 loader が
  失敗したら合成 loader 全体を失敗させる（fail-closed、部分成功データで
  描画を継続しない）ことを規約として固定する。コード例・詳細な規約は
  `docs/design/loader-extension-design.md` 第 5.4 節を参照。
- **再評価トリガー**: ページ数・データソース数の増加でボイラープレート
  （合成 loader の手書きコード量）が実測で問題化した場合。その場合もまず
  既存規約でのコード量を実測してから、専用コンビネータ API 導入の是非を
  再評価する。
### 3.16 束縛点整合性の型付きフィールド enum によるコンパイル時強制（イシュー #380 / PR #390）

- **概要**: `data-bind-*` トークン（producer: `crates/core/src/bind.rs`）と
  `BindingSource` フィールド名（consumer: `crates/wasm-client/src/binding.rs` の
  `impl BindingSource for AppState`）の整合を、現行の実行時文字列契約
  ではなく型付きフィールド enum でコンパイル時に強制する方式
  （`docs/design/dom-binding-update-design.md` §12.4）。
- **一般的な採用動機**: 文字列 typo・フィールド追従漏れをコンパイルエラー
  として検出でき、実行時の「無音の表示更新停止」（同書 §12.2）を構造的に
  防げる。
- **評価軸での評価**:
  - 明示性: 型付き enum 化は不整合をコンパイルエラーとして明示できる点で
    優れるが、SSR 出力形式（同書 §3.1）はすでに文字列トークンとして凍結
    済みであり、型レベル強制を導入するには `bind_*` API・`BindingSource`
    trait 自体の再設計（公開 API の破壊的変更）が前提となる。
  - 決定性: 型付き化自体は決定性を損なわないが、再設計の影響範囲
    （SSR/CSR 双方の凍結済み契約）が本イシュー（#380）のスコープを超える。
  - 機械検証可能性: 現行の `crates/wasm-client/tests/binding_logic.rs`
    `app_state_view_has_no_unresolved_bindings`（§12.3 のテスト時構造検証
    API 経由）が `cargo test -p fandhe-frontend-wasm-client` で決定的に検証済みであり、
    型付き化による追加の機械検証可能性の向上分は、再設計コストに対して
    限定的。
  - コンテキスト消費: `bind_*` API・`BindingSource` の再設計は、産出物
    （SSR 出力形式・`AppState::view` 実装・`BindingSource` 実装クレート
    全体）の学習コストを増やす。
- **本フレームワークでの代替**: テスト時構造検証 API
  （`crates/wasm-client/src/binding.rs` の `collect_binding_specs` /
  `unresolved_binding_specs`）+ 回帰テスト
  `crates/wasm-client/tests/binding_logic.rs::app_state_view_has_no_unresolved_bindings`。
  第 9 節（同設計書）の fail-closed（panic しない・no-op）不変条件は維持
  したまま、テスト実行時に不整合を機械検出する（同書 §12.3）。
- **再評価トリガー**: 束縛点を使うクレートが増え、テストユーティリティ
  では網羅が追えなくなった場合（例: `wasm-full` が独自の `BindingSource`
  実装を持ち、view 側マーカーとの同期漏れが反復的に発生する場合、
  同設計書 §12.5）。

### 3.17 検証ユーティリティの他クレート横展開（イシュー #380 / PR #390）

- **概要**: `collect_binding_specs` / `unresolved_binding_specs`
  （`crates/wasm-client/src/binding.rs`、§3.16 参照）を、`wasm-full` 等の他
  クレートが持つ独自の `BindingSource` 実装へ横展開し、整合検証を共通化
  する方式。
- **一般的な採用動機**: 束縛点整合検証を全クレート共通の仕組みとして
  提供できれば、クレートごとの検証実装の重複を避けられる。
- **評価軸での評価**:
  - 明示性・機械検証可能性: 横展開自体は既存の検証ロジック
    （`element_binding_specs` への委譲、§12.3）を再利用する設計であり、
    軸としては悪化させない。
  - コンテキスト消費: 現時点の consumer は `wasm-client` の
    `impl BindingSource for AppState` のみであり、横展開先となる実需要
    （他クレートの独自 `BindingSource` 実装）が存在しない。存在しない
    抽象化を先行実装すると、AI エージェントが「なぜこの汎用化が必要か」
    を判断するための追加コンテキストを要求する。
- **本フレームワークでの代替**: 横展開は行わず、`wasm-client` 内の
  `collect_binding_specs` / `unresolved_binding_specs` を単一 consumer
  向けの検証手段として維持する（§3.13 と同じ回帰テストで担保）。
- **再評価トリガー**: `wasm-full` 等が実際に独自の `BindingSource` 実装を
  持った場合。その時点で横展開を実施し、§3.16 の型付き API 再検討
  トリガーと連動して評価する。

### 3.18 `<base href>` / `<meta http-equiv="refresh">` 経由の間接的 URL 制御対策（イシュー #373 / PR #386）

- **概要**: 属性値そのものではなく、ページ内の別要素（`<base href>` /
  `<meta http-equiv="refresh">` 等）がナビゲーション先へ間接的に影響する
  経路に対する検証・ブロック方式
  （`docs/policy/attribute-output-policy.md` §2 脅威マトリクス最終行・
  §6 第 1 項）。
- **一般的な採用動機**: `href`/`src` 属性値の URL スキーム検証（同書
  §3.1）を回避し、`<base href>` でページ全体の相対 URL 解決先を書き換える
  等の間接的なナビゲーション制御ベクタへの対策。
- **評価軸での評価**:
  - 明示性・機械検証可能性: 属性出力ポリシー（`is_safe_url` /
    `URL_ATTRS` / `on*` 一律ブロック）は「属性値そのものの URL スキーム
    検証」を対象範囲として明示しており、`<base href>` 等の間接効果は
    脅威の性質が異なる（同書 §2「該当なし（別要素からの間接効果）」）。
    対象を混在させると、単一の検証関数（`is_safe_url`）が担う責務境界が
    曖昧になり、既存の機械検証可能性（`crates/core/tests/xss_escape.rs` の回帰
    テストが担保する範囲）を不明瞭にする。
  - 本フレームワークでの正規経路: `raw_html()` 以外の経路では、利用者
    入力から `<base>` / `<meta>` 要素を動的に組み立てる場合もノード木
    API（`.claude/rules/coding-rust.md`「HTML 文字列の直接組み立て禁止」）
    + 既定エスケープ（REQ-1）を経由するため、任意の外部入力が無検証で
    これらの要素を注入することはできない。ただし既定エスケープは
    breakout（属性境界の脱出）防止が目的であり、正当に構築された
    `<base href="https://evil.com">` のような要素自体（エスケープ済みで
    構文的に妥当な属性値）がナビゲーション解決先を書き換える効果までは
    防がない点に注意する。`raw_html()` 経由でこれらの要素が持ち込まれる
    残余リスクは既存の 3 層検査（`docs/design/gate-design.md` §2.2）に
    委ねる。
- **本フレームワークでの代替**: 現時点で専用の追加対策は導入しない。
  対策の実体は「利用者入力から要素自体を無検証に注入できない」という
  ノード木 API の構造的制約（上記正規経路）であり、属性値の URL スキーム
  検証（`is_safe_url` 等）とは異なる保証軸である点を明示した上で、
  属性出力ポリシーの対象外（本節）として台帳に留め置く。
- **再評価トリガー**: 対策強化が必要と判断された場合（実害あるベクタの
  判明・利用者からの需要確定）に、別 Issue として提案する
  （`docs/policy/attribute-output-policy.md` §6、
  `.claude/rules/out-of-scope-tracking.md` 準拠）。

### 3.19 `wasm-client`（最小ハイドレーション方式）側のクライアントルーティング対応（イシュー #405）

- **概要**: クライアント側ルーティング（履歴 API 連携・URL 同期・遷移時
  loader 配線）は `fandhe-frontend-wasm-full` の `nav` モジュール（`crates/wasm-full/src/nav.rs`、
  イシュー #374 / PR #383）として実装済みである。`fandhe-frontend-wasm-client`（最小
  ハイドレーション方式、REQ-6）側の遷移対応・loader 移行は PR #383 の
  out-of-scope 節、`docs/design/wasm-full-architecture.md` §10 に対象外
  事項として記録されており、本節はその採否判断を確定する。
- **一般的な採用動機**: 最小構成（`wasm-client`）のみを採用する利用者にも
  ページ全体リロードなしのクライアント遷移（history API 連携・URL 同期）を
  提供し、`wasm-full` へ移行しなくても SPA 的な遷移体験を得られるようにする
  動機。
- **評価軸での評価**（§2 の 4 軸）:
  - **明示性**: `wasm-client` は `hydrate()`（リスナー後付けのみ・DOM 再構築
    禁止）と `mount_csr()`（初回マウント時のみ `set_inner_html`）という 2 API
    への単純な責務分担を持つ（`crates/wasm-client/src/lib.rs` クレートドキュメント
    冒頭の不変条件 2・3）。クライアント遷移は「ページサブツリーの全差し替え」
    （`wasm-full` の `render_route` が `build_dom_node` で新規 DOM を構築し
    root の子を丸ごと差し替える方式）を本質的に要求し、上記不変条件 3 の
    DOM 変異 3 種別（`set_text_content` / `set_attribute` / `class_list`）
    限定契約を破る第 4 の変異カテゴリの新設になる。既存 2 経路の読み下しが
    複雑化し、明示性が悪化する。
  - **決定性**: 既存 2 経路（`hydrate()` / `mount_csr()`）とは独立した判断
    軸であり、決定性そのものは悪化しない（公平に記載）。
  - **機械検証可能性**: ルート表のクライアント側コピーが `wasm-client` にも
    必要になり、`crates/wasm-full/tests/route_sync_static.rs` 相当のドリフト検知を
    server ↔ wasm-full ↔ wasm-client の 3 点同期として二重に維持する必要が
    生じ、機械検証可能性が悪化する。
  - **コンテキスト消費**: 同一機能（クライアント遷移）の実装が `wasm-full` /
    `wasm-client` の 2 クレートに並存することになり、AI が変更影響を判断する
    際に読むべき範囲が倍増する。
  - 最小バイナリ・最小公開 API という `wasm-client` の方針（REQ-6）に対し、
    nav 配線の追加は web-sys features（`History` / `Location` /
    `MouseEvent`）とルート表・配線コードの増加を伴い、バイナリサイズと公開
    API 面の双方を拡大する。
- **検討した構成案とコスト**:

| 案 | 構成 | 却下理由 |
|----|------|---------|
| A: `wasm-client` 内に nav を独自実装 | `resolve_path` / 配線層を `wasm-client` へ複製する | ルート表・安全策（`is_safe_relative_path` 等）の重複コピーが発生し、ドリフト検知の三重管理になる。`hydrate()` の DOM 変異 3 種別限定契約（不変条件 3）を破る |
| B: `nav` を `wasm-client` へ移設し `wasm-full` が再エクスポートする | 依存方向（`wasm-full` → `wasm-client`）に沿って共有する | ルーティング（アプリ遷移機構）が「最小ハイドレーション」クレートの責務に混入し、最小 API 方針と `docs/api/hydration-api.md` の凍結範囲を崩す。凍結済み公開面の破壊的変更になる |
| C: `wasm-client` が `wasm-full` の `nav` に依存する | `wasm-client` → `wasm-full` 依存を追加する | 既存の依存方向（`wasm-full` は `wasm-client` に依存しない独立クレート、`docs/design/wasm-full-architecture.md` §2 判断 6）が逆転し循環依存になる（構造的に不可能） |

- **本フレームワークでの代替**: クライアント遷移が必要なユースケースは既定
  方式である `fandhe-frontend-wasm-full` を使う（`docs/design/opt-in-thin-js-glue.md`
  §2 の選定フローチャートと同じ判断に従う）。`wasm-client` の
  [`find_list_nav_targets`](../../wasm-client/src/lib.rs)（一覧ページの
  `data-nav` 属性値を列挙する純粋契約関数）は削除せず、将来採用時の対象
  特定契約として維持する。
- **再評価トリガー**: 以下のいずれかが実測・仕様変更・後続イシューの解決
  過程で確認された場合に限る。
  1. `wasm-client` 採用構成でのクライアント遷移の実需要が確定し、かつ
     `wasm-full` への移行がバイナリ予算等の実測根拠で成立しない場合。
  2. 仕様（REQ）側で全クライアント方式へのクライアント遷移提供が必須化
     された場合（この場合は fandhe-frontend-spec リポジトリ側での提案が
     前提となる）。
  3. イシュー #403（遷移後のインタラクティブ要素再配線）等の解決の過程で、
     `wasm-full` の `nav` と `wasm-client` の `hydrate()` の統合設計が
     避けられなくなった場合。
- **XSS 回帰テストの位置付け**: 本節の非採用判断はコード挙動を変更しない
  （`crates/wasm-client/src/lib.rs` の rustdoc 更新のみ）ため、`wasm-full` の
  `nav` に関する XSS 回帰テスト（`crates/wasm-full/tests/nav_native.rs` /
  `nav_browser.rs` 等）・`wasm-client` の既存テスト（doctest 含む）のいずれ
  も削除・弱体化・追加は不要である（`.claude/rules/coding-rust.md`「XSS 回帰
  テストは削除・弱体化しない」）。

### 3.20 Floating UI 相当の高度 positioning middleware 群（イシュー #639、出典 ADR §4.3）

- **概要**: anchor positioning 実装（`crates/headless-ui/src/positioning.rs` の
  純粋関数 + `crates/wasm-full/src/position.rs` の計測注入層、イシュー #590 /
  PR #622）は Floating UI 相当の middleware のうち flip（主軸の単純反転
  1 候補のみ）・shift（viewport 内クランプのみ）・sameWidth の 3 種類に限定
  採用している。本節は、それ以外の高度 middleware 6 種
  （autoPlacement / inline / hide / size（sameWidth 以外）/ VirtualElement /
  ポインタ追従・アニメーションフレーム連動の連続再計算（`autoUpdate` 相当））
  を導入しない判断を確定する。一次記録は
  `docs/design/anchor-positioning-design.md`（イシュー #589 / PR #613）
  第 4.3 節の非採用表であり、本節はその転記である。
- **一般的な採用動機**: Floating UI / ark-ui 互換の網羅的な配置制御（全方位
  探索による最適配置決定・インライン要素対応・anchor 不可視時の自動非表示・
  floating 要素自体の動的リサイズ・仮想参照要素・スクロール/リサイズに
  依存しない連続追従等）。
- **評価軸での評価**（§2 の 4 軸、ADR §4.3 の非採用表を展開）:
  - **明示性**: `VirtualElement`（実 DOM 要素を持たない仮想参照要素、例:
    マウスカーソル追従）は、本フレームワークの anchor が常に実 DOM 要素
    （トリガー・アンカーパーツ）であるという入力契約（ADR 第 4.1 節）に
    分岐を生む。
  - **決定性**: `autoPlacement`（全方位から最適解を探索）は探索順序・評価
    関数の実装差でエッジケースの結果が変わりやすく、本フレームワークの
    flip（1 候補のみの単純反転）と比べ挙動の予測可能性を下げる。
    ポインタ追従・アニメーションフレーム連動の連続再計算（`autoUpdate` 相当）
    は `requestAnimationFrame` 連動の連続監視を要し、テストの決定性を弱める
    （signal/store 非採用の評価軸、§3.4 と同根）。
  - **機械検証可能性**: `hide`（anchor が viewport 外に出た際の非表示制御）
    は「anchor の可視性」の判定にスクロール位置の連続監視を要し、本
    フレームワークが定める「呼び出し側が明示的に呼ぶ」という単純な再計算
    モデル（signal/store 非採用の継承）と相性が悪い。
  - **コンテキスト消費**: `inline`（インライン折り返しテキストの矩形分割
    対応）は、折り返しテキストを参照要素とするユースケースが本フレーム
    ワークの 4 コンポーネント（Popover/Tooltip/Menu/Select、いずれもボタン・
    トリガー要素が anchor）に存在しない。`size`（sameWidth 以外、floating
    要素自体の高さ等を viewport に合わせて動的に縮小）は、高さの動的
    リサイズが CSS（`max-height` + `overflow`）側で静的に対応可能な範囲が
    大きく、JS 計算側に持ち込む必要性が低い。
- **本フレームワークでの代替**: flip（主軸の単純反転 1 候補のみ）/
  shift（viewport 内クランプのみ）/ sameWidth の限定 3 middleware
  （ADR 第 4.3 節）と、スクロール・リサイズイベントを契機とした離散的な
  再計算呼び出し（ADR 第 5 節「再計算タイミング」、連続監視は非採用）。
- **再評価トリガー**: 以下のいずれかが実測・需要確定・ユーザー承認で
  確認された場合に限る（ADR §4.3 表の per-middleware トリガーに対応）。
  1. 単純な主軸反転（flip）では実運用上 viewport 内に収まらないケースが
     実測で確認された場合（`autoPlacement`）。
  2. インライン要素（`<a>` 内テキスト範囲等）を anchor とするコンポーネント
     の需要が確定した場合（`inline`）。
  3. スクロール連動の連続監視機構（IntersectionObserver 相当）の導入が
     ユーザー承認を得て確定した場合（`hide`）。
  4. 動的な高さ調整（viewport 残り高さに応じた `max-height` の実行時計算）
     が pre-styled-ui 側の CSS だけでは表現できないケースが確定した場合
     （`size`）。
  5. コンテキストメニュー等、マウス座標を anchor とするコンポーネントの
     実装が確定した場合（`VirtualElement`）。
  6. スクロール・リサイズの都度呼び出し（イベント駆動の離散再計算）では
     実用上不十分と判明した場合（`autoUpdate` 相当）。

### 3.21 CSS Anchor Positioning（Web 標準）（イシュー #639、出典 ADR §4.5）

- **概要**: `anchor-name` / `position-anchor` / `position-try-fallbacks` 等の
  CSS プロパティのみで anchor 要素への相対配置を実現する Web 標準方式
  （CSS Anchor Positioning）。一次記録は
  `docs/design/anchor-positioning-design.md`（イシュー #589 / PR #613）
  第 4.5 節であり、本節はその転記である。調査日 2026-07-22 時点のブラウザ
  実装状況（[MDN CSS Anchor Positioning](https://developer.mozilla.org/en-US/docs/Web/CSS/CSS_anchor_positioning)・
  [Can I Use: css-anchor-positioning](https://caniuse.com/css-anchor-positioning)
  参照）は Chrome/Edge 125 以降・Firefox 147 以降・Safari 26.0 以降が対応
  （グローバル使用率約 81.67%）であり、Baseline の「Widely available」判定
  基準（主要 3 エンジン対応後 30 か月経過）を、Firefox・Safari の対応が
  直近であるため満たしていない。
- **判断（非採用）**: Baseline「Widely available」未達である限り、安全側の
  既定として非採用とする。
- **評価軸での評価**（§2 の 4 軸 + Web 標準成熟度の観点）:
  - **決定性**: 未対応ブラウザ（Baseline 未達分の利用者環境）では、CSS の
    みに依拠すると位置決めが機能しない挙動分岐が生じる。
  - **機械検証可能性**: `position-try-fallbacks` が本フレームワークの
    flip/shift 相当（ADR 第 4.3 節の限定版）をどこまでカバーするかの
    再検証が必要であり、現行の決定的ユニットテスト
    （`cargo test -p fandhe-frontend-headless-ui`、ADR 第 5 節）に相当する
    機械検証手段が CSS のみの経路には確立していない。
  - **コンテキスト消費**: JS（wasm 層計測）方式との二重経路化（progressive
    enhancement）は、AI エージェントが読むべき位置決めロジックの経路を
    実質的に倍加させる。
  - **Web 標準成熟度**: Baseline「Widely available」未達は、本フレームワーク
    が安全側の既定として採用可否判断に用いる代理指標である。
- **本フレームワークでの代替**: wasm 層計測 + 純粋関数計算 + CSS カスタム
  プロパティ出力（`--fandhe-x` 等）を正式方式として採用する
  （ADR 第 4.1〜4.4 節）。
- **再評価トリガー**: 以下のいずれかが確認された場合に限る。
  1. CSS Anchor Positioning が Baseline「Widely available」へ到達した場合。
     到達時は `@supports (anchor-name: --a)` 等の機能検出による progressive
     enhancement（対応ブラウザは CSS のみで位置決めし、JS 計算をスキップ
     する）を再評価候補とするが、SSR/no-JS 環境での初期表示・flip/shift の
     挙動の再検証が必要であり、採用は ADR（`anchor-positioning-design.md`）
     の再改訂を伴う。
  2. ブラウザサポート状況の実測が ADR 第 4.5 節の記載と乖離したことが
     判明した場合。
- **追加検討記録（イシュー #644）**: 上記トリガー 1 の `@supports` progressive
  enhancement 案について、CSS 側（`pre-styled-ui` の `SlotRecipe::css` へ
  `@supports (anchor-name: --a)` ブロックを追加）・wasm 側（`reposition_all`/
  `reposition_one` 入口での `CSS.supports` 機能検出）・SSR/no-JS 側（既存の
  静的フォールバックのまま変更不要）の具体的なフォールバック設計案を
  `docs/design/anchor-positioning-design.md` 第 4.5a 節に記録した。判断
  （非採用）自体は変更しない。

### 3.22 高度入力系 UI 部品（image-cropper / signature-pad / angle-slider / rich-text-editor）（イシュー #735、angle-slider は #842 で再導入済み）

- **概要**: `docs/design/component-coverage-map.md`（イシュー #734）が
  「保留」区分の前方参照プレースホルダとして記録していた ark-ui /
  chakra-ui の高度入力系 4 コンポーネント。ImageCropper（canvas 上での
  画像トリミング）・SignaturePad（canvas へのポインタ座標ストローク
  記録）・AngleSlider（ポインタ座標から角度を算出する回転スライダー）・
  RichTextEditor（`contenteditable` ベースのリッチテキスト編集）。
- **一般的な採用動機**: 画像加工・署名取得・角度指定・書式付きテキスト
  編集は、フォーム部品としてありふれた需要である。
- **評価軸での評価**（§2 の 4 軸）:
  - **明示性**: canvas 描画（ImageCropper・SignaturePad）はピクセル単位の
    描画命令列・変換行列の内部状態に依存し、コードを読むだけでは最終的な
    出力（トリミング結果・署名画像）を判断できない。AngleSlider のポインタ
    座標→角度変換も、実装（`atan2` 系の計算とラップアラウンド処理）を
    読まないと挙動が分からない。
  - **決定性**: ポインタイベント（`pointermove`/`touchmove`）のストリームは
    デバイス・ブラウザ間で発火頻度・座標精度が異なり、同一操作から同一
    出力を再現する保証がない。canvas 描画結果も端末の解像度・DPI に依存
    しうる。
  - **機械検証可能性**: canvas ピクセル出力・ポインタ座標ストリームは
    `cargo test` 的な決定的アサーションで検証しづらく、視覚回帰テスト等
    の別カテゴリの検証基盤（本フレームワーク未整備）を要する。
  - **コンテキスト消費**: 上記 3 部品は WASM 層で `web-sys` の `Canvas`
    API・ポインタイベント API を新規に扱う必要があり、AI が変更影響を
    把握すべき API サーフェスを増やす。RichTextEditor は
    `contenteditable` 由来の HTML を扱うため、既定エスケープ（REQ-1）の
    経路外から HTML 相当のデータが持ち込まれる構造になり、「`raw_html()`
    以外の経路では必ずエスケープされる」という不変条件
    （`.claude/rules/code-comment-style.md` のセキュリティ不変条件の例）
    と衝突しうる迂回経路の新設に相当する。
- **本フレームワークでの代替**: 現時点で代替 API は提供しない。画像加工・
  署名取得・角度指定入力・リッチテキスト編集が必要なユースケースでは、
  利用者側が `raw_html()` を明示的に使い自前の検証責任を負う運用とする
  （`.claude/rules/coding-rust.md`「既定エスケープを弱めない」）。
- **再評価トリガー**: 以下のいずれかが確認された場合に限る。
  1. ImageCropper・SignaturePad・AngleSlider について、canvas 描画・
     ポインタ座標ストリームを決定的に検証できる自動テスト基盤（視覚
     回帰・座標アサーション等）が別途確立し、かつ利用要望が具体的な
     ユースケースを伴って issue で確定した場合。
  2. RichTextEditor について、既定エスケープ（REQ-1）を迂回しない形で
     `contenteditable` 由来の出力を安全に扱える Web 標準（例: EditContext
     API 等の構造化編集 API）が成熟し、かつ利用要望が確定した場合。
- **追加検討記録（イシュー #843、§4 手続きによる部分再導入）**: SignaturePad
  **のみ**、canvas を一切使わない決定的 SVG path 方式で再導入した
  （`crates/headless-ui/src/signature_pad.rs`/`crates/pre-styled-ui/src/signature_pad.rs`/
  `crates/wasm-full/src/headless_signature_pad.rs`）。
  - **評価軸の再評価**: 明示性は「ストローク座標列 → SVG path 文字列」の
    決定的純粋関数（`stroke_path_d`、丸め規則を固定表記で rustdoc に明文化）
    に置き換わり、コードと状態から最終出力が判断できる。決定性は同一座標列
    → 同一出力の純粋関数で保証し、デバイス依存のポインタ座標ストリームは
    wasm 層（`headless_signature_pad::StrokeCollector`）が「明示的な座標列」
    へ正規化してから headless 層の状態機械へ渡す構造にした。機械検証可能性
    は座標アサーション（合成座標列 + 合成 `PointerEvent`）による golden
    テストで `cargo test`/`wasm-pack test` の両方から決定的に検証できる。
    コンテキスト消費は `PointerEvent` API のみの追加に留め、`Canvas` 系
    API は一切導入しない。
  - **トリガー充足の根拠**: 上記トリガー 1「ポインタ座標ストリームを決定的
    に検証できる自動テスト基盤の確立 + 利用要望の issue 確定」を、座標
    アサーションによる決定的検証（視覚回帰基盤は不要）とイシュー #843 の
    利用要望確定をもって満たしたと判断した。
  - **判断の範囲**: canvas 方式（ImageCropper・AngleSlider・RichTextEditor、
    および SignaturePad の canvas 実装）の非採用判断自体は**変更しない**。
    本追補は SignaturePad の SVG path 方式による再導入のみを記録する。
- **再導入記録（イシュー #842、AngleSlider のみ。ImageCropper・
  SignaturePad・RichTextEditor は非採用のまま変更しない）**: 再評価
  トリガー 1（AngleSlider について、ポインタ座標ストリームを決定的に
  検証できる自動テスト基盤が別途確立し、かつ利用要望が具体的なユース
  ケースを伴って issue で確定した場合）を、以下の設計で充足したと判断し、
  §4 の手続きに従い再導入した。
  - **明示性**: 責務を 3 層に分離した。headless 層
    （`crates/headless-ui/src/angle_slider.rs`）は角度値（`0..=359` の
    整数）の状態機械のみでポインタ座標を一切扱わず、ラップアラウンド・
    step スナップ規則を整数演算のみで rustdoc に明文化する。座標→角度
    変換は wasm-full 層の単一の純粋関数
    [`angle_from_offset`](../../crates/wasm-full/src/angle_slider.rs)
    （`atan2` の使用箇所はこの 1 点のみ）へ完全に閉じ込め、canvas の
    描画命令列・変換行列に相当する内部状態は持たない。
  - **決定性**: 座標→角度変換は「最後に観測した座標 1 点から角度を
    再計算する」設計（`f64` 2 値 → 整数角度の純粋関数）であり、ポインタ
    イベントのストリーム頻度・座標精度差・履歴・速度に一切依存しない。
    表示は CSS `transform: rotate(var(--fandhe-angle))` のみ（canvas
    不使用）で端末の解像度・DPI にも依存しない。
  - **機械検証可能性**: headless 層・wasm 層のいずれも native
    `cargo test` の決定的アサーション（角度網羅表・ラップアラウンド
    境界・丸め境界・ARIA 出力の golden テスト・golden CSS）で検証する。
    視覚回帰基盤の新規整備は不要。
  - **コンテキスト消費**: pointer イベント API（`PointerEvent`/
    `setPointerCapture`）は新規サーフェスだが、`crates/wasm-full/src/angle_slider.rs`
    1 ファイルへ閉じ込め、既存の `headless.rs`（click 配線）・
    `headless_clipboard.rs`（独立配線モジュール）と同じ「純粋ロジック層
    + `#[cfg(target_arch = "wasm32")]` 配線層」2 層構成を踏襲した。
    canvas・`contenteditable` は導入していない。
  - 既存不変条件（既定エスケープ・`forbid(unsafe_code)`・`core` 外部依存
    ゼロ・`headless-ui` 外部依存は `core`/`interactive`（いずれも path）
    のみ・依存上限 60 件/深さ 6）は変更していない（新規依存クレートの
    追加なし、`web-sys` の既存依存への feature 追加のみ）。
- **利用者向けの等価概念対応表**: RichTextEditor（非採用維持）の等価概念は
  `docs/design/component-coverage-map.md` §8（イシュー #855）を参照。

### 3.23 JS ランタイム固有 utilities（portal / show / for / presence / client-only / environment / frame / swap / focus-trap / format-\* / locale / async-list / checkmark / radiomark / overlay-manager）（イシュー #735）

- **概要**: `docs/design/component-coverage-map.md`（イシュー #734）が
  「保留」区分の前方参照プレースホルダとして記録していた ark-ui /
  chakra-ui の utilities 群のうち、React/Solid 等の JS ランタイム機構に
  強く依存し、本フレームワークのノード木 + WASM 構成に対応概念がない
  24 件。Portal・Show・For（条件分岐・リストレンダリングのランタイム
  ヘルパー）、Presence・ClientOnly・EnvironmentProvider・Frame・Swap
  （マウント/アンマウント・実行環境分岐・iframe 内レンダリング・要素
  差し替えのランタイム機構）、FocusTrap・OverlayManager（フォーカス
  トラップ・オーバーレイのスタック管理）、FormatByte・FormatNumber・
  FormatRelativeTime・FormatTime・LocaleProvider・AsyncListCollection
  （国際化・非同期コレクションのランタイムユーティリティ）、Checkmark・
  Radiomark（チェック/ラジオの装飾専用アイコン）。
- **一般的な採用動機**: React/Solid フレームワーク上で条件付きレンダリング・
  環境分岐・フォーカス管理・国際化表示等を宣言的に書ける共通ヘルパーを
  提供する。
- **評価軸での評価**（§2 の 4 軸）:
  - **明示性**: Portal・Show・For・Presence 等はホストフレームワークの
    コンポーネントツリー・ライフサイクルに割り込むランタイム機構であり、
    本フレームワークの「`fandhe_frontend_core::Node` を経由するノード木」
    という単一の描画モデル（§3.1 参照）に該当概念自体が存在しない。
    無理に模して導入すると、ノード木 API 以外の第 2 の描画経路を作ることに
    なり明示性を損なう。
  - **決定性・機械検証可能性**: FocusTrap・OverlayManager が担う機能は
    既に本フレームワークで実装済みである（`crates/wasm-full/src/focus_trap.rs`
    のフォーカストラップ実装、`crates/wasm-full/src/overlay.rs` の
    オーバーレイスタック管理）。Checkmark・Radiomark 相当の装飾は
    `checkbox`・`radio_group` mod（`crates/headless-ui/src/checkbox.rs` /
    `radio_group.rs`）の状態機械にチェック/ラジオ表示として吸収済みである。
    これら実装済み代替と別に汎用ユーティリティを新設すると、同一機能への
    2 系統の実装が並存し機械検証可能性を損なう（§3.8 と同型の懸念）。
  - **コンテキスト消費**: FormatByte・FormatNumber・FormatRelativeTime・
    FormatTime・LocaleProvider・AsyncListCollection は UI コンポーネントで
    はなく国際化・非同期データ処理のランタイムライブラリであり、
    `fandhe-frontend-headless-ui`（UI コンポーネント層、CLAUDE.md）の責務
    境界外にある概念を持ち込み、AI が「なぜ UI コンポーネント層に国際化
    ライブラリがあるか」を把握する追加コンテキストを要求する。
- **本フレームワークでの代替**:
  - FocusTrap → `crates/wasm-full/src/focus_trap.rs`（既存実装）
  - OverlayManager → `crates/wasm-full/src/overlay.rs`（既存実装）
  - Checkmark / Radiomark → `checkbox` / `radio_group` mod
    （`crates/headless-ui/src/checkbox.rs` / `radio_group.rs`）
  - Portal / Show / For / Presence / ClientOnly / EnvironmentProvider /
    Frame / Swap / Format\* / LocaleProvider / AsyncListCollection は
    代替を提供しない。条件分岐・繰り返しは Rust の通常の制御構文
    （`if`/`for`）でノード木を組み立てる、国際化・数値整形は利用者側の
    通常の Rust 関数で行う運用とする。
- **再評価トリガー**: 以下のいずれかが確認された場合に限る（サブグループ
  ごとに記載）。
  1. Portal / Show / For / Presence / ClientOnly / EnvironmentProvider /
     Frame / Swap: SSR/CSR 境界の扱いを変えるアーキテクチャ変更（例:
     部分ハイドレーションの粒度細分化）がユーザー承認を得て採用され、
     該当概念が本フレームワークのアーキテクチャ上に新設された場合。
  2. FocusTrap / OverlayManager: 既存実装（`focus_trap.rs` / `overlay.rs`）
     では対応できない要件（多階層ネストの優先順位制御等）が実測で確認
     された場合（この場合も新規 utilities API の追加ではなく既存実装の
     拡張として検討する）。
  3. Format\* / LocaleProvider / AsyncListCollection: `fandhe-frontend-headless-ui`
     とは別の専用クレート（国際化・非同期コレクション処理）の新設が
     ユーザー承認を得た場合。
  4. Checkmark / Radiomark: `checkbox` / `radio_group` から装飾表現を
     切り出す具体的な需要（他コンポーネントでの再利用等）が確定した場合。
- **再導入記録（イシュー #853、FormatByte / FormatNumber / FormatTime /
  FormatRelativeTime の 4 件のみ。Portal / Show / For / Presence /
  ClientOnly / EnvironmentProvider / Frame / Swap / FocusTrap /
  OverlayManager / LocaleProvider / AsyncListCollection / Checkmark /
  Radiomark は非採用のまま変更しない）**: 再評価トリガー 3（「`Format*` は
  `fandhe-frontend-headless-ui` とは別の専用クレートの新設がユーザー承認を
  得た場合」）を、専用クレートの新設ではなく「`fandhe-frontend-headless-ui`
  内モジュール `format`」として実装することで解消したと判断した。上記
  コンテキスト消費の評価軸で挙げた懸念（国際化ライブラリの概念持ち込み）
  は、Intl API・`LocaleProvider` 等の JS ランタイム機構を一切使わない
  外部依存ゼロの決定的純関数（現在時刻 API 非依存、`Locale` enum による
  ロケール拡張点を型で明示）として実装することで解消し、専用クレート
  新設という再評価トリガーの文言そのものよりも軽量な手段で評価軸 4 項目
  （明示性・決定性・機械検証可能性・コンテキスト消費）を充足したと判断
  した。`crates/headless-ui/src/format.rs`（[`mod@format`]、
  `format_byte`/`format_number`/`format_time`/`format_relative_time`）
  として実装済み（`docs/api/headless-ui-api.md` の「Format ユーティリティ」
  節参照）。LocaleProvider / AsyncListCollection（en 以外のロケール拡張含む）
  はイシュー #854・#855 のスコープであり本記録の対象外。
- **利用者向けの等価概念対応表**: 上記 24 件のうち非採用のまま変更しない
  20 件それぞれの等価概念・代替実装は `docs/design/component-coverage-map.md`
  §8（イシュー #855）に一覧化する。

### 3.24 その他 UI 部品（marquee / chakra `Theme` コンポーネント）（イシュー #735、marquee は #831 で再導入済み）

- **概要**: `docs/design/component-coverage-map.md`（イシュー #734）が
  「保留」区分の前方参照プレースホルダとして記録していた 2 件。
  Marquee（自動流動テキスト、ark-ui/chakra-ui 双方に存在）と、chakra-ui
  の `Theme` コンポーネント（スコープ付きテーマ切替 utility）。
- **一般的な採用動機**: Marquee はニュースティッカー等の装飾表現、
  `Theme` コンポーネントはページの一部だけ異なるテーマ（配色トークン）を
  適用したい場合の宣言的な切り替え手段。
- **評価軸での評価**（§2 の 4 軸）:
  - **明示性・コンテキスト消費（Marquee）**: 自動流動テキストは純粋な
    装飾効果であり、CSS アニメーション（`@keyframes` + `overflow: hidden`）
    のみで実現可能で、専用コンポーネント API を新設する必然性がない。
    専用 API を追加すると「なぜ CSS だけで書けるものにコンポーネントが
    あるか」を AI が把握する追加コンテキストを要求する。
  - **決定性・機械検証可能性（Marquee）**: 自動アニメーションは
    `prefers-reduced-motion` 等のアクセシビリティ対応を要する挙動分岐を
    持ち込み、`autoUpdate` 相当の連続再計算（§3.20 と同根）と同様に
    決定的なユニットテストでの検証がなじまない。
  - **明示性・機械検証可能性（`Theme`）**: `crates/pre-styled-ui` は既に
    `theme` mod（テーマトークン定義）・`recipe` mod（スタイルバリアント
    合成）・`stylesheet` mod（CSS 出力）でテーマ管理の役割を担っている。
    chakra `Theme` コンポーネント相当の機能を並存導入すると、同一責務
    （テーマ管理）への 2 系統目の入口が生まれ、どちらが実際に適用される
    テーマを決定するかの機械検証可能性・明示性を損なう（§3.8 と同型の
    懸念）。
- **本フレームワークでの代替**:
  - Marquee: 代替を提供しない。自動流動テキストが必要な場合は利用者側の
    CSS（`@keyframes` 等）で実現する運用とする。
  - `Theme`: `crates/pre-styled-ui` の `theme` / `recipe` / `stylesheet`
    mod を既存の唯一の入口として維持する。
- **再評価トリガー**: 以下のいずれかが確認された場合に限る。
  1. Marquee: 自動流動テキストの需要が確定し、かつ
     `prefers-reduced-motion` 等のアクセシビリティ要件を満たす決定的な
     設計案が提示された場合。
  2. `Theme`: 既存 `theme` mod では表現できないスコープ付きテーマ切替
     （ページの一部だけ異なるテーマトークン集合を適用する等）の需要が
     具体的なユースケースを伴って確定した場合。
- **再導入記録（イシュー #831、Marquee のみ。`Theme` は非採用のまま
  変更しない）**: 再評価トリガー 1（自動流動テキストの需要が確定し、かつ
  `prefers-reduced-motion` 等のアクセシビリティ要件を満たす決定的な設計案
  が提示された場合）を、CSS のみ（JS ゼロ）・`prefers-reduced-motion:
  reduce` でのアニメーション停止・`hover`/`focus-within` での常時一時停止
  という決定的設計案で充足したと判断し、`crates/pre-styled-ui/src/marquee.rs`
  として §4 の手続きに従い再導入した（評価軸充足の詳細は同ファイル冒頭
  rustdoc、golden CSS 固定は `crates/pre-styled-ui/tests/marquee_css.rs`
  参照）。
- **利用者向けの等価概念対応表**: chakra `Theme`（非採用維持）の等価概念は
  `docs/design/component-coverage-map.md` §8（イシュー #855）を参照。

## 4. 運用（再導入提案時の手続き）

上記各項目のいずれかを再導入したいと判断した場合、以下を Issue・PR に
明記する。

1. §2 の評価軸 4 項目（明示性・決定性・機械検証可能性・コンテキスト消費）
   について、再導入後の設計がどう評価されるかを個別に記述する。
2. 該当項目の再評価トリガー（§3 各節）が実際に充足していることを、実測
   データ・受け入れ基準の未達実績等の根拠とともに示す。
3. 既存の不変条件（既定エスケープ・`forbid(unsafe_code)`・依存上限
   60 件/深さ 6・`core` 外部依存ゼロ、`.claude/rules/coding-rust.md`）
   を弱めない設計であることを示す。
4. 仕様（`docs/spec/`）の変更を伴う場合は、本リポジトリではなく
   fandhe-frontend-spec リポジトリ側で提案する。

## 5. 参照

- `docs/design/dom-binding-update-design.md`（イシュー #340、束縛点更新・
  keyed list の設計確定書。仮想 DOM 非採用の設計根拠・#341〜#345 の移行計画。
  §12.4・§12.5〔イシュー #380〕は本書 §3.13・§3.14 の非採用判断に対応）
- `docs/design/wasm-full-architecture.md`（イベント委譲・`set_inner_html`
  再描画の設計制約、REQ-1/REQ-11 不変条件）
- `docs/spec/03-poc/differentiation-analysis/README.md`（PoC-1、Leptos/
  Dioxus/Yew/Sycamore の差別化分析。Sycamore の fine-grained reactivity
  採用に関する記述を含む）
- `docs/ci/perf-browser-harness.md` / `docs/reports/perf-browser-report.md`
  （REQ-11 性能実測）
- `crates/server/src/router.rs`（宣言的 `Router` テーブル、TASK-7.2b）
- `docs/api/router-path-matching.md`（パスパターン照合仕様）
- `docs/design/impact-analysis-design.md`（`fw impact` シンボル単位影響
  解析、§7「既知の限界と将来スコープ」が §3.11 の非採用検討の出発点）
- `crates/cli/src/impact.rs` / `crates/cli/src/loaders.rs`（イシュー #379、`fw impact` の
  現行ヒューリスティック実装。`#[cfg(test)]` 内「#379 characterization
  tests」が §3.11 の判断根拠となった偽陽性・偽陰性の実例を固定する）
- `docs/spec/04-requirements.md`（REQ-10・REQ-11・REQ-13）
- `crates/dist-server/benches/rebuild_latency.rs` /
  `docs/reports/rebuild-latency-acceptance-report.md`（rebuild latency
  実測）
- `docs/guides/browser-testing.md`（ブラウザ自動検証）
- `crates/interactive/src/lib.rs`（`AppState` / `dispatch` / action 単一状態
  機械）
- `docs/api/interactive-api.md` / `docs/api/hydration-state-format.md`
  （`fandhe-frontend-interactive` API・ハイドレーション状態フォーマット）
- `docs/policy/attribute-output-policy.md`（イシュー #373、属性値の URL
  スキーム検証・属性出力ポリシー。§3.5〜§3.9 の非採用判断の対応元。§6
  第 1 項は本書 §3.15 の非採用判断に対応）
- `docs/design/opt-in-thin-js-glue.md`（イシュー #376、`fandhe-frontend-wasm-thin` の
  位置づけ・公開 API 凍結表・JS グルー規範。§3.10 の非採用根拠）
- `crates/headless-ui/src/format.rs`（イシュー #853、§3.23 の Format\* 再導入
  記録。JS の `Intl` API・`LocaleProvider` に依存しない外部依存ゼロの決定的
  純関数として byte / number / time / relative-time の 4 種を実装）
- `crates/wasm-thin/tests/thin_runtime.rs`（イシュー #376、`wasm-thin` の XSS 回帰
  テスト群）
- `docs/design/xss-escape-wasm-test-design.md`（イシュー #376、JS グルー
  結合の実ブラウザ XSS 検証スコープ判断）
- `docs/design/loader-extension-design.md`（イシュー #377、Loader の async 化・
  キャッシュ / 再検証・複数 loader 合成の設計確定書。§3.13〜§3.15 の非採用根拠）
- `crates/wasm-full/src/nav.rs`（イシュー #374 / PR #383、クライアント側ルーティング
  の実装本体。§3.19 の非採用判断における既定方式）
- `docs/design/wasm-full-architecture.md` §10（`nav` モジュールのスコープ外
  一覧。`wasm-client` 側の遷移対応・loader 移行が対象外事項として記録された
  出典。§3.19 の非採用根拠）
- `docs/api/hydration-api.md`（`fandhe-frontend-wasm-client` の最小ハイドレーション方式の
  凍結済み設計・不変条件。§3.19 の非採用根拠）
- `docs/design/anchor-positioning-design.md`（イシュー #589 / PR #613、
  anchor positioning の設計確定書。第 4.3 節が §3.20（Floating UI 相当の
  高度 middleware 非採用）、第 4.5 節が §3.21（CSS Anchor Positioning
  非採用）の一次記録）
- `crates/headless-ui/src/positioning.rs`（イシュー #590 / PR #622、
  anchor positioning の純粋関数実装。flip/shift/sameWidth の限定 3
  middleware。§3.20 の代替実装）
- `crates/wasm-full/src/position.rs`（イシュー #590 / PR #622、anchor
  positioning の wasm 層計測注入層。§3.20 の代替実装）
- `docs/design/component-coverage-map.md`（イシュー #734、ark-ui /
  chakra-ui 全コンポーネント対応表。§3.22〜§3.24・§7 が非採用・保留と
  確定した項目の一次対応表。§8 = イシュー #855 で追加した、§3.22〜§3.24 の
  意図的非採用項目についての利用者向け等価概念対応表）
- `docs/api/pre-styled-ui-api.md`（イシュー #716/#724、layout プリミティブ
  非採用の一次記録。component-coverage-map の「意図的非採用」区分の先例）
- `crates/wasm-full/src/focus_trap.rs`（フォーカストラップ実装。§3.23 の
  FocusTrap 代替）
- `crates/wasm-full/src/overlay.rs`（オーバーレイスタック管理実装。§3.23 の
  OverlayManager 代替）
- `crates/headless-ui/src/checkbox.rs` / `crates/headless-ui/src/radio_group.rs`
  （§3.23 の Checkmark / Radiomark 代替、状態機械への吸収）
- `crates/pre-styled-ui` の `theme` / `recipe` / `stylesheet` mod（§3.24 の
  chakra `Theme` コンポーネント代替）

## 6. スコープ外（放置しない事項）

- `fandhe-frontend-wasm-full` への束縛点更新 + keyed list の一般化（イシュー #344・
  #345）自体の実装は本書のスコープ外であり、追跡状況の記録にとどめる。
  実装は既存イシューで追跡済みのため新規起票は不要。
- 評価軸（§2）を `fw gate` 等の機械ゲートへ組み込む自動化は、イシュー
  #381 で検討し非採用と判断した（§3.12）。再評価トリガー（§3.12）充足
  時に別イシューとして再提案する。

## 7. 保留項目の記録と再評価トリガー（イシュー #735）

`docs/design/component-coverage-map.md`（イシュー #734）が「保留」区分の
前方参照プレースホルダとして記録していた項目のうち、§3.22〜§3.24 で
非採用確定した項目を除く残り（38 行。うち floating-panel の ark-ui/chakra-ui
参照 2 行はイシュー #827 で実装確定し「実装済み」区分へ移行済みのため、
本節時点の現存保留行は 36 行）は、本節でも「保留のまま維持」と
確定し、再評価トリガーのみを群単位で記録する。

**保留と非採用の違い**: 保留は非採用ではない。トリガー充足時は §4 の
「再導入手続き」を経ずに、通常の feature issue として起票する（§4 の
評価軸再確認・再評価トリガー充足の提示・不変条件維持の確認という手続きは、
一度「非採用」と確定した項目の再導入提案にのみ適用する）。

| 項目群 | 対象コンポーネント（対応表の参照ファイル） | 保留理由 | 再評価トリガー |
|---|---|---|---|
| date-time 系のうち実装済み | calendar / date-picker / date-input / timer | 保留解除・実装済み（暦計算コア `headless-ui::date`（イシュー #833）を先行前提として、外部依存ゼロ・現在時刻非取得の decisive な実装が確立したため、Calendar/DatePicker（イシュー #835）・DateInput（イシュー #834）の保留を解除。Timer（イシュー #836）も tick を外部から明示的に注入する決定的状態機械として暦・ロケール・タイムゾーンを一切必要としない設計を示し、同トリガーを充足したため保留を解除。date-time 系のうち保留のまま残るコンポーネントは現時点でない。headless `crates/headless-ui/src/calendar.rs`・`crates/headless-ui/src/date_picker.rs`・`crates/headless-ui/src/date_input.rs`・`crates/headless-ui/src/timer.rs` + styled `crates/pre-styled-ui/src/calendar.rs`・`crates/pre-styled-ui/src/date_picker.rs`・`crates/pre-styled-ui/src/date_input.rs`・`crates/pre-styled-ui/src/timer.rs` として実装済み。詳細は `docs/design/component-coverage-map.md` 該当行参照） | （実装済みのため該当なし） |
| 高度入力系（フォーム部品）のうち実装済み | color-picker / color-swatch / file-upload（いずれも ark・chakra 双方） | 保留解除・実装済み（color-swatch: イシュー #838 で色見本の静的表示を CSS `background-image` のみ（canvas 非依存）で実装。color-picker: イシュー #839（親 #837）で色領域・色相/アルファスライダーを CSS グラデーション + 導出整数割合（canvas 非依存）で実装し、再評価トリガー「canvas 依存部分を隔離し状態機械を純粋関数に保つ設計」を充足。file-upload: イシュー #840 の起票により保留解除、`headless-ui`/`pre-styled-ui`/`wasm-full` の `file_upload` mod として実装済み。`File` オブジェクト自体を headless-ui 層で一切保持せず、実 `File` API 接触は `wasm-full::headless_file_upload` の配線層のみに隔離する設計で再評価トリガーを充足した。ItemPreview/ItemPreviewImage（object URL プレビュー）は `File` オブジェクト非保持設計と両立しないためスコープ外のまま。ポインタドラッグ・キーボード操作の DOM 配線は引き続き `wasm-full` 側の後続対応。詳細は `docs/design/component-coverage-map.md` 該当行参照。高度入力系のうち保留のまま残るコンポーネントは現時点でない） | （実装済みのため該当なし） |
| JS ランタイム固有 utilities のうち静的実装可能なもの | download-trigger（ark・chakra 双方）/ json-tree-view | いずれも保留解除・実装済み（download-trigger: 利用要望 issue #828 の起票により保留解除、`headless-ui`/`pre-styled-ui` の `download_trigger` mod として実装済み。json-tree-view: イシュー #829 で実装済み `tree_view`（#753）の派生として実装完了、headless `crates/headless-ui/src/json_tree_view.rs` + styled `crates/pre-styled-ui/src/json_tree_view.rs` として実装済み。詳細は `docs/design/component-coverage-map.md` 該当行参照） | （両者とも実装済みのため該当なし） |
| charts 全般 | area-chart / bar-chart / bar-list / bar-segment / cartesian-grid / donut-chart / installation / legend / line-chart / pie-chart / radar-chart / scatter-chart / sparkline / tooltip / use-chart / axes（計 16 件） | 描画特性（SVG 座標計算・データスケーリング）が既存 UI 部品と異なり、`headless-ui`/`pre-styled-ui` とは別クレートとすべきか判断が必要。依存グラフ上限（60 件/深さ 6、REQ-3）との整合評価も未了。**Phase #845 で保留解除進行中**: 再評価トリガー（外部依存ゼロを維持したまま SVG ノード木生成のみで実装できる設計の確立）を充足する基盤（座標スケーリング・SVG ノード木生成・`ChartData` モデル）をイシュー #846 で実装済み（新クレートは作らず `pre-styled-ui::charts` モジュール群、判断根拠は `docs/design/charts-foundation-design.md`）。installation/use-chart の 2 件は保留解除・実装済みへ更新済み（`docs/design/component-coverage-map.md` 該当行参照）。残る 14 件（各チャート部品・軸/グリッド/凡例/ツールチップ）は #847〜#851 で個別に保留解除する | 利用要望の確定、および `core`/`headless-ui` と同じ外部依存ゼロ方針を維持したまま SVG ノード木生成のみで実装できる設計の確立（別クレート新設の要否はユーザー承認事項）。基盤（#846）は充足済み、残 14 件は #847〜#851 の実装完了が個別トリガー |
| 装飾系（CSS 主体で実装可能性がある、または既存基盤で実装見込みがあるもの） | tour | tour は状態機械（ステップ管理・ハイライト対象の同期）が大きく需要待ち。floating-panel（ark・chakra 双方）はイシュー #827 で headless+styled 実装済み、scroll-area（ark・chakra 双方）はイシュー #825 で保留解除・実装済み、splitter（ark・chakra 双方）はイシュー #826 で保留解除・実装済みのため本群から除外（`docs/design/component-coverage-map.md` の該当行を「実装済み」へ更新済み。scroll-area は JS によるスクロール位置追従・thumb drag が同イシューのスコープ外のまま） | 利用要望 issue の起票 |
| Button バリエーション | close-button / icon-button（`pre-styled-ui` の `Button` variant 拡張要望 issue #830 の起票により保留解除、`crates/pre-styled-ui/src/button.rs` の `icon_button`/`close_button` として実装済み。独立部品ではなく `button` recipe の icon-only 修飾 variant。詳細は `docs/design/component-coverage-map.md` 該当行参照） | 実装済み styled `Button`（`crates/pre-styled-ui`）の variant で近似可能であり、専用部品としての独立実装が必要かは需要待ち | `pre-styled-ui` の `Button` variant 拡張要望 issue の起票 |

再評価トリガー充足時の手続き: 上記表の該当行に基づき、通常の feature issue
（`create-issue` 等）を起票し、本節・`docs/design/component-coverage-map.md`
の該当行の区分・根拠列を実装確定後に更新する。
