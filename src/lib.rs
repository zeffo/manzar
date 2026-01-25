use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::prelude::*;
use web_sys::{HtmlElement, MouseEvent};

// (x, y)
struct Point(i32, i32);

enum Direction {
    N,
    S,
    E,
    W,
    NE,
    NW,
    SE,
    SW,
    None,
}

impl Direction {
    fn from_deltas(dx: f32, dy: f32) -> Self {
        match (dy > 0.5, dy < -0.5, dx > 0.5, dx < -0.5) {
            (true, _, _, true) => Direction::NE,
            (true, _, true, _) => Direction::NW,
            (_, true, _, true) => Direction::SE,
            (_, true, true, _) => Direction::SW,
            (true, _, _, _) => Direction::N,
            (_, true, _, _) => Direction::S,
            (_, _, true, _) => Direction::W,
            (_, _, _, true) => Direction::E,
            _ => Direction::None,
        }
    }
}

enum AnimationDuration {
    Infinite,
    Definite(u32),
}

struct Animation {
    states: &'static [Point],
    duration: AnimationDuration,
    speed: u32,
}

impl Animation {
    fn is_infinite(&self) -> bool {
        matches!(self.duration, AnimationDuration::Infinite)
    }
}

enum Sprite {
    Static(Point),
    Animated(Animation),
}

struct CardinalSprites {
    n: Sprite,
    e: Sprite,
    s: Sprite,
    w: Sprite,
}

struct OrdinalSprites {
    ne: Sprite,
    se: Sprite,
    sw: Sprite,
    nw: Sprite,
}

// When the cat is scratching a page wall
struct ScratchSprites {
    cat: Sprite,
    cardinal: CardinalSprites,
}

struct ManzarSprites {
    idle: Sprite,
    alert: Sprite,
    tired: Sprite,
    sleeping: Sprite,
    cardinal: CardinalSprites,
    ordinal: OrdinalSprites,
    scratch: ScratchSprites,
}

struct AnimationState {
    sprite: &'static Sprite,
    frame: u32,
}

struct IdleState {
    timeout: u32,
    frame: u32,
    buffer: u32,
}

static SPRITES: ManzarSprites = ManzarSprites {
    idle: Sprite::Static(Point(-3, -3)),
    alert: Sprite::Static(Point(-7, -3)),
    tired: Sprite::Static(Point(-3, -2)),
    sleeping: Sprite::Animated(Animation {
        states: &[Point(-2, 0), Point(-2, -1)],
        duration: AnimationDuration::Infinite,
        speed: 25,
    }),
    cardinal: CardinalSprites {
        n: Sprite::Animated(Animation {
            states: &[Point(-1, -2), Point(-1, -3)],
            duration: AnimationDuration::Infinite,
            speed: 100,
        }),
        e: Sprite::Animated(Animation {
            states: &[Point(-3, 0), Point(-3, -1)],
            duration: AnimationDuration::Infinite,
            speed: 100,
        }),
        s: Sprite::Animated(Animation {
            states: &[Point(-6, -3), Point(-7, -2)],
            duration: AnimationDuration::Infinite,
            speed: 100,
        }),
        w: Sprite::Animated(Animation {
            states: &[Point(-4, -2), Point(-4, -3)],
            duration: AnimationDuration::Infinite,
            speed: 100,
        }),
    },
    ordinal: OrdinalSprites {
        ne: Sprite::Animated(Animation {
            states: &[Point(0, -2), Point(0, -3)],
            duration: AnimationDuration::Infinite,
            speed: 100,
        }),
        se: Sprite::Animated(Animation {
            states: &[Point(-5, -1), Point(-5, -2)],
            duration: AnimationDuration::Infinite,
            speed: 100,
        }),
        sw: Sprite::Animated(Animation {
            states: &[Point(-5, -3), Point(-6, -1)],
            duration: AnimationDuration::Infinite,
            speed: 100,
        }),
        nw: Sprite::Animated(Animation {
            states: &[Point(-1, 0), Point(-1, -1)],
            duration: AnimationDuration::Infinite,
            speed: 100,
        }),
    },
    scratch: ScratchSprites {
        cat: Sprite::Animated(Animation {
            states: &[Point(-5, 0), Point(-6, 0), Point(-7, 0)],
            duration: AnimationDuration::Definite(20),
            speed: 100,
        }),
        cardinal: CardinalSprites {
            n: Sprite::Animated(Animation {
                states: &[Point(0, 0), Point(0, -1)],
                duration: AnimationDuration::Definite(20),
                speed: 100,
            }),
            e: Sprite::Animated(Animation {
                states: &[Point(-2, -2), Point(-2, -3)],
                duration: AnimationDuration::Definite(20),
                speed: 100,
            }),
            w: Sprite::Animated(Animation {
                states: &[Point(-4, 0), Point(-4, -1)],
                duration: AnimationDuration::Definite(20),
                speed: 100,
            }),
            s: Sprite::Animated(Animation {
                states: &[Point(-7, -1), Point(-6, -2)],
                duration: AnimationDuration::Definite(20),
                speed: 100,
            }),
        },
    },
};

