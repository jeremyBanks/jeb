use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{Arc, LazyLock},
};

use ignorable::{Hash, Ord, PartialEq, PartialOrd};
use indexmap::IndexMap;

// The configuration for an execution includes which commands and rules
// will be used.
#[derive(Clone, Default)]
pub struct Configuration {
    commands: BTreeSet<Command>,
    hooks: BTreeSet<Hook>,
    preludes: Vec<Call>,
}

impl Configuration {
    pub fn all() -> Configuration {
        Configuration {
            commands: Command::all().into_iter().collect(),
            hooks: Hook::all().into_iter().collect(),
            preludes: vec![],
        }
    }
}

pub struct Context {
    calls: Vec<Call>,
    aliases: IndexMap<String, String>,
}

// A command is... a named command!
#[derive(Clone, Hash, Ord, PartialEq, PartialOrd)]
struct Command {
    name: &'static str,
    #[ignored(PartialEq, Hash, Ord, PartialOrd)]
    implementation: Arc<dyn CommandImpl>,
}

impl Eq for Command {}

trait CommandImpl {
    #[allow(unused_variables)]
    fn spawn(
        &self,
        context: &mut Context,
        named: &IndexMap<String, String>,
        positional: &[String],
        body: Option<String>,
    ) -> Box<()> {
        unimplemented!()
    }
}

impl Command {
    pub fn all() -> impl IntoIterator<Item = Command> {
        vec![]
    }

    pub fn accepts_keywords(&self) -> bool {
        true
    }

    pub fn accepts_positional(&self) -> bool {
        true
    }

    pub fn accepts_body(&self) -> bool {
        true
    }
}

#[derive(Debug, Clone)]
struct Assign;
impl Assign {
    const COMMAND: LazyLock<Command> = LazyLock::new(|| Command {
        name: "assign",
        implementation: Arc::new(Self),
    });
}
impl CommandImpl for Assign {}

// A call is an invocation of a command, with optional named and positional
// arguments, and an optional body (yes that's a lot of pieces).
// The syntax is like
//
// ```
// call(named=1, named=2, positional, positional):body
// ```
//
// whitespace trimming here means ascii whitespace.
//
// so: everything before the first `(` or `:` or `=` is the command name, with
// whitespace trimmed. if there's a `(`, everything until `)` has whitespace
// trimmed, is split by commas, has whitespace trimmed again, then any which
// contain an `=` are split by that (whitespace trimmed again on both
// components) and put into the keyword argument map (an indexedmap)
// and everything that's not is put into a positional argument list.
// if there's no `)` then it's invalid sytax, and if `)` is directly followed by
// anything except the end of the string or a `:` (optionally with whitepsace in
// between) it's an error. it's valid for arguments to be the empty string, like
// `call(,,,,a=,b=,)` would have several, including one empty string positional
// arugment following the trailing comma (they have no special treatment. So if
// you have no `(` at all, then the positional vec is empty, but if you
// have `()` then that's single positional empty string argument, so they're not
// identical; if there's a `(...)` then there will be at least one positional or
// keyword arugment. If there's an `:` (which may have leading whitespace
// skipped), then everything after it is the body. We distinguish no body from
// an empty body. if there's a `=` before either `(` or `:`, then ignore the
// above rules. This is a special case which is an assignment, which is to say
// a call to the always-available special assign command, which interacts
// directly with the context to update some state. So name=value is equivalent
// to `assign(name=name):value` -- including in that whitespace is trimmed
// around `name`, but everything after the `=` is preserved literally, there's
// no implicit whitespace trimming after that.
#[derive(Clone)]
struct Call {
    command: Arc<Command>,
    named: IndexMap<String, String>,
    positional: Vec<String>,
    body: Option<String>,
}

#[derive(Clone, Hash, Ord, PartialEq, PartialOrd)]
struct Hook {
    name: &'static str,
}

impl Eq for Hook {}

impl Hook {
    pub fn all() -> impl IntoIterator<Item = Hook> {
        vec![]
    }

    pub fn apply(calls: &[Call]) -> Vec<Call> {
        calls.to_vec()
    }
}
