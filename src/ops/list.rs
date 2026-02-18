use std::path::PathBuf;

use anyhow::Result;
use owo_colors::OwoColorize;
use unicode_width::UnicodeWidthStr;

use crate::git;
use crate::registry::Registry;
use crate::utilities;

pub fn cmd_list() -> Result<()> {
    enum Style { Normal, Yellow, Blue, Red, RedThrough }
    struct Row {
        name: String,
        desc: String,
        location: String,
        status: String,
        style: Style,
    }

    let registry = Registry::load()?;
    if registry.templates.is_empty() {
        println!("no templates available: use `templative add <FOLDER>` to add a template");
        return Ok(());
    }
    let templates = registry.templates_sorted();

    let rows: Vec<Row> = templates.iter().map(|t| {
        let path = PathBuf::from(&t.location);
        let is_url = utilities::is_git_url(&t.location);
        let is_sym = !is_url && path.is_symlink();
        let is_broken_sym = is_sym && !path.exists();
        let is_missing = !is_url && !is_sym && !path.exists();
        let is_file = !is_url && !is_missing && !is_broken_sym && !is_sym && path.is_file();
        let is_empty = !is_url && !is_missing && !is_broken_sym && !is_file
            && utilities::is_dir_empty(&path).unwrap_or(false);
        let has_no_git = !is_url && !is_missing && !is_broken_sym && !is_file && !is_empty
            && !path.join(".git").exists();

        let (status, style) = if is_missing {
            ("(folder missing)".into(), Style::RedThrough)
        } else if is_broken_sym {
            ("(symlink broken)".into(), Style::RedThrough)
        } else if is_empty {
            ("(folder empty)".into(), Style::Red)
        } else if let Some(ref_val) = t.commit.as_deref().or(t.git_ref.as_deref()) {
            let repo = if is_url {
                utilities::cache_path_for_url(&t.location).ok()
                    .filter(|p| p.join(".git").exists())
            } else if path.join(".git").exists() {
                Some(path.clone())
            } else {
                None
            };
            match repo {
                None => {
                    let s = if t.commit.is_some() {
                        format!("(at git commit {})", ref_val)
                    } else {
                        format!("(git ref {})", ref_val)
                    };
                    (s, Style::Blue)
                }
                Some(r) if !git::ref_exists(&r, ref_val) => {
                    (format!("(git {} missing)", ref_val), Style::Red)
                }
                Some(r) => {
                    let s = if t.commit.is_some() {
                        format!("(at git commit {})", ref_val)
                    } else {
                        match git::classify_ref(&r, ref_val) {
                            git::RefKind::Branch => format!("(in git branch {})", ref_val),
                            git::RefKind::Tag    => format!("(at git tag {})", ref_val),
                            git::RefKind::Commit => format!("(at git commit {})", ref_val),
                        }
                    };
                    (s, Style::Blue)
                }
            }
        } else if is_file {
            ("(single file)".into(), Style::Blue)
        } else if is_sym {
            ("(symlink)".into(), Style::Blue)
        } else if has_no_git {
            ("(no git)".into(), Style::Yellow)
        } else {
            (String::new(), Style::Normal)
        };

        Row {
            name: t.name.clone(),
            desc: t.description.as_deref().unwrap_or("").to_string(),
            location: t.location.clone(),
            status,
            style,
        }
    }).collect();

    let pad = |s: &str, w: usize| -> String {
        format!("{}{}", s, " ".repeat(w.saturating_sub(s.width())))
    };
    let upad = |s: &str, w: usize| -> String {
        format!("{}{}", s.underline(), " ".repeat(w.saturating_sub(s.width())))
    };

    let name_w   = rows.iter().map(|r| r.name.width()).max().unwrap_or(0).max("NAME".width());
    let status_w = rows.iter().map(|r| r.status.width()).max().unwrap_or(0).max("STATUS".width());
    let desc_w   = rows.iter().map(|r| r.desc.width()).max().unwrap_or(0).max("DESCRIPTION".width());
    let loc_w    = rows.iter().map(|r| r.location.width()).max().unwrap_or(0).max("LOCATION".width());

    println!("{}  {}  {}  {}",
        upad("NAME", name_w), upad("STATUS", status_w),
        upad("DESCRIPTION", desc_w), "LOCATION".underline());

    for row in &rows {
        let line = format!("{}  {}  {}  {}",
            pad(&row.name, name_w), pad(&row.status, status_w),
            pad(&row.desc, desc_w), row.location);
        match row.style {
            Style::Normal   => println!("{}", line),
            Style::Yellow   => println!("{}", line.yellow()),
            Style::Blue     => println!("{}", line.blue()),
            Style::Red      => println!("{}", line.red()),
            Style::RedThrough => println!("{}", line.red().strikethrough()),
        }
    }
    Ok(())
}
