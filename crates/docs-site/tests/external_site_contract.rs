//! イシュー #3724: 外部リポジトリ相当の最小サイトを `docs-site --no-page-sections` で
//! 生成し、生成ファイル一覧・ブランド反映・帰属表記・予約パスの非注入を端から端まで固定する
//! 契約テスト。
//!
//! 設計の正は `docs/design/docs-site-external-use.md`。キー単位の検証は
//! `site_optional_keys.rs` / `site_brand_mark.rs` / `site_build.rs` が担い、本テストは
//! 「全キー同時指定 + 予約パス + リダイレクト」を 1 つのサイトで通した結果だけを固定する。
//! 入力は実ディレクトリ `tests/fixtures/site-external`（#3725 の CI ジョブも同じ入力を使う）。
//!
//! `fandhe-frontend` 検査の範囲（設計文書 §6 から #3724 へ委ねられた決定）:
//! - 対象は全出力ファイル（HTML・CSS・JS・JSON・SVG・リダイレクト案内）と出力の相対パス名。
//! - 検査語は `fandhe-frontend` と `Fandhe-AI`（ASCII 大文字小文字無視）。
//! - 許容はクロームを持つ HTML の `div.docs-footer-bottom` の 2 番目の `<p>`（帰属表記）のみ。
//! - `fandhe` だけを含む内部識別子（`--fandhe-*` など）と CLI の標準出力は対象外。

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use fandhe_frontend_docs_site::component_page;
use fandhe_frontend_docs_site::site_footer::FOOTER_BOTTOM_CLASS;

/// `Drop` 時に削除する一時ディレクトリ（`CARGO_TARGET_TMPDIR` 配下。`/tmp` へは出さない）。
struct TempDir(PathBuf);

impl TempDir {
    fn new(tag: &str) -> Self {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let path = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!(
            "fandhe-frontend-docs-site-external-{tag}-{}-{unique}",
            std::process::id()
        ));
        std::fs::create_dir_all(&path).expect("create temp dir");
        Self(path)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

type Tree = BTreeMap<String, Vec<u8>>;

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/site-external")
}

/// 外部利用者と同じ経路（バイナリ + `--no-page-sections`）で生成する。
fn run_docs_site(root: &Path, out: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_docs-site"))
        .arg("--no-page-sections")
        .arg("--root")
        .arg(root)
        .arg("--out")
        .arg(out)
        .output()
        .expect("spawn docs-site")
}

fn read_tree(out: &Path) -> Tree {
    fn walk(base: &Path, dir: &Path, acc: &mut Tree) {
        for e in std::fs::read_dir(dir).expect("read_dir") {
            let p = e.expect("entry").path();
            if p.is_dir() {
                walk(base, &p, acc);
            } else {
                let rel = p
                    .strip_prefix(base)
                    .unwrap()
                    .components()
                    .map(|c| c.as_os_str().to_string_lossy().into_owned())
                    .collect::<Vec<_>>()
                    .join("/");
                acc.insert(rel, std::fs::read(&p).expect("read file"));
            }
        }
    }
    let mut acc = Tree::new();
    walk(out, out, &mut acc);
    acc
}

/// `root` からビルドして成功を assert し、出力ツリーと標準出力を返す。
fn build_from(temp: &TempDir, root: &Path) -> (Tree, String) {
    let out = temp.0.join("out");
    let o = run_docs_site(root, &out);
    assert!(
        o.status.success(),
        "docs-site failed: {}",
        String::from_utf8_lossy(&o.stderr)
    );
    (
        read_tree(&out),
        String::from_utf8_lossy(&o.stdout).into_owned(),
    )
}

fn build_fixture(tag: &str) -> (TempDir, Tree, String) {
    let temp = TempDir::new(tag);
    let (tree, stdout) = build_from(&temp, &fixture_root());
    (temp, tree, stdout)
}

