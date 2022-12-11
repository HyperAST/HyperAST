use std::fmt::Display;

use hyper_ast::types::Type;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum Primitive {
    Double,
    Float,
    Long,
    Int,
    Char,
    Short,
    Byte,
    Boolean,
    Null,
    Void,
}

impl From<Type> for Primitive {
    fn from(s: Type) -> Self {
        match s {
            Type::BooleanType => Self::Boolean,
            Type::VoidType => Self::Void,
            Type::FloatingPointType => Self::Float,
            Type::IntegralType => Self::Int,
            // Literals
            Type::True => Self::Boolean,
            Type::False => Self::Boolean,
            Type::OctalIntegerLiteral => Self::Int,
            Type::BinaryIntegerLiteral => Self::Int,
            Type::DecimalIntegerLiteral => Self::Int,
            Type::HexFloatingPointLiteral => Self::Float,
            Type::DecimalFloatingPointLiteral => Self::Float,
            Type::HexIntegerLiteral => Self::Float,
            Type::StringLiteral => panic!("{:?}", s),
            Type::CharacterLiteral => Self::Char,
            Type::NullLiteral => Self::Null,
            _ => panic!("{:?}", s),
        }
    }
}

impl From<&str> for Primitive {
    fn from(s: &str) -> Self {
        match s {
            "boolean" => Self::Boolean,
            "void" => Self::Void,
            "float" => Self::Float,
            "double" => Self::Double,
            "byte" => Self::Byte,
            "char" => Self::Char,
            "short" => Self::Short,
            "int" => Self::Int,
            "long" => Self::Long,
            // Literals
            "true" => Self::Boolean,
            "false" => Self::Boolean,
            "null" => Self::Null,
            s => panic!("{:?}", s),
        }
    }
}

impl Display for Primitive {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Primitive::Double => "double",
                Primitive::Float => "float",
                Primitive::Long => "long",
                Primitive::Int => "int",
                Primitive::Char => "char",
                Primitive::Short => "short",
                Primitive::Byte => "byte",
                Primitive::Boolean => "boolean",
                Primitive::Null => "null",
                Primitive::Void => "void",
            }
        )
    }
}

trait SubTyping: PartialOrd {}

impl PartialOrd for Primitive {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        use std::cmp::Ordering;
        let r = match (self, other) {
            (x, y) if x == y => Some(Ordering::Equal),
            (Primitive::Double, Primitive::Double) => Some(Ordering::Equal),
            // double >1 float
            (Primitive::Double, Primitive::Float) => Some(Ordering::Greater),
            (Primitive::Float, Primitive::Float) => Some(Ordering::Equal),
            // float >1 long
            (Primitive::Float, Primitive::Long) => Some(Ordering::Greater),
            (Primitive::Long, Primitive::Long) => Some(Ordering::Equal),
            // long >1 int
            (Primitive::Long, Primitive::Int) => Some(Ordering::Greater),
            (Primitive::Int, Primitive::Int) => Some(Ordering::Equal),
            // int >1 char
            (Primitive::Int, Primitive::Char) => Some(Ordering::Greater),
            // int >1 short
            (Primitive::Int, Primitive::Short) => Some(Ordering::Greater),
            (Primitive::Char, Primitive::Char) => Some(Ordering::Equal),
            (Primitive::Short, Primitive::Short) => Some(Ordering::Equal),
            // short >1 byte
            (Primitive::Short, Primitive::Byte) => Some(Ordering::Greater),
            (Primitive::Byte, Primitive::Byte) => Some(Ordering::Equal),
            (Primitive::Boolean, Primitive::Boolean) => Some(Ordering::Equal),
            (Primitive::Null, Primitive::Null) => Some(Ordering::Equal),
            (Primitive::Void, Primitive::Void) => Some(Ordering::Equal),
            _ => None,
        };
        if r.is_none() {
            other.partial_cmp(self).map(Ordering::reverse)
        } else {
            r
        }
    }
}

impl SubTyping for Primitive {}
