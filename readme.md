https://github.com/user-attachments/assets/e9be9984-0c72-44d7-8d40-3d1311b4c4c4

this project depends on a [fork i made of macroquad](https://github.com/ingobeans/macroquad/tree/exposed-windows),

* which in turn depends on a [fork i made of miniquad](https://github.com/ingobeans/miniquad/tree/exposed-windows)


## building

project is split in two binaries. `renderer` can be ran standalone but `pixedwall` depends on the renderer, so make sure to always build both binaries
when building pixedwall.

build `pixedwall`: 
```bash
cargo b && cargo r --bin pixedwall
```
---

while to build the renderer only, you can of course just do this:

build `renderer`: 
```bash
cargo r --bin renderer
```