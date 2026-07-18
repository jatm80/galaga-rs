# fui-rs examples

All examples use `use fui::prelude::*;` and compile with the scaffold as-is.

## 1. Retained page with managed ownership

```rust
use fui::prelude::*;
use std::cell::Cell;
use std::rc::Rc;

#[derive(Clone)]
struct CounterPage {
    root: FlexBox,
    count_label: Text,
}

fui_component!(CounterPage => root);

impl CounterPage {
    fn new() -> Self {
        let count_label = text("Count: 0");
        let button = button("Increment");
        let count = Rc::new(Cell::new(0));

        button.on_click({
            let (count_label, count) = (count_label.clone(), count.clone());
            move |_| {
                let next = count.get() + 1;
                count.set(next);
                count_label.text(format!("Count: {next}"));
            }
        });

        let root = ui! {
            column().padding(16.0, 16.0, 16.0, 16.0).gap(8.0) {
                count_label.clone(),
                button,
            }
        };
        Self { root, count_label }
    }
}

fui_managed_app!(CounterPage, CounterPage::new, |p: &CounterPage| p.root.clone());
```

## 2. Animated custom drawing + keyboard (the game pattern)

Key ideas: `custom_drawable` for pixels, shared state in `Rc<...Cell>`, a
self-re-arming `set_timeout` loop that calls `mark_dirty()`, and
`focusable(true, 0)` + `focus_now()` so the canvas receives keys.

```rust
use fui::prelude::*;
use std::cell::Cell;
use std::rc::{Rc, Weak};

const W: f32 = 400.0;
const H: f32 = 300.0;

// Game data only — no canvas handle here, so there's no Rc reference cycle.
struct GameState {
    x: Cell<f32>, y: Cell<f32>,
    vx: Cell<f32>, vy: Cell<f32>,
    left: Cell<bool>, right: Cell<bool>,
}

impl GameState {
    fn step(&self) {
        if self.left.get()  { self.x.set(self.x.get() - 5.0); }
        if self.right.get() { self.x.set(self.x.get() + 5.0); }
        self.x.set(self.x.get() + self.vx.get());
        self.y.set(self.y.get() + self.vy.get());
        if self.x.get() < 10.0 || self.x.get() > W - 10.0 { self.vx.set(-self.vx.get()); }
        if self.y.get() < 10.0 || self.y.get() > H - 10.0 { self.vy.set(-self.vy.get()); }
    }
}

// The loop holds a Weak to the state (so it stops when the page drops) and a
// clone of the canvas handle (cheap) to request repaints.
fn schedule_tick(state: Weak<GameState>, canvas: CustomDrawable) {
    set_timeout(33, move || {                       // ~30fps
        let Some(state) = state.upgrade() else { return; };
        state.step();
        canvas.mark_dirty();
        schedule_tick(Rc::downgrade(&state), canvas);  // re-arm the loop
    });
}

fn build_game() -> FlexBox {
    let state = Rc::new(GameState {
        x: Cell::new(200.0), y: Cell::new(150.0),
        vx: Cell::new(3.0), vy: Cell::new(2.0),
        left: Cell::new(false), right: Cell::new(false),
    });

    // Draw closure owns a clone of the (single) state and reads live positions.
    let canvas = custom_drawable({
        let state = state.clone();
        move |ctx| {
            ctx.draw_rect(0.0, 0.0, W, H, Paint::fill(rgb(6, 8, 22)));
            ctx.draw_circle(state.x.get(), state.y.get(), 10.0, Paint::fill(rgb(90, 220, 255)));
        }
    });
    canvas.width(W).height(H).focusable(true, 0);

    canvas.on_key_down({
        let s = state.clone();
        move |e| match e.key.as_str() {
            "ArrowLeft"  => { s.left.set(true);  e.handled = true; }
            "ArrowRight" => { s.right.set(true); e.handled = true; }
            _ => {}
        }
    });
    canvas.on_key_up({
        let s = state.clone();
        move |e| match e.key.as_str() {
            "ArrowLeft"  => s.left.set(false),
            "ArrowRight" => s.right.set(false),
            _ => {}
        }
    });

    on_loaded({
        let canvas = canvas.clone();
        let state = state.clone();
        move |_| { canvas.focus_now(); schedule_tick(Rc::downgrade(&state), canvas.clone()); }
    });

    ui! { column().fill_size() { canvas.clone() } }
}

fui_app!(FlexBox, build_game);
```

## 3. Deploy the static bundle (Docker)

`npm run build` emits a self-contained `public/` (or scaffold's dist dir). Serve
it with nginx:

```dockerfile
FROM node:22-slim AS build
WORKDIR /app
COPY package.json package-lock.json ./
RUN npm ci
COPY . .
RUN npm run build
FROM nginx:1.27-alpine
COPY --from=build /app/public /usr/share/nginx/html   # adjust to the scaffold's build output dir
EXPOSE 80
```

The build stage also needs Rust + `wasm32-unknown-unknown`; either use a Rust base
image with Node added, or install rustup in the build stage before `npm run build`.
nginx ≥1.21 serves `.wasm` as `application/wasm`, which the runtime requires.
