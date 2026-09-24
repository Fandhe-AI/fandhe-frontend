//! 部品横断で使う最小 SVG ラインアートアイコンセット（イシュー #2606）。
//!
//! `fandhe-frontend-core` の [`fandhe_frontend_core::el`] のみでノード木を
//! 組み立てる。`format!` によるマークアップ文字列組み立て・`raw_html` は
//! 一切使わない（`docs/design/wireframe-ui-architecture.md` §1 の全部品
//! 共通不変条件、`coding-rust.md` の HTML 文字列直接組み立て禁止）。
//!
//! # 出力契約
//!
//! 各アイコン関数は次の固定属性を持つ `<svg>` を返す（値はすべて
//! `&'static str` リテラルで利用者入力を含まない）:
//!
//! | 属性 | 値 | 根拠 |
//! |---|---|---|
//! | `class` | `fw-wire-icon-glyph fw-wire-size-<段階>` | §10.1 命名規約 |
//! | `data-icon` | アイコン名（[`ALL`] の名前と同一） | showcase・golden テストでの識別子（表示状態ではない） |
//! | `viewBox` | `0 0 24 24` | 24 グリッド固定 |
//! | `width` / `height` | `1em` | CSS 未読込時のフォールバック（`<svg>` 既定 300×150 の回避） |
//! | `fill` | `none` | 線画契約（塗りを持たない） |
//! | `stroke` | `currentColor` | モノクロ・祖先文字色への追従 |
//! | `stroke-width` | `1.5` | トークンの `line-width: 1.5px`（`crate::tokens`）と同値 |
//! | `stroke-linecap` / `stroke-linejoin` | `round` | 線画の統一 |
//! | `aria-hidden` | `true` | 装飾用途。対話的 ARIA（`role`/`aria-expanded` 等）は付与しない |
//! | `focusable` | `false` | 非インタラクティブ（`docs/design/wireframe-ui-architecture.md` §7） |
//!
//! `href`/`xlink:href`/`on*`/`style` は一切出力しない。子要素は
//! `path`/`circle`/`line`/`polyline`/`polygon`/`rect` のみで、属性は
//! いずれも固定リテラル（`d`/`cx`/`cy`/`r`/`x1`/`y1`/`x2`/`y2`/`points`/
//! `x`/`y`/`width`/`height`/`rx`）。子要素にも `fill` は付けない
//! （ルートの `fill="none"` を継承する）。
//!
//! # サイズ機構
//!
//! [`ICON_GLYPH_CSS`] が `.fw-wire-icon-glyph` で `width`/`height: 1em` と
//! `font-size: var(--fw-wire-font-size, 1rem)` を宣言する。実際の寸法は
//! `<svg>` 自身に付与した `fw-wire-size-<段階>`（[`crate::size::css`] が
//! 定義する `--fw-wire-font-size`）が同一要素上で決める。`size::SCALE` の
//! 値そのものはここへ書き写さない（`docs/design/wireframe-ui-architecture.md`
//! §10「値を書き写さない」）。
//!
//! # ジオメトリの出自
//!
//! 全アイコンは 24×24 グリッド上の単純図形として独自に描く。blocks.pm の
//! 外観を書き写さないこと（`docs/design/wireframe-ui-architecture.md` §2）
//! に加え、Lucide / Feather / Heroicons 等の既存アイコンセットのパス
//! データもコピーしない（帰属表示付きライセンスであり、本リポジトリに
//! その受け入れ方針の記録がないため。`docs/policy/intentional-non-adoption.md`
//! 系の先例と同じ fail-closed 判断）。
//!
//! # `Node` スロット規約の例
//!
//! アイコン差し替え（Figma の instance swap 相当）を受ける部品は、
//! `Option<Node>` のスロット引数として本モジュールの関数の戻り値を
//! そのまま受け取る設計を標準とする（`leading`/`trailing` 等）。ホスト
//! 要素は `div`/`span` のような非対話要素を使う（`button` 等の対話要素は
//! 出力しない、同文書 §7）。
//!
//! ```
//! use fandhe_frontend_core::{div, render, text, Node};
//! use fandhe_frontend_wireframe_ui::{icon, Size};
//!
//! // アイコン差し替えスロットを取る最小の部品もどき。
//! fn labeled_row(leading: Option<Node>, label: &str) -> Node {
//!     let mut children: Vec<Node> = Vec::new();
//!     if let Some(icon) = leading {
//!         children.push(icon);
//!     }
//!     children.push(text(label));
//!     div(vec![("class", "fw-wire-row")], children)
//! }
//!
//! // 呼び出し側は `icon::<name>(size)` を `Some` で渡すだけでよい。
//! let with_icon = labeled_row(Some(icon::search(Size::Md)), "<script>x</script>");
//! assert!(render(&with_icon).contains("<svg"));
//! assert!(render(&with_icon).contains("&lt;script&gt;"));
//!
//! // `None` を渡せばアイコンなしで描画される（テキストのエスケープは不変）。
//! let without_icon = labeled_row(None, "plain");
//! assert!(!render(&without_icon).contains("<svg"));
//! ```
//!
//! # #2652 との継ぎ目（解決済み）
//!
//! イシュー #2652（`icon` 部品、Phase 7「Data display」の 7 番目の部品）は
//! 本モジュールへ [`icon`] 関数・[`ICON_CSS`] を追加した。確定した設計
//! 判断は次のとおり。
//!
//! - **シグネチャ**: [`icon`] は `Node` ではなく `fn(Size) -> Node`
//!   （[`IconEntry`] の要素型と同じ関数ポインタ）を受け取る。各グリフは
//!   自身の `<svg>` に `fw-wire-size-<段階>` を持ち、その要素上で
//!   `--fw-wire-font-size` を決めるため、`icon(glyph: Node, size)` 形式で
//!   `icon(icon::search(Size::Sm), Size::Lg)` のようにサイズを 2 か所で
//!   指定できてしまうと黙って `Sm` のまま描画される誤用を招く。部品側が
//!   `glyph(size)` を 1 回だけ呼ぶことで、サイズ指定を 1 か所に固定する
//!   （`docs/design/wireframe-ui-architecture.md` §11.6 の想定どおり）。
//! - **`Option<Node>` スロット規約（§11.4）との違い**: §11.4 は「本文の
//!   横にアイコンを置くホスト部品」（`link`/`tag`/`alert` 等）向けの規約
//!   であり、差し替え対象は任意の `Node` である。対して `icon` 部品に
//!   とっての差し替え対象は「どのグリフか」であり、[`IconEntry`] の
//!   関数ポインタ型がその型付き表現になる（[`ALL`] の一覧表示が
//!   `icon(*ctor, size)` とそのまま書ける）。任意の `Node`（例:
//!   `crate::avatar` が返す画像枠）はこのスロットに渡せないが、`icon`
//!   部品の責務（グリフ差し替え）の外として許容する。
//! - **`role`/`aria-label` は付けない**: 全部品は非インタラクティブな
//!   表示専用プレースホルダーという不変条件（`docs/design/wireframe-ui-architecture.md`
//!   §1/§7）に従い、ルートは `role`/`aria-*`/`tabindex`/`style`/`data-*`
//!   を一切持たない `<span>` とする。アクセシビリティラベルを持たせる
//!   ための `label: &str` 引数も追加しない（テキスト引数を持たない層に
//!   アクセシビリティ責務を持ち込まない判断）。実際にラベル付きで
//!   アイコンを使いたい利用者には Themes の Icon 相当を案内する
//!   （`site/wireframes/icon.md` 参照）。

