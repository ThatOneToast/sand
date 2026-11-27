# The Sand Language

Sand packs can be written in Rust or in the **Sand language**: `.sand` files
with Rust's syntax that compile straight to a datapack. Both are entry points
into the same compiler. A `.sand` package lowers to Sand's portable program
model and is validated and emitted by the same backend as Rust and JSON
frontends, so equivalent packs produce byte-identical datapacks.

The difference is *when* code runs. In Rust, an `if` runs while the pack is
being built. In Sand source, every `if`, `for`, and assignment is gameplay that
runs inside Minecraft. You don't need macros or a Cargo build, and you don't
have to think in two stages.

```rust,ignore
//! Coins for everyone.
#![pack(namespace = "coins", minecraft = "26.2")]

player state Wallet {
    #[clamp(0..=9999)]
    coins: number,
}

#[on(first_join)]
fn welcome() {
    chat!("{gold}Welcome, {player}!{reset} You have {Wallet.coins} coins.");
    give_item(Item::Bread, 4);
}

#[on(death)]
fn tax() {
    Wallet.coins = Wallet.coins * 9 / 10;
    actionbar!("{red}You dropped some coins.");
}
```

## Getting started

```sh
sand lang new my-pack      # create my-pack/ with pack.sand and a module
cd my-pack
sand fmt                   # format the package (or --check for CI)
sand lang check            # report errors and warnings
sand lang build            # write the datapack to dist/<namespace>
sand lang run              # compile and launch a local Minecraft server
sand lang emit             # print the portable program JSON
```

`sand lang run` downloads the matching server, installs the pack in a test
world under `dist/server/`, and starts it (add `--offline` for easier local
joining). Language projects have no `sand.toml`; `sand run` also works from a
folder containing `pack.sand`. While the server runs, `sand lang build` updates
the test world, so `/reload` picks up your edits.

To use a pack elsewhere, copy `dist/<namespace>` into a world's `datapacks`
folder and run `/reload`.
A larger example lives in
[`examples/lang_encounters`](../../examples/lang_encounters/pack.sand).

## Editor setup (Zed)

The language server gives you diagnostics as you type, hover explanations for
every item, builtin, attribute, and keyword, plus completion, go to definition,
references, rename, formatting, naming and mutability quick fixes, an outline,
and inlay hints showing where each function
runs.

1. Install the `sand` CLI so editors can start `sand lsp`:
   `cargo install --path sand-cli` from the Sand repository.
2. Run `sand lang zed`. It writes a Zed extension and a tree-sitter grammar
   for highlighting to `~/.sand/zed` (or `--dir <path>`).
3. In Zed, run **`zed: install dev extension`** from the command palette and
   choose the printed `zed-sand` folder. Zed compiles it with Rust (rustup).

Open any `.sand` file to start the server. Sand finds the package by walking
up from the file to the nearest `pack.sand`. If `sand` is not on Zed's `PATH`,
point Zed at it in `settings.json`:

```json
{ "lsp": { "sand": { "binary": { "path": "/path/to/sand", "arguments": ["lsp"] } } } }
```

After upgrading Sand, run `sand lang zed` again and rebuild the extension from
Zed's extensions page. Other editors can use any LSP client: run `sand lsp` as
a stdio server for `*.sand` files.

## Naming

Sand follows one naming convention, the same one Rust uses. The compiler
warns when a name breaks it and suggests the fixed spelling.

| Style | Used for | Examples |
| --- | --- | --- |
| `PascalCase` | things you declare and refer to: `state`, `enum` and its variants, `flow`, `item`, `recipe`, `tag`; typed value sets and their values | `Wallet`, `Phase::Idle`, `RubySword`, `Item::Diamond`, `Rarity::Epic` |
| `snake_case` | things that run or hold values: functions, builtins, fields, variables, parameters, modules | `buy_heart`, `give_item`, `Wallet.coins`, `let price` |
| `SCREAMING_SNAKE_CASE` | constants | `MAX_HEALTH` |

Builtins are named `verb` or `verb_object`: `give_item`, `clear_item`,
`add_effect`, `play_sound`, `spawn_particle`, `add_tag`, `count_entities`.
Value sets are singular nouns: `Item`,
`Block`, `Entity`, `Sound`, `Particle`, `Attribute`, `Effect`, `Potion`,
`Rarity`, `Sort`. Attribute words such as `#[tick]` and `#[on(death)]` are
lowercase, as in Rust.

Resource identities are always snake_case: `item RubySword` in the root
module becomes `<namespace>:ruby_sword`.

## Packages and modules

A package starts at a root file, conventionally `pack.sand`, which declares the
pack with an inner attribute:

```rust,ignore
#![pack(namespace = "arena", minecraft = "26.2", description = "Arena minigame")]
```

`namespace` is required. `minecraft` defaults to the newest supported target,
and `description` defaults to the root file's `//!` docs.

Modules follow Rust's layout. `mod combat;` in the root loads `combat.sand` or
`combat/mod.sand`, and `mod hits;` inside `combat.sand` loads
`combat/hits.sand`. Inline `mod name { ... }` blocks work too. Paths and
privacy follow Rust: `use crate::Hero;`, `use super::util::{a, b as c};`, and
items are private to their module (and its children) unless marked `pub`.

Every generated identity derives from the namespace, module path, and item
name: `fn grant` in `rewards.sand` becomes `arena:rewards/grant`, and
`player state Hero` becomes the State `arena:hero`.

## Declarations

| Item | Meaning |
| --- | --- |
| `player state Name { field: number = 0, ... }` | One value per player |
| `global state Name { ... }` | One shared value for the server |
| `enum Phase { Idle, Active = 5 }` | Named integer phases, usable as field types |
| `const NAME: number = expr;` | Compile-time constant (`+ - * / %`, `as number`) |
| `struct Point { x: float, y: float }` | A named, typed value with fields |
| `impl Point { ... }` | Associated functions and methods for a struct or State |
| `fn name() { ... }` | A Minecraft function |
| `extern "ns:path" fn name();` | A function implemented by another pack |
| `flow Name on State.field { ... }` | A state machine over a field |
| `mod name;` / `use path;` | Modules and imports |
| `item Name { ... }` | A custom item |
| `recipe Name { ... }` | A crafting, cooking, or stonecutting recipe |
| `tag Name: Item { ... }` | A group of items, blocks (`Block`), or entities (`Entity`) |

Fields default to `0`, or to the first variant for enum fields.
`#[clamp(min..=max)]` clamps every write to an `number` field.

Function attributes:

- `#[tick]` / `#[load]` run the function every tick or on load, on the server.
- `#[on(event)]` runs the function as the player when a player event happens:
  `join` (every join, including the first), `first_join`, `death`, `kill`
  (any entity, or `kill = Entity::Zombie`), `kill_player`,
  `killed_by = Entity::Creeper`, `use_item = Item::CarrotOnAStick`,
  `craft_item = Item::Bread`, `pick_up_item = Item::Diamond`,
  `mine_block = Block::DiamondOre`, `jump`, `drop_item`, `sleep`,
  `take_damage`, and `deal_damage`.
- `#[tag("ns:name")]` adds the function to any function tag.
- `#[player]` requires a player executor explicitly. Without it, the context is
  inferred (see below).

## Struct values and methods

Structs group existing value types into named records. They work in locals,
function arguments and returns, event payloads, and State fields, including nested
records. Copies are independent; changing one copy does not change another.

```rust,ignore
struct Point { pub x: float, pub y: float }
impl Point {
    pub fn new(x: float, y: float) -> Self { return Self { x, y }; }
    pub fn east(mut self, distance: float) -> Self {
        self.x += distance;
        return self;
    }
}
player state Hero { position: Point, energy: number = 100 }
impl Hero {
    pub fn spend(&mut self, amount: number) { self.energy -= amount; }
    pub fn energy_left(&self) -> number { return self.energy; }
}
fn example() {
    let origin = Point::new(1.0, 2.0);
    let shifted = origin.east(3.0);
    for player in players() {
        Hero.position = shifted;
        Hero.spend(5);
    }
}
```

Use `Type::function(...)` for associated functions and `value.method(...)` for
methods. `Self` names the enclosing type. Field shorthand (`Point { x, y }`)
uses locals with matching names. Literal fields evaluate in their written order;
all fields must appear exactly once. Struct fields and methods follow module
privacy; add `pub` to expose them. Implementations may live in another module of
the same package, with the same visibility checks as ordinary code.

Struct methods can take `self` or `mut self` **by value**: return `Self` and
assign it back to keep a change. They can also take `&self` to read a value or
`&mut self` to update a mutable local or State field. Mutable receiver changes
commit when the call returns, including early returns and recursive calls; these
are scoped method receivers rather than general reference values. Mutable calls
on immutable variables or temporary expressions are errors. Use `==` and `!=` to compare records of the same type field by field.
Empty records and direct recursive layouts are rejected. A vector breaks layout
recursion, allowing records such as `Node { children: Vec<Node> }`. Each record has at
most 32 levels and 1,024 scalar leaves. Struct declarations have no field defaults
or clamps; an uninitialized record recursively uses zero or the first enum
variant, and string fields use empty text. State defaults can instead provide a complete record literal.

State methods use `&self` or `&mut self` to access persistent storage. These are
State receivers, not general Rust references. Player and entity receivers require
the corresponding executor, even if a method does not read a field. A read-only
receiver cannot write its State or call its mutable methods. Associated State
functions can use `Self.field` without a receiver. Existing clamps apply to final
field writes, and a whole-record State assignment copies every leaf. Arguments,
returns, recursive calls and event payloads use the same record representation.

The [Blinkstep example](../../examples/lang_methods/pack.sand) combines a private
position record with player State methods and a runtime cooldown.

## Fixed-size arrays

Arrays use one shared element type and a compile-time length:

```rust,ignore
let mut values: [number; 3] = [1, 2, 3];
values[1] += 5;
let zeros = [0; 8];
for value in values { chat!("{value}"); }
```

Arrays work in locals, State, records, function arguments/returns, custom events,
and recursive frames. Elements can themselves be arrays, structs, enums, or
entity references. Type inference works for nonempty literals and repeat forms;
an empty literal needs a type such as `[number; 0]`. Lengths can name number
constants. Arrays and their elements copy independently. `[expression; N]`
evaluates its expression once and copies the result, even when N is zero.

`for value in array` evaluates its source once and iterates a snapshot. Changing
the original array does not change later elements in that iteration. The loop
binding is an immutable value copy. Nested loops, `break`, `continue` and early
returns use the same rules as counting loops.

