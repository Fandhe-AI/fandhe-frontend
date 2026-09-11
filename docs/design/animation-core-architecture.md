# fandhe-animation アーキテクチャ設計

- **イシュー**: [#2367](https://github.com/Fandhe-AI/fandhe-frontend/issues/2367)（親: #2365「Phase 0」、トラッキング: #2364「Motion/Motion+ 参照アニメーション充実」）
- **対象**: 新規文書のみ（実装コードの変更なし）
- **関連**: `docs/design/motion-reference-adoption-policy.md`（#2366、3 群分類・3 層構成・ゼロコスト方針の決定記録）・`docs/policy/dependency-graph-policy.md`（REQ-3 上限値運用）・`docs/policy/unsafe-boundary.md`（`unsafe` 使用箇所一覧）・`docs/design/wasm-full-feature-gating-evaluation.md`（feature 分割の実装 issue 分割の先例）・`.claude/rules/coding-rust.md`

## 1. 背景・スコープ

`docs/design/motion-reference-adoption-policy.md`（#2366）は Motion/Motion+ 機能の 3 群分類（A: CSS のみ／B: 小 DOM 配線／C:
フレームループ必須）と、C 群を実装する 3 層構成（演算コア `fandhe-animation` → Web アダプタ `fandhe-frontend-animation` →
配線層 `crates/wasm-full/`）の**責務境界**を決定記録として固定した（同文書 §6、判断記録 5/9）。同文書 §11 は「`fandhe-animation`
のアーキテクチャ設計詳細（モジュール構成・trait 設計等）→ #2367」と本文書へ委譲している。

本文書が扱う範囲は次の 4 点に限定する。

1. `fandhe-animation` の `Driver`/`Target`/`Interpolate` trait 境界（§2・§3）
2. no_std + alloc 化の評価（§4）
3. 将来の別リポジトリ切り出し手順（§5）
4. 3 層構成のモジュール構成詳細化と REQ-3 依存グラフ試算（§6・§7）

実コードの実装（`crates/animation/` 本体）・`structure.toml`/CLAUDE.md リポジトリツリーの本格更新・
`docs/policy/dependency-graph-policy.md` の実測値反映は本文書のスコープ外であり、§10 に列挙する後続 issue が担う。

## 2. Driver / Target trait 境界

### 2.1 Driver: tick 供給

`Driver` は「フレームループそのもの」ではなく「tick（経過時間）の供給源」を抽象化する。フレームループ（`requestAnimationFrame`
の呼び出し自体）は Web アダプタ（`fandhe-frontend-animation`、#2417 以降）の責務であり、`fandhe-animation` は「tick を受け取って
状態を進める」側に徹する。

pull 型（呼び出し側が能動的に次の tick を取得する）と push 型（`Driver` 側がコールバックを呼ぶ）を比較した。

| 方式 | 例 | native テストでの決定論的進行 | Web 側 rAF ラップ |
|---|---|---|---|
| pull 型（`fn tick(&mut self) -> Option<f64>`） | `ManualDriver` が固定 delta を返す | 容易（テストコードが `tick()` を好きな回数呼べる） | rAF コールバック内で `driver.tick()` を呼ぶだけで足りる |
| push 型（`fn subscribe(&mut self, cb: impl FnMut(f64))`） | `Driver` が内部で rAF ループを持ちコールバックを呼ぶ | 困難（テスト側がコールバックの起動タイミングを制御しづらい、`FnMut` の所有権管理が絡む） | `Driver` 実装側で rAF ループ管理が必要（設計が重くなる） |

**採用案は pull 型**とする。理由は native テストでの決定論的進行のしやすさ（`ManualDriver` に任意個の delta を積んでおき
`while let Some(dt) = driver.tick() { ... }` で進行を検証できる）と、Web 側の rAF ラップの単純さ（rAF コールバックから
`tick()` を 1 回呼ぶだけで結線が完結する）を両立できるため。

```rust
/// tick（経過時間）の供給源を抽象化する trait。
/// フレームループの起動・停止自体はこの trait の実装側（呼び出し元）が担い、
/// `fandhe-animation` はここから受け取った delta で状態を進めるのみ。
pub trait Driver {
    /// 次の tick の経過時間（秒）を返す。もう tick がない場合は `None`。
    fn tick(&mut self) -> Option<f64>;
}
```

native 参照実装（`ManualDriver`）は `#[cfg(test)]` または `test-utils` feature 配下に置き、`fandhe-animation` 本体の
公開 API 表面には持ち込まない（外部依存ゼロ方針・API 最小化の両立）。

### 2.2 Target<T>: 値の書き込み先

`Target<T>` は「補間結果をどこへ書き込むか」を抽象化する。`fandhe-animation` は `Target` の DOM 向け実装（CSSOM プロパティ
書き込み等）を一切持たず、trait 定義のみを提供する。これにより `fandhe-animation` は DOM 非依存を維持したまま、Web アダプタ
（`fandhe-frontend-animation`）側で `DomTarget`（CSSOM プロパティ経由の書き込み）等を実装できる。

```rust
/// 補間結果 `T` の書き込み先を抽象化する trait。
/// `fandhe-animation` はこの trait の DOM 向け実装を持たない
/// （DOM 非依存を保つための境界。実装は `fandhe-frontend-animation` の責務）。
pub trait Target<T> {
    fn write(&mut self, value: T);
}
```

native 参照実装（`RecordingTarget<T>`、書き込まれた値を `Vec<T>` に蓄積するのみ）も `Driver` 同様 `#[cfg(test)]` /
`test-utils` feature 配下に置く。

### 2.3 Web 実装可能性の検証

`fandhe-frontend-animation`（#2417 以降のスコープ）は次の実装で上記 trait 境界を満たせる見込みである（実装は #2417 以降が
担い、本節は「実装可能である」ことの設計根拠のみを示す）。

- `RafDriver`: `web_sys::window().request_animation_frame` をラップし、コールバック起動のたびに直前 tick からの
  経過時間（`performance.now()` の差分）を `tick()` の戻り値として供給する `Driver` 実装
- `DomTarget<T>`: CSSOM の `CSSStyleDeclaration::set_property` 等で DOM へ書き込む `Target<T>` 実装

いずれも `Driver`/`Target<T>` の trait シグネチャ（§2.1・§2.2）だけで表現可能であり、`fandhe-animation` 側へ Web 固有の
型・API を持ち込む必要はない。

## 3. Interpolate trait と値型

### 3.1 trait 境界

```rust
/// 2 値間の補間を表す trait。`t` は 0.0〜1.0 に限定しない
/// （spring・overshoot 系イージングは 1.0 を超える／下回る t を渡し得るため）。
pub trait Interpolate {
    fn interpolate(&self, other: &Self, t: f64) -> Self;
}
```

### 3.2 対象値型

`docs/design/motion-reference-adoption-policy.md` 判断記録 6（3D 用途を見据えた値型拡張）に基づき、次の値型を対象とする
（実装自体は #2374 のスコープ）。

- スカラー: `f32` / `f64`
- ベクトル: `Vec2` / `Vec3` / `Vec4`
- 回転: `Quat`（クォータニオン）
- 変換行列: `Mat4`
- 色: RGBA（各チャンネル線形補間）
- 複数値の集合: `[T; N]`（固定長配列）および `Vec<T>`（要素ごと `Interpolate` を要求、長さ不一致は呼び出し側の責務とし
  `fandhe-animation` はパニックしない契約とする方向で #2374 が詳細設計する）

### 3.3 依存ゼロ制約下での行列/クォータニオン型の扱い

外部依存ゼロ方針（`.claude/rules/coding-rust.md`）により `glam`/`nalgebra` 等の外部 crate は導入しない。`Vec3`/`Quat`/`Mat4`
は `fandhe-animation` 内に最小限のフィールドのみを持つ自前型として定義する（他クレートとの相互運用が必要になった場合の
変換関数〔`From`/`Into` 等〕は導入時期を待たず、必要になった時点で #2374 以降が追加する）。汎用スライスでの値表現（型消去した
`&[f64]` を補間する設計）は型安全性を失うため不採用とし、名前付きの自前型を優先する。

## 4. no_std + alloc 評価

**判定: 現時点で no_std 化は見送り（`std` を使う）**。

根拠:

1. 外部依存ゼロ制約下では、`f32`/`f64` の `sqrt`/`powf`/三角関数等の超越関数は `core` に存在せず、`std` かまたは外部
   `libm` crate が必要になる。spring ソルバ（#2375）・イージング関数（#2373、cubic-bezier のニュートン法反復等）は
   これらの超越関数を必須で使う。`libm` の新規導入は「`fandhe-animation` は外部依存ゼロ」という #2371 の受け入れ条件
   （`docs/design/motion-reference-adoption-policy.md` §6 の層定義）と衝突するため、現時点では選択しない
2. 現行の唯一の想定ターゲットは wasm32-unknown-unknown（`fandhe-frontend-animation` 経由の Web アダプタ）であり、
   `std` は全面利用可能（`wasm32-unknown-unknown` は `std` をサポートする対象トリプル）
3. embedded 等の no_std ターゲットへの具体的な利用要望は本文書作成時点で存在しない（仮説段階）

**再評価トリガー**: embedded/no_std ターゲットへの具体的な利用要望が出た場合、または `fandhe-animation` の切り出し
（§5）後に外部 `libm` 依存を許容する運用へ方針転換した場合。

## 5. 将来の別リポジトリ切り出し手順

`fandhe-animation` は「将来別リポジトリへ切り出し前提」（`docs/design/motion-reference-adoption-policy.md` §6）である
ため、切り出し手段を比較しておく。

| 選択肢 | 概要 | 長所 | 短所 |
|---|---|---|---|
| `git subtree split` による履歴保持 | `crates/animation/` の履歴のみを抽出し新規リポジトリへ push | commit 履歴（誰が・いつ・なぜ変更したか）を保持したまま切り出せる。監査可能性（REQ-3 の思想と整合）を維持しやすい | 実行手順がやや複雑（`git subtree split --prefix=crates/animation` の実行・force push 先の新規リポジトリ作成が必要） |
| submodule 化 | 切り出し後、本リポジトリから submodule として参照し直す | `docs/spec/` サブモジュール運用の先例がありオペレーションに慣れがある | workspace member として `cargo build` に組み込む場合、submodule 化は逆に複雑さを増す（`fandhe-frontend-animation` からの path 依存が submodule 境界を跨ぐ） |
| 新規リポジトリへの再作成（履歴非継承） | 現時点のスナップショットのみを新規リポジトリへコピー | 手順が最も単純 | 実装の変更履歴・議論の文脈（Issue 参照等）が失われる。監査可能性を重視する本リポジトリの方針と相性が悪い |

**推奨案**: `git subtree split` による履歴保持を推奨する。理由は本リポジトリが Issue 番号・決定記録（本文書を含む）を
コミット・PR に紐付けて追跡する運用（`.claude/rules/conventional-commits.md`・`out-of-scope-tracking.md`）を徹底しており、
履歴を失う切り出し手段はこの運用と整合しないため。

**運用への合流**: `fandhe-animation` は当面（切り出しが具体化するまで）本リポジトリの workspace member として運用する。
crates.io への独立公開（`.github/workflows/release.yml`・`docs/ci/version-bump-publish-order-gap.md` の運用）は本リポジトリの
既存クレートと同じフローに乗せる。切り出し自体は本文書のスコープ外であり、実装が安定した後の別 issue とする。

## 6. 3 層構成の詳細化（モジュール構成）

### 6.1 `fandhe-animation` 内モジュール案

| モジュール | 責務 | 対応 issue |
|---|---|---|
| `easing` | cubic-bezier / steps / linear() サンプリング等のイージング関数 | #2373 |
| `interpolate` | `Interpolate` trait・値型（§3） | #2374 |
| `spring` | 物理ベースの spring ソルバ | #2375 |
| `keyframes` | 複数値の時間割補間 | #2376 |
| `timeline` | timeline / sequence / stagger スケジューリング | #2377 |
| `driver` | `Driver` trait（§2.1）・native 参照実装（`test-utils` feature） | #2378 |
| `target` | `Target<T>` trait（§2.2）・native 参照実装（`test-utils` feature） | #2378 |

### 6.2 `fandhe-frontend-animation` 内モジュール案

| モジュール | 責務 |
|---|---|
| `raf_driver` | `requestAnimationFrame` ベースの `Driver` 実装（§2.3） |
| `dom_target` | CSSOM プロパティ書き込みの `Target<T>` 実装（§2.3） |
| `waapi` | Web Animations API 経由の実装（A/C 境界にまたがる場合の橋渡し） |
| `flip` | layout（FLIP）アニメーションの座標計測・適用 |
| `svg_path` | SVG path drawing（`stroke-dashoffset` 連続更新） |
| `scroll` | scroll-timeline 相当の連続値取得（evaluation-only 範囲を除く実装対象部分） |

各モジュールと Phase 4（トラッキング #2364 配下）の対応 issue（layout FLIP・SVG path drawing 等の実装 issue）は、
`fandhe-frontend-animation` crate 雛形作成（#2417）以降に個別起票される。

### 6.3 `wasm-full` 側の feature 名称

`crates/pre-styled-ui/` の `motion` feature（`docs/design/motion-reference-adoption-policy.md` §7、#2416 で契約テスト
予定）と命名を揃え、`wasm-full` 側の対応 feature 名にも `motion` を推奨する。ただし最終決定は Phase 4 のチェックリスト
issue（#2395）に委ねる（本文書は推奨に留め断定しない）。

### 6.4 兄弟アダプタ（評価文書止まり）

JS/React・wgpu 等のアダプタは `fandhe-frontend-animation` と同列の兄弟 crate（例: 将来の `fandhe-frontend-animation-wgpu`
等、命名は仮）として位置づける。これらの実アダプタ自体は評価文書止まりであり実装対象外（`docs/design/motion-reference-adoption-policy.md`
判断記録 6、Phase 6 #2413〜#2415）。

## 7. REQ-3 依存グラフ試算

`docs/policy/dependency-graph-policy.md` の実測値記法（`deps-check: packages=X/60 depth=Y/6 result=PASS` 形式）を
踏襲した**試算**（実測ではなく見積り。実測値の同文書への反映は #2372 のスコープ）を示す。

| クレート | 外部依存パッケージ数（試算） | 深さ増分（試算） | 根拠 |
|---|---|---|---|
| `fandhe-animation` | 0 | 0（ルートからの直接追加分） | `crates/core` と同型の外部依存ゼロクレート（§4 の no_std 見送り判定は「`std` を使う」の意味であり、外部サードパーティ crate の追加ではない） |
| `fandhe-frontend-animation` | `wasm-bindgen`/`web-sys`/`js-sys`（既存 `wasm-full` も依存する既存パッケージ群のみ） | 新規パッケージ増分はごく小さいと見積もる | `wasm-full` が既に同系統のパッケージへ依存しており、`cargo metadata` 上の重複解決により新規追加分は限定的 |
| `wasm-full` 経由の深さ増分 | — | `wasm-full → fandhe-frontend-animation → fandhe-animation` の 2 段 | `docs/design/motion-reference-adoption-policy.md` §6 の結論を踏襲・再掲（本文書独自の新規見積りではない） |

`fandhe-animation`/`fandhe-frontend-animation` は現行の `dependency-graph-policy.md` §4 の計測対象 7 パッケージに
含まれていない。計測対象への追加要否・実測値の反映は本文書のスコープ外とし、#2372（`fandhe-animation` を CI・依存グラフ
運用へ組み込む）が担う。

## 8. Phase 1 実装 issue 分解案（起票済み）

`docs/design/wasm-full-feature-gating-evaluation.md` §13 の表形式を踏襲し、本文書の各設計節と既存の起票済み issue を
対応付ける。issue 本文が使う内部ラベルと実際の issue 番号のマッピングは次のとおり。

| 内部ラベル | issue | 対応する本文書の節 |
|---|---|---|
| p1-1-a | [#2371](https://github.com/Fandhe-AI/fandhe-frontend/issues/2371) | `fandhe-animation` crate 雛形作成（§6.1 のモジュール骨格） |
| p1-1-b | [#2372](https://github.com/Fandhe-AI/fandhe-frontend/issues/2372) | CI・依存グラフ運用への組み込み（§7 の実測値反映） |
| p1-2 | [#2373](https://github.com/Fandhe-AI/fandhe-frontend/issues/2373) | easing 関数（§6.1 `easing`） |
| p1-3 | [#2374](https://github.com/Fandhe-AI/fandhe-frontend/issues/2374) | Interpolate trait + 値型（§3、§6.1 `interpolate`） |
| p1-4 | [#2375](https://github.com/Fandhe-AI/fandhe-frontend/issues/2375) | spring ソルバ（§6.1 `spring`） |
| p1-5 | [#2376](https://github.com/Fandhe-AI/fandhe-frontend/issues/2376) | keyframes 補間（§6.1 `keyframes`） |
| p1-6 | [#2377](https://github.com/Fandhe-AI/fandhe-frontend/issues/2377) | timeline / sequence / stagger スケジューリング（§6.1 `timeline`） |
| p1-7 | [#2378](https://github.com/Fandhe-AI/fandhe-frontend/issues/2378) | Driver / Target trait 境界の設計・実装（§2、§6.1 `driver`/`target`） |

上記に加え、3 層構成決定（#2417「`fandhe-frontend-animation` crate 雛形を作成し CI・publish 運用へ組み込む」）が Phase 1
の親 issue（#2369）と並行して起票済みである（§6.2 の Web アダプタ層の雛形に対応）。

## 9. セキュリティ・方針整合

- `fandhe-animation` は `#![forbid(unsafe_code)]` を維持する（`crates/core`/`crates/interactive` と同方針、
  `.claude/rules/coding-rust.md` REQ-2）。本文書の trait 設計（§2・§3）はいずれも `unsafe` を要しない
- `fandhe-frontend-animation` の `unsafe` は wasm-bindgen 境界に限定する（`wasm-client`/`wasm-full` と同方針、
  `docs/policy/unsafe-boundary.md` §2 のマトリクス）。同ドキュメントへの列挙は当該 crate 実装時（#2417 以降）に行い、
  本文書では追加しない（`fandhe-animation` 自体は `unsafe` を持たないため §2 のマトリクスに新規行は不要）

## 10. スコープ外

- 実コード実装（`crates/animation/`・`crates/frontend-animation/`）→ #2371・#2417 以降の各 Phase issue
- `structure.toml`/CLAUDE.md リポジトリ構成ツリーの本格更新 → #2368
- `docs/policy/dependency-graph-policy.md` の実測値反映 → #2372

## 再評価トリガー

- no_std 評価（§4）: embedded/no_std ターゲットへの具体的な利用要望が出た場合、または `fandhe-animation` の切り出し
  （§5）後に外部 `libm` 依存を許容する運用へ方針転換した場合
- `Interpolate` 対象値型（§3.2）に新規要望（Mat3・Bezier 曲線制御点等）が出た場合
