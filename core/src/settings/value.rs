use std::{fmt, ops};
use std::collections::HashMap;
use yaml_rust::yaml::{self, Yaml};

use settings::Error;

pub enum Value {
    Real(f64),
    Integer(i64),
    String(String),
    Boolean(bool),
    Array(ValueArray),
    Map(ValueMap),
    None,
    NotFound,
}

pub struct ValueArray(Vec<Value>);
pub struct ValueMap(HashMap<String, Value>);

impl Value {
    pub fn as_f64(&self) -> Option<f64> {
        if let &Value::Real(r) = self {
            Some(r)
        } else {
            None
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        if let &Value::Integer(i) = self {
            Some(i)
        } else {
            None
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        if let &Value::String(ref s) = self {
            Some(s)
        } else {
            None
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        if let &Value::Boolean(b) = self {
            Some(b)
        } else {
            None
        }
    }

    pub fn as_array(&self) -> Option<&ValueArray> {
        if let &Value::Array(ref a) = self {
            Some(a)
        } else {
            None
        }
    }

    pub fn as_map(&self) -> Option<&ValueMap> {
        if let &Value::Map(ref m) = self {
            Some(m)
        } else {
            None
        }
    }

    pub fn is_valid(&self) -> bool {
        if let &Value::NotFound = self {
            false
        } else {
            true
        }
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Value::Real(r) => r.fmt(f),
            &Value::Integer(i) => i.fmt(f),
            &Value::String(ref s) => s.fmt(f),
            &Value::Boolean(b) => b.fmt(f),
            &Value::Array(ref a) => a.fmt(f),
            &Value::Map(ref m) => m.fmt(f),
            &Value::None => write!(f, "None"),
            &Value::NotFound => write!(f, "NotFound"),
        }
    }
}

impl fmt::Debug for ValueArray {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::Debug for ValueMap {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

pub static NOT_FOUND: &'static Value = &Value::NotFound;

impl<'a> ops::Index<&'a str> for Value {
    type Output = Value;
    fn index(&self, i: &'a str) -> &Value {
        self.as_map()
            .map(|m| &m[i])
            .unwrap_or(NOT_FOUND)
    }
}

impl ops::Index<usize> for Value {
    type Output = Value;
    fn index(&self, i: usize) -> &Value {
        self.as_array()
            .map(|a| &a[i])
            .unwrap_or(NOT_FOUND)
    }
}

impl ops::Index<usize> for ValueArray {
    type Output = Value;
    fn index(&self, i: usize) -> &Value {
        self.0.get(i).unwrap_or(NOT_FOUND)
    }
}

impl<'a> ops::Index<&'a str> for ValueMap {
    type Output = Value;
    fn index(&self, i: &'a str) -> &Value {
        self.0.get(i).unwrap_or(NOT_FOUND)
    }
}

impl Value {
    #[doc(hidden)]
    pub fn from(v: Yaml) -> Result<Self, Error> {
        Ok(match v {
            Yaml::Real(s) => Value::Real(parse_float(s)),
            Yaml::Integer(i) => Value::Integer(i),
            Yaml::String(s) => Value::String(s),
            Yaml::Boolean(b) => Value::Boolean(b),
            Yaml::Array(a) => Value::Array(try!(ValueArray::from(a))),
            Yaml::Hash(h) => Value::Map(try!(ValueMap::from(h))),
            Yaml::Alias(_) => panic!("yaml aliases are not supported"),
            Yaml::Null => Value::None,
            Yaml::BadValue => Value::NotFound,
        })
    }

    #[doc(hidden)]
    pub fn override_with(&mut self, v: Yaml) -> Result<(), Error> {
        let v = match self {
            &mut Value::Map(ref mut m) => {
                match v {
                    Yaml::Hash(h) => return m.override_with(h),
                    _ => v,
                }
            }
            _ => v,
        };

        *self = try!(Self::from(v));
        Ok(())
    }
}

fn parse_float(s: String) -> f64 {
    match s.parse::<f64>() {
        Ok(v) => v,
        Err(_) => panic!("yaml float couldn't be parsed"),
    }
}

impl ValueArray {
    #[doc(hidden)]
    pub fn from(a: yaml::Array) -> Result<Self, Error> {
        let mut array = Vec::with_capacity(a.len());
        for v in a {
            array.push(try!(Value::from(v)));
        }

        Ok(ValueArray(array))
    }
}

impl ValueMap {
    #[doc(hidden)]
    pub fn empty() -> Self {
        ValueMap(HashMap::new())
    }

    #[doc(hidden)]
    pub fn from(h: yaml::Hash) -> Result<Self, Error> {
        let mut map = HashMap::with_capacity(h.len());
        for (k, v) in h {
            match k {
                Yaml::String(s) => {
                    map.insert(s, try!(Value::from(v)));
                }
                _ => return Err(Error::InvalidKey),
            }
        }

        Ok(ValueMap(map))
    }

    #[doc(hidden)]
    pub fn override_with(&mut self, h: yaml::Hash) -> Result<(), Error> {
        for (k, v) in h {
            match k {
                Yaml::String(ref s) => {
                    if let Some(current) = self.0.get_mut(s) {
                        try!(current.override_with(v));
                    } else {
                        return Err(Error::InvalidOverride);
                    }
                }
                _ => return Err(Error::InvalidKey),
            }
        }

        Ok(())
    }
}
