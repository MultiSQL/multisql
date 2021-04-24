use {crate::Value, sled::IVec, std::convert::From};

impl From<&IVec> for Value {
    fn from(from: &IVec) -> Self {
        let result = Value::Bytes(from.to_vec());
        result
    }
}

impl From<&Value> for IVec {
    fn from(from: &Value) -> Self {
        if let Value::Bytes(bytes) = from {
            IVec::from(bytes.clone())
        } else {
            panic!("Tried to convert value of non-bytes into IVec")
        }
    }
}
