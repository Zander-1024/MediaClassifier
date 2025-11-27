//! MediaClassifier GUI Application
//!
//! 使用 Slint 构建的媒体文件分类工具图形界面
//! 支持 i18n、主题切换、多页面导航

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;

use mc_lib::{ClassifyResult, Config, FileFilter, classify_file_with_config};
use walkdir::WalkDir;

slint::include_modules!();

/// 中文 i18n 字符串
fn get_zh_strings() -> I18nStrings {
    I18nStrings {
        app_title: "🎬 媒体文件分类器".into(),
        working_directory: "工作目录".into(),
        select_directory: "选择工作目录".into(),
        start_working: "开始工作".into(),
        show_details: "显示详情".into(),
        hide_details: "隐藏详情".into(),
        progress_label: "处理进度".into(),
        log_error_dir_not_exist: "❌ 错误: 目录不存在".into(),
        log_scanning: "🔍 开始扫描文件...".into(),
        log_processing: "📁 处理:".into(),
        stats_title: "📊 处理完成".into(),
        stats_total: "总计".into(),
        stats_success: "成功".into(),
        stats_renamed: "重命名".into(),
        stats_skipped: "跳过".into(),
        stats_failed: "失败".into(),
        stats_close: "关闭".into(),
        config_title: "⚙️ 配置管理".into(),
        config_add: "➕ 新增规则".into(),
        config_exclude: "🚫 屏蔽文件夹".into(),
        config_back: "← 返回主页".into(),
        config_rule_name: "规则名称".into(),
        config_rule_desc: "规则描述".into(),
        config_rule_ext: "文件扩展名".into(),
        config_rule_template: "目录模板".into(),
        config_rule_min_size: "最小大小".into(),
        config_rule_max_size: "最大大小".into(),
        config_rule_enabled: "启用".into(),
        config_edit: "编辑".into(),
        config_delete: "删除".into(),
        config_save: "保存".into(),
        config_cancel: "取消".into(),
        nav_config: "⚙️".into(),
        nav_main: "🏠".into(),
        theme_auto: "自动".into(),
        theme_light: "浅色".into(),
        theme_dark: "深色".into(),
        lang_zh: "中文".into(),
        lang_en: "EN".into(),
        placeholder_select_folder: "点击选择文件夹...".into(),
        placeholder_ext_example: "jpg,png,gif".into(),
        placeholder_template_example: "{ext}/{date}".into(),
        placeholder_min_size: "0B".into(),
        placeholder_max_size: "无限制".into(),
        status_processing: "⏳ 处理中...".into(),
        exclude_title: "🚫 屏蔽文件夹管理".into(),
        exclude_add: "添加".into(),
        exclude_add_folder: "+ 添加文件夹".into(),
        exclude_placeholder: "输入文件夹名称（如：.git, node_modules）".into(),
        about_title: "🎞️ 关于 MediaClassifier".into(),
        about_version: "版本：v1.2.0".into(),
        about_description:
            "一款基于规则的媒体文件自动分类工具，支持自定义目录模板、文件大小过滤等功能。".into(),
        about_author: "作者：Zander".into(),
        about_support: "如果您觉得这个工具有用，欢迎支持：".into(),
    }
}

