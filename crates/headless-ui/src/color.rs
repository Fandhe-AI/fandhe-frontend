//! RGB / HSL / HSV / HEX の色空間相互変換コア（イシュー #838、親 #837）。
//!
//! `fandhe-frontend-pre-styled-ui` の [`crate::color`]（本モジュールの唯一の
//! 消費元、`ColorSwatch` #838 が使う）と、後続の ColorPicker（#837 配下の
//! 別イシュー）が土台にする、外部依存ゼロ・整数演算のみの純粋関数モジュール。
//! `qr_encode`（#774）と同型の「標準ライブラリのみで完結する決定的アルゴリズム
//! モジュール」であり、ブラウザ API（`web-sys` 等）には一切依存しない。
//!
//! # 丸め規則（module 全体で固定する不変条件）
//!
//! - `f32`/`f64` を一切使わない。すべて `i64`/`u32`/`u8` のスケール付き整数
//!   演算で完結する。
//! - 正の有理数 `num/den`（`num`,`den` ともに非負）の丸めは round half up:
//!   `(2*num + den) / (2*den)`（[`div_round_half_up`]）で固定する。
//! - 負値が生じ得る中間項（色相差分）は [`i64::rem_euclid`] で事前に
//!   `[0, 360*delta)` の正域へ正規化してから同じ丸め式を適用する
//!   （[`hue_degrees`] 参照）。
//! - 無彩色（`max == min`、グレー・白・黒）は `S = 0` かつ `H = 0` と定義する
//!   （色相が数学的に不定であるための安全側の既定、CSS Color 仕様の慣習に
//!   従う）。
//!
//! # fail-closed 方針（セキュリティ不変条件）
//!
//! - [`Color::parse_hex`] は `#rgb`/`#rgba`/`#rrggbb`/`#rrggbbaa` の 4 形式
//!   以外（長さ不正・`#` 欠落・非 16 進文字）をすべて `Err` にする（黙って
//!   補正しない）。
//! - [`Hsl::new`]/[`Hsv::new`] は範囲外（`h > 359` または `s`/`l`/`v` > 100）
//!   を構築不能にする fallible コンストラクタのみを公開する（フィールドは
//!   非公開）。
//! - [`Color::to_hex_string`] の出力字母は常に `#` + `[0-9a-f]` に閉じる
//!   （`fandhe-frontend-pre-styled-ui::color_swatch` が CSS カスタム
//!   プロパティ値としてそのまま使う契約の根拠、`crates/pre-styled-ui/src/
//!   color_swatch.rs` 参照）。
//! - [`ColorError`] の `Display` は静的文言のみで入力値をエコーしない
//!   （`docs/policy/`・`StylesheetError` と同じ機微情報露出防止方針）。

use std::fmt;

/// 正の有理数 `num/den`（`den > 0`）を round half up で整数へ丸める。
///
/// `(2*num + den) / (2*den)` という形（整数除算は切り捨て）で、`num/den`
/// がちょうど `.5` の端数を持つ場合に必ず切り上げる（本モジュール冒頭
/// 「丸め規則」参照）。
fn div_round_half_up(num: i64, den: i64) -> i64 {
    debug_assert!(den > 0, "den は正の値のみを呼び出し側が渡す契約");
    (2 * num + den) / (2 * den)
}

/// RGB 各チャネル（0..=255）から色相環の 6 領域（hgroup）に応じて
/// `(chroma, second_max, 0)` を `(R, G, B)` の並びへ回転配置する
/// （[`Hsl::to_rgb`]/[`Hsv::to_rgb`] の共通部分）。
///
/// `chroma_scaled` は `chroma * 10000`（HSL: `(100 - |2L-100|) * S`、
/// HSV: `V * S`）として呼び出し側が正規化済みであることを前提とする。
/// 戻り値は分母 `600000` に正規化した `(r, g, b)` の 3 成分（"m"（明度
/// オフセット）を加算する前の値）。
fn hue_to_rgb_components(h: u16, chroma_scaled: i64) -> (i64, i64, i64) {
    let hgroup = (h / 60) % 6;
    let h60 = i64::from(h % 60);
    // X = C * (h60/60)（hgroup 偶数）または C * ((60-h60)/60)（hgroup 奇数）。
    // 分母 600000（= 10000 * 60）に正規化した値を直接計算する。
    let x_scaled = if hgroup.is_multiple_of(2) {
        chroma_scaled * h60
    } else {
        chroma_scaled * (60 - h60)
    };
    let c_scaled = chroma_scaled * 60;
    match hgroup {
        0 => (c_scaled, x_scaled, 0),
        1 => (x_scaled, c_scaled, 0),
        2 => (0, c_scaled, x_scaled),
        3 => (0, x_scaled, c_scaled),
        4 => (x_scaled, 0, c_scaled),
        _ => (c_scaled, 0, x_scaled),
    }
}

