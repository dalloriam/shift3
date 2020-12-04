#[macro_export]
macro_rules! export {
    (($($action_struct:ty)*), ($($trigger_struct:ty)*)) => {
        #[no_mangle]
        pub extern "C" fn init_plugin() -> Box<$crate::Plugin> {
            match std::panic::catch_unwind(|| {
                let mut actions: Vec<Box<dyn $crate::ActionPlugin + 'static>> = vec![$(Box::from(<$action_struct>::default()),)*];
                let mut triggers: Vec<Box<dyn $crate::TriggerPlugin + 'static>> = vec![$(Box::from(<$trigger_struct>::default()),)*];
                Box::new(
                    $crate::Plugin::new(actions, triggers)
                )
            }) {
                Ok(ptr) => ptr,
                Err(e) => {
                    // TODO: Find way of logging this _properly_ in case of error.
                    eprintln!("FFI: caught unwinding panic in plugin [{}]", env!("CARGO_PKG_NAME"));
                    Box::new($crate::Plugin::new(Vec::new(), Vec::new()))
                }
            }
        }
    }
}
