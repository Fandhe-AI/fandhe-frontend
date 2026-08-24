# DOM 適用層（`crates/wasm-client`）の payload 内訳分析と削減試行の実測レポート

## 1. 目的とトレーサビリティ

- 本イシュー: #1407（本レポート作成対象）
- 親: #1405（wasm payload 削減の継続。gzip ≈41KB → ≈20KB 目標）
- 前提: #1406（closed、レポート `docs/reports/wasm-payload-baseline-1406.md`）。
  ハッシャ化（#1375）の payload 実効果は**削減ではなく微増**（gzip +983B）
  であったこと、残存する hashbrown/SipHash（named wasm 比 10.51%）は
  すべて `crates/wasm-client` の素 `std::collections::HashMap`/`HashSet`
  に帰着することを確認済み

本イシューの要件（自分の言葉での要約）:

1. twiggy で DOM 適用層（`crates/wasm-client` の keyed_dom / keyed_apply
   等）の関数別サイズを実測し、縮減対象を選定根拠付きで記録する
2. 重複経路の統合・単型化の集約・inline 抑制等でコードサイズを縮減する
   （既定エスケープ・DOM 適用契約・公開 API の意味論は不変）
3. `bench/payload` + `bench/csr` で before/after を実測し、payload 削減と
   op_ms 回帰なし（±5% 目安）を確認する
4. 縮減余地がない・op_ms とのトレードオフになるレバーは、差し戻して判断
   根拠を記録する（受け入れ条件が明示的に許容）

**結論を先に述べる**: 本イシューでは実測により縮減余地が確認できるレバーを
見つけられなかった。試行した唯一の実装レバー（cold path の
`#[cold]`/`#[inline(never)]` 化）は payload を **悪化**させたため差し戻した。
コード変更は含まない（`docs/reports/` の本レポート追加のみ）。

## 2. 計測環境・再現手順

| 項目 | 値 |
|------|-----|
| OS | Linux 7.0.0-29-generic（Ubuntu 系） |
| rustc | 1.96.0 (ac68faa20 2026-05-25) |
| wasm-bindgen-cli | 0.2.127（`bench/csr/fandhe/Cargo.lock` の pin と一致） |
| wasm-opt（binaryen） | version 116（`build.sh` の `WASM_OPT_EXPECTED_VERSION` と一致） |
| twiggy | twiggy-opt 0.8.0 |
| Node.js | v24.13.0 |
| 計測日 | 2026-08-24 |
| HEAD コミット | `8713254`（#1410 マージ直後） |

### 2.1 実行コマンド（payload 実測、production 相当）

```bash
bash bench/csr/fandhe/build.sh                    # -Os 配布物（正）
node bench/payload/measure.mjs --framework fandhe  # raw/gzip 実測
```

`meta.json` の `wasm_opt` は `"116"`（`"skipped"` ではない）ことを確認済み。

### 2.2 twiggy 用名前付き変種の生成

```bash
CARGO_TARGET_DIR=<scratch>/target cargo build \
  --manifest-path bench/csr/fandhe/Cargo.toml \
  --target wasm32-unknown-unknown --release
wasm-bindgen --target web --out-dir <scratch> --out-name fandhe_bench \
  <scratch>/target/wasm32-unknown-unknown/release/fandhe_bench.wasm
wasm-opt -Os -g <scratch>/fandhe_bench_bg.wasm -o <scratch>/named.wasm
twiggy top -n 3000 <scratch>/named.wasm
twiggy monos -n 20 <scratch>/named.wasm
twiggy diff <before>/named.wasm <scratch>/named.wasm
```

（`<scratch>` はセッション専用スクラッチパッド配下の隔離ディレクトリ。
`CARGO_TARGET_DIR` はフィクスチャ専用ディレクトリを明示指定する
`.claude/rules/ci.md` の原則を踏襲し、共有 target dir から隔離する。
`twiggy top`/`twiggy monos`/`twiggy diff` はそれぞれ独立したコマンドで
あり、`/` 連結はシェルでは実行できないため各行に分けて記載する）

