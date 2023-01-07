# Domi

`domi`: an immediate-mode DOM library.

## Using domi

To see some examples of using this crate you can look in the [`examples/`](https://github.com/mattfbacon/domi/tree/main/examples) directory.
The examples include comments explaining significant features of the library as they are used.

## Why immediate mode?

Immediate mode allows avoiding the duplication of state that typically occurs in "retained-mode" GUIs.

The DOM itself is fundamentally retained-mode: when you add, remove, or update elements in the DOM, the browser remembers those changes and continues to render the elements to the user.

In the most primitive applications that interact with DOM, they simply make imperative calls to modify the DOM by adding, removing, or updating elements.
This represents the worst case in terms of duplicated state, because not only is the rendered view duplicated, the application could not even reconstruct the view from the app state!

In the next evolution of DOM interaction, libraries like React and Vue present a "component" abstraction that have state and can render when necessary.
Components are able to purely construct their DOM view from their state, and the library handles synchronizing the DOM with the app state.
This represents a great improvement over the previous generation!

However, there is still duplication of state.
For example, a text input element corresponding to a string in the app state are not one and the same string.
Rather, there are two strings that are intermittently kept in sync by, at best, a "two-way data binding", or, at worst, imperative getting and setting or asynchronous event handlers.

I like to think that immediate mode represents the next evolution beyond that type of library. It papers over all this duplication of state and allows you to write simple, direct code, like this:

```rust
// initial state:

let mut clicked = false;

// in the render function:

let mut button = ui.element("button", "button");

button.children().text(if clicked {
	"Thanks for clicking!"
} else {
	"Click me!"
});

if button.clicked() {
	clicked = true;
}
```

(Note: this library is evolving quickly and it's likely that the code will be become even cleaner as the library evolves.)

## A Note on IDs

One of the caveats of immediate mode is retaining the state that simply must be retained between renderings.
For example, events happen between render calls, so how can we know that a button added in a previous render call is the same button that was just added, in order to accurately deliver events?

There are multiple possible solutions here, but the approach that `domi` has chosen is to explicitly require the user to provide IDs.
An ID can be anything that is `Hash`able, but is usually a number or a string.

However, `domi` tries not to make IDs into a footgun. IDs must only be unique *between the direct children of an element*. For example, the following code is acceptable:

```rust
let mut div1 = ui.element("div1", "div");
div1.children().element("repeated", "div");

let mut div2 = ui.element("div2", "div");
div2.children().element("repeated", "div");
```

Even though the ID `"repeated"` is repeated globally, it's not repeated within the direct children of one element, so it's fine.

Event handling will become unreliable if there are ID collisions.

## License

AGPL-3.0-or-later
