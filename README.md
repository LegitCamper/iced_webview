# Iced_webview [![Rust](https://github.com/LegitCamper/iced_webview/actions/workflows/rust.yml/badge.svg)](https://github.com/LegitCamper/iced_webview/actions/workflows/rust.yml)

A library to embed Web views in iced applications

> Note: Currently this library only supports [Ultralight]/Webkit, but more rendering engines are planned to be supported.

> [Ultralight has its own licence](https://ultralig.ht/pricing/) that should be reviewed before deciding if it works for you

#### examples:

##### `examples/embedded_webview`
A simple example to showcase an embedded webview (uses the basic webview)
![image](https://raw.githubusercontent.com/LegitCamper/iced_webview/refs/heads/main/assets/embedded_webview.png)
```sh
cargo run --example embedded_webview --features ultralight-resources
```

##### `examples/multi_webview`
A more advanced example that uses the advanced webview module and has two simultaneous webviews open
![image](https://raw.githubusercontent.com/LegitCamper/iced_webview/refs/heads/main/assets/multi_view.png)
```sh
cargo run --example multi_webview --features ultralight-resources
```

## Extra files (Resources)

You need to include `resources` folder in the execution directory(same level as Cargo.toml).

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
