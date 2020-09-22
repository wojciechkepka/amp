use amp;

fn main() {
    env_logger::builder().format_module_path(false).init();
    amp::interactive::AmpCli::run()
}
