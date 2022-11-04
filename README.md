# `memset-nt`

An implementation of a non-temporal `memset` in Rust.

May be useful in situations where memory is being bulk-set to zero, for example,
but the stores to memory shouldn't pollute data caches otherwise.

Currently only has an accelerated implementation for x86\_64, otherwise falls
back to `memset` on all other architectures.
