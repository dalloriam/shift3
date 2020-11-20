use protocol::{Rule, TriggerConfiguration};

use toolkit::db::sled::{EntityStore, SledStore};

fn main() {
    let store = SledStore::new("./test.db").unwrap();

    let rules: EntityStore<Rule> = store.entity("Rule").unwrap();

    let r = Rule {
        trigger_config_id: 1,
        action_config:
            "{\"body\": \"New File: {{file_name}}\", \"title\": \"ShifTTT Notification\"}".into(),
        action_type: String::from("notify"),
    };

    let r_id = rules.insert(&r).unwrap();

    let trigger_cfgs: EntityStore<TriggerConfiguration> =
        store.entity("TriggerConfiguration").unwrap();

    let trigger_cfg = TriggerConfiguration {
        id: 1,
        rule: r_id.clone(),
        trigger_type: "directory_watch".into(),
        data: "{\"directory\": \"/home/wdussault/temp\"}".into(),
    };
    trigger_cfgs.insert(&trigger_cfg).unwrap();

    trigger_cfgs.flush().unwrap();
    rules.flush().unwrap();
}
