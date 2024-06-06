### This repo contains an example for https://github.com/rust-lang/flate2-rs/issues/276

Requirements -
- Have rust installed
- Have python installed
- `pip install rangehttpserver`

Running -

- In a terminal tab, execute `python -m RangeHTTPServer`
- In another, execute `cargo run`

Output -

```
                        Trying to decode the partial gzip from the initial request.
                        This will fail because we have no idea what the actual length of the decoded content is
                        and read_to_end simply tries going past the end and fails because the GzDecoder obviously
                        thinks that this is a corrupt file.

Error: corrupt deflate stream
```
