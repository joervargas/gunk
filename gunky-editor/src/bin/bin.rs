use gunk_engine::{
    core::application,
};


pub fn main()
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