Indexes accept runtime whole numbers. A constant out-of-range index is an
error. An invalid runtime read returns the element's default; an invalid write
leaves the array unchanged. `get_or(index, fallback)` provides an explicit
fallback. Mutable methods on invalid indexed receivers skip their bodies and
return a default result. Indexes evaluate once, including for nested assignment
and mutable method receivers. Indexed assignment freezes its indexes before
evaluating the right-hand side.

| Method | Behavior |
| --- | --- |
| `array.len()` | Fixed length as a number |
| `array.is_empty()` | Whether length is zero |
| `array.contains(value)` | Value equality, including nested aggregate leaves |
| `array.get_or(index, fallback)` | Element or supplied fallback |
| `array.fill(value)` | Replace every element with an independent copy; needs a mutable receiver |

Arrays have at most 1,024 elements; complete aggregate layouts have at most
1,024 scalar leaves and 32 storage levels. Arrays may contain strings, vectors
and records with stored fields. See the
[quest-board example](../../examples/lang_collections/pack.sand).

## Growable vectors

`Vec<T>` is an owned growable sequence of one value type. Initialize it with an
array-shaped literal in a `Vec<T>` context, leave it empty with a typed binding,
or convert an existing array with `.to_vec()`.

```rust,ignore
let mut values: Vec<number> = [1, 2, 3];
values.push(4);
let mut copied = values;
copied[0] = 90; // values still starts with 1
for value in values { broadcast!(value.to_string()); }
let words = ["hello", "world"].to_vec();
```

Vectors work in State, nested records, function parameters/returns, and event
payloads. Elements may contain arrays, strings, other vectors or recursive
records. Every copy is independent. Vector indexing reads the element's default
when the index is outside its runtime length; invalid writes leave the vector
unchanged. A constant negative index is a diagnostic. Indexed mutable methods
freeze the index once, and skip the method body for an invalid receiver.

A `for value in collection` loop evaluates its source once and iterates a value
snapshot. Mutating the original collection in the loop preserves the original
iteration sequence. This applies to arrays, vectors and strings; string loop
bindings are single-character strings. Loop bindings are immutable values.

| Method | Result |
| --- | --- |
| `len()`, `is_empty()` | Current element count or emptiness |
| `get_or(index, fallback)` | Element or eagerly evaluated fallback |
| `contains(value)` | Deep typed value equality |
| `push(value)` | Append an independent copy; return whether it was added |
| `pop_or(fallback)` | Remove the last element, or return the fallback |
| `remove_or(index, fallback)` | Remove one indexed element, or return the fallback |
| `insert(index, value)` | Insert at an index up to the length; return whether inserted |
| `extend(values)` | Append independent copies and return the count added |
| `clear()` | Remove all elements |
| `truncate(length)` | Keep at most that many elements; a negative length clears the vector |

Mutation methods require a mutable local, writable record field, or writable
State receiver. As with borrowed struct methods, the receiver is copied in
before explicit arguments and committed when the method returns. Explicit
arguments evaluate once in authored order, including fallbacks and arguments to
an operation that cannot add an element. `==` and `!=` compare entire vectors.

Each vector supports up to 1,024 elements. A full vector rejects `push` and
`insert`; `extend` adds as many elements as fit. Expanded State defaults have a
1 MiB budget checked before repeat expansion; build larger values at runtime.
General borrowed slices remain future work.

See the [exploration-log example](../../examples/lang_vectors/pack.sand), which
stores a separate growable vector of coordinate records for each player.

### Maps and sets

Run `sand lang add collections` from your project directory to install an
editable `collections.sand` module. It works offline and preserves any existing
file at that path. Import its types as normal Sand definitions:

```rust,ignore
use collections::{Map, Set};

player state Journal {
    counts: Map<string, number>,
    discoveries: Set<string>
}

fn record(ore: string) {
    let total = Journal.counts.get_or(ore, 0) + 1;
    Journal.counts.insert(ore, total);
    Journal.discoveries.insert(ore);
}
```

Maps preserve insertion order and compare keys by value. `insert` replaces an
existing value in place and returns true. It returns false when adding a new
key would exceed capacity. Sets keep distinct values in insertion order.
Their `insert` returns false for a duplicate or a full set. Each collection
holds up to 1,024 elements and uses linear lookup. Large collections and complex
keys can consume much of Minecraft's command budget.

`Map` provides `get_or`, `contains_key`, `insert`, `remove`, `remove_or`,
`keys`, `values`, and `entries`. `Set` provides `contains`, `insert`,
`remove`, and `values`. Both have `new`, `len`, `is_empty`, and `clear`.
Returned entries and vectors are independent copies.

See the [ore journal example](../../examples/lang_maps/pack.sand).

### Tests and source maps

Mark a parameterless server-context function returning `bool` with `#[test]`.
Use `assert!` to fail at a specific source line. A test can also return
`false` directly.

```rust,ignore
fn doubled(value: number) -> number { return value * 2; }

#[test]
fn doubles_numbers() -> bool {
    assert!(doubled(7) == 14);
    return true;
}
```

Run `sand lang test --list` to discover tests or
`sand lang test --accept-eula` to run them in a disposable vanilla world.
Provide `--jar path/to/server.jar` for an existing server artifact. The runner
checks the server version and exits unsuccessfully if any test fails. Tests share
the disposable world during one suite, so initialize any State a test relies on.
Normal builds omit test entry points.

Run `sand lang source-map` to print JSON that links portable operation pointers,
emitted function paths, and generated command lines to Sand source. The file
table is relative to the package root directory. Source ranges use UTF-8 byte
offsets. The `commands` index uses pack-relative `.mcfunction` paths and
inclusive one-based line ranges, including generated branches and macros. This
command compiles in memory and reports ordinary Sand errors if emission fails.

## Types

| Type | Holds | Stored as |
| --- | --- | --- |
| `number` | whole numbers | a scoreboard score |
| `float` | decimals with three places, like `1.5` | a score holding `value * 1000` |
| `bool` | `true` or `false` | a score holding one or zero |
| an `enum` | one of its named alternatives, optionally with data | a tag score and slots for owned payloads |
| `string` | owned Unicode text | UUID-owned or global command storage |
| a `struct` | named fields of existing value types | score or storage slots per leaf |
| `[T; N]` | N independent values of one type | the same aggregate leaf layout |
| `Vec<T>` | up to 1,024 owned values of one type | UUID-owned or global command storage |

Scoreboards only hold whole numbers, so `float` is fixed point, the same
scheme as Rust's `Fixed` fields. Sand rescales arithmetic for you. Its range
is about +/-2,147,483, and multiplying two large fields can overflow; constant
factors are reduced first to make that less likely.

`number` and `float` do not mix implicitly (a whole-number literal is fine in
a `float` calculation); convert with `as float` and `as number`. `as number`
also converts enum phases and rounds a `float` down.

```rust,ignore
player state Hero {
    #[clamp(0.0..=2.0)]
    speed: float = 0.5,
    coins: number,
    rich: bool,
}

fn update() {
    Hero.speed = Hero.speed * 1.5;
    Hero.rich = Hero.coins >= 50;
    if Hero.rich && !is_tired() { Hero.coins -= 1; }
}
```

A `bool` works directly in conditions, and conditions also work as function
arguments: `accept(n > 0, !ready)`. Predicates such as `has_state(Mana)` can
be stored or passed as booleans. Text can only show whole numbers, so copy a
`float` first:
`let shown = Hero.speed as number;`, then write `{shown}`.

### Payload enums

A variant can carry one or more owned values. Use the variant name with
positional arguments to construct it, then bind the payload with `match`.

```rust,ignore
enum Delivery {
    Waiting,
    Delivered(number, string),
    Rejected(string),
}

global state Desk { latest: Delivery = Delivery::Waiting }

fn summary(value: Delivery) -> string {
    return match value {
        Delivery::Waiting => "Waiting",
        Delivery::Delivered(count, recipient) => recipient + ": " + count.to_string(),
        Delivery::Rejected(reason) => reason,
    };
}
```

Payloads work in State, collections, function arguments and returns, and
custom events. A match can bind each value, ignore it with `_`, or
destructure a record or fixed array. Guards run after the bindings are
available. An unguarded arm must cover each variant unless the match has
a catch-all. State defaults choose the first variant with default values
for its payload, or use an explicit constant constructor. Use explicit
discriminants when the tags need to stay stable across pack updates.
Enums with only unit variants retain their numeric casts and score layout.

Use type parameters to reuse a payload enum across value types. A constructor
can infer its type arguments from the payload or take them from a surrounding
type annotation. Supply them explicitly when needed, as in
`Option::<number>::None`. Patterns can omit the arguments because the matched
value supplies the concrete enum type.

Run `sand lang add outcomes` in the pack directory to install the bundled
`Option<T>` and `Result<T, E>` declarations as an ordinary module.

```rust,ignore
use outcomes::{Option, Result};

fn require_sample(sample: Option<number>) -> Result<number, string> {
    return match sample {
        Option::Some(units) => Result::Ok(units),
        Option::None => Result::Err("No sample available"),
    };
}
```

## Functions and variables

Functions take typed parameters and may return a value, just like Rust:

```rust,ignore
/// The price of one extra heart, rising with each purchase.
fn heart_price(owned: number) -> number {
    return 50 + owned * 25;
}

#[on(use_item = Item::CarrotOnAStick)]
fn buy_heart() {
    let health = Player.max_health;                   // 20.0 = ten hearts
    if health >= 40.0 {
        actionbar!("{gold}You already have the maximum health.");
        return;                                       // early return
    }
    let price = heart_price(((health - 20.0) / 2.0) as number);
    if Wallet.coins < price {
        chat!("{red}You need {price} coins.");
        return;
    }
    Wallet.coins -= price;
    Player.max_health = health + 2.0;
}
```

- `let name = value;` declares a variable; `let mut` allows reassignment.
  Variables are block-scoped and can shadow outer ones.
- Parameters are immutable by default; write `mut amount: number` to allow
  reassignment within the function. Arguments are passed by value.
- `return;` ends a function early; `return value;` returns from a function
  declared with `-> number`. Every path must return a value in such functions.
- Functions can call each other and themselves, directly or through other
  functions. Recursion works like in Rust: each call has its own variables.

