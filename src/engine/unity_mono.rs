/// Unity Mono backend (stub)

use super::error::{EngineError, Result};
use super::types::*;
use super::GameEngine;
use std::any::Any;

pub struct UnityMonoEngine {
    process_handle: usize,
    mono_module: usize,
    root_domain: usize,
    initialized: bool,
}

impl UnityMonoEngine {
    pub fn new(process_handle: usize) -> Self {
        Self {
            process_handle,
            mono_module: 0,
            root_domain: 0,
            initialized: false,
        }
    }
}

impl GameEngine for UnityMonoEngine {
    fn name(&self) -> &'static str {
        "Unity (Mono)"
    }

    fn initialize(&mut self) -> Result<()> {
        Err(EngineError::InitializationFailed(
            "Unity Mono backend not implemented".into(),
        ))
    }

    fn is_initialized(&self) -> bool {
        self.initialized
    }

    fn find_class(&self, name: &str) -> Result<ClassHandle> {
        Err(EngineError::ClassNotFound(name.to_string()))
    }

    fn get_class_info(&self, _class: ClassHandle) -> Result<ClassInfo> {
        Err(EngineError::UnsupportedOperation("Not implemented".into()))
    }

    fn enumerate_classes(&self) -> Result<Vec<ClassInfo>> {
        Ok(Vec::new())
    }

    fn find_method(&self, _class: ClassHandle, name: &str) -> Result<MethodHandle> {
        Err(EngineError::MethodNotFound(name.to_string()))
    }

    fn get_method_info(&self, _method: MethodHandle) -> Result<MethodInfo> {
        Err(EngineError::UnsupportedOperation("Not implemented".into()))
    }

    fn enumerate_methods(&self, _class: ClassHandle) -> Result<Vec<MethodInfo>> {
        Ok(Vec::new())
    }

    fn find_field(&self, _class: ClassHandle, name: &str) -> Result<FieldHandle> {
        Err(EngineError::FieldNotFound(name.to_string()))
    }

    fn get_field_info(&self, _field: FieldHandle) -> Result<FieldInfo> {
        Err(EngineError::UnsupportedOperation("Not implemented".into()))
    }

    fn enumerate_fields(&self, _class: ClassHandle) -> Result<Vec<FieldInfo>> {
        Ok(Vec::new())
    }

    fn get_instances(&self, _class: ClassHandle) -> Result<Vec<InstanceHandle>> {
        Ok(Vec::new())
    }

    fn get_instance_class(&self, _instance: InstanceHandle) -> Result<ClassHandle> {
        Err(EngineError::UnsupportedOperation("Not implemented".into()))
    }

    fn invoke(
        &self,
        _instance: Option<InstanceHandle>,
        _method: MethodHandle,
        _args: &[Value],
    ) -> Result<Value> {
        Err(EngineError::UnsupportedOperation("Not implemented".into()))
    }

    fn read_field(&self, _instance: InstanceHandle, _field: FieldHandle) -> Result<Value> {
        Err(EngineError::UnsupportedOperation("Not implemented".into()))
    }

    fn write_field(
        &self,
        _instance: InstanceHandle,
        _field: FieldHandle,
        _value: &Value,
    ) -> Result<()> {
        Err(EngineError::UnsupportedOperation("Not implemented".into()))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
