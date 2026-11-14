// In release builds, do not open an extra console window next to the app on Windows.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    nurbit_desktop_lib::run()
}
