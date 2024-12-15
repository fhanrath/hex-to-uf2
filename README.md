# hex_to_uf2

This crate converts .hex files to .uf2 files according to the following standard:
https://github.com/microsoft/uf2

## usage

```Rust
use hex_to_uf2::hex_to_uf2;

fn main() {
    let static_string = ":020000041000EA
:1000000000B5324B212058609868022188439860DF
:10001000D860186158612E4B002199600221596106
:100020000121F02299502B49196001219960352056
:1000300000F044F80222904214D00621196600F024
:1000400034F8196E01211966002018661A6600F04E
:100050002CF8196E196E196E052000F02FF8012189
:100060000842F9D1002199601B49196000215960AB";

    let uf2_bytes = hex_to_uf2(static_string.lines(), None);
    println!("{uf2_bytes:X?}");
}
```

```Rust
use hex_to_uf2::hex_to_uf2_file;

fn main() {
    hex_to_uf2_file(
        Path::new("./test/rmk-central.hex"),
        Path::new("./test/rmk-central.uf2"),
    )
    .unwrap();
}
```

## Attributions
The algorithm for conversion is taken out of the python code: https://github.com/HaoboGu/rmk/blob/main/scripts/uf2conv.py (MIT)
I hereby grant the RMK project to use code from this library under the terms of MIT. Everyone else please stick to AGPL :)
