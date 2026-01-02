/// Engine abstraction types - エンジン非依存の型定義

use std::fmt;

// ============================================
// ハンドル型（エンジン非依存）
// ============================================

/// クラス/型へのハンドル
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ClassHandle(pub usize);

/// メソッド/関数へのハンドル
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MethodHandle(pub usize);

/// フィールド/プロパティへのハンドル
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FieldHandle(pub usize);

/// オブジェクトインスタンスへのハンドル
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct InstanceHandle(pub usize);

// ============================================
// 値の抽象化
// ============================================

/// エンジン非依存の値表現
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    F32(f32),
    F64(f64),
    String(String),
    Object(InstanceHandle),
    Array(Vec<Value>),
    /// 生バイト（エンジン固有の複雑な構造体）
    Struct(Vec<u8>),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Null => write!(f, "null"),
            Value::Bool(v) => write!(f, "{}", v),
            Value::I8(v) => write!(f, "{}", v),
            Value::I16(v) => write!(f, "{}", v),
            Value::I32(v) => write!(f, "{}", v),
            Value::I64(v) => write!(f, "{}", v),
            Value::U8(v) => write!(f, "{}", v),
            Value::U16(v) => write!(f, "{}", v),
            Value::U32(v) => write!(f, "{}", v),
            Value::U64(v) => write!(f, "{}", v),
            Value::F32(v) => write!(f, "{}", v),
            Value::F64(v) => write!(f, "{}", v),
            Value::String(v) => write!(f, "\"{}\"", v),
            Value::Object(h) => write!(f, "Object@0x{:X}", h.0),
            Value::Array(v) => write!(f, "[{} items]", v.len()),
            Value::Struct(v) => write!(f, "Struct[{} bytes]", v.len()),
        }
    }
}

// ============================================
// リフレクション情報
// ============================================

/// クラス情報
#[derive(Clone, Debug)]
pub struct ClassInfo {
    pub name: String,
    pub handle: ClassHandle,
    pub parent: Option<ClassHandle>,
    pub size: usize,
}

/// メソッド情報
#[derive(Clone, Debug)]
pub struct MethodInfo {
    pub name: String,
    pub handle: MethodHandle,
    pub params: Vec<ParamInfo>,
    pub return_type: Option<TypeInfo>,
    pub is_static: bool,
}

/// フィールド情報
#[derive(Clone, Debug)]
pub struct FieldInfo {
    pub name: String,
    pub handle: FieldHandle,
    pub offset: usize,
    pub type_info: TypeInfo,
}

/// パラメータ情報
#[derive(Clone, Debug)]
pub struct ParamInfo {
    pub name: String,
    pub type_info: TypeInfo,
}

/// 型情報
#[derive(Clone, Debug)]
pub struct TypeInfo {
    pub name: String,
    pub size: usize,
    pub kind: TypeKind,
}

impl PartialEq for TypeInfo {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.size == other.size && self.kind == other.kind
    }
}

/// 型の種類
#[derive(Clone, Debug)]
pub enum TypeKind {
    Primitive(PrimitiveType),
    Class(ClassHandle),
    Struct(ClassHandle),
    Array(Box<TypeInfo>),
    Pointer(Box<TypeInfo>),
    Unknown,
}

impl PartialEq for TypeKind {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (TypeKind::Primitive(a), TypeKind::Primitive(b)) => a == b,
            (TypeKind::Class(a), TypeKind::Class(b)) => a == b,
            (TypeKind::Struct(a), TypeKind::Struct(b)) => a == b,
            (TypeKind::Array(a), TypeKind::Array(b)) => **a == **b,
            (TypeKind::Pointer(a), TypeKind::Pointer(b)) => **a == **b,
            (TypeKind::Unknown, TypeKind::Unknown) => true,
            _ => false,
        }
    }
}

/// プリミティブ型
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrimitiveType {
    Bool,
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    F32,
    F64,
}

impl PrimitiveType {
    pub fn size(&self) -> usize {
        match self {
            PrimitiveType::Bool | PrimitiveType::I8 | PrimitiveType::U8 => 1,
            PrimitiveType::I16 | PrimitiveType::U16 => 2,
            PrimitiveType::I32 | PrimitiveType::U32 | PrimitiveType::F32 => 4,
            PrimitiveType::I64 | PrimitiveType::U64 | PrimitiveType::F64 => 8,
        }
    }
}
