use tiny_2ltt::run_program;

const SRC: &str = r"
let pow : Nat -> Code Nat -> Code Nat =
  \n. \x. if n == 0 then <1> else <~x * ~(pow (n - 1) x)>;

eval <\(y : Nat). ~(pow 4 <y>)>;
";

fn main() {
    match run_program(SRC) {
        Ok(out) => print!("{out}"),
        Err(e) => eprintln!("error: {e}"),
    }
}
