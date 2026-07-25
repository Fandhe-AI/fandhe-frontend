//! docs サイトが出力する唯一の JS（イシュー #951）。
//!
//! # 役割・呼び出し文脈
//!
//! docs サイト（`crate::layout`）は #951 以前は JS を 1 バイトも出力して
//! いなかった（`crate::layout` モジュール doc の旧宣言参照）。本モジュールは
//! テーマトグル（ダーク/ライト切替）の実装として、初めて docs サイトへ
//! クライアント側 JS を持ち込む。
//!
//! - [`INLINE_THEME_BOOTSTRAP`]: `crate::layout::docs_page_with_assets` が
//!   `<head>` の先頭付近（スタイルシートより前）へ同期実行の `<script>` として
//!   埋め込む FOUC 抑止スニペット。`localStorage` に保存済みのテーマがあれば
//!   CSS 適用前に `<html data-theme="...">` を確定させる。
//! - [`SITE_JS`]: `crate::build::build_site` が [`SCRIPT_REL_PATH`]
//!   （`out_dir` 起点）へ書き出す本体。`.docs-theme-toggle` ボタンの
//!   ラベル・`aria-pressed` 更新、クリック時の切替・保存、および
//!   `hidden` 属性の解除（配線完了後にのみ可視化する）を担う。加えて
//!   イシュー #950 で右目次（`crate::layout::toc_nav` が出力する
//!   `nav.docs-toc`）のスクロールスパイ（現在地ハイライト）を担う。加えて
//!   イシュー #958 で検索 UI（`crate::layout::docs_page_with_assets` が
//!   出力する `div.docs-search`）を担う: 初回フォーカス時に
//!   `crate::search_index` が生成する `assets/search-index.json` を遅延
//!   `fetch()` し、部分一致検索・結果一覧描画・キーボード操作
//!   （`/`・矢印キー・`Enter`・`Escape`）を配線する。
//!   テーマトグルの IIFE は `.docs-theme-toggle` が無いページで早期
//!   return するため、目次ハイライト・検索はそれぞれ**別の独立した IIFE**
//!   として実装し、テーマトグルの早期 return に巻き込まれないようにする
//!   （テーマトグル・検索のいずれかが無い構成でも残りの機能が動作する
//!   必要があるため）。
//!
//! # セキュリティ不変条件（REQ-1、`.claude/rules/coding-rust.md`）
//!
//! [`Node::Text`]（`fandhe_frontend_core`）は `<script>` の中身であっても
//! 必ず [`fandhe_frontend_core::escape_html_into`] を経由する。`<script>` の
//! 中身は HTML パーサが実体参照を復号しない raw text であるため、
//! エスケープ対象文字（`< > & " '`）を 1 文字でも含む JS ソースを
//! `text()` 経由で埋め込むと構文が壊れる。[`INLINE_THEME_BOOTSTRAP`] は
//! 文字列リテラルにバッククォート（テンプレートリテラル）のみを使い、
//! `&&` の代わりに `||` を使うことでこれらの文字を一切含まない。
//! [`is_escape_safe`] がこの性質をコンパイル後にも機械検証し、
//! [`inline_theme_bootstrap`] は検証に落ちた場合 `None` を返す
//! fail-closed のアクセサとする（`raw_html()` は新規に導入しない）。
//!
//! `${`（テンプレートリテラル補間）も [`is_escape_safe`] の対象外文字列
//! として禁止する。本モジュールの定数はすべて `&'static str` で外部入力・
//! `nav.toml` 由来の値を一切含まないが、将来の変数補間の混入を
//! テストで機械的にブロックする構造的な防御である。
//!
//! `localStorage` はスクリプトの実行主体（同一オリジンの他スクリプト・
//! 利用者自身）が改変できる非信頼データのため、[`INLINE_THEME_BOOTSTRAP`]・
//! [`SITE_JS`] のいずれも読み出した値を `dark`/`light` の allowlist と
//! 一致した場合のみ `data-theme` へ反映する。

/// [`SITE_JS`] の出力先（`out_dir` 起点の相対パス）。
/// `crate::build::build_site` が本パスへ書き出し、
/// `crate::layout::docs_page_with_assets` が `<script src>`（`defer`）で参照する
/// 単一実装点。
pub const SCRIPT_REL_PATH: &str = "assets/site.js";