```rust,ignore
fn factorial(n: number) -> number {
    if n <= 1 { return 1; }
    return n * factorial(n - 1);
}

fn is_even(n: number) -> bool {
    if n == 0 { return true; }
    return is_odd(n - 1);
}
fn is_odd(n: number) -> bool {
    if n == 0 { return false; }
    return is_even(n - 1);
}
```

  A function returning `bool` can be used directly as a condition:
  `if is_even(Hero.gold) { ... }`. Minecraft limits how many commands one
  tick can run (65,536 by default), which bounds how deep recursion can go.
- Entry points (`#[tick]`, `#[load]`, `#[on(...)]`, `schedule`, flow hooks)
  run without arguments, so they cannot take parameters.

## Statements

```rust,ignore
Hero.energy = 10;                       // set a State field
Hero.energy += 5;                       // also -=, *=, /=, %=
let total = Hero.gold * 3 + bonus(2);   // any integer expression
Dice.roll = random(1..=6);              // random integer
grant();                                // call (module paths work: rewards::grant())
for player in players() { ... }         // run the body as each online player
for i in 0..10 { ... }                  // count 0 to 9; `0..=10` includes 10
while n != 1 && steps < 100 { ... }     // loop while any condition holds
loop { if done() { break; } }           // loop until `break` or `return`
emit Payday(25);                        // run every handler of a custom event
if Hero.energy >= 30 && Hero.gold > Rival.gold { ... } else { ... }
```

Arithmetic follows normal precedence with parentheses, using Minecraft's 32-bit
scoreboard maths (division rounds down). A clamped field is clamped once,
after the whole value is computed.

Conditions compare any integer expressions (`== != < <= > >=`) or test `in` a
range (`a..b`, `a..=b`, `a..`, `..=b`). Combine them with `&&`, `||`, `!`, and
parentheses. Every comparison is evaluated at most once, and exactly one branch
runs.

### Loops

`while`, `loop`, and counting `for` repeat their body; `break;` leaves the
innermost one and `continue;` skips to its next pass:

```rust,ignore
fn first_square_over(limit: number) -> number {
    for i in 0..1000 {
        if i * i > limit { return i; }
    }
    return -1;
}

fn collatz_steps(start: number) -> number {
    let mut n = start;
    let mut steps = 0;
    while n != 1 {
        if n % 2 == 0 { n = n / 2; } else { n = n * 3 + 1; }
        steps += 1;
    }
    return steps;
}
```

A counting `for` computes both bounds once, from left to right, before the
first pass, and its variable cannot be assigned. Parenthesized ranges support
`.rev()` and `.step_by(n)`, where `n` is a positive constant whole number:

```rust,ignore
for i in (0..10).step_by(2) { ... }       // 0, 2, 4, 6, 8
for i in (0..10).rev() { ... }            // 9 down to 0
for i in (0..=10).rev().step_by(3) { ... } // 10, 7, 4, 1
```

Empty ranges stay empty when reversed. Counting stops at the scoreboard's
integer boundaries without wrapping.

 `while` accepts any condition; one plain
comparison is checked directly, and anything else is computed into a flag
before each pass.

Inside `for player in players()`, `continue;` skips to the next player;
`break;` is an error there, because every player runs once. As in
Rust, the loop changes who runs the body but not where: `offset(..)`, sounds,
particles, and summons stay at the caller's position (the world spawn in
`#[tick]`). `#[on(...)]` handlers and `for name in entities(..)` bodies do run
positioned at their player or entity.

Conditions can also ask about the world and the executor:

| Condition | True when |
| --- | --- |
| `has_state(Mana)` | the executor has this `entity state` attached |
| `any_entities(type = .., tag = .., distance = ..)` | at least one entity matches |
| `block_is(Block::Water, offset(0, -1, 0))` | the position holds this block or `Block` tag |
| `holding(Item::Diamond)`, `holding(Wand, offhand = true)`, `holding(Helmet, slot = EquipmentSlot::Head)` | the executor has this item in the selected slot |
| `is_sneaking()`, `is_sprinting()`, `is_swimming()`, `is_on_fire()` | the executor is doing it |
| `timer_running(Ability.dash)`, `timer_done(Ability.dash)` | a `timer` field has time left, or none |


`equip(item, slot, count = 1)` replaces one equipment stack on the executor.
`clear_equipment(slot)` discards that stack. Use `EquipmentSlot::Head`, `Chest`,
`Legs`, `Feet`, `Mainhand`, or `Offhand` for writes and checks. A declared
custom item carries its components when equipped and matches by identity in
`holding`. Choose either `slot` or `offhand` when checking equipment. A slot
must exist on the executing entity, and Minecraft applies its native stack
limits.

### Generics

Functions and structs can declare type parameters. Calls infer them from value
arguments and an expected result type. Use `::<T>` when a type has no value from
which to infer it:

```rust,ignore
fn duplicate<T>(value: T) -> [T; 2] { return [value, value]; }
struct Envelope<T> { value: T }
impl<T> Envelope<T> {
    fn new(value: T) -> Self { return Self { value }; }
    fn read(&self) -> T { return self.value; }
    fn choose<U>(&self, first: U, second: U) -> U { return first; }
}
let copies = duplicate("hello");
let envelope = Envelope::<string>::new("saved");
let wrapped: Envelope<Vec<number>> = Envelope { value: [1, 2] };
```

Type annotations use `Envelope<string>`; record literals may infer their type
arguments from fields or the expected type, or spell `Envelope::<string> { ... }`.
Generic records work in State, defaults, events, arrays, vectors, parameters and
returns. Recursive records can use vectors, such as
`Node<T> { value: T, children: Vec<Node<T>> }`.

Generic impl blocks use the same ownership, receiver and privacy rules as other
impls. Methods can introduce their own parameters and accept explicit arguments
as `value.method::<T>(...)`. Concrete call slots and execution contexts belong to
each specialization; copies and recursive calls preserve the same isolation as
ordinary functions. Navigation and rename follow the authored source across
specializations. Signature help uses concrete parameter types, including nested
generic arguments, and completed call results can supply receiver completions.

Sand specializes generic bodies when concrete types are used. It checks each
specialization, rather than proving an unused generic body valid for all possible
types. Trait bounds check the concrete types used in each specialization. A package supports
at most 256 function specializations and 256 record specializations; expanding
record types stop at 64 nested specializations. The existing finite record-layout
limits still apply. These checks prevent recursively growing compile-time types.
See the [generic mining-history example](../../examples/lang_generics/pack.sand).

### Function values and closures

Use `fn(number) -> string` for a callable that takes a number and returns text.
Omit the arrow for a function that produces no value. An ordinary named function
can be stored by writing its name without parentheses. Bind a stored callback to
a local to call it.

```rust,ignore
fn with_prefix(prefix: string) -> fn(number) -> string {
    return move |points: number| -> string {
        return prefix + points.to_string();
    };
}

let label = with_prefix("Points: ");
let message = label(7);
```

A move closure owns a snapshot of its captured locals. Captures are immutable
inside the closure and can contain records, collections, or other callbacks.
Function values can be passed to generic functions and saved in State. Dispatch
only invokes compile-checked functions with the same signature. A missing or
unrecognized callable skips the call and returns the value type's default.

Callable targets currently run without a player or entity executor. Named
targets must be concrete non-generic functions or associated functions. Define a
closure in a generic body to retain a specialized value. Closures survive reload
of the same compiled pack. Recompiling a changed source can invalidate stored
closure identities, so initialize persistent callbacks as part of pack setup.
See the [reward notes example](../../examples/lang_callbacks/pack.sand).

#### Executor-scoped callbacks

Use `player fn(...)` when a callback can read player State or use player
actions. Use `entity fn(...)` for callbacks callable as any entity, including
players. A plain `fn(...)` can be invoked from the server and accepts targets
that need neither a player nor an entity.

```rust,ignore
player state Journal { score: number, reward: player fn(number) }

fn make_reward(bonus: number) -> player fn(number) {
    return move |points: number| { Journal.score += points + bonus; };
}

#[player]
fn collect() {
    Journal.reward = make_reward(2);
    let reward = Journal.reward;
    reward(5);
}
```

A function name or closure literal takes its executor contract from the expected
type. Without an annotation or expected callable type, its type defaults to
plain `fn(...)`. A server-safe named function can be assigned directly to an
entity or player callback. An entity function can also be assigned directly to
a player callback. Stored callable values require the same signature, including
its executor contract. Invoking a callback uses the caller's current executor.
Creating or storing one captures only its ordinary local values.

### Traits

A trait declares method signatures and can provide default bodies. Use
`impl Trait for Type` to supply required methods or override defaults.
Use `T: Trait` to require that contract in a generic function. Combine
requirements with `+`.

```rust,ignore
trait Named {
    fn name(&self) -> string;
    fn heading(&self) -> string { return "Quest: " + self.name(); }
}

struct Quest { title: string }
impl Named for Quest {
    fn name(&self) -> string { return self.title; }
}

fn announce<T: Named>(quest: T) {
    chat!(quest.heading());
}
```

Implementations may use borrowed receivers and refer to the implementing type as
`Self`. Generic records can implement a trait as in
`impl<T> Named for Container<T>`. Every override must match the declared
receiver, parameter types, and return type. Sand checks each inherited body
with the implementing type as `Self` and resolves module names where the trait
is declared. A call to `self.name()` in a default body selects the implementing
type's method. Generic calls select a concrete implementation at compile time.
Trait objects, associated types, and fully qualified calls to overlapping
methods are still outside the language.

### Match

`match` evaluates its value once and runs exactly one arm. Arms have their own
local scope and can return, break, or continue their enclosing loop:

```rust,ignore
fn price(phase: Phase) -> number {
    match phase {
        Phase::Idle => { return 10; }
        Phase::Fighting | Phase::Resting => { return 20; }
    }
}
```

Use `|` to join constant or unit-variant patterns. Give each payload
variant pattern its own arm.
Cover every enum variant, both booleans, or add a catch-all arm such as
`_ => { ... }` or `let name => { ... }`. Duplicate unguarded patterns, missing
cases, and arms following an unguarded catch-all are errors. Dispatch depth
does not grow with the number of arms.

A value match evaluates only the selected arm's expression. It works in local
initializers, assignments, returns, arguments, and larger expressions. All arms
must produce the same type, including owned strings, records, arrays and vectors:

```rust,ignore
let label = match phase {
    Phase::Idle => "waiting",
    Phase::Fighting | Phase::Resting => "busy",
};
let rewards: Vec<number> = match won { true => [5, 10], false => [] };
```

Value arms are comma-separated expressions. Statement arms use scoped blocks.

