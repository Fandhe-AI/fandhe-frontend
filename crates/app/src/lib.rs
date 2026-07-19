//! `fandhe-frontend-app`: SSR / SSG / CSR 三モード共通のモード非依存コンポーネントライブラリ。
//!
//! `fandhe-frontend-core`（`Node` / `el` / `text` / `render` 等）**のみ**に依存する。マクロ DSL
//! には依存せず、`Node` を返す通常の Rust 関数としてコンポーネントを記述する
//! （`docs/api/component-api.md` の「コンポーネント記述の標準規約」に従う）。
//!
//! # 三モード契約（REQ-6）
//!
//! [`list_page`] / [`detail_page`] / [`page_shell`] は、SSR（`fandhe-frontend-server` の
//! axum ハンドラ想定・TASK-6.1c）・SSG（同クレートの静的書き出しバイナリ想定）・
//! CSR（`fandhe-frontend-wasm-client` 想定・TASK-6.2 系）の**いずれのモードからも同一関数が
//! そのまま呼ばれる**ことを前提とする。モード別の分岐・モード別の出力差異を
//! 本クレートに持ち込まない（同一入力に対し常に同一の [`fandhe_frontend_core::render`]
//! 出力を返すことをテストで固定する）。
//!
//! # 既定エスケープの引き継ぎ（REQ-1）
//!
//! 本クレートはテキスト・属性値をすべて `fandhe_frontend_core::text` / `fandhe_frontend_core::el` の
//! attrs 経由で組み立て、独自のエスケープ処理・独自の raw 出力経路を持たない。
//! `format!` によるタグ文字列の直接組み立ては行わない（`coding-rust.md`
//! 「HTML 文字列の直接組み立て禁止」）。[`page_shell`] が前置する
//! `<!DOCTYPE html>` のみ、ユーザー入力を一切含まない固定リテラルとして
//! 文字列結合する（`fandhe_frontend_core::render` 済みの既定エスケープ済み HTML の前に
//! 付与するのみであり、新たな迂回経路ではない）。
//!
//! # スコープ外
//!
//! ハイドレーション支援 API（`find_attr_values`/`find_nav_targets` 相当）は
//! `fandhe-frontend-core` 側の TASK-6.2 系で追加予定であり、本クレートでは使用しない。
//! `server/src/main.rs`（SSR/SSG エントリ）は TASK-6.1c、三モード統合テストは
//! TASK-6.1d のスコープであり本クレートには含めない。
//!
//! # ルーティング（[`router`] / [`routes`]、イシュー #407）
//!
//! `server`（SSR/SSG）・`wasm-full`（CSR）双方から依存可能な唯一の層
//! （`structure.toml` の `allowed_dependents` 参照）として、パスマッチング
//! エンジン（[`router`]）とルート表の単一定義（[`routes`]）を本クレートへ
//! 集約する。詳細は各モジュールの doc コメントと
//! `docs/design/route-definition-sharing.md` を参照。

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use fandhe_frontend_core::{a, div, el, h1, li, main_tag, p, text, ul, Node};
use std::convert::Infallible;

pub mod router;
pub mod routes;

/// ハイドレーション後にクライアント側の `click` イベントで参照される
/// `id` 属性値。`fandhe-frontend-wasm-client`（TASK-6.2 系）がこの定数で DOM 要素を
/// 検索する前提の契約であり、値を変更する場合はクライアント側と合わせて
/// 更新する必要がある。
pub const LIKE_BUTTON_ID: &str = "like-btn";

/// 一覧・詳細画面の最小データモデル。
///
/// PoC-3 の固定データ構造を踏襲しつつ、フィールドをすべて所有型
/// （`String`）に一般化している。SSR/SSG/CSR いずれの呼び出し元も、
/// データベース・API・埋め込みデータ等の由来を問わず本構造体を組み立てて
/// [`list_page`] / [`detail_page`] に渡すことを想定する（PoC-3 のような
/// クレート内固定データへの決め打ちを避けるための一般化）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    /// 一覧・詳細間の参照キー。URL パス片（`/items/{id}`）にそのまま使う。
    pub id: String,
    /// 表示用タイトル。[`text`] 経由で必ず既定エスケープされる。
    pub title: String,
    /// 本文。タイトルと同様に既定エスケープ対象。
    pub body: String,
}

