# rust canvascycler / pixedwall

hi! i totally adore effectgames' [canvascycle demo](https://www.effectgames.com/demos/canvascycle/?sound=0), which shows off a technique use by really old computer videogames to create animated artworks. the images are super gorgeous (with beautiful animations!):

<img src="https://cdn.hackclub.com/01a15e73-6011-73aa-b62f-b2375f19f73f/Screenshot_20261021_143241.png" width=250>

these artworks use a custom fileformat and this project originally started as a rust implementation of that renderer and parser, but then i also added support for using these animated images as your desktop wallpaper (Windows only):

https://github.com/user-attachments/assets/e9be9984-0c72-44d7-8d40-3d1311b4c4c4

^todo: update this demo

so the final state of the project is primarily a animated wallpaper engine for Windows, which uses these pixel art artworks. 

the project is however split in two binaries, the `pixedwall` binary is the actual wallpaper engine, while `renderer` just renders an 


## about dependencies

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