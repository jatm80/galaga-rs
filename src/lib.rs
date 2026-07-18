// Galaga — a minimal shoot-'em-up drawn entirely with fui-rs immediate-mode
// drawing. No HTML, no CSS. One custom_drawable canvas + a retained status line.
// Ported from the fui-as (AssemblyScript) version.
use fui::prelude::*;
use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};

const FIELD_W: f32 = 480.0;
const FIELD_H: f32 = 600.0;

const SHIP_W: f32 = 34.0;
const SHIP_H: f32 = 22.0;
const SHIP_SPEED: f32 = 5.0;
const SHIP_Y: f32 = FIELD_H - 40.0;

const ENEMY_COLS: i32 = 8;
const ENEMY_ROWS: i32 = 4;
const ENEMY_W: f32 = 30.0;
const ENEMY_H: f32 = 22.0;
const ENEMY_GAP_X: f32 = 22.0;
const ENEMY_GAP_Y: f32 = 20.0;
const ENEMY_TOP: f32 = 60.0;

const P_BULLET_SPEED: f32 = 9.0;
const E_BULLET_SPEED: f32 = 4.0;
const MAX_P_BULLETS: usize = 3;
const MAX_E_BULLETS: usize = 6;

const TICK_MS: i32 = 33; // ~30 fps
const EXPLO_LIFE: i32 = 14; // ticks an explosion lives
const PAGE_PADDING_X: f32 = 32.0;
const MIN_PLAYFIELD_W: f32 = 260.0;
const MOBILE_CONTROL_H: f32 = 54.0;

fn col_bg() -> u32 {
    rgba(6, 8, 22, 255)
}
fn enemy_color(row: i32) -> u32 {
    match row {
        0 => rgba(255, 90, 110, 255),
        1 => rgba(255, 160, 60, 255),
        2 => rgba(120, 230, 140, 255),
        _ => rgba(190, 130, 255, 255),
    }
}

fn game_scale() -> f32 {
    let viewport_w = fui::bindings::ui::get_viewport_width();
    ((viewport_w - PAGE_PADDING_X).max(MIN_PLAYFIELD_W) / FIELD_W).min(1.0)
}

fn fit_canvas_to_viewport(canvas: &CustomDrawable) {
    let scale = game_scale();
    canvas
        .width(FIELD_W * scale, Unit::Pixel)
        .height(FIELD_H * scale, Unit::Pixel);
}

#[derive(Clone, Copy)]
struct Bullet {
    x: f32,
    y: f32,
    active: bool,
}

#[derive(Clone, Copy)]
struct Enemy {
    x: f32,
    y: f32,
    alive: bool,
    row: i32,
}

#[derive(Clone, Copy)]
struct Star {
    x: f32,
    y: f32,
    speed: f32,
}

#[derive(Clone, Copy)]
struct Explosion {
    x: f32,
    y: f32,
    age: i32,
    color: u32,
}

struct GameState {
    ship_x: Cell<f32>,
    left: Cell<bool>,
    right: Cell<bool>,
    fire: Cell<bool>,
    fire_cooldown: Cell<i32>,

    enemies: RefCell<Vec<Enemy>>,
    p_bullets: RefCell<Vec<Bullet>>,
    e_bullets: RefCell<Vec<Bullet>>,
    stars: RefCell<Vec<Star>>,
    explosions: RefCell<Vec<Explosion>>,

    form_x: Cell<f32>,
    form_dir: Cell<f32>,
    form_step: Cell<f32>,

    score: Cell<i32>,
    lives: Cell<i32>,
    over: Cell<bool>,
    won: Cell<bool>,

    status: Text,
    rng: Cell<u32>,
}

