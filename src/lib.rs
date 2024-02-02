// Global Types
use serde_json::json;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Bar {
    origin: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Message {
    role: String,
    content: String,
    images: Option<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
enum Response {
    Done {
        model: String,
        created_at: String,
        eval_count: usize,
        done: bool,
    },

    Data {
        model: String,
        created_at: String,
        message: Message,
        done: bool,
    },
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
enum ResponseRaw {
    Done {
        model: String,
        created_at: chrono::DateTime<chrono::Local>,
        eval_count: usize,
        done: bool,
    },

    Data {
        model: String,
        created_at: chrono::DateTime<chrono::Local>,
        response: String,
        done: bool,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures_util::StreamExt;

    #[tokio::test]
    async fn test_ollama_chat_endpoint() -> anyhow::Result<()> {
        let schema = ::serde_json::to_string_pretty(&json!({
            "name": "add_two_numbers",
            "description": "Adds two numbers together.",
            "signature": "(first_number: int, second_number: int) -> int",
            "output": "<class 'int'>"
        }))?;

        let query0 = "How is the weather in Hawaii right now in International units?";
        let schema0 = ::serde_json::to_string_pretty(&json!({
            "name": "get_weather",
            "description": "Useful to get the weather in a specific location",
            "signature": "(location: str, degree: str) -> str",
            "output": "<class 'str'>"
        }))?;
        let resp0 = ::serde_json::to_string_pretty(&json!({
            "name": "get_weather",
            "data": {
                "location": "London",
                "degree": "Celsius"
            }
        }))?;

        let query = "Add 2 and 5";
        let prompt = format!(
            "\
            <s>[INST] You are a helpful assistant designed to output JSON.\n\
            Given the following function schema\n\
            < {schema} >\n\
            and query\n\
            < {query} >\n\
            extract the parameters values from the query, in a valid JSON format.\n\
            Example:\n\
            Input:\n\
            query: \"{query0}\"\n\
            schema: {schema0} [/INST] \n\
            Result: {resp0}</s> \n\
            [INST] Input:\n\
            query: {query}\n\
            schema: {schema} [/INST] \n\
            Result: \
            "
        );

        use colored::Colorize;
        let p_prompt = format!("TEMPLATE ```\n{prompt}\n```")
            .bright_yellow()
            .dimmed();
        let p_query = format!(">>> {query}").yellow();
        println!("{p_prompt}\n\n{p_query}",);

        let stream = reqwest::Client::new()
            .post("http://0.0.0.0:11434/api/generate")
            .json(&json!({
                "model": "mistral:instruct",
                "prompt": prompt
            }))
            .send()
            .await?
            .bytes_stream();

        let mut stream = json_stream::JsonStream::<ResponseRaw, _>::new(stream);

        println!("{}", "\n\n==Response==\n".dimmed());
        while let Some(Ok(item)) = stream.next().await {
            match item {
                ResponseRaw::Done { .. } => (),
                ResponseRaw::Data { response, .. } => print!("{response}"),
            }
            // println!("Chunk: {:?}", item.unwrap());
        }
        println!("{}", "\n\n==End of Response==".dimmed());

        Ok(())
    }

    // Implementation of mock
    // ollama client for tests
    // #[tokio::test]
    async fn test_ollama_api() {
        let line = "The quick brown fox jumps over a lazy dog";
        let mut s = line.split_inclusive(' ');

        let data = core::iter::from_fn(|| {
            Some(Ok::<_, std::io::Error>(
                (json!({
                  "model": "llama2",
                  "created_at": "2023-08-04T08:52:19.385406455-07:00",
                  "message": {
                    "role": "assistant",
                    "content": s.next()?,
                  },
                  "done": false
                })
                .to_string()
                    + "\n")
                    .into_bytes(),
            ))
        })
        .chain(core::iter::once(Ok((json!({
          "model": "llama2",
          "created_at": "2023-08-04T19:22:45.499127Z",
          "eval_count": 282,
          "done": true,
        })
        .to_string()
            + "\n")
            .into_bytes())));

        let stream = futures_util::stream::iter(data);

        let mut stream = json_stream::JsonStream::<Response, _>::new(stream);

        while let Some(foo) = stream.next().await {
            println!("{:?}", foo);
        }
    }

    // #[tokio::test]
    async fn test_stream() {
        let stream = reqwest::get("http://httpbin.org/ip")
            .await
            .unwrap()
            .bytes_stream();

        let mut stream = json_stream::JsonStream::<Bar, _>::new(stream);

        while let Some(item) = stream.next().await {
            println!("Chunk: {:?}", item.unwrap());
        }
    }

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}

// let (B_FUNC, E_FUNC) = ("<FUNCTIONS>", "</FUNCTIONS>");
// let (B_INST, E_INST) = ("[INST]", "[/INST]");
// let (B_SYS, E_SYS) = ("<s>", "</s>");
//
// let f = serde_json::json!({
//     "function": "search_bing",
//     "description": "Search the web for content on Bing. This allows users to search online/the internet/the web for content.",
//     "arguments": [
//         {
//             "name": "query",
//             "type": "string",
//             "description": "The search query string"
//         }
//     ]
// });
//
// let system_prompt = "You have access to the following functions. Use them if required:";
// let user_prompt = "Search for the latest news on AI.";
//
// let prompt = format!("{B_FUNC}{f}{E_FUNC}\n\n{B_INST} {B_SYS}\n{system_prompt}\n{E_SYS}\n\n{user_prompt} {E_INST}\n\n");
