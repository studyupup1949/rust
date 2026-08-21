# Assignment Tracker
A simple GUI-based assignment tracker built in Rust using `egui` and `eframe`.
It helps students manage their assignments with reminders, due dates, and emoji-supported visuals.

## Run

```bash
cargo run
```

## Controls
- Click input boxes or press `Tab` to switch fields.
- Type to enter text, `Backspace` to delete.
- Press `Enter` or click **+ Add** to create an assignment.
- Click `[ ]` to toggle done, **Delete** to remove an item.

## Persistence
Saves to a human-readable `assignments.json` next to the executable.