impl GameState {
    fn new(status: Text) -> Self {
        let mut stars = Vec::with_capacity(48);
        // deterministic seed; positions filled below via rand()
        let s = Self {
            ship_x: Cell::new(FIELD_W / 2.0),
            left: Cell::new(false),
            right: Cell::new(false),
            fire: Cell::new(false),
            fire_cooldown: Cell::new(0),
            enemies: RefCell::new(Vec::new()),
            p_bullets: RefCell::new(vec![
                Bullet {
                    x: 0.0,
                    y: 0.0,
                    active: false
                };
                MAX_P_BULLETS
            ]),
            e_bullets: RefCell::new(vec![
                Bullet {
                    x: 0.0,
                    y: 0.0,
                    active: false
                };
                MAX_E_BULLETS
            ]),
            stars: RefCell::new(Vec::new()),
            explosions: RefCell::new(Vec::new()),
            form_x: Cell::new(0.0),
            form_dir: Cell::new(1.0),
            form_step: Cell::new(0.0),
            score: Cell::new(0),
            lives: Cell::new(3),
            over: Cell::new(false),
            won: Cell::new(false),
            status,
            rng: Cell::new(0x1234_5678),
        };
        for _ in 0..48 {
            stars.push(Star {
                x: s.rand() * FIELD_W,
                y: s.rand() * FIELD_H,
                speed: 0.4 + s.rand() * 1.6,
            });
        }
        *s.stars.borrow_mut() = stars;
        s.reset();
        s
    }

    // xorshift32 in [0, 1) — avoids pulling in a rand crate.
    fn rand(&self) -> f32 {
        let mut x = self.rng.get();
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.rng.set(x);
        (x as f32) / (u32::MAX as f32)
    }

    fn reset(&self) {
        self.score.set(0);
        self.lives.set(3);
        self.over.set(false);
        self.won.set(false);
        self.left.set(false);
        self.right.set(false);
        self.fire.set(false);
        self.fire_cooldown.set(0);
        self.ship_x.set(FIELD_W / 2.0);
        self.form_x.set(0.0);
        self.form_dir.set(1.0);
        self.form_step.set(0.0);

        let grid_w = ENEMY_COLS as f32 * ENEMY_W + (ENEMY_COLS - 1) as f32 * ENEMY_GAP_X;
        let start_x = (FIELD_W - grid_w) / 2.0;
        let mut enemies = Vec::with_capacity((ENEMY_ROWS * ENEMY_COLS) as usize);
        for r in 0..ENEMY_ROWS {
            for c in 0..ENEMY_COLS {
                enemies.push(Enemy {
                    x: start_x + c as f32 * (ENEMY_W + ENEMY_GAP_X),
                    y: ENEMY_TOP + r as f32 * (ENEMY_H + ENEMY_GAP_Y),
                    alive: true,
                    row: r,
                });
            }
        }
        *self.enemies.borrow_mut() = enemies;
        for b in self.p_bullets.borrow_mut().iter_mut() {
            b.active = false;
        }
        for b in self.e_bullets.borrow_mut().iter_mut() {
            b.active = false;
        }
        self.explosions.borrow_mut().clear();
        self.update_status();
    }

    fn spawn_explosion(&self, x: f32, y: f32, color: u32) {
        self.explosions.borrow_mut().push(Explosion {
            x,
            y,
            age: 0,
            color,
        });
    }

    fn step_explosions(&self) {
        let mut ex = self.explosions.borrow_mut();
        for e in ex.iter_mut() {
            e.age += 1;
        }
        ex.retain(|e| e.age < EXPLO_LIFE);
    }

    fn update_status(&self) {
        let mut s = format!("SCORE {}    LIVES {}", self.score.get(), self.lives.get());
        if self.won.get() {
            s += "    — YOU WIN! Enter to restart";
        } else if self.over.get() {
            s += "    — GAME OVER! Enter to restart";
        }
        self.status.text(s);
    }

    fn tick(&self) {
        self.step_stars();
        self.step_explosions(); // animate even after game over (ship death)
        if !self.over.get() {
            self.step_ship();
            self.step_enemies();
            self.step_bullets();
            self.check_collisions();
            self.maybe_enemy_fire();
        }
    }

    fn step_stars(&self) {
        // rand() only touches self.rng (a Cell), so it's safe to call while
        // stars is borrowed — no need for a second pass.
        for s in self.stars.borrow_mut().iter_mut() {
            s.y += s.speed;
            if s.y > FIELD_H {
                s.y = 0.0;
                s.x = self.rand() * FIELD_W;
            }
        }
    }

