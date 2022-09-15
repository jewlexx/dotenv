#[macro_use]
extern crate dotenv_codegen;

mod private_module {
    dotenv_module!(filename = ".env", visibility = "pub");
}

dotenv_module!();

#[test]
fn test_vars_at_build() {
    dotenv_build!();

    assert_eq!(std::env::var("CODEGEN_TEST_VAR1"), Ok("hello!".to_owned()));
    assert_eq!(
        std::env::var("CODEGEN_TEST_VAR2"),
        Ok("'quotes within quotes'".to_owned())
    );
    assert_eq!(std::env::var("CODEGEN_TEST_VAR3"), Ok("69".to_owned()));
}

#[test]
fn test_private_vars() {
    // assert_eq!(private_module::, "hello!");
    // assert_eq!(private_module::CODEGEN_TEST_VAR2, "'quotes within quotes'");
    // assert_eq!(private_module::CODEGEN_TEST_VAR3, "69");
}

#[test]
fn test_custom_path() {
    dotenv_build!(filename = ".env");

    assert_eq!(std::env::var("CODEGEN_TEST_VAR1"), Ok("hello!".to_owned()));
    assert_eq!(
        std::env::var("CODEGEN_TEST_VAR2"),
        Ok("'quotes within quotes'".to_owned())
    );
    assert_eq!(std::env::var("CODEGEN_TEST_VAR3"), Ok("69".to_owned()));
}

#[test]
fn test_vars_in_module() {
    use dotenv_vars::*;
    assert_eq!(CODEGEN_TEST_VAR1, "hello!");
    assert_eq!(CODEGEN_TEST_VAR2, "'quotes within quotes'");
    assert_eq!(CODEGEN_TEST_VAR3, "69");
}
