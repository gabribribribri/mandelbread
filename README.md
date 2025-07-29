# Mandelbread - Bad Mandelbrot Renderer 
The name doesn't mean anything, it just sounds funny.
I thought it would be cool to build my own renderer fractal renderer, so I did.
I built it using Rust, the egui graphic libary for the UI and SFML for the rendering.
Not the best decision.

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
- Zoom with scroll wheel 

## TODOs
If I do all of these the work will definitly be finished and my job here will be forever done (I will not)

 - [ ] Fix bug where Workers crash when switching backend while computing
 - [ ] Fix bug with GPU resolution changing
 - [ ] Fix reload time displaying for GPU
 - [ ] Implement changing the gradient colors
 - [ ] Implement diffusion theory rendering *(looks hard)*

## Installation 
### Windows
1. Switch to Linux

### MacOS
1. Reflect on your life choices
2. Switch to Linux

### Linux
#### Fedora
I only tried **Fedora 42** so good luck on your distro
Apparently, these are all the things you need to install to make it work on Fedora :
```bash
libmpc-devel mesa-libGL-devel mesa-libGLU-devel  libX11-devel  libICE-devel  libSM-devel libxkbcommon-devel  libXaw-devel  libxcb-devel  libXpm-devel  libXt-devel libudev-devel libxkbfile-devel  libxcb-cursor-devel  libxcb-errors-devel  libxcb-ewmh-devel libxcb-icccm-devel  libxcb-image-devel  libxcb-keysyms-devel libXrandr-devel libXcursor-devel libc-devel libgcc libstdc++-devel gmp-devel mpfr-devel SFML SFML-devel
```
Group install this :
```bash
"Development Tools"
```
Some of them don't even exist, that's ok

