use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use chrono::{Datelike, Local, NaiveDate};
use clap::{Parser, Subcommand};
use directories::ProjectDirs;

#[derive(Parser)]
#[command(name = "bj", version, about = "Bullet journal CLI")] 
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
			let (text, pr, tags) = parse_text_meta(rest);
			let notes = collect_notes(lines, idx + 1);
			out.push(Bullet { line_index: idx, visible_index: visible, completed: false, text, priority: pr, tags, notes });
		} else if let Some(rest) = trimmed.strip_prefix("- [x] ") {
			visible += 1;
			let (text, pr, tags) = parse_text_meta(rest);
			let notes = collect_notes(lines, idx + 1);
			out.push(Bullet { line_index: idx, visible_index: visible, completed: true, text, priority: pr, tags, notes });
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

fn parse_text_meta(rest: &str) -> (String, Option<u8>, Vec<String>) {
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
		let tags = if b.tags.is_empty() { String::new() } else { format!(" {}", b.tags.iter().map(|t| format!("#{}", t)).collect::<Vec<_>>().join(" ")) };
		println!("{:>3}. [{}] {}{}{}", b.visible_index, status, pr, b.text, tags);
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
			let (text, pr, tags) = parse_text_meta(&text);
			add_bullet(to, &text, pr, &tags, &[])?;
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
			let tags = if b.tags.is_empty() { String::new() } else { format!(" {}", b.tags.iter().map(|t| format!("#{}", t)).collect::<Vec<_>>().join(" ")) };
			println!(" - [{}] {}{}{}", status, pr, b.text, tags);
			for n in &b.notes { println!("   ↳ {}", n); }
		}
	}
	Ok(())
}
