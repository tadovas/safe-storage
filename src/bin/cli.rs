use anyhow::anyhow;
use clap::{ArgAction, Parser, Subcommand};
use safe_storage::client::Client;
use safe_storage::merkle;
use safe_storage::sha3::hash_content;
use serde::{Deserialize, Serialize};
use tokio::io::AsyncWriteExt;

/// A simple command line interface to interact with safe-storage server (must be already running)
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct CmdArgs {
    #[arg(long, default_value = "http://localhost:8080")]
    server_url: String,
    #[arg(short, long, default_value = ".state.json")]
    state_file: String,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Upload one or more files to the server, storing calculated merkle root hash in local state
    Upload {
        /// file list to upload
        #[arg(action = ArgAction::Append)]
        files: Vec<String>,
    },
    /// List all files available on server
    List,
    /// Download any file by given id from the list automatically verifying integrity with proof
    /// from server and merkle root from local storage
    Download {
        /// file id to download
        id: u32,
        /// optionally specify under which name to save file content, otherwise original name will be used
        #[arg(long, value_name = "FILENAME")]
        save_as: Option<String>,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cmd_args = CmdArgs::parse();
    match cmd_args.command {
        Command::Download { id, save_as } => {
            download_file(cmd_args.server_url, cmd_args.state_file, id, save_as).await
        }
        Command::Upload { files } => {
            upload_files(cmd_args.server_url, cmd_args.state_file, files).await
        }
        Command::List => list_all_files(cmd_args.server_url).await,
    }
}

async fn list_all_files(server_url: String) -> anyhow::Result<()> {
    let client = Client::new(server_url);
    let files = client.get_file_list().await?;
    for file in files.files {
        println!("{}: {}", file.id, file.name);
    }
    Ok(())
}

async fn upload_files(
    server_url: String,
    state_filename: String,
    files: Vec<String>,
) -> anyhow::Result<()> {
    let client = Client::new(server_url);
    let mut light_tree = load_state(state_filename.clone()).await?.light_tree;
    if files.is_empty() {
        println!("Nothing to upload");
        return Ok(());
    }
    for file in files {
        let content = tokio::fs::read(&file).await?;
        light_tree.append(hash_content(&content));
        let new_file = client.upload_new_file(&file, &content).await?;
        println!("{file} uploaded with id: {}", new_file.id);
    }

    let local_hash = light_tree
        .root()
        .expect("should be present if at least one file was uploaded");
    let remote_hash = client.fetch_root().await?.hash;
    println!("Local  hash: {local_hash}");
    println!("Remote hash: {remote_hash}");
    if local_hash != remote_hash {
        println!("Local root hash differs from remote hash - multiple uploads detected, which is not supported yet. Verification won't work");
        println!("Service restart is required to clean the state")
    }

    store_state(state_filename, LocalState { light_tree }).await
}

async fn download_file(
    server_url: String,
    state_filename: String,
    id: u32,
    save_as: Option<String>,
) -> anyhow::Result<()> {
    let light_tree = load_state(state_filename).await?.light_tree;
    let client = Client::new(server_url);
    let file = client.download_file(id).await?;
    let file_hash = hash_content(&file.content);
    let verified = file
        .proof
        .verify(&light_tree.root().expect("must be present"), &file_hash);
    if !verified {
        return Err(anyhow!("Verification failed!"));
    }
    println!("File contents verified");
    let path = save_as.unwrap_or(file.name);
    tokio::fs::write(&path, &file.content).await?;
    println!("File {id} saved as {path}");
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
struct LocalState {
    light_tree: merkle::Sha3LightTree,
}

async fn load_state(filename: String) -> anyhow::Result<LocalState> {
    let content = tokio::fs::read_to_string(filename).await?;
    Ok(serde_json::from_str(&content)?)
}

async fn store_state(filename: String, state: LocalState) -> anyhow::Result<()> {
    let serialized = serde_json::ser::to_vec_pretty(&state)?;
    let mut file = tokio::fs::File::create(filename).await?;
    file.write_all(&serialized).await?;
    Ok(())
}
