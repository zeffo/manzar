use std::{cell::RefCell, collections::HashMap, rc::Rc};
use wasm_bindgen::prelude::*;
use web_sys::{HtmlElement, MouseEvent};

// (x, y)
#[derive(Clone)]
struct Point(i32, i32);

#[derive(Clone)]
enum Sprite {
    Static(Point),
    Animated(Vec<Point>),
}

#[derive(Clone)]
struct CardinalSprites {
    n: Sprite,
    e: Sprite,
    s: Sprite,
    w: Sprite,
}

#[derive(Clone)]
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

struct ManzarState {
    element: HtmlElement,
    sprites: ManzarSprites,
    mouse: Point,
    cat: Point,
    speed: i32,
    frame: u32,
    idle_frame: u32,
    idle_timeout: u32,
    idle_buffer: u32,
}

impl ManzarState {
    /// on_mouse_down callback
    fn on_mouse_down(&mut self, event: MouseEvent) {
        let x = event.client_x();
        let y = event.client_y();
        self.mouse = Point(x, y);
    }

    fn render(&mut self) {
        self.frame = self.frame + 1;

        let diff_x = self.cat.0 - self.mouse.0;
        let diff_y = self.cat.1 - self.mouse.1;
        let dist = ((diff_x.pow(2) + diff_y.pow(2)) as f32).abs().sqrt();

        let speed = self.speed as f32;

        if dist < speed {
            // It's close enough to the mouse.
            if self.idle_frame == 0 {
                self.set_sprite(&self.sprites.idle.clone());
                self.idle_frame = 1;
            } else {
                self.idle_frame = self.idle_frame + 1;

                if self.idle_frame >= self.idle_timeout {
                    let diff = self.idle_frame - self.idle_timeout;
                    if diff > 10 && diff < 32 {
                        self.set_sprite(&self.sprites.scratch.cat.clone());
                    } else if diff > self.idle_timeout {
                        self.set_idle_sprite(&self.sprites.sleeping.clone());
                    } else {
                        self.set_sprite(&self.sprites.idle.clone());
                    }
                } else if (0..10).contains(&(self.idle_timeout - self.idle_frame)) {
                    self.set_sprite(&self.sprites.tired.clone());
                } else {
                    self.set_sprite(&self.sprites.idle.clone());
                }

                if self.idle_buffer == 0 {
                    self.idle_buffer = self.idle_timeout / 10; // change this to adjust alert time
                }
            }
            return ();
        }

        self.idle_frame = 0;

        if self.idle_buffer > 0 {
            self.idle_buffer = self.idle_buffer - 1;
            self.set_sprite(&self.sprites.alert.clone());
            return ();
        }

        let cur_x = self.cat.0 as f32;
        let cur_y = self.cat.1 as f32;

        // Make sure distance is not 0 here! WASM will give you an unreadable error!

        let dx = diff_x as f32 / dist;
        let dy = diff_y as f32 / dist;

        let x = cur_x - dx * speed;
        let y = cur_y - dy * speed;

        let mut direction = String::new();

        if dy > 0.5 {
            direction.push('N')
        } else if dy < -0.5 {
            direction.push('S')
        }

        if dx > 0.5 {
            direction.push('W')
        } else if dx < -0.5 {
            direction.push('E')
        }

        let sprite = self.get_compass_sprites(direction);
        // let sprite = set[(self.frame % 2) as usize];
        self.set_sprite(&sprite);
        self.move_to(x.round() as i32, y.round() as i32);
    }

    fn set_idle_sprite(&mut self, sprite: &Sprite) {
        let pt = match sprite {
            Sprite::Animated(pts) => &pts[((self.frame / 4) % (pts.len() as u32)) as usize],
            Sprite::Static(pt) => pt,
        };
        self.element
            .style()
            .set_property(
                "background-position",
                &format!("{}px {}px", pt.0 * 32, pt.1 * 32),
            )
            .unwrap();
    }

