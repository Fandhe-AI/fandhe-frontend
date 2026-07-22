//! テーマトークン・ダークモード基盤（イシュー #547、親 #545/#520）。
//!
//! chakra-ui の semantic token を参考に、CSS custom properties ベースの色・余白・
//! タイポグラフィトークンと、`prefers-color-scheme` メディアクエリ + `data-theme`
//! 属性上書きの両対応ダークモードを提供する。出力はプレーン CSS の決定的な静的
//! 文字列であり、ランタイム CSS-in-JS を持たない（`docs/policy/intentional-non-adoption.md`
//! の評価軸に整合。variant API・クラス名/静的 CSS 生成はイシュー #548 のスコープ）。
//!
//! # 他クレート・他モジュールとの契約
//!
//! - [`Theme::to_css`] の出力は `<style>` タグへの埋め込みを想定しない。本クレートの
//!   不変条件（`raw_html()` 不使用、[`crate`] の rustdoc 参照）を維持するため、静的
//!   `.css` ファイルとして配信する利用形態を前提とする。
//! - [`color_var`] / [`space_var`] / [`typography_var`] は、イシュー #548（variant
//!   API）・#550/#551（styled 部品）が `var(--fandhe-...)` 参照を組み立てる際に使う
//!   想定のヘルパ。
//!
//! # セキュリティ上の不変条件（REQ-1 相当、CSS 文脈）
//!
//! [`Theme`] へ値を追加する経路は [`CssValue::new`] / [`TokenName::new`] の
//! allowlist 検証のみを通過させる（fail-closed）。`:` `;` `{` `}` を拒否するため
//! 宣言追加・セレクタ脱出・`url(javascript:...)` が構成不可能であり、`<` `>` `/` を
//! 拒否するため `</style>` 脱出も構成不可能になる。core の `escape_html`（HTML
//! 文脈のエスケープ）とは独立した、CSS 文脈専用の入力検証層である。

use std::error::Error;
use std::fmt;

/// テーマトークン API の検証エラー（fail-closed。不正値は黙って除去せず `Err` を返す）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ThemeError {
    /// [`CssValue::new`] に渡された文字列が allowlist 検証を通過しなかった。
    InvalidCssValue {
        /// 検証に失敗した入力（診断用。機微情報は含まれない前提の CSS 値のみ）。
        value: String,
    },
    /// [`TokenName::new`] に渡された文字列がトークン名の命名規則を満たさなかった。
    InvalidTokenName {
        /// 検証に失敗した入力。
        value: String,
    },
    /// 同一グループ内で既に登録済みのトークン名が再度渡された（上書きによる意図しない
    /// 挙動を防ぐため fail-closed で拒否する）。
    DuplicateTokenName {
        /// 重複していたトークン名。
        name: String,
    },
}

impl fmt::Display for ThemeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ThemeError::InvalidCssValue { value } => {
                write!(f, "invalid CSS value (rejected by allowlist): {value:?}")
            }
            ThemeError::InvalidTokenName { value } => {
                write!(f, "invalid token name (rejected by naming rule): {value:?}")
            }
            ThemeError::DuplicateTokenName { name } => {
                write!(f, "duplicate token name in the same group: {name:?}")
            }
        }
    }
}

impl Error for ThemeError {}

/// CSS 値の長さ上限（文字数）。極端に長い入力を早期に拒否する（DoS 耐性・意図しない
/// 断片混入の抑止）。
const CSS_VALUE_MAX_LEN: usize = 256;

/// トークン名の長さ上限（文字数）。
const TOKEN_NAME_MAX_LEN: usize = 64;

/// allowlist 検証を通過した CSS 値。
///
/// `Theme` へ追加できる値はこの型を経由したものに限られる（[`Theme`] の各
/// `push_*` メソッドは `&str` を受け取った直後に内部で検証する）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CssValue(String);

