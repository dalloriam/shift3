use anyhow::Error;
use handlebars::Handlebars;
use serde_json::{self, json};

use protocol::rule::ActionConfiguration;

pub fn render_template(
    action_configuration: ActionConfiguration,
    trigger_data: String,
) -> Result<ActionConfiguration, Error> {
    let string_template = serde_json::to_string(&action_configuration)?;

    let reg = Handlebars::new();
    let rendered_template = reg.render_template(string_template.as_str(), &json!(trigger_data))?;

    let result: ActionConfiguration = serde_json::from_str(&rendered_template)?;

    Ok(result)
}
