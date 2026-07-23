//! recipe / theme が生成する静的 CSS の配布ヘルパ（イシュー #605、親 #603）。
//!
//! [`crate::recipe::SlotRecipe::css`]・[`crate::theme::Theme::to_css`]・各 styled
//! 部品の `css()`/`stylesheet()` は決定的な CSS 文字列を返すのみで、その先の
//! 配布（静的 `.css` ファイル書き出し・SSR での `<style>` 要素埋め込み）は
//! 呼び出し側任せだった（`crate` 冒頭の不変条件 2 の従来記述、`examples/headless-pre-styled-ui`
//! の手書き `static/ui.css` コピーが実例）。本モジュールはこの 2 経路を提供する。
//!
//! # セキュリティ上の設計（本クレート不変条件 2 の唯一の例外）
//!
//! [`StyleSheet`] は「検証済み CSS のみを保持する型」であり、[`raw_html`] の
//! 使用は [`StyleSheet::style_element`] 内の 1 箇所のみに閉じ込める
//! （呼び出し側へエスケープ迂回経路を公開しない）。防御は多層:
//!
//! 1. 生成側 allowlist — [`crate::css::is_valid_value`]・[`crate::theme::CssValue`]・
//!    [`crate::theme::TokenName`] が `<` を構成不能にしている（各モジュールの
//!    doc 参照）。[`StyleSheet::push_recipe`]/[`StyleSheet::push_theme`] はこの
//!    保証に依拠して infallible にしている。
//! 2. [`StyleSheet::push_css`] の fail-closed 検証 — `<` を 1 文字でも含む、
//!    または改行・タブ・復帰以外の制御文字を含む入力を `Err` にする。これにより
//!    任意文字列の直接持ち込みを遮断する。
//! 3. [`StyleSheet`] の構築経路 — フィールドは private であり、[`StyleSheet::push_css`]
//!    （検証あり）と [`StyleSheet::push_recipe`]/[`StyleSheet::push_theme`]
//!    （検証済み生成元限定）以外に CSS を追加する経路を公開しない。
//! 4. [`StyleSheet::style_element`] 直前の再検証 — 上記 3 層を抜けて `<` が
//!    紛れ込んだ場合でも（到達不能想定・多層防御）空の `<style>` を返す。
//!
//! `<style>` 要素は RAWTEXT 文脈であり、そこから HTML 文脈へ脱出できる唯一の
//! 手段は `</style` 断片である。`<` を全面拒否すれば `</style` は構成不能になる
//! （`crates/pre-styled-ui/src/dialog.rs` の `stylesheet_never_contains_style_breakout_sequences`
//! と同じ根拠）。

use std::error::Error;
use std::fmt;
use std::io;
use std::path::Path;

use fandhe_frontend_headless_ui::fandhe_frontend_core::{el, raw_html, Node};

use crate::recipe::SlotRecipe;
use crate::theme::Theme;

/// [`StyleSheet::push_css`] の検証エラー。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StylesheetError {
    /// 渡された CSS 断片が `<` を含む、または改行・タブ・復帰以外の制御文字を
    /// 含んでいたため拒否された（fail-closed）。
    CssRejected {
        /// 拒否理由（診断用。入力 CSS 断片自体は含めない。機微情報の露出防止、
        /// `.claude/rules/security.md` 参照）。
        reason: &'static str,
    },
}

impl fmt::Display for StylesheetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StylesheetError::CssRejected { reason } => {
                write!(f, "CSS fragment rejected: {reason}")
            }
        }
    }
}

impl Error for StylesheetError {}

/// CSS 断片が [`StyleSheet`] へ取り込んでよい形式かどうかを判定する。
///
/// `<` を 1 文字でも含む場合、または改行 (`\n`)・タブ (`\t`)・復帰 (`\r`) 以外の
/// 制御文字を含む場合に `false` を返す（fail-closed）。改行・タブ・復帰を許容
/// するのは、[`crate::recipe::SlotRecipe::css`] 等が返す複数行 CSS をそのまま
/// 取り込めるようにするためであり、`crate::css::is_valid_value`（宣言 1 件の値
/// 検証、制御文字を全面拒否）とは適用対象が異なる。
fn is_safe_css_text(s: &str) -> bool {
    !s.contains('<')
        && !s
            .chars()
            .any(|c| c.is_control() && !matches!(c, '\n' | '\t' | '\r'))
}