impl CssValue {
    /// CSS 値として安全な文字集合のみを許可する（fail-closed）。
    ///
    /// # 許可文字
    ///
    /// ASCII 英数字・空白・`#` `%` `.` `,` `(` `)` `-` `_` のみ。
    ///
    /// # 拒否条件
    ///
    /// - `:` `;` `{` `}` `<` `>` `&` `"` `'` `\` `/` `!` のいずれかを含む
    ///   （宣言追加・セレクタ脱出・`url(javascript:...)`・`</style>` 脱出を
    ///   構成不能にするための拒否文字）
    /// - 制御文字・非 ASCII 文字を含む
    /// - 大文字小文字を無視して `expression(` を部分文字列として含む
    ///   （レガシー IE の CSS `expression()` 動的プロパティ経由のスクリプト実行を
    ///   拒否する。英数字と `(` `)` のみの許可文字集合では区別できないため、
    ///   allowlist に加えて明示的な denylist として扱う）
    /// - 空文字列
    /// - [`CSS_VALUE_MAX_LEN`] を超える長さ
    ///
    /// # Errors
    ///
    /// 上記いずれかに該当する場合 [`ThemeError::InvalidCssValue`] を返す。
    pub fn new(s: &str) -> Result<Self, ThemeError> {
        let is_valid = !s.is_empty()
            && s.len() <= CSS_VALUE_MAX_LEN
            && s.chars().all(|c| {
                c.is_ascii_alphanumeric()
                    || c == ' '
                    || c == '#'
                    || c == '%'
                    || c == '.'
                    || c == ','
                    || c == '('
                    || c == ')'
                    || c == '-'
                    || c == '_'
            })
            && !s.to_ascii_lowercase().contains("expression(");

        if is_valid {
            Ok(Self(s.to_string()))
        } else {
            Err(ThemeError::InvalidCssValue {
                value: s.to_string(),
            })
        }
    }

    /// 検証済み CSS 値を文字列スライスとして取得する。
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// allowlist 検証を通過したトークン名（例: `bg` `space-4` `font-size-md`）。
///
/// CSS custom property 名の一部（`--fandhe-<group>-<name>`）として使われるため、
/// CSS 識別子として安全な文字集合に制限する。
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TokenName(String);

impl TokenName {
    /// トークン名の命名規則を検証する（fail-closed）。
    ///
    /// # 規則
    ///
    /// - `[a-z0-9]` で始まる
    /// - 以降は `[a-z0-9-]` のみ
    /// - 末尾がハイフンでない
    /// - 空文字列でない
    /// - [`TOKEN_NAME_MAX_LEN`] を超えない
    ///
    /// # Errors
    ///
    /// 上記いずれかを満たさない場合 [`ThemeError::InvalidTokenName`] を返す。
    pub fn new(s: &str) -> Result<Self, ThemeError> {
        let bytes = s.as_bytes();
        let starts_alnum_lower =
            matches!(bytes.first(), Some(b) if b.is_ascii_lowercase() || b.is_ascii_digit());
        let is_valid = !s.is_empty()
            && s.len() <= TOKEN_NAME_MAX_LEN
            && starts_alnum_lower
            && s.bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
            && bytes[bytes.len() - 1] != b'-';

        if is_valid {
            Ok(Self(s.to_string()))
        } else {
            Err(ThemeError::InvalidTokenName {
                value: s.to_string(),
            })
        }
    }

