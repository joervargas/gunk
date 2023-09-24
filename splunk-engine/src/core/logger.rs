
pub extern crate colored;

// enum LogLevel
// {
//     Info,
//     Warn,
//     Error,
// }

#[macro_export]
macro_rules! logger {
    () => {};
    ( $x:expr ) => 
    {
        println!( "{}", format!("{}", $x).as_str() );
    };
    ( $( $x:expr ),* ) =>
    {
        let mut t = std::string::String::from("");

        $( t.push_str(format!("{}", $x ).as_str()); )*
        println!("{}", t );
    };
}

/// ### log_info!( ... )
/// *Logs messages as information.<br> Will print blue text in the terminal.*
#[macro_export]
macro_rules! log_info {
    () => {};
    ( $x:expr ) => 
    {
        let label: &str = "Info: ";
        let data = format!("{}", $x );
        let meta_data = format!("\n\tfile: {} line: {}", file!(), line!() );
        
        $crate::logger!(
            // label.blue().bold(),
            $crate::core::logger::colored::Colorize::bold($crate::core::logger::colored::Colorize::blue(label)),
            // data.blue(),
            $crate::core::logger::colored::Colorize::blue(data.as_str()),
            // meta_data.blue()
            $crate::core::logger::colored::Colorize::blue(meta_data.as_str())
        );
    };
    ( $( $x:expr ),* ) =>
    {
        // use colored::Colorize;

        let mut t = std::string::String::from("");

        let label: &str = "Info: ";
        $( t.push_str(format!("{}", $x ).as_str()); )*
        let meta_data = format!("\n\tfile: {} line: {}", file!(), line!() );
        
        $crate::logger!(
            // label.blue().bold(),
            $crate::core::logger::colored::Colorize::bold($crate::core::logger::colored::Colorize::blue(label)),
            // t.blue(), 
            $crate::core::logger::colored::Colorize::blue(tas_str()),
            // meta_data.blue()
            $crate::core::logger::colored::Colorize::blue(meta_data.as_str())
        );
    };
}

/// ### log_warn!( ... )
/// *Logs messages as warnings.<br> Will print yellow text in the terminal.*
#[macro_export]
macro_rules! log_warn {
    () => {};
    ( $x:expr ) => 
    {
        // use colored::Colorize;

        let label: &str = "Warning: ";
        let data = format!("{}", $x );
        let meta_data = format!("\n\tfile: {} line: {}", file!(), line!() );
        
        $crate::logger!(
            // label.yellow().bold(),
            $crate::core::logger::colored::Colorize::bold($crate::core::logger::colored::Colorize::yellow(label)),
            // data.yellow(), 
            $crate::core::logger::colored::Colorize::yellow(data.as_str()),
            // meta_data.yellow()
            $crate::core::logger::colored::Colorize::yellow(meta_data.as_str())
        );
    };
    ( $( $x:expr ),* ) =>
    {
        // use colored::Colorize;

        let mut t = std::string::String::from("");

        let label: &str = "Warning: ";
        $( t.push_str(format!("{}", $x ).as_str()); )*
        let meta_data = format!("\n\tfile: {} line: {}", file!(), line!() );
        
        $crate::logger!(
            // label.yellow().bold(), 
            $crate::core::logger::colored::Colorize::bold($crate::core::logger::colored::Colorize::yellow(label)),
            // t.yellow(), 
            $crate::core::logger::colored::Colorize::yellow(t.as_str()),
            // meta_data.yellow()
            $crate::core::logger::colored::Colorize::yellow(meta_data.as_str())
        );
    };
}

/// ### log_err!( ... )
/// *Logs messages as errors.<br> Will print red text in the terminal.*
#[macro_export]
macro_rules! log_err {
    () => {};
    ( $x:expr ) => 
    {
        // use colored::Colorize;
        
        let label = "Error: ";
        let data = format!("{}", $x );
        let meta_data = format!("\n\tfile: {} line: {}", file!(), line!() );

        $crate::logger!(
            // label.red().bold(), 
            $crate::core::logger::colored::Colorize::bold($crate::core::logger::colored::Colorize::red(label)),
            // data.red(), 
            $crate::core::logger::colored::Colorize::red(data.as_str()),
            // meta_data.red()
            $crate::core::logger::colored::Colorize::red(meta_data.as_str())
        );
    };
    ( $( $x:expr ),* ) =>
    {
        // use colored::Colorize;

        let mut t = std::string::String::from("");

        let label: &str = "Error: ";
        $( t.push_str(format!("{}", $x ).as_str()); )*
        let meta_data = format!("\n\tfile: {} line: {}", file!(), line!() );

        $crate::logger!(
            // label.red().bold(), 
            $crate::core::logger::colored::Colorize::bold($crate::core::logger::colored::Colorize::red(label)),
            // t.red(), 
            $crate::core::logger::colored::Colorize::red(t.as_str()),
            // meta_data.red()
            $crate::core::logger::colored::Colorize::red(meta_data.as_str())
        );
    };
}

/// ### check_err!( ... )
/// *Logs an error if present in Result\<()\>*
#[macro_export]
macro_rules! check_err {
    ( $result:expr ) => 
    {
        match $result
        {
            Ok(obj) => { Some(obj) },
            Err(e) => { $crate::log_err!(e); None }
        }
    };
}