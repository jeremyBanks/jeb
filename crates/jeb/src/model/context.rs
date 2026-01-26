use {
    ignorable::{
        Hash,
        Ord,
        PartialEq,
        PartialOrd,
    },
    ordermap::OrderMap,
    std::{
        collections::{
            BTreeMap,
            BTreeSet,
        },
        sync::{
            Arc,
            LazyLock,
        },
    },
};
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
    aliases: OrderMap<String, String>,
}
#[derive(Clone, Hash, Ord, PartialEq, PartialOrd)]
struct Command {
    name: &'static str,
    #[ignored(
        PartialEq, Hash, Ord, PartialOrd
    )]
    implementation: Arc<dyn CommandImpl>,
}
impl Eq for Command {}
trait CommandImpl {
    #[allow(unused_variables)]
    fn spawn(
        &self,
        context: &mut Context,
        named: &OrderMap<String, String>,
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
#[derive(Clone)]
struct Call {
    command: Arc<Command>,
    named: OrderMap<String, String>,
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
