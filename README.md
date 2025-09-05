# Mandelbread - Bad Mandelbrot Renderer 
The name doesn't mean anything, it just sounds funny.
I thought it would be cool to build my own fractal renderer, so I did.
I built it using Rust, the egui graphic libary for the UI. And SFML for the rendering.
Not the best decisions.

## Features
- Render the Mandelbrot set with RGB gradient
- Multi-threading
- Rendering with double precision floating point
- Rendering with GNU's MPFR libraries **(SLOW AF)**
- GPU accelerated rendering using GLSL Shaders.
- Changing the number of iterations of each pixel
- Automatic iteration change
- Changing the distance of convergence
- Changing the resolution
- Click to move
- Scroll Wheel to zoom

## TODOs
If I do all of these the work will definitly be finished and my job here will be forever done (I will not)

 - [x] Fix bug where Workers crash when switching backend while computing
 - [x] Fix bug with GPU resolution changing
 - [ ] Fix reload time displaying for GPU (I failed I have no idea how to do that)
 - [ ] Implement smooth zooming
 - [ ] Implement changing the gradient colors
 - [ ] Implement diffusion theory rendering *(looks hard)*

## Building 
### Windows
After git cloning the repository :
1. Install [Rust](https://www.rust-lang.org/)
2. Install [MSYS2](https://www.msys2.org/)
3. Install [cmake](https://cmake.org/download/)
4. Add MinGW-w64 to your PATH (C:\msys64\mingw64\bin if you did not change the install folder)
5. Open MSYS2 and enter these commands
```bash
pacman -Syu
pacman -S mingw-w64-x86_64-gcc mingw-w64-x86_64-make
```
6. Run `rustup default stable-x86_64-pc-windows-gnu`
7. Run `cargo run --release`

### MacOS
1. Reflect on your life choices
2. Switch to Linux

### Linux
I only tried **Fedora 42** so good luck on your distro.
#### Fedora
1. Git clone the repository
2. Apparently, these are all the things you need to install (with dnf) to make it work on Fedora :
```bash
libmpc-devel mesa-libGL-devel mesa-libGLU-devel  libX11-devel  libICE-devel  libSM-devel libxkbcommon-devel  libXaw-devel  libxcb-devel  libXpm-devel  libXt-devel libudev-devel libxkbfile-devel libXrandr-devel libXcursor-devel libgcc libstdc++-devel gmp-devel mpfr-devel SFML SFML-devel
```
Group install this :
```bash
"Development Tools"
```
Some of them don't even exist, that's ok
3. `cargo run`

