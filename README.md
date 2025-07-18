Apparently, these are all the things you need to install to make it work on Fedora :
```bash
libmpc-devel mesa-libGL-devel mesa-libGLU-devel  libX11-devel  libICE-devel  libSM-devel
libxkbcommon-devel  libXaw-devel  libxcb-devel  libXpm-devel  libXt-devel libudev-devel
libxkbfile-devel  libxcb-cursor-devel  libxcb-errors-devel  libxcb-ewmh-devel
libxcb-icccm-devel  libxcb-image-devel  libxcb-keysyms-devel libXrandr-devel
libXcursor-devel
glibc-devel libgcc libstdc++-devel gmp-devel mpfr-devel
SFML SFML-devel
```
```
# Group install this with dnf4:
"Development Tools"
```
Some of them don't even exist, that's ok
