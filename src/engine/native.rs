/// Native (non-engine) backend

use super::error::{EngineError, Result};
use super::types::*;
use super::GameEngine;
use std::any::Any;
use std::collections::HashMap;

/// Native バックエンド（リフレクション機能が限定的）
pub struct NativeEngine {
    process_handle: usize,
    /// PE Export Table から取得したシンボル
    symbols: HashMap<String, usize>,
    initialized: bool,
}

impl NativeEngine {
    pub fn new(process_handle: usize) -> Self {
        Self {
            process_handle,
            symbols: HashMap::new(),
            initialized: false,
        }
    }

    /// PE Export Table を解析
    fn parse_export_table(&mut self) -> Result<()> {
        // TODO: PE ヘッダーから Export Table を読み取り
        // symbols に関数名 -> アドレスのマッピングを構築
        Ok(())
    }
}

impl GameEngine for NativeEngine {
    fn name(&self) -> &'static str {
        "Native"
    }

    fn initialize(&mut self) -> Result<()> {
        if self.initialized {
            return Ok(());
        }

        self.parse_export_table()?;
        self.initialized = true;
        Ok(())
    }

    fn is_initialized(&self) -> bool {
        self.initialized
    }

    fn find_class(&self, name: &str) -> Result<ClassHandle> {
        // Native にはクラスの概念がないため、
        // シンボル名のプレフィックスで擬似的に対応
        Err(EngineError::ClassNotFound(format!(
            "Native engine has limited class support: {}",
            name
        )))
    }

    fn get_class_info(&self, _class: ClassHandle) -> Result<ClassInfo> {
        Err(EngineError::UnsupportedOperation(
            "Native engine does not support class info".into(),
        ))
    }

    fn enumerate_classes(&self) -> Result<Vec<ClassInfo>> {
        Ok(Vec::new())
    }

    fn find_method(&self, _class: ClassHandle, name: &str) -> Result<MethodHandle> {
        // シンボルテーブルから検索
        self.symbols
            .get(name)
            .map(|&addr| MethodHandle(addr))
            .ok_or_else(|| EngineError::MethodNotFound(name.to_string()))
    }

    fn get_method_info(&self, method: MethodHandle) -> Result<MethodInfo> {
        // シンボル名を逆引き
        let name = self
            .symbols
            .iter()
            .find(|(_, &addr)| addr == method.0)
            .map(|(n, _)| n.clone())
            .unwrap_or_else(|| format!("func_0x{:X}", method.0));

        Ok(MethodInfo {
            name,
            handle: method,
            params: Vec::new(), // Native では型情報不明
            return_type: None,
            is_static: true, // すべて static として扱う
        })
    }

    fn enumerate_methods(&self, _class: ClassHandle) -> Result<Vec<MethodInfo>> {
        // すべてのシンボルを列挙
        Ok(self
            .symbols
            .iter()
            .map(|(name, &addr)| MethodInfo {
                name: name.clone(),
                handle: MethodHandle(addr),
                params: Vec::new(),
                return_type: None,
                is_static: true,
            })
            .collect())
    }

    fn find_field(&self, _class: ClassHandle, name: &str) -> Result<FieldHandle> {
        Err(EngineError::FieldNotFound(format!(
            "Native engine does not support field lookup: {}",
            name
        )))
    }

    fn get_field_info(&self, _field: FieldHandle) -> Result<FieldInfo> {
        Err(EngineError::UnsupportedOperation(
            "Native engine does not support field info".into(),
        ))
    }

    fn enumerate_fields(&self, _class: ClassHandle) -> Result<Vec<FieldInfo>> {
        Ok(Vec::new())
    }

    fn get_instances(&self, _class: ClassHandle) -> Result<Vec<InstanceHandle>> {
        // Native ではインスタンス列挙は不可能
        Ok(Vec::new())
    }

    fn get_instance_class(&self, _instance: InstanceHandle) -> Result<ClassHandle> {
        Err(EngineError::UnsupportedOperation(
            "Native engine does not support instance class".into(),
        ))
    }

    fn invoke(
        &self,
        _instance: Option<InstanceHandle>,
        _method: MethodHandle,
        _args: &[Value],
    ) -> Result<Value> {
        // TODO: CreateRemoteThread + シェルコード生成
        // 呼び出し規約（stdcall/cdecl/fastcall）を考慮する必要がある
        Err(EngineError::UnsupportedOperation(
            "Native method invocation not implemented".into(),
        ))
    }

    fn read_field(&self, _instance: InstanceHandle, _field: FieldHandle) -> Result<Value> {
        Err(EngineError::UnsupportedOperation(
            "Native field reading not supported".into(),
        ))
    }

    fn write_field(
        &self,
        _instance: InstanceHandle,
        _field: FieldHandle,
        _value: &Value,
    ) -> Result<()> {
        Err(EngineError::UnsupportedOperation(
            "Native field writing not supported".into(),
        ))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
