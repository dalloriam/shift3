use anyhow::Error;
use handlebars::Handlebars;
use serde_json::Value;

pub fn render_template(
    action_configuration: String,
    trigger_data: String,
) -> Result<String, Error> {
    let reg = Handlebars::new();

    let json_value: Value = serde_json::from_str(&trigger_data)?;

    let rendered_template = reg.render_template(action_configuration.as_str(), &json_value)?;

    Ok(rendered_template)
}
