# RustyNES

An NES emulator written in Rust!

https://user-images.githubusercontent.com/31429425/149857800-5445866b-7a37-433f-9389-f19356b2c5fd.mp4


## Supported Mappers
* Mapper 0
* Mapper 1
* Mapper 2
* Mapper 3
* Mapper 4

A list of all NES games and their associated mappers can be found <a href="http://tuxnes.sourceforge.net/nesmapper.txt">here</a>.

## Usage
The lastest versions of RustyNES for Ubuntu, macOS, and Windows can be found under Releases.

Games can be loaded in two different ways. 

### No Args
If you run the binary/executable with no args, the following splash screen will appear.

<img src="./src/resources/rustynes_splash_screen.png" width=500/>

Click File -> Open, select a .rom file, and start playing! 

### Load a Specific ROM
If you would like to load a .nes file at launch, you can use the following argument:
```
./rustynes --rom /path/to/rom.nes
```
or 
```
./rustynes -r /path/to/rom.nes
```

### Controls
<table>
  <tr>
    <th>Key</th>
    <th>Function</th>
  </tr>
  <tr>
    <td>Enter</td>
    <td>NES Start</td>
  </tr>
  <tr>
    <td>Right Shift</td>
    <td>NES Select</td>
  </tr>
  <tr>
    <td>Z</td>
    <td>NES A</td>
  </tr>
  <tr>
    <td>X</td>
    <td>NES B</td>
  </tr>
  <tr>
    <td>Arrow Keys</td>
    <td>NES D-pad</td>
  </tr>
  <tr>
    <td>Esc</td>
    <td>Exits to splash screen. If already on splash screen, exits the emulator.</td>
  </tr>
</table>

### Save States
If you would like to create a save state of a game, click File -> Save State on the toolbar and choose a directory for your state file. To load a state file, click File -> Load State on the toolbar.

## Building
In order to build RustyNES, you will need to install the SDL2 development libraries. The following operating systems are supported:

* <a href="https://github.com/Rust-SDL2/rust-sdl2#linux">Linux</a>
* <a href="https://github.com/Rust-SDL2/rust-sdl2#macos">macOS</a>
* <a href="https://github.com/Rust-SDL2/rust-sdl2#windows-msvc">Windows</a>
  
Once the libraries are installed, you should be able to build a binary by running the following command in the root of your RustyNES repository: 
```
cargo build --release
```
