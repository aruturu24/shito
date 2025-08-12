### Shito — D&D 5e character sheets in your terminal
Shito is a fast, keyboard-driven TUI app built with [Rust](https://www.rust-lang.org/) and [ratatui](https://github.com/ratatui-org/ratatui) for creating, managing, and playing D&D 5e characters. It stores your sheets locally in a SQLite database and helps you roll with the right modifiers.

---

## Features
- **Beautiful TUI**: Clean list + tabbed details (General, Skills, Inventory)
- **Create sheets**: Guided wizard captures name, class, race, abilities, HP, AC, Speed, and skill proficiencies
- **Manage sheets**: Edit HP, level, spell slots, and inventory items
- **Local storage**: SQLite (`shito.sqlite3`) with JSON for arrays
- **Dice roller**:
  - `NdM` (e.g., `2d6`, `d20`)
  - Skill checks (e.g., `stealth`) include ability + proficiency if proficient
  - Ability checks (`str`, `dex`, `con`, `int`, `wis`, `cha`)

---

## Install
Requirements:
- Rust toolchain (`rustup`) and Cargo
Build and run:
```bash
cargo run
```
The binary name is `Shito` (capital S) if you build with `cargo build`:
```bash
cargo build --release
./target/release/Shito
```

---

## Data storage
- Database: `shito.sqlite3` in working directory
- Schema: single `characters` table; arrays (spell slots, inventory, proficiencies) are stored as JSON

---

## License
[MIT](https://github.com/aruturu24/shito/blob/main/LICENSE)
