use crate::system::TriggerSystem;

use super::mock::Dummy;

#[test]
fn basic_test() {
    let sys = TriggerSystem::start::<Dummy, Dummy>(Default::default(), Default::default());
    sys.stop().unwrap();
}
