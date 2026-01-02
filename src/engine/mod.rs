/// Game Engine abstraction layer
///
/// このモジュールは異なるゲームエンジン（UE, Unity, Native等）に対して
/// 統一的なリフレクション・関数呼び出しインターフェースを提供します

pub mod error;
pub mod types;
#[allow(clippy::module_inception)]
pub mod r#trait;

// エンジン実装（後で追加）
pub mod unreal;
pub mod unity_mono;
pub mod unity_il2cpp;
pub mod native;

// Re-exports
pub use error::{EngineError, Result};
pub use r#trait::{EngineDetector, GameEngine};
pub use types::*;
