# Emberbound

A pixel-art dragon action RPG that runs in the browser, written in Rust with
[Bevy](https://bevyengine.org) and compiled to WebAssembly.

You are the last Wyrmwarden, bonded to a hatchling dragon called Cinder. Cut
through the monsters of a procedurally generated valley, gather their embers,
choose a boon every level, and lure out and slay three elder dragons: the Moss
Wyrm, the Frost Drake and the Ashen Tyrant. Cinder grows up alongside you.

Every sprite is drawn procedurally in code, and so is the pixel font (its
glyphs are compiled into a TrueType file at start-up), so the repository
contains no image or font files at all.

## Controls

| Action | Keys |
|---|---|
| Move | `WASD` / arrow keys |
| Sword | `Space` / `J` / left click (hold) |
| Dash (brief invulnerability) | `Shift` / `K` |
| Pick a boon | `1` `2` `3` |
| Start / restart | `Enter` |

### On a phone or tablet

Hold the device in landscape. The first tap goes fullscreen where the
browser allows it.

| Action | Touch |
|---|---|
| Move | Thumb anywhere on the left side (a stick appears under it) |
| Sword | Hold anywhere on the right side |
| Dash | The DASH button above the sword |
| Pick a boon | Tap a card |
| Start / restart | Tap |

## Boons

Every level offers three boons. They come in three rarities, and the
rarer ones change how a run plays rather than just its numbers:

- **Common** (grey): steady upgrades such as Whetted Edge (+25% sword
  damage), Scalemail (+2 armor), Thornhide (monsters that touch you get hurt)
  or Ember Tithe (+25% experience).
- **Rare** (frost): Ember Wake (your dash leaves a trail of fire), Sanguine
  Edge (critical hits heal), Twin Flame (Cinder's bolts pierce), Wildfire
  (slain monsters burst into fire bolts), Last Stand (stronger and faster
  when wounded) and more.
- **Legendary** (gold, smouldering frame): Crescent Wave (every third swing
  hurls a piercing crescent of light), Phoenix Feather (survive one fatal
  blow in a burst of flame) and Molten Heart (huge damage, but it is
  *cursed* and costs 30 max health).

Legendaries become likelier as you level. Most boons stack up to a limit;
the card shows how many you already have.

## Run it

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk --locked
trunk serve --release        # http://localhost:8080
```

A native window works too: `cargo run --release`.

For play-testing, `--features cheats` adds debug keys (`G` embers, `H` smite
monsters, `N` smite the dragon, `T` heal, `B` take the next boon); see
`src/cheats.rs`.

```bash
cargo test -p emberbound_domain -p emberbound_pixelart   # engine-free, fast
```

## Deploy to GitHub Pages

1. Push this repository to GitHub (branch `main`).
2. In *Settings -> Pages*, set *Source* to **GitHub Actions**.
3. Every push to `main` runs `.github/workflows/deploy.yml`, which tests the
   domain, builds with Trunk and publishes `dist/`.

The build uses relative URLs, so it works under `https://<user>.github.io/<repo>/`.

## Architecture

The code is organised by bounded context, and dependencies point inwards: the
engine depends on the rules, never the reverse.

```
                 emberbound (binary)        composition root, input adapter, run session
                        |
     +---------+--------+--------+---------+
   world    combat    monsters    ui         one Bevy plugin per bounded context
     +---------+--------+--------+---------+   (they never depend on each other)
                        |
                      core                   shared kernel: states, sets, components,
                     /    \                  messages, resources, sprite library
               domain      pixelart          pure Rust, no engine
```

| Crate | Role |
|---|---|
| `crates/domain` | The rules: combat maths, XP and boons, the bestiary, the encounter director, companion growth, valley generation. No dependencies; randomness is a port (`RandomSource`). |
| `crates/pixelart` | Every sprite, authored as palette-indexed grids and rasterised to RGBA. No dependencies. |
| `crates/core` | The contract the gameplay crates cooperate through, and the adapters for the domain's ports (`GameRng`) and the art (`SpriteLibrary`). |
| `crates/world` | Camera, tile map, the hero, movement and terrain collision. |
| `crates/combat` | Sword, projectiles, damage resolution, loot, the dragon companion. |
| `crates/monsters` | Drives the domain's director; monster and dragon AI. |
| `crates/ui` | HUD, screens, announcements, damage numbers and effects. |
| `src/` | Wires everything together; turns devices into `PlayerIntent`; owns the life cycle of a run. |

Gameplay crates talk only through messages (`DamageRequest`, `ActorDied`,
`SpawnProjectile`, ...) and components defined in `core`, and read input as a
device-independent `PlayerIntent` rather than key codes.

## License

MIT