    fn step_ship(&self) {
        let mut x = self.ship_x.get();
        if self.left.get() {
            x -= SHIP_SPEED;
        }
        if self.right.get() {
            x += SHIP_SPEED;
        }
        let half = SHIP_W / 2.0;
        x = x.clamp(half, FIELD_W - half);
        self.ship_x.set(x);

        let cd = self.fire_cooldown.get();
        if cd > 0 {
            self.fire_cooldown.set(cd - 1);
        }
        if self.fire.get() && self.fire_cooldown.get() == 0 {
            let mut bullets = self.p_bullets.borrow_mut();
            if let Some(b) = bullets.iter_mut().find(|b| !b.active) {
                b.active = true;
                b.x = x;
                b.y = SHIP_Y - SHIP_H;
                self.fire_cooldown.set(8);
            }
        }
    }

    fn alive_count(&self) -> i32 {
        self.enemies.borrow().iter().filter(|e| e.alive).count() as i32
    }

    fn step_enemies(&self) {
        let (mut min_x, mut max_x, mut any) = (99999.0_f32, -99999.0_f32, false);
        {
            let enemies = self.enemies.borrow();
            let fx = self.form_x.get();
            for e in enemies.iter() {
                if !e.alive {
                    continue;
                }
                any = true;
                let x = e.x + fx;
                if x < min_x {
                    min_x = x;
                }
                if x + ENEMY_W > max_x {
                    max_x = x + ENEMY_W;
                }
            }
        }
        if !any {
            self.won.set(true);
            self.over.set(true);
            self.update_status();
            return;
        }

        let alive = self.alive_count();
        let speed = 0.8 + (ENEMY_ROWS * ENEMY_COLS - alive) as f32 * 0.06;
        let dir = self.form_dir.get();
        self.form_x.set(self.form_x.get() + dir * speed);
        if max_x + dir * speed >= FIELD_W - 6.0 {
            self.form_dir.set(-1.0);
            self.form_step.set(self.form_step.get() + 14.0);
        } else if min_x + dir * speed <= 6.0 {
            self.form_dir.set(1.0);
            self.form_step.set(self.form_step.get() + 14.0);
        }

        // Reaching the ship line = game over.
        let fs = self.form_step.get();
        let reached = self
            .enemies
            .borrow()
            .iter()
            .any(|e| e.alive && e.y + fs + ENEMY_H >= SHIP_Y - SHIP_H);
        if reached {
            self.lives.set(0);
            self.over.set(true);
            self.update_status();
        }
    }

    fn step_bullets(&self) {
        for b in self.p_bullets.borrow_mut().iter_mut() {
            if !b.active {
                continue;
            }
            b.y -= P_BULLET_SPEED;
            if b.y < -8.0 {
                b.active = false;
            }
        }
        for b in self.e_bullets.borrow_mut().iter_mut() {
            if !b.active {
                continue;
            }
            b.y += E_BULLET_SPEED;
            if b.y > FIELD_H + 8.0 {
                b.active = false;
            }
        }
    }

    fn maybe_enemy_fire(&self) {
        if self.rand() > 0.06 {
            return;
        }
        let alive = self.alive_count();
        if alive == 0 {
            return;
        }
        let target = (self.rand() * alive as f32) as i32;

        let (bx, by) = {
            let enemies = self.enemies.borrow();
            let fx = self.form_x.get();
            let fs = self.form_step.get();
            let mut idx = 0;
            let mut spawn = None;
            for e in enemies.iter() {
                if !e.alive {
                    continue;
                }
                if idx == target {
                    spawn = Some((e.x + fx + ENEMY_W / 2.0, e.y + fs + ENEMY_H));
                    break;
                }
                idx += 1;
            }
            match spawn {
                Some(p) => p,
                None => return,
            }
        };
        if let Some(b) = self.e_bullets.borrow_mut().iter_mut().find(|b| !b.active) {
            b.active = true;
            b.x = bx;
            b.y = by;
        }
    }