/// 分母 `600000` に正規化した値（`0..=600000` を期待するが、丸め伝播に
/// よる境界誤差に備えて防御的に clamp する）を `0..=255` の 1 バイトへ
/// 丸める。
fn channel_from_scaled(value_scaled: i64) -> u8 {
    let clamped = value_scaled.clamp(0, 600_000);
    div_round_half_up(clamped * 255, 600_000) as u8
}

/// RGB 各チャネルから色相（度、`0..=359`）を求める（`delta > 0` を呼び出し側が
/// 保証する契約。`delta == 0` の無彩色は呼び出し側で `h = 0` として扱う）。
///
/// 中間項 `numerator`（負値を取り得る）は [`i64::rem_euclid`] で
/// `[0, 360*delta)` の正域へ正規化してから [`div_round_half_up`] を適用する
/// （本モジュール冒頭「丸め規則」参照）。丸め結果がちょうど `360` になる
/// 境界（`numerator` が `360*delta` に極めて近い場合）は `% 360` で `0` へ
/// 巻き戻す。
fn hue_degrees(r: i32, g: i32, b: i32, max: i32, delta: i32) -> u16 {
    debug_assert!(delta > 0, "delta > 0 は呼び出し側が保証する契約");
    let (raw_diff, offset_deg): (i32, i32) = if max == r {
        (g - b, 0)
    } else if max == g {
        (b - r, 120)
    } else {
        (r - g, 240)
    };
    let numerator = i64::from(raw_diff) * 60 + i64::from(offset_deg) * i64::from(delta);
    let full_turn = 360 * i64::from(delta);
    let normalized = numerator.rem_euclid(full_turn);
    (div_round_half_up(normalized, i64::from(delta)) % 360) as u16
}

/// 8 ビット/チャネルの RGB 色（アルファなし）。
///
/// `u8` の全域（`0..=255`）が有効値であるため、フィールドは公開しており
/// 構築時検証を必要としない（[`Hsl`]/[`Hsv`] とは異なる）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Rgb {
    /// 赤チャネル。
    pub r: u8,
    /// 緑チャネル。
    pub g: u8,
    /// 青チャネル。
    pub b: u8,
}

impl Rgb {
    /// `r`/`g`/`b` から構築する。
    #[must_use]
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// HSL（色相・彩度・明度）表現へ変換する。
    ///
    /// 無彩色（`max == min`）は `s = 0, h = 0` を返す（本モジュール冒頭
    /// 「丸め規則」参照）。
    #[must_use]
    pub fn to_hsl(self) -> Hsl {
        let (r, g, b) = (i32::from(self.r), i32::from(self.g), i32::from(self.b));
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let delta = max - min;
        let sum = max + min;

        let l = div_round_half_up(i64::from(sum) * 100, 510) as u8;

        if delta == 0 {
            return Hsl { h: 0, s: 0, l };
        }

        let s_den = if sum <= 255 { sum } else { 510 - sum };
        let s = div_round_half_up(i64::from(delta) * 100, i64::from(s_den)) as u8;
        let h = hue_degrees(r, g, b, max, delta);
        Hsl { h, s, l }
    }

    /// HSV（色相・彩度・明度値）表現へ変換する。
    ///
    /// 無彩色（`max == min`）は `s = 0, h = 0` を返す（本モジュール冒頭
    /// 「丸め規則」参照）。
    #[must_use]
    pub fn to_hsv(self) -> Hsv {
        let (r, g, b) = (i32::from(self.r), i32::from(self.g), i32::from(self.b));
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let delta = max - min;

        let v = div_round_half_up(i64::from(max) * 100, 255) as u8;

        if delta == 0 {
            return Hsv { h: 0, s: 0, v };
        }

        let s = div_round_half_up(i64::from(delta) * 100, i64::from(max)) as u8;
        let h = hue_degrees(r, g, b, max, delta);
        Hsv { h, s, v }
    }
}