    /// 検証済みトークン名を文字列スライスとして取得する。
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// ライト/ダークの 2 値を持つトークン 1 件。
///
/// colors（イシュー #547）・shadows（イシュー #606）が共用する内部表現。
/// shadow はダークモードで光量の異なる影が必要になる（chakra-ui も同様に
/// カラーモードごとに異なる `box-shadow` 値を持つ）ため、color と同じ
/// light/dark 2 値構造を採用し、二重定義を避ける。
#[derive(Debug, Clone)]
struct DualModeToken {
    name: TokenName,
    light: CssValue,
    dark: CssValue,
}

/// モード非依存のスケールトークン 1 件（spaces・typography 共用）。
#[derive(Debug, Clone)]
struct ScaleToken {
    name: TokenName,
    value: CssValue,
}

/// CSS custom property のプレフィックス。他クレート（#548/#550/#551）が同一の
/// 名前空間で var() 参照を組み立てられるよう固定する。
const VAR_PREFIX: &str = "--fandhe";

/// テーマ全体（色・余白・タイポグラフィ・角丸・影トークンの集合）。
///
/// フィールドは `Vec`（挿入順 = 出力順）で保持し、[`Theme::to_css`] の出力の
/// 決定性を構造的に保証する（`HashMap` 等の非決定順コレクションを使わない）。
#[derive(Debug, Clone)]
pub struct Theme {
    colors: Vec<DualModeToken>,
    spaces: Vec<ScaleToken>,
    typography: Vec<ScaleToken>,
    radii: Vec<ScaleToken>,
    shadows: Vec<DualModeToken>,
}

impl Default for Theme {
    /// chakra-ui の semantic token を参考にした既定パレット（色・余白・
    /// タイポグラフィの最小構成、イシュー #547 スコープ）。
    ///
    /// 既定値はすべて [`CssValue`]/[`TokenName`] の allowlist を満たす定数の
    /// ため、`push_*` の `Result` は `expect` せず構築時に確定させる
    /// （ライブラリコードでの `panic!`/`unwrap()` 回避のため、内部専用の
    /// infallible ヘルパを用いる）。
    fn default() -> Self {
        let mut theme = Theme {
            colors: Vec::new(),
            spaces: Vec::new(),
            typography: Vec::new(),
            radii: Vec::new(),
            shadows: Vec::new(),
        };

        for (name, light, dark) in DEFAULT_COLORS {
            theme.push_color(name, light, dark).expect(
                "既定パレットの定数は allowlist を満たすよう手動で検証済み（ユニットテストで固定）",
            );
        }
        for (name, value) in DEFAULT_SPACES {
            theme.push_space(name, value).expect(
                "既定パレットの定数は allowlist を満たすよう手動で検証済み（ユニットテストで固定）",
            );
        }
        for (name, value) in DEFAULT_TYPOGRAPHY {
            theme.push_typography(name, value).expect(
                "既定パレットの定数は allowlist を満たすよう手動で検証済み（ユニットテストで固定）",
            );
        }
        for (name, value) in DEFAULT_RADII {
            theme.push_radius(name, value).expect(
                "既定パレットの定数は allowlist を満たすよう手動で検証済み（ユニットテストで固定）",
            );
        }
        for (name, light, dark) in DEFAULT_SHADOWS {
            theme.push_shadow(name, light, dark).expect(
                "既定パレットの定数は allowlist を満たすよう手動で検証済み（ユニットテストで固定）",
            );
        }

        theme
    }
}

/// 既定の色トークン（name, light, dark）。chakra-ui semantic token 参考の最小構成。
const DEFAULT_COLORS: &[(&str, &str, &str)] = &[
    ("bg", "#ffffff", "#111111"),
    ("bg-subtle", "#f7f7f7", "#1a1a1a"),
    ("bg-muted", "#eeeeee", "#242424"),
    ("fg", "#111111", "#f7f7f7"),
    ("fg-muted", "#4a4a4a", "#cccccc"),
    ("fg-subtle", "#767676", "#a3a3a3"),
    ("border", "#d9d9d9", "#3a3a3a"),
    ("border-muted", "#e6e6e6", "#2a2a2a"),
    ("accent", "#3182ce", "#4299e1"),
    ("accent-emphasized", "#2b6cb0", "#63b3ed"),
    ("accent-fg", "#ffffff", "#0b1720"),
    ("info", "#3182ce", "#63b3ed"),
    ("info-emphasized", "#2b6cb0", "#90cdf4"),
    ("info-fg", "#ffffff", "#0b1720"),
    ("success", "#2f855a", "#68d391"),
    ("success-emphasized", "#276749", "#9ae6b4"),
    ("success-fg", "#ffffff", "#0b1a12"),
    ("warning", "#b7791f", "#f6ad55"),
    ("warning-emphasized", "#975a16", "#fbd38d"),
    ("warning-fg", "#ffffff", "#1a1203"),
    ("danger", "#c53030", "#fc8181"),
    ("danger-emphasized", "#9b2c2c", "#feb2b2"),
    ("danger-fg", "#ffffff", "#1a0b0b"),
];

/// 既定の角丸トークン（name, value）。モード非依存。既存 styled 部品
/// （Button/Badge/Spinner/Alert/Card、イシュー #550）のリテラル値をそのまま
/// 吸収する初期スケール（イシュー #606）。
const DEFAULT_RADII: &[(&str, &str)] = &[
    ("sm", "0.25rem"),
    ("md", "0.375rem"),
    ("lg", "0.5rem"),
    ("xl", "0.75rem"),
    ("full", "9999px"),
];

/// 既定の影トークン（name, light, dark）。ダークモードは light 比で不透明度を
/// 上げ、暗背景上でも輪郭が視認できるようにする（イシュー #606）。`sm` の
/// light 値は Card Elevated の既存リテラル（イシュー #550）を踏襲する。
const DEFAULT_SHADOWS: &[(&str, &str, &str)] = &[
    (
        "xs",
        "0 1px 2px rgba(0, 0, 0, 0.06)",
        "0 1px 2px rgba(0, 0, 0, 0.24)",
    ),
    (
        "sm",
        "0 1px 3px rgba(0, 0, 0, 0.12)",
        "0 1px 3px rgba(0, 0, 0, 0.32)",
    ),
    (
        "md",
        "0 4px 6px rgba(0, 0, 0, 0.1)",
        "0 4px 6px rgba(0, 0, 0, 0.3)",
    ),
    (
        "lg",
        "0 10px 15px rgba(0, 0, 0, 0.16)",
        "0 10px 15px rgba(0, 0, 0, 0.4)",
    ),
];

/// 既定の余白トークン（name, value）。chakra 風のスケール。モード非依存。
const DEFAULT_SPACES: &[(&str, &str)] = &[
    ("1", "0.25rem"),
    ("2", "0.5rem"),
    ("3", "0.75rem"),
    ("4", "1rem"),
    ("5", "1.25rem"),
    ("6", "1.5rem"),
    ("8", "2rem"),
    ("10", "2.5rem"),
    ("12", "3rem"),
    ("16", "4rem"),
];

/// 既定のタイポグラフィトークン（name, value）。モード非依存。
const DEFAULT_TYPOGRAPHY: &[(&str, &str)] = &[
    ("font-body", "system-ui, -apple-system, sans-serif"),
    ("font-mono", "ui-monospace, monospace"),
    ("font-size-xs", "0.75rem"),
    ("font-size-sm", "0.875rem"),
    ("font-size-md", "1rem"),
    ("font-size-lg", "1.125rem"),
    ("font-size-xl", "1.25rem"),
    ("font-size-2xl", "1.5rem"),
    ("font-size-3xl", "1.875rem"),
    ("font-size-4xl", "2.25rem"),
    ("font-weight-normal", "400"),
    ("font-weight-medium", "500"),
    ("font-weight-semibold", "600"),
    ("font-weight-bold", "700"),
    ("line-height-tight", "1.25"),
    ("line-height-normal", "1.5"),
    ("line-height-relaxed", "1.75"),
];

impl Theme {
    /// 空のテーマを構築する（既定トークンなし）。カスタムテーマをゼロから
    /// 組み立てたい呼び出し元向け。既定パレットが欲しい場合は
    /// [`Theme::default`] を使う。
    #[must_use]
    pub fn empty() -> Self {
        Theme {
            colors: Vec::new(),
            spaces: Vec::new(),
            typography: Vec::new(),
            radii: Vec::new(),
            shadows: Vec::new(),
        }
    }

