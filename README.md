# Tinyserve

Tinyserve is a simple multi-threaded web server written in Rust. It serves the specified web root at the specified address.

```
USAGE:
    main [FLAGS] [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -v               Use -v for verbose output.
    -V, --version    Prints version information

OPTIONS:
    -a, --address <127.0.0.1>              Sets the IPv4 address to bind to. Default is 127.0.0.1 or localhost.
    -p, --port <8000>                      Sets the port on the specified address. Default is 8000.
    -r, --webroot <directory_with_html>    Sets the web root for the server. index.html is loaded from this directory.
                                           Default is ~/.config/tinyserve.
```

## How to use

Your web root directory must contain at the very least, a file called *404.html*. Tinyserve will crash if this is not present. If there is no *index.html* to serve for a ```GET /``` request, Tinyserve will fall back to *404.html*.

Copyright (c) 2019 Tanner Babcock.