    fn check_collisions(&self) {
        // Player bullets vs enemies.
        let fx = self.form_x.get();
        let fs = self.form_step.get();
        let mut enemy_booms: Vec<(f32, f32, u32)> = Vec::new();
        {
            let mut p_bullets = self.p_bullets.borrow_mut();
            let mut enemies = self.enemies.borrow_mut();
            for b in p_bullets.iter_mut() {
                if !b.active {
                    continue;
                }
                for e in enemies.iter_mut() {
                    if !e.alive {
                        continue;
                    }
                    let ex = e.x + fx;
                    let ey = e.y + fs;
                    if b.x >= ex && b.x <= ex + ENEMY_W && b.y >= ey && b.y <= ey + ENEMY_H {
                        e.alive = false;
                        b.active = false;
                        self.score.set(self.score.get() + 100);
                        enemy_booms.push((
                            ex + ENEMY_W / 2.0,
                            ey + ENEMY_H / 2.0,
                            enemy_color(e.row),
                        ));
                        break;
                    }
                }
            }
        }
        for (x, y, c) in enemy_booms {
            self.spawn_explosion(x, y, c);
        }
        self.update_status();

        // Enemy bullets vs ship.
        let ship_x = self.ship_x.get();
        let ship_l = ship_x - SHIP_W / 2.0;
        let ship_r = ship_x + SHIP_W / 2.0;
        let ship_t = SHIP_Y - SHIP_H;
        let mut hit = false;
        for b in self.e_bullets.borrow_mut().iter_mut() {
            if !b.active {
                continue;
            }
            if b.x >= ship_l && b.x <= ship_r && b.y >= ship_t && b.y <= SHIP_Y {
                b.active = false;
                hit = true;
            }
        }
        if hit {
            self.spawn_explosion(ship_x, SHIP_Y - SHIP_H / 2.0, rgba(255, 140, 40, 255));
            let lives = self.lives.get() - 1;
            self.lives.set(lives.max(0));
            if lives <= 0 {
                self.over.set(true);
            }
            self.update_status();
        }
    }

    fn draw(&self, ctx: &mut DrawContext) {
        ctx.draw_rect(0.0, 0.0, FIELD_W, FIELD_H, Paint::fill(col_bg()));

        for s in self.stars.borrow().iter() {
            ctx.draw_rect(
                s.x,
                s.y,
                1.5,
                1.5 + s.speed,
                Paint::fill(rgba(180, 190, 220, 160)),
            );
        }

        let fx = self.form_x.get();
        let fs = self.form_step.get();
        for e in self.enemies.borrow().iter() {
            if !e.alive {
                continue;
            }
            self.draw_enemy(ctx, e.x + fx, e.y + fs, enemy_color(e.row));
        }

        for b in self.p_bullets.borrow().iter() {
            if b.active {
                ctx.draw_rect(
                    b.x - 1.5,
                    b.y,
                    3.0,
                    12.0,
                    Paint::fill(rgba(255, 235, 90, 255)),
                );
            }
        }
        for b in self.e_bullets.borrow().iter() {
            if b.active {
                ctx.draw_circle(b.x, b.y, 4.0, Paint::fill(rgba(255, 90, 110, 255)));
            }
        }

        if !self.over.get() || self.won.get() {
            self.draw_ship(ctx);
        }

        // Explosions on top of everything.
        for e in self.explosions.borrow().iter() {
            self.draw_explosion(ctx, e);
        }
    }

    fn draw_explosion(&self, ctx: &mut DrawContext, e: &Explosion) {
        let p = e.age as f32 / EXPLO_LIFE as f32; // 0..1 progress
        let fade = 1.0 - p;
        let ring_alpha = (fade * 255.0) as u32;
        // Expanding shock ring.
        ctx.draw_circle(
            e.x,
            e.y,
            3.0 + p * 20.0,
            Paint::stroke(with_alpha(e.color, ring_alpha), 2.0),
        );
        // Bright core that dies faster.
        let core_alpha = (fade * fade * 255.0) as u32;
        ctx.draw_circle(
            e.x,
            e.y,
            fade * 7.0,
            Paint::fill(with_alpha(rgba(255, 255, 255, 255), core_alpha)),
        );
        // Radial debris particles.
        let dist = p * 18.0;
        let r = fade * 3.0;
        for k in 0..6 {
            let a = k as f32 * (std::f32::consts::PI / 3.0);
            ctx.draw_circle(
                e.x + a.cos() * dist,
                e.y + a.sin() * dist,
                r,
                Paint::fill(with_alpha(e.color, ring_alpha)),
            );
        }
    }

