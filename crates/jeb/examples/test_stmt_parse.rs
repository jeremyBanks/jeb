macro_rules! test_stmt {
    // Base case
    ([] ) => {};

    // Hit a semicolon - emit accumulated tokens with the semicolon
    ([$($acc:tt)*] ; $($rest:tt)*) => {
        println!("emitting: {}", stringify!($($acc)*));
        $($acc)*;
        test_stmt!([] $($rest)*);
    };

    // Accumulate tokens
    ([$($acc:tt)*] $next:tt $($rest:tt)*) => {
        test_stmt!([$($acc)* $next] $($rest)*);
    };

    // Entry point
    ($($input:tt)*) => {
        test_stmt!([] $($input)*);
    };
}

fn main() {
    test_stmt!(
        let x = 1;
        foo();
        returns_string();
    );
    println!("x = {}", x);
}

fn foo() { println!("foo"); }
fn returns_string() -> String { String::from("hi") }
