# media-object

`fandhe-frontend-pre-styled-ui` の `item` / `image` 部品を合成した、画像
（またはアイコン）と見出し・説明文を横並びにする media object の**整列
パターン集**です。Blocks セクションは新規部品を追加するものではなく、既存
の Themes/Primitives 部品を組み合わせた実例集であることに注意してください
（主参照は対応表 ID R1062、集約元は R1063〜R1069。出典の固有名・ファイル名
は記載しません）。

上揃え（基準形）を軸に、縦中央揃え・下揃え・画像を行高いっぱいに伸ばす・
画像を右側に配置・狭幅で縦積み・狭幅で画像を全幅化・本文内への小さい
media object の入れ子という 7 通りの差分パターンを併記しています。狭幅
パターン（縦積み・画像全幅化）は、ビューポート全体ではなく各パネルの
実測幅で折り返しを切り替えるため `@container` を使い、パネルへ
`max-width: 20rem` を与えて同一ビューポートのまま狭幅後の見た目を再現して
います。

本 Demo は静的な整列パターン集であり、`<form>` 要素・ボタン・リンク・`id`
属性を一切持たず、データの取得・送信・状態管理を行いません。画像はすべて
装飾用のプレースホルダ（`alt=""`）で、見出し・説明文の文言はすべて独自に
書いた架空のものであり、実企業名・実クレデンシャル・PII を含みません。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::image::{self, ImageProps};
use fandhe_frontend_pre_styled_ui::item::{self, ItemMediaVariant, ItemRootProps};

/// media（画像）パーツを組み立てる。`extra_attr` は `variant` 固有の
/// フック用 `data-*` 属性（例: `-media-nested`）。
fn media_image(extra_attr: (&'static str, &'static str)) -> Node {
    item::media(
        ItemMediaVariant::Image,
        vec![extra_attr],
        vec![image::image(
            &ImageProps::new(dummy_assets::PRODUCT_SRC, ""),
            vec![],
        )],
    )
}

/// content（見出し + 説明文）パーツを組み立てる。
fn body(title: &str, description: &str) -> Node {
    body_with_extra(title, description, Vec::new())
}

/// content（見出し + 説明文 + 追加の子ノード）パーツを組み立てる。`nested`
/// インスタンスが入れ子の media object を本文末尾へ追加するために使う
/// （[`body`] はこの追加子ノードが空の特殊形）。
fn body_with_extra(title: &str, description: &str, extra: Vec<Node>) -> Node {
    let mut children = vec![
        item::title(vec![], vec![text(title)]),
        item::description(vec![], vec![text(description)]),
    ];
    children.extend(extra);
    item::content(vec![], children)
}

/// パネル見出し（短い説明、[`el`] で `<p>` として直接組み立てる。見出し
/// 階層は docs ページ H1 → 生成 H2「Demo」の配下のため `h2`/`h3` を増やさず
/// `p` で十分）。
fn caption(text_content: &str) -> Node {
    el(
        "p",
        vec![("class", "blocks-media-object-caption")],
        vec![text(text_content)],
    )
}

/// パネル外枠。`narrow` が `true` のパネルは
/// `data-blocks-media-object-narrow`（[`LAYOUT_CSS`] の `max-width` 規則）を
/// 追加で持つ（`stack`/`stack-full` 用）。
fn panel(variant: &'static str, caption_text: &str, narrow: bool, item_node: Node) -> Node {
    let mut attrs: Vec<(&str, &str)> = vec![
        ("class", "blocks-media-object-panel"),
        ("data-blocks-media-object-variant", variant),
    ];
    if narrow {
        attrs.push(("data-blocks-media-object-narrow", ""));
    }
    div(attrs, vec![caption(caption_text), item_node])
}

/// `root`（item）パーツ。`variant` を `data-blocks-media-object-variant`
/// へ渡し、既定の整列（`top`）以外は [`LAYOUT_CSS`] の対応セレクタが上書く。
fn media_object_root(variant: &'static str, media: Node, content: Node) -> Node {
    item::root(
        ItemRootProps::default(),
        vec![
            ("data-blocks-media-object-root", ""),
            ("data-blocks-media-object-variant", variant),
        ],
        vec![media, content],
    )
}

/// `nested`（R1069）の入れ子部分。外側の `nowrap`/`align-items` 規則を
/// 継承させないため `data-blocks-media-object-root` は付与せず、media のみ
/// `-media-nested` フックを持つ（[`LAYOUT_CSS`] 参照）。
fn nested_media_object() -> Node {
    item::root(
        ItemRootProps::default(),
        vec![],
        vec![
            media_image(("data-blocks-media-object-media-nested", "")),
            body(
                "関連するアップデート",
                "同じ整列パターンを縮小して本文内に再利用した例です。",
            ),
        ],
    )
}

/// `top`（R1062・主参照）インスタンス。
fn top_instance() -> Node {
    panel(
        "top",
        "上揃え（基準形）",
        false,
        media_object_root(
            "top",
            media_image(("data-blocks-media-object-media", "")),
            body(
                "新しいダッシュボードのレイアウト",
                "サイドバーと本文の比率を見直し、狭い画面での折り返しを改善しました。",
            ),
        ),
    )
}

