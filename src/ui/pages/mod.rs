pub(super) mod home;
pub(super) mod man;
pub(super) mod search;
mod utils;

pub(super) use home::{HomePage, HomePageState};
pub(super) use man::{ManPage, ManPageState};
// pub(super) use search::{SearchPage, SearchPageState};

use super::{App, app::AppContext, events::EventHandler};

pub(super) enum Page {
    None,
    Home(HomePageState),
    // SearchPage(SearchPageState),
    Man(ManPageState),
}

fn switch_page(ctx: &mut AppContext, next_page: Page) {
    let prev_page = std::mem::replace(&mut ctx.current_page, next_page);

    match prev_page {
        Page::Home(state) => {
            HomePageState::on_drop(ctx);
        }
        // Page::SearchPage(state) => {
        //     state.on_drop(ctx);
        // }
        Page::Man(state) => {
            ManPageState::on_drop(ctx);
        }
        Page::None => {}
    }
}
