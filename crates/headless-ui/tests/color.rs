//! `fandhe-frontend-headless-ui` の色変換コア公開 API 統合テスト（イシュー #838）。
//!
//! `crates/headless-ui/src/color.rs` 内の `#[cfg(test)]` 単体テストが実装
//! 詳細（丸めヘルパ・色相環内部計算）を固定するのに対し、本ファイルは
//! [`fandhe_frontend_headless_ui::color`] の**公開 API**（[`Color`]/[`Rgb`]/
//! [`Hsl`]/[`Hsv`]）をクレート外部利用と同じ経路（ルート再エクスポート、
//! `crates/headless-ui/src/lib.rs` の `pub use color::{...}`）で固定する。

use fandhe_frontend_headless_ui::{Color, ColorError, Hsl, Hsv, Rgb};

#[test]
fn root_reexports_are_reachable_without_module_path() {
    // ルート再エクスポート（`fandhe_frontend_headless_ui::{Color, Hsl, Hsv, Rgb}`）
    // が `color::` モジュールパスを介さず使えることを固定する。
    let rgb = Rgb::new(18, 52, 86);
    let hsl: Hsl = rgb.to_hsl();
    let hsv: Hsv = rgb.to_hsv();
    let color = Color::from_rgb(rgb);
    assert_eq!(color.rgb(), rgb);
    assert!(hsl.h() < 360);
    assert!(hsv.h() < 360);
}

#[test]
fn hex_round_trip_via_public_api() {
    let color = Color::parse_hex("#123456").expect("有効な HEX");
    assert_eq!(color.rgb(), Rgb::new(0x12, 0x34, 0x56));
    assert_eq!(color.alpha(), 255);
    assert_eq!(color.to_hex_string(), "#123456");
}

#[test]
fn hex_with_alpha_round_trip_via_public_api() {
    let color = Color::parse_hex("#12345678").expect("有効な HEX");
    assert_eq!(color.rgb(), Rgb::new(0x12, 0x34, 0x56));
    assert_eq!(color.alpha(), 0x78);
    assert_eq!(color.to_hex_string(), "#12345678");
}

#[test]
fn invalid_hex_is_rejected_fail_closed() {
    assert_eq!(Color::parse_hex("not-a-color"), Err(ColorError::InvalidHex));
    assert_eq!(Color::parse_hex(""), Err(ColorError::InvalidHex));
}

#[test]
fn out_of_range_hsl_hsv_are_rejected_fail_closed() {
    assert_eq!(Hsl::new(400, 0, 0), Err(ColorError::OutOfRange));
    assert_eq!(Hsv::new(0, 200, 0), Err(ColorError::OutOfRange));
}

#[test]
fn conversions_are_deterministic_across_repeated_calls() {
    let rgb = Rgb::new(59, 130, 246);
    assert_eq!(rgb.to_hsl(), rgb.to_hsl());
    assert_eq!(rgb.to_hsv(), rgb.to_hsv());

    let hsl = Hsl::new(217, 91, 60).expect("有効な範囲");
    assert_eq!(hsl.to_rgb(), hsl.to_rgb());
}

#[test]
fn color_error_display_is_static_and_does_not_echo_input() {
    // エラーメッセージが入力値をエコーしない（機微情報露出防止方針）ことを
    // 固定する: 攻撃者が制御しうる入力文字列断片が Display 出力へ現れない。
    let attacker_controlled = "<script>alert(1)</script>";
    let err = Color::parse_hex(attacker_controlled).unwrap_err();
    let message = err.to_string();
    assert!(!message.contains("script"));
    assert_eq!(
        message,
        "invalid hex color format (expected #rgb/#rgba/#rrggbb/#rrggbbaa)"
    );
}