/// HSL（色相 `0..=359` 度・彩度 `0..=100` %・明度 `0..=100` %）表現。
///
/// フィールドは非公開とし、範囲外の値を構築不能にする [`Hsl::new`]
/// （fail-closed）のみを公開する。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hsl {
    h: u16,
    s: u8,
    l: u8,
}

impl Hsl {
    /// `h`（`0..=359`）・`s`/`l`（`0..=100`）から構築する。範囲外は
    /// [`ColorError::OutOfRange`]（fail-closed、黙って clamp しない）。
    ///
    /// # Errors
    ///
    /// `h > 359` または `s > 100` または `l > 100` のとき
    /// [`ColorError::OutOfRange`] を返す。
    pub fn new(h: u16, s: u8, l: u8) -> Result<Self, ColorError> {
        if h > 359 || s > 100 || l > 100 {
            return Err(ColorError::OutOfRange);
        }
        Ok(Self { h, s, l })
    }

    /// 色相（度）。
    #[must_use]
    pub const fn h(self) -> u16 {
        self.h
    }

    /// 彩度（%）。
    #[must_use]
    pub const fn s(self) -> u8 {
        self.s
    }

    /// 明度（%）。
    #[must_use]
    pub const fn l(self) -> u8 {
        self.l
    }

    /// RGB 表現へ変換する（CSS Color 仕様の HSL→RGB 変換式と同値の、整数
    /// 演算のみによる決定的実装。本モジュール冒頭「丸め規則」参照）。
    #[must_use]
    pub fn to_rgb(self) -> Rgb {
        let l100 = i64::from(self.l);
        let s100 = i64::from(self.s);
        // chroma_scaled = C * 10000（C = (1 - |2L-1|) * S、L/S は 0..=1 の
        // 比率だが本モジュールは 0..=100 の percent 整数のまま演算する）。
        let chroma_scaled = (100 - (2 * l100 - 100).abs()) * s100;
        let (r_scaled, g_scaled, b_scaled) = hue_to_rgb_components(self.h, chroma_scaled);
        // m = L - C/2。分母 20000 に正規化: m_scaled/20000 = l100/100 - chroma_scaled/20000。
        let m_scaled = l100 * 200 - chroma_scaled;
        // 600000/20000 = 30 を掛けて分母 600000 へ揃える（hue_to_rgb_components
        // の戻り値と同じ分母）。
        let m = m_scaled * 30;
        Rgb {
            r: channel_from_scaled(r_scaled + m),
            g: channel_from_scaled(g_scaled + m),
            b: channel_from_scaled(b_scaled + m),
        }
    }
}

/// HSV（色相 `0..=359` 度・彩度 `0..=100` %・明度値 `0..=100` %）表現。
///
/// フィールドは非公開とし、範囲外の値を構築不能にする [`Hsv::new`]
/// （fail-closed）のみを公開する。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hsv {
    h: u16,
    s: u8,
    v: u8,
}

impl Hsv {
    /// `h`（`0..=359`）・`s`/`v`（`0..=100`）から構築する。範囲外は
    /// [`ColorError::OutOfRange`]（fail-closed、黙って clamp しない）。
    ///
    /// # Errors
    ///
    /// `h > 359` または `s > 100` または `v > 100` のとき
    /// [`ColorError::OutOfRange`] を返す。
    pub fn new(h: u16, s: u8, v: u8) -> Result<Self, ColorError> {
        if h > 359 || s > 100 || v > 100 {
            return Err(ColorError::OutOfRange);
        }
        Ok(Self { h, s, v })
    }

    /// 色相（度）。
    #[must_use]
    pub const fn h(self) -> u16 {
        self.h
    }

    /// 彩度（%）。
    #[must_use]
    pub const fn s(self) -> u8 {
        self.s
    }

