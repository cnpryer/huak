use crate::{errors::HuakResult, project::Project};

pub fn activate_project_venv(project: &Project) -> HuakResult<()> {
    let venv = project
        .venv()
        .as_ref()
        .expect("`Project::from` creates venv if it doesn't exists.");

    println!("Venv activated: {}", venv.path.display());

    venv.activate()?;

    println!("Venv deactivated.");
    Ok(())
}
