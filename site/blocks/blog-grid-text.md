# blog-grid-text

`heading` / `text` / `badge` / `card` / `avatar` / `link` / `link-overlay` を
合成した、画像を持たない記事カードのグリッドです。上部にセクション見出しと
説明文、その下に等幅の記事カードグリッドを配置します。列数は狭い幅で 1 列、
md（48rem）以上で 2 列、lg（64rem）以上で 3 列に切り替わります。

各カードはメタ行（日付 + カテゴリの badge）→ タイトル → 抜粋（3 行で
省略表示）→ 著者（アバター + 氏名 + 役職）の構成で、カード全面が記事への
リンクになっています。著者行はカード全面リンクとは独立したリンクとして
クリックできます。

見出しの揃え（左寄せ/中央）と上罫線の有無という 2 つのバリエーションを、
キャプション付きの 2 インスタンス併記で示しています。文言・数値・人名は
すべて架空のもので、データ取得・送信は行わない静的な表示例です。リンク先は
すべてリポジトリへの固定リンクです。`<form>` は使用しません。著者名・役職は
コード内のリテラルとして直接持ち、アバターはイニシャル表示（fallback）です。

主参照は対応表 ID R0772 で、R0016 / R0417 / R0775 を構造として集約
しています。

## Rust コード

```rust
use fandhe_frontend_core::{div, el, p, section, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::link_overlay::{self, overlay};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};
use fandhe_frontend_pre_styled_ui::Size;

/// リンク先の固定外部 URL（モジュール doc「`href="#"` を使わない」節参照）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// 記事 1 件分のダミーデータ（架空、実在の人物・企業とは無関係）。著者
/// 情報は `crate::blocks::dummy_assets`（`pub(crate)`）を参照せず、
/// Markdown 原稿のコードフェンスが単体でコンパイルできるようリテラルで
/// 直接持つ（モジュール doc「`crate::blocks::dummy_assets` を使わない
/// 理由」節参照）。
struct Post {
    date_iso: &'static str,
    date_label: &'static str,
    category: &'static str,
    title: &'static str,
    excerpt: &'static str,
    author_name: &'static str,
    author_role: &'static str,
    author_initials: &'static str,
}

/// インスタンス A（左寄せ見出し・上罫線あり）用の記事 3 件（架空）。
const INSTANCE_A: [Post; 3] = [
    Post {
        date_iso: "2026-09-18",
        date_label: "2026年9月18日",
        category: "アーキテクチャ",
        title: "ノード木 API だけで組み立てる合成例の作り方",
        excerpt: "HTML 文字列を直接組み立てず、既存部品を合成するときに気を付けている判断軸を振り返ります。",
        author_name: "高橋 美咲",
        author_role: "フロントエンドエンジニア",
        author_initials: "MT",
    },
    Post {
        date_iso: "2026-09-11",
        date_label: "2026年9月11日",
        category: "セキュリティ",
        title: "既定エスケープだけで守れる範囲を広げる",
        excerpt: "テキスト補間を必ずエスケープ経由にする設計判断が、レビューの負荷をどう下げたかをまとめました。",
        author_name: "中村 悠斗",
        author_role: "セキュリティエンジニア",
        author_initials: "YN",
    },
    Post {
        date_iso: "2026-09-04",
        date_label: "2026年9月4日",
        category: "配布",
        title: "単一バイナリ配布までの最短ルート",
        excerpt: "SSR から単一実行ファイルへ至る構成を、最小手順で振り返ります。",
        author_name: "小林 彩花",
        author_role: "SRE",
        author_initials: "AK",
    },
];

/// インスタンス B（中央揃え見出し・上罫線なし）用の記事 3 件（架空、
/// インスタンス A とは別内容にして区別しやすくする）。
const INSTANCE_B: [Post; 3] = [
    Post {
        date_iso: "2026-08-28",
        date_label: "2026年8月28日",
        category: "テスト",
        title: "XSS 回帰テストを削除せずに保つための工夫",
        excerpt: "SSR/SSG/CSR/WASM の各経路で回帰テストを弱体化させない運用について書きました。",
        author_name: "山本 拓海",
        author_role: "QA エンジニア",
        author_initials: "TY",
    },
    Post {
        date_iso: "2026-08-21",
        date_label: "2026年8月21日",
        category: "CI",
        title: "壁時計時間を優先した CI 並列化の考え方",
        excerpt: "資源の無駄を許容してでも所要時間を優先するときの判断基準を整理します。",
        author_name: "渡辺 さくら",
        author_role: "CI/CD エンジニア",
        author_initials: "SW",
    },
    Post {
        date_iso: "2026-08-14",
        date_label: "2026年8月14日",
        category: "設計",
        title: "依存グラフの上限を機械で守る",
        excerpt: "60 件・深さ 6 という上限を、レビューではなく機械検証で保つ仕組みを紹介します。",
        author_name: "佐々木 陸",
        author_role: "アーキテクト",
        author_initials: "RS",
    },
];

/// `<time datetime>` を組み立てる（モジュール doc「`<time datetime>` と
/// 表示日付の一致」節）。
fn post_date(iso: &str, label: &str) -> Node {
    el(
        "time",
        vec![("class", "blocks-blog-grid-text-date"), ("datetime", iso)],
        vec![text(label)],
    )
}

/// メタ行（日付 + カテゴリ badge）を組み立てる。
fn post_meta(post: &Post) -> Node {
    div(
        vec![("class", "blocks-blog-grid-text-meta")],
        vec![
            post_date(post.date_iso, post.date_label),
            badge::badge(
                &BadgeProps {
                    size: Size::Sm,
                    ..BadgeProps::default()
                },
                vec![("data-blocks-blog-grid-text-badge", "")],
                vec![text(post.category)],
            ),
        ],
    )
}

/// 著者リンク（アバターのイニシャル fallback + 氏名・役職）。`overlay`
/// の外へ兄弟として置くことでクリック可能なまま保つ（モジュール doc
/// 「カード全面リンクと著者リンクを両立する 2 段構成」節参照）。画像
/// アセットへは依存しない（モジュール doc「`crate::blocks::dummy_assets`
/// を使わない理由」節参照）。
fn author(name: &str, role: &str, initials: &str) -> Node {
    link::root(
        REPO,
        &LinkProps::default(),
        vec![("data-blocks-blog-grid-text-author", "")],
        vec![
            avatar::root(
                &AvatarProps {
                    size: Size::Sm,
                    ..AvatarProps::default()
                },
                vec![],
                vec![avatar::fallback(
                    ImageStatus::Error,
                    vec![],
                    vec![text(initials)],
                )],
            ),
            div(
                vec![("class", "blocks-blog-grid-text-author-info")],
                vec![
                    div(
                        vec![("class", "blocks-blog-grid-text-author-name")],
                        vec![text(name)],
                    ),
                    div(
                        vec![("class", "blocks-blog-grid-text-author-role")],
                        vec![text(role)],
                    ),
                ],
            ),
        ],
    )
}

/// 記事カード 1 件（メタ行・タイトル・抜粋・全面 overlay + 独立した著者
/// リンク）を組み立てる。
fn article_card(post: &Post) -> Node {
    card::root(
        CardProps::default(),
        vec![("data-blocks-blog-grid-text-card", "")],
        vec![card::body(
            vec![("data-blocks-blog-grid-text-body", "")],
            vec![
                link_overlay::root(
                    vec![("data-blocks-blog-grid-text-article", "")],
                    vec![
                        post_meta(post),
                        heading(
                            HeadingLevel::H4,
                            &HeadingProps::default(),
                            vec![],
                            vec![text(post.title)],
                        ),
                        styled_text::text(
                            &TextProps {
                                variant: TextVariant::Muted,
                                ..TextProps::default()
                            },
                            vec![("data-blocks-blog-grid-text-excerpt", "")],
                            vec![text(post.excerpt)],
                        ),
                        overlay(REPO, vec![("aria-label", post.title)], vec![]),
                    ],
                ),
                author(post.author_name, post.author_role, post.author_initials),
            ],
        )],
    )
}

/// セクション見出し・説明・「すべての記事を見る」リンクからなるヘッダを
/// 組み立てる。`align`/`rule` がそれぞれ「見出しの揃え」「上罫線の有無」
/// バリエーションを表す（モジュール doc「静的な 2 インスタンス併記」
/// 節参照）。
fn section_header(heading_text: &str, description: &str, align: &str, rule: bool) -> Node {
    let mut attrs: Vec<(&str, &str)> = vec![
        ("class", "blocks-blog-grid-text-header"),
        ("data-blocks-blog-grid-text-align", align),
    ];
    if rule {
        attrs.push(("data-blocks-blog-grid-text-rule", ""));
    }
    div(
        attrs,
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    ..HeadingProps::default()
                },
                vec![],
                vec![text(heading_text)],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(description)],
            ),
            link::root(
                REPO,
                &LinkProps::default(),
                vec![],
                vec![text("すべての記事を見る")],
            ),
        ],
    )
}

/// インスタンス 1 件（ヘッダ + 記事グリッド）を組み立てる。
fn section_instance(
    heading_text: &str,
    description: &str,
    align: &str,
    rule: bool,
    posts: &[Post],
) -> Node {
    let cards: Vec<Node> = posts.iter().map(article_card).collect();
    section(
        vec![],
        vec![
            section_header(heading_text, description, align, rule),
            div(vec![("class", "blocks-blog-grid-text-grid")], cards),
        ],
    )
}

/// `blog-grid-text` の Demo 本体。呼び出しごとに同一の `Node` を返す純関数
/// （モジュール doc「静的な 2 インスタンス併記」節）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-blog-grid-text-stack")],
        vec![
            p(
                vec![("class", "blocks-blog-grid-text-caption")],
                vec![text("左寄せ見出し・上罫線あり")],
            ),
            section_instance(
                "最近の記事",
                "チームが公開した記事から、まだ読んでいないものを見つけてください。",
                "start",
                true,
                &INSTANCE_A,
            ),
            p(
                vec![("class", "blocks-blog-grid-text-caption")],
                vec![text("中央揃え見出し・上罫線なし")],
            ),
            section_instance(
                "ブログ",
                "運用・設計・テストにまつわる記事を集めました。",
                "center",
                false,
                &INSTANCE_B,
            ),
        ],
    )
}
```