/// テーマ選択を保存する `localStorage` キー。[`INLINE_THEME_BOOTSTRAP`] と
/// [`SITE_JS`] の双方が同じキーを参照する契約であることを
/// `script_js_and_inline_bootstrap_share_the_same_storage_key`
/// （本モジュールの `tests`）が固定する（キー名の二重管理ドリフト検知）。
pub const THEME_STORAGE_KEY: &str = "fandhe-docs-theme";

/// `<head>` の先頭付近（スタイルシートより前）に同期実行で埋め込む
/// FOUC 抑止スニペット。`localStorage` から保存済みテーマを読み、
/// `dark`/`light` のいずれかであれば `<html>` の `data-theme` 属性を
/// CSS 適用前に確定させる。`localStorage` アクセス例外（Safari プライベート
/// ブラウズ等）は握りつぶし、失敗時は `data-theme` 未設定のまま
/// （`--fandhe-*` テーマトークンの `@media (prefers-color-scheme: dark)`
/// 経路、`crates/pre-styled-ui/src/theme.rs` の `Theme::to_css`）へ退避する。
///
/// 責務はここまで（属性設定のみ）。ボタンのイベント配線・ラベル更新は
/// すべて [`SITE_JS`] 側が担う（`site.js` の読み込み失敗時にもこのスニペット
/// だけは動作し、保存済みテーマの反映は維持される）。
pub const INLINE_THEME_BOOTSTRAP: &str = "try{var t=localStorage.getItem(`fandhe-docs-theme`);if(t===`dark`||t===`light`){document.documentElement.setAttribute(`data-theme`,t);}}catch(e){}";