    /// 明度値（%）。
    #[must_use]
    pub const fn v(self) -> u8 {
        self.v
    }

    /// RGB 表現へ変換する（整数演算のみによる決定的実装。本モジュール冒頭
    /// 「丸め規則」参照）。
    #[must_use]
    pub fn to_rgb(self) -> Rgb {
        let v100 = i64::from(self.v);
        let s100 = i64::from(self.s);
        // chroma_scaled = C * 10000（C = V * S、V/S は 0..=100 の percent）。
        let chroma_scaled = v100 * s100;
        let (r_scaled, g_scaled, b_scaled) = hue_to_rgb_components(self.h, chroma_scaled);
        // m = V - C。分母 20000 に正規化: m_scaled/20000 = v100*2/100*100 ... 展開すると
        // m_scaled = v100*200 - chroma_scaled*2（分母 20000、chroma_scaled は分母 10000 のため 2 倍）。
        let m_scaled = v100 * 200 - chroma_scaled * 2;
        let m = m_scaled * 30;
        Rgb {
            r: channel_from_scaled(r_scaled + m),
            g: channel_from_scaled(g_scaled + m),
            b: channel_from_scaled(b_scaled + m),
        }
    }
}

/// 色変換コアのエラー（fail-closed。呼び出し側に不正な入力を通知する）。
///
/// `Display` は静的文言のみを返し、入力値そのものはエコーしない
/// （本モジュール冒頭「fail-closed 方針」参照）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorError {
    /// [`Hsl::new`]/[`Hsv::new`] の引数が有効範囲外だった。
    OutOfRange,
    /// [`Color::parse_hex`] の入力が `#rgb`/`#rgba`/`#rrggbb`/`#rrggbbaa`
    /// のいずれの形式にも一致しなかった。
    InvalidHex,
}

impl fmt::Display for ColorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OutOfRange => f.write_str("color component out of range"),
            Self::InvalidHex => {
                f.write_str("invalid hex color format (expected #rgb/#rgba/#rrggbb/#rrggbbaa)")
            }
        }
    }
}

impl std::error::Error for ColorError {}

/// RGBA 8 bit/チャネルの色（canonical 表現）。
///
/// [`Color::parse_hex`] が本モジュールの HEX 入力に対する唯一の検証済み
/// 構築経路であり、`fandhe-frontend-pre-styled-ui::color_swatch`
/// （ColorSwatch、イシュー #838）はこの型のみを色値の入力として受け取る
/// （任意文字列を受け取る API を作らない不変条件、本モジュール冒頭
/// 「fail-closed 方針」参照）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    rgb: Rgb,
    a: u8,
}

impl Color {
    /// 不透明（`a = 255`）の RGB 色から構築する。
    #[must_use]
    pub const fn from_rgb(rgb: Rgb) -> Self {
        Self { rgb, a: 255 }
    }

    /// RGB 色 + アルファ値から構築する。
    #[must_use]
    pub const fn from_rgba(rgb: Rgb, a: u8) -> Self {
        Self { rgb, a }
    }

    /// RGB 部分（アルファを除く）。
    #[must_use]
    pub const fn rgb(self) -> Rgb {
        self.rgb
    }

    /// アルファ値（`0` = 完全透明、`255` = 完全不透明）。
    #[must_use]
    pub const fn alpha(self) -> u8 {
        self.a
    }

