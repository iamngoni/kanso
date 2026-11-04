//! The PDF worker's entry point; everything is in `kanso_pdf::worker`.

fn main() -> std::process::ExitCode {
    kanso_pdf::worker::run()
}
