pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;
    use mpc::{MpcResult, mpc_parse, mpc_string};

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }

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
