//! docs サイトが出力する外部 JS 2 本（`assets/theme-init.js` と `assets/site.js`、イシュー #951/#3676）。
//!
//! # 役割・呼び出し文脈
//!
//! docs サイト（`crate::layout`）は #951 以前は JS を 1 バイトも出力して
//! いなかった（`crate::layout` モジュール doc の旧宣言参照）。本モジュールは
//! テーマトグル（ダーク/ライト切替）の実装として、初めて docs サイトへ
//! クライアント側 JS を持ち込む。
//!
//! - [`THEME_INIT_JS`]: `crate::build::build_site` が [`THEME_INIT_REL_PATH`]
//!   へ書き出し、`crate::layout::docs_page_with_assets` が `<head>` 内・
//!   全スタイルシートより前の**同期**（`defer`/`async` なし）`<script src>` で
//!   読む FOUC 抑止スクリプト（イシュー #3676。`script-src 'self'` の CSP 下で
//!   実行できないインライン `<script>` の外部化。同期 script はパーサを
//!   ブロックするため初回ペイント前に `data-theme` が確定する）。`localStorage` に保存済みのテーマがあれば
//!   CSS 適用前に `<html data-theme="...">` を確定させる。
//! - [`SITE_JS`]: `crate::build::build_site` が [`SCRIPT_REL_PATH`]
//!   （`out_dir` 起点）へ書き出す本体。`.docs-theme-toggle` ボタンの
//!   ラベル・`aria-pressed` 更新、クリック時の切替・保存、および
//!   `hidden` 属性の解除（配線完了後にのみ可視化する）を担う。加えて
//!   イシュー #950 で右目次（`crate::layout::toc_nav` が出力する
//!   `nav.docs-toc`）のスクロールスパイ（現在地ハイライト）を担う。加えて
//!   イシュー #958 で検索 UI（`crate::layout::docs_page_with_assets` が
//!   出力する `div.docs-search`）を担う: 初回フォーカス時に
//!   `crate::search_index` が生成する `assets/search-index.json`
//!   （マニフェスト）とセクション別 `assets/search-index/<slug>.json` を遅延
//!   `fetch()` し、部分一致検索・結果一覧描画・キーボード操作
//!   （`/`・矢印キー・`Enter`・`Escape`）を配線する。
//!   テーマトグルの IIFE は `.docs-theme-toggle` が無いページで早期
//!   return するため、目次ハイライト・検索はそれぞれ**別の独立した IIFE**
//!   として実装し、テーマトグルの早期 return に巻き込まれないようにする
//!   （テーマトグル・検索のいずれかが無い構成でも残りの機能が動作する
//!   必要があるため）。
//!   加えてイシュー #3605 で、フェンスコードのコピーボタン
//!   （`crate::code_copy` が `hidden` 付きで出力）を配線する 4 つ目の IIFE を持つ。
//!
//! # セキュリティ不変条件（REQ-1、`.claude/rules/coding-rust.md`）
//!
//! #3676 以降、JS はどちらも外部ファイルとして書き出す（`<script>` へ本文を
//! 埋め込まない）ため、「raw text へ埋め込むと構文が壊れる」という従来の
//! 理由は消えた。それでも [`is_escape_safe`]（`< > & " '` と `${` の禁止）は
//! (1) 将来の変数補間の混入を防ぐ構造的防御、(2) 外部 JS 全体へ一律に課す
//! 基準として残し、`crate::build::build_site` が書き出し前に
//! [`theme_init_js`] で fail-closed に検証する。[`THEME_INIT_JS`] は
//! 文字列リテラルにバッククォート（テンプレートリテラル）のみを使い、
//! `&&` の代わりに `||` を使うことでこれらの文字を一切含まない。
//! [`is_escape_safe`] がこの性質をコンパイル後にも機械検証し、
//! [`theme_init_js`] は検証に落ちた場合 `None` を返す
//! fail-closed のアクセサとする（`raw_html()` は新規に導入しない）。
//!
//! `${`（テンプレートリテラル補間）も [`is_escape_safe`] の対象外文字列
//! として禁止する。本モジュールの定数はすべて `&'static str` で外部入力・
//! `nav.toml` 由来の値を一切含まないが、将来の変数補間の混入を
//! テストで機械的にブロックする構造的な防御である。
//!
//! `localStorage` はスクリプトの実行主体（同一オリジンの他スクリプト・
//! 利用者自身）が改変できる非信頼データのため、[`THEME_INIT_JS`]・
//! [`SITE_JS`] のいずれも読み出した値を `dark`/`light` の allowlist と
//! 一致した場合のみ `data-theme` へ反映する。

