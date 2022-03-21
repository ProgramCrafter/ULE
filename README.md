### ULE - Minecraft's server core written in Rust

> Fork by ProgramCrafter
> - already done
>   - added support for clients grouping some packets in one TCP message
>   - fixed problem with serializing VarInts
>   - added support for favicons
> - in progress
>   - adding support for mods
> 
> If you want to contribute - **W.I.P.**

Original description by Distemi:
```
This's server core fully written in Rust-Lang and using more custom code
for best perfomance and controlling.
```

If you want to [contribute - i'm exists on Patreon.](https://github.com/Distemi/ULE)

What's libraries using for server's core:
- ahash ( best HashMap )
- lazy_static ( Global variables )
- serde ( Serializing and Deserializing structs)
- serde_json ( convertor for JSON <-> Structs )
- log ( Logging framework )
- fern ( Logging framework's utilities )
- mio ( Single-threaded TCP and UDP server and client )
