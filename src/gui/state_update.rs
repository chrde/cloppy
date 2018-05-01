use gui::wnd_proc::Event;
use file_listing::State;
use gui::context_stash::set_state;
use gui::status_bar;
use gui::list_view;
use gui::context_stash::CONTEXT_STASH;
use StateChange;

pub fn handle(event: Event) {
    let new_state = unsafe { Box::from_raw(event.w_param as *mut State) };
    CONTEXT_STASH.with(|context_stash| {
        let mut context_stash = context_stash.borrow_mut();
        match *new_state.status() {
            StateChange::NEW => {
                context_stash.as_mut().unwrap().state = new_state;
            },
            StateChange::UPDATE => {
                context_stash.as_mut().unwrap().state.update_with(*new_state)
            }
        }
    });
    status_bar::update_status_bar();
    list_view::update_list_view();
}