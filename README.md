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

### Benchmarks results:

P3 tests are done on a small ascii file.
P6 were done on a giant 3GB binary file.

```
test p3::bench::bench_create_file     ... bench:      32,818 ns/iter (+/- 4,981)
test p3::bench::bench_greyscale_image ... bench:     546,130 ns/iter (+/- 40,703)
test p3::bench::bench_invert_image    ... bench:     544,812 ns/iter (+/- 50,055)
test p3::bench::bench_output_file     ... bench:     553,708 ns/iter (+/- 46,956)
test p6::bench::bench_greyscale_image ... bench: 3,753,314,670 ns/iter (+/- 965,171,269)
test p6::bench::bench_invert_image    ... bench: 3,703,109,740 ns/iter (+/- 853,761,956)
```

### Members of the team are :
N'ZAOU Renaud
SOUISSI Mohamed
DURAND Stanislas
OYE Ken-Williams
