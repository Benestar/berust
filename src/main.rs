extern crate befunge;

use befunge::interpreter::Interpreter;
use befunge::playfield::Playfield;

fn main() {
    let playfield = Playfield::new(
        r#">              v
v  ,,,,,"Hello"<
>48*,          v
v,,,,,,"World!"<
>25*,@
"#,
    );

    let mut interpreter = Interpreter::new(playfield);

    interpreter.run();
}