/// デモ・テスト用の固定データ（TASK-6.1c 以降が実データ接続するまでの暫定値）。
///
/// `demo_items()[1]` の title に意図的な XSS ペイロードを含めており、
/// [`list_page`] / [`detail_page`] の既定エスケープ回帰テストの入力として
/// も利用する（PoC-2/PoC-3 の XSS 実証データを踏襲）。
pub fn demo_items() -> Vec<Item> {
    vec![
        Item {
            id: "1".to_string(),
            title: "Rust 製フロントエンド基盤の構想".to_string(),
            body: "安全性・Web 標準尊重・思想のグラデーション・単一バイナリ配布を統合する。"
                .to_string(),
        },
        Item {
            id: "2".to_string(),
            title: "<script>alert('xss')</script><img src=x onerror=alert(1)>".to_string(),
            body: "このタイトルは意図的な XSS ペイロードであり、既定エスケープの実証に使う。"
                .to_string(),
        },
        Item {
            id: "3".to_string(),
            title: "View Transitions API の薄いラッパー評価".to_string(),
            body: "標準 API を直接呼び出す形でページ遷移を演出できるかを検証する。".to_string(),
        },
    ]
}

/// SSR・SSG・CSR の三モードから同一実装が呼ばれるデータ取得契約
/// （イシュー #346 設計確定書 `docs/design/loader-trait-design.md` §3.2 の
/// 凍結シグネチャに一字一句準拠する。実装が本 trait と乖離した場合は
/// 同設計書を正とする）。
///
/// `load()` の実装は 1 箇所のみとし、モード別の分岐を持たない
/// （REQ-6 の三モード契約を Loader にも適用する）。`fandhe-frontend-server`（#348）・
/// `fandhe-frontend-wasm-full`（#349）はいずれも本 trait の同一 `impl` を呼ぶのみで、
/// モードごとに別実装を作らない。
///
/// # 型で保証する範囲（設計書 §3.4）
///
/// 保証するのは `Output` 型とページ関数（[`list_page`] / [`detail_page`]）
/// への型接続のみであり、`load` 自体の実行時決定性（外界 I/O を含みうる）
/// は型システムの外側（テスト）の責務とする。
pub trait Loader {
    /// ルートパラメータ等の解決入力（例: 一覧 = `()`、詳細 = id）。
    type Input;
    /// ページ関数への唯一のデータ源。
    type Output;
    /// 解決失敗を表す型。
    ///
    /// 表示用文字列に内部パス・スタックトレース・接続情報等の内部情報を
    /// 含めない契約とする（fail-closed、`security.md`「機微情報の露出」・
    /// 設計書 §5・§9-5）。エラー時の実際の応答（500 / ビルド失敗 / 固定
    /// エラービュー）は呼び出し元（`fandhe-frontend-server` #348・`fandhe-frontend-wasm-full` #349）
    /// の責務であり、本 trait はエラーの型のみを規定する。
    type Error;

    /// `input` からデータを解決する。三モードいずれの呼び出し元からも
    /// 同一実装が呼ばれる（型で保証する範囲は本 trait の rustdoc 冒頭を
    /// 参照）。
    fn load(&self, input: &Self::Input) -> Result<Self::Output, Self::Error>;
}

/// 一覧画面（[`list_page`]）向けの参照 loader 実装。
///
/// 内部で [`demo_items()`] を呼ぶのみであり、デモデータは解決に失敗しない
/// ため `Error = Infallible` とする（設計書 §7.1 の参照実装）。#348 が
/// エラー経路をテストする際は、この loader とは別に失敗する loader を
/// server 側テストで定義できる（`Loader` は汎用のまま）。
#[derive(Debug, Clone, Copy, Default)]
pub struct DemoItemsLoader;

