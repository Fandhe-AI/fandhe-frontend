# Color Picker

HSV 色相環 + アルファ選択のポップオーバー型入力です。色領域・チャンネルスライダーの見た目は CSS グラデーションと thumb 位置のみで表現し、`canvas`/`web-sys` には依存しません（値の算出は決定的な純粋関数が担います）。

`fandhe-frontend-headless-ui` の `color_picker` mod が提供する構造・アクセシビリティ（WAI-ARIA・キーボード操作）・表示状態（`data-*`）のみを持つ unstyled 部品です。Themes 版が備える見た目（実際のグラデーション CSS 等）は持たず、anatomy・ARIA・`data-*` のみを提供します。CSS は利用者が当てます。

`ColorPickerProps`（`disabled`/`readonly`/`invalid`/`required`）が Root/Label/Control/Trigger/Area/AreaBackground/AreaThumb/ChannelInput へ `data-disabled`/`data-readonly`/`data-invalid` を一律付与し、Label のみ追加で `data-required` を付与します（`required` は `<input type="hidden">` である HiddenInput には効果を持たせません）。ChannelSlider/ChannelSliderTrack/ChannelSliderThumb は `Channel::as_str()` の固定語彙（`hue`/`saturation`/`value`/`alpha`）による `data-channel` と、`Orientation` 引数による `data-orientation` を出力します（ChannelSliderThumb には `aria-orientation` も付与）。ChannelInput は `data-channel="hex"` 固定リテラルを出力し、`readonly` のときネイティブ `readonly` 属性、`invalid` のとき `aria-invalid="true"` を追加します（valid のときは `aria-invalid` 自体を省略します）。

**キーボード操作**

| キー | 対象 | 効果 |
|---|---|---|
| Enter | Trigger | popover を開く（dispatch `"open"`） |
| Enter | ChannelInput | HEX 文字列を確定する（dispatch `"set_hex"`） |
| Esc | Content | popover を閉じる（dispatch `"close"`。trigger へのフォーカス復帰は呼び出し側の DOM 配線が担います） |
| ArrowLeft / ArrowRight | ChannelSliderThumb | 該当チャンネルを 1 減らす/増やす（dispatch `"decrement"`/`"increment"`、`0..=Channel::max()` へ clamp、ラップしません） |
| ArrowUp / ArrowDown | AreaThumb | 彩度・明度をそれぞれ 1 単位変化させる（dispatch `"set_channel"` で表現） |
| Home / End | ChannelSliderThumb | 最小値・最大値へ設定する（dispatch `"set_channel"` の `0`/最大値ペイロードで表現） |

ポインタ・キーボード操作の実際の DOM 配線（`fandhe-frontend-wasm-full`）は本イシューのスコープ外です。本部品は SSR 静的マークアップと dispatch 契約のみを提供します。

スタイル済みの表示例は [Color Picker](../themes/color-picker.md) を参照してください。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)

**自前 CSS の最小例**

Themes 版を使わず本部品を直接使う場合、`[data-scope="color-picker"][data-part="..."]` セレクタでスタイルを当てます。以下は area/area-thumb の配置、hue スライダーのグラデーション、状態セレクタの最小例です。

```css
[data-scope="color-picker"][data-part="area"] {
  position: relative;
  width: 100%;
  aspect-ratio: 4 / 3;
}

[data-scope="color-picker"][data-part="area-thumb"] {
  position: absolute;
  /* --fandhe-color-picker-x/-y は呼び出し側が area-thumb の attrs
     経由で style 属性に設定する（例:
     ("style", "--fandhe-color-picker-x: 40%; --fandhe-color-picker-y: 30%")）。 */
  left: var(--fandhe-color-picker-x, 0%);
  top: var(--fandhe-color-picker-y, 0%);
  width: 0.75rem;
  height: 0.75rem;
  border-radius: 50%;
  border: 2px solid #fff;
  transform: translate(-50%, -50%);
}

[data-scope="color-picker"][data-part="hue-slider-track"] {
  height: 0.75rem;
  border-radius: 999px;
  background: linear-gradient(
    to right,
    red,
    yellow,
    lime,
    cyan,
    blue,
    magenta,
    red
  );
}

[data-scope="color-picker"][data-part="root"][data-disabled] {
  opacity: 0.5;
}

[data-scope="color-picker"][data-part="channel-input"][data-invalid] {
  outline: 2px solid crimson;
}

[data-scope="color-picker"][data-part="trigger"][data-state="open"] {
  outline: 2px solid #333;
}
```
