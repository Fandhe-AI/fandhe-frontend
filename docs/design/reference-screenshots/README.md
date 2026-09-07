# reference-screenshots

UI 部品スタイル調整（参考サイト基準への調整、ルート issue #1420）および shadcn/ui 突合（後続ツリー）の Issue ツリーで参照するスクリーンショット。

## 内容

- `chakra-<slug>-<n>.png` / `ark-<slug>-<n>.png` / `radixp-<slug>-<n>.png` / `radixt-<slug>-<n>.png`: 参考サイト（chakra-ui / Ark UI / Radix Primitives / Radix Themes）の各部品ページの先頭デモ領域（取得日 2026-08-30、viewport 1280x900）。画像ごとの取得元 URL は `SOURCES.md` を参照
- `shadcn-<slug>-<n>.png`: shadcn/ui（https://ui.shadcn.com）の各部品ページ（`/docs/components/base/<slug>`）の先頭デモ領域（`[data-slot="preview"]` の要素スクリーンショット、最大 3 枚）。`shadcn-charts-<category>-<n>.png` は `/charts/<category>` の各デモカード（iframe を内包する `div.rounded-xl.bg-background`）の要素スクリーンショット、`shadcn-block-<block>-<n>.png` は `/view/new-york-v4/<block>` のブロック単体ビュー（取得日 2026-09-07、viewport 1280x900）。画像ごとの取得元 URL は `SOURCES.md` を参照
- 例外的な取得元: `shadcn-typography-<n>.png` は `/docs/components/base/typography` がコード例のみでデモを持たないため、レンダリング済みの文字組みを示す `/typeset`（プリセット 01 / 03 / 05）のデモ枠を取得している。`shadcn-sidebar-1.png` は同ページ唯一のレンダリング済みデモ（`figure` 内の iframe）1 枚のみで、残りのサイドバー比較資料は `shadcn-block-sidebar-*.png` が担う（コードブロック・インストールコマンドは取得対象にしない）
- `themes-<kebab>.png` / `primitives-<kebab>.png`: 本リポジトリ docs サイト（`make docs` 出力）の各部品ページ Demo 領域（ライトテーマ、同日取得）

## 命名・配置規約（確定版、イシュー #1428）

本ディレクトリはフラット配置（サブディレクトリを持たない）とし、ファイル名は以下の 2 パターンのいずれかに正規化する。

- 参照サイト側: `<site>-<slug>-<n>.png`（正規表現: `^(chakra|ark|radixp|radixt|shadcn)-[a-z0-9-]+-[0-9]+\.png$`）。`site` は `chakra`（chakra-ui、241 枚）/ `ark`（Ark UI、150 枚）/ `radixt`（Radix Themes、116 枚）/ `radixp`（Radix Primitives、26 枚）/ `shadcn`（shadcn/ui、215 枚。部品 186 + charts 21 + blocks 8）の 5 種。`slug` は部品名の kebab-case、`n` は同一部品内での連番（デモのバリエーション違いに対応）
- ローカル側: `<layer>-<kebab>.png`（正規表現: `^(themes|primitives)-[a-z0-9-]+\.png$`）。`themes-<kebab>.png` は Themes 層（`fandhe-frontend-pre-styled-ui` 相当、107 部品と一致）、`primitives-<kebab>.png` は Primitives 層（`fandhe-frontend-headless-ui` 相当、63 部品と一致）

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
  | grep -vE '^(themes|primitives)-[a-z0-9-]+\.png$' \
  | grep -vE '^(README|SOURCES|THIRD_PARTY_NOTICES)\.md$'
