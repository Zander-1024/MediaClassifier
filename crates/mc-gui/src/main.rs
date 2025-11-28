//! MediaClassifier GUI Application
//!
//! 使用 Slint 构建的媒体文件分类工具图形界面
//! 支持 i18n（使用 @tr() 宏）、主题切换、多页面导航

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;

use mc_lib::{ClassifyResult, Config, FileFilter, classify_file_with_config};
use walkdir::WalkDir;

slint::include_modules!();

/// 加载配置文件
fn load_config() -> Config {
    if let Ok(config_path) = Config::default_config_path()
        && Config::ensure_config_exists(&config_path).is_ok()
        && let Ok(config) = Config::load(&config_path)
    {
        return config;
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

/// 打开 GitHub 页面
fn open_github_url() {
    let url = "https://github.com/Zander-1024/MediaClassifier";
    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("xdg-open").arg(url).spawn();
    }
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open").arg(url).spawn();
    }
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("cmd")
            .args(["/c", "start", url])
            .spawn();
    }
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
                window.set_log_content("❌ 错误: 目录不存在\n".into());
                return;
            }

            window.set_app_state(AppState::Working);
            window.set_progress(0.0);
            window.set_log_content("🔍 开始扫描文件...\n".into());

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

                    // 更新 UI 进度
                    let window_weak_ui = window_weak_thread.clone();
                    slint::invoke_from_event_loop(move || {
                        if let Some(window) = window_weak_ui.upgrade() {
                            window.set_progress(progress);
                        }
                    })
                    .ok();

                    // 分类文件
                    match classify_file_with_config(&config_guard, &target_dir, file) {
                        Ok(ClassifyResult::Success { from, to }) => {
                            success += 1;
                            let msg = format!("[SUCCESS] {} -> {}\n", from.display(), to.display());
                            let window_weak_ui = window_weak_thread.clone();
                            slint::invoke_from_event_loop(move || {
                                if let Some(window) = window_weak_ui.upgrade() {
                                    let current_log = window.get_log_content().to_string();
                                    window.set_log_content((current_log + &msg).into());
                                }
                            })
                            .ok();
                        },
                        Ok(ClassifyResult::Skipped { path, reason }) => {
                            skipped += 1;
                            let msg =
                                format!("[SKIPPED] {} | Reason: {}\n", path.display(), reason);
                            let window_weak_ui = window_weak_thread.clone();
                            slint::invoke_from_event_loop(move || {
                                if let Some(window) = window_weak_ui.upgrade() {
                                    let current_log = window.get_log_content().to_string();
                                    window.set_log_content((current_log + &msg).into());
                                }
                            })
                            .ok();
                        },
                        Ok(ClassifyResult::Renamed { from, to }) => {
                            renamed += 1;
                            let msg = format!("[RENAMED] {} -> {}\n", from.display(), to.display());
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
                            let msg = format!("[FAILED] {} | Error: {}\n", path.display(), error);
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
                            let msg = format!("[ERROR] {} | {}\n", file.display(), e);
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
    // 打开 GitHub 链接 (Task 2)
    // ========================================================================
    main_window.on_open_github(move || {
        open_github_url();
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
        if let Some(window) = main_window_weak.upgrade()
            && let Some(folder) = rfd::FileDialog::new().pick_folder()
            && let Some(folder_name) = folder.file_name()
        {
            window.set_new_exclude_folder(folder_name.to_string_lossy().to_string().into());
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
