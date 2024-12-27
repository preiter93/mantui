pub(super) mod desc;
pub(super) mod help;
pub(super) mod home;
pub(super) mod list;
pub(super) mod search;
mod utils;

use desc::DescPageState;
pub(super) use home::{HomePage, HomePageState};
pub(super) use list::{ListPage, ListPageState};
// pub(super) use search::{SearchPage, SearchPageState};

use super::{App, app::AppContext, events::EventHandler};

pub(super) enum Page {
    None,
    Home(HomePageState),
    List(ListPageState),
    Desc(DescPageState),
}

fn drop_page(ctx: &mut AppContext) {
    match &ctx.current_page {
        Page::Home(state) => {
            HomePageState::on_drop(ctx);
        }
        Page::List(state) => {
            ListPageState::on_drop(ctx);
        }
        Page::Desc(state) => {
            DescPageState::on_drop(ctx);
        }
        Page::None => {}
    }
}

// fn switch_page(ctx: &mut AppContext, next_page: Page) {
//     let prev_page = std::mem::replace(&mut ctx.current_page, next_page);
//
//     match prev_page {
//         Page::Home(state) => {
//             HomePageState::on_drop(ctx);
//         }
//         // Page::SearchPage(state) => {
//         //     state.on_drop(ctx);
//         // }
//         Page::Man(state) => {
//             ManPageState::on_drop(ctx);
//         }
//         Page::None => {}
//     }
// }