/// [`THEME_INIT_JS`] の出力先（`out_dir` 起点の相対パス）。
/// `crate::build::build_site` が書き出し、`crate::layout::docs_page_with_assets` が
/// 同期の `<script src>` で参照する単一実装点。
pub const THEME_INIT_REL_PATH: &str = "assets/theme-init.js";

/// [`SITE_JS`] の出力先（`out_dir` 起点の相対パス）。
/// `crate::build::build_site` が本パスへ書き出し、
/// `crate::layout::docs_page_with_assets` が `<script src>`（`defer`）で参照する
/// 単一実装点。
pub const SCRIPT_REL_PATH: &str = "assets/site.js";

/// テーマ選択を保存する `localStorage` キー。[`THEME_INIT_JS`] と
/// [`SITE_JS`] の双方が同じキーを参照する契約であることを
/// `site_js_and_theme_init_share_the_same_storage_key`
/// （本モジュールの `tests`）が固定する（キー名の二重管理ドリフト検知）。
pub const THEME_STORAGE_KEY: &str = "fandhe-docs-theme";

/// [`THEME_INIT_REL_PATH`] へ書き出し、`<head>` のスタイルシートより前で
/// 同期読み込みする FOUC 抑止スクリプト（本文は #3676 以前のインライン版と同一）。`localStorage` から保存済みテーマを読み、
/// `dark`/`light` のいずれかであれば `<html>` の `data-theme` 属性を
/// CSS 適用前に確定させる。`localStorage` アクセス例外（Safari プライベート
/// ブラウズ等）は握りつぶし、失敗時は `data-theme` 未設定のまま
/// （`--fandhe-*` テーマトークンの `@media (prefers-color-scheme: dark)`
/// 経路、`crates/pre-styled-ui/src/theme.rs` の `Theme::to_css`）へ退避する。
///
/// 責務はここまで（属性設定のみ）。ボタンのイベント配線・ラベル更新は
/// すべて [`SITE_JS`] 側が担う（`site.js` の読み込み失敗時にもこのスニペット
/// だけは動作し、保存済みテーマの反映は維持される）。
pub const THEME_INIT_JS: &str = "try{var t=localStorage.getItem(`fandhe-docs-theme`);if(t===`dark`||t===`light`){document.documentElement.setAttribute(`data-theme`,t);}}catch(e){}";