`#1406` §2.2 と同じ手順。`-Os -g` named wasm は name section 分だけ配布物
（`-Os` のみ）より大きく、内訳の相対比較専用として扱う（payload の正の値
には使わない）。

### 2.3 既知の制約

- payload 計測は `--framework fandhe` の部分実行（`bench/PROTOCOL.md` §2.2
  が許容する経路）
- op_ms（実行時間）ガードは §5 の理由により本イシューでは未実施

## 3. ベースライン再計測（本イシュー着手時、HEAD `8713254`）

`node bench/payload/measure.mjs --framework fandhe` の実測 [再計測]:

| ファイル | raw | gzip |
|---|---:|---:|
| bootstrap.js | 421 B | 204 B |
| fandhe_bench.js | 7,945 B | 2,801 B |
| fandhe_bench_bg.wasm | 98,680 B | 41,339 B |
| **配布合計** | **107,046 B** | **44,344 B** |

`meta.json`: `{"framework":"fandhe","wasm_opt":"116"}`

#1406 記録値（HEAD `1a67469`、wasm gzip 41,345B）との差は -6B（正常なノイズ
範囲）。twiggy 用 named wasm は **121,513 B**（#1406 の 121,516B と 3B 差、
同様に正常範囲）であり、#1406 の内訳（`apply_keyed_list_core` 19,351B・
15.93%、hashbrown+SipHash 系 12,771B・10.51% 相当）がそのまま再現されて
いることを確認した（本レポートでは重複するため表を再掲しない。詳細は
`docs/reports/wasm-payload-baseline-1406.md` §4 参照）。

## 4. 関数別内訳分析と縮減対象の選定

### 4.1 `twiggy monos`（単型化重複）の実測

```
twiggy monos -n 20 named-before.wasm
```

| Apprx. Bloat Bytes | 対象 |
|---:|---|
| 1,755 | `hashbrown::rustc_entry::<impl HashMap>::rustc_entry`（2 単型化） |
| 1,499 | `hashbrown::raw::RawTable::reserve_rehash`（2 単型化） |
| 1,312 | `wasm_bindgen::convert::closures::_::invoke`（多数の小さい単型化、wasm-bindgen 生成コード） |
| 938 | `hashbrown::map::HashMap::insert`（3 単型化） |
| 872 | `Vec::from_iter`（`SpecFromIterNested`、4 単型化） |
| （以下省略） | |
| **9,216（7.58%）** | **全単型化の Apprx. Bloat 合計** |

計画で仮説として挙げていた `replace_root_node<D: RootReplaceDom>`
（`ListElementRootReplace`/`ParentNodeRootReplace` の 2 instantiation）は
`twiggy top -n 40` にも `twiggy monos -n 20` の上位にも**一切現れない**
（トレイトメソッド `WebSysKeyedDom::replace_root`〔484B〕は別シンボルで
あり、混同しないよう注意。ジェネリック本体 `replace_root_node<D>` 自体は
実測サイズが小さすぎてどちらの一覧にも捕捉されないこと自体が「単型化
統合の効果はごく小さい」ことを示す実測結果である）。

**判定（仮説 1・4「単型化重複の集約」）**: 単型化の Apprx. Bloat 合計は
named wasm 全体の 7.58% だが、その大半は `hashbrown`（rustc-entry・
reserve_rehash・insert 系）と `wasm-bindgen` 生成コード（`invoke` 系、
アプリの束縛点の数だけ生成される多重定義）であり、いずれも
`crates/wasm-client` 側のコードを直接書き換えても削減できない
（`hashbrown` は依存クレート内部の単型化、`wasm-bindgen::convert::closures`
は wasm-bindgen マクロ展開由来）。`crates/wasm-client` 自身の関数で単型化
重複が実質的な量を持つものは見当たらなかった。**このレバーは実装対象
なしと判定し、差し戻す（コード変更を行わない）**。

### 4.2 `apply_keyed_list_core` の 19.4KB の内実（inline 展開の実測検証）

