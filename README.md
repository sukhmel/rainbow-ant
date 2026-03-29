rainbow-ant
===========

[Langton's ant](https://en.wikipedia.org/wiki/Langton%27s_ant) with ~~interchangeable fields (e.g. text instead of 
picture),~~ more than one ant at once ~~and prolonged states~~ (can't remember what this is about, so not with that).

Written in ~~C++ using Qt~~ Rust with [iced](https://github.com/iced-rs/iced/tree/master).

Basic ant is implemented, some UI is there.

TODO:
- [x] Display instructions
- [ ] Edit instructions
- [ ] Highlight the ant in ant pane on hover
- [ ] Edit ant settings
- [ ] Visual direction picker
- [ ] Zoom in/out ([viewport](https://docs.rs/iced_glow/latest/iced_glow/struct.Viewport.html) 
    with [canvas](https://docs.rs/iced/latest/iced/widget/canvas/index.html)?)
- [ ] Support for hexagonal grid
  - [ ] display grid
  - [ ] move set
- [ ] Support for triangular grid
  - [ ] display grid
  - [ ] move set
- [ ] Save/load configuration
- [ ] Save/load state
- [ ] Undo/redo