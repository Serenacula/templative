use anyhow::Result;

use crate::registry::Registry;

pub fn cmd_remove(template_names: Vec<String>) -> Result<()> {
    let mut registry = Registry::load()?;
    for name in &template_names {
        registry.remove(name)?;
    }
    registry.save()?;
    for name in &template_names {
        println!("removed {}", name);
    }
    Ok(())
}
