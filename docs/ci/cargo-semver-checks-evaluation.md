# cargo-semver-checks 導入評価（イシュー #656）

## 背景

- イシュー #638（PR #647）で `version-bump-guard`（`crates/xtask/src/check_version_bump.rs`、
  `.github/workflows/ci.yml` の `version-bump-guard` ジョブ）を導入した。これは
  「公開済みクレート（crates.io）の `src/` / `Cargo.toml` / `build.rs` が変更され、かつ
  Cargo.toml の `version` が既公開バージョンと同一のまま」という状態を機械的に検知する
  軽量チェックであり、公開 API の意味論的な後方互換性（引数追加等の破壊的変更そのもの）
  までは検証しない。
- 導入の直接の動機は headless-ui 0.1.0 公開直後に発生した事故（`radio_group::item` への
  引数追加、PR #611）であり、バージョンバンプなしで main へマージされて crates.io
  バージョン依存の examples e2e が型エラーで main を赤にした（復旧: PR #634 + 0.2.0 再公開）。
- PR #647 の out-of-scope 節に「cargo-semver-checks 等による本格的な semver 互換性検査は
  誤検知・検知漏れの実績を見て再評価する」と明記されており、本イシュー（#656、親: #655）は
  その評価タスクにあたる。`crates/xtask/src/check_version_bump.rs` のモジュール doc にも
  同旨の「別イシュー起票候補」の記述がある。

## 運用実績の集計

- **集計期間**: `version-bump-guard` ジョブが main へマージされた時点（PR #647 マージ
  コミット `e06f9f3`、2026-07-23 02:29 UTC 相当）〜本評価作成時点（2026-07-23）。
- **pull_request イベントでの実行数**: 2 件。いずれも PR #653
  （`test(wasm-full): Menu / Select / Tooltip 配線の実ブラウザテストを拡充する`、
  ブランチ `test/645-menu-select-tooltip-browser-test`）に対する再プッシュに伴う再実行。
  PR #647 自身（`ci/638-version-bump-guard` ブランチ）を対象とする CI 実行では、
  ジョブがまだ main にマージされていない自ブランチの変更を追加する形だったため
  `version-bump-guard` ジョブは走っていない。

| run ID | 実行日時 (UTC) | PR | conclusion | 分類 |
|---|---|---|---|---|
| 29975230218 | 2026-07-23T02:45:52Z | #653 | failure | 誤検知（false positive） |
| 29975576237 | 2026-07-23T02:54:20Z | #653 | success | exempt 宣言により PASS |

- **FAIL 事例の内容（run 29975230218）**: `version-bump-check: crate=fandhe-frontend-wasm-full
  version=0.1.0 published=yes result=FAIL`。実体は `crates/wasm-full/Cargo.toml` の
  `[dev-dependencies.web-sys]` へのフィーチャー追加のみであり、公開 API・ランタイム挙動に
  影響しない変更だった。**軽量チェック（`src/` / `Cargo.toml` / `build.rs` 変更を粒度で
  見る設計）が想定通り誤検知しうる類型の初事例**（dev-dependencies のみの変更は semver
  互換性に無関係だが、`Cargo.toml` 変更として一律検知される）。
- **exempt 宣言の使用実績**: 1 件（PR #653 本文）。

  ```
  version-bump-exempt: fandhe-frontend-wasm-full (crates/wasm-full/Cargo.toml の
  [dev-dependencies.web-sys] へ HtmlHeadElement feature を追加したのみで、新規外部
  クレート追加でも公開 API・ランタイム挙動の変更でもないため)
  ```

  クレート名の完全一致宣言により意図通り PASS した（`parse_exempt_crates` の包括免除を
  作らない設計が機能した事例）。
- **検知漏れ（guard が PASS したが実際は破壊的変更だった事例）**: 観測なし。ただし
  観察期間が実質 1 日・PR 1 件のみであり、判断材料として十分ではない。

集計に用いたコマンド（再現用。PR 本文・ログはユーザー制御の信頼しない入力のため、
シェルへ直接展開せず `--json`/`--jq` の構造化出力のみを参照した）:

```bash
gh run list --workflow ci.yml --event pull_request \
  --json databaseId,conclusion,createdAt,headBranch -L 100

gh run view <run-id> --json jobs \
  --jq '.jobs[] | select(.name | contains("version-bump")) | {name,conclusion}'

gh search prs --repo Fandhe-AI/fandhe-frontend "version-bump-exempt" \
  --json number,title
```

## cargo-semver-checks の評価

イシューの受け入れ条件（検知能力・依存追加の脅威面・CI 実行時間・保守結合）に沿って
評価する。

- **検知能力**: rustdoc JSON を突き合わせた lint 集合ベースの検査であり、lint 未収載の
  破壊的変更まで含めた完全性は保証されない。加えて、本評価で観測された唯一の FAIL
  （PR #653、dev-dependencies のみの `Cargo.toml` 変更）は cargo-semver-checks の
  対象外（rustdoc に現れない非 API 変更）であり、**既存 guard の代替にはならず、
  「バンプ済みだが不十分なバンプ」「exempt 宣言の妥当性検証」を補う位置付けにしかならない**。
  PR #611 型の事故（公開 API への引数追加、バージョン据え置き）は既存 guard で検知可能な
  類型であり、cargo-semver-checks の追加価値はここでは限定的。