    fn draw_enemy(&self, ctx: &mut DrawContext, x: f32, y: f32, color: u32) {
        ctx.draw_round_rect(
            x,
            y + 4.0,
            ENEMY_W,
            ENEMY_H - 6.0,
            6.0,
            6.0,
            Paint::fill(color),
        );
        ctx.draw_rect(x - 4.0, y + 8.0, 6.0, 8.0, Paint::fill(color));
        ctx.draw_rect(x + ENEMY_W - 2.0, y + 8.0, 6.0, 8.0, Paint::fill(color));
        ctx.draw_circle(x + 9.0, y + 11.0, 2.5, Paint::fill(col_bg()));
        ctx.draw_circle(x + ENEMY_W - 9.0, y + 11.0, 2.5, Paint::fill(col_bg()));
    }

    fn draw_ship(&self, ctx: &mut DrawContext) {
        let cx = self.ship_x.get();
        let top = SHIP_Y - SHIP_H;
        let ship = rgba(90, 220, 255, 255);
        ctx.draw_round_rect(
            cx - SHIP_W / 2.0,
            top + 8.0,
            SHIP_W,
            SHIP_H - 8.0,
            4.0,
            4.0,
            Paint::fill(ship),
        );
        ctx.draw_line(cx, top, cx - 7.0, top + 12.0, ship, 3.0);
        ctx.draw_line(cx, top, cx + 7.0, top + 12.0, ship, 3.0);
        ctx.draw_circle(cx, top + 12.0, 3.5, Paint::fill(rgba(255, 255, 255, 255)));
    }
}

fn schedule_tick(state: Weak<GameState>, canvas: CustomDrawable) {
    set_timeout(TICK_MS, move || {
        let Some(state) = state.upgrade() else {
            return;
        };
        state.tick();
        fit_canvas_to_viewport(&canvas);
        canvas.mark_dirty();
        schedule_tick(Rc::downgrade(&state), canvas.clone());
    });
}

fn control_surface(label: &str, color: u32) -> FlexBox {
    let label = text(label);
    label
        .font_size(15.0)
        .text_align(TextAlign::Center)
        .text_color(rgba(235, 245, 255, 255))
        .width(100.0, Unit::Percent);

    let surface = flex_box();
    surface
        .fill_width()
        .height(MOBILE_CONTROL_H, Unit::Pixel)
        .padding(8.0, 10.0, 8.0, 10.0)
        .corner_radius(14.0)
        .bg_color(color)
        .justify_content(JustifyContent::Center)
        .align_items(AlignItems::Center)
        .child(&label);
    surface
}

