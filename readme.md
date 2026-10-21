# rust canvascycler / pixedwall

hi! i totally adore effectgames' [canvascycle demo](https://www.effectgames.com/demos/canvascycle/?sound=0), which shows off a technique use by really old computer videogames to create animated artworks. the images are super gorgeous (with beautiful animations!):

<img src="https://cdn.hackclub.com/01a15e73-6011-73aa-b62f-b2375f19f73f/Screenshot_20261021_143241.png" width=250>

these artworks use a custom fileformat and animation technique (read more about this [here](https://www.effectgames.com/effect/article-Old_School_Color_Cycling_with_HTML5.html)) and this project originally started as a rust implementation of that renderer and parser, but then i also added support for using these animated images as your desktop wallpaper (Windows only):


https://github.com/user-attachments/assets/70643187-5dea-4d3c-b3c9-159a41e90e53

<img src="https://user-cdn.hackclub-assets.com/01a15eca-560b-7cb4-a8de-91574a90f803/Screenshot%202026-10-21%20160701.png" width=250> <img src="https://user-cdn.hackclub-assets.com/01a15ecd-e93c-7218-9525-92efefaed3aa/Screenshot%202026-10-21%20161124.png" width=250>

so the final state of the project is primarily a animated wallpaper engine for Windows, which uses these pixel art artworks. 

the project is however split in two binaries, the `pixedwall` binary is the actual wallpaper engine, while `renderer` just renders an 

## features

* render animated pixel artworks from the .lbm.json format
* wallpaper engine used to display these artworks
  * wallpaper engine written from scratch with low level windows api calls
  * customize looks and active wallpaper from tray icon menu (also from scratch with windows api)
  * multi monitor support
  * 



## about dependencies

this project depends on a [fork i made of macroquad](https://github.com/ingobeans/macroquad/tree/exposed-windows),

* which in turn depends on a [fork i made of miniquad](https://github.com/ingobeans/miniquad/tree/exposed-windows)

## cross platform?

the wallpaper engine system (the `pixedwall` binary) is exclusive to windows since it directly uses low level windows api for the entire functionality.

the renderer (the `renderer` binary) is however cross platform, though on its own it isnt really useful for much.

## usage

download and extract the zip archive from the Releases tab.

to launch it, simply open the executable called `pixedwall.exe`.

you should now see your desktop wallpaper has been replaced by an animated pixel art artwork!

open your system tray to see the pixedwall icon. click it, and you can configure the active wallpaper! there's over 30 bundled wallpapers, and you can change the scale & positioning.

you can also exit and restart the program from here.

<img width="330" alt="image" src="https://github.com/user-attachments/assets/96fc0817-4c57-4684-ad03-e0fc19fdb306" />

<img width="269" height="204" alt="image" src="https://github.com/user-attachments/assets/a6ceb41c-006f-432f-9ec1-5e23eff8227d" />


## troubleshooting

if you have multiple monitors and the wallpaper is applied inconsistently, you should only have to restart it (from the tray icon). this sometimes happens on the first launch, but usually never again.

if the wallpaper engine fails to launch, try running it through a terminal and check the output. 


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