/// 検証済み CSS のみを保持するシート（SSG のビルド時 CSS 集約・SSR の
/// `<style>` 埋め込みの両方で使う入れ物）。
///
/// フィールドは private であり、任意文字列から直接構築する経路を公開しない
/// （唯一の取り込み口は [`StyleSheet::push_css`] とその薄いラッパである
/// [`StyleSheet::push_recipe`]/[`StyleSheet::push_theme`]）。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StyleSheet {
    css: String,
}

impl StyleSheet {
    /// 空の [`StyleSheet`] を作る。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 任意の CSS 断片を検証して追加する（唯一の fallible な取り込み口）。
    ///
    /// 追加順に連結する（決定的）。既存内容と新規断片の間に区切りは挿入しない
    /// ため、呼び出し側は断片末尾の改行を CSS 生成元（[`crate::recipe::SlotRecipe::css`]
    /// 等はすべて末尾改行付きで返す）に揃えること。
    ///
    /// # Errors
    ///
    /// `css` が `<` を含む、または改行・タブ・復帰以外の制御文字を含む場合
    /// [`StylesheetError::CssRejected`] を返す（何も追加しない）。
    pub fn push_css(&mut self, css: &str) -> Result<(), StylesheetError> {
        if !is_safe_css_text(css) {
            return Err(StylesheetError::CssRejected {
                reason: "contains '<' or a disallowed control character",
            });
        }
        self.css.push_str(css);
        Ok(())
    }

    /// [`crate::recipe::SlotRecipe::css`] の出力を取り込む。
    ///
    /// `SlotRecipe::css` の出力は [`crate::css::is_valid_value`]／
    /// [`crate::css::is_valid_identifier`] の検証により `<` を構成不能に
    /// している（不変条件、モジュール doc 参照）ため、[`Self::push_css`] は
    /// 常に `Ok` を返す想定である（`tests/stylesheet_embed.rs` が全 styled
    /// 部品の `css()`/`stylesheet()` に対して機械検証する）。万一検証に落ちる
    /// 場合でも、生成側の fail-closed 方針（`crates/pre-styled-ui/src/css.rs`
    /// 冒頭 doc 参照）に合わせて黙ってスキップする（多層防御。到達不能。
    /// パニックさせない）。
    pub fn push_recipe(&mut self, recipe: &SlotRecipe) {
        let _ = self.push_css(&recipe.css());
    }

    /// [`crate::theme::Theme::to_css`] の出力を取り込む。
    ///
    /// `Theme::to_css` の出力は [`crate::theme::CssValue`]／
    /// [`crate::theme::TokenName`] の allowlist 検証により `<` を構成不能に
    /// している（不変条件、モジュール doc 参照）ため、[`Self::push_css`] は
    /// 常に `Ok` を返す想定である。万一検証に落ちる場合でも黙ってスキップする
    /// （[`Self::push_recipe`] と同じ多層防御の方針）。
    pub fn push_theme(&mut self, theme: &Theme) {
        let _ = self.push_css(&theme.to_css());
    }

    /// これまでに取り込んだ CSS 全量を返す。
    #[must_use]
    pub fn as_css(&self) -> &str {
        &self.css
    }

