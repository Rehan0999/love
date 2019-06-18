//! tools for working with love projects
//!
//! supports love project folders and love `.love` packages

use failure::Error;
use version_lp::Version;
use zip;
use regex::Regex;

use std::path::{Path,PathBuf};
use std::fs::File;
use std::io::{Cursor,Read};

#[derive(Debug,Eq,PartialEq)]
pub enum Project {
    File(Version),
    Folder(Version),
    None
}

pub fn get_required_version<P : AsRef<Path>>(project_path : P) -> Result<Version,Error> {
    //! will look inside the `conf.lua` file for a project or `.love` file 
    //! to determine what version of love should be used
    //! 
    let path = PathBuf::from(project_path.as_ref());
    
    let content = match is_love_package(&path) {
        true => { get_file_contents_from_archive(&path,"conf.lua")? }
        false => { get_file_contents(&path,"conf.lua")? }
    };

    let re_version_assignment = Regex::new(r#"version *= *["|'](.*)["|']"#).unwrap();
    if let Some(version) = re_version_assignment.captures(&content) {
        let captured_version = version.get(1).unwrap().as_str().to_string();
        if let Some(version) = Version::from_str(&captured_version) {
            return Ok(version);
        }
    }

    Err(format_err!("Failed to determine the version"))
}

pub fn project_type<P : AsRef<Path>>(project_path : P) -> Result<Project,Error> {
    let path = PathBuf::from(project_path.as_ref());

    if is_love_package(&path) {
        let content = get_file_contents_from_archive(&path,"conf.lua")?;
        match get_version_from_file(&content) {
            Some(version) => return Ok(Project::File(version)),
            None => {
                // TODO : some kind of error handling, because we found a file but no conf.lua content??
            }
        }
    }

    if is_love_project_folder(&path) {
        let content = get_file_contents(&path,"conf.lua")?;

        match get_version_from_file(&content) {
            Some(version) => return Ok(Project::Folder(version)),
            None => {
                // TODO : some kind of error handling, because we found a file but no conf.lua content??
            }
        }
    }

    Ok(Project::None)
}

// PRIVATE FUNCTIONS ///////////////////////

fn get_version_from_file(file_content : &str) -> Option<Version> {
    let re_version_assignment = Regex::new(r#"version *= *["|'](.*)["|']"#).unwrap();

    if let Some(version) = re_version_assignment.captures(file_content) {
        let captured_version = version.get(1).unwrap().as_str().to_string();
        if let Some(version) = Version::from_str(&captured_version) {
            return Some(version);
        }
    }

    None
}

fn is_love_package(project_path : &PathBuf) -> bool {
    //! checks if the path given is a love package or a folder
    //! essentially just checks the extension

    let string : String = project_path.display().to_string();
    let split : Vec<&str> = string.split(".").collect();

    if split[split.len()-1] == "love" { 
        true
    } else {
        false
    }
}

fn is_love_project_folder(project_path : &PathBuf) -> bool {
    //! checks if the given folder is a love project. it determines
    //! this if it has a 'main.lua' in the root (all that it can really 
    //! check for)
    
    let mut main_lua_path = project_path.clone();
    main_lua_path.push("main.lua");

    main_lua_path.exists()
}

fn get_file_contents(project_path : &PathBuf,file_path : &str) -> Result<String,Error> {
    //! gets the contents of the file from a folder.

    let mut buf : String = String::new();

    // does this section if it a .love file
    if is_love_package(project_path) {
        return get_file_contents_from_archive(&project_path,&file_path);
    // if this is a directory
    } else {
        let mut project_file_path = project_path.clone();
        project_file_path.push(file_path);

        // loads the contents of the file if found, else it returns an error!!!
        let mut project_file = File::open(project_file_path)?;
        project_file.read_to_string(&mut buf)?;
    }

    if buf.len() > 0 { 
        Ok(buf)
    } else {
        Err(format_err!("Failed to read file's contents"))
    }
}

fn get_file_contents_from_archive(archive_path : &PathBuf, file_path_in_archive : &str) -> Result<String,Error> {
    //! gets the contents of a file that is in a .love package

    let mut content : String = "".to_string();
    let mut buf : Vec<u8> = Vec::new();

    let mut zip_file = File::open(&archive_path)?;
    zip_file.read_to_end(&mut buf)?;

    let mut archive = zip::ZipArchive::new(Cursor::new(buf))?;
    let mut file = archive.by_name(&file_path_in_archive)?;
    file.read_to_string(&mut content)?;

    if content.len() > 0 {
        Ok(content)
    } else {
        Err(format_err!("Failed to read file's contents"))
    }
}