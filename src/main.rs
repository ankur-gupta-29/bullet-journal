use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use chrono::{Datelike, Local, NaiveDate, NaiveTime};
use colored::Colorize;
use clap::{Parser, Subcommand};
use directories::ProjectDirs;

#[derive(Parser)]
#[command(
    name = "bj",
    version,
    about = "Bullet journal CLI",
    long_about = "A fast terminal bullet journal that stores Markdown per day.\n\nFeatures:\n- Add/list/done/delete/migrate bullets with priority, tags, notes\n- Week and month calendar views\n- Meetings: add/list/notify with start time and duration\n- Optional daily and meeting notifications (systemd user timers)",
    after_help = "Examples:\n  bj add \"Draft project plan\"\n  bj add -p high -t work -n \"prep\" \"Release train\"\n  bj list -t work -p 3\n  bj done 2\n  bj delete 3\n  bj migrate --from 2025-11-04\n  bj migrate --from 2025-11-04 --to 2025-11-10\n  bj migrate --from 2025-11-04 --to 2025-11-10 --id 2\n  bj week -t work\n  bj cal\n  bj meeting add -t 15:00 -u 30 \"Team sync\"\n  bj meeting list\n  bj meeting notify -w 15"
)] 
struct Cli {
	#[command(subcommand)]
	action: Action,
}

#[derive(Subcommand)]
enum Action {
	/// Add a bullet to a date (default today)
	Add {
		/// Bullet text
		text: Vec<String>,
		/// Date YYYY-MM-DD (default: today)
		#[arg(short = 'd', long = "date")]
		date: Option<String>,
		/// Priority: low, med, high (or 1/2/3)
		#[arg(short = 'p', long = "priority")]
		priority: Option<String>,
		/// One or more tags
		#[arg(short = 't', long = "tag")]
		tags: Vec<String>,
		/// Optional note lines (can repeat)
		#[arg(short = 'n', long = "note")]
		notes: Vec<String>,
	},
	/// List bullets for a date (default today)
	List {
		/// Date YYYY-MM-DD (default: today)
		#[arg(short = 'd', long = "date")]
		date: Option<String>,
		/// Filter by tag (can repeat)
		#[arg(short = 't', long = "tag")]
		tags: Vec<String>,
		/// Filter by priority: low, med, high (or 1/2/3)
		#[arg(short = 'p', long = "priority")]
		priority: Option<String>,
	},
	/// Mark a bullet done by ID for a date (default today)
	Done {
		/// Bullet ID (1-based visible index)
		id: usize,
		/// Date YYYY-MM-DD (default: today)
		#[arg(short = 'd', long = "date")]
		date: Option<String>,
	},
	/// Delete a bullet or meeting by ID for a date (default today)
	Delete {
		/// Bullet or meeting ID (1-based visible index)
		id: usize,
		/// Date YYYY-MM-DD (default: today)
		#[arg(short = 'd', long = "date")]
		date: Option<String>,
	},
	/// Migrate all open bullets from a date to another date (default: from yesterday to today)
	Migrate {
		/// Source date YYYY-MM-DD (default: yesterday)
		#[arg(long = "from")]
		from: Option<String>,
		/// Target date YYYY-MM-DD (default: today)
		#[arg(long = "to")]
		to: Option<String>,
		/// Optional bullet ID to migrate (1-based). If omitted, all open bullets are migrated.
		#[arg(long = "id")]
		id: Option<usize>,
	},
	/// Show a weekly view for the week containing date (default: today)
	Week {
		/// Any date in the target week YYYY-MM-DD (default: today)
		#[arg(short = 'd', long = "date")]
		date: Option<String>,
		/// Filter by tag (can repeat)
		#[arg(short = 't', long = "tag")]
		tags: Vec<String>,
		/// Filter by priority: low, med, high (or 1/2/3)
		#[arg(short = 'p', long = "priority")]
		priority: Option<String>,
	},
	/// Manage meetings: add/list/notify
	Meeting {
		#[command(subcommand)]
		cmd: MeetingCmd,
	},
	/// Show a month calendar with markers for bullets/meetings
	Cal {
		/// Any date in the month (default today)
		#[arg(short = 'd', long = "date")]
		date: Option<String>,
	},
}

#[derive(Subcommand)]
enum MeetingCmd {
	/// Add a meeting
	Add {
		/// Title
		title: Vec<String>,
		/// Date YYYY-MM-DD (default: today)
		#[arg(short = 'd', long = "date")]
		date: Option<String>,
		/// Start time HH:MM (24h)
		#[arg(short = 't', long = "time")]
		time: String,
		/// Duration minutes
		#[arg(short = 'u', long = "duration", default_value_t = 60)]
		duration: u32,
		/// Tags
		#[arg(short = 'g', long = "tag")]
		tags: Vec<String>,
		/// Notes
		#[arg(short = 'n', long = "note")]
		notes: Vec<String>,
	},
	/// List meetings for a date (default today)
	List {
		#[arg(short = 'd', long = "date")]
		date: Option<String>,
	},
	/// Send notifications for meetings starting within N minutes (default 15)
	Notify {
		#[arg(short = 'w', long = "window", default_value_t = 15)]
		window_minutes: i64,
	},
}

fn main() -> Result<()> {
	let cli = Cli::parse();
	match cli.action {
		Action::Add { text, date, priority, tags, notes } => {
			let date = parse_or_today(date.as_deref())?;
			let pr = parse_priority_opt(priority.as_deref())?;
			add_bullet(date, &text.join(" "), pr, &tags, &notes)?
		}
		Action::List { date, tags, priority } => {
			let date = parse_or_today(date.as_deref())?;
			let pr = parse_priority_opt(priority.as_deref())?;
			list_bullets(date, &tags, pr)?
		}
		Action::Done { id, date } => {
			let date = parse_or_today(date.as_deref())?;
			mark_done(date, id)?
		}
		Action::Delete { id, date } => {
			let date = parse_or_today(date.as_deref())?;
			delete_bullet(date, id)?
		}
		Action::Migrate { from, to, id } => {
			let from_date = match from {
				Some(d) => parse_date(&d)?,
				None => {
					let today = Local::now().date_naive();
					today.pred_opt().context("cannot compute yesterday")?
				}
			};
			let to_date = parse_or_today(to.as_deref())?;
			if let Some(bid) = id {
				migrate_one(from_date, to_date, bid)?;
			} else {
				migrate_open(from_date, to_date)?;
			}
		}
		Action::Week { date, tags, priority } => {
			let base = parse_or_today(date.as_deref())?;
			let pr = parse_priority_opt(priority.as_deref())?;
			week_view(base, &tags, pr)?
		}
		Action::Meeting { cmd } => match cmd {
			MeetingCmd::Add { title, date, time, duration, tags, notes } => {
				let date = parse_or_today(date.as_deref())?;
				let time = NaiveTime::parse_from_str(&time, "%H:%M").with_context(|| format!("invalid time: {}", time))?;
				add_meeting(date, time, duration, &title.join(" "), &tags, &notes)?
			}
			MeetingCmd::List { date } => {
				let date = parse_or_today(date.as_deref())?;
				list_meetings(date)?
			}
			MeetingCmd::Notify { window_minutes } => {
				notify_upcoming_meetings(window_minutes)?
			}
		},
		Action::Cal { date } => {
			let base = parse_or_today(date.as_deref())?;
			month_calendar(base)?
		}
	}
	Ok(())
}

