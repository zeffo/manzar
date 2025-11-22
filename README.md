# manzar <img width="32" height="32" alt="_28" src="https://github.com/user-attachments/assets/92043ef7-1263-4b8c-a337-76056e410803" />

cat that wants to catch your mouse. <br>

You can view a live demo on [my website](https://zeffo.me).
Sprites by [aeshani](https://aeshanisingh.com/).

## Usage

1. Clone this repo.
1. Install wasm-pack: `cargo install wasm-pack`
1. Build: `wasm-pack build --target web`

## Performance

Likely the same or worse than the javascript version, since WASM cannot efficiently manipulate the DOM.
And the WASM filesize is also larger! So it's only good for bragging rights :3

## Credits

Inspired by [adryd325/oneko.js](https://github.com/adryd325/oneko.js)

