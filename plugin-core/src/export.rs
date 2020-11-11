#[macro_export]
macro_rules! export {
    (($($action_struct:ty)*), ($($trigger_struct:ty)*)) => {
        #[no_mangle]
        pub extern "C" fn init_plugin() -> Box<$crate::Plugin> {
            let mut actions: Vec<Box<dyn $crate::ActionPlugin + 'static>> = vec![$(Box::from(<$action_struct>::default()),)*];
            let mut triggers: Vec<Box<dyn $crate::TriggerPlugin + 'static>> = vec![$(Box::from(<$trigger_struct>::default()),)*];
            Box::new(
                $crate::Plugin::new(actions, triggers)
            )
        }
    }
}
