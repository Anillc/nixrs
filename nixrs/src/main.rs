use crate::store::Store;

mod utils;
mod context;
mod store;

fn main() {
    let store = Store::new("daemon").unwrap();
    dbg!(store);
}
