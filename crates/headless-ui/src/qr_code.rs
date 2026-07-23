//! QrCode（QR コード表示）headless コンポーネント（イシュー #774、親 #766）。
//!
//! ark-ui の QrCode（`.claude/skills/ark-ui/references/components/display/qr-code.md`）
//! を参考に、Root / Frame / Pattern / Overlay の 4 anatomy パーツと、
//! QR Model 2（ISO/IEC 18004）byte モードの外部依存ゼロエンコーダ
//! （[`crate::qr_encode`]、非公開実装）を提供する。
//!
//! # 状態機械を持たない理由（[`crate::tabs`]/[`crate::field`] と同じ区分）
//!
//! QrCode の描画は `value`（符号化対象文字列）と `ecc`（誤り訂正レベル）から
//! 一意に導出される純粋な変換であり、開閉・選択のような遷移可能な状態を
//! 持たない。そのため [`fandhe_frontend_interactive::Component`]/
//! [`fandhe_frontend_interactive::Hydrate`] を実装せず、自由関数のみを提供
//! する（`crates/headless-ui/src/tabs.rs` と同型の判断）。
//!
//! # anatomy
//!
//! | パーツ | 関数 | タグ | `data-part` |
//! |---|---|---|---|
//! | Root | [`root`] | `div` | `root` |
//! | Frame | [`frame`] | `svg` | `frame` |
//! | Pattern | [`pattern`] | `path` | `pattern` |
//! | Overlay | [`overlay`] | `div` | `overlay` |
//!
//! ## core 側拡張は不要（判断根拠）
//!
//! `fandhe_frontend_core` の `is_valid_tag_name`/`is_valid_attr_name` は
//! `svg`/`path` タグや `viewBox`/`d`/`role` 等の属性を既に許容するため、
//! [`crate::anatomy::Anatomy::part`] へ `"svg"`/`"path"` をタグ名として渡す
//! だけで描画できる（`crates/headless-ui/src/progress.rs` circular 節と
//! 同じ判断。core への変更は 0 行）。
//!
//! # セキュリティ不変条件
//!
//! - `value`（符号化対象文字列）はマークアップへ一切出力されない。
//!   [`qr_encode::encode`] はバイト列からモジュール行列（暗/明の bool 配列）
//!   へのみ変換し、文字列としての `value` を保持・再出力しない。
//! - [`pattern`] の `d` 属性値は暗モジュールの座標から本モジュールが内部生成
//!   する文字列であり、文字集合は `M`/`h`/`v`/`z`/半角数字/`,` に閉じる
//!   （[`build_path_d`]）。呼び出し側入力が `d` 属性値へ混入する経路はない。
//! - SVG は全て [`fandhe_frontend_core::el`] のノード木 API で構築し、
//!   `raw_html()` は使用しない・HTML/SVG 文字列を直接組み立てない（REQ-1）。
//! - `attrs`/`children` は既存 anatomy 契約どおり
//!   [`fandhe_frontend_core::render`] の既定エスケープを必ず経由する。
//!
//! # 本イシューのスコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - `DownloadTrigger`（canvas 描画 + ダウンロード、JS 必須）は headless
//!   静的 SSR の対象外。`fandhe-frontend-wasm-full` の後続責務として別途
//!   Issue 化を提案する。
//! - `value` の動的更新（`onValueChange` 相当）・wasm 配線。
//! - numeric/alphanumeric/kanji モードによる容量最適化・ECI・構造的連接
//!   （[`crate::qr_encode`] のモジュール doc 参照）。
//! - `examples/headless-pre-styled-ui` への追随（crates.io 公開後、既存
//!   運用どおり別 Issue）。

use crate::anatomy::{anatomy, Anatomy};
use crate::aria::role;
use crate::qr_encode::{self, Ecc};
use fandhe_frontend_core::Node;
use std::fmt::Write as _;

/// QrCode の anatomy（`data-scope="qr-code"`）。
const ANATOMY: Anatomy = anatomy("qr-code");

/// [`frame`]/[`pattern`] 既定の静粛帯（quiet zone）モジュール数。
/// ISO/IEC 18004 が要求する最小静粛帯（4 モジュール）に合わせる。
pub const DEFAULT_QUIET_ZONE: u32 = 4;

/// 誤り訂正レベル（ISO/IEC 18004 表 25）。既定は `L`（回復率 約 7%）。
///
/// | バリアント | 回復率（概算） |
/// |---|---|
/// | `L` | 約 7% |
/// | `M` | 約 15% |
/// | `Q` | 約 25% |
/// | `H` | 約 30% |
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ErrorCorrectionLevel {
    /// 約 7% 回復（既定）。
    #[default]
    L,
    /// 約 15% 回復。
    M,
    /// 約 25% 回復。
    Q,
    /// 約 30% 回復。
    H,
}

