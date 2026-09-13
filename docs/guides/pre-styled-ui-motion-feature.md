# pre-styled-ui `motion` feature ガイド

本ドキュメントはイシュー #2416 を契機に作成しました。`fandhe-frontend-pre-styled-ui`
の Cargo feature `motion`（既定 off）を利用者の目的別に説明します。
機械可読な一次情報は `crates/pre-styled-ui/Cargo.toml` の `[features]`
直前コメント・`crates/pre-styled-ui/src/lib.rs` クレート doc「Cargo
feature `motion`」節であり、本書はそれを読みやすく再構成したものです。

## 1. 対象読者・要約

- `fandhe-frontend-pre-styled-ui` を直接依存に持ち、アニメーション拡張
  （spring 近似イージング等、親トラッキング #2379 配下）を使いたい利用者
- `motion` feature を有効化しなければ、crate サイズ・ビルド時間・
  `Theme::to_css`（`crates/pre-styled-ui/src/theme.rs`）の処理量・CSS
  出力サイズのいずれも変わりません（`docs/design/motion-reference-adoption-policy.md`
  §7「ゼロコスト方針」）。この保証は
  `crates/pre-styled-ui/tests/motion_zero_cost.rs` の 4 テストが CI で
  fail-closed に固定しています。

## 2. 有効化手順

```toml
[dependencies]
fandhe-frontend-pre-styled-ui = { version = "0.190", features = ["motion"] }
```

有効化すると `fandhe-animation`（外部依存ゼロ・`forbid(unsafe_code)`、
プラットフォーム非依存のアニメーション演算基幹）が依存グラフに加わります。

有効化で使えるようになる公開 API（`crates/pre-styled-ui/src/recipe.rs`）:

- **stagger CSS ユーティリティ（イシュー #2384）**: `recipe::STAGGER_INDEX_VAR`・
  `recipe::stagger_delay_declaration(step)`・`recipe::stagger_index_style(index)`・
  `recipe::SlotRecipe::stagger_delay(slot, step)`。要素ごとの「起点からの
  距離」を `--fandhe-motion-stagger-index` custom property として
  SSR/アプリコード側が `(\"style\", &stagger_index_style(i))` で書き出し、
  recipe 側は `stagger_delay(slot, MotionDuration::Fast)` の 1 行で
  `animation-delay: calc(var(--fandhe-motion-stagger-index, 0) *
  var(--fandhe-motion-duration-fast))` を登録できます。`fandhe_animation::
  timeline::Stagger::new(each).delay(index, total)`（`from: First`）と
  意味論が一致することを `recipe::stagger_parity_tests` が固定しています。

その他の拡張（spring 近似 `linear()` プリセット等）は後続イシュー
（#2381 以降）が実体を追加した時点で本節に追記します。

## 3. 無効時ゼロコスト保証の内容

| 指標 | 保証内容 | 対応する契約テスト |
|---|---|---|
| crate サイズ | 既定 feature の依存グラフに `fandhe-animation` が現れない | `motion_off_excludes_fandhe_animation_from_dependency_graph` |
| ビルド時間 | 上記の帰結（依存グラフに現れない依存はビルド対象にならない） | 同上 |
| `Theme::to_css` の処理量 | 走査ループへ motion feature 由来の実行時分岐（`cfg!(feature = "motion")` 等）を追加しない | `to_css_body_has_no_feature_cfg_or_motion_branch` |
| CSS 出力 | 既定テーマの `to_css()` 全文がバイト一致（`--features motion` 実行でも同一） | `default_theme_css_matches_pre_motion_golden` |

陽性対照（feature 名・`dep:` 配線の破損を検知）として
`motion_on_includes_fandhe_animation_in_dependency_graph` も併走します。

CI では `.github/workflows/ci.yml` の `clippy` ジョブが
`--no-default-features`/`--features motion` の両構成で `cargo check`・
`cargo clippy --all-targets -- -D warnings` を実行し、`test` ジョブが
`--features motion` 構成で上記契約テストを実行します。

## 4. gating の範囲

`motion` feature が除外するのは「C 群のみに分類され実装対象と定められた
拡張出力」（spring `linear()` プリセット等、
`docs/design/motion-reference-adoption-policy.md` §7）です。

- **presence（#2383）は feature 配下に置きません**: 同文書 §7 が「既定
  出力に無条件で含む」と明示しています。
- 共通 `@keyframes`（#2382）・scroll-driven（#2385）は同文書 §4 各行の
  採用方針に従い、追加時に判断します。stagger（#2384）は feature 配下に
  実装済みです（§2 参照）。

## 5. 消費者別の指定方針

- **docs-site（`fandhe-frontend-docs-site`）**: 現時点では `motion` を
  有効化していません（Demo 対象がまだ無いため）。有効化する際は
  `structure.toml` の `directories.pre-styled-ui.depends_on` へ
  `"animation"` を、`directories.animation.allowed_dependents` へ
  `"pre-styled-ui"` を同時に追加する必要があります（optional 依存が
  既定 feature 側で解決されるようになるため、`fw structure` の
  depends_on 完全一致検証の対象に入ります）。
- **dist-server 配布物**: `fandhe-frontend-dist-server` の配布 WASM は
  `crates/dist-server/src/wasm_dist_features.rs` の `WASM_DIST_FEATURES`
  （wasm-full 側の feature 集合）に従い、pre-styled-ui の CSS 生成
  （`Theme::to_css`）自体には関与しません。`motion` feature の on/off は
  配布物のビルド設定に影響しません。
- **examples**: `examples/headless-pre-styled-ui` 等は crates.io バージョン
  依存のため既定 off のままです。

## 6. 公開順序の注意

`fandhe-animation` は本 issue（#2416）時点で crates.io 未公開です
（`docs/ci/version-bump-publish-order-gap.md` §11）。`cargo publish` は
optional 依存であっても registry 上の解決を要求するため、
`fandhe-frontend-pre-styled-ui` 0.187.0 以降を公開する場合は
`fandhe-animation` の初回公開（同文書 §11 C 手順）を先に完了し、
sparse index への反映を確認してから実行してください
（`.github/workflows/release.yml` の該当コメント参照）。

## 7. 検証方法

```sh
cargo check  -p fandhe-frontend-pre-styled-ui --no-default-features --all-targets --locked
cargo clippy -p fandhe-frontend-pre-styled-ui --no-default-features --all-targets --locked -- -D warnings
cargo check  -p fandhe-frontend-pre-styled-ui --features motion --all-targets --locked
cargo clippy -p fandhe-frontend-pre-styled-ui --features motion --all-targets --locked -- -D warnings
cargo test   -p fandhe-frontend-pre-styled-ui --test motion_zero_cost --locked
cargo test   -p fandhe-frontend-pre-styled-ui --features motion --test motion_zero_cost --locked
cargo test   -p fandhe-frontend-pre-styled-ui --features motion --test motion_stagger_css --locked
cargo tree   -p fandhe-frontend-pre-styled-ui -e normal --prefix none --locked | grep -c fandhe-animation   # 0
```