fn build_game() -> SelectionArea {
    use_system_theme();

    let status = text("SCORE 0    LIVES 3");
    status
        .font_size(16.0)
        .text_align(TextAlign::Center)
        .text_color(rgb(180, 220, 255))
        .width(100.0, Unit::Percent);

    let state = Rc::new(GameState::new(status.clone()));

    let canvas = custom_drawable({
        let state = state.clone();
        move |ctx| {
            let scale = game_scale();
            ctx.save();
            ctx.scale(scale, scale);
            state.draw(ctx);
            ctx.restore();
        }
    });
    canvas
        .width(FIELD_W, Unit::Pixel)
        .height(FIELD_H, Unit::Pixel)
        .max_width(100.0, Unit::Percent)
        .focusable(true, 0);

    canvas.on_key_down({
        let state = state.clone();
        move |e| match e.key.as_str() {
            "ArrowLeft" | "a" | "A" => {
                state.left.set(true);
                e.handled = true;
            }
            "ArrowRight" | "d" | "D" => {
                state.right.set(true);
                e.handled = true;
            }
            " " | "Spacebar" => {
                state.fire.set(true);
                e.handled = true;
            }
            "Enter" | "r" | "R" => {
                if state.over.get() {
                    state.reset();
                }
                e.handled = true;
            }
            _ => {}
        }
    });
    canvas.on_key_up({
        let state = state.clone();
        move |e| match e.key.as_str() {
            "ArrowLeft" | "a" | "A" => state.left.set(false),
            "ArrowRight" | "d" | "D" => state.right.set(false),
            " " | "Spacebar" => state.fire.set(false),
            _ => {}
        }
    });

    let title = text("G A L A G A");
    title
        .font_size(30.0)
        .text_align(TextAlign::Center)
        .text_color(rgb(120, 230, 255))
        .width(100.0, Unit::Percent);
    let hint = text(
        "Desktop: ← → / A D move, Space fires, Enter/R restarts. Mobile: hold LEFT/RIGHT and FIRE.",
    );
    hint.font_size(13.0)
        .text_align(TextAlign::Center)
        .text_color(rgb(120, 140, 180))
        .width(100.0, Unit::Percent);

    let left_btn = control_surface("◀ HOLD LEFT", rgba(35, 61, 125, 255));
    left_btn.on_pointer_down({
        let state = state.clone();
        move |e| {
            state.left.set(true);
            e.handled = true;
        }
    });
    left_btn.on_pointer_up({
        let state = state.clone();
        move |e| {
            state.left.set(false);
            e.handled = true;
        }
    });
    left_btn.on_pointer_cancel({
        let state = state.clone();
        move |e| {
            state.left.set(false);
            e.handled = true;
        }
    });
    left_btn.on_pointer_leave({
        let state = state.clone();
        move |_e| state.left.set(false)
    });

    let fire_btn = control_surface("HOLD FIRE", rgba(125, 76, 22, 255));
    fire_btn.on_pointer_down({
        let state = state.clone();
        move |e| {
            if state.over.get() {
                state.reset();
            } else {
                state.fire.set(true);
            }
            e.handled = true;
        }
    });
    fire_btn.on_pointer_up({
        let state = state.clone();
        move |e| {
            state.fire.set(false);
            e.handled = true;
        }
    });
    fire_btn.on_pointer_cancel({
        let state = state.clone();
        move |e| {
            state.fire.set(false);
            e.handled = true;
        }
    });
    fire_btn.on_pointer_leave({
        let state = state.clone();
        move |_e| state.fire.set(false)
    });

    let right_btn = control_surface("HOLD RIGHT ▶", rgba(35, 61, 125, 255));
    right_btn.on_pointer_down({
        let state = state.clone();
        move |e| {
            state.right.set(true);
            e.handled = true;
        }
    });
    right_btn.on_pointer_up({
        let state = state.clone();
        move |e| {
            state.right.set(false);
            e.handled = true;
        }
    });
    right_btn.on_pointer_cancel({
        let state = state.clone();
        move |e| {
            state.right.set(false);
            e.handled = true;
        }
    });
    right_btn.on_pointer_leave({
        let state = state.clone();
        move |_e| state.right.set(false)
    });

    left_btn.margin(0.0, 4.0, 0.0, 0.0);
    fire_btn.margin(0.0, 4.0, 0.0, 4.0);
    right_btn.margin(0.0, 0.0, 0.0, 4.0);

    let controls = ui! {
        row()
            .width(FIELD_W, Unit::Pixel)
            .max_width(100.0, Unit::Percent) {
                left_btn,
                fire_btn,
                right_btn,
        }
    };

    let content = ui! {
        column()
            .fill_size()
            .padding(16.0, 16.0, 16.0, 16.0)
            .justify_content(JustifyContent::Center)
            .align_items(AlignItems::Center) {
                title,
                status.clone(),
                canvas.clone(),
                controls,
                hint,
        }
    };

    let root = selection_area();
    root.fill_size().child(&content).bind_theme(|root, theme| {
        root.bg_color(theme.colors.background);
    });

    on_loaded({
        let canvas = canvas.clone();
        let state = state.clone();
        move |_| {
            fit_canvas_to_viewport(&canvas);
            // No focus_now() on CustomDrawable in this SDK build; focus the node
            // directly so key events (which route to the focused node) land here.
            fui::bindings::ui::request_focus(canvas.handle().raw());
            schedule_tick(Rc::downgrade(&state), canvas.clone());
        }
    });

    root
}

fui_app!(SelectionArea, build_game);