/// [`SCRIPT_REL_PATH`] へ書き出す `assets/site.js` の全量。
///
/// 13 以降（イシュー #3605、独立した 4 つ目の IIFE）: `.docs-code-copy` が
/// 0 件、または `isSecureContext` / `navigator.clipboard.writeText` が使えない
/// 場合は即 return し、ボタンは `hidden` のまま残す。コピー元は同一
/// `.docs-code-block` 内 `pre` の `textContent`（`innerHTML` は読まない）。
/// 状態は `data-copy-state`（`idle`/`copied`/`failed`）、完了通知は
/// `.docs-code-copy-status`（`aria-live`）へ書く。`hidden` はクリック配線の後に解除する。
///
/// 責務:
///
/// 1. `.docs-theme-toggle`（ラッパー span、#3606）を取得し、内側の
///    `button` と `.docs-theme-toggle-label` も取得する（いずれか無ければ
///    即 return し `hidden` のまま残す。docs-site 以外のページ・将来の骨格
///    変更で要素が消えても例外を投げない防御的実装）。ラベル書き換えは
///    `.docs-theme-toggle-label` の `textContent` のみ（ボタン全体を書き換えると
///    アイコン SVG が消えるため）。
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
/// 8. `IntersectionObserver` で可視見出し集合を維持し、現在地は
///    `update()` が次の順で決める。(1) スクロール可能なページの末尾なら
///    文書順で最後の見出し、(2) 各見出しの `scroll-margin-top`（実測）+
///    余裕を読み位置線とし、それを過ぎた見出しのうち文書順で最後のもの
///    （`lastPassedTarget`）、(3) いずれも無ければ可視集合の文書順で最初の
///    見出し。`scroll` / `resize` は rAF で間引いて同じ `update()` を
///    再評価する。対応リンクにのみ `aria-current="location"` を付与する
///    （サイドバーの `aria-current="page"` とは値を分け、意味の衝突を
///    避ける）。
///
/// 9. （イシュー #958/#3672、独立した 3 つ目の IIFE）`.docs-search-input`・
///    `.docs-search`・`#docs-search-results`・`#docs-search-dialog`・検索ボタン
///    （`.docs-search-trigger button`）のいずれか欠ければ即 return する。
///    `dialog.showModal` 非対応・`window.fetch` 非対応・`data-search-index`
///    属性が空の場合も `hidden` のまま即 return する（無 JS と同じ fail-closed）。
///    配線完了時にボタンへ `aria-haspopup="dialog"` と `aria-expanded` を付与し、
///    開閉のたびに `aria-expanded` を `dialog.open` へ同期する（SSR は固定値を出さない）。
/// 10. インデックスはダイアログを開いた時点（`openDialog()`）でのみ `fetch()` する
///     （single-flight。状態は `idle`/`loading`/`ready`/`failed` の 4 値で
///     管理し、再フェッチしない。`failed` の場合のみ再フォーカス時に再試行を
///     許す）。取得はマニフェスト（`crate::search_index::REL_PATH`、
///     `data-search-index` 属性）→ その `sections[].href` が指す全セクション
///     ファイル（`assets/search-index/<slug>.json`、イシュー #3173）の 2 段で、
///     セクションファイルは `Promise.all` で並列 fetch しマニフェスト順に
///     結合する（結合後の `pages` 順 = `nav.toml` 宣言順というタイブレーク
///     契約を保つ）。`response.ok`・`version !== 2`
///     （[`crate::search_index::SCHEMA_VERSION`] と数値リテラルで一致させる
///     契約。ドリフト検知は本モジュールの
///     `tests::site_js_pins_the_same_schema_version_as_search_index_rs`
///     参照）・`Array.isArray(manifest.sections)`・各 `section.href` の
///     `isSafePath`・`Array.isArray(data.pages)` のいずれかを 1 件でも満たさ
///     ない場合は全体を `failed` として扱う（fail-closed。部分的な索引で
///     「見つからない」と誤答せず、壊れた検索結果も表示しない）。
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
/// 12. 開閉操作: 検索ボタンの `click`、または `document` 上の `/`（ダイアログが
///     開いている・Ctrl/Meta/Alt 併用・フォーム要素・`contentEditable` 上では何も
///     しない）で `showModal()` し、入力へフォーカスする。Escape（`input` の
///     keydown で `dialog.close()`）・背景クリック（押下位置と click 対象がともに
///     dialog のときだけ）・結果リンクのクリックで閉じる。`close` イベントが
///     結果掃除・入力クリア・`aria-expanded` 同期・ボタンへのフォーカス復帰の
///     合流点（結果リンクで閉じた場合のみフォーカス復帰を省き、移動先の焦点を
///     奪わない）。`input` 上の `ArrowDown`/`ArrowUp` で選択移動（端で停止、
///     循環しない）、`Enter` で選択項目のアンカーを `click()` する
///     （`location.href` への代入はしない）。選択位置は
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
  var control = toggle.querySelector(`button`);
  var label = toggle.querySelector(`.docs-theme-toggle-label`);
  if (!control || !label) {
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
    control.setAttribute(`aria-pressed`, theme === `dark` ? `true` : `false`);
    label.textContent = theme === `dark` ? `Light` : `Dark`;
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
    control.addEventListener(`click`, function () {
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

  // 見出しを「読んでいる」と見なす位置。`#見出し` 直リンク・目次ジャンプで
  // 止まった見出しは top が各見出しの `scroll-margin-top` に一致する。
  // この値は幅により異なる（広幅はヘッダー高 + 1rem、狭幅は 1rem）ため、
  // 固定値だと狭幅で直後の子 h3 まで通過済みと扱われ、ジャンプ先の親 h2
  // でなく h3 が current になる。そこで見出しごとに実測した
  // `scroll-margin-top` + 余裕を線とし、取得できない場合のみ
  // READING_LINE_PX へフォールバックする。親 h2 の直後に短い h3 が続くと
  // 両者が同時に判定帯へ入り文書順先頭の h2 が current に留まっていた
  // ため、帯内の先頭ではなくこの線を過ぎた最後の見出しを優先する
  // （イシュー #3658）。
  var READING_LINE_PX = HEADER_OFFSET_PX + 32;
  var READING_SLACK_PX = 4;

  function readingLineFor(target) {
    var margin = Number.parseFloat(
      window.getComputedStyle(target).scrollMarginTop,
    );
    if (Number.isNaN(margin)) {
      return READING_LINE_PX;
    }
    return margin + READING_SLACK_PX;
  }

  // ページ末尾到達の判定余裕（サブピクセル誤差吸収）。
  var BOTTOM_SLACK_PX = 2;

  function firstVisibleInDocumentOrder() {
    var found = null;
    targets.forEach(function (target) {
      if (found !== null) {
        return;
      }
      if (visible.indexOf(target) !== -1) {
        found = target;
      }
    });
    return found;
  }

  // 読み位置線を過ぎた見出しのうち文書順で最後のものを返す。
  // 見出しの top は文書順に単調増加するため二分探索し、スクロールごとの
  // getBoundingClientRect() 呼び出しを O(log n) に抑える。評価は update()
  // 経由のみ（IntersectionObserver 通知と rAF 間引きの scroll 時）。
  function lastPassedTarget() {
    var lo = 0;
    var hi = targets.length - 1;
    var found = -1;
    while (Math.sign(hi - lo) !== -1) {
      var mid = Math.floor((lo + hi) / 2);
      var rect = targets[mid].getBoundingClientRect();
      if (Math.sign(rect.top - readingLineFor(targets[mid])) !== 1) {
        found = mid;
        lo = mid + 1;
      } else {
        hi = mid - 1;
      }
    }
    return found === -1 ? null : targets[found];
  }

  // スクロール可能なページで末尾に到達したか。ビューポートに収まる短い
  // ページは常に「残り 0」になるため、スクロール可能量が余裕を超える場合
  // に限って末尾と見なす（初回描画で読み位置線上の見出しを潰さない）。
  function atPageBottom() {
    var doc = document.documentElement;
    var scrollable = doc.scrollHeight - window.innerHeight;
    if (Math.sign(scrollable - BOTTOM_SLACK_PX) !== 1) {
      return false;
    }
    var remaining = scrollable - window.scrollY;
    return Math.sign(remaining - BOTTOM_SLACK_PX) !== 1;
  }

  // 判定順: (1) スクロール可能なページの末尾なら判定帯に依存せず文書順で
  // 最後の見出し（末尾の短い節は帯・読み位置線まで上がりきらないため）、
  // (2) 読み位置線を過ぎた最後の見出し、(3) 先頭付近は帯内の最初の見出し。
  function update() {
    var current = null;
    if (atPageBottom()) {
      current = targets[targets.length - 1];
    } else {
      current = lastPassedTarget();
    }
    if (!current) {
      current = firstVisibleInDocumentOrder();
    }
    if (current) {
      markCurrent(current);
    } else {
      clearCurrent();
    }
  }

  // 末尾到達は帯内集合が変わらずに起こり得るため、rAF で間引いた passive
  // scroll で update() を再評価する。atPageBottom() は window.innerHeight に
  // 依存するため、画面サイズ変更（resize）でも同じ経路で再判定する。
  var scrollQueued = false;
  function queueUpdate() {
    if (scrollQueued) {
      return;
    }
    scrollQueued = true;
    window.requestAnimationFrame(function () {
      scrollQueued = false;
      update();
    });
  }
  window.addEventListener(`scroll`, queueUpdate, { passive: true });
  window.addEventListener(`resize`, queueUpdate, { passive: true });

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
  var dialog = document.getElementById(`docs-search-dialog`);
  if (!dialog) {
    return;
  }
  var trigger = box.querySelector(`.docs-search-trigger button`);
  if (!trigger) {
    return;
  }
  if (typeof dialog.showModal !== `function`) {
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
  var downOnBackdrop = false;
  var skipFocusRestore = false;

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

  // DOM の除去に加えて `currentResults`/`selectedIndex` も必ずリセットする。
  // DOM のみ消して `currentResults` を残すと、Escape・クエリクリア後も
  // 矢印キーで `moveSelection` が古い `currentResults.length` を境界に
  // `selectedIndex` を進めてしまい、`updateSelection` が存在しない
  // `docs-search-result-N` を `aria-activedescendant` にセットする
  // （行が消えているのに読み上げ対象扱いになる）。
  function clearResults() {
    while (list.firstChild) {
      list.removeChild(list.firstChild);
    }
    currentResults = [];
    selectedIndex = -1;
    input.removeAttribute(`aria-activedescendant`);
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

  // 小文字化は索引結合時に 1 回だけ行い（`ensureIndexLoaded` の
  // `prepareLowerCache`）、打鍵ごとの `toLowerCase()` による全 text
  // （約 1.9 MB）の再割り当てを避ける（設計 §10-16、イシュー #3171）。
  // キャッシュ項目は比較専用で DOM へは書き出さない。
  // 加算スコアリング（title +3 / section +2 / text +1、各独立判定）。
  // タイトル一致で早期 return すると見出し一致の加点・ディープリンクが
  // 落ちるため（設計 docs/design/docs-site-search-design.md セクション
  // 4-4）、3 条件すべてを毎回評価してから合算する。section は最初に
  // 一致した見出し（renderResults がリンク先のフラグメント生成に使う）
  // で、title 一致の有無に関わらず独立して求める。
  function scorePage(page, query) {
    var score = 0;
    if (page.titleLower.indexOf(query) !== -1) {
      score += 3;
    }
    var matchedSection = null;
    page.sectionsLower.forEach(function (sectionLower, i) {
      if (matchedSection) {
        return;
      }
      if (sectionLower.indexOf(query) !== -1) {
        matchedSection = page.sections[i];
      }
    });
    if (matchedSection) {
      score += 2;
    }
    if (page.textLower.indexOf(query) !== -1) {
      score += 1;
    }
    return { score: score, section: matchedSection };
  }

  // 検索比較用の小文字化済みコピーを page へ 1 度だけ付与する。
  // 部分一致・nav 宣言順タイブレークの意味は変えない。
  function prepareLowerCache(page) {
    page.titleLower = page.title.toLowerCase();
    page.sectionsLower = page.sections.map(function (section) {
      return section.title.toLowerCase();
    });
    page.textLower = page.text.toLowerCase();
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
    fetchJson(indexUrl).then(function (manifest) {
      if (!Array.isArray(manifest.sections)) {
        throw new Error(`bad sections`);
      }
      // マニフェスト順に全セクションファイルを並列 fetch する（順序は
      // Promise.all が保つため、結合後の pages 順 = nav.toml 宣言順という
      // タイブレーク契約が崩れない）。1 件でも失敗すれば全体を failed に
      // する（部分的な索引で「見つからない」と誤答しない fail-closed）。
      return Promise.all(manifest.sections.map(function (section) {
        if (typeof section.href !== `string` || !isSafePath(section.href)) {
          throw new Error(`bad section href`);
        }
        return fetchJson(section.href).then(function (data) {
          if (!Array.isArray(data.pages)) {
            throw new Error(`bad pages`);
          }
          return data.pages;
        });
      }));
    }).then(function (pageLists) {
      var pages = [];
      pageLists.forEach(function (list) {
        list.forEach(function (page) {
          pages.push(page);
        });
      });
      pages.forEach(prepareLowerCache);
      indexData = { pages: pages };
      state = `ready`;
      runSearch();
    }).catch(function () {
      state = `failed`;
      runSearch();
    });
  }

  // マニフェスト・セクションファイルに共通の取得 + スキーマ検証。
  // `version` は `crate::search_index::SCHEMA_VERSION` と数値リテラルで
  // 一致させる（ドリフト検知は tests::site_js_pins_the_same_schema_version_as_search_index_rs）。
  function fetchJson(url) {
    return fetch(url).then(function (response) {
      if (!response.ok) {
        throw new Error(`bad response`);
      }
      return response.json();
    }).then(function (data) {
      if (!data || data.version !== 2) {
        throw new Error(`bad version`);
      }
      return data;
    });
  }

  function syncExpanded() {
    trigger.setAttribute(`aria-expanded`, dialog.open ? `true` : `false`);
  }

  // 開く操作はボタンと `/` の 2 経路。索引は focus イベント頼みにせず
  // ここで明示的に読み込む（失敗時のみ再試行を許す）。
  function openDialog() {
    if (dialog.open) {
      return;
    }
    dialog.showModal();
    syncExpanded();
    if (state === `idle` || state === `failed`) {
      ensureIndexLoaded();
    }
    input.focus();
  }

  function init() {
    trigger.setAttribute(`aria-haspopup`, `dialog`);
    trigger.setAttribute(`aria-expanded`, `false`);

    trigger.addEventListener(`click`, openDialog);

    // Escape・背景クリック・結果選択・プログラム閉鎖の合流点。掃除とフォーカス復帰を
    // ここへ集約する。
    dialog.addEventListener(`close`, function () {
      closeResults();
      clearResults();
      input.value = ``;
      syncExpanded();
      // 結果リンクで閉じた場合は移動先（#hash の見出し等）の焦点を奪わない。
      if (skipFocusRestore) {
        skipFocusRestore = false;
        return;
      }
      trigger.focus({ preventScroll: true });
    });

    // 入力内ドラッグの終点が背景でも閉じないよう、押下位置も背景のときだけ閉じる。
    dialog.addEventListener(`mousedown`, function (event) {
      downOnBackdrop = event.target === dialog;
    });
    dialog.addEventListener(`click`, function (event) {
      if (event.target === dialog) {
        if (downOnBackdrop) {
          dialog.close();
        }
      }
      downOnBackdrop = false;
    });

    // 同一ページ内 #hash 遷移ではページが破棄されず、モーダルが残るため閉じる。
    list.addEventListener(`click`, function (event) {
      var node = event.target;
      while (node) {
        if (node === list) {
          return;
        }
        if (node.tagName === `A`) {
          skipFocusRestore = true;
          dialog.close();
          return;
        }
        node = node.parentNode;
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
        event.preventDefault();
        dialog.close();
      }
    });

    document.addEventListener(`keydown`, function (event) {
      if (event.key !== `/`) {
        return;
      }
      if (dialog.open) {
        return;
      }
      if (event.ctrlKey || event.metaKey || event.altKey) {
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
      openDialog();
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

// フェンスコードのコピーボタン（イシュー #3605、独立した 4 つ目の IIFE）。
(function () {
  var buttons = document.querySelectorAll(`.docs-code-copy`);
  if (buttons.length === 0) {
    return;
  }
  // Clipboard API が使えない環境ではボタンを hidden のまま残す（fail-closed）。
  if (!window.isSecureContext || !navigator.clipboard || typeof navigator.clipboard.writeText !== `function`) {
    return;
  }

  function wire(button) {
    var block = button.closest(`.docs-code-block`);
    if (!block) {
      return;
    }
    var pre = block.querySelector(`pre`);
    var status = block.querySelector(`.docs-code-copy-status`);
    if (!pre || !status) {
      return;
    }
    var timer = null;

    function setState(state, label, message) {
      button.setAttribute(`data-copy-state`, state);
      button.textContent = label;
      status.textContent = message;
      if (timer !== null) {
        clearTimeout(timer);
      }
      timer = setTimeout(function () {
        button.setAttribute(`data-copy-state`, `idle`);
        button.textContent = `Copy`;
        status.textContent = ``;
        timer = null;
      }, 2000);
    }

    // 例外・Promise 拒否ではボタンを再び hidden にする（設計文書 §5 の契約）。
    // 失敗の通知はステータス領域（aria-live）にのみ残す。
    function fail() {
      if (timer !== null) {
        clearTimeout(timer);
        timer = null;
      }
      button.setAttribute(`data-copy-state`, `failed`);
      button.setAttribute(`hidden`, ``);
      status.textContent = `Copy failed`;
    }

    button.setAttribute(`data-copy-state`, `idle`);
    button.textContent = `Copy`;
    button.addEventListener(`click`, function () {
      var text = pre.textContent;
      try {
        navigator.clipboard.writeText(text).then(function () {
          setState(`copied`, `Copied`, `Copied to clipboard`);
        }, fail);
      } catch (err) {
        fail();
      }
    });

    // 配線がすべて完了した後にのみ可視化する（テーマトグルと同じ契約）。
    button.removeAttribute(`hidden`);
  }

  function init() {
    for (var i = 0; i !== buttons.length; i++) {
      wire(buttons[i]);
    }
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

/// [`THEME_INIT_JS`] が [`is_escape_safe`] を満たす場合のみ
/// `Some` を返す fail-closed のアクセサ。
///
/// `crate::build::build_site` が書き出し前（`ssg::generate_pages` より前）に
/// 呼び、`None` なら `BuildError::UnsafeGeneratedScript` でビルドを止める
/// （`out_dir` を汚さず、ファイルを欠いたまま参照するページを出荷しない）。
pub fn theme_init_js() -> Option<&'static str> {
    if is_escape_safe(THEME_INIT_JS) {
        Some(THEME_INIT_JS)
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
    fn theme_init_js_is_escape_safe() {
        assert!(is_escape_safe(THEME_INIT_JS));
    }

    #[test]
    fn site_js_is_escape_safe() {
        assert!(is_escape_safe(SITE_JS));
    }

    #[test]
    fn theme_init_js_accessor_returns_some_for_the_safe_constant() {
        assert_eq!(theme_init_js(), Some(THEME_INIT_JS));
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

    /// キー名の二重管理ドリフト検知: [`THEME_INIT_JS`] と
    /// [`SITE_JS`] の双方が [`THEME_STORAGE_KEY`] と同じ文字列を参照する
    /// ことを固定する（片方だけキー名を変更してリロード後の復元が壊れる
    /// 事故を防ぐ）。
    #[test]
    fn site_js_and_theme_init_share_the_same_storage_key() {
        assert!(THEME_INIT_JS.contains(THEME_STORAGE_KEY));
        assert!(SITE_JS.contains(THEME_STORAGE_KEY));
    }

    /// `localStorage` アクセスの例外握りつぶし（try/catch）が消えていない
    /// ことを固定する。Safari プライベートブラウズ等での例外時に
    /// スクリプト全体が停止し、ナビゲーション等の既存機能まで壊れる
    /// 回帰を防ぐ回帰テスト。
    #[test]
    fn theme_init_js_swallows_localstorage_exceptions() {
        assert!(THEME_INIT_JS.contains("try{"));
        assert!(THEME_INIT_JS.contains("catch"));
    }

    /// テーマ初期化 JS が DOM への文字列注入 API を使わないことを固定する。
    #[test]
    fn theme_init_js_uses_no_html_injection_apis() {
        for needle in [
            "innerHTML",
            "outerHTML",
            "document.write",
            "eval(",
            "insertAdjacentHTML",
        ] {
            assert!(!THEME_INIT_JS.contains(needle), "{needle}");
        }
    }

    #[test]
    fn site_js_swallows_localstorage_exceptions() {
        assert!(SITE_JS.contains("try {"));
        assert!(SITE_JS.contains("catch"));
    }

    /// テーマトグルのラベル書き換えは `.docs-theme-toggle-label` のみを対象とし
    /// （ボタン全体の `textContent` を書き換えるとアイコン SVG が消える、#3606）、
    /// 内側 `button` の取得が click 配線より前にあることを固定する。
    #[test]
    fn site_js_theme_toggle_rewrites_only_the_label_span() {
        assert!(SITE_JS.contains("label.textContent = theme === `dark`"));
        assert!(!SITE_JS.contains("toggle.textContent"));
        let control_pos = SITE_JS
            .find("toggle.querySelector(`button`)")
            .expect("SITE_JS should look up the inner button");
        let listener_pos = SITE_JS
            .find("control.addEventListener(`click`")
            .expect("SITE_JS should wire click on the inner button");
        assert!(control_pos < listener_pos);
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
            "READING_LINE_PX",
            "lastPassedTarget",
            "passive",
            "`resize`",
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
            iife_terminators >= 4,
            "SITE_JS should contain at least four independent IIFEs (found {iife_terminators})"
        );
    }

    /// コピー IIFE（イシュー #3605）は click 配線の後にのみ `hidden` を除去する。
    #[test]
    fn site_js_reveals_copy_button_only_after_listeners_are_wired() {
        let start = SITE_JS.find("querySelectorAll(`.docs-code-copy`)").unwrap();
        let slice = &SITE_JS[start..];
        let listener = slice.find("addEventListener(`click`").unwrap();
        let reveal = slice.find("removeAttribute(`hidden`)").unwrap();
        assert!(listener < reveal);
    }

    #[test]
    fn site_js_wires_code_copy() {
        use crate::code_copy::{
            CODE_BLOCK_CLASS, COPY_BUTTON_CLASS, COPY_STATE_ATTR, COPY_STATUS_CLASS,
        };
        for token in [
            CODE_BLOCK_CLASS,
            COPY_BUTTON_CLASS,
            COPY_STATE_ATTR,
            COPY_STATUS_CLASS,
            "navigator.clipboard",
            "writeText",
            "isSecureContext",
            "textContent",
        ] {
            assert!(SITE_JS.contains(token), "missing {token}");
        }
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
            "showModal",
            "docs-search-dialog",
            "aria-haspopup",
            "aria-expanded",
            "`close`",
        ] {
            assert!(SITE_JS.contains(needle), "SITE_JS should wire {needle}");
        }
    }

    /// 検索ダイアログ（イシュー #3672）の開閉・フォーカス管理の必須配線を固定する。
    #[test]
    fn site_js_search_dialog_open_close_contract() {
        let start = SITE_JS
            .find(".docs-search-input")
            .expect("search IIFE should exist");
        let search = &SITE_JS[start..];
        // 4 つ目の IIFE（コピーボタン）の配線を含めない。
        let search = &search[..search
            .find("// フェンスコードのコピーボタン")
            .unwrap_or(search.len())];
        // `/` は入力へ直接フォーカスせずダイアログを開く。
        assert!(search.contains("openDialog();"));
        assert!(!search.contains("input.focus();\n    });"));
        // Escape は dialog.close() に集約し、掃除とフォーカス復帰は close ハンドラ側。
        assert!(search.contains("dialog.close();"));
        assert!(search.contains("trigger.focus("));
        // 結果リンクで閉じた場合はフォーカス復帰を省く。
        assert!(search.contains("skipFocusRestore = true;"));
        // 背景クリックは押下位置と click の対象がともに dialog のときだけ閉じる。
        assert!(search.contains("downOnBackdrop = event.target === dialog;"));
        assert!(search.contains("event.target === dialog"));
        // showModal 非対応は hidden のまま即 return（fail-closed）。
        let guard = search
            .find("typeof dialog.showModal !== `function`")
            .expect("showModal capability guard");
        assert!(
            search[guard..].starts_with("typeof dialog.showModal !== `function`) {\n    return;")
        );
        // 開いている間と修飾キー併用では `/` を横取りしない。
        assert!(search.contains("dialog.open"));
        assert!(search.contains("event.ctrlKey"));
        // 結果リンクのクリックでダイアログを閉じる（同一ページ #hash 遷移対策）。
        assert!(search.contains("list.addEventListener(`click`"));
        // aria-haspopup / aria-expanded は JS が付与する。
        assert!(search.contains("trigger.setAttribute(`aria-haspopup`, `dialog`)"));
        // 全 addEventListener の後に可視化する。
        let reveal = search
            .find("box.removeAttribute(`hidden`)")
            .expect("reveal");
        // init() 内の配線（DOMContentLoaded 待ちの外側 addEventListener は対象外）が
        // すべて reveal より前にある。
        let init_end = search.find("if (document.readyState").expect("init end");
        assert!(!search[reveal..init_end].contains("addEventListener"));
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

    /// `scorePage` が title/section/text の 3 条件を独立に加算することを
    /// 固定する（設計 `docs/design/docs-site-search-design.md` §4-4、
    /// 「加算」の明記）。タイトル一致時点で `{ score: 3, section: null }`
    /// を早期 return する退行（Bugbot 指摘）が再導入されると、この
    /// アサーションが検知する。
    #[test]
    fn site_js_score_page_accumulates_title_section_text_scores() {
        for needle in ["score += 3", "score += 2", "score += 1"] {
            assert!(
                SITE_JS.contains(needle),
                "SITE_JS の scorePage は {needle} で加算する必要がある"
            );
        }
        assert!(
            !SITE_JS.contains("return { score: 3"),
            "scorePage がタイトル一致で早期 return すると section 加点・\
             ディープリンクが失われる"
        );
        assert!(
            !SITE_JS.contains("return { score: 2"),
            "scorePage が section 一致で早期 return すると text 加点が失われる"
        );
    }

    /// `scorePage` はタイトル一致時も見出し一致を独立して評価し、
    /// `section` を返す（見出しへのディープリンクのため）。タイトル一致の
    /// `if` ブロックが `section: null` を伴う `return` のままだと、
    /// このアサーションが検知する。
    #[test]
    fn site_js_score_page_keeps_matched_section_independent_of_title_match() {
        let score_page_start = SITE_JS
            .find("function scorePage(page, query)")
            .expect("SITE_JS should define scorePage");
        let score_page_slice = &SITE_JS[score_page_start..];
        let body_end = score_page_slice
            .find("\n  }\n")
            .expect("scorePage should have a closing brace");
        let body = &score_page_slice[..body_end];
        assert!(
            !body.contains("section: null"),
            "scorePage の title 一致分岐が section を null 固定で早期 return している"
        );
        assert!(
            body.contains("return { score: score, section: matchedSection }"),
            "scorePage は最終的に加算済み score と独立評価した section を返す必要がある"
        );
    }

    /// 小文字化は索引結合時に 1 回だけ行い、`scorePage` は打鍵ごとに
    /// `toLowerCase()` を呼ばない（設計 §10-16、イシュー #3171）。
    #[test]
    fn site_js_score_page_uses_precomputed_lowercase_cache() {
        assert!(SITE_JS.contains("pages.forEach(prepareLowerCache)"));
        let start = SITE_JS
            .find("function scorePage(page, query)")
            .expect("SITE_JS should define scorePage");
        let slice = &SITE_JS[start..];
        let end = slice.find("\n  }\n").expect("scorePage closing brace");
        let body = &slice[..end];
        assert!(
            !body.contains("toLowerCase"),
            "scorePage は毎打鍵で小文字化しない"
        );
        for field in ["titleLower", "sectionsLower", "textLower"] {
            assert!(
                body.contains(field),
                "scorePage は {field} を参照する必要がある"
            );
        }
    }

    /// `clearResults` が DOM の除去に加えて `currentResults`/`selectedIndex`/
    /// `aria-activedescendant` をリセットすることを固定する（Bugbot 指摘:
    /// DOM のみ消して state を残すと、Escape・クエリクリア後の矢印キーで
    /// 存在しない結果行を指す `aria-activedescendant` が発生する）。
    #[test]
    fn site_js_clear_results_resets_selection_state() {
        let clear_results_start = SITE_JS
            .find("function clearResults()")
            .expect("SITE_JS should define clearResults");
        let clear_results_slice = &SITE_JS[clear_results_start..];
        let body_end = clear_results_slice
            .find("\n  }\n")
            .expect("clearResults should have a closing brace");
        let body = &clear_results_slice[..body_end];
        for needle in [
            "currentResults = []",
            "selectedIndex = -1",
            "input.removeAttribute(`aria-activedescendant`)",
        ] {
            assert!(
                body.contains(needle),
                "clearResults は {needle} を実行して選択状態をリセットする必要がある"
            );
        }
    }
}
