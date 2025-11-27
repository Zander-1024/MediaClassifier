fn main() {
    // 编译 Slint UI
    slint_build::compile("ui/main_window.slint").expect("Slint build failed");

    // Windows 平台设置应用图标
    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("../../assets/icon.ico");
        res.compile().ok();
    }
}
