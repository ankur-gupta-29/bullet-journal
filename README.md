# bullet-journal (bj)

A simple terminal bullet journaling CLI. Stores daily Markdown files in `~/.local/share/bullet_journal/YYYY-MM-DD.md`.

## Install

```bash
cargo install --path "$HOME/bullet-journal"
```

If you want the short command `bj`:

```bash
ln -sf "$HOME/.cargo/bin/bullet-journal" "$HOME/.cargo/bin/bj"
```

Ensure `~/.cargo/bin` is on your PATH.

## Uninstall

Remove the installed binaries and optional symlink:

```bash
cargo uninstall bullet-journal
rm -f "$HOME/.cargo/bin/bj"
```

Remove the daily reminder (if you enabled it) and its files:

```bash
systemctl --user disable --now bj-remind.timer || true
rm -f "$HOME/.config/systemd/user/bj-remind.timer" "$HOME/.config/systemd/user/bj-remind.service"
systemctl --user daemon-reload
rm -f "$HOME/.local/bin/bj-remind"
```

Optionally delete your journal data (irreversible):

```bash
rm -rf "$HOME/.local/share/bullet_journal"
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
bj delete 3                            # delete item 3 (today)
bj delete -d 2025-11-05 2              # delete item 2 for date
bj migrate --from 2025-11-04           # move open items from date to today
bj migrate --from 2025-11-04 --to 2025-11-10          # move open items to specific date
bj migrate --from 2025-11-04 --to 2025-11-10 --id 2   # move specific item to date
bj week                                # weekly view for current week
bj week -d 2025-11-05 -t work          # weekly view filtered
bj cal                                 # month calendar with markers

# Meetings
bj meeting add -d 2025-11-05 -t 15:00 -u 30 -g work "Team sync"
bj meeting list                        # list today's meetings sorted by time
bj meeting notify -w 15                # notify meetings starting in next 15 minutes
```

- **Storage format** (Markdown):
- Open: `- [ ] (!!!) Release train #work #urgent`
- Done: `- [x] (!!!) Release train #work #urgent`
- Notes: indented lines like `  - note: prep notes`
- **IDs**: positional per day file (1-based) and ignore non-bullet lines.

### Meetings format
- Stored inline as bullets with a prefix, e.g.: `- [ ] [mtg 15:00 30] Team sync #work`
- Duration is optional; default 60 minutes.
- Notes can be added with `-n` and appear on indented lines under the item.

## Meeting notifier (optional)

Enable periodic checks (every minute) for upcoming meetings with notifications 15 minutes before start:

```bash
install -d -m 755 "$HOME/.config/systemd/user"
cat > "$HOME/.config/systemd/user/bj-meeting-notify.service" << 'EOF'
[Unit]
Description=Bullet Journal meeting notifications

[Service]
Type=oneshot
Environment=PATH=%h/.cargo/bin:%h/.local/bin:/usr/local/bin:/usr/bin
ExecStart=%h/.cargo/bin/bj meeting notify -w 15

[Install]
WantedBy=default.target
EOF

cat > "$HOME/.config/systemd/user/bj-meeting-notify.timer" << 'EOF'
[Unit]
Description=Run meeting notifications every minute

[Timer]
OnCalendar=*-*-* *:*:00
Unit=bj-meeting-notify.service
Persistent=true

[Install]
WantedBy=timers.target
EOF

systemctl --user daemon-reload
systemctl --user enable --now bj-meeting-notify.timer
```

## Daily reminder (optional)

Create a daily desktop notification at 09:00 using systemd user timers:

```bash
install -d -m 755 "$HOME/.local/bin"
cat > "$HOME/.local/bin/bj-remind" << 'EOF'
#!/usr/bin/env bash
set -euo pipefail
TITLE="Bullet Journal"
MSG="Don’t forget to fill your journal today. Run: bj add \"...\""
if command -v notify-send >/dev/null 2>&1; then
  notify-send "$TITLE" "$MSG"
else
  echo "$TITLE: $MSG"
fi
EOF
chmod +x "$HOME/.local/bin/bj-remind"

install -d -m 755 "$HOME/.config/systemd/user"
cat > "$HOME/.config/systemd/user/bj-remind.service" << 'EOF'
[Unit]
Description=Bullet Journal daily reminder notification

[Service]
Type=oneshot
Environment=PATH=%h/.local/bin:/usr/local/bin:/usr/bin
ExecStart=%h/.local/bin/bj-remind

[Install]
WantedBy=default.target
EOF

cat > "$HOME/.config/systemd/user/bj-remind.timer" << 'EOF'
[Unit]
Description=Run Bullet Journal reminder daily at 09:00

[Timer]
OnCalendar=*-*-* 09:00:00
Persistent=true
Unit=bj-remind.service

[Install]
WantedBy=timers.target
EOF

systemctl --user daemon-reload
systemctl --user enable --now bj-remind.timer
```

Change the reminder time by editing `OnCalendar` and restarting the timer.

