//! IBM 701 Speedcoding System (SpeedCo I) library crate.

pub mod arithmetic;
pub mod control;
pub mod core_types;
pub mod diagnostics;
pub mod io;
pub mod parser;

#[cfg(test)]
mod tests {
    use mpc::{MpcResult, mpc_parse, mpc_string};

    #[test]
    fn mpc_library_inclusion() {
        let parser = mpc_string("speedcoding");
        match mpc_parse("test", "speedcoding", &parser) {
            MpcResult::Ok(val) => {
                assert_eq!(
                    val.downcast_ref::<String>().map(String::as_str),
                    Some("speedcoding")
                );
            }
            MpcResult::Err(err) => panic!("unexpected parse error: {err}"),
        }
    }
}
