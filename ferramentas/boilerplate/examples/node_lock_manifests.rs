use rust_boilerplate::{
    config::Database,
    generator::{integrations::node::manifest, render::node_engine},
};
use serde_json::{Value, json};
use std::collections::BTreeMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut result = BTreeMap::<String, Value>::new();
    for (id, source) in [
        (
            "vanilla-js",
            include_str!("../templates/vanilla-js/package.json"),
        ),
        (
            "react-vite-ts",
            include_str!("../templates/react-vite-ts/package.json"),
        ),
        ("next-ts", include_str!("../templates/next-ts/package.json")),
        (
            "express-ts",
            include_str!("../templates/express-ts/package.json"),
        ),
        ("nest-ts", include_str!("../templates/nest-ts/package.json")),
    ] {
        let source = source
            .replace("{{PROJECT_NAME}}", id)
            .replace("{{PACKAGE_TYPE}}", "module")
            .replace("{{NODE_ENGINE}}", &node_engine("24"));
        let base: Value = serde_json::from_str(&source)?;
        result.insert(format!("templates/{id}/package-lock.json"), base);
        let lock = match id {
            "express-ts" => "express",
            "nest-ts" => "nest",
            "next-ts" => "next",
            _ => continue,
        };
        let mut superset = if id == "next-ts" {
            manifest(&source, Database::None, false, true)?
        } else {
            let mut package = manifest(&source, Database::Postgres, true, true)?;
            let mongo = manifest(&source, Database::Mongodb, false, false)?;
            package["dependencies"]
                .as_object_mut()
                .unwrap()
                .extend(mongo["dependencies"].as_object().unwrap().clone());
            package
        };
        superset["name"] = json!(id);
        result.insert(format!("integrations/node/{lock}-lock.json"), superset);
    }
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}
