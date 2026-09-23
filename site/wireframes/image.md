# Image

`fandhe-frontend-wireframe-ui` の画像プレースホルダーです。対角のバツ印が
入った正方形（または円形）の枠で、画像が入る場所をワイヤーフレーム上に
示す非インタラクティブな部品です。API は `image(content, size, circle,
primary)` の 4 引数（詳細は下の引数表を参照）で、props 構造体は導入して
いません。

視覚的な参照元は [blocks.pm](https://www.blocks.pm/) の Image 部品ですが、
ライセンス上の理由によりスクリーンショットは掲載していません（詳細は
`docs/design/wireframe-ui-architecture.md` §2）。参照したい場合は外部リンク先を
直接確認してください。

本部品は表示専用のため `<img>` は出力せず、画像 URL を受け取る API も
持ちません。実際に画像を表示する部品が必要な場合は [Image（Themes）](../themes/image.md)
を検討してください（対応する Primitives 部品はありません）。

## 原案差分メモ

- **API は独自設計**: `docs/design/wireframe-ui-architecture.md` §4・§6 の
  Figma プロパティ変換規約（円形・強調・プレースホルダー有無）から独立
  設計しました。原案の Figma プロパティ構成をそのまま転写したものでは
  ありません。
- **`<img>` と URL API は持たない**: ルートは `div` とし、
  `src`/`href`/`style` は一切出力しません（同文書 §7 の非対話制約）。
  画像 URL を受け取る API も設けていません（外部リソース読み込みの経路を
  作らないため）。
- **バツ印は CSS グラデーションで描き、トークン色のグレースケールを使う**:
  疑似要素を `rotate()` で回す方式ではなく、`background-image` の
  `linear-gradient` 2 本（角から角へ）を重ねて描きます。角へ正確に届く・
  アスペクト比が変わっても崩れない・追加マークアップが要らない、という
  利点があります。線色は黒塗り二値ではなく `--fw-wire-line`（既定）の
  トークンを使います。
- **circle は avatar と同型の部品固有修飾 class で、`props` へは昇格しない**:
  `circle: bool` は `crate::avatar`/`crate::frame` の
  `circle`/`bordered: bool` と同型の判断で、部品固有の修飾 class
  （`fw-wire-image-circle`）として表現します。2 部品目（avatar・image）で
  横断再利用の兆候が出ていますが、`crate::props` への型昇格は本 PR の
  スコープ外としました。
- **黒塗りは `Primary` の opt-in**: 既定はグレースケール（`--fw-wire-fill-subtle`
  の背景と `--fw-wire-line` の線）で、黒塗り（反転配色）は既存の共通型
  `Primary` を渡したときだけ有効になります（新しい型は追加していません）。
- **`content` スロットは §11.4 に準拠する（avatar のような逸脱はない）**:
  `content` が `None` のときはバツ印プレースホルダーを描き、子要素は一切
  出力しません。`Some(node)` を渡した場合はそのノードをそのまま子要素に
  します（画像アイコンやキャプションテキストへの差し替えが可能です）。
  `crate::avatar` が `icon::user` へフォールバックする逸脱とは異なり、本
  部品は「`None` ならスロット要素を出力しない」という §11.4 の原則に
  そのまま従います。
- **正方形固定で、アスペクト比のバリアントは対象外**: 寸法は常に正方形
  （`--fw-wire-control-size` の 3 倍）で、16:9 等のアスペクト比バリアント
  は本 PR のスコープ外です。
- **スクリーンショット非掲載**: `docs/design/reference-screenshots/` への
  blocks.pm 画像取り込みは #2602 で fail-closed に非掲載と確定しています。
  本ページも画像は置かず、上記の外部リンクのみで代替します。
