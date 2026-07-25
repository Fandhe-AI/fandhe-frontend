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
//!   `hidden` 属性の解除（配線完了後にのみ可視化する）を担う。
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

    /// [`SITE_JS`] は危険な DOM 操作 API（`innerHTML`/`document.write`/
    /// `eval`/`new Function`）を使わない（OWASP A03、モジュール doc 参照）。
    #[test]
    fn site_js_does_not_use_dangerous_dom_apis() {
        for needle in ["innerHTML", "document.write", "eval(", "new Function"] {
            assert!(!SITE_JS.contains(needle), "SITE_JS should not use {needle}");
        }
    }
}