/// [`SCRIPT_REL_PATH`] へ書き出す `assets/site.js` の全量。
///
/// 責務:
///
/// 1. `.docs-theme-toggle` ボタンを取得する（無ければ即 return。
///    docs-site 以外のページ・将来の骨格変更で要素が消えても例外を
///    投げない防御的実装）。
/// 2. 実効テーマを解決する: `<html data-theme>` 属性値
///    （`dark`/`light` のみ採用） → 無ければ
///    `matchMedia("(prefers-color-scheme: dark)")`。
/// 3. ボタンのラベル・`aria-pressed` を実効テーマに合わせて初期化する
///    （この時点では `data-theme` を書き込まない。利用者が未選択なら
///    OS 設定追従のままにする）。
/// 4. `click` で実効テーマの反対側へ切替 → `localStorage` へ保存
///    （例外は握りつぶす） → `data-theme` 属性を更新 → ラベル更新。
/// 5. **すべての配線が完了した後にのみ** `hidden` 属性を解除する。
///    `hidden` の除去を `<head>` のインラインスニペットや CSS 側で行うと、
///    `site.js` の読み込み失敗（ネットワーク断・将来 CSP 等）時に
///    「押しても何も起きないボタン」が残ってしまう。JS 無効時だけでなく
///    JS が届かなかった場合の受け入れ条件（「非表示 + OS 設定追従」）を
///    満たすため、可視化は配線完了後に限定する（レビューで安易に
///    単純化しないこと）。
/// 6. `document.readyState === "loading"` なら `DOMContentLoaded` を待ち、
///    そうでなければ即時実行する。
///
/// 7. （イシュー #950、独立した 2 つ目の IIFE）`.docs-toc` が無い・
///    `IntersectionObserver` 非対応のいずれかなら即 return する
///    （progressive enhancement。目次はハイライトが無くてもリンクとして
///    機能する）。`.docs-toc a` を列挙し、`href` の属性値
///    （`getAttribute`。`link.hash` は日本語 id をパーセントエンコードして
///    返すため使わない）を `decodeURIComponent` してから
///    `document.getElementById` で対応見出しを引く（`querySelector('#'+id)`
///    は使わない。著者由来の id をセレクタとして組み立てるとセレクタ
///    インジェクション経路になり得るため、OWASP A03 対策として避ける）。
/// 8. `IntersectionObserver` で可視見出し集合を維持し、可視集合が空で
///    なければ文書順で最初の可視見出し、空ならヘッダー下端を過ぎ去った
///    見出しのうち文書順で最後のものを現在地とし、対応リンクにのみ
///    `aria-current="location"` を付与する（サイドバーの
///    `aria-current="page"` とは値を分け、意味の衝突を避ける）。
///
/// 9. （イシュー #958、独立した 3 つ目の IIFE）`.docs-search-input`・
///    `.docs-search`・`#docs-search-results` のいずれか欠ければ即 return
///    する（他 2 つの IIFE 同様、要素が消えても例外を投げない）。
///    `window.fetch` 非対応・`data-search-index` 属性が空の場合も同様に
///    即 return する。
/// 10. インデックス JSON（`crate::search_index::REL_PATH`）は初回 `focus`
///     イベントでのみ `fetch()` する（single-flight。状態は
///     `idle`/`loading`/`ready`/`failed` の 4 値で管理し、再フェッチしない。
///     `failed` の場合のみ再フォーカス時に再試行を許す）。
///     `response.ok`・`data.version !== 1`（[`crate::search_index::SCHEMA_VERSION`]
///     と数値リテラルで一致させる契約。ドリフト検知は本モジュールの
///     `tests::site_js_pins_the_same_schema_version_as_search_index_rs`
///     参照）・`Array.isArray(data.pages)` のいずれかを満たさない応答は
///     `failed` として扱う（fail-closed。壊れた検索結果を表示しない）。
/// 11. `input` イベントで部分一致検索を実行する。正規化は
///     `toLowerCase()`/`trim()` のみ。マッチは `indexOf(query) !== -1` の
///     部分一致で、ページタイトル一致を最優先（スコア 3）・見出しタイトル
///     一致（スコア 2）・本文一致（スコア 1）の順で評価し、0 件（無関係）は
///     除外する。スコア降順・同点はインデックス順で並べ、上位 10 件のみ
///     `document.createElement`/`textContent`/`setAttribute` のみで描画する
///     （`innerHTML` は使わない）。結果 `href`（`page.href` または
///     `page.href + "#" + encodeURIComponent(section.id)`）は `/` で始まり
///     `//` で始まらない場合のみ採用し、満たさない項目は描画自体をしない
///     （OWASP A01/A03、`javascript:` 等のスキーム URL を構造的に排除する）。
/// 12. キーボード操作: `document` 上の `/`（フォーム要素・`contentEditable`
///     以外にフォーカスがある場合のみ）で検索欄へフォーカス、`input` 上の
///     `ArrowDown`/`ArrowUp` で選択移動（端で停止、循環しない）、`Enter` で
///     選択項目のアンカーを `click()`（`location.href` への代入はしない）、
///     `Escape` で結果を閉じて入力をクリアする。選択位置は
///     `aria-activedescendant`（未選択時は除去）で表す。
///
/// 文字列リテラルはすべてバッククォート（テンプレートリテラル。補間は
/// 使わない）を使い、`&&` の代わりに `||` を使うことでエスケープ対象文字
/// （`< > & " '`）を含まない（[`is_escape_safe`] 参照）。`innerHTML` /
/// `document.write` / `eval` / `new Function` は使わない
/// （DOM 操作は `setAttribute`/`removeAttribute`/`textContent`/
/// `addEventListener` に限定する）。
pub const SITE_JS: &str = "\
(function () {
  var STORAGE_KEY = `fandhe-docs-theme`;
  var toggle = document.querySelector(`.docs-theme-toggle`);
  if (!toggle) {
    return;
  }

  function effectiveTheme() {
    var attr = document.documentElement.getAttribute(`data-theme`);
    if (attr === `dark` || attr === `light`) {
      return attr;
    }
    var prefersDark = false;
    if (window.matchMedia) {
      prefersDark = window.matchMedia(`(prefers-color-scheme: dark)`).matches;
    }
    return prefersDark ? `dark` : `light`;
  }

  function applyLabel(theme) {
    toggle.setAttribute(`aria-pressed`, theme === `dark` ? `true` : `false`);
    toggle.textContent = theme === `dark` ? `Light` : `Dark`;
  }

  function storeTheme(theme) {
    try {
      window.localStorage.setItem(STORAGE_KEY, theme);
    } catch (err) {
      // localStorage が使えない環境（Safari プライベートブラウズ等）では
      // 保存をあきらめ、今回の切替自体は続行する。
    }
  }

  function init() {
    applyLabel(effectiveTheme());
    toggle.addEventListener(`click`, function () {
      var next = effectiveTheme() === `dark` ? `light` : `dark`;
      storeTheme(next);
      document.documentElement.setAttribute(`data-theme`, next);
      applyLabel(next);
    });
    // 配線がすべて完了した後にのみ可視化する（上記 doc コメント手順 5）。
    toggle.removeAttribute(`hidden`);
  }

  if (document.readyState === `loading`) {
    document.addEventListener(`DOMContentLoaded`, init);
  } else {
    init();
  }
})();