    /// 静的 `.css` ファイルへ書き出す（SSG・ビルドスクリプト向け）。
    ///
    /// `path` の親ディレクトリが存在しない場合は作成する。書き出す内容は
    /// [`Self::as_css`] とバイト一致し、同一の `self` に対する複数回の呼び出しは
    /// 常に同一内容を書き出す（決定的）。
    ///
    /// `path` はビルド時に呼び出し側（開発者のビルドスクリプト・SSG エントリ）が
    /// 指定する契約であり、リクエスト由来の入力をそのまま渡す経路（サーバーの
    /// リクエスト処理）からの呼び出しは想定しない（パストラバーサル対策は
    /// 呼び出し側の責務、`.claude/rules/security.md` の SSRF/パストラバーサル
    /// 節参照）。
    ///
    /// # Errors
    ///
    /// 親ディレクトリの作成・ファイル書き込みに失敗した場合、`std::io::Error`
    /// をそのまま返す。
    pub fn write_css_file(&self, path: &Path) -> io::Result<()> {
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }
        std::fs::write(path, &self.css)
    }

    /// SSR 埋め込み用の `<style>` 要素ノードを返す。
    ///
    /// 本クレートで [`raw_html`] を使用する唯一の箇所（`crate` 冒頭の
    /// 不変条件 2 の例外、モジュール doc 参照）。呼び出し文には
    /// `#[expect(clippy::disallowed_methods, reason = "ESCAPE-REVIEWED: ...")]`
    /// を付与済みであり、`clippy::disallowed_methods`（`clippy.toml`）が
    /// レビュー済みオプトインであることをコンパイラのパス解決で保証する。
    ///
    /// 埋め込み直前に `<` の非含有を再検証し、万一（構築経路上は到達不能）
    /// 違反していた場合は空の `<style></style>` を返す（fail-closed の
    /// 最終防壁）。
    #[must_use]
    pub fn style_element(&self) -> Node {
        if self.css.contains('<') {
            return el("style", vec![], vec![]);
        }
        #[expect(clippy::disallowed_methods, reason = "ESCAPE-REVIEWED: 検証済み(doc)")]
        let style_node = raw_html(self.css.clone());
        el("style", vec![], vec![style_node])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::recipe::SlotRecipe;
    use crate::theme::Theme;
    use fandhe_frontend_core::render;

    #[test]
    fn push_css_accepts_multiline_css_with_tabs_and_crlf() {
        let mut sheet = StyleSheet::new();
        assert!(sheet.push_css("a {\n\tcolor: red;\n}\r\n").is_ok());
        assert_eq!(sheet.as_css(), "a {\n\tcolor: red;\n}\r\n");
    }

    #[test]
    fn push_css_rejects_angle_bracket() {
        let mut sheet = StyleSheet::new();
        let err = sheet.push_css("a { color: red }</style><script>alert(1)</script>");
        assert_eq!(
            err,
            Err(StylesheetError::CssRejected {
                reason: "contains '<' or a disallowed control character",
            })
        );
        assert_eq!(sheet.as_css(), "");
    }

    #[test]
    fn push_css_rejects_less_than_alone() {
        let mut sheet = StyleSheet::new();
        assert!(sheet.push_css("a < b").is_err());
    }

    #[test]
    fn push_css_rejects_nul_byte() {
        let mut sheet = StyleSheet::new();
        assert!(sheet.push_css("a\u{0}b").is_err());
    }

    #[test]
    fn push_css_does_not_mutate_on_rejection() {
        let mut sheet = StyleSheet::new();
        sheet.push_css("a { color: red; }\n").unwrap();
        let before = sheet.as_css().to_string();
        assert!(sheet.push_css("bad < value").is_err());
        assert_eq!(sheet.as_css(), before);
    }

    /// 全 styled 部品の（モジュール名, 既定 CSS）一覧（イシュー #707 の一元化リスト）。
    ///
    /// `push_recipe_is_infallible_for_all_styled_components`（本リストの生成元が
    /// `push_css` を常に `Ok` で通ることの機械検証）と
    /// `all_styled_component_css_covers_every_component_module`（本リストの部品名
    /// 集合が `src/` 配下の `css()`/`stylesheet()` 公開モジュール集合と一致する
    /// ことの機械検証）の双方が参照する唯一の正。**新しい styled 部品を追加したら
    /// 必ずここへ登録する**こと（登録漏れは後者のドリフト検知テストが検出する）。
    fn all_styled_component_css() -> Vec<(&'static str, String)> {
        vec![
            ("button", crate::button::css()),
            ("badge", crate::badge::css()),
            ("spinner", crate::spinner::css()),
            ("alert", crate::alert::css()),
            ("card", crate::card::css()),
            ("dialog", crate::dialog::stylesheet()),
            ("drawer", crate::drawer::stylesheet()),
            ("progress", crate::progress::stylesheet()),
            ("hover_card", crate::hover_card::stylesheet()),
            ("tabs", crate::tabs::stylesheet()),
            ("accordion", crate::accordion::stylesheet()),
            ("menu", crate::menu::stylesheet()),
            ("select", crate::select::stylesheet()),
            ("skeleton", crate::skeleton::css()),
            ("separator", crate::separator::css()),
            ("combobox", crate::combobox::stylesheet()),
            ("popover", crate::popover::stylesheet()),
            ("tooltip", crate::tooltip::stylesheet()),
            ("toggle_tip", crate::toggle_tip::stylesheet()),
            ("switch", crate::switch::stylesheet()),
            ("radio_group", crate::radio_group::stylesheet()),
            ("avatar", crate::avatar::stylesheet()),
            ("checkbox", crate::checkbox::stylesheet()),
            ("checkbox_card", crate::checkbox_card::stylesheet()),
            ("radio_card", crate::radio_card::stylesheet()),
            ("input", crate::input::css()),
            ("textarea", crate::textarea::css()),
            ("native_select", crate::native_select::css()),
            ("number_input", crate::number_input::stylesheet()),
            ("slider", crate::slider::stylesheet()),
            ("pin_input", crate::pin_input::stylesheet()),
            ("tags_input", crate::tags_input::stylesheet()),
            ("rating_group", crate::rating_group::stylesheet()),
            ("toggle", crate::toggle::stylesheet()),
            ("toggle_group", crate::toggle_group::stylesheet()),
            ("segment_group", crate::segment_group::stylesheet()),
            ("tree_view", crate::tree_view::stylesheet()),
            ("pagination", crate::pagination::stylesheet()),
            ("breadcrumb", crate::breadcrumb::stylesheet()),
            ("carousel", crate::carousel::stylesheet()),
            ("link", crate::link::stylesheet()),
            ("link_overlay", crate::link_overlay::stylesheet()),
            ("nav_list", crate::nav_list::stylesheet()),
        ]
    }

    #[test]
    fn push_recipe_is_infallible_for_all_styled_components() {
        // 全 styled 部品（一元化リスト `all_styled_component_css`、イシュー #707）
        // の css()/stylesheet() が push_css で常に Ok になることを機械検証する
        // （push_recipe/push_theme の「到達不能スキップ」の根拠）。
        let mut sheet = StyleSheet::new();
        for (name, css) in all_styled_component_css() {
            assert!(
                sheet.push_css(&css).is_ok(),
                "component `{name}` css must pass push_css"
            );
        }
    }

    #[test]
    fn all_styled_component_css_covers_every_component_module() {
        // `all_styled_component_css`（一元化リスト）の部品名集合が、`src/` 配下で
        // `pub fn css()`/`pub fn stylesheet()` を公開する実モジュール集合と
        // 一致することを機械検証する（イシュー #707: #664 で追加された
        // popover/tooltip の登録漏れの再発防止）。ネットワーク・`/tmp` に依存
        // せず、コンパイル時確定の `CARGO_MANIFEST_DIR` のみを使う決定的判定
        // （`.claude/rules/ci.md` の self-hosted 共有環境への配慮に合わせる）。
        let src_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");

        let mut modules_with_css_fn: Vec<String> = Vec::new();
        for entry in std::fs::read_dir(&src_dir).expect("src ディレクトリを読み取れること")
        {
            let entry = entry.expect("dir entry を読み取れること");
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("rs") {
                continue;
            }
            let stem = path
                .file_stem()
                .and_then(|s| s.to_str())
                .expect("有効なファイル名であること")
                .to_string();
            // `lib.rs`（クレート入口）と `stylesheet.rs`（本ファイル自身）は
            // styled 部品モジュールではない。特に本ファイルは
            // `all_styled_component_css`/本テストの doc コメント中に
            // "pub fn css()"/"pub fn stylesheet()" という文字列そのものが
            // 出現するため、除外しないと自己参照で誤検知する。
            if stem == "lib" || stem == "stylesheet" {
                continue;
            }
            let source = std::fs::read_to_string(&path).expect("モジュールソースを読み取れること");
            if source.contains("pub fn css()") || source.contains("pub fn stylesheet()") {
                modules_with_css_fn.push(stem);
            }
        }
        modules_with_css_fn.sort();

        let mut listed: Vec<String> = all_styled_component_css()
            .into_iter()
            .map(|(name, _)| name.to_string())
            .collect();
        listed.sort();

        assert_eq!(
            modules_with_css_fn, listed,
            "all_styled_component_css() の一覧と src/ 配下の css()/stylesheet() \
             公開モジュール集合が一致しません（新規部品の登録漏れ、または \
             削除・リネームへの追随漏れの可能性があります）"
        );
    }

    #[test]
    fn push_theme_is_infallible_for_default_theme() {
        let mut sheet = StyleSheet::new();
        assert!(sheet.push_css(&Theme::default().to_css()).is_ok());
    }

    #[test]
    fn push_recipe_appends_recipe_css() {
        let recipe = SlotRecipe::new("widget", &["root"]);
        let mut sheet = StyleSheet::new();
        sheet.push_recipe(&recipe);
        assert_eq!(sheet.as_css(), recipe.css());
    }

    #[test]
    fn push_theme_appends_theme_css() {
        let theme = Theme::default();
        let mut sheet = StyleSheet::new();
        sheet.push_theme(&theme);
        assert_eq!(sheet.as_css(), theme.to_css());
    }

    #[test]
    fn style_element_renders_as_style_tag_wrapping_css_verbatim() {
        let mut sheet = StyleSheet::new();
        sheet.push_theme(&Theme::default());
        sheet.push_recipe(&SlotRecipe::new("widget", &["root"]));
        let rendered = render(&sheet.style_element());
        assert_eq!(rendered, format!("<style>{}</style>", sheet.as_css()));
    }

    #[test]
    fn style_element_on_empty_sheet_renders_empty_style_tag() {
        let sheet = StyleSheet::new();
        let rendered = render(&sheet.style_element());
        assert_eq!(rendered, "<style></style>");
    }

    #[test]
    fn as_css_and_render_never_contain_style_breakout_sequences() {
        let mut sheet = StyleSheet::new();
        sheet.push_theme(&Theme::default());
        sheet.push_recipe(&SlotRecipe::new("widget", &["root"]));
        sheet.push_css(&crate::dialog::stylesheet()).unwrap();
        assert!(!sheet.as_css().contains('<'));
        let rendered = render(&sheet.style_element());
        assert!(!rendered.contains("</style><"));
        assert_eq!(rendered.matches("</style>").count(), 1);
    }

    #[test]
    fn write_css_file_roundtrips_and_creates_parent_dir() {
        let mut sheet = StyleSheet::new();
        sheet.push_theme(&Theme::default());

        // self-hosted runner の共有 `/tmp` はユーザー単位のディスククォータが
        // 逼迫しうる（`.claude/rules/ci.md` 参照、PR #572 で既知の環境要因）ため
        // `std::env::temp_dir()` には依存せず、常にワークスペース内（`cargo test`
        // 実行時の cwd はパッケージルート）に一意なディレクトリを作る。
        let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join(format!(
                "stylesheet-test-tmp-{}-write_css_file_roundtrips_and_creates_parent_dir",
                std::process::id()
            ));
        let path = dir.join("nested").join("ui.css");

        sheet.write_css_file(&path).unwrap();
        let read_back = std::fs::read_to_string(&path).unwrap();
        assert_eq!(read_back, sheet.as_css());

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn write_css_file_is_deterministic_across_independent_sheets() {
        let mut sheet_a = StyleSheet::new();
        sheet_a.push_theme(&Theme::default());
        sheet_a.push_recipe(&SlotRecipe::new("widget", &["root"]));

        let mut sheet_b = StyleSheet::new();
        sheet_b.push_theme(&Theme::default());
        sheet_b.push_recipe(&SlotRecipe::new("widget", &["root"]));

        assert_eq!(sheet_a.as_css(), sheet_b.as_css());
    }
}
