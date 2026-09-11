# shadcn/ui 参照軸の位置づけ・適用原則 決定記録

- **イシュー**: [#2003](https://github.com/Fandhe-AI/fandhe-frontend/issues/2003)（親: #2001「shadcn/ui 突合ツリー」Phase 0）
- **対象**: `docs/policy/intentional-non-adoption.md`（§3.25 の適用対象として参照のみ、本イシューでは変更しない）・Phase 1〜3 の各部品突合イシュー（#2008〜#2056）
- **関連**: `docs/design/color-token-system.md`（#1422）・`docs/design/pre-styled-ui-scale-tokens.md`（#1423）・`docs/design/pre-styled-ui-interaction-visual-language.md`（#1425）・`docs/design/pre-styled-ui-size-and-color-palette-axes.md`（#1678）・`docs/design/pre-styled-ui-data-attr-vocabulary.md`（`data-*` 語彙）・`docs/design/component-coverage-map.md`（#2004 が更新）・`docs/internal/pre-styled-ui-golden-test-update-guide.md`

## 1. 背景

#1420（107→110 部品を chakra-ui / Radix Themes 基準へ調整し golden テストでバイト一致固定）の直後に shadcn/ui を無条件の「主基準」として取り込むと、golden 全体の再 churn と視覚言語のハイブリッド化（chakra 風と shadcn 風の混在）を招く。ルート #2001 は起票時点の前提として、既に「補完参照」を採用方針として明記している（#1422・#1423・#1425 の決定を無効化しない、golden 再 churn 回避が根拠）。本記録はこの方針を正式な決定記録として固定し、Phase 1〜3（#2008〜#2056、計 46 イシュー）が実際に参照できる粒度の判定基準を明文化するものであり、位置づけの再検討（主基準化への転換）は本記録のスコープ外とする。本記録は 2026-09-07 のユーザー判断により改訂され、改訂内容は §8 を正とする。

## 2. 決定事項

| 論点 | 決定 | 根拠 |
|---|---|---|
| 位置づけ | **主基準の 1 つ**（chakra-ui / Radix Themes / shadcn-ui の 3 者共同、pre-styled-ui の視覚言語に限る。2026-09-07 改訂、§8） | 既存決定記録（#1422/#1423/#1425）の無効化を避けつつ、新規追加分での視覚言語参照軸を拡張（§8） |
| 優先順位が競合した場合 | 部品ごとに判断し、当該イシュー / PR 本文へ定型行で記録する（§8） | 「3 者共同」の実装方針として、単一優先順位の固定より柔軟な部品ごと判定を採用（§8） |
| golden への影響方針 | 既存 variant の CSS 出力は原則バイト同一を維持する「純追加」を基本とする。既存出力の変更が避けられない場合は、当該 Phase イシュー本文に理由を明記する | `docs/internal/pre-styled-ui-golden-test-update-guide.md` の運用と整合 |

## 3. 判定基準（Phase 1〜3 が参照する一次フィルタ）

### 合わせる（追加候補）

- shadcn/ui にしか無い variant / size / state で、既存の軸（`docs/design/pre-styled-ui-size-and-color-palette-axes.md` の size / ColorPalette 軸）へ無理なく写像できるもの
- 既存部品に欠けている anatomy slot（構造上の欠落）
- shadcn/ui の合成パターン（Examples）のうち、本リポジトリの既存部品の組み合わせで再現可能なもの（docs サイト部品ページの Examples 節へ Rust コードとして転記する。§4 参照）

### 合わせない（非対象）

- 色味・角丸・影・spacing・フォントサイズなど「トークン値」の差のみ（#1422/#1423/#1425 の決定が優先する。ただし §8 の部品ごと判断で shadcn の値を採ると記録した新規追加分は例外）
- shadcn 固有の実装語彙（Tailwind ユーティリティクラス・`cn()`・`data-slot` 属性名）をそのまま持ち込むこと。`data-*` 語彙は `docs/design/pre-styled-ui-data-attr-vocabulary.md` の既存規約に従う
- バリデーション・送信処理・データ整形・永続化等のアプリケーションロジック（`docs/policy/intentional-non-adoption.md` §3.25 規則 1）
- 装飾・アニメーション・レイアウト計測の関心を headless 層（`crates/headless-ui/`）へ混入すること（同 §3.25 規則 2。必要なら `crates/pre-styled-ui/` 側の責務として設計する）
- shadcn 側にのみ存在するが本リポジトリの製品スコープ外の関心（採否判定は #2006 に委ねる）

## 4. Examples（合成パターン）転記の扱い

- shadcn/ui の Examples ページのコード（TSX・Tailwind クラス）はそのまま複製しない（ライセンス上は複製可能だが、本リポジトリの API 形状・既定エスケープ方針と異質なため）
- 転記は「合成の考え方」のみを参照し、コードは `crates/docs-site/src/component_specs*` 配下の Examples 節へ本リポジトリの API（ノード木 API。`format!` 等による HTML 文字列直接組み立ては REQ-1 違反のため禁止）で書き直す
- スクリーンショットは `docs/design/reference-screenshots/shadcn-*.png` をコミット SHA 固定 raw URL でイシューコメントに参照するのみとし、docs サイトのビルド生成物（`assets/*.svg` 等）には転用しない

## 5. 適用範囲

Phase 1〜3（#2008〜#2056）は本記録の判定基準を「合わせる／合わせない」の一次フィルタとして使う。個別部品固有の判断が必要な場合は当該イシュー内で追加検討し、本記録を上書きしない。

## 6. スコープ外（姉妹イシューへの委譲）

本記録が扱わない事項は以下のイシューに委ねる。決定の重複・矛盾を避けるためここでは判断しない。

- `docs/design/component-coverage-map.md` への shadcn 列追加・部品ごとの対応表（Part A〜E 全行）→ #2004
- テーマトークン（neutral oklch・`--radius` スケール・chart・sidebar トークン、`theme.rs` の変更）→ #2005
- shadcn 固有部品（data-table / questionnaire / message-scroller / 会話系 4 部品 / direction / aspect-ratio / form / utils）の採否・責務境界判定 → #2006（`docs/policy/intentional-non-adoption.md` §3.25 適用）
- blocks の置き場所（docs-site `/blocks/` vs examples）→ #2007

## 7. 意図的に採らなかった案（改訂前）

- **主基準化**: shadcn/ui の視覚言語（色・角丸・影・spacing）を既存部品へ全面適用する案。#2003 時点では golden 全面 churn・既存決定記録（#1422/#1423/#1425）の無効化を理由に不採用としたが、2026-09-07 のユーザー判断で改訂（§8）。ただし golden 全面 churn を伴う形は引き続き採らない
- **判定基準を Phase イシューごとに都度決める運用**: Phase 1〜3 が 46 件あり、判定のブレ・重複議論を避けるため事前の一括基準化を採用した

## 再評価トリガー

#2003 時点の旧トリガーは §8 改訂で充足済み・消費済み。新トリガーとして、部品ごと判断の記録が Phase 1〜3 で判定不能・矛盾を多発（目安 5 件以上）した場合は固定優先順位方式へ戻すことを再検討する。判定結果は §8「再評価トリガーの判定（2026-09-11、#2158）」を参照。

## 8. 改訂（2026-09-07、ユーザー判断、イシュー #2153）

ユーザー判断（2026-09-07）により、本記録は以下 5 点を確定した。改訂作業の追跡はイシュー #2153（親 #2001）で行う。

### 改訂内容

1. **主基準化**: chakra-ui / Radix Themes に加え、shadcn-ui を pre-styled-ui（Themes 層）の視覚言語の主基準として格上げ。「補完参照」から「主基準の 1 つ」へ昇格。
2. **部品ごと判断**: 3 者で値が競合した場合は単一優先順位ルールではなく、部品ごとに判断し、当該イシュー / PR 本文に定型行で記録する。定型行形式:
   ```
   参照競合の判定: <部品> の <要素> は <chakra-ui / Radix Themes / shadcn-ui> の値を採る。理由: …
   ```
3. **golden への影響方針（不変）**: 既存 variant の CSS 出力は原則バイト同一を維持する「純追加」。既存出力の変更が避けられない場合は当該イシュー本文に理由を明記。つまり「主基準化」は新規追加分（欠落 variant / state / 合成パターン / 新規部品 / blocks / charts）で shadcn の値を採ってよいという意味であり、既存出力の再 churn の許可ではない。
4. **headless-ui（Primitives 層）は不変**: 一次参照軸は ark-ui のまま維持。本改訂は pre-styled-ui の視覚言語のみに適用。`data-*` 語彙規約（`docs/design/pre-styled-ui-data-attr-vocabulary.md`）・size / ColorPalette 軸（`docs/design/pre-styled-ui-size-and-color-palette-axes.md`、#1678）は不変で、shadcn 固有語彙（`data-slot` / `cn()` / Tailwind クラス / shadcn の variant 名そのまま）の持ち込み禁止は維持。
5. **`docs/policy/intentional-non-adoption.md` §3.25 規則 1・2 は不変**: バリデーション・送信・整形・永続化等のアプリケーションロジックを UI 部品へ内包しない（規則 1）。装飾・アニメーション・レイアウト計測を headless 層へ混入しない（規則 2）。本改訂の前提条件である。

本記録 §8 がこれら 5 点の正であり、Phase 1〜3 の各イシュー本文にある「shadcn/ui は補完参照」の定型文は本 §8 の読み替えで運用する（イシュー本文が「Phase 0 の適用原則に従う」と委ねているため本記録が正）。

### 再評価トリガーの判定（2026-09-11、イシュー #2158）

**判定対象・母数**: Phase 1〜3（#2008〜#2056）の closing PR 46 件のうち、§8 改訂（2026-09-07、PR #2155）以前にマージ済みの 11 件（#2009/#2010/#2011/#2012/#2014/#2015/#2027/#2028/#2029/#2030/#2032、イシュー本文に定型行テンプレートがなく記録義務の対象外）を除いた **35 件** を母数とする。

**情報源**: 「参照競合の判定」の定型行は PR 本文に限らず 4 経路に分散して記録されている。(1) closing PR 本文（#2177 date-picker、#2238 table の 2 件）、(2) closing PR のコミット本文（#2231 progress の 1 件）、(3) issue コメント（#2018 radio-group / #2024 toggle-group / #2026 accordion / #2035 navigation-menu / #2038 splitter / #2041 tooltip / #2045 badge / #2046 card / #2050 skeleton / #2055 text / #2056 heading・blockquote・list・code の 11 issue）、(4) rustdoc（`crates/pre-styled-ui/src/` 19 ファイル + `crates/headless-ui/src/data_table.rs`、計 37 箇所）。

**集計結果**: 4 経路すべてを対象に `参照競合の判定` を含む記述を横断的に収集し、「判定不能」（競合を挙げながら採用軸を確定していない記述）・「矛盾」（同一部品・同一要素で異なる参照軸を採った記録、または理由の説明なく判断が割れた記録）に該当する記述を検索した結果、**該当 0 件**だった（`crates/pre-styled-ui/src` / `crates/headless-ui/src` / 本ポリシー文書 / `docs/design/pre-styled-ui-forms-disabled-required-matrix.md` に対する `判定不能` / `矛盾` の全文検索でも、参照競合の判定と無関係な箇所〔設計上の非矛盾を確認する記述・条件式のコメント等〕以外に一致なし）。抽出できた判定はいずれも「理由付きで単一の参照軸（多くは chakra-ui / Radix Themes、一部 shadcn-ui）を採る」形か、「競合なし（純追加のみ）」の明示（#2041 tooltip、#2038 splitter 等）のいずれかであり、35 件中で判定不能・矛盾は **0 件**（目安の 5 件を大きく下回る）。

**判定結果**: 再評価トリガーは**未発火**。§2 の「部品ごと判断」方式（固定優先順位方式への回帰）を再検討する必要はなく、§8 改訂の運用を継続する。

**運用上の所見**: 定型行の記録先が PR 本文だけでなく issue コメント・コミット本文・rustdoc の 4 経路へ分散していることが判明した。矛盾ではなく記録先の分散であり、今後 §8 の運用を監査する際は PR 本文のみでなく 4 経路を横断して確認する必要がある。

**本判定のスコープ外**: `crates/pre-styled-ui/src/{collapsible,hover_card,accordion,input,drawer}.rs` の rustdoc に旧「補完参照」表記が残存している（#2161 後の残存、5 ファイル）。表記の是正は `crates/` の `src/` 変更を伴い version-bump-guard の対象となるため、本イシュー（#2158、docs のみ）のスコープ外として別 Issue 化を提案する。
