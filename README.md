# Crisp
A minimalist GTK Builder system written in Rust, with the goal of being simple and easy to understand.

## How does it work?
It works by implementing the [Rune](https://github.com/rune-rs/rune) scripting language and exporting a baggage of functions for it.

One example is for setting the title of your window: `GTK::set_window_title("My custom window!");`.

All functions are high-level abstractions of real GTK4 functions, which is intentional as Crisp was made mainly to ease the process of creating widgets.

## Functions and Documentation
TBD

## State
- Supported widgets as of now are: Labels, Separators, Boxes and Buttons.
- Swapping widget focus is working.
- Detecting when the mouse enters/leave a widget is working.
- Detecting button presses is working.
- Registering for events in Rune is working.
- Getting the output of a command as a string is working.
- Modifying the window properties (like being resizable, default size, etc) is working.
- Making your window a layer-shell is working.

## Planned Features
- Adding support for a lot more widgets.
- Adding support for executing a command without getting the output.

## Contribution
Pull Requests are more than welcome, as long as they follow the current code-quality and style.

Avoid comments inside of functions unless required, name your variables properly, and avoid making the code complex or hard to read.
