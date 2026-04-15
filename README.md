rainbow-ant
===========

[Langton's ant](https://en.wikipedia.org/wiki/Langton%27s_ant) with ~~interchangeable fields (e.g. text instead of 
picture),~~ more than one ant at once ~~and prolonged states~~ (can't remember what this is about, so not with that).

Written in ~~C++ using Qt~~ Rust with [iced](https://github.com/iced-rs/iced/tree/master).

Basic ant is implemented, some UI is there.

TODO:
- [x] Display instructions
- [x] Edit instructions
- [ ] Highlight the ant in ant pane on hover
- [x] Edit ant settings
  - [ ] Reorder ants
  - [ ] Remove ants using the ants pane
- [ ] Visual direction picker
- [x] Zoom in/out ([viewport](https://docs.rs/iced_glow/latest/iced_glow/struct.Viewport.html) 
    with [canvas](https://docs.rs/iced/latest/iced/widget/canvas/index.html)?)
- [x] Grid resize
- [ ] Support for hexagonal grid
  - [ ] display grid
  - [ ] move set
- [ ] Support for triangular grid
  - [ ] display grid
  - [ ] move set
- [ ] Save/load configuration
- [ ] Save/load state
- [ ] Undo/redo
- [x] Set a specific step number