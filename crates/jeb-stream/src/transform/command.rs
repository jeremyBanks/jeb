// TODO: this is going to pipe data into an external command's stdin,
// and take the output (lines?) of stdout and stderr as yielded ok and err.
// Maximum length of a "line" is 64KiB (this is our general upper limit buffer
// choice for lots of things, it's a nice number we like it).
//
// we might need some kind of new split option that splits on individual
// items instead of on the joined stream. like a split_items.
