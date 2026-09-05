# Download Trigger

`a[download]` 属性による宣言的なファイルダウンロードトリガーです。`Blob` 生成のような JS 実行を必要とせず、プレーンな HTML のみで完結する静的部品として実装されています。`root`（`a[href][download]`）1 パーツのみの最小構成で、時間変化する内部状態を持たないため `data-*` の状態語彙は一切出力しません。

Themes 版（`/themes/download-trigger/`）は Button recipe を流用した既定 CSS を追加する薄いラッパーであり、本 Primitives 部品自体は CSS を一切持ちません。スタイル済みの表示例は [Download Trigger](../themes/download-trigger.md) を参照してください。

**キーボード操作**

| キー | 対象 | 効果 |
|---|---|---|
| Tab | Root | ネイティブ `a[href]` へのフォーカス移動です。 |
| Enter | Root | ネイティブ `a[href]` の起動（ダウンロード開始）です。**Space はリンクを起動しません**（`<a>` 要素は Space キーでは発火しないブラウザ標準挙動です）。 |

**参考サイトとの差分**

ark-ui / chakra-ui の DownloadTrigger は `Blob` 生成・非同期データ解決を行う JS ユーティリティで、`<button type="button">`（chakra は `asChild` で任意要素へ差し替え可能）を起点とし、Anatomy 節・Accessibility 節を持ちません（`data-*` 状態語彙も ARIA 属性の付与もありません）。本実装は以下の点で意図的に差分を残しています。

- **要素種別**: 参考サイトの `button` に対し、本実装は `a[href][download]` を採用します（プレーンな HTML を尊重する静的部品化方針）。この差により、参考サイトでは効く Space キーでの起動が本実装では効きません（上記キーボード操作表参照）。
- **`data-scope`/`data-part`**: 参考サイトには anatomy の概念自体が無く、本実装の `data-scope="download-trigger"` / `data-part="root"` はこちら側の superset です。
- **非対応の prop**: `data`（`Blob`/`ArrayBuffer`/非同期関数）・`mimeType`・`asChild` は非対応です。`Blob` 生成はクライアント JS 前提であり静的部品化方針の対象外、実ファイル配信時の `Content-Type` は配信側ヘッダで表現します。
- **`disabled` の非提供**: `a` 要素にはネイティブの disabled 意味論がなく、`root` は常に `href` 属性を出力するため、呼び出し側 `attrs` で `aria-disabled="true"` + `tabindex="-1"` を付与しても**クリック（および Enter キー）でのダウンロード起動は実際には防げません**（`aria-disabled` は状態を伝えるだけ、`tabindex="-1"` は Tab 移動対象から外すだけで、`href` を保持したままの `a` はブラウザ標準動作としてクリック起動可能です）。無効状態が必要な場合は、呼び出し側で `download_trigger::root` の呼び出し自体を止め、非操作要素（`span`/`disabled` な `button` 等）へ描画を差し替えてください。
- **cross-origin での `download` 無視**: ブラウザ仕様上、`download` 属性は same-origin（および `blob:`/`data:`）以外のリンクでは無視され、通常のナビゲーションとして扱われます。

一方で「状態を表す `data-*` を出力しない」「`role`/`aria-*` を独自付与しない」点は参考サイトと一致しています。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)

**自前 CSS の最小例**

Themes 版を使わず本部品を直接使う場合、`[data-scope="download-trigger"][data-part="root"]` セレクタと `:focus-visible` 擬似クラスでスタイルを当てます。状態を表す `data-*` が無いため、当てられるセレクタはこの 2 種類のみです。

```css
[data-scope="download-trigger"][data-part="root"] {
  display: inline-flex;
  align-items: center;
  gap: 0.375rem;
  padding: 0.5rem 0.875rem;
  border: 1px solid #888;
  border-radius: 6px;
  text-decoration: none;
}

[data-scope="download-trigger"][data-part="root"]:focus-visible {
  outline: 2px solid #2563eb;
  outline-offset: 2px;
}
```
