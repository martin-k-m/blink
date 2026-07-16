use anyhow::Result;

use crate::cli::DashboardArgs;

pub fn run(args: DashboardArgs) -> Result<()> {
    blink_dashboard::run_dashboard(args.path)
}
