use safe_storage::client::Client;
use safe_storage::sha3::hash_content;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = Client::new("http://localhost:8080".to_string());

    let file = client
        .upload_new_file("file_1".to_string(), "some content".as_bytes())
        .await?;
    println!("Uploaded {} with id {}", file.name, file.id);

    // alternative - client can construct its own merkle tree and take root hash from there to be independent from server
    let root_hash = client.fetch_root().await?.hash;
    println!("Root hash: {root_hash}");

    let file_list = client.get_file_list().await?;

    for file in file_list.files {
        let file = client.download_file(file.id).await?;
        let verified = file.proof.verify(&root_hash, &hash_content(&file.content));
        println!(
            "{}. {} downloaded. Size: {} Verified: {}",
            file.id,
            file.name,
            file.content.len(),
            verified
        );
    }

    Ok(())
}
