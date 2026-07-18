# galaga-rs

A basic Galaga-style game implemented with [fui-rs](https://github.com/zion-sati/fui-rs).

The app source uses `fui_app!`; it does not hand-write `#[no_mangle]` lifecycle exports.

Claude skill docs for working on FUI-RS are available in `.claude/skills/fui-rs/`.

Run with Docker Compose:

```sh
docker compose up --build
```

Then open <http://localhost:8080>.

For local development without Docker:

```sh
npm install
npm run build
npm run dev
```

Create an optimized static deployment in `published/`:

```sh
npm run publish
```
