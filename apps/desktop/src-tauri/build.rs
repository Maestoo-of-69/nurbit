// Runs before compiling: Tauri reads tauri.conf.json and capabilities/ and
// embeds the app icon and Windows metadata into the executable.
fn main() {
    tauri_build::build()
}
