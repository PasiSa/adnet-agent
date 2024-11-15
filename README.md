# AdNet-Agent

This server software is used for assignments in Aalto University's **Advanced
Networking** course (ELEC-E7321). It waits for incoming TCP connections and a
special command string sent through the connection, that starts a specific
assignment protocol, as described in the task description. The server software
is implemented in Rust.

To install the server software, you'll need to do the following.

1. Install [Rust](https://www.rust-lang.org/learn/get-started) (if you haven't
   already). On MacOS or Linux, the following should work:
   `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`

2. Clone this repository

3. Change working directory to the cloned repository and build the Rust code:
   `cargo build`

4. Start the server: `cargo run`

You can add more debug information about the server software using `RUST_LOG`
environment variable, for example: `RUST_LOG=debug cargo run`

By default the server binds to any address (0.0.0.0), TCP port 12345. You can
change the address and port to bind to using the `-l` command line argument, for
example: `cargo run -- -l 127.0.0.1:9999`.
