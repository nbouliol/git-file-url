use anyhow::anyhow;
use git2::Repository;
use std::fmt;
use std::path::PathBuf;
use structopt::StructOpt;

pub type Result<T> = anyhow::Result<T>;

fn github(repo_url: &str, branch_or_commit: &str, path: &str, line: Option<u16>) -> Result<String> {
    let mut url = String::new();

    fmt::write(
        &mut url,
        format_args!(
            "{url}/blob/{branch}/{path}",
            url = repo_url,
            branch = branch_or_commit,
            path = path
        ),
    )?;

    if let Some(line) = line {
        fmt::write(&mut url, format_args!("#L{}", line))?;
    }

    Ok(url)
}

fn gitlab(repo_url: &str, branch_or_commit: &str, path: &str, line: Option<u16>) -> Result<String> {
    let mut url = String::new();

    fmt::write(
        &mut url,
        format_args!(
            "{url}/-/blob/{branch}/{path}",
            url = repo_url,
            branch = branch_or_commit,
            path = path
        ),
    )?;

    if let Some(line) = line {
        fmt::write(&mut url, format_args!("#L{}", line))?;
    }

    Ok(url)
}

fn get_url(url: Option<&str>) -> Result<String> {
    if let Some(url) = url {
        let parsed_url: String;

        if url.starts_with("git@") {
            // ssh
            let stripped = url
                .strip_prefix("git@")
                .and_then(|url| url.strip_suffix(".git"));
            if let Some(url) = stripped {
                parsed_url = format!("https://{}", url.replace(":", "/"));
            } else {
                return Err(anyhow!("Invalid remote form"));
            }
        } else if let Some(stripped) = url.strip_suffix(".git") {
            parsed_url = stripped.to_owned();
        // parsed_url = url.strip_suffix(".git").unwrap().to_string();
        } else {
            return Err(anyhow!("cannot get repository url"));
        }
        if !parsed_url.contains("http") {
            return Err(anyhow!("Invalid remote form"));
        }
        Ok(parsed_url)
    } else {
        Err(anyhow!("cannot get repository url"))
    }
}

fn main() -> Result<()> {
    let opt = Opt::from_args();

    let repo = match Repository::open(".") {
        Ok(repo) => repo,
        Err(_) => Repository::discover(".")?, // open repossitory if in subfolder
    };

    if repo.is_bare() {
        return Err(anyhow!("Cannot use a bare repository"));
    } else if repo.is_empty()? {
        return Err(anyhow!("Cannot use an empty repository"));
    }

    let workdir = repo.workdir().unwrap();
    let head = repo.head()?;

    let branch_or_commit: String;

    if head.is_branch() {
        branch_or_commit = if let Some(name) = repo.head()?.name() {
            name.replace("refs/heads/", "")
        } else {
            return Err(anyhow!("Cannot get branch or commit"));
        }
    } else {
        branch_or_commit = repo.head()?.peel_to_commit()?.id().to_string();
    }

    let absolute_file = opt.file.canonicalize()?;
    let file = absolute_file.strip_prefix(workdir)?.to_str().unwrap();

    let remote = repo.find_remote("origin")?;

    let url: String;
    if let Some(param_url) = opt.url {
        url = param_url;
    } else {
        url = get_url(remote.url())?;
    }

    if let Some(platform) = opt.platform {
        let file_url = match platform {
            Platform::Github => github(&url, &branch_or_commit, file, opt.line)?,
            Platform::Gitlab => gitlab(&url, &branch_or_commit, file, opt.line)?,
        };

        println!("{}", file_url);
    } else if url.contains("github") {
        let file_url = github(&url, &branch_or_commit, file, opt.line).unwrap();
        println!("{}", file_url);
    } else if url.contains("gitlab") {
        let file_url = gitlab(&url, &branch_or_commit, file, opt.line).unwrap();
        println!("{}", file_url);
    } else {
        return Err(anyhow!("unknown url, try passing --url param"));
    }

    Ok(())
}

#[derive(Debug, StructOpt)]
#[structopt(name = "git url", about = "Get file url")]
struct Opt {
    #[structopt(parse(from_os_str))]
    file: PathBuf,

    #[structopt(short, long, help = "File line")]
    line: Option<u16>,

    #[structopt(
        short,
        long,
        parse(try_from_str = parse_platform),
        help="Platform : gitlab or github" 
    )]
    platform: Option<Platform>,

    #[structopt(long, help = "Repository url")]
    url: Option<String>,
}

#[derive(Debug)]
enum Platform {
    Github,
    Gitlab,
}

fn parse_platform(p: &str) -> Result<Platform> {
    match p.to_lowercase().as_str() {
        "github" => Ok(Platform::Github),
        "gitlab" => Ok(Platform::Gitlab),
        _ => Err(anyhow!("Invalid platform {}", p)),
    }
}

#[cfg(test)]
mod test {

    #[test]
    fn github() {
        let url = crate::github(
            "https://github.com/nbouliol/git-files",
            "master",
            "readme.md",
            None,
        );

        assert!(url.is_ok());
        assert!(url.unwrap() == "https://github.com/nbouliol/git-files/blob/master/readme.md");
    }

    #[test]
    fn github_with_line() {
        let url = crate::github(
            "https://github.com/nbouliol/git-files",
            "master",
            "readme.md",
            Some(5),
        );

        assert!(url.is_ok());
        assert!(url.unwrap() == "https://github.com/nbouliol/git-files/blob/master/readme.md#L5")
    }

    #[test]
    fn gitlab() {
        let url = crate::gitlab(
            "https://gitlab.com/nbouliol/git-files",
            "master",
            "readme.md",
            None,
        );

        assert!(url.is_ok());
        assert!(url.unwrap() == "https://gitlab.com/nbouliol/git-files/-/blob/master/readme.md");
    }

    #[test]
    fn gitlab_with_line() {
        let url = crate::gitlab(
            "https://gitlab.com/nbouliol/git-files",
            "master",
            "readme.md",
            Some(5),
        );

        assert!(url.is_ok());
        assert!(url.unwrap() == "https://gitlab.com/nbouliol/git-files/-/blob/master/readme.md#L5")
    }

    #[test]
    fn get_url_none() {
        let url = crate::get_url(None);
        assert!(!url.is_ok());
    }

    #[test]
    fn get_url_http() {
        let url = Some("https://github.com/someone/repo.git");
        let get = crate::get_url(url);
        assert!(get.is_ok());
        assert!(get.unwrap() == String::from("https://github.com/someone/repo"));

        let url = Some("https://github.com/someone/repo");
        let get = crate::get_url(url);
        assert!(!get.is_ok());
    }

    #[test]
    fn get_url_ssh() {
        let url = Some("git@github.com:someone/repo.git");
        let get = crate::get_url(url);
        assert!(get.is_ok());
        assert!(get.unwrap() == String::from("https://github.com/someone/repo"));

        let url = Some("github.com:someone/repo.git");
        let get = crate::get_url(url);
        assert!(!get.is_ok());

        let url = Some("git@github.com:someone/repo");
        let get = crate::get_url(url);
        assert!(!get.is_ok());
    }
}