fn parse_or_today(s: Option<&str>) -> Result<NaiveDate> {
	match s {
		Some(v) => parse_date(v),
		None => Ok(Local::now().date_naive()),
	}
}

fn parse_date(s: &str) -> Result<NaiveDate> {
	NaiveDate::parse_from_str(s, "%Y-%m-%d").with_context(|| format!("invalid date: {}", s))
}

fn data_dir() -> Result<PathBuf> {
	let proj = ProjectDirs::from("dev", "local", "bullet_journal").context("cannot resolve project dirs")?;
	let dir = proj.data_dir().to_path_buf();
	fs::create_dir_all(&dir).with_context(|| format!("create data dir {}", dir.display()))?;
	Ok(dir)
}

fn file_for(date: NaiveDate) -> Result<PathBuf> {
	let dir = data_dir()?;
	let fname = format!("{}-{:02}-{:02}.md", date.year(), date.month(), date.day());
	Ok(dir.join(fname))
}

#[derive(Debug, Clone)]
struct Bullet {
	line_index: usize, // index in file content lines
	visible_index: usize, // 1-based index among bullet lines
	completed: bool,
	text: String,
	priority: Option<u8>,
	tags: Vec<String>,
	notes: Vec<String>,
	meeting_time: Option<NaiveTime>,
	meeting_duration_min: Option<u32>,
}

fn read_file_lines(path: &Path) -> Result<Vec<String>> {
	if !path.exists() {
		return Ok(vec![]);
	}
	let mut f = OpenOptions::new().read(true).open(path).with_context(|| format!("open {}", path.display()))?;
	let mut s = String::new();
	f.read_to_string(&mut s).with_context(|| format!("read {}", path.display()))?;
	Ok(s.lines().map(|l| l.to_string()).collect())
}

fn write_file_lines(path: &Path, lines: &[String]) -> Result<()> {
	let mut f = OpenOptions::new().create(true).truncate(true).write(true).open(path).with_context(|| format!("write {}", path.display()))?;
	let contents = if lines.is_empty() { String::new() } else { format!("{}\n", lines.join("\n")) };
	f.write_all(contents.as_bytes()).with_context(|| format!("write {}", path.display()))?;
	Ok(())
}

fn parse_bullets(lines: &[String]) -> Vec<Bullet> {
	let mut out = Vec::new();
	let mut visible = 0usize;
	let mut idx = 0usize;
	while idx < lines.len() {
		let line = &lines[idx];
		let trimmed = line.trim_start();
		if let Some(rest) = trimmed.strip_prefix("- [ ] ") {
			visible += 1;
			let (text, pr, tags, mt, dur) = parse_text_meeting_meta(rest);
			let notes = collect_notes(lines, idx + 1);
			out.push(Bullet { line_index: idx, visible_index: visible, completed: false, text, priority: pr, tags, notes, meeting_time: mt, meeting_duration_min: dur });
		} else if let Some(rest) = trimmed.strip_prefix("- [x] ") {
			visible += 1;
			let (text, pr, tags, mt, dur) = parse_text_meeting_meta(rest);
			let notes = collect_notes(lines, idx + 1);
			out.push(Bullet { line_index: idx, visible_index: visible, completed: true, text, priority: pr, tags, notes, meeting_time: mt, meeting_duration_min: dur });
		}
		idx += 1;
	}
	out
}

fn collect_notes(lines: &[String], mut from: usize) -> Vec<String> {
	let mut notes = Vec::new();
	while from < lines.len() {
		let l = &lines[from];
		if let Some(n) = l.strip_prefix("  - note: ") {
			notes.push(n.to_string());
			from += 1;
			continue;
		}
		break;
	}
	notes
}

fn parse_text_meta_only(rest: &str) -> (String, Option<u8>, Vec<String>) {
	let mut text = rest.to_string();
	let mut pr = None;
	if let Some(stripped) = text.strip_prefix("(!!!) ") {
		pr = Some(3);
		text = stripped.to_string();
	} else if let Some(stripped) = text.strip_prefix("(!!) ") {
		pr = Some(2);
		text = stripped.to_string();
	} else if let Some(stripped) = text.strip_prefix("(!) ") {
		pr = Some(1);
		text = stripped.to_string();
	}
	let mut tags = Vec::new();
	let parts: Vec<&str> = text.split_whitespace().collect();
	let mut kept: Vec<&str> = Vec::new();
	for p in parts {
		if let Some(t) = p.strip_prefix('#') {
			if !t.is_empty() { tags.push(t.to_string()); }
		} else {
			kept.push(p);
		}
	}
	let final_text = kept.join(" ");
	(final_text, pr, tags)
}

fn parse_text_meeting_meta(rest: &str) -> (String, Option<u8>, Vec<String>, Option<NaiveTime>, Option<u32>) {
	let mut remaining = rest.to_string();
	let mut meeting_time: Option<NaiveTime> = None;
	let mut duration: Option<u32> = None;
	// Meeting prefix format: [mtg HH:MM] or [mtg HH:MM D]
	if let Some(body) = remaining.strip_prefix("[mtg ") {
		if let Some(close_idx) = body.find(']') {
			let spec = &body[..close_idx];
			let after = &body[close_idx+1..];
			let parts: Vec<&str> = spec.split_whitespace().collect();
			if !parts.is_empty() {
				if let Ok(t) = NaiveTime::parse_from_str(parts[0], "%H:%M") { meeting_time = Some(t); }
				if parts.len() > 1 { if let Ok(d) = parts[1].parse::<u32>() { duration = Some(d); } }
			}
			remaining = after.trim_start().to_string();
		}
	}
	let (text, pr, tags) = parse_text_meta_only(&remaining);
	(text, pr, tags, meeting_time, duration)
}

