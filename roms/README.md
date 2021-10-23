# ROMs

The binaries of the GameBoy cartriges (the games!).

Can dump the binary in hexadecimal:
```
hexdump -C dmg_boot.bin
```
will output the following:
```
00000000  31 fe ff af 21 ff 9f 32  cb 7c 20 fb 21 26 ff 0e  |1...!..2.| .!&..|
00000010  11 3e 80 32 e2 0c 3e f3  e2 32 3e 77 77 3e fc e0  |.>.2..>..2>ww>..|
...
000000e0  21 04 01 11 a8 00 1a 13  be 20 fe 23 7d fe 34 20  |!........ .#}.4 |
000000f0  f5 06 19 78 86 23 05 20  fb 86 20 fe 3e 01 e0 50  |...x.#. .. .>..P|
```
The first column is the hexadecimal offset (for example, `00000010` = `16`), followed by 8 sets of 2-byte displays.


## List of ROMs
* `dmg_boot.bin` [GameBoy Bootstrap ROM](https://gbdev.gg8.se/wiki/articles/Gameboy_Bootstrap_ROM), the checksum and logo display when the GameBoy turns on.
    * Downloaded from [gbdev.gg8.se](https://gbdev.gg8.se/files/roms/bootroms/).
