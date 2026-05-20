use std::fs;

use tiny_2ltt::run_program;

#[test]
fn cases() {
    insta::glob!("cases/*.2ltt", |path| {
        let src = fs::read_to_string(path).unwrap();
        let out = match run_program(&src) {
            Ok(s) => s,
            Err(e) => format!("error: {e}\n"),
        };
        insta::assert_snapshot!(out);
    });
}
