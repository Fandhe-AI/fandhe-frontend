# JS ゼロ SSG での利用ガイド

本ドキュメントはイシュー #1118（利用者フィードバック「styled Accordion が
JS ゼロ SSG で常時閉のまま」）を契機に作成しました。`fandhe-frontend-server`
の `generate_pages`（SSG）で静的サイトを構築し、クライアント側 JavaScript
（`fandhe-frontend-wasm-full` によるハイドレーション）を一切読み込まない
構成を選んだ場合に、`fandhe-frontend-headless-ui` / `fandhe-frontend-pre-styled-ui`
の部品がどう振る舞うかを解説します。

## 1. 前提: 部品の開閉状態は「誰が」変えるのか

`accordion` / `dialog` / `menu` / `select` / `tabs` などの部品は、開閉・選択
などの表示状態を `data-state`（`open`/`closed` 等）属性で表します。この
属性値は、SSR/SSG のビルド時に **Rust コード側が渡した引数**（呼び出し側が
組み立てた状態）でそのまま出力されます。ブラウザ内でクリックに応じて
この属性を書き換えるのは、`fandhe-frontend-wasm-full` が担うクライアント側
ハイドレーション（`crates/wasm-full/src/headless.rs` の「クリックされた
`data-scope`/`data-part` から操作を解決する」配線）の役割です。

つまり、JS ハイドレーションを読み込まない JS ゼロ SSG 構成では、ページ
初期表示の `data-state` はビルド時に固定された値のまま変化しません。
「常時閉に見える」現象は、多くの場合バグではなく、ハイドレーション JS を
読み込んでいないことによる仕様どおりの挙動です。

## 2. どの部品がクリック操作の配線を持つか

クライアント側の配線対象は `crates/wasm-full/src/headless.rs` の
`MAPPING_TABLE`（`(data-scope, data-part)` → 操作の対応表）が一次情報源
です。本書執筆時点で配線済みの `data-scope` は次のとおりです。

- `collapsible` / `dialog` / `popover` / `tooltip` / `menu` / `tabs` /
  `radio-group` / `select` / `signature-pad` / `combobox` / `toggle-group` /
  `tree-view` / `calendar`

**`accordion`**（`fandhe-frontend-pre-styled-ui::accordion` /
`fandhe-frontend-headless-ui::accordion`、`data-scope="accordion"`）は
本書執筆時点で `MAPPING_TABLE` に未登録であり、`fandhe-frontend-wasm-full`
を読み込んだ状態でもクリックによる開閉トリガーは配線されていません。
JS の有無にかかわらず、開閉状態はビルド時に Rust コード側が渡した値の
まま固定されます。この状態は本ガイドの対象外である別課題としてイシュー
#1127 で追跡しています。`fandhe-frontend-wasm-full` 側の対応状況は
`crates/wasm-full/src/headless.rs` の `MAPPING_TABLE` を都度確認してくだ
さい（本書の記述は執筆時点のスナップショットであり、対応表が更新され
次第、本書の記述より対応表を正としてください）。

## 3. JS ゼロ SSG での開閉 UI の代替パターン

開閉インタラクションが必要で、かつクライアント JS を一切読み込まない
構成を選ぶ場合は、ブラウザネイティブの `<details>`/`<summary>` 要素を
使う方法があります。開閉の状態管理をブラウザに委譲できるため、
`fandhe-frontend-wasm-full` のハイドレーションを前提とせずに動作します。

本フレームワーク自身の docs サイト（`crates/docs-site`）でも、右カラムの
目次が `< 1200px` で非表示になる代替として、本文冒頭に折りたたみ目次を
`<details>`/`<summary>` で実装しています（イシュー #1080、
`crates/docs-site/src/layout.rs::toc_inline`）。これを先例として、
ノード木 API のみで開閉 UI を組み立てる最小例を示します。

```rust
use fandhe_frontend_core::{el, text, Node};

/// 開閉可能な補足情報パネルを組み立てる。
///
/// ブラウザネイティブの `<details>`/`<summary>` を使うため、
/// `fandhe-frontend-wasm-full` のハイドレーションを読み込まない
/// JS ゼロ SSG 構成でもクリックで開閉できる（`fandhe-frontend-core` は
/// `details`/`summary` 専用のタグ関数を持たないため、汎用 `el()` を使う。
/// `crates/docs-site/src/layout.rs::toc_inline` と同型のパターン）。
fn disclosure_panel(summary_text: &str, body_text: &str) -> Node {
    let summary = el("summary", vec![], vec![text(summary_text.to_string())]);
    let body = el("div", vec![], vec![text(body_text.to_string())]);
    el("details", vec![], vec![summary, body])
}
```

`fandhe-frontend-core` の描画は既定エスケープ経由のノード木 API のみで
組み立て、HTML 文字列を `format!` 等で直接組み立てるコードは書きません
（REQ-1）。`Theme`（`fandhe-frontend-pre-styled-ui::theme`）のトークンを
参照する CSS を別途スタイルシートへ足すことで、既存の styled 部品と
見た目の一貫性を保てます。

複数項目を同時に開ける・排他制御したいアコーディオン的な UI が必要な
場合も、`<details name="...">` （同じ `name` を持つ `<details>` 同士が
排他的に開閉する、HTML Living Standard の `name` 属性）で JS なしに実現
できます。ただし `name` 属性は比較的新しい機能であり、対応していない
古いブラウザでは単に無視され、各 `<details>` が独立して開閉する（排他
制御されないだけで、開閉自体は引き続き機能する）挙動へ自然劣化します。
致命的な破綻ではありませんが、確定的な排他制御が要件に含まれる場合は
利用者側のブラウザ対応状況を事前に確認してください。

## 4. まとめ

| 観点 | JS ハイドレーションあり | JS ゼロ SSG |
|---|---|---|
| `data-scope` が `MAPPING_TABLE` に登録済みの部品（dialog/menu/select/tabs 等） | クリックで `data-state` が更新される | 初期状態のまま固定（代替として `<details>`/`<summary>` を検討） |
| `accordion`（本書執筆時点で `MAPPING_TABLE` 未登録） | クリックでは更新されない（配線なし） | 初期状態のまま固定（代替として `<details>`/`<summary>` を検討） |
| 開閉不要の静的表示部品（`heading`/`text`/`separator` 等） | 影響なし | 影響なし |

## 関連ドキュメント

- [コンポーネント記述ガイド](./component-authoring.md) — ノード木 API での
  部品組み立ての基本パターン
- [`docs/api/pre-styled-ui-api.md`](../api/pre-styled-ui-api.md) — モジュール
  一覧と crates.io 公開状況
- `docs/api/hydration-api.md` — `fandhe-frontend-wasm-full` のハイドレーション
  契約