    /// ライト/ダーク値を持つ色トークンを追加する。
    ///
    /// # Errors
    ///
    /// - `name` / `light` / `dark` のいずれかが allowlist 検証を通過しない場合
    /// - `name` が colors グループ内で既に登録済みの場合（[`ThemeError::DuplicateTokenName`]）
    pub fn push_color(&mut self, name: &str, light: &str, dark: &str) -> Result<(), ThemeError> {
        let name = TokenName::new(name)?;
        let light = CssValue::new(light)?;
        let dark = CssValue::new(dark)?;

        if self.colors.iter().any(|t| t.name == name) {
            return Err(ThemeError::DuplicateTokenName {
                name: name.as_str().to_string(),
            });
        }

        self.colors.push(DualModeToken { name, light, dark });
        Ok(())
    }

    /// モード非依存の余白トークンを追加する。
    ///
    /// # Errors
    ///
    /// [`Theme::push_color`] と同様（`name`/`value` の検証・重複拒否）。
    pub fn push_space(&mut self, name: &str, value: &str) -> Result<(), ThemeError> {
        push_scale(&mut self.spaces, name, value)
    }

    /// モード非依存のタイポグラフィトークンを追加する。
    ///
    /// # Errors
    ///
    /// [`Theme::push_color`] と同様（`name`/`value` の検証・重複拒否）。
    pub fn push_typography(&mut self, name: &str, value: &str) -> Result<(), ThemeError> {
        push_scale(&mut self.typography, name, value)
    }

