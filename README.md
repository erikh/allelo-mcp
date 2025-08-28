Open source MCP proxy for the Alello Social Media platform. This enables phones to host a mcp and push out to an intermediate proxy (this software) which will multiplex the responses from the LLM back through SSE. This enables a variety of network usage patterns that are more reliable on cell phone networks.

Usage notes:

**DO NOT RUN cargo test**. Run `make test`. You will need `docker` installed.
