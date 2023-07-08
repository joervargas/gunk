pub mod resource;

use gunk_engine::{
    core::application
};

#[cfg(target_arch="wasm32")]
use wasm_bindgen::prelude::*;

#[cfg_attr(target_arch="wasm32", wasm_bindgen(start))]
pub fn run_web()
{
    let app_config = application::AppConfig{ 
        width: 1000, height: 800, 
        title: String::from("Gunky Editor"), 
        web_id: String::from("gunky-editor")
    };

    let (mut app, evloop) = application::Application::new(app_config);
    app.init();
    app.run(evloop);
}