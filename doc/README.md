# Documentation

Notes on how the GameBoy works. [Old README of resources](https://github.com/atskae/gameboy-emulator/tree/master/res).

## Links
* [The Ultimate Game Boy Talk](https://www.youtube.com/watch?v=HyzD8pNlpwI)
* [Opcode table](https://www.pastraiser.com/cpu/gameboy/gameboy_opcodes.html)
    * Pick column first, then row
* [Decoding Z80 opcodes (for the GameBoy)](https://gb-archive.github.io/salvage/decoding_gbz80_opcodes/Decoding%20Gamboy%20Z80%20Opcodes.html)

Refresher from [The Ultimate Game Boy Talk](https://www.youtube.com/watch?v=HyzD8pNlpwI):

## DMG-CPU
* CPU
* Interrupt Controller
* Timer
* Memory
* Boot ROM
* Peripherals
    * Joypad Input
    * Serial Data Transfer
        * Link cable
    * Sound Controller
    * Pixel Processing Unit (video controller)

### CPU
* Sharp LR35902
    * Intel 8080 (core architecture)
    * Zilog Z80 (a few features)
* [Registers](https://www.youtube.com/watch?v=HyzD8pNlpwI&t=656s)
  * Can combine registers get 16-bit register
  * Flag register
* [Instructions](https://www.youtube.com/watch?v=HyzD8pNlpwI&t=707s)
    * Stack
* [Interrupt model](https://www.youtube.com/watch?v=HyzD8pNlpwI&t=744s)
    * Jumps to fixed locations in RAM
    * Turning on GameBoy starts at location 0
* Memory mapped I/O instead of ports (?)
* [Zero page](https://www.youtube.com/watch?v=HyzD8pNlpwI&t=860s)
    * FF00-FFFF
    * Dedicated page to optimize some instructions
* [Prefixed instructions (CB)](https://www.youtube.com/watch?v=HyzD8pNlpwI&t=920s)
* [How to read opcode table](https://www.youtube.com/watch?v=HyzD8pNlpwI&t=935s)
    * Number of bytes in the instruction, number of clocks

### [Address Space](https://www.youtube.com/watch?v=HyzD8pNlpwI&t=1027s)
* 16-bit address space, 64KB
* ROM space (cartridge is loaded here)
    * 32 KB
* Video, ...

#### How Games are Loaded
* If > 32KB (ROM space)
    * Cartridge uses internal memory banks
    * Memory bank controller swaps out banks
* [Boot ROM(https://www.youtube.com/watch?v=HyzD8pNlpwI&t=1156s])
    * Nintendo logo and chime
    * Built into GameBoy
    * Initialize/setup, logo comparison
    * After, goes straight to game ROM

### [I/O RAM, HRAM](https://www.youtube.com/watch?v=HyzD8pNlpwI&t=1300s)
* Peripheral registers: interrupt controller, sound controller, Joypad input, ...

### [Joypad Input](https://www.youtube.com/watch?v=HyzD8pNlpwI&t=1329s)

### Serial Data Transfer

### [Timer](https://www.youtube.com/watch?v=HyzD8pNlpwI&t=1391s)

### [Interrupt Controller](https://www.youtube.com/watch?v=HyzD8pNlpwI&t=1417s)
* Peripheral events
* Addresses of specific peripherals

### [Sound Controller](https://www.youtube.com/watch?v=HyzD8pNlpwI&t=1449s)
* Many registers, 4 voices, 5 registers each
* Wave register, Pulse A/B
* Noise (random numbers to generate noise)
* Which speaker (L/R) to play sound

### [Pixel Processing Unit](https://www.youtube.com/watch?v=HyzD8pNlpwI&t=1758s)
* Screen size, 4 colors, pixel tiles, sprites
* Tile (16 bytes), how to encode into bytes
* How scrolling works: background map, screen becomes viewport
    * Infinite scrolling (Super Mario Land)
      * Draws columns as viewport moves
* Window layer, overlays on top of Background layer (eg, for scoreboards)
* Sprites layer (on top of Window)
    * Attributes, [Object Attribute Map](https://www.youtube.com/watch?v=HyzD8pNlpwI&t=2057s) entry per sprite
* OAM RAM
* Larger sprites

#### [VRAM Memory Map](https://www.youtube.com/watch?v=HyzD8pNlpwI&t=2347s)
* How sprites, BG tiles, etc, are mapped in to memory

#### [Vertical Timing](https://www.youtube.com/watch?v=HyzD8pNlpwI&t=2425s)
* How images are drawn in the screen
* How to split screen and have each section have their own scroll effect
    * Racecar game example
* Fun effects (SCX register)
    * Warp, curve a source image

#### [Horizontal Timing](https://www.youtube.com/watch?v=HyzD8pNlpwI&t=2683s)
* PPU timing in terms of number of clocks
* Timeline, OAM Search, Pixel Transfer, H-Blank
    * OAM search: find sprites that are visible on screen
        * OAM bug
* [VRAM access](https://www.youtube.com/watch?v=HyzD8pNlpwI&t=2816s)
    * CPU -> RAM, PPU -> Video RAM
* Some memory spaces can only be accessed at certains times in PPU timing

#### [Pixel Pipeline](https://www.youtube.com/watch?v=HyzD8pNlpwI&t=2957s)
* How GameBoy draws pixels
* Pixel FIFO
    * Sends pixels to LCD screen
    * Fetcher fills FIFO with background tiles
* H-Blank mode when pixels don't need to be drawn?
* [How sprites are drawn](https://www.youtube.com/watch?v=HyzD8pNlpwI&t=3176s)
    * Sprite comparator
    * Sprite fetch, overlay over background pixels
* [Pixel FIFO stores metadata of the pixels](https://www.youtube.com/watch?v=HyzD8pNlpwI&t=3283s)
    * Pixel mixing
    * Palette is applied when pixel leaves the FIFO

## Other
* GameBoy Game Development
* GameBoy Camera
