fn main() {
    // 编译 Slint UI，并捆绑翻译文件
    let config = slint_build::CompilerConfiguration::new()
        .with_bundled_translations("lang");
    slint_build::compile_with_config("ui/main_window.slint", config)
        .expect("Failed to compile Slint UI with bundled translations");

    // Windows 平台设置应用图标
    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("../../assets/icon.ico");
        res.compile().ok();
    }
}
