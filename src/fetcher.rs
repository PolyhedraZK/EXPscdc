use std::{
    collections::BTreeMap,
    fs::{self, create_dir_all, remove_file},
    io::Write,
    path::PathBuf,
    thread::sleep,
    time::Duration,
};

use anyhow::{anyhow, Result};
use base64::{engine::general_purpose::STANDARD, Engine};
use flate2::write::GzDecoder;
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
    async fn get_data(&self) -> Result<BTreeMap<String, Vec<u8>>> {
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
            if let Ok((tx_hash, data)) = decode_tx(tx) {
                ret.insert(tx_hash, data);
            }
        }
        Ok(ret)
    }

    async fn save_data(&self, hash: &str, data: &[u8]) -> Result<()> {
        let hash = hash.strip_prefix("0x").unwrap_or(hash);

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

fn decode_tx(tx: &str) -> Result<(String, Vec<u8>)> {
    let tx_bytes = STANDARD.decode(tx)?;
    let tx = serde_json::from_slice::<Value>(&tx_bytes)?;
    if Some("blob") != tx["type"].as_str() {
        return Err(anyhow!("type error"));
    }
    let data = tx["body"]["data"]
        .as_str()
        .ok_or(anyhow!("data not found"))?;

    let data = hex::decode(data)?;

    let tx_hash = hex::encode(Sha256::digest(&tx_bytes));

    let mut e = GzDecoder::new(Vec::new());
    e.write_all(&data)?;
    let data = e.finish()?;

    Ok((tx_hash, data))
}
