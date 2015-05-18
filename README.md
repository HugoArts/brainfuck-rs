Brainfuck-rs
============

brainfuck-rs is a brainfuck interpreter implemented in [Rust](http://rust-lang.org).

# Usage

The package is split in a small static library and an executable which provides a simple interface. Using the binary is straightforward:

```Shell
$ brainfsk -h
Usage: brainfsk [options] [--] <filename>
       brainfsk (-h | --help)

Options:
    -d, --dump  Instead of executing, dump string representation of ops to stdout
    -h, --help  Show this message
```