- **依存追加の脅威面**: スタンドアロンのプリビルトバイナリとして運用すればワークスペースの
  依存グラフ（REQ-3: 60 件 / 深さ 6）には算入されない（xtask は外部依存ゼロ方針を維持できる）。
  一方でバイナリ供給元が 1 つ増えることはサプライチェーン面の拡大であり（security.md A06）、
  導入する場合は `tools/ci/ensure-gate-tools.sh` と同じ「バージョン固定 + SHA256 検証済み
  プリビルトバイナリ」パターン（`cargo install` によるソースからの任意最新版コンパイル禁止）
  への準拠が必須になる。
- **CI 実行時間**: 公開対象クレートは 11 件（core / interactive / app / server /
  wasm-client / wasm-full / wasm-thin / dist-server / headless-ui / pre-styled-ui / cli。
  `publish = false` の xtask・docs-site を除く）。baseline（crates.io 公開版）取得と
  現在の HEAD の双方で rustdoc JSON を生成する必要があり、11 クレート分のビルドが
  runner を追加で占有する（注記、イシュー #1238: 本評価時点は self-hosted runner
  の占有コストとして記載していたが、CI runner 方針のホステッドランナー既定への
  反転〔#1220〕により前提が変化した。public リポジトリでは標準ホステッド
  ランナーは無料・使い捨て VM のため「専有される runner 資源」という文脈の
  コスト論拠は薄れるが、同時実行数上限〔`docs/ci/hosted-runner-migration.md`
  §5.1〕やジョブ実行時間の増加という形でコストは残る。次回再評価時はホステッド
  前提でコストを再計算すること。結論への影響は限定的）。既存 `version-bump-guard` ジョブの軽量さ
  （`cargo metadata`・`git diff`・crates.io sparse index 照会のみ、`timeout-minutes: 10`）
  とは性質が異なり、独立ジョブとして分離するか `timeout-minutes` を再設計する必要がある。
- **保守結合**: rustdoc JSON のフォーマットはツールチェーンバージョンと結合しており、
  stable 更新のたびに cargo-semver-checks 側のバージョン追随が必要になり得る
  （`ensure-gate-tools.sh` の cargo-deny 同様、pin 値の定期更新運用が発生する）。

## 結論

**現時点では導入を見送る。** 根拠:

1. **観察期間の不足**: guard 稼働は実質 1 日・PR 1 件のみで、イシューが求める
   「誤検知・検知漏れの実績を観察してから判断」という前提を満たす材料がない。
2. **観測された唯一の FAIL は cargo-semver-checks で解決しない類型**: dev-dependencies
   のみの `Cargo.toml` 変更（PR #653）は semver 互換性検査の対象外であり、
   cargo-semver-checks は既存 guard の代替ではなく補完（exempt 宣言の妥当性検証）にしか
   ならない。
3. **コスト面**: 11 公開クレート × baseline/現在の両側 rustdoc JSON 生成による CI 実行時間
   増、rustdoc JSON フォーマットとツールチェーンバージョンの結合による保守負担、
   プリビルトバイナリ供給元追加によるサプライチェーン面の拡大。
4. **動機となった事故（PR #611 型）は既存 guard で検知可能**: headless-ui の引数追加事故は
   「`src/` 変更 × バージョン据え置き」であり、既存の軽量チェックが検知できる類型。

## 再評価トリガー

以下のいずれかに該当した場合、再評価のためのイシューを起票する:

- exempt 宣言の誤用（公開 API の破壊的変更なのに exempt 宣言で通過してしまう）による
  検知漏れ事故が 1 件でも発生した場合。
- exempt 宣言の使用頻度が恒常的に高くなり（目安: 月 5 件超）、宣言妥当性の人手レビューが
  負担化した場合。
- 公開クレート数・公開頻度が大きく増え、バンプ判断の機械検証価値が現状より上がった場合。

## 導入する場合の実装方式メモ（将来の再評価用参考）

- `tools/ci/ensure-gate-tools.sh` と同じ「バージョン固定 + SHA256 検証済みプリビルトバイナリ」
  パターンでインストールステップを追加する（`CARGO_DENY_VERSION` / `CARGO_DENY_SHA256` に
  倣った専用の pin 変数を用意する）。
- `version-bump-guard` ジョブへの後段ステップとして追加するか、CI 実行時間への影響を
  踏まえて独立ジョブに分離するかを、その時点の実測値で判断する。
- 既存の `environment error: ` プレフィックスによる環境エラーとコード起因 FAIL の区別規約
  （`docs/design/gate-design.md` §2.3a、`check_version_bump.rs` の fail-closed 契約）を
  踏襲する。
- 導入判断時に cargo-semver-checks の最新バージョン・配布形態（プリビルトバイナリの
  提供有無・チェックサム公開有無）・ツールチェーン互換方針を改めて確認すること
  （本評価作成時点では未確認、確定値ではない）。