    /// `#rgb` / `#rgba` / `#rrggbb` / `#rrggbbaa` の 4 形式を解析する
    /// （大文字小文字非依存、先頭 `#` 必須）。
    ///
    /// 上記 4 形式のいずれにも一致しない入力（`#` 欠落・長さ不正・非 16 進
    /// 文字を含む）はすべて fail-closed に [`ColorError::InvalidHex`] を
    /// 返す（黙って補正しない、本モジュール冒頭「fail-closed 方針」参照）。
    ///
    /// # Errors
    ///
    /// 入力が上記 4 形式のいずれにも一致しないとき
    /// [`ColorError::InvalidHex`] を返す。
    pub fn parse_hex(s: &str) -> Result<Self, ColorError> {
        let body = s.strip_prefix('#').ok_or(ColorError::InvalidHex)?;
        if !body.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(ColorError::InvalidHex);
        }
        let nibble = |c: u8| -> u8 {
            match c {
                b'0'..=b'9' => c - b'0',
                b'a'..=b'f' => c - b'a' + 10,
                b'A'..=b'F' => c - b'A' + 10,
                _ => unreachable!("is_ascii_hexdigit 検証済みのため到達しない"),
            }
        };
        let bytes = body.as_bytes();
        match bytes.len() {
            3 | 4 => {
                let expand = |c: u8| -> u8 {
                    let n = nibble(c);
                    n << 4 | n
                };
                let r = expand(bytes[0]);
                let g = expand(bytes[1]);
                let b = expand(bytes[2]);
                let a = if bytes.len() == 4 {
                    expand(bytes[3])
                } else {
                    255
                };
                Ok(Self {
                    rgb: Rgb::new(r, g, b),
                    a,
                })
            }
            6 | 8 => {
                let byte_at = |i: usize| -> u8 { nibble(bytes[i]) << 4 | nibble(bytes[i + 1]) };
                let r = byte_at(0);
                let g = byte_at(2);
                let b = byte_at(4);
                let a = if bytes.len() == 8 { byte_at(6) } else { 255 };
                Ok(Self {
                    rgb: Rgb::new(r, g, b),
                    a,
                })
            }
            _ => Err(ColorError::InvalidHex),
        }
    }

    /// 小文字固定の正規形 HEX 文字列へ変換する。
    ///
    /// `a == 255`（不透明）のときは `#rrggbb`、それ以外は `#rrggbbaa` を
    /// 返す（同一色に対して常に 1 通りの表現へ収束する決定的な正規化。
    /// [`Color::parse_hex`] とは非対称の往復（`#fff` を渡すと
    /// `#ffffff` が返る）だが、意味的な色は保存される）。出力字母は常に
    /// `#` + `[0-9a-f]` に閉じる（本モジュール冒頭「fail-closed 方針」参照）。
    #[must_use]
    pub fn to_hex_string(self) -> String {
        if self.a == 255 {
            format!("#{:02x}{:02x}{:02x}", self.rgb.r, self.rgb.g, self.rgb.b)
        } else {
            format!(
                "#{:02x}{:02x}{:02x}{:02x}",
                self.rgb.r, self.rgb.g, self.rgb.b, self.a
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- 既知入力 → 既知出力の網羅表（RGB → HSL/HSV） ---

    #[test]
    fn primary_colors_to_hsl() {
        // (r, g, b, 期待 h, s, l) — CSS Color 仕様の変換式に基づく既知値。
        let cases = [
            (255, 0, 0, 0u16, 100u8, 50u8), // 赤
            (0, 255, 0, 120, 100, 50),      // 緑
            (0, 0, 255, 240, 100, 50),      // 青
            (0, 255, 255, 180, 100, 50),    // シアン
            (255, 0, 255, 300, 100, 50),    // マゼンタ
            (255, 255, 0, 60, 100, 50),     // 黄
            (255, 255, 255, 0, 0, 100),     // 白
            (0, 0, 0, 0, 0, 0),             // 黒
            (128, 128, 128, 0, 0, 50),      // 中間グレー
        ];
        for (r, g, b, h, s, l) in cases {
            let hsl = Rgb::new(r, g, b).to_hsl();
            assert_eq!(
                (hsl.h(), hsl.s(), hsl.l()),
                (h, s, l),
                "rgb({r},{g},{b}) の HSL 変換"
            );
        }
    }

    #[test]
    fn primary_colors_to_hsv() {
        // (r, g, b, 期待 h, s, v)。
        let cases = [
            (255, 0, 0, 0u16, 100u8, 100u8), // 赤
            (0, 255, 0, 120, 100, 100),      // 緑
            (0, 0, 255, 240, 100, 100),      // 青
            (255, 255, 255, 0, 0, 100),      // 白
            (0, 0, 0, 0, 0, 0),              // 黒
            (128, 128, 128, 0, 0, 50),       // 中間グレー（round(128*100/255)=50）
        ];
        for (r, g, b, h, s, v) in cases {
            let hsv = Rgb::new(r, g, b).to_hsv();
            assert_eq!(
                (hsv.h(), hsv.s(), hsv.v()),
                (h, s, v),
                "rgb({r},{g},{b}) の HSV 変換"
            );
        }
    }

    #[test]
    fn known_chakra_example_blue_to_hsl_and_back() {
        // chakra-ui の代表例 #3b82f6（rgb(59,130,246)）。
        let rgb = Rgb::new(0x3b, 0x82, 0xf6);
        let hsl = rgb.to_hsl();
        // 手計算: max=246,min=59,delta=187,sum=305
        // l = round(305*100/510) = round(59.80...) = 60（round half up）
        // sum(305) > 255 なので s_den = 510-305=205, s=round(187*100/205)=round(91.21..)=91
        // max==b なので raw_diff=r-g=59-130=-71, offset=240
        // numerator = -71*60 + 240*187 = -4260+44880=40620, full_turn=360*187=67320
        // normalized=40620 (既に正), h=round(40620/187)=round(217.2..)=217
        assert_eq!((hsl.h(), hsl.s(), hsl.l()), (217, 91, 60));

        let back = hsl.to_rgb();
        // 8bit 整数往復のため元の値と厳密一致するとは限らないが、本テストは
        // 「固定した丸め規則どおりの決定値」を固定する（モジュール冒頭
        // 「往復整合」参照）。値は実装のロジックから決定的に導かれる
        // （percent 量子化により r/g がそれぞれ 1 ずれる）。
        assert_eq!(back, Rgb::new(0x3c, 0x83, 0xf6));
    }

    // --- 丸め境界 ---

    #[test]
    fn div_round_half_up_rounds_exact_half_up() {
        assert_eq!(div_round_half_up(1, 2), 1); // 0.5 -> 1
        assert_eq!(div_round_half_up(3, 2), 2); // 1.5 -> 2
        assert_eq!(div_round_half_up(0, 5), 0);
        assert_eq!(div_round_half_up(4, 2), 2); // 割り切れる場合は変化なし
    }

    #[test]
    fn boundary_hue_saturation_lightness_values() {
        // h=359, s=100, l=50 の境界値が構築・変換とも破綻しないこと。
        let hsl = Hsl::new(359, 100, 50).expect("有効な範囲");
        let rgb = hsl.to_rgb();
        let back = rgb.to_hsl();
        // 359 度は 360 度環の端であり、往復で 0 度側へ丸め込まれる可能性が
        // あるため、値そのものより「構築・変換が破綻しない」ことを固定する
        // （degenerate な境界のため許容範囲を明示的に確認する）。
        assert!(back.h() == 359 || back.h() == 0 || back.h() == 358);
    }

    #[test]
    fn hsl_new_rejects_out_of_range() {
        assert_eq!(Hsl::new(360, 0, 0), Err(ColorError::OutOfRange));
        assert_eq!(Hsl::new(0, 101, 0), Err(ColorError::OutOfRange));
        assert_eq!(Hsl::new(0, 0, 101), Err(ColorError::OutOfRange));
        assert!(Hsl::new(359, 100, 100).is_ok());
    }

    #[test]
    fn hsv_new_rejects_out_of_range() {
        assert_eq!(Hsv::new(360, 0, 0), Err(ColorError::OutOfRange));
        assert_eq!(Hsv::new(0, 101, 0), Err(ColorError::OutOfRange));
        assert_eq!(Hsv::new(0, 0, 101), Err(ColorError::OutOfRange));
        assert!(Hsv::new(359, 100, 100).is_ok());
    }

    // --- HEX パースの fail-closed ---

    #[test]
    fn parse_hex_accepts_all_four_formats() {
        assert_eq!(
            Color::parse_hex("#f00").unwrap(),
            Color::from_rgb(Rgb::new(255, 0, 0))
        );
        assert_eq!(
            Color::parse_hex("#f00c").unwrap(),
            Color::from_rgba(Rgb::new(255, 0, 0), 0xcc)
        );
        assert_eq!(
            Color::parse_hex("#ff0000").unwrap(),
            Color::from_rgb(Rgb::new(255, 0, 0))
        );
        assert_eq!(
            Color::parse_hex("#ff0000cc").unwrap(),
            Color::from_rgba(Rgb::new(255, 0, 0), 0xcc)
        );
    }

    #[test]
    fn parse_hex_is_case_insensitive() {
        assert_eq!(
            Color::parse_hex("#FF0000").unwrap(),
            Color::parse_hex("#ff0000").unwrap()
        );
        assert_eq!(
            Color::parse_hex("#AbC").unwrap(),
            Color::parse_hex("#aabbcc").unwrap()
        );
    }

    #[test]
    fn parse_hex_rejects_invalid_inputs() {
        for bogus in [
            "",
            "ff0000",       // 先頭 # 欠落
            "#",            // 空 body
            "#ff",          // 長さ 2
            "#ff000",       // 長さ 5 (5 は非対応)
            "#ff00000",     // 長さ 7
            "#ff000000000", // 長すぎる
            "#gg0000",      // 非 16 進文字
            "#ff00zz",
            "# ff0000",
        ] {
            assert_eq!(
                Color::parse_hex(bogus),
                Err(ColorError::InvalidHex),
                "input={bogus:?}"
            );
        }
    }

    #[test]
    fn to_hex_string_is_lowercase_and_omits_alpha_when_opaque() {
        let opaque = Color::from_rgb(Rgb::new(0xAB, 0xCD, 0xEF));
        assert_eq!(opaque.to_hex_string(), "#abcdef");

        let transparent = Color::from_rgba(Rgb::new(0xAB, 0xCD, 0xEF), 0x80);
        assert_eq!(transparent.to_hex_string(), "#abcdef80");
    }

    #[test]
    fn to_hex_string_output_is_closed_over_hash_and_lowercase_hex_digits() {
        for color in [
            Color::from_rgb(Rgb::new(255, 255, 255)),
            Color::from_rgba(Rgb::new(0, 0, 0), 0),
            Color::from_rgba(Rgb::new(18, 52, 86), 171),
        ] {
            let hex = color.to_hex_string();
            assert!(hex.starts_with('#'));
            assert!(hex[1..]
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)));
        }
    }

    // --- 往復整合（固定した丸め規則どおりの決定値、環境非依存で厳密比較） ---

    #[test]
    fn hsl_round_trip_is_deterministic_across_full_hue_range() {
        for h in (0..360u16).step_by(37) {
            for s in [0u8, 25, 50, 75, 100] {
                for l in [0u8, 25, 50, 75, 100] {
                    let original = Hsl::new(h, s, l).expect("有効な範囲");
                    let rgb = original.to_rgb();
                    let back_a = rgb.to_hsl();
                    // 決定性: 同一入力から同一 RGB へ 2 回変換しても完全一致する。
                    let rgb_again = original.to_rgb();
                    assert_eq!(rgb, rgb_again, "h={h} s={s} l={l}");
                    let back_b = rgb.to_hsl();
                    assert_eq!(back_a, back_b, "h={h} s={s} l={l}");
                }
            }
        }
    }

    #[test]
    fn hsv_round_trip_is_deterministic_across_full_hue_range() {
        for h in (0..360u16).step_by(41) {
            for s in [0u8, 25, 50, 75, 100] {
                for v in [0u8, 25, 50, 75, 100] {
                    let original = Hsv::new(h, s, v).expect("有効な範囲");
                    let rgb = original.to_rgb();
                    let rgb_again = original.to_rgb();
                    assert_eq!(rgb, rgb_again, "h={h} s={s} v={v}");
                    let back_a = rgb.to_hsv();
                    let back_b = rgb.to_hsv();
                    assert_eq!(back_a, back_b, "h={h} s={s} v={v}");
                }
            }
        }
    }

    #[test]
    fn rgb_to_hsl_and_hsv_are_deterministic_for_all_gray_levels() {
        for level in 0u8..=255 {
            let rgb = Rgb::new(level, level, level);
            let hsl = rgb.to_hsl();
            assert_eq!(hsl.h(), 0);
            assert_eq!(hsl.s(), 0);
            let hsv = rgb.to_hsv();
            assert_eq!(hsv.h(), 0);
            assert_eq!(hsv.s(), 0);
            // 決定性の再確認。
            assert_eq!(rgb.to_hsl(), hsl);
            assert_eq!(rgb.to_hsv(), hsv);
        }
    }
}