fn add_bullet(date: NaiveDate, text: &str, priority: Option<u8>, tags: &[String], notes: &[String]) -> Result<()> {
	let path = file_for(date)?;
	let mut lines = read_file_lines(&path)?;
	let mut prefix = String::new();
	match priority {
		Some(3) => prefix.push_str("(!!!) "),
		Some(2) => prefix.push_str("(!!) "),
		Some(1) => prefix.push_str("(!) "),
		_ => {}
	}
	let mut suffix = String::new();
	if !tags.is_empty() {
		for t in tags { suffix.push_str(&format!(" #{}", t)); }
	}
	let new_line = format!("- [ ] {}{}{}", prefix, text.trim(), suffix);
	lines.push(new_line);
	for n in notes {
		lines.push(format!("  - note: {}", n));
	}
	write_file_lines(&path, &lines)?;
	println!("Added to {}", path.display());
	Ok(())
}

fn add_meeting(date: NaiveDate, time: NaiveTime, duration_min: u32, title: &str, tags: &[String], notes: &[String]) -> Result<()> {
	let mut mt_prefix = format!("[mtg {} {}] ", time.format("%H:%M"), duration_min);
	let full = format!("{}{}", mt_prefix, title);
	add_bullet(date, &full, None, tags, notes)
}

fn list_meetings(date: NaiveDate) -> Result<()> {
	let path = file_for(date)?;
	let lines = read_file_lines(&path)?;
	let mut bullets = parse_bullets(&lines)
		.into_iter()
		.filter(|b| b.meeting_time.is_some())
		.collect::<Vec<_>>();
	if bullets.is_empty() { println!("No meetings for {}", date); return Ok(()); }
	bullets.sort_by_key(|b| b.meeting_time);
	for b in bullets {
		let t = b.meeting_time.unwrap();
		let dur = b.meeting_duration_min.unwrap_or(60);
		println!("{} {:>5} ({}m) {}", date, t.format("%H:%M"), dur, b.text);
	}
	Ok(())
}

fn migrate_one(from: NaiveDate, to: NaiveDate, id: usize) -> Result<()> {
	if from == to { bail!("from and to dates are the same; nothing to migrate"); }
	let from_path = file_for(from)?;
	let mut from_lines = read_file_lines(&from_path)?;
	let bullets = parse_bullets(&from_lines);
	let Some(target) = bullets.iter().find(|b| b.visible_index == id) else { bail!("bullet {} not found on {}", id, from) };
	if target.completed { bail!("bullet {} is already completed", id); }
	// reconstruct text without leading marker
	let raw = from_lines[target.line_index].clone();
	let text = raw.trim_start().trim_start_matches("- [ ] ").to_string();
	let (text, pr, tags, mt, dur) = parse_text_meeting_meta(&text);
	let mut full_text = String::new();
	if let Some(t) = mt { full_text.push_str(&format!("[mtg {}{}] ", t.format("%H:%M"), dur.map(|d| format!(" {}", d)).unwrap_or_default())); }
	full_text.push_str(&text);
	add_bullet(to, &full_text, pr, &tags, &[])?;
	from_lines.remove(target.line_index);
	write_file_lines(&from_path, &from_lines)?;
	println!("{}", format!("Migrated bullet {} from {} to {}", id, from, to).green());
	Ok(())
}

// Backward compatibility wrapper
fn migrate_one_to_today(from: NaiveDate, id: usize) -> Result<()> {
	let to = Local::now().date_naive();
	migrate_one(from, to, id)
}

fn notified_state_path() -> Result<PathBuf> { Ok(data_dir()?.join("notified.meetings")) }

fn notify_upcoming_meetings(window_minutes: i64) -> Result<()> {
	let today = Local::now().date_naive();
	let now = Local::now().time();
	let path = file_for(today)?;
	let lines = read_file_lines(&path)?;
	let bullets = parse_bullets(&lines);
	let mut state_path = notified_state_path()?;
	let mut sent = std::collections::HashSet::new();
	if state_path.exists() {
		let s = fs::read_to_string(&state_path).unwrap_or_default();
		for line in s.lines() { sent.insert(line.to_string()); }
	}
	let mut new_sent: Vec<String> = Vec::new();
	for b in bullets {
		let Some(t) = b.meeting_time else { continue };
		let start_key = format!("{}|{}", today, t.format("%H:%M"));
		if sent.contains(&start_key) { continue; }
		let diff = (t - now).num_minutes();
		if diff >= 0 && diff <= window_minutes {
			let title = "Upcoming meeting";
			let msg = format!("{} at {} (in {} min)", b.text, t.format("%H:%M"), diff);
			if which::which("notify-send").is_ok() {
				let _ = std::process::Command::new("notify-send").arg(title).arg(msg).status();
			} else {
				println!("{}: {}", title, msg);
			}
			new_sent.push(start_key);
		}
	}
	if !new_sent.is_empty() {
		let mut contents = String::new();
		for k in sent { contents.push_str(&format!("{}\n", k)); }
		for k in new_sent { contents.push_str(&format!("{}\n", k)); }
		fs::write(&state_path, contents).ok();
	}
	Ok(())
}

