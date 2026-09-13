//! アニメーション計算結果の書き込み先を抽象化する `Target<T>`（イシュー #2378）。
//!
//! `Spring::at` / `Keyframes::at` 等が返す値をどこへ書くか（DOM プロパティ・
//! CSS カスタムプロパティ・任意のフィールド）は本 crate の関心事ではない。
//! `fandhe-frontend-animation`（#2417/#2403）が Web 実装（`web-sys` 経由の
//! DOM Target）を提供し、本 crate 側は「値を渡せば書き込まれる」という
//! 契約のみを定義する。

/// 計算済みの値 `T` を書き込む先。
///
/// `write` は毎フレーム呼ばれる想定のため、実装側は panic しない・
/// エラーを返さない（書き込み失敗は実装側が黙って無視するか、または
/// 実装側の責務でログ等に逃がす）ことを前提とする。
pub trait Target<T> {
    /// 値を書き込む。
    fn write(&mut self, value: T);
}

/// テスト用の参照実装: 書き込まれた値を到着順に蓄積するだけの `Target`。
///
/// native テスト（本 crate 内・後続クレートの結合テスト）が「Driver から
/// 駆動された値が期待どおりの順序・内容で書き込まれたか」を検証するために
/// 使う。`test-utils` feature 経由で crate 外（`fandhe-frontend-animation`
/// 等）のテストコードからも利用できる。
#[cfg(any(test, feature = "test-utils"))]
#[derive(Debug, Clone)]
pub struct RecordingTarget<T> {
    values: Vec<T>,
}

#[cfg(any(test, feature = "test-utils"))]
impl<T> Default for RecordingTarget<T> {
    fn default() -> Self {
        Self { values: Vec::new() }
    }
}

#[cfg(any(test, feature = "test-utils"))]
impl<T> RecordingTarget<T> {
    /// 空の状態で生成する。
    pub fn new() -> Self {
        Self::default()
    }

    /// これまでに書き込まれた値を到着順に返す。
    pub fn values(&self) -> &[T] {
        &self.values
    }

    /// 所有権ごと取り出す。
    pub fn into_values(self) -> Vec<T> {
        self.values
    }
}

#[cfg(any(test, feature = "test-utils"))]
impl<T> Target<T> for RecordingTarget<T> {
    fn write(&mut self, value: T) {
        self.values.push(value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recording_target_starts_empty() {
        let target: RecordingTarget<f64> = RecordingTarget::new();
        assert!(target.values().is_empty());
    }

    #[test]
    fn recording_target_appends_in_write_order() {
        let mut target = RecordingTarget::new();
        target.write(1.0);
        target.write(2.0);
        target.write(3.0);
        assert_eq!(target.values(), &[1.0, 2.0, 3.0]);
    }

    #[test]
    fn recording_target_into_values_transfers_ownership() {
        let mut target = RecordingTarget::new();
        target.write("a".to_string());
        target.write("b".to_string());
        assert_eq!(target.into_values(), vec!["a".to_string(), "b".to_string()]);
    }
}