impl Loader for DemoItemsLoader {
    type Input = ();
    type Output = Vec<Item>;
    type Error = Infallible;

    fn load(&self, _input: &()) -> Result<Vec<Item>, Infallible> {
        Ok(demo_items())
    }
}

/// 詳細画面（[`detail_page`]）向けの参照 loader 実装。
///
/// `Input` は URL パス片（`/items/{id}`）に対応する id 文字列。id が
/// [`demo_items()`] に存在しない場合は `Output = None` を返す（見つから
/// ない、を `Error` ではなく `Output` の一部として表現することで、404
/// 相当を fail-closed なエラー扱いにしない — 設計書 §3.3 の
/// `detail_page(item: Option<&Item>)` 契約とそのまま接続する）。
#[derive(Debug, Clone, Copy, Default)]
pub struct DemoItemDetailLoader;

impl Loader for DemoItemDetailLoader {
    type Input = String;
    type Output = Option<Item>;
    type Error = Infallible;

    fn load(&self, id: &String) -> Result<Option<Item>, Infallible> {
        Ok(demo_items().into_iter().find(|it| &it.id == id))
    }
}

/// loader の解決結果を一覧ページへ型接続する。
///
/// `L::Output` が `Vec<Item>` でない loader を渡すとコンパイルエラーに
/// なる（`where` 束縛による型接続。設計書 §3.4 の「保証する範囲」）。
/// `load` が失敗した場合は `?` で即座に `Err` を返し、未解決データで
/// 描画を続行しない（fail-closed、設計書 §5）。呼び出し元（`fandhe-frontend-server`
/// #348・`fandhe-frontend-wasm-full` #349）がこの `Err` をモードごとの応答（500 /
/// ビルド失敗 / 固定エラービュー）へ変換する。
///
/// `list_page` の引数は `&[Item]` であり `&L::Output`（`&Vec<Item>`）とは
/// 借用の形が異なるため、`.as_slice()` で薄く変換する（設計書 §3.3 注記。
/// `list_page` 自体の純関数シグネチャは変更しない）。
pub fn assemble_list_page<L>(loader: &L, input: &L::Input) -> Result<Node, L::Error>
where
    L: Loader<Output = Vec<Item>>,
{
    Ok(list_page(loader.load(input)?.as_slice()))
}

/// loader の解決結果を詳細ページへ型接続する。[`assemble_list_page`] と
/// 同様に `where` 束縛で `Output` を `Option<Item>` に固定し、型不整合を
/// コンパイルエラーにする。
///
/// `detail_page` の引数は `Option<&Item>` であり `&L::Output`
/// （`&Option<Item>`）とは `Option` の内外どちらを参照で包むかが異なる
/// ため、`.as_ref()` で薄く変換する（設計書 §3.3 注記）。
pub fn assemble_detail_page<L>(loader: &L, input: &L::Input) -> Result<Node, L::Error>
where
    L: Loader<Output = Option<Item>>,
{
    Ok(detail_page(loader.load(input)?.as_ref()))
}

/// 共通レイアウト（ヘッダー相当）。[`list_page`] / [`detail_page`] の両方から
/// 呼ばれる、モード非依存の骨格コンポーネント。
///
/// `title` は [`fandhe_frontend_core::text`] 経由で渡すため既定エスケープされる
/// （呼び出し元が信頼できない文字列を渡しても生タグとして解釈されない）。
pub fn layout(title: &str, body: Node) -> Node {
    el(
        "div",
        vec![("id", "app-root"), ("data-fandhe-frontend", "root")],
        vec![h1(vec![], vec![text(title)]), main_tag(vec![], vec![body])],
    )
}