`apply_keyed_list_core`（`crates/wasm-client/src/keyed_dom.rs:1473`）自体の
ソース行数は約 100 行（うち大半が rustdoc コメント）だが、shallow bytes は
19,351B（named wasm 比 15.93%）。これは callee（`apply_ops_with_items`
（`keyed_apply.rs:1836`、約 700 行）等）が LLVM/wasm-opt の判断でこの関数へ
インライン展開された結果と考えられる。

**縮減候補として、`apply_ops_with_items` 内の明確に「稀パス」と判定できる
関数**（`resync_required`・rollback・警告 `warn_*`・エラー分岐）に
`#[cold]`/`#[inline(never)]` を付与し、正常系ホットパス（create/update/
clear）へのインライン展開を避ける仮説を実測で検証した（§4.3）。

## 5. 実装レバー: cold path 抑制の試行と実測結果（差し戻し）

### 5.1 適用した変更（一時的、最終的に revert 済み）

`crates/wasm-client/src/keyed_dom.rs` に以下 4 箇所を `#[cold]`/
`#[inline(never)]` 化:

1. `warn_replace_item_children_rollback_failed`（既存の独立関数、ロール
   バック失敗時のみ呼ばれる）
2. `warn_replace_root_rollback_failed`（同上）
3. `replace_list_element_for_tag_change`（`list_element` 自身のタグ変更
   時のみ到達する稀パス）
4. `build_dom_node_with_namespace` 内の `Node::RawHtml` 分岐（新規に
   `warn_skipped_raw_html_node` へ抽出、keyed list 経由の挿入ノードには
   構造的に出現しない防御的フォールバック）

いずれも `unwrap()`/`panic!` を伴わない、呼び出し頻度が明確に低いと判断
できる既存の分岐・関数であり、契約・戻り値・エスケープ挙動は一切変更して
いない。

### 5.2 実測結果

`cargo test -p fandhe-frontend-wasm-client --locked` は全 126+ テスト green
（native 側の振る舞いは不変であることを確認済み）。

payload 実測（`bash bench/csr/fandhe/build.sh` → `measure.mjs`、2 回再現
性確認済み・決定的）:

| 構成 | wasm raw | wasm gzip | 配布合計 raw | 配布合計 gzip |
|---|---:|---:|---:|---:|
| before（変更前、HEAD） | 98,680 B | 41,339 B | 107,046 B | 44,344 B |
| after（cold path 抑制後） | 98,810 B | 41,409 B | 107,176 B | 44,414 B |
| **差分** | **+130 B（+0.13%）** | **+70 B（+0.17%）** | +130 B | +70 B |

**payload は削減ではなく悪化した。** `-Os` 配布物（正の値）は raw +130B・
gzip +70B の悪化。twiggy diff（named wasm、`-Os -g`、相対比較専用）では
named wasm 全体は -48B（`apply_keyed_list_core` 自体は **+474B**
〔`#[inline(never)]` により呼び出し先が非インライン化された分、呼び出し元
に残った準備コードがむしろ増えた〕、他の -341B〔`Vec::from_iter` 単型化〕・
-178B〔"function names" subsection〕という無関係なシンボルの副次的な最適化
順序変化がこれを相殺）。抽出した `warn_replace_item_children_rollback_failed`
自体の diff は +36B（旧呼び出し元 `on_rollback_failed` 側の -36B と対で
ほぼ相殺、本体が移動しただけで縮んではいない）であり、`warn_*` 系の抽出
自体に削減効果はなかった。**named wasm（-48B）と `-Os` 配布物（+130B raw）
とで縮小/悪化の方向が食い違っている点** は、両者が異なる `wasm-opt`
呼び出し（`-Os -g` と `-Os`）である以上あり得る差であり、§2.2 で
明示したとおり named wasm 側は内訳の相対比較専用（payload の正の値では
ない）ため、**payload の合否判定は `-Os` 配布物の実測（+130B raw・+70B
gzip の悪化）を正とする**。

### 5.3 原因分析（判断根拠）

