use anyhow::Result;
use clap::Subcommand;
use std::env;

#[derive(Subcommand, Clone)]
pub enum GrokCommands {
    /// Digest content into chunks about a topic
    Digest {
        /// Topic to digest content about
        #[arg(short, long)]
        topic: String,
        /// Content to digest
        content: String,
    },
    /// Ask questions and search the knowledgebase
    Ask {
        /// Query to search for
        query: String,
        /// Optional topic to filter by
        #[arg(short, long)]
        topic: Option<String>,
    },
    /// Learn from URLs or files
    Learn {
        /// Source URL or file path
        #[arg(short, long)]
        source: Option<String>,
        /// Content to learn from
        content: String,
    },
}

pub fn handle_grok_command(command: GrokCommands) -> Result<()> {
    match command {
        GrokCommands::Digest { topic, content } => {
            handle_digest(&topic, &content)
        }
        GrokCommands::Ask { query, topic } => {
            handle_ask(&query, topic.as_deref())
        }
        GrokCommands::Learn { source, content } => {
            handle_learn(source.as_deref(), &content)
        }
    }
}

fn handle_digest(topic: &str, content: &str) -> Result<()> {
    let qdrant_url = get_qdrant_url()?;
    let api_key = get_qdrant_api_key()?;
    
    // Create PyGrokClient and call digest
    let python_code = format!(
        r#"
import json
from b00t_grok import PyGrokClient

client = PyGrokClient("{}", "{}")
chunk_json = client.digest("{}", "{}")
chunk = json.loads(chunk_json)

print(f"✅ Digested chunk for topic '{}':")
print(f"📄 ID: {{chunk['id']}}")
print(f"💬 Content: {{chunk['content'][:100]}}...")
print(f"📅 Created: {{chunk['metadata']['created_at']}}")
"#,
        qdrant_url,
        api_key,
        topic.replace('"', r#"\""#),
        content.replace('"', r#"\""#).replace('\n', r#"\n"#),
        topic
    );
    
    let output = std::process::Command::new("python3")
        .arg("-c")
        .arg(python_code)
        .output()?;
    
    if output.status.success() {
        print!("{}", String::from_utf8_lossy(&output.stdout));
    } else {
        eprintln!("❌ Error: {}", String::from_utf8_lossy(&output.stderr));
    }
    
    Ok(())
}

fn handle_ask(query: &str, topic: Option<&str>) -> Result<()> {
    let qdrant_url = get_qdrant_url()?;
    let api_key = get_qdrant_api_key()?;
    
    let python_code = format!(
        r#"
from b00t_grok import PyGrokClient

client = PyGrokClient("{}", "{}")
results = client.ask("{}", {})

print(f"🔍 Query: '{}'")
print(f"📊 Found {{len(results)}} results")

for i, result_json in enumerate(results):
    print(f"{{i+1}}. {{result_json[:100]}}...")
"#,
        qdrant_url,
        api_key,
        query.replace('"', r#"\""#),
        if let Some(t) = topic {
            format!(r#""{}""#, t.replace('"', r#"\""#))
        } else {
            "None".to_string()
        },
        query
    );
    
    let output = std::process::Command::new("python3")
        .arg("-c")
        .arg(python_code)
        .output()?;
    
    if output.status.success() {
        print!("{}", String::from_utf8_lossy(&output.stdout));
    } else {
        eprintln!("❌ Error: {}", String::from_utf8_lossy(&output.stderr));
    }
    
    Ok(())
}

fn handle_learn(source: Option<&str>, content: &str) -> Result<()> {
    let qdrant_url = get_qdrant_url()?;
    let api_key = get_qdrant_api_key()?;
    
    let source_str = source.unwrap_or("direct_input");
    
    let python_code = format!(
        r#"
from b00t_grok import PyGrokClient

client = PyGrokClient("{}", "{}")
chunks = client.learn("{}", "{}")

print(f"📚 Learning from source: '{}'")
print(f"📦 Generated {{len(chunks)}} chunks")

for i, chunk_json in enumerate(chunks):
    print(f"{{i+1}}. Chunk created")
"#,
        qdrant_url,
        api_key,
        source_str.replace('"', r#"\""#),
        content.replace('"', r#"\""#).replace('\n', r#"\n"#),
        source_str
    );
    
    let python_executable = get_python_executable()?;
    let output = std::process::Command::new(&python_executable)
        .arg("-c")
        .arg(python_code)
        .output()?;
    
    if output.status.success() {
        print!("{}", String::from_utf8_lossy(&output.stdout));
    } else {
        eprintln!("❌ Error: {}", String::from_utf8_lossy(&output.stderr));
    }
    
    Ok(())
}

fn get_python_executable() -> Result<String> {
    // 🤓 Get the correct Python executable from current environment (uv venv compatible)
    let python_path = std::process::Command::new("python3")
        .arg("-c")
        .arg("import sys; print(sys.executable)")
        .output()?
        .stdout;
    
    Ok(String::from_utf8(python_path)?.trim().to_string())
}

fn get_qdrant_url() -> Result<String> {
    // First try from _b00t_.toml session config
    if let Ok(session_path) = env::var("PWD") {
        let toml_path = format!("{}/.git/_b00t_.toml", session_path);
        if let Ok(contents) = std::fs::read_to_string(toml_path) {
            if let Ok(config) = toml::from_str::<toml::Value>(&contents) {
                if let Some(url) = config
                    .get("qdrant")
                    .and_then(|q| q.get("url"))
                    .and_then(|u| u.as_str()) {
                    return Ok(url.to_string());
                }
            }
        }
    }
    
    // Fallback to hardcoded URL
    // Fallback to environment variable
    if let Ok(url) = env::var("QDRANT_URL") {
        return Ok(url);
    }
    Err(anyhow::anyhow!("Qdrant URL not found in config or QDRANT_URL environment variable"))
}

fn get_qdrant_api_key() -> Result<String> {
    env::var("QDRANT_API_KEY")
        .map_err(|_| anyhow::anyhow!("QDRANT_API_KEY environment variable not set"))
}