use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Cell, Color, Table};
use mc_lib::{Config, FileSize};

/// 显示配置信息（表格格式）
pub fn show_config(config: &Config, config_path: &str) {
    println!("\n📋 Configuration: {}\n", config_path);

    // 全局设置表格
    show_global_settings(config);

    // 规则表格
    show_rules(config);

    // 扩展名别名
    if !config.extension_aliases.is_empty() {
        show_extension_aliases(config);
    }

    // 排除规则
    show_exclude_rules(config);
}

fn show_global_settings(config: &Config) {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_header(vec![
            Cell::new("Setting").fg(Color::Cyan),
            Cell::new("Value").fg(Color::Green),
        ]);

    table.add_row(vec!["Date Format", &config.global.date_format]);
    table.add_row(vec![
        "Directory Template",
        &config.global.directory_template,
    ]);
    table.add_row(vec![
        "Clean Empty Dirs",
        if config.global.clean_empty_dirs {
            "true"
        } else {
            "false"
        },
    ]);

    // 全局文件大小过滤（可选）
    if let Some(file_size) = &config.global.file_size {
        let min = file_size.min.as_deref().unwrap_or("∞");
        let max = file_size.max.as_deref().unwrap_or("∞");
        table.add_row(vec![
            "Min File Size (Global)",
            &format_size_for_display_str(min),
        ]);
        table.add_row(vec![
            "Max File Size (Global)",
            &format_size_for_display_str(max),
        ]);
    } else {
        table.add_row(vec!["File Size Filter (Global)", "None"]);
    }

    println!("Global Settings:");
    println!("{table}\n");
}

fn show_rules(config: &Config) {
    let enabled_count = config.rules.iter().filter(|r| r.enabled).count();

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_header(vec![
            Cell::new("#").fg(Color::Cyan),
            Cell::new("Name").fg(Color::Cyan),
            Cell::new("Extensions").fg(Color::Cyan),
            Cell::new("Min Size").fg(Color::Cyan),
            Cell::new("Max Size").fg(Color::Cyan),
            Cell::new("Directory Template").fg(Color::Cyan),
            Cell::new("Enabled").fg(Color::Cyan),
        ]);

    for (index, rule) in config.rules.iter().enumerate() {
        let extensions = if rule.extensions.len() > 3 {
            format!(
                "{}... ({} total)",
                rule.extensions[..3].join(","),
                rule.extensions.len()
            )
        } else {
            rule.extensions.join(",")
        };

        let (min_size, max_size) = if let Some(filter) = &rule.file_size {
            let min = filter
                .min
                .as_ref()
                .map(|s| format_size_for_display_str(s))
                .unwrap_or_else(|| "∞".to_string());
            let max = filter
                .max
                .as_ref()
                .map(|s| format_size_for_display_str(s))
                .unwrap_or_else(|| "∞".to_string());
            (min, max)
        } else {
            ("∞".to_string(), "∞".to_string())
        };

        let enabled_symbol = if rule.enabled { "✓" } else { "✗" };
        let enabled_cell = if rule.enabled {
            Cell::new(enabled_symbol).fg(Color::Green)
        } else {
            Cell::new(enabled_symbol).fg(Color::Red)
        };

        table.add_row(vec![
            Cell::new((index + 1).to_string()),
            Cell::new(&rule.name),
            Cell::new(extensions),
            Cell::new(min_size),
            Cell::new(max_size),
            Cell::new(&rule.directory_template),
            enabled_cell,
        ]);
    }

    println!("Classification Rules ({} enabled):", enabled_count);
    println!("{table}\n");
}

fn show_extension_aliases(config: &Config) {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_header(vec![
            Cell::new("Alias").fg(Color::Cyan),
            Cell::new("Extensions").fg(Color::Cyan),
        ]);

    for (alias, extensions) in &config.extension_aliases {
        table.add_row(vec![alias, &extensions.join(", ")]);
    }

    println!("Extension Aliases:");
    println!("{table}\n");
}

fn show_exclude_rules(config: &Config) {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_header(vec![
            Cell::new("Setting").fg(Color::Cyan),
            Cell::new("Value").fg(Color::Green),
        ]);

    table.add_row(vec![
        "Hidden Files",
        if config.exclude.hidden_files {
            "Excluded"
        } else {
            "Included"
        },
    ]);

    if !config.exclude.directories.is_empty() {
        table.add_row(vec![
            "Excluded Directories",
            &config.exclude.directories.join(", "),
        ]);
    }

    if !config.exclude.patterns.is_empty() {
        table.add_row(vec![
            "Excluded Patterns",
            &config.exclude.patterns.join(", "),
        ]);
    }

    println!("Exclude Rules:");
    println!("{table}\n");
}

fn format_size_for_display_str(size_str: &str) -> String {
    match FileSize::parse(size_str) {
        Ok(size) if size.bytes == 0 => "∞".to_string(),
        Ok(size) => size.format(),
        Err(_) => size_str.to_string(),
    }
}
