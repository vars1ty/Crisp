# Crisp
A minimalist GTK Builder system written in Rust, with the goal of being simple and easy to understand.

## How does it work?
It works by implementing the [Rune](https://github.com/rune-rs/rune) scripting language and exporting a baggage of functions for it.

One example is for setting the title of your window: `GTK::set_window_title("My custom window!");`.

All functions are high-level abstractions of real GTK4 functions, which is intentional as Crisp was made mainly to ease the process of creating widgets.

## Functions and Documentation
TBD

## Current Features
- Supported widgets as of now are: Labels and Boxes.
- Swapping widget focus is working.
- Modifying the window properties (like being resizable, default size, etc) is working.

## Planned Features
- Getting the output of a shell-command and returning it as a string.
- Adding support for a lot more widgets.
- Adding support for modifying widget properties.
- Adding support for making your window a layer-shell, or have it displayed as a regular window.
- Adding support for loading a custom stylesheet via the `STYLESHEET="relative_css_path"` environment variable.

## Contribution
Pull Requests are more than welcome, as long as they follow the current code-quality and style.

Avoid comments inside of functions unless required, name your variables properly, and avoid making the code complex or hard to read.