struct ManzarState {
    element: HtmlElement,
    mouse: Point,
    cat: Point,
    speed: i32,
    frame: u32,
    animation: AnimationState,
    idle: IdleState,
    window_size: (i32, i32),
}

impl ManzarState {
    fn on_mouse_down(&mut self, event: &MouseEvent) {
        let x = event.client_x();
        let y = event.client_y();
        self.mouse = Point(x, y);
    }

    fn get_cardinal_scratch_sprite(&self) -> &'static Sprite {
        let cx = self.cat.0;
        let cy = self.cat.1;
        let wx = self.window_size.0;
        let wy = self.window_size.1;
        let margin = 10;

        let scratch = &SPRITES.scratch;
        let distances = [
            (cx, &scratch.cardinal.w),
            (cy, &scratch.cardinal.n),
            (wx - cx, &scratch.cardinal.e),
            (wy - cy, &scratch.cardinal.s),
        ];

        distances
            .iter()
            .filter(|(d, _)| *d < margin)
            .min_by_key(|(d, _)| *d)
            .map(|(_, sprite)| *sprite)
            .unwrap_or(&scratch.cat)
    }

    fn render(&mut self) {
        self.frame += 1;

        let diff_x = self.cat.0 - self.mouse.0;
        let diff_y = self.cat.1 - self.mouse.1;
        let dist = ((diff_x.pow(2) + diff_y.pow(2)) as f32).abs().sqrt();

        let speed = self.speed as f32;

        // Idle Logic (cat close to mouse)
        if dist < speed {
            if self.idle.frame == 0 {
                self.set_sprite(&SPRITES.idle);
                self.idle.frame = 1;
            } else {
                self.idle.frame += 1;
                if self.idle.frame >= self.idle.timeout {
                    let diff = self.idle.frame - self.idle.timeout;

                    // change below to adjust scratch frequency
                    // (we don't have access to rng)
                    let scratch_flag = self.frame % 101 == 0;

                    if diff > 40 {
                        self.set_sprite(&SPRITES.sleeping);
                    } else if scratch_flag {
                        self.set_sprite(self.get_cardinal_scratch_sprite());
                    } else if (20..40).contains(&diff) {
                        self.set_sprite(&SPRITES.tired);
                    } else {
                        self.set_sprite(self.animation.sprite);
                    }
                }
            }
            if self.idle.buffer == 0 {
                // change below to adjust alert time
                self.idle.buffer = 5;
            }
            return;
        }

        self.idle.frame = 0;
        if self.idle.buffer > 0 {
            self.idle.buffer -= 1;
            self.set_sprite(&SPRITES.alert);
            return;
        }

        let cur_x = self.cat.0 as f32;
        let cur_y = self.cat.1 as f32;

        let dx = diff_x as f32 / dist;
        let dy = diff_y as f32 / dist;

        let x = cur_x - dx * speed;
        let y = cur_y - dy * speed;

        let direction = Direction::from_deltas(dx, dy);
        let sprite = Self::get_compass_sprite(direction);
        self.set_sprite(sprite);
        match &self.animation.sprite {
            Sprite::Static(_) => (),
            Sprite::Animated(anim) => {
                if !anim.is_infinite() {
                    return; // Don't move if a definite animation is playing
                }
            }
        }
        self.move_to(x.round() as i32, y.round() as i32);
    }

    /// Change the sprite while respecting currently playing animations
    fn set_sprite(&mut self, sprite: &'static Sprite) {
        let cur = self.animation.sprite;
        let target = match cur {
            Sprite::Animated(anim) => match &anim.duration {
                AnimationDuration::Definite(_) => cur, // if we are currently playing a definite
                // animation, lets finish it before changing sprites
                AnimationDuration::Infinite => sprite,
            },
            Sprite::Static(_) => sprite,
        };
        self._set_sprite(target);
    }

    fn _set_sprite(&mut self, sprite: &'static Sprite) {
        let pt = match sprite {
            Sprite::Animated(anim) => {
                match anim.duration {
                    AnimationDuration::Definite(duration) => {
                        if duration <= self.animation.frame {
                            self._set_sprite(&SPRITES.idle);
                            self.animation.frame = 0;
                            self.idle.frame = 0;
                            self.frame = 0;
                            return;
                        }
                    }
                    AnimationDuration::Infinite => (),
                }
                let len = anim.states.len() as u32;
                self.animation.frame += 1;
                &anim.states[((self.animation.frame / (100 / anim.speed)) % len) as usize]
            }
            Sprite::Static(pt) => {
                self.animation.frame = 0;
                pt
            }
        };
        self.animation.sprite = sprite;
        self.element
            .style()
            .set_property(
                "background-position",
                &format!("{}px {}px", pt.0 * 32, pt.1 * 32),
            )
            .unwrap();
    }

    fn move_to(&mut self, x: i32, y: i32) {
        let get_style = |v: i32| format!("{}px", v);
        let style = self.element.style();

        style
            .set_property("left", get_style(x - 16).as_str())
            .unwrap();
        style
            .set_property("top", get_style(y - 16).as_str())
            .unwrap();

        self.cat = Point(x, y);
    }

    fn get_compass_sprite(direction: Direction) -> &'static Sprite {
        let s = &SPRITES;
        match direction {
            Direction::N => &s.cardinal.n,
            Direction::S => &s.cardinal.s,
            Direction::W => &s.cardinal.w,
            Direction::E => &s.cardinal.e,
            Direction::NW => &s.ordinal.nw,
            Direction::NE => &s.ordinal.ne,
            Direction::SW => &s.ordinal.sw,
            Direction::SE => &s.ordinal.se,
            Direction::None => &s.idle,
        }
    }
}

