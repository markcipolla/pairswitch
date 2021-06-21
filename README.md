# pairswitch

A tool to help wrangling git to show collaboration when pairing.
Pairing on code should be trivial to indicate that it was worked on by multiple people.

The idea is to make it incredibly easy to add git co-authors.

It should:
- [ ] allow a user to start and end a pairing session
- [ ] autocomplete the pair name (or future case allow via config switches)
- [ ] on commit, append the co-authored by (initial thoughts are to hook into the git hook cycle. Needs investigation)
- [ ] allow changing authors / co-authors for a group of commits, selected from a list
- [ ] connect to [pairswit.ch](https://pairswit.ch/) and log the commit / pairing event
- [ ] group pairing sessions into logical groups
- [ ] offer suggestions on whom else to pair with

It's a rust app using [Cursive](https://crates.io/crates/cursive).

- First off, [install rust](https://doc.rust-lang.org/cargo/getting-started/installation.html). 
- Start with [Cargo](https://doc.rust-lang.org/cargo/index.html), the package manager.
- `cargo run`

`q` quits the app.