# 空出力なら PASS（2026-08-30 時点の 703 枚全件、2026-09-07 の shadcn 215 枚追加後の全件で確認済み）
```

`find` でディレクトリツリー全体を列挙するため、`ls '*.png'` 限定と異なり
命名規約違反（誤命名）だけでなくサブディレクトリ配置・非 PNG（GIF・動画等）
ファイルの混入も検出できる。正規表現は single quote 内でバックスラッシュ
1 個（`\.png$`）を使い、ERE の「バックスラッシュ＋任意文字」誤解釈を避ける
（ドットをリテラルとしてエスケープする正しい記法）。

`shadcn-*.png` の適用方針（補完参照。既存部品の視覚言語は chakra-ui / Radix
Themes 基準を維持し、shadcn/ui は欠落バリアント・合成パターンの補完にのみ
用いる）は `docs/design/shadcn-reference-adoption-policy.md` を参照。

## ローカルスクショ再取得手順

本リポジトリ docs サイトの部品ページ（Themes / Primitives）を撮り直す場合の手順。

1. `make docs` で docs サイトを `dist/` へ SSG ビルドする
2. 任意の静的サーバでローカル配信する（docs-site 自体に serve サブコマンドはないため、汎用ツールを使う。例: `python3 -m http.server 8000 --directory dist`）
3. ブラウザで viewport `1280x900`・ライトテーマに設定し、`http://localhost:8000/themes/<kebab>/` または `http://localhost:8000/primitives/<kebab>/` を開いて Demo 領域のみをスクリーンショットする
4. 同名ファイル（`themes-<kebab>.png` / `primitives-<kebab>.png`）へ上書きする

撮影時はブラウザの他タブ・ブックマークバー・拡張機能の通知等、個人情報やローカル環境情報（トークン・内部 URL 等）が画面に写り込まないよう注意する。

## 参照サイト側の取得規約

参照サイト（chakra-ui / Ark UI / Radix Primitives / Radix Themes / shadcn/ui）の画像を追加・更新する場合の規約。

- viewport は `1280x900` に統一する
- 撮影範囲は各部品ページのデモ領域のみとし、サイトのロゴ・ヘッダー・商標を含めない
- 取得時は `SOURCES.md` へ当該画像の取得元 URL・取得日を追加する
- 各サイトの MIT ライセンス帰属表示（本 README の帰属表・`THIRD_PARTY_NOTICES.md`）を維持する
- 用途は本リポジトリの UI 部品との視覚比較（設計資料）に限る。それ以外の目的（宣伝・独立した二次配布等）での利用は想定しない

## `after/` ディレクトリ（Phase 2 実装後スクショ、イシュー #1420 系）

`after/themes-<kebab>.png` は、ルート issue #1420 の Phase 2（button / checkbox /
checkbox-group のスタイル調整、PR #1730・#1731・#1734・#1735・#1738・#1739）
マージ後に `make docs` で再ビルドした docs サイトを、base path `/fandhe-frontend/`
を含む静的サーバ（`site/nav.toml` が絶対パスでアセットを参照するため、dist を
直接ルート配信すると CSS 404 でスタイル未適用になる。`python3 -m http.server` で
`dist` を `fandhe-frontend` という名前のディレクトリ配下に見せる、またはリバース
プロキシで `/fandhe-frontend/` プレフィックスを付与して配信する）から撮影した
実装後のローカルスクショである（撮影日 2026-09-01、撮影コミット
`chore/phase2-after-screenshots` ブランチのマージコミット、viewport 1280 幅・
ライトテーマ）。トップレベルの `themes-<kebab>.png`（実装前・Demo 領域のみの
クロップ）と異なり、`after/` 配下は各部品ページの**フルページ**スクショである。
本ディレクトリはフラット配置が原則（上記「命名・配置規約」）だが、`after/` は
スタイル調整前後の比較を issue コメントに残す受け入れ条件（各子 issue の受け入れ
条件参照）専用の意図的な例外ディレクトリであり、上記の自己検証コマンド
（サブディレクトリ非存在チェック）は `after/` を対象外とする。

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

- 現状実測（2026-08-30 時点）: 703 枚・ディレクトリ計約 13 MB・1 枚あたり平均約 18 KB・最大約 220 KB
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

各リポジトリの LICENSE 全文（MIT 許諾表示）は `THIRD_PARTY_NOTICES.md` に同梱する（画像との対応は `SOURCES.md`）。各サイトのロゴ・商標は本ディレクトリに
含めない（取得対象はデモ領域のみ）。

issue 本文からはコミット SHA 固定の raw URL で参照する（詳細は上記「issue への貼り付け手順」節）。
