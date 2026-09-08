use boml::types::TomlValue;

pub enum Value {
    String(String),
    Integer(i64),
    Float(f64),
}

impl Value {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_integer(&self) -> Option<i64> {
        match self {
            Self::Integer(i) => Some(*i),
            _ => None,
        }
    }

    pub fn as_float(&self) -> Option<f64> {
        match self {
            Self::Float(f) => Some(*f),
            Self::Integer(i) => Some(*i as f64),
            _ => None,
        }
    }

    pub fn from_boml(val: &TomlValue) -> Option<Self> {
        match val {
            TomlValue::String(s) => Some(Self::String(s.as_str().to_owned())),
            TomlValue::Integer(i) => Some(Self::Integer(*i)),
            TomlValue::Float(f) => Some(Self::Float(*f)),
            _ => None,
        }
    }
}
