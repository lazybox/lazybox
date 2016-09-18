use std::{fmt, ops};
use std::collections::HashMap;
use yaml_rust::yaml::{self, Yaml};

use Error;

pub enum Value {
    Real(f64),
    Integer(i64),
    String(String),
    Boolean(bool),
    Array(ValueArray),
    Map(ValueMap),
    None,
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

pub static NONE: &'static Value = &Value::None;

impl<'a> ops::Index<&'a str> for Value {
    type Output = Value;
    fn index(&self, i: &'a str) -> &Value {
        self.as_map()
            .map(|m| &m[i])
            .unwrap_or(NONE)
    }
}

impl ops::Index<usize> for Value {
    type Output = Value;
    fn index(&self, i: usize) -> &Value {
        self.as_array()
            .map(|a| &a[i])
            .unwrap_or(NONE)
    }
}

impl ops::Index<usize> for ValueArray {
    type Output = Value;
    fn index(&self, i: usize) -> &Value {
        self.0.get(i).unwrap_or(NONE)
    }
}

impl<'a> ops::Index<&'a str> for ValueMap {
    type Output = Value;
    fn index(&self, i: &'a str) -> &Value {
        self.0.get(i).unwrap_or(NONE)
    }
}

impl Value {
    pub(crate) fn from(v: Yaml) -> Result<Self, Error> {
        Ok(match v {
            Yaml::Real(s) => Value::Real(parse_float(s)),
            Yaml::Integer(i) => Value::Integer(i),
            Yaml::String(s) => Value::String(s),
            Yaml::Boolean(b) => Value::Boolean(b),
            Yaml::Array(a) => Value::Array(try!(ValueArray::from(a))),
            Yaml::Hash(h) => Value::Map(try!(ValueMap::from(h))),
            Yaml::Alias(_) => panic!("yaml aliases are not supported"),
            Yaml::Null => Value::None,
            Yaml::BadValue => Value::None,
        })
    }

    pub(crate) fn override_with(&mut self, v: Yaml) -> Result<(), Error> {
        macro_rules! expect {
            ($p:pat = $v:expr => $e:expr) => {
                match $v {
                    $p => $e,
                    _ => return Err(Error::OverrideMismatch),
                }
            }
        }

        match self {
            &mut Value::Real(ref mut p) =>
                expect!(Yaml::Real(r) = v => *p = parse_float(r)),
            &mut Value::Integer(ref mut p) =>
                expect!(Yaml::Integer(i) = v => *p = i),
            &mut Value::String(ref mut p) =>
                expect!(Yaml::String(s) = v => *p = s),
            &mut Value::Boolean(ref mut p) =>
                expect!(Yaml::Boolean(b) = v => *p = b),
            &mut Value::Array(ref mut p) =>
                expect!(Yaml::Array(a) = v => try!(p.override_with(a))),
            &mut Value::Map(ref mut p) =>
                expect!(Yaml::Hash(h) = v => try!(p.override_with(h))),
            &mut Value::None => return Err(Error::NoneOverride),
        }

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
    pub(crate) fn from(a: yaml::Array) -> Result<Self, Error> {
        let mut array = Vec::with_capacity(a.len());
        for v in a {
            array.push(try!(Value::from(v)));
        }

        Ok(ValueArray(array))
    }

    pub(crate) fn override_with(&mut self, a: yaml::Array) -> Result<(), Error>{
        self.0.clear();
        for v in a {
            self.0.push(try!(Value::from(v)));
        }

        Ok(())
    }
}

impl ValueMap {
    pub(crate) fn empty() -> Self {
        ValueMap(HashMap::new())
    }

    pub(crate) fn from(h: yaml::Hash) -> Result<Self, Error> {
        let mut map = HashMap::with_capacity(h.len());
        for (k, v) in h {
            match k {
                Yaml::String(s) => { map.insert(s, try!(Value::from(v))); }
                _ => return Err(Error::InvalidKey),
            }
        }

        Ok(ValueMap(map))
    }

    pub(crate) fn override_with(&mut self, h: yaml::Hash) -> Result<(), Error> {
        for (k, v) in h {
            match k {
                Yaml::String(ref s) =>
                    if let Some(current) = self.0.get_mut(s) {
                       try!(current.override_with(v));
                    } else {
                        return Err(Error::NoneOverride);
                    },
                _ => return Err(Error::InvalidKey),
            }
        }

        Ok(())
    }
}