Use `if` after a pattern to add a guard in either form of `match`. Sand tests
the guard only after the pattern matches. If it evaluates to false, matching
continues with the next arm. Only unguarded arms contribute to exhaustiveness.

Write `let` in an arm to bind the matched value or destructure a record or
fixed array. The bindings are owned copies visible in that arm's guard and body
or expression. An unguarded binding pattern covers every remaining value of the
matched type.

```rust,ignore
let amount = match reward {
    let Reward { amounts: [iron, gold], .. } if gold > 0 => iron + gold,
    let Reward { amounts: [iron, _], .. } => iron,
};
```


```rust,ignore
let rank = match mined {
    10 | 20 => Rank::Beginner,
    _ if mined <= 40 => Rank::Explorer,
    _ => Rank::Veteran,
};
```

### Block expressions

A block can produce a value after executing scoped statements. Leave off the
semicolon on the last expression. Locals declared inside the block are visible
through its final expression. Use a block anywhere an expression fits,
including a function argument or a value match arm.

```rust,ignore
let label = match active {
    true => {
        let score = Progress.mined.to_string();
        "Mined " + score + " blocks"
    },
    false => "Waiting",
};
```

Use the final expression to yield a result. Loops inside the block can use
`break` and `continue` locally. Function `return`, suspension and jumps to
surrounding loops are unavailable inside value blocks.


### Destructuring

`let` can unpack a record or a fixed array into independent owned locals. Nested
patterns, renamed fields, `_` and `..` for omitted record fields are supported.
Use `mut` on each binding that will change:

```rust,ignore
let Point { x, y: mut height } = point;
let [first, _, mut last] = [1, 2, 3];
height += x;
last += first;
```

The initializer runs once before any binding becomes visible. Array patterns
must have the exact fixed length; vectors require explicit checked element access.
Record fields follow privacy rules. Missing fields without `..`, duplicate fields,
duplicate bindings, and incompatible types are errors. Renaming a shorthand field
or local in the editor expands the shorthand to preserve the other symbol.
See the [mining-streak example](../../examples/lang_patterns/pack.sand).

### Casts

`value as T` converts between `number`, `float`, `bool`, and enums. The same
rules apply to constants and to values computed while the game runs:

| From \\ to | `number` | `float` | `bool` | enum |
| --- | --- | --- | --- | --- |
| `number` | same | `3` → `3.0` | `x != 0` | the variant with that value |
| `float` | rounds down: `2.7` → `2`, `-1.5` → `-2` | same | `x != 0.0` | error |
| `bool` | `0` or `1` | `0.0` or `1.0` | same | error |
| enum | its value | its value | error | the same enum only |

As in Rust, `-x as T` casts the negated value. Impossible casts say what to
write instead, and a constant cast to an enum must name an existing variant.

## Players and entities

`Player` is the executing player, read and written like a State. Its fields
are entity data (read-only) and every attribute (read and write):

```rust,ignore
fn tune() {
    if Player.health < 6.0 && Player.food > 10 {
        Player.movement_speed *= 1.2;          // attributes can be assigned
    }
    Player.max_health = 40.0;
    Player.step_height = 1.0;
}
```

Inside `for zombie in entities(..)` and `for player in players()`, the loop
variable is a handle to the entity running the body: `zombie.health`,
`zombie.scale = 2.0;`. Only the innermost loop's entity can be used.

| Property | Type | Notes |
| --- | --- | --- |
| `health`, `absorption` | `float` | half-hearts |
| `air` | `number` | ticks of breath; 300 is full |
| `fire` | `number` | ticks left burning |
| `fall_distance` | `float` | blocks |
| `on_ground` | `bool` | |
| `x`, `y`, `z` | `float` | position |
| `food`, `xp_level`, `xp_total`, `selected_slot` | `number` | players only |
| `saturation`, `xp_progress` | `float` | players only |
| any attribute: `max_health`, `movement_speed`, `step_height`, `scale`, `gravity`, ... | `float` | base value; assignable |

Attribute names are the attribute's identity without `minecraft:`, so
`Attribute::StepHeight` is `step_height`. Assigning read-only data is an error
that names the builtin to use instead, such as `damage(..)` or `teleport(..)`.
The editor completes properties after `Player.` or a loop variable and shows
each one's type, whether it can be assigned, and the command it becomes.

## Text

Text macros take a string with `{...}` interpolation:

| Macro | Shows |
| --- | --- |
| `chat!("...")` | a chat message to the executing player |
| `broadcast!("...")` | a chat message to every player |
| `actionbar!("...")` | text above the executing player's hotbar |
| `title!("...")` / `subtitle!("...")` | a big title (set the subtitle first) |

Inside the text, `{variable}` or `{State.field}` shows a number or string and `{player}`
shows the player's name. Style names (`{gold}`, `{red}`, `{bold}`, `{italic}`,
`{underlined}`, all sixteen colours, and `{reset}`) style the text that follows.
Write `{{` and `}}` for literal braces.

```rust,ignore
title!("{yellow}{bold}Round {World.round}");
chat!("{gray}Coins: {gold}{Wallet.coins}");
```

`Text::translate("key", ...)` sends Minecraft a client translation key and one
positional text argument per placeholder. Each argument can be a format string,
a runtime `string` in a message, or another translation. Item names, lore, and
boss bar titles can use translations with fixed arguments.

```rust,ignore
fn greeting(label: string) {
    chat!(Text::translate("chat.type.text", "{player}", label));
}
```

Built-in keys come from Minecraft’s language files. Keys supplied by a resource
pack resolve for players who have that pack. Sand stores runtime strings as
plain text when it passes them to placeholder arguments.

### Interactive chat

`Text::hover(label, tooltip)` adds a text tooltip. `Text::link(label, url)`
opens an HTTP(S) link, `Text::suggest(label, command)` fills the chat input,
and `Text::copy(label, value)` copies a quoted string to the clipboard.
These components work in `chat!`, `broadcast!`, and written-book pages. Compose them with
translations and formatted text.

```rust,ignore
chat!(Text::hover(
    Text::suggest("{aqua}Next round", "/trigger next"),
    "Click to put the command in chat",
));
```

### Runtime strings

`string` is an owned value supported in locals, constants, State, record fields,
arrays, function parameters/returns and event payloads. Assignment makes an
independent copy; borrowed struct methods and recursive calls preserve strings
through the same value and frame model as other fields.

```rust,ignore
const PREFIX: string = "Hello, ";
player state Notes { title: string = "Journal" }
let mut message = PREFIX + "world 🌍";
message += "!";
let count = 42;
let line = "Collected " + count.to_string() + " items";
chat!(line);
chat!("{gold}{Notes.title}: {message}");
```

Literal macro arguments retain format interpolation. Runtime string arguments
are shown as plain text, so braces and text such as `$(uuid)` remain literal.
`{local.field}` and nested State/record field paths may interpolate strings.

| Operation | Result |
| --- | --- |
| `left + right`, `text += suffix` | Concatenate text |
| `left == right`, `left != right` | Compare the full character sequence |
| `text.len()`, `text.is_empty()` | Character count or emptiness |
| `text[index]` | One character as a string; invalid runtime indices return empty text |
| `text.slice(start, end)` | Independent text in a clamped half-open range; reversed ranges are empty |
| `text.contains(value)` | Substring membership; empty text is always contained |
| `text.starts_with(value)`, `text.ends_with(value)` | Prefix or suffix membership |
| `value.to_string()` | Signed decimal number, three-place float, boolean or enum variant name |

Indices count Unicode scalar values, so a supplementary character counts once;
combining characters remain separate. Stored strings use typed NBT character
lists, and each State owner uses the canonical UUID lifecycle. These are plain
text values, not rich-text component values or arbitrary imported NBT strings.
String comparisons support equality rather than ordering. String constants may
combine literals and other string constants. Numeric casts do not parse text.

Escapes include `\\`, `\"`, `\n`, `\r`, `\t`, `\0`, `\b`, `\f` and
`\u{hex}` for a Unicode scalar value. Malformed escapes report a source error.
`to_string` preserves the minimum signed integer without overflowing. A float
always shows three decimal places, including trailing zeros; an invalid enum
discriminant formats as its number. Text literals and expanded string constants
support up to 65,536 UTF-8 bytes, with at most 64 expansion levels. Runtime text
operations run as Minecraft commands,
so large values and searches remain subject to command execution limits.

See the [saved journal example](../../examples/lang_runtime_text/pack.sand).

## Vanilla values

Items, blocks, entity types, sounds, particles, attributes, status effects,
and potions are typed, exactly as in Rust:

```rust,ignore
give_item(Item::Diamond, 3);
add_effect(Effect::Speed, seconds = 10);
play_sound(Sound::EntityPlayerLevelup);
spawn_particle(Particle::Heart, count = 5);
```

| Sand | Rust |
| --- | --- |
| `Item::...`, `Block::...`, `Entity::...` | `sand::vanilla::Item` and friends |
| `Sound::...`, `Particle::...`, `Attribute::...` | `sand::vanilla::SoundEvent`, ... |
| `Effect::...`, `Potion::...` | `sand::registry::EffectId`, `PotionId` |