/// fixture を複製し、`nav.toml` の `[site]` から `drop` のキー行を除いたルートを返す。
fn copy_fixture_dropping_site_keys(temp: &TempDir, drop: &[&str]) -> PathBuf {
    let root = temp.0.join("src-root");
    let src = fixture_root().join("site");
    let dst = root.join("site");
    fn copy(from: &Path, to: &Path) {
        std::fs::create_dir_all(to).unwrap();
        for e in std::fs::read_dir(from).unwrap() {
            let p = e.unwrap().path();
            let t = to.join(p.file_name().unwrap());
            if p.is_dir() {
                copy(&p, &t);
            } else {
                std::fs::copy(&p, &t).unwrap();
            }
        }
    }
    copy(&src, &dst);
    let nav = std::fs::read_to_string(dst.join("nav.toml")).unwrap();
    let filtered: String = nav
        .lines()
        .filter(|l| !drop.iter().any(|k| l.starts_with(&format!("{k} = "))))
        .map(|l| format!("{l}\n"))
        .collect();
    std::fs::write(dst.join("nav.toml"), filtered).unwrap();
    root
}

fn text(tree: &Tree, rel: &str) -> String {
    String::from_utf8(
        tree.get(rel)
            .unwrap_or_else(|| panic!("missing output {rel}"))
            .clone(),
    )
    .unwrap_or_else(|_| panic!("{rel} is not utf-8"))
}

/// クロームを持つページ（通常 4 ページ + 404）。
const CHROME_PAGES: [&str; 5] = [
    "index.html",
    "guide/quickstart/index.html",
    "themes/accordion/index.html",
    "primitives/accordion/index.html",
    "404.html",
];

/// 帰属表記（`div.docs-footer-bottom` の 2 番目の `<p>`）と、それを除いた全文を返す。
fn attribution_split(html: &str) -> (String, String) {
    assert_eq!(
        html.matches(FOOTER_BOTTOM_CLASS).count(),
        1,
        "footer bottom class must appear exactly once"
    );
    let bottom = html.find(FOOTER_BOTTOM_CLASS).unwrap();
    let first_end = bottom + html[bottom..].find("</p>").expect("copyright </p>") + "</p>".len();
    let p_start = first_end + html[first_end..].find("<p").expect("attribution <p>");
    let p_end = p_start + html[p_start..].find("</p>").expect("attribution </p>") + "</p>".len();
    let attribution = html[p_start..p_end].to_owned();
    let rest = format!("{}{}", &html[..p_start], &html[p_end..]);
    (attribution, rest)
}

const NEEDLES: [&str; 2] = ["fandhe-frontend", "fandhe-ai"];

/// 検査語の出現を「相対パス: 文脈」で集める（帰属表記は許容）。
fn leaks(tree: &Tree) -> Vec<String> {
    let mut found = Vec::new();
    for (rel, bytes) in tree {
        let lower_rel = rel.to_ascii_lowercase();
        if NEEDLES.iter().any(|n| lower_rel.contains(n)) {
            found.push(format!("{rel}: <path name>"));
        }
        let body = String::from_utf8_lossy(bytes).into_owned();
        let scanned = if CHROME_PAGES.contains(&rel.as_str()) {
            attribution_split(&body).1
        } else {
            body
        };
        let lower = scanned.to_ascii_lowercase();
        for n in NEEDLES {
            let mut from = 0;
            while let Some(i) = lower[from..].find(n) {
                let at = from + i;
                let s = at.saturating_sub(30);
                let e = (at + n.len() + 30).min(lower.len());
                found.push(format!("{rel}: ...{}...", scanned[s..e].replace('\n', " ")));
                from = at + n.len();
            }
        }
    }
    found
}

