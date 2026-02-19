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

fn git_ref_status(tmpl: &Template, path: &PathBuf, is_url: bool) -> Option<(String, Style)> {
    let ref_val = tmpl.git_ref.as_deref()?;
    let repo = if is_url {
        utilities::cache_path_for_url(&tmpl.location).ok()
            .filter(|cache_path| cache_path.join(".git").exists())
    } else if path.join(".git").exists() {
        Some(path.clone())
    } else {
        None
    };
    Some(match repo {
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
    })
}

fn worse_style(a: Style, b: Style) -> Style {
    match (a, b) {
        (Style::RedThrough, _) | (_, Style::RedThrough) => Style::RedThrough,
        (Style::Red, _) | (_, Style::Red) => Style::Red,
        (Style::Yellow, _) | (_, Style::Yellow) => Style::Yellow,
        (Style::Blue, _) | (_, Style::Blue) => Style::Blue,
        _ => Style::Normal,
    }
}

fn template_status(tmpl: &Template) -> (String, Style) {
    let path = PathBuf::from(&tmpl.location);
    let is_url = utilities::is_git_url(&tmpl.location);
    let is_missing = !is_url && !path.exists();
    let is_file = !is_url && !is_missing && path.is_file();
    let is_empty = !is_url && !is_missing && !is_file
        && utilities::is_dir_empty(&path).unwrap_or(false);
    let has_no_git = !is_url && !is_missing && !is_file && !is_empty
        && !path.join(".git").exists();

    if is_missing {
        return ("(template missing)".into(), Style::RedThrough);
    }
    if is_empty {
        return ("(folder empty)".into(), Style::Red);
    }
    if is_file {
        if let Some((git_str, git_style)) = git_ref_status(tmpl, &path, is_url) {
            let combined_style = worse_style(Style::Blue, git_style);
            return (format!("(single file) {}", git_str), combined_style);
        }
        return ("(single file)".into(), Style::Blue);
    }
    if let Some(git_annotation) = git_ref_status(tmpl, &path, is_url) {
        return git_annotation;
    }
    if has_no_git {
        return ("(no git)".into(), Style::Yellow);
    }
    (String::new(), Style::Normal)
}

fn col_width(header: &str, values: impl Iterator<Item = usize>) -> usize {
    values.max().unwrap_or(0).max(header.width())
}

pub fn cmd_list(color: bool) -> Result<()> {
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
        if color {
            format!("{}{}", text.underline(), " ".repeat(width.saturating_sub(text.width())))
        } else {
            format!("{}{}", text, " ".repeat(width.saturating_sub(text.width())))
        }
    };

    let truecolor = color && std::env::var("COLORTERM")
        .map(|val| val == "truecolor" || val == "24bit")
        .unwrap_or(false);

    let show_status = rows.iter().any(|row| !row.status.is_empty());
    let show_desc   = rows.iter().any(|row| !row.description.is_empty());

    let name_w   = col_width("NAME",        rows.iter().map(|row| row.name.width()));
    let status_w = if show_status { col_width("STATUS",      rows.iter().map(|row| row.status.width())) } else { 0 };
    let desc_w   = if show_desc   { col_width("DESCRIPTION", rows.iter().map(|row| row.description.width())) } else { 0 };

    let apply_style = |text: String, style: &Style| -> String {
        if !color { return text; }
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
    let location_header = if color { format!("{}", "LOCATION".underline()) } else { "LOCATION".to_string() };
    println!("{}  {}", header, location_header);

    for row in &rows {
        let mut line = pad(&row.name, name_w);
        if show_status { line = format!("{}  {}", line, pad(&row.status, status_w)); }
        if show_desc   { line = format!("{}  {}", line, pad(&row.description, desc_w)); }
        let line = format!("{}  {}", line, row.location);
        println!("{}", apply_style(line, &row.style));
    }
    Ok(())
}
