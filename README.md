# RustyNES

An NES emulator written in Rust!

## Supported Mappers
* Mapper 0
* Mapper 1
* Mapper 2

## Usage
The lastest versions of RustyNES for Ubuntu, macOS, and Windows can be found under Releases.

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
    <td>I</td>
    <td>Displays a pop-up with the controls.</td>
  </tr>
  <tr>
    <td>Esc</td>
    <td>Exits to splash screen. If already on splash screen, exits the emulator.</td>
  </tr>
</table>


Games can be loaded in two different ways. 

### No Args
If you run the binary/executable with no args, the following splash screen will appear.

Simply drag-and-drop the .nes file into the window, and the game will load

### Load a Specific ROM
If you would like to load a .nes file at launch, you can use the following argument:
```
./rustynes --rom=/path/to/rom.nes
```
or 
```
./rustynes -r=/path/to/rom.nes
```

## Building on Linux and macOS
In order to build RustyNES, you will need SDL2 bindings for Rust. The base SDL2 bindings can be found <a href="https://github.com/Rust-SDL2/rust-sdl2">here.</a> You will also need the SDL2 GFX package.
Ubuntu/Mint:
```
sudo apt install libsdl2-gfx-dev
```
macOS:
```
brew install libsdl2-gfx-dev
```
Once the libraries are installed, you should be able to build a binary by navigating to the root directory and running
```
cargo build --release
```

### Building on Windows
On Windows, building RustyNES is a little more complex. First, you'll need to run the following command in Command Prompt or Powershell:
```
rustup target add i686-pc-windows-msvc
```
This will allow you to build a 32-bit executable of RustyNES. Unfortunately, only 32-bit builds are supported right now, as I only have a 32-bit dll of SDL2 GFX.

After following the instructions outlined here <a href="https://github.com/Rust-SDL2/rust-sdl2">here</a>, you will need to add the dll files in *** to your *** folder. This will give Rust access to the SDL2 libraries when building RustyNES.

Now you're ready to go! To create an executable, simply run:
```
PLACEHOLDER
```

## RustyNES In Action!
https://user-images.githubusercontent.com/31429425/136135867-dcd07b5e-8dd4-4862-9bb5-18d45d7a7789.mp4

