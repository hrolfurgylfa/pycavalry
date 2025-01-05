use pycavalry::RevealTypeDiag;

mod common;
use common::*;

#[test]
fn test_lambda_return_no_args() {
    run_with_errors(
        "test_lambda_return_no_args.py",
        "reveal_type((lambda x, y, z: \"asdf\")(1, 2, 3))",
        vec![RevealTypeDiag::new(ann("Literal[\"asdf\"]"), r(12..45)).into()],
    );
}
