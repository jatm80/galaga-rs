---
name: fui-rs
description: Getting started with fui-rs, the Rust SDK for building GPU-rendered, retained-mode apps (no HTML/CSS) that compile to WebAssembly on the EffinDom runtime. Use when scaffolding or building a fui-rs / EffinDom Rust app, writing retained UI with the ui! macro, doing immediate-mode drawing with custom_drawable/DrawContext, or when the user mentions fui-rs, fui-as, or EffinDom.
---

# fui-rs

Rust SDK for **EffinDom** — retained-mode UI compiled to WebAssembly
(`wasm32-unknown-unknown`). No DOM, no HTML, no CSS: you write Rust, it renders
on a GPU canvas via the shared browser runtime. (`fui-as` is the sibling
AssemblyScript SDK — same runtime, different language.)

## Prerequisites

Rust (stable) + `rustup target add wasm32-unknown-unknown`, Node.js ≥18 + npm.
Binaryen (`wasm-opt`) is optional and only speeds up release builds.

## Quick start

```bash
npx @effindomv2/create-fui-rs-app my-app          # add: -- --template routed  for micro-frontends
cd my-app && npm install
npm run dev                                        # fast debug WASM rebuilds + local server (URL printed)
npm run build                                      # optimized build
```

Your crate is a `cdylib`; app code always uses `use fui::prelude::*;` (avoid
`bindings`, `generated`, or raw FFI modules).

## Minimal app

```rust
use fui::prelude::*;

fn build_page() -> FlexBox {
    ui! {
        column().fill_size().padding(24.0, 24.0, 24.0, 24.0) {
            text("Hello from fui-rs").font_size(28.0),
            button("Click me").on_click(|_| logger::info("App", "clicked")),
        }
    }
}

fui_app!(FlexBox, build_page);   // emits the harness lifecycle exports; never hand-write __runApp
```

## Retained-mode model (the one thing to get right)

Build controls **once**, keep stateful ones as fields/clones, and mutate them in
callbacks. Never recreate controls in a render loop — that loses focus, scroll,
and subscription state. Controls are cheap cloned handles; clone them into
closures. Use `Rc<Cell<T>>` / `Rc<RefCell<T>>` for shared mutable state.

```rust
let count = Rc::new(Cell::new(0));
let label = text("Count: 0");
let button = button("Increment");
button.on_click({
    let (label, count) = (label.clone(), count.clone());
    move |_| { let n = count.get() + 1; count.set(n); label.text(format!("Count: {n}")); }
});
```

For page/controller ownership use `fui_managed_app!` + `fui_component!` (see EXAMPLES.md).

## Custom graphics / games

Subclass nothing — call `custom_drawable(|ctx| { ... })`, draw with immediate-mode
`DrawContext` calls, and animate by re-arming `set_timeout` + calling `mark_dirty()`.
There is **no automatic per-frame loop**. See [EXAMPLES.md](EXAMPLES.md).

## Ship it

`npm run build` emits a self-contained static bundle (`index.html`, `*.wasm`,
harness/runtime JS). Serve statically; **`.wasm` must be `application/wasm`**
(nginx ≥1.21 does this by default).

## More

- Full API (nodes, controls, DrawContext, timers, input, theming): [REFERENCE.md](REFERENCE.md)
- Runnable examples incl. an animated drawing canvas + keyboard: [EXAMPLES.md](EXAMPLES.md)

## Reference repos

- Rust SDK + docs: https://github.com/zion-sati/fui-rs
- SDK docs index: https://github.com/zion-sati/fui-rs/blob/main/docs/v2/fui-rs/SDK_INDEX.md
- Quickstart / API reference: `docs/v2/fui-rs/QUICKSTART.md`, `docs/v2/fui-rs/API_REFERENCE.md`
- AssemblyScript sibling SDK: https://github.com/zion-sati/fui-as
- npm: `@effindomv2/create-fui-rs-app`, `@effindomv2/runtime`