#[derive(Clone)]
struct Manzar {
    state: Rc<RefCell<ManzarState>>,
}

#[wasm_bindgen]
pub unsafe fn start(sprites_path: String) -> Result<(), JsValue> {
    let window = web_sys::window().expect("no window exists.");
    let document = window.document().expect("no document exists.");
    let body = document.body().expect("document does not have a body.");
    let div = document
        .create_element("div")
        .unwrap()
        .dyn_into::<HtmlElement>()?;

    div.set_id("Manzar");

    let styles: [(&str, &str); 7] = [
        ("height", "32px"),
        ("width", "32px"),
        ("top", "16px"),
        ("left", "16px"),
        (
            "background-image",
            &format!("url('{}')", sprites_path.as_str()),
        ),
        ("position", "fixed"),
        ("image-rendering", "pixelated"),
    ];

    for (prop, val) in &styles {
        div.style().set_property(prop, val)?;
    }
    body.append_child(&div)?;

    let de = document.document_element().unwrap();

    let manzar_state = ManzarState {
        element: div,
        mouse: Point(32, 32),
        cat: Point(32, 32),
        speed: 10,
        frame: 0,
        animation: AnimationState {
            sprite: &SPRITES.idle,
            frame: 0,
        },
        idle: IdleState {
            timeout: 50,
            frame: 0,
            buffer: 0,
        },
        window_size: (de.scroll_width(), de.scroll_height()),
    };

    let manzar = Manzar {
        state: Rc::new(RefCell::new(manzar_state)),
    };

    // https://rustwasm.github.io/wasm-bindgen/examples/closures.html

    let mouse_clone = manzar.clone();

    let mouse_callback = Closure::<dyn FnMut(_)>::new(move |e: MouseEvent| {
        mouse_clone.state.borrow_mut().on_mouse_down(&e);
    });

    let frame_clone = manzar.clone();
    let frame_update = Closure::<dyn FnMut()>::new(move || {
        frame_clone.state.borrow_mut().render();
    });

    document
        .add_event_listener_with_callback("mousedown", mouse_callback.as_ref().unchecked_ref())?;
    window.set_interval_with_callback_and_timeout_and_arguments_0(
        frame_update.as_ref().unchecked_ref(),
        100,
    )?;

    mouse_callback.forget();
    frame_update.forget();

    Ok(())
}
