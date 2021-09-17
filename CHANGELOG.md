# Changelog

## v0.1 Changes from inside substrate tree

- DEFAULT_VALUE now an AtomicU16 rather than a Mutex. Set with `set_default()`
- try_from(u16), try_from(u8) => from(u16) and from(u8) as the conversions are infallable.
