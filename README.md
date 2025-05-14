to install:

- get rust
- `cargo install --git https://github.com/erikh/mcp-test`
    - or clone the repository and `cargo install --path .` inside of it
- configure claude (edit the path to `mcp-test`):

```json
{
    "mcpServers": {
        "halo": {
            "command": "/home/erikh/.cargo/bin/mcp-test",
            "args": ["stdio"]
        }
    }
}
```

- restart and enjoy. ensure the tool is recognized and running.
- you can ask for faults and about halo a bit
