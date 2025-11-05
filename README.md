# bullet-journal (bj)

A simple terminal bullet journaling CLI. Stores daily Markdown files in `~/.local/share/bullet_journal/YYYY-MM-DD.md`.

## Install

```bash
cargo install --path "$HOME/bullet-journal"
```

## Usage

```bash
bj add "Draft project plan"                               # add to today
bj add -d 2025-11-05 "Backfill notes"                     # add to a date
bj add -p high -t work -t urgent "Release train" -n "prep notes" -n "ping QA"
bj list                                # list today
bj list -d 2025-11-05                  # list a date
bj list -t work -p 3                   # filter by tag and priority
bj done 2                              # mark item 2 done (today)
bj done -d 2025-11-05 3                # mark item 3 done for date
bj migrate --from 2025-11-04           # move open items to today
bj week                                # weekly view for current week
bj week -d 2025-11-05 -t work          # weekly view filtered
```

- Open items use `- [ ]` and completed items use `- [x]` in Markdown. Priority appears as `(!)`, `(!!)`, or `(!!!)` before the text. Tags appear as `#tag` suffixes. Notes are on subsequent lines like `  - note: ...`.
- IDs are positional per day file (1-based) and ignore non-bullet lines.

