# tiny-2ltt

A toy implementation of two-level type theory: a calculus with `<e>` (quote / code) and `~e` (splice) where the type system separates a compile-time meta language from a runtime object language.

The base type theory is unsurprising (Nat, Bool, arrow, `if`, `let`, ... the MLy usual goodness). But the interesting part is the `Code A` type constructor and the two staging operators that mediate between the two levels:

- if `e : A` at stage 1 then `<e> : Code A` at stage 0
- if `e : Code A` at stage 0 then `~e : A` at stage 1
- `~<e>` reduces to `e`

A stage-0 term is run. A stage-1 term is quoted, evaluated only as far as the splices it contains require. The output of `eval` is a residual stage-1 program with all the compile-time work done.

```bash
cargo build
cargo test
cargo run -- examples/power.2ltt
```

```text
let pow : Nat -> Code Nat -> Code Nat
<\y_0. y_0 * (y_0 * (y_0 * (y_0 * (y_0 * 1))))> : Code (Nat -> Nat)
```

The classic example: a stage-0 `pow n x` whose recursion is fully specialised away when `n` is a literal, leaving an unrolled stage-1 term.

```
let pow : Nat -> Code Nat -> Code Nat =
  \n. \x. if n == 0 then <1> else <~x * ~(pow (n - 1) x)>;

eval <\(y : Nat). ~(pow 5 <y>)>;
```

A more compelling example is square-and-multiply with shared squarings, where the residual program is logarithmic in the exponent and its `let` structure mirrors the binary representation of `n`. See [`examples/fastpow.2ltt`](examples/fastpow.2ltt).

```
let sq : Code Nat -> Code Nat = \c. <let y : Nat = ~c in y * y>;

let fastpow : Nat -> Code Nat -> Code Nat = \n. \x.
  if n == 0 then <1>
  else if mod2 n == 0 then sq (fastpow (div2 n) x)
  else <~x * ~(sq (fastpow (div2 n) x))>;
```

```text
<\z_0. z_0 * (let y_4 = let y_3 = z_0 * (let y_2 = z_0 * (let y_1 = 1 in y_1 * y_1) in y_2 * y_2) in y_3 * y_3 in y_4 * y_4)>
```

More examples under [`examples/`](examples/) and [`tests/cases/`](tests/cases/).

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