/// 英文 i18n 字符串
fn get_en_strings() -> I18nStrings {
    I18nStrings {
        app_title: "🎬 MediaClassifier".into(),
        working_directory: "Working Directory".into(),
        select_directory: "Select Directory".into(),
        start_working: "Start".into(),
        show_details: "Show Details".into(),
        hide_details: "Hide Details".into(),
        progress_label: "Progress".into(),
        log_error_dir_not_exist: "❌ Error: Directory does not exist".into(),
        log_scanning: "🔍 Scanning files...".into(),
        log_processing: "📁 Processing:".into(),
        stats_title: "📊 Completed".into(),
        stats_total: "Total".into(),
        stats_success: "Success".into(),
        stats_renamed: "Renamed".into(),
        stats_skipped: "Skipped".into(),
        stats_failed: "Failed".into(),
        stats_close: "Close".into(),
        config_title: "⚙️ Configuration".into(),
        config_add: "➕ Add Rule".into(),
        config_exclude: "🚫 Exclude Folders".into(),
        config_back: "← Back".into(),
        config_rule_name: "Name".into(),
        config_rule_desc: "Description".into(),
        config_rule_ext: "Extensions".into(),
        config_rule_template: "Template".into(),
        config_rule_min_size: "Min Size".into(),
        config_rule_max_size: "Max Size".into(),
        config_rule_enabled: "Enabled".into(),
        config_edit: "Edit".into(),
        config_delete: "Delete".into(),
        config_save: "Save".into(),
        config_cancel: "Cancel".into(),
        nav_config: "⚙️".into(),
        nav_main: "🏠".into(),
        theme_auto: "Auto".into(),
        theme_light: "Light".into(),
        theme_dark: "Dark".into(),
        lang_zh: "中文".into(),
        lang_en: "EN".into(),
        placeholder_select_folder: "Click to select folder...".into(),
        placeholder_ext_example: "jpg,png,gif".into(),
        placeholder_template_example: "{ext}/{date}".into(),
        placeholder_min_size: "0B".into(),
        placeholder_max_size: "Unlimited".into(),
        status_processing: "⏳ Processing...".into(),
        exclude_title: "🚫 Manage Exclude Folders".into(),
        exclude_add: "Add".into(),
        exclude_add_folder: "+ Add Folder".into(),
        exclude_placeholder: "Enter folder name (e.g., .git, node_modules)".into(),
        about_title: "🎞️ About MediaClassifier".into(),
        about_version: "Version: v1.2.0".into(),
        about_description: "A rule-based media file auto-classification tool with custom directory templates, file size filtering, and more.".into(),
        about_author: "Author: Zander".into(),
        about_support: "If you find this tool useful, feel free to support:".into(),
    }
}

/// 加载配置文件
fn load_config() -> Config {
    if let Ok(config_path) = Config::default_config_path() {
        if Config::ensure_config_exists(&config_path).is_ok() {
            if let Ok(config) = Config::load(&config_path) {
                return config;
            }
        }
    }
    Config::default()
}

/// 保存配置文件
fn save_config(config: &Config) -> anyhow::Result<()> {
    let config_path = Config::default_config_path()?;
    config.save(&config_path)?;
    Ok(())
}

/// 从 Config 规则转换为 GUI RuleItem
fn rules_to_gui(config: &Config) -> Vec<RuleItem> {
    config
        .rules
        .iter()
        .enumerate()
        .map(|(idx, rule)| RuleItem {
            id: idx as i32,
            name: rule.name.clone().into(),
            description: rule.description.clone().into(),
            extensions: rule.extensions.join(",").into(),
            directory_template: rule.directory_template.clone().into(),
            min_size: rule
                .file_size
                .as_ref()
                .and_then(|f| f.min.clone())
                .unwrap_or_default()
                .into(),
            max_size: rule
                .file_size
                .as_ref()
                .and_then(|f| f.max.clone())
                .unwrap_or_default()
                .into(),
            enabled: rule.enabled,
        })
        .collect()
}

