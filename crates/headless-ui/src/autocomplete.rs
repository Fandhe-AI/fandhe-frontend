//! `autocomplete` 属性値の固定語彙型（`webauthn` 検出トークン対応、イシュー #3815）。
//!
//! HTML Standard の autofill detail tokens では、`input` の `autocomplete` は
//! field 名の後ろに任意の `webauthn` トークンを付けられる
//! （例: `current-password webauthn`、passkey の conditional mediation に必要）。
//! 本モジュールは「field 名 + webauthn の有無」を enum/bool で表し、
//! 出力文字列を全組み合わせの `&'static str` リテラルとして返すことで、
//! 任意文字列が `autocomplete` へ混入する経路を型で閉じる
//! （`.claude/rules/security.md` A05）。
//!
//! # セキュリティ不変条件
//!
//! - 任意 `&str` を受ける公開コンストラクタ・`FromStr`・`From<&str>` を持たない。
//! - 文字列連結・`format!` を使わず、[`AutocompleteToken::as_str`] は
//!   リテラルの `match` のみで値を返す。
//! - `webauthn` 単独・`off`/`on` との併用（`off webauthn`）・`one-time-code webauthn`・先頭/中間への配置は型として表現できない。`off` は単独値としてのみ表せる（イシュー #3859）。
//!
//! # 呼び出し文脈
//!
//! [`crate::password_input::input_with_autocomplete`] と
//! [`crate::field::input_with_autocomplete`]（イシュー #3816）が受け取り、
//! `autocomplete` 属性へ出力する。`AutocompleteField` は `#[non_exhaustive]`
//! で、今後の field 名追加は非破壊。`email webauthn` も仕様上有効なため許容する。
//! `new-password webauthn` は仕様上は有効なため許容するが、通常は
//! `current-password` / `username` と併用する。

/// `autocomplete` の field 名（固定語彙）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum AutocompleteField {
    /// `username`。
    Username,
    /// `email`（Contact カテゴリ。`email webauthn` も文法上は有効だが通常は使わない、イシュー #3816）。
    Email,
    /// `current-password`。
    CurrentPassword,
    /// `new-password`。
    NewPassword,
    /// `one-time-code`（ワンタイムコード欄。`webauthn` とは併用しない、イシュー #3859）。
    OneTimeCode,
}

/// トークンの内部表現。`off` は field 名ではなく autofill expectation name のため、
/// field + webauthn の組とは別 variant にして `off webauthn` を型で表現不能にする。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Inner {
    Off,
    Field {
        field: AutocompleteField,
        webauthn: bool,
    },
}

/// field 名 + 任意の `webauthn` トークン、または単独の `off` からなる `autocomplete` 値。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AutocompleteToken {
    inner: Inner,
}

impl AutocompleteToken {
    /// `webauthn` なしのトークンを作る。
    #[must_use]
    pub const fn new(field: AutocompleteField) -> Self {
        Self {
            inner: Inner::Field {
                field,
                webauthn: false,
            },
        }
    }

    /// 単独の `off`（autofill expectation name。`webauthn` 等の detail token とは併用不可）。
    #[must_use]
    pub const fn off() -> Self {
        Self { inner: Inner::Off }
    }

    /// `off` かを返す。`off` のとき [`field`](Self::field) は意味を持たないため先に判定する。
    #[must_use]
    pub const fn is_off(self) -> bool {
        matches!(self.inner, Inner::Off)
    }

    /// 末尾に `webauthn` を付けたトークンを返す。
    ///
    /// `off` と [`AutocompleteField::OneTimeCode`] では仕様上意味を持たず
    /// 不正値（`off webauthn` 等）になるため無視（no-op）する。
    #[must_use]
    pub const fn with_webauthn(self) -> Self {
        match self.inner {
            Inner::Field { field, .. } if !matches!(field, AutocompleteField::OneTimeCode) => {
                Self {
                    inner: Inner::Field {
                        field,
                        webauthn: true,
                    },
                }
            }
            _ => self,
        }
    }

    /// field 名を返す。`off` のときは意味を持たない値（`Username`）を返すため、
    /// 先に [`is_off`](Self::is_off) で判定すること。
    #[must_use]
    pub const fn field(self) -> AutocompleteField {
        match self.inner {
            Inner::Off => AutocompleteField::Username,
            Inner::Field { field, .. } => field,
        }
    }

    /// `webauthn` が付いているかを返す（`off` では常に `false`）。
    #[must_use]
    pub const fn webauthn(self) -> bool {
        match self.inner {
            Inner::Off => false,
            Inner::Field { webauthn, .. } => webauthn,
        }
    }

    /// `autocomplete` 属性値（全組み合わせの静的リテラル）。
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self.inner {
            Inner::Off => "off",
            Inner::Field { field, webauthn } => match (field, webauthn) {
                (AutocompleteField::Username, false) => "username",
                (AutocompleteField::Username, true) => "username webauthn",
                (AutocompleteField::Email, false) => "email",
                (AutocompleteField::Email, true) => "email webauthn",
                (AutocompleteField::CurrentPassword, false) => "current-password",
                (AutocompleteField::CurrentPassword, true) => "current-password webauthn",
                (AutocompleteField::NewPassword, false) => "new-password",
                (AutocompleteField::NewPassword, true) => "new-password webauthn",
                (AutocompleteField::OneTimeCode, _) => "one-time-code",
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIELDS: [AutocompleteField; 5] = [
        AutocompleteField::Username,
        AutocompleteField::Email,
        AutocompleteField::CurrentPassword,
        AutocompleteField::NewPassword,
        AutocompleteField::OneTimeCode,
    ];

    #[test]
    fn all_combinations_are_fixed_strings() {
        let expected = [
            "username",
            "username webauthn",
            "email",
            "email webauthn",
            "current-password",
            "current-password webauthn",
            "new-password",
            "new-password webauthn",
            "one-time-code",
            "one-time-code",
        ];
        let mut got = Vec::new();
        for f in FIELDS {
            let t = AutocompleteToken::new(f);
            got.push(t.as_str());
            got.push(t.with_webauthn().as_str());
        }
        assert_eq!(got, expected);
    }

    #[test]
    fn off_is_standalone_and_ignores_webauthn() {
        let off = AutocompleteToken::off();
        assert!(off.is_off());
        assert_eq!(off.as_str(), "off");
        assert_eq!(off.with_webauthn().as_str(), "off");
        assert!(!off.with_webauthn().webauthn());
        assert!(!AutocompleteToken::new(AutocompleteField::Username).is_off());
    }

    #[test]
    fn one_time_code_ignores_webauthn() {
        let t = AutocompleteToken::new(AutocompleteField::OneTimeCode);
        assert_eq!(t.with_webauthn(), t);
        assert!(!t.with_webauthn().webauthn());
        assert_eq!(t.field(), AutocompleteField::OneTimeCode);
    }

    #[test]
    fn webauthn_only_appears_as_trailing_token() {
        for f in FIELDS {
            for t in [
                AutocompleteToken::new(f),
                AutocompleteToken::new(f).with_webauthn(),
            ] {
                let toks: Vec<&str> = t.as_str().split(' ').collect();
                assert!(!toks.is_empty() && toks.len() <= 2);
                assert_ne!(toks[0], "webauthn");
                assert_eq!(t.webauthn(), toks.get(1) == Some(&"webauthn"));
                assert_eq!(t.field(), f);
                assert!(!t.is_off());
            }
        }
    }
}
