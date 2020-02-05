# Hello World

Example of how to serve a Gotham application on a Unix domain socket.

## Running

From the `examples/hello_world_uds` directory:

```
Terminal 1:
$ cargo run
   Compiling gotham_examples_hello_world_uds v0.0.0 (/.../gotham/examples/hello_world_uds)
    Finished dev [unoptimized + debuginfo] target(s) in 2.73s
     Running `/.../gotham/target/debug/gotham_examples_hello_world_uds`
 Listening for requests on the Unix socket at ./server.ipc

Terminal 2:
$ curl -v --unix-socket ./server.ipc http://some-domain
*   Trying ./server.ipc:0...
* Connected to some-domain (server.ipc) port 80 (#0)
> GET / HTTP/1.1
> Host: some-domain
> User-Agent: curl/7.68.0
> Accept: */*
> 
* Mark bundle as not supporting multiuse
< HTTP/1.1 200 OK
< x-request-id: 10560879-7450-4c19-9a7a-c80ffef5d423
< content-type: text/plain
< content-length: 12
< date: Wed, 05 Feb 2020 18:17:34 GMT
< 
* Connection #0 to host some-domain left intact
Hello World!
```

## License

Licensed under your option of:

* [MIT License](../../LICENSE-MIT)
* [Apache License, Version 2.0](../../LICENSE-APACHE)

## Community

The following policies guide participation in our project and our community:

* [Code of conduct](../../CODE_OF_CONDUCT.md)
* [Contributing](../../CONTRIBUTING.md)
