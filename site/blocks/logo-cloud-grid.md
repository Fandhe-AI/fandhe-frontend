# logo-cloud-grid

見出し・リード文の下に、グレースケールのロゴを折り返し行またはグリッドで並べる block です。

`heading` / `text` / `image` / `card` / `tag` / `link` の 6 部品だけで組み立てており、新しい UI 部品は追加していません。

5 つのバリエーションを縦に併記しています。

- **A**: タグライン（tag）→ 見出し → リード文 → ロゴ 5 個の折り返し行
- **B**: 見出し無し、リード文 → ロゴ 6 個の折り返し行
- **C**: 見出し + 「GitHub で見る」リンクの見出し行 → 社名タグ付きロゴカード 6 枚
- **D**: 見出し無し、淡色枠のロゴタイル 6 枚のグリッド
- **E**: ロゴ 5 個の折り返し行 → 下にピル型の告知リンク

列数は `md`（768px）で 3〜4 列、`lg`（1024px）で 6 列に広がります（形 C/D のグリッド）。

ロゴ画像は同一のプレースホルダーの反復のため `alt=""`（装飾用途）にし、グレースケール表現は CSS の `filter: grayscale(1)` で与えています。各ロゴには可視の社名ラベル（`tag`）を必ず添え、スクリーンリーダー利用者へもロゴ列の内容が伝わるようにしています。`<form>` は使わず、送信処理・データ取得を持たない静的な合成例です。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::heading::{self as styled_heading, HeadingLevel, HeadingProps};
use fandhe_frontend_pre_styled_ui::image::{self, ImageFit, ImageProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::tag::{self, TagProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};

/// 実在する GitHub リポジトリへの外部絶対 URL（`cta_feature_links` と
/// 同じ固定値。死リンク `href="#"` を出さないための実在先）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

const LOGO_ATTR: &str = "data-blocks-logo-cloud-grid-logo";

/// ロゴ 1 枚（`alt=""`、装飾用途）。
fn logo() -> Node {
    image::image(
        &ImageProps {
            fit: ImageFit::Contain,
            ..ImageProps::new(dummy_assets::LOGO_SRC, "")
        },
        vec![(LOGO_ATTR, "")],
    )
}

/// ロゴ 1 枚 + 可視の社名ラベル（`tag`）。画像は装飾（`alt=""`）のまま、
/// 社名はテキストとして読み上げ可能にする（Codex レビュー指摘、PR
/// #3244）。
fn logo_item(name: &str) -> Node {
    let name_tag = tag::root(
        &TagProps::default(),
        vec![],
        vec![tag::label(vec![], vec![text(name)])],
    );
    div(
        vec![("class", "blocks-logo-cloud-grid-logo-item")],
        vec![logo(), name_tag],
    )
}

/// 形 A: タグライン（tag）→ 見出し → リード文 → ロゴ 5 個の折り返し行。
fn variant_a() -> Node {
    let tagline = tag::root(
        &TagProps::default(),
        vec![],
        vec![tag::label(vec![], vec![text("Trusted by")])],
    );
    let title = styled_heading::heading(
        HeadingLevel::H3,
        &HeadingProps::default(),
        vec![],
        vec![text("あらゆる規模のチームに選ばれています")],
    );
    let lead = styled_text::text(
        &TextProps {
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![],
        vec![text(
            "業種を問わず、日々の業務でこのプラットフォームが使われています。",
        )],
    );
    let row = div(
        vec![("class", "blocks-logo-cloud-grid-row")],
        dummy_assets::COMPANY_NAMES[..5]
            .iter()
            .map(|name| logo_item(name))
            .collect(),
    );
    div(
        vec![("class", "blocks-logo-cloud-grid-stack")],
        vec![tagline, title, lead, row],
    )
}

/// 形 B: 見出し無し、リード文 → ロゴ 6 個の折り返し行。
fn variant_b() -> Node {
    let lead = styled_text::text(
        &TextProps {
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![],
        vec![text("すでに数千のチームが導入しています")],
    );
    let row = div(
        vec![("class", "blocks-logo-cloud-grid-row")],
        dummy_assets::COMPANY_NAMES
            .iter()
            .map(|name| logo_item(name))
            .collect(),
    );
    div(
        vec![("class", "blocks-logo-cloud-grid-stack")],
        vec![lead, row],
    )
}

/// 形 C: 見出し + link の見出し行 → 社名タグ付きロゴカード 6 枚。
fn variant_c() -> Node {
    let title = styled_heading::heading(
        HeadingLevel::H3,
        &HeadingProps::default(),
        vec![],
        vec![text("導入企業")],
    );
    let more = link::root(
        REPO,
        &LinkProps::default(),
        vec![],
        vec![text("GitHub で見る")],
    );
    let header = div(
        vec![("class", "blocks-logo-cloud-grid-header")],
        vec![title, more],
    );
    let cards = dummy_assets::COMPANY_NAMES
        .iter()
        .map(|name| {
            let name_tag = tag::root(
                &TagProps::default(),
                vec![(TILE_LABEL_ATTR, "")],
                vec![tag::label(vec![], vec![text(*name)])],
            );
            card::root(
                CardProps::default(),
                vec![],
                vec![card::body(vec![], vec![logo(), name_tag])],
            )
        })
        .collect();
    let grid = div(vec![("class", "blocks-logo-cloud-grid-cards")], cards);
    div(
        vec![("class", "blocks-logo-cloud-grid-stack")],
        vec![header, grid],
    )
}

/// 形 C・D のカード/タイル内社名タグ用の折り返しマーカー。両形とも `lg`
/// 6 カラム時にカード/タイル幅が縮むため、`tag` 既定の
/// `white-space: nowrap` を上書きして折り返し可能にする（Bugbot 指摘
/// 「Tile shrink overflows company labels」・codex 指摘「形 C の社名
/// タグが 6 列表示でカードからはみ出す」、PR #3244）。
const TILE_LABEL_ATTR: &str = "data-blocks-logo-cloud-grid-tile-label";

/// 形 D: 見出し無し、淡色枠のロゴタイル 6 枚のグリッド。
fn variant_d() -> Node {
    let tiles = dummy_assets::COMPANY_NAMES
        .iter()
        .map(|name| {
            let name_tag = tag::root(
                &TagProps::default(),
                vec![(TILE_LABEL_ATTR, "")],
                vec![tag::label(vec![], vec![text(*name)])],
            );
            let item = div(
                vec![("class", "blocks-logo-cloud-grid-logo-item")],
                vec![logo(), name_tag],
            );
            div(vec![("class", "blocks-logo-cloud-grid-tile")], vec![item])
        })
        .collect();
    div(vec![("class", "blocks-logo-cloud-grid-tiles")], tiles)
}

/// 形 E: ロゴ 5 個の折り返し行 → 下にピル型の告知リンク。
fn variant_e() -> Node {
    let row = div(
        vec![("class", "blocks-logo-cloud-grid-row")],
        dummy_assets::COMPANY_NAMES[..5]
            .iter()
            .map(|name| logo_item(name))
            .collect(),
    );
    let pill = link::root(
        REPO,
        &LinkProps::default(),
        vec![("data-blocks-logo-cloud-grid-pill", "")],
        vec![text("GitHub で見る")],
    );
    div(
        vec![("class", "blocks-logo-cloud-grid-stack")],
        vec![row, pill],
    )
}

/// `logo-cloud-grid` の Demo 本体。5 形（A〜E）を縦に併記する。呼び出し
/// ごとに同一の `Node` を返す純関数。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-logo-cloud-grid-demo")],
        vec![
            variant_a(),
            variant_b(),
            variant_c(),
            variant_d(),
            variant_e(),
        ],
    )
}
```

## 原案差分メモ

集約元 10 件（対応表 ID 準拠、固有名・ファイル名は記載しません）を 5 形へ圧縮しました。

- 基準形（タグライン → 見出し → リード文 → ロゴ列）に相当する複数件は形 A へ統合しました。
- リード文のみ・見出し無しの複数件は形 B へ統合しました。
- 社名タグ付きロゴカードの 1 件は形 C として個別実装しました。
- 淡色枠のタイル表現の複数件は形 D へ統合しました。
- ピル型の告知リンクを伴う 1 件は形 E として個別実装しました。
- 暗色固定版の 1 件は、本サイトのテーマトグルで代替できるため Demo には併記せず、本メモへの記載のみに留めました。
- 左寄せレイアウトの 1 件は、基準形との差が文字揃え（`text-align`）のみであるため Demo には併記せず、本メモへの記載のみに留めました。
- 文言・配色は独自に書き起こしたもので、実在ブランドの文言・配色は持ち込んでいません。
- `_/blocks-intake/` の参照ファイルは本 worktree に存在せず、上記の対応はイシュー本文の 1 行要約のみに基づく設計仮説です。実物との細部の差分は未検証です。
- `id`・`href="#"`・`<form>` は出力しません。
