use std::path::PathBuf;

use anyhow::Result;
use owo_colors::OwoColorize;
use unicode_width::UnicodeWidthStr;

use crate::git;
use crate::registry::{Registry, Template};
use crate::utilities;

enum Style { Normal, Yellow, Blue, Red, RedThrough }

struct Row {
    name: String,
    description: String,
    location: String,
    status: String,
    style: Style,
}

fn template_status(tmpl: &Template) -> (String, Style) {
    let path = PathBuf::from(&tmpl.location);
    let is_url = utilities::is_git_url(&tmpl.location);
    let is_sym = !is_url && path.is_symlink();
    let is_broken_sym = is_sym && !path.exists();
    let is_missing = !is_url && !is_sym && !path.exists();
    let is_file = !is_url && !is_missing && !is_broken_sym && !is_sym && path.is_file();
    let is_empty = !is_url && !is_missing && !is_broken_sym && !is_file
        && utilities::is_dir_empty(&path).unwrap_or(false);
    let has_no_git = !is_url && !is_missing && !is_broken_sym && !is_file && !is_empty
        && !path.join(".git").exists();

    if is_missing {
        ("(folder missing)".into(), Style::RedThrough)
    } else if is_broken_sym {
        ("(symlink broken)".into(), Style::RedThrough)
    } else if is_empty {
        ("(folder empty)".into(), Style::Red)
    } else if let Some(ref_val) = tmpl.commit.as_deref().or(tmpl.git_ref.as_deref()) {
        if tmpl.commit.is_some() {
            (format!("(at git commit {})", ref_val), Style::Blue)
        } else {
            let repo = if is_url {
                utilities::cache_path_for_url(&tmpl.location).ok()
                    .filter(|cache_path| cache_path.join(".git").exists())
            } else if path.join(".git").exists() {
                Some(path.clone())
            } else {
                None
            };
            match repo {
                None => (format!("(git ref {})", ref_val), Style::Blue),
                Some(repo_path) if !git::ref_exists(&repo_path, ref_val) => {
                    (format!("(git {} missing)", ref_val), Style::Red)
                }
                Some(repo_path) => {
                    let status_str = match git::classify_ref(&repo_path, ref_val) {
                        git::RefKind::Branch => format!("(in git branch {})", ref_val),
                        git::RefKind::Tag    => format!("(at git tag {})", ref_val),
                        git::RefKind::Commit => format!("(at git commit {})", ref_val),
                    };
                    (status_str, Style::Blue)
                }
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
    }
}

fn col_width(header: &str, values: impl Iterator<Item = usize>) -> usize {
    values.max().unwrap_or(0).max(header.width())
}

pub fn cmd_list() -> Result<()> {
    let registry = Registry::load()?;
    if registry.templates.is_empty() {
        println!("no templates available: use `templative add <FOLDER>` to add a template");
        return Ok(());
    }

    let rows: Vec<Row> = registry.templates_sorted().iter().map(|tmpl| {
        let (status, style) = template_status(tmpl);
        Row {
            name: tmpl.name.clone(),
            description: tmpl.description.as_deref().unwrap_or("").to_string(),
            location: tmpl.location.clone(),
            status,
            style,
        }
    }).collect();

    let pad = |text: &str, width: usize| -> String {
        format!("{}{}", text, " ".repeat(width.saturating_sub(text.width())))
    };
    let pad_underlined = |text: &str, width: usize| -> String {
        format!("{}{}", text.underline(), " ".repeat(width.saturating_sub(text.width())))
    };

    let truecolor = std::env::var("COLORTERM")
        .map(|val| val == "truecolor" || val == "24bit")
        .unwrap_or(false);

    let show_status = rows.iter().any(|row| !row.status.is_empty());
    let show_desc   = rows.iter().any(|row| !row.description.is_empty());

    let name_w   = col_width("NAME",        rows.iter().map(|row| row.name.width()));
    let status_w = if show_status { col_width("STATUS",      rows.iter().map(|row| row.status.width())) } else { 0 };
    let desc_w   = if show_desc   { col_width("DESCRIPTION", rows.iter().map(|row| row.description.width())) } else { 0 };

    let apply_style = |text: String, style: &Style| -> String {
        match style {
            Style::Normal     => text,
            Style::Yellow     => if truecolor { format!("{}", text.truecolor(252, 221, 42)) } else { format!("{}", text.yellow()) },
            Style::Blue       => format!("{}", text.blue()),
            Style::Red        => format!("{}", text.red()),
            Style::RedThrough => format!("{}", text.red().strikethrough()),
        }
    };

    let mut header = pad_underlined("NAME", name_w);
    if show_status { header = format!("{}  {}", header, pad_underlined("STATUS", status_w)); }
    if show_desc   { header = format!("{}  {}", header, pad_underlined("DESCRIPTION", desc_w)); }
    println!("{}  {}", header, "LOCATION".underline());

    for row in &rows {
        let name_pad = " ".repeat(name_w.saturating_sub(row.name.width()));
        let mut line = format!("{}{}", apply_style(row.name.clone(), &row.style), name_pad);
        if show_status {
            let status_pad = " ".repeat(status_w.saturating_sub(row.status.width()));
            line = format!("{}  {}{}", line, apply_style(row.status.clone(), &row.style), status_pad);
        }
        if show_desc { line = format!("{}  {}", line, pad(&row.description, desc_w)); }
        println!("{}  {}", line, row.location);
    }
    Ok(())
}
