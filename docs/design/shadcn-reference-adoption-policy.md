# shadcn/ui 参照軸の位置づけ・適用原則 決定記録

- **イシュー**: [#2003](https://github.com/Fandhe-AI/fandhe-frontend/issues/2003)（親: #2001「shadcn/ui 突合ツリー」Phase 0）
- **対象**: `docs/policy/intentional-non-adoption.md`（§3.25 の適用対象として参照のみ、本イシューでは変更しない）・Phase 1〜3 の各部品突合イシュー（#2008〜#2056）
- **関連**: `docs/design/color-token-system.md`（#1422）・`docs/design/pre-styled-ui-scale-tokens.md`（#1423）・`docs/design/pre-styled-ui-interaction-visual-language.md`（#1425）・`docs/design/pre-styled-ui-size-and-color-palette-axes.md`（#1678）・`docs/design/pre-styled-ui-data-attr-vocabulary.md`（`data-*` 語彙）・`docs/design/component-coverage-map.md`（#2004 が更新）・`docs/internal/pre-styled-ui-golden-test-update-guide.md`

## 1. 背景

#1420（107→110 部品を chakra-ui / Radix Themes 基準へ調整し golden テストでバイト一致固定）の直後に shadcn/ui を無条件の「主基準」として取り込むと、golden 全体の再 churn と視覚言語のハイブリッド化（chakra 風と shadcn 風の混在）を招く。ルート #2001 は起票時点の前提として、既に「補完参照」を採用方針として明記している（#1422・#1423・#1425 の決定を無効化しない、golden 再 churn 回避が根拠）。本記録はこの方針を正式な決定記録として固定し、Phase 1〜3（#2008〜#2056、計 46 イシュー）が実際に参照できる粒度の判定基準を明文化するものであり、位置づけの再検討（主基準化への転換）は本記録のスコープ外とする。

## 2. 決定事項

| 論点 | 決定 | 根拠 |
|---|---|---|
| 位置づけ | **補完参照**。既存部品の視覚言語（色・角丸・影・spacing・フォント）は chakra-ui / Radix Themes 基準（#1422/#1423/#1425）を維持し、shadcn/ui は欠落バリアント・状態・合成パターン・新規部品・blocks/charts の補完にのみ用いる | #1420 直後の再 churn 回避、既存決定記録の無効化回避、golden 更新コスト |
| 優先順位が競合した場合 | chakra-ui / Radix Themes の値が shadcn/ui の値に優先する。shadcn 側にしか無い要素のみ補完対象になる | 「補完参照」の定義上の帰結 |
| golden への影響方針 | 既存 variant の CSS 出力は原則バイト同一を維持する「純追加」を基本とする。既存出力の変更が避けられない場合は、当該 Phase イシュー本文に理由を明記する | `docs/internal/pre-styled-ui-golden-test-update-guide.md` の運用と整合 |

## 3. 判定基準（Phase 1〜3 が参照する一次フィルタ）

### 合わせる（追加候補）

- shadcn/ui にしか無い variant / size / state で、既存の軸（`docs/design/pre-styled-ui-size-and-color-palette-axes.md` の size / ColorPalette 軸）へ無理なく写像できるもの
- 既存部品に欠けている anatomy slot（構造上の欠落）
- shadcn/ui の合成パターン（Examples）のうち、本リポジトリの既存部品の組み合わせで再現可能なもの（docs サイト部品ページの Examples 節へ Rust コードとして転記する。§4 参照）

### 合わせない（非対象）

- 色味・角丸・影・spacing・フォントサイズなど「トークン値」の差のみ（#1422/#1423/#1425 の決定が優先する）
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

## 7. 意図的に採らなかった案

- **主基準化**: shadcn/ui の視覚言語（色・角丸・影・spacing）を既存部品へ全面適用する案。golden 全面 churn・既存決定記録（#1422/#1423/#1425）の無効化を理由に不採用
- **判定基準を Phase イシューごとに都度決める運用**: Phase 1〜3 が 46 件あり、判定のブレ・重複議論を避けるため事前の一括基準化を採用した

## 再評価トリガー

Phase 1〜3 実施の結果、本記録の判定基準では割り切れないケースが一定数（目安 5 件以上）発生した場合、または将来 #1420 系の golden が再度全面更新される機会があれば、補完参照から主基準化への転換を再検討する。
