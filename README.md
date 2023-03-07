# birthday_display
A rust tool to read persons, their birthday and a personalised image from a csv file and display the data for every person whose birthay is today.

# Running
The executable takes the path of a csv file with the data as a madatory argument. The file should be in the format `lastname,firstname,dd.mm.YYYY,gender,[image url]`.
The gender must be a single character.
The image url is optional and must use either http or https.

# Installation
Install cargo using your platform's installation method.
Complie the project with
``` sh
cargo build --release
```
to use native gpu driver (Vulkan, Metal DX12).

To complie with support for OpenGL 2.1 or OpenGL ES 2.0 run
``` sh
cargo build --release --features glow
```
For more information see the [iced-rs](https://github.com/iced-rs/iced#graphicsadapternotfound) documentation.

The compiled executable will be in `target/release/birthday_display.exe`.
