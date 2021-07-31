use std::path::{Path, PathBuf};
use std::process::Command;
use std::{env, fs, process};
use url::Url;

fn main() {
    let mut args = env::args();
    let program_name = args.next().expect("no program name");

    // If we don't have any arguments, print usage
    let input = args.next().unwrap_or_else(|| {
        eprintln!(
            "usage: {} [GIT_URL] [PROVIDER/OWNER/REPO] [init [FUNCTION_NAME]]",
            program_name
        );
        process::exit(1);
    });

    // Init command
    if input == "init" {
        let function_name = args.next().unwrap_or_else(|| "projx".to_string());
        println!(r##"{}() {{ cd "$({} $1)" }}"##, function_name, program_name);
        process::exit(0);
    }

    match run(input) {
        Ok(project_directory) => println!("{}", project_directory),
        Err(error) => {
            eprintln!("error: {}", error);
            process::exit(1);
        }
    }
}

enum Provider {
    Github,
    Gitlab,
}

struct Repository {
    owner: String,
    name: String,
    provider: Provider,
}

impl Repository {
    fn parse(input: String) -> Result<Self, String> {
        let (owner, name, provider) = if let Ok(url) = Url::parse(&input) {
            let mut path_segments = url.path_segments().ok_or("no base")?;
            let owner = path_segments.next().ok_or("no owner")?.to_string();
            let name = path_segments.next().ok_or("no repo")?.to_string();

            let provider = match url.host_str().ok_or("no host")? {
                "github.com" => Provider::Github,
                "gitlab.com" => Provider::Gitlab,
                _ => return Err(format!("unsupported git provider: {}", url)),
            };

            (owner, name, provider)
        } else {
            // If we're not parsing a URL, treat the input as "provider/owner/repo"
            let mut parts = input.split('/');
            let provider = parts.next().ok_or("no provider")?;
            let owner = parts.next().ok_or("no owner")?.to_string();
            let name = parts.next().ok_or("no repo")?.to_string();

            let provider = match provider {
                "github" => Provider::Github,
                "gitlab" => Provider::Gitlab,
                _ => return Err(format!("unknown provider: {}", provider)),
            };

            (owner, name, provider)
        };

        Ok(Repository {
            owner,
            name,
            provider,
        })
    }

    fn provider_str(&self) -> &str {
        match self.provider {
            Provider::Github => "github",
            Provider::Gitlab => "gitlab",
        }
    }

    fn directory(&self) -> PathBuf {
        Path::new(self.provider_str())
            .join(&self.owner)
            .join(&self.name)
    }

    fn url(&self) -> String {
        match self.provider {
            Provider::Github => format!("https://github.com/{}/{}", self.owner, self.name),
            Provider::Gitlab => format!("https://gitlab.com/{}/{}", self.owner, self.name),
        }
    }
}

fn run(input: String) -> Result<String, String> {
    let projects_base_directory = PathBuf::from(
        env::var("PROJX_DIR").map_err(|_| "PROJX_DIR environment variable not set".to_string())?,
    );

    if !projects_base_directory.is_dir() {
        return Err(format!(
            "PROJX_DIR is not a directory: {}",
            projects_base_directory.display()
        ));
    }

    let repository = Repository::parse(input)?;
    let project_directory = projects_base_directory.join(repository.directory());

    if !project_directory.join(".git").is_dir() {
        // Create the project directory
        fs::create_dir_all(&project_directory).map_err(|_| "unable to create directory")?;

        let cleanup_empty_dirs = || {
            let repo_dir = &project_directory;
            let _ = fs::remove_dir(&repo_dir);
            let owner_dir = repo_dir.parent().unwrap();
            let _ = fs::remove_dir(&owner_dir);
            let provider_dir = owner_dir.parent().unwrap();
            let _ = fs::remove_dir(&provider_dir).map_err(|e| format!("{}", e));
        };

        // Clone repo
        match Command::new("git")
            .arg("clone")
            .arg(&repository.url())
            .arg(&project_directory)
            .status()
        {
            Ok(status) => {
                if !status.success() {
                    cleanup_empty_dirs();
                    return Err(format!(
                        "git command was unsuccessful: {}",
                        status.code().unwrap_or(1)
                    ));
                }
            }
            Err(_) => {
                cleanup_empty_dirs();
                return Err("failed to execute git".to_string());
            }
        }
    }

    project_directory
        .to_str()
        .ok_or_else(|| format!("invalid path: {}", project_directory.display()))
        .map(|p| p.to_string())
}
