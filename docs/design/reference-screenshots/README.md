# reference-screenshots

UI 部品スタイル調整（参考サイト基準への調整、ルート issue #1420）および shadcn/ui 突合（ルート issue #2001）の Issue ツリーで参照するスクリーンショット。

## 内容

- `chakra-<slug>-<n>.png` / `ark-<slug>-<n>.png` / `radixp-<slug>-<n>.png` / `radixt-<slug>-<n>.png`: 参考サイト（chakra-ui / Ark UI / Radix Primitives / Radix Themes）の各部品ページの先頭デモ領域（取得日 2026-08-30、viewport 1280x900）。画像ごとの取得元 URL は `SOURCES.md` を参照
- `shadcn-<slug>-<n>.png`: shadcn/ui（https://ui.shadcn.com）の各部品ページ（`/docs/components/base/<slug>`）の先頭デモ領域（`[data-slot="preview"]` の要素スクリーンショット、最大 3 枚）。`shadcn-charts-<category>-<n>.png` は `/charts/<category>` の各デモカード（iframe を内包する `div.rounded-xl.bg-background`）の要素スクリーンショット、`shadcn-block-<block>-<n>.png` は `/view/new-york-v4/<block>` のブロック単体ビュー（取得日 2026-09-07、viewport 1280x900）。画像ごとの取得元 URL は `SOURCES.md` を参照
- 例外的な取得元: `shadcn-typography-<n>.png` は `/docs/components/base/typography` がコード例のみでデモを持たないため、レンダリング済みの文字組みを示す `/typeset`（プリセット 01 / 03 / 05）のデモ枠を取得している。`shadcn-sidebar-1.png` は同ページ唯一のレンダリング済みデモ（`figure` 内の iframe）1 枚のみで、残りのサイドバー比較資料は `shadcn-block-sidebar-*.png` が担う（コードブロック・インストールコマンドは取得対象にしない）
- `themes-<kebab>.png` / `primitives-<kebab>.png`: 本リポジトリ docs サイト（`make docs` 出力）の各部品ページ Demo 領域（ライトテーマ、同日取得）
- `blocks-<kebab>.png`: 本リポジトリ docs サイトの `/blocks/<kebab>/` フルページ（Blocks セクション、イシュー #2089。合成部品ページのため部品 Demo 領域限定ではなくページ全体を対象とする）
- `wireframe-<kebab>.png` は **意図的に追加しない**。`fandhe-frontend-wireframe-ui`（イシュー #2599 ツリー）が対応範囲の参考とする blocks.pm（https://www.blocks.pm/）は積極的な再配布許諾根拠を持たないため、本ディレクトリへの画像取り込みは見送りと確定した（イシュー #2602。詳細は下記「出典・ライセンス・再配布根拠」節）

## 命名・配置規約（確定版、イシュー #1428）

本ディレクトリはフラット配置（サブディレクトリを持たない）とし、ファイル名は以下の 2 パターンのいずれかに正規化する。

- 参照サイト側: `<site>-<slug>-<n>.png`（正規表現: `^(chakra|ark|radixp|radixt|shadcn)-[a-z0-9-]+-[0-9]+\.png$`）。`site` は `chakra`（chakra-ui、241 枚）/ `ark`（Ark UI、150 枚）/ `radixt`（Radix Themes、116 枚）/ `radixp`（Radix Primitives、26 枚）/ `shadcn`（shadcn/ui、215 枚。部品 186 + charts 21 + blocks 8）の 5 種。`slug` は部品名の kebab-case、`n` は同一部品内での連番（デモのバリエーション違いに対応）
- ローカル側: `<layer>-<kebab>.png`（正規表現: `^(themes|primitives|blocks)-[a-z0-9-]+\.png$`）。`themes-<kebab>.png` は Themes 層（`fandhe-frontend-pre-styled-ui` 相当、`site/nav.toml` 現在登録は 123 部品）、`primitives-<kebab>.png` は Primitives 層（`fandhe-frontend-headless-ui` 相当、`site/nav.toml` 現在登録は 75 部品）、`blocks-<kebab>.png` は Blocks セクション（kebab は `crate::blocks::all_blocks()` の登録 block 名に対応、`site/nav.toml` 現在登録は 7 block、イシュー #2089）。トップレベルのローカルスクショは 2026-08-30 取得セットの `themes-` 107 枚・`primitives-` 63 枚と、その後追加した `blocks-` 5 枚（登録 7 block 中 sidebar-07 / login-04 が未取得）に留まり、#2001 ツリー（shadcn/ui 突合）で増えた部品・block（Themes 16 件・Primitives 12 件・Blocks 2 件）のトップレベルスクショ再取得は本 README では未着手として扱う（`after/` 配下の再取得は #2098 が担う。本 README は命名規約・自己検証コマンドの正であり、枚数の目標値自体は `site/nav.toml` を正とする）