(function () {
  var toc = document.querySelector(`.docs-toc`);
  if (!toc) {
    return;
  }
  if (!window.IntersectionObserver) {
    return;
  }

  // href は日本語 id を含み得る。DOM プロパティ経由（ハッシュ由来の値）
  // だとブラウザがパーセントエンコードした値を返し getElementById が
  // 一致しない。属性値そのもの（getAttribute）を decodeURIComponent
  // してから引く。
  var links = [];
  var targets = [];
  var anchors = toc.querySelectorAll(`a`);
  anchors.forEach(function (anchor) {
    var href = anchor.getAttribute(`href`);
    if (!href) {
      return;
    }
    if (href.charAt(0) !== `#`) {
      return;
    }
    var target = document.getElementById(decodeURIComponent(href.slice(1)));
    if (!target) {
      return;
    }
    links.push(anchor);
    targets.push(target);
  });

  if (targets.length === 0) {
    return;
  }

  // ヘッダー下端の判定閾値。IntersectionObserver の rootMargin 上端
  // オフセットと lastPassedTarget のフォールバック判定は、概念上
  // 同じ「ヘッダー下端を過ぎたか」を表すため単一の定数に統一する
  // （2 箇所に分散させるとヘッダー高さ変更時に片方だけ更新され
  // ハイライト境界がずれるリスクがあるため）。
  var HEADER_OFFSET_PX = 64;

  var visible = [];

  function clearCurrent() {
    links.forEach(function (link) {
      link.removeAttribute(`aria-current`);
    });
  }

  function markCurrent(target) {
    clearCurrent();
    var index = targets.indexOf(target);
    if (index === -1) {
      return;
    }
    links[index].setAttribute(`aria-current`, `location`);
  }

  // 可視集合が空でなければ文書順で最初の可視見出しを採用する。
  function firstVisibleInDocumentOrder() {
    var found = null;
    targets.forEach(function (target) {
      if (found) {
        return;
      }
      if (visible.indexOf(target) !== -1) {
        found = target;
      }
    });
    return found;
  }

  // 可視集合が空のとき（本文が長く見出し同士が離れている場合）の
  // フォールバック: ヘッダー下端を過ぎ去った見出しのうち文書順で
  // 最後のものを現在地とする。`getBoundingClientRect` はこの分岐でのみ
  // 評価し、scroll イベントリスナは張らない（レイアウトスラッシング回避）。
  function lastPassedTarget() {
    var found = null;
    targets.forEach(function (target) {
      var rect = target.getBoundingClientRect();
      if (Math.sign(rect.top - HEADER_OFFSET_PX) !== 1) {
        found = target;
      }
    });
    return found;
  }

  function update() {
    var current = firstVisibleInDocumentOrder();
    if (!current) {
      current = lastPassedTarget();
    }
    if (current) {
      markCurrent(current);
    } else {
      clearCurrent();
    }
  }

  var observer = new IntersectionObserver(function (entries) {
    entries.forEach(function (entry) {
      var idx = visible.indexOf(entry.target);
      if (entry.isIntersecting) {
        if (idx === -1) {
          visible.push(entry.target);
        }
      } else {
        if (idx !== -1) {
          visible.splice(idx, 1);
        }
      }
    });
    update();
  }, { rootMargin: `-` + HEADER_OFFSET_PX + `px 0px -60% 0px` });

  targets.forEach(function (target) {
    observer.observe(target);
  });
})();

