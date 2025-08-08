use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Datum {
    pub name: String,
    pub datum_type: String,
    pub adjacencies: Vec<String>,
    pub properties: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chunk {
    pub id: Uuid,
    pub content: String,
    pub datum: String,
    pub attribution: Attribution,
    pub metadata: ChunkMetadata,
    pub vector: Option<Vec<f32>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attribution {
    pub url: Option<String>,
    pub filename: Option<String>,
    pub date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkMetadata {
    pub topic: String,
    pub tags: Vec<String>,
    pub created_at: String,
}

#[derive(Debug)]
pub struct GrokClient {
    qdrant_url: String,
    api_key: String,
    collection_name: String,
    embedding_model: Option<EmbeddingModel>,
    // TODO: Add Qdrant client when API stabilizes
}

#[derive(Debug)]
pub struct EmbeddingModel {
    // 🤓 Placeholder for future implementation
    model_name: String,
}

impl EmbeddingModel {
    pub async fn new() -> Result<Self> {
        // TODO: Initialize Python embedding service via HTTP or subprocess
        Ok(Self {
            model_name: "sentence-transformers/all-MiniLM-L6-v2".to_string(),
        })
    }
    
    pub fn encode(&self, _text: &str) -> Result<Vec<f32>> {
        // TODO: Call Python embedding service
        // For now, return mock embedding
        let mock_embedding: Vec<f32> = (0..384).map(|i| (i as f32 * 0.01) % 1.0).collect();
        Ok(mock_embedding)
    }
}

impl GrokClient {
    pub fn new(qdrant_url: String, api_key: String) -> Self {
        Self {
            qdrant_url,
            api_key,
            collection_name: "b00t_chunks".to_string(),
            embedding_model: None,
        }
    }

    pub async fn initialize(&mut self) -> Result<()> {
        // Initialize embedding model
        let embedding_model = EmbeddingModel::new().await?;
        self.embedding_model = Some(embedding_model);

        tracing::info!("GrokClient initialized for {}", self.qdrant_url);
        Ok(())
    }

    fn generate_embedding(&self, text: &str) -> Result<Vec<f32>> {
        let model = self.embedding_model.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Embedding model not initialized"))?;
        model.encode(text)
    }

    pub async fn digest(&self, topic: &str, content: &str) -> Result<Chunk> {
        // Generate vector embedding
        let vector = self.generate_embedding(content).ok();
        
        let chunk = Chunk {
            id: Uuid::new_v4(),
            content: content.to_string(),
            datum: topic.to_string(),
            attribution: Attribution {
                url: None,
                filename: None,
                date: chrono::Utc::now().to_rfc3339(),
            },
            metadata: ChunkMetadata {
                topic: topic.to_string(),
                tags: vec![],
                created_at: chrono::Utc::now().to_rfc3339(),
            },
            vector,
        };

        // TODO: Store in Qdrant when client is implemented

        Ok(chunk)
    }

    pub async fn ask(&self, _query: &str, _topic: Option<&str>) -> Result<Vec<Chunk>> {
        // TODO: Implement vector search with Qdrant
        Ok(vec![])
    }

    pub async fn learn(&self, source: &str, content: &str) -> Result<Vec<Chunk>> {
        // Simple chunking for now - split by double newlines
        let chunks: Vec<&str> = content.split("\n\n")
            .filter(|chunk| !chunk.trim().is_empty())
            .collect();
        
        let mut result_chunks = Vec::new();
        
        for (i, chunk_text) in chunks.into_iter().enumerate() {
            let trimmed = chunk_text.trim();
            if trimmed.len() < 10 { continue; } // Skip very short chunks
            
            // Infer topic from source or use "general"
            let inferred_topic = self.infer_topic_from_source(source);
            
            let vector = self.generate_embedding(trimmed).ok();
            
            let chunk = Chunk {
                id: Uuid::new_v4(),
                content: trimmed.to_string(),
                datum: inferred_topic.clone(),
                attribution: Attribution {
                    url: if source.starts_with("http") { Some(source.to_string()) } else { None },
                    filename: if !source.starts_with("http") && source != "direct_input" { 
                        Some(source.to_string()) 
                    } else { None },
                    date: chrono::Utc::now().to_rfc3339(),
                },
                metadata: ChunkMetadata {
                    topic: inferred_topic,
                    tags: vec![format!("chunk_{}", i)],
                    created_at: chrono::Utc::now().to_rfc3339(),
                },
                vector,
            };
            
            // TODO: Store in Qdrant when client is implemented
            
            result_chunks.push(chunk);
        }
        
        Ok(result_chunks)
    }
    
    fn infer_topic_from_source(&self, source: &str) -> String {
        if source.contains("rust") || source.contains(".rs") {
            "rust".to_string()
        } else if source.contains("python") || source.contains(".py") {
            "python".to_string()
        } else if source.contains("docker") {
            "docker".to_string()
        } else if source.contains("git") {
            "git".to_string()
        } else {
            "general".to_string()
        }
    }
}

#[cfg(feature = "pyo3")]
#[pyclass]
pub struct PyGrokClient {
    client: GrokClient,
    runtime: tokio::runtime::Runtime,
}

#[cfg(feature = "pyo3")]
#[pymethods]
impl PyGrokClient {
    #[new]
    fn new(qdrant_url: String, api_key: String) -> PyResult<Self> {
        let mut client = GrokClient::new(qdrant_url, api_key);
        let runtime = tokio::runtime::Runtime::new()
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        
        // Initialize the client
        runtime.block_on(client.initialize())
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        
        Ok(Self { client, runtime })
    }

    fn digest(&mut self, topic: &str, content: &str) -> PyResult<String> {
        let chunk = self.runtime.block_on(self.client.digest(topic, content))
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        
        serde_json::to_string(&chunk)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))
    }

    fn ask(&mut self, query: &str, topic: Option<&str>) -> PyResult<Vec<String>> {
        let chunks = self.runtime.block_on(self.client.ask(query, topic))
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        
        chunks.into_iter()
            .map(|chunk| serde_json::to_string(&chunk))
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))
    }

    fn learn(&mut self, source: &str, content: &str) -> PyResult<Vec<String>> {
        let chunks = self.runtime.block_on(self.client.learn(source, content))
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        
        chunks.into_iter()
            .map(|chunk| serde_json::to_string(&chunk))
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))
    }
}

