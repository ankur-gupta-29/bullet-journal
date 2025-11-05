use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use chrono::{Datelike, Local, NaiveDate, NaiveTime};
use clap::{Parser, Subcommand};
use directories::ProjectDirs;

#[derive(Parser)]
#[command(
    name = "bj",
    version,
    about = "Bullet journal CLI",
    long_about = "A fast terminal bullet journal that stores Markdown per day.\n\nFeatures:\n- Add/list/done/migrate bullets with priority, tags, notes\n- Week and month calendar views\n- Meetings: add/list/notify with start time and duration\n- Optional daily and meeting notifications (systemd user timers)",
    after_help = "Examples:\n  bj add \"Draft project plan\"\n  bj add -p high -t work -n \"prep\" \"Release train\"\n  bj list -t work -p 3\n  bj done 2\n  bj migrate --from 2025-11-04\n  bj week -t work\n  bj cal\n  bj meeting add -t 15:00 -u 30 \"Team sync\"\n  bj meeting list\n  bj meeting notify -w 15"
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
	/// Migrate all open bullets from a date to today (default: from yesterday)
	Migrate {
		/// Source date YYYY-MM-DD (default: yesterday)
		#[arg(long = "from")]
		from: Option<String>,
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
		Action::Migrate { from } => {
			let from_date = match from {
				Some(d) => parse_date(&d)?,
				None => {
					let today = Local::now().date_naive();
					today.pred_opt().context("cannot compute yesterday")?
				}
			};
			migrate_open_to_today(from_date)?
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
		println!("No bullets for {}", date);
		return Ok(());
	}
	for b in bullets {
		if let Some(p) = filter_priority { if b.priority != Some(p) { continue; } }
		if !filter_tags.is_empty() {
			if !filter_tags.iter().all(|t| b.tags.iter().any(|bt| bt == t)) { continue; }
		}
		let status = if b.completed { "x" } else { " " };
		let pr = match b.priority { Some(3) => "(!!!) ", Some(2) => "(!!) ", Some(1) => "(!) ", _ => "" };
		let mut time_prefix = String::new();
		if let Some(t) = b.meeting_time { time_prefix = format!("[mtg {}] ", t.format("%H:%M")); }
		let tags = if b.tags.is_empty() { String::new() } else { format!(" {}", b.tags.iter().map(|t| format!("#{}", t)).collect::<Vec<_>>().join(" ")) };
		println!("{:>3}. [{}] {}{}{}{}", b.visible_index, status, pr, time_prefix, b.text, tags);
		for n in &b.notes {
			println!("     ↳ {}", n);
		}
	}
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

fn migrate_open_to_today(from: NaiveDate) -> Result<()> {
	let to = Local::now().date_naive();
	if from == to { bail!("from date is today; nothing to migrate"); }
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
	for i in 0..7 {
		let day = start + chrono::Days::new(i);
		println!("\n# {}", day);
		let path = file_for(day)?;
		let lines = read_file_lines(&path)?;
		let bullets = parse_bullets(&lines);
		let mut count_open = 0usize;
		let mut count_done = 0usize;
		for b in &bullets {
			if let Some(p) = filter_priority { if b.priority != Some(p) { continue; } }
			if !filter_tags.is_empty() {
				if !filter_tags.iter().all(|t| b.tags.iter().any(|bt| bt == t)) { continue; }
			}
			if b.completed { count_done += 1; } else { count_open += 1; }
		}
		println!("Open: {}, Done: {}", count_open, count_done);
		for b in bullets {
			if let Some(p) = filter_priority { if b.priority != Some(p) { continue; } }
			if !filter_tags.is_empty() {
				if !filter_tags.iter().all(|t| b.tags.iter().any(|bt| bt == t)) { continue; }
			}
			let status = if b.completed { "x" } else { " " };
			let pr = match b.priority { Some(3) => "(!!!) ", Some(2) => "(!!) ", Some(1) => "(!) ", _ => "" };
			let mut time_prefix = String::new();
			if let Some(t) = b.meeting_time { time_prefix = format!("[mtg {}] ", t.format("%H:%M")); }
			let tags = if b.tags.is_empty() { String::new() } else { format!(" {}", b.tags.iter().map(|t| format!("#{}", t)).collect::<Vec<_>>().join(" ")) };
			println!(" - [{}] {}{}{}{}", status, pr, time_prefix, b.text, tags);
			for n in &b.notes { println!("   ↳ {}", n); }
		}
	}
	Ok(())
}

fn month_calendar(base: NaiveDate) -> Result<()> {
	let first = NaiveDate::from_ymd_opt(base.year(), base.month(), 1).context("invalid month")?;
	let next_month = if base.month() == 12 { NaiveDate::from_ymd_opt(base.year()+1, 1, 1).unwrap() } else { NaiveDate::from_ymd_opt(base.year(), base.month()+1, 1).unwrap() };
	let last_day = (next_month - chrono::Days::new(1)).day();
	println!("{}-{:02}", base.year(), base.month());
	println!("Mo Tu We Th Fr Sa Su");
	let offset = first.weekday().num_days_from_monday();
	let mut d = 1u32;
	for i in 0..offset { if i==0 { print!("") } print!("   "); }
	while d <= last_day {
		let cur = NaiveDate::from_ymd_opt(base.year(), base.month(), d).unwrap();
		let path = file_for(cur)?;
		let lines = read_file_lines(&path)?;
		let bullets = parse_bullets(&lines);
		let mark = if bullets.iter().any(|b| b.meeting_time.is_some()) { '*' } else if bullets.iter().any(|b| !b.completed) { '+' } else if !bullets.is_empty() { '.' } else { ' ' };
		print!("{:>2}{} ", d, mark);
		if cur.weekday().number_from_monday() == 7 { println!(); }
		d += 1;
	}
	println!();
	println!("legend: * meeting, + open bullets, . only done, ' ' empty");
	Ok(())
}
