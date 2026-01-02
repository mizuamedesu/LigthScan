/// Core GameEngine trait - エンジン共通インターフェース

use super::error::Result;
use super::types::*;
use std::any::Any;

/// ゲームエンジンの共通インターフェース
///
/// このトレイトは UE, Unity, Native など異なるエンジンに対して
/// 統一的なリフレクション・関数呼び出しインターフェースを提供します
pub trait GameEngine: Send + Sync {
    /// エンジン名を返す
    fn name(&self) -> &'static str;

    /// エンジンのバージョン情報
    fn version(&self) -> Option<String> {
        None
    }

    /// 初期化（GNames/GObjects 等の検索）
    ///
    /// # 実装例
    /// - UE: GNames, GObjects, ProcessEvent のアドレスをシグネチャスキャン
    /// - Unity Mono: MonoDomain, Assembly の取得
    /// - Unity IL2CPP: GlobalMetadata, Il2CppDomain の取得
    fn initialize(&mut self) -> Result<()>;

    /// 初期化済みかどうか
    fn is_initialized(&self) -> bool;

    // ====== クラス操作 ======

    /// 名前からクラスを検索
    ///
    /// # 例
    /// ```ignore
    /// let player_class = engine.find_class("PlayerController")?;
    /// ```
    fn find_class(&self, name: &str) -> Result<ClassHandle>;

    /// クラス情報を取得
    fn get_class_info(&self, class: ClassHandle) -> Result<ClassInfo>;

    /// すべてのクラスを列挙（重い操作の可能性あり）
    fn enumerate_classes(&self) -> Result<Vec<ClassInfo>>;

    // ====== メソッド操作 ======

    /// クラスからメソッドを検索
    ///
    /// # 例
    /// ```ignore
    /// let add_gold = engine.find_method(player_class, "AddGold")?;
    /// ```
    fn find_method(&self, class: ClassHandle, name: &str) -> Result<MethodHandle>;

    /// メソッド情報を取得
    fn get_method_info(&self, method: MethodHandle) -> Result<MethodInfo>;

    /// クラスのすべてのメソッドを列挙
    fn enumerate_methods(&self, class: ClassHandle) -> Result<Vec<MethodInfo>>;

    // ====== フィールド操作 ======

    /// クラスからフィールドを検索
    ///
    /// # 例
    /// ```ignore
    /// let health_field = engine.find_field(player_class, "Health")?;
    /// ```
    fn find_field(&self, class: ClassHandle, name: &str) -> Result<FieldHandle>;

    /// フィールド情報を取得
    fn get_field_info(&self, field: FieldHandle) -> Result<FieldInfo>;

    /// クラスのすべてのフィールドを列挙
    fn enumerate_fields(&self, class: ClassHandle) -> Result<Vec<FieldInfo>>;

    // ====== インスタンス操作 ======

    /// クラスのすべてのインスタンスを取得
    ///
    /// # 注意
    /// - UE: GObjects を走査
    /// - Unity: ヒープ走査またはシグネチャスキャン
    /// - Native: サポート困難（空のVecを返す可能性）
    fn get_instances(&self, class: ClassHandle) -> Result<Vec<InstanceHandle>>;

    /// インスタンスのクラスを取得
    fn get_instance_class(&self, instance: InstanceHandle) -> Result<ClassHandle>;

    // ====== 呼び出し・読み書き ======

    /// メソッド呼び出し
    ///
    /// # 引数
    /// - `instance`: インスタンス（static method の場合は None）
    /// - `method`: 呼び出すメソッド
    /// - `args`: 引数のリスト
    ///
    /// # 例
    /// ```ignore
    /// engine.invoke(
    ///     Some(player_instance),
    ///     add_gold_method,
    ///     &[Value::I32(99999)]
    /// )?;
    /// ```
    fn invoke(
        &self,
        instance: Option<InstanceHandle>,
        method: MethodHandle,
        args: &[Value],
    ) -> Result<Value>;

    /// フィールド読み取り
    ///
    /// # 例
    /// ```ignore
    /// let health = engine.read_field(player_instance, health_field)?;
    /// ```
    fn read_field(&self, instance: InstanceHandle, field: FieldHandle) -> Result<Value>;

    /// フィールド書き込み
    ///
    /// # 例
    /// ```ignore
    /// engine.write_field(player_instance, health_field, &Value::F32(999.0))?;
    /// ```
    fn write_field(
        &self,
        instance: InstanceHandle,
        field: FieldHandle,
        value: &Value,
    ) -> Result<()>;

    // ====== ダウンキャスト用 ======

    /// エンジン固有機能にアクセスするためのダウンキャスト
    fn as_any(&self) -> &dyn Any;

    /// ミュータブルなダウンキャスト
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// エンジン検出用のトレイト
pub trait EngineDetector {
    /// プロセスからエンジンを検出して初期化
    fn detect() -> Result<Box<dyn GameEngine>>;

    /// 特定のエンジンかどうかを判定
    fn is_match() -> bool;
}
