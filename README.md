# Iced_webview

A library to embed Web views in iced applications

> Note: Currently this library only supports [Ultralight]/Webkit, but more rendering engines are planned to be supported.

#### examples:

##### `examples/embedded_webview`
```sh
cargo run --example embedded_webview --features ultralight-resources
```

## Extra files

You need to include `resources` folder in the execution directory.

You can find the resources folder in the [Ultralight SDK]

> Rust will automatically symlink the directory for development with `--features --ultralight-resources`

## Deployment

The samples compiled rely on dynamic libraries provided by `Ultralight`:
- `libUltralightCore.so`/`UltralightCore.dll`
- `libUltralight.so`/`Ultralight.dll`
- `libWebCore.so`/`WebCore.dll`
- `libAppCore.so`/`AppCore.dll`

These can be downloaded from the [Ultralight SDK].

> Rust will download them during build as well, but are kept inside the `target` directory.

[Ultralight]: https://ultralig.ht
[Ultralight SDK]: https://ultralig.ht/download/
