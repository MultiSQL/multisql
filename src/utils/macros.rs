macro_rules! warning {
    ($expr: expr) => {
        println!("GlueSQL Warning: {}", $expr);
    };
}

pub(crate) use warning;
