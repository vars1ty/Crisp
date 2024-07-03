# Crisp
A minimalist GTK Builder system written in Rust, with the goal of being simple and easy to understand.

## How does it work?
It works by implementing the [Rune](https://github.com/rune-rs/rune) scripting language and exporting a baggage of functions for it.

One example is for setting the title of your window: `GTK::set_window_title(Some("My custom window!"));`.

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
- Declaring multiple background loops that can access the UI, is working.
    - This uses unsafe code in certain places, so beware!
- Setting up listening commands and reading their output, is working.
    - The command is read in a background thread, in order to ensure it doesn't
    - block the UI in any way.

## Unsafe Code
Yes, Crisp uses unsafe code and it's not going to change.

Why? Because:
1. In some cases, it's needed.
2. Unsafe isn't bad by itself. It's bad if __you__ don't know how to handle it.
3. In some cases, it can remove the need for additional crates, or much more code.

If it's not crashing, causing memory issues, or anything alike; It's mostly accepted.

## Planned Features
- Adding support for a lot more widgets.
- Adding support for executing a command without getting the output.
- Listing the functions exposed via Rune, and document custom ones.
    - If a function is 99% just the standard GTK4 implementation, then no documentation will be provided,
    - as you can find it online.

## Contribution
Pull Requests are more than welcome, as long as they follow the current code-quality and style.

Avoid comments inside of functions unless required, name your variables properly, and avoid making the code complex or hard to read.