**イシュー当初案（`local/<layer>-<kebab>.png` のようなサブディレクトリ分離）は不採用と確定する。** 根拠は次の 3 点である。

1. `themes-` / `primitives-` のプレフィックスが層情報を既に一意に表しており、ディレクトリ分離による追加情報がない
2. 703 ファイルが main に既にフラット配置でコミット済みであり、移動は差分ノイズが大きい一方で利点がない
3. 画像の参照はコミット SHA 固定の raw URL（後述）で行うため、旧 SHA を指す既存リンクは配置を変えても壊れないが、変更する積極的理由がない

全件がこの規約に一致していることは、以下のコマンドが空出力を返すことで確認できる（自己検証コマンド）。

```bash
# (1) サブディレクトリが存在しないこと（フラット配置の確認。`after/` は下記「`after/`
#     ディレクトリ」節で説明する意図的な例外のため除外する）
find docs/design/reference-screenshots -mindepth 1 -type d -not -path 'docs/design/reference-screenshots/after'
# 空出力なら PASS

# (2) トップレベルファイルが命名規約 2 パターンまたは付随文書 3 点のいずれかに一致すること
find docs/design/reference-screenshots -maxdepth 1 -type f | xargs -n1 basename \
  | grep -vE '^(chakra|ark|radixp|radixt|shadcn)-[a-z0-9-]+-[0-9]+\.png$' \
  | grep -vE '^(themes|primitives|blocks)-[a-z0-9-]+\.png$' \
  | grep -vE '^(README|SOURCES|THIRD_PARTY_NOTICES)\.md$'
# 空出力なら PASS（2026-08-30 時点の 703 枚全件、2026-09-07 の shadcn 215 枚追加後の全件、2026-09-11 時点のトップレベル 923 枚〔blocks 5 枚含む〕で確認済み）

# (3) `after/` 配下がローカル側の命名規約（トップレベルと同一の正規表現。二重定義を避けるため
#     (2) の `(themes|primitives|blocks)` パターンをそのまま再利用する）に一致し、
#     さらに入れ子のサブディレクトリを持たないこと
find docs/design/reference-screenshots/after -mindepth 1 -type d
find docs/design/reference-screenshots/after -maxdepth 1 -type f -exec basename {} \; \
  | grep -vE '^(themes|primitives|blocks)-[a-z0-9-]+\.png$'
# 2 コマンドとも空出力なら PASS
```

`find` でディレクトリツリー全体を列挙するため、`ls '*.png'` 限定と異なり
命名規約違反（誤命名）だけでなくサブディレクトリ配置・非 PNG（GIF・動画等）
ファイルの混入も検出できる。正規表現は single quote 内でバックスラッシュ
1 個（`\.png$`）を使い、ERE の「バックスラッシュ＋任意文字」誤解釈を避ける
（ドットをリテラルとしてエスケープする正しい記法）。
(3) は (1)(2) が対象外とする `after/` 配下の命名規約適合・入れ子ディレクトリ
非存在を担う（正規表現をトップレベルと共有し、二重定義しない）。

`shadcn-*.png` の適用方針（補完参照。既存部品の視覚言語は chakra-ui / Radix
Themes 基準を維持し、shadcn/ui は欠落バリアント・合成パターンの補完にのみ
用いる）は `docs/design/shadcn-reference-adoption-policy.md` を参照。

## ローカルスクショ再取得手順

本リポジトリ docs サイトの部品ページ（Themes / Primitives / Blocks）を撮り直す場合の手順。

