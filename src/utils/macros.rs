macro_rules! warning {
    ($expr: expr) => {
        println!("GlueSQL Warning: {}", $expr);
    };
}
pub(crate) use warning;

macro_rules! try_block {
    ($storage: expr, $block: block) => {{
        match (|| async { $block })().await {
            Err(e) => {
                return Err(($storage, e));
            }
            Ok(v) => v,
        }
    }};
}
pub(crate) use try_block;

macro_rules! try_option {
    ($try: expr) => {
        match $try {
            Ok(success) => success,
            Err(error) => return Some(Err(error)),
        }
    };
}
pub(crate) use try_option;

macro_rules! try_into {
    ($self: expr, $expr: expr) => {
        match $expr {
            Err(e) => {
                return Err(($self, e));
            }
            Ok(v) => v,
        }
    };
}
pub(crate) use try_into;
