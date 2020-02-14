# PPM library:

This Rust library allows you to process PPM files according to this specification:
http://netpbm.sourceforge.net/doc/ppm.html

It has bufferized and threaded support for reading and writing binary files.

It can invert colors and apply greyscale on any ppm image, binary or ASCII (P6/P3).

Check the source code for more documentation !

WARNING: not thread safe.

### Run tests:

```
cargo test -- --test-threads=1
```

Running tests with more threads will fail.


### Run benchmarks:

```
cargo bench
```

### Members of the team are :
N'ZAOU Renaud
SOUISSI Mohamed
DURAND Stanislas
OYE Ken-Williams