fn list_bullets(date: NaiveDate, filter_tags: &[String], filter_priority: Option<u8>) -> Result<()> {
	let path = file_for(date)?;
	let lines = read_file_lines(&path)?;
	let bullets = parse_bullets(&lines);
	
	if bullets.is_empty() {
		println!("\n{} {}", "📭".normal(), format!("No bullets for {}", date).dimmed());
		return Ok(());
	}
	
	// Get today's date
	let today = Local::now().date_naive();
	
	// Count tasks
	let total = bullets.len();
	let completed = bullets.iter().filter(|b| b.completed).count();
	let meetings = bullets.iter().filter(|b| b.meeting_time.is_some()).count();
	
	// Progress bar
	let pct = if total > 0 { (completed as f64 / total as f64 * 100.0) as usize } else { 0 };
	let bars = 20;
	let filled = if total > 0 { (completed * bars) / total } else { 0 };
	let empty = bars - filled;
	let progress_bar = format!("{}{}", "━".repeat(filled).green(), "━".repeat(empty).bright_black());
	
	// Box width
	let box_width = 50;
	
	// Date string
	let date_str = if date == today {
		"Today".to_string()
	} else if date == today.pred_opt().unwrap_or(date) {
		"Yesterday".to_string()
	} else if date == today.succ_opt().unwrap_or(date) {
		"Tomorrow".to_string()
	} else {
		date.format("%A, %B %d").to_string()
	};
	
	// Header
	println!("\n{}", format!("╭{:─<width$}╮", "", width = box_width).bright_black());
	
	// Title centered
	let title = "BULLET JOURNAL";
	let pad_left = (box_width - title.len()) / 2;
	let pad_right = box_width - title.len() - pad_left;
	println!("│{}{}{}│", 
		" ".repeat(pad_left),
		title.bold().magenta(),
		" ".repeat(pad_right)
	);
	
	println!("{}", format!("├{:─<width$}┤", "", width = box_width).bright_black());
	
	// Date line
	let date_display = format!("📅 {}", date_str);
	let date_len = date_display.chars().count(); 
	let date_pad = if box_width > date_len { box_width - date_len - 2 } else { 0 };
	println!("│ {}{} │", date_display.bold().cyan(), " ".repeat(date_pad));
	
	// Stats line
	let stats_display = format!("{} {}%", progress_bar, pct);
	let stats_len = bars + 1 + pct.to_string().len() + 1; // bars + space + pct + %
	let stats_pad = if box_width > stats_len { box_width - stats_len - 2 } else { 0 };
	println!("│ {}{} │", stats_display, " ".repeat(stats_pad));
	
	let summary = format!("✓ {}/{} done  •  🗓 {} mtgs", completed, total, meetings);
	let sum_len = summary.chars().count();
	let sum_pad = if box_width > sum_len { box_width - sum_len - 2 } else { 0 };
	println!("│ {}{} │", summary.italic().dimmed(), " ".repeat(sum_pad));
	
	println!("{}", format!("╰{:─<width$}╯", "", width = box_width).bright_black());
	println!();
	
	for b in bullets {
		if let Some(p) = filter_priority { if b.priority != Some(p) { continue; } }
		if !filter_tags.is_empty() {
			if !filter_tags.iter().all(|t| b.tags.iter().any(|bt| bt == t)) { continue; }
		}
		
		// Fancy Checkbox
		let checkbox = if b.completed { "●".green() } else { "○".bright_black() };
		
		// Priority with different style
		let priority_icon = match b.priority {
			Some(3) => "▲".red(),
			Some(2) => "▵".yellow(),
			Some(1) => "▽".green(),
			_ => " ".normal(),
		};
		
		// Time with clock icon
		let time_str = if let Some(t) = b.meeting_time {
			format!("{} {}", "🕒".cyan(), t.format("%H:%M").to_string().cyan())
		} else {
			"        ".normal().to_string()
		};
		
		// Tags as badges
		let tags_str = if b.tags.is_empty() { String::new() } else { 
			format!(" {}", b.tags.iter().map(|t| format!("{}", t)).collect::<Vec<_>>().join(" "))
		};
		
		let idx = format!("{:>2}", b.visible_index).dimmed();
		let text = if b.completed { b.text.dimmed().strikethrough() } else { b.text.bold() };
		
		// Main line
		println!(" {} {} {} {} {}{}", 
			idx, 
			checkbox, 
			priority_icon, 
			time_str, 
			text, 
			if b.tags.is_empty() { "".normal() } else { format!("  {}", tags_str).blue().italic() }
		);
		
		// Notes with nice tree structure
		let last_note_idx = b.notes.len().saturating_sub(1);
		for (i, n) in b.notes.iter().enumerate() {
			let connector = if i == last_note_idx { "└──" } else { "├──" };
			println!("       {} {}", connector.bright_black(), n.dimmed());
		}
	}
	println!();
	Ok(())
}

fn mark_done(date: NaiveDate, id: usize) -> Result<()> {
	let path = file_for(date)?;
	let mut lines = read_file_lines(&path)?;
	let bullets = parse_bullets(&lines);
	let Some(target) = bullets.iter().find(|b| b.visible_index == id) else { bail!("bullet {} not found", id) };
	let raw = &lines[target.line_index];
	let replaced = if raw.trim_start().starts_with("- [ ] ") {
		raw.replacen("- [ ] ", "- [x] ", 1)
	} else if raw.trim_start().starts_with("- [x] ") {
		raw.to_string()
	} else {
		raw.to_string()
	};
	lines[target.line_index] = replaced;
	write_file_lines(&path, &lines)?;
	println!("Marked done: {} #{}", date, id);
	Ok(())
}

fn delete_bullet(date: NaiveDate, id: usize) -> Result<()> {
	let path = file_for(date)?;
	let mut lines = read_file_lines(&path)?;
	let bullets = parse_bullets(&lines);
	let Some(target) = bullets.iter().find(|b| b.visible_index == id) else { bail!("bullet {} not found", id) };
	
	// Get the text for confirmation message before deleting
	let bullet_text = target.text.clone();
	
	// Remove the bullet line and any associated note lines
	let mut lines_to_remove = vec![target.line_index];
	
	// Find note lines that belong to this bullet (indented lines immediately following)
	for i in (target.line_index + 1)..lines.len() {
		let line = &lines[i];
		if line.trim().is_empty() {
			// Empty line might separate bullets, keep looking
			continue;
		} else if line.starts_with("  ") && !line.trim_start().starts_with("- ") {
			// This is an indented note line
			lines_to_remove.push(i);
		} else {
			// Hit the next bullet or non-note content
			break;
		}
	}
	
	// Remove lines in reverse order to maintain indices
	for &idx in lines_to_remove.iter().rev() {
		lines.remove(idx);
	}
	
	write_file_lines(&path, &lines)?;
	println!("Deleted: {} #{} - \"{}\"", date, id, bullet_text);
	Ok(())
}