- `wasm-opt -Os` は既定でインライン化の可否をコスト見積もりに基づいて
  自動選択する。今回対象にした 4 箇所はいずれも**呼び出し元が 1〜3 箇所
  のみ**の小さい関数であり、そもそも重複コピーが生じていなかった
  （インライン化で膨張していたのではなく、`wasm-opt` が「呼び出し回数が
  少ない小関数はインライン化した方が総サイズが小さい」と正しく判断して
  いたケース）。`#[inline(never)]` で強制的に非インライン化すると、関数
  呼び出し自体のオーバーヘッド（引数配置・call/return・型シグネチャの
  register）が「インライン化で消えるはずだった重複」を上回り、net で
  払い戻しが負になる
- この結果は #1406 §5.2 の教訓（「ハッシャ切り替えの適応コストが縮小分を
  上回った」）と同じパターンである: **見積もり（「巨大な shallow bytes
  ＝インライン展開の膨張、非インライン化すれば縮む」）は、実際には
  wasm-opt 側のコストモデルによる正しい選択を上書きしただけであり、逆
  方向に効いた**

### 5.4 差し戻し

上記の理由により、`crates/wasm-client/src/keyed_dom.rs` への変更は
`git checkout --` で完全に revert した。本レポート作成時点の
`crates/wasm-client` の実装は HEAD（`8713254`）から**無変更**である。

## 6. 検討したが実装しなかったレバー（判断根拠）

### 6.1 ハッシャ最小差分置換（当初の仮説 3）

`crates/wasm-client` の `keyed_apply.rs`（83 箇所）・`keyed_dom.rs`
（8 箇所）の素 `HashMap`/`HashSet` を軽量ハッシャ（`fx_hash` 型エイリア
ス）へ置換する案は、以下の理由で本イシューでは実装を見送った:

1. **#1406 §5 の実測が既に同型の試行の失敗を記録している**: `core::keyed`
   側のハッシャ切り替え（#1375）は payload を +983B（gzip）悪化させた。
   根本原因は「SipHash 実装バイト自体（596B）はリンクから外れず、
   `Result` 化・エラー型・呼び出し側適応のコード増（+2,958B）がハッシュ
   マップ単型化の縮小（-499B）を上回った」こと（#1406 §5.2）
2. SipHash 実装バイトを実際に回収するには、**wasm-client 内の最後の std
   `HashMap`/`HashSet` 利用者まで変換し切る必要がある可能性が高い**
   （`keyed_apply.rs`・`keyed_dom.rs` に加え `registry.rs`/`timer.rs` の
   static `RefCell<HashMap<...>>` も含む）。本 bench ワークロードでは
   `registry.rs`/`timer.rs` は未到達だが、この懸念は**未検証の仮説**である:
   一般に static な `HashMap` フィールドの型が残る限り
   `RandomState`/`SipHash` の実装コードがリンカの dead-code elimination
   で除去されない可能性はあるが、本イシューでは
   `keyed_apply.rs`/`keyed_dom.rs` のみを変換した部分変換版を実際に
   ビルドして twiggy で SipHash 実装バイトの残存有無を確認する実測は
   行っていない。したがって「`keyed_apply.rs`/`keyed_dom.rs` だけの変換
   では SipHash 実装バイトを回収できない」は実測根拠のない仮説であり、
   #1375 と同じ「適応コストだけが乗って正味悪化する」失敗パターンを
   再現する**リスクがあると見積もっている**（断定はしない）
3. 91 箇所（`keyed_apply.rs` 83 + `keyed_dom.rs` 8）+ `registry.rs`/
   `timer.rs` の全書き換えは、型エイリアスへの機械的差し替えとはいえ
   diff 規模が大きく、本イシューの実測サイクル（レバーごとに独立測定・
   高い確度で採否判断）の範囲を超える。§5 で「小さく確度の高い変更です
   ら実測で逆方向に振れた」ことを踏まえると、大規模な変更を通した上で
   悪化判定になった場合の revert コストも大きい

**再挑戦する場合の前提条件**（#1406 §6 を踏襲・具体化）: (a)
`keyed_apply.rs`・`keyed_dom.rs`・`registry.rs`・`timer.rs` の**全** std
`HashMap`/`HashSet` を同一 PR で一括変換する（§6.1 のとおり部分変換で
SipHash 実装バイトを回収できるかは未検証であり、まず一括変換で検証する
方が高確率での悪化再現を避けられる）、(b) シグネチャ変更（`Result` 化等）
を混在させない最小差分に限定する、(c) 変換後に twiggy で
`core::hash::sip` シンボルの消滅を確認してから payload 実測を確定値とする
（消滅しなければ着手前提が崩れているため即座に revert 判断）。

