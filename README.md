# bpl-class-registration
Register a person for a free public class at the Brooklyn Public Library. Please be responsible with your requests.

This is meant purely as a learning exercise to learn Rust. The code seen herein could probably be optimized, but the point is to learn Rust.

I am using ChatGPT 4.0 to generate some code as well, and will ask it questions when I get stuck. I am a total beginner in Rust and am reading [Programming Rust](https://www.oreilly.com/library/view/programming-rust-2nd/9781492052586/).

You can only register for an event and cancel twice before your email cannot be used for registering for that event.

## Roadmap

* Get a list of all events, and optionally filter by location and keyword(s).
* Before registering for an event, check if the event requires registration.
  * If so, register for the event.
    * If registration is closed, raise an error.
    * If registration is set to open in the future, schedule registration for the date in question when registration opens.
* Use `PyO3` to wrap the methods herein for execution in Python.
