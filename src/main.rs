use clap::{Command, Parser};
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

/// Takes an input file with paths to yaml files with env source values and writes them to output path
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// The path to the input file with the paths to the yaml
    #[clap(short = 'c', long = "config")]
    #[clap(parse(from_os_str))]
    config_path: std::path::PathBuf,
    /// The path to the output file
    #[clap(short = 'o', long = "output")]
    #[clap(parse(from_os_str))]
    output_path: std::path::PathBuf,
}

const CONFIG_READ_ERROR_MESSAGE: &str = "Could not read config file";
const NOT_YAML_FILE_PATH_ERROR_MESSAGE: &str = "All paths in config file must have .yaml extension";

/// Create an error for when yaml file could not be read at all
fn create_yaml_file_read_err(path: &PathBuf) -> String {
    return format!(
        "Could not read yaml file with path: {err_file_path}",
        err_file_path = path.clone().into_os_string().into_string().unwrap()
    );
}

/// Create an error for when yaml content has invalid format
fn create_yaml_content_validation_err(path: &PathBuf) -> String {
    return format!(
        "Unsupported yaml structure in file with path: {err_file_path}",
        err_file_path = path.clone().into_os_string().into_string().unwrap()
    );
}

/// Read all paths to the input yaml files from the config file
fn read_config_file(path: &PathBuf) -> Vec<PathBuf> {
    return std::fs::read_to_string(path)
        .expect(CONFIG_READ_ERROR_MESSAGE)
        .lines()
        .map(|l| PathBuf::from(l))
        .collect::<Vec<PathBuf>>();
}

/// Ensure all paths end in '.yaml'
fn assert_paths_are_yaml_files(
    paths: Vec<PathBuf>,
    cmd: &mut Command,
) -> Result<Vec<PathBuf>, clap::Error> {
    if paths.iter().all(|path| path.extension().unwrap() == "yaml") {
        return Ok(paths);
    } else {
        println!("{:?}", paths);
        let err: clap::Error = cmd.error(
            clap::ErrorKind::ValueValidation,
            NOT_YAML_FILE_PATH_ERROR_MESSAGE,
        );
        return Err(err);
    }
}

/// Read yaml files and add values to env hashmap
fn create_env_hashmap(
    paths: Vec<PathBuf>,
    cmd: &mut Command,
) -> Result<HashMap<String, String>, clap::Error> {
    let mut env_hash_map = HashMap::new();

    for path in paths.iter() {
        // read file
        let file = std::fs::read_to_string(path).expect(create_yaml_file_read_err(path).as_str());

        // read yaml lines and transform into collection of key value pairs
        let parsed_line_key_val_pairs = file.lines().map(|l| match l.split_once(':') {
            Some((key, value)) => Some((String::from(key), String::from(value))),
            None => None,
        });

        // throw err if any line could not be split into two on ':' char
        if parsed_line_key_val_pairs.clone().any(|res| res.is_none()) {
            let err: clap::Error = cmd.error(
                clap::ErrorKind::ValueValidation,
                create_yaml_content_validation_err(path),
            );
            return Err(err);
        }

        // add key value pairs to hashmap
        env_hash_map.extend(
            parsed_line_key_val_pairs.map(|res| res.unwrap_or((String::new(), String::new()))),
        )
    }

    Ok(env_hash_map)
}

/// Concert hashmap to string
fn convert_map_to_string(env_map: HashMap<String, String>) -> String {
    let mut output_string = String::new();
    env_map.iter().for_each(|(k, v)| {
        let formatted_key = k.trim();
        let formatted_value = v.trim();
        let line = format!("{}={}\n", formatted_key, formatted_value);
        output_string.push_str(line.as_str());
    });
    output_string
}

fn write_env_file(output_path: &PathBuf, output_content: &str) -> std::io::Result<()> {
    let mut file = File::create(output_path)?;
    file.write_all(output_content.as_bytes())?;
    Ok(())
}

fn main() {
    let mut cmd: Command = Command::new("YAML to .env");
    let args = Args::parse();
    let input_paths = read_config_file(&args.config_path);
    let yaml_file_paths = assert_paths_are_yaml_files(input_paths, &mut cmd).unwrap();
    let env_map = create_env_hashmap(yaml_file_paths, &mut cmd).unwrap();
    let output_string = convert_map_to_string(env_map);
    let res = write_env_file(&args.output_path, &output_string);
    match res {
        Ok(_) => println!("Env file created succesfully."),
        Err(err) => println!("Error when trying to write env file: {}", err),
    }
}
