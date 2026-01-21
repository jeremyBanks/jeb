use std::fmt::Debug;

macro_rules! alias {
    ($vis:vis impl $name:ident: $($rest:tt)+) => {
        $vis trait $name: $($rest)+ {}
        impl<T: $($rest)+> $name for T {}
    };
    ($vis:vis dyn $name:ident: $($rest:tt)+) => {
        alias! { $vis impl $name: $($rest)+ }
        const _: Option<Box<dyn $name>> = None;
    };
}

alias! { pub impl EqPartialEq: Debug + CloneDebug }
alias! { pub dyn CloneDebug: Debug }

fn main() {
    // println!("{}", CloneDebug::is_some());
}