(function () {
  var input = document.querySelector(`.docs-search-input`);
  if (!input) {
    return;
  }
  var box = document.querySelector(`.docs-search`);
  if (!box) {
    return;
  }
  var list = document.getElementById(`docs-search-results`);
  if (!list) {
    return;
  }
  if (!window.fetch) {
    return;
  }
  var indexUrl = input.getAttribute(`data-search-index`);
  if (!indexUrl) {
    return;
  }

  var state = `idle`;
  var indexData = null;
  var selectedIndex = -1;
  var currentResults = [];

  function setExpanded(expanded) {
    input.setAttribute(`aria-expanded`, expanded ? `true` : `false`);
  }

  function closeResults() {
    list.setAttribute(`hidden`, ``);
    setExpanded(false);
    selectedIndex = -1;
    input.removeAttribute(`aria-activedescendant`);
  }

  function openResults() {
    list.removeAttribute(`hidden`);
    setExpanded(true);
  }

  function clearResults() {
    while (list.firstChild) {
      list.removeChild(list.firstChild);
    }
  }

  // 著者由来ではなくビルド生成インデックス由来の href でも、多層防御として
  // 相対パス（先頭が 1 個の /）以外は構造的に描画しない（OWASP A01/A03）。
  function isSafePath(href) {
    if (href.charAt(0) !== `/`) {
      return false;
    }
    if (href.charAt(1) === `/`) {
      return false;
    }
    return true;
  }

  function renderEmpty(message) {
    clearResults();
    var item = document.createElement(`li`);
    item.className = `docs-search-empty`;
    item.textContent = message;
    list.appendChild(item);
    currentResults = [];
    selectedIndex = -1;
    openResults();
  }

  function renderResults(matches) {
    clearResults();
    var safeMatches = [];
    matches.forEach(function (match) {
      var href = match.page.href;
      if (match.section) {
        href = match.page.href + `#` + encodeURIComponent(match.section.id);
      }
      if (!isSafePath(href)) {
        return;
      }
      safeMatches.push({ page: match.page, section: match.section, href: href });
    });
    currentResults = safeMatches;
    selectedIndex = -1;
    if (safeMatches.length === 0) {
      renderEmpty(`No results`);
      return;
    }
    safeMatches.forEach(function (match, i) {
      var item = document.createElement(`li`);
      item.className = `docs-search-result`;
      item.setAttribute(`role`, `option`);
      item.id = `docs-search-result-` + i;
      item.setAttribute(`aria-selected`, `false`);

      var anchor = document.createElement(`a`);
      anchor.setAttribute(`href`, match.href);

      var title = document.createElement(`span`);
      title.className = `docs-search-result-title`;
      title.textContent = match.page.title;
      anchor.appendChild(title);

      if (match.section) {
        var section = document.createElement(`span`);
        section.className = `docs-search-result-section`;
        section.textContent = match.section.title;
        anchor.appendChild(section);
      }

      item.appendChild(anchor);
      list.appendChild(item);
    });
    openResults();
  }

  function updateSelection() {
    for (var i = 0; i !== currentResults.length; i++) {
      var item = document.getElementById(`docs-search-result-` + i);
      if (!item) {
        continue;
      }
      if (i === selectedIndex) {
        item.setAttribute(`aria-selected`, `true`);
      } else {
        item.setAttribute(`aria-selected`, `false`);
      }
    }
    if (selectedIndex === -1) {
      input.removeAttribute(`aria-activedescendant`);
    } else {
      input.setAttribute(`aria-activedescendant`, `docs-search-result-` + selectedIndex);
    }
  }

  function moveSelection(delta) {
    if (currentResults.length === 0) {
      return;
    }
    var lastIndex = currentResults.length - 1;
    var next = Math.min(lastIndex, Math.max(0, selectedIndex + delta));
    selectedIndex = next;
    updateSelection();
  }

  function scorePage(page, query) {
    var titleLower = page.title.toLowerCase();
    if (titleLower.indexOf(query) !== -1) {
      return { score: 3, section: null };
    }
    var matchedSection = null;
    page.sections.forEach(function (section) {
      if (matchedSection) {
        return;
      }
      var sectionLower = section.title.toLowerCase();
      if (sectionLower.indexOf(query) !== -1) {
        matchedSection = section;
      }
    });
    if (matchedSection) {
      return { score: 2, section: matchedSection };
    }
    var textLower = page.text.toLowerCase();
    if (textLower.indexOf(query) !== -1) {
      return { score: 1, section: null };
    }
    return { score: 0, section: null };
  }

  function runSearch() {
    var query = input.value.toLowerCase();
    query = query.trim();
    if (query === ``) {
      closeResults();
      clearResults();
      return;
    }
    if (state === `loading`) {
      return;
    }
    if (state === `failed`) {
      renderEmpty(`Search is unavailable`);
      return;
    }
    if (state !== `ready`) {
      return;
    }
    var matches = [];
    indexData.pages.forEach(function (page, i) {
      var result = scorePage(page, query);
      if (result.score === 0) {
        return;
      }
      matches.push({ page: page, section: result.section, score: result.score, order: i });
    });
    matches.sort(function (a, b) {
      if (a.score !== b.score) {
        return b.score - a.score;
      }
      return a.order - b.order;
    });
    renderResults(matches.slice(0, 10));
  }

  function ensureIndexLoaded() {
    if (state === `loading`) {
      return;
    }
    if (state === `ready`) {
      return;
    }
    state = `loading`;
    fetch(indexUrl).then(function (response) {
      if (!response.ok) {
        throw new Error(`bad response`);
      }
      return response.json();
    }).then(function (data) {
      if (data.version !== 1) {
        throw new Error(`bad version`);
      }
      if (!Array.isArray(data.pages)) {
        throw new Error(`bad pages`);
      }
      indexData = data;
      state = `ready`;
      runSearch();
    }).catch(function () {
      state = `failed`;
      runSearch();
    });
  }

  function init() {
    input.addEventListener(`focus`, function () {
      if (state === `idle`) {
        ensureIndexLoaded();
        return;
      }
      if (state === `failed`) {
        ensureIndexLoaded();
      }
    });

    input.addEventListener(`input`, runSearch);

    input.addEventListener(`keydown`, function (event) {
      if (event.key === `ArrowDown`) {
        event.preventDefault();
        moveSelection(1);
        return;
      }
      if (event.key === `ArrowUp`) {
        event.preventDefault();
        moveSelection(-1);
        return;
      }
      if (event.key === `Enter`) {
        if (selectedIndex === -1) {
          return;
        }
        event.preventDefault();
        var item = document.getElementById(`docs-search-result-` + selectedIndex);
        if (!item) {
          return;
        }
        var anchor = item.querySelector(`a`);
        if (!anchor) {
          return;
        }
        anchor.click();
        return;
      }
      if (event.key === `Escape`) {
        closeResults();
        clearResults();
        input.value = ``;
      }
    });

    document.addEventListener(`keydown`, function (event) {
      if (event.key !== `/`) {
        return;
      }
      var target = event.target;
      if (target) {
        var tag = target.tagName;
        if (tag === `INPUT`) {
          return;
        }
        if (tag === `TEXTAREA`) {
          return;
        }
        if (tag === `SELECT`) {
          return;
        }
        if (target.isContentEditable) {
          return;
        }
      }
      event.preventDefault();
      input.focus();
    });

    // 配線がすべて完了した後にのみ可視化する（テーマトグルと同じ契約、
    // モジュール doc 手順 5・9 参照）。
    box.removeAttribute(`hidden`);
  }

  if (document.readyState === `loading`) {
    document.addEventListener(`DOMContentLoaded`, init);
  } else {
    init();
  }
})();
";

