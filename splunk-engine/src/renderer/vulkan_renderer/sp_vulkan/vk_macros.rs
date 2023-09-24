
/// ### vk_check!( ... )
/// *Logs an error if present in VkResult\<()\>*
#[macro_export]
macro_rules! vk_check {
    ( $vk_result:expr ) => 
    {
        match $vk_result
        {
            Ok(obj) => { Some(obj) },
            Err(e) => { $crate::log_err!(e); None }
        }
    };
}

#[macro_export]
macro_rules! vk_validate_info {
    () => {};
    ( $x:expr ) => 
    {
        use colored::Colorize;

        let label: &str = "Info: ";
        let data = format!("{}", $x );
        
        $crate::logger!(label.blue().bold(), data.blue());
    };
    ( $( $x:expr ),* ) =>
    {
        use colored::Colorize;

        let mut t = std::string::String::from("");

        let label: &str = "Info: ";
        $( t.push_str(format!("{}", $x ).as_str()); )*
        
        $crate::logger!(label.blue().bold(), t.blue());
    };
}

#[macro_export]
macro_rules! vk_validate_warn {
    () => {};
    ( $x:expr ) => 
    {
        use colored::Colorize;

        let label: &str = "Warning: ";
        let data = format!("{}", $x );
        
        $crate::logger!(label.yellow().bold(), data.yellow());
    };
    ( $( $x:expr ),* ) =>
    {
        use colored::Colorize;

        let mut t = std::string::String::from("");

        let label: &str = "Warning: ";
        $( t.push_str(format!("{}", $x ).as_str()); )*
        
        $crate::logger!(label.yellow().bold(), t.yellow());
    };
}

#[macro_export]
macro_rules! vk_validate_err {
    () => {};
    ( $x:expr ) => 
    {
        use colored::Colorize;
        
        let label: &str = "Error: ";
        let data = format!("{}", $x );

        $crate::logger!(label.red().bold(), data.red());
    };
    ( $( $x:expr ),* ) =>
    {
        use colored::Colorize;

        let mut t = std::string::String::from("");

        let label: &str = "Error: ";
        $( t.push_str(format!("{}", $x ).as_str()); )*

        $crate::logger!(label.red().bold(), t.red());
    };
}