<h1> Rocket Dstat </h1>
A simple, highly effecient DDOS statistic (dstat) server written in Rust using the Rocket web framework

<h3> Configuration </h3>
The server can be configured through the Rocket.toml config file. The following values should be changed for the server to operate correctly.

```
[default.dstat]
server_name = "Example Server 1"
control_server = "https://example.com"
shared_secret = "changemeplease"
```
The control_server should be the URL of your [main server](https://github.com/chanderlud/dstat_frontend)
<br>
The server_name is used to reference the dstat server on the main server
<br>
The shared_secret is used to authenticate the report requests that the dstat server makes to the main server
<br>
<br>
For configuring the web server, refer to [the Rocket configuration documentation](https://rocket.rs/v0.5-rc/guide/configuration/#overview)