/// `source` が HTML エスケープ対象文字（`< > & " '`）を 1 文字も含まず、
/// かつテンプレートリテラル補間（`${`）を含まないかを判定する純関数。
///
/// [`fandhe_frontend_core::escape_html_into`] の変換対象文字と完全一致させる
/// ことで、`<script>` の中身（HTML パーサが実体参照を復号しない raw text）に
/// 埋め込んでも構文が壊れないことを保証する。`${` の禁止は、将来
/// 変数補間を追加しようとした際にこのテストが機械的に検知するための
/// 構造的な防御である（変数補間は非信頼データを script コンテキストへ
/// 注入する経路になり得るため、docs-site では導入しない方針）。
pub fn is_escape_safe(source: &str) -> bool {
    !source
        .chars()
        .any(|c| matches!(c, '<' | '>' | '&' | '"' | '\''))
        && !source.contains("${")
}

/// [`INLINE_THEME_BOOTSTRAP`] が [`is_escape_safe`] を満たす場合のみ
/// `Some` を返す fail-closed のアクセサ。
///
/// `crate::layout::docs_page_with_assets` はこの関数が `None` を返した場合
/// `<script>` 自体を出力しない（壊れた JS を配信するくらいなら
/// `prefers-color-scheme` 追従へ退避する。
/// `fandhe_frontend_pre_styled_ui::StyleSheet` の検証済み CSS のみを
/// 保持する方針と同型の最終防壁）。
pub fn inline_theme_bootstrap() -> Option<&'static str> {
    if is_escape_safe(INLINE_THEME_BOOTSTRAP) {
        Some(INLINE_THEME_BOOTSTRAP)
    } else {
        None
    }
}