1. `make docs` で docs サイトを `dist/` へ SSG ビルドする
2. 任意の静的サーバでローカル配信する（docs-site 自体に serve サブコマンドはないため、汎用ツールを使う。例: `python3 -m http.server 8000 --directory dist`）
3. ブラウザで viewport `1280x900`・ライトテーマに設定し、`http://localhost:8000/themes/<kebab>/` または `http://localhost:8000/primitives/<kebab>/` を開いて Demo 領域のみをスクリーンショットする（`/blocks/<kebab>/` はフルページで `blocks-<kebab>.png` として撮影する）
4. 同名ファイル（`themes-<kebab>.png` / `primitives-<kebab>.png` / `blocks-<kebab>.png`）へ上書きする

撮影時はブラウザの他タブ・ブックマークバー・拡張機能の通知等、個人情報やローカル環境情報（トークン・内部 URL 等）が画面に写り込まないよう注意する。

## 参照サイト側の取得規約

参照サイト（chakra-ui / Ark UI / Radix Primitives / Radix Themes / shadcn/ui）の画像を追加・更新する場合の規約。

- viewport は `1280x900` に統一する
- 撮影範囲は各部品ページのデモ領域のみとし、サイトのロゴ・ヘッダー・商標を含めない
- 取得時は `SOURCES.md` へ当該画像の取得元 URL・取得日を追加する
- 各サイトの MIT ライセンス帰属表示（本 README の帰属表・`THIRD_PARTY_NOTICES.md`）を維持する
- 用途は本リポジトリの UI 部品との視覚比較（設計資料）に限る。それ以外の目的（宣伝・独立した二次配布等）での利用は想定しない

## `after/` ディレクトリ（実装後スクショ。Phase 2 #1420 系・#2001 ツリー #2098）

`after/` は、実装後の docs サイトを撮影したスクリーンショットを置く意図的な
例外ディレクトリである（本ディレクトリはフラット配置が原則、上記「命名・配置
規約」）。ファイル名は **`after/<layer>-<kebab>.png`**（`layer` は `themes` /
`primitives` / `blocks`。命名規約・正規表現はトップレベルのローカル側と同一で、
`after/` 用に別定義は持たない）とする。

トップレベルと `after/` とで撮影対象（Demo 領域限定かフルページか）が層ごとに
反転しており混同しやすいため、以下に明示する。

| プレフィックス | トップレベル（`docs/design/reference-screenshots/<file>.png`） | `after/<file>.png` |
|---|---|---|
| `themes-` / `primitives-` | 各部品ページの Demo 領域のみのクロップ | 各部品ページの**フルページ** |
| `blocks-` | `/blocks/<kebab>/` の**フルページ**（合成部品ページのため） | `## Demo` 節のラッパ `div.blocks-demo`（`crate::blocks::DEMO_CLASS = "blocks-demo"`、`crates/docs-site/src/blocks/mod.rs`）の要素スクリーンショット（全幅 Demo 領域のみ）。合成部品の比較には Demo 領域で足り、フルページ撮影は上記「サイズ方針」の 1 枚あたり 500 KB 上限を超えやすいためトップレベルと逆に領域限定とする |

いずれの層も、撮影条件（base path `/fandhe-frontend/` 付き静的サーバ配信・
viewport 1280 幅・ライトテーマ・写り込み注意）は下記の Phase 2 実績と同一とする。

**Phase 2（イシュー #1420 系）の既存 3 枚の経緯**: `after/themes-<kebab>.png` 3 枚
は、ルート issue #1420 の Phase 2（button / checkbox / checkbox-group のスタイル
調整、PR #1730・#1731・#1734・#1735・#1738・#1739）マージ後に `make docs` で
再ビルドした docs サイトを、base path `/fandhe-frontend/` を含む静的サーバ
（`site/nav.toml` が絶対パスでアセットを参照するため、dist を直接ルート配信する
と CSS 404 でスタイル未適用になる。`python3 -m http.server` で `dist` を
`fandhe-frontend` という名前のディレクトリ配下に見せる、またはリバースプロキシ
で `/fandhe-frontend/` プレフィックスを付与して配信する）から撮影した実装後の
ローカルスクショである（撮影日 2026-09-01、撮影コミット
`chore/phase2-after-screenshots` ブランチのマージコミット、viewport 1280 幅・
ライトテーマ）。