    /// モード非依存の角丸（`border-radius`）トークンを追加する（イシュー #606）。
    ///
    /// `fandhe-frontend-pre-styled-ui` の styled 部品（Button/Badge/Spinner/
    /// Alert/Card）が `border-radius: var(--fandhe-radius-<name>)` として
    /// 参照する想定のトークン。
    ///
    /// # Errors
    ///
    /// [`Theme::push_color`] と同様（`name`/`value` の検証・重複拒否）。
    pub fn push_radius(&mut self, name: &str, value: &str) -> Result<(), ThemeError> {
        push_scale(&mut self.radii, name, value)
    }

    /// ライト/ダーク値を持つ影（`box-shadow`）トークンを追加する（イシュー #606）。
    ///
    /// ダークモードで光量の異なる影が必要になるため、[`Theme::push_color`] と
    /// 同じ light/dark 2 値構造を取る（内部表現は [`DualModeToken`] を共用）。
    ///
    /// # Errors
    ///
    /// - `name` / `light` / `dark` のいずれかが allowlist 検証を通過しない場合
    /// - `name` が shadows グループ内で既に登録済みの場合（[`ThemeError::DuplicateTokenName`]）
    pub fn push_shadow(&mut self, name: &str, light: &str, dark: &str) -> Result<(), ThemeError> {
        let name = TokenName::new(name)?;
        let light = CssValue::new(light)?;
        let dark = CssValue::new(dark)?;

        if self.shadows.iter().any(|t| t.name == name) {
            return Err(ThemeError::DuplicateTokenName {
                name: name.as_str().to_string(),
            });
        }

        self.shadows.push(DualModeToken { name, light, dark });
        Ok(())
    }