/// 画面 1: 一覧画面。各項目へのリンクに `data-nav` 属性を付与する
/// （`fandhe-frontend-core` 側 TASK-6.2 系のハイドレーション支援 API がこの属性を
/// 実 DOM なしに機械的検出する前提の契約。本クレートでは検出処理自体は
/// 実装しない＝スコープ外）。
///
/// `items` は呼び出し元（SSR/SSG/CSR いずれの層）が用意したデータをそのまま
/// 受け取る。本関数はモード分岐を持たず、同一引数には常に同一の [`Node`]
/// 木を返す（REQ-6 のモード非依存性契約）。
pub fn list_page(items: &[Item]) -> Node {
    let list_items: Vec<Node> = items
        .iter()
        .map(|it| {
            let href = format!("/items/{}", it.id);
            li(
                vec![],
                vec![a(
                    vec![("href", &href), ("data-nav", &href)],
                    vec![text(it.title.clone())],
                )],
            )
        })
        .collect();
    layout(
        "記事一覧",
        ul(vec![("data-testid", "item-list")], list_items),
    )
}

/// 画面 2: 詳細画面。呼び出し元が対象 `Item` の解決（ID 引き当て）を
/// 済ませた結果を `Option<&Item>` として受け取る（本クレートは検索・
/// データストアの責務を持たない）。`None` の場合は 404 相当のノードを返し、
/// ライブラリコードで `panic!` しない（`coding-rust.md` のエラー処理規約）。
pub fn detail_page(item: Option<&Item>) -> Node {
    match item {
        Some(item) => layout(
            "記事詳細",
            div(
                vec![("data-testid", "item-detail")],
                vec![
                    p(
                        vec![("data-testid", "item-title")],
                        vec![text(item.title.clone())],
                    ),
                    p(
                        vec![("data-testid", "item-body")],
                        vec![text(item.body.clone())],
                    ),
                    el(
                        "button",
                        vec![("id", LIKE_BUTTON_ID), ("data-hydrate", "like")],
                        vec![text("いいね")],
                    ),
                    a(
                        vec![("href", "/"), ("data-nav", "/")],
                        vec![text("一覧へ戻る")],
                    ),
                ],
            ),
        ),
        None => layout(
            "見つかりません",
            p(vec![], vec![text("指定された記事は存在しません。")]),
        ),
    }
}