(1) は `after/` を除外するが、`after/` 配下の命名規約適合・入れ子ディレクトリ
非存在は上記自己検証コマンドの (3) が検証する。

`after/` 配下は現状 Phase 2（イシュー #1420 系）の 3 枚のみであり、#2001 ツリー
（shadcn/ui 突合）で新設・変更された部品・block の実装後スクショは含まない。
`after/` 配下への追加取得は #2098 が担い、追加後も (3) が空出力を返すことで
命名規約への適合を確認できる。

## issue への貼り付け手順（raw URL）

Issue コメント・PR 本文へ画像を貼る際は、**コミット SHA 固定**の raw URL を使う。

1. 画像を含むコミットの SHA を取得する（例: `git rev-parse HEAD`、または GitHub 上のコミットページから）
2. 次の形式で URL を組み立て、Markdown 画像記法で貼る。

   ```
   https://raw.githubusercontent.com/Fandhe-AI/fandhe-frontend/<commit-sha>/docs/design/reference-screenshots/<file>.png
   ```

   例: `https://raw.githubusercontent.com/Fandhe-AI/fandhe-frontend/dcd63e31943fc8a4f3991e37dcaf38ff9298b771/docs/design/reference-screenshots/themes-button.png`

**ブランチ名固定の URL（`.../main/docs/design/...`）は使わない。** ブランチ参照は後続コミットで画像が差し替わると閲覧者の見ている内容が変わってしまう（改ざん耐性・リンク安定性が損なわれる）。コミット SHA 固定であれば、対象ファイルが後で改名・移動・削除されても当該 SHA 時点の内容を指し続ける。

## サイズ方針

- 現状実測（2026-09-11 時点、`after/` 3 枚含む）: 926 枚・ディレクトリ計約 18.2 MB・1 枚あたり平均約 19.7 KB・最大約 746 KB（`blocks-signup-01.png`）
- **上限超過の既知事項**: `blocks-signup-01.png`（約 746 KB）・`blocks-login-01.png`（約 560 KB）が下記「1 枚あたり 500 KB 以下」を超過している。Blocks はフルページ撮影のため他プレフィックスより大きくなりやすく、是正（低解像度・圧縮での再出力、または block フルページ向けの上限見直し）は #2098 の再取得作業と合わせて検討する
- 上限方針: 1 枚あたり 500 KB 以下・ディレクトリ総量 30 MB 目安（現状比で余裕を持たせた目安値）
- 形式は PNG のみとする（GIF・動画等は不可）
- **Git LFS は不採用。** 根拠は (1) GitHub の raw URL 経由で LFS ポインタファイル（実体でなくポインタテキスト）が返るため、上記の issue 埋め込み手順がそのままでは機能しなくなる、(2) 追加ツール依存が増える（本フレームワークの依存最小化・自己完結志向〔REQ-3〕と同じ考え方）
- 画像を差し替える場合、git の性質上コミット履歴に旧 blob が残り続けディレクトリ実サイズは単調増加する。差し替えは本当に必要な場合（内容の誤り・レイアウト大幅変更等）に限り、無駄な再取得は避ける

## 出典・ライセンス・再配布根拠

参考サイト由来の画像は、各サイトのドキュメント原稿・デモ実装を含む以下の MIT ライセンスリポジトリの内容を
レンダリングしたものであり、MIT ライセンス（改変・再配布可、著作権表示と許諾表示の保持が条件）に基づき
本リポジトリへ複製・配布する。用途は本リポジトリの UI 部品との視覚比較（設計資料）に限る。

