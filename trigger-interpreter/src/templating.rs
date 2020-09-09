use anyhow::Error;
use handlebars::Handlebars;
use serde_json::json;

pub fn render_template(
    action_configuration: String,
    trigger_data: String,
) -> Result<String, Error> {
    let reg = Handlebars::new();

    let rendered_template =
        reg.render_template(action_configuration.as_str(), &json!(trigger_data))?;

    Ok(rendered_template)
}