    fn set_sprite(&mut self, sprite: &Sprite) {
        // log(format!("Setting Sprite to {:#?}", sprite));

        let pt = match sprite {
            Sprite::Animated(pts) => &pts[(self.frame % (pts.len() as u32)) as usize],
            Sprite::Static(pt) => pt,
        };
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

    fn get_compass_sprites(&self, direction: String) -> Sprite {
        let c = self.sprites.cardinal.clone();
        let o = self.sprites.ordinal.clone();

        let map = HashMap::from([
            ("N", c.n),
            ("E", c.e),
            ("W", c.w),
            ("S", c.s),
            ("NE", o.ne),
            ("NW", o.nw),
            ("SE", o.se),
            ("SW", o.sw),
        ]);

        map.get(direction.as_str()).unwrap().clone()
    }
}

#[derive(Clone)]
struct Manzar {
    state: Rc<RefCell<ManzarState>>,
}

#[wasm_bindgen(start)]
pub unsafe fn start_manzar() -> Result<(), JsValue> {
    let window = web_sys::window().expect("no window exists.");
    let document = window.document().expect("no document exists.");
    let body = document.body().expect("document does not have a body.");
    let div = document
        .create_element("div")
        .unwrap()
        .dyn_into::<HtmlElement>()?;

    div.set_id("Manzar");

    const STYLES: [(&str, &str); 7] = [
        ("height", "32px"),
        ("width", "32px"),
        ("top", "16px"),
        ("left", "16px"),
        ("background-image", "url('kitty.gif')"),
        ("position", "fixed"),
        ("imageRendering", "pixelated"),
    ];

    for (prop, val) in STYLES.iter() {
        div.style().set_property(prop, val)?; // Set all styles defined in `STYLES`
    }
    body.append_child(&div)?;

    let cardinal = CardinalSprites {
        n: Sprite::Animated(vec![Point(-1, -2), Point(-1, -3)]),
        e: Sprite::Animated(vec![Point(-3, 0), Point(-3, -1)]),
        s: Sprite::Animated(vec![Point(-6, -3), Point(-7, -2)]),
        w: Sprite::Animated(vec![Point(-4, -2), Point(-4, -3)]),
    };

    let ordinal = OrdinalSprites {
        ne: Sprite::Animated(vec![Point(0, -2), Point(0, -3)]),
        se: Sprite::Animated(vec![Point(-5, -1), Point(-5, -2)]),
        sw: Sprite::Animated(vec![Point(-5, -3), Point(-6, -1)]),
        nw: Sprite::Animated(vec![Point(-1, 0), Point(-1, -1)]),
    };

    let scratch = ScratchSprites {
        cat: Sprite::Animated(vec![Point(-5, 0), Point(-6, 0), Point(-7, 0)]),
        cardinal: CardinalSprites {
            n: Sprite::Animated(vec![Point(0, 0), Point(0, -1)]),
            e: Sprite::Animated(vec![Point(-2, -2), Point(-2, -3)]),
            w: Sprite::Animated(vec![Point(-4, 0), Point(-4, -1)]),
            s: Sprite::Animated(vec![Point(-7, -1), Point(-6, -2)]),
        },
    };

    let sprites = ManzarSprites {
        idle: Sprite::Static(Point(-3, -3)),
        alert: Sprite::Static(Point(-7, -3)),
        tired: Sprite::Static(Point(-3, -2)),
        sleeping: Sprite::Animated(vec![Point(-2, 0), Point(-2, -1)]),
        cardinal,
        ordinal,
        scratch,
    };

    let manzar_state = ManzarState {
        element: div,
        sprites,
        mouse: Point(32, 32),
        cat: Point(32, 32),
        speed: 10,
        frame: 0,
        idle_frame: 0,
        idle_timeout: 50,
        idle_buffer: 0,
    };

    let manzar = Manzar {
        state: Rc::new(RefCell::new(manzar_state)),
    };

    // https://rustwasm.github.io/wasm-bindgen/examples/closures.html

    let mouse_clone = manzar.clone();

    let mouse_callback = Closure::<dyn FnMut(_)>::new(move |e: MouseEvent| {
        mouse_clone.state.borrow_mut().on_mouse_down(e);
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
