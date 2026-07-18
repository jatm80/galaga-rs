# fui-rs API reference

App code: `use fui::prelude::*;`. Everything below is re-exported there. Colors
are packed `u32` from `rgb(r,g,b)` / `rgba(r,g,b,a)` (`with_alpha`, `mix_color`,
`hsl_to_color` also available). Setters return `&Self` — chain them; `ui!` accepts
those borrowed fluent expressions directly.

## App entrypoint macros

- `fui_app!(RootType, build_fn)` — simplest: `build_fn() -> RootType` (e.g. `FlexBox`).
- `fui_managed_app!(Page, Page::new, |p: &Page| p.root.clone())` — retained page/controller
  ownership; optional `mount:` / `dispose:` callbacks for route-scoped resources.
- `fui_component!(Name => root)` or `fui_component!(Name => root, owner: state_field)` —
  derive the component boilerplate for a struct whose root node is `root`.

Both macros emit the `#[no_mangle]` harness exports — never hand-write `__runApp`.

## Layout & containers

- `column()`, `row()`, `flex_box()` → `FlexBox`.
- Builders: `.fill_size()`, `.fill_width()`, `.fill_height()`, `.width(f32)`, `.height(f32)`,
  `.padding(t,r,b,l)`, `.margin(t,r,b,l)`, `.gap(f32)`, `.flex_wrap(FlexWrap::Wrap)`,
  `.justify_content(...)`, `.align_items(...)`, `.child(node)`.
- `ui! { column().gap(12.0) { child_a, row() { child_b, child_c }, } }` — sugar over the
  builders for mixed child trees (Rust `Vec` needs one type; `ui!` avoids that).

## Controls & text nodes

- `text("s")`: `.text(format!("{n}"))` (mutate live), `.font_size(f32)`, `.text_color(u32)`,
  `.text_align(...)`, `.font_weight(...)`, `.font_family(...)`.
- `button("label").on_click(|_ev| { ... })`.
- `rich_text!["plain ".italic(), { format!("{v}") }.bold().text_color(rgb(0x3a,0xc5,0x6c)), span => prebuilt]`.
- `text_input()`, `text_area()`, `checkbox()`, `switch()`, `slider()`, `dropdown()`,
  `combo_box()`, `radio_group()`, `progress_bar()`, `scroll_box()`, `dialog()`, `popup()`,
  `nav_link()`, `selection_area()`. Configure with `.configure(|c| { ... })` where needed.

## Immediate-mode drawing

```rust
let canvas = custom_drawable(move |ctx| { /* runs only when dirty */ });
canvas.width(480.0).height(600.0);
// later, when state changes:
canvas.mark_dirty();
```

- `custom_drawable(impl Fn(&mut DrawContext) + 'static) -> CustomDrawable`.
- **Dirty model**: draws once on first frame, then only after `mark_dirty()`. No 60fps
  loop — drive animation from a timer (below).
- `DrawContext`: `draw_rect(x,y,w,h,paint)`, `draw_round_rect(x,y,w,h,rx,ry,paint)`,
  `draw_circle(cx,cy,r,paint)`, `draw_line(x1,y1,x2,y2,color:u32,stroke_width:f32)`,
  `draw_image(texture_id,x,y,w,h)`, `draw_text_layout(&layout,x,y)`, `draw_text_node(&node,x,y)`,
  plus `save/restore`, `translate/scale/rotate`, `clip*`.
- `Paint::fill(color)`, `Paint::stroke(color, width)`, `Paint::filled_stroke(fill, stroke, w)`.
- `Bitmap` for direct pixels + `draw_image(bitmap.texture_id(), ...)` (see demo `paint_canvas.rs`).

## Timers / game loop

```rust
use fui::prelude::*;
fn schedule_tick(state: std::rc::Weak<GameState>) {
    set_timeout(33, move || {                 // ~30fps
        let Some(state) = state.upgrade() else { return; };
        state.tick();                          // update + state.canvas.mark_dirty()
        schedule_tick(std::rc::Rc::downgrade(&state));  // re-arm
    });
}
```

- `set_timeout(delay_ms: i32, impl Fn() + 'static) -> TimerHandle` — one-shot; re-call to loop.
- `cancel_timeout(handle)`, `cancel_all_timers()`.
- Capture a `Weak` to the state so the loop stops itself when the page is dropped.
- `on_loaded(move |_ev| { ... })` — run after first layout (focus a node, start the loop).

## Keyboard & pointer input

- Focus first: `.focusable(true, 0)` (enabled, tab_index) + `.focus_now()` (after `on_loaded`).
- `.on_key_down(move |e: &mut KeyEventArgs| { ... })`, `.on_key_up(...)`.
  `KeyEventArgs { event_type: KeyEventType, key: String, modifiers: u32, handled: bool }`.
  `e.key` is a DOM-style string: `"ArrowLeft"`, `"ArrowRight"`, `" "`, `"Enter"`, `"Escape"`, `"a"`…
  Set `e.handled = true` to consume. Match with `e.key.as_str()`.
- Pointer handlers (`on_pointer_down/move/up`), pointer capture, and gestures also exist.

## State & theming

- Shared mutable state: `Rc<Cell<T>>` (copy types) / `Rc<RefCell<T>>` (rest); clone into closures.
- `current_theme() -> Theme`, `theme.colors...`, `theme.fonts...`; theme-change subscription helpers.
- `logger::info("Tag", "msg")` for debug logging.

## Rust / WASM gotchas

- Crate is `crate-type = ["cdylib"]`, target `wasm32-unknown-unknown` (the scaffold sets this).
- Drawing/timer closures are `'static` — move `Rc`/handle clones in; don't borrow locals.
- Callbacks are `Fn` (not `FnMut`) — mutate through `Cell`/`RefCell`, not captured `&mut`.
- Keep control handles as struct fields when a later method/callback must mutate them.
- `f32` for geometry; `std::f32::consts::PI`, `.sin()/.cos()`, `rand` via your own crate if needed.
