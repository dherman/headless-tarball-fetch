Minimal example of how we can ask a remote server to determine the uncompressed size
of a tarball for Notion, in a better way than we've been doing it so far:

- Immediately creates a connection to fetch the full payload, cutting down on latency.
- Only requires a maximum of two HTTP connections instead of three.
- Does not use a HEAD request, which doesn't work with GitHub releases (and therefore doesn't work for Yarn).
