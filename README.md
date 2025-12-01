# 📓 Bullet Journal CLI (bj)

> A beautiful, fast, and terminal-based bullet journal for hackers and developers.

`bj` is a command-line tool designed to help you organize your day, manage tasks, and track meetings without ever leaving your terminal. It stores everything in simple Markdown files, making your data portable and easy to edit.

## ✨ Features

- **📝 Daily Journaling**: Add, list, and manage tasks for any date.
- **🎨 Beautiful UI**: Modern terminal interface with progress bars, icons, and colors.
- **📅 Calendar Views**:
  - **Daily View**: See your tasks with priorities, tags, and notes.
  - **Weekly View**: Visualize your week with a timeline-style layout.
  - **Monthly Calendar**: Overview of your month with activity markers.
- **⚡ Priorities & Tags**: Organize tasks with High (▲), Medium (▵), and Low (▽) priorities, and group them with `#tags`.
- **🤝 Meeting Management**: Schedule meetings, track durations, and get notifications.
- **🔄 Migration**: Easily move unfinished tasks to the next day or a specific date.
- **🔔 Notifications**: Optional systemd integration for meeting reminders and daily prompts.
- **💾 Markdown Storage**: Your data belongs to you. Everything is stored as standard Markdown.

## 🚀 Installation

### Prerequisites

You need to have **Rust** installed. If you don't have it, install it easily:

- **Linux / macOS**:
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```
- **Windows**:
  Download and run [rustup-init.exe](https://win.rustup.rs/).

### Install `bj`

Once Rust is installed, clone the repo and install:

#### Linux & macOS

```bash
# Clone the repository
git clone https://github.com/ankur-gupta-29/bullet-journal.git
cd bullet-journal

# Install
cargo install --path .
```

**Add Alias (Recommended):**

To use `bj` instead of `bullet-journal`, run the command for your shell:

**Bash:**
```bash
echo 'alias bj="bullet-journal"' >> ~/.bashrc
source ~/.bashrc
```

**Zsh (macOS default):**
```bash
echo 'alias bj="bullet-journal"' >> ~/.zshrc
source ~/.zshrc
```

#### Windows (PowerShell)

```powershell
# Clone the repository
git clone https://github.com/ankur-gupta-29/bullet-journal.git
cd bullet-journal

# Install
cargo install --path .
```

**Add Alias:**

Run this in PowerShell to make the `bj` alias permanent:

```powershell
# Create profile if it doesn't exist and add alias
if (!(Test-Path $PROFILE)) { New-Item -Type File -Path $PROFILE -Force }
Add-Content $PROFILE "`nSet-Alias -Name bj -Value bullet-journal"

# Reload profile
. $PROFILE
```

## 📖 Usage

### 1. Managing Tasks

```bash
# Add a task for today
bj add "Review Pull Requests"

# Add a task with priority, tags, and notes
bj add "Write Documentation" -p high -t work -t docs -n "Focus on API" -n "Include examples"

# Add a task to a specific date
bj add -d 2025-12-01 "Plan Q1 Roadmap"

# Mark a task as done (by ID)
bj done 1

# Delete a task
bj delete 2
```

### 2. Meetings

```bash
# Add a meeting (default duration 60m)
bj meeting add -t 10:00 "Daily Standup"

# Add a meeting with duration and tags
bj meeting add -t 14:00 -u 30 -g work "Design Review"

# List today's meetings
bj meeting list
```

### 3. Views

```bash
# List today's tasks and meetings
bj list

# Show the weekly timeline
bj week

# Show the monthly calendar
bj cal
```

### 4. Task Migration

```bash
# Move all open tasks from yesterday to today
bj migrate

# Move tasks from a specific date
bj migrate --from 2025-11-20

# Move a specific task to another date
bj migrate --from 2025-11-20 --to 2025-11-25 --id 3
```

## 🖼️ Visuals

**Daily List View:**
```text
╭──────────────────────────────────────────────────╮
│                  BULLET JOURNAL                  │
├──────────────────────────────────────────────────┤
│ 📅 Today                                          │
│ ━━━━━━━━━━━━━━━━━━━━ 16%                         │
│ ✓ 1/6 done  •  🗓 2 mtgs                          │
╰──────────────────────────────────────────────────╯

  1 ○ ▲          Review PRs   work
  2 ●            Lunch with team   social
  3 ○ ▵          Write documentation   work
       ├── Focus on API docs
       └── Include examples
  4 ○ ▽          Buy groceries   personal
  5 ○   🕒 10:00 Daily Standup   work
```

**Monthly Calendar:**
```text
╭──────────────────────────────────────────╮
│              November 2025               │
├──────────────────────────────────────────┤
│  Mo    Tu    We    Th    Fr    Sa    Su  │
│                                1     2   │
│  3     4     5     6     7     8     9   │
│ 10    11    12    13    14    15    16   │
│ 17    18    19    20    21    22    23   │
│ 24    25•   26    27    28    29    30   │
╰──────────────────────────────────────────╯
```

## ⚙️ Configuration & Data

- **Data Location**: `~/.local/share/bullet_journal/YYYY-MM-DD.md`
- **Format**: Standard Markdown. You can edit files manually if you prefer!

## 🤖 Automation (Optional)

### Meeting Notifications
Get notified 15 minutes before a meeting starts.

```bash
# Create systemd service and timer
# (See 'Meeting notifier' section in previous docs for full script)
bj meeting notify -w 15
```

## 📄 License

MIT License.