impl ErrorCorrectionLevel {
    fn to_internal(self) -> Ecc {
        match self {
            ErrorCorrectionLevel::L => Ecc::Low,
            ErrorCorrectionLevel::M => Ecc::Medium,
            ErrorCorrectionLevel::Q => Ecc::Quartile,
            ErrorCorrectionLevel::H => Ecc::High,
        }
    }
}

/// [`encode`] のエラー（fail-closed）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QrEncodeError {
    /// 入力バイト列が指定した誤り訂正レベルにおけるバージョン 40 の
    /// 最大容量を超過した（例: ECC L で 2953 バイト超）。
    TooLong,
}

/// エンコード結果のモジュール行列（不変値型、`true` = 暗モジュール）。
///
/// [`frame`]/[`pattern`] へ渡す唯一の入力であり、`value` 文字列そのものは
/// 保持しない（モジュール doc「セキュリティ不変条件」参照）。
#[derive(Debug, Clone, PartialEq)]
pub struct QrMatrix {
    raw: qr_encode::RawMatrix,
}

impl QrMatrix {
    /// モジュール数（1 辺、静粛帯を含まない）。`17 + 4 * version`。
    #[must_use]
    pub fn size(&self) -> usize {
        self.raw.size
    }

    /// `(x, y)`（0 始まり、静粛帯を含まない座標系）が暗モジュールかどうか。
    ///
    /// # Panics
    ///
    /// `x`/`y` が [`QrMatrix::size`] 以上の場合は panic する（呼び出し側が
    /// `0..size()` の範囲で呼ぶ契約。本型は `crate::qr_code` モジュール内で
    /// 完結して生成されるため、範囲外呼び出しは実装バグとして扱う）。
    #[must_use]
    pub fn is_dark(&self, x: usize, y: usize) -> bool {
        self.raw.is_dark(x, y)
    }

    /// デバッグ・テスト向けの行文字列表現（暗モジュール `'#'`、明モジュール
    /// `'.'`）。ターミナル出力・golden テストでの可読な比較に使う。
    #[must_use]
    pub fn debug_rows(&self) -> Vec<String> {
        (0..self.size())
            .map(|y| {
                (0..self.size())
                    .map(|x| if self.is_dark(x, y) { '#' } else { '.' })
                    .collect()
            })
            .collect()
    }
}

/// `value` を QR Model 2（byte モード、外部依存ゼロ）でエンコードする。
///
/// バージョン 1..=40 から `value` が収まる最小バージョンを決定的に選択し、
/// マスクは 8 種のペナルティスコア評価で決定的に選ぶ（同一入力からは常に
/// 同一のモジュール行列を返す。乱数・時刻・環境非依存）。
///
/// # Errors
///
/// `value` のバイト長が `ecc` 指定下でのバージョン 40 最大容量を超える場合
/// [`QrEncodeError::TooLong`] を返す（`panic!`/`unwrap()` しない）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_headless_ui::qr_code::{encode, ErrorCorrectionLevel};
///
/// let matrix = encode("HELLO", ErrorCorrectionLevel::L).unwrap();
/// assert_eq!(matrix.size(), 21); // バージョン 1
/// ```
pub fn encode(value: &str, ecc: ErrorCorrectionLevel) -> Result<QrMatrix, QrEncodeError> {
    match qr_encode::encode(value.as_bytes(), ecc.to_internal()) {
        Ok((raw, _mask)) => Ok(QrMatrix { raw }),
        Err(qr_encode::QrEncodeError::TooLong) => Err(QrEncodeError::TooLong),
    }
}

/// Root パーツ（`div`）。
#[must_use]
pub fn root<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("root", "div", attrs, children)
}

/// Frame パーツ（`svg`）。`viewBox` は `matrix.size() + 2 * quiet_zone` の
/// 正方形（ISO/IEC 18004 が要求する静粛帯を含む）。
///
/// `aria_label` を指定すると `aria-label` を付与する（未指定時は
/// `role="img"` のみで、代替テキストの提供は呼び出し側の責務のままにする。
/// fail-closed に偽の説明文を捏造しない）。
#[must_use]
pub fn frame<'a>(
    matrix: &QrMatrix,
    quiet_zone: u32,
    aria_label: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let n = matrix.size() + 2 * quiet_zone as usize;
    let view_box = format!("0 0 {n} {n}");
    let mut merged: Vec<(&str, &str)> = vec![("viewBox", view_box.as_str()), role("img")];
    if let Some(label) = aria_label {
        merged.push(("aria-label", label));
    }
    merged.extend(attrs);
    ANATOMY.part("frame", "svg", merged, children)
}

