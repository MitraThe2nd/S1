use serde::Deserialize;
use serde_yaml::from_reader;
use reqwest::Client;
use tokio::task;
use std::fs::File;
use std::error::Error;

#[derive(Debug, Deserialize)]
struct Configuration {
    rpc_endpoint: String,
    account_list: Vec<String>,
}

async fn fetch_balance(
    http_client: &Client,
    rpc_endpoint: &str,
    account: &str,
) -> Result<u64, Box<dyn Error + Send + Sync>> {
    let endpoint = format!("{}/", rpc_endpoint);
    let response = http_client.post(&endpoint)
        .json(&serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getBalance",
            "params": [account],
        }))
        .send()
        .await?;

    let parsed: serde_json::Value = response.json().await?;
    let balance = parsed["result"]["value"].as_u64()
        .ok_or("Error: Balance not found in response")?;
    Ok(balance)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Чтение конфигурации
    let configuration_file = File::open("config.yaml")?;
    let config: Configuration = from_reader(configuration_file)?;

    // Инициализация HTTP клиента
    let http_client = Client::new();

    // Параллельное получение балансов
    let mut tasks = Vec::new();
    for account in &config.account_list {
        let http_client_clone = http_client.clone();
        let rpc_endpoint_clone = config.rpc_endpoint.clone();
        let account_clone = account.clone();

        tasks.push(task::spawn(async move {
            fetch_balance(&http_client_clone, &rpc_endpoint_clone, &account_clone).await
        }));
    }

    // Сбор результатов
    for task in tasks {
        match task.await? {
            Ok(balance) => println!("Баланс: {} лямпортов", balance),
            Err(err) => println!("Ошибка: {}", err),
        }
    }

    Ok(())
}
