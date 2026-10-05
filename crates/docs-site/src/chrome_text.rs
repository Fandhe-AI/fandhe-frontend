//! 生成器が出す固定のクローム文言（本文以外）を言語ごとに 1 か所へ集約する表と、
//! リポジトリリンクの表示種別（GitHub か汎用か）の判定。
//!
//! # 役割・呼び出し文脈
//!
//! - `crate::layout`（検索ボタン・検索ダイアログ・リポジトリリンク）、
//!   `crate::nav`（前後ページャ）、`crate::redirect`（リダイレクト案内）、
//!   `crate::not_found`（404）、`crate::site_footer`（Resources 列のリポジトリリンク）が、
//!   `[site].lang`（`crate::nav::Site::html_lang`）を [`ChromeText::for_lang`] へ渡して
//!   文言を引く。文言の選択はここ 1 か所にしかない。
//! - 選択規則（`docs/design/docs-site-external-use.md` §4.3）: `lang` の先頭サブタグが
//!   `ja`（ASCII 大文字小文字無視）なら [`JA`]（変更前の出力と 1 バイトも変えない現行文言）、
//!   それ以外は全て [`EN`]。未指定の既定 `ja` もこの規則で [`JA`] になる。
//! - 対象外: `[site]` キーで差し替えられる文言（`tagline` / `copyright` 等）と、本番
//!   registry 専用の生成節（`landing.rs`・各索引・部品ページ）。
//!
//! # セキュリティ不変条件
//!
//! 文言は全て `&'static str` の定数で、利用者が任意文言を差し込むキーは存在しない。
//! 出力は `text()` と属性（`el()` の既定エスケープ）経由のみで、JS 文字列リテラルへ
//! 埋め込む文言は無い（`assets/site.js` が持つ文言は元から英語で、本表の対象外）。
//! `raw_html()` 相当の迂回経路は持たない。

/// 1 言語分のクローム文言。`JA` / `EN` 定数の並びで項目ごとの対応が取れる。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChromeText {
    /// 検索ボタンの `aria-label`。
    pub search_button_aria: &'static str,
    /// 検索ボタンの可視ラベル。
    pub search_button_label: &'static str,
    /// 検索ダイアログと検索入力の `aria-label`。
    pub search_dialog_aria: &'static str,
    /// 検索入力の `placeholder`。
    pub search_placeholder: &'static str,
    /// 前後ページャ（`nav.prev-next`）の `aria-label`。
    pub pager_aria: &'static str,
    /// 前ページカードのラベル。
    pub pager_prev: &'static str,
    /// 次ページカードのラベル。
    pub pager_next: &'static str,
    /// リダイレクト案内の `<title>` の前置き（`{prefix} | {site_title}` の形で使う）。
    pub redirect_title_prefix: &'static str,
    /// リダイレクト案内の本文（直後にリンクが続く）。
    pub redirect_body: &'static str,
    /// 404 の `<title>` と見出し。
    pub not_found_title: &'static str,
    /// 404 の説明文。
    pub not_found_description: &'static str,
    /// 404 のセクション一覧の見出し。
    pub not_found_sections: &'static str,
}

/// 日本語の文言表。変更前の出力そのもの（バイト一致の基準）。
pub const JA: ChromeText = ChromeText {
    search_button_aria: "ドキュメントを検索",
    search_button_label: "検索",
    search_dialog_aria: "ドキュメント内検索",
    search_placeholder: "ドキュメントを検索",
    pager_aria: "前後のページ",
    pager_prev: "前へ",
    pager_next: "次へ",
    redirect_title_prefix: "移転しました",
    redirect_body: "このページは移動しました。自動的に移動しない場合は次のリンクを開いてください: ",
    not_found_title: "ページが見つかりません",
    not_found_description: "お探しのページは移動または削除された可能性があります。\
                            下のセクション一覧から探すか、JavaScript が有効な環境ではヘッダーの検索をお使いください。",
    not_found_sections: "主なセクション",
};

/// 英語の文言表（`ja` 以外の全言語で使う）。
pub const EN: ChromeText = ChromeText {
    search_button_aria: "Search documentation",
    search_button_label: "Search",
    search_dialog_aria: "Search in documentation",
    search_placeholder: "Search documentation",
    pager_aria: "Previous and next pages",
    pager_prev: "Previous",
    pager_next: "Next",
    redirect_title_prefix: "Moved",
    redirect_body:
        "This page has moved. If you are not redirected automatically, open the following link: ",
    not_found_title: "Page not found",
    not_found_description: "The page you are looking for may have been moved or removed. \
                            Look for it in the section list below, or use the search in the header if JavaScript is enabled.",
    not_found_sections: "Main sections",
};

impl ChromeText {
    /// `lang`（`[site].lang` 解決済みの値）の先頭サブタグで文言表を選ぶ。
    ///
    /// 先頭サブタグが `ja`（ASCII 大文字小文字無視）なら [`JA`]、それ以外は [`EN`]。
    /// `lang` は `parse_nav` が BCP 47 の構文サブセットとして検証済みの値を受ける前提だが、
    /// 空文字などの想定外入力でも panic せず [`EN`] へ倒れる。
    #[must_use]
    pub fn for_lang(lang: &str) -> &'static ChromeText {
        let primary = lang.split('-').next().unwrap_or("");
        if primary.eq_ignore_ascii_case("ja") {
            &JA
        } else {
            &EN
        }
    }
}

