# Rust + Tokio + Axum on ESP32-S3

This is an example application that demonstrates how to use Rust, Tokio and Axum to build a web server on the ESP32-S3 microcontroller.

## Quick Start

You'll need tools:
- `espup` - a tool to install the ESP32 toolchain and other tools
- `cargo-espflash` - a tool to flash the ESP32

```
cargo install espup cargo-espflash
espup install
```

### Build and run

1. Set the WiFi credentials as environment variables:
    - WIFI_SSID
    - WIFI_PASS
2. This command builds the project and flashes it to the ESP32-S3:
    ```sh
    cargo espflash flash --release --target xtensa-esp32s3-espidf --monitor 
    ```

### Test it

Once the ESP32-S3 is connected to your network, it displays the IP address on the serial console. The server runs on port 80.

```sh
$ curl http://192.168.0.24/state | jq
{
  "counter": 940,
  "free_heap": 216452,
  "ip_address": "192.168.0.24",
  "mac_address": "DC:DA:0C:2A:26:C8",
  "message": "Hello from ESP32!"
}
```

## But is it any good?

I mean, this is strapping rockets to a bicycle.

It works if you don't overload it. The ESP32-S3 is a microcontroller with limited resources. The web server can only handle a few requests at a time before it runs out of memory. 

### Ballpark numbers
- 5 concurrent requests
- 50 requests per second

If I go over ~5 concurrent requests, IDF runs out of memory and crashes. 

Axum wasn't designed for microcontrollers to put it mildly. It's actually a miracle it works at all. I did this for science. We do what we must because we can.


### Alternatives

Besides Axum, Warp also works. It uses somewhat less memory, so you can maybe throw a few more requests at it.

At the time of writing this, Rocket and Actix-web don't work since they require Tokio's `rt-multi-thread` feature which doesn't compile on EPS32 yet.


## Acknowledgements

This project is heavily inspired by Sami J. Mäkinen's [esp32temp](https://github.com/sjm42/esp32temp). Thanks for figuring out how to make this work!


## License

Whatever.