#[test]
fn external_fixture_emits_exactly_the_expected_files() {
    let (_t, tree, stdout) = build_fixture("files");
    assert!(
        stdout.contains("wrote 4 page(s), 1 redirect(s) and 9 asset(s)"),
        "unexpected stdout: {stdout}"
    );
    let expected: BTreeSet<&str> = [
        "index.html",                      // 通常ページ（Home）
        "guide/quickstart/index.html",     // 通常ページ（Quickstart）
        "themes/accordion/index.html",     // 予約パスに置いた通常ページ
        "primitives/accordion/index.html", // 予約パスに置いた通常ページ
        "docs/quickstart/index.html",      // redirects.toml の案内ページ
        "404.html",                        // 404 ページ
        "assets/site.css",                 // 骨格 CSS
        "assets/skip-nav.css",             // スキップリンク CSS
        "assets/site.js",                  // 検索・テーマ等の JS
        "assets/theme-init.js",            // 保存済みテーマの初期化 JS
        "assets/favicon.svg",              // ブランドマーク
        "assets/search-index.json",        // 検索インデックスのマニフェスト
        "assets/search-index/guide.json",  // セクション別インデックス
        "assets/search-index/reference.json",
    ]
    .into_iter()
    .collect();
    let actual: BTreeSet<&str> = tree.keys().map(String::as_str).collect();
    assert_eq!(actual, expected, "output file set drifted");

    for (rel, bytes) in &tree {
        assert!(!bytes.is_empty(), "{rel} is empty");
        if rel.ends_with(".html") {
            assert!(
                String::from_utf8_lossy(bytes).starts_with("<!DOCTYPE html>"),
                "{rel} lacks doctype"
            );
        }
    }
    for rel in CHROME_PAGES {
        let html = text(&tree, rel);
        for asset in ["site.css", "site.js", "theme-init.js", "favicon.svg"] {
            assert!(
                html.contains(&format!("/acme-docs/assets/{asset}")),
                "{rel} does not reference {asset}"
            );
        }
    }
}

#[test]
fn brand_keys_are_reflected_on_every_chrome_page() {
    let (_t, tree, _) = build_fixture("brand");
    let favicon = text(&tree, "assets/favicon.svg");
    assert!(favicon.contains("fill=\"#AB12CD\""), "{favicon}");
    assert!(favicon.contains(">A</text>"));
    assert!(favicon.contains("aria-label=\"Acme\""));
    for rel in CHROME_PAGES {
        let html = text(&tree, rel);
        assert!(html.contains("<html lang=\"en\""), "{rel}: lang");
        assert!(html.contains(">Acme</a>"), "{rel}: header brand");
        assert!(html.contains("Docs for the Acme toolkit"), "{rel}: tagline");
        assert!(html.contains("(c) 2026 Acme Inc."), "{rel}: copyright");
        assert!(html.contains("v1.2.3"), "{rel}: badge");
        assert_eq!(
            html.matches("href=\"https://example.com/acme/acme-docs\"")
                .count(),
            2,
            "{rel}: repository links"
        );
        for absent in ["core v", "crates.io/crates", "Licensed under"] {
            assert!(!html.contains(absent), "{rel}: unexpected {absent}");
        }
        assert!(html.contains("Content-Security-Policy"), "{rel}: CSP");
        let start = html.find("class=\"docs-brand\"").expect("docs-brand");
        let svg_start = start + html[start..].find("<svg").expect("svg");
        let svg_end = svg_start + html[svg_start..].find("</svg>").unwrap() + "</svg>".len();
        assert_eq!(&html[svg_start..svg_end], favicon, "{rel}: header svg");
    }
    assert!(text(&tree, "docs/quickstart/index.html").contains("<html lang=\"en\""));
}