    /// テーマを決定的なプレーン CSS 文字列へ変換する。
    ///
    /// 出力構造（固定順、`docs` は伴わず本 rustdoc が正）:
    ///
    /// 1. `:root { color-scheme: light dark; --fandhe-... }`（light 値、
    ///    colors → spaces → typography → radii → shadows の順。radii は
    ///    モード非依存のため 1 値、shadows は light 値をここに出力する。
    ///    イシュー #606 で追加した 2 グループは末尾に純追加する構成のため、
    ///    radii/shadows を push しないテーマの出力は変更前とバイト同一になる）
    /// 2. `:root[data-theme="light"] { color-scheme: light; }`
    /// 3. `@media (prefers-color-scheme: dark) { :root:not([data-theme="light"]) { ... } }`
    ///    （dark 値。OS 設定追従。colors → shadows の順）
    /// 4. `:root[data-theme="dark"] { ... }`（dark 値。明示指定は常に勝つ。
    ///    3 と同特異度のため、末尾に置く出力順序でメディアクエリより優先させる）
    ///
    /// 3 と 4 の dark トークン列は同一の内部ヘルパ（[`Theme::write_dark_declarations`]）
    /// から生成し、二重管理による乖離を構造的に防ぐ。
    ///
    /// 呼び出し元は返り値を静的 `.css` ファイルとして配信する想定であり、
    /// `<style>` タグへの埋め込みは本クレートの責務外（[`crate`] 冒頭の
    /// 不変条件を参照）。
    #[must_use]
    pub fn to_css(&self) -> String {
        let mut out = String::new();

        out.push_str(":root {\n");
        out.push_str("  color-scheme: light dark;\n");
        for token in &self.colors {
            out.push_str(&format!(
                "  {VAR_PREFIX}-color-{}: {};\n",
                token.name.as_str(),
                token.light.as_str()
            ));
        }
        for token in &self.spaces {
            out.push_str(&format!(
                "  {VAR_PREFIX}-space-{}: {};\n",
                token.name.as_str(),
                token.value.as_str()
            ));
        }
        for token in &self.typography {
            out.push_str(&format!(
                "  {VAR_PREFIX}-font-{}: {};\n",
                token.name.as_str(),
                token.value.as_str()
            ));
        }
        for token in &self.radii {
            out.push_str(&format!(
                "  {VAR_PREFIX}-radius-{}: {};\n",
                token.name.as_str(),
                token.value.as_str()
            ));
        }
        for token in &self.shadows {
            out.push_str(&format!(
                "  {VAR_PREFIX}-shadow-{}: {};\n",
                token.name.as_str(),
                token.light.as_str()
            ));
        }
        out.push_str("}\n");

        out.push_str(":root[data-theme=\"light\"] { color-scheme: light; }\n");

        out.push_str("@media (prefers-color-scheme: dark) {\n");
        out.push_str("  :root:not([data-theme=\"light\"]) {\n");
        out.push_str("    color-scheme: dark;\n");
        self.write_dark_declarations(&mut out, "    ");
        out.push_str("  }\n");
        out.push_str("}\n");

        out.push_str(":root[data-theme=\"dark\"] {\n");
        out.push_str("  color-scheme: dark;\n");
        self.write_dark_declarations(&mut out, "  ");
        out.push_str("}\n");

        out
    }

