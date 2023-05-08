# Macrospace Fatal Force extractor

mff-extract is a command line utility written in Rust used to extract PAK archives (`resource0.pak`) from the J2ME game
Fatal Force: Earth Assault (2004) by Macrospace

## Usage 

```console
$ mff-extract list resource0.pak
Listing archive: resource0.pak
Physical size: 163.5 KiB
File count: 118
|     Size |  Offset  | File Name           |
|----------|----------|---------------------|
|    273 B | 00000836 | a.png               |
|    138 B | 00000947 | a2.png              |
...
|  2.8 KiB | 00027a51 | _e.png              |
|  2.1 KiB | 00028588 | _i.png              |
```

```console
$ mff-extract extract resource0.pak -v 
a.png [273 B]
a2.png [138 B]
...
_e.png [2.8 KiB]
_i.png [2.1 KiB]
Extracted 118 files.
```