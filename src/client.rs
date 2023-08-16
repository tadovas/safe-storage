use crate::api::{File, FileContent, FileList, NewFile, RootHash};
use anyhow::anyhow;
use reqwest::Response;
use serde::de::DeserializeOwned;
use serde::Serialize;

pub struct Client {
    api_base: String,
    client: reqwest::Client,
}

impl Client {
    pub fn new(api_base: String) -> Self {
        Self {
            api_base,
            client: reqwest::Client::new(),
        }
    }

    pub async fn get_file_list(&self) -> anyhow::Result<FileList> {
        let url = format!("{}/files", self.api_base);
        self.get(url).await
    }

    pub async fn upload_new_file(&self, filename: &str, content: &[u8]) -> anyhow::Result<File> {
        let url = format!("{}/files", self.api_base);
        self.post(
            url,
            NewFile {
                content: content.to_vec(),
                name: filename.to_string(),
            },
        )
        .await
    }

    pub async fn download_file(&self, id: u32) -> anyhow::Result<FileContent> {
        let url = format!("{}/files/{}", self.api_base, id);
        self.get(url).await
    }

    pub async fn fetch_root(&self) -> anyhow::Result<RootHash> {
        let url = format!("{}/root", self.api_base);
        self.get(url).await
    }

    async fn get<R: DeserializeOwned>(&self, url: String) -> anyhow::Result<R> {
        let resp = self.client.get(&url).send().await?;
        check_response(resp).await
    }

    async fn post<B: Serialize, R: DeserializeOwned>(
        &self,
        url: String,
        body: B,
    ) -> anyhow::Result<R> {
        let resp = self.client.post(&url).json(&body).send().await?;
        check_response(resp).await
    }
}

async fn check_response<T: DeserializeOwned>(resp: Response) -> anyhow::Result<T> {
    if !resp.status().is_success() {
        let code = resp.status();
        let text = resp.text().await?;
        return Err(anyhow!("http error: {} body: {}", code, text));
    }
    let res = resp.json().await?;
    Ok(res)
}
