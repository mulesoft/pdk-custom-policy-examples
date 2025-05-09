use pdk::hl::PropertyAccessor;
use pdk::script::AttributesBinding;
use std::collections::HashMap;
use std::slice::Iter;

pub(crate) const SOURCE_ADDRESS: &[&str] = &["source", "address"];
pub(crate) const DESTINATION_ADDRESS: &[&str] = &["destination", "address"];
pub(crate) const REQUEST_SCHEME: &[&str] = &["request", "scheme"];
pub(crate) const REQUEST_PROTOCOL: &[&str] = &["request", "protocol"];

pub struct HeadersAttributes<'a> {
    headers: Vec<(String, String)>,
    properties: &'a dyn PropertyAccessor,
}

impl<'a> HeadersAttributes<'a> {
    pub fn new(headers: Vec<(String, String)>, properties: &'a dyn PropertyAccessor) -> Self {
        HeadersAttributes {
            headers,
            properties,
        }
    }
}

impl<'a> AttributesBinding for HeadersAttributes<'a> {
    fn extract_headers(&self) -> HashMap<String, String> {
        let vec: &Vec<(String, String)> = &self.headers;
        let iter: Iter<(String, String)> = vec.into_iter();
        let x: HashMap<String, String> = iter.map(|p| p.clone()).collect();
        x.clone()
    }

    fn extract_header(&self, key: &str) -> Option<String> {
        let x = &self
            .headers
            .iter()
            .find(|&(k, _)| k.eq_ignore_ascii_case(key))
            .map(|(_, v)| v.clone());
        x.clone()
    }

    fn remote_address(&self) -> Option<String> {
        self.properties
            .read_property(SOURCE_ADDRESS)
            .map(|bytes| String::from_utf8_lossy(bytes.as_slice()).to_string())
    }

    fn local_address(&self) -> Option<String> {
        self.properties
            .read_property(DESTINATION_ADDRESS)
            .map(|bytes| String::from_utf8_lossy(bytes.as_slice()).to_string())
    }

    fn scheme(&self) -> Option<String> {
        self.properties
            .read_property(REQUEST_SCHEME)
            .map(|bytes| String::from_utf8_lossy(bytes.as_slice()).to_string())
    }

    fn version(&self) -> Option<String> {
        self.properties
            .read_property(REQUEST_PROTOCOL)
            .map(|bytes| String::from_utf8_lossy(bytes.as_slice()).to_string())
    }
}