/// リポジトリリンクの文言・アイコン種別。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepositoryLinkKind {
    /// ホストが `github.com`（`www.` 付きを含む）。文言 "GitHub" と GitHub マーク。
    GitHub,
    /// それ以外。文言 "Repository" と、特定サービスを示さない汎用アイコン。
    Generic,
}

impl RepositoryLinkKind {
    /// `repository_url`（`parse_nav` が `https://` を検証済み）のホストで種別を決める。
    ///
    /// ホスト部は `crate::nav::https_url_host`（検証と同じ authority 規則: userinfo と
    /// ポートを除く）から得て、ASCII 大文字小文字を無視して `github.com` /
    /// `www.github.com` と完全一致した場合だけ [`Self::GitHub`] にする。
    /// `github.com.evil.example` や `evilgithub.com`、`https://github.com@evil.example/`
    /// （userinfo が github.com でホストは evil.example）は [`Self::Generic`]。
    #[must_use]
    pub fn from_url(repository_url: &str) -> Self {
        let host = crate::nav::https_url_host(repository_url);
        if host.eq_ignore_ascii_case("github.com") || host.eq_ignore_ascii_case("www.github.com") {
            Self::GitHub
        } else {
            Self::Generic
        }
    }

    /// リンクの可視文言（ヘッダー・フッター Resources 列で共通）。
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::GitHub => "GitHub",
            Self::Generic => "Repository",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lang_selects_table_by_primary_subtag() {
        for lang in ["ja", "JA", "Ja", "ja-JP", "ja-jp"] {
            assert_eq!(ChromeText::for_lang(lang), &JA, "{lang}");
        }
        for lang in ["en", "en-US", "fr", "zh-Hans", "jaa", "j", "", "xja"] {
            assert_eq!(ChromeText::for_lang(lang), &EN, "{lang}");
        }
    }

    #[test]
    fn ja_table_keeps_pre_change_strings() {
        assert_eq!(JA.search_button_aria, "ドキュメントを検索");
        assert_eq!(JA.search_button_label, "検索");
        assert_eq!(JA.search_dialog_aria, "ドキュメント内検索");
        assert_eq!(JA.pager_aria, "前後のページ");
        assert_eq!((JA.pager_prev, JA.pager_next), ("前へ", "次へ"));
        assert_eq!(JA.redirect_title_prefix, "移転しました");
        assert_eq!(JA.not_found_title, "ページが見つかりません");
        assert_eq!(JA.not_found_sections, "主なセクション");
    }

    #[test]
    fn en_table_is_ascii_only() {
        let all = [
            EN.search_button_aria,
            EN.search_button_label,
            EN.search_dialog_aria,
            EN.search_placeholder,
            EN.pager_aria,
            EN.pager_prev,
            EN.pager_next,
            EN.redirect_title_prefix,
            EN.redirect_body,
            EN.not_found_title,
            EN.not_found_description,
            EN.not_found_sections,
        ];
        for s in all {
            assert!(s.is_ascii(), "{s}");
        }
    }

    #[test]
    fn repository_host_decides_kind() {
        let github = [
            "https://github.com/o/r",
            "https://GitHub.com/o/r",
            "https://www.github.com/o/r",
            "https://WWW.GITHUB.COM",
            "https://github.com:443/o/r",
            "https://user@github.com/o/r",
            "https://github.com?x=1",
        ];
        for u in github {
            assert_eq!(
                RepositoryLinkKind::from_url(u),
                RepositoryLinkKind::GitHub,
                "{u}"
            );
        }
        let generic = [
            "https://gitlab.com/o/r",
            "https://github.com.evil.example/o/r",
            "https://evilgithub.com/o/r",
            "https://github.com@evil.example/",
            "https://sub.github.com/o/r",
            "https://example.com/github.com",
        ];
        for u in generic {
            assert_eq!(
                RepositoryLinkKind::from_url(u),
                RepositoryLinkKind::Generic,
                "{u}"
            );
        }
    }

    #[test]
    fn backslash_urls_are_not_github_even_without_validation() {
        // 検証を経ない直接呼び出しでも、`\` を `/` と解釈するブラウザの遷移先
        // （evil.example）と食い違う GitHub 判定をしない。
        for u in [
            r"https://evil.example\@github.com/",
            r"https://evil.example\github.com/",
            r"https://evil.example\@www.github.com/",
        ] {
            assert_eq!(
                RepositoryLinkKind::from_url(u),
                RepositoryLinkKind::Generic,
                "{u}"
            );
        }
    }

    #[test]
    fn labels_follow_kind() {
        assert_eq!(RepositoryLinkKind::GitHub.label(), "GitHub");
        assert_eq!(RepositoryLinkKind::Generic.label(), "Repository");
    }
}
