# hledger-desktop

desktop app for [hledger][] built with [tauri][] and [egui][]

## roadmap

- [ ] feature parity with hledger-web
- [ ] update / delete transactions

## development

### setup

[hledger-desktop][] runs [hledger-web] instances to read and write data, so you need to prepare the binary first:

1. [install hledger][]
2. put installed hledger-web binary into [binaries][], specifying your architecture
   ```bash
   cp $(which hledger-web) ./binaries/hledger-web-$(rustc -Vv | grep host | cut -d' ' -f2-)
   ```

### run

```bash
cargo tauri dev
```

[binaries]: ./binaries/
[hledger]: https://github.com/simonmichael/hledger
[tauri]: https://github.com/tauri-apps/tauri
[egui]: https://github.com/emilk/egui
[install hledger]: https://hledger.org/install.html
[hledger-desktop]: https://github.com/ngalaiko/hledger-desktop
[hledger-web]: https://github.com/simonmichael/hledger/tree/master/hledger-web
