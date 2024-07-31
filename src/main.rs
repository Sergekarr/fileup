use clap::Parser;
//use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use std::path::Path;
use std::process::Command;

#[derive(Debug, Parser)]
#[command(version, about)]
struct Cli {
    path: String,
    #[arg(short, help = "Creates a torrent file for downloading")]
    torrent: bool,
    #[arg(
        short,
        help = "Expiry time (e.g., 1H (1 Hour), 1D (1 Day), 3D (3 Days), 1W (1 Week), 1M (1 Month))"
    )]
    expiry: Option<String>,
    #[arg(short, help = "Custom name for the download file")]
    filename: Option<String>,
}

fn parse_expiry(expiry: &Option<String>) -> Option<u64> {
    expiry.as_ref().map(|expiry_str| {
        let (num, unit) = expiry_str.split_at(expiry_str.len() - 1);
        let num: u64 = num.parse().expect("Failed to parse number");

        match (num, unit) {
            (1, "H") => 3600,
            (1, "D") => 86400,
            (3, "D") => 259200,
            (1, "W") => 604800,
            (1, "M") => 2592000,
            _ => {
                eprintln!("Invalid expiry time! (e.g., 1H, 1D, 3D, 1W, 1M)");
                std::process::exit(1);
            }
        }
    })
}

fn main() {
    let cli = Cli::parse();
    let path_arg = if cli.path.is_empty() {
        panic!("Provide a file PATH");
    } else {
        cli.path
    };

    let path = Path::new(&path_arg);
    println!("Path: {:?}", path);

    let file_name = path
        .file_name()
        .expect("Invalid file path")
        .to_str()
        .expect("File name is not valid UTF-8");
    println!("File name: {}", file_name);

    let encoded_file_name = utf8_percent_encode(file_name, NON_ALPHANUMERIC).to_string();
    let torrent = if cli.torrent { "1" } else { "" };
    use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};

    let upload_url = if let Some(custom_name) = &cli.filename {
        let encoded_custom_name = utf8_percent_encode(custom_name, NON_ALPHANUMERIC).to_string();
        format!("https://filehaus.su/api/upload/{}", encoded_custom_name)
    } else {
        format!("https://filehaus.su/api/upload/{}", encoded_file_name)
    };
    let mut command = Command::new("curl");
    command
        // .arg("-s")
        .arg("-X")
        .arg("PUT")
        .arg("-H")
        .arg("Content-Type: application/octet-stream")
        .arg("-H")
        .arg(format!("X-Torrent: {}", torrent));

    if let Some(expiry) = parse_expiry(&cli.expiry) {
        command.arg("-H").arg(format!("X-Expire-After: {}", expiry));
    }

    command
        .arg("--data-binary")
        .arg(format!("@{}", path.to_str().unwrap()))
        .arg(&upload_url);

    println!("Executing command: {:?}", command);

    let output = command.output().expect("Failed to execute command");

    println!(
        "Link expires in {:?}",
        cli.expiry.as_ref().map_or("No expiry set", |expiry| expiry)
    );

    let stdout_str = String::from_utf8_lossy(&output.stdout);
    if cli.torrent {
        println!(
            "\nURL: {} \nTorrent: {}",
            &stdout_str[..stdout_str.len() - 8],
            stdout_str
        );
    } else {
        println!("URL: {}", &stdout_str)
    };
    println!("stderr: {}", String::from_utf8_lossy(&output.stderr));
}