    /// dark モードの custom property 宣言列を書き出す内部ヘルパ。
    ///
    /// `@media` ブロックと `:root[data-theme="dark"]` ブロックの双方が本関数を
    /// 経由することで、同一のトークン列（`self.colors`/`self.shadows` の dark
    /// 値）から生成し、2 箇所の出力が構造的に一致することを保証する
    /// （手書きの二重管理を避ける）。radii はモード非依存のため対象外。
    fn write_dark_declarations(&self, out: &mut String, indent: &str) {
        for token in &self.colors {
            out.push_str(&format!(
                "{indent}{VAR_PREFIX}-color-{}: {};\n",
                token.name.as_str(),
                token.dark.as_str()
            ));
        }
        for token in &self.shadows {
            out.push_str(&format!(
                "{indent}{VAR_PREFIX}-shadow-{}: {};\n",
                token.name.as_str(),
                token.dark.as_str()
            ));
        }
    }
}

/// [`Theme::push_space`] / [`Theme::push_typography`] 共通の検証・追加ロジック。
fn push_scale(target: &mut Vec<ScaleToken>, name: &str, value: &str) -> Result<(), ThemeError> {
    let name = TokenName::new(name)?;
    let value = CssValue::new(value)?;

    if target.iter().any(|t| t.name == name) {
        return Err(ThemeError::DuplicateTokenName {
            name: name.as_str().to_string(),
        });
    }

    target.push(ScaleToken { name, value });
    Ok(())
}

/// 色トークン名から `var(--fandhe-color-<name>)` 参照を組み立てる。
///
/// #548（variant API）・#550/#551（styled 部品）がインライン style 属性値等を
/// 組み立てる際の参照ヘルパ。`name` は [`TokenName`] の命名規則で検証する。
///
/// # Errors
///
/// `name` がトークン名の命名規則を満たさない場合 [`ThemeError::InvalidTokenName`]
/// を返す。
pub fn color_var(name: &str) -> Result<String, ThemeError> {
    let name = TokenName::new(name)?;
    Ok(format!("var({VAR_PREFIX}-color-{})", name.as_str()))
}

/// 余白トークン名から `var(--fandhe-space-<name>)` 参照を組み立てる。
///
/// # Errors
///
/// [`color_var`] と同様。
pub fn space_var(name: &str) -> Result<String, ThemeError> {
    let name = TokenName::new(name)?;
    Ok(format!("var({VAR_PREFIX}-space-{})", name.as_str()))
}

/// 角丸トークン名から `var(--fandhe-radius-<name>)` 参照を組み立てる（イシュー
/// #606）。styled 部品が `border-radius` の値として参照する想定。
///
/// # Errors
///
/// [`color_var`] と同様。
pub fn radius_var(name: &str) -> Result<String, ThemeError> {
    let name = TokenName::new(name)?;
    Ok(format!("var({VAR_PREFIX}-radius-{})", name.as_str()))
}

/// 影トークン名から `var(--fandhe-shadow-<name>)` 参照を組み立てる（イシュー
/// #606）。styled 部品が `box-shadow` の値として参照する想定。
///
/// # Errors
///
/// [`color_var`] と同様。
pub fn shadow_var(name: &str) -> Result<String, ThemeError> {
    let name = TokenName::new(name)?;
    Ok(format!("var({VAR_PREFIX}-shadow-{})", name.as_str()))
}

/// タイポグラフィトークン名から `var(--fandhe-font-<name>)` 参照を組み立てる。
///
/// # Errors
///
/// [`color_var`] と同様。
pub fn typography_var(name: &str) -> Result<String, ThemeError> {
    let name = TokenName::new(name)?;
    Ok(format!("var({VAR_PREFIX}-font-{})", name.as_str()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn css_value_accepts_typical_values() {
        assert!(CssValue::new("#ffffff").is_ok());
        assert!(CssValue::new("1rem").is_ok());
        assert!(CssValue::new("system-ui, sans-serif").is_ok());
        assert!(CssValue::new("rgba(0, 0, 0)").is_ok());
    }

    #[test]
    fn css_value_rejects_injection_characters() {
        for bad in [
            "red;}",
            "x{y:z}",
            "red:blue",
            "</style><script>alert(1)</script>",
            "url(javascript:alert(1))",
            "expression(alert(1))",
            "a\"b",
            "a'b",
            "a\\b",
            "a&b",
        ] {
            assert!(
                CssValue::new(bad).is_err(),
                "should reject injection-capable value: {bad}"
            );
        }
    }

    #[test]
    fn css_value_rejects_empty_control_and_overlong() {
        assert!(CssValue::new("").is_err());
        assert!(CssValue::new("a\u{0}b").is_err());
        assert!(CssValue::new("日本語").is_err());
        let overlong = "a".repeat(CSS_VALUE_MAX_LEN + 1);
        assert!(CssValue::new(&overlong).is_err());
        let exact = "a".repeat(CSS_VALUE_MAX_LEN);
        assert!(CssValue::new(&exact).is_ok());
    }

    #[test]
    fn token_name_accepts_valid_names() {
        assert!(TokenName::new("bg").is_ok());
        assert!(TokenName::new("space-4").is_ok());
        assert!(TokenName::new("font-size-md").is_ok());
        assert!(TokenName::new("1").is_ok());
    }

    #[test]
    fn token_name_rejects_invalid_names() {
        for bad in [
            "", "Bg", "bg ", "-bg", "bg-", "bg_muted", "bg:hover", "<bg>",
        ] {
            assert!(TokenName::new(bad).is_err(), "should reject: {bad}");
        }
        let overlong = "a".repeat(TOKEN_NAME_MAX_LEN + 1);
        assert!(TokenName::new(&overlong).is_err());
    }

    #[test]
    fn push_color_rejects_duplicate_name() {
        let mut theme = Theme::empty();
        theme.push_color("bg", "#ffffff", "#111111").unwrap();
        let err = theme.push_color("bg", "#eeeeee", "#222222").unwrap_err();
        assert_eq!(
            err,
            ThemeError::DuplicateTokenName {
                name: "bg".to_string()
            }
        );
    }

    #[test]
    fn push_space_and_typography_reject_duplicate_name() {
        let mut theme = Theme::empty();
        theme.push_space("4", "1rem").unwrap();
        assert!(theme.push_space("4", "2rem").is_err());

        theme.push_typography("font-size-md", "1rem").unwrap();
        assert!(theme.push_typography("font-size-md", "1.25rem").is_err());
    }

    #[test]
    fn var_helpers_build_expected_references() {
        assert_eq!(color_var("bg").unwrap(), "var(--fandhe-color-bg)");
        assert_eq!(space_var("4").unwrap(), "var(--fandhe-space-4)");
        assert_eq!(
            typography_var("font-size-md").unwrap(),
            "var(--fandhe-font-font-size-md)"
        );
        assert_eq!(radius_var("md").unwrap(), "var(--fandhe-radius-md)");
        assert_eq!(shadow_var("sm").unwrap(), "var(--fandhe-shadow-sm)");
        assert!(color_var("Bg").is_err());
    }

    #[test]
    fn default_theme_to_css_is_deterministic() {
        let a = Theme::default().to_css();
        let b = Theme::default().to_css();
        assert_eq!(a, b);
    }

    #[test]
    fn push_radius_and_shadow_reject_duplicate_name() {
        let mut theme = Theme::empty();
        theme.push_radius("md", "0.375rem").unwrap();
        assert!(theme.push_radius("md", "0.5rem").is_err());

        theme
            .push_shadow(
                "sm",
                "0 1px 3px rgba(0, 0, 0, 0.12)",
                "0 1px 3px rgba(0, 0, 0, 0.32)",
            )
            .unwrap();
        assert!(theme
            .push_shadow(
                "sm",
                "0 1px 3px rgba(0, 0, 0, 0.2)",
                "0 1px 3px rgba(0, 0, 0, 0.5)"
            )
            .is_err());
    }

    #[test]
    fn radii_and_shadows_appear_in_css_output_light_and_dark() {
        let mut theme = Theme::empty();
        theme.push_radius("md", "0.375rem").unwrap();
        theme
            .push_shadow(
                "sm",
                "0 1px 3px rgba(0, 0, 0, 0.12)",
                "0 1px 3px rgba(0, 0, 0, 0.32)",
            )
            .unwrap();

        let css = theme.to_css();
        assert!(css.contains("--fandhe-radius-md: 0.375rem;"));
        assert!(css.contains("--fandhe-shadow-sm: 0 1px 3px rgba(0, 0, 0, 0.12);"));
        let dark_count = css
            .matches("--fandhe-shadow-sm: 0 1px 3px rgba(0, 0, 0, 0.32);")
            .count();
        assert_eq!(
            dark_count, 2,
            "dark shadow value must appear in both the media query and data-theme blocks"
        );
    }

    #[test]
    fn theme_without_radii_or_shadows_matches_pre_606_snapshot() {
        // radii/shadows を一切 push しないテーマの `to_css()` 出力が、本イシュー
        // （#606）で追加したグループの純追加であることを保証する回帰テスト
        // （既存 `tests/theme_css.rs::custom_theme_output_matches_full_snapshot`
        // と対をなす）。
        let mut theme = Theme::empty();
        theme.push_color("bg", "#ffffff", "#111111").unwrap();

        let css = theme.to_css();
        assert!(!css.contains("--fandhe-radius-"));
        assert!(!css.contains("--fandhe-shadow-"));
    }
}
