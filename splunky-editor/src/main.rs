use splunk_engine::core::application;

fn main() {
    println!("Hello, world!");

    let app_config = application::AppConfig
    {
        title: String::from("Splunk Editor"),
        width: 1000,
        height: 800,
        is_fullscreen: false,
        is_resizable: true,
        has_border: true
    };
    
    let (mut app, evloop) = application::Application::new(app_config);
    
    app.init();
    app.run(evloop);
}
