use pixelforge_backend::models::project::CreateProjectRequest;

#[test]
fn project_name_should_have_minimum_length() {
    let request = CreateProjectRequest {
        name: "ab".to_owned(),
        description: None,
    };

    let result = request.validate();
    assert!(result.is_err());
}

#[test]
fn project_name_should_be_trimmed_and_accepted() {
    let request = CreateProjectRequest {
        name: "  Hero Banner  ".to_owned(),
        description: Some("  Marketing assets  ".to_owned()),
    };

    let validated = request.validate().expect("request should validate");
    assert_eq!(validated.name, "Hero Banner");
    assert_eq!(
        validated.description.expect("description should be present"),
        "Marketing assets"
    );
}
