#[macro_use]
extern crate dotenv_codegen;

#[test]
fn test_vars_at_build() {
    dotenv_build!();

    assert_eq!(std::env::var("CODEGEN_TEST_VAR1"), Ok("hello!".to_owned()));
    assert_eq!(
        std::env::var("CODEGEN_TEST_VAR2"),
        Ok("'quotes within quotes'".to_owned())
    );
}