### 6.2 `registry.rs`/`timer.rs` のハッシャ置換単独

§6.1 のとおり `keyed_apply.rs`/`keyed_dom.rs` を伴わない単独変換で
SipHash 実装バイトを回収できるかは未検証だが、その最大の呼び出し数
（91 箇所中）を占める `keyed_apply.rs`/`keyed_dom.rs` を残したままでは
回収の可能性が低いと見積もられるため、単独では優先度は低いままとする
（#1406 §6 の記録を継承）。

## 7. op_ms 計測について

§5 の cold path レバーは payload 実測の時点で明確に悪化と判定できたため、
op_ms（`bench/csr/run_csr.mjs`）による回帰確認は実施していない（悪化した
レバーを op_ms 面でも正当化する意味がないため）。ハッシャ最小差分置換
（§6.1）は実装自体を見送ったため同様に op_ms 実測対象がない。

なお、次回このレバーへ再挑戦する場合の注意として: 本環境（12 vCPU、QEMU
仮想化ゲスト）での `run_csr.mjs` 実行時間は run 間のばらつきが無視できない
可能性があり、±5% ガードを実測の一点比較だけで判定すると偽陰性/偽陽性の
リスクがある。ベースラインを複数回計測してばらつきの幅を先に把握してから
判定に使うことを推奨する（今回は payload 側で明確に悪化したため、この
論点は本イシューでは検証していない）。

## 8. 結論（#1405 の残レバー判断向け集約）

| 項目 | 値 | provenance |
|---|---:|---|
| 現行 wasm gzip（本イシュー着手時 HEAD, `8713254`） | 41,339 B | [再計測] |
| 現行配布合計 gzip | 44,344 B | [再計測] |
| #1405 目標 gzip | ≈20,000 B | [issue 記録値] |
| 単型化重複（`twiggy monos` 合計 Apprx. Bloat） | 9,216 B（7.58%）、うち `crates/wasm-client` 自身の関数由来はごく僅少 | [再計測] |
| cold path 抑制（`#[cold]`/`#[inline(never)]`）試行結果 | **+70 B gzip（悪化、差し戻し済み）** | [再計測] |
| ハッシャ最小差分置換 | 実装見送り（#1406 の失敗パターン再現リスクが高く、全 std HashMap/HashSet 利用者を一括変換する前提条件を満たせる確度がなかったため） | [判断・未実装] |

**本イシューの受け入れ条件との対応**:

- (a) 関数別内訳の実測と選定根拠の記録: §4 で実施（`twiggy top`/`monos`/
  `diff`）
- (b) payload before/after の実測、または削減不能の判断根拠: §5 で実施
  （唯一試行したレバーは実測により悪化と判明、差し戻し済み。§6 で
  未実装レバーの判断根拠を記録）
- (c) XSS 回帰テスト・既存テスト全 green: `cargo test --workspace --locked`
  を最終差分（本レポート追加のみ、`crates/*/src` 無変更）に対して実行し
  全 green を確認済み（§5.2 の `cargo test -p fandhe-frontend-wasm-client`
  は差し戻し前の中間状態に対する実行であり、この workspace 全体実行が
  最終差分に対する確認である）

DOM 適用層のコードサイズは、少なくとも「明確に稀と判断できる分岐の
非インライン化」という直感的に安全に見えるレバーでは削減できないことが
実測で判明した（`wasm-opt -Os` が既にこの種の最適化を適切に行っている
ため）。target ≈20KB との残差（現行 41,339B との差、約 21,339B）を埋める
には、§6.1 に記した「全 std HashMap/HashSet 利用者の一括ハッシャ変換」の
ような大規模な変更、またはアルゴリズム自体の再設計（`apply_ops_with_items`
の分割・簡略化）等、本イシューの実測サイクルで検証できる粒度を超える
対応が必要になると考えられる。