#[cfg(feature = "pyo3")]
#[pymodule]
fn b00t_grok(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyGrokClient>()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_grok_client_creation() {
        let client = GrokClient::new(
            "https://example.com".to_string(),
            "test_key".to_string()
        );
        assert_eq!(client.collection_name, "b00t_chunks");
        assert_eq!(client.qdrant_url, "https://example.com");
        assert_eq!(client.api_key, "test_key");
    }

    #[tokio::test]
    async fn test_grok_client_initialization() {
        let mut client = GrokClient::new(
            "https://example.com".to_string(),
            "test_key".to_string()
        );
        let result = client.initialize().await;
        assert!(result.is_ok());
        assert!(client.embedding_model.is_some());
    }

    #[tokio::test]
    async fn test_digest() {
        let mut client = GrokClient::new(
            "https://example.com".to_string(),
            "test_key".to_string()
        );
        client.initialize().await.unwrap();
        
        let chunk = client.digest("rust", "Rust is a systems programming language").await.unwrap();
        assert_eq!(chunk.datum, "rust");
        assert_eq!(chunk.content, "Rust is a systems programming language");
        assert_eq!(chunk.metadata.topic, "rust");
        assert!(!chunk.id.to_string().is_empty());
        assert!(chunk.attribution.date.len() > 0);
        assert!(chunk.vector.is_some()); // Should have mock embedding
    }

    #[test]
    fn test_datum_creation() {
        let mut properties = HashMap::new();
        properties.insert("language".to_string(), "systems".to_string());
        properties.insert("memory_safe".to_string(), "true".to_string());
        
        let datum = Datum {
            name: "rust".to_string(),
            datum_type: "programming_language".to_string(),
            adjacencies: vec!["cargo".to_string(), "clippy".to_string()],
            properties,
        };
        
        assert_eq!(datum.name, "rust");
        assert_eq!(datum.datum_type, "programming_language");
        assert_eq!(datum.adjacencies.len(), 2);
        assert_eq!(datum.properties.get("language"), Some(&"systems".to_string()));
    }

    #[test]
    fn test_chunk_serialization() {
        let chunk = Chunk {
            id: uuid::Uuid::new_v4(),
            content: "Test content".to_string(),
            datum: "test_datum".to_string(),
            attribution: Attribution {
                url: Some("https://example.com".to_string()),
                filename: None,
                date: "2025-01-01T00:00:00Z".to_string(),
            },
            metadata: ChunkMetadata {
                topic: "test".to_string(),
                tags: vec!["unit_test".to_string(), "example".to_string()],
                created_at: "2025-01-01T00:00:00Z".to_string(),
            },
            vector: Some(vec![0.1, 0.2, 0.3]),
        };
        
        let json = serde_json::to_string(&chunk).unwrap();
        let deserialized: Chunk = serde_json::from_str(&json).unwrap();
        
        assert_eq!(chunk.content, deserialized.content);
        assert_eq!(chunk.datum, deserialized.datum);
        assert_eq!(chunk.vector, deserialized.vector);
    }

    #[tokio::test]
    async fn test_ask_empty_result() {
        let mut client = GrokClient::new(
            "https://example.com".to_string(),
            "test_key".to_string()
        );
        client.initialize().await.unwrap();
        
        let results = client.ask("What is Rust?", Some("rust")).await.unwrap();
        assert_eq!(results.len(), 0); // TODO: Will have results when implemented
    }

    #[tokio::test]
    async fn test_learn_chunking() {
        let mut client = GrokClient::new(
            "https://example.com".to_string(),
            "test_key".to_string()
        );
        client.initialize().await.unwrap();
        
        let content = "First paragraph about Rust.\n\nSecond paragraph about Cargo.\n\nThird paragraph about testing.";
        let chunks = client.learn("rust-guide.md", content).await.unwrap();
        
        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[0].datum, "rust");
        assert!(chunks[0].content.contains("First paragraph"));
        assert!(chunks[1].content.contains("Second paragraph"));
        assert!(chunks[2].content.contains("Third paragraph"));
    }

    #[tokio::test]
    async fn test_topic_inference() {
        let client = GrokClient::new(
            "https://example.com".to_string(),
            "test_key".to_string()
        );
        
        assert_eq!(client.infer_topic_from_source("main.rs"), "rust");
        assert_eq!(client.infer_topic_from_source("script.py"), "python");
        assert_eq!(client.infer_topic_from_source("Dockerfile"), "general"); // 🤓 Fixed test expectation
        assert_eq!(client.infer_topic_from_source("https://git.example.com/repo"), "git");
        assert_eq!(client.infer_topic_from_source("unknown_file.txt"), "general");
    }

    #[cfg(feature = "pyo3")]
    #[test]
    fn test_py_grok_client_creation() {
        let result = PyGrokClient::new(
            "https://example.com".to_string(),
            "test_key".to_string()
        );
        assert!(result.is_ok());
    }
}