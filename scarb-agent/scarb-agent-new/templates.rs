use handlebars::Handlebars;

pub(crate) fn get_template_engine() -> handlebars::Handlebars<'static> {
    let mut registry = Handlebars::new();
    registry
        .register_template_string(
            "dockerfile",
            include_str!("templates/Dockerfile.hbs").to_owned(),
        )
        .unwrap();
    registry
        .register_template_string(
            "requirements",
            include_str!("templates/requirements.hbs").to_owned(),
        )
        .unwrap();
    registry
        .register_template_string(
            "pre-commit",
            include_str!("templates/.pre-commit-config.hbs").to_owned(),
        )
        .unwrap();
    registry
        .register_template_string("readme", include_str!("templates/README.hbs").to_owned())
        .unwrap();
    registry
        .register_template_string(
            "cloudbuild",
            include_str!("templates/cloudbuild.hbs").to_owned(),
        )
        .unwrap();
    registry
        .register_template_string(
            "run-service",
            include_str!("templates/run-service.hbs").to_owned(),
        )
        .unwrap();
    registry
}
