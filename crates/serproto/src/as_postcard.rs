//! Serializer wrapping a value so that it's encoded by Postcard, and then that
//! is itself encoded by the enclosing serializer.

struct AsPostcard;
struct AsJson;
