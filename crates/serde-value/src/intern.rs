use std::{
    collections::HashSet,
    sync::{LazyLock, Mutex},
};

static INTERNED_STRINGS: LazyLock<Mutex<HashSet<&'static str>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));

pub(crate) fn intern_string_owned(s: String) -> &'static str {
    let mut set = INTERNED_STRINGS.lock().unwrap();
    if let Some(&existing) = set.get(s.as_str()) {
        return existing;
    }
    let leaked: &'static str = Box::leak(s.into_boxed_str());
    set.insert(leaked);
    leaked
}