/// `center`（R1063）インスタンス。
fn center_instance() -> Node {
    panel(
        "center",
        "縦中央揃え",
        false,
        media_object_root(
            "center",
            media_image(("data-blocks-media-object-media", "")),
            body(
                "通知設定の見直し",
                "メール通知とアプリ内通知を個別に切り替えられるようにしました。",
            ),
        ),
    )
}

/// `bottom`（R1064）インスタンス。
fn bottom_instance() -> Node {
    panel(
        "bottom",
        "下揃え",
        false,
        media_object_root(
            "bottom",
            media_image(("data-blocks-media-object-media", "")),
            body(
                "エクスポート機能の追加",
                "表示中のデータを CSV 形式でダウンロードできるようになりました。",
            ),
        ),
    )
}

/// `stretch`（R1065）インスタンス。
fn stretch_instance() -> Node {
    panel(
        "stretch",
        "画像を行高いっぱいに伸ばす",
        false,
        media_object_root(
            "stretch",
            media_image(("data-blocks-media-object-media", "")),
            body(
                "検索結果の並べ替え",
                "更新日時・関連度・名前の 3 種類から並べ替え基準を選べます。",
            ),
        ),
    )
}

/// `right`（R1066）インスタンス。
fn right_instance() -> Node {
    panel(
        "right",
        "画像を右側に配置",
        false,
        media_object_root(
            "right",
            media_image(("data-blocks-media-object-media", "")),
            body(
                "キーボードショートカットの追加",
                "よく使う操作をキーボードだけで完結できるようにしました。",
            ),
        ),
    )
}

/// `stack`（R1067）インスタンス。狭幅パネル（`@container`）で縦積みに
/// 折り返す。
fn stack_instance() -> Node {
    panel(
        "stack",
        "狭幅で縦積み",
        true,
        media_object_root(
            "stack",
            media_image(("data-blocks-media-object-media", "")),
            body(
                "タグ機能のプレビュー",
                "アイテムに自由なタグを付けて絞り込めるようになります。",
            ),
        ),
    )
}

/// `stack-full`（R1068）インスタンス。狭幅パネルで縦積み + 画像を全幅化。
fn stack_full_instance() -> Node {
    panel(
        "stack-full",
        "狭幅で画像を全幅化",
        true,
        media_object_root(
            "stack-full",
            media_image(("data-blocks-media-object-media", "")),
            body(
                "共有リンクの有効期限",
                "共有リンクに有効期限を設定し、期限切れ後は自動で無効になります。",
            ),
        ),
    )
}

/// `nested`（R1069）インスタンス。本文内に小さい media object を入れ子。
fn nested_instance() -> Node {
    panel(
        "nested",
        "本文内に小さい media object を入れ子",
        false,
        media_object_root(
            "nested",
            media_image(("data-blocks-media-object-media", "")),
            body_with_extra(
                "リリースノート 2026.9",
                "今回のリリースでは以下の変更を行いました。",
                vec![nested_media_object()],
            ),
        ),
    )
}

/// `media-object` の Demo 本体。呼び出しごとに同一の `Node` を返す純関数。
/// 主参照（R1062・top）を先頭に、8 インスタンスを縦に並べる。
#[must_use]
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-media-object-layout")],
        vec![
            top_instance(),
            center_instance(),
            bottom_instance(),
            stretch_instance(),
            right_instance(),
            stack_instance(),
            stack_full_instance(),
            nested_instance(),
        ],
    )
}
```

## 原案差分メモ

- 主参照は R1062（画像 + 見出し + 説明文の横並び、上揃え基準形）です。
  集約元 7 件（R1063 縦中央揃え・R1064 下揃え・R1065 画像を行高い
  っぱいに伸ばす・R1066 画像を右側に配置・R1067 狭幅で縦積み・R1068 狭幅
  で画像を全幅化・R1069 本文内に小さい media object を入れ子）を、
  それぞれ `data-blocks-media-object-variant` で区別した別パネルとして
  併記しています。
- 狭幅パターン（`stack`/`stack-full`）はビューポート幅ではなくパネル自体
  の実測幅（`@container`）で折り返しを切り替えます。パネルへ
  `max-width: 20rem` を与えることで、同一ビューポート・無 JS のまま狭幅後
  の見た目を静的に表現しています。
- `nested` パターンは本文（`item::content`）の末尾へ、より小さい media
  object（media `2.5rem` 角）をそのまま入れ子にしています。入れ子側の
  `item::root` には外側専用の `nowrap`/`align-items` 上書きフック
  （`data-blocks-media-object-root`）を付与せず、既定の折り返しレイアウト
  を継承させています。
- 画像はすべて `dummy_assets::PRODUCT_SRC`（モノトーン抽象図形の SVG）を
  使う装飾画像として `alt=""` を付与し、意味を持つ画像として偽装していま
  せん。見出し・説明文はすべて独自に書いた架空データです。
- 参照元（`_/blocks-intake/`）は本イシュー着手時点で worktree に存在せず、
  対応表 ID とイシュー記載の差分説明のみを根拠に実装しました
  （`page-heading-avatar` と同じ扱い）。
- ブラウザでの実機確認（狭幅パネルでの折り返し・ライト/ダーク両テーマ）は
  サンドボックス制約により未実施です。cargo test による出力検証のみで
  代替しました。
