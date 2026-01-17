use crate::domain::ports::ILLMService;
use ollama_rs::Ollama;
use ollama_rs::generation::completion::request::GenerationRequest;
use std::sync::Arc;
use tokio::runtime::Runtime;

pub struct OllamaAdapter {
    runtime: Arc<Runtime>,
    client: Ollama,
    model: std::sync::Mutex<String>,
}

impl OllamaAdapter {
    pub fn new(model: &str) -> Self {
        let runtime = Arc::new(Runtime::new().expect("Failed to create Tokio runtime"));
        let client = Ollama::default();
        Self {
            runtime,
            client,
            model: std::sync::Mutex::new(model.to_string()),
        }
    }

    fn ensure_server_running(&self) -> Result<(), String> {
        // Only run on Linux for now
        if cfg!(target_os = "linux") {
            // Check if listening on default port
             if std::net::TcpStream::connect("127.0.0.1:11434").is_err() {
                 println!("Ollama not running. Starting server...");
                 // Attempt to start
                 let _ = std::process::Command::new("ollama")
                     .arg("serve")
                     .spawn()
                     .map_err(|e| format!("Failed to auto-start ollama: {}. Is it installed?", e))?;
                 
                 // Wait for up to 5 seconds
                 for _ in 0..10 {
                     std::thread::sleep(std::time::Duration::from_millis(500));
                     if std::net::TcpStream::connect("127.0.0.1:11434").is_ok() {
                         return Ok(());
                     }
                 }
                 return Err("Timed out waiting for Ollama to auto-start. Check logs.".to_string());
             }
        }
        Ok(())
    }
}

impl ILLMService for OllamaAdapter {
    fn query(&self, prompt: &str, context: Option<String>) -> Result<String, String> {
        self.ensure_server_running()?;
        let model = self.model.lock().unwrap().clone();
        
        let mut full_prompt = prompt.to_string();
        if let Some(ctx) = context {
            full_prompt = format!(
                "Context: The following relevant files/locations were found on the user's system: {}\n\nUser Question: {}\n\nPlease use the context to provide a more accurate and helpful answer.",
                ctx, prompt
            );
        }

        let prompt_string = full_prompt;
        let client = self.client.clone();
        
        // Blocking call to async code
        self.runtime.block_on(async move {
            let request = GenerationRequest::new(model, prompt_string);
            match client.generate(request).await {
                Ok(res) => Ok(res.response),
                Err(e) => Err(format!("Ollama Error: {}", e)),
            }
        })
    }

    fn list_models(&self) -> Result<Vec<String>, String> {
        self.ensure_server_running()?;
        let client = self.client.clone();
        self.runtime.block_on(async move {
             match client.list_local_models().await {
                 Ok(models) => Ok(models.iter().map(|m| m.name.clone()).collect()),
                 Err(e) => Err(format!("Failed to list models: {}", e)),
             }
        })
    }

    fn pull_model(&self, model: &str, on_progress: Box<dyn Fn(f64) + Send>) -> Result<(), String> {
        self.ensure_server_running()?;
        let client = self.client.clone();
        let model_name = model.to_string();
        self.runtime.block_on(async move {
            use tokio_stream::StreamExt;
            let mut stream = client.pull_model_stream(model_name, false).await
                .map_err(|e| format!("Failed to download model: {}", e))?;
            
            while let Some(res) = stream.next().await {
                match res {
                    Ok(status) => {
                        if let (Some(total), Some(completed)) = (status.total, status.completed) {
                            if total > 0 {
                                let progress = completed as f64 / total as f64;
                                on_progress(progress);
                            }
                        }
                    }
                    Err(e) => return Err(format!("Download error: {}", e)),
                }
            }
            Ok(())
        })
    }

     fn delete_model(&self, model: &str) -> Result<(), String> {
          self.ensure_server_running()?;
          let client = self.client.clone();
          let model_name = model.to_string();
          self.runtime.block_on(async move {
              match client.delete_model(model_name).await {
                  Ok(_) => Ok(()),
                  Err(e) => Err(format!("Failed to delete model: {}", e)),
              }
          })
     }

     fn set_model(&self, model: &str) {
         let mut m = self.model.lock().unwrap();
         *m = model.to_string();
     }
 }
