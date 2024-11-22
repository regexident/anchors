use std::cell::RefCell;

use anchors::singlethread::*;

thread_local! {
    pub static ENGINE: RefCell<Engine> = RefCell::new(Engine::new());
}

fn main() {
    // important to call ENGINE.with before we create any Anchors, since the engine
    // must have been initialized for an anchor to be created.
    ENGINE.with(|engine| {
        let var = Var::new(1);
        let var_added = var.watch().map(|n| n + 1);
        println!("{:?}", engine.borrow_mut().get(&var_added));
    });
}
