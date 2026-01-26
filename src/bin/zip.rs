use std::collections::BTreeMap;

use fiction::zip::zip;

fn main() -> Result<(), fiction::panic> {
    // std::fs::write(
    //     "target/test.zip",
    //     zip(BTreeMap::from_iter([
    //         ("1.txt", "alfa"),
    //         ("2.txt", "bravo"),
    //         ("empty.txt", ""),
    //         ("meta/empty.txt", ""),
    //         ("mimetype", "text/plain"),
    //         ("meta/mimetype", "text/plain"),
    //         ("3.txt", "charlie"),
    //         ("4.txt", "delta"),
    //         ("4.txt", "quad"),
    //     ])
    //     .iter()),
    // )?;

    Ok(())
}