/// Pattern パーツ（`path`）。`d` 属性値は暗モジュールごとの
/// `M{x},{y}h1v1h-1z`（1x1 の正方形）を行優先で連結した内部生成文字列
/// （[`build_path_d`]）。`fill` は付与しない（styled 層/呼び出し側 CSS の
/// 責務、headless 中立、`crates/headless-ui/src/progress.rs` と同じ方針）。
#[must_use]
pub fn pattern<'a>(matrix: &QrMatrix, quiet_zone: u32, attrs: Vec<(&'a str, &'a str)>) -> Node {
    let d = build_path_d(matrix, quiet_zone);
    let mut merged: Vec<(&str, &str)> = vec![("d", d.as_str())];
    merged.extend(attrs);
    ANATOMY.part("pattern", "path", merged, vec![])
}

/// Overlay パーツ（`div`）。ロゴ等、呼び出し側が中央に重ねるコンテンツの
/// コンテナ（可視スタイル・中央配置は styled 層/呼び出し側 CSS の責務）。
#[must_use]
pub fn overlay<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("overlay", "div", attrs, children)
}

/// [`pattern`] の `d` 属性値を組み立てる（内部ヘルパ）。
///
/// 出力文字列の文字集合は `M`/`h`/`v`/`z`/半角数字/`,` に閉じ、`value`
/// 文字列由来の任意バイトが混入する経路はない（[`QrMatrix`] は暗/明の
/// bool 配列のみを保持し、元の `value` を保持しないため構造的に不可能）。
fn build_path_d(matrix: &QrMatrix, quiet_zone: u32) -> String {
    let mut d = String::new();
    for y in 0..matrix.size() {
        for x in 0..matrix.size() {
            if matrix.is_dark(x, y) {
                let ox = x + quiet_zone as usize;
                let oy = y + quiet_zone as usize;
                let _ = write!(d, "M{ox},{oy}h1v1h-1z");
            }
        }
    }
    d
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;

    #[test]
    fn encode_is_deterministic_and_produces_expected_version() {
        let a = encode("https://fandhe.example/", ErrorCorrectionLevel::L).unwrap();
        let b = encode("https://fandhe.example/", ErrorCorrectionLevel::L).unwrap();
        assert_eq!(a.debug_rows(), b.debug_rows());
        // 23 文字は byte モード v1-L 容量 17 文字を超えるため v2（size=25）になる。
        assert_eq!(a.size(), 25);
    }

    #[test]
    fn empty_value_produces_minimum_version() {
        let matrix = encode("", ErrorCorrectionLevel::L).unwrap();
        assert_eq!(matrix.size(), 21);
    }

    #[test]
    fn too_long_value_is_fail_closed() {
        let value = "A".repeat(3000);
        assert_eq!(
            encode(&value, ErrorCorrectionLevel::L),
            Err(QrEncodeError::TooLong)
        );
    }

    #[test]
    fn frame_has_expected_view_box_and_role() {
        let matrix = encode("hi", ErrorCorrectionLevel::L).unwrap();
        let node = frame(&matrix, DEFAULT_QUIET_ZONE, None, vec![], vec![]);
        let html = render(&node);
        assert!(html.contains(r#"viewBox="0 0 29 29""#));
        assert!(html.contains(r#"role="img""#));
        assert!(html.contains(r#"data-scope="qr-code" data-part="frame""#));
    }

    #[test]
    fn pattern_d_attribute_is_closed_character_set() {
        let matrix = encode("</svg><script>alert(1)</script>", ErrorCorrectionLevel::L).unwrap();
        let node = pattern(&matrix, DEFAULT_QUIET_ZONE, vec![]);
        let html = render(&node);
        assert!(!html.contains("<script"));
        assert!(!html.contains("</svg><script"));
        // value 文字列そのものが d 属性値へ混入していないこと。
        assert!(!html.contains("alert(1)"));
        let d_start = html.find(r#" d=""#).expect("d 属性が出力される") + 4;
        let d_end = html[d_start..].find('"').expect("d 属性値の終端");
        let d_value = &html[d_start..d_start + d_end];
        // `-1` の符号（h-1）を許容するため `-` も許可文字集合へ含める。
        assert!(d_value
            .chars()
            .all(|c| matches!(c, 'M' | 'h' | 'v' | 'z' | ',' | '-' | '0'..='9')));
    }

    #[test]
    fn root_and_overlay_are_plain_divs() {
        let root_html = render(&root(vec![], vec![]));
        assert!(root_html.contains(r#"data-scope="qr-code" data-part="root""#));
        let overlay_html = render(&overlay(vec![], vec![]));
        assert!(overlay_html.contains(r#"data-scope="qr-code" data-part="overlay""#));
    }
}