use fandhe_frontend_core::{el, el_owned, Node};

use crate::class::class_list;
use crate::size::Size;

/// グリフの基盤 class（部品ルートなしで単独使用する唯一の例外的パート
/// class）に対する CSS。`.fw-wire-` で始まるトップレベルセレクタ 1 行のみ
/// を持ち、`--fw-wire-font-size`（[`crate::size::css`]）を `var()` で参照
/// するだけで自身は値を書き写さない。[`crate::css::PARTS`] へ登録される。
pub const ICON_GLYPH_CSS: &str = ".fw-wire-icon-glyph { display: inline-block; width: 1em; height: 1em; font-size: var(--fw-wire-font-size, 1rem); vertical-align: -0.125em; flex-shrink: 0; }\n";

/// [`ALL`] の要素型。アイコン名とコンストラクタ関数ポインタの組。
pub type IconEntry = (&'static str, fn(Size) -> Node);

/// 全アイコンのレジストリ（名前 → コンストラクタ、宣言順）。
///
/// #2607（docs サイト showcase）・#2652（`icon` 部品）の一覧表示、および
/// 本クレートの契約テスト（`tests/icon.rs`）が全アイコン × 全 [`Size`] を
/// 走査するために使う。新規アイコンはここへの登録を追記契約とする。
pub const ALL: &[IconEntry] = &[
    ("plus", plus),
    ("minus", minus),
    ("search", search),
    ("cog", cog),
    ("house", house),
    ("ellipsis", ellipsis),
    ("caret-up", caret_up),
    ("caret-down", caret_down),
    ("caret-left", caret_left),
    ("caret-right", caret_right),
    ("check", check),
    ("x", x),
    ("play", play),
    ("external", external),
    ("at", at),
    ("star", star),
    ("user", user),
    ("image", image),
    ("calendar", calendar),
    ("menu", menu),
    ("bell", bell),
    ("cursor-arrow", cursor_arrow),
    ("cursor-hand", cursor_hand),
];

/// [`el`] の所有属性値版で `<line>` 子要素を組み立てる内部ヘルパ。
fn line(x1: &'static str, y1: &'static str, x2: &'static str, y2: &'static str) -> Node {
    el(
        "line",
        vec![("x1", x1), ("y1", y1), ("x2", x2), ("y2", y2)],
        vec![],
    )
}

/// `<circle>` 子要素を組み立てる内部ヘルパ。
fn circle(cx: &'static str, cy: &'static str, r: &'static str) -> Node {
    el("circle", vec![("cx", cx), ("cy", cy), ("r", r)], vec![])
}

/// `<polyline>` 子要素を組み立てる内部ヘルパ。
fn polyline(points: &'static str) -> Node {
    el("polyline", vec![("points", points)], vec![])
}

/// `<polygon>` 子要素を組み立てる内部ヘルパ。
fn polygon(points: &'static str) -> Node {
    el("polygon", vec![("points", points)], vec![])
}

/// `<path>` 子要素を組み立てる内部ヘルパ。
fn path(d: &'static str) -> Node {
    el("path", vec![("d", d)], vec![])
}

/// `<rect>` 子要素を組み立てる内部ヘルパ。
fn rect(
    x: &'static str,
    y: &'static str,
    width: &'static str,
    height: &'static str,
    rx: &'static str,
) -> Node {
    el(
        "rect",
        vec![
            ("x", x),
            ("y", y),
            ("width", width),
            ("height", height),
            ("rx", rx),
        ],
        vec![],
    )
}

/// §2.3 の `<svg>` ルート出力契約を組み立てる非公開ヘルパ。
///
/// 公開すると利用者が任意の子ノードを線画契約外で流し込めてしまうため
/// `pub` にしない（本モジュール内の各アイコン関数のみが呼ぶ）。
fn glyph(name: &'static str, size: Size, children: Vec<Node>) -> Node {
    el_owned(
        "svg",
        vec![
            (
                "class".to_string(),
                class_list("fw-wire-icon-glyph", &[Some(size.class())]),
            ),
            ("data-icon".to_string(), name.to_string()),
            ("viewBox".to_string(), "0 0 24 24".to_string()),
            ("width".to_string(), "1em".to_string()),
            ("height".to_string(), "1em".to_string()),
            ("fill".to_string(), "none".to_string()),
            ("stroke".to_string(), "currentColor".to_string()),
            ("stroke-width".to_string(), "1.5".to_string()),
            ("stroke-linecap".to_string(), "round".to_string()),
            ("stroke-linejoin".to_string(), "round".to_string()),
            ("aria-hidden".to_string(), "true".to_string()),
            ("focusable".to_string(), "false".to_string()),
        ],
        children,
    )
}

/// 追加（プラス）アイコン。
#[must_use]
pub fn plus(size: Size) -> Node {
    glyph(
        "plus",
        size,
        vec![line("12", "5", "12", "19"), line("5", "12", "19", "12")],
    )
}

/// 削除・折りたたみ（マイナス）アイコン。
#[must_use]
pub fn minus(size: Size) -> Node {
    glyph("minus", size, vec![line("5", "12", "19", "12")])
}

/// 検索（虫眼鏡）アイコン。
#[must_use]
pub fn search(size: Size) -> Node {
    glyph(
        "search",
        size,
        vec![circle("10.5", "10.5", "6"), line("15", "15", "20", "20")],
    )
}

/// 設定（歯車）アイコン。
#[must_use]
pub fn cog(size: Size) -> Node {
    glyph(
        "cog",
        size,
        vec![
            circle("12", "12", "3.5"),
            path(
                "M12 3v3M12 18v3M21 12h-3M6 12H3M18.4 5.6l-2.1 2.1M7.7 16.3l-2.1 2.1M18.4 18.4l-2.1-2.1M7.7 7.7L5.6 5.6",
            ),
        ],
    )
}

/// ホーム（家）アイコン。
#[must_use]
pub fn house(size: Size) -> Node {
    glyph(
        "house",
        size,
        vec![
            polyline("4,11 12,4 20,11"),
            path("M6 10V20H18V10"),
            path("M10 20V14H14V20"),
        ],
    )
}

/// その他（横 3 点）アイコン。
#[must_use]
pub fn ellipsis(size: Size) -> Node {
    glyph(
        "ellipsis",
        size,
        vec![
            circle("6", "12", "1.25"),
            circle("12", "12", "1.25"),
            circle("18", "12", "1.25"),
        ],
    )
}

/// 上向きキャレット（開閉・並び替え等に使う）。
#[must_use]
pub fn caret_up(size: Size) -> Node {
    glyph("caret-up", size, vec![polyline("6,15 12,9 18,15")])
}

/// 下向きキャレット。
#[must_use]
pub fn caret_down(size: Size) -> Node {
    glyph("caret-down", size, vec![polyline("6,9 12,15 18,9")])
}

/// 左向きキャレット。
#[must_use]
pub fn caret_left(size: Size) -> Node {
    glyph("caret-left", size, vec![polyline("15,6 9,12 15,18")])
}

/// 右向きキャレット。
#[must_use]
pub fn caret_right(size: Size) -> Node {
    glyph("caret-right", size, vec![polyline("9,6 15,12 9,18")])
}

/// 完了（チェック）アイコン。
#[must_use]
pub fn check(size: Size) -> Node {
    glyph("check", size, vec![polyline("5,12.5 10,17.5 19,7.5")])
}

/// 閉じる（バツ）アイコン。
#[must_use]
pub fn x(size: Size) -> Node {
    glyph(
        "x",
        size,
        vec![line("6", "6", "18", "18"), line("18", "6", "6", "18")],
    )
}

/// 再生（三角）アイコン。
#[must_use]
pub fn play(size: Size) -> Node {
    glyph("play", size, vec![polygon("8,5 19,12 8,19")])
}

/// 外部リンクアイコン。
#[must_use]
pub fn external(size: Size) -> Node {
    glyph(
        "external",
        size,
        vec![
            path("M14 4h6v6"),
            line("20", "4", "11", "13"),
            path("M18 13v6H5V6h6"),
        ],
    )
}

/// メール（アットマーク）アイコン。
#[must_use]
pub fn at(size: Size) -> Node {
    glyph(
        "at",
        size,
        vec![
            circle("12", "12", "4"),
            path("M16 12v1.5a2.5 2.5 0 0 0 5 0V12a9 9 0 1 0-3.5 7.1"),
        ],
    )
}

/// お気に入り（星）アイコン。
#[must_use]
pub fn star(size: Size) -> Node {
    glyph(
        "star",
        size,
        vec![polygon(
            "12,3.5 14.4,9.5 20.8,9.9 15.8,13.9 17.5,20.1 12,16.5 6.5,20.1 8.2,13.9 3.2,9.9 9.6,9.5",
        )],
    )
}

/// 単一ユーザーアイコン。
#[must_use]
pub fn user(size: Size) -> Node {
    glyph(
        "user",
        size,
        vec![circle("12", "8", "4"), path("M4 20a8 8 0 0 1 16 0")],
    )
}

/// 画像プレースホルダーアイコン。
#[must_use]
pub fn image(size: Size) -> Node {
    glyph(
        "image",
        size,
        vec![
            rect("4", "5", "16", "14", "1.5"),
            circle("9", "10", "1.5"),
            polyline("4,17 10,12 14,15 17,13 20,16"),
        ],
    )
}

/// カレンダーアイコン。
#[must_use]
pub fn calendar(size: Size) -> Node {
    glyph(
        "calendar",
        size,
        vec![
            rect("4", "6", "16", "14", "1.5"),
            line("4", "10", "20", "10"),
            line("8", "4", "8", "8"),
            line("16", "4", "16", "8"),
        ],
    )
}

/// ハンバーガーメニューアイコン。
#[must_use]
pub fn menu(size: Size) -> Node {
    glyph(
        "menu",
        size,
        vec![
            line("4", "7", "20", "7"),
            line("4", "12", "20", "12"),
            line("4", "17", "20", "17"),
        ],
    )
}

/// 通知（ベル）アイコン。
#[must_use]
pub fn bell(size: Size) -> Node {
    glyph(
        "bell",
        size,
        vec![
            path("M6 16V11a6 6 0 0 1 12 0v5l1.5 2h-15z"),
            path("M10 20a2 2 0 0 0 4 0"),
        ],
    )
}

/// マウスカーソル（矢印）アイコン。イシュー #2642 で [`crate::cursor`] のため
/// 追加。24×24 グリッド上で独自に描いた閉じた輪郭（`path` 1 個、単一 `Z`
/// 閉路）で、Lucide/Feather/Heroicons 等の既存アイコンセットのパスデータは
/// コピーしていない（本モジュール doc「ジオメトリの出自」節）。
#[must_use]
pub fn cursor_arrow(size: Size) -> Node {
    glyph(
        "cursor-arrow",
        size,
        vec![path(
            "M5,3 L5,19 L9.2,15.3 L12,21.5 L15,20 L12.2,13.8 L18,13.8 Z",
        )],
    )
}

/// マウスカーソル（手のひら）アイコン。イシュー #2642 で [`crate::cursor`]
/// のため追加。外形は 1 本の閉じた `path`（丸みを持つグローブ状の輪郭）で
/// 描き、指の区切りは内部の `line` 3 本で表現する（`docs/design/wireframe-ui-architecture.md`
/// §11.7 の追記どおり、外形と区切り線を分離する構成）。
#[must_use]
pub fn cursor_hand(size: Size) -> Node {
    glyph(
        "cursor-hand",
        size,
        vec![
            path("M7,21 L7,10 Q7,4 12,4 Q17,4 17,10 L17,21 Z"),
            line("9.5", "6", "9.5", "11"),
            line("12", "4.5", "12", "11"),
            line("14.5", "6", "14.5", "11"),
        ],
    )
}

/// `icon` 部品ルート CSS（`.fw-wire-icon` 1 セレクタ）。[`ICON_GLYPH_CSS`]
/// はグリフ自身の寸法を担うためここでは複製しない（本モジュール doc
/// 「サイズ機構」節）。色リテラル（`#`・`rgb(`）は持たず、
/// `--fw-wire-ink`（`crate::tokens`）を `var()` で参照する。
/// [`crate::css::PARTS`] へ登録される。
pub const ICON_CSS: &str = ".fw-wire-icon { display: inline-flex; align-items: center; justify-content: center; line-height: 1; vertical-align: middle; color: var(--fw-wire-ink); }\n";

/// アイコン単体を示すワイヤーフレーム部品（イシュー #2652、Phase 7
/// 「Data display」の 7 番目の部品）。
///
/// `fandhe-frontend-docs-site` の `wireframes::icon` showcase
/// （`/wireframes/icon/`）から呼ばれ、[`ALL`] 全種の一覧表示元を兼ねる
/// （`docs/design/wireframe-ui-architecture.md` §12 D8）。
///
/// - `glyph`: 表示するアイコンのコンストラクタ（[`IconEntry`] の要素型と
///   同じ `fn(Size) -> Node`）。[`ALL`] の要素や個別関数（例: [`search`]）
///   をそのまま渡せる。
/// - `size`: [`Size`] 5 段。`glyph` へそのまま渡され、部品ルート class
///   `fw-wire-size-<段階>` としても付与する（サイズ指定を 1 か所に固定
///   する設計判断は本モジュール doc「#2652 との継ぎ目」節を参照）。
///
/// ルートは `<span class="fw-wire-icon fw-wire-size-<段階>">` のみで、
/// `role`/`aria-*`/`tabindex`/`style`/`data-*`（部品側）は一切付与しない
/// （非インタラクティブな表示専用プレースホルダーという不変条件、
/// `docs/design/wireframe-ui-architecture.md` §1/§7）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_wireframe_ui::{icon, Size};
///
/// let node = icon(icon::search, Size::Lg);
/// let html = render(&node);
/// assert!(html.contains(r#"class="fw-wire-icon fw-wire-size-lg""#));
/// assert!(html.contains(r#"data-icon="search""#));
/// assert!(!html.contains(" role=\""));
/// assert!(!html.contains("aria-label"));
/// assert!(!html.contains("<button"));
/// assert!(!html.contains("<a "));
/// ```
#[must_use]
pub fn icon(glyph: fn(Size) -> Node, size: Size) -> Node {
    let class = class_list("fw-wire-icon", &[Some(size.class())]);
    el_owned(
        "span",
        vec![("class".to_string(), class)],
        vec![glyph(size)],
    )
}
