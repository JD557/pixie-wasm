extern crate yew;
extern crate pixie_wasm;

use yew::prelude::*;
use pixie_wasm::Model;

fn main() {
    yew::initialize();
    App::<Model>::new().mount_to_body();
    yew::run_loop();
}
