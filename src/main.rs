use regex::{Captures, Regex};
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use std::string::String;
use std::vec::Vec;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
    /// The path to the file to read
    #[structopt(parse(from_os_str))]
    #[structopt(short = "f", long = "file")]
    file_path: std::path::PathBuf,
    #[structopt(parse(from_os_str))]
    #[structopt(short = "r", long = "resource_folder")]
    image_repo: std::path::PathBuf,
}

#[derive(Debug)]
struct Link {
    is_local: bool,
    location: String,
}

fn find_links(text: String) -> Vec<Link> {
    let mut links = Vec::new();
    let search = r"(?P<format>!\[[a-z]*\])\((?P<link>[a-za-z:/\-\._0-9]*/(?P<file_name>[A-Za-z_0-9]+.(png|gif|svg)))\)";
    let regex = Regex::new(search).unwrap();

    let captures = regex.captures_iter(&text);

    for capture in captures {
        let location = capture["link"].to_string();
        let link = Link {
            is_local: !location.starts_with("http"),
            location: location,
        };
        print!("{}\n", link.location);
        links.push(link);
    }
    return links;
}

async fn download_from_web_write_on_disk(
    url: String,
    resource_repo: &PathBuf,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let mut res = reqwest::get(&url).await?;

    let matcher = Regex::new(r"(?P<file_name>[A-Za-z_0-9\-]+.(png|gif|svg)$)").unwrap();
    let capture = matcher.captures(&url);

    match capture {
        Some(captured) => {
            let filename = &captured["file_name"];

            let new_path = resource_repo.as_os_str();
            let mut path = PathBuf::new();
            path.push(new_path);
            path.push(filename);
            let file_path = path.to_str().unwrap();

            let mut buffer = File::create(String::from(file_path))?;

            while let Some(item) = res.chunk().await? {
                buffer.write(&item);
            }
        }
        None => panic!("Unsuported url: {}", url),
    };

    Ok(())
}

fn replace_url_paths_with_new_references(content: String, image_repo: &PathBuf) {
    let matcher = Regex::new(r"(?P<format>!\[[a-z]*\])\((?P<link>[a-za-z:/\-\._0-9]*/(?P<file_name>[A-Za-z_0-9]+.(png|gif|svg)))\)").unwrap();
    let path = image_repo.to_str().unwrap();
    // let replace_str : String = format!("$format!{}/$file_name)", path);

    let result = matcher.replace_all(&content, |caps: &Captures| {
        format!("{}({}{})", &caps["format"], &path, &caps["file_name"])
    });
    print!("{}", result);
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::from_args();
    let mut file = File::open(args.file_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let links = find_links(contents.to_string());

    if args.image_repo.exists() {
        for link in links {
            download_from_web_write_on_disk(link.location, &args.image_repo).await;
        }

        replace_url_paths_with_new_references(contents.to_string(), &args.image_repo);
    } else {
        panic!("Image repo path does not exist");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn find_empty_links() {
        let links = super::find_links(String::from(""));
        assert_eq!(links.len(), 0);
    }

    #[test]
    fn find_one_link() {
        let links = super::find_links(String::from("![](https://png)"));

        assert_eq!(links[0].location, "https://png");
        assert!(!links[0].is_local);
        assert_eq!(links.len(), 1);
    }

    #[test]
    fn find_one_link_among_text() {
        let links = super::find_links(String::from("asdfasd \n ![](https://png) asdfasdf \n"));
        assert_eq!(links.len(), 1);
    }

    #[test]
    fn find_many_links() {
        let links = super::find_links(String::from(
            "![](https://png)![](https://png) sdfasdfas ![](/home/test)",
        ));
        assert_eq!(links.len(), 3);
        assert_eq!(links[0].location, "https://png");
        assert_eq!(links[1].location, "https://png");
        assert_eq!(links[2].location, "/home/test");
        assert!(links[2].is_local);
    }
}

