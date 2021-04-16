use {
    super::ValueCore,
    crate::{Convert, ConvertFrom, Result, Value, ValueError},
};

// These were using references, they now consume their variables. See ::recipe.
macro_rules! natural_binary_op {
    ($name: ident, $trait: ident, $op: tt) => {
        pub fn $name<Core>(self, other: Self) -> Result<Self>
        where
            Core: ValueCore + $trait<Output = Core>,
        {
            let (left, right) = (Core::convert_from(self)?, Core::convert_from(other)?);
            let result = left $op right;
            Ok(result.into())
        }
    };
}
macro_rules! natural_binary_ops {
    ($(($name: ident, $trait: ident, $op: tt, $generic_name: ident)),+) => {
        use std::ops::{$($trait),+};
        impl Value {
            $(
                natural_binary_op!($name, $trait, $op);
                generic!($name, $generic_name);
            )+
        }
    }
}

macro_rules! boolean_binary_op {
    ($name: ident, $op: tt) => {
        pub fn $name(self, other: Self) -> Result<Self>
        {
            let (left, right): (bool, bool) = (self.convert()?, other.convert()?);
            let result = left $op right;
            Ok(result.into())
        }
    };
}
macro_rules! boolean_binary_ops {
    ($(($name: ident, $op: tt)),+) => {
        impl Value {
            $(boolean_binary_op!($name, $op);)+
        }
    }
}

macro_rules! comparative_binary_op {
    ($name: ident, $op: tt) => {
        pub fn $name(self, other: Self) -> Result<Self> {
            Ok(Value::Bool(self $op other))
        }
    };
}
macro_rules! comparative_binary_ops {
    ($(($name: ident, $op: tt)),+) => {
        impl Value {
            $(comparative_binary_op!($name, $op);)+
        }
    }
}

macro_rules! generic {
    ($name: ident, $generic_name: ident) => {
        pub fn $generic_name(self, other: Self) -> Result<Self> {
            if i64::can_be_from(&self) && i64::can_be_from(&other) {
                self.$name::<i64>(other)
            } else if f64::can_be_from(&self) && f64::can_be_from(&other) {
                self.$name::<f64>(other)
            } else {
                Err(ValueError::OnlySupportsNumeric(
                    if !f64::can_be_from(&self) {
                        self
                    } else {
                        other
                    },
                    stringify!($name),
                )
                .into())
            }
        }
    };
}

natural_binary_ops!(
    (add, Add, +, generic_add),
    (subtract, Sub, -, generic_subtract),
    (multiply, Mul, *, generic_multiply),
    (divide, Div, /, generic_divide),
    (modulus, Rem, %, generic_modulus)
);

boolean_binary_ops!(
    (and, &),
    (or, |),
    (xor, ^)
);

comparative_binary_ops!(
    (eq, ==),
    (not_eq, !=),
    (gt, >),
    (gt_eq, >=),
    (lt, <),
    (lt_eq, <=)
);

impl Value {
    pub fn string_concat(self, other: Self) -> Result<Self> {
        Ok(self)
    }
}
