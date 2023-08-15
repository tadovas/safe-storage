use safe_storage::client::Client;
use safe_storage::sha3::hash_content;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = Client::new("http://localhost:8080".to_string());

    let file_list = client.get_file_list().await?;
    println!("{:?}", file_list);

    let file = client
        .upload_new_file("file_1".to_string(), "some content".as_bytes())
        .await?;
    println!("Uploaded {} with id {}", file.name, file.id);

    let file_list = client.get_file_list().await?;
    println!("{:?}", file_list);

    let root_hash = client.fetch_root().await?.hash;
    println!("{root_hash}");

    let file = client.download_file(file.id).await?;
    println!(
        "file: {} with id: {} contains: [{}]",
        file.name,
        file.id,
        String::from_utf8_lossy(&file.content)
    );
    println!("{:?}", file.proof);

    println!(
        "Verified: {}",
        file.proof.verify(&root_hash, &hash_content(&file.content))
    );
    Ok(())
}