The editor lists every value after `Item::` (or straight away inside a
builtin's argument), hovers show the Minecraft identity, and misspellings
get a "did you mean". The values come from the same generated registries as
Rust, so they always match the target Minecraft version. A namespaced string
such as `"mymod:ruby"` still works for modded or custom content; writing a
vanilla identity as a string gets a warning pointing at the typed value.

## Builtins

| Builtin | Does |
| --- | --- |
| `play_sound(Sound::..., volume = 1.0, pitch = 1.0)` | play a sound to the player |
| `spawn_particle(Particle::..., count = 1, spread = 0.0, speed = 0.0)` | spawn particles above the executor |
| `add_effect(Effect::..., seconds = 10, level = 2, hide_particles = true)` | give a status effect (omit `seconds` for infinite) |
| `clear_effect()` / `clear_effect(Effect::...)` | remove effects |
| `give_item(Item::..., 3)` | give items |
| `clear_item(Item::..., 2)` / `clear_inventory()` | remove items |
| `schedule(function, seconds = 5)` | run a server function later (`ticks = n` also works) |
| `raw!("command")` | run one command exactly as written (escape hatch) |

Hover any builtin in the editor to see its arguments and the Minecraft command
it becomes.

### Particle effects

`spawn_particle` places particles one block above the current execution
position. The count, spread and speed accept runtime values. Count must be a
`number` from 0 to 100,000. Spread and speed accept numbers or floats from 0 to
1,000. Sand evaluates arguments in source order and skips the command when a
runtime value is out of range.

```rust,ignore
let sparks = Hero.energy / 10;
spawn_particle(Particle::HappyVillager, count = sparks, spread = 0.25);
```

## Entities

`for name in entities(..)` runs its body as each matching entity, positioned
at it. Filters are named arguments:

```rust,ignore
#[tick]
fn sweep() {
    for player in players() {
        for zombie in entities(type = Entity::Zombie, distance = ..=8, limit = 3, sort = Sort::Nearest) {
            damage(4.0);
            add_tag("hit");
        }
    }
    World.zombies = count_entities(type = Entity::Zombie);
}
```

| Builtin | Does |
| --- | --- |
| `entities(type, tag, distance, limit, sort)` | `for` source: run as each match |
| `count_entities(...)` | how many entities match (an expression) |
| `kill()` | kill the executing entity |
| `add_tag("x")` / `remove_tag("x")` | add or remove a tag |
| `damage(amount, by = reference)` | damage the executor, optionally credited to a retained attacker |
| `summon(Entity::..., tag = "x")` | summon at the executor's position |
| `set_velocity(x, y, z)` | set the executing non-player entity’s velocity in blocks per tick |

`any_entities`, `count_entities`, `entities`, and `teleport_to` share their
filters. Use `with = Threat` to require attached entity State. Use
`score = Threat.level` with `score_range = 3..=8` to check a `number` field
on each candidate. Exact numbers, open bounds, and exclusive upper bounds
also work. Missing scores exclude the candidate.

Use `nbt = Nbt { Silent: true }` to match a subset of the entity’s native data.
Compounds accept nested `Nbt { ... }`, lists, literal strings, and constant
numbers and booleans. Integers use NBT ints, decimals use doubles, and bools
use bytes. NBT numeric tag types must match the native field. The compiler
applies all filters before sorting and limiting the selection.

Read `entity.velocity_x`, `entity.velocity_y`, and `entity.velocity_z` through
the entity handle. `set_velocity` writes all three axes together. Every axis
must be within -10 to 10 blocks per tick. A runtime value outside that range
skips the write.

An entity may not be a player, so player State and player builtins are errors
inside an `entities` loop. A function that uses `kill()`, `tag`, or `damage`
outside a loop **runs as an entity**, the third execution context: callable
from player functions and entity loops, but not from `#[tick]` directly.

### Entity State

`entity state` stores values on individual entities, using the same model as
Rust's `#[state(scope = entity)]`:

```rust,ignore
entity state Mana {
    #[clamp(0..=100)]
    current: number = 10,
}

#[tick]
fn tick() {
    for mob in entities(type = Entity::Witch) {
        if has_state(Mana) {
            Mana.current += 1;
        } else {
            attach_state(Mana);
        }
    }
    for carrier in entities(with = Mana) { damage(1.0); }
}
```

`attach_state` gives fields without a value their defaults, and keeps existing
values. `detach_state` clears them. Value reads from an absent entity-State field
use the type's default (zero, empty text/vector, or the first enum phase), so a
getter cannot retain a previous entity's value. Reading does not attach the State
or write its declared field default. A function that touches entity State
outside a loop runs as an entity. Flows only run over player or global State.

### Retained entity references

`EntityRef` stores a persistent, pack-owned entity identity in a local, State
field, record, parameter, return value, or custom event payload. Capture the
executing entity with `entity_ref()` and resolve it explicitly:

```rust,ignore
entity state Mark { owner: EntityRef }
fn mark() {
    let owner = entity_ref();
    for target in entities(type = Entity::Zombie, distance = ..=8) {
        attach_state(Mark);
        Mark.owner = owner;
    }
}
fn hurt_marked() {
    for target in entities(with = Mark) {
        damage(4.0, by = Mark.owner);
        for player in resolve_player(Mark.owner) { chat!("Your mark hit!"); }
    }
}
```

Pass `relation = Relation::Vehicle` to retain the executor's current vehicle.
`Relation::Owner`, `Leasher`, `Target`, `Controller`, `Attacker`, and `Origin`
also resolve to at most one entity. A missing relationship returns an empty
reference. Once retained, the reference identifies that entity even if the
relationship changes.

Use `mount(vehicle)` to ride a retained non-player entity in the current
dimension. An empty or unavailable vehicle leaves the executor where it is.
Use `dismount()` to get off the current vehicle.
See the [minecart shuttle example](../../examples/lang_vehicles/pack.sand).

`resolve_player(reference)` runs a `for` body as an online player, at that player,
including across dimensions. `resolve_entity(reference)` also resolves loaded
non-player entities in the current dimension; its body has an entity context.
Missing, dead, unloaded, and empty references skip the body. Default references
are empty. IDs survive reload, are isolated by pack, never wrap, and are never
reassigned; exhaustion produces an empty reference. References support copying
and equality, not arithmetic or numeric casts.

`damage(amount, by = reference)` attributes generic damage to the retained
attacker, including vanilla kill credit. Missing attackers skip the damage.
This preserves ownership instead of silently making an attack unattributed.
The [marked-combat example](../../examples/lang_reference_combat/pack.sand)
combines these references with delayed entity-State timers.

### Movement

`teleport(x, y, z)` moves the executor to coordinates computed at runtime or
written as constants,
`teleport_by(dx, dy, dz)` moves it relatively, and `teleport_to(type = ...)`
moves it to the nearest entity matching the same filters as `entities(..)`.

## Custom events

An event is a name your own functions can react to. Declare it, optionally with
a typed payload; mark handlers with `#[on(...)]`; run them all with `emit`:

```rust,ignore
event Payday(amount: number);

#[on(Payday)]
fn deposit(amount: number) { Wallet.coins += amount; }

#[on(Payday)]
fn celebrate() { chat!("{gold}Paid!"); }   // ignores the payload

fn wages() { emit Payday(25); }
```

Handlers run in declaration order, as whoever emitted the event, and take
either exactly the event's parameters or none. `emit Name;` emits an event
without a payload. Arguments are evaluated once, and a handler may emit the
event again (it is recursion, so it needs a stopping condition). If a handler needs
a player, emitting the event needs one too, and the editor says so. Any module
can handle a `pub event`.

## Timers and cooldowns

A `timer` field counts down by one every tick until it reaches zero, using the
same State lifecycle as Rust's `Timer` and `Cooldown` fields:

```rust,ignore
player state Ability { dash: timer }

fn dash() {
    if timer_running(Ability.dash) {
        actionbar!("{red}{Ability.dash} ticks left");
        return;
    }
    start_timer(Ability.dash, seconds = 3);
    teleport_by(0.0, 0.5, 0.0);
}
```

`start_timer` takes exactly one of `ticks` or `seconds`, including runtime
`number` values. Durations must be positive: ticks allow up to 2,147,483,647,
seconds up to 107,374,182. Invalid constants are errors; an invalid runtime
duration leaves the timer unchanged. `stop_timer` ends one early. A timer reads as the number of ticks left, so comparisons and `{Ability.dash}`
text work too.

### Duration and position libraries

Run `sand lang add time` to use the bundled `Duration` type. Read a value
with `.as_ticks()` when passing it to `wait` or `start_timer`. Negative tick
values and seconds outside the representable range become zero.
`saturating_add` caps the total at the largest scoreboard integer.

Run `sand lang add spatial` to use `Position`. Construct one from coordinates
or take a player snapshot with `Position::player()`. It stores three
fixed-point `float` coordinates. Calling `.teleport()` moves the current
executor to that position in its current dimension. A position can be stored
in State, copied, and passed to functions.

The [checkpoints example](../../examples/lang_checkpoints/) saves a position
per player and uses a typed cooldown for returning to it.

### State migrations

Add a migration when a new pack version needs to transform saved State.
Keep old fields in the declaration during the upgrade so the migration can
read them using their original names and types.

```rust,ignore
#[migrate(upgrade_ledger)]
player state Ledger {
    coins: number,
    credits: number,
}

fn upgrade_ledger() {
    Ledger.credits = Ledger.coins * 2;
}
```

Each `#[migrate(function)]` upgrades one revision, starting at revision 1.
Keep migrations in order and append new ones. Sand initializes missing fields
from their defaults, runs the pending transforms in order, and advances the
owner's revision after each transform. Fresh State starts at the current
revision. Global State upgrades on load, player State upgrades during
initialization, and attached entity State upgrades when loaded for a tick or
explicitly attached again.

Use ordinary synchronous score and owned-value operations, conditionals,
and local loops. A migration takes no arguments and returns no value. Keep
the transform deterministic and use assignments that produce the same result
if an interrupted upgrade is retried. Calls to other functions and gameplay
side effects are outside migration bodies.

### Waiting within a function

Use `wait(20)` or `wait(seconds = 1)` in a function body, conditional branch,
statement `match` arm, or local loop with an entity or player executor. When the delay ends, the rest of the function runs
with its own saved parameters and locals. It runs as the original executor at
the saved position and dimension. State fields are read live when the function
resumes. A caller can continue as soon as the called function suspends.
Pass a computed `number` in ticks or seconds to give each invocation a
different delay. Invalid runtime durations continue immediately. Invalid
constant durations produce a diagnostic.

```rust,ignore
player state Orders { count: number }

fn prepare() {
    let quantity = Orders.count;
    actionbar!("Preparing {quantity} loaves…");
    if quantity > 1 {
        wait(seconds = 2);
    } else {
        wait(10);
    }
    give_item(Item::Bread, count = quantity);
}
```

Delays count ticks while the continuation's chunk is loaded in a vanilla
dimension or a dimension declared in the pack. A missing owner ends the wait.
A player can move to another dimension while the saved chunk is still loaded. A datapack reload
invalidates outstanding waits. Use `cancel_waits()` as the original executor
to cancel that entity's pending waits in this pack.

Use `wait(seconds = 2, group = "bakery")` and
`cancel_waits(group = "bakery")` to cancel just one system's waits for the
executing entity. Group names are shared across the pack. Write 1 to 32
lowercase letters, digits, or underscores in quotes. Cancelling a group with no
pending waits has no effect.

Waits keep up to 64 local fields including record fields and admit up to 128
pending instances at each wait statement.
After a branch finishes, execution continues with the statements that follow
it. A `return;` within a resumed branch ends that invocation. Use waits in
functions with no return value or mutable borrowed receiver. A waiting loop keeps its current iteration, bounds or collection
snapshot, and jump state. In an entity-selector loop, start delayed work by
calling a function as each selected entity or placing the wait directly in the
loop body. Each selected executor gets its own saved values. The caller keeps
running, and local writes by a resumed iteration stay within that iteration.
See the
[bakery example](../../examples/lang_waits/pack.sand).

Use `wait(20, server = true)` in a server-context function to wait without an
entity. Server waits count world ticks and resume in the server tick context.
They keep local values, support nested waits, and share the 128-instance limit
per wait site. Use `cancel_waits(server = true, group = "bakery")` to cancel
server waits in a group. A reload clears pending server waits.

A waiting function can deliver a computed result through a typed callback.
Keep the work that needs the result in that callback. It runs as part of
the resumed continuation.

```rust,ignore
fn compute_later(done: fn(number)) {
    wait(20, server = true);
    done(42);
}
```

## The world

Block positions are `pos(x, y, z)` in world coordinates or `offset(dx, dy, dz)`
relative to the executor. Both take constants.

`dimension_is(Dimension::Overworld)` checks the current command dimension.
`in_dimension(Dimension::TheNether, setup_nether)` runs a parameterless function
in that dimension with the same executor. Subsequent statements return to the
original context. Minecraft scales horizontal coordinates when crossing between
the Overworld and Nether. Use an absolute `pos(...)` for specific block locations.
The vanilla choices are `Overworld`, `TheNether`, and `TheEnd`. Pass the name of
a dimension declared by the pack, or a namespaced identity such as
`"another_pack:arena"` when the dimension comes from another pack.

`biome_is(Biome::Plains, pos(0, 80, 0))` checks the biome at a block position
in the current dimension. Biome identifiers and editor suggestions come from
the selected Minecraft version. Use a declared biome by name or a namespaced
identity for a biome from another pack.

### Flat dimensions

Declare a flat dimension with a stack of terrain layers in bottom-to-top order.

```rust,ignore
dimension Workshop {
    type: DimensionType::Overworld,
    biome: Biome::Desert,
    layers: [
        { block: Block::Bedrock, height: 1 },
        { block: Block::Sandstone, height: 2 },
    ],
}

fn inspect_workshop() {
    in_dimension(Workshop, report_workshop);
}

fn report_workshop() {
    if dimension_is(Workshop) {
        broadcast!("The workshop is available.");
    }
}
```

Each layer needs a block and a positive height. The defaults are
`DimensionType::Overworld`, `Biome::Plains`, and `structures: false`. Install
the pack before starting the world so Minecraft registers the dimension.
As in vanilla dimensions, scheduled entity work counts ticks only while its
chunk is loaded.

### Custom biomes

A `biome` declaration adds climate and colors that a dimension or predicate
can reference by name.

```rust,ignore
biome LanternGrove {
    temperature: 0.65,
    downfall: 0.5,
    water_color: "#246a73",
    foliage_color: "#436a55",
}

dimension Grove {
    biome: LanternGrove,
    layers: [{ block: Block::GrassBlock, height: 1 }],
}
```

`temperature` and `downfall` default to 0.5. Downfall ranges from zero to one.
`precipitation` defaults to true and enables rain or snow according to temperature.
Use quoted `"#rrggbb"` colors for `fog_color`, `water_color`, `water_fog_color`,
`sky_color`, `grass_color`, and `foliage_color`. The biome starts with empty
terrain feature and natural mob spawn tables. Install the pack before the
world starts to register the biome and any dimension that uses it.

Use `spawners` to assign natural spawning entries by mob category.
Entries use typed Minecraft entity names. Weights are relative to other entries
in the same category, and the default minimum group size is one.
When omitted, the maximum defaults to the minimum.
`spawn_costs` optionally supplies an entity’s local density budget and its
per-mob charge.

```rust,ignore
biome RabbitGrove {
    spawners: {
        creature: [{ entity: Entity::Rabbit, weight: 12, min_count: 2, max_count: 4 }],
    },
    spawn_costs: [{ entity: Entity::Rabbit, energy_budget: 1.5, charge: 0.25 }],
}
```

Vanilla still applies spawn rules such as block, light, and difficulty checks.

### Reading entity data and inventories

`entity_data_number("Health")` reads the vanilla integer result of a checked
NBT path on the executing entity. Use this for fields beyond Sand's named
entity properties. A list query such as `entity_data_number("Pos")` returns
its length. A missing value returns zero.

A player function can use `inventory_count(Item::Bread)` to count matching
items without removing them. A vanilla item counts all matching base stacks.
Pass a declared custom item to count only stacks created with that identity.
Use `clear_item(item, count = quantity)` with a constant or computed
`number` to remove up to that many matches. Zero removes nothing. A negative
runtime count skips removal.

### Moving items between inventory slots

An `inventory_transfer` moves items between two explicit slots of the current
player's inventory. Slots 0 through 8 are the hotbar. Slots 9 through 35 are the
main inventory.

```rust,ignore
inventory_transfer StowRations {
    from: 0,
    to: 9,
    amount: 2,
    success: packed,
    rejected: show_rejection,
}

fn pack_rations() {
    transfer_inventory(StowRations);
}

fn packed() { actionbar!("{green}Packed two rations"); }
fn show_rejection() { actionbar!("{red}The move could not be completed"); }
```

Use a constant `amount` from 1 to 99 for an exact move. Omit it to move the
whole stack. An occupied slot accepts items when their identity and components
match and it has capacity. The optional zero-argument callbacks run as the
player after the attempt releases the inventory lock.

### Player-owned HUDs

A named `hud` can show an actionbar, a private boss bar, or both. Start it
as a player with `start_hud` and use `cancel_hud` to end that player's
instances early.

```rust,ignore
player state Scout { progress: number }

hud Survey {
    ticks: 200,
    every: 5,
    actionbar: "{aqua}Survey in progress",
    bossbar: { title: "Survey", value: Scout.progress, max: 100 },
}

fn begin_survey() {
    Scout.progress = 0;
    start_hud(Survey);
}
```

A HUD lasts for its configured number of loaded ticks. The `every` field
sets the refresh interval and defaults to one tick. A boss bar value can be
a constant or a `number` or `timer` State field. It stays between zero and the bar's
maximum. A player's most recently started Sand HUD with an actionbar owns
that slot. Sand removes its presentation at cancellation, expiry, or reload.

### Structure templates

Save a structure as a Minecraft gzip NBT file and put it inside the package.
Declare its path relative to the package root that contains `pack.sand`.

```rust,ignore
structure_template CheckpointMarker {
    source: "structures/checkpoint_marker.nbt",
}

fn build_checkpoint() {
    place_template(CheckpointMarker, offset(0, 0, 2));
}
```

The build copies the asset into the datapack under its declared name.
`place_template` accepts `pos(...)` or `offset(...)` and requires the destination
chunks to be loaded. To place a template supplied by another pack, pass a
namespaced identity such as `"adventure:rooms/camp"`. The portable JSON produced
by `sand lang emit` includes the asset bytes, so another compiler host can
build the same template without access to the package files.

### Block data and regions

`block_data_number(at, path)` reads the vanilla integer result of a block
entity's data query. For example, `block_data_number(offset(0, 0, 1), "Items")`
reads the number of entries. A failed read returns zero. Use
`set_block_data(at, path, value)` to write a computed integer or a typed NBT
literal. String and list literals and `Nbt { ... }` compounds work as values.
`copy_block_data(from, source_path, to, destination_path)` transfers the
selected value between loaded block entities. The copy requires a present
source value. Use `remove_block_data(at, path)` to clear a selection.
Minecraft applies each block entity's own data rules.

`clone_blocks(from, to, destination)` copies an inclusive region to a destination
corner. Use `masked = true` to preserve the destination beneath source air, or
`filter = Block::Stone` to copy only one block or a declared block tag.
`mode = CloneMode::Move` clears what it copies, and `CloneMode::Force` allows
overlapping regions. The default mode is `CloneMode::Normal`. See the
[checkpoint room example](../../examples/lang_world_ops/pack.sand).

```rust,ignore
set_block(Block::Stone, offset(0, -1, 0));
fill_blocks(Block::Air, offset(-2, 0, -2), offset(2, 3, 2));
set_time(6000);
set_weather(Weather::Rain, seconds = 60);
set_gamemode(GameMode::Adventure);
add_experience(points = 10);   // or levels = 2
```

## Boss bars

```rust,ignore
bossbar Raid {
    title: "{red}Raid Boss",
    color: BossBarColor::Purple,
    style: BossBarStyle::Notched10,
    max: 200,
}

fn begin() {
    show_bossbar_all(Raid);
    set_bossbar(Raid, Fight.boss_hp);
}
```

`show_bossbar`/`hide_bossbar` change one player and leave the others alone;
`show_bossbar_all`/`hide_bossbar_all` change everyone online. `set_bossbar` and
`set_bossbar_max` take any `number` expression.

## Items, recipes, and tags

Data resources are declarations with typed fields:

```rust,ignore
/// A sword forged from redstone.
item RubySword {
    base: Item::DiamondSword,
    name: "{red}{bold}Ruby Sword",
    lore: ["{gray}Forged in fire."],
    rarity: Rarity::Epic,
    unbreakable: true,
    enchantments: [{ enchantment: Enchantment::Sharpness, level: 5 }],
}

tag Ores: Item { values: [Item::IronOre, "#minecraft:coal_ores"] }

recipe RubySwordRecipe {
    shaped: [" R ", " R ", " S "],
    key: { R: Item::RedstoneBlock, S: Item::Stick },
    result: RubySword,
}

recipe Nugget { smelting: Ores, result: Item::IronNugget, experience: 0.5 }
recipe RedDye { shapeless: [Item::Poppy, Item::Poppy], result: Item::RedDye, count: 2 }
```

- `item` fields: `base`, `name`, `lore`, `rarity`, `glint`, `unbreakable`,
  `max_stack`, `enchantments`, `custom_model_data`. Enchantment variants use the
  same target catalog as Rust. Use a namespaced string for an external
  enchantment. `give_item(RubySword)` and
  `clear_item(RubySword)` use its identity, a `custom_data` marker, exactly
  like Rust's `CustomItem::custom_data`.
- `recipe` takes exactly one of `shaped` (with `key`), `shapeless`,
  `smelting`, `blasting`, `smoking`, `campfire`, or `stonecutting`, plus
  `result` and optionally `count`, `experience`, and `time`. Custom items can
  be results but not ingredients: vanilla matches ingredients only by item or
  tag.
- `tag Name: Item { values: [...] }` (or `Block`, `Entity`) takes `values`:
  typed values, other tags of the same type, or `"#ns:tag"` strings, and
  optionally `replace`.

These lower through the same typed builders as Rust (`ShapedRecipe`,
`CustomItem`, `TypedTag`, ...). The editor completes field names and
offers the right typed values for each field.

Smithing upgrades name their template, `base`, and `addition`:

```rust,ignore
recipe Upgrade {
    smithing: Item::NetheriteUpgradeSmithingTemplate,
    base: Item::DiamondAxe,
    addition: Item::NetheriteIngot,
    result: Item::NetheriteAxe,
}
```

Smithing trims apply a named pattern to the armor in the base slot, preserving
its item and components. Use a quoted namespaced trim pattern ID. Omit
`result` and `count` for trim recipes.

```rust,ignore
recipe GuardTrim {
    smithing_trim: Item::SentryArmorTrimSmithingTemplate,
    base: "#minecraft:trimmable_armor",
    addition: Item::AmethystShard,
    trim_pattern: "minecraft:sentry",
}
```

### Written books

Use `book` on a custom item with `base: Item::WrittenBook`. The native title and
author are quoted strings. Pages accept fixed formatted text, translations and
interactive components. `Text::page(label, number)` links to a page in the same
book, starting at 1.

```rust,ignore
item FieldGuide {
    base: Item::WrittenBook,
    book: {
        title: "Field guide",
        author: "Sand",
        pages: [
            Text::page("{aqua}Contents", 2),
            "{bold}Chapter 1\n{reset}Explore the world.",
        ],
    },
}
```

Give the book with `give_item(FieldGuide)` or use it as a recipe result. Sand
validates the native 32 UTF-16 code-unit title limit and supports up to 100
pages. Pages use fixed content authored in the item declaration.

### Advancements

Use named criteria to define player milestones tracked by Minecraft. Each
criterion uses one trigger: `inventory_item`, `place_block`, or `kill_entity`.
Every criterion is required by default. Set `require_any: true` to require one.
An advancement can award experience points once per completion.

```rust,ignore
advancement ReadyForAdventure {
    display: {
        icon: Item::Spyglass,
        title: "{gold}Ready for adventure",
        description: "Get a spyglass or place a campfire.",
        background: "minecraft:gui/advancements/backgrounds/adventure",
    },
    criteria: {
        get_spyglass: { inventory_item: Item::Spyglass },
        place_campfire: { place_block: Block::Campfire },
    },
    require_any: true,
    experience: 5,
}
```

An advancement without a `display` still tracks progress and awards experience.
Minecraft 26.3 requires a background texture for a visible root. Keep criterion
names stable to preserve progress after a datapack update.

Use `advancement_done(ReadyForAdventure)` in a player function to read the
whole advancement. Pass `criterion = "get_spyglass"` to check one named
criterion. A quoted namespaced ID also lets you read another pack’s advancement.

Use `#[on(advancement = ReadyForAdventure)]` to run a function as the player
when they complete this pack's advancement.

```rust,ignore
#[on(advancement = ReadyForAdventure)]
fn celebrate() {
    chat!("{gold}Ready for adventure!");
}
```

The advancement's native reward runs the handlers once per player completion.
Handlers in the same module run in declaration order. Experience rewards and
the player's advancement progress use the same vanilla completion.

### Owned display scenes

A scene groups named block, item and text displays. A part's optional pose
sets local translation, scale and yaw. Each keyframe replaces the complete
pose and starts vanilla interpolation at a positive loaded tick. Use
`follow: true` to keep a scene at its owner's position. Each start creates a
separate instance, and expiry or cancellation removes its displays.

```sand
scene SupplyBeacon {
    ticks: 40,
    parts: [{ name: "base", block: Block::GoldBlock }],
    keyframes: [
        { part: "base", at: 5, duration: 10, pose: { translation: [0, 2.5, 0] } },
    ],
}
```

Run `show_scene(SupplyBeacon)` as the player, or
`cancel_scene(SupplyBeacon)` to remove their instances. See the
[supply beacon example](../../examples/lang_signal_scene/).

### Bounded player jobs

A job processes zero-based indices in order. The step function takes one
`number` and runs as the player who started that instance. The optional
completion function has no parameters and runs after the final step. The
`per_tick` callback count is per instance, and defaults to one. The `ticks`
deadline uses loaded ticks. `capacity` limits simultaneous instances across
owners and defaults to 128.

```sand
job CheckSupplies {
    ticks: 40,
    count: 5,
    per_tick: 2,
    step: inspect_supply,
    complete: finish_inspection,
}
```

Use `start_job(CheckSupplies)` to start a fresh instance. Use
`cancel_job(CheckSupplies)` to stop the executing player's instances. Work that
expires, is cancelled, or loses its owner ends without invoking completion.
See the [supply inspection example](../../examples/lang_supply_job/).

### Player progression and sessions

A `progression` stores its next objective in a player's State. The field must be
a non-ticking number with a zero default and, when bounded, it must admit zero
through the objective count. Objectives are checked in order. Each eligible
advance commits its successor before running its optional reward function.

```rust,ignore
player state Journal { stage: number, arrived: bool, skip_optional: bool }

progression Trail {
    progress: Journal.stage,
    objectives: [
        { when: Journal.arrived, reward: grant_rations },
        { reward: offer_glowstone },
        { reward: finish_trail },
    ],
    branches: [{ from: 0, to: 2, when: Journal.skip_optional }],
}

session TrailRun { progression: Trail, ticks: 200, every: 20 }
```

A branch uses zero-based indices. The first eligible forward branch wins, and
the total number of objectives is the completion index. Skipped objectives
award nothing. `advance_progression(Trail)` checks one objective, and
`progression_completed(Trail)` tests completion. `reset_progression(Trail)`
enables an explicit replay.

Use `start_session(TrailRun)` to advance on the declared cadence until
completion or expiry. `cancel_session` preserves progress, `reset_session`
cancels and resets, and `restart_session` resets and starts again. Changes to
objective order affect saved indices. See the [trail quest example](../../examples/lang_quest_session/).

### Custom health and combat

A `combat` policy changes the executing player's authoritative health stored
in State. It can consume a shield score first, apply a resistance fraction and
conditional critical multiplier, and start an immunity timer after committing
a hit. Use `combat_damage(Training, amount)` and `combat_heal(Training, amount)`
with constants or runtime numbers.

```rust,ignore
player state Vitals { health: number = 20, shield: number = 5, immunity: timer }

combat Training {
    health: Vitals.health,
    max: 20,
    shield: Vitals.shield,
    resistance: 0.25,
    immunity: { timer: Vitals.immunity, ticks: 10 },
    hit: show_hit,
}
```

Optional `hit`, `defeat`, and `healed` callbacks run after their State updates.
Healing caps at the maximum and keeps a defeated player at zero. Native
Minecraft damage uses the separate `damage` operation. See the
[custom health drill](../../examples/lang_state_combat/).

### Abilities with costs and cooldowns

An `ability` checks admission for the executing player. It can use a number
as its balance, a timer as its cooldown, and a boolean State field as an
additional requirement. Add an optional projectile to use its flight and
impact behavior with the ability's own lifetime.

```rust,ignore
player state Focus { mana: number = 8, cooldown: timer, ready: bool = true }

ability CastSpark {
    ticks: 24,
    projectile: Spark,
    cost: { balance: Focus.mana, amount: 2 },
    cooldown: { timer: Focus.cooldown, ticks: 20 },
    when: Focus.ready,
}
```

Use `activate_ability(CastSpark)` to attempt admission. The runtime commits the
charge and cooldown before running the optional `start` callback.
`diagnose_ability(CastSpark)` reports failed requirements without activation.
`cancel_ability(CastSpark)` stops that player's instances and preserves the
committed State. See the [mana-powered spark example](../../examples/lang_abilities/).

### Player-owned projectiles

Declare a sampled projectile, then launch it from a player function.
`ticks` limits its loaded flight time, `speed` controls travel per tick, and
`radius` controls its entity collision samples. Omit `target` to consider all
eligible entities. A projectile can hit a non-air block regardless of the
entity filter.

```rust,ignore
projectile Spark {
    ticks: 24,
    speed: 1.5,
    radius: 0.75,
    target: Entity::Zombie,
    damage: 6,
    hit: show_impact,
}

fn show_impact() {
    spawn_particle(Particle::Flame, count = 8);
}

fn cast() { launch_projectile(Spark); }
fn stop_casting() { cancel_projectile(Spark); }
```

`damage` uses native damage in hit points and awards kill credit to the
launching player. The optional `hit` function runs as that player at the
collision position for block and entity hits. Cancellation affects only
the executing player's active flights of that projectile. Flight stops in
unloaded chunks. See the [training sparks example](../../examples/lang_projectiles/).

### Native menus

Use `menu` for a native dialog with gameplay choices. A button can call a
parameterless function as the selecting player, enter another view, or do both.

```rust,ignore
menu CampSupplies {
    title: "{gold}Camp supplies",
    ticks: 200,
    page_size: 1,
    while_open: Camp.active,
    choices: [
        { label: "Stow rations", next: "confirm" },
        { label: "Show instructions", action: show_slots },
    ],
    views: [{
        name: "confirm",
        title: "Pack two rations?",
        choices: [
            { label: "Pack rations", action: pack_rations, when: Camp.can_pack },
            { label: "Back", next: "main" },
        ],
    }],
}

fn open_supplies() {
    open_menu(CampSupplies);
}
```

The engine gives each player a new set of button tokens on opening or changing
pages. A valid choice consumes that player's session before calling its
function. Use `close_menu(CampSupplies)` to invalidate its session early.
`ticks` sets the online timeout. A menu supports up to 256 buttons across all
views, and each view needs at least one button. Set `page_size` from 1 to 64 for
automatic Previous and Next controls within each view. The menu opens at
`"main"` unless `start` names another view. A transition gives the destination
fresh tokens. If the callback opens another menu itself, that menu takes priority.

`when` checks a boolean State field immediately before a choice runs. The
button stays visible when the field is false. `while_open` checks a boolean
State field when opening, on each online tick, and again during dispatch. A
false or unavailable field closes or rejects that player's session.

### Notice dialogs

A named `dialog` displays a native notice with fixed rich text and a dismiss
button. Use `show_dialog` and `clear_dialog` in a player function.

```rust,ignore
dialog Guide {
    title: "{gold}Field journal",
    body: [
        "Mine stone or defeat zombies to record entries.",
        "Right-click the notebook to read your notes.",
    ],
    button: "Start exploring",
}

#[on(first_join)]
fn welcome() {
    show_dialog(Guide);
}
```

The body and button are optional. Omit the button label to use the client's
localized Done label. The title, body and button can use `Text::translate`.
A quoted namespaced ID can refer to a dialog in another pack.

### Named predicates

```rust,ignore
predicate Camp {
    block: Block::Stone,
    biome: Biome::Plains,
    dimension: Dimension::Overworld,
}
fn welcome() {
    if predicate_matches(Camp) {
        broadcast!("The camp is ready.");
    }
}
```

A declaration exports a native predicate. Include at least one constraint.
All listed constraints must pass. `block`, `biome`, and `dimension` read the
current execution location. `weather` and `chance` add optional native checks.
A loot pool can share it with `{ when: Camp, items: [Item::Bread] }`.

### Damage types

```rust,ignore
damage_type Thorn {
    message_id: "scout_thorn",
    exhaustion: 0.2,
    scaling: DamageScaling::Never,
    effects: DamageEffects::Thorns,
}

fn thorn_hit(caster: EntityRef) {
    damage(2.0, type = Thorn, by = caster);
}
```

A declaration exports a native damage type. `message_id` supplies the suffix of
its `death.attack` translation key. Provide translated text in the resource
pack. Use `DamageType::Generic` or another typed vanilla ID when a custom
resource is unnecessary. The optional `by` retains vanilla attribution for
either kind of damage type.

### Loot tables

```rust,ignore
loot Crate {
    rolls: 2,
    items: [Item::Diamond, { item: Item::Emerald, weight: 4, count: 3 }],
}

fn open_crate() { give_loot(Crate); }   // or drop_loot(Crate) at the executor
```

Use `pools` for independent groups of loot. Each pool rolls its own weighted
items, and its optional `chance` and `weather` conditions must both pass.
`Weather::Rain` matches rain without thunder. `Weather::Thunder` matches
thunderstorms. Chance runs from zero to one and is checked once per pool.

```rust,ignore
loot RaidSupplies {
    pools: [
        { items: [Item::Bread] },
        { weather: Weather::Thunder, chance: 0.25, items: [Item::Diamond] },
    ],
}
```

## Flows

A flow drives a State field through phases. Inside a flow over an enum field,
variants can be written without their enum prefix.

```rust,ignore
enum Phase { Idle, Fighting, Resting }
player state Hero { phase: Phase, energy: number = 100 }

flow Encounter on Hero.phase {
    Idle -> Fighting if Hero.energy >= 30 priority 10;
    Fighting -> Resting if Hero.energy <= 10;

    enter Fighting => start_fight;       // a named function
    tick Fighting every 20 {             // or an inline body
        Hero.energy -= 1;
    }
    exit Resting { Hero.energy = 100; }
}
```

At most one transition fires per cycle (higher `priority` first, then
declaration order). A transition guard compares the field with constants.

## Execution context

Player State belongs to the executing player, so code that reads or writes it
needs one. Sand infers each function's context:

- a function that touches player State, uses a player builtin (`chat!`,
  `give_item`, `play_sound`, `{player}`, ...) outside a `for player in players()` loop,
  or calls a function that does, **runs as a player**;
- a function that acts on the executing entity (`kill()`, `tag`, `damage`)
  outside a loop, or calls one that does, **runs as an entity**;
- everything else **runs on the server** and can be called from anywhere.

Entry points without a player are `#[tick]`, `#[load]`, the
`minecraft:tick`/`minecraft:load` tags, scheduled functions, and global flow
hooks. If one needs a player, the error explains the chain that requires it:

```text
error[context]: `tick` runs from #[tick] without a player, but it calls `regen`, which needs a player
  --> pack.sand:5:4
  |
5 | fn tick() {
  |    ^^^^
  --> pack.sand:4:1
  |
4 | #[tick]
  | ^^^^^^^ runs on the server here
  --> pack.sand:6:5
  |
6 |     regen();
  |     ^^^^^ player function called here
  = help: move player work into `for player in players() { ... }`
```

Player event handlers (`#[on(...)]`) always run as the player.

## How Sand compiles

`sand lang build` runs four stages, all in one process with no Cargo build:

1. **Parse.** Each `.sand` file is parsed into a syntax tree with error
   recovery, so one mistake never hides the rest of the file.
2. **Analyse.** Names resolve through modules and `use`; types, mutability,
   arguments, and returns are checked; and every function's execution context
   (player or server) is inferred from what it touches. This is the same
   analysis the language server runs as you type.
3. **Lower.** The package becomes Sand's portable program: a JSON-shaped model
   of State, functions, and typed operations (`sand lang emit` prints it).
4. **Emit.** The same backend used by Rust and JSON frontends validates the
   program and writes the datapack.

Datapacks have no variables, call stack, or return values, so Sand maps each
language feature onto what Minecraft does have:

| Sand | Minecraft |
| --- | --- |
| `player state` / `global state` field | a scoreboard objective, held by `@s` or a fixed holder |
| `let` variable, parameter, temporary | a slot on one compiler-owned global holder |
| `fn` | a `.mcfunction` file |
| calling `f(a, b)` | copy arguments into `f`'s parameter slots, then `function ns:f` |
| a call that can come back to its caller | first save the caller's slots on a `<ns>:__sand_frames` storage stack, restore them after |
| `return value;` | write `f`'s return slot, then `return 1` |
| `return;` from inside an `if` | the branch helper returns 1; the caller runs `execute if function <helper> run return 1` |
| `if` / `else` | `execute if score ... run return run function <then>` in a helper |
| `while`, `loop`, `for i in a..b` | a helper that runs its body and tail-calls itself while the condition holds |
| `break;` / `continue;` | a jump flag that skips the rest of the pass |
| `Player.health` | `execute store result score ... run data get entity @s Health 1000` |
| `Player.max_health = expr;` | the value is copied into storage and applied by a macro function |
| `x * 3 + y` | `scoreboard players operation` steps through temporaries |

Variables are slots rather than stack frames, so Sand works out from the call
graph which calls can lead back into their caller (directly, through other
functions, or through an event a handler emits again). Only those calls save
and restore the caller's variables; every other call is a plain `function`
command. Arguments that call functions, and the arguments of a function
calling itself, are computed before any parameter is overwritten. Commands
run one after another within a tick, so players never see each other's
variables, even inside `for player in players()`.

## Parity with Rust

The language emits resources through Sand's compiler and uses the same typed
command and State models as Rust. Today the language covers:

| Area | Language | Rust-only for now |
| --- | --- | --- |
| State | player, global, and entity State; scalars, records, strings, arrays, vectors, timers, clamps, flows, impl methods, and ordered migrations | derived values and general lifecycle hooks |
| Logic | recursive functions, borrowed method receivers, generics, static traits, function values, owned closures, destructuring, value-producing `match`, loops, casts, and entity/player waits inside branches, statement matches and local loops | Rust compile-time metaprogramming and richer owned tasks |
| Players and entities | loops and queries with NBT and score filters, retained identities and relationships, attributed damage, mounts, teleportation, attributes, entity properties and checked NBT reads, effects, items, inventory counts, equipment slots, non-player velocity, experience, and entity flags | archetypes |
| Events | join, first join, death, kills (by type), killed by, item use, crafting, pickups, mining, jumping, dropping, sleeping, damage taken/dealt, custom events with payloads, authored advancement completion | broader advancement triggers, equipment changes, tracked transitions, event chains |
| World | block placement, reads and cloning, time, weather, dimension and biome tests, scoped calls, flat dimensions, custom biome climate, colors and spawn tables, structure templates, typed block data literals and copies | natural structure generation, biome terrain features, other dimension generators and types |
| Text and feedback | chat, broadcast, actionbar, titles, translations, clickable text and tooltips, sounds, particles, boss bars | VFX sequences |
| Data | custom items, crafting/cooking/stonecutting/smithing upgrade and trim recipes, item/block/entity tags, conditional multi-pool loot, damage types, native world predicates, advancements, written books and notice dialogs | broader triggers, worldgen, interactive dialog flows |
| Gameplay systems | checked inventory transfers, player-owned HUDs, native menus, projectiles, player progression, sessions, abilities, State combat, bounded jobs and display scenes | encounters, rosters, status effects, checkpoints |

`raw!("...")` covers single commands that have no builtin yet.


## Automated runtime regression probe

The repository includes a control-flow fixture and a runner that compiles it,
executes 20 assertions on a disposable vanilla 26.2 server, and checks reload:

```sh
cargo build -p sand-cli
python3 scripts/validate_language.py --binary target/debug/sand \
  --jar /path/to/bundled-server.jar --output /tmp/sand-language-probe
```

The JAR must be the complete downloaded server bundle, including its libraries.
The output path must be new; it retains the emitted pack and server logs.

## Runtime command arguments

Item counts, experience points/levels, sound volume/pitch, damage amounts and
teleport coordinates accept parameters, State values and calculations:

```rust,ignore
fn reward(count: number) { give_item(Item::Bread, count); }
fn hit(amount: float) { damage(amount); }
fn move_to(x: float, y: float, z: float) { teleport(x, y, z); }
```

Arguments are evaluated once, in source order. A later argument cannot change a
value already captured by an earlier one. Runtime item counts outside `1..=6400`
and damage amounts outside `0.01..=1000000` perform no action; invalid constants
are compile errors. Runtime sound volumes must be nonnegative and pitch must be
within `0..=2`; out-of-range values perform no action. Other builtin parameters,
including effect durations, still require constants unless their hover
documentation says otherwise.

The compiler validates vanilla identifiers against the pack's selected version,
independently of the version used to build the CLI. Completion follows that same
catalog. Custom namespaces remain explicit interoperability boundaries. Refresh
committed catalogs only through the maintenance generator:

```sh
cargo run -p sand-build --example refresh_vanilla_catalog -- \
  sand-version/src/vanilla_catalog.json 26.1 26.1.1 26.1.2 26.2 26.3
```

A separate vanilla 26.2 proof uses two real passive clients:

```sh
python3 scripts/validate_language_values.py --binary target/debug/sand \
  --jar /path/to/bundled-server.jar --output /tmp/sand-values-probe
```

It checks independent inventories, invalid-count guards, custom-item macro text,
player position/attribute properties, dynamic experience and entity damage, and
coordinate evaluation order. It does not verify rendered UI or scripted player
controls.