**原案差分メモ**

参照（対応表 ID R0772 が主参照、R0016 / R0417 / R0775 を構造として集約。
出典の固有名・ファイル名は記載しません）からの意図的な差分は次のとおりです。

- R0775 の 1 列レイアウトは、第 3 インスタンスとして個別に再現せず、狭い
  幅でグリッドが 1 列へ折り返すレスポンシブ挙動として集約しました。
- R0016 の簡素な導入は、中央揃え・上罫線なしのインスタンスとして表現
  しました。
- R0417 の著者アバターは、`crate::blocks::dummy_assets`（`pub(crate)`）に
  依存すると Markdown 原稿のコードフェンスが単体でコンパイルできなくなる
  ため、イニシャル表示（fallback）と氏名・役職の 2 行で表現しました。
- 見出しレベルは `h2` から `h3`（セクション見出し）・`h4`（記事タイトル）
  へ下げました（ページ側が `## Demo` として `h2` を出すため）。
- `href="#"` の死リンクは、すべてリポジトリへの固定外部 URL に置き換え
  ました。
- `id` や `aria-describedby` は出力しません（宙に浮いた ARIA 参照・id 重複
  を構造的に避けるため）。
- 抜粋は CSS の `line-clamp` で 3 行に省略しています。
- 文言（記事タイトル・抜粋・見出し・説明文）はすべて独自に書き直しました。
- 配色・余白・角丸は独自実装せず、既存のテーマトークンにそのまま従います。
- 参照側にあり得る、日付の機械可読値（`datetime`）と表示値の食い違いは
  持ち込みません（本実装は常に同じ日を指す値の組にしています）。
