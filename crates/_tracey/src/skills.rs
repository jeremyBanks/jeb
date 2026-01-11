//! Embedded skill files
//! [impl _trace.skills.embedded]

use std::fs;
use std::io;
use std::path::Path;

/// Embedded skill file content
/// [impl _trace.skills.embedded]
pub struct EmbeddedSkill {
    pub filename: &'static str,
    pub content: &'static str,
}

/// All embedded skills
/// [impl _trace.skills.embedded]
pub const SKILLS: &[EmbeddedSkill] = &[
    EmbeddedSkill {
        filename: "validate-requirements.md",
        content: include_str!("../.claude/skills/validate-requirements.md"),
    },
    EmbeddedSkill {
        filename: "work-requirements.md",
        content: include_str!("../.claude/skills/work-requirements.md"),
    },
    EmbeddedSkill {
        filename: "add-requirement.md",
        content: include_str!("../.claude/skills/add-requirement.md"),
    },
];

/// Install embedded skills to the project's .claude/skills/ directory
/// [impl _trace.cli.install-skills]
pub fn install_skills(target_dir: &Path) -> io::Result<()> {
    let skills_dir = target_dir.join(".claude").join("skills");

    // Check if any files already exist (fail before creating anything)
    for skill in SKILLS {
        let target_path = skills_dir.join(skill.filename);
        if target_path.exists() {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                format!("File already exists: {}", target_path.display()),
            ));
        }
    }

    // Create directory if it doesn't exist
    fs::create_dir_all(&skills_dir)?;

    // Write all skill files
    for skill in SKILLS {
        let target_path = skills_dir.join(skill.filename);
        fs::write(&target_path, skill.content)?;
        println!("Installed: {}", target_path.display());
    }

    println!("\nSkills installed to {}", skills_dir.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// [test _trace.skills.embedded]
    #[test]
    fn test_skills_are_embedded() {
        // Verify skills are properly embedded
        assert!(!SKILLS.is_empty(), "Should have embedded skills");

        for skill in SKILLS {
            assert!(!skill.filename.is_empty(), "Skill should have filename");
            assert!(!skill.content.is_empty(), "Skill should have content");
            assert!(skill.filename.ends_with(".md"), "Skills should be markdown files");
        }
    }

    /// [test _trace.skills.embedded]
    #[test]
    fn test_expected_skills_exist() {
        let filenames: Vec<&str> = SKILLS.iter().map(|s| s.filename).collect();

        assert!(filenames.contains(&"validate-requirements.md"));
        assert!(filenames.contains(&"work-requirements.md"));
        assert!(filenames.contains(&"add-requirement.md"));
    }

    /// [test _trace.cli.install-skills]
    #[test]
    fn test_install_fails_if_exists() {
        use std::env;

        // Create a temp directory
        let temp_dir = env::temp_dir().join("_trace_test_skills");
        let skills_dir = temp_dir.join(".claude").join("skills");
        let _ = fs::remove_dir_all(&temp_dir); // Clean up any previous runs

        // First install should succeed
        fs::create_dir_all(&skills_dir).unwrap();
        fs::write(skills_dir.join("validate-requirements.md"), "existing").unwrap();

        // Second install should fail
        let result = install_skills(&temp_dir);
        assert!(result.is_err(), "Should fail when files already exist");

        // Clean up
        let _ = fs::remove_dir_all(&temp_dir);
    }
}
