//! 部品横断で使う汎用の小さな修飾型。
//!
//! ここで定義する型は「wireframe/UI キット表現として独立に定義した最小
//! 集合」であり、blocks.pm のプロパティ schema をそのまま転写したもの
//! ではない（ライセンス保留、イシュー #2605 実装計画 §3.2）。
//!
//! 型は 2 系統に分かれる:
//!
//! - 視覚修飾（[`Bold`]/[`Primary`]）: class を付与する
//! - 表示状態（[`Active`]/[`Disabled`]）: `data-*` 属性を付与する
//!   （`docs/design/wireframe-ui-architecture.md` §5「表示状態を示す
//!   `data-*` まで」の責務境界に従う）
//!
//! 対話セマンティクス（`role`/`aria-*`/`tabindex`）に相当する型は
//! 意図的に持たない（同文書 §1/§5/§7）。

/// 太字修飾。`true` のとき class `fw-wire-bold` を付与する。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Bold(pub bool);

impl Bold {
    /// 付与する class（`false` のときは `None`）。
    pub const fn class(self) -> Option<&'static str> {
        if self.0 {
            Some("fw-wire-bold")
        } else {
            None
        }
    }
}

impl From<bool> for Bold {
    fn from(value: bool) -> Self {
        Bold(value)
    }
}

/// 主要（強調）修飾。`true` のとき class `fw-wire-primary` を付与する。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Primary(pub bool);

impl Primary {
    /// 付与する class（`false` のときは `None`）。
    pub const fn class(self) -> Option<&'static str> {
        if self.0 {
            Some("fw-wire-primary")
        } else {
            None
        }
    }
}

impl From<bool> for Primary {
    fn from(value: bool) -> Self {
        Primary(value)
    }
}

/// アクティブ状態。`true` のとき `data-active=""` 属性を付与する。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Active(pub bool);

impl Active {
    /// 付与する属性（`false` のときは `None`）。
    /// `fandhe_frontend_core::attr_if` に委譲する（`data-active` は
    /// core の `is_valid_attr_name` を通過する属性名）。
    pub fn attr(self) -> Option<(String, String)> {
        fandhe_frontend_core::attr_if(self.0, "data-active")
    }
}

impl From<bool> for Active {
    fn from(value: bool) -> Self {
        Active(value)
    }
}

/// 無効状態。`true` のとき `data-disabled=""` 属性を付与する。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Disabled(pub bool);

impl Disabled {
    /// 付与する属性（`false` のときは `None`）。
    pub fn attr(self) -> Option<(String, String)> {
        fandhe_frontend_core::attr_if(self.0, "data-disabled")
    }
}

impl From<bool> for Disabled {
    fn from(value: bool) -> Self {
        Disabled(value)
    }
}

/// 方向。stack / divider / tabs / slider 等が共通で使う。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Orientation {
    /// 水平（既定）。
    #[default]
    Horizontal,
    /// 垂直。
    Vertical,
}

impl Orientation {
    /// 付与する class（`fw-wire-horizontal` / `fw-wire-vertical`）。
    pub const fn class(self) -> &'static str {
        match self {
            Orientation::Horizontal => "fw-wire-horizontal",
            Orientation::Vertical => "fw-wire-vertical",
        }
    }
}