/// [`SITE_JS`] を返す。`crate::build::build_site` が
/// [`SCRIPT_REL_PATH`] へそのまま書き出す。
pub fn site_js() -> &'static str {
    SITE_JS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inline_theme_bootstrap_is_escape_safe() {
        assert!(is_escape_safe(INLINE_THEME_BOOTSTRAP));
    }

    #[test]
    fn site_js_is_escape_safe() {
        assert!(is_escape_safe(SITE_JS));
    }

    #[test]
    fn inline_theme_bootstrap_accessor_returns_some_for_the_safe_constant() {
        assert_eq!(inline_theme_bootstrap(), Some(INLINE_THEME_BOOTSTRAP));
    }

    #[test]
    fn is_escape_safe_rejects_html_escape_target_characters() {
        assert!(!is_escape_safe("a<b"));
        assert!(!is_escape_safe("a>b"));
        assert!(!is_escape_safe("a&b"));
        assert!(!is_escape_safe("a\"b"));
        assert!(!is_escape_safe("a'b"));
    }

    #[test]
    fn is_escape_safe_rejects_template_literal_interpolation() {
        assert!(!is_escape_safe("var x = `${y}`;"));
    }

    #[test]
    fn is_escape_safe_accepts_plain_js_without_quotes_or_interpolation() {
        assert!(is_escape_safe(
            "(function () { var x = `plain`; return x; })();"
        ));
    }

    /// キー名の二重管理ドリフト検知: [`INLINE_THEME_BOOTSTRAP`] と
    /// [`SITE_JS`] の双方が [`THEME_STORAGE_KEY`] と同じ文字列を参照する
    /// ことを固定する（片方だけキー名を変更してリロード後の復元が壊れる
    /// 事故を防ぐ）。
    #[test]
    fn script_js_and_inline_bootstrap_share_the_same_storage_key() {
        assert!(INLINE_THEME_BOOTSTRAP.contains(THEME_STORAGE_KEY));
        assert!(SITE_JS.contains(THEME_STORAGE_KEY));
    }

    /// `localStorage` アクセスの例外握りつぶし（try/catch）が消えていない
    /// ことを固定する。Safari プライベートブラウズ等での例外時に
    /// スクリプト全体が停止し、ナビゲーション等の既存機能まで壊れる
    /// 回帰を防ぐ回帰テスト。
    #[test]
    fn inline_theme_bootstrap_swallows_localstorage_exceptions() {
        assert!(INLINE_THEME_BOOTSTRAP.contains("try{"));
        assert!(INLINE_THEME_BOOTSTRAP.contains("catch"));
    }

    #[test]
    fn site_js_swallows_localstorage_exceptions() {
        assert!(SITE_JS.contains("try {"));
        assert!(SITE_JS.contains("catch"));
    }

    /// [`SITE_JS`] は `hidden` の解除をイベント配線完了後にのみ行う
    /// （上記 doc コメント手順 5）。`removeAttribute` 呼び出しが `init`
    /// 関数の最後（`addEventListener` の後）に位置することを、文字列上の
    /// 出現順で固定する。
    #[test]
    fn site_js_reveals_toggle_only_after_click_handler_is_wired() {
        let listener_pos = SITE_JS
            .find("addEventListener")
            .expect("SITE_JS should wire a click handler");
        let reveal_pos = SITE_JS
            .find("removeAttribute(`hidden`)")
            .expect("SITE_JS should reveal the toggle by removing the hidden attribute");
        assert!(
            listener_pos < reveal_pos,
            "hidden の解除はイベント配線より後である必要がある"
        );
    }

    /// [`SITE_JS`] は危険な DOM 操作 API（`innerHTML`/`insertAdjacentHTML`/
    /// `document.write`/`eval`/`new Function`）を使わない（OWASP A03、
    /// モジュール doc 参照。検索 UI 追加（イシュー #958）に伴う多層防御として
    /// `insertAdjacentHTML` を追加）。
    #[test]
    fn site_js_does_not_use_dangerous_dom_apis() {
        for needle in [
            "innerHTML",
            "insertAdjacentHTML",
            "document.write",
            "eval(",
            "new Function",
        ] {
            assert!(!SITE_JS.contains(needle), "SITE_JS should not use {needle}");
        }
    }

    /// 右目次のスクロールスパイ（イシュー #950）が `IntersectionObserver`・
    /// `.docs-toc`・`aria-current`・`location` を配線していることを固定する。
    #[test]
    fn site_js_wires_toc_scrollspy_with_intersection_observer() {
        for needle in [
            "IntersectionObserver",
            ".docs-toc",
            "aria-current",
            "location",
        ] {
            assert!(SITE_JS.contains(needle), "SITE_JS should wire {needle}");
        }
    }

    /// 目次の対応見出しは `getElementById` + `decodeURIComponent` で解決する
    /// （モジュール doc 手順 7 参照）。`querySelector('#'` によるセレクタ
    /// 組み立て（セレクタインジェクション経路）と `link.hash`（日本語 id を
    /// パーセントエンコードして返しマッチしない罠）のいずれも使わないことを
    /// 固定する。
    #[test]
    fn site_js_resolves_toc_targets_by_get_element_by_id() {
        assert!(SITE_JS.contains("getElementById"));
        assert!(SITE_JS.contains("decodeURIComponent"));
        assert!(!SITE_JS.contains("querySelector(`#"));
        assert!(!SITE_JS.contains(".hash"));
    }

    /// スクロールスパイの IIFE がテーマトグルの早期 return に巻き込まれない
    /// よう独立していることを固定する（モジュール doc 参照。同じ IIFE 内に
    /// 追記すると `.docs-theme-toggle` が無い構成で目次ハイライトごと死ぬ）。
    #[test]
    fn site_js_scrollspy_is_isolated_from_the_theme_toggle_guard() {
        let iife_terminators = SITE_JS.matches("})();").count();
        assert!(
            iife_terminators >= 3,
            "SITE_JS should contain at least three independent IIFEs (found {iife_terminators})"
        );
    }

    /// 検索 IIFE（イシュー #958）のイベント配線がすべて完了した後にのみ
    /// `hidden` を除去する契約を、`.docs-search-input` 以降の切り出しスライス
    /// で固定する（テーマトグルの `site_js_reveals_toggle_only_after_click_handler_is_wired`
    /// と同型。全文検索だと第 1 IIFE の `removeAttribute(\`hidden\`)` を
    /// 誤って拾ってしまうため、検索 IIFE の開始位置以降に限定する）。
    #[test]
    fn site_js_reveals_search_box_only_after_listeners_are_wired() {
        let search_start = SITE_JS
            .find(".docs-search-input")
            .expect("SITE_JS should reference .docs-search-input");
        let search_slice = &SITE_JS[search_start..];
        let listener_pos = search_slice
            .find("addEventListener")
            .expect("search IIFE should wire event listeners");
        let reveal_pos = search_slice
            .find("box.removeAttribute(`hidden`)")
            .expect("search IIFE should reveal the search box by removing the hidden attribute");
        assert!(
            listener_pos < reveal_pos,
            "検索ボックスの hidden 解除はイベント配線より後である必要がある"
        );
    }

    /// 検索 UI（イシュー #958）の必須配線が消えていないことを固定する。
    #[test]
    fn site_js_wires_search_ui() {
        for needle in [
            "data-search-index",
            ".docs-search-input",
            "docs-search-results",
            "fetch",
            "createElement",
            "textContent",
            "encodeURIComponent",
            "aria-activedescendant",
        ] {
            assert!(SITE_JS.contains(needle), "SITE_JS should wire {needle}");
        }
    }

    /// スキーマバージョンの二重管理ドリフト検知: [`crate::search_index::SCHEMA_VERSION`]
    /// と [`SITE_JS`] 側の `version !== <N>` fail-closed チェックが同じ数値を
    /// 参照することを固定する（設計文書 §3-1/§4-5、片側だけ更新されて
    /// スキーマ検証が無効化される事故を防ぐ）。
    #[test]
    fn site_js_pins_the_same_schema_version_as_search_index_rs() {
        let needle = format!("version !== {}", crate::search_index::SCHEMA_VERSION);
        assert!(
            SITE_JS.contains(&needle),
            "SITE_JS should contain {needle:?}"
        );
    }

    /// [`SITE_JS`] は `location.href` への文字列代入によるナビゲーションを
    /// 行わない（`Enter` キー選択は `anchor.click()` を使う、モジュール doc
    /// 手順 12 参照）。
    #[test]
    fn site_js_does_not_assign_location_href() {
        assert!(!SITE_JS.contains("location.href ="));
    }
}