fn migrate_open(from: NaiveDate, to: NaiveDate) -> Result<()> {
	if from == to { bail!("from and to dates are the same; nothing to migrate"); }
	let from_path = file_for(from)?;
	let mut from_lines = read_file_lines(&from_path)?;
	let bullets = parse_bullets(&from_lines);
	let mut moved_any = false;
	for b in bullets.into_iter().rev() { // reverse so removals do not shift earlier indexes
		if !b.completed {
			let raw = from_lines[b.line_index].clone();
			let text = raw.trim_start().trim_start_matches("- [ ] ").to_string();
			let (text, pr, tags, mt, dur) = parse_text_meeting_meta(&text);
			// Preserve meeting marker if present by reconstructing text with meeting prefix
			let mut full_text = String::new();
			if let Some(t) = mt { full_text.push_str(&format!("[mtg {}{}] ", t.format("%H:%M"), dur.map(|d| format!(" {}", d)).unwrap_or_default())); }
			full_text.push_str(&text);
			add_bullet(to, &full_text, pr, &tags, &[])?;
			from_lines.remove(b.line_index);
			moved_any = true;
		}
	}
	write_file_lines(&from_path, &from_lines)?;
	if moved_any {
		println!("Migrated open bullets from {} to {}", from, to);
	} else {
		println!("No open bullets to migrate from {}", from);
	}
	Ok(())
}

// Backward compatibility wrapper
fn migrate_open_to_today(from: NaiveDate) -> Result<()> {
	let to = Local::now().date_naive();
	migrate_open(from, to)
}

fn parse_priority_opt(v: Option<&str>) -> Result<Option<u8>> {
	match v {
		None => Ok(None),
		Some(s) => {
			let s = s.to_lowercase();
			let p = match s.as_str() {
				"3" | "high" | "h" => 3,
				"2" | "med" | "m" | "medium" => 2,
				"1" | "low" | "l" => 1,
				_ => bail!("invalid priority: {}", s),
			};
			Ok(Some(p))
		}
	}
}

fn week_view(base: NaiveDate, filter_tags: &[String], filter_priority: Option<u8>) -> Result<()> {
	let weekday = base.weekday().num_days_from_monday() as i64;
	let start = base - chrono::Days::new(weekday as u64);
	
	// Header for the week
	let end = start + chrono::Days::new(6);
	println!("\n{}", format!("Week: {} - {}", start.format("%b %d"), end.format("%b %d")).bold().underline());
	
	for i in 0..7 {
		let day = start + chrono::Days::new(i);
		let path = file_for(day)?;
		let lines = read_file_lines(&path)?;
		let bullets = parse_bullets(&lines);
		
		let is_today = day == Local::now().date_naive();
		let day_header = format!("{}", day.format("%A, %b %d"));
		
		// Day header with separator
		if is_today {
			println!("\n{} {}", "●".cyan(), day_header.bold().black().on_cyan());
		} else {
			println!("\n{} {}", "○".bright_black(), day_header.bold().cyan());
		}
		
		if bullets.is_empty() {
			println!("   {}", "No tasks".dimmed().italic());
			continue;
		}
		
		for b in bullets {
			if let Some(p) = filter_priority { if b.priority != Some(p) { continue; } }
			if !filter_tags.is_empty() {
				if !filter_tags.iter().all(|t| b.tags.iter().any(|bt| bt == t)) { continue; }
			}
			
			let checkbox = if b.completed { "●".green() } else { "○".bright_black() };
			let priority_icon = match b.priority {
				Some(3) => "▲".red(),
				Some(2) => "▵".yellow(),
				Some(1) => "▽".green(),
				_ => " ".normal(),
			};
			
			let time_str = if let Some(t) = b.meeting_time {
				format!("{} ", t.format("%H:%M")).cyan().to_string()
			} else {
				"      ".normal().to_string()
			};
			
			let tags_str = if b.tags.is_empty() { String::new() } else { 
				format!(" {}", b.tags.iter().map(|t| format!("{}", t)).collect::<Vec<_>>().join(" "))
			};
			
			let text = if b.completed { b.text.dimmed().strikethrough() } else { b.text.normal() };
			
			println!("   {} {} {} {}{}", checkbox, priority_icon, time_str, text, if b.tags.is_empty() { "".normal() } else { format!("  {}", tags_str).blue().italic() });
			
			let last_note_idx = b.notes.len().saturating_sub(1);
			for (i, n) in b.notes.iter().enumerate() {
				let connector = if i == last_note_idx { "└──" } else { "├──" };
				println!("         {} {}", connector.bright_black(), n.dimmed());
			}
		}
	}
	println!();
	Ok(())
}