/// ページ全体（`<!DOCTYPE html>` を含む完全文書）を組み立てる。
/// SSR（axum ハンドラ想定）・SSG（静的書き出しバイナリ想定）の両方から
/// 呼ばれる共通関数（TASK-6.1c で実際のエントリポイントが接続される）。
///
/// `title` は [`fandhe_frontend_core::el`] の `<title>` 子ノードとして [`text`] 経由で
/// 渡すため既定エスケープされる（PoC-3 の手動 `escape_html` 呼び出しより
/// 安全な構造。`text()` を経由しない独自のエスケープ処理を持たない）。
/// `<!DOCTYPE html>` はユーザー入力を一切含まない固定リテラルとして
/// [`fandhe_frontend_core::render`] 済みの文字列の前に結合するのみであり、新たな
/// エスケープ迂回経路ではない。
///
/// `@view-transition { navigation: auto; }`（CSS Level 2 の at-rule）は
/// Cross-Document View Transitions を有効化する。過去の `<meta
/// name="view-transition" content="same-origin">` は現行ブラウザ・仕様で
/// 廃止扱いのため採用しない（Bugbot 指摘対応）。この CSS はユーザー入力を
/// 含まない固定リテラルであり `text()` 経由で `<style>` 子ノードとして
/// 出力するため、既定エスケープ経路を迂回しない。フレームワーク固有の
/// JS ラッパーを必要としない（PoC-3 の検証結果を踏襲）。
pub fn page_shell(title: &str, body: Node) -> String {
    let head = el(
        "head",
        vec![],
        vec![
            el("meta", vec![("charset", "utf-8")], vec![]),
            el(
                "meta",
                vec![
                    ("name", "viewport"),
                    ("content", "width=device-width, initial-scale=1"),
                ],
                vec![],
            ),
            el(
                "style",
                vec![],
                vec![text("@view-transition { navigation: auto; }")],
            ),
            el("title", vec![], vec![text(title)]),
            el(
                "link",
                vec![("rel", "stylesheet"), ("href", "/static/style.css")],
                vec![],
            ),
        ],
    );
    let document_body = el(
        "body",
        vec![],
        vec![
            body,
            el(
                "script",
                vec![("type", "module"), ("src", "/static/hydrate.js")],
                vec![],
            ),
        ],
    );
    let html = el("html", vec![("lang", "ja")], vec![head, document_body]);
    format!("<!DOCTYPE html>\n{}", fandhe_frontend_core::render(&html))
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;

    /// REQ-6 中核: 同一関数（`list_page`）を 2 回呼び出しても完全一致する
    /// ことを固定する。SSR で呼んでも CSR で呼んでも同一関数・同一入力なら
    /// 同一 DOM が得られるという三モード契約をそのまま証明する。
    #[test]
    fn list_page_render_is_mode_independent_and_matches_expected_dom() {
        let items = demo_items();
        let html_as_ssr = render(&list_page(&items));
        let html_as_csr = render(&list_page(&items));
        assert_eq!(
            html_as_ssr, html_as_csr,
            "SSR/CSR で同一コードから同一 DOM が得られること"
        );

        assert!(html_as_ssr.contains(r#"data-testid="item-list""#));
        assert!(html_as_ssr.contains("Rust 製フロントエンド基盤の構想"));
        assert!(html_as_ssr.contains(r#"data-nav="/items/1""#));
        // XSS ペイロードはテキストノード経由のため既定エスケープされる。
        assert!(!html_as_ssr.contains("<script>alert"));
        assert!(html_as_ssr.contains("&lt;script&gt;alert"));
    }

    #[test]
    fn detail_page_render_matches_expected_dom_for_existing_item() {
        let items = demo_items();
        let item = items.iter().find(|it| it.id == "1");
        let html = render(&detail_page(item));
        assert!(html.contains(r#"data-testid="item-detail""#));
        assert!(html.contains("Rust 製フロントエンド基盤の構想"));
        assert!(html.contains("一覧へ戻る"));
        assert!(html.contains(LIKE_BUTTON_ID));
    }

    #[test]
    fn detail_page_render_handles_missing_item() {
        let html = render(&detail_page(None));
        assert!(html.contains("見つかりません"));
    }

    /// PoC-3 成功基準 1（SSG 側）: SSG が書き出す文字列は SSR が返す文字列と
    /// 完全一致すること（同一コードであることの直接証明）。
    #[test]
    fn ssg_output_equals_ssr_output_for_list_and_detail() {
        let items = demo_items();
        let ssr_list = render(&list_page(&items));
        let ssg_list = render(&list_page(&items));
        assert_eq!(ssr_list, ssg_list);

        let item = items.iter().find(|it| it.id == "2");
        let ssr_detail = render(&detail_page(item));
        let ssg_detail = render(&detail_page(item));
        assert_eq!(ssr_detail, ssg_detail);
        // demo_items()[1] の title は XSS ペイロード。detail_page 経由でも
        // 既定エスケープされることを確認する。
        assert!(!ssr_detail.contains("<script>alert"));
        assert!(ssr_detail.contains("&lt;script&gt;alert"));
    }

    #[test]
    fn page_shell_includes_view_transition_at_rule_and_matches_across_ssr_and_ssg() {
        let items = demo_items();
        let ssr_doc = page_shell("記事一覧", list_page(&items));
        let ssg_doc = page_shell("記事一覧", list_page(&items));
        assert_eq!(ssr_doc, ssg_doc);
        assert!(ssr_doc.contains("<style>@view-transition { navigation: auto; }</style>"));
        assert!(ssr_doc.starts_with("<!DOCTYPE html>"));
    }

    /// `page_shell` の `title` は既定エスケープされ、`<title>` タグ内で
    /// XSS ペイロードがそのまま解釈されないことを確認する（REQ-1 の
    /// 三経路目: レイアウト title・詳細/一覧本文に続く page_shell title 経路）。
    #[test]
    fn page_shell_escapes_title_to_prevent_xss() {
        let doc = page_shell("<script>alert('xss')</script>", div(vec![], vec![]));
        assert!(!doc.contains("<title><script>alert"));
        assert!(doc.contains("<title>&lt;script&gt;alert(&#x27;xss&#x27;)&lt;/script&gt;</title>"));
    }

    /// `layout` 単体の既定エスケープ回帰（`h1` タイトル経由）。
    #[test]
    fn layout_escapes_title_to_prevent_xss() {
        let html = render(&layout(
            "<script>alert('xss')</script>",
            p(vec![], vec![text("body")]),
        ));
        assert!(!html.contains("<script>alert"));
        assert!(html.contains("&lt;script&gt;alert"));
    }

    /// 設計書 §3.4「型で保証しない範囲」の実行時側: 同一 `Input` を渡した
    /// `DemoItemsLoader::load` の呼び出し結果が完全一致することを固定する
    /// （REQ-6 の三モード契約を loader にも適用したことの決定性証明）。
    #[test]
    fn demo_items_loader_load_is_deterministic_for_same_input() {
        let loader = DemoItemsLoader;
        let first = loader.load(&()).expect("Infallible は必ず Ok");
        let second = loader.load(&()).expect("Infallible は必ず Ok");
        assert_eq!(first, second);
    }

    /// 受け入れ条件 1 の直接証明: loader 経由（`assemble_list_page`）と
    /// 純関数直呼び（`list_page(&demo_items())`）が完全一致する Node 木を
    /// 生成する。`Output = Vec<Item>` という `where` 束縛が
    /// `list_page` の引数型と接続していることを実行結果でも裏付ける。
    #[test]
    fn assemble_list_page_matches_direct_list_page_call() {
        let via_loader =
            render(&assemble_list_page(&DemoItemsLoader, &()).expect("Infallible は必ず Ok"));
        let direct = render(&list_page(&demo_items()));
        assert_eq!(via_loader, direct);
    }

    /// `assemble_detail_page` の型接続版。存在する id では
    /// `detail_page(Some(..))` と、存在しない id では `detail_page(None)`
    /// と同一の Node 木になることを確認する。
    #[test]
    fn assemble_detail_page_matches_direct_detail_page_call_for_existing_and_missing_id() {
        let loader = DemoItemDetailLoader;

        let via_loader =
            render(&assemble_detail_page(&loader, &"1".to_string()).expect("Infallible は必ず Ok"));
        let items = demo_items();
        let direct = render(&detail_page(items.iter().find(|it| it.id == "1")));
        assert_eq!(via_loader, direct);
        assert!(via_loader.contains(r#"data-testid="item-detail""#));

        let via_loader_missing = render(
            &assemble_detail_page(&loader, &"does-not-exist".to_string())
                .expect("Infallible は必ず Ok"),
        );
        assert!(via_loader_missing.contains("見つかりません"));
    }

    /// loader 経由の XSS 回帰: `demo_items()[1]` の XSS ペイロードが
    /// `assemble_list_page` / `assemble_detail_page` 経路でも既定エスケープ
    /// されることを確認する（既存の直接呼び出し経路の XSS 回帰テストを
    /// 弱体化させず、loader 経路の同等テストを追加する形を取る）。
    #[test]
    fn assemble_pages_escape_xss_payload_via_loader_path() {
        let list_html = render(&assemble_list_page(&DemoItemsLoader, &()).expect("Infallible"));
        assert!(!list_html.contains("<script>alert"));
        assert!(list_html.contains("&lt;script&gt;alert"));

        let detail_html = render(
            &assemble_detail_page(&DemoItemDetailLoader, &"2".to_string()).expect("Infallible"),
        );
        assert!(!detail_html.contains("<script>alert"));
        assert!(detail_html.contains("&lt;script&gt;alert"));
    }
}
