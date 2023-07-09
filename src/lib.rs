use std::{cell::RefCell, collections::HashMap, rc::Rc};

use wasm_bindgen::prelude::*;
use web_sys::{HtmlElement, MouseEvent};

// The default CSS styles to apply to the div
const STYLES: [(&str, &str); 7] = [
    ("height", "32px"),
    ("width", "32px"),
    ("top", "16px"),
    ("left", "16px"),
    ("background-image", "url('kitty.gif')"),
    ("position", "fixed"),
    ("imageRendering", "pixelated"),
];

type Coord = Vec<(i32, i32)>;

/// Holds coordinates for the 4 cardinal directions
struct Cardinal {
    n: Coord,
    e: Coord,
    s: Coord,
    w: Coord,
}

/// Holds coordinates for the 4 ordinal directions
struct Ordinal {
    ne: Coord,
    se: Coord,
    sw: Coord,
    nw: Coord,
}

struct ScratchSprites {
    cat: Coord,
    cardinal: Cardinal,
}

/// This will hold state for the cat's sprites, and will handle switching and other logic
struct Sprites {
    idle: Coord,
    alert: Coord,
    tired: Coord,
    sleeping: Coord,
    cardinal: Cardinal,
    ordinal: Ordinal,
    scratch: ScratchSprites,
}

struct Point {
    x: i32,
    y: i32,
}

impl Point {
    fn update(&mut self, x: i32, y: i32) {
        self.x = x;
        self.y = y;
    }
}

/// Maintains the state and rendering of the actual manzar.
/// We need to maintain the HTML element (for changing it's location and styles),
/// the available sprites, and the coordinates of our element as well as the last mousedown event.
struct ManzarState {
    element: HtmlElement, // The HTML Element that will display the cat
    sprites: Sprites,
    mouse: Point,
    cat: Point,
    speed: i32,
    frame: i32,
}

impl ManzarState {
    fn on_mouse_down(&mut self, event: MouseEvent) {
        let x = event.client_x();
        let y = event.client_y();
        self.mouse.update(x, y);
    }

    fn get_compass_sprites(&self, direction: String) -> &Coord {
        let c = &self.sprites.cardinal;
        let o = &self.sprites.ordinal;

        let map = HashMap::from([
            ("N", &c.n),
            ("E", &c.e),
            ("W", &c.w),
            ("S", &c.s),
            ("NE", &o.ne),
            ("NW", &o.nw),
            ("SE", &o.se),
            ("SW", &o.sw),
        ]);

        map.get(direction.as_str()).unwrap() // TODO: Refactor to avoid cloning...
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

        self.cat.update(x, y);
    }

    fn set_sprite(&mut self, sprite: (i32, i32)) {
        // log(format!("Setting Sprite to {:#?}", sprite));
        self.element
            .style()
            .set_property(
                "background-position",
                &format!("{}px {}px", sprite.0 * 32, sprite.1 * 32),
            )
            .unwrap();
    }

    fn render(&mut self) {
        self.frame = self.frame + 1;

        let diff_x = self.cat.x - self.mouse.x;
        let diff_y = self.cat.y - self.mouse.y;
        let dist = ((diff_x.pow(2) + diff_y.pow(2)) as f32).abs().sqrt();

        let speed = self.speed as f32;

        if dist < speed {
            // It's close enough to the mouse.
            self.set_sprite(self.sprites.idle[0]);
            return ();
        }

        let cur_x = self.cat.x as f32;
        let cur_y = self.cat.y as f32;

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

        let set = self.get_compass_sprites(direction);
        let sprite = set[(self.frame % 2) as usize];
        self.set_sprite(sprite);
        self.move_to(x.round() as i32, y.round() as i32);
    }
}

/// This will hold the actual state in an Rc<RefCell<T>>
/// We need this because the event handler closures also need to own this
/// as well as mutate the state.
#[derive(Clone)]
struct Manzar {
    state: Rc<RefCell<ManzarState>>,
}

impl Manzar {
    fn new(element: HtmlElement, sprites: Sprites, speed: i32) -> Self {
        let state = ManzarState {
            element,
            sprites,
            speed,
            mouse: Point { x: 0, y: 0 },
            cat: Point { x: 32, y: 32 },
            frame: 0,
        };

        Self {
            state: Rc::new(RefCell::new(state)),
        }
    }
}

#[wasm_bindgen(start)]
pub unsafe fn start_manzar() -> Result<(), JsValue> {
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    let body = document.body().expect("document should have a body");

    let div = document
        .create_element("div")
        .unwrap()
        .dyn_into::<HtmlElement>()?;

    div.set_id("Manzar");

    for (prop, val) in STYLES.iter() {
        div.style().set_property(prop, val)?; // Set all styles defined in `STYLES`
    }
    body.append_child(&div)?;

    let sprites = Sprites {
        idle: vec![(-3, -3)],
        alert: vec![(-7, -3)],
        tired: vec![(-3, -2)],
        sleeping: vec![(-2, 0), (-2, -1)],
        cardinal: Cardinal {
            n: vec![(-1, -2), (-1, -3)],
            e: vec![(-3, -0), (-3, -1)],
            w: vec![(-4, -2), (-4, -3)],
            s: vec![(-6, -3), (-7, -2)],
        },
        ordinal: Ordinal {
            ne: vec![(0, -2), (0, -3)],
            se: vec![(-5, -1), (-5, -2)],
            sw: vec![(-5, -3), (-6, -1)],
            nw: vec![(-1, 0), (-1, -1)],
        },
        scratch: ScratchSprites {
            cat: vec![(-5, 0), (-6, 0), (-7, 0)],
            cardinal: Cardinal {
                n: vec![(0, 0), (0, -1)],
                e: vec![(-2, -2), (-2, -3)],
                w: vec![(-4, 0), (-4, -1)],
                s: vec![(-7, -1), (-6, -2)],
            },
        },
    };

    let manzar = Manzar::new(div, sprites, 10);

    let mouse_clone = manzar.clone();
    let mouse_update = Closure::<dyn FnMut(_)>::new(move |e: MouseEvent| {
        mouse_clone.state.borrow_mut().on_mouse_down(e);
    });

    let frame_clone = manzar.clone();
    let frame_update = Closure::<dyn FnMut()>::new(move || {
        frame_clone.state.borrow_mut().render();
    });

    document
        .add_event_listener_with_callback("mousedown", mouse_update.as_ref().unchecked_ref())?;

    window.set_interval_with_callback_and_timeout_and_arguments_0(
        frame_update.as_ref().unchecked_ref(),
        100,
    )?;

    // We need to forget these closures so they aren't dropped
    frame_update.forget();
    mouse_update.forget();

    Ok(())
}
