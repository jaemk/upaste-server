//! Routes
//!  - Map url endpoints to our `handlers`
use router::Router;
use handlers;


/// Mount our urls and routers on our `Router`
pub fn mount(router: &mut Router) {
    router.post("/new", handlers::new_paste, "new_paste");
    router.get("/:key", handlers::view_paste, "view_paste");
    router.get("/raw/:key", handlers::view_paste_raw, "get_paste_raw");
    router.get("/new", handlers::home, "edit_new_paste");
    router.get("/", handlers::home, "home");
}
