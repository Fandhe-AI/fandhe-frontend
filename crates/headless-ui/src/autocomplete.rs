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
//! - `webauthn` 単独・`off`/`on` との併用・先頭/中間への配置は型として表現できない。
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
}

/// field 名 + 任意の `webauthn` トークンからなる `autocomplete` 値。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AutocompleteToken {
    field: AutocompleteField,
    webauthn: bool,
}

impl AutocompleteToken {
    /// `webauthn` なしのトークンを作る。
    #[must_use]
    pub const fn new(field: AutocompleteField) -> Self {
        Self {
            field,
            webauthn: false,
        }
    }

    /// 末尾に `webauthn` を付けたトークンを返す。
    #[must_use]
    pub const fn with_webauthn(self) -> Self {
        Self {
            field: self.field,
            webauthn: true,
        }
    }

    /// field 名を返す。
    #[must_use]
    pub const fn field(self) -> AutocompleteField {
        self.field
    }

    /// `webauthn` が付いているかを返す。
    #[must_use]
    pub const fn webauthn(self) -> bool {
        self.webauthn
    }

    /// `autocomplete` 属性値（全組み合わせの静的リテラル）。
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match (self.field, self.webauthn) {
            (AutocompleteField::Username, false) => "username",
            (AutocompleteField::Username, true) => "username webauthn",
            (AutocompleteField::Email, false) => "email",
            (AutocompleteField::Email, true) => "email webauthn",
            (AutocompleteField::CurrentPassword, false) => "current-password",
            (AutocompleteField::CurrentPassword, true) => "current-password webauthn",
            (AutocompleteField::NewPassword, false) => "new-password",
            (AutocompleteField::NewPassword, true) => "new-password webauthn",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIELDS: [AutocompleteField; 4] = [
        AutocompleteField::Username,
        AutocompleteField::Email,
        AutocompleteField::CurrentPassword,
        AutocompleteField::NewPassword,
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
            }
        }
    }
}
