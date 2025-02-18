use indoc::indoc;

mod common;
use common::*;

#[test]
fn test_lambda_return_no_args() {
    run_with_errors(
        "test_lambda_return_no_args.py",
        indoc! {r#"
            reveal_type((lambda x, y, z: "asdf")(1, 2, 3))  # Debug: RevealTypeDiag(Literal["asdf"], )
        "#},
    );
}
