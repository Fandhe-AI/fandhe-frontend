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
//! - [`Theme::to_css`] の出力は静的 `.css` ファイルとして配信する利用形態、または
//!   [`crate::stylesheet::StyleSheet::push_theme`] で取り込んで
//!   [`crate::stylesheet::StyleSheet::style_element`] により `<style>` 要素へ
//!   埋め込む利用形態の両方を前提とする（#605、`raw_html()` は
//!   [`crate::stylesheet::StyleSheet`] 内のレビュー済み 1 箇所に限定、
//!   [`crate`] の rustdoc 参照）。
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
//! 文脈のエスケープ）とは独立した、CSS 文脈専用の入力検証層である。既存
//! トークンを上書きする `upsert_*`（イシュー #1138）も同じ allowlist 検証を
//! 唯一の入口とし、迂回経路を持たない。

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
    /// 挙動を防ぐため fail-closed で拒否する）。`push_*` 系のみが返す。既定値の
    /// 上書き等、意図的に上書きしたい場合は `upsert_*`（イシュー #1138）を使う。
    DuplicateTokenName {
        /// 重複していたトークン名。
        name: String,
    },
    /// [`Theme::push_z_index`] / [`Theme::upsert_z_index`] に渡された値が
    /// `z-index` プロパティとして無効だった（イシュー #1423、codex-review
    /// #1705 P1 指摘）。`CssValue` の文字 allowlist は満たすが、`z-index` の
    /// 値として意味を持たない入力（例: `red` `1rem` `url(foo)`）をここで
    /// 拒否する。許可されるのは整数（符号任意）と CSS グローバル値
    /// （`auto` `inherit` `initial` `revert` `revert-layer` `unset`）のみ。
    InvalidZIndexValue {
        /// 検証に失敗した入力。
        value: String,
    },
    /// [`Theme::push_focus_ring`] / [`Theme::upsert_focus_ring`] に渡された
    /// 値が `outline-width`/`outline-offset` の寸法値として無効だった
    /// （イシュー #1424、codex-review #1707 P1 指摘）。`CssValue` の文字
    /// allowlist は満たすが、色・`rgba(...)` 等の寸法として無効な値を
    /// ここで拒否する。フォーカスリング色は本バリアントの対象外
    /// （[`Theme::push_color`]（`focus-ring` 名）が別途担う）であり、色
    /// 以外の無効値（例: `expression(...)`（`CssValue::new` 側で既に拒否）
    /// を除く任意の非寸法トークン）が寸法値としてそのまま `outline` 宣言に
    /// 混入すると、CSS の型不一致で宣言全体が computed-value time に無効
    /// となりキーボードフォーカス表示が消え得る。許可されるのは CSS
    /// `<length>`（数値 + 単位、または単位なしの `0`）と CSS グローバル値
    /// （`auto` `inherit` `initial` `revert` `revert-layer` `unset`）のみ。
    InvalidFocusRingValue {
        /// 検証に失敗した入力。
        value: String,
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
            ThemeError::InvalidZIndexValue { value } => {
                write!(
                    f,
                    "invalid z-index value (not an integer or CSS global value): {value:?}"
                )
            }
            ThemeError::InvalidFocusRingValue { value } => {
                write!(
                    f,
                    "invalid focus-ring dimension value (not a CSS length or global value): {value:?}"
                )
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
    /// ASCII 英数字・空白・`#` `%` `.` `,` `(` `)` `-` `_` `+` のみ。
    /// `+` は `z-index` 等の CSS `<integer>` が許可する正符号
    /// （`+1` `+1600` 等、イシュー #1423・codex-review #1705 P1 指摘）を
    /// 表現するために許可する。`+` 自体は宣言追加・セレクタ脱出・
    /// スクリプト実行のいずれの構文要素にもならないため、下記拒否文字の
    /// 安全性不変条件を弱めない。
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
                    || c == '+'
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
    /// 重なり順（`z-index`）のモード非依存スケール（イシュー #1423）。
    z_indices: Vec<ScaleToken>,
    /// フォーカスリングの寸法（`width`/`offset`）のモード非依存スケール
    /// （イシュー #1424）。リング色は `colors` グループ（`focus-ring`）が
    /// 担う（ダークモード追従のため）。
    focus_ring: Vec<ScaleToken>,
}

impl Default for Theme {
    /// chakra-ui の semantic token を参考にした既定パレット（色・余白・
    /// タイポグラフィの最小構成、イシュー #547 スコープ）。既定値を上書き
    /// したい場合（例: `font-body` を差し替える）は `push_*` ではなく
    /// `upsert_*`（イシュー #1138）を使う。
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
            z_indices: Vec::new(),
            focus_ring: Vec::new(),
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
        for (name, value) in DEFAULT_Z_INDICES {
            theme.push_z_index(name, value).expect(
                "既定パレットの定数は allowlist を満たすよう手動で検証済み（ユニットテストで固定）",
            );
        }
        for (name, value) in DEFAULT_FOCUS_RING {
            theme.push_focus_ring(name, value).expect(
                "既定パレットの定数は allowlist を満たすよう手動で検証済み（ユニットテストで固定）",
            );
        }

        theme
    }
}