| サイト | 元リポジトリ | ライセンス | 著作権表示 |
|---|---|---|---|
| chakra-ui (https://chakra-ui.com) | https://github.com/chakra-ui/chakra-ui | MIT | Copyright (c) Segun Adebayo |
| Ark UI (https://ark-ui.com) | https://github.com/chakra-ui/ark | MIT | Copyright (c) Chakra UI |
| Radix Primitives / Radix Themes (https://www.radix-ui.com) | https://github.com/radix-ui/website（原稿・デモ）, https://github.com/radix-ui/primitives, https://github.com/radix-ui/themes | MIT | Copyright (c) WorkOS |
| shadcn/ui (https://ui.shadcn.com) | https://github.com/shadcn-ui/ui（原稿・デモ・部品実装・charts・blocks を同一リポジトリで管理） | MIT | Copyright (c) 2023 shadcn |
| blocks.pm (https://www.blocks.pm/、Figma プラグイン「Blocks – Wireframe」) | https://www.figma.com/community/plugin/1332372435133832847/blocks-wireframe（Figma Community 配布） | Community Free Resource License（`https://www.figma.com/legal/community-free-resource-license/`。プラグインページ本体下部に「Licensed under Community Free Resource License」の明示リンクがあり、当該プラグインが無料公開であることも踏まえ本ライセンスの適用を一次情報で確認済み〔確認日 2026-09-22、根拠は下記「blocks.pm ライセンス適用の一次情報確認」節〕。第三者への再配布・derivative work 作成を明示的に禁止し、出力物へのスクリーンショット掲載を許諾する記載はない） | 作者 Hexa（連絡先 love@blocks.pm） |

上記のうち MIT ライセンスの既存 4 サイト（chakra-ui / Ark UI / Radix Primitives・Themes / shadcn/ui）
各リポジトリの LICENSE 全文（MIT 許諾表示）は `THIRD_PARTY_NOTICES.md` に同梱する（画像との対応は `SOURCES.md`）。
blocks.pm は下記のとおり画像取り込みの対象外であり、`THIRD_PARTY_NOTICES.md` への LICENSE 全文同梱の対象にも
含めない（表への記載は出典・ライセンス条件の記録のみを目的とする）。各サイトのロゴ・商標は本ディレクトリに
含めない（取得対象はデモ領域のみ）。

issue 本文からはコミット SHA 固定の raw URL で参照する（詳細は上記「issue への貼り付け手順」節）。

### blocks.pm ライセンス適用の一次情報確認

`https://www.figma.com/community/plugin/1332372435133832847/blocks-wireframe`（blocks.pm の Figma
Community プラグインページ本体）を一次情報として確認したところ、ページ下部のライセンス表示欄に
「Licensed under Community Free Resource License」という明示リンク（リンク先は
`https://www.figma.com/legal/community-free-resource-license/`）があり、当該プラグインは価格表示のない
無料公開（Free）である（確認日 2026-09-22）。Community Free Resource License 自体は条文中で
「このライセンスにリンクするウェブサイト上のダウンロード可能なリソースを対象とする」という一般的な
適用範囲しか定めていないが、Figma 公式ヘルプセンター記事（`Figma Community copyright and licensing`、
`https://help.figma.com/hc/en-us/articles/360042296374-Figma-Community-copyright-and-licensing`）が
「By default, free plugins are published under the Community Free Resource License（無料プラグインは
既定で Community Free Resource License の下で公開される）」と明記しており、この 2 点（プラグインページ
本体の明示リンク + Figma 公式ヘルプセンターの一般規則）を一次情報として突き合わせることで、
blocks.pm（無料プラグイン）への Community Free Resource License 適用を確認済みとする。したがって
下表・下記段落の「Community Free Resource License が適用される」という記述は推測ではなく確認済みの
事実として扱う。

**blocks.pm は上記 4 サイトと異なり画像取り込みの対象外とする（イシュー #2602、fail-closed 判断）。**MIT ライセンスの GitHub リポジトリという積極的な再配布許諾根拠を持つ他 4 サイトに対し、blocks.pm の配布元ライセンス（Community Free Resource License、上記「blocks.pm ライセンス適用の一次情報確認」節のとおり適用を一次情報で確認済み）は第三者再配布・derivative work を明示的に禁止し、スクリーンショット掲載を許諾する記載を持たない（沈黙）。「明示的に禁止されていない」ことを取り込みの根拠にはせず、参照は https://www.blocks.pm/ への外部リンクに限る。この derivative work 禁止は画像取り込みに限らず、blocks.pm の外観・anatomy・プロパティ構成を `fandhe-frontend-wireframe-ui` の実装へ翻案する行為にも及ぶため、`docs/design/wireframe-ui-architecture.md` §2 のとおり実装への転用も書面許諾が得られるまで保留する。**再評価トリガー**: 作者 Hexa（love@blocks.pm）から derivative work 作成（画像取り込み・実装への翻案の双方を含む）についての書面での明示的な許諾が得られた場合。
