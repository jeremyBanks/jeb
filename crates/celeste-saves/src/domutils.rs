use minidom::Element;
use tracing_unwrap::OptionExt;

pub trait DomUtils {
    fn expect_parse_attr<T: std::str::FromStr>(&self, name: &str) -> T;
    fn expect_child(&self, name: &str) -> &Self;
    fn expect_parse_child<T: std::str::FromStr>(&self, name: &str) -> T;
}

impl DomUtils for Element {
    fn expect_parse_attr<T: std::str::FromStr>(&self, name: &str) -> T {
        self.attr(name)
            .expect(name)
            .parse::<T>()
            .ok()
            .unwrap_or_log()
    }

    fn expect_child(&self, name: &str) -> &Self {
        let matches: Vec<&Self> = self.children().filter(|el| el.name() == name).collect();
        assert!(matches.len() == 1);
        matches[0]
    }

    fn expect_parse_child<T: std::str::FromStr>(&self, name: &str) -> T {
        let matches: Vec<&Self> = self.children().filter(|el| el.name() == name).collect();
        assert!(matches.len() == 1);
        matches[0].text().parse::<T>().ok().unwrap_or_log()
    }
}
