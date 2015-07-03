#rust-firmata

Control your [firmata](https://github.com/firmata/protocol) powered device with rust!

Getting Started
---
```bash
$ git clone https://github.com/zankich/rust-firmata && cd rust-firmata
$ cargo build
$ cargo run --example blink
```
Usage
---
Add `firmata` to  your `Cargo.toml`
```
[dependencies]                                                                                       
firmata = "0.1.0"
```

Supported Functionality
---
- Pwm 
- Servo
- Analog
- Digital

## License
Copyright (c) 2015 Adrian  Zankich

Distributed under the [MIT License](LICENSE).