/// 既定の色トークン（name, light, dark）。chakra-ui semantic token 参考の最小構成。
///
/// イシュー #1422 で `bg-emphasized`/`bg-overlay`・`border-subtle`/
/// `border-emphasized`・各ステータス（accent/info/success/warning/danger）の
/// `-subtle`/`-muted`/`-fg-subtle`・`neutral` 系統・`focus-ring` の 26 件を
/// 追加した（29 → 55 件）。既存 29 件の名前・値は破壊的変更を避けるため
/// 変更していない（`docs/design/color-token-system.md` §8 参照）。Radix
/// Themes の 12 段数値トークンはここでは採用せず、chakra-ui 風の直接命名
/// （semantic 名）のまま拡張する方針を維持する（同文書 §6 の非採用判断）。
/// light/dark 双方のコントラスト比は `contrast` テストモジュール
/// （本ファイル下部）で WCAG 2.x 相対輝度を計算して回帰検証する。
const DEFAULT_COLORS: &[(&str, &str, &str)] = &[
    ("bg", "#ffffff", "#111111"),
    ("bg-subtle", "#f7f7f7", "#1a1a1a"),
    ("bg-muted", "#eeeeee", "#242424"),
    // chakra `bg.emphasized` / Radix gray 4-5 相当（hover 等の強調背景）。
    ("bg-emphasized", "#e2e2e2", "#2e2e2e"),
    // dialog/drawer の backdrop（イシュー #1422 時点では未使用、部品側の
    // `rgba(0, 0, 0, 0.4)` リテラルからの置換は Phase 1 部品 issue へ申し送り。
    // `docs/design/color-token-system.md` §6 参照）。
    ("bg-overlay", "rgba(0, 0, 0, 0.4)", "rgba(0, 0, 0, 0.6)"),
    ("fg", "#111111", "#f7f7f7"),
    ("fg-muted", "#4a4a4a", "#cccccc"),
    ("fg-subtle", "#767676", "#a3a3a3"),
    ("border", "#d9d9d9", "#3a3a3a"),
    ("border-muted", "#e6e6e6", "#2a2a2a"),
    // chakra `border.subtle` / Radix gray 6 相当（`border-muted` よりさらに淡い）。
    ("border-subtle", "#f0f0f0", "#202020"),
    // chakra `border.emphasized` / Radix gray 8 相当（hover 時の強調枠線）。
    ("border-emphasized", "#b3b3b3", "#525252"),
    ("accent", "#3182ce", "#4299e1"),
    ("accent-emphasized", "#2b6cb0", "#63b3ed"),
    ("accent-fg", "#ffffff", "#0b1720"),
    // chakra `<palette>.subtle` / Radix accent 3 相当（淡色背景）。
    // `tree-view.rs`/`menubar.rs`/`navigation-menu.rs`/`toolbar.rs` が
    // フォールバック無しで参照していた未定義トークンをここで正式定義する
    // （イシュー #1422、既存の透明描画バグを閉じる）。
    ("accent-subtle", "#ebf8ff", "#1a2b3d"),
    // chakra `<palette>.muted` / Radix accent 5-6 相当（淡色枠線・hover 淡色）。
    ("accent-muted", "#bee3f8", "#2c4a66"),
    // chakra `<palette>.fg` / Radix accent 11 相当（`accent-subtle` 背景上の
    // 本文色。既存 `accent-fg` は solid 背景上の文字色＝ chakra
    // `<palette>.contrast` に相当するため別名が必要、イシュー #1422）。
    ("accent-fg-subtle", "#1a4971", "#90cdf4"),
    ("info", "#3182ce", "#63b3ed"),
    ("info-emphasized", "#2b6cb0", "#90cdf4"),
    ("info-fg", "#ffffff", "#0b1720"),
    ("info-subtle", "#ebf8ff", "#1a2b3d"),
    ("info-muted", "#bee3f8", "#2c4a66"),
    ("info-fg-subtle", "#1a4971", "#90cdf4"),
    ("success", "#2f855a", "#68d391"),
    ("success-emphasized", "#276749", "#9ae6b4"),
    ("success-fg", "#ffffff", "#0b1a12"),
    ("success-subtle", "#f0fff4", "#122a1c"),
    ("success-muted", "#c6f6d5", "#1c4a32"),
    ("success-fg-subtle", "#1c4a32", "#9ae6b4"),
    ("warning", "#b7791f", "#f6ad55"),
    ("warning-emphasized", "#975a16", "#fbd38d"),
    ("warning-fg", "#ffffff", "#1a1203"),
    ("warning-subtle", "#fffaf0", "#2e2410"),
    ("warning-muted", "#feebc8", "#4a3510"),
    ("warning-fg-subtle", "#5a3c0a", "#fbd38d"),
    ("danger", "#c53030", "#fc8181"),
    ("danger-emphasized", "#9b2c2c", "#feb2b2"),
    ("danger-fg", "#ffffff", "#1a0b0b"),
    ("danger-subtle", "#fff5f5", "#2e1616"),
    ("danger-muted", "#fed7d7", "#4a1f1f"),
    ("danger-fg-subtle", "#6b1414", "#feb2b2"),
    // neutral（gray）系統。chakra `gray` colorPalette / Radix gray 9-12 相当。
    // ステータス色 5 系統（accent/info/success/warning/danger）に対する
    // 5 系統目としての中立色（イシュー #1422、チェックリスト
    // 「info / success / warning / error / neutral」）。
    ("neutral", "#718096", "#a0aec0"),
    ("neutral-emphasized", "#4a5568", "#cbd5e0"),
    ("neutral-fg", "#ffffff", "#0b1720"),
    ("neutral-subtle", "#f7f7f7", "#1a1a1a"),
    ("neutral-muted", "#e2e8f0", "#2d3748"),
    ("neutral-fg-subtle", "#333333", "#d4d4d4"),
    // フォーカスリング（イシュー #1422 で追加、#1424 で値確定）。
    // `date-input.rs` が `var(--fandhe-color-focus-ring,
    // var(--fandhe-color-accent))` フォールバック付きで参照していた
    // 未定義トークンをここで正式定義する（#1422）。値は既存の
    // accent（light）/ info dark（dark）と同値だが、`accent` トークンの
    // 意味（アクセントカラー全般）から独立させることで、フォーカスリング
    // 色だけを差し替えたい場合に他の accent 用途へ波及しないよう #1424 で
    // dark 値を確定した（`docs/design/pre-styled-ui-focus-ring-and-size-conventions.md`
    // 参照。base 取り込みで #1422 側の暫定値との重複エントリを解消し、
    // このエントリ 1 件に一本化した）。
    ("focus-ring", "#3182ce", "#63b3ed"),
    // 系列配色トークン（イシュー #846）。chakra-ui の chart カラースケール
    // （blue/orange/green/purple/pink/teal 系統）を参考にした 6 色。light/dark
    // ともに `bg`/`bg-subtle` 背景（本ファイル冒頭の DEFAULT_COLORS 参照）に
    // 対し可読なコントラストを確保する値を選定した。
    // `crates/pre-styled-ui/src/charts/mod.rs::series_color_var` が
    // `var(--fandhe-color-chart-N)` として参照する。
    ("chart-1", "#3182ce", "#63b3ed"),
    ("chart-2", "#dd6b20", "#f6ad55"),
    ("chart-3", "#2f855a", "#68d391"),
    ("chart-4", "#805ad5", "#b794f4"),
    ("chart-5", "#d53f8c", "#f687b3"),
    ("chart-6", "#00a3c4", "#76e4f7"),
];

/// 既定の角丸トークン（name, value）。モード非依存。既存 styled 部品
/// （Button/Badge/Spinner/Alert/Card、イシュー #550）のリテラル値をそのまま
/// 吸収する初期スケール（イシュー #606）。イシュー #1423 で `none`/`xs`/`2xl`
/// を純追加し 5 段 → 8 段へ拡充した（chakra-ui `xs`〜`2xl` 相当、Radix
/// `--radius-1..6` との対応は `docs/design/pre-styled-ui-scale-tokens.md` 参照。
/// 既存 5 段の名前・値は不変のため、この追加は既存出力を壊さない）。
const DEFAULT_RADII: &[(&str, &str)] = &[
    ("none", "0"),
    ("xs", "0.125rem"),
    ("sm", "0.25rem"),
    ("md", "0.375rem"),
    ("lg", "0.5rem"),
    ("xl", "0.75rem"),
    ("2xl", "1rem"),
    ("full", "9999px"),
];

/// 既定の影トークン（name, light, dark）。ダークモードは light 比で不透明度を
/// 上げ、暗背景上でも輪郭が視認できるようにする（イシュー #606）。`sm` の
/// light 値は Card Elevated の既存リテラル（イシュー #550）を踏襲する。
/// イシュー #1423 で `xl`/`2xl` を末尾へ純追加し 4 段 → 6 段へ拡充した
/// （chakra-ui `xs`〜`2xl`・Radix `--shadow-1..6` と同段数。ダーク値は既存
/// 規則〔light 比で不透明度を上げる〕を踏襲し、overlay 系部品が別途持つ
/// `border` による境界確保〔Radix 方式〕への切り替えは色トークン確定後の
/// 再評価事項として `docs/design/pre-styled-ui-scale-tokens.md` に明記する）。
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
    (
        "xl",
        "0 20px 25px rgba(0, 0, 0, 0.2)",
        "0 20px 25px rgba(0, 0, 0, 0.5)",
    ),
    (
        "2xl",
        "0 25px 50px rgba(0, 0, 0, 0.25)",
        "0 25px 50px rgba(0, 0, 0, 0.55)",
    ),
];

