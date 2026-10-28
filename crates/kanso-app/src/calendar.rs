//! The Journal's month calendar, apart from drawing (`calendar_view.rs`):
//! the grid and month arithmetic are `kanso_engine::calendar`; what is left
//! here asks the system which day its weeks start on.

pub use kanso_engine::calendar::{MonthGrid, add_months, shift_month, weekday_labels};

/// The first day of the week where the person is: the system's choice on
/// macOS, Monday elsewhere. 0 for Monday through 6 for Sunday.
// no-prop-test: asks the platform; `from_first_weekday` is the arithmetic.
pub fn system_week_start() -> usize {
    #[cfg(target_os = "macos")]
    {
        let calendar = objc2_foundation::NSCalendar::autoupdatingCurrentCalendar();
        kanso_engine::calendar::from_first_weekday(calendar.firstWeekday())
    }
    #[cfg(not(target_os = "macos"))]
    {
        0
    }
}
