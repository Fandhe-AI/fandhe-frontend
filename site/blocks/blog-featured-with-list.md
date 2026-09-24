# blog-featured-with-list

`heading` / `text` / `avatar` / `separator` / `link` / `link-overlay` を
合成した、特集記事 1 件 + 通常記事リスト 2 件を並べる 2 カラムの合成例です。

幅 lg（64rem）以上では左に特集記事、右に罫線区切りの記事リストを並べる
2 カラム表示になり、それより狭い幅では特集記事の下へ記事リストを縦に
積む 1 列表示に切り替わります。文言・数値はすべて架空のもので、データ
取得・送信は行わない静的な表示例です。リンク先はすべてリポジトリへの
固定リンクです。`<form>` は使用しません。

集約元は 1 件のみ（対応表 ID R0777）です。

## Rust コード

```rust
use fandhe_frontend_core::{div, el, footer, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::link_overlay::{self, overlay};
use fandhe_frontend_pre_styled_ui::separator::{separator, SeparatorProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};
use fandhe_frontend_pre_styled_ui::Size;

/// リンク先の固定外部 URL（モジュール doc「`href="#"` を使わない」節参照）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// 記事 1 件分のダミーデータ（架空、実在の人物・企業とは無関係）。
struct Post {
    date_iso: &'static str,
    date_label: &'static str,
    title: &'static str,
    excerpt: &'static str,
    author_name: &'static str,
    author_initials: &'static str,
}

/// 特集記事（架空）。
const FEATURED: Post = Post {
    date_iso: "2026-09-20",
    date_label: "2026年9月20日",
    title: "既定エスケープだけで守れる範囲を広げる",
    excerpt: "テキスト補間を必ずエスケープ経由にする設計判断が、レビューの負荷をどう下げたかをまとめました。",
    author_name: "遠藤 佑奈",
    author_initials: "EY",
};

/// 通常記事リスト（架空、2 件）。
const LIST: [Post; 2] = [
    Post {
        date_iso: "2026-09-12",
        date_label: "2026年9月12日",
        title: "単一バイナリ配布までの最短ルート",
        excerpt: "SSR から単一実行ファイルへ至る構成を、最小手順で振り返ります。",
        author_name: "冨田 千夏",
        author_initials: "TC",
    },
    Post {
        date_iso: "2026-09-05",
        date_label: "2026年9月5日",
        title: "ノード木 API で組み立てる合成例",
        excerpt: "HTML 文字列を直接組み立てず既存部品を合成するときの考え方を紹介します。",
        author_name: "宮下 大和",
        author_initials: "MY",
    },
];

/// `<time datetime>` を組み立てる（モジュール doc「`<time datetime>` と
/// 表示日付の一致」節）。
fn post_date(iso: &str, label: &str) -> Node {
    el(
        "time",
        vec![
            ("class", "blocks-blog-featured-with-list-date"),
            ("datetime", iso),
        ],
        vec![text(label)],
    )
}

/// 著者リンク（アバターのイニシャル fallback + 氏名）。`overlay` の外へ
/// 兄弟として置くことでクリック可能なまま保つ（モジュール doc「記事
/// リンクの入れ子を避ける 2 段構成」節参照）。
fn author(name: &str, initials: &str) -> Node {
    link::root(
        REPO,
        &LinkProps::default(),
        vec![("data-blocks-blog-featured-with-list-author", "")],
        vec![
            avatar::root(
                &AvatarProps {
                    size: Size::Xs,
                    ..AvatarProps::default()
                },
                vec![],
                vec![avatar::fallback(
                    ImageStatus::Error,
                    vec![],
                    vec![text(initials)],
                )],
            ),
            text(name),
        ],
    )
}

/// 通常記事リストの 1 件（日付・見出し・抜粋 + 全面クリック用 `overlay`。
/// 著者は `overlay` の外の兄弟としてリンクを保つ）。
fn list_article(post: &Post) -> Node {
    div(
        vec![("data-blocks-blog-featured-with-list-article", "")],
        vec![
            link_overlay::root(
                vec![],
                vec![
                    post_date(post.date_iso, post.date_label),
                    heading(
                        HeadingLevel::H3,
                        &HeadingProps::default(),
                        vec![],
                        vec![text(post.title)],
                    ),
                    styled_text::text(
                        &TextProps {
                            variant: TextVariant::Muted,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text(post.excerpt)],
                    ),
                    overlay(REPO, vec![("aria-label", post.title)], vec![]),
                ],
            ),
            author(post.author_name, post.author_initials),
        ],
    )
}

/// 特集記事フッタ（「続きを読む」リンク + 著者）。`aria-label` は
/// [`FEATURED`] の見出しから `format!` で導出し、文言の複製によるドリフト
/// を避ける。
fn read_more_footer() -> Node {
    let aria_label = format!("続きを読む: {}", FEATURED.title);
    footer(
        vec![("class", "blocks-blog-featured-with-list-featured-footer")],
        vec![
            link::root(
                REPO,
                &LinkProps::default(),
                vec![("aria-label", aria_label.as_str())],
                vec![
                    text("続きを読む"),
                    el("span", vec![("aria-hidden", "true")], vec![text("→")]),
                ],
            ),
            author(FEATURED.author_name, FEATURED.author_initials),
        ],
    )
}

/// `blog-featured-with-list` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数（モジュール doc「レイアウト」節）。
pub fn demo() -> Node {
    let featured = div(
        vec![("class", "blocks-blog-featured-with-list-featured")],
        vec![
            post_date(FEATURED.date_iso, FEATURED.date_label),
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    weight: HeadingWeight::Bold,
                },
                vec![],
                vec![text(FEATURED.title)],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(FEATURED.excerpt)],
            ),
            read_more_footer(),
        ],
    );

    let list = div(
        vec![("class", "blocks-blog-featured-with-list-list")],
        vec![
            list_article(&LIST[0]),
            separator(
                &SeparatorProps::default(),
                vec![("data-blocks-blog-featured-with-list-separator", "")],
            ),
            list_article(&LIST[1]),
        ],
    );

    div(
        vec![("class", "blocks-blog-featured-with-list-layout")],
        vec![featured, list],
    )
}
```

## 原案差分メモ

参照（対応表 ID R0777。出典の固有名・ファイル名は記載しません）からの
意図的な差分は次のとおりです。

- 見出しを `h2` から `h3` へ変更しました（ページ側が `## Demo` として
  `h2` を出すため）。
- `href="#"` の死リンクをリポジトリへの固定外部 URL へ置き換えました。
- `aria-describedby` や `id` 属性は出力しません（宙に浮いた ARIA 参照・
  id 重複を構造的に避けるため）。
- アバターは画像ではなく、既存部品のイニシャル fallback 表示にしました。
- 文言（記事タイトル・抜粋・著者名）はすべて独自に書き直しました。
- 配色・余白・角丸は独自実装せず、既存のテーマトークンにそのまま従います。
- 参照側にあり得る、日付の機械可読値（`datetime`）と表示値の食い違いは
  持ち込みません（本実装は常に同じ日を指す値の組にしています）。