#[test]
fn fandhe_frontend_appears_only_in_the_attribution() {
    // 前提: 入力側に検査語がないので、出力の検査語は生成器由来だと言える。
    let mut inputs = Vec::new();
    fn collect(dir: &Path, acc: &mut Vec<PathBuf>) {
        for e in std::fs::read_dir(dir).unwrap() {
            let p = e.unwrap().path();
            if p.is_dir() {
                collect(&p, acc);
            } else {
                acc.push(p);
            }
        }
    }
    collect(&fixture_root(), &mut inputs);
    for p in inputs {
        let lower = std::fs::read_to_string(&p).unwrap().to_ascii_lowercase();
        assert!(
            !lower.contains("fandhe"),
            "fixture input mentions fandhe: {p:?}"
        );
    }

    let (_t, tree, _) = build_fixture("leak");
    let found = leaks(&tree);
    assert!(
        found.is_empty(),
        "brand leaks outside attribution:\n{}",
        found.join("\n")
    );

    for rel in CHROME_PAGES {
        let (attr, _) = attribution_split(&text(&tree, rel));
        assert!(attr.contains("Built with "), "{rel}: {attr}");
        let count = |needle: &str| attr.matches(needle).count();
        assert_eq!(
            count("href=\"https://github.com/Fandhe-AI/fandhe-frontend\""),
            1,
            "{rel}"
        );
        assert_eq!(count(">fandhe-frontend docs-site<"), 1, "{rel}");
        assert_eq!(count("LICENSE-MIT\""), 1, "{rel}: MIT link");
        assert_eq!(count("LICENSE-APACHE\""), 1, "{rel}: Apache link");
        assert!(attr.contains("rel=\"noopener noreferrer\""), "{rel}");
    }
}

#[test]
fn leak_scan_detects_default_brand_and_pins_the_required_keys() {
    // 既定文言を変えてこのテストが落ちたら、利用ガイドと設計文書の「必要十分なキー」も直す。
    let build_dropping = |tag: &str, drop: &[&str]| {
        let temp = TempDir::new(tag);
        let root = copy_fixture_dropping_site_keys(&temp, drop);
        let (tree, _) = build_from(&temp, &root);
        (temp, tree)
    };

    let all = [
        "brand",
        "repository_url",
        "tagline",
        "copyright",
        "version_badge",
        "lang",
        "brand_mark",
        "brand_color",
    ];
    let (_t, tree) = build_dropping("neg-all", &all);
    let found = leaks(&tree);
    assert!(
        found.iter().any(|f| f.starts_with("index.html:")),
        "{found:?}"
    );
    assert!(
        found.iter().any(|f| f.starts_with("assets/favicon.svg:")),
        "{found:?}"
    );

    for key in ["brand", "repository_url", "copyright", "version_badge"] {
        let (_t, tree) = build_dropping(&format!("neg-{key}"), &[key]);
        assert!(
            !leaks(&tree).is_empty(),
            "dropping {key} must leak the default brand"
        );
    }
    for key in ["tagline", "lang", "brand_mark", "brand_color"] {
        let (_t, tree) = build_dropping(&format!("pos-{key}"), &[key]);
        let found = leaks(&tree);
        assert!(found.is_empty(), "dropping {key} must not leak: {found:?}");
    }
}

#[test]
fn reserved_paths_render_only_the_fixture_markdown() {
    // 陽性対照: 本サイトなら 2 パスはショーケース注入の対象である。
    for path in ["/themes/accordion/", "/primitives/accordion/"] {
        assert!(
            component_page::generated_content(path).is_some(),
            "{path} is no longer a showcase path; pick another fixture path"
        );
    }
    let (_t, tree, _) = build_fixture("reserved");
    let links = |html: &str| -> Vec<String> {
        html.match_indices("<link ")
            .filter_map(|(i, _)| {
                let tag = &html[i..i + html[i..].find('>')?];
                tag.contains("rel=\"stylesheet\"").then(|| {
                    let h = tag.find("href=\"").unwrap() + 6;
                    tag[h..h + tag[h..].find('"').unwrap()].to_owned()
                })
            })
            .collect()
    };
    let baseline = links(&text(&tree, "guide/quickstart/index.html"));
    for (rel, note) in [
        ("themes/accordion/index.html", "Acme accordion theme notes."),
        (
            "primitives/accordion/index.html",
            "Acme accordion primitive notes.",
        ),
    ] {
        let html = text(&tree, rel);
        assert!(html.contains(note), "{rel}: body");
        assert!(html.contains("Usage"), "{rel}: heading");
        for absent in [
            "data-scope=\"accordion\"",
            "pre-styled-showcase",
            "showcase-anatomy",
            "primitives-demo-anatomy",
            "pre-styled-ui.css",
            "primitives-showcase.css",
            "site-primitives.css",
            "blocks.css",
            "wireframes.css",
        ] {
            assert!(!html.contains(absent), "{rel}: showcase leak {absent}");
        }
        assert_eq!(links(&html), baseline, "{rel}: stylesheet set differs");
    }
    for asset in [
        "assets/pre-styled-ui.css",
        "assets/primitives-showcase.css",
        "assets/site-primitives.css",
        "assets/image-demo.svg",
    ] {
        assert!(!tree.contains_key(asset), "{asset} must not be emitted");
    }
}

