//! MediaClassifier GUI Application
//!
//! 使用 Slint 构建的媒体文件分类工具图形界面

slint::include_modules!();

fn main() -> anyhow::Result<()> {
    let main_window = MainWindow::new()?;

    // 设置默认值
    main_window.set_working_directory(".".into());
    if let Ok(config_path) = mc_lib::Config::default_config_path() {
        main_window.set_config_path(config_path.display().to_string().into());
    }
    main_window.set_status_message("就绪 - 请选择目录并开始扫描".into());

    // 设置回调处理
    let main_window_weak = main_window.as_weak();
    main_window.on_browse_directory(move || {
        if let Some(window) = main_window_weak.upgrade() {
            // TODO: 实现目录选择对话框
            window.set_status_message("目录选择功能开发中...".into());
        }
    });

    let main_window_weak = main_window.as_weak();
    main_window.on_browse_config(move || {
        if let Some(window) = main_window_weak.upgrade() {
            // TODO: 实现配置文件选择对话框
            window.set_status_message("配置文件选择功能开发中...".into());
        }
    });

    let main_window_weak = main_window.as_weak();
    main_window.on_scan_files(move || {
        if let Some(window) = main_window_weak.upgrade() {
            window.set_app_state(AppState::Scanning);
            window.set_status_message("正在扫描文件...".into());
            window.set_progress(0.0);

            // TODO: 实现文件扫描逻辑
            // 目前仅设置状态用于演示
            window.set_app_state(AppState::Idle);
            window.set_status_message("扫描完成".into());
        }
    });

    let main_window_weak = main_window.as_weak();
    main_window.on_start_classification(move || {
        if let Some(window) = main_window_weak.upgrade() {
            window.set_app_state(AppState::Processing);
            window.set_status_message("正在分类文件...".into());
            window.set_progress(0.0);

            // TODO: 实现文件分类逻辑
            // 目前仅设置状态用于演示
            window.set_app_state(AppState::Completed);
            window.set_status_message("分类完成".into());
        }
    });

    let main_window_weak = main_window.as_weak();
    main_window.on_stop_classification(move || {
        if let Some(window) = main_window_weak.upgrade() {
            window.set_app_state(AppState::Idle);
            window.set_status_message("操作已取消".into());
        }
    });

    let main_window_weak = main_window.as_weak();
    main_window.on_show_config(move || {
        if let Some(window) = main_window_weak.upgrade() {
            // TODO: 显示配置信息对话框
            window.set_status_message("配置查看功能开发中...".into());
        }
    });

    let main_window_weak = main_window.as_weak();
    main_window.on_reload_config(move || {
        if let Some(window) = main_window_weak.upgrade() {
            // TODO: 重新加载配置
            window.set_status_message("配置已重新加载".into());
        }
    });

    // 运行应用
    main_window.run()?;
    Ok(())
}