fn main() -> anyhow::Result<()> {
    // 初始化日志
    simplelog::TermLogger::init(
        simplelog::LevelFilter::Info,
        simplelog::Config::default(),
        simplelog::TerminalMode::Mixed,
        simplelog::ColorChoice::Auto,
    )
    .ok();

    let main_window = MainWindow::new()?;

    // 加载配置
    let config = Arc::new(Mutex::new(load_config()));

    // 设置默认值
    main_window.set_working_directory("".into());
    main_window.set_current_language("zh".into());

    // ========================================================================
    // 浏览目录 - 使用 rfd 文件对话框
    // ========================================================================
    let main_window_weak = main_window.as_weak();
    main_window.on_browse_directory(move || {
        let window_weak = main_window_weak.clone();
        // 使用 rfd 打开目录选择对话框
        let folder = rfd::FileDialog::new().pick_folder();
        if let (Some(path), Some(window)) = (folder, window_weak.upgrade()) {
            window.set_working_directory(path.display().to_string().into());
        }
    });

    // ========================================================================
    // 开始工作 - 使用 mc-lib 进行文件分类
    // ========================================================================
    let main_window_weak = main_window.as_weak();
    let config_clone = config.clone();
    main_window.on_start_work(move || {
        let window_weak = main_window_weak.clone();
        let config = config_clone.clone();

        if let Some(window) = window_weak.upgrade() {
            let working_dir = window.get_working_directory().to_string();
            if working_dir.is_empty() {
                return;
            }

            let target_dir = PathBuf::from(&working_dir);
            if !target_dir.exists() || !target_dir.is_dir() {
                let i18n = window.get_i18n();
                window.set_log_content(format!("{}\n", i18n.log_error_dir_not_exist).into());
                return;
            }

            // Get i18n strings before spawning thread
            let i18n = window.get_i18n();
            let log_scanning = i18n.log_scanning.to_string();
            let log_processing = i18n.log_processing.to_string();

            window.set_app_state(AppState::Working);
            window.set_progress(0.0);
            window.set_log_content(format!("{}\n", log_scanning).into());

            // 在新线程中处理文件
            let window_weak_thread = window_weak.clone();
            thread::spawn(move || {
                let config_guard = config.lock().unwrap();
                let filter = FileFilter::new(&config_guard.exclude);

                // 收集所有媒体文件
                let files: Vec<PathBuf> = WalkDir::new(&target_dir)
                    .into_iter()
                    .filter_entry(|e| !filter.should_exclude_entry(e))
                    .filter_map(|e| e.ok())
                    .filter(|e| e.file_type().is_file())
                    .filter(|e| !filter.should_exclude_file(e.path()))
                    .filter(|e| mc_lib::get_media_info(e.path()).is_some())
                    .map(|e| e.into_path())
                    .collect();

                let total = files.len();
                let mut success = 0;
                let mut skipped = 0;
                let mut renamed = 0;
                let mut failed = 0;

                for (idx, file) in files.iter().enumerate() {
                    let progress = (idx + 1) as f32 / total as f32;
                    let log_entry = format!("{} {}\n", log_processing, file.display());

                    // 更新 UI
                    let window_weak_ui = window_weak_thread.clone();
                    let progress_val = progress;
                    let log_text = log_entry.clone();
                    slint::invoke_from_event_loop(move || {
                        if let Some(window) = window_weak_ui.upgrade() {
                            window.set_progress(progress_val);
                            let current_log = window.get_log_content().to_string();
                            window.set_log_content((current_log + &log_text).into());
                        }
                    })
                    .ok();

                    // 分类文件
                    match classify_file_with_config(&config_guard, &target_dir, file) {
                        Ok(ClassifyResult::Success { from, to }) => {
                            success += 1;
                            let msg = format!("✅ {} -> {}\n", from.display(), to.display());
                            let window_weak_ui = window_weak_thread.clone();
                            slint::invoke_from_event_loop(move || {
                                if let Some(window) = window_weak_ui.upgrade() {
                                    let current_log = window.get_log_content().to_string();
                                    window.set_log_content((current_log + &msg).into());
                                }
                            })
                            .ok();
                        },
                        Ok(ClassifyResult::Skipped { .. }) => {
                            skipped += 1;
                        },
                        Ok(ClassifyResult::Renamed { from, to }) => {
                            renamed += 1;
                            let msg = format!("🔄 {} -> {}\n", from.display(), to.display());
                            let window_weak_ui = window_weak_thread.clone();
                            slint::invoke_from_event_loop(move || {
                                if let Some(window) = window_weak_ui.upgrade() {
                                    let current_log = window.get_log_content().to_string();
                                    window.set_log_content((current_log + &msg).into());
                                }
                            })
                            .ok();
                        },
                        Ok(ClassifyResult::Failed { path, error }) => {
                            failed += 1;
                            let msg = format!("❌ {}: {}\n", path.display(), error);
                            let window_weak_ui = window_weak_thread.clone();
                            slint::invoke_from_event_loop(move || {
                                if let Some(window) = window_weak_ui.upgrade() {
                                    let current_log = window.get_log_content().to_string();
                                    window.set_log_content((current_log + &msg).into());
                                }
                            })
                            .ok();
                        },
                        Err(e) => {
                            failed += 1;
                            let msg = format!("❌ {}: {}\n", file.display(), e);
                            let window_weak_ui = window_weak_thread.clone();
                            slint::invoke_from_event_loop(move || {
                                if let Some(window) = window_weak_ui.upgrade() {
                                    let current_log = window.get_log_content().to_string();
                                    window.set_log_content((current_log + &msg).into());
                                }
                            })
                            .ok();
                        },
                    }
                }

                // 清理空目录
                if config_guard.global.clean_empty_dirs {
                    mc_lib::remove_empty_dirs(&target_dir).ok();
                }

                // 完成，更新统计
                let window_weak_final = window_weak_thread.clone();
                let total_val = total as i32;
                slint::invoke_from_event_loop(move || {
                    if let Some(window) = window_weak_final.upgrade() {
                        window.set_progress(1.0);
                        window.set_app_state(AppState::Completed);
                        window.set_stats(Statistics {
                            total: total_val,
                            success,
                            skipped,
                            renamed,
                            failed,
                        });
                        window.set_show_stats_popup(true);
                    }
                })
                .ok();
            });
        }
    });

    // ========================================================================
    // 切换日志显示
    // ========================================================================
    let main_window_weak = main_window.as_weak();
    main_window.on_toggle_log(move || {
        if let Some(window) = main_window_weak.upgrade() {
            let current = window.get_show_log();
            window.set_show_log(!current);
        }
    });

    // ========================================================================
    // 关闭统计弹窗
    // ========================================================================
    let main_window_weak = main_window.as_weak();
    main_window.on_close_stats_popup(move || {
        if let Some(window) = main_window_weak.upgrade() {
            window.set_show_stats_popup(false);
            window.set_app_state(AppState::Idle);
        }
    });

    // ========================================================================
    // 切换到配置页面
    // ========================================================================
    let main_window_weak = main_window.as_weak();
    let config_clone = config.clone();
    main_window.on_go_to_config(move || {
        if let Some(window) = main_window_weak.upgrade() {
            window.set_current_page(PageType::Config);

            // 加载配置规则
            let config_guard = config_clone.lock().unwrap();
            let rules = rules_to_gui(&config_guard);
            let rules_model = std::rc::Rc::new(slint::VecModel::from(rules));
            window.set_rules(rules_model.into());
        }
    });

    // ========================================================================
    // 返回主页面
    // ========================================================================
    let main_window_weak = main_window.as_weak();
    main_window.on_go_to_main(move || {
        if let Some(window) = main_window_weak.upgrade() {
            window.set_current_page(PageType::Main);
        }
    });

    // ========================================================================
    // 添加规则
    // ========================================================================
    let main_window_weak = main_window.as_weak();
    main_window.on_add_rule(move || {
        if let Some(window) = main_window_weak.upgrade() {
            // 重置表单
            window.set_is_editing_rule(false);
            window.set_editing_rule_id(-1);
            window.set_new_rule_name("".into());
            window.set_new_rule_desc("".into());
            window.set_new_rule_ext("".into());
            window.set_new_rule_template("".into());
            window.set_new_rule_min_size("".into());
            window.set_new_rule_max_size("".into());
            window.set_new_rule_enabled(true);
            window.set_show_add_rule_popup(true);
        }
    });

    // ========================================================================
    // 编辑规则
    // ========================================================================
    let main_window_weak = main_window.as_weak();
    let config_clone = config.clone();
    main_window.on_edit_rule(move |rule_id| {
        if let Some(window) = main_window_weak.upgrade() {
            let config_guard = config_clone.lock().unwrap();

            if let Some(rule) = config_guard.rules.get(rule_id as usize) {
                window.set_is_editing_rule(true);
                window.set_editing_rule_id(rule_id);
                window.set_new_rule_name(rule.name.clone().into());
                window.set_new_rule_desc(rule.description.clone().into());
                window.set_new_rule_ext(rule.extensions.join(",").into());
                window.set_new_rule_template(rule.directory_template.clone().into());
                window.set_new_rule_min_size(
                    rule.file_size
                        .as_ref()
                        .and_then(|f| f.min.clone())
                        .unwrap_or_default()
                        .into(),
                );
                window.set_new_rule_max_size(
                    rule.file_size
                        .as_ref()
                        .and_then(|f| f.max.clone())
                        .unwrap_or_default()
                        .into(),
                );
                window.set_new_rule_enabled(rule.enabled);
                window.set_show_add_rule_popup(true);
            }
        }
    });

    // ========================================================================
    // 保存新规则
    // ========================================================================
    let main_window_weak = main_window.as_weak();
    let config_clone = config.clone();
    main_window.on_save_new_rule(move || {
        if let Some(window) = main_window_weak.upgrade() {
            let name = window.get_new_rule_name().to_string();
            let desc = window.get_new_rule_desc().to_string();
            let ext = window.get_new_rule_ext().to_string();
            let template = window.get_new_rule_template().to_string();
            let min_size = window.get_new_rule_min_size().to_string();
            let max_size = window.get_new_rule_max_size().to_string();
            let enabled = window.get_new_rule_enabled();
            let is_editing = window.get_is_editing_rule();
            let editing_id = window.get_editing_rule_id();

            // 创建新规则
            let extensions: Vec<String> = ext
                .split(',')
                .map(|s| s.trim().to_lowercase())
                .filter(|s| !s.is_empty())
                .collect();

            let file_size = if min_size.is_empty() && max_size.is_empty() {
                None
            } else {
                Some(mc_lib::FileSizeFilter {
                    min: if min_size.is_empty() {
                        None
                    } else {
                        Some(min_size)
                    },
                    max: if max_size.is_empty() {
                        None
                    } else {
                        Some(max_size)
                    },
                })
            };

            let new_rule = mc_lib::Rule {
                name,
                description: desc,
                extensions,
                file_size,
                directory_template: template,
                date_format: Some("YYYYMMDD".to_string()),
                enabled,
            };

            // 添加或更新配置并保存
            {
                let mut config_guard = config_clone.lock().unwrap();

                if is_editing && editing_id >= 0 && (editing_id as usize) < config_guard.rules.len()
                {
                    // 更新现有规则
                    config_guard.rules[editing_id as usize] = new_rule;
                } else {
                    // 添加新规则
                    config_guard.rules.push(new_rule);
                }

                save_config(&config_guard).ok();

                // 更新 UI
                let rules = rules_to_gui(&config_guard);
                let rules_model = std::rc::Rc::new(slint::VecModel::from(rules));
                window.set_rules(rules_model.into());
            }

            window.set_show_add_rule_popup(false);
        }
    });

    // ========================================================================
    // 取消添加规则
    // ========================================================================
    let main_window_weak = main_window.as_weak();
    main_window.on_cancel_add_rule(move || {
        if let Some(window) = main_window_weak.upgrade() {
            window.set_show_add_rule_popup(false);
        }
    });

    // ========================================================================
    // 删除规则
    // ========================================================================
    let main_window_weak = main_window.as_weak();
    let config_clone = config.clone();
    main_window.on_delete_rule(move |rule_id| {
        if let Some(window) = main_window_weak.upgrade() {
            let mut config_guard = config_clone.lock().unwrap();

            if (rule_id as usize) < config_guard.rules.len() {
                config_guard.rules.remove(rule_id as usize);
                save_config(&config_guard).ok();

                // 更新 UI
                let rules = rules_to_gui(&config_guard);
                let rules_model = std::rc::Rc::new(slint::VecModel::from(rules));
                window.set_rules(rules_model.into());
            }
        }
    });

    // ========================================================================
    // 切换主题
    // ========================================================================
    let main_window_weak = main_window.as_weak();
    main_window.on_change_theme(move |theme| {
        if let Some(window) = main_window_weak.upgrade() {
            window.set_theme_mode(theme);
        }
    });

    // ========================================================================
    // 切换语言
    // ========================================================================
    let main_window_weak = main_window.as_weak();
    main_window.on_change_language(move |lang| {
        if let Some(window) = main_window_weak.upgrade() {
            window.set_current_language(lang.clone());
            if lang == "en" {
                window.set_i18n(get_en_strings());
            } else {
                window.set_i18n(get_zh_strings());
            }
        }
    });

    // ========================================================================
    // 管理屏蔽文件夹
    // ========================================================================
    let main_window_weak = main_window.as_weak();
    let config_clone = config.clone();
    main_window.on_manage_exclude_folders(move || {
        if let Some(window) = main_window_weak.upgrade() {
            // 从配置加载屏蔽文件夹列表
            let exclude_list: Vec<slint::SharedString> = config_clone
                .lock()
                .unwrap()
                .exclude
                .directories
                .iter()
                .map(|s| s.clone().into())
                .collect();

            let model = std::rc::Rc::new(slint::VecModel::from(exclude_list));
            window.set_exclude_folders(model.into());
            window.set_show_exclude_popup(true);
        }
    });

    // 添加屏蔽文件夹
    let main_window_weak = main_window.as_weak();
    let config_clone = config.clone();
    main_window.on_add_exclude_folder(move || {
        if let Some(window) = main_window_weak.upgrade() {
            let folder = window
                .get_new_exclude_folder()
                .to_string()
                .trim()
                .to_string();
            if !folder.is_empty() {
                // 添加到配置
                let mut cfg = config_clone.lock().unwrap();
                if !cfg.exclude.directories.contains(&folder) {
                    cfg.exclude.directories.push(folder.clone());
                    let _ = save_config(&cfg);

                    // 更新UI列表
                    let exclude_list: Vec<slint::SharedString> = cfg
                        .exclude
                        .directories
                        .iter()
                        .map(|s| s.clone().into())
                        .collect();

                    let model = std::rc::Rc::new(slint::VecModel::from(exclude_list));
                    window.set_exclude_folders(model.into());
                    window.set_new_exclude_folder("".into());
                }
            }
        }
    });

    // 删除屏蔽文件夹
    let main_window_weak = main_window.as_weak();
    let config_clone = config.clone();
    main_window.on_remove_exclude_folder(move |index| {
        if let Some(window) = main_window_weak.upgrade() {
            let mut cfg = config_clone.lock().unwrap();
            if (index as usize) < cfg.exclude.directories.len() {
                cfg.exclude.directories.remove(index as usize);
                let _ = save_config(&cfg);

                // 更新UI列表
                let exclude_list: Vec<slint::SharedString> = cfg
                    .exclude
                    .directories
                    .iter()
                    .map(|s| s.clone().into())
                    .collect();

                let model = std::rc::Rc::new(slint::VecModel::from(exclude_list));
                window.set_exclude_folders(model.into());
            }
        }
    });

    // 浏览选择屏蔽文件夹
    let main_window_weak = main_window.as_weak();
    main_window.on_browse_exclude_folder(move || {
        if let Some(window) = main_window_weak.upgrade() {
            if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                if let Some(folder_name) = folder.file_name() {
                    window.set_new_exclude_folder(folder_name.to_string_lossy().to_string().into());
                }
            }
        }
    });

    // ========================================================================
    // 显示关于应用弹窗
    // ========================================================================
    let main_window_weak = main_window.as_weak();
    main_window.on_show_about(move || {
        if let Some(window) = main_window_weak.upgrade() {
            window.set_show_about_popup(true);
        }
    });

    // 运行应用
    main_window.run()?;
    Ok(())
}