/// 日本語（ひらがな・カタカナ・漢字）を含むか。
fn has_cjk(s: &str) -> bool {
    s.chars()
        .any(|c| ('\u{3040}'..='\u{30ff}').contains(&c) || ('\u{4e00}'..='\u{9fff}').contains(&c))
}

/// fixture は `lang = "en"`・GitHub 以外の `repository_url`。生成器が出すクローム文言は
/// 全て英語になり（検索 UI・ページャ・404・リダイレクト案内）、リポジトリリンクは
/// "Repository" と汎用アイコンになる（`docs/design/docs-site-external-use.md` §4.3）。
#[test]
fn en_fixture_emits_english_chrome_and_generic_repository_link() {
    let (_temp, tree, _stdout) = build_fixture("en-chrome");
    for rel in CHROME_PAGES {
        let html = text(&tree, rel);
        assert!(html.contains("<html lang=\"en\">"), "{rel}");
        assert!(!has_cjk(&html), "{rel}: CJK text in an English site");
        assert!(
            html.contains("aria-label=\"Search documentation\""),
            "{rel}"
        );
        assert!(
            html.contains("placeholder=\"Search documentation\""),
            "{rel}"
        );
        // ヘッダーとフッター Resources 列の 2 か所が "Repository"、旧固定文言の表記は無い。
        assert_eq!(html.matches(">Repository<").count(), 2, "{rel}");
        assert!(!html.contains(">GitHub<"), "{rel}");
    }
    let guide = text(&tree, "guide/quickstart/index.html");
    assert!(guide.contains("aria-label=\"Previous and next pages\""));
    assert!(guide.contains(">Previous<"));
    let not_found = text(&tree, "404.html");
    assert!(not_found.contains("<title>Page not found"));
    assert!(not_found.contains("Main sections"));
    let redirect = text(&tree, "docs/quickstart/index.html");
    assert!(redirect.contains("Moved | Acme Docs"));
    assert!(redirect.contains("This page has moved."));
    assert!(!has_cjk(&redirect));
    // 帰属表記の文言は言語で変えない。
    let (attribution, _) = attribution_split(&guide);
    assert!(attribution.contains("Built with "));
    assert!(attribution.contains("fandhe-frontend docs-site"));
}

/// `lang` を外す（既定 `ja`）と、同じ fixture で現行の日本語文言になる。
/// リポジトリリンクの文言は `lang` と無関係にホストだけで決まる。
#[test]
fn default_lang_fixture_keeps_japanese_chrome() {
    let temp = TempDir::new("ja-chrome");
    let root = copy_fixture_dropping_site_keys(&temp, &["lang"]);
    let (tree, _stdout) = build_from(&temp, &root);
    let not_found = text(&tree, "404.html");
    assert!(not_found.contains("<html lang=\"ja\">"));
    assert!(not_found.contains("ページが見つかりません"));
    assert!(not_found.contains("aria-label=\"ドキュメントを検索\""));
    assert!(text(&tree, "docs/quickstart/index.html").contains("移転しました | Acme Docs"));
    assert_eq!(not_found.matches(">Repository<").count(), 2);
}
