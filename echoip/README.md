# Echo IP
A tool that echos visitor's ip. The tool is built for learning rust
tokio.

Start the server by `echoip [port]` (default port is 7878).

A simple benchmark `ab -c 200 -n 100000 -k http://example.com:7878/`
turns out the QPS is about 200k with keep-alive on; however, due to
implement problem it doesn't support legacy one-request-per-connection
behaviour, which apache benchmark is used as default. 