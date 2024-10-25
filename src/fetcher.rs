use std::{
    collections::BTreeMap,
    fs::{self, create_dir_all, remove_file},
    path::PathBuf,
    thread::sleep,
    time::Duration,
};

use anyhow::{anyhow, Result};
use base64::{engine::general_purpose::STANDARD, Engine};
use serde_json::Value;
use sha2::{Digest, Sha256};

const INDEX_FILE: &str = "index";
pub struct Fetcher {
    path: PathBuf,
    url: String,
    height: u64,
}
impl Fetcher {
    pub fn new(path: PathBuf, url: &str) -> Result<Self> {
        let height = if path.exists() {
            let index_file = path.join(INDEX_FILE);
            if index_file.exists() {
                let content = fs::read_to_string(&index_file)?;
                u64::from_str_radix(&content, 10)?
            } else {
                1
            }
        } else {
            create_dir_all(&path)?;
            1
        };

        Ok(Self {
            path,
            url: url.to_string(),
            height,
        })
    }
    pub async fn run(&mut self) -> Result<()> {
        loop {
            log::debug!("get block {}", self.height);
            match self.get_data().await {
                Ok(data) => {
                    for (hash, data) in data.iter() {
                        self.save_data(hash, data).await?;
                    }
                    self.height += 1;
                    fs::write(self.path.join(INDEX_FILE), format!("{}", self.height))?;
                }
                Err(e) => {
                    log::error!("fetch block {} error: {}", self.height, e);
                    sleep(Duration::from_secs(10));
                }
            }
        }
    }
    async fn get_data(&self) -> Result<BTreeMap<String, String>> {
        let url = format!("{}/block?height={}", self.url, self.height);

        let resp: Value = reqwest::get(url).await?.json::<Value>().await?;

        let txs = resp["result"]["block"]["data"]["txs"]
            .as_array()
            .and_then(|txs| txs.iter().map(|tx| tx.as_str()).collect::<Option<Vec<_>>>())
            .ok_or(anyhow!(format!(
                "block {} get txs error:{}",
                self.height, resp
            )))?;

        let mut ret = BTreeMap::new();
        log::debug!("get {} tx:{:?}", self.height, txs);
        for tx in txs {
            if let Ok((tx_bytes, tx)) =
                STANDARD
                    .decode(tx)
                    .map_err(|e| anyhow!("{}", e))
                    .and_then(|tx_bytes| {
                        serde_json::from_slice::<Value>(&tx_bytes)
                            .map(|tx| (tx_bytes, tx))
                            .map_err(|e| anyhow!("{}", e))
                    })
            {
                if Some("blob") == tx["type"].as_str() {
                    let tx_hash = hex::encode(Sha256::digest(&tx_bytes));
                    if let Some(v) = tx["body"]["data"].as_str() {
                        ret.insert(tx_hash, v.to_string());
                    }
                }
            }
        }
        Ok(ret)
    }

    async fn save_data(&self, hash: &str, data: &str) -> Result<()> {
        let hash = hash.strip_prefix("0x").unwrap_or(hash);
        let data = data.strip_prefix("0x").unwrap_or(data);

        let path = self.path.join(hash[0..4].to_string());
        if !path.exists() {
            create_dir_all(&path)?;
        }
        let file = path.join(hash);
        if file.exists() {
            remove_file(&file)?;
        }
        fs::write(file, data)?;

        Ok(())
    }
}