fn month_calendar(base: NaiveDate) -> Result<()> {
	let today = Local::now().date_naive();
	let first = NaiveDate::from_ymd_opt(base.year(), base.month(), 1).context("invalid month")?;
	let next_month = if base.month() == 12 { 
		NaiveDate::from_ymd_opt(base.year()+1, 1, 1).unwrap() 
	} else { 
		NaiveDate::from_ymd_opt(base.year(), base.month()+1, 1).unwrap() 
	};
	let last_day = (next_month - chrono::Days::new(1)).day();
	
	let month_name = base.format("%B").to_string();
	let header_text = format!("{} {}", month_name, base.year());
	
	// Width calculation: 7 days * 6 chars + 2 border = 44
	let width = 42;
	let pad_left = (width - header_text.len()) / 2;
	let pad_right = width - header_text.len() - pad_left;
	
	println!("\n{}", format!("╭{:─<width$}╮", "", width = width).bright_black());
	println!("│{}{}{}│", " ".repeat(pad_left), header_text.bold().cyan(), " ".repeat(pad_right));
	println!("{}", format!("├{:─<width$}┤", "", width = width).bright_black());
	
	// Correctly spaced header: 6 chars per day
	println!("│{:^6}{:^6}{:^6}{:^6}{:^6}{:^6}{:^6}│", 
		"Mo".bold(), "Tu".bold(), "We".bold(), "Th".bold(), 
		"Fr".bold(), "Sa".bold().bright_blue(), "Su".bold().bright_blue());
	
	let offset = first.weekday().num_days_from_monday();
	let mut col = 0;
	
	print!("│");
	for _ in 0..offset { print!("      "); col += 1; }
	
	let mut d = 1u32;
	while d <= last_day {
		let cur = NaiveDate::from_ymd_opt(base.year(), base.month(), d).unwrap();
		let path = file_for(cur)?;
		let lines = read_file_lines(&path)?;
		let bullets = parse_bullets(&lines);
		
		let has_meeting = bullets.iter().any(|b| b.meeting_time.is_some());
		let has_open = bullets.iter().any(|b| !b.completed);
		let all_done = !bullets.is_empty() && bullets.iter().all(|b| b.completed);
		
		let marker = if has_meeting { "•".red() }
		else if has_open { "•".yellow() }
		else if all_done { "•".green() }
		else { " ".normal() };
		
		let day_str = if cur == today {
			format!("{:>2}", d).bold().white().on_blue()
		} else if cur.weekday().number_from_monday() >= 6 {
			format!("{:>2}", d).bright_blue()
		} else {
			format!("{:>2}", d).normal()
		};
		
		print!(" {}{}  ", day_str, marker);
		col += 1;
		
		if col == 7 {
			println!("│");
			if d < last_day { print!("│"); }
			col = 0;
		}
		d += 1;
	}
	
	if col > 0 {
		while col < 7 { print!("      "); col += 1; }
		println!("│");
	}
	
	println!("{}", format!("╰{:─<width$}╯", "", width = width).bright_black());
	
	// Legend
	println!("\n {}", "Legend:".bold().underline());
	println!("  {} Meeting   {} Open task", "•".red(), "•".yellow());
	println!("  {} All done  {} Today", "•".green(), "12".bold().white().on_blue());
	println!();
	
	Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::env;
    use serial_test::serial;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static TEST_COUNTER: AtomicUsize = AtomicUsize::new(0);

    struct TestEnv {
        root: std::path::PathBuf,
        data_dir: std::path::PathBuf,
        _prev_xdg: Option<String>,
    }
    
    impl TestEnv {
        fn new() -> Self {
            // Save current XDG_DATA_HOME
            let prev_xdg = env::var("XDG_DATA_HOME").ok();
            
            // Create unique test directory using test counter
            let mut test_root = env::temp_dir();
            let test_num = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
            let uniq = format!("bj_test_{}_{}", 
                std::process::id(),
                test_num);
            test_root.push(uniq);
            fs::create_dir_all(&test_root).expect("create test root");
            
            // Create XDG data dir inside test root
            let data_home = test_root.join("data");
            let data_dir = data_home.join("bullet_journal");
            fs::create_dir_all(&data_dir).expect("create data dir");
            
            // Set XDG_DATA_HOME to our test directory
            env::set_var("XDG_DATA_HOME", data_home.to_str().unwrap());
            
            TestEnv {
                root: test_root,
                data_dir,
                _prev_xdg: prev_xdg,
            }
        }
    }
    
    impl Drop for TestEnv {
        fn drop(&mut self) {
            // Restore previous XDG_DATA_HOME
            match &self._prev_xdg {
                Some(prev) => env::set_var("XDG_DATA_HOME", prev),
                None => env::remove_var("XDG_DATA_HOME"),
            }
            // Clean up test directory
            fs::remove_dir_all(&self.root).ok();
        }
    }

    #[test]
    fn test_parse_text_meta_only() {
        // Test priority and tags
        let s = "(!!!) Test bullet #work #urgent";
        let (text, pr, tags) = parse_text_meta_only(s);
        assert_eq!(text, "Test bullet", "Text not correctly extracted");
        assert_eq!(pr, Some(3), "High priority not detected");
        assert_eq!(tags, vec!["work".to_string(), "urgent".to_string()], "Tags not correctly parsed");

        // Test medium priority
        let s = "(!!) Medium priority #dev";
        let (text, pr, tags) = parse_text_meta_only(s);
        assert_eq!(text, "Medium priority", "Text with medium priority not extracted");
        assert_eq!(pr, Some(2), "Medium priority not detected");
        assert_eq!(tags, vec!["dev".to_string()], "Single tag not parsed");

        // Test no metadata
        let s = "Simple bullet";
        let (text, pr, tags) = parse_text_meta_only(s);
        assert_eq!(text, "Simple bullet", "Plain text not preserved");
        assert_eq!(pr, None, "Should have no priority");
        assert!(tags.is_empty(), "Should have no tags");
    }

    #[test]
    fn test_parse_text_meeting_meta() {
        // Test full meeting metadata
        let s = "[mtg 15:30 45] Team sync #work";
        let (text, pr, tags, mt, dur) = parse_text_meeting_meta(s);
        assert_eq!(text, "Team sync", "Meeting text not extracted");
        assert_eq!(pr, None, "Should have no priority");
        assert_eq!(tags, vec!["work".to_string()], "Meeting tag not parsed");
        assert_eq!(mt.unwrap().format("%H:%M").to_string(), "15:30", "Meeting time not parsed");
        assert_eq!(dur, Some(45), "Meeting duration not parsed");

        // Test meeting without duration
        let s = "[mtg 09:00] Daily standup";
        let (text, _pr, _tags, mt, dur) = parse_text_meeting_meta(s);
        assert_eq!(text, "Daily standup", "Simple meeting text not extracted");
        assert_eq!(mt.unwrap().format("%H:%M").to_string(), "09:00", "Simple meeting time not parsed");
        assert_eq!(dur, None, "Should have no duration");

        // Test non-meeting text
        let s = "Regular bullet";
        let (text, _pr, _tags, mt, dur) = parse_text_meeting_meta(s);
        assert_eq!(text, "Regular bullet", "Non-meeting text should be preserved");
        assert!(mt.is_none(), "Non-meeting should have no time");
        assert!(dur.is_none(), "Non-meeting should have no duration");
    }

    #[test]
    #[serial]  // Prevent parallel test runs
    fn test_meeting_metadata() -> Result<()> {
        let _env = TestEnv::new();
        let date = NaiveDate::from_ymd_opt(2025, 11, 6).unwrap();
        
        // Add a meeting
        add_meeting(date, 
            NaiveTime::from_hms_opt(14, 30, 0).unwrap(),
            45,
            "Team Sync",
            &vec!["work".to_string()],
            &vec!["Prep required".to_string()]
        )?;
        
        // Verify the meeting was added correctly
        let path = file_for(date)?;
        let lines = read_file_lines(&path)?;
        let bullets = parse_bullets(&lines);
        assert_eq!(bullets.len(), 1, "Expected exactly one meeting bullet");
        
        let mtg = &bullets[0];
        assert_eq!(mtg.text, "Team Sync", "Meeting title mismatch");
        assert_eq!(mtg.meeting_time.unwrap().format("%H:%M").to_string(), "14:30", "Meeting time mismatch");
        assert_eq!(mtg.meeting_duration_min, Some(45), "Meeting duration mismatch");
        assert_eq!(mtg.tags, vec!["work"], "Meeting tag mismatch");
        assert_eq!(mtg.notes, vec!["Prep required"], "Meeting note mismatch");
        
        Ok(())
    }

    

    #[test]
    #[serial]  // Prevent parallel test runs 
    fn test_add_and_parse_bullet() -> Result<()> {
        let _env = TestEnv::new();
        let date = NaiveDate::from_ymd_opt(2025, 11, 6).unwrap();
        
        // Add a bullet with priority, tags, and notes
        add_bullet(date, "Write tests", Some(2), &vec!["dev".to_string()], &vec!["first note".to_string()])?;
        
        let path = file_for(date)?;
        let lines = read_file_lines(&path)?;
        let bullets = parse_bullets(&lines);
        
        assert_eq!(bullets.len(), 1, "Expected exactly one bullet");
        let b = &bullets[0];
        assert_eq!(b.text, "Write tests", "Bullet text mismatch");
        assert_eq!(b.priority, Some(2), "Priority mismatch");
        assert_eq!(b.tags, vec!["dev"], "Tags mismatch");
        assert_eq!(b.notes.len(), 1, "Expected one note");
        assert!(b.notes[0].contains("first note"), "Note content mismatch");
        
        Ok(())
    }

    #[test]
    #[serial]  // Prevent parallel test runs
    fn test_mark_done() -> Result<()> {
        let _env = TestEnv::new();
        let date = NaiveDate::from_ymd_opt(2025, 11, 6).unwrap();
        
        // Add two bullets
        add_bullet(date, "Task A", None, &vec![], &vec![])?;
        add_bullet(date, "Task B", None, &vec![], &vec![])?;
        
        // Parse to verify initial state
        let initial = parse_bullets(&read_file_lines(&file_for(date)?)?);
        assert_eq!(initial.len(), 2, "Expected two bullets initially");
        assert!(!initial[0].completed && !initial[1].completed, "Bullets should start incomplete");
        
        // Mark first one done
        mark_done(date, 1)?;
        
        let bullets = parse_bullets(&read_file_lines(&file_for(date)?)?);
        assert_eq!(bullets.len(), 2, "Should still have two bullets after marking one done");
        assert!(bullets[0].completed, "First bullet should be marked done");
        assert!(!bullets[1].completed, "Second bullet should still be incomplete");
        
        Ok(())
    }

    #[test]
    #[serial]  // Prevent parallel test runs
    fn test_migrate_one_to_today() -> Result<()> {
        let _env = TestEnv::new();
        let from = NaiveDate::from_ymd_opt(2025, 11, 4).unwrap();
        let today = Local::now().date_naive();
        
        // Create two bullets on source date with unique identifiable text
        add_bullet(from, "Source Bullet A", None, &vec![], &vec![])?;
        add_bullet(from, "Source Bullet B", Some(2), &vec!["important".to_string()], &vec![])?;
        
        // Read source file to find bullet indices
        let from_path = file_for(from)?;
        let initial_lines = read_file_lines(&from_path)?;
        let initial = parse_bullets(&initial_lines);
        assert_eq!(initial.len(), 2, "Should have two bullets initially");
        
        // Find B's index and migrate it
        let b_index = initial.iter()
            .find(|b| b.text == "Source Bullet B")
            .map(|b| b.visible_index)
            .expect("Should find bullet B");
        migrate_one_to_today(from, b_index)?;
        
        // Verify source file - should only have bullet A
        let source_after = parse_bullets(&read_file_lines(&from_path)?);
        assert_eq!(source_after.len(), 1, "Source should have one bullet remaining");
        assert_eq!(source_after[0].text, "Source Bullet A", "Wrong bullet removed from source");
        
        // Verify target file - should have bullet B with metadata
        let today_path = file_for(today)?;
        let target = parse_bullets(&read_file_lines(&today_path)?);
        assert_eq!(target.len(), 1, "Target should have one bullet");
        
        let migrated = &target[0];
        assert_eq!(migrated.text, "Source Bullet B", "Wrong bullet migrated");
        assert_eq!(migrated.priority, Some(2), "Priority not preserved");
        assert_eq!(migrated.tags, vec!["important"], "Tags not preserved");
        
        Ok(())
    }

    #[test]
    #[serial]  // Prevent parallel test runs
    fn test_migrate_open_to_today() -> Result<()> {
        let _env = TestEnv::new();
        let from = NaiveDate::from_ymd_opt(2025, 11, 3).unwrap();
        let today = Local::now().date_naive();
        
        // Add three bullets with unique identifiable text
        add_bullet(from, "First Task (Done)", None, &vec![], &vec![])?;
        add_bullet(from, "Second Task (Open)", Some(1), &vec!["tag1".to_string()], &vec![])?;
        add_bullet(from, "Third Task (Open)", Some(3), &vec!["tag2".to_string()], &vec![])?;
        
        // Mark first task done
        let from_path = file_for(from)?;
        let initial = parse_bullets(&read_file_lines(&from_path)?);
        let first_id = initial.iter()
            .find(|b| b.text == "First Task (Done)")
            .map(|b| b.visible_index)
            .expect("Should find first task");
        mark_done(from, first_id)?;
        
        // Migrate open tasks
        migrate_open_to_today(from)?;
        
        // Verify source - should only have done task
        let source_after = parse_bullets(&read_file_lines(&from_path)?);
        assert_eq!(source_after.len(), 1, "Source should have one bullet");
        assert!(source_after[0].completed, "Source bullet should be done");
        assert_eq!(source_after[0].text, "First Task (Done)", "Wrong task in source");
        
        // Verify target - should have both open tasks
        let today_path = file_for(today)?;
        let target = parse_bullets(&read_file_lines(&today_path)?);
        assert_eq!(target.len(), 2, "Target should have two bullets");
        
        let second = target.iter()
            .find(|b| b.text == "Second Task (Open)")
            .expect("Second task should be migrated");
        assert_eq!(second.priority, Some(1), "Priority not preserved");
        assert_eq!(second.tags, vec!["tag1"], "Tags not preserved");
        
        let third = target.iter()
            .find(|b| b.text == "Third Task (Open)")
            .expect("Third task should be migrated");
        assert_eq!(third.priority, Some(3), "Priority not preserved");
        assert_eq!(third.tags, vec!["tag2"], "Tags not preserved");
        
        Ok(())
    }

    #[test]
    #[serial]  // Prevent parallel test runs
    fn test_delete_bullet() -> Result<()> {
        let _env = TestEnv::new();
        let date = NaiveDate::from_ymd_opt(2025, 11, 6).unwrap();
        
        // Add three bullets
        add_bullet(date, "Task A", None, &vec![], &vec![])?;
        add_bullet(date, "Task B", Some(2), &vec!["important".to_string()], &vec!["Note 1".to_string(), "Note 2".to_string()])?;
        add_bullet(date, "Task C", None, &vec![], &vec![])?;
        
        // Verify initial state
        let path = file_for(date)?;
        let initial = parse_bullets(&read_file_lines(&path)?);
        assert_eq!(initial.len(), 3, "Expected three bullets initially");
        
        // Find Task B's ID and delete it
        let b_id = initial.iter()
            .find(|b| b.text == "Task B")
            .map(|b| b.visible_index)
            .expect("Should find Task B");
        delete_bullet(date, b_id)?;
        
        // Verify deletion
        let after = parse_bullets(&read_file_lines(&path)?);
        assert_eq!(after.len(), 2, "Should have two bullets after deletion");
        
        // Verify Task B is gone and others remain
        assert!(after.iter().any(|b| b.text == "Task A"), "Task A should remain");
        assert!(after.iter().any(|b| b.text == "Task C"), "Task C should remain");
        assert!(!after.iter().any(|b| b.text == "Task B"), "Task B should be deleted");
        
        // Verify visible indices are renumbered correctly
        assert_eq!(after[0].visible_index, 1, "First bullet should be index 1");
        assert_eq!(after[1].visible_index, 2, "Second bullet should be index 2");
        
        Ok(())
    }

    #[test]
    #[serial]  // Prevent parallel test runs
    fn test_delete_meeting() -> Result<()> {
        let _env = TestEnv::new();
        let date = NaiveDate::from_ymd_opt(2025, 11, 6).unwrap();
        
        // Add a regular bullet and a meeting
        add_bullet(date, "Regular Task", None, &vec![], &vec![])?;
        add_meeting(date, 
            NaiveTime::from_hms_opt(14, 30, 0).unwrap(),
            45,
            "Team Sync",
            &vec!["work".to_string()],
            &vec!["Prep agenda".to_string()]
        )?;
        
        // Verify initial state
        let path = file_for(date)?;
        let initial = parse_bullets(&read_file_lines(&path)?);
        assert_eq!(initial.len(), 2, "Expected two bullets initially");
        
        // Find and delete the meeting
        let mtg_id = initial.iter()
            .find(|b| b.text == "Team Sync")
            .map(|b| b.visible_index)
            .expect("Should find meeting");
        delete_bullet(date, mtg_id)?;
        
        // Verify deletion
        let after = parse_bullets(&read_file_lines(&path)?);
        assert_eq!(after.len(), 1, "Should have one bullet after deletion");
        assert_eq!(after[0].text, "Regular Task", "Regular task should remain");
        assert!(!after.iter().any(|b| b.text == "Team Sync"), "Meeting should be deleted");
        
        Ok(())
    }

    #[test]
    #[serial]  // Prevent parallel test runs
    fn test_migrate_to_specific_date() -> Result<()> {
        let _env = TestEnv::new();
        let from = NaiveDate::from_ymd_opt(2025, 11, 4).unwrap();
        let to = NaiveDate::from_ymd_opt(2025, 11, 10).unwrap();
        
        // Create a bullet on source date
        add_bullet(from, "Task for next week", Some(2), &vec!["work".to_string()], &vec![])?;
        
        // Get the bullet ID
        let from_path = file_for(from)?;
        let initial = parse_bullets(&read_file_lines(&from_path)?);
        assert_eq!(initial.len(), 1, "Should have one bullet");
        let bullet_id = initial[0].visible_index;
        
        // Migrate to specific date
        migrate_one(from, to, bullet_id)?;
        
        // Verify source is empty
        let source_after = parse_bullets(&read_file_lines(&from_path)?);
        assert_eq!(source_after.len(), 0, "Source should be empty after migration");
        
        // Verify target has the bullet
        let to_path = file_for(to)?;
        let target = parse_bullets(&read_file_lines(&to_path)?);
        assert_eq!(target.len(), 1, "Target should have one bullet");
        assert_eq!(target[0].text, "Task for next week", "Bullet text should match");
        assert_eq!(target[0].priority, Some(2), "Priority should be preserved");
        assert_eq!(target[0].tags, vec!["work"], "Tags should be preserved");
        
        Ok(())
    }

    #[test]
    #[serial]  // Prevent parallel test runs
    fn test_migrate_open_to_specific_date() -> Result<()> {
        let _env = TestEnv::new();
        let from = NaiveDate::from_ymd_opt(2025, 11, 3).unwrap();
        let to = NaiveDate::from_ymd_opt(2025, 11, 15).unwrap();
        
        // Add bullets with different states
        add_bullet(from, "Done Task", None, &vec![], &vec![])?;
        add_bullet(from, "Open Task 1", Some(1), &vec!["tag1".to_string()], &vec![])?;
        add_bullet(from, "Open Task 2", Some(2), &vec!["tag2".to_string()], &vec![])?;
        
        // Mark first task done
        let from_path = file_for(from)?;
        let initial = parse_bullets(&read_file_lines(&from_path)?);
        let done_id = initial.iter()
            .find(|b| b.text == "Done Task")
            .map(|b| b.visible_index)
            .expect("Should find done task");
        mark_done(from, done_id)?;
        
        // Migrate all open tasks to specific date
        migrate_open(from, to)?;
        
        // Verify source only has completed task
        let source_after = parse_bullets(&read_file_lines(&from_path)?);
        assert_eq!(source_after.len(), 1, "Source should have one bullet");
        assert!(source_after[0].completed, "Remaining bullet should be completed");
        
        // Verify target has both open tasks
        let to_path = file_for(to)?;
        let target = parse_bullets(&read_file_lines(&to_path)?);
        assert_eq!(target.len(), 2, "Target should have two bullets");
        assert!(target.iter().any(|b| b.text == "Open Task 1"), "Open Task 1 should be migrated");
        assert!(target.iter().any(|b| b.text == "Open Task 2"), "Open Task 2 should be migrated");
        
        Ok(())
    }


}

