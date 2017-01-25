**Disclaimer: This is a draft version**

# rori_server

![](https://travis-ci.org/AmarOk1412/rori_server.svg?branch=master)

This is the central point of _[RORI](https://github.com/AmarOk1412/rori/)_. This application get data from entry points, call modules to process this data and send data to endpoints to execute commands. To understand how it works, you can read the [wiki](https://github.com/AmarOk1412/rori/wiki).

# Installation

This application requires _Rust language_ (please read this [page](https://www.rust-lang.org/en-US/install.html)) and _openssl_. To build the software, you just have to launch `cargo build` in this repository. You will need _Python 3_ to run modules. By the way, you will need dependencies for the modules you want to execute.

# Configuration

## From rori_www

This is not implemented yet, but you can try to connect [rori_www](https://github.com/AmarOk1412/RORI_www).

## From config_server.json

You can configure your server and the API from _config_server.json_:

```
"ip":"0.0.0.0",
"port":"1412",
"api_ip":"0.0.0.0",
"api_port":"3000",
```

## Connect entry and endpoints

Please read the [wiki](https://github.com/AmarOk1412/rori/wiki) or the [README of rori_desktop_client](https://github.com/AmarOk1412/rori_desktop_endpoint) for example.

## Tls configuration

All connections need to be secured. So you need to generate a private key and a certificate. On linux, you can run this following command: `openssl req -x509 -newkey rsa:4096 -keyout key.pem -out cert.pem`. It will create a certificate (_cert.pem_) and a private key (_key.pem_). Now, you can add these files to _config_server.json_.

## Final

Now, you _config_server.json_ looks like this:

```json
{
 "ip":"0.0.0.0",
 "port":"1412",
 "api_ip":"127.0.0.1",
 "api_port":"3000",
 "cert":"key/cert.pem",
 "key":"key/key.pem",
 "secret":"secret",
 "authorize": [
   {
     "name":"entry",
     "secret":"2BB80D537B1DA3E38BD30361AA855686BDE0EACD7162FEF6A25FE97BF527A25B"
   },
   {
     "name":"endpoint",
     "secret":"2BB80D537B1DA3E38BD30361AA855686BDE0EACD7162FEF6A25FE97BF527A25B"
   }
 ]
}
```

# Understanding the code

To understand the code, I will describe the process of a _RORIData_ through this application. A _RORIData_ is received by `core::Server::handle_client`. This function will make the distinction between a _RORIData_ for the server or for an endpoint. For example, if the datatype is _register_, it's a _RORIData_ for the server. This data is processed by the `ENDPOINTMANAGER` which manage endpoints. In the second case, the data is processed by a `ModuleManager` which calls modules for this datatype. To understand how modules works, you can read this [page](https://github.com/AmarOk1412/RORI/wiki/Write-modules). Then, when a module want to send data to a particular endpoint, it can use the API to get endpoint and send data. The code of these API is in `core::mod.rs` (TODO, the class will move in `core::API`).

# Execution

A binary is present in the _target/_ directory after a `cargo build` or you can execute `cargo run` in your shell.

# License

```
DO WHAT THE FUCK YOU WANT TO PUBLIC LICENSE
        Version 2, December 2004

Copyright (C) 2016 Sébastien (AmarOk) Blin <https://enconn.fr>

Everyone is permitted to copy and distribute verbatim or modified
copies of this license document, and changing it is allowed as long
as the name is changed.

DO WHAT THE FUCK YOU WANT TO PUBLIC LICENSE
TERMS AND CONDITIONS FOR COPYING, DISTRIBUTION AND MODIFICATION

0\. You just DO WHAT THE FUCK YOU WANT TO.
```

# Contribute

Please, feel free to contribute to this project in submitting patches, corrections, opening issues, etc.