/// 既定の余白トークン（name, value）。chakra 風のスケール。モード非依存。
/// イシュー #1423 で `0-5`/`1-5`/`2-5`（4px 格子の半刻み）と `20`/`24`
/// （大きめの余白）を純追加し 10 段 → 15 段へ拡充した。既存 10 段の名前・
/// 値は不変。[`TokenName`] は `.` を許可しないため chakra の `0.5`/`1.5`/
/// `2.5` 相当は `-` 区切り（`0-5`/`1-5`/`2-5`）で表記する
/// （`docs/design/pre-styled-ui-scale-tokens.md` 参照）。
const DEFAULT_SPACES: &[(&str, &str)] = &[
    ("0-5", "0.125rem"),
    ("1", "0.25rem"),
    ("1-5", "0.375rem"),
    ("2", "0.5rem"),
    ("2-5", "0.625rem"),
    ("3", "0.75rem"),
    ("4", "1rem"),
    ("5", "1.25rem"),
    ("6", "1.5rem"),
    ("8", "2rem"),
    ("10", "2.5rem"),
    ("12", "3rem"),
    ("16", "4rem"),
    ("20", "5rem"),
    ("24", "6rem"),
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

/// 既定の z-index トークン（name, value）。モード非依存の新規グループ
/// （イシュー #1423）。chakra-ui の `hide`〜`max` を参考にした 100 刻みの
/// 重なり順スケールで、`dropdown < sticky < popover < overlay < modal <
/// skip-nav < toast < tooltip` の順を満たす（dialog/drawer は同段
/// `overlay`/`modal` とし、同時表示時の前後関係は DOM 順に委ねる。chakra
/// も同段）。値の割り当て根拠・部品ごとの適用予定（後続 Phase の各部品
/// issue が消し込む）は `docs/design/pre-styled-ui-scale-tokens.md` 参照。
/// `toast.rs` が既に使っていた未宣言変数 `--fandhe-z-index-toast` は
/// この正式トークン化を受けてテーマ側の宣言を追加したが、`toast.rs` 側の
/// `var(--fandhe-z-index-toast, 9999)` fallback は維持している
/// （codex-review #1705 P1 指摘: `Theme::empty()` から必要トークンのみ
/// 構築する既存利用者・`toast::stylesheet()` 単独利用者ではテーマ CSS が
/// 注入されず未定義のままになり得るため、公開クレートの既存 CSS 契約を
/// 壊さないよう fallback を残す）。
const DEFAULT_Z_INDICES: &[(&str, &str)] = &[
    ("hide", "-1"),
    ("base", "0"),
    ("docked", "10"),
    ("dropdown", "1000"),
    ("sticky", "1100"),
    ("popover", "1200"),
    ("overlay", "1300"),
    ("modal", "1400"),
    ("skip-nav", "1500"),
    ("toast", "1600"),
    ("tooltip", "1700"),
    ("max", "2147483647"),
];

/// 既定のフォーカスリング寸法トークン（name, value）。モード非依存
/// （イシュー #1424）。`width`/`offset` を分離トークン化することで、
/// `crates/pre-styled-ui/src/recipe.rs::focus_ring_declarations` が
/// 太さ・オフセットを個別に参照でき、太さのみ・オフセットのみの変更が
/// 1 箇所で完結する（規約の詳細は
/// `docs/design/pre-styled-ui-focus-ring-and-size-conventions.md` 参照）。
/// リング色は色トークン（`DEFAULT_COLORS` の `focus-ring`）で別管理する
/// （ダークモード追従が必要なため `colors` グループが担う）。
const DEFAULT_FOCUS_RING: &[(&str, &str)] = &[("width", "2px"), ("offset", "2px")];

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
            z_indices: Vec::new(),
            focus_ring: Vec::new(),
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

    /// モード非依存の重なり順（`z-index`）トークンを追加する（イシュー #1423）。
    ///
    /// `fandhe-frontend-pre-styled-ui` の overlay 系 styled 部品（Menu/Popover/
    /// Dialog/Drawer/Toast/Tooltip 等）が `z-index: var(--fandhe-z-index-<name>)`
    /// として参照する想定のトークン。既定スケール（[`DEFAULT_Z_INDICES`]）は
    /// dropdown < sticky < popover < overlay < modal < skip-nav < toast <
    /// tooltip の重なり順を満たす。
    ///
    /// # Errors
    ///
    /// [`Theme::push_color`] と同様（`name`/`value` の検証・重複拒否）。
    /// # Errors
    ///
    /// - `name` が [`TokenName`] の命名規則を満たさない場合
    /// - `value` が `z-index` として無効な場合（[`ThemeError::InvalidZIndexValue`]。
    ///   整数と CSS グローバル値のみ許可、詳細は同バリアントの rustdoc 参照）
    /// - `name` が z-indices グループ内で既に登録済みの場合（[`ThemeError::DuplicateTokenName`]）
    pub fn push_z_index(&mut self, name: &str, value: &str) -> Result<(), ThemeError> {
        let name = TokenName::new(name)?;
        let value = validate_z_index_value(value)?;

        if self.z_indices.iter().any(|t| t.name == name) {
            return Err(ThemeError::DuplicateTokenName {
                name: name.as_str().to_string(),
            });
        }

        self.z_indices.push(ScaleToken { name, value });
        Ok(())
    }

    /// モード非依存のフォーカスリング寸法（`width`/`offset`）トークンを追加
    /// する（イシュー #1424）。`crates/pre-styled-ui/src/recipe.rs::focus_ring_declarations`
    /// が `outline-width`/`outline-offset` の値として参照する想定のトークン。
    /// リング色は [`Theme::push_color`]（`focus-ring` 名）が担う。
    ///
    /// # Errors
    ///
    /// - `name` が [`TokenName`] の命名規則を満たさない場合
    /// - `value` が `outline-width`/`outline-offset` の寸法値として無効な
    ///   場合（[`ThemeError::InvalidFocusRingValue`]。CSS `<length>` と
    ///   CSS グローバル値のみ許可、詳細は同バリアントの rustdoc 参照。
    ///   codex-review #1707 P1 指摘対応）
    /// - `name` が focus_ring グループ内で既に登録済みの場合（[`ThemeError::DuplicateTokenName`]）
    pub fn push_focus_ring(&mut self, name: &str, value: &str) -> Result<(), ThemeError> {
        let name = TokenName::new(name)?;
        let value = validate_focus_ring_value(value)?;

        if self.focus_ring.iter().any(|t| t.name == name) {
            return Err(ThemeError::DuplicateTokenName {
                name: name.as_str().to_string(),
            });
        }

        self.focus_ring.push(ScaleToken { name, value });
        Ok(())
    }

    /// ライト/ダーク値を持つ色トークンを追加、または既存トークンを上書きする
    /// （イシュー #1138）。
    ///
    /// [`Theme::push_color`] は同名トークンを [`ThemeError::DuplicateTokenName`]
    /// で拒否するため、[`Theme::default`] の既定パレットを差し替える正規経路が
    /// 存在しなかった（イシュー #1118 で判明した欠落）。本メソッドは `name` が
    /// colors グループに既に存在する場合は当該位置の値を in-place 置換し
    /// （挿入順＝出力順を変えない）、存在しない場合は末尾へ追加する
    /// （[`Theme::push_color`] と同じ挙動）。
    ///
    /// 検証は [`Theme::push_color`] と同一の allowlist（[`TokenName::new`] /
    /// [`CssValue::new`]）を必ず通過させ、迂回経路を作らない
    /// （本ファイル冒頭「セキュリティ上の不変条件」参照）。全引数の検証を
    /// 完了してから書き込むため、検証失敗時は `self` を一切変更しない。
    ///
    /// `ThemeError::DuplicateTokenName` を返すことはない。
    ///
    /// # Errors
    ///
    /// `name` / `light` / `dark` のいずれかが allowlist 検証を通過しない場合。
    pub fn upsert_color(&mut self, name: &str, light: &str, dark: &str) -> Result<(), ThemeError> {
        upsert_dual(&mut self.colors, name, light, dark)
    }

    /// モード非依存の余白トークンを追加、または既存トークンを上書きする
    /// （イシュー #1138）。セマンティクスは [`Theme::upsert_color`] 参照。
    ///
    /// # Errors
    ///
    /// `name` / `value` のいずれかが allowlist 検証を通過しない場合。
    pub fn upsert_space(&mut self, name: &str, value: &str) -> Result<(), ThemeError> {
        upsert_scale(&mut self.spaces, name, value)
    }

    /// モード非依存のタイポグラフィトークンを追加、または既存トークンを上書き
    /// する（イシュー #1138）。イシュー #1118 の実シナリオ（`font-body` を
    /// 日本語フォントスタックへ差し替える）はこのメソッドを使う。
    /// セマンティクスは [`Theme::upsert_color`] 参照。
    ///
    /// # Errors
    ///
    /// `name` / `value` のいずれかが allowlist 検証を通過しない場合。
    pub fn upsert_typography(&mut self, name: &str, value: &str) -> Result<(), ThemeError> {
        upsert_scale(&mut self.typography, name, value)
    }

    /// モード非依存の角丸（`border-radius`）トークンを追加、または既存
    /// トークンを上書きする（イシュー #1138）。セマンティクスは
    /// [`Theme::upsert_color`] 参照。
    ///
    /// # Errors
    ///
    /// `name` / `value` のいずれかが allowlist 検証を通過しない場合。
    pub fn upsert_radius(&mut self, name: &str, value: &str) -> Result<(), ThemeError> {
        upsert_scale(&mut self.radii, name, value)
    }

    /// ライト/ダーク値を持つ影（`box-shadow`）トークンを追加、または既存
    /// トークンを上書きする（イシュー #1138）。セマンティクスは
    /// [`Theme::upsert_color`] 参照。
    ///
    /// # Errors
    ///
    /// `name` / `light` / `dark` のいずれかが allowlist 検証を通過しない場合。
    pub fn upsert_shadow(&mut self, name: &str, light: &str, dark: &str) -> Result<(), ThemeError> {
        upsert_dual(&mut self.shadows, name, light, dark)
    }

    /// モード非依存の重なり順（`z-index`）トークンを追加、または既存
    /// トークンを上書きする（イシュー #1423）。セマンティクスは
    /// [`Theme::upsert_color`] 参照。
    ///
    /// # Errors
    ///
    /// `name` / `value` のいずれかが allowlist 検証を通過しない場合。
    ///
    /// # Errors
    ///
    /// `name` / `value` のいずれかが検証を通過しない場合。`value` の検証は
    /// [`Theme::push_z_index`] と同一（[`ThemeError::InvalidZIndexValue`]）。
    pub fn upsert_z_index(&mut self, name: &str, value: &str) -> Result<(), ThemeError> {
        let name = TokenName::new(name)?;
        let value = validate_z_index_value(value)?;

        if let Some(existing) = self.z_indices.iter_mut().find(|t| t.name == name) {
            existing.value = value;
        } else {
            self.z_indices.push(ScaleToken { name, value });
        }
        Ok(())
    }

    /// モード非依存のフォーカスリング寸法（`width`/`offset`）トークンを追加、
    /// または既存トークンを上書きする（イシュー #1424）。セマンティクスは
    /// [`Theme::upsert_color`] 参照。
    ///
    /// # Errors
    ///
    /// `name` / `value` のいずれかが検証を通過しない場合。`value` の検証は
    /// [`Theme::push_focus_ring`] と同一（[`ThemeError::InvalidFocusRingValue`]。
    /// codex-review #1707 P1 指摘対応）。
    pub fn upsert_focus_ring(&mut self, name: &str, value: &str) -> Result<(), ThemeError> {
        let name = TokenName::new(name)?;
        let value = validate_focus_ring_value(value)?;

        if let Some(existing) = self.focus_ring.iter_mut().find(|t| t.name == name) {
            existing.value = value;
        } else {
            self.focus_ring.push(ScaleToken { name, value });
        }
        Ok(())
    }

    /// テーマを決定的なプレーン CSS 文字列へ変換する。
    ///
    /// 出力構造（固定順、`docs` は伴わず本 rustdoc が正）:
    ///
    /// 1. `:root { color-scheme: light dark; --fandhe-... }`（light 値、
    ///    colors → spaces → typography → radii → shadows → z-indices →
    ///    focus-ring の順。radii/z-indices/focus-ring はモード非依存のため
    ///    1 値、shadows は light 値をここに出力する。イシュー #606 で追加した
    ///    radii/shadows、イシュー #1423 で追加した z-indices、イシュー #1424
    ///    で追加した focus-ring（寸法。リング色は `colors` グループの
    ///    `focus-ring` エントリが担う）はいずれも末尾に純追加する構成のため、
    ///    当該グループを push しないテーマの出力は追加前とバイト同一になる）
    /// 2. `:root[data-theme="light"] { color-scheme: light; }`
    /// 3. `@media (prefers-color-scheme: dark) { :root:not([data-theme="light"]) { ... } }`
    ///    （dark 値。OS 設定追従。colors → shadows の順）
    /// 4. `:root[data-theme="dark"] { ... }`（dark 値。明示指定は常に勝つ。
    ///    3 と同特異度のため、末尾に置く出力順序でメディアクエリより優先させる）
    ///
    /// 3 と 4 の dark トークン列は同一の内部ヘルパ（[`Theme::write_dark_declarations`]）
    /// から生成し、二重管理による乖離を構造的に防ぐ。
    ///
    /// 呼び出し元は返り値を静的 `.css` ファイルとして配信する、または
    /// [`crate::stylesheet::StyleSheet::push_theme`] へ渡して `<style>` 要素へ
    /// 埋め込む（#605、[`crate`] 冒頭の不変条件を参照）。
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
        for token in &self.z_indices {
            out.push_str(&format!(
                "  {VAR_PREFIX}-z-index-{}: {};\n",
                token.name.as_str(),
                token.value.as_str()
            ));
        }
        for token in &self.focus_ring {
            out.push_str(&format!(
                "  {VAR_PREFIX}-focus-ring-{}: {};\n",
                token.name.as_str(),
                token.value.as_str()
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

/// [`Theme::push_z_index`] / [`Theme::upsert_z_index`] 共通の値検証ロジック
/// （イシュー #1423、codex-review #1705 P1 指摘）。
///
/// [`CssValue::new`] の文字 allowlist 検証に加え、`z-index` プロパティとして
/// 意味を持つ値のみを許可する: 符号任意の整数（`-1` `0` `1600` `+1` 等）、または
/// CSS グローバル値（`auto` `inherit` `initial` `revert` `revert-layer`
/// `unset`）。`red` `1rem` `url(foo)` のような文字集合は満たすが `z-index` と
/// しては無効な値をここで拒否する（ブラウザに宣言ごと破棄され重なり順が
/// 失われるのを防ぐ）。
///
/// # Errors
///
/// [`CssValue::new`] が失敗した場合は [`ThemeError::InvalidCssValue`]、文字集合は
/// 満たすが整数でも既知のグローバル値でもない場合は
/// [`ThemeError::InvalidZIndexValue`] を返す。
fn validate_z_index_value(value: &str) -> Result<CssValue, ThemeError> {
    let css_value = CssValue::new(value)?;
    let s = css_value.as_str();

    let numeric_part = s
        .strip_prefix('-')
        .or_else(|| s.strip_prefix('+'))
        .unwrap_or(s);
    let is_integer = !numeric_part.is_empty() && numeric_part.bytes().all(|b| b.is_ascii_digit());
    let is_global_keyword = matches!(
        s,
        "auto" | "inherit" | "initial" | "revert" | "revert-layer" | "unset"
    );

    if is_integer || is_global_keyword {
        Ok(css_value)
    } else {
        Err(ThemeError::InvalidZIndexValue {
            value: value.to_string(),
        })
    }
}

/// CSS `<length>` の単位として認める文字列（降順ではなく列挙順、
/// [`validate_focus_ring_value`] 専用）。`ch`/`ex` 等の希少単位も含め、
/// MDN `<length>` の代表的な絶対/相対単位を allowlist する。
const CSS_LENGTH_UNITS: &[&str] = &[
    "px", "rem", "em", "ch", "ex", "vh", "vw", "vmin", "vmax", "pt", "pc", "in", "cm", "mm", "q",
];

/// [`Theme::push_focus_ring`] / [`Theme::upsert_focus_ring`] 共通の値検証
/// （イシュー #1424、codex-review #1707 P1 指摘対応）。
///
/// フォーカスリングの寸法トークンは `crates/pre-styled-ui/src/recipe.rs::focus_ring_declarations`
/// が `outline-width`/`outline-offset` の値として直接埋め込む。`CssValue`
/// の文字 allowlist は満たすが CSS `<length>` としては無効な値（色・
/// `rgba(...)` 等）を許してしまうと、`outline` 宣言全体が computed-value
/// time に無効となりキーボードフォーカス表示が消える
/// （[`ThemeError::InvalidFocusRingValue`] rustdoc 参照）。
///
/// # 許可する値
///
/// - 単位なしの `0`（`-0`/`+0` を含む。CSS が唯一単位省略を許す長さ）
/// - 符号任意の数値（整数・小数、`1.5` `.5` 形式のいずれも可）+
///   [`CSS_LENGTH_UNITS`] のいずれかの単位（大文字小文字を区別しない）
/// - CSS グローバル値（`auto` `inherit` `initial` `revert` `revert-layer`
///   `unset`）
///
/// # Errors
///
/// [`CssValue::new`] が失敗した場合は [`ThemeError::InvalidCssValue`]、
/// 文字集合は満たすが上記のいずれにも該当しない場合は
/// [`ThemeError::InvalidFocusRingValue`] を返す。
fn validate_focus_ring_value(value: &str) -> Result<CssValue, ThemeError> {
    let css_value = CssValue::new(value)?;
    let s = css_value.as_str();

    let is_global_keyword = matches!(
        s,
        "auto" | "inherit" | "initial" | "revert" | "revert-layer" | "unset"
    );

    let numeric_part = s
        .strip_prefix('-')
        .or_else(|| s.strip_prefix('+'))
        .unwrap_or(s);
    let is_bare_zero = numeric_part == "0";

    // 単位を後方一致（大文字小文字無視）で剥がし、残りが妥当な数値
    // （整数・小数点付き、空でない）であることを確認する。
    let is_length_with_unit = CSS_LENGTH_UNITS.iter().any(|unit| {
        let Some(rest) = strip_suffix_ignore_ascii_case(numeric_part, unit) else {
            return false;
        };
        is_valid_css_number(rest)
    });

    if is_bare_zero || is_length_with_unit || is_global_keyword {
        Ok(css_value)
    } else {
        Err(ThemeError::InvalidFocusRingValue {
            value: value.to_string(),
        })
    }
}

/// ASCII 大文字小文字を無視して `s` の末尾から `suffix` を剥がす
/// （[`validate_focus_ring_value`] 専用の内部ヘルパ。`str::strip_suffix`
/// は大文字小文字を区別するため、単位表記の揺れ（`PX`/`Px` 等）を
/// 吸収するために自前実装する）。
fn strip_suffix_ignore_ascii_case<'a>(s: &'a str, suffix: &str) -> Option<&'a str> {
    if s.len() < suffix.len() {
        return None;
    }
    let (rest, tail) = s.split_at(s.len() - suffix.len());
    if tail.eq_ignore_ascii_case(suffix) {
        Some(rest)
    } else {
        None
    }
}

/// CSS `<number>` として妥当な文字列か判定する（符号なし。呼び出し元で
/// 符号を既に剥がしている前提）。整数部・小数部の少なくとも一方が必須で、
/// 小数点は高々 1 個まで（[`validate_focus_ring_value`] 専用）。
fn is_valid_css_number(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    let mut seen_dot = false;
    let mut has_digit = false;
    for c in s.chars() {
        if c == '.' {
            if seen_dot {
                return false;
            }
            seen_dot = true;
        } else if c.is_ascii_digit() {
            has_digit = true;
        } else {
            return false;
        }
    }
    has_digit
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

/// [`Theme::upsert_space`] / [`Theme::upsert_typography`] / [`Theme::upsert_radius`]
/// 共通の検証・upsert ロジック（イシュー #1138）。[`push_scale`] と同一の
/// allowlist 検証を経てから、既存位置があれば in-place 置換・なければ末尾
/// 追加する（挿入順＝出力順の決定性を保つ）。
fn upsert_scale(target: &mut Vec<ScaleToken>, name: &str, value: &str) -> Result<(), ThemeError> {
    let name = TokenName::new(name)?;
    let value = CssValue::new(value)?;

    if let Some(existing) = target.iter_mut().find(|t| t.name == name) {
        existing.value = value;
    } else {
        target.push(ScaleToken { name, value });
    }
    Ok(())
}

/// [`Theme::upsert_color`] / [`Theme::upsert_shadow`] 共通の検証・upsert
/// ロジック（イシュー #1138）。[`Theme::push_color`] と同一の allowlist 検証を
/// 経てから、既存位置があれば in-place 置換・なければ末尾追加する
/// （挿入順＝出力順の決定性を保つ）。
fn upsert_dual(
    target: &mut Vec<DualModeToken>,
    name: &str,
    light: &str,
    dark: &str,
) -> Result<(), ThemeError> {
    let name = TokenName::new(name)?;
    let light = CssValue::new(light)?;
    let dark = CssValue::new(dark)?;

    if let Some(existing) = target.iter_mut().find(|t| t.name == name) {
        existing.light = light;
        existing.dark = dark;
    } else {
        target.push(DualModeToken { name, light, dark });
    }
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

/// z-index トークン名から `var(--fandhe-z-index-<name>)` 参照を組み立てる
/// （イシュー #1423）。styled 部品が `z-index` の値として参照する想定。
///
/// # Errors
///
/// [`color_var`] と同様。
pub fn z_index_var(name: &str) -> Result<String, ThemeError> {
    let name = TokenName::new(name)?;
    Ok(format!("var({VAR_PREFIX}-z-index-{})", name.as_str()))
}

/// フォーカスリング寸法トークン名から `var(--fandhe-focus-ring-<name>)`
/// 参照を組み立てる（イシュー #1424）。
/// `crates/pre-styled-ui/src/recipe.rs::focus_ring_declarations` が
/// `outline-width`/`outline-offset` の値として参照する想定。リング色は
/// [`color_var`]（`focus-ring` 名）を使う。
///
/// # Errors
///
/// [`color_var`] と同様。
pub fn focus_ring_var(name: &str) -> Result<String, ThemeError> {
    let name = TokenName::new(name)?;
    Ok(format!("var({VAR_PREFIX}-focus-ring-{})", name.as_str()))
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
        // `+` は z-index 等の正符号付き整数のために許可する
        // （イシュー #1423・codex-review #1705 P1 指摘）。
        assert!(CssValue::new("+1").is_ok());
        assert!(CssValue::new("+1600").is_ok());
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

    // イシュー #1138: upsert API のユニットテスト。

    #[test]
    fn upsert_color_inserts_when_absent() {
        let mut theme = Theme::empty();
        theme.upsert_color("bg", "#ffffff", "#111111").unwrap();
        let css = theme.to_css();
        assert!(css.contains("--fandhe-color-bg: #ffffff;"));
    }

    #[test]
    fn upsert_color_overwrites_in_place_preserving_order() {
        let mut theme = Theme::empty();
        theme.push_color("bg", "#ffffff", "#111111").unwrap();
        theme.push_color("fg", "#111111", "#f7f7f7").unwrap();

        // 1 件目（bg）を upsert で上書きしても、to_css の出力順（挿入順）は
        // 変わらない（bg が fg より先に現れ続ける）ことを確認する。
        theme.upsert_color("bg", "#eeeeee", "#222222").unwrap();

        let css = theme.to_css();
        let bg_pos = css.find("--fandhe-color-bg: #eeeeee;").unwrap();
        let fg_pos = css.find("--fandhe-color-fg: #111111;").unwrap();
        assert!(bg_pos < fg_pos, "upsert しても挿入順が保たれること");
        assert!(!css.contains("#ffffff"), "旧値は残らないこと");
    }

    #[test]
    fn upsert_rejects_invalid_value_and_name_without_mutation() {
        let mut theme = Theme::empty();
        theme.push_color("bg", "#ffffff", "#111111").unwrap();
        let before = theme.to_css();

        assert!(theme.upsert_color("bg", "red;}", "#222222").is_err());
        assert!(theme
            .upsert_color("Bg Invalid", "#eeeeee", "#222222")
            .is_err());

        // 検証失敗時は state を一切変更しない（部分書き込みの排除）。
        assert_eq!(theme.to_css(), before);
    }

    #[test]
    fn upsert_typography_overrides_default_font_body() {
        // イシュー #1118 の実シナリオ: Theme::default() の既定 font-body を
        // 日本語フォントスタックへ差し替える。
        let mut theme = Theme::default();
        theme
            .upsert_typography("font-body", "Noto Sans JP, system-ui, sans-serif")
            .unwrap();

        let css = theme.to_css();
        assert!(css.contains("--fandhe-font-font-body: Noto Sans JP, system-ui, sans-serif;"));
        assert!(!css.contains("system-ui, -apple-system, sans-serif"));
    }

    #[test]
    fn upsert_space_and_radius_overwrite_existing_value() {
        let mut theme = Theme::empty();
        theme.push_space("4", "1rem").unwrap();
        theme.upsert_space("4", "1.5rem").unwrap();
        assert!(theme.to_css().contains("--fandhe-space-4: 1.5rem;"));

        theme.push_radius("md", "0.375rem").unwrap();
        theme.upsert_radius("md", "0.5rem").unwrap();
        assert!(theme.to_css().contains("--fandhe-radius-md: 0.5rem;"));
    }

    #[test]
    fn upsert_shadow_overwrites_existing_light_and_dark_value() {
        let mut theme = Theme::empty();
        theme
            .push_shadow(
                "sm",
                "0 1px 3px rgba(0, 0, 0, 0.12)",
                "0 1px 3px rgba(0, 0, 0, 0.32)",
            )
            .unwrap();
        theme
            .upsert_shadow(
                "sm",
                "0 1px 3px rgba(0, 0, 0, 0.2)",
                "0 1px 3px rgba(0, 0, 0, 0.5)",
            )
            .unwrap();

        let css = theme.to_css();
        assert!(css.contains("--fandhe-shadow-sm: 0 1px 3px rgba(0, 0, 0, 0.2);"));
        let dark_count = css
            .matches("--fandhe-shadow-sm: 0 1px 3px rgba(0, 0, 0, 0.5);")
            .count();
        assert_eq!(dark_count, 2);
    }

    #[test]
    fn upsert_never_returns_duplicate_token_name_error() {
        let mut theme = Theme::empty();
        theme.push_color("bg", "#ffffff", "#111111").unwrap();
        // upsert は既存名でも Err にならない（DuplicateTokenName を返さない契約）。
        assert!(theme.upsert_color("bg", "#eeeeee", "#222222").is_ok());
    }

    // --- WCAG コントラスト回帰（イシュー #1422） ---
    //
    // `DEFAULT_COLORS` の主要な前景/背景ペアが WCAG 2.x のコントラスト比
    // 閾値（本文 4.5:1・大字/UI 部品 3:1）を light/dark 双方で満たすことを
    // 固定する。値の調整自体は `docs/design/color-token-system.md` §7/§8 の
    // 記録対象であり、本テストは「閾値を緩めずに検証する」歯止めを担う
    // （`docs/spec` 側の受け入れ基準・親 #1421 のチェックリスト項目）。
    // 外部クレート依存ゼロ（std のみ）で相対輝度・コントラスト比を計算する。

    /// `#rrggbb` の 1 チャンネル値を sRGB → 線形光へ変換する（WCAG 2.x 定義）。
    fn linearize_channel(c: u8) -> f64 {
        let c = f64::from(c) / 255.0;
        if c <= 0.039_28 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    }

    /// `#rrggbb` 形式の 6 桁 hex から WCAG 2.x 相対輝度を計算する。
    ///
    /// # Panics
    ///
    /// テスト専用ヘルパのため、`#` 始まり 6 桁 hex 以外の入力（本ファイルの
    /// `DEFAULT_COLORS` に含まれる `rgba(...)` 値等）を渡すとパニックする。
    /// 呼び出し側でペア検証対象から `bg-overlay` を除外する。
    fn relative_luminance(hex: &str) -> f64 {
        let hex = hex.strip_prefix('#').expect("hex color must start with #");
        assert_eq!(hex.len(), 6, "expected 6-digit hex color, got: {hex}");
        let r = u8::from_str_radix(&hex[0..2], 16).expect("valid hex red channel");
        let g = u8::from_str_radix(&hex[2..4], 16).expect("valid hex green channel");
        let b = u8::from_str_radix(&hex[4..6], 16).expect("valid hex blue channel");
        0.2126 * linearize_channel(r)
            + 0.7152 * linearize_channel(g)
            + 0.0722 * linearize_channel(b)
    }

    /// WCAG 2.x のコントラスト比（明度の高い方 / 低い方）を計算する。
    fn contrast_ratio(a: &str, b: &str) -> f64 {
        let la = relative_luminance(a);
        let lb = relative_luminance(b);
        let (l1, l2) = if la > lb { (la, lb) } else { (lb, la) };
        (l1 + 0.05) / (l2 + 0.05)
    }

    /// `DEFAULT_COLORS` からトークン名でライト/ダーク値を引く（テスト専用）。
    fn default_color(name: &str) -> (&'static str, &'static str) {
        DEFAULT_COLORS
            .iter()
            .find(|(n, _, _)| *n == name)
            .map(|(_, light, dark)| (*light, *dark))
            .unwrap_or_else(|| panic!("DEFAULT_COLORS に {name} が存在しない"))
    }

    /// 本文相当ペア（4.5:1 以上）。5 ステータス系統 + neutral の
    /// `<p>-fg-subtle`/`<p>-subtle`（淡色背景上の本文）を含む。
    const BODY_TEXT_PAIRS: &[(&str, &str)] = &[
        ("fg", "bg"),
        ("fg", "bg-subtle"),
        ("fg", "bg-muted"),
        ("fg-muted", "bg"),
        ("accent-fg-subtle", "accent-subtle"),
        ("info-fg-subtle", "info-subtle"),
        ("success-fg-subtle", "success-subtle"),
        ("warning-fg-subtle", "warning-subtle"),
        ("danger-fg-subtle", "danger-subtle"),
        ("neutral-fg-subtle", "neutral-subtle"),
    ];

    /// 大字・UI 部品相当ペア（3:1 以上）。solid 背景上の文字色・背景自体の
    /// 可読性・フォーカスリングの視認性を含む。`border`/`bg` は chakra-ui /
    /// Radix Themes でも満たさない設計（gray 6 系）のため対象外とする
    /// （`docs/design/color-token-system.md` §7 に記録）。
    const LARGE_TEXT_UI_PAIRS: &[(&str, &str)] = &[
        ("accent-fg", "accent"),
        ("accent-fg", "accent-emphasized"),
        ("info-fg", "info"),
        ("info-fg", "info-emphasized"),
        ("success-fg", "success"),
        ("success-fg", "success-emphasized"),
        ("warning-fg", "warning"),
        ("warning-fg", "warning-emphasized"),
        ("danger-fg", "danger"),
        ("danger-fg", "danger-emphasized"),
        ("neutral-fg", "neutral"),
        ("neutral-fg", "neutral-emphasized"),
        ("fg-subtle", "bg"),
        ("accent", "bg"),
        ("info", "bg"),
        ("success", "bg"),
        ("warning", "bg"),
        ("danger", "bg"),
        ("neutral", "bg"),
        ("focus-ring", "bg"),
    ];

    #[test]
    fn body_text_pairs_meet_wcag_4_5_to_1_in_light_and_dark() {
        for (fg_name, bg_name) in BODY_TEXT_PAIRS {
            let (fg_light, fg_dark) = default_color(fg_name);
            let (bg_light, bg_dark) = default_color(bg_name);

            let light_ratio = contrast_ratio(fg_light, bg_light);
            assert!(
                light_ratio >= 4.5,
                "light: {fg_name}/{bg_name} = {light_ratio:.3} (< 4.5:1)"
            );

            let dark_ratio = contrast_ratio(fg_dark, bg_dark);
            assert!(
                dark_ratio >= 4.5,
                "dark: {fg_name}/{bg_name} = {dark_ratio:.3} (< 4.5:1)"
            );
        }
    }

    // イシュー #1423: radius/shadow/spacing 拡充・z-index 新設のユニットテスト。

    #[test]
    fn z_index_var_builds_expected_reference() {
        assert_eq!(z_index_var("toast").unwrap(), "var(--fandhe-z-index-toast)");
        assert!(z_index_var("Toast").is_err());
    }

    #[test]
    fn push_z_index_rejects_duplicate_name() {
        let mut theme = Theme::empty();
        theme.push_z_index("toast", "1600").unwrap();
        assert!(theme.push_z_index("toast", "1700").is_err());
    }

    #[test]
    fn upsert_z_index_overwrites_existing_value() {
        let mut theme = Theme::empty();
        theme.push_z_index("toast", "1600").unwrap();
        theme.upsert_z_index("toast", "1650").unwrap();
        assert!(theme.to_css().contains("--fandhe-z-index-toast: 1650;"));
    }

    /// codex-review #1705 P1 指摘の回帰テスト: `CssValue` の文字 allowlist は
    /// 満たすが `z-index` プロパティとしては無効な値（色名・単位付き寸法・
    /// `url()`）を `push_z_index` が拒否することを固定する。
    #[test]
    fn push_z_index_rejects_non_integer_non_keyword_values() {
        let mut theme = Theme::empty();
        assert_eq!(
            theme.push_z_index("bad", "red"),
            Err(ThemeError::InvalidZIndexValue {
                value: "red".to_string(),
            })
        );
        assert_eq!(
            theme.push_z_index("bad", "1rem"),
            Err(ThemeError::InvalidZIndexValue {
                value: "1rem".to_string(),
            })
        );
        assert_eq!(
            theme.push_z_index("bad", "url(foo)"),
            Err(ThemeError::InvalidZIndexValue {
                value: "url(foo)".to_string(),
            })
        );
        // 拒否した場合は一切追加されない。
        assert!(theme.to_css().is_empty() || !theme.to_css().contains("z-index-bad"));
    }

    /// 同上の回帰テスト（`upsert_z_index` 側）。
    #[test]
    fn upsert_z_index_rejects_non_integer_non_keyword_values() {
        let mut theme = Theme::empty();
        assert_eq!(
            theme.upsert_z_index("bad", "1rem"),
            Err(ThemeError::InvalidZIndexValue {
                value: "1rem".to_string(),
            })
        );
    }

    /// 整数（符号付き含む）と CSS グローバル値は許可されることを固定する。
    #[test]
    fn push_z_index_accepts_integers_and_global_keywords() {
        let mut theme = Theme::empty();
        theme.push_z_index("neg", "-1").unwrap();
        theme.push_z_index("zero", "0").unwrap();
        theme.push_z_index("big", "2147483647").unwrap();
        theme.push_z_index("auto", "auto").unwrap();
        theme.push_z_index("inherited", "inherit").unwrap();
        let css = theme.to_css();
        assert!(css.contains("--fandhe-z-index-neg: -1;"));
        assert!(css.contains("--fandhe-z-index-auto: auto;"));
    }

    /// codex-review #1705 P1 回帰: 正符号付きの整数（`+1` `+1600` 等）も
    /// CSS の有効な `<integer>` であり、`push_z_index` は受理しなければ
    /// ならない（`strip_prefix('-')` のみで `'+'` を考慮していなかった
    /// ためのすり抜けの再発防止）。
    #[test]
    fn push_z_index_accepts_positive_signed_integers() {
        let mut theme = Theme::empty();
        theme.push_z_index("plus-one", "+1").unwrap();
        theme.push_z_index("plus-big", "+1600").unwrap();
        let css = theme.to_css();
        assert!(css.contains("--fandhe-z-index-plus-one: +1;"));
        assert!(css.contains("--fandhe-z-index-plus-big: +1600;"));
    }

    /// 同上の回帰テスト（`upsert_z_index` 側）。
    #[test]
    fn upsert_z_index_accepts_positive_signed_integers() {
        let mut theme = Theme::empty();
        theme.upsert_z_index("plus-one", "+1").unwrap();
        theme.upsert_z_index("plus-big", "+1600").unwrap();
        let css = theme.to_css();
        assert!(css.contains("--fandhe-z-index-plus-one: +1;"));
        assert!(css.contains("--fandhe-z-index-plus-big: +1600;"));
    }

    #[test]
    fn default_theme_includes_new_1423_tokens() {
        let css = Theme::default().to_css();
        // radii の純追加分。
        assert!(css.contains("--fandhe-radius-none: 0;"));
        assert!(css.contains("--fandhe-radius-xs: 0.125rem;"));
        assert!(css.contains("--fandhe-radius-2xl: 1rem;"));
        // shadows の純追加分。
        assert!(css.contains("--fandhe-shadow-xl: 0 20px 25px rgba(0, 0, 0, 0.2);"));
        assert!(css.contains("--fandhe-shadow-2xl: 0 25px 50px rgba(0, 0, 0, 0.25);"));
        // spaces の純追加分。
        assert!(css.contains("--fandhe-space-0-5: 0.125rem;"));
        assert!(css.contains("--fandhe-space-1-5: 0.375rem;"));
        assert!(css.contains("--fandhe-space-2-5: 0.625rem;"));
        assert!(css.contains("--fandhe-space-20: 5rem;"));
        assert!(css.contains("--fandhe-space-24: 6rem;"));
        // z-index 新設グループ（既定 12 件）。
        for (name, value) in DEFAULT_Z_INDICES {
            assert!(
                css.contains(&format!("--fandhe-z-index-{name}: {value};")),
                "missing z-index token: {name}"
            );
        }
    }

    #[test]
    fn large_text_and_ui_pairs_meet_wcag_3_to_1_in_light_and_dark() {
        for (fg_name, bg_name) in LARGE_TEXT_UI_PAIRS {
            let (fg_light, fg_dark) = default_color(fg_name);
            let (bg_light, bg_dark) = default_color(bg_name);

            let light_ratio = contrast_ratio(fg_light, bg_light);
            assert!(
                light_ratio >= 3.0,
                "light: {fg_name}/{bg_name} = {light_ratio:.3} (< 3:1)"
            );

            let dark_ratio = contrast_ratio(fg_dark, bg_dark);
            assert!(
                dark_ratio >= 3.0,
                "dark: {fg_name}/{bg_name} = {dark_ratio:.3} (< 3:1)"
            );
        }
    }

    #[test]
    fn default_theme_z_indices_do_not_appear_in_dark_blocks() {
        // z-index はモード非依存のため、`write_dark_declarations` 経由の
        // dark ブロック（`@media` と `[data-theme="dark"]`）には一切現れず、
        // `:root` ブロックの 1 箇所にのみ出現する（radii と同じ扱い）。
        let css = Theme::default().to_css();
        let count = css.matches("--fandhe-z-index-toast:").count();
        assert_eq!(
            count, 1,
            "z-index はモード非依存のため :root に 1 回だけ出現するはず"
        );
    }

    #[test]
    fn empty_theme_without_z_indices_omits_z_index_vars() {
        // z-indices を一切 push しないテーマの `to_css()` 出力は、本イシュー
        // （#1423）で追加した z-indices グループの純追加であることを保証する
        // 回帰テスト（`theme_without_radii_or_shadows_matches_pre_606_snapshot`
        // と対をなす）。
        let mut theme = Theme::empty();
        theme.push_color("bg", "#ffffff", "#111111").unwrap();

        let css = theme.to_css();
        assert!(!css.contains("--fandhe-z-index-"));
    }

    // イシュー #1424: フォーカスリングトークン（寸法グループ + 専用色）の
    // ユニットテスト。

    #[test]
    fn focus_ring_var_builds_expected_reference() {
        assert_eq!(
            focus_ring_var("width").unwrap(),
            "var(--fandhe-focus-ring-width)"
        );
        assert!(focus_ring_var("Width").is_err());
    }

    #[test]
    fn push_focus_ring_rejects_duplicate_name() {
        let mut theme = Theme::empty();
        theme.push_focus_ring("width", "2px").unwrap();
        assert!(theme.push_focus_ring("width", "3px").is_err());
    }

    #[test]
    fn upsert_focus_ring_overwrites_existing_value() {
        let mut theme = Theme::empty();
        theme.push_focus_ring("width", "2px").unwrap();
        theme.upsert_focus_ring("width", "3px").unwrap();
        assert!(theme.to_css().contains("--fandhe-focus-ring-width: 3px;"));
    }

    // codex-review #1707 P1 指摘の回帰テスト: `push_focus_ring`/
    // `upsert_focus_ring` は `outline-width`/`outline-offset` の寸法値と
    // して無効な値（色・`rgba(...)` 等）を拒否しなければならない
    // （型不一致で `outline` 宣言全体が無効化されキーボードフォーカス
    // 表示が消え得るため）。

    #[test]
    fn push_focus_ring_accepts_valid_lengths_and_global_keywords() {
        let mut theme = Theme::empty();
        assert!(theme.push_focus_ring("width", "2px").is_ok());
        assert!(theme.push_focus_ring("offset", "0").is_ok());
        assert!(theme.push_focus_ring("gap", "0.125rem").is_ok());
        assert!(theme.push_focus_ring("keyword", "inherit").is_ok());
    }

    #[test]
    fn push_focus_ring_rejects_color_value() {
        let mut theme = Theme::empty();
        assert_eq!(
            theme.push_focus_ring("width", "#4299e1"),
            Err(ThemeError::InvalidFocusRingValue {
                value: "#4299e1".to_string()
            })
        );
    }

    #[test]
    fn push_focus_ring_rejects_rgba_value() {
        let mut theme = Theme::empty();
        assert!(matches!(
            theme.push_focus_ring("width", "rgba(0, 0, 0, 0.4)"),
            Err(ThemeError::InvalidFocusRingValue { .. })
        ));
    }

    #[test]
    fn upsert_focus_ring_rejects_invalid_dimension_value() {
        let mut theme = Theme::empty();
        theme.push_focus_ring("width", "2px").unwrap();
        assert!(matches!(
            theme.upsert_focus_ring("width", "solid"),
            Err(ThemeError::InvalidFocusRingValue { .. })
        ));
        // 検証失敗時は既存値を上書きしない。
        assert!(theme.to_css().contains("--fandhe-focus-ring-width: 2px;"));
    }

    #[test]
    fn default_theme_includes_focus_ring_tokens() {
        let css = Theme::default().to_css();
        // 寸法トークン（モード非依存、:root に 1 回のみ）。
        assert!(css.contains("--fandhe-focus-ring-width: 2px;"));
        assert!(css.contains("--fandhe-focus-ring-offset: 2px;"));
        assert_eq!(css.matches("--fandhe-focus-ring-width:").count(), 1);

        // 専用色トークン（colors グループ経由、light は :root、dark は
        // メディアクエリと data-theme の 2 箇所）。
        assert!(css.contains("--fandhe-color-focus-ring: #3182ce;"));
        assert_eq!(
            css.matches("--fandhe-color-focus-ring: #63b3ed;").count(),
            2,
            "dark 値は media query と data-theme の双方に出現するはず"
        );
    }

    #[test]
    fn empty_theme_without_focus_ring_omits_focus_ring_vars() {
        // focus_ring を一切 push しないテーマの `to_css()` 出力は、本イシュー
        // （#1424）で追加した focus_ring グループの純追加であることを保証する
        // 回帰テスト（`empty_theme_without_z_indices_omits_z_index_vars` と
        // 対をなす）。
        let mut theme = Theme::empty();
        theme.push_color("bg", "#ffffff", "#111111").unwrap();

        let css = theme.to_css();
        assert!(!css.contains("--fandhe-focus-ring-"));
    }
}
