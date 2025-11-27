//! MediaClassifier GUI Application
//!
//! 使用 Slint 构建的媒体文件分类工具图形界面
//! 支持 i18n、主题切换、多页面导航

slint::include_modules!();

fn main() -> anyhow::Result<()> {
    let main_window = MainWindow::new()?;

    // 设置默认值
    main_window.set_working_directory("".into());

    // 设置回调处理

    // 浏览目录
    let main_window_weak = main_window.as_weak();
    main_window.on_browse_directory(move || {
        if let Some(window) = main_window_weak.upgrade() {
            // TODO: 实现目录选择对话框
            // 临时设置一个测试目录
            window.set_working_directory("/path/to/media".into());
        }
    });

    // 开始工作
    let main_window_weak = main_window.as_weak();
    main_window.on_start_work(move || {
        if let Some(window) = main_window_weak.upgrade() {
            window.set_app_state(AppState::Working);
            window.set_progress(0.0);
            window.set_log_content("开始处理...\n".into());

            // TODO: 实现实际的文件处理逻辑
            // 模拟处理完成
            window.set_progress(1.0);
            window.set_app_state(AppState::Completed);
            window.set_show_stats_popup(true);
            window.set_stats(Statistics {
                total: 100,
                success: 85,
                skipped: 10,
                renamed: 3,
                failed: 2,
            });
        }
    });

    // 切换日志显示
    let main_window_weak = main_window.as_weak();
    main_window.on_toggle_log(move || {
        if let Some(window) = main_window_weak.upgrade() {
            let current = window.get_show_log();
            window.set_show_log(!current);
        }
    });

    // 关闭统计弹窗
    let main_window_weak = main_window.as_weak();
    main_window.on_close_stats_popup(move || {
        if let Some(window) = main_window_weak.upgrade() {
            window.set_show_stats_popup(false);
            window.set_app_state(AppState::Idle);
        }
    });

    // 切换到配置页面
    let main_window_weak = main_window.as_weak();
    main_window.on_go_to_config(move || {
        if let Some(window) = main_window_weak.upgrade() {
            window.set_current_page(PageType::Config);
            
            // 加载配置规则
            // TODO: 从实际配置加载
            let rules = std::rc::Rc::new(slint::VecModel::from(vec![
                RuleItem {
                    id: 1,
                    name: "高质量照片".into(),
                    description: "大尺寸照片分类".into(),
                    extensions: "jpg,jpeg,png".into(),
                    directory_template: "Photos/{year}/{month}".into(),
                    min_size: "1MB".into(),
                    max_size: "".into(),
                    enabled: true,
                },
                RuleItem {
                    id: 2,
                    name: "RAW照片".into(),
                    description: "RAW格式照片".into(),
                    extensions: "nef,cr2,arw,dng".into(),
                    directory_template: "RAW/{year}/{month}/{day}".into(),
                    min_size: "".into(),
                    max_size: "".into(),
                    enabled: true,
                },
                RuleItem {
                    id: 3,
                    name: "视频".into(),
                    description: "视频文件".into(),
                    extensions: "mp4,mov,avi,mkv".into(),
                    directory_template: "Videos/{year}".into(),
                    min_size: "".into(),
                    max_size: "".into(),
                    enabled: true,
                },
            ]));
            window.set_rules(rules.into());
        }
    });

    // 返回主页面
    let main_window_weak = main_window.as_weak();
    main_window.on_go_to_main(move || {
        if let Some(window) = main_window_weak.upgrade() {
            window.set_current_page(PageType::Main);
        }
    });

    // 添加规则
    let main_window_weak = main_window.as_weak();
    main_window.on_add_rule(move || {
        if let Some(window) = main_window_weak.upgrade() {
            // 重置表单
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

    // 保存新规则
    let main_window_weak = main_window.as_weak();
    main_window.on_save_new_rule(move || {
        if let Some(window) = main_window_weak.upgrade() {
            // TODO: 实际保存规则到配置文件
            window.set_show_add_rule_popup(false);
            
            // 重新加载规则列表
            // 这里应该调用 go_to_config 的逻辑重新加载
        }
    });

    // 取消添加规则
    let main_window_weak = main_window.as_weak();
    main_window.on_cancel_add_rule(move || {
        if let Some(window) = main_window_weak.upgrade() {
            window.set_show_add_rule_popup(false);
        }
    });

    // 删除规则
    let main_window_weak = main_window.as_weak();
    main_window.on_delete_rule(move |rule_id| {
        if let Some(_window) = main_window_weak.upgrade() {
            // TODO: 实际删除规则
            println!("Delete rule: {}", rule_id);
        }
    });

    // 切换主题
    let main_window_weak = main_window.as_weak();
    main_window.on_change_theme(move |theme| {
        if let Some(window) = main_window_weak.upgrade() {
            window.set_theme_mode(theme);
            // TODO: 实际应用主题切换
        }
    });

    // 切换语言
    let main_window_weak = main_window.as_weak();
    main_window.on_change_language(move |lang| {
        if let Some(window) = main_window_weak.upgrade() {
            // TODO: 加载对应语言的 i18n 字符串
            if lang == "en" {
                window.set_i18n(I18nStrings {
                    app_title: "🎬 MediaClassifier".into(),
                    working_directory: "Working Directory".into(),
                    select_directory: "Select Directory".into(),
                    start_working: "Start".into(),
                    show_details: "Show Details".into(),
                    hide_details: "Hide Details".into(),
                    progress_label: "Progress".into(),
                    stats_title: "📊 Completed".into(),
                    stats_total: "Total".into(),
                    stats_success: "Success".into(),
                    stats_renamed: "Renamed".into(),
                    stats_skipped: "Skipped".into(),
                    stats_failed: "Failed".into(),
                    stats_close: "Close".into(),
                    config_title: "⚙️ Configuration".into(),
                    config_add: "➕ Add Rule".into(),
                    config_back: "← Back".into(),
                    config_rule_name: "Name".into(),
                    config_rule_desc: "Description".into(),
                    config_rule_ext: "Extensions".into(),
                    config_rule_template: "Template".into(),
                    config_rule_min_size: "Min Size".into(),
                    config_rule_max_size: "Max Size".into(),
                    config_rule_enabled: "Enabled".into(),
                    config_save: "Save".into(),
                    config_cancel: "Cancel".into(),
                    nav_config: "⚙️ Config".into(),
                    nav_main: "🏠 Home".into(),
                    theme_auto: "Auto".into(),
                    theme_light: "Light".into(),
                    theme_dark: "Dark".into(),
                });
            }
        }
    });

    // 运行应用
    main_window.run()?;
    Ok(())
